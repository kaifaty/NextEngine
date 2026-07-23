use std::error::Error;
use std::fmt::{Display, Formatter};

use sha2::{Digest, Sha256};

use crate::ids::{IdentifierError, validate_identifier};

pub const CANONICAL_BINARY_V1_MAGIC: [u8; 4] = *b"NECB";
pub const CANONICAL_BINARY_V1_VERSION: u16 = 1;
pub const CANONICAL_TYPE_BYTES: u8 = 1;
pub const CANONICAL_TYPE_U64: u8 = 2;
pub const CANONICAL_TYPE_U32: u8 = 3;
pub const CANONICAL_TYPE_U8: u8 = 4;
pub const CANONICAL_TYPE_SEQUENCE: u8 = 5;
pub const CANONICAL_TYPE_OPTIONAL: u8 = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalDecodeLimits {
    pub max_total_bytes: usize,
    pub max_identifier_bytes: usize,
    pub max_fields: usize,
    pub max_field_payload_bytes: usize,
    pub max_sequence_items: usize,
}

impl Default for CanonicalDecodeLimits {
    fn default() -> Self {
        Self {
            max_total_bytes: 16 * 1024 * 1024,
            max_identifier_bytes: 4 * 1024,
            max_fields: 4 * 1024,
            max_field_payload_bytes: 8 * 1024 * 1024,
            max_sequence_items: 1024 * 1024,
        }
    }
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CanonicalDecodeError {
    InputTooLarge {
        actual: usize,
        limit: usize,
    },
    UnexpectedEnd,
    InvalidMagic,
    UnsupportedVersion(u16),
    IdentifierTooLarge {
        actual: usize,
        limit: usize,
    },
    InvalidUtf8,
    InvalidIdentifier(IdentifierError),
    TooManyFields {
        actual: usize,
        limit: usize,
    },
    FieldPayloadTooLarge {
        field_id: u32,
        actual: usize,
        limit: usize,
    },
    LengthOverflow,
    DuplicateField(u32),
    FieldsNotStrictlySorted {
        previous: u32,
        actual: u32,
    },
    UnknownTypeTag {
        field_id: u32,
        type_tag: u8,
    },
    TrailingBytes,
}

impl Display for CanonicalDecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputTooLarge { actual, limit } => {
                write!(
                    formatter,
                    "canonical input has {actual} bytes; limit is {limit}"
                )
            }
            Self::UnexpectedEnd => formatter.write_str("canonical input ended unexpectedly"),
            Self::InvalidMagic => formatter.write_str("canonical input has invalid magic"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported canonical binary version {version}")
            }
            Self::IdentifierTooLarge { actual, limit } => {
                write!(
                    formatter,
                    "canonical identifier has {actual} bytes; limit is {limit}"
                )
            }
            Self::InvalidUtf8 => formatter.write_str("canonical identifier is not valid UTF-8"),
            Self::InvalidIdentifier(error) => write!(formatter, "invalid identifier: {error}"),
            Self::TooManyFields { actual, limit } => {
                write!(
                    formatter,
                    "canonical segment has {actual} fields; limit is {limit}"
                )
            }
            Self::FieldPayloadTooLarge {
                field_id,
                actual,
                limit,
            } => write!(
                formatter,
                "canonical field {field_id} has {actual} payload bytes; limit is {limit}"
            ),
            Self::LengthOverflow => {
                formatter.write_str("canonical length cannot be represented on this host")
            }
            Self::DuplicateField(field_id) => {
                write!(formatter, "duplicate canonical field id {field_id}")
            }
            Self::FieldsNotStrictlySorted { previous, actual } => write!(
                formatter,
                "canonical fields are not strictly sorted: {actual} follows {previous}"
            ),
            Self::UnknownTypeTag { field_id, type_tag } => {
                write!(
                    formatter,
                    "canonical field {field_id} has unknown type tag {type_tag}"
                )
            }
            Self::TrailingBytes => formatter.write_str("canonical input has trailing bytes"),
        }
    }
}

impl Error for CanonicalDecodeError {}

impl From<IdentifierError> for CanonicalDecodeError {
    fn from(error: IdentifierError) -> Self {
        Self::InvalidIdentifier(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedCanonicalSegment {
    pub owner_id: String,
    pub schema_id: String,
    pub segment_id: String,
    pub fields: Vec<CanonicalField>,
}

impl DecodedCanonicalSegment {
    #[must_use]
    pub fn field(&self, field_id: u32) -> Option<&CanonicalField> {
        self.fields
            .binary_search_by_key(&field_id, |field| field.field_id)
            .ok()
            .map(|index| &self.fields[index])
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

pub fn decode_canonical_segment(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<DecodedCanonicalSegment, CanonicalDecodeError> {
    if bytes.len() > limits.max_total_bytes {
        return Err(CanonicalDecodeError::InputTooLarge {
            actual: bytes.len(),
            limit: limits.max_total_bytes,
        });
    }

    let mut cursor = CanonicalCursor::new(bytes);
    if cursor.read_exact(CANONICAL_BINARY_V1_MAGIC.len())? != CANONICAL_BINARY_V1_MAGIC {
        return Err(CanonicalDecodeError::InvalidMagic);
    }
    let version = cursor.read_u16()?;
    if version != CANONICAL_BINARY_V1_VERSION {
        return Err(CanonicalDecodeError::UnsupportedVersion(version));
    }
    let owner_id = cursor.read_identifier(limits.max_identifier_bytes)?;
    let schema_id = cursor.read_identifier(limits.max_identifier_bytes)?;
    let segment_id = cursor.read_identifier(limits.max_identifier_bytes)?;
    let field_count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(field_count);
    let mut previous = None;
    for _ in 0..field_count {
        let field_id = cursor.read_u32()?;
        if let Some(previous) = previous {
            if field_id == previous {
                return Err(CanonicalDecodeError::DuplicateField(field_id));
            }
            if field_id < previous {
                return Err(CanonicalDecodeError::FieldsNotStrictlySorted {
                    previous,
                    actual: field_id,
                });
            }
        }
        previous = Some(field_id);

        let type_tag = cursor.read_u8()?;
        if !is_known_type_tag(type_tag) {
            return Err(CanonicalDecodeError::UnknownTypeTag { field_id, type_tag });
        }
        let payload_length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| CanonicalDecodeError::LengthOverflow)?;
        if payload_length > limits.max_field_payload_bytes {
            return Err(CanonicalDecodeError::FieldPayloadTooLarge {
                field_id,
                actual: payload_length,
                limit: limits.max_field_payload_bytes,
            });
        }
        let payload = cursor.read_exact(payload_length)?.to_vec();
        fields.push(CanonicalField::new(field_id, type_tag, payload));
    }
    cursor.finish()?;
    Ok(DecodedCanonicalSegment {
        owner_id,
        schema_id,
        segment_id,
        fields,
    })
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

pub(crate) struct CanonicalCursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> CanonicalCursor<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub(crate) fn read_exact(&mut self, length: usize) -> Result<&'a [u8], CanonicalDecodeError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(CanonicalDecodeError::LengthOverflow)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(CanonicalDecodeError::UnexpectedEnd)?;
        self.position = end;
        Ok(value)
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8, CanonicalDecodeError> {
        Ok(self.read_exact(1)?[0])
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16, CanonicalDecodeError> {
        let bytes: [u8; 2] = self
            .read_exact(2)?
            .try_into()
            .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
        Ok(u16::from_le_bytes(bytes))
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32, CanonicalDecodeError> {
        let bytes: [u8; 4] = self
            .read_exact(4)?
            .try_into()
            .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64, CanonicalDecodeError> {
        let bytes: [u8; 8] = self
            .read_exact(8)?
            .try_into()
            .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
        Ok(u64::from_le_bytes(bytes))
    }

    pub(crate) fn read_count(
        &mut self,
        limit: usize,
        too_many: impl FnOnce(usize, usize) -> CanonicalDecodeError,
    ) -> Result<usize, CanonicalDecodeError> {
        let count =
            usize::try_from(self.read_u32()?).map_err(|_| CanonicalDecodeError::LengthOverflow)?;
        if count > limit {
            return Err(too_many(count, limit));
        }
        Ok(count)
    }

    pub(crate) fn read_u32_length_prefixed(
        &mut self,
        limit: usize,
    ) -> Result<&'a [u8], CanonicalDecodeError> {
        let length =
            usize::try_from(self.read_u32()?).map_err(|_| CanonicalDecodeError::LengthOverflow)?;
        if length > limit {
            return Err(CanonicalDecodeError::IdentifierTooLarge {
                actual: length,
                limit,
            });
        }
        self.read_exact(length)
    }

    pub(crate) fn finish(self) -> Result<(), CanonicalDecodeError> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err(CanonicalDecodeError::TrailingBytes)
        }
    }

    fn read_identifier(&mut self, limit: usize) -> Result<String, CanonicalDecodeError> {
        let bytes = self.read_u32_length_prefixed(limit)?;
        let value = std::str::from_utf8(bytes).map_err(|_| CanonicalDecodeError::InvalidUtf8)?;
        validate_identifier(value)?;
        Ok(value.to_owned())
    }
}

const fn is_known_type_tag(type_tag: u8) -> bool {
    matches!(
        type_tag,
        CANONICAL_TYPE_BYTES
            | CANONICAL_TYPE_U64
            | CANONICAL_TYPE_U32
            | CANONICAL_TYPE_U8
            | CANONICAL_TYPE_SEQUENCE
            | CANONICAL_TYPE_OPTIONAL
    )
}

#[cfg(test)]
mod tests {
    use super::{
        CANONICAL_BINARY_V1_MAGIC, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError,
        CanonicalField, decode_canonical_segment, encode_canonical_segment, sha256,
    };

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
    fn canonical_segment_round_trips_through_bounded_decoder() {
        let encoded = encode_canonical_segment(
            "runtime",
            "nextengine.test",
            "fixture",
            [
                CanonicalField::new(1, 1, vec![1]),
                CanonicalField::new(2, 2, 7_u64.to_le_bytes().to_vec()),
            ],
        )
        .expect("valid canonical segment");
        let decoded = decode_canonical_segment(&encoded, CanonicalDecodeLimits::default())
            .expect("encoded segment decodes");

        assert_eq!(decoded.owner_id, "runtime");
        assert_eq!(decoded.schema_id, "nextengine.test");
        assert_eq!(decoded.segment_id, "fixture");
        assert_eq!(
            decoded.field(2).map(|field| field.payload.as_slice()),
            Some(7_u64.to_le_bytes().as_slice())
        );
    }

    #[test]
    fn decoder_rejects_truncation_unknown_type_and_trailing_bytes() {
        let encoded = encode_canonical_segment(
            "runtime",
            "nextengine.test",
            "fixture",
            [CanonicalField::new(1, 1, vec![1])],
        )
        .expect("valid canonical segment");

        assert_eq!(
            decode_canonical_segment(
                &encoded[..encoded.len() - 1],
                CanonicalDecodeLimits::default()
            ),
            Err(CanonicalDecodeError::UnexpectedEnd)
        );

        let mut unknown_type = encoded.clone();
        let type_offset = 4 + 2 + (4 + 7) + (4 + 15) + (4 + 7) + 4 + 4;
        unknown_type[type_offset] = 0xff;
        assert_eq!(
            decode_canonical_segment(&unknown_type, CanonicalDecodeLimits::default()),
            Err(CanonicalDecodeError::UnknownTypeTag {
                field_id: 1,
                type_tag: 0xff,
            })
        );

        let mut trailing = encoded;
        trailing.push(0);
        assert_eq!(
            decode_canonical_segment(&trailing, CanonicalDecodeLimits::default()),
            Err(CanonicalDecodeError::TrailingBytes)
        );
    }

    #[test]
    fn decoder_rejects_wrong_magic_and_oversized_input_before_parsing() {
        let mut wrong_magic = CANONICAL_BINARY_V1_MAGIC.to_vec();
        wrong_magic[0] = b'X';
        wrong_magic.extend_from_slice(&1_u16.to_le_bytes());
        assert_eq!(
            decode_canonical_segment(&wrong_magic, CanonicalDecodeLimits::default()),
            Err(CanonicalDecodeError::InvalidMagic)
        );

        let limits = CanonicalDecodeLimits {
            max_total_bytes: 1,
            ..CanonicalDecodeLimits::default()
        };
        assert_eq!(
            decode_canonical_segment(&wrong_magic, limits),
            Err(CanonicalDecodeError::InputTooLarge {
                actual: wrong_magic.len(),
                limit: 1,
            })
        );
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
