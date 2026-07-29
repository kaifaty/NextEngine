use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_U32, CanonicalCursor,
    CanonicalDecodeLimits, DecodedCanonicalSegment, sha256,
};
use crate::ids::{AssetId, ContentHash, SchemaId, content_hash_from_bytes};
use crate::manifest_jcs::{JcsValue, encode_canonical_jcs};
use crate::project::{AssetRevisionRefV1, SchemaEncodingV1, SchemaRefV1, SchemaRoleV1};

use super::{
    RENDER_CONTENT_OWNER_ID, RENDER_CONTENT_SCHEMA_VERSION, RENDER_CONTENT_SEGMENT_ID,
    RenderContentContractError,
};

pub(super) fn validate_schema_ref(
    schema_ref: &SchemaRefV1,
    expected_schema_id: &str,
) -> Result<(), RenderContentContractError> {
    schema_ref.validate()?;
    if schema_ref.schema_id.as_str() != expected_schema_id
        || schema_ref.schema_version != RENDER_CONTENT_SCHEMA_VERSION
        || schema_ref.role != SchemaRoleV1::NeutralContent
        || schema_ref.encoding != SchemaEncodingV1::CanonicalBinaryV1
    {
        return Err(RenderContentContractError::SchemaMismatch);
    }
    ensure_nonzero_hash(schema_ref.descriptor_sha256)?;
    Ok(())
}

pub(super) fn validate_envelope(
    segment: &DecodedCanonicalSegment,
    expected_schema_id: &str,
    expected_field_count: usize,
) -> Result<(), RenderContentContractError> {
    if segment.owner_id != RENDER_CONTENT_OWNER_ID
        || segment.schema_id != expected_schema_id
        || segment.segment_id != RENDER_CONTENT_SEGMENT_ID
        || segment.fields.len() != expected_field_count
    {
        return Err(RenderContentContractError::EnvelopeMismatch);
    }
    let version = read_u32(field(segment, 1, CANONICAL_TYPE_U32)?)?;
    if version != RENDER_CONTENT_SCHEMA_VERSION {
        return Err(RenderContentContractError::UnsupportedVersion(version));
    }
    Ok(())
}

pub(super) fn schema_ref_from_segment(
    segment: &DecodedCanonicalSegment,
    descriptor_field_id: u32,
) -> Result<SchemaRefV1, RenderContentContractError> {
    let schema_ref = SchemaRefV1 {
        schema_id: SchemaId::new(segment.schema_id.clone())?,
        schema_version: read_u32(field(segment, 1, CANONICAL_TYPE_U32)?)?,
        descriptor_sha256: ContentHash::from_bytes(read_array(field(
            segment,
            descriptor_field_id,
            CANONICAL_TYPE_HASH256,
        )?)?),
        role: SchemaRoleV1::NeutralContent,
        encoding: SchemaEncodingV1::CanonicalBinaryV1,
    };
    validate_schema_ref(&schema_ref, &segment.schema_id)?;
    Ok(schema_ref)
}

pub(super) fn asset_id_from_segment(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<AssetId, RenderContentContractError> {
    Ok(AssetId::from_bytes(read_array(field(
        segment,
        field_id,
        CANONICAL_TYPE_ID128,
    )?)?))
}

pub(super) fn field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<&[u8], RenderContentContractError> {
    let value = segment
        .field(field_id)
        .ok_or(RenderContentContractError::InvalidPayload)?;
    if value.type_tag != expected_type {
        return Err(RenderContentContractError::WrongFieldType(field_id));
    }
    Ok(&value.payload)
}

pub(super) fn read_u8(bytes: &[u8]) -> Result<u8, RenderContentContractError> {
    bytes
        .first()
        .copied()
        .filter(|_| bytes.len() == 1)
        .ok_or(RenderContentContractError::InvalidPayload)
}

pub(super) fn read_u16(bytes: &[u8]) -> Result<u16, RenderContentContractError> {
    Ok(u16::from_le_bytes(read_array(bytes)?))
}

pub(super) fn read_u32(bytes: &[u8]) -> Result<u32, RenderContentContractError> {
    Ok(u32::from_le_bytes(read_array(bytes)?))
}

pub(super) fn read_u64(bytes: &[u8]) -> Result<u64, RenderContentContractError> {
    Ok(u64::from_le_bytes(read_array(bytes)?))
}

pub(super) fn read_i32(bytes: &[u8]) -> Result<i32, RenderContentContractError> {
    Ok(i32::from_le_bytes(read_array(bytes)?))
}

pub(super) fn read_array<const LENGTH: usize>(
    bytes: &[u8],
) -> Result<[u8; LENGTH], RenderContentContractError> {
    bytes
        .try_into()
        .map_err(|_| RenderContentContractError::InvalidPayload)
}

pub(super) fn ensure_nonzero_hash(hash: ContentHash) -> Result<(), RenderContentContractError> {
    if hash == ContentHash::default() {
        Err(RenderContentContractError::ZeroHash)
    } else {
        Ok(())
    }
}

pub(super) fn ensure_limit(actual: usize, limit: usize) -> Result<(), RenderContentContractError> {
    if actual > limit {
        Err(RenderContentContractError::LimitExceeded { actual, limit })
    } else {
        Ok(())
    }
}

pub(super) fn ensure_unique<T: Ord>(values: &[T]) -> Result<(), RenderContentContractError> {
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(RenderContentContractError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

pub(super) fn extend_count(
    target: &mut Vec<u8>,
    count: usize,
) -> Result<(), RenderContentContractError> {
    target.extend_from_slice(
        &u32::try_from(count)
            .map_err(|_| RenderContentContractError::IntegerOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

pub(super) fn extend_sized(
    target: &mut Vec<u8>,
    value: &[u8],
) -> Result<(), RenderContentContractError> {
    target.extend_from_slice(
        &u64::try_from(value.len())
            .map_err(|_| RenderContentContractError::IntegerOverflow)?
            .to_le_bytes(),
    );
    target.extend_from_slice(value);
    Ok(())
}

pub(super) fn read_count(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
    contract_limit: usize,
) -> Result<usize, RenderContentContractError> {
    let count = usize::try_from(cursor.read_u32()?)
        .map_err(|_| RenderContentContractError::IntegerOverflow)?;
    ensure_limit(count, limits.max_sequence_items.min(contract_limit))?;
    Ok(count)
}

pub(super) fn read_sized<'a>(
    cursor: &mut CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
) -> Result<&'a [u8], RenderContentContractError> {
    let length = usize::try_from(cursor.read_u64()?)
        .map_err(|_| RenderContentContractError::IntegerOverflow)?;
    ensure_limit(length, limits.max_field_payload_bytes)?;
    Ok(cursor.read_exact(length)?)
}

pub(super) fn encode_asset_revision(reference: AssetRevisionRefV1) -> [u8; 48] {
    let mut bytes = [0_u8; 48];
    bytes[..16].copy_from_slice(reference.asset_id.as_bytes());
    bytes[16..].copy_from_slice(reference.record_sha256.as_bytes());
    bytes
}

pub(super) fn decode_asset_revision(
    bytes: &[u8],
) -> Result<AssetRevisionRefV1, RenderContentContractError> {
    if bytes.len() != 48 {
        return Err(RenderContentContractError::InvalidPayload);
    }
    let reference = AssetRevisionRefV1 {
        asset_id: AssetId::from_bytes(read_array(&bytes[..16])?),
        record_sha256: ContentHash::from_bytes(read_array(&bytes[16..])?),
    };
    ensure_nonzero_hash(reference.record_sha256)?;
    Ok(reference)
}

pub(super) fn neutral_record_hash(
    schema_ref: &SchemaRefV1,
    record_bytes: &[u8],
) -> Result<ContentHash, RenderContentContractError> {
    let schema_ref_bytes = encode_canonical_jcs(&schema_ref_value(schema_ref));
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.neutral-record.v1\0");
    preimage.extend_from_slice(
        &u32::try_from(schema_ref_bytes.len())
            .map_err(|_| RenderContentContractError::IntegerOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(&schema_ref_bytes);
    preimage.extend_from_slice(
        &u64::try_from(record_bytes.len())
            .map_err(|_| RenderContentContractError::IntegerOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(record_bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

pub(super) fn domain_hash(
    domain: &str,
    bytes: &[u8],
) -> Result<ContentHash, RenderContentContractError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| RenderContentContractError::IntegerOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn schema_ref_value(schema_ref: &SchemaRefV1) -> JcsValue {
    let mut object = BTreeMap::new();
    object.insert(
        "descriptor_sha256".to_owned(),
        JcsValue::String(schema_ref.descriptor_sha256.to_hex()),
    );
    object.insert(
        "encoding".to_owned(),
        JcsValue::String("canonical-binary-v1".to_owned()),
    );
    object.insert(
        "role".to_owned(),
        JcsValue::String("neutral-content".to_owned()),
    );
    object.insert(
        "schema_id".to_owned(),
        JcsValue::String(schema_ref.schema_id.as_str().to_owned()),
    );
    object.insert(
        "schema_version".to_owned(),
        JcsValue::Number(u64::from(schema_ref.schema_version)),
    );
    JcsValue::Object(object)
}
