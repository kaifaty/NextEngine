//! Recursion-guard absence proof across emitted release artifacts.
//!
//! ADR-043 admits count-bearing callbacks without a dynamic recursion guard
//! only when the pinned release IR and assembly carry no `in_callback` state
//! or recursion-rejection machinery at all.

use super::AllocatorCounterCodegenError;

pub(super) fn validate_recursion_free_paths(
    counter_ir: &str,
    counter_asm: &str,
    helper_ir: &str,
    helper_asm: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    for (artifact, body) in [
        ("counter IR", counter_ir),
        ("counter ASM", counter_asm),
        ("helper IR", helper_ir),
        ("helper ASM", helper_asm),
    ] {
        if let Some(found) = ["in_callback", "reject_recursion"]
            .iter()
            .find(|needle| body.contains(**needle))
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_RECURSION_GUARD_PRESENT",
                format!("{artifact}: {found}"),
            ));
        }
    }
    Ok(())
}
