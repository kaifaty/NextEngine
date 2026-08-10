use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use next_contracts::ids::ContentHash;
use next_render::{B0FramePlanV1, RenderTargetV1, build_b0_frame_plan};

use crate::PreparedGameFrameV1;
use crate::player_fixture::prepare_game_frame_with_scratch;
use crate::scratch::{ScratchContext, ScratchDirectory};

const RENDER_FRAME_PLANNING_CYCLES: u64 = 10_000;
const REFERENCE_RENDER_TARGET: RenderTargetV1 = RenderTargetV1 {
    extent: [960, 540],
    target_revision: 1,
};
static NEXT_RENDER_PREPARATION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderFramePlanningPerformanceReport {
    pub cycles: u64,
    pub elapsed_microseconds: u128,
    pub visible_object_count: u32,
    pub indexed_draw_count: u32,
    pub fallback_material_draw_count: u32,
    pub frame_plan_hash: ContentHash,
}

/// A frame-planning workload whose project, render inputs, baseline, and
/// cleanup scope have already been prepared.
pub struct PreparedRenderFramePlanningPerformanceCheck {
    preparation_id: u64,
    directory: ScratchDirectory,
    prepared: PreparedGameFrameV1,
    baseline: B0FramePlanV1,
    run_started: bool,
}

/// Opaque measured outcome consumed by
/// [`PreparedRenderFramePlanningPerformanceCheck::finish`] after the caller
/// closes its external measurement window.
pub struct RenderFramePlanningPerformanceMeasurement {
    preparation_id: u64,
    elapsed: Duration,
}

pub fn run_render_frame_planning_performance_check()
-> Result<RenderFramePlanningPerformanceReport, RenderFramePlanningPerformanceError> {
    run_render_frame_planning_performance_check_in(&std::env::temp_dir())
}

pub fn run_render_frame_planning_performance_check_in(
    scratch_root: &Path,
) -> Result<RenderFramePlanningPerformanceReport, RenderFramePlanningPerformanceError> {
    let mut prepared = prepare_render_frame_planning_performance_check_in(scratch_root)?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

pub(crate) fn run_render_frame_planning_performance_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<RenderFramePlanningPerformanceReport, RenderFramePlanningPerformanceError> {
    let mut prepared = prepare_render_frame_planning_performance_check_with_scratch(scratch)?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

pub fn prepare_render_frame_planning_performance_check()
-> Result<PreparedRenderFramePlanningPerformanceCheck, RenderFramePlanningPerformanceError> {
    prepare_render_frame_planning_performance_check_in(&std::env::temp_dir())
}

pub fn prepare_render_frame_planning_performance_check_in(
    scratch_root: &Path,
) -> Result<PreparedRenderFramePlanningPerformanceCheck, RenderFramePlanningPerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| RenderFramePlanningPerformanceError::Setup(error.to_string()))?;
    prepare_render_frame_planning_performance_check_with_scratch(&scratch)
}

fn prepare_render_frame_planning_performance_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<PreparedRenderFramePlanningPerformanceCheck, RenderFramePlanningPerformanceError> {
    let directory = scratch
        .create_directory("render-planning-performance")
        .map_err(|error| RenderFramePlanningPerformanceError::Setup(error.to_string()))?;
    let prepared = match prepare_game_frame_with_scratch(&directory.context()) {
        Ok(prepared) => prepared,
        Err(error) => {
            return Err(finish_failed_preparation(
                directory,
                RenderFramePlanningPerformanceError::Setup(error.to_string()),
            ));
        }
    };
    let baseline = build_b0_frame_plan(
        &prepared.snapshot,
        &prepared.render_content_catalog,
        REFERENCE_RENDER_TARGET,
    )
    .map_err(|error| RenderFramePlanningPerformanceError::Planner {
        cycle: 0,
        detail: error.to_string(),
    });
    let baseline = match baseline {
        Ok(baseline) => baseline,
        Err(error) => return Err(finish_failed_preparation(directory, error)),
    };
    if let Err(error) = validate_fixture_baseline(&baseline, &prepared.check) {
        return Err(finish_failed_preparation(directory, error));
    }
    let preparation_id = match next_render_preparation_id() {
        Ok(preparation_id) => preparation_id,
        Err(error) => return Err(finish_failed_preparation(directory, error)),
    };
    Ok(PreparedRenderFramePlanningPerformanceCheck {
        preparation_id,
        directory,
        prepared,
        baseline,
        run_started: false,
    })
}

impl PreparedRenderFramePlanningPerformanceCheck {
    /// Executes only the repeated production frame-plan construction and
    /// equality check from the legacy measured kernel.
    pub fn run_measured(
        &mut self,
    ) -> Result<RenderFramePlanningPerformanceMeasurement, RenderFramePlanningPerformanceError>
    {
        if self.run_started {
            return Err(RenderFramePlanningPerformanceError::InvalidMeasurement(
                "measured workload can run only once".to_owned(),
            ));
        }
        self.run_started = true;
        let started = Instant::now();
        for cycle in 1..=RENDER_FRAME_PLANNING_CYCLES {
            let plan = build_b0_frame_plan(
                &self.prepared.snapshot,
                &self.prepared.render_content_catalog,
                REFERENCE_RENDER_TARGET,
            )
            .map_err(|error| RenderFramePlanningPerformanceError::Planner {
                cycle,
                detail: error.to_string(),
            })?;
            if plan != self.baseline {
                return Err(RenderFramePlanningPerformanceError::OutputChanged {
                    cycle,
                    expected_frame_plan_hash: self.baseline.frame_plan_hash,
                    actual_frame_plan_hash: plan.frame_plan_hash,
                    expected_counts: plan_counts(&self.baseline),
                    actual_counts: plan_counts(&plan),
                });
            }
        }
        Ok(RenderFramePlanningPerformanceMeasurement {
            preparation_id: self.preparation_id,
            elapsed: started.elapsed(),
        })
    }

    /// Validates and materializes the report after the external measurement
    /// window, then removes the prepared cleanup scope on every result path.
    pub fn finish(
        self,
        measurement: Result<
            RenderFramePlanningPerformanceMeasurement,
            RenderFramePlanningPerformanceError,
        >,
    ) -> Result<RenderFramePlanningPerformanceReport, RenderFramePlanningPerformanceError> {
        let result = measurement.and_then(|measurement| self.build_report(measurement));
        drop(self.prepared);
        self.directory.finish(result, |error| {
            RenderFramePlanningPerformanceError::Setup(error.to_string())
        })
    }

    /// Cancels a prepared workload before measurement and reports cleanup
    /// failures instead of relying on best-effort `Drop` cleanup.
    pub fn cancel(self) -> Result<(), RenderFramePlanningPerformanceError> {
        drop(self.prepared);
        self.directory.finish(Ok(()), |error| {
            RenderFramePlanningPerformanceError::Setup(error.to_string())
        })
    }

    fn build_report(
        &self,
        measurement: RenderFramePlanningPerformanceMeasurement,
    ) -> Result<RenderFramePlanningPerformanceReport, RenderFramePlanningPerformanceError> {
        if !self.run_started || measurement.preparation_id != self.preparation_id {
            return Err(RenderFramePlanningPerformanceError::InvalidMeasurement(
                "measurement does not belong to this completed preparation".to_owned(),
            ));
        }
        Ok(RenderFramePlanningPerformanceReport {
            cycles: RENDER_FRAME_PLANNING_CYCLES,
            elapsed_microseconds: measurement.elapsed.as_micros(),
            visible_object_count: self.baseline.visible_object_count,
            indexed_draw_count: self.baseline.indexed_draw_count,
            fallback_material_draw_count: self.baseline.fallback_material_draw_count,
            frame_plan_hash: self.baseline.frame_plan_hash,
        })
    }
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
    InvalidMeasurement(String),
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
}

impl Display for RenderFramePlanningPerformanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Setup(detail) => write!(formatter, "render planning setup: {detail}"),
            Self::InvalidMeasurement(detail) => {
                write!(formatter, "render planning measurement: {detail}")
            }
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
        }
    }
}

impl Error for RenderFramePlanningPerformanceError {}

fn next_render_preparation_id() -> Result<u64, RenderFramePlanningPerformanceError> {
    NEXT_RENDER_PREPARATION_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| {
            RenderFramePlanningPerformanceError::InvalidMeasurement(
                "preparation identity overflow".to_owned(),
            )
        })
}

fn finish_failed_preparation(
    directory: ScratchDirectory,
    error: RenderFramePlanningPerformanceError,
) -> RenderFramePlanningPerformanceError {
    match directory.finish(Err::<(), _>(error), |cleanup_error| {
        RenderFramePlanningPerformanceError::Setup(cleanup_error.to_string())
    }) {
        Err(error) => error,
        Ok(()) => unreachable!("an error result cannot become successful during cleanup"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RenderFramePlanningPerformanceError, prepare_render_frame_planning_performance_check_in,
        run_render_frame_planning_performance_check_in,
    };

    #[test]
    fn frame_planner_is_stable_and_has_a_bounded_workload() {
        let _measurement_guard = crate::test_support::lock_numeric_performance_measurement();
        let report =
            super::run_render_frame_planning_performance_check().expect("performance gate");
        assert_eq!(report.cycles, 10_000);
        assert_eq!(report.visible_object_count, 6);
        assert_eq!(report.indexed_draw_count, 6);
        assert_eq!(report.fallback_material_draw_count, 0);
        assert_ne!(
            report.frame_plan_hash,
            next_contracts::ids::ContentHash::default()
        );
    }

    #[test]
    fn explicit_preparation_preserves_the_legacy_plan() {
        let _measurement_guard = crate::test_support::lock_numeric_performance_measurement();
        let scratch = std::env::temp_dir();
        let legacy =
            run_render_frame_planning_performance_check_in(&scratch).expect("legacy workload");
        let mut prepared = prepare_render_frame_planning_performance_check_in(&scratch)
            .expect("prepared workload");
        let measurement = prepared.run_measured();
        let explicit = prepared.finish(measurement).expect("finish workload");
        assert_eq!(explicit.cycles, legacy.cycles);
        assert_eq!(explicit.visible_object_count, legacy.visible_object_count);
        assert_eq!(explicit.indexed_draw_count, legacy.indexed_draw_count);
        assert_eq!(
            explicit.fallback_material_draw_count,
            legacy.fallback_material_draw_count
        );
        assert_eq!(explicit.frame_plan_hash, legacy.frame_plan_hash);
    }

    #[test]
    fn prepared_workload_is_single_use_and_cleans_up() {
        let _measurement_guard = crate::test_support::lock_numeric_performance_measurement();
        let mut prepared =
            prepare_render_frame_planning_performance_check_in(&std::env::temp_dir())
                .expect("prepared workload");
        let path = prepared.directory.path().to_path_buf();
        let measurement = prepared.run_measured();
        let second = match prepared.run_measured() {
            Err(error) => error,
            Ok(_) => panic!("prepared workload must be single-use"),
        };
        assert!(second.to_string().contains("can run only once"));
        prepared.finish(measurement).expect("finish first run");
        assert!(!path.exists());
    }

    #[test]
    fn finish_rejects_foreign_measurement_and_cleans_both_preparations() {
        let _measurement_guard = crate::test_support::lock_numeric_performance_measurement();
        let mut source = prepare_render_frame_planning_performance_check_in(&std::env::temp_dir())
            .expect("source preparation");
        let source_path = source.directory.path().to_path_buf();
        let measurement = source.run_measured().expect("source measurement");
        let target = prepare_render_frame_planning_performance_check_in(&std::env::temp_dir())
            .expect("target preparation");
        let target_path = target.directory.path().to_path_buf();
        let error = target
            .finish(Ok(measurement))
            .expect_err("foreign measurement must fail");
        assert!(error.to_string().contains("does not belong"));
        assert!(!target_path.exists());
        let source_error = source
            .finish(injected_failure())
            .expect_err("injected failure");
        assert!(source_error.to_string().contains("injected caller failure"));
        assert!(!source_path.exists());
    }

    #[test]
    fn finish_preserves_a_workload_error_and_cleans_prepared_scratch() {
        let prepared = prepare_render_frame_planning_performance_check_in(&std::env::temp_dir())
            .expect("prepared workload");
        let path = prepared.directory.path().to_path_buf();
        let error = prepared
            .finish(injected_failure())
            .expect_err("injected failure");
        assert!(error.to_string().contains("injected caller failure"));
        assert!(!path.exists());
    }

    fn injected_failure()
    -> Result<super::RenderFramePlanningPerformanceMeasurement, RenderFramePlanningPerformanceError>
    {
        Err(RenderFramePlanningPerformanceError::Planner {
            cycle: 0,
            detail: "injected caller failure".to_owned(),
        })
    }
}
