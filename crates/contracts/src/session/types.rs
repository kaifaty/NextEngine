use crate::ids::{ContentHash, SchemaId};

use super::SessionContractError;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CompositionRootV1 {
    Game = 1,
    Headless = 2,
    Tools = 3,
    CaptureWorker = 4,
}

impl CompositionRootV1 {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Game => "Game",
            Self::Headless => "Headless",
            Self::Tools => "Tools",
            Self::CaptureWorker => "CaptureWorker",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, SessionContractError> {
        match value {
            "Game" => Ok(Self::Game),
            "Headless" => Ok(Self::Headless),
            "Tools" => Ok(Self::Tools),
            "CaptureWorker" => Ok(Self::CaptureWorker),
            _ => Err(SessionContractError::UnknownClosedValue),
        }
    }
}

pub fn validate_root_target(
    root: CompositionRootV1,
    target: crate::platform::PresentationTargetKindV1,
) -> Result<(), SessionContractError> {
    use crate::platform::PresentationTargetKindV1;
    if matches!(
        (root, target),
        (
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive
        ) | (CompositionRootV1::Game, PresentationTargetKindV1::None)
            | (CompositionRootV1::Headless, PresentationTargetKindV1::None)
            | (CompositionRootV1::Tools, PresentationTargetKindV1::None)
            | (
                CompositionRootV1::Tools,
                PresentationTargetKindV1::Interactive
            )
            | (
                CompositionRootV1::CaptureWorker,
                PresentationTargetKindV1::DisplaylessOffscreen
            )
    ) {
        Ok(())
    } else {
        Err(SessionContractError::InvalidRootTarget)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ApplicationSessionStatusV1 {
    Created = 1,
    CompositionStaged = 2,
    RuntimeStaged = 3,
    Active = 4,
    Suspended = 5,
    Quiescing = 6,
    Finalizing = 7,
    Closed = 8,
}

impl ApplicationSessionStatusV1 {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Created => "Created",
            Self::CompositionStaged => "CompositionStaged",
            Self::RuntimeStaged => "RuntimeStaged",
            Self::Active => "Active",
            Self::Suspended => "Suspended",
            Self::Quiescing => "Quiescing",
            Self::Finalizing => "Finalizing",
            Self::Closed => "Closed",
        }
    }

    #[must_use]
    pub const fn can_transition_to(self, target: Self) -> bool {
        matches!(
            (self, target),
            (Self::Created, Self::CompositionStaged)
                | (Self::CompositionStaged, Self::RuntimeStaged)
                | (Self::RuntimeStaged, Self::Active)
                | (Self::Active, Self::Suspended)
                | (Self::Suspended, Self::Active)
                | (Self::Active, Self::Quiescing)
                | (Self::Suspended, Self::Quiescing)
                | (Self::Quiescing, Self::Finalizing)
                | (Self::Finalizing, Self::Closed)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CausalInputSourceKindV1 {
    PlatformEvent = 1,
    PlayerAction = 2,
    ToolRequest = 3,
    System = 4,
}

impl CausalInputSourceKindV1 {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::PlatformEvent => "PlatformEvent",
            Self::PlayerAction => "PlayerAction",
            Self::ToolRequest => "ToolRequest",
            Self::System => "System",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInputReferenceV1 {
    pub source_kind: CausalInputSourceKindV1,
    pub canonical_hash: ContentHash,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum LifecycleReasonKindV1 {
    Launch = 1,
    CompositionReady = 2,
    RuntimeReady = 3,
    SuspendRequested = 4,
    ResumeRequested = 5,
    UserCloseRequested = 6,
    HostCloseRequested = 7,
    FatalHostFault = 8,
    FinalSaveReady = 9,
    Recovery = 10,
}

impl LifecycleReasonKindV1 {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Launch => "Launch",
            Self::CompositionReady => "CompositionReady",
            Self::RuntimeReady => "RuntimeReady",
            Self::SuspendRequested => "SuspendRequested",
            Self::ResumeRequested => "ResumeRequested",
            Self::UserCloseRequested => "UserCloseRequested",
            Self::HostCloseRequested => "HostCloseRequested",
            Self::FatalHostFault => "FatalHostFault",
            Self::FinalSaveReady => "FinalSaveReady",
            Self::Recovery => "Recovery",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleReasonV1 {
    pub kind: LifecycleReasonKindV1,
    pub reason_code: SchemaId,
}
