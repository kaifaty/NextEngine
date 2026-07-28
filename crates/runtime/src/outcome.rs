use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CanonicalError, CommandPhase, CommandStreamId, DomainEvent, IssuerPrincipal, RpgCommandV1,
    SystemId, WorldCommand,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum OutcomePayload {
    Noop,
    Rpg(RpgCommandV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutcomeProposal {
    system_id: SystemId,
    stream_id: CommandStreamId,
    sequence: u64,
    precondition_revision: Option<u64>,
    payload: OutcomePayload,
}

impl OutcomeProposal {
    #[must_use]
    pub const fn noop(system_id: SystemId, stream_id: CommandStreamId, sequence: u64) -> Self {
        Self {
            system_id,
            stream_id,
            sequence,
            precondition_revision: None,
            payload: OutcomePayload::Noop,
        }
    }

    #[must_use]
    pub const fn rpg(
        system_id: SystemId,
        stream_id: CommandStreamId,
        sequence: u64,
        payload: RpgCommandV1,
    ) -> Self {
        Self {
            system_id,
            stream_id,
            sequence,
            precondition_revision: None,
            payload: OutcomePayload::Rpg(payload),
        }
    }

    #[must_use]
    pub const fn with_precondition_revision(mut self, revision: u64) -> Self {
        self.precondition_revision = Some(revision);
        self
    }

    pub(crate) fn into_command(self, tick: u64) -> Result<WorldCommand, CanonicalError> {
        let issuer = IssuerPrincipal::InternalSystem(self.system_id);
        let mut command = match self.payload {
            OutcomePayload::Noop => {
                WorldCommand::noop(self.stream_id, issuer, self.sequence, tick)?
            }
            OutcomePayload::Rpg(payload) => {
                WorldCommand::rpg(self.stream_id, issuer, self.sequence, tick, payload)?
            }
        };
        command.phase = CommandPhase::Outcome;
        command.set_precondition_revision(self.precondition_revision)?;
        Ok(command)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OutcomeContext<'a> {
    pub tick: u64,
    pub authoritative_revision: u64,
    pub ingress_events: &'a [DomainEvent],
}

pub trait OutcomeProvider {
    fn collect(
        &mut self,
        context: OutcomeContext<'_>,
        sink: &mut OutcomeSink,
    ) -> Result<(), OutcomeCollectionError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoOutcomes;

impl OutcomeProvider for NoOutcomes {
    fn collect(
        &mut self,
        _context: OutcomeContext<'_>,
        _sink: &mut OutcomeSink,
    ) -> Result<(), OutcomeCollectionError> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct OutcomeSink {
    proposals: Vec<OutcomeProposal>,
}

impl OutcomeSink {
    pub(crate) const fn new() -> Self {
        Self {
            proposals: Vec::new(),
        }
    }

    pub fn submit(&mut self, proposal: OutcomeProposal) {
        self.proposals.push(proposal);
    }

    pub fn enter_same_tick_batch(&mut self) -> Result<(), OutcomeCollectionError> {
        Err(OutcomeCollectionError::OutcomeReentryForbidden)
    }

    pub(crate) fn into_proposals(self) -> Vec<OutcomeProposal> {
        self.proposals
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeCollectionError {
    OutcomeReentryForbidden,
}

impl OutcomeCollectionError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::OutcomeReentryForbidden => "OUTCOME_REENTRY_FORBIDDEN",
        }
    }
}

impl Display for OutcomeCollectionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for OutcomeCollectionError {}

#[cfg(test)]
mod tests {
    use super::{OutcomeCollectionError, OutcomeSink};

    #[test]
    fn closed_outcome_sink_rejects_nested_same_tick_batch() {
        let mut sink = OutcomeSink::new();
        assert_eq!(
            sink.enter_same_tick_batch(),
            Err(OutcomeCollectionError::OutcomeReentryForbidden)
        );
    }
}
