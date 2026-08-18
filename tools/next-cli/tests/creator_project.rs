use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::{CONTENT_GENERATIONS_DIRECTORY, ContentStore};
use next_cli::{CreatorCommandReportV1, CreatorPackageCommandReportV1, CreatorRunCommandReportV1};
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

fn run_report(output: &Output) -> CreatorRunCommandReportV1 {
    let stdout = text(output);
    assert!(stdout.ends_with('\n'));
    assert_eq!(stdout.bytes().filter(|byte| *byte == b'\n').count(), 1);
    serde_json::from_str(stdout.trim_end()).expect("decode Creator Run Report V1")
}

fn package_report(output: &Output) -> CreatorPackageCommandReportV1 {
    let stdout = text(output);
    assert!(stdout.ends_with('\n'));
    assert_eq!(stdout.bytes().filter(|byte| *byte == b'\n').count(), 1);
    serde_json::from_str(stdout.trim_end()).expect("decode Creator Package Report V1")
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
fn creator_project_run_is_deterministic_and_uses_the_generic_application_runtime() {
    let project = creator_project();
    let args = [
        OsStr::new("project"),
        OsStr::new("run"),
        OsStr::new("--project"),
        project.as_os_str(),
    ];
    let first = next(&args);
    let second = next(&args);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert!(first.stderr.is_empty());

    let CreatorRunCommandReportV1::Pass(pass) = run_report(&first) else {
        panic!("creator project runs");
    };
    assert_eq!(pass.command, "project.run");
    assert_eq!(pass.details.source, "authoring");
    assert_eq!(
        pass.details.project.project_id,
        "org.nextengine.creator-smoke"
    );
    assert_eq!(pass.details.runtime.composition_root, "Headless");
    assert_eq!(pass.details.runtime.ticks, 1);
    assert_eq!(pass.details.runtime.events, 0);
    assert_eq!(pass.details.runtime.rpg_events, 0);
    assert_eq!(pass.details.runtime.status, "PASS");
    assert_eq!(
        pass.details.runtime.project_composition_lock_hash,
        pass.details.project.project_lock_sha256
    );
}

#[test]
fn creator_project_package_is_reproducible_and_runs_from_its_published_bytes() {
    let scratch = TestDirectory::new("package");
    let first_root = scratch.path().join("package-one");
    let second_root = scratch.path().join("package-two");
    let project = creator_project();
    let package = |output: &Path| {
        next(&[
            OsStr::new("project"),
            OsStr::new("package"),
            OsStr::new("--project"),
            project.as_os_str(),
            OsStr::new("--output"),
            output.as_os_str(),
        ])
    };
    let first = package(&first_root);
    let second = package(&second_root);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(
        fs::read(first_root.join("creator-package.manifest.jcs"))
            .expect("read first package manifest"),
        fs::read(second_root.join("creator-package.manifest.jcs"))
            .expect("read second package manifest")
    );
    assert_eq!(
        fs::read(first_root.join("NOTICE")).expect("read packaged notice"),
        fs::read(project.join("NOTICE")).expect("read source notice")
    );

    let CreatorPackageCommandReportV1::Pass(packaged) = package_report(&first) else {
        panic!("creator project packages");
    };
    assert_eq!(packaged.command, "project.package");
    assert_eq!(
        packaged.details.package_format,
        "nextengine.creator-project-package.v1"
    );
    assert_eq!(packaged.details.required_notices, ["NOTICE"]);
    assert_eq!(packaged.details.runtime.ticks, 1);

    let launched = next(&[
        OsStr::new("project"),
        OsStr::new("run"),
        OsStr::new("--package"),
        first_root.as_os_str(),
    ]);
    assert!(launched.status.success());
    let CreatorRunCommandReportV1::Pass(launched) = run_report(&launched) else {
        panic!("published package runs");
    };
    assert_eq!(launched.details.source, "package");
    assert_eq!(launched.details.project, packaged.details.project);
    assert_eq!(launched.details.runtime, packaged.details.runtime);
}

#[test]
fn tampered_creator_package_is_rejected_before_runtime_launch() {
    let scratch = TestDirectory::new("tampered-package");
    let package_root = scratch.path().join("package");
    let project = creator_project();
    let built = next(&[
        OsStr::new("project"),
        OsStr::new("package"),
        OsStr::new("--project"),
        project.as_os_str(),
        OsStr::new("--output"),
        package_root.as_os_str(),
    ]);
    assert!(built.status.success());
    fs::write(package_root.join("NOTICE"), b"tampered notice").expect("tamper packaged notice");

    let failed = next(&[
        OsStr::new("project"),
        OsStr::new("run"),
        OsStr::new("--package"),
        package_root.as_os_str(),
    ]);
    assert!(!failed.status.success());
    let CreatorRunCommandReportV1::Fail(failure) = run_report(&failed) else {
        panic!("tampered package fails");
    };
    assert_eq!(failure.command, "project.run");
    assert_eq!(failure.diagnostic.code, "CREATOR_PACKAGE_INVALID");
    assert!(!text(&failed).contains(package_root.to_string_lossy().as_ref()));
}

#[test]
fn unsupported_creator_package_version_is_rejected_before_nested_decode() {
    let scratch = TestDirectory::new("unsupported-package");
    let package_root = scratch.path().join("package");
    let project = creator_project();
    let built = next(&[
        OsStr::new("project"),
        OsStr::new("package"),
        OsStr::new("--project"),
        project.as_os_str(),
        OsStr::new("--output"),
        package_root.as_os_str(),
    ]);
    assert!(built.status.success());
    let manifest_path = package_root.join("creator-package.manifest.jcs");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read creator package manifest"))
            .expect("decode creator package manifest");
    manifest["schema_version"] = serde_json::Value::from(2);
    fs::write(
        &manifest_path,
        serde_json::to_vec(&manifest).expect("encode unsupported package manifest"),
    )
    .expect("write unsupported package manifest");

    let failed = next(&[
        OsStr::new("project"),
        OsStr::new("run"),
        OsStr::new("--package"),
        package_root.as_os_str(),
    ]);
    assert!(!failed.status.success());
    let CreatorRunCommandReportV1::Fail(failure) = run_report(&failed) else {
        panic!("unsupported package fails");
    };
    assert_eq!(
        failure.diagnostic.code,
        "UNSUPPORTED_CREATOR_PACKAGE_FORMAT"
    );
}

#[test]
fn existing_package_output_is_rejected_without_touching_its_contents() {
    let scratch = TestDirectory::new("package-output");
    let output_root = scratch.path().join("ordinary-directory");
    fs::create_dir(&output_root).expect("create ordinary package output");
    let sentinel = output_root.join("keep.txt");
    fs::write(&sentinel, b"keep this package output").expect("write package sentinel");
    let project = creator_project();
    let failed = next(&[
        OsStr::new("project"),
        OsStr::new("package"),
        OsStr::new("--project"),
        project.as_os_str(),
        OsStr::new("--output"),
        output_root.as_os_str(),
    ]);
    assert!(!failed.status.success());
    let CreatorPackageCommandReportV1::Fail(failure) = package_report(&failed) else {
        panic!("existing package output fails");
    };
    assert_eq!(failure.diagnostic.code, "CREATOR_PACKAGE_OUTPUT_INVALID");
    assert_eq!(
        fs::read(&sentinel).expect("read package sentinel"),
        b"keep this package output"
    );
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

    let package_output = scratch.path().join("package-output");
    let linked_package_output = scratch.path().join("linked-package-output");
    fs::create_dir(&package_output).expect("create actual package output");
    symlink(&package_output, &linked_package_output).expect("create package output symlink");
    let linked_package = next(&[
        OsStr::new("project"),
        OsStr::new("package"),
        OsStr::new("--project"),
        project.as_os_str(),
        OsStr::new("--output"),
        linked_package_output.as_os_str(),
    ]);
    assert!(!linked_package.status.success());
    let CreatorPackageCommandReportV1::Fail(failure) = package_report(&linked_package) else {
        panic!("symlink package output fails");
    };
    assert_eq!(failure.diagnostic.code, "CREATOR_PACKAGE_OUTPUT_INVALID");
    assert_eq!(
        fs::read_dir(&package_output)
            .expect("read actual package output")
            .count(),
        0
    );

    let linked_notice_project = scratch.path().join("linked-notice-project");
    fs::create_dir_all(linked_notice_project.join("assets")).expect("create linked notice project");
    fs::copy(
        project.join("project.authoring.json"),
        linked_notice_project.join("project.authoring.json"),
    )
    .expect("copy linked notice authoring manifest");
    fs::copy(
        project.join("assets/original-source.txt"),
        linked_notice_project.join("assets/original-source.txt"),
    )
    .expect("copy linked notice source");
    fs::copy(
        project.join("NOTICE"),
        linked_notice_project.join("NOTICE.real"),
    )
    .expect("copy linked notice target");
    symlink(
        linked_notice_project.join("NOTICE.real"),
        linked_notice_project.join("NOTICE"),
    )
    .expect("create linked notice");
    let linked_notice_output = scratch.path().join("linked-notice-package");
    let linked_notice = next(&[
        OsStr::new("project"),
        OsStr::new("package"),
        OsStr::new("--project"),
        linked_notice_project.as_os_str(),
        OsStr::new("--output"),
        linked_notice_output.as_os_str(),
    ]);
    assert!(!linked_notice.status.success());
    let CreatorPackageCommandReportV1::Fail(failure) = package_report(&linked_notice) else {
        panic!("linked notice fails");
    };
    assert_eq!(failure.diagnostic.code, "CREATOR_PACKAGE_NOTICE_INVALID");
    assert!(!linked_notice_output.exists());
}
