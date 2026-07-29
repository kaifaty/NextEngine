use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_RUST_SOURCE_LINES: usize = 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SourceSizeExemption {
    path: &'static str,
    max_lines: usize,
}

// Temporary ratchet for the pre-existing oversized files. Refactoring a file
// below the hard limit must remove its entry in the same change.
const SOURCE_SIZE_EXEMPTIONS: &[SourceSizeExemption] = &[
    SourceSizeExemption {
        path: "crates/runtime/src/engine.rs",
        max_lines: 2_817,
    },
    SourceSizeExemption {
        path: "crates/verification/src/player_fixture.rs",
        max_lines: 2_542,
    },
    SourceSizeExemption {
        path: "crates/verification/src/persistence_replay.rs",
        max_lines: 1_738,
    },
    SourceSizeExemption {
        path: "crates/verification/src/lib.rs",
        max_lines: 1_318,
    },
];

pub fn boundary_scan(root: &Path) -> Result<(), String> {
    validate_source_file_sizes(root)?;
    validate_public_contracts(root)?;
    validate_mechanics_package_boundary(root)?;
    validate_importer_boundary(root)?;
    validate_ffi_policy(root)?;
    println!(
        "PASS boundary-scan: bounded Rust sources, public contracts, public mechanics package path, importer boundary and audited FFI allowlist verified"
    );
    Ok(())
}

fn validate_mechanics_package_boundary(root: &Path) -> Result<(), String> {
    for application in ["apps/game/Cargo.toml", "apps/headless/Cargo.toml"] {
        let body = read(&root.join(application))?;
        if body.contains("next_mechanics") || body.contains("next_rpg") {
            return Err(format!(
                "BOUNDARY_APP_PRIVATE_MECHANICS_DEPENDENCY: {application}"
            ));
        }
    }
    if let Some(path) = find_source_file_containing(
        &root.join("crates/runtime/src"),
        "org.nextengine.core.combat",
    )? {
        return Err(format!(
            "BOUNDARY_RUNTIME_FIRST_PARTY_COMBAT_ID: {}",
            path.display()
        ));
    }
    let mechanics = read(&root.join("crates/mechanics/src/lib.rs"))?;
    let production = mechanics
        .split_once("#[cfg(test)]")
        .map_or(mechanics.as_str(), |(production, _)| production);
    if production.contains("org.nextengine.core.combat") {
        return Err("BOUNDARY_MECHANICS_HOST_FIRST_PARTY_PACKAGE_ID".to_owned());
    }
    Ok(())
}

fn validate_source_file_sizes(root: &Path) -> Result<(), String> {
    let mut source_sizes = BTreeMap::new();
    for relative_root in ["apps", "crates", "tools"] {
        let source_root = root.join(relative_root);
        if !source_root.is_dir() {
            return Err(format!("SOURCE_ROOT_MISSING: {}", source_root.display()));
        }
        let mut files = Vec::new();
        collect_strict_source_files(&source_root, &mut files)?;
        files.sort();
        for file in files {
            let relative = workspace_relative_path(root, &file)?;
            let line_count = read(&file)?.lines().count();
            if source_sizes.insert(relative.clone(), line_count).is_some() {
                return Err(format!("SOURCE_FILE_DUPLICATE: {relative}"));
            }
        }
    }
    validate_source_size_inventory(&source_sizes, SOURCE_SIZE_EXEMPTIONS)
}

fn validate_source_size_inventory(
    source_sizes: &BTreeMap<String, usize>,
    exemptions: &[SourceSizeExemption],
) -> Result<(), String> {
    let mut exemption_limits = BTreeMap::new();
    for exemption in exemptions {
        if exemption_limits
            .insert(exemption.path, exemption.max_lines)
            .is_some()
        {
            return Err(format!(
                "SOURCE_FILE_SIZE_EXEMPTION_DUPLICATE: {}",
                exemption.path
            ));
        }
    }

    for (path, line_count) in source_sizes {
        let Some(max_lines) = exemption_limits.get(path.as_str()).copied() else {
            if *line_count > MAX_RUST_SOURCE_LINES {
                return Err(format!(
                    "SOURCE_FILE_TOO_LARGE: {path} has {line_count} lines; limit is {MAX_RUST_SOURCE_LINES}"
                ));
            }
            continue;
        };
        if *line_count <= MAX_RUST_SOURCE_LINES {
            return Err(format!(
                "SOURCE_FILE_SIZE_EXEMPTION_STALE: {path} has {line_count} lines; remove its exemption"
            ));
        }
        if *line_count > max_lines {
            return Err(format!(
                "SOURCE_FILE_SIZE_REGRESSION: {path} has {line_count} lines; exemption ceiling is {max_lines}"
            ));
        }
    }

    for path in exemption_limits.keys() {
        if !source_sizes.contains_key(*path) {
            return Err(format!("SOURCE_FILE_SIZE_EXEMPTION_MISSING: {path}"));
        }
    }
    Ok(())
}

fn workspace_relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        format!(
            "SOURCE_FILE_OUTSIDE_WORKSPACE: {} is not under {}",
            path.display(),
            root.display()
        )
    })?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
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

fn find_source_file_containing(root: &Path, needle: &str) -> Result<Option<PathBuf>, String> {
    let mut files = Vec::new();
    collect_strict_source_files(root, &mut files)?;
    files.sort();
    for file in files {
        if read(&file)?.contains(needle) {
            return Ok(Some(file));
        }
    }
    Ok(None)
}

fn collect_strict_source_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    let root_metadata =
        fs::symlink_metadata(root).map_err(|error| format!("{}: {error}", root.display()))?;
    if root_metadata.file_type().is_symlink() {
        return Err(format!("SOURCE_SYMLINK_FORBIDDEN: {}", root.display()));
    }
    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if file_type.is_symlink() {
            return Err(format!("SOURCE_SYMLINK_FORBIDDEN: {}", path.display()));
        }
        if file_type.is_dir() {
            collect_strict_source_files(&path, output)?;
        } else if file_type.is_file() && path.extension() == Some(OsStr::new("rs")) {
            output.push(path);
        }
    }
    Ok(())
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
    use std::collections::BTreeMap;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        MAX_RUST_SOURCE_LINES, SourceSizeExemption, contains_forbidden_public_token,
        contains_unsafe_code, find_source_file_containing, validate_source_size_inventory,
    };

    fn temporary_source_root(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must follow the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "nextengine-boundary-scan-{label}-{}-{unique}",
            std::process::id()
        ))
    }

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

    #[test]
    fn source_size_inventory_rejects_a_new_oversized_file() {
        let source_sizes = BTreeMap::from([(
            "crates/example/src/lib.rs".to_owned(),
            MAX_RUST_SOURCE_LINES + 1,
        )]);

        let error = validate_source_size_inventory(&source_sizes, &[])
            .expect_err("an unexempted oversized source must fail");

        assert!(error.starts_with("SOURCE_FILE_TOO_LARGE: crates/example/src/lib.rs"));
    }

    #[test]
    fn source_size_inventory_ratchets_exemptions() {
        let exemption = SourceSizeExemption {
            path: "crates/example/src/lib.rs",
            max_lines: 1_200,
        };
        let accepted = BTreeMap::from([(exemption.path.to_owned(), 1_200)]);
        validate_source_size_inventory(&accepted, &[exemption])
            .expect("a source at its exemption ceiling remains accepted");

        let grown = BTreeMap::from([(exemption.path.to_owned(), 1_201)]);
        let error = validate_source_size_inventory(&grown, &[exemption])
            .expect_err("growth above an exemption ceiling must fail");
        assert!(error.starts_with("SOURCE_FILE_SIZE_REGRESSION:"));

        let refactored = BTreeMap::from([(exemption.path.to_owned(), MAX_RUST_SOURCE_LINES)]);
        let error = validate_source_size_inventory(&refactored, &[exemption])
            .expect_err("a completed refactor must remove its exemption");
        assert!(error.starts_with("SOURCE_FILE_SIZE_EXEMPTION_STALE:"));
    }

    #[test]
    fn source_size_inventory_rejects_missing_and_duplicate_exemptions() {
        let exemption = SourceSizeExemption {
            path: "crates/example/src/lib.rs",
            max_lines: 1_200,
        };
        let error = validate_source_size_inventory(&BTreeMap::new(), &[exemption])
            .expect_err("an exemption for a missing file must fail");
        assert_eq!(
            error,
            "SOURCE_FILE_SIZE_EXEMPTION_MISSING: crates/example/src/lib.rs"
        );

        let source_sizes = BTreeMap::from([(exemption.path.to_owned(), 1_100)]);
        let error = validate_source_size_inventory(&source_sizes, &[exemption, exemption])
            .expect_err("duplicate exemptions must fail");
        assert_eq!(
            error,
            "SOURCE_FILE_SIZE_EXEMPTION_DUPLICATE: crates/example/src/lib.rs"
        );
    }

    #[test]
    fn source_token_scan_descends_into_target_named_module() {
        let root = temporary_source_root("target-module");
        let nested = root.join("target/mod.rs");
        fs::create_dir_all(
            nested
                .parent()
                .expect("the nested fixture must have a parent"),
        )
        .expect("the test source tree must be created");
        fs::write(&nested, "const PACKAGE: &str = \"forbidden.package\";\n")
            .expect("the nested source fixture must be written");

        let found = find_source_file_containing(&root, "forbidden.package")
            .expect("the recursive scan must succeed");

        assert_eq!(found.as_deref(), Some(nested.as_path()));
        fs::remove_dir_all(&root).expect("the test source tree must be removed");
    }

    #[cfg(unix)]
    #[test]
    fn source_token_scan_rejects_directory_symlinks() {
        let root = temporary_source_root("symlink");
        let external = temporary_source_root("external");
        fs::create_dir_all(&root).expect("the source root must be created");
        fs::create_dir_all(&external).expect("the external directory must be created");
        symlink(&external, root.join("escaped")).expect("the directory symlink must be created");

        let error = find_source_file_containing(&root, "forbidden.package")
            .expect_err("source scans must not follow directory symlinks");

        assert!(error.starts_with("SOURCE_SYMLINK_FORBIDDEN:"));
        fs::remove_dir_all(&root).expect("the source root must be removed");
        fs::remove_dir_all(&external).expect("the external directory must be removed");
    }
}
