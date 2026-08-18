use xtask::report::{CommandReportV1, PhysicalCharacterConformanceDetailsV1};

pub(super) fn run() -> Result<(), String> {
    let report = next_verification::run_physical_gameplay_conformance_check()
        .map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "physical-character",
        "PASS",
        PhysicalCharacterConformanceDetailsV1 {
            trip_contact_events: report.trip_contact_events,
            trip_final_pose_um: report.trip_final_pose_micrometres,
            carry_contact_events: report.carry_contact_events,
            carry_final_pose_um: report.carry_final_pose_micrometres,
            carried_load_pose_um: report.carried_load_pose_micrometres,
            capsule_clearance_um: report.capsule_clearance_micrometres,
            restored_contact_ticks: report.restored_contact_ticks,
            melee_contact_events: report.melee_contact_events,
            melee_npc_health: report.melee_npc_health,
            repeated_run_identical: report.repeated_run_identical,
            final_state_root: report.final_state_root.to_hex(),
            final_command_ledger_hash: report.final_command_ledger_hash.to_hex(),
            final_physics_checkpoint_hash: report.final_physics_checkpoint_hash.to_hex(),
            matrix_digest: report.matrix_digest.to_hex(),
        },
    )
}
