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
        power_plan: "AMD Ryzenв„ў High Performance".to_owned(),
    }
}

fn allocation_counter(
    scenario: PerformanceScenarioV1,
    scenario_hash: &str,
) -> ProcessAllocationCounterV1 {
    ProcessAllocationCounterV1::from_exact_parts(
        scenario,
        scenario_hash,
        ProcessAllocationCounterInputV1 {
            producer_pid: 42,
            window_id: 7,
            alloc_count: 1,
            alloc_bytes: 4,
            alloc_zeroed_count: 2,
            alloc_zeroed_bytes: 8,
            realloc_count: 3,
            realloc_bytes: 12,
        },
    )
    .expect("valid allocation counter")
}

fn pinned_toolchain(target_triple: &str) -> String {
    format!(
        "rustc 1.93.0 (254b59607 2026-01-19)\nbinary: rustc\ncommit-hash: {PERFORMANCE_PINNED_RUSTC_COMMIT_HASH}\ncommit-date: 2026-01-19\nhost: {target_triple}\nrelease: 1.93.0\nLLVM version: 21.1.0",
    )
}

fn logical_resource_charges() -> PerformanceLogicalResourceChargesV1 {
    PerformanceLogicalResourceChargesV1::new(
        sha256_hex(b"nextengine.test.logical-resource-accounting-profile.v1"),
        1,
        2,
        3,
        4,
        5,
        6,
    )
    .expect("valid logical resource charges")
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
fn preflight_recomputes_readiness_from_typed_evidence() {
    let mut preflight = PerformancePreflightV1 {
        cpu_load_percent: Some(0),
        gpu_load_percent: Some(0),
        free_ram_bytes: Some(MINIMUM_FREE_RAM_BYTES),
        cpu_clock_percent_of_maximum: Some(100),
        gpu_thermal_slowdown_active: Some(false),
        ready: true,
        diagnostics: Vec::new(),
    };
    preflight
        .validate_ready_evidence()
        .expect("complete typed preflight is ready");
    preflight.free_ram_bytes = Some(MINIMUM_FREE_RAM_BYTES - 1);
    assert_eq!(
        preflight.validate_ready_evidence(),
        Err(vec!["PERF_FREE_RAM_BELOW_TWENTY_GIB".to_owned()])
    );
}

#[test]
fn schema_round_trip_rejects_unknown_fields() {
    let mut run = PerformanceRunV3::empty(
        PerformanceScenarioV1::Smoke,
        PerformanceModeV1::Report,
        "release",
    );
    run.target_fingerprint = Some(fingerprint());
    assert_eq!(run.schema_version, 3);
    assert_eq!(
        run.methodology.methodology_version,
        "nextengine-performance-v3"
    );
    assert_eq!(PERFORMANCE_REPORT_FILE_NAME, "performance-report-v3.json");
    assert_eq!(
        PERFORMANCE_BASELINE_FILE_NAME,
        "performance-baseline-v3.json"
    );
    let json = serde_json::to_vec(&run).expect("serialize run");
    let decoded: PerformanceRunV3 = serde_json::from_slice(&json).expect("decode run");
    assert_eq!(decoded, run);

    let mut wrong_hash = run.clone();
    wrong_hash.scenario_hash = "0".repeat(64);
    assert_eq!(
        wrong_hash.validate_wire_version(),
        Err(vec!["PERF_RUN_SCENARIO_HASH_MISMATCH".to_owned()])
    );
    let mut wrong_methodology = run.clone();
    wrong_methodology
        .methodology
        .notes
        .push("tampered".to_owned());
    assert_eq!(
        wrong_methodology.validate_wire_version(),
        Err(vec!["PERF_RUN_METHODOLOGY_MISMATCH".to_owned()])
    );

    let mut historical: serde_json::Value =
        serde_json::from_slice(&json).expect("decode historical fixture");
    historical["schema_version"] = serde_json::Value::from(2);
    historical["methodology"]["methodology_version"] =
        serde_json::Value::String("nextengine-performance-v2".to_owned());
    let counters = historical["resource_counters"]
        .as_object_mut()
        .expect("resource counters object");
    counters.remove("process_peak_working_set_bytes");
    counters.remove("logical_resource_charges");
    counters.insert("host_resident_bytes".to_owned(), serde_json::Value::Null);
    let historical: PerformanceRunV2 =
        serde_json::from_value(historical).expect("V2 remains readable as historical evidence");
    historical
        .validate_historical_wire()
        .expect("the historical V2 identity remains valid");

    let mut value: serde_json::Value = serde_json::from_slice(&json).expect("decode JSON value");
    value
        .as_object_mut()
        .expect("run object")
        .insert("unknown".to_owned(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<PerformanceRunV3>(value).is_err());
}

#[test]
fn strict_build_provenance_pins_commit_release_and_host() {
    let commit = "a".repeat(40);
    let toolchain = pinned_toolchain(PERFORMANCE_WINDOWS_TARGET_TRIPLE);
    validate_performance_build_provenance(&commit, &toolchain, PERFORMANCE_WINDOWS_TARGET_TRIPLE)
        .expect("exact pinned native provenance");

    assert_eq!(
        validate_performance_build_provenance(
            "UNKNOWN",
            &toolchain,
            PERFORMANCE_WINDOWS_TARGET_TRIPLE,
        ),
        Err(vec!["PERF_BUILD_COMMIT_INVALID".to_owned()])
    );
    let wrong_release = toolchain.replace("release: 1.93.0", "release: 1.94.0");
    assert_eq!(
        validate_performance_build_provenance(
            &commit,
            &wrong_release,
            PERFORMANCE_WINDOWS_TARGET_TRIPLE,
        ),
        Err(vec!["PERF_BUILD_RUSTC_RELEASE_MISMATCH".to_owned()])
    );
    let wrong_commit = toolchain.replace(
        PERFORMANCE_PINNED_RUSTC_COMMIT_HASH,
        "0000000000000000000000000000000000000000",
    );
    assert_eq!(
        validate_performance_build_provenance(
            &commit,
            &wrong_commit,
            PERFORMANCE_WINDOWS_TARGET_TRIPLE,
        ),
        Err(vec!["PERF_BUILD_RUSTC_COMMIT_MISMATCH".to_owned()])
    );
    assert_eq!(
        validate_performance_build_provenance(&commit, &toolchain, PERFORMANCE_LINUX_TARGET_TRIPLE,),
        Err(vec!["PERF_BUILD_TOOLCHAIN_HOST_MISMATCH".to_owned()])
    );
}

#[test]
fn compiled_xtask_provenance_uses_the_pinned_native_toolchain() {
    let toolchain = decode_build_toolchain_hex(env!("NEXTENGINE_BUILD_TOOLCHAIN_HEX"))
        .expect("build-script toolchain payload");
    validate_performance_build_provenance(
        env!("NEXTENGINE_BUILD_COMMIT"),
        &toolchain,
        env!("NEXTENGINE_BUILD_TARGET_TRIPLE"),
    )
    .expect("compiled xtask provenance must be repository-pinned");
}

#[test]
fn command_report_status_is_derived_from_the_nested_verdict() {
    let mut run = PerformanceRunV3::empty(
        PerformanceScenarioV1::Smoke,
        PerformanceModeV1::Report,
        "debug",
    );
    run.verdict = PerformanceVerdict::ReportOnly;
    run.validate_command_report_status("PASS")
        .expect("report-only maps to a successful command report");
    assert_eq!(
        run.validate_command_report_status("NOT_RUN"),
        Err(
            "PERF_COMMAND_STATUS_MISMATCH: expected PASS for nested REPORT_ONLY, found NOT_RUN"
                .to_owned()
        )
    );
}

#[test]
fn allocator_payload_is_strict_and_top_level_totals_match() {
    let scenario_hash = sha256_hex(b"smoke.v2");
    let counter = allocation_counter(PerformanceScenarioV1::Smoke, &scenario_hash);
    assert_eq!(counter.schema_version, 1);
    assert_eq!(counter.source, "rust-global-allocator");
    assert_eq!(counter.allocator, "rust-std-system");
    assert_eq!(counter.counter_scope, "xtask-process-scenario-window-v1");
    assert_eq!(counter.deallocation_semantics, "delegated-not-subtracted");
    assert_eq!(counter.allocator_allocation_count, 6);
    assert_eq!(counter.allocator_allocated_bytes, 24);

    let mut counters = PerformanceResourceCountersV3::default();
    counters
        .attach_allocator_counter(counter.clone())
        .expect("attach exact evidence");
    assert_eq!(counters.allocator_allocation_count, Some(6));
    assert_eq!(counters.allocator_allocated_bytes, Some(24));
    counters
        .validate_optional_allocator_for_run(PerformanceScenarioV1::Smoke, &scenario_hash)
        .expect("run identity matches");
    assert_eq!(
        counters.validate_optional_allocator_for_run(
            PerformanceScenarioV1::Smoke,
            &sha256_hex(b"smoke.other-version"),
        ),
        Err(vec![
            "PERF_ALLOCATOR_COUNTER_SCENARIO_HASH_MISMATCH".to_owned()
        ])
    );

    let mut value = serde_json::to_value(counter).expect("serialize counter");
    value
        .as_object_mut()
        .expect("counter object")
        .insert("unknown".to_owned(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<ProcessAllocationCounterV1>(value).is_err());

    let mut incomplete = serde_json::to_value(allocation_counter(
        PerformanceScenarioV1::Smoke,
        &scenario_hash,
    ))
    .expect("serialize counter");
    incomplete
        .as_object_mut()
        .expect("counter object")
        .remove("realloc_bytes");
    assert!(serde_json::from_value::<ProcessAllocationCounterV1>(incomplete).is_err());
}

#[test]
fn v3_hard_counters_require_logical_charges_but_not_allocator_evidence() {
    let scenario = PerformanceScenarioV1::R3MultiregionStreaming;
    let scenario_hash = performance_scenario_hash(scenario);
    let mut counters = PerformanceResourceCountersV3 {
        process_peak_working_set_bytes: Some(1024),
        device_resident_bytes: Some(512),
        io_read_bytes: Some(128),
        io_write_bytes: Some(64),
        logical_resource_charges: Some(logical_resource_charges()),
        vulkan_timestamp_queries: 2,
        unavailable: vec!["PERF_ALLOCATOR_COUNTER_OPTIONAL_NOT_COLLECTED".to_owned()],
        ..PerformanceResourceCountersV3::default()
    };
    counters
        .validate_for_hard_timing_for_run(scenario, &scenario_hash)
        .expect("V3 hard evidence does not require allocator instrumentation");

    counters.logical_resource_charges = None;
    assert_eq!(
        counters.validate_for_hard_timing_for_run(scenario, &scenario_hash),
        Err(vec![
            "PERF_REQUIRED_COUNTER_MISSING: logical_resource_charges".to_owned()
        ])
    );

    counters.logical_resource_charges = Some(logical_resource_charges());
    counters.allocator_allocated_bytes = Some(1);
    assert_eq!(
        counters.validate_for_hard_timing_for_run(scenario, &scenario_hash),
        Err(vec![
            "PERF_ALLOCATOR_COUNTER_INCOMPLETE".to_owned(),
            "PERF_ALLOCATOR_COUNTER_OPTIONAL_NOT_COLLECTED".to_owned(),
        ])
    );
}

#[test]
fn logical_resource_charge_root_rejects_tampered_totals() {
    let mut charges = logical_resource_charges();
    charges.total_host_charged_bytes += 1;
    assert_eq!(
        charges.validate(),
        Err(vec![
            "PERF_LOGICAL_RESOURCE_HOST_TOTAL_MISMATCH".to_owned(),
            "PERF_LOGICAL_RESOURCE_ROOT_MISMATCH".to_owned(),
        ])
    );
}

#[test]
fn allocator_validation_rejects_partial_mismatched_and_overflowing_evidence() {
    let scenario_hash = sha256_hex(b"long-session-soak.v3");
    let mut counters = PerformanceResourceCountersV3::default();
    counters
        .validate_optional_allocator_for_run(PerformanceScenarioV1::LongSessionSoak, &scenario_hash)
        .expect("allocator evidence is optional in V3");
    counters.allocator_allocated_bytes = Some(1);
    assert_eq!(
        counters.validate_optional_allocator_for_run(
            PerformanceScenarioV1::LongSessionSoak,
            &scenario_hash,
        ),
        Err(vec!["PERF_ALLOCATOR_COUNTER_INCOMPLETE".to_owned()])
    );

    let mut mismatched = PerformanceResourceCountersV3::default();
    mismatched
        .attach_allocator_counter(allocation_counter(
            PerformanceScenarioV1::LongSessionSoak,
            &scenario_hash,
        ))
        .expect("attach counter");
    mismatched.allocator_allocation_count = Some(5);
    assert_eq!(
        mismatched.validate_optional_allocator_for_run(
            PerformanceScenarioV1::LongSessionSoak,
            &scenario_hash,
        ),
        Err(vec!["PERF_ALLOCATOR_COUNTER_TOP_LEVEL_MISMATCH".to_owned()])
    );

    let mut contradictory = PerformanceResourceCountersV3::default();
    contradictory
        .attach_allocator_counter(allocation_counter(
            PerformanceScenarioV1::LongSessionSoak,
            &scenario_hash,
        ))
        .expect("attach counter");
    contradictory
        .unavailable
        .push("PERF_ALLOCATOR_COUNTER_OVERFLOW".to_owned());
    assert_eq!(
        contradictory.validate_optional_allocator_for_run(
            PerformanceScenarioV1::LongSessionSoak,
            &scenario_hash,
        ),
        Err(vec![
            "PERF_ALLOCATOR_COUNTER_CONTRADICTORY".to_owned(),
            "PERF_ALLOCATOR_COUNTER_OVERFLOW".to_owned(),
        ])
    );

    let mut legacy_contradiction = PerformanceResourceCountersV3::default();
    legacy_contradiction
        .attach_allocator_counter(allocation_counter(
            PerformanceScenarioV1::LongSessionSoak,
            &scenario_hash,
        ))
        .expect("attach counter");
    legacy_contradiction
        .unavailable
        .push("allocator counters require the runtime allocator hook".to_owned());
    assert_eq!(
        legacy_contradiction.validate_optional_allocator_for_run(
            PerformanceScenarioV1::LongSessionSoak,
            &scenario_hash,
        ),
        Err(vec![
            "PERF_ALLOCATOR_COUNTER_CONTRADICTORY".to_owned(),
            "allocator counters require the runtime allocator hook".to_owned(),
        ])
    );

    let overflow = ProcessAllocationCounterV1::from_exact_parts(
        PerformanceScenarioV1::Smoke,
        sha256_hex(b"smoke.v2"),
        ProcessAllocationCounterInputV1 {
            producer_pid: 1,
            window_id: 1,
            alloc_count: u64::MAX,
            alloc_bytes: 0,
            alloc_zeroed_count: 1,
            alloc_zeroed_bytes: 0,
            realloc_count: 0,
            realloc_bytes: 0,
        },
    );
    assert_eq!(
        overflow,
        Err("PERF_ALLOCATOR_COUNTER_TOTAL_OVERFLOW".to_owned())
    );

    let mut overflowing_payload =
        allocation_counter(PerformanceScenarioV1::Smoke, &sha256_hex(b"smoke.v2"));
    overflowing_payload.alloc_count = u64::MAX;
    overflowing_payload.alloc_zeroed_count = 1;
    assert_eq!(
        overflowing_payload.validate(),
        Err(vec!["PERF_ALLOCATOR_COUNTER_TOTAL_OVERFLOW".to_owned()])
    );

    let mut mismatched_payload =
        allocation_counter(PerformanceScenarioV1::Smoke, &sha256_hex(b"smoke.v2"));
    mismatched_payload.allocator_allocated_bytes = 23;
    assert_eq!(
        mismatched_payload.validate(),
        Err(vec!["PERF_ALLOCATOR_COUNTER_TOTAL_MISMATCH".to_owned()])
    );

    let mut invalid_pid =
        allocation_counter(PerformanceScenarioV1::Smoke, &sha256_hex(b"smoke.v2"));
    invalid_pid.producer_pid = 0;
    assert_eq!(
        invalid_pid.validate(),
        Err(vec!["PERF_ALLOCATOR_COUNTER_PID_INVALID".to_owned()])
    );
}

#[test]
fn allocator_run_validation_rejects_an_exact_but_empty_window() {
    let scenario_hash = sha256_hex(b"smoke.empty-window");
    let counter = ProcessAllocationCounterV1::from_exact_parts(
        PerformanceScenarioV1::Smoke,
        &scenario_hash,
        ProcessAllocationCounterInputV1 {
            producer_pid: 42,
            window_id: 7,
            alloc_count: 0,
            alloc_bytes: 0,
            alloc_zeroed_count: 0,
            alloc_zeroed_bytes: 0,
            realloc_count: 0,
            realloc_bytes: 0,
        },
    )
    .expect("an empty snapshot remains structurally exact");
    counter
        .validate()
        .expect("offline structural validation does not invent workload policy");

    let mut counters = PerformanceResourceCountersV3::default();
    counters
        .attach_allocator_counter(counter)
        .expect("attach structurally exact evidence");
    assert_eq!(
        counters.validate_optional_allocator_for_run(PerformanceScenarioV1::Smoke, &scenario_hash),
        Err(vec!["PERF_ALLOCATOR_COUNTER_EMPTY".to_owned()])
    );
}

#[test]
fn allocator_offline_validation_does_not_compare_reader_pid() {
    let scenario_hash = sha256_hex(b"production-worker-soak.v1");
    let mut counter =
        allocation_counter(PerformanceScenarioV1::ProductionWorkerSoak, &scenario_hash);
    counter.producer_pid = if std::process::id() == 1 { 2 } else { 1 };
    let mut counters = PerformanceResourceCountersV3::default();
    counters
        .attach_allocator_counter(counter)
        .expect("historical producer PID is provenance");
    counters
        .validate_optional_allocator_for_run(
            PerformanceScenarioV1::ProductionWorkerSoak,
            &scenario_hash,
        )
        .expect("reader PID is intentionally ignored");
}

#[test]
fn production_worker_scenario_has_distinct_versioned_methodology() {
    let run = PerformanceRunV3::empty(
        PerformanceScenarioV1::ProductionWorkerSoak,
        PerformanceModeV1::Report,
        "release",
    );
    let smoke = PerformanceRunV3::empty(
        PerformanceScenarioV1::Smoke,
        PerformanceModeV1::Report,
        "release",
    );
    let methodology = methodology_for(PerformanceScenarioV1::ProductionWorkerSoak);

    assert_ne!(run.scenario_hash, smoke.scenario_hash);
    assert_eq!(
        run.scenario_hash,
        performance_scenario_hash(PerformanceScenarioV1::ProductionWorkerSoak)
    );
    assert_eq!(methodology.measured_samples, 240);
    assert!(
        methodology
            .notes
            .iter()
            .any(|note| note.contains("production-worker-soak.v1"))
    );
}

#[test]
fn r2_alpha_render_is_an_available_six_window_workload() {
    let scenario = PerformanceScenarioV1::R2AlphaRender;
    assert_eq!(scenario.unavailable_reason(), None);
    assert!(
        PerformanceScenarioV1::R3MultiregionStreaming
            .unavailable_reason()
            .is_some()
    );
    assert_eq!(
        performance_scenario_hash(scenario),
        sha256_hex(
            b"nextengine.performance.r2-alpha-render.v1:reference-alpha:frontier-relay:windows=exploration+combat+ui-dialogue:profiles=primary-1920x1080+fallback-b0-safe-1280x720p30:each=600-warmup+3600-measured:critical=max-cpu-extract-submit-gpu:retain-all:resource-window=sequential-six-window-production-vulkan:logical-accounting=r2-alpha-render-v1"
        )
    );
    let methodology = methodology_for(scenario);
    assert_eq!(methodology.warmup_samples, 3_600);
    assert_eq!(methodology.measured_samples, 21_600);
    assert_eq!(
        methodology.frame_critical_path.as_deref(),
        Some("max(cpu_extract_and_submit_us,gpu_timestamp_duration_us)")
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
fn enabled_instrumentation_requires_parity_and_bounded_overhead_evidence() {
    let mut instrumentation = PerformanceInstrumentationV1::disabled();
    instrumentation.enabled = true;
    assert_eq!(
        instrumentation.validate(),
        Err("PERF_PROFILER_AUTHORITY_UNAVAILABLE".to_owned())
    );
    instrumentation.authoritative_hash_parity = Some(true);
    assert_eq!(
        instrumentation.validate(),
        Err("PERF_PROFILER_OVERHEAD_UNAVAILABLE".to_owned())
    );
    instrumentation.overhead_basis_points = Some(-1);
    assert_eq!(
        instrumentation.validate(),
        Err("PERF_PROFILER_OVERHEAD_INVALID".to_owned())
    );
    instrumentation.overhead_basis_points = Some(300);
    instrumentation
        .validate()
        .expect("complete profiler evidence at the limit is valid");
}

#[test]
fn baseline_requires_ten_clean_compatible_runs() {
    let mut run = PerformanceRunV3::empty(
        PerformanceScenarioV1::R2AlphaRender,
        PerformanceModeV1::Report,
        "release",
    );
    run.commit = "a".repeat(40);
    run.worktree_clean = true;
    run.target_triple = PERFORMANCE_WINDOWS_TARGET_TRIPLE.to_owned();
    run.toolchain = pinned_toolchain(&run.target_triple);
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
    run.scenario_hash = performance_scenario_hash(run.scenario);
    run.metrics = vec![
        PerformanceMetricV1::from_samples(
            "frame",
            "microseconds",
            vec![10, 11, 12],
            Some(PerformanceBudgetV1 {
                p95_max: Some(20),
                p99_max: Some(20),
            }),
        )
        .expect("metric"),
    ];
    run.instrumentation.enabled = true;
    run.instrumentation.authoritative_hash_parity = Some(true);
    run.instrumentation.overhead_basis_points = Some(0);
    run.resource_counters = PerformanceResourceCountersV3 {
        process_peak_working_set_bytes: Some(1),
        device_resident_bytes: Some(1),
        io_read_bytes: Some(0),
        io_write_bytes: Some(0),
        logical_resource_charges: Some(logical_resource_charges()),
        allocator_allocated_bytes: None,
        allocator_allocation_count: None,
        allocator_counter: None,
        vulkan_timestamp_queries: 2,
        unavailable: Vec::new(),
    };
    run.verdict = PerformanceVerdict::ReportOnly;
    run.authoritative_hashes
        .insert("state".to_owned(), "d".repeat(64));
    let runs = vec![run; 10];
    let baseline = PerformanceBaselineV3::from_runs(&runs).expect("baseline");
    assert_eq!(baseline.calibration_runs, 10);
    assert_eq!(baseline.target_triple, PERFORMANCE_WINDOWS_TARGET_TRIPLE);
    assert_eq!(baseline.metrics[0].raw_samples.len(), 30);
}

#[test]
fn baseline_centrally_rejects_an_incompatible_run_wire_version() {
    let mut run = PerformanceRunV3::empty(
        PerformanceScenarioV1::R2AlphaRender,
        PerformanceModeV1::Report,
        "release",
    );
    run.schema_version = 1;
    run.commit = "a".repeat(40);
    run.worktree_clean = true;
    run.target_triple = PERFORMANCE_WINDOWS_TARGET_TRIPLE.to_owned();
    run.toolchain = pinned_toolchain(&run.target_triple);
    run.target_fingerprint = Some(fingerprint());

    let diagnostics = PerformanceBaselineV3::from_runs(&vec![run; 10])
        .expect_err("from_runs owns wire admission");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic == "PERF_BASELINE_RUN_INVALID: 0: PERF_RUN_SCHEMA_MISMATCH"
    }));
}
