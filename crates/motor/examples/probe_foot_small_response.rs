//! FOOT-CONTROL-01: bounded local plant response, not a training controller.

#[cfg(feature = "physx-sdk")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use next_contracts::ids::PersistentId;
    use next_motor::{BiomechanicsSafetyController, CompiledBodySchemaV4, JointControlStateV1};
    use next_physics_physx::CanonicalPhysXSnapshotV2;
    use serde_json::json;

    if std::env::args().len() != 1 {
        return Err("no arguments: exact FOOT-CONTROL-01 census".into());
    }
    let body = next_motor::biomechanics_humanoid_body_schema_v8();
    let compiled = CompiledBodySchemaV4::compile(&body, PersistentId::from_bytes([0; 16]))?;
    let base = &compiled.base.base;
    let safety = BiomechanicsSafetyController::new(base)?;
    let encode = |state: &CanonicalPhysXSnapshotV2| {
        json!({
            "joints":state.joints.iter().map(|j|json!({"ordinal":j.ordinal,"position_urad":j.position_microradians,
                "velocity_urad_s":j.velocity_microradians_per_second})).collect::<Vec<_>>(),
            "links":state.links.iter().map(|l|json!({"body_token":l.user_token,"position_um":l.position_micrometres,
                "rotation_q1_30":l.rotation_q1_30,"angular_velocity_urad_s":l.angular_velocity_microradians_per_second,
                "linear_velocity_um_s":l.linear_velocity_micrometres_per_second})).collect::<Vec<_>>(),
            "contacts":state.contacts.iter().map(|c|json!({"actors":[c.actor_a_token,c.actor_b_token],
                "shapes":[c.shape_a_token,c.shape_b_token],"position_um":c.position_micrometres,
                "normal_q1_30":c.normal_q1_30,"impulse_uns":c.impulse_micronewton_seconds,
                "separation_um":c.separation_micrometres})).collect::<Vec<_>>()
        })
    };
    let mut fixture = compiled.create_world()?;
    let mut reset = fixture.raw_checkpoint();
    reset.links[0].position_bits[1] =
        (f32::from_bits(reset.links[0].position_bits[1]) + 10.0).to_bits();
    let mut changed = 0;
    for (joint, dof) in &base.joint_dof_ordinals {
        if joint.as_str().ends_with("-knee") || joint.as_str().ends_with("-elbow") {
            reset.joints[*dof as usize].position_bits = 0.1_f32.to_bits();
            changed += 1;
        }
    }
    assert_eq!(changed, 4);
    let initial = fixture.restore(&reset)?;
    let mut inputs = vec![(None, 0_i64), (None, 0_i64)];
    for magnitude in [10_000_i64, 20_000] {
        for dof in 0..25_u32 {
            for sign in [-1, 1] {
                inputs.push((Some(dof), sign * magnitude));
            }
        }
    }
    let mut trials = Vec::new();
    for (dof, effort) in inputs {
        let mut world = compiled.create_world()?;
        assert_eq!(world.restore(&reset)?, initial);
        let mut applied = [0; 25];
        if let Some(dof) = dof {
            applied[dof as usize] = effort;
        }
        for (channel, ordinal) in base
            .actuator_definitions
            .iter()
            .zip(&base.actuator_dof_ordinals)
        {
            let value = applied[*ordinal as usize];
            assert!(
                (channel.minimum_effort_micronewton_metres
                    ..=channel.maximum_effort_micronewton_metres)
                    .contains(&value)
            );
            // Strict floor is conservative relative to ties-even production rate.
            assert!(
                value.unsigned_abs()
                    <= channel.maximum_effort_rate_micronewton_metres_per_second / 240
            );
        }
        let after = world.apply_efforts_and_step(&applied)?;
        let states = base
            .actuator_dof_ordinals
            .iter()
            .map(|dof| {
                let joint = after
                    .joints
                    .iter()
                    .find(|j| j.ordinal == *dof)
                    .expect("compiled DOF");
                JointControlStateV1 {
                    position_microradians: joint.position_microradians,
                    velocity_microradians_per_second: joint.velocity_microradians_per_second,
                }
            })
            .collect::<Vec<_>>();
        let observed = safety
            .validate_observed_joint_states(&states)
            .map_or_else(|e| e.stable_code(), |()| "valid");
        trials.push(
            json!({"dof":dof,"effort_unm":effort,"applied_efforts_by_dof_unm":applied,
            "observed_safety":observed,"after":encode(&after)}),
        );
    }
    println!(
        "{}",
        serde_json::to_string(&json!({"probe":"FOOT-CONTROL-01.v1",
        "body_schema_hash":body.schema_hash()?.to_hex(),"compiled_descriptor_hash":compiled.compiled_descriptor_hash.to_hex(),
        "physics_hz":240,"position_iterations":base.physx_scene_profile.position_iterations,
        "initial":encode(&initial),"trials":trials}))?
    );
    Ok(())
}

#[cfg(not(feature = "physx-sdk"))]
fn main() {
    panic!("requires --features physx-sdk");
}
