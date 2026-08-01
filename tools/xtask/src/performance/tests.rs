use super::*;

fn fingerprint() -> PerformanceTargetFingerprintV1 {
    PerformanceTargetFingerprintV1 {
        target_id: THOTH_TARGET_ID.to_owned(),
        hostname: "THOTH".to_owned(),
        cpu_model: "AMD Ryzen 9 3950X 16-Core Processor".to_owned(),
        physical_cores: 16,
        logical_threads: 32,
        gpu_model: "NVIDIA GeForce RTX 3080".to_owned(),
        gpu_vram_mib: 10_240,
        ram_bytes: 32 * 1024 * 1024 * 1024,
        storage_model: "WDS100T1X0E-00AFY0".to_owned(),
        storage_bytes: 1_000_204_886_016,
        os_name: "Microsoft Windows 11 Pro".to_owned(),
        os_build: "26200".to_owned(),
        bios_version: "test-bios".to_owned(),
        gpu_driver: "591.86".to_owned(),
        power_plan: "AMD Ryzen™ High Performance".to_owned(),
    }
}

#[test]
fn nearest_rank_retains_outliers() {
    let samples = [1, 2, 3, 4, 100];
    assert_eq!(nearest_rank_percentile(&samples, 50), Ok(3));
    assert_eq!(nearest_rank_percentile(&samples, 95), Ok(100));
    assert_eq!(nearest_rank_percentile(&samples, 99), Ok(100));
}

#[test]
fn thoth_fingerprint_is_exact_on_invalidating_fields() {
    assert!(validate_thoth_fingerprint(&fingerprint()).is_empty());
    let mut wrong = fingerprint();
    wrong.gpu_driver = "591.85".to_owned();
    assert_eq!(
        validate_thoth_fingerprint(&wrong),
        vec!["PERF_GPU_DRIVER_MISMATCH"]
    );
}

#[test]
fn schema_round_trip_rejects_unknown_fields() {
    let mut run = PerformanceRunV1::empty(
        PerformanceScenarioV1::Smoke,
        PerformanceModeV1::Report,
        "release",
    );
    run.target_fingerprint = Some(fingerprint());
    let json = serde_json::to_vec(&run).expect("serialize run");
    let decoded: PerformanceRunV1 = serde_json::from_slice(&json).expect("decode run");
    assert_eq!(decoded, run);

    let mut value: serde_json::Value = serde_json::from_slice(&json).expect("decode JSON value");
    value
        .as_object_mut()
        .expect("run object")
        .insert("unknown".to_owned(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<PerformanceRunV1>(value).is_err());
}

#[test]
fn production_worker_scenario_has_distinct_versioned_methodology() {
    let run = PerformanceRunV1::empty(
        PerformanceScenarioV1::ProductionWorkerSoak,
        PerformanceModeV1::Report,
        "release",
    );
    let smoke = PerformanceRunV1::empty(
        PerformanceScenarioV1::Smoke,
        PerformanceModeV1::Report,
        "release",
    );
    let methodology = methodology_for(PerformanceScenarioV1::ProductionWorkerSoak);

    assert_ne!(run.scenario_hash, smoke.scenario_hash);
    assert_eq!(run.scenario_hash, sha256_hex(b"production-worker-soak"));
    assert_eq!(methodology.measured_samples, 240);
    assert!(
        methodology
            .notes
            .iter()
            .any(|note| note.contains("production-worker-soak.v1"))
    );
}

#[test]
fn incompatible_host_is_not_a_conditional_pass() {
    let mut wrong = fingerprint();
    wrong.hostname = "OTHER".to_owned();
    assert_eq!(
        validate_thoth_fingerprint(&wrong),
        vec!["PERF_HOSTNAME_MISMATCH"]
    );
}

#[test]
fn relative_policy_distinguishes_noise_warning_and_failure() {
    assert_eq!(
        relative_verdict(PerformanceVerdict::ReportOnly, 199, [100, 250]),
        PerformanceVerdict::ReportOnly
    );
    assert_eq!(
        relative_verdict(PerformanceVerdict::ReportOnly, 300, [200, 400]),
        PerformanceVerdict::Warning
    );
    assert_eq!(
        relative_verdict(PerformanceVerdict::ReportOnly, 550, [510, 600]),
        PerformanceVerdict::Fail
    );
}

#[test]
fn deterministic_bootstrap_matches_known_constant_samples() {
    assert_eq!(
        bootstrap_change_interval(&[100; 16], &[100; 16], 200),
        Ok([0, 0])
    );
    assert_eq!(
        bootstrap_change_interval(&[106; 16], &[100; 16], 200),
        Ok([600, 600])
    );
}

#[test]
fn dropped_or_unowned_spans_invalidate_instrumentation() {
    let mut instrumentation = PerformanceInstrumentationV1::disabled();
    instrumentation.dropped_spans = 1;
    assert_eq!(
        instrumentation.validate(),
        Err("PERF_DROPPED_SPANS".to_owned())
    );
    instrumentation.dropped_spans = 0;
    instrumentation.unowned_spans = 1;
    assert_eq!(
        instrumentation.validate(),
        Err("UNOWNED_GAMEPLAY_SPAN".to_owned())
    );
    instrumentation.unowned_spans = 0;
    instrumentation.recorded_spans = vec![PerformanceSpanV1 {
        category: "unknown-stage".to_owned(),
        thread_index: 0,
        duration_microseconds: 1,
    }];
    assert_eq!(
        instrumentation.validate(),
        Err("UNOWNED_GAMEPLAY_SPAN".to_owned())
    );
}

#[test]
fn baseline_requires_ten_clean_compatible_runs() {
    let mut run = PerformanceRunV1::empty(
        PerformanceScenarioV1::R2AlphaRender,
        PerformanceModeV1::Report,
        "release",
    );
    run.commit = "a".repeat(40);
    run.worktree_clean = true;
    run.toolchain = "rustc 1.93.0".to_owned();
    run.target_fingerprint = Some(fingerprint());
    run.preflight = Some(PerformancePreflightV1 {
        cpu_load_percent: Some(0),
        gpu_load_percent: Some(0),
        free_ram_bytes: Some(MINIMUM_FREE_RAM_BYTES),
        cpu_clock_percent_of_maximum: Some(100),
        gpu_thermal_slowdown_active: Some(false),
        ready: true,
        diagnostics: Vec::new(),
    });
    run.content_hash = "b".repeat(64);
    run.scenario_hash = "c".repeat(64);
    run.metrics = vec![
        PerformanceMetricV1::from_samples("frame", "microseconds", vec![10, 11, 12], None)
            .expect("metric"),
    ];
    run.instrumentation.enabled = true;
    run.instrumentation.authoritative_hash_parity = Some(true);
    run.instrumentation.overhead_basis_points = Some(0);
    run.resource_counters = PerformanceResourceCountersV1 {
        host_resident_bytes: Some(1),
        device_resident_bytes: Some(1),
        io_read_bytes: Some(0),
        io_write_bytes: Some(0),
        allocator_allocated_bytes: Some(1),
        allocator_allocation_count: Some(1),
        vulkan_timestamp_queries: 2,
        unavailable: Vec::new(),
    };
    run.verdict = PerformanceVerdict::ReportOnly;
    run.authoritative_hashes
        .insert("state".to_owned(), "d".repeat(64));
    let runs = vec![run; 10];
    let baseline = PerformanceBaselineV1::from_runs(&runs).expect("baseline");
    assert_eq!(baseline.calibration_runs, 10);
    assert_eq!(baseline.metrics[0].raw_samples.len(), 30);
}
