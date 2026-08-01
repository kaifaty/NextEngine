//! Bounded isolated target directory for release evidence builds.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::AllocatorCounterCodegenError;

static NEXT_SCRATCH_ID: AtomicU64 = AtomicU64::new(0);

pub(super) struct CodegenScratch {
    path: PathBuf,
    root_target: PathBuf,
}

impl CodegenScratch {
    pub(super) fn create(root: &Path) -> Result<Self, AllocatorCounterCodegenError> {
        let root_target = root.join("target");
        fs::create_dir_all(&root_target).map_err(|error| {
            AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_SCRATCH_FAILED",
                error.to_string(),
            )
        })?;
        let sequence = NEXT_SCRATCH_ID.fetch_add(1, Ordering::Relaxed);
        let path = root_target.join(format!(
            "allocator-counter-codegen-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).map_err(|error| {
            AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_SCRATCH_FAILED",
                format!("{}: {error}", path.display()),
            )
        })?;
        Ok(Self { path, root_target })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn finish(self) -> Result<(), AllocatorCounterCodegenError> {
        if self.path.parent() != Some(self.root_target.as_path())
            || !self
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("allocator-counter-codegen-"))
        {
            return Err(AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_SCRATCH_INVALID",
                self.path.display().to_string(),
            ));
        }
        fs::remove_dir_all(&self.path).map_err(|error| {
            AllocatorCounterCodegenError::new(
                "ALLOCATOR_COUNTER_CODEGEN_CLEANUP_FAILED",
                format!("{}: {error}", self.path.display()),
            )
        })
    }
}
