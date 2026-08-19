#![forbid(unsafe_code)]

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{
    REFERENCE_CORPUS_MISMATCH, REFERENCE_INPUT_CAPACITY_EXCEEDED, SCENARIO_INVALID, WaterError,
};
use crate::model::checked_scalar;
use crate::model::{OutputMetric, Scenario};
use crate::profile::{MAXIMUM_REFERENCE_INPUT_BYTES, quantize_micrometres, quantize_ppb};
use crate::scenario::validate_capacity;

const MAGIC: &[u8; 8] = b"CWREFV1\0";

const HYDRO_REFERENCE_SHA256: &str =
    "84ae867f5b336cd0bd51be6f29a6a2a1f27f702c424f1dbd0a8f735b9f4bb435";
const DAM_BREAK_REFERENCE_SHA256: &str =
    "853d965489a40082a024aeee5a19f98aef054417014af8556fa212687d88d12c";
const ORIFICE_REFERENCE_SHA256: &str =
    "60e9b3538d621ef1a3f1ae77569640740df471fbe4e5ef8eaa813d3751930849";

const REFERENCE_ATTESTATION_PROJECTION: &str = concat!(
    "REFERENCE_ATTESTATION_V1_BEGIN\n",
    "upstream.repository=InteractiveComputerGraphics/SPlisHSPlasH\n",
    "upstream.commit=eccce86155776f6ac52d5080b1f720a52bf29450\n",
    "adaptation.tracked-diff.sha256=4effa812553649c89135ca9515aaac410e47eca1fe250890766c0d6183165a4b\n",
    "adaptation.comparator-source.sha256=f87a598a03b1893646de8188c393b41c262fb335e4f980a3672ea099d8461c11\n",
    "adaptation.binary.sha256=ee0e12ea5ef6afc6090a6404a259d75769bae28448119edc9764a96bb705d0aa\n",
    "build=release;binary64;avx-off;omp-threads-1\n",
    "solver=dfsph;dt=1/240;cfl-off;warmstart-off;viscosity-off;surface-tension-off\n",
    "fluid=6000-samples;volume-m3=0.000125;mass-kg=0.125\n",
    "boundary.outer=support-complete-two-layer-lattice;volume=akinci-2012\n",
    "boundary.orifice=left-source-two-layer-support;volume=akinci-2012\n",
    "contact.outer=predictive-particle-radius;clearance-m=0.025\n",
    "contact.internal=swept-plane-edge-corner;iterations=8;clearance-m=0.025\n",
    "validation=outer-clearance;internal-clearance;safe-aperture-crossing;self-test-vectors\n",
    "reference.CW-HYDRO-001.sha256=84ae867f5b336cd0bd51be6f29a6a2a1f27f702c424f1dbd0a8f735b9f4bb435\n",
    "reference.CW-DAMBREAK-001.sha256=853d965489a40082a024aeee5a19f98aef054417014af8556fa212687d88d12c\n",
    "reference.CW-ORIFICE-001.sha256=60e9b3538d621ef1a3f1ae77569640740df471fbe4e5ef8eaa813d3751930849\n",
    "REFERENCE_ATTESTATION_V1_END\n",
);

pub(crate) fn expected_sha256(scenario_id: &str) -> Option<&'static str> {
    match scenario_id {
        "CW-HYDRO-001" => Some(HYDRO_REFERENCE_SHA256),
        "CW-DAMBREAK-001" => Some(DAM_BREAK_REFERENCE_SHA256),
        "CW-ORIFICE-001" => Some(ORIFICE_REFERENCE_SHA256),
        _ => None,
    }
}

pub(crate) fn attestation_profile_root() -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"nextengine.continuum-water.w1-reference-attestation.v1\0");
    digest.update(REFERENCE_ATTESTATION_PROJECTION.as_bytes());
    digest.finalize().into()
}

#[derive(Clone, Debug)]
pub(crate) struct ReferenceCorpus {
    pub(crate) sha256: String,
    pub(crate) outputs: Vec<ReferenceOutput>,
}

#[derive(Clone, Debug)]
pub(crate) struct ReferenceOutput {
    pub(crate) step: u32,
    pub(crate) q99_x_um: i64,
    pub(crate) q99_y_um: i64,
    pub(crate) receiver_count: u32,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CurveComparison {
    pub(crate) metric: &'static str,
    pub(crate) rmse_ppb: i64,
    pub(crate) maximum_absolute_error_ppb: i64,
    pub(crate) maximum_absolute_error_step: u32,
    pub(crate) candidate_at_maximum_ppb: i64,
    pub(crate) reference_at_maximum_ppb: i64,
    pub(crate) threshold_rmse_ppb: i64,
    pub(crate) threshold_maximum_absolute_error_ppb: i64,
    pub(crate) passed: bool,
}

pub(crate) fn load(
    path: &Path,
    scenario: &Scenario,
    scenario_root: &[u8; 32],
    sample_count: usize,
) -> Result<ReferenceCorpus, WaterError> {
    let byte_count = preflight_size(path)?;
    let mut file = File::open(path).map_err(|error| {
        WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            format!("cannot open reference {}: {error}", path.display()),
        )
    })?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(byte_count)
        .map_err(reference_reserve_error)?;
    file.read_to_end(&mut bytes).map_err(|error| {
        WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            format!("cannot read reference {}: {error}", path.display()),
        )
    })?;
    if bytes.len() != byte_count {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            "reference size changed while it was read",
        ));
    }
    let sha256 = digest_hex(&bytes);
    let mut cursor = Cursor::new(&bytes);
    if cursor.take(8)? != MAGIC {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            "reference magic is not CWREFV1",
        ));
    }
    if cursor.take(32)? != scenario_root {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            "reference scenario root differs from the requested scenario",
        ));
    }
    let output_count = usize::try_from(cursor.u32()?).map_err(|_| {
        WaterError::new(REFERENCE_CORPUS_MISMATCH, "reference output count overflow")
    })?;
    let encoded_sample_count = usize::try_from(cursor.u32()?).map_err(|_| {
        WaterError::new(REFERENCE_CORPUS_MISMATCH, "reference sample count overflow")
    })?;
    if encoded_sample_count != sample_count {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            format!("reference has {encoded_sample_count} samples, expected {sample_count}"),
        ));
    }
    let expected_steps = output_steps(scenario)?;
    if output_count != expected_steps.len() {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            format!(
                "reference has {output_count} outputs, expected {}",
                expected_steps.len()
            ),
        ));
    }
    let payload_per_output = sample_count
        .checked_mul(24)
        .and_then(|value| value.checked_add(4))
        .ok_or_else(|| {
            WaterError::new(
                REFERENCE_INPUT_CAPACITY_EXCEEDED,
                "reference output byte product overflow",
            )
        })?;
    let expected_bytes = output_count
        .checked_mul(payload_per_output)
        .and_then(|value| value.checked_add(48))
        .ok_or_else(|| {
            WaterError::new(
                REFERENCE_INPUT_CAPACITY_EXCEEDED,
                "reference total byte product overflow",
            )
        })?;
    if expected_bytes != bytes.len() {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            format!(
                "reference byte length {} differs from exact expected {expected_bytes}",
                bytes.len()
            ),
        ));
    }
    let mut outputs = Vec::new();
    outputs.try_reserve_exact(output_count).map_err(|error| {
        WaterError::new(
            REFERENCE_INPUT_CAPACITY_EXCEEDED,
            format!("cannot reserve reference outputs: {error}"),
        )
    })?;
    for expected_step in expected_steps {
        let step = cursor.u32()?;
        if step != expected_step {
            return Err(WaterError::new(
                REFERENCE_CORPUS_MISMATCH,
                format!("reference step {step}, expected {expected_step}"),
            ));
        }
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        xs.try_reserve_exact(sample_count)
            .map_err(reference_reserve_error)?;
        ys.try_reserve_exact(sample_count)
            .map_err(reference_reserve_error)?;
        let mut receiver_count = 0_u32;
        for _sample_id in 0..sample_count {
            let x = quantize_micrometres(f64::from_bits(cursor.u64()?))?;
            let y = quantize_micrometres(f64::from_bits(cursor.u64()?))?;
            let _z = quantize_micrometres(f64::from_bits(cursor.u64()?))?;
            xs.push(x);
            ys.push(y);
            if x >= 1_000_000 {
                receiver_count = receiver_count.checked_add(1).ok_or_else(|| {
                    WaterError::new(REFERENCE_CORPUS_MISMATCH, "receiver count overflow")
                })?;
            }
        }
        xs.sort_unstable();
        ys.sort_unstable();
        outputs.push(ReferenceOutput {
            step,
            q99_x_um: nearest_rank_99(&xs)?,
            q99_y_um: nearest_rank_99(&ys)?,
            receiver_count,
        });
    }
    if cursor.remaining() != 0 {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            format!("reference has {} trailing bytes", cursor.remaining()),
        ));
    }
    Ok(ReferenceCorpus { sha256, outputs })
}

pub(crate) fn compare_curves(
    scenario: &Scenario,
    candidate: &[OutputMetric],
    reference: &ReferenceCorpus,
) -> Result<Vec<CurveComparison>, WaterError> {
    if candidate.len() != reference.outputs.len() {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            "candidate and reference output counts differ",
        ));
    }
    match scenario.id {
        "CW-DAMBREAK-001" => Ok(vec![
            compare(
                "dam-break-front",
                candidate,
                &reference.outputs,
                |value| normalized_front(value.q99_x_um, 4_000_000),
                |value| normalized_front(value.q99_x_um, 4_000_000),
            )?,
            compare(
                "dam-break-height",
                candidate,
                &reference.outputs,
                |value| normalized_front(value.q99_y_um, 1_000_000),
                |value| normalized_front(value.q99_y_um, 1_000_000),
            )?,
        ]),
        "CW-ORIFICE-001" => Ok(vec![compare(
            "orifice-transfer",
            candidate,
            &reference.outputs,
            |value| Ok(f64::from(value.receiver_count) / 6_000.0),
            |value| Ok(f64::from(value.receiver_count) / 6_000.0),
        )?]),
        _ => Ok(Vec::new()),
    }
}

fn compare(
    metric: &'static str,
    candidate: &[OutputMetric],
    reference: &[ReferenceOutput],
    candidate_value: impl Fn(&OutputMetric) -> Result<f64, WaterError>,
    reference_value: impl Fn(&ReferenceOutput) -> Result<f64, WaterError>,
) -> Result<CurveComparison, WaterError> {
    if candidate.is_empty() {
        return Err(WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            "curve comparison has no outputs",
        ));
    }
    let mut squared_sum = 0.0;
    let mut maximum = 0.0;
    let mut maximum_step = 0_u32;
    let mut candidate_at_maximum = 0.0;
    let mut reference_at_maximum = 0.0;
    for (candidate, reference) in candidate.iter().zip(reference) {
        if candidate.step != reference.step {
            return Err(WaterError::new(
                REFERENCE_CORPUS_MISMATCH,
                format!(
                    "candidate step {} differs from reference step {}",
                    candidate.step, reference.step
                ),
            ));
        }
        let candidate_curve_value = candidate_value(candidate)?;
        let reference_curve_value = reference_value(reference)?;
        let difference = checked_scalar(
            candidate_curve_value - reference_curve_value,
            "reference curve difference",
        )?;
        let square = checked_scalar(difference * difference, "reference curve square")?;
        squared_sum = checked_scalar(squared_sum + square, "reference curve reduction")?;
        let absolute = difference.abs();
        if absolute > maximum {
            maximum = absolute;
            maximum_step = candidate.step;
            candidate_at_maximum = candidate_curve_value;
            reference_at_maximum = reference_curve_value;
        }
    }
    let mean = checked_scalar(
        squared_sum / (candidate.len() as f64),
        "reference curve mean square",
    )?;
    let rmse_ppb = quantize_ppb(checked_scalar(mean.sqrt(), "reference curve RMSE")?)?;
    let maximum_absolute_error_ppb = quantize_ppb(maximum)?;
    Ok(CurveComparison {
        metric,
        rmse_ppb,
        maximum_absolute_error_ppb,
        maximum_absolute_error_step: maximum_step,
        candidate_at_maximum_ppb: quantize_ppb(candidate_at_maximum)?,
        reference_at_maximum_ppb: quantize_ppb(reference_at_maximum)?,
        threshold_rmse_ppb: 50_000_000,
        threshold_maximum_absolute_error_ppb: 100_000_000,
        passed: rmse_ppb <= 50_000_000 && maximum_absolute_error_ppb <= 100_000_000,
    })
}

fn normalized_front(q99_um: i64, maximum_um: i64) -> Result<f64, WaterError> {
    let particle_edge = q99_um
        .checked_add(25_000)
        .ok_or_else(|| WaterError::new(REFERENCE_CORPUS_MISMATCH, "reference front overflow"))?;
    let clamped = if particle_edge < maximum_um {
        particle_edge
    } else {
        maximum_um
    };
    Ok((clamped as f64) / (maximum_um as f64))
}

pub(crate) fn output_steps(scenario: &Scenario) -> Result<Vec<u32>, WaterError> {
    let count = scenario
        .steps
        .checked_div(scenario.output_every)
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "output step count overflow"))?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(count as usize)
        .map_err(reference_reserve_error)?;
    let mut step = 0_u32;
    loop {
        result.push(step);
        if step == scenario.steps {
            break;
        }
        step = step
            .checked_add(scenario.output_every)
            .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "output step increment overflow"))?;
        if step > scenario.steps {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                "output cadence does not include the final step",
            ));
        }
    }
    Ok(result)
}

pub(crate) fn nearest_rank_99(sorted: &[i64]) -> Result<i64, WaterError> {
    if sorted.is_empty() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "q99 is undefined for an empty sample set",
        ));
    }
    let numerator = sorted
        .len()
        .checked_mul(99)
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "q99 numerator overflow"))?;
    let rank = numerator
        .checked_add(99)
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "q99 ceil overflow"))?
        / 100;
    Ok(sorted[rank - 1])
}

pub(crate) fn validate_reference_capacity(byte_count: usize) -> Result<(), WaterError> {
    validate_capacity(
        byte_count,
        MAXIMUM_REFERENCE_INPUT_BYTES,
        REFERENCE_INPUT_CAPACITY_EXCEEDED,
        "reference input bytes",
    )
}

pub(crate) fn preflight_size(path: &Path) -> Result<usize, WaterError> {
    let metadata = fs::metadata(path).map_err(|error| {
        WaterError::new(
            REFERENCE_CORPUS_MISMATCH,
            format!("cannot inspect reference {}: {error}", path.display()),
        )
    })?;
    let byte_count = usize::try_from(metadata.len()).map_err(|_| {
        WaterError::new(
            REFERENCE_INPUT_CAPACITY_EXCEEDED,
            "reference byte count does not fit usize",
        )
    })?;
    validate_reference_capacity(byte_count)?;
    Ok(byte_count)
}

fn reference_reserve_error(error: std::collections::TryReserveError) -> WaterError {
    WaterError::new(
        REFERENCE_INPUT_CAPACITY_EXCEEDED,
        format!("reference allocation failed: {error}"),
    )
}

fn digest_hex(bytes: &[u8]) -> String {
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    crate::hash::hex(&digest)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], WaterError> {
        let end = self.offset.checked_add(count).ok_or_else(|| {
            WaterError::new(REFERENCE_CORPUS_MISMATCH, "reference cursor overflow")
        })?;
        let value = self.bytes.get(self.offset..end).ok_or_else(|| {
            WaterError::new(REFERENCE_CORPUS_MISMATCH, "reference payload is truncated")
        })?;
        self.offset = end;
        Ok(value)
    }

    fn u32(&mut self) -> Result<u32, WaterError> {
        let bytes: [u8; 4] = self
            .take(4)?
            .try_into()
            .map_err(|_| WaterError::new(REFERENCE_CORPUS_MISMATCH, "invalid u32 field"))?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn u64(&mut self) -> Result<u64, WaterError> {
        let bytes: [u8; 8] = self
            .take(8)?
            .try_into()
            .map_err(|_| WaterError::new(REFERENCE_CORPUS_MISMATCH, "invalid u64 field"))?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Box3i, Geometry, Vec3i};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn reference_capacity_accepts_n_minus_one_and_n() {
        assert!(validate_reference_capacity(MAXIMUM_REFERENCE_INPUT_BYTES - 1).is_ok());
        assert!(validate_reference_capacity(MAXIMUM_REFERENCE_INPUT_BYTES).is_ok());
        assert_eq!(
            validate_reference_capacity(MAXIMUM_REFERENCE_INPUT_BYTES + 1)
                .unwrap_err()
                .code(),
            REFERENCE_INPUT_CAPACITY_EXCEEDED
        );
    }

    #[test]
    fn nearest_rank_quantile_uses_frozen_integer_index() {
        let values: Vec<i64> = (1..=100).collect();
        assert_eq!(nearest_rank_99(&values).unwrap(), 99);
        assert_eq!(nearest_rank_99(&[7]).unwrap(), 7);
    }

    #[test]
    fn w1_reference_attestation_admits_only_the_three_frozen_hashes() {
        assert_eq!(
            expected_sha256("CW-HYDRO-001"),
            Some(HYDRO_REFERENCE_SHA256)
        );
        assert_eq!(
            expected_sha256("CW-DAMBREAK-001"),
            Some(DAM_BREAK_REFERENCE_SHA256)
        );
        assert_eq!(
            expected_sha256("CW-ORIFICE-001"),
            Some(ORIFICE_REFERENCE_SHA256)
        );
        assert_eq!(expected_sha256("CW-FREEFALL-001"), None);
        assert_eq!(
            crate::hash::hex(&attestation_profile_root()),
            "186e1e31c0aa2636525bbc54e4fe4335b8432e7e99eddf3221d08e0380b65c90"
        );
    }

    #[test]
    fn bounded_binary_interchange_decodes_exact_canonical_positions() {
        let scenario = Scenario {
            id: "TEST-REFERENCE",
            kind: "test-only",
            geometry: Geometry {
                bounds: Box3i {
                    min: Vec3i::new(-1_000_000, -1_000_000, -1_000_000),
                    max: Vec3i::new(1_000_000, 1_000_000, 1_000_000),
                },
                aperture: None,
            },
            dimensions: [1, 1, 1],
            first_position_um: Vec3i::new(0, 0, 0),
            initial_velocity_um_s: Vec3i::new(0, 0, 0),
            steps: 0,
            output_every: 1,
            smoke_only: true,
        };
        let root = [0x5a_u8; 32];
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&root);
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        for value in [0.125_f64, -0.25_f64, 0.5_f64] {
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nextengine-water-reference-{}-{nonce}.bin",
            std::process::id()
        ));
        fs::write(&path, &bytes).unwrap();
        let decoded = load(&path, &scenario, &root, 1).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(decoded.outputs.len(), 1);
        assert_eq!(decoded.outputs[0].step, 0);
        assert_eq!(decoded.outputs[0].q99_x_um, 125_000);
        assert_eq!(decoded.outputs[0].q99_y_um, -250_000);
    }
}
