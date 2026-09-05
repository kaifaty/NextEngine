#[cfg(feature = "physx-sdk")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use next_contracts::ids::PersistentId;
    use next_contracts::motor::MotorTerminalDispositionV1;
    use next_motor::{
        BiomechanicsContactClassifier, BiomechanicsProceduralStandingControllerV1,
        BiomechanicsSafetyController, BiomechanicsSkillContactProfileV1,
        BiomechanicsTerminalEvaluator, CompiledBodySchemaV4, JointControlStateV1,
    };
    use next_physics_physx::{CanonicalPhysXSnapshotV2, PhysXArticulationWorldV3};
    use serde_json::json;

    let revision = std::env::args()
        .nth(1)
        .ok_or("expected body revision 5 or 6")?;
    let ankle_offset: i64 = std::env::args().nth(2).map_or(Ok(0), |s| s.parse())?;
    let hip_offset: i64 = std::env::args().nth(3).map_or(Ok(0), |s| s.parse())?;
    let reference_mode = std::env::args()
        .nth(4)
        .unwrap_or_else(|| "baseline".to_owned());
    let hip_feedback_gain = match reference_mode.as_str() {
        "baseline" | "neutral-targets" => 0,
        "hip-feedback" => 2,
        "hip-feedback-4" => 4,
        _ => {
            return Err(
                "mode must be baseline, neutral-targets, hip-feedback or hip-feedback-4".into(),
            );
        }
    };
    let per_iteration = match std::env::args().nth(5).as_deref() {
        None => false,
        Some("per-iteration") => true,
        _ => return Err("optional force schedule must be per-iteration".into()),
    };
    let actuator_probe = std::env::args()
        .nth(6)
        .unwrap_or_else(|| "unchanged".to_owned());
    if !matches!(
        actuator_probe.as_str(),
        "unchanged" | "shoulder-yaw-near-passive" | "shoulder-yaw-gain-16"
    ) {
        return Err(
            "actuator probe must be unchanged, shoulder-yaw-near-passive or shoulder-yaw-gain-16"
                .into(),
        );
    }
    if !(-140_000..=140_000).contains(&ankle_offset)
        || !(0..=150_000).contains(&hip_offset)
        || std::env::args().len() > 7
    {
        return Err("offset bounds: ankle +/-140000; hip 0..150000 microradians".into());
    }
    let schema = match revision.as_str() {
        "5" => next_motor::biomechanics_humanoid_body_schema_v5(),
        "6" => next_motor::biomechanics_humanoid_body_schema_v6(),
        _ => return Err("expected body revision 5 or 6".into()),
    };
    let schema = if actuator_probe != "unchanged" {
        shoulder_yaw_discriminator(schema, &actuator_probe)?
    } else {
        schema
    };
    let successor = CompiledBodySchemaV4::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let compiled = &successor.base;
    let base = &compiled.base;
    let mut world = if per_iteration {
        successor.create_world()?
    } else {
        PhysXArticulationWorldV3::create(base.physx_scene_profile, &compiled.physx_catalog)?
    };
    let mut snapshot = world.capture()?;
    let mut safety = BiomechanicsSafetyController::new(base)?;
    let mut classifier = BiomechanicsContactClassifier::new(base)?;
    let mut terminal = BiomechanicsTerminalEvaluator::new(
        base,
        BiomechanicsSkillContactProfileV1::Locomotion,
        1_800,
    )?;
    // Existing controller: only knee/ankle reference targets are nonzero.
    // Optional ankle/hip counterfactuals, before unchanged safety. These are
    // diagnostic candidates, not selected production reference profiles.
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
    let sample = |tick: u64,
                  substeps: u64,
                  snapshot: &CanonicalPhysXSnapshotV2,
                  reference: &[i64],
                  applied: &[i64]| {
        json!({
            "motor_tick": tick, "physics_substeps": substeps,
            "reference_targets_urad": reference, "applied_targets_urad": applied,
            "joints": snapshot.joints.iter().map(|j| json!({"ordinal": j.ordinal,
                "position_urad": j.position_microradians, "velocity_urad_s": j.velocity_microradians_per_second})).collect::<Vec<_>>(),
            "links": snapshot.links.iter().map(|l| json!({"body_token": l.user_token,
                "position_um": l.position_micrometres, "rotation_q1_30": l.rotation_q1_30,
                "angular_velocity_urad_s": l.angular_velocity_microradians_per_second,
                "linear_velocity_um_s": l.linear_velocity_micrometres_per_second})).collect::<Vec<_>>()
        })
    };
    let mut samples = vec![sample(0, 0, &snapshot, &[], &[])];
    let mut reason = String::from("incomplete");
    let mut substeps = 0_u64;
    let mut substep_samples = Vec::new();
    let envelopes = safety.default_skill_envelopes();
    'episode: for tick in 1..=1_800_u64 {
        let mut reference = standing.reference_targets(&snapshot)?;
        let root = snapshot
            .links
            .iter()
            .find(|l| l.user_token == base.physx_catalog.links[0].user_token)
            .expect("root");
        let feedback = if hip_feedback_gain != 0 {
            let pitch_urad = i128::from(root.rotation_q1_30[0]) * 2_000_000 / (1_i128 << 30);
            i64::try_from(
                -hip_feedback_gain * pitch_urad
                    - i128::from(root.angular_velocity_microradians_per_second[0]) / 5,
            )?
        } else {
            0
        };
        for (target, actuator) in reference
            .iter_mut()
            .zip(&base.physics_descriptors.actuators)
        {
            if reference_mode == "neutral-targets" {
                *target = actuator.base.neutral_position_microradians;
            }
            if actuator.base.joint_id.as_str().ends_with("-ankle-pitch") {
                *target += ankle_offset;
            } else if actuator.base.joint_id.as_str().ends_with("-hip-pitch") {
                *target += hip_offset + feedback;
            }
        }
        let applied_targets = safety
            .begin_motor_tick(&reference, &[0; 23], &envelopes)?
            .iter()
            .map(|target| target.target_microradians)
            .collect::<Vec<_>>();
        let mut contacts = Vec::with_capacity(4);
        for _ in 0..4 {
            let efforts = safety.step_substep(&states(&snapshot))?;
            let mut applied = [0; 23];
            for (effort, &dof) in efforts.iter().zip(&base.actuator_dof_ordinals) {
                applied[dof as usize] = effort.effort_micronewton_metres;
            }
            snapshot = world.apply_efforts_and_step(&applied)?;
            substeps += 1;
            let root = snapshot
                .links
                .iter()
                .find(|l| l.user_token == base.physx_catalog.links[0].user_token)
                .expect("root");
            substep_samples.push(json!({
                "physics_substep": substeps,
                "rotation_q1_30": root.rotation_q1_30,
                "angular_velocity_urad_s": root.angular_velocity_microradians_per_second,
                "position_um": root.position_micrometres,
                "linear_velocity_um_s": root.linear_velocity_micrometres_per_second,
                "applied_efforts_by_dof_unm": applied,
                "joints": snapshot.joints.iter().map(|j| json!({"ordinal": j.ordinal,
                    "position_urad": j.position_microradians,
                    "velocity_urad_s": j.velocity_microradians_per_second})).collect::<Vec<_>>(),
                "contacts": snapshot.contacts.iter().map(|c| json!({
                    "actors": [c.actor_a_token, c.actor_b_token],
                    "shapes": [c.shape_a_token, c.shape_b_token],
                    "position_um": c.position_micrometres,
                    "normal_q1_30": c.normal_q1_30,
                    "impulse_uns": c.impulse_micronewton_seconds,
                    "separation_um": c.separation_micrometres,
                })).collect::<Vec<_>>()
            }));
            if let Err(error) = safety.validate_observed_joint_states(&states(&snapshot)) {
                reason = format!("joint-safety: {error}");
                samples.push(sample(
                    tick,
                    substeps,
                    &snapshot,
                    &reference,
                    &applied_targets,
                ));
                break 'episode;
            }
            contacts.push(
                classifier
                    .classify_substep(&snapshot, BiomechanicsSkillContactProfileV1::Locomotion)?,
            );
        }
        let decision = terminal.evaluate_motor_tick(tick, &snapshot, &contacts, false, None)?;
        samples.push(sample(
            tick,
            substeps,
            &snapshot,
            &reference,
            &applied_targets,
        ));
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
            "schema_version": 9, "probe": "native-body-standing-actuator-discriminator-v9",
            "actuator_probe": actuator_probe,
            "body_schema_id": schema.schema_id.as_str(),
            "reference_mode": reference_mode,
            "force_schedule": if per_iteration { "every-solver-position-iteration" } else { "frame-start" },
            "force_schedule_profile_id": per_iteration.then_some(next_motor::BIOMECHANICS_FORCE_SCHEDULE_PROFILE_ID_V1),
            "ankle_reference_offset_urad": ankle_offset,
            "hip_reference_offset_urad": hip_offset,
            "hip_feedback_gain": hip_feedback_gain,
            "ordered_actuator_ids": base.actuator_definitions.iter().map(|a| a.actuator_id.as_str()).collect::<Vec<_>>(),
            "actuators": base.actuator_definitions.iter().zip(&base.actuator_dof_ordinals).map(|(a, dof)| json!({
                "actuator_id": a.actuator_id.as_str(), "dof_ordinal": dof,
                "stiffness_q16": a.stiffness_q16, "damping_q16": a.damping_q16,
            })).collect::<Vec<_>>(),
            "body_schema_hash": base.body_schema_hash.to_hex(),
            "compiled_descriptor_hash": if per_iteration { successor.compiled_descriptor_hash } else { compiled.compiled_descriptor_hash }.to_hex(),
            "body_revision": schema.schema_revision, "reason": reason,
            "requested_motor_ticks": 1_800, "physics_substeps": substeps,
            "scope": "nominal procedural standing diagnostic, not learned quality",
            "samples": samples,
            "substep_samples": substep_samples,
        }))?
    );
    Ok(())
}

/// Diagnostic input through the normal schema compiler and safety controller.
/// The contract requires positive gains: 1 Q16 is near-passive, not zero torque.
/// No selected body/profile or downstream effort/safety rule is changed.
#[cfg(feature = "physx-sdk")]
fn shoulder_yaw_discriminator(
    mut schema: next_contracts::body::BodySchemaV2,
    mode: &str,
) -> Result<next_contracts::body::BodySchemaV2, Box<dyn std::error::Error>> {
    use next_contracts::canonical::sha256;
    use next_contracts::ids::{SchemaId, content_hash_from_bytes};

    if !matches!(mode, "shoulder-yaw-near-passive" | "shoulder-yaw-gain-16") {
        return Err("unknown shoulder-yaw discriminator".into());
    }
    schema.schema_id = SchemaId::new(format!("{}.{mode}", schema.schema_id.as_str()))?;
    let mut source = format!("nextengine.probe.{mode}.v1\0").into_bytes();
    source.extend_from_slice(schema.source_provenance_hash.as_bytes());
    schema.source_provenance_hash = content_hash_from_bytes(sha256(&source));
    let mut count = 0;
    for actuator in &mut schema.actuators {
        if actuator.joint_id.as_str().ends_with("-shoulder-yaw") {
            if mode == "shoulder-yaw-near-passive" {
                actuator.stiffness_q16 = 1;
                actuator.damping_q16 = 1;
            } else {
                actuator.stiffness_q16 /= 16;
                actuator.damping_q16 /= 16;
            }
            count += 1;
        }
    }
    if count != 2 {
        return Err("expected exactly two shoulder-yaw actuators".into());
    }
    schema.validate()?;
    Ok(schema)
}

#[cfg(all(test, feature = "physx-sdk"))]
mod tests {
    #[test]
    fn actuator_discriminator_changes_only_identified_shoulder_yaw_gains() {
        let original = next_motor::biomechanics_humanoid_body_schema_v6();
        for mode in ["shoulder-yaw-near-passive", "shoulder-yaw-gain-16"] {
            let mut candidate = super::shoulder_yaw_discriminator(original.clone(), mode).unwrap();
            assert_ne!(candidate.schema_id, original.schema_id);
            assert_ne!(
                candidate.source_provenance_hash,
                original.source_provenance_hash
            );
            candidate.schema_id = original.schema_id.clone();
            candidate.source_provenance_hash = original.source_provenance_hash;
            let mut changed = 0;
            for (actual, expected) in candidate.actuators.iter_mut().zip(&original.actuators) {
                if actual != expected {
                    assert!(actual.joint_id.as_str().ends_with("-shoulder-yaw"));
                    let gains = if mode == "shoulder-yaw-near-passive" {
                        (1, 1)
                    } else {
                        (expected.stiffness_q16 / 16, expected.damping_q16 / 16)
                    };
                    assert_eq!((actual.stiffness_q16, actual.damping_q16), gains);
                    actual.stiffness_q16 = expected.stiffness_q16;
                    actual.damping_q16 = expected.damping_q16;
                    changed += 1;
                }
            }
            assert_eq!(changed, 2);
            assert_eq!(candidate, original);
        }
        assert!(super::shoulder_yaw_discriminator(original, "unknown").is_err());
    }
}

#[cfg(not(feature = "physx-sdk"))]
fn main() {
    panic!("probe_biomechanics_body_standing requires --features physx-sdk");
}
