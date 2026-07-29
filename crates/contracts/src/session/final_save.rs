use crate::ids::{ApplicationSessionId, CloseRequestId, ContentHash};
use crate::manifest_jcs::JcsValue;

use super::codec::{array, number, object, optional_hash, session_hash, string};
use super::{
    APPLICATION_SESSION_SCHEMA_VERSION, FailureDispositionV1, FinalSavePolicyV1,
    SessionContractError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalSaveReservationBodyV1 {
    pub schema_version: u32,
    pub session_id: ApplicationSessionId,
    pub close_request_id: CloseRequestId,
    pub canonical_close_request_hash: ContentHash,
    pub close_request_archive_ref: ContentHash,
    pub starting_session_revision: u64,
    pub final_save_policy: FinalSavePolicyV1,
    pub shutdown_policy_hash: ContentHash,
    pub maximum_attempts: u16,
}

impl FinalSaveReservationBodyV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the reservation identity binds every close/save authority input"
    )]
    pub fn new(
        session_id: ApplicationSessionId,
        close_request_id: CloseRequestId,
        canonical_close_request_hash: ContentHash,
        close_request_archive_ref: ContentHash,
        starting_session_revision: u64,
        final_save_policy: FinalSavePolicyV1,
        shutdown_policy_hash: ContentHash,
        maximum_attempts: u16,
    ) -> Result<Self, SessionContractError> {
        if maximum_attempts == 0 {
            return Err(SessionContractError::InvalidAttemptCount);
        }
        Ok(Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            session_id,
            close_request_id,
            canonical_close_request_hash,
            close_request_archive_ref,
            starting_session_revision,
            final_save_policy,
            shutdown_policy_hash,
            maximum_attempts,
        })
    }

    #[must_use]
    pub fn reservation_hash(&self) -> ContentHash {
        session_hash("nextengine.final-save-reservation.v1", &self.body_value())
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        if self.maximum_attempts == 0 {
            return Err(SessionContractError::InvalidAttemptCount);
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
                "close_request_archive_ref",
                string(self.close_request_archive_ref.to_hex()),
            ),
            ("close_request_id", string(self.close_request_id.to_hex())),
            ("final_save_policy", string(self.final_save_policy.token())),
            ("maximum_attempts", number(self.maximum_attempts)),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
            (
                "shutdown_policy_hash",
                string(self.shutdown_policy_hash.to_hex()),
            ),
            (
                "starting_session_revision",
                number(self.starting_session_revision),
            ),
        ])
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FinalSaveLedgerStatusV1 {
    Reserved = 1,
    RetryPending = 2,
    Committed = 3,
    Failed = 4,
}

impl FinalSaveLedgerStatusV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Reserved => "Reserved",
            Self::RetryPending => "RetryPending",
            Self::Committed => "Committed",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SessionFinalSaveLedgerEntryV1 {
    pub schema_version: u32,
    pub session_id: ApplicationSessionId,
    pub close_request_id: CloseRequestId,
    pub canonical_close_request_hash: ContentHash,
    pub close_request_archive_ref: ContentHash,
    pub reservation_hash: ContentHash,
    pub status: FinalSaveLedgerStatusV1,
    pub attempt_count: u16,
    pub maximum_attempts: u16,
    pub last_failure_code: Option<crate::ids::SchemaId>,
    pub save_generation_hash: Option<ContentHash>,
    pub final_save_receipt_hash: Option<ContentHash>,
    pub entry_hash: ContentHash,
}

impl SessionFinalSaveLedgerEntryV1 {
    pub fn reserved(
        reservation: &FinalSaveReservationBodyV1,
    ) -> Result<Self, SessionContractError> {
        reservation.validate()?;
        Ok(Self::new_unchecked(
            reservation,
            FinalSaveLedgerStatusV1::Reserved,
            0,
            None,
            None,
            None,
        ))
    }

    pub fn retry_pending(
        &self,
        failure_code: crate::ids::SchemaId,
    ) -> Result<Self, SessionContractError> {
        let attempt = self.next_attempt()?;
        if attempt >= self.maximum_attempts {
            return Err(SessionContractError::InvalidAttemptCount);
        }
        Ok(self.with_outcome(
            FinalSaveLedgerStatusV1::RetryPending,
            attempt,
            Some(failure_code),
            None,
            None,
        ))
    }

    pub fn failed(&self, failure_code: crate::ids::SchemaId) -> Result<Self, SessionContractError> {
        let attempt = self.next_attempt()?;
        Ok(self.with_outcome(
            FinalSaveLedgerStatusV1::Failed,
            attempt,
            Some(failure_code),
            None,
            None,
        ))
    }

    pub fn committed(&self, receipt: &FinalSaveReceiptV1) -> Result<Self, SessionContractError> {
        let attempt = self.next_attempt()?;
        receipt.validate()?;
        if receipt.session_id != self.session_id
            || receipt.close_request_id != self.close_request_id
            || receipt.canonical_close_request_hash != self.canonical_close_request_hash
            || receipt.reservation_hash != self.reservation_hash
            || receipt.attempt_ordinal != attempt
        {
            return Err(SessionContractError::InvalidLedgerEntry);
        }
        Ok(self.with_outcome(
            FinalSaveLedgerStatusV1::Committed,
            attempt,
            None,
            Some(receipt.save_generation_hash),
            Some(receipt.canonical_hash),
        ))
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION
            || self.maximum_attempts == 0
            || self.attempt_count > self.maximum_attempts
        {
            return Err(SessionContractError::InvalidAttemptCount);
        }
        let fields_valid = match self.status {
            FinalSaveLedgerStatusV1::Reserved => {
                self.attempt_count == 0
                    && self.last_failure_code.is_none()
                    && self.save_generation_hash.is_none()
                    && self.final_save_receipt_hash.is_none()
            }
            FinalSaveLedgerStatusV1::RetryPending => {
                self.attempt_count > 0
                    && self.attempt_count < self.maximum_attempts
                    && self.last_failure_code.is_some()
                    && self.save_generation_hash.is_none()
                    && self.final_save_receipt_hash.is_none()
            }
            FinalSaveLedgerStatusV1::Committed => {
                self.attempt_count > 0
                    && self.last_failure_code.is_none()
                    && self.save_generation_hash.is_some()
                    && self.final_save_receipt_hash.is_some()
            }
            FinalSaveLedgerStatusV1::Failed => {
                self.attempt_count > 0
                    && self.last_failure_code.is_some()
                    && self.save_generation_hash.is_none()
                    && self.final_save_receipt_hash.is_none()
            }
        };
        if !fields_valid || self.computed_hash() != self.entry_hash {
            return Err(SessionContractError::InvalidLedgerEntry);
        }
        Ok(())
    }

    fn next_attempt(&self) -> Result<u16, SessionContractError> {
        self.validate()?;
        if matches!(
            self.status,
            FinalSaveLedgerStatusV1::Committed | FinalSaveLedgerStatusV1::Failed
        ) {
            return Err(SessionContractError::InvalidLedgerEntry);
        }
        let attempt = self
            .attempt_count
            .checked_add(1)
            .ok_or(SessionContractError::InvalidAttemptCount)?;
        if attempt > self.maximum_attempts {
            return Err(SessionContractError::InvalidAttemptCount);
        }
        Ok(attempt)
    }

    fn with_outcome(
        &self,
        status: FinalSaveLedgerStatusV1,
        attempt_count: u16,
        last_failure_code: Option<crate::ids::SchemaId>,
        save_generation_hash: Option<ContentHash>,
        final_save_receipt_hash: Option<ContentHash>,
    ) -> Self {
        let mut value = self.clone();
        value.status = status;
        value.attempt_count = attempt_count;
        value.last_failure_code = last_failure_code;
        value.save_generation_hash = save_generation_hash;
        value.final_save_receipt_hash = final_save_receipt_hash;
        value.entry_hash = value.computed_hash();
        value
    }

    fn new_unchecked(
        reservation: &FinalSaveReservationBodyV1,
        status: FinalSaveLedgerStatusV1,
        attempt_count: u16,
        last_failure_code: Option<crate::ids::SchemaId>,
        save_generation_hash: Option<ContentHash>,
        final_save_receipt_hash: Option<ContentHash>,
    ) -> Self {
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            session_id: reservation.session_id,
            close_request_id: reservation.close_request_id,
            canonical_close_request_hash: reservation.canonical_close_request_hash,
            close_request_archive_ref: reservation.close_request_archive_ref,
            reservation_hash: reservation.reservation_hash(),
            status,
            attempt_count,
            maximum_attempts: reservation.maximum_attempts,
            last_failure_code,
            save_generation_hash,
            final_save_receipt_hash,
            entry_hash: ContentHash::default(),
        };
        value.entry_hash = value.computed_hash();
        value
    }

    fn computed_hash(&self) -> ContentHash {
        session_hash(
            "nextengine.session-final-save-ledger-entry.v1",
            &self.body_value(),
        )
    }

    fn body_value(&self) -> JcsValue {
        object([
            ("attempt_count", number(self.attempt_count)),
            (
                "canonical_close_request_hash",
                string(self.canonical_close_request_hash.to_hex()),
            ),
            (
                "close_request_archive_ref",
                string(self.close_request_archive_ref.to_hex()),
            ),
            ("close_request_id", string(self.close_request_id.to_hex())),
            (
                "final_save_receipt_hash_or_none",
                optional_hash(self.final_save_receipt_hash),
            ),
            (
                "last_failure_code_or_none",
                self.last_failure_code
                    .as_ref()
                    .map_or_else(|| string("none"), |value| string(value.as_str())),
            ),
            ("maximum_attempts", number(self.maximum_attempts)),
            ("reservation_hash", string(self.reservation_hash.to_hex())),
            (
                "save_generation_hash_or_none",
                optional_hash(self.save_generation_hash),
            ),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
            ("status", string(self.status.token())),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionFinalSaveLedgerV1 {
    pub schema_version: u32,
    pub entries: Vec<SessionFinalSaveLedgerEntryV1>,
    pub canonical_hash: ContentHash,
}

impl SessionFinalSaveLedgerV1 {
    pub fn new(
        mut entries: Vec<SessionFinalSaveLedgerEntryV1>,
    ) -> Result<Self, SessionContractError> {
        for entry in &entries {
            entry.validate()?;
        }
        entries.sort_by_key(|entry| (entry.session_id, entry.close_request_id));
        if entries.windows(2).any(|pair| {
            (pair[0].session_id, pair[0].close_request_id)
                == (pair[1].session_id, pair[1].close_request_id)
        }) {
            return Err(SessionContractError::DuplicateIdentity);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            entries,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash(
            "nextengine.session-final-save-ledger.v1",
            &value.body_value(),
        );
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        if Self::new(self.entries.clone())?.canonical_hash != self.canonical_hash {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }

    fn body_value(&self) -> JcsValue {
        object([
            (
                "ordered_entries",
                array(
                    self.entries
                        .iter()
                        .map(SessionFinalSaveLedgerEntryV1::body_value)
                        .collect(),
                ),
            ),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalSaveReceiptV1 {
    pub schema_version: u32,
    pub session_id: ApplicationSessionId,
    pub close_request_id: CloseRequestId,
    pub canonical_close_request_hash: ContentHash,
    pub reservation_hash: ContentHash,
    pub attempt_ordinal: u16,
    pub source_session_revision: u64,
    pub source_world_revision: u64,
    pub save_generation_hash: ContentHash,
    pub save_manifest_hash: ContentHash,
    pub authoritative_owner_segments_root: ContentHash,
    pub canonical_hash: ContentHash,
}

impl FinalSaveReceiptV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the receipt binds every identity and authoritative save root"
    )]
    pub fn new(
        session_id: ApplicationSessionId,
        close_request_id: CloseRequestId,
        canonical_close_request_hash: ContentHash,
        reservation_hash: ContentHash,
        attempt_ordinal: u16,
        source_session_revision: u64,
        source_world_revision: u64,
        save_generation_hash: ContentHash,
        save_manifest_hash: ContentHash,
        authoritative_owner_segments_root: ContentHash,
    ) -> Result<Self, SessionContractError> {
        if attempt_ordinal == 0 {
            return Err(SessionContractError::InvalidAttemptCount);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            session_id,
            close_request_id,
            canonical_close_request_hash,
            reservation_hash,
            attempt_ordinal,
            source_session_revision,
            source_world_revision,
            save_generation_hash,
            save_manifest_hash,
            authoritative_owner_segments_root,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash =
            session_hash("nextengine.final-save-receipt.v1", &value.body_value());
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        if Self::new(
            self.session_id,
            self.close_request_id,
            self.canonical_close_request_hash,
            self.reservation_hash,
            self.attempt_ordinal,
            self.source_session_revision,
            self.source_world_revision,
            self.save_generation_hash,
            self.save_manifest_hash,
            self.authoritative_owner_segments_root,
        )?
        .canonical_hash
            != self.canonical_hash
        {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }

    fn body_value(&self) -> JcsValue {
        object([
            ("attempt_ordinal", number(self.attempt_ordinal)),
            (
                "authoritative_owner_segments_root",
                string(self.authoritative_owner_segments_root.to_hex()),
            ),
            (
                "canonical_close_request_hash",
                string(self.canonical_close_request_hash.to_hex()),
            ),
            ("close_request_id", string(self.close_request_id.to_hex())),
            ("reservation_hash", string(self.reservation_hash.to_hex())),
            (
                "save_generation_hash",
                string(self.save_generation_hash.to_hex()),
            ),
            (
                "save_manifest_hash",
                string(self.save_manifest_hash.to_hex()),
            ),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
            (
                "source_session_revision",
                number(self.source_session_revision),
            ),
            ("source_world_revision", number(self.source_world_revision)),
        ])
    }
}

#[must_use]
pub const fn can_close_after_failed_save(
    disposition: FailureDispositionV1,
    last_safe_generation_hash: Option<ContentHash>,
) -> bool {
    matches!(
        (disposition, last_safe_generation_hash),
        (FailureDispositionV1::AllowLastSafeGeneration, Some(_))
    )
}
