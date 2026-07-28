use super::*;

fn id(byte: u8) -> PersistentId {
    PersistentId::from_bytes([byte; 16])
}

fn schema(value: &str) -> SchemaId {
    SchemaId::new(value).expect("test schema ID is valid")
}

fn definition(byte: u8) -> DefinitionRefV1 {
    DefinitionRefV1::Exact {
        asset_id: AssetId::from_bytes([byte; 16]),
        content_hash: ContentHash::from_bytes([byte; 32]),
    }
}

fn aggregate(id_byte: u8, payload: RpgAggregatePayloadV1) -> RpgAggregateEnvelopeV1 {
    RpgAggregateEnvelopeV1::new(
        id(id_byte),
        1,
        0,
        definition(id_byte),
        ProvenanceBindingV1::Exact(ContentHash::from_bytes([id_byte; 32])),
        payload,
    )
    .expect("test aggregate is valid")
}

#[test]
fn aggregate_snapshot_round_trip_is_byte_exact() {
    let snapshot = RpgSnapshotV2 {
        aggregates: vec![
            aggregate(
                1,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: Some(id(3)),
                    equipment_id: Some(id(4)),
                    resources: vec![],
                    skills: vec![],
                }),
            ),
            aggregate(
                2,
                RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                    quantity: 1,
                    durability: 100,
                    custom_state: Vec::new(),
                }),
            ),
            aggregate(
                3,
                RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                    owner_id: id(1),
                    capacity: 8,
                    item_ids: vec![id(2)],
                    reservations: vec![],
                }),
            ),
            aggregate(
                4,
                RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                    character_id: id(1),
                    slot_policy: definition(9),
                    assignments: vec![],
                }),
            ),
            aggregate(
                5,
                RpgAggregatePayloadV1::Relationship(RelationshipPayloadV1 {
                    source_id: id(1),
                    target_id: id(6),
                    dimensions: vec![RelationshipDimensionV1 {
                        dimension_id: schema("nextengine.relationship.trust"),
                        value: 7,
                    }],
                }),
            ),
        ],
    };

    let bytes = snapshot.canonical_bytes().expect("snapshot encodes");
    let decoded = RpgSnapshotV2::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
        .expect("snapshot decodes");
    assert_eq!(decoded, snapshot);
    assert_eq!(decoded.canonical_bytes().expect("re-encodes"), bytes);
}

#[test]
fn snapshot_requires_canonical_aggregate_order() {
    let mut snapshot = RpgSnapshotV2 {
        aggregates: vec![
            aggregate(
                2,
                RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                    quantity: 1,
                    durability: 1,
                    custom_state: Vec::new(),
                }),
            ),
            aggregate(
                1,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: None,
                    equipment_id: None,
                    resources: vec![],
                    skills: vec![],
                }),
            ),
        ],
    };
    assert_eq!(
        snapshot.validate(),
        Err(RpgContractErrorV1::AggregateOrderInvalid)
    );
    snapshot.aggregates.sort();
    assert!(snapshot.validate().is_ok());
}

#[test]
fn command_rejects_operation_gap_and_unsorted_targets() {
    let command = RpgCommandV1 {
        operations: vec![RpgOperationV1 {
            operation_slot: 1,
            targets: vec![RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::Dialogue,
                persistent_id: id(1),
                expected_revision: 0,
            }],
            definition_policy_hashes: vec![],
            payload: RpgOperationPayloadV1::AdvanceDialogue {
                dialogue_id: id(1),
                expected_node_id: schema("nextengine.dialogue.offer"),
                next_node_id: schema("nextengine.dialogue.accepted"),
            },
        }],
    };
    assert_eq!(
        command.validate(),
        Err(RpgContractErrorV1::OperationOrderInvalid)
    );

    let mut unsorted = command;
    unsorted.operations[0].operation_slot = 0;
    unsorted.operations[0].targets = vec![
        RpgAggregateRefV1 {
            aggregate_kind: RpgAggregateKindV1::Dialogue,
            persistent_id: id(1),
            expected_revision: 0,
        },
        RpgAggregateRefV1 {
            aggregate_kind: RpgAggregateKindV1::Quest,
            persistent_id: id(2),
            expected_revision: 0,
        },
    ];
    assert_eq!(
        unsorted.validate(),
        Err(RpgContractErrorV1::TargetSetInvalid)
    );
}

#[test]
fn command_round_trip_is_byte_exact() {
    let command = RpgCommandV1 {
        operations: vec![
            RpgOperationV1 {
                operation_slot: 0,
                targets: vec![RpgAggregateRefV1 {
                    aggregate_kind: RpgAggregateKindV1::Dialogue,
                    persistent_id: id(1),
                    expected_revision: 0,
                }],
                definition_policy_hashes: vec![],
                payload: RpgOperationPayloadV1::AdvanceDialogue {
                    dialogue_id: id(1),
                    expected_node_id: schema("nextengine.dialogue.offer"),
                    next_node_id: schema("nextengine.dialogue.accepted"),
                },
            },
            RpgOperationV1 {
                operation_slot: 1,
                targets: vec![RpgAggregateRefV1 {
                    aggregate_kind: RpgAggregateKindV1::Quest,
                    persistent_id: id(2),
                    expected_revision: 0,
                }],
                definition_policy_hashes: vec![],
                payload: RpgOperationPayloadV1::TransitionQuest {
                    quest_id: id(2),
                    expected_state_id: schema("nextengine.quest.available"),
                    next_state_id: schema("nextengine.quest.active"),
                },
            },
        ],
    };

    let bytes = command.canonical_payload_bytes().expect("command encodes");
    let decoded =
        RpgCommandV1::from_canonical_payload_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("command decodes");
    assert_eq!(decoded, command);
    assert_eq!(
        decoded.canonical_payload_bytes().expect("re-encodes"),
        bytes
    );
}

#[test]
fn payload_hash_mismatch_is_fail_closed() {
    let mut aggregate = aggregate(
        2,
        RpgAggregatePayloadV1::Item(ItemPayloadV1 {
            quantity: 1,
            durability: 1,
            custom_state: Vec::new(),
        }),
    );
    aggregate.payload_hash = ContentHash::from_bytes([0xff; 32]);
    assert_eq!(
        aggregate.validate(),
        Err(RpgContractErrorV1::PayloadHashMismatch)
    );
}

fn dialogue_plan() -> RpgTransactionPlanV1 {
    let before = aggregate(
        7,
        RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
            speaker_id: id(1),
            listener_id: id(2),
            node_id: schema("nextengine.dialogue.offer"),
        }),
    );
    let after = RpgAggregateEnvelopeV1::new(
        id(7),
        1,
        1,
        definition(7),
        ProvenanceBindingV1::Exact(ContentHash::from_bytes([7; 32])),
        RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
            speaker_id: id(1),
            listener_id: id(2),
            node_id: schema("nextengine.dialogue.accepted"),
        }),
    )
    .expect("after aggregate is valid");
    let policy_hash = ContentHash::from_bytes([8; 32]);
    let operation = RpgOperationV1 {
        operation_slot: 0,
        targets: vec![RpgAggregateRefV1 {
            aggregate_kind: RpgAggregateKindV1::Dialogue,
            persistent_id: id(7),
            expected_revision: 0,
        }],
        definition_policy_hashes: vec![policy_hash],
        payload: RpgOperationPayloadV1::AdvanceDialogue {
            dialogue_id: id(7),
            expected_node_id: schema("nextengine.dialogue.offer"),
            next_node_id: schema("nextengine.dialogue.accepted"),
        },
    };
    let event = RpgEventV1::DialogueAdvanced {
        dialogue_id: id(7),
        node_id: schema("nextengine.dialogue.accepted"),
    };
    let before_state_hash = before.state_hash().expect("before hashes");
    let after_state_hash = after.state_hash().expect("after hashes");
    let mut plan = RpgTransactionPlanV1 {
        causal_command_id: CommandId::from_bytes([1; 16]),
        canonical_command_body_hash: CommandBodyHash::from_bytes([2; 32]),
        project_composition_lock_hash: ContentHash::from_bytes([3; 32]),
        schema_registry_hash: ContentHash::from_bytes([4; 32]),
        definition_policy_hashes: vec![policy_hash],
        validated_fact_hashes: vec![],
        ordered_operations: vec![operation],
        ordered_read_set: vec![RpgReadSetEntryV1 {
            aggregate_ref: RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::Dialogue,
                persistent_id: id(7),
                expected_revision: 0,
            },
            state_hash: before_state_hash,
        }],
        ordered_write_set: vec![RpgWriteSetEntryV1 {
            aggregate_kind: RpgAggregateKindV1::Dialogue,
            persistent_id: id(7),
            before_revision: 0,
            after_revision: 1,
            before_state_hash,
            after_state_hash,
            after,
        }],
        ordered_event_drafts: vec![RpgEventDraftV1 {
            operation_slot: 0,
            event_local_slot: 0,
            event_schema_id: schema(event.schema_id()),
            primary_aggregate_kind: RpgAggregateKindV1::Dialogue,
            primary_persistent_id: id(7),
            event,
        }],
        budget_policy_hash: ContentHash::from_bytes([5; 32]),
        plan_hash: ContentHash::from_bytes([0; 32]),
    };
    plan.plan_hash = plan.recompute_plan_hash().expect("plan hashes");
    plan
}

#[test]
fn transaction_plan_round_trip_is_byte_exact() {
    let plan = dialogue_plan();
    let bytes = plan.canonical_bytes().expect("plan encodes");
    let decoded =
        RpgTransactionPlanV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("plan decodes");
    assert_eq!(decoded, plan);
    assert_eq!(decoded.canonical_bytes().expect("plan re-encodes"), bytes);
}

#[test]
fn transaction_plan_rejects_tampered_hash_and_write_binding() {
    let mut plan = dialogue_plan();
    plan.plan_hash = ContentHash::from_bytes([9; 32]);
    assert_eq!(plan.validate(), Err(RpgContractErrorV1::PlanHashMismatch));

    let mut plan = dialogue_plan();
    plan.ordered_write_set[0].after_revision = 2;
    plan.plan_hash = plan.recompute_plan_hash().expect("tampered plan hashes");
    assert_eq!(plan.validate(), Err(RpgContractErrorV1::PlanWriteInvalid));
}

#[test]
fn transaction_plan_rejects_event_schema_or_slot_mismatch() {
    let mut plan = dialogue_plan();
    plan.ordered_event_drafts[0].event_local_slot = 1;
    plan.plan_hash = plan.recompute_plan_hash().expect("tampered plan hashes");
    assert_eq!(plan.validate(), Err(RpgContractErrorV1::EventOrderInvalid));

    let mut plan = dialogue_plan();
    plan.ordered_event_drafts[0].event_schema_id = schema("nextengine.event.rpg.wrong.v1");
    plan.plan_hash = plan.recompute_plan_hash().expect("tampered plan hashes");
    assert_eq!(plan.validate(), Err(RpgContractErrorV1::EventOrderInvalid));
}

#[test]
fn physical_contact_fact_is_canonical_and_subject_ordered() {
    let fact = RpgPhysicalContactFactV1 {
        gameplay_tick: 7,
        contact_id: PhysicsContactId::from_bytes([1; 16]),
        subject_low: id(1),
        subject_high: id(2),
        physics_checkpoint_revision: 9,
        source_snapshot_hash: ContentHash::from_bytes([3; 32]),
        contact_batch_hash: ContentHash::from_bytes([4; 32]),
    };
    assert_eq!(fact.validate(), Ok(()));
    assert_ne!(
        fact.fact_hash().expect("fact hashes"),
        ContentHash::default()
    );
    assert!(fact.connects(id(2), id(1)));

    let invalid = RpgPhysicalContactFactV1 {
        subject_low: id(2),
        subject_high: id(1),
        ..fact
    };
    assert_eq!(
        invalid.validate(),
        Err(RpgContractErrorV1::PhysicalFactInvalid)
    );
}
