#![forbid(unsafe_code)]

mod close;
mod coordinator;
mod durable;
mod environment;
mod error;
mod interactive_worker;
mod launch;
mod live_schedule;
mod preferences;
pub mod replay;
mod report;

pub use close::{ApplicationCloseOutcomeV1, CloseExecutionOptionsV1, FinalSaveAttemptFailureV1};
pub use coordinator::{ApplicationAudioFrameV1, ApplicationCoordinator, ApplicationRunOutcomeV1};
pub use environment::default_user_state_root;
pub use error::ApplicationError;
pub use interactive_worker::{
    INTERACTIVE_SIMULATION_QUEUE_CAPACITY, InteractiveMainSnapshotReadV1, InteractiveMainSubmitV1,
    InteractiveSimulationWorkerV1, InteractiveWorkerDiagnosticMetricsV1,
    InteractiveWorkerDiagnosticOptionsV1, InteractiveWorkerDiagnosticReportV1,
    InteractiveWorkerFailureV1, InteractiveWorkerFinalizationV1, InteractiveWorkerFixedStepClassV1,
    InteractiveWorkerFixedStepSampleV1, InteractiveWorkerMessageAgeSampleV1,
    InteractiveWorkerPublicationSampleV1, InteractiveWorkerReadyV1,
    InteractiveWorkerSnapshotReadSampleV1, InteractiveWorkerTimingSampleV1,
    PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS, PreparedProductionWorkerDiagnosticV1,
    ProductionWorkerDiagnosticMeasurementV1, prepare_production_worker_diagnostic,
    run_production_worker_diagnostic,
};
pub use launch::{LaunchRequestV1, ProjectSelectionV1};
pub use live_schedule::FixedStepLiveSchedulerV1;
pub use next_contracts::preferences::text_scale_from_milli;
pub use preferences::{
    PLAYER_PREFERENCE_FILE_NAME, PLAYER_PREFERENCE_QUARANTINE_SUFFIX,
    PlayerPreferenceLoadOutcomeV1, PlayerPreferenceLoadV1, PlayerPreferenceStoreV1,
    preference_ui_options, quarantine_path,
};
pub use report::{
    DiagnosticContextV1, DiagnosticReportV1, OPERATIONAL_REPORT_SCHEMA_VERSION,
    PresentationReportV1, RunReportV1,
};
