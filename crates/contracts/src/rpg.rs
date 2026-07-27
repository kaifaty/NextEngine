use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U32, CanonicalCursor, CanonicalDecodeError,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, DecodedCanonicalSegment,
    decode_canonical_segment, encode_canonical_segment, extend_u32_length_prefixed,
};
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

impl RpgSnapshot {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RPG_SNAPSHOT_OWNER_ID,
            RPG_SNAPSHOT_SCHEMA_ID,
            RPG_SNAPSHOT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    RPG_SNAPSHOT_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_characters(&self.characters)?,
                ),
                CanonicalField::new(3, CANONICAL_TYPE_SEQUENCE, encode_items(&self.items)?),
                CanonicalField::new(4, CANONICAL_TYPE_SEQUENCE, encode_quests(&self.quests)?),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_dialogues(&self.dialogues)?,
                ),
                CanonicalField::new(6, CANONICAL_TYPE_SEQUENCE, encode_factions(&self.factions)?),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_interactive_objects(&self.interactive_objects)?,
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_world_chunk_records(&self.world_chunk_records)?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgDecodeError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_snapshot_envelope(&segment)?;
        validate_snapshot_fields(&segment)?;
        let version = decode_u32_field(&segment, 1)?;
        if version != RPG_SNAPSHOT_SCHEMA_VERSION {
            return Err(RpgDecodeError::UnsupportedSnapshotVersion(version));
        }
        let snapshot = Self {
            characters: decode_characters(field_payload(&segment, 2)?, limits)?,
            items: decode_items(field_payload(&segment, 3)?, limits)?,
            quests: decode_quests(field_payload(&segment, 4)?, limits)?,
            dialogues: decode_dialogues(field_payload(&segment, 5)?, limits)?,
            factions: decode_factions(field_payload(&segment, 6)?, limits)?,
            interactive_objects: decode_interactive_objects(field_payload(&segment, 7)?, limits)?,
            world_chunk_records: decode_world_chunk_records(field_payload(&segment, 8)?, limits)?,
        };
        if snapshot.canonical_bytes()? != bytes {
            return Err(RpgDecodeError::NonCanonicalEncoding);
        }
        Ok(snapshot)
    }
}

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

fn encode_characters(records: &[CharacterSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.archetype_id)?;

        let mut skills = record.skills.clone();
        skills.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));
        ensure_unique_by(&skills, |entry| entry.skill_id.clone())?;
        extend_count(bytes, skills.len())?;
        for skill in skills {
            extend_text(bytes, &skill.skill_id)?;
            bytes.extend_from_slice(&skill.proficiency.get().to_le_bytes());
        }

        let mut relationships = record.relationships.clone();
        relationships.sort_by(|left, right| {
            (&left.target, &left.dimension_id).cmp(&(&right.target, &right.dimension_id))
        });
        ensure_unique_by(&relationships, |entry| {
            (entry.target, entry.dimension_id.clone())
        })?;
        extend_count(bytes, relationships.len())?;
        for relationship in relationships {
            bytes.extend_from_slice(relationship.target.as_bytes());
            extend_text(bytes, &relationship.dimension_id)?;
            bytes.extend_from_slice(&relationship.value.to_le_bytes());
        }
        Ok(())
    })
}

fn encode_items(records: &[ItemSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.archetype_id)?;
        extend_optional_id(bytes, record.owner);
        bytes.extend_from_slice(&record.quantity.to_le_bytes());
        Ok(())
    })
}

fn encode_quests(records: &[QuestSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.definition_id)?;
        extend_text(bytes, &record.state_id)?;
        Ok(())
    })
}

fn encode_dialogues(records: &[DialogueSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.definition_id)?;
        bytes.extend_from_slice(record.speaker.as_bytes());
        bytes.extend_from_slice(record.listener.as_bytes());
        extend_text(bytes, &record.node_id)?;
        Ok(())
    })
}

fn encode_factions(records: &[FactionSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.definition_id)?;
        let mut members = record.members.clone();
        members.sort();
        ensure_unique_by(&members, |member| *member)?;
        extend_count(bytes, members.len())?;
        for member in members {
            bytes.extend_from_slice(member.as_bytes());
        }
        Ok(())
    })
}

fn encode_interactive_objects(
    records: &[InteractiveObjectSnapshot],
) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.archetype_id)?;
        extend_text(bytes, &record.state_id)?;
        Ok(())
    })
}

fn encode_world_chunk_records(
    records: &[WorldChunkRecordSnapshot],
) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.record_schema_id)?;
        extend_text(bytes, &record.state_id)?;
        Ok(())
    })
}

fn encode_records<T>(
    records: &[T],
    id_and_encode: impl Fn(&mut Vec<u8>, &T) -> Result<(), CanonicalError>,
) -> Result<Vec<u8>, CanonicalError>
where
    T: Clone + HasPersistentId,
{
    let mut records = records.to_vec();
    records.sort_by_key(HasPersistentId::persistent_id);
    ensure_unique_by(&records, HasPersistentId::persistent_id)?;
    let mut bytes = Vec::new();
    extend_count(&mut bytes, records.len())?;
    for record in &records {
        id_and_encode(&mut bytes, record)?;
    }
    Ok(bytes)
}

trait HasPersistentId {
    fn persistent_id(&self) -> PersistentId;
}

macro_rules! impl_has_persistent_id {
    ($($type:ty),+ $(,)?) => {
        $(
            impl HasPersistentId for $type {
                fn persistent_id(&self) -> PersistentId {
                    self.id
                }
            }
        )+
    };
}

impl_has_persistent_id!(
    CharacterSnapshot,
    ItemSnapshot,
    QuestSnapshot,
    DialogueSnapshot,
    FactionSnapshot,
    InteractiveObjectSnapshot,
    WorldChunkRecordSnapshot,
);

fn decode_characters(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CharacterSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        let id = read_id(cursor)?;
        let revision = cursor.read_u64()?;
        let archetype_id = read_text(cursor, limits)?;
        let skill_count = read_count(cursor, limits)?;
        let mut skills = Vec::with_capacity(skill_count);
        for _ in 0..skill_count {
            let skill_id = read_text(cursor, limits)?;
            let raw = read_u16(cursor)?;
            let proficiency = SkillProficiency::new(raw)
                .map_err(|_| RpgDecodeError::InvalidSkillProficiency(raw))?;
            skills.push(SkillProficiencyEntry {
                skill_id,
                proficiency,
            });
        }
        let relationship_count = read_count(cursor, limits)?;
        let mut relationships = Vec::with_capacity(relationship_count);
        for _ in 0..relationship_count {
            relationships.push(RelationshipEntry {
                target: read_id(cursor)?,
                dimension_id: read_text(cursor, limits)?,
                value: read_i32(cursor)?,
            });
        }
        Ok(CharacterSnapshot {
            id,
            revision,
            archetype_id,
            skills,
            relationships,
        })
    })
}

fn decode_items(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<ItemSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(ItemSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            archetype_id: read_text(cursor, limits)?,
            owner: read_optional_id(cursor)?,
            quantity: cursor.read_u32()?,
        })
    })
}

fn decode_quests(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<QuestSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(QuestSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            definition_id: read_text(cursor, limits)?,
            state_id: read_text(cursor, limits)?,
        })
    })
}

fn decode_dialogues(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<DialogueSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(DialogueSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            definition_id: read_text(cursor, limits)?,
            speaker: read_id(cursor)?,
            listener: read_id(cursor)?,
            node_id: read_text(cursor, limits)?,
        })
    })
}

fn decode_factions(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<FactionSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        let id = read_id(cursor)?;
        let revision = cursor.read_u64()?;
        let definition_id = read_text(cursor, limits)?;
        let member_count = read_count(cursor, limits)?;
        let mut members = Vec::with_capacity(member_count);
        for _ in 0..member_count {
            members.push(read_id(cursor)?);
        }
        Ok(FactionSnapshot {
            id,
            revision,
            definition_id,
            members,
        })
    })
}

fn decode_interactive_objects(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<InteractiveObjectSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(InteractiveObjectSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            archetype_id: read_text(cursor, limits)?,
            state_id: read_text(cursor, limits)?,
        })
    })
}

fn decode_world_chunk_records(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<WorldChunkRecordSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(WorldChunkRecordSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            record_schema_id: read_text(cursor, limits)?,
            state_id: read_text(cursor, limits)?,
        })
    })
}

fn decode_records<T>(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    decode: impl Fn(&mut CanonicalCursor<'_>) -> Result<T, RpgDecodeError>,
) -> Result<Vec<T>, RpgDecodeError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits)?;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        records.push(decode(&mut cursor)?);
    }
    cursor.finish()?;
    Ok(records)
}

fn validate_snapshot_envelope(segment: &DecodedCanonicalSegment) -> Result<(), RpgDecodeError> {
    if segment.owner_id != RPG_SNAPSHOT_OWNER_ID
        || segment.schema_id != RPG_SNAPSHOT_SCHEMA_ID
        || segment.segment_id != RPG_SNAPSHOT_SEGMENT_ID
    {
        return Err(RpgDecodeError::WrongSnapshotEnvelope);
    }
    Ok(())
}

fn validate_snapshot_fields(segment: &DecodedCanonicalSegment) -> Result<(), RpgDecodeError> {
    const EXPECTED: [(u32, u8); 8] = [
        (1, CANONICAL_TYPE_U32),
        (2, CANONICAL_TYPE_SEQUENCE),
        (3, CANONICAL_TYPE_SEQUENCE),
        (4, CANONICAL_TYPE_SEQUENCE),
        (5, CANONICAL_TYPE_SEQUENCE),
        (6, CANONICAL_TYPE_SEQUENCE),
        (7, CANONICAL_TYPE_SEQUENCE),
        (8, CANONICAL_TYPE_SEQUENCE),
    ];
    for field in &segment.fields {
        if !EXPECTED
            .iter()
            .any(|(field_id, _)| *field_id == field.field_id)
        {
            return Err(RpgDecodeError::UnknownSnapshotField(field.field_id));
        }
    }
    for (field_id, expected_type) in EXPECTED {
        let field = segment
            .field(field_id)
            .ok_or(RpgDecodeError::MissingSnapshotField(field_id))?;
        if field.type_tag != expected_type {
            return Err(RpgDecodeError::SnapshotFieldType {
                field_id,
                expected: expected_type,
                actual: field.type_tag,
            });
        }
    }
    Ok(())
}

fn field_payload(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<&[u8], RpgDecodeError> {
    segment
        .field(field_id)
        .map(|field| field.payload.as_slice())
        .ok_or(RpgDecodeError::MissingSnapshotField(field_id))
}

fn decode_u32_field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<u32, RpgDecodeError> {
    let payload = field_payload(segment, field_id)?;
    let bytes: [u8; 4] = payload
        .try_into()
        .map_err(|_| RpgDecodeError::SnapshotFieldLength {
            field_id,
            expected: 4,
            actual: payload.len(),
        })?;
    Ok(u32::from_le_bytes(bytes))
}

fn extend_text(target: &mut Vec<u8>, value: &SchemaId) -> Result<(), CanonicalError> {
    extend_u32_length_prefixed(target, value.as_str().as_bytes())
}

fn extend_optional_id(target: &mut Vec<u8>, value: Option<PersistentId>) {
    target.push(u8::from(value.is_some()));
    if let Some(value) = value {
        target.extend_from_slice(value.as_bytes());
    }
}

fn extend_count(target: &mut Vec<u8>, count: usize) -> Result<(), CanonicalError> {
    target.extend_from_slice(
        &u32::try_from(count)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

fn ensure_unique_by<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> Result<(), CanonicalError> {
    if values.windows(2).any(|pair| key(&pair[0]) == key(&pair[1])) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    Ok(())
}

fn read_id(cursor: &mut CanonicalCursor<'_>) -> Result<PersistentId, RpgDecodeError> {
    let bytes: [u8; 16] = cursor
        .read_exact(16)?
        .try_into()
        .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
    Ok(PersistentId::from_bytes(bytes))
}

fn read_optional_id(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<Option<PersistentId>, RpgDecodeError> {
    match cursor.read_u8()? {
        0 => Ok(None),
        1 => Ok(Some(read_id(cursor)?)),
        _ => Err(RpgDecodeError::InvalidOptionalId),
    }
}

fn read_text(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<SchemaId, RpgDecodeError> {
    let bytes = cursor.read_u32_length_prefixed(limits.max_identifier_bytes)?;
    let value = std::str::from_utf8(bytes).map_err(|_| CanonicalDecodeError::InvalidUtf8)?;
    Ok(SchemaId::new(value)?)
}

fn read_count(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<usize, RpgDecodeError> {
    let count =
        usize::try_from(cursor.read_u32()?).map_err(|_| CanonicalDecodeError::LengthOverflow)?;
    if count > limits.max_sequence_items {
        return Err(RpgDecodeError::TooManyRecords {
            actual: count,
            limit: limits.max_sequence_items,
        });
    }
    Ok(count)
}

fn read_u16(cursor: &mut CanonicalCursor<'_>) -> Result<u16, RpgDecodeError> {
    let bytes: [u8; 2] = cursor
        .read_exact(2)?
        .try_into()
        .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_i32(cursor: &mut CanonicalCursor<'_>) -> Result<i32, RpgDecodeError> {
    let bytes: [u8; 4] = cursor
        .read_exact(4)?
        .try_into()
        .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
    Ok(i32::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::{
        CharacterSnapshot, RelationshipEntry, RpgCommand, RpgSnapshot, SkillProficiency,
        SkillProficiencyEntry,
    };
    use crate::{CanonicalDecodeLimits, PersistentId, SchemaId};

    fn schema(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test schema id is valid")
    }

    #[test]
    fn snapshot_round_trip_is_byte_exact_and_order_independent() {
        let first = CharacterSnapshot {
            id: PersistentId::from_bytes([1; 16]),
            revision: 3,
            archetype_id: schema("rpg.character.generic"),
            skills: vec![
                SkillProficiencyEntry {
                    skill_id: schema("rpg.skill.alchemy"),
                    proficiency: SkillProficiency::new(200).expect("bounded"),
                },
                SkillProficiencyEntry {
                    skill_id: schema("rpg.skill.survival"),
                    proficiency: SkillProficiency::new(500).expect("bounded"),
                },
            ],
            relationships: vec![RelationshipEntry {
                target: PersistentId::from_bytes([2; 16]),
                dimension_id: schema("rpg.relationship.trust"),
                value: 7,
            }],
        };
        let second = CharacterSnapshot {
            id: PersistentId::from_bytes([2; 16]),
            revision: 0,
            archetype_id: schema("rpg.character.generic"),
            skills: vec![],
            relationships: vec![],
        };
        let left = RpgSnapshot {
            characters: vec![first.clone(), second.clone()],
            ..RpgSnapshot::default()
        };
        let right = RpgSnapshot {
            characters: vec![second, first],
            ..RpgSnapshot::default()
        };

        let bytes = left.canonical_bytes().expect("snapshot is canonical");
        assert_eq!(
            bytes,
            right.canonical_bytes().expect("order is canonicalized")
        );
        assert_eq!(
            RpgSnapshot::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("snapshot decodes"),
            left
        );
    }

    #[test]
    fn command_payload_round_trip_is_byte_exact() {
        let command = RpgCommand::AdvanceDialogueQuest {
            dialogue_id: PersistentId::from_bytes([1; 16]),
            expected_dialogue_node_id: schema("dialogue.node.offer"),
            next_dialogue_node_id: schema("dialogue.node.accepted"),
            quest_id: PersistentId::from_bytes([2; 16]),
            expected_quest_state_id: schema("quest.state.available"),
            next_quest_state_id: schema("quest.state.active"),
            relationship_source: PersistentId::from_bytes([3; 16]),
            relationship_target: PersistentId::from_bytes([4; 16]),
            relationship_dimension_id: schema("relationship.trust"),
            relationship_delta: 5,
        };
        let bytes = command
            .canonical_payload_bytes()
            .expect("command is canonical");
        assert_eq!(
            RpgCommand::from_canonical_payload_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("command decodes"),
            command
        );
    }

    #[test]
    fn proficiency_rejects_values_above_fixed_point_bound() {
        assert!(SkillProficiency::new(10_000).is_ok());
        assert!(SkillProficiency::new(10_001).is_err());
    }
}
