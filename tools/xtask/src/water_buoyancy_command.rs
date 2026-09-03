use xtask::report::{CommandReportV1, WaterBuoyancyDetailsV1};

pub(super) fn run() -> Result<(), String> {
    let report =
        next_verification::run_water_buoyancy_check().map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "water-buoyancy",
        "PASS",
        WaterBuoyancyDetailsV1 {
            check_id: "CONTINUUM-WATER-BUOYANCY-P1".to_owned(),
            crate_body: report.crate_body_id.subject_id.to_hex(),
            dry_body: report.dry_body_id.subject_id.to_hex(),
            run_ticks: report.run_ticks,
            level_initial_um: report.level_initial_micrometres,
            level_raised_um: report.level_raised_micrometres,
            immersion_settled_um: report.immersion_settled_micrometres,
            immersion_raised_um: report.immersion_raised_micrometres,
            crate_bottom_raised_um: report.crate_bottom_raised_micrometres,
            crate_max_velocity_um_per_s: report.crate_max_velocity_micrometres_per_second,
            ticks_with_crate_record: report.ticks_with_crate_record,
            max_displaced_volume_mm3: report.max_displaced_volume_cubic_millimetres,
            dry_body_records: report.dry_body_records,
            dry_body_trajectory_identical: report.dry_body_trajectory_identical,
            step_inputs_round_trip: report.step_inputs_round_trip,
            checkpoint_round_trip: report.checkpoint_round_trip,
            restored_run_identical: report.restored_run_identical,
            repeated_run_identical: report.repeated_run_identical,
            batch_cost_bodies: report.batch_cost_bodies,
            batch_cost_volumes: report.batch_cost_volumes,
            batch_cost_max_us: report.batch_cost_max_microseconds.to_string(),
            batch_cost_debug_build: report.batch_cost_debug_build,
            final_state_root: report.final_state_root.to_hex(),
            final_physics_checkpoint_hash: report.final_physics_checkpoint_hash.to_hex(),
            matrix_digest: report.matrix_digest.to_hex(),
        },
    )
}
