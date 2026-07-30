#![forbid(unsafe_code)]

mod agent_performance;
mod content_package;
mod live_runtime_performance;
mod persistence_replay;
mod physics_parity;
mod platform_check;
mod player_fixture;
mod render_performance;
mod replay;
mod runtime_fixture;
mod scratch;
mod state_root;
mod streaming_performance;
mod v1_closure;

pub use agent_performance::{
    AgentPlanningPerformanceError, AgentPlanningPerformanceReport,
    run_agent_planning_performance_check, run_agent_planning_performance_check_in,
};
pub use content_package::{
    ContentPackageCheckError, ContentPackageCheckReport, run_content_package_check,
    run_content_package_check_in,
};
pub use live_runtime_performance::{
    LiveRuntimePerformanceError, LiveRuntimePerformanceReport, run_live_runtime_performance_check,
    run_live_runtime_performance_check_in,
};
pub use persistence_replay::{
    PersistenceReplayBackend, PersistenceReplayCheckError, PersistenceReplayCheckReport,
    run_persistence_replay_check, run_persistence_replay_check_with_backend,
    run_persistence_replay_check_with_backend_in,
};
pub use physics_parity::{
    PhysicsBackendParityError, PhysicsBackendParityReport, run_physics_backend_parity_check,
};
pub use platform_check::{
    PlatformCandidateStatus, PlatformCheckError, PlatformCheckReport, run_platform_check,
    run_platform_check_in,
};
pub use player_fixture::{
    CanonicalFixtureError, GameCheckReport, NeutralPlayerFixture, PhysicsCollisionBackend,
    PhysicsCollisionCheckReport, PlayCheckError, PlayCheckReport, PreparedGameFrameV1,
    build_neutral_player_fixture, build_neutral_player_fixture_from_activated_project,
    build_physx_player_fixture, cooked_interaction_outcome, cooked_project_rpg_snapshot,
    player_action_sample, player_equip_use_sample, player_interact_sample, player_melee_sample,
    player_pickup_sample, prepare_game_frame, prepare_game_frame_in,
    prepare_game_frame_with_activated_project, run_game_check, run_physics_collision_check,
    run_physics_collision_check_with_backend, run_play_check, run_play_check_in,
    run_play_check_with_activated_project,
};
pub use render_performance::{
    RenderFramePlanningPerformanceError, RenderFramePlanningPerformanceReport,
    run_render_frame_planning_performance_check, run_render_frame_planning_performance_check_in,
};
pub use replay::{
    ReplayComparePointMismatch, ReplayError, ReplayInput, ReplayOutput, ReplayTickInput,
    ReplayTickRecord, RpgReplayInput, RpgReplayOutput, compare_replay_outputs,
    compute_world_checkpoint_root, run_replay, run_replay_manifest, run_replay_manifest_v5,
    run_rpg_replay, verify_replay,
};
pub use runtime_fixture::{
    NeutralFixtureError, NeutralRuntimeFixture, build_neutral_runtime_fixture,
};
pub use state_root::{StateRootError, StateSegment, compute_state_root};
pub use streaming_performance::{
    StreamingPerformanceError, StreamingPerformanceReport, run_streaming_performance_check,
    run_streaming_performance_check_in,
};
pub use v1_closure::{
    TargetGateStatusV1, V1ClosureCheckError, V1ClosureCheckReport, V1TargetGateV1,
    run_v1_closure_check, run_v1_closure_check_in,
};

pub(crate) use replay::{
    checkpoint_segment_hashes, replay_command_results,
    run_replay_manifest_v5_with_definitions_and_physics_options,
};
