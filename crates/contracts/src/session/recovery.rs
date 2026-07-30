use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{ApplicationSessionId, CloseRequestId, ContentHash, SchemaId};
use crate::manifest_jcs::JcsValue;

use super::codec::{
    close_request_id, decoded_object, encoded, hash, number, object, reject_unknown, session_hash,
    session_id, string, take, text, u32_value, u64_value,
};
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

    #[must_use]
    pub fn to_jcs_bytes(&self) -> Vec<u8> {
        encoded(&self.body_value())
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SessionContractError> {
        let mut value = decoded_object(bytes, limits, "recovery_session_link")?;
        let schema_version = u32_value(take(&mut value, "schema_version")?, "schema_version")?;
        if schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        if text(
            take(&mut value, "prior_session_disposition")?,
            "prior_session_disposition",
        )? != "RecoverySuperseded"
        {
            return Err(SessionContractError::UnknownClosedValue);
        }
        let link = Self::new(
            session_id(take(&mut value, "prior_session_id")?, "prior_session_id")?,
            hash(
                take(&mut value, "prior_session_state_hash")?,
                "prior_session_state_hash",
            )?,
            u64_value(
                take(&mut value, "prior_session_revision")?,
                "prior_session_revision",
            )?,
            close_request_id(
                take(&mut value, "prior_close_request_id")?,
                "prior_close_request_id",
            )?,
            hash(
                take(&mut value, "canonical_close_request_hash")?,
                "canonical_close_request_hash",
            )?,
            hash(
                take(&mut value, "failed_final_save_ledger_entry_hash")?,
                "failed_final_save_ledger_entry_hash",
            )?,
            hash(
                take(&mut value, "project_composition_lock_hash")?,
                "project_composition_lock_hash",
            )?,
            hash(
                take(&mut value, "prior_application_session_manifest_hash")?,
                "prior_application_session_manifest_hash",
            )?,
            hash(
                take(&mut value, "last_safe_save_generation_hash")?,
                "last_safe_save_generation_hash",
            )?,
            hash(
                take(&mut value, "last_safe_save_manifest_hash")?,
                "last_safe_save_manifest_hash",
            )?,
            session_id(take(&mut value, "new_session_id")?, "new_session_id")?,
            SchemaId::new(text(
                take(&mut value, "recovery_reason")?,
                "recovery_reason",
            )?)?,
        )?;
        reject_unknown(value)?;
        if link.to_jcs_bytes() != bytes {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(link)
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
