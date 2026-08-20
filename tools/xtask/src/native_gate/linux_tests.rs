use super::*;

fn hash(byte: char) -> String {
    std::iter::repeat_n(byte, 64).collect()
}

fn release_roots() -> NativeGateReleaseRootsV2 {
    let mut roots = NativeGateReleaseRootsV2 {
        project_composition_lock_hash: hash('1'),
        schema_registry_hash: hash('2'),
        content_manifest_hash: hash('3'),
        mechanics_lock_hash: hash('4'),
        world_partition_hash: hash('5'),
        luau_manifest_hash: hash('6'),
        wasm_manifest_hash: hash('7'),
        wit_v2_hash: hash('8'),
        wit_v3_hash: hash('9'),
        extension_compatibility_hash: hash('a'),
        play_state_root: hash('b'),
        play_ledger_hash: hash('c'),
        replay_state_root: hash('d'),
        replay_ledger_hash: hash('e'),
        platform_state_root: hash('b'),
        platform_ledger_hash: hash('c'),
        presentation_snapshot_hash: hash('f'),
        streaming_performance_hash: hash('1'),
        agent_performance_hash: hash('2'),
        audio_scene_pcm_digest: hash('3'),
        packaged_game_state_root: hash('4'),
        packaged_game_ledger_hash: hash('5'),
        packaged_headless_state_root: hash('6'),
        packaged_headless_ledger_hash: hash('7'),
        closure_hash: hash('0'),
        package_descriptor_hash: hash('0'),
    };
    roots.package_descriptor_hash = linux_package_descriptor_hash_v2(&roots);
    roots.closure_hash = linux_closure_hash_v2(&roots);
    roots
}

fn package(roots: &NativeGateReleaseRootsV2) -> NativeGatePackageSummaryV1 {
    NativeGatePackageSummaryV1 {
        status: NativeGateRunStatusV1::Pass,
        target_triple: LINUX_TARGET_TRIPLE.to_owned(),
        relative_path: "package".to_owned(),
        package_manifest_sha256: hash('8'),
        project_lock_sha256: roots.project_composition_lock_hash.clone(),
        schema_registry_sha256: roots.schema_registry_hash.clone(),
        content_manifest_sha256: roots.content_manifest_hash.clone(),
        mechanics_lock_sha256: roots.mechanics_lock_hash.clone(),
        world_partition_sha256: roots.world_partition_hash.clone(),
        game_binary_sha256: hash('9'),
        headless_binary_sha256: hash('a'),
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

fn report() -> NativeGateLinuxReportV2 {
    let roots = release_roots();
    NativeGateLinuxReportV2 {
        schema_version: NATIVE_GATE_SCHEMA_VERSION,
        status: NativeGateRunStatusV1::Pass,
        release_ready: true,
        git_commit_sha: std::iter::repeat_n('a', 40).collect(),
        cargo_lock_sha256: hash('b'),
        rustc_release: NATIVE_GATE_RUSTC_RELEASE.to_owned(),
        target_triple: LINUX_TARGET_TRIPLE.to_owned(),
        checks: NativeGateCheckNameV1::ORDERED
            .into_iter()
            .map(|check| NativeGateCheckRecordV1 {
                check,
                status: NativeGateCheckStatusV1::Pass,
                report_path: Some(check.report_path()),
                report_sha256: Some(hash('c')),
                diagnostic: None,
                elapsed_milliseconds: 1,
            })
            .collect(),
        release_target: Some(NativeGateClosureTargetSummaryV1 {
            target_triple: LINUX_TARGET_TRIPLE.to_owned(),
            package_descriptor_hash: roots.package_descriptor_hash.clone(),
            runtime_check: NativeGateTargetExecutionStatusV1::Pass,
            desktop_smoke: NativeGateTargetExecutionStatusV1::Pass,
        }),
        package: Some(package(&roots)),
        release_roots: Some(roots),
    }
}

#[test]
fn one_complete_linux_report_is_release_ready() {
    validate_native_gate_linux_report(&report()).expect("complete Linux report passes");
}

#[test]
fn old_schema_and_windows_target_fail_closed() {
    let mut old = report();
    old.schema_version = LEGACY_NATIVE_GATE_SCHEMA_VERSION;
    assert_eq!(
        validate_native_gate_linux_report(&old)
            .expect_err("schema 1 must not be relabeled")
            .code(),
        NATIVE_GATE_REPORT_INVALID
    );

    let mut windows = report();
    windows.target_triple = WINDOWS_TARGET_TRIPLE.to_owned();
    assert_eq!(
        validate_native_gate_linux_report(&windows)
            .expect_err("Windows is not a current release target")
            .code(),
        NATIVE_GATE_TARGET_SET_INVALID
    );
}

#[test]
fn incomplete_or_tampered_linux_report_fails_closed() {
    let mut not_ready = report();
    not_ready.release_ready = false;
    assert!(validate_native_gate_linux_report(&not_ready).is_err());

    let mut tampered = report();
    tampered.release_roots.as_mut().expect("roots").closure_hash = hash('f');
    assert_eq!(
        validate_native_gate_linux_report(&tampered)
            .expect_err("derived closure must bind all roots")
            .code(),
        NATIVE_GATE_ROOT_MISMATCH
    );
}

#[test]
fn schemas_are_not_cross_decoded_or_extended() {
    let current = serde_json::to_value(report()).expect("serialize current report");
    assert!(serde_json::from_value::<NativeGateTargetReportV1>(current.clone()).is_err());

    let legacy = serde_json::to_value(super::tests::report(LINUX_TARGET_TRIPLE))
        .expect("serialize legacy report");
    assert!(serde_json::from_value::<NativeGateLinuxReportV2>(legacy).is_err());

    let mut extended = current;
    extended
        .as_object_mut()
        .expect("report object")
        .insert("windows".to_owned(), serde_json::json!({}));
    assert!(serde_json::from_value::<NativeGateLinuxReportV2>(extended).is_err());
}
