#![forbid(unsafe_code)]

mod activation;
mod authoring;
mod cook;
mod cook_rpg;
mod cook_support;

pub use activation::{ProjectActivationError, activate_project};
pub use authoring::{
    PROJECT_AUTHORING_MANIFEST_FILE, ProjectAuthoringError, load_project_authoring_v2,
    load_project_authoring_v2_with_project_id,
};
pub use cook::{
    CookedProjectV2, NeutralProjectSourceV2, ProjectCookError, SourceChunkBindingV1,
    cook_project_v2,
};
