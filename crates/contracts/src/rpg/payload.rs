use crate::canonical::{
    CanonicalCursor, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError,
    extend_u32_length_prefixed,
};
use crate::{PersistentId, SchemaId};

use super::model::{
    RPG_EVENT_DIALOGUE_QUEST_ADVANCED_SCHEMA_ID,
    RPG_EVENT_INTERACTIVE_OBJECT_STATE_CHANGED_SCHEMA_ID, RPG_EVENT_ITEM_TRANSFERRED_SCHEMA_ID,
    RPG_EVENT_SKILL_LEARNED_SCHEMA_ID, RpgDecodeError, SkillProficiency,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RpgCommand {
    AdvanceDialogueQuest {
        dialogue_id: PersistentId,
        expected_dialogue_node_id: SchemaId,
        next_dialogue_node_id: SchemaId,
        quest_id: PersistentId,
        expected_quest_state_id: SchemaId,
        next_quest_state_id: SchemaId,
        relationship_source: PersistentId,
        relationship_target: PersistentId,
        relationship_dimension_id: SchemaId,
        relationship_delta: i32,
    },
    TransferItem {
        item_id: PersistentId,
        expected_owner: Option<PersistentId>,
        new_owner: Option<PersistentId>,
    },
    LearnSkill {
        character_id: PersistentId,
        skill_id: SchemaId,
        delta: u16,
    },
    SetInteractiveObjectState {
        object_id: PersistentId,
        expected_state_id: SchemaId,
        next_state_id: SchemaId,
    },
}

impl RpgCommand {
    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        match self {
            Self::AdvanceDialogueQuest {
                dialogue_id,
                expected_dialogue_node_id,
                next_dialogue_node_id,
                quest_id,
                expected_quest_state_id,
                next_quest_state_id,
                relationship_source,
                relationship_target,
                relationship_dimension_id,
                relationship_delta,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(dialogue_id.as_bytes());
                extend_text(&mut bytes, expected_dialogue_node_id)?;
                extend_text(&mut bytes, next_dialogue_node_id)?;
                bytes.extend_from_slice(quest_id.as_bytes());
                extend_text(&mut bytes, expected_quest_state_id)?;
                extend_text(&mut bytes, next_quest_state_id)?;
                bytes.extend_from_slice(relationship_source.as_bytes());
                bytes.extend_from_slice(relationship_target.as_bytes());
                extend_text(&mut bytes, relationship_dimension_id)?;
                bytes.extend_from_slice(&relationship_delta.to_le_bytes());
            }
            Self::TransferItem {
                item_id,
                expected_owner,
                new_owner,
            } => {
                bytes.push(2);
                bytes.extend_from_slice(item_id.as_bytes());
                extend_optional_id(&mut bytes, *expected_owner);
                extend_optional_id(&mut bytes, *new_owner);
            }
            Self::LearnSkill {
                character_id,
                skill_id,
                delta,
            } => {
                bytes.push(3);
                bytes.extend_from_slice(character_id.as_bytes());
                extend_text(&mut bytes, skill_id)?;
                bytes.extend_from_slice(&delta.to_le_bytes());
            }
            Self::SetInteractiveObjectState {
                object_id,
                expected_state_id,
                next_state_id,
            } => {
                bytes.push(4);
                bytes.extend_from_slice(object_id.as_bytes());
                extend_text(&mut bytes, expected_state_id)?;
                extend_text(&mut bytes, next_state_id)?;
            }
        }
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgDecodeError> {
        if bytes.len() > limits.max_field_payload_bytes {
            return Err(RpgDecodeError::InputTooLarge {
                actual: bytes.len(),
                limit: limits.max_field_payload_bytes,
            });
        }
        let mut cursor = CanonicalCursor::new(bytes);
        let tag = cursor.read_u8()?;
        let command = match tag {
            1 => Self::AdvanceDialogueQuest {
                dialogue_id: read_id(&mut cursor)?,
                expected_dialogue_node_id: read_text(&mut cursor, limits)?,
                next_dialogue_node_id: read_text(&mut cursor, limits)?,
                quest_id: read_id(&mut cursor)?,
                expected_quest_state_id: read_text(&mut cursor, limits)?,
                next_quest_state_id: read_text(&mut cursor, limits)?,
                relationship_source: read_id(&mut cursor)?,
                relationship_target: read_id(&mut cursor)?,
                relationship_dimension_id: read_text(&mut cursor, limits)?,
                relationship_delta: read_i32(&mut cursor)?,
            },
            2 => Self::TransferItem {
                item_id: read_id(&mut cursor)?,
                expected_owner: read_optional_id(&mut cursor)?,
                new_owner: read_optional_id(&mut cursor)?,
            },
            3 => Self::LearnSkill {
                character_id: read_id(&mut cursor)?,
                skill_id: read_text(&mut cursor, limits)?,
                delta: read_u16(&mut cursor)?,
            },
            4 => Self::SetInteractiveObjectState {
                object_id: read_id(&mut cursor)?,
                expected_state_id: read_text(&mut cursor, limits)?,
                next_state_id: read_text(&mut cursor, limits)?,
            },
            _ => return Err(RpgDecodeError::UnknownCommandTag(tag)),
        };
        cursor.finish()?;
        if command.canonical_payload_bytes()? != bytes {
            return Err(RpgDecodeError::NonCanonicalEncoding);
        }
        Ok(command)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RpgEvent {
    DialogueQuestAdvanced {
        dialogue_id: PersistentId,
        dialogue_node_id: SchemaId,
        quest_id: PersistentId,
        quest_state_id: SchemaId,
        relationship_source: PersistentId,
        relationship_target: PersistentId,
        relationship_dimension_id: SchemaId,
        relationship_value: i32,
    },
    ItemTransferred {
        item_id: PersistentId,
        previous_owner: Option<PersistentId>,
        new_owner: Option<PersistentId>,
    },
    SkillLearned {
        character_id: PersistentId,
        skill_id: SchemaId,
        proficiency: SkillProficiency,
    },
    InteractiveObjectStateChanged {
        object_id: PersistentId,
        state_id: SchemaId,
    },
}

impl RpgEvent {
    #[must_use]
    pub const fn schema_id(&self) -> &'static str {
        match self {
            Self::DialogueQuestAdvanced { .. } => RPG_EVENT_DIALOGUE_QUEST_ADVANCED_SCHEMA_ID,
            Self::ItemTransferred { .. } => RPG_EVENT_ITEM_TRANSFERRED_SCHEMA_ID,
            Self::SkillLearned { .. } => RPG_EVENT_SKILL_LEARNED_SCHEMA_ID,
            Self::InteractiveObjectStateChanged { .. } => {
                RPG_EVENT_INTERACTIVE_OBJECT_STATE_CHANGED_SCHEMA_ID
            }
        }
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        match self {
            Self::DialogueQuestAdvanced {
                dialogue_id,
                dialogue_node_id,
                quest_id,
                quest_state_id,
                relationship_source,
                relationship_target,
                relationship_dimension_id,
                relationship_value,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(dialogue_id.as_bytes());
                extend_text(&mut bytes, dialogue_node_id)?;
                bytes.extend_from_slice(quest_id.as_bytes());
                extend_text(&mut bytes, quest_state_id)?;
                bytes.extend_from_slice(relationship_source.as_bytes());
                bytes.extend_from_slice(relationship_target.as_bytes());
                extend_text(&mut bytes, relationship_dimension_id)?;
                bytes.extend_from_slice(&relationship_value.to_le_bytes());
            }
            Self::ItemTransferred {
                item_id,
                previous_owner,
                new_owner,
            } => {
                bytes.push(2);
                bytes.extend_from_slice(item_id.as_bytes());
                extend_optional_id(&mut bytes, *previous_owner);
                extend_optional_id(&mut bytes, *new_owner);
            }
            Self::SkillLearned {
                character_id,
                skill_id,
                proficiency,
            } => {
                bytes.push(3);
                bytes.extend_from_slice(character_id.as_bytes());
                extend_text(&mut bytes, skill_id)?;
                bytes.extend_from_slice(&proficiency.get().to_le_bytes());
            }
            Self::InteractiveObjectStateChanged {
                object_id,
                state_id,
            } => {
                bytes.push(4);
                bytes.extend_from_slice(object_id.as_bytes());
                extend_text(&mut bytes, state_id)?;
            }
        }
        Ok(bytes)
    }
}

pub(super) fn extend_text(target: &mut Vec<u8>, value: &SchemaId) -> Result<(), CanonicalError> {
    extend_u32_length_prefixed(target, value.as_str().as_bytes())
}

pub(super) fn extend_optional_id(target: &mut Vec<u8>, value: Option<PersistentId>) {
    target.push(u8::from(value.is_some()));
    if let Some(value) = value {
        target.extend_from_slice(value.as_bytes());
    }
}

pub(super) fn read_id(cursor: &mut CanonicalCursor<'_>) -> Result<PersistentId, RpgDecodeError> {
    let bytes: [u8; 16] = cursor
        .read_exact(16)?
        .try_into()
        .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
    Ok(PersistentId::from_bytes(bytes))
}

pub(super) fn read_optional_id(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<Option<PersistentId>, RpgDecodeError> {
    match cursor.read_u8()? {
        0 => Ok(None),
        1 => Ok(Some(read_id(cursor)?)),
        _ => Err(RpgDecodeError::InvalidOptionalId),
    }
}

pub(super) fn read_text(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<SchemaId, RpgDecodeError> {
    let bytes = cursor.read_u32_length_prefixed(limits.max_identifier_bytes)?;
    let value = std::str::from_utf8(bytes).map_err(|_| CanonicalDecodeError::InvalidUtf8)?;
    Ok(SchemaId::new(value)?)
}

pub(super) fn read_u16(cursor: &mut CanonicalCursor<'_>) -> Result<u16, RpgDecodeError> {
    let bytes: [u8; 2] = cursor
        .read_exact(2)?
        .try_into()
        .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
    Ok(u16::from_le_bytes(bytes))
}

pub(super) fn read_i32(cursor: &mut CanonicalCursor<'_>) -> Result<i32, RpgDecodeError> {
    let bytes: [u8; 4] = cursor
        .read_exact(4)?
        .try_into()
        .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
    Ok(i32::from_le_bytes(bytes))
}
