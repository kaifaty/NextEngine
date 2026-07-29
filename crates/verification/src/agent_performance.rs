use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::Instant;

use next_contracts::agent::MotorCapabilityStateV1;
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PhysicsContactId, SchemaId, content_hash_from_bytes};
use next_contracts::input::CORE_MELEE_ACTION_ID;
use next_contracts::rpg::RpgPhysicalContactFactV1;

use crate::cooked_project_rpg_snapshot;
use crate::player_fixture::build_neutral_player_fixture_with_scratch;
use crate::scratch::ScratchContext;

const AGENT_PLANNING_CYCLES: u64 = 1_000;
const AGENT_PLANNING_MAX_MICROSECONDS: u128 = 30_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentPlanningPerformanceReport {
    pub cycles: u64,
    pub elapsed_microseconds: u128,
    pub final_plan_hash: ContentHash,
}

pub fn run_agent_planning_performance_check()
-> Result<AgentPlanningPerformanceReport, AgentPlanningPerformanceError> {
    run_agent_planning_performance_check_in(&std::env::temp_dir())
}

pub fn run_agent_planning_performance_check_in(
    scratch_root: &Path,
) -> Result<AgentPlanningPerformanceReport, AgentPlanningPerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| AgentPlanningPerformanceError::Setup(error.to_string()))?;
    run_agent_planning_performance_check_with_scratch(&scratch)
}

pub(crate) fn run_agent_planning_performance_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<AgentPlanningPerformanceReport, AgentPlanningPerformanceError> {
    let fixture =
        build_neutral_player_fixture_with_scratch(scratch, "nextengine.performance.agent")
            .map_err(|error| AgentPlanningPerformanceError::Setup(error.to_string()))?;
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
    let started = Instant::now();
    let mut chain = b"nextengine.agent-planning-performance.v1\0".to_vec();
    for decision_seed in 0..AGENT_PLANNING_CYCLES {
        let request = next_agent::AgentPlanningRequestV1 {
            gameplay_tick: 7,
            world_generation: 2,
            decision_seed,
            source_character_id: fixture.npc_character_id,
            target_character_id: fixture.body_id,
            allowed_semantic_actions: vec![action.clone()],
            motor_state: MotorCapabilityStateV1::ProceduralFallback,
            ai_host_available: false,
            model_available: false,
            rpg_snapshot: &rpg_snapshot,
            definitions: &fixture.activated_project.rpg_definitions,
            physical_contact_facts: std::slice::from_ref(&fact),
        };
        let snapshot = next_agent::build_planner_snapshot_v1(&request)
            .map_err(|error| AgentPlanningPerformanceError::Planner(error.to_string()))?;
        chain.extend_from_slice(snapshot.snapshot_hash.as_bytes());
    }
    let elapsed_microseconds = started.elapsed().as_micros();
    if elapsed_microseconds > AGENT_PLANNING_MAX_MICROSECONDS {
        return Err(AgentPlanningPerformanceError::BudgetExceeded {
            elapsed_microseconds,
            maximum_microseconds: AGENT_PLANNING_MAX_MICROSECONDS,
        });
    }
    Ok(AgentPlanningPerformanceReport {
        cycles: AGENT_PLANNING_CYCLES,
        elapsed_microseconds,
        final_plan_hash: content_hash_from_bytes(sha256(&chain)),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentPlanningPerformanceError {
    Setup(String),
    Planner(String),
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

#[cfg(test)]
mod tests {
    #[test]
    fn planner_hot_path_has_a_bounded_numeric_gate() {
        let report = super::run_agent_planning_performance_check().expect("performance gate");
        assert_eq!(report.cycles, 1_000);
        assert_ne!(
            report.final_plan_hash,
            next_contracts::ids::ContentHash::default()
        );
    }
}
