use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static HOST_CHECK_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn run_output_with_state(
    root: &Path,
    program: &str,
    arguments: &[&str],
    state_root: Option<&Path>,
) -> Result<Output, String> {
    let mut command = Command::new(program);
    command.args(arguments).current_dir(root);
    let mut temporary = None;
    if let Some(state_root) = state_root {
        let local_app_data = state_root.join("local-app-data");
        let xdg_state_home = state_root.join("xdg-state");
        let roaming_app_data = state_root.join("roaming-app-data");
        for directory in [&local_app_data, &xdg_state_home, &roaming_app_data] {
            fs::create_dir_all(directory).map_err(|error| {
                format!(
                    "failed to create isolated host-check directory {}: {error}",
                    directory.display()
                )
            })?;
        }
        let host_temporary = create_external_host_check_temporary(root)?;
        command
            .env("LOCALAPPDATA", local_app_data)
            .env("APPDATA", roaming_app_data)
            .env("XDG_STATE_HOME", xdg_state_home)
            .env("TMP", &host_temporary)
            .env("TEMP", &host_temporary)
            .env("TMPDIR", &host_temporary);
        temporary = Some(host_temporary);
    }

    let output = command.output();
    let cleanup = temporary.as_deref().map(remove_external_temporary);
    match (output, cleanup) {
        (Ok(output), None | Some(Ok(()))) => Ok(output),
        (Ok(_), Some(Err(error))) => Err(error),
        (Err(error), None | Some(Ok(()))) => Err(format!("failed to run {program}: {error}")),
        (Err(error), Some(Err(cleanup))) => Err(format!(
            "failed to run {program}: {error}; host-check temporary cleanup also failed: {cleanup}"
        )),
    }
}

fn create_external_host_check_temporary(root: &Path) -> Result<PathBuf, String> {
    let base = external_temporary_base(root, &env::temp_dir())?;
    let sequence = HOST_CHECK_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = base.join(format!(
        "nextengine-native-host-check-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&temporary).map_err(|error| {
        format!(
            "failed to create external host-check temporary {}: {error}",
            temporary.display()
        )
    })?;
    Ok(temporary)
}

fn external_temporary_base(root: &Path, candidate: &Path) -> Result<PathBuf, String> {
    let repository = fs::canonicalize(root).map_err(|error| {
        format!(
            "failed to resolve repository for host-check isolation {}: {error}",
            root.display()
        )
    })?;
    let candidate = fs::canonicalize(candidate).map_err(|error| {
        format!(
            "failed to resolve host temporary base {}: {error}",
            candidate.display()
        )
    })?;
    if candidate.starts_with(&repository) {
        return Err(format!(
            "host-check temporary base must stay outside the repository: {}",
            candidate.display()
        ));
    }
    Ok(candidate)
}

fn remove_external_temporary(path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to remove external host-check temporary {}: {error}",
            path.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_check_temporary_base_is_external_to_the_repository() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("xtask belongs to the workspace");
        let base = external_temporary_base(root, &env::temp_dir()).expect("external temp base");
        assert!(!base.starts_with(fs::canonicalize(root).expect("repository")));
    }

    #[test]
    fn repository_local_temporary_base_is_rejected() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("xtask belongs to the workspace");
        let local = root.join("target");
        assert!(
            external_temporary_base(root, &local)
                .expect_err("repository-local temp rejects")
                .contains("must stay outside the repository")
        );
    }

    #[cfg(unix)]
    #[test]
    fn state_runner_gives_the_child_an_external_temporary_and_cleans_it() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("xtask belongs to the workspace");
        let state_root = root.join("target").join(format!(
            "nextengine-native-gate-environment-test-{}",
            std::process::id()
        ));
        assert!(!state_root.exists(), "test state must start absent");
        let output = run_output_with_state(
            root,
            "sh",
            &["-c", "printf %s \"$TMPDIR\""],
            Some(&state_root),
        )
        .expect("isolated child runs");
        assert!(output.status.success());
        let temporary = PathBuf::from(String::from_utf8(output.stdout).expect("UTF-8 temp path"));
        assert!(!temporary.starts_with(fs::canonicalize(root).expect("repository")));
        assert!(!temporary.exists(), "external temporary is cleaned");
        fs::remove_dir_all(state_root).expect("remove owned test state");
    }
}
