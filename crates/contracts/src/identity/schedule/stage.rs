use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum RuntimeStageId {
    InputIngest = 1,
    CandidateAuthentication = 2,
    IngressValidationAndPlan = 3,
    IngressAdmission = 4,
    IngressCommit = 5,
    WorldStreamingCommit = 6,
    AgentPlanning = 7,
    PhysicalStep = 8,
    OutcomeCommit = 9,
    ResidencyCommit = 10,
    StateHash = 11,
    SnapshotPublication = 12,
}

impl RuntimeStageId {
    pub(super) fn from_tag(tag: u8) -> Result<Self, IdentityContractError> {
        match tag {
            1 => Ok(Self::InputIngest),
            2 => Ok(Self::CandidateAuthentication),
            3 => Ok(Self::IngressValidationAndPlan),
            4 => Ok(Self::IngressAdmission),
            5 => Ok(Self::IngressCommit),
            6 => Ok(Self::WorldStreamingCommit),
            7 => Ok(Self::AgentPlanning),
            8 => Ok(Self::PhysicalStep),
            9 => Ok(Self::OutcomeCommit),
            10 => Ok(Self::ResidencyCommit),
            11 => Ok(Self::StateHash),
            12 => Ok(Self::SnapshotPublication),
            value => Err(IdentityContractError::UnknownTag(value)),
        }
    }
}
