use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_ID128, CANONICAL_TYPE_U16, CanonicalCursor,
    CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::ids::{
    AssetId, ContentHash, IdentifierError, PersistentId, SchemaId, content_hash_from_bytes,
};

pub const COGNITION_SCHEMA_VERSION: u16 = 1;
pub const AGENT_COGNITION_COMMAND_SCHEMA_VERSION: u32 = 1;
pub const AGENT_COGNITION_EVENT_SCHEMA_VERSION: u32 = 1;

pub const AGENT_COGNITION_CATALOG_OWNER_ID: &str = "nextengine.assets";
pub const AGENT_COGNITION_CATALOG_SCHEMA_ID: &str = "nextengine.content.agent-cognition-catalog";
pub const AGENT_COGNITION_CATALOG_SEGMENT_ID: &str = "nextengine.agent-cognition-catalog.v1";

pub const AGENT_MEMORY_SNAPSHOT_OWNER_ID: &str = "nextengine.memory-service";
pub const AGENT_MEMORY_SNAPSHOT_SCHEMA_ID: &str = "nextengine.agent-memory-snapshot";
pub const AGENT_MEMORY_SNAPSHOT_SEGMENT_ID: &str = "agent-memory";
pub const AGENT_RUNTIME_SNAPSHOT_OWNER_ID: &str = "nextengine.agent-runtime";
pub const AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID: &str = "nextengine.agent-cognition-snapshot";
pub const AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID: &str = "agent-cognition";

pub const AGENT_COGNITION_COMMAND_SCHEMA_ID: &str = "nextengine.command.agent-cognition";
pub const AGENT_COGNITION_COMMAND_KIND_ID: &str = "nextengine.command-kind.agent-cognition";
pub const AGENT_COGNITION_EVENT_SCHEMA_ID: &str = "nextengine.event.agent-decision-committed";
pub const AGENT_COGNITION_CAPABILITY_ID: &str = "nextengine.capability.agent-cognition-commit";
pub const AGENT_COGNITION_SYSTEM_ID: &str = "nextengine.system.agent-cognition-boundary";
pub const AGENT_COGNITION_CAPABILITY_SUBJECT_ID: &str =
    "nextengine.capability-subject.agent-cognition-boundary";
pub const AGENT_COGNITION_SHARD_PLAN_ID: &str = "nextengine.shard-plan.agent-cognition-single";
pub const AGENT_COGNITION_PRIORITY_CLASS: u16 = 270;

pub const COGNITION_MAX_BELIEFS: usize = 32;
pub const COGNITION_MAX_RETRIEVED_BELIEFS: usize = 8;
pub const COGNITION_MAX_FACTS: usize = 16;
pub const COGNITION_MAX_AFFORDANCES: usize = 8;
pub const COGNITION_MAX_GOALS: usize = 8;
pub const COGNITION_MAX_PLAN_STEPS: usize = 8;
pub const COGNITION_MAX_SUSPENDED_GOALS: usize = 4;
pub const COGNITION_MAX_TEXT_BYTES: usize = 4 * 1024;
pub const COGNITION_Q16_ONE: i32 = 1 << 16;
const COGNITION_MAX_SCORE_Q16: i32 = 256 << 16;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BeliefSourceV1 {
    AuthoredSeed = 1,
    OwnerProjection = 2,
}

impl BeliefSourceV1 {
    fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
        match tag {
            1 => Ok(Self::AuthoredSeed),
            2 => Ok(Self::OwnerProjection),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BeliefContradictionV1 {
    Consistent = 1,
    Contradicted = 2,
}

impl BeliefContradictionV1 {
    fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
        match tag {
            1 => Ok(Self::Consistent),
            2 => Ok(Self::Contradicted),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SemanticBeliefV1 {
    pub belief_id: ContentHash,
    pub subject_id: PersistentId,
    pub predicate_id: SchemaId,
    pub value_id: SchemaId,
    pub confidence_q16: u32,
    pub source: BeliefSourceV1,
    pub learned_tick: u64,
    pub last_verified_revision: u64,
    pub contradiction: BeliefContradictionV1,
}

impl SemanticBeliefV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "belief provenance is an exact public contract"
    )]
    pub fn new(
        subject_id: PersistentId,
        predicate_id: SchemaId,
        value_id: SchemaId,
        confidence_q16: u32,
        source: BeliefSourceV1,
        learned_tick: u64,
        last_verified_revision: u64,
        contradiction: BeliefContradictionV1,
    ) -> Result<Self, CognitionContractError> {
        let mut value = Self {
            belief_id: ContentHash::default(),
            subject_id,
            predicate_id,
            value_id,
            confidence_q16,
            source,
            learned_tick,
            last_verified_revision,
            contradiction,
        };
        value.belief_id = value.computed_id()?;
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.confidence_q16 > u32::try_from(COGNITION_Q16_ONE).expect("q16 one is positive")
            || self.computed_id()? != self.belief_id
        {
            return Err(CognitionContractError::BeliefInvalid);
        }
        Ok(())
    }

    fn computed_id(&self) -> Result<ContentHash, CognitionContractError> {
        let mut writer = Writer::with_domain(b"nextengine.semantic-belief.v1\0");
        write_belief_identity(&mut writer, self)?;
        Ok(writer.finish_hash())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCognitionCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: AssetId,
    pub subject_id: PersistentId,
    pub evaluation_start_tick: u64,
    pub evaluation_period_ticks: u64,
    pub retrieval_limit: u16,
    pub goal_switch_threshold_q16: i32,
    pub emergency_health_threshold: i32,
    pub planner_max_depth: u8,
    pub planner_max_expanded_nodes: u16,
    pub ordinary_goal_id: SchemaId,
    pub emergency_goal_id: SchemaId,
    pub navigate_action_id: SchemaId,
    pub hold_action_id: SchemaId,
    pub route_known_fact_id: SchemaId,
    pub travel_needed_fact_id: SchemaId,
    pub emergency_fact_id: SchemaId,
    pub navigate_ready_fact_id: SchemaId,
    pub hold_ready_fact_id: SchemaId,
    pub seed_beliefs: Vec<SemanticBeliefV1>,
}

impl AgentCognitionCatalogV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || self.evaluation_start_tick == 0
            || self.evaluation_period_ticks == 0
            || self.retrieval_limit == 0
            || usize::from(self.retrieval_limit) > COGNITION_MAX_RETRIEVED_BELIEFS
            || self.goal_switch_threshold_q16 < 0
            || self.goal_switch_threshold_q16 > COGNITION_MAX_SCORE_Q16
            || self.emergency_health_threshold <= 0
            || self.planner_max_depth == 0
            || usize::from(self.planner_max_depth) > COGNITION_MAX_PLAN_STEPS
            || self.planner_max_expanded_nodes == 0
            || usize::from(self.planner_max_expanded_nodes) > 256
            || self.ordinary_goal_id == self.emergency_goal_id
            || self.navigate_action_id == self.hold_action_id
            || self.seed_beliefs.is_empty()
            || self.seed_beliefs.len() > COGNITION_MAX_BELIEFS
            || self.seed_beliefs.windows(2).any(|pair| pair[0] >= pair[1])
            || self
                .seed_beliefs
                .iter()
                .any(|belief| belief.subject_id != self.subject_id || belief.validate().is_err())
        {
            return Err(CognitionContractError::ContentInvalid);
        }
        let facts = [
            &self.route_known_fact_id,
            &self.travel_needed_fact_id,
            &self.emergency_fact_id,
            &self.navigate_ready_fact_id,
            &self.hold_ready_fact_id,
        ];
        if facts
            .iter()
            .enumerate()
            .any(|(index, value)| facts[index + 1..].contains(value))
        {
            return Err(CognitionContractError::ContentInvalid);
        }
        Ok(())
    }

    #[must_use]
    pub fn is_due(&self, tick: u64) -> bool {
        tick >= self.evaluation_start_tick
            && (tick - self.evaluation_start_tick).is_multiple_of(self.evaluation_period_ticks)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CognitionContractError> {
        self.validate()?;
        let mut payload = Writer::new();
        payload.u64(self.evaluation_start_tick);
        payload.u64(self.evaluation_period_ticks);
        payload.u16(self.retrieval_limit);
        payload.i32(self.goal_switch_threshold_q16);
        payload.i32(self.emergency_health_threshold);
        payload.u8(self.planner_max_depth);
        payload.u16(self.planner_max_expanded_nodes);
        for value in [
            &self.ordinary_goal_id,
            &self.emergency_goal_id,
            &self.navigate_action_id,
            &self.hold_action_id,
            &self.route_known_fact_id,
            &self.travel_needed_fact_id,
            &self.emergency_fact_id,
            &self.navigate_ready_fact_id,
            &self.hold_ready_fact_id,
        ] {
            payload.text(value.as_str())?;
        }
        payload.count(self.seed_beliefs.len())?;
        for belief in &self.seed_beliefs {
            write_belief(&mut payload, belief)?;
        }
        Ok(encode_canonical_segment(
            AGENT_COGNITION_CATALOG_OWNER_ID,
            AGENT_COGNITION_CATALOG_SCHEMA_ID,
            AGENT_COGNITION_CATALOG_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_ID128,
                    self.catalog_asset_id.as_bytes().to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_ID128, self.subject_id.as_bytes().to_vec()),
                CanonicalField::new(4, CANONICAL_TYPE_BYTES, payload.into_bytes()),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CognitionContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        require_envelope(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            AGENT_COGNITION_CATALOG_OWNER_ID,
            AGENT_COGNITION_CATALOG_SCHEMA_ID,
            AGENT_COGNITION_CATALOG_SEGMENT_ID,
        )?;
        require_fields(
            &segment.fields,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let mut payload = Reader::new(field(&segment.fields, 4)?, limits);
        let value = Self {
            schema_version: read_u16_exact(field(&segment.fields, 1)?)?,
            catalog_asset_id: AssetId::from_bytes(read_exact(field(&segment.fields, 2)?)?),
            subject_id: PersistentId::from_bytes(read_exact(field(&segment.fields, 3)?)?),
            evaluation_start_tick: payload.u64()?,
            evaluation_period_ticks: payload.u64()?,
            retrieval_limit: payload.u16()?,
            goal_switch_threshold_q16: payload.i32()?,
            emergency_health_threshold: payload.i32()?,
            planner_max_depth: payload.u8()?,
            planner_max_expanded_nodes: payload.u16()?,
            ordinary_goal_id: payload.schema_id()?,
            emergency_goal_id: payload.schema_id()?,
            navigate_action_id: payload.schema_id()?,
            hold_action_id: payload.schema_id()?,
            route_known_fact_id: payload.schema_id()?,
            travel_needed_fact_id: payload.schema_id()?,
            emergency_fact_id: payload.schema_id()?,
            navigate_ready_fact_id: payload.schema_id()?,
            hold_ready_fact_id: payload.schema_id()?,
            seed_beliefs: payload.beliefs(COGNITION_MAX_BELIEFS)?,
        };
        payload.finish()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(CognitionContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn revision(&self) -> Result<ContentHash, CognitionContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AgentMemorySnapshotV1 {
    pub schema_version: u16,
    pub revision: u64,
    pub subject_id: PersistentId,
    pub last_retrieval_tick: u64,
    pub beliefs: Vec<SemanticBeliefV1>,
}

impl AgentMemorySnapshotV1 {
    pub fn initial(catalog: &AgentCognitionCatalogV1) -> Result<Self, CognitionContractError> {
        catalog.validate()?;
        let value = Self {
            schema_version: COGNITION_SCHEMA_VERSION,
            revision: 0,
            subject_id: catalog.subject_id,
            last_retrieval_tick: 0,
            beliefs: catalog.seed_beliefs.clone(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn retrieved_at(&self, tick: u64) -> Result<Self, CognitionContractError> {
        if tick <= self.last_retrieval_tick {
            return Err(CognitionContractError::SnapshotInvalid);
        }
        let value = Self {
            schema_version: self.schema_version,
            revision: self
                .revision
                .checked_add(1)
                .ok_or(CognitionContractError::RevisionExhausted)?,
            subject_id: self.subject_id,
            last_retrieval_tick: tick,
            beliefs: self.beliefs.clone(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || self.beliefs.is_empty()
            || self.beliefs.len() > COGNITION_MAX_BELIEFS
            || self.beliefs.windows(2).any(|pair| pair[0] >= pair[1])
            || self
                .beliefs
                .iter()
                .any(|belief| belief.subject_id != self.subject_id || belief.validate().is_err())
            || (self.revision == 0) != (self.last_retrieval_tick == 0)
        {
            return Err(CognitionContractError::SnapshotInvalid);
        }
        Ok(())
    }

    pub fn retrieve(&self, limit: usize) -> Result<Vec<SemanticBeliefV1>, CognitionContractError> {
        self.validate()?;
        if limit == 0 || limit > COGNITION_MAX_RETRIEVED_BELIEFS {
            return Err(CognitionContractError::LimitExceeded);
        }
        let mut beliefs = self
            .beliefs
            .iter()
            .filter(|belief| belief.contradiction == BeliefContradictionV1::Consistent)
            .cloned()
            .collect::<Vec<_>>();
        beliefs.sort_by(|left, right| {
            right
                .confidence_q16
                .cmp(&left.confidence_q16)
                .then_with(|| right.learned_tick.cmp(&left.learned_tick))
                .then_with(|| left.belief_id.cmp(&right.belief_id))
        });
        beliefs.truncate(limit);
        Ok(beliefs)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CognitionContractError> {
        self.validate()?;
        let mut payload = Writer::new();
        payload.u64(self.revision);
        payload.u64(self.last_retrieval_tick);
        payload.count(self.beliefs.len())?;
        for belief in &self.beliefs {
            write_belief(&mut payload, belief)?;
        }
        Ok(encode_canonical_segment(
            AGENT_MEMORY_SNAPSHOT_OWNER_ID,
            AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
            AGENT_MEMORY_SNAPSHOT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_ID128, self.subject_id.as_bytes().to_vec()),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, payload.into_bytes()),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CognitionContractError> {
        let segment = decode_snapshot_segment(
            bytes,
            limits,
            AGENT_MEMORY_SNAPSHOT_OWNER_ID,
            AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
            AGENT_MEMORY_SNAPSHOT_SEGMENT_ID,
        )?;
        let mut payload = Reader::new(field(&segment.fields, 3)?, limits);
        let value = Self {
            schema_version: read_u16_exact(field(&segment.fields, 1)?)?,
            subject_id: PersistentId::from_bytes(read_exact(field(&segment.fields, 2)?)?),
            revision: payload.u64()?,
            last_retrieval_tick: payload.u64()?,
            beliefs: payload.beliefs(COGNITION_MAX_BELIEFS)?,
        };
        payload.finish()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(CognitionContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoalPriorityBandV1 {
    Ordinary = 1,
    Emergency = 2,
}

impl GoalPriorityBandV1 {
    fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
        match tag {
            1 => Ok(Self::Ordinary),
            2 => Ok(Self::Emergency),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GoalCandidateV1 {
    pub goal_id: SchemaId,
    pub priority_band: GoalPriorityBandV1,
    pub target_id_or_none: Option<SchemaId>,
    pub desired_fact_id: SchemaId,
    pub drive_utility_q16: i32,
    pub aspiration_utility_q16: i32,
    pub inertia_utility_q16: i32,
    pub risk_cost_q16: i32,
    pub expected_cost_q16: i32,
    pub total_utility_q16: i32,
    pub cited_belief_ids: Vec<ContentHash>,
}

impl GoalCandidateV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        let components = [
            self.drive_utility_q16,
            self.aspiration_utility_q16,
            self.inertia_utility_q16,
            self.risk_cost_q16,
            self.expected_cost_q16,
        ];
        if components
            .iter()
            .any(|value| value.abs() > COGNITION_MAX_SCORE_Q16)
            || self.risk_cost_q16 < 0
            || self.expected_cost_q16 < 0
            || self.cited_belief_ids.len() > COGNITION_MAX_RETRIEVED_BELIEFS
            || self
                .cited_belief_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self.total_utility_q16
                != self
                    .drive_utility_q16
                    .checked_add(self.aspiration_utility_q16)
                    .and_then(|value| value.checked_add(self.inertia_utility_q16))
                    .and_then(|value| value.checked_sub(self.risk_cost_q16))
                    .and_then(|value| value.checked_sub(self.expected_cost_q16))
                    .ok_or(CognitionContractError::GoalInvalid)?
        {
            return Err(CognitionContractError::GoalInvalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AffordanceExecutionV1 {
    RequestLogicalRoute = 1,
    HoldPosition = 2,
}

impl AffordanceExecutionV1 {
    fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
        match tag {
            1 => Ok(Self::RequestLogicalRoute),
            2 => Ok(Self::HoldPosition),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SemanticAffordanceV1 {
    pub action_id: SchemaId,
    pub owner_revision: u64,
    pub precondition_fact_ids: Vec<SchemaId>,
    pub effect_fact_ids: Vec<SchemaId>,
    pub cost_q16: u32,
    pub execution: AffordanceExecutionV1,
    pub route_plan_hash_or_none: Option<ContentHash>,
    pub target_id_or_none: Option<SchemaId>,
}

impl SemanticAffordanceV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.precondition_fact_ids.len() > COGNITION_MAX_FACTS
            || self.effect_fact_ids.is_empty()
            || self.effect_fact_ids.len() > COGNITION_MAX_FACTS
            || self
                .precondition_fact_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self
                .effect_fact_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self.cost_q16 == 0
            || self.cost_q16 > u32::try_from(COGNITION_MAX_SCORE_Q16).expect("score bound positive")
            || (self.execution == AffordanceExecutionV1::RequestLogicalRoute)
                != (self.route_plan_hash_or_none.is_some() && self.target_id_or_none.is_some())
        {
            return Err(CognitionContractError::AffordanceInvalid);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicViewV1 {
    pub schema_version: u16,
    pub gameplay_tick: u64,
    pub subject_id: PersistentId,
    pub cognition_catalog_revision: ContentHash,
    pub memory_revision: u64,
    pub rpg_owner_revision: u64,
    pub population_record_revision: u64,
    pub current_node_id: SchemaId,
    pub navigation_goal_node_id: SchemaId,
    pub health_current: i32,
    pub health_maximum: i32,
    pub known_fact_ids: Vec<SchemaId>,
    pub retrieved_beliefs: Vec<SemanticBeliefV1>,
    pub affordances: Vec<SemanticAffordanceV1>,
    pub view_hash: ContentHash,
}

impl EpistemicViewV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the view binds every permitted owner projection"
    )]
    pub fn new(
        gameplay_tick: u64,
        subject_id: PersistentId,
        cognition_catalog_revision: ContentHash,
        memory_revision: u64,
        rpg_owner_revision: u64,
        population_record_revision: u64,
        current_node_id: SchemaId,
        navigation_goal_node_id: SchemaId,
        health_current: i32,
        health_maximum: i32,
        mut known_fact_ids: Vec<SchemaId>,
        mut retrieved_beliefs: Vec<SemanticBeliefV1>,
        mut affordances: Vec<SemanticAffordanceV1>,
    ) -> Result<Self, CognitionContractError> {
        known_fact_ids.sort();
        known_fact_ids.dedup();
        retrieved_beliefs.sort_by_key(|belief| belief.belief_id);
        affordances.sort_by(|left, right| left.action_id.cmp(&right.action_id));
        let mut value = Self {
            schema_version: COGNITION_SCHEMA_VERSION,
            gameplay_tick,
            subject_id,
            cognition_catalog_revision,
            memory_revision,
            rpg_owner_revision,
            population_record_revision,
            current_node_id,
            navigation_goal_node_id,
            health_current,
            health_maximum,
            known_fact_ids,
            retrieved_beliefs,
            affordances,
            view_hash: ContentHash::default(),
        };
        value.view_hash = value.computed_hash()?;
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || self.health_maximum <= 0
            || self.health_current < 0
            || self.health_current > self.health_maximum
            || self.known_fact_ids.len() > COGNITION_MAX_FACTS
            || self
                .known_fact_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self.retrieved_beliefs.len() > COGNITION_MAX_RETRIEVED_BELIEFS
            || self
                .retrieved_beliefs
                .windows(2)
                .any(|pair| pair[0].belief_id >= pair[1].belief_id)
            || self.retrieved_beliefs.iter().any(|belief| {
                belief.subject_id != self.subject_id
                    || belief.contradiction != BeliefContradictionV1::Consistent
                    || belief.validate().is_err()
            })
            || self.affordances.len() > COGNITION_MAX_AFFORDANCES
            || self
                .affordances
                .windows(2)
                .any(|pair| pair[0].action_id >= pair[1].action_id)
            || self
                .affordances
                .iter()
                .any(|affordance| affordance.validate().is_err())
            || self.computed_hash()? != self.view_hash
        {
            return Err(CognitionContractError::ViewInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CognitionContractError> {
        let mut writer = Writer::with_domain(b"nextengine.epistemic-view.v1\0");
        writer.u16(self.schema_version);
        writer.u64(self.gameplay_tick);
        writer.id(self.subject_id);
        writer.hash(self.cognition_catalog_revision);
        writer.u64(self.memory_revision);
        writer.u64(self.rpg_owner_revision);
        writer.u64(self.population_record_revision);
        writer.text(self.current_node_id.as_str())?;
        writer.text(self.navigation_goal_node_id.as_str())?;
        writer.i32(self.health_current);
        writer.i32(self.health_maximum);
        writer.schema_ids(&self.known_fact_ids)?;
        writer.count(self.retrieved_beliefs.len())?;
        for belief in &self.retrieved_beliefs {
            write_belief(&mut writer, belief)?;
        }
        writer.count(self.affordances.len())?;
        for affordance in &self.affordances {
            write_affordance(&mut writer, affordance)?;
        }
        Ok(writer.into_bytes())
    }

    fn computed_hash(&self) -> Result<ContentHash, CognitionContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriveViewV1 {
    pub schema_version: u16,
    pub gameplay_tick: u64,
    pub safety_pressure_q16: i32,
    pub duty_pressure_q16: i32,
}

impl DriveViewV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || !(0..=COGNITION_Q16_ONE).contains(&self.safety_pressure_q16)
            || !(0..=COGNITION_Q16_ONE).contains(&self.duty_pressure_q16)
        {
            return Err(CognitionContractError::ViewInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CognitionContractError> {
        self.validate()?;
        let mut writer = Writer::with_domain(b"nextengine.drive-view.v1\0");
        writer.u16(self.schema_version);
        writer.u64(self.gameplay_tick);
        writer.i32(self.safety_pressure_q16);
        writer.i32(self.duty_pressure_q16);
        Ok(writer.into_bytes())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActiveGoalV1 {
    pub goal_id: SchemaId,
    pub priority_band: GoalPriorityBandV1,
    pub target_id_or_none: Option<SchemaId>,
    pub desired_fact_id: SchemaId,
    pub selected_utility_q16: i32,
    pub selected_tick: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SuspendedGoalV1 {
    pub goal: ActiveGoalV1,
    pub suspended_tick: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StrategicPlanStepV1 {
    pub action_id: SchemaId,
    pub owner_revision: u64,
    pub cost_q16: u32,
    pub execution: AffordanceExecutionV1,
    pub route_plan_hash_or_none: Option<ContentHash>,
    pub target_id_or_none: Option<SchemaId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StrategicPlanV1 {
    pub goal_id: SchemaId,
    pub desired_fact_id: SchemaId,
    pub steps: Vec<StrategicPlanStepV1>,
    pub cursor: u8,
    pub expanded_nodes: u16,
    pub planner_max_depth: u8,
    pub planner_max_expanded_nodes: u16,
}

impl StrategicPlanV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.steps.is_empty()
            || self.steps.len() > COGNITION_MAX_PLAN_STEPS
            || usize::from(self.cursor) >= self.steps.len()
            || self.expanded_nodes == 0
            || self.expanded_nodes > self.planner_max_expanded_nodes
            || self.steps.len() > usize::from(self.planner_max_depth)
            || self.steps.iter().any(|step| {
                step.cost_q16 == 0
                    || (step.execution == AffordanceExecutionV1::RequestLogicalRoute)
                        != (step.route_plan_hash_or_none.is_some()
                            && step.target_id_or_none.is_some())
            })
        {
            return Err(CognitionContractError::PlanInvalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TaskLifecycleV1 {
    Pending = 1,
    Active = 2,
    Succeeded = 3,
    Failed = 4,
    Cancelled = 5,
    Suspended = 6,
}

impl TaskLifecycleV1 {
    fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
        match tag {
            1 => Ok(Self::Pending),
            2 => Ok(Self::Active),
            3 => Ok(Self::Succeeded),
            4 => Ok(Self::Failed),
            5 => Ok(Self::Cancelled),
            6 => Ok(Self::Suspended),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PrivateTaskStateV1 {
    pub action_id: SchemaId,
    pub lifecycle: TaskLifecycleV1,
    pub started_tick: u64,
    pub attempt: u16,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StrategicAgentIntentKindV1 {
    RequestLogicalRoute {
        start_node_id: SchemaId,
        goal_node_id: SchemaId,
        route_plan_hash: ContentHash,
    },
    HoldPosition,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StrategicAgentIntentV1 {
    pub schema_version: u16,
    pub intent_id: ContentHash,
    pub subject_id: PersistentId,
    pub goal_id: SchemaId,
    pub action_id: SchemaId,
    pub creation_tick: u64,
    pub expiry_tick: u64,
    pub kind: StrategicAgentIntentKindV1,
}

impl StrategicAgentIntentV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || self.creation_tick.checked_add(1) != Some(self.expiry_tick)
            || self.computed_id()? != self.intent_id
        {
            return Err(CognitionContractError::IntentInvalid);
        }
        Ok(())
    }

    pub fn request_logical_route(
        subject_id: PersistentId,
        goal_id: SchemaId,
        action_id: SchemaId,
        creation_tick: u64,
        start_node_id: SchemaId,
        goal_node_id: SchemaId,
        route_plan_hash: ContentHash,
    ) -> Result<Self, CognitionContractError> {
        let mut value = Self {
            schema_version: COGNITION_SCHEMA_VERSION,
            intent_id: ContentHash::default(),
            subject_id,
            goal_id,
            action_id,
            creation_tick,
            expiry_tick: creation_tick
                .checked_add(1)
                .ok_or(CognitionContractError::LimitExceeded)?,
            kind: StrategicAgentIntentKindV1::RequestLogicalRoute {
                start_node_id,
                goal_node_id,
                route_plan_hash,
            },
        };
        value.intent_id = value.computed_id()?;
        value.validate()?;
        Ok(value)
    }

    pub fn hold_position(
        subject_id: PersistentId,
        goal_id: SchemaId,
        action_id: SchemaId,
        creation_tick: u64,
    ) -> Result<Self, CognitionContractError> {
        let mut value = Self {
            schema_version: COGNITION_SCHEMA_VERSION,
            intent_id: ContentHash::default(),
            subject_id,
            goal_id,
            action_id,
            creation_tick,
            expiry_tick: creation_tick
                .checked_add(1)
                .ok_or(CognitionContractError::LimitExceeded)?,
            kind: StrategicAgentIntentKindV1::HoldPosition,
        };
        value.intent_id = value.computed_id()?;
        value.validate()?;
        Ok(value)
    }

    fn computed_id(&self) -> Result<ContentHash, CognitionContractError> {
        let mut writer = Writer::with_domain(b"nextengine.strategic-agent-intent.v1\0");
        write_intent_identity(&mut writer, self)?;
        Ok(writer.finish_hash())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AgentCognitionSnapshotV1 {
    pub schema_version: u16,
    pub revision: u64,
    pub subject_id: PersistentId,
    pub active_goal_or_none: Option<ActiveGoalV1>,
    pub suspended_goals: Vec<SuspendedGoalV1>,
    pub plan_or_none: Option<StrategicPlanV1>,
    pub task_or_none: Option<PrivateTaskStateV1>,
    pub pending_intent_or_none: Option<StrategicAgentIntentV1>,
    pub last_selected_utility_q16: i32,
    pub decision_rng_state: u64,
    pub last_epistemic_hash: ContentHash,
}

impl AgentCognitionSnapshotV1 {
    #[must_use]
    pub const fn initial(subject_id: PersistentId, decision_rng_state: u64) -> Self {
        Self {
            schema_version: COGNITION_SCHEMA_VERSION,
            revision: 0,
            subject_id,
            active_goal_or_none: None,
            suspended_goals: Vec::new(),
            plan_or_none: None,
            task_or_none: None,
            pending_intent_or_none: None,
            last_selected_utility_q16: 0,
            decision_rng_state,
            last_epistemic_hash: ContentHash::from_bytes([0; 32]),
        }
    }

    pub fn validate(&self) -> Result<(), CognitionContractError> {
        let empty = self.revision == 0;
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || self.suspended_goals.len() > COGNITION_MAX_SUSPENDED_GOALS
            || self
                .suspended_goals
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self
                .plan_or_none
                .as_ref()
                .is_some_and(|plan| plan.validate().is_err())
            || self
                .pending_intent_or_none
                .as_ref()
                .is_some_and(|intent| intent.validate().is_err())
            || empty
                != (self.active_goal_or_none.is_none()
                    && self.plan_or_none.is_none()
                    && self.task_or_none.is_none()
                    && self.pending_intent_or_none.is_none()
                    && self.last_epistemic_hash == ContentHash::default())
        {
            return Err(CognitionContractError::SnapshotInvalid);
        }
        if let Some(goal) = &self.active_goal_or_none
            && (self
                .plan_or_none
                .as_ref()
                .is_none_or(|plan| plan.goal_id != goal.goal_id)
                || self
                    .task_or_none
                    .as_ref()
                    .zip(self.plan_or_none.as_ref())
                    .is_none_or(|(task, plan)| {
                        plan.steps
                            .get(usize::from(plan.cursor))
                            .is_none_or(|step| step.action_id != task.action_id)
                    })
                || self.pending_intent_or_none.as_ref().is_none_or(|intent| {
                    intent.subject_id != self.subject_id || intent.goal_id != goal.goal_id
                }))
        {
            return Err(CognitionContractError::SnapshotInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CognitionContractError> {
        self.validate()?;
        let mut payload = Writer::new();
        payload.u64(self.revision);
        write_optional_active_goal(&mut payload, self.active_goal_or_none.as_ref())?;
        payload.count(self.suspended_goals.len())?;
        for suspended in &self.suspended_goals {
            write_active_goal(&mut payload, &suspended.goal)?;
            payload.u64(suspended.suspended_tick);
        }
        write_optional_plan(&mut payload, self.plan_or_none.as_ref())?;
        write_optional_task(&mut payload, self.task_or_none.as_ref())?;
        write_optional_intent(&mut payload, self.pending_intent_or_none.as_ref())?;
        payload.i32(self.last_selected_utility_q16);
        payload.u64(self.decision_rng_state);
        payload.hash(self.last_epistemic_hash);
        Ok(encode_canonical_segment(
            AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
            AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID,
            AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_ID128, self.subject_id.as_bytes().to_vec()),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, payload.into_bytes()),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CognitionContractError> {
        let segment = decode_snapshot_segment(
            bytes,
            limits,
            AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
            AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID,
            AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID,
        )?;
        let mut payload = Reader::new(field(&segment.fields, 3)?, limits);
        let value = Self {
            schema_version: read_u16_exact(field(&segment.fields, 1)?)?,
            subject_id: PersistentId::from_bytes(read_exact(field(&segment.fields, 2)?)?),
            revision: payload.u64()?,
            active_goal_or_none: payload.optional_active_goal()?,
            suspended_goals: payload.suspended_goals()?,
            plan_or_none: payload.optional_plan()?,
            task_or_none: payload.optional_task()?,
            pending_intent_or_none: payload.optional_intent()?,
            last_selected_utility_q16: payload.i32()?,
            decision_rng_state: payload.u64()?,
            last_epistemic_hash: payload.hash()?,
        };
        payload.finish()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(CognitionContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DecisionSwitchReasonV1 {
    InitialSelection = 1,
    RetainedByInertia = 2,
    HigherUtility = 3,
    EmergencyInterrupt = 4,
    EmergencyExitResume = 5,
    SafeFallback = 6,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PlanningFailureV1 {
    None = 0,
    MissingAffordance = 1,
    PlanBudgetExhausted = 2,
    RouteUnavailable = 3,
    StaleEpistemicView = 4,
}

impl PlanningFailureV1 {
    #[must_use]
    pub const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::None => "STRATEGIC_PLAN_OK",
            Self::MissingAffordance => "STRATEGIC_AFFORDANCE_MISSING",
            Self::PlanBudgetExhausted => "STRATEGIC_PLAN_BUDGET_EXHAUSTED",
            Self::RouteUnavailable => "STRATEGIC_ROUTE_UNAVAILABLE",
            Self::StaleEpistemicView => "STRATEGIC_EPISTEMIC_STALE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionTraceV1 {
    pub schema_version: u16,
    pub gameplay_tick: u64,
    pub subject_id: PersistentId,
    pub epistemic_view_hash: ContentHash,
    pub drive_view_hash: ContentHash,
    pub candidates: Vec<GoalCandidateV1>,
    pub selected_goal_id: SchemaId,
    pub switch_reason: DecisionSwitchReasonV1,
    pub plan_or_none: Option<StrategicPlanV1>,
    pub planning_failure: PlanningFailureV1,
    pub intent_id_or_none: Option<ContentHash>,
    pub trace_hash: ContentHash,
}

impl DecisionTraceV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the diagnostic projection binds every decision input and result"
    )]
    pub fn new(
        gameplay_tick: u64,
        subject_id: PersistentId,
        epistemic_view_hash: ContentHash,
        drive_view_hash: ContentHash,
        candidates: Vec<GoalCandidateV1>,
        selected_goal_id: SchemaId,
        switch_reason: DecisionSwitchReasonV1,
        plan_or_none: Option<StrategicPlanV1>,
        planning_failure: PlanningFailureV1,
        intent_id_or_none: Option<ContentHash>,
    ) -> Result<Self, CognitionContractError> {
        let mut value = Self {
            schema_version: COGNITION_SCHEMA_VERSION,
            gameplay_tick,
            subject_id,
            epistemic_view_hash,
            drive_view_hash,
            candidates,
            selected_goal_id,
            switch_reason,
            plan_or_none,
            planning_failure,
            intent_id_or_none,
            trace_hash: ContentHash::default(),
        };
        value.trace_hash = value.computed_hash()?;
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || self.candidates.is_empty()
            || self.candidates.len() > COGNITION_MAX_GOALS
            || self
                .candidates
                .iter()
                .any(|candidate| candidate.validate().is_err())
            || self
                .candidates
                .windows(2)
                .any(|pair| candidate_order(&pair[0], &pair[1]) != std::cmp::Ordering::Less)
            || !self
                .candidates
                .iter()
                .any(|candidate| candidate.goal_id == self.selected_goal_id)
            || (self.planning_failure == PlanningFailureV1::None) != self.plan_or_none.is_some()
            || self
                .plan_or_none
                .as_ref()
                .is_some_and(|plan| plan.validate().is_err())
            || self.computed_hash()? != self.trace_hash
        {
            return Err(CognitionContractError::TraceInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CognitionContractError> {
        let mut writer = Writer::with_domain(b"nextengine.decision-trace.v1\0");
        writer.u16(self.schema_version);
        writer.u64(self.gameplay_tick);
        writer.id(self.subject_id);
        writer.hash(self.epistemic_view_hash);
        writer.hash(self.drive_view_hash);
        writer.count(self.candidates.len())?;
        for candidate in &self.candidates {
            write_goal_candidate(&mut writer, candidate)?;
        }
        writer.text(self.selected_goal_id.as_str())?;
        writer.u8(self.switch_reason as u8);
        write_optional_plan(&mut writer, self.plan_or_none.as_ref())?;
        writer.u8(self.planning_failure as u8);
        writer.optional_hash(self.intent_id_or_none);
        Ok(writer.into_bytes())
    }

    fn computed_hash(&self) -> Result<ContentHash, CognitionContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AgentCognitionCommandV1 {
    CommitDecision {
        expected_agent_revision: u64,
        expected_memory_revision: u64,
        cognition_catalog_revision: ContentHash,
        epistemic_view_hash: ContentHash,
        next_agent_snapshot: AgentCognitionSnapshotV1,
        next_memory_snapshot: AgentMemorySnapshotV1,
    },
}

impl AgentCognitionCommandV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        match self {
            Self::CommitDecision {
                expected_agent_revision,
                expected_memory_revision,
                cognition_catalog_revision,
                epistemic_view_hash,
                next_agent_snapshot,
                next_memory_snapshot,
            } => {
                next_agent_snapshot.validate()?;
                next_memory_snapshot.validate()?;
                if next_agent_snapshot.subject_id != next_memory_snapshot.subject_id
                    || expected_agent_revision.checked_add(1) != Some(next_agent_snapshot.revision)
                    || expected_memory_revision.checked_add(1)
                        != Some(next_memory_snapshot.revision)
                    || *cognition_catalog_revision == ContentHash::default()
                    || *epistemic_view_hash == ContentHash::default()
                    || next_agent_snapshot.last_epistemic_hash != *epistemic_view_hash
                {
                    return Err(CognitionContractError::CommandInvalid);
                }
            }
        }
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate()
            .map_err(|_| CanonicalError::LengthOverflow)?;
        let mut writer = Writer::with_domain(b"nextengine.agent-cognition-command.v1\0");
        writer.u16(COGNITION_SCHEMA_VERSION);
        match self {
            Self::CommitDecision {
                expected_agent_revision,
                expected_memory_revision,
                cognition_catalog_revision,
                epistemic_view_hash,
                next_agent_snapshot,
                next_memory_snapshot,
            } => {
                writer.u8(1);
                writer.u64(*expected_agent_revision);
                writer.u64(*expected_memory_revision);
                writer.hash(*cognition_catalog_revision);
                writer.hash(*epistemic_view_hash);
                writer.bytes(
                    &next_agent_snapshot
                        .canonical_bytes()
                        .map_err(|_| CanonicalError::LengthOverflow)?,
                )?;
                writer.bytes(
                    &next_memory_snapshot
                        .canonical_bytes()
                        .map_err(|_| CanonicalError::LengthOverflow)?,
                )?;
            }
        }
        Ok(writer.into_bytes())
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CognitionContractError> {
        let mut reader = Reader::new(bytes, limits);
        reader.domain(b"nextengine.agent-cognition-command.v1\0")?;
        if reader.u16()? != COGNITION_SCHEMA_VERSION || reader.u8()? != 1 {
            return Err(CognitionContractError::CommandInvalid);
        }
        let value = Self::CommitDecision {
            expected_agent_revision: reader.u64()?,
            expected_memory_revision: reader.u64()?,
            cognition_catalog_revision: reader.hash()?,
            epistemic_view_hash: reader.hash()?,
            next_agent_snapshot: AgentCognitionSnapshotV1::from_canonical_bytes(
                reader.bytes()?,
                limits,
            )?,
            next_memory_snapshot: AgentMemorySnapshotV1::from_canonical_bytes(
                reader.bytes()?,
                limits,
            )?,
        };
        reader.finish()?;
        value.validate()?;
        if value.canonical_payload_bytes()? != bytes {
            return Err(CognitionContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AgentDecisionCommittedV1 {
    pub schema_version: u16,
    pub subject_id: PersistentId,
    pub agent_revision: u64,
    pub memory_revision: u64,
    pub active_goal_id: SchemaId,
    pub intent_id: ContentHash,
}

impl AgentDecisionCommittedV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || self.agent_revision == 0
            || self.memory_revision == 0
            || self.intent_id == ContentHash::default()
        {
            return Err(CognitionContractError::EventInvalid);
        }
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate()
            .map_err(|_| CanonicalError::LengthOverflow)?;
        let mut writer = Writer::with_domain(b"nextengine.agent-decision-committed.v1\0");
        writer.u16(self.schema_version);
        writer.id(self.subject_id);
        writer.u64(self.agent_revision);
        writer.u64(self.memory_revision);
        writer
            .text(self.active_goal_id.as_str())
            .map_err(|_| CanonicalError::LengthOverflow)?;
        writer.hash(self.intent_id);
        Ok(writer.into_bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CognitionContractError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    ContentInvalid,
    BeliefInvalid,
    ViewInvalid,
    GoalInvalid,
    AffordanceInvalid,
    PlanInvalid,
    IntentInvalid,
    SnapshotInvalid,
    TraceInvalid,
    CommandInvalid,
    EventInvalid,
    RevisionExhausted,
    LimitExceeded,
    MissingField(u32),
    UnknownField(u32),
    FieldType,
    WrongEnvelope,
    UnknownTag(u8),
    NonCanonicalEncoding,
}

impl CognitionContractError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::RevisionExhausted => "AGENT_COGNITION_REVISION_EXHAUSTED",
            Self::LimitExceeded => "AGENT_COGNITION_LIMIT_EXCEEDED",
            _ => "AGENT_COGNITION_CONTRACT_INVALID",
        }
    }
}

impl Display for CognitionContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for CognitionContractError {}

impl From<CanonicalError> for CognitionContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for CognitionContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for CognitionContractError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

pub fn candidate_order(left: &GoalCandidateV1, right: &GoalCandidateV1) -> std::cmp::Ordering {
    right
        .priority_band
        .cmp(&left.priority_band)
        .then_with(|| right.total_utility_q16.cmp(&left.total_utility_q16))
        .then_with(|| left.goal_id.cmp(&right.goal_id))
        .then_with(|| left.target_id_or_none.cmp(&right.target_id_or_none))
}

struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn with_domain(domain: &[u8]) -> Self {
        Self {
            bytes: domain.to_vec(),
        }
    }

    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn id(&mut self, value: PersistentId) {
        self.bytes.extend_from_slice(value.as_bytes());
    }

    fn hash(&mut self, value: ContentHash) {
        self.bytes.extend_from_slice(value.as_bytes());
    }

    fn optional_hash(&mut self, value: Option<ContentHash>) {
        match value {
            None => self.u8(0),
            Some(value) => {
                self.u8(1);
                self.hash(value);
            }
        }
    }

    fn count(&mut self, count: usize) -> Result<(), CognitionContractError> {
        self.u32(u32::try_from(count).map_err(|_| CognitionContractError::LimitExceeded)?);
        Ok(())
    }

    fn text(&mut self, value: &str) -> Result<(), CognitionContractError> {
        if value.len() > COGNITION_MAX_TEXT_BYTES {
            return Err(CognitionContractError::LimitExceeded);
        }
        self.count(value.len())?;
        self.bytes.extend_from_slice(value.as_bytes());
        Ok(())
    }

    fn bytes(&mut self, value: &[u8]) -> Result<(), CanonicalError> {
        self.bytes.extend_from_slice(
            &u32::try_from(value.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn schema_ids(&mut self, values: &[SchemaId]) -> Result<(), CognitionContractError> {
        self.count(values.len())?;
        for value in values {
            self.text(value.as_str())?;
        }
        Ok(())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    fn finish_hash(self) -> ContentHash {
        content_hash_from_bytes(sha256(&self.bytes))
    }
}

struct Reader<'a> {
    cursor: CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], limits: CanonicalDecodeLimits) -> Self {
        Self {
            cursor: CanonicalCursor::new(bytes),
            limits,
        }
    }

    fn domain(&mut self, expected: &[u8]) -> Result<(), CognitionContractError> {
        if self.cursor.read_exact(expected.len())? != expected {
            return Err(CognitionContractError::NonCanonicalEncoding);
        }
        Ok(())
    }

    fn u8(&mut self) -> Result<u8, CognitionContractError> {
        Ok(self.cursor.read_u8()?)
    }

    fn u16(&mut self) -> Result<u16, CognitionContractError> {
        Ok(self.cursor.read_u16()?)
    }

    fn u32(&mut self) -> Result<u32, CognitionContractError> {
        Ok(self.cursor.read_u32()?)
    }

    fn u64(&mut self) -> Result<u64, CognitionContractError> {
        Ok(self.cursor.read_u64()?)
    }

    fn i32(&mut self) -> Result<i32, CognitionContractError> {
        Ok(i32::from_le_bytes(read_exact(self.cursor.read_exact(4)?)?))
    }

    fn id(&mut self) -> Result<PersistentId, CognitionContractError> {
        Ok(PersistentId::from_bytes(read_exact(
            self.cursor.read_exact(16)?,
        )?))
    }

    fn hash(&mut self) -> Result<ContentHash, CognitionContractError> {
        Ok(ContentHash::from_bytes(read_exact(
            self.cursor.read_exact(32)?,
        )?))
    }

    fn optional_hash(&mut self) -> Result<Option<ContentHash>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.hash()?)),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn count(&mut self, limit: usize) -> Result<usize, CognitionContractError> {
        Ok(self
            .cursor
            .read_count(limit, |actual, limit| CanonicalDecodeError::TooManyFields {
                actual,
                limit,
            })?)
    }

    fn text(&mut self) -> Result<&'a str, CognitionContractError> {
        let bytes = self
            .cursor
            .read_u32_length_prefixed(COGNITION_MAX_TEXT_BYTES)?;
        Ok(std::str::from_utf8(bytes).map_err(|_| CanonicalDecodeError::InvalidUtf8)?)
    }

    fn schema_id(&mut self) -> Result<SchemaId, CognitionContractError> {
        Ok(SchemaId::new(self.text()?)?)
    }

    fn bytes(&mut self) -> Result<&'a [u8], CognitionContractError> {
        let length =
            usize::try_from(self.u32()?).map_err(|_| CognitionContractError::LimitExceeded)?;
        if length > self.limits.max_field_payload_bytes {
            return Err(CognitionContractError::LimitExceeded);
        }
        Ok(self.cursor.read_exact(length)?)
    }

    fn beliefs(&mut self, limit: usize) -> Result<Vec<SemanticBeliefV1>, CognitionContractError> {
        (0..self.count(limit)?).map(|_| read_belief(self)).collect()
    }

    fn optional_active_goal(&mut self) -> Result<Option<ActiveGoalV1>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(read_active_goal(self)?)),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn suspended_goals(&mut self) -> Result<Vec<SuspendedGoalV1>, CognitionContractError> {
        (0..self.count(COGNITION_MAX_SUSPENDED_GOALS)?)
            .map(|_| {
                Ok(SuspendedGoalV1 {
                    goal: read_active_goal(self)?,
                    suspended_tick: self.u64()?,
                })
            })
            .collect()
    }

    fn optional_plan(&mut self) -> Result<Option<StrategicPlanV1>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(read_plan(self)?)),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn optional_task(&mut self) -> Result<Option<PrivateTaskStateV1>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(PrivateTaskStateV1 {
                action_id: self.schema_id()?,
                lifecycle: TaskLifecycleV1::from_tag(self.u8()?)?,
                started_tick: self.u64()?,
                attempt: self.u16()?,
            })),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn optional_intent(
        &mut self,
    ) -> Result<Option<StrategicAgentIntentV1>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(read_intent(self)?)),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn finish(self) -> Result<(), CognitionContractError> {
        Ok(self.cursor.finish()?)
    }
}

fn write_belief_identity(
    writer: &mut Writer,
    belief: &SemanticBeliefV1,
) -> Result<(), CognitionContractError> {
    writer.id(belief.subject_id);
    writer.text(belief.predicate_id.as_str())?;
    writer.text(belief.value_id.as_str())?;
    writer.u32(belief.confidence_q16);
    writer.u8(belief.source as u8);
    writer.u64(belief.learned_tick);
    writer.u64(belief.last_verified_revision);
    writer.u8(belief.contradiction as u8);
    Ok(())
}

fn write_belief(
    writer: &mut Writer,
    belief: &SemanticBeliefV1,
) -> Result<(), CognitionContractError> {
    belief.validate()?;
    writer.hash(belief.belief_id);
    write_belief_identity(writer, belief)
}

fn read_belief(reader: &mut Reader<'_>) -> Result<SemanticBeliefV1, CognitionContractError> {
    let value = SemanticBeliefV1 {
        belief_id: reader.hash()?,
        subject_id: reader.id()?,
        predicate_id: reader.schema_id()?,
        value_id: reader.schema_id()?,
        confidence_q16: reader.u32()?,
        source: BeliefSourceV1::from_tag(reader.u8()?)?,
        learned_tick: reader.u64()?,
        last_verified_revision: reader.u64()?,
        contradiction: BeliefContradictionV1::from_tag(reader.u8()?)?,
    };
    value.validate()?;
    Ok(value)
}

fn write_affordance(
    writer: &mut Writer,
    value: &SemanticAffordanceV1,
) -> Result<(), CognitionContractError> {
    value.validate()?;
    writer.text(value.action_id.as_str())?;
    writer.u64(value.owner_revision);
    writer.schema_ids(&value.precondition_fact_ids)?;
    writer.schema_ids(&value.effect_fact_ids)?;
    writer.u32(value.cost_q16);
    writer.u8(value.execution as u8);
    writer.optional_hash(value.route_plan_hash_or_none);
    write_optional_schema_id(writer, value.target_id_or_none.as_ref())
}

fn write_goal_candidate(
    writer: &mut Writer,
    value: &GoalCandidateV1,
) -> Result<(), CognitionContractError> {
    value.validate()?;
    writer.text(value.goal_id.as_str())?;
    writer.u8(value.priority_band as u8);
    write_optional_schema_id(writer, value.target_id_or_none.as_ref())?;
    writer.text(value.desired_fact_id.as_str())?;
    writer.i32(value.drive_utility_q16);
    writer.i32(value.aspiration_utility_q16);
    writer.i32(value.inertia_utility_q16);
    writer.i32(value.risk_cost_q16);
    writer.i32(value.expected_cost_q16);
    writer.i32(value.total_utility_q16);
    writer.count(value.cited_belief_ids.len())?;
    for belief_id in &value.cited_belief_ids {
        writer.hash(*belief_id);
    }
    Ok(())
}

fn write_active_goal(
    writer: &mut Writer,
    value: &ActiveGoalV1,
) -> Result<(), CognitionContractError> {
    writer.text(value.goal_id.as_str())?;
    writer.u8(value.priority_band as u8);
    write_optional_schema_id(writer, value.target_id_or_none.as_ref())?;
    writer.text(value.desired_fact_id.as_str())?;
    writer.i32(value.selected_utility_q16);
    writer.u64(value.selected_tick);
    Ok(())
}

fn read_active_goal(reader: &mut Reader<'_>) -> Result<ActiveGoalV1, CognitionContractError> {
    Ok(ActiveGoalV1 {
        goal_id: reader.schema_id()?,
        priority_band: GoalPriorityBandV1::from_tag(reader.u8()?)?,
        target_id_or_none: read_optional_schema_id(reader)?,
        desired_fact_id: reader.schema_id()?,
        selected_utility_q16: reader.i32()?,
        selected_tick: reader.u64()?,
    })
}

fn write_optional_active_goal(
    writer: &mut Writer,
    value: Option<&ActiveGoalV1>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            write_active_goal(writer, value)?;
        }
    }
    Ok(())
}

fn write_plan(writer: &mut Writer, value: &StrategicPlanV1) -> Result<(), CognitionContractError> {
    value.validate()?;
    writer.text(value.goal_id.as_str())?;
    writer.text(value.desired_fact_id.as_str())?;
    writer.count(value.steps.len())?;
    for step in &value.steps {
        writer.text(step.action_id.as_str())?;
        writer.u64(step.owner_revision);
        writer.u32(step.cost_q16);
        writer.u8(step.execution as u8);
        writer.optional_hash(step.route_plan_hash_or_none);
        write_optional_schema_id(writer, step.target_id_or_none.as_ref())?;
    }
    writer.u8(value.cursor);
    writer.u16(value.expanded_nodes);
    writer.u8(value.planner_max_depth);
    writer.u16(value.planner_max_expanded_nodes);
    Ok(())
}

fn read_plan(reader: &mut Reader<'_>) -> Result<StrategicPlanV1, CognitionContractError> {
    let goal_id = reader.schema_id()?;
    let desired_fact_id = reader.schema_id()?;
    let steps = (0..reader.count(COGNITION_MAX_PLAN_STEPS)?)
        .map(|_| {
            Ok(StrategicPlanStepV1 {
                action_id: reader.schema_id()?,
                owner_revision: reader.u64()?,
                cost_q16: reader.u32()?,
                execution: AffordanceExecutionV1::from_tag(reader.u8()?)?,
                route_plan_hash_or_none: reader.optional_hash()?,
                target_id_or_none: read_optional_schema_id(reader)?,
            })
        })
        .collect::<Result<Vec<_>, CognitionContractError>>()?;
    let value = StrategicPlanV1 {
        goal_id,
        desired_fact_id,
        steps,
        cursor: reader.u8()?,
        expanded_nodes: reader.u16()?,
        planner_max_depth: reader.u8()?,
        planner_max_expanded_nodes: reader.u16()?,
    };
    value.validate()?;
    Ok(value)
}

fn write_optional_plan(
    writer: &mut Writer,
    value: Option<&StrategicPlanV1>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            write_plan(writer, value)?;
        }
    }
    Ok(())
}

fn write_optional_task(
    writer: &mut Writer,
    value: Option<&PrivateTaskStateV1>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            writer.text(value.action_id.as_str())?;
            writer.u8(value.lifecycle as u8);
            writer.u64(value.started_tick);
            writer.u16(value.attempt);
        }
    }
    Ok(())
}

fn write_intent_identity(
    writer: &mut Writer,
    value: &StrategicAgentIntentV1,
) -> Result<(), CognitionContractError> {
    writer.u16(value.schema_version);
    writer.id(value.subject_id);
    writer.text(value.goal_id.as_str())?;
    writer.text(value.action_id.as_str())?;
    writer.u64(value.creation_tick);
    writer.u64(value.expiry_tick);
    match &value.kind {
        StrategicAgentIntentKindV1::RequestLogicalRoute {
            start_node_id,
            goal_node_id,
            route_plan_hash,
        } => {
            writer.u8(1);
            writer.text(start_node_id.as_str())?;
            writer.text(goal_node_id.as_str())?;
            writer.hash(*route_plan_hash);
        }
        StrategicAgentIntentKindV1::HoldPosition => writer.u8(2),
    }
    Ok(())
}

fn write_intent(
    writer: &mut Writer,
    value: &StrategicAgentIntentV1,
) -> Result<(), CognitionContractError> {
    value.validate()?;
    writer.hash(value.intent_id);
    write_intent_identity(writer, value)
}

fn read_intent(reader: &mut Reader<'_>) -> Result<StrategicAgentIntentV1, CognitionContractError> {
    let intent_id = reader.hash()?;
    let schema_version = reader.u16()?;
    let subject_id = reader.id()?;
    let goal_id = reader.schema_id()?;
    let action_id = reader.schema_id()?;
    let creation_tick = reader.u64()?;
    let expiry_tick = reader.u64()?;
    let kind = match reader.u8()? {
        1 => StrategicAgentIntentKindV1::RequestLogicalRoute {
            start_node_id: reader.schema_id()?,
            goal_node_id: reader.schema_id()?,
            route_plan_hash: reader.hash()?,
        },
        2 => StrategicAgentIntentKindV1::HoldPosition,
        value => return Err(CognitionContractError::UnknownTag(value)),
    };
    let value = StrategicAgentIntentV1 {
        schema_version,
        intent_id,
        subject_id,
        goal_id,
        action_id,
        creation_tick,
        expiry_tick,
        kind,
    };
    value.validate()?;
    Ok(value)
}

fn write_optional_intent(
    writer: &mut Writer,
    value: Option<&StrategicAgentIntentV1>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            write_intent(writer, value)?;
        }
    }
    Ok(())
}

fn write_optional_schema_id(
    writer: &mut Writer,
    value: Option<&SchemaId>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            writer.text(value.as_str())?;
        }
    }
    Ok(())
}

fn read_optional_schema_id(
    reader: &mut Reader<'_>,
) -> Result<Option<SchemaId>, CognitionContractError> {
    match reader.u8()? {
        0 => Ok(None),
        1 => Ok(Some(reader.schema_id()?)),
        value => Err(CognitionContractError::UnknownTag(value)),
    }
}

fn decode_snapshot_segment(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment_id: &str,
) -> Result<crate::canonical::DecodedCanonicalSegment, CognitionContractError> {
    let segment = decode_canonical_segment(bytes, limits)?;
    require_envelope(
        &segment.owner_id,
        &segment.schema_id,
        &segment.segment_id,
        owner,
        schema,
        segment_id,
    )?;
    require_fields(
        &segment.fields,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_ID128),
            (3, CANONICAL_TYPE_BYTES),
        ],
    )?;
    Ok(segment)
}

fn require_envelope(
    owner: &str,
    schema: &str,
    segment: &str,
    expected_owner: &str,
    expected_schema: &str,
    expected_segment: &str,
) -> Result<(), CognitionContractError> {
    if owner != expected_owner || schema != expected_schema || segment != expected_segment {
        return Err(CognitionContractError::WrongEnvelope);
    }
    Ok(())
}

fn require_fields(
    actual: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), CognitionContractError> {
    for field in actual {
        let Some((_, tag)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(CognitionContractError::UnknownField(field.field_id));
        };
        if field.type_tag != *tag {
            return Err(CognitionContractError::FieldType);
        }
    }
    for (id, _) in expected {
        if !actual.iter().any(|field| field.field_id == *id) {
            return Err(CognitionContractError::MissingField(*id));
        }
    }
    Ok(())
}

fn field(fields: &[CanonicalField], id: u32) -> Result<&[u8], CognitionContractError> {
    Ok(&fields
        .iter()
        .find(|field| field.field_id == id)
        .ok_or(CognitionContractError::MissingField(id))?
        .payload)
}

fn read_exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], CognitionContractError> {
    bytes
        .try_into()
        .map_err(|_| CognitionContractError::NonCanonicalEncoding)
}

fn read_u16_exact(bytes: &[u8]) -> Result<u16, CognitionContractError> {
    Ok(u16::from_le_bytes(read_exact(bytes)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test schema id")
    }

    fn catalog() -> AgentCognitionCatalogV1 {
        let subject_id = PersistentId::from_bytes([7; 16]);
        let belief = SemanticBeliefV1::new(
            subject_id,
            schema("knowledge.route-purpose"),
            schema("knowledge.frontier-duty"),
            u32::try_from(COGNITION_Q16_ONE).expect("positive"),
            BeliefSourceV1::AuthoredSeed,
            0,
            0,
            BeliefContradictionV1::Consistent,
        )
        .expect("belief");
        AgentCognitionCatalogV1 {
            schema_version: COGNITION_SCHEMA_VERSION,
            catalog_asset_id: AssetId::from_bytes([8; 16]),
            subject_id,
            evaluation_start_tick: 1,
            evaluation_period_ticks: 3,
            retrieval_limit: 4,
            goal_switch_threshold_q16: COGNITION_Q16_ONE / 4,
            emergency_health_threshold: 30,
            planner_max_depth: 2,
            planner_max_expanded_nodes: 8,
            ordinary_goal_id: schema("goal.reach-frontier"),
            emergency_goal_id: schema("goal.preserve-self"),
            navigate_action_id: schema("affordance.request-logical-route"),
            hold_action_id: schema("affordance.hold-position"),
            route_known_fact_id: schema("fact.route-known"),
            travel_needed_fact_id: schema("fact.travel-needed"),
            emergency_fact_id: schema("fact.emergency"),
            navigate_ready_fact_id: schema("fact.navigate-intent-ready"),
            hold_ready_fact_id: schema("fact.hold-intent-ready"),
            seed_beliefs: vec![belief],
        }
    }

    #[test]
    fn cognition_catalog_and_owner_snapshots_round_trip() {
        let catalog = catalog();
        let bytes = catalog.canonical_bytes().expect("catalog bytes");
        assert_eq!(
            AgentCognitionCatalogV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("catalog round trip"),
            catalog
        );

        let memory = AgentMemorySnapshotV1::initial(&catalog).expect("memory");
        let memory_bytes = memory.canonical_bytes().expect("memory bytes");
        assert_eq!(
            AgentMemorySnapshotV1::from_canonical_bytes(
                &memory_bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("memory round trip"),
            memory
        );

        let agent = AgentCognitionSnapshotV1::initial(catalog.subject_id, 17);
        let agent_bytes = agent.canonical_bytes().expect("agent bytes");
        assert_eq!(
            AgentCognitionSnapshotV1::from_canonical_bytes(
                &agent_bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("agent round trip"),
            agent
        );
    }

    #[test]
    fn memory_retrieval_is_bounded_and_canonical() {
        let catalog = catalog();
        let memory = AgentMemorySnapshotV1::initial(&catalog).expect("memory");
        let first = memory.retrieve(1).expect("retrieve");
        assert_eq!(first, catalog.seed_beliefs);
        assert_eq!(
            memory.retrieve(0),
            Err(CognitionContractError::LimitExceeded)
        );
    }
}
