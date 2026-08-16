use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldCheckpointError {
    Canonicalization(CanonicalError),
    Runtime(SnapshotDecodeError),
    RpgV2(RpgContractErrorV1),
    Physics(PhysicsContractError),
    WorldStreaming(crate::world::WorldStreamingContractError),
    WorldPopulation(crate::world_population::WorldPopulationContractError),
    AgentCognition,
    CoreInteractionClosure(CoreDialogueQuestClosureError),
    ClosureMismatch,
}

impl WorldCheckpointError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::ClosureMismatch => "WORLD_CHECKPOINT_CLOSURE_CORRUPT",
            Self::Runtime(error) => error.stable_code(),
            Self::RpgV2(error) => error.stable_code(),
            Self::Physics(_) => "WORLD_CHECKPOINT_PHYSICS_CORRUPT",
            Self::WorldStreaming(error) => match error {
                crate::world::WorldStreamingContractError::UnsupportedVersion(_) => {
                    "WORLD_STREAM_SCHEMA_UNSUPPORTED"
                }
                _ => "WORLD_CHECKPOINT_STREAMING_CORRUPT",
            },
            Self::WorldPopulation(error) => error.diagnostic_code(),
            Self::AgentCognition => "AGENT_COGNITION_SNAPSHOT_INVALID",
            Self::CoreInteractionClosure(error) => error.stable_code(),
            Self::Canonicalization(_) => "WORLD_CHECKPOINT_CANONICALIZATION_FAILED",
        }
    }
}

impl Display for WorldCheckpointError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for WorldCheckpointError {}

impl From<CanonicalError> for WorldCheckpointError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<SnapshotDecodeError> for WorldCheckpointError {
    fn from(error: SnapshotDecodeError) -> Self {
        Self::Runtime(error)
    }
}

impl From<RpgContractErrorV1> for WorldCheckpointError {
    fn from(error: RpgContractErrorV1) -> Self {
        Self::RpgV2(error)
    }
}

impl From<PhysicsContractError> for WorldCheckpointError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<crate::world::WorldStreamingContractError> for WorldCheckpointError {
    fn from(error: crate::world::WorldStreamingContractError) -> Self {
        Self::WorldStreaming(error)
    }
}

impl From<crate::world_population::WorldPopulationContractError> for WorldCheckpointError {
    fn from(error: crate::world_population::WorldPopulationContractError) -> Self {
        Self::WorldPopulation(error)
    }
}

impl From<CoreDialogueQuestClosureError> for WorldCheckpointError {
    fn from(error: CoreDialogueQuestClosureError) -> Self {
        Self::CoreInteractionClosure(error)
    }
}
