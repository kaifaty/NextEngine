mod extensions;
mod fault_injection;
mod replay_support;
mod rpg_fixture;
mod runner;

#[cfg(test)]
mod tests;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use next_contracts::ids::{CommandLedgerHash, ContentHash, SchemaId, StateRoot};
use next_contracts::physics::PhysicsPoseV1;

pub(crate) use runner::run_persistence_replay_check_for_project;

static NEXT_CHECK_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistenceReplayCheckReport {
    pub ticks: u64,
    pub generations: u64,
    pub final_pose: PhysicsPoseV1,
    pub rpg_events: u64,
    pub interactive_object_state: SchemaId,
    pub dialogue_node_id: SchemaId,
    pub quest_state_id: SchemaId,
    pub npc_player_trust: i32,
    pub npc_health: i32,
    pub player_health: i32,
    pub agent_intent_id: ContentHash,
    pub agent_projection_hash: ContentHash,
    pub luau_package_state_hash: ContentHash,
    pub wasm_plugin_state_hash: ContentHash,
    pub world_streaming_generation: u64,
    pub current_chunk_id: SchemaId,
    pub final_state_root: StateRoot,
    pub final_command_ledger_hash: CommandLedgerHash,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PersistenceReplayBackend {
    #[default]
    Reference,
    PhysX,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistenceReplayCheckError {
    context: &'static str,
    detail: String,
}

impl PersistenceReplayCheckError {
    fn new(context: &'static str, detail: impl Into<String>) -> Self {
        Self {
            context,
            detail: detail.into(),
        }
    }

    fn condition(context: &'static str) -> Self {
        Self::new(context, "acceptance condition was false")
    }
}

impl Display for PersistenceReplayCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for PersistenceReplayCheckError {}

struct CheckDirectory {
    path: PathBuf,
}

impl CheckDirectory {
    fn new() -> Result<Self, PersistenceReplayCheckError> {
        let sequence = NEXT_CHECK_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-persistence-replay-{}-{sequence}",
            std::process::id()
        ));
        match fs::remove_dir_all(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(PersistenceReplayCheckError::new(
                    "remove stale product-check directory",
                    error.to_string(),
                ));
            }
        }
        fs::create_dir_all(&path).map_err(|error| {
            PersistenceReplayCheckError::new("create product-check directory", error.to_string())
        })?;
        Ok(Self { path })
    }
}

impl Drop for CheckDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn run_persistence_replay_check()
-> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    run_persistence_replay_check_with_backend(PersistenceReplayBackend::Reference)
}

pub fn run_persistence_replay_check_with_backend(
    backend: PersistenceReplayBackend,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    match backend {
        PersistenceReplayBackend::Reference => run_persistence_replay_check_for_project(
            backend,
            "nextengine.persistence-replay",
            false,
        ),
        PersistenceReplayBackend::PhysX => run_persistence_replay_check_for_project(
            backend,
            "nextengine.persistence-replay.physx",
            true,
        ),
    }
}
