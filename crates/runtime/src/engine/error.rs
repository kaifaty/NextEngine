use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CanonicalError, CommandLedgerError, CoreDialogueQuestClosureError, IdentityContractError,
    InputContractError, MechanicsContractError, PhysicsContractError, RpgContractErrorV1,
    SnapshotDecodeError, WorldCheckpointError,
};
use next_mechanics::MechanicsHostError;
use next_physics_api::PhysicsBackendError;
use next_rpg::RpgStateError;

use crate::outcome::OutcomeCollectionError;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InputAdmissionError {
    Canonical(CanonicalError),
    Contract(InputContractError),
    PrincipalUnauthenticated,
    SourceUnbound,
    ResourceLimit,
}

impl InputAdmissionError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Canonical(_) | Self::Contract(_) => "INPUT_SAMPLE_INVALID",
            Self::PrincipalUnauthenticated => "INPUT_PRINCIPAL_UNAUTHENTICATED",
            Self::SourceUnbound => "INPUT_SOURCE_UNBOUND",
            Self::ResourceLimit => "INPUT_RESOURCE_LIMIT",
        }
    }
}

impl Display for InputAdmissionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for InputAdmissionError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeFatalError {
    TickExhausted,
    EventCountExhausted,
    RevisionExhausted,
    TraceCountExhausted,
    IngressGenerationExhausted,
    IngressCheckpointCorrupt,
    Input(InputContractError),
    OutcomeCollection(OutcomeCollectionError),
    InternalCanonicalization(CanonicalError),
    LedgerCorrupt(CommandLedgerError),
    Physics(PhysicsBackendError),
    PhysicalOutcomeInvariant,
    InternalIdentityCollision,
    CoreInteractionClosure(CoreDialogueQuestClosureError),
    Mechanics(MechanicsHostError),
    Snapshot(SnapshotDecodeError),
}

impl RuntimeFatalError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::TickExhausted => "SIMULATION_TICK_EXHAUSTED",
            Self::EventCountExhausted => "DOMAIN_EVENT_COUNT_EXHAUSTED",
            Self::RevisionExhausted => "AUTHORITATIVE_REVISION_EXHAUSTED",
            Self::TraceCountExhausted => "STAGE_TRACE_COUNT_EXHAUSTED",
            Self::IngressGenerationExhausted => "INGRESS_QUEUE_GENERATION_EXHAUSTED",
            Self::IngressCheckpointCorrupt => "INGRESS_CHECKPOINT_CORRUPT",
            Self::Input(_) => "INGRESS_CONTRACT_CORRUPT",
            Self::OutcomeCollection(error) => error.stable_code(),
            Self::InternalCanonicalization(_) => "INTERNAL_CANONICALIZATION_FAILED",
            Self::LedgerCorrupt(_) => "COMMAND_LEDGER_CORRUPT",
            Self::Physics(error) => error.stable_code(),
            Self::PhysicalOutcomeInvariant => "PHYSICAL_OUTCOME_INVARIANT_FAILED",
            Self::InternalIdentityCollision => "INTERNAL_IDENTITY_COLLISION",
            Self::CoreInteractionClosure(error) => error.stable_code(),
            Self::Mechanics(_) => "MECHANICS_HOST_INVARIANT_FAILED",
            Self::Snapshot(_) => "RUNTIME_SNAPSHOT_CLOSURE_CORRUPT",
        }
    }
}

impl Display for RuntimeFatalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.stable_code())
    }
}

impl Error for RuntimeFatalError {}

impl From<CanonicalError> for RuntimeFatalError {
    fn from(error: CanonicalError) -> Self {
        Self::InternalCanonicalization(error)
    }
}

impl From<CommandLedgerError> for RuntimeFatalError {
    fn from(error: CommandLedgerError) -> Self {
        Self::LedgerCorrupt(error)
    }
}

impl From<InputContractError> for RuntimeFatalError {
    fn from(error: InputContractError) -> Self {
        Self::Input(error)
    }
}

impl From<PhysicsBackendError> for RuntimeFatalError {
    fn from(error: PhysicsBackendError) -> Self {
        Self::Physics(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SnapshotRestoreError {
    Canonicalization(CanonicalError),
    Decode(SnapshotDecodeError),
    Identity(IdentityContractError),
    Input(InputContractError),
    Physics(PhysicsContractError),
    PhysicsController(PhysicsBackendError),
    WorldCheckpoint(WorldCheckpointError),
    Ledger(CommandLedgerError),
    RpgContract(RpgContractErrorV1),
    RpgState(RpgStateError),
    RpgDefinitions(MechanicsContractError),
    BootstrapClosureMismatch,
    CommandRegistryMismatch,
    InactivePrincipal,
    IdentityCollision,
    ControllerClosureMismatch,
    CoreInteractionClosure(CoreDialogueQuestClosureError),
}

impl Display for SnapshotRestoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonicalization(error) => write!(formatter, "canonicalization failed: {error}"),
            Self::Decode(error) => write!(formatter, "snapshot validation failed: {error}"),
            Self::Identity(error) => write!(formatter, "identity validation failed: {error}"),
            Self::Input(error) => write!(formatter, "input contract validation failed: {error}"),
            Self::Physics(error) => {
                write!(formatter, "physical contract validation failed: {error}")
            }
            Self::PhysicsController(error) => {
                write!(formatter, "physical controller validation failed: {error}")
            }
            Self::WorldCheckpoint(error) => {
                write!(formatter, "world checkpoint validation failed: {error}")
            }
            Self::Ledger(error) => write!(formatter, "ledger validation failed: {error}"),
            Self::RpgContract(error) => write!(formatter, "RPG contract failed: {error}"),
            Self::RpgState(error) => write!(formatter, "RPG snapshot state failed: {error}"),
            Self::RpgDefinitions(error) => {
                write!(formatter, "RPG definition registry failed: {error}")
            }
            Self::BootstrapClosureMismatch => {
                formatter.write_str("runtime bootstrap world/profile closure does not match")
            }
            Self::CommandRegistryMismatch => {
                formatter.write_str("runtime command registry hash does not match")
            }
            Self::InactivePrincipal => {
                formatter.write_str("runtime bootstrap authority or stream principal is inactive")
            }
            Self::IdentityCollision => formatter.write_str("runtime bootstrap identity collision"),
            Self::ControllerClosureMismatch => {
                formatter.write_str("runtime controller registry closure does not match")
            }
            Self::CoreInteractionClosure(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for SnapshotRestoreError {}

impl From<CanonicalError> for SnapshotRestoreError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<SnapshotDecodeError> for SnapshotRestoreError {
    fn from(error: SnapshotDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentityContractError> for SnapshotRestoreError {
    fn from(error: IdentityContractError) -> Self {
        Self::Identity(error)
    }
}

impl From<InputContractError> for SnapshotRestoreError {
    fn from(error: InputContractError) -> Self {
        Self::Input(error)
    }
}

impl From<PhysicsContractError> for SnapshotRestoreError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<PhysicsBackendError> for SnapshotRestoreError {
    fn from(error: PhysicsBackendError) -> Self {
        Self::PhysicsController(error)
    }
}

impl From<WorldCheckpointError> for SnapshotRestoreError {
    fn from(error: WorldCheckpointError) -> Self {
        Self::WorldCheckpoint(error)
    }
}

impl From<CommandLedgerError> for SnapshotRestoreError {
    fn from(error: CommandLedgerError) -> Self {
        Self::Ledger(error)
    }
}

impl From<RpgContractErrorV1> for SnapshotRestoreError {
    fn from(error: RpgContractErrorV1) -> Self {
        Self::RpgContract(error)
    }
}

impl From<MechanicsContractError> for SnapshotRestoreError {
    fn from(error: MechanicsContractError) -> Self {
        Self::RpgDefinitions(error)
    }
}

impl From<RpgStateError> for SnapshotRestoreError {
    fn from(error: RpgStateError) -> Self {
        Self::RpgState(error)
    }
}

impl From<CoreDialogueQuestClosureError> for SnapshotRestoreError {
    fn from(error: CoreDialogueQuestClosureError) -> Self {
        Self::CoreInteractionClosure(error)
    }
}
