use next_contracts::ids::CommandStreamId;
use next_contracts::input::{ActionMapManifestV1, InputContextStackV1};
use next_contracts::physics::{
    PhysicsCoordinateProfileV1, PhysicsLimitsProfileV1, PhysicsSolverSemanticsProfileV1,
    PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
};

use super::*;

pub(super) struct Fixture {
    pub(super) runtime: RuntimeState,
    pub(super) principal: IssuerPrincipal,
    pub(super) stream_id: CommandStreamId,
}

pub(super) fn fixture() -> Fixture {
    fixture_for(
        "nextengine.runtime-fixture",
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([3; 16])),
    )
}

pub(super) fn fixture_for(project_id: &str, principal: IssuerPrincipal) -> Fixture {
    let profile = RuntimeDeterminismBundleV1::core_r8c()
        .expect("determinism bundle")
        .runtime_profile();
    let world = WorldIdentityManifestV1::new(
        ProjectId::new(project_id).expect("project"),
        [1; 32],
        [2; 32],
        profile.profile_hash().expect("profile hash"),
    )
    .expect("world");
    let mut principals = PrincipalRegistryV1::empty(world.world_namespace);
    principals
        .register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash: content_hash_from_bytes([4; 32]),
                capability_subject_id: SchemaId::new("fixture.player").expect("subject"),
                status: PrincipalStatus::Active,
            },
        )
        .expect("principal");
    let mut streams = CommandStreamRegistryV1::empty(world.world_namespace);
    let stream_id = streams.allocate_stream(principal.clone()).expect("stream");
    let mut authority = AuthorityRegistry::new();
    authority
        .register(
            principal.clone(),
            [CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID).expect("capability")],
        )
        .expect("authority");
    let runtime = RuntimeState::new(
        RuntimeBootstrapV4::new(world, principals, streams, profile),
        authority,
    )
    .expect("runtime");
    Fixture {
        runtime,
        principal,
        stream_id,
    }
}

pub(super) fn command(fixture: &Fixture, sequence: u64, target_tick: u64) -> WorldCommand {
    WorldCommand::noop(
        fixture.stream_id,
        fixture.principal.clone(),
        sequence,
        target_tick,
    )
    .expect("command")
}

pub(super) struct PhysicalFixture {
    pub(super) runtime: RuntimeState,
    pub(super) principal: IssuerPrincipal,
    pub(super) source_id: InputSourceId,
    pub(super) controller_id: PersistentId,
    pub(super) physics_body_id: PhysicsBodyIdV1,
    pub(super) command_stream_id: CommandStreamId,
    pub(super) action_map_hash: ContentHash,
    pub(super) context_stack_hash: ContentHash,
}

pub(super) fn physical_fixture() -> PhysicalFixture {
    physical_fixture_with_wall(false)
}

pub(super) fn clipped_physical_fixture() -> PhysicalFixture {
    physical_fixture_with_wall(true)
}

fn physical_fixture_with_wall(include_wall: bool) -> PhysicalFixture {
    let principal = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([13; 16]));
    let profile = RuntimeDeterminismBundleV1::core_r8c()
        .expect("determinism bundle")
        .runtime_profile();
    let world = WorldIdentityManifestV1::new(
        ProjectId::new("nextengine.runtime-physical-fixture").expect("project"),
        [14; 32],
        [15; 32],
        profile.profile_hash().expect("profile hash"),
    )
    .expect("world");
    let mut principals = PrincipalRegistryV1::empty(world.world_namespace);
    principals
        .register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash: content_hash_from_bytes([16; 32]),
                capability_subject_id: SchemaId::new("fixture.physical-player").expect("subject"),
                status: PrincipalStatus::Active,
            },
        )
        .expect("principal");
    let mut streams = CommandStreamRegistryV1::empty(world.world_namespace);
    let stream_id = streams.allocate_stream(principal.clone()).expect("stream");
    let mut authority = AuthorityRegistry::new();
    authority
        .register(
            principal.clone(),
            [CapabilityId::new(PHYSICAL_COMMAND_CAPABILITY_ID).expect("capability")],
        )
        .expect("authority");

    let source_id = InputSourceId::from_bytes([17; 16]);
    let controller_id = PersistentId::from_bytes([18; 16]);
    let body_id = PersistentId::from_bytes([19; 16]);
    authority
        .register_root_motion_source(
            principal.clone(),
            body_id,
            ContentHash::from_bytes([31; 32]),
            ContentHash::from_bytes([32; 32]),
        )
        .expect("root-motion source authority");
    let action_map = ActionMapManifestV1::core_keyboard_mouse_v1().expect("core action map");
    let action_map_hash = action_map.content_hash;
    let context_stack = InputContextStackV1::gameplay_v1().expect("gameplay context stack");
    let context_stack_hash = context_stack.content_hash;
    let mut bootstrap = RuntimeBootstrapV4::new(world, principals, streams, profile);
    let binding = PlayerControllerBindingV1 {
        principal: principal.clone(),
        source_id,
        controller_id,
        controlled_body_id: body_id,
        command_stream_id: stream_id,
        action_map_hash,
        action_map_revision: 1,
        context_stack_hash,
        context_stack_revision: 1,
        action_map,
        context_stack,
    };
    bootstrap
        .player_controller_registry
        .bindings
        .insert(source_id, binding);
    let physics_body_id = PhysicsBodyIdV1 {
        subject_id: body_id,
        body_slot: 0,
    };
    bootstrap.physics_checkpoint = grounded_test_checkpoint(
        PhysicsWorldId::from_bytes(*bootstrap.world_identity.world_namespace.as_bytes()),
        physics_body_id,
        &bootstrap.tick_rate_profile,
        &bootstrap.authoritative_numeric_profile,
        &bootstrap.physics_quantization_profile,
        include_wall,
    );
    let runtime = RuntimeState::new(bootstrap, authority).expect("physical runtime");
    PhysicalFixture {
        runtime,
        principal,
        source_id,
        controller_id,
        physics_body_id,
        command_stream_id: stream_id,
        action_map_hash,
        context_stack_hash,
    }
}

fn grounded_test_checkpoint(
    world_id: PhysicsWorldId,
    capsule_body_id: PhysicsBodyIdV1,
    tick_rate: &TickRateProfileV1,
    numeric: &AuthoritativeNumericProfileV1,
    quantization: &PhysicsQuantizationProfileV1,
    include_wall: bool,
) -> PhysicsWorldCheckpointV1 {
    let material_id = SchemaId::new("nextengine.physics.material.reference-zero").expect("id");
    let material = PhysicsMaterialDescriptorV1 {
        material_id: material_id.clone(),
        descriptor_revision: 1,
        static_friction_q16: 0,
        dynamic_friction_q16: 0,
        restitution_q16: 0,
        canonical_material_tags: Vec::new(),
    };
    let capsule_shape_id = PhysicsShapeIdV1 {
        body_id: capsule_body_id,
        shape_slot: 0,
    };
    let capsule_shape = PhysicsShapeDescriptorV1 {
        shape_id: capsule_shape_id,
        descriptor_revision: 1,
        local_pose: PhysicsPoseV1::default(),
        geometry: PhysicsGeometryV1::Capsule {
            radius_micrometres: 300_000,
            half_segment_micrometres: 600_000,
        },
        material_id: material_id.clone(),
        collision_layer: 0,
        collision_mask: 1,
        participation: PhysicsParticipationV1::Solid,
        contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
    };
    let capsule_pose = PhysicsPoseV1 {
        translation_micrometres: [0, 900_000, 0],
        ..PhysicsPoseV1::default()
    };
    let capsule = PhysicsBodyDescriptorV1 {
        body_id: capsule_body_id,
        descriptor_revision: 1,
        motion_kind: PhysicsMotionKindV1::Kinematic,
        initial_pose: capsule_pose,
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        shapes: BTreeMap::from([(capsule_shape_id, capsule_shape)]),
    };
    let floor_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([22; 16]),
        body_slot: 0,
    };
    let floor_shape_id = PhysicsShapeIdV1 {
        body_id: floor_body_id,
        shape_slot: 0,
    };
    let floor_pose = PhysicsPoseV1 {
        translation_micrometres: [0, -100_000, 0],
        ..PhysicsPoseV1::default()
    };
    let floor_shape = PhysicsShapeDescriptorV1 {
        shape_id: floor_shape_id,
        descriptor_revision: 1,
        local_pose: PhysicsPoseV1::default(),
        geometry: PhysicsGeometryV1::Box {
            half_extents_micrometres: [10_000_000, 100_000, 10_000_000],
        },
        material_id: material_id.clone(),
        collision_layer: 0,
        collision_mask: 1,
        participation: PhysicsParticipationV1::Solid,
        contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
    };
    let floor = PhysicsBodyDescriptorV1 {
        body_id: floor_body_id,
        descriptor_revision: 1,
        motion_kind: PhysicsMotionKindV1::Static,
        initial_pose: floor_pose,
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        shapes: BTreeMap::from([(floor_shape_id, floor_shape)]),
    };
    let mut bodies = BTreeMap::from([(capsule_body_id, capsule), (floor_body_id, floor)]);
    if include_wall {
        let wall_body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([23; 16]),
            body_slot: 0,
        };
        let wall_shape_id = PhysicsShapeIdV1 {
            body_id: wall_body_id,
            shape_slot: 0,
        };
        let wall_shape = PhysicsShapeDescriptorV1 {
            shape_id: wall_shape_id,
            descriptor_revision: 1,
            local_pose: PhysicsPoseV1::default(),
            geometry: PhysicsGeometryV1::Box {
                half_extents_micrometres: [1_000_000, 900_000, 100_000],
            },
            material_id: material_id.clone(),
            collision_layer: 0,
            collision_mask: 1,
            participation: PhysicsParticipationV1::Solid,
            contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
        };
        bodies.insert(
            wall_body_id,
            PhysicsBodyDescriptorV1 {
                body_id: wall_body_id,
                descriptor_revision: 1,
                motion_kind: PhysicsMotionKindV1::Static,
                initial_pose: PhysicsPoseV1 {
                    translation_micrometres: [0, 900_000, 450_000],
                    ..PhysicsPoseV1::default()
                },
                initial_linear_velocity_micrometres_per_second: [0; 3],
                initial_angular_velocity_q16: [0; 3],
                active: true,
                shapes: BTreeMap::from([(wall_shape_id, wall_shape)]),
            },
        );
    }
    let catalog = PhysicsWorldCatalogV1::new(
        world_id,
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("coordinate"),
            limits: PhysicsLimitsProfileV1::reference_v1().expect("limits"),
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("solver"),
            tick_rate_hash: tick_rate.profile_hash().expect("tick hash"),
            authoritative_numeric_hash: numeric.profile_hash().expect("numeric hash"),
            quantization_hash: quantization.profile_hash().expect("quantization hash"),
        },
        BTreeMap::from([(material_id, material)]),
        bodies,
        BTreeMap::from([(capsule_body_id.subject_id, capsule_body_id)]),
    )
    .expect("catalog");
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(&catalog, tick_rate, numeric, quantization)
        .expect("snapshot");
    PhysicsWorldCheckpointV1::new(catalog, snapshot).expect("checkpoint")
}

pub(super) fn movement_sample(
    fixture: &PhysicalFixture,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    direction_q15: [i16; 2],
    wall_time: Option<i64>,
) -> InputSampleV1 {
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: sequence,
        action_map_hash: fixture.action_map_hash,
        action_map_revision: 1,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: 1,
        actions: vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("action"),
            phase,
            value: PlayerActionValueV1::Vector2Q15(direction_q15),
            semantic_occurrence_ordinal: 0,
        }],
    };
    InputSampleV1 {
        schema_version: 1,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
        source_id: fixture.source_id,
        source_sequence: sequence,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("schema"),
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes().expect("frame"),
        sampled_wall_time: wall_time,
    }
}
