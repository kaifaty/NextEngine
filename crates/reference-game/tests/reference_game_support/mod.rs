use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_runtime::{TickReport, TransactionStage};

pub(super) fn assert_reference_topology_faults_are_typed(
    project: &next_contracts::project::ActivatedProjectV7,
    topology: &next_reference_game::ReferenceWorldTopologyV1,
) {
    let record_index = |project: &next_contracts::project::ActivatedProjectV7,
                        chunk_id: &SchemaId| {
        let asset_id = project
            .world_partition
            .body
            .chunk_bindings
            .iter()
            .find(|binding| &binding.chunk_id == chunk_id)
            .expect("topology chunk binding")
            .chunk_asset
            .asset_id;
        project
            .neutral_records
            .iter()
            .position(|record| record.asset_id == asset_id)
            .expect("topology chunk record")
    };

    let initial_index = record_index(project, topology.initial_chunk_id());
    let target_index = record_index(project, topology.gameplay_target_chunk_id());
    let role_index = |project: &next_contracts::project::ActivatedProjectV7, index: usize| {
        project.neutral_records[index]
            .properties
            .iter()
            .position(|property| property.property_id.as_str() == "nextengine.reference.role")
            .expect("reference role property")
    };

    let mut missing_role = project.clone();
    let initial_role_index = role_index(&missing_role, initial_index);
    missing_role.neutral_records[initial_index].properties[initial_role_index].value_id =
        SchemaId::new("nextengine.reference-alpha.world-chunk.unassigned").expect("role id");
    assert!(matches!(
        next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&missing_role),
        Err(next_reference_game::ReferenceGameError::WorldChunkRoleMissing("initial"))
    ));

    let mut duplicate_role = project.clone();
    let target_role_index = role_index(&duplicate_role, target_index);
    duplicate_role.neutral_records[target_index].properties[target_role_index].value_id =
        SchemaId::new("nextengine.reference-alpha.world-chunk.relay-station").expect("role id");
    assert!(matches!(
        next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&duplicate_role),
        Err(next_reference_game::ReferenceGameError::WorldChunkRoleDuplicate("initial"))
    ));

    let mut wrong_class = project.clone();
    wrong_class.neutral_records[initial_index].kind =
        next_contracts::content::NeutralRecordKindV1::Collider;
    assert!(matches!(
        next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&wrong_class),
        Err(next_reference_game::ReferenceGameError::WorldChunkRecordKindMismatch)
    ));

    let mut missing_record = project.clone();
    missing_record.neutral_records.remove(initial_index);
    assert!(matches!(
        next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&missing_record),
        Err(next_reference_game::ReferenceGameError::WorldChunkRecordMissing)
    ));
}

pub(super) fn assert_world_services_stage_order(reports: &[TickReport]) {
    let world_services_reports = reports
        .iter()
        .filter(|report| {
            report
                .stage_trace
                .iter()
                .any(|entry| entry.stage == TransactionStage::WorldStreamingCommit)
        })
        .collect::<Vec<_>>();
    assert_eq!(world_services_reports.len(), reports.len());
    for report in world_services_reports {
        let ingress = report
            .stage_trace
            .iter()
            .position(|entry| entry.stage == TransactionStage::IngressCommit)
            .expect("ingress commit stage");
        let streaming = report
            .stage_trace
            .iter()
            .position(|entry| entry.stage == TransactionStage::WorldStreamingCommit)
            .expect("world streaming stage");
        let physical = report
            .stage_trace
            .iter()
            .position(|entry| entry.stage == TransactionStage::PhysicalStep)
            .expect("physical stage");
        assert!(ingress < streaming && streaming < physical);
    }
}

pub(super) fn text_resolver(
    package: &next_project::ActivatedProjectPackage,
    locale: &str,
) -> next_presentation::TextCatalogResolverV1 {
    next_presentation::TextCatalogResolverV1::new(package.project.text_catalogs.clone(), locale)
        .expect("text resolver")
}

pub(super) fn assert_live_state_eq(
    left: &next_reference_game::ReferenceLiveStateV2,
    right: &next_reference_game::ReferenceLiveStateV2,
) {
    assert_eq!(left.checkpoint, right.checkpoint);
    assert_eq!(
        left.checkpoint_canonical_components,
        right.checkpoint_canonical_components
    );
    assert_eq!(
        left.world_streaming_snapshot,
        right.world_streaming_snapshot
    );
    assert_eq!(left.ticks, right.ticks);
    assert_eq!(left.events, right.events);
    assert_eq!(left.rpg_events, right.rpg_events);
    assert_eq!(
        left.project_composition_lock_hash,
        right.project_composition_lock_hash
    );
    assert_eq!(left.content_manifest_hash, right.content_manifest_hash);
    assert_eq!(
        left.presentation_input_count,
        right.presentation_input_count
    );
    assert_eq!(left.presentation_snapshot, right.presentation_snapshot);
    assert_eq!(left.driver_recovery, right.driver_recovery);
}

pub(super) fn control_event(
    device_class: &str,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
) -> PlatformEventV1 {
    control_event_with_sequence(device_class, control_path, phase, value, 0)
}

pub(super) fn control_event_with_sequence(
    device_class: &str,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
    source_sequence: u64,
) -> PlatformEventV1 {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(device_class).expect("device class"),
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(control_path).expect("control path"),
        phase,
        value,
        Vec::new(),
        0,
        source_sequence,
    )
    .expect("control");
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.reference-test").expect("source"),
        source_sequence,
        0,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(next_contracts::canonical::sha256(
            b"nextengine.platform.reference-test-capabilities.v1",
        )),
    )
    .expect("platform event")
}

pub(super) fn test_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "nextengine-reference-game-{label}-{}-{}",
        std::process::id(),
        super::TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ))
}
