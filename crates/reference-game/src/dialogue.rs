//! Deterministic live dialogue arbitration (semantic UI S4, Q2A).
//!
//! The live driver owns the modal dialogue as presentation-only state derived
//! from the committed action stream, exactly like the read-only screen state
//! (S3). A committed `interact` press that targets the reference NPC opens the
//! dialogue instead of reaching the runtime; `ui-nav` moves the selection and
//! a committed `ui-confirm` either closes the dialogue (`Leave`) or queues the
//! `Accept` choice. The accepted choice is delivered back into the production
//! interaction path as a canonical `interact` action inside the runtime input
//! sample of the first frame resolved again under the gameplay context stack —
//! the runtime mapper, affordance selection and `AdvanceDialogueQuest` command
//! stay the single accept path (no new command, no UI authority over state).
//!
//! Context switching uses the production `PlayerInputSessionV1` queueing API:
//! while the dialogue is open the session runs a stack revision that layers the
//! modal `ui-dialogue` context above gameplay, so only the universal UI actions
//! resolve; closing queues the gameplay-only revision again. The runtime
//! controller registry is activated to each applied revision at the commit
//! boundary, keeping sample hashes, receipts and recovery aligned.

use next_contracts::ids::SchemaId;
use next_contracts::input::{
    CORE_INTERACT_ACTION_ID, CORE_MOVE_ACTION_ID, CORE_UI_BACK_ACTION_ID,
    CORE_UI_CONFIRM_ACTION_ID, CORE_UI_NAVIGATE_ACTION_ID, INPUT_SAMPLE_SCHEMA_VERSION,
    InputContextStackV1, InputContextV1, InputSampleV1, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS, PlayerActionFrameV1,
    PlayerActionPhaseV1, PlayerActionV1, PlayerActionValueV1,
};
use next_contracts::physics::{PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2};
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgSnapshotV2};
use next_player::{PlayerInputSessionV1, ResolvedPlayerInputFrameV1};

use crate::error::ReferenceGameError;
use crate::input::ReferenceUiScreenV1;
use crate::session::ReferenceGameSession;

/// Mirrors the runtime `interaction-nearby.v1` targeting profile; the driver
/// only opens the dialogue when the NPC is inside the same distance the
/// production affordance query would hit.
const INTERACTION_DISTANCE_MICROMETRES: i64 = 1_000_000;

/// The two authored choices of the reference dialogue offer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceDialogueChoiceV1 {
    Accept,
    Leave,
}

/// Presentation-only live dialogue state (S4).
///
/// `AcceptPending` means a committed `ui-confirm` selected `Accept`: the driver
/// injects the canonical `interact` into the runtime sample of the first frame
/// resolved again under the gameplay stack, completing the production accept
/// path. The state derives deterministically from the committed action stream
/// and the authoritative RPG/physics snapshots, never mutates domain state and
/// never suspends the simulation (Q5A).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReferenceDialogueUiV1 {
    #[default]
    Closed,
    Open {
        selection: ReferenceDialogueChoiceV1,
    },
    AcceptPending,
}

/// Outcome of applying one committed frame to the dialogue state.
pub(crate) struct DialogueFrameOutcomeV1 {
    pub state: ReferenceDialogueUiV1,
    /// True when a committed `ui-back` press closed the dialogue or cancelled
    /// a pending accept and is therefore consumed (the pause suspend request
    /// is suppressed, matching the screen consumption rule from S3).
    pub back_consumed_by_dialogue: bool,
}

/// The authored entry (offer) node of the reference dialogue definition.
pub fn reference_dialogue_entry_node_id(
    fixture: &ReferenceGameSession,
) -> Result<SchemaId, ReferenceGameError> {
    let definitions = &fixture.activated_project.rpg_definitions;
    let interaction = definitions
        .interactions
        .first()
        .ok_or(ReferenceGameError::RecoveryInvalid)?;
    let dialogue = definitions
        .dialogue(interaction.dialogue_definition)
        .ok_or(ReferenceGameError::RecoveryInvalid)?;
    Ok(dialogue
        .transitions
        .iter()
        .find(|transition| transition.transition_id == interaction.dialogue_transition_id)
        .ok_or(ReferenceGameError::RecoveryInvalid)?
        .source_state_id
        .clone())
}

/// The exact authored acceptance interaction represented by the reference
/// dialogue surface. Availability is queried from Runtime before the surface
/// opens, so presentation never invents a second routine-gating rule.
pub fn reference_dialogue_interaction_id(
    fixture: &ReferenceGameSession,
) -> Result<SchemaId, ReferenceGameError> {
    fixture
        .activated_project
        .rpg_definitions
        .interactions
        .first()
        .map(|interaction| interaction.interaction_id.clone())
        .ok_or(ReferenceGameError::RecoveryInvalid)
}

/// True while the dialogue aggregate sits at the authored offer node, which is
/// exactly when the runtime dialogue/quest closure reports the binding ready.
fn dialogue_offer_available(
    rpg: &RpgSnapshotV2,
    fixture: &ReferenceGameSession,
    entry_node_id: &SchemaId,
) -> bool {
    rpg.aggregates
        .iter()
        .find(|aggregate| {
            aggregate.aggregate_kind == RpgAggregateKindV1::Dialogue
                && aggregate.persistent_id == fixture.dialogue_id
        })
        .is_some_and(|aggregate| {
            matches!(
                &aggregate.payload,
                RpgAggregatePayloadV1::Dialogue(dialogue)
                    if &dialogue.node_id == entry_node_id
            )
        })
}

/// Squared horizontal (x/z) distance between two snapshot bodies.
fn squared_horizontal_distance(
    snapshot: &PhysicsCanonicalSnapshotV2,
    first_subject: next_contracts::ids::PersistentId,
    second_subject: next_contracts::ids::PersistentId,
) -> Result<i128, ReferenceGameError> {
    let body_id = |subject_id| PhysicsBodyIdV1 {
        subject_id,
        body_slot: 0,
    };
    let first = snapshot
        .sorted_body_states
        .get(&body_id(first_subject))
        .ok_or(ReferenceGameError::BodyMissing)?
        .pose
        .translation_micrometres;
    let second = snapshot
        .sorted_body_states
        .get(&body_id(second_subject))
        .ok_or(ReferenceGameError::BodyMissing)?
        .pose
        .translation_micrometres;
    let dx = i128::from(first[0] - second[0]);
    let dz = i128::from(first[2] - second[2]);
    Ok(dx * dx + dz * dz)
}

/// True when a committed `interact` press should open the dialogue instead of
/// reaching the runtime: the NPC is inside the production interaction distance
/// and strictly nearer than the reference switch, mirroring the affordance
/// candidate order on the committed physics snapshot. A `false` result leaves
/// `interact` on the legacy immediate path (switch activation, or the legacy
/// instant dialogue accept outside the mirrored region).
pub(crate) fn dialogue_interact_targets_npc(
    snapshot: &PhysicsCanonicalSnapshotV2,
    fixture: &ReferenceGameSession,
) -> Result<bool, ReferenceGameError> {
    let npc =
        squared_horizontal_distance(snapshot, fixture.body_id, fixture.quest_giver_character_id)?;
    let switch =
        squared_horizontal_distance(snapshot, fixture.body_id, fixture.interactive_object_id)?;
    let distance = i128::from(INTERACTION_DISTANCE_MICROMETRES);
    Ok(npc <= distance * distance && npc < switch)
}

/// True when a committed `interact` press opens the dialogue: the authored
/// offer is still available and the NPC is the mirrored interaction target.
pub(crate) fn dialogue_open_available(
    rpg: &RpgSnapshotV2,
    snapshot: &PhysicsCanonicalSnapshotV2,
    fixture: &ReferenceGameSession,
    entry_node_id: &SchemaId,
) -> Result<bool, ReferenceGameError> {
    if !dialogue_offer_available(rpg, fixture, entry_node_id) {
        return Ok(false);
    }
    dialogue_interact_targets_npc(snapshot, fixture)
}

/// Applies the committed dialogue-relevant actions in canonical frame order.
/// `ui-nav` clamps the selection (up/left toward `Accept`, down/right toward
/// `Leave`), `ui-confirm` settles the selected choice and `ui-back` closes the
/// dialogue or cancels a pending accept; both are consumed before the pause
/// suspend request.
pub(crate) fn apply_dialogue_frame_actions(
    frame: &PlayerActionFrameV1,
    initial: ReferenceDialogueUiV1,
) -> DialogueFrameOutcomeV1 {
    let mut state = initial;
    let mut back_consumed_by_dialogue = false;
    for action in &frame.actions {
        match action.action_id.as_str() {
            CORE_UI_NAVIGATE_ACTION_ID => {
                if let ReferenceDialogueUiV1::Open { selection } = state
                    && let PlayerActionValueV1::Vector2Q15(value) = action.value
                    && matches!(
                        action.phase,
                        PlayerActionPhaseV1::Started | PlayerActionPhaseV1::Performed
                    )
                {
                    state = ReferenceDialogueUiV1::Open {
                        selection: if value[1] > 0 || value[0] < 0 {
                            ReferenceDialogueChoiceV1::Accept
                        } else if value[1] < 0 || value[0] > 0 {
                            ReferenceDialogueChoiceV1::Leave
                        } else {
                            selection
                        },
                    };
                }
            }
            CORE_UI_CONFIRM_ACTION_ID => {
                if action.phase == PlayerActionPhaseV1::Started
                    && action.value == PlayerActionValueV1::Digital(true)
                    && let ReferenceDialogueUiV1::Open { selection } = state
                {
                    state = match selection {
                        ReferenceDialogueChoiceV1::Accept => ReferenceDialogueUiV1::AcceptPending,
                        ReferenceDialogueChoiceV1::Leave => ReferenceDialogueUiV1::Closed,
                    };
                }
            }
            CORE_UI_BACK_ACTION_ID
                if action.phase == PlayerActionPhaseV1::Started
                    && action.value == PlayerActionValueV1::Digital(true)
                    && state != ReferenceDialogueUiV1::Closed =>
            {
                state = ReferenceDialogueUiV1::Closed;
                back_consumed_by_dialogue = true;
            }
            _ => {}
        }
    }
    DialogueFrameOutcomeV1 {
        state,
        back_consumed_by_dialogue,
    }
}

/// True when the stack lets the modal UI actions resolve (dialogue layer on
/// top of gameplay), i.e. the session currently runs a dialogue revision.
pub(crate) fn stack_has_dialogue_layer(stack: &InputContextStackV1) -> bool {
    let Ok(ui_confirm) = SchemaId::new(CORE_UI_CONFIRM_ACTION_ID) else {
        return false;
    };
    stack.allows_action(&ui_confirm)
}

/// Queues the context-stack revision the dialogue state requires, when the
/// session is not already on it. Opening layers the modal `ui-dialogue`
/// context above gameplay; any closed/pending state returns to the
/// gameplay-only entries. The session applies the queued revision at its next
/// `close_frame`, so frames, receipts and recovery bytes stay consistent.
pub(crate) fn sync_context_stack(
    session: &mut PlayerInputSessionV1,
    state: ReferenceDialogueUiV1,
) -> Result<(), ReferenceGameError> {
    let wants_dialogue = matches!(state, ReferenceDialogueUiV1::Open { .. });
    if wants_dialogue == stack_has_dialogue_layer(session.context_stack()) {
        return Ok(());
    }
    let current = session.context_stack();
    let next_revision = current
        .revision
        .checked_add(1)
        .ok_or(ReferenceGameError::CountOverflow)?;
    let gameplay_entries = current
        .entries
        .iter()
        .filter(|entry| {
            entry.context_id.as_str() != next_contracts::input::CORE_UI_DIALOGUE_CONTEXT_ID
                && entry.context_id.as_str() != next_contracts::input::CORE_UI_MENU_CONTEXT_ID
        })
        .cloned()
        .collect::<Vec<_>>();
    let entries = if wants_dialogue {
        let modal = InputContextV1::new(
            SchemaId::new(next_contracts::input::CORE_UI_DIALOGUE_CONTEXT_ID)?,
            1,
            200,
            next_contracts::input::InputContextCapturePolicyV1::CaptureAll,
            vec![
                SchemaId::new(CORE_UI_BACK_ACTION_ID)?,
                SchemaId::new(CORE_UI_CONFIRM_ACTION_ID)?,
                SchemaId::new(CORE_UI_NAVIGATE_ACTION_ID)?,
            ],
        )?;
        let mut entries = vec![modal];
        entries.extend(gameplay_entries);
        entries
    } else {
        gameplay_entries
    };
    session.queue_context_stack(InputContextStackV1::new(
        current.stack_id.clone(),
        next_revision,
        entries,
    )?)?;
    Ok(())
}

/// Builds the runtime input sample for one closed frame under dialogue
/// arbitration.
///
/// The camera strip (presentation-only orbit) always applies. An opening frame
/// additionally strips `interact` and `move`: the committed press is consumed
/// by the dialogue and the avatar holds position so the confirm-time
/// affordance observes the same geometry as the open-time check. An accept
/// frame injects exactly one canonical `interact` `Started` (stripping any
/// real `interact` re-resolution) so the production mapper delivers the choice
/// through the existing accept path. When no physical frame resolved, an
/// accept frame is fabricated whole against the tag revision of the session.
///
/// `tag_session` is the pre-close session: frames are tagged with the context
/// stack that was live at the start of their `close_frame`, which is the
/// session state before this advance, not after it.
pub(crate) fn dialogue_runtime_sample(
    resolved: Option<&ResolvedPlayerInputFrameV1>,
    tag_session: &PlayerInputSessionV1,
    logical_frame_sequence: u64,
    strip_interaction_movement: bool,
    inject_interact: bool,
) -> Result<Option<InputSampleV1>, ReferenceGameError> {
    let interact_id = SchemaId::new(CORE_INTERACT_ACTION_ID)?;
    let mut frame = match resolved {
        Some(resolved) => {
            let mut frame = resolved.frame.clone();
            frame.actions.retain(|action| {
                action.action_id.as_str() != next_contracts::input::CORE_CAMERA_ORBIT_ACTION_ID
            });
            frame
        }
        None if inject_interact => PlayerActionFrameV1 {
            schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
            controller_id: tag_session.controller_id(),
            logical_frame_sequence,
            action_map_hash: tag_session.action_map().content_hash,
            action_map_revision: tag_session.action_map().revision,
            context_stack_hash: tag_session.context_stack().content_hash,
            context_stack_revision: tag_session.context_stack().revision,
            actions: Vec::new(),
        },
        None => return Ok(None),
    };
    if strip_interaction_movement {
        frame.actions.retain(|action| {
            action.action_id.as_str() != CORE_INTERACT_ACTION_ID
                && action.action_id.as_str() != CORE_MOVE_ACTION_ID
        });
    }
    if inject_interact {
        frame
            .actions
            .retain(|action| action.action_id != interact_id);
        frame.actions.push(PlayerActionV1 {
            action_id: interact_id,
            phase: PlayerActionPhaseV1::Started,
            value: PlayerActionValueV1::Digital(true),
            semantic_occurrence_ordinal: 0,
        });
    }
    if frame.actions.is_empty() {
        return Ok(None);
    }
    for (ordinal, action) in frame.actions.iter_mut().enumerate() {
        action.semantic_occurrence_ordinal =
            u32::try_from(ordinal).map_err(|_| ReferenceGameError::CountOverflow)?;
    }
    frame.validate_against(tag_session.action_map(), tag_session.context_stack())?;
    let payload = frame.canonical_bytes()?;
    let mut sample = match resolved {
        Some(resolved) => resolved.sample.clone(),
        None => InputSampleV1 {
            schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
            source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS)?,
            source_id: tag_session.source_id(),
            source_sequence: logical_frame_sequence,
            payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID)?,
            payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
            payload: Vec::new(),
            sampled_wall_time: None,
        },
    };
    sample.payload = payload;
    Ok(Some(sample))
}

/// Marks the exclusive-modal interaction between the dialogue and the
/// read-only screens: a dialogue that opens this frame closes any open screen.
pub(crate) fn screen_for_dialogue(
    screen: ReferenceUiScreenV1,
    opened: bool,
) -> ReferenceUiScreenV1 {
    if opened {
        ReferenceUiScreenV1::None
    } else {
        screen
    }
}
