use super::*;

fn catalog() -> WorldRoutineCatalogV1 {
    WorldRoutineCatalogV1 {
        schema_version: 1,
        catalog_asset_id: AssetId::from_bytes([0x41; 16]),
        profile: WorldRoutineProfileV1 {
            schema_version: 1,
            anchor_simulation_tick: 10,
            anchor_world_tick: 100,
            world_ticks_per_simulation_tick_num: 3,
            world_ticks_per_simulation_tick_den: 2,
        },
        routine: WorldRoutineDefinitionV1 {
            schema_version: 1,
            subject_id: PersistentId::from_bytes([0x44; 16]),
            initial_activity: WorldRoutineActivityV1::Duty,
            transition_world_tick: 104,
            next_activity: WorldRoutineActivityV1::Rest,
        },
    }
}

#[test]
fn calendar_projection_and_due_tick_are_exact() {
    let catalog = catalog();
    assert_eq!(catalog.profile.world_tick(10), Ok(100));
    assert_eq!(catalog.profile.world_tick(12), Ok(103));
    assert_eq!(catalog.profile.world_tick(13), Ok(104));
    assert_eq!(catalog.due_simulation_tick(), Ok(13));
}

#[test]
fn catalog_and_snapshot_round_trip_canonically() {
    let catalog = catalog();
    let bytes = catalog.canonical_bytes().expect("catalog encodes");
    assert_eq!(
        WorldRoutineCatalogV1::from_canonical_bytes(&bytes, Default::default()),
        Ok(catalog)
    );
    let snapshot = WorldRoutineSnapshotV1::initial(&catalog).expect("snapshot builds");
    snapshot
        .validate_against(&catalog, 13)
        .expect("save immediately before due tick remains Duty");
    let bytes = snapshot.canonical_bytes().expect("snapshot encodes");
    assert_eq!(
        WorldRoutineSnapshotV1::from_canonical_bytes(&bytes, Default::default()),
        Ok(snapshot)
    );
}

#[test]
fn malformed_ratio_and_activity_pair_fail_closed() {
    let mut invalid_ratio = catalog();
    invalid_ratio.profile.world_ticks_per_simulation_tick_den = 0;
    assert_eq!(
        invalid_ratio.validate(),
        Err(WorldRoutineContractError::CalendarProfileInvalid)
    );
    let mut zero_numerator = catalog();
    zero_numerator.profile.world_ticks_per_simulation_tick_num = 0;
    assert_eq!(
        zero_numerator.validate(),
        Err(WorldRoutineContractError::CalendarProfileInvalid)
    );
    let mut overflowing_due_tick = catalog();
    overflowing_due_tick.profile.anchor_simulation_tick = u64::MAX;
    overflowing_due_tick.profile.anchor_world_tick = 0;
    overflowing_due_tick
        .profile
        .world_ticks_per_simulation_tick_num = 1;
    overflowing_due_tick
        .profile
        .world_ticks_per_simulation_tick_den = 1;
    overflowing_due_tick.routine.transition_world_tick = 1;
    assert_eq!(
        overflowing_due_tick.validate(),
        Err(WorldRoutineContractError::CalendarProfileInvalid)
    );
    let mut invalid_boundary = catalog();
    invalid_boundary.routine.transition_world_tick = invalid_boundary.profile.anchor_world_tick;
    assert_eq!(
        invalid_boundary.validate(),
        Err(WorldRoutineContractError::CalendarProfileInvalid)
    );
    let mut invalid_pair = catalog();
    invalid_pair.routine.next_activity = WorldRoutineActivityV1::Duty;
    assert_eq!(
        invalid_pair.validate(),
        Err(WorldRoutineContractError::ContentInvalid)
    );
    let mut invalid_initial = catalog();
    invalid_initial.routine.initial_activity = WorldRoutineActivityV1::Rest;
    assert_eq!(
        invalid_initial.validate(),
        Err(WorldRoutineContractError::ContentInvalid)
    );
}

#[test]
fn invalid_interaction_binding_fails_closed() {
    let catalog = catalog();
    let wrong_subject = WorldRoutineInteractionBindingV1 {
        interaction_id: SchemaId::new("nextengine.test.interaction").expect("ID"),
        subject_id: PersistentId::from_bytes([0xff; 16]),
        required_activity: WorldRoutineActivityV1::Duty,
    };
    assert_eq!(
        wrong_subject.validate_against(&catalog),
        Err(WorldRoutineContractError::BindingInvalid)
    );
    let wrong_activity = WorldRoutineInteractionBindingV1 {
        subject_id: catalog.routine.subject_id,
        required_activity: WorldRoutineActivityV1::Rest,
        ..wrong_subject
    };
    assert_eq!(
        wrong_activity.validate_against(&catalog),
        Err(WorldRoutineContractError::BindingInvalid)
    );
}

#[test]
fn command_delta_binds_exact_owner_write() {
    let catalog = catalog();
    let snapshot = WorldRoutineSnapshotV1::initial(&catalog).expect("snapshot builds");
    let command = WorldRoutineCommandV1::commit_boundary(&snapshot, &catalog)
        .expect("boundary command builds");
    let payload = command.canonical_payload_bytes().expect("payload encodes");
    assert_eq!(
        WorldRoutineCommandV1::from_canonical_payload_bytes(&payload, Default::default()),
        Ok(command)
    );
    let delta = command.owner_delta_bytes().expect("delta encodes");
    assert!(delta.starts_with(b"nextengine.world-routine-owner-write-set.v1\0"));
    assert_eq!(delta.len(), 138);
}
