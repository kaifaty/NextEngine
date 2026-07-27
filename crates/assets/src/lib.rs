#![forbid(unsafe_code)]

mod save;

pub use save::{
    LoadedSave, PreservedFile, RejectedGeneration, SaveCommitReceipt, SaveImage, SaveLoadError,
    SaveStore, SaveStoreError, ValidatedSaveImage,
};
