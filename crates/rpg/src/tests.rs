use next_contracts::ids::{
    AssetId, CommandBodyHash, CommandId, ContentHash, PhysicsContactId, SchemaId,
};
use next_contracts::rpg::{
    BodyConditionPayloadV1, BodyImpairmentV1, BodyRecoveryStageV1, BodyTreatmentChannelV1,
    CharacterPayloadV1, CharacterResourceEntryV1, CommitmentPayloadV1, CommitmentStateV1,
    DefinitionRefV1, DialoguePayloadV1, EquipmentPayloadV1, InteractiveObjectPayloadV1,
    InventoryPayloadV1, ItemPayloadV1, ProvenanceBindingV1, QuestPayloadV1,
    RelationshipDimensionV1, RelationshipPayloadV1, RpgAggregateEnvelopeV1, RpgAggregateKindV1,
    RpgAggregatePayloadV1, RpgAggregateRefV1, RpgCommandV1, RpgOperationPayloadV1, RpgOperationV1,
    RpgPhysicalContactFactV1, RpgSnapshotV2, SystemicConditionV1,
};
use next_contracts::rpg::{
    CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID, CORE_INTERACTIVE_OBJECT_READY_STATE_ID,
};

use super::{
    RpgPlanBuildError, RpgPlanMaterializeError, RpgPlanningContextV1, RpgState,
    build_transaction_plan_v1, materialize_transaction_plan_v1, recheck_transaction_plan_v1,
};

fn id(value: u8) -> next_contracts::ids::PersistentId {
    next_contracts::ids::PersistentId::from_bytes([value; 16])
}

fn schema(value: &str) -> SchemaId {
    SchemaId::new(value).expect("test schema is valid")
}

fn definition(value: u8) -> DefinitionRefV1 {
    DefinitionRefV1::Exact {
        asset_id: AssetId::from_bytes([value; 16]),
        content_hash: ContentHash::from_bytes([value; 32]),
    }
}

fn aggregate(value: u8, payload: RpgAggregatePayloadV1) -> RpgAggregateEnvelopeV1 {
    RpgAggregateEnvelopeV1::new(
        id(value),
        1,
        0,
        definition(value),
        ProvenanceBindingV1::Exact(ContentHash::from_bytes([value; 32])),
        payload,
    )
    .expect("aggregate is valid")
}

fn fixture() -> RpgState {
    fixture_with_capacity(8)
}

fn fixture_with_capacity(capacity: u32) -> RpgState {
    let mut aggregates = vec![
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
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: None,
                equipment_id: None,
                resources: vec![],
                skills: vec![],
            }),
        ),
        aggregate(
            5,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 100,
                custom_state: vec![],
            }),
        ),
        aggregate(
            3,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: id(1),
                capacity,
                item_ids: vec![id(5)],
                reservations: vec![],
            }),
        ),
        aggregate(
            4,
            RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                character_id: id(1),
                slot_policy: definition(44),
                assignments: vec![],
            }),
        ),
        aggregate(
            6,
            RpgAggregatePayloadV1::Quest(QuestPayloadV1 {
                state_id: schema("rpg.quest.available"),
            }),
        ),
        aggregate(
            7,
            RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
                speaker_id: id(1),
                listener_id: id(2),
                node_id: schema("rpg.dialogue.offer"),
            }),
        ),
        aggregate(
            8,
            RpgAggregatePayloadV1::Relationship(RelationshipPayloadV1 {
                source_id: id(1),
                target_id: id(2),
                dimensions: vec![RelationshipDimensionV1 {
                    dimension_id: schema("rpg.relationship.trust"),
                    value: 0,
                }],
            }),
        ),
        aggregate(
            9,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 100,
                custom_state: vec![],
            }),
        ),
        aggregate(
            10,
            RpgAggregatePayloadV1::InteractiveObject(InteractiveObjectPayloadV1 {
                state_id: schema(CORE_INTERACTIVE_OBJECT_READY_STATE_ID),
                linked_item_id: Some(id(9)),
            }),
        ),
        aggregate(
            11,
            RpgAggregatePayloadV1::Commitment(CommitmentPayloadV1 {
                issuer_character_id: id(2),
                recipient_character_id: id(1),
                work_id: schema("rpg.work.relay-shift"),
                workplace_node_id: schema("rpg.location.relay-station"),
                currency_resource_id: schema("rpg.resource.currency"),
                wage_amount: 12,
                state: CommitmentStateV1::Offered,
            }),
        ),
        aggregate(
            12,
            RpgAggregatePayloadV1::BodyCondition(BodyConditionPayloadV1 {
                character_id: id(1),
                body_schema_hash: ContentHash::from_bytes([20; 32]),
                anatomy_profile_hash: ContentHash::from_bytes([21; 32]),
                region_id: schema("body-region.left-lower-limb"),
                impairment: BodyImpairmentV1::Intact,
                recovery_stage: BodyRecoveryStageV1::Untreated,
                systemic_condition: SystemicConditionV1::Stable,
            }),
        ),
    ];
    aggregates.sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
    RpgState::from_snapshot(RpgSnapshotV2 { aggregates }).expect("fixture is valid")
}

fn context<'a>(active: &'a [ContentHash]) -> RpgPlanningContextV1<'a> {
    context_with_facts(active, &[])
}

fn context_with_facts<'a>(
    active: &'a [ContentHash],
    physical_contact_facts: &'a [RpgPhysicalContactFactV1],
) -> RpgPlanningContextV1<'a> {
    RpgPlanningContextV1 {
        gameplay_tick: 0,
        causal_command_id: CommandId::from_bytes([1; 16]),
        canonical_command_body_hash: CommandBodyHash::from_bytes([2; 32]),
        project_composition_lock_hash: ContentHash::from_bytes([3; 32]),
        schema_registry_hash: ContentHash::from_bytes([4; 32]),
        budget_policy_hash: ContentHash::from_bytes([5; 32]),
        active_definition_policy_hashes: active,
        physical_contact_facts,
    }
}

fn target(kind: RpgAggregateKindV1, value: u8, revision: u64) -> RpgAggregateRefV1 {
    RpgAggregateRefV1 {
        aggregate_kind: kind,
        persistent_id: id(value),
        expected_revision: revision,
    }
}

fn dialogue_quest_command(policy: ContentHash) -> RpgCommandV1 {
    RpgCommandV1 {
        operations: vec![
            RpgOperationV1 {
                operation_slot: 0,
                targets: vec![target(RpgAggregateKindV1::Dialogue, 7, 0)],
                definition_policy_hashes: vec![policy],
                payload: RpgOperationPayloadV1::AdvanceDialogue {
                    dialogue_id: id(7),
                    expected_node_id: schema("rpg.dialogue.offer"),
                    next_node_id: schema("rpg.dialogue.accepted"),
                },
            },
            RpgOperationV1 {
                operation_slot: 1,
                targets: vec![target(RpgAggregateKindV1::Quest, 6, 0)],
                definition_policy_hashes: vec![policy],
                payload: RpgOperationPayloadV1::TransitionQuest {
                    quest_id: id(6),
                    expected_state_id: schema("rpg.quest.available"),
                    next_state_id: schema("rpg.quest.active"),
                },
            },
            RpgOperationV1 {
                operation_slot: 2,
                targets: vec![target(RpgAggregateKindV1::Relationship, 8, 0)],
                definition_policy_hashes: vec![policy],
                payload: RpgOperationPayloadV1::AdjustRelationship {
                    relationship_id: id(8),
                    dimension_id: schema("rpg.relationship.trust"),
                    delta: 7,
                },
            },
        ],
    }
}

fn pickup_command(policy: ContentHash, expected_proxy_state: &str) -> RpgCommandV1 {
    RpgCommandV1 {
        operations: vec![
            RpgOperationV1 {
                operation_slot: 0,
                targets: vec![
                    target(RpgAggregateKindV1::Item, 9, 0),
                    target(RpgAggregateKindV1::Inventory, 3, 0),
                ],
                definition_policy_hashes: vec![policy],
                payload: RpgOperationPayloadV1::TransferItem {
                    item_id: id(9),
                    source_inventory_id: None,
                    destination_inventory_id: Some(id(3)),
                    quantity: 1,
                },
            },
            RpgOperationV1 {
                operation_slot: 1,
                targets: vec![target(RpgAggregateKindV1::InteractiveObject, 10, 0)],
                definition_policy_hashes: vec![policy],
                payload: RpgOperationPayloadV1::TransitionInteractiveObject {
                    object_id: id(10),
                    expected_state_id: schema(expected_proxy_state),
                    next_state_id: schema(CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID),
                },
            },
        ],
    }
}

fn pickup_fact() -> RpgPhysicalContactFactV1 {
    RpgPhysicalContactFactV1 {
        gameplay_tick: 0,
        contact_id: PhysicsContactId::from_bytes([12; 16]),
        subject_low: id(1),
        subject_high: id(10),
        physics_checkpoint_revision: 7,
        source_snapshot_hash: ContentHash::from_bytes([13; 32]),
        contact_batch_hash: ContentHash::from_bytes([14; 32]),
    }
}

fn assign_equipment_command(policy: ContentHash, equipment_revision: u64) -> RpgCommandV1 {
    RpgCommandV1 {
        operations: vec![RpgOperationV1 {
            operation_slot: 0,
            targets: vec![
                target(RpgAggregateKindV1::Item, 5, 0),
                target(RpgAggregateKindV1::Inventory, 3, 0),
                target(RpgAggregateKindV1::Equipment, 4, equipment_revision),
            ],
            definition_policy_hashes: vec![policy],
            payload: RpgOperationPayloadV1::AssignEquipment {
                equipment_id: id(4),
                inventory_id: id(3),
                item_id: id(5),
                slot_id: schema("rpg.equipment.main-hand"),
            },
        }],
    }
}

fn body_condition_command(
    policy: ContentHash,
    revision: u64,
    payload: RpgOperationPayloadV1,
) -> RpgCommandV1 {
    RpgCommandV1 {
        operations: vec![RpgOperationV1 {
            operation_slot: 0,
            targets: vec![target(RpgAggregateKindV1::BodyCondition, 12, revision)],
            definition_policy_hashes: vec![policy],
            payload,
        }],
    }
}

#[test]
fn build_plan_is_pure_and_repeatable() {
    let state = fixture();
    let before = state.snapshot();
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let command = dialogue_quest_command(policy);

    let first =
        build_transaction_plan_v1(&state, &command, context(&active)).expect("first plan builds");
    let second =
        build_transaction_plan_v1(&state, &command, context(&active)).expect("second plan builds");

    assert_eq!(first, second);
    assert_eq!(state.snapshot(), before);
    assert_eq!(first.as_contract().ordered_read_set.len(), 3);
    assert_eq!(first.as_contract().ordered_write_set.len(), 3);
    assert_eq!(first.as_contract().ordered_event_drafts.len(), 3);
}

#[test]
fn multi_aggregate_plan_materializes_atomically() {
    let state = fixture();
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let plan = build_transaction_plan_v1(&state, &dialogue_quest_command(policy), context(&active))
        .expect("plan builds");
    let next = materialize_transaction_plan_v1(&state, &plan).expect("plan materializes");

    assert_eq!(
        next.dialogue(id(7)).expect("dialogue").node_id,
        schema("rpg.dialogue.accepted")
    );
    assert_eq!(
        next.quest(id(6)).expect("quest").state_id,
        schema("rpg.quest.active")
    );
    assert_eq!(
        next.relationship(id(8)).expect("relationship").dimensions[0].value,
        7
    );
    for kind in [
        RpgAggregateKindV1::Dialogue,
        RpgAggregateKindV1::Quest,
        RpgAggregateKindV1::Relationship,
    ] {
        let value = match kind {
            RpgAggregateKindV1::Dialogue => 7,
            RpgAggregateKindV1::Quest => 6,
            RpgAggregateKindV1::Relationship => 8,
            _ => unreachable!(),
        };
        assert_eq!(
            next.aggregate(kind, id(value)).expect("aggregate").revision,
            1
        );
    }
}

#[test]
fn multiple_operations_increment_one_aggregate_once() {
    let state = fixture();
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let command = RpgCommandV1 {
        operations: vec![
            RpgOperationV1 {
                operation_slot: 0,
                targets: vec![target(RpgAggregateKindV1::Relationship, 8, 0)],
                definition_policy_hashes: vec![policy],
                payload: RpgOperationPayloadV1::AdjustRelationship {
                    relationship_id: id(8),
                    dimension_id: schema("rpg.relationship.trust"),
                    delta: 2,
                },
            },
            RpgOperationV1 {
                operation_slot: 1,
                targets: vec![target(RpgAggregateKindV1::Relationship, 8, 0)],
                definition_policy_hashes: vec![policy],
                payload: RpgOperationPayloadV1::AdjustRelationship {
                    relationship_id: id(8),
                    dimension_id: schema("rpg.relationship.trust"),
                    delta: 3,
                },
            },
        ],
    };
    let plan = build_transaction_plan_v1(&state, &command, context(&active)).expect("plan builds");
    assert_eq!(plan.as_contract().ordered_write_set.len(), 1);
    assert_eq!(plan.as_contract().ordered_write_set[0].after_revision, 1);
    let next = materialize_transaction_plan_v1(&state, &plan).expect("plan applies");
    assert_eq!(
        next.relationship(id(8)).expect("relationship").dimensions[0].value,
        5
    );
}

#[test]
fn commitment_becomes_authoritative_only_through_a_valid_transition_plan() {
    let state = fixture();
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let command = RpgCommandV1 {
        operations: vec![RpgOperationV1 {
            operation_slot: 0,
            targets: vec![target(RpgAggregateKindV1::Commitment, 11, 0)],
            definition_policy_hashes: vec![policy],
            payload: RpgOperationPayloadV1::TransitionCommitment {
                commitment_id: id(11),
                expected_state: CommitmentStateV1::Offered,
                next_state: CommitmentStateV1::Accepted,
            },
        }],
    };

    let plan = build_transaction_plan_v1(&state, &command, context(&active))
        .expect("accepted commitment plan builds");
    assert_eq!(
        state.commitment(id(11)).expect("commitment").state,
        CommitmentStateV1::Offered
    );
    let next = materialize_transaction_plan_v1(&state, &plan).expect("commitment plan commits");
    assert_eq!(
        next.commitment(id(11)).expect("commitment").state,
        CommitmentStateV1::Accepted
    );
    assert_eq!(
        next.aggregate(RpgAggregateKindV1::Commitment, id(11))
            .expect("commitment aggregate")
            .revision,
        1
    );

    let invalid = RpgCommandV1 {
        operations: vec![RpgOperationV1 {
            operation_slot: 0,
            targets: vec![target(RpgAggregateKindV1::Commitment, 11, 0)],
            definition_policy_hashes: vec![policy],
            payload: RpgOperationPayloadV1::TransitionCommitment {
                commitment_id: id(11),
                expected_state: CommitmentStateV1::Offered,
                next_state: CommitmentStateV1::Fulfilled,
            },
        }],
    };
    assert!(build_transaction_plan_v1(&state, &invalid, context(&active)).is_err());
    assert_eq!(
        state.commitment(id(11)).expect("commitment").state,
        CommitmentStateV1::Offered
    );
}

#[test]
fn body_condition_follows_damage_stabilize_repair_rehabilitate_sequence() {
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let profile_hash = ContentHash::from_bytes([21; 32]);
    let state = fixture();
    let damage = body_condition_command(
        policy,
        0,
        RpgOperationPayloadV1::ApplyBodyImpairment {
            condition_id: id(12),
            anatomy_profile_hash: profile_hash,
            expected_impairment: BodyImpairmentV1::Intact,
            next_impairment: BodyImpairmentV1::NerveControlLost,
        },
    );
    let plan = build_transaction_plan_v1(&state, &damage, context(&active)).expect("damage plan");
    let state = materialize_transaction_plan_v1(&state, &plan).expect("damage commits");
    let damaged = state.body_condition(id(12)).expect("condition");
    assert_eq!(damaged.impairment, BodyImpairmentV1::NerveControlLost);
    assert_eq!(damaged.recovery_stage, BodyRecoveryStageV1::Untreated);
    assert_eq!(damaged.systemic_condition, SystemicConditionV1::Impaired);

    let treatment_steps = [
        (
            BodyRecoveryStageV1::Untreated,
            BodyRecoveryStageV1::Stabilized,
            BodyTreatmentChannelV1::Medical,
        ),
        (
            BodyRecoveryStageV1::Stabilized,
            BodyRecoveryStageV1::Repaired,
            BodyTreatmentChannelV1::Magical,
        ),
        (
            BodyRecoveryStageV1::Repaired,
            BodyRecoveryStageV1::Rehabilitated,
            BodyTreatmentChannelV1::Medical,
        ),
    ];
    let mut state = state;
    for (ordinal, (expected_stage, next_stage, channel)) in treatment_steps.into_iter().enumerate()
    {
        let revision = u64::try_from(ordinal).expect("small ordinal") + 1;
        let command = body_condition_command(
            policy,
            revision,
            RpgOperationPayloadV1::AdvanceBodyTreatment {
                condition_id: id(12),
                anatomy_profile_hash: profile_hash,
                channel,
                expected_stage,
                next_stage,
            },
        );
        let plan =
            build_transaction_plan_v1(&state, &command, context(&active)).expect("treatment plan");
        state = materialize_transaction_plan_v1(&state, &plan).expect("treatment commits");
    }
    let recovered = state.body_condition(id(12)).expect("condition");
    assert_eq!(recovered.impairment, BodyImpairmentV1::Intact);
    assert_eq!(recovered.recovery_stage, BodyRecoveryStageV1::Rehabilitated);
    assert_eq!(recovered.systemic_condition, SystemicConditionV1::Stable);
    assert_eq!(
        state
            .aggregate(RpgAggregateKindV1::BodyCondition, id(12))
            .expect("aggregate")
            .revision,
        4
    );
}

#[test]
fn stale_revision_and_definition_mismatch_produce_no_plan() {
    let state = fixture();
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let mut command = dialogue_quest_command(policy);
    command.operations[0].targets[0].expected_revision = 1;
    assert!(matches!(
        build_transaction_plan_v1(&state, &command, context(&active)),
        Err(RpgPlanBuildError::RevisionStale { .. })
    ));

    let other = [ContentHash::from_bytes([10; 32])];
    assert_eq!(
        build_transaction_plan_v1(&state, &dialogue_quest_command(policy), context(&other),),
        Err(RpgPlanBuildError::DefinitionMismatch)
    );
}

#[test]
fn transfer_uses_inventory_as_the_only_membership_authority() {
    let state = fixture();
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let command = RpgCommandV1 {
        operations: vec![RpgOperationV1 {
            operation_slot: 0,
            targets: vec![
                target(RpgAggregateKindV1::Item, 5, 0),
                target(RpgAggregateKindV1::Inventory, 3, 0),
            ],
            definition_policy_hashes: vec![policy],
            payload: RpgOperationPayloadV1::TransferItem {
                item_id: id(5),
                source_inventory_id: Some(id(3)),
                destination_inventory_id: None,
                quantity: 1,
            },
        }],
    };
    let plan = build_transaction_plan_v1(&state, &command, context(&active))
        .expect("transfer plan builds");
    assert_eq!(plan.as_contract().ordered_write_set.len(), 1);
    assert_eq!(
        plan.as_contract().ordered_write_set[0].aggregate_kind,
        RpgAggregateKindV1::Inventory
    );
    let next = materialize_transaction_plan_v1(&state, &plan).expect("transfer applies");
    assert!(
        next.inventory(id(3))
            .expect("inventory")
            .item_ids
            .is_empty()
    );
    assert_eq!(
        next.aggregate(RpgAggregateKindV1::Item, id(5))
            .expect("item")
            .revision,
        0
    );
}

#[test]
fn stale_plan_is_rejected_before_materialization() {
    let state = fixture();
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let plan = build_transaction_plan_v1(&state, &dialogue_quest_command(policy), context(&active))
        .expect("plan builds");
    let next = materialize_transaction_plan_v1(&state, &plan).expect("first plan applies");
    assert_eq!(
        recheck_transaction_plan_v1(&next, &plan),
        Err(RpgPlanMaterializeError::PlanStale)
    );
}

#[test]
fn failed_operation_leaves_state_byte_exact() {
    let state = fixture();
    let before = state
        .snapshot()
        .canonical_bytes()
        .expect("snapshot canonicalizes");
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let mut command = dialogue_quest_command(policy);
    if let RpgOperationPayloadV1::TransitionQuest {
        expected_state_id, ..
    } = &mut command.operations[1].payload
    {
        *expected_state_id = schema("rpg.quest.wrong");
    }
    assert_eq!(
        build_transaction_plan_v1(&state, &command, context(&active)),
        Err(RpgPlanBuildError::TransitionInvalid)
    );
    assert_eq!(
        state
            .snapshot()
            .canonical_bytes()
            .expect("snapshot canonicalizes"),
        before
    );
}

#[test]
fn pickup_requires_a_current_revision_bound_contact_fact() {
    let state = fixture();
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let command = pickup_command(policy, CORE_INTERACTIVE_OBJECT_READY_STATE_ID);

    assert_eq!(
        build_transaction_plan_v1(&state, &command, context(&active)),
        Err(RpgPlanBuildError::PhysicalPreconditionMissing)
    );

    let fact = pickup_fact();
    let facts = [fact];
    let plan = build_transaction_plan_v1(&state, &command, context_with_facts(&active, &facts))
        .expect("contact-gated pickup plan builds");
    assert_eq!(
        plan.as_contract().validated_fact_hashes,
        vec![fact.fact_hash().expect("fact hashes")]
    );
    let next = materialize_transaction_plan_v1(&state, &plan).expect("pickup materializes");
    assert_eq!(
        next.inventory(id(3)).expect("inventory").item_ids,
        [id(5), id(9)]
    );
    assert_eq!(
        next.interactive_object(id(10)).expect("proxy").state_id,
        schema(CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID)
    );
}

#[test]
fn pickup_rejects_full_inventory_without_partial_proxy_transition() {
    let state = fixture_with_capacity(1);
    let before = state.snapshot();
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let facts = [pickup_fact()];

    assert_eq!(
        build_transaction_plan_v1(
            &state,
            &pickup_command(policy, CORE_INTERACTIVE_OBJECT_READY_STATE_ID),
            context_with_facts(&active, &facts),
        ),
        Err(RpgPlanBuildError::OwnershipConflict)
    );
    assert_eq!(state.snapshot(), before);
}

#[test]
fn pickup_fault_after_staged_transfer_is_byte_exact() {
    let state = fixture();
    let before = state
        .snapshot()
        .canonical_bytes()
        .expect("snapshot canonicalizes");
    let policy = ContentHash::from_bytes([9; 32]);
    let active = [policy];
    let facts = [pickup_fact()];

    assert_eq!(
        build_transaction_plan_v1(
            &state,
            &pickup_command(policy, "rpg.interactive.wrong"),
            context_with_facts(&active, &facts),
        ),
        Err(RpgPlanBuildError::TransitionInvalid)
    );
    assert_eq!(
        state
            .snapshot()
            .canonical_bytes()
            .expect("snapshot canonicalizes"),
        before
    );
}

#[test]
fn inventory_membership_and_equipment_slot_policy_are_enforced() {
    let state = fixture();
    let policy = ContentHash::from_bytes([44; 32]);
    let active = [policy];
    let plan = build_transaction_plan_v1(
        &state,
        &assign_equipment_command(policy, 0),
        context(&active),
    )
    .expect("first assignment builds");
    let equipped =
        materialize_transaction_plan_v1(&state, &plan).expect("first assignment materializes");

    assert_eq!(
        build_transaction_plan_v1(
            &equipped,
            &assign_equipment_command(policy, 1),
            context(&active),
        ),
        Err(RpgPlanBuildError::OwnershipConflict)
    );

    let wrong_policy = ContentHash::from_bytes([45; 32]);
    assert_eq!(
        build_transaction_plan_v1(
            &state,
            &assign_equipment_command(wrong_policy, 0),
            context(&[wrong_policy]),
        ),
        Err(RpgPlanBuildError::DefinitionMismatch)
    );

    let invalid_source = RpgCommandV1 {
        operations: vec![RpgOperationV1 {
            operation_slot: 0,
            targets: vec![
                target(RpgAggregateKindV1::Item, 9, 0),
                target(RpgAggregateKindV1::Inventory, 3, 0),
            ],
            definition_policy_hashes: vec![],
            payload: RpgOperationPayloadV1::TransferItem {
                item_id: id(9),
                source_inventory_id: Some(id(3)),
                destination_inventory_id: None,
                quantity: 1,
            },
        }],
    };
    assert_eq!(
        build_transaction_plan_v1(&state, &invalid_source, context(&[])),
        Err(RpgPlanBuildError::OwnershipConflict)
    );
}

#[test]
fn character_resource_adjustment_is_atomic_revision_bound_and_bounded() {
    let resource_id = schema("nextengine.test.resource.health");
    let state = RpgState::from_snapshot(RpgSnapshotV2 {
        aggregates: vec![
            aggregate(
                1,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: None,
                    equipment_id: None,
                    resources: vec![],
                    skills: vec![],
                }),
            ),
            aggregate(
                2,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: None,
                    equipment_id: None,
                    resources: vec![CharacterResourceEntryV1 {
                        resource_id: resource_id.clone(),
                        current_value: 100,
                        minimum_value: 0,
                        maximum_value: 100,
                    }],
                    skills: vec![],
                }),
            ),
        ],
    })
    .expect("resource fixture is valid");
    let policy = ContentHash::from_bytes([0x77; 32]);
    let command = |expected_revision, expected_value, delta| RpgCommandV1 {
        operations: vec![RpgOperationV1 {
            operation_slot: 0,
            targets: vec![
                target(RpgAggregateKindV1::Character, 1, 0),
                target(RpgAggregateKindV1::Character, 2, expected_revision),
            ],
            definition_policy_hashes: vec![policy],
            payload: RpgOperationPayloadV1::AdjustCharacterResource {
                source_character_id: id(1),
                character_id: id(2),
                resource_id: resource_id.clone(),
                expected_value,
                delta,
            },
        }],
    };

    let plan = build_transaction_plan_v1(&state, &command(0, 100, -25), context(&[policy]))
        .expect("valid damage builds one atomic plan");
    let committed = materialize_transaction_plan_v1(&state, &plan).expect("valid damage commits");
    assert!(matches!(
        committed
            .character(id(2))
            .and_then(|character| character.resources.first()),
        Some(resource) if resource.current_value == 75
    ));
    assert!(matches!(
        build_transaction_plan_v1(&state, &command(1, 100, -25), context(&[policy])),
        Err(RpgPlanBuildError::RevisionStale { .. })
    ));
    assert_eq!(
        build_transaction_plan_v1(&state, &command(0, 100, -101), context(&[policy])),
        Err(RpgPlanBuildError::TransitionInvalid)
    );
}
