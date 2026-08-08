use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    PerformanceBaselineMetricV1, PerformanceInstrumentationV1, PerformanceMethodologyV1,
    PerformanceMetricV1, PerformanceModeV1, PerformancePreflightV1, PerformanceScenarioV1,
    PerformanceTargetFingerprintV1, PerformanceVerdict, ProcessAllocationCounterV1,
    performance_scenario_hash,
};

pub const LEGACY_PERFORMANCE_RUN_SCHEMA_VERSION: u32 = 2;
pub const LEGACY_PERFORMANCE_BASELINE_SCHEMA_VERSION: u32 = 2;
pub const LEGACY_PERFORMANCE_METHODOLOGY_VERSION: &str = "nextengine-performance-v2";

/// Decode-only representation of historical performance V2 resource evidence.
/// V2 values cannot be promoted into a V3 baseline or hard timing verdict.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceResourceCountersV2 {
    pub host_resident_bytes: Option<u64>,
    pub device_resident_bytes: Option<u64>,
    pub io_read_bytes: Option<u64>,
    pub io_write_bytes: Option<u64>,
    pub allocator_allocated_bytes: Option<u64>,
    pub allocator_allocation_count: Option<u64>,
    pub allocator_counter: Option<ProcessAllocationCounterV1>,
    pub vulkan_timestamp_queries: u64,
    pub unavailable: Vec<String>,
}

/// Decode-only historical V2 run. Current commands emit `PerformanceRunV3`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceRunV2 {
    pub schema_version: u32,
    pub commit: String,
    pub worktree_clean: bool,
    pub toolchain: String,
    pub target_triple: String,
    pub scenario: PerformanceScenarioV1,
    pub scenario_hash: String,
    pub content_hash: String,
    pub target_fingerprint: Option<PerformanceTargetFingerprintV1>,
    pub preflight: Option<PerformancePreflightV1>,
    pub build_profile: String,
    pub mode: PerformanceModeV1,
    pub instrumentation: PerformanceInstrumentationV1,
    pub resource_counters: PerformanceResourceCountersV2,
    pub methodology: PerformanceMethodologyV1,
    pub metrics: Vec<PerformanceMetricV1>,
    pub authoritative_hashes: BTreeMap<String, String>,
    pub verdict: PerformanceVerdict,
    pub diagnostics: Vec<String>,
}

impl PerformanceRunV2 {
    pub fn validate_historical_wire(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != LEGACY_PERFORMANCE_RUN_SCHEMA_VERSION {
            diagnostics.push("PERF_V2_RUN_SCHEMA_MISMATCH".to_owned());
        }
        if self.scenario_hash != performance_scenario_hash(self.scenario) {
            diagnostics.push("PERF_V2_RUN_SCENARIO_HASH_MISMATCH".to_owned());
        }
        if self.methodology.methodology_version != LEGACY_PERFORMANCE_METHODOLOGY_VERSION {
            diagnostics.push("PERF_V2_RUN_METHODOLOGY_MISMATCH".to_owned());
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
}

/// Decode-only historical V2 baseline. It is never accepted by a V3 gate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceBaselineV2 {
    pub schema_version: u32,
    pub methodology_version: String,
    pub source_commit: String,
    pub toolchain: String,
    pub target_triple: String,
    pub scenario: PerformanceScenarioV1,
    pub scenario_hash: String,
    pub content_hash: String,
    pub target_fingerprint: PerformanceTargetFingerprintV1,
    pub build_profile: String,
    pub calibration_runs: u32,
    pub metrics: Vec<PerformanceBaselineMetricV1>,
    pub authoritative_hashes: BTreeMap<String, String>,
}

impl PerformanceBaselineV2 {
    pub fn validate_historical_wire(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != LEGACY_PERFORMANCE_BASELINE_SCHEMA_VERSION {
            diagnostics.push("PERF_V2_BASELINE_SCHEMA_MISMATCH".to_owned());
        }
        if self.methodology_version != LEGACY_PERFORMANCE_METHODOLOGY_VERSION {
            diagnostics.push("PERF_V2_BASELINE_METHODOLOGY_MISMATCH".to_owned());
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
}
