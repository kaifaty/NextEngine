use std::collections::BTreeMap;
use std::sync::Arc;

use next_contracts::{
    CharacterPayloadV1, DialoguePayloadV1, EquipmentPayloadV1, FactionMembershipPayloadV1,
    InventoryPayloadV1, PersistentId, QuestPayloadV1, RelationshipPayloadV1,
    RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1, RpgSnapshotV2,
};

use crate::RpgStateError;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgAggregateKeyV1 {
    pub aggregate_kind: RpgAggregateKindV1,
    pub persistent_id: PersistentId,
}

impl RpgAggregateKeyV1 {
    #[must_use]
    pub const fn new(aggregate_kind: RpgAggregateKindV1, persistent_id: PersistentId) -> Self {
        Self {
            aggregate_kind,
            persistent_id,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RpgState {
    pub(super) aggregates: BTreeMap<RpgAggregateKeyV1, Arc<RpgAggregateEnvelopeV1>>,
}

impl RpgState {
    pub fn from_snapshot(snapshot: RpgSnapshotV2) -> Result<Self, RpgStateError> {
        snapshot.validate().map_err(RpgStateError::Contract)?;
        let mut aggregates = BTreeMap::new();
        for aggregate in snapshot.aggregates {
            let key = RpgAggregateKeyV1::new(aggregate.aggregate_kind, aggregate.persistent_id);
            if aggregates.insert(key, Arc::new(aggregate)).is_some() {
                return Err(RpgStateError::DuplicateAggregate(key));
            }
        }
        let state = Self { aggregates };
        state.validate_cross_references()?;
        Ok(state)
    }

    #[must_use]
    pub fn snapshot(&self) -> RpgSnapshotV2 {
        RpgSnapshotV2 {
            aggregates: self
                .aggregates
                .values()
                .map(|aggregate| aggregate.as_ref().clone())
                .collect(),
        }
    }

    #[must_use]
    pub fn aggregate(
        &self,
        aggregate_kind: RpgAggregateKindV1,
        persistent_id: PersistentId,
    ) -> Option<&RpgAggregateEnvelopeV1> {
        self.aggregates
            .get(&RpgAggregateKeyV1::new(aggregate_kind, persistent_id))
            .map(Arc::as_ref)
    }

    #[must_use]
    pub fn character(&self, id: PersistentId) -> Option<&CharacterPayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Character, id)?.payload {
            RpgAggregatePayloadV1::Character(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn inventory(&self, id: PersistentId) -> Option<&InventoryPayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Inventory, id)?.payload {
            RpgAggregatePayloadV1::Inventory(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn equipment(&self, id: PersistentId) -> Option<&EquipmentPayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Equipment, id)?.payload {
            RpgAggregatePayloadV1::Equipment(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn quest(&self, id: PersistentId) -> Option<&QuestPayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Quest, id)?.payload {
            RpgAggregatePayloadV1::Quest(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn dialogue(&self, id: PersistentId) -> Option<&DialoguePayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Dialogue, id)?.payload {
            RpgAggregatePayloadV1::Dialogue(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn relationship(&self, id: PersistentId) -> Option<&RelationshipPayloadV1> {
        match &self
            .aggregate(RpgAggregateKindV1::Relationship, id)?
            .payload
        {
            RpgAggregatePayloadV1::Relationship(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn interactive_object(
        &self,
        id: PersistentId,
    ) -> Option<&next_contracts::InteractiveObjectPayloadV1> {
        match &self
            .aggregate(RpgAggregateKindV1::InteractiveObject, id)?
            .payload
        {
            RpgAggregatePayloadV1::InteractiveObject(payload) => Some(payload),
            _ => None,
        }
    }

    pub(super) fn validate_cross_references(&self) -> Result<(), RpgStateError> {
        let mut inventory_membership = BTreeMap::new();
        for (key, aggregate) in &self.aggregates {
            aggregate.validate().map_err(RpgStateError::Contract)?;
            match &aggregate.payload {
                RpgAggregatePayloadV1::Character(payload) => {
                    if let Some(inventory_id) = payload.inventory_id {
                        let inventory = self
                            .inventory(inventory_id)
                            .ok_or(RpgStateError::DanglingAggregateReference)?;
                        if inventory.owner_id != key.persistent_id {
                            return Err(RpgStateError::OwnerReferenceMismatch);
                        }
                    }
                    if let Some(equipment_id) = payload.equipment_id {
                        let equipment = self
                            .equipment(equipment_id)
                            .ok_or(RpgStateError::DanglingAggregateReference)?;
                        if equipment.character_id != key.persistent_id {
                            return Err(RpgStateError::OwnerReferenceMismatch);
                        }
                    }
                }
                RpgAggregatePayloadV1::Inventory(payload) => {
                    if self
                        .aggregate(RpgAggregateKindV1::Character, payload.owner_id)
                        .is_none()
                    {
                        return Err(RpgStateError::DanglingAggregateReference);
                    }
                    if usize::try_from(payload.capacity)
                        .map_or(true, |capacity| payload.item_ids.len() > capacity)
                    {
                        return Err(RpgStateError::InventoryCapacityExceeded);
                    }
                    for item_id in &payload.item_ids {
                        if self.aggregate(RpgAggregateKindV1::Item, *item_id).is_none() {
                            return Err(RpgStateError::DanglingAggregateReference);
                        }
                        if inventory_membership
                            .insert(*item_id, key.persistent_id)
                            .is_some()
                        {
                            return Err(RpgStateError::ItemHasMultipleInventories);
                        }
                    }
                }
                RpgAggregatePayloadV1::Equipment(payload) => {
                    let character = self
                        .character(payload.character_id)
                        .ok_or(RpgStateError::DanglingAggregateReference)?;
                    let inventory_id = character
                        .inventory_id
                        .ok_or(RpgStateError::OwnerReferenceMismatch)?;
                    let inventory = self
                        .inventory(inventory_id)
                        .ok_or(RpgStateError::DanglingAggregateReference)?;
                    for assignment in &payload.assignments {
                        if !inventory.item_ids.contains(&assignment.item_id) {
                            return Err(RpgStateError::EquippedItemNotInInventory);
                        }
                    }
                }
                RpgAggregatePayloadV1::Dialogue(payload) => {
                    ensure_kind_exists(self, RpgAggregateKindV1::Character, payload.speaker_id)?;
                    ensure_kind_exists(self, RpgAggregateKindV1::Character, payload.listener_id)?;
                }
                RpgAggregatePayloadV1::FactionMembership(payload) => {
                    validate_membership_references(self, payload)?;
                }
                RpgAggregatePayloadV1::Relationship(payload) => {
                    if !self.has_relationship_endpoint(payload.source_id)
                        || !self.has_relationship_endpoint(payload.target_id)
                    {
                        return Err(RpgStateError::DanglingAggregateReference);
                    }
                }
                RpgAggregatePayloadV1::DivineStanding(payload) => {
                    ensure_kind_exists(
                        self,
                        RpgAggregateKindV1::Character,
                        payload.subject_character_id,
                    )?;
                }
                RpgAggregatePayloadV1::Item(_)
                | RpgAggregatePayloadV1::Quest(_)
                | RpgAggregatePayloadV1::Faction(_)
                | RpgAggregatePayloadV1::InteractiveObject(_) => {}
            }
        }
        Ok(())
    }

    fn has_relationship_endpoint(&self, id: PersistentId) -> bool {
        [RpgAggregateKindV1::Character, RpgAggregateKindV1::Faction]
            .into_iter()
            .any(|kind| self.aggregate(kind, id).is_some())
    }
}

fn validate_membership_references(
    state: &RpgState,
    payload: &FactionMembershipPayloadV1,
) -> Result<(), RpgStateError> {
    ensure_kind_exists(state, RpgAggregateKindV1::Character, payload.character_id)?;
    ensure_kind_exists(state, RpgAggregateKindV1::Faction, payload.faction_id)
}

fn ensure_kind_exists(
    state: &RpgState,
    kind: RpgAggregateKindV1,
    id: PersistentId,
) -> Result<(), RpgStateError> {
    if state.aggregate(kind, id).is_some() {
        Ok(())
    } else {
        Err(RpgStateError::DanglingAggregateReference)
    }
}
