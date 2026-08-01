use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use next_contracts::agent::MotorCapabilityStateV1;
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PhysicsContactId, SchemaId, content_hash_from_bytes};
use next_contracts::input::CORE_MELEE_ACTION_ID;
use next_contracts::rpg::{RpgPhysicalContactFactV1, RpgSnapshotV2};

use crate::NeutralPlayerFixture;
use crate::cooked_project_rpg_snapshot;
use crate::player_fixture::build_neutral_player_fixture_with_scratch;
use crate::scratch::{ScratchContext, ScratchDirectory};

const AGENT_PLANNING_CYCLES: u64 = 1_000;
const AGENT_PLANNING_MAX_MICROSECONDS: u128 = 30_000_000;
const AGENT_PLANNING_CHAIN_PREFIX: &[u8] = b"nextengine.agent-planning-performance.v1\0";
static NEXT_AGENT_PREPARATION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentPlanningPerformanceReport {
    pub cycles: u64,
    pub elapsed_microseconds: u128,
    pub final_plan_hash: ContentHash,
}

/// An agent-planning workload whose project, RPG snapshot, contact fact, and
/// output buffer have already been prepared.
pub struct PreparedAgentPlanningPerformanceCheck {
    preparation_id: u64,
    directory: ScratchDirectory,
    fixture: NeutralPlayerFixture,
    rpg_snapshot: RpgSnapshotV2,
    fact: RpgPhysicalContactFactV1,
    action: SchemaId,
    chain: Vec<u8>,
    run_started: bool,
}

/// Opaque measured outcome consumed by
/// [`PreparedAgentPlanningPerformanceCheck::finish`] after the caller closes
/// its external measurement window.
pub struct AgentPlanningPerformanceMeasurement {
    preparation_id: u64,
    elapsed_microseconds: u128,
}

pub fn run_agent_planning_performance_check()
-> Result<AgentPlanningPerformanceReport, AgentPlanningPerformanceError> {
    run_agent_planning_performance_check_in(&std::env::temp_dir())
}

pub fn run_agent_planning_performance_check_in(
    scratch_root: &Path,
) -> Result<AgentPlanningPerformanceReport, AgentPlanningPerformanceError> {
    let mut prepared = prepare_agent_planning_performance_check_in(scratch_root)?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

pub(crate) fn run_agent_planning_performance_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<AgentPlanningPerformanceReport, AgentPlanningPerformanceError> {
    let mut prepared = prepare_agent_planning_performance_check_with_scratch(scratch)?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

pub fn prepare_agent_planning_performance_check()
-> Result<PreparedAgentPlanningPerformanceCheck, AgentPlanningPerformanceError> {
    prepare_agent_planning_performance_check_in(&std::env::temp_dir())
}

pub fn prepare_agent_planning_performance_check_in(
    scratch_root: &Path,
) -> Result<PreparedAgentPlanningPerformanceCheck, AgentPlanningPerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| AgentPlanningPerformanceError::Setup(error.to_string()))?;
    prepare_agent_planning_performance_check_with_scratch(&scratch)
}

fn prepare_agent_planning_performance_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<PreparedAgentPlanningPerformanceCheck, AgentPlanningPerformanceError> {
    let directory = scratch
        .create_directory("agent-planning-performance")
        .map_err(|error| AgentPlanningPerformanceError::Setup(error.to_string()))?;
    let fixture = match build_neutral_player_fixture_with_scratch(
        &directory.context(),
        "nextengine.performance.agent",
    ) {
        Ok(fixture) => fixture,
        Err(error) => {
            return Err(finish_failed_preparation(
                directory,
                AgentPlanningPerformanceError::Setup(error.to_string()),
            ));
        }
    };
    let rpg_snapshot = cooked_project_rpg_snapshot(&fixture);
    let (subject_low, subject_high) = if fixture.npc_character_id < fixture.body_id {
        (fixture.npc_character_id, fixture.body_id)
    } else {
        (fixture.body_id, fixture.npc_character_id)
    };
    let fact = RpgPhysicalContactFactV1 {
        gameplay_tick: 7,
        contact_id: PhysicsContactId::from_bytes([0x44; 16]),
        subject_low,
        subject_high,
        physics_checkpoint_revision: 1,
        source_snapshot_hash: ContentHash::from_bytes([0x45; 32]),
        contact_batch_hash: ContentHash::from_bytes([0x46; 32]),
    };
    let action =
        SchemaId::new(CORE_MELEE_ACTION_ID).expect("engine-owned melee action is canonical");
    let hash_bytes = usize::try_from(AGENT_PLANNING_CYCLES)
        .ok()
        .and_then(|cycles| cycles.checked_mul(32))
        .and_then(|bytes| bytes.checked_add(AGENT_PLANNING_CHAIN_PREFIX.len()))
        .ok_or_else(|| {
            AgentPlanningPerformanceError::Setup(
                "agent planning chain reservation overflow".to_owned(),
            )
        });
    let hash_bytes = match hash_bytes {
        Ok(hash_bytes) => hash_bytes,
        Err(error) => return Err(finish_failed_preparation(directory, error)),
    };
    let mut chain = Vec::new();
    if let Err(error) = chain.try_reserve_exact(hash_bytes) {
        return Err(finish_failed_preparation(
            directory,
            AgentPlanningPerformanceError::Setup(format!("reserve agent planning chain: {error}")),
        ));
    }
    chain.extend_from_slice(AGENT_PLANNING_CHAIN_PREFIX);
    let preparation_id = match next_agent_preparation_id() {
        Ok(preparation_id) => preparation_id,
        Err(error) => return Err(finish_failed_preparation(directory, error)),
    };
    Ok(PreparedAgentPlanningPerformanceCheck {
        preparation_id,
        directory,
        fixture,
        rpg_snapshot,
        fact,
        action,
        chain,
        run_started: false,
    })
}

impl PreparedAgentPlanningPerformanceCheck {
    /// Executes only the planner snapshot loop from the legacy measured kernel.
    pub fn run_measured(
        &mut self,
    ) -> Result<AgentPlanningPerformanceMeasurement, AgentPlanningPerformanceError> {
        if self.run_started {
            return Err(AgentPlanningPerformanceError::InvalidMeasurement(
                "measured workload can run only once".to_owned(),
            ));
        }
        self.run_started = true;
        let started = Instant::now();
        for decision_seed in 0..AGENT_PLANNING_CYCLES {
            let request = next_agent::AgentPlanningRequestV1 {
                gameplay_tick: 7,
                world_generation: 2,
                decision_seed,
                source_character_id: self.fixture.npc_character_id,
                target_character_id: self.fixture.body_id,
                allowed_semantic_actions: vec![self.action.clone()],
                motor_state: MotorCapabilityStateV1::ProceduralFallback,
                ai_host_available: false,
                model_available: false,
                rpg_snapshot: &self.rpg_snapshot,
                definitions: &self.fixture.activated_project.rpg_definitions,
                physical_contact_facts: std::slice::from_ref(&self.fact),
            };
            let snapshot = next_agent::build_planner_snapshot_v1(&request)
                .map_err(|error| AgentPlanningPerformanceError::Planner(error.to_string()))?;
            self.chain
                .extend_from_slice(snapshot.snapshot_hash.as_bytes());
        }
        Ok(AgentPlanningPerformanceMeasurement {
            preparation_id: self.preparation_id,
            elapsed_microseconds: started.elapsed().as_micros(),
        })
    }

    /// Validates and materializes the report after the external measurement
    /// window, then removes the prepared cleanup scope on every result path.
    pub fn finish(
        self,
        measurement: Result<AgentPlanningPerformanceMeasurement, AgentPlanningPerformanceError>,
    ) -> Result<AgentPlanningPerformanceReport, AgentPlanningPerformanceError> {
        let result = measurement.and_then(|measurement| self.build_report(measurement));
        drop(self.fixture);
        self.directory.finish(result, |error| {
            AgentPlanningPerformanceError::Setup(error.to_string())
        })
    }

    /// Cancels a prepared workload before measurement and reports cleanup
    /// failures instead of relying on best-effort `Drop` cleanup.
    pub fn cancel(self) -> Result<(), AgentPlanningPerformanceError> {
        drop(self.fixture);
        self.directory.finish(Ok(()), |error| {
            AgentPlanningPerformanceError::Setup(error.to_string())
        })
    }

    fn build_report(
        &self,
        measurement: AgentPlanningPerformanceMeasurement,
    ) -> Result<AgentPlanningPerformanceReport, AgentPlanningPerformanceError> {
        if !self.run_started || measurement.preparation_id != self.preparation_id {
            return Err(AgentPlanningPerformanceError::InvalidMeasurement(
                "measurement does not belong to this completed preparation".to_owned(),
            ));
        }
        if measurement.elapsed_microseconds > AGENT_PLANNING_MAX_MICROSECONDS {
            return Err(AgentPlanningPerformanceError::BudgetExceeded {
                elapsed_microseconds: measurement.elapsed_microseconds,
                maximum_microseconds: AGENT_PLANNING_MAX_MICROSECONDS,
            });
        }
        Ok(AgentPlanningPerformanceReport {
            cycles: AGENT_PLANNING_CYCLES,
            elapsed_microseconds: measurement.elapsed_microseconds,
            final_plan_hash: content_hash_from_bytes(sha256(&self.chain)),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentPlanningPerformanceError {
    Setup(String),
    Planner(String),
    InvalidMeasurement(String),
    BudgetExceeded {
        elapsed_microseconds: u128,
        maximum_microseconds: u128,
    },
}

impl Display for AgentPlanningPerformanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Setup(detail) => write!(formatter, "agent performance setup: {detail}"),
            Self::Planner(detail) => write!(formatter, "agent performance planner: {detail}"),
            Self::InvalidMeasurement(detail) => {
                write!(formatter, "agent performance measurement: {detail}")
            }
            Self::BudgetExceeded {
                elapsed_microseconds,
                maximum_microseconds,
            } => write!(
                formatter,
                "agent planning took {elapsed_microseconds}us, budget is {maximum_microseconds}us"
            ),
        }
    }
}

impl Error for AgentPlanningPerformanceError {}

fn next_agent_preparation_id() -> Result<u64, AgentPlanningPerformanceError> {
    NEXT_AGENT_PREPARATION_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| {
            AgentPlanningPerformanceError::InvalidMeasurement(
                "preparation identity overflow".to_owned(),
            )
        })
}

fn finish_failed_preparation(
    directory: ScratchDirectory,
    error: AgentPlanningPerformanceError,
) -> AgentPlanningPerformanceError {
    match directory.finish(Err::<(), _>(error), |cleanup_error| {
        AgentPlanningPerformanceError::Setup(cleanup_error.to_string())
    }) {
        Err(error) => error,
        Ok(()) => unreachable!("an error result cannot become successful during cleanup"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AgentPlanningPerformanceError, prepare_agent_planning_performance_check_in,
        run_agent_planning_performance_check_in,
    };

    #[test]
    fn planner_hot_path_has_a_bounded_numeric_gate() {
        let report = super::run_agent_planning_performance_check().expect("performance gate");
        assert_eq!(report.cycles, 1_000);
        assert_ne!(
            report.final_plan_hash,
            next_contracts::ids::ContentHash::default()
        );
    }

    #[test]
    fn explicit_preparation_preserves_the_legacy_hash() {
        let scratch = std::env::temp_dir();
        let legacy = run_agent_planning_performance_check_in(&scratch).expect("legacy workload");
        let mut prepared =
            prepare_agent_planning_performance_check_in(&scratch).expect("prepared workload");
        let measurement = prepared.run_measured();
        let explicit = prepared.finish(measurement).expect("finish workload");
        assert_eq!(explicit.cycles, legacy.cycles);
        assert_eq!(explicit.final_plan_hash, legacy.final_plan_hash);
    }

    #[test]
    fn prepared_workload_is_single_use_and_cleans_up() {
        let mut prepared = prepare_agent_planning_performance_check_in(&std::env::temp_dir())
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
        let mut source = prepare_agent_planning_performance_check_in(&std::env::temp_dir())
            .expect("source preparation");
        let source_path = source.directory.path().to_path_buf();
        let measurement = source.run_measured().expect("source measurement");
        let target = prepare_agent_planning_performance_check_in(&std::env::temp_dir())
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
        let prepared = prepare_agent_planning_performance_check_in(&std::env::temp_dir())
            .expect("prepared workload");
        let path = prepared.directory.path().to_path_buf();
        let error = prepared
            .finish(injected_failure())
            .expect_err("injected failure");
        assert!(error.to_string().contains("injected caller failure"));
        assert!(!path.exists());
    }

    fn injected_failure()
    -> Result<super::AgentPlanningPerformanceMeasurement, AgentPlanningPerformanceError> {
        Err(AgentPlanningPerformanceError::Planner(
            "injected caller failure".to_owned(),
        ))
    }
}
