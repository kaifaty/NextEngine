use xtask::report::{AnimationLodConformanceDetailsV1, CommandReportV1};

pub(super) fn run() -> Result<(), String> {
    let report = next_verification::run_animation_lod_conformance_check()
        .map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "animation-lod",
        "PASS",
        AnimationLodConformanceDetailsV1 {
            cycles: report.cycles,
            full_pose_requests: report.full_pose_requests,
            reduced_pose_requests: report.reduced_pose_requests,
            held_pose_requests: report.held_pose_requests,
            intent_only_requests: report.intent_only_requests,
            culled_pose_requests: report.culled_pose_requests,
            sampled_pose_projections: report.sampled_pose_projections,
            held_pose_projections: report.held_pose_projections,
            bind_pose_projections: report.bind_pose_projections,
            no_pose_projections: report.no_pose_projections,
            complete_snapshot_publications: report.complete_snapshot_publications,
            rejected_snapshot_publications: report.rejected_snapshot_publications,
            due_intent_evaluations: report.due_intent_evaluations,
            resource_fallbacks: report.resource_fallbacks,
            authoritative_isolation_checks: report.authoritative_isolation_checks,
            renderer_frame_plans: report.renderer_frame_plans,
            lod_profile_revision: report.lod_profile_revision.to_hex(),
            final_state_root: report.final_state_root.to_hex(),
            final_command_ledger_hash: report.final_command_ledger_hash.to_hex(),
            final_physics_checkpoint_hash: report.final_physics_checkpoint_hash.to_hex(),
            final_animation_snapshot_hash: report.final_animation_snapshot_hash.to_hex(),
            final_presentation_snapshot_hash: report.final_presentation_snapshot_hash.to_hex(),
            final_frame_plan_hash: report.final_frame_plan_hash.to_hex(),
            matrix_digest: report.matrix_digest.to_hex(),
        },
    )
}
