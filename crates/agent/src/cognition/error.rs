use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::cognition::CognitionContractError;

#[derive(Debug)]
#[non_exhaustive]
pub enum StrategicAgentError {
    Contract(CognitionContractError),
    Population(next_contracts::world_population::WorldPopulationContractError),
    ObservationInvalid,
    StaleEpistemicView,
    NoGoalCandidate,
    SuspendedGoalLimit,
    UtilityOverflow,
    PlannerInvariant,
    RevisionExhausted,
    OwnerClosureInvalid,
    PreparedPublicationStale,
}

impl Display for StrategicAgentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "cognition contract: {error}"),
            Self::Population(error) => write!(formatter, "world population contract: {error}"),
            Self::ObservationInvalid => formatter.write_str("strategic observation is invalid"),
            Self::StaleEpistemicView => formatter.write_str("epistemic view is stale"),
            Self::NoGoalCandidate => formatter.write_str("no strategic goal candidate exists"),
            Self::SuspendedGoalLimit => formatter.write_str("suspended goal limit reached"),
            Self::UtilityOverflow => formatter.write_str("fixed-point utility overflow"),
            Self::PlannerInvariant => formatter.write_str("strategic planner invariant failed"),
            Self::RevisionExhausted => formatter.write_str("strategic owner revision exhausted"),
            Self::OwnerClosureInvalid => formatter.write_str("strategic owner closure is invalid"),
            Self::PreparedPublicationStale => {
                formatter.write_str("strategic prepared publication is stale")
            }
        }
    }
}

impl Error for StrategicAgentError {}

impl From<CognitionContractError> for StrategicAgentError {
    fn from(error: CognitionContractError) -> Self {
        Self::Contract(error)
    }
}
