#![forbid(unsafe_code)]

mod activation;
mod cook;
mod resolver;

pub use activation::{ProjectActivationError, activate_project};
pub use cook::{
    CookedProjectV1, NeutralProjectSourceV1, ProjectCookError, SourceChunkBindingV1,
    cook_project_v1, neutral_vertical_slice_source_v1,
};
pub use resolver::{ProjectResolutionError, resolve_project_records_v1};
