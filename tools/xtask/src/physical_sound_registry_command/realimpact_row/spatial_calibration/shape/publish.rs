use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::acquire::Acquisition;

static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

pub(super) fn development(
    output: &Path,
    manifest: &[u8],
    acquisitions: &[Acquisition],
    report: &[u8],
) -> Result<(), String> {
    let staging = staging(output, "shape-development")?;
    let guard = StagingGuard(staging.clone());
    write_file(&staging.join("manifest.json"), manifest)?;
    for acquisition in acquisitions {
        write_file(
            &staging.join(&acquisition.summary.selected_block_path),
            &acquisition.payload,
        )?;
    }
    write_file(&staging.join("report.json"), report)?;
    complete(output, &staging, guard, "shape-development")
}

pub(super) fn calibration(
    output: &Path,
    manifest: &[u8],
    development_report: &[u8],
    acquisitions: &[Acquisition],
    report: &[u8],
) -> Result<(), String> {
    let staging = staging(output, "shape-calibration")?;
    let guard = StagingGuard(staging.clone());
    write_file(&staging.join("manifest.json"), manifest)?;
    write_file(&staging.join("development-report.json"), development_report)?;
    for acquisition in acquisitions {
        write_file(
            &staging.join(&acquisition.summary.selected_block_path),
            &acquisition.payload,
        )?;
    }
    write_file(&staging.join("report.json"), report)?;
    complete(output, &staging, guard, "shape-calibration")
}

fn staging(output: &Path, role: &str) -> Result<PathBuf, String> {
    let parent = output
        .parent()
        .ok_or_else(|| format!("{role} output has no parent"))?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-realimpact-{role}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging).map_err(|error| format!("create {role} staging: {error}"))?;
    Ok(staging)
}

fn complete(output: &Path, staging: &Path, guard: StagingGuard, role: &str) -> Result<(), String> {
    if output.exists() {
        fs::remove_dir(output)
            .map_err(|error| format!("remove confirmed-empty {role} output: {error}"))?;
    }
    fs::rename(staging, output).map_err(|error| format!("publish {role} output: {error}"))?;
    std::mem::forget(guard);
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

struct StagingGuard(PathBuf);

impl Drop for StagingGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
