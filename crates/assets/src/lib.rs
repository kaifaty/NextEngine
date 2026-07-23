#![forbid(unsafe_code)]

mod migration;
mod save;

pub use migration::{
    MigrationFailure, MigrationRegistry, MigrationRegistryError, MigrationStepError,
    PreservedSaveBytes, SaveMigration,
};
pub use save::{
    LoadedSave, PreservedFile, RejectedGeneration, SaveCommitReceipt, SaveImage, SaveLoadError,
    SaveStore, SaveStoreError,
};
