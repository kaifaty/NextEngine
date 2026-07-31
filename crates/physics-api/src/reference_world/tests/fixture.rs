use std::collections::BTreeMap;

use next_contracts::ids::{CommandId, PersistentId, PhysicsWorldId, SchemaId};
use next_contracts::input::TickRateProfileV1;
use next_contracts::physics::{
    AcceptedLocomotionIntentV2, AuthoritativeNumericProfileV1, PHYSICS_STEP_INPUT_SCHEMA_VERSION,
    PhysicsBodyDescriptorV1, PhysicsBodyIdV1, PhysicsBodyStateV2, PhysicsCanonicalSnapshotV2,
    PhysicsContactReportingV1, PhysicsCoordinateProfileV1, PhysicsGeometryV1,
    PhysicsLimitsProfileV1, PhysicsMaterialDescriptorV1, PhysicsMotionKindV1,
    PhysicsParticipationV1, PhysicsPoseV1, PhysicsQuantizationProfileV1, PhysicsShapeDescriptorV1,
    PhysicsShapeIdV1, PhysicsSolverSemanticsProfileV1, PhysicsStepInputV2, PhysicsStepResultV1,
    PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1, PhysicsWorldCheckpointV1,
};

use super::super::{
    GroundedCapsuleQuery, GroundedCapsuleSweepRequest, GroundedCapsuleSweepResult,
    ReferencePhysicsError, ReferencePhysicsWorld,
};

#[derive(Debug)]
pub(super) struct FailingQuery;

impl GroundedCapsuleQuery for FailingQuery {
    fn backend_kind(&self) -> crate::PhysicsBackendKind {
        crate::PhysicsBackendKind::PhysX
    }

    fn recreate(&self) -> Result<Self, ReferencePhysicsError> {
        Ok(Self)
    }

    fn sweep_axis(
        &mut self,
        _request: GroundedCapsuleSweepRequest<'_>,
    ) -> Result<GroundedCapsuleSweepResult, ReferencePhysicsError> {
        Err(ReferencePhysicsError::BackendFailure)
    }
}

pub(super) fn world(
    gameplay_hz: u32,
    physics_hz: u32,
    initial_centre: [i64; 3],
    wall_collision_mask: u64,
) -> ReferencePhysicsWorld {
    world_with_wall(
        gameplay_hz,
        physics_hz,
        initial_centre,
        wall_collision_mask,
        700_000,
        100_000,
    )
}

pub(super) fn world_with_wall(
    gameplay_hz: u32,
    physics_hz: u32,
    initial_centre: [i64; 3],
    wall_collision_mask: u64,
    wall_centre_z: i64,
    wall_half_extent_z: i64,
) -> ReferencePhysicsWorld {
    world_with_wall_reporting(
        gameplay_hz,
        physics_hz,
        initial_centre,
        wall_collision_mask,
        wall_centre_z,
        wall_half_extent_z,
        PhysicsContactReportingV1::BeginPersistEnd,
    )
}

pub(super) fn world_with_wall_reporting(
    gameplay_hz: u32,
    physics_hz: u32,
    initial_centre: [i64; 3],
    wall_collision_mask: u64,
    wall_centre_z: i64,
    wall_half_extent_z: i64,
    contact_reporting: PhysicsContactReportingV1,
) -> ReferencePhysicsWorld {
    let tick_rate = TickRateProfileV1 {
        schema_version: 1,
        gameplay_hz,
        physics_substeps_per_gameplay_tick: physics_hz / gameplay_hz,
        motor_period_physics_substeps: 1,
        first_gameplay_tick: 0,
    };
    let quantization = PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
    let numeric =
        AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
    let material_id =
        SchemaId::new("nextengine.physics.material.reference-zero").expect("material identifier");
    let material = PhysicsMaterialDescriptorV1 {
        material_id: material_id.clone(),
        descriptor_revision: 1,
        static_friction_q16: 0,
        dynamic_friction_q16: 0,
        restitution_q16: 0,
        canonical_material_tags: Vec::new(),
    };
    let capsule_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([1; 16]),
        body_slot: 0,
    };
    let capsule_shape_id = PhysicsShapeIdV1 {
        body_id: capsule_body_id,
        shape_slot: 0,
    };
    let capsule = body(
        capsule_body_id,
        PhysicsMotionKindV1::Kinematic,
        initial_centre,
        shape(
            capsule_shape_id,
            PhysicsGeometryV1::Capsule {
                radius_micrometres: 300_000,
                half_segment_micrometres: 600_000,
            },
            &material_id,
            1,
            contact_reporting,
        ),
    );
    let floor_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([2; 16]),
        body_slot: 0,
    };
    let floor_shape_id = PhysicsShapeIdV1 {
        body_id: floor_body_id,
        shape_slot: 0,
    };
    let floor = body(
        floor_body_id,
        PhysicsMotionKindV1::Static,
        [0, -100_000, 0],
        shape(
            floor_shape_id,
            PhysicsGeometryV1::Box {
                half_extents_micrometres: [10_000_000, 100_000, 10_000_000],
            },
            &material_id,
            1,
            contact_reporting,
        ),
    );
    let wall_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([3; 16]),
        body_slot: 0,
    };
    let wall_shape_id = PhysicsShapeIdV1 {
        body_id: wall_body_id,
        shape_slot: 0,
    };
    let wall = body(
        wall_body_id,
        PhysicsMotionKindV1::Static,
        [0, 900_000, wall_centre_z],
        shape(
            wall_shape_id,
            PhysicsGeometryV1::Box {
                half_extents_micrometres: [10_000_000, 10_000_000, wall_half_extent_z],
            },
            &material_id,
            wall_collision_mask,
            contact_reporting,
        ),
    );
    let catalog = PhysicsWorldCatalogV1::new(
        PhysicsWorldId::from_bytes([9; 16]),
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("coordinates"),
            limits: PhysicsLimitsProfileV1::reference_v1().expect("limits"),
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("solver"),
            tick_rate_hash: tick_rate.profile_hash().expect("tick hash"),
            authoritative_numeric_hash: numeric.profile_hash().expect("numeric hash"),
            quantization_hash: quantization.profile_hash().expect("quantization hash"),
        },
        BTreeMap::from([(material_id, material)]),
        BTreeMap::from([
            (capsule_body_id, capsule),
            (floor_body_id, floor),
            (wall_body_id, wall),
        ]),
        BTreeMap::from([(capsule_body_id.subject_id, capsule_body_id)]),
    )
    .expect("catalog");
    let snapshot =
        PhysicsCanonicalSnapshotV2::genesis(&catalog, &tick_rate, &numeric, &quantization)
            .expect("snapshot");
    let checkpoint = PhysicsWorldCheckpointV1::new(catalog, snapshot).expect("checkpoint");
    ReferencePhysicsWorld::new(checkpoint, tick_rate, numeric, quantization).expect("world")
}

pub(super) fn body(
    body_id: PhysicsBodyIdV1,
    motion_kind: PhysicsMotionKindV1,
    translation_micrometres: [i64; 3],
    shape: PhysicsShapeDescriptorV1,
) -> PhysicsBodyDescriptorV1 {
    PhysicsBodyDescriptorV1 {
        body_id,
        descriptor_revision: 1,
        motion_kind,
        initial_pose: PhysicsPoseV1 {
            translation_micrometres,
            ..PhysicsPoseV1::default()
        },
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        shapes: BTreeMap::from([(shape.shape_id, shape)]),
    }
}

pub(super) fn shape(
    shape_id: PhysicsShapeIdV1,
    geometry: PhysicsGeometryV1,
    material_id: &SchemaId,
    collision_mask: u64,
    contact_reporting: PhysicsContactReportingV1,
) -> PhysicsShapeDescriptorV1 {
    PhysicsShapeDescriptorV1 {
        shape_id,
        descriptor_revision: 1,
        local_pose: PhysicsPoseV1::default(),
        geometry,
        material_id: material_id.clone(),
        collision_layer: 0,
        collision_mask,
        participation: PhysicsParticipationV1::Solid,
        contact_reporting,
    }
}

pub(super) fn step(
    world: &mut ReferencePhysicsWorld,
    gameplay_tick: u64,
    direction: Option<[i16; 2]>,
) -> PhysicsStepResultV1 {
    let input = step_input(world, gameplay_tick, direction);
    world.step(&input).expect("physics step")
}

pub(super) fn step_input(
    world: &ReferencePhysicsWorld,
    gameplay_tick: u64,
    direction: Option<[i16; 2]>,
) -> PhysicsStepInputV2 {
    let snapshot = world.snapshot();
    let accepted_intents = direction.map_or_else(Vec::new, |direction_q15| {
        let body_id = *world
            .checkpoint()
            .catalog
            .avatar_bindings
            .values()
            .next()
            .expect("capsule binding");
        vec![AcceptedLocomotionIntentV2 {
            causal_command_id: CommandId::from_bytes([gameplay_tick as u8; 16]),
            controlled_target_id: body_id.subject_id,
            body_id,
            target_gameplay_tick: gameplay_tick,
            direction_q15,
        }]
    });
    PhysicsStepInputV2 {
        schema_version: PHYSICS_STEP_INPUT_SCHEMA_VERSION,
        world_id: snapshot.world_id,
        expected_world_revision: snapshot.world_revision,
        expected_snapshot_hash: snapshot.snapshot_hash().expect("snapshot hash"),
        expected_catalog_hash: world
            .checkpoint()
            .catalog
            .catalog_hash()
            .expect("catalog hash"),
        gameplay_tick,
        first_physics_tick: snapshot
            .physics_tick
            .checked_add(1)
            .expect("test physics tick remains bounded"),
        physics_substeps: world.tick_rate_profile().physics_substeps_per_gameplay_tick,
        accepted_intents,
    }
}

pub(super) fn capsule_state(world: &ReferencePhysicsWorld) -> &PhysicsBodyStateV2 {
    let body_id = world
        .checkpoint()
        .catalog
        .avatar_bindings
        .values()
        .next()
        .expect("capsule binding");
    &world.snapshot().sorted_body_states[body_id]
}

pub(super) fn reconstruct(
    checkpoint: PhysicsWorldCheckpointV1,
    source: &ReferencePhysicsWorld,
) -> Result<ReferencePhysicsWorld, ReferencePhysicsError> {
    ReferencePhysicsWorld::new(
        checkpoint,
        source.tick_rate_profile().to_owned(),
        source.numeric_profile().to_owned(),
        source.quantization_profile().to_owned(),
    )
}
