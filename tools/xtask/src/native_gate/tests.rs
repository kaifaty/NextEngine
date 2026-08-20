use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

mod comparison_roots;
mod package_validation;
mod performance_fixture;

use performance_fixture::performance_run_value;

static NEXT_TEMP_BUNDLE: AtomicU64 = AtomicU64::new(0);

pub(super) struct TempBundle {
    pub(super) root: PathBuf,
}

impl TempBundle {
    pub(super) fn new() -> Self {
        let sequence = NEXT_TEMP_BUNDLE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "nextengine-native-gate-test-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create isolated target bundle");
        Self { root }
    }

    pub(super) fn report_path(&self) -> PathBuf {
        self.root.join(TARGET_REPORT_FILE)
    }
}

impl Drop for TempBundle {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub(super) fn hash(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn remote_not_run() -> NativeGateTargetExecutionStatusV1 {
    NativeGateTargetExecutionStatusV1::NotRun {
        reason: "TARGET_EXECUTION_UNAVAILABLE".to_owned(),
    }
}

fn roots() -> NativeGateComparableRootsV1 {
    let mut roots = NativeGateComparableRootsV1 {
        project_composition_lock_hash: hash('a'),
        schema_registry_hash: hash('b'),
        content_manifest_hash: hash('c'),
        mechanics_lock_hash: hash('d'),
        world_partition_hash: hash('e'),
        luau_manifest_hash: hash('1'),
        wasm_manifest_hash: hash('2'),
        wit_v2_hash: hash('3'),
        wit_v3_hash: hash('4'),
        extension_compatibility_hash: hash('5'),
        play_state_root: hash('6'),
        play_ledger_hash: hash('7'),
        replay_state_root: hash('8'),
        replay_ledger_hash: hash('9'),
        platform_state_root: hash('a'),
        platform_ledger_hash: hash('b'),
        presentation_snapshot_hash: hash('c'),
        streaming_performance_hash: hash('d'),
        agent_performance_hash: hash('e'),
        audio_scene_pcm_digest: hash('0'),
        packaged_game_state_root: hash('1'),
        packaged_game_ledger_hash: hash('2'),
        packaged_headless_state_root: hash('1'),
        packaged_headless_ledger_hash: hash('2'),
        closure_hash: hash('3'),
        windows_package_descriptor_hash: hash('4'),
        linux_package_descriptor_hash: hash('5'),
    };
    roots.windows_package_descriptor_hash =
        target_package_descriptor_hash(WINDOWS_TARGET_TRIPLE, &roots);
    roots.linux_package_descriptor_hash =
        target_package_descriptor_hash(LINUX_TARGET_TRIPLE, &roots);
    roots.closure_hash = closure_hash(&roots);
    roots
}

fn closure_targets(
    native_target: &str,
    roots: &NativeGateComparableRootsV1,
) -> NativeGateClosureTargetSetV1 {
    NativeGateClosureTargetSetV1 {
        windows: NativeGateClosureTargetSummaryV1 {
            target_triple: WINDOWS_TARGET_TRIPLE.to_owned(),
            package_descriptor_hash: roots.windows_package_descriptor_hash.clone(),
            runtime_check: if native_target == WINDOWS_TARGET_TRIPLE {
                NativeGateTargetExecutionStatusV1::Pass
            } else {
                remote_not_run()
            },
            desktop_smoke: if native_target == WINDOWS_TARGET_TRIPLE {
                NativeGateTargetExecutionStatusV1::Pass
            } else {
                remote_not_run()
            },
        },
        linux: NativeGateClosureTargetSummaryV1 {
            target_triple: LINUX_TARGET_TRIPLE.to_owned(),
            package_descriptor_hash: roots.linux_package_descriptor_hash.clone(),
            runtime_check: if native_target == LINUX_TARGET_TRIPLE {
                NativeGateTargetExecutionStatusV1::Pass
            } else {
                remote_not_run()
            },
            desktop_smoke: if native_target == LINUX_TARGET_TRIPLE {
                NativeGateTargetExecutionStatusV1::Pass
            } else {
                remote_not_run()
            },
        },
    }
}

fn checks() -> Vec<NativeGateCheckRecordV1> {
    NativeGateCheckNameV1::ORDERED
        .into_iter()
        .enumerate()
        .map(|(index, check)| NativeGateCheckRecordV1 {
            check,
            status: NativeGateCheckStatusV1::Pass,
            report_path: Some(check.report_path()),
            report_sha256: Some(hash('a')),
            diagnostic: None,
            elapsed_milliseconds: u64::try_from(index + 1).expect("small test index"),
        })
        .collect()
}

fn package(target: &str, roots: &NativeGateComparableRootsV1) -> NativeGatePackageSummaryV1 {
    NativeGatePackageSummaryV1 {
        status: NativeGateRunStatusV1::Pass,
        target_triple: target.to_owned(),
        relative_path: "package".to_owned(),
        package_manifest_sha256: hash('6'),
        project_lock_sha256: roots.project_composition_lock_hash.clone(),
        schema_registry_sha256: roots.schema_registry_hash.clone(),
        content_manifest_sha256: roots.content_manifest_hash.clone(),
        mechanics_lock_sha256: roots.mechanics_lock_hash.clone(),
        world_partition_sha256: roots.world_partition_hash.clone(),
        game_binary_sha256: hash('7'),
        headless_binary_sha256: hash('8'),
        game: NativeGatePackagedLaunchSummaryV1 {
            status: NativeGateCheckStatusV1::Pass,
            state_root: roots.packaged_game_state_root.clone(),
            ledger_hash: roots.packaged_game_ledger_hash.clone(),
        },
        headless: NativeGatePackagedLaunchSummaryV1 {
            status: NativeGateCheckStatusV1::Pass,
            state_root: roots.packaged_headless_state_root.clone(),
            ledger_hash: roots.packaged_headless_ledger_hash.clone(),
        },
    }
}

pub(super) fn report(target: &str) -> NativeGateTargetReportV1 {
    let comparable_roots = roots();
    NativeGateTargetReportV1 {
        schema_version: LEGACY_NATIVE_GATE_SCHEMA_VERSION,
        status: NativeGateRunStatusV1::Pass,
        git_commit_sha: "a".repeat(40),
        cargo_lock_sha256: hash('b'),
        rustc_release: NATIVE_GATE_RUSTC_RELEASE.to_owned(),
        target_triple: target.to_owned(),
        checks: checks(),
        closure_targets: Some(closure_targets(target, &comparable_roots)),
        package: Some(package(target, &comparable_roots)),
        comparable_roots: Some(comparable_roots),
    }
}

pub(super) fn controlled_fail_report(target: &str) -> NativeGateTargetReportV1 {
    let mut report = report(target);
    report.status = NativeGateRunStatusV1::Fail;
    report.closure_targets = None;
    report.comparable_roots = None;
    report.package = None;
    let failed = report.checks.last_mut().expect("v1-package record");
    failed.status = NativeGateCheckStatusV1::Fail;
    failed.report_path = None;
    failed.report_sha256 = None;
    failed.diagnostic = Some(NativeGateDiagnosticV1 {
        code: "NATIVE_GATE_CHECK_FAILED".to_owned(),
        message: "controlled package failure".to_owned(),
    });
    report
}

pub(super) fn materialize_check_reports(
    bundle: &TempBundle,
    report: &mut NativeGateTargetReportV1,
) {
    fs::create_dir(bundle.root.join("checks")).expect("checks directory");
    for index in 0..report.checks.len() {
        let check = report.checks[index].check;
        let Some(relative_path) = report.checks[index].report_path.clone() else {
            continue;
        };
        let bytes =
            serde_json::to_vec(&check_report_value(check, report)).expect("serialize check report");
        fs::write(bundle.root.join(&relative_path), &bytes).expect("write check report");
        report.checks[index].report_sha256 = Some(sha256_hex(&bytes));
    }
}

fn replace_performance_check_report(
    bundle: &TempBundle,
    report: &mut NativeGateTargetReportV1,
    value: &serde_json::Value,
) {
    let index = report
        .checks
        .iter()
        .position(|record| record.check == NativeGateCheckNameV1::Performance)
        .expect("performance record");
    let bytes = serde_json::to_vec(value).expect("serialize performance report");
    let path = report.checks[index]
        .report_path
        .as_ref()
        .expect("performance report path");
    fs::write(bundle.root.join(path), &bytes).expect("replace performance report");
    report.checks[index].report_sha256 = Some(sha256_hex(&bytes));
}

fn check_report_value(
    check: NativeGateCheckNameV1,
    report: &NativeGateTargetReportV1,
) -> serde_json::Value {
    let roots = report.comparable_roots.clone().unwrap_or_else(roots);
    let targets = report
        .closure_targets
        .clone()
        .unwrap_or_else(|| closure_targets(&report.target_triple, &roots));
    let package = report
        .package
        .clone()
        .unwrap_or_else(|| package(&report.target_triple, &roots));
    match check {
        NativeGateCheckNameV1::HostCheck => serde_json::json!({
            "schema_version": 1,
            "status": "PASS",
            "command": "host-check",
            "details": {
                "host": report.target_triple,
                "rustc_release": report.rustc_release,
            }
        }),
        NativeGateCheckNameV1::Play => serde_json::json!({
            "schema_version": 1,
            "status": "PASS",
            "composition_root": "Tools",
            "session_id": "session",
            "close_receipt_hash": hash('a'),
            "close_result": "Saved",
            "final_save_generation_hash": hash('b'),
            "project_composition_lock_hash": roots.project_composition_lock_hash,
            "ticks": 1,
            "events": 1,
            "rpg_events": 1,
            "authoritative_revision": 1,
            "authoritative_state_root": roots.play_state_root,
            "command_archive_root": hash('c'),
            "command_identity_index_root": hash('d'),
            "command_ledger_hash": roots.play_ledger_hash,
            "interactive_host_object_count": 0,
            "presentation": null,
        }),
        NativeGateCheckNameV1::PersistenceReplay => serde_json::json!({
            "schema_version": 1,
            "status": "PASS",
            "command": "persistence-replay",
            "details": {
                "ticks": 1,
                "generations": 1,
                "rpg_events": 1,
                "interactive_object_state": "state",
                "dialogue_node": "node",
                "quest_state": "quest",
                "npc_player_trust": 0,
                "npc_health": 1,
                "player_health": 1,
                "agent_intent": hash('1'),
                "agent_projection": hash('2'),
                "luau_state_hash": hash('3'),
                "wasm_state_hash": hash('4'),
                "world_streaming_generation": 1,
                "current_chunk": "chunk",
                "final_state_root": roots.replay_state_root,
                "final_ledger_root": roots.replay_ledger_hash,
            }
        }),
        NativeGateCheckNameV1::ContentPackage => serde_json::json!({
            "schema_version": 1,
            "status": "PASS",
            "command": "content-package",
            "details": {
                "records": 1,
                "chunks": 1,
                "creator_records": 1,
                "creator_chunks": 1,
                "creator_composition_lock_hash": hash('5'),
                "mechanic_packages": 1,
                "luau_packages": 1,
                "wasm_plugins": 1,
                "combat_npc_health": 1,
                "scripted_player_health": 1,
                "wasm_player_health": 1,
                "luau_state_hash": hash('1'),
                "wasm_state_hash": hash('2'),
                "wasm_host_api_major": 1,
                "schema_registry_hash": roots.schema_registry_hash,
                "content_manifest_hash": roots.content_manifest_hash,
                "mechanics_lock_hash": roots.mechanics_lock_hash,
                "world_partition_hash": roots.world_partition_hash,
                "composition_lock_hash": roots.project_composition_lock_hash,
            }
        }),
        NativeGateCheckNameV1::Platform => serde_json::json!({
            "schema_version": 1,
            "status": "PASS",
            "command": "platform",
            "details": {
                "portable_contract": "PASS",
                "sdl_ash_candidate": "PASS",
                "normalized_events": 1,
                "rendered_objects": 1,
                "presentation_snapshot_hash": roots.presentation_snapshot_hash,
                "ledger_hash": roots.platform_ledger_hash,
                "state_root": roots.platform_state_root,
            }
        }),
        NativeGateCheckNameV1::Performance => serde_json::json!({
            "schema_version": 1,
            "status": "PASS",
            "command": "performance",
            "details": {
                "run": performance_run_value(
                    &report.git_commit_sha,
                    &report.target_triple,
                    &roots.streaming_performance_hash,
                    &roots.agent_performance_hash,
                    &hash('f'),
                    &hash('0'),
                ),
                "streaming": {
                    "cycles": 1,
                    "staged_asset_references": 1,
                    "elapsed_microseconds": 1,
                    "final_generation": 1,
                    "final_world_state_hash": roots.streaming_performance_hash,
                },
                "agent_planning": {
                    "cycles": 1,
                    "elapsed_microseconds": 1,
                    "final_plan_hash": roots.agent_performance_hash,
                },
                "render_planning": {
                    "cycles": 1,
                    "elapsed_microseconds": 1,
                    "visible_object_count": 1,
                    "indexed_draw_count": 1,
                    "fallback_material_draw_count": 0,
                    "frame_plan_hash": hash('f'),
                },
                "live_runtime": {
                    "ticks": 1,
                    "command_body_count": 1,
                    "elapsed_microseconds": 1,
                    "window_microseconds": [1, 1, 1],
                    "checkpoint_microseconds": [1, 1, 1],
                    "final_state_root": hash('0'),
                }
            }
        }),
        NativeGateCheckNameV1::V1Closure => serde_json::json!({
            "schema_version": 1,
            "status": "LOCAL_PASS_SHIPPING_TARGETS_NOT_RUN",
            "command": "v1-closure",
            "details": {
                "shipping_ready": false,
                "checks": ["content_package"],
                "project_composition_lock_hash": roots.project_composition_lock_hash,
                "schema_registry_hash": roots.schema_registry_hash,
                "content_manifest_hash": roots.content_manifest_hash,
                "mechanics_lock_hash": roots.mechanics_lock_hash,
                "world_partition_hash": roots.world_partition_hash,
                "luau_manifest_hash": roots.luau_manifest_hash,
                "wasm_manifest_hash": roots.wasm_manifest_hash,
                "wit_v2_hash": roots.wit_v2_hash,
                "wit_v3_hash": roots.wit_v3_hash,
                "extension_compatibility_hash": roots.extension_compatibility_hash,
                "play_state_root": roots.play_state_root,
                "play_ledger_hash": roots.play_ledger_hash,
                "replay_state_root": roots.replay_state_root,
                "replay_ledger_hash": roots.replay_ledger_hash,
                "audio_scene_pcm_digest": roots.audio_scene_pcm_digest,
                "windows": {
                    "target": targets.windows.target_triple,
                    "package_descriptor_hash": targets.windows.package_descriptor_hash,
                    "runtime_check": target_status(&targets.windows.runtime_check),
                    "desktop_smoke": target_status(&targets.windows.desktop_smoke),
                },
                "linux": {
                    "target": targets.linux.target_triple,
                    "package_descriptor_hash": targets.linux.package_descriptor_hash,
                    "runtime_check": target_status(&targets.linux.runtime_check),
                    "desktop_smoke": target_status(&targets.linux.desktop_smoke),
                },
                "closure_hash": roots.closure_hash,
            }
        }),
        NativeGateCheckNameV1::V1Package => serde_json::json!({
            "schema_version": 1,
            "status": "PASS",
            "command": "v1-package",
            "details": {
                "target": package.target_triple,
                "output": "package",
                "package_manifest_hash": package.package_manifest_sha256,
                "composition_lock_hash": package.project_lock_sha256,
                "game_binary_hash": package.game_binary_sha256,
                "headless_binary_hash": package.headless_binary_sha256,
                "game_launch": "PASS",
                "headless_launch": "PASS",
            }
        }),
    }
}

fn target_status(status: &NativeGateTargetExecutionStatusV1) -> String {
    match status {
        NativeGateTargetExecutionStatusV1::Pass => "PASS".to_owned(),
        NativeGateTargetExecutionStatusV1::NotRun { reason } => format!("NOT_RUN({reason})"),
    }
}

pub(super) fn write_target_report(bundle: &TempBundle, report: &NativeGateTargetReportV1) {
    let bytes = serde_json::to_vec(report).expect("serialize target report");
    fs::write(bundle.report_path(), bytes).expect("write target report");
}

fn package_manifest_from_summary(
    report: &NativeGateTargetReportV1,
) -> crate::package::PackageManifestV4 {
    let summary = report.package.as_ref().expect("package summary");
    let roots = crate::package::PackageTargetNeutralRootsV3 {
        content_manifest_sha256: summary.content_manifest_sha256.clone(),
        mechanics_lock_sha256: summary.mechanics_lock_sha256.clone(),
        project_lock_sha256: summary.project_lock_sha256.clone(),
        schema_registry_sha256: summary.schema_registry_sha256.clone(),
        world_partition_sha256: summary.world_partition_sha256.clone(),
    };
    let packaged_run = |composition_root: &str,
                        binary_path: &str,
                        binary_sha256: &str,
                        launch: &NativeGatePackagedLaunchSummaryV1| {
        crate::package::PackagedRunV2 {
            authoritative_state_root: launch.state_root.clone(),
            binary_path: binary_path.to_owned(),
            binary_sha256: binary_sha256.to_owned(),
            command_ledger_hash: launch.ledger_hash.clone(),
            composition_root: composition_root.to_owned(),
            launch_status: "PASS".to_owned(),
            project_composition_lock_hash: summary.project_lock_sha256.clone(),
        }
    };
    crate::package::PackageManifestV4 {
        binaries: crate::package::PackageBinariesV2 {
            game: packaged_run(
                "Game",
                "bin/next_game.exe",
                &summary.game_binary_sha256,
                &summary.game,
            ),
            headless: packaged_run(
                "Headless",
                "bin/next_headless.exe",
                &summary.headless_binary_sha256,
                &summary.headless,
            ),
        },
        file_inventory: Vec::new(),
        required_notices: Vec::new(),
        runtime_profile: crate::package::PackageRuntimeProfileV3 {
            abi: crate::package::PackageRuntimeAbiV3::WindowsMsvcX64 {
                crt: crate::package::PackageWindowsCrtV3::DynamicSystem,
            },
            binaries: Vec::new(),
            external_prerequisites: Vec::new(),
        },
        schema_version: crate::package::PACKAGE_MANIFEST_SCHEMA_VERSION,
        target_neutral_roots: roots,
        target_triple: report.target_triple.clone(),
    }
}

fn comparison_error(
    windows: &NativeGateTargetReportV1,
    linux: &NativeGateTargetReportV1,
) -> NativeGateComparisonError {
    compare_native_gate_reports(windows, linux).expect_err("comparison must fail")
}

#[test]
fn strict_schema_round_trips_and_rejects_unknown_fields_and_statuses() {
    let target_report = report(WINDOWS_TARGET_TRIPLE);
    let encoded = serde_json::to_string(&target_report).expect("serialize report");
    let decoded: NativeGateTargetReportV1 =
        serde_json::from_str(&encoded).expect("strict report round trip");
    assert_eq!(decoded, target_report);

    let mut value = serde_json::to_value(&target_report).expect("report value");
    value
        .as_object_mut()
        .expect("report object")
        .insert("hostname".to_owned(), serde_json::json!("forbidden"));
    assert!(serde_json::from_value::<NativeGateTargetReportV1>(value).is_err());

    let invalid_status = encoded.replacen("\"status\":\"PASS\"", "\"status\":\"UNKNOWN\"", 1);
    assert!(serde_json::from_str::<NativeGateTargetReportV1>(&invalid_status).is_err());

    let mut wrong_version = report(WINDOWS_TARGET_TRIPLE);
    wrong_version.schema_version = LEGACY_NATIVE_GATE_SCHEMA_VERSION + 1;
    assert_eq!(
        validate_native_gate_target_report(&wrong_version)
            .expect_err("unknown schema version")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );
}

#[test]
fn comparison_accepts_either_input_order_and_preserves_target_local_summaries() {
    let windows = report(WINDOWS_TARGET_TRIPLE);
    let linux = report(LINUX_TARGET_TRIPLE);
    let direct = compare_native_gate_reports(&windows, &linux).expect("native comparison");
    let reversed = compare_native_gate_reports(&linux, &windows).expect("reversed inputs");

    assert_eq!(direct, reversed);
    assert!(direct.native_gate_ready);
    assert_eq!(direct.status, NativeGateRunStatusV1::Pass);
    assert_eq!(direct.windows.target_triple, WINDOWS_TARGET_TRIPLE);
    assert_eq!(direct.linux.target_triple, LINUX_TARGET_TRIPLE);
    assert_eq!(direct.windows.total_elapsed_milliseconds, 36);
}

#[test]
fn derived_descriptor_and_closure_hashes_match_the_verification_oracle() {
    let oracle = next_verification::run_v1_closure_check().expect("v1 closure oracle");
    let legacy = roots();
    let roots = NativeGateReleaseRootsV2 {
        project_composition_lock_hash: oracle.project_composition_lock_hash.to_hex(),
        schema_registry_hash: oracle.schema_registry_hash.to_hex(),
        content_manifest_hash: oracle.content_manifest_hash.to_hex(),
        mechanics_lock_hash: oracle.mechanics_lock_hash.to_hex(),
        world_partition_hash: oracle.world_partition_hash.to_hex(),
        luau_manifest_hash: oracle.luau_manifest_hash.to_hex(),
        wasm_manifest_hash: oracle.wasm_manifest_hash.to_hex(),
        wit_v2_hash: oracle.wit_v2_hash.to_hex(),
        wit_v3_hash: oracle.wit_v3_hash.to_hex(),
        extension_compatibility_hash: oracle.extension_compatibility_hash.to_hex(),
        play_state_root: oracle.play_state_root.to_hex(),
        play_ledger_hash: oracle.play_ledger_hash.to_hex(),
        replay_state_root: oracle.replay_state_root.to_hex(),
        replay_ledger_hash: oracle.replay_ledger_hash.to_hex(),
        platform_state_root: legacy.platform_state_root,
        platform_ledger_hash: legacy.platform_ledger_hash,
        presentation_snapshot_hash: legacy.presentation_snapshot_hash,
        streaming_performance_hash: oracle.streaming_performance_hash.to_hex(),
        agent_performance_hash: oracle.agent_performance_hash.to_hex(),
        audio_scene_pcm_digest: oracle.audio_scene_pcm_digest.to_hex(),
        packaged_game_state_root: legacy.packaged_game_state_root,
        packaged_game_ledger_hash: legacy.packaged_game_ledger_hash,
        packaged_headless_state_root: legacy.packaged_headless_state_root,
        packaged_headless_ledger_hash: legacy.packaged_headless_ledger_hash,
        closure_hash: oracle.closure_hash.to_hex(),
        package_descriptor_hash: oracle.release_target.package_descriptor_hash.to_hex(),
    };
    assert_eq!(
        linux_package_descriptor_hash_v2(&roots),
        roots.package_descriptor_hash
    );
    assert_eq!(linux_closure_hash_v2(&roots), roots.closure_hash);
}

#[test]
fn comparison_ignores_target_specific_hashes_and_timings() {
    let windows = report(WINDOWS_TARGET_TRIPLE);
    let mut linux = report(LINUX_TARGET_TRIPLE);
    linux.checks[0].elapsed_milliseconds = u64::MAX;
    let package = linux.package.as_mut().expect("package");
    package.package_manifest_sha256 = hash('9');
    package.game_binary_sha256 = hash('a');
    package.headless_binary_sha256 = hash('b');

    let compared =
        compare_native_gate_reports(&windows, &linux).expect("target-local fields are ignored");
    assert_eq!(compared.linux.package_manifest_sha256, hash('9'));
    assert_eq!(compared.linux.game_binary_sha256, hash('a'));
    assert_eq!(compared.linux.total_elapsed_milliseconds, u64::MAX);
}

#[test]
fn comparison_rejects_duplicate_or_unsupported_target_sets() {
    let windows = report(WINDOWS_TARGET_TRIPLE);
    let duplicate = report(WINDOWS_TARGET_TRIPLE);
    let error = comparison_error(&windows, &duplicate);
    assert_eq!(error.code(), NATIVE_GATE_TARGET_SET_INVALID);

    let mut unsupported = report(LINUX_TARGET_TRIPLE);
    unsupported.target_triple = "aarch64-apple-darwin".to_owned();
    let error = comparison_error(&windows, &unsupported);
    assert_eq!(error.code(), NATIVE_GATE_TARGET_SET_INVALID);
}

#[test]
fn comparison_requires_two_complete_pass_reports() {
    let windows = report(WINDOWS_TARGET_TRIPLE);
    let mut linux = report(LINUX_TARGET_TRIPLE);
    linux.status = NativeGateRunStatusV1::Fail;
    linux.package = None;
    linux.comparable_roots = None;
    linux.closure_targets = None;
    linux.checks[0].status = NativeGateCheckStatusV1::Fail;
    linux.checks[0].report_path = None;
    linux.checks[0].report_sha256 = None;
    linux.checks[0].diagnostic = Some(NativeGateDiagnosticV1 {
        code: "NATIVE_GATE_CHECK_FAILED".to_owned(),
        message: "controlled failure".to_owned(),
    });
    for check in &mut linux.checks[1..] {
        check.status = NativeGateCheckStatusV1::NotRun;
        check.report_path = None;
        check.report_sha256 = None;
        check.diagnostic = Some(NativeGateDiagnosticV1 {
            code: "PRIOR_CHECK_FAILED".to_owned(),
            message: "prior check failed".to_owned(),
        });
    }

    validate_native_gate_target_report(&linux).expect("controlled FAIL report is valid");
    let error = comparison_error(&windows, &linux);
    assert_eq!(error.code(), NATIVE_GATE_REPORT_INVALID);
}

#[test]
fn comparison_rejects_commit_lock_and_toolchain_mismatches() {
    let windows = report(WINDOWS_TARGET_TRIPLE);

    let mut linux = report(LINUX_TARGET_TRIPLE);
    linux.git_commit_sha = "b".repeat(40);
    let error = comparison_error(&windows, &linux);
    assert_eq!(error.code(), NATIVE_GATE_COMMIT_MISMATCH);
    assert!(error.detail().contains("git_commit_sha"));

    let mut linux = report(LINUX_TARGET_TRIPLE);
    linux.cargo_lock_sha256 = hash('c');
    let error = comparison_error(&windows, &linux);
    assert_eq!(error.code(), NATIVE_GATE_COMMIT_MISMATCH);
    assert!(error.detail().contains("cargo_lock_sha256"));

    let mut linux = report(LINUX_TARGET_TRIPLE);
    linux.rustc_release = "1.94.0".to_owned();
    let error = comparison_error(&windows, &linux);
    assert_eq!(error.code(), NATIVE_GATE_REPORT_INVALID);
    assert!(error.detail().contains("rustc_release"));
}

#[test]
fn package_summary_is_validated_but_target_local_hashes_are_not_compared() {
    let windows = report(WINDOWS_TARGET_TRIPLE);
    let mut linux = report(LINUX_TARGET_TRIPLE);
    linux.package.as_mut().expect("package").target_triple = WINDOWS_TARGET_TRIPLE.to_owned();
    let error = comparison_error(&windows, &linux);
    assert_eq!(error.code(), NATIVE_GATE_PACKAGE_INVALID);

    let mut linux = report(LINUX_TARGET_TRIPLE);
    linux.package.as_mut().expect("package").game.state_root = hash('f');
    let error = comparison_error(&windows, &linux);
    assert_eq!(error.code(), NATIVE_GATE_PACKAGE_INVALID);
}

#[test]
fn fixed_check_order_paths_and_fail_chain_are_enforced() {
    let mut valid_fail = report(WINDOWS_TARGET_TRIPLE);
    valid_fail.status = NativeGateRunStatusV1::Fail;
    valid_fail.package = None;
    valid_fail.comparable_roots = None;
    valid_fail.closure_targets = None;
    let failed_index = 3;
    valid_fail.checks[failed_index].status = NativeGateCheckStatusV1::Fail;
    valid_fail.checks[failed_index].report_path = None;
    valid_fail.checks[failed_index].report_sha256 = None;
    valid_fail.checks[failed_index].diagnostic = Some(NativeGateDiagnosticV1 {
        code: "NATIVE_GATE_CHECK_FAILED".to_owned(),
        message: "content check failed".to_owned(),
    });
    for record in &mut valid_fail.checks[failed_index + 1..] {
        record.status = NativeGateCheckStatusV1::NotRun;
        record.report_path = None;
        record.report_sha256 = None;
        record.diagnostic = Some(NativeGateDiagnosticV1 {
            code: "PRIOR_CHECK_FAILED".to_owned(),
            message: "content-package failed".to_owned(),
        });
    }
    validate_native_gate_target_report(&valid_fail).expect("valid fail chain");

    let mut stale_projection = valid_fail.clone();
    stale_projection.closure_targets = report(WINDOWS_TARGET_TRIPLE).closure_targets;
    assert_eq!(
        validate_native_gate_target_report(&stale_projection)
            .expect_err("FAIL report must not retain closure projections")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );

    let mut unknown_diagnostic = valid_fail.clone();
    unknown_diagnostic.checks[failed_index]
        .diagnostic
        .as_mut()
        .expect("failed diagnostic")
        .code = "SOME_UNREGISTERED_FAILURE".to_owned();
    assert_eq!(
        validate_native_gate_target_report(&unknown_diagnostic)
            .expect_err("diagnostic code must be stable")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );

    let mut wrong_order = valid_fail.clone();
    wrong_order.checks.swap(0, 1);
    assert_eq!(
        validate_native_gate_target_report(&wrong_order)
            .expect_err("wrong check order")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );

    let mut wrong_path = report(WINDOWS_TARGET_TRIPLE);
    wrong_path.checks[0].report_path = Some("../host.json".to_owned());
    assert_eq!(
        validate_native_gate_target_report(&wrong_path)
            .expect_err("unsafe check path")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );

    let mut broken_chain = valid_fail;
    broken_chain.checks[6].status = NativeGateCheckStatusV1::Pass;
    broken_chain.checks[6].report_path = Some(NativeGateCheckNameV1::V1Closure.report_path());
    broken_chain.checks[6].report_sha256 = Some(hash('a'));
    broken_chain.checks[6].diagnostic = None;
    assert_eq!(
        validate_native_gate_target_report(&broken_chain)
            .expect_err("broken failure chain")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );
}

#[test]
fn native_and_remote_closure_target_states_are_enforced() {
    let mut windows = report(WINDOWS_TARGET_TRIPLE);
    windows
        .closure_targets
        .as_mut()
        .expect("targets")
        .windows
        .runtime_check = remote_not_run();
    assert_eq!(
        validate_native_gate_target_report(&windows)
            .expect_err("native check must pass")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );

    let mut windows = report(WINDOWS_TARGET_TRIPLE);
    windows
        .closure_targets
        .as_mut()
        .expect("targets")
        .linux
        .desktop_smoke = NativeGateTargetExecutionStatusV1::Pass;
    assert_eq!(
        validate_native_gate_target_report(&windows)
            .expect_err("remote check must not run")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );
}

#[test]
fn malformed_identity_hashes_and_diagnostics_are_rejected() {
    let mut target = report(WINDOWS_TARGET_TRIPLE);
    target.git_commit_sha = "ABC".to_owned();
    assert_eq!(
        validate_native_gate_target_report(&target)
            .expect_err("invalid commit")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );

    let mut target = report(WINDOWS_TARGET_TRIPLE);
    target.checks[0].status = NativeGateCheckStatusV1::Fail;
    target.checks[0].diagnostic = Some(NativeGateDiagnosticV1 {
        code: "not_stable".to_owned(),
        message: "failure".to_owned(),
    });
    target.status = NativeGateRunStatusV1::Fail;
    target.package = None;
    target.comparable_roots = None;
    target.closure_targets = None;
    for record in &mut target.checks[1..] {
        record.status = NativeGateCheckStatusV1::NotRun;
        record.report_path = None;
        record.report_sha256 = None;
        record.diagnostic = Some(NativeGateDiagnosticV1 {
            code: "PRIOR_CHECK_FAILED".to_owned(),
            message: "prior check failed".to_owned(),
        });
    }
    assert_eq!(
        validate_native_gate_target_report(&target)
            .expect_err("invalid diagnostic")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );
}

#[test]
fn bundle_validation_loads_strict_fail_report_and_hashes_every_present_check() {
    let bundle = TempBundle::new();
    let mut report = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
    materialize_check_reports(&bundle, &mut report);
    write_target_report(&bundle, &report);

    let validated =
        validate_native_gate_target_bundle(&bundle.report_path()).expect("valid FAIL bundle");
    assert_eq!(validated, report);
}

#[test]
fn bundle_validation_rejects_missing_or_tampered_check_reports() {
    let missing_bundle = TempBundle::new();
    let mut missing_report = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
    materialize_check_reports(&missing_bundle, &mut missing_report);
    fs::remove_file(
        missing_bundle
            .root
            .join(missing_report.checks[2].report_path.as_ref().expect("path")),
    )
    .expect("remove one check report");
    write_target_report(&missing_bundle, &missing_report);
    let error = validate_native_gate_target_bundle(&missing_bundle.report_path())
        .expect_err("missing check report");
    assert_eq!(error.code(), NATIVE_GATE_REPORT_INVALID);
    assert!(error.detail().contains("persistence-replay"));

    let tampered_bundle = TempBundle::new();
    let mut tampered_report = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
    materialize_check_reports(&tampered_bundle, &mut tampered_report);
    fs::write(
        tampered_bundle.root.join(
            tampered_report.checks[4]
                .report_path
                .as_ref()
                .expect("path"),
        ),
        b"tampered",
    )
    .expect("tamper check report");
    write_target_report(&tampered_bundle, &tampered_report);
    let error = validate_native_gate_target_bundle(&tampered_bundle.report_path())
        .expect_err("tampered check report");
    assert_eq!(error.code(), NATIVE_GATE_REPORT_INVALID);
    assert!(error.detail().contains("platform report hash mismatch"));
}

#[test]
fn bundle_validation_rejects_performance_report_without_v5_run() {
    let bundle = TempBundle::new();
    let mut report = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
    materialize_check_reports(&bundle, &mut report);
    let index = report
        .checks
        .iter()
        .position(|record| record.check == NativeGateCheckNameV1::Performance)
        .expect("performance record");
    let mut value = check_report_value(NativeGateCheckNameV1::Performance, &report);
    value["details"]
        .as_object_mut()
        .expect("performance details")
        .remove("run");
    let bytes = serde_json::to_vec(&value).expect("serialize legacy performance report");
    let relative_path = report.checks[index]
        .report_path
        .as_ref()
        .expect("performance report path");
    fs::write(bundle.root.join(relative_path), &bytes).expect("write legacy performance report");
    report.checks[index].report_sha256 = Some(sha256_hex(&bytes));
    write_target_report(&bundle, &report);

    let error = validate_native_gate_target_bundle(&bundle.report_path())
        .expect_err("missing V5 performance run");
    assert_eq!(error.code(), NATIVE_GATE_REPORT_INVALID);
    assert!(error.detail().contains("performance V5 run is missing"));
}

#[test]
fn bundle_validation_rejects_wrong_performance_run_mode() {
    let bundle = TempBundle::new();
    let mut report = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
    materialize_check_reports(&bundle, &mut report);
    let index = 5;
    let mut value = check_report_value(NativeGateCheckNameV1::Performance, &report);
    value["details"]["run"]["mode"] = serde_json::Value::String("gate".to_owned());
    let bytes = serde_json::to_vec(&value).expect("serialize wrong-mode performance report");
    let path = report.checks[index]
        .report_path
        .as_ref()
        .expect("performance report path");
    fs::write(bundle.root.join(path), &bytes).expect("write wrong-mode performance report");
    report.checks[index].report_sha256 = Some(sha256_hex(&bytes));

    let error = check_reports::validate_check_reports(&bundle.root, &report)
        .expect_err("wrong performance mode");
    assert!(error.detail().contains("clean Smoke/Report/REPORT_ONLY"));
}

#[test]
fn bundle_validation_binds_performance_build_target_and_toolchain() {
    let bundle = TempBundle::new();
    let mut report = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
    materialize_check_reports(&bundle, &mut report);
    let mut value = check_report_value(NativeGateCheckNameV1::Performance, &report);
    value["details"]["run"]["target_triple"] =
        serde_json::Value::String(LINUX_TARGET_TRIPLE.to_owned());
    let toolchain = value["details"]["run"]["toolchain"]
        .as_str()
        .expect("toolchain")
        .replace(WINDOWS_TARGET_TRIPLE, LINUX_TARGET_TRIPLE);
    value["details"]["run"]["toolchain"] = serde_json::Value::String(toolchain);
    replace_performance_check_report(&bundle, &mut report, &value);

    let error = check_reports::validate_check_reports(&bundle.root, &report)
        .expect_err("run target must bind to the native bundle");
    assert!(error.detail().contains("performance target triple"));
}

#[test]
fn bundle_validation_rejects_missing_performance_environment_evidence() {
    for field in ["target_fingerprint", "preflight"] {
        let bundle = TempBundle::new();
        let mut report = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
        materialize_check_reports(&bundle, &mut report);
        let mut value = check_report_value(NativeGateCheckNameV1::Performance, &report);
        value["details"]["run"][field] = serde_json::Value::Null;
        replace_performance_check_report(&bundle, &mut report, &value);

        let error = check_reports::validate_check_reports(&bundle.root, &report)
            .expect_err("environment evidence is mandatory");
        assert!(error.detail().contains("is missing"));
    }
}

#[test]
fn bundle_validation_rejects_performance_diagnostics() {
    let bundle = TempBundle::new();
    let mut report = controlled_fail_report(WINDOWS_TARGET_TRIPLE);
    materialize_check_reports(&bundle, &mut report);
    let mut value = check_report_value(NativeGateCheckNameV1::Performance, &report);
    value["details"]["run"]["diagnostics"] = serde_json::json!(["PERF_RUNTIME_TOOLCHAIN_MISMATCH"]);
    replace_performance_check_report(&bundle, &mut report, &value);

    let error = check_reports::validate_check_reports(&bundle.root, &report)
        .expect_err("native evidence cannot carry diagnostics");
    assert!(error.detail().contains("diagnostics must be empty"));
}
