use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use next_assets::ContentStore;
use next_contracts::{ContentHash, SchemaId};
use next_world::WorldStreamerV1;

static PERFORMANCE_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);
const STREAMING_PERFORMANCE_CYCLES: u64 = 1_000;
const STREAMING_PERFORMANCE_LIMIT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingPerformanceReport {
    pub cycles: u64,
    pub staged_asset_references: u64,
    pub elapsed_microseconds: u128,
    pub final_generation: u64,
    pub final_world_state_hash: ContentHash,
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
    let source = next_project::neutral_vertical_slice_source_v1()
        .map_err(|error| StreamingPerformanceError::new("fixture source", error.to_string()))?;
    let cooked = next_project::cook_project_v1(source)
        .map_err(|error| StreamingPerformanceError::new("cook fixture", error.to_string()))?;
    let directory = std::env::temp_dir().join(format!(
        "nextengine-streaming-performance-{}-{}",
        std::process::id(),
        PERFORMANCE_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let store = ContentStore::new(&directory);
    let result = (|| {
        store
            .publish(&cooked.publication().map_err(|error| {
                StreamingPerformanceError::new("build publication", error.to_string())
            })?)
            .map_err(|error| {
                StreamingPerformanceError::new("publish fixture", error.to_string())
            })?;
        let project = next_project::activate_project(&store).map_err(|error| {
            StreamingPerformanceError::new("activate fixture", error.to_string())
        })?;
        let chunks = project
            .world_partition
            .body
            .chunk_bindings
            .iter()
            .map(|binding| binding.chunk_id.clone())
            .collect::<Vec<_>>();
        if chunks.len() != 2 {
            return Err(StreamingPerformanceError::new(
                "fixture topology",
                "exactly two chunks are required",
            ));
        }
        let mut world = WorldStreamerV1::activate(project, chunks[0].clone())
            .map_err(|error| StreamingPerformanceError::new("activate world", error.to_string()))?;
        let started = Instant::now();
        let mut staged_asset_references = 0_u64;
        for cycle in 0..STREAMING_PERFORMANCE_CYCLES {
            let target: SchemaId = if cycle % 2 == 0 {
                chunks[1].clone()
            } else {
                chunks[0].clone()
            };
            let plan = world.begin_transition(target, cycle).map_err(|error| {
                StreamingPerformanceError::new("plan transition", error.to_string())
            })?;
            staged_asset_references = staged_asset_references
                .checked_add(
                    u64::try_from(plan.ordered_required_asset_ids.len()).map_err(|error| {
                        StreamingPerformanceError::new("asset count", error.to_string())
                    })?,
                )
                .ok_or_else(|| {
                    StreamingPerformanceError::new("asset count", "staged asset count overflow")
                })?;
            let mut worker_order = plan.ordered_required_asset_ids.clone();
            if cycle % 3 != 0 {
                worker_order.reverse();
            }
            let staged = world.stage(&plan, &worker_order).map_err(|error| {
                StreamingPerformanceError::new("stage transition", error.to_string())
            })?;
            world.validate_staged(&staged).map_err(|error| {
                StreamingPerformanceError::new("validate transition", error.to_string())
            })?;
            world.commit(&staged, false).map_err(|error| {
                StreamingPerformanceError::new("commit transition", error.to_string())
            })?;
        }
        let elapsed = started.elapsed();
        if elapsed > STREAMING_PERFORMANCE_LIMIT
            || world.snapshot().generation != STREAMING_PERFORMANCE_CYCLES
        {
            return Err(StreamingPerformanceError::new(
                "streaming performance threshold",
                format!(
                    "elapsed={elapsed:?}, generation={}",
                    world.snapshot().generation
                ),
            ));
        }
        Ok(StreamingPerformanceReport {
            cycles: STREAMING_PERFORMANCE_CYCLES,
            staged_asset_references,
            elapsed_microseconds: elapsed.as_micros(),
            final_generation: world.snapshot().generation,
            final_world_state_hash: world.snapshot().state_hash().map_err(|error| {
                StreamingPerformanceError::new("final world hash", error.to_string())
            })?,
        })
    })();
    if directory.exists() {
        std::fs::remove_dir_all(&directory).map_err(|error| {
            StreamingPerformanceError::new("remove performance fixture", error.to_string())
        })?;
    }
    result
}
