use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};
use next_contracts::rpg::CommitmentStateV1;
use next_contracts::world_activity::{
    WorldActivityCatalogV1, WorldActivityChangedV1, WorldActivityCommandV1,
    WorldActivityContractError, WorldActivityEvidenceV1, WorldActivitySnapshotV1,
    WorldActivityStateV1,
};
use next_contracts::world_population::{
    PopulationTierV1, WorldPopulationRecordV1, WorldPopulationSnapshotV1,
};

#[derive(Clone, Copy, Debug)]
pub struct WorldActivityObservationV1<'a> {
    pub commitment_id: next_contracts::ids::PersistentId,
    pub commitment_revision: u64,
    pub commitment_state: CommitmentStateV1,
    pub population_record: &'a WorldPopulationRecordV1,
    pub population_snapshot: &'a WorldPopulationSnapshotV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldActivityOwnerV1 {
    catalog: WorldActivityCatalogV1,
    snapshot: WorldActivitySnapshotV1,
}

impl WorldActivityOwnerV1 {
    pub fn activate(
        catalog: WorldActivityCatalogV1,
        next_simulation_tick: u64,
    ) -> Result<Self, WorldActivityOwnerError> {
        if next_simulation_tick != 0 {
            return Err(WorldActivityOwnerError::ProjectMismatch);
        }
        let snapshot = WorldActivitySnapshotV1::initial(&catalog)?;
        let value = Self { catalog, snapshot };
        value.validate(next_simulation_tick)?;
        Ok(value)
    }

    pub fn restore(
        catalog: WorldActivityCatalogV1,
        snapshot: WorldActivitySnapshotV1,
        next_simulation_tick: u64,
    ) -> Result<Self, WorldActivityOwnerError> {
        let value = Self { catalog, snapshot };
        value.validate(next_simulation_tick)?;
        Ok(value)
    }

    #[must_use]
    pub const fn catalog(&self) -> &WorldActivityCatalogV1 {
        &self.catalog
    }

    #[must_use]
    pub const fn snapshot(&self) -> &WorldActivitySnapshotV1 {
        &self.snapshot
    }

    pub fn validate(&self, next_simulation_tick: u64) -> Result<(), WorldActivityOwnerError> {
        self.snapshot
            .validate_against(&self.catalog, next_simulation_tick)?;
        Ok(())
    }

    pub fn expected_command_for_snapshot(
        &self,
        snapshot: &WorldActivitySnapshotV1,
        simulation_tick: u64,
        observation: WorldActivityObservationV1<'_>,
    ) -> Result<Option<WorldActivityCommandV1>, WorldActivityOwnerError> {
        snapshot.validate_against(&self.catalog, simulation_tick)?;
        self.validate_observation(observation)?;
        let evidence = match snapshot.state {
            WorldActivityStateV1::Unassigned
                if observation.commitment_state == CommitmentStateV1::Accepted =>
            {
                Some(WorldActivityEvidenceV1::AcceptedCommitment {
                    commitment_id: observation.commitment_id,
                    commitment_revision: observation.commitment_revision,
                })
            }
            WorldActivityStateV1::Assigned
                if observation.population_record.current_node_id
                    == self.catalog.workplace_node_id
                    && matches!(
                        observation.population_record.tier,
                        PopulationTierV1::Simulated | PopulationTierV1::Active
                    ) =>
            {
                Some(WorldActivityEvidenceV1::WorkplacePresence {
                    population_record_revision: observation.population_record.record_revision,
                    population_snapshot_hash: population_snapshot_hash(
                        observation.population_snapshot,
                    )?,
                })
            }
            WorldActivityStateV1::Working
                if snapshot
                    .work_started_tick_or_none
                    .and_then(|started| started.checked_add(self.catalog.work_duration_ticks))
                    .is_some_and(|due| simulation_tick >= due) =>
            {
                Some(WorldActivityEvidenceV1::ElapsedWork {
                    work_started_tick: snapshot
                        .work_started_tick_or_none
                        .ok_or(WorldActivityOwnerError::SnapshotInvariant)?,
                })
            }
            _ => None,
        };
        let Some(evidence) = evidence else {
            return Ok(None);
        };
        let current_state = snapshot
            .state
            .next()
            .ok_or(WorldActivityOwnerError::SnapshotInvariant)?;
        let command = WorldActivityCommandV1::Transition {
            subject_id: self.catalog.worker_subject_id,
            expected_record_revision: snapshot.record_revision,
            catalog_asset_id: self.catalog.catalog_asset_id,
            catalog_revision: self.catalog.revision()?,
            previous_state: snapshot.state,
            current_state,
            boundary_tick: simulation_tick,
            evidence,
        };
        command.validate_shape()?;
        Ok(Some(command))
    }

    pub fn next_observable_boundary(
        &self,
        snapshot: &WorldActivitySnapshotV1,
        simulation_tick: u64,
        observation: WorldActivityObservationV1<'_>,
    ) -> Result<Option<u64>, WorldActivityOwnerError> {
        if self
            .expected_command_for_snapshot(snapshot, simulation_tick, observation)?
            .is_some()
        {
            return Ok(Some(simulation_tick));
        }
        match snapshot.state {
            WorldActivityStateV1::Working => snapshot
                .work_started_tick_or_none
                .and_then(|started| started.checked_add(self.catalog.work_duration_ticks))
                .map(Some)
                .ok_or(WorldActivityOwnerError::TickExhausted),
            _ => Ok(None),
        }
    }

    pub fn apply_command_to_snapshot(
        &self,
        snapshot: &mut WorldActivitySnapshotV1,
        command: &WorldActivityCommandV1,
        simulation_tick: u64,
        observation: WorldActivityObservationV1<'_>,
    ) -> Result<WorldActivityChangedV1, WorldActivityOwnerError> {
        if self
            .expected_command_for_snapshot(snapshot, simulation_tick, observation)?
            .as_ref()
            != Some(command)
        {
            return Err(WorldActivityOwnerError::CommandMismatch);
        }
        let WorldActivityCommandV1::Transition {
            previous_state,
            current_state,
            boundary_tick,
            evidence,
            ..
        } = command;
        let next_revision = snapshot
            .record_revision
            .checked_add(1)
            .ok_or(WorldActivityOwnerError::TickExhausted)?;
        snapshot.record_revision = next_revision;
        snapshot.state = *current_state;
        match current_state {
            WorldActivityStateV1::Assigned => {
                snapshot.assigned_tick_or_none = Some(*boundary_tick);
            }
            WorldActivityStateV1::Working => {
                snapshot.work_started_tick_or_none = Some(*boundary_tick);
            }
            WorldActivityStateV1::Completed => {
                snapshot.completed_tick_or_none = Some(*boundary_tick);
            }
            WorldActivityStateV1::Unassigned => {
                return Err(WorldActivityOwnerError::SnapshotInvariant);
            }
        }
        snapshot.validate_against(
            &self.catalog,
            simulation_tick
                .checked_add(1)
                .ok_or(WorldActivityOwnerError::TickExhausted)?,
        )?;
        Ok(WorldActivityChangedV1 {
            subject_id: self.catalog.worker_subject_id,
            previous_state: *previous_state,
            current_state: *current_state,
            boundary_tick: *boundary_tick,
            record_revision: next_revision,
            evidence_hash: evidence.canonical_hash(),
        })
    }

    pub fn prepare_publication(
        &self,
        next_snapshot: WorldActivitySnapshotV1,
        next_simulation_tick: u64,
    ) -> Result<PreparedWorldActivityPublicationV1, WorldActivityOwnerError> {
        next_snapshot.validate_against(&self.catalog, next_simulation_tick)?;
        Ok(PreparedWorldActivityPublicationV1 {
            base: WorldActivityOwnerBaseV1::capture(self)?,
            next_snapshot,
            next_simulation_tick,
        })
    }

    pub fn validate_prepared_publication(
        &self,
        prepared: PreparedWorldActivityPublicationV1,
    ) -> Result<ValidatedWorldActivityPublicationV1, WorldActivityOwnerError> {
        self.preflight_base(&prepared.base)?;
        prepared
            .next_snapshot
            .validate_against(&self.catalog, prepared.next_simulation_tick)?;
        Ok(ValidatedWorldActivityPublicationV1 {
            base: prepared.base,
            next_snapshot: prepared.next_snapshot,
        })
    }

    pub fn preflight_validated_publication(
        &self,
        validated: &ValidatedWorldActivityPublicationV1,
    ) -> Result<(), WorldActivityOwnerError> {
        self.preflight_base(&validated.base)
    }

    pub fn commit_validated_publication(&mut self, validated: ValidatedWorldActivityPublicationV1) {
        self.snapshot = validated.next_snapshot;
    }

    fn validate_observation(
        &self,
        observation: WorldActivityObservationV1<'_>,
    ) -> Result<(), WorldActivityOwnerError> {
        if observation.commitment_id != self.catalog.commitment_id
            || observation.population_record.subject_id != self.catalog.worker_subject_id
            || observation
                .population_snapshot
                .record(self.catalog.worker_subject_id)
                != Some(observation.population_record)
        {
            return Err(WorldActivityOwnerError::ObservationInvalid);
        }
        Ok(())
    }

    fn preflight_base(
        &self,
        base: &WorldActivityOwnerBaseV1,
    ) -> Result<(), WorldActivityOwnerError> {
        if WorldActivityOwnerBaseV1::capture(self)? != *base {
            return Err(WorldActivityOwnerError::PublicationStale);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldActivityOwnerBaseV1 {
    catalog_revision: ContentHash,
    snapshot: WorldActivitySnapshotV1,
}

impl WorldActivityOwnerBaseV1 {
    fn capture(owner: &WorldActivityOwnerV1) -> Result<Self, WorldActivityOwnerError> {
        Ok(Self {
            catalog_revision: owner.catalog.revision()?,
            snapshot: owner.snapshot.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedWorldActivityPublicationV1 {
    base: WorldActivityOwnerBaseV1,
    next_snapshot: WorldActivitySnapshotV1,
    next_simulation_tick: u64,
}

impl PreparedWorldActivityPublicationV1 {
    #[must_use]
    pub const fn snapshot(&self) -> &WorldActivitySnapshotV1 {
        &self.next_snapshot
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedWorldActivityPublicationV1 {
    base: WorldActivityOwnerBaseV1,
    next_snapshot: WorldActivitySnapshotV1,
}

impl ValidatedWorldActivityPublicationV1 {
    #[must_use]
    pub const fn snapshot(&self) -> &WorldActivitySnapshotV1 {
        &self.next_snapshot
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldActivityOwnerError {
    Contract(WorldActivityContractError),
    ProjectMismatch,
    SnapshotInvariant,
    ObservationInvalid,
    CommandMismatch,
    PublicationStale,
    TickExhausted,
}

impl WorldActivityOwnerError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.diagnostic_code(),
            Self::ProjectMismatch => "WORLD_ACTIVITY_CONTENT_INVALID",
            Self::SnapshotInvariant => "WORLD_ACTIVITY_SNAPSHOT_CLOSURE_INVALID",
            Self::ObservationInvalid => "WORLD_ACTIVITY_OBSERVATION_INVALID",
            Self::CommandMismatch => "WORLD_ACTIVITY_COMMAND_MISMATCH",
            Self::PublicationStale => "PREPARED_WORLD_SERVICES_GENERATION_STALE",
            Self::TickExhausted => "WORLD_ACTIVITY_TICK_EXHAUSTED",
        }
    }
}

impl Display for WorldActivityOwnerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for WorldActivityOwnerError {}

impl From<WorldActivityContractError> for WorldActivityOwnerError {
    fn from(error: WorldActivityContractError) -> Self {
        Self::Contract(error)
    }
}

fn population_snapshot_hash(
    snapshot: &WorldPopulationSnapshotV1,
) -> Result<ContentHash, WorldActivityOwnerError> {
    let bytes = snapshot
        .canonical_bytes()
        .map_err(|_| WorldActivityOwnerError::ObservationInvalid)?;
    Ok(content_hash_from_bytes(sha256(&bytes)))
}

#[cfg(test)]
mod tests {
    use next_contracts::ids::{AssetId, PersistentId, SchemaId};
    use next_contracts::world_activity::WORLD_ACTIVITY_SCHEMA_VERSION;

    use super::*;

    fn catalog() -> WorldActivityCatalogV1 {
        WorldActivityCatalogV1 {
            schema_version: WORLD_ACTIVITY_SCHEMA_VERSION,
            catalog_asset_id: AssetId::from_bytes([0xa1; 16]),
            worker_subject_id: PersistentId::from_bytes([0x11; 16]),
            commitment_id: PersistentId::from_bytes([0x22; 16]),
            work_id: SchemaId::new("nextengine.work.relay-shift").expect("work"),
            workplace_node_id: SchemaId::new("nextengine.location.frontier").expect("place"),
            work_duration_ticks: 1,
        }
    }

    fn population(
        tier: PopulationTierV1,
        node_id: &SchemaId,
        revision: u64,
    ) -> WorldPopulationSnapshotV1 {
        WorldPopulationSnapshotV1 {
            schema_version: next_contracts::world_population::WORLD_POPULATION_SCHEMA_VERSION,
            catalog_asset_id: AssetId::from_bytes([0x95; 16]),
            catalog_revision: ContentHash::from_bytes([0x95; 32]),
            navigation_catalog_asset_id: AssetId::from_bytes([0x96; 16]),
            navigation_catalog_revision: ContentHash::from_bytes([0x96; 32]),
            records: vec![WorldPopulationRecordV1 {
                subject_id: PersistentId::from_bytes([0x11; 16]),
                record_revision: revision,
                home_region_id: SchemaId::new("nextengine.region.relay").expect("region"),
                home_node_id: SchemaId::new("nextengine.location.relay").expect("node"),
                current_region_id: SchemaId::new("nextengine.region.frontier").expect("region"),
                current_node_id: node_id.clone(),
                tier,
            }],
        }
    }

    fn observation<'a>(
        catalog: &WorldActivityCatalogV1,
        population: &'a WorldPopulationSnapshotV1,
        state: CommitmentStateV1,
        revision: u64,
    ) -> WorldActivityObservationV1<'a> {
        WorldActivityObservationV1 {
            commitment_id: catalog.commitment_id,
            commitment_revision: revision,
            commitment_state: state,
            population_record: &population.records[0],
            population_snapshot: population,
        }
    }

    #[test]
    fn activity_requires_commitment_presence_and_elapsed_work_in_order() {
        let catalog = catalog();
        let mut owner = WorldActivityOwnerV1::activate(catalog.clone(), 0).expect("owner");
        let workplace = catalog.workplace_node_id.clone();
        let population = population(PopulationTierV1::Simulated, &workplace, 2);

        let mut snapshot = owner.snapshot().clone();
        let command = owner
            .expected_command_for_snapshot(
                &snapshot,
                1,
                observation(&catalog, &population, CommitmentStateV1::Accepted, 1),
            )
            .expect("assignment")
            .expect("assignment due");
        owner
            .apply_command_to_snapshot(
                &mut snapshot,
                &command,
                1,
                observation(&catalog, &population, CommitmentStateV1::Accepted, 1),
            )
            .expect("assign");

        let command = owner
            .expected_command_for_snapshot(
                &snapshot,
                2,
                observation(&catalog, &population, CommitmentStateV1::Accepted, 1),
            )
            .expect("work start")
            .expect("work start due");
        owner
            .apply_command_to_snapshot(
                &mut snapshot,
                &command,
                2,
                observation(&catalog, &population, CommitmentStateV1::Accepted, 1),
            )
            .expect("start");
        let command = owner
            .expected_command_for_snapshot(
                &snapshot,
                3,
                observation(&catalog, &population, CommitmentStateV1::Accepted, 1),
            )
            .expect("completion")
            .expect("completion due");
        owner
            .apply_command_to_snapshot(
                &mut snapshot,
                &command,
                3,
                observation(&catalog, &population, CommitmentStateV1::Accepted, 1),
            )
            .expect("complete");
        let prepared = owner.prepare_publication(snapshot, 4).expect("prepare");
        let validated = owner
            .validate_prepared_publication(prepared)
            .expect("validate");
        owner.commit_validated_publication(validated);
        assert_eq!(owner.snapshot().state, WorldActivityStateV1::Completed);
    }

    #[test]
    fn abstract_or_wrong_location_never_fabricates_work_start() {
        let catalog = catalog();
        let owner = WorldActivityOwnerV1::activate(catalog.clone(), 0).expect("owner");
        let mut assigned = owner.snapshot().clone();
        assigned.state = WorldActivityStateV1::Assigned;
        assigned.record_revision = 1;
        assigned.assigned_tick_or_none = Some(1);
        let wrong = SchemaId::new("nextengine.location.relay").expect("wrong node");
        for population in [
            population(PopulationTierV1::Abstract, &catalog.workplace_node_id, 2),
            population(PopulationTierV1::Simulated, &wrong, 0),
        ] {
            assert!(
                owner
                    .expected_command_for_snapshot(
                        &assigned,
                        2,
                        observation(&catalog, &population, CommitmentStateV1::Accepted, 1),
                    )
                    .expect("defer")
                    .is_none()
            );
        }
    }
}
