use std::error::Error;
use std::fmt::{Display, Formatter};

use unicode_normalization::UnicodeNormalization;

macro_rules! opaque_id {
    ($name:ident, $length:expr) => {
        #[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; $length]);

        impl $name {
            #[must_use]
            pub const fn from_bytes(bytes: [u8; $length]) -> Self {
                Self(bytes)
            }

            #[must_use]
            pub const fn as_bytes(&self) -> &[u8; $length] {
                &self.0
            }

            #[must_use]
            pub fn to_hex(self) -> String {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let mut output = String::with_capacity($length * 2);
                for byte in self.0 {
                    output.push(char::from(HEX[usize::from(byte >> 4)]));
                    output.push(char::from(HEX[usize::from(byte & 0x0f)]));
                }
                output
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.to_hex())
                    .finish()
            }
        }
    };
}

opaque_id!(PersistentId, 16);
opaque_id!(AssetId, 16);
opaque_id!(PlayerPrincipalId, 16);
opaque_id!(CommandStreamId, 16);
opaque_id!(CommandId, 16);
opaque_id!(EventId, 16);
opaque_id!(StateRoot, 32);
opaque_id!(ContentHash, 32);
opaque_id!(CommandLedgerHash, 32);
opaque_id!(WorldNamespaceId, 16);
opaque_id!(CommandBodyHash, 32);
opaque_id!(InputSourceId, 16);
opaque_id!(PhysicsWorldId, 16);
opaque_id!(PhysicsContactId, 16);
opaque_id!(ApplicationSessionId, 16);
opaque_id!(SessionRequestId, 16);
opaque_id!(SessionTransitionId, 16);
opaque_id!(CloseRequestId, 16);

#[must_use]
pub const fn content_hash_from_bytes(bytes: [u8; 32]) -> ContentHash {
    ContentHash(bytes)
}

#[must_use]
pub const fn command_ledger_hash_from_bytes(bytes: [u8; 32]) -> CommandLedgerHash {
    CommandLedgerHash(bytes)
}

#[must_use]
pub const fn command_body_hash_from_bytes(bytes: [u8; 32]) -> CommandBodyHash {
    CommandBodyHash(bytes)
}

macro_rules! text_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
                let value = value.into();
                validate_identifier(&value)?;
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<&str> for $name {
            type Error = IdentifierError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

text_id!(SchemaId);
text_id!(ProjectId);
text_id!(CapabilityId);
text_id!(MechanicPackageId);
text_id!(PluginId);
text_id!(ScriptPrincipalId);
text_id!(ToolPrincipalId);
text_id!(SystemId);

impl PersistentId {
    #[must_use]
    pub fn derive_runtime(
        world_namespace: WorldNamespaceId,
        causal_command_id: CommandId,
        spawn_slot: u32,
        record_kind: &SchemaId,
    ) -> Self {
        let record_kind_bytes = record_kind.as_str().as_bytes();
        let record_kind_length = u32::try_from(record_kind_bytes.len())
            .expect("SchemaId construction enforces the u32 canonical length limit");
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.persistent-id.v1\0");
        preimage.extend_from_slice(world_namespace.as_bytes());
        preimage.extend_from_slice(causal_command_id.as_bytes());
        preimage.extend_from_slice(&spawn_slot.to_le_bytes());
        preimage.extend_from_slice(&record_kind_length.to_le_bytes());
        preimage.extend_from_slice(record_kind_bytes);
        let digest = crate::canonical::sha256(&preimage);
        let mut truncated = [0; 16];
        truncated.copy_from_slice(&digest[..16]);
        Self::from_bytes(truncated)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierError {
    Empty,
    ContainsNul,
    NotNfc,
    TooLong,
}

impl Display for IdentifierError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "identifier must not be empty",
            Self::ContainsNul => "identifier must not contain NUL",
            Self::NotNfc => "identifier must use Unicode NFC",
            Self::TooLong => "identifier exceeds the canonical u32 length limit",
        })
    }
}

impl Error for IdentifierError {}

pub(crate) fn validate_identifier(value: &str) -> Result<(), IdentifierError> {
    if value.is_empty() {
        return Err(IdentifierError::Empty);
    }
    if value.contains('\0') {
        return Err(IdentifierError::ContainsNul);
    }
    if value.len() > u32::MAX as usize {
        return Err(IdentifierError::TooLong);
    }
    if !value.nfc().eq(value.chars()) {
        return Err(IdentifierError::NotNfc);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CommandId, IdentifierError, PersistentId, SchemaId, WorldNamespaceId};

    #[test]
    fn nominal_id_preserves_bytes_and_formats_lowercase_hex() {
        let bytes = [0xab; 16];
        let id = PersistentId::from_bytes(bytes);
        assert_eq!(id.as_bytes(), &bytes);
        assert_eq!(id.to_hex(), "abababababababababababababababab");
    }

    #[test]
    fn text_id_requires_nonempty_nfc_without_nul() {
        assert_eq!(SchemaId::new(""), Err(IdentifierError::Empty));
        assert_eq!(
            SchemaId::new("nextengine\0schema"),
            Err(IdentifierError::ContainsNul)
        );
        assert_eq!(SchemaId::new("e\u{301}"), Err(IdentifierError::NotNfc));
        assert_eq!(
            SchemaId::new("\u{e9}")
                .expect("precomposed identifier is NFC")
                .as_str(),
            "\u{e9}"
        );
    }

    #[test]
    fn runtime_persistent_id_matches_causal_golden_vector() {
        let id = PersistentId::derive_runtime(
            WorldNamespaceId::from_bytes([1; 16]),
            CommandId::from_bytes([2; 16]),
            3,
            &SchemaId::new("npc.v1").expect("valid record kind"),
        );
        assert_eq!(id.to_hex(), "fecc70ea8f1edb8dbfc694d74af50fa6");
    }
}
