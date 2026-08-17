//! Host-side interactive pause menu (S5, decisions S5-Q1A/Q2A/Q3A).
//!
//! While the application is `Suspended` no game ticks run, so the pause-menu
//! navigation cannot ride the production input ingress. This controller keeps
//! a deterministic presentation-only selection against the published
//! pause-menu records and activates items through the existing production
//! paths: resume via an admitted `ResumeRequested` platform event, save via
//! the same save-store write the final save uses, and load via the
//! verified save-store generation and an atomic same-session world replacement.
//! No new lifecycle edges are introduced.

use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};

use next_contracts::ids::PersistentId;
use next_contracts::ids::{ContentHash, SchemaId};
use next_contracts::input::{
    KEYBOARD_DOWN_CONTROL_PATH_ID, KEYBOARD_ESCAPE_CONTROL_PATH_ID, KEYBOARD_LEFT_CONTROL_PATH_ID,
    KEYBOARD_RETURN_CONTROL_PATH_ID, KEYBOARD_RIGHT_CONTROL_PATH_ID, KEYBOARD_UP_CONTROL_PATH_ID,
};
use next_contracts::platform::{
    NormalizedControlPhaseV1, PlatformCapabilitySetV1, PlatformEventKindV1, PlatformEventPayloadV1,
    PlatformEventV1,
};
use next_contracts::presentation::{
    PresentationSnapshotV3, SemanticUiPresentationRecordV1, UiSemanticElementV1, UiStyleRoleV1,
    UiTextRefV1,
};
use next_contracts::session::ApplicationSessionStatusV1;
use next_reference_game::{
    PAUSE_MENU_LOAD_ELEMENT_ID, PAUSE_MENU_LOADED_TEXT_ID, PAUSE_MENU_RESUME_ELEMENT_ID,
    PAUSE_MENU_SAVE_ELEMENT_ID, PAUSE_MENU_SAVED_TEXT_ID, PAUSE_MENU_SURFACE_ID,
};

use super::runtime::record_interactive_worker_failure;
use super::{
    InteractivePresentationMailboxV1, InteractivePublishedSnapshotV1, InteractiveWorkerFailureV1,
};
use crate::{ApplicationCoordinator, ApplicationError, FixedStepLiveSchedulerV1};

/// Dedicated platform source class for menu-fabricated lifecycle events; it
/// owns an independent admission cursor, so synthetic sequences never
/// collide with adapter-emitted sources.
pub(super) const PAUSE_MENU_SOURCE_CLASS: &str = "nextengine.platform.source.pause-menu";
pub(super) const PAUSE_MENU_RESUME_REASON: &str = "nextengine.platform.reason.pause-menu-resume";

/// Selectable pause-menu items in the stable user-facing display order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PauseMenuItemV1 {
    Load,
    Resume,
    Save,
}

impl PauseMenuItemV1 {
    const ORDER: [Self; 3] = [Self::Resume, Self::Save, Self::Load];

    fn index(self) -> usize {
        Self::ORDER
            .iter()
            .position(|item| *item == self)
            .expect("pause menu item order is total")
    }

    fn moved(self, delta: isize) -> Self {
        let len = Self::ORDER.len() as isize;
        let index = (self.index() as isize + delta).rem_euclid(len);
        Self::ORDER[usize::try_from(index).expect("rem_euclid is bounded by the order length")]
    }

    fn element_id(self) -> &'static str {
        match self {
            Self::Load => PAUSE_MENU_LOAD_ELEMENT_ID,
            Self::Resume => PAUSE_MENU_RESUME_ELEMENT_ID,
            Self::Save => PAUSE_MENU_SAVE_ELEMENT_ID,
        }
    }
}

/// Production action requested by a committed menu activation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PauseMenuActionV1 {
    Resume,
    Save,
    Load,
}

/// Outcome of routing one submit batch through the menu controller.
pub(super) struct PauseMenuProcessingV1 {
    pub action: Option<PauseMenuActionV1>,
    pub remaining_events: Vec<PlatformEventV1>,
    pub consumed_events: Vec<PlatformEventV1>,
    pub selection_changed: bool,
}

/// Deterministic presentation-only pause-menu state.
pub(super) struct PauseMenuControllerV1 {
    selection: PauseMenuItemV1,
    synthetic_sequence: u64,
    swallowed: [bool; 6],
}

/// Menu keys; the horizontal arrows mirror the vertical pair (ui-nav).
#[derive(Clone, Copy)]
enum PauseMenuKeyV1 {
    Up = 0,
    Down = 1,
    Confirm = 2,
    Back = 3,
    Left = 4,
    Right = 5,
}

impl PauseMenuKeyV1 {
    fn nav_delta(self) -> Option<isize> {
        match self {
            Self::Up | Self::Left => Some(-1),
            Self::Down | Self::Right => Some(1),
            Self::Confirm | Self::Back => None,
        }
    }
}

impl PauseMenuControllerV1 {
    pub fn new() -> Self {
        Self {
            selection: PauseMenuItemV1::Resume,
            synthetic_sequence: 0,
            swallowed: [false; 6],
        }
    }

    pub fn selection(&self) -> PauseMenuItemV1 {
        self.selection
    }

    pub fn reset_selection(&mut self) {
        self.selection = PauseMenuItemV1::Resume;
    }

    /// Routes one suspended submit batch. Menu keys (`ui-nav` arrows,
    /// `ui-confirm` return, `ui-back` escape) are consumed; every other event
    /// passes through to the suspended scheduler unchanged. A press the game
    /// saw before the suspension keeps its release path: only completions of
    /// menu-swallowed presses are swallowed, so the player input session
    /// never dangles across the resume.
    pub fn process_events(&mut self, events: &[PlatformEventV1]) -> PauseMenuProcessingV1 {
        let mut action = None;
        let mut remaining_events = Vec::with_capacity(events.len());
        let mut consumed_events = Vec::new();
        let mut selection_changed = false;
        for event in events {
            if action.is_some() {
                // The menu is closing on this batch; everything after the
                // activation belongs to the resumed game.
                remaining_events.push(event.clone());
                continue;
            }
            let Some((key, phase)) = menu_key(event) else {
                remaining_events.push(event.clone());
                continue;
            };
            match phase {
                NormalizedControlPhaseV1::Started => {
                    self.swallowed[key as usize] = true;
                    consumed_events.push(event.clone());
                    if let Some(delta) = key.nav_delta() {
                        self.selection = self.selection.moved(delta);
                        selection_changed = true;
                    } else {
                        action = Some(match key {
                            PauseMenuKeyV1::Back => PauseMenuActionV1::Resume,
                            _ => match self.selection {
                                PauseMenuItemV1::Load => PauseMenuActionV1::Load,
                                PauseMenuItemV1::Resume => PauseMenuActionV1::Resume,
                                PauseMenuItemV1::Save => PauseMenuActionV1::Save,
                            },
                        });
                        // The menu is closing; releases after this point flow
                        // to the resumed game, where a completion without an
                        // active press is a quiet no-op.
                        self.swallowed = [false; 6];
                    }
                }
                _ => {
                    if self.swallowed[key as usize] {
                        self.swallowed[key as usize] = false;
                        consumed_events.push(event.clone());
                    } else {
                        remaining_events.push(event.clone());
                    }
                }
            }
        }
        PauseMenuProcessingV1 {
            action,
            remaining_events,
            consumed_events,
            selection_changed,
        }
    }

    /// Fabricates a `ResumeRequested` platform event admitted through the
    /// production lifecycle path (same transition, dedup and archive as a
    /// window-restored resume).
    pub fn fabricate_resume_event(
        &mut self,
        host_instance_id: PersistentId,
        capability_set_hash: ContentHash,
    ) -> Result<PlatformEventV1, ApplicationError> {
        let event = PlatformEventV1::new(
            host_instance_id,
            SchemaId::new(PAUSE_MENU_SOURCE_CLASS)?,
            self.synthetic_sequence,
            0,
            PlatformEventKindV1::ResumeRequested,
            PlatformEventPayloadV1::Reason {
                reason: SchemaId::new(PAUSE_MENU_RESUME_REASON)?,
            },
            capability_set_hash,
        )?;
        self.synthetic_sequence = self
            .synthetic_sequence
            .checked_add(1)
            .ok_or(ApplicationError::LiveTickBacklogExceeded)?;
        Ok(event)
    }
}

fn menu_key(event: &PlatformEventV1) -> Option<(PauseMenuKeyV1, NormalizedControlPhaseV1)> {
    if event.kind != PlatformEventKindV1::Control {
        return None;
    }
    let PlatformEventPayloadV1::Control(control) = &event.payload else {
        return None;
    };
    let key = match control.control_path_id.as_str() {
        KEYBOARD_UP_CONTROL_PATH_ID => PauseMenuKeyV1::Up,
        KEYBOARD_DOWN_CONTROL_PATH_ID => PauseMenuKeyV1::Down,
        KEYBOARD_LEFT_CONTROL_PATH_ID => PauseMenuKeyV1::Left,
        KEYBOARD_RIGHT_CONTROL_PATH_ID => PauseMenuKeyV1::Right,
        KEYBOARD_RETURN_CONTROL_PATH_ID => PauseMenuKeyV1::Confirm,
        KEYBOARD_ESCAPE_CONTROL_PATH_ID => PauseMenuKeyV1::Back,
        _ => return None,
    };
    Some((key, control.phase))
}

/// True when the published snapshot carries the pause-menu surface, i.e. the
/// suspension came from the declared ui-back pause path (a platform suspend,
/// e.g. window minimize, publishes no menu and stays non-interactive).
pub(super) fn snapshot_has_pause_menu(snapshot: &PresentationSnapshotV3) -> bool {
    snapshot
        .semantic_ui_records()
        .any(|record| record.surface_id.as_str() == PAUSE_MENU_SURFACE_ID)
}

/// Rebuilds a presentation-only clone of the suspending snapshot with the
/// menu selection applied (`selected` flag + accent style, mirroring the
/// canonical pause-menu publication) under the next publication sequence.
pub(super) fn pause_menu_snapshot_with_selection(
    current: &PresentationSnapshotV3,
    selection: PauseMenuItemV1,
    next_sequence: u64,
) -> Result<PresentationSnapshotV3, InteractiveWorkerFailureV1> {
    rebuild_snapshot(current, next_sequence, Some(selection), None, None)
}

/// Rebuilds the selected pause-menu publication with an explicit successful
/// save label. The clone is presentation-only and does not enter save state.
pub(super) fn pause_menu_snapshot_with_save_confirmation(
    current: &PresentationSnapshotV3,
    selection: PauseMenuItemV1,
    next_sequence: u64,
) -> Result<PresentationSnapshotV3, InteractiveWorkerFailureV1> {
    rebuild_snapshot(
        current,
        next_sequence,
        Some(selection),
        Some(PauseMenuItemV1::Save),
        None,
    )
}

/// Rebuilds the selected pause-menu publication with an explicit successful
/// load label. Keeping the menu open lets the player acknowledge the restored
/// state before resuming simulation.
pub(super) fn pause_menu_snapshot_with_load_confirmation(
    current: &PresentationSnapshotV3,
    menu_template: &PresentationSnapshotV3,
    selection: PauseMenuItemV1,
    next_sequence: u64,
) -> Result<PresentationSnapshotV3, InteractiveWorkerFailureV1> {
    rebuild_snapshot(
        current,
        next_sequence,
        Some(selection),
        Some(PauseMenuItemV1::Load),
        Some(menu_template),
    )
}

/// Rebuilds a presentation-only clone of a real publication whose sequence
/// collides with menu-republished clones, continuing the strictly increasing
/// per-epoch order the desktop adapter enforces. Content is unchanged.
pub(super) fn resequenced_snapshot(
    current: &PresentationSnapshotV3,
    next_sequence: u64,
) -> Result<PresentationSnapshotV3, InteractiveWorkerFailureV1> {
    rebuild_snapshot(current, next_sequence, None, None, None)
}

/// The clone never enters recovery evidence: it exists only so the desktop
/// adapter renders the host-side menu state. Rebuilt batches re-chunk the
/// flattened records (canonical sort is preserved by the constructor); cue
/// batches would be lossy, so their presence is refused and the reference
/// publication never carries them.
fn rebuild_snapshot(
    current: &PresentationSnapshotV3,
    next_sequence: u64,
    selection: Option<PauseMenuItemV1>,
    confirmed_item: Option<PauseMenuItemV1>,
    menu_template: Option<&PresentationSnapshotV3>,
) -> Result<PresentationSnapshotV3, InteractiveWorkerFailureV1> {
    fn rebuild_failure() -> InteractiveWorkerFailureV1 {
        InteractiveWorkerFailureV1::runtime(
            "PLATFORM_PRESENTATION_SNAPSHOT_REBUILD_FAILED",
            "pause-menu host publication failed to rebuild the presentation snapshot",
        )
    }
    if !current.cue_batches.is_empty() {
        return Err(rebuild_failure());
    }
    let mut source_ui_records = current.semantic_ui_records().cloned().collect::<Vec<_>>();
    if let Some(template) = menu_template {
        source_ui_records.retain(|record| record.surface_id.as_str() != PAUSE_MENU_SURFACE_ID);
        source_ui_records.extend(
            template
                .semantic_ui_records()
                .filter(|record| record.surface_id.as_str() == PAUSE_MENU_SURFACE_ID)
                .cloned(),
        );
    }
    let mut ui_records = Vec::with_capacity(source_ui_records.len());
    for record in source_ui_records {
        let item = PauseMenuItemV1::ORDER
            .iter()
            .copied()
            .find(|item| item.element_id() == record.element.element_id.as_str());
        let element = &record.element;
        let (selected, style_role) = match (item, selection) {
            (Some(item), Some(selection)) => (
                item == selection,
                if item == selection {
                    UiStyleRoleV1::Accent
                } else {
                    UiStyleRoleV1::Default
                },
            ),
            _ => (element.selected, element.style_role),
        };
        let confirmation_text_id = match (confirmed_item, item) {
            (Some(PauseMenuItemV1::Save), Some(PauseMenuItemV1::Save)) => {
                Some(PAUSE_MENU_SAVED_TEXT_ID)
            }
            (Some(PauseMenuItemV1::Load), Some(PauseMenuItemV1::Load)) => {
                Some(PAUSE_MENU_LOADED_TEXT_ID)
            }
            _ => None,
        };
        let text_or_none = if let Some(text_id) = confirmation_text_id {
            Some(
                UiTextRefV1::new(
                    SchemaId::new(text_id).map_err(|_| rebuild_failure())?,
                    Vec::new(),
                )
                .map_err(|_| rebuild_failure())?,
            )
        } else {
            element.text_or_none.clone()
        };
        ui_records.push(
            SemanticUiPresentationRecordV1::new(
                current.snapshot_epoch,
                record.surface_id.clone(),
                record.semantic_path_id.clone(),
                record.source_snapshot_hash,
                UiSemanticElementV1::new(
                    element.element_id.clone(),
                    element.role,
                    style_role,
                    element.accessibility_role,
                    element.enabled,
                    element.visible,
                    selected,
                    text_or_none,
                    element.value,
                    element.affordances.clone(),
                )
                .map_err(|_| rebuild_failure())?,
            )
            .map_err(|_| rebuild_failure())?,
        );
    }
    let scene_records: Vec<_> = current.scene_records().cloned().collect::<Vec<_>>();
    let camera_records: Vec<_> = current.camera_records().cloned().collect::<Vec<_>>();
    let character_skinning_records = current.character_skinning_records().cloned().collect();
    let max_scene = scene_records.len().max(1);
    let max_camera = camera_records.len().max(1);
    let max_ui = ui_records.len().max(1);
    PresentationSnapshotV3::new_with_character_skinning_records(
        current.snapshot_epoch,
        next_sequence,
        current.simulation_tick,
        current.project_composition_lock_hash,
        current.content_manifest_hash,
        current.presentation_profile_hash,
        scene_records,
        camera_records,
        ui_records,
        character_skinning_records,
        max_scene,
        max_camera,
        max_ui,
        current.environment_batch,
    )
    .map_err(|_| rebuild_failure())
}

/// Mutable borrow bundle for one suspended pause-menu frame; every field is
/// owned by the worker loop in `runtime.rs` and lent for the menu routing
/// only.
pub(super) struct PauseMenuFrameContextV1<'a> {
    pub capabilities: &'a PlatformCapabilitySetV1,
    pub application: &'a mut ApplicationCoordinator,
    pub fixed_step: &'a mut FixedStepLiveSchedulerV1,
    pub host_instance_id: &'a mut PersistentId,
    pub presentation_mailbox: &'a Mutex<InteractivePresentationMailboxV1>,
    pub last_publication: &'a mut (ContentHash, u64),
    pub pending_failure: &'a mut Option<InteractiveWorkerFailureV1>,
    pub failure_sender: &'a SyncSender<InteractiveWorkerFailureV1>,
    pub callback_sequence: u64,
}

/// Runs one suspended pause-menu frame: consumes menu input through the
/// controller, applies the requested action through the production paths
/// (admitted resume, save-store write, durable reload) and republishes a
/// selection clone when the highlight moved. Returns the events that keep
/// their path to the suspended fixed-step scheduler.
pub(super) fn handle_suspended_pause_menu_frame(
    context: &mut PauseMenuFrameContextV1<'_>,
    controller: &mut PauseMenuControllerV1,
    events: Vec<PlatformEventV1>,
) -> Vec<PlatformEventV1> {
    let PauseMenuFrameContextV1 {
        capabilities,
        application,
        fixed_step,
        host_instance_id,
        presentation_mailbox,
        last_publication,
        pending_failure,
        failure_sender,
        callback_sequence,
    } = context;
    let capabilities = *capabilities;
    let application = &mut **application;
    let fixed_step = &mut **fixed_step;
    let host_instance_id = &mut **host_instance_id;
    let presentation_mailbox = *presentation_mailbox;
    let last_publication = &mut **last_publication;
    let pending_failure = &mut **pending_failure;
    let failure_sender = *failure_sender;
    let callback_sequence = *callback_sequence;
    let processing = controller.process_events(&events);
    let mut remaining_events = processing.remaining_events;
    let mut save_confirmed = false;
    let mut load_confirmed = false;
    let mut load_menu_template = None;
    if !processing.consumed_events.is_empty() {
        // Menu keys were admitted at the coordinator boundary but must never
        // reach the simulation; queue them for cursor-only admission so the
        // input session's per-source continuity stays exact across the
        // suspension.
        if let Err(error) = application.queue_host_consumed_live_input(&processing.consumed_events)
        {
            record_interactive_worker_failure(
                pending_failure,
                failure_sender,
                InteractiveWorkerFailureV1::application(error),
            );
        }
    }
    if matches!(
        processing.action,
        Some(PauseMenuActionV1::Save | PauseMenuActionV1::Load)
    ) {
        // Save keeps the menu open and Load resets the fixed-step queue. Menu
        // key releases trailing their activation therefore remain host-owned;
        // admit them cursor-only now so the continuing keyboard stream cannot
        // acquire a gap at the save/load cut. Non-menu events still pass on.
        let trailing_menu_events = remaining_events
            .iter()
            .filter(|event| menu_key(event).is_some())
            .cloned()
            .collect::<Vec<_>>();
        remaining_events.retain(|event| menu_key(event).is_none());
        if let Err(error) = application.queue_host_consumed_live_input(&trailing_menu_events) {
            record_interactive_worker_failure(
                pending_failure,
                failure_sender,
                InteractiveWorkerFailureV1::application(error),
            );
        }
    }
    if pending_failure.is_none()
        && let Some(action) = processing.action
    {
        match action {
            PauseMenuActionV1::Resume => {
                let resumed = controller
                    .fabricate_resume_event(*host_instance_id, capabilities.canonical_hash)
                    .and_then(|event| application.resume_from_platform_event(&event).map(|_| ()));
                if let Err(error) = resumed {
                    record_interactive_worker_failure(
                        pending_failure,
                        failure_sender,
                        InteractiveWorkerFailureV1::application(error),
                    );
                }
            }
            PauseMenuActionV1::Save => match application.save_current_prepared_run() {
                Ok(_) => save_confirmed = true,
                Err(error) => {
                    record_interactive_worker_failure(
                        pending_failure,
                        failure_sender,
                        InteractiveWorkerFailureV1::application(error),
                    );
                }
            },
            PauseMenuActionV1::Load => {
                let menu_template = presentation_mailbox.lock().ok().and_then(|mailbox| {
                    mailbox
                        .latest()
                        .map(|published| Arc::clone(&published.snapshot))
                        .filter(|snapshot| snapshot_has_pause_menu(snapshot))
                });
                let unapplied = fixed_step.take_unapplied_events_for_save_load();
                if let Err(error) = application.queue_host_consumed_live_input(&unapplied) {
                    record_interactive_worker_failure(
                        pending_failure,
                        failure_sender,
                        InteractiveWorkerFailureV1::application(error),
                    );
                    return remaining_events;
                }
                match application.load_latest_save_into_live_run() {
                    Ok(run) => {
                        let snapshot = run.presentation_snapshot.map(Arc::new);
                        let Some(snapshot) = snapshot else {
                            record_interactive_worker_failure(
                                pending_failure,
                                failure_sender,
                                InteractiveWorkerFailureV1::runtime(
                                    "PLATFORM_PRESENTATION_SNAPSHOT_MISSING",
                                    "loaded save produced no presentation snapshot",
                                ),
                            );
                            return remaining_events;
                        };
                        *fixed_step = FixedStepLiveSchedulerV1::reference_game_v1();
                        match presentation_mailbox.lock() {
                            Ok(mut mailbox) => {
                                *last_publication =
                                    (snapshot.snapshot_epoch, snapshot.snapshot_sequence);
                                if let Err(failure) = mailbox.publish_required_recovery_boundary(
                                    InteractivePublishedSnapshotV1 {
                                        snapshot,
                                        callback_sequence: Some(callback_sequence),
                                    },
                                ) {
                                    record_interactive_worker_failure(
                                        pending_failure,
                                        failure_sender,
                                        failure,
                                    );
                                }
                            }
                            Err(_) => record_interactive_worker_failure(
                                pending_failure,
                                failure_sender,
                                InteractiveWorkerFailureV1::runtime(
                                    "PLATFORM_PRESENTATION_STATE_POISONED",
                                    "presentation mailbox lock was poisoned",
                                ),
                            ),
                        }
                        if menu_template.is_some() {
                            // Reattach the presentation-only menu to the
                            // validated recovery cut under its fresh epoch.
                            // The restored authoritative world stays paused
                            // until the player explicitly chooses Resume.
                            load_confirmed = true;
                            load_menu_template = menu_template;
                        } else {
                            // Older/final saves need not carry the pause-menu
                            // surface. Avoid a suspended session with no UI by
                            // retaining the historical auto-resume fallback.
                            controller.reset_selection();
                            let resumed = controller
                                .fabricate_resume_event(
                                    *host_instance_id,
                                    capabilities.canonical_hash,
                                )
                                .and_then(|event| {
                                    application.resume_from_platform_event(&event).map(|_| ())
                                });
                            if let Err(error) = resumed {
                                record_interactive_worker_failure(
                                    pending_failure,
                                    failure_sender,
                                    InteractiveWorkerFailureV1::application(error),
                                );
                            }
                        }
                    }
                    Err(error) => record_interactive_worker_failure(
                        pending_failure,
                        failure_sender,
                        InteractiveWorkerFailureV1::application(error),
                    ),
                }
            }
        }
    }
    if (processing.selection_changed || save_confirmed || load_confirmed)
        && pending_failure.is_none()
        && application.state().state == ApplicationSessionStatusV1::Suspended
    {
        let current = presentation_mailbox.lock().ok().and_then(|mailbox| {
            mailbox
                .latest()
                .map(|published| Arc::clone(&published.snapshot))
        });
        if let Some(current) =
            current.filter(|snapshot| load_confirmed || snapshot_has_pause_menu(snapshot))
        {
            if load_confirmed && load_menu_template.is_none() {
                record_interactive_worker_failure(
                    pending_failure,
                    failure_sender,
                    InteractiveWorkerFailureV1::runtime(
                        "PLATFORM_PRESENTATION_SNAPSHOT_REBUILD_FAILED",
                        "load confirmation lost its pause-menu template",
                    ),
                );
                return remaining_events;
            }
            let rebuilt = if save_confirmed {
                pause_menu_snapshot_with_save_confirmation(
                    &current,
                    controller.selection(),
                    last_publication.1.saturating_add(1),
                )
            } else if load_confirmed {
                pause_menu_snapshot_with_load_confirmation(
                    &current,
                    load_menu_template
                        .as_deref()
                        .expect("load confirmation template was checked above"),
                    controller.selection(),
                    last_publication.1.saturating_add(1),
                )
            } else {
                pause_menu_snapshot_with_selection(
                    &current,
                    controller.selection(),
                    last_publication.1.saturating_add(1),
                )
            };
            match rebuilt {
                Ok(clone) => {
                    *last_publication = (clone.snapshot_epoch, clone.snapshot_sequence);
                    match presentation_mailbox.lock() {
                        Ok(mut mailbox) => {
                            mailbox.publish_latest(InteractivePublishedSnapshotV1 {
                                snapshot: Arc::new(clone),
                                callback_sequence: Some(callback_sequence),
                            });
                        }
                        Err(_) => record_interactive_worker_failure(
                            pending_failure,
                            failure_sender,
                            InteractiveWorkerFailureV1::runtime(
                                "PLATFORM_PRESENTATION_STATE_POISONED",
                                "presentation mailbox lock was poisoned",
                            ),
                        ),
                    }
                }
                Err(failure) => {
                    record_interactive_worker_failure(pending_failure, failure_sender, failure)
                }
            }
        }
    }
    remaining_events
}

#[cfg(test)]
mod tests {
    use next_contracts::input::KEYBOARD_DEVICE_CLASS_ID;
    use next_contracts::platform::NormalizedControlEventV1;
    use next_contracts::rpg::RpgSnapshotV2;

    use super::*;

    fn control_event(
        control_path: &str,
        phase: NormalizedControlPhaseV1,
        source_sequence: u64,
    ) -> PlatformEventV1 {
        let control = NormalizedControlEventV1::new(
            SchemaId::new(KEYBOARD_DEVICE_CLASS_ID).expect("device class"),
            PersistentId::from_bytes([0x74; 16]),
            SchemaId::new(control_path).expect("control path"),
            phase,
            vec![if phase == NormalizedControlPhaseV1::Started {
                i16::MAX
            } else {
                0
            }],
            Vec::new(),
            source_sequence,
            source_sequence,
        )
        .expect("control event");
        PlatformEventV1::new(
            PersistentId::from_bytes([0x75; 16]),
            SchemaId::new("nextengine.platform.source.pause-menu-test").expect("source class"),
            source_sequence,
            source_sequence,
            PlatformEventKindV1::Control,
            PlatformEventPayloadV1::Control(control),
            ContentHash::from_bytes(next_contracts::canonical::sha256(
                b"nextengine.platform.pause-menu-test-capabilities.v1",
            )),
        )
        .expect("platform event")
    }

    fn press(controller: &mut PauseMenuControllerV1, control_path: &str) -> PauseMenuProcessingV1 {
        let started = control_event(control_path, NormalizedControlPhaseV1::Started, 1);
        let completed = control_event(control_path, NormalizedControlPhaseV1::Completed, 2);
        let processing = controller.process_events(&[started]);
        assert!(processing.remaining_events.is_empty());
        let release = controller.process_events(&[completed]);
        if processing.action.is_some() {
            // After an activation the press state is reset, so the release
            // flows through to the resumed game as a quiet no-op.
            assert_eq!(release.remaining_events.len(), 1);
        } else {
            assert!(release.remaining_events.is_empty());
        }
        processing
    }

    fn menu_snapshot(sequence: u64) -> PresentationSnapshotV3 {
        let epoch = ContentHash::from_bytes(next_contracts::canonical::sha256(
            b"nextengine.pause-menu-test.epoch.v1",
        ));
        let rpg = RpgSnapshotV2 {
            aggregates: Vec::new(),
        };
        let records = next_reference_game::pause_menu_semantic_ui_records(
            epoch,
            &rpg,
            ContentHash::from_bytes(next_contracts::canonical::sha256(
                b"nextengine.pause-menu-test.causal.v1",
            )),
        )
        .expect("pause-menu records");
        PresentationSnapshotV3::new_with_camera_and_semantic_ui_records(
            epoch,
            sequence,
            7,
            ContentHash::from_bytes(next_contracts::canonical::sha256(b"lock")),
            ContentHash::from_bytes(next_contracts::canonical::sha256(b"manifest")),
            ContentHash::from_bytes(next_contracts::canonical::sha256(b"profile")),
            Vec::new(),
            Vec::new(),
            records,
            1,
            1,
            8,
            ContentHash::from_bytes(next_contracts::canonical::sha256(b"environment")),
        )
        .expect("menu snapshot")
    }

    fn selected_menu_element(snapshot: &PresentationSnapshotV3) -> Option<String> {
        snapshot.semantic_ui_records().find_map(|record| {
            (record.element.selected && record.surface_id.as_str() == PAUSE_MENU_SURFACE_ID)
                .then(|| record.element.element_id.as_str().to_owned())
        })
    }

    #[test]
    fn navigation_cycles_in_display_order_and_confirm_maps_the_selection() {
        let mut controller = PauseMenuControllerV1::new();
        assert_eq!(controller.selection(), PauseMenuItemV1::Resume);

        let processing = press(&mut controller, KEYBOARD_DOWN_CONTROL_PATH_ID);
        assert!(processing.selection_changed);
        assert_eq!(controller.selection(), PauseMenuItemV1::Save);
        press(&mut controller, KEYBOARD_DOWN_CONTROL_PATH_ID);
        assert_eq!(controller.selection(), PauseMenuItemV1::Load);
        press(&mut controller, KEYBOARD_DOWN_CONTROL_PATH_ID);
        assert_eq!(controller.selection(), PauseMenuItemV1::Resume);
        press(&mut controller, KEYBOARD_LEFT_CONTROL_PATH_ID);
        assert_eq!(controller.selection(), PauseMenuItemV1::Load);
        press(&mut controller, KEYBOARD_RIGHT_CONTROL_PATH_ID);
        assert_eq!(controller.selection(), PauseMenuItemV1::Resume);

        let processing = press(&mut controller, KEYBOARD_RETURN_CONTROL_PATH_ID);
        assert_eq!(processing.action, Some(PauseMenuActionV1::Resume));

        press(&mut controller, KEYBOARD_UP_CONTROL_PATH_ID);
        let processing = press(&mut controller, KEYBOARD_RETURN_CONTROL_PATH_ID);
        assert_eq!(processing.action, Some(PauseMenuActionV1::Load));
    }

    #[test]
    fn back_requests_resume_and_events_after_activation_pass_through() {
        let mut controller = PauseMenuControllerV1::new();
        let escape = control_event(
            KEYBOARD_ESCAPE_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            1,
        );
        let w_press = control_event(
            next_contracts::input::KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            2,
        );
        let processing = controller.process_events(&[escape, w_press.clone()]);
        assert_eq!(processing.action, Some(PauseMenuActionV1::Resume));
        assert_eq!(processing.remaining_events, vec![w_press]);

        // A release the game saw before the suspension is never swallowed.
        let stray_release = control_event(
            KEYBOARD_RETURN_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Completed,
            3,
        );
        let processing = controller.process_events(std::slice::from_ref(&stray_release));
        assert_eq!(processing.remaining_events, vec![stray_release]);
    }

    #[test]
    fn selection_republication_flips_only_the_selected_button() {
        let base = menu_snapshot(11);
        assert!(snapshot_has_pause_menu(&base));
        assert_eq!(
            selected_menu_element(&base),
            Some(PAUSE_MENU_RESUME_ELEMENT_ID.to_owned())
        );
        let clone = pause_menu_snapshot_with_selection(&base, PauseMenuItemV1::Save, 12)
            .expect("selection clone");
        clone.validate().expect("clone validates");
        assert_eq!(clone.snapshot_epoch, base.snapshot_epoch);
        assert_eq!(clone.snapshot_sequence, 12);
        assert_eq!(clone.simulation_tick, base.simulation_tick);
        assert_eq!(
            selected_menu_element(&clone),
            Some(PAUSE_MENU_SAVE_ELEMENT_ID.to_owned())
        );
        let save = clone
            .semantic_ui_records()
            .find(|record| record.element.element_id.as_str() == PAUSE_MENU_SAVE_ELEMENT_ID)
            .expect("save element");
        assert_eq!(save.element.style_role, UiStyleRoleV1::Accent);
        let resume = clone
            .semantic_ui_records()
            .find(|record| record.element.element_id.as_str() == PAUSE_MENU_RESUME_ELEMENT_ID)
            .expect("resume element");
        assert!(!resume.element.selected);
        assert_eq!(resume.element.style_role, UiStyleRoleV1::Default);
        assert_eq!(
            clone.semantic_ui_records().count(),
            base.semantic_ui_records().count()
        );
    }

    #[test]
    fn resequenced_clone_preserves_content_under_the_next_sequence() {
        let base = menu_snapshot(41);
        let clone = resequenced_snapshot(&base, 42).expect("resequenced clone");
        clone.validate().expect("clone validates");
        assert_eq!(clone.snapshot_sequence, 42);
        assert_eq!(
            clone.semantic_ui_records().count(),
            base.semantic_ui_records().count()
        );
        assert_eq!(selected_menu_element(&clone), selected_menu_element(&base));
        assert_ne!(clone.canonical_hash, base.canonical_hash);
    }

    #[test]
    fn fabricated_resume_event_carries_the_dedicated_source_cursor() {
        let mut controller = PauseMenuControllerV1::new();
        let first = controller
            .fabricate_resume_event(
                PersistentId::from_bytes([0x75; 16]),
                ContentHash::from_bytes(next_contracts::canonical::sha256(b"capabilities")),
            )
            .expect("first resume event");
        let second = controller
            .fabricate_resume_event(
                PersistentId::from_bytes([0x75; 16]),
                ContentHash::from_bytes(next_contracts::canonical::sha256(b"capabilities")),
            )
            .expect("second resume event");
        first.validate().expect("first validates");
        assert_eq!(first.kind, PlatformEventKindV1::ResumeRequested);
        assert_eq!(
            first.source_class.as_str(),
            "nextengine.platform.source.pause-menu"
        );
        assert_eq!(first.source_sequence, 0);
        assert_eq!(second.source_sequence, 1);
        assert_ne!(first.platform_event_id, second.platform_event_id);
    }
}
