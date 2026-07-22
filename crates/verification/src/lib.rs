#![forbid(unsafe_code)]

use next_contracts::{DomainEvent, EventPayload};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerificationRecord {
    pub tick: u64,
    pub event_hash: u64,
}

impl VerificationRecord {
    #[must_use]
    pub fn from_event(event: &DomainEvent) -> Self {
        let mut bytes = Vec::with_capacity(20);
        bytes.extend_from_slice(&event.schema_version.to_le_bytes());
        bytes.extend_from_slice(&event.tick.to_le_bytes());
        match event.payload {
            EventPayload::CommandAccepted { command_sequence } => {
                bytes.extend_from_slice(&command_sequence.to_le_bytes());
            }
        }
        Self {
            tick: event.tick,
            event_hash: stable_hash(&bytes),
        }
    }
}

#[must_use]
pub fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use next_contracts::{CONTRACT_SCHEMA_VERSION, DomainEvent, EventPayload};

    use super::{VerificationRecord, stable_hash};

    #[test]
    fn hashing_is_stable() {
        assert_eq!(stable_hash(b"next-engine"), 17_648_770_222_346_673_729);
    }

    #[test]
    fn record_is_derived_from_immutable_event() {
        let event = DomainEvent {
            schema_version: CONTRACT_SCHEMA_VERSION,
            tick: 1,
            payload: EventPayload::CommandAccepted {
                command_sequence: 0,
            },
        };
        assert_eq!(VerificationRecord::from_event(&event).tick, 1);
    }
}
