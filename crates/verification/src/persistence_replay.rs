use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::SaveStore;
use next_contracts::{
    AuthorityGrant, CharacterSnapshot, CommandLedgerHash, ContentHash, IssuerPrincipal,
    ItemSnapshot, PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID, PersistentId,
    PhysicsPoseV1, PhysicsWorldCheckpointV1, PlayerActionPhaseV1, RPG_SNAPSHOT_OWNER_ID,
    RPG_SNAPSHOT_SCHEMA_ID, RPG_SNAPSHOT_SCHEMA_VERSION, RPG_SNAPSHOT_SEGMENT_ID,
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SCHEMA_VERSION,
    RUNTIME_SNAPSHOT_SEGMENT_ID, ReplayComparePointV3, ReplayManifestV3, ReplayOwnerSegmentV2,
    ReplayTickManifestV3, RpgCommand, RpgSnapshot, SaveCompatibility, SaveSegmentDescriptor,
    SchemaId, StateRoot, TickSettings, WorldCheckpointV3, WorldCommand,
};
use next_runtime::{RuntimeState, TickReport};

use crate::{
    ReplayOutput, build_neutral_player_fixture, checkpoint_segment_hashes,
    compute_world_checkpoint_root, player_action_sample, replay_command_results,
    run_replay_manifest,
};

static NEXT_CHECK_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistenceReplayCheckReport {
    pub ticks: u64,
    pub generations: u64,
    pub final_pose: PhysicsPoseV1,
    pub final_state_root: StateRoot,
    pub final_command_ledger_hash: CommandLedgerHash,
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
    let fixture =
        build_neutral_player_fixture("nextengine.persistence-replay").map_err(|error| {
            PersistenceReplayCheckError::new("build player fixture", error.to_string())
        })?;
    let initial_rpg = initial_rpg_snapshot()?;
    let mut direct = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        initial_rpg,
    )
    .map_err(|error| PersistenceReplayCheckError::new("create runtime", error.to_string()))?;

    let first_input = player_action_sample(
        &fixture,
        0,
        PlayerActionPhaseV1::Started,
        [0, 32_767],
        Some(1_000),
    )
    .map_err(|error| PersistenceReplayCheckError::new("first input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, first_input)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue first input", error.to_string())
        })?;
    let initial_checkpoint = direct.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("initial checkpoint", error.to_string())
    })?;
    let direct_commands = rpg_commands(fixture.rpg_stream_id, fixture.principal.clone())?;
    let first = direct
        .run_tick(direct_commands.clone())
        .map_err(|error| PersistenceReplayCheckError::new("run tick zero", error.to_string()))?;

    let queued_input = player_action_sample(
        &fixture,
        1,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        Some(9_999_999),
    )
    .map_err(|error| PersistenceReplayCheckError::new("queued input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, queued_input)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue queued input", error.to_string())
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
    let mut restored =
        RuntimeState::restore_world_checkpoint(loaded.checkpoint, fixture.authority.clone())
            .map_err(|error| {
                PersistenceReplayCheckError::new("restore checkpoint", error.to_string())
            })?;

    let direct_second = direct
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct tick one", error.to_string()))?;
    let restored_second = restored.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("restored tick one", error.to_string())
    })?;
    if direct_second != restored_second {
        return Err(PersistenceReplayCheckError::condition(
            "queued input continues exactly after restore",
        ));
    }

    let stop = player_action_sample(&fixture, 2, PlayerActionPhaseV1::Completed, [0, 0], None)
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
    let direct_third = direct
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct tick two", error.to_string()))?;
    let restored_third = restored.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("restored tick two", error.to_string())
    })?;
    if direct_third != restored_third
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

    let reports = vec![first, direct_second, direct_third];
    let replay_manifest = replay_manifest(
        compatibility.clone(),
        &fixture.authority,
        initial_checkpoint,
        &reports,
        vec![direct_commands, Vec::new(), Vec::new()],
    )?;
    let replay = run_replay_manifest(&replay_manifest).map_err(|error| {
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
        corrupt_physics_segment(&store, &compatibility, generation_one.slot)?;
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
            "corrupt physics generation falls back without rewriting bytes",
        ));
    }

    let final_pose = final_checkpoint
        .physics_checkpoint
        .snapshot
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or_else(|| PersistenceReplayCheckError::condition("final capsule body exists"))?
        .pose;
    if final_pose.translation_micrometres != [0, 900_000, 200_000] {
        return Err(PersistenceReplayCheckError::condition(
            "queued movement applies exactly once",
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
        ticks: 3,
        generations: 2,
        final_pose,
        final_state_root,
        final_command_ledger_hash,
    })
}

fn replay_manifest(
    compatibility: SaveCompatibility,
    authority: &next_runtime::AuthorityRegistry,
    initial_checkpoint: WorldCheckpointV3,
    reports: &[TickReport],
    direct_commands: Vec<Vec<WorldCommand>>,
) -> Result<ReplayManifestV3, PersistenceReplayCheckError> {
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
        let checkpoint = WorldCheckpointV3::new(
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
        ticks.push(ReplayTickManifestV3 {
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
        compare_points.push(ReplayComparePointV3 {
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
    Ok(ReplayManifestV3 {
        schema_version: next_contracts::REPLAY_MANIFEST_V3_SCHEMA_VERSION,
        compatibility,
        initial_owner_segments,
        initial_state_root,
        authority,
        ticks,
        compare_points,
    })
}

fn owner_segments(
    checkpoint: &WorldCheckpointV3,
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
            RPG_SNAPSHOT_OWNER_ID,
            RPG_SNAPSHOT_SCHEMA_ID,
            RPG_SNAPSHOT_SEGMENT_ID,
            RPG_SNAPSHOT_SCHEMA_VERSION,
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
        let checkpoint = WorldCheckpointV3::new(
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

fn initial_rpg_snapshot() -> Result<RpgSnapshot, PersistenceReplayCheckError> {
    Ok(RpgSnapshot {
        characters: vec![
            CharacterSnapshot {
                id: PersistentId::from_bytes([0x20; 16]),
                revision: 0,
                archetype_id: SchemaId::new("rpg.character.persistence-owner").map_err(
                    |error| PersistenceReplayCheckError::new("owner archetype", error.to_string()),
                )?,
                skills: vec![],
                relationships: vec![],
            },
            CharacterSnapshot {
                id: PersistentId::from_bytes([0x30; 16]),
                revision: 0,
                archetype_id: SchemaId::new("rpg.character.persistence-recipient").map_err(
                    |error| {
                        PersistenceReplayCheckError::new("recipient archetype", error.to_string())
                    },
                )?,
                skills: vec![],
                relationships: vec![],
            },
        ],
        items: vec![ItemSnapshot {
            id: PersistentId::from_bytes([0x10; 16]),
            revision: 0,
            archetype_id: SchemaId::new("rpg.item.persistence-token").map_err(|error| {
                PersistenceReplayCheckError::new("item archetype", error.to_string())
            })?,
            owner: Some(PersistentId::from_bytes([0x20; 16])),
            quantity: 1,
        }],
        ..RpgSnapshot::default()
    })
}

fn rpg_commands(
    stream_id: next_contracts::CommandStreamId,
    principal: IssuerPrincipal,
) -> Result<Vec<WorldCommand>, PersistenceReplayCheckError> {
    let item_id = PersistentId::from_bytes([0x10; 16]);
    let first_owner = PersistentId::from_bytes([0x20; 16]);
    let second_owner = PersistentId::from_bytes([0x30; 16]);
    Ok(vec![
        WorldCommand::rpg(
            stream_id,
            principal.clone(),
            0,
            0,
            RpgCommand::TransferItem {
                item_id,
                expected_owner: Some(first_owner),
                new_owner: Some(second_owner),
            },
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("current RPG command", error.to_string())
        })?,
        WorldCommand::rpg(
            stream_id,
            principal,
            1,
            2,
            RpgCommand::TransferItem {
                item_id,
                expected_owner: Some(second_owner),
                new_owner: Some(first_owner),
            },
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
    let segment_path = segment_path(store.root(), slot, segment_index);
    let mut bytes = fs::read(&segment_path).map_err(|error| {
        PersistenceReplayCheckError::new("read physics segment", error.to_string())
    })?;
    let first = bytes
        .first_mut()
        .ok_or_else(|| PersistenceReplayCheckError::condition("physics segment is not empty"))?;
    *first ^= 0xff;
    fs::write(&segment_path, &bytes).map_err(|error| {
        PersistenceReplayCheckError::new("corrupt physics segment", error.to_string())
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
    use super::run_persistence_replay_check;

    #[test]
    fn product_check_covers_input_save_restore_replay_and_corrupt_physics_fallback() {
        let report = run_persistence_replay_check().expect("product check passes");
        assert_eq!(report.ticks, 3);
        assert_eq!(report.generations, 2);
        assert_eq!(
            report.final_pose.translation_micrometres,
            [0, 900_000, 200_000]
        );
    }
}
