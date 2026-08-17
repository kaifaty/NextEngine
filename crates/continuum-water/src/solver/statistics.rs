#![forbid(unsafe_code)]

use super::heap_error;
use crate::error::{NONFINITE_VALUE, NUMERIC_OVERFLOW, WaterError};
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
