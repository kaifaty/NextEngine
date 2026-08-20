use serde::{Deserialize, Serialize};

use crate::{CreatorDiagnosticV1, CreatorProjectIdentityV1};

pub const CREATOR_REPLAY_REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatorReplayCommandReportV1 {
    Pass(Box<CreatorReplayCommandPassReportV1>),
    Fail(Box<CreatorReplayCommandFailureReportV1>),
}

impl CreatorReplayCommandReportV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorReplayCommandPassReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: CreatorReplayDetailsV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CreatorReplayDetailsV1 {
    Validate {
        replay: CreatorReplayIdentityV1,
        project: CreatorProjectIdentityV1,
        source: String,
        validation_state: String,
    },
    Inspect {
        replay: CreatorReplayIdentityV1,
        project: CreatorProjectIdentityV1,
        source: String,
        verification_state: String,
        projection: CreatorReplayDomainProjectionV1,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorReplayIdentityV1 {
    pub format: String,
    pub replay_sha256: String,
    pub project_id: String,
    pub initial_state_root: String,
    pub first_tick: u64,
    pub last_tick: u64,
    pub tick_count: u32,
    pub initial_owner_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "domain", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CreatorReplayDomainProjectionV1 {
    Runtime {
        tick: u64,
        state_root: String,
        command_ledger_hash: String,
        closed_ingress_batch_hash: String,
        ingress_command_batch_hash: String,
        outcome_command_batch_hash: String,
        direct_command_count: u32,
        command_result_count: u32,
        event_count: u32,
    },
    WorldServices {
        tick: u64,
        streaming_operation: String,
        target_chunk_id: Option<String>,
        mapping_receipt_count: u32,
        interaction_count: u32,
        interaction_availability_hash: String,
        targeting_query_count: u32,
        targeting_query_trace_hash: String,
    },
    Physics {
        tick: u64,
        physics_substeps: u32,
        accepted_intent_count: u32,
        contact_count: u32,
        query_count: u32,
        query_result_count: u32,
        physics_step_input_hash: String,
        contact_batch_hash: String,
        physics_query_batch_hash: String,
        physics_query_results_hash: String,
    },
    Owners {
        tick: u64,
        owners: Vec<CreatorReplayOwnerProjectionV1>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorReplayOwnerProjectionV1 {
    pub owner_id: String,
    pub schema_id: String,
    pub segment_id: String,
    pub schema_version: u32,
    pub byte_length: u64,
    pub content_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorReplayCommandFailureReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub diagnostic: CreatorReplayDiagnosticV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorReplayDiagnosticV1 {
    pub code: String,
    pub subsystem: String,
    pub message_key: String,
    pub replay: Option<CreatorReplayIdentityV1>,
    pub project: Option<CreatorProjectIdentityV1>,
    pub divergence: Option<CreatorReplayDivergenceV1>,
}

impl From<CreatorDiagnosticV1> for CreatorReplayDiagnosticV1 {
    fn from(value: CreatorDiagnosticV1) -> Self {
        Self {
            code: value.code,
            subsystem: value.subsystem,
            message_key: value.message_key,
            replay: None,
            project: None,
            divergence: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorReplayDivergenceV1 {
    pub first_divergent_tick: u64,
    pub stage: String,
    pub owner: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}
