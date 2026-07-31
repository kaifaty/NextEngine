#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod host;
pub use host::{
    finish_process_counters, inspect_current_host, inspect_process_counters,
    validate_thoth_fingerprint,
};
#[cfg(test)]
mod tests;

pub const PERFORMANCE_RUN_SCHEMA_VERSION: u32 = 1;
pub const PERFORMANCE_BASELINE_SCHEMA_VERSION: u32 = 1;
pub const PERFORMANCE_METHODOLOGY_VERSION: &str = "nextengine-performance-v1";
pub const THOTH_TARGET_ID: &str = "ref-win-thoth-v1";
pub const MINIMUM_FREE_RAM_BYTES: u64 = 20 * 1024 * 1024 * 1024;
pub const MAX_PROFILER_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_SPANS_PER_THREAD: u32 = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PerformanceScenarioV1 {
    #[serde(rename = "smoke")]
    Smoke,
    #[serde(rename = "long-session-soak")]
    LongSessionSoak,
    #[serde(rename = "interactive-frame-soak")]
    InteractiveFrameSoak,
    #[serde(rename = "r2-alpha-render")]
    R2AlphaRender,
    #[serde(rename = "r3-multiregion-streaming")]
    R3MultiregionStreaming,
    #[serde(rename = "r4-100npc")]
    R4_100Npc,
    #[serde(rename = "r5-physics-16")]
    R5Physics16,
}

impl PerformanceScenarioV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::LongSessionSoak => "long-session-soak",
            Self::InteractiveFrameSoak => "interactive-frame-soak",
            Self::R2AlphaRender => "r2-alpha-render",
            Self::R3MultiregionStreaming => "r3-multiregion-streaming",
            Self::R4_100Npc => "r4-100npc",
            Self::R5Physics16 => "r5-physics-16",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "smoke" => Ok(Self::Smoke),
            "long-session-soak" => Ok(Self::LongSessionSoak),
            "interactive-frame-soak" => Ok(Self::InteractiveFrameSoak),
            "r2-alpha-render" => Ok(Self::R2AlphaRender),
            "r3-multiregion-streaming" => Ok(Self::R3MultiregionStreaming),
            "r4-100npc" => Ok(Self::R4_100Npc),
            "r5-physics-16" => Ok(Self::R5Physics16),
            _ => Err(format!("unknown performance scenario: {value}")),
        }
    }

    pub const fn unavailable_reason(self) -> Option<&'static str> {
        match self {
            Self::Smoke | Self::LongSessionSoak | Self::InteractiveFrameSoak => None,
            Self::R2AlphaRender => Some(
                "R2_ALPHA_PROJECT_UNAVAILABLE: the representative alpha project and 60-second semantic action windows are not implemented",
            ),
            Self::R3MultiregionStreaming => Some(
                "R3_STREAMING_WORKLOAD_UNAVAILABLE: the production four-region/64-chunk job and residency substrate is not implemented",
            ),
            Self::R4_100Npc => Some(
                "R4_100NPC_WORKLOAD_UNAVAILABLE: population, navigation and integrated ADR-016 workload owners are not implemented",
            ),
            Self::R5Physics16 => Some(
                "R5_PHYSICS_WORKLOAD_UNAVAILABLE: the 16-avatar production motor/animation workload is not implemented",
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceModeV1 {
    Report,
    Gate,
}

impl PerformanceModeV1 {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "report" => Ok(Self::Report),
            "gate" => Ok(Self::Gate),
            _ => Err(format!("unknown performance mode: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PerformanceVerdict {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "WARNING")]
    Warning,
    #[serde(rename = "REPORT_ONLY")]
    ReportOnly,
    #[serde(rename = "NOT_RUN")]
    NotRun,
}

impl PerformanceVerdict {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Warning => "WARNING",
            Self::ReportOnly => "REPORT_ONLY",
            Self::NotRun => "NOT_RUN",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceTargetFingerprintV1 {
    pub target_id: String,
    pub hostname: String,
    pub cpu_model: String,
    pub physical_cores: u32,
    pub logical_threads: u32,
    pub gpu_model: String,
    pub gpu_vram_mib: u64,
    pub ram_bytes: u64,
    pub storage_model: String,
    pub storage_bytes: u64,
    pub os_name: String,
    pub os_build: String,
    pub bios_version: String,
    pub gpu_driver: String,
    pub power_plan: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformancePreflightV1 {
    pub cpu_load_percent: Option<u32>,
    pub gpu_load_percent: Option<u32>,
    pub free_ram_bytes: Option<u64>,
    pub cpu_clock_percent_of_maximum: Option<u32>,
    pub gpu_thermal_slowdown_active: Option<bool>,
    pub ready: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceSpanV1 {
    pub category: String,
    pub thread_index: u32,
    pub duration_microseconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceInstrumentationV1 {
    pub enabled: bool,
    pub max_threads: u32,
    pub max_spans_per_thread: u32,
    pub reserved_bytes: u64,
    pub recorded_spans: Vec<PerformanceSpanV1>,
    pub dropped_spans: u64,
    pub unowned_spans: u64,
    pub overhead_basis_points: Option<i64>,
    pub authoritative_hash_parity: Option<bool>,
}

impl PerformanceInstrumentationV1 {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            max_threads: 32,
            max_spans_per_thread: MAX_SPANS_PER_THREAD,
            reserved_bytes: 0,
            recorded_spans: Vec::new(),
            dropped_spans: 0,
            unowned_spans: 0,
            overhead_basis_points: None,
            authoritative_hash_parity: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.max_threads == 0 || self.max_spans_per_thread == 0 {
            return Err("PERF_PROFILER_BUFFER_INVALID".to_owned());
        }
        if self.reserved_bytes > MAX_PROFILER_BYTES {
            return Err("PERF_PROFILER_MEMORY_EXCEEDED".to_owned());
        }
        if self.dropped_spans != 0 {
            return Err("PERF_DROPPED_SPANS".to_owned());
        }
        if self.unowned_spans != 0 {
            return Err("UNOWNED_GAMEPLAY_SPAN".to_owned());
        }
        if self.authoritative_hash_parity == Some(false) {
            return Err("PERF_PROFILER_AUTHORITY_DIVERGED".to_owned());
        }
        if self.overhead_basis_points.is_some_and(|value| value > 300) {
            return Err("PERF_PROFILER_OVERHEAD_EXCEEDED".to_owned());
        }
        let mut spans_per_thread = BTreeMap::<u32, u32>::new();
        for span in &self.recorded_spans {
            if span.thread_index >= self.max_threads {
                return Err("UNOWNED_GAMEPLAY_SPAN".to_owned());
            }
            if !matches!(
                span.category.as_str(),
                "runtime-stages"
                    | "render-extraction"
                    | "physics-motor"
                    | "streaming-io"
                    | "agent-planning"
                    | "navigation"
            ) {
                return Err("UNOWNED_GAMEPLAY_SPAN".to_owned());
            }
            let count = spans_per_thread.entry(span.thread_index).or_default();
            *count = count
                .checked_add(1)
                .ok_or_else(|| "PERF_PROFILER_SPAN_COUNT_OVERFLOW".to_owned())?;
            if *count > self.max_spans_per_thread {
                return Err("PERF_DROPPED_SPANS".to_owned());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceResourceCountersV1 {
    pub host_resident_bytes: Option<u64>,
    pub device_resident_bytes: Option<u64>,
    pub io_read_bytes: Option<u64>,
    pub io_write_bytes: Option<u64>,
    pub allocator_allocated_bytes: Option<u64>,
    pub allocator_allocation_count: Option<u64>,
    pub vulkan_timestamp_queries: u64,
    pub unavailable: Vec<String>,
}

impl PerformanceResourceCountersV1 {
    pub fn validate_for_hard_timing(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        for (name, value) in [
            ("host_resident_bytes", self.host_resident_bytes),
            ("device_resident_bytes", self.device_resident_bytes),
            ("io_read_bytes", self.io_read_bytes),
            ("io_write_bytes", self.io_write_bytes),
            ("allocator_allocated_bytes", self.allocator_allocated_bytes),
            (
                "allocator_allocation_count",
                self.allocator_allocation_count,
            ),
        ] {
            if value.is_none() {
                diagnostics.push(format!("PERF_REQUIRED_COUNTER_MISSING: {name}"));
            }
        }
        if self.vulkan_timestamp_queries == 0 {
            diagnostics.push("PERF_REQUIRED_COUNTER_MISSING: vulkan_timestamp_queries".to_owned());
        }
        if !self.unavailable.is_empty() {
            diagnostics.push("PERF_REQUIRED_COUNTER_UNAVAILABLE".to_owned());
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceBudgetV1 {
    pub p95_max: Option<u64>,
    pub p99_max: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceRelativeComparisonV1 {
    /// 100 basis points equal 1%.
    pub change_basis_points: i64,
    /// Deterministic percentile-bootstrap 95% interval, in basis points.
    pub confidence_interval_95_basis_points: [i64; 2],
    pub baseline_p95: u64,
    pub baseline_p99: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceMetricV1 {
    pub name: String,
    pub unit: String,
    pub raw_samples: Vec<u64>,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub absolute_budget: Option<PerformanceBudgetV1>,
    pub relative: Option<PerformanceRelativeComparisonV1>,
    pub verdict: PerformanceVerdict,
}

impl PerformanceMetricV1 {
    pub fn from_samples(
        name: impl Into<String>,
        unit: impl Into<String>,
        raw_samples: Vec<u64>,
        absolute_budget: Option<PerformanceBudgetV1>,
    ) -> Result<Self, String> {
        if raw_samples.is_empty() {
            return Err("performance metric requires at least one raw sample".to_owned());
        }
        let p50 = nearest_rank_percentile(&raw_samples, 50)?;
        let p95 = nearest_rank_percentile(&raw_samples, 95)?;
        let p99 = nearest_rank_percentile(&raw_samples, 99)?;
        let verdict = match &absolute_budget {
            Some(budget)
                if budget.p95_max.is_some_and(|maximum| p95 > maximum)
                    || budget.p99_max.is_some_and(|maximum| p99 > maximum) =>
            {
                PerformanceVerdict::Fail
            }
            Some(_) => PerformanceVerdict::Pass,
            None => PerformanceVerdict::ReportOnly,
        };
        Ok(Self {
            name: name.into(),
            unit: unit.into(),
            raw_samples,
            p50,
            p95,
            p99,
            absolute_budget,
            relative: None,
            verdict,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceBaselineMetricV1 {
    pub name: String,
    pub unit: String,
    pub raw_samples: Vec<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceBaselineV1 {
    pub schema_version: u32,
    pub methodology_version: String,
    pub source_commit: String,
    pub toolchain: String,
    pub scenario: PerformanceScenarioV1,
    pub scenario_hash: String,
    pub content_hash: String,
    pub target_fingerprint: PerformanceTargetFingerprintV1,
    pub build_profile: String,
    pub calibration_runs: u32,
    pub metrics: Vec<PerformanceBaselineMetricV1>,
    pub authoritative_hashes: BTreeMap<String, String>,
}

impl PerformanceBaselineV1 {
    pub fn from_runs(runs: &[PerformanceRunV1]) -> Result<Self, Vec<String>> {
        if runs.len() != 10 {
            return Err(vec!["PERF_BASELINE_REQUIRES_TEN_RUNS".to_owned()]);
        }
        let first = &runs[0];
        let Some(fingerprint) = &first.target_fingerprint else {
            return Err(vec!["PERF_BASELINE_FINGERPRINT_MISSING".to_owned()]);
        };
        let mut diagnostics = Vec::new();
        let mut metrics: BTreeMap<String, (String, Vec<u64>)> = BTreeMap::new();
        let mut expected_metrics = BTreeMap::new();
        for metric in &first.metrics {
            if expected_metrics
                .insert(metric.name.clone(), metric.unit.clone())
                .is_some()
            {
                diagnostics.push(format!("PERF_BASELINE_DUPLICATE_METRIC: {}", metric.name));
            }
        }
        for (index, run) in runs.iter().enumerate() {
            if !run.worktree_clean {
                diagnostics.push(format!("PERF_BASELINE_RUN_DIRTY: {index}"));
            }
            if run.mode != PerformanceModeV1::Report
                || matches!(
                    run.verdict,
                    PerformanceVerdict::Fail | PerformanceVerdict::NotRun
                )
                || !run.diagnostics.is_empty()
            {
                diagnostics.push(format!("PERF_BASELINE_RUN_INVALID: {index}"));
            }
            if run.schema_version != PERFORMANCE_RUN_SCHEMA_VERSION
                || run.commit == "UNKNOWN"
                || run.commit != first.commit
                || run.toolchain != first.toolchain
                || run.scenario != first.scenario
                || run.scenario_hash != first.scenario_hash
                || run.content_hash != first.content_hash
                || run.target_fingerprint.as_ref() != Some(fingerprint)
                || run.build_profile != "release"
                || run.methodology != first.methodology
                || run.authoritative_hashes != first.authoritative_hashes
            {
                diagnostics.push(format!("PERF_BASELINE_RUN_INCOMPATIBLE: {index}"));
            }
            if run
                .preflight
                .as_ref()
                .is_none_or(|preflight| !preflight.ready)
            {
                diagnostics.push(format!("PERF_BASELINE_PREFLIGHT_NOT_READY: {index}"));
            }
            let fingerprint_diagnostics = run
                .target_fingerprint
                .as_ref()
                .map(validate_thoth_fingerprint)
                .unwrap_or_else(|| vec!["PERF_BASELINE_FINGERPRINT_MISSING".to_owned()]);
            for diagnostic in fingerprint_diagnostics {
                diagnostics.push(format!("PERF_BASELINE_RUN_INVALID: {index}: {diagnostic}"));
            }
            if let Err(error) = run.instrumentation.validate() {
                diagnostics.push(format!("PERF_BASELINE_RUN_INVALID: {index}: {error}"));
            }
            if !run.instrumentation.enabled {
                diagnostics.push(format!(
                    "PERF_BASELINE_RUN_INVALID: {index}: PERF_PROFILER_DISABLED"
                ));
            }
            if let Err(errors) = run.resource_counters.validate_for_hard_timing() {
                for error in errors {
                    diagnostics.push(format!("PERF_BASELINE_RUN_INVALID: {index}: {error}"));
                }
            }
            let mut observed_metrics = BTreeMap::new();
            for metric in &run.metrics {
                if observed_metrics
                    .insert(metric.name.clone(), metric.unit.clone())
                    .is_some()
                {
                    diagnostics.push(format!("PERF_BASELINE_DUPLICATE_METRIC: {}", metric.name));
                    continue;
                }
                match metrics.get_mut(&metric.name) {
                    Some((unit, samples)) if *unit == metric.unit => {
                        samples.extend_from_slice(&metric.raw_samples);
                    }
                    Some(_) => {
                        diagnostics.push(format!("PERF_BASELINE_UNIT_MISMATCH: {}", metric.name))
                    }
                    None => {
                        metrics.insert(
                            metric.name.clone(),
                            (metric.unit.clone(), metric.raw_samples.clone()),
                        );
                    }
                }
            }
            if observed_metrics != expected_metrics {
                diagnostics.push(format!("PERF_BASELINE_METRIC_SET_MISMATCH: {index}"));
            }
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        Ok(Self {
            schema_version: PERFORMANCE_BASELINE_SCHEMA_VERSION,
            methodology_version: PERFORMANCE_METHODOLOGY_VERSION.to_owned(),
            source_commit: first.commit.clone(),
            toolchain: first.toolchain.clone(),
            scenario: first.scenario,
            scenario_hash: first.scenario_hash.clone(),
            content_hash: first.content_hash.clone(),
            target_fingerprint: fingerprint.clone(),
            build_profile: "release".to_owned(),
            calibration_runs: 10,
            metrics: metrics
                .into_iter()
                .map(|(name, (unit, raw_samples))| PerformanceBaselineMetricV1 {
                    name,
                    unit,
                    raw_samples,
                })
                .collect(),
            authoritative_hashes: first.authoritative_hashes.clone(),
        })
    }

    pub fn validate_for(
        &self,
        run: &PerformanceRunV1,
    ) -> Result<BTreeMap<&str, &PerformanceBaselineMetricV1>, Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != PERFORMANCE_BASELINE_SCHEMA_VERSION {
            diagnostics.push("PERF_BASELINE_SCHEMA_MISMATCH".to_owned());
        }
        if self.methodology_version != PERFORMANCE_METHODOLOGY_VERSION
            || self.methodology_version != run.methodology.methodology_version
        {
            diagnostics.push("PERF_BASELINE_METHODOLOGY_MISMATCH".to_owned());
        }
        if self.calibration_runs != 10 {
            diagnostics.push("PERF_BASELINE_REQUIRES_TEN_RUNS".to_owned());
        }
        if self.scenario != run.scenario || self.scenario_hash != run.scenario_hash {
            diagnostics.push("PERF_BASELINE_SCENARIO_MISMATCH".to_owned());
        }
        if self.content_hash != run.content_hash {
            diagnostics.push("PERF_BASELINE_CONTENT_MISMATCH".to_owned());
        }
        if self.build_profile != "release" || self.build_profile != run.build_profile {
            diagnostics.push("PERF_BASELINE_BUILD_PROFILE_MISMATCH".to_owned());
        }
        if self.toolchain != run.toolchain {
            diagnostics.push("PERF_BASELINE_TOOLCHAIN_MISMATCH".to_owned());
        }
        diagnostics.extend(validate_thoth_fingerprint(&self.target_fingerprint));
        if run.target_fingerprint.as_ref() != Some(&self.target_fingerprint) {
            diagnostics.push("PERF_BASELINE_FINGERPRINT_MISMATCH".to_owned());
        }
        if run.authoritative_hashes != self.authoritative_hashes {
            diagnostics.push("PERF_AUTHORITATIVE_HASH_DIVERGED".to_owned());
        }
        let mut metrics = BTreeMap::new();
        for metric in &self.metrics {
            if metric.raw_samples.is_empty() {
                diagnostics.push(format!("PERF_BASELINE_EMPTY_METRIC: {}", metric.name));
            }
            if metrics.insert(metric.name.as_str(), metric).is_some() {
                diagnostics.push(format!("PERF_BASELINE_DUPLICATE_METRIC: {}", metric.name));
            }
        }
        if diagnostics.is_empty() {
            Ok(metrics)
        } else {
            Err(diagnostics)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceMethodologyV1 {
    pub methodology_version: String,
    pub warmup_samples: u64,
    pub measured_samples: u64,
    pub percentile_method: String,
    pub outlier_policy: String,
    pub frame_critical_path: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceRunV1 {
    pub schema_version: u32,
    pub commit: String,
    pub worktree_clean: bool,
    pub toolchain: String,
    pub scenario: PerformanceScenarioV1,
    pub scenario_hash: String,
    pub content_hash: String,
    pub target_fingerprint: Option<PerformanceTargetFingerprintV1>,
    pub preflight: Option<PerformancePreflightV1>,
    pub build_profile: String,
    pub mode: PerformanceModeV1,
    pub instrumentation: PerformanceInstrumentationV1,
    pub resource_counters: PerformanceResourceCountersV1,
    pub methodology: PerformanceMethodologyV1,
    pub metrics: Vec<PerformanceMetricV1>,
    pub authoritative_hashes: BTreeMap<String, String>,
    pub verdict: PerformanceVerdict,
    pub diagnostics: Vec<String>,
}

impl PerformanceRunV1 {
    pub fn empty(
        scenario: PerformanceScenarioV1,
        mode: PerformanceModeV1,
        build_profile: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: PERFORMANCE_RUN_SCHEMA_VERSION,
            commit: "UNKNOWN".to_owned(),
            worktree_clean: false,
            toolchain: "UNKNOWN".to_owned(),
            scenario,
            scenario_hash: sha256_hex(scenario.as_str().as_bytes()),
            content_hash: sha256_hex(b"nextengine.content.unavailable.v1"),
            target_fingerprint: None,
            preflight: None,
            build_profile: build_profile.into(),
            mode,
            instrumentation: PerformanceInstrumentationV1::disabled(),
            resource_counters: PerformanceResourceCountersV1::default(),
            methodology: methodology_for(scenario),
            metrics: Vec::new(),
            authoritative_hashes: BTreeMap::new(),
            verdict: PerformanceVerdict::NotRun,
            diagnostics: Vec::new(),
        }
    }
}

pub fn nearest_rank_percentile(samples: &[u64], percentile: u32) -> Result<u64, String> {
    if samples.is_empty() {
        return Err("nearest-rank percentile requires samples".to_owned());
    }
    if !(1..=100).contains(&percentile) {
        return Err("nearest-rank percentile must be in 1..=100".to_owned());
    }
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let numerator = usize::try_from(percentile)
        .map_err(|error| error.to_string())?
        .checked_mul(ordered.len())
        .ok_or_else(|| "nearest-rank index overflow".to_owned())?;
    let rank = numerator.div_ceil(100);
    Ok(ordered[rank.saturating_sub(1)])
}

pub fn compare_metrics_to_baseline(
    run: &mut PerformanceRunV1,
    baseline: &PerformanceBaselineV1,
) -> Result<(), Vec<String>> {
    let baseline_metrics = baseline.validate_for(run)?;
    let mut diagnostics = Vec::new();
    for metric in &mut run.metrics {
        let Some(reference) = baseline_metrics.get(metric.name.as_str()) else {
            diagnostics.push(format!("PERF_BASELINE_METRIC_MISSING: {}", metric.name));
            continue;
        };
        if reference.unit != metric.unit {
            diagnostics.push(format!("PERF_BASELINE_UNIT_MISMATCH: {}", metric.name));
            continue;
        }
        let baseline_p95 =
            nearest_rank_percentile(&reference.raw_samples, 95).map_err(|error| vec![error])?;
        let baseline_p99 =
            nearest_rank_percentile(&reference.raw_samples, 99).map_err(|error| vec![error])?;
        let change_basis_points = relative_change_basis_points(metric.p95, baseline_p95);
        let confidence_interval_95_basis_points =
            bootstrap_change_interval(&metric.raw_samples, &reference.raw_samples, 2_000)
                .map_err(|error| vec![error])?;
        metric.relative = Some(PerformanceRelativeComparisonV1 {
            change_basis_points,
            confidence_interval_95_basis_points,
            baseline_p95,
            baseline_p99,
        });
        metric.verdict = relative_verdict(
            metric.verdict,
            change_basis_points,
            confidence_interval_95_basis_points,
        );
    }
    if diagnostics.is_empty() {
        run.verdict = aggregate_metric_verdict(&run.metrics);
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub fn aggregate_metric_verdict(metrics: &[PerformanceMetricV1]) -> PerformanceVerdict {
    if metrics
        .iter()
        .any(|metric| metric.verdict == PerformanceVerdict::Fail)
    {
        PerformanceVerdict::Fail
    } else if metrics
        .iter()
        .any(|metric| metric.verdict == PerformanceVerdict::Warning)
    {
        PerformanceVerdict::Warning
    } else if metrics
        .iter()
        .any(|metric| metric.verdict == PerformanceVerdict::Pass)
    {
        PerformanceVerdict::Pass
    } else {
        PerformanceVerdict::ReportOnly
    }
}

pub fn methodology_for(scenario: PerformanceScenarioV1) -> PerformanceMethodologyV1 {
    let mut methodology = PerformanceMethodologyV1 {
        methodology_version: PERFORMANCE_METHODOLOGY_VERSION.to_owned(),
        warmup_samples: 0,
        measured_samples: 0,
        percentile_method: "nearest-rank".to_owned(),
        outlier_policy: "retain-all-samples".to_owned(),
        frame_critical_path: None,
        notes: Vec::new(),
    };
    match scenario {
        PerformanceScenarioV1::Smoke => {
            methodology.measured_samples = 4;
            methodology.notes = vec![
                "two-chunk streaming, five-object render planning, one-agent planning and live movement are smoke fixtures only".to_owned(),
                "aggregate smoke timings are report-only and cannot close B-12".to_owned(),
            ];
        }
        PerformanceScenarioV1::LongSessionSoak => {
            methodology.measured_samples = 3;
            methodology.notes = vec![
                "3,600 live ticks in three 1,200-tick windows with held movement and periodic camera input run through both the live driver and interactive application scheduler".to_owned(),
                "identity-index and command-body archive roots are recomputed after each window outside the window timing".to_owned(),
                "application checkpoint samples include the mandatory 30-tick durable publication path and reuse validated canonical component bytes".to_owned(),
                "report-only granular samples separate driver prepare, infallible driver commit, checkpoint materialization, ordinary application ticks, and checkpoint application ticks".to_owned(),
                "application input is staged before timing and each measured host pump advances exactly one 30 Hz fixed step".to_owned(),
                "the soak is report-only and diagnoses history-dependent degradation; it cannot close B-12".to_owned(),
            ];
        }
        PerformanceScenarioV1::InteractiveFrameSoak => {
            methodology.measured_samples = 240;
            methodology.frame_critical_path =
                Some("max(cpu_extract_and_submit_us,gpu_timestamp_duration_us)".to_owned());
            methodology.notes = vec![
                "240 FIFO-presented frames use the production Vulkan frame path at requested 1920x1080 and immutable reference-game render inputs".to_owned(),
                "phase timings separate event polling plus immutable frame-source update, frame-slot/acquire/image waits, frame-plan, command recording, submit, present and GPU execution".to_owned(),
                "the static render-input fixture does not time the game composition root's main-to-simulation-worker handoff; that remains a separate production-worker diagnostic gap".to_owned(),
                "the workload is report-only and diagnostic; it is not the representative R2 alpha project and cannot close B-12".to_owned(),
            ];
        }
        PerformanceScenarioV1::R2AlphaRender => {
            methodology.warmup_samples = 600 * 3;
            methodology.measured_samples = 3_600 * 3;
            methodology.frame_critical_path =
                Some("max(cpu_extract_and_submit_us,gpu_timestamp_duration_us)".to_owned());
            methodology.notes = vec![
                "three 60-second windows: exploration, combat and UI/dialogue".to_owned(),
                "VSync wait excluded; missed deadlines counted separately".to_owned(),
            ];
        }
        PerformanceScenarioV1::R3MultiregionStreaming => {
            methodology.measured_samples = 1_000;
            methodology.notes = vec![
                "four regions, 64 chunks and approximately 150% of the residency budget".to_owned(),
                "worker permutations 1/2/8/16 require identical authoritative roots".to_owned(),
            ];
        }
        PerformanceScenarioV1::R4_100Npc => {
            methodology.warmup_samples = 1_000;
            methodology.measured_samples = 10_000;
            methodology.notes = vec![
                "ADR-016 integrated 100-NPC workload at 30 Hz".to_owned(),
                "all stage rows are exclusive; unowned time invalidates the run".to_owned(),
            ];
        }
        PerformanceScenarioV1::R5Physics16 => {
            methodology.measured_samples = 10_000;
            methodology.notes = vec![
                "16 avatars, physics at 120 Hz and motor at 60 Hz".to_owned(),
                "fixed reduction order and authoritative replay parity are required".to_owned(),
            ];
        }
    }
    methodology
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(value, "{byte:02x}");
    }
    value
}

fn relative_change_basis_points(current: u64, baseline: u64) -> i64 {
    if baseline == 0 {
        return if current == 0 { 0 } else { i64::MAX };
    }
    let current = i128::from(current);
    let baseline = i128::from(baseline);
    let value = (current - baseline)
        .saturating_mul(10_000)
        .checked_div(baseline)
        .unwrap_or(i128::from(i64::MAX));
    i64::try_from(value).unwrap_or_else(|_| {
        if value.is_negative() {
            i64::MIN
        } else {
            i64::MAX
        }
    })
}

fn relative_verdict(
    absolute: PerformanceVerdict,
    change_basis_points: i64,
    confidence_interval: [i64; 2],
) -> PerformanceVerdict {
    if absolute == PerformanceVerdict::Fail
        || (change_basis_points >= 500 && confidence_interval[0] >= 500)
    {
        PerformanceVerdict::Fail
    } else if change_basis_points >= 200 {
        PerformanceVerdict::Warning
    } else if absolute == PerformanceVerdict::Pass {
        PerformanceVerdict::Pass
    } else {
        PerformanceVerdict::ReportOnly
    }
}

fn bootstrap_change_interval(
    current: &[u64],
    baseline: &[u64],
    iterations: usize,
) -> Result<[i64; 2], String> {
    if current.is_empty() || baseline.is_empty() || iterations == 0 {
        return Err("bootstrap requires non-empty samples and iterations".to_owned());
    }
    let mut rng = XorShift64::new(0x4e45_5854_5045_5246);
    let mut current_resample = vec![0_u64; current.len()];
    let mut baseline_resample = vec![0_u64; baseline.len()];
    let mut changes = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        for sample in &mut current_resample {
            *sample = current[rng.index(current.len())];
        }
        for sample in &mut baseline_resample {
            *sample = baseline[rng.index(baseline.len())];
        }
        let current_p95 = nearest_rank_percentile(&current_resample, 95)?;
        let baseline_p95 = nearest_rank_percentile(&baseline_resample, 95)?;
        changes.push(relative_change_basis_points(current_p95, baseline_p95));
    }
    changes.sort_unstable();
    Ok([
        nearest_rank_fraction_i64(&changes, 25, 1_000)?,
        nearest_rank_fraction_i64(&changes, 975, 1_000)?,
    ])
}

fn nearest_rank_fraction_i64(
    samples: &[i64],
    numerator: usize,
    denominator: usize,
) -> Result<i64, String> {
    if samples.is_empty() || numerator == 0 || numerator > denominator {
        return Err("invalid signed nearest-rank fraction input".to_owned());
    }
    let rank_numerator = numerator
        .checked_mul(samples.len())
        .ok_or_else(|| "signed nearest-rank index overflow".to_owned())?;
    let rank = rank_numerator.div_ceil(denominator);
    Ok(samples[rank.saturating_sub(1)])
}

struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn index(&mut self, length: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(length).unwrap_or(u64::MAX)).unwrap_or(0)
    }
}
