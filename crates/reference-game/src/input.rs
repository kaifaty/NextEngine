use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    CORE_EQUIP_USE_ACTION_ID, CORE_INTERACT_ACTION_ID, CORE_MELEE_ACTION_ID, CORE_MOVE_ACTION_ID,
    CORE_PICKUP_ACTION_ID, CORE_UI_BACK_ACTION_ID, CORE_UI_INVENTORY_ACTION_ID,
    CORE_UI_JOURNAL_ACTION_ID, InputSampleV1, KEYBOARD_A_CONTROL_PATH_ID,
    KEYBOARD_D_CONTROL_PATH_ID, KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_E_CONTROL_PATH_ID,
    KEYBOARD_F_CONTROL_PATH_ID, KEYBOARD_Q_CONTROL_PATH_ID, KEYBOARD_R_CONTROL_PATH_ID,
    KEYBOARD_S_CONTROL_PATH_ID, KEYBOARD_W_CONTROL_PATH_ID, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS, PlayerActionFrameV1,
    PlayerActionPhaseV1, PlayerActionV1, PlayerActionValueV1,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_player::PlayerInputSessionV1;

use crate::{ReferenceGameError, ReferenceGameSession, ReferenceInputError};

pub fn player_action_sample(
    fixture: &ReferenceGameSession,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    direction_q15: [i16; 2],
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, ReferenceInputError> {
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: sequence,
        action_map_hash: fixture.action_map_hash,
        action_map_revision: fixture.action_map.revision,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: fixture.context_stack.revision,
        actions: vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_MOVE_ACTION_ID)?,
            phase,
            value: PlayerActionValueV1::Vector2Q15(direction_q15),
            semantic_occurrence_ordinal: 0,
        }],
    };
    Ok(InputSampleV1 {
        schema_version: 1,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS)?,
        source_id: fixture.source_id,
        source_sequence: sequence,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID)?,
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes()?,
        sampled_wall_time,
    })
}

pub fn player_interact_sample(
    fixture: &ReferenceGameSession,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, ReferenceInputError> {
    player_semantic_action_sample(
        fixture,
        sequence,
        CORE_INTERACT_ACTION_ID,
        phase,
        pressed,
        sampled_wall_time,
    )
}

pub fn player_pickup_sample(
    fixture: &ReferenceGameSession,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, ReferenceInputError> {
    player_semantic_action_sample(
        fixture,
        sequence,
        CORE_PICKUP_ACTION_ID,
        phase,
        pressed,
        sampled_wall_time,
    )
}

pub fn player_equip_use_sample(
    fixture: &ReferenceGameSession,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, ReferenceInputError> {
    player_semantic_action_sample(
        fixture,
        sequence,
        CORE_EQUIP_USE_ACTION_ID,
        phase,
        pressed,
        sampled_wall_time,
    )
}

pub fn player_melee_sample(
    fixture: &ReferenceGameSession,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, ReferenceInputError> {
    player_semantic_action_sample(
        fixture,
        sequence,
        CORE_MELEE_ACTION_ID,
        phase,
        pressed,
        sampled_wall_time,
    )
}

fn player_semantic_action_sample(
    fixture: &ReferenceGameSession,
    sequence: u64,
    action_id: &str,
    phase: PlayerActionPhaseV1,
    pressed: bool,
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, ReferenceInputError> {
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: sequence,
        action_map_hash: fixture.action_map_hash,
        action_map_revision: fixture.action_map.revision,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: fixture.context_stack.revision,
        actions: vec![PlayerActionV1 {
            action_id: SchemaId::new(action_id)?,
            phase,
            value: PlayerActionValueV1::Digital(pressed),
            semantic_occurrence_ordinal: 0,
        }],
    };
    Ok(InputSampleV1 {
        schema_version: 1,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS)?,
        source_id: fixture.source_id,
        source_sequence: sequence,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID)?,
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes()?,
        sampled_wall_time,
    })
}

pub(crate) struct NormalizedReferenceInputV1 {
    session: PlayerInputSessionV1,
    next_platform_sequence: u64,
    held_movement_control: Option<&'static str>,
    pending_semantic_release: Option<&'static str>,
}

impl NormalizedReferenceInputV1 {
    pub(crate) fn new(fixture: &ReferenceGameSession) -> Result<Self, ReferenceGameError> {
        Ok(Self {
            session: PlayerInputSessionV1::new(
                fixture.controller_id,
                fixture.source_id,
                fixture.action_map.clone(),
                fixture.context_stack.clone(),
            )?,
            next_platform_sequence: 0,
            held_movement_control: None,
            pending_semantic_release: None,
        })
    }

    pub(crate) fn movement_sample(
        &mut self,
        logical_sequence: u64,
        direction_q15: [i16; 2],
    ) -> Result<InputSampleV1, ReferenceGameError> {
        let requested_control = match direction_q15 {
            [-32_767, 0] => Some(KEYBOARD_A_CONTROL_PATH_ID),
            [32_767, 0] => Some(KEYBOARD_D_CONTROL_PATH_ID),
            [0, -32_767] => Some(KEYBOARD_S_CONTROL_PATH_ID),
            [0, 32_767] => Some(KEYBOARD_W_CONTROL_PATH_ID),
            [0, 0] => None,
            _ => return Err(ReferenceGameError::InputFrameMissing),
        };
        let mut events = self.release_pending_semantic(logical_sequence)?;
        if self.held_movement_control != requested_control {
            if let Some(control_path) = self.held_movement_control.take() {
                events.push(self.keyboard_event(
                    logical_sequence,
                    control_path,
                    NormalizedControlPhaseV1::Completed,
                )?);
            }
            if let Some(control_path) = requested_control {
                events.push(self.keyboard_event(
                    logical_sequence,
                    control_path,
                    NormalizedControlPhaseV1::Started,
                )?);
                self.held_movement_control = Some(control_path);
            }
        }
        self.close(logical_sequence, events)
    }

    pub(crate) fn semantic_sample(
        &mut self,
        logical_sequence: u64,
        action_id: &str,
    ) -> Result<InputSampleV1, ReferenceGameError> {
        let control_path = match action_id {
            CORE_INTERACT_ACTION_ID => KEYBOARD_E_CONTROL_PATH_ID,
            CORE_PICKUP_ACTION_ID => KEYBOARD_Q_CONTROL_PATH_ID,
            CORE_EQUIP_USE_ACTION_ID => KEYBOARD_R_CONTROL_PATH_ID,
            CORE_MELEE_ACTION_ID => KEYBOARD_F_CONTROL_PATH_ID,
            _ => return Err(ReferenceGameError::InputFrameMissing),
        };
        let mut events = self.release_pending_semantic(logical_sequence)?;
        events.push(self.keyboard_event(
            logical_sequence,
            control_path,
            NormalizedControlPhaseV1::Started,
        )?);
        self.pending_semantic_release = Some(control_path);
        self.close(logical_sequence, events)
    }

    fn release_pending_semantic(
        &mut self,
        logical_sequence: u64,
    ) -> Result<Vec<PlatformEventV1>, ReferenceGameError> {
        let Some(control_path) = self.pending_semantic_release.take() else {
            return Ok(Vec::new());
        };
        Ok(vec![self.keyboard_event(
            logical_sequence,
            control_path,
            NormalizedControlPhaseV1::Completed,
        )?])
    }

    fn close(
        &mut self,
        logical_sequence: u64,
        events: Vec<PlatformEventV1>,
    ) -> Result<InputSampleV1, ReferenceGameError> {
        self.session.submit_platform_events(&events)?;
        self.session
            .close_frame(logical_sequence)?
            .resolved
            .map(|resolved| resolved.sample)
            .ok_or(ReferenceGameError::InputFrameMissing)
    }

    fn keyboard_event(
        &mut self,
        logical_sequence: u64,
        control_path: &'static str,
        phase: NormalizedControlPhaseV1,
    ) -> Result<PlatformEventV1, ReferenceGameError> {
        let source_sequence = self.next_platform_sequence;
        self.next_platform_sequence = self
            .next_platform_sequence
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        let control = NormalizedControlEventV1::new(
            SchemaId::new(KEYBOARD_DEVICE_CLASS_ID)?,
            PersistentId::from_bytes([0x71; 16]),
            SchemaId::new(control_path)?,
            phase,
            vec![if phase == NormalizedControlPhaseV1::Started {
                i16::MAX
            } else {
                0
            }],
            Vec::new(),
            logical_sequence,
            source_sequence,
        )?;
        Ok(PlatformEventV1::new(
            PersistentId::from_bytes([0x72; 16]),
            SchemaId::new("nextengine.platform.source.reference-headless")?,
            source_sequence,
            logical_sequence,
            PlatformEventKindV1::Control,
            PlatformEventPayloadV1::Control(control),
            ContentHash::from_bytes(next_contracts::canonical::sha256(
                b"nextengine.platform.reference-headless-capabilities.v1",
            )),
        )?)
    }
}

/// Returns the canonical-frame hash when the committed frame carries a
/// `ui-back` press — the replayable causal evidence for the declared pause
/// lifecycle request at the session boundary.
pub(crate) fn ui_suspend_causal_hash(
    frame: &PlayerActionFrameV1,
) -> Result<Option<ContentHash>, ReferenceGameError> {
    let requested = frame.actions.iter().any(|action| {
        action.action_id.as_str() == CORE_UI_BACK_ACTION_ID
            && action.phase == PlayerActionPhaseV1::Started
            && action.value == PlayerActionValueV1::Digital(true)
    });
    if !requested {
        return Ok(None);
    }
    Ok(Some(ContentHash::from_bytes(
        next_contracts::canonical::sha256(&frame.canonical_bytes()?),
    )))
}

/// Which read-only screen surface the live publication currently shows.
///
/// The state is presentation-only: it derives deterministically from the
/// committed `ui-inventory`/`ui-journal`/`ui-back` action stream (Q1A),
/// never mutates domain state and never suspends the simulation (Q5A).
/// Screens are exclusive — opening one replaces the other — so `ui-back`
/// always unambiguously closes the open screen; a `ui-back` press with no
/// open screen falls through to the declared pause suspend request.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReferenceUiScreenV1 {
    #[default]
    None,
    Inventory,
    Journal,
}

/// Outcome of applying one committed frame to the screen state.
pub(crate) struct UiScreenFrameOutcomeV1 {
    pub screen: ReferenceUiScreenV1,
    /// True when a committed `ui-back` press closed an open screen and is
    /// therefore consumed (the pause suspend request is suppressed).
    pub back_consumed_by_screen: bool,
}

/// Applies the committed `Started` digital UI screen actions in canonical
/// frame order. `ui-inventory`/`ui-journal` toggle their screen (opening one
/// closes the other); `ui-back` closes any open screen and is consumed.
pub(crate) fn apply_ui_screen_actions(
    frame: &PlayerActionFrameV1,
    initial: ReferenceUiScreenV1,
) -> UiScreenFrameOutcomeV1 {
    let mut screen = initial;
    let mut back_consumed_by_screen = false;
    for action in &frame.actions {
        if action.phase != PlayerActionPhaseV1::Started
            || action.value != PlayerActionValueV1::Digital(true)
        {
            continue;
        }
        match action.action_id.as_str() {
            CORE_UI_INVENTORY_ACTION_ID => {
                screen = match screen {
                    ReferenceUiScreenV1::Inventory => ReferenceUiScreenV1::None,
                    _ => ReferenceUiScreenV1::Inventory,
                };
            }
            CORE_UI_JOURNAL_ACTION_ID => {
                screen = match screen {
                    ReferenceUiScreenV1::Journal => ReferenceUiScreenV1::None,
                    _ => ReferenceUiScreenV1::Journal,
                };
            }
            CORE_UI_BACK_ACTION_ID if screen != ReferenceUiScreenV1::None => {
                screen = ReferenceUiScreenV1::None;
                back_consumed_by_screen = true;
            }
            _ => {}
        }
    }
    UiScreenFrameOutcomeV1 {
        screen,
        back_consumed_by_screen,
    }
}
