use std::collections::BTreeMap;

pub(super) fn append_authoritative_hashes(
    hashes: &mut BTreeMap<String, String>,
    report: &next_verification::R2AlphaRenderPerformanceReportV1,
) {
    hashes.insert(
        "r2_alpha_state".to_owned(),
        report.authoritative_state_root.to_hex(),
    );
    hashes.insert(
        "r2_alpha_ledger".to_owned(),
        report.command_ledger_hash.to_hex(),
    );
}

pub(super) fn attach_resource_evidence(
    counters: &mut xtask::performance::PerformanceResourceCountersV3,
    methodology_notes: &mut Vec<String>,
    report: &next_verification::R2AlphaRenderPerformanceReportV1,
) -> Result<(), String> {
    let timestamp_queries = report.vulkan_timestamp_queries();
    let device_bytes = report.device_allocation_ceiling_bytes();
    let device_count = report.device_allocation_ceiling_count();
    if timestamp_queries == 0 || device_bytes == 0 || device_count == 0 {
        return Err("R2_ALPHA_RENDER_RESOURCE_EVIDENCE_EMPTY".to_owned());
    }
    counters.vulkan_timestamp_queries = timestamp_queries;
    counters.device_resident_bytes = Some(device_bytes);
    counters.logical_resource_charges = Some(
        xtask::performance::PerformanceLogicalResourceChargesV1::new(
            report.accounting_profile_hash.to_hex(),
            report.authoritative_state_bytes,
            report.required_staging_bytes,
            report.reconstructible_host_cache_bytes,
            device_bytes,
            report.presentation_transient_bytes,
            report.tooling_transient_bytes,
        )?,
    );
    counters.unavailable.retain(|diagnostic| {
        !diagnostic.starts_with("Vulkan timestamps require")
            && !diagnostic.starts_with("device residency requires")
    });
    methodology_notes.push(format!(
        "six sequential production Vulkan windows emitted {timestamp_queries} timestamp queries; device residency is the maximum {}-allocation engine-owned ceiling, not a sum across profiles",
        device_count,
    ));
    methodology_notes.push(format!(
        "logical resource accounting profile {} charges {} authoritative, {} staging, {} reconstructible host-cache, {} device-cache, {} presentation-transient and {} tooling-transient bytes",
        report.accounting_profile_hash.to_hex(),
        report.authoritative_state_bytes,
        report.required_staging_bytes,
        report.reconstructible_host_cache_bytes,
        device_bytes,
        report.presentation_transient_bytes,
        report.tooling_transient_bytes,
    ));
    Ok(())
}

pub(super) fn append_metrics(
    metrics: &mut Vec<xtask::performance::PerformanceMetricV1>,
    report: &next_verification::R2AlphaRenderPerformanceReportV1,
) -> Result<(), String> {
    for window in &report.windows {
        let prefix = format!(
            "r2-alpha-render.{}.{}",
            window.profile.as_str(),
            window.window.as_str(),
        );
        let critical_path = window
            .timing
            .samples
            .iter()
            .map(|sample| {
                sample
                    .cpu_extract_and_submit_microseconds
                    .max(sample.gpu_duration_microseconds)
            })
            .collect::<Vec<_>>();
        let budget = match window.profile {
            next_verification::R2AlphaRenderProfileV1::Primary1080p => {
                xtask::performance::PerformanceBudgetV1 {
                    p95_max: Some(14_000),
                    p99_max: Some(16_670),
                }
            }
            next_verification::R2AlphaRenderProfileV1::Safe720p30 => {
                xtask::performance::PerformanceBudgetV1 {
                    p95_max: None,
                    p99_max: Some(33_330),
                }
            }
        };
        let deadline = budget
            .p99_max
            .ok_or_else(|| "R2_ALPHA_RENDER_DEADLINE_MISSING".to_owned())?;
        let deadline_misses = u64::try_from(
            critical_path
                .iter()
                .filter(|sample| **sample > deadline)
                .count(),
        )
        .map_err(|error| error.to_string())?;
        metrics.push(xtask::performance::PerformanceMetricV1::from_samples(
            format!("{prefix}.critical-path"),
            "microseconds",
            critical_path,
            Some(budget),
        )?);
        metrics.push(xtask::performance::PerformanceMetricV1::from_samples(
            format!("{prefix}.cpu-extract-submit"),
            "microseconds",
            window
                .timing
                .samples
                .iter()
                .map(|sample| sample.cpu_extract_and_submit_microseconds)
                .collect(),
            None,
        )?);
        metrics.push(xtask::performance::PerformanceMetricV1::from_samples(
            format!("{prefix}.gpu"),
            "microseconds",
            window
                .timing
                .samples
                .iter()
                .map(|sample| sample.gpu_duration_microseconds)
                .collect(),
            None,
        )?);
        metrics.push(xtask::performance::PerformanceMetricV1::from_samples(
            format!("{prefix}.deadline-misses"),
            "count",
            vec![deadline_misses],
            None,
        )?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::ids::{CommandLedgerHash, ContentHash, StateRoot};

    fn timing(value: u64) -> next_verification::DesktopFrameTimingSmokeReport {
        next_verification::DesktopFrameTimingSmokeReport {
            samples: vec![next_verification::DesktopFrameTimingSmokeSample {
                cpu_extract_and_submit_microseconds: value,
                gpu_duration_microseconds: value.saturating_sub(1),
                event_and_frame_source_update_microseconds: 1,
                frame_slot_wait_microseconds: 1,
                image_acquire_wait_microseconds: 1,
                swapchain_image_wait_microseconds: 1,
                frame_plan_microseconds: 1,
                command_record_microseconds: 1,
                queue_submit_microseconds: 1,
                present_wait_microseconds: 1,
            }],
            drawable_extent: [1_920, 1_080],
            timestamp_query_count: 2,
            dropped_samples: 0,
            frame_plan_hash: ContentHash::from_bytes([3; 32]),
            frame_plan_cache_hits: 0,
            frame_plan_cache_misses: 1,
            frame_plan_build_failures: 0,
            frame_plan_explicit_invalidations: 0,
            software_paced_iterations: 0,
            software_pacing_sleep_microseconds: 0,
            device_allocation_bytes: 1,
            device_allocation_count: 1,
        }
    }

    fn report() -> next_verification::R2AlphaRenderPerformanceReportV1 {
        next_verification::R2AlphaRenderPerformanceReportV1 {
            windows: vec![next_verification::R2AlphaRenderWindowReportV1 {
                window: next_verification::R2AlphaRenderWindowV1::Exploration,
                profile: next_verification::R2AlphaRenderProfileV1::Primary1080p,
                presentation_snapshot_hash: ContentHash::from_bytes([4; 32]),
                elapsed_microseconds: 1,
                timing: timing(14_001),
            }],
            authoritative_state_root: StateRoot::from_bytes([5; 32]),
            command_ledger_hash: CommandLedgerHash::from_bytes([6; 32]),
            accounting_profile_hash: ContentHash::from_bytes([7; 32]),
            authoritative_state_bytes: 1,
            required_staging_bytes: 0,
            reconstructible_host_cache_bytes: 2,
            presentation_transient_bytes: 3,
            tooling_transient_bytes: 4,
        }
    }

    #[test]
    fn primary_and_fallback_budgets_remain_explicit() {
        let mut metrics = Vec::new();
        append_metrics(&mut metrics, &report()).expect("metrics");
        let critical = &metrics[0];
        assert_eq!(
            critical.name,
            "r2-alpha-render.primary-1080p.exploration.critical-path"
        );
        assert_eq!(
            critical.absolute_budget.as_ref().expect("budget").p95_max,
            Some(14_000)
        );
        assert_eq!(
            critical.absolute_budget.as_ref().expect("budget").p99_max,
            Some(16_670)
        );
        assert_eq!(
            critical.verdict,
            xtask::performance::PerformanceVerdict::Fail
        );
        assert_eq!(metrics[3].raw_samples, vec![0]);
    }
}
