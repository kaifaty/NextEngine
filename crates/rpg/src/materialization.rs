use std::sync::Arc;

use crate::{BuiltRpgTransactionPlanV1, RpgAggregateKeyV1, RpgPlanMaterializeError, RpgState};

pub fn recheck_transaction_plan_v1(
    current: &RpgState,
    plan: &BuiltRpgTransactionPlanV1,
) -> Result<(), RpgPlanMaterializeError> {
    plan.0
        .validate()
        .map_err(RpgPlanMaterializeError::Contract)?;
    for read in &plan.0.ordered_read_set {
        let aggregate = current
            .aggregate(
                read.aggregate_ref.aggregate_kind,
                read.aggregate_ref.persistent_id,
            )
            .ok_or(RpgPlanMaterializeError::PlanStale)?;
        if aggregate.revision != read.aggregate_ref.expected_revision
            || aggregate
                .state_hash()
                .map_err(|_| RpgPlanMaterializeError::TransactionAborted)?
                != read.state_hash
        {
            return Err(RpgPlanMaterializeError::PlanStale);
        }
    }
    Ok(())
}

pub fn materialize_transaction_plan_v1(
    current: &RpgState,
    plan: &BuiltRpgTransactionPlanV1,
) -> Result<RpgState, RpgPlanMaterializeError> {
    recheck_transaction_plan_v1(current, plan)?;
    let mut next = current.clone();
    for write in &plan.0.ordered_write_set {
        let key = RpgAggregateKeyV1::new(write.aggregate_kind, write.persistent_id);
        next.aggregates.insert(key, Arc::new(write.after.clone()));
    }
    next.validate_cross_references()
        .map_err(RpgPlanMaterializeError::State)?;
    Ok(next)
}
