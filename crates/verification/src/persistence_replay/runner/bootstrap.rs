use next_contracts::input::PlayerActionPhaseV1;
use next_physics_api::PhysicsBackendPolicy;
use next_runtime::{PhysicsLaunchOptions, RuntimeState};
use next_world::WorldStreamerV1;

use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;
use crate::{
    player_action_sample, player_equip_use_sample, player_interact_sample, player_melee_sample,
    player_pickup_sample,
};

use super::super::extensions::{verify_luau_state_round_trip, verify_wasm_state_round_trip};
use super::super::rpg_fixture::{initial_rpg_snapshot, rpg_commands};
use super::super::{PersistenceReplayBackend, PersistenceReplayCheckError};
use super::DirectScenario;

pub(super) fn initialize(
    scratch: &ScratchContext,
    backend: PersistenceReplayBackend,
    project_id: &str,
    physx_compatible_profile: bool,
) -> Result<DirectScenario, PersistenceReplayCheckError> {
    let physics_options = match backend {
        PersistenceReplayBackend::Reference => PhysicsLaunchOptions::default(),
        PersistenceReplayBackend::PhysX => {
            PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX)
        }
    };
    let project_package = prepare_fixture_project_package_with_scratch(scratch, project_id)
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare packaged player fixture", error.to_string())
        })?;
    let fixture = if physx_compatible_profile {
        next_reference_game::build_reference_game_session_with_profile(
            project_package.package.project.clone(),
            true,
        )
    } else {
        next_reference_game::build_reference_game_session(project_package.package.project.clone())
    }
    .map_err(|error| PersistenceReplayCheckError::new("build player fixture", error.to_string()))?;
    let luau_package_state_hash = verify_luau_state_round_trip()?;
    let wasm_plugin_state_hash = verify_wasm_state_round_trip()?;
    let initial_rpg = initial_rpg_snapshot(&fixture)?;
    let initial_chunk_id = fixture.world_topology().initial_chunk_id().clone();
    let transition_chunk_id = fixture.world_topology().gameplay_target_chunk_id().clone();
    let content_generation = project_package.package.content_generation.clone();
    let world = WorldStreamerV1::activate(
        fixture.activated_project.clone(),
        content_generation.clone(),
        initial_chunk_id.clone(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("activate world streaming", error.to_string())
    })?;
    let runtime = RuntimeState::with_rpg_snapshot_and_physics_options(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        initial_rpg,
        physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("create runtime", error.to_string()))?;
    let initial_checkpoint = runtime.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("initial checkpoint", error.to_string())
    })?;
    let direct_commands = rpg_commands(fixture.rpg_stream_id, fixture.principal.clone())?;

    Ok(DirectScenario {
        fixture,
        content_generation,
        _project_package: project_package,
        physics_options,
        luau_package_state_hash,
        wasm_plugin_state_hash,
        initial_chunk_id,
        transition_chunk_id,
        world,
        runtime,
        initial_checkpoint,
        direct_commands,
        reports: Vec::new(),
    })
}

pub(super) fn run_pre_save(
    scenario: &mut DirectScenario,
) -> Result<(), PersistenceReplayCheckError> {
    run_forward_movement(scenario)?;
    run_item_and_switch_interactions(scenario)?;
    run_to_npc_contact(scenario)?;
    queue_saved_melee(scenario)
}

fn run_forward_movement(scenario: &mut DirectScenario) -> Result<(), PersistenceReplayCheckError> {
    for sequence in 0_u64..4 {
        let input = player_action_sample(
            &scenario.fixture,
            sequence,
            if sequence == 0 {
                PlayerActionPhaseV1::Started
            } else {
                PlayerActionPhaseV1::Performed
            },
            [0, 32_767],
            Some(
                i64::try_from(sequence).map_err(|error| {
                    PersistenceReplayCheckError::new("movement wall time", error.to_string())
                })? * 1_000,
            ),
        )
        .map_err(|error| PersistenceReplayCheckError::new("movement input", error.to_string()))?;
        scenario
            .runtime
            .enqueue_input_sample(&scenario.fixture.principal, input)
            .map_err(|error| {
                PersistenceReplayCheckError::new("enqueue movement input", error.to_string())
            })?;
        let commands = if sequence == 0 {
            scenario.direct_commands.clone()
        } else {
            Vec::new()
        };
        scenario
            .reports
            .push(scenario.runtime.run_tick(commands).map_err(|error| {
                PersistenceReplayCheckError::new("run pre-save movement", error.to_string())
            })?);
    }
    Ok(())
}

fn run_item_and_switch_interactions(
    scenario: &mut DirectScenario,
) -> Result<(), PersistenceReplayCheckError> {
    let pickup = player_pickup_sample(
        &scenario.fixture,
        4,
        PlayerActionPhaseV1::Started,
        true,
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("pickup input", error.to_string()))?;
    scenario
        .runtime
        .enqueue_input_sample(&scenario.fixture.principal, pickup)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue pickup input", error.to_string())
        })?;
    scenario
        .reports
        .push(scenario.runtime.run_tick([]).map_err(|error| {
            PersistenceReplayCheckError::new("run pickup interaction", error.to_string())
        })?);

    let equip = player_equip_use_sample(
        &scenario.fixture,
        5,
        PlayerActionPhaseV1::Started,
        true,
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("equip input", error.to_string()))?;
    scenario
        .runtime
        .enqueue_input_sample(&scenario.fixture.principal, equip)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue equip input", error.to_string())
        })?;
    scenario
        .reports
        .push(scenario.runtime.run_tick([]).map_err(|error| {
            PersistenceReplayCheckError::new("run equip interaction", error.to_string())
        })?);

    let switch_interaction = player_interact_sample(
        &scenario.fixture,
        6,
        PlayerActionPhaseV1::Started,
        true,
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("switch input", error.to_string()))?;
    scenario
        .runtime
        .enqueue_input_sample(&scenario.fixture.principal, switch_interaction)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue switch input", error.to_string())
        })?;
    scenario
        .reports
        .push(scenario.runtime.run_tick([]).map_err(|error| {
            PersistenceReplayCheckError::new("run switch interaction", error.to_string())
        })?);
    Ok(())
}

fn run_to_npc_contact(scenario: &mut DirectScenario) -> Result<(), PersistenceReplayCheckError> {
    let backward = player_action_sample(
        &scenario.fixture,
        7,
        PlayerActionPhaseV1::Performed,
        [0, -32_767],
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("backward input", error.to_string()))?;
    scenario
        .runtime
        .enqueue_input_sample(&scenario.fixture.principal, backward)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue backward input", error.to_string())
        })?;
    scenario
        .reports
        .push(scenario.runtime.run_tick([]).map_err(|error| {
            PersistenceReplayCheckError::new("run backward movement", error.to_string())
        })?);

    for sequence in 8_u64..11 {
        let right = player_action_sample(
            &scenario.fixture,
            sequence,
            if sequence == 8 {
                PlayerActionPhaseV1::Started
            } else {
                PlayerActionPhaseV1::Performed
            },
            [32_767, 0],
            None,
        )
        .map_err(|error| PersistenceReplayCheckError::new("right input", error.to_string()))?;
        scenario
            .runtime
            .enqueue_input_sample(&scenario.fixture.principal, right)
            .map_err(|error| {
                PersistenceReplayCheckError::new("enqueue right input", error.to_string())
            })?;
        scenario
            .reports
            .push(scenario.runtime.run_tick([]).map_err(|error| {
                PersistenceReplayCheckError::new("run right movement", error.to_string())
            })?);
    }
    if !scenario
        .runtime
        .physics_snapshot()
        .sorted_contact_continuity_states
        .values()
        .any(|contact| {
            contact.participant_low.body_id.subject_id == scenario.fixture.npc_character_id
                || contact.participant_high.body_id.subject_id == scenario.fixture.npc_character_id
        })
    {
        return Err(PersistenceReplayCheckError::condition(
            "save boundary has active NPC contact",
        ));
    }
    Ok(())
}

fn queue_saved_melee(scenario: &mut DirectScenario) -> Result<(), PersistenceReplayCheckError> {
    let queued_melee = player_melee_sample(
        &scenario.fixture,
        11,
        PlayerActionPhaseV1::Started,
        true,
        Some(11_999_999),
    )
    .map_err(|error| PersistenceReplayCheckError::new("melee input", error.to_string()))?;
    scenario
        .runtime
        .enqueue_input_sample(&scenario.fixture.principal, queued_melee)
        .map_err(|error| PersistenceReplayCheckError::new("enqueue melee input", error.to_string()))
}
