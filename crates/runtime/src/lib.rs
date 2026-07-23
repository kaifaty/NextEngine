#![forbid(unsafe_code)]

mod authority;
mod engine;
mod outcome;
mod registry;

pub use authority::{AuthorityRegistry, AuthorityRegistryError};
pub use engine::{
    CommandDisposition, CommandResult, RejectionCode, RuntimeFatalError, RuntimeState,
    StageTraceEntry, TickReport, TransactionStage,
};
pub use outcome::{
    NoOutcomes, OutcomeCollectionError, OutcomeContext, OutcomeProposal, OutcomeProvider,
    OutcomeSink,
};
pub use registry::{
    COMMAND_KIND_REGISTRY_VERSION, CommandKindDescriptor, CommandKindRegistry, CommandPayloadKind,
    NOOP_PRIORITY_CLASS,
};
