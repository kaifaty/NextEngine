use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_ID128, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_UTF8_NFC, CanonicalCursor,
    CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError,
};
use crate::ids::{
    MechanicPackageId, PersistentId, PlayerPrincipalId, PluginId, ScriptPrincipalId, SystemId,
    ToolPrincipalId,
};

use super::codec::{encode_nested_value, read_nested_value};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IssuerPrincipal {
    Player(PlayerPrincipalId),
    Agent(PersistentId),
    Package(MechanicPackageId),
    Script(ScriptPrincipalId),
    Plugin(PluginId),
    Tool(ToolPrincipalId),
    InternalSystem(SystemId),
}

pub type IssuerPrincipalV2 = IssuerPrincipal;

impl IssuerPrincipal {
    #[must_use]
    pub const fn tag(&self) -> u8 {
        match self {
            Self::Player(_) => 0x01,
            Self::Agent(_) => 0x02,
            Self::Package(_) => 0x03,
            Self::Script(_) => 0x04,
            Self::Plugin(_) => 0x05,
            Self::Tool(_) => 0x06,
            Self::InternalSystem(_) => 0x07,
        }
    }

    #[must_use]
    pub fn identifier_bytes(&self) -> &[u8] {
        match self {
            Self::Player(id) => id.as_bytes(),
            Self::Agent(id) => id.as_bytes(),
            Self::Package(id) => id.as_str().as_bytes(),
            Self::Script(id) => id.as_str().as_bytes(),
            Self::Plugin(id) => id.as_str().as_bytes(),
            Self::Tool(id) => id.as_str().as_bytes(),
            Self::InternalSystem(id) => id.as_str().as_bytes(),
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_nested_value(CANONICAL_TYPE_TAGGED_UNION, &self.tagged_union_payload()?)
    }

    pub(super) fn tagged_union_payload(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut payload = vec![self.tag()];
        let nested = match self {
            Self::Player(id) => encode_nested_value(CANONICAL_TYPE_ID128, id.as_bytes())?,
            Self::Agent(id) => encode_nested_value(CANONICAL_TYPE_ID128, id.as_bytes())?,
            Self::Package(id) => {
                encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?
            }
            Self::Script(id) => {
                encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?
            }
            Self::Plugin(id) => {
                encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?
            }
            Self::Tool(id) => encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?,
            Self::InternalSystem(id) => {
                encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?
            }
        };
        payload.extend_from_slice(&nested);
        Ok(payload)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PrincipalDecodeError> {
        if bytes.len() > limits.max_field_payload_bytes {
            return Err(PrincipalDecodeError::InputTooLarge {
                actual: bytes.len(),
                limit: limits.max_field_payload_bytes,
            });
        }
        let mut cursor = CanonicalCursor::new(bytes);
        let (type_tag, payload) = read_nested_value(&mut cursor, limits)?;
        cursor.finish()?;
        if type_tag != CANONICAL_TYPE_TAGGED_UNION {
            return Err(PrincipalDecodeError::WrongType(type_tag));
        }
        Self::from_tagged_union_payload(payload, limits)
    }

    pub(super) fn from_tagged_union_payload(
        payload: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PrincipalDecodeError> {
        let mut cursor = CanonicalCursor::new(payload);
        let tag = cursor.read_u8()?;
        let (type_tag, nested) = read_nested_value(&mut cursor, limits)?;
        cursor.finish()?;
        let principal = match tag {
            0x01 => Self::Player(PlayerPrincipalId::from_bytes(decode_id128(
                type_tag, nested,
            )?)),
            0x02 => Self::Agent(PersistentId::from_bytes(decode_id128(type_tag, nested)?)),
            0x03 => Self::Package(MechanicPackageId::new(decode_text(type_tag, nested)?)?),
            0x04 => Self::Script(ScriptPrincipalId::new(decode_text(type_tag, nested)?)?),
            0x05 => Self::Plugin(PluginId::new(decode_text(type_tag, nested)?)?),
            0x06 => Self::Tool(ToolPrincipalId::new(decode_text(type_tag, nested)?)?),
            0x07 => Self::InternalSystem(SystemId::new(decode_text(type_tag, nested)?)?),
            _ => return Err(PrincipalDecodeError::UnknownTag(tag)),
        };
        Ok(principal)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PrincipalDecodeError {
    InputTooLarge { actual: usize, limit: usize },
    Canonical(CanonicalDecodeError),
    Identifier(crate::IdentifierError),
    WrongType(u8),
    InvalidNestedType { expected: u8, actual: u8 },
    InvalidNestedLength { expected: usize, actual: usize },
    UnknownTag(u8),
}

impl Display for PrincipalDecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputTooLarge { actual, limit } => {
                write!(formatter, "principal has {actual} bytes; limit is {limit}")
            }
            Self::Canonical(error) => write!(formatter, "principal encoding is invalid: {error}"),
            Self::Identifier(error) => {
                write!(formatter, "principal identifier is invalid: {error}")
            }
            Self::WrongType(type_tag) => {
                write!(formatter, "principal has canonical type {type_tag:#04x}")
            }
            Self::InvalidNestedType { expected, actual } => write!(
                formatter,
                "principal nested value has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::InvalidNestedLength { expected, actual } => write!(
                formatter,
                "principal nested value has {actual} bytes; expected {expected}"
            ),
            Self::UnknownTag(tag) => write!(formatter, "unknown issuer principal tag {tag}"),
        }
    }
}

impl Error for PrincipalDecodeError {}

impl From<CanonicalDecodeError> for PrincipalDecodeError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<crate::IdentifierError> for PrincipalDecodeError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

fn decode_id128(type_tag: u8, payload: &[u8]) -> Result<[u8; 16], PrincipalDecodeError> {
    if type_tag != CANONICAL_TYPE_ID128 {
        return Err(PrincipalDecodeError::InvalidNestedType {
            expected: CANONICAL_TYPE_ID128,
            actual: type_tag,
        });
    }
    payload
        .try_into()
        .map_err(|_| PrincipalDecodeError::InvalidNestedLength {
            expected: 16,
            actual: payload.len(),
        })
}

fn decode_text(type_tag: u8, payload: &[u8]) -> Result<&str, PrincipalDecodeError> {
    if type_tag != CANONICAL_TYPE_UTF8_NFC {
        return Err(PrincipalDecodeError::InvalidNestedType {
            expected: CANONICAL_TYPE_UTF8_NFC,
            actual: type_tag,
        });
    }
    std::str::from_utf8(payload)
        .map_err(|_| PrincipalDecodeError::Canonical(CanonicalDecodeError::InvalidUtf8))
}
