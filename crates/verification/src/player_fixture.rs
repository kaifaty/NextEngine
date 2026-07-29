use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

use next_contracts::{
    ActivatedProjectV1, AssetId, AssetRevisionRefV1, CORE_CHARACTER_HEALTH_RESOURCE_ID,
    CORE_EQUIP_USE_ACTION_ID, CORE_EQUIPMENT_MAIN_HAND_SLOT_ID, CORE_INTERACT_ACTION_ID,
    CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID, CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID,
    CORE_INTERACTIVE_OBJECT_READY_STATE_ID, CORE_MELEE_ACTION_ID, CORE_MOVE_ACTION_ID,
    CORE_PICKUP_ACTION_ID, CapabilityId, CharacterPayloadV1, CharacterResourceEntryV1,
    CommandLedgerHash, CommandStreamId, ContactPhaseV1, ContentHash, DefinitionRefV1,
    DialoguePayloadV1, EquipmentPayloadV1, EventPayload, InputSampleV1, InputSourceId,
    InteractiveObjectPayloadV1, InventoryPayloadV1, IssuerPrincipal, ItemPayloadV1,
    PHYSICAL_COMMAND_CAPABILITY_ID, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS, PLAYER_INTERACTION_SYSTEM_ID,
    PersistentId, PhysicsBodyDescriptorV1, PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2,
    PhysicsContactReportingV1, PhysicsCoordinateProfileV1, PhysicsGeometryV1,
    PhysicsLimitsProfileV1, PhysicsMaterialDescriptorV1, PhysicsMotionKindV1,
    PhysicsParticipationV1, PhysicsPoseV1, PhysicsShapeDescriptorV1, PhysicsShapeIdV1,
    PhysicsSolverSemanticsProfileV1, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    PhysicsWorldCheckpointV1, PhysicsWorldId, PlayerActionFrameV1, PlayerActionPhaseV1,
    PlayerActionV1, PlayerActionValueV1, PlayerControllerBindingV1, PlayerPrincipalId,
    ProvenanceBindingV1, QuestPayloadV1, RPG_COMMAND_CAPABILITY_ID, RelationshipDimensionV1,
    RelationshipPayloadV1, RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1,
    RpgPhysicalContactFactV1, RpgSnapshotV2, SchemaId, StateRoot, SystemId,
    core_player_action_map_v2_hash, domain_hash,
};
use next_physics_api::PhysicsBackendPolicy;
use next_presentation::{PresentationBindingV1, PresentationExtractorV1};
use next_render::{ReferenceB0Renderer, RenderDevice, RenderTargetV1};
use next_runtime::PhysicsLaunchOptions;
use next_runtime::{RuntimeFatalError, RuntimeState, SnapshotRestoreError};
use next_world::{WorldStreamerV1, WorldStreamingError};

use crate::{NeutralFixtureError, build_neutral_runtime_fixture, compute_world_checkpoint_root};

#[derive(Clone, Debug)]
pub struct NeutralPlayerFixture {
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
    pub action_map_hash: ContentHash,
    pub context_stack_hash: ContentHash,
    pub activated_project: ActivatedProjectV1,
}

static PROJECT_FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn build_neutral_player_fixture(
    project_id: &str,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    build_neutral_player_fixture_with_profile(project_id, false)
}

pub fn build_physx_player_fixture(
    project_id: &str,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    build_neutral_player_fixture_with_profile(project_id, true)
}

fn build_neutral_player_fixture_with_profile(
    project_id: &str,
    physx_compatible: bool,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    let activated_project = activate_fixture_project(project_id)?;
    build_neutral_player_fixture_with_activated_project(activated_project, physx_compatible)
}

pub fn build_neutral_player_fixture_from_activated_project(
    activated_project: ActivatedProjectV1,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    build_neutral_player_fixture_with_activated_project(activated_project, false)
}

fn build_neutral_player_fixture_with_activated_project(
    activated_project: ActivatedProjectV1,
    physx_compatible: bool,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
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
    let base = build_neutral_runtime_fixture(
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
        let quantization = next_contracts::PhysicsQuantizationProfileV1::grounded_capsule_v2()?;
        let numeric =
            next_contracts::AuthoritativeNumericProfileV1::grounded_capsule_v2(&quantization)?;
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
                .map(next_contracts::interaction_definition_hash),
        );
    bootstrap
        .rpg_bindings
        .active_definition_policy_hashes
        .extend(
            activated_project
                .rpg_definitions
                .abilities
                .iter()
                .map(next_contracts::ability_definition_hash),
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
    let action_map_hash = core_player_action_map_v2_hash();
    let context_stack_hash = ContentHash::from_bytes([0x56; 32]);
    bootstrap.player_controller_registry.bindings.insert(
        source_id,
        PlayerControllerBindingV1 {
            principal: principal.clone(),
            source_id,
            controller_id,
            controlled_body_id: body_id,
            command_stream_id: movement_stream_id,
            action_map_hash,
            action_map_revision: 1,
            context_stack_hash,
            context_stack_revision: 1,
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
    Ok(NeutralPlayerFixture {
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
        action_map_hash,
        context_stack_hash,
        activated_project,
    })
}

fn activate_fixture_project(project_id: &str) -> Result<ActivatedProjectV1, NeutralFixtureError> {
    let mut source = next_project::neutral_vertical_slice_source_v1()?;
    source.project_id = next_contracts::ProjectId::new(project_id)?;
    let cooked = next_project::cook_project_v1(source)?;
    let counter = PROJECT_FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let output = std::env::temp_dir().join(format!(
        "nextengine-play-project-{}-{counter}",
        std::process::id()
    ));
    let store = next_assets::ContentStore::new(&output);
    let result = (|| {
        store.publish(&cooked.publication()?)?;
        Ok(next_project::activate_project(&store)?)
    })();
    if output.exists() {
        std::fs::remove_dir_all(&output).map_err(next_assets::ContentStoreError::from)?;
    }
    result
}

fn grounded_capsule_checkpoint(
    world_id: PhysicsWorldId,
    capsule_body_id: PhysicsBodyIdV1,
    tick_rate: &next_contracts::TickRateProfileV1,
    numeric: &next_contracts::AuthoritativeNumericProfileV1,
    quantization: &next_contracts::PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldCheckpointV1, NeutralFixtureError> {
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

#[must_use]
pub fn cooked_project_rpg_snapshot(fixture: &NeutralPlayerFixture) -> RpgSnapshotV2 {
    let definitions = &fixture.activated_project.rpg_definitions;
    let dialogue_definition = definitions
        .dialogues
        .first()
        .expect("cooked fixture has one dialogue definition");
    let quest_definition = definitions
        .quests
        .first()
        .expect("cooked fixture has one quest definition");
    let relationship_definition = definitions
        .relationships
        .first()
        .expect("cooked fixture has one relationship definition");
    let interaction_definition = definitions
        .interactions
        .first()
        .expect("cooked fixture has one interaction definition");
    let ability_definition = definitions
        .abilities
        .first()
        .expect("cooked fixture has one ability definition");
    let health_resource = || CharacterResourceEntryV1 {
        resource_id: SchemaId::new(CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .expect("engine-owned health resource is valid"),
        current_value: 100,
        minimum_value: 0,
        maximum_value: 100,
    };
    let mut aggregates = vec![
        fixture_aggregate(
            fixture.body_id,
            0x54,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(fixture.player_inventory_id),
                equipment_id: Some(fixture.player_equipment_id),
                resources: vec![health_resource()],
                skills: Vec::new(),
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.pickup_item_id,
            ability_definition.required_item_definition,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 100,
                custom_state: Vec::new(),
            }),
        ),
        fixture_aggregate(
            fixture.player_inventory_id,
            0x5d,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: fixture.body_id,
                capacity: 8,
                item_ids: Vec::new(),
                reservations: Vec::new(),
            }),
        ),
        fixture_aggregate(
            fixture.player_equipment_id,
            0x5e,
            RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                character_id: fixture.body_id,
                slot_policy: DefinitionRefV1::Exact {
                    asset_id: AssetId::from_bytes([0x5e; 16]),
                    content_hash: next_runtime::bootstrap_equipment_slot_policy_hash_v1(),
                },
                assignments: Vec::new(),
            }),
        ),
        fixture_aggregate(
            fixture.npc_character_id,
            0x59,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(fixture.npc_inventory_id),
                equipment_id: Some(fixture.npc_equipment_id),
                resources: vec![health_resource()],
                skills: Vec::new(),
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.npc_weapon_item_id,
            ability_definition.required_item_definition,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 100,
                custom_state: Vec::new(),
            }),
        ),
        fixture_aggregate(
            fixture.npc_inventory_id,
            0x61,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: fixture.npc_character_id,
                capacity: 1,
                item_ids: vec![fixture.npc_weapon_item_id],
                reservations: Vec::new(),
            }),
        ),
        fixture_aggregate(
            fixture.npc_equipment_id,
            0x62,
            RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                character_id: fixture.npc_character_id,
                slot_policy: DefinitionRefV1::Exact {
                    asset_id: AssetId::from_bytes([0x62; 16]),
                    content_hash: next_runtime::bootstrap_equipment_slot_policy_hash_v1(),
                },
                assignments: vec![next_contracts::EquipmentSlotAssignmentV1 {
                    slot_id: ability_definition.required_equipment_slot_id.clone(),
                    item_id: fixture.npc_weapon_item_id,
                }],
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.quest_id,
            quest_definition.asset_revision,
            RpgAggregatePayloadV1::Quest(QuestPayloadV1 {
                state_id: quest_definition.entry_state_id.clone(),
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.dialogue_id,
            dialogue_definition.asset_revision,
            RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
                speaker_id: fixture.npc_character_id,
                listener_id: fixture.body_id,
                node_id: dialogue_definition.entry_node_id.clone(),
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.relationship_id,
            relationship_definition.asset_revision,
            RpgAggregatePayloadV1::Relationship(RelationshipPayloadV1 {
                source_id: fixture.npc_character_id,
                target_id: fixture.body_id,
                dimensions: vec![RelationshipDimensionV1 {
                    dimension_id: relationship_definition.dimension_id.clone(),
                    value: interaction_definition.relationship_source_value,
                }],
            }),
        ),
        fixture_aggregate(
            fixture.interactive_object_id,
            0x58,
            RpgAggregatePayloadV1::InteractiveObject(InteractiveObjectPayloadV1 {
                state_id: SchemaId::new(CORE_INTERACTIVE_OBJECT_READY_STATE_ID)
                    .expect("built-in interactive-object state is valid"),
                linked_item_id: None,
            }),
        ),
        fixture_aggregate(
            fixture.pickup_proxy_id,
            0x60,
            RpgAggregatePayloadV1::InteractiveObject(InteractiveObjectPayloadV1 {
                state_id: SchemaId::new(CORE_INTERACTIVE_OBJECT_READY_STATE_ID)
                    .expect("built-in interactive-object state is valid"),
                linked_item_id: Some(fixture.pickup_item_id),
            }),
        ),
    ];
    aggregates.sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
    RpgSnapshotV2 { aggregates }
}

#[must_use]
pub fn cooked_interaction_outcome(
    fixture: &NeutralPlayerFixture,
) -> (SchemaId, SchemaId, SchemaId, i32) {
    let definitions = &fixture.activated_project.rpg_definitions;
    let interaction = definitions
        .interactions
        .first()
        .expect("cooked fixture has one interaction definition");
    let dialogue = definitions
        .dialogue(interaction.dialogue_definition)
        .expect("interaction dialogue definition is closed");
    let quest = definitions
        .quest(interaction.quest_definition)
        .expect("interaction quest definition is closed");
    let relationship = definitions
        .relationship(interaction.relationship_definition)
        .expect("interaction relationship definition is closed");
    let dialogue_target = dialogue
        .transitions
        .iter()
        .find(|transition| transition.transition_id == interaction.dialogue_transition_id)
        .expect("dialogue transition is closed")
        .target_state_id
        .clone();
    let quest_target = quest
        .transitions
        .iter()
        .find(|transition| transition.transition_id == interaction.quest_transition_id)
        .expect("quest transition is closed")
        .target_state_id
        .clone();
    (
        dialogue_target,
        quest_target,
        relationship.dimension_id.clone(),
        interaction
            .relationship_source_value
            .checked_add(interaction.relationship_delta)
            .expect("cooked relationship outcome fits i32"),
    )
}

fn fixture_aggregate_from_asset(
    persistent_id: PersistentId,
    definition: AssetRevisionRefV1,
    payload: RpgAggregatePayloadV1,
) -> RpgAggregateEnvelopeV1 {
    RpgAggregateEnvelopeV1::new(
        persistent_id,
        1,
        0,
        DefinitionRefV1::Exact {
            asset_id: definition.asset_id,
            content_hash: definition.record_sha256,
        },
        ProvenanceBindingV1::None,
        payload,
    )
    .expect("cooked fixture aggregate is canonical")
}

pub(crate) fn fixture_aggregate(
    persistent_id: PersistentId,
    definition_seed: u8,
    payload: RpgAggregatePayloadV1,
) -> RpgAggregateEnvelopeV1 {
    RpgAggregateEnvelopeV1::new(
        persistent_id,
        1,
        0,
        DefinitionRefV1::Exact {
            asset_id: AssetId::from_bytes([definition_seed; 16]),
            content_hash: ContentHash::from_bytes([definition_seed; 32]),
        },
        ProvenanceBindingV1::Exact(ContentHash::from_bytes([definition_seed; 32])),
        payload,
    )
    .expect("built-in fixture aggregate is valid")
}

fn aggregate_payload(
    snapshot: &RpgSnapshotV2,
    kind: RpgAggregateKindV1,
    persistent_id: PersistentId,
) -> Option<&RpgAggregatePayloadV1> {
    snapshot
        .aggregates
        .iter()
        .find(|aggregate| {
            aggregate.aggregate_kind == kind && aggregate.persistent_id == persistent_id
        })
        .map(|aggregate| &aggregate.payload)
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

pub fn player_action_sample(
    fixture: &NeutralPlayerFixture,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    direction_q15: [i16; 2],
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, CanonicalFixtureError> {
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: sequence,
        action_map_hash: fixture.action_map_hash,
        action_map_revision: 1,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: 1,
        actions: vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_MOVE_ACTION_ID)?,
            phase,
            value: PlayerActionValueV1::Vector2Q15(direction_q15),
            semantic_occurrence_ordinal: 0,
        }],
    };
    Ok(InputSampleV1 {
        schema_version: 1,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS)?,
        source_id: fixture.source_id,
        source_sequence: sequence,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID)?,
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes()?,
        sampled_wall_time,
    })
}

pub fn player_interact_sample(
    fixture: &NeutralPlayerFixture,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, CanonicalFixtureError> {
    player_semantic_action_sample(
        fixture,
        sequence,
        CORE_INTERACT_ACTION_ID,
        phase,
        pressed,
        sampled_wall_time,
    )
}

pub fn player_pickup_sample(
    fixture: &NeutralPlayerFixture,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, CanonicalFixtureError> {
    player_semantic_action_sample(
        fixture,
        sequence,
        CORE_PICKUP_ACTION_ID,
        phase,
        pressed,
        sampled_wall_time,
    )
}

pub fn player_equip_use_sample(
    fixture: &NeutralPlayerFixture,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, CanonicalFixtureError> {
    player_semantic_action_sample(
        fixture,
        sequence,
        CORE_EQUIP_USE_ACTION_ID,
        phase,
        pressed,
        sampled_wall_time,
    )
}

pub fn player_melee_sample(
    fixture: &NeutralPlayerFixture,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, CanonicalFixtureError> {
    player_semantic_action_sample(
        fixture,
        sequence,
        CORE_MELEE_ACTION_ID,
        phase,
        pressed,
        sampled_wall_time,
    )
}

fn player_semantic_action_sample(
    fixture: &NeutralPlayerFixture,
    sequence: u64,
    action_id: &str,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, CanonicalFixtureError> {
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: sequence,
        action_map_hash: fixture.action_map_hash,
        action_map_revision: 1,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: 1,
        actions: vec![PlayerActionV1 {
            action_id: SchemaId::new(action_id)?,
            phase,
            value: PlayerActionValueV1::Digital(pressed),
            semantic_occurrence_ordinal: 0,
        }],
    };
    Ok(InputSampleV1 {
        schema_version: 1,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS)?,
        source_id: fixture.source_id,
        source_sequence: sequence,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID)?,
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes()?,
        sampled_wall_time,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayCheckReport {
    pub ticks: u64,
    pub final_pose: PhysicsPoseV1,
    pub events: u64,
    pub rpg_events: u64,
    pub interactive_object_state: SchemaId,
    pub dialogue_node_id: SchemaId,
    pub quest_state_id: SchemaId,
    pub npc_player_trust: i32,
    pub npc_health: i32,
    pub player_health: i32,
    pub agent_intent_id: ContentHash,
    pub agent_projection_hash: ContentHash,
    pub world_streaming_generation: u64,
    pub current_chunk_id: SchemaId,
    pub final_command_ledger_hash: CommandLedgerHash,
    pub final_state_root: StateRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameCheckReport {
    pub play: PlayCheckReport,
    pub presentation_snapshot_hash: ContentHash,
    pub rendered_object_count: u32,
    pub frame_plan_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedGameFrameV1 {
    pub check: GameCheckReport,
    pub snapshot: next_contracts::PresentationSnapshotV2,
}

pub fn run_play_check() -> Result<PlayCheckReport, PlayCheckError> {
    let scenario = run_grounded_collision_scenario(true)?;
    play_check_report(scenario)
}

pub fn run_game_check() -> Result<GameCheckReport, PlayCheckError> {
    Ok(prepare_game_frame()?.check)
}

pub fn prepare_game_frame() -> Result<PreparedGameFrameV1, PlayCheckError> {
    let scenario = run_grounded_collision_scenario(true)?;
    prepare_game_frame_from_scenario(scenario)
}

pub fn prepare_game_frame_with_activated_project(
    activated_project: ActivatedProjectV1,
) -> Result<PreparedGameFrameV1, PlayCheckError> {
    let project_id = activated_project
        .composition_lock
        .project_id
        .as_str()
        .to_owned();
    let scenario = run_grounded_collision_scenario_with_backend(
        true,
        &project_id,
        false,
        PhysicsLaunchOptions::default(),
        Some(activated_project),
    )?;
    prepare_game_frame_from_scenario(scenario)
}

fn prepare_game_frame_from_scenario(
    scenario: GroundedCollisionScenario,
) -> Result<PreparedGameFrameV1, PlayCheckError> {
    let mut extractor = PresentationExtractorV1::new(
        scenario.project_composition_lock_hash,
        domain_hash(
            "nextengine.presentation-profile.b0.v1",
            b"sdr-reference-no-optional-features",
        ),
        8,
    )?;
    let snapshot = extractor
        .extract(
            scenario.ticks,
            scenario.project_composition_lock_hash,
            scenario.content_manifest_hash,
            scenario.runtime.physics_snapshot(),
            &scenario.presentation_bindings,
        )?
        .clone();
    let mut renderer = ReferenceB0Renderer::new();
    let frame = renderer.render(
        &snapshot,
        RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        },
    )?;
    let play = play_check_report(scenario)?;
    let check = GameCheckReport {
        play,
        presentation_snapshot_hash: snapshot.canonical_hash,
        rendered_object_count: frame.rendered_object_count,
        frame_plan_hash: frame.frame_plan_hash,
    };
    Ok(PreparedGameFrameV1 { check, snapshot })
}

pub fn run_play_check_with_activated_project(
    activated_project: ActivatedProjectV1,
) -> Result<PlayCheckReport, PlayCheckError> {
    let project_id = activated_project
        .composition_lock
        .project_id
        .as_str()
        .to_owned();
    let scenario = run_grounded_collision_scenario_with_backend(
        true,
        &project_id,
        false,
        PhysicsLaunchOptions::default(),
        Some(activated_project),
    )?;
    play_check_report(scenario)
}

fn play_check_report(
    scenario: GroundedCollisionScenario,
) -> Result<PlayCheckReport, PlayCheckError> {
    let checkpoint = scenario.runtime.world_checkpoint()?;
    let rpg = scenario.runtime.rpg_snapshot();
    let interactive_object_state = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::InteractiveObject,
        scenario.interactive_object_id,
    ) {
        Some(RpgAggregatePayloadV1::InteractiveObject(object)) => object.state_id.clone(),
        _ => return Err(PlayCheckError::InteractiveObjectMissing),
    };
    let dialogue_node_id =
        match aggregate_payload(&rpg, RpgAggregateKindV1::Dialogue, scenario.dialogue_id) {
            Some(RpgAggregatePayloadV1::Dialogue(dialogue)) => dialogue.node_id.clone(),
            _ => return Err(PlayCheckError::CookedDialogueMissing),
        };
    let quest_state_id = match aggregate_payload(&rpg, RpgAggregateKindV1::Quest, scenario.quest_id)
    {
        Some(RpgAggregatePayloadV1::Quest(quest)) => quest.state_id.clone(),
        _ => return Err(PlayCheckError::CookedQuestMissing),
    };
    let npc_player_trust = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Relationship,
        scenario.relationship_id,
    ) {
        Some(RpgAggregatePayloadV1::Relationship(relationship))
            if relationship.source_id == scenario.npc_character_id
                && relationship.target_id == scenario.player_character_id =>
        {
            relationship
                .dimensions
                .iter()
                .find(|dimension| dimension.dimension_id == scenario.relationship_dimension_id)
                .map_or(0, |dimension| dimension.value)
        }
        _ => return Err(PlayCheckError::CookedNpcMissing),
    };
    let npc_health = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Character,
        scenario.npc_character_id,
    ) {
        Some(RpgAggregatePayloadV1::Character(character)) => character
            .resources
            .iter()
            .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .map_or(0, |resource| resource.current_value),
        _ => return Err(PlayCheckError::CookedNpcMissing),
    };
    let player_health = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Character,
        scenario.player_character_id,
    ) {
        Some(RpgAggregatePayloadV1::Character(character)) => character
            .resources
            .iter()
            .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .map_or(0, |resource| resource.current_value),
        _ => return Err(PlayCheckError::CookedPlayerMissing),
    };
    Ok(PlayCheckReport {
        ticks: scenario.ticks,
        final_pose: scenario.final_pose,
        events: scenario.events,
        rpg_events: scenario.rpg_events,
        interactive_object_state,
        dialogue_node_id,
        quest_state_id,
        npc_player_trust,
        npc_health,
        player_health,
        agent_intent_id: scenario
            .agent_intent_id
            .ok_or(PlayCheckError::AgentActionMissing)?,
        agent_projection_hash: scenario
            .agent_projection_hash
            .ok_or(PlayCheckError::AgentActionMissing)?,
        world_streaming_generation: scenario.world_streaming_snapshot.generation,
        current_chunk_id: scenario.world_streaming_snapshot.current_chunk_id.clone(),
        final_command_ledger_hash: checkpoint.runtime_snapshot.command_ledger_hash()?,
        final_state_root: next_contracts::world_checkpoint_with_streaming_v1_state_root(
            &checkpoint.runtime_snapshot,
            &checkpoint.rpg_snapshot,
            &checkpoint.physics_checkpoint,
            &scenario.world_streaming_snapshot,
        )?,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsCollisionCheckReport {
    pub gameplay_ticks: u64,
    pub physics_substeps: u64,
    pub begin_contacts: u64,
    pub persist_contacts: u64,
    pub end_contacts: u64,
    pub final_pose: PhysicsPoseV1,
    pub contact_batches_hash: ContentHash,
    pub physics_checkpoint_hash: ContentHash,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PhysicsCollisionBackend {
    #[default]
    Reference,
    PhysX,
    Compare,
}

pub fn run_physics_collision_check() -> Result<PhysicsCollisionCheckReport, PlayCheckError> {
    run_physics_collision_check_with_backend(PhysicsCollisionBackend::Reference)
}

pub fn run_physics_collision_check_with_backend(
    backend: PhysicsCollisionBackend,
) -> Result<PhysicsCollisionCheckReport, PlayCheckError> {
    let scenario = match backend {
        PhysicsCollisionBackend::Reference => run_grounded_collision_scenario(false)?,
        PhysicsCollisionBackend::PhysX => run_grounded_collision_scenario_with_backend(
            false,
            "nextengine.physics-collision.physx",
            true,
            PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX),
            None,
        )?,
        PhysicsCollisionBackend::Compare => {
            let reference = run_grounded_collision_scenario_with_backend(
                false,
                "nextengine.physics-collision.compare",
                true,
                PhysicsLaunchOptions::new(PhysicsBackendPolicy::ReferenceOnly),
                None,
            )?;
            let physx = run_grounded_collision_scenario_with_backend(
                false,
                "nextengine.physics-collision.compare",
                true,
                PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX),
                None,
            )?;
            if reference.tick_reports != physx.tick_reports {
                return Err(PlayCheckError::BackendParityMismatch);
            }
            let reference_checkpoint = reference.runtime.world_checkpoint()?;
            let physx_checkpoint = physx.runtime.world_checkpoint()?;
            if reference_checkpoint != physx_checkpoint
                || compute_world_checkpoint_root(&reference_checkpoint)?
                    != compute_world_checkpoint_root(&physx_checkpoint)?
            {
                return Err(PlayCheckError::BackendParityMismatch);
            }
            let reference_replay =
                crate::persistence_replay::run_persistence_replay_check_for_project(
                    crate::PersistenceReplayBackend::Reference,
                    "nextengine.physics-collision.compare-replay",
                    true,
                )?;
            let physx_replay = crate::persistence_replay::run_persistence_replay_check_for_project(
                crate::PersistenceReplayBackend::PhysX,
                "nextengine.physics-collision.compare-replay",
                true,
            )?;
            if reference_replay != physx_replay {
                return Err(PlayCheckError::BackendParityMismatch);
            }
            physx
        }
    };
    Ok(PhysicsCollisionCheckReport {
        gameplay_ticks: scenario.ticks,
        physics_substeps: scenario.runtime.physics_snapshot().physics_tick,
        begin_contacts: scenario.begin_contacts,
        persist_contacts: scenario.persist_contacts,
        end_contacts: scenario.end_contacts,
        final_pose: scenario.final_pose,
        contact_batches_hash: scenario.contact_batches_hash,
        physics_checkpoint_hash: scenario.runtime.physics_checkpoint().checkpoint_hash()?,
    })
}

struct GroundedCollisionScenario {
    runtime: RuntimeState,
    ticks: u64,
    final_pose: PhysicsPoseV1,
    events: u64,
    rpg_events: u64,
    begin_contacts: u64,
    persist_contacts: u64,
    end_contacts: u64,
    contact_batches_hash: ContentHash,
    interactive_object_id: PersistentId,
    npc_character_id: PersistentId,
    player_character_id: PersistentId,
    dialogue_id: PersistentId,
    quest_id: PersistentId,
    relationship_id: PersistentId,
    relationship_dimension_id: SchemaId,
    project_composition_lock_hash: ContentHash,
    content_manifest_hash: ContentHash,
    presentation_bindings: Vec<PresentationBindingV1>,
    tick_reports: Vec<next_runtime::TickReport>,
    world_streaming_snapshot: next_contracts::WorldStreamingSnapshotV1,
    agent_intent_id: Option<ContentHash>,
    agent_projection_hash: Option<ContentHash>,
}

enum ScenarioAction {
    Movement(PlayerActionPhaseV1, [i16; 2]),
    Interaction,
    Pickup,
    EquipUse,
    Melee,
    AgentMelee,
    ChunkTransition(SchemaId, bool),
}

fn run_grounded_collision_scenario(
    include_interaction: bool,
) -> Result<GroundedCollisionScenario, PlayCheckError> {
    run_grounded_collision_scenario_with_backend(
        include_interaction,
        "nextengine.play",
        false,
        PhysicsLaunchOptions::default(),
        None,
    )
}

fn run_grounded_collision_scenario_with_backend(
    include_interaction: bool,
    project_id: &str,
    physx_compatible: bool,
    physics_options: PhysicsLaunchOptions,
    activated_project: Option<ActivatedProjectV1>,
) -> Result<GroundedCollisionScenario, PlayCheckError> {
    let fixture = match activated_project {
        Some(project) => {
            debug_assert!(!physx_compatible);
            build_neutral_player_fixture_from_activated_project(project)?
        }
        None if physx_compatible => build_physx_player_fixture(project_id)?,
        None => build_neutral_player_fixture(project_id)?,
    };
    let (
        expected_dialogue_node_id,
        expected_quest_state_id,
        relationship_dimension_id,
        expected_relationship_value,
    ) = cooked_interaction_outcome(&fixture);
    let rpg_snapshot = if include_interaction {
        cooked_project_rpg_snapshot(&fixture)
    } else {
        RpgSnapshotV2::default()
    };
    let mut runtime = RuntimeState::with_rpg_snapshot_and_physics_options(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        rpg_snapshot,
        physics_options,
    )?;
    let initial_chunk_id = fixture
        .activated_project
        .world_partition
        .body
        .chunk_bindings
        .first()
        .ok_or(PlayCheckError::WorldPartitionEmpty)?
        .chunk_id
        .clone();
    let transition_chunk_id = fixture
        .activated_project
        .world_partition
        .body
        .chunk_bindings
        .get(1)
        .ok_or(PlayCheckError::WorldPartitionEmpty)?
        .chunk_id
        .clone();
    let mut world_streamer =
        WorldStreamerV1::activate(fixture.activated_project.clone(), initial_chunk_id.clone())?;
    let mut inputs = vec![
        ScenarioAction::Movement(PlayerActionPhaseV1::Started, [0, 32_767]),
        ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
        ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
        ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
    ];
    if include_interaction {
        inputs.extend([
            ScenarioAction::Pickup,
            ScenarioAction::EquipUse,
            ScenarioAction::ChunkTransition(transition_chunk_id, true),
            ScenarioAction::Interaction,
        ]);
        inputs.extend([
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, -32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Started, [32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [32_767, 0]),
            ScenarioAction::Melee,
            ScenarioAction::AgentMelee,
            ScenarioAction::Interaction,
            ScenarioAction::ChunkTransition(initial_chunk_id, false),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Completed, [0, 0]),
        ]);
    } else {
        inputs.extend([
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, -32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Completed, [0, 0]),
        ]);
    }
    let mut events = 0_u64;
    let mut rpg_events = 0_u64;
    let mut begin_contacts = 0_u64;
    let mut persist_contacts = 0_u64;
    let mut end_contacts = 0_u64;
    let mut contact_preimage = b"nextengine.physics-collision-check.contacts.v1\0".to_vec();
    let mut tick_reports: Vec<next_runtime::TickReport> = Vec::new();
    let mut sequence = 0_u64;
    let mut agent_intent_id = None;
    let mut agent_projection_hash = None;
    for action in inputs {
        if let ScenarioAction::ChunkTransition(target_chunk_id, save_restore) = action {
            let rpg_before = runtime.rpg_snapshot();
            let plan = world_streamer.begin_transition(target_chunk_id, sequence)?;
            let mut worker_order = plan.ordered_required_asset_ids.clone();
            worker_order.reverse();
            let staged = world_streamer.stage(&plan, &worker_order)?;
            if save_restore {
                let saved = world_streamer.snapshot().canonical_bytes()?;
                let decoded = next_contracts::WorldStreamingSnapshotV1::from_canonical_bytes(
                    &saved,
                    next_contracts::CanonicalDecodeLimits::default(),
                )?;
                world_streamer =
                    WorldStreamerV1::restore(fixture.activated_project.clone(), decoded)?;
                let (_, rebuilt) = world_streamer.resume_pending()?;
                if rebuilt != staged {
                    return Err(PlayCheckError::WorldStreamingResumeMismatch);
                }
            } else {
                world_streamer.validate_staged(&staged)?;
            }
            world_streamer.commit(&staged, false)?;
            if runtime.rpg_snapshot() != rpg_before {
                return Err(PlayCheckError::WorldStreamingMutatedRpg);
            }
            continue;
        }
        if matches!(&action, ScenarioAction::AgentMelee) {
            let previous = tick_reports
                .last()
                .ok_or(PlayCheckError::AgentActionMissing)?;
            let facts = rpg_contact_facts_from_report(
                &previous.contact_batch,
                runtime.physics_snapshot().checkpoint_revision,
            );
            let agent_rpg_snapshot = runtime.rpg_snapshot();
            let request = next_agent::AgentPlanningRequestV1 {
                gameplay_tick: previous.tick,
                world_generation: world_streamer.snapshot().generation,
                decision_seed: 0x4e45_5854,
                source_character_id: fixture.npc_character_id,
                target_character_id: fixture.body_id,
                allowed_semantic_actions: vec![
                    SchemaId::new(CORE_MELEE_ACTION_ID)
                        .expect("engine-owned melee action is valid"),
                ],
                motor_state: next_contracts::MotorCapabilityStateV1::ProceduralFallback,
                ai_host_available: false,
                model_available: false,
                rpg_snapshot: &agent_rpg_snapshot,
                definitions: &fixture.activated_project.rpg_definitions,
                physical_contact_facts: &facts,
            };
            let planned = next_agent::propose_world_command_v1(
                &request,
                next_agent::AgentCommandRouteV1 {
                    issuer: fixture.agent_principal.clone(),
                    stream_id: fixture.agent_stream_id,
                    sequence: 0,
                    target_tick: runtime.next_tick(),
                },
            )?;
            let report = runtime.run_tick([planned.world_command])?;
            if !report.results.iter().any(|result| {
                matches!(
                    result.disposition,
                    next_runtime::CommandDisposition::Committed
                )
            }) {
                return Err(PlayCheckError::AgentCommandRejected);
            }
            agent_intent_id = Some(planned.intent.intent_id);
            agent_projection_hash = Some(planned.procedural_projection.projection_hash);
            accumulate_report(
                &report,
                &mut events,
                &mut rpg_events,
                &mut begin_contacts,
                &mut persist_contacts,
                &mut end_contacts,
                &mut contact_preimage,
            )?;
            tick_reports.push(report);
            continue;
        }
        let wall_time =
            Some(i64::try_from(sequence).map_err(|_| PlayCheckError::CountOverflow)? * 1000);
        let sample = match action {
            ScenarioAction::Movement(phase, direction) => {
                player_action_sample(&fixture, sequence, phase, direction, wall_time)?
            }
            ScenarioAction::Interaction => player_interact_sample(
                &fixture,
                sequence,
                PlayerActionPhaseV1::Started,
                true,
                wall_time,
            )?,
            ScenarioAction::Pickup => player_pickup_sample(
                &fixture,
                sequence,
                PlayerActionPhaseV1::Started,
                true,
                wall_time,
            )?,
            ScenarioAction::EquipUse => player_equip_use_sample(
                &fixture,
                sequence,
                PlayerActionPhaseV1::Started,
                true,
                wall_time,
            )?,
            ScenarioAction::Melee => player_melee_sample(
                &fixture,
                sequence,
                PlayerActionPhaseV1::Started,
                true,
                wall_time,
            )?,
            ScenarioAction::AgentMelee => unreachable!("handled before input mapping"),
            ScenarioAction::ChunkTransition(_, _) => unreachable!("handled before input mapping"),
        };
        runtime.enqueue_input_sample(&fixture.principal, sample)?;
        let report = runtime.run_tick([])?;
        accumulate_report(
            &report,
            &mut events,
            &mut rpg_events,
            &mut begin_contacts,
            &mut persist_contacts,
            &mut end_contacts,
            &mut contact_preimage,
        )?;
        tick_reports.push(report);
        sequence = sequence
            .checked_add(1)
            .ok_or(PlayCheckError::CountOverflow)?;
    }
    let final_pose = runtime
        .physics_snapshot()
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or(PlayCheckError::BodyMissing)?
        .pose;
    let expected_ticks = if include_interaction { 16 } else { 6 };
    let expected_substeps = if include_interaction { 32 } else { 12 };
    let expected_events = if include_interaction { 17 } else { 4 };
    let expected_persists = if include_interaction { 53 } else { 15 };
    let expected_pose = if include_interaction {
        [200_000, 900_000, 200_000]
    } else {
        [0, 900_000, 200_000]
    };
    let expected_rpg_events = if include_interaction { 9 } else { 0 };
    let expected_begins = if include_interaction { 4 } else { 3 };
    let expected_ends = if include_interaction { 3 } else { 2 };
    let object_is_activated = !include_interaction
        || matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::InteractiveObject,
                fixture.interactive_object_id,
            ),
            Some(RpgAggregatePayloadV1::InteractiveObject(object))
                if object.state_id.as_str() == CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
        );
    let cooked_dialogue_completed = !include_interaction
        || matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::Dialogue,
                fixture.dialogue_id,
            ),
            Some(RpgAggregatePayloadV1::Dialogue(dialogue))
                if dialogue.node_id == expected_dialogue_node_id
        );
    let cooked_quest_completed = !include_interaction
        || matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::Quest,
                fixture.quest_id,
            ),
            Some(RpgAggregatePayloadV1::Quest(quest))
                if quest.state_id == expected_quest_state_id
        );
    let cooked_relationship_applied = !include_interaction
        || matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::Relationship,
                fixture.relationship_id,
            ),
            Some(RpgAggregatePayloadV1::Relationship(relationship))
                if relationship.source_id == fixture.npc_character_id
                    && relationship.target_id == fixture.body_id
                    && relationship.dimensions.iter().any(|dimension| {
                        dimension.dimension_id == relationship_dimension_id
                            && dimension.value == expected_relationship_value
                    })
        );
    let pickup_completed = !include_interaction
        || matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::InteractiveObject,
                fixture.pickup_proxy_id,
            ),
            Some(RpgAggregatePayloadV1::InteractiveObject(object))
                if object.state_id.as_str() == CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID
        );
    let item_is_owned = !include_interaction
        || matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::Inventory,
                fixture.player_inventory_id,
            ),
            Some(RpgAggregatePayloadV1::Inventory(inventory))
                if inventory.item_ids == [fixture.pickup_item_id]
        );
    let item_is_equipped = !include_interaction
        || matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::Equipment,
                fixture.player_equipment_id,
            ),
            Some(RpgAggregatePayloadV1::Equipment(equipment))
                if equipment.assignments.iter().any(|assignment| {
                    assignment.slot_id.as_str() == CORE_EQUIPMENT_MAIN_HAND_SLOT_ID
                        && assignment.item_id == fixture.pickup_item_id
                })
        );
    let npc_health_adjusted = !include_interaction
        || matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::Character,
                fixture.npc_character_id,
            ),
            Some(RpgAggregatePayloadV1::Character(character))
                if character.resources.iter().any(|resource| {
                    resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID
                        && resource.current_value == 75
                })
        );
    let player_health_adjusted = !include_interaction
        || matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::Character,
                fixture.body_id,
            ),
            Some(RpgAggregatePayloadV1::Character(character))
                if character.resources.iter().any(|resource| {
                    resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID
                        && resource.current_value == 75
                })
        );
    let world_streaming_snapshot = world_streamer.snapshot();
    let expected_current_chunk_id = &world_streaming_snapshot
        .chunks
        .first()
        .ok_or(PlayCheckError::WorldPartitionEmpty)?
        .chunk_id;
    if final_pose.translation_micrometres != expected_pose
        || runtime.physics_snapshot().physics_tick != expected_substeps
        || events != expected_events
        || rpg_events != expected_rpg_events
        || begin_contacts != expected_begins
        || persist_contacts != expected_persists
        || end_contacts != expected_ends
        || !object_is_activated
        || !cooked_dialogue_completed
        || !cooked_quest_completed
        || !cooked_relationship_applied
        || !pickup_completed
        || !item_is_owned
        || !item_is_equipped
        || !npc_health_adjusted
        || !player_health_adjusted
        || (include_interaction
            && (agent_intent_id.is_none()
                || agent_projection_hash.is_none()
                || world_streaming_snapshot.generation != 2
                || &world_streaming_snapshot.current_chunk_id != expected_current_chunk_id))
    {
        return Err(PlayCheckError::AcceptanceMismatch(format!(
            "pose={:?} physics_tick={} events={events} rpg_events={rpg_events} \
             contacts={begin_contacts}/{persist_contacts}/{end_contacts} \
             object={object_is_activated} dialogue={cooked_dialogue_completed} \
             quest={cooked_quest_completed} relationship={cooked_relationship_applied} \
             pickup={pickup_completed} owned={item_is_owned} equipped={item_is_equipped} \
             npc_health={npc_health_adjusted} player_health={player_health_adjusted} \
             agent={} world_generation={} current_chunk={}",
            final_pose.translation_micrometres,
            runtime.physics_snapshot().physics_tick,
            agent_intent_id.is_some() && agent_projection_hash.is_some(),
            world_streaming_snapshot.generation,
            world_streaming_snapshot.current_chunk_id.as_str(),
        )));
    }
    Ok(GroundedCollisionScenario {
        ticks: expected_ticks,
        final_pose,
        events,
        rpg_events,
        runtime,
        begin_contacts,
        persist_contacts,
        end_contacts,
        contact_batches_hash: ContentHash::from_bytes(next_contracts::sha256(&contact_preimage)),
        interactive_object_id: fixture.interactive_object_id,
        npc_character_id: fixture.npc_character_id,
        player_character_id: fixture.body_id,
        dialogue_id: fixture.dialogue_id,
        quest_id: fixture.quest_id,
        relationship_id: fixture.relationship_id,
        relationship_dimension_id,
        project_composition_lock_hash: fixture
            .activated_project
            .composition_lock
            .composition_lock_sha256,
        content_manifest_hash: fixture
            .activated_project
            .content_manifest
            .content_manifest_sha256,
        presentation_bindings: fixture_presentation_bindings(&fixture)?,
        tick_reports,
        world_streaming_snapshot: world_streaming_snapshot.clone(),
        agent_intent_id,
        agent_projection_hash,
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "the acceptance accumulator updates the complete deterministic report"
)]
fn accumulate_report(
    report: &next_runtime::TickReport,
    events: &mut u64,
    rpg_events: &mut u64,
    begin_contacts: &mut u64,
    persist_contacts: &mut u64,
    end_contacts: &mut u64,
    contact_preimage: &mut Vec<u8>,
) -> Result<(), PlayCheckError> {
    *events = events
        .checked_add(u64::try_from(report.events.len()).map_err(|_| PlayCheckError::CountOverflow)?)
        .ok_or(PlayCheckError::CountOverflow)?;
    *rpg_events = rpg_events
        .checked_add(
            u64::try_from(
                report
                    .events
                    .iter()
                    .filter(|event| matches!(&event.payload, EventPayload::Rpg(_)))
                    .count(),
            )
            .map_err(|_| PlayCheckError::CountOverflow)?,
        )
        .ok_or(PlayCheckError::CountOverflow)?;
    for contact in &report.contact_batch.events {
        let count = match contact.phase {
            ContactPhaseV1::Begin => &mut *begin_contacts,
            ContactPhaseV1::Persist => &mut *persist_contacts,
            ContactPhaseV1::End => &mut *end_contacts,
        };
        *count = count.checked_add(1).ok_or(PlayCheckError::CountOverflow)?;
    }
    contact_preimage.extend_from_slice(report.contact_batch.batch_hash.as_bytes());
    Ok(())
}

fn rpg_contact_facts_from_report(
    batch: &next_contracts::ClosedPhysicsContactBatchV1,
    physics_checkpoint_revision: u64,
) -> Vec<RpgPhysicalContactFactV1> {
    let mut facts = batch
        .events
        .iter()
        .filter(|event| matches!(event.phase, ContactPhaseV1::Begin | ContactPhaseV1::Persist))
        .filter_map(|event| {
            let first = event.participant_low.body_id.subject_id;
            let second = event.participant_high.body_id.subject_id;
            if first == second {
                return None;
            }
            let (subject_low, subject_high) = if first < second {
                (first, second)
            } else {
                (second, first)
            };
            Some(RpgPhysicalContactFactV1 {
                gameplay_tick: batch.gameplay_tick,
                contact_id: event.contact_id,
                subject_low,
                subject_high,
                physics_checkpoint_revision,
                source_snapshot_hash: event.source_snapshot_hash,
                contact_batch_hash: batch.batch_hash,
            })
        })
        .collect::<Vec<_>>();
    facts.sort_unstable();
    facts.dedup();
    facts
}

fn fixture_presentation_bindings(
    fixture: &NeutralPlayerFixture,
) -> Result<Vec<PresentationBindingV1>, PlayCheckError> {
    let asset = |kind: next_contracts::NeutralRecordKindV1| {
        fixture
            .activated_project
            .neutral_records
            .iter()
            .find(|record| record.kind == kind)
            .map(|record| record.asset_id)
            .ok_or(PlayCheckError::PresentationAssetMissing)
    };
    Ok(vec![
        PresentationBindingV1 {
            persistent_id: PersistentId::from_bytes([0x57; 16]),
            presentation_role: next_contracts::PresentationRoleV1::Environment,
            incarnation: 0,
            presentation_layer: 0,
            asset_id: asset(next_contracts::NeutralRecordKindV1::Scene)?,
            instance_ordinal: 0,
            primitive: next_contracts::PresentationPrimitiveV1::Floor,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([0x57; 16]),
                body_slot: 0,
            }),
            fallback_transform: next_contracts::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.body_id,
            presentation_role: next_contracts::PresentationRoleV1::PlayerAvatar,
            incarnation: 0,
            presentation_layer: 1,
            asset_id: asset(next_contracts::NeutralRecordKindV1::Collider)?,
            instance_ordinal: 0,
            primitive: next_contracts::PresentationPrimitiveV1::Capsule,
            physics_body_id: Some(fixture.physics_body_id),
            fallback_transform: next_contracts::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.interactive_object_id,
            presentation_role: next_contracts::PresentationRoleV1::InteractiveObject,
            incarnation: 0,
            presentation_layer: 2,
            asset_id: asset(next_contracts::NeutralRecordKindV1::InteractionDefinition)?,
            instance_ordinal: 0,
            primitive: next_contracts::PresentationPrimitiveV1::Switch,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.interactive_object_id,
                body_slot: 0,
            }),
            fallback_transform: next_contracts::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.pickup_item_id,
            presentation_role: next_contracts::PresentationRoleV1::Item,
            incarnation: 0,
            presentation_layer: 3,
            asset_id: asset(next_contracts::NeutralRecordKindV1::ItemDefinition)?,
            instance_ordinal: 0,
            primitive: next_contracts::PresentationPrimitiveV1::Item,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.pickup_proxy_id,
                body_slot: 0,
            }),
            fallback_transform: next_contracts::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.npc_character_id,
            presentation_role: next_contracts::PresentationRoleV1::Character,
            incarnation: 0,
            presentation_layer: 4,
            asset_id: asset(next_contracts::NeutralRecordKindV1::CharacterDefinition)?,
            instance_ordinal: 0,
            primitive: next_contracts::PresentationPrimitiveV1::Character,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.npc_character_id,
                body_slot: 0,
            }),
            fallback_transform: next_contracts::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
    ])
}

#[derive(Debug)]
pub enum CanonicalFixtureError {
    Identifier(next_contracts::IdentifierError),
    Canonical(next_contracts::CanonicalError),
}

impl Display for CanonicalFixtureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for CanonicalFixtureError {}

impl From<next_contracts::IdentifierError> for CanonicalFixtureError {
    fn from(error: next_contracts::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<next_contracts::CanonicalError> for CanonicalFixtureError {
    fn from(error: next_contracts::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

#[derive(Debug)]
pub enum PlayCheckError {
    Fixture(NeutralFixtureError),
    CanonicalFixture(CanonicalFixtureError),
    Input(next_runtime::InputAdmissionError),
    Runtime(RuntimeFatalError),
    Restore(SnapshotRestoreError),
    Checkpoint(next_contracts::WorldCheckpointError),
    Replay(crate::ReplayError),
    PersistenceReplay(crate::PersistenceReplayCheckError),
    Ledger(next_contracts::CommandLedgerError),
    Canonical(next_contracts::CanonicalError),
    CountOverflow,
    BodyMissing,
    InteractiveObjectMissing,
    CookedNpcMissing,
    CookedPlayerMissing,
    CookedDialogueMissing,
    CookedQuestMissing,
    AcceptanceMismatch(String),
    BackendParityMismatch,
    PresentationAssetMissing,
    WorldPartitionEmpty,
    WorldStreaming(WorldStreamingError),
    WorldStreamingContract(next_contracts::WorldStreamingContractError),
    WorldStreamingResumeMismatch,
    WorldStreamingMutatedRpg,
    Agent(next_agent::AgentPlannerError),
    AgentActionMissing,
    AgentCommandRejected,
    PresentationExtraction(next_presentation::PresentationExtractionError),
    Render(next_render::RenderDeviceError),
}

impl Display for PlayCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixture(error) => write!(formatter, "{error}"),
            Self::CanonicalFixture(error) => write!(formatter, "{error}"),
            Self::Input(error) => write!(formatter, "{error}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::Restore(error) => write!(formatter, "{error}"),
            Self::Checkpoint(error) => write!(formatter, "{error}"),
            Self::Replay(error) => write!(formatter, "{error}"),
            Self::PersistenceReplay(error) => write!(formatter, "{error}"),
            Self::Ledger(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
            Self::CountOverflow => formatter.write_str("play check count overflow"),
            Self::BodyMissing => formatter.write_str("play check capsule body is missing"),
            Self::InteractiveObjectMissing => {
                formatter.write_str("play check interactive object is missing")
            }
            Self::CookedNpcMissing => formatter.write_str("play check cooked NPC is missing"),
            Self::CookedPlayerMissing => formatter.write_str("play check player is missing"),
            Self::CookedDialogueMissing => {
                formatter.write_str("play check cooked dialogue is missing")
            }
            Self::CookedQuestMissing => formatter.write_str("play check cooked quest is missing"),
            Self::AcceptanceMismatch(details) => {
                write!(formatter, "play check result did not match: {details}")
            }
            Self::BackendParityMismatch => {
                formatter.write_str("reference and PhysX tick reports diverged")
            }
            Self::PresentationAssetMissing => {
                formatter.write_str("cooked presentation asset is missing")
            }
            Self::WorldPartitionEmpty => {
                formatter.write_str("cooked world partition requires two chunks")
            }
            Self::WorldStreaming(error) => write!(formatter, "{error}"),
            Self::WorldStreamingContract(error) => write!(formatter, "{error}"),
            Self::WorldStreamingResumeMismatch => {
                formatter.write_str("restaged world group changed after save/restore")
            }
            Self::WorldStreamingMutatedRpg => {
                formatter.write_str("world transition mutated durable RPG state")
            }
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::AgentActionMissing => formatter.write_str("deterministic NPC action is missing"),
            Self::AgentCommandRejected => {
                formatter.write_str("deterministic NPC command was rejected")
            }
            Self::PresentationExtraction(error) => write!(formatter, "{error}"),
            Self::Render(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for PlayCheckError {}

impl From<NeutralFixtureError> for PlayCheckError {
    fn from(error: NeutralFixtureError) -> Self {
        Self::Fixture(error)
    }
}

impl From<CanonicalFixtureError> for PlayCheckError {
    fn from(error: CanonicalFixtureError) -> Self {
        Self::CanonicalFixture(error)
    }
}

impl From<next_runtime::InputAdmissionError> for PlayCheckError {
    fn from(error: next_runtime::InputAdmissionError) -> Self {
        Self::Input(error)
    }
}

impl From<RuntimeFatalError> for PlayCheckError {
    fn from(error: RuntimeFatalError) -> Self {
        Self::Runtime(error)
    }
}

impl From<SnapshotRestoreError> for PlayCheckError {
    fn from(error: SnapshotRestoreError) -> Self {
        Self::Restore(error)
    }
}

impl From<next_contracts::WorldCheckpointError> for PlayCheckError {
    fn from(error: next_contracts::WorldCheckpointError) -> Self {
        Self::Checkpoint(error)
    }
}

impl From<crate::ReplayError> for PlayCheckError {
    fn from(error: crate::ReplayError) -> Self {
        Self::Replay(error)
    }
}

impl From<crate::PersistenceReplayCheckError> for PlayCheckError {
    fn from(error: crate::PersistenceReplayCheckError) -> Self {
        Self::PersistenceReplay(error)
    }
}

impl From<next_contracts::CommandLedgerError> for PlayCheckError {
    fn from(error: next_contracts::CommandLedgerError) -> Self {
        Self::Ledger(error)
    }
}

impl From<next_contracts::CanonicalError> for PlayCheckError {
    fn from(error: next_contracts::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<next_presentation::PresentationExtractionError> for PlayCheckError {
    fn from(error: next_presentation::PresentationExtractionError) -> Self {
        Self::PresentationExtraction(error)
    }
}

impl From<next_render::RenderDeviceError> for PlayCheckError {
    fn from(error: next_render::RenderDeviceError) -> Self {
        Self::Render(error)
    }
}

impl From<WorldStreamingError> for PlayCheckError {
    fn from(error: WorldStreamingError) -> Self {
        Self::WorldStreaming(error)
    }
}

impl From<next_contracts::WorldStreamingContractError> for PlayCheckError {
    fn from(error: next_contracts::WorldStreamingContractError) -> Self {
        Self::WorldStreamingContract(error)
    }
}

impl From<next_agent::AgentPlannerError> for PlayCheckError {
    fn from(error: next_agent::AgentPlannerError) -> Self {
        Self::Agent(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::{INPUT_SAMPLE_SCHEMA_VERSION, InputMappingCodeV1, RpgEventV1};

    fn interactive_snapshot(fixture: &NeutralPlayerFixture, state: &str) -> RpgSnapshotV2 {
        RpgSnapshotV2 {
            aggregates: vec![fixture_aggregate(
                fixture.interactive_object_id,
                0x58,
                RpgAggregatePayloadV1::InteractiveObject(InteractiveObjectPayloadV1 {
                    state_id: SchemaId::new(state).expect("interactive state"),
                    linked_item_id: None,
                }),
            )],
        }
    }

    fn sample_with_actions(
        fixture: &NeutralPlayerFixture,
        sequence: u64,
        actions: Vec<PlayerActionV1>,
    ) -> InputSampleV1 {
        let frame = PlayerActionFrameV1 {
            schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
            controller_id: fixture.controller_id,
            logical_frame_sequence: sequence,
            action_map_hash: fixture.action_map_hash,
            action_map_revision: 1,
            context_stack_hash: fixture.context_stack_hash,
            context_stack_revision: 1,
            actions,
        };
        InputSampleV1 {
            schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
            source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
            source_id: fixture.source_id,
            source_sequence: sequence,
            payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("frame schema"),
            payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
            payload: frame.canonical_bytes().expect("canonical frame"),
            sampled_wall_time: None,
        }
    }

    fn ledger_hash(runtime: &RuntimeState) -> CommandLedgerHash {
        runtime
            .world_checkpoint()
            .expect("checkpoint")
            .runtime_snapshot
            .command_ledger_hash()
            .expect("ledger hash")
    }

    #[test]
    #[cfg(any(feature = "physx", feature = "physx-mock"))]
    fn prefer_physx_selects_physx_when_activation_succeeds() {
        let fixture = build_physx_player_fixture("nextengine.test.prefer-physx").expect("fixture");
        let runtime = RuntimeState::with_rpg_snapshot_and_physics_options(
            fixture.bootstrap,
            fixture.authority,
            RpgSnapshotV2::default(),
            PhysicsLaunchOptions::new(PhysicsBackendPolicy::PreferPhysXThenReference),
        )
        .expect("PhysX activation");
        assert_eq!(
            runtime.physics_backend_kind(),
            next_physics_api::PhysicsBackendKind::PhysX
        );
    }

    #[test]
    fn interaction_without_eligible_contact_is_accepted_without_ledger_or_rpg_mutation() {
        let fixture = build_neutral_player_fixture("nextengine.test.interaction-no-contact")
            .expect("fixture");
        let initial_rpg = interactive_snapshot(&fixture, CORE_INTERACTIVE_OBJECT_READY_STATE_ID);
        let mut runtime = RuntimeState::with_rpg_snapshot(
            fixture.bootstrap.clone(),
            fixture.authority.clone(),
            initial_rpg.clone(),
        )
        .expect("runtime");
        let ledger_before = ledger_hash(&runtime);
        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_interact_sample(&fixture, 0, PlayerActionPhaseV1::Started, true, None)
                    .expect("interaction sample"),
            )
            .expect("enqueue");
        let report = runtime.run_tick([]).expect("tick");
        assert_eq!(report.mapping_receipts.len(), 1);
        assert_eq!(
            report.mapping_receipts[0].code,
            InputMappingCodeV1::Accepted
        );
        assert_eq!(report.mapping_receipts[0].derived_command_id, None);
        assert!(report.results.is_empty());
        assert_eq!(runtime.rpg_snapshot(), initial_rpg);
        assert_eq!(ledger_hash(&runtime), ledger_before);
    }

    #[test]
    fn invalid_mixed_and_colliding_interaction_input_never_mutates_rpg() {
        let fixture =
            build_neutral_player_fixture("nextengine.test.interaction-invalid").expect("fixture");
        let initial_rpg = interactive_snapshot(&fixture, CORE_INTERACTIVE_OBJECT_READY_STATE_ID);
        let mut runtime = RuntimeState::with_rpg_snapshot(
            fixture.bootstrap.clone(),
            fixture.authority.clone(),
            initial_rpg.clone(),
        )
        .expect("runtime");

        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_interact_sample(&fixture, 0, PlayerActionPhaseV1::Performed, true, None)
                    .expect("invalid interaction"),
            )
            .expect("enqueue invalid interaction");
        let invalid = runtime.run_tick([]).expect("invalid tick is nonfatal");
        assert_eq!(
            invalid.mapping_receipts[0].code,
            InputMappingCodeV1::ValueOutOfProfile
        );

        let mixed = sample_with_actions(
            &fixture,
            1,
            vec![
                PlayerActionV1 {
                    action_id: SchemaId::new(CORE_INTERACT_ACTION_ID).expect("interact ID"),
                    phase: PlayerActionPhaseV1::Started,
                    value: PlayerActionValueV1::Digital(true),
                    semantic_occurrence_ordinal: 0,
                },
                PlayerActionV1 {
                    action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("move ID"),
                    phase: PlayerActionPhaseV1::Performed,
                    value: PlayerActionValueV1::Vector2Q15([0, 32_767]),
                    semantic_occurrence_ordinal: 1,
                },
            ],
        );
        runtime
            .enqueue_input_sample(&fixture.principal, mixed)
            .expect("enqueue mixed frame");
        let mixed_report = runtime.run_tick([]).expect("mixed tick is nonfatal");
        assert_eq!(
            mixed_report.mapping_receipts[0].code,
            InputMappingCodeV1::ActionUnmapped
        );

        let interact =
            player_interact_sample(&fixture, 2, PlayerActionPhaseV1::Started, true, None)
                .expect("interaction sample");
        let movement =
            player_action_sample(&fixture, 2, PlayerActionPhaseV1::Started, [0, 32_767], None)
                .expect("movement sample");
        runtime
            .enqueue_input_sample(&fixture.principal, interact)
            .expect("enqueue interaction collision candidate");
        runtime
            .enqueue_input_sample(&fixture.principal, movement)
            .expect("enqueue movement collision candidate");
        let collision = runtime.run_tick([]).expect("collision tick is nonfatal");
        assert!(collision.mapping_receipts.is_empty());
        assert_eq!(
            collision
                .closed_ingress_batch
                .body
                .equivalence_receipts
                .len(),
            1
        );
        assert_eq!(runtime.rpg_snapshot(), initial_rpg);
    }

    #[test]
    fn committed_interaction_retry_after_restore_is_an_accepted_noop() {
        let fixture =
            build_neutral_player_fixture("nextengine.test.interaction-retry").expect("fixture");
        let mut runtime = RuntimeState::with_rpg_snapshot(
            fixture.bootstrap.clone(),
            fixture.authority.clone(),
            interactive_snapshot(&fixture, CORE_INTERACTIVE_OBJECT_READY_STATE_ID),
        )
        .expect("runtime");
        for sequence in 0_u64..4 {
            runtime
                .enqueue_input_sample(
                    &fixture.principal,
                    player_action_sample(
                        &fixture,
                        sequence,
                        if sequence == 0 {
                            PlayerActionPhaseV1::Started
                        } else {
                            PlayerActionPhaseV1::Performed
                        },
                        [0, 32_767],
                        None,
                    )
                    .expect("movement sample"),
                )
                .expect("enqueue movement");
            let _ = runtime.run_tick([]).expect("movement tick");
        }
        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_interact_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, None)
                    .expect("interaction sample"),
            )
            .expect("enqueue interaction");
        let committed = runtime.run_tick([]).expect("interaction tick");
        assert_eq!(
            committed
                .events
                .iter()
                .filter(|event| matches!(
                    event.payload,
                    EventPayload::Rpg(RpgEventV1::InteractiveObjectTransitioned { .. })
                ))
                .count(),
            1
        );

        let checkpoint = runtime.world_checkpoint().expect("checkpoint");
        let mut restored = RuntimeState::restore_world_checkpoint_with_definitions(
            checkpoint,
            fixture.authority.clone(),
            fixture.activated_project.rpg_definitions.clone(),
        )
        .expect("restore");
        let ledger_before = ledger_hash(&restored);
        let rpg_before = restored.rpg_snapshot();
        restored
            .enqueue_input_sample(
                &fixture.principal,
                player_interact_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, Some(42))
                    .expect("retry sample"),
            )
            .expect("enqueue retry");
        let retry = restored.run_tick([]).expect("retry tick");
        assert_eq!(retry.mapping_receipts[0].code, InputMappingCodeV1::Accepted);
        assert_eq!(retry.mapping_receipts[0].derived_command_id, None);
        assert!(retry.results.is_empty());
        assert!(retry.events.is_empty());
        assert_eq!(restored.rpg_snapshot(), rpg_before);
        assert_eq!(ledger_hash(&restored), ledger_before);
    }

    #[test]
    fn pickup_and_equip_retry_after_restore_do_not_duplicate_state_or_events() {
        let fixture =
            build_neutral_player_fixture("nextengine.test.pickup-equip-retry").expect("fixture");
        let mut runtime = RuntimeState::with_rpg_snapshot(
            fixture.bootstrap.clone(),
            fixture.authority.clone(),
            cooked_project_rpg_snapshot(&fixture),
        )
        .expect("runtime");
        for sequence in 0_u64..4 {
            runtime
                .enqueue_input_sample(
                    &fixture.principal,
                    player_action_sample(
                        &fixture,
                        sequence,
                        if sequence == 0 {
                            PlayerActionPhaseV1::Started
                        } else {
                            PlayerActionPhaseV1::Performed
                        },
                        [0, 32_767],
                        None,
                    )
                    .expect("movement sample"),
                )
                .expect("enqueue movement");
            runtime.run_tick([]).expect("movement tick");
        }
        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_pickup_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, None)
                    .expect("pickup sample"),
            )
            .expect("enqueue pickup");
        let pickup = runtime.run_tick([]).expect("pickup tick");
        assert_eq!(
            pickup
                .events
                .iter()
                .filter(|event| matches!(event.payload, EventPayload::Rpg(_)))
                .count(),
            2
        );
        assert_eq!(pickup.rpg_plan_traces.len(), 1);

        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_equip_use_sample(&fixture, 5, PlayerActionPhaseV1::Started, true, None)
                    .expect("equip sample"),
            )
            .expect("enqueue equip");
        let equip = runtime.run_tick([]).expect("equip tick");
        assert_eq!(
            equip
                .events
                .iter()
                .filter(|event| matches!(
                    event.payload,
                    EventPayload::Rpg(RpgEventV1::EquipmentAssigned { .. })
                ))
                .count(),
            1
        );

        let checkpoint = runtime.world_checkpoint().expect("checkpoint");
        let mut restored = RuntimeState::restore_world_checkpoint_with_definitions(
            checkpoint,
            fixture.authority.clone(),
            fixture.activated_project.rpg_definitions.clone(),
        )
        .expect("restore");
        let state_before = restored.rpg_snapshot();
        let ledger_before = ledger_hash(&restored);

        for (sequence, sample) in [
            (
                4,
                player_pickup_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, Some(42))
                    .expect("pickup retry"),
            ),
            (
                5,
                player_equip_use_sample(&fixture, 5, PlayerActionPhaseV1::Started, true, Some(43))
                    .expect("equip retry"),
            ),
        ] {
            restored
                .enqueue_input_sample(&fixture.principal, sample)
                .expect("enqueue retry");
            let retry = restored.run_tick([]).expect("retry tick");
            assert_eq!(retry.mapping_receipts[0].source_sequence, sequence);
            assert_eq!(retry.mapping_receipts[0].code, InputMappingCodeV1::Accepted);
            assert_eq!(retry.mapping_receipts[0].derived_command_id, None);
            assert!(retry.events.is_empty());
            assert!(retry.results.is_empty());
        }
        assert_eq!(restored.rpg_snapshot(), state_before);
        assert_eq!(ledger_hash(&restored), ledger_before);
    }

    #[test]
    fn cooked_dialogue_requires_contact_and_invalid_participant_closure_fails_activation() {
        let fixture =
            build_neutral_player_fixture("nextengine.test.dialogue-closure").expect("fixture");
        let ready = cooked_project_rpg_snapshot(&fixture);
        let mut runtime = RuntimeState::with_rpg_snapshot(
            fixture.bootstrap.clone(),
            fixture.authority.clone(),
            ready.clone(),
        )
        .expect("ready core dialogue runtime");
        let ledger_before = ledger_hash(&runtime);
        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_interact_sample(&fixture, 0, PlayerActionPhaseV1::Started, true, None)
                    .expect("interaction sample"),
            )
            .expect("enqueue interaction");
        let report = runtime.run_tick([]).expect("contact-free interaction tick");
        assert_eq!(
            report.mapping_receipts[0].code,
            InputMappingCodeV1::Accepted
        );
        assert_eq!(report.mapping_receipts[0].derived_command_id, None);
        assert!(report.results.is_empty());
        assert_eq!(runtime.rpg_snapshot(), ready);
        assert_eq!(ledger_hash(&runtime), ledger_before);

        let mut invalid = cooked_project_rpg_snapshot(&fixture);
        let dialogue = invalid
            .aggregates
            .iter_mut()
            .find(|aggregate| {
                aggregate.aggregate_kind == RpgAggregateKindV1::Dialogue
                    && aggregate.persistent_id == fixture.dialogue_id
            })
            .expect("fixture dialogue");
        let RpgAggregatePayloadV1::Dialogue(mut payload) = dialogue.payload.clone() else {
            panic!("fixture dialogue payload");
        };
        payload.listener_id = fixture.npc_character_id;
        *dialogue = RpgAggregateEnvelopeV1::new(
            dialogue.persistent_id,
            dialogue.schema_version,
            dialogue.revision,
            dialogue.definition_ref.clone(),
            dialogue.provenance.clone(),
            RpgAggregatePayloadV1::Dialogue(payload),
        )
        .expect("mutated dialogue aggregate is canonical");
        assert!(matches!(
            RuntimeState::with_rpg_snapshot(fixture.bootstrap, fixture.authority, invalid),
            Err(SnapshotRestoreError::CoreInteractionClosure(_))
        ));
    }

    #[test]
    fn same_contact_switch_precedes_npc_then_dialogue_transition_is_one_shot() {
        let fixture =
            build_neutral_player_fixture("nextengine.test.dialogue-tie-break").expect("fixture");
        let entry_node = fixture
            .activated_project
            .rpg_definitions
            .dialogues
            .first()
            .expect("dialogue definition")
            .entry_node_id
            .clone();
        let mut rpg = cooked_project_rpg_snapshot(&fixture);
        rpg.aggregates
            .iter_mut()
            .find(|aggregate| {
                aggregate.aggregate_kind == RpgAggregateKindV1::InteractiveObject
                    && aggregate.persistent_id == fixture.interactive_object_id
            })
            .expect("fixture interactive object")
            .persistent_id = fixture.npc_character_id;
        let mut runtime = RuntimeState::with_rpg_snapshot(
            fixture.bootstrap.clone(),
            fixture.authority.clone(),
            rpg,
        )
        .expect("runtime");
        let movement = [
            (PlayerActionPhaseV1::Started, [0, 32_767]),
            (PlayerActionPhaseV1::Performed, [0, 32_767]),
            (PlayerActionPhaseV1::Performed, [0, 32_767]),
            (PlayerActionPhaseV1::Performed, [0, 32_767]),
            (PlayerActionPhaseV1::Performed, [0, -32_767]),
            (PlayerActionPhaseV1::Started, [32_767, 0]),
            (PlayerActionPhaseV1::Performed, [32_767, 0]),
            (PlayerActionPhaseV1::Performed, [32_767, 0]),
        ];
        for (sequence, (phase, direction)) in movement.into_iter().enumerate() {
            let sequence = u64::try_from(sequence).expect("bounded test sequence");
            runtime
                .enqueue_input_sample(
                    &fixture.principal,
                    player_action_sample(&fixture, sequence, phase, direction, None)
                        .expect("movement sample"),
                )
                .expect("enqueue movement");
            let _ = runtime.run_tick([]).expect("movement tick");
        }

        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_interact_sample(&fixture, 8, PlayerActionPhaseV1::Started, true, None)
                    .expect("interaction sample"),
            )
            .expect("enqueue first interaction");
        let switch = runtime.run_tick([]).expect("switch wins tie");
        assert!(switch.events.iter().any(|event| matches!(
            event.payload,
            EventPayload::Rpg(RpgEventV1::InteractiveObjectTransitioned { .. })
        )));
        assert!(matches!(
            aggregate_payload(
                &runtime.rpg_snapshot(),
                RpgAggregateKindV1::Dialogue,
                fixture.dialogue_id,
            ),
            Some(RpgAggregatePayloadV1::Dialogue(dialogue))
                if dialogue.node_id == entry_node
        ));

        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_interact_sample(&fixture, 9, PlayerActionPhaseV1::Started, true, None)
                    .expect("interaction sample"),
            )
            .expect("enqueue dialogue interaction");
        let dialogue = runtime.run_tick([]).expect("dialogue transition");
        assert_eq!(
            dialogue
                .events
                .iter()
                .filter(|event| matches!(
                    event.payload,
                    EventPayload::Rpg(
                        RpgEventV1::DialogueAdvanced { .. }
                            | RpgEventV1::QuestTransitioned { .. }
                            | RpgEventV1::RelationshipAdjusted { .. }
                    )
                ))
                .count(),
            3
        );

        let checkpoint = runtime.world_checkpoint().expect("checkpoint");
        let mut restored = RuntimeState::restore_world_checkpoint_with_definitions(
            checkpoint,
            fixture.authority.clone(),
            fixture.activated_project.rpg_definitions.clone(),
        )
        .expect("restore");
        let rpg_before = restored.rpg_snapshot();
        let ledger_before = ledger_hash(&restored);
        restored
            .enqueue_input_sample(
                &fixture.principal,
                player_interact_sample(&fixture, 9, PlayerActionPhaseV1::Started, true, Some(99))
                    .expect("retry interaction"),
            )
            .expect("enqueue retry");
        let retry = restored.run_tick([]).expect("completed interaction retry");
        assert_eq!(retry.mapping_receipts[0].code, InputMappingCodeV1::Accepted);
        assert_eq!(retry.mapping_receipts[0].derived_command_id, None);
        assert!(retry.results.is_empty());
        assert!(retry.events.is_empty());
        assert_eq!(restored.rpg_snapshot(), rpg_before);
        assert_eq!(ledger_hash(&restored), ledger_before);
    }
}
