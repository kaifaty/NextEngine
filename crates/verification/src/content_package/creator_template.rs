use std::ffi::OsString;
use std::fs;
use std::path::Path;

use next_project::{cook_project_v7, load_project_authoring_v7};
use serde_json::Value;

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
        let generated_readme = fs::read_to_string(project_directory.join("README.md"))
            .map_err(ContentPackageCheckError::Cleanup)?;
        if !generated_readme.contains("docs/creator-sdk.md")
            || !generated_readme.contains("project cook")
            || !generated_readme.contains("project inspect")
            || !generated_readme.contains("project diff")
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        personalize_public_authoring(&project_directory)?;
        let source = load_project_authoring_v7(&project_directory)?;
        if created.command != "project.create"
            || created.details.project_id != "org.nextengine.generated-rpg-check"
            || created.details.publication_state != "created-and-validated"
            || created.details.neutral_record_count != 10
            || created.details.world_chunk_count != 3
            || source.project_id.as_str() != "org.nextengine.generated-rpg-check"
            || source.records.len() != 10
            || source.chunks.len() != 3
            || !source.records.iter().any(|record| {
                record.kind == next_contracts::content::NeutralRecordKindV1::CharacterDefinition
                    && record.properties.iter().any(|property| {
                        property.property_id.as_str() == "nextengine.creator.role"
                            && property.value_id.as_str()
                                == "org.nextengine.generated-rpg-check.character.ridge-warden"
                    })
            })
            || !source.chunks.iter().any(|chunk| {
                chunk.chunk_id.as_str() == "org.nextengine.generated-rpg-check.chunk.signal-tower"
                    && chunk.region_id.as_str()
                        == "org.nextengine.generated-rpg-check.region.signal-tower"
            })
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        let cooked = cook_project_v7(source)?;
        let expected_lock = cooked.project_lock.project_lock_sha256.to_hex();
        if created.details.project_lock_sha256 == expected_lock
            || cooked.rpg_definitions.abilities[0].ability_id.as_str()
                != "org.nextengine.generated-rpg-check.ability.ridge-pulse"
            || cooked.rpg_definitions.quests[0].entry_state_id.as_str()
                != "org.nextengine.generated-rpg-check.quest.awaiting-signal"
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }

        let validated = next_cli::execute([
            OsString::from("project"),
            OsString::from("validate"),
            OsString::from("--project"),
            project_directory.as_os_str().to_owned(),
        ]);
        let next_cli::CreatorCliReportV1::Project(next_cli::CreatorCommandReportV1::Pass(
            validated,
        )) = validated
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        if validated.command != "project.validate"
            || validated.details.publication_state != "validated-not-written"
            || validated.details.project_lock_sha256 != expected_lock
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }

        let cooked_output = directory.path().join("generated-rpg-cooked");
        let published = next_cli::execute([
            OsString::from("project"),
            OsString::from("cook"),
            OsString::from("--project"),
            project_directory.as_os_str().to_owned(),
            OsString::from("--output"),
            cooked_output.as_os_str().to_owned(),
        ]);
        let next_cli::CreatorCliReportV1::Project(next_cli::CreatorCommandReportV1::Pass(
            published,
        )) = published
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        if published.command != "project.cook"
            || published.details.publication_state != "published-and-activated"
            || published.details.project_lock_sha256 != expected_lock
        {
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

fn personalize_public_authoring(project_directory: &Path) -> Result<(), ContentPackageCheckError> {
    let path = project_directory.join("project.authoring.json");
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&path).map_err(ContentPackageCheckError::Cleanup)?)
            .map_err(|_| ContentPackageCheckError::FixtureClosureMismatch)?;
    set_record_property(
        &mut manifest,
        "character-definition",
        "nextengine.creator.role",
        "org.nextengine.generated-rpg-check.character.ridge-warden",
    )?;
    set_record_property(
        &mut manifest,
        "ability-definition",
        "nextengine.ability.definition-id",
        "org.nextengine.generated-rpg-check.ability.ridge-pulse",
    )?;
    set_record_property(
        &mut manifest,
        "quest-definition",
        "nextengine.quest.entry-state",
        "org.nextengine.generated-rpg-check.quest.awaiting-signal",
    )?;
    set_record_property(
        &mut manifest,
        "interaction-definition",
        "nextengine.interaction.quest-source",
        "org.nextengine.generated-rpg-check.quest.awaiting-signal",
    )?;
    let chunks = manifest["partition"]["chunks"]
        .as_array_mut()
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
    let crossing = chunks
        .iter_mut()
        .find(|chunk| chunk["chunk_id"] == "org.nextengine.generated-rpg-check.chunk.crossing")
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
    crossing["chunk_id"] =
        Value::String("org.nextengine.generated-rpg-check.chunk.signal-tower".to_owned());
    crossing["region_id"] =
        Value::String("org.nextengine.generated-rpg-check.region.signal-tower".to_owned());
    let mut bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|_| ContentPackageCheckError::FixtureClosureMismatch)?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(ContentPackageCheckError::Cleanup)
}

fn set_record_property(
    manifest: &mut Value,
    kind: &str,
    property_id: &str,
    value_id: &str,
) -> Result<(), ContentPackageCheckError> {
    let records = manifest["records"]
        .as_array_mut()
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
    let record = records
        .iter_mut()
        .find(|record| record["kind"] == kind)
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
    let properties = record["properties"]
        .as_array_mut()
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
    let property = properties
        .iter_mut()
        .find(|property| property["property_id"] == property_id)
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
    property["value_id"] = Value::String(value_id.to_owned());
    Ok(())
}
