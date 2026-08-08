#![forbid(unsafe_code)]

mod activation;
mod authoring;
mod cook;
mod cook_rpg;
mod cook_support;
mod resolver;

pub use activation::{ProjectActivationError, activate_project};
pub use authoring::{
    PROJECT_AUTHORING_MANIFEST_FILE, ProjectAuthoringError, load_project_authoring_v1,
    load_project_authoring_v1_with_project_id,
};
pub use cook::{
    CookedProjectV1, NeutralProjectSourceV1, ProjectCookError, SourceChunkBindingV1,
    cook_project_v1,
};
pub use resolver::{ProjectResolutionError, resolve_project_records_v1};
