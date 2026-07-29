use serde::{Deserialize, Serialize};

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
    pub windows: TargetGateDetailsV1,
    pub linux: TargetGateDetailsV1,
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
pub struct PerformanceDetailsV1 {
    pub streaming: StreamingPerformanceDetailsV1,
    pub agent_planning: AgentPerformanceDetailsV1,
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
