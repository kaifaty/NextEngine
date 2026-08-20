use std::collections::{BTreeMap, BTreeSet};

use super::{PerformanceBudgetV1, PerformanceMetricV1, PerformanceScenarioV1};

const R2_PRIMARY: PerformanceBudgetV1 = PerformanceBudgetV1 {
    p95_max: Some(14_000),
    p99_max: Some(16_670),
};
const R2_SAFE: PerformanceBudgetV1 = PerformanceBudgetV1 {
    p95_max: None,
    p99_max: Some(33_330),
};
const R3_TOTAL: PerformanceBudgetV1 = PerformanceBudgetV1 {
    p95_max: Some(1_500_000),
    p99_max: Some(1_500_000),
};

pub fn canonical_budget_for_metric(
    scenario: PerformanceScenarioV1,
    name: &str,
) -> Option<PerformanceBudgetV1> {
    let budget = match (scenario, name) {
        (PerformanceScenarioV1::R2AlphaRender, name)
            if is_r2_critical_path(name, "primary-1080p") =>
        {
            R2_PRIMARY
        }
        (PerformanceScenarioV1::R2AlphaRender, name)
            if is_r2_critical_path(name, "b0-safe-720p30") =>
        {
            R2_SAFE
        }
        (PerformanceScenarioV1::R3MultiregionStreaming, "r3-multiregion-streaming.total") => {
            R3_TOTAL
        }
        (PerformanceScenarioV1::R4_100Npc, "r4-100npc.navigation-due-work")
        | (PerformanceScenarioV1::R4_100Npc, "r4-100npc.tier-cognition-due-work") => {
            budget(1_250, 1_500)
        }
        (PerformanceScenarioV1::R4_100Npc, "r4-100npc.integrated-world-services-tick") => {
            budget(8_000, 12_000)
        }
        (PerformanceScenarioV1::R5Physics16, name) => r5_budget(name)?,
        _ => return None,
    };
    Some(budget)
}

pub fn validate_metric_policy(
    scenario: PerformanceScenarioV1,
    metrics: &[PerformanceMetricV1],
) -> Result<(), Vec<String>> {
    let mut diagnostics = Vec::new();
    let mut observed = BTreeMap::new();
    for metric in metrics {
        if observed
            .insert(metric.name.as_str(), metric.absolute_budget.as_ref())
            .is_some()
        {
            diagnostics.push(format!("PERF_METRIC_DUPLICATE: {}", metric.name));
        }
        let expected = canonical_budget_for_metric(scenario, &metric.name);
        if metric.absolute_budget != expected {
            diagnostics.push(format!("PERF_METRIC_POLICY_MISMATCH: {}", metric.name));
        }
    }
    for name in required_hard_metric_names(scenario) {
        if !observed.contains_key(name) {
            diagnostics.push(format!("PERF_HARD_METRIC_MISSING: {name}"));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn required_hard_metric_names(scenario: PerformanceScenarioV1) -> BTreeSet<&'static str> {
    match scenario {
        PerformanceScenarioV1::R2AlphaRender => [
            "r2-alpha-render.primary-1080p.exploration.critical-path",
            "r2-alpha-render.primary-1080p.combat.critical-path",
            "r2-alpha-render.primary-1080p.ui-dialogue.critical-path",
            "r2-alpha-render.b0-safe-720p30.exploration.critical-path",
            "r2-alpha-render.b0-safe-720p30.combat.critical-path",
            "r2-alpha-render.b0-safe-720p30.ui-dialogue.critical-path",
        ]
        .into_iter()
        .collect(),
        PerformanceScenarioV1::R3MultiregionStreaming => {
            ["r3-multiregion-streaming.total"].into_iter().collect()
        }
        PerformanceScenarioV1::R4_100Npc => [
            "r4-100npc.navigation-due-work",
            "r4-100npc.tier-cognition-due-work",
            "r4-100npc.integrated-world-services-tick",
        ]
        .into_iter()
        .collect(),
        PerformanceScenarioV1::R5Physics16 => [
            "r5-physics-16.worker-1.physics-motor-frame",
            "r5-physics-16.worker-1.physics-substep-cost",
            "r5-physics-16.worker-1.motor-frame-cost",
            "r5-physics-16.worker-4.physics-motor-frame",
            "r5-physics-16.worker-4.physics-substep-cost",
            "r5-physics-16.worker-4.motor-frame-cost",
            "r5-physics-16.worker-4.scaling-inefficiency",
            "r5-physics-16.worker-8.physics-motor-frame",
            "r5-physics-16.worker-8.physics-substep-cost",
            "r5-physics-16.worker-8.motor-frame-cost",
            "r5-physics-16.worker-8.scaling-inefficiency",
            "r5-physics-16.restore-fresh-scene",
            "r5-physics-16.checkpoint-bytes-per-slot",
            "r5-physics-16.replay-prefix-overhead",
            "r5-physics-16.process-peak-working-set",
            "r5-physics-16.logical-host-bytes-per-slot",
        ]
        .into_iter()
        .collect(),
        PerformanceScenarioV1::Smoke
        | PerformanceScenarioV1::LongSessionSoak
        | PerformanceScenarioV1::InteractiveFrameSoak
        | PerformanceScenarioV1::ProductionWorkerSoak => BTreeSet::new(),
    }
}

fn is_r2_critical_path(name: &str, profile: &str) -> bool {
    ["exploration", "combat", "ui-dialogue"]
        .into_iter()
        .any(|window| name == format!("r2-alpha-render.{profile}.{window}.critical-path"))
}

fn r5_budget(name: &str) -> Option<PerformanceBudgetV1> {
    let value = match name {
        "r5-physics-16.worker-1.physics-motor-frame" => budget(20_000, 25_000),
        "r5-physics-16.worker-1.physics-substep-cost" => budget(250_000, 250_000),
        "r5-physics-16.worker-1.motor-frame-cost" => budget(1_000_000, 1_000_000),
        "r5-physics-16.worker-4.physics-motor-frame" => budget(6_000, 8_000),
        "r5-physics-16.worker-4.physics-substep-cost" => budget(83_334, 83_334),
        "r5-physics-16.worker-4.motor-frame-cost" => budget(333_334, 333_334),
        "r5-physics-16.worker-4.scaling-inefficiency" => budget(3_500, 3_500),
        "r5-physics-16.worker-8.physics-motor-frame" => budget(4_000, 6_000),
        "r5-physics-16.worker-8.physics-substep-cost" => budget(50_000, 50_000),
        "r5-physics-16.worker-8.motor-frame-cost" => budget(200_000, 200_000),
        "r5-physics-16.worker-8.scaling-inefficiency" => budget(4_500, 4_500),
        "r5-physics-16.restore-fresh-scene" => budget(3_600_000, 4_000_000),
        "r5-physics-16.checkpoint-bytes-per-slot" => budget(4 * 1024 * 1024, 4 * 1024 * 1024),
        "r5-physics-16.replay-prefix-overhead" => budget(12_000, 12_000),
        "r5-physics-16.process-peak-working-set" => budget(320 * 1024 * 1024, 320 * 1024 * 1024),
        "r5-physics-16.logical-host-bytes-per-slot" => budget(4 * 1024 * 1024, 4 * 1024 * 1024),
        _ => return None,
    };
    Some(value)
}

const fn budget(p95_max: u64, p99_max: u64) -> PerformanceBudgetV1 {
    PerformanceBudgetV1 {
        p95_max: Some(p95_max),
        p99_max: Some(p99_max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn representative_policy_has_required_hard_metrics() {
        for scenario in [
            PerformanceScenarioV1::R2AlphaRender,
            PerformanceScenarioV1::R3MultiregionStreaming,
            PerformanceScenarioV1::R4_100Npc,
            PerformanceScenarioV1::R5Physics16,
        ] {
            let names = required_hard_metric_names(scenario);
            assert!(!names.is_empty());
            assert!(
                names
                    .iter()
                    .all(|name| canonical_budget_for_metric(scenario, name).is_some())
            );
        }
    }
}
