use super::*;

fn fingerprint() -> PerformanceTargetFingerprintV1 {
    PerformanceTargetFingerprintV1 {
        target_id: LINUX_RELEASE_TARGET_ID.to_owned(),
        hostname: "kaifaty-B550I-AORUS-PRO-AX".to_owned(),
        cpu_model: "AMD Ryzen 9 3950X 16-Core Processor".to_owned(),
        physical_cores: 16,
        logical_threads: 32,
        gpu_model: "NVIDIA GeForce RTX 3080".to_owned(),
        gpu_vram_mib: 10_240,
        ram_bytes: 32 * 1024 * 1024 * 1024,
        storage_model: "SAMSUNG MZVL2512HCJQ-00BH1".to_owned(),
        storage_bytes: 512_110_190_592,
        os_name: "Linux (Ubuntu 26.04 LTS)".to_owned(),
        os_build: "26.04; kernel 7.0.0-29-generic".to_owned(),
        bios_version: "F16e".to_owned(),
        gpu_driver: "610.43.02".to_owned(),
        power_plan: "linux-governor:performance".to_owned(),
    }
}

fn pinned_toolchain(target_triple: &str) -> String {
    format!(
        "rustc 1.97.1 (8bab26f4f 2026-07-14)\nbinary: rustc\ncommit-hash: {PERFORMANCE_PINNED_RUSTC_COMMIT_HASH}\ncommit-date: 2026-07-14\nhost: {target_triple}\nrelease: 1.97.1\nLLVM version: 22.1.6",
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
fn linux_release_fingerprint_is_exact_on_invalidating_fields() {
    assert!(validate_linux_release_fingerprint(&fingerprint()).is_empty());
    let mut wrong = fingerprint();
    wrong.gpu_driver = "591.85".to_owned();
    assert_eq!(
        validate_linux_release_fingerprint(&wrong),
        vec!["PERF_GPU_DRIVER_MISMATCH"]
    );
}

#[test]
fn preflight_recomputes_readiness_from_typed_evidence() {
    let mut preflight = PerformancePreflightV1 {
        cpu_load_percent: Some(PREFLIGHT_LOAD_PERCENT_EXCLUSIVE - 1),
        gpu_load_percent: Some(PREFLIGHT_LOAD_PERCENT_EXCLUSIVE - 1),
        free_ram_bytes: Some(MINIMUM_FREE_RAM_BYTES),
        cpu_clock_percent_of_maximum: Some(100),
        gpu_thermal_slowdown_active: Some(false),
        ready: false,
        diagnostics: vec!["STALE_DIAGNOSTIC".to_owned()],
    };
    preflight.recompute_readiness();
    assert!(preflight.ready);
    assert!(preflight.diagnostics.is_empty());
    preflight
        .validate_ready_evidence()
        .expect("complete typed preflight is ready");
    preflight.free_ram_bytes = Some(MINIMUM_FREE_RAM_BYTES - 1);
    assert_eq!(
        preflight.validate_ready_evidence(),
        Err(vec!["PERF_FREE_RAM_LIMIT_NOT_MET".to_owned()])
    );

    preflight.free_ram_bytes = Some(MINIMUM_FREE_RAM_BYTES);
    preflight.cpu_load_percent = Some(PREFLIGHT_LOAD_PERCENT_EXCLUSIVE);
    assert_eq!(
        preflight.validate_ready_evidence(),
        Err(vec!["PERF_CPU_LOAD_LIMIT_EXCEEDED".to_owned()])
    );

    preflight.cpu_load_percent = Some(PREFLIGHT_LOAD_PERCENT_EXCLUSIVE - 1);
    preflight.gpu_load_percent = Some(PREFLIGHT_LOAD_PERCENT_EXCLUSIVE);
    assert_eq!(
        preflight.validate_ready_evidence(),
        Err(vec!["PERF_GPU_LOAD_LIMIT_EXCEEDED".to_owned()])
    );
}

#[test]
fn schema_round_trip_rejects_unknown_fields() {
    let mut run = PerformanceRunV6::empty(
        PerformanceScenarioV1::Smoke,
        PerformanceModeV1::Report,
        "release",
    );
    run.target_fingerprint = Some(fingerprint());
    assert_eq!(run.schema_version, 6);
    assert_eq!(
        run.methodology.methodology_version,
        "nextengine-performance-v9"
    );
    assert_eq!(PERFORMANCE_REPORT_FILE_NAME, "performance-report-v6.json");
    assert_eq!(
        PERFORMANCE_BASELINE_FILE_NAME,
        "performance-baseline-v6.json"
    );
    let json = serde_json::to_vec(&run).expect("serialize run");
    let decoded: PerformanceRunV6 = serde_json::from_slice(&json).expect("decode run");
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

    for (schema_version, methodology_version) in [
        (2, "nextengine-performance-v2"),
        (3, "nextengine-performance-v3"),
    ] {
        let mut historical: PerformanceRunV6 =
            serde_json::from_slice(&json).expect("decode current fixture");
        historical.schema_version = schema_version;
        historical.methodology.methodology_version = methodology_version.to_owned();
        let diagnostics = historical
            .validate_wire_version()
            .expect_err("historical wire versions are unsupported");
        assert!(diagnostics.contains(&"PERF_RUN_SCHEMA_MISMATCH".to_owned()));
        assert!(diagnostics.contains(&"PERF_RUN_METHODOLOGY_MISMATCH".to_owned()));
    }

    for prior in [
        "nextengine-performance-v4",
        "nextengine-performance-v5",
        "nextengine-performance-v6",
        "nextengine-performance-v7",
        "nextengine-performance-v8",
    ] {
        let mut prior_methodology: PerformanceRunV6 =
            serde_json::from_slice(&json).expect("decode current fixture");
        prior_methodology.methodology.methodology_version = prior.to_owned();
        let diagnostics = prior_methodology
            .validate_wire_version()
            .expect_err("prior methodology is incompatible with current admission");
        assert_eq!(
            diagnostics,
            vec!["PERF_RUN_METHODOLOGY_MISMATCH".to_owned()]
        );
    }

    let mut value: serde_json::Value = serde_json::from_slice(&json).expect("decode JSON value");
    value
        .as_object_mut()
        .expect("run object")
        .insert("unknown".to_owned(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<PerformanceRunV6>(value).is_err());
}

#[test]
fn strict_build_provenance_pins_commit_release_and_host() {
    let commit = "a".repeat(40);
    let toolchain = pinned_toolchain(PERFORMANCE_LINUX_TARGET_TRIPLE);
    validate_performance_build_provenance(&commit, &toolchain, PERFORMANCE_LINUX_TARGET_TRIPLE)
        .expect("exact pinned native provenance");

    assert_eq!(
        validate_performance_build_provenance(
            "UNKNOWN",
            &toolchain,
            PERFORMANCE_LINUX_TARGET_TRIPLE,
        ),
        Err(vec!["PERF_BUILD_COMMIT_INVALID".to_owned()])
    );
    let wrong_release = toolchain.replace("release: 1.97.1", "release: 1.96.1");
    assert_eq!(
        validate_performance_build_provenance(
            &commit,
            &wrong_release,
            PERFORMANCE_LINUX_TARGET_TRIPLE,
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
            PERFORMANCE_LINUX_TARGET_TRIPLE,
        ),
        Err(vec!["PERF_BUILD_RUSTC_COMMIT_MISMATCH".to_owned()])
    );
    assert_eq!(
        validate_performance_build_provenance(
            &commit,
            &toolchain,
            PERFORMANCE_WINDOWS_TARGET_TRIPLE,
        ),
        Err(vec![
            "PERF_BUILD_TARGET_UNSUPPORTED".to_owned(),
            "PERF_BUILD_TOOLCHAIN_HOST_MISMATCH".to_owned(),
        ])
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
    let mut run = PerformanceRunV6::empty(
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
fn v4_hard_counters_require_complete_low_overhead_evidence() {
    let mut counters = PerformanceResourceCountersV4 {
        process_peak_working_set_bytes: Some(1024),
        device_resident_bytes: Some(512),
        io_read_bytes: Some(128),
        io_write_bytes: Some(64),
        logical_resource_charges: Some(logical_resource_charges()),
        vulkan_timestamp_queries: 2,
        unavailable: Vec::new(),
    };
    counters
        .validate_for_hard_timing(PerformanceScenarioV1::R2AlphaRender)
        .expect("V4 hard evidence is complete");

    counters.vulkan_timestamp_queries = 0;
    counters
        .validate_for_hard_timing(PerformanceScenarioV1::R5Physics16)
        .expect("CPU PhysX hard evidence does not fabricate Vulkan timestamps");

    counters.vulkan_timestamp_queries = 2;
    counters.logical_resource_charges = None;
    assert_eq!(
        counters.validate_for_hard_timing(PerformanceScenarioV1::R2AlphaRender),
        Err(vec![
            "PERF_REQUIRED_COUNTER_MISSING: logical_resource_charges".to_owned()
        ])
    );
}

#[test]
fn no_device_workload_declares_zero_residency_without_fabricating_timestamps() {
    let mut counters = PerformanceResourceCountersV4 {
        process_peak_working_set_bytes: Some(1024),
        device_resident_bytes: None,
        io_read_bytes: Some(128),
        io_write_bytes: Some(64),
        logical_resource_charges: Some(logical_resource_charges()),
        vulkan_timestamp_queries: 0,
        unavailable: vec![
            "device residency requires a representative Vulkan workload".to_owned(),
            "Vulkan timestamps require a representative render workload".to_owned(),
        ],
    };

    counters.declare_no_device_workload();

    assert_eq!(counters.device_resident_bytes, Some(0));
    assert!(counters.unavailable.is_empty());
    counters
        .validate_for_hard_timing(PerformanceScenarioV1::R3MultiregionStreaming)
        .expect("CPU-only streaming evidence is complete");
}

#[test]
fn r5_physics_methodology_binds_the_production_humanoid_workload() {
    let scenario = PerformanceScenarioV1::R5Physics16;
    let methodology = methodology_for(scenario);
    assert_eq!(methodology.warmup_samples, 240);
    assert_eq!(methodology.measured_samples, 10_000);
    assert!(
        methodology
            .notes
            .iter()
            .any(|note| note.contains("240 Hz physics"))
    );
    assert_eq!(
        performance_scenario_hash(scenario),
        sha256_hex(
            b"nextengine.performance.r5-physics-16.v2:slots=16:dof=23:physics=240hz:motor=60hz:warmup-substeps-per-slot=240:measured-substeps-per-slot=10000:workers=1+4+8:fixed-standing-controller:fresh-scene-restore:adr062-budgets:hard-host=ref-linux-b550i-3950x-rtx3080-v1:exact-worker-root-parity:logical-accounting=r5-physics-16-v1"
        )
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
fn production_worker_scenario_has_distinct_versioned_methodology() {
    let run = PerformanceRunV6::empty(
        PerformanceScenarioV1::ProductionWorkerSoak,
        PerformanceModeV1::Report,
        "release",
    );
    let smoke = PerformanceRunV6::empty(
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
    assert_eq!(
        PerformanceScenarioV1::R3MultiregionStreaming.unavailable_reason(),
        None
    );
    assert_eq!(
        performance_scenario_hash(scenario),
        sha256_hex(
            b"nextengine.performance.r2-alpha-render.v4:reference-alpha:frontier-relay:desktop-views=exploration+combat+ui-dialogue:hard-host=ref-linux-b550i-3950x-rtx3080-v1:profiles=primary-1920x1080+fallback-b0-safe-1280x720p30:each=600-warmup+3600-measured:critical=max-cpu-extract-submit-gpu:retain-all:resource-window=sequential-six-window-production-vulkan:logical-accounting=r2-alpha-render-v1"
        )
    );
    let methodology = methodology_for(scenario);
    assert_eq!(methodology.warmup_samples, 3_600);
    assert_eq!(methodology.measured_samples, 21_600);
    assert_eq!(
        methodology.frame_critical_path.as_deref(),
        Some("max(cpu_extract_and_submit_us,gpu_timestamp_duration_us)")
    );
    assert!(
        methodology
            .notes
            .iter()
            .any(|note| note.contains("sole hard release host"))
    );
}

#[test]
fn r4_population_is_an_available_hard_production_workload() {
    let scenario = PerformanceScenarioV1::R4_100Npc;
    assert_eq!(scenario.unavailable_reason(), None);
    assert_eq!(
        performance_scenario_hash(scenario),
        sha256_hex(
            b"nextengine.performance.r4-100npc.v2:reference-alpha:npcs=100:cadence=16x3+32x15+52x60:warmup=1000:measured=10000:production-joint-world-services-tick:engine-graph-navigation:adr016-budgets:hard-host=ref-linux-b550i-3950x-rtx3080-v1:exact-due-trace:no-starvation:logical-accounting=r4-100npc-v1"
        )
    );
    let methodology = methodology_for(scenario);
    assert_eq!(methodology.warmup_samples, 1_000);
    assert_eq!(methodology.measured_samples, 10_000);
    assert!(
        methodology
            .notes
            .iter()
            .any(|note| note.contains("16 active / 32 near / 52 background"))
    );
}

#[test]
fn incompatible_host_is_not_a_conditional_pass() {
    let mut wrong = fingerprint();
    wrong.hostname = "OTHER".to_owned();
    assert_eq!(
        validate_linux_release_fingerprint(&wrong),
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
        bootstrap_median_change_interval(&[100; 16], &[100; 16], 200),
        Ok([0, 0])
    );
    assert_eq!(
        bootstrap_median_change_interval(&[106; 16], &[100; 16], 200),
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
    instrumentation.recorded_spans = ["navigation", "tier-cognition", "runtime-stages"]
        .into_iter()
        .map(|category| PerformanceSpanV1 {
            category: category.to_owned(),
            thread_index: 0,
            duration_microseconds: 1,
        })
        .collect();
    assert_eq!(instrumentation.validate(), Ok(()));
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
    let mut run = PerformanceRunV6::empty(
        PerformanceScenarioV1::R3MultiregionStreaming,
        PerformanceModeV1::Report,
        "release",
    );
    run.commit = "a".repeat(40);
    run.worktree_clean = true;
    run.target_triple = PERFORMANCE_LINUX_TARGET_TRIPLE.to_owned();
    run.toolchain = pinned_toolchain(&run.target_triple);
    run.target_fingerprint = Some(fingerprint());
    let ready_environment = PerformancePreflightV1 {
        cpu_load_percent: Some(0),
        gpu_load_percent: Some(0),
        free_ram_bytes: Some(MINIMUM_FREE_RAM_BYTES),
        cpu_clock_percent_of_maximum: Some(100),
        gpu_thermal_slowdown_active: Some(false),
        ready: true,
        diagnostics: Vec::new(),
    };
    run.preflight = Some(ready_environment.clone());
    run.environment_samples = vec![ready_environment.clone(), ready_environment];
    run.content_hash = "b".repeat(64);
    run.scenario_hash = performance_scenario_hash(run.scenario);
    run.metrics = vec![
        PerformanceMetricV1::from_samples(
            "r3-multiregion-streaming.total",
            "microseconds",
            vec![10, 11, 12],
            canonical_budget_for_metric(run.scenario, "r3-multiregion-streaming.total"),
        )
        .expect("metric"),
    ];
    run.instrumentation.enabled = true;
    run.instrumentation.authoritative_hash_parity = Some(true);
    run.instrumentation.overhead_basis_points = Some(0);
    run.resource_counters = PerformanceResourceCountersV4 {
        process_peak_working_set_bytes: Some(1),
        device_resident_bytes: Some(1),
        io_read_bytes: Some(0),
        io_write_bytes: Some(0),
        logical_resource_charges: Some(logical_resource_charges()),
        vulkan_timestamp_queries: 2,
        unavailable: Vec::new(),
    };
    run.verdict = PerformanceVerdict::ReportOnly;
    run.authoritative_hashes
        .insert("state".to_owned(), "d".repeat(64));
    let runs = vec![run; 10];
    let baseline = PerformanceBaselineV6::from_runs(&runs).expect("baseline");
    assert_eq!(baseline.calibration_runs, 10);
    assert_eq!(baseline.target_triple, PERFORMANCE_LINUX_TARGET_TRIPLE);
    assert_eq!(baseline.metrics[0].raw_samples.len(), 30);
    assert_eq!(baseline.metrics[0].sample_run_lengths, vec![3; 10]);

    let mut candidate = runs[0].clone();
    candidate.mode = PerformanceModeV1::Gate;
    candidate.evidence_runs = HARD_GATE_EVIDENCE_RUNS;
    candidate.environment_samples = vec![
        candidate.preflight.clone().expect("ready environment");
        usize::try_from(HARD_GATE_EVIDENCE_RUNS * 2)
            .expect("environment sample count")
    ];
    candidate.metrics = vec![
        PerformanceMetricV1::from_sample_runs(
            "r3-multiregion-streaming.total",
            "microseconds",
            vec![vec![11, 12], vec![11, 12], vec![19, 20]],
            canonical_budget_for_metric(candidate.scenario, "r3-multiregion-streaming.total"),
        )
        .expect("candidate metric"),
    ];
    compare_metrics_to_baseline(&mut candidate, &baseline).expect("run-level comparison");
    let relative = candidate.metrics[0]
        .relative
        .as_ref()
        .expect("relative evidence");
    assert_eq!(candidate.metrics[0].p95, 20);
    assert_eq!(relative.change_basis_points, 0);
    assert_eq!(candidate.metrics[0].verdict, PerformanceVerdict::Pass);
}

#[test]
fn aggregated_metrics_preserve_run_boundaries_and_fail_closed_on_absolute_tails() {
    let metric = PerformanceMetricV1::from_sample_runs(
        "frame",
        "microseconds",
        vec![vec![10, 11, 12], vec![9, 10, 30], vec![10, 10, 11]],
        Some(PerformanceBudgetV1 {
            p95_max: Some(20),
            p99_max: Some(20),
        }),
    )
    .expect("aggregate metric");

    assert_eq!(metric.sample_run_lengths, vec![3, 3, 3]);
    assert_eq!(metric.p50, 10);
    assert_eq!(metric.p95, 30);
    assert_eq!(metric.p99, 30);
    assert_eq!(metric.verdict, PerformanceVerdict::Fail);
}

#[test]
fn malformed_metric_run_boundaries_are_rejected() {
    let mut metric = PerformanceMetricV1::from_samples(
        "frame",
        "microseconds",
        vec![10, 11, 12],
        Some(PerformanceBudgetV1 {
            p95_max: Some(20),
            p99_max: Some(20),
        }),
    )
    .expect("metric");
    metric.sample_run_lengths = vec![2];

    assert_eq!(
        metric.validate_samples_and_budget(),
        Err(vec![
            "PERF_METRIC_RUN_BOUNDARY_LENGTH_MISMATCH: frame".to_owned()
        ])
    );
}

#[test]
fn baseline_centrally_rejects_an_incompatible_run_wire_version() {
    let mut run = PerformanceRunV6::empty(
        PerformanceScenarioV1::R3MultiregionStreaming,
        PerformanceModeV1::Report,
        "release",
    );
    run.schema_version = 1;
    run.commit = "a".repeat(40);
    run.worktree_clean = true;
    run.target_triple = PERFORMANCE_LINUX_TARGET_TRIPLE.to_owned();
    run.toolchain = pinned_toolchain(&run.target_triple);
    run.target_fingerprint = Some(fingerprint());

    let diagnostics = PerformanceBaselineV6::from_runs(&vec![run; 10])
        .expect_err("from_runs owns wire admission");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic == "PERF_BASELINE_RUN_INVALID: 0: PERF_RUN_SCHEMA_MISMATCH"
    }));
}
