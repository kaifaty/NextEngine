use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::{Duration, Instant};

use next_assets::ContentStore;
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, StateRoot};
use next_contracts::input::{KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_W_CONTROL_PATH_ID};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_reference_game::ReferenceGameDriverV1;

use crate::scratch::ScratchContext;

const LIVE_MOVEMENT_TICKS: u64 = 900;
const MEASUREMENT_WINDOW_TICKS: u64 = 300;
const CHECKPOINT_INTERVAL_TICKS: u64 = 30;
#[cfg(not(debug_assertions))]
const LIVE_MOVEMENT_LIMIT: Duration = Duration::from_secs(30);
#[cfg(debug_assertions)]
const LIVE_MOVEMENT_LIMIT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRuntimePerformanceReport {
    pub ticks: u64,
    pub command_body_count: u64,
    pub elapsed_microseconds: u128,
    pub window_microseconds: [u128; 3],
    pub checkpoint_microseconds: [u128; 3],
    pub final_state_root: StateRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRuntimePerformanceError {
    context: &'static str,
    detail: String,
}

impl LiveRuntimePerformanceError {
    fn new(context: &'static str, detail: impl Into<String>) -> Self {
        Self {
            context,
            detail: detail.into(),
        }
    }
}

impl Display for LiveRuntimePerformanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for LiveRuntimePerformanceError {}

pub fn run_live_runtime_performance_check()
-> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    run_live_runtime_performance_check_in(&std::env::temp_dir())
}

pub fn run_live_runtime_performance_check_in(
    scratch_root: &Path,
) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| LiveRuntimePerformanceError::new("scratch root", error.to_string()))?;
    run_live_runtime_performance_check_with_scratch(&scratch)
}

fn run_live_runtime_performance_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    let source = next_reference_game::project_source_v2()
        .map_err(|error| LiveRuntimePerformanceError::new("fixture source", error.to_string()))?;
    let cooked = next_project::cook_project_v1(source)
        .map_err(|error| LiveRuntimePerformanceError::new("cook fixture", error.to_string()))?;
    let directory = scratch
        .create_directory("live-runtime-performance")
        .map_err(|error| {
            LiveRuntimePerformanceError::new("create performance fixture", error.to_string())
        })?;
    let store = ContentStore::new(directory.path());
    let result = (|| {
        store
            .publish(&cooked.publication().map_err(|error| {
                LiveRuntimePerformanceError::new("build publication", error.to_string())
            })?)
            .map_err(|error| {
                LiveRuntimePerformanceError::new("publish fixture", error.to_string())
            })?;
        let project = next_project::activate_project(&store).map_err(|error| {
            LiveRuntimePerformanceError::new("activate fixture", error.to_string())
        })?;
        let mut driver = ReferenceGameDriverV1::new(project, true).map_err(|error| {
            LiveRuntimePerformanceError::new("create live driver", error.to_string())
        })?;
        let movement = movement_started_event()?;

        let started = Instant::now();
        let mut window_started = started;
        let mut window_microseconds = [0_u128; 3];
        let mut checkpoint_microseconds = [0_u128; 3];
        let mut checkpoint_root = StateRoot::default();
        for tick in 1..=LIVE_MOVEMENT_TICKS {
            let events = if tick == 1 {
                std::slice::from_ref(&movement)
            } else {
                &[]
            };
            driver.advance(events).map_err(|error| {
                LiveRuntimePerformanceError::new("advance live movement", error.to_string())
            })?;
            if tick.is_multiple_of(CHECKPOINT_INTERVAL_TICKS) {
                let checkpoint_started = Instant::now();
                checkpoint_root = driver
                    .state()
                    .map_err(|error| {
                        LiveRuntimePerformanceError::new("build live checkpoint", error.to_string())
                    })?
                    .checkpoint
                    .state_root;
                let window_index =
                    usize::try_from((tick - 1) / MEASUREMENT_WINDOW_TICKS).map_err(|error| {
                        LiveRuntimePerformanceError::new(
                            "checkpoint measurement window",
                            error.to_string(),
                        )
                    })?;
                checkpoint_microseconds[window_index] = checkpoint_microseconds[window_index]
                    .checked_add(checkpoint_started.elapsed().as_micros())
                    .ok_or_else(|| {
                        LiveRuntimePerformanceError::new(
                            "checkpoint measurement",
                            "elapsed microseconds overflow",
                        )
                    })?;
            }
            if tick.is_multiple_of(MEASUREMENT_WINDOW_TICKS) {
                let window_index = usize::try_from(
                    tick.checked_div(MEASUREMENT_WINDOW_TICKS)
                        .and_then(|value| value.checked_sub(1))
                        .ok_or_else(|| {
                            LiveRuntimePerformanceError::new(
                                "measurement window",
                                "window index underflow",
                            )
                        })?,
                )
                .map_err(|error| {
                    LiveRuntimePerformanceError::new("measurement window", error.to_string())
                })?;
                window_microseconds[window_index] = window_started.elapsed().as_micros();
                window_started = Instant::now();
            }
        }
        let elapsed = started.elapsed();
        let state = driver.state().map_err(|error| {
            LiveRuntimePerformanceError::new("final live state", error.to_string())
        })?;
        let command_body_count = u64::try_from(
            state
                .checkpoint
                .runtime_snapshot
                .body_archive
                .entries()
                .len(),
        )
        .map_err(|error| {
            LiveRuntimePerformanceError::new("command body count", error.to_string())
        })?;
        if state.ticks != LIVE_MOVEMENT_TICKS
            || command_body_count != LIVE_MOVEMENT_TICKS
            || checkpoint_root != state.checkpoint.state_root
        {
            return Err(LiveRuntimePerformanceError::new(
                "live runtime output",
                format!(
                    "ticks={}, command_bodies={}, checkpoint_root={}, final_root={}",
                    state.ticks,
                    command_body_count,
                    checkpoint_root.to_hex(),
                    state.checkpoint.state_root.to_hex()
                ),
            ));
        }
        if elapsed > LIVE_MOVEMENT_LIMIT {
            return Err(LiveRuntimePerformanceError::new(
                "live runtime performance threshold",
                format!(
                    "elapsed={elapsed:?}, windows_us={window_microseconds:?}, limit={LIVE_MOVEMENT_LIMIT:?}"
                ),
            ));
        }
        Ok(LiveRuntimePerformanceReport {
            ticks: state.ticks,
            command_body_count,
            elapsed_microseconds: elapsed.as_micros(),
            window_microseconds,
            checkpoint_microseconds,
            final_state_root: state.checkpoint.state_root,
        })
    })();
    directory.finish(result, |error| {
        LiveRuntimePerformanceError::new("remove performance fixture", error.to_string())
    })
}

fn movement_started_event() -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(KEYBOARD_DEVICE_CLASS_ID)
            .map_err(|error| LiveRuntimePerformanceError::new("device class", error.to_string()))?,
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(KEYBOARD_W_CONTROL_PATH_ID)
            .map_err(|error| LiveRuntimePerformanceError::new("control path", error.to_string()))?,
        NormalizedControlPhaseV1::Started,
        vec![i16::MAX],
        Vec::new(),
        0,
        0,
    )
    .map_err(|error| LiveRuntimePerformanceError::new("movement control", error.to_string()))?;
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.live-runtime-performance")
            .map_err(|error| LiveRuntimePerformanceError::new("source id", error.to_string()))?,
        0,
        0,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(sha256(
            b"nextengine.platform.live-runtime-performance-capabilities.v1",
        )),
    )
    .map_err(|error| LiveRuntimePerformanceError::new("movement event", error.to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn sustained_live_movement_remains_within_the_declared_runtime_budget() {
        let report =
            super::run_live_runtime_performance_check().expect("live runtime performance gate");
        println!("{report:?}");
        assert_eq!(report.ticks, 900);
        assert_eq!(report.command_body_count, 900);
        assert_ne!(
            report.final_state_root,
            next_contracts::ids::StateRoot::default()
        );
    }
}
