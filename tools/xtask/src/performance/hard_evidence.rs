use super::{PerformanceMetricV1, PerformanceRunV4, nearest_rank_percentile};

impl PerformanceMetricV1 {
    pub fn validate_samples_and_budget(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.name.is_empty() || self.unit.is_empty() || self.raw_samples.is_empty() {
            diagnostics.push("PERF_METRIC_SHAPE_INVALID".to_owned());
        } else {
            for (percentile, observed) in [(50, self.p50), (95, self.p95), (99, self.p99)] {
                match nearest_rank_percentile(&self.raw_samples, percentile) {
                    Ok(expected) if expected != observed => {
                        diagnostics.push(format!(
                            "PERF_METRIC_PERCENTILE_MISMATCH: {}: p{percentile}",
                            self.name
                        ));
                    }
                    Err(error) => diagnostics.push(error),
                    Ok(_) => {}
                }
            }
        }
        match &self.absolute_budget {
            Some(budget) if budget.p95_max.is_none() && budget.p99_max.is_none() => {
                diagnostics.push(format!("PERF_ABSOLUTE_BUDGET_EMPTY: {}", self.name));
            }
            None => diagnostics.push(format!("PERF_ABSOLUTE_BUDGET_MISSING: {}", self.name)),
            Some(_) => {}
        }
        finish_diagnostics(diagnostics)
    }
}

impl PerformanceRunV4 {
    pub fn validate_hard_evidence(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = self.validate_wire_version().err().unwrap_or_default();
        if !self.instrumentation.enabled {
            diagnostics.push("PERF_PROFILER_DISABLED".to_owned());
        }
        if let Err(error) = self.instrumentation.validate() {
            diagnostics.push(error);
        }
        if let Err(errors) = self
            .resource_counters
            .validate_for_hard_timing(self.scenario)
        {
            diagnostics.extend(errors);
        }
        if self.authoritative_hashes.is_empty() {
            diagnostics.push("PERF_AUTHORITATIVE_ROOTS_MISSING".to_owned());
        }
        for (name, root) in &self.authoritative_hashes {
            if name.is_empty() || !is_lowercase_sha256(root) {
                diagnostics.push(format!("PERF_AUTHORITATIVE_ROOT_INVALID: {name}"));
            }
        }
        if self.metrics.is_empty() {
            diagnostics.push("PERF_METRICS_MISSING".to_owned());
        }
        for metric in &self.metrics {
            if let Err(errors) = metric.validate_samples_and_budget() {
                diagnostics.extend(errors);
            }
        }
        finish_diagnostics(diagnostics)
    }
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn finish_diagnostics(mut diagnostics: Vec<String>) -> Result<(), Vec<String>> {
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
