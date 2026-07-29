mod extensions;
mod fault_injection;
mod replay_support;
mod rpg_fixture;
mod runner;

#[cfg(test)]
mod tests;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

use next_contracts::ids::{CommandLedgerHash, ContentHash, SchemaId, StateRoot};
use next_contracts::physics::PhysicsPoseV1;

use crate::scratch::{ScratchContext, ScratchDirectory};

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
    directory: ScratchDirectory,
}

impl CheckDirectory {
    fn new(scratch: &ScratchContext, label: &str) -> Result<Self, PersistenceReplayCheckError> {
        let directory = scratch.create_directory(label).map_err(|error| {
            PersistenceReplayCheckError::new("create product-check directory", error.to_string())
        })?;
        Ok(Self { directory })
    }

    fn path(&self) -> &Path {
        self.directory.path()
    }
}

pub fn run_persistence_replay_check()
-> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    run_persistence_replay_check_with_backend(PersistenceReplayBackend::Reference)
}

pub fn run_persistence_replay_check_with_backend(
    backend: PersistenceReplayBackend,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    run_persistence_replay_check_with_backend_in(backend, &std::env::temp_dir())
}

pub fn run_persistence_replay_check_with_backend_in(
    backend: PersistenceReplayBackend,
    scratch_root: &Path,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| PersistenceReplayCheckError::new("scratch root", error.to_string()))?;
    run_persistence_replay_check_with_backend_and_scratch(backend, &scratch)
}

pub(crate) fn run_persistence_replay_check_with_backend_and_scratch(
    backend: PersistenceReplayBackend,
    scratch: &ScratchContext,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    let directory = scratch
        .create_directory("persistence-replay")
        .map_err(|error| {
            PersistenceReplayCheckError::new("create persistence scratch", error.to_string())
        })?;
    let scoped = directory.context();
    match backend {
        PersistenceReplayBackend::Reference => {
            let result = runner::run_persistence_replay_check_for_project_with_scratch(
                &scoped,
                backend,
                "nextengine.persistence-replay",
                false,
            );
            directory.finish(result, |error| {
                PersistenceReplayCheckError::new("remove persistence scratch", error.to_string())
            })
        }
        PersistenceReplayBackend::PhysX => {
            let result = runner::run_persistence_replay_check_for_project_with_scratch(
                &scoped,
                backend,
                "nextengine.persistence-replay.physx",
                true,
            );
            directory.finish(result, |error| {
                PersistenceReplayCheckError::new("remove persistence scratch", error.to_string())
            })
        }
    }
}

pub(crate) fn run_persistence_replay_check_for_project(
    backend: PersistenceReplayBackend,
    project_id: &str,
    physx_compatible_profile: bool,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    let scratch = ScratchContext::new(&std::env::temp_dir())
        .map_err(|error| PersistenceReplayCheckError::new("scratch root", error.to_string()))?;
    let directory = scratch
        .create_directory("persistence-replay")
        .map_err(|error| {
            PersistenceReplayCheckError::new("create persistence scratch", error.to_string())
        })?;
    let result = runner::run_persistence_replay_check_for_project_with_scratch(
        &directory.context(),
        backend,
        project_id,
        physx_compatible_profile,
    );
    directory.finish(result, |error| {
        PersistenceReplayCheckError::new("remove persistence scratch", error.to_string())
    })
}
