use std::error::Error;
use std::fmt::{Display, Formatter};

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
