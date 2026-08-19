#![forbid(unsafe_code)]

use super::heap_error;
use crate::error::{NONFINITE_VALUE, NUMERIC_OVERFLOW, WaterError};
use crate::model::{CanonicalSample, Vec3i};
use crate::profile::quantize_ppb;

pub(super) fn density_ratio_percentiles(values: &[f64]) -> Result<[i64; 3], WaterError> {
    if values.is_empty() {
        return Ok([0; 3]);
    }
    let mut sorted = Vec::new();
    sorted.try_reserve_exact(values.len()).map_err(heap_error)?;
    for value in values {
        if !value.is_finite() {
            return Err(WaterError::new(
                NONFINITE_VALUE,
                "nonfinite reconstructed density ratio",
            ));
        }
        sorted.push(*value);
    }
    sorted.sort_unstable_by(f64::total_cmp);
    let percentile = |percent: usize| -> Result<i64, WaterError> {
        let rank = sorted
            .len()
            .checked_mul(percent)
            .and_then(|value| value.checked_add(99))
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "density percentile overflow"))?
            / 100;
        quantize_ppb(sorted[rank - 1])
    };
    Ok([percentile(50)?, percentile(95)?, percentile(99)?])
}

pub(super) fn centre_of_mass(samples: &[CanonicalSample]) -> Result<Vec3i, WaterError> {
    if samples.is_empty() {
        return Ok(Vec3i::new(0, 0, 0));
    }
    let mut sums = [0_i128; 3];
    for sample in samples {
        for (sum, value) in sums.iter_mut().zip([
            sample.position_um.x,
            sample.position_um.y,
            sample.position_um.z,
        ]) {
            *sum = sum
                .checked_add(i128::from(value))
                .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "centre-of-mass sum overflow"))?;
        }
    }
    let denominator = i128::try_from(samples.len()).map_err(|_| {
        WaterError::new(NUMERIC_OVERFLOW, "centre-of-mass count conversion overflow")
    })?;
    Ok(Vec3i::new(
        round_ratio(sums[0], denominator)?,
        round_ratio(sums[1], denominator)?,
        round_ratio(sums[2], denominator)?,
    ))
}

fn round_ratio(numerator: i128, denominator: i128) -> Result<i64, WaterError> {
    let negative = numerator < 0;
    let magnitude = numerator.unsigned_abs();
    let divisor = u128::try_from(denominator).map_err(|_| {
        WaterError::new(
            NUMERIC_OVERFLOW,
            "centre-of-mass divisor conversion overflow",
        )
    })?;
    let quotient = magnitude / divisor;
    let remainder = magnitude % divisor;
    let twice_remainder = remainder
        .checked_mul(2)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "centre-of-mass remainder overflow"))?;
    let rounded =
        if twice_remainder > divisor || (twice_remainder == divisor && (quotient & 1) != 0) {
            quotient + 1
        } else {
            quotient
        };
    let signed = i128::try_from(rounded)
        .map_err(|_| WaterError::new(NUMERIC_OVERFLOW, "centre-of-mass result overflow"))?;
    let signed = if negative { -signed } else { signed };
    i64::try_from(signed)
        .map_err(|_| WaterError::new(NUMERIC_OVERFLOW, "centre-of-mass i64 overflow"))
}
