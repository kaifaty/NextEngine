use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BeliefSourceV1 {
    AuthoredSeed = 1,
    OwnerProjection = 2,
}

impl BeliefSourceV1 {
    pub(super) fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
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
    pub(super) fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
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
    pub(super) fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
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
    pub(super) fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
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
