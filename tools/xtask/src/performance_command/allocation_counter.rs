#[cfg(all(
    test,
    not(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))
))]
const ALLOCATOR_COUNTER_TARGET_NOT_ADMITTED: &str = "PERF_ALLOCATOR_COUNTER_TARGET_NOT_ADMITTED";

pub(super) struct ScenarioResourceWindow {
    process_counters_before: xtask::performance::PerformanceResourceCountersV3,
}

impl ScenarioResourceWindow {
    pub(super) fn begin(
        _scenario: xtask::performance::PerformanceScenarioV1,
        _scenario_hash: &str,
    ) -> Self {
        Self {
            process_counters_before: xtask::performance::inspect_process_counters(),
        }
    }

    pub(super) fn finish(self) -> xtask::performance::PerformanceResourceCountersV3 {
        let mut counters =
            xtask::performance::finish_process_counters(&self.process_counters_before);
        remove_allocator_placeholder_diagnostics(&mut counters);
        push_unavailable(
            &mut counters,
            "PERF_ALLOCATOR_COUNTER_OPTIONAL_NOT_COLLECTED".to_owned(),
        );
        counters
    }
}

#[cfg(test)]
const fn allocator_counter_target_admitted() -> bool {
    cfg!(all(
        target_os = "windows",
        target_arch = "x86_64",
        target_env = "msvc"
    ))
}

fn remove_allocator_placeholder_diagnostics(
    counters: &mut xtask::performance::PerformanceResourceCountersV3,
) {
    counters.unavailable.retain(|diagnostic| {
        diagnostic != "allocator counters require the runtime allocator hook"
            && diagnostic != "PERF_ALLOCATOR_COUNTER_UNAVAILABLE"
    });
}

fn push_unavailable(
    counters: &mut xtask::performance::PerformanceResourceCountersV3,
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
