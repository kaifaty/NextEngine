use next_contracts::ids::{ContentHash, SchemaId};
use next_contracts::session::{CloseSessionProgressV1, CloseSessionResultV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinalSaveAttemptFailureV1 {
    Retryable(SchemaId),
    Terminal(SchemaId),
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct CloseExecutionOptionsV1 {
    pub final_save_failure: Option<FinalSaveAttemptFailureV1>,
    pub last_safe_generation_hash: Option<ContentHash>,
}

impl CloseExecutionOptionsV1 {
    #[must_use]
    pub const fn with_failure(
        final_save_failure: FinalSaveAttemptFailureV1,
        last_safe_generation_hash: Option<ContentHash>,
    ) -> Self {
        Self {
            final_save_failure: Some(final_save_failure),
            last_safe_generation_hash,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationCloseOutcomeV1 {
    Progress(CloseSessionProgressV1),
    Closed {
        receipt_hash: ContentHash,
        result: CloseSessionResultV1,
        save_generation_hash: Option<ContentHash>,
    },
}

impl ApplicationCloseOutcomeV1 {
    #[must_use]
    pub const fn receipt_hash(&self) -> Option<ContentHash> {
        match self {
            Self::Progress(_) => None,
            Self::Closed { receipt_hash, .. } => Some(*receipt_hash),
        }
    }
}
