use std::collections::BTreeMap;

use next_contracts::body::{BodyInstanceProjectionV1, BodyProjectionRootsV1, BodySchemaV1};
use next_contracts::ids::{ContentHash, PersistentId, PhysicsWorldId};
use next_contracts::physics::{
    PHYSICS_SNAPSHOT_SCHEMA_VERSION, PhysicsBodyIdV1, PhysicsBodyStateV2,
    PhysicsCanonicalSnapshotV2, PhysicsPoseV1,
};

use crate::{
    CapsuleMotorRouteV1, CapsuleProceduralMotorControllerV1, CapsuleProceduralMotorError,
    CompiledBodySchemaV1, neutral_body_instance_projection_v1, reference_humanoid_body_schema_v1,
};

struct Fixture {
    schema: BodySchemaV1,
    projection: BodyInstanceProjectionV1,
    roots: BodyProjectionRootsV1,
    body_id: PhysicsBodyIdV1,
    controller: CapsuleProceduralMotorControllerV1,
    physics: PhysicsCanonicalSnapshotV2,
}

fn fixture() -> Fixture {
    let schema = reference_humanoid_body_schema_v1();
    let subject_id = PersistentId::from_bytes([0x91; 16]);
    let projection =
        neutral_body_instance_projection_v1(&schema, subject_id).expect("neutral projection");
    let compiled =
        CompiledBodySchemaV1::compile_projection(&schema, &projection).expect("compiled body");
    let roots = compiled.projection_roots;
    let body_id = PhysicsBodyIdV1 {
        subject_id,
        body_slot: 0,
    };
    let controller =
        CapsuleProceduralMotorControllerV1::activate(&schema, &projection, &roots, body_id, 30)
            .expect("procedural capsule controller");
    let body = PhysicsBodyStateV2 {
        body_id,
        body_revision: 7,
        pose: PhysicsPoseV1::default(),
        linear_velocity_micrometres_per_second: [0; 3],
        angular_velocity_q16: [0; 3],
        active: true,
        sleep_counter: 0,
    };
    let physics = PhysicsCanonicalSnapshotV2 {
        schema_version: PHYSICS_SNAPSHOT_SCHEMA_VERSION,
        world_id: PhysicsWorldId::from_bytes([0x92; 16]),
        world_revision: 0,
        checkpoint_revision: 0,
        physics_tick: 19,
        world_descriptor_hash: ContentHash::from_bytes([1; 32]),
        catalog_hash: ContentHash::from_bytes([2; 32]),
        tick_rate_profile_hash: ContentHash::from_bytes([3; 32]),
        authoritative_numeric_profile_hash: ContentHash::from_bytes([4; 32]),
        physics_quantization_profile_hash: ContentHash::from_bytes([5; 32]),
        physics_limits_profile_hash: ContentHash::from_bytes([6; 32]),
        sorted_body_states: BTreeMap::from([(body_id, body)]),
        sorted_contact_continuity_states: BTreeMap::new(),
        sorted_solver_continuation_states: BTreeMap::new(),
    };
    Fixture {
        schema,
        projection,
        roots,
        body_id,
        controller,
        physics,
    }
}

#[test]
fn exact_projection_binds_a_stable_tracking_profile() {
    let fixture = fixture();
    assert_eq!(
        fixture.controller.subject_id(),
        fixture.projection.subject_id
    );
    assert_eq!(fixture.controller.body_id(), fixture.body_id);
    assert_eq!(
        fixture.controller.body_projection_root(),
        fixture.roots.projection_root().expect("projection root")
    );
    assert_eq!(
        fixture.controller.action_layout_hash(),
        fixture.roots.action_layout_hash
    );
    assert_eq!(
        fixture.controller.actuator_safety_root(),
        fixture.roots.actuator_safety_root
    );

    let right = fixture
        .controller
        .evaluate(&fixture.physics, [32_767, 0])
        .expect("safe cardinal action");
    assert_eq!(right.route, CapsuleMotorRouteV1::ProceduralTracking);
    assert_eq!(right.requested_direction_q15, [32_767, 0]);
    assert_eq!(right.applied_direction_q15, [32_767, 0]);
    assert_eq!(right.clamp_mask, 0);
    assert_eq!(right.source_physics_tick, 19);
    assert_eq!(right.source_body_revision, 7);
    assert_eq!(
        right,
        fixture
            .controller
            .evaluate(&fixture.physics, [32_767, 0])
            .expect("repeat exact decision")
    );

    let idle = fixture
        .controller
        .evaluate(&fixture.physics, [0, 0])
        .expect("idle action");
    assert_eq!(idle.route, CapsuleMotorRouteV1::Idle);
    assert_eq!(idle.applied_direction_q15, [0, 0]);
    assert_ne!(idle.decision_hash, right.decision_hash);
}

#[test]
fn falling_body_selects_zero_recovery_and_resumes_without_motor_state() {
    let mut fixture = fixture();
    fixture
        .physics
        .sorted_body_states
        .get_mut(&fixture.body_id)
        .expect("body")
        .linear_velocity_micrometres_per_second[1] = -2_400_000;
    let recovery = fixture
        .controller
        .evaluate(&fixture.physics, [0, 32_767])
        .expect("recovery action");
    assert_eq!(recovery.route, CapsuleMotorRouteV1::ProceduralRecovery);
    assert_eq!(recovery.applied_direction_q15, [0, 0]);
    assert_eq!(recovery.clamp_mask, 0b10);

    fixture
        .physics
        .sorted_body_states
        .get_mut(&fixture.body_id)
        .expect("body")
        .linear_velocity_micrometres_per_second[1] = 0;
    let resumed = fixture
        .controller
        .evaluate(&fixture.physics, [0, 32_767])
        .expect("resumed action");
    assert_eq!(resumed.route, CapsuleMotorRouteV1::ProceduralTracking);
    assert_eq!(resumed.applied_direction_q15, [0, 32_767]);
    assert_eq!(resumed.clamp_mask, 0);
    assert_ne!(resumed.decision_hash, recovery.decision_hash);
}

#[test]
fn malformed_projection_candidate_and_body_state_fail_closed() {
    let fixture = fixture();
    assert_eq!(
        fixture
            .controller
            .evaluate(&fixture.physics, [32_767, 32_767]),
        Err(CapsuleProceduralMotorError::CandidateInvalid)
    );

    let mut roots = fixture.roots;
    roots.actuator_safety_root = ContentHash::from_bytes([0xa5; 32]);
    assert_eq!(
        CapsuleProceduralMotorControllerV1::activate(
            &fixture.schema,
            &fixture.projection,
            &roots,
            fixture.body_id,
            30,
        ),
        Err(CapsuleProceduralMotorError::ProjectionMismatch)
    );

    let mut missing = fixture.physics.clone();
    missing.sorted_body_states.clear();
    assert_eq!(
        fixture.controller.evaluate(&missing, [32_767, 0]),
        Err(CapsuleProceduralMotorError::BodyMissing)
    );

    let mut tilted = fixture.physics;
    tilted
        .sorted_body_states
        .get_mut(&fixture.body_id)
        .expect("body")
        .pose
        .rotation_q1_30 = [1, 0, 0, 1 << 30];
    assert_eq!(
        fixture.controller.evaluate(&tilted, [32_767, 0]),
        Err(CapsuleProceduralMotorError::PhysicsSnapshotInvalid)
    );
}
