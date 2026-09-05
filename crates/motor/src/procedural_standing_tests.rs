use next_contracts::ids::PersistentId;
use next_physics_physx::{CanonicalPhysXLinkState, CanonicalPhysXSnapshotV2};

use crate::{
    BiomechanicsProceduralStandingControllerV1, CompiledBodySchemaV2, ProceduralStandingError,
    biomechanics_humanoid_body_schema_v2,
};

#[test]
fn upright_reference_matches_frozen_integer_hip_law_and_preserves_other_channels() {
    let compiled = crate::CompiledBodySchemaV4::compile(
        &crate::biomechanics_humanoid_body_schema_v7(),
        PersistentId::from_bytes([62; 16]),
    )
    .unwrap();
    let base = &compiled.base.base;
    let root = base.body_tokens[&base.construction_order[0]];
    let reset = snapshot(root, 0);
    let old = BiomechanicsProceduralStandingControllerV1::new(base, &reset).unwrap();
    let new = crate::BiomechanicsProceduralStandingControllerV2::new(&compiled, &reset).unwrap();
    assert_ne!(old.state_root(), new.state_root());
    for qx in [-536_870_912, -53_687_091, -1, 0, 1, 53_687_091, 536_870_912] {
        let mut state = reset.clone();
        state.links[0].rotation_q1_30[0] = qx;
        for omega in [-123_456, 0, 789_123] {
            state.links[0].angular_velocity_microradians_per_second[0] = omega;
            let mut expected = old.reference_targets(&state).unwrap();
            for (value, actuator) in expected.iter_mut().zip(&base.actuator_definitions) {
                if actuator.joint_id.as_str().ends_with("-hip-pitch") {
                    *value += (-2 * (i128::from(qx) * 2_000_000 / (1_i128 << 30))) as i64;
                }
            }
            assert_eq!(new.reference_targets(&state).unwrap(), expected);
        }
    }
    let shifted =
        crate::BiomechanicsProceduralStandingControllerV2::new(&compiled, &snapshot(root, 1))
            .unwrap();
    assert_ne!(new.state_root(), shifted.state_root());
    let other_subject = crate::CompiledBodySchemaV4::compile(
        &crate::biomechanics_humanoid_body_schema_v7(),
        PersistentId::from_bytes([63; 16]),
    )
    .unwrap();
    let other =
        crate::BiomechanicsProceduralStandingControllerV2::new(&other_subject, &reset).unwrap();
    assert_ne!(new.state_root(), other.state_root());
    let mut invalid = reset.clone();
    invalid.links.push(invalid.links[0].clone());
    assert_eq!(
        new.reference_targets(&invalid),
        Err(ProceduralStandingError::RootState)
    );
}

#[test]
fn upright_reference_rejects_old_and_tampered_compilations() {
    let subject = PersistentId::from_bytes([0; 16]);
    let v7 = crate::CompiledBodySchemaV4::compile(
        &crate::biomechanics_humanoid_body_schema_v7(),
        subject,
    )
    .unwrap();
    let root = v7.base.base.body_tokens[&v7.base.base.construction_order[0]];
    let reset = snapshot(root, 0);
    let old = crate::CompiledBodySchemaV4::compile(
        &crate::biomechanics_humanoid_body_schema_v6(),
        subject,
    )
    .unwrap();
    let mut tampered = v7.clone();
    tampered.base.base.actuator_definitions[0].damping_q16 += 1;
    for invalid in [old, tampered] {
        assert_eq!(
            crate::BiomechanicsProceduralStandingControllerV2::new(&invalid, &reset),
            Err(ProceduralStandingError::ProfileMismatch)
        );
    }
    let mut absent = reset.clone();
    absent.links.clear();
    assert_eq!(
        crate::BiomechanicsProceduralStandingControllerV2::new(&v7, &absent),
        Err(ProceduralStandingError::RootState)
    );
}

fn compiled() -> CompiledBodySchemaV2 {
    CompiledBodySchemaV2::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([62; 16]),
    )
    .expect("compile biomechanics profile")
}

fn snapshot(root_actor: u64, z: i64) -> CanonicalPhysXSnapshotV2 {
    CanonicalPhysXSnapshotV2 {
        links: vec![CanonicalPhysXLinkState {
            user_token: root_actor,
            position_micrometres: [0, 1_095_000, z],
            rotation_q1_30: [0, 0, 0, 1 << 30],
            linear_velocity_micrometres_per_second: [0; 3],
            angular_velocity_microradians_per_second: [0; 3],
        }],
        joints: Vec::new(),
        contacts: Vec::new(),
    }
}

#[test]
fn standing_reference_has_exact_knee_ankle_and_neutral_targets() {
    let compiled = compiled();
    let root_actor = compiled.body_tokens[&compiled.construction_order[0]];
    let reset = snapshot(root_actor, -79_401);
    let controller = BiomechanicsProceduralStandingControllerV1::new(&compiled, &reset)
        .expect("standing controller");
    let targets = controller
        .reference_targets(&reset)
        .expect("standing targets");
    for (target, actuator) in targets.iter().zip(&compiled.physics_descriptors.actuators) {
        let joint = actuator.base.joint_id.as_str();
        if joint.ends_with("-knee") {
            assert_eq!(*target, 100_000);
        } else if joint.ends_with("-ankle-pitch") {
            assert_eq!(*target, -140_000);
        } else {
            assert_eq!(*target, actuator.base.neutral_position_microradians);
        }
    }

    let mut perturbed = reset.clone();
    let root = &mut perturbed.links[0];
    root.rotation_q1_30[0] = 53_687_091;
    root.angular_velocity_microradians_per_second[0] = 20_000;
    root.position_micrometres[2] += 10_000;
    root.linear_velocity_micrometres_per_second[2] = 50_000;
    let targets = controller
        .reference_targets(&perturbed)
        .expect("feedback targets");
    for (target, actuator) in targets.iter().zip(&compiled.physics_descriptors.actuators) {
        if actuator.base.joint_id.as_str().ends_with("-ankle-pitch") {
            assert_eq!(*target, -87_000);
        }
    }
}

#[test]
fn standing_reset_state_is_hash_bound_and_requires_one_root() {
    let compiled = compiled();
    let root_actor = compiled.body_tokens[&compiled.construction_order[0]];
    let first =
        BiomechanicsProceduralStandingControllerV1::new(&compiled, &snapshot(root_actor, -79_401))
            .expect("first reset");
    let second =
        BiomechanicsProceduralStandingControllerV1::new(&compiled, &snapshot(root_actor, -79_400))
            .expect("second reset");
    assert_ne!(first.state_root(), second.state_root());
    assert_eq!(
        BiomechanicsProceduralStandingControllerV1::new(
            &compiled,
            &CanonicalPhysXSnapshotV2 {
                links: Vec::new(),
                joints: Vec::new(),
                contacts: Vec::new(),
            },
        ),
        Err(ProceduralStandingError::RootState)
    );
}

#[test]
fn walking_reference_is_translation_invariant_without_changing_standing() {
    let compiled = compiled();
    let root_actor = compiled.body_tokens[&compiled.construction_order[0]];
    let reset = snapshot(root_actor, 0);
    let standing = BiomechanicsProceduralStandingControllerV1::new(&compiled, &reset)
        .expect("standing controller");
    let walking = BiomechanicsProceduralStandingControllerV1::new_walking_translation_invariant(
        &compiled, &reset,
    )
    .expect("walking reference controller");
    let mut translated = reset.clone();
    translated.links[0].position_micrometres[2] = 1_000_000;

    let standing_reset = standing.reference_targets(&reset).expect("standing reset");
    let standing_translated = standing
        .reference_targets(&translated)
        .expect("standing translated");
    let walking_reset = walking.reference_targets(&reset).expect("walking reset");
    let walking_translated = walking
        .reference_targets(&translated)
        .expect("walking translated");

    assert_eq!(walking_reset, walking_translated);
    assert_ne!(standing_reset, standing_translated);
    for ((reset_target, translated_target), actuator) in standing_reset
        .iter()
        .zip(&standing_translated)
        .zip(&compiled.physics_descriptors.actuators)
    {
        if actuator.base.joint_id.as_str().ends_with("-ankle-pitch") {
            assert_eq!(*translated_target - *reset_target, 100_000);
        } else {
            assert_eq!(translated_target, reset_target);
        }
    }
    assert_ne!(standing.state_root(), walking.state_root());

    let translated_walking =
        BiomechanicsProceduralStandingControllerV1::new_walking_translation_invariant(
            &compiled,
            &translated,
        )
        .expect("translated walking reference controller");
    assert_eq!(walking.state_root(), translated_walking.state_root());
}
