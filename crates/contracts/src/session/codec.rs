use std::collections::BTreeMap;

use crate::canonical::{CanonicalDecodeLimits, sha256};
use crate::ids::{
    ApplicationSessionId, CloseRequestId, ContentHash, SessionRequestId, SessionTransitionId,
    content_hash_from_bytes,
};
use crate::manifest_jcs::{JcsValue, decode_canonical_jcs, encode_canonical_jcs};
use crate::persistence::ManifestCodecError;

use super::SessionContractError;

pub(crate) fn session_hash(domain: &str, body: &JcsValue) -> ContentHash {
    let bytes = encode_canonical_jcs(body);
    let mut preimage = Vec::with_capacity(domain.len() + 1 + bytes.len());
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(&bytes);
    content_hash_from_bytes(sha256(&preimage))
}

pub(crate) fn encoded(body: &JcsValue) -> Vec<u8> {
    encode_canonical_jcs(body)
}

pub(crate) fn decoded_object(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    field: &'static str,
) -> Result<BTreeMap<String, JcsValue>, SessionContractError> {
    require_object(decode_canonical_jcs(bytes, limits)?, field)
}

pub(crate) fn object<const N: usize>(entries: [(&str, JcsValue); N]) -> JcsValue {
    JcsValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

fn require_object(
    value: JcsValue,
    field: &'static str,
) -> Result<BTreeMap<String, JcsValue>, SessionContractError> {
    match value {
        JcsValue::Object(value) => Ok(value),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an object",
        }
        .into()),
    }
}

pub(crate) fn nested_object(
    value: JcsValue,
    field: &'static str,
) -> Result<BTreeMap<String, JcsValue>, SessionContractError> {
    require_object(value, field)
}

pub(crate) fn string(value: impl Into<String>) -> JcsValue {
    JcsValue::String(value.into())
}

pub(crate) fn number(value: impl Into<u64>) -> JcsValue {
    JcsValue::Number(value.into())
}

pub(crate) fn boolean(value: bool) -> JcsValue {
    string(if value { "true" } else { "false" })
}

pub(crate) fn optional_hash(value: Option<ContentHash>) -> JcsValue {
    value.map_or_else(|| string("none"), |value| string(value.to_hex()))
}

pub(crate) fn take(
    object: &mut BTreeMap<String, JcsValue>,
    field: &'static str,
) -> Result<JcsValue, SessionContractError> {
    object
        .remove(field)
        .ok_or_else(|| ManifestCodecError::MissingField(field.to_owned()).into())
}

pub(crate) fn reject_unknown(
    object: BTreeMap<String, JcsValue>,
) -> Result<(), SessionContractError> {
    if let Some((field, _)) = object.into_iter().next() {
        Err(ManifestCodecError::UnknownField(field).into())
    } else {
        Ok(())
    }
}

pub(crate) fn text(value: JcsValue, field: &'static str) -> Result<String, SessionContractError> {
    match value {
        JcsValue::String(value) => Ok(value),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "a string",
        }
        .into()),
    }
}

pub(crate) fn u32_value(value: JcsValue, field: &'static str) -> Result<u32, SessionContractError> {
    match value {
        JcsValue::Number(value) => u32::try_from(value)
            .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()).into()),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an unsigned integer",
        }
        .into()),
    }
}

pub(crate) fn u64_value(value: JcsValue, field: &'static str) -> Result<u64, SessionContractError> {
    match value {
        JcsValue::Number(value) => Ok(value),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an unsigned integer",
        }
        .into()),
    }
}

pub(crate) fn hash(
    value: JcsValue,
    field: &'static str,
) -> Result<ContentHash, SessionContractError> {
    let bytes = hex::<32>(&text(value, field)?, field)?;
    Ok(ContentHash::from_bytes(bytes))
}

pub(crate) fn optional_hash_value(
    value: JcsValue,
    field: &'static str,
) -> Result<Option<ContentHash>, SessionContractError> {
    let value = text(value, field)?;
    if value == "none" {
        Ok(None)
    } else {
        Ok(Some(ContentHash::from_bytes(hex::<32>(&value, field)?)))
    }
}

pub(crate) fn session_id(
    value: JcsValue,
    field: &'static str,
) -> Result<ApplicationSessionId, SessionContractError> {
    Ok(ApplicationSessionId::from_bytes(hex::<16>(
        &text(value, field)?,
        field,
    )?))
}

pub(crate) fn request_id(
    value: JcsValue,
    field: &'static str,
) -> Result<SessionRequestId, SessionContractError> {
    Ok(SessionRequestId::from_bytes(hex::<16>(
        &text(value, field)?,
        field,
    )?))
}

pub(crate) fn close_request_id(
    value: JcsValue,
    field: &'static str,
) -> Result<CloseRequestId, SessionContractError> {
    Ok(CloseRequestId::from_bytes(hex::<16>(
        &text(value, field)?,
        field,
    )?))
}

pub(crate) fn transition_id(
    value: JcsValue,
    field: &'static str,
) -> Result<SessionTransitionId, SessionContractError> {
    Ok(SessionTransitionId::from_bytes(hex::<16>(
        &text(value, field)?,
        field,
    )?))
}

pub(crate) fn optional_transition_id(
    value: JcsValue,
    field: &'static str,
) -> Result<Option<SessionTransitionId>, SessionContractError> {
    let value = text(value, field)?;
    if value == "none" {
        Ok(None)
    } else {
        Ok(Some(SessionTransitionId::from_bytes(hex::<16>(
            &value, field,
        )?)))
    }
}

fn hex<const N: usize>(value: &str, field: &'static str) -> Result<[u8; N], SessionContractError> {
    if value.len() != N * 2
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ManifestCodecError::InvalidHex(field.to_owned()).into());
    }
    let mut bytes = [0_u8; N];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    Ok(bytes)
}

fn nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => unreachable!("hex validated before conversion"),
    }
}
