use next_contracts::physical_animation::PhysicalAnimationSnapshotV1;

use super::{
    ReferenceDialogueUiV1, ReferenceGameDriverV2, ReferenceGameError, ReferenceUiScreenV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ReferenceGameGenerationV1 {
    next_logical_frame_sequence: u64,
    events: u64,
    rpg_events: u64,
    camera_yaw_millidegrees: i32,
    camera_pitch_millidegrees: i32,
    camera_cut: bool,
    input_last_logical_frame_sequence: Option<u64>,
    presentation_snapshot_sequence: u64,
    presentation_simulation_tick: u64,
    ui_screen: ReferenceUiScreenV1,
    dialogue: ReferenceDialogueUiV1,
    physical_animation_snapshot: PhysicalAnimationSnapshotV1,
}

impl ReferenceGameGenerationV1 {
    pub(super) fn capture(driver: &ReferenceGameDriverV2) -> Result<Self, ReferenceGameError> {
        let presentation = driver.presentation_snapshot()?;
        Ok(Self {
            next_logical_frame_sequence: driver.next_logical_frame_sequence,
            events: driver.events,
            rpg_events: driver.rpg_events,
            camera_yaw_millidegrees: driver.camera_yaw_millidegrees,
            camera_pitch_millidegrees: driver.camera_pitch_millidegrees,
            camera_cut: driver.camera_cut,
            input_last_logical_frame_sequence: driver.input.last_logical_frame_sequence(),
            presentation_snapshot_sequence: presentation.snapshot_sequence,
            presentation_simulation_tick: presentation.simulation_tick,
            ui_screen: driver.ui_screen,
            dialogue: driver.dialogue,
            physical_animation_snapshot: driver.physical_animation.snapshot().clone(),
        })
    }

    pub(super) fn matches(&self, driver: &ReferenceGameDriverV2) -> bool {
        let Some(presentation) = driver.presentation_extractor.accepted_snapshot() else {
            return false;
        };
        self.next_logical_frame_sequence == driver.next_logical_frame_sequence
            && self.events == driver.events
            && self.rpg_events == driver.rpg_events
            && self.camera_yaw_millidegrees == driver.camera_yaw_millidegrees
            && self.camera_pitch_millidegrees == driver.camera_pitch_millidegrees
            && self.camera_cut == driver.camera_cut
            && self.input_last_logical_frame_sequence == driver.input.last_logical_frame_sequence()
            && self.presentation_snapshot_sequence == presentation.snapshot_sequence
            && self.presentation_simulation_tick == presentation.simulation_tick
            && self.ui_screen == driver.ui_screen
            && self.dialogue == driver.dialogue
            && self.physical_animation_snapshot == *driver.physical_animation.snapshot()
    }
}
