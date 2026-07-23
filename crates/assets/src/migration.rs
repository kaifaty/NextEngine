use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreservedSaveBytes {
    pub manifest_bytes: Vec<u8>,
    pub segment_bytes: Vec<Vec<u8>>,
}

pub trait SaveMigration {
    fn source_version(&self) -> u32;
    fn target_version(&self) -> u32;
    fn migrate(
        &self,
        source: &PreservedSaveBytes,
    ) -> Result<PreservedSaveBytes, MigrationStepError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrationStepError {
    pub stable_code: &'static str,
}

impl Display for MigrationStepError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code)
    }
}

impl Error for MigrationStepError {}

#[derive(Default)]
pub struct MigrationRegistry {
    steps: Vec<Box<dyn SaveMigration>>,
}

impl MigrationRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn register(
        &mut self,
        migration: impl SaveMigration + 'static,
    ) -> Result<(), MigrationRegistryError> {
        let source = migration.source_version();
        let target = migration.target_version();
        if target
            != source
                .checked_add(1)
                .ok_or(MigrationRegistryError::VersionExhausted)?
        {
            return Err(MigrationRegistryError::NonAdjacentStep { source, target });
        }
        if self
            .steps
            .iter()
            .any(|existing| existing.source_version() == source)
        {
            return Err(MigrationRegistryError::DuplicateSourceVersion(source));
        }
        self.steps.push(Box::new(migration));
        self.steps.sort_by_key(|step| step.source_version());
        Ok(())
    }

    pub fn migrate(
        &self,
        source_version: u32,
        target_version: u32,
        original: &PreservedSaveBytes,
    ) -> Result<PreservedSaveBytes, MigrationFailure> {
        if source_version > target_version {
            return Err(MigrationFailure {
                original: original.clone(),
                failed_at_version: source_version,
                error: MigrationStepError {
                    stable_code: "SAVE_MIGRATION_DOWNGRADE_FORBIDDEN",
                },
            });
        }
        let mut version = source_version;
        let mut working_copy = original.clone();
        while version < target_version {
            let Some(step) = self
                .steps
                .iter()
                .find(|step| step.source_version() == version)
            else {
                return Err(MigrationFailure {
                    original: original.clone(),
                    failed_at_version: version,
                    error: MigrationStepError {
                        stable_code: "SAVE_MIGRATION_STEP_MISSING",
                    },
                });
            };
            working_copy = step
                .migrate(&working_copy)
                .map_err(|error| MigrationFailure {
                    original: original.clone(),
                    failed_at_version: version,
                    error,
                })?;
            version = step.target_version();
        }
        Ok(working_copy)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MigrationRegistryError {
    NonAdjacentStep { source: u32, target: u32 },
    DuplicateSourceVersion(u32),
    VersionExhausted,
}

impl Display for MigrationRegistryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonAdjacentStep { source, target } => {
                write!(
                    formatter,
                    "migration step must be adjacent: {source} -> {target}"
                )
            }
            Self::DuplicateSourceVersion(version) => {
                write!(
                    formatter,
                    "migration from version {version} is already registered"
                )
            }
            Self::VersionExhausted => formatter.write_str("migration source version is exhausted"),
        }
    }
}

impl Error for MigrationRegistryError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationFailure {
    pub original: PreservedSaveBytes,
    pub failed_at_version: u32,
    pub error: MigrationStepError,
}

impl Display for MigrationFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "save migration failed at version {}: {}",
            self.failed_at_version, self.error
        )
    }
}

impl Error for MigrationFailure {}

#[cfg(test)]
mod tests {
    use super::{MigrationRegistry, MigrationStepError, PreservedSaveBytes, SaveMigration};

    struct AppendMigration {
        source: u32,
        byte: u8,
        fail: bool,
    }

    impl SaveMigration for AppendMigration {
        fn source_version(&self) -> u32 {
            self.source
        }

        fn target_version(&self) -> u32 {
            self.source + 1
        }

        fn migrate(
            &self,
            source: &PreservedSaveBytes,
        ) -> Result<PreservedSaveBytes, MigrationStepError> {
            if self.fail {
                return Err(MigrationStepError {
                    stable_code: "TEST_MIGRATION_FAILED",
                });
            }
            let mut migrated = source.clone();
            migrated.manifest_bytes.push(self.byte);
            Ok(migrated)
        }
    }

    #[test]
    fn ordered_migrations_use_copies_and_reach_exact_target() {
        let original = PreservedSaveBytes {
            manifest_bytes: vec![0],
            segment_bytes: vec![vec![1]],
        };
        let mut registry = MigrationRegistry::new();
        registry
            .register(AppendMigration {
                source: 1,
                byte: 2,
                fail: false,
            })
            .expect("first step registers");
        registry
            .register(AppendMigration {
                source: 0,
                byte: 1,
                fail: false,
            })
            .expect("second step is sorted by source");

        let migrated = registry
            .migrate(0, 2, &original)
            .expect("ordered migration succeeds");
        assert_eq!(migrated.manifest_bytes, vec![0, 1, 2]);
        assert_eq!(original.manifest_bytes, vec![0]);
    }

    #[test]
    fn failed_migration_returns_untouched_original_bytes() {
        let original = PreservedSaveBytes {
            manifest_bytes: vec![9],
            segment_bytes: vec![vec![8]],
        };
        let mut registry = MigrationRegistry::new();
        registry
            .register(AppendMigration {
                source: 0,
                byte: 1,
                fail: true,
            })
            .expect("step registers");

        let failure = registry
            .migrate(0, 1, &original)
            .expect_err("step must fail");
        assert_eq!(failure.original, original);
        assert_eq!(failure.error.stable_code, "TEST_MIGRATION_FAILED");
    }
}
