use next_contracts::ids::SchemaId;
use next_contracts::input::{
    CORE_EQUIP_USE_ACTION_ID, CORE_INTERACT_ACTION_ID, CORE_MELEE_ACTION_ID, CORE_MOVE_ACTION_ID,
    CORE_PICKUP_ACTION_ID, InputSampleV1, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS, PlayerActionFrameV1,
    PlayerActionPhaseV1, PlayerActionV1, PlayerActionValueV1,
};

use crate::{ReferenceGameSession, ReferenceInputError};

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
        action_map_revision: 1,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: 1,
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
        action_map_revision: 1,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: 1,
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
