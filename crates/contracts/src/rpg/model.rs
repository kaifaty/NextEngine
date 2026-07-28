use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{CanonicalDecodeError, CanonicalError};
use crate::{IdentifierError, PersistentId, SchemaId};

pub const RPG_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const RPG_SNAPSHOT_OWNER_ID: &str = "rpg";
pub const RPG_SNAPSHOT_SCHEMA_ID: &str = "nextengine.rpg.snapshot";
pub const RPG_SNAPSHOT_SEGMENT_ID: &str = "domain-state";
pub const RPG_COMMAND_SCHEMA_ID: &str = "nextengine.command.rpg";
pub const RPG_COMMAND_CAPABILITY_ID: &str = "rpg.command.propose";
pub const CORE_INTERACTIVE_OBJECT_ARCHETYPE_ID: &str = "nextengine.rpg.interactive.core-switch";
pub const CORE_INTERACTIVE_OBJECT_READY_STATE_ID: &str = "nextengine.rpg.interactive.ready";
pub const CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID: &str = "nextengine.rpg.interactive.activated";
pub const CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID: &str = "nextengine.rpg.interactive.collected";
pub const CORE_EQUIPMENT_MAIN_HAND_SLOT_ID: &str = "nextengine.rpg.equipment-slot.main-hand";
pub const CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID: &str =
    "nextengine.rpg.character.core-quest-giver";
pub const CORE_HELP_DIALOGUE_DEFINITION_ID: &str = "nextengine.rpg.dialogue.core-help";
pub const CORE_DIALOGUE_OFFER_NODE_ID: &str = "nextengine.rpg.dialogue-node.offer";
pub const CORE_DIALOGUE_ACCEPTED_NODE_ID: &str = "nextengine.rpg.dialogue-node.accepted";
pub const CORE_HELP_QUEST_DEFINITION_ID: &str = "nextengine.rpg.quest.core-help";
pub const CORE_QUEST_AVAILABLE_STATE_ID: &str = "nextengine.rpg.quest-state.available";
pub const CORE_QUEST_ACTIVE_STATE_ID: &str = "nextengine.rpg.quest-state.active";
pub const CORE_RELATIONSHIP_TRUST_DIMENSION_ID: &str = "nextengine.rpg.relationship.trust";
pub const CORE_DIALOGUE_QUEST_TRUST_DELTA: i32 = 7;

pub const RPG_EVENT_DIALOGUE_QUEST_ADVANCED_SCHEMA_ID: &str =
    "nextengine.event.rpg.dialogue-quest-advanced";
pub const RPG_EVENT_ITEM_TRANSFERRED_SCHEMA_ID: &str = "nextengine.event.rpg.item-transferred";
pub const RPG_EVENT_SKILL_LEARNED_SCHEMA_ID: &str = "nextengine.event.rpg.skill-learned";
pub const RPG_EVENT_INTERACTIVE_OBJECT_STATE_CHANGED_SCHEMA_ID: &str =
    "nextengine.event.rpg.interactive-object-state-changed";

pub const SKILL_PROFICIENCY_MAX: u16 = 10_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SkillProficiency(u16);

impl SkillProficiency {
    pub fn new(value: u16) -> Result<Self, SkillProficiencyError> {
        if value > SKILL_PROFICIENCY_MAX {
            return Err(SkillProficiencyError::OutOfRange(value));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillProficiencyError {
    OutOfRange(u16),
}

impl Display for SkillProficiencyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfRange(value) => {
                write!(
                    formatter,
                    "skill proficiency {value} is outside 0..={SKILL_PROFICIENCY_MAX}"
                )
            }
        }
    }
}

impl Error for SkillProficiencyError {}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SkillProficiencyEntry {
    pub skill_id: SchemaId,
    pub proficiency: SkillProficiency,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationshipEntry {
    pub target: PersistentId,
    pub dimension_id: SchemaId,
    pub value: i32,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CharacterSnapshot {
    pub id: PersistentId,
    pub revision: u64,
    pub archetype_id: SchemaId,
    pub skills: Vec<SkillProficiencyEntry>,
    pub relationships: Vec<RelationshipEntry>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ItemSnapshot {
    pub id: PersistentId,
    pub revision: u64,
    pub archetype_id: SchemaId,
    pub owner: Option<PersistentId>,
    pub quantity: u32,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct QuestSnapshot {
    pub id: PersistentId,
    pub revision: u64,
    pub definition_id: SchemaId,
    pub state_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DialogueSnapshot {
    pub id: PersistentId,
    pub revision: u64,
    pub definition_id: SchemaId,
    pub speaker: PersistentId,
    pub listener: PersistentId,
    pub node_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FactionSnapshot {
    pub id: PersistentId,
    pub revision: u64,
    pub definition_id: SchemaId,
    pub members: Vec<PersistentId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InteractiveObjectSnapshot {
    pub id: PersistentId,
    pub revision: u64,
    pub archetype_id: SchemaId,
    pub state_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldChunkRecordSnapshot {
    pub id: PersistentId,
    pub revision: u64,
    pub record_schema_id: SchemaId,
    pub state_id: SchemaId,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RpgSnapshot {
    pub characters: Vec<CharacterSnapshot>,
    pub items: Vec<ItemSnapshot>,
    pub quests: Vec<QuestSnapshot>,
    pub dialogues: Vec<DialogueSnapshot>,
    pub factions: Vec<FactionSnapshot>,
    pub interactive_objects: Vec<InteractiveObjectSnapshot>,
    pub world_chunk_records: Vec<WorldChunkRecordSnapshot>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreDialogueQuestProfileState {
    Ready,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedCoreDialogueQuestBinding {
    pub npc_id: PersistentId,
    pub player_id: PersistentId,
    pub dialogue_id: PersistentId,
    pub quest_id: PersistentId,
    pub state: CoreDialogueQuestProfileState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreDialogueQuestClosureError;

impl CoreDialogueQuestClosureError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        "RPG_CORE_INTERACTION_CLOSURE_INVALID"
    }
}

impl Display for CoreDialogueQuestClosureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for CoreDialogueQuestClosureError {}

impl RpgSnapshot {
    #[must_use]
    pub fn has_core_dialogue_quest_records(&self) -> bool {
        self.characters.iter().any(|character| {
            character.archetype_id.as_str() == CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID
        }) || self
            .dialogues
            .iter()
            .any(|dialogue| dialogue.definition_id.as_str() == CORE_HELP_DIALOGUE_DEFINITION_ID)
            || self
                .quests
                .iter()
                .any(|quest| quest.definition_id.as_str() == CORE_HELP_QUEST_DEFINITION_ID)
    }

    pub fn resolve_core_dialogue_quest_binding(
        &self,
        controlled_character_id: PersistentId,
    ) -> Result<Option<ResolvedCoreDialogueQuestBinding>, CoreDialogueQuestClosureError> {
        if !self.has_core_dialogue_quest_records() {
            return Ok(None);
        }

        let mut quest_givers = self.characters.iter().filter(|character| {
            character.archetype_id.as_str() == CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID
        });
        let npc = quest_givers.next().ok_or(CoreDialogueQuestClosureError)?;
        if quest_givers.next().is_some() {
            return Err(CoreDialogueQuestClosureError);
        }

        let mut dialogues = self
            .dialogues
            .iter()
            .filter(|dialogue| dialogue.definition_id.as_str() == CORE_HELP_DIALOGUE_DEFINITION_ID);
        let dialogue = dialogues.next().ok_or(CoreDialogueQuestClosureError)?;
        if dialogues.next().is_some() {
            return Err(CoreDialogueQuestClosureError);
        }

        let mut quests = self
            .quests
            .iter()
            .filter(|quest| quest.definition_id.as_str() == CORE_HELP_QUEST_DEFINITION_ID);
        let quest = quests.next().ok_or(CoreDialogueQuestClosureError)?;
        if quests.next().is_some()
            || !self
                .characters
                .iter()
                .any(|character| character.id == controlled_character_id)
            || dialogue.speaker != npc.id
            || dialogue.listener != controlled_character_id
        {
            return Err(CoreDialogueQuestClosureError);
        }

        let trust_value = npc
            .relationships
            .iter()
            .find(|relationship| {
                relationship.target == controlled_character_id
                    && relationship.dimension_id.as_str() == CORE_RELATIONSHIP_TRUST_DIMENSION_ID
            })
            .map_or(0, |relationship| relationship.value);
        let state = match (
            dialogue.node_id.as_str(),
            quest.state_id.as_str(),
            trust_value,
        ) {
            (CORE_DIALOGUE_OFFER_NODE_ID, CORE_QUEST_AVAILABLE_STATE_ID, 0) => {
                CoreDialogueQuestProfileState::Ready
            }
            (
                CORE_DIALOGUE_ACCEPTED_NODE_ID,
                CORE_QUEST_ACTIVE_STATE_ID,
                CORE_DIALOGUE_QUEST_TRUST_DELTA,
            ) => CoreDialogueQuestProfileState::Completed,
            _ => return Err(CoreDialogueQuestClosureError),
        };

        Ok(Some(ResolvedCoreDialogueQuestBinding {
            npc_id: npc.id,
            player_id: controlled_character_id,
            dialogue_id: dialogue.id,
            quest_id: quest.id,
            state,
        }))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgDecodeError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Identifier(IdentifierError),
    InputTooLarge {
        actual: usize,
        limit: usize,
    },
    WrongSnapshotEnvelope,
    UnknownSnapshotField(u32),
    MissingSnapshotField(u32),
    SnapshotFieldType {
        field_id: u32,
        expected: u8,
        actual: u8,
    },
    SnapshotFieldLength {
        field_id: u32,
        expected: usize,
        actual: usize,
    },
    UnsupportedSnapshotVersion(u32),
    TooManyRecords {
        actual: usize,
        limit: usize,
    },
    InvalidOptionalId,
    InvalidSkillProficiency(u16),
    UnknownCommandTag(u8),
    NonCanonicalEncoding,
}

impl Display for RpgDecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "RPG encoding is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "RPG canonicalization failed: {error}")
            }
            Self::Identifier(error) => write!(formatter, "RPG identifier is invalid: {error}"),
            Self::InputTooLarge { actual, limit } => {
                write!(
                    formatter,
                    "RPG payload has {actual} bytes; limit is {limit}"
                )
            }
            Self::WrongSnapshotEnvelope => formatter.write_str("RPG snapshot envelope is invalid"),
            Self::UnknownSnapshotField(field_id) => {
                write!(formatter, "unknown RPG snapshot field {field_id}")
            }
            Self::MissingSnapshotField(field_id) => {
                write!(formatter, "missing RPG snapshot field {field_id}")
            }
            Self::SnapshotFieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "RPG snapshot field {field_id} has type {actual}; expected {expected}"
            ),
            Self::SnapshotFieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "RPG snapshot field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnsupportedSnapshotVersion(version) => {
                write!(formatter, "unsupported RPG snapshot version {version}")
            }
            Self::TooManyRecords { actual, limit } => {
                write!(
                    formatter,
                    "RPG sequence has {actual} records; limit is {limit}"
                )
            }
            Self::InvalidOptionalId => formatter.write_str("RPG optional id is malformed"),
            Self::InvalidSkillProficiency(value) => {
                write!(formatter, "RPG skill proficiency {value} is out of range")
            }
            Self::UnknownCommandTag(tag) => write!(formatter, "unknown RPG command tag {tag}"),
            Self::NonCanonicalEncoding => {
                formatter.write_str("RPG value does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for RpgDecodeError {}

impl From<CanonicalDecodeError> for RpgDecodeError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for RpgDecodeError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<IdentifierError> for RpgDecodeError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}
