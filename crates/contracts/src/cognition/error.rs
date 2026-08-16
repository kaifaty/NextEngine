use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CognitionContractError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    ContentInvalid,
    BeliefInvalid,
    ViewInvalid,
    GoalInvalid,
    AffordanceInvalid,
    PlanInvalid,
    IntentInvalid,
    SnapshotInvalid,
    TraceInvalid,
    CommandInvalid,
    EventInvalid,
    RevisionExhausted,
    LimitExceeded,
    MissingField(u32),
    UnknownField(u32),
    FieldType,
    WrongEnvelope,
    UnknownTag(u8),
    NonCanonicalEncoding,
}

impl CognitionContractError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::RevisionExhausted => "AGENT_COGNITION_REVISION_EXHAUSTED",
            Self::LimitExceeded => "AGENT_COGNITION_LIMIT_EXCEEDED",
            _ => "AGENT_COGNITION_CONTRACT_INVALID",
        }
    }
}

impl Display for CognitionContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for CognitionContractError {}

impl From<CanonicalError> for CognitionContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for CognitionContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for CognitionContractError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

pub fn candidate_order(left: &GoalCandidateV1, right: &GoalCandidateV1) -> std::cmp::Ordering {
    right
        .priority_band
        .cmp(&left.priority_band)
        .then_with(|| right.total_utility_q16.cmp(&left.total_utility_q16))
        .then_with(|| left.goal_id.cmp(&right.goal_id))
        .then_with(|| left.target_id_or_none.cmp(&right.target_id_or_none))
}
