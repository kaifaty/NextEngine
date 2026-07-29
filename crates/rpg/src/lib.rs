#![forbid(unsafe_code)]

mod error;
mod materialization;
mod operation;
mod planning;
mod state;

pub use error::{RpgPlanBuildError, RpgPlanMaterializeError, RpgStateError};
pub use materialization::{materialize_transaction_plan_v1, recheck_transaction_plan_v1};
pub use planning::{BuiltRpgTransactionPlanV1, RpgPlanningContextV1, build_transaction_plan_v1};
pub use state::{RpgAggregateKeyV1, RpgState};

#[cfg(test)]
mod tests;
