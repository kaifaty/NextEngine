use std::env;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

fn main() {
    let repository = repository_root();
    emit_tracked_inputs(&repository);
    emit_build_identity(&repository);

    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_owned());
    println!("cargo:rustc-env=NEXTENGINE_BUILD_PROFILE={profile}");

    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    let encoded_rustflags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    println!(
        "cargo:rustc-env=NEXTENGINE_EFFECTIVE_RUSTFLAGS_HEX={}",
        hex_encode(encoded_rustflags.as_bytes())
    );
    let mut profile_overrides = env::vars()
        .filter(|(name, _)| name.starts_with("CARGO_PROFILE_"))
        .collect::<Vec<_>>();
    profile_overrides.sort_by(|left, right| left.0.cmp(&right.0));
    for (name, _) in &profile_overrides {
        println!("cargo:rerun-if-env-changed={name}");
    }
    println!(
        "cargo:rustc-env=NEXTENGINE_PROFILE_OVERRIDES={}",
        encode_profile_overrides(&profile_overrides)
    );
    println!(
        "cargo:rustc-env=NEXTENGINE_BUILD_OPT_LEVEL={}",
        env::var("OPT_LEVEL").unwrap_or_else(|_| "unknown".to_owned())
    );
    let rustflags = encoded_rustflags.split('\u{1f}').collect::<Vec<_>>();
    let profile_generate = codegen_option(&rustflags, "profile-generate");
    let profile_use = codegen_option(&rustflags, "profile-use");
    let forbidden = rustflags.iter().any(|flag| {
        let flag = flag.to_ascii_lowercase();
        flag.contains("target-cpu=native")
            || flag.contains("llvm-bolt")
            || flag.contains("emit-relocs")
            || flag.contains("use-gnu-stack")
    });
    let mode = match (profile_generate, profile_use) {
        (Some(_), None) => "pgo-instrumented",
        (None, Some(_)) => "pgo-use",
        (None, None) => "plain",
        (Some(_), Some(_)) => "conflicting-pgo-flags",
    };
    println!("cargo:rustc-env=NEXTENGINE_CODEGEN_MODE={mode}");
    println!(
        "cargo:rustc-env=NEXTENGINE_CODEGEN_FORBIDDEN_FLAGS={}",
        if forbidden { "present" } else { "absent" }
    );
    if let Some(path) = profile_use {
        println!("cargo:rustc-env=NEXTENGINE_PGO_PROFILE_PATH={path}");
        let profile_path = resolve_from_root(&repository, Path::new(path));
        println!("cargo:rerun-if-changed={}", profile_path.display());
        let profile_hash = hash_file(&profile_path).unwrap_or_else(|error| {
            panic!(
                "failed to hash PGO profile {} during build: {error}",
                profile_path.display()
            )
        });
        println!("cargo:rustc-env=NEXTENGINE_PGO_PROFILE_SHA256={profile_hash}");
    } else {
        println!("cargo:rustc-env=NEXTENGINE_PGO_PROFILE_PATH=");
        println!("cargo:rustc-env=NEXTENGINE_PGO_PROFILE_SHA256=");
    }
}

fn repository_root() -> PathBuf {
    let manifest_directory = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default();
    let repository = manifest_directory
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_default();
    if repository.is_absolute() {
        repository
    } else {
        env::current_dir().unwrap_or_default().join(repository)
    }
}

fn emit_build_identity(repository: &Path) {
    let git_directory = command_output_in(
        repository,
        "git",
        &["rev-parse", "--path-format=absolute", "--git-dir"],
    )
    .map(PathBuf::from)
    .unwrap_or_else(|| repository.join(".git"));
    let head_path = git_path(repository, "HEAD").unwrap_or_else(|| git_directory.join("HEAD"));
    let index_path = git_path(repository, "index").unwrap_or_else(|| git_directory.join("index"));
    println!("cargo:rerun-if-changed={}", head_path.display());
    println!("cargo:rerun-if-changed={}", index_path.display());
    if let Ok(head) = std::fs::read_to_string(&head_path)
        && let Some(reference) = head.trim().strip_prefix("ref: ")
        && let Some(reference_path) = git_path(repository, reference)
    {
        println!("cargo:rerun-if-changed={}", reference_path.display());
    }

    let commit = command_output_in(repository, "git", &["rev-parse", "HEAD"])
        .unwrap_or_else(|| "UNKNOWN".to_owned());
    let worktree_clean = command_output_in(repository, "git", &["status", "--porcelain"])
        .is_some_and(|status| status.is_empty());
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let toolchain = command_output_in(repository, &rustc, &["-vV"]).unwrap_or_default();

    println!("cargo:rustc-env=NEXTENGINE_BUILD_COMMIT={commit}");
    println!(
        "cargo:rustc-env=NEXTENGINE_BUILD_WORKTREE_CLEAN={}",
        if worktree_clean { "true" } else { "false" }
    );
    println!(
        "cargo:rustc-env=NEXTENGINE_BUILD_TOOLCHAIN_HEX={}",
        hex_encode(toolchain.as_bytes())
    );
}

fn emit_tracked_inputs(repository: &Path) {
    let output = command_bytes_in(
        repository,
        "git",
        &["-c", "core.quotepath=false", "ls-files", "-z", "--cached"],
    )
    .unwrap_or_else(|| {
        panic!(
            "failed to enumerate tracked inputs in {}",
            repository.display()
        )
    });
    let mut relative_paths = output
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            String::from_utf8(path.to_vec())
                .unwrap_or_else(|_| panic!("tracked repository path is not UTF-8"))
        })
        .collect::<Vec<_>>();
    relative_paths.sort();
    relative_paths.dedup();
    for relative in relative_paths {
        if relative.contains('\r') || relative.contains('\n') {
            panic!("tracked repository path contains a line break: {relative:?}");
        }
        let relative_path = Path::new(&relative);
        if relative_path.is_absolute()
            || relative_path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::Prefix(_)
                        | std::path::Component::RootDir
                        | std::path::Component::ParentDir
                )
            })
        {
            panic!("git ls-files returned a non-relative path: {relative:?}");
        }
        println!(
            "cargo:rerun-if-changed={}",
            repository.join(relative_path).display()
        );
    }
}

fn git_path(repository: &Path, logical_path: &str) -> Option<PathBuf> {
    let output = command_output_in(
        repository,
        "git",
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            logical_path,
        ],
    )?;
    (!output.is_empty()).then(|| PathBuf::from(output))
}

fn command_output_in(directory: &Path, program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn command_bytes_in(directory: &Path, program: &str, arguments: &[&str]) -> Option<Vec<u8>> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout)
}

fn resolve_from_root(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}

fn hash_file(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.file_type().is_file() {
        return Err("PGO profile must be a non-symlink regular file".to_owned());
    }
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn encode_profile_overrides(overrides: &[(String, String)]) -> String {
    overrides
        .iter()
        .map(|(name, value)| {
            format!(
                "{}:{}",
                hex_encode(name.as_bytes()),
                hex_encode(value.as_bytes())
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn codegen_option<'a>(flags: &'a [&str], name: &str) -> Option<&'a str> {
    let compact = format!("-C{name}=");
    let value = format!("{name}=");
    flags.iter().enumerate().find_map(|(index, flag)| {
        flag.strip_prefix(&compact).or_else(|| {
            (index > 0 && flags[index - 1] == "-C")
                .then(|| flag.strip_prefix(&value))
                .flatten()
        })
    })
}
