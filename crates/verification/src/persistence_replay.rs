use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::SaveStore;
use next_contracts::{
    AuthorityGrant, CORE_DIALOGUE_ACCEPTED_NODE_ID, CORE_DIALOGUE_QUEST_TRUST_DELTA,
    CORE_EQUIPMENT_MAIN_HAND_SLOT_ID, CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID,
    CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID, CORE_QUEST_ACTIVE_STATE_ID,
    CORE_RELATIONSHIP_TRUST_DIMENSION_ID, CharacterPayloadV1, CommandLedgerHash, ContactPhaseV1,
    ContentHash, EventPayload, InputMappingCodeV1, InventoryPayloadV1, IssuerPrincipal,
    ItemPayloadV1, PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID, PersistentId,
    PhysicsPoseV1, PhysicsWorldCheckpointV1, PlayerActionPhaseV1, RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
    RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION,
    RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID, RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID,
    RUNTIME_SNAPSHOT_SCHEMA_VERSION, RUNTIME_SNAPSHOT_SEGMENT_ID, ReplayComparePointV4,
    ReplayManifestV4, ReplayOwnerSegmentV2, ReplayTickManifestV4, RpgAggregateEnvelopeV1,
    RpgAggregateKindV1, RpgAggregatePayloadV1, RpgAggregateRefV1, RpgCommandV1, RpgEventV1,
    RpgOperationPayloadV1, RpgOperationV1, RpgSnapshotV2, SaveCompatibility, SaveSegmentDescriptor,
    SchemaId, StateRoot, TickSettings, WorldCheckpointV4, WorldCommand,
};
use next_physics_api::PhysicsBackendPolicy;
use next_runtime::{PhysicsLaunchOptions, RuntimeState, TickReport};

use crate::player_fixture::fixture_aggregate;
use crate::{
    ReplayOutput, build_neutral_player_fixture, build_physx_player_fixture,
    checkpoint_segment_hashes, compute_world_checkpoint_root, core_interaction_rpg_snapshot,
    player_action_sample, player_equip_use_sample, player_interact_sample, player_pickup_sample,
    replay_command_results, run_replay_manifest_with_physics_options,
};

static NEXT_CHECK_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistenceReplayCheckReport {
    pub ticks: u64,
    pub generations: u64,
    pub final_pose: PhysicsPoseV1,
    pub rpg_events: u64,
    pub interactive_object_state: SchemaId,
    pub dialogue_node_id: SchemaId,
    pub quest_state_id: SchemaId,
    pub npc_player_trust: i32,
    pub final_state_root: StateRoot,
    pub final_command_ledger_hash: CommandLedgerHash,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PersistenceReplayBackend {
    #[default]
    Reference,
    PhysX,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistenceReplayCheckError {
    context: &'static str,
    detail: String,
}

impl PersistenceReplayCheckError {
    fn new(context: &'static str, detail: impl Into<String>) -> Self {
        Self {
            context,
            detail: detail.into(),
        }
    }

    fn condition(context: &'static str) -> Self {
        Self::new(context, "acceptance condition was false")
    }
}

impl Display for PersistenceReplayCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for PersistenceReplayCheckError {}

struct CheckDirectory {
    path: PathBuf,
}

impl CheckDirectory {
    fn new() -> Result<Self, PersistenceReplayCheckError> {
        let sequence = NEXT_CHECK_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-persistence-replay-{}-{sequence}",
            std::process::id()
        ));
        match fs::remove_dir_all(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(PersistenceReplayCheckError::new(
                    "remove stale product-check directory",
                    error.to_string(),
                ));
            }
        }
        fs::create_dir_all(&path).map_err(|error| {
            PersistenceReplayCheckError::new("create product-check directory", error.to_string())
        })?;
        Ok(Self { path })
    }
}

impl Drop for CheckDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn run_persistence_replay_check()
-> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    run_persistence_replay_check_with_backend(PersistenceReplayBackend::Reference)
}

pub fn run_persistence_replay_check_with_backend(
    backend: PersistenceReplayBackend,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    match backend {
        PersistenceReplayBackend::Reference => run_persistence_replay_check_for_project(
            backend,
            "nextengine.persistence-replay",
            false,
        ),
        PersistenceReplayBackend::PhysX => run_persistence_replay_check_for_project(
            backend,
            "nextengine.persistence-replay.physx",
            true,
        ),
    }
}

pub(crate) fn run_persistence_replay_check_for_project(
    backend: PersistenceReplayBackend,
    project_id: &str,
    physx_compatible_profile: bool,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    let physics_options = match backend {
        PersistenceReplayBackend::Reference => PhysicsLaunchOptions::default(),
        PersistenceReplayBackend::PhysX => {
            PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX)
        }
    };
    let fixture = if physx_compatible_profile {
        build_physx_player_fixture(project_id)
    } else {
        build_neutral_player_fixture(project_id)
    }
    .map_err(|error| PersistenceReplayCheckError::new("build player fixture", error.to_string()))?;
    let initial_rpg = initial_rpg_snapshot(&fixture)?;
    let mut direct = RuntimeState::with_rpg_snapshot_and_physics_options(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        initial_rpg,
        physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("create runtime", error.to_string()))?;

    let initial_checkpoint = direct.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("initial checkpoint", error.to_string())
    })?;
    let direct_commands = rpg_commands(fixture.rpg_stream_id, fixture.principal.clone())?;
    let mut reports = Vec::new();
    for sequence in 0_u64..4 {
        let input = player_action_sample(
            &fixture,
            sequence,
            if sequence == 0 {
                PlayerActionPhaseV1::Started
            } else {
                PlayerActionPhaseV1::Performed
            },
            [0, 32_767],
            Some(
                i64::try_from(sequence).map_err(|error| {
                    PersistenceReplayCheckError::new("movement wall time", error.to_string())
                })? * 1_000,
            ),
        )
        .map_err(|error| PersistenceReplayCheckError::new("movement input", error.to_string()))?;
        direct
            .enqueue_input_sample(&fixture.principal, input)
            .map_err(|error| {
                PersistenceReplayCheckError::new("enqueue movement input", error.to_string())
            })?;
        let commands = if sequence == 0 {
            direct_commands.clone()
        } else {
            Vec::new()
        };
        reports.push(direct.run_tick(commands).map_err(|error| {
            PersistenceReplayCheckError::new("run pre-save movement", error.to_string())
        })?);
    }

    let pickup = player_pickup_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, None)
        .map_err(|error| PersistenceReplayCheckError::new("pickup input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, pickup)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue pickup input", error.to_string())
        })?;
    reports.push(direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("run pickup interaction", error.to_string())
    })?);

    let equip = player_equip_use_sample(&fixture, 5, PlayerActionPhaseV1::Started, true, None)
        .map_err(|error| PersistenceReplayCheckError::new("equip input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, equip)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue equip input", error.to_string())
        })?;
    reports.push(direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("run equip interaction", error.to_string())
    })?);

    let switch_interaction =
        player_interact_sample(&fixture, 6, PlayerActionPhaseV1::Started, true, None)
            .map_err(|error| PersistenceReplayCheckError::new("switch input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, switch_interaction)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue switch input", error.to_string())
        })?;
    reports.push(direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("run switch interaction", error.to_string())
    })?);

    let backward = player_action_sample(
        &fixture,
        7,
        PlayerActionPhaseV1::Performed,
        [0, -32_767],
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("backward input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, backward)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue backward input", error.to_string())
        })?;
    reports.push(direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("run backward movement", error.to_string())
    })?);

    for sequence in 8_u64..11 {
        let right = player_action_sample(
            &fixture,
            sequence,
            if sequence == 8 {
                PlayerActionPhaseV1::Started
            } else {
                PlayerActionPhaseV1::Performed
            },
            [32_767, 0],
            None,
        )
        .map_err(|error| PersistenceReplayCheckError::new("right input", error.to_string()))?;
        direct
            .enqueue_input_sample(&fixture.principal, right)
            .map_err(|error| {
                PersistenceReplayCheckError::new("enqueue right input", error.to_string())
            })?;
        reports.push(direct.run_tick([]).map_err(|error| {
            PersistenceReplayCheckError::new("run right movement", error.to_string())
        })?);
    }
    if !direct
        .physics_snapshot()
        .sorted_contact_continuity_states
        .values()
        .any(|contact| {
            contact.participant_low.body_id.subject_id == fixture.npc_character_id
                || contact.participant_high.body_id.subject_id == fixture.npc_character_id
        })
    {
        return Err(PersistenceReplayCheckError::condition(
            "save boundary has active NPC contact",
        ));
    }

    let queued_interaction = player_interact_sample(
        &fixture,
        11,
        PlayerActionPhaseV1::Started,
        true,
        Some(11_999_999),
    )
    .map_err(|error| PersistenceReplayCheckError::new("interaction input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, queued_interaction)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue interaction input", error.to_string())
        })?;

    let directory = CheckDirectory::new()?;
    let store = SaveStore::new(&directory.path);
    let compatibility = compatibility()?;
    let saved_checkpoint = direct.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("mid-run checkpoint", error.to_string())
    })?;
    let generation_zero = store
        .commit_world_checkpoint(compatibility.clone(), &saved_checkpoint)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit generation zero", error.to_string())
        })?;
    if generation_zero.generation != 0 {
        return Err(PersistenceReplayCheckError::condition(
            "first generation is zero",
        ));
    }
    let loaded = store.load_latest(&compatibility).map_err(|error| {
        PersistenceReplayCheckError::new("load generation zero", error.to_string())
    })?;
    let mut restored = RuntimeState::restore_world_checkpoint_with_physics_options(
        loaded.checkpoint,
        fixture.authority.clone(),
        physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("restore checkpoint", error.to_string()))?;

    let direct_interaction = direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("direct interaction", error.to_string())
    })?;
    let restored_interaction = restored.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("restored interaction", error.to_string())
    })?;
    if direct_interaction != restored_interaction
        || direct_interaction.mapping_receipts.len() != 1
        || direct_interaction.mapping_receipts[0].code != InputMappingCodeV1::Accepted
        || direct_interaction.mapping_receipts[0]
            .derived_command_id
            .is_none()
        || direct_interaction
            .contact_batch
            .events
            .iter()
            .any(|event| event.phase == ContactPhaseV1::Begin)
        || !direct_interaction
            .contact_batch
            .events
            .iter()
            .any(|event| event.phase == ContactPhaseV1::Persist)
        || direct_interaction
            .events
            .iter()
            .filter(|event| {
                matches!(
                    event.payload,
                    EventPayload::Rpg(
                        RpgEventV1::DialogueAdvanced { .. }
                            | RpgEventV1::QuestTransitioned { .. }
                            | RpgEventV1::RelationshipAdjusted { .. }
                    )
                )
            })
            .count()
            != 3
    {
        return Err(PersistenceReplayCheckError::condition(
            "queued contact-gated interaction continues exactly after restore",
        ));
    }
    reports.push(direct_interaction);

    let left = player_action_sample(
        &fixture,
        12,
        PlayerActionPhaseV1::Performed,
        [-32_767, 0],
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("left input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, left.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct left", error.to_string())
        })?;
    restored
        .enqueue_input_sample(&fixture.principal, left)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored left", error.to_string())
        })?;
    let direct_left = direct
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct left", error.to_string()))?;
    let restored_left = restored
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("restored left", error.to_string()))?;
    if direct_left != restored_left
        || !direct_left
            .contact_batch
            .events
            .iter()
            .any(|event| event.phase == ContactPhaseV1::End)
    {
        return Err(PersistenceReplayCheckError::condition(
            "left movement ends NPC contact exactly after interaction restore",
        ));
    }
    reports.push(direct_left);

    let stop = player_action_sample(&fixture, 13, PlayerActionPhaseV1::Completed, [0, 0], None)
        .map_err(|error| PersistenceReplayCheckError::new("stop input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, stop.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct stop", error.to_string())
        })?;
    restored
        .enqueue_input_sample(&fixture.principal, stop)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored stop", error.to_string())
        })?;
    let direct_stop = direct
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct stop", error.to_string()))?;
    let restored_stop = restored
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("restored stop", error.to_string()))?;
    if direct_stop != restored_stop
        || direct.world_checkpoint().map_err(|error| {
            PersistenceReplayCheckError::new("direct final checkpoint", error.to_string())
        })? != restored.world_checkpoint().map_err(|error| {
            PersistenceReplayCheckError::new("restored final checkpoint", error.to_string())
        })?
    {
        return Err(PersistenceReplayCheckError::condition(
            "uninterrupted and restored worlds remain exact",
        ));
    }
    reports.push(direct_stop);

    let replay_manifest = replay_manifest(
        compatibility.clone(),
        &fixture.authority,
        initial_checkpoint,
        &reports,
        vec![
            direct_commands,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ],
    )?;
    let replay = run_replay_manifest_with_physics_options(&replay_manifest, physics_options)
        .map_err(|error| {
            PersistenceReplayCheckError::new("closed-batch replay", error.to_string())
        })?;
    compare_replay(&direct, &reports, &replay)?;

    let final_checkpoint = direct
        .world_checkpoint()
        .map_err(|error| PersistenceReplayCheckError::new("final checkpoint", error.to_string()))?;
    let generation_one = store
        .commit_world_checkpoint(compatibility.clone(), &final_checkpoint)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit generation one", error.to_string())
        })?;
    if generation_one.generation != 1 {
        return Err(PersistenceReplayCheckError::condition(
            "second generation is one",
        ));
    }
    let (corrupt_path, corrupt_bytes) =
        corrupt_rpg_segment(&store, &compatibility, generation_one.slot)?;
    let fallback = store
        .load_latest(&compatibility)
        .map_err(|error| PersistenceReplayCheckError::new("load fallback", error.to_string()))?;
    if fallback.image.manifest.generation != 0
        || fallback.rejected_generations.len() != 1
        || fallback.checkpoint != saved_checkpoint
        || !fallback.rejected_generations[0]
            .original_files
            .iter()
            .any(|file| file.bytes == corrupt_bytes)
        || fs::read(&corrupt_path).map_err(|error| {
            PersistenceReplayCheckError::new("read corrupt source", error.to_string())
        })? != corrupt_bytes
    {
        return Err(PersistenceReplayCheckError::condition(
            "corrupt RPG generation falls back without rewriting bytes",
        ));
    }

    let physics_directory = CheckDirectory::new()?;
    let physics_store = SaveStore::new(&physics_directory.path);
    physics_store
        .commit_world_checkpoint(compatibility.clone(), &saved_checkpoint)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit physics fallback baseline", error.to_string())
        })?;
    let physics_latest = physics_store
        .commit_world_checkpoint(compatibility.clone(), &final_checkpoint)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit physics fallback candidate", error.to_string())
        })?;
    let (corrupt_physics_path, corrupt_physics_bytes) =
        corrupt_physics_segment(&physics_store, &compatibility, physics_latest.slot)?;
    let physics_fallback = physics_store.load_latest(&compatibility).map_err(|error| {
        PersistenceReplayCheckError::new("load physics fallback", error.to_string())
    })?;
    if physics_fallback.image.manifest.generation != 0
        || physics_fallback.checkpoint != saved_checkpoint
        || physics_fallback
            .rejected_generations
            .first()
            .is_none_or(|rejected| {
                !rejected
                    .original_files
                    .iter()
                    .any(|file| file.bytes == corrupt_physics_bytes)
            })
        || fs::read(&corrupt_physics_path).map_err(|error| {
            PersistenceReplayCheckError::new("read corrupt physics source", error.to_string())
        })? != corrupt_physics_bytes
    {
        return Err(PersistenceReplayCheckError::condition(
            "corrupt physics generation remains a separate exact fallback regression",
        ));
    }

    let final_pose = final_checkpoint
        .physics_checkpoint
        .snapshot
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or_else(|| PersistenceReplayCheckError::condition("final capsule body exists"))?
        .pose;
    if final_pose.translation_micrometres != [200_000, 900_000, 200_000] {
        return Err(PersistenceReplayCheckError::condition(
            "queued movement applies exactly once",
        ));
    }
    let interactive_object_state = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::InteractiveObject,
        fixture.interactive_object_id,
    ) {
        Some(RpgAggregatePayloadV1::InteractiveObject(object)) => object.state_id.clone(),
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final interactive object exists",
            ));
        }
    };
    let rpg_events = reports
        .iter()
        .flat_map(|report| &report.events)
        .filter(|event| matches!(event.payload, EventPayload::Rpg(_)))
        .count();
    let dialogue_node_id = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::Dialogue,
        fixture.dialogue_id,
    ) {
        Some(RpgAggregatePayloadV1::Dialogue(dialogue)) => dialogue.node_id.clone(),
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final core dialogue exists",
            ));
        }
    };
    let quest_state_id = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::Quest,
        fixture.quest_id,
    ) {
        Some(RpgAggregatePayloadV1::Quest(quest)) => quest.state_id.clone(),
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final core quest exists",
            ));
        }
    };
    let npc_player_trust = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::Relationship,
        fixture.relationship_id,
    ) {
        Some(RpgAggregatePayloadV1::Relationship(relationship))
            if relationship.source_id == fixture.npc_character_id
                && relationship.target_id == fixture.body_id =>
        {
            relationship
                .dimensions
                .iter()
                .find(|dimension| {
                    dimension.dimension_id.as_str() == CORE_RELATIONSHIP_TRUST_DIMENSION_ID
                })
                .map_or(0, |dimension| dimension.value)
        }
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final core relationship exists",
            ));
        }
    };
    let pickup_is_collected = matches!(
        aggregate_payload(
            &final_checkpoint.rpg_snapshot,
            RpgAggregateKindV1::InteractiveObject,
            fixture.pickup_proxy_id,
        ),
        Some(RpgAggregatePayloadV1::InteractiveObject(object))
            if object.state_id.as_str() == CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID
    );
    let pickup_is_owned = matches!(
        aggregate_payload(
            &final_checkpoint.rpg_snapshot,
            RpgAggregateKindV1::Inventory,
            fixture.player_inventory_id,
        ),
        Some(RpgAggregatePayloadV1::Inventory(inventory))
            if inventory.item_ids == [fixture.pickup_item_id]
    );
    let pickup_is_equipped = matches!(
        aggregate_payload(
            &final_checkpoint.rpg_snapshot,
            RpgAggregateKindV1::Equipment,
            fixture.player_equipment_id,
        ),
        Some(RpgAggregatePayloadV1::Equipment(equipment))
            if equipment.assignments.iter().any(|assignment| {
                assignment.slot_id.as_str() == CORE_EQUIPMENT_MAIN_HAND_SLOT_ID
                    && assignment.item_id == fixture.pickup_item_id
            })
    );
    if interactive_object_state.as_str() != CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
        || dialogue_node_id.as_str() != CORE_DIALOGUE_ACCEPTED_NODE_ID
        || quest_state_id.as_str() != CORE_QUEST_ACTIVE_STATE_ID
        || npc_player_trust != CORE_DIALOGUE_QUEST_TRUST_DELTA
        || rpg_events != 9
        || !pickup_is_collected
        || !pickup_is_owned
        || !pickup_is_equipped
    {
        return Err(PersistenceReplayCheckError::condition(
            "interaction activates object exactly once",
        ));
    }
    let final_state_root = compute_world_checkpoint_root(&final_checkpoint)
        .map_err(|error| PersistenceReplayCheckError::new("final state root", error.to_string()))?;
    let final_command_ledger_hash = final_checkpoint
        .runtime_snapshot
        .command_ledger_hash()
        .map_err(|error| {
            PersistenceReplayCheckError::new("final ledger hash", error.to_string())
        })?;
    Ok(PersistenceReplayCheckReport {
        ticks: 14,
        generations: 2,
        final_pose,
        rpg_events: u64::try_from(rpg_events).map_err(|error| {
            PersistenceReplayCheckError::new("RPG event count", error.to_string())
        })?,
        interactive_object_state,
        dialogue_node_id,
        quest_state_id,
        npc_player_trust,
        final_state_root,
        final_command_ledger_hash,
    })
}

fn replay_manifest(
    compatibility: SaveCompatibility,
    authority: &next_runtime::AuthorityRegistry,
    initial_checkpoint: WorldCheckpointV4,
    reports: &[TickReport],
    direct_commands: Vec<Vec<WorldCommand>>,
) -> Result<ReplayManifestV4, PersistenceReplayCheckError> {
    let initial_state_root =
        compute_world_checkpoint_root(&initial_checkpoint).map_err(|error| {
            PersistenceReplayCheckError::new("initial replay root", error.to_string())
        })?;
    let initial_owner_segments = owner_segments(&initial_checkpoint)?;
    let authority = authority
        .entries()
        .map(|(principal, capabilities)| AuthorityGrant {
            principal: principal.clone(),
            capabilities: capabilities.iter().cloned().collect(),
        })
        .collect();
    let mut ticks = Vec::with_capacity(reports.len());
    let mut compare_points = Vec::with_capacity(reports.len());
    let physics_catalog = initial_checkpoint.physics_checkpoint.catalog.clone();
    for ((report, direct), expected_tick) in reports.iter().zip(direct_commands).zip(0_u64..) {
        if report.tick != expected_tick {
            return Err(PersistenceReplayCheckError::condition(
                "recorded replay ticks are contiguous",
            ));
        }
        let checkpoint = WorldCheckpointV4::new(
            report.snapshot.clone(),
            report.rpg_snapshot.clone(),
            PhysicsWorldCheckpointV1::new(physics_catalog.clone(), report.physics_snapshot.clone())
                .map_err(|error| {
                    PersistenceReplayCheckError::new("record physics checkpoint", error.to_string())
                })?,
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("record checkpoint", error.to_string())
        })?;
        let state_root = compute_world_checkpoint_root(&checkpoint).map_err(|error| {
            PersistenceReplayCheckError::new("record state root", error.to_string())
        })?;
        let (runtime_segment_hash, rpg_segment_hash, physics_segment_hash) =
            checkpoint_segment_hashes(&checkpoint).map_err(|error| {
                PersistenceReplayCheckError::new("record segment hashes", error.to_string())
            })?;
        let direct_external_commands = direct
            .iter()
            .map(next_contracts::ReplayCommandRecord::from_command)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                PersistenceReplayCheckError::new("record direct commands", error.to_string())
            })?;
        ticks.push(ReplayTickManifestV4 {
            tick: report.tick,
            closed_ingress_batch: report.closed_ingress_batch.clone(),
            direct_external_commands,
            expected_ingress_command_batch: report.command_batches[0].clone(),
            expected_physics_step_input: report.physics_step_input.clone(),
            expected_contact_batch: report.contact_batch.clone(),
            expected_outcome_command_batch: report.command_batches[1].clone(),
            expected_mapping_receipts: report.mapping_receipts.clone(),
            expected_command_results: replay_command_results(&report.results),
            expected_events: report.events.clone(),
        });
        compare_points.push(ReplayComparePointV4 {
            tick: report.tick,
            state_root,
            command_ledger_hash: report.snapshot.command_ledger_hash().map_err(|error| {
                PersistenceReplayCheckError::new("record ledger hash", error.to_string())
            })?,
            runtime_segment_hash,
            rpg_segment_hash,
            physics_segment_hash,
            closed_ingress_batch_hash: report.closed_ingress_batch.batch_hash,
            ingress_command_batch_hash: report.command_batches[0].batch_hash,
            physics_step_input_hash: report.physics_step_input.input_hash().map_err(|error| {
                PersistenceReplayCheckError::new("record physics input hash", error.to_string())
            })?,
            contact_batch_hash: report.contact_batch.batch_hash,
            outcome_command_batch_hash: report.command_batches[1].batch_hash,
        });
    }
    Ok(ReplayManifestV4 {
        schema_version: next_contracts::REPLAY_MANIFEST_V4_SCHEMA_VERSION,
        compatibility,
        initial_owner_segments,
        initial_state_root,
        authority,
        ticks,
        compare_points,
    })
}

fn owner_segments(
    checkpoint: &WorldCheckpointV4,
) -> Result<Vec<ReplayOwnerSegmentV2>, PersistenceReplayCheckError> {
    let raw = [
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            RUNTIME_SNAPSHOT_SCHEMA_VERSION,
            checkpoint
                .runtime_snapshot
                .canonical_bytes()
                .map_err(|error| {
                    PersistenceReplayCheckError::new("runtime segment", error.to_string())
                })?,
        ),
        (
            RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION,
            checkpoint.rpg_snapshot.canonical_bytes().map_err(|error| {
                PersistenceReplayCheckError::new("RPG segment", error.to_string())
            })?,
        ),
        (
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            u32::from(PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION),
            checkpoint
                .physics_checkpoint
                .canonical_bytes()
                .map_err(|error| {
                    PersistenceReplayCheckError::new("physics segment", error.to_string())
                })?,
        ),
    ];
    let mut segments = raw
        .into_iter()
        .map(|(owner, schema, segment, version, canonical_bytes)| {
            let owner_id = SchemaId::new(owner).map_err(|error| {
                PersistenceReplayCheckError::new("replay owner ID", error.to_string())
            })?;
            let schema_id = SchemaId::new(schema).map_err(|error| {
                PersistenceReplayCheckError::new("replay schema ID", error.to_string())
            })?;
            let segment_id = SchemaId::new(segment).map_err(|error| {
                PersistenceReplayCheckError::new("replay segment ID", error.to_string())
            })?;
            let descriptor = SaveSegmentDescriptor::for_bytes(
                owner_id,
                schema_id,
                segment_id,
                version,
                &canonical_bytes,
            )
            .map_err(|error| {
                PersistenceReplayCheckError::new("replay segment descriptor", error.to_string())
            })?;
            Ok(ReplayOwnerSegmentV2 {
                descriptor,
                canonical_bytes,
            })
        })
        .collect::<Result<Vec<_>, PersistenceReplayCheckError>>()?;
    segments.sort_by(|left, right| {
        (
            &left.descriptor.owner_id,
            &left.descriptor.schema_id,
            &left.descriptor.segment_id,
        )
            .cmp(&(
                &right.descriptor.owner_id,
                &right.descriptor.schema_id,
                &right.descriptor.segment_id,
            ))
    });
    Ok(segments)
}

fn compare_replay(
    direct: &RuntimeState,
    reports: &[TickReport],
    replay: &ReplayOutput,
) -> Result<(), PersistenceReplayCheckError> {
    if reports.len() != replay.ticks.len() {
        return Err(PersistenceReplayCheckError::condition(
            "direct and replay tick counts match",
        ));
    }
    for (report, replay_tick) in reports.iter().zip(&replay.ticks) {
        let checkpoint = WorldCheckpointV4::new(
            report.snapshot.clone(),
            report.rpg_snapshot.clone(),
            PhysicsWorldCheckpointV1::new(
                direct.physics_checkpoint().catalog.clone(),
                report.physics_snapshot.clone(),
            )
            .map_err(|error| {
                PersistenceReplayCheckError::new("compare physics checkpoint", error.to_string())
            })?,
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("compare checkpoint", error.to_string())
        })?;
        let state_root = compute_world_checkpoint_root(&checkpoint).map_err(|error| {
            PersistenceReplayCheckError::new("compare state root", error.to_string())
        })?;
        if report.tick != replay_tick.tick
            || report.results != replay_tick.command_results
            || report.events != replay_tick.events
            || state_root != replay_tick.state_root
            || report.snapshot.command_ledger_hash().map_err(|error| {
                PersistenceReplayCheckError::new("compare ledger", error.to_string())
            })? != replay_tick.command_ledger_hash
        {
            return Err(PersistenceReplayCheckError::condition(
                "direct and replay compare points are exact",
            ));
        }
    }
    let direct_checkpoint = direct.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("direct replay checkpoint", error.to_string())
    })?;
    if replay.final_checkpoint != direct_checkpoint {
        return Err(PersistenceReplayCheckError::condition(
            "direct and replay final checkpoints are exact",
        ));
    }
    Ok(())
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

fn initial_rpg_snapshot(
    fixture: &crate::NeutralPlayerFixture,
) -> Result<RpgSnapshotV2, PersistenceReplayCheckError> {
    let mut snapshot = core_interaction_rpg_snapshot(fixture);
    let item_id = PersistentId::from_bytes([0x10; 16]);
    let first_character_id = PersistentId::from_bytes([0x20; 16]);
    let first_inventory_id = PersistentId::from_bytes([0x21; 16]);
    let second_character_id = PersistentId::from_bytes([0x30; 16]);
    let second_inventory_id = PersistentId::from_bytes([0x31; 16]);
    snapshot.aggregates.extend([
        fixture_aggregate(
            first_character_id,
            0x20,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(first_inventory_id),
                equipment_id: None,
                skills: Vec::new(),
            }),
        ),
        fixture_aggregate(
            second_character_id,
            0x30,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(second_inventory_id),
                equipment_id: None,
                skills: Vec::new(),
            }),
        ),
        fixture_aggregate(
            item_id,
            0x10,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 100,
                custom_state: Vec::new(),
            }),
        ),
        fixture_aggregate(
            first_inventory_id,
            0x21,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: first_character_id,
                capacity: 8,
                item_ids: vec![item_id],
                reservations: Vec::new(),
            }),
        ),
        fixture_aggregate(
            second_inventory_id,
            0x31,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: second_character_id,
                capacity: 8,
                item_ids: Vec::new(),
                reservations: Vec::new(),
            }),
        ),
    ]);
    snapshot
        .aggregates
        .sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
    Ok(snapshot)
}

fn rpg_commands(
    stream_id: next_contracts::CommandStreamId,
    principal: IssuerPrincipal,
) -> Result<Vec<WorldCommand>, PersistenceReplayCheckError> {
    let item_id = PersistentId::from_bytes([0x10; 16]);
    let first_inventory_id = PersistentId::from_bytes([0x21; 16]);
    let second_inventory_id = PersistentId::from_bytes([0x31; 16]);
    let operation =
        |source_inventory_id, destination_inventory_id, inventory_revision| RpgCommandV1 {
            operations: vec![RpgOperationV1 {
                operation_slot: 0,
                targets: vec![
                    RpgAggregateRefV1 {
                        aggregate_kind: RpgAggregateKindV1::Item,
                        persistent_id: item_id,
                        expected_revision: 0,
                    },
                    RpgAggregateRefV1 {
                        aggregate_kind: RpgAggregateKindV1::Inventory,
                        persistent_id: first_inventory_id,
                        expected_revision: inventory_revision,
                    },
                    RpgAggregateRefV1 {
                        aggregate_kind: RpgAggregateKindV1::Inventory,
                        persistent_id: second_inventory_id,
                        expected_revision: inventory_revision,
                    },
                ],
                definition_policy_hashes: Vec::new(),
                payload: RpgOperationPayloadV1::TransferItem {
                    item_id,
                    source_inventory_id: Some(source_inventory_id),
                    destination_inventory_id: Some(destination_inventory_id),
                    quantity: 1,
                },
            }],
        };
    Ok(vec![
        WorldCommand::rpg(
            stream_id,
            principal.clone(),
            0,
            0,
            operation(first_inventory_id, second_inventory_id, 0),
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("current RPG command", error.to_string())
        })?,
        WorldCommand::rpg(
            stream_id,
            principal,
            1,
            2,
            operation(second_inventory_id, first_inventory_id, 1),
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("future RPG command", error.to_string())
        })?,
    ])
}

fn compatibility() -> Result<SaveCompatibility, PersistenceReplayCheckError> {
    Ok(SaveCompatibility {
        engine_build_hash: ContentHash::from_bytes([1; 32]),
        game_build_hash: ContentHash::from_bytes([2; 32]),
        project_id: SchemaId::new("nextengine.persistence-replay").map_err(|error| {
            PersistenceReplayCheckError::new("save project ID", error.to_string())
        })?,
        schema_registry_hash: ContentHash::from_bytes([3; 32]),
        content_manifest_hash: ContentHash::from_bytes([4; 32]),
        mechanics_lock_hash: ContentHash::from_bytes([5; 32]),
        tick_settings: TickSettings {
            gameplay_hz: 30,
            physics_hz: 60,
            motor_hz: 60,
        },
        loaded_chunk_revisions: vec![],
        rng_stream_states: vec![],
        physical_bindings: vec![],
        policy_state_schemas: vec![],
        plugin_script_bindings: vec![],
    })
}

fn corrupt_rpg_segment(
    store: &SaveStore,
    compatibility: &SaveCompatibility,
    slot: u8,
) -> Result<(PathBuf, Vec<u8>), PersistenceReplayCheckError> {
    let latest = store.load_latest(compatibility).map_err(|error| {
        PersistenceReplayCheckError::new("inspect newest RPG generation", error.to_string())
    })?;
    let segment_index = latest
        .image
        .manifest
        .segments
        .iter()
        .position(|descriptor| descriptor.owner_id.as_str() == RPG_AGGREGATE_SNAPSHOT_OWNER_ID)
        .ok_or_else(|| PersistenceReplayCheckError::condition("newest RPG segment exists"))?;
    let mut corrupt_snapshot = latest.checkpoint.rpg_snapshot;
    let dialogue = corrupt_snapshot
        .aggregates
        .iter_mut()
        .find(|aggregate| aggregate.aggregate_kind == RpgAggregateKindV1::Dialogue)
        .ok_or_else(|| PersistenceReplayCheckError::condition("newest core dialogue exists"))?;
    let RpgAggregatePayloadV1::Dialogue(payload) = &dialogue.payload else {
        return Err(PersistenceReplayCheckError::condition(
            "newest core dialogue payload matches its kind",
        ));
    };
    let replacement = RpgAggregateEnvelopeV1::new(
        dialogue.persistent_id,
        dialogue.schema_version,
        dialogue.revision,
        dialogue.definition_ref.clone(),
        dialogue.provenance.clone(),
        RpgAggregatePayloadV1::Dialogue(next_contracts::DialoguePayloadV1 {
            speaker_id: payload.speaker_id,
            listener_id: PersistentId::from_bytes([0xee; 16]),
            node_id: payload.node_id.clone(),
        }),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("construct structural RPG corruption", error.to_string())
    })?;
    *dialogue = replacement;
    let bytes = corrupt_snapshot.canonical_bytes().map_err(|error| {
        PersistenceReplayCheckError::new("encode structural RPG corruption", error.to_string())
    })?;
    let segment_path = segment_path(store.root(), slot, segment_index);
    fs::write(&segment_path, &bytes).map_err(|error| {
        PersistenceReplayCheckError::new("corrupt RPG segment", error.to_string())
    })?;
    let mut manifest = latest.image.manifest;
    let descriptor = manifest.segments[segment_index].clone();
    manifest.segments[segment_index] = SaveSegmentDescriptor::for_bytes(
        descriptor.owner_id,
        descriptor.schema_id,
        descriptor.segment_id,
        descriptor.schema_version,
        &bytes,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("bind structural RPG corruption", error.to_string())
    })?;
    let manifest_path = store
        .root()
        .join(format!("slot-{slot}"))
        .join("manifest.jcs");
    fs::write(
        &manifest_path,
        manifest.to_jcs_bytes().map_err(|error| {
            PersistenceReplayCheckError::new(
                "encode corrupt RPG generation manifest",
                error.to_string(),
            )
        })?,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("write corrupt RPG generation manifest", error.to_string())
    })?;
    Ok((segment_path, bytes))
}

fn corrupt_physics_segment(
    store: &SaveStore,
    compatibility: &SaveCompatibility,
    slot: u8,
) -> Result<(PathBuf, Vec<u8>), PersistenceReplayCheckError> {
    let latest = store.load_latest(compatibility).map_err(|error| {
        PersistenceReplayCheckError::new("inspect newest generation", error.to_string())
    })?;
    let segment_index = latest
        .image
        .manifest
        .segments
        .iter()
        .position(|descriptor| descriptor.owner_id.as_str() == PHYSICS_SNAPSHOT_OWNER_ID)
        .ok_or_else(|| PersistenceReplayCheckError::condition("newest physics segment exists"))?;
    let mut corrupt_checkpoint = latest.checkpoint.physics_checkpoint;
    let static_body_id = corrupt_checkpoint
        .catalog
        .bodies
        .iter()
        .find_map(|(body_id, descriptor)| {
            (descriptor.motion_kind == next_contracts::PhysicsMotionKindV1::Static)
                .then_some(*body_id)
        })
        .ok_or_else(|| {
            PersistenceReplayCheckError::condition("newest physics catalog has a static body")
        })?;
    corrupt_checkpoint
        .snapshot
        .sorted_body_states
        .get_mut(&static_body_id)
        .ok_or_else(|| {
            PersistenceReplayCheckError::condition("newest physics snapshot has static state")
        })?
        .linear_velocity_micrometres_per_second[0] = 1;
    let bytes = corrupt_checkpoint.canonical_bytes().map_err(|error| {
        PersistenceReplayCheckError::new("encode structural physics corruption", error.to_string())
    })?;
    let segment_path = segment_path(store.root(), slot, segment_index);
    fs::write(&segment_path, &bytes).map_err(|error| {
        PersistenceReplayCheckError::new("corrupt physics segment", error.to_string())
    })?;
    let mut manifest = latest.image.manifest;
    let descriptor = manifest.segments[segment_index].clone();
    manifest.segments[segment_index] = SaveSegmentDescriptor::for_bytes(
        descriptor.owner_id,
        descriptor.schema_id,
        descriptor.segment_id,
        descriptor.schema_version,
        &bytes,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("bind structural physics corruption", error.to_string())
    })?;
    let manifest_path = store
        .root()
        .join(format!("slot-{slot}"))
        .join("manifest.jcs");
    fs::write(
        &manifest_path,
        manifest.to_jcs_bytes().map_err(|error| {
            PersistenceReplayCheckError::new(
                "encode corrupt generation manifest",
                error.to_string(),
            )
        })?,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("write corrupt generation manifest", error.to_string())
    })?;
    Ok((segment_path, bytes))
}

fn segment_path(root: &Path, slot: u8, segment_index: usize) -> PathBuf {
    root.join(format!("slot-{slot}"))
        .join("segments")
        .join(format!("{segment_index:08}.necb"))
}

#[cfg(test)]
mod tests {
    use super::{
        CORE_DIALOGUE_ACCEPTED_NODE_ID, CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID,
        CORE_QUEST_ACTIVE_STATE_ID, run_persistence_replay_check,
    };

    #[test]
    fn product_check_covers_npc_transition_replay_and_structural_fallbacks() {
        let report = run_persistence_replay_check().expect("product check passes");
        assert_eq!(report.ticks, 14);
        assert_eq!(report.generations, 2);
        assert_eq!(report.rpg_events, 9);
        assert_eq!(
            report.interactive_object_state.as_str(),
            CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
        );
        assert_eq!(
            report.dialogue_node_id.as_str(),
            CORE_DIALOGUE_ACCEPTED_NODE_ID
        );
        assert_eq!(report.quest_state_id.as_str(), CORE_QUEST_ACTIVE_STATE_ID);
        assert_eq!(report.npc_player_trust, 7);
        assert_eq!(
            report.final_pose.translation_micrometres,
            [200_000, 900_000, 200_000]
        );
    }
}
