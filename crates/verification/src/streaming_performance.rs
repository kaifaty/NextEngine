use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use next_assets::ContentStore;
use next_contracts::ids::{ContentHash, SchemaId};
use next_runtime::RuntimeState;
use next_world::WorldStreamerV1;

use crate::scratch::{ScratchContext, ScratchDirectory};

const STREAMING_PERFORMANCE_CYCLES: u64 = 1_000;
const STREAMING_PERFORMANCE_LIMIT: Duration = Duration::from_secs(30);
static NEXT_STREAMING_PREPARATION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingPerformanceReport {
    pub cycles: u64,
    pub staged_asset_references: u64,
    pub required_staging_bytes: u64,
    pub elapsed_microseconds: u128,
    pub within_report_only_limit: bool,
    pub final_generation: u64,
    pub final_world_state_hash: ContentHash,
}

/// A streaming workload whose project, content store, initial world, and
/// cleanup scope have already been prepared.
pub struct PreparedStreamingPerformanceCheck {
    preparation_id: u64,
    directory: ScratchDirectory,
    world: WorldStreamerV1,
    runtime: RuntimeState,
    route: Vec<SchemaId>,
    run_started: bool,
}

/// Opaque measured outcome consumed by
/// [`PreparedStreamingPerformanceCheck::finish`] after the caller closes its
/// external measurement window.
pub struct StreamingPerformanceMeasurement {
    preparation_id: u64,
    staged_asset_references: u64,
    required_staging_bytes: u64,
    elapsed: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingPerformanceError {
    context: &'static str,
    detail: String,
}

impl StreamingPerformanceError {
    fn new(context: &'static str, detail: impl Into<String>) -> Self {
        Self {
            context,
            detail: detail.into(),
        }
    }
}

impl Display for StreamingPerformanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for StreamingPerformanceError {}

pub fn run_streaming_performance_check()
-> Result<StreamingPerformanceReport, StreamingPerformanceError> {
    run_streaming_performance_check_in(&std::env::temp_dir())
}

pub fn run_streaming_performance_check_in(
    scratch_root: &Path,
) -> Result<StreamingPerformanceReport, StreamingPerformanceError> {
    let mut prepared = prepare_streaming_performance_check_in(scratch_root)?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

pub fn run_multiregion_streaming_performance_check()
-> Result<StreamingPerformanceReport, StreamingPerformanceError> {
    run_multiregion_streaming_performance_check_in(&std::env::temp_dir())
}

pub fn run_multiregion_streaming_performance_check_in(
    scratch_root: &Path,
) -> Result<StreamingPerformanceReport, StreamingPerformanceError> {
    let mut prepared = prepare_multiregion_streaming_performance_check_in(scratch_root)?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

pub(crate) fn run_streaming_performance_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<StreamingPerformanceReport, StreamingPerformanceError> {
    let mut prepared = prepare_streaming_performance_check_with_scratch(
        scratch,
        StreamingRouteKind::RolePair,
        "streaming-performance",
    )?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

pub fn prepare_streaming_performance_check()
-> Result<PreparedStreamingPerformanceCheck, StreamingPerformanceError> {
    prepare_streaming_performance_check_in(&std::env::temp_dir())
}

pub fn prepare_streaming_performance_check_in(
    scratch_root: &Path,
) -> Result<PreparedStreamingPerformanceCheck, StreamingPerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| StreamingPerformanceError::new("scratch root", error.to_string()))?;
    prepare_streaming_performance_check_with_scratch(
        &scratch,
        StreamingRouteKind::RolePair,
        "streaming-performance",
    )
}

pub fn prepare_multiregion_streaming_performance_check()
-> Result<PreparedStreamingPerformanceCheck, StreamingPerformanceError> {
    prepare_multiregion_streaming_performance_check_in(&std::env::temp_dir())
}

pub fn prepare_multiregion_streaming_performance_check_in(
    scratch_root: &Path,
) -> Result<PreparedStreamingPerformanceCheck, StreamingPerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| StreamingPerformanceError::new("scratch root", error.to_string()))?;
    prepare_streaming_performance_check_with_scratch(
        &scratch,
        StreamingRouteKind::Multiregion,
        "multiregion-streaming-performance",
    )
}

#[derive(Clone, Copy)]
enum StreamingRouteKind {
    RolePair,
    Multiregion,
}

fn prepare_streaming_performance_check_with_scratch(
    scratch: &ScratchContext,
    route_kind: StreamingRouteKind,
    directory_label: &str,
) -> Result<PreparedStreamingPerformanceCheck, StreamingPerformanceError> {
    let source = next_reference_game::project_source_v5()
        .map_err(|error| StreamingPerformanceError::new("fixture source", error.to_string()))?;
    let cooked = next_project::cook_project_v5(source)
        .map_err(|error| StreamingPerformanceError::new("cook fixture", error.to_string()))?;
    let directory = scratch.create_directory(directory_label).map_err(|error| {
        StreamingPerformanceError::new("create performance fixture", error.to_string())
    })?;
    let store = ContentStore::new(directory.path());
    let prepared = (|| {
        store
            .publish(&cooked.publication().map_err(|error| {
                StreamingPerformanceError::new("build publication", error.to_string())
            })?)
            .map_err(|error| {
                StreamingPerformanceError::new("publish fixture", error.to_string())
            })?;
        let package = next_project::activate_project_package(&store).map_err(|error| {
            StreamingPerformanceError::new("activate fixture", error.to_string())
        })?;
        let fixture = next_reference_game::build_reference_game_session(package.project.clone())
            .map_err(|error| {
                StreamingPerformanceError::new("runtime fixture", error.to_string())
            })?;
        let route = match route_kind {
            StreamingRouteKind::RolePair => vec![
                fixture.world_topology().initial_chunk_id().clone(),
                fixture.world_topology().gameplay_target_chunk_id().clone(),
            ],
            StreamingRouteKind::Multiregion => fixture
                .world_topology()
                .ordered_multiregion_route()
                .iter()
                .map(|entry| entry.chunk_id.clone())
                .collect(),
        };
        if route.len() < 2 {
            return Err(StreamingPerformanceError::new(
                "streaming route",
                "performance route requires at least two chunks",
            ));
        }
        let runtime = RuntimeState::new(fixture.bootstrap, fixture.authority).map_err(|error| {
            StreamingPerformanceError::new("activate runtime", error.to_string())
        })?;
        let world = WorldStreamerV1::activate(
            package.project,
            package.content_generation,
            route[0].clone(),
        )
        .map_err(|error| StreamingPerformanceError::new("activate world", error.to_string()))?;
        Ok((world, runtime, route))
    })();
    let (world, runtime, route) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => return Err(finish_failed_preparation(directory, error)),
    };
    let preparation_id = match next_streaming_preparation_id() {
        Ok(preparation_id) => preparation_id,
        Err(error) => return Err(finish_failed_preparation(directory, error)),
    };
    Ok(PreparedStreamingPerformanceCheck {
        preparation_id,
        directory,
        world,
        runtime,
        route,
        run_started: false,
    })
}

impl PreparedStreamingPerformanceCheck {
    /// Executes only the transition planning, staging, validation, and commit
    /// loop from the legacy measured kernel.
    pub fn run_measured(
        &mut self,
    ) -> Result<StreamingPerformanceMeasurement, StreamingPerformanceError> {
        if self.run_started {
            return Err(StreamingPerformanceError::new(
                "prepared streaming workload",
                "measured workload can run only once",
            ));
        }
        self.run_started = true;
        let started = Instant::now();
        let mut staged_asset_references = 0_u64;
        let mut required_staging_bytes = 0_u64;
        for cycle in 0..STREAMING_PERFORMANCE_CYCLES {
            let route_index = usize::try_from(cycle)
                .map_err(|error| StreamingPerformanceError::new("route index", error.to_string()))?
                .checked_add(1)
                .ok_or_else(|| {
                    StreamingPerformanceError::new("route index", "route index overflow")
                })?
                % self.route.len();
            let target = self.route[route_index].clone();
            let publication = self
                .world
                .prepare_begin_transition(target, self.runtime.next_tick())
                .map_err(|error| {
                    StreamingPerformanceError::new("plan transition", error.to_string())
                })?;
            let prepared = self
                .runtime
                .tick_preparation()
                .prepare_with_world_streaming([], &self.world, publication)
                .map_err(|error| {
                    StreamingPerformanceError::new("prepare requested tick", error.to_string())
                })?;
            let validated = self
                .runtime
                .validate_prepared_world_tick(&self.world, prepared)
                .map_err(|error| {
                    StreamingPerformanceError::new("validate requested tick", error.to_string())
                })?;
            let (_report, receipt) = self
                .runtime
                .commit_validated_world_tick(&mut self.world, validated);
            if receipt.is_some() {
                return Err(StreamingPerformanceError::new(
                    "requested publication",
                    "request unexpectedly completed",
                ));
            }

            let loaded = self
                .world
                .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
                .map_err(|error| {
                    StreamingPerformanceError::new("packaged load", error.to_string())
                })?;
            let metrics = loaded.metrics();
            staged_asset_references = staged_asset_references
                .checked_add(u64::try_from(metrics.asset_count).map_err(|error| {
                    StreamingPerformanceError::new("asset count", error.to_string())
                })?)
                .ok_or_else(|| {
                    StreamingPerformanceError::new("asset count", "staged asset count overflow")
                })?;
            required_staging_bytes =
                required_staging_bytes.max(u64::try_from(metrics.required_staging_bytes).map_err(
                    |error| StreamingPerformanceError::new("staging bytes", error.to_string()),
                )?);
            let publication = self
                .world
                .prepare_loaded_commit(loaded, self.runtime.next_tick())
                .map_err(|error| {
                    StreamingPerformanceError::new("prepare completion", error.to_string())
                })?;
            let prepared = self
                .runtime
                .tick_preparation()
                .prepare_with_world_streaming([], &self.world, publication)
                .map_err(|error| {
                    StreamingPerformanceError::new("prepare completion tick", error.to_string())
                })?;
            let validated = self
                .runtime
                .validate_prepared_world_tick(&self.world, prepared)
                .map_err(|error| {
                    StreamingPerformanceError::new("validate completion tick", error.to_string())
                })?;
            let (_report, receipt) = self
                .runtime
                .commit_validated_world_tick(&mut self.world, validated);
            if receipt.is_none() {
                return Err(StreamingPerformanceError::new(
                    "completion publication",
                    "completion receipt missing",
                ));
            }
        }
        Ok(StreamingPerformanceMeasurement {
            preparation_id: self.preparation_id,
            staged_asset_references,
            required_staging_bytes,
            elapsed: started.elapsed(),
        })
    }

    /// Validates and materializes the report after the external measurement
    /// window, then removes the prepared fixture on every result path.
    pub fn finish(
        self,
        measurement: Result<StreamingPerformanceMeasurement, StreamingPerformanceError>,
    ) -> Result<StreamingPerformanceReport, StreamingPerformanceError> {
        let result = measurement.and_then(|measurement| self.build_report(measurement));
        drop(self.runtime);
        drop(self.world);
        self.directory.finish(result, |error| {
            StreamingPerformanceError::new("remove performance fixture", error.to_string())
        })
    }

    /// Cancels a prepared workload before measurement and reports cleanup
    /// failures instead of relying on best-effort `Drop` cleanup.
    pub fn cancel(self) -> Result<(), StreamingPerformanceError> {
        drop(self.world);
        self.directory.finish(Ok(()), |error| {
            StreamingPerformanceError::new("remove performance fixture", error.to_string())
        })
    }

    fn build_report(
        &self,
        measurement: StreamingPerformanceMeasurement,
    ) -> Result<StreamingPerformanceReport, StreamingPerformanceError> {
        if !self.run_started || measurement.preparation_id != self.preparation_id {
            return Err(StreamingPerformanceError::new(
                "prepared streaming workload",
                "measurement does not belong to this completed preparation",
            ));
        }
        let final_generation = self.world.snapshot().generation;
        if final_generation != STREAMING_PERFORMANCE_CYCLES {
            return Err(StreamingPerformanceError::new(
                "streaming performance threshold",
                format!(
                    "elapsed={:?}, generation={final_generation}",
                    measurement.elapsed
                ),
            ));
        }
        Ok(StreamingPerformanceReport {
            cycles: STREAMING_PERFORMANCE_CYCLES,
            staged_asset_references: measurement.staged_asset_references,
            required_staging_bytes: measurement.required_staging_bytes,
            elapsed_microseconds: measurement.elapsed.as_micros(),
            within_report_only_limit: measurement.elapsed <= STREAMING_PERFORMANCE_LIMIT,
            final_generation,
            final_world_state_hash: self.world.snapshot().state_hash().map_err(|error| {
                StreamingPerformanceError::new("final world hash", error.to_string())
            })?,
        })
    }
}

fn next_streaming_preparation_id() -> Result<u64, StreamingPerformanceError> {
    NEXT_STREAMING_PREPARATION_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| {
            StreamingPerformanceError::new(
                "prepared streaming workload",
                "preparation identity overflow",
            )
        })
}

fn finish_failed_preparation(
    directory: ScratchDirectory,
    error: StreamingPerformanceError,
) -> StreamingPerformanceError {
    match directory.finish(Err::<(), _>(error), |cleanup_error| {
        StreamingPerformanceError::new("remove performance fixture", cleanup_error.to_string())
    }) {
        Err(error) => error,
        Ok(()) => unreachable!("an error result cannot become successful during cleanup"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        StreamingPerformanceError, prepare_streaming_performance_check_in,
        run_streaming_performance_check_in,
    };

    #[test]
    fn explicit_preparation_preserves_the_legacy_result() {
        let _measurement_guard = crate::test_support::lock_numeric_performance_measurement();
        let scratch = std::env::temp_dir();
        let legacy = run_streaming_performance_check_in(&scratch).expect("legacy workload");
        let mut prepared =
            prepare_streaming_performance_check_in(&scratch).expect("prepared workload");
        let measurement = prepared.run_measured();
        let explicit = prepared.finish(measurement).expect("finish workload");
        assert_eq!(explicit.cycles, legacy.cycles);
        assert_eq!(
            explicit.staged_asset_references,
            legacy.staged_asset_references
        );
        assert_eq!(explicit.final_generation, legacy.final_generation);
        assert_eq!(
            explicit.final_world_state_hash,
            legacy.final_world_state_hash
        );
    }

    #[test]
    fn prepared_workload_is_single_use_and_cleans_up() {
        let _measurement_guard = crate::test_support::lock_numeric_performance_measurement();
        let mut prepared = prepare_streaming_performance_check_in(&std::env::temp_dir())
            .expect("prepared workload");
        let path = prepared.directory.path().to_path_buf();
        let measurement = prepared.run_measured();
        let second = match prepared.run_measured() {
            Err(error) => error,
            Ok(_) => panic!("prepared workload must be single-use"),
        };
        assert!(second.to_string().contains("can run only once"));
        prepared.finish(measurement).expect("finish first run");
        assert!(!path.exists());
    }

    #[test]
    fn finish_rejects_foreign_measurement_and_cleans_both_preparations() {
        let _measurement_guard = crate::test_support::lock_numeric_performance_measurement();
        let mut source = prepare_streaming_performance_check_in(&std::env::temp_dir())
            .expect("source preparation");
        let source_path = source.directory.path().to_path_buf();
        let measurement = source.run_measured().expect("source measurement");
        let target = prepare_streaming_performance_check_in(&std::env::temp_dir())
            .expect("target preparation");
        let target_path = target.directory.path().to_path_buf();
        let error = target
            .finish(Ok(measurement))
            .expect_err("foreign measurement must fail");
        assert!(error.to_string().contains("does not belong"));
        assert!(!target_path.exists());
        let source_error = source
            .finish(injected_failure())
            .expect_err("injected failure");
        assert!(source_error.to_string().contains("injected caller failure"));
        assert!(!source_path.exists());
    }

    #[test]
    fn finish_preserves_a_workload_error_and_cleans_prepared_scratch() {
        let prepared = prepare_streaming_performance_check_in(&std::env::temp_dir())
            .expect("prepared workload");
        let path = prepared.directory.path().to_path_buf();
        let error = prepared
            .finish(injected_failure())
            .expect_err("injected failure");
        assert!(error.to_string().contains("injected caller failure"));
        assert!(!path.exists());
    }

    fn injected_failure()
    -> Result<super::StreamingPerformanceMeasurement, StreamingPerformanceError> {
        Err(StreamingPerformanceError::new(
            "measured workload",
            "injected caller failure",
        ))
    }
}
