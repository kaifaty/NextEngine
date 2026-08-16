use next_contracts::cognition::{
    StrategicAgentIntentKindV1, StrategicAgentIntentV1, SystemicExecutionFailureV1,
};
use next_contracts::rpg::{
    CommitmentStateV1, RpgAggregateKindV1, RpgAggregateRefV1, RpgCommandV1, RpgOperationPayloadV1,
    RpgOperationV1,
};
use next_contracts::world_activity::{
    WorldActivityCatalogV1, WorldActivitySnapshotV1, WorldActivityStateV1,
};
use next_rpg::RpgState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SystemicRpgDecisionV1 {
    pub command_or_none: Option<(u64, RpgCommandV1)>,
    pub failure_or_none: Option<SystemicExecutionFailureV1>,
}

impl SystemicRpgDecisionV1 {
    const fn none() -> Self {
        Self {
            command_or_none: None,
            failure_or_none: None,
        }
    }

    const fn failure(failure: SystemicExecutionFailureV1) -> Self {
        Self {
            command_or_none: None,
            failure_or_none: Some(failure),
        }
    }
}

pub(super) fn build_systemic_rpg_decision_v1(
    intent_or_none: Option<&StrategicAgentIntentV1>,
    activity_catalog: &WorldActivityCatalogV1,
    activity_snapshot: &WorldActivitySnapshotV1,
    rpg: &RpgState,
    simulation_tick: u64,
) -> SystemicRpgDecisionV1 {
    let Some(intent) = intent_or_none else {
        return SystemicRpgDecisionV1::none();
    };
    if intent.subject_id != activity_catalog.worker_subject_id {
        return SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::JobUnavailable);
    }
    match &intent.kind {
        StrategicAgentIntentKindV1::CommitSocialExchange {
            exchange_hash,
            commitment_id,
        } => build_acceptance(
            intent,
            *exchange_hash,
            *commitment_id,
            activity_catalog,
            rpg,
            simulation_tick,
        ),
        StrategicAgentIntentKindV1::AwaitActivity {
            commitment_id,
            expected_activity_revision,
        } => {
            if *commitment_id != activity_catalog.commitment_id
                || activity_snapshot.record_revision < *expected_activity_revision
            {
                SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::ActivityIncomplete)
            } else {
                SystemicRpgDecisionV1::none()
            }
        }
        StrategicAgentIntentKindV1::SettleSystemicExchange {
            commitment_id,
            expected_activity_revision,
        } => build_settlement(
            intent,
            *commitment_id,
            *expected_activity_revision,
            activity_catalog,
            activity_snapshot,
            rpg,
            simulation_tick,
        ),
        StrategicAgentIntentKindV1::RequestLogicalRoute { .. }
        | StrategicAgentIntentKindV1::HoldPosition => SystemicRpgDecisionV1::none(),
    }
}

fn build_acceptance(
    intent: &StrategicAgentIntentV1,
    exchange_hash: next_contracts::ids::ContentHash,
    commitment_id: next_contracts::ids::PersistentId,
    catalog: &WorldActivityCatalogV1,
    rpg: &RpgState,
    simulation_tick: u64,
) -> SystemicRpgDecisionV1 {
    let Some(commitment) = rpg.commitment(catalog.commitment_id) else {
        return SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::JobUnavailable);
    };
    if commitment.state == CommitmentStateV1::Accepted
        || commitment.state == CommitmentStateV1::Fulfilled
    {
        return SystemicRpgDecisionV1::none();
    }
    if simulation_tick > intent.expiry_tick {
        return SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::IntentStale);
    }
    if commitment_id != catalog.commitment_id
        || catalog.systemic_work.work_exchange.canonical_hash().ok() != Some(exchange_hash)
        || !commitment_matches_catalog(commitment, catalog)
        || commitment.state != CommitmentStateV1::Offered
    {
        return SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::JobUnavailable);
    }
    let Some(target) = aggregate_ref(rpg, RpgAggregateKindV1::Commitment, commitment_id) else {
        return SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::JobUnavailable);
    };
    SystemicRpgDecisionV1 {
        command_or_none: Some((
            0,
            RpgCommandV1 {
                operations: vec![RpgOperationV1 {
                    operation_slot: 0,
                    targets: vec![target],
                    definition_policy_hashes: Vec::new(),
                    payload: RpgOperationPayloadV1::TransitionCommitment {
                        commitment_id,
                        expected_state: CommitmentStateV1::Offered,
                        next_state: CommitmentStateV1::Accepted,
                    },
                }],
            },
        )),
        failure_or_none: None,
    }
}

fn build_settlement(
    intent: &StrategicAgentIntentV1,
    commitment_id: next_contracts::ids::PersistentId,
    expected_activity_revision: u64,
    catalog: &WorldActivityCatalogV1,
    activity: &WorldActivitySnapshotV1,
    rpg: &RpgState,
    simulation_tick: u64,
) -> SystemicRpgDecisionV1 {
    let Some(commitment) = rpg.commitment(catalog.commitment_id) else {
        return SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::JobUnavailable);
    };
    if commitment.state == CommitmentStateV1::Fulfilled {
        return SystemicRpgDecisionV1::none();
    }
    if simulation_tick > intent.expiry_tick {
        return SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::IntentStale);
    }
    if commitment_id != catalog.commitment_id
        || !commitment_matches_catalog(commitment, catalog)
        || commitment.state != CommitmentStateV1::Accepted
    {
        return SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::JobUnavailable);
    }
    if activity.state != WorldActivityStateV1::Completed
        || activity.record_revision != expected_activity_revision
    {
        return SystemicRpgDecisionV1::failure(SystemicExecutionFailureV1::ActivityIncomplete);
    }
    match settlement_command(catalog, rpg) {
        Ok(command) => SystemicRpgDecisionV1 {
            command_or_none: Some((1, command)),
            failure_or_none: None,
        },
        Err(failure) => SystemicRpgDecisionV1::failure(failure),
    }
}

fn settlement_command(
    catalog: &WorldActivityCatalogV1,
    rpg: &RpgState,
) -> Result<RpgCommandV1, SystemicExecutionFailureV1> {
    let profile = &catalog.systemic_work;
    let worker_id = catalog.worker_subject_id;
    let employer_id = profile.employer_character_id;
    let seller_id = profile.seller_character_id;
    let worker = rpg
        .character(worker_id)
        .ok_or(SystemicExecutionFailureV1::JobUnavailable)?;
    let employer = rpg
        .character(employer_id)
        .ok_or(SystemicExecutionFailureV1::JobUnavailable)?;
    let seller = rpg
        .character(seller_id)
        .ok_or(SystemicExecutionFailureV1::JobUnavailable)?;
    if worker.inventory_id != Some(profile.worker_inventory_id)
        || seller.inventory_id != Some(profile.seller_inventory_id)
    {
        return Err(SystemicExecutionFailureV1::InventoryUnavailable);
    }
    let worker_inventory = rpg
        .inventory(profile.worker_inventory_id)
        .ok_or(SystemicExecutionFailureV1::InventoryUnavailable)?;
    let seller_inventory = rpg
        .inventory(profile.seller_inventory_id)
        .ok_or(SystemicExecutionFailureV1::InventoryUnavailable)?;
    let item = rpg
        .aggregate(RpgAggregateKindV1::Item, profile.food_item_id)
        .ok_or(SystemicExecutionFailureV1::InventoryUnavailable)?;
    let next_contracts::rpg::RpgAggregatePayloadV1::Item(item) = &item.payload else {
        return Err(SystemicExecutionFailureV1::InventoryUnavailable);
    };
    if item.quantity != 1
        || seller_inventory
            .item_ids
            .binary_search(&profile.food_item_id)
            .is_err()
        || usize::try_from(worker_inventory.capacity)
            .map_or(true, |capacity| worker_inventory.item_ids.len() >= capacity)
    {
        return Err(SystemicExecutionFailureV1::InventoryUnavailable);
    }

    let employer_currency = resource_value(employer, &profile.currency_resource_id)
        .ok_or(SystemicExecutionFailureV1::InsufficientCurrency)?;
    let worker_currency = resource_value(worker, &profile.currency_resource_id)
        .ok_or(SystemicExecutionFailureV1::InsufficientCurrency)?;
    let seller_currency = resource_value(seller, &profile.currency_resource_id)
        .ok_or(SystemicExecutionFailureV1::InsufficientCurrency)?;
    let hunger = resource_value(worker, &profile.hunger_resource_id)
        .ok_or(SystemicExecutionFailureV1::InventoryUnavailable)?;
    let satiety = resource_value(worker, &profile.satiety_resource_id)
        .ok_or(SystemicExecutionFailureV1::InventoryUnavailable)?;
    let worker_after_wage = worker_currency
        .current_value
        .checked_add(profile.wage_amount)
        .ok_or(SystemicExecutionFailureV1::InsufficientCurrency)?;
    let employer_after_wage = employer_currency
        .current_value
        .checked_sub(profile.wage_amount)
        .ok_or(SystemicExecutionFailureV1::InsufficientCurrency)?;
    let worker_after_trade = worker_after_wage
        .checked_sub(profile.food_price)
        .ok_or(SystemicExecutionFailureV1::InsufficientCurrency)?;
    let seller_after_trade = seller_currency
        .current_value
        .checked_add(profile.food_price)
        .ok_or(SystemicExecutionFailureV1::InsufficientCurrency)?;
    if employer_currency.current_value < profile.wage_amount
        || worker_after_wage < profile.food_price
        || !employer_currency.contains(employer_after_wage)
        || !worker_currency.contains(worker_after_trade)
        || !seller_currency.contains(seller_after_trade)
    {
        return Err(SystemicExecutionFailureV1::InsufficientCurrency);
    }
    let hunger_after = hunger
        .current_value
        .checked_sub(profile.hunger_restore_amount)
        .ok_or(SystemicExecutionFailureV1::InventoryUnavailable)?;
    let satiety_after = satiety
        .current_value
        .checked_add(profile.satiety_gain_amount)
        .ok_or(SystemicExecutionFailureV1::InventoryUnavailable)?;
    if !hunger.contains(hunger_after) || !satiety.contains(satiety_after) {
        return Err(SystemicExecutionFailureV1::InventoryUnavailable);
    }

    let worker_revision = aggregate_revision(rpg, RpgAggregateKindV1::Character, worker_id)?;
    let employer_revision = aggregate_revision(rpg, RpgAggregateKindV1::Character, employer_id)?;
    let seller_revision = aggregate_revision(rpg, RpgAggregateKindV1::Character, seller_id)?;
    let worker_inventory_revision = aggregate_revision(
        rpg,
        RpgAggregateKindV1::Inventory,
        profile.worker_inventory_id,
    )?;
    let seller_inventory_revision = aggregate_revision(
        rpg,
        RpgAggregateKindV1::Inventory,
        profile.seller_inventory_id,
    )?;
    let item_revision = aggregate_revision(rpg, RpgAggregateKindV1::Item, profile.food_item_id)?;
    let commitment_revision =
        aggregate_revision(rpg, RpgAggregateKindV1::Commitment, catalog.commitment_id)?;

    let resource_operation = |slot,
                              source_character_id,
                              character_id,
                              resource_id: &next_contracts::ids::SchemaId,
                              expected_value,
                              delta|
     -> RpgOperationV1 {
        let mut targets = vec![
            RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::Character,
                persistent_id: source_character_id,
                expected_revision: if source_character_id == worker_id {
                    worker_revision
                } else if source_character_id == employer_id {
                    employer_revision
                } else {
                    seller_revision
                },
            },
            RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::Character,
                persistent_id: character_id,
                expected_revision: if character_id == worker_id {
                    worker_revision
                } else if character_id == employer_id {
                    employer_revision
                } else {
                    seller_revision
                },
            },
        ];
        targets.sort_by_key(|target| (target.aggregate_kind, target.persistent_id));
        RpgOperationV1 {
            operation_slot: slot,
            targets,
            definition_policy_hashes: Vec::new(),
            payload: RpgOperationPayloadV1::AdjustCharacterResource {
                source_character_id,
                character_id,
                resource_id: resource_id.clone(),
                expected_value,
                delta,
            },
        }
    };
    let transfer_operation = |slot, source_inventory_id, destination_inventory_id| {
        let mut targets = vec![RpgAggregateRefV1 {
            aggregate_kind: RpgAggregateKindV1::Item,
            persistent_id: profile.food_item_id,
            expected_revision: item_revision,
        }];
        if let Some(id) = source_inventory_id {
            targets.push(RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::Inventory,
                persistent_id: id,
                expected_revision: if id == profile.worker_inventory_id {
                    worker_inventory_revision
                } else {
                    seller_inventory_revision
                },
            });
        }
        if let Some(id) = destination_inventory_id {
            targets.push(RpgAggregateRefV1 {
                aggregate_kind: RpgAggregateKindV1::Inventory,
                persistent_id: id,
                expected_revision: if id == profile.worker_inventory_id {
                    worker_inventory_revision
                } else {
                    seller_inventory_revision
                },
            });
        }
        targets.sort_by_key(|target| (target.aggregate_kind, target.persistent_id));
        RpgOperationV1 {
            operation_slot: slot,
            targets,
            definition_policy_hashes: Vec::new(),
            payload: RpgOperationPayloadV1::TransferItem {
                item_id: profile.food_item_id,
                source_inventory_id,
                destination_inventory_id,
                quantity: 1,
            },
        }
    };
    let command = RpgCommandV1 {
        operations: vec![
            resource_operation(
                0,
                worker_id,
                employer_id,
                &profile.currency_resource_id,
                employer_currency.current_value,
                -profile.wage_amount,
            ),
            resource_operation(
                1,
                employer_id,
                worker_id,
                &profile.currency_resource_id,
                worker_currency.current_value,
                profile.wage_amount,
            ),
            resource_operation(
                2,
                seller_id,
                worker_id,
                &profile.currency_resource_id,
                worker_after_wage,
                -profile.food_price,
            ),
            resource_operation(
                3,
                worker_id,
                seller_id,
                &profile.currency_resource_id,
                seller_currency.current_value,
                profile.food_price,
            ),
            transfer_operation(
                4,
                Some(profile.seller_inventory_id),
                Some(profile.worker_inventory_id),
            ),
            transfer_operation(5, Some(profile.worker_inventory_id), None),
            resource_operation(
                6,
                seller_id,
                worker_id,
                &profile.hunger_resource_id,
                hunger.current_value,
                -profile.hunger_restore_amount,
            ),
            resource_operation(
                7,
                seller_id,
                worker_id,
                &profile.satiety_resource_id,
                satiety.current_value,
                profile.satiety_gain_amount,
            ),
            RpgOperationV1 {
                operation_slot: 8,
                targets: vec![RpgAggregateRefV1 {
                    aggregate_kind: RpgAggregateKindV1::Commitment,
                    persistent_id: catalog.commitment_id,
                    expected_revision: commitment_revision,
                }],
                definition_policy_hashes: Vec::new(),
                payload: RpgOperationPayloadV1::TransitionCommitment {
                    commitment_id: catalog.commitment_id,
                    expected_state: CommitmentStateV1::Accepted,
                    next_state: CommitmentStateV1::Fulfilled,
                },
            },
        ],
    };
    command
        .validate()
        .map_err(|_| SystemicExecutionFailureV1::InventoryUnavailable)?;
    Ok(command)
}

fn commitment_matches_catalog(
    commitment: &next_contracts::rpg::CommitmentPayloadV1,
    catalog: &WorldActivityCatalogV1,
) -> bool {
    let profile = &catalog.systemic_work;
    commitment.issuer_character_id == profile.employer_character_id
        && commitment.recipient_character_id == catalog.worker_subject_id
        && commitment.work_id == catalog.work_id
        && commitment.workplace_node_id == catalog.workplace_node_id
        && commitment.currency_resource_id == profile.currency_resource_id
        && commitment.wage_amount == profile.wage_amount
}

fn aggregate_ref(
    rpg: &RpgState,
    kind: RpgAggregateKindV1,
    id: next_contracts::ids::PersistentId,
) -> Option<RpgAggregateRefV1> {
    Some(RpgAggregateRefV1 {
        aggregate_kind: kind,
        persistent_id: id,
        expected_revision: rpg.aggregate(kind, id)?.revision,
    })
}

fn aggregate_revision(
    rpg: &RpgState,
    kind: RpgAggregateKindV1,
    id: next_contracts::ids::PersistentId,
) -> Result<u64, SystemicExecutionFailureV1> {
    rpg.aggregate(kind, id)
        .map(|aggregate| aggregate.revision)
        .ok_or(SystemicExecutionFailureV1::InventoryUnavailable)
}

fn resource_value<'a>(
    character: &'a next_contracts::rpg::CharacterPayloadV1,
    resource_id: &next_contracts::ids::SchemaId,
) -> Option<&'a next_contracts::rpg::CharacterResourceEntryV1> {
    character
        .resources
        .binary_search_by(|resource| resource.resource_id.cmp(resource_id))
        .ok()
        .and_then(|index| character.resources.get(index))
}

trait ResourceRange {
    fn contains(&self, value: i32) -> bool;
}

impl ResourceRange for next_contracts::rpg::CharacterResourceEntryV1 {
    fn contains(&self, value: i32) -> bool {
        (self.minimum_value..=self.maximum_value).contains(&value)
    }
}
