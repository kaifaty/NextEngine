use std::collections::BTreeMap;

use next_assets::{SaveStore, SessionStore};
use next_contracts::ids::{ApplicationSessionId, CommandLedgerHash, ContentHash};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::session::ApplicationSessionManifestV1;
use next_runtime::ApplicationSessionMachine;

use crate::LaunchRequestV1;
use crate::durable::DurableApplicationSnapshotV1;

mod activation;
mod close_flow;
mod identity;
mod platform_host;
mod publication;
mod recovery;
mod run;

use platform_host::RegisteredPlatformHostV1;
use run::PreparedRunV1;

const PROJECT_DIRECTORY: &str = "project";
const SESSION_DIRECTORY: &str = "sessions";
const SAVE_DIRECTORY: &str = "saves";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationRunOutcomeV1 {
    pub session_id: ApplicationSessionId,
    pub project_composition_lock_hash: ContentHash,
    pub ticks: u64,
    pub events: u64,
    pub rpg_events: u64,
    pub authoritative_revision: u64,
    pub authoritative_state_root: ContentHash,
    pub command_archive_root: ContentHash,
    pub command_identity_index_root: ContentHash,
    pub command_ledger_hash: CommandLedgerHash,
    pub presentation_input_count: u64,
    pub presentation_snapshot: Option<next_contracts::presentation::PresentationSnapshotV2>,
}

pub struct ApplicationCoordinator {
    launch: LaunchRequestV1,
    activated_project: ActivatedProjectV2,
    session_store: SessionStore,
    save_store: SaveStore,
    machine: ApplicationSessionMachine,
    durable: DurableApplicationSnapshotV1,
    current_generation: ContentHash,
    objects: BTreeMap<ContentHash, Vec<u8>>,
    prepared_run_objects: BTreeMap<ContentHash, Vec<u8>>,
    prepared_run: Option<PreparedRunV1>,
    live_run: Option<next_reference_game::ReferenceGameDriverV1>,
    platform_host: Option<RegisteredPlatformHostV1>,
    #[cfg(test)]
    pause_after_save_commit: bool,
    #[cfg(test)]
    fail_next_state_publication: bool,
}

impl ApplicationCoordinator {
    #[must_use]
    pub fn state(&self) -> &next_contracts::session::ApplicationSessionStateV1 {
        self.machine.state()
    }

    #[must_use]
    pub fn manifest(&self) -> &ApplicationSessionManifestV1 {
        self.machine.manifest()
    }

    #[must_use]
    pub fn activated_project(&self) -> &ActivatedProjectV2 {
        &self.activated_project
    }

    #[must_use]
    pub const fn current_session_generation(&self) -> ContentHash {
        self.current_generation
    }
}

#[cfg(test)]
mod tests;
