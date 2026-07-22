#![forbid(unsafe_code)]

pub const CONTRACT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct PersistentId([u8; 16]);

impl PersistentId {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct AssetId([u8; 16]);

impl AssetId {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCommand {
    pub schema_version: u32,
    pub sequence: u64,
    pub issuer: PersistentId,
    pub payload: CommandPayload,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandPayload {
    Noop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainEvent {
    pub schema_version: u32,
    pub tick: u64,
    pub payload: EventPayload,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventPayload {
    CommandAccepted { command_sequence: u64 },
}

#[cfg(test)]
mod tests {
    use super::{AssetId, PersistentId};

    #[test]
    fn nominal_ids_preserve_bytes() {
        let bytes = [7; 16];
        assert_eq!(PersistentId::from_bytes(bytes).as_bytes(), &bytes);
        assert_eq!(AssetId::from_bytes(bytes).as_bytes(), &bytes);
    }
}
