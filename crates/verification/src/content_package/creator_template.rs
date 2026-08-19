use std::ffi::OsString;

use next_project::{cook_project_v7, load_project_authoring_v7};

use super::{ContentPackageCheckError, ScratchContext, verify_creator_public_run_and_package};

pub(super) fn verify(scratch: &ScratchContext) -> Result<(), ContentPackageCheckError> {
    let directory = scratch
        .create_directory("content-package-creator-template")
        .map_err(ContentPackageCheckError::Cleanup)?;
    let result = (|| {
        let project_directory = directory.path().join("generated-rpg-project");
        let created = next_cli::execute([
            OsString::from("project"),
            OsString::from("create"),
            OsString::from("--template"),
            OsString::from("rpg-starter"),
            OsString::from("--project-id"),
            OsString::from("org.nextengine.generated-rpg-check"),
            OsString::from("--output"),
            project_directory.as_os_str().to_owned(),
        ]);
        let next_cli::CreatorCliReportV1::Project(next_cli::CreatorCommandReportV1::Pass(created)) =
            created
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let source = load_project_authoring_v7(&project_directory)?;
        if created.command != "project.create"
            || created.details.project_id != "org.nextengine.generated-rpg-check"
            || created.details.publication_state != "created-and-validated"
            || created.details.neutral_record_count != 10
            || created.details.world_chunk_count != 3
            || source.project_id.as_str() != "org.nextengine.generated-rpg-check"
            || source.records.len() != 10
            || source.chunks.len() != 3
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        let cooked = cook_project_v7(source)?;
        if created.details.project_lock_sha256 != cooked.project_lock.project_lock_sha256.to_hex() {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        verify_creator_public_run_and_package(
            scratch,
            &project_directory,
            cooked.project_lock.project_lock_sha256,
        )
    })();
    directory.finish(result, ContentPackageCheckError::Cleanup)
}
