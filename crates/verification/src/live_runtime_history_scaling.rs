use std::path::Path;
use std::time::Instant;

use next_assets::ContentStore;
use next_contracts::ids::{CommandLedgerHash, ContentHash, StateRoot};
use next_contracts::ledger::command_identity_index_root;
use next_reference_game::{ReferenceGameDriverV1, ReferenceLiveStateV1};

use crate::live_runtime_performance::{LiveRuntimePerformanceError, movement_started_event};
use crate::scratch::ScratchContext;

/// Exact pre-commit history sizes around the retained-receipt capacity edge.
///
/// The sample at `4_095` measures the append that fills the retained window;
/// the sample at `4_096` measures the first append that must evict its oldest
/// receipt. The workload is diagnostic and has no timing verdict.
const RETAINED_RECEIPT_CAPACITY: u64 =
    next_contracts::ledger::COMMAND_RECEIPT_WINDOW_CAPACITY as u64;

pub const LIVE_RUNTIME_HISTORY_SCALING_BOUNDARIES: [u64; 3] =
    [0, RETAINED_RECEIPT_CAPACITY - 1, RETAINED_RECEIPT_CAPACITY];

const FINAL_HISTORY_SIZE: u64 = RETAINED_RECEIPT_CAPACITY + 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRuntimeHistoryScalingSample {
    pub history_size: u64,
    pub retained_receipt_count: u64,
    pub checkpoint_materialization_microseconds: u128,
    pub identity_index_root_probe_microseconds: u128,
    pub archive_root_probe_microseconds: u128,
    pub next_tick_prepare_microseconds: u128,
    pub next_tick_commit_microseconds: u128,
    pub authoritative_state_root: StateRoot,
    pub command_ledger_hash: CommandLedgerHash,
    pub command_archive_root: ContentHash,
    pub command_identity_index_root: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRuntimeHistoryScalingReport {
    pub samples: Vec<LiveRuntimeHistoryScalingSample>,
    pub final_history_size: u64,
    pub final_authoritative_state_root: StateRoot,
    pub final_command_ledger_hash: CommandLedgerHash,
    pub final_command_archive_root: ContentHash,
    pub final_command_identity_index_root: ContentHash,
    pub final_retained_receipt_count: u64,
    pub repeated_authoritative_root_parity: bool,
}

pub fn run_live_runtime_history_scaling_diagnostic()
-> Result<LiveRuntimeHistoryScalingReport, LiveRuntimePerformanceError> {
    run_live_runtime_history_scaling_diagnostic_in(&std::env::temp_dir())
}

pub fn run_live_runtime_history_scaling_diagnostic_in(
    scratch_root: &Path,
) -> Result<LiveRuntimeHistoryScalingReport, LiveRuntimePerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| LiveRuntimePerformanceError::new("scratch root", error.to_string()))?;
    let directory = scratch
        .create_directory("live-runtime-history-scaling")
        .map_err(|error| {
            LiveRuntimePerformanceError::new("create history-scaling fixture", error.to_string())
        })?;
    let store = ContentStore::new(directory.path());
    let result = (|| {
        let source = next_reference_game::project_source_v2().map_err(|error| {
            LiveRuntimePerformanceError::new("history-scaling fixture source", error.to_string())
        })?;
        let cooked = next_project::cook_project_v2(source).map_err(|error| {
            LiveRuntimePerformanceError::new("cook history-scaling fixture", error.to_string())
        })?;
        store
            .publish(&cooked.publication().map_err(|error| {
                LiveRuntimePerformanceError::new(
                    "build history-scaling publication",
                    error.to_string(),
                )
            })?)
            .map_err(|error| {
                LiveRuntimePerformanceError::new(
                    "publish history-scaling fixture",
                    error.to_string(),
                )
            })?;
        let project = next_project::activate_project(&store).map_err(|error| {
            LiveRuntimePerformanceError::new("activate history-scaling fixture", error.to_string())
        })?;
        let mut measured = ReferenceGameDriverV1::new(project.clone(), true).map_err(|error| {
            LiveRuntimePerformanceError::new(
                "create measured history-scaling driver",
                error.to_string(),
            )
        })?;
        let mut repeated = ReferenceGameDriverV1::new(project, true).map_err(|error| {
            LiveRuntimePerformanceError::new(
                "create repeated history-scaling driver",
                error.to_string(),
            )
        })?;
        let movement = movement_started_event()?;
        let mut samples = Vec::with_capacity(LIVE_RUNTIME_HISTORY_SCALING_BOUNDARIES.len());

        for history_size in 0..FINAL_HISTORY_SIZE {
            let measured_boundary = LIVE_RUNTIME_HISTORY_SCALING_BOUNDARIES.contains(&history_size);
            let mut sample = if measured_boundary {
                Some(capture_boundary_sample(&measured, &repeated, history_size)?)
            } else {
                None
            };
            let events = if history_size == 0 {
                std::slice::from_ref(&movement)
            } else {
                &[]
            };

            let prepare_started = sample.as_ref().map(|_| Instant::now());
            let prepared = measured.stage_advance(events).map_err(|error| {
                LiveRuntimePerformanceError::new(
                    "prepare measured history-scaling tick",
                    error.to_string(),
                )
            })?;
            let validated = measured
                .validate_prepared_advance(prepared)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new(
                        "validate measured history-scaling tick",
                        error.to_string(),
                    )
                })?;
            if let (Some(sample), Some(started)) = (&mut sample, prepare_started) {
                sample.next_tick_prepare_microseconds = started.elapsed().as_micros();
            }

            let commit_started = sample.as_ref().map(|_| Instant::now());
            measured
                .commit_validated_advance(validated)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new(
                        "commit measured history-scaling tick",
                        error.to_string(),
                    )
                })?;
            if let (Some(sample), Some(started)) = (&mut sample, commit_started) {
                sample.next_tick_commit_microseconds = started.elapsed().as_micros();
            }
            let _ = repeated.advance(events).map_err(|error| {
                LiveRuntimePerformanceError::new(
                    "advance repeated history-scaling tick",
                    error.to_string(),
                )
            })?;
            if let Some(sample) = sample {
                samples.push(sample);
            }
        }

        let measured_final = measured.state().map_err(|error| {
            LiveRuntimePerformanceError::new("measured final history state", error.to_string())
        })?;
        let repeated_final = repeated.state().map_err(|error| {
            LiveRuntimePerformanceError::new("repeated final history state", error.to_string())
        })?;
        let final_roots =
            compare_authoritative_roots(&measured_final, &repeated_final, FINAL_HISTORY_SIZE)?;
        validate_sample_boundaries(&samples)?;

        Ok(LiveRuntimeHistoryScalingReport {
            samples,
            final_history_size: FINAL_HISTORY_SIZE,
            final_authoritative_state_root: final_roots.authoritative_state_root,
            final_command_ledger_hash: final_roots.command_ledger_hash,
            final_command_archive_root: final_roots.command_archive_root,
            final_command_identity_index_root: final_roots.command_identity_index_root,
            final_retained_receipt_count: retained_receipt_count(
                &measured_final.checkpoint.runtime_snapshot,
            )?,
            repeated_authoritative_root_parity: true,
        })
    })();
    directory.finish(result, |error| {
        LiveRuntimePerformanceError::new("remove history-scaling fixture", error.to_string())
    })
}

fn capture_boundary_sample(
    measured: &ReferenceGameDriverV1,
    repeated: &ReferenceGameDriverV1,
    expected_history_size: u64,
) -> Result<LiveRuntimeHistoryScalingSample, LiveRuntimePerformanceError> {
    let checkpoint_started = Instant::now();
    let measured_state = measured.state().map_err(|error| {
        LiveRuntimePerformanceError::new("materialize measured history boundary", error.to_string())
    })?;
    let checkpoint_materialization_microseconds = checkpoint_started.elapsed().as_micros();
    let repeated_state = repeated.state().map_err(|error| {
        LiveRuntimePerformanceError::new("materialize repeated history boundary", error.to_string())
    })?;
    let roots =
        compare_authoritative_roots(&measured_state, &repeated_state, expected_history_size)?;
    let snapshot = &measured_state.checkpoint.runtime_snapshot;
    let retained_receipt_count = retained_receipt_count(snapshot)?;

    let identity_probe_started = Instant::now();
    let probed_identity_root = command_identity_index_root(
        &snapshot.command_ledger.identity_index.body,
    )
    .map_err(|error| {
        LiveRuntimePerformanceError::new("history identity-index root probe", error.to_string())
    })?;
    let identity_index_root_probe_microseconds = identity_probe_started.elapsed().as_micros();
    if probed_identity_root != roots.command_identity_index_root {
        return Err(LiveRuntimePerformanceError::new(
            "history identity-index root probe",
            "recomputed root differs from the published root",
        ));
    }

    let archive_probe_started = Instant::now();
    let probed_archive = snapshot.body_archive.manifest().map_err(|error| {
        LiveRuntimePerformanceError::new("history command-body archive probe", error.to_string())
    })?;
    let archive_root_probe_microseconds = archive_probe_started.elapsed().as_micros();
    if probed_archive != snapshot.command_ledger.body_archive {
        return Err(LiveRuntimePerformanceError::new(
            "history command-body archive probe",
            "recomputed manifest differs from the published manifest",
        ));
    }

    Ok(LiveRuntimeHistoryScalingSample {
        history_size: expected_history_size,
        retained_receipt_count,
        checkpoint_materialization_microseconds,
        identity_index_root_probe_microseconds,
        archive_root_probe_microseconds,
        next_tick_prepare_microseconds: 0,
        next_tick_commit_microseconds: 0,
        authoritative_state_root: roots.authoritative_state_root,
        command_ledger_hash: roots.command_ledger_hash,
        command_archive_root: roots.command_archive_root,
        command_identity_index_root: roots.command_identity_index_root,
    })
}

struct AuthoritativeRoots {
    authoritative_state_root: StateRoot,
    command_ledger_hash: CommandLedgerHash,
    command_archive_root: ContentHash,
    command_identity_index_root: ContentHash,
}

fn compare_authoritative_roots(
    measured: &ReferenceLiveStateV1,
    repeated: &ReferenceLiveStateV1,
    expected_history_size: u64,
) -> Result<AuthoritativeRoots, LiveRuntimePerformanceError> {
    let measured_snapshot = &measured.checkpoint.runtime_snapshot;
    let repeated_snapshot = &repeated.checkpoint.runtime_snapshot;
    let measured_history_size = u64::try_from(measured_snapshot.body_archive.entries().len())
        .map_err(|error| {
            LiveRuntimePerformanceError::new("measured history size", error.to_string())
        })?;
    let repeated_history_size = u64::try_from(repeated_snapshot.body_archive.entries().len())
        .map_err(|error| {
            LiveRuntimePerformanceError::new("repeated history size", error.to_string())
        })?;
    let measured_ledger_hash = measured_snapshot.command_ledger_hash().map_err(|error| {
        LiveRuntimePerformanceError::new("measured command-ledger hash", error.to_string())
    })?;
    let repeated_ledger_hash = repeated_snapshot.command_ledger_hash().map_err(|error| {
        LiveRuntimePerformanceError::new("repeated command-ledger hash", error.to_string())
    })?;
    let measured_archive_root = measured_snapshot.command_ledger.body_archive.archive_root;
    let repeated_archive_root = repeated_snapshot.command_ledger.body_archive.archive_root;
    let measured_identity_root = measured_snapshot.command_ledger.identity_index.index_root;
    let repeated_identity_root = repeated_snapshot.command_ledger.identity_index.index_root;
    validate_receipt_history_shape(measured_snapshot, expected_history_size, "measured")?;
    validate_receipt_history_shape(repeated_snapshot, expected_history_size, "repeated")?;
    if measured.ticks != expected_history_size
        || repeated.ticks != expected_history_size
        || measured_history_size != expected_history_size
        || repeated_history_size != expected_history_size
        || measured.checkpoint.state_root != repeated.checkpoint.state_root
        || measured_ledger_hash != repeated_ledger_hash
        || measured_archive_root != repeated_archive_root
        || measured_identity_root != repeated_identity_root
    {
        return Err(LiveRuntimePerformanceError::new(
            "history-scaling authoritative parity",
            format!(
                "expected_history={expected_history_size}, measured_ticks={}, repeated_ticks={}, measured_history={measured_history_size}, repeated_history={repeated_history_size}, measured_state={}, repeated_state={}, measured_ledger={}, repeated_ledger={}, measured_archive={}, repeated_archive={}, measured_identity={}, repeated_identity={}",
                measured.ticks,
                repeated.ticks,
                measured.checkpoint.state_root.to_hex(),
                repeated.checkpoint.state_root.to_hex(),
                measured_ledger_hash.to_hex(),
                repeated_ledger_hash.to_hex(),
                measured_archive_root.to_hex(),
                repeated_archive_root.to_hex(),
                measured_identity_root.to_hex(),
                repeated_identity_root.to_hex(),
            ),
        ));
    }
    Ok(AuthoritativeRoots {
        authoritative_state_root: measured.checkpoint.state_root,
        command_ledger_hash: measured_ledger_hash,
        command_archive_root: measured_archive_root,
        command_identity_index_root: measured_identity_root,
    })
}

fn validate_receipt_history_shape(
    snapshot: &next_contracts::snapshot::RuntimeSnapshotV3,
    expected_history_size: u64,
    label: &'static str,
) -> Result<(), LiveRuntimePerformanceError> {
    let mut active_stream_count = 0_u64;
    let mut maximum_finalized_count = 0_u64;
    let mut maximum_retained_count = 0_u64;
    for stream in snapshot.command_ledger.streams.values() {
        if stream.finalized_receipt_count != 0 {
            active_stream_count = active_stream_count.checked_add(1).ok_or_else(|| {
                LiveRuntimePerformanceError::new("history active stream count", "count overflow")
            })?;
        }
        maximum_finalized_count = maximum_finalized_count.max(stream.finalized_receipt_count);
        maximum_retained_count = maximum_retained_count.max(
            u64::try_from(stream.receipt_window.len()).map_err(|error| {
                LiveRuntimePerformanceError::new("history retained window size", error.to_string())
            })?,
        );
    }
    let expected_active_stream_count = u64::from(expected_history_size != 0);
    let expected_retained_count = expected_history_size.min(RETAINED_RECEIPT_CAPACITY);
    if active_stream_count != expected_active_stream_count
        || maximum_finalized_count != expected_history_size
        || maximum_retained_count != expected_retained_count
    {
        return Err(LiveRuntimePerformanceError::new(
            "history-scaling receipt-window shape",
            format!(
                "label={label}, expected_history={expected_history_size}, active_streams={active_stream_count}, maximum_finalized={maximum_finalized_count}, maximum_retained={maximum_retained_count}"
            ),
        ));
    }
    Ok(())
}

fn retained_receipt_count(
    snapshot: &next_contracts::snapshot::RuntimeSnapshotV3,
) -> Result<u64, LiveRuntimePerformanceError> {
    snapshot
        .command_ledger
        .streams
        .values()
        .try_fold(0_u64, |count, stream| {
            count
                .checked_add(u64::try_from(stream.receipt_window.len()).map_err(|error| {
                    LiveRuntimePerformanceError::new("retained receipt count", error.to_string())
                })?)
                .ok_or_else(|| {
                    LiveRuntimePerformanceError::new("retained receipt count", "count overflow")
                })
        })
}

fn validate_sample_boundaries(
    samples: &[LiveRuntimeHistoryScalingSample],
) -> Result<(), LiveRuntimePerformanceError> {
    let observed = samples
        .iter()
        .map(|sample| sample.history_size)
        .collect::<Vec<_>>();
    if observed != LIVE_RUNTIME_HISTORY_SCALING_BOUNDARIES {
        return Err(LiveRuntimePerformanceError::new(
            "history-scaling sample boundaries",
            format!("expected={LIVE_RUNTIME_HISTORY_SCALING_BOUNDARIES:?}, observed={observed:?}"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn scaling_boundaries_track_the_retained_receipt_window_capacity() {
        let capacity = u64::try_from(next_contracts::ledger::COMMAND_RECEIPT_WINDOW_CAPACITY)
            .expect("receipt window capacity");
        assert_eq!(
            super::LIVE_RUNTIME_HISTORY_SCALING_BOUNDARIES,
            [0, capacity - 1, capacity],
        );
    }

    #[test]
    #[ignore = "report-only 4,097-tick history-scaling diagnostic"]
    fn exact_receipt_window_boundary_preserves_authoritative_parity() {
        let report = super::run_live_runtime_history_scaling_diagnostic()
            .expect("history-scaling diagnostic");
        println!("{report:#?}");
        assert_eq!(
            report
                .samples
                .iter()
                .map(|sample| sample.history_size)
                .collect::<Vec<_>>(),
            super::LIVE_RUNTIME_HISTORY_SCALING_BOUNDARIES,
        );
        assert_eq!(
            report
                .samples
                .iter()
                .map(|sample| sample.retained_receipt_count)
                .collect::<Vec<_>>(),
            [0, 4_095, 4_096],
        );
        assert_eq!(
            report.final_history_size,
            super::RETAINED_RECEIPT_CAPACITY + 1,
        );
        assert_eq!(
            report.final_retained_receipt_count,
            super::RETAINED_RECEIPT_CAPACITY,
        );
        assert!(report.repeated_authoritative_root_parity);
        assert_ne!(
            report.final_authoritative_state_root,
            next_contracts::ids::StateRoot::default(),
        );
    }
}
