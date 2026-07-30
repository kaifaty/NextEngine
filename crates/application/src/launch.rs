use std::path::PathBuf;

use next_contracts::ids::{CapabilityId, ContentHash, SchemaId};
use next_contracts::platform::{
    NormalizedPlatformCapabilityV1, PlatformCapabilitySetV1, PresentationTargetKindV1,
};
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
    pub platform_capability_set: Option<PlatformCapabilitySetV1>,
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
            platform_capability_set: reference_capability_set(presentation_target),
        }
    }
}

fn reference_capability_set(
    presentation_target: PresentationTargetKindV1,
) -> Option<PlatformCapabilitySetV1> {
    if presentation_target == PresentationTargetKindV1::None {
        return None;
    }
    let input_classes = if presentation_target == PresentationTargetKindV1::Interactive {
        vec![
            static_schema_id("nextengine.input.keyboard"),
            static_schema_id("nextengine.input.mouse"),
        ]
    } else {
        Vec::new()
    };
    Some(
        PlatformCapabilitySetV1::new(
            static_schema_id("nextengine.platform.reference-fixture-capabilities"),
            static_schema_id("nextengine.platform.reference-fixture"),
            vec![NormalizedPlatformCapabilityV1 {
                capability_id: static_capability_id("nextengine.platform.presentation-target"),
                limit: 1,
            }],
            Vec::new(),
            vec![presentation_target],
            input_classes,
            static_schema_id("nextengine.platform.timebase.reference-ticks"),
        )
        .unwrap_or_else(|_| unreachable!("static reference capability set is valid")),
    )
}

fn static_schema_id(value: &'static str) -> SchemaId {
    SchemaId::new(value).unwrap_or_else(|_| unreachable!("static schema identifier is valid"))
}

fn static_capability_id(value: &'static str) -> CapabilityId {
    CapabilityId::new(value)
        .unwrap_or_else(|_| unreachable!("static capability identifier is valid"))
}
