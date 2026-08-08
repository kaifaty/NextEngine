use next_contracts::ids::ContentHash;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationCloseOutcomeV2 {
    Closed {
        receipt_hash: ContentHash,
        save_generation_hash: ContentHash,
    },
}
