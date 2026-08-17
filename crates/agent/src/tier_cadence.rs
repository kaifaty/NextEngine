use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, content_hash_from_bytes};
use next_contracts::world_population::{
    PopulationCadenceClassV1, PopulationTierV1, WorldNavigationCatalogV1, WorldPopulationCatalogV1,
    WorldPopulationSnapshotV1,
};

use crate::cognition::StrategicAgentError;

/// Maximum number of population records admitted by the current R4 profile.
pub const TIER_COGNITION_MAX_WORK_ITEMS_V1: usize = 100;

/// Precision allowed for one due population record. This is scheduling
/// evidence, not an authoritative gameplay outcome.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TierCognitionWorkKindV1 {
    FullEvaluation = 1,
    ReducedEvaluation = 2,
    AbstractMaintenanceOnly = 3,
    DormantWakeCheckOnly = 4,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TierCognitionWorkItemV1 {
    pub subject_id: PersistentId,
    pub population_record_revision: u64,
    pub cadence_class: PopulationCadenceClassV1,
    pub tier: PopulationTierV1,
    pub work_kind: TierCognitionWorkKindV1,
}

/// Immutable per-tick dispatcher result produced at AgentPlanning. Abstract
/// and dormant work is deliberately classified without fabricating traversal,
/// combat, trade, quest, or other authoritative outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierCognitionServiceReportV1 {
    pub simulation_tick: u64,
    pub work_items: Vec<TierCognitionWorkItemV1>,
    pub full_evaluation_due: u32,
    pub reduced_evaluation_due: u32,
    pub abstract_maintenance_due: u32,
    pub dormant_wake_due: u32,
    pub fabricated_outcome_count: u32,
    pub work_root: ContentHash,
}

impl TierCognitionServiceReportV1 {
    #[must_use]
    pub fn due_count(&self) -> usize {
        self.work_items.len()
    }
}

pub fn dispatch_tier_cognition_v1(
    population: &WorldPopulationCatalogV1,
    navigation: &WorldNavigationCatalogV1,
    snapshot: &WorldPopulationSnapshotV1,
    simulation_tick: u64,
) -> Result<TierCognitionServiceReportV1, StrategicAgentError> {
    snapshot
        .validate_against(population, navigation, simulation_tick)
        .map_err(StrategicAgentError::Population)?;
    let population_revision = population
        .revision(navigation)
        .map_err(StrategicAgentError::Population)?;
    let mut work_items = Vec::new();
    let mut full_evaluation_due = 0_u32;
    let mut reduced_evaluation_due = 0_u32;
    let mut abstract_maintenance_due = 0_u32;
    let mut dormant_wake_due = 0_u32;
    for definition in &population.records {
        if simulation_tick % definition.cadence_class.period_ticks() != definition.cadence_phase {
            continue;
        }
        let record = snapshot
            .record(definition.subject_id)
            .ok_or(StrategicAgentError::ObservationInvalid)?;
        let work_kind = match record.tier {
            PopulationTierV1::Active => {
                increment(&mut full_evaluation_due)?;
                TierCognitionWorkKindV1::FullEvaluation
            }
            PopulationTierV1::Simulated => {
                increment(&mut reduced_evaluation_due)?;
                TierCognitionWorkKindV1::ReducedEvaluation
            }
            PopulationTierV1::Abstract => {
                increment(&mut abstract_maintenance_due)?;
                TierCognitionWorkKindV1::AbstractMaintenanceOnly
            }
            PopulationTierV1::Dormant => {
                increment(&mut dormant_wake_due)?;
                TierCognitionWorkKindV1::DormantWakeCheckOnly
            }
        };
        work_items.push(TierCognitionWorkItemV1 {
            subject_id: definition.subject_id,
            population_record_revision: record.record_revision,
            cadence_class: definition.cadence_class,
            tier: record.tier,
            work_kind,
        });
    }
    if work_items.len() > TIER_COGNITION_MAX_WORK_ITEMS_V1
        || work_items.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(StrategicAgentError::PlannerInvariant);
    }
    let mut preimage = b"nextengine.agent-tier-cognition-service.v1\0".to_vec();
    preimage.extend_from_slice(&simulation_tick.to_le_bytes());
    preimage.extend_from_slice(population_revision.as_bytes());
    for item in &work_items {
        preimage.extend_from_slice(item.subject_id.as_bytes());
        preimage.extend_from_slice(&item.population_record_revision.to_le_bytes());
        preimage.push(item.cadence_class as u8);
        preimage.push(item.tier as u8);
        preimage.push(item.work_kind as u8);
    }
    preimage.extend_from_slice(&full_evaluation_due.to_le_bytes());
    preimage.extend_from_slice(&reduced_evaluation_due.to_le_bytes());
    preimage.extend_from_slice(&abstract_maintenance_due.to_le_bytes());
    preimage.extend_from_slice(&dormant_wake_due.to_le_bytes());
    let fabricated_outcome_count = 0_u32;
    preimage.extend_from_slice(&fabricated_outcome_count.to_le_bytes());
    Ok(TierCognitionServiceReportV1 {
        simulation_tick,
        work_items,
        full_evaluation_due,
        reduced_evaluation_due,
        abstract_maintenance_due,
        dormant_wake_due,
        fabricated_outcome_count,
        work_root: content_hash_from_bytes(sha256(&preimage)),
    })
}

fn increment(value: &mut u32) -> Result<(), StrategicAgentError> {
    *value = value
        .checked_add(1)
        .ok_or(StrategicAgentError::PlannerInvariant)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_reference_population_routes_every_due_record_without_fabricated_outcomes() {
        let cooked = next_project::cook_project_v7(
            next_reference_game::project_source_v7().expect("reference source"),
        )
        .expect("reference cook");
        let reports = (0_u64..60)
            .map(|tick| {
                let snapshot = reference_snapshot_at_tick(&cooked, tick);
                dispatch_tier_cognition_v1(
                    &cooked.world_population_catalog,
                    &cooked.world_navigation_catalog,
                    &snapshot,
                    tick,
                )
                .expect("tier dispatch")
            })
            .collect::<Vec<_>>();
        let totals = reports.iter().fold([0_u64; 5], |mut totals, report| {
            totals[0] += u64::from(report.full_evaluation_due);
            totals[1] += u64::from(report.reduced_evaluation_due);
            totals[2] += u64::from(report.abstract_maintenance_due);
            totals[3] += u64::from(report.dormant_wake_due);
            totals[4] += u64::try_from(report.due_count()).expect("bounded due count");
            assert_eq!(report.fabricated_outcome_count, 0);
            assert!(report.work_items.len() <= TIER_COGNITION_MAX_WORK_ITEMS_V1);
            assert!(
                report
                    .work_items
                    .windows(2)
                    .all(|pair| pair[0].subject_id < pair[1].subject_id)
            );
            totals
        });
        let courier_due_tick = cooked
            .world_population_catalog
            .definition(cooked.world_population_catalog.courier_subject_id)
            .expect("courier definition")
            .cadence_phase;
        let courier_tier = reference_snapshot_at_tick(&cooked, courier_due_tick)
            .record(cooked.world_population_catalog.courier_subject_id)
            .expect("courier record")
            .tier;
        let mut expected = [320_u64, 128, 51, 0, 500];
        match courier_tier {
            PopulationTierV1::Active => expected[0] += 1,
            PopulationTierV1::Simulated => expected[1] += 1,
            PopulationTierV1::Abstract => expected[2] += 1,
            PopulationTierV1::Dormant => expected[3] += 1,
        }
        assert_eq!(totals, expected);
        assert_eq!(
            reports,
            (0_u64..60)
                .map(|tick| {
                    let snapshot = reference_snapshot_at_tick(&cooked, tick);
                    dispatch_tier_cognition_v1(
                        &cooked.world_population_catalog,
                        &cooked.world_navigation_catalog,
                        &snapshot,
                        tick,
                    )
                    .expect("repeat tier dispatch")
                })
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn incomplete_population_snapshot_is_rejected_before_dispatch() {
        let cooked = next_project::cook_project_v7(
            next_reference_game::project_source_v7().expect("reference source"),
        )
        .expect("reference cook");
        let mut snapshot = WorldPopulationSnapshotV1::initial(
            &cooked.world_population_catalog,
            &cooked.world_navigation_catalog,
        )
        .expect("population snapshot");
        snapshot.records.pop();
        assert!(matches!(
            dispatch_tier_cognition_v1(
                &cooked.world_population_catalog,
                &cooked.world_navigation_catalog,
                &snapshot,
                0,
            ),
            Err(StrategicAgentError::Population(_))
        ));
    }

    fn reference_snapshot_at_tick(
        cooked: &next_project::CookedProjectV7,
        tick: u64,
    ) -> WorldPopulationSnapshotV1 {
        let mut snapshot = WorldPopulationSnapshotV1::initial(
            &cooked.world_population_catalog,
            &cooked.world_navigation_catalog,
        )
        .expect("population snapshot");
        let completed = tick
            .saturating_sub(
                cooked
                    .world_population_catalog
                    .courier_transition_start_tick,
            )
            .min(7);
        let definition = cooked
            .world_population_catalog
            .definition(cooked.world_population_catalog.courier_subject_id)
            .expect("courier definition");
        let (tier, node) = match completed {
            0 => (PopulationTierV1::Dormant, &definition.initial_node_id),
            1 => (PopulationTierV1::Abstract, &definition.initial_node_id),
            2 | 6 => (
                PopulationTierV1::Abstract,
                &definition.navigation_goal_node_id,
            ),
            3 | 5 => (
                PopulationTierV1::Simulated,
                &definition.navigation_goal_node_id,
            ),
            4 => (
                PopulationTierV1::Active,
                &definition.navigation_goal_node_id,
            ),
            7 => (
                PopulationTierV1::Dormant,
                &definition.navigation_goal_node_id,
            ),
            _ => unreachable!("completed is clamped"),
        };
        let region = cooked
            .world_navigation_catalog
            .region_for_node(node)
            .expect("courier region")
            .clone();
        let record = snapshot
            .records
            .iter_mut()
            .find(|record| record.subject_id == cooked.world_population_catalog.courier_subject_id)
            .expect("courier record");
        record.record_revision = completed;
        record.current_region_id = region;
        record.current_node_id = node.clone();
        record.tier = tier;
        snapshot
    }
}
