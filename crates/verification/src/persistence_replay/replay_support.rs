use next_contracts::{
    AuthorityGrant, ContactPhaseV1, PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
    PhysicsWorldCheckpointV1, RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION, RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SCHEMA_VERSION,
    RUNTIME_SNAPSHOT_SEGMENT_ID, ReplayComparePointV4, ReplayManifestV4, ReplayOwnerSegmentV2,
    ReplayTickManifestV4, RpgPhysicalContactFactV1, SaveCompatibility, SaveSegmentDescriptor,
    SchemaId, WorldCheckpointV4, WorldCommand,
};
use next_runtime::{RuntimeState, TickReport};
use next_world::WorldStreamerV1;

use crate::{
    ReplayOutput, checkpoint_segment_hashes, compute_world_checkpoint_root, replay_command_results,
};

use super::PersistenceReplayCheckError;

pub(super) fn transition_world(
    world: &mut WorldStreamerV1,
    target_chunk_id: SchemaId,
    gameplay_tick: u64,
    context: &'static str,
) -> Result<(), PersistenceReplayCheckError> {
    let plan = world
        .begin_transition(target_chunk_id, gameplay_tick)
        .map_err(|error| PersistenceReplayCheckError::new(context, error.to_string()))?;
    let mut worker_order = plan.ordered_required_asset_ids.clone();
    worker_order.reverse();
    let staged = world
        .stage(&plan, &worker_order)
        .map_err(|error| PersistenceReplayCheckError::new(context, error.to_string()))?;
    world
        .validate_staged(&staged)
        .map_err(|error| PersistenceReplayCheckError::new(context, error.to_string()))?;
    world
        .commit(&staged, false)
        .map_err(|error| PersistenceReplayCheckError::new(context, error.to_string()))?;
    Ok(())
}

pub(super) fn rpg_contact_facts_from_report(
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

pub(super) fn replay_manifest(
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

pub(super) fn compare_replay(
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
