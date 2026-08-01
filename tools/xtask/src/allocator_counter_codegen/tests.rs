use super::*;

const ATTRIBUTES: &str = "attributes #0 = { nounwind }\n";

fn ir_function(name: &str, body: &str) -> String {
    format!("define void @{name}() #0 {{\n{body}\n}}\n")
}

fn ir_function_with_personality(name: &str, body: &str) -> String {
    format!("define void @{name}() #0 personality ptr @__CxxFrameHandler3 {{\n{body}\n}}\n")
}

fn asm_function(name: &str, body: &str) -> String {
    format!("{name}:\n.seh_proc {name}\n{body}\n.seh_endproc\n")
}

fn leaf_asm_function(name: &str, body: &str) -> String {
    format!("{name}:\n{body}\n")
}

#[test]
fn release_artifact_build_is_locked_serial_and_exact() {
    let root = Path::new("repo");
    let target_dir = Path::new("scratch");
    let command = release_artifact_command(
        root,
        target_dir,
        &["-p", "next_process_allocation_counter", "--lib"],
    );
    assert_eq!(command.get_program(), "cargo");
    assert_eq!(command.get_current_dir(), Some(root));
    let arguments = command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        arguments,
        [
            "rustc",
            "--locked",
            "--release",
            "--target",
            WINDOWS_TARGET,
            "--target-dir",
            "scratch",
            "--jobs",
            "1",
            "-p",
            "next_process_allocation_counter",
            "--lib",
            "--",
            "--emit=asm,llvm-ir",
            "-Cpanic=unwind",
        ]
    );
    assert_eq!(
        arguments
            .iter()
            .filter(|argument| argument.as_str() == "--jobs")
            .count(),
        1
    );
    let removed_environment = command
        .get_envs()
        .filter(|(_, value)| value.is_none())
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(removed_environment, JOBSERVER_ENVIRONMENT);

    let probe = release_artifact_command(
        root,
        target_dir,
        &[
            "-p",
            "next_process_allocation_counter",
            "--bin",
            "allocator_counter_codegen_probe",
        ],
    );
    let probe_arguments = probe
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        &probe_arguments[9..14],
        [
            "-p",
            "next_process_allocation_counter",
            "--bin",
            "allocator_counter_codegen_probe",
            "--",
        ]
    );
    assert!(!probe_arguments.iter().any(|argument| argument == "xtask"));
}

#[test]
fn parser_finds_seh_and_leaf_windows_assembly_functions() {
    let ir = format!(
        "{}{}",
        ir_function("alloc_owner_active", "  ret void"),
        ATTRIBUTES
    );
    let assembly = format!(
        "{}{}",
        asm_function("alloc_owner_active", "\tretq"),
        leaf_asm_function("claim_slot_slow", "\tretq")
    );
    assert_eq!(
        unique_ir_function(&ir, "alloc_owner_active")
            .unwrap()
            .lines()
            .count(),
        3
    );
    assert!(
        unique_asm_function(&assembly, "alloc_owner_active")
            .unwrap()
            .contains(".seh_proc")
    );
    assert!(
        unique_asm_function(&assembly, "claim_slot_slow")
            .unwrap()
            .contains("retq")
    );
}

#[test]
fn semantic_symbol_parser_accepts_exact_legacy_rust_components_only() {
    let legacy = "define void @_ZN4next7counter18alloc_owner_active17h0123456789abcdefE() #0 {";
    assert!(semantic_symbol_match(legacy, "alloc_owner_active"));
    assert!(semantic_symbol_match(
        "define void @alloc_owner_active() #0 {",
        "alloc_owner_active"
    ));
    assert!(!semantic_symbol_match(
        "define void @side_alloc_owner_active() #0 {",
        "alloc_owner_active"
    ));
    assert!(!semantic_symbol_match(
        "define void @_ZN4next7counter18alloc_owner_active17h0123456789abcdegE() #0 {",
        "alloc_owner_active"
    ));
    assert!(!semantic_symbol_match(
        "define void @_ZN4next7counter19alloc_owner_active217h0123456789abcdefE() #0 {",
        "alloc_owner_active"
    ));

    let actual_v0 = "define internal void @_RNvCshXwFllX56pT_7___rustc14___rust_dealloc() #0 {";
    assert!(semantic_symbol_match(actual_v0, "___rust_dealloc"));
    assert!(!semantic_symbol_match(
        "define internal void @_RNvCshXwFllX56pT_7___rustc15___rust_dealloc() #0 {",
        "___rust_dealloc"
    ));
}

#[test]
fn eh_parser_distinguishes_passive_personality_from_real_cleanup() {
    let passive = format!(
        "{}{}",
        ir_function_with_personality("hot", "  ret void"),
        ATTRIBUTES
    );
    let passive_function = unique_ir_function(&passive, "hot").unwrap();
    validate_no_eh_cleanup(passive_function, "hot").unwrap();
    assert_eq!(
        validate_no_eh(passive_function, "hot").unwrap_err().code(),
        "ALLOCATOR_COUNTER_CODEGEN_UNWIND_PATH_PRESENT"
    );
    validate_nounwind(passive_function, &passive, "hot").unwrap();

    let abort_cleanup = ir_function_with_personality(
        "hot",
        "  invoke void @system() to label %ok unwind label %cs_terminate\ncs_terminate:\n  %catch = catchswitch within none [label %abort] unwind to caller",
    );
    assert_eq!(
        validate_no_eh_cleanup(&abort_cleanup, "hot")
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_UNWIND_PATH_PRESENT"
    );
}

#[test]
fn nounwind_parser_rejects_a_first_touch_function_without_attribute() {
    let missing = "define void @claim_slot_slow() {\n  ret void\n}\n";
    assert_eq!(
        validate_nounwind(missing, missing, "claim_slot_slow")
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_NOUNWIND_MISSING"
    );
}

#[test]
fn windows_eh_parser_allows_plain_seh_but_rejects_cleanup_handlers() {
    let plain = asm_function("hot", "\t.seh_pushreg\t%rbx\n\t.seh_stackalloc\t64\n\tretq");
    validate_no_windows_eh(&plain, "hot").unwrap();
    assert_eq!(windows_stack_frame_bytes(&plain), 72);

    let cleanup = asm_function("hot", "\t.seh_handler __CxxFrameHandler3, @unwind, @except");
    assert_eq!(
        validate_no_windows_eh(&cleanup, "hot").unwrap_err().code(),
        "ALLOCATOR_COUNTER_CODEGEN_EH_CLEANUP_PRESENT"
    );
}

#[test]
fn outer_shim_parser_requires_one_load_direct_system_and_no_tls_or_rmw() {
    let mut valid_ir = String::new();
    let mut valid_asm = String::new();
    for (symbol, active_symbol, system_symbol) in [
        ("___rust_alloc", "alloc_owner_active", "process_heap_alloc"),
        (
            "___rust_alloc_zeroed",
            "alloc_zeroed_owner_active",
            "process_heap_alloc",
        ),
        ("___rust_realloc", "realloc_owner_active", "HeapReAlloc"),
    ] {
        valid_ir.push_str(&ir_function(
            symbol,
            &format!(
                "  %control = load atomic i64, ptr @STATE seq_cst, align 8\n  %hook = tail call i64 @observe_hook_slow(i64 %control)\n  call void @{active_symbol}()\n  call void @{system_symbol}()\n  ret void"
            ),
        ));
        valid_asm.push_str(&asm_function(
            symbol,
            &format!(
                "\tmovq\t__imp_STATE(%rip), %rax\n\tcallq\tobserve_hook_slow\n\tcallq\t{active_symbol}\n\tcallq\t{system_symbol}\n\tretq"
            ),
        ));
    }
    valid_ir.push_str(ATTRIBUTES);
    validate_outer_shims(&valid_ir, &valid_asm).unwrap();

    let invalid = valid_ir.replacen(
        "  ret void",
        "  %old = atomicrmw add ptr @STATE, i64 1 seq_cst\n  ret void",
        1,
    );
    assert_eq!(
        validate_outer_shims(&invalid, &valid_asm)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_INACTIVE_PATH_MISMATCH"
    );

    for forbidden in [
        "  %second = load atomic i64, ptr @STATE seq_cst, align 8\n",
        "  %tls = tail call ptr @llvm.threadlocal.address.p0(ptr @THREAD_STATE)\n",
    ] {
        let invalid = valid_ir.replacen("  ret void", &format!("{forbidden}  ret void"), 1);
        assert_eq!(
            validate_outer_shims(&invalid, &valid_asm)
                .unwrap_err()
                .code(),
            "ALLOCATOR_COUNTER_CODEGEN_INACTIVE_PATH_MISMATCH"
        );
    }
}

fn direct_dealloc_fixtures() -> (String, String) {
    let ir = format!(
        "{}{}",
        ir_function(
            "___rust_dealloc",
            "  ; call std::sys::alloc::windows::get_process_heap\n  tail call void @llvm.assume(i1 true)\n  %heap = tail call ptr @get_process_heap()\n  %freed = tail call i32 @HeapFree(ptr %heap, i32 0, ptr %allocation)\n  ret void",
        ),
        ATTRIBUTES
    );
    let assembly = asm_function(
        "___rust_dealloc",
        "\tcallq\tget_process_heap\n\trex64 jmpq\t*__imp_HeapFree(%rip)",
    );
    (ir, assembly)
}

#[test]
fn dealloc_parser_requires_one_direct_backend_call_without_measurement_work() {
    let (ir, assembly) = direct_dealloc_fixtures();
    dealloc::validate("", "", &ir, &assembly).unwrap();

    let actual_v0_symbol = "_RNvCshXwFllX56pT_7___rustc14___rust_dealloc";
    let actual_v0_ir = ir.replace("___rust_dealloc", actual_v0_symbol);
    let actual_v0_assembly = assembly.replace("___rust_dealloc", actual_v0_symbol);
    dealloc::validate("", "", &actual_v0_ir, &actual_v0_assembly).unwrap();

    let passive_personality_ir = ir.replace(
        "define void @___rust_dealloc() #0 {",
        "define void @___rust_dealloc() #0 personality ptr @__CxxFrameHandler3 {",
    );
    dealloc::validate("", "", &passive_personality_ir, &assembly).unwrap();

    let cleanup_ir = passive_personality_ir.replacen(
        "  ret void",
        "  invoke void @cleanup() to label %ok unwind label %abort\n  ret void",
        1,
    );
    assert_eq!(
        dealloc::validate("", "", &cleanup_ir, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_UNWIND_PATH_PRESENT"
    );

    let legacy_process_heap = "_ZN3std3sys5alloc7windows16get_process_heap17h0123456789abcdefE";
    let legacy_ir = ir.replace("@get_process_heap(", &format!("@{legacy_process_heap}("));
    let legacy_assembly = assembly.replace("get_process_heap", legacy_process_heap);
    dealloc::validate("", "", &legacy_ir, &legacy_assembly).unwrap();

    for forbidden in [
        "  %control = load atomic i64, ptr @STATE seq_cst, align 8\n",
        "  %tls = tail call ptr @llvm.threadlocal.address.p0(ptr @THREAD_STATE)\n",
        "  %slot = getelementptr i8, ptr @SLOTS, i64 0\n",
        "  call void @record_preadmission_fault()\n",
        "  call void @next_process_allocation_counter_helper()\n",
    ] {
        let invalid = ir.replacen("  ret void", &format!("{forbidden}  ret void"), 1);
        assert_eq!(
            dealloc::validate("", "", &invalid, &assembly)
                .unwrap_err()
                .code(),
            "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
        );
    }

    let duplicate_backend = ir.replacen(
        "  ret void",
        "  call i32 @HeapFree(ptr %heap, i32 0, ptr %allocation)\n  ret void",
        1,
    );
    assert_eq!(
        dealloc::validate("", "", &duplicate_backend, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );

    let unrelated_call = ir.replacen(
        "  ret void",
        "  call void @side_effect_helper()\n  ret void",
        1,
    );
    assert_eq!(
        dealloc::validate("", "", &unrelated_call, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );

    let unrelated_call_with_assume_operand = ir.replacen(
        "  ret void",
        "  call void @side_effect_helper(ptr @llvm.assume)\n  ret void",
        1,
    );
    assert_eq!(
        dealloc::validate("", "", &unrelated_call_with_assume_operand, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );

    let suffixed_backend = ir.replace("@HeapFree(", "@HeapFree2(");
    assert_eq!(
        dealloc::validate("", "", &suffixed_backend, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );

    let prefixed_backend = ir.replace("@HeapFree(", "@side_HeapFree(");
    assert_eq!(
        dealloc::validate("", "", &prefixed_backend, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );

    let suffixed_process_heap = ir.replace("@get_process_heap(", "@get_process_heap2(");
    assert_eq!(
        dealloc::validate("", "", &suffixed_process_heap, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );

    for forbidden in [
        "\tmovq\tSTATE(%rip), %rax\n",
        "\tmovl\t_tls_index(%rip), %eax\n",
        "\tlock\tincq\tSLOTS(%rip)\n",
        "\txchgq\t%rax, (%rcx)\n",
        "\tcallq\trecord_admitted_fault\n",
        "\tcallq\tnext_process_allocation_counter_helper\n",
        "\tcallq\t__chkstk\n",
    ] {
        let invalid = assembly.replacen("\tcallq\tget_process_heap", forbidden, 1);
        assert_eq!(
            dealloc::validate("", "", &ir, &invalid).unwrap_err().code(),
            "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
        );
    }

    let duplicate_backend = assembly.replacen(
        "\trex64 jmpq\t*__imp_HeapFree(%rip)",
        "\tcallq\t*__imp_HeapFree(%rip)\n\trex64 jmpq\t*__imp_HeapFree(%rip)",
        1,
    );
    assert_eq!(
        dealloc::validate("", "", &ir, &duplicate_backend)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );

    let unrelated_call = assembly.replacen(
        "\trex64 jmpq\t*__imp_HeapFree(%rip)",
        "\tcallq\tside_effect_helper\n\trex64 jmpq\t*__imp_HeapFree(%rip)",
        1,
    );
    assert_eq!(
        dealloc::validate("", "", &ir, &unrelated_call)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );

    let local_conditional_branch = assembly.replacen(
        "\trex64 jmpq\t*__imp_HeapFree(%rip)",
        "\tje\t.Ldealloc_done\n\trex64 jmpq\t*__imp_HeapFree(%rip)",
        1,
    );
    dealloc::validate("", "", &ir, &local_conditional_branch).unwrap();

    for external_target in ["side_effect_helper", "__imp_HeapFree"] {
        let external_conditional_branch = assembly.replacen(
            "\trex64 jmpq\t*__imp_HeapFree(%rip)",
            &format!("\tje\t{external_target}\n\trex64 jmpq\t*__imp_HeapFree(%rip)"),
            1,
        );
        assert_eq!(
            dealloc::validate("", "", &ir, &external_conditional_branch)
                .unwrap_err()
                .code(),
            "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
        );
    }

    let suffixed_backend = assembly.replace("HeapFree", "HeapFree2");
    assert_eq!(
        dealloc::validate("", "", &ir, &suffixed_backend)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );

    let prefixed_backend = assembly.replace("__imp_HeapFree", "side_HeapFree");
    assert_eq!(
        dealloc::validate("", "", &ir, &prefixed_backend)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
    );
}

#[test]
fn dealloc_parser_rejects_legacy_owner_and_foreign_helpers_in_every_artifact() {
    let (ir, assembly) = direct_dealloc_fixtures();
    for helper in ["dealloc_owner_active", "dealloc_foreign_active"] {
        for error in [
            dealloc::validate(helper, "", &ir, &assembly).unwrap_err(),
            dealloc::validate("", helper, &ir, &assembly).unwrap_err(),
            dealloc::validate("", "", &format!("{ir}\n; {helper}"), &assembly).unwrap_err(),
            dealloc::validate("", "", &ir, &format!("{assembly}\n; {helper}")).unwrap_err(),
        ] {
            assert_eq!(
                error.code(),
                "ALLOCATOR_COUNTER_CODEGEN_DEALLOC_PATH_MISMATCH"
            );
        }
    }
}

#[test]
fn first_touch_parser_requires_one_nounwind_global_claim_cas() {
    let ir = format!(
        "{}{}",
        ir_function(
            "claim_slot_slow",
            "  %next = cmpxchg weak ptr @NEXT_SLOT, i64 %old, i64 %new monotonic monotonic, align 8\n  ret void",
        ),
        ATTRIBUTES
    );
    let assembly = leaf_asm_function(
        "claim_slot_slow",
        "\tmovq\tNEXT_SLOT(%rip), %rax\n\tlock\tcmpxchgq\t%rcx, NEXT_SLOT(%rip)\n\tretq",
    );
    validate_first_touch_claim(&ir, &assembly).unwrap();

    let allocating = ir.replace("  ret void", "  call void @__rust_alloc()\n  ret void");
    assert_eq!(
        validate_first_touch_claim(&allocating, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_FIRST_TOUCH_MISMATCH"
    );
}

fn active_path_fixtures() -> (String, String) {
    let mut ir = String::new();
    let mut assembly = String::new();
    for (owner_symbol, foreign_symbol, system_symbol) in ACTIVE_HELPERS {
        ir.push_str(&ir_function_with_personality(
            owner_symbol,
            &format!(
                "  %tls = tail call ptr @llvm.threadlocal.address.p0(ptr @THREAD_STATE)\n  %cookie = load i64, ptr %tls\n  %owner = icmp eq i64 %cookie, %observed\n  br i1 %owner, label %owner_path, label %foreign_path\nowner_path:\n  call void @{system_symbol}()\n  ret void\nforeign_path:\n  call void @{foreign_symbol}()\n  ret void"
            ),
        ));
        assembly.push_str(&asm_function(
            owner_symbol,
            &format!(
                "\t.seh_pushreg\t%rbx\n\t.seh_stackalloc\t64\n\tmovl\t_tls_index(%rip), %eax\n\tmovq\t%gs:88, %rcx\n\tleaq\tTHREAD_STATE@SECREL32(%rax), %rax\n\tcmpq\t%rdx, 8(%rax)\n\tjne\t.Lforeign_path\n\tcallq\t{system_symbol}\n\tretq\n.Lforeign_path:\n\tcallq\t{foreign_symbol}\n\tretq"
            ),
        ));

        ir.push_str(&ir_function_with_personality(
            foreign_symbol,
            &format!(
                "  %slot = getelementptr i8, ptr @SLOTS, i64 0\n  %sequence = load atomic i64, ptr %slot monotonic, align 8\n  %odd = atomicrmw add ptr %slot, i64 1 seq_cst\n  %confirmed = load atomic i64, ptr @STATE seq_cst, align 8\n  %same = icmp eq i64 %confirmed, %observed\n  call void @{system_symbol}()\n  ret void"
            ),
        ));
        assembly.push_str(&asm_function(
            foreign_symbol,
            &format!(
                "\t.seh_pushreg\t%rbx\n\t.seh_stackalloc\t64\n\tleaq\tSLOTS(%rip), %rbx\n\tlock\txaddq\t%rcx, (%rbx)\n\tmovq\tSTATE(%rip), %rax\n\tcmpq\t%rdx, %rax\n\tcallq\t{system_symbol}\n\tretq"
            ),
        ));
    }
    ir.push_str(ATTRIBUTES);
    (ir, assembly)
}

#[test]
fn owner_parser_allows_passive_personality_and_forbids_shared_state_work() {
    let (ir, assembly) = active_path_fixtures();
    validate_owner_active_helpers(&ir, &assembly).unwrap();

    let invalid_ir = ir.replacen(
        "  %cookie = load i64, ptr %tls",
        "  %control = load atomic i64, ptr @STATE seq_cst, align 8\n  %cookie = load i64, ptr %tls",
        1,
    );
    assert_eq!(
        validate_owner_active_helpers(&invalid_ir, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_OWNER_PATH_MISMATCH"
    );

    for forbidden in [
        "  %slot = getelementptr i8, ptr @SLOTS, i64 0\n",
        "  %sequence = load i64, ptr %tls\n",
        "  %old = atomicrmw add ptr %tls, i64 1 seq_cst\n",
    ] {
        let invalid_ir = ir.replacen(
            "  %cookie = load i64, ptr %tls",
            &format!("{forbidden}  %cookie = load i64, ptr %tls"),
            1,
        );
        assert_eq!(
            validate_owner_active_helpers(&invalid_ir, &assembly)
                .unwrap_err()
                .code(),
            "ALLOCATOR_COUNTER_CODEGEN_OWNER_PATH_MISMATCH"
        );
    }

    let invalid_assembly = assembly.replacen("\tretq", "\tlock\tincq\t8(%rax)\n\tretq", 1);
    assert_eq!(
        validate_owner_active_helpers(&ir, &invalid_assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_OWNER_PATH_MISMATCH"
    );
}

#[test]
fn foreign_parser_requires_one_slot_rmw_followed_by_exact_control_postcheck() {
    let (ir, assembly) = active_path_fixtures();
    validate_foreign_active_helpers(&ir, &assembly).unwrap();

    let second_rmw = ir.replacen(
        "  %confirmed = load atomic i64, ptr @STATE seq_cst, align 8",
        "  %extra = atomicrmw add ptr %slot, i64 1 seq_cst\n  %confirmed = load atomic i64, ptr @STATE seq_cst, align 8",
        1,
    );
    assert_eq!(
        validate_foreign_active_helpers(&second_rmw, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_FOREIGN_PATH_MISMATCH"
    );

    let precheck = ir.replacen(
        "  %odd = atomicrmw add ptr %slot, i64 1 seq_cst\n  %confirmed = load atomic i64, ptr @STATE seq_cst, align 8",
        "  %confirmed = load atomic i64, ptr @STATE seq_cst, align 8\n  %odd = atomicrmw add ptr %slot, i64 1 seq_cst",
        1,
    );
    assert_eq!(
        validate_foreign_active_helpers(&precheck, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_FOREIGN_PATH_MISMATCH"
    );

    let wrong_identity = ir.replacen(
        "  %same = icmp eq i64 %confirmed, %observed",
        "  %same = icmp eq i64 %unrelated, %observed",
        1,
    );
    assert_eq!(
        validate_foreign_active_helpers(&wrong_identity, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_FOREIGN_PATH_MISMATCH"
    );

    let second_postcheck = ir.replacen(
        "  %same = icmp eq i64 %confirmed, %observed",
        "  %second = load atomic i64, ptr @STATE seq_cst, align 8\n  %same = icmp eq i64 %confirmed, %observed",
        1,
    );
    assert_eq!(
        validate_foreign_active_helpers(&second_postcheck, &assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_FOREIGN_PATH_MISMATCH"
    );
}

#[test]
fn owner_parser_rejects_an_abort_cleanup_even_with_nounwind_attribute() {
    let aborting = format!(
        "{}{}",
        ir_function_with_personality(
            "alloc_owner_active",
            "  invoke void @process_heap_alloc() to label %ok unwind label %cs_terminate\ncs_terminate:\n  %catch = catchswitch within none [label %abort] unwind to caller",
        ),
        ATTRIBUTES
    );
    assert_eq!(
        validate_no_eh_cleanup(
            unique_ir_function(&aborting, "alloc_owner_active").unwrap(),
            "alloc_owner_active"
        )
        .unwrap_err()
        .code(),
        "ALLOCATOR_COUNTER_CODEGEN_UNWIND_PATH_PRESENT"
    );
}

#[test]
fn storage_parser_requires_exact_static_aligned_slot_array() {
    let ir = concat!(
        "%AllocationSlot = type { %AtomicU64, %AtomicU64, %AtomicU64, %AtomicU64, %AtomicU64, %AtomicU64, %AtomicU64, [9 x i64] }\n",
        "@SLOTS = internal global [524288 x i8] zeroinitializer, align 128\n",
    );
    let assembly = ".section .bss\n.p2align\t7\nSLOTS:\n.zero\t524288\n";
    validate_static_storage(ir, assembly).unwrap();

    let invalid = ir.replace("align 128", "align 64");
    assert_eq!(
        validate_static_storage(&invalid, assembly)
            .unwrap_err()
            .code(),
        "ALLOCATOR_COUNTER_CODEGEN_STATIC_LAYOUT_MISMATCH"
    );
}

#[test]
fn tls_parser_requires_direct_const_no_destructor_windows_tls() {
    let selector_first = "@THREAD_STATE = internal thread_local unnamed_addr global <{ [5 x i8], [3 x i8], [7 x i64] }> <{ [5 x i8] c\"\\FF\\FF\\FF\\FF\\00\", [3 x i8] undef, [7 x i64] zeroinitializer }>, align 8\n";
    let reordered = format!(
        "@THREAD_STATE = internal thread_local global <{{ [61 x i8], [3 x i8] }}> <{{ [61 x i8] c\"{}\\FF\\FF\\FF\\FF\\00\", [3 x i8] undef }}>, align 8\n",
        r"\00".repeat(56)
    );
    let assembly = concat!(
        ".section\t.tls$,\"dw\"\n",
        "movl _tls_index(%rip), %eax\n",
        "movq %gs:88, %rcx\n",
        "leaq THREAD_STATE@SECREL32(%rax), %rax\n",
    );
    validate_tls_shape(selector_first, assembly).unwrap();
    validate_tls_shape(&reordered, assembly).unwrap();

    let lazy = format!("{reordered}declare void @register_dtor()\n");
    assert_eq!(
        validate_tls_shape(&lazy, assembly).unwrap_err().code(),
        "ALLOCATOR_COUNTER_CODEGEN_TLS_RUNTIME_FORBIDDEN"
    );

    let corrupt_cookie = reordered.replacen(r"\FF", r"\01", 1);
    let error = validate_tls_shape(&corrupt_cookie, assembly).unwrap_err();
    assert_eq!(
        error.code(),
        "ALLOCATOR_COUNTER_CODEGEN_TLS_LAYOUT_MISMATCH"
    );
    assert!(error.detail().contains("unexpected=1"));

    let misaligned = reordered.replace("align 8", "align 4");
    let error = validate_tls_shape(&misaligned, assembly).unwrap_err();
    assert_eq!(
        error.detail(),
        "expected the 64-byte TLS state to have align 8"
    );
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
#[test]
#[ignore = "emits the complete pinned ADR-041/ADR-042 release artifacts in an isolated target directory"]
fn pinned_windows_release_artifacts_pass_the_gate() {
    let root =
        std::fs::canonicalize(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .expect("the workspace root must be canonicalizable");
    validate_pinned_windows_release(&root)
        .unwrap_or_else(|error| panic!("{}: {}", error.code(), error.detail()));
}
