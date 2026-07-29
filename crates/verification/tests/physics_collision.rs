use next_verification::run_physics_collision_check;

#[test]
fn collision_only_check_does_not_require_gameplay_agent_or_stream_transition() {
    let report = run_physics_collision_check().expect("collision-only check must pass");

    assert_eq!(report.gameplay_ticks, 6);
    assert_eq!(report.physics_substeps, 12);
}
