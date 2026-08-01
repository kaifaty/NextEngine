//! Pinned release-codegen validation for the tooling-only allocation counter.
//!
//! ADR-041 and ADR-042 make the Windows x86_64 Rust 1.93 machine-code shape part
//! of the counter's availability contract. This module builds isolated release
//! IR and assembly artifacts and separately proves the inactive, owner-thread,
//! foreign-thread, and unconditional deallocation paths. It rejects TLS runtime
//! work, allocator recursion, unexpected locked atomics, or unwind-cleanup
//! machinery.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use self::symbols::semantic_symbol_match;
use self::tls::validate_const_tls_definition;

mod dealloc;
mod symbols;
mod tls;

const EXPECTED_RUST_RELEASE: &str = "1.93.0";
const EXPECTED_RUST_COMMIT: &str = "254b59607d4417e9dffbc307138ae5c86280fe4c";
const WINDOWS_TARGET: &str = "x86_64-pc-windows-msvc";
const CODEGEN_BUILD_JOBS: &str = "1";
const JOBSERVER_ENVIRONMENT: [&str; 3] = ["CARGO_MAKEFLAGS", "MAKEFLAGS", "MFLAGS"];
const MAX_COMMAND_OUTPUT_BYTES: usize = 256 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 32 * 1024 * 1024;
const MAX_JOINED_ARTIFACT_BYTES: usize = 64 * 1024 * 1024;
const MAX_HOT_STACK_BYTES: u64 = 512;
const SLOT_COUNT: u64 = 4_096;
const SLOT_BYTES: u64 = 128;
const SLOT_STORAGE_BYTES: u64 = SLOT_COUNT * SLOT_BYTES;

const ACTIVE_HELPERS: [(&str, &str, &str); 3] = [
    (
        "alloc_owner_active",
        "alloc_foreign_active",
        "process_heap_alloc",
    ),
    (
        "alloc_zeroed_owner_active",
        "alloc_zeroed_foreign_active",
        "process_heap_alloc",
    ),
    (
        "realloc_owner_active",
        "realloc_foreign_active",
        "HeapReAlloc",
    ),
];

static NEXT_SCRATCH_ID: AtomicU64 = AtomicU64::new(0);

/// Stable failure returned to `allocator-counter-check`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocatorCounterCodegenError {
    code: &'static str,
    detail: String,
}

impl AllocatorCounterCodegenError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    /// Stable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Bounded human-readable diagnostic detail.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Validates the actual pinned Windows release shape before timing starts.
pub fn validate_pinned_windows_release(root: &Path) -> Result<(), AllocatorCounterCodegenError> {
    validate_host_toolchain()?;
    let scratch = CodegenScratch::create(root)?;
    let validation = build_and_validate(root, scratch.path());
    match validation {
        Ok(()) => scratch.finish(),
        Err(error) => Err(AllocatorCounterCodegenError::new(
            error.code(),
            format!(
                "{}; scratch preserved: {}",
                error.detail(),
                scratch.path().display()
            ),
        )),
    }
}

fn validate_host_toolchain() -> Result<(), AllocatorCounterCodegenError> {
    if !cfg!(all(
        target_arch = "x86_64",
        target_os = "windows",
        target_env = "msvc"
    )) {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_TARGET_UNSUPPORTED",
            format!("expected {WINDOWS_TARGET}"),
        ));
    }
    let output = Command::new("rustc").arg("-Vv").output().map_err(|error| {
        AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_TOOLCHAIN_UNAVAILABLE",
            error.to_string(),
        )
    })?;
    if !output.status.success() {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_TOOLCHAIN_UNAVAILABLE",
            bounded_lossy(&output.stderr),
        ));
    }
    let version = String::from_utf8(output.stdout).map_err(|error| {
        AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_TOOLCHAIN_INVALID",
            error.to_string(),
        )
    })?;
    let required = [
        format!("release: {EXPECTED_RUST_RELEASE}"),
        format!("commit-hash: {EXPECTED_RUST_COMMIT}"),
        format!("host: {WINDOWS_TARGET}"),
    ];
    if required
        .iter()
        .any(|line| !version.lines().any(|value| value == line))
    {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_TOOLCHAIN_MISMATCH",
            version.trim(),
        ));
    }
    Ok(())
}

fn build_and_validate(root: &Path, target_dir: &Path) -> Result<(), AllocatorCounterCodegenError> {
    emit_release_artifacts(
        root,
        target_dir,
        &["-p", "next_process_allocation_counter", "--lib"],
    )?;
    emit_release_artifacts(
        root,
        target_dir,
        &[
            "-p",
            "next_process_allocation_counter",
            "--bin",
            "allocator_counter_codegen_probe",
        ],
    )?;
    let deps = target_dir.join(WINDOWS_TARGET).join("release").join("deps");
    let counter_ir = read_artifacts(&deps, "next_process_allocation_counter", "ll")?;
    let counter_asm = read_artifacts(&deps, "next_process_allocation_counter", "s")?;
    let helper_ir = read_artifacts(&deps, "allocator_counter_codegen_probe", "ll")?;
    let helper_asm = read_artifacts(&deps, "allocator_counter_codegen_probe", "s")?;
    validate_artifacts(&counter_ir, &counter_asm, &helper_ir, &helper_asm)
}

fn emit_release_artifacts(
    root: &Path,
    target_dir: &Path,
    target_arguments: &[&str],
) -> Result<(), AllocatorCounterCodegenError> {
    let mut command = release_artifact_command(root, target_dir, target_arguments);
    let output = command.output().map_err(|error| {
        AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_BUILD_FAILED",
            error.to_string(),
        )
    })?;
    if output.stdout.len().saturating_add(output.stderr.len()) > MAX_COMMAND_OUTPUT_BYTES {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_BUILD_OUTPUT_EXCEEDED",
            format!(
                "stdout={}, stderr={}",
                output.stdout.len(),
                output.stderr.len()
            ),
        ));
    }
    if !output.status.success() {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_BUILD_FAILED",
            format!(
                "status={}, stdout={:?}, stderr={:?}",
                output.status,
                bounded_lossy(&output.stdout),
                bounded_lossy(&output.stderr)
            ),
        ));
    }
    Ok(())
}

fn release_artifact_command(root: &Path, target_dir: &Path, target_arguments: &[&str]) -> Command {
    let mut command = Command::new("cargo");
    command
        .args([
            "rustc",
            "--locked",
            "--release",
            "--target",
            WINDOWS_TARGET,
            "--target-dir",
        ])
        .arg(target_dir)
        .args(["--jobs", CODEGEN_BUILD_JOBS])
        .args(target_arguments)
        .args(["--", "--emit=asm,llvm-ir", "-Cpanic=unwind"])
        .current_dir(root);
    for variable in JOBSERVER_ENVIRONMENT {
        command.env_remove(variable);
    }
    command
}

fn read_artifacts(
    deps: &Path,
    prefix: &str,
    extension: &str,
) -> Result<String, AllocatorCounterCodegenError> {
    let entries = fs::read_dir(deps).map_err(|error| {
        AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_ARTIFACT_MISSING",
            format!("{}: {error}", deps.display()),
        )
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_ARTIFACT_INVALID",
                error.to_string(),
            )
        })?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with(prefix)
            && path.extension().and_then(|value| value.to_str()) == Some(extension)
        {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_ARTIFACT_MISSING",
            format!("{prefix}*.{extension}"),
        ));
    }
    let mut joined = String::new();
    for path in paths {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_ARTIFACT_INVALID",
                format!("{}: {error}", path.display()),
            )
        })?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_ARTIFACT_BYTES
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_ARTIFACT_INVALID",
                path.display().to_string(),
            ));
        }
        let body = fs::read_to_string(&path).map_err(|error| {
            AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_ARTIFACT_INVALID",
                format!("{}: {error}", path.display()),
            )
        })?;
        if joined
            .len()
            .checked_add(body.len())
            .is_none_or(|size| size > MAX_JOINED_ARTIFACT_BYTES)
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_ARTIFACT_INVALID",
                "combined codegen artifacts exceed 64 MiB",
            ));
        }
        joined.push_str("\n; nextengine-artifact: ");
        joined.push_str(&path.display().to_string());
        joined.push('\n');
        joined.push_str(&body);
    }
    Ok(joined)
}

fn validate_artifacts(
    counter_ir: &str,
    counter_asm: &str,
    helper_ir: &str,
    helper_asm: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    validate_static_storage(counter_ir, counter_asm)?;
    validate_tls_shape(counter_ir, counter_asm)?;
    validate_outer_shims(helper_ir, helper_asm)?;
    dealloc::validate(counter_ir, counter_asm, helper_ir, helper_asm)?;
    validate_first_touch_claim(counter_ir, counter_asm)?;
    validate_owner_active_helpers(counter_ir, counter_asm)?;
    validate_foreign_active_helpers(counter_ir, counter_asm)
}

fn validate_static_storage(
    counter_ir: &str,
    counter_asm: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    let expected_extent = format!("[{SLOT_STORAGE_BYTES} x i8]");
    let slots_lines = counter_ir
        .lines()
        .filter(|line| line.contains("SLOTS") && line.contains("internal global"))
        .collect::<Vec<_>>();
    if slots_lines.len() != 1
        || !slots_lines[0].contains(&expected_extent)
        || !slots_lines[0].contains("zeroinitializer")
        || !slots_lines[0].contains("align 128")
    {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_STATIC_LAYOUT_MISMATCH",
            format!(
                "expected one {SLOT_STORAGE_BYTES}-byte zeroed slot array aligned to {SLOT_BYTES}"
            ),
        ));
    }
    let slot_type = counter_ir
        .lines()
        .find(|line| line.starts_with("%AllocationSlot = type "));
    if slot_type
        .is_none_or(|line| count_occurrences(line, "AtomicU64") != 7 || !line.contains("[9 x i64]"))
    {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_STATIC_LAYOUT_MISMATCH",
            "expected seven counters padded to one 128-byte slot",
        ));
    }
    let static_assembly_matches = counter_asm.match_indices("SLOTS").any(|(index, _)| {
        let start = index.saturating_sub(256);
        let end = index.saturating_add(768).min(counter_asm.len());
        counter_asm.get(start..end).is_some_and(|window| {
            window.contains(&format!(".zero\t{SLOT_STORAGE_BYTES}"))
                && (window.contains(".p2align\t7") || window.contains(".p2align 7"))
        })
    });
    if !static_assembly_matches {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_STATIC_LAYOUT_MISMATCH",
            "expected a 4096 x 128-byte static slot array",
        ));
    }
    Ok(())
}

fn validate_tls_shape(
    counter_ir: &str,
    counter_asm: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    let tls_lines = counter_ir
        .lines()
        .filter(|line| line.contains("thread_local") && line.contains("THREAD"))
        .collect::<Vec<_>>();
    if tls_lines.len() != 1 {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_TLS_LAYOUT_MISMATCH",
            format!("thread-local definitions={}", tls_lines.len()),
        ));
    }
    validate_const_tls_definition(tls_lines[0]).map_err(|detail| {
        AllocatorCounterCodegenError::new("ALLOCATOR_COUNTER_CODEGEN_TLS_LAYOUT_MISMATCH", detail)
    })?;
    let forbidden = [
        "LazyStorage",
        "EagerStorage",
        "register_dtor",
        "__cxa_thread_atexit",
        "_tlregdtor",
        "TlsAlloc",
        "TlsGetValue",
        "FlsAlloc",
        "FlsGetValue",
        "InitOnceExecuteOnce",
        "AcquireSRWLock",
        "EnterCriticalSection",
        "__tls_get_addr",
        "__rust_alloc",
        "exchange_malloc",
        "RawVec",
    ];
    if let Some(found) = forbidden
        .iter()
        .find(|needle| counter_ir.contains(**needle) || counter_asm.contains(**needle))
    {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_TLS_RUNTIME_FORBIDDEN",
            *found,
        ));
    }
    if !(counter_asm.contains(".section\t.tls$")
        && counter_asm.contains("_tls_index")
        && counter_asm.contains("%gs:88")
        && counter_asm.contains("@SECREL32"))
    {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_TLS_DIRECT_ACCESS_MISSING",
            "expected native Windows TEB TLS access",
        ));
    }
    Ok(())
}

fn validate_outer_shims(
    helper_ir: &str,
    helper_asm: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    for (symbol, active_symbol, system_symbol) in [
        ("___rust_alloc", "alloc_owner_active", "process_heap_alloc"),
        (
            "___rust_alloc_zeroed",
            "alloc_zeroed_owner_active",
            "process_heap_alloc",
        ),
        ("___rust_realloc", "realloc_owner_active", "HeapReAlloc"),
    ] {
        let function = unique_ir_function(helper_ir, symbol)?;
        validate_no_eh(function, symbol)?;
        let atomic_loads = function
            .lines()
            .filter(|line| line.contains("load atomic"))
            .collect::<Vec<_>>();
        if atomic_loads.len() != 1
            || !atomic_loads[0].contains("STATE")
            || !atomic_loads[0].contains("seq_cst")
            || function.contains("atomicrmw")
            || function.contains("cmpxchg")
            || function.contains("threadlocal.address")
            || function.contains("THREAD_STATE")
            || !function.contains("observe_hook_slow")
            || !function.contains(active_symbol)
            || !function.contains(system_symbol)
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_INACTIVE_PATH_MISMATCH",
                symbol,
            ));
        }
        let assembly = unique_asm_function(helper_asm, symbol)?;
        validate_no_windows_eh(assembly, symbol)?;
        let state_references = assembly
            .lines()
            .filter(|line| line.contains("STATE") && line.contains("(%rip)"))
            .count();
        if state_references != 1
            || assembly.lines().any(is_locked_instruction)
            || ["_tls_index", "%gs:88", "@SECREL32", "THREAD_STATE"]
                .iter()
                .any(|needle| assembly.contains(needle))
            || !assembly.contains(system_symbol)
            || assembly.contains("__chkstk")
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_INACTIVE_PATH_MISMATCH",
                format!("{symbol}: native state references={state_references}"),
            ));
        }
    }
    Ok(())
}

fn validate_first_touch_claim(
    counter_ir: &str,
    counter_asm: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    const SYMBOL: &str = "claim_slot_slow";
    let function = unique_ir_function(counter_ir, SYMBOL)?;
    validate_no_eh(function, SYMBOL)?;
    validate_nounwind(function, counter_ir, SYMBOL)?;
    let compare_exchanges = function
        .lines()
        .filter(|line| line.contains("cmpxchg"))
        .collect::<Vec<_>>();
    if compare_exchanges.len() != 1
        || !compare_exchanges[0].contains("NEXT_SLOT")
        || !compare_exchanges[0].contains("monotonic monotonic")
        || function.contains("atomicrmw")
        || function.contains("threadlocal.address")
        || function.contains("THREAD_STATE")
        || [
            "__rust_alloc",
            "exchange_malloc",
            "RawVec",
            "process_heap_alloc",
            "HeapAlloc",
        ]
        .iter()
        .any(|needle| function.contains(needle))
    {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_FIRST_TOUCH_MISMATCH",
            SYMBOL,
        ));
    }
    let assembly = unique_asm_function(counter_asm, SYMBOL)?;
    validate_no_windows_eh(assembly, SYMBOL)?;
    let locked_lines = assembly
        .lines()
        .filter(|line| is_locked_instruction(line))
        .collect::<Vec<_>>();
    if locked_lines.len() != 1
        || !locked_lines[0].contains("NEXT_SLOT")
        || assembly.contains("__chkstk")
        || assembly
            .lines()
            .any(|line| line.trim_start().starts_with("call"))
    {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_FIRST_TOUCH_MISMATCH",
            format!("native locked instructions={}", locked_lines.len()),
        ));
    }
    Ok(())
}

fn validate_owner_active_helpers(
    counter_ir: &str,
    counter_asm: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    for (owner_symbol, foreign_symbol, system_symbol) in ACTIVE_HELPERS {
        let function = unique_ir_function(counter_ir, owner_symbol)?;
        // Rust 1.93 keeps passive personality metadata on these Rust-ABI
        // functions even though the optimized body has no EH control flow.
        // ADR-041 forbids real EH control flow, not passive metadata.
        validate_no_eh_cleanup(function, owner_symbol)?;
        if function.contains("atomicrmw")
            || function.contains("cmpxchg")
            || function.contains("load atomic")
            || function.lines().any(is_control_state_reference)
            || function.contains("SLOTS")
            || function.contains("NEXT_SLOT")
            || function.contains("sequence")
            || !function.contains("threadlocal.address")
            || !function.contains("THREAD_STATE")
            || !function.contains(system_symbol)
            || !function.contains(foreign_symbol)
            || [
                "__rust_alloc",
                "__rg_alloc",
                "exchange_malloc",
                "RawVec",
                "thread::current",
                "LocalKey",
                "system_alloc_once",
                "system_alloc_zeroed_once",
                "system_realloc_once",
                "system_dealloc_once",
            ]
            .iter()
            .any(|needle| function.contains(needle))
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_OWNER_PATH_MISMATCH",
                owner_symbol,
            ));
        }
        let assembly = unique_asm_function(counter_asm, owner_symbol)?;
        validate_no_windows_eh(assembly, owner_symbol)?;
        if assembly.lines().any(is_locked_instruction)
            || assembly.lines().any(is_control_state_reference)
            || ["NEXT_SLOT", "SLOTS", "sequence"]
                .iter()
                .any(|needle| assembly.contains(needle))
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_OWNER_PATH_MISMATCH",
                owner_symbol,
            ));
        }
        if !assembly.contains("_tls_index")
            || !assembly.contains("%gs:88")
            || !assembly.contains("@SECREL32")
            || !assembly.contains(system_symbol)
            || !assembly.contains(foreign_symbol)
            || assembly.contains("__chkstk")
            || windows_stack_frame_bytes(assembly) > MAX_HOT_STACK_BYTES
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_STACK_BOUND_EXCEEDED",
                owner_symbol,
            ));
        }
    }
    Ok(())
}

fn validate_foreign_active_helpers(
    counter_ir: &str,
    counter_asm: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    for (_, foreign_symbol, system_symbol) in ACTIVE_HELPERS {
        let function = unique_ir_function(counter_ir, foreign_symbol)?;
        validate_no_eh_cleanup(function, foreign_symbol)?;
        let lines = function.lines().collect::<Vec<_>>();
        let read_modify_writes = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains("atomicrmw") || line.contains("cmpxchg"))
            .collect::<Vec<_>>();
        let control_loads = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains("load atomic") && is_control_state_reference(line))
            .collect::<Vec<_>>();
        let has_identity_comparison = control_loads.first().is_some_and(|(load_index, load)| {
            let loaded_value = load
                .split_once('=')
                .map(|(value, _)| value.trim())
                .filter(|value| value.starts_with('%'));
            loaded_value.is_some_and(|loaded_value| {
                lines.iter().skip(*load_index + 1).any(|line| {
                    (line.contains("icmp eq i64") || line.contains("icmp ne i64"))
                        && line.contains(loaded_value)
                })
            })
        });
        if read_modify_writes.len() != 1
            || !read_modify_writes[0].1.contains("atomicrmw add")
            || !read_modify_writes[0].1.contains("seq_cst")
            || control_loads.len() != 1
            || !control_loads[0].1.contains("seq_cst")
            || control_loads[0].0 <= read_modify_writes[0].0
            || !has_identity_comparison
            || !function.contains("SLOTS")
            || !function.contains(system_symbol)
            || function.contains("cmpxchg")
            || [
                "__rust_alloc",
                "__rg_alloc",
                "exchange_malloc",
                "RawVec",
                "thread::current",
                "LocalKey",
                "system_alloc_once",
                "system_alloc_zeroed_once",
                "system_realloc_once",
                "system_dealloc_once",
            ]
            .iter()
            .any(|needle| function.contains(needle))
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_FOREIGN_PATH_MISMATCH",
                foreign_symbol,
            ));
        }

        let assembly = unique_asm_function(counter_asm, foreign_symbol)?;
        validate_no_windows_eh(assembly, foreign_symbol)?;
        let assembly_lines = assembly.lines().collect::<Vec<_>>();
        let locked_lines = assembly_lines
            .iter()
            .enumerate()
            .filter(|(_, line)| is_locked_instruction(line))
            .collect::<Vec<_>>();
        let state_references = assembly_lines
            .iter()
            .enumerate()
            .filter(|(_, line)| is_control_state_reference(line) && line.contains("(%rip)"))
            .collect::<Vec<_>>();
        if locked_lines.len() != 1
            || locked_lines[0].1.contains("%rip")
            || ["STATE", "NEXT_SLOT", "COUNTER"]
                .iter()
                .any(|needle| locked_lines[0].1.contains(needle))
            || state_references.len() != 1
            || state_references[0].0 <= locked_lines[0].0
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_FOREIGN_PATH_MISMATCH",
                format!(
                    "{foreign_symbol}: locked={}, state-postchecks={}",
                    locked_lines.len(),
                    state_references.len()
                ),
            ));
        }
        if !assembly.contains(system_symbol)
            || assembly.contains("__chkstk")
            || windows_stack_frame_bytes(assembly) > MAX_HOT_STACK_BYTES
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_STACK_BOUND_EXCEEDED",
                foreign_symbol,
            ));
        }
    }
    Ok(())
}

fn unique_ir_function<'a>(
    module: &'a str,
    symbol: &str,
) -> Result<&'a str, AllocatorCounterCodegenError> {
    let functions = ir_functions(module)
        .into_iter()
        .filter(|function| {
            function
                .lines()
                .next()
                .is_some_and(|line| semantic_symbol_match(line, symbol))
        })
        .collect::<Vec<_>>();
    if functions.len() != 1 {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_SYMBOL_MISMATCH",
            format!("IR {symbol}: matches={}", functions.len()),
        ));
    }
    Ok(functions[0])
}

fn unique_asm_function<'a>(
    module: &'a str,
    symbol: &str,
) -> Result<&'a str, AllocatorCounterCodegenError> {
    let functions = asm_functions(module)
        .into_iter()
        .filter(|function| {
            function
                .lines()
                .next()
                .is_some_and(|line| semantic_symbol_match(line, symbol))
        })
        .collect::<Vec<_>>();
    if functions.len() != 1 {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_SYMBOL_MISMATCH",
            format!("ASM {symbol}: matches={}", functions.len()),
        ));
    }
    Ok(functions[0])
}

fn ir_functions(module: &str) -> Vec<&str> {
    let mut functions = Vec::new();
    let mut offset = 0;
    while let Some(relative_start) = module[offset..].find("define ") {
        let start = offset + relative_start;
        let line_start = module[..start].rfind('\n').map_or(0, |value| value + 1);
        if !module[line_start..start].trim().is_empty() {
            offset = start + "define ".len();
            continue;
        }
        let Some(relative_end) = module[start..].find("\n}") else {
            break;
        };
        let end = start + relative_end + 2;
        functions.push(&module[start..end]);
        offset = end;
    }
    functions
}

fn asm_functions(module: &str) -> Vec<&str> {
    let mut starts = Vec::new();
    let mut offset = 0;
    for line in module.split_inclusive('\n') {
        let candidate = line.trim_end_matches(&['\r', '\n'][..]);
        if !candidate.starts_with(' ')
            && !candidate.starts_with('\t')
            && candidate.ends_with(':')
            && !candidate.starts_with(".L")
        {
            starts.push(offset);
        }
        offset += line.len();
    }
    starts
        .iter()
        .enumerate()
        .map(|(index, start)| {
            let end = starts.get(index + 1).copied().unwrap_or(module.len());
            &module[*start..end]
        })
        .collect()
}

fn validate_no_eh(function: &str, symbol: &str) -> Result<(), AllocatorCounterCodegenError> {
    validate_no_eh_cleanup(function, symbol)?;
    if function.contains(" personality ") {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_UNWIND_PATH_PRESENT",
            symbol,
        ));
    }
    Ok(())
}

fn validate_no_eh_cleanup(
    function: &str,
    symbol: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    if [
        "invoke ",
        "landingpad",
        "resume ",
        "cleanuppad",
        "cleanupret",
        "catchswitch",
        "catchpad",
        "catchret",
        "panic_cannot_unwind",
        "panic_in_cleanup",
        "panic_bounds_check",
        "panic_fmt",
        "drop_in_place",
        "terminate",
        "unwind label",
        "unwind to caller",
    ]
    .iter()
    .any(|needle| function.contains(needle))
    {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_UNWIND_PATH_PRESENT",
            symbol,
        ));
    }
    Ok(())
}

fn validate_nounwind(
    function: &str,
    module: &str,
    symbol: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    if !function_has_attribute(function, module, "nounwind") {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_NOUNWIND_MISSING",
            symbol,
        ));
    }
    Ok(())
}

fn function_has_attribute(function: &str, module: &str, attribute: &str) -> bool {
    let Some(definition) = function.lines().next() else {
        return false;
    };
    if definition.contains(attribute) {
        return true;
    }
    let Some(group_start) = definition.rfind(" #") else {
        return false;
    };
    let group = definition[group_start + 2..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    !group.is_empty()
        && module.lines().any(|line| {
            line.starts_with(&format!("attributes #{group} =")) && line.contains(attribute)
        })
}

fn validate_no_windows_eh(
    function: &str,
    symbol: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    let forbidden = [
        ".seh_handler",
        ".seh_handlerdata",
        "__CxxFrameHandler3",
        "?dtor$",
        "$cppxdata",
        "$stateUnwindMap",
        "panic_bounds_check",
        "panic_cannot_unwind",
        "panicking",
        "terminate",
    ];
    if let Some(found) = forbidden.iter().find(|needle| function.contains(**needle)) {
        return Err(AllocatorCounterCodegenError::new(
            "ALLOCATOR_COUNTER_CODEGEN_EH_CLEANUP_PRESENT",
            format!("{symbol}: {found}"),
        ));
    }
    Ok(())
}

fn maximum_stack_allocation(function: &str) -> u64 {
    function
        .lines()
        .filter_map(|line| line.trim().strip_prefix(".seh_stackalloc"))
        .filter_map(|value| value.trim().parse::<u64>().ok())
        .max()
        .unwrap_or(0)
}

fn windows_stack_frame_bytes(function: &str) -> u64 {
    let pushed_register_bytes = function
        .lines()
        .filter(|line| line.trim_start().starts_with(".seh_pushreg"))
        .count() as u64
        * 8;
    maximum_stack_allocation(function).saturating_add(pushed_register_bytes)
}

fn is_locked_instruction(line: &str) -> bool {
    line.trim_start().starts_with("lock")
}

fn is_control_state_reference(line: &str) -> bool {
    line.replace("THREAD_STATE", "").contains("STATE")
}

fn count_occurrences(body: &str, needle: &str) -> usize {
    body.match_indices(needle).count()
}

fn bounded_lossy(bytes: &[u8]) -> String {
    let end = bytes.len().min(MAX_COMMAND_OUTPUT_BYTES);
    String::from_utf8_lossy(&bytes[..end]).trim().to_owned()
}

struct CodegenScratch {
    path: PathBuf,
    root_target: PathBuf,
}

impl CodegenScratch {
    fn create(root: &Path) -> Result<Self, AllocatorCounterCodegenError> {
        let root_target = root.join("target");
        fs::create_dir_all(&root_target).map_err(|error| {
            AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_SCRATCH_FAILED",
                error.to_string(),
            )
        })?;
        let sequence = NEXT_SCRATCH_ID.fetch_add(1, Ordering::Relaxed);
        let path = root_target.join(format!(
            "allocator-counter-codegen-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).map_err(|error| {
            AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_SCRATCH_FAILED",
                format!("{}: {error}", path.display()),
            )
        })?;
        Ok(Self { path, root_target })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn finish(self) -> Result<(), AllocatorCounterCodegenError> {
        if self.path.parent() != Some(self.root_target.as_path())
            || !self
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("allocator-counter-codegen-"))
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_SCRATCH_INVALID",
                self.path.display().to_string(),
            ));
        }
        fs::remove_dir_all(&self.path).map_err(|error| {
            AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_CLEANUP_FAILED",
                format!("{}: {error}", self.path.display()),
            )
        })
    }
}

#[cfg(test)]
mod tests;
