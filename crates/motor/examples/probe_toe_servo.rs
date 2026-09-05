//! FOOT-SERVO-01: unchanged-body contact-free/grounded diagnostic, not balance.

fn pulse(tick: u64) -> i64 {
    match tick {
        1..=15 => (tick * 50_000 / 15) as i64,
        16..=30 => 50_000,
        31..=45 => ((45 - tick) * 50_000 / 15) as i64,
        _ => 0,
    }
}

#[cfg(feature = "physx-sdk")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use next_contracts::ids::PersistentId;
    use next_motor::{
        BiomechanicsProceduralStandingControllerV3, BiomechanicsSafetyController,
        CompiledBodySchemaV4, JointControlStateV1,
    };
    use next_physics_physx::CanonicalPhysXSnapshotV2;
    use serde_json::json;

    if std::env::args().len() != 1 {
        return Err("no arguments: runs the six frozen FOOT-SERVO-01 cases".into());
    }
    let body = next_motor::biomechanics_humanoid_body_schema_v8();
    let compiled = CompiledBodySchemaV4::compile(&body, PersistentId::from_bytes([0; 16]))?;
    let base = &compiled.base.base;
    let encode = |state: &CanonicalPhysXSnapshotV2| {
        json!({
            "joints":state.joints.iter().map(|j|json!({"ordinal":j.ordinal,
                "position_urad":j.position_microradians,"velocity_urad_s":j.velocity_microradians_per_second})).collect::<Vec<_>>(),
            "links":state.links.iter().map(|l|json!({"body_token":l.user_token,"position_um":l.position_micrometres,
                "rotation_q1_30":l.rotation_q1_30,"angular_velocity_urad_s":l.angular_velocity_microradians_per_second,
                "linear_velocity_um_s":l.linear_velocity_micrometres_per_second})).collect::<Vec<_>>(),
            "contacts":state.contacts.iter().map(|c|json!({"actors":[c.actor_a_token,c.actor_b_token],
                "shapes":[c.shape_a_token,c.shape_b_token],"position_um":c.position_micrometres,
                "normal_q1_30":c.normal_q1_30,"impulse_uns":c.impulse_micronewton_seconds,
                "separation_um":c.separation_micrometres})).collect::<Vec<_>>()
        })
    };
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
    let mut trials = Vec::new();
    for height in [0_i64, 10_000_000] {
        for side in ["none", "left", "right"] {
            let mut world = compiled.create_world()?;
            let original = world.capture()?;
            let mut raw = world.raw_checkpoint();
            let root_y = f32::from_bits(raw.links[0].position_bits[1]);
            raw.links[0].position_bits[1] = (root_y + height as f32 / 1_000_000.0).to_bits();
            let mut state = world.restore(&raw)?;
            assert_eq!(original.joints, state.joints);
            for (old, new) in original.links.iter().zip(&state.links) {
                assert_eq!(old.user_token, new.user_token);
                assert_eq!(old.rotation_q1_30, new.rotation_q1_30);
                assert_eq!(
                    old.linear_velocity_micrometres_per_second,
                    new.linear_velocity_micrometres_per_second
                );
                assert_eq!(
                    old.angular_velocity_microradians_per_second,
                    new.angular_velocity_microradians_per_second
                );
                for axis in 0..3 {
                    assert!(
                        (new.position_micrometres[axis]
                            - old.position_micrometres[axis]
                            - if axis == 1 { height } else { 0 })
                        .abs()
                            <= 2
                    );
                }
            }
            let initial = encode(&state);
            let standing = BiomechanicsProceduralStandingControllerV3::new(&compiled, &state)?;
            let mut safety = BiomechanicsSafetyController::new(base)?;
            let envelopes = safety.default_skill_envelopes();
            let mut frames = Vec::new();
            let mut reason = "horizon".to_owned();
            'episode: for tick in 1..=60_u64 {
                let mut reference = standing.reference_targets(&state)?;
                let mut selected = 0;
                for (target, actuator) in reference.iter_mut().zip(&base.actuator_definitions) {
                    if actuator.joint_id.as_str() == format!("joint.{side}-mtp") {
                        *target += pulse(tick);
                        selected += 1;
                    }
                }
                assert_eq!(selected, usize::from(side != "none"));
                let targets = safety.begin_motor_tick(&reference, &[0; 25], &envelopes)?;
                for substep in 0..4 {
                    let before = states(&state);
                    let efforts = match safety.step_substep(&before) {
                        Ok(value) => value,
                        Err(error) => {
                            reason = format!("controller: {error}");
                            break 'episode;
                        }
                    };
                    let mut applied = [0; 25];
                    let mut requested = Vec::new();
                    for (index, (effort, dof)) in
                        efforts.iter().zip(&base.actuator_dof_ordinals).enumerate()
                    {
                        applied[*dof as usize] = effort.effort_micronewton_metres;
                        requested.push(json!({"actuator":effort.actuator_id.as_str(),"dof":dof,
                            "target_urad":targets[index].target_microradians,"target_flags":targets[index].clamp_flags,
                            "input_position_urad":before[index].position_microradians,
                            "input_velocity_urad_s":before[index].velocity_microradians_per_second,
                            "effort_unm":effort.effort_micronewton_metres,"effort_flags":effort.clamp_flags}));
                    }
                    state = world.apply_efforts_and_step(&applied)?;
                    frames.push(json!({"physics_substep":(tick-1)*4+substep+1,"motor_tick":tick,
                        "reference_targets_urad":reference,"channels":requested,"after":encode(&state)}));
                    if let Err(error) = safety.validate_observed_joint_states(&states(&state)) {
                        reason = format!("joint-safety: {error}");
                        break 'episode;
                    }
                }
            }
            trials.push(json!({"height_um":height,"side":side,"initial":initial,
                "standing_reset_root":standing.state_root().to_hex(),"reason":reason,"frames":frames}));
        }
    }
    println!(
        "{}",
        serde_json::to_string(&json!({"probe":"FOOT-SERVO-01.v1","physics_hz":240,
        "body_schema_hash":body.schema_hash()?.to_hex(),"compiled_descriptor_hash":compiled.compiled_descriptor_hash.to_hex(),
        "trials":trials}))?
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
    fn frozen_input_has_bounded_slew_and_returns_to_zero() {
        for tick in 0..=60 {
            assert!((0..=50_000).contains(&super::pulse(tick)));
        }
        for tick in 1..=60 {
            assert!((super::pulse(tick) - super::pulse(tick - 1)).abs() <= 3334);
        }
        for tick in 0..=15 {
            assert_eq!(super::pulse(tick), super::pulse(45 - tick));
        }
        assert_eq!(super::pulse(15), 50_000);
        assert_eq!(super::pulse(30), 50_000);
        assert_eq!(super::pulse(45), 0);
        assert_eq!(super::pulse(60), 0);
    }
}
