use super::*;

#[test]
fn performance_cli_defaults_to_smoke_report() {
    let request = parse_arguments(std::iter::empty()).expect("default arguments");
    assert_eq!(
        request.scenario,
        xtask::performance::PerformanceScenarioV1::Smoke
    );
    assert_eq!(request.mode, xtask::performance::PerformanceModeV1::Report);
    assert_eq!(request.target, None);
}

#[test]
fn performance_gate_defaults_to_thoth_and_rejects_duplicate_flags() {
    let request = parse_arguments(
        ["--scenario", "r5-physics-16", "--mode", "gate"]
            .into_iter()
            .map(str::to_owned),
    )
    .expect("gate arguments");
    assert_eq!(
        request.target.as_deref(),
        Some(xtask::performance::THOTH_TARGET_ID)
    );
    assert!(
        parse_arguments(
            ["--mode", "report", "--mode", "gate"]
                .into_iter()
                .map(str::to_owned)
        )
        .is_err()
    );
}

#[test]
fn calibration_preflight_guard_is_explicit_and_unique() {
    let request = parse_arguments(
        [
            "--scenario",
            "r5-physics-16",
            "--mode",
            "report",
            "--require-ready-preflight",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("guarded report arguments");
    assert!(request.require_ready_preflight);
    assert!(
        parse_arguments(
            ["--require-ready-preflight", "--require-ready-preflight",]
                .into_iter()
                .map(str::to_owned),
        )
        .is_err()
    );
}

#[test]
fn performance_cli_accepts_the_report_only_long_session_soak() {
    let request = parse_arguments(
        ["--scenario", "long-session-soak", "--mode", "report"]
            .into_iter()
            .map(str::to_owned),
    )
    .expect("long-session soak arguments");
    assert_eq!(
        request.scenario,
        xtask::performance::PerformanceScenarioV1::LongSessionSoak
    );
    assert_eq!(request.mode, xtask::performance::PerformanceModeV1::Report);
    assert_eq!(request.target, None);
}

#[test]
fn performance_cli_accepts_the_report_only_interactive_frame_soak() {
    let request = parse_arguments(
        ["--scenario", "interactive-frame-soak", "--mode", "report"]
            .into_iter()
            .map(str::to_owned),
    )
    .expect("interactive frame soak arguments");
    assert_eq!(
        request.scenario,
        xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
    );
    assert_eq!(request.mode, xtask::performance::PerformanceModeV1::Report);
    assert_eq!(request.target, None);
}

#[test]
fn performance_cli_accepts_the_report_only_production_worker_soak() {
    let request = parse_arguments(
        ["--scenario", "production-worker-soak", "--mode", "report"]
            .into_iter()
            .map(str::to_owned),
    )
    .expect("production worker soak arguments");
    assert_eq!(
        request.scenario,
        xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak
    );
    assert_eq!(request.mode, xtask::performance::PerformanceModeV1::Report);
    assert_eq!(request.target, None);
    assert_eq!(
        performance_scenario_hash(request.scenario),
        xtask::performance::sha256_hex(
            b"nextengine.performance.production-worker-soak.v3:240-fifo-main-callbacks:60hz:bounded-sync-queue:next-simulation-worker:fixed-step-application:shared-presentation-publication:main-snapshot-read:resource-observation=production-worker-diagnostic-only"
        )
    );
}

#[test]
fn production_worker_details_are_versioned_and_round_trip_strictly() {
    let worker = ProductionWorkerPerformanceDetailsV1 {
        diagnostic_schema_version: 1,
        diagnostic_methodology_version: "production-worker-soak.v1".to_owned(),
        callbacks: 240,
        callback_cadence_hz: 60,
        queue_capacity: 8,
        queue_high_water: 8,
        submitted_callbacks: 240,
        processed_callbacks: 240,
        fixed_steps: 120,
        ordinary_fixed_steps: 116,
        lifecycle_boundary_fixed_steps: 4,
        snapshot_publications: 121,
        snapshot_reads: 240,
        fresh_snapshot_reads: 120,
        dropped_callbacks: 0,
        reordered_callbacks: 0,
        final_snapshot_sequence: 120,
        final_simulation_tick: 120,
        authoritative_state_root: "a".repeat(64),
        command_archive_root: "b".repeat(64),
        command_identity_index_root: "c".repeat(64),
        command_ledger_hash: "d".repeat(64),
    };
    let run = xtask::performance::PerformanceRunV4::empty(
        xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak,
        xtask::performance::PerformanceModeV1::Report,
        "release",
    );
    let report = performance_command_report(run, None, None, None, None, Some(worker.clone()));
    let json = report.to_json().expect("serialize worker report");
    let decoded: CommandReportV1<PerformanceDetailsV1> =
        serde_json::from_str(&json).expect("decode worker report");
    assert_eq!(decoded.details.production_worker, Some(worker));

    let mut value: serde_json::Value =
        serde_json::from_str(&json).expect("decode worker report value");
    value["details"]["production_worker"]["unknown"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<CommandReportV1<PerformanceDetailsV1>>(value).is_err());
}

#[test]
fn diagnostic_baseline_comparison_cannot_promote_report_only_to_a_gate() {
    for scenario in [
        xtask::performance::PerformanceScenarioV1::Smoke,
        xtask::performance::PerformanceScenarioV1::LongSessionSoak,
        xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak,
        xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak,
        xtask::performance::PerformanceScenarioV1::R2AlphaRender,
        xtask::performance::PerformanceScenarioV1::R3MultiregionStreaming,
    ] {
        assert_eq!(
            preserve_report_only_scenario_verdict(
                scenario,
                xtask::performance::PerformanceModeV1::Report,
                xtask::performance::PerformanceVerdict::Fail,
            ),
            xtask::performance::PerformanceVerdict::ReportOnly,
        );
        assert_eq!(
            preserve_report_only_scenario_verdict(
                scenario,
                xtask::performance::PerformanceModeV1::Report,
                xtask::performance::PerformanceVerdict::NotRun,
            ),
            xtask::performance::PerformanceVerdict::NotRun,
        );
    }
    assert_eq!(
        preserve_report_only_scenario_verdict(
            xtask::performance::PerformanceScenarioV1::R5Physics16,
            xtask::performance::PerformanceModeV1::Gate,
            xtask::performance::PerformanceVerdict::Fail,
        ),
        xtask::performance::PerformanceVerdict::Fail,
    );

    assert_eq!(
        report_only_gate_diagnostic(
            xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak,
        ),
        Some(
            "PERF_PRODUCTION_WORKER_SOAK_REPORT_ONLY: the production worker soak is diagnostic and cannot gate",
        ),
    );
    assert_eq!(
        report_only_gate_diagnostic(
            xtask::performance::PerformanceScenarioV1::R3MultiregionStreaming,
        ),
        Some(
            "PERF_R3_MULTIREGION_STREAMING_REPORT_ONLY: B-12 and the clean ten-run THOTH hard gate remain open",
        ),
    );
}

#[test]
fn scenario_dispatch_keeps_large_workload_branches_out_of_line() {
    const SOURCE: &str = include_str!("workloads.rs");
    const HELPERS: [&str; 5] = [
        "run_smoke_scenario_workloads",
        "run_long_session_scenario_workloads",
        "run_production_worker_scenario_workloads",
        "run_interactive_frame_scenario_workloads",
        "run_r2_alpha_render_scenario_workloads",
    ];

    let dispatcher = SOURCE
        .split_once("pub(super) fn run_scenario_workloads(")
        .and_then(|(_, remainder)| remainder.split_once("#[inline(never)]"))
        .map(|(dispatcher, _)| dispatcher)
        .expect("thin scenario dispatcher source");

    for helper in HELPERS {
        assert!(
            dispatcher.contains(helper),
            "dispatcher must delegate to {helper}"
        );
        let declaration = format!("fn {helper}");
        let helper_prefix = SOURCE
            .split_once(&declaration)
            .map(|(prefix, _)| prefix)
            .expect("scenario helper declaration");
        assert!(
            helper_prefix.trim_end().ends_with("#[inline(never)]"),
            "{helper} must remain an out-of-line stack boundary"
        );
    }
    assert!(
        !dispatcher.contains("ScenarioResourceWindow::begin"),
        "measurement owners must not return to the shared dispatcher frame"
    );
}
