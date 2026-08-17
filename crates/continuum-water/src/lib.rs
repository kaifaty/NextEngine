#![forbid(unsafe_code)]

mod audit;
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
    mut arguments: impl Iterator<Item = String>,
) -> Result<String, String> {
    let material = arguments.next();
    let command = arguments.next();
    let result = match (&material, &command) {
        (Some(material), Some(command)) if material == "water" && command == "audit-hydro" => {
            audit::run_xtask(repository_root, arguments)
        }
        _ => oracle::run_xtask(
            repository_root,
            material.into_iter().chain(command).chain(arguments),
        ),
    };
    result.map_err(|error| error.to_string())
}
