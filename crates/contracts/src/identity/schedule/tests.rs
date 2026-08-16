use super::*;

#[test]
fn r4b_schedule_materializes_twelve_stages_and_two_ordered_systems() {
    let schedule = ScheduleManifestV1::core_r4b().expect("schedule builds");
    assert_eq!(schedule.stage_order.len(), 12);
    assert_eq!(schedule.systems.len(), 2);
    assert_eq!(schedule.reducers.len(), 0);
    assert_eq!(schedule.shard_plans.len(), 2);
    assert_eq!(schedule.command_admission_barriers.len(), 2);
    let bytes = schedule.canonical_bytes().expect("schedule encodes");
    assert_eq!(
        ScheduleManifestV1::from_canonical_bytes(&bytes, Default::default()),
        Ok(schedule.clone())
    );
    assert_ne!(
        schedule.profile_hash().expect("schedule hashes"),
        ContentHash::default()
    );
}

#[test]
fn r4b_schedule_rejects_registration_order_and_access_ambiguity() {
    let mut schedule = ScheduleManifestV1::core_r4b().expect("schedule builds");
    let system = schedule
        .systems
        .values_mut()
        .next()
        .expect("routine system exists");
    system.access.reads.reverse();
    assert_eq!(
        schedule.validate(),
        Err(IdentityContractError::ScheduleClosureInvalid)
    );
}

#[test]
fn r4c_schedule_adds_agent_planning_as_the_third_ordered_system() {
    let schedule = ScheduleManifestV1::core_r4c().expect("schedule builds");
    assert_eq!(schedule.stage_order.len(), 12);
    assert_eq!(schedule.systems.len(), 3);
    assert_eq!(schedule.shard_plans.len(), 3);
    let cognition = schedule
        .systems
        .get(&SystemId::new(AGENT_COGNITION_SYSTEM_ID).expect("system id"))
        .expect("cognition system is registered");
    assert_eq!(cognition.stage_id, RuntimeStageId::AgentPlanning);
}
