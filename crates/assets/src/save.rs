mod error;
mod generation;
mod image;
mod store;

pub use error::{PreservedFile, RejectedGeneration, SaveLoadError, SaveStoreError};
pub use generation::LoadedSave;
pub use image::{SaveImage, ValidatedSaveImage};
pub use store::{SaveCommitReceipt, SaveStore};

#[cfg(test)]
mod tests;
