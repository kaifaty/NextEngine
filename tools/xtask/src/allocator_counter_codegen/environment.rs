//! Inherited compiler/wrapper/profile/linker environment admission.
//!
//! ADR-043 fixes the exact codegen command identity: inherited compiler,
//! wrapper, profile or linker overrides either are proven empty/absent or fail
//! closed before any evidence build starts.

use super::AllocatorCounterCodegenError;

const FORBIDDEN_BUILD_ENVIRONMENT: [&str; 6] = [
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER",
    "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS",
];

pub(super) fn validate_build_environment() -> Result<(), AllocatorCounterCodegenError> {
    validate_build_environment_vars(std::env::vars_os())
}

pub(super) fn validate_build_environment_vars(
    variables: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Result<(), AllocatorCounterCodegenError> {
    for (name, value) in variables {
        let Some(name) = name.to_str() else {
            continue;
        };
        let forbidden =
            FORBIDDEN_BUILD_ENVIRONMENT.contains(&name) || name.starts_with("CARGO_PROFILE_");
        if forbidden && !value.is_empty() {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_ENVIRONMENT_FORBIDDEN",
                name,
            ));
        }
    }
    Ok(())
}
