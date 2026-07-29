use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct CooperativePublishLock {
    path: PathBuf,
}

impl CooperativePublishLock {
    pub(crate) fn acquire(output: &Path) -> Result<Self, String> {
        let parent = output.parent().ok_or_else(|| {
            "NATIVE_GATE_REPORT_INVALID: publish output has no parent directory".to_owned()
        })?;
        let output_name = output
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                "NATIVE_GATE_REPORT_INVALID: publish output requires a UTF-8 name".to_owned()
            })?;
        let path = parent.join(format!(".{output_name}.publish.lock"));
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                return Err(format!(
                    "NATIVE_GATE_OUTPUT_EXISTS: publish lock already exists: {}",
                    path.display()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "NATIVE_GATE_REPORT_INVALID: failed to create publish lock {}: {error}",
                    path.display()
                ));
            }
        };
        if let Err(error) = writeln!(file, "{}", std::process::id()) {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(format!(
                "NATIVE_GATE_REPORT_INVALID: failed to write publish lock {}: {error}",
                path.display()
            ));
        }
        Ok(Self { path })
    }
}

impl Drop for CooperativePublishLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub(crate) fn path_exists_without_following(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!(
            "NATIVE_GATE_REPORT_INVALID: failed to inspect {}: {error}",
            path.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn cooperative_lock_rejects_a_second_publisher() {
        let directory = test_directory("lock");
        let output = directory.join("target");
        let first = CooperativePublishLock::acquire(&output).expect("first lock");
        let error =
            CooperativePublishLock::acquire(&output).expect_err("second lock must be rejected");
        assert!(error.starts_with("NATIVE_GATE_OUTPUT_EXISTS"));
        drop(first);
        CooperativePublishLock::acquire(&output).expect("lock is released");
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn no_follow_existence_detects_a_dangling_link_when_supported() {
        let directory = test_directory("dangling");
        let link = directory.join("output");
        let missing = directory.join("missing");
        if !create_file_link(&missing, &link) {
            fs::remove_dir_all(directory).expect("remove unsupported test directory");
            return;
        }
        assert!(!link.exists(), "link must be dangling for this test");
        assert!(path_exists_without_following(&link).expect("inspect dangling link"));
        fs::remove_file(&link).expect("remove dangling link");
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    fn test_directory(label: &str) -> PathBuf {
        let sequence = DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-native-gate-publish-{label}-{}-{sequence}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).expect("remove stale test directory");
        }
        fs::create_dir(&path).expect("create test directory");
        path
    }

    #[cfg(unix)]
    fn create_file_link(target: &Path, link: &Path) -> bool {
        std::os::unix::fs::symlink(target, link).expect("create dangling symlink");
        true
    }

    #[cfg(windows)]
    fn create_file_link(target: &Path, link: &Path) -> bool {
        match std::os::windows::fs::symlink_file(target, link) {
            Ok(()) => true,
            Err(error)
                if error.kind() == ErrorKind::PermissionDenied
                    || error.raw_os_error() == Some(1314) =>
            {
                false
            }
            Err(error) => panic!("create dangling symlink: {error}"),
        }
    }
}
