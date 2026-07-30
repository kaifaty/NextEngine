use next_contracts::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_OPTIONAL,
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits,
    CanonicalField, DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment,
};
use next_contracts::ids::{
    ApplicationSessionId, CloseRequestId, ContentHash, SchemaId, SessionRequestId,
    SessionTransitionId,
};
use next_contracts::session::{
    ApplicationSessionManifestV1, ApplicationSessionStateV1, ApplicationSessionStatusV1,
    CloseSessionOperationStageV1, FailureDispositionV1, FinalSaveLedgerStatusV1,
};

use crate::ApplicationError;

const DURABLE_SNAPSHOT_SCHEMA_VERSION_V1: u32 = 1;
const DURABLE_SNAPSHOT_SCHEMA_VERSION_V2: u32 = 2;
const DURABLE_SNAPSHOT_SCHEMA_VERSION: u32 = 3;
// 1,024 platform-owned requests plus 16 engine-owned launch/close edges.
// At two object records per edge this stays well below SessionStore's 4,096
// object ceiling and leaves deterministic capacity for live recovery and close.
pub(crate) const LIFECYCLE_ARCHIVE_MAX_ENTRIES: usize = 1_040;
pub(crate) const RECOVERY_EVIDENCE_ARCHIVE_MAX_ENTRIES: usize = 64;
pub(crate) const RECOVERY_EVIDENCE_ARCHIVE_MAX_OBJECT_REFERENCES: usize = 16_384;
pub(crate) const RECOVERY_EVIDENCE_OBJECT_BUDGET: usize = 1_984;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct DurableLifecycleArchiveEntryV1 {
    pub request_id: SessionRequestId,
    pub request_object_hash: ContentHash,
    pub event_object_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DurableRecoveryEvidenceEntryV1 {
    pub recovery_link_hash: ContentHash,
    pub recovery_link_object_hash: ContentHash,
    pub prior_snapshot_object_hash: ContentHash,
    pub evidence_object_hashes: Vec<ContentHash>,
}

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
    pub live_run_recovery_manifest_hash: Option<ContentHash>,
    /// Records whether the decoded durable generation carried the v2 recovery
    /// field. This is intentionally not an independent serialized value:
    /// every newly published generation is v3 and carries it, while a decoded
    /// v1 generation must remain distinguishable until recovery policy has
    /// been applied.
    pub live_run_recovery_field_present: bool,
    pub lifecycle_archive: Vec<DurableLifecycleArchiveEntryV1>,
    /// Like the live-recovery marker, this distinguishes a legacy v1/v2
    /// generation from a newly published v3 generation with an empty archive.
    pub lifecycle_archive_field_present: bool,
    pub recovery_evidence_archive: Vec<DurableRecoveryEvidenceEntryV1>,
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
                optional_hash_field(6, self.live_run_recovery_manifest_hash),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_lifecycle_archive(&self.lifecycle_archive)?,
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_recovery_evidence_archive(&self.recovery_evidence_archive)?,
                ),
            ],
        )?)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, ApplicationError> {
        let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())?;
        let schema_version = decode_u32(field(&decoded, 1, CANONICAL_TYPE_U32)?)?;
        let expected_field_count = match schema_version {
            DURABLE_SNAPSHOT_SCHEMA_VERSION_V1 => 5,
            DURABLE_SNAPSHOT_SCHEMA_VERSION_V2 => 6,
            DURABLE_SNAPSHOT_SCHEMA_VERSION => 8,
            _ => return Err(ApplicationError::DurableSnapshotInvalid),
        };
        ensure_segment(
            &decoded,
            "nextengine.application",
            "nextengine.application-durable-snapshot.v1",
            expected_field_count,
        )?;
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
            live_run_recovery_manifest_hash: if schema_version == DURABLE_SNAPSHOT_SCHEMA_VERSION_V1
            {
                None
            } else {
                decode_optional_hash(field(&decoded, 6, CANONICAL_TYPE_OPTIONAL)?)?
            },
            live_run_recovery_field_present: schema_version >= DURABLE_SNAPSHOT_SCHEMA_VERSION_V2,
            lifecycle_archive: if schema_version == DURABLE_SNAPSHOT_SCHEMA_VERSION {
                decode_lifecycle_archive(field(&decoded, 7, CANONICAL_TYPE_SEQUENCE)?)?
            } else {
                Vec::new()
            },
            lifecycle_archive_field_present: schema_version == DURABLE_SNAPSHOT_SCHEMA_VERSION,
            recovery_evidence_archive: if schema_version == DURABLE_SNAPSHOT_SCHEMA_VERSION {
                decode_recovery_evidence_archive(field(&decoded, 8, CANONICAL_TYPE_SEQUENCE)?)?
            } else {
                Vec::new()
            },
        })
    }
}

fn encode_lifecycle_archive(
    entries: &[DurableLifecycleArchiveEntryV1],
) -> Result<Vec<u8>, ApplicationError> {
    if entries.len() > LIFECYCLE_ARCHIVE_MAX_ENTRIES
        || entries
            .windows(2)
            .any(|pair| pair[0].request_id >= pair[1].request_id)
        || entries.iter().any(|entry| {
            entry.request_object_hash == ContentHash::default()
                || entry.event_object_hash == ContentHash::default()
        })
    {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    let count =
        u32::try_from(entries.len()).map_err(|_| ApplicationError::DurableSnapshotInvalid)?;
    let mut bytes = Vec::with_capacity(4 + entries.len() * 80);
    bytes.extend_from_slice(&count.to_le_bytes());
    for entry in entries {
        bytes.extend_from_slice(entry.request_id.as_bytes());
        bytes.extend_from_slice(entry.request_object_hash.as_bytes());
        bytes.extend_from_slice(entry.event_object_hash.as_bytes());
    }
    Ok(bytes)
}

fn decode_lifecycle_archive(
    bytes: &[u8],
) -> Result<Vec<DurableLifecycleArchiveEntryV1>, ApplicationError> {
    let (count_bytes, mut remainder) = bytes
        .split_at_checked(4)
        .ok_or(ApplicationError::DurableSnapshotInvalid)?;
    let count = usize::try_from(decode_u32(count_bytes)?)
        .map_err(|_| ApplicationError::DurableSnapshotInvalid)?;
    if count > LIFECYCLE_ARCHIVE_MAX_ENTRIES
        || remainder.len()
            != count
                .checked_mul(80)
                .ok_or(ApplicationError::DurableSnapshotInvalid)?
    {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let (request_id, next) = remainder
            .split_at_checked(16)
            .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        let (request_object_hash, next) = next
            .split_at_checked(32)
            .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        let (event_object_hash, next) = next
            .split_at_checked(32)
            .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        entries.push(DurableLifecycleArchiveEntryV1 {
            request_id: SessionRequestId::from_bytes(
                request_id
                    .try_into()
                    .map_err(|_| ApplicationError::DurableSnapshotInvalid)?,
            ),
            request_object_hash: decode_hash(request_object_hash)?,
            event_object_hash: decode_hash(event_object_hash)?,
        });
        remainder = next;
    }
    if entries
        .windows(2)
        .any(|pair| pair[0].request_id >= pair[1].request_id)
        || entries.iter().any(|entry| {
            entry.request_object_hash == ContentHash::default()
                || entry.event_object_hash == ContentHash::default()
        })
    {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    Ok(entries)
}

fn encode_recovery_evidence_archive(
    entries: &[DurableRecoveryEvidenceEntryV1],
) -> Result<Vec<u8>, ApplicationError> {
    let reference_count = entries.iter().try_fold(0_usize, |count, entry| {
        count
            .checked_add(entry.evidence_object_hashes.len())
            .ok_or(ApplicationError::DurableSnapshotInvalid)
    })?;
    if entries.len() > RECOVERY_EVIDENCE_ARCHIVE_MAX_ENTRIES
        || reference_count > RECOVERY_EVIDENCE_ARCHIVE_MAX_OBJECT_REFERENCES
        || entries.iter().any(|entry| {
            entry.recovery_link_hash == ContentHash::default()
                || entry.recovery_link_object_hash == ContentHash::default()
                || entry.prior_snapshot_object_hash == ContentHash::default()
                || entry.evidence_object_hashes.len() > RECOVERY_EVIDENCE_OBJECT_BUDGET
                || entry
                    .evidence_object_hashes
                    .iter()
                    .any(|hash| *hash == ContentHash::default())
                || entry
                    .evidence_object_hashes
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])
        })
        || entries.iter().enumerate().any(|(index, entry)| {
            entries[..index].iter().any(|prior| {
                prior.recovery_link_hash == entry.recovery_link_hash
                    || prior.recovery_link_object_hash == entry.recovery_link_object_hash
                    || prior.prior_snapshot_object_hash == entry.prior_snapshot_object_hash
            })
        })
    {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    let count =
        u32::try_from(entries.len()).map_err(|_| ApplicationError::DurableSnapshotInvalid)?;
    let capacity = 4_usize
        .checked_add(
            entries
                .len()
                .checked_mul(100)
                .ok_or(ApplicationError::DurableSnapshotInvalid)?,
        )
        .and_then(|value| value.checked_add(reference_count.checked_mul(32)?))
        .ok_or(ApplicationError::DurableSnapshotInvalid)?;
    let mut bytes = Vec::with_capacity(capacity);
    bytes.extend_from_slice(&count.to_le_bytes());
    for entry in entries {
        bytes.extend_from_slice(entry.recovery_link_hash.as_bytes());
        bytes.extend_from_slice(entry.recovery_link_object_hash.as_bytes());
        bytes.extend_from_slice(entry.prior_snapshot_object_hash.as_bytes());
        bytes.extend_from_slice(
            &u32::try_from(entry.evidence_object_hashes.len())
                .map_err(|_| ApplicationError::DurableSnapshotInvalid)?
                .to_le_bytes(),
        );
        for hash in &entry.evidence_object_hashes {
            bytes.extend_from_slice(hash.as_bytes());
        }
    }
    Ok(bytes)
}

fn decode_recovery_evidence_archive(
    bytes: &[u8],
) -> Result<Vec<DurableRecoveryEvidenceEntryV1>, ApplicationError> {
    let (count_bytes, mut remainder) = bytes
        .split_at_checked(4)
        .ok_or(ApplicationError::DurableSnapshotInvalid)?;
    let count = usize::try_from(decode_u32(count_bytes)?)
        .map_err(|_| ApplicationError::DurableSnapshotInvalid)?;
    if count > RECOVERY_EVIDENCE_ARCHIVE_MAX_ENTRIES {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    let mut entries = Vec::with_capacity(count);
    let mut reference_count = 0_usize;
    for _ in 0..count {
        let (header, next) = remainder
            .split_at_checked(100)
            .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        let object_count = usize::try_from(decode_u32(&header[96..100])?)
            .map_err(|_| ApplicationError::DurableSnapshotInvalid)?;
        reference_count = reference_count
            .checked_add(object_count)
            .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        if object_count > RECOVERY_EVIDENCE_OBJECT_BUDGET
            || reference_count > RECOVERY_EVIDENCE_ARCHIVE_MAX_OBJECT_REFERENCES
        {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        let object_bytes = object_count
            .checked_mul(32)
            .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        let (objects, next_remainder) = next
            .split_at_checked(object_bytes)
            .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        entries.push(DurableRecoveryEvidenceEntryV1 {
            recovery_link_hash: decode_hash(&header[..32])?,
            recovery_link_object_hash: decode_hash(&header[32..64])?,
            prior_snapshot_object_hash: decode_hash(&header[64..96])?,
            evidence_object_hashes: objects
                .chunks_exact(32)
                .map(decode_hash)
                .collect::<Result<Vec<_>, _>>()?,
        });
        remainder = next_remainder;
    }
    if !remainder.is_empty() {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    encode_recovery_evidence_archive(&entries)?;
    Ok(entries)
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
