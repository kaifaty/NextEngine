#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const EXPECTED_DOCUMENTS: usize = 32;
const EXPECTED_SPECS: usize = 16;
const EXPECTED_VERTICAL_GATES: usize = 15;
const EXPECTED_REQUIREMENTS: usize = 78;
const EXPECTED_FAILURES: usize = 24;
const EXPECTED_TECHNOLOGIES: usize = 23;
const EXPECTED_PROPOSED_TECHNOLOGIES: usize = 16;

fn main() {
    if let Err(error) = run() {
        eprintln!("xtask: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let root = env::current_dir().map_err(|error| error.to_string())?;
    let command = env::args()
        .nth(1)
        .ok_or_else(|| "expected docs-check, boundary-scan, or host-check".to_owned())?;
    match command.as_str() {
        "docs-check" => docs_check(&root),
        "boundary-scan" => boundary_scan(&root),
        "host-check" => host_check(&root),
        _ => Err(format!("unknown command: {command}")),
    }
}

fn docs_check(root: &Path) -> Result<(), String> {
    let docs_root = root.join("docs/architecture");
    let mut files = Vec::new();
    collect_files(&docs_root, Some("md"), &mut files)?;
    files.sort();
    require_count("Markdown documents", files.len(), EXPECTED_DOCUMENTS)?;

    let spec_count = files
        .iter()
        .filter(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| {
                    name.len() > 3
                        && name.as_bytes()[0].is_ascii_digit()
                        && name.as_bytes()[1].is_ascii_digit()
                        && name.as_bytes()[2] == b'-'
                })
        })
        .count();
    require_count("subsystem SPEC files", spec_count, EXPECTED_SPECS)?;

    let mut document_ids = BTreeSet::new();
    for file in &files {
        let body = read(file)?;
        for marker in ["TODO", "TBD", "FIXME"] {
            if body
                .split(|character: char| !character.is_ascii_alphanumeric())
                .any(|word| word == marker)
            {
                return Err(format!("unresolved marker {marker} in {}", file.display()));
            }
        }
        let id = body
            .lines()
            .find_map(|line| line.strip_prefix("| ID | "))
            .and_then(|value| value.strip_suffix(" |"))
            .ok_or_else(|| format!("missing document ID in {}", file.display()))?;
        if !document_ids.insert(id.to_owned()) {
            return Err(format!("duplicate document ID: {id}"));
        }
        validate_relative_links(file, &body)?;
    }

    let readme = read(&docs_root.join("README.md"))?;
    for file in files
        .iter()
        .filter(|file| file.file_name() != Some(OsStr::new("README.md")))
    {
        let relative = file
            .strip_prefix(&docs_root)
            .map_err(|error| error.to_string())?
            .to_string_lossy();
        if !readme.contains(&format!("]({relative})")) {
            return Err(format!("document missing from README index: {relative}"));
        }
    }

    let traceability = read(&docs_root.join("traceability.md"))?;
    require_count(
        "requirements",
        table_ids(&traceability, "REQ-").len(),
        EXPECTED_REQUIREMENTS,
    )?;
    require_count(
        "failure paths",
        table_ids(&traceability, "FAIL-").len(),
        EXPECTED_FAILURES,
    )?;
    require_sequential(&table_ids(&traceability, "REQ-"), "REQ-", 3)?;
    require_sequential(&table_ids(&traceability, "FAIL-"), "FAIL-", 3)?;

    let vertical = read(&docs_root.join("12-vertical-slice-conformance.md"))?;
    let vertical_ids = table_ids(&vertical, "VS-");
    require_count(
        "vertical gates",
        vertical_ids.len(),
        EXPECTED_VERTICAL_GATES,
    )?;
    require_sequential(&vertical_ids, "VS-", 2)?;

    let evidence = read(&docs_root.join("evidence-register.md"))?;
    let technology_rows: Vec<_> = evidence
        .lines()
        .filter(|line| {
            ["Accepted", "Proposed", "Rejected", "Superseded"]
                .iter()
                .any(|status| line.contains(&format!("| {status} |")))
                && line.split('|').count() == 11
        })
        .collect();
    require_count(
        "technology rows",
        technology_rows.len(),
        EXPECTED_TECHNOLOGIES,
    )?;
    require_count(
        "Proposed technology rows",
        technology_rows
            .iter()
            .filter(|line| line.contains("| Proposed |"))
            .count(),
        EXPECTED_PROPOSED_TECHNOLOGIES,
    )?;

    println!("PASS docs-check: packet 1.4 structure and links verified");
    Ok(())
}

fn boundary_scan(root: &Path) -> Result<(), String> {
    let contracts_root = root.join("crates/contracts");
    let mut contract_files = Vec::new();
    collect_files(&contracts_root, Some("rs"), &mut contract_files)?;
    let forbidden = [
        "ash::",
        "bevy",
        "daedalus",
        "gothic",
        "jolt",
        "physx",
        "vulkan",
        "windows_sys",
        "x11",
    ];
    for file in contract_files {
        let lower = read(&file)?.to_ascii_lowercase();
        for needle in forbidden {
            if lower.contains(needle) {
                return Err(format!(
                    "forbidden public-contract token {needle} in {}",
                    file.display()
                ));
            }
        }
    }

    let mut cargo_files = Vec::new();
    collect_named_files(root, "Cargo.toml", &mut cargo_files)?;
    for file in cargo_files {
        let body = read(&file)?;
        if body.contains("incubator/gothic-importer") || body.contains("gothic-importer") {
            return Err(format!("importer Cargo dependency in {}", file.display()));
        }
    }

    let tracked_importer = run_output(
        root,
        "git",
        &["ls-files", "--", "incubator/gothic-importer"],
    )?;
    if !tracked_importer.stdout.is_empty() {
        return Err("parent repository tracks incubator/gothic-importer".to_owned());
    }
    if root.join(".github/workflows").exists() {
        return Err("CI workflows are outside local-bootstrap scope".to_owned());
    }

    println!("PASS boundary-scan: public contracts and importer boundary verified");
    Ok(())
}

fn host_check(root: &Path) -> Result<(), String> {
    let version = run_output(root, "rustc", &["-vV"])?;
    let details = String::from_utf8(version.stdout).map_err(|error| error.to_string())?;
    if !details.lines().any(|line| line == "release: 1.93.0") {
        return Err("rustc release must be exactly 1.93.0".to_owned());
    }
    let host = details
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| "rustc did not report host".to_owned())?;
    let supported = [
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "x86_64-unknown-linux-gnu",
    ];
    if !supported.contains(&host) {
        return Err(format!("unsupported bootstrap host: {host}"));
    }

    run_checked(root, "cargo", &["fmt", "--all", "--", "--check"])?;
    run_checked(
        root,
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_checked(root, "cargo", &["test", "--workspace"])?;
    docs_check(root)?;
    boundary_scan(root)?;
    println!("PASS host-check: host={host}, rustc=1.93.0");
    Ok(())
}

fn validate_relative_links(file: &Path, body: &str) -> Result<(), String> {
    let mut remainder = body;
    while let Some(start) = remainder.find("](") {
        let after = &remainder[start + 2..];
        let Some(end) = after.find(')') else {
            return Err(format!("unterminated Markdown link in {}", file.display()));
        };
        let target = after[..end].trim().trim_matches(['<', '>']);
        let path_part = target.split('#').next().unwrap_or_default();
        if !path_part.is_empty()
            && !path_part.starts_with("http://")
            && !path_part.starts_with("https://")
            && !path_part.starts_with("mailto:")
        {
            let resolved = file
                .parent()
                .ok_or_else(|| format!("no parent for {}", file.display()))?
                .join(path_part);
            if !resolved.exists() {
                return Err(format!(
                    "broken relative link in {}: {target}",
                    file.display()
                ));
            }
        }
        remainder = &after[end + 1..];
    }
    Ok(())
}

fn table_ids(body: &str, prefix: &str) -> BTreeSet<String> {
    body.lines()
        .filter_map(|line| line.strip_prefix("| "))
        .filter_map(|line| line.split_whitespace().next())
        .filter(|id| id.starts_with(prefix))
        .map(str::to_owned)
        .collect()
}

fn require_sequential(ids: &BTreeSet<String>, prefix: &str, width: usize) -> Result<(), String> {
    for (index, id) in ids.iter().enumerate() {
        let expected = format!("{prefix}{:0width$}", index + 1);
        if id != &expected {
            return Err(format!(
                "non-sequential {prefix} IDs: expected {expected}, got {id}"
            ));
        }
    }
    Ok(())
}

fn require_count(label: &str, actual: usize, expected: usize) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label}: expected {expected}, got {actual}"))
    }
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn collect_files(
    root: &Path,
    extension: Option<&str>,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, extension, output)?;
        } else if extension.is_none_or(|expected| path.extension() == Some(OsStr::new(expected))) {
            output.push(path);
        }
    }
    Ok(())
}

fn collect_named_files(root: &Path, name: &str, output: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.file_name() == Some(OsStr::new(".git"))
            || path.file_name() == Some(OsStr::new("target"))
            || path.starts_with(root.join("incubator/gothic-importer"))
        {
            continue;
        }
        if path.is_dir() {
            collect_named_files(&path, name, output)?;
        } else if path.file_name() == Some(OsStr::new(name)) {
            output.push(path);
        }
    }
    Ok(())
}

fn run_checked(root: &Path, program: &str, arguments: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .status()
        .map_err(|error| format!("failed to run {program}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} {} failed with {status}",
            arguments.join(" ")
        ))
    }
}

fn run_output(root: &Path, program: &str, arguments: &[&str]) -> Result<Output, String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run {program}: {error}"))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "{program} {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
