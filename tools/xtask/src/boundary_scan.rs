use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod allocator_counter;
mod source_layout;

use source_layout::{collect_strict_source_files, validate_source_layout};

pub fn boundary_scan(root: &Path) -> Result<(), String> {
    validate_source_layout(root)?;
    validate_publish_policy(root)?;
    validate_public_contracts(root)?;
    validate_mechanics_package_boundary(root)?;
    validate_production_verification_boundary(root)?;
    validate_importer_boundary(root)?;
    validate_unsafe_policy(root)?;
    Ok(())
}

fn validate_production_verification_boundary(root: &Path) -> Result<(), String> {
    for manifest in ["apps/game/Cargo.toml", "apps/headless/Cargo.toml"] {
        let body = read(&root.join(manifest))?;
        let dependencies = body
            .split_once("[dependencies]")
            .map(|(_, body)| body.split("\n[").next().unwrap_or(body))
            .unwrap_or_default();
        if dependencies.contains("next_verification") {
            return Err(format!(
                "BOUNDARY_PRODUCTION_VERIFICATION_DEPENDENCY: {manifest}"
            ));
        }
    }
    for directory in [
        "apps/game/src",
        "apps/headless/src",
        "crates/application/src",
        "crates/assets/src",
        "crates/project/src",
        "crates/reference-game/src",
        "crates/runtime/src",
    ] {
        if let Some(path) = find_source_file_containing(&root.join(directory), "next_verification")?
        {
            return Err(format!(
                "BOUNDARY_PRODUCTION_VERIFICATION_REFERENCE: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn validate_publish_policy(root: &Path) -> Result<(), String> {
    let mut manifests = Vec::new();
    collect_named_files(root, "Cargo.toml", &mut manifests)?;
    for manifest in manifests
        .into_iter()
        .filter(|manifest| manifest != &root.join("Cargo.toml"))
    {
        let body = read(&manifest)?;
        let Some(name) = package_name(&body) else {
            continue;
        };
        if !package_publish_is_false(&body) {
            return Err(format!(
                "PACKAGE_PUBLISH_POLICY_MISSING: {name} in {}",
                manifest.display()
            ));
        }
    }
    Ok(())
}

fn package_publish_is_false(manifest: &str) -> bool {
    let mut in_package = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package && trimmed == "publish = false" {
            return true;
        }
    }
    false
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

fn validate_unsafe_policy(root: &Path) -> Result<(), String> {
    let workspace_manifest = read(&root.join("Cargo.toml"))?;
    if !workspace_manifest.contains("[workspace.lints.rust]\nunsafe_code = \"forbid\"") {
        return Err("UNSAFE_WORKSPACE_FORBID_MISSING".to_owned());
    }
    let ffi_allowlist = parse_allowlist_section(
        &workspace_manifest,
        "[workspace.metadata.nextengine.ffi]",
        "FFI",
    )?;
    let tooling_allowlist = parse_allowlist_section(
        &workspace_manifest,
        "[workspace.metadata.nextengine.unsafe]",
        "UNSAFE",
    )?;
    let allowlist = combine_unsafe_allowlists(&ffi_allowlist, &tooling_allowlist)?;

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
            return Err(format!("UNSAFE_DUPLICATE_PACKAGE_NAME: {name}"));
        }
        if ffi_allowlist.contains(&name) {
            validate_allowlisted_manifest(&name, &manifest, &body, "ffi_adr", "FFI")?;
        } else if tooling_allowlist.contains(&name) {
            validate_allowlisted_manifest(&name, &manifest, &body, "unsafe_adr", "UNSAFE")?;
        } else if !body.contains("[lints]\nworkspace = true") {
            return Err(format!(
                "UNSAFE_LINT_INHERITANCE_BYPASS: {name} in {}",
                manifest.display()
            ));
        }
    }
    for allowed in &ffi_allowlist {
        if !packages.contains_key(allowed) {
            return Err(format!("FFI_ALLOWLIST_UNKNOWN_CRATE: {allowed}"));
        }
    }
    for allowed in &tooling_allowlist {
        if !packages.contains_key(allowed) {
            return Err(format!("UNSAFE_ALLOWLIST_UNKNOWN_CRATE: {allowed}"));
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
            .ok_or_else(|| format!("UNSAFE_UNOWNED_SOURCE: {}", file.display()))?;
        if !allowlist.contains(&package) {
            return Err(format!(
                "UNSAFE_SOURCE_OUTSIDE_ALLOWLIST: {package} in {}",
                file.display()
            ));
        }
    }
    allocator_counter::validate(root)?;
    Ok(())
}

fn combine_unsafe_allowlists(
    ffi_allowlist: &BTreeSet<String>,
    tooling_allowlist: &BTreeSet<String>,
) -> Result<BTreeSet<String>, String> {
    if let Some(overlap) = ffi_allowlist.intersection(tooling_allowlist).next() {
        return Err(format!("UNSAFE_ALLOWLIST_OVERLAP: {overlap}"));
    }
    Ok(ffi_allowlist.union(tooling_allowlist).cloned().collect())
}

fn parse_allowlist_section(
    manifest: &str,
    section: &str,
    diagnostic_prefix: &str,
) -> Result<BTreeSet<String>, String> {
    let mut in_section = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }
        if in_section && let Some(value) = trimmed.strip_prefix("allowed_crates = ") {
            let inner = value
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
                .ok_or_else(|| format!("{diagnostic_prefix}_ALLOWLIST_INVALID_ARRAY"))?;
            let mut crates = BTreeSet::new();
            for item in inner
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
            {
                let name = item
                    .strip_prefix('"')
                    .and_then(|item| item.strip_suffix('"'))
                    .ok_or_else(|| format!("{diagnostic_prefix}_ALLOWLIST_INVALID_ENTRY"))?;
                if !crates.insert(name.to_owned()) {
                    return Err(format!("{diagnostic_prefix}_ALLOWLIST_DUPLICATE: {name}"));
                }
            }
            return Ok(crates);
        }
    }
    Err(format!("{diagnostic_prefix}_ALLOWLIST_MISSING"))
}

fn validate_allowlisted_manifest(
    name: &str,
    path: &Path,
    body: &str,
    adr_key: &str,
    diagnostic_prefix: &str,
) -> Result<(), String> {
    if body.contains("[lints]\nworkspace = true") {
        return Err(format!(
            "{diagnostic_prefix}_ALLOWLISTED_CRATE_INHERITS_FORBID: {name}"
        ));
    }
    if !body.contains("[lints.rust]")
        || !body.contains("unsafe_code = \"warn\"")
        || !body.contains("unsafe_op_in_unsafe_fn = \"deny\"")
        || !body.contains("[lints.clippy]")
        || !body.contains("undocumented_unsafe_blocks = \"deny\"")
    {
        return Err(format!(
            "{diagnostic_prefix}_ALLOWLISTED_CRATE_POLICY_MISSING: {name} in {}",
            path.display()
        ));
    }
    if !body.contains(&format!("{adr_key} = \"ADR-")) {
        return Err(format!(
            "{diagnostic_prefix}_ALLOWLISTED_CRATE_ADR_MISSING: {name}"
        ));
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
    fs::read_to_string(path)
        .map(|body| normalize_line_endings(&body))
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn normalize_line_endings(body: &str) -> String {
    body.replace("\r\n", "\n")
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
    use std::collections::BTreeSet;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        combine_unsafe_allowlists, contains_forbidden_public_token, contains_unsafe_code,
        find_source_file_containing, normalize_line_endings, package_publish_is_false,
        parse_allowlist_section, validate_allowlisted_manifest,
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
    fn ffi_and_tooling_unsafe_allowlists_are_distinct_and_exact() {
        let manifest = concat!(
            "[workspace.metadata.nextengine.ffi]\n",
            "allowed_crates = [\"ffi_a\", \"ffi_b\"]\n\n",
            "[workspace.metadata.nextengine.unsafe]\n",
            "allowed_crates = [\"tooling_allocator\"]\n",
        );
        let ffi = parse_allowlist_section(manifest, "[workspace.metadata.nextengine.ffi]", "FFI")
            .expect("the FFI allowlist must parse");
        let tooling =
            parse_allowlist_section(manifest, "[workspace.metadata.nextengine.unsafe]", "UNSAFE")
                .expect("the tooling unsafe allowlist must parse");

        assert_eq!(
            ffi,
            BTreeSet::from(["ffi_a".to_owned(), "ffi_b".to_owned()])
        );
        assert_eq!(tooling, BTreeSet::from(["tooling_allocator".to_owned()]));
        assert_eq!(
            combine_unsafe_allowlists(&ffi, &tooling)
                .expect("lists are disjoint")
                .len(),
            3
        );
    }

    #[test]
    fn unsafe_allowlist_overlap_is_rejected() {
        let ffi = BTreeSet::from(["shared".to_owned()]);
        let tooling = BTreeSet::from(["shared".to_owned()]);

        assert_eq!(
            combine_unsafe_allowlists(&ffi, &tooling),
            Err("UNSAFE_ALLOWLIST_OVERLAP: shared".to_owned())
        );
    }

    #[test]
    fn tooling_unsafe_manifest_requires_reviewed_policy_and_adr() {
        let valid = concat!(
            "[package]\nname = \"counter\"\npublish = false\n\n",
            "[package.metadata.nextengine]\nunsafe_adr = \"ADR-039\"\n\n",
            "[lints.rust]\nunsafe_code = \"warn\"\n",
            "unsafe_op_in_unsafe_fn = \"deny\"\n\n",
            "[lints.clippy]\nundocumented_unsafe_blocks = \"deny\"\n",
        );
        validate_allowlisted_manifest(
            "counter",
            std::path::Path::new("counter/Cargo.toml"),
            valid,
            "unsafe_adr",
            "UNSAFE",
        )
        .expect("the reviewed tooling unsafe manifest must pass");

        let missing_adr = valid.replace("unsafe_adr = \"ADR-039\"\n", "");
        assert_eq!(
            validate_allowlisted_manifest(
                "counter",
                std::path::Path::new("counter/Cargo.toml"),
                &missing_adr,
                "unsafe_adr",
                "UNSAFE",
            ),
            Err("UNSAFE_ALLOWLISTED_CRATE_ADR_MISSING: counter".to_owned())
        );
    }

    #[test]
    fn publish_policy_must_be_explicit_and_false_in_the_package_section() {
        assert!(package_publish_is_false(
            "[package]\nname = \"demo\"\npublish = false\n\n[dependencies]\n"
        ));
        assert!(!package_publish_is_false(
            "[package]\nname = \"demo\"\n\n[dependencies]\npublish = false\n"
        ));
        assert!(!package_publish_is_false(
            "[package]\nname = \"demo\"\npublish = true\n"
        ));
    }

    #[test]
    fn manifest_scans_are_independent_of_checkout_line_endings() {
        assert_eq!(
            normalize_line_endings("[workspace.lints.rust]\r\nunsafe_code = \"forbid\"\r\n"),
            "[workspace.lints.rust]\nunsafe_code = \"forbid\"\n"
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
