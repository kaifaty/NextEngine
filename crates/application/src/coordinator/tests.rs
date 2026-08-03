use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use next_assets::{ContentStore, SessionObjectV1, SessionPublicationV1, SessionStore};
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_ESCAPE_CONTROL_PATH_ID, KEYBOARD_W_CONTROL_PATH_ID,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_contracts::session::{
    ApplicationLifecycleRequestV1, ApplicationSessionStatusV1, BoundedDeadlineClassV1,
    CausalInputReferenceV1, CausalInputSourceKindV1, CloseSessionRequestV1, CloseSessionResultV1,
    CompositionRootV1, FailureDispositionV1, LifecycleReasonKindV1, LifecycleReasonV1,
    PresentationTargetKindV1, ShutdownPolicyV1,
};
use next_project::cook_project_v1;
use next_reference_game::project_source_v2;
use next_runtime::SessionTransitionReferencesV1;

use crate::{
    ApplicationCloseOutcomeV1, CloseExecutionOptionsV1, FinalSaveAttemptFailureV1,
    FixedStepLiveSchedulerV1, LaunchRequestV1, ProjectSelectionV1,
};

use super::ApplicationCoordinator;

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

mod close;
mod helpers;
mod live;
mod platform;
mod recovery;
mod scheduler;

use helpers::*;
