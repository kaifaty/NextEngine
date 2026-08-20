pub(super) const PACKAGE_SMOKE_FIXTURE_SOURCE: &str = r#"
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
        "source/reference-alpha",
        "ACCEPTANCE.md",
        "LICENSE",
        "NOTICE",
        "REFERENCE_ALPHA_NOTICE",
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
    validate_environment(&package_root);

    let file_name = executable
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_else(|| fail("executable name is not UTF-8"));
    let (composition_root, interactive_host_object_count, presentation) =
        if file_name.starts_with("next_headless") {
        if arguments.iter().any(|argument| argument == "--interactive") {
            fail("headless smoke unexpectedly received --interactive");
        }
        if argument_value(&arguments, "--live-ticks") != "0" {
            fail("headless smoke is not the exact zero-tick live launch");
        }
        ("Headless", 0, "null".to_owned())
    } else if file_name.starts_with("next_game") {
        if !arguments.iter().any(|argument| argument == "--interactive")
            || argument_value(&arguments, "--maximum-frames") != "1"
        {
            fail("game smoke is not the bounded interactive launch");
        }
        (
            "Game",
            1,
            format!(
                "{{\"target\":\"Interactive\",\"snapshot_hash\":\"{}\",\"object_count\":1}}",
                "9".repeat(64)
            ),
        )
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
\"interactive_host_object_count\":{interactive_host_object_count},\
\"presentation\":{presentation}}}"
    );
}

fn validate_environment(package_root: &Path) {
    for name in [
        "HOME", "USERPROFILE", "LOCALAPPDATA", "APPDATA", "XDG_STATE_HOME", "PROGRAMDATA",
        "ALLUSERSPROFILE", "TMP", "TEMP", "TMPDIR",
    ] {
        require_isolated_directory(package_root, name);
    }
    for (name, _) in env::vars_os() {
        let Some(name) = name.to_str() else { fail("environment variable name is not UTF-8") };
        if name == "PATH" || name.starts_with("LD_") || name.starts_with("VK_") || name.starts_with("SDL_") {
            fail(&format!("forbidden inherited environment variable {name}"));
        }
    }
}
"#;

pub(super) const PACKAGE_TOOL_SMOKE_FIXTURE_SOURCE: &str = r#"
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn fail(message: &str) -> ! { eprintln!("{message}"); std::process::exit(2); }

fn main() {
    let package_root = fs::canonicalize(".").unwrap_or_else(|error| fail(&error.to_string()));
    let executable = env::current_exe().and_then(fs::canonicalize)
        .unwrap_or_else(|error| fail(&error.to_string()));
    let package_bin = fs::canonicalize(package_root.join("bin"))
        .unwrap_or_else(|error| fail(&error.to_string()));
    if executable.parent() != Some(package_bin.as_path()) { fail("tool was not copied"); }
    if !package_root.join("source/reference-alpha/project.authoring.json").is_file()
        || !package_root.join("source/reference-alpha/assets/humanoid-cc0.catalog.json").is_file()
    { fail("frozen source is missing"); }
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments != ["project", "run", "--project", "source/reference-alpha"] {
        fail("unexpected tools command");
    }
    for name in ["HOME", "USERPROFILE", "LOCALAPPDATA", "APPDATA", "XDG_STATE_HOME",
        "PROGRAMDATA", "ALLUSERSPROFILE", "TMP", "TEMP", "TMPDIR"] {
        let path = PathBuf::from(env::var_os(name).unwrap_or_else(|| fail("missing isolated env")));
        if !path.is_absolute() || !path.is_dir() || path.starts_with(&package_root) {
            fail("environment is not isolated");
        }
    }
    for (name, _) in env::vars_os() {
        let Some(name) = name.to_str() else { fail("environment variable name is not UTF-8") };
        if name == "PATH" || name.starts_with("LD_") || name.starts_with("VK_") || name.starts_with("SDL_") {
            fail("forbidden inherited environment");
        }
    }
    let hash_a = "a".repeat(64); let hash_b = "b".repeat(64); let hash_c = "c".repeat(64);
    let hash_d = "d".repeat(64); let hash_e = "e".repeat(64); let hash_f = "f".repeat(64);
    println!("{{\"schema_version\":1,\"status\":\"PASS\",\"command\":\"project.run\",\"details\":{{\"project\":{{\"project_id\":\"fixture\",\"project_revision\":1,\"authoring_sha256\":\"{hash_a}\",\"project_lock_sha256\":\"__PROJECT_LOCK__\",\"schema_registry_sha256\":\"__SCHEMA__\",\"content_manifest_sha256\":\"__CONTENT__\",\"world_partition_sha256\":\"__WORLD__\",\"mechanics_lock_sha256\":\"__MECHANICS__\"}},\"runtime\":{{\"status\":\"PASS\",\"composition_root\":\"Headless\",\"session_id\":\"{hash_b}\",\"close_receipt_hash\":\"{hash_c}\",\"final_save_generation_hash\":\"{hash_d}\",\"ticks\":1,\"events\":1,\"rpg_events\":1,\"authoritative_revision\":1,\"authoritative_state_root\":\"{hash_e}\",\"command_archive_root\":\"{hash_f}\",\"command_identity_index_root\":\"{hash_a}\",\"command_ledger_hash\":\"{hash_b}\",\"project_composition_lock_hash\":\"__PROJECT_LOCK__\"}},\"source\":\"authoring\"}}}}")
}
"#;

pub(super) const PACKAGE_SMOKE_TIMEOUT_FIXTURE_SOURCE: &str = r#"
use std::io::Write;
use std::time::Duration;

fn main() {
    std::io::stdout()
        .write_all(&vec![b'o'; 2 * 1024 * 1024])
        .expect("stdout");
    std::io::stderr()
        .write_all(&vec![b'e'; 128 * 1024])
        .expect("stderr");
    std::thread::sleep(Duration::from_secs(60));
}
"#;
