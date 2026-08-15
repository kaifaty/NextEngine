#![forbid(unsafe_code)]

mod activation;
mod authoring;
mod cook;
mod cook_rpg;
mod cook_support;

pub use activation::{
    ActivatedProjectPackage, ProjectActivationError, activate_project, activate_project_package,
};
pub use authoring::{
    PROJECT_AUTHORING_MANIFEST_FILE, ProjectAuthoringError, load_project_authoring_v3,
    load_project_authoring_v3_with_project_id,
};
pub use cook::{
    CookedProjectV3, NeutralProjectSourceV3, ProjectCookError, SourceChunkBindingV1,
    cook_project_v3,
};
