//! Bounded diagnostic input, not a standing/reference or training profile change.

fn pulse(tick: u64) -> i64 {
    match tick {
        0..=300 => 0,
        301..=360 => ((tick - 300) * 100_000 / 60) as i64,
        361..=480 => 100_000,
        481..=540 => ((540 - tick) * 100_000 / 60) as i64,
        _ => 0,
    }
}

#[cfg(feature = "physx-sdk")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use next_contracts::ids::PersistentId;
    use next_contracts::motor::MotorTerminalDispositionV1;
    use next_motor::{
        BiomechanicsContactClassifierV2, BiomechanicsProceduralStandingControllerV3,
        BiomechanicsSafetyController, BiomechanicsSkillContactProfileV1,
        BiomechanicsTerminalEvaluator, CompiledBodySchemaV4, JointControlStateV1,
    };
    use next_physics_physx::CanonicalPhysXSnapshotV2;
    use serde_json::json;

    let mut args: Vec<_> = std::env::args().skip(1).collect();
    let sampled_damping = args.last().is_some_and(|arg| arg == "--body-v9");
    let screened_damping = args.last().is_some_and(|arg| arg == "--body-v10");
    if sampled_damping || screened_damping {
        args.pop();
    }
    if args.is_empty()
        || args.len() > 2
        || !matches!(args[0].as_str(), "none" | "left" | "right")
        || (args.len() == 2 && args[1] != "neutral-toe")
    {
        return Err("expected side none/left/right and optional neutral-toe".into());
    }
    let neutral_toe = args.len() == 2;
    let side = &args[0];
    let body = if screened_damping {
        next_motor::biomechanics_humanoid_body_schema_v10()
    } else if sampled_damping {
        next_motor::biomechanics_humanoid_body_schema_v9()
    } else {
        next_motor::biomechanics_humanoid_body_schema_v8()
    };
    let compiled = CompiledBodySchemaV4::compile(&body, PersistentId::from_bytes([0; 16]))?;
    let base = &compiled.base.base;
    let mut world = compiled.create_world()?;
    let mut state = world.capture()?;
    let standing = if screened_damping {
        BiomechanicsProceduralStandingControllerV3::new_screened_damping(&compiled, &state)?
    } else if sampled_damping {
        BiomechanicsProceduralStandingControllerV3::new_sampled_damping(&compiled, &state)?
    } else {
        BiomechanicsProceduralStandingControllerV3::new(&compiled, &state)?
    };
    let mut safety = BiomechanicsSafetyController::new(base)?;
    let mut contacts = if screened_damping {
        BiomechanicsContactClassifierV2::new_screened_damping(&compiled)?
    } else if sampled_damping {
        BiomechanicsContactClassifierV2::new_sampled_damping(&compiled)?
    } else {
        BiomechanicsContactClassifierV2::new(&compiled)?
    };
    let terminal_constructor = if screened_damping {
        BiomechanicsTerminalEvaluator::new_screened_damping
    } else if sampled_damping {
        BiomechanicsTerminalEvaluator::new_sampled_damping
    } else {
        BiomechanicsTerminalEvaluator::new_articulated
    };
    let mut terminal = terminal_constructor(
        &compiled,
        BiomechanicsSkillContactProfileV1::Locomotion,
        900,
    )?;
    let envelopes = safety.default_skill_envelopes();
    let states = |snapshot: &CanonicalPhysXSnapshotV2| {
        base.actuator_dof_ordinals
            .iter()
            .map(|dof| {
                let j = snapshot
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
    let mut frames = Vec::new();
    let mut reason = "incomplete".to_owned();
    'episode: for tick in 1..=900_u64 {
        let mut reference = standing.reference_targets(&state)?;
        let offset = if side == "none" { 0 } else { pulse(tick) };
        let mut modified = 0;
        for (target, actuator) in reference.iter_mut().zip(&base.actuator_definitions) {
            if actuator.joint_id.as_str() == format!("joint.{side}-ankle-pitch")
                || (!neutral_toe && actuator.joint_id.as_str() == format!("joint.{side}-mtp"))
            {
                *target += offset;
                modified += 1;
            }
        }
        assert_eq!(
            modified,
            if side == "none" {
                0
            } else if neutral_toe {
                1
            } else {
                2
            }
        );
        let targets = safety
            .begin_motor_tick(&reference, &[0; 25], &envelopes)?
            .iter()
            .map(|t| t.target_microradians)
            .collect::<Vec<_>>();
        let mut contact_frames = Vec::new();
        for substep in 0..4 {
            let efforts = safety.step_substep(&states(&state))?;
            let mut applied = [0; 25];
            for (effort, dof) in efforts.iter().zip(&base.actuator_dof_ordinals) {
                applied[*dof as usize] = effort.effort_micronewton_metres;
            }
            state = world.apply_efforts_and_step(&applied)?;
            let classified =
                contacts.classify_substep(&state, BiomechanicsSkillContactProfileV1::Locomotion)?;
            frames.push(json!({
                "motor_tick":tick,"physics_substep":(tick-1)*4+substep+1,"offset_urad":offset,
                "reference_targets_urad":reference,"applied_targets_urad":targets,"applied_efforts_by_dof_unm":applied,
                "joints":state.joints.iter().map(|j|json!({"ordinal":j.ordinal,"position_urad":j.position_microradians,"velocity_urad_s":j.velocity_microradians_per_second})).collect::<Vec<_>>(),
                "links":state.links.iter().map(|l|json!({"body_token":l.user_token,"position_um":l.position_micrometres,"rotation_q1_30":l.rotation_q1_30,
                    "angular_velocity_urad_s":l.angular_velocity_microradians_per_second,"linear_velocity_um_s":l.linear_velocity_micrometres_per_second})).collect::<Vec<_>>(),
                "contacts":state.contacts.iter().map(|c|json!({"actors":[c.actor_a_token,c.actor_b_token],"shapes":[c.shape_a_token,c.shape_b_token],
                    "position_um":c.position_micrometres,"normal_q1_30":c.normal_q1_30,"impulse_uns":c.impulse_micronewton_seconds,"separation_um":c.separation_micrometres})).collect::<Vec<_>>(),
                "anatomical_foot_impulses_uns":contacts.foot_impulses_micronewton_seconds(),
                "classification_root":classified.classification_root.to_hex(),
                "continuity_root":classified.continuity_root.to_hex(),
            }));
            contact_frames.push(classified);
            if let Err(error) = safety.validate_observed_joint_states(&states(&state)) {
                reason = format!("joint-safety: {error}");
                break 'episode;
            }
        }
        let decision = terminal.evaluate_motor_tick(tick, &state, &contact_frames, false, None)?;
        if decision.disposition != MotorTerminalDispositionV1::Running {
            reason = decision
                .reason
                .map_or("unknown", |r| r.stable_id())
                .to_owned();
            break;
        }
    }
    println!(
        "{}",
        serde_json::to_string(
            &json!({"schema_version":1,"probe":if neutral_toe {"nextengine.articulated-foot-transfer.v2"} else {"nextengine.articulated-foot-transfer.v1"},"side":side,
        "body_schema_hash":body.schema_hash()?.to_hex(),"compiled_descriptor_hash":compiled.compiled_descriptor_hash.to_hex(),
        "standing_reset_root":standing.state_root().to_hex(),"contact_profile_hash":standing.contact_profile_hash().to_hex(),
        "physics_hz":240,"motor_hz":60,"requested_motor_ticks":900,"reason":reason,"frames":frames})
        )?
    );
    Ok(())
}

#[cfg(not(feature = "physx-sdk"))]
fn main() {
    let _ = pulse(0);
    panic!("requires --features physx-sdk");
}

#[cfg(test)]
mod tests {
    #[test]
    fn pulse_is_bounded_symmetric_and_zero_outside_one_transfer() {
        for tick in 0..=900 {
            assert!((0..=100_000).contains(&super::pulse(tick)));
        }
        for offset in 0..=60 {
            assert_eq!(super::pulse(300 + offset), super::pulse(540 - offset));
        }
        assert_eq!(super::pulse(300), 0);
        assert_eq!(super::pulse(360), 100_000);
        assert_eq!(super::pulse(480), 100_000);
        assert_eq!(super::pulse(540), 0);
        assert_eq!(super::pulse(900), 0);
        for tick in 1..=900 {
            assert!((super::pulse(tick) - super::pulse(tick - 1)).abs() <= 1667);
        }
    }
}
