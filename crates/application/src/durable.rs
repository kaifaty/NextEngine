use next_assets::SaveImage;
use next_contracts::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64,
    CanonicalDecodeLimits, CanonicalField, DecodedCanonicalSegment, decode_canonical_segment,
    encode_canonical_segment,
};
use next_contracts::session::{
    ApplicationLifecycleEventV2, ApplicationLifecycleRequestV2, ApplicationSessionManifestV2,
    ApplicationSessionStateV2, CloseSessionJournalStageV2, CloseSessionJournalV2,
    CloseSessionReceiptV2, CloseSessionRequestV2,
};
use next_runtime::LastLifecycleRecordV2;

use crate::ApplicationError;

const DURABLE_SNAPSHOT_SCHEMA_VERSION: u32 = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DurableApplicationSnapshotV4 {
    pub store_sequence: u64,
    pub manifest: ApplicationSessionManifestV2,
    pub state: ApplicationSessionStateV2,
    pub last_lifecycle: Option<LastLifecycleRecordV2>,
    pub close_request: Option<CloseSessionRequestV2>,
    pub close_journal: Option<CloseSessionJournalV2>,
    pub close_receipt: Option<CloseSessionReceiptV2>,
    pub prepared_save_image: Option<SaveImage>,
}

impl DurableApplicationSnapshotV4 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ApplicationError> {
        self.validate()?;
        Ok(encode_canonical_segment(
            "nextengine.application",
            "nextengine.application-durable-snapshot.v4",
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
                CanonicalField::new(4, CANONICAL_TYPE_BYTES, self.state.to_jcs_bytes()),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_OPTIONAL,
                    self.last_lifecycle
                        .as_ref()
                        .map_or_else(Vec::new, encode_last_lifecycle),
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_OPTIONAL,
                    self.close_request
                        .as_ref()
                        .map_or_else(Vec::new, CloseSessionRequestV2::canonical_bytes),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_OPTIONAL,
                    self.close_journal
                        .as_ref()
                        .map_or_else(Vec::new, CloseSessionJournalV2::canonical_bytes),
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_OPTIONAL,
                    self.close_receipt
                        .as_ref()
                        .map_or_else(Vec::new, CloseSessionReceiptV2::canonical_bytes),
                ),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_OPTIONAL,
                    self.prepared_save_image
                        .as_ref()
                        .map_or(Ok(Vec::new()), encode_save_image)?,
                ),
            ],
        )?)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, ApplicationError> {
        let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())?;
        ensure_segment(&decoded)?;
        if decode_u32(field(&decoded, 1, CANONICAL_TYPE_U32)?)? != DURABLE_SNAPSHOT_SCHEMA_VERSION {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        let manifest = ApplicationSessionManifestV2::from_jcs_bytes(
            field(&decoded, 3, CANONICAL_TYPE_BYTES)?,
            CanonicalDecodeLimits::default(),
        )?;
        let state = ApplicationSessionStateV2::from_jcs_bytes(
            field(&decoded, 4, CANONICAL_TYPE_BYTES)?,
            CanonicalDecodeLimits::default(),
        )?;
        let last_lifecycle =
            decode_optional(field(&decoded, 5, CANONICAL_TYPE_OPTIONAL)?, |bytes| {
                decode_last_lifecycle(bytes)
            })?;
        let close_request =
            decode_optional(field(&decoded, 6, CANONICAL_TYPE_OPTIONAL)?, |bytes| {
                Ok(CloseSessionRequestV2::from_canonical_bytes(
                    bytes,
                    CanonicalDecodeLimits::default(),
                )?)
            })?;
        let close_journal =
            decode_optional(field(&decoded, 7, CANONICAL_TYPE_OPTIONAL)?, |bytes| {
                Ok(CloseSessionJournalV2::from_canonical_bytes(
                    bytes,
                    CanonicalDecodeLimits::default(),
                )?)
            })?;
        let close_receipt =
            decode_optional(field(&decoded, 8, CANONICAL_TYPE_OPTIONAL)?, |bytes| {
                Ok(CloseSessionReceiptV2::from_canonical_bytes(
                    bytes,
                    CanonicalDecodeLimits::default(),
                )?)
            })?;
        let prepared_save_image = decode_optional(
            field(&decoded, 9, CANONICAL_TYPE_OPTIONAL)?,
            decode_save_image,
        )?;
        let value = Self {
            store_sequence: decode_u64(field(&decoded, 2, CANONICAL_TYPE_U64)?)?,
            manifest,
            state,
            last_lifecycle,
            close_request,
            close_journal,
            close_receipt,
            prepared_save_image,
        };
        if decoded.segment_id != value.manifest.body.session_id.to_hex() {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), ApplicationError> {
        self.manifest.validate()?;
        self.state.validate()?;
        if self.state.session_id != self.manifest.body.session_id
            || self.state.application_session_manifest_hash != self.manifest.canonical_hash
            || self.state.project_composition_lock_hash
                != self.manifest.body.project_composition_lock_hash
        {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        if let Some(request) = self.close_request.as_ref() {
            request.validate()?;
            if request.session_id != self.state.session_id {
                return Err(ApplicationError::DurableSnapshotInvalid);
            }
        }
        if let Some(journal) = self.close_journal.as_ref() {
            journal.validate()?;
            if self.close_request.as_ref().is_none_or(|request| {
                request.close_request_id != journal.close_request_id
                    || request.canonical_hash != journal.close_request_hash
            }) {
                return Err(ApplicationError::DurableSnapshotInvalid);
            }
        }
        if let Some(receipt) = self.close_receipt.as_ref() {
            receipt.validate()?;
            if receipt.session_id != self.state.session_id
                || self
                    .close_request
                    .as_ref()
                    .is_none_or(|request| request.close_request_id != receipt.close_request_id)
            {
                return Err(ApplicationError::DurableSnapshotInvalid);
            }
        }
        let prepared = self
            .close_journal
            .as_ref()
            .is_some_and(|journal| journal.stage == CloseSessionJournalStageV2::Prepared);
        if prepared != self.prepared_save_image.is_some() {
            return Err(ApplicationError::DurableSnapshotInvalid);
        }
        if let Some(image) = self.prepared_save_image.as_ref() {
            image.validate_world_light()?;
            let image_hash = image.content_hash()?;
            if self
                .close_journal
                .as_ref()
                .is_none_or(|journal| journal.save_image_hash != image_hash)
            {
                return Err(ApplicationError::DurableSnapshotInvalid);
            }
        }
        Ok(())
    }
}

fn encode_last_lifecycle(last: &LastLifecycleRecordV2) -> Vec<u8> {
    encode_canonical_segment(
        "nextengine.application",
        "nextengine.last-lifecycle.v2",
        &last.request_id.to_hex(),
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_BYTES,
                last.canonical_request_bytes.clone(),
            ),
            CanonicalField::new(2, CANONICAL_TYPE_BYTES, last.event.canonical_bytes()),
        ],
    )
    .expect("validated lifecycle records have canonical encodings")
}

fn decode_last_lifecycle(bytes: &[u8]) -> Result<LastLifecycleRecordV2, ApplicationError> {
    let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())?;
    if decoded.owner_id != "nextengine.application"
        || decoded.schema_id != "nextengine.last-lifecycle.v2"
        || decoded.fields.len() != 2
    {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    let request_bytes = field(&decoded, 1, CANONICAL_TYPE_BYTES)?.to_vec();
    let request = ApplicationLifecycleRequestV2::from_jcs_bytes(
        &request_bytes,
        CanonicalDecodeLimits::default(),
    )?;
    let event = ApplicationLifecycleEventV2::from_jcs_bytes(
        field(&decoded, 2, CANONICAL_TYPE_BYTES)?,
        &request,
        CanonicalDecodeLimits::default(),
    )?;
    if decoded.segment_id != request.request_id.to_hex() {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    Ok(LastLifecycleRecordV2 {
        request_id: request.request_id,
        canonical_request_hash: request.canonical_hash,
        canonical_request_bytes: request_bytes,
        event,
    })
}

fn encode_save_image(image: &SaveImage) -> Result<Vec<u8>, ApplicationError> {
    let mut segments = Vec::new();
    segments.extend_from_slice(
        &u32::try_from(image.segments.len())
            .map_err(|_| ApplicationError::DurableSnapshotInvalid)?
            .to_le_bytes(),
    );
    for bytes in &image.segments {
        segments.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| ApplicationError::DurableSnapshotInvalid)?
                .to_le_bytes(),
        );
        segments.extend_from_slice(bytes);
    }
    Ok(encode_canonical_segment(
        "nextengine.application",
        "nextengine.prepared-save-image.v1",
        &image.manifest.generation.to_string(),
        [
            CanonicalField::new(1, CANONICAL_TYPE_BYTES, image.manifest.to_jcs_bytes()?),
            CanonicalField::new(2, CANONICAL_TYPE_BYTES, segments),
        ],
    )?)
}

fn decode_save_image(bytes: &[u8]) -> Result<SaveImage, ApplicationError> {
    let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())?;
    if decoded.owner_id != "nextengine.application"
        || decoded.schema_id != "nextengine.prepared-save-image.v1"
        || decoded.fields.len() != 2
    {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    let manifest = next_contracts::persistence::SaveManifestV2::from_jcs_bytes(
        field(&decoded, 1, CANONICAL_TYPE_BYTES)?,
        CanonicalDecodeLimits::default(),
    )?;
    if decoded.segment_id != manifest.generation.to_string() {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    let mut bytes = field(&decoded, 2, CANONICAL_TYPE_BYTES)?;
    let count = usize::try_from(take_u32(&mut bytes)?)
        .map_err(|_| ApplicationError::DurableSnapshotInvalid)?;
    let mut segments = Vec::with_capacity(count);
    for _ in 0..count {
        let length = usize::try_from(take_u64(&mut bytes)?)
            .map_err(|_| ApplicationError::DurableSnapshotInvalid)?;
        let (segment, remainder) = bytes
            .split_at_checked(length)
            .ok_or(ApplicationError::DurableSnapshotInvalid)?;
        segments.push(segment.to_vec());
        bytes = remainder;
    }
    if !bytes.is_empty() {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    let image = SaveImage { manifest, segments };
    image.validate_world_light()?;
    Ok(image)
}

fn ensure_segment(decoded: &DecodedCanonicalSegment) -> Result<(), ApplicationError> {
    if decoded.owner_id != "nextengine.application"
        || decoded.schema_id != "nextengine.application-durable-snapshot.v4"
        || decoded.fields.len() != 9
    {
        return Err(ApplicationError::DurableSnapshotInvalid);
    }
    Ok(())
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

fn decode_optional<T>(
    bytes: &[u8],
    decode: impl FnOnce(&[u8]) -> Result<T, ApplicationError>,
) -> Result<Option<T>, ApplicationError> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        decode(bytes).map(Some)
    }
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

fn take_u32(bytes: &mut &[u8]) -> Result<u32, ApplicationError> {
    let (value, remainder) = bytes
        .split_at_checked(4)
        .ok_or(ApplicationError::DurableSnapshotInvalid)?;
    *bytes = remainder;
    decode_u32(value)
}

fn take_u64(bytes: &mut &[u8]) -> Result<u64, ApplicationError> {
    let (value, remainder) = bytes
        .split_at_checked(8)
        .ok_or(ApplicationError::DurableSnapshotInvalid)?;
    *bytes = remainder;
    decode_u64(value)
}
