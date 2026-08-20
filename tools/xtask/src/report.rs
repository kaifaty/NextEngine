use serde::{Deserialize, Serialize};

use crate::performance::PerformanceRunV6;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandReportV1<T> {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: T,
}

impl<T: Serialize> CommandReportV1<T> {
    pub fn new(command: &str, status: &str, details: T) -> Self {
        Self {
            schema_version: 1,
            status: status.to_owned(),
            command: command.to_owned(),
            details,
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|error| error.to_string())
    }

    pub fn emit_report(&self) -> Result<(), String> {
        println!("{}", self.to_json()?);
        Ok(())
    }

    pub fn emit(command: &str, status: &str, details: T) -> Result<(), String> {
        Self::new(command, status, details).emit_report()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandReportV2<T> {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: T,
}

impl<T: Serialize> CommandReportV2<T> {
    pub fn new(command: &str, status: &str, details: T) -> Self {
        Self {
            schema_version: 2,
            status: status.to_owned(),
            command: command.to_owned(),
            details,
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|error| error.to_string())
    }

    pub fn emit_report(&self) -> Result<(), String> {
        println!("{}", self.to_json()?);
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryScanDetailsV1 {
    pub checks: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostCheckDetailsV1 {
    pub host: String,
    pub rustc_release: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageDetailsV1 {
    pub target: String,
    pub output: String,
    pub package_manifest_hash: String,
    pub composition_lock_hash: String,
    pub game_binary_hash: String,
    pub headless_binary_hash: String,
    pub game_launch: String,
    pub headless_launch: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageDetailsV2 {
    pub target: String,
    pub output: String,
    pub package_manifest_hash: String,
    pub composition_lock_hash: String,
    pub game_binary_hash: String,
    pub headless_binary_hash: String,
    pub tool_binary_hash: String,
    pub game_launch: String,
    pub headless_launch: String,
    pub tool_launch: String,
    pub source_project: String,
    pub tool_authoring_hash: String,
    pub tool_neutral_record_count: u32,
    pub tool_publication_file_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetGateDetailsV1 {
    pub target: String,
    pub package_descriptor_hash: String,
    pub runtime_check: String,
    pub desktop_smoke: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct V1ClosureDetailsV1 {
    pub shipping_ready: bool,
    pub checks: Vec<String>,
    pub project_composition_lock_hash: String,
    pub schema_registry_hash: String,
    pub content_manifest_hash: String,
    pub mechanics_lock_hash: String,
    pub world_partition_hash: String,
    pub luau_manifest_hash: String,
    pub wasm_manifest_hash: String,
    pub wit_v2_hash: String,
    pub wit_v3_hash: String,
    pub extension_compatibility_hash: String,
    pub play_state_root: String,
    pub play_ledger_hash: String,
    pub replay_state_root: String,
    pub replay_ledger_hash: String,
    pub audio_scene_pcm_digest: String,
    pub windows: TargetGateDetailsV1,
    pub linux: TargetGateDetailsV1,
    pub closure_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct V1ClosureDetailsV2 {
    pub release_ready: bool,
    pub checks: Vec<String>,
    pub project_composition_lock_hash: String,
    pub schema_registry_hash: String,
    pub content_manifest_hash: String,
    pub mechanics_lock_hash: String,
    pub world_partition_hash: String,
    pub luau_manifest_hash: String,
    pub wasm_manifest_hash: String,
    pub wit_v2_hash: String,
    pub wit_v3_hash: String,
    pub extension_compatibility_hash: String,
    pub play_state_root: String,
    pub play_ledger_hash: String,
    pub replay_state_root: String,
    pub replay_ledger_hash: String,
    pub audio_scene_pcm_digest: String,
    pub release_target: TargetGateDetailsV1,
    pub closure_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamingPerformanceDetailsV1 {
    pub cycles: u64,
    pub staged_asset_references: u64,
    pub elapsed_microseconds: u128,
    pub final_generation: u64,
    pub final_world_state_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentPerformanceDetailsV1 {
    pub cycles: u64,
    pub elapsed_microseconds: u128,
    pub final_plan_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderPlanningPerformanceDetailsV1 {
    pub cycles: u64,
    pub elapsed_microseconds: u128,
    pub visible_object_count: u32,
    pub indexed_draw_count: u32,
    pub fallback_material_draw_count: u32,
    pub frame_plan_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveRuntimePerformanceDetailsV1 {
    pub ticks: u64,
    pub command_body_count: u64,
    pub elapsed_microseconds: u128,
    pub window_microseconds: [u128; 3],
    pub checkpoint_microseconds: [u128; 3],
    pub final_state_root: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionWorkerPerformanceDetailsV1 {
    pub diagnostic_schema_version: u32,
    pub diagnostic_methodology_version: String,
    pub callbacks: u64,
    pub callback_cadence_hz: u32,
    pub queue_capacity: u32,
    pub queue_high_water: u64,
    pub submitted_callbacks: u64,
    pub processed_callbacks: u64,
    pub fixed_steps: u64,
    pub ordinary_fixed_steps: u64,
    pub lifecycle_boundary_fixed_steps: u64,
    pub snapshot_publications: u64,
    pub snapshot_reads: u64,
    pub fresh_snapshot_reads: u64,
    pub dropped_callbacks: u64,
    pub reordered_callbacks: u64,
    pub final_snapshot_sequence: u64,
    pub final_simulation_tick: u64,
    pub authoritative_state_root: String,
    pub command_archive_root: String,
    pub command_identity_index_root: String,
    pub command_ledger_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4PopulationPerformanceDetailsV1 {
    pub npc_count: u32,
    pub active_count: u32,
    pub near_count: u32,
    pub background_count: u32,
    pub warmup_ticks: u64,
    pub measured_ticks: u64,
    pub active_due: u64,
    pub near_due: u64,
    pub background_due: u64,
    pub navigation_queries: u64,
    pub full_evaluation_due: u64,
    pub reduced_evaluation_due: u64,
    pub abstract_maintenance_due: u64,
    pub dormant_wake_due: u64,
    pub cognition_work_items: u64,
    pub maximum_queue_depth: u32,
    pub maximum_cognition_queue_depth: u32,
    pub deferred_work: u64,
    pub dropped_work: u64,
    pub maximum_starvation_age_ticks: u64,
    pub abstract_outcome_deferrals: u64,
    pub fabricated_outcomes: u64,
    pub navigation_p95_microseconds: u64,
    pub navigation_p99_microseconds: u64,
    pub cognition_dispatch_p95_microseconds: u64,
    pub cognition_dispatch_p99_microseconds: u64,
    pub world_services_p95_microseconds: u64,
    pub world_services_p99_microseconds: u64,
    pub elapsed_microseconds: u128,
    pub command_body_count: u64,
    pub final_population_snapshot_bytes: u64,
    pub due_trace_root: String,
    pub tier_cognition_trace_root: String,
    pub final_population_state_hash: String,
    pub final_activity_state_hash: String,
    pub final_agent_state_hash: String,
    pub final_memory_state_hash: String,
    pub final_application_state_root: String,
    pub final_command_ledger_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R5PhysicsWorkerPerformanceDetailsV1 {
    pub worker_count: u32,
    pub elapsed_microseconds: u64,
    pub aggregate_physics_substeps_per_second: u64,
    pub aggregate_motor_frames_per_second: u64,
    pub scaling_efficiency_basis_points: u64,
    pub motor_frame_p95_microseconds: u64,
    pub motor_frame_p99_microseconds: u64,
    pub authoritative_root: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R5PhysicsPerformanceDetailsV1 {
    pub evidence_run_count: u32,
    pub slot_count: u32,
    pub degrees_of_freedom_per_slot: u32,
    pub physics_hz: u32,
    pub motor_hz: u32,
    pub warmup_substeps_per_slot: u64,
    pub measured_substeps_per_slot: u64,
    pub measured_motor_frames_per_slot: u64,
    pub worker_runs: Vec<R5PhysicsWorkerPerformanceDetailsV1>,
    pub checkpoint_bytes_per_slot: Vec<u64>,
    pub restore_microseconds_per_slot: Vec<u64>,
    pub restore_p95_microseconds: u64,
    pub restore_p99_microseconds: u64,
    pub restore_wall_microseconds: u64,
    pub replay_prefix_substeps_per_slot: u64,
    pub replay_prefix_overhead_basis_points: u64,
    pub process_peak_working_set_bytes: u64,
    pub logical_host_bytes_per_slot: u64,
    pub authoritative_root: String,
    pub worker_root_parity: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceDetailsV1 {
    #[serde(default)]
    pub run: Option<PerformanceRunV6>,
    #[serde(default)]
    pub streaming: Option<StreamingPerformanceDetailsV1>,
    #[serde(default)]
    pub agent_planning: Option<AgentPerformanceDetailsV1>,
    #[serde(default)]
    pub render_planning: Option<RenderPlanningPerformanceDetailsV1>,
    #[serde(default)]
    pub live_runtime: Option<LiveRuntimePerformanceDetailsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub production_worker: Option<ProductionWorkerPerformanceDetailsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r4_100npc: Option<R4PopulationPerformanceDetailsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r5_physics: Option<R5PhysicsPerformanceDetailsV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformDetailsV1 {
    pub portable_contract: String,
    pub sdl_ash_candidate: String,
    pub normalized_events: usize,
    pub rendered_objects: u32,
    pub presentation_snapshot_hash: String,
    pub ledger_hash: String,
    pub state_root: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentPackageDetailsV1 {
    pub records: usize,
    pub chunks: usize,
    pub creator_records: usize,
    pub creator_chunks: usize,
    pub creator_composition_lock_hash: String,
    pub mechanic_packages: usize,
    pub luau_packages: usize,
    pub wasm_plugins: usize,
    pub combat_npc_health: i32,
    pub scripted_player_health: i32,
    pub wasm_player_health: i32,
    pub luau_state_hash: String,
    pub wasm_state_hash: String,
    pub wasm_host_api_major: u16,
    pub schema_registry_hash: String,
    pub content_manifest_hash: String,
    pub mechanics_lock_hash: String,
    pub world_partition_hash: String,
    pub composition_lock_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicsParityDetailsV1 {
    pub compared_substeps: u64,
    pub registration_permutations: u64,
    pub world_lifecycle_cycles: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicsCollisionDetailsV1 {
    pub gameplay_ticks: u64,
    pub physics_substeps: u64,
    pub begin_contacts: u64,
    pub persist_contacts: u64,
    pub end_contacts: u64,
    pub final_pose_um: [i64; 3],
    pub contact_batches_hash: String,
    pub physics_checkpoint_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalCharacterConformanceDetailsV1 {
    pub trip_contact_events: u64,
    pub trip_final_pose_um: [i64; 3],
    pub carry_contact_events: u64,
    pub carry_final_pose_um: [i64; 3],
    pub carried_load_pose_um: [i64; 3],
    pub capsule_clearance_um: i64,
    pub restored_contact_ticks: u64,
    pub melee_contact_events: u64,
    pub melee_npc_health: i32,
    pub repeated_run_identical: bool,
    pub final_state_root: String,
    pub final_command_ledger_hash: String,
    pub final_physics_checkpoint_hash: String,
    pub matrix_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootMotionConformanceDetailsV1 {
    pub cycles: u64,
    pub accepted_cycles: u64,
    pub rejected_cycles: u64,
    pub retried_cycles: u64,
    pub save_load_cycles: u64,
    pub lod_cycles: u64,
    pub canonical_proposal_round_trips: u64,
    pub motor_safety_decisions: u64,
    pub replayed_cycles: u64,
    pub full_motion_outcomes: u64,
    pub clipped_motion_outcomes: u64,
    pub blocked_motion_outcomes: u64,
    pub fault_no_mutation_outcomes: u64,
    pub lod_full_projection_probes: u64,
    pub lod_fallback_projection_probes: u64,
    pub final_pose_um: [i64; 3],
    pub final_physics_checkpoint_hash: String,
    pub final_command_ledger_hash: String,
    pub matrix_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationLodConformanceDetailsV1 {
    pub cycles: u64,
    pub full_pose_requests: u64,
    pub reduced_pose_requests: u64,
    pub held_pose_requests: u64,
    pub intent_only_requests: u64,
    pub culled_pose_requests: u64,
    pub sampled_pose_projections: u64,
    pub held_pose_projections: u64,
    pub bind_pose_projections: u64,
    pub no_pose_projections: u64,
    pub complete_snapshot_publications: u64,
    pub rejected_snapshot_publications: u64,
    pub due_intent_evaluations: u64,
    pub resource_fallbacks: u64,
    pub authoritative_isolation_checks: u64,
    pub renderer_frame_plans: u64,
    pub lod_profile_revision: String,
    pub final_state_root: String,
    pub final_command_ledger_hash: String,
    pub final_physics_checkpoint_hash: String,
    pub final_animation_snapshot_hash: String,
    pub final_presentation_snapshot_hash: String,
    pub final_frame_plan_hash: String,
    pub matrix_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceReplayDetailsV1 {
    pub ticks: u64,
    pub generations: u64,
    pub rpg_events: u64,
    pub interactive_object_state: String,
    pub dialogue_node: String,
    pub quest_state: String,
    pub npc_player_trust: i32,
    pub npc_health: i32,
    pub player_health: i32,
    pub agent_intent: String,
    pub agent_projection: String,
    pub luau_state_hash: String,
    pub wasm_state_hash: String,
    pub world_streaming_generation: u64,
    pub current_chunk: String,
    pub final_state_root: String,
    pub final_ledger_root: String,
}

#[cfg(test)]
mod tests {
    use super::PerformanceDetailsV1;

    #[test]
    fn performance_details_without_worker_preserve_legacy_json_shape() {
        let details = PerformanceDetailsV1 {
            run: None,
            streaming: None,
            agent_planning: None,
            render_planning: None,
            live_runtime: None,
            production_worker: None,
            r4_100npc: None,
            r5_physics: None,
        };

        let value = serde_json::to_value(&details).expect("serialize legacy performance details");
        assert_eq!(
            value,
            serde_json::json!({
                "run": null,
                "streaming": null,
                "agent_planning": null,
                "render_planning": null,
                "live_runtime": null,
            })
        );

        let decoded: PerformanceDetailsV1 =
            serde_json::from_value(value).expect("decode legacy performance details");
        assert_eq!(decoded, details);
    }
}
