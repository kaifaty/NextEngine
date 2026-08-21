#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::performance::{
    PERFORMANCE_METHODOLOGY_VERSION, PERFORMANCE_RUN_SCHEMA_VERSION, PerformanceModeV1,
    PerformanceRunV6, PerformanceScenarioV1, PerformanceVerdict, nearest_rank_percentile,
    validate_linux_release_fingerprint,
};

mod statistics;

use statistics::{
    bootstrap_change_interval, bootstrap_combined_change_interval, mean_i64,
    relative_change_basis_points,
};

pub const CODEGEN_COMPARISON_SCHEMA_VERSION: u32 = 1;
pub const CODEGEN_BUILD_SCHEMA_VERSION: u32 = 1;
pub const CODEGEN_RUN_PROVENANCE_SCHEMA_VERSION: u32 = 1;
pub const CODEGEN_CALIBRATION_RUNS: usize = 10;
pub const CODEGEN_BOOTSTRAP_ITERATIONS: usize = 2_000;
pub const PINNED_RUSTC_RELEASE: &str = "1.97.1";
pub const CODEGEN_SCENARIOS: [PerformanceScenarioV1; 4] = [
    PerformanceScenarioV1::R2AlphaRender,
    PerformanceScenarioV1::R3MultiregionStreaming,
    PerformanceScenarioV1::R4_100Npc,
    PerformanceScenarioV1::R5Physics16,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CodegenCandidateV1 {
    #[serde(rename = "baseline")]
    Baseline,
    #[serde(rename = "thin-lto")]
    ThinLto,
    #[serde(rename = "pgo")]
    Pgo,
}

impl CodegenCandidateV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::ThinLto => "thin-lto",
            Self::Pgo => "pgo",
        }
    }

    pub const fn expected_build_profile(self) -> &'static str {
        match self {
            Self::Baseline => "release",
            Self::ThinLto => "release-thin-lto",
            Self::Pgo => "release-pgo",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "baseline" => Ok(Self::Baseline),
            "thin-lto" => Ok(Self::ThinLto),
            "pgo" => Ok(Self::Pgo),
            _ => Err(format!("unknown codegen candidate: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CodegenComparisonVerdictV1 {
    #[serde(rename = "REPORT_ONLY")]
    ReportOnly,
    #[serde(rename = "NOT_RUN")]
    NotRun,
}

impl CodegenComparisonVerdictV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReportOnly => "REPORT_ONLY",
            Self::NotRun => "NOT_RUN",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodegenProfileOverrideV1 {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodegenBuildProvenanceV1 {
    pub schema_version: u32,
    pub candidate: CodegenCandidateV1,
    pub build_profile: String,
    pub commit: String,
    pub worktree_clean: bool,
    pub executable_sha256: String,
    pub toolchain: String,
    pub opt_level: String,
    pub effective_rustflags: Vec<String>,
    pub profile_overrides: Vec<CodegenProfileOverrideV1>,
    pub codegen_mode: String,
    pub pgo_profile_sha256: Option<String>,
    pub forbidden_codegen_flags_present: bool,
}

impl CodegenBuildProvenanceV1 {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != CODEGEN_BUILD_SCHEMA_VERSION {
            diagnostics.push("CODEGEN_BUILD_SCHEMA_MISMATCH".to_owned());
        }
        if self.build_profile != self.candidate.expected_build_profile() {
            diagnostics.push("CODEGEN_BUILD_PROFILE_MISMATCH".to_owned());
        }
        if self.commit == "UNKNOWN" || !self.worktree_clean {
            diagnostics.push("CODEGEN_BUILD_REQUIRES_CLEAN_COMMIT".to_owned());
        }
        if !is_sha256(&self.executable_sha256) {
            diagnostics.push("CODEGEN_EXECUTABLE_HASH_INVALID".to_owned());
        }
        if rustc_release(&self.toolchain) != Some(PINNED_RUSTC_RELEASE) {
            diagnostics.push("CODEGEN_RUSTC_RELEASE_MISMATCH".to_owned());
        }
        if self.opt_level != "3" {
            diagnostics.push("CODEGEN_OPT_LEVEL_MISMATCH".to_owned());
        }
        if !self.profile_overrides.is_empty() {
            diagnostics.push("CODEGEN_PROFILE_OVERRIDES_FORBIDDEN".to_owned());
        }
        if self.forbidden_codegen_flags_present {
            diagnostics.push("CODEGEN_FORBIDDEN_FLAGS_PRESENT".to_owned());
        }
        let effective_flags_valid = match self.candidate {
            CodegenCandidateV1::Baseline | CodegenCandidateV1::ThinLto => {
                self.effective_rustflags.is_empty()
            }
            CodegenCandidateV1::Pgo => is_exact_pgo_use_flags(&self.effective_rustflags),
        };
        if !effective_flags_valid {
            diagnostics.push("CODEGEN_EFFECTIVE_RUSTFLAGS_INVALID".to_owned());
        }
        match self.candidate {
            CodegenCandidateV1::Baseline => {
                if self.codegen_mode != "plain" || self.pgo_profile_sha256.is_some() {
                    diagnostics.push("CODEGEN_BASELINE_PROVENANCE_INVALID".to_owned());
                }
            }
            CodegenCandidateV1::ThinLto => {
                if self.codegen_mode != "plain" || self.pgo_profile_sha256.is_some() {
                    diagnostics.push("CODEGEN_THIN_LTO_PROVENANCE_INVALID".to_owned());
                }
            }
            CodegenCandidateV1::Pgo => {
                if self.codegen_mode != "pgo-use"
                    || self
                        .pgo_profile_sha256
                        .as_deref()
                        .is_none_or(|hash| !is_sha256(hash))
                {
                    diagnostics.push("CODEGEN_PGO_PROVENANCE_INVALID".to_owned());
                }
            }
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
pub struct CodegenRunProvenanceV1 {
    pub schema_version: u32,
    pub build: CodegenBuildProvenanceV1,
    pub performance_report_sha256: String,
}

impl CodegenRunProvenanceV1 {
    pub fn validate_for(&self, build: &CodegenBuildProvenanceV1) -> Result<(), Vec<String>> {
        let mut diagnostics = self.build.validate().err().unwrap_or_default();
        if self.schema_version != CODEGEN_RUN_PROVENANCE_SCHEMA_VERSION {
            diagnostics.push("CODEGEN_RUN_PROVENANCE_SCHEMA_MISMATCH".to_owned());
        }
        if &self.build != build {
            diagnostics.push("CODEGEN_RUN_BUILD_MISMATCH".to_owned());
        }
        if !is_sha256(&self.performance_report_sha256) {
            diagnostics.push("CODEGEN_RUN_REPORT_HASH_INVALID".to_owned());
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
}

#[derive(Clone, Debug)]
pub struct CodegenScenarioRunSetV1 {
    pub scenario: PerformanceScenarioV1,
    pub runs: Vec<PerformanceRunV6>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodegenMetricComparisonV1 {
    pub name: String,
    pub unit: String,
    pub baseline_p95: u64,
    pub candidate_p95: u64,
    /// 100 basis points equal 1%; negative values are improvements.
    pub change_basis_points: i64,
    pub confidence_interval_95_basis_points: [i64; 2],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodegenScenarioComparisonV1 {
    pub scenario: PerformanceScenarioV1,
    pub mean_change_basis_points: i64,
    pub worst_metric_change_basis_points: i64,
    pub metrics: Vec<CodegenMetricComparisonV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodegenComparisonV1 {
    pub schema_version: u32,
    pub candidate: CodegenCandidateV1,
    pub verdict: CodegenComparisonVerdictV1,
    pub baseline_profile: String,
    pub candidate_profile: String,
    pub calibration_runs_per_scenario: u32,
    pub scenarios: Vec<CodegenScenarioComparisonV1>,
    pub combined_change_basis_points: Option<i64>,
    pub combined_confidence_interval_95_basis_points: Option<[i64; 2]>,
    pub statistically_significant_combined_improvement: bool,
    pub no_hard_scenario_regression: bool,
    pub eligible_for_separate_shipping_decision: bool,
    pub fallback_profile: String,
    pub diagnostics: Vec<String>,
}

impl CodegenComparisonV1 {
    pub fn not_run(candidate: CodegenCandidateV1, diagnostics: Vec<String>) -> Self {
        Self {
            schema_version: CODEGEN_COMPARISON_SCHEMA_VERSION,
            candidate,
            verdict: CodegenComparisonVerdictV1::NotRun,
            baseline_profile: "release".to_owned(),
            candidate_profile: candidate.expected_build_profile().to_owned(),
            calibration_runs_per_scenario: CODEGEN_CALIBRATION_RUNS as u32,
            scenarios: Vec::new(),
            combined_change_basis_points: None,
            combined_confidence_interval_95_basis_points: None,
            statistically_significant_combined_improvement: false,
            no_hard_scenario_regression: false,
            eligible_for_separate_shipping_decision: false,
            fallback_profile: "release".to_owned(),
            diagnostics,
        }
    }
}

pub fn compare_codegen_run_sets(
    candidate: CodegenCandidateV1,
    baseline_provenance: &CodegenBuildProvenanceV1,
    candidate_provenance: &CodegenBuildProvenanceV1,
    baseline_sets: &[CodegenScenarioRunSetV1],
    candidate_sets: &[CodegenScenarioRunSetV1],
) -> Result<CodegenComparisonV1, Vec<String>> {
    let mut diagnostics = baseline_provenance.validate().err().unwrap_or_default();
    diagnostics.extend(candidate_provenance.validate().err().unwrap_or_default());
    if candidate == CodegenCandidateV1::Baseline {
        diagnostics.push("CODEGEN_COMPARISON_REQUIRES_OPTIMIZATION_CANDIDATE".to_owned());
    }
    if baseline_provenance.candidate != CodegenCandidateV1::Baseline {
        diagnostics.push("CODEGEN_BASELINE_BUILD_KIND_MISMATCH".to_owned());
    }
    if candidate_provenance.candidate != candidate {
        diagnostics.push("CODEGEN_BUILD_CANDIDATE_MISMATCH".to_owned());
    }
    let baseline_anchor = first_run(baseline_sets);
    if baseline_provenance.commit
        != baseline_anchor
            .map(|run| run.commit.as_str())
            .unwrap_or("UNKNOWN")
    {
        diagnostics.push("CODEGEN_BASELINE_BUILD_COMMIT_MISMATCH".to_owned());
    }
    if baseline_provenance.toolchain
        != baseline_anchor
            .map(|run| run.toolchain.as_str())
            .unwrap_or("UNKNOWN")
    {
        diagnostics.push("CODEGEN_BASELINE_BUILD_TOOLCHAIN_MISMATCH".to_owned());
    }
    if candidate_provenance.commit != baseline_provenance.commit {
        diagnostics.push("CODEGEN_BUILD_COMMIT_MISMATCH".to_owned());
    }
    if candidate_provenance.toolchain != baseline_provenance.toolchain {
        diagnostics.push("CODEGEN_BUILD_TOOLCHAIN_MISMATCH".to_owned());
    }

    for scenario in CODEGEN_SCENARIOS {
        if let Some(reason) = scenario.unavailable_reason() {
            diagnostics.push(format!(
                "CODEGEN_REPRESENTATIVE_WORKLOAD_UNAVAILABLE: {}: {reason}",
                scenario.as_str()
            ));
        }
        validate_group_presence(baseline_sets, scenario, "baseline", &mut diagnostics);
        validate_group_presence(candidate_sets, scenario, "candidate", &mut diagnostics);
    }
    reject_unexpected_groups(baseline_sets, "baseline", &mut diagnostics);
    reject_unexpected_groups(candidate_sets, "candidate", &mut diagnostics);

    let global_anchor = baseline_anchor;
    for set in baseline_sets {
        validate_run_group(set, "release", "baseline", global_anchor, &mut diagnostics);
    }
    for set in candidate_sets {
        validate_run_group(
            set,
            candidate.expected_build_profile(),
            "candidate",
            global_anchor,
            &mut diagnostics,
        );
    }

    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }

    let mut scenarios = Vec::with_capacity(CODEGEN_SCENARIOS.len());
    let mut combined_changes = Vec::new();
    let mut combined_bootstrap_scenarios = Vec::with_capacity(CODEGEN_SCENARIOS.len());
    for scenario in CODEGEN_SCENARIOS {
        let baseline = find_group(baseline_sets, scenario).expect("validated baseline group");
        let current = find_group(candidate_sets, scenario).expect("validated candidate group");
        let comparison = compare_scenario(baseline, current).map_err(|error| vec![error])?;
        for metric in &comparison.metrics {
            combined_changes.push(metric.change_basis_points);
        }
        combined_bootstrap_scenarios
            .push(combined_bootstrap_scenario(baseline, current).map_err(|error| vec![error])?);
        scenarios.push(comparison);
    }

    let combined_change = mean_i64(&combined_changes).map_err(|error| vec![error])?;
    let combined_interval = bootstrap_combined_change_interval(
        &combined_bootstrap_scenarios,
        CODEGEN_BOOTSTRAP_ITERATIONS,
    )
    .map_err(|error| vec![error])?;
    let statistically_significant_combined_improvement =
        combined_change < 0 && combined_interval[1] < 0;
    let no_hard_scenario_regression = scenarios
        .iter()
        .all(|scenario| scenario.worst_metric_change_basis_points < 200);
    let eligible_for_separate_shipping_decision =
        statistically_significant_combined_improvement && no_hard_scenario_regression;
    let mut diagnostics = Vec::new();
    if !statistically_significant_combined_improvement {
        diagnostics.push("CODEGEN_COMBINED_IMPROVEMENT_NOT_SIGNIFICANT".to_owned());
    }
    for scenario in &scenarios {
        if scenario.worst_metric_change_basis_points >= 200 {
            diagnostics.push(format!(
                "CODEGEN_HARD_SCENARIO_REGRESSION: {}: {} basis points",
                scenario.scenario.as_str(),
                scenario.worst_metric_change_basis_points
            ));
        }
    }
    if eligible_for_separate_shipping_decision {
        diagnostics.push("CODEGEN_CANDIDATE_ELIGIBLE_FOR_SEPARATE_SHIPPING_DECISION".to_owned());
    }

    Ok(CodegenComparisonV1 {
        schema_version: CODEGEN_COMPARISON_SCHEMA_VERSION,
        candidate,
        verdict: CodegenComparisonVerdictV1::ReportOnly,
        baseline_profile: "release".to_owned(),
        candidate_profile: candidate.expected_build_profile().to_owned(),
        calibration_runs_per_scenario: CODEGEN_CALIBRATION_RUNS as u32,
        scenarios,
        combined_change_basis_points: Some(combined_change),
        combined_confidence_interval_95_basis_points: Some(combined_interval),
        statistically_significant_combined_improvement,
        no_hard_scenario_regression,
        eligible_for_separate_shipping_decision,
        fallback_profile: "release".to_owned(),
        diagnostics,
    })
}

fn first_run(sets: &[CodegenScenarioRunSetV1]) -> Option<&PerformanceRunV6> {
    sets.iter().find_map(|set| set.runs.first())
}

fn find_group(
    sets: &[CodegenScenarioRunSetV1],
    scenario: PerformanceScenarioV1,
) -> Option<&CodegenScenarioRunSetV1> {
    sets.iter().find(|set| set.scenario == scenario)
}

fn validate_group_presence(
    sets: &[CodegenScenarioRunSetV1],
    scenario: PerformanceScenarioV1,
    side: &str,
    diagnostics: &mut Vec<String>,
) {
    let count = sets.iter().filter(|set| set.scenario == scenario).count();
    if count != 1 {
        diagnostics.push(format!(
            "CODEGEN_SCENARIO_SET_INVALID: {side}: {}: found {count}",
            scenario.as_str()
        ));
    }
}

fn reject_unexpected_groups(
    sets: &[CodegenScenarioRunSetV1],
    side: &str,
    diagnostics: &mut Vec<String>,
) {
    for set in sets {
        if !CODEGEN_SCENARIOS.contains(&set.scenario) {
            diagnostics.push(format!(
                "CODEGEN_UNEXPECTED_SCENARIO: {side}: {}",
                set.scenario.as_str()
            ));
        }
    }
}

fn validate_run_group(
    set: &CodegenScenarioRunSetV1,
    expected_profile: &str,
    side: &str,
    global_anchor: Option<&PerformanceRunV6>,
    diagnostics: &mut Vec<String>,
) {
    let prefix = format!("{side}:{}", set.scenario.as_str());
    if set.runs.len() != CODEGEN_CALIBRATION_RUNS {
        diagnostics.push(format!(
            "CODEGEN_REQUIRES_TEN_RUNS: {prefix}: found {}",
            set.runs.len()
        ));
    }
    let Some(group_anchor) = set.runs.first() else {
        return;
    };
    for (index, run) in set.runs.iter().enumerate() {
        let run_prefix = format!("{prefix}:run-{index}");
        if run.schema_version != PERFORMANCE_RUN_SCHEMA_VERSION
            || run.scenario != set.scenario
            || run.mode != PerformanceModeV1::Report
            || run.build_profile != expected_profile
        {
            diagnostics.push(format!("CODEGEN_RUN_IDENTITY_INVALID: {run_prefix}"));
        }
        if run.commit == "UNKNOWN" || !run.worktree_clean {
            diagnostics.push(format!("CODEGEN_RUN_REQUIRES_CLEAN_COMMIT: {run_prefix}"));
        }
        if matches!(
            run.verdict,
            PerformanceVerdict::Fail | PerformanceVerdict::NotRun
        ) || !run.diagnostics.is_empty()
        {
            diagnostics.push(format!("CODEGEN_RUN_INVALID: {run_prefix}"));
        }
        if run
            .preflight
            .as_ref()
            .is_none_or(|preflight| !preflight.ready)
        {
            diagnostics.push(format!("CODEGEN_PREFLIGHT_NOT_READY: {run_prefix}"));
        }
        match &run.target_fingerprint {
            Some(fingerprint) => {
                for diagnostic in validate_linux_release_fingerprint(fingerprint) {
                    diagnostics.push(format!(
                        "CODEGEN_FINGERPRINT_INVALID: {run_prefix}: {diagnostic}"
                    ));
                }
            }
            None => diagnostics.push(format!("CODEGEN_FINGERPRINT_MISSING: {run_prefix}")),
        }
        if let Err(errors) = run.validate_hard_evidence() {
            diagnostics.extend(
                errors
                    .into_iter()
                    .map(|error| format!("CODEGEN_HARD_EVIDENCE_INVALID: {run_prefix}: {error}")),
            );
        }
        if run.commit != group_anchor.commit
            || run.toolchain != group_anchor.toolchain
            || run.scenario_hash != group_anchor.scenario_hash
            || run.content_hash != group_anchor.content_hash
            || run.target_fingerprint != group_anchor.target_fingerprint
            || run.methodology != group_anchor.methodology
            || run.methodology.methodology_version != PERFORMANCE_METHODOLOGY_VERSION
            || run.authoritative_hashes != group_anchor.authoritative_hashes
        {
            diagnostics.push(format!("CODEGEN_RUN_SET_INCOMPATIBLE: {run_prefix}"));
        }
        if let Some(anchor) = global_anchor
            && (run.commit != anchor.commit
                || run.toolchain != anchor.toolchain
                || run.target_fingerprint != anchor.target_fingerprint)
        {
            diagnostics.push(format!("CODEGEN_GLOBAL_IDENTITY_MISMATCH: {run_prefix}"));
        }
        if run.metrics.is_empty() {
            diagnostics.push(format!("CODEGEN_METRICS_MISSING: {run_prefix}"));
        }
        let mut names = BTreeMap::new();
        for metric in &run.metrics {
            if metric.raw_samples.is_empty()
                || names
                    .insert(metric.name.as_str(), metric.unit.as_str())
                    .is_some()
            {
                diagnostics.push(format!(
                    "CODEGEN_METRIC_INVALID: {run_prefix}: {}",
                    metric.name
                ));
            }
        }
        let anchor_names = group_anchor
            .metrics
            .iter()
            .map(|metric| (metric.name.as_str(), metric.unit.as_str()))
            .collect::<BTreeMap<_, _>>();
        if names != anchor_names {
            diagnostics.push(format!("CODEGEN_METRIC_SET_MISMATCH: {run_prefix}"));
        }
    }
}

fn compare_scenario(
    baseline: &CodegenScenarioRunSetV1,
    candidate: &CodegenScenarioRunSetV1,
) -> Result<CodegenScenarioComparisonV1, String> {
    let baseline_metrics = metric_run_p95s(&baseline.runs)?;
    let candidate_metrics = metric_run_p95s(&candidate.runs)?;
    if baseline_metrics.keys().ne(candidate_metrics.keys()) {
        return Err(format!(
            "CODEGEN_METRIC_SET_MISMATCH: {}",
            baseline.scenario.as_str()
        ));
    }
    let mut metrics = Vec::with_capacity(baseline_metrics.len());
    for (name, (unit, baseline_run_p95s)) in baseline_metrics {
        let Some((candidate_unit, candidate_run_p95s)) = candidate_metrics.get(&name) else {
            return Err(format!("CODEGEN_METRIC_MISSING: {name}"));
        };
        if candidate_unit != &unit {
            return Err(format!("CODEGEN_METRIC_UNIT_MISMATCH: {name}"));
        }
        let baseline_p95 = nearest_rank_percentile(&baseline_run_p95s, 50)?;
        let candidate_p95 = nearest_rank_percentile(candidate_run_p95s, 50)?;
        metrics.push(CodegenMetricComparisonV1 {
            name,
            unit,
            baseline_p95,
            candidate_p95,
            change_basis_points: relative_change_basis_points(candidate_p95, baseline_p95),
            confidence_interval_95_basis_points: bootstrap_change_interval(
                candidate_run_p95s,
                &baseline_run_p95s,
                CODEGEN_BOOTSTRAP_ITERATIONS,
            )?,
        });
    }
    let changes = metrics
        .iter()
        .map(|metric| metric.change_basis_points)
        .collect::<Vec<_>>();
    let mean_change_basis_points = mean_i64(&changes)?;
    let worst_metric_change_basis_points = changes
        .iter()
        .copied()
        .max()
        .ok_or_else(|| "CODEGEN_METRICS_MISSING".to_owned())?;
    Ok(CodegenScenarioComparisonV1 {
        scenario: baseline.scenario,
        mean_change_basis_points,
        worst_metric_change_basis_points,
        metrics,
    })
}

struct CombinedBootstrapMetric {
    baseline_run_p95s: Vec<u64>,
    candidate_run_p95s: Vec<u64>,
}

struct CombinedBootstrapScenario {
    metrics: Vec<CombinedBootstrapMetric>,
}

fn combined_bootstrap_scenario(
    baseline: &CodegenScenarioRunSetV1,
    candidate: &CodegenScenarioRunSetV1,
) -> Result<CombinedBootstrapScenario, String> {
    let baseline_metrics = metric_run_p95s(&baseline.runs)?;
    let candidate_metrics = metric_run_p95s(&candidate.runs)?;
    if baseline_metrics.keys().ne(candidate_metrics.keys()) {
        return Err(format!(
            "CODEGEN_METRIC_SET_MISMATCH: {}",
            baseline.scenario.as_str()
        ));
    }
    let mut metrics = Vec::with_capacity(baseline_metrics.len());
    for (name, (unit, baseline_run_p95s)) in baseline_metrics {
        let Some((candidate_unit, candidate_run_p95s)) = candidate_metrics.get(&name) else {
            return Err(format!("CODEGEN_METRIC_MISSING: {name}"));
        };
        if candidate_unit != &unit {
            return Err(format!("CODEGEN_METRIC_UNIT_MISMATCH: {name}"));
        }
        metrics.push(CombinedBootstrapMetric {
            baseline_run_p95s,
            candidate_run_p95s: candidate_run_p95s.clone(),
        });
    }
    Ok(CombinedBootstrapScenario { metrics })
}

fn metric_run_p95s(
    runs: &[PerformanceRunV6],
) -> Result<BTreeMap<String, (String, Vec<u64>)>, String> {
    let mut metrics = BTreeMap::<String, (String, Vec<u64>)>::new();
    for run in runs {
        for metric in &run.metrics {
            let run_p95 = nearest_rank_percentile(&metric.raw_samples, 95)?;
            match metrics.get_mut(&metric.name) {
                Some((unit, run_p95s)) if unit == &metric.unit => {
                    run_p95s.push(run_p95);
                }
                Some(_) => return Err(format!("CODEGEN_METRIC_UNIT_MISMATCH: {}", metric.name)),
                None => {
                    metrics.insert(metric.name.clone(), (metric.unit.clone(), vec![run_p95]));
                }
            }
        }
    }
    Ok(metrics)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn rustc_release(toolchain: &str) -> Option<&str> {
    let mut releases = toolchain
        .lines()
        .filter_map(|line| line.strip_prefix("release: "));
    let release = releases.next()?;
    releases.next().is_none().then_some(release)
}

fn is_exact_pgo_use_flags(flags: &[String]) -> bool {
    let [flag] = flags else {
        return false;
    };
    flag.strip_prefix("-Cprofile-use=")
        .is_some_and(|path| !path.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pinned_toolchain() -> String {
        "rustc 1.97.1 (8bab26f4f 2026-07-14)\n\
binary: rustc\n\
commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452\n\
commit-date: 2026-07-14\n\
host: x86_64-pc-windows-msvc\n\
release: 1.97.1\n\
LLVM version: 22.1.6"
            .to_owned()
    }

    fn valid_thin_lto_build() -> CodegenBuildProvenanceV1 {
        CodegenBuildProvenanceV1 {
            schema_version: CODEGEN_BUILD_SCHEMA_VERSION,
            candidate: CodegenCandidateV1::ThinLto,
            build_profile: "release-thin-lto".to_owned(),
            commit: "a".repeat(40),
            worktree_clean: true,
            executable_sha256: "b".repeat(64),
            toolchain: pinned_toolchain(),
            opt_level: "3".to_owned(),
            effective_rustflags: Vec::new(),
            profile_overrides: Vec::new(),
            codegen_mode: "plain".to_owned(),
            pgo_profile_sha256: None,
            forbidden_codegen_flags_present: false,
        }
    }

    fn valid_baseline_build() -> CodegenBuildProvenanceV1 {
        let mut provenance = valid_thin_lto_build();
        provenance.candidate = CodegenCandidateV1::Baseline;
        provenance.build_profile = "release".to_owned();
        provenance.executable_sha256 = "c".repeat(64);
        provenance
    }

    fn assert_build_diagnostic(provenance: &CodegenBuildProvenanceV1, expected: &str) {
        let diagnostics = provenance.validate().expect_err("invalid provenance");
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic == expected),
            "missing {expected} in {diagnostics:?}"
        );
    }

    #[test]
    fn build_provenance_rejects_plain_pgo_and_forbidden_flags() {
        let provenance = CodegenBuildProvenanceV1 {
            schema_version: CODEGEN_BUILD_SCHEMA_VERSION,
            candidate: CodegenCandidateV1::Pgo,
            build_profile: "release-pgo".to_owned(),
            commit: "a".repeat(40),
            worktree_clean: true,
            executable_sha256: "b".repeat(64),
            toolchain: pinned_toolchain(),
            opt_level: "3".to_owned(),
            effective_rustflags: Vec::new(),
            profile_overrides: Vec::new(),
            codegen_mode: "plain".to_owned(),
            pgo_profile_sha256: None,
            forbidden_codegen_flags_present: true,
        };
        let diagnostics = provenance.validate().expect_err("invalid provenance");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic == "CODEGEN_PGO_PROVENANCE_INVALID")
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic == "CODEGEN_FORBIDDEN_FLAGS_PRESENT")
        );
    }

    #[test]
    fn build_provenance_rejects_stable_rustc_after_pinned_release() {
        let mut provenance = valid_thin_lto_build();
        provenance.toolchain = provenance.toolchain.replace(
            "rustc 1.97.1 (8bab26f4f 2026-07-14)",
            "rustc 1.98.0 (000000000 2026-09-01)",
        );
        provenance.toolchain = provenance
            .toolchain
            .replace("release: 1.97.1", "release: 1.98.0");

        let diagnostics = provenance.validate().expect_err("unpinned rustc");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic == "CODEGEN_RUSTC_RELEASE_MISMATCH")
        );
    }

    #[test]
    fn build_provenance_requires_lowercase_executable_sha256() {
        let mut provenance = valid_thin_lto_build();
        provenance.executable_sha256 = "A".repeat(64);

        let diagnostics = provenance.validate().expect_err("uppercase hash");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic == "CODEGEN_EXECUTABLE_HASH_INVALID")
        );
    }

    #[test]
    fn baseline_provenance_requires_plain_release_binary() {
        valid_baseline_build()
            .validate()
            .expect("plain release baseline");
        let mut provenance = valid_baseline_build();
        provenance.codegen_mode = "pgo-use".to_owned();
        provenance.pgo_profile_sha256 = Some("d".repeat(64));

        let diagnostics = provenance.validate().expect_err("PGO baseline");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic == "CODEGEN_BASELINE_PROVENANCE_INVALID")
        );
    }

    #[test]
    fn plain_codegen_rejects_any_effective_rustflags() {
        for flag in [
            "-Copt-level=0",
            "-Clto=off",
            "-Ctarget-feature=+avx2",
            "-Ctarget-cpu=native",
        ] {
            let mut provenance = valid_baseline_build();
            provenance.effective_rustflags = vec![flag.to_owned()];
            provenance.forbidden_codegen_flags_present = flag == "-Ctarget-cpu=native";
            assert_build_diagnostic(&provenance, "CODEGEN_EFFECTIVE_RUSTFLAGS_INVALID");
        }
    }

    #[test]
    fn build_provenance_rejects_opt_level_and_arbitrary_profile_override() {
        let mut wrong_opt_level = valid_baseline_build();
        wrong_opt_level.opt_level = "0".to_owned();
        assert_build_diagnostic(&wrong_opt_level, "CODEGEN_OPT_LEVEL_MISMATCH");

        let mut overridden_profile = valid_baseline_build();
        overridden_profile
            .profile_overrides
            .push(CodegenProfileOverrideV1 {
                name: "CARGO_PROFILE_RELEASE_LTO".to_owned(),
                value: "off".to_owned(),
            });
        assert_build_diagnostic(&overridden_profile, "CODEGEN_PROFILE_OVERRIDES_FORBIDDEN");
    }

    #[test]
    fn pgo_provenance_allows_only_one_exact_profile_use_flag() {
        let mut provenance = valid_thin_lto_build();
        provenance.candidate = CodegenCandidateV1::Pgo;
        provenance.build_profile = "release-pgo".to_owned();
        provenance.codegen_mode = "pgo-use".to_owned();
        provenance.pgo_profile_sha256 = Some("d".repeat(64));
        provenance.effective_rustflags =
            vec!["-Cprofile-use=D:\\profiles\\merged.profdata".to_owned()];
        provenance.validate().expect("exact PGO profile-use flag");

        provenance
            .effective_rustflags
            .push("-Ctarget-feature=+avx2".to_owned());
        assert_build_diagnostic(&provenance, "CODEGEN_EFFECTIVE_RUSTFLAGS_INVALID");
    }

    #[test]
    fn deterministic_bootstrap_detects_clear_improvement() {
        let baseline = vec![1_000; 100];
        let candidate = vec![800; 100];
        assert_eq!(
            bootstrap_change_interval(&candidate, &baseline, 200).expect("bootstrap"),
            [-2_000, -2_000]
        );
    }

    #[test]
    fn combined_bootstrap_does_not_average_marginal_ci_endpoints() {
        let baseline = vec![100; 10];
        let candidates = (0..5)
            .map(|metric_index| {
                let mut values = vec![90; 10];
                values[metric_index * 2] = 1_000;
                values[metric_index * 2 + 1] = 1_000;
                values
            })
            .collect::<Vec<_>>();
        let old_upper_bounds = candidates
            .iter()
            .map(|candidate| {
                bootstrap_change_interval(candidate, &baseline, CODEGEN_BOOTSTRAP_ITERATIONS)
                    .expect("marginal bootstrap")[1]
            })
            .collect::<Vec<_>>();
        assert_eq!(old_upper_bounds, vec![-1_000; 5]);
        assert_eq!(
            mean_i64(&old_upper_bounds).expect("old combined upper"),
            -1_000
        );

        let scenario = CombinedBootstrapScenario {
            metrics: candidates
                .into_iter()
                .map(|candidate_run_p95s| CombinedBootstrapMetric {
                    baseline_run_p95s: baseline.clone(),
                    candidate_run_p95s,
                })
                .collect(),
        };
        let direct = bootstrap_combined_change_interval(&[scenario], CODEGEN_BOOTSTRAP_ITERATIONS)
            .expect("direct combined bootstrap");

        assert_eq!(direct, [-1_000, 17_200]);
        assert!(direct[1] >= 0);
    }

    #[test]
    fn not_run_always_falls_back_to_release() {
        let report = CodegenComparisonV1::not_run(
            CodegenCandidateV1::ThinLto,
            vec!["CODEGEN_REPRESENTATIVE_WORKLOAD_UNAVAILABLE".to_owned()],
        );
        assert_eq!(report.verdict, CodegenComparisonVerdictV1::NotRun);
        assert_eq!(report.fallback_profile, "release");
        assert!(!report.eligible_for_separate_shipping_decision);
    }

    #[test]
    fn comparison_refuses_to_infer_missing_representative_workloads() {
        let baseline = valid_baseline_build();
        let candidate = valid_thin_lto_build();
        let diagnostics =
            compare_codegen_run_sets(CodegenCandidateV1::ThinLto, &baseline, &candidate, &[], &[])
                .expect_err("missing scenarios cannot produce a comparison");
        if cfg!(feature = "physx") {
            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.starts_with("CODEGEN_SCENARIO_SET_INVALID: baseline: r5-physics-16")
            }));
        } else {
            assert!(diagnostics.iter().any(|diagnostic| {
                diagnostic.starts_with("CODEGEN_REPRESENTATIVE_WORKLOAD_UNAVAILABLE")
            }));
        }
    }

    #[test]
    fn run_sidecar_binds_exact_build_and_report_hash() {
        let build = valid_thin_lto_build();
        let mut sidecar = CodegenRunProvenanceV1 {
            schema_version: CODEGEN_RUN_PROVENANCE_SCHEMA_VERSION,
            build: build.clone(),
            performance_report_sha256: "b".repeat(64),
        };
        sidecar.validate_for(&build).expect("matching sidecar");
        let mut replaced_binary = build.clone();
        replaced_binary.executable_sha256 = "c".repeat(64);
        let diagnostics = sidecar
            .validate_for(&replaced_binary)
            .expect_err("replaced executable must not match the sidecar");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic == "CODEGEN_RUN_BUILD_MISMATCH")
        );
        sidecar.performance_report_sha256 = "not-a-hash".to_owned();
        assert!(sidecar.validate_for(&build).is_err());
    }

    #[test]
    fn codegen_run_validation_requires_v4_resource_evidence() {
        let set = CodegenScenarioRunSetV1 {
            scenario: PerformanceScenarioV1::R2AlphaRender,
            runs: vec![PerformanceRunV6::empty(
                PerformanceScenarioV1::R2AlphaRender,
                PerformanceModeV1::Report,
                "release",
            )],
        };
        let mut diagnostics = Vec::new();
        validate_run_group(&set, "release", "baseline", None, &mut diagnostics);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("CODEGEN_HARD_EVIDENCE_INVALID")
                && diagnostic.contains("logical_resource_charges")
        }));
    }
}
