use next_contracts::ids::{AssetId, PersistentId};
use next_contracts::world_routine::{
    WORLD_ROUTINE_SCHEMA_VERSION, WORLD_ROUTINE_SYSTEM_ID, WorldRoutineActivityV1,
    WorldRoutineCatalogV1, WorldRoutineDefinitionV1, WorldRoutineProfileV1,
};

use super::*;
use crate::engine::world_routine::WorldRoutineStageContextV1;

fn catalog() -> WorldRoutineCatalogV1 {
    WorldRoutineCatalogV1 {
        schema_version: WORLD_ROUTINE_SCHEMA_VERSION,
        catalog_asset_id: AssetId::from_bytes([0x94; 16]),
        profile: WorldRoutineProfileV1 {
            schema_version: WORLD_ROUTINE_SCHEMA_VERSION,
            anchor_simulation_tick: 0,
            anchor_world_tick: 0,
            world_ticks_per_simulation_tick_num: 1,
            world_ticks_per_simulation_tick_den: 1,
        },
        routine: WorldRoutineDefinitionV1 {
            schema_version: WORLD_ROUTINE_SCHEMA_VERSION,
            subject_id: PersistentId::from_bytes([0x44; 16]),
            initial_activity: WorldRoutineActivityV1::Duty,
            transition_world_tick: 2,
            next_activity: WorldRoutineActivityV1::Rest,
        },
    }
}

fn routine_stage_fixture() -> (
    fixtures::Fixture,
    next_world::WorldRoutineOwnerV1,
    WorldRoutineStageContextV1,
) {
    let mut fixture = fixture();
    let routine_principal = IssuerPrincipal::InternalSystem(
        SystemId::new(WORLD_ROUTINE_SYSTEM_ID).expect("routine system ID"),
    );
    fixture
        .runtime
        .stream_registry
        .allocate_stream(routine_principal)
        .expect("routine stream");
    let owner = next_world::WorldRoutineOwnerV1::activate(Some(catalog()), 0)
        .expect("routine owner activates");
    let stage = WorldRoutineStageContextV1::capture(&fixture.runtime, &owner)
        .expect("routine stage captures");
    (fixture, owner, stage)
}

#[test]
fn repeated_stage_6_production_fails_before_runtime_publication() {
    let (fixture, _owner, mut stage) = routine_stage_fixture();
    let before_snapshot = fixture.runtime.snapshot();
    let before_archive = fixture.runtime.body_archive().clone();
    let before_ledger = fixture.runtime.command_ledger().clone();

    stage
        .produce_stage_6(2, 0)
        .expect("first stage-6 production");
    let error = stage
        .produce_stage_6(2, 0)
        .expect_err("second stage-6 production is impossible");

    assert_eq!(error.stable_code(), "WORLD_ROUTINE_INTERNAL_INVARIANT");
    assert_eq!(fixture.runtime.snapshot(), before_snapshot);
    assert_eq!(fixture.runtime.body_archive(), &before_archive);
    assert_eq!(fixture.runtime.command_ledger(), &before_ledger);
}

#[test]
fn stage_9_rejects_non_exact_dedicated_candidate_without_owner_or_ledger_mutation() {
    let (fixture, _owner, mut stage) = routine_stage_fixture();
    let before_snapshot = fixture.runtime.snapshot();
    let before_archive = fixture.runtime.body_archive().clone();
    let before_ledger = fixture.runtime.command_ledger().clone();
    let before_routine = stage.committed_projection_or_none().copied();

    stage
        .produce_stage_6(2, 0)
        .expect("stage-6 proposal is captured");
    let mut malformed = stage
        .proposal_for_stage_9(2, 0)
        .expect("stage-9 proposal validates")
        .expect("boundary proposal exists");
    malformed.target_tick = 3;
    let malformed_id = malformed.compute_command_id().expect("candidate ID");
    let error = stage
        .apply_stage_9(&malformed, 2, 0, malformed_id)
        .expect_err("non-exact stage-9 candidate must reject");

    assert_eq!(error.stable_code(), "WORLD_ROUTINE_INTERNAL_INVARIANT");
    assert_eq!(
        stage.committed_projection_or_none().copied(),
        before_routine
    );
    assert_eq!(fixture.runtime.snapshot(), before_snapshot);
    assert_eq!(fixture.runtime.body_archive(), &before_archive);
    assert_eq!(fixture.runtime.command_ledger(), &before_ledger);
}

#[test]
fn stage_9_revision_prediction_is_bound_to_stage_6_bytes() {
    let (_fixture, _owner, mut stage) = routine_stage_fixture();
    stage
        .produce_stage_6(2, 7)
        .expect("stage-6 proposal is captured");

    let error = stage
        .proposal_for_stage_9(2, 8)
        .expect_err("changed phase revision must reject");
    assert_eq!(error.stable_code(), "WORLD_ROUTINE_INTERNAL_INVARIANT");
}
