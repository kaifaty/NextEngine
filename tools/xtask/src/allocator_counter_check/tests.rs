use super::*;
use xtask::allocator_counter_kernel::AllocatorCounterSnapshotV1;

#[test]
fn parser_accepts_one_output_and_rejects_duplicates() {
    let default = parse_arguments(std::iter::empty()).expect("default arguments");
    assert_eq!(default.output, None);

    let request = parse_arguments(
        ["--output", "target/evidence"]
            .into_iter()
            .map(str::to_owned),
    )
    .expect("output arguments");
    assert_eq!(request.output, Some(PathBuf::from("target/evidence")));
    assert!(
        parse_arguments(
            ["--output", "one", "--output", "two"]
                .into_iter()
                .map(str::to_owned)
        )
        .is_err()
    );
    assert!(parse_arguments(["--unknown", "x"].into_iter().map(str::to_owned)).is_err());
}

#[test]
fn stable_diagnostic_preserves_the_failure_tail() {
    let detail = format!(
        "{}ACTUAL_COMPILER_ERROR_AT_TAIL",
        "compile-log ".repeat(100)
    );
    let diagnostic = stable_diagnostic("BUILD_FAILED", &detail);
    assert!(diagnostic.starts_with("BUILD_FAILED: compile-log"));
    assert!(diagnostic.ends_with("ACTUAL_COMPILER_ERROR_AT_TAIL"));
    assert!(diagnostic.contains(" ... "));
}

#[test]
fn interleaved_order_rotates_every_round() {
    use AllocatorCounterKernelModeV1::{InstrumentedEnabled, InstrumentedInactive, System};
    assert_eq!(
        rotated_mode_order(0),
        [System, InstrumentedInactive, InstrumentedEnabled]
    );
    assert_eq!(
        rotated_mode_order(1),
        [InstrumentedInactive, InstrumentedEnabled, System]
    );
    assert_eq!(
        rotated_mode_order(2),
        [InstrumentedEnabled, System, InstrumentedInactive]
    );
    assert_eq!(rotated_mode_order(3), rotated_mode_order(0));
}

#[test]
fn nearest_rank_median_and_basis_point_threshold_are_exact() {
    let reports = (1..=15)
        .rev()
        .map(|elapsed| report(AllocatorCounterKernelModeV1::System, elapsed, 'a'))
        .collect::<Vec<_>>();
    let summary =
        summarize_mode(AllocatorCounterKernelModeV1::System, &reports).expect("median summary");
    assert_eq!(summary.median_nanoseconds, 8);
    assert_eq!(overhead_basis_points(10_000, 10_300).expect("300 bp"), 300);
    assert_eq!(overhead_basis_points(10_000, 10_301).expect("301 bp"), 301);
}

#[test]
fn threshold_evaluation_fails_only_the_over_budget_mode() {
    let retained = RetainedReports {
        system: repeated_reports(AllocatorCounterKernelModeV1::System, 10_000, 'a'),
        inactive: repeated_reports(
            AllocatorCounterKernelModeV1::InstrumentedInactive,
            10_301,
            'a',
        ),
        enabled: repeated_reports(
            AllocatorCounterKernelModeV1::InstrumentedEnabled,
            10_300,
            'a',
        ),
    };
    let identity = HelperIdentity::from(&retained.system[0]);
    let roots = retained.system[0].roots.clone();
    let evaluated = evaluate_retained(retained, Some(identity), Some(roots), Vec::new())
        .expect("threshold report");
    assert_eq!(evaluated.status, AllocatorCounterCheckStatusV1::Fail);
    assert_eq!(evaluated.inactive_overhead_basis_points, Some(301));
    assert_eq!(evaluated.enabled_overhead_basis_points, Some(300));
    assert_eq!(evaluated.diagnostics.len(), 1);
    assert!(
        evaluated.diagnostics[0].starts_with("ALLOCATOR_COUNTER_CHECK_INACTIVE_OVERHEAD_EXCEEDED")
    );
}

#[test]
fn root_mismatch_is_a_stable_fail() {
    let expected_hash = xtask::performance::sha256_hex(ALLOCATOR_COUNTER_KERNEL_PREIMAGE);
    let mut identity = None;
    let mut roots = None;
    let system = report(AllocatorCounterKernelModeV1::System, 10_000, 'a');
    validate_helper_report(
        &system,
        AllocatorCounterKernelModeV1::System,
        &expected_hash,
        &mut identity,
        &mut roots,
    )
    .expect("system reference");
    let inactive = report(
        AllocatorCounterKernelModeV1::InstrumentedInactive,
        10_000,
        'b',
    );
    let failure = validate_helper_report(
        &inactive,
        AllocatorCounterKernelModeV1::InstrumentedInactive,
        &expected_hash,
        &mut identity,
        &mut roots,
    )
    .expect_err("root mismatch");
    assert_eq!(failure.status, AllocatorCounterCheckStatusV1::Fail);
    assert!(
        failure
            .diagnostic
            .starts_with("ALLOCATOR_COUNTER_CHECK_ROOT_MISMATCH")
    );
}

fn repeated_reports(
    mode: AllocatorCounterKernelModeV1,
    elapsed: u64,
    root: char,
) -> Vec<AllocatorCounterKernelReportV1> {
    (0..RETAINED_ROUNDS)
        .map(|_| report(mode, elapsed, root))
        .collect()
}

fn report(
    mode: AllocatorCounterKernelModeV1,
    elapsed_nanoseconds: u64,
    root: char,
) -> AllocatorCounterKernelReportV1 {
    let reserved_bytes = match mode {
        AllocatorCounterKernelModeV1::System => 0,
        _ => u64::try_from(next_process_allocation_counter::reserved_bytes())
            .expect("reserved bytes"),
    };
    let allocator_snapshot = (mode == AllocatorCounterKernelModeV1::InstrumentedEnabled).then_some(
        AllocatorCounterSnapshotV1 {
            producer_pid: 1,
            window_id: 1,
            alloc_count: 1,
            alloc_bytes: 64,
            alloc_zeroed_count: 0,
            alloc_zeroed_bytes: 0,
            realloc_count: 0,
            realloc_bytes: 0,
            allocator_allocation_count: 1,
            allocator_allocated_bytes: 64,
        },
    );
    AllocatorCounterKernelReportV1 {
        schema_version: ALLOCATOR_COUNTER_KERNEL_SCHEMA_VERSION,
        methodology_version: ALLOCATOR_COUNTER_KERNEL_METHODOLOGY_VERSION.to_owned(),
        mode,
        kernel_hash: xtask::performance::sha256_hex(ALLOCATOR_COUNTER_KERNEL_PREIMAGE),
        commit: "a".repeat(40),
        worktree_clean: true,
        toolchain_hex: "aa".to_owned(),
        target_triple: "x86_64-pc-windows-msvc".to_owned(),
        build_profile: "release".to_owned(),
        elapsed_nanoseconds,
        roots: AllocatorCounterKernelRootsV1 {
            ticks: 900,
            command_body_count: 900,
            final_state_root: root.to_string().repeat(64),
            final_command_archive_root: "c".repeat(64),
            final_command_identity_index_root: "d".repeat(64),
            application_final_state_root: None,
        },
        allocator_snapshot,
        reserved_bytes,
    }
}
