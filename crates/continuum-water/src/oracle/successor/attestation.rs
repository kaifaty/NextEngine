use super::W1SolverMode;
use crate::error::{REFERENCE_CORPUS_MISMATCH, WaterError};
use crate::reference;

pub(super) fn validate(
    solver_mode: W1SolverMode,
    required: bool,
    scenario_id: &str,
    actual_sha256: &str,
) -> Result<bool, WaterError> {
    let attested = reference::expected_sha256(scenario_id) == Some(actual_sha256);
    if solver_mode == W1SolverMode::FrozenSuccessor && required && !attested {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            format!(
                "{scenario_id} reference SHA-256 is not admitted by the W1 attestation profile"
            ),
        ));
    }
    Ok(attested)
}
