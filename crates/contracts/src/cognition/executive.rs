use super::*;

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
    pub(super) fn from_tag(tag: u8) -> Result<Self, CognitionContractError> {
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
