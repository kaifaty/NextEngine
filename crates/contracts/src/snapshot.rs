use crate::canonical::{
    CanonicalError, CanonicalField, encode_canonical_segment, extend_u32_length_prefixed,
};
use crate::{CommandId, CommandStreamId, IssuerPrincipal};

const TYPE_U64: u8 = 2;
const TYPE_SEQUENCE: u8 = 5;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandLedgerSnapshot {
    pub stream_id: CommandStreamId,
    pub issuer: IssuerPrincipal,
    pub last_sequence: u64,
    pub command_id: CommandId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSnapshot {
    pub next_tick: u64,
    pub committed_event_count: u64,
    pub command_ledgers: Vec<CommandLedgerSnapshot>,
}

impl RuntimeSnapshot {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut ledgers = self.command_ledgers.clone();
        ledgers.sort();
        let mut ledger_bytes = Vec::new();
        ledger_bytes.extend_from_slice(
            &u32::try_from(ledgers.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        for ledger in ledgers {
            ledger_bytes.extend_from_slice(ledger.stream_id.as_bytes());
            let principal = ledger.issuer.canonical_bytes()?;
            extend_u32_length_prefixed(&mut ledger_bytes, &principal)?;
            ledger_bytes.extend_from_slice(&ledger.last_sequence.to_le_bytes());
            ledger_bytes.extend_from_slice(ledger.command_id.as_bytes());
        }

        encode_canonical_segment(
            "runtime",
            "nextengine.runtime.snapshot",
            "command-ledger",
            [
                CanonicalField::new(1, TYPE_U64, self.next_tick.to_le_bytes().to_vec()),
                CanonicalField::new(
                    2,
                    TYPE_U64,
                    self.committed_event_count.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(3, TYPE_SEQUENCE, ledger_bytes),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CommandId, CommandLedgerSnapshot, CommandStreamId, IssuerPrincipal, PlayerPrincipalId,
        RuntimeSnapshot,
    };

    #[test]
    fn ledger_order_does_not_change_snapshot_bytes() {
        let first = CommandLedgerSnapshot {
            stream_id: CommandStreamId::from_bytes([1; 16]),
            issuer: IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([1; 16])),
            last_sequence: 4,
            command_id: CommandId::from_bytes([1; 16]),
        };
        let second = CommandLedgerSnapshot {
            stream_id: CommandStreamId::from_bytes([2; 16]),
            issuer: IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16])),
            last_sequence: 5,
            command_id: CommandId::from_bytes([2; 16]),
        };
        let ordered = RuntimeSnapshot {
            next_tick: 6,
            committed_event_count: 2,
            command_ledgers: vec![first.clone(), second.clone()],
        };
        let reversed = RuntimeSnapshot {
            command_ledgers: vec![second, first],
            ..ordered.clone()
        };
        assert_eq!(
            ordered.canonical_bytes().expect("canonical snapshot"),
            reversed.canonical_bytes().expect("canonical snapshot")
        );
    }
}
