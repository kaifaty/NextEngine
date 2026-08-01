const ALLOCATOR_COUNTER_TARGET_NOT_ADMITTED: &str = "PERF_ALLOCATOR_COUNTER_TARGET_NOT_ADMITTED";

pub(super) struct ScenarioResourceWindow {
    scenario: xtask::performance::PerformanceScenarioV1,
    scenario_hash: String,
    process_counters_before: xtask::performance::PerformanceResourceCountersV2,
    allocation_measurement: Option<next_process_allocation_counter::Measurement>,
    begin_diagnostic: Option<String>,
}

impl ScenarioResourceWindow {
    pub(super) fn begin(
        scenario: xtask::performance::PerformanceScenarioV1,
        scenario_hash: &str,
    ) -> Self {
        let scenario_hash = scenario_hash.to_owned();
        let process_counters_before = xtask::performance::inspect_process_counters();
        let (allocation_measurement, begin_diagnostic) = if allocator_counter_target_admitted() {
            match next_process_allocation_counter::begin() {
                Ok(measurement) => (Some(measurement), None),
                Err(error) => (None, Some(error.as_str().to_owned())),
            }
        } else {
            (None, Some(ALLOCATOR_COUNTER_TARGET_NOT_ADMITTED.to_owned()))
        };
        Self {
            scenario,
            scenario_hash,
            process_counters_before,
            allocation_measurement,
            begin_diagnostic,
        }
    }

    pub(super) fn finish(mut self) -> xtask::performance::PerformanceResourceCountersV2 {
        // Close the allocation window before the OS after-probe starts a child process.
        let allocation_result = self
            .allocation_measurement
            .take()
            .map(next_process_allocation_counter::Measurement::finish);
        let mut counters =
            xtask::performance::finish_process_counters(&self.process_counters_before);
        remove_allocator_placeholder_diagnostics(&mut counters);

        if let Some(diagnostic) = self.begin_diagnostic {
            push_unavailable(&mut counters, diagnostic);
            return counters;
        }
        let Some(allocation_result) = allocation_result else {
            push_unavailable(
                &mut counters,
                "PERF_ALLOCATOR_COUNTER_UNAVAILABLE".to_owned(),
            );
            return counters;
        };
        let snapshot = match allocation_result {
            Ok(snapshot) => snapshot,
            Err(error) => {
                push_unavailable(&mut counters, error.as_str().to_owned());
                return counters;
            }
        };
        if snapshot.producer_pid != std::process::id() {
            push_unavailable(&mut counters, "PERF_ALLOCATOR_PROCESS_MISMATCH".to_owned());
            return counters;
        }
        let input = xtask::performance::ProcessAllocationCounterInputV1 {
            producer_pid: snapshot.producer_pid,
            window_id: snapshot.window_id,
            alloc_count: snapshot.alloc_count,
            alloc_bytes: snapshot.alloc_bytes,
            alloc_zeroed_count: snapshot.alloc_zeroed_count,
            alloc_zeroed_bytes: snapshot.alloc_zeroed_bytes,
            realloc_count: snapshot.realloc_count,
            realloc_bytes: snapshot.realloc_bytes,
        };
        let counter = match xtask::performance::ProcessAllocationCounterV1::from_exact_parts(
            self.scenario,
            self.scenario_hash,
            input,
        ) {
            Ok(counter) => counter,
            Err(error) => {
                push_unavailable(&mut counters, error);
                return counters;
            }
        };
        if counter.allocator_allocation_count != snapshot.allocator_allocation_count
            || counter.allocator_allocated_bytes != snapshot.allocator_allocated_bytes
        {
            push_unavailable(
                &mut counters,
                "PERF_ALLOCATOR_COUNTER_TOTAL_MISMATCH".to_owned(),
            );
            return counters;
        }
        if let Err(error) = counters.attach_allocator_counter(counter) {
            push_unavailable(&mut counters, error);
        }
        counters
    }
}

const fn allocator_counter_target_admitted() -> bool {
    cfg!(all(
        target_os = "windows",
        target_arch = "x86_64",
        target_env = "msvc"
    ))
}

fn remove_allocator_placeholder_diagnostics(
    counters: &mut xtask::performance::PerformanceResourceCountersV2,
) {
    counters.unavailable.retain(|diagnostic| {
        diagnostic != "allocator counters require the runtime allocator hook"
            && diagnostic != "PERF_ALLOCATOR_COUNTER_UNAVAILABLE"
    });
}

fn push_unavailable(
    counters: &mut xtask::performance::PerformanceResourceCountersV2,
    diagnostic: String,
) {
    if !counters.unavailable.contains(&diagnostic) {
        counters.unavailable.push(diagnostic);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_counter_target_admission_is_pinned_to_windows_x64_msvc() {
        assert_eq!(
            allocator_counter_target_admitted(),
            cfg!(all(
                target_os = "windows",
                target_arch = "x86_64",
                target_env = "msvc"
            ))
        );
    }

    #[cfg(not(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc")))]
    #[test]
    fn unadmitted_target_uses_stable_fail_closed_diagnostic() {
        assert!(!allocator_counter_target_admitted());
        assert_eq!(
            ALLOCATOR_COUNTER_TARGET_NOT_ADMITTED,
            "PERF_ALLOCATOR_COUNTER_TARGET_NOT_ADMITTED"
        );
    }
}
