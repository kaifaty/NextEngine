#[cfg(feature = "physx-sdk")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use next_contracts::ids::PersistentId;
    use next_contracts::motor::MotorTerminalDispositionV1;
    use next_motor::{
        BiomechanicsContactClassifier, BiomechanicsProceduralStandingControllerV1,
        BiomechanicsSafetyController, BiomechanicsSkillContactProfileV1,
        BiomechanicsTerminalEvaluator, CompiledBodySchemaV3, JointControlStateV1,
    };
    use next_physics_physx::{CanonicalPhysXSnapshotV2, PhysXArticulationWorldV3};
    use serde_json::json;

    let revision = std::env::args()
        .nth(1)
        .ok_or("expected body revision 5 or 6")?;
    let schema = match revision.as_str() {
        "5" => next_motor::biomechanics_humanoid_body_schema_v5(),
        "6" => next_motor::biomechanics_humanoid_body_schema_v6(),
        _ => return Err("expected body revision 5 or 6".into()),
    };
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let base = &compiled.base;
    let mut world =
        PhysXArticulationWorldV3::create(base.physx_scene_profile, &compiled.physx_catalog)?;
    let mut snapshot = world.capture()?;
    let mut safety = BiomechanicsSafetyController::new(base)?;
    let mut classifier = BiomechanicsContactClassifier::new(base)?;
    let mut terminal = BiomechanicsTerminalEvaluator::new(
        base,
        BiomechanicsSkillContactProfileV1::Locomotion,
        1_800,
    )?;
    // Existing controller: only knee/ankle reference targets are nonzero.
    // No controller tuning, learned weights or disabled safety in this probe.
    let standing = BiomechanicsProceduralStandingControllerV1::new(base, &snapshot)?;
    let states = |snapshot: &CanonicalPhysXSnapshotV2| {
        base.actuator_dof_ordinals
            .iter()
            .map(|dof| {
                let joint = snapshot
                    .joints
                    .iter()
                    .find(|s| s.ordinal == *dof)
                    .expect("compiled dof");
                JointControlStateV1 {
                    position_microradians: joint.position_microradians,
                    velocity_microradians_per_second: joint.velocity_microradians_per_second,
                }
            })
            .collect::<Vec<_>>()
    };
    let sample = |tick: u64, substeps: u64, snapshot: &CanonicalPhysXSnapshotV2| {
        json!({
            "motor_tick": tick, "physics_substeps": substeps,
            "links": snapshot.links.iter().map(|l| json!({"body_token": l.user_token,
                "position_um": l.position_micrometres, "rotation_q1_30": l.rotation_q1_30,
                "linear_velocity_um_s": l.linear_velocity_micrometres_per_second})).collect::<Vec<_>>()
        })
    };
    let mut samples = vec![sample(0, 0, &snapshot)];
    let mut reason = String::from("incomplete");
    let mut substeps = 0_u64;
    let envelopes = safety.default_skill_envelopes();
    'episode: for tick in 1..=1_800_u64 {
        let reference = standing.reference_targets(&snapshot)?;
        safety.begin_motor_tick(&reference, &[0; 23], &envelopes)?;
        let mut contacts = Vec::with_capacity(4);
        for _ in 0..4 {
            let efforts = safety.step_substep(&states(&snapshot))?;
            let mut applied = [0; 23];
            for (effort, &dof) in efforts.iter().zip(&base.actuator_dof_ordinals) {
                applied[dof as usize] = effort.effort_micronewton_metres;
            }
            snapshot = world.apply_efforts_and_step(&applied)?;
            substeps += 1;
            if let Err(error) = safety.validate_observed_joint_states(&states(&snapshot)) {
                reason = format!("joint-safety: {error}");
                samples.push(sample(tick, substeps, &snapshot));
                break 'episode;
            }
            contacts.push(
                classifier
                    .classify_substep(&snapshot, BiomechanicsSkillContactProfileV1::Locomotion)?,
            );
        }
        let decision = terminal.evaluate_motor_tick(tick, &snapshot, &contacts, false, None)?;
        samples.push(sample(tick, substeps, &snapshot));
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
        serde_json::to_string(&json!({
            "schema_version": 1, "probe": "native-body-standing-control-v1",
            "body_schema_hash": base.body_schema_hash.to_hex(),
            "compiled_descriptor_hash": compiled.compiled_descriptor_hash.to_hex(),
            "body_revision": schema.schema_revision, "reason": reason,
            "requested_motor_ticks": 1_800, "physics_substeps": substeps,
            "scope": "nominal procedural standing diagnostic, not learned quality",
            "samples": samples,
        }))?
    );
    Ok(())
}

#[cfg(not(feature = "physx-sdk"))]
fn main() {
    panic!("probe_biomechanics_body_standing requires --features physx-sdk");
}
