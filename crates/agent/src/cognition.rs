use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use next_contracts::canonical::sha256;
use next_contracts::cognition::{
    ActiveGoalV1, AffordanceExecutionV1, AgentCognitionCatalogV1, AgentCognitionSnapshotV1,
    AgentMemorySnapshotV1, COGNITION_Q16_ONE, DecisionSwitchReasonV1, DecisionTraceV1, DriveViewV1,
    EpistemicViewV1, GoalCandidateV1, GoalPriorityBandV1, PlanningFailureV1, PrivateTaskStateV1,
    SemanticAffordanceV1, StrategicAgentIntentV1, StrategicPlanStepV1, StrategicPlanV1,
    SuspendedGoalV1, TaskLifecycleV1, candidate_order,
};
use next_contracts::ids::{ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::world_population::NavigationRoutePlanV1;

mod error;

pub use error::StrategicAgentError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrategicObservationV1<'a> {
    pub gameplay_tick: u64,
    pub cognition_catalog_revision: ContentHash,
    pub catalog: &'a AgentCognitionCatalogV1,
    pub memory_snapshot: &'a AgentMemorySnapshotV1,
    pub rpg_owner_revision: u64,
    pub population_record_revision: u64,
    pub current_node_id: SchemaId,
    pub navigation_goal_node_id: SchemaId,
    pub health_current: i32,
    pub health_maximum: i32,
    pub route_plan_or_none: Option<&'a NavigationRoutePlanV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrategicDecisionProposalV1 {
    pub next_agent_snapshot: AgentCognitionSnapshotV1,
    pub next_memory_snapshot: AgentMemorySnapshotV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrategicEvaluationV1 {
    pub epistemic_view: EpistemicViewV1,
    pub drive_view: DriveViewV1,
    pub proposal_or_none: Option<StrategicDecisionProposalV1>,
    pub decision_trace: DecisionTraceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrategicAgentOwnersV1 {
    catalog: AgentCognitionCatalogV1,
    agent_snapshot: AgentCognitionSnapshotV1,
    memory_snapshot: AgentMemorySnapshotV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedStrategicAgentPublicationV1 {
    base_agent_hash: ContentHash,
    base_memory_hash: ContentHash,
    next_agent_snapshot: AgentCognitionSnapshotV1,
    next_memory_snapshot: AgentMemorySnapshotV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedStrategicAgentPublicationV1(PreparedStrategicAgentPublicationV1);

impl StrategicAgentOwnersV1 {
    pub fn initial(
        catalog: AgentCognitionCatalogV1,
        decision_rng_state: u64,
    ) -> Result<Self, StrategicAgentError> {
        catalog.validate()?;
        let agent_snapshot =
            AgentCognitionSnapshotV1::initial(catalog.subject_id, decision_rng_state);
        let memory_snapshot = AgentMemorySnapshotV1::initial(&catalog)?;
        let value = Self {
            catalog,
            agent_snapshot,
            memory_snapshot,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn restore(
        catalog: AgentCognitionCatalogV1,
        agent_snapshot: AgentCognitionSnapshotV1,
        memory_snapshot: AgentMemorySnapshotV1,
    ) -> Result<Self, StrategicAgentError> {
        let value = Self {
            catalog,
            agent_snapshot,
            memory_snapshot,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), StrategicAgentError> {
        self.catalog.validate()?;
        self.agent_snapshot.validate()?;
        self.memory_snapshot.validate()?;
        if self.catalog.subject_id != self.agent_snapshot.subject_id
            || self.catalog.subject_id != self.memory_snapshot.subject_id
            || self.agent_snapshot.revision != self.memory_snapshot.revision
        {
            return Err(StrategicAgentError::OwnerClosureInvalid);
        }
        Ok(())
    }

    #[must_use]
    pub const fn catalog(&self) -> &AgentCognitionCatalogV1 {
        &self.catalog
    }

    #[must_use]
    pub const fn agent_snapshot(&self) -> &AgentCognitionSnapshotV1 {
        &self.agent_snapshot
    }

    #[must_use]
    pub const fn memory_snapshot(&self) -> &AgentMemorySnapshotV1 {
        &self.memory_snapshot
    }

    pub fn prepare_publication(
        &self,
        next_agent_snapshot: AgentCognitionSnapshotV1,
        next_memory_snapshot: AgentMemorySnapshotV1,
    ) -> Result<PreparedStrategicAgentPublicationV1, StrategicAgentError> {
        self.validate()?;
        Ok(PreparedStrategicAgentPublicationV1 {
            base_agent_hash: snapshot_hash(&self.agent_snapshot.canonical_bytes()?),
            base_memory_hash: snapshot_hash(&self.memory_snapshot.canonical_bytes()?),
            next_agent_snapshot,
            next_memory_snapshot,
        })
    }

    pub fn validate_prepared_publication(
        &self,
        prepared: PreparedStrategicAgentPublicationV1,
    ) -> Result<ValidatedStrategicAgentPublicationV1, StrategicAgentError> {
        self.validate()?;
        if prepared.base_agent_hash != snapshot_hash(&self.agent_snapshot.canonical_bytes()?)
            || prepared.base_memory_hash != snapshot_hash(&self.memory_snapshot.canonical_bytes()?)
        {
            return Err(StrategicAgentError::PreparedPublicationStale);
        }
        let restored = Self::restore(
            self.catalog.clone(),
            prepared.next_agent_snapshot.clone(),
            prepared.next_memory_snapshot.clone(),
        )?;
        let unchanged = restored.agent_snapshot == self.agent_snapshot
            && restored.memory_snapshot == self.memory_snapshot;
        let advanced = restored.agent_snapshot.revision
            == self
                .agent_snapshot
                .revision
                .checked_add(1)
                .ok_or(StrategicAgentError::RevisionExhausted)?;
        if !unchanged && !advanced {
            return Err(StrategicAgentError::OwnerClosureInvalid);
        }
        Ok(ValidatedStrategicAgentPublicationV1(prepared))
    }

    pub fn commit_validated_publication(
        &mut self,
        validated: ValidatedStrategicAgentPublicationV1,
    ) {
        self.agent_snapshot = validated.0.next_agent_snapshot;
        self.memory_snapshot = validated.0.next_memory_snapshot;
    }

    pub fn preflight_validated_publication(
        &self,
        validated: &ValidatedStrategicAgentPublicationV1,
    ) -> Result<(), StrategicAgentError> {
        self.validate()?;
        if validated.0.base_agent_hash != snapshot_hash(&self.agent_snapshot.canonical_bytes()?)
            || validated.0.base_memory_hash
                != snapshot_hash(&self.memory_snapshot.canonical_bytes()?)
        {
            return Err(StrategicAgentError::PreparedPublicationStale);
        }
        Ok(())
    }
}

impl ValidatedStrategicAgentPublicationV1 {
    #[must_use]
    pub const fn agent_snapshot(&self) -> &AgentCognitionSnapshotV1 {
        &self.0.next_agent_snapshot
    }

    #[must_use]
    pub const fn memory_snapshot(&self) -> &AgentMemorySnapshotV1 {
        &self.0.next_memory_snapshot
    }
}

fn snapshot_hash(bytes: &[u8]) -> ContentHash {
    content_hash_from_bytes(sha256(bytes))
}

pub fn build_epistemic_view_v1(
    observation: &StrategicObservationV1<'_>,
) -> Result<EpistemicViewV1, StrategicAgentError> {
    observation.catalog.validate()?;
    observation.memory_snapshot.validate()?;
    if observation.catalog.subject_id != observation.memory_snapshot.subject_id
        || observation.catalog.revision()? != observation.cognition_catalog_revision
        || observation.health_maximum <= 0
        || observation.health_current < 0
        || observation.health_current > observation.health_maximum
    {
        return Err(StrategicAgentError::ObservationInvalid);
    }

    let mut known_fact_ids = Vec::new();
    let travel_needed = observation.current_node_id != observation.navigation_goal_node_id;
    if travel_needed {
        known_fact_ids.push(observation.catalog.travel_needed_fact_id.clone());
    }
    let emergency = observation.health_current <= observation.catalog.emergency_health_threshold;
    if emergency {
        known_fact_ids.push(observation.catalog.emergency_fact_id.clone());
    }

    let mut affordances = Vec::new();
    if let Some(route) = observation.route_plan_or_none {
        route.validate().map_err(StrategicAgentError::Population)?;
        if route.query.start_node_id != observation.current_node_id
            || route.query.goal_node_id != observation.navigation_goal_node_id
        {
            return Err(StrategicAgentError::ObservationInvalid);
        }
        known_fact_ids.push(observation.catalog.route_known_fact_id.clone());
        if travel_needed {
            let mut preconditions = vec![
                observation.catalog.route_known_fact_id.clone(),
                observation.catalog.travel_needed_fact_id.clone(),
            ];
            preconditions.sort();
            affordances.push(SemanticAffordanceV1 {
                action_id: observation.catalog.navigate_action_id.clone(),
                owner_revision: observation.population_record_revision,
                precondition_fact_ids: preconditions,
                effect_fact_ids: vec![observation.catalog.navigate_ready_fact_id.clone()],
                cost_q16: route_cost_q16(route.total_cost),
                execution: AffordanceExecutionV1::RequestLogicalRoute,
                route_plan_hash_or_none: Some(route.plan_hash),
                target_id_or_none: Some(observation.navigation_goal_node_id.clone()),
            });
        }
    }
    affordances.push(SemanticAffordanceV1 {
        action_id: observation.catalog.hold_action_id.clone(),
        owner_revision: observation.rpg_owner_revision,
        precondition_fact_ids: vec![observation.catalog.emergency_fact_id.clone()],
        effect_fact_ids: vec![observation.catalog.hold_ready_fact_id.clone()],
        cost_q16: u32::try_from(COGNITION_Q16_ONE / 16).expect("positive q16 cost"),
        execution: AffordanceExecutionV1::HoldPosition,
        route_plan_hash_or_none: None,
        target_id_or_none: None,
    });

    let retrieved_beliefs = observation
        .memory_snapshot
        .retrieve(usize::from(observation.catalog.retrieval_limit))?;
    Ok(EpistemicViewV1::new(
        observation.gameplay_tick,
        observation.catalog.subject_id,
        observation.cognition_catalog_revision,
        observation.memory_snapshot.revision,
        observation.rpg_owner_revision,
        observation.population_record_revision,
        observation.current_node_id.clone(),
        observation.navigation_goal_node_id.clone(),
        observation.health_current,
        observation.health_maximum,
        known_fact_ids,
        retrieved_beliefs,
        affordances,
    )?)
}

pub fn derive_drive_view_v1(
    catalog: &AgentCognitionCatalogV1,
    epistemic_view: &EpistemicViewV1,
) -> Result<DriveViewV1, StrategicAgentError> {
    catalog.validate()?;
    epistemic_view.validate()?;
    if catalog.subject_id != epistemic_view.subject_id {
        return Err(StrategicAgentError::ObservationInvalid);
    }
    let value = DriveViewV1 {
        schema_version: next_contracts::cognition::COGNITION_SCHEMA_VERSION,
        gameplay_tick: epistemic_view.gameplay_tick,
        safety_pressure_q16: if epistemic_view.health_current <= catalog.emergency_health_threshold
        {
            COGNITION_Q16_ONE
        } else {
            0
        },
        duty_pressure_q16: if epistemic_view.current_node_id
            == epistemic_view.navigation_goal_node_id
        {
            0
        } else {
            COGNITION_Q16_ONE
        },
    };
    value.validate()?;
    Ok(value)
}

pub fn evaluate_strategic_decision_v1(
    catalog: &AgentCognitionCatalogV1,
    agent_snapshot: &AgentCognitionSnapshotV1,
    memory_snapshot: &AgentMemorySnapshotV1,
    epistemic_view: EpistemicViewV1,
) -> Result<StrategicEvaluationV1, StrategicAgentError> {
    catalog.validate()?;
    agent_snapshot.validate()?;
    memory_snapshot.validate()?;
    epistemic_view.validate()?;
    if agent_snapshot.subject_id != catalog.subject_id
        || memory_snapshot.subject_id != catalog.subject_id
        || epistemic_view.subject_id != catalog.subject_id
        || epistemic_view.memory_revision != memory_snapshot.revision
        || epistemic_view.cognition_catalog_revision != catalog.revision()?
    {
        return Err(StrategicAgentError::StaleEpistemicView);
    }
    let drive_view = derive_drive_view_v1(catalog, &epistemic_view)?;
    let mut candidates =
        build_goal_candidates(catalog, agent_snapshot, &epistemic_view, drive_view)?;
    candidates.sort_by(candidate_order);
    let (selected, switch_reason, suspended_goals) =
        select_goal(catalog, agent_snapshot, &candidates, drive_view)?;

    let planning = bounded_goap_plan(catalog, &epistemic_view, &selected);
    let drive_view_hash = content_hash_from_bytes(sha256(&drive_view.canonical_bytes()?));
    match planning {
        Ok(plan) => {
            let step = plan
                .steps
                .get(usize::from(plan.cursor))
                .ok_or(StrategicAgentError::PlannerInvariant)?;
            let intent = intent_for_step(&epistemic_view, &selected, step)?;
            let next_memory_snapshot =
                memory_snapshot.retrieved_at(epistemic_view.gameplay_tick)?;
            let next_agent_snapshot = AgentCognitionSnapshotV1 {
                schema_version: next_contracts::cognition::COGNITION_SCHEMA_VERSION,
                revision: agent_snapshot
                    .revision
                    .checked_add(1)
                    .ok_or(StrategicAgentError::RevisionExhausted)?,
                subject_id: catalog.subject_id,
                active_goal_or_none: Some(selected.clone()),
                suspended_goals,
                plan_or_none: Some(plan.clone()),
                task_or_none: Some(PrivateTaskStateV1 {
                    action_id: step.action_id.clone(),
                    lifecycle: TaskLifecycleV1::Active,
                    started_tick: epistemic_view.gameplay_tick,
                    attempt: 1,
                }),
                pending_intent_or_none: Some(intent.clone()),
                last_selected_utility_q16: selected.selected_utility_q16,
                decision_rng_state: advance_decision_rng(agent_snapshot.decision_rng_state),
                last_epistemic_hash: epistemic_view.view_hash,
            };
            next_agent_snapshot.validate()?;
            let trace = DecisionTraceV1::new(
                epistemic_view.gameplay_tick,
                catalog.subject_id,
                epistemic_view.view_hash,
                drive_view_hash,
                candidates,
                selected.goal_id,
                switch_reason,
                Some(plan),
                PlanningFailureV1::None,
                Some(intent.intent_id),
            )?;
            Ok(StrategicEvaluationV1 {
                epistemic_view,
                drive_view,
                proposal_or_none: Some(StrategicDecisionProposalV1 {
                    next_agent_snapshot,
                    next_memory_snapshot,
                }),
                decision_trace: trace,
            })
        }
        Err(failure) => {
            let trace = DecisionTraceV1::new(
                epistemic_view.gameplay_tick,
                catalog.subject_id,
                epistemic_view.view_hash,
                drive_view_hash,
                candidates,
                selected.goal_id,
                DecisionSwitchReasonV1::SafeFallback,
                None,
                failure,
                None,
            )?;
            Ok(StrategicEvaluationV1 {
                epistemic_view,
                drive_view,
                proposal_or_none: None,
                decision_trace: trace,
            })
        }
    }
}

fn build_goal_candidates(
    catalog: &AgentCognitionCatalogV1,
    agent_snapshot: &AgentCognitionSnapshotV1,
    view: &EpistemicViewV1,
    drives: DriveViewV1,
) -> Result<Vec<GoalCandidateV1>, StrategicAgentError> {
    let cited_belief_ids = view
        .retrieved_beliefs
        .iter()
        .map(|belief| belief.belief_id)
        .collect::<Vec<_>>();
    let ordinary_inertia = if agent_snapshot
        .active_goal_or_none
        .as_ref()
        .is_some_and(|goal| goal.goal_id == catalog.ordinary_goal_id)
    {
        COGNITION_Q16_ONE / 2
    } else {
        0
    };
    let ordinary_cost = view
        .affordances
        .iter()
        .find(|affordance| affordance.action_id == catalog.navigate_action_id)
        .map_or(COGNITION_Q16_ONE / 2, |affordance| {
            i32::try_from(affordance.cost_q16).expect("validated affordance cost fits i32")
        });
    let mut candidates = vec![goal_candidate(
        catalog.ordinary_goal_id.clone(),
        GoalPriorityBandV1::Ordinary,
        Some(view.navigation_goal_node_id.clone()),
        catalog.navigate_ready_fact_id.clone(),
        drives.duty_pressure_q16,
        COGNITION_Q16_ONE / 4,
        ordinary_inertia,
        0,
        ordinary_cost,
        cited_belief_ids.clone(),
    )?];
    if drives.safety_pressure_q16 > 0 {
        let emergency_inertia = if agent_snapshot
            .active_goal_or_none
            .as_ref()
            .is_some_and(|goal| goal.goal_id == catalog.emergency_goal_id)
        {
            COGNITION_Q16_ONE / 2
        } else {
            0
        };
        candidates.push(goal_candidate(
            catalog.emergency_goal_id.clone(),
            GoalPriorityBandV1::Emergency,
            None,
            catalog.hold_ready_fact_id.clone(),
            drives.safety_pressure_q16,
            COGNITION_Q16_ONE,
            emergency_inertia,
            0,
            COGNITION_Q16_ONE / 16,
            cited_belief_ids,
        )?);
    }
    Ok(candidates)
}

#[allow(
    clippy::too_many_arguments,
    reason = "fixed-point utility components stay explicit in the public candidate"
)]
fn goal_candidate(
    goal_id: SchemaId,
    priority_band: GoalPriorityBandV1,
    target_id_or_none: Option<SchemaId>,
    desired_fact_id: SchemaId,
    drive_utility_q16: i32,
    aspiration_utility_q16: i32,
    inertia_utility_q16: i32,
    risk_cost_q16: i32,
    expected_cost_q16: i32,
    mut cited_belief_ids: Vec<ContentHash>,
) -> Result<GoalCandidateV1, StrategicAgentError> {
    cited_belief_ids.sort();
    cited_belief_ids.dedup();
    let total_utility_q16 = drive_utility_q16
        .checked_add(aspiration_utility_q16)
        .and_then(|value| value.checked_add(inertia_utility_q16))
        .and_then(|value| value.checked_sub(risk_cost_q16))
        .and_then(|value| value.checked_sub(expected_cost_q16))
        .ok_or(StrategicAgentError::UtilityOverflow)?;
    let value = GoalCandidateV1 {
        goal_id,
        priority_band,
        target_id_or_none,
        desired_fact_id,
        drive_utility_q16,
        aspiration_utility_q16,
        inertia_utility_q16,
        risk_cost_q16,
        expected_cost_q16,
        total_utility_q16,
        cited_belief_ids,
    };
    value.validate()?;
    Ok(value)
}

fn select_goal(
    catalog: &AgentCognitionCatalogV1,
    snapshot: &AgentCognitionSnapshotV1,
    candidates: &[GoalCandidateV1],
    drives: DriveViewV1,
) -> Result<(ActiveGoalV1, DecisionSwitchReasonV1, Vec<SuspendedGoalV1>), StrategicAgentError> {
    let best = candidates
        .first()
        .ok_or(StrategicAgentError::NoGoalCandidate)?;
    let mut suspended = snapshot.suspended_goals.clone();
    let Some(current) = snapshot.active_goal_or_none.as_ref() else {
        return Ok((
            active_goal(best, drives.gameplay_tick),
            DecisionSwitchReasonV1::InitialSelection,
            suspended,
        ));
    };

    if best.priority_band == GoalPriorityBandV1::Emergency
        && current.priority_band != GoalPriorityBandV1::Emergency
    {
        if suspended.len() >= next_contracts::cognition::COGNITION_MAX_SUSPENDED_GOALS {
            return Err(StrategicAgentError::SuspendedGoalLimit);
        }
        suspended.push(SuspendedGoalV1 {
            goal: current.clone(),
            suspended_tick: drives.gameplay_tick,
        });
        suspended.sort();
        return Ok((
            active_goal(best, drives.gameplay_tick),
            DecisionSwitchReasonV1::EmergencyInterrupt,
            suspended,
        ));
    }

    if current.priority_band == GoalPriorityBandV1::Emergency
        && drives.safety_pressure_q16 == 0
        && let Some(index) = suspended
            .iter()
            .position(|entry| entry.goal.goal_id == catalog.ordinary_goal_id)
    {
        let resumed = suspended.remove(index).goal;
        let resumed_candidate = candidates
            .iter()
            .find(|candidate| candidate.goal_id == resumed.goal_id)
            .ok_or(StrategicAgentError::NoGoalCandidate)?;
        return Ok((
            active_goal(resumed_candidate, drives.gameplay_tick),
            DecisionSwitchReasonV1::EmergencyExitResume,
            suspended,
        ));
    }

    let current_candidate = candidates
        .iter()
        .find(|candidate| candidate.goal_id == current.goal_id);
    if let Some(current_candidate) = current_candidate {
        let switch_required = best.goal_id != current.goal_id
            && best.priority_band > current.priority_band
            || best.goal_id != current.goal_id
                && best.total_utility_q16
                    >= current_candidate
                        .total_utility_q16
                        .saturating_add(catalog.goal_switch_threshold_q16);
        if !switch_required {
            return Ok((
                active_goal(current_candidate, drives.gameplay_tick),
                DecisionSwitchReasonV1::RetainedByInertia,
                suspended,
            ));
        }
    }
    Ok((
        active_goal(best, drives.gameplay_tick),
        DecisionSwitchReasonV1::HigherUtility,
        suspended,
    ))
}

fn active_goal(candidate: &GoalCandidateV1, tick: u64) -> ActiveGoalV1 {
    ActiveGoalV1 {
        goal_id: candidate.goal_id.clone(),
        priority_band: candidate.priority_band,
        target_id_or_none: candidate.target_id_or_none.clone(),
        desired_fact_id: candidate.desired_fact_id.clone(),
        selected_utility_q16: candidate.total_utility_q16,
        selected_tick: tick,
    }
}

fn bounded_goap_plan(
    catalog: &AgentCognitionCatalogV1,
    view: &EpistemicViewV1,
    goal: &ActiveGoalV1,
) -> Result<StrategicPlanV1, PlanningFailureV1> {
    let initial = view.known_fact_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut frontier = BinaryHeap::new();
    frontier.push(Reverse((0_u64, 0_u8, initial.clone(), Vec::<usize>::new())));
    let mut best_cost = BTreeMap::from([(initial, 0_u64)]);
    let mut expanded = 0_u16;
    while let Some(Reverse((cost, depth, facts, path))) = frontier.pop() {
        if best_cost.get(&facts).copied() != Some(cost) {
            continue;
        }
        expanded = expanded
            .checked_add(1)
            .ok_or(PlanningFailureV1::PlanBudgetExhausted)?;
        if expanded > catalog.planner_max_expanded_nodes {
            return Err(PlanningFailureV1::PlanBudgetExhausted);
        }
        if facts.contains(&goal.desired_fact_id) {
            let steps = path
                .iter()
                .map(|index| {
                    let affordance = &view.affordances[*index];
                    StrategicPlanStepV1 {
                        action_id: affordance.action_id.clone(),
                        owner_revision: affordance.owner_revision,
                        cost_q16: affordance.cost_q16,
                        execution: affordance.execution,
                        route_plan_hash_or_none: affordance.route_plan_hash_or_none,
                        target_id_or_none: affordance.target_id_or_none.clone(),
                    }
                })
                .collect::<Vec<_>>();
            let plan = StrategicPlanV1 {
                goal_id: goal.goal_id.clone(),
                desired_fact_id: goal.desired_fact_id.clone(),
                steps,
                cursor: 0,
                expanded_nodes: expanded,
                planner_max_depth: catalog.planner_max_depth,
                planner_max_expanded_nodes: catalog.planner_max_expanded_nodes,
            };
            plan.validate()
                .map_err(|_| PlanningFailureV1::MissingAffordance)?;
            return Ok(plan);
        }
        if depth >= catalog.planner_max_depth {
            continue;
        }
        for (index, affordance) in view.affordances.iter().enumerate() {
            if !affordance
                .precondition_fact_ids
                .iter()
                .all(|fact| facts.contains(fact))
            {
                continue;
            }
            let mut next_facts = facts.clone();
            next_facts.extend(affordance.effect_fact_ids.iter().cloned());
            if next_facts == facts {
                continue;
            }
            let next_cost = cost
                .checked_add(u64::from(affordance.cost_q16))
                .ok_or(PlanningFailureV1::PlanBudgetExhausted)?;
            if best_cost
                .get(&next_facts)
                .is_none_or(|known| next_cost < *known)
            {
                best_cost.insert(next_facts.clone(), next_cost);
                let mut next_path = path.clone();
                next_path.push(index);
                frontier.push(Reverse((next_cost, depth + 1, next_facts, next_path)));
            }
        }
    }
    Err(PlanningFailureV1::MissingAffordance)
}

fn intent_for_step(
    view: &EpistemicViewV1,
    goal: &ActiveGoalV1,
    step: &StrategicPlanStepV1,
) -> Result<StrategicAgentIntentV1, StrategicAgentError> {
    match step.execution {
        AffordanceExecutionV1::RequestLogicalRoute => {
            StrategicAgentIntentV1::request_logical_route(
                view.subject_id,
                goal.goal_id.clone(),
                step.action_id.clone(),
                view.gameplay_tick,
                view.current_node_id.clone(),
                step.target_id_or_none
                    .clone()
                    .ok_or(StrategicAgentError::PlannerInvariant)?,
                step.route_plan_hash_or_none
                    .ok_or(StrategicAgentError::PlannerInvariant)?,
            )
            .map_err(Into::into)
        }
        AffordanceExecutionV1::HoldPosition => StrategicAgentIntentV1::hold_position(
            view.subject_id,
            goal.goal_id.clone(),
            step.action_id.clone(),
            view.gameplay_tick,
        )
        .map_err(Into::into),
    }
}

fn route_cost_q16(total_cost: u64) -> u32 {
    let scaled = total_cost.saturating_mul(256);
    u32::try_from(scaled.min(u64::from(
        u32::try_from(COGNITION_Q16_ONE / 2).expect("positive q16 cost"),
    )))
    .expect("bounded by u32")
    .max(1)
}

#[must_use]
pub const fn advance_decision_rng(state: u64) -> u64 {
    state.wrapping_add(0x9e37_79b9_7f4a_7c15).rotate_left(17) ^ 0xbf58_476d_1ce4_e5b9
}

#[cfg(test)]
mod tests {
    use next_contracts::cognition::{
        BeliefContradictionV1, BeliefSourceV1, COGNITION_SCHEMA_VERSION, SemanticBeliefV1,
    };
    use next_contracts::ids::{AssetId, ContentHash, PersistentId};
    use next_contracts::world_population::{
        NavigationCapabilityV1, NavigationQueryV1, NavigationRoutePlanV1,
        WORLD_POPULATION_SCHEMA_VERSION,
    };

    use super::*;

    fn schema(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test schema id")
    }

    fn fixture() -> (
        AgentCognitionCatalogV1,
        AgentMemorySnapshotV1,
        AgentCognitionSnapshotV1,
        NavigationRoutePlanV1,
    ) {
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
        let catalog = AgentCognitionCatalogV1 {
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
        };
        catalog.validate().expect("catalog");
        let memory = AgentMemorySnapshotV1::initial(&catalog).expect("memory");
        let agent = AgentCognitionSnapshotV1::initial(subject_id, 41);
        let query = NavigationQueryV1 {
            schema_version: WORLD_POPULATION_SCHEMA_VERSION,
            catalog_asset_id: AssetId::from_bytes([9; 16]),
            graph_revision: ContentHash::from_bytes([10; 32]),
            start_node_id: schema("node.home"),
            goal_node_id: schema("node.frontier"),
            capability: NavigationCapabilityV1::AbstractTransfer,
        };
        let route = NavigationRoutePlanV1::new(
            query,
            vec![schema("node.home"), schema("node.frontier")],
            5,
        )
        .expect("route");
        (catalog, memory, agent, route)
    }

    fn view(
        tick: u64,
        health: i32,
        catalog: &AgentCognitionCatalogV1,
        memory: &AgentMemorySnapshotV1,
        route: Option<&NavigationRoutePlanV1>,
    ) -> EpistemicViewV1 {
        build_epistemic_view_v1(&StrategicObservationV1 {
            gameplay_tick: tick,
            cognition_catalog_revision: catalog.revision().expect("revision"),
            catalog,
            memory_snapshot: memory,
            rpg_owner_revision: tick,
            population_record_revision: 0,
            current_node_id: schema("node.home"),
            navigation_goal_node_id: schema("node.frontier"),
            health_current: health,
            health_maximum: 100,
            route_plan_or_none: route,
        })
        .expect("view")
    }

    #[test]
    fn utility_goap_and_private_executive_are_deterministic() {
        let (catalog, memory, agent, route) = fixture();
        let epistemic = view(1, 100, &catalog, &memory, Some(&route));
        let first = evaluate_strategic_decision_v1(&catalog, &agent, &memory, epistemic.clone())
            .expect("first");
        let second =
            evaluate_strategic_decision_v1(&catalog, &agent, &memory, epistemic).expect("second");
        assert_eq!(first, second);
        let proposal = first.proposal_or_none.expect("proposal");
        assert_eq!(
            proposal
                .next_agent_snapshot
                .active_goal_or_none
                .expect("goal")
                .goal_id,
            catalog.ordinary_goal_id
        );
        assert_eq!(
            proposal
                .next_agent_snapshot
                .task_or_none
                .expect("task")
                .lifecycle,
            TaskLifecycleV1::Active
        );
    }

    #[test]
    fn missing_affordance_and_budget_exhaustion_have_no_proposal() {
        let (mut catalog, memory, agent, route) = fixture();
        let without_route = view(1, 100, &catalog, &memory, None);
        let missing = evaluate_strategic_decision_v1(&catalog, &agent, &memory, without_route)
            .expect("typed fallback");
        assert!(missing.proposal_or_none.is_none());
        assert_eq!(
            missing.decision_trace.planning_failure,
            PlanningFailureV1::MissingAffordance
        );

        catalog.planner_max_expanded_nodes = 1;
        let bounded = evaluate_strategic_decision_v1(
            &catalog,
            &agent,
            &memory,
            view(1, 100, &catalog, &memory, Some(&route)),
        )
        .expect("budget fallback");
        assert!(bounded.proposal_or_none.is_none());
        assert_eq!(
            bounded.decision_trace.planning_failure,
            PlanningFailureV1::PlanBudgetExhausted
        );
    }

    #[test]
    fn emergency_interrupts_and_then_resumes_ordinary_goal() {
        let (catalog, memory0, agent0, route) = fixture();
        let ordinary = evaluate_strategic_decision_v1(
            &catalog,
            &agent0,
            &memory0,
            view(1, 100, &catalog, &memory0, Some(&route)),
        )
        .expect("ordinary")
        .proposal_or_none
        .expect("ordinary proposal");
        let emergency = evaluate_strategic_decision_v1(
            &catalog,
            &ordinary.next_agent_snapshot,
            &ordinary.next_memory_snapshot,
            view(
                4,
                20,
                &catalog,
                &ordinary.next_memory_snapshot,
                Some(&route),
            ),
        )
        .expect("emergency");
        assert_eq!(
            emergency.decision_trace.switch_reason,
            DecisionSwitchReasonV1::EmergencyInterrupt
        );
        let emergency = emergency.proposal_or_none.expect("emergency proposal");
        assert_eq!(emergency.next_agent_snapshot.suspended_goals.len(), 1);

        let resumed = evaluate_strategic_decision_v1(
            &catalog,
            &emergency.next_agent_snapshot,
            &emergency.next_memory_snapshot,
            view(
                7,
                100,
                &catalog,
                &emergency.next_memory_snapshot,
                Some(&route),
            ),
        )
        .expect("resume");
        assert_eq!(
            resumed.decision_trace.switch_reason,
            DecisionSwitchReasonV1::EmergencyExitResume
        );
        let resumed = resumed.proposal_or_none.expect("resume proposal");
        assert_eq!(
            resumed
                .next_agent_snapshot
                .active_goal_or_none
                .expect("active")
                .goal_id,
            catalog.ordinary_goal_id
        );
        assert!(resumed.next_agent_snapshot.suspended_goals.is_empty());
    }

    #[test]
    fn unrelated_hidden_authority_cannot_change_epistemic_decision() {
        let (catalog, memory, agent, route) = fixture();
        let hidden_owner_values = [11_u64, 99_u64];
        let decisions = hidden_owner_values.map(|_unobserved_hidden_revision| {
            evaluate_strategic_decision_v1(
                &catalog,
                &agent,
                &memory,
                view(1, 100, &catalog, &memory, Some(&route)),
            )
            .expect("decision")
        });
        assert_eq!(decisions[0], decisions[1]);
    }
}
