use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::Duration;

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, StateRoot};
use next_contracts::input::{
    KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_W_CONTROL_PATH_ID, MOUSE_DELTA_CONTROL_PATH_ID,
    MOUSE_DEVICE_CLASS_ID,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};

mod prepared;

pub use prepared::{
    LiveRuntimePerformanceMeasurement, PreparedLiveRuntimePerformanceCheck,
    prepare_live_runtime_long_session_performance_check,
    prepare_live_runtime_long_session_performance_check_in, prepare_live_runtime_performance_check,
    prepare_live_runtime_performance_check_in,
};

const LIVE_MOVEMENT_TICKS: u64 = 900;
const MEASUREMENT_WINDOW_TICKS: u64 = 300;
const LONG_SESSION_TICKS: u64 = 3_600;
const LONG_SESSION_WINDOW_TICKS: u64 = 1_200;
const LONG_SESSION_CAMERA_INTERVAL_TICKS: u64 = 15;
const STATE_SAMPLE_INTERVAL_TICKS: u64 = 30;
const APPLICATION_ONE_TICK_ELAPSED: Duration = Duration::from_nanos(33_333_334);
#[cfg(not(debug_assertions))]
const LIVE_MOVEMENT_LIMIT: Duration = Duration::from_secs(30);
#[cfg(debug_assertions)]
const LIVE_MOVEMENT_LIMIT: Duration = Duration::from_secs(60);
#[cfg(not(debug_assertions))]
const LONG_SESSION_LIMIT: Duration = Duration::from_secs(90);
#[cfg(debug_assertions)]
const LONG_SESSION_LIMIT: Duration = Duration::from_secs(240);

#[derive(Clone, Copy)]
struct LiveRuntimeWorkload {
    directory_label: &'static str,
    ticks: u64,
    window_ticks: u64,
    camera_interval_ticks: Option<u64>,
    time_limit: Duration,
}

const SMOKE_WORKLOAD: LiveRuntimeWorkload = LiveRuntimeWorkload {
    directory_label: "live-runtime-performance",
    ticks: LIVE_MOVEMENT_TICKS,
    window_ticks: MEASUREMENT_WINDOW_TICKS,
    camera_interval_ticks: None,
    time_limit: LIVE_MOVEMENT_LIMIT,
};

const LONG_SESSION_WORKLOAD: LiveRuntimeWorkload = LiveRuntimeWorkload {
    directory_label: "live-runtime-long-session-performance",
    ticks: LONG_SESSION_TICKS,
    window_ticks: LONG_SESSION_WINDOW_TICKS,
    camera_interval_ticks: Some(LONG_SESSION_CAMERA_INTERVAL_TICKS),
    time_limit: LONG_SESSION_LIMIT,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRuntimePerformanceReport {
    pub ticks: u64,
    pub command_body_count: u64,
    pub elapsed_microseconds: u128,
    pub window_microseconds: [u128; 3],
    pub checkpoint_microseconds: [u128; 3],
    pub driver_prepare_microseconds: Vec<u128>,
    pub driver_commit_microseconds: Vec<u128>,
    pub driver_checkpoint_materialization_microseconds: Vec<u128>,
    pub identity_index_root_probe_microseconds: [u128; 3],
    pub archive_root_probe_microseconds: [u128; 3],
    pub camera_event_count: u64,
    pub application_window_microseconds: [u128; 3],
    pub application_sample_interval_microseconds: [u128; 3],
    pub application_non_sample_tick_microseconds: Vec<u128>,
    pub application_sample_tick_microseconds: Vec<u128>,
    pub application_final_state_root: Option<ContentHash>,
    pub final_command_archive_root: ContentHash,
    pub final_command_identity_index_root: ContentHash,
    pub final_state_root: StateRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRuntimePerformanceError {
    context: &'static str,
    detail: String,
}

impl LiveRuntimePerformanceError {
    pub(super) fn new(context: &'static str, detail: impl Into<String>) -> Self {
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
    let mut prepared = prepare_live_runtime_performance_check_in(scratch_root)?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

pub fn run_live_runtime_long_session_performance_check()
-> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    run_live_runtime_long_session_performance_check_in(&std::env::temp_dir())
}

pub fn run_live_runtime_long_session_performance_check_in(
    scratch_root: &Path,
) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    let mut prepared = prepare_live_runtime_long_session_performance_check_in(scratch_root)?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

fn camera_changed_event(
    source_sequence: u64,
    sample_tick: u64,
) -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    let yaw = if source_sequence.is_multiple_of(2) {
        -12
    } else {
        12
    };
    let control = NormalizedControlEventV1::new(
        SchemaId::new(MOUSE_DEVICE_CLASS_ID)
            .map_err(|error| LiveRuntimePerformanceError::new("device class", error.to_string()))?,
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(MOUSE_DELTA_CONTROL_PATH_ID)
            .map_err(|error| LiveRuntimePerformanceError::new("control path", error.to_string()))?,
        NormalizedControlPhaseV1::Changed,
        vec![yaw, -7],
        Vec::new(),
        sample_tick,
        source_sequence,
    )
    .map_err(|error| LiveRuntimePerformanceError::new("camera control", error.to_string()))?;
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.live-runtime-performance")
            .map_err(|error| LiveRuntimePerformanceError::new("source id", error.to_string()))?,
        source_sequence,
        sample_tick,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(sha256(
            b"nextengine.platform.live-runtime-performance-capabilities.v1",
        )),
    )
    .map_err(|error| LiveRuntimePerformanceError::new("camera event", error.to_string()))
}

fn movement_started_event_for_host(
    host_instance_id: PersistentId,
    capability_set_hash: ContentHash,
) -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    control_event_for_host(
        host_instance_id,
        capability_set_hash,
        KEYBOARD_DEVICE_CLASS_ID,
        KEYBOARD_W_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
        vec![i16::MAX],
        0,
        0,
    )
}

fn camera_changed_event_for_host(
    host_instance_id: PersistentId,
    capability_set_hash: ContentHash,
    source_sequence: u64,
    sample_tick: u64,
) -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    let yaw = if source_sequence.is_multiple_of(2) {
        -12
    } else {
        12
    };
    control_event_for_host(
        host_instance_id,
        capability_set_hash,
        MOUSE_DEVICE_CLASS_ID,
        MOUSE_DELTA_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Changed,
        vec![yaw, -7],
        sample_tick,
        source_sequence,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the helper mirrors the complete normalized platform control identity"
)]
fn control_event_for_host(
    host_instance_id: PersistentId,
    capability_set_hash: ContentHash,
    device_class: &str,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
    sample_tick: u64,
    source_sequence: u64,
) -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(device_class)
            .map_err(|error| LiveRuntimePerformanceError::new("device class", error.to_string()))?,
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(control_path)
            .map_err(|error| LiveRuntimePerformanceError::new("control path", error.to_string()))?,
        phase,
        value,
        Vec::new(),
        sample_tick,
        source_sequence,
    )
    .map_err(|error| LiveRuntimePerformanceError::new("host control", error.to_string()))?;
    PlatformEventV1::new(
        host_instance_id,
        SchemaId::new("nextengine.platform.source.live-runtime-performance")
            .map_err(|error| LiveRuntimePerformanceError::new("source id", error.to_string()))?,
        source_sequence,
        sample_tick,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        capability_set_hash,
    )
    .map_err(|error| LiveRuntimePerformanceError::new("host control event", error.to_string()))
}

pub(super) fn movement_started_event() -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
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
        let _measurement_guard = crate::test_support::lock_numeric_performance_measurement();
        let report =
            super::run_live_runtime_performance_check().expect("live runtime performance gate");
        println!("{report:?}");
        assert_eq!(report.ticks, 900);
        assert_eq!(report.command_body_count, 900);
        assert_eq!(report.driver_prepare_microseconds.len(), 900);
        assert_eq!(report.driver_commit_microseconds.len(), 900);
        assert_eq!(
            report.driver_checkpoint_materialization_microseconds.len(),
            30
        );
        assert_ne!(
            report.final_state_root,
            next_contracts::ids::StateRoot::default()
        );
    }
}
