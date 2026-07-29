use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{RpgAggregateKindV1, RpgContractErrorV1};

use crate::RpgAggregateKeyV1;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgStateError {
    Contract(RpgContractErrorV1),
    DuplicateAggregate(RpgAggregateKeyV1),
    DanglingAggregateReference,
    OwnerReferenceMismatch,
    InventoryCapacityExceeded,
    ItemHasMultipleInventories,
    EquippedItemNotInInventory,
}

impl RpgStateError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::DuplicateAggregate(_) => "RPG_DUPLICATE_AGGREGATE",
            Self::DanglingAggregateReference => "RPG_DANGLING_AGGREGATE_REFERENCE",
            Self::OwnerReferenceMismatch => "RPG_OWNERSHIP_CONFLICT",
            Self::InventoryCapacityExceeded => "RPG_INVENTORY_CAPACITY_EXCEEDED",
            Self::ItemHasMultipleInventories => "RPG_OWNERSHIP_CONFLICT",
            Self::EquippedItemNotInInventory => "RPG_OWNERSHIP_CONFLICT",
        }
    }
}

impl Display for RpgStateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for RpgStateError {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgPlanBuildError {
    Contract(RpgContractErrorV1),
    AggregateNotFound(RpgAggregateKindV1),
    RevisionStale {
        aggregate_kind: RpgAggregateKindV1,
        current_revision: u64,
    },
    RevisionExhausted,
    TransitionInvalid,
    OwnershipConflict,
    ReservationInvalid,
    PhysicalPreconditionMissing,
    DefinitionMismatch,
    CommitmentRejected,
    TransactionAborted,
}

impl RpgPlanBuildError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::AggregateNotFound(_) => "RPG_AGGREGATE_NOT_FOUND",
            Self::RevisionStale { .. } => "RPG_REVISION_STALE",
            Self::RevisionExhausted => "RPG_REVISION_EXHAUSTED",
            Self::TransitionInvalid => "RPG_TRANSITION_INVALID",
            Self::OwnershipConflict => "RPG_OWNERSHIP_CONFLICT",
            Self::ReservationInvalid => "RPG_RESERVATION_INVALID",
            Self::PhysicalPreconditionMissing => "RPG_PHYSICAL_PRECONDITION_MISSING",
            Self::DefinitionMismatch => "RPG_DEFINITION_MISMATCH",
            Self::CommitmentRejected => "RPG_COMMITMENT_REJECTED",
            Self::TransactionAborted => "RPG_TRANSACTION_ABORTED",
        }
    }
}

impl Display for RpgPlanBuildError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RevisionStale {
                aggregate_kind,
                current_revision,
            } => write!(
                formatter,
                "{}: {:?}@{}",
                self.stable_code(),
                aggregate_kind,
                current_revision
            ),
            Self::AggregateNotFound(kind) => {
                write!(formatter, "{}: {kind:?}", self.stable_code())
            }
            _ => formatter.write_str(self.stable_code()),
        }
    }
}

impl Error for RpgPlanBuildError {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgPlanMaterializeError {
    Contract(RpgContractErrorV1),
    State(RpgStateError),
    PlanStale,
    TransactionAborted,
}

impl RpgPlanMaterializeError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::State(error) => error.stable_code(),
            Self::PlanStale => "RPG_PLAN_STALE",
            Self::TransactionAborted => "RPG_TRANSACTION_ABORTED",
        }
    }
}

impl Display for RpgPlanMaterializeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for RpgPlanMaterializeError {}
