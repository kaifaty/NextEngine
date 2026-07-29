#![forbid(unsafe_code)]

mod close;
mod coordinator;
mod durable;
mod environment;
mod error;
mod launch;
pub mod replay;
mod report;

pub use close::{ApplicationCloseOutcomeV1, CloseExecutionOptionsV1, FinalSaveAttemptFailureV1};
pub use coordinator::{ApplicationCoordinator, ApplicationRunOutcomeV1};
pub use environment::default_user_state_root;
pub use error::ApplicationError;
pub use launch::{LaunchRequestV1, ProjectSelectionV1};
pub use report::{
    DiagnosticContextV1, DiagnosticReportV1, OPERATIONAL_REPORT_SCHEMA_VERSION,
    PresentationReportV1, RunReportV1,
};
