use xtask::report::{CommandReportV1, WaterPresentDetailsV1};

pub(super) fn run() -> Result<(), String> {
    let report = next_verification::run_water_present_check().map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "water-present",
        "PASS",
        WaterPresentDetailsV1 {
            check_id: "CONTINUUM-WATER-PRESENT-P1".to_owned(),
            run_ticks: report.run_ticks,
            frames: report.frames,
            surfaces_per_frame: report.surfaces_per_frame,
            vertex_capacity: report.vertex_capacity,
            index_capacity: report.index_capacity,
            particle_capacity: report.particle_capacity,
            roots_identical: report.roots_identical,
            capacity_violations: report.capacity_violations,
            bounds_violations: report.bounds_violations,
            purity_identical: report.purity_identical,
            max_jet_particles: report.max_jet_particles,
            frames_with_jet: report.frames_with_jet,
            max_ripple_um: report.max_ripple_micrometres,
            stage_cost_max_us: report.stage_cost_max_microseconds.to_string(),
            stage_cost_mean_us: report.stage_cost_mean_microseconds.to_string(),
            stage_cost_debug_build: report.stage_cost_debug_build,
            repeated_run_identical: report.repeated_run_identical,
            final_state_root: report.final_state_root.to_hex(),
            final_physics_checkpoint_hash: report.final_physics_checkpoint_hash.to_hex(),
            frames_digest: report.frames_digest.to_hex(),
            matrix_digest: report.matrix_digest.to_hex(),
        },
    )
}
