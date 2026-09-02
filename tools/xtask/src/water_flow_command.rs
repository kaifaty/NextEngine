use xtask::report::{CommandReportV1, WaterFlowDetailsV1};

pub(super) fn run() -> Result<(), String> {
    let report = next_verification::run_water_flow_check().map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "water-flow",
        "PASS",
        WaterFlowDetailsV1 {
            check_id: "CONTINUUM-WATER-FLOW-P1".to_owned(),
            vessel_a_id: report.vessel_a_id.to_hex(),
            vessel_b_id: report.vessel_b_id.to_hex(),
            gate_id: report.gate_id.to_hex(),
            run_ticks: report.run_ticks,
            analytic_drain_ticks: report.analytic_drain_ticks,
            drained_by_tick: report.drained_by_tick,
            level_a_initial_um: report.level_a_initial_micrometres,
            level_a_final_um: report.level_a_final_micrometres,
            level_b_final_um: report.level_b_final_micrometres,
            total_volume_initial_mm3: report.total_volume_initial_cubic_millimetres.to_string(),
            total_volume_final_mm3: report.total_volume_final_cubic_millimetres.to_string(),
            source_volume_mm3: report.source_volume_cubic_millimetres.to_string(),
            sink_volume_mm3: report.sink_volume_cubic_millimetres.to_string(),
            conservation_exact: report.conservation_exact,
            gate_closed_flux_zero: report.gate_closed_flux_zero,
            gate_reopened_flux_positive: report.gate_reopened_flux_positive,
            committed_commands: report.committed_commands,
            rejected_commands: report.rejected_commands,
            flow_events: report.flow_events,
            checkpoint_round_trip: report.checkpoint_round_trip,
            restored_run_identical: report.restored_run_identical,
            repeated_run_identical: report.repeated_run_identical,
            step_cost_cells: report.step_cost_cells,
            step_cost_edges: report.step_cost_edges,
            step_cost_max_us: report.step_cost_max_microseconds.to_string(),
            step_cost_debug_build: report.step_cost_debug_build,
            final_state_root: report.final_state_root.to_hex(),
            final_physics_checkpoint_hash: report.final_physics_checkpoint_hash.to_hex(),
            matrix_digest: report.matrix_digest.to_hex(),
        },
    )
}
