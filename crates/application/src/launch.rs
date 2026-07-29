use std::path::PathBuf;

use next_contracts::ids::ContentHash;
use next_contracts::platform::PresentationTargetKindV1;
use next_contracts::session::CompositionRootV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectSelectionV1 {
    Reference,
    PublishedStateRoot(PathBuf),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchRequestV1 {
    pub project: ProjectSelectionV1,
    pub expected_project_lock: Option<ContentHash>,
    pub state_root: PathBuf,
    pub composition_root: CompositionRootV1,
    pub presentation_target: PresentationTargetKindV1,
}

impl LaunchRequestV1 {
    #[must_use]
    pub fn reference(
        state_root: impl Into<PathBuf>,
        composition_root: CompositionRootV1,
        presentation_target: PresentationTargetKindV1,
    ) -> Self {
        Self {
            project: ProjectSelectionV1::Reference,
            expected_project_lock: None,
            state_root: state_root.into(),
            composition_root,
            presentation_target,
        }
    }
}
