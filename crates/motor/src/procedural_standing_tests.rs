use next_contracts::ids::PersistentId;
use next_physics_physx::{CanonicalPhysXLinkState, CanonicalPhysXSnapshotV2};

use crate::{
    BiomechanicsProceduralStandingControllerV1, CompiledBodySchemaV2, ProceduralStandingError,
    biomechanics_humanoid_body_schema_v2,
};

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
