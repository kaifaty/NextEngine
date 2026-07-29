use std::cmp::Ordering;

use next_contracts::{
    ClosedCommandAdmissionBatchV2, ClosedIngressBatchV1, ClosedPhysicsContactBatchV1, CommandId,
    CommandPhase, ContentHash, DomainEvent, InputMappingReceiptV1, PhysicsCanonicalSnapshotV2,
    PhysicsStepInputV2, RpgSnapshotV2, RuntimeSnapshot, WorldCommand,
};

use crate::registry::CommandKindRegistry;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct CommandOrderKey {
    target_tick: u64,
    phase: CommandPhase,
    pub(super) priority_class: u16,
    issuer_tag: u8,
    issuer_id_bytes: Vec<u8>,
    sequence: u64,
    command_id: CommandId,
}

impl CommandOrderKey {
    pub(super) fn new(command: &WorldCommand, priority_class: u16) -> Self {
        Self {
            target_tick: command.target_tick,
            phase: command.phase,
            priority_class,
            issuer_tag: command.issuer.tag(),
            issuer_id_bytes: command.issuer.identifier_bytes().to_vec(),
            sequence: command.sequence,
            command_id: command.compute_command_id().unwrap_or_default(),
        }
    }

    pub(super) fn from_command(command: &WorldCommand, registry: &CommandKindRegistry) -> Self {
        let priority_class = registry
            .descriptor(&command.payload_schema_id, command.payload_schema_version)
            .filter(|descriptor| descriptor.accepts_payload(&command.payload))
            .map_or(u16::MAX, |descriptor| descriptor.priority_class());
        Self::new(command, priority_class)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RejectionCode {
    SchemaMismatch,
    CanonicalCommandInvalid,
    CommandIdMismatch,
    CommandPhaseForbidden,
    InternalOutcomeIssuerRequired,
    IssuerUnauthenticated,
    CommandStreamUnbound,
    CapabilityRequired,
    CapabilityDenied,
    PreconditionFailed,
    TargetNotAllowed,
    PhysicalTargetUnbound,
    PhysicalBodyInactive,
    PhysicalIntentAlreadyAssigned,
    CommandExpired,
    CommandFutureLimit,
    CommandSequenceCollision,
    CommandIdCollision,
    CommandSequenceFinalized,
    CommandSequenceExhausted,
    CommandStreamTimeRegression,
    CommandPendingLimit,
    CommandCollisionLocked,
    CommandStreamClosed,
    RpgAggregateNotFound,
    RpgSchemaUnsupported,
    RpgRevisionStale,
    RpgRevisionExhausted,
    RpgOperationOrderInvalid,
    RpgTransitionInvalid,
    RpgOwnershipConflict,
    RpgReservationInvalid,
    RpgPhysicalPreconditionMissing,
    RpgDefinitionMismatch,
    RpgPlanStale,
    RpgEventOrderInvalid,
    RpgTransactionAborted,
    RpgCommitmentRejected,
}

impl RejectionCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SchemaMismatch => "COMMAND_SCHEMA_MISMATCH",
            Self::CanonicalCommandInvalid => "CANONICAL_COMMAND_INVALID",
            Self::CommandIdMismatch => "COMMAND_ID_MISMATCH",
            Self::CommandPhaseForbidden => "COMMAND_PHASE_FORBIDDEN",
            Self::InternalOutcomeIssuerRequired => "INTERNAL_OUTCOME_ISSUER_REQUIRED",
            Self::IssuerUnauthenticated => "ISSUER_UNAUTHENTICATED",
            Self::CommandStreamUnbound => "COMMAND_STREAM_UNBOUND",
            Self::CapabilityRequired => "CAPABILITY_REQUIRED",
            Self::CapabilityDenied => "CAPABILITY_DENIED",
            Self::PreconditionFailed => "PRECONDITION_FAILED",
            Self::TargetNotAllowed => "COMMAND_TARGET_NOT_ALLOWED",
            Self::PhysicalTargetUnbound => "PHYSICAL_TARGET_UNBOUND",
            Self::PhysicalBodyInactive => "PHYSICAL_BODY_INACTIVE",
            Self::PhysicalIntentAlreadyAssigned => "PHYSICAL_INTENT_ALREADY_ASSIGNED",
            Self::CommandExpired => "COMMAND_EXPIRED",
            Self::CommandFutureLimit => "COMMAND_FUTURE_LIMIT",
            Self::CommandSequenceCollision => "COMMAND_SEQUENCE_COLLISION",
            Self::CommandIdCollision => "COMMAND_ID_COLLISION",
            Self::CommandSequenceFinalized => "COMMAND_SEQUENCE_FINALIZED",
            Self::CommandSequenceExhausted => "COMMAND_SEQUENCE_EXHAUSTED",
            Self::CommandStreamTimeRegression => "COMMAND_STREAM_TIME_REGRESSION",
            Self::CommandPendingLimit => "COMMAND_PENDING_LIMIT",
            Self::CommandCollisionLocked => "COMMAND_COLLISION_LOCKED",
            Self::CommandStreamClosed => "COMMAND_STREAM_CLOSED",
            Self::RpgAggregateNotFound => "RPG_AGGREGATE_NOT_FOUND",
            Self::RpgSchemaUnsupported => "RPG_SCHEMA_UNSUPPORTED",
            Self::RpgRevisionStale => "RPG_REVISION_STALE",
            Self::RpgRevisionExhausted => "RPG_REVISION_EXHAUSTED",
            Self::RpgOperationOrderInvalid => "RPG_OPERATION_ORDER_INVALID",
            Self::RpgTransitionInvalid => "RPG_TRANSITION_INVALID",
            Self::RpgOwnershipConflict => "RPG_OWNERSHIP_CONFLICT",
            Self::RpgReservationInvalid => "RPG_RESERVATION_INVALID",
            Self::RpgPhysicalPreconditionMissing => "RPG_PHYSICAL_PRECONDITION_MISSING",
            Self::RpgDefinitionMismatch => "RPG_DEFINITION_MISMATCH",
            Self::RpgPlanStale => "RPG_PLAN_STALE",
            Self::RpgEventOrderInvalid => "RPG_EVENT_ORDER_INVALID",
            Self::RpgTransactionAborted => "RPG_TRANSACTION_ABORTED",
            Self::RpgCommitmentRejected => "RPG_COMMITMENT_REJECTED",
        }
    }

    pub(super) fn from_stable_code(code: &str) -> Self {
        [
            Self::SchemaMismatch,
            Self::CanonicalCommandInvalid,
            Self::CommandIdMismatch,
            Self::CommandPhaseForbidden,
            Self::InternalOutcomeIssuerRequired,
            Self::IssuerUnauthenticated,
            Self::CommandStreamUnbound,
            Self::CapabilityRequired,
            Self::CapabilityDenied,
            Self::PreconditionFailed,
            Self::TargetNotAllowed,
            Self::PhysicalTargetUnbound,
            Self::PhysicalBodyInactive,
            Self::PhysicalIntentAlreadyAssigned,
            Self::CommandExpired,
            Self::CommandFutureLimit,
            Self::CommandSequenceCollision,
            Self::CommandIdCollision,
            Self::CommandSequenceFinalized,
            Self::CommandSequenceExhausted,
            Self::CommandStreamTimeRegression,
            Self::CommandPendingLimit,
            Self::CommandCollisionLocked,
            Self::CommandStreamClosed,
            Self::RpgAggregateNotFound,
            Self::RpgSchemaUnsupported,
            Self::RpgRevisionStale,
            Self::RpgRevisionExhausted,
            Self::RpgOperationOrderInvalid,
            Self::RpgTransitionInvalid,
            Self::RpgOwnershipConflict,
            Self::RpgReservationInvalid,
            Self::RpgPhysicalPreconditionMissing,
            Self::RpgDefinitionMismatch,
            Self::RpgPlanStale,
            Self::RpgEventOrderInvalid,
            Self::RpgTransactionAborted,
            Self::RpgCommitmentRejected,
        ]
        .into_iter()
        .find(|candidate| candidate.as_str() == code)
        .unwrap_or(Self::RpgTransactionAborted)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandDisposition {
    Reserved,
    Committed,
    Deduplicated,
    Rejected(RejectionCode),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandResult {
    pub command_id: CommandId,
    pub sequence: u64,
    pub disposition: CommandDisposition,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TransactionStage {
    IngressClose,
    IngressAdmission,
    IngressCommit,
    PhysicalStep,
    PhysicsContactPublication,
    OutcomeCollection,
    OutcomeAdmission,
    OutcomeCommit,
    SnapshotPublication,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StageTraceEntry {
    pub stage: TransactionStage,
    pub received: u64,
    pub accepted: u64,
    pub rejected: u64,
    pub committed: u64,
    pub deduplicated: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TickReport {
    pub tick: u64,
    pub results: Vec<CommandResult>,
    pub events: Vec<DomainEvent>,
    pub stage_trace: Vec<StageTraceEntry>,
    pub snapshot: RuntimeSnapshot,
    pub rpg_snapshot: RpgSnapshotV2,
    pub physics_snapshot: PhysicsCanonicalSnapshotV2,
    pub physics_step_input: PhysicsStepInputV2,
    pub contact_batch: ClosedPhysicsContactBatchV1,
    pub physics_checkpoint_hash: ContentHash,
    pub closed_ingress_batch: ClosedIngressBatchV1,
    pub mapping_receipts: Vec<InputMappingReceiptV1>,
    pub command_batches: Vec<ClosedCommandAdmissionBatchV2>,
    pub rpg_plan_traces: Vec<CommittedRpgPlanTraceV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommittedRpgPlanTraceV1 {
    pub command_id: CommandId,
    pub plan_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OrderedResult {
    order_key: CommandOrderKey,
    pub(super) result: CommandResult,
}

impl OrderedResult {
    pub(super) fn with_disposition(
        order_key: CommandOrderKey,
        command_id: CommandId,
        sequence: u64,
        disposition: CommandDisposition,
    ) -> Self {
        Self {
            order_key,
            result: CommandResult {
                command_id,
                sequence,
                disposition,
            },
        }
    }

    pub(super) fn reserved(
        order_key: CommandOrderKey,
        command_id: CommandId,
        sequence: u64,
    ) -> Self {
        Self::with_disposition(
            order_key,
            command_id,
            sequence,
            CommandDisposition::Reserved,
        )
    }

    pub(super) fn committed(
        order_key: CommandOrderKey,
        command_id: CommandId,
        sequence: u64,
    ) -> Self {
        Self::with_disposition(
            order_key,
            command_id,
            sequence,
            CommandDisposition::Committed,
        )
    }

    pub(super) fn deduplicated(
        order_key: CommandOrderKey,
        command_id: CommandId,
        sequence: u64,
    ) -> Self {
        Self::with_disposition(
            order_key,
            command_id,
            sequence,
            CommandDisposition::Deduplicated,
        )
    }

    pub(super) fn rejected(
        order_key: CommandOrderKey,
        command_id: CommandId,
        sequence: u64,
        code: RejectionCode,
    ) -> Self {
        Self::with_disposition(
            order_key,
            command_id,
            sequence,
            CommandDisposition::Rejected(code),
        )
    }
}

impl Ord for OrderedResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order_key.cmp(&other.order_key)
    }
}

impl PartialOrd for OrderedResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
