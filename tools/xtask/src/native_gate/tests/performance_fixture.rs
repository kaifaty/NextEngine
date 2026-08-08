pub(super) fn performance_run_value(
    commit: &str,
    target_triple: &str,
    streaming_world: &str,
    agent_plan: &str,
    render_frame_plan: &str,
    live_runtime_state: &str,
) -> serde_json::Value {
    let scenario = crate::performance::PerformanceScenarioV1::Smoke;
    let mut run = crate::performance::PerformanceRunV4::empty(
        scenario,
        crate::performance::PerformanceModeV1::Report,
        "debug",
    );
    run.scenario_hash = crate::performance::performance_scenario_hash(scenario);
    run.commit = commit.to_owned();
    run.worktree_clean = true;
    run.target_triple = target_triple.to_owned();
    run.toolchain = format!(
        "rustc 1.93.0 (254b59607 2026-01-19)\nbinary: rustc\ncommit-hash: {}\ncommit-date: 2026-01-19\nhost: {target_triple}\nrelease: 1.93.0\nLLVM version: 21.1.0",
        crate::performance::PERFORMANCE_PINNED_RUSTC_COMMIT_HASH,
    );
    let windows = target_triple == crate::performance::PERFORMANCE_WINDOWS_TARGET_TRIPLE;
    run.target_fingerprint = Some(crate::performance::PerformanceTargetFingerprintV1 {
        target_id: "observed-host-v1".to_owned(),
        hostname: "native-fixture".to_owned(),
        cpu_model: "fixture-cpu".to_owned(),
        physical_cores: 8,
        logical_threads: 16,
        gpu_model: "fixture-gpu".to_owned(),
        gpu_vram_mib: 1,
        ram_bytes: 1,
        storage_model: "fixture-storage".to_owned(),
        storage_bytes: 1,
        os_name: if windows { "Windows" } else { "Linux" }.to_owned(),
        os_build: "fixture-build".to_owned(),
        bios_version: "fixture-bios".to_owned(),
        gpu_driver: "fixture-driver".to_owned(),
        power_plan: "fixture-power".to_owned(),
    });
    run.preflight = Some(crate::performance::PerformancePreflightV1 {
        cpu_load_percent: Some(0),
        gpu_load_percent: Some(0),
        free_ram_bytes: Some(crate::performance::MINIMUM_FREE_RAM_BYTES),
        cpu_clock_percent_of_maximum: Some(100),
        gpu_thermal_slowdown_active: Some(false),
        ready: true,
        diagnostics: Vec::new(),
    });
    run.verdict = crate::performance::PerformanceVerdict::ReportOnly;
    for (name, value) in [
        ("streaming_world", streaming_world),
        ("agent_plan", agent_plan),
        ("render_frame_plan", render_frame_plan),
        ("live_runtime_state", live_runtime_state),
    ] {
        run.authoritative_hashes
            .insert(name.to_owned(), value.to_owned());
    }
    serde_json::to_value(run).expect("serialize native-gate performance run")
}
