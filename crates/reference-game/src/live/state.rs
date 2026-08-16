//! Live driver state structures: the durable live-state bundle and the exact
//! recovery evidence of the interactive reference driver. Kept out of
//! `live.rs` to respect the 1000-line source-file limit.

use next_contracts::cognition::{AgentCognitionSnapshotV1, AgentMemorySnapshotV1};
use next_contracts::ids::ContentHash;
use next_contracts::physical_animation::PhysicalAnimationSnapshotV1;
use next_contracts::presentation::PresentationSnapshotV2;
use next_contracts::snapshot::{WorldCheckpointCanonicalComponentsV1, WorldCheckpointV4};
use next_contracts::world::WorldStreamingSnapshotV1;
use next_contracts::world_activity::WorldActivitySnapshotV1;
use next_contracts::world_population::WorldPopulationSnapshotV1;
use next_contracts::world_routine::WorldRoutineSnapshotV1;

use crate::dialogue::ReferenceDialogueUiV1;
use crate::input::ReferenceUiScreenV1;

pub struct ReferenceLiveStateV2 {
    pub checkpoint: WorldCheckpointV4,
    pub checkpoint_canonical_components: WorldCheckpointCanonicalComponentsV1,
    pub world_streaming_snapshot: WorldStreamingSnapshotV1,
    pub world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    pub world_population_snapshot: WorldPopulationSnapshotV1,
    pub world_activity_snapshot: WorldActivitySnapshotV1,
    pub agent_cognition_snapshot: AgentCognitionSnapshotV1,
    pub agent_memory_snapshot: AgentMemorySnapshotV1,
    pub physical_animation_snapshot: PhysicalAnimationSnapshotV1,
    pub ticks: u64,
    pub events: u64,
    pub rpg_events: u64,
    pub project_composition_lock_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub presentation_input_count: u64,
    pub presentation_snapshot: PresentationSnapshotV2,
    pub driver_recovery: ReferenceLiveDriverRecoveryV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceLiveDriverRecoveryV1 {
    pub next_logical_frame_sequence: u64,
    pub events: u64,
    pub rpg_events: u64,
    pub camera_yaw_millidegrees: i32,
    pub camera_pitch_millidegrees: i32,
    pub camera_cut: bool,
    pub ui_screen: ReferenceUiScreenV1,
    pub dialogue: ReferenceDialogueUiV1,
    pub input_session_bytes: Vec<u8>,
    pub presentation_snapshot_bytes: Vec<u8>,
}
