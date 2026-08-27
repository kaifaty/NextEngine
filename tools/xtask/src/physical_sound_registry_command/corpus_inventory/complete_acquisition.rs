use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::{
    FileRef, MAX_REFERENCED_FILE_BYTES, canonical_external_file, read_bounded_file,
    resolve_artifact, validate_file_ref, validate_label,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompleteAcquisition {
    pub(super) repeat_id: String,
    pub(super) force_profile_sample_rate_hz: u32,
    pub(super) force_profile_sample_count: usize,
    pub(super) force_profile_f32le_newtons: FileRef,
    pub(super) material_composition: FileRef,
    pub(super) support_fixture_revision: FileRef,
    pub(super) microphone_calibration: FileRef,
    pub(super) force_calibration: FileRef,
}

#[derive(Debug, Serialize)]
pub(super) struct CompleteAcquisitionReport {
    repeat_id: String,
    force_profile: ForceProfileReport,
    material_composition_sha256: String,
    support_fixture_revision_sha256: String,
    microphone_calibration_sha256: String,
    force_calibration_sha256: String,
}

#[derive(Debug, Serialize)]
struct ForceProfileReport {
    format: &'static str,
    sample_rate_hz: u32,
    sample_count: usize,
    sha256: String,
    byte_count: usize,
    peak_abs_newtons: f64,
    rms_newtons: f64,
    positive_impulse_newton_seconds: f64,
}

pub(super) fn validate(
    entry_id: &str,
    microphone_sample_rate_hz: u32,
    microphone_sample_count: usize,
    complete: &CompleteAcquisition,
) -> Result<(), String> {
    validate_label(&complete.repeat_id, "repeat id")?;
    if complete.force_profile_sample_rate_hz != microphone_sample_rate_hz
        || complete.force_profile_sample_count != microphone_sample_count
    {
        return Err(format!(
            "entry {entry_id} force and microphone dimensions must match exactly"
        ));
    }
    complete
        .force_profile_sample_count
        .checked_mul(4)
        .filter(|bytes| *bytes <= MAX_REFERENCED_FILE_BYTES)
        .ok_or_else(|| format!("entry {entry_id} force profile byte count is out of bounds"))?;
    for (reference, role) in [
        (&complete.force_profile_f32le_newtons, "force profile"),
        (&complete.material_composition, "material composition"),
        (
            &complete.support_fixture_revision,
            "support fixture revision",
        ),
        (&complete.microphone_calibration, "microphone calibration"),
        (&complete.force_calibration, "force calibration"),
    ] {
        validate_file_ref(reference, role)?;
    }
    Ok(())
}

pub(super) fn build_report(
    root: &Path,
    manifest_directory: &Path,
    entry_id: &str,
    complete: &CompleteAcquisition,
) -> Result<CompleteAcquisitionReport, String> {
    Ok(CompleteAcquisitionReport {
        repeat_id: complete.repeat_id.clone(),
        force_profile: analyse_force_profile(root, manifest_directory, entry_id, complete)?,
        material_composition_sha256: resolve_hash(
            root,
            manifest_directory,
            &complete.material_composition,
            "material composition",
        )?,
        support_fixture_revision_sha256: resolve_hash(
            root,
            manifest_directory,
            &complete.support_fixture_revision,
            "support fixture revision",
        )?,
        microphone_calibration_sha256: resolve_hash(
            root,
            manifest_directory,
            &complete.microphone_calibration,
            "microphone calibration",
        )?,
        force_calibration_sha256: resolve_hash(
            root,
            manifest_directory,
            &complete.force_calibration,
            "force calibration",
        )?,
    })
}

fn analyse_force_profile(
    root: &Path,
    manifest_directory: &Path,
    entry_id: &str,
    complete: &CompleteAcquisition,
) -> Result<ForceProfileReport, String> {
    let reference = &complete.force_profile_f32le_newtons;
    let artifact = resolve_artifact(root, manifest_directory, reference, "force profile")?;
    let expected_bytes = complete
        .force_profile_sample_count
        .checked_mul(4)
        .ok_or_else(|| format!("entry {entry_id} force byte count overflow"))?;
    if artifact.byte_count != expected_bytes {
        return Err(format!(
            "entry {entry_id} force byte count mismatch: expected {expected_bytes}, got {}",
            artifact.byte_count
        ));
    }
    let path = canonical_external_file(
        root,
        &manifest_directory.join(&reference.path),
        "force profile",
    )?;
    let bytes = read_bounded_file(&path, MAX_REFERENCED_FILE_BYTES, "force profile")?;
    let mut peak_abs = 0.0_f64;
    let mut square_sum = 0.0_f64;
    let mut positive_sum = 0.0_f64;
    for sample in bytes.chunks_exact(4) {
        let value = f32::from_le_bytes(sample.try_into().expect("four-byte chunk"));
        if !value.is_finite() {
            return Err(format!(
                "entry {entry_id} force profile contains non-finite samples"
            ));
        }
        let value = f64::from(value);
        peak_abs = peak_abs.max(value.abs());
        square_sum += value * value;
        positive_sum += value.max(0.0);
    }
    if peak_abs <= 0.0 || positive_sum <= 0.0 {
        return Err(format!(
            "entry {entry_id} force profile has no positive impact"
        ));
    }
    Ok(ForceProfileReport {
        format: "f32_le_newtons",
        sample_rate_hz: complete.force_profile_sample_rate_hz,
        sample_count: complete.force_profile_sample_count,
        sha256: artifact.sha256,
        byte_count: artifact.byte_count,
        peak_abs_newtons: peak_abs,
        rms_newtons: (square_sum / complete.force_profile_sample_count as f64).sqrt(),
        positive_impulse_newton_seconds: positive_sum
            / f64::from(complete.force_profile_sample_rate_hz),
    })
}

fn resolve_hash(
    root: &Path,
    manifest_directory: &Path,
    reference: &FileRef,
    role: &str,
) -> Result<String, String> {
    Ok(resolve_artifact(root, manifest_directory, reference, role)?.sha256)
}
