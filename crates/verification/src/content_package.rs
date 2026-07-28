use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ContentHash;
use next_project::{
    ProjectActivationError, ProjectCookError, activate_project, cook_project_v1,
    neutral_vertical_slice_source_v1,
};

static RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentPackageCheckReport {
    pub records: usize,
    pub chunks: usize,
    pub mechanic_packages: usize,
    pub combat_npc_health: i32,
    pub schema_registry_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub world_partition_hash: ContentHash,
    pub composition_lock_hash: ContentHash,
}

pub fn run_content_package_check() -> Result<ContentPackageCheckReport, ContentPackageCheckError> {
    let source = neutral_vertical_slice_source_v1()?;
    let cooked = cook_project_v1(source)?;
    let counter = RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let output = std::env::temp_dir().join(format!(
        "nextengine-content-package-{}-{counter}",
        std::process::id()
    ));
    let result = (|| {
        let store = ContentStore::new(&output);
        store.publish(&cooked.publication()?)?;
        let activated = activate_project(&store)?;
        let gameplay = crate::run_play_check_with_activated_project(activated.clone())?;
        if activated.content_manifest.body.asset_entries.len() != 11
            || activated.world_partition.body.chunk_bindings.len() != 1
            || activated.rpg_definitions.packages.len() != 2
            || activated.rpg_definitions.abilities.len() != 1
            || gameplay.npc_health != 75
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        Ok(ContentPackageCheckReport {
            records: activated.content_manifest.body.asset_entries.len(),
            chunks: activated.world_partition.body.chunk_bindings.len(),
            mechanic_packages: activated.rpg_definitions.packages.len(),
            combat_npc_health: gameplay.npc_health,
            schema_registry_hash: activated.schema_registry.schema_registry_manifest_sha256,
            content_manifest_hash: activated.content_manifest.content_manifest_sha256,
            world_partition_hash: activated.world_partition.world_partition_manifest_sha256,
            composition_lock_hash: activated.composition_lock.composition_lock_sha256,
        })
    })();
    if output.exists() {
        std::fs::remove_dir_all(&output).map_err(ContentPackageCheckError::Cleanup)?;
    }
    result
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ContentPackageCheckError {
    Cook(ProjectCookError),
    Store(next_assets::ContentStoreError),
    Activation(ProjectActivationError),
    Gameplay(crate::PlayCheckError),
    Cleanup(std::io::Error),
    FixtureClosureMismatch,
}

impl Display for ContentPackageCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cook(error) => write!(formatter, "content-package cook failed: {error}"),
            Self::Store(error) => write!(formatter, "content-package publish failed: {error}"),
            Self::Activation(error) => {
                write!(formatter, "content-package activation failed: {error}")
            }
            Self::Gameplay(error) => write!(formatter, "content-package gameplay failed: {error}"),
            Self::Cleanup(error) => write!(formatter, "content-package cleanup failed: {error}"),
            Self::FixtureClosureMismatch => {
                formatter.write_str("content-package fixture closure mismatch")
            }
        }
    }
}

impl Error for ContentPackageCheckError {}

impl From<ProjectCookError> for ContentPackageCheckError {
    fn from(error: ProjectCookError) -> Self {
        Self::Cook(error)
    }
}

impl From<next_assets::ContentStoreError> for ContentPackageCheckError {
    fn from(error: next_assets::ContentStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<ProjectActivationError> for ContentPackageCheckError {
    fn from(error: ProjectActivationError) -> Self {
        Self::Activation(error)
    }
}

impl From<crate::PlayCheckError> for ContentPackageCheckError {
    fn from(error: crate::PlayCheckError) -> Self {
        Self::Gameplay(error)
    }
}

#[cfg(test)]
mod tests {
    use super::run_content_package_check;

    #[test]
    fn content_package_uses_cooker_publisher_and_production_loader() {
        let report = run_content_package_check().expect("content-package passes");
        assert_eq!(report.records, 11);
        assert_eq!(report.chunks, 1);
        assert_eq!(report.mechanic_packages, 2);
        assert_eq!(report.combat_npc_health, 75);
    }
}
