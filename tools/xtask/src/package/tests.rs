use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
const PACKAGE_SMOKE_FIXTURE_SOURCE: &str = r#"
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(2);
}

fn argument_value(arguments: &[OsString], name: &str) -> String {
    let position = arguments
        .iter()
        .position(|argument| argument == name)
        .unwrap_or_else(|| fail(&format!("missing {name}")));
    arguments
        .get(position + 1)
        .and_then(|value| value.to_str())
        .unwrap_or_else(|| fail(&format!("invalid {name} value")))
        .to_owned()
}

fn require_isolated_directory(package_root: &Path, name: &str) {
    let value = env::var_os(name).unwrap_or_else(|| fail(&format!("missing {name}")));
    let path = PathBuf::from(value);
    if !path.is_absolute() || !path.is_dir() || path.starts_with(package_root) {
        fail(&format!("{name} is not an isolated existing directory"));
    }
}

fn main() {
    let package_root = fs::canonicalize(".").unwrap_or_else(|error| {
        fail(&format!("failed to resolve package cwd: {error}"));
    });
    let executable = env::current_exe()
        .and_then(fs::canonicalize)
        .unwrap_or_else(|error| fail(&format!("failed to resolve executable: {error}")));
    let package_bin = fs::canonicalize(package_root.join("bin"))
        .unwrap_or_else(|error| fail(&format!("failed to resolve package bin: {error}")));
    if executable.parent() != Some(package_bin.as_path()) {
        fail("smoke did not execute the copied package/bin binary");
    }
    for required in [
        "project",
        "LICENSE",
        "NOTICE",
        "THIRD_PARTY_NOTICES.md",
        "MIGRATION_PROVENANCE.md",
    ] {
        if !package_root.join(required).exists() {
            fail(&format!("package cwd is missing {required}"));
        }
    }

    let arguments: Vec<OsString> = env::args_os().skip(1).collect();
    if argument_value(&arguments, "--project") != "project" {
        fail("project argument must be package-relative");
    }
    let project_lock = argument_value(&arguments, "--lock");
    if project_lock.len() != 64
        || !project_lock
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        fail("project lock is not canonical lowercase SHA-256");
    }
    let state_root = PathBuf::from(argument_value(&arguments, "--state-root"));
    if !state_root.is_absolute() || state_root.starts_with(&package_root) || !state_root.is_dir() {
        fail("state root is not an isolated existing directory");
    }
    fs::write(state_root.join("smoke-state.marker"), b"disposable")
        .unwrap_or_else(|error| fail(&format!("failed to write smoke state: {error}")));
    for name in [
        "HOME",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "XDG_STATE_HOME",
        "TMP",
        "TEMP",
        "TMPDIR",
    ] {
        require_isolated_directory(&package_root, name);
    }

    let file_name = executable
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_else(|| fail("executable name is not UTF-8"));
    let composition_root = if file_name.starts_with("next_headless") {
        if arguments.iter().any(|argument| argument == "--interactive") {
            fail("headless smoke unexpectedly received --interactive");
        }
        "Headless"
    } else if file_name.starts_with("next_game") {
        if !arguments.iter().any(|argument| argument == "--interactive")
            || argument_value(&arguments, "--maximum-frames") != "1"
        {
            fail("game smoke is not the bounded interactive launch");
        }
        "Game"
    } else {
        fail("unexpected packaged binary name");
    };

    let state = "a".repeat(64);
    let ledger = "b".repeat(64);
    let archive = "c".repeat(64);
    let identity = "d".repeat(64);
    let receipt = "e".repeat(64);
    let session = "f".repeat(64);
    println!(
        "{{\"schema_version\":1,\"status\":\"PASS\",\"composition_root\":\"{composition_root}\",\
\"session_id\":\"{session}\",\"close_receipt_hash\":\"{receipt}\",\"close_result\":\"Saved\",\
\"final_save_generation_hash\":null,\"project_composition_lock_hash\":\"{project_lock}\",\
\"ticks\":1,\"events\":1,\"rpg_events\":1,\"authoritative_revision\":1,\
\"authoritative_state_root\":\"{state}\",\"command_archive_root\":\"{archive}\",\
\"command_identity_index_root\":\"{identity}\",\"command_ledger_hash\":\"{ledger}\",\
\"interactive_host_object_count\":0,\"presentation\":null}}"
    );
}
"#;

#[test]
fn manifest_encoding_is_canonical_and_round_trips() {
    let manifest = fixture_manifest();
    let bytes = canonical_json_bytes(&manifest).expect("canonical JSON");
    let decoded: PackageManifestV2 = serde_json::from_slice(&bytes).expect("manifest decodes");
    assert_eq!(decoded, manifest);
    assert!(bytes.starts_with(br#"{"binaries":"#));
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
    let fixture_binary = compile_package_smoke_fixture(temporary.path());
    let repository_root = fs::canonicalize(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("xtask belongs to the repository workspace"),
    )
    .expect("repository root");
    let output = temporary.path().join("package");
    let target_triple = native_shipping_target_triple().expect("native shipping host");

    let result =
        build_v1_package_with_binary_sources(&repository_root, &output, |_, requested_target| {
            assert_eq!(requested_target, target_triple);
            Ok(PackageBinarySources {
                game: fixture_binary.clone(),
                headless: fixture_binary.clone(),
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
fn publish_lock_and_destination_checks_never_accept_existing_objects() {
    let temporary = TestDirectory::new("publish-collision");
    let output = temporary.path().join("package");
    let first_lock = PublishLock::acquire(&output).expect("first publish lock");
    assert!(PublishLock::acquire(&output).is_err());
    drop(first_lock);
    assert!(PublishLock::acquire(&output).is_ok());

    fs::write(&output, b"existing").expect("destination");
    assert!(ensure_publish_destination_absent(&output).is_err());
}

#[cfg(unix)]
#[test]
fn destination_check_rejects_dangling_symlink() {
    use std::os::unix::fs::symlink;

    let temporary = TestDirectory::new("dangling-destination");
    let output = temporary.path().join("package");
    symlink("missing", &output).expect("dangling symlink");
    assert!(ensure_publish_destination_absent(&output).is_err());
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
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn compile_package_smoke_fixture(directory: &Path) -> PathBuf {
    let source = directory.join("package-smoke-fixture.rs");
    fs::write(&source, PACKAGE_SMOKE_FIXTURE_SOURCE).expect("fixture source");
    let executable_suffix = if cfg!(windows) { ".exe" } else { "" };
    let executable = directory.join(format!("package-smoke-fixture{executable_suffix}"));
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
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

fn fixture_manifest() -> PackageManifestV2 {
    let project_lock = "1".repeat(64);
    let state = "2".repeat(64);
    let ledger = "3".repeat(64);
    let game_hash = "4".repeat(64);
    let headless_hash = "5".repeat(64);
    PackageManifestV2 {
        binaries: PackageBinariesV2 {
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
        },
        file_inventory: vec![
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
        schema_version: PACKAGE_MANIFEST_SCHEMA_VERSION,
        target_neutral_roots: PackageTargetNeutralRootsV2 {
            content_manifest_sha256: "6".repeat(64),
            mechanics_lock_sha256: "7".repeat(64),
            project_composition_lock_sha256: project_lock,
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
