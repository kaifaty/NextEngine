use next_contracts::cognition::{AffordanceExecutionV1, COGNITION_Q16_ONE, SemanticAffordanceV1};
use next_contracts::rpg::CommitmentStateV1;
use next_contracts::world_activity::{
    WorldActivityCatalogV1, WorldActivitySnapshotV1, WorldActivityStateV1,
};

use super::{StrategicAgentError, StrategicObservationV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemicStrategicObservationV1<'a> {
    pub activity_catalog: &'a WorldActivityCatalogV1,
    pub activity_snapshot: &'a WorldActivitySnapshotV1,
    pub commitment_or_none: Option<SystemicCommitmentObservationV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemicCommitmentObservationV1 {
    pub revision: u64,
    pub state: CommitmentStateV1,
}

pub(super) fn validate_systemic_observation(
    observation: &StrategicObservationV1<'_>,
    systemic: SystemicStrategicObservationV1<'_>,
) -> Result<(), StrategicAgentError> {
    systemic
        .activity_catalog
        .validate()
        .map_err(|_| StrategicAgentError::ObservationInvalid)?;
    systemic
        .activity_snapshot
        .validate_against(systemic.activity_catalog, observation.gameplay_tick)
        .map_err(|_| StrategicAgentError::ObservationInvalid)?;
    let threat = &systemic.activity_catalog.systemic_work.threat_act;
    if systemic.activity_catalog.worker_subject_id != observation.catalog.subject_id
        || systemic.activity_snapshot.worker_subject_id != observation.catalog.subject_id
        || threat.listener_id != observation.catalog.subject_id
    {
        return Err(StrategicAgentError::ObservationInvalid);
    }
    Ok(())
}

pub(super) fn extend_systemic_affordances(
    observation: &StrategicObservationV1<'_>,
    systemic: SystemicStrategicObservationV1<'_>,
    affordances: &mut Vec<SemanticAffordanceV1>,
) -> Result<(), StrategicAgentError> {
    let activity = systemic.activity_snapshot;
    let catalog = systemic.activity_catalog;
    let profile = &catalog.systemic_work;
    let Some(commitment) = systemic.commitment_or_none else {
        return Ok(());
    };
    let exchange_tick = profile
        .work_exchange
        .acts
        .first()
        .ok_or(StrategicAgentError::ObservationInvalid)?
        .creation_tick;
    let (action_id, effect_fact_id, execution, owner_revision) = if activity.state
        == WorldActivityStateV1::Unassigned
        && commitment.state == CommitmentStateV1::Offered
        && observation.gameplay_tick == exchange_tick
    {
        (
            &profile.social_action_id,
            &profile.social_ready_fact_id,
            AffordanceExecutionV1::CommitSocialExchange,
            commitment.revision,
        )
    } else if commitment.state == CommitmentStateV1::Accepted
        && (activity.state == WorldActivityStateV1::Completed
            || activity.state == WorldActivityStateV1::Working
                && activity
                    .work_started_tick_or_none
                    .and_then(|tick| tick.checked_add(catalog.work_duration_ticks))
                    .is_some_and(|due| observation.gameplay_tick >= due))
    {
        (
            &profile.settlement_action_id,
            &profile.settlement_ready_fact_id,
            AffordanceExecutionV1::SettleSystemicExchange,
            if activity.state == WorldActivityStateV1::Completed {
                activity.record_revision
            } else {
                activity
                    .record_revision
                    .checked_add(1)
                    .ok_or(StrategicAgentError::RevisionExhausted)?
            },
        )
    } else if commitment.state == CommitmentStateV1::Accepted
        && matches!(
            activity.state,
            WorldActivityStateV1::Assigned | WorldActivityStateV1::Working
        )
    {
        (
            &profile.await_activity_action_id,
            &profile.activity_ready_fact_id,
            AffordanceExecutionV1::AwaitActivity,
            activity.record_revision,
        )
    } else {
        return Ok(());
    };
    affordances.push(SemanticAffordanceV1 {
        action_id: action_id.clone(),
        owner_revision,
        precondition_fact_ids: Vec::new(),
        effect_fact_ids: vec![effect_fact_id.clone()],
        cost_q16: u32::try_from(COGNITION_Q16_ONE / 16).expect("positive q16 cost"),
        execution,
        route_plan_hash_or_none: None,
        target_id_or_none: None,
    });
    Ok(())
}
