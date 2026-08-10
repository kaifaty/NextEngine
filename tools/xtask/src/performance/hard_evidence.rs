use super::{
    PerformanceMetricV1, PerformanceRunV5, metric_run_percentiles, nearest_rank_percentile,
    validate_sample_run_lengths,
};

impl PerformanceMetricV1 {
    pub fn validate_samples_and_budget(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.name.is_empty() || self.unit.is_empty() || self.raw_samples.is_empty() {
            diagnostics.push("PERF_METRIC_SHAPE_INVALID".to_owned());
        } else if let Err(error) = validate_sample_run_lengths(
            &self.raw_samples,
            &self.sample_run_lengths,
            self.sample_run_lengths.len(),
        ) {
            diagnostics.push(format!("{error}: {}", self.name));
        } else {
            for (percentile, observed) in [(50, self.p50), (95, self.p95), (99, self.p99)] {
                match metric_run_percentiles(self, percentile) {
                    Ok(values) => {
                        let expected = if percentile == 50 {
                            nearest_rank_percentile(&values, 50)
                        } else {
                            values
                                .into_iter()
                                .max()
                                .ok_or_else(|| "PERF_METRIC_SHAPE_INVALID".to_owned())
                        };
                        match expected {
                            Ok(expected) if expected != observed => diagnostics.push(format!(
                                "PERF_METRIC_PERCENTILE_MISMATCH: {}: p{percentile}",
                                self.name
                            )),
                            Err(error) => diagnostics.push(error),
                            Ok(_) => {}
                        }
                    }
                    Err(error) => diagnostics.push(error),
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

impl PerformanceRunV5 {
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
            if metric.sample_run_lengths.len()
                != usize::try_from(self.evidence_runs).unwrap_or(usize::MAX)
            {
                diagnostics.push(format!(
                    "PERF_METRIC_EVIDENCE_RUN_COUNT_MISMATCH: {}",
                    metric.name
                ));
            }
            if let Err(errors) = metric.validate_samples_and_budget() {
                diagnostics.extend(errors);
            }
        }
        let start_samples = usize::try_from(self.evidence_runs).ok();
        let complete_samples = self
            .evidence_runs
            .checked_mul(2)
            .and_then(|count| usize::try_from(count).ok());
        if start_samples != Some(self.environment_samples.len())
            && complete_samples != Some(self.environment_samples.len())
        {
            diagnostics.push("PERF_ENVIRONMENT_SAMPLE_COUNT_MISMATCH".to_owned());
        }
        let has_postflight = complete_samples == Some(self.environment_samples.len());
        for (index, sample) in self.environment_samples.iter().enumerate() {
            let validation = if !has_postflight || index.is_multiple_of(2) {
                sample.validate_ready_evidence()
            } else {
                sample.validate_postflight_evidence()
            };
            if let Err(errors) = validation {
                diagnostics.extend(
                    errors
                        .into_iter()
                        .map(|error| format!("PERF_ENVIRONMENT_SAMPLE_INVALID: {index}: {error}")),
                );
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
