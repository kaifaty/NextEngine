use std::collections::BTreeMap;

use next_contracts::command::IssuerPrincipal;
use next_contracts::ids::{
    CapabilityId, CommandStreamId, ContentHash, InputSourceId, PersistentId, PhysicsWorldId,
    PlayerPrincipalId, SchemaId, SystemId,
};
use next_contracts::input::{
    ActionMapManifestV1, InputContextStackV1, PLAYER_INTERACTION_SYSTEM_ID,
    PlayerControllerBindingV1,
};
use next_contracts::physics::{
    PHYSICAL_COMMAND_CAPABILITY_ID, PhysicsBodyDescriptorV1, PhysicsBodyIdV1,
    PhysicsCanonicalSnapshotV2, PhysicsContactReportingV1, PhysicsCoordinateProfileV1,
    PhysicsGeometryV1, PhysicsLimitsProfileV1, PhysicsMaterialDescriptorV1, PhysicsMotionKindV1,
    PhysicsParticipationV1, PhysicsPoseV1, PhysicsShapeDescriptorV1, PhysicsShapeIdV1,
    PhysicsSolverSemanticsProfileV1, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    PhysicsWorldCheckpointV1,
};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::rpg::RPG_COMMAND_CAPABILITY_ID;

use crate::{ReferenceGameError, build_reference_runtime_bootstrap};

#[derive(Clone, Debug)]
pub struct ReferenceGameSession {
    pub bootstrap: next_runtime::RuntimeBootstrapV3,
    pub authority: next_runtime::AuthorityRegistry,
    pub principal: IssuerPrincipal,
    pub movement_stream_id: CommandStreamId,
    pub rpg_stream_id: CommandStreamId,
    pub interaction_stream_id: CommandStreamId,
    pub source_id: InputSourceId,
    pub controller_id: PersistentId,
    pub body_id: PersistentId,
    pub physics_body_id: PhysicsBodyIdV1,
    pub interactive_object_id: PersistentId,
    pub npc_character_id: PersistentId,
    pub dialogue_id: PersistentId,
    pub quest_id: PersistentId,
    pub relationship_id: PersistentId,
    pub player_inventory_id: PersistentId,
    pub player_equipment_id: PersistentId,
    pub pickup_item_id: PersistentId,
    pub pickup_proxy_id: PersistentId,
    pub npc_inventory_id: PersistentId,
    pub npc_equipment_id: PersistentId,
    pub npc_weapon_item_id: PersistentId,
    pub agent_principal: IssuerPrincipal,
    pub agent_stream_id: CommandStreamId,
    pub action_map: ActionMapManifestV1,
    pub action_map_hash: ContentHash,
    pub context_stack: InputContextStackV1,
    pub context_stack_hash: ContentHash,
    pub activated_project: ActivatedProjectV2,
}

pub fn build_reference_game_session(
    activated_project: ActivatedProjectV2,
) -> Result<ReferenceGameSession, ReferenceGameError> {
    build_reference_game_session_with_profile(activated_project, false)
}

pub fn build_reference_game_session_with_profile(
    activated_project: ActivatedProjectV2,
    physx_compatible: bool,
) -> Result<ReferenceGameSession, ReferenceGameError> {
    let project_id = activated_project
        .composition_lock
        .project_id
        .as_str()
        .to_owned();
    let principal = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([0x51; 16]));
    let interaction_principal =
        IssuerPrincipal::InternalSystem(SystemId::new(PLAYER_INTERACTION_SYSTEM_ID)?);
    let agent_principal =
        IssuerPrincipal::InternalSystem(SystemId::new("nextengine.agent.planner")?);
    let base = build_reference_runtime_bootstrap(
        &project_id,
        [
            (
                principal.clone(),
                vec![
                    CapabilityId::new(PHYSICAL_COMMAND_CAPABILITY_ID)?,
                    CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)?,
                ],
            ),
            (
                interaction_principal.clone(),
                vec![CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)?],
            ),
            (
                agent_principal.clone(),
                vec![CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)?],
            ),
        ],
    )?;
    let movement_stream_id = base
        .stream_for(&principal)
        .expect("neutral fixture allocates its declared principal stream");
    let interaction_stream_id = base
        .stream_for(&interaction_principal)
        .expect("neutral fixture allocates the interaction system stream");
    let agent_stream_id = base
        .stream_for(&agent_principal)
        .expect("neutral fixture allocates the agent planner stream");
    let mut bootstrap = base.bootstrap;
    if physx_compatible {
        let quantization =
            next_contracts::physics::PhysicsQuantizationProfileV1::grounded_capsule_v2()?;
        let numeric = next_contracts::physics::AuthoritativeNumericProfileV1::grounded_capsule_v2(
            &quantization,
        )?;
        bootstrap.runtime_profile.numeric_profile_hash = numeric.profile_hash()?;
        bootstrap.runtime_profile.physics_quantization_profile_hash =
            quantization.profile_hash()?;
        bootstrap.world_identity.runtime_determinism_profile_hash =
            bootstrap.runtime_profile.profile_hash()?;
        bootstrap.authoritative_numeric_profile = numeric;
        bootstrap.physics_quantization_profile = quantization;
    }
    bootstrap.rpg_definitions = activated_project.rpg_definitions.clone();
    bootstrap.rpg_bindings.project_composition_lock_hash =
        activated_project.composition_lock.composition_lock_sha256;
    bootstrap.rpg_bindings.schema_registry_hash = activated_project
        .schema_registry
        .schema_registry_manifest_sha256;
    bootstrap
        .rpg_bindings
        .active_definition_policy_hashes
        .extend(
            activated_project
                .rpg_definitions
                .interactions
                .iter()
                .map(next_contracts::mechanics::interaction_definition_hash),
        );
    bootstrap
        .rpg_bindings
        .active_definition_policy_hashes
        .extend(
            activated_project
                .rpg_definitions
                .abilities
                .iter()
                .map(next_contracts::mechanics::ability_definition_hash),
        );
    bootstrap
        .rpg_bindings
        .active_definition_policy_hashes
        .sort_unstable();
    bootstrap
        .rpg_bindings
        .active_definition_policy_hashes
        .dedup();
    let rpg_stream_id = bootstrap
        .stream_registry
        .allocate_stream(principal.clone())?;
    let source_id = InputSourceId::from_bytes([0x52; 16]);
    let controller_id = PersistentId::from_bytes([0x53; 16]);
    let body_id = PersistentId::from_bytes([0x54; 16]);
    let action_map = ActionMapManifestV1::core_keyboard_mouse_v1()?;
    let action_map_hash = action_map.content_hash;
    let context_stack = InputContextStackV1::gameplay_v1()?;
    let context_stack_hash = context_stack.content_hash;
    bootstrap.player_controller_registry.bindings.insert(
        source_id,
        PlayerControllerBindingV1 {
            principal: principal.clone(),
            source_id,
            controller_id,
            controlled_body_id: body_id,
            command_stream_id: movement_stream_id,
            action_map_hash,
            action_map_revision: action_map.revision,
            context_stack_hash,
            context_stack_revision: context_stack.revision,
            action_map: action_map.clone(),
            context_stack: context_stack.clone(),
        },
    );
    let physics_body_id = PhysicsBodyIdV1 {
        subject_id: body_id,
        body_slot: 0,
    };
    let interactive_object_id = PersistentId::from_bytes([0x58; 16]);
    let npc_character_id = PersistentId::from_bytes([0x59; 16]);
    let dialogue_id = PersistentId::from_bytes([0x5a; 16]);
    let quest_id = PersistentId::from_bytes([0x5b; 16]);
    let relationship_id = PersistentId::from_bytes([0x5c; 16]);
    let player_inventory_id = PersistentId::from_bytes([0x5d; 16]);
    let player_equipment_id = PersistentId::from_bytes([0x5e; 16]);
    let pickup_item_id = PersistentId::from_bytes([0x5f; 16]);
    let pickup_proxy_id = PersistentId::from_bytes([0x60; 16]);
    let npc_inventory_id = PersistentId::from_bytes([0x61; 16]);
    let npc_equipment_id = PersistentId::from_bytes([0x62; 16]);
    let npc_weapon_item_id = PersistentId::from_bytes([0x63; 16]);
    bootstrap.physics_checkpoint = grounded_capsule_checkpoint(
        PhysicsWorldId::from_bytes(*bootstrap.world_identity.world_namespace.as_bytes()),
        physics_body_id,
        &bootstrap.tick_rate_profile,
        &bootstrap.authoritative_numeric_profile,
        &bootstrap.physics_quantization_profile,
    )?;
    Ok(ReferenceGameSession {
        bootstrap,
        authority: base.authority,
        principal,
        movement_stream_id,
        rpg_stream_id,
        interaction_stream_id,
        source_id,
        controller_id,
        body_id,
        physics_body_id,
        interactive_object_id,
        npc_character_id,
        dialogue_id,
        quest_id,
        relationship_id,
        player_inventory_id,
        player_equipment_id,
        pickup_item_id,
        pickup_proxy_id,
        npc_inventory_id,
        npc_equipment_id,
        npc_weapon_item_id,
        agent_principal,
        agent_stream_id,
        action_map,
        action_map_hash,
        context_stack,
        context_stack_hash,
        activated_project,
    })
}

fn grounded_capsule_checkpoint(
    world_id: PhysicsWorldId,
    capsule_body_id: PhysicsBodyIdV1,
    tick_rate: &next_contracts::input::TickRateProfileV1,
    numeric: &next_contracts::physics::AuthoritativeNumericProfileV1,
    quantization: &next_contracts::physics::PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldCheckpointV1, ReferenceGameError> {
    let material_id = SchemaId::new("nextengine.physics.material.reference-zero")?;
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
        subject_id: PersistentId::from_bytes([0x57; 16]),
        body_slot: 0,
    };
    let floor_shape_id = PhysicsShapeIdV1 {
        body_id: floor_body_id,
        shape_slot: 0,
    };
    let floor = static_box_descriptor(
        floor_body_id,
        floor_shape_id,
        &material_id,
        [0, -100_000, 0],
        [10_000_000, 100_000, 10_000_000],
    );
    let wall_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x58; 16]),
        body_slot: 0,
    };
    let wall_shape_id = PhysicsShapeIdV1 {
        body_id: wall_body_id,
        shape_slot: 0,
    };
    let wall = static_box_descriptor(
        wall_body_id,
        wall_shape_id,
        &material_id,
        [0, 900_000, 700_000],
        [10_000_000, 10_000_000, 100_000],
    );
    let pickup_proxy_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x60; 16]),
        body_slot: 0,
    };
    let pickup_proxy_shape_id = PhysicsShapeIdV1 {
        body_id: pickup_proxy_body_id,
        shape_slot: 0,
    };
    let pickup_proxy = static_box_descriptor(
        pickup_proxy_body_id,
        pickup_proxy_shape_id,
        &material_id,
        [0, 900_000, 700_000],
        [100_000, 900_000, 100_000],
    );
    let npc_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x59; 16]),
        body_slot: 0,
    };
    let npc_shape_id = PhysicsShapeIdV1 {
        body_id: npc_body_id,
        shape_slot: 0,
    };
    let npc = static_box_descriptor(
        npc_body_id,
        npc_shape_id,
        &material_id,
        [700_000, 900_000, 200_000],
        [100_000, 900_000, 100_000],
    );
    let catalog = PhysicsWorldCatalogV1::new(
        world_id,
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1()?,
            limits: PhysicsLimitsProfileV1::reference_v1()?,
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1()?,
            tick_rate_hash: tick_rate.profile_hash()?,
            authoritative_numeric_hash: numeric.profile_hash()?,
            quantization_hash: quantization.profile_hash()?,
        },
        BTreeMap::from([(material_id, material)]),
        BTreeMap::from([
            (capsule_body_id, capsule),
            (floor_body_id, floor),
            (wall_body_id, wall),
            (pickup_proxy_body_id, pickup_proxy),
            (npc_body_id, npc),
        ]),
        BTreeMap::from([(capsule_body_id.subject_id, capsule_body_id)]),
    )?;
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(&catalog, tick_rate, numeric, quantization)?;
    Ok(PhysicsWorldCheckpointV1::new(catalog, snapshot)?)
}

fn static_box_descriptor(
    body_id: PhysicsBodyIdV1,
    shape_id: PhysicsShapeIdV1,
    material_id: &SchemaId,
    translation_micrometres: [i64; 3],
    half_extents_micrometres: [i64; 3],
) -> PhysicsBodyDescriptorV1 {
    PhysicsBodyDescriptorV1 {
        body_id,
        descriptor_revision: 1,
        motion_kind: PhysicsMotionKindV1::Static,
        initial_pose: PhysicsPoseV1 {
            translation_micrometres,
            ..PhysicsPoseV1::default()
        },
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        shapes: BTreeMap::from([(
            shape_id,
            PhysicsShapeDescriptorV1 {
                shape_id,
                descriptor_revision: 1,
                local_pose: PhysicsPoseV1::default(),
                geometry: PhysicsGeometryV1::Box {
                    half_extents_micrometres,
                },
                material_id: material_id.clone(),
                collision_layer: 0,
                collision_mask: 1,
                participation: PhysicsParticipationV1::Solid,
                contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
            },
        )]),
    }
}
