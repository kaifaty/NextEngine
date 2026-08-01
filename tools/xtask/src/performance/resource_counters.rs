use serde::{Deserialize, Serialize};

use super::PerformanceScenarioV1;

pub const PROCESS_ALLOCATION_COUNTER_SCHEMA_VERSION: u32 = 1;
pub const PROCESS_ALLOCATION_COUNTER_METHODOLOGY_VERSION: &str =
    "nextengine-process-allocation-counter-v1";
pub const PROCESS_ALLOCATION_COUNTER_SOURCE: &str = "rust-global-allocator";
pub const PROCESS_ALLOCATION_COUNTER_ALLOCATOR: &str = "rust-std-system";
pub const PROCESS_ALLOCATION_COUNTER_SCOPE: &str = "xtask-process-scenario-window-v1";
pub const PROCESS_ALLOCATION_COUNTER_DEALLOCATION_SEMANTICS: &str = "delegated-not-subtracted";
pub const LEGACY_ALLOCATOR_COUNTER_UNAVAILABLE_DIAGNOSTIC: &str =
    "allocator counters require the runtime allocator hook";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessAllocationCounterInputV1 {
    pub producer_pid: u32,
    pub window_id: u64,
    pub alloc_count: u64,
    pub alloc_bytes: u64,
    pub alloc_zeroed_count: u64,
    pub alloc_zeroed_bytes: u64,
    pub realloc_count: u64,
    pub realloc_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessAllocationCounterV1 {
    pub schema_version: u32,
    pub methodology_version: String,
    pub source: String,
    pub allocator: String,
    pub counter_scope: String,
    pub producer_pid: u32,
    pub window_id: u64,
    pub scenario: PerformanceScenarioV1,
    pub scenario_hash: String,
    pub alloc_count: u64,
    pub alloc_bytes: u64,
    pub alloc_zeroed_count: u64,
    pub alloc_zeroed_bytes: u64,
    pub realloc_count: u64,
    pub realloc_bytes: u64,
    pub allocator_allocation_count: u64,
    pub allocator_allocated_bytes: u64,
    pub deallocation_semantics: String,
}

impl ProcessAllocationCounterV1 {
    pub fn from_exact_parts(
        scenario: PerformanceScenarioV1,
        scenario_hash: impl Into<String>,
        input: ProcessAllocationCounterInputV1,
    ) -> Result<Self, String> {
        let allocator_allocation_count = checked_breakdown_sum(
            input.alloc_count,
            input.alloc_zeroed_count,
            input.realloc_count,
        )?;
        let allocator_allocated_bytes = checked_breakdown_sum(
            input.alloc_bytes,
            input.alloc_zeroed_bytes,
            input.realloc_bytes,
        )?;
        let counter = Self {
            schema_version: PROCESS_ALLOCATION_COUNTER_SCHEMA_VERSION,
            methodology_version: PROCESS_ALLOCATION_COUNTER_METHODOLOGY_VERSION.to_owned(),
            source: PROCESS_ALLOCATION_COUNTER_SOURCE.to_owned(),
            allocator: PROCESS_ALLOCATION_COUNTER_ALLOCATOR.to_owned(),
            counter_scope: PROCESS_ALLOCATION_COUNTER_SCOPE.to_owned(),
            producer_pid: input.producer_pid,
            window_id: input.window_id,
            scenario,
            scenario_hash: scenario_hash.into(),
            alloc_count: input.alloc_count,
            alloc_bytes: input.alloc_bytes,
            alloc_zeroed_count: input.alloc_zeroed_count,
            alloc_zeroed_bytes: input.alloc_zeroed_bytes,
            realloc_count: input.realloc_count,
            realloc_bytes: input.realloc_bytes,
            allocator_allocation_count,
            allocator_allocated_bytes,
            deallocation_semantics: PROCESS_ALLOCATION_COUNTER_DEALLOCATION_SEMANTICS.to_owned(),
        };
        counter
            .validate()
            .map_err(|diagnostics| diagnostics.join("; "))?;
        Ok(counter)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        self.validate_for_scenario(None)
    }

    fn validate_for_scenario(
        &self,
        expected: Option<(PerformanceScenarioV1, &str)>,
    ) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != PROCESS_ALLOCATION_COUNTER_SCHEMA_VERSION {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_SCHEMA_MISMATCH".to_owned());
        }
        if self.methodology_version != PROCESS_ALLOCATION_COUNTER_METHODOLOGY_VERSION {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_METHODOLOGY_MISMATCH".to_owned());
        }
        if self.source != PROCESS_ALLOCATION_COUNTER_SOURCE {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_SOURCE_MISMATCH".to_owned());
        }
        if self.allocator != PROCESS_ALLOCATION_COUNTER_ALLOCATOR {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_ALLOCATOR_MISMATCH".to_owned());
        }
        if self.counter_scope != PROCESS_ALLOCATION_COUNTER_SCOPE {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_SCOPE_MISMATCH".to_owned());
        }
        if self.producer_pid == 0 {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_PID_INVALID".to_owned());
        }
        if self.window_id == 0 {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_WINDOW_ID_INVALID".to_owned());
        }
        if !is_sha256(&self.scenario_hash) {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_SCENARIO_HASH_INVALID".to_owned());
        }
        if let Some((scenario, run_scenario_hash)) = expected {
            if self.scenario != scenario {
                diagnostics.push("PERF_ALLOCATOR_COUNTER_SCENARIO_MISMATCH".to_owned());
            }
            if self.scenario_hash != run_scenario_hash {
                diagnostics.push("PERF_ALLOCATOR_COUNTER_SCENARIO_HASH_MISMATCH".to_owned());
            }
            if self.allocator_allocation_count == 0 || self.allocator_allocated_bytes == 0 {
                diagnostics.push("PERF_ALLOCATOR_COUNTER_EMPTY".to_owned());
            }
        }
        match checked_breakdown_sum(
            self.alloc_count,
            self.alloc_zeroed_count,
            self.realloc_count,
        ) {
            Ok(total) if total != self.allocator_allocation_count => {
                diagnostics.push("PERF_ALLOCATOR_COUNTER_TOTAL_MISMATCH".to_owned());
            }
            Err(error) => diagnostics.push(error),
            Ok(_) => {}
        }
        match checked_breakdown_sum(
            self.alloc_bytes,
            self.alloc_zeroed_bytes,
            self.realloc_bytes,
        ) {
            Ok(total) if total != self.allocator_allocated_bytes => {
                diagnostics.push("PERF_ALLOCATOR_COUNTER_TOTAL_MISMATCH".to_owned());
            }
            Err(error) => diagnostics.push(error),
            Ok(_) => {}
        }
        if self.deallocation_semantics != PROCESS_ALLOCATION_COUNTER_DEALLOCATION_SEMANTICS {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_DEALLOCATION_SEMANTICS_MISMATCH".to_owned());
        }
        finish_validation(diagnostics)
    }
}

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

impl PerformanceResourceCountersV2 {
    pub fn attach_allocator_counter(
        &mut self,
        counter: ProcessAllocationCounterV1,
    ) -> Result<(), String> {
        if self.allocator_counter.is_some()
            || self.allocator_allocated_bytes.is_some()
            || self.allocator_allocation_count.is_some()
        {
            return Err("PERF_ALLOCATOR_COUNTER_ALREADY_ATTACHED".to_owned());
        }
        counter
            .validate()
            .map_err(|diagnostics| diagnostics.join("; "))?;
        let allocated_bytes = counter.allocator_allocated_bytes;
        let allocation_count = counter.allocator_allocation_count;
        self.allocator_counter = Some(counter);
        self.allocator_allocated_bytes = Some(allocated_bytes);
        self.allocator_allocation_count = Some(allocation_count);
        Ok(())
    }

    pub fn validate_allocator_for_run(
        &self,
        scenario: PerformanceScenarioV1,
        scenario_hash: &str,
    ) -> Result<(), Vec<String>> {
        self.validate_allocator(Some((scenario, scenario_hash)))
    }

    pub fn validate_for_hard_timing(&self) -> Result<(), Vec<String>> {
        self.validate_hard_counters(None)
    }

    pub fn validate_for_hard_timing_for_run(
        &self,
        scenario: PerformanceScenarioV1,
        scenario_hash: &str,
    ) -> Result<(), Vec<String>> {
        self.validate_hard_counters(Some((scenario, scenario_hash)))
    }

    fn validate_hard_counters(
        &self,
        expected: Option<(PerformanceScenarioV1, &str)>,
    ) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        for (name, value) in [
            ("host_resident_bytes", self.host_resident_bytes),
            ("device_resident_bytes", self.device_resident_bytes),
            ("io_read_bytes", self.io_read_bytes),
            ("io_write_bytes", self.io_write_bytes),
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
        if let Err(errors) = self.validate_allocator(expected) {
            diagnostics.extend(errors);
        }
        finish_validation(diagnostics)
    }

    fn validate_allocator(
        &self,
        expected: Option<(PerformanceScenarioV1, &str)>,
    ) -> Result<(), Vec<String>> {
        let allocator_faults = self
            .unavailable
            .iter()
            .filter(|diagnostic| {
                diagnostic.starts_with("PERF_ALLOCATOR_")
                    || diagnostic.as_str() == LEGACY_ALLOCATOR_COUNTER_UNAVAILABLE_DIAGNOSTIC
            })
            .cloned()
            .collect::<Vec<_>>();
        let Some(counter) = &self.allocator_counter else {
            let mut diagnostics = allocator_faults;
            diagnostics.push(
                if self.allocator_allocated_bytes.is_none()
                    && self.allocator_allocation_count.is_none()
                {
                    "PERF_ALLOCATOR_COUNTER_UNAVAILABLE".to_owned()
                } else {
                    "PERF_ALLOCATOR_COUNTER_INCOMPLETE".to_owned()
                },
            );
            return finish_validation(diagnostics);
        };
        let (Some(allocated_bytes), Some(allocation_count)) = (
            self.allocator_allocated_bytes,
            self.allocator_allocation_count,
        ) else {
            return Err(vec!["PERF_ALLOCATOR_COUNTER_INCOMPLETE".to_owned()]);
        };
        let mut diagnostics = counter
            .validate_for_scenario(expected)
            .err()
            .unwrap_or_default();
        if allocated_bytes != counter.allocator_allocated_bytes
            || allocation_count != counter.allocator_allocation_count
        {
            diagnostics.push("PERF_ALLOCATOR_COUNTER_TOP_LEVEL_MISMATCH".to_owned());
        }
        if !allocator_faults.is_empty() {
            diagnostics.extend(allocator_faults);
            diagnostics.push("PERF_ALLOCATOR_COUNTER_CONTRADICTORY".to_owned());
        }
        finish_validation(diagnostics)
    }
}

fn checked_breakdown_sum(first: u64, second: u64, third: u64) -> Result<u64, String> {
    first
        .checked_add(second)
        .and_then(|total| total.checked_add(third))
        .ok_or_else(|| "PERF_ALLOCATOR_COUNTER_TOTAL_OVERFLOW".to_owned())
}

fn finish_validation(mut diagnostics: Vec<String>) -> Result<(), Vec<String>> {
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
