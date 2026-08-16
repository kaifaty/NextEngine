use super::*;

fn systemic_work(worker: PersistentId) -> SystemicWorkProfileV1 {
    let employer = PersistentId::from_bytes([0x12; 16]);
    let seller = PersistentId::from_bytes([0x13; 16]);
    let topic = SchemaId::new("nextengine.topic.relay-work").expect("topic");
    let claim = || {
        crate::cognition::SpeechClaimV1::new(
            worker,
            SchemaId::new("nextengine.claim.work-available").expect("predicate"),
            SchemaId::new("nextengine.claim-value.relay-shift").expect("value"),
            COGNITION_Q16_ONE.unsigned_abs(),
            vec![ContentHash::from_bytes([0x41; 32])],
        )
        .expect("claim")
    };
    let ask = StructuredSpeechActV1::new(
        worker,
        employer,
        SpeechActKindV1::Ask,
        topic.clone(),
        0,
        None,
        Some(SpeechActKindV1::Inform),
        None,
        1,
        8,
    )
    .expect("ask");
    let inform = StructuredSpeechActV1::new(
        employer,
        worker,
        SpeechActKindV1::Inform,
        topic.clone(),
        1,
        Some(claim()),
        None,
        Some(ask.act_id),
        1,
        8,
    )
    .expect("inform");
    let offer = StructuredSpeechActV1::new(
        employer,
        worker,
        SpeechActKindV1::Offer,
        topic.clone(),
        2,
        Some(claim()),
        Some(SpeechActKindV1::Accept),
        None,
        1,
        8,
    )
    .expect("offer");
    let accept = StructuredSpeechActV1::new(
        worker,
        employer,
        SpeechActKindV1::Accept,
        topic,
        3,
        None,
        None,
        Some(offer.act_id),
        1,
        8,
    )
    .expect("accept");
    let threat_act = StructuredSpeechActV1::new(
        seller,
        worker,
        SpeechActKindV1::Threaten,
        SchemaId::new("nextengine.topic.market-threat").expect("threat topic"),
        0,
        Some(claim()),
        None,
        None,
        4,
        5,
    )
    .expect("threat");
    SystemicWorkProfileV1 {
        employer_character_id: employer,
        seller_character_id: seller,
        worker_inventory_id: PersistentId::from_bytes([0x14; 16]),
        seller_inventory_id: PersistentId::from_bytes([0x15; 16]),
        food_item_id: PersistentId::from_bytes([0x16; 16]),
        currency_resource_id: SchemaId::new("nextengine.resource.currency").expect("currency"),
        hunger_resource_id: SchemaId::new("nextengine.resource.hunger").expect("hunger"),
        satiety_resource_id: SchemaId::new("nextengine.resource.satiety").expect("satiety"),
        wage_amount: 10,
        food_price: 4,
        hunger_restore_amount: 100,
        satiety_gain_amount: 100,
        listener_trust_q16: COGNITION_Q16_ONE.unsigned_abs(),
        social_action_id: SchemaId::new("nextengine.action.social-work").expect("action"),
        await_activity_action_id: SchemaId::new("nextengine.action.await-work").expect("action"),
        settlement_action_id: SchemaId::new("nextengine.action.settle-work").expect("action"),
        social_ready_fact_id: SchemaId::new("nextengine.fact.social-ready").expect("fact"),
        activity_ready_fact_id: SchemaId::new("nextengine.fact.activity-ready").expect("fact"),
        settlement_ready_fact_id: SchemaId::new("nextengine.fact.settlement-ready").expect("fact"),
        work_exchange: StructuredSpeechExchangeV1 {
            acts: vec![ask, inform, offer, accept],
        },
        threat_act,
    }
}

fn catalog() -> WorldActivityCatalogV1 {
    let worker_subject_id = PersistentId::from_bytes([17; 16]);
    WorldActivityCatalogV1 {
        schema_version: WORLD_ACTIVITY_SCHEMA_VERSION,
        catalog_asset_id: AssetId::from_bytes([0xa1; 16]),
        worker_subject_id,
        commitment_id: PersistentId::from_bytes([0x22; 16]),
        work_id: SchemaId::new("nextengine.work.relay-shift").expect("work"),
        workplace_node_id: SchemaId::new("nextengine.location.frontier").expect("place"),
        work_duration_ticks: 1,
        systemic_work: systemic_work(worker_subject_id),
    }
}

#[test]
fn catalog_snapshot_command_and_event_round_trip() {
    let catalog = catalog();
    let bytes = catalog.canonical_bytes().expect("catalog bytes");
    assert_eq!(
        WorldActivityCatalogV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("catalog decode"),
        catalog
    );
    let snapshot = WorldActivitySnapshotV1::initial(&catalog).expect("snapshot");
    snapshot.validate_against(&catalog, 0).expect("closure");
    let bytes = snapshot.canonical_bytes().expect("snapshot bytes");
    assert_eq!(
        WorldActivitySnapshotV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("snapshot decode"),
        snapshot
    );

    let command = WorldActivityCommandV1::Transition {
        subject_id: catalog.worker_subject_id,
        expected_record_revision: 0,
        catalog_asset_id: catalog.catalog_asset_id,
        catalog_revision: catalog.revision().expect("revision"),
        previous_state: WorldActivityStateV1::Unassigned,
        current_state: WorldActivityStateV1::Assigned,
        boundary_tick: 2,
        evidence: WorldActivityEvidenceV1::AcceptedCommitment {
            commitment_id: catalog.commitment_id,
            commitment_revision: 1,
        },
    };
    let bytes = command.canonical_payload_bytes().expect("command bytes");
    assert_eq!(
        WorldActivityCommandV1::from_canonical_payload_bytes(
            &bytes,
            CanonicalDecodeLimits::default()
        )
        .expect("command decode"),
        command
    );
    let event = WorldActivityChangedV1 {
        subject_id: catalog.worker_subject_id,
        previous_state: WorldActivityStateV1::Unassigned,
        current_state: WorldActivityStateV1::Assigned,
        boundary_tick: 2,
        record_revision: 1,
        evidence_hash: match &command {
            WorldActivityCommandV1::Transition { evidence, .. } => evidence.canonical_hash(),
        },
    };
    let bytes = event.canonical_payload_bytes().expect("event bytes");
    assert_eq!(
        WorldActivityChangedV1::from_canonical_payload_bytes(&bytes).expect("event decode"),
        event
    );
}

#[test]
fn evidence_kind_and_state_transition_must_match() {
    let catalog = catalog();
    let invalid = WorldActivityCommandV1::Transition {
        subject_id: catalog.worker_subject_id,
        expected_record_revision: 0,
        catalog_asset_id: catalog.catalog_asset_id,
        catalog_revision: catalog.revision().expect("revision"),
        previous_state: WorldActivityStateV1::Unassigned,
        current_state: WorldActivityStateV1::Assigned,
        boundary_tick: 2,
        evidence: WorldActivityEvidenceV1::ElapsedWork {
            work_started_tick: 1,
        },
    };
    assert_eq!(
        invalid.validate_shape(),
        Err(WorldActivityContractError::CommandInvalid)
    );
}
