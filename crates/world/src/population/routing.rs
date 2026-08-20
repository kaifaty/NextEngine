use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use next_contracts::ids::{ContentHash, SchemaId};
use next_contracts::world_population::{
    NavigationQueryV1, NavigationRoutePlanV1, WORLD_POPULATION_SCHEMA_VERSION,
    WorldNavigationCatalogV1, WorldPopulationContractError,
};

pub fn route_query(
    catalog: &WorldNavigationCatalogV1,
    query: &NavigationQueryV1,
) -> Result<NavigationRoutePlanV1, WorldPopulationContractError> {
    catalog.validate()?;
    let catalog_revision = catalog.revision()?;
    route_query_prevalidated(catalog, query, catalog_revision)
}

pub(super) fn route_query_prevalidated(
    catalog: &WorldNavigationCatalogV1,
    query: &NavigationQueryV1,
    catalog_revision: ContentHash,
) -> Result<NavigationRoutePlanV1, WorldPopulationContractError> {
    if query.schema_version != WORLD_POPULATION_SCHEMA_VERSION
        || query.catalog_asset_id != catalog.catalog_asset_id
        || query.graph_revision != catalog_revision
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
