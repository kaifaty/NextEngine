use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalField, encode_canonical_segment, sha256,
};
use crate::ids::{
    ApplicationSessionId, CloseRequestId, ContentHash, SchemaId, content_hash_from_bytes,
};
use crate::manifest_jcs::JcsValue;

use super::codec::{number, object, optional_hash, session_hash, string};
use super::{
    APPLICATION_SESSION_SCHEMA_VERSION, ApplicationSessionStatusV1, BoundedDeadlineClassV1,
    CausalInputReferenceV1, FinalSavePolicyV1, LifecycleReasonV1, SessionContractError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseSessionRequestV1 {
    pub schema_version: u32,
    pub close_request_id: CloseRequestId,
    pub session_id: ApplicationSessionId,
    pub starting_session_revision: u64,
    pub starting_session_state: ApplicationSessionStatusV1,
    pub shutdown_policy_hash: ContentHash,
    pub final_save_policy: FinalSavePolicyV1,
    pub bounded_deadline_class: BoundedDeadlineClassV1,
    pub reason: LifecycleReasonV1,
    pub causal_input_reference: CausalInputReferenceV1,
    pub canonical_close_request_hash: ContentHash,
}

impl CloseSessionRequestV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the exactly-once close identity binds every request field"
    )]
    pub fn new(
        close_request_id: CloseRequestId,
        session_id: ApplicationSessionId,
        starting_session_revision: u64,
        starting_session_state: ApplicationSessionStatusV1,
        shutdown_policy_hash: ContentHash,
        final_save_policy: FinalSavePolicyV1,
        bounded_deadline_class: BoundedDeadlineClassV1,
        reason: LifecycleReasonV1,
        causal_input_reference: CausalInputReferenceV1,
    ) -> Result<Self, SessionContractError> {
        if !matches!(
            starting_session_state,
            ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended
        ) {
            return Err(SessionContractError::InvalidCloseState);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            close_request_id,
            session_id,
            starting_session_revision,
            starting_session_state,
            shutdown_policy_hash,
            final_save_policy,
            bounded_deadline_class,
            reason,
            causal_input_reference,
            canonical_close_request_hash: ContentHash::default(),
        };
        value.canonical_close_request_hash =
            value.compute_hash(&value.canonical_bytes_without_hash()?);
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION {
            return Err(SessionContractError::UnsupportedVersion);
        }
        if !matches!(
            self.starting_session_state,
            ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended
        ) {
            return Err(SessionContractError::InvalidCloseState);
        }
        if self.compute_hash(&self.canonical_bytes_without_hash()?)
            != self.canonical_close_request_hash
        {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, SessionContractError> {
        let mut fields = self.fields();
        fields.push(CanonicalField::new(
            11,
            CANONICAL_TYPE_HASH256,
            self.canonical_close_request_hash.as_bytes().to_vec(),
        ));
        Ok(encode_canonical_segment(
            "nextengine.runtime",
            "nextengine.close-session-request.v1",
            &self.close_request_id.to_hex(),
            fields,
        )?)
    }

    fn canonical_bytes_without_hash(&self) -> Result<Vec<u8>, SessionContractError> {
        Ok(encode_canonical_segment(
            "nextengine.runtime",
            "nextengine.close-session-request.v1",
            &self.close_request_id.to_hex(),
            self.fields(),
        )?)
    }

    fn fields(&self) -> Vec<CanonicalField> {
        vec![
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                self.schema_version.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_ID128,
                self.close_request_id.as_bytes().to_vec(),
            ),
            CanonicalField::new(3, CANONICAL_TYPE_ID128, self.session_id.as_bytes().to_vec()),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_U64,
                self.starting_session_revision.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_U8,
                vec![self.starting_session_state as u8],
            ),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_HASH256,
                self.shutdown_policy_hash.as_bytes().to_vec(),
            ),
            CanonicalField::new(7, CANONICAL_TYPE_U8, vec![self.final_save_policy as u8]),
            CanonicalField::new(
                8,
                CANONICAL_TYPE_U8,
                vec![self.bounded_deadline_class as u8],
            ),
            CanonicalField::new(9, CANONICAL_TYPE_U16, {
                let bytes = self.reason.reason_code.as_str().as_bytes();
                let mut payload = Vec::with_capacity(1 + bytes.len());
                payload.push(self.reason.kind as u8);
                payload.extend_from_slice(bytes);
                payload
            }),
            CanonicalField::new(10, CANONICAL_TYPE_HASH256, {
                let mut payload = Vec::with_capacity(33);
                payload.push(self.causal_input_reference.source_kind as u8);
                payload.extend_from_slice(self.causal_input_reference.canonical_hash.as_bytes());
                payload
            }),
        ]
    }

    fn compute_hash(&self, bytes: &[u8]) -> ContentHash {
        let mut preimage = b"nextengine.close-session-request.v1\0".to_vec();
        preimage.extend_from_slice(bytes);
        content_hash_from_bytes(sha256(&preimage))
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CloseSessionOperationStageV1 {
    Registered = 1,
    Quiesced = 2,
    Finalizing = 3,
    SaveRetryPending = 4,
    SaveTerminal = 5,
    Closed = 6,
}

impl CloseSessionOperationStageV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Registered => "Registered",
            Self::Quiesced => "Quiesced",
            Self::Finalizing => "Finalizing",
            Self::SaveRetryPending => "SaveRetryPending",
            Self::SaveTerminal => "SaveTerminal",
            Self::Closed => "Closed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseSessionOperationJournalV1 {
    pub schema_version: u32,
    pub session_id: ApplicationSessionId,
    pub close_request_id: CloseRequestId,
    pub canonical_close_request_hash: ContentHash,
    pub close_request_archive_ref: ContentHash,
    pub starting_session_revision: u64,
    pub starting_session_state: ApplicationSessionStatusV1,
    pub stage: CloseSessionOperationStageV1,
    pub quiesce_event_hash: Option<ContentHash>,
    pub finalizing_event_hash: Option<ContentHash>,
    pub final_save_ledger_entry_hash: Option<ContentHash>,
    pub close_session_receipt_hash: Option<ContentHash>,
    pub canonical_hash: ContentHash,
}

impl CloseSessionOperationJournalV1 {
    pub fn registered(
        request: &CloseSessionRequestV1,
        close_request_archive_ref: ContentHash,
    ) -> Result<Self, SessionContractError> {
        request.validate()?;
        Ok(Self::new_unchecked(
            request,
            close_request_archive_ref,
            CloseSessionOperationStageV1::Registered,
            None,
            None,
            None,
            None,
        ))
    }

    pub fn with_stage(
        &self,
        request: &CloseSessionRequestV1,
        stage: CloseSessionOperationStageV1,
        quiesce_event_hash: Option<ContentHash>,
        finalizing_event_hash: Option<ContentHash>,
        final_save_ledger_entry_hash: Option<ContentHash>,
        close_session_receipt_hash: Option<ContentHash>,
    ) -> Result<Self, SessionContractError> {
        self.validate()?;
        request.validate()?;
        if request.session_id != self.session_id
            || request.close_request_id != self.close_request_id
            || request.canonical_close_request_hash != self.canonical_close_request_hash
        {
            return Err(SessionContractError::IdentityMismatch);
        }
        let next = Self::new_unchecked(
            request,
            self.close_request_archive_ref,
            stage,
            quiesce_event_hash,
            finalizing_event_hash,
            final_save_ledger_entry_hash,
            close_session_receipt_hash,
        );
        next.validate()?;
        Ok(next)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if self.schema_version != APPLICATION_SESSION_SCHEMA_VERSION
            || !matches!(
                self.starting_session_state,
                ApplicationSessionStatusV1::Active | ApplicationSessionStatusV1::Suspended
            )
        {
            return Err(SessionContractError::InvalidCloseState);
        }
        let valid = match self.stage {
            CloseSessionOperationStageV1::Registered => {
                self.quiesce_event_hash.is_none()
                    && self.finalizing_event_hash.is_none()
                    && self.final_save_ledger_entry_hash.is_none()
                    && self.close_session_receipt_hash.is_none()
            }
            CloseSessionOperationStageV1::Quiesced => {
                self.quiesce_event_hash.is_some()
                    && self.finalizing_event_hash.is_none()
                    && self.close_session_receipt_hash.is_none()
            }
            CloseSessionOperationStageV1::Finalizing => {
                self.quiesce_event_hash.is_some()
                    && self.finalizing_event_hash.is_some()
                    && self.final_save_ledger_entry_hash.is_some()
                    && self.close_session_receipt_hash.is_none()
            }
            CloseSessionOperationStageV1::SaveRetryPending
            | CloseSessionOperationStageV1::SaveTerminal => {
                self.quiesce_event_hash.is_some()
                    && self.finalizing_event_hash.is_some()
                    && self.final_save_ledger_entry_hash.is_some()
                    && self.close_session_receipt_hash.is_none()
            }
            CloseSessionOperationStageV1::Closed => {
                self.quiesce_event_hash.is_some()
                    && self.finalizing_event_hash.is_some()
                    && self.final_save_ledger_entry_hash.is_some()
                    && self.close_session_receipt_hash.is_some()
            }
        };
        if !valid || self.computed_hash() != self.canonical_hash {
            return Err(SessionContractError::InvalidStateFields);
        }
        Ok(())
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "journal reconstruction keeps all durable stage references explicit"
    )]
    fn new_unchecked(
        request: &CloseSessionRequestV1,
        close_request_archive_ref: ContentHash,
        stage: CloseSessionOperationStageV1,
        quiesce_event_hash: Option<ContentHash>,
        finalizing_event_hash: Option<ContentHash>,
        final_save_ledger_entry_hash: Option<ContentHash>,
        close_session_receipt_hash: Option<ContentHash>,
    ) -> Self {
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            session_id: request.session_id,
            close_request_id: request.close_request_id,
            canonical_close_request_hash: request.canonical_close_request_hash,
            close_request_archive_ref,
            starting_session_revision: request.starting_session_revision,
            starting_session_state: request.starting_session_state,
            stage,
            quiesce_event_hash,
            finalizing_event_hash,
            final_save_ledger_entry_hash,
            close_session_receipt_hash,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = value.computed_hash();
        value
    }

    fn computed_hash(&self) -> ContentHash {
        session_hash(
            "nextengine.close-session-operation-journal.v1",
            &object([
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
                    "close_session_receipt_hash_or_none",
                    optional_hash(self.close_session_receipt_hash),
                ),
                (
                    "final_save_ledger_entry_hash_or_none",
                    optional_hash(self.final_save_ledger_entry_hash),
                ),
                (
                    "finalizing_event_hash_or_none",
                    optional_hash(self.finalizing_event_hash),
                ),
                (
                    "quiesce_event_hash_or_none",
                    optional_hash(self.quiesce_event_hash),
                ),
                ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
                ("session_id", string(self.session_id.to_hex())),
                ("stage", string(self.stage.token())),
                (
                    "starting_session_revision",
                    number(self.starting_session_revision),
                ),
                (
                    "starting_session_state",
                    string(self.starting_session_state.token()),
                ),
            ]),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseSessionProgressResultV1 {
    RetryPending,
    FinalSaveRequiredFailed,
}

impl CloseSessionProgressResultV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::RetryPending => "RetryPending",
            Self::FinalSaveRequiredFailed => "FinalSaveRequiredFailed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseSessionProgressV1 {
    pub schema_version: u32,
    pub session_id: ApplicationSessionId,
    pub close_request_id: CloseRequestId,
    pub canonical_close_request_hash: ContentHash,
    pub operation_journal_hash: ContentHash,
    pub current_session_revision: u64,
    pub current_session_state: ApplicationSessionStatusV1,
    pub final_save_ledger_entry_hash: ContentHash,
    pub attempt_count: u16,
    pub result: CloseSessionProgressResultV1,
    pub canonical_hash: ContentHash,
}

impl CloseSessionProgressV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "durable progress binds the exact journal and ledger revision"
    )]
    pub fn new(
        session_id: ApplicationSessionId,
        close_request_id: CloseRequestId,
        canonical_close_request_hash: ContentHash,
        operation_journal_hash: ContentHash,
        current_session_revision: u64,
        final_save_ledger_entry_hash: ContentHash,
        attempt_count: u16,
        result: CloseSessionProgressResultV1,
    ) -> Result<Self, SessionContractError> {
        if attempt_count == 0 {
            return Err(SessionContractError::InvalidAttemptCount);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            session_id,
            close_request_id,
            canonical_close_request_hash,
            operation_journal_hash,
            current_session_revision,
            current_session_state: ApplicationSessionStatusV1::Finalizing,
            final_save_ledger_entry_hash,
            attempt_count,
            result,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash =
            session_hash("nextengine.close-session-progress.v1", &value.body_value());
        Ok(value)
    }

    fn body_value(&self) -> JcsValue {
        object([
            ("attempt_count", number(self.attempt_count)),
            (
                "canonical_close_request_hash",
                string(self.canonical_close_request_hash.to_hex()),
            ),
            ("close_request_id", string(self.close_request_id.to_hex())),
            (
                "current_session_revision",
                number(self.current_session_revision),
            ),
            (
                "current_session_state",
                string(self.current_session_state.token()),
            ),
            (
                "final_save_ledger_entry_hash",
                string(self.final_save_ledger_entry_hash.to_hex()),
            ),
            (
                "operation_journal_hash",
                string(self.operation_journal_hash.to_hex()),
            ),
            ("result", string(self.result.token())),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
        ])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseSessionResultV1 {
    Saved,
    ClosedUsingLastSafeGeneration,
}

impl CloseSessionResultV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Saved => "Saved",
            Self::ClosedUsingLastSafeGeneration => "ClosedUsingLastSafeGeneration",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseSessionReceiptV1 {
    pub schema_version: u32,
    pub close_request_id: CloseRequestId,
    pub session_id: ApplicationSessionId,
    pub canonical_close_request_hash: ContentHash,
    pub starting_revision: u64,
    pub quiesce_event_hash: ContentHash,
    pub final_save_ledger_entry_hash: ContentHash,
    pub final_save_receipt_hash: Option<ContentHash>,
    pub last_safe_generation_hash: Option<ContentHash>,
    pub finalizing_event_hash: ContentHash,
    pub preclose_operation_journal_hash: ContentHash,
    pub closed_event_hash: ContentHash,
    pub terminal_session_revision: u64,
    pub result: CloseSessionResultV1,
    pub canonical_hash: ContentHash,
}

impl CloseSessionReceiptV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "terminal receipt binds every preceding durable close publication"
    )]
    pub fn new(
        close_request_id: CloseRequestId,
        session_id: ApplicationSessionId,
        canonical_close_request_hash: ContentHash,
        starting_revision: u64,
        quiesce_event_hash: ContentHash,
        final_save_ledger_entry_hash: ContentHash,
        final_save_receipt_hash: Option<ContentHash>,
        last_safe_generation_hash: Option<ContentHash>,
        finalizing_event_hash: ContentHash,
        preclose_operation_journal_hash: ContentHash,
        closed_event_hash: ContentHash,
        terminal_session_revision: u64,
        result: CloseSessionResultV1,
    ) -> Result<Self, SessionContractError> {
        let references_valid = matches!(
            (result, final_save_receipt_hash, last_safe_generation_hash),
            (CloseSessionResultV1::Saved, Some(_), None)
                | (
                    CloseSessionResultV1::ClosedUsingLastSafeGeneration,
                    None,
                    Some(_)
                )
        );
        if !references_valid || terminal_session_revision <= starting_revision {
            return Err(SessionContractError::InvalidStateFields);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            close_request_id,
            session_id,
            canonical_close_request_hash,
            starting_revision,
            quiesce_event_hash,
            final_save_ledger_entry_hash,
            final_save_receipt_hash,
            last_safe_generation_hash,
            finalizing_event_hash,
            preclose_operation_journal_hash,
            closed_event_hash,
            terminal_session_revision,
            result,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash =
            session_hash("nextengine.close-session-receipt.v1", &value.body_value());
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        Self::new(
            self.close_request_id,
            self.session_id,
            self.canonical_close_request_hash,
            self.starting_revision,
            self.quiesce_event_hash,
            self.final_save_ledger_entry_hash,
            self.final_save_receipt_hash,
            self.last_safe_generation_hash,
            self.finalizing_event_hash,
            self.preclose_operation_journal_hash,
            self.closed_event_hash,
            self.terminal_session_revision,
            self.result,
        )
        .and_then(|canonical| {
            if canonical.canonical_hash == self.canonical_hash {
                Ok(())
            } else {
                Err(SessionContractError::HashMismatch)
            }
        })
    }

    fn body_value(&self) -> JcsValue {
        object([
            (
                "canonical_close_request_hash",
                string(self.canonical_close_request_hash.to_hex()),
            ),
            ("close_request_id", string(self.close_request_id.to_hex())),
            ("closed_event_hash", string(self.closed_event_hash.to_hex())),
            (
                "final_save_ledger_entry_hash",
                string(self.final_save_ledger_entry_hash.to_hex()),
            ),
            (
                "final_save_receipt_hash_or_none",
                optional_hash(self.final_save_receipt_hash),
            ),
            (
                "finalizing_event_hash",
                string(self.finalizing_event_hash.to_hex()),
            ),
            (
                "last_safe_generation_hash_or_none",
                optional_hash(self.last_safe_generation_hash),
            ),
            (
                "preclose_operation_journal_hash",
                string(self.preclose_operation_journal_hash.to_hex()),
            ),
            (
                "quiesce_event_hash",
                string(self.quiesce_event_hash.to_hex()),
            ),
            ("result", string(self.result.token())),
            ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ("session_id", string(self.session_id.to_hex())),
            ("starting_revision", number(self.starting_revision)),
            (
                "terminal_session_revision",
                number(self.terminal_session_revision),
            ),
        ])
    }
}

#[must_use]
pub fn close_request_archive_ref(bytes: &[u8]) -> ContentHash {
    content_hash_from_bytes(sha256(bytes))
}

pub fn stable_failure_code(value: &str) -> Result<SchemaId, SessionContractError> {
    Ok(SchemaId::new(value)?)
}
