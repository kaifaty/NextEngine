use crate::ids::{ContentHash, SchemaId};

use super::codec::{boolean, number, object, session_hash, string};
use super::{APPLICATION_SESSION_SCHEMA_VERSION, SessionContractError};

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

// Project authoring v1 still carries these fields until the exact-lock cut in
// ADR-048. They are not consumed by the V2 application-session protocol.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FailureDispositionV1 {
    RequireFinalSave = 1,
    AllowLastSafeGeneration = 2,
}

impl FailureDispositionV1 {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::RequireFinalSave => "RequireFinalSave",
            Self::AllowLastSafeGeneration => "AllowLastSafeGeneration",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, SessionContractError> {
        match value {
            "RequireFinalSave" => Ok(Self::RequireFinalSave),
            "AllowLastSafeGeneration" => Ok(Self::AllowLastSafeGeneration),
            _ => Err(SessionContractError::UnknownClosedValue),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShutdownPolicyV1 {
    pub schema_version: u32,
    pub maximum_attempts: u16,
    pub failure_disposition: FailureDispositionV1,
    pub canonical_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryPolicyV1 {
    pub schema_version: u32,
    pub permit_required_save_recovery: bool,
    pub preserve_prior_history: bool,
    pub canonical_hash: ContentHash,
}

impl ShutdownPolicyV1 {
    pub fn new(
        maximum_attempts: u16,
        failure_disposition: FailureDispositionV1,
    ) -> Result<Self, SessionContractError> {
        if maximum_attempts == 0 {
            return Err(SessionContractError::InvalidAttemptCount);
        }
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            maximum_attempts,
            failure_disposition,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash(
            "nextengine.shutdown-policy.v1",
            &object([
                ("failure_disposition", string(failure_disposition.token())),
                ("maximum_attempts", number(maximum_attempts)),
                ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ]),
        );
        Ok(value)
    }

    #[must_use]
    pub fn reference_game_default() -> Self {
        Self::new(3, FailureDispositionV1::RequireFinalSave).expect("reference shutdown policy")
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if Self::new(self.maximum_attempts, self.failure_disposition)?.canonical_hash
            != self.canonical_hash
        {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }
}

impl RecoveryPolicyV1 {
    #[must_use]
    pub fn new(permit_required_save_recovery: bool, preserve_prior_history: bool) -> Self {
        let mut value = Self {
            schema_version: APPLICATION_SESSION_SCHEMA_VERSION,
            permit_required_save_recovery,
            preserve_prior_history,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = session_hash(
            "nextengine.recovery-policy.v1",
            &object([
                (
                    "permit_required_save_recovery",
                    boolean(permit_required_save_recovery),
                ),
                ("preserve_prior_history", boolean(preserve_prior_history)),
                ("schema_version", number(APPLICATION_SESSION_SCHEMA_VERSION)),
            ]),
        );
        value
    }

    #[must_use]
    pub fn reference_game_default() -> Self {
        Self::new(true, true)
    }

    pub fn validate(&self) -> Result<(), SessionContractError> {
        if Self::new(
            self.permit_required_save_recovery,
            self.preserve_prior_history,
        )
        .canonical_hash
            != self.canonical_hash
        {
            return Err(SessionContractError::HashMismatch);
        }
        Ok(())
    }
}
