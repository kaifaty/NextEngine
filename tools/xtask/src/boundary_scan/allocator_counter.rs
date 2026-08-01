use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::{
    collect_files, collect_named_files, contains_unsafe_code, read, strip_comments_and_strings,
};

pub(super) fn validate(root: &Path) -> Result<(), String> {
    validate_dependency_boundary(root)?;
    validate_source_boundary(root)?;
    validate_cargo_graph(root)
}

fn validate_dependency_boundary(root: &Path) -> Result<(), String> {
    let expected_consumer = root.join("tools/xtask/Cargo.toml");
    let allocator_manifest = root.join("tools/process-allocation-counter/Cargo.toml");
    let mut manifests = Vec::new();
    collect_named_files(root, "Cargo.toml", &mut manifests)?;
    let mut consumer_seen = false;
    for manifest in manifests {
        if manifest == root.join("Cargo.toml") || manifest == allocator_manifest {
            continue;
        }
        let body = read(&manifest)?;
        if !body.contains("next_process_allocation_counter") {
            continue;
        }
        if manifest != expected_consumer {
            return Err(format!(
                "UNSAFE_ALLOCATOR_COUNTER_FORBIDDEN_CONSUMER: {}",
                manifest.display()
            ));
        }
        consumer_seen = true;
    }
    if !consumer_seen {
        return Err("UNSAFE_ALLOCATOR_COUNTER_XTASK_DEPENDENCY_MISSING".to_owned());
    }
    let xtask_main = read(&root.join("tools/xtask/src/main.rs"))?;
    if !xtask_main.contains("#[global_allocator]")
        || !xtask_main.contains("next_process_allocation_counter::ProcessAllocationCounter")
    {
        return Err("UNSAFE_ALLOCATOR_COUNTER_GLOBAL_HOOK_MISSING".to_owned());
    }
    Ok(())
}

fn validate_source_boundary(root: &Path) -> Result<(), String> {
    let source_root = root.join("tools/process-allocation-counter/src");
    let library_path = source_root.join("lib.rs");
    let active_path = source_root.join("active.rs");
    let tests_path = source_root.join("tests.rs");
    let tests_root = source_root.join("tests");
    let codegen_probe_path = source_root
        .join("bin")
        .join("allocator_counter_codegen_probe.rs");
    let mut source_files = Vec::new();
    collect_files(&source_root, Some("rs"), &mut source_files)?;
    let mut structural_source = String::new();
    for source_file in &source_files {
        let body = read(source_file)?;
        let is_test_source =
            source_file.as_path() == tests_path || source_file.starts_with(&tests_root);
        if !is_test_source {
            structural_source.push_str(&body);
            structural_source.push('\n');
        }
        if contains_unsafe_code(&body)
            && source_file.as_path() != library_path
            && source_file.as_path() != active_path
            && !is_test_source
        {
            return Err(format!(
                "UNSAFE_ALLOCATOR_COUNTER_SOURCE_SCOPE_MISMATCH: {}",
                source_file.display()
            ));
        }
    }
    let library = read(&library_path)?;
    validate_library_source(&structural_source, &structural_source)?;
    let codegen_probe = read(&codegen_probe_path)?;
    if !codegen_probe.contains("#[global_allocator]")
        || !codegen_probe
            .contains("next_process_allocation_counter::ProcessAllocationCounter::system()")
        || codegen_probe.contains("xtask::")
    {
        return Err("UNSAFE_ALLOCATOR_COUNTER_CODEGEN_PROBE_MISMATCH".to_owned());
    }
    if !tests_path.is_file() || !library.contains("#[cfg(test)]") || !library.contains("mod tests;")
    {
        return Err("UNSAFE_ALLOCATOR_COUNTER_TEST_SCOPE_MISSING".to_owned());
    }
    Ok(())
}

fn validate_library_source(source: &str, structural_source: &str) -> Result<(), String> {
    validate_adr_041_structure(structural_source)?;
    let expected_unsafe_functions = [
        "alloc",
        "alloc_foreign_active",
        "alloc_owner_active",
        "alloc_zeroed",
        "alloc_zeroed_foreign_active",
        "alloc_zeroed_owner_active",
        "dealloc",
        "realloc",
        "realloc_foreign_active",
        "realloc_owner_active",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    let mut unsafe_functions = BTreeSet::new();
    let mut unsafe_impls = 0_u32;
    for line in source.lines() {
        let code = strip_comments_and_strings(line);
        if code.contains("unsafe extern") {
            return Err("UNSAFE_ALLOCATOR_COUNTER_EXTERN_FORBIDDEN".to_owned());
        }
        if code.contains("unsafe impl ") {
            if code.trim() != "unsafe impl GlobalAlloc for ProcessAllocationCounter {" {
                return Err(format!(
                    "UNSAFE_ALLOCATOR_COUNTER_IMPL_SCOPE_MISMATCH: {}",
                    code.trim()
                ));
            }
            unsafe_impls = unsafe_impls.saturating_add(1);
        }
        if let Some((_, suffix)) = code.split_once("unsafe fn ") {
            let name = suffix
                .chars()
                .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
                .collect::<String>();
            if name.is_empty() || !unsafe_functions.insert(name) {
                return Err("UNSAFE_ALLOCATOR_COUNTER_FUNCTION_SCOPE_MISMATCH".to_owned());
            }
        }
    }
    validate_unsafe_blocks(source)?;
    if unsafe_impls != 1 || unsafe_functions != expected_unsafe_functions {
        return Err("UNSAFE_ALLOCATOR_COUNTER_BOUNDARY_MISMATCH".to_owned());
    }
    Ok(())
}

fn validate_unsafe_blocks(source: &str) -> Result<(), String> {
    let code = source
        .lines()
        .map(strip_comments_and_strings)
        .collect::<Vec<_>>()
        .join("\n");
    let compact = code
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let allowed = [
        "self.system.alloc(",
        "self.system.alloc_zeroed(",
        "self.system.realloc(",
        "self.system.dealloc(",
        "self.alloc_owner_active(",
        "self.alloc_foreign_active(",
        "self.alloc_zeroed_owner_active(",
        "self.alloc_zeroed_foreign_active(",
        "self.realloc_owner_active(",
        "self.realloc_foreign_active(",
    ];
    for (start, _) in compact.match_indices("unsafe{") {
        let body_start = start + "unsafe".len();
        let Some(block) = brace_block(&compact, body_start) else {
            return Err("UNSAFE_ALLOCATOR_COUNTER_BLOCK_SCOPE_MISMATCH: unclosed".to_owned());
        };
        if !allowed.iter().any(|call| block.contains(call)) {
            return Err(
                "UNSAFE_ALLOCATOR_COUNTER_BLOCK_SCOPE_MISMATCH: non-System call".to_owned(),
            );
        }
    }
    Ok(())
}

fn validate_adr_041_structure(source: &str) -> Result<(), String> {
    if source.contains("/*") || source.contains("*/") {
        return Err("UNSAFE_ALLOCATOR_COUNTER_ADR042_BLOCK_COMMENT_FORBIDDEN".to_owned());
    }
    let code = source
        .lines()
        .map(strip_comments_and_strings)
        .collect::<Vec<_>>()
        .join("\n");
    let compact = code
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let required = [
        ("4_096", "fixed slot count"),
        ("#[repr(align(128))]", "cache-line-isolated slot layout"),
        (
            "staticSLOTS:[AllocationSlot;SLOT_COUNT]",
            "static fixed-capacity slot array",
        ),
        ("thread_local!", "thread-local selector"),
        (
            "staticTHREAD_STATE:AllocationThreadState=const{AllocationThreadState::new()};",
            "const-initialized thread-local state",
        ),
        (
            "size_of::<AllocationSlot>()==128",
            "exact 128-byte slot size assertion",
        ),
        (
            "align_of::<AllocationSlot>()==128",
            "exact 128-byte slot alignment assertion",
        ),
        (
            "!needs_drop::<AllocationThreadState>()",
            "destructor-free TLS assertion",
        ),
        (
            "same_thread:PhantomData<Rc<()>>",
            "non-Send and non-Sync owner token marker",
        ),
        ("owner_window_id:Cell<u64>", "exact owner window cookie"),
        ("owner_alloc_count:Cell<u64>", "owner allocation count"),
        ("owner_alloc_bytes:Cell<u64>", "owner allocation bytes"),
        (
            "owner_alloc_zeroed_count:Cell<u64>",
            "owner zeroed-allocation count",
        ),
        (
            "owner_alloc_zeroed_bytes:Cell<u64>",
            "owner zeroed-allocation bytes",
        ),
        ("owner_realloc_count:Cell<u64>", "owner reallocation count"),
        ("owner_realloc_bytes:Cell<u64>", "owner reallocation bytes"),
        (
            "prepare_owner_thread(window_id)",
            "owner registration and TLS preparation before Active",
        ),
        (
            "owner_window_id.set(window_id)",
            "exact owner cookie publication",
        ),
        ("owner_window_id.set(0)", "exact owner cookie cleanup"),
        (
            "fetch_add(1,Ordering::SeqCst)",
            "foreign sequentially-consistent callback admission",
        ),
        (
            "fetch_add(0,Ordering::SeqCst)",
            "sequentially-consistent close handshake",
        ),
        ("fnalloc_owner_active(", "allocation owner dispatcher"),
        ("fnalloc_foreign_active(", "allocation foreign helper"),
        (
            "fnalloc_zeroed_owner_active(",
            "zeroed-allocation owner dispatcher",
        ),
        (
            "fnalloc_zeroed_foreign_active(",
            "zeroed-allocation foreign helper",
        ),
        ("fnrealloc_owner_active(", "reallocation owner dispatcher"),
        ("fnrealloc_foreign_active(", "reallocation foreign helper"),
    ];
    for (anchor, label) in required {
        if !compact.contains(anchor) {
            return Err(format!(
                "UNSAFE_ALLOCATOR_COUNTER_ADR041_STRUCTURE_MISSING: {label}"
            ));
        }
    }

    validate_owner_publication_order(&compact)?;
    validate_owner_dispatchers(&compact)?;
    validate_foreign_admission(&compact)?;
    validate_dealloc_pass_through(&compact)?;

    let forbidden = [
        "Vec<",
        "Box<",
        "Mutex<",
        "RwLock<",
        "OnceLock<",
        "LazyLock<",
        "UnsafeCell",
        "ThreadId",
        "thread::current(",
        "Rc::new(",
        "staticmut",
        "TlsAlloc",
        "TlsGetValue",
        "catch_unwind(",
        "panic!(",
        "unreachable!(",
        "in_callback",
        "reject_recursion",
    ];
    if let Some(token) = forbidden.iter().find(|token| compact.contains(**token)) {
        return Err(format!(
            "UNSAFE_ALLOCATOR_COUNTER_ADR041_FORBIDDEN_SOURCE: {token}"
        ));
    }
    Ok(())
}

fn validate_owner_publication_order(compact: &str) -> Result<(), String> {
    let begin = compact_function(compact, "begin").ok_or_else(|| {
        "UNSAFE_ALLOCATOR_COUNTER_ADR041_STRUCTURE_MISSING: begin function".to_owned()
    })?;
    let prepare = begin.find("prepare_owner_thread(window_id)");
    let publish = begin.find("encode_control(PHASE_ACTIVE,window_id)");
    if prepare.is_none() || publish.is_none() || prepare >= publish {
        return Err("UNSAFE_ALLOCATOR_COUNTER_ADR041_OWNER_PUBLICATION_ORDER_MISMATCH".to_owned());
    }

    let prepare_owner = compact_function(compact, "prepare_owner_thread").ok_or_else(|| {
        "UNSAFE_ALLOCATOR_COUNTER_ADR041_STRUCTURE_MISSING: owner preparation".to_owned()
    })?;
    for anchor in [
        "slot_index.get()==UNCLAIMED_SLOT",
        "claim_slot_slow()",
        "owner_window_id.set(window_id)",
        "reset_owner_counters()",
    ] {
        if !prepare_owner.contains(anchor) {
            return Err(format!(
                "UNSAFE_ALLOCATOR_COUNTER_ADR041_OWNER_PREPARATION_MISMATCH: {anchor}"
            ));
        }
    }
    Ok(())
}

fn validate_owner_dispatchers(compact: &str) -> Result<(), String> {
    for (owner, foreign, system) in [
        (
            "alloc_owner_active",
            "alloc_foreign_active",
            "self.system.alloc(",
        ),
        (
            "alloc_zeroed_owner_active",
            "alloc_zeroed_foreign_active",
            "self.system.alloc_zeroed(",
        ),
        (
            "realloc_owner_active",
            "realloc_foreign_active",
            "self.system.realloc(",
        ),
    ] {
        let function = compact_function(compact, owner)
            .ok_or_else(|| format!("UNSAFE_ALLOCATOR_COUNTER_ADR041_STRUCTURE_MISSING: {owner}"))?;
        if !function.contains("THREAD_STATE.try_with(")
            || !function.contains("owner_window_id.get()")
            || !function.contains("control_window(observed)")
            || !function.contains(foreign)
            || !function.contains(system)
        {
            return Err(format!(
                "UNSAFE_ALLOCATOR_COUNTER_ADR041_OWNER_DISPATCH_MISMATCH: {owner}"
            ));
        }
        let without_tls_name = function.replace("THREAD_STATE", "");
        if [
            "SLOTS",
            "NEXT_SLOT",
            "STATE.",
            ".sequence",
            "Ordering::",
            "fetch_add(",
            "fetch_or(",
            "compare_exchange",
        ]
        .iter()
        .any(|token| without_tls_name.contains(token))
        {
            return Err(format!(
                "UNSAFE_ALLOCATOR_COUNTER_ADR041_OWNER_SHARED_STATE_MISMATCH: {owner}"
            ));
        }
    }
    Ok(())
}

fn validate_foreign_admission(compact: &str) -> Result<(), String> {
    for (foreign, system) in [
        ("alloc_foreign_active", "self.system.alloc("),
        ("alloc_zeroed_foreign_active", "self.system.alloc_zeroed("),
        ("realloc_foreign_active", "self.system.realloc("),
    ] {
        let function = compact_function(compact, foreign).ok_or_else(|| {
            format!("UNSAFE_ALLOCATOR_COUNTER_ADR041_STRUCTURE_MISSING: {foreign}")
        })?;
        if !function.contains("admit_nonrecursive(") || !function.contains(system) {
            return Err(format!(
                "UNSAFE_ALLOCATOR_COUNTER_ADR041_FOREIGN_DISPATCH_MISMATCH: {foreign}"
            ));
        }
    }

    let admission = compact_function(compact, "admit_nonrecursive").ok_or_else(|| {
        "UNSAFE_ALLOCATOR_COUNTER_ADR041_STRUCTURE_MISSING: foreign admission".to_owned()
    })?;
    let odd = admission.find("fetch_add(1,Ordering::SeqCst)");
    let postcheck = admission.find("STATE.control.load(Ordering::SeqCst)");
    let identity = admission.find("confirmed!=observed");
    if count_occurrences(admission, "fetch_add(1,Ordering::SeqCst)") != 1
        || count_occurrences(admission, "STATE.control.load(Ordering::SeqCst)") != 1
        || odd.is_none()
        || postcheck.is_none()
        || identity.is_none()
        || odd >= postcheck
        || postcheck >= identity
    {
        return Err("UNSAFE_ALLOCATOR_COUNTER_ADR041_FOREIGN_ADMISSION_MISMATCH".to_owned());
    }
    Ok(())
}

fn validate_dealloc_pass_through(compact: &str) -> Result<(), String> {
    for helper in ["dealloc_owner_active", "dealloc_foreign_active"] {
        if compact.contains(&format!("fn{helper}(")) {
            return Err(format!(
                "UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_HELPER_FORBIDDEN: {helper}"
            ));
        }
    }

    const GLOBAL_ALLOC_IMPL: &str = "unsafeimplGlobalAllocforProcessAllocationCounter";
    if count_occurrences(compact, GLOBAL_ALLOC_IMPL) != 1 {
        return Err("UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISSING".to_owned());
    }
    let impl_start = compact
        .find(GLOBAL_ALLOC_IMPL)
        .ok_or_else(|| "UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISSING".to_owned())?;
    let impl_body_start = impl_start
        + compact[impl_start..].find('{').ok_or_else(|| {
            "UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISMATCH".to_owned()
        })?;
    let impl_body = brace_block(compact, impl_body_start).ok_or_else(|| {
        "UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISMATCH".to_owned()
    })?;
    const DEALLOC_SIGNATURE: &str = "unsafefndealloc(";
    if count_occurrences(impl_body, DEALLOC_SIGNATURE) != 1 {
        return Err("UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISMATCH".to_owned());
    }
    let function = compact_function_with_signature(impl_body, DEALLOC_SIGNATURE)
        .ok_or_else(|| "UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISSING".to_owned())?;
    let body_start = function.find('{').ok_or_else(|| {
        "UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISMATCH".to_owned()
    })?;
    if !matches!(
        &function[body_start..],
        "{unsafe{self.system.dealloc(ptr,layout)};}" | "{unsafe{self.system.dealloc(ptr,layout)}}"
    ) {
        return Err("UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISMATCH".to_owned());
    }
    Ok(())
}

fn compact_function<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let signature = format!("fn{name}(");
    compact_function_with_signature(source, &signature)
}

fn compact_function_with_signature<'a>(source: &'a str, signature: &str) -> Option<&'a str> {
    let start = source.find(signature)?;
    let body_start = start + source[start..].find('{')?;
    let body = brace_block(source, body_start)?;
    source.get(start..body_start + body.len())
}

fn brace_block(source: &str, body_start: usize) -> Option<&str> {
    if source.get(body_start..)?.chars().next()? != '{' {
        return None;
    }
    let mut depth = 0_u32;
    for (relative, character) in source[body_start..].char_indices() {
        match character {
            '{' => depth = depth.checked_add(1)?,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return source.get(body_start..=body_start + relative);
                }
            }
            _ => {}
        }
    }
    None
}

fn count_occurrences(body: &str, needle: &str) -> usize {
    body.match_indices(needle).count()
}

#[derive(Deserialize)]
struct CargoMetadataV1 {
    packages: Vec<CargoMetadataPackageV1>,
}

#[derive(Deserialize)]
struct CargoMetadataPackageV1 {
    name: String,
    dependencies: Vec<CargoMetadataDependencyV1>,
}

#[derive(Deserialize)]
struct CargoMetadataDependencyV1 {
    name: String,
}

fn validate_cargo_graph(root: &Path) -> Result<(), String> {
    let output = Command::new("cargo")
        .args(["metadata", "--locked", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("UNSAFE_CARGO_METADATA_FAILED: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "UNSAFE_CARGO_METADATA_FAILED: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata = serde_json::from_slice::<CargoMetadataV1>(&output.stdout)
        .map_err(|error| format!("UNSAFE_CARGO_METADATA_INVALID: {error}"))?;
    validate_metadata(&metadata)
}

fn validate_metadata(metadata: &CargoMetadataV1) -> Result<(), String> {
    const COUNTER: &str = "next_process_allocation_counter";
    const OWNER: &str = "xtask";
    let workspace_names = metadata
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect::<BTreeSet<_>>();
    let Some(counter_package) = metadata
        .packages
        .iter()
        .find(|package| package.name == COUNTER)
    else {
        return Err("UNSAFE_ALLOCATOR_COUNTER_PACKAGE_MISSING".to_owned());
    };
    if !counter_package.dependencies.is_empty() {
        let mut dependencies = counter_package
            .dependencies
            .iter()
            .map(|dependency| dependency.name.as_str())
            .collect::<Vec<_>>();
        dependencies.sort_unstable();
        return Err(format!(
            "UNSAFE_ALLOCATOR_COUNTER_STD_ONLY_VIOLATION: {}",
            dependencies.join(",")
        ));
    }
    let graph = metadata
        .packages
        .iter()
        .map(|package| {
            let dependencies = package
                .dependencies
                .iter()
                .filter(|dependency| workspace_names.contains(dependency.name.as_str()))
                .map(|dependency| dependency.name.as_str())
                .collect::<BTreeSet<_>>();
            (package.name.as_str(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    let direct_consumers = graph
        .iter()
        .filter_map(|(package, dependencies)| dependencies.contains(COUNTER).then_some(*package))
        .collect::<Vec<_>>();
    if direct_consumers != [OWNER] {
        return Err(format!(
            "UNSAFE_ALLOCATOR_COUNTER_REVERSE_DEPENDENCY_MISMATCH: {}",
            direct_consumers.join(",")
        ));
    }
    for package in graph.keys().copied() {
        if package == OWNER || package == COUNTER {
            continue;
        }
        if dependency_reaches(&graph, package, COUNTER) {
            return Err(format!(
                "UNSAFE_ALLOCATOR_COUNTER_TRANSITIVE_CONSUMER: {package}"
            ));
        }
    }
    Ok(())
}

fn dependency_reaches<'a>(
    graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    start: &'a str,
    target: &str,
) -> bool {
    let mut pending = vec![start];
    let mut visited = BTreeSet::new();
    while let Some(package) = pending.pop() {
        if !visited.insert(package) {
            continue;
        }
        let Some(dependencies) = graph.get(package) else {
            continue;
        };
        if dependencies.contains(target) {
            return true;
        }
        pending.extend(dependencies.iter().copied());
    }
    false
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

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
    fn dependency_is_xtask_only() {
        let root = temporary_source_root("allocator-consumer");
        let counter = root.join("tools/process-allocation-counter");
        let xtask = root.join("tools/xtask");
        let game = root.join("apps/game");
        fs::create_dir_all(&counter).expect("the counter directory must be created");
        fs::create_dir_all(xtask.join("src")).expect("the xtask directory must be created");
        fs::create_dir_all(&game).expect("the game directory must be created");
        fs::write(root.join("Cargo.toml"), "[workspace]\n")
            .expect("the root manifest must be written");
        fs::write(
            counter.join("Cargo.toml"),
            "[package]\nname = \"next_process_allocation_counter\"\n",
        )
        .expect("the counter manifest must be written");
        fs::write(
            xtask.join("Cargo.toml"),
            "[package]\nname = \"xtask\"\n[dependencies]\nnext_process_allocation_counter.workspace = true\n",
        )
        .expect("the xtask manifest must be written");
        fs::write(
            xtask.join("src/main.rs"),
            "#[global_allocator]\nstatic PROCESS_ALLOCATOR: next_process_allocation_counter::ProcessAllocationCounter = next_process_allocation_counter::ProcessAllocationCounter::system();\n",
        )
        .expect("the xtask hook source must be written");
        fs::write(game.join("Cargo.toml"), "[package]\nname = \"game\"\n")
            .expect("the game manifest must be written");

        validate_dependency_boundary(&root).expect("xtask must be the sole counter consumer");

        fs::write(
            game.join("Cargo.toml"),
            "[package]\nname = \"game\"\n[dependencies]\nnext_process_allocation_counter.workspace = true\n",
        )
        .expect("the forbidden consumer manifest must be written");
        let error =
            validate_dependency_boundary(&root).expect_err("a shipping consumer must be rejected");
        assert!(error.starts_with("UNSAFE_ALLOCATOR_COUNTER_FORBIDDEN_CONSUMER:"));
        fs::remove_dir_all(&root).expect("the boundary fixture must be removed");
    }

    #[test]
    fn metadata_rejects_a_transitive_shipping_consumer() {
        let metadata = CargoMetadataV1 {
            packages: vec![
                CargoMetadataPackageV1 {
                    name: "next_process_allocation_counter".to_owned(),
                    dependencies: Vec::new(),
                },
                CargoMetadataPackageV1 {
                    name: "xtask".to_owned(),
                    dependencies: vec![CargoMetadataDependencyV1 {
                        name: "next_process_allocation_counter".to_owned(),
                    }],
                },
                CargoMetadataPackageV1 {
                    name: "next_game".to_owned(),
                    dependencies: vec![CargoMetadataDependencyV1 {
                        name: "xtask".to_owned(),
                    }],
                },
            ],
        };

        assert_eq!(
            validate_metadata(&metadata),
            Err("UNSAFE_ALLOCATOR_COUNTER_TRANSITIVE_CONSUMER: next_game".to_owned())
        );
    }

    #[test]
    fn metadata_requires_a_std_only_counter_crate() {
        let metadata = CargoMetadataV1 {
            packages: vec![
                CargoMetadataPackageV1 {
                    name: "next_process_allocation_counter".to_owned(),
                    dependencies: vec![CargoMetadataDependencyV1 {
                        name: "unexpected_dependency".to_owned(),
                    }],
                },
                CargoMetadataPackageV1 {
                    name: "xtask".to_owned(),
                    dependencies: vec![CargoMetadataDependencyV1 {
                        name: "next_process_allocation_counter".to_owned(),
                    }],
                },
            ],
        };

        assert_eq!(
            validate_metadata(&metadata),
            Err("UNSAFE_ALLOCATOR_COUNTER_STD_ONLY_VIOLATION: unexpected_dependency".to_owned())
        );
    }

    #[test]
    fn adr_041_source_is_limited_to_the_owner_and_foreign_system_paths() {
        let source = concat!(
            "const SLOT_COUNT: usize = 4_096;\n",
            "#[repr(align(128))]\n",
            "struct AllocationSlot;\n",
            "struct AllocationThreadState {\n",
            "slot_index: Cell<u32>, owner_window_id: Cell<u64>,\n",
            "owner_alloc_count: Cell<u64>, owner_alloc_bytes: Cell<u64>,\n",
            "owner_alloc_zeroed_count: Cell<u64>, owner_alloc_zeroed_bytes: Cell<u64>,\n",
            "owner_realloc_count: Cell<u64>, owner_realloc_bytes: Cell<u64>,\n",
            "}\n",
            "impl AllocationThreadState {\n",
            "const fn new() -> Self { todo!() }\n",
            "fn reset_owner_counters(&self) {\n",
            "self.owner_alloc_count.set(0); self.owner_alloc_bytes.set(0);\n",
            "self.owner_alloc_zeroed_count.set(0); self.owner_alloc_zeroed_bytes.set(0);\n",
            "self.owner_realloc_count.set(0); self.owner_realloc_bytes.set(0);\n",
            "}\n",
            "}\n",
            "struct Measurement { same_thread: PhantomData<Rc<()>> }\n",
            "static SLOTS: [AllocationSlot; SLOT_COUNT] = [AllocationSlot; SLOT_COUNT];\n",
            "thread_local! { static THREAD_STATE: AllocationThreadState = const { AllocationThreadState::new() }; }\n",
            "const _: () = assert!(size_of::<AllocationSlot>() == 128);\n",
            "const _: () = assert!(align_of::<AllocationSlot>() == 128);\n",
            "const _: () = assert!(!needs_drop::<AllocationThreadState>());\n",
            "fn begin() { prepare_owner_thread(window_id); let active = encode_control(PHASE_ACTIVE, window_id); }\n",
            "fn prepare_owner_thread(window_id: u64) {\n",
            "if thread.slot_index.get() == UNCLAIMED_SLOT { claim_slot_slow(); }\n",
            "thread.owner_window_id.set(window_id); thread.reset_owner_counters();\n",
            "}\n",
            "fn clear_owner_window() { thread.owner_window_id.set(0); }\n",
            "fn admit_nonrecursive(sequence: &AtomicU64, observed: u64) {\n",
            "sequence.fetch_add(1, Ordering::SeqCst);\n",
            "let confirmed = STATE.control.load(Ordering::SeqCst);\n",
            "if confirmed != observed {}\n",
            "}\n",
            "fn handshake(sequence: &AtomicU64) { sequence.fetch_add(0, Ordering::SeqCst); }\n",
            "unsafe fn alloc_owner_active() { THREAD_STATE.try_with(|thread| {\n",
            "if thread.owner_window_id.get() == control_window(observed) { unsafe { self.system.alloc(layout) };\n",
            "} else { unsafe { self.alloc_foreign_active(layout, observed) }; } }); }\n",
            "unsafe fn alloc_foreign_active() { admit_nonrecursive(thread, observed); unsafe { self.system.alloc(layout) }; }\n",
            "unsafe fn alloc_zeroed_owner_active() { THREAD_STATE.try_with(|thread| {\n",
            "if thread.owner_window_id.get() == control_window(observed) { unsafe { self.system.alloc_zeroed(layout) };\n",
            "} else { unsafe { self.alloc_zeroed_foreign_active(layout, observed) }; } }); }\n",
            "unsafe fn alloc_zeroed_foreign_active() { admit_nonrecursive(thread, observed); unsafe { self.system.alloc_zeroed(layout) }; }\n",
            "unsafe fn realloc_owner_active() { THREAD_STATE.try_with(|thread| {\n",
            "if thread.owner_window_id.get() == control_window(observed) { unsafe { self.system.realloc(ptr, layout, size) };\n",
            "} else { unsafe { self.realloc_foreign_active(ptr, layout, size, observed) }; } }); }\n",
            "unsafe fn realloc_foreign_active() { admit_nonrecursive(thread, observed); unsafe { self.system.realloc(ptr, layout, size) }; }\n",
            "unsafe impl GlobalAlloc for ProcessAllocationCounter {\n",
            "unsafe fn alloc() { unsafe { self.alloc_owner_active(layout, observed) }; }\n",
            "unsafe fn alloc_zeroed() { unsafe { self.alloc_zeroed_owner_active(layout, observed) }; }\n",
            "unsafe fn realloc() { unsafe { self.realloc_owner_active(ptr, layout, size, observed) }; }\n",
            "unsafe fn dealloc() { unsafe { self.system.dealloc(ptr, layout) }; }\n",
            "}\n",
        );
        validate_library_source(source, source).expect("the exact allocator boundary must pass");

        let widened = source.replace(
            "unsafe { self.alloc_owner_active(layout, observed) }",
            "unsafe { arbitrary_raw_pointer_write() }",
        );
        assert!(
            validate_library_source(&widened, &widened)
                .expect_err("an unrelated unsafe block must be rejected")
                .starts_with("UNSAFE_ALLOCATOR_COUNTER_BLOCK_SCOPE_MISMATCH")
        );

        let allocating =
            source.replace("struct AllocationSlot;", "struct AllocationSlot(Vec<u8>);");
        assert_eq!(
            validate_library_source(&allocating, &allocating),
            Err("UNSAFE_ALLOCATOR_COUNTER_ADR041_FORBIDDEN_SOURCE: Vec<".to_owned())
        );

        let recursion_guard = source.replace(
            "slot_index: Cell<u32>, owner_window_id: Cell<u64>,",
            "slot_index: Cell<u32>, in_callback: Cell<bool>, owner_window_id: Cell<u64>,",
        );
        assert_eq!(
            validate_library_source(&recursion_guard, &recursion_guard),
            Err("UNSAFE_ALLOCATOR_COUNTER_ADR041_FORBIDDEN_SOURCE: in_callback".to_owned())
        );

        let owner_slot_access = source.replace(
            "unsafe fn alloc_owner_active() { THREAD_STATE.try_with(|thread| {",
            "unsafe fn alloc_owner_active() { let slot = &SLOTS[0]; THREAD_STATE.try_with(|thread| {",
        );
        assert_eq!(
            validate_library_source(&owner_slot_access, &owner_slot_access),
            Err(
                "UNSAFE_ALLOCATOR_COUNTER_ADR041_OWNER_SHARED_STATE_MISMATCH: alloc_owner_active"
                    .to_owned()
            )
        );

        let precheck = source.replace(
            "sequence.fetch_add(1, Ordering::SeqCst);\nlet confirmed = STATE.control.load(Ordering::SeqCst);",
            "let confirmed = STATE.control.load(Ordering::SeqCst);\nsequence.fetch_add(1, Ordering::SeqCst);",
        );
        assert_eq!(
            validate_library_source(&precheck, &precheck),
            Err("UNSAFE_ALLOCATOR_COUNTER_ADR041_FOREIGN_ADMISSION_MISMATCH".to_owned())
        );

        let legacy_dealloc_helpers = source.replace(
            "unsafe fn dealloc() { unsafe { self.system.dealloc(ptr, layout) }; }",
            concat!(
                "unsafe fn dealloc_owner_active() { unsafe { self.system.dealloc(ptr, layout) }; }\n",
                "unsafe fn dealloc_foreign_active() { unsafe { self.system.dealloc(ptr, layout) }; }\n",
                "unsafe fn dealloc() { unsafe { self.dealloc_owner_active(ptr, layout, observed) }; }",
            ),
        );
        assert_eq!(
            validate_library_source(&legacy_dealloc_helpers, &legacy_dealloc_helpers),
            Err(
                "UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_HELPER_FORBIDDEN: dealloc_owner_active"
                    .to_owned()
            )
        );

        let stateful_dealloc = source.replace(
            "unsafe fn dealloc() { unsafe { self.system.dealloc(ptr, layout) }; }",
            concat!(
                "unsafe fn dealloc() {\n",
                "let observed = observe_control();\n",
                "if phase(observed) == PHASE_ACTIVE { STATE.control.load(Ordering::SeqCst); }\n",
                "unsafe { self.system.dealloc(ptr, layout) };\n",
                "}",
            ),
        );
        assert_eq!(
            validate_library_source(&stateful_dealloc, &stateful_dealloc),
            Err("UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISMATCH".to_owned())
        );

        let masked_stateful_dealloc = format!(
            "fn dealloc() {{ unsafe {{ self.system.dealloc(ptr, layout) }}; }}\n{stateful_dealloc}"
        );
        assert_eq!(
            validate_library_source(&masked_stateful_dealloc, &masked_stateful_dealloc),
            Err("UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISMATCH".to_owned())
        );

        let block_comment_masked_dealloc = stateful_dealloc.replace(
            "unsafe impl GlobalAlloc for ProcessAllocationCounter {",
            concat!(
                "unsafe impl GlobalAlloc for ProcessAllocationCounter {\n",
                "/* fn dealloc() { unsafe { self.system.dealloc(ptr, layout) }; } */",
            ),
        );
        assert_eq!(
            validate_library_source(&block_comment_masked_dealloc, &block_comment_masked_dealloc),
            Err("UNSAFE_ALLOCATOR_COUNTER_ADR042_BLOCK_COMMENT_FORBIDDEN".to_owned())
        );

        let cfg_disabled_in_impl_decoy = stateful_dealloc.replace(
            "unsafe impl GlobalAlloc for ProcessAllocationCounter {",
            concat!(
                "unsafe impl GlobalAlloc for ProcessAllocationCounter {\n",
                "#[cfg(any())]\n",
                "fn dealloc() { unsafe { self.system.dealloc(ptr, layout) }; }\n",
            ),
        );
        assert_eq!(
            validate_library_source(&cfg_disabled_in_impl_decoy, &cfg_disabled_in_impl_decoy),
            Err("UNSAFE_ALLOCATOR_COUNTER_ADR042_DEALLOC_PASS_THROUGH_MISMATCH".to_owned())
        );
    }
}
