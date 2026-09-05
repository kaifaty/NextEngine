//! BODY-BANDWIDTH-01 r2 / BODY-COMBINED-01 r1, not a balance/training run.

fn offset(tick: u64, sine: bool) -> i64 {
    if !(61..=180).contains(&tick) {
        return 0;
    }
    if sine {
        let phase = (tick - 61) as f64 * std::f64::consts::TAU / 60.0;
        (25_000.0 * phase.sin()).round_ties_even() as i64
    } else {
        50_000
    }
}

fn input_offset(tick: u64, case: u32, dof: u32) -> i64 {
    let sign = if case == 27 && matches!(dof, 1 | 8) {
        -1
    } else {
        1
    };
    sign * offset(tick, case == 26)
}

fn parse_args(args: &[String]) -> Result<(u32, bool), &'static str> {
    let (version, combined) = match args {
        [v] => (v, false),
        [v, mode] if mode == "abduction-step" => (v, true),
        _ => return Err("expected 8|11 [abduction-step]"),
    };
    match version.as_str() {
        "8" => Ok((8, combined)),
        "11" => Ok((11, combined)),
        _ => Err("expected exact body revision 8 or 11"),
    }
}

#[cfg(feature = "physx-sdk")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use next_contracts::ids::PersistentId;
    use next_motor::{
        BiomechanicsContactClassV1, BiomechanicsContactClassifier, BiomechanicsSafetyController,
        BiomechanicsSkillContactProfileV1, CompiledBodySchemaV4, JointControlStateV1,
    };
    use next_physics_physx::CanonicalPhysXSnapshotV2;
    use serde_json::json;
    use std::io::{BufWriter, Write};

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (version, combined) = parse_args(&args)?;
    let body = match version {
        8 => next_motor::biomechanics_humanoid_body_schema_v8(),
        11 => next_motor::biomechanics_humanoid_body_schema_v11(),
        _ => unreachable!("validated revision"),
    };
    let compiled = CompiledBodySchemaV4::compile(&body, PersistentId::from_bytes([0; 16]))?;
    let base = &compiled.base.base;
    let encode = |s: &CanonicalPhysXSnapshotV2| {
        json!({
            "joints":s.joints.iter().map(|j|json!({"ordinal":j.ordinal,
                "position_urad":j.position_microradians,"velocity_urad_s":j.velocity_microradians_per_second})).collect::<Vec<_>>(),
            "links":s.links.iter().map(|l|json!({"body_token":l.user_token,"position_um":l.position_micrometres,
                "rotation_q1_30":l.rotation_q1_30,"angular_velocity_urad_s":l.angular_velocity_microradians_per_second,
                "linear_velocity_um_s":l.linear_velocity_micrometres_per_second})).collect::<Vec<_>>(),
            "contacts":s.contacts.iter().map(|c|json!({"actors":[c.actor_a_token,c.actor_b_token],
                "shapes":[c.shape_a_token,c.shape_b_token],"position_um":c.position_micrometres,
                "normal_q1_30":c.normal_q1_30,"impulse_uns":c.impulse_micronewton_seconds,
                "separation_um":c.separation_micrometres})).collect::<Vec<_>>()
        })
    };
    let states = |s: &CanonicalPhysXSnapshotV2| {
        base.actuator_dof_ordinals
            .iter()
            .map(|dof| {
                let j = s
                    .joints
                    .iter()
                    .find(|j| j.ordinal == *dof)
                    .expect("compiled DOF");
                JointControlStateV1 {
                    position_microradians: j.position_microradians,
                    velocity_microradians_per_second: j.velocity_microradians_per_second,
                }
            })
            .collect::<Vec<_>>()
    };
    let mut fixture = compiled.create_world()?;
    let mut reset = fixture.raw_checkpoint();
    reset.links[0].position_bits[1] =
        (f32::from_bits(reset.links[0].position_bits[1]) + 100.0).to_bits();
    for (joint, dof) in &base.joint_dof_ordinals {
        if joint.as_str().ends_with("-knee") || joint.as_str().ends_with("-elbow") {
            reset.joints[*dof as usize].position_bits = 0.1_f32.to_bits();
        }
    }
    let initial = fixture.restore(&reset)?;
    let reset_reference = states(&initial)
        .iter()
        .map(|s| s.position_microradians)
        .collect::<Vec<_>>();
    let mut output = BufWriter::new(std::io::stdout().lock());
    let cases = if combined { 27..28_u32 } else { 0..27_u32 };
    writeln!(
        output,
        "{}",
        json!({"kind":"header","probe":if combined {"BODY-COMBINED-01.r1"} else {"BODY-BANDWIDTH-01.r2"},
        "body_revision":body.schema_revision,"body_schema_hash":body.schema_hash()?.to_hex(),
        "compiled_descriptor_hash":compiled.compiled_descriptor_hash.to_hex(),"physics_hz":240,
        "position_iterations":base.physx_scene_profile.position_iterations,"trial_count":cases.len(),
        "ordered_actuator_ids":base.actuator_definitions.iter().map(|a|a.actuator_id.as_str()).collect::<Vec<_>>(),
        "actuator_dof_ordinals":base.actuator_dof_ordinals,"initial":encode(&initial)})
    )?;
    for case in cases {
        let selected_dof = (case < 25).then_some(case);
        let sine = case == 26;
        let mut world = compiled.create_world()?;
        let mut state = world.restore(&reset)?;
        assert_eq!(state, initial);
        let mut safety = BiomechanicsSafetyController::new(base)?;
        safety.validate_observed_joint_states(&states(&state))?;
        safety.reset_to_reference(&reset_reference)?;
        let reset_safety_root = safety.checkpoint_root().to_hex();
        let envelopes = safety.default_skill_envelopes();
        let mut classifier = BiomechanicsContactClassifier::new(base)?;
        let mut ticks = Vec::new();
        let mut frames = Vec::new();
        let mut reason = "horizon".to_owned();
        'trial: for tick in 1..=240_u64 {
            let reference = reset_reference
                .iter()
                .zip(&base.actuator_dof_ordinals)
                .map(|(q, dof)| {
                    q + if selected_dof.is_none_or(|selected| selected == *dof) {
                        input_offset(tick, case, *dof)
                    } else {
                        0
                    }
                })
                .collect::<Vec<_>>();
            let targets = match safety.begin_motor_tick(&reference, &[0; 25], &envelopes) {
                Ok(t) => t,
                Err(e) => {
                    reason = format!("target: {e}");
                    break 'trial;
                }
            };
            ticks.push(json!({"motor_tick":tick,"reference_targets_urad":reference,
                "applied_targets_urad":targets.iter().map(|t|t.target_microradians).collect::<Vec<_>>(),
                "target_flags":targets.iter().map(|t|t.clamp_flags).collect::<Vec<_>>() }));
            for sub in 0..4 {
                let efforts = match safety.step_substep(&states(&state)) {
                    Ok(e) => e,
                    Err(e) => {
                        reason = format!("controller: {e}");
                        break 'trial;
                    }
                };
                let mut applied = [0; 25];
                for (e, dof) in efforts.iter().zip(&base.actuator_dof_ordinals) {
                    applied[*dof as usize] = e.effort_micronewton_metres;
                }
                state = world.apply_efforts_and_step(&applied)?;
                frames.push(json!({"physics_substep":(tick-1)*4+sub+1,
                    "applied_efforts_by_dof_unm":applied,
                    "effort_flags":efforts.iter().map(|e|e.clamp_flags).collect::<Vec<_>>(),"after":encode(&state)}));
                if let Err(e) = safety.validate_observed_joint_states(&states(&state)) {
                    reason = format!("joint-safety: {e}");
                    break 'trial;
                }
                if state
                    .contacts
                    .iter()
                    .any(|c| c.actor_a_token == 1 || c.actor_b_token == 1)
                {
                    reason = "diagnostic.ground-contact".to_owned();
                    break 'trial;
                }
                let contact = classifier
                    .classify_substep(&state, BiomechanicsSkillContactProfileV1::Locomotion)?;
                if contact.contacts.iter().any(|c| {
                    c.hard_impact_violation
                        || matches!(
                            c.class,
                            BiomechanicsContactClassV1::SelfCollisionViolation
                                | BiomechanicsContactClassV1::ForbiddenLocomotion
                        )
                }) {
                    reason = "diagnostic.forbidden-contact".to_owned();
                    break 'trial;
                }
            }
        }
        writeln!(
            output,
            "{}",
            json!({"kind":"trial","case":case,"selected_dof":selected_dof,
            "sine":sine,"reset_safety_root":reset_safety_root,"reason":reason,"ticks":ticks,"frames":frames})
        )?;
        output.flush()?;
    }
    Ok(())
}

#[cfg(not(feature = "physx-sdk"))]
fn main() {
    let _ = input_offset(0, 0, 0);
    let _ = parse_args(&[]);
    panic!("requires --features physx-sdk");
}

#[cfg(test)]
mod tests {
    #[test]
    fn combined_mode_is_explicit_and_changes_only_two_step_signs() {
        for tick in 0..=241 {
            for dof in 0..25 {
                for case in 0..27 {
                    assert_eq!(
                        super::input_offset(tick, case, dof),
                        super::offset(tick, case == 26)
                    );
                }
                let sign = if matches!(dof, 1 | 8) { -1 } else { 1 };
                assert_eq!(
                    super::input_offset(tick, 27, dof),
                    sign * super::offset(tick, false)
                );
            }
        }
        for version in ["8", "11"] {
            let v = version.parse::<u32>().unwrap();
            assert_eq!(super::parse_args(&[version.into()]), Ok((v, false)));
            assert_eq!(
                super::parse_args(&[version.into(), "abduction-step".into()]),
                Ok((v, true))
            );
        }
        for args in [
            vec![],
            vec!["9".into()],
            vec!["11".into(), "unknown".into()],
            vec!["11".into(), "abduction-step".into(), "extra".into()],
        ] {
            assert!(super::parse_args(&args).is_err());
        }
    }

    #[test]
    fn frozen_pulses_have_exact_windows_and_bounded_slew() {
        for tick in 0..=241 {
            assert_eq!(
                super::offset(tick, false),
                if (61..=180).contains(&tick) {
                    50_000
                } else {
                    0
                }
            );
            assert!(super::offset(tick, true).abs() <= 25_000);
            if tick > 0 {
                assert!((super::offset(tick, true) - super::offset(tick - 1, true)).abs() <= 2614);
            }
        }
        for tick in [0, 60, 61, 91, 121, 151, 181, 240] {
            assert_eq!(super::offset(tick, true), 0);
        }
        assert_eq!(super::offset(76, true), 25_000);
        assert_eq!(super::offset(106, true), -25_000);
    }
}
