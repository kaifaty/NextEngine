use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) const LINUX_GLIBC_BASELINE_BUILD_CACHE: &str = "nextengine-package-glibc-2.35-v3";
pub(super) const LINUX_SDL_CMAKE_TOOLCHAIN: &str = "tools/linux-sdl-glibc-2.35.cmake";
pub(super) const LINUX_SDL_CMAKE_TOOLCHAIN_ENV: &str =
    "CMAKE_TOOLCHAIN_FILE_x86_64_unknown_linux_gnu";

pub(super) fn run_checked_with_environment(
    root: &Path,
    program: &str,
    arguments: &[&str],
    environment: &[(&str, &Path)],
) -> Result<(), String> {
    let mut command = Command::new(program);
    command.args(arguments).current_dir(root);
    for (name, value) in environment {
        command.env(name, value);
    }
    let output = command.output().map_err(|error| {
        format!("NATIVE_GATE_PACKAGE_INVALID: failed to run {program}: {error}")
    })?;
    if !output.stdout.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "NATIVE_GATE_PACKAGE_INVALID: {program} {} failed with {}",
            arguments.join(" "),
            output.status
        ))
    }
}

pub(super) fn cargo_target_directory(repository_root: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR").map_or_else(
        || repository_root.join("target"),
        |configured| {
            let configured = PathBuf::from(configured);
            if configured.is_absolute() {
                configured
            } else {
                repository_root.join(configured)
            }
        },
    )
}

pub(super) fn package_build_target_directory(
    target_directory: &Path,
    target_triple: &str,
) -> PathBuf {
    if target_triple == "x86_64-unknown-linux-gnu" {
        target_directory.join(LINUX_GLIBC_BASELINE_BUILD_CACHE)
    } else {
        target_directory.to_path_buf()
    }
}

pub(super) fn package_sdl_toolchain_file(
    repository_root: &Path,
    target_triple: &str,
) -> Option<PathBuf> {
    (target_triple == "x86_64-unknown-linux-gnu")
        .then(|| repository_root.join(LINUX_SDL_CMAKE_TOOLCHAIN))
}

pub(super) fn release_binary_directory(target_directory: &Path, target_triple: &str) -> PathBuf {
    target_directory.join(target_triple).join("release")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_package_build_isolates_the_glibc_baseline_configuration() {
        let repository = Path::new("repository");
        let target_directory = Path::new("cargo-target");
        assert_eq!(
            package_build_target_directory(target_directory, "x86_64-unknown-linux-gnu"),
            target_directory.join(LINUX_GLIBC_BASELINE_BUILD_CACHE)
        );
        assert_eq!(
            package_sdl_toolchain_file(repository, "x86_64-unknown-linux-gnu"),
            Some(repository.join(LINUX_SDL_CMAKE_TOOLCHAIN))
        );
        assert_eq!(
            package_build_target_directory(target_directory, "x86_64-pc-windows-msvc"),
            target_directory
        );
        assert_eq!(
            package_sdl_toolchain_file(repository, "x86_64-pc-windows-msvc"),
            None
        );
    }
}
