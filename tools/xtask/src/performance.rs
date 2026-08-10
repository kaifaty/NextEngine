#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

mod host;
pub use host::{
    finish_process_counters, inspect_current_host, inspect_process_counters,
    validate_thoth_fingerprint,
};
mod hard_evidence;
mod resource_counters;
mod support;
pub use resource_counters::{PerformanceLogicalResourceChargesV1, PerformanceResourceCountersV4};
use support::{bootstrap_change_interval, relative_change_basis_points, relative_verdict};
pub use support::{methodology_for, sha256_hex};
#[cfg(test)]
mod tests;

pub const PERFORMANCE_RUN_SCHEMA_VERSION: u32 = 4;
pub const PERFORMANCE_BASELINE_SCHEMA_VERSION: u32 = 4;
pub const PERFORMANCE_METHODOLOGY_VERSION: &str = "nextengine-performance-v6";
pub const PERFORMANCE_REPORT_FILE_NAME: &str = "performance-report-v4.json";
pub const PERFORMANCE_BASELINE_FILE_NAME: &str = "performance-baseline-v4.json";
pub const PERFORMANCE_REPORT_TEMP_FILE_NAME: &str = ".performance-report-v4.json.tmp";
pub const PERFORMANCE_BASELINE_TEMP_FILE_NAME: &str = ".performance-baseline-v4.json.tmp";
pub const PERFORMANCE_PINNED_RUSTC_RELEASE: &str = "1.93.0";
pub const PERFORMANCE_PINNED_RUSTC_COMMIT_HASH: &str = "254b59607d4417e9dffbc307138ae5c86280fe4c";
pub const PERFORMANCE_WINDOWS_TARGET_TRIPLE: &str = "x86_64-pc-windows-msvc";
pub const PERFORMANCE_LINUX_TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
pub const THOTH_TARGET_ID: &str = "ref-win-thoth-v1";
pub const PREFLIGHT_LOAD_PERCENT_EXCLUSIVE: u32 = 40;
pub const MINIMUM_FREE_RAM_BYTES: u64 = 10 * 1024 * 1024 * 1024;
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
    #[serde(rename = "production-worker-soak")]
    ProductionWorkerSoak,
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
            Self::ProductionWorkerSoak => "production-worker-soak",
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
            "production-worker-soak" => Ok(Self::ProductionWorkerSoak),
            "r2-alpha-render" => Ok(Self::R2AlphaRender),
            "r3-multiregion-streaming" => Ok(Self::R3MultiregionStreaming),
            "r4-100npc" => Ok(Self::R4_100Npc),
            "r5-physics-16" => Ok(Self::R5Physics16),
            _ => Err(format!("unknown performance scenario: {value}")),
        }
    }

    pub const fn unavailable_reason(self) -> Option<&'static str> {
        match self {
            Self::Smoke
            | Self::LongSessionSoak
            | Self::InteractiveFrameSoak
            | Self::ProductionWorkerSoak
            | Self::R2AlphaRender
            | Self::R3MultiregionStreaming => None,
            Self::R4_100Npc => Some(
                "R4_100NPC_WORKLOAD_UNAVAILABLE: population, navigation and integrated ADR-016 workload owners are not implemented",
            ),
            Self::R5Physics16 => Some(
                "R5_PHYSICS_WORKLOAD_UNAVAILABLE: the 16-avatar production motor/animation workload is not implemented",
            ),
        }
    }
}

pub fn performance_scenario_hash(scenario: PerformanceScenarioV1) -> String {
    let preimage: &[u8] = match scenario {
        PerformanceScenarioV1::Smoke => {
            b"nextengine.performance.smoke.v5:r3a-packaged-io:two-fixed-ticks-per-transition:bounded-workers=2:cycles=1000:five-object:one-agent:900-live-ticks:logical-staging-charge:resource-observation=streaming+agent-planning+render-planning+live-runtime"
        }
        PerformanceScenarioV1::LongSessionSoak => {
            b"nextengine.performance.long-session-soak.v5:3600-live-ticks:1200-tick-windows:held-movement:camera-every-15-ticks:driver-and-interactive-application:one-fixed-step-per-measured-pump:resource-observation=live-runtime-only"
        }
        PerformanceScenarioV1::InteractiveFrameSoak => {
            b"nextengine.performance.interactive-frame-soak.v3:240-fifo-frames:1920x1080:reference-render-inputs:phase-timings:frame-plan-cache:resource-observation=desktop-frame-workload-only"
        }
        PerformanceScenarioV1::ProductionWorkerSoak => {
            b"nextengine.performance.production-worker-soak.v3:240-fifo-main-callbacks:60hz:bounded-sync-queue:next-simulation-worker:fixed-step-application:shared-presentation-publication:main-snapshot-read:resource-observation=production-worker-diagnostic-only"
        }
        PerformanceScenarioV1::R2AlphaRender => {
            b"nextengine.performance.r2-alpha-render.v2:reference-alpha:frontier-relay:windows=exploration+combat+ui-dialogue:profiles=primary-1920x1080+fallback-b0-safe-1280x720p30:each=600-warmup+3600-measured:critical=max-cpu-extract-submit-gpu:retain-all:resource-window=sequential-six-window-production-vulkan:logical-accounting=r2-alpha-render-v1"
        }
        PerformanceScenarioV1::R3MultiregionStreaming => {
            b"nextengine.performance.r3-multiregion-streaming.v1:reference-alpha:regions=4:chunks=64:cycles=1000:canonical-cyclic-route:two-fixed-ticks-per-transition:packaged-io:bounded-workers=2:logical-staging-charge:resource-observation=streaming-only"
        }
        _ => return sha256_hex(scenario.as_str().as_bytes()),
    };
    sha256_hex(preimage)
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

    pub const fn command_report_status(self) -> &'static str {
        match self {
            Self::Pass | Self::ReportOnly => "PASS",
            Self::Fail => "FAIL",
            Self::Warning => "WARNING",
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

impl PerformancePreflightV1 {
    pub(super) fn threshold_diagnostics(&self) -> Vec<String> {
        let mut diagnostics = Vec::new();
        if self
            .cpu_load_percent
            .is_none_or(|percent| percent >= PREFLIGHT_LOAD_PERCENT_EXCLUSIVE)
        {
            diagnostics.push("PERF_CPU_LOAD_AT_OR_ABOVE_FORTY_PERCENT".to_owned());
        }
        if self
            .gpu_load_percent
            .is_none_or(|percent| percent >= PREFLIGHT_LOAD_PERCENT_EXCLUSIVE)
        {
            diagnostics.push("PERF_GPU_LOAD_AT_OR_ABOVE_FORTY_PERCENT".to_owned());
        }
        if self
            .free_ram_bytes
            .is_none_or(|bytes| bytes < MINIMUM_FREE_RAM_BYTES)
        {
            diagnostics.push("PERF_FREE_RAM_BELOW_TEN_GIB".to_owned());
        }
        if self
            .cpu_clock_percent_of_maximum
            .is_none_or(|percent| percent < 80)
        {
            diagnostics.push("PERF_CPU_THROTTLING_CHECK_FAILED".to_owned());
        }
        if self.gpu_thermal_slowdown_active != Some(false) {
            diagnostics.push("PERF_GPU_THERMAL_SLOWDOWN_CHECK_FAILED".to_owned());
        }
        diagnostics
    }

    pub(super) fn recompute_readiness(&mut self) {
        self.diagnostics = self.threshold_diagnostics();
        self.ready = self.diagnostics.is_empty();
    }

    pub fn validate_ready_evidence(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = self.threshold_diagnostics();
        if !self.ready || !self.diagnostics.is_empty() {
            diagnostics.push("PERF_PREFLIGHT_NOT_READY".to_owned());
        }
        diagnostics.sort();
        diagnostics.dedup();
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
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
        if self.enabled {
            match self.authoritative_hash_parity {
                Some(true) => {}
                Some(false) => return Err("PERF_PROFILER_AUTHORITY_DIVERGED".to_owned()),
                None => return Err("PERF_PROFILER_AUTHORITY_UNAVAILABLE".to_owned()),
            }
            match self.overhead_basis_points {
                Some(0..=300) => {}
                Some(value) if value > 300 => {
                    return Err("PERF_PROFILER_OVERHEAD_EXCEEDED".to_owned());
                }
                Some(_) => return Err("PERF_PROFILER_OVERHEAD_INVALID".to_owned()),
                None => return Err("PERF_PROFILER_OVERHEAD_UNAVAILABLE".to_owned()),
            }
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
pub struct PerformanceBaselineV4 {
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

impl PerformanceBaselineV4 {
    pub fn from_runs(runs: &[PerformanceRunV4]) -> Result<Self, Vec<String>> {
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
            if let Err(errors) = run.validate_build_provenance() {
                for error in errors {
                    diagnostics.push(format!("PERF_BASELINE_RUN_INVALID: {index}: {error}"));
                }
            }
            if run.target_triple != PERFORMANCE_WINDOWS_TARGET_TRIPLE {
                diagnostics.push(format!(
                    "PERF_BASELINE_RUN_INVALID: {index}: PERF_BASELINE_TARGET_MISMATCH"
                ));
            }
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
                || run.target_triple != first.target_triple
                || run.scenario != first.scenario
                || run.scenario_hash != first.scenario_hash
                || run.content_hash != first.content_hash
                || run.target_fingerprint.as_ref() != Some(fingerprint)
                || run.build_profile != "release"
                || run.methodology != first.methodology
                || run.methodology.methodology_version != PERFORMANCE_METHODOLOGY_VERSION
                || run.authoritative_hashes != first.authoritative_hashes
            {
                diagnostics.push(format!("PERF_BASELINE_RUN_INCOMPATIBLE: {index}"));
            }
            match run
                .preflight
                .as_ref()
                .map(PerformancePreflightV1::validate_ready_evidence)
            {
                Some(Ok(())) => {}
                Some(Err(errors)) => {
                    for error in errors {
                        diagnostics.push(format!(
                            "PERF_BASELINE_PREFLIGHT_NOT_READY: {index}: {error}"
                        ));
                    }
                }
                None => diagnostics.push(format!(
                    "PERF_BASELINE_PREFLIGHT_NOT_READY: {index}: PERF_PREFLIGHT_UNAVAILABLE"
                )),
            }
            let fingerprint_diagnostics = run
                .target_fingerprint
                .as_ref()
                .map(validate_thoth_fingerprint)
                .unwrap_or_else(|| vec!["PERF_BASELINE_FINGERPRINT_MISSING".to_owned()]);
            for diagnostic in fingerprint_diagnostics {
                diagnostics.push(format!("PERF_BASELINE_RUN_INVALID: {index}: {diagnostic}"));
            }
            if let Err(errors) = run.validate_hard_evidence() {
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
            target_triple: first.target_triple.clone(),
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
        run: &PerformanceRunV4,
    ) -> Result<BTreeMap<&str, &PerformanceBaselineMetricV1>, Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != PERFORMANCE_BASELINE_SCHEMA_VERSION {
            diagnostics.push("PERF_BASELINE_SCHEMA_MISMATCH".to_owned());
        }
        if let Err(errors) = run.validate_wire_version() {
            diagnostics.extend(errors);
        }
        if let Err(errors) = run.validate_build_provenance() {
            diagnostics.extend(errors);
        }
        if let Err(errors) = validate_performance_build_provenance(
            &self.source_commit,
            &self.toolchain,
            &self.target_triple,
        ) {
            diagnostics.extend(errors);
        }
        if self.target_triple != PERFORMANCE_WINDOWS_TARGET_TRIPLE {
            diagnostics.push("PERF_BASELINE_TARGET_MISMATCH".to_owned());
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
        if self.target_triple != run.target_triple {
            diagnostics.push("PERF_BASELINE_TARGET_MISMATCH".to_owned());
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
pub struct PerformanceRunV4 {
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
    pub resource_counters: PerformanceResourceCountersV4,
    pub methodology: PerformanceMethodologyV1,
    pub metrics: Vec<PerformanceMetricV1>,
    pub authoritative_hashes: BTreeMap<String, String>,
    pub verdict: PerformanceVerdict,
    pub diagnostics: Vec<String>,
}

impl PerformanceRunV4 {
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
            target_triple: "UNKNOWN".to_owned(),
            scenario,
            scenario_hash: performance_scenario_hash(scenario),
            content_hash: sha256_hex(b"nextengine.content.unavailable.v1"),
            target_fingerprint: None,
            preflight: None,
            build_profile: build_profile.into(),
            mode,
            instrumentation: PerformanceInstrumentationV1::disabled(),
            resource_counters: PerformanceResourceCountersV4::default(),
            methodology: methodology_for(scenario),
            metrics: Vec::new(),
            authoritative_hashes: BTreeMap::new(),
            verdict: PerformanceVerdict::NotRun,
            diagnostics: Vec::new(),
        }
    }

    pub fn validate_report_evidence(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = self.validate_wire_version().err().unwrap_or_default();
        if let Err(errors) = self.resource_counters.validate_report_evidence() {
            diagnostics.extend(errors);
        }
        diagnostics.sort();
        diagnostics.dedup();
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    pub fn validate_wire_version(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != PERFORMANCE_RUN_SCHEMA_VERSION {
            diagnostics.push("PERF_RUN_SCHEMA_MISMATCH".to_owned());
        }
        if self.scenario_hash != performance_scenario_hash(self.scenario) {
            diagnostics.push("PERF_RUN_SCENARIO_HASH_MISMATCH".to_owned());
        }
        if self.methodology != methodology_for(self.scenario) {
            diagnostics.push("PERF_RUN_METHODOLOGY_MISMATCH".to_owned());
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    pub fn validate_build_provenance(&self) -> Result<(), Vec<String>> {
        validate_performance_build_provenance(&self.commit, &self.toolchain, &self.target_triple)
    }

    pub fn validate_command_report_status(&self, status: &str) -> Result<(), String> {
        let expected = self.verdict.command_report_status();
        if status == expected {
            Ok(())
        } else {
            Err(format!(
                "PERF_COMMAND_STATUS_MISMATCH: expected {expected} for nested {}, found {status}",
                self.verdict.as_str()
            ))
        }
    }
}

pub fn validate_performance_build_provenance(
    commit: &str,
    toolchain: &str,
    target_triple: &str,
) -> Result<(), Vec<String>> {
    let mut diagnostics = Vec::new();
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        diagnostics.push("PERF_BUILD_COMMIT_INVALID".to_owned());
    }
    if !matches!(
        target_triple,
        PERFORMANCE_WINDOWS_TARGET_TRIPLE | PERFORMANCE_LINUX_TARGET_TRIPLE
    ) {
        diagnostics.push("PERF_BUILD_TARGET_UNSUPPORTED".to_owned());
    }
    let first_line = toolchain.lines().next().unwrap_or_default();
    if first_line
        != format!(
            "rustc {PERFORMANCE_PINNED_RUSTC_RELEASE} ({} 2026-01-19)",
            &PERFORMANCE_PINNED_RUSTC_COMMIT_HASH[..9]
        )
    {
        diagnostics.push("PERF_BUILD_TOOLCHAIN_IDENTITY_INVALID".to_owned());
    }
    match unique_verbose_field(toolchain, "commit-hash") {
        Some(commit_hash) if commit_hash == PERFORMANCE_PINNED_RUSTC_COMMIT_HASH => {}
        Some(_) => diagnostics.push("PERF_BUILD_RUSTC_COMMIT_MISMATCH".to_owned()),
        None => diagnostics.push("PERF_BUILD_RUSTC_COMMIT_MISSING".to_owned()),
    }
    match unique_verbose_field(toolchain, "release") {
        Some(release) if release == PERFORMANCE_PINNED_RUSTC_RELEASE => {}
        Some(_) => diagnostics.push("PERF_BUILD_RUSTC_RELEASE_MISMATCH".to_owned()),
        None => diagnostics.push("PERF_BUILD_RUSTC_RELEASE_MISSING".to_owned()),
    }
    match unique_verbose_field(toolchain, "host") {
        Some(host) if host == target_triple => {}
        Some(_) => diagnostics.push("PERF_BUILD_TOOLCHAIN_HOST_MISMATCH".to_owned()),
        None => diagnostics.push("PERF_BUILD_TOOLCHAIN_HOST_MISSING".to_owned()),
    }
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub fn decode_build_toolchain_hex(value: &str) -> Result<String, String> {
    if !value.len().is_multiple_of(2) {
        return Err("hex payload has an odd length".to_owned());
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let high = decode_hex_digit(pair[0])?;
        let low = decode_hex_digit(pair[1])?;
        bytes.push((high << 4) | low);
    }
    String::from_utf8(bytes).map_err(|error| format!("toolchain payload is not UTF-8: {error}"))
}

fn decode_hex_digit(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(format!("invalid lowercase hex digit: {value:#04x}")),
    }
}

fn unique_verbose_field<'a>(toolchain: &'a str, field: &str) -> Option<&'a str> {
    let prefix = format!("{field}: ");
    let mut values = toolchain
        .lines()
        .filter_map(|line| line.strip_prefix(&prefix));
    let value = values.next()?;
    (!value.is_empty() && values.next().is_none()).then_some(value)
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
    run: &mut PerformanceRunV4,
    baseline: &PerformanceBaselineV4,
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
