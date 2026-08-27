use std::path::Path;

use super::super::{
    MAX_REFERENCED_FILE_BYTES, canonical_external_file, read_bounded_file, resolve_artifact,
};
use super::{AudioReport, InventoryEntry};

pub(super) fn analyse(
    root: &Path,
    manifest_directory: &Path,
    entry: &InventoryEntry,
) -> Result<AudioReport, String> {
    let artifact = resolve_artifact(
        root,
        manifest_directory,
        &entry.audio_payload,
        "inventory audio payload",
    )?;
    let expected_bytes = entry
        .sample_count
        .checked_mul(4)
        .ok_or_else(|| format!("entry {} sample byte count overflow", entry.id))?;
    if artifact.byte_count != expected_bytes {
        return Err(format!(
            "entry {} audio byte count mismatch: expected {expected_bytes}, got {}",
            entry.id, artifact.byte_count
        ));
    }
    let path = canonical_external_file(
        root,
        &manifest_directory.join(&entry.audio_payload.path),
        "inventory audio payload",
    )?;
    let bytes = read_bounded_file(&path, MAX_REFERENCED_FILE_BYTES, "inventory audio payload")?;
    let mut peak_abs = 0.0_f64;
    let mut square_sum = 0.0_f64;
    for sample in bytes.chunks_exact(4) {
        let value = f32::from_le_bytes(sample.try_into().expect("four-byte chunk"));
        if !value.is_finite() {
            return Err(format!(
                "entry {} audio contains non-finite samples",
                entry.id
            ));
        }
        let value = f64::from(value);
        peak_abs = peak_abs.max(value.abs());
        square_sum += value * value;
    }
    Ok(AudioReport {
        format: "f32_le_mono",
        sha256: artifact.sha256,
        byte_count: artifact.byte_count,
        peak_abs,
        rms: (square_sum / entry.sample_count as f64).sqrt(),
    })
}
