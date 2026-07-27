#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CORE_DIALOGUE_ACCEPTED_NODE_ID, CORE_DIALOGUE_OFFER_NODE_ID, CORE_DIALOGUE_QUEST_TRUST_DELTA,
    CORE_HELP_DIALOGUE_DEFINITION_ID, CORE_HELP_QUEST_DEFINITION_ID, CORE_QUEST_ACTIVE_STATE_ID,
    CORE_QUEST_AVAILABLE_STATE_ID, CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID,
    CORE_RELATIONSHIP_TRUST_DIMENSION_ID, CharacterSnapshot, DialogueSnapshot, FactionSnapshot,
    InteractiveObjectSnapshot, ItemSnapshot, PersistentId, QuestSnapshot, RelationshipEntry,
    RpgCommand, RpgEvent, RpgSnapshot, SKILL_PROFICIENCY_MAX, SkillProficiency,
    SkillProficiencyEntry, WorldChunkRecordSnapshot,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RpgState {
    characters: BTreeMap<PersistentId, CharacterSnapshot>,
    items: BTreeMap<PersistentId, ItemSnapshot>,
    quests: BTreeMap<PersistentId, QuestSnapshot>,
    dialogues: BTreeMap<PersistentId, DialogueSnapshot>,
    factions: BTreeMap<PersistentId, FactionSnapshot>,
    interactive_objects: BTreeMap<PersistentId, InteractiveObjectSnapshot>,
    world_chunk_records: BTreeMap<PersistentId, WorldChunkRecordSnapshot>,
}

impl RpgState {
    pub fn from_snapshot(snapshot: RpgSnapshot) -> Result<Self, RpgStateError> {
        let state = Self {
            characters: collect_unique(snapshot.characters, RpgAggregateKind::Character)?,
            items: collect_unique(snapshot.items, RpgAggregateKind::Item)?,
            quests: collect_unique(snapshot.quests, RpgAggregateKind::Quest)?,
            dialogues: collect_unique(snapshot.dialogues, RpgAggregateKind::Dialogue)?,
            factions: collect_unique(snapshot.factions, RpgAggregateKind::Faction)?,
            interactive_objects: collect_unique(
                snapshot.interactive_objects,
                RpgAggregateKind::InteractiveObject,
            )?,
            world_chunk_records: collect_unique(
                snapshot.world_chunk_records,
                RpgAggregateKind::WorldChunkRecord,
            )?,
        };
        state.validate_cross_references()?;
        Ok(state)
    }

    #[must_use]
    pub fn snapshot(&self) -> RpgSnapshot {
        RpgSnapshot {
            characters: self.characters.values().cloned().collect(),
            items: self.items.values().cloned().collect(),
            quests: self.quests.values().cloned().collect(),
            dialogues: self.dialogues.values().cloned().collect(),
            factions: self.factions.values().cloned().collect(),
            interactive_objects: self.interactive_objects.values().cloned().collect(),
            world_chunk_records: self.world_chunk_records.values().cloned().collect(),
        }
    }

    #[must_use]
    pub fn character(&self, id: PersistentId) -> Option<&CharacterSnapshot> {
        self.characters.get(&id)
    }

    #[must_use]
    pub fn item(&self, id: PersistentId) -> Option<&ItemSnapshot> {
        self.items.get(&id)
    }

    #[must_use]
    pub fn quest(&self, id: PersistentId) -> Option<&QuestSnapshot> {
        self.quests.get(&id)
    }

    #[must_use]
    pub fn dialogue(&self, id: PersistentId) -> Option<&DialogueSnapshot> {
        self.dialogues.get(&id)
    }

    pub fn apply(&mut self, command: &RpgCommand) -> Result<RpgEvent, RpgApplyError> {
        let mut staged = self.clone();
        let event = staged.apply_inner(command)?;
        *self = staged;
        Ok(event)
    }

    fn apply_inner(&mut self, command: &RpgCommand) -> Result<RpgEvent, RpgApplyError> {
        match command {
            RpgCommand::AdvanceDialogueQuest {
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
                let dialogue = self
                    .dialogues
                    .get(dialogue_id)
                    .ok_or(RpgApplyError::AggregateNotFound(RpgAggregateKind::Dialogue))?;
                if relationship_source != &dialogue.speaker
                    || relationship_target != &dialogue.listener
                {
                    return Err(RpgApplyError::InvariantViolation(
                        "RPG_DIALOGUE_PARTICIPANT_MISMATCH",
                    ));
                }
                if &dialogue.node_id != expected_dialogue_node_id {
                    return Err(RpgApplyError::StatePreconditionFailed(
                        RpgAggregateKind::Dialogue,
                    ));
                }
                let quest = self
                    .quests
                    .get(quest_id)
                    .ok_or(RpgApplyError::AggregateNotFound(RpgAggregateKind::Quest))?;
                if &quest.state_id != expected_quest_state_id {
                    return Err(RpgApplyError::StatePreconditionFailed(
                        RpgAggregateKind::Quest,
                    ));
                }
                let source = self.characters.get(relationship_source).ok_or(
                    RpgApplyError::AggregateNotFound(RpgAggregateKind::Character),
                )?;
                if !self.characters.contains_key(relationship_target) {
                    return Err(RpgApplyError::AggregateNotFound(
                        RpgAggregateKind::Character,
                    ));
                }
                let is_core_dialogue =
                    dialogue.definition_id.as_str() == CORE_HELP_DIALOGUE_DEFINITION_ID;
                let is_core_quest = quest.definition_id.as_str() == CORE_HELP_QUEST_DEFINITION_ID;
                if (is_core_dialogue || is_core_quest)
                    && (!is_core_dialogue
                        || !is_core_quest
                        || source.archetype_id.as_str() != CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID
                        || expected_dialogue_node_id.as_str() != CORE_DIALOGUE_OFFER_NODE_ID
                        || next_dialogue_node_id.as_str() != CORE_DIALOGUE_ACCEPTED_NODE_ID
                        || expected_quest_state_id.as_str() != CORE_QUEST_AVAILABLE_STATE_ID
                        || next_quest_state_id.as_str() != CORE_QUEST_ACTIVE_STATE_ID
                        || relationship_dimension_id.as_str()
                            != CORE_RELATIONSHIP_TRUST_DIMENSION_ID
                        || *relationship_delta != CORE_DIALOGUE_QUEST_TRUST_DELTA)
                {
                    return Err(RpgApplyError::InvariantViolation(
                        "RPG_CORE_INTERACTION_COMMAND_INVALID",
                    ));
                }
                let current_relationship = source
                    .relationships
                    .iter()
                    .find(|entry| {
                        entry.target == *relationship_target
                            && entry.dimension_id == *relationship_dimension_id
                    })
                    .map_or(0, |entry| entry.value);
                let relationship_value = current_relationship
                    .checked_add(*relationship_delta)
                    .ok_or(RpgApplyError::RelationshipOverflow)?;

                let dialogue = self
                    .dialogues
                    .get_mut(dialogue_id)
                    .expect("validated dialogue remains present");
                dialogue.node_id = next_dialogue_node_id.clone();
                dialogue.revision = increment_revision(dialogue.revision)?;

                let quest = self
                    .quests
                    .get_mut(quest_id)
                    .expect("validated quest remains present");
                quest.state_id = next_quest_state_id.clone();
                quest.revision = increment_revision(quest.revision)?;

                let source = self
                    .characters
                    .get_mut(relationship_source)
                    .expect("validated character remains present");
                match source.relationships.iter_mut().find(|entry| {
                    entry.target == *relationship_target
                        && entry.dimension_id == *relationship_dimension_id
                }) {
                    Some(entry) => entry.value = relationship_value,
                    None => source.relationships.push(RelationshipEntry {
                        target: *relationship_target,
                        dimension_id: relationship_dimension_id.clone(),
                        value: relationship_value,
                    }),
                }
                source.relationships.sort();
                source.revision = increment_revision(source.revision)?;

                Ok(RpgEvent::DialogueQuestAdvanced {
                    dialogue_id: *dialogue_id,
                    dialogue_node_id: next_dialogue_node_id.clone(),
                    quest_id: *quest_id,
                    quest_state_id: next_quest_state_id.clone(),
                    relationship_source: *relationship_source,
                    relationship_target: *relationship_target,
                    relationship_dimension_id: relationship_dimension_id.clone(),
                    relationship_value,
                })
            }
            RpgCommand::TransferItem {
                item_id,
                expected_owner,
                new_owner,
            } => {
                if let Some(owner) = new_owner {
                    ensure_character_exists(&self.characters, *owner)?;
                }
                let item = self
                    .items
                    .get(item_id)
                    .ok_or(RpgApplyError::AggregateNotFound(RpgAggregateKind::Item))?;
                if item.owner != *expected_owner {
                    return Err(RpgApplyError::StatePreconditionFailed(
                        RpgAggregateKind::Item,
                    ));
                }
                if item.quantity == 0 {
                    return Err(RpgApplyError::InvariantViolation("RPG_ITEM_QUANTITY_ZERO"));
                }
                let previous_owner = item.owner;
                let item = self
                    .items
                    .get_mut(item_id)
                    .expect("validated item remains present");
                item.owner = *new_owner;
                item.revision = increment_revision(item.revision)?;
                Ok(RpgEvent::ItemTransferred {
                    item_id: *item_id,
                    previous_owner,
                    new_owner: *new_owner,
                })
            }
            RpgCommand::LearnSkill {
                character_id,
                skill_id,
                delta,
            } => {
                let character =
                    self.characters
                        .get(character_id)
                        .ok_or(RpgApplyError::AggregateNotFound(
                            RpgAggregateKind::Character,
                        ))?;
                let current = character
                    .skills
                    .iter()
                    .find(|entry| entry.skill_id == *skill_id)
                    .map_or(0, |entry| entry.proficiency.get());
                let next = current
                    .checked_add(*delta)
                    .filter(|value| *value <= SKILL_PROFICIENCY_MAX)
                    .ok_or(RpgApplyError::SkillProficiencyOutOfRange)?;
                let proficiency = SkillProficiency::new(next)
                    .map_err(|_| RpgApplyError::SkillProficiencyOutOfRange)?;
                let character = self
                    .characters
                    .get_mut(character_id)
                    .expect("validated character remains present");
                match character
                    .skills
                    .iter_mut()
                    .find(|entry| entry.skill_id == *skill_id)
                {
                    Some(entry) => entry.proficiency = proficiency,
                    None => character.skills.push(SkillProficiencyEntry {
                        skill_id: skill_id.clone(),
                        proficiency,
                    }),
                }
                character.skills.sort();
                character.revision = increment_revision(character.revision)?;
                Ok(RpgEvent::SkillLearned {
                    character_id: *character_id,
                    skill_id: skill_id.clone(),
                    proficiency,
                })
            }
            RpgCommand::SetInteractiveObjectState {
                object_id,
                expected_state_id,
                next_state_id,
            } => {
                let object = self.interactive_objects.get(object_id).ok_or(
                    RpgApplyError::AggregateNotFound(RpgAggregateKind::InteractiveObject),
                )?;
                if object.state_id != *expected_state_id {
                    return Err(RpgApplyError::StatePreconditionFailed(
                        RpgAggregateKind::InteractiveObject,
                    ));
                }
                let object = self
                    .interactive_objects
                    .get_mut(object_id)
                    .expect("validated object remains present");
                object.state_id = next_state_id.clone();
                object.revision = increment_revision(object.revision)?;
                Ok(RpgEvent::InteractiveObjectStateChanged {
                    object_id: *object_id,
                    state_id: next_state_id.clone(),
                })
            }
        }
    }

    fn validate_cross_references(&self) -> Result<(), RpgStateError> {
        for character in self.characters.values() {
            ensure_strictly_sorted_unique(
                &character.skills,
                |entry| entry.skill_id.clone(),
                RpgStateError::DuplicateSkill,
            )?;
            ensure_strictly_sorted_unique(
                &character.relationships,
                |entry| (entry.target, entry.dimension_id.clone()),
                RpgStateError::DuplicateRelationship,
            )?;
            for relationship in &character.relationships {
                if !self.characters.contains_key(&relationship.target) {
                    return Err(RpgStateError::DanglingCharacterReference);
                }
            }
        }
        for item in self.items.values() {
            if item.quantity == 0 {
                return Err(RpgStateError::ZeroItemQuantity);
            }
            if let Some(owner) = item.owner {
                ensure_character_exists_for_snapshot(&self.characters, owner)?;
            }
        }
        for dialogue in self.dialogues.values() {
            ensure_character_exists_for_snapshot(&self.characters, dialogue.speaker)?;
            ensure_character_exists_for_snapshot(&self.characters, dialogue.listener)?;
        }
        for faction in self.factions.values() {
            ensure_strictly_sorted_unique(
                &faction.members,
                |member| *member,
                RpgStateError::DuplicateFactionMember,
            )?;
            for member in &faction.members {
                ensure_character_exists_for_snapshot(&self.characters, *member)?;
            }
        }
        Ok(())
    }
}

trait AggregateSnapshot {
    fn id(&self) -> PersistentId;
}

macro_rules! impl_aggregate_snapshot {
    ($($type:ty),+ $(,)?) => {
        $(
            impl AggregateSnapshot for $type {
                fn id(&self) -> PersistentId {
                    self.id
                }
            }
        )+
    };
}

impl_aggregate_snapshot!(
    CharacterSnapshot,
    ItemSnapshot,
    QuestSnapshot,
    DialogueSnapshot,
    FactionSnapshot,
    InteractiveObjectSnapshot,
    WorldChunkRecordSnapshot,
);

fn collect_unique<T: AggregateSnapshot>(
    records: Vec<T>,
    kind: RpgAggregateKind,
) -> Result<BTreeMap<PersistentId, T>, RpgStateError> {
    let mut result = BTreeMap::new();
    for record in records {
        if result.insert(record.id(), record).is_some() {
            return Err(RpgStateError::DuplicateAggregate(kind));
        }
    }
    Ok(result)
}

fn ensure_strictly_sorted_unique<T, K: Ord>(
    values: &[T],
    key: impl Fn(&T) -> K,
    error: RpgStateError,
) -> Result<(), RpgStateError> {
    if values.windows(2).any(|pair| key(&pair[0]) >= key(&pair[1])) {
        return Err(error);
    }
    Ok(())
}

fn ensure_character_exists(
    characters: &BTreeMap<PersistentId, CharacterSnapshot>,
    id: PersistentId,
) -> Result<(), RpgApplyError> {
    if characters.contains_key(&id) {
        Ok(())
    } else {
        Err(RpgApplyError::AggregateNotFound(
            RpgAggregateKind::Character,
        ))
    }
}

fn ensure_character_exists_for_snapshot(
    characters: &BTreeMap<PersistentId, CharacterSnapshot>,
    id: PersistentId,
) -> Result<(), RpgStateError> {
    if characters.contains_key(&id) {
        Ok(())
    } else {
        Err(RpgStateError::DanglingCharacterReference)
    }
}

fn increment_revision(revision: u64) -> Result<u64, RpgApplyError> {
    revision
        .checked_add(1)
        .ok_or(RpgApplyError::AggregateRevisionExhausted)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RpgAggregateKind {
    Character,
    Item,
    Quest,
    Dialogue,
    Faction,
    InteractiveObject,
    WorldChunkRecord,
}

impl RpgAggregateKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Character => "character",
            Self::Item => "item",
            Self::Quest => "quest",
            Self::Dialogue => "dialogue",
            Self::Faction => "faction",
            Self::InteractiveObject => "interactive-object",
            Self::WorldChunkRecord => "world-chunk-record",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgStateError {
    DuplicateAggregate(RpgAggregateKind),
    DuplicateSkill,
    DuplicateRelationship,
    DuplicateFactionMember,
    DanglingCharacterReference,
    ZeroItemQuantity,
}

impl RpgStateError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::DuplicateAggregate(_) => "RPG_DUPLICATE_AGGREGATE",
            Self::DuplicateSkill => "RPG_DUPLICATE_SKILL",
            Self::DuplicateRelationship => "RPG_DUPLICATE_RELATIONSHIP",
            Self::DuplicateFactionMember => "RPG_DUPLICATE_FACTION_MEMBER",
            Self::DanglingCharacterReference => "RPG_DANGLING_CHARACTER_REFERENCE",
            Self::ZeroItemQuantity => "RPG_ITEM_QUANTITY_ZERO",
        }
    }
}

impl Display for RpgStateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateAggregate(kind) => {
                write!(formatter, "{}: {}", self.stable_code(), kind.as_str())
            }
            _ => formatter.write_str(self.stable_code()),
        }
    }
}

impl Error for RpgStateError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgApplyError {
    AggregateNotFound(RpgAggregateKind),
    StatePreconditionFailed(RpgAggregateKind),
    InvariantViolation(&'static str),
    RelationshipOverflow,
    SkillProficiencyOutOfRange,
    AggregateRevisionExhausted,
}

impl RpgApplyError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::AggregateNotFound(_) => "RPG_AGGREGATE_NOT_FOUND",
            Self::StatePreconditionFailed(_) => "RPG_STATE_PRECONDITION_FAILED",
            Self::InvariantViolation(code) => code,
            Self::RelationshipOverflow => "RPG_RELATIONSHIP_OVERFLOW",
            Self::SkillProficiencyOutOfRange => "RPG_SKILL_PROFICIENCY_OUT_OF_RANGE",
            Self::AggregateRevisionExhausted => "RPG_AGGREGATE_REVISION_EXHAUSTED",
        }
    }
}

impl Display for RpgApplyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AggregateNotFound(kind) | Self::StatePreconditionFailed(kind) => {
                write!(formatter, "{}: {}", self.stable_code(), kind.as_str())
            }
            _ => formatter.write_str(self.stable_code()),
        }
    }
}

impl Error for RpgApplyError {}

#[cfg(test)]
mod tests {
    use next_contracts::{
        CORE_DIALOGUE_ACCEPTED_NODE_ID, CORE_DIALOGUE_OFFER_NODE_ID,
        CORE_DIALOGUE_QUEST_TRUST_DELTA, CORE_HELP_DIALOGUE_DEFINITION_ID,
        CORE_HELP_QUEST_DEFINITION_ID, CORE_QUEST_ACTIVE_STATE_ID, CORE_QUEST_AVAILABLE_STATE_ID,
        CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID, CORE_RELATIONSHIP_TRUST_DIMENSION_ID,
        CharacterSnapshot, DialogueSnapshot, ItemSnapshot, PersistentId, QuestSnapshot,
        RelationshipEntry, RpgCommand, RpgSnapshot, SchemaId, SkillProficiency,
        SkillProficiencyEntry,
    };

    use super::{RpgApplyError, RpgState};

    fn id(value: u8) -> PersistentId {
        PersistentId::from_bytes([value; 16])
    }

    fn schema(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test schema is valid")
    }

    fn fixture() -> RpgSnapshot {
        RpgSnapshot {
            characters: vec![
                CharacterSnapshot {
                    id: id(1),
                    revision: 0,
                    archetype_id: schema("rpg.character.generic-npc"),
                    skills: vec![SkillProficiencyEntry {
                        skill_id: schema("rpg.skill.survival"),
                        proficiency: SkillProficiency::new(9_900).expect("bounded"),
                    }],
                    relationships: vec![RelationshipEntry {
                        target: id(2),
                        dimension_id: schema("rpg.relationship.trust"),
                        value: 0,
                    }],
                },
                CharacterSnapshot {
                    id: id(2),
                    revision: 0,
                    archetype_id: schema("rpg.character.player"),
                    skills: vec![],
                    relationships: vec![],
                },
            ],
            items: vec![ItemSnapshot {
                id: id(3),
                revision: 0,
                archetype_id: schema("rpg.item.quest-token"),
                owner: Some(id(1)),
                quantity: 1,
            }],
            quests: vec![QuestSnapshot {
                id: id(4),
                revision: 0,
                definition_id: schema("rpg.quest.generic-help"),
                state_id: schema("rpg.quest-state.available"),
            }],
            dialogues: vec![DialogueSnapshot {
                id: id(5),
                revision: 0,
                definition_id: schema("rpg.dialogue.generic-help"),
                speaker: id(1),
                listener: id(2),
                node_id: schema("rpg.dialogue-node.offer"),
            }],
            ..RpgSnapshot::default()
        }
    }

    #[test]
    fn dialogue_quest_relationship_transition_is_atomic() {
        let mut state = RpgState::from_snapshot(fixture()).expect("fixture is valid");
        let command = RpgCommand::AdvanceDialogueQuest {
            dialogue_id: id(5),
            expected_dialogue_node_id: schema("rpg.dialogue-node.offer"),
            next_dialogue_node_id: schema("rpg.dialogue-node.accepted"),
            quest_id: id(4),
            expected_quest_state_id: schema("rpg.quest-state.available"),
            next_quest_state_id: schema("rpg.quest-state.active"),
            relationship_source: id(1),
            relationship_target: id(2),
            relationship_dimension_id: schema("rpg.relationship.trust"),
            relationship_delta: 7,
        };

        state.apply(&command).expect("transition commits");
        assert_eq!(
            state.dialogue(id(5)).expect("dialogue").node_id,
            schema("rpg.dialogue-node.accepted")
        );
        assert_eq!(
            state.quest(id(4)).expect("quest").state_id,
            schema("rpg.quest-state.active")
        );
        assert_eq!(
            state.character(id(1)).expect("character").relationships[0].value,
            7
        );
    }

    #[test]
    fn failed_multi_aggregate_precondition_rolls_back_every_owner_record() {
        let mut state = RpgState::from_snapshot(fixture()).expect("fixture is valid");
        let before = state.snapshot();
        let command = RpgCommand::AdvanceDialogueQuest {
            dialogue_id: id(5),
            expected_dialogue_node_id: schema("rpg.dialogue-node.offer"),
            next_dialogue_node_id: schema("rpg.dialogue-node.accepted"),
            quest_id: id(4),
            expected_quest_state_id: schema("rpg.quest-state.wrong"),
            next_quest_state_id: schema("rpg.quest-state.active"),
            relationship_source: id(1),
            relationship_target: id(2),
            relationship_dimension_id: schema("rpg.relationship.trust"),
            relationship_delta: 7,
        };

        assert_eq!(
            state.apply(&command),
            Err(RpgApplyError::StatePreconditionFailed(
                super::RpgAggregateKind::Quest
            ))
        );
        assert_eq!(state.snapshot(), before);
    }

    #[test]
    fn skill_overflow_is_rejected_without_mutation() {
        let mut state = RpgState::from_snapshot(fixture()).expect("fixture is valid");
        let before = state.snapshot();
        let command = RpgCommand::LearnSkill {
            character_id: id(1),
            skill_id: schema("rpg.skill.survival"),
            delta: 101,
        };

        assert_eq!(
            state.apply(&command),
            Err(RpgApplyError::SkillProficiencyOutOfRange)
        );
        assert_eq!(state.snapshot(), before);
    }

    #[test]
    fn dialogue_relationship_participants_must_match_speaker_and_listener() {
        let mut state = RpgState::from_snapshot(fixture()).expect("fixture is valid");
        let before = state.snapshot();
        let command = RpgCommand::AdvanceDialogueQuest {
            dialogue_id: id(5),
            expected_dialogue_node_id: schema("rpg.dialogue-node.offer"),
            next_dialogue_node_id: schema("rpg.dialogue-node.accepted"),
            quest_id: id(4),
            expected_quest_state_id: schema("rpg.quest-state.available"),
            next_quest_state_id: schema("rpg.quest-state.active"),
            relationship_source: id(2),
            relationship_target: id(1),
            relationship_dimension_id: schema("rpg.relationship.trust"),
            relationship_delta: 7,
        };

        assert_eq!(
            state.apply(&command),
            Err(RpgApplyError::InvariantViolation(
                "RPG_DIALOGUE_PARTICIPANT_MISMATCH"
            ))
        );
        assert_eq!(state.snapshot(), before);
    }

    #[test]
    fn core_dialogue_transition_accepts_only_the_bounded_profile() {
        let mut snapshot = fixture();
        snapshot.characters[0].archetype_id = schema(CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID);
        snapshot.dialogues[0].definition_id = schema(CORE_HELP_DIALOGUE_DEFINITION_ID);
        snapshot.dialogues[0].node_id = schema(CORE_DIALOGUE_OFFER_NODE_ID);
        snapshot.quests[0].definition_id = schema(CORE_HELP_QUEST_DEFINITION_ID);
        snapshot.quests[0].state_id = schema(CORE_QUEST_AVAILABLE_STATE_ID);
        let mut state = RpgState::from_snapshot(snapshot).expect("core fixture is valid");
        let before = state.snapshot();
        let invalid = RpgCommand::AdvanceDialogueQuest {
            dialogue_id: id(5),
            expected_dialogue_node_id: schema(CORE_DIALOGUE_OFFER_NODE_ID),
            next_dialogue_node_id: schema(CORE_DIALOGUE_ACCEPTED_NODE_ID),
            quest_id: id(4),
            expected_quest_state_id: schema(CORE_QUEST_AVAILABLE_STATE_ID),
            next_quest_state_id: schema(CORE_QUEST_ACTIVE_STATE_ID),
            relationship_source: id(1),
            relationship_target: id(2),
            relationship_dimension_id: schema(CORE_RELATIONSHIP_TRUST_DIMENSION_ID),
            relationship_delta: CORE_DIALOGUE_QUEST_TRUST_DELTA + 1,
        };
        assert_eq!(
            state.apply(&invalid),
            Err(RpgApplyError::InvariantViolation(
                "RPG_CORE_INTERACTION_COMMAND_INVALID"
            ))
        );
        assert_eq!(state.snapshot(), before);

        let valid = RpgCommand::AdvanceDialogueQuest {
            dialogue_id: id(5),
            expected_dialogue_node_id: schema(CORE_DIALOGUE_OFFER_NODE_ID),
            next_dialogue_node_id: schema(CORE_DIALOGUE_ACCEPTED_NODE_ID),
            quest_id: id(4),
            expected_quest_state_id: schema(CORE_QUEST_AVAILABLE_STATE_ID),
            next_quest_state_id: schema(CORE_QUEST_ACTIVE_STATE_ID),
            relationship_source: id(1),
            relationship_target: id(2),
            relationship_dimension_id: schema(CORE_RELATIONSHIP_TRUST_DIMENSION_ID),
            relationship_delta: CORE_DIALOGUE_QUEST_TRUST_DELTA,
        };
        state.apply(&valid).expect("bounded transition commits");
        assert_eq!(
            state.dialogue(id(5)).expect("dialogue").node_id,
            schema(CORE_DIALOGUE_ACCEPTED_NODE_ID)
        );
        assert_eq!(
            state.quest(id(4)).expect("quest").state_id,
            schema(CORE_QUEST_ACTIVE_STATE_ID)
        );
    }
}
