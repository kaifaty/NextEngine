use std::error::Error;
use std::fmt::{Display, Formatter};

use sha2::{Digest, Sha256};

use crate::ids::{IdentifierError, validate_identifier};

pub const CANONICAL_BINARY_V1_MAGIC: [u8; 4] = *b"NECB";
pub const CANONICAL_BINARY_V1_VERSION: u16 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalField {
    pub field_id: u32,
    pub type_tag: u8,
    pub payload: Vec<u8>,
}

impl CanonicalField {
    #[must_use]
    pub fn new(field_id: u32, type_tag: u8, payload: Vec<u8>) -> Self {
        Self {
            field_id,
            type_tag,
            payload,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalError {
    InvalidIdentifier(IdentifierError),
    LengthOverflow,
    DuplicateField(u32),
    DuplicateSequenceValue,
}

impl Display for CanonicalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdentifier(error) => write!(formatter, "invalid identifier: {error}"),
            Self::LengthOverflow => formatter.write_str("canonical length exceeds its field width"),
            Self::DuplicateField(field_id) => {
                write!(formatter, "duplicate canonical field id {field_id}")
            }
            Self::DuplicateSequenceValue => {
                formatter.write_str("duplicate value in canonical set-like sequence")
            }
        }
    }
}

impl Error for CanonicalError {}

impl From<IdentifierError> for CanonicalError {
    fn from(error: IdentifierError) -> Self {
        Self::InvalidIdentifier(error)
    }
}

pub fn encode_canonical_segment(
    owner_id: &str,
    schema_id: &str,
    segment_id: &str,
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    validate_identifier(owner_id)?;
    validate_identifier(schema_id)?;
    validate_identifier(segment_id)?;

    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    for pair in fields.windows(2) {
        if pair[0].field_id == pair[1].field_id {
            return Err(CanonicalError::DuplicateField(pair[0].field_id));
        }
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&CANONICAL_BINARY_V1_MAGIC);
    bytes.extend_from_slice(&CANONICAL_BINARY_V1_VERSION.to_le_bytes());
    extend_u32_length_prefixed(&mut bytes, owner_id.as_bytes())?;
    extend_u32_length_prefixed(&mut bytes, schema_id.as_bytes())?;
    extend_u32_length_prefixed(&mut bytes, segment_id.as_bytes())?;
    bytes.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for field in fields {
        bytes.extend_from_slice(&field.field_id.to_le_bytes());
        bytes.push(field.type_tag);
        bytes.extend_from_slice(
            &u64::try_from(field.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&field.payload);
    }
    Ok(bytes)
}

#[must_use]
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub(crate) fn extend_u32_length_prefixed(
    target: &mut Vec<u8>,
    value: &[u8],
) -> Result<(), CanonicalError> {
    target.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    target.extend_from_slice(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CanonicalError, CanonicalField, encode_canonical_segment, sha256};

    #[test]
    fn segment_fields_are_sorted_by_stable_field_id() {
        let encoded = encode_canonical_segment(
            "runtime",
            "nextengine.test",
            "fixture",
            [
                CanonicalField::new(2, 1, vec![2]),
                CanonicalField::new(1, 1, vec![1]),
            ],
        )
        .expect("valid canonical segment");

        let first_field_offset = 4 + 2 + (4 + 7) + (4 + 15) + (4 + 7) + 4;
        assert_eq!(
            &encoded[first_field_offset..first_field_offset + 4],
            &1_u32.to_le_bytes()
        );
    }

    #[test]
    fn duplicate_field_is_rejected() {
        let error = encode_canonical_segment(
            "runtime",
            "nextengine.test",
            "fixture",
            [
                CanonicalField::new(1, 1, vec![1]),
                CanonicalField::new(1, 1, vec![2]),
            ],
        )
        .expect_err("duplicate fields must fail");
        assert_eq!(error, CanonicalError::DuplicateField(1));
    }

    #[test]
    fn sha256_matches_standard_empty_vector() {
        assert_eq!(
            hex(sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    fn hex(bytes: [u8; 32]) -> String {
        bytes
            .into_iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}
