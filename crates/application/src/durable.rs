use next_contracts::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_OPTIONAL,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits, CanonicalField,
    DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment,
};
use next_contracts::ids::{
    ApplicationSessionId, CloseRequestId, ContentHash, SchemaId, SessionTransitionId,
};
use next_contracts::session::{
    ApplicationSessionManifestV1, ApplicationSessionStateV1, ApplicationSessionStatusV1,
    CloseSessionOperationStageV1, FailureDispositionV1, FinalSaveLedgerStatusV1,
};

use crate::ApplicationError;

const DURABLE_SNAPSHOT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DurableLedgerV1 {
    pub status: FinalSaveLedgerStatusV1,
    pub attempt_count: u16,
    pub maximum_attempts: u16,
    pub last_failure_code: Option<SchemaId>,
    pub reservation_hash: ContentHash,
    pub entry_hash: ContentHash,
    pub save_generation_hash: Option<ContentHash>,
    pub final_save_receipt_hash: Option<ContentHash>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DurableCloseOperationV1 {
    pub close_request_id: CloseRequestId,
    pub canonical_close_request_hash: ContentHash,
    pub canonical_close_request_bytes: Vec<u8>,
    pub close_request_archive_ref: ContentHash,
    pub starting_session_revision: u64,
    pub starting_session_state: ApplicationSessionStatusV1,
    pub stage: CloseSessionOperationStageV1,
    pub operation_journal_hash: ContentHash,
    pub quiesce_event_hash: Option<ContentHash>,
    pub finalizing_event_hash: Option<ContentHash>,
    pub ledger: Option<DurableLedgerV1>,
    pub failure_disposition: FailureDispositionV1,
    pub close_session_receipt_hash: Option<ContentHash>,
    pub closed_event_hash: Option<ContentHash>,
    pub last_safe_generation_hash: Option<ContentHash>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DurableApplicationSnapshotV1 {
    pub store_sequence: u64,
    pub manifest: ApplicationSessionManifestV1,
    pub state: ApplicationSessionStateV1,
    pub close: Option<DurableCloseOperationV1>,
}

impl DurableApplicationSnapshotV1 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ApplicationError> {
        self.manifest.validate()?;
        self.state.validate()?;
        Ok(encode_canonical_segment(
            "nextengine.application",
            "nextengine.application-durable-snapshot.v1",
            &self.manifest.body.session_id.to_hex(),
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    DURABLE_SNAPSHOT_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_U64,
                    self.store_sequence.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, self.manifest.to_jcs_bytes()),
                CanonicalField::new(4, CANONICAL_TYPE_STRUCT, encode_state(&self.state)?),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_OPTIONAL,
                    self.close.as_ref().map_or(Ok(Vec::new()), encode_close)?,
                ),
            ],
        )?)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, ApplicationError> {
        let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())?;
        ensure_segment(
            &decoded,
            "nextengine.application",
            "nextengine.application-durable-snapshot.v1",
            5,
        )?;
        if decode_u32(field(&decoded, 1, CANONICAL_TYPE_U32)?)? != DURABLE_SNAPSHOT_SCHEMA_VERSION {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        let manifest = ApplicationSessionManifestV1::from_jcs_bytes(
            field(&decoded, 3, CANONICAL_TYPE_BYTES)?,
            CanonicalDecodeLimits::default(),
        )?;
        if decoded.segment_id != manifest.body.session_id.to_hex() {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        let state = decode_state(field(&decoded, 4, CANONICAL_TYPE_STRUCT)?)?;
        if state.session_id != manifest.body.session_id
            || state.application_session_manifest_hash != manifest.canonical_hash
        {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        let close_bytes = field(&decoded, 5, CANONICAL_TYPE_OPTIONAL)?;
        let close = if close_bytes.is_empty() {
            None
        } else {
            Some(decode_close(close_bytes)?)
        };
        Ok(Self {
            store_sequence: decode_u64(field(&decoded, 2, CANONICAL_TYPE_U64)?)?,
            manifest,
            state,
            close,
        })
    }
}

fn encode_state(state: &ApplicationSessionStateV1) -> Result<Vec<u8>, ApplicationError> {
    Ok(encode_canonical_segment(
        "nextengine.runtime",
        "nextengine.application-session-state-store.v1",
        &state.session_id.to_hex(),
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                state.schema_version.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_ID128,
                state.session_id.as_bytes().to_vec(),
            ),
            CanonicalField::new(3, CANONICAL_TYPE_U8, vec![state.state as u8]),
            CanonicalField::new(4, CANONICAL_TYPE_U64, state.revision.to_le_bytes().to_vec()),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_HASH256,
                state.application_session_manifest_hash.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_HASH256,
                state.project_composition_lock_hash.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_OPTIONAL,
                state
                    .active_runtime_revision
                    .map_or_else(Vec::new, |value| value.to_le_bytes().to_vec()),
            ),
            CanonicalField::new(
                8,
                CANONICAL_TYPE_OPTIONAL,
                state
                    .active_save_generation_hash
                    .map_or_else(Vec::new, |value| value.as_bytes().to_vec()),
            ),
            CanonicalField::new(
                9,
                CANONICAL_TYPE_OPTIONAL,
                state
                    .last_transition_id
                    .map_or_else(Vec::new, |value| value.as_bytes().to_vec()),
            ),
            CanonicalField::new(
                10,
                CANONICAL_TYPE_OPTIONAL,
                state
                    .terminal_receipt_hash
                    .map_or_else(Vec::new, |value| value.as_bytes().to_vec()),
            ),
            CanonicalField::new(
                11,
                CANONICAL_TYPE_HASH256,
                state.canonical_hash.as_bytes().to_vec(),
            ),
        ],
    )?)
}

fn decode_state(bytes: &[u8]) -> Result<ApplicationSessionStateV1, ApplicationError> {
    let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())?;
    ensure_segment(
        &decoded,
        "nextengine.runtime",
        "nextengine.application-session-state-store.v1",
        11,
    )?;
    let state = ApplicationSessionStateV1 {
        schema_version: decode_u32(field(&decoded, 1, CANONICAL_TYPE_U32)?)?,
        session_id: decode_session_id(field(&decoded, 2, CANONICAL_TYPE_ID128)?)?,
        state: decode_session_status(decode_u8(field(&decoded, 3, CANONICAL_TYPE_U8)?)?)?,
        revision: decode_u64(field(&decoded, 4, CANONICAL_TYPE_U64)?)?,
        application_session_manifest_hash: decode_hash(field(
            &decoded,
            5,
            CANONICAL_TYPE_HASH256,
        )?)?,
        project_composition_lock_hash: decode_hash(field(&decoded, 6, CANONICAL_TYPE_HASH256)?)?,
        active_runtime_revision: decode_optional_u64(field(&decoded, 7, CANONICAL_TYPE_OPTIONAL)?)?,
        active_save_generation_hash: decode_optional_hash(field(
            &decoded,
            8,
            CANONICAL_TYPE_OPTIONAL,
        )?)?,
        last_transition_id: decode_optional_transition_id(field(
            &decoded,
            9,
            CANONICAL_TYPE_OPTIONAL,
        )?)?,
        terminal_receipt_hash: decode_optional_hash(field(&decoded, 10, CANONICAL_TYPE_OPTIONAL)?)?,
        canonical_hash: decode_hash(field(&decoded, 11, CANONICAL_TYPE_HASH256)?)?,
    };
    state.validate()?;
    if decoded.segment_id != state.session_id.to_hex() {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    Ok(state)
}

fn encode_close(close: &DurableCloseOperationV1) -> Result<Vec<u8>, ApplicationError> {
    let (
        ledger_status,
        attempt_count,
        maximum_attempts,
        last_failure,
        reservation,
        entry,
        save,
        receipt,
    ) = close.ledger.as_ref().map_or(
        (
            0,
            0,
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        |ledger| {
            (
                ledger.status as u8,
                ledger.attempt_count,
                ledger.maximum_attempts,
                ledger
                    .last_failure_code
                    .as_ref()
                    .map_or_else(Vec::new, |value| value.as_str().as_bytes().to_vec()),
                ledger.reservation_hash.as_bytes().to_vec(),
                ledger.entry_hash.as_bytes().to_vec(),
                ledger
                    .save_generation_hash
                    .map_or_else(Vec::new, |value| value.as_bytes().to_vec()),
                ledger
                    .final_save_receipt_hash
                    .map_or_else(Vec::new, |value| value.as_bytes().to_vec()),
            )
        },
    );
    Ok(encode_canonical_segment(
        "nextengine.application",
        "nextengine.durable-close-operation.v1",
        &close.close_request_id.to_hex(),
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_ID128,
                close.close_request_id.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_HASH256,
                close.canonical_close_request_hash.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_BYTES,
                close.canonical_close_request_bytes.clone(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_HASH256,
                close.close_request_archive_ref.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_U64,
                close.starting_session_revision.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_U8,
                vec![close.starting_session_state as u8],
            ),
            CanonicalField::new(7, CANONICAL_TYPE_U8, vec![close.stage as u8]),
            CanonicalField::new(
                8,
                CANONICAL_TYPE_HASH256,
                close.operation_journal_hash.as_bytes().to_vec(),
            ),
            optional_hash_field(9, close.quiesce_event_hash),
            optional_hash_field(10, close.finalizing_event_hash),
            CanonicalField::new(11, CANONICAL_TYPE_U8, vec![ledger_status]),
            CanonicalField::new(12, CANONICAL_TYPE_U16, attempt_count.to_le_bytes().to_vec()),
            CanonicalField::new(
                13,
                CANONICAL_TYPE_U16,
                maximum_attempts.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(14, CANONICAL_TYPE_U8, vec![close.failure_disposition as u8]),
            CanonicalField::new(15, CANONICAL_TYPE_UTF8_NFC, last_failure),
            CanonicalField::new(16, CANONICAL_TYPE_OPTIONAL, reservation),
            CanonicalField::new(17, CANONICAL_TYPE_OPTIONAL, entry),
            CanonicalField::new(18, CANONICAL_TYPE_OPTIONAL, save),
            CanonicalField::new(19, CANONICAL_TYPE_OPTIONAL, receipt),
            optional_hash_field(20, close.close_session_receipt_hash),
            optional_hash_field(21, close.closed_event_hash),
            optional_hash_field(22, close.last_safe_generation_hash),
        ],
    )?)
}

fn decode_close(bytes: &[u8]) -> Result<DurableCloseOperationV1, ApplicationError> {
    let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())?;
    ensure_segment(
        &decoded,
        "nextengine.application",
        "nextengine.durable-close-operation.v1",
        22,
    )?;
    let close_request_id = decode_close_id(field(&decoded, 1, CANONICAL_TYPE_ID128)?)?;
    let ledger_status = decode_u8(field(&decoded, 11, CANONICAL_TYPE_U8)?)?;
    let last_failure = field(&decoded, 15, CANONICAL_TYPE_UTF8_NFC)?;
    let ledger = if ledger_status == 0 {
        None
    } else {
        Some(DurableLedgerV1 {
            status: decode_ledger_status(ledger_status)?,
            attempt_count: decode_u16(field(&decoded, 12, CANONICAL_TYPE_U16)?)?,
            maximum_attempts: decode_u16(field(&decoded, 13, CANONICAL_TYPE_U16)?)?,
            last_failure_code: if last_failure.is_empty() {
                None
            } else {
                Some(SchemaId::new(
                    std::str::from_utf8(last_failure)
                        .map_err(|_| ApplicationError::DurableSnapshotInvalid)?,
                )?)
            },
            reservation_hash: decode_hash(field(&decoded, 16, CANONICAL_TYPE_OPTIONAL)?)?,
            entry_hash: decode_hash(field(&decoded, 17, CANONICAL_TYPE_OPTIONAL)?)?,
            save_generation_hash: decode_optional_hash(field(
                &decoded,
                18,
                CANONICAL_TYPE_OPTIONAL,
            )?)?,
            final_save_receipt_hash: decode_optional_hash(field(
                &decoded,
                19,
                CANONICAL_TYPE_OPTIONAL,
            )?)?,
        })
    };
    if decoded.segment_id != close_request_id.to_hex() {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    Ok(DurableCloseOperationV1 {
        close_request_id,
        canonical_close_request_hash: decode_hash(field(&decoded, 2, CANONICAL_TYPE_HASH256)?)?,
        canonical_close_request_bytes: field(&decoded, 3, CANONICAL_TYPE_BYTES)?.to_vec(),
        close_request_archive_ref: decode_hash(field(&decoded, 4, CANONICAL_TYPE_HASH256)?)?,
        starting_session_revision: decode_u64(field(&decoded, 5, CANONICAL_TYPE_U64)?)?,
        starting_session_state: decode_session_status(decode_u8(field(
            &decoded,
            6,
            CANONICAL_TYPE_U8,
        )?)?)?,
        stage: decode_close_stage(decode_u8(field(&decoded, 7, CANONICAL_TYPE_U8)?)?)?,
        operation_journal_hash: decode_hash(field(&decoded, 8, CANONICAL_TYPE_HASH256)?)?,
        quiesce_event_hash: decode_optional_hash(field(&decoded, 9, CANONICAL_TYPE_OPTIONAL)?)?,
        finalizing_event_hash: decode_optional_hash(field(&decoded, 10, CANONICAL_TYPE_OPTIONAL)?)?,
        ledger,
        failure_disposition: decode_disposition(decode_u8(field(
            &decoded,
            14,
            CANONICAL_TYPE_U8,
        )?)?)?,
        close_session_receipt_hash: decode_optional_hash(field(
            &decoded,
            20,
            CANONICAL_TYPE_OPTIONAL,
        )?)?,
        closed_event_hash: decode_optional_hash(field(&decoded, 21, CANONICAL_TYPE_OPTIONAL)?)?,
        last_safe_generation_hash: decode_optional_hash(field(
            &decoded,
            22,
            CANONICAL_TYPE_OPTIONAL,
        )?)?,
    })
}

fn optional_hash_field(field_id: u32, value: Option<ContentHash>) -> CanonicalField {
    CanonicalField::new(
        field_id,
        CANONICAL_TYPE_OPTIONAL,
        value.map_or_else(Vec::new, |value| value.as_bytes().to_vec()),
    )
}

fn ensure_segment(
    decoded: &DecodedCanonicalSegment,
    owner: &str,
    schema: &str,
    field_count: usize,
) -> Result<(), ApplicationError> {
    if decoded.owner_id != owner
        || decoded.schema_id != schema
        || decoded.fields.len() != field_count
    {
        Err(ApplicationError::DurableSnapshotInvalid)
    } else {
        Ok(())
    }
}

fn field(decoded: &DecodedCanonicalSegment, id: u32, tag: u8) -> Result<&[u8], ApplicationError> {
    let field = decoded
        .field(id)
        .ok_or(ApplicationError::DurableSnapshotInvalid)?;
    if field.type_tag != tag {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    Ok(&field.payload)
}

fn decode_u8(bytes: &[u8]) -> Result<u8, ApplicationError> {
    bytes
        .first()
        .copied()
        .filter(|_| bytes.len() == 1)
        .ok_or(ApplicationError::DurableSnapshotInvalid)
}

fn decode_u16(bytes: &[u8]) -> Result<u16, ApplicationError> {
    Ok(u16::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::DurableSnapshotInvalid)?,
    ))
}

fn decode_u32(bytes: &[u8]) -> Result<u32, ApplicationError> {
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::DurableSnapshotInvalid)?,
    ))
}

fn decode_u64(bytes: &[u8]) -> Result<u64, ApplicationError> {
    Ok(u64::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::DurableSnapshotInvalid)?,
    ))
}

fn decode_hash(bytes: &[u8]) -> Result<ContentHash, ApplicationError> {
    Ok(ContentHash::from_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::DurableSnapshotInvalid)?,
    ))
}

fn decode_optional_hash(bytes: &[u8]) -> Result<Option<ContentHash>, ApplicationError> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        decode_hash(bytes).map(Some)
    }
}

fn decode_session_id(bytes: &[u8]) -> Result<ApplicationSessionId, ApplicationError> {
    Ok(ApplicationSessionId::from_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::DurableSnapshotInvalid)?,
    ))
}

fn decode_close_id(bytes: &[u8]) -> Result<CloseRequestId, ApplicationError> {
    Ok(CloseRequestId::from_bytes(
        bytes
            .try_into()
            .map_err(|_| ApplicationError::DurableSnapshotInvalid)?,
    ))
}

fn decode_optional_transition_id(
    bytes: &[u8],
) -> Result<Option<SessionTransitionId>, ApplicationError> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        Ok(Some(SessionTransitionId::from_bytes(
            bytes
                .try_into()
                .map_err(|_| ApplicationError::DurableSnapshotInvalid)?,
        )))
    }
}

fn decode_optional_u64(bytes: &[u8]) -> Result<Option<u64>, ApplicationError> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        decode_u64(bytes).map(Some)
    }
}

fn decode_session_status(value: u8) -> Result<ApplicationSessionStatusV1, ApplicationError> {
    match value {
        1 => Ok(ApplicationSessionStatusV1::Created),
        2 => Ok(ApplicationSessionStatusV1::CompositionStaged),
        3 => Ok(ApplicationSessionStatusV1::RuntimeStaged),
        4 => Ok(ApplicationSessionStatusV1::Active),
        5 => Ok(ApplicationSessionStatusV1::Suspended),
        6 => Ok(ApplicationSessionStatusV1::Quiescing),
        7 => Ok(ApplicationSessionStatusV1::Finalizing),
        8 => Ok(ApplicationSessionStatusV1::Closed),
        _ => Err(ApplicationError::DurableSnapshotInvalid),
    }
}

fn decode_close_stage(value: u8) -> Result<CloseSessionOperationStageV1, ApplicationError> {
    match value {
        1 => Ok(CloseSessionOperationStageV1::Registered),
        2 => Ok(CloseSessionOperationStageV1::Quiesced),
        3 => Ok(CloseSessionOperationStageV1::Finalizing),
        4 => Ok(CloseSessionOperationStageV1::SaveRetryPending),
        5 => Ok(CloseSessionOperationStageV1::SaveTerminal),
        6 => Ok(CloseSessionOperationStageV1::Closed),
        _ => Err(ApplicationError::DurableSnapshotInvalid),
    }
}

fn decode_ledger_status(value: u8) -> Result<FinalSaveLedgerStatusV1, ApplicationError> {
    match value {
        1 => Ok(FinalSaveLedgerStatusV1::Reserved),
        2 => Ok(FinalSaveLedgerStatusV1::RetryPending),
        3 => Ok(FinalSaveLedgerStatusV1::Committed),
        4 => Ok(FinalSaveLedgerStatusV1::Failed),
        _ => Err(ApplicationError::DurableSnapshotInvalid),
    }
}

fn decode_disposition(value: u8) -> Result<FailureDispositionV1, ApplicationError> {
    match value {
        1 => Ok(FailureDispositionV1::RequireFinalSave),
        2 => Ok(FailureDispositionV1::AllowLastSafeGeneration),
        _ => Err(ApplicationError::DurableSnapshotInvalid),
    }
}
