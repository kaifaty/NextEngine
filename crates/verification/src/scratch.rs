use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(test)]
use std::sync::{Arc, Mutex};

static NEXT_SCRATCH_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub(crate) struct ScratchContext {
    root: PathBuf,
    #[cfg(test)]
    allocations: Arc<Mutex<Vec<PathBuf>>>,
}

impl ScratchContext {
    pub(crate) fn new(root: &Path) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "verification scratch root must be an existing non-link directory",
            ));
        }
        let root = fs::canonicalize(root)?;
        Ok(Self {
            root,
            #[cfg(test)]
            allocations: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub(crate) fn create_directory(&self, label: &str) -> io::Result<ScratchDirectory> {
        validate_label(label)?;
        let sequence = NEXT_SCRATCH_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = self.root.join(format!(
            "nextengine-{label}-{}-{sequence}",
            std::process::id()
        ));
        if path.parent() != Some(self.root.as_path()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "verification scratch path escaped its supplied root",
            ));
        }
        fs::create_dir(&path)?;
        #[cfg(test)]
        self.allocations
            .lock()
            .expect("scratch allocation audit mutex")
            .push(path.clone());
        Ok(ScratchDirectory {
            path: Some(path),
            #[cfg(test)]
            allocations: Arc::clone(&self.allocations),
        })
    }

    #[cfg(test)]
    pub(crate) fn allocated_paths(&self) -> Vec<PathBuf> {
        self.allocations
            .lock()
            .expect("scratch allocation audit mutex")
            .clone()
    }
}

#[derive(Debug)]
pub(crate) struct ScratchDirectory {
    path: Option<PathBuf>,
    #[cfg(test)]
    allocations: Arc<Mutex<Vec<PathBuf>>>,
}

impl ScratchDirectory {
    pub(crate) fn path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("owned scratch directory is available until cleanup")
    }

    pub(crate) fn context(&self) -> ScratchContext {
        ScratchContext {
            root: self.path().to_path_buf(),
            #[cfg(test)]
            allocations: Arc::clone(&self.allocations),
        }
    }

    pub(crate) fn finish<T, E>(
        mut self,
        result: Result<T, E>,
        cleanup_error: impl FnOnce(io::Error) -> E,
    ) -> Result<T, E> {
        match self.cleanup() {
            Ok(()) => result,
            Err(error) => Err(cleanup_error(error)),
        }
    }

    fn cleanup(&mut self) -> io::Result<()> {
        let Some(path) = self.path.take() else {
            return Ok(());
        };
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() || is_reparse_point(&metadata) => {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "owned verification scratch directory became a link",
                ))
            }
            Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(path),
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "owned verification scratch directory changed file type",
            )),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

fn validate_label(label: &str) -> io::Result<()> {
    let mut components = Path::new(label).components();
    if label.is_empty()
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "verification scratch label must be one normal path component",
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::ScratchContext;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn scratch_child_is_fresh_bounded_and_removed() {
        let root = std::env::temp_dir().join(format!(
            "nextengine-verification-scratch-test-{}-{}",
            std::process::id(),
            TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("create test root");
        let canonical_root = fs::canonicalize(&root).expect("canonical test root");
        let context = ScratchContext::new(&root).expect("scratch context");
        let directory = context
            .create_directory("bounded")
            .expect("fresh scratch child");
        assert!(directory.path().starts_with(&canonical_root));
        let path = directory.path().to_path_buf();
        directory
            .finish(Ok::<_, std::io::Error>(()), |error| error)
            .expect("scratch cleanup");
        assert!(!path.exists());

        let failed_directory = context
            .create_directory("failed-check")
            .expect("fresh failed-check child");
        let failed_path = failed_directory.path().to_path_buf();
        let result = failed_directory.finish::<(), _>(Err("CHECK_FAILED"), |_| "CLEANUP_FAILED");
        assert_eq!(result, Err("CHECK_FAILED"));
        assert!(!failed_path.exists());
        fs::remove_dir(&root).expect("remove test root");
    }

    #[test]
    fn scratch_root_and_child_collisions_fail_closed() {
        let missing = std::env::temp_dir().join(format!(
            "nextengine-verification-missing-test-{}-{}",
            std::process::id(),
            TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(ScratchContext::new(&missing).is_err());

        let root = std::env::temp_dir().join(format!(
            "nextengine-verification-collision-test-{}-{}",
            std::process::id(),
            TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("create test root");
        let context = ScratchContext::new(&root).expect("scratch context");
        assert!(context.create_directory("../escape").is_err());
        fs::remove_dir(&root).expect("remove test root");
    }
}
