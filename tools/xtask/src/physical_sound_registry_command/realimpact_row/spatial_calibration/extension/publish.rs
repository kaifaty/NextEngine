use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

pub(super) fn development(
    output: &Path,
    manifest: &[u8],
    spatial_report: &[u8],
    transfer_report: &[u8],
    payload: &[u8],
    report: &[u8],
) -> Result<(), String> {
    let staging = staging(output, "spatial-extension")?;
    let guard = StagingGuard(staging.clone());
    write_file(&staging.join("manifest.json"), manifest)?;
    write_file(
        &staging.join("spatial-calibration-report.json"),
        spatial_report,
    )?;
    write_file(
        &staging.join("transfer-calibration-report.json"),
        transfer_report,
    )?;
    write_file(&staging.join("green-axis-selected-blocks.f32le"), payload)?;
    write_file(&staging.join("report.json"), report)?;
    complete(output, &staging, guard, "spatial-extension")
}

#[allow(clippy::too_many_arguments)]
pub(super) fn evaluation(
    output: &Path,
    manifest: &[u8],
    spatial_report: &[u8],
    transfer_report: &[u8],
    development_report: &[u8],
    blue_payload: &[u8],
    glass_payload: &[u8],
    report: &[u8],
) -> Result<(), String> {
    let staging = staging(output, "spatial-evaluation")?;
    let guard = StagingGuard(staging.clone());
    write_file(&staging.join("manifest.json"), manifest)?;
    write_file(
        &staging.join("spatial-calibration-report.json"),
        spatial_report,
    )?;
    write_file(
        &staging.join("transfer-calibration-report.json"),
        transfer_report,
    )?;
    write_file(
        &staging.join("axis-development-report.json"),
        development_report,
    )?;
    write_file(
        &staging.join("blue-axis-selected-blocks.f32le"),
        blue_payload,
    )?;
    write_file(
        &staging.join("glass-axis-selected-blocks.f32le"),
        glass_payload,
    )?;
    write_file(&staging.join("report.json"), report)?;
    complete(output, &staging, guard, "spatial-evaluation")
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
    fs::create_dir(&staging)
        .map_err(|error| format!("create {role} staging directory: {error}"))?;
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
