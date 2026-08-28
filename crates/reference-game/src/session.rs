use std::collections::BTreeMap;

use next_contracts::cognition::{AGENT_COGNITION_CAPABILITY_ID, AGENT_COGNITION_SYSTEM_ID};
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
use next_contracts::project::ActivatedProjectV8;
use next_contracts::rpg::RPG_COMMAND_CAPABILITY_ID;

use crate::{
    ReferenceBodyProjectionSetV1, ReferenceGameError, ReferenceWorldTopologyV1,
    build_reference_runtime_bootstrap,
};

const WORLD_COLLISION_LAYER: u8 = 0;
const WORLD_COLLISION_MASK: u64 = 1 << WORLD_COLLISION_LAYER;
const CARRIED_LOAD_COLLISION_LAYER: u8 = 1;
const CARRIED_LOAD_COLLISION_MASK: u64 = 1 << CARRIED_LOAD_COLLISION_LAYER;
const R5B_COURSE_Z_MICROMETRES: i64 = -6_000_000;
const CARRIED_LOAD_LOCAL_TRANSLATION_MICROMETRES: [i64; 3] = [-700_000, 400_000, 500_000];
const CARRIED_LOAD_HALF_EXTENTS_MICROMETRES: [i64; 3] = [200_000, 300_000, 200_000];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceCapsuleCourseV1 {
    pub static_body_id: PhysicsBodyIdV1,
    pub trip_shape_id: PhysicsShapeIdV1,
    pub carry_blocker_shape_id: PhysicsShapeIdV1,
    pub dynamic_body_id: PhysicsBodyIdV1,
    pub dynamic_shape_id: PhysicsShapeIdV1,
    pub sensor_body_id: PhysicsBodyIdV1,
    pub sensor_shape_id: PhysicsShapeIdV1,
    pub centre_z_micrometres: i64,
}

impl ReferenceCapsuleCourseV1 {
    fn production_v1() -> Self {
        let static_body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([0x78; 16]),
            body_slot: 0,
        };
        let dynamic_body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([0x79; 16]),
            body_slot: 0,
        };
        let sensor_body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([0x7a; 16]),
            body_slot: 0,
        };
        Self {
            static_body_id,
            trip_shape_id: PhysicsShapeIdV1 {
                body_id: static_body_id,
                shape_slot: 0,
            },
            carry_blocker_shape_id: PhysicsShapeIdV1 {
                body_id: static_body_id,
                shape_slot: 7,
            },
            dynamic_body_id,
            dynamic_shape_id: PhysicsShapeIdV1 {
                body_id: dynamic_body_id,
                shape_slot: 0,
            },
            sensor_body_id,
            sensor_shape_id: PhysicsShapeIdV1 {
                body_id: sensor_body_id,
                shape_slot: 0,
            },
            centre_z_micrometres: R5B_COURSE_Z_MICROMETRES,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReferenceGameSession {
    pub bootstrap: next_runtime::RuntimeBootstrapV4,
    pub authority: next_runtime::AuthorityRegistry,
    pub principal: IssuerPrincipal,
    pub movement_stream_id: CommandStreamId,
    pub rpg_stream_id: CommandStreamId,
    pub interaction_stream_id: CommandStreamId,
    pub source_id: InputSourceId,
    pub controller_id: PersistentId,
    pub body_id: PersistentId,
    pub physics_body_id: PhysicsBodyIdV1,
    pub carried_load_id: PersistentId,
    pub carried_load_shape_id: PhysicsShapeIdV1,
    pub carried_load_local_translation_micrometres: [i64; 3],
    pub r5b_course: ReferenceCapsuleCourseV1,
    pub interactive_object_id: PersistentId,
    pub npc_character_id: PersistentId,
    pub player_body_condition_id: PersistentId,
    pub npc_body_condition_id: PersistentId,
    pub body_projections: ReferenceBodyProjectionSetV1,
    pub procedural_motor: next_motor::CapsuleProceduralMotorControllerV1,
    pub quest_giver_character_id: PersistentId,
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
    pub cognition_principal: IssuerPrincipal,
    pub cognition_stream_id: CommandStreamId,
    pub cognition_rpg_stream_id: CommandStreamId,
    pub cognition_subject_id: PersistentId,
    pub activity_principal: IssuerPrincipal,
    pub activity_stream_id: CommandStreamId,
    pub action_map: ActionMapManifestV1,
    pub action_map_hash: ContentHash,
    pub context_stack: InputContextStackV1,
    pub context_stack_hash: ContentHash,
    pub activated_project: ActivatedProjectV8,
    world_topology: ReferenceWorldTopologyV1,
}

impl ReferenceGameSession {
    #[must_use]
    pub fn world_topology(&self) -> &ReferenceWorldTopologyV1 {
        &self.world_topology
    }

    pub fn initial_cognition_owners(
        &self,
    ) -> Result<next_agent::cognition::StrategicAgentOwnersV1, ReferenceGameError> {
        let mut seed_preimage = b"nextengine.reference-agent-rng.v1\0".to_vec();
        seed_preimage.extend_from_slice(&self.bootstrap.world_identity.rng_root_seed);
        seed_preimage.extend_from_slice(self.cognition_subject_id.as_bytes());
        let digest = next_contracts::canonical::sha256(&seed_preimage);
        let decision_rng_state = u64::from_le_bytes(
            digest[..8]
                .try_into()
                .expect("sha256 always has at least eight bytes"),
        );
        Ok(next_agent::cognition::StrategicAgentOwnersV1::initial(
            self.activated_project.agent_cognition_catalog.clone(),
            decision_rng_state,
        )?)
    }

    pub fn initial_activity_owner(
        &self,
    ) -> Result<next_world::WorldActivityOwnerV1, ReferenceGameError> {
        Ok(next_world::WorldActivityOwnerV1::activate(
            self.activated_project.world_activity_catalog.clone(),
            0,
        )?)
    }
}

pub fn build_reference_game_session(
    activated_project: ActivatedProjectV8,
) -> Result<ReferenceGameSession, ReferenceGameError> {
    build_reference_game_session_with_profile(activated_project, false)
}

pub fn build_reference_game_session_with_profile(
    activated_project: ActivatedProjectV8,
    physx_compatible: bool,
) -> Result<ReferenceGameSession, ReferenceGameError> {
    let world_topology = ReferenceWorldTopologyV1::from_activated_project(&activated_project)?;
    let project_id = activated_project
        .project_lock
        .project_id
        .as_str()
        .to_owned();
    let principal = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([0x51; 16]));
    let interaction_principal =
        IssuerPrincipal::InternalSystem(SystemId::new(PLAYER_INTERACTION_SYSTEM_ID)?);
    let agent_principal =
        IssuerPrincipal::InternalSystem(SystemId::new("nextengine.agent.planner")?);
    let cognition_principal =
        IssuerPrincipal::InternalSystem(SystemId::new(AGENT_COGNITION_SYSTEM_ID)?);
    let activity_principal = IssuerPrincipal::InternalSystem(SystemId::new(
        next_contracts::world_activity::WORLD_ACTIVITY_SYSTEM_ID,
    )?);
    let routine_principal = IssuerPrincipal::InternalSystem(SystemId::new(
        next_contracts::world_routine::WORLD_ROUTINE_SYSTEM_ID,
    )?);
    let population_principal = IssuerPrincipal::InternalSystem(SystemId::new(
        next_contracts::world_population::WORLD_POPULATION_SYSTEM_ID,
    )?);
    let mut grants = vec![
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
        (cognition_principal.clone(), {
            let mut capabilities = vec![
                CapabilityId::new(AGENT_COGNITION_CAPABILITY_ID)?,
                CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)?,
            ];
            capabilities.sort();
            capabilities
        }),
        (
            activity_principal.clone(),
            vec![CapabilityId::new(
                next_contracts::world_activity::WORLD_ACTIVITY_CAPABILITY_ID,
            )?],
        ),
    ];
    if activated_project.world_routine_catalog_or_none.is_some() {
        grants.push((
            routine_principal,
            vec![CapabilityId::new(
                next_contracts::world_routine::WORLD_ROUTINE_CAPABILITY_ID,
            )?],
        ));
    }
    grants.push((
        population_principal,
        vec![CapabilityId::new(
            next_contracts::world_population::WORLD_POPULATION_CAPABILITY_ID,
        )?],
    ));
    let base = build_reference_runtime_bootstrap(&project_id, grants)?;
    let movement_stream_id = base
        .stream_for(&principal)
        .expect("neutral fixture allocates its declared principal stream");
    let interaction_stream_id = base
        .stream_for(&interaction_principal)
        .expect("neutral fixture allocates the interaction system stream");
    let agent_stream_id = base
        .stream_for(&agent_principal)
        .expect("neutral fixture allocates the agent planner stream");
    let cognition_stream_id = base
        .stream_for(&cognition_principal)
        .expect("neutral fixture allocates the cognition boundary stream");
    let activity_stream_id = base
        .stream_for(&activity_principal)
        .expect("neutral fixture allocates the activity boundary stream");
    let mut bootstrap = base.bootstrap;
    let mut authority = base.authority;
    let cognition_rpg_stream_id = bootstrap
        .stream_registry
        .allocate_stream(cognition_principal.clone())?;
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
        activated_project.project_lock.project_lock_sha256;
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
                .map(next_contracts::mechanics::interaction_definition_hash_v2),
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
    let action_map = ActionMapManifestV1::core_keyboard_mouse_controller_v1()?;
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
    let carried_load_id = PersistentId::from_bytes([0x7b; 16]);
    let carried_load_shape_id = PhysicsShapeIdV1 {
        body_id: physics_body_id,
        shape_slot: 1,
    };
    let r5b_course = ReferenceCapsuleCourseV1::production_v1();
    let interactive_object_id = PersistentId::from_bytes([0x58; 16]);
    let npc_character_id = PersistentId::from_bytes([0x59; 16]);
    let player_body_condition_id = PersistentId::from_bytes([0xc0; 16]);
    let npc_body_condition_id = PersistentId::from_bytes([0xc1; 16]);
    let body_projections =
        ReferenceBodyProjectionSetV1::compile(&activated_project, body_id, npc_character_id)?;
    let procedural_motor = next_motor::CapsuleProceduralMotorControllerV1::activate(
        &activated_project.body_schema_asset.body_schema,
        &body_projections.player.instance,
        &body_projections.player.roots,
        physics_body_id,
        bootstrap.tick_rate_profile.gameplay_hz,
    )?;
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
    let quest_giver_character_id = activated_project
        .world_routine_catalog_or_none
        .as_ref()
        .ok_or(ReferenceGameError::WorldRoutineContentInvalid)?
        .routine
        .subject_id;
    let cognition_subject_id = activated_project.agent_cognition_catalog.subject_id;
    if [
        controller_id,
        body_id,
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
        player_body_condition_id,
        npc_body_condition_id,
        PersistentId::from_bytes([0x57; 16]),
        PersistentId::from_bytes([0x71; 16]),
        PersistentId::from_bytes([0x72; 16]),
        PersistentId::from_bytes([0x73; 16]),
        PersistentId::from_bytes([0x74; 16]),
        r5b_course.static_body_id.subject_id,
        r5b_course.dynamic_body_id.subject_id,
        r5b_course.sensor_body_id.subject_id,
        carried_load_id,
    ]
    .iter()
    .any(|identity| *identity == quest_giver_character_id || *identity == cognition_subject_id)
        || quest_giver_character_id == cognition_subject_id
    {
        return Err(ReferenceGameError::WorldRoutineContentInvalid);
    }
    bootstrap.physics_checkpoint = grounded_capsule_checkpoint(
        PhysicsWorldId::from_bytes(*bootstrap.world_identity.world_namespace.as_bytes()),
        physics_body_id,
        quest_giver_character_id,
        &bootstrap.tick_rate_profile,
        &bootstrap.authoritative_numeric_profile,
        &bootstrap.physics_quantization_profile,
    )?;
    let (source_graph_hash, source_clip_hash) =
        crate::physical_animation::reference_root_motion_source_hashes(
            &activated_project,
            bootstrap.tick_rate_profile.gameplay_hz,
        )?;
    authority.register_root_motion_source(
        principal.clone(),
        body_id,
        source_graph_hash,
        source_clip_hash,
    )?;
    Ok(ReferenceGameSession {
        bootstrap,
        authority,
        principal,
        movement_stream_id,
        rpg_stream_id,
        interaction_stream_id,
        source_id,
        controller_id,
        body_id,
        physics_body_id,
        carried_load_id,
        carried_load_shape_id,
        carried_load_local_translation_micrometres: CARRIED_LOAD_LOCAL_TRANSLATION_MICROMETRES,
        r5b_course,
        interactive_object_id,
        npc_character_id,
        player_body_condition_id,
        npc_body_condition_id,
        body_projections,
        procedural_motor,
        quest_giver_character_id,
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
        cognition_principal,
        cognition_stream_id,
        cognition_rpg_stream_id,
        cognition_subject_id,
        activity_principal,
        activity_stream_id,
        action_map,
        action_map_hash,
        context_stack,
        context_stack_hash,
        activated_project,
        world_topology,
    })
}

fn grounded_capsule_checkpoint(
    world_id: PhysicsWorldId,
    capsule_body_id: PhysicsBodyIdV1,
    quest_giver_character_id: PersistentId,
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
    let carried_load_shape_id = PhysicsShapeIdV1 {
        body_id: capsule_body_id,
        shape_slot: 1,
    };
    let carried_load_shape = box_shape_descriptor(
        carried_load_shape_id,
        &material_id,
        CARRIED_LOAD_LOCAL_TRANSLATION_MICROMETRES,
        CARRIED_LOAD_HALF_EXTENTS_MICROMETRES,
        CARRIED_LOAD_COLLISION_LAYER,
        CARRIED_LOAD_COLLISION_MASK,
    );
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
        // R5j's carried load is a fixed local compound shape on the same
        // kinematic body. It therefore has no second transform/save owner.
        shapes: BTreeMap::from([
            (capsule_shape_id, capsule_shape),
            (carried_load_shape_id, carried_load_shape),
        ]),
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
        WORLD_COLLISION_LAYER,
        WORLD_COLLISION_MASK,
    );
    let relay_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x58; 16]),
        body_slot: 0,
    };
    let relay = relay_gate_descriptor(relay_body_id, &material_id);
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
        WORLD_COLLISION_LAYER,
        WORLD_COLLISION_MASK,
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
        WORLD_COLLISION_LAYER,
        WORLD_COLLISION_MASK,
    );
    let quest_giver_body_id = PhysicsBodyIdV1 {
        subject_id: quest_giver_character_id,
        body_slot: 0,
    };
    let quest_giver_shape_id = PhysicsShapeIdV1 {
        body_id: quest_giver_body_id,
        shape_slot: 0,
    };
    let quest_giver = static_box_descriptor(
        quest_giver_body_id,
        quest_giver_shape_id,
        &material_id,
        [-500_000, 900_000, 0],
        [100_000, 900_000, 100_000],
        WORLD_COLLISION_LAYER,
        WORLD_COLLISION_MASK,
    );
    // Four inset proxies match the four large authored rocks in the batched
    // environment mesh. Each box stays inside the visible silhouette so a
    // player can never collide with an invisible continuation.
    let rock_boxes = [
        (0x71, [-2_800_000, 520_000, -1_800_000]),
        (0x72, [2_800_000, 520_000, -1_400_000]),
        (0x73, [-3_000_000, 520_000, 2_500_000]),
        (0x74, [3_000_000, 520_000, 2_800_000]),
    ]
    .map(|(persistent_byte, translation)| {
        let body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([persistent_byte; 16]),
            body_slot: 0,
        };
        let shape_id = PhysicsShapeIdV1 {
            body_id,
            shape_slot: 0,
        };
        (
            body_id,
            static_box_descriptor(
                body_id,
                shape_id,
                &material_id,
                translation,
                [320_000, 500_000, 300_000],
                WORLD_COLLISION_LAYER,
                WORLD_COLLISION_MASK,
            ),
        )
    });
    let r5b_course = ReferenceCapsuleCourseV1::production_v1();
    let course_static = r5b_course_static_descriptor(r5b_course.static_body_id, &material_id);
    let course_dynamic = r5b_course_dynamic_descriptor(
        r5b_course.dynamic_body_id,
        r5b_course.dynamic_shape_id,
        &material_id,
    );
    let course_sensor = r5b_course_sensor_descriptor(
        r5b_course.sensor_body_id,
        r5b_course.sensor_shape_id,
        &material_id,
    );
    let mut bodies = BTreeMap::from([
        (capsule_body_id, capsule),
        (floor_body_id, floor),
        (relay_body_id, relay),
        (pickup_proxy_body_id, pickup_proxy),
        (npc_body_id, npc),
        (quest_giver_body_id, quest_giver),
        (r5b_course.static_body_id, course_static),
        (r5b_course.dynamic_body_id, course_dynamic),
        (r5b_course.sensor_body_id, course_sensor),
    ]);
    bodies.extend(rock_boxes);
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
        bodies,
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
    collision_layer: u8,
    collision_mask: u64,
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
            box_shape_descriptor(
                shape_id,
                material_id,
                [0; 3],
                half_extents_micrometres,
                collision_layer,
                collision_mask,
            ),
        )]),
    }
}

fn r5b_course_static_descriptor(
    body_id: PhysicsBodyIdV1,
    material_id: &SchemaId,
) -> PhysicsBodyDescriptorV1 {
    let boxes = [
        (
            [600_000, 25_000, R5B_COURSE_Z_MICROMETRES],
            [200_000, 25_000, 500_000],
        ),
        (
            [1_000_000, 50_000, R5B_COURSE_Z_MICROMETRES],
            [200_000, 50_000, 500_000],
        ),
        (
            [1_400_000, 75_000, R5B_COURSE_Z_MICROMETRES],
            [200_000, 75_000, 500_000],
        ),
        (
            [1_800_000, 100_000, R5B_COURSE_Z_MICROMETRES],
            [200_000, 100_000, 500_000],
        ),
        (
            [2_200_000, 200_000, R5B_COURSE_Z_MICROMETRES],
            [200_000, 200_000, 500_000],
        ),
        (
            [3_000_000, 300_000, R5B_COURSE_Z_MICROMETRES],
            [600_000, 300_000, 500_000],
        ),
        (
            [6_800_000, 900_000, R5B_COURSE_Z_MICROMETRES],
            [100_000, 900_000, 500_000],
        ),
    ];
    let mut shapes: BTreeMap<_, _> = boxes
        .into_iter()
        .enumerate()
        .map(|(shape_slot, (translation, half_extents))| {
            let shape_id = PhysicsShapeIdV1 {
                body_id,
                shape_slot: u32::try_from(shape_slot).expect("bounded R5b shape count"),
            };
            (
                shape_id,
                box_shape_descriptor(
                    shape_id,
                    material_id,
                    translation,
                    half_extents,
                    WORLD_COLLISION_LAYER,
                    WORLD_COLLISION_MASK,
                ),
            )
        })
        .collect();
    // The visible tall wall also opts into the carried-load channel. Keeping
    // this proxy separate from its world-layer twin preserves the existing
    // capsule/dynamic-box course while making payload clearance intentional.
    let carry_blocker_shape_id = PhysicsShapeIdV1 {
        body_id,
        shape_slot: 7,
    };
    shapes.insert(
        carry_blocker_shape_id,
        box_shape_descriptor(
            carry_blocker_shape_id,
            material_id,
            [6_800_000, 900_000, R5B_COURSE_Z_MICROMETRES],
            [100_000, 900_000, 500_000],
            CARRIED_LOAD_COLLISION_LAYER,
            CARRIED_LOAD_COLLISION_MASK,
        ),
    );
    PhysicsBodyDescriptorV1 {
        body_id,
        descriptor_revision: 1,
        motion_kind: PhysicsMotionKindV1::Static,
        initial_pose: PhysicsPoseV1::default(),
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        shapes,
    }
}

fn r5b_course_dynamic_descriptor(
    body_id: PhysicsBodyIdV1,
    shape_id: PhysicsShapeIdV1,
    material_id: &SchemaId,
) -> PhysicsBodyDescriptorV1 {
    PhysicsBodyDescriptorV1 {
        body_id,
        descriptor_revision: 1,
        motion_kind: PhysicsMotionKindV1::Dynamic,
        initial_pose: PhysicsPoseV1 {
            translation_micrometres: [5_200_000, 300_000, R5B_COURSE_Z_MICROMETRES],
            ..PhysicsPoseV1::default()
        },
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        shapes: BTreeMap::from([(
            shape_id,
            box_shape_descriptor(
                shape_id,
                material_id,
                [0; 3],
                [200_000, 300_000, 200_000],
                WORLD_COLLISION_LAYER,
                WORLD_COLLISION_MASK,
            ),
        )]),
    }
}

fn r5b_course_sensor_descriptor(
    body_id: PhysicsBodyIdV1,
    shape_id: PhysicsShapeIdV1,
    material_id: &SchemaId,
) -> PhysicsBodyDescriptorV1 {
    let mut shape = box_shape_descriptor(
        shape_id,
        material_id,
        [0; 3],
        [100_000, 900_000, 500_000],
        WORLD_COLLISION_LAYER,
        WORLD_COLLISION_MASK,
    );
    shape.participation = PhysicsParticipationV1::Sensor;
    PhysicsBodyDescriptorV1 {
        body_id,
        descriptor_revision: 1,
        motion_kind: PhysicsMotionKindV1::Static,
        initial_pose: PhysicsPoseV1 {
            translation_micrometres: [4_300_000, 900_000, R5B_COURSE_Z_MICROMETRES],
            ..PhysicsPoseV1::default()
        },
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        shapes: BTreeMap::from([(shape_id, shape)]),
    }
}

fn relay_gate_descriptor(
    body_id: PhysicsBodyIdV1,
    material_id: &SchemaId,
) -> PhysicsBodyDescriptorV1 {
    let shape = |shape_slot, local_translation, half_extents_micrometres| {
        let shape_id = PhysicsShapeIdV1 {
            body_id,
            shape_slot,
        };
        (
            shape_id,
            box_shape_descriptor(
                shape_id,
                material_id,
                local_translation,
                half_extents_micrometres,
                WORLD_COLLISION_LAYER,
                WORLD_COLLISION_MASK,
            ),
        )
    };
    PhysicsBodyDescriptorV1 {
        body_id,
        descriptor_revision: 1,
        motion_kind: PhysicsMotionKindV1::Static,
        initial_pose: PhysicsPoseV1 {
            translation_micrometres: [0, 900_000, 700_000],
            ..PhysicsPoseV1::default()
        },
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        // These four boxes match the authored relay mesh: two pillars, the
        // top beam and the central switch. The former single 20-metre-wide
        // test wall extended far beyond every visible surface.
        shapes: BTreeMap::from([
            shape(0, [-1_425_000, 225_000, 0], [175_000, 1_125_000, 180_000]),
            shape(1, [1_425_000, 225_000, 0], [175_000, 1_125_000, 180_000]),
            shape(2, [0, 1_250_000, 0], [1_600_000, 200_000, 200_000]),
            // Keep the switch collider well inside the visible 600-mm depth.
            // Its front face matches the pickup proxy at z=600 mm, preserving
            // the physical-contact proof required by the pickup transaction.
            shape(3, [0, -300_000, 0], [320_000, 600_000, 100_000]),
        ]),
    }
}

fn box_shape_descriptor(
    shape_id: PhysicsShapeIdV1,
    material_id: &SchemaId,
    local_translation: [i64; 3],
    half_extents_micrometres: [i64; 3],
    collision_layer: u8,
    collision_mask: u64,
) -> PhysicsShapeDescriptorV1 {
    PhysicsShapeDescriptorV1 {
        shape_id,
        descriptor_revision: 1,
        local_pose: PhysicsPoseV1 {
            translation_micrometres: local_translation,
            ..PhysicsPoseV1::default()
        },
        geometry: PhysicsGeometryV1::Box {
            half_extents_micrometres,
        },
        material_id: material_id.clone(),
        collision_layer,
        collision_mask,
        participation: PhysicsParticipationV1::Solid,
        contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
    }
}
