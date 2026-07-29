use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::sha256;
use crate::ids::{AssetId, ContentHash, content_hash_from_bytes};
use crate::manifest_jcs::JcsValue;
use crate::persistence::ManifestCodecError;

use super::{
    PROJECT_CATALOG_FORMAT_V1, PROJECT_COMPOSITION_LOCK_FORMAT_V2,
    SCHEMA_REGISTRY_MANIFEST_FORMAT_V1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProjectContractError {
    Manifest(ManifestCodecError),
    Identifier(crate::ids::IdentifierError),
    ZeroRevision,
    DuplicateIdentity,
    LimitExceeded { actual: usize, limit: usize },
    HashMismatch,
    NonCanonical,
    UnknownClosedValue,
    InvalidSemanticVersion,
    MissingSchema,
    MissingReference,
    DependencyCycle,
    InvalidText,
}

impl Display for ProjectContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manifest(error) => write!(formatter, "project manifest is invalid: {error}"),
            Self::Identifier(error) => write!(formatter, "project identifier is invalid: {error}"),
            Self::ZeroRevision => formatter.write_str("project revision/version must be positive"),
            Self::DuplicateIdentity => formatter.write_str("project identity is duplicated"),
            Self::LimitExceeded { actual, limit } => {
                write!(formatter, "project count {actual} exceeds limit {limit}")
            }
            Self::HashMismatch => formatter.write_str("project artifact hash mismatch"),
            Self::NonCanonical => formatter.write_str("project artifact is not canonical"),
            Self::UnknownClosedValue => formatter.write_str("unknown closed project value"),
            Self::InvalidSemanticVersion => formatter.write_str("invalid semantic version"),
            Self::MissingSchema => formatter.write_str("schema reference is absent from registry"),
            Self::MissingReference => formatter.write_str("content reference is absent"),
            Self::DependencyCycle => formatter.write_str("required content dependency cycle"),
            Self::InvalidText => formatter.write_str("content text is not bounded NFC text"),
        }
    }
}

impl Error for ProjectContractError {}

impl From<ManifestCodecError> for ProjectContractError {
    fn from(error: ManifestCodecError) -> Self {
        Self::Manifest(error)
    }
}

impl From<crate::ids::IdentifierError> for ProjectContractError {
    fn from(error: crate::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

#[must_use]
pub fn domain_hash(domain: &str, bytes: &[u8]) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .expect("in-memory manifest byte length fits u64")
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    content_hash_from_bytes(sha256(&preimage))
}

#[must_use]
pub fn canonical_empty_manifest_hash(domain: &str) -> ContentHash {
    domain_hash(domain, b"{}")
}

pub(super) fn plain_jcs_hash(bytes: &[u8]) -> ContentHash {
    content_hash_from_bytes(sha256(bytes))
}

pub(super) fn string(value: impl Into<String>) -> JcsValue {
    JcsValue::String(value.into())
}

pub(super) fn object(
    value: JcsValue,
    field: &'static str,
) -> Result<BTreeMap<String, JcsValue>, ProjectContractError> {
    match value {
        JcsValue::Object(value) => Ok(value),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an object",
        }
        .into()),
    }
}

pub(super) fn array(
    value: JcsValue,
    field: &'static str,
) -> Result<Vec<JcsValue>, ProjectContractError> {
    match value {
        JcsValue::Array(value) => Ok(value),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an array",
        }
        .into()),
    }
}

pub(super) fn text(value: JcsValue, field: &'static str) -> Result<String, ProjectContractError> {
    match value {
        JcsValue::String(value) => Ok(value),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "a string",
        }
        .into()),
    }
}

pub(super) fn take(
    object: &mut BTreeMap<String, JcsValue>,
    field: &'static str,
) -> Result<JcsValue, ProjectContractError> {
    object
        .remove(field)
        .ok_or_else(|| ManifestCodecError::MissingField(field.to_owned()).into())
}

pub(super) fn reject_unknown(
    object: BTreeMap<String, JcsValue>,
) -> Result<(), ProjectContractError> {
    if let Some((field, _)) = object.into_iter().next() {
        Err(ManifestCodecError::UnknownField(field).into())
    } else {
        Ok(())
    }
}

pub(super) fn expect_format(
    object: &mut BTreeMap<String, JcsValue>,
    expected: &'static str,
) -> Result<(), ProjectContractError> {
    let field = if expected == PROJECT_CATALOG_FORMAT_V1 {
        "catalog_format"
    } else if expected == PROJECT_COMPOSITION_LOCK_FORMAT_V2 {
        "lock_format"
    } else if expected == SCHEMA_REGISTRY_MANIFEST_FORMAT_V1 {
        "manifest_schema"
    } else {
        "manifest_format"
    };
    if text(take(object, field)?, field)? != expected {
        return Err(ProjectContractError::UnknownClosedValue);
    }
    Ok(())
}

pub(super) fn u32_number(
    value: JcsValue,
    field: &'static str,
) -> Result<u32, ProjectContractError> {
    match value {
        JcsValue::Number(value) => u32::try_from(value)
            .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()).into()),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an unsigned JSON integer",
        }
        .into()),
    }
}

pub(super) fn u64_text(value: JcsValue, field: &'static str) -> Result<u64, ProjectContractError> {
    let value = text(value, field)?;
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ManifestCodecError::InvalidInteger(field.to_owned()).into());
    }
    value
        .parse()
        .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()).into())
}

pub(super) fn hash(
    value: JcsValue,
    field: &'static str,
) -> Result<ContentHash, ProjectContractError> {
    let value = text(value, field)?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ManifestCodecError::InvalidHex(field.to_owned()).into());
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
    }
    Ok(content_hash_from_bytes(bytes))
}

pub(super) fn asset_id(
    value: JcsValue,
    field: &'static str,
) -> Result<AssetId, ProjectContractError> {
    let value = text(value, field)?;
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ManifestCodecError::InvalidHex(field.to_owned()).into());
    }
    let mut bytes = [0_u8; 16];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
    }
    Ok(AssetId::from_bytes(bytes))
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}

pub(super) fn ensure_unique<T: Ord>(
    values: impl IntoIterator<Item = T>,
) -> Result<(), ProjectContractError> {
    let values: Vec<_> = values.into_iter().collect();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ProjectContractError::DuplicateIdentity);
    }
    Ok(())
}

pub(super) fn enforce_limit(actual: usize, limit: usize) -> Result<(), ProjectContractError> {
    if actual > limit {
        Err(ProjectContractError::LimitExceeded { actual, limit })
    } else {
        Ok(())
    }
}
