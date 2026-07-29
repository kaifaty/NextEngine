use std::fs;

use super::super::{PublishLock, ensure_publish_destination_absent, publish_staged_directory_with};
use super::TestDirectory;

#[test]
fn publish_lock_and_destination_checks_never_accept_existing_objects() {
    let temporary = TestDirectory::new("publish-collision");
    let output = temporary.path().join("package");
    let first_lock = PublishLock::acquire(&output).expect("first publish lock");
    assert!(PublishLock::acquire(&output).is_err());
    drop(first_lock);
    assert!(PublishLock::acquire(&output).is_ok());

    fs::write(&output, b"existing").expect("destination");
    assert!(ensure_publish_destination_absent(&output).is_err());
}

#[test]
fn late_destination_collision_removes_completed_staging() {
    let temporary = TestDirectory::new("late-publish-collision");
    let output = temporary.path().join("package");
    let staging = temporary.path().join(".package.staging");
    fs::create_dir(&staging).expect("staging");
    fs::write(staging.join("complete"), b"staged package").expect("staged file");

    let error = publish_staged_directory_with(&staging, &output, |_, destination| {
        fs::write(destination, b"racing publisher").expect("late destination");
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "simulated publish race",
        ))
    })
    .expect_err("late collision must fail closed");

    assert!(error.starts_with("NATIVE_GATE_OUTPUT_EXISTS:"), "{error}");
    assert!(!staging.exists(), "failed publication must remove staging");
    assert_eq!(
        fs::read(&output).expect("existing destination remains"),
        b"racing publisher"
    );
}

#[cfg(unix)]
#[test]
fn destination_check_rejects_dangling_symlink() {
    use std::os::unix::fs::symlink;

    let temporary = TestDirectory::new("dangling-destination");
    let output = temporary.path().join("package");
    symlink("missing", &output).expect("dangling symlink");
    assert!(ensure_publish_destination_absent(&output).is_err());
}
