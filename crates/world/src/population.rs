use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, content_hash_from_bytes};
use next_contracts::world_population::{
    NavigationCapabilityV1, NavigationQueryV1, NavigationRoutePlanV1, PopulationCadenceClassV1,
    PopulationTierV1, WorldNavigationCatalogV1, WorldPopulationCatalogV1, WorldPopulationChangedV1,
    WorldPopulationCommandV1, WorldPopulationContractError, WorldPopulationSnapshotV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopulationNavigationServiceReportV1 {
    pub simulation_tick: u64,
    pub active_due: u32,
    pub near_due: u32,
    pub background_due: u32,
    pub query_count: u32,
    pub route_plan_root: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPopulationOwnerV1 {
    population_catalog_or_none: Option<WorldPopulationCatalogV1>,
    navigation_catalog_or_none: Option<WorldNavigationCatalogV1>,
    snapshot_or_none: Option<WorldPopulationSnapshotV1>,
}

impl WorldPopulationOwnerV1 {
    pub fn empty() -> Self {
        Self {
            population_catalog_or_none: None,
            navigation_catalog_or_none: None,
            snapshot_or_none: None,
        }
    }

    pub fn activate(
        population_catalog: WorldPopulationCatalogV1,
        navigation_catalog: WorldNavigationCatalogV1,
        next_simulation_tick: u64,
    ) -> Result<Self, WorldPopulationOwnerError> {
        if next_simulation_tick != 0 {
            return Err(WorldPopulationOwnerError::ProjectMismatch);
        }
        let snapshot =
            WorldPopulationSnapshotV1::initial(&population_catalog, &navigation_catalog)?;
        let value = Self {
            population_catalog_or_none: Some(population_catalog),
            navigation_catalog_or_none: Some(navigation_catalog),
            snapshot_or_none: Some(snapshot),
        };
        value.validate(next_simulation_tick)?;
        Ok(value)
    }

    pub fn restore(
        population_catalog: WorldPopulationCatalogV1,
        navigation_catalog: WorldNavigationCatalogV1,
        snapshot: WorldPopulationSnapshotV1,
        next_simulation_tick: u64,
    ) -> Result<Self, WorldPopulationOwnerError> {
        let value = Self {
            population_catalog_or_none: Some(population_catalog),
            navigation_catalog_or_none: Some(navigation_catalog),
            snapshot_or_none: Some(snapshot),
        };
        value.validate(next_simulation_tick)?;
        Ok(value)
    }

    #[must_use]
    pub const fn population_catalog_or_none(&self) -> Option<&WorldPopulationCatalogV1> {
        self.population_catalog_or_none.as_ref()
    }

    #[must_use]
    pub const fn navigation_catalog_or_none(&self) -> Option<&WorldNavigationCatalogV1> {
        self.navigation_catalog_or_none.as_ref()
    }

    #[must_use]
    pub const fn snapshot_or_none(&self) -> Option<&WorldPopulationSnapshotV1> {
        self.snapshot_or_none.as_ref()
    }

    pub fn validate(&self, next_simulation_tick: u64) -> Result<(), WorldPopulationOwnerError> {
        match (
            &self.population_catalog_or_none,
            &self.navigation_catalog_or_none,
            &self.snapshot_or_none,
        ) {
            (Some(population), Some(navigation), Some(snapshot)) => {
                snapshot.validate_against(population, navigation, next_simulation_tick)?;
                Ok(())
            }
            (None, None, None) => Ok(()),
            _ => Err(WorldPopulationOwnerError::ProjectMismatch),
        }
    }

    pub fn route(
        &self,
        query: &NavigationQueryV1,
    ) -> Result<NavigationRoutePlanV1, WorldPopulationOwnerError> {
        route_query(
            self.navigation_catalog_or_none
                .as_ref()
                .ok_or(WorldPopulationOwnerError::ProjectMismatch)?,
            query,
        )
        .map_err(Into::into)
    }

    pub fn service_tick(
        &self,
        simulation_tick: u64,
    ) -> Result<PopulationNavigationServiceReportV1, WorldPopulationOwnerError> {
        let snapshot = self
            .snapshot_or_none
            .as_ref()
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        self.service_snapshot(snapshot, simulation_tick)
    }

    pub fn service_snapshot(
        &self,
        snapshot: &WorldPopulationSnapshotV1,
        simulation_tick: u64,
    ) -> Result<PopulationNavigationServiceReportV1, WorldPopulationOwnerError> {
        let population = self
            .population_catalog_or_none
            .as_ref()
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        let navigation = self
            .navigation_catalog_or_none
            .as_ref()
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        snapshot.validate_against(population, navigation, simulation_tick)?;
        let mut active_due = 0_u32;
        let mut near_due = 0_u32;
        let mut background_due = 0_u32;
        let mut query_count = 0_u32;
        let mut root_preimage = b"nextengine.population-navigation-service.v1\0".to_vec();
        root_preimage.extend_from_slice(&simulation_tick.to_le_bytes());
        for definition in &population.records {
            if simulation_tick % definition.cadence_class.period_ticks() != definition.cadence_phase
            {
                continue;
            }
            match definition.cadence_class {
                PopulationCadenceClassV1::Active => {
                    active_due = active_due
                        .checked_add(1)
                        .ok_or(WorldPopulationOwnerError::CountOverflow)?;
                }
                PopulationCadenceClassV1::Near => {
                    near_due = near_due
                        .checked_add(1)
                        .ok_or(WorldPopulationOwnerError::CountOverflow)?;
                }
                PopulationCadenceClassV1::Background => {
                    background_due = background_due
                        .checked_add(1)
                        .ok_or(WorldPopulationOwnerError::CountOverflow)?;
                }
            }
            let record = snapshot
                .record(definition.subject_id)
                .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
            let query = NavigationQueryV1 {
                schema_version: next_contracts::world_population::WORLD_POPULATION_SCHEMA_VERSION,
                catalog_asset_id: navigation.catalog_asset_id,
                graph_revision: navigation.revision()?,
                start_node_id: record.current_node_id.clone(),
                goal_node_id: definition.navigation_goal_node_id.clone(),
                capability: NavigationCapabilityV1::AbstractTransfer,
            };
            let plan = route_query(navigation, &query)?;
            root_preimage.extend_from_slice(definition.subject_id.as_bytes());
            root_preimage.extend_from_slice(plan.plan_hash.as_bytes());
            query_count = query_count
                .checked_add(1)
                .ok_or(WorldPopulationOwnerError::CountOverflow)?;
        }
        root_preimage.extend_from_slice(&active_due.to_le_bytes());
        root_preimage.extend_from_slice(&near_due.to_le_bytes());
        root_preimage.extend_from_slice(&background_due.to_le_bytes());
        root_preimage.extend_from_slice(&query_count.to_le_bytes());
        Ok(PopulationNavigationServiceReportV1 {
            simulation_tick,
            active_due,
            near_due,
            background_due,
            query_count,
            route_plan_root: content_hash_from_bytes(sha256(&root_preimage)),
        })
    }

    pub fn expected_command_for_snapshot(
        &self,
        snapshot: &WorldPopulationSnapshotV1,
        simulation_tick: u64,
    ) -> Result<Option<WorldPopulationCommandV1>, WorldPopulationOwnerError> {
        let population = self
            .population_catalog_or_none
            .as_ref()
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        let navigation = self
            .navigation_catalog_or_none
            .as_ref()
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        snapshot.validate_against(population, navigation, simulation_tick)?;
        let start = population.courier_transition_start_tick;
        let Some(ordinal) = simulation_tick
            .checked_sub(start)
            .filter(|ordinal| *ordinal < 7)
        else {
            return Ok(None);
        };
        let record = snapshot
            .record(population.courier_subject_id)
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        let catalog_revision = population.revision(navigation)?;
        let command = match ordinal {
            0 => tier_command(
                population,
                record,
                catalog_revision,
                simulation_tick,
                PopulationTierV1::Dormant,
                PopulationTierV1::Abstract,
            )?,
            1 => self.abstract_transfer_command(
                snapshot,
                population.courier_subject_id,
                simulation_tick,
            )?,
            2 => tier_command(
                population,
                record,
                catalog_revision,
                simulation_tick,
                PopulationTierV1::Abstract,
                PopulationTierV1::Simulated,
            )?,
            3 => tier_command(
                population,
                record,
                catalog_revision,
                simulation_tick,
                PopulationTierV1::Simulated,
                PopulationTierV1::Active,
            )?,
            4 => tier_command(
                population,
                record,
                catalog_revision,
                simulation_tick,
                PopulationTierV1::Active,
                PopulationTierV1::Simulated,
            )?,
            5 => tier_command(
                population,
                record,
                catalog_revision,
                simulation_tick,
                PopulationTierV1::Simulated,
                PopulationTierV1::Abstract,
            )?,
            6 => tier_command(
                population,
                record,
                catalog_revision,
                simulation_tick,
                PopulationTierV1::Abstract,
                PopulationTierV1::Dormant,
            )?,
            _ => unreachable!("ordinal is bounded to seven actions"),
        };
        Ok(Some(command))
    }

    pub fn abstract_transfer_command(
        &self,
        snapshot: &WorldPopulationSnapshotV1,
        subject_id: PersistentId,
        simulation_tick: u64,
    ) -> Result<WorldPopulationCommandV1, WorldPopulationOwnerError> {
        let population = self
            .population_catalog_or_none
            .as_ref()
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        let navigation = self
            .navigation_catalog_or_none
            .as_ref()
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        snapshot.validate_against(population, navigation, simulation_tick)?;
        let definition = population
            .definition(subject_id)
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        let record = snapshot
            .record(subject_id)
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        if record.tier == PopulationTierV1::Active {
            return Err(WorldPopulationContractError::PhysicalTraversalRequired.into());
        }
        if record.tier != PopulationTierV1::Abstract {
            return Err(WorldPopulationContractError::CommandInvalid.into());
        }
        let query = NavigationQueryV1 {
            schema_version: next_contracts::world_population::WORLD_POPULATION_SCHEMA_VERSION,
            catalog_asset_id: navigation.catalog_asset_id,
            graph_revision: navigation.revision()?,
            start_node_id: record.current_node_id.clone(),
            goal_node_id: definition.navigation_goal_node_id.clone(),
            capability: NavigationCapabilityV1::AbstractTransfer,
        };
        let plan = route_query(navigation, &query)?;
        let target_region_id = navigation
            .region_for_node(&definition.navigation_goal_node_id)
            .ok_or(WorldPopulationContractError::NodeUnavailable)?
            .clone();
        let command = WorldPopulationCommandV1::CommitAbstractTransfer {
            subject_id,
            expected_record_revision: record.record_revision,
            catalog_asset_id: population.catalog_asset_id,
            catalog_revision: population.revision(navigation)?,
            navigation_catalog_asset_id: navigation.catalog_asset_id,
            navigation_catalog_revision: navigation.revision()?,
            route_plan_hash: plan.plan_hash,
            source_region_id: record.current_region_id.clone(),
            source_node_id: record.current_node_id.clone(),
            target_region_id,
            target_node_id: definition.navigation_goal_node_id.clone(),
        };
        command.validate_shape()?;
        Ok(command)
    }

    pub fn apply_command_to_snapshot(
        &self,
        snapshot: &mut WorldPopulationSnapshotV1,
        command: &WorldPopulationCommandV1,
        simulation_tick: u64,
    ) -> Result<WorldPopulationChangedV1, WorldPopulationOwnerError> {
        if self
            .expected_command_for_snapshot(snapshot, simulation_tick)?
            .as_ref()
            != Some(command)
        {
            return Err(WorldPopulationOwnerError::CommandMismatch);
        }
        let population = self
            .population_catalog_or_none
            .as_ref()
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        let navigation = self
            .navigation_catalog_or_none
            .as_ref()
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        let record = snapshot
            .records
            .binary_search_by_key(&command.subject_id(), |record| record.subject_id)
            .ok()
            .and_then(|index| snapshot.records.get_mut(index))
            .ok_or(WorldPopulationOwnerError::ProjectMismatch)?;
        let next_revision = record
            .record_revision
            .checked_add(1)
            .ok_or(WorldPopulationContractError::RevisionExhausted)?;
        let event = match command {
            WorldPopulationCommandV1::TransitionTier {
                previous_tier,
                current_tier,
                transition_tick,
                ..
            } => {
                record.tier = *current_tier;
                WorldPopulationChangedV1::TierTransitioned {
                    subject_id: record.subject_id,
                    previous_tier: *previous_tier,
                    current_tier: *current_tier,
                    transition_tick: *transition_tick,
                    record_revision: next_revision,
                }
            }
            WorldPopulationCommandV1::CommitAbstractTransfer {
                source_region_id,
                source_node_id,
                target_region_id,
                target_node_id,
                route_plan_hash,
                ..
            } => {
                record.current_region_id = target_region_id.clone();
                record.current_node_id = target_node_id.clone();
                WorldPopulationChangedV1::AbstractTransferred {
                    subject_id: record.subject_id,
                    source_region_id: source_region_id.clone(),
                    source_node_id: source_node_id.clone(),
                    target_region_id: target_region_id.clone(),
                    target_node_id: target_node_id.clone(),
                    route_plan_hash: *route_plan_hash,
                    record_revision: next_revision,
                }
            }
        };
        record.record_revision = next_revision;
        snapshot.validate_against(
            population,
            navigation,
            simulation_tick
                .checked_add(1)
                .ok_or(WorldPopulationOwnerError::TickExhausted)?,
        )?;
        Ok(event)
    }

    pub fn prepare_publication(
        &self,
        next_snapshot_or_none: Option<WorldPopulationSnapshotV1>,
        service_report_or_none: Option<PopulationNavigationServiceReportV1>,
        next_simulation_tick: u64,
    ) -> Result<PreparedWorldPopulationPublicationV1, WorldPopulationOwnerError> {
        validate_snapshot_set(
            self.population_catalog_or_none.as_ref(),
            self.navigation_catalog_or_none.as_ref(),
            next_snapshot_or_none.as_ref(),
            next_simulation_tick,
        )?;
        if self.snapshot_or_none.is_some() != service_report_or_none.is_some()
            || service_report_or_none.as_ref().is_some_and(|report| {
                report.simulation_tick.checked_add(1) != Some(next_simulation_tick)
            })
        {
            return Err(WorldPopulationOwnerError::ProjectMismatch);
        }
        Ok(PreparedWorldPopulationPublicationV1 {
            base: WorldPopulationOwnerBaseV1::capture(self)?,
            next_snapshot_or_none,
            service_report_or_none,
            next_simulation_tick,
        })
    }

    pub fn validate_prepared_publication(
        &self,
        prepared: PreparedWorldPopulationPublicationV1,
    ) -> Result<ValidatedWorldPopulationPublicationV1, WorldPopulationOwnerError> {
        self.preflight_base(&prepared.base)?;
        validate_snapshot_set(
            self.population_catalog_or_none.as_ref(),
            self.navigation_catalog_or_none.as_ref(),
            prepared.next_snapshot_or_none.as_ref(),
            prepared.next_simulation_tick,
        )?;
        Ok(ValidatedWorldPopulationPublicationV1 {
            base: prepared.base,
            next_snapshot_or_none: prepared.next_snapshot_or_none,
            service_report_or_none: prepared.service_report_or_none,
        })
    }

    pub fn preflight_validated_publication(
        &self,
        validated: &ValidatedWorldPopulationPublicationV1,
    ) -> Result<(), WorldPopulationOwnerError> {
        self.preflight_base(&validated.base)
    }

    pub fn commit_validated_publication(
        &mut self,
        validated: ValidatedWorldPopulationPublicationV1,
    ) {
        self.snapshot_or_none = validated.next_snapshot_or_none;
    }

    fn preflight_base(
        &self,
        base: &WorldPopulationOwnerBaseV1,
    ) -> Result<(), WorldPopulationOwnerError> {
        if WorldPopulationOwnerBaseV1::capture(self)? != *base {
            return Err(WorldPopulationOwnerError::PublicationStale);
        }
        Ok(())
    }
}

fn tier_command(
    population: &WorldPopulationCatalogV1,
    record: &next_contracts::world_population::WorldPopulationRecordV1,
    catalog_revision: ContentHash,
    transition_tick: u64,
    previous_tier: PopulationTierV1,
    current_tier: PopulationTierV1,
) -> Result<WorldPopulationCommandV1, WorldPopulationOwnerError> {
    if record.tier != previous_tier {
        return Err(WorldPopulationOwnerError::CommandMismatch);
    }
    let command = WorldPopulationCommandV1::TransitionTier {
        subject_id: record.subject_id,
        expected_record_revision: record.record_revision,
        catalog_asset_id: population.catalog_asset_id,
        catalog_revision,
        transition_tick,
        previous_tier,
        current_tier,
    };
    command.validate_shape()?;
    Ok(command)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldPopulationOwnerBaseV1 {
    population_revision_or_none: Option<ContentHash>,
    navigation_revision_or_none: Option<ContentHash>,
    snapshot_or_none: Option<WorldPopulationSnapshotV1>,
}

impl WorldPopulationOwnerBaseV1 {
    fn capture(owner: &WorldPopulationOwnerV1) -> Result<Self, WorldPopulationOwnerError> {
        let population_revision_or_none = match (
            owner.population_catalog_or_none.as_ref(),
            owner.navigation_catalog_or_none.as_ref(),
        ) {
            (Some(population), Some(navigation)) => Some(population.revision(navigation)?),
            (None, None) => None,
            _ => return Err(WorldPopulationOwnerError::ProjectMismatch),
        };
        Ok(Self {
            population_revision_or_none,
            navigation_revision_or_none: owner
                .navigation_catalog_or_none
                .as_ref()
                .map(WorldNavigationCatalogV1::revision)
                .transpose()?,
            snapshot_or_none: owner.snapshot_or_none.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedWorldPopulationPublicationV1 {
    base: WorldPopulationOwnerBaseV1,
    next_snapshot_or_none: Option<WorldPopulationSnapshotV1>,
    service_report_or_none: Option<PopulationNavigationServiceReportV1>,
    next_simulation_tick: u64,
}

impl PreparedWorldPopulationPublicationV1 {
    #[must_use]
    pub const fn snapshot_or_none(&self) -> Option<&WorldPopulationSnapshotV1> {
        self.next_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn service_report_or_none(&self) -> Option<&PopulationNavigationServiceReportV1> {
        self.service_report_or_none.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedWorldPopulationPublicationV1 {
    base: WorldPopulationOwnerBaseV1,
    next_snapshot_or_none: Option<WorldPopulationSnapshotV1>,
    service_report_or_none: Option<PopulationNavigationServiceReportV1>,
}

impl ValidatedWorldPopulationPublicationV1 {
    #[must_use]
    pub const fn snapshot_or_none(&self) -> Option<&WorldPopulationSnapshotV1> {
        self.next_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn service_report_or_none(&self) -> Option<&PopulationNavigationServiceReportV1> {
        self.service_report_or_none.as_ref()
    }
}

fn validate_snapshot_set(
    population: Option<&WorldPopulationCatalogV1>,
    navigation: Option<&WorldNavigationCatalogV1>,
    snapshot: Option<&WorldPopulationSnapshotV1>,
    next_simulation_tick: u64,
) -> Result<(), WorldPopulationOwnerError> {
    match (population, navigation, snapshot) {
        (Some(population), Some(navigation), Some(snapshot)) => {
            snapshot.validate_against(population, navigation, next_simulation_tick)?;
            Ok(())
        }
        (None, None, None) => Ok(()),
        _ => Err(WorldPopulationOwnerError::ProjectMismatch),
    }
}

pub fn route_query(
    catalog: &WorldNavigationCatalogV1,
    query: &NavigationQueryV1,
) -> Result<NavigationRoutePlanV1, WorldPopulationContractError> {
    catalog.validate()?;
    if query.schema_version != next_contracts::world_population::WORLD_POPULATION_SCHEMA_VERSION
        || query.catalog_asset_id != catalog.catalog_asset_id
        || query.graph_revision != catalog.revision()?
    {
        return Err(WorldPopulationContractError::RouteStale);
    }
    if catalog.node(&query.start_node_id).is_none() || catalog.node(&query.goal_node_id).is_none() {
        return Err(WorldPopulationContractError::NodeUnavailable);
    }
    if query.start_node_id == query.goal_node_id {
        return NavigationRoutePlanV1::new(query.clone(), vec![query.start_node_id.clone()], 0);
    }

    let mut distances = BTreeMap::<SchemaId, u64>::new();
    let mut predecessors = BTreeMap::<SchemaId, SchemaId>::new();
    let mut pending = BinaryHeap::new();
    distances.insert(query.start_node_id.clone(), 0);
    pending.push(Reverse((0_u64, query.start_node_id.clone())));
    while let Some(Reverse((distance, node_id))) = pending.pop() {
        if distances.get(&node_id).copied() != Some(distance) {
            continue;
        }
        if node_id == query.goal_node_id {
            break;
        }
        for edge in &catalog.edges {
            if edge.capability != query.capability {
                continue;
            }
            let neighbour = if edge.node_low == node_id {
                Some(&edge.node_high)
            } else if edge.node_high == node_id {
                Some(&edge.node_low)
            } else {
                None
            };
            let Some(neighbour) = neighbour else {
                continue;
            };
            let candidate = distance
                .checked_add(u64::from(edge.cost))
                .ok_or(WorldPopulationContractError::RoutePlanInvalid)?;
            if distances
                .get(neighbour)
                .is_none_or(|current| candidate < *current)
            {
                distances.insert(neighbour.clone(), candidate);
                predecessors.insert(neighbour.clone(), node_id.clone());
                pending.push(Reverse((candidate, neighbour.clone())));
            }
        }
    }
    let total_cost = distances
        .get(&query.goal_node_id)
        .copied()
        .ok_or(WorldPopulationContractError::RouteUnavailable)?;
    let mut reversed = vec![query.goal_node_id.clone()];
    while reversed.last() != Some(&query.start_node_id) {
        let previous = predecessors
            .get(
                reversed
                    .last()
                    .ok_or(WorldPopulationContractError::RoutePlanInvalid)?,
            )
            .ok_or(WorldPopulationContractError::RoutePlanInvalid)?
            .clone();
        reversed.push(previous);
    }
    reversed.reverse();
    NavigationRoutePlanV1::new(query.clone(), reversed, total_cost)
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldPopulationOwnerError {
    Contract(WorldPopulationContractError),
    ProjectMismatch,
    CommandMismatch,
    PublicationStale,
    CountOverflow,
    TickExhausted,
}

impl WorldPopulationOwnerError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.diagnostic_code(),
            Self::ProjectMismatch => "WORLD_POPULATION_CONTENT_INVALID",
            Self::CommandMismatch => "WORLD_POPULATION_COMMAND_MISMATCH",
            Self::PublicationStale => "PREPARED_WORLD_SERVICES_GENERATION_STALE",
            Self::CountOverflow => "WORLD_POPULATION_COUNT_OVERFLOW",
            Self::TickExhausted => "WORLD_POPULATION_TICK_EXHAUSTED",
        }
    }
}

impl Display for WorldPopulationOwnerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for WorldPopulationOwnerError {}

impl From<WorldPopulationContractError> for WorldPopulationOwnerError {
    fn from(error: WorldPopulationContractError) -> Self {
        Self::Contract(error)
    }
}

#[cfg(test)]
mod tests {
    use next_contracts::ids::AssetId;
    use next_contracts::world_population::{
        WORLD_POPULATION_ACTIVE_COUNT_V1, WORLD_POPULATION_BACKGROUND_COUNT_V1,
        WORLD_POPULATION_COUNT_V1, WORLD_POPULATION_NEAR_COUNT_V1, WorldNavigationEdgeV1,
        WorldNavigationNodeV1, WorldNavigationTileV1, WorldPopulationDefinitionV1,
        navigation_tile_revision, population_cadence_phase,
    };

    use super::*;

    fn fixture() -> WorldPopulationOwnerV1 {
        let regions = [
            SchemaId::new("nextengine.test.region.a").expect("region"),
            SchemaId::new("nextengine.test.region.b").expect("region"),
        ];
        let nodes = [
            SchemaId::new("nextengine.test.chunk.a").expect("node"),
            SchemaId::new("nextengine.test.chunk.b").expect("node"),
        ];
        let tile_ids = [
            SchemaId::new("nextengine.test.tile.a").expect("tile"),
            SchemaId::new("nextengine.test.tile.b").expect("tile"),
        ];
        let navigation = WorldNavigationCatalogV1 {
            schema_version: 1,
            catalog_asset_id: AssetId::from_bytes([0x96; 16]),
            topology_revision: 1,
            tiles: (0..2)
                .map(|index| WorldNavigationTileV1 {
                    tile_id: tile_ids[index].clone(),
                    region_id: regions[index].clone(),
                    tile_revision: navigation_tile_revision(
                        &tile_ids[index],
                        &regions[index],
                        &[&nodes[index]],
                    )
                    .expect("tile revision"),
                })
                .collect(),
            nodes: (0..2)
                .map(|index| WorldNavigationNodeV1 {
                    node_id: nodes[index].clone(),
                    tile_id: tile_ids[index].clone(),
                    chunk_id: nodes[index].clone(),
                })
                .collect(),
            edges: vec![WorldNavigationEdgeV1 {
                node_low: nodes[0].clone(),
                node_high: nodes[1].clone(),
                cost: 10,
                capability: NavigationCapabilityV1::AbstractTransfer,
            }],
        };
        let navigation_revision = navigation.revision().expect("navigation revision");
        let mut records = (0_u8..100)
            .map(|ordinal| {
                let subject_id = PersistentId::from_bytes([ordinal; 16]);
                let (cadence_class, initial_tier) =
                    if usize::from(ordinal) < WORLD_POPULATION_ACTIVE_COUNT_V1 {
                        (PopulationCadenceClassV1::Active, PopulationTierV1::Active)
                    } else if usize::from(ordinal)
                        < WORLD_POPULATION_ACTIVE_COUNT_V1 + WORLD_POPULATION_NEAR_COUNT_V1
                    {
                        (PopulationCadenceClassV1::Near, PopulationTierV1::Simulated)
                    } else if usize::from(ordinal) == WORLD_POPULATION_COUNT_V1 - 1 {
                        (
                            PopulationCadenceClassV1::Background,
                            PopulationTierV1::Dormant,
                        )
                    } else {
                        (
                            PopulationCadenceClassV1::Background,
                            PopulationTierV1::Abstract,
                        )
                    };
                WorldPopulationDefinitionV1 {
                    subject_id,
                    home_node_id: nodes[0].clone(),
                    initial_node_id: nodes[0].clone(),
                    navigation_goal_node_id: nodes[1].clone(),
                    cadence_class,
                    cadence_phase: population_cadence_phase(subject_id, cadence_class),
                    initial_tier,
                }
            })
            .collect::<Vec<_>>();
        records.sort();
        debug_assert_eq!(
            records
                .iter()
                .filter(|record| record.cadence_class == PopulationCadenceClassV1::Background)
                .count(),
            WORLD_POPULATION_BACKGROUND_COUNT_V1
        );
        let population = WorldPopulationCatalogV1 {
            schema_version: 1,
            catalog_asset_id: AssetId::from_bytes([0x95; 16]),
            navigation_catalog_asset_id: navigation.catalog_asset_id,
            navigation_catalog_revision: navigation_revision,
            courier_subject_id: PersistentId::from_bytes([99; 16]),
            courier_transition_start_tick: 1,
            records,
        };
        WorldPopulationOwnerV1::activate(population, navigation, 0).expect("activate")
    }

    #[test]
    fn courier_crosses_all_tiers_and_active_transfer_is_rejected() {
        let mut owner = fixture();
        for tick in 0_u64..8 {
            let mut staged = owner.snapshot_or_none().cloned().expect("snapshot");
            let report = owner
                .service_snapshot(&staged, tick)
                .expect("service report");
            if let Some(command) = owner
                .expected_command_for_snapshot(&staged, tick)
                .expect("expected command")
            {
                owner
                    .apply_command_to_snapshot(&mut staged, &command, tick)
                    .expect("apply command");
            }
            let prepared = owner
                .prepare_publication(Some(staged), Some(report), tick + 1)
                .expect("prepare");
            let validated = owner
                .validate_prepared_publication(prepared)
                .expect("validate");
            owner.commit_validated_publication(validated);
            if tick == 4 {
                let error = owner
                    .abstract_transfer_command(
                        owner.snapshot_or_none().expect("snapshot"),
                        owner
                            .population_catalog_or_none()
                            .expect("catalog")
                            .courier_subject_id,
                        tick + 1,
                    )
                    .expect_err("active transfer must require physical traversal");
                assert_eq!(error.diagnostic_code(), "PHYSICAL_TRAVERSAL_REQUIRED");
            }
        }
        let snapshot = owner.snapshot_or_none().expect("snapshot");
        let courier = snapshot
            .record(
                owner
                    .population_catalog_or_none()
                    .expect("catalog")
                    .courier_subject_id,
            )
            .expect("courier");
        assert_eq!(courier.record_revision, 7);
        assert_eq!(courier.tier, PopulationTierV1::Dormant);
    }

    #[test]
    fn stale_route_and_stale_publication_fail_without_mutation() {
        let mut owner = fixture();
        let navigation = owner.navigation_catalog_or_none().expect("navigation");
        let query = NavigationQueryV1 {
            schema_version: 1,
            catalog_asset_id: navigation.catalog_asset_id,
            graph_revision: ContentHash::from_bytes([0xff; 32]),
            start_node_id: navigation.nodes[0].node_id.clone(),
            goal_node_id: navigation.nodes[1].node_id.clone(),
            capability: NavigationCapabilityV1::AbstractTransfer,
        };
        assert_eq!(
            owner
                .route(&query)
                .expect_err("stale route")
                .diagnostic_code(),
            "NAVIGATION_ROUTE_STALE"
        );

        let report = owner.service_tick(0).expect("report");
        let prepared = owner
            .prepare_publication(owner.snapshot_or_none().cloned(), Some(report), 1)
            .expect("prepare");
        let validated = owner
            .validate_prepared_publication(prepared)
            .expect("validate");
        let mut replacement = owner.snapshot_or_none().cloned().expect("snapshot");
        let report = owner.service_snapshot(&replacement, 1).expect("report");
        let command = owner
            .expected_command_for_snapshot(&replacement, 1)
            .expect("command")
            .expect("due");
        owner
            .apply_command_to_snapshot(&mut replacement, &command, 1)
            .expect("apply");
        let replacement = owner
            .prepare_publication(Some(replacement), Some(report), 2)
            .expect("prepare replacement");
        let replacement = owner
            .validate_prepared_publication(replacement)
            .expect("validate replacement");
        owner.commit_validated_publication(replacement);
        assert_eq!(
            owner
                .preflight_validated_publication(&validated)
                .expect_err("stale generation")
                .diagnostic_code(),
            "PREPARED_WORLD_SERVICES_GENERATION_STALE"
        );
    }
}
