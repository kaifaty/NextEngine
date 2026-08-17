#![forbid(unsafe_code)]

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WaterError {
    code: &'static str,
    detail: String,
}

impl WaterError {
    pub(crate) fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub(crate) fn code(&self) -> &'static str {
        self.code
    }

    pub(crate) fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for WaterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.detail.is_empty() {
            formatter.write_str(self.code)
        } else {
            write!(formatter, "{}: {}", self.code, self.detail)
        }
    }
}

impl std::error::Error for WaterError {}

pub(crate) const PROFILE_MISMATCH: &str = "WATER_PROFILE_MISMATCH";
pub(crate) const FLOAT_ENVIRONMENT_MISMATCH: &str = "WATER_FLOAT_ENVIRONMENT_MISMATCH";
pub(crate) const SCENARIO_INVALID: &str = "WATER_SCENARIO_INVALID";
pub(crate) const DUPLICATE_SAMPLE_ID: &str = "WATER_DUPLICATE_SAMPLE_ID";
pub(crate) const SAMPLE_CAPACITY_EXCEEDED: &str = "WATER_SAMPLE_CAPACITY_EXCEEDED";
pub(crate) const BOUNDARY_CAPACITY_EXCEEDED: &str = "WATER_BOUNDARY_CAPACITY_EXCEEDED";
pub(crate) const NEIGHBOR_CAPACITY_EXCEEDED: &str = "WATER_NEIGHBOR_CAPACITY_EXCEEDED";
pub(crate) const BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED: &str =
    "WATER_BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED";
pub(crate) const STEP_CAPACITY_EXCEEDED: &str = "WATER_STEP_CAPACITY_EXCEEDED";
pub(crate) const REPORT_CAPACITY_EXCEEDED: &str = "WATER_REPORT_CAPACITY_EXCEEDED";
pub(crate) const REFERENCE_INPUT_CAPACITY_EXCEEDED: &str =
    "WATER_REFERENCE_INPUT_CAPACITY_EXCEEDED";
pub(crate) const DECODED_HEAP_CAPACITY_EXCEEDED: &str = "WATER_DECODED_HEAP_CAPACITY_EXCEEDED";
pub(crate) const NONFINITE_VALUE: &str = "WATER_NONFINITE_VALUE";
pub(crate) const NUMERIC_OVERFLOW: &str = "WATER_NUMERIC_OVERFLOW";
pub(crate) const DIVERGENCE_NONCONVERGENCE: &str = "WATER_DIVERGENCE_NONCONVERGENCE";
pub(crate) const DENSITY_NONCONVERGENCE: &str = "WATER_DENSITY_NONCONVERGENCE";
pub(crate) const BOUNDARY_PENETRATION_LIMIT: &str = "WATER_BOUNDARY_PENETRATION_LIMIT";
pub(crate) const BOUNDARY_ESCAPE: &str = "WATER_BOUNDARY_ESCAPE";
pub(crate) const INVARIANT_MISMATCH: &str = "WATER_INVARIANT_MISMATCH";
pub(crate) const REFERENCE_CORPUS_MISMATCH: &str = "WATER_REFERENCE_CORPUS_MISMATCH";
pub(crate) const NONDETERMINISTIC_RESULT: &str = "WATER_NONDETERMINISTIC_RESULT";
