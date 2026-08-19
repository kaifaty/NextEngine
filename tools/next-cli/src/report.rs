use serde::{Deserialize, Serialize};

use next_project::CookedProjectV7;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatorCommandReportV1 {
    Pass(CreatorCommandPassReportV1),
    Fail(CreatorCommandFailureReportV1),
}

impl CreatorCommandReportV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass(_))
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorCommandPassReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: CreatorProjectDetailsV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorCommandFailureReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub diagnostic: CreatorDiagnosticV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorDiagnosticV1 {
    pub code: String,
    pub subsystem: String,
    pub message_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorProjectDetailsV1 {
    pub project_id: String,
    pub project_revision: u64,
    pub authoring_sha256: String,
    pub project_lock_sha256: String,
    pub schema_registry_sha256: String,
    pub content_manifest_sha256: String,
    pub world_partition_sha256: String,
    pub mechanics_lock_sha256: String,
    pub root_asset_count: u32,
    pub content_entry_count: u32,
    pub neutral_record_count: u32,
    pub render_asset_count: u32,
    pub world_chunk_count: u32,
    pub publication_file_count: u32,
    pub publication_state: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorProjectIdentityV1 {
    pub project_id: String,
    pub project_revision: u64,
    pub authoring_sha256: String,
    pub project_lock_sha256: String,
    pub schema_registry_sha256: String,
    pub content_manifest_sha256: String,
    pub world_partition_sha256: String,
    pub mechanics_lock_sha256: String,
}

pub(crate) fn project_identity_from_cooked(cooked: &CookedProjectV7) -> CreatorProjectIdentityV1 {
    CreatorProjectIdentityV1 {
        project_id: cooked.project_lock.project_id.as_str().to_owned(),
        project_revision: cooked.project_lock.project_revision,
        authoring_sha256: cooked.project_lock.authoring_sha256.to_hex(),
        project_lock_sha256: cooked.project_lock.project_lock_sha256.to_hex(),
        schema_registry_sha256: cooked
            .schema_registry
            .schema_registry_manifest_sha256
            .to_hex(),
        content_manifest_sha256: cooked.content_manifest.content_manifest_sha256.to_hex(),
        world_partition_sha256: cooked
            .world_partition
            .world_partition_manifest_sha256
            .to_hex(),
        mechanics_lock_sha256: cooked
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256
            .to_hex(),
    }
}

pub(crate) fn project_identity_from_activated(
    activated: &next_contracts::project::ActivatedProjectV8,
) -> CreatorProjectIdentityV1 {
    CreatorProjectIdentityV1 {
        project_id: activated.project_lock.project_id.as_str().to_owned(),
        project_revision: activated.project_lock.project_revision,
        authoring_sha256: activated.project_lock.authoring_sha256.to_hex(),
        project_lock_sha256: activated.project_lock.project_lock_sha256.to_hex(),
        schema_registry_sha256: activated
            .schema_registry
            .schema_registry_manifest_sha256
            .to_hex(),
        content_manifest_sha256: activated.content_manifest.content_manifest_sha256.to_hex(),
        world_partition_sha256: activated
            .world_partition
            .world_partition_manifest_sha256
            .to_hex(),
        mechanics_lock_sha256: activated
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256
            .to_hex(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorRuntimeProofV1 {
    pub status: String,
    pub composition_root: String,
    pub session_id: String,
    pub close_receipt_hash: String,
    pub final_save_generation_hash: String,
    pub ticks: u64,
    pub events: u64,
    pub rpg_events: u64,
    pub authoritative_revision: u64,
    pub authoritative_state_root: String,
    pub command_archive_root: String,
    pub command_identity_index_root: String,
    pub command_ledger_hash: String,
    pub project_composition_lock_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatorRunCommandReportV1 {
    Pass(Box<CreatorRunCommandPassReportV1>),
    Fail(CreatorCommandFailureReportV1),
}

impl CreatorRunCommandReportV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorRunCommandPassReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: CreatorRunDetailsV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorRunDetailsV1 {
    pub project: CreatorProjectIdentityV1,
    pub runtime: CreatorRuntimeProofV1,
    pub source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatorPackageCommandReportV1 {
    Pass(Box<CreatorPackageCommandPassReportV1>),
    Fail(CreatorCommandFailureReportV1),
}

impl CreatorPackageCommandReportV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorPackageCommandPassReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: CreatorPackageDetailsV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorPackageDetailsV1 {
    pub project: CreatorProjectIdentityV1,
    pub package_format: String,
    pub package_manifest_sha256: String,
    pub packaged_file_count: u32,
    pub packaged_size_bytes: u64,
    pub required_notices: Vec<String>,
    pub runtime: CreatorRuntimeProofV1,
}
