use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::{Duration, Instant};

use next_contracts::ids::ContentHash;
use next_render::{B0FramePlanV1, RenderTargetV1, build_b0_frame_plan};

use crate::player_fixture::prepare_game_frame_with_scratch;
use crate::scratch::ScratchContext;

const RENDER_FRAME_PLANNING_CYCLES: u64 = 10_000;
const RENDER_FRAME_PLANNING_LIMIT: Duration = Duration::from_secs(30);
const REFERENCE_RENDER_TARGET: RenderTargetV1 = RenderTargetV1 {
    extent: [960, 540],
    target_revision: 1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderFramePlanningPerformanceReport {
    pub cycles: u64,
    pub elapsed_microseconds: u128,
    pub visible_object_count: u32,
    pub indexed_draw_count: u32,
    pub fallback_material_draw_count: u32,
    pub frame_plan_hash: ContentHash,
}

pub fn run_render_frame_planning_performance_check()
-> Result<RenderFramePlanningPerformanceReport, RenderFramePlanningPerformanceError> {
    run_render_frame_planning_performance_check_in(&std::env::temp_dir())
}

pub fn run_render_frame_planning_performance_check_in(
    scratch_root: &Path,
) -> Result<RenderFramePlanningPerformanceReport, RenderFramePlanningPerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| RenderFramePlanningPerformanceError::Setup(error.to_string()))?;
    run_render_frame_planning_performance_check_with_scratch(&scratch)
}

pub(crate) fn run_render_frame_planning_performance_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<RenderFramePlanningPerformanceReport, RenderFramePlanningPerformanceError> {
    let prepared = prepare_game_frame_with_scratch(scratch)
        .map_err(|error| RenderFramePlanningPerformanceError::Setup(error.to_string()))?;
    let baseline = build_b0_frame_plan(
        &prepared.snapshot,
        &prepared.render_content_catalog,
        REFERENCE_RENDER_TARGET,
    )
    .map_err(|error| RenderFramePlanningPerformanceError::Planner {
        cycle: 0,
        detail: error.to_string(),
    })?;
    validate_fixture_baseline(&baseline, &prepared.check)?;

    let started = Instant::now();
    for cycle in 1..=RENDER_FRAME_PLANNING_CYCLES {
        let plan = build_b0_frame_plan(
            &prepared.snapshot,
            &prepared.render_content_catalog,
            REFERENCE_RENDER_TARGET,
        )
        .map_err(|error| RenderFramePlanningPerformanceError::Planner {
            cycle,
            detail: error.to_string(),
        })?;
        if plan != baseline {
            return Err(RenderFramePlanningPerformanceError::OutputChanged {
                cycle,
                expected_frame_plan_hash: baseline.frame_plan_hash,
                actual_frame_plan_hash: plan.frame_plan_hash,
                expected_counts: plan_counts(&baseline),
                actual_counts: plan_counts(&plan),
            });
        }
    }
    let elapsed = started.elapsed();
    if elapsed > RENDER_FRAME_PLANNING_LIMIT {
        return Err(RenderFramePlanningPerformanceError::BudgetExceeded {
            elapsed_microseconds: elapsed.as_micros(),
            maximum_microseconds: RENDER_FRAME_PLANNING_LIMIT.as_micros(),
        });
    }

    Ok(RenderFramePlanningPerformanceReport {
        cycles: RENDER_FRAME_PLANNING_CYCLES,
        elapsed_microseconds: elapsed.as_micros(),
        visible_object_count: baseline.visible_object_count,
        indexed_draw_count: baseline.indexed_draw_count,
        fallback_material_draw_count: baseline.fallback_material_draw_count,
        frame_plan_hash: baseline.frame_plan_hash,
    })
}

fn validate_fixture_baseline(
    baseline: &B0FramePlanV1,
    check: &crate::GameCheckReport,
) -> Result<(), RenderFramePlanningPerformanceError> {
    let expected_counts = (
        check.rendered_object_count,
        check.indexed_draw_count,
        check.fallback_material_draw_count,
    );
    let actual_counts = plan_counts(baseline);
    if baseline.frame_plan_hash != check.frame_plan_hash || actual_counts != expected_counts {
        return Err(RenderFramePlanningPerformanceError::FixtureMismatch {
            expected_frame_plan_hash: check.frame_plan_hash,
            actual_frame_plan_hash: baseline.frame_plan_hash,
            expected_counts,
            actual_counts,
        });
    }
    Ok(())
}

const fn plan_counts(plan: &B0FramePlanV1) -> (u32, u32, u32) {
    (
        plan.visible_object_count,
        plan.indexed_draw_count,
        plan.fallback_material_draw_count,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenderFramePlanningPerformanceError {
    Setup(String),
    Planner {
        cycle: u64,
        detail: String,
    },
    FixtureMismatch {
        expected_frame_plan_hash: ContentHash,
        actual_frame_plan_hash: ContentHash,
        expected_counts: (u32, u32, u32),
        actual_counts: (u32, u32, u32),
    },
    OutputChanged {
        cycle: u64,
        expected_frame_plan_hash: ContentHash,
        actual_frame_plan_hash: ContentHash,
        expected_counts: (u32, u32, u32),
        actual_counts: (u32, u32, u32),
    },
    BudgetExceeded {
        elapsed_microseconds: u128,
        maximum_microseconds: u128,
    },
}

impl Display for RenderFramePlanningPerformanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Setup(detail) => write!(formatter, "render planning setup: {detail}"),
            Self::Planner { cycle, detail } => {
                write!(formatter, "render planning cycle {cycle}: {detail}")
            }
            Self::FixtureMismatch {
                expected_frame_plan_hash,
                actual_frame_plan_hash,
                expected_counts,
                actual_counts,
            } => write!(
                formatter,
                "render planning fixture mismatch: expected hash {} and counts {expected_counts:?}, actual hash {} and counts {actual_counts:?}",
                expected_frame_plan_hash.to_hex(),
                actual_frame_plan_hash.to_hex()
            ),
            Self::OutputChanged {
                cycle,
                expected_frame_plan_hash,
                actual_frame_plan_hash,
                expected_counts,
                actual_counts,
            } => write!(
                formatter,
                "render planning output changed at cycle {cycle}: expected hash {} and counts {expected_counts:?}, actual hash {} and counts {actual_counts:?}",
                expected_frame_plan_hash.to_hex(),
                actual_frame_plan_hash.to_hex()
            ),
            Self::BudgetExceeded {
                elapsed_microseconds,
                maximum_microseconds,
            } => write!(
                formatter,
                "render frame planning took {elapsed_microseconds}us, budget is {maximum_microseconds}us"
            ),
        }
    }
}

impl Error for RenderFramePlanningPerformanceError {}

#[cfg(test)]
mod tests {
    #[test]
    fn frame_planner_is_stable_and_has_a_bounded_numeric_gate() {
        let report =
            super::run_render_frame_planning_performance_check().expect("performance gate");
        assert_eq!(report.cycles, 10_000);
        assert_eq!(report.visible_object_count, 5);
        assert_eq!(report.indexed_draw_count, 5);
        assert_eq!(report.fallback_material_draw_count, 0);
        assert_ne!(
            report.frame_plan_hash,
            next_contracts::ids::ContentHash::default()
        );
    }
}
