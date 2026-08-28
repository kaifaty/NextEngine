use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

pub(super) fn report(
    output: &Path,
    role: &str,
    manifest: &[u8],
    prerequisites: &[(&str, &[u8])],
    report: &[u8],
) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| format!("{role} output has no parent"))?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-realimpact-{role}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging).map_err(|error| format!("create {role} staging: {error}"))?;
    let guard = StagingGuard(staging.clone());
    write(&staging.join("manifest.json"), manifest)?;
    for (name, bytes) in prerequisites {
        write(&staging.join(name), bytes)?;
    }
    write(&staging.join("report.json"), report)?;
    if output.exists() {
        fs::remove_dir(output)
            .map_err(|error| format!("remove confirmed-empty {role} output: {error}"))?;
    }
    fs::rename(&staging, output).map_err(|error| format!("publish {role}: {error}"))?;
    std::mem::forget(guard);
    Ok(())
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

struct StagingGuard(PathBuf);

impl Drop for StagingGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
