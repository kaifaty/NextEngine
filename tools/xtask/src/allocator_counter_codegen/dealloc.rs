use super::*;

const DEALLOC_SYMBOL: &str = "___rust_dealloc";
const DEALLOC_BACKEND_SYMBOL: &str = "HeapFree";
const PROCESS_HEAP_SYMBOL: &str = "get_process_heap";
const DEALLOC_HELPERS: [&str; 2] = ["dealloc_owner_active", "dealloc_foreign_active"];
const DEALLOC_MISMATCH: &str = "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH";

pub(super) fn validate(
    counter_ir: &str,
    counter_asm: &str,
    helper_ir: &str,
    helper_asm: &str,
) -> Result<(), AllocatorCounterCodegenError> {
    validate_helpers_absent([
        ("counter IR", counter_ir),
        ("counter ASM", counter_asm),
        ("helper IR", helper_ir),
        ("helper ASM", helper_asm),
    ])?;

    let function = unique_ir_function(helper_ir, DEALLOC_SYMBOL)?;
    validate_no_eh_cleanup(function, DEALLOC_SYMBOL)?;
    let forbidden_ir = [
        "STATE",
        "SLOTS",
        "NEXT_SLOT",
        "sequence",
        "threadlocal.address",
        "LocalKey",
        "next_process_allocation_counter",
        "PROCESS_ALLOCATOR",
        "observe_control",
        "observe_hook_slow",
        "record_admitted_fault",
        "record_preadmission_fault",
        "poison_exact",
        "FAULT",
        "fault",
        "__rust_alloc",
        "__rg_alloc",
        "exchange_malloc",
        "RawVec",
        "process_heap_alloc",
        "HeapAlloc",
        "HeapReAlloc",
        "realloc_fallback",
    ];
    let unexpected_ir = forbidden_ir
        .iter()
        .find(|needle| function.contains(**needle));
    let malformed_ir_call = function
        .lines()
        .find(|line| is_ir_call_instruction(line) && ir_call_target(line).is_none());
    let unexpected_ir_call = function.lines().find(|line| {
        ir_call_target(line).is_some_and(|target| {
            !is_heap_free_target(target)
                && !is_process_heap_target(target)
                && target != "llvm.assume"
        })
    });
    let heap_free_calls = ir_call_references(function, is_heap_free_target);
    let process_heap_calls = ir_call_references(function, is_process_heap_target);
    if function.lines().any(is_ir_atomic_instruction)
        || unexpected_ir.is_some()
        || malformed_ir_call.is_some()
        || unexpected_ir_call.is_some()
        || heap_free_calls != 1
        || process_heap_calls != 1
    {
        return Err(AllocatorCounterCodegenError::new(
            DEALLOC_MISMATCH,
            format!(
                "IR: forbidden={:?}, malformed call={malformed_ir_call:?}, unexpected call={unexpected_ir_call:?}, HeapFree calls={heap_free_calls}, process-heap calls={process_heap_calls}",
                unexpected_ir.copied(),
            ),
        ));
    }

    let assembly = unique_asm_function(helper_asm, DEALLOC_SYMBOL)?;
    validate_no_windows_eh(assembly, DEALLOC_SYMBOL)?;
    let forbidden_asm = [
        "STATE",
        "SLOTS",
        "NEXT_SLOT",
        "sequence",
        "_tls_index",
        "%gs:88",
        "@SECREL32",
        "LocalKey",
        "next_process_allocation_counter",
        "PROCESS_ALLOCATOR",
        "observe_control",
        "observe_hook_slow",
        "record_admitted_fault",
        "record_preadmission_fault",
        "poison_exact",
        "FAULT",
        "fault",
        "process_heap_alloc",
        "HeapAlloc",
        "HeapReAlloc",
        "realloc_fallback",
    ];
    let unexpected_asm = forbidden_asm
        .iter()
        .find(|needle| assembly.contains(**needle));
    let malformed_asm_call = assembly
        .lines()
        .find(|line| is_asm_call_instruction(line) && asm_transfer_target(line).is_none());
    let unexpected_asm_call = assembly.lines().find(|line| {
        is_asm_call_instruction(line)
            && asm_transfer_target(line).is_none_or(|target| {
                !is_heap_free_target(target) && !is_process_heap_target(target)
            })
    });
    let unexpected_external_jump = assembly.lines().find(|line| {
        is_asm_unconditional_jump(line)
            && asm_transfer_target(line)
                .is_none_or(|target| !is_heap_free_target(target) && !target.starts_with(".L"))
    });
    let unexpected_conditional_branch = assembly.lines().find(|line| {
        is_asm_conditional_branch(line)
            && asm_transfer_target(line).is_none_or(|target| !target.starts_with(".L"))
    });
    let heap_free_transfers = asm_transfer_references(assembly, is_heap_free_target);
    let process_heap_transfers = asm_transfer_references(assembly, is_process_heap_target);
    if assembly.lines().any(is_asm_atomic_instruction)
        || unexpected_asm.is_some()
        || malformed_asm_call.is_some()
        || unexpected_asm_call.is_some()
        || unexpected_external_jump.is_some()
        || unexpected_conditional_branch.is_some()
        || heap_free_transfers != 1
        || process_heap_transfers != 1
        || assembly.contains("__chkstk")
        || windows_stack_frame_bytes(assembly) > MAX_HOT_STACK_BYTES
    {
        return Err(AllocatorCounterCodegenError::new(
            DEALLOC_MISMATCH,
            format!(
                "ASM: forbidden={:?}, malformed call={malformed_asm_call:?}, unexpected call={unexpected_asm_call:?}, unexpected jump={unexpected_external_jump:?}, unexpected conditional branch={unexpected_conditional_branch:?}, HeapFree transfers={heap_free_transfers}, process-heap transfers={process_heap_transfers}",
                unexpected_asm.copied(),
            ),
        ));
    }
    Ok(())
}

fn validate_helpers_absent(
    artifacts: [(&str, &str); 4],
) -> Result<(), AllocatorCounterCodegenError> {
    for (artifact_name, artifact) in artifacts {
        if let Some(helper) = DEALLOC_HELPERS
            .iter()
            .find(|helper| artifact.contains(**helper))
        {
            return Err(AllocatorCounterCodegenError::new(
                DEALLOC_MISMATCH,
                format!("{artifact_name}: unexpected helper {helper}"),
            ));
        }
    }
    Ok(())
}

fn is_ir_atomic_instruction(line: &str) -> bool {
    [
        "load atomic",
        "store atomic",
        "atomicrmw",
        "cmpxchg",
        "fence ",
    ]
    .iter()
    .any(|needle| line.contains(needle))
}

fn is_ir_call_instruction(line: &str) -> bool {
    ir_call_suffix(line).is_some()
}

fn is_asm_atomic_instruction(line: &str) -> bool {
    let instruction = line.trim_start();
    is_locked_instruction(line)
        || instruction.starts_with("xchg")
        || instruction.starts_with("cmpxchg")
        || instruction.starts_with("mfence")
        || instruction.starts_with("sfence")
        || instruction.starts_with("lfence")
}

fn is_asm_call_instruction(line: &str) -> bool {
    asm_transfer(line).is_some_and(|(mnemonic, _)| mnemonic.starts_with("call"))
}

fn is_asm_unconditional_jump(line: &str) -> bool {
    asm_transfer(line).is_some_and(|(mnemonic, _)| mnemonic.starts_with("jmp"))
}

fn is_asm_conditional_branch(line: &str) -> bool {
    asm_transfer(line)
        .is_some_and(|(mnemonic, _)| mnemonic.starts_with('j') && !mnemonic.starts_with("jmp"))
}

fn ir_call_references(function: &str, matches_target: fn(&str) -> bool) -> usize {
    function
        .lines()
        .filter_map(ir_call_target)
        .filter(|target| matches_target(target))
        .count()
}

fn asm_transfer_references(function: &str, matches_target: fn(&str) -> bool) -> usize {
    function
        .lines()
        .filter_map(asm_transfer_target)
        .filter(|target| matches_target(target))
        .count()
}

fn ir_call_suffix(line: &str) -> Option<&str> {
    let code = line
        .split_once(';')
        .map_or(line, |(code, _)| code)
        .trim_start();
    if let Some(suffix) = code.strip_prefix("call ") {
        return Some(suffix);
    }
    let index = code.find(" call ")?;
    code.get(index + " call ".len()..)
}

fn ir_call_target(line: &str) -> Option<&str> {
    let suffix = ir_call_suffix(line)?;
    let target_start = suffix.find('@')? + 1;
    let target_and_arguments = suffix.get(target_start..)?;
    let target_end = target_and_arguments.find('(')?;
    let target = target_and_arguments[..target_end].trim();
    (!target.is_empty() && !target.chars().any(char::is_whitespace)).then_some(target)
}

fn asm_transfer(line: &str) -> Option<(&str, &str)> {
    let code = line.split_once('#').map_or(line, |(code, _)| code);
    let code = code
        .split_once(';')
        .map_or(code, |(code, _)| code)
        .trim_start();
    let mut fields = code.split_whitespace();
    let first = fields.next()?;
    let mnemonic = if first == "rex64" {
        fields.next()?
    } else {
        first
    };
    if !mnemonic.starts_with("call") && !mnemonic.starts_with('j') {
        return None;
    }
    Some((mnemonic, fields.next()?))
}

fn asm_transfer_target(line: &str) -> Option<&str> {
    let (_, operand) = asm_transfer(line)?;
    let target = operand.trim_start_matches('*');
    Some(target.strip_suffix("(%rip)").unwrap_or(target))
}

fn is_heap_free_target(target: &str) -> bool {
    target == DEALLOC_BACKEND_SYMBOL || target == "__imp_HeapFree"
}

fn is_process_heap_target(target: &str) -> bool {
    if target == PROCESS_HEAP_SYMBOL {
        return true;
    }
    const LEGACY_PREFIX: &str = "_ZN3std3sys5alloc7windows16get_process_heap17h";
    let Some(hash_and_end) = target.strip_prefix(LEGACY_PREFIX) else {
        return false;
    };
    let mut characters = hash_and_end.chars();
    (0..16).all(|_| {
        characters
            .next()
            .is_some_and(|character| character.is_ascii_hexdigit())
    }) && characters.next() == Some('E')
        && characters.next().is_none()
}
