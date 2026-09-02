use xtask::report::{CommandReportV1, WaterProbeDetailsV1, WaterVolumeDetailsV1};

fn probes(probes: &[next_verification::WaterProbeResultV1]) -> Vec<WaterProbeDetailsV1> {
    probes
        .iter()
        .map(|probe| WaterProbeDetailsV1 {
            label: probe.label.clone(),
            point_um: probe.point_micrometres,
            in_volume: probe.in_volume,
            depth_um: probe.depth_micrometres,
            class: format!("{:?}", probe.class),
        })
        .collect()
}

pub(super) fn run() -> Result<(), String> {
    let report = next_verification::run_water_volume_check().map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "water-volume",
        "PASS",
        WaterVolumeDetailsV1 {
            check_id: "CONTINUUM-WATER-VOLUME-P1".to_owned(),
            basin_id: report.basin_id.to_hex(),
            player_pose_um: report.player_pose_micrometres,
            player_class_initial: format!("{:?}", report.player_class_initial),
            player_class_raised: format!("{:?}", report.player_class_raised),
            surface_translation_initial_um: report.surface_translation_initial_micrometres,
            surface_translation_raised_um: report.surface_translation_raised_micrometres,
            initial_level_um: report.initial_level_micrometres,
            committed_level_um: report.committed_level_micrometres,
            initial_probes: probes(&report.initial_probes),
            raised_probes: probes(&report.raised_probes),
            committed_commands: report.committed_commands,
            rejected_commands: report.rejected_commands,
            water_events: report.water_events,
            checkpoint_round_trip: report.checkpoint_round_trip,
            restored_run_identical: report.restored_run_identical,
            repeated_run_identical: report.repeated_run_identical,
            final_state_root: report.final_state_root.to_hex(),
            final_physics_checkpoint_hash: report.final_physics_checkpoint_hash.to_hex(),
            matrix_digest: report.matrix_digest.to_hex(),
        },
    )
}
