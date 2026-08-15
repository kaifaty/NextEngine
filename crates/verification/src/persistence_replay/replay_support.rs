use next_contracts::command::WorldCommand;
use next_contracts::ids::SchemaId;
use next_contracts::persistence::{
    AuthorityGrant, ReplayComparePointV7, ReplayManifestV7, ReplayOwnerSegmentV2,
    ReplayTickManifestV7, SaveCompatibility, SaveSegmentDescriptor, WorldStreamingReplayInputV1,
    replay_physics_query_batch_hash, replay_physics_query_results_hash,
    replay_targeting_query_trace_hash,
};
use next_contracts::physics::{
    ContactPhaseV1, PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
};
use next_contracts::rpg::{
    RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION, RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
    RpgPhysicalContactFactV1,
};
use next_contracts::snapshot::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SCHEMA_VERSION,
    RUNTIME_SNAPSHOT_SEGMENT_ID, WorldCheckpointV4,
};
use next_contracts::world::{
    WORLD_STREAMING_SNAPSHOT_OWNER_ID, WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
    WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION, WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
    WorldStreamingSnapshotV1,
};
use next_contracts::world_population::{
    WORLD_POPULATION_SCHEMA_VERSION, WORLD_POPULATION_SNAPSHOT_OWNER_ID,
    WORLD_POPULATION_SNAPSHOT_SCHEMA_ID, WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
    WorldPopulationSnapshotV1,
};
use next_contracts::world_routine::{
    WORLD_ROUTINE_SCHEMA_VERSION, WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
    WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID, WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID, WorldRoutineSnapshotV1,
    interaction_availability_batch_hash,
};
use next_runtime::{RuntimeState, TickReport, WorldServicesTickCommitV1};

use crate::{ReplayOutput, replay_command_results};

use super::PersistenceReplayCheckError;

pub(super) fn rpg_contact_facts_from_report(
    batch: &next_contracts::physics::ClosedPhysicsContactBatchV1,
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

#[allow(
    clippy::too_many_arguments,
    reason = "the replay manifest constructor binds six initial owners plus the exact recorded tick streams"
)]
pub(super) fn replay_manifest(
    compatibility: SaveCompatibility,
    authority: &next_runtime::AuthorityRegistry,
    initial_checkpoint: &WorldCheckpointV4,
    initial_world_snapshot: &WorldStreamingSnapshotV1,
    initial_routine_snapshot_or_none: Option<&WorldRoutineSnapshotV1>,
    initial_population_snapshot: &WorldPopulationSnapshotV1,
    reports: &[TickReport],
    world_services_commits: &[WorldServicesTickCommitV1],
    streaming_inputs: &[WorldStreamingReplayInputV1],
    direct_commands: &[Vec<WorldCommand>],
) -> Result<ReplayManifestV7, PersistenceReplayCheckError> {
    if reports.len() != world_services_commits.len()
        || reports.len() != streaming_inputs.len()
        || reports.len() != direct_commands.len()
    {
        return Err(PersistenceReplayCheckError::condition(
            "recorded replay streams have exact common length",
        ));
    }
    let initial_state_root =
        next_contracts::snapshot::world_checkpoint_with_world_services_v1_state_root(
            &initial_checkpoint.runtime_snapshot,
            &initial_checkpoint.rpg_snapshot,
            &initial_checkpoint.physics_checkpoint,
            initial_world_snapshot,
            initial_routine_snapshot_or_none,
            Some(initial_population_snapshot),
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("initial replay root", error.to_string())
        })?;
    let initial_owner_segments = owner_segments(
        initial_checkpoint,
        initial_world_snapshot,
        initial_routine_snapshot_or_none,
        initial_population_snapshot,
    )?;
    let authority = authority
        .entries()
        .map(|(principal, capabilities)| AuthorityGrant {
            principal: principal.clone(),
            capabilities: capabilities.iter().cloned().collect(),
        })
        .collect();
    let mut ticks = Vec::with_capacity(reports.len());
    let mut compare_points = Vec::with_capacity(reports.len());
    let first_tick = initial_checkpoint.runtime_snapshot.next_tick;
    for (index, report) in reports.iter().enumerate() {
        let expected_tick = first_tick
            .checked_add(u64::try_from(index).map_err(|error| {
                PersistenceReplayCheckError::new("record replay tick", error.to_string())
            })?)
            .ok_or_else(|| {
                PersistenceReplayCheckError::new("record replay tick", "tick overflow")
            })?;
        let commit = &world_services_commits[index];
        if report.tick != expected_tick || commit.runtime_report != *report {
            return Err(PersistenceReplayCheckError::condition(
                "recorded replay ticks and joint commits are contiguous and exact",
            ));
        }
        let direct_external_commands = direct_commands[index]
            .iter()
            .map(next_contracts::persistence::ReplayCommandRecord::from_command)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                PersistenceReplayCheckError::new("record direct commands", error.to_string())
            })?;
        ticks.push(ReplayTickManifestV7 {
            tick: report.tick,
            world_streaming_input: streaming_inputs[index].clone(),
            closed_ingress_batch: report.closed_ingress_batch.clone(),
            direct_external_commands,
            expected_ingress_command_batch: report.command_batches[0].clone(),
            expected_physics_step_input: report.physics_step_input.clone(),
            expected_contact_batch: report.contact_batch.clone(),
            expected_targeting_intents: report.targeting_intents.clone(),
            expected_authoritative_targeting_queries: report
                .authoritative_targeting_queries
                .clone(),
            expected_physics_query_batch: report.physics_query_batch.clone(),
            expected_physics_query_results: report.physics_query_results.clone(),
            expected_outcome_command_batch: report.command_batches[1].clone(),
            expected_mapping_receipts: report.mapping_receipts.clone(),
            expected_interaction_availability: report.interaction_availability.clone(),
            expected_command_results: replay_command_results(&report.results),
            expected_events: report.events.clone(),
        });
        compare_points.push(ReplayComparePointV7 {
            tick: report.tick,
            state_root: commit.application_state_root,
            command_ledger_hash: report.snapshot.command_ledger_hash().map_err(|error| {
                PersistenceReplayCheckError::new("record ledger hash", error.to_string())
            })?,
            owner_segments: commit.application_owner_segments.clone(),
            closed_ingress_batch_hash: report.closed_ingress_batch.batch_hash,
            ingress_command_batch_hash: report.command_batches[0].batch_hash,
            physics_step_input_hash: report.physics_step_input.input_hash().map_err(|error| {
                PersistenceReplayCheckError::new("record physics input hash", error.to_string())
            })?,
            contact_batch_hash: report.contact_batch.batch_hash,
            physics_query_batch_hash: replay_physics_query_batch_hash(&report.physics_query_batch)
                .map_err(|error| {
                    PersistenceReplayCheckError::new(
                        "record physics query batch hash",
                        error.to_string(),
                    )
                })?,
            physics_query_results_hash: replay_physics_query_results_hash(
                &report.physics_query_results,
            )
            .map_err(|error| {
                PersistenceReplayCheckError::new(
                    "record physics query results hash",
                    error.to_string(),
                )
            })?,
            targeting_query_trace_hash: replay_targeting_query_trace_hash(
                &report.targeting_intents,
                &report.authoritative_targeting_queries,
            )
            .map_err(|error| {
                PersistenceReplayCheckError::new(
                    "record targeting query trace hash",
                    error.to_string(),
                )
            })?,
            outcome_command_batch_hash: report.command_batches[1].batch_hash,
            interaction_availability_hash: interaction_availability_batch_hash(
                &report.interaction_availability,
            )
            .map_err(|error| {
                PersistenceReplayCheckError::new(
                    "record interaction availability hash",
                    error.to_string(),
                )
            })?,
        });
    }
    Ok(ReplayManifestV7 {
        schema_version: next_contracts::persistence::REPLAY_MANIFEST_V7_SCHEMA_VERSION,
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
    world: &WorldStreamingSnapshotV1,
    routine_or_none: Option<&WorldRoutineSnapshotV1>,
    population: &WorldPopulationSnapshotV1,
) -> Result<Vec<ReplayOwnerSegmentV2>, PersistenceReplayCheckError> {
    let mut raw = vec![
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
        (
            WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION,
            world.canonical_bytes().map_err(|error| {
                PersistenceReplayCheckError::new("world streaming segment", error.to_string())
            })?,
        ),
    ];
    if let Some(routine) = routine_or_none {
        raw.push((
            WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
            u32::from(WORLD_ROUTINE_SCHEMA_VERSION),
            routine.canonical_bytes().map_err(|error| {
                PersistenceReplayCheckError::new("world routine segment", error.to_string())
            })?,
        ));
    }
    raw.push((
        WORLD_POPULATION_SNAPSHOT_OWNER_ID,
        WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
        WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
        u32::from(WORLD_POPULATION_SCHEMA_VERSION),
        population.canonical_bytes().map_err(|error| {
            PersistenceReplayCheckError::new("world population segment", error.to_string())
        })?,
    ));
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
    world_services_commits: &[WorldServicesTickCommitV1],
    replay: &ReplayOutput,
) -> Result<(), PersistenceReplayCheckError> {
    if reports.len() != replay.ticks.len() || reports.len() != world_services_commits.len() {
        return Err(PersistenceReplayCheckError::condition(
            "direct and replay tick counts match",
        ));
    }
    for ((report, commit), replay_tick) in reports
        .iter()
        .zip(world_services_commits)
        .zip(&replay.ticks)
    {
        if report.tick != replay_tick.tick
            || report.results != replay_tick.command_results
            || report.events != replay_tick.events
            || commit.application_state_root != replay_tick.state_root
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
