use super::*;
use std::env;
use std::ffi::OsString;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

mod fixtures;
mod publish;

use fixtures::*;

#[test]
fn manifest_encoding_is_canonical_and_round_trips() {
    let manifest = fixture_manifest();
    let bytes = canonical_json_bytes(&manifest).expect("canonical JSON");
    let decoded: PackageManifestV5 = serde_json::from_slice(&bytes).expect("manifest decodes");
    assert_eq!(decoded, manifest);
    assert!(bytes.starts_with(br#"{"binaries":"#));
}

#[test]
fn retired_manifest_and_unknown_fields_are_rejected_without_migration() {
    let manifest = fixture_manifest();
    let mut value = serde_json::to_value(&manifest).expect("manifest value");
    let object = value.as_object_mut().expect("manifest object");
    object.remove("runtime_profile");
    object.insert("schema_version".to_owned(), serde_json::json!(2));
    let bytes = serde_json::to_vec(&value).expect("legacy manifest");
    assert!(serde_json::from_slice::<PackageManifestV5>(&bytes).is_err());

    let mut value = serde_json::to_value(&manifest).expect("manifest value");
    value["binaries"]
        .as_object_mut()
        .expect("binary object")
        .remove("tools");
    let bytes = serde_json::to_vec(&value).expect("missing tools manifest");
    assert!(serde_json::from_slice::<PackageManifestV5>(&bytes).is_err());

    let mut value = serde_json::to_value(&manifest).expect("manifest value");
    value
        .as_object_mut()
        .expect("manifest object")
        .insert("unexpected".to_owned(), serde_json::json!(true));
    let bytes = serde_json::to_vec(&value).expect("unknown-field manifest");
    assert!(serde_json::from_slice::<PackageManifestV5>(&bytes).is_err());

    let mut value = serde_json::to_value(&manifest).expect("manifest value");
    value["runtime_profile"]["abi"]["unexpected"] = serde_json::json!(true);
    let bytes = serde_json::to_vec(&value).expect("unknown nested field manifest");
    assert!(serde_json::from_slice::<PackageManifestV5>(&bytes).is_err());

    let mut value = serde_json::to_value(&manifest).expect("manifest value");
    value["runtime_profile"]["binaries"][0]
        .as_object_mut()
        .expect("runtime binary")
        .remove("direct_libraries");
    let bytes = serde_json::to_vec(&value).expect("missing nested field manifest");
    assert!(serde_json::from_slice::<PackageManifestV5>(&bytes).is_err());

    let mut wrong_version = manifest;
    wrong_version.schema_version = PACKAGE_MANIFEST_SCHEMA_VERSION - 1;
    assert!(
        validate_manifest_fields(&wrong_version)
            .expect_err("retired package format")
            .starts_with("UNSUPPORTED_PACKAGE_FORMAT:")
    );
}

#[test]
fn validator_rejects_path_escape_and_forbidden_operational_paths() {
    for path in [
        "../secret",
        "/absolute",
        "C:/absolute",
        r"bin\game.exe",
        "project//file",
        "state/session.json",
        "project/state-game/session.json",
        "project/cache/blob",
        "project/shadercache/blob",
        "project/debug.log",
        "project/.env",
        "credentials/token",
        "project/savegames/slot-1",
        "project/session-store/runtime.bin",
        "project/imported/asset",
        "project/generations/hash/.ssh/id_rsa",
        "project/generations/hash/identity/id_ed25519",
        "project/generations/hash/identity/private.key",
        "project/generations/hash/secrets/api-key.pem",
        "project/generations/hash/auth_token.json",
        "bin/.netrc",
        "bin/.npmrc",
        "project/generations/hash/.dockerconfigjson",
        "project/generations/hash/.kubeconfig",
        "project/generations/hash/known_hosts",
        "project/generations/hash/service-account.json",
    ] {
        assert!(
            validate_relative_package_path(path).is_err(),
            "{path} must be rejected"
        );
    }
    assert!(validate_relative_package_path("project/manifests/content.json").is_ok());
}

#[test]
fn project_store_layout_allows_only_current_generation() {
    let temporary = TestDirectory::new("project-layout");
    let project = temporary.path().join("project");
    let generations = project.join(next_assets::CONTENT_GENERATIONS_DIRECTORY);
    let generation = "a".repeat(64);
    fs::create_dir_all(generations.join(&generation)).expect("generation directory");
    fs::write(
        project.join(next_assets::CONTENT_CURRENT_FILE),
        format!("{generation}\n"),
    )
    .expect("CURRENT");
    assert!(validate_project_store_layout(temporary.path()).is_ok());

    fs::create_dir(generations.join("b".repeat(64))).expect("extra generation");
    assert!(validate_project_store_layout(temporary.path()).is_err());
    fs::remove_dir(generations.join("b".repeat(64))).expect("remove extra generation");
    fs::write(project.join("session.json"), b"not allowed").expect("extra project object");
    assert!(validate_project_store_layout(temporary.path()).is_err());
}

#[test]
fn package_root_requires_every_notice_and_rejects_smoke_or_state_objects() {
    let temporary = TestDirectory::new("top-level-layout");
    fs::create_dir(temporary.path().join("bin")).expect("bin");
    fs::create_dir(temporary.path().join("project")).expect("project");
    fs::create_dir(temporary.path().join("source")).expect("source");
    for file in [
        PACKAGE_MANIFEST_FILE,
        "LICENSE",
        "NOTICE",
        "THIRD_PARTY_NOTICES.md",
        "MIGRATION_PROVENANCE.md",
    ] {
        fs::write(temporary.path().join(file), b"fixture").expect("required file");
    }
    validate_package_root(temporary.path()).expect("complete package root");

    fs::remove_file(temporary.path().join("NOTICE")).expect("remove notice");
    assert!(validate_package_root(temporary.path()).is_err());
    fs::write(temporary.path().join("NOTICE"), b"fixture").expect("restore notice");

    fs::create_dir(temporary.path().join("state")).expect("state");
    assert!(validate_package_root(temporary.path()).is_err());
}

#[test]
fn inventory_enforces_depth_count_and_byte_limits() {
    let temporary = TestDirectory::new("inventory-limits");
    let sample = temporary.path().join("sample.bin");
    let file = fs::File::create(&sample).expect("sample file");
    file.set_len(inventory::MAX_PACKAGE_FILE_BYTES)
        .expect("sparse sample");
    let metadata = fs::metadata(&sample).expect("metadata");

    let mut depth_budget = inventory::TraversalBudget::default();
    assert!(
        depth_budget
            .observe_entry(inventory::MAX_PACKAGE_DEPTH + 1, &metadata)
            .is_err()
    );

    let small = temporary.path().join("small.bin");
    fs::write(&small, b"x").expect("small file");
    let small_metadata = fs::metadata(&small).expect("small metadata");
    let mut count_budget = inventory::TraversalBudget::default();
    for _ in 0..inventory::MAX_PACKAGE_ENTRIES {
        count_budget
            .observe_entry(1, &small_metadata)
            .expect("entry within count budget");
    }
    assert!(count_budget.observe_entry(1, &small_metadata).is_err());

    let mut total_budget = inventory::TraversalBudget::default();
    let files_within_total = inventory::MAX_PACKAGE_TOTAL_BYTES / inventory::MAX_PACKAGE_FILE_BYTES;
    for _ in 0..files_within_total {
        total_budget
            .observe_entry(1, &metadata)
            .expect("file within total byte budget");
    }
    assert!(total_budget.observe_entry(1, &metadata).is_err());

    file.set_len(inventory::MAX_PACKAGE_FILE_BYTES + 1)
        .expect("oversized sparse sample");
    let oversized = fs::metadata(&sample).expect("oversized metadata");
    assert!(
        inventory::TraversalBudget::default()
            .observe_entry(1, &oversized)
            .is_err()
    );
}

#[test]
fn inventory_hashes_files_with_standard_sha256() {
    let temporary = TestDirectory::new("streaming-hash");
    let path = temporary.path().join("sample.bin");
    fs::write(&path, b"abc").expect("sample");
    assert_eq!(
        hash_file(&path).expect("hash"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn inventory_is_complete_sorted_and_excludes_only_the_self_manifest() {
    let temporary = TestDirectory::new("inventory-shape");
    fs::create_dir(temporary.path().join("bin")).expect("bin");
    fs::write(temporary.path().join("bin/z.bin"), b"z").expect("z");
    fs::write(temporary.path().join("bin/a.bin"), b"a").expect("a");
    fs::write(temporary.path().join(PACKAGE_MANIFEST_FILE), b"self").expect("manifest");

    let inventory = collect_inventory(temporary.path()).expect("inventory");
    assert_eq!(
        inventory
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>(),
        ["bin/a.bin", "bin/z.bin"]
    );
}

#[test]
fn release_binaries_are_selected_from_the_explicit_native_target_directory() {
    assert_eq!(
        release_binary_directory(Path::new("target-root"), "x86_64-pc-windows-msvc"),
        Path::new("target-root")
            .join("x86_64-pc-windows-msvc")
            .join("release")
    );
    assert_eq!(
        release_binary_directory(Path::new("target-root"), "x86_64-unknown-linux-gnu"),
        Path::new("target-root")
            .join("x86_64-unknown-linux-gnu")
            .join("release")
    );
}

#[test]
#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn package_pipeline_copies_and_smokes_packaged_binaries_without_nested_cargo() {
    let temporary = TestDirectory::new("pipeline");
    let repository_root = fs::canonicalize(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("xtask belongs to the repository workspace"),
    )
    .expect("repository root");
    let cooked = next_project::cook_project_v7(
        next_reference_game::project_source_v7().expect("reference source"),
    )
    .expect("reference project cooks");
    let expected_roots = PackageTargetNeutralRootsV3 {
        content_manifest_sha256: cooked.content_manifest.content_manifest_sha256.to_hex(),
        mechanics_lock_sha256: cooked
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256
            .to_hex(),
        project_lock_sha256: cooked.project_lock.project_lock_sha256.to_hex(),
        schema_registry_sha256: cooked
            .schema_registry
            .schema_registry_manifest_sha256
            .to_hex(),
        world_partition_sha256: cooked
            .world_partition
            .world_partition_manifest_sha256
            .to_hex(),
    };
    let fixture_binary = compile_package_smoke_fixture(temporary.path());
    let validation = next_cli::execute([
        OsString::from("project"),
        OsString::from("validate"),
        OsString::from("--project"),
        repository_root
            .join("projects/reference-alpha")
            .into_os_string(),
    ]);
    let next_cli::CreatorCliReportV1::Project(next_cli::CreatorCommandReportV1::Pass(validation)) =
        validation
    else {
        panic!("reference project validation must pass");
    };
    let tool_fixture = compile_package_tool_fixture(temporary.path(), &validation.details);
    let output = temporary.path().join("package");
    let target_triple = native_shipping_target_triple().expect("native shipping host");

    let result =
        build_v1_package_with_binary_sources(&repository_root, &output, |_, requested_target| {
            assert_eq!(requested_target, target_triple);
            Ok(PackageBinarySources {
                game: fixture_binary.clone(),
                headless: fixture_binary.clone(),
                tools: tool_fixture.clone(),
            })
        })
        .expect("package pipeline");

    assert_eq!(result.output, output);
    assert_eq!(
        validate_v1_package(&output).expect("published package"),
        result.manifest
    );
    assert_eq!(
        result.package_manifest_sha256,
        hash_file(&output.join(PACKAGE_MANIFEST_FILE)).expect("manifest hash")
    );
    let executable_suffix = if target_triple == "x86_64-pc-windows-msvc" {
        ".exe"
    } else {
        ""
    };
    let source_hash = hash_file(&fixture_binary).expect("fixture hash");
    for (run, binary_name) in [
        (
            &result.manifest.binaries.game,
            format!("next_game{executable_suffix}"),
        ),
        (
            &result.manifest.binaries.headless,
            format!("next_headless{executable_suffix}"),
        ),
    ] {
        let copied = output.join("bin").join(binary_name);
        assert!(copied.is_file());
        assert_ne!(
            fs::canonicalize(&copied).expect("copied binary"),
            fs::canonicalize(&fixture_binary).expect("fixture binary")
        );
        assert_eq!(run.binary_sha256, source_hash);
        assert_eq!(hash_file(&copied).expect("copied hash"), source_hash);
    }
    let copied_tool = output.join(&result.manifest.binaries.tools.binary_path);
    assert!(copied_tool.is_file());
    assert_eq!(
        result.manifest.binaries.tools.binary_sha256,
        hash_file(&tool_fixture).expect("tool fixture hash")
    );
    assert_eq!(result.manifest.binaries.tools.launch_status, "PASS");
    assert_eq!(
        result.manifest.binaries.tools.source_project_path,
        "source/reference-alpha"
    );
    assert_eq!(result.manifest.target_neutral_roots, expected_roots);
    assert_eq!(
        result.manifest.binaries.game.authoritative_state_root,
        result.manifest.binaries.headless.authoritative_state_root
    );
    assert_eq!(
        result.manifest.binaries.game.command_ledger_hash,
        result.manifest.binaries.headless.command_ledger_hash
    );
    for notice in required_notice_paths() {
        assert!(
            result
                .manifest
                .file_inventory
                .iter()
                .any(|entry| entry.path == notice),
            "{notice} must be inventoried"
        );
        assert_eq!(
            fs::read(output.join(&notice)).expect("packaged notice"),
            fs::read(repository_root.join(&notice)).expect("source notice")
        );
    }
    for (source_path, package_path) in REFERENCE_PROJECT_DOCUMENT_PATHS {
        assert!(
            result
                .manifest
                .file_inventory
                .iter()
                .any(|entry| entry.path == package_path),
            "{package_path} must be inventoried"
        );
        assert_eq!(
            fs::read(output.join(package_path)).expect("packaged reference project document"),
            fs::read(repository_root.join(source_path)).expect("source reference project document")
        );
    }
    for relative in source::REFERENCE_SOURCE_FILES {
        let package_path = format!("source/reference-alpha/{relative}");
        assert!(
            result
                .manifest
                .file_inventory
                .iter()
                .any(|entry| entry.path == package_path),
            "{package_path} must be inventoried"
        );
        assert_eq!(
            fs::read(output.join(&package_path)).expect("frozen source file"),
            fs::read(
                repository_root
                    .join("projects/reference-alpha")
                    .join(relative)
            )
            .expect("repository source file")
        );
    }
    let smoke_root = temporary
        .path()
        .join(format!(".package.smoke-{}", std::process::id()));
    let staging_root = temporary
        .path()
        .join(format!(".package.staging-{}", std::process::id()));
    assert!(
        !smoke_root.exists(),
        "disposable smoke state must be removed"
    );
    assert!(!staging_root.exists(), "staging must be atomically renamed");

    let mut tampered_profile = result.manifest.runtime_profile.clone();
    tampered_profile.binaries[0]
        .direct_libraries
        .push("vendor-renderer.dll".to_owned());
    assert!(
        runtime::validate_runtime_profile(
            &output,
            target_triple,
            &tampered_profile,
            &[
                result.manifest.binaries.game.binary_path.as_str(),
                result.manifest.binaries.headless.binary_path.as_str(),
                result.manifest.binaries.tools.binary_path.as_str(),
            ],
        )
        .is_err()
    );
    let mut mismatched_tools = result.manifest.binaries.tools.clone();
    mismatched_tools.authoring_sha256 = "f".repeat(64);
    assert!(
        source::validate_reference_project_source(
            &output,
            &result.manifest.target_neutral_roots,
            &mismatched_tools,
        )
        .expect_err("mismatched tool receipt must fail")
        .contains("tools validation receipt does not match")
    );
    fs::write(
        output.join("source/reference-alpha/unexpected.txt"),
        b"unexpected",
    )
    .expect("tampered source");
    assert!(
        source::validate_reference_project_source(
            &output,
            &result.manifest.target_neutral_roots,
            &result.manifest.binaries.tools,
        )
        .expect_err("extra source file must fail")
        .contains("source layout is not exact")
    );
}

#[test]
#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn package_pipeline_audits_binary_abi_before_smoke_launch() {
    let temporary = TestDirectory::new("audit-before-smoke");
    let malformed_binary = temporary.path().join("malformed-binary");
    fs::write(&malformed_binary, b"not a native executable").expect("malformed binary");
    let repository_root = fs::canonicalize(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("xtask belongs to the repository workspace"),
    )
    .expect("repository root");
    let output = temporary.path().join("package");
    let error = build_v1_package_with_binary_sources(&repository_root, &output, |_, _| {
        Ok(PackageBinarySources {
            game: malformed_binary.clone(),
            headless: malformed_binary.clone(),
            tools: malformed_binary.clone(),
        })
    })
    .expect_err("malformed binary must fail before smoke");
    assert!(
        error.starts_with("NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED:"),
        "{error}"
    );
    assert!(!output.exists(), "invalid package must not publish");
    assert!(
        !temporary
            .path()
            .join(format!(".package.smoke-{}", std::process::id()))
            .exists(),
        "runtime audit must fail before smoke root creation"
    );
}

#[test]
fn smoke_environment_is_explicit_and_drops_loader_overrides() {
    let temporary = TestDirectory::new("smoke-environment");
    let home = temporary.path().join("home");
    let local_app_data = temporary.path().join("local");
    let roaming_app_data = temporary.path().join("roaming");
    let xdg_state_home = temporary.path().join("state");
    let program_data = temporary.path().join("program-data");
    let temp = temporary.path().join("temp");
    let mut command = Command::new("fixture");
    configure_smoke_environment(
        &mut command,
        &home,
        &local_app_data,
        &roaming_app_data,
        &xdg_state_home,
        &program_data,
        &temp,
    );

    let environment = command
        .get_envs()
        .map(|(name, value)| {
            (
                name.to_string_lossy().into_owned(),
                value.map(|value| value.to_os_string()),
            )
        })
        .collect::<Vec<_>>();
    for required in [
        "HOME",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "XDG_STATE_HOME",
        "PROGRAMDATA",
        "ALLUSERSPROFILE",
        "TMP",
        "TEMP",
        "TMPDIR",
    ] {
        assert!(
            environment
                .iter()
                .any(|(name, value)| name == required && value.is_some()),
            "{required}"
        );
    }
    assert!(environment.iter().all(|(name, _)| {
        name != "PATH"
            && !name.starts_with("LD_")
            && !name.starts_with("VK_")
            && !name.starts_with("SDL_")
    }));
}

#[test]
fn smoke_failure_routes_known_runtime_prerequisite_diagnostics() {
    assert_eq!(
        runtime_prerequisite_failure_code(
            Some(2),
            br#"{"code":"PLATFORM_GRAPHICS_LOADER_UNAVAILABLE"}"#,
            b"",
        ),
        Some("NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING")
    );
    assert_eq!(
        runtime_prerequisite_failure_code(
            Some(2),
            b"",
            b"PLATFORM_GRAPHICS_ICD_UNAVAILABLE: no driver",
        ),
        Some("NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING")
    );
    assert_eq!(
        runtime_prerequisite_failure_code(
            Some(2),
            b"",
            b"PLATFORM_GRAPHICS_LOADER_VERSION_UNSUPPORTED: 1.2",
        ),
        Some("NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED")
    );
    assert_eq!(
        runtime_prerequisite_failure_code(Some(2), b"", b"GPU_UNSUPPORTED: feature missing"),
        None
    );
    assert_eq!(
        runtime_prerequisite_failure_code(
            Some(LINUX_DYNAMIC_LOADER_FAILURE_EXIT_CODE),
            b"",
            b"next_game: error while loading shared libraries: libfoo.so.1: cannot open shared object file",
        ),
        Some("NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING")
    );
    assert_eq!(
        runtime_prerequisite_failure_code(
            Some(1),
            b"",
            b"error while loading shared libraries: libfoo.so.1",
        ),
        None,
        "glibc text without the native loader exit code is not enough"
    );
    assert_eq!(
        runtime_prerequisite_failure_code(Some(WINDOWS_STATUS_DLL_NOT_FOUND), b"", b""),
        Some("NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING")
    );
    for status in [
        WINDOWS_STATUS_INVALID_IMAGE_FORMAT,
        WINDOWS_STATUS_INVALID_IMAGE_LE_FORMAT,
        WINDOWS_STATUS_INVALID_IMAGE_NOT_MZ,
        WINDOWS_STATUS_INVALID_IMAGE_WIN_16,
        WINDOWS_STATUS_ORDINAL_NOT_FOUND,
        WINDOWS_STATUS_ENTRYPOINT_NOT_FOUND,
    ] {
        assert_eq!(
            runtime_prerequisite_failure_code(Some(status), b"", b""),
            Some("NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED"),
            "{status:#010x}"
        );
    }
}

#[test]
fn smoke_requires_interactive_game_presentation_and_nonzero_host_objects() {
    let mut report = fixture_smoke_report("Game");
    report.interactive_host_object_count = 0;
    let error = super::smoke::validate_presentation_contract(&report, "Game")
        .expect_err("zero rendered objects must fail package smoke");
    assert_eq!(
        error,
        "NATIVE_GATE_PACKAGE_INVALID: packaged Game smoke reported zero interactive host objects"
    );

    report.interactive_host_object_count = 1;
    report.presentation = None;
    let error = super::smoke::validate_presentation_contract(&report, "Game")
        .expect_err("missing presentation must fail package smoke");
    assert_eq!(
        error,
        "NATIVE_GATE_PACKAGE_INVALID: packaged Game smoke omitted its presentation snapshot"
    );

    report.presentation = Some(next_application::PresentationReportV1 {
        target: "Interactive".to_owned(),
        snapshot_hash: "c".repeat(64),
        object_count: 1,
    });
    super::smoke::validate_presentation_contract(&report, "Game")
        .expect("interactive Game report is truthful");

    report.interactive_host_object_count = 2;
    let error = super::smoke::validate_presentation_contract(&report, "Game")
        .expect_err("rendered and published object counts must agree");
    assert_eq!(
        error,
        "NATIVE_GATE_PACKAGE_INVALID: packaged Game smoke rendered 2 objects but its presentation snapshot contains 1"
    );
}

#[test]
fn smoke_requires_headless_to_remain_presentation_free() {
    let mut report = fixture_smoke_report("Headless");
    super::smoke::validate_presentation_contract(&report, "Headless")
        .expect("headless report is presentation-free");

    report.interactive_host_object_count = 1;
    let error = super::smoke::validate_presentation_contract(&report, "Headless")
        .expect_err("headless host objects must fail package smoke");
    assert_eq!(
        error,
        "NATIVE_GATE_PACKAGE_INVALID: packaged Headless smoke reported interactive host objects"
    );

    report.interactive_host_object_count = 0;
    report.presentation = Some(next_application::PresentationReportV1 {
        target: "Interactive".to_owned(),
        snapshot_hash: "c".repeat(64),
        object_count: 1,
    });
    let error = super::smoke::validate_presentation_contract(&report, "Headless")
        .expect_err("headless presentation must fail package smoke");
    assert_eq!(
        error,
        "NATIVE_GATE_PACKAGE_INVALID: packaged Headless smoke unexpectedly reported a presentation snapshot"
    );
}

#[test]
#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn smoke_timeout_kills_child_and_bounds_both_output_streams() {
    let temporary = TestDirectory::new("smoke-timeout");
    let fixture = compile_rust_fixture(
        temporary.path(),
        "package-smoke-timeout-fixture",
        PACKAGE_SMOKE_TIMEOUT_FIXTURE_SOURCE,
    );
    let error = run_packaged_binary_with_timeout(
        &fixture,
        &[],
        &temporary.path().join("smoke"),
        temporary.path(),
        "Game",
        &"a".repeat(64),
        Duration::from_millis(250),
    )
    .expect_err("fixture must time out");
    assert!(error.starts_with("NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT:"));
    assert!(error.contains("[stdout truncated]"));
    assert!(error.contains("[stderr truncated]"));
}

fn fixture_smoke_report(composition_root: &str) -> next_application::RunReportV1 {
    next_application::RunReportV1 {
        schema_version: 1,
        status: "PASS".to_owned(),
        composition_root: composition_root.to_owned(),
        session_id: "1".repeat(64),
        close_receipt_hash: "2".repeat(64),
        close_result: "Saved".to_owned(),
        final_save_generation_hash: None,
        project_composition_lock_hash: "3".repeat(64),
        ticks: 1,
        events: 1,
        rpg_events: 1,
        authoritative_revision: 1,
        authoritative_state_root: "4".repeat(64),
        command_archive_root: "5".repeat(64),
        command_identity_index_root: "6".repeat(64),
        command_ledger_hash: "7".repeat(64),
        interactive_host_object_count: 0,
        presentation: None,
    }
}

#[test]
fn smoke_inventory_requires_byte_identity() {
    let before = vec![PackageFileV2 {
        path: "bin/next_game.exe".to_owned(),
        sha256: "a".repeat(64),
        size_bytes: 10,
    }];
    let mut after = before.clone();
    assert!(ensure_inventory_unchanged(&before, &after).is_ok());
    after[0].sha256 = "b".repeat(64);
    assert!(ensure_inventory_unchanged(&before, &after).is_err());
}

#[test]
fn manifest_validator_allows_target_local_binary_hashes() {
    let mut manifest = fixture_manifest();
    manifest.binaries.game.binary_sha256 = "9".repeat(64);
    manifest.binaries.headless.binary_sha256 = "a".repeat(64);
    assert!(validate_manifest_fields(&manifest).is_ok());
}

#[test]
fn manifest_validator_requires_game_headless_authority_parity() {
    let mut manifest = fixture_manifest();
    manifest.binaries.game.authoritative_state_root = "b".repeat(64);
    assert!(validate_manifest_fields(&manifest).is_err());

    let mut manifest = fixture_manifest();
    manifest.binaries.game.command_ledger_hash = "b".repeat(64);
    assert!(validate_manifest_fields(&manifest).is_err());

    let mut manifest = fixture_manifest();
    manifest.required_notices.pop();
    assert!(validate_manifest_fields(&manifest).is_err());

    let mut manifest = fixture_manifest();
    manifest.binaries.tools.launch_status = "FAIL".to_owned();
    assert!(validate_manifest_fields(&manifest).is_err());

    let mut manifest = fixture_manifest();
    manifest.binaries.tools.publication_file_count = 0;
    assert!(validate_manifest_fields(&manifest).is_err());
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn compile_package_smoke_fixture(directory: &Path) -> PathBuf {
    compile_rust_fixture(
        directory,
        "package-smoke-fixture",
        PACKAGE_SMOKE_FIXTURE_SOURCE,
    )
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn compile_package_tool_fixture(
    directory: &Path,
    details: &next_cli::CreatorProjectDetailsV1,
) -> PathBuf {
    let source = package_tool_smoke_fixture_source(details);
    compile_rust_fixture(directory, "package-tool-smoke-fixture", &source)
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn compile_rust_fixture(directory: &Path, name: &str, source_text: &str) -> PathBuf {
    let source = directory.join(format!("{name}.rs"));
    fs::write(&source, source_text).expect("fixture source");
    let executable_suffix = if cfg!(windows) { ".exe" } else { "" };
    let executable = directory.join(format!("{name}{executable_suffix}"));
    let rustc = env::var_os("RUSTC")
        .map(PathBuf::from)
        .or_else(|| option_env!("RUSTC").map(PathBuf::from))
        .or_else(|| {
            option_env!("CARGO").map(|cargo| {
                Path::new(cargo).with_file_name(if cfg!(windows) { "rustc.exe" } else { "rustc" })
            })
        })
        .unwrap_or_else(|| PathBuf::from("rustc"));
    let output = Command::new(rustc)
        .arg("--edition=2024")
        .arg("-C")
        .arg("debuginfo=0")
        .arg(&source)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("launch rustc for smoke fixture");
    assert!(
        output.status.success(),
        "fixture compilation failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

fn fixture_manifest() -> PackageManifestV5 {
    let project_lock = "1".repeat(64);
    let state = "2".repeat(64);
    let ledger = "3".repeat(64);
    let game_hash = "4".repeat(64);
    let headless_hash = "5".repeat(64);
    let tool_hash = "a".repeat(64);
    PackageManifestV5 {
        binaries: PackageBinariesV3 {
            game: PackagedRunV2 {
                authoritative_state_root: state.clone(),
                binary_path: "bin/next_game.exe".to_owned(),
                binary_sha256: game_hash.clone(),
                command_ledger_hash: ledger.clone(),
                composition_root: "Game".to_owned(),
                launch_status: "PASS".to_owned(),
                project_composition_lock_hash: project_lock.clone(),
            },
            headless: PackagedRunV2 {
                authoritative_state_root: state,
                binary_path: "bin/next_headless.exe".to_owned(),
                binary_sha256: headless_hash.clone(),
                command_ledger_hash: ledger,
                composition_root: "Headless".to_owned(),
                launch_status: "PASS".to_owned(),
                project_composition_lock_hash: project_lock.clone(),
            },
            tools: PackagedToolValidationV1 {
                authoring_sha256: "b".repeat(64),
                binary_path: "bin/next.exe".to_owned(),
                binary_sha256: tool_hash.clone(),
                command: "project.validate".to_owned(),
                content_entry_count: 2,
                launch_status: "PASS".to_owned(),
                neutral_record_count: 3,
                project_id: "reference-alpha".to_owned(),
                project_composition_lock_hash: project_lock.clone(),
                project_revision: 1,
                publication_file_count: 4,
                publication_state: "validated-not-written".to_owned(),
                render_asset_count: 5,
                root_asset_count: 1,
                source_project_path: "source/reference-alpha".to_owned(),
                world_chunk_count: 1,
            },
        },
        file_inventory: vec![
            PackageFileV2 {
                path: "bin/next.exe".to_owned(),
                sha256: tool_hash,
                size_bytes: 12,
            },
            PackageFileV2 {
                path: "bin/next_game.exe".to_owned(),
                sha256: game_hash,
                size_bytes: 10,
            },
            PackageFileV2 {
                path: "bin/next_headless.exe".to_owned(),
                sha256: headless_hash,
                size_bytes: 11,
            },
        ],
        required_notices: required_notice_paths(),
        runtime_profile: PackageRuntimeProfileV3 {
            abi: PackageRuntimeAbiV3::WindowsMsvcX64 {
                crt: PackageWindowsCrtV3::DynamicSystem,
            },
            binaries: vec![
                PackageBinaryRuntimeV3 {
                    binary_path: "bin/next.exe".to_owned(),
                    direct_libraries: vec!["kernel32.dll".to_owned()],
                    maximum_required_glibc: None,
                },
                PackageBinaryRuntimeV3 {
                    binary_path: "bin/next_game.exe".to_owned(),
                    direct_libraries: vec!["kernel32.dll".to_owned()],
                    maximum_required_glibc: None,
                },
                PackageBinaryRuntimeV3 {
                    binary_path: "bin/next_headless.exe".to_owned(),
                    direct_libraries: vec!["kernel32.dll".to_owned()],
                    maximum_required_glibc: None,
                },
            ],
            external_prerequisites: Vec::new(),
        },
        schema_version: PACKAGE_MANIFEST_SCHEMA_VERSION,
        target_neutral_roots: PackageTargetNeutralRootsV3 {
            content_manifest_sha256: "6".repeat(64),
            mechanics_lock_sha256: "7".repeat(64),
            project_lock_sha256: project_lock,
            schema_registry_sha256: "8".repeat(64),
            world_partition_sha256: "9".repeat(64),
        },
        target_triple: "x86_64-pc-windows-msvc".to_owned(),
    }
}

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "nextengine-package-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("unique test directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.path.is_dir() {
            fs::remove_dir_all(&self.path).expect("remove test directory");
        }
    }
}
