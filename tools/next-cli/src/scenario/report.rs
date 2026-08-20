use serde::{Deserialize, Serialize};

use crate::{CreatorProjectIdentityV1, CreatorRuntimeProofV1};

pub const CREATOR_RUNTIME_SCENARIO_FORMAT_V1: &str = "nextengine.creator-runtime-scenario.v1";
pub const CREATOR_SCENARIO_REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorRuntimeScenarioV1 {
    pub format: String,
    pub scenario_id: String,
    pub project: CreatorProjectIdentityV1,
    pub limits: CreatorScenarioLimitsV1,
    pub actions: Vec<CreatorScenarioActionV1>,
    pub assertions: Vec<CreatorScenarioAssertionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorScenarioLimitsV1 {
    pub tick_budget: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorScenarioActionV1 {
    pub action_id: String,
    pub kind: CreatorScenarioActionKindV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CreatorScenarioActionKindV1 {
    Tick,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "probe", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CreatorScenarioAssertionV1 {
    Ticks {
        assertion_id: String,
        expected: u64,
    },
    Events {
        assertion_id: String,
        expected: u64,
    },
    RpgEvents {
        assertion_id: String,
        expected: u64,
    },
    AuthoritativeRevision {
        assertion_id: String,
        expected: u64,
    },
    AuthoritativeStateRoot {
        assertion_id: String,
        expected: String,
    },
    CommandArchiveRoot {
        assertion_id: String,
        expected: String,
    },
    CommandIdentityIndexRoot {
        assertion_id: String,
        expected: String,
    },
    CommandLedgerHash {
        assertion_id: String,
        expected: String,
    },
    FinalSaveGenerationHash {
        assertion_id: String,
        expected: String,
    },
}

impl CreatorScenarioAssertionV1 {
    pub(super) fn assertion_id(&self) -> &str {
        match self {
            Self::Ticks { assertion_id, .. }
            | Self::Events { assertion_id, .. }
            | Self::RpgEvents { assertion_id, .. }
            | Self::AuthoritativeRevision { assertion_id, .. }
            | Self::AuthoritativeStateRoot { assertion_id, .. }
            | Self::CommandArchiveRoot { assertion_id, .. }
            | Self::CommandIdentityIndexRoot { assertion_id, .. }
            | Self::CommandLedgerHash { assertion_id, .. }
            | Self::FinalSaveGenerationHash { assertion_id, .. } => assertion_id,
        }
    }

    pub(super) const fn probe(&self) -> &'static str {
        match self {
            Self::Ticks { .. } => "ticks",
            Self::Events { .. } => "events",
            Self::RpgEvents { .. } => "rpg-events",
            Self::AuthoritativeRevision { .. } => "authoritative-revision",
            Self::AuthoritativeStateRoot { .. } => "authoritative-state-root",
            Self::CommandArchiveRoot { .. } => "command-archive-root",
            Self::CommandIdentityIndexRoot { .. } => "command-identity-index-root",
            Self::CommandLedgerHash { .. } => "command-ledger-hash",
            Self::FinalSaveGenerationHash { .. } => "final-save-generation-hash",
        }
    }

    pub(super) fn expected(&self) -> String {
        match self {
            Self::Ticks { expected, .. }
            | Self::Events { expected, .. }
            | Self::RpgEvents { expected, .. }
            | Self::AuthoritativeRevision { expected, .. } => expected.to_string(),
            Self::AuthoritativeStateRoot { expected, .. }
            | Self::CommandArchiveRoot { expected, .. }
            | Self::CommandIdentityIndexRoot { expected, .. }
            | Self::CommandLedgerHash { expected, .. }
            | Self::FinalSaveGenerationHash { expected, .. } => expected.clone(),
        }
    }

    pub(super) fn actual(&self, runtime: &CreatorRuntimeProofV1) -> String {
        match self {
            Self::Ticks { .. } => runtime.ticks.to_string(),
            Self::Events { .. } => runtime.events.to_string(),
            Self::RpgEvents { .. } => runtime.rpg_events.to_string(),
            Self::AuthoritativeRevision { .. } => runtime.authoritative_revision.to_string(),
            Self::AuthoritativeStateRoot { .. } => runtime.authoritative_state_root.clone(),
            Self::CommandArchiveRoot { .. } => runtime.command_archive_root.clone(),
            Self::CommandIdentityIndexRoot { .. } => runtime.command_identity_index_root.clone(),
            Self::CommandLedgerHash { .. } => runtime.command_ledger_hash.clone(),
            Self::FinalSaveGenerationHash { .. } => runtime.final_save_generation_hash.clone(),
        }
    }

    pub(super) fn expected_hash_or_none(&self) -> Option<&str> {
        match self {
            Self::AuthoritativeStateRoot { expected, .. }
            | Self::CommandArchiveRoot { expected, .. }
            | Self::CommandIdentityIndexRoot { expected, .. }
            | Self::CommandLedgerHash { expected, .. }
            | Self::FinalSaveGenerationHash { expected, .. } => Some(expected),
            Self::Ticks { .. }
            | Self::Events { .. }
            | Self::RpgEvents { .. }
            | Self::AuthoritativeRevision { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatorScenarioCommandReportV1 {
    Pass(Box<CreatorScenarioCommandPassReportV1>),
    Fail(Box<CreatorScenarioCommandFailureReportV1>),
}

impl CreatorScenarioCommandReportV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorScenarioCommandPassReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: CreatorScenarioDetailsV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CreatorScenarioDetailsV1 {
    Validate {
        scenario: CreatorScenarioIdentityV1,
        project: CreatorProjectIdentityV1,
        source: String,
        validation_state: String,
    },
    Run {
        scenario: CreatorScenarioIdentityV1,
        project: CreatorProjectIdentityV1,
        source: String,
        runtime: CreatorRuntimeProofV1,
        assertions_checked: u32,
    },
    Minimize {
        original_scenario: CreatorScenarioIdentityV1,
        minimized_scenario: CreatorScenarioIdentityV1,
        project: CreatorProjectIdentityV1,
        source: String,
        preserved_failure: CreatorScenarioFailureIdentityV1,
        minimization_status: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorScenarioIdentityV1 {
    pub format: String,
    pub scenario_id: String,
    pub scenario_sha256: String,
    pub project_lock_sha256: String,
    pub action_count: u32,
    pub assertion_count: u32,
    pub tick_budget: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorScenarioCommandFailureReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub diagnostic: CreatorScenarioDiagnosticV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorScenarioDiagnosticV1 {
    pub code: String,
    pub subsystem: String,
    pub message_key: String,
    pub scenario: Option<CreatorScenarioIdentityV1>,
    pub project: Option<CreatorProjectIdentityV1>,
    pub failure: Option<CreatorScenarioFailureIdentityV1>,
    pub minimization_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorScenarioFailureIdentityV1 {
    pub category: String,
    pub assertion_id: String,
    pub action_id: String,
    pub tick: u64,
    pub probe: String,
    pub expected: String,
    pub actual: String,
}
