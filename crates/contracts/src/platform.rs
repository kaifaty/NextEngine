use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::ids::{CapabilityId, ContentHash, PersistentId, SchemaId};
use crate::manifest_jcs::{JcsValue, encode_canonical_jcs};
use crate::project::domain_hash;

pub const PLATFORM_CAPABILITY_SET_SCHEMA_VERSION: u32 = 1;
pub const PLATFORM_TIMEBASE_SCHEMA_VERSION: u32 = 1;
pub const PLATFORM_EVENT_SCHEMA_VERSION: u32 = 1;
pub const NORMALIZED_CONTROL_EVENT_SCHEMA_VERSION: u32 = 1;
pub const PLATFORM_MAX_CAPABILITIES: usize = 128;
pub const PLATFORM_MAX_CONTROL_COMPONENTS: usize = 8;
pub const PLATFORM_MAX_MODIFIERS: usize = 16;

mod capability;
pub use capability::*;
mod event;
pub use event::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PlatformContractError {
    UnsupportedVersion,
    DuplicateIdentity,
    NonCanonicalOrder,
    LimitExceeded { actual: usize, limit: usize },
    HashMismatch,
    InvalidTimebase,
    KindPayloadMismatch,
}

impl PlatformContractError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion | Self::KindPayloadMismatch => "PLATFORM_EVENT_SCHEMA_INVALID",
            Self::DuplicateIdentity | Self::HashMismatch => "PLATFORM_EVENT_IDENTITY_COLLISION",
            Self::NonCanonicalOrder => "PLATFORM_EVENT_SEQUENCE_GAP",
            Self::InvalidTimebase => "PLATFORM_TIMEBASE_INVALID",
            Self::LimitExceeded { .. } => "PLATFORM_EVENT_SCHEMA_INVALID",
        }
    }
}

impl Display for PlatformContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion => formatter.write_str("platform schema version unsupported"),
            Self::DuplicateIdentity => formatter.write_str("platform identity duplicated"),
            Self::NonCanonicalOrder => formatter.write_str("platform collection is not canonical"),
            Self::LimitExceeded { actual, limit } => {
                write!(formatter, "platform limit exceeded: {actual} > {limit}")
            }
            Self::HashMismatch => formatter.write_str("platform canonical hash mismatch"),
            Self::InvalidTimebase => formatter.write_str("platform timebase invalid"),
            Self::KindPayloadMismatch => {
                formatter.write_str("platform event kind/payload mismatch")
            }
        }
    }
}

impl Error for PlatformContractError {}

fn enforce_unique<T: Ord>(values: &[T]) -> Result<(), PlatformContractError> {
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(PlatformContractError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn enforce_capability_identities_unique(
    values: &[NormalizedPlatformCapabilityV1],
) -> Result<(), PlatformContractError> {
    if values
        .windows(2)
        .any(|pair| pair[0].capability_id == pair[1].capability_id)
    {
        Err(PlatformContractError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn enforce_canonical<T: Ord>(values: &[T]) -> Result<(), PlatformContractError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        Err(PlatformContractError::NonCanonicalOrder)
    } else {
        Ok(())
    }
}

fn enforce_limit(actual: usize, limit: usize) -> Result<(), PlatformContractError> {
    if actual > limit {
        Err(PlatformContractError::LimitExceeded { actual, limit })
    } else {
        Ok(())
    }
}

fn object<const N: usize>(entries: [(&str, JcsValue); N]) -> JcsValue {
    JcsValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn string(value: impl Into<String>) -> JcsValue {
    JcsValue::String(value.into())
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[usize::from(*byte >> 4)]));
        value.push(char::from(HEX[usize::from(*byte & 0x0f)]));
    }
    value
}

#[cfg(test)]
mod tests;
