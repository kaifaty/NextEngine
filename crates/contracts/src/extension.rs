use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{CanonicalDecodeLimits, sha256};
use crate::ids::{
    CapabilityId, ContentHash, MechanicPackageId, PluginId, SchemaId, content_hash_from_bytes,
};

pub const EXTENSION_PACKAGE_STATE_SCHEMA_VERSION: u32 = 1;
pub const EXTENSION_CIRCUIT_THRESHOLD: usize = 3;
pub const EXTENSION_CIRCUIT_WINDOW_TICKS: u64 = 1_800;
pub const EXTENSION_MAX_CAPABILITIES: usize = 128;
pub const EXTENSION_MAX_STATE_BYTES: usize = 1_048_576;
pub const WASM_PLUGIN_STATE_SCHEMA_VERSION: u32 = 1;
pub const WASM_HOST_CURRENT_API_MAJOR: u16 = 3;
pub const WASM_HOST_PREVIOUS_API_MAJOR: u16 = 2;
pub const WASM_DEFAULT_LINEAR_MEMORY_BYTES: u64 = 64 * 1_024 * 1_024;
pub const WASM_DEFAULT_TABLE_LIMIT: u32 = 1;
pub const WASM_DEFAULT_INSTANCE_LIMIT: u32 = 1;
pub const WASM_DEFAULT_FUEL_PER_CALL: u64 = 10_000_000;
pub const WASM_DEFAULT_FUEL_PER_TICK: u64 = 20_000_000;
const WASM_MAX_SOURCE_METADATA_BYTES: usize = 4_096;
const STATE_PREFIX: &[u8] = b"nextengine.extension-package-state.v1\0";
const WASM_STATE_PREFIX: &[u8] = b"nextengine.wasm-plugin-state.v1\0";

mod manifest;
pub use manifest::*;
mod state;
pub use state::*;

fn extend_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), ExtensionContractError> {
    bytes.extend_from_slice(
        &u32::try_from(count)
            .map_err(|_| ExtensionContractError::LimitExceeded)?
            .to_le_bytes(),
    );
    Ok(())
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), ExtensionContractError> {
    extend_bytes(bytes, value.as_bytes())
}

fn extend_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), ExtensionContractError> {
    extend_count(bytes, value.len())?;
    bytes.extend_from_slice(value);
    Ok(())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], ExtensionContractError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(ExtensionContractError::DecodeLimit)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(ExtensionContractError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], ExtensionContractError> {
        self.read_exact(N)?
            .try_into()
            .map_err(|_| ExtensionContractError::Truncated)
    }

    fn read_u8(&mut self) -> Result<u8, ExtensionContractError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32, ExtensionContractError> {
        Ok(u32::from_le_bytes(self.read_array()?))
    }

    fn read_u64(&mut self) -> Result<u64, ExtensionContractError> {
        Ok(u64::from_le_bytes(self.read_array()?))
    }

    fn read_count(&mut self, maximum: usize) -> Result<usize, ExtensionContractError> {
        let count =
            usize::try_from(self.read_u32()?).map_err(|_| ExtensionContractError::DecodeLimit)?;
        if count > maximum {
            return Err(ExtensionContractError::DecodeLimit);
        }
        Ok(count)
    }

    fn read_bytes(&mut self, maximum: usize) -> Result<Vec<u8>, ExtensionContractError> {
        let length = self.read_count(maximum)?;
        Ok(self.read_exact(length)?.to_vec())
    }

    fn read_text(&mut self, maximum: usize) -> Result<String, ExtensionContractError> {
        String::from_utf8(self.read_bytes(maximum)?)
            .map_err(|_| ExtensionContractError::InvalidState)
    }

    fn finish(self) -> Result<(), ExtensionContractError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(ExtensionContractError::NonCanonical)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExtensionContractError {
    InvalidBudget,
    InvalidManifest,
    InvalidState,
    StateLimitExceeded,
    LimitExceeded,
    DecodeLimit,
    WrongEnvelope,
    UnsupportedVersion,
    Truncated,
    NonCanonical,
    IncompatibleInterface,
    InvalidHandle,
}

impl Display for ExtensionContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidBudget => "extension budget policy is invalid",
            Self::InvalidManifest => "extension package manifest is invalid",
            Self::InvalidState => "extension package state is invalid",
            Self::StateLimitExceeded => "extension package state exceeds its limit",
            Self::LimitExceeded => "extension contract limit exceeded",
            Self::DecodeLimit => "extension decode limit exceeded",
            Self::WrongEnvelope => "extension state envelope is invalid",
            Self::UnsupportedVersion => "extension state version is unsupported",
            Self::Truncated => "extension state is truncated",
            Self::NonCanonical => "extension state encoding is non-canonical",
            Self::IncompatibleInterface => "WIT interface is outside the supported N/N-1 matrix",
            Self::InvalidHandle => "WIT resource handle is invalid or forged",
        })
    }
}

impl Error for ExtensionContractError {}

#[cfg(test)]
mod tests;
