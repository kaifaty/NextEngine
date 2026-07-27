use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn boundary_scan(root: &Path) -> Result<(), String> {
    validate_public_contracts(root)?;
    validate_importer_boundary(root)?;
    validate_ffi_policy(root)?;
    println!(
        "PASS boundary-scan: public contracts, importer boundary and audited FFI allowlist verified"
    );
    Ok(())
}

fn validate_public_contracts(root: &Path) -> Result<(), String> {
    let contracts_root = root.join("crates/contracts");
    let mut contract_files = Vec::new();
    collect_files(&contracts_root, Some("rs"), &mut contract_files)?;
    let forbidden = [
        "ash::",
        "bevy",
        "daedalus",
        "extern \"c\"",
        "gothic",
        "jolt",
        "physx",
        "*const ",
        "*mut ",
        "vulkan",
        "windows_sys",
        "x11",
    ];
    for file in contract_files {
        let lower = read(&file)?.to_ascii_lowercase();
        for needle in forbidden {
            if contains_forbidden_public_token(&lower, needle) {
                return Err(format!(
                    "BOUNDARY_FORBIDDEN_PUBLIC_TOKEN: {needle} in {}",
                    file.display()
                ));
            }
        }
    }
    Ok(())
}

fn contains_forbidden_public_token(body: &str, needle: &str) -> bool {
    if needle != "ash::" {
        return body.contains(needle);
    }
    body.match_indices(needle).any(|(index, _)| {
        index == 0
            || body
                .as_bytes()
                .get(index - 1)
                .is_some_and(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
    })
}

fn validate_importer_boundary(root: &Path) -> Result<(), String> {
    let mut cargo_files = Vec::new();
    collect_named_files(root, "Cargo.toml", &mut cargo_files)?;
    for file in cargo_files {
        let body = read(&file)?;
        if body.contains("incubator/gothic-importer") || body.contains("gothic-importer") {
            return Err(format!(
                "BOUNDARY_IMPORTER_CARGO_DEPENDENCY: {}",
                file.display()
            ));
        }
    }

    let output = Command::new("git")
        .args(["ls-files", "--", "incubator/gothic-importer"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "BOUNDARY_GIT_SCAN_FAILED: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !output.stdout.is_empty() {
        return Err("BOUNDARY_IMPORTER_TRACKED".to_owned());
    }
    if root.join(".github/workflows").exists() {
        return Err("BOUNDARY_BOOTSTRAP_CI_WORKFLOW_FORBIDDEN".to_owned());
    }
    Ok(())
}

fn validate_ffi_policy(root: &Path) -> Result<(), String> {
    let workspace_manifest = read(&root.join("Cargo.toml"))?;
    if !workspace_manifest.contains("[workspace.lints.rust]\nunsafe_code = \"forbid\"") {
        return Err("FFI_WORKSPACE_UNSAFE_FORBID_MISSING".to_owned());
    }
    let allowlist = parse_ffi_allowlist(&workspace_manifest)?;

    let mut manifests = Vec::new();
    collect_named_files(root, "Cargo.toml", &mut manifests)?;
    let mut packages = BTreeMap::new();
    for manifest in manifests
        .into_iter()
        .filter(|manifest| manifest != &root.join("Cargo.toml"))
    {
        let body = read(&manifest)?;
        let Some(name) = package_name(&body) else {
            continue;
        };
        if packages.insert(name.clone(), manifest.clone()).is_some() {
            return Err(format!("FFI_DUPLICATE_PACKAGE_NAME: {name}"));
        }
        if allowlist.contains(&name) {
            validate_allowlisted_manifest(&name, &manifest, &body)?;
        } else if !body.contains("[lints]\nworkspace = true") {
            return Err(format!(
                "FFI_LINT_INHERITANCE_BYPASS: {name} in {}",
                manifest.display()
            ));
        }
    }
    for allowed in &allowlist {
        if !packages.contains_key(allowed) {
            return Err(format!("FFI_ALLOWLIST_UNKNOWN_CRATE: {allowed}"));
        }
    }

    let mut rust_files = Vec::new();
    collect_source_files(root, &mut rust_files)?;
    for file in rust_files {
        let body = read(&file)?;
        if !contains_unsafe_code(&body) {
            continue;
        }
        let package = owning_package(root, &file, &packages)
            .ok_or_else(|| format!("FFI_UNOWNED_UNSAFE_SOURCE: {}", file.display()))?;
        if !allowlist.contains(&package) {
            return Err(format!(
                "FFI_UNSAFE_OUTSIDE_ALLOWLIST: {package} in {}",
                file.display()
            ));
        }
    }
    Ok(())
}

fn parse_ffi_allowlist(manifest: &str) -> Result<BTreeSet<String>, String> {
    let mut in_section = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == "[workspace.metadata.nextengine.ffi]";
            continue;
        }
        if in_section && let Some(value) = trimmed.strip_prefix("allowed_crates = ") {
            let inner = value
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
                .ok_or_else(|| "FFI_ALLOWLIST_INVALID_ARRAY".to_owned())?;
            let mut crates = BTreeSet::new();
            for item in inner
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
            {
                let name = item
                    .strip_prefix('"')
                    .and_then(|item| item.strip_suffix('"'))
                    .ok_or_else(|| "FFI_ALLOWLIST_INVALID_ENTRY".to_owned())?;
                if !crates.insert(name.to_owned()) {
                    return Err(format!("FFI_ALLOWLIST_DUPLICATE: {name}"));
                }
            }
            return Ok(crates);
        }
    }
    Err("FFI_ALLOWLIST_MISSING".to_owned())
}

fn validate_allowlisted_manifest(name: &str, path: &Path, body: &str) -> Result<(), String> {
    if body.contains("[lints]\nworkspace = true") {
        return Err(format!("FFI_ALLOWLISTED_CRATE_INHERITS_FORBID: {name}"));
    }
    if !body.contains("[lints.rust]")
        || !body.contains("unsafe_code = \"warn\"")
        || !body.contains("unsafe_op_in_unsafe_fn = \"deny\"")
    {
        return Err(format!(
            "FFI_ALLOWLISTED_CRATE_POLICY_MISSING: {name} in {}",
            path.display()
        ));
    }
    if !body.contains("ffi_adr = \"ADR-") {
        return Err(format!("FFI_ALLOWLISTED_CRATE_ADR_MISSING: {name}"));
    }
    Ok(())
}

fn package_name(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package && let Some(value) = trimmed.strip_prefix("name = ") {
            return value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .map(str::to_owned);
        }
    }
    None
}

fn contains_unsafe_code(body: &str) -> bool {
    body.lines().any(|line| {
        let code = strip_comments_and_strings(line);
        code.contains("unsafe {")
            || code.contains("unsafe fn ")
            || code.contains("unsafe impl ")
            || code.contains("unsafe extern")
    })
}

fn strip_comments_and_strings(line: &str) -> String {
    let mut output = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = line.chars().peekable();
    while let Some(character) = chars.next() {
        if !in_string && character == '/' && chars.peek() == Some(&'/') {
            break;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            output.push(' ');
        } else if character == '"' {
            in_string = true;
            output.push(' ');
        } else {
            output.push(character);
        }
    }
    output
}

fn owning_package(
    root: &Path,
    source: &Path,
    packages: &BTreeMap<String, PathBuf>,
) -> Option<String> {
    packages.iter().find_map(|(name, manifest)| {
        let package_root = manifest.parent()?;
        if source.starts_with(package_root) && package_root.starts_with(root) {
            Some(name.clone())
        } else {
            None
        }
    })
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
        if should_skip(&path) {
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

fn collect_source_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if should_skip(&path) {
            continue;
        }
        if path.is_dir() {
            collect_source_files(&path, output)?;
        } else if path.extension() == Some(OsStr::new("rs")) {
            output.push(path);
        }
    }
    Ok(())
}

fn should_skip(path: &Path) -> bool {
    path.file_name() == Some(OsStr::new(".git"))
        || path.file_name() == Some(OsStr::new("target"))
        || path.file_name() == Some(OsStr::new(".codex-artifacts"))
        || path.file_name() == Some(OsStr::new(".venv"))
        || path.file_name() == Some(OsStr::new(".local"))
        || path.file_name() == Some(OsStr::new("__pycache__"))
        || path.ends_with("incubator/gothic-importer")
}

#[cfg(test)]
mod tests {
    use super::{contains_forbidden_public_token, contains_unsafe_code};

    #[test]
    fn ash_path_scan_uses_an_identifier_boundary() {
        assert!(contains_forbidden_public_token("ash::vk", "ash::"));
        assert!(contains_forbidden_public_token("(ash::vk)", "ash::"));
        assert!(!contains_forbidden_public_token(
            "ContentHash::from_bytes",
            "ash::"
        ));
    }

    #[test]
    fn unsafe_scan_detects_extern_blocks_after_string_elision() {
        assert!(contains_unsafe_code("unsafe extern \"C\" { fn call(); }"));
        assert!(!contains_unsafe_code(
            "const NOTE: &str = \"unsafe extern C\";"
        ));
    }
}
