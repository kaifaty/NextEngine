use xtask::report::{CommandReportV1, WaterLatticeDetailsV1};

pub(super) fn run() -> Result<(), String> {
    let report = next_verification::run_water_lattice_check().map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "water-lattice",
        "PASS",
        WaterLatticeDetailsV1 {
            check_id: "CONTINUUM-WATER-LATTICE-P1".to_owned(),
            region_id: report.region_id.to_hex(),
            cells: report.cells,
            edges: report.edges,
            run_ticks: report.run_ticks,
            total_volume_initial_mm3: report.total_volume_initial_cubic_millimetres.to_string(),
            total_volume_final_mm3: report.total_volume_final_cubic_millimetres.to_string(),
            conservation_exact: report.conservation_exact,
            east_column_wet_by_tick: report.east_column_wet_by_tick,
            settled_difference_max_um: report.settled_difference_max_micrometres,
            settle_tolerance_um: report.settle_tolerance_micrometres,
            levels_in_extent: report.levels_in_extent,
            dry_cells_final: report.dry_cells_final,
            final_level_east_um: report.final_level_east_micrometres,
            repeated_run_identical: report.repeated_run_identical,
            in_place_equals_cloning: report.in_place_equals_cloning,
            step_cost_cells: report.step_cost_cells,
            step_cost_edges: report.step_cost_edges,
            step_cost_max_us: report.step_cost_max_microseconds.to_string(),
            step_cost_mean_us: report.step_cost_mean_microseconds.to_string(),
            step_cost_debug_build: report.step_cost_debug_build,
            final_network_hash: report.final_network_hash.to_hex(),
            final_table_hash: report.final_table_hash.to_hex(),
        },
    )
}
