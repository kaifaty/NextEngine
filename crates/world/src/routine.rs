use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::ids::ContentHash;
use next_contracts::world_routine::{
    WorldRoutineCatalogV1, WorldRoutineContractError, WorldRoutineSnapshotV1,
};

/// Live World Services owner for the one optional R4a routine record.
///
/// The immutable catalog and the durable snapshot remain outside Runtime.
/// Prepared publications capture the exact record/catalog base so a caller can
/// perform a final read-only joint preflight before any live owner is changed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldRoutineOwnerV1 {
    catalog_or_none: Option<WorldRoutineCatalogV1>,
    snapshot_or_none: Option<WorldRoutineSnapshotV1>,
}

impl WorldRoutineOwnerV1 {
    pub fn activate(
        catalog_or_none: Option<WorldRoutineCatalogV1>,
        next_simulation_tick: u64,
    ) -> Result<Self, WorldRoutineOwnerError> {
        let snapshot_or_none = match catalog_or_none.as_ref() {
            Some(catalog) => {
                if catalog.profile.anchor_simulation_tick != next_simulation_tick {
                    return Err(WorldRoutineOwnerError::ProjectMismatch);
                }
                Some(WorldRoutineSnapshotV1::initial(catalog)?)
            }
            None => None,
        };
        let value = Self {
            catalog_or_none,
            snapshot_or_none,
        };
        value.validate(next_simulation_tick)?;
        Ok(value)
    }

    pub fn restore(
        catalog_or_none: Option<WorldRoutineCatalogV1>,
        snapshot_or_none: Option<WorldRoutineSnapshotV1>,
        next_simulation_tick: u64,
    ) -> Result<Self, WorldRoutineOwnerError> {
        let value = Self {
            catalog_or_none,
            snapshot_or_none,
        };
        value.validate(next_simulation_tick)?;
        Ok(value)
    }

    #[must_use]
    pub const fn catalog_or_none(&self) -> Option<&WorldRoutineCatalogV1> {
        self.catalog_or_none.as_ref()
    }

    #[must_use]
    pub const fn snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.snapshot_or_none.as_ref()
    }

    pub fn validate(&self, next_simulation_tick: u64) -> Result<(), WorldRoutineOwnerError> {
        match (&self.catalog_or_none, &self.snapshot_or_none) {
            (Some(catalog), Some(snapshot)) => {
                snapshot.validate_against(catalog, next_simulation_tick)?;
                Ok(())
            }
            (None, None) => Ok(()),
            _ => Err(WorldRoutineOwnerError::ProjectMismatch),
        }
    }

    pub fn prepare_publication(
        &self,
        next_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
        next_simulation_tick: u64,
    ) -> Result<PreparedWorldRoutinePublicationV1, WorldRoutineOwnerError> {
        validate_snapshot_pair(
            self.catalog_or_none.as_ref(),
            next_snapshot_or_none.as_ref(),
            next_simulation_tick,
        )?;
        Ok(PreparedWorldRoutinePublicationV1 {
            base: WorldRoutineOwnerBaseV1::capture(self)?,
            next_snapshot_or_none,
            next_simulation_tick,
        })
    }

    pub fn validate_prepared_publication(
        &self,
        prepared: PreparedWorldRoutinePublicationV1,
    ) -> Result<ValidatedWorldRoutinePublicationV1, WorldRoutineOwnerError> {
        self.preflight_base(&prepared.base)?;
        validate_snapshot_pair(
            self.catalog_or_none.as_ref(),
            prepared.next_snapshot_or_none.as_ref(),
            prepared.next_simulation_tick,
        )?;
        Ok(ValidatedWorldRoutinePublicationV1 {
            base: prepared.base,
            next_snapshot_or_none: prepared.next_snapshot_or_none,
        })
    }

    /// Rechecks the captured live base without changing the owner.
    pub fn preflight_validated_publication(
        &self,
        validated: &ValidatedWorldRoutinePublicationV1,
    ) -> Result<(), WorldRoutineOwnerError> {
        self.preflight_base(&validated.base)
    }

    /// Publishes an already validated snapshot with no fallible work.
    pub fn commit_validated_publication(&mut self, validated: ValidatedWorldRoutinePublicationV1) {
        self.snapshot_or_none = validated.next_snapshot_or_none;
    }

    fn preflight_base(&self, base: &WorldRoutineOwnerBaseV1) -> Result<(), WorldRoutineOwnerError> {
        if WorldRoutineOwnerBaseV1::capture(self)? != *base {
            return Err(WorldRoutineOwnerError::PublicationStale);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorldRoutineOwnerBaseV1 {
    catalog_revision_or_none: Option<ContentHash>,
    snapshot_or_none: Option<WorldRoutineSnapshotV1>,
}

impl WorldRoutineOwnerBaseV1 {
    fn capture(owner: &WorldRoutineOwnerV1) -> Result<Self, WorldRoutineOwnerError> {
        Ok(Self {
            catalog_revision_or_none: owner
                .catalog_or_none
                .as_ref()
                .map(WorldRoutineCatalogV1::revision)
                .transpose()?,
            snapshot_or_none: owner.snapshot_or_none,
        })
    }
}

/// Complete next routine generation which has not yet been published.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedWorldRoutinePublicationV1 {
    base: WorldRoutineOwnerBaseV1,
    next_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    next_simulation_tick: u64,
}

impl PreparedWorldRoutinePublicationV1 {
    #[must_use]
    pub const fn snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.next_snapshot_or_none.as_ref()
    }
}

/// Commit-only routine generation that retains its base for final preflight.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedWorldRoutinePublicationV1 {
    base: WorldRoutineOwnerBaseV1,
    next_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
}

impl ValidatedWorldRoutinePublicationV1 {
    #[must_use]
    pub const fn snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.next_snapshot_or_none.as_ref()
    }
}

fn validate_snapshot_pair(
    catalog_or_none: Option<&WorldRoutineCatalogV1>,
    snapshot_or_none: Option<&WorldRoutineSnapshotV1>,
    next_simulation_tick: u64,
) -> Result<(), WorldRoutineOwnerError> {
    match (catalog_or_none, snapshot_or_none) {
        (Some(catalog), Some(snapshot)) => {
            snapshot.validate_against(catalog, next_simulation_tick)?;
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(WorldRoutineOwnerError::ProjectMismatch),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldRoutineOwnerError {
    Contract(WorldRoutineContractError),
    ProjectMismatch,
    PublicationStale,
}

impl WorldRoutineOwnerError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Contract(_) | Self::ProjectMismatch => "WORLD_ROUTINE_CONTENT_INVALID",
            Self::PublicationStale => "PREPARED_WORLD_SERVICES_GENERATION_STALE",
        }
    }
}

impl Display for WorldRoutineOwnerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for WorldRoutineOwnerError {}

impl From<WorldRoutineContractError> for WorldRoutineOwnerError {
    fn from(error: WorldRoutineContractError) -> Self {
        Self::Contract(error)
    }
}

#[cfg(test)]
mod tests {
    use next_contracts::ids::{AssetId, PersistentId};
    use next_contracts::world_routine::{
        WORLD_ROUTINE_SCHEMA_VERSION, WorldRoutineActivityV1, WorldRoutineDefinitionV1,
        WorldRoutineProfileV1,
    };

    use super::*;

    fn catalog() -> WorldRoutineCatalogV1 {
        WorldRoutineCatalogV1 {
            schema_version: WORLD_ROUTINE_SCHEMA_VERSION,
            catalog_asset_id: AssetId::from_bytes([0x94; 16]),
            profile: WorldRoutineProfileV1 {
                schema_version: WORLD_ROUTINE_SCHEMA_VERSION,
                anchor_simulation_tick: 0,
                anchor_world_tick: 0,
                world_ticks_per_simulation_tick_num: 1,
                world_ticks_per_simulation_tick_den: 1,
            },
            routine: WorldRoutineDefinitionV1 {
                schema_version: WORLD_ROUTINE_SCHEMA_VERSION,
                subject_id: PersistentId::from_bytes([0x44; 16]),
                initial_activity: WorldRoutineActivityV1::Duty,
                transition_world_tick: 2,
                next_activity: WorldRoutineActivityV1::Rest,
            },
        }
    }

    #[test]
    fn activation_restore_and_stale_preflight_are_closed() {
        let catalog = catalog();
        let mut owner = WorldRoutineOwnerV1::activate(Some(catalog), 0).expect("activate");
        let prepared = owner
            .prepare_publication(owner.snapshot_or_none().copied(), 1)
            .expect("prepare");
        let validated = owner
            .validate_prepared_publication(prepared)
            .expect("validate");

        let mut changed = *owner.snapshot_or_none().expect("snapshot");
        changed.record.record_revision = 1;
        changed.record.current_activity = WorldRoutineActivityV1::Rest;
        let replacement = owner
            .prepare_publication(Some(changed), 3)
            .expect("replacement");
        let replacement = owner
            .validate_prepared_publication(replacement)
            .expect("validate replacement");
        owner.commit_validated_publication(replacement);

        assert_eq!(
            owner
                .preflight_validated_publication(&validated)
                .expect_err("captured generation must be stale")
                .diagnostic_code(),
            "PREPARED_WORLD_SERVICES_GENERATION_STALE"
        );
        WorldRoutineOwnerV1::restore(Some(catalog), owner.snapshot_or_none().copied(), 3)
            .expect("restore");
    }
}
