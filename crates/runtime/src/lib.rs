#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{CONTRACT_SCHEMA_VERSION, DomainEvent, EventPayload, WorldCommand};

#[derive(Debug, Default)]
pub struct RuntimeState {
    tick: u64,
    last_sequence: Option<u64>,
}

impl RuntimeState {
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    pub fn apply(&mut self, command: &WorldCommand) -> Result<DomainEvent, ApplyError> {
        if command.schema_version != CONTRACT_SCHEMA_VERSION {
            return Err(ApplyError::SchemaMismatch {
                expected: CONTRACT_SCHEMA_VERSION,
                actual: command.schema_version,
            });
        }

        let expected = self.last_sequence.map_or(0, |sequence| sequence + 1);
        if command.sequence != expected {
            return Err(ApplyError::SequenceMismatch {
                expected,
                actual: command.sequence,
            });
        }

        self.tick += 1;
        self.last_sequence = Some(command.sequence);
        Ok(DomainEvent {
            schema_version: CONTRACT_SCHEMA_VERSION,
            tick: self.tick,
            payload: EventPayload::CommandAccepted {
                command_sequence: command.sequence,
            },
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplyError {
    SchemaMismatch { expected: u32, actual: u32 },
    SequenceMismatch { expected: u64, actual: u64 },
}

impl Display for ApplyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaMismatch { expected, actual } => {
                write!(
                    formatter,
                    "schema mismatch: expected {expected}, got {actual}"
                )
            }
            Self::SequenceMismatch { expected, actual } => {
                write!(
                    formatter,
                    "sequence mismatch: expected {expected}, got {actual}"
                )
            }
        }
    }
}

impl Error for ApplyError {}

#[cfg(test)]
mod tests {
    use next_contracts::{CONTRACT_SCHEMA_VERSION, CommandPayload, PersistentId, WorldCommand};

    use super::{ApplyError, RuntimeState};

    fn command(sequence: u64) -> WorldCommand {
        WorldCommand {
            schema_version: CONTRACT_SCHEMA_VERSION,
            sequence,
            issuer: PersistentId::default(),
            payload: CommandPayload::Noop,
        }
    }

    #[test]
    fn accepts_ordered_commands() {
        let mut runtime = RuntimeState::default();
        let event = runtime.apply(&command(0)).expect("valid command");
        assert_eq!(event.tick, 1);
        assert_eq!(runtime.tick(), 1);
    }

    #[test]
    fn rejects_out_of_order_command_without_mutation() {
        let mut runtime = RuntimeState::default();
        assert_eq!(
            runtime.apply(&command(2)),
            Err(ApplyError::SequenceMismatch {
                expected: 0,
                actual: 2,
            })
        );
        assert_eq!(runtime.tick(), 0);
    }
}
