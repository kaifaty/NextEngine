#![forbid(unsafe_code)]

mod authority;
mod engine;
mod outcome;
mod registry;
mod session;
mod stage_zone;

pub use authority::{AuthorityRegistry, AuthorityRegistryError};
pub use engine::{
    CommandDisposition, CommandResult, InputAdmissionError, PhysicsLaunchOptions,
    PreparedRuntimeTick, PreparedRuntimeWorldServicesTickV1, PreparedRuntimeWorldTick,
    RejectionCode, RuntimeBootstrapV4, RuntimeFatalError, RuntimeReplayDriver, RuntimeReplayError,
    RuntimeState, RuntimeTickPreparation, SnapshotRestoreError, StageTraceEntry, TickReport,
    TransactionStage, ValidatedRuntimeTick, ValidatedRuntimeWorldServicesTickV1,
    ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1, ValidatedRuntimeWorldTick,
    WorldServicesTickCommitV1, bootstrap_equipment_slot_policy_hash_v1,
};
pub use outcome::{
    NoOutcomes, OutcomeCollectionError, OutcomeContext, OutcomeProposal, OutcomeProvider,
    OutcomeSink,
};
pub use registry::{
    CommandKindDescriptor, CommandKindRegistry, NOOP_PRIORITY_CLASS, PHYSICAL_PRIORITY_CLASS,
    RPG_PRIORITY_CLASS,
};
pub use session::{
    ApplicationSessionMachine, LastLifecycleRecordV2, SessionMachineError,
    SessionStatePublicationPlanV1, SessionTransitionPlanV1, SessionTransitionReferencesV1,
};
