use super::*;

impl WorldPopulationSnapshotV1 {
    /// Performs full boundary validation, including canonical catalog
    /// validation and revision calculation.
    pub fn validate_against(
        &self,
        catalog: &WorldPopulationCatalogV1,
        navigation: &WorldNavigationCatalogV1,
        next_simulation_tick: u64,
    ) -> Result<(), WorldPopulationContractError> {
        catalog.validate_against_navigation(navigation)?;
        let catalog_revision = catalog.revision(navigation)?;
        let navigation_revision = navigation.revision()?;
        self.validate_against_prevalidated_revisions(
            catalog,
            navigation,
            next_simulation_tick,
            catalog_revision,
            navigation_revision,
        )
    }

    /// Validates a snapshot against revisions already established at an
    /// owning boundary. Callers must keep both catalogs immutable while the
    /// supplied revisions are cached.
    pub fn validate_against_prevalidated_revisions(
        &self,
        catalog: &WorldPopulationCatalogV1,
        navigation: &WorldNavigationCatalogV1,
        next_simulation_tick: u64,
        catalog_revision: ContentHash,
        navigation_revision: ContentHash,
    ) -> Result<(), WorldPopulationContractError> {
        if self.schema_version != WORLD_POPULATION_SCHEMA_VERSION
            || self.catalog_asset_id != catalog.catalog_asset_id
            || self.catalog_revision != catalog_revision
            || self.navigation_catalog_asset_id != navigation.catalog_asset_id
            || self.navigation_catalog_revision != navigation_revision
            || self.records.len() != catalog.records.len()
            || !strictly_sorted(&self.records)
        {
            return Err(WorldPopulationContractError::SnapshotClosureInvalid);
        }
        let completed = next_simulation_tick
            .saturating_sub(catalog.courier_transition_start_tick)
            .min(7);
        for (definition, record) in catalog.records.iter().zip(&self.records) {
            let home_region = navigation
                .region_for_node(&definition.home_node_id)
                .ok_or(WorldPopulationContractError::SnapshotClosureInvalid)?;
            let initial_region = navigation
                .region_for_node(&definition.initial_node_id)
                .ok_or(WorldPopulationContractError::SnapshotClosureInvalid)?;
            if definition.subject_id != record.subject_id
                || &record.home_region_id != home_region
                || record.home_node_id != definition.home_node_id
            {
                return Err(WorldPopulationContractError::SnapshotClosureInvalid);
            }
            if record.subject_id != catalog.courier_subject_id {
                if record.record_revision != 0
                    || &record.current_region_id != initial_region
                    || record.current_node_id != definition.initial_node_id
                    || record.tier != definition.initial_tier
                {
                    return Err(WorldPopulationContractError::SnapshotClosureInvalid);
                }
                continue;
            }
            let goal_region = navigation
                .region_for_node(&definition.navigation_goal_node_id)
                .ok_or(WorldPopulationContractError::SnapshotClosureInvalid)?;
            let (expected_tier, expected_region, expected_node) = match completed {
                0 => (
                    PopulationTierV1::Dormant,
                    initial_region,
                    &definition.initial_node_id,
                ),
                1 => (
                    PopulationTierV1::Abstract,
                    initial_region,
                    &definition.initial_node_id,
                ),
                2 => (
                    PopulationTierV1::Abstract,
                    goal_region,
                    &definition.navigation_goal_node_id,
                ),
                3 => (
                    PopulationTierV1::Simulated,
                    goal_region,
                    &definition.navigation_goal_node_id,
                ),
                4 => (
                    PopulationTierV1::Active,
                    goal_region,
                    &definition.navigation_goal_node_id,
                ),
                5 => (
                    PopulationTierV1::Simulated,
                    goal_region,
                    &definition.navigation_goal_node_id,
                ),
                6 => (
                    PopulationTierV1::Abstract,
                    goal_region,
                    &definition.navigation_goal_node_id,
                ),
                7 => (
                    PopulationTierV1::Dormant,
                    goal_region,
                    &definition.navigation_goal_node_id,
                ),
                _ => unreachable!("completed is clamped to seven"),
            };
            if record.record_revision != completed
                || record.tier != expected_tier
                || &record.current_region_id != expected_region
                || &record.current_node_id != expected_node
            {
                return Err(WorldPopulationContractError::SnapshotClosureInvalid);
            }
        }
        Ok(())
    }
}
