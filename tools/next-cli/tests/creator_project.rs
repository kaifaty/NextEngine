use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::{CONTENT_GENERATIONS_DIRECTORY, ContentStore};
use next_cli::CreatorCommandReportV1;
use next_project::{PROJECT_AUTHORING_MAX_SOURCE_BYTES, activate_project};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Self {
        let ordinal = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-creator-cli-{label}-{}-{ordinal}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated creator CLI test directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn creator_project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../projects/creator-smoke")
}

fn next(arguments: &[&OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_next"))
        .args(arguments)
        .output()
        .expect("run next creator CLI")
}

fn text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("CLI stdout is UTF-8 JSON")
}

fn report(output: &Output) -> CreatorCommandReportV1 {
    let stdout = text(output);
    assert!(stdout.ends_with('\n'));
    assert_eq!(stdout.bytes().filter(|byte| *byte == b'\n').count(), 1);
    serde_json::from_str(stdout.trim_end()).expect("decode Creator Command Report V1")
}

fn generation_count(output_root: &Path) -> usize {
    fs::read_dir(output_root.join(CONTENT_GENERATIONS_DIRECTORY))
        .expect("read generation directory")
        .count()
}

#[test]
fn creator_project_validate_is_deterministic_and_side_effect_free() {
    let project = creator_project();
    let args = [
        OsStr::new("project"),
        OsStr::new("validate"),
        OsStr::new("--project"),
        project.as_os_str(),
    ];
    let first = next(&args);
    let second = next(&args);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert!(first.stderr.is_empty());

    let CreatorCommandReportV1::Pass(pass) = report(&first) else {
        panic!("creator project validates");
    };
    assert_eq!(pass.command, "project.validate");
    assert_eq!(pass.details.project_id, "org.nextengine.creator-smoke");
    assert_eq!(pass.details.project_revision, 1);
    assert_eq!(pass.details.neutral_record_count, 10);
    assert_eq!(pass.details.world_chunk_count, 3);
    assert_eq!(pass.details.render_asset_count, 3);
    assert_eq!(pass.details.publication_state, "validated-not-written");
}

#[test]
fn creator_project_cook_is_idempotent_and_reopens_through_production_activation() {
    let scratch = TestDirectory::new("cook");
    let output_root = scratch.path().join("cooked");
    let project = creator_project();
    let args = [
        OsStr::new("project"),
        OsStr::new("cook"),
        OsStr::new("--project"),
        project.as_os_str(),
        OsStr::new("--output"),
        output_root.as_os_str(),
    ];
    let first = next(&args);
    let second = next(&args);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert!(first.stderr.is_empty());
    assert_eq!(generation_count(&output_root), 1);

    let CreatorCommandReportV1::Pass(pass) = report(&first) else {
        panic!("creator project cooks");
    };
    assert_eq!(pass.command, "project.cook");
    assert_eq!(pass.details.publication_state, "published-and-activated");
    assert_eq!(
        fs::read_to_string(output_root.join("CURRENT")).expect("read current generation"),
        format!("{}\n", pass.details.project_lock_sha256)
    );
    let activated = activate_project(&ContentStore::new(&output_root))
        .expect("public cook reopens through production activation");
    assert_eq!(
        activated.project_lock.project_id.as_str(),
        "org.nextengine.creator-smoke"
    );
    assert_eq!(activated.project_lock.project_revision, 1);
    assert_eq!(activated.neutral_records.len(), 10);
    assert_eq!(activated.world_partition.body.chunk_bindings.len(), 3);
}

#[test]
fn invalid_project_preserves_the_prior_complete_generation() {
    let scratch = TestDirectory::new("preserve");
    let output_root = scratch.path().join("cooked");
    let project = creator_project();
    let valid = next(&[
        OsStr::new("project"),
        OsStr::new("cook"),
        OsStr::new("--project"),
        project.as_os_str(),
        OsStr::new("--output"),
        output_root.as_os_str(),
    ]);
    assert!(valid.status.success());
    let current_before = fs::read(output_root.join("CURRENT")).expect("read current generation");
    let generation_count_before = generation_count(&output_root);

    let retired = scratch.path().join("retired-project");
    fs::create_dir(&retired).expect("create retired project");
    fs::write(
        retired.join("project.authoring.json"),
        br#"{"format":"nextengine.project-authoring.v1","legacy":true}"#,
    )
    .expect("write retired project");
    let failed = next(&[
        OsStr::new("project"),
        OsStr::new("cook"),
        OsStr::new("--project"),
        retired.as_os_str(),
        OsStr::new("--output"),
        output_root.as_os_str(),
    ]);
    assert!(!failed.status.success());
    let stdout = text(&failed);
    assert!(!stdout.contains(retired.to_string_lossy().as_ref()));
    let CreatorCommandReportV1::Fail(failure) = report(&failed) else {
        panic!("retired project fails");
    };
    assert_eq!(failure.command, "project.cook");
    assert_eq!(
        failure.diagnostic.code,
        "UNSUPPORTED_PROJECT_AUTHORING_FORMAT"
    );
    assert_eq!(
        fs::read(output_root.join("CURRENT")).expect("read preserved current generation"),
        current_before
    );
    assert_eq!(generation_count(&output_root), generation_count_before);
}

#[test]
fn unrelated_output_directory_is_rejected_without_touching_its_contents() {
    let scratch = TestDirectory::new("unrelated-output");
    let output_root = scratch.path().join("ordinary-directory");
    fs::create_dir(&output_root).expect("create ordinary output directory");
    let sentinel = output_root.join("keep.txt");
    fs::write(&sentinel, b"keep this content").expect("write sentinel");
    let project = creator_project();
    let failed = next(&[
        OsStr::new("project"),
        OsStr::new("cook"),
        OsStr::new("--project"),
        project.as_os_str(),
        OsStr::new("--output"),
        output_root.as_os_str(),
    ]);
    assert!(!failed.status.success());
    let CreatorCommandReportV1::Fail(failure) = report(&failed) else {
        panic!("unrelated output fails");
    };
    assert_eq!(failure.diagnostic.code, "CREATOR_OUTPUT_INVALID");
    assert_eq!(
        fs::read(&sentinel).expect("read sentinel"),
        b"keep this content"
    );
    assert!(!output_root.join(CONTENT_GENERATIONS_DIRECTORY).exists());
}

#[test]
fn oversized_authoring_source_fails_before_json_decode() {
    let scratch = TestDirectory::new("source-limit");
    let project = scratch.path().join("oversized-project");
    fs::create_dir(&project).expect("create oversized project");
    let manifest = fs::File::create(project.join("project.authoring.json"))
        .expect("create oversized manifest");
    manifest
        .set_len(PROJECT_AUTHORING_MAX_SOURCE_BYTES + 1)
        .expect("size oversized manifest");

    let failed = next(&[
        OsStr::new("project"),
        OsStr::new("validate"),
        OsStr::new("--project"),
        project.as_os_str(),
    ]);
    assert!(!failed.status.success());
    let CreatorCommandReportV1::Fail(failure) = report(&failed) else {
        panic!("oversized source fails");
    };
    assert_eq!(failure.diagnostic.code, "CONTENT_SOURCE_LIMIT_EXCEEDED");
}

#[cfg(unix)]
#[test]
fn symlink_source_escape_and_symlink_output_are_rejected() {
    use std::os::unix::fs::symlink;

    let scratch = TestDirectory::new("symlink-escape");
    let escaped_project = scratch.path().join("project");
    fs::create_dir_all(escaped_project.join("assets")).expect("create escaped project");
    fs::copy(
        creator_project().join("project.authoring.json"),
        escaped_project.join("project.authoring.json"),
    )
    .expect("copy authoring manifest");
    fs::copy(
        creator_project().join("NOTICE"),
        escaped_project.join("NOTICE"),
    )
    .expect("copy notice");
    let outside_source = scratch.path().join("outside-source.txt");
    fs::copy(
        creator_project().join("assets/original-source.txt"),
        &outside_source,
    )
    .expect("copy outside source");
    symlink(
        &outside_source,
        escaped_project.join("assets/original-source.txt"),
    )
    .expect("create escaped source symlink");

    let escaped = next(&[
        OsStr::new("project"),
        OsStr::new("validate"),
        OsStr::new("--project"),
        escaped_project.as_os_str(),
    ]);
    assert!(!escaped.status.success());
    let CreatorCommandReportV1::Fail(failure) = report(&escaped) else {
        panic!("escaped source fails");
    };
    assert_eq!(failure.diagnostic.code, "CONTENT_SOURCE_PATH_INVALID");
    assert!(!text(&escaped).contains(outside_source.to_string_lossy().as_ref()));

    let actual_output = scratch.path().join("actual-output");
    fs::create_dir(&actual_output).expect("create actual output");
    let linked_output = scratch.path().join("linked-output");
    symlink(&actual_output, &linked_output).expect("create output symlink");
    let project = creator_project();
    let linked = next(&[
        OsStr::new("project"),
        OsStr::new("cook"),
        OsStr::new("--project"),
        project.as_os_str(),
        OsStr::new("--output"),
        linked_output.as_os_str(),
    ]);
    assert!(!linked.status.success());
    let CreatorCommandReportV1::Fail(failure) = report(&linked) else {
        panic!("symlink output fails");
    };
    assert_eq!(failure.diagnostic.code, "CREATOR_OUTPUT_INVALID");
    assert_eq!(
        fs::read_dir(&actual_output)
            .expect("read actual output")
            .count(),
        0
    );
}
