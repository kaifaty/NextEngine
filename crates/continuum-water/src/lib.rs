#![forbid(unsafe_code)]

mod boundary;
mod error;
mod hash;
mod kernel;
mod model;
mod oracle;
mod profile;
mod reference;
mod scenario;
mod solver;

use std::path::Path;

/// Tool-only entry point. Continuum-water records intentionally do not cross
/// into engine contracts or runtime crates.
pub fn run_xtask(
    repository_root: &Path,
    arguments: impl Iterator<Item = String>,
) -> Result<String, String> {
    oracle::run_xtask(repository_root, arguments).map_err(|error| error.to_string())
}
