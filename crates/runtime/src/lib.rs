#![forbid(unsafe_code)]

mod authority;
mod engine;
mod outcome;
mod registry;

pub use authority::{AuthorityRegistry, AuthorityRegistryError};
pub use engine::{
    CommandDisposition, CommandResult, InputAdmissionError, PhysicsLaunchOptions, RejectionCode,
    RuntimeBootstrapV3, RuntimeFatalError, RuntimeReplayDriver, RuntimeReplayError, RuntimeState,
    SnapshotRestoreError, StageTraceEntry, TickReport, TransactionStage,
    bootstrap_equipment_slot_policy_hash_v1,
};
pub use outcome::{
    NoOutcomes, OutcomeCollectionError, OutcomeContext, OutcomeProposal, OutcomeProvider,
    OutcomeSink,
};
pub use registry::{
    COMMAND_KIND_REGISTRY_VERSION, CommandKindDescriptor, CommandKindRegistry, CommandPayloadKind,
    NOOP_PRIORITY_CLASS, PHYSICAL_PRIORITY_CLASS, RPG_PRIORITY_CLASS,
};
