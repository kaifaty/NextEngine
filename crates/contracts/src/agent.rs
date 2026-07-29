use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::sha256;
use crate::ids::{ContentHash, PersistentId, SchemaId, content_hash_from_bytes};

pub const AGENT_INTENT_SCHEMA_VERSION: u32 = 1;
pub const AGENT_MAX_AFFORDANCES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MotorCapabilityStateV1 {
    Unavailable = 0,
    ProceduralFallback = 1,
    Active = 2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ProceduralAvatarRouteV1 {
    Idle = 0,
    Locomotion = 1,
    Melee = 2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AvatarAnimationPhaseV1 {
    Idle = 0,
    Locomotion = 1,
    MeleeWindup = 2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ProceduralFallbackReasonV1 {
    AiHostUnavailable = 1,
    ModelUnavailable = 2,
    LearnedPolicyNotActivated = 3,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AgentAffordanceCandidateV1 {
    pub semantic_action_id: SchemaId,
    pub ability_definition_hash: ContentHash,
    pub target_character_id: PersistentId,
    pub utility_q16: i32,
    pub required_route: ProceduralAvatarRouteV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentPlannerSnapshotV1 {
    pub gameplay_tick: u64,
    pub world_generation: u64,
    pub rpg_state_hash: ContentHash,
    pub mechanics_lock_hash: ContentHash,
    pub decision_seed: u64,
    pub source_character_id: PersistentId,
    pub motor_state: MotorCapabilityStateV1,
    pub allowed_semantic_actions: Vec<SchemaId>,
    pub affordances: Vec<AgentAffordanceCandidateV1>,
    pub snapshot_hash: ContentHash,
}

impl AgentPlannerSnapshotV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the immutable planner snapshot binds every authoritative source explicitly"
    )]
    pub fn new(
        gameplay_tick: u64,
        world_generation: u64,
        rpg_state_hash: ContentHash,
        mechanics_lock_hash: ContentHash,
        decision_seed: u64,
        source_character_id: PersistentId,
        motor_state: MotorCapabilityStateV1,
        mut allowed_semantic_actions: Vec<SchemaId>,
        mut affordances: Vec<AgentAffordanceCandidateV1>,
    ) -> Result<Self, AgentContractError> {
        allowed_semantic_actions.sort();
        affordances.sort_by(agent_affordance_order);
        if allowed_semantic_actions.len() > AGENT_MAX_AFFORDANCES
            || affordances.len() > AGENT_MAX_AFFORDANCES
            || allowed_semantic_actions
                .windows(2)
                .any(|pair| pair[0] == pair[1])
            || affordances.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(AgentContractError::InvalidSnapshot);
        }
        let mut value = Self {
            gameplay_tick,
            world_generation,
            rpg_state_hash,
            mechanics_lock_hash,
            decision_seed,
            source_character_id,
            motor_state,
            allowed_semantic_actions,
            affordances,
            snapshot_hash: ContentHash::default(),
        };
        value.snapshot_hash = value.computed_hash()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), AgentContractError> {
        if self.allowed_semantic_actions.len() > AGENT_MAX_AFFORDANCES
            || self.affordances.len() > AGENT_MAX_AFFORDANCES
            || self
                .allowed_semantic_actions
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self
                .affordances
                .windows(2)
                .any(|pair| agent_affordance_order(&pair[0], &pair[1]).is_ge())
            || self.computed_hash()? != self.snapshot_hash
        {
            return Err(AgentContractError::InvalidSnapshot);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, AgentContractError> {
        let mut bytes = b"nextengine.agent-planner-snapshot.v1\0".to_vec();
        bytes.extend_from_slice(&AGENT_INTENT_SCHEMA_VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.gameplay_tick.to_le_bytes());
        bytes.extend_from_slice(&self.world_generation.to_le_bytes());
        bytes.extend_from_slice(self.rpg_state_hash.as_bytes());
        bytes.extend_from_slice(self.mechanics_lock_hash.as_bytes());
        bytes.extend_from_slice(&self.decision_seed.to_le_bytes());
        bytes.extend_from_slice(self.source_character_id.as_bytes());
        bytes.push(self.motor_state as u8);
        extend_count(&mut bytes, self.allowed_semantic_actions.len())?;
        for action in &self.allowed_semantic_actions {
            extend_text(&mut bytes, action.as_str())?;
        }
        extend_count(&mut bytes, self.affordances.len())?;
        for affordance in &self.affordances {
            extend_text(&mut bytes, affordance.semantic_action_id.as_str())?;
            bytes.extend_from_slice(affordance.ability_definition_hash.as_bytes());
            bytes.extend_from_slice(affordance.target_character_id.as_bytes());
            bytes.extend_from_slice(&affordance.utility_q16.to_le_bytes());
            bytes.push(affordance.required_route as u8);
        }
        Ok(bytes)
    }

    fn computed_hash(&self) -> Result<ContentHash, AgentContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentIntentV1 {
    pub schema_version: u32,
    pub intent_id: ContentHash,
    pub planner_snapshot_hash: ContentHash,
    pub source_character_id: PersistentId,
    pub target_character_id: PersistentId,
    pub semantic_action_id: SchemaId,
    pub ability_definition_hash: ContentHash,
    pub creation_tick: u64,
    pub expiry_tick: u64,
    pub decision_seed: u64,
}

impl AgentIntentV1 {
    pub fn new(
        snapshot: &AgentPlannerSnapshotV1,
        candidate: &AgentAffordanceCandidateV1,
    ) -> Result<Self, AgentContractError> {
        snapshot.validate()?;
        let expiry_tick = snapshot
            .gameplay_tick
            .checked_add(1)
            .ok_or(AgentContractError::LimitExceeded)?;
        let mut value = Self {
            schema_version: AGENT_INTENT_SCHEMA_VERSION,
            intent_id: ContentHash::default(),
            planner_snapshot_hash: snapshot.snapshot_hash,
            source_character_id: snapshot.source_character_id,
            target_character_id: candidate.target_character_id,
            semantic_action_id: candidate.semantic_action_id.clone(),
            ability_definition_hash: candidate.ability_definition_hash,
            creation_tick: snapshot.gameplay_tick,
            expiry_tick,
            decision_seed: snapshot.decision_seed,
        };
        value.intent_id = value.computed_id()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), AgentContractError> {
        if self.schema_version != AGENT_INTENT_SCHEMA_VERSION
            || self.source_character_id == self.target_character_id
            || self.creation_tick.checked_add(1) != Some(self.expiry_tick)
            || self.computed_id()? != self.intent_id
        {
            return Err(AgentContractError::InvalidIntent);
        }
        Ok(())
    }

    fn computed_id(&self) -> Result<ContentHash, AgentContractError> {
        let mut bytes = b"nextengine.agent-intent.v1\0".to_vec();
        bytes.extend_from_slice(&self.schema_version.to_le_bytes());
        bytes.extend_from_slice(self.planner_snapshot_hash.as_bytes());
        bytes.extend_from_slice(self.source_character_id.as_bytes());
        bytes.extend_from_slice(self.target_character_id.as_bytes());
        extend_text(&mut bytes, self.semantic_action_id.as_str())?;
        bytes.extend_from_slice(self.ability_definition_hash.as_bytes());
        bytes.extend_from_slice(&self.creation_tick.to_le_bytes());
        bytes.extend_from_slice(&self.expiry_tick.to_le_bytes());
        bytes.extend_from_slice(&self.decision_seed.to_le_bytes());
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProceduralAvatarProjectionV1 {
    pub subject_id: PersistentId,
    pub simulation_tick: u64,
    pub route: ProceduralAvatarRouteV1,
    pub animation_phase: AvatarAnimationPhaseV1,
    pub source_intent_id: Option<ContentHash>,
    pub fallback_reason: ProceduralFallbackReasonV1,
    pub projection_hash: ContentHash,
}

impl ProceduralAvatarProjectionV1 {
    pub fn from_intent(
        intent: &AgentIntentV1,
        route: ProceduralAvatarRouteV1,
        fallback_reason: ProceduralFallbackReasonV1,
    ) -> Result<Self, AgentContractError> {
        intent.validate()?;
        let animation_phase = match route {
            ProceduralAvatarRouteV1::Idle => AvatarAnimationPhaseV1::Idle,
            ProceduralAvatarRouteV1::Locomotion => AvatarAnimationPhaseV1::Locomotion,
            ProceduralAvatarRouteV1::Melee => AvatarAnimationPhaseV1::MeleeWindup,
        };
        let mut value = Self {
            subject_id: intent.source_character_id,
            simulation_tick: intent.expiry_tick,
            route,
            animation_phase,
            source_intent_id: Some(intent.intent_id),
            fallback_reason,
            projection_hash: ContentHash::default(),
        };
        value.projection_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), AgentContractError> {
        if self.computed_hash() != self.projection_hash {
            return Err(AgentContractError::InvalidProjection);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        let mut bytes = b"nextengine.procedural-avatar-projection.v1\0".to_vec();
        bytes.extend_from_slice(self.subject_id.as_bytes());
        bytes.extend_from_slice(&self.simulation_tick.to_le_bytes());
        bytes.push(self.route as u8);
        bytes.push(self.animation_phase as u8);
        match self.source_intent_id {
            None => bytes.push(0),
            Some(intent_id) => {
                bytes.push(1);
                bytes.extend_from_slice(intent_id.as_bytes());
            }
        }
        bytes.push(self.fallback_reason as u8);
        content_hash_from_bytes(sha256(&bytes))
    }
}

fn agent_affordance_order(
    left: &AgentAffordanceCandidateV1,
    right: &AgentAffordanceCandidateV1,
) -> std::cmp::Ordering {
    right
        .utility_q16
        .cmp(&left.utility_q16)
        .then_with(|| left.semantic_action_id.cmp(&right.semantic_action_id))
        .then_with(|| {
            left.ability_definition_hash
                .cmp(&right.ability_definition_hash)
        })
        .then_with(|| left.target_character_id.cmp(&right.target_character_id))
        .then_with(|| left.required_route.cmp(&right.required_route))
}

fn extend_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), AgentContractError> {
    bytes.extend_from_slice(
        &u32::try_from(count)
            .map_err(|_| AgentContractError::LimitExceeded)?
            .to_le_bytes(),
    );
    Ok(())
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), AgentContractError> {
    extend_count(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AgentContractError {
    InvalidSnapshot,
    InvalidIntent,
    InvalidProjection,
    LimitExceeded,
}

impl Display for AgentContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSnapshot => "agent planner snapshot is invalid",
            Self::InvalidIntent => "agent intent is invalid",
            Self::InvalidProjection => "procedural avatar projection is invalid",
            Self::LimitExceeded => "agent contract limit exceeded",
        })
    }
}

impl Error for AgentContractError {}
