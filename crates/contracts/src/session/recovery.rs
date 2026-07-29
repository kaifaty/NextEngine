use crate::ids::{ApplicationSessionId, CloseRequestId, ContentHash, SchemaId};
use crate::manifest_jcs::JcsValue;

use super::codec::{number, object, session_hash, string};
use super::{APPLICATION_SESSION_SCHEMA_VERSION, SessionContractError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PriorSessionDispositionV1 {
    RecoverySuperseded,
}

impl PriorSessionDispositionV1 {
    const fn token(self) -> &'static str {
        "RecoverySuperseded"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoverySessionLinkV1 {
    pub schema_version: u32,
    pub prior_session_id: ApplicationSessionId,
    pub prior_session_state_hash: ContentHash,
    pub prior_session_revision: u64,
    pub prior_close_request_id: CloseRequestId,
    pub canonical_close_request_hash: ContentHash,
    pub failed_final_save_ledger_entry_hash: ContentHash,
    pub project_composition_lock_hash: ContentHash,
    pub prior_application_session_manifest_hash: ContentHash,
    pub last_safe_save_generation_hash: ContentHash,
    pub last_safe_save_manifest_hash: ContentHash,
    pub new_session_id: ApplicationSessionId,
    pub prior_session_disposition: PriorSessionDispositionV1,
    pub recovery_reason: SchemaId,
    pub canonical_hash: ContentHash,
}

impl RecoverySessionLinkV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "recovery binds every prior failure, project and last-safe identity"
    )]
    pub fn new(
        prior_session_id: ApplicationSessionId,
        prior_session_state_hash: ContentHash,
        prior_session_revision: u64,
        prior_close_request_id: CloseRequestId,
        canonical_close_request_hash: ContentHash,
        failed_final_save_ledger_entry_hash: ContentHash,
        project_composition_lock_hash: ContentHash,
        prior_application_session_manifest_hash: ContentHash,
        last_safe_save_generation_hash: ContentHash,
        last_safe_save_manifest_hash: ContentHash,
        new_session_id: ApplicationSessionId,
        recovery_reason: SchemaId,
    ) -> Result<Self, SessionContractError> {
        if prior_session_id == new_session_id {
            return Err(SessionContractError::IdentityMismatch);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            prior_session_id,
            prior_session_state_hash,
            prior_session_revision,
            prior_close_request_id,
            canonical_close_request_hash,
            failed_final_save_ledger_entry_hash,
            project_composition_lock_hash,
            prior_application_session_manifest_hash,
            last_safe_save_generation_hash,
            last_safe_save_manifest_hash,
            new_session_id,
            prior_session_disposition: PriorSessionDispositionV1::RecoverySuperseded,
            recovery_reason,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash =
            session_hash("nextengine.recovery-session-link.v1", &value.body_value());
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        let canonical = Self::new(
            self.prior_session_id,
            self.prior_session_state_hash,
            self.prior_session_revision,
            self.prior_close_request_id,
            self.canonical_close_request_hash,
            self.failed_final_save_ledger_entry_hash,
            self.project_composition_lock_hash,
            self.prior_application_session_manifest_hash,
            self.last_safe_save_generation_hash,
            self.last_safe_save_manifest_hash,
            self.new_session_id,
            self.recovery_reason.clone(),
        )?;
        if canonical.canonical_hash != self.canonical_hash {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }

    fn body_value(&self) -> JcsValue {
        object([
            (
                "canonical_close_request_hash",
                string(self.canonical_close_request_hash.to_hex()),
            ),
            (
                "failed_final_save_ledger_entry_hash",
                string(self.failed_final_save_ledger_entry_hash.to_hex()),
            ),
            (
                "last_safe_save_generation_hash",
                string(self.last_safe_save_generation_hash.to_hex()),
            ),
            (
                "last_safe_save_manifest_hash",
                string(self.last_safe_save_manifest_hash.to_hex()),
            ),
            ("new_session_id", string(self.new_session_id.to_hex())),
            (
                "prior_application_session_manifest_hash",
                string(self.prior_application_session_manifest_hash.to_hex()),
            ),
            (
                "prior_close_request_id",
                string(self.prior_close_request_id.to_hex()),
            ),
            ("prior_session_id", string(self.prior_session_id.to_hex())),
            (
                "prior_session_disposition",
                string(self.prior_session_disposition.token()),
            ),
            (
                "prior_session_revision",
                number(self.prior_session_revision),
            ),
            (
                "prior_session_state_hash",
                string(self.prior_session_state_hash.to_hex()),
            ),
            (
                "project_composition_lock_hash",
                string(self.project_composition_lock_hash.to_hex()),
            ),
            ("recovery_reason", string(self.recovery_reason.as_str())),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
        ])
    }
}
