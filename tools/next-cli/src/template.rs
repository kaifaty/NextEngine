use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use next_project::{CookedProjectV7, cook_project_v7, load_project_authoring_v7};

const RPG_STARTER_ID: &str = "rpg-starter";
const TEMPLATE_PROJECT_ID: &str = "org.nextengine.creator-smoke";
const TEMPLATE_NAMESPACE: &str = "nextengine.creator-smoke";

const AUTHORING_SOURCE: &str =
    include_str!("../../../projects/creator-smoke/project.authoring.json");
const NOTICE_SOURCE: &[u8] = include_bytes!("../../../projects/creator-smoke/NOTICE");
const ORIGINAL_SOURCE: &[u8] =
    include_bytes!("../../../projects/creator-smoke/assets/original-source.txt");

static STAGING_ORDINAL: AtomicU64 = AtomicU64::new(0);

pub(super) struct CreatedProject {
    pub(super) cooked: CookedProjectV7,
    pub(super) neutral_record_count: usize,
    pub(super) publication_file_count: usize,
}

#[derive(Debug)]
pub(super) enum CreatorTemplateError {
    Argument,
    Invalid,
    OutputInvalid,
    Storage,
}

pub(super) fn create_project(
    template: &OsStr,
    requested_project_id: &OsStr,
    requested_output: &Path,
) -> Result<CreatedProject, CreatorTemplateError> {
    if template != OsStr::new(RPG_STARTER_ID) {
        return Err(CreatorTemplateError::Argument);
    }
    let project_id = requested_project_id
        .to_str()
        .filter(|value| valid_project_id(value))
        .ok_or(CreatorTemplateError::Argument)?;
    let staging = StagingDirectory::new(requested_output)?;
    let result = (|| {
        fs::create_dir(staging.path().join("assets")).map_err(|_| CreatorTemplateError::Storage)?;
        let authoring = render_authoring(project_id)?;
        fs::write(staging.path().join("project.authoring.json"), authoring)
            .map_err(|_| CreatorTemplateError::Storage)?;
        fs::write(staging.path().join("NOTICE"), NOTICE_SOURCE)
            .map_err(|_| CreatorTemplateError::Storage)?;
        fs::write(
            staging.path().join("assets/original-source.txt"),
            ORIGINAL_SOURCE,
        )
        .map_err(|_| CreatorTemplateError::Storage)?;
        fs::write(staging.path().join("README.md"), starter_readme(project_id))
            .map_err(|_| CreatorTemplateError::Storage)?;

        let source =
            load_project_authoring_v7(staging.path()).map_err(|_| CreatorTemplateError::Invalid)?;
        let neutral_record_count = source.records.len();
        let cooked = cook_project_v7(source).map_err(|_| CreatorTemplateError::Invalid)?;
        let publication_file_count = cooked
            .publication()
            .map_err(|_| CreatorTemplateError::Invalid)?
            .files
            .len();
        Ok(CreatedProject {
            cooked,
            neutral_record_count,
            publication_file_count,
        })
    })();
    match result {
        Ok(created) => {
            staging.publish()?;
            Ok(created)
        }
        Err(error) => Err(error),
    }
}

fn render_authoring(project_id: &str) -> Result<Vec<u8>, CreatorTemplateError> {
    let mut value: serde_json::Value =
        serde_json::from_str(AUTHORING_SOURCE).map_err(|_| CreatorTemplateError::Invalid)?;
    rewrite_strings(&mut value, project_id);
    let mut rendered =
        serde_json::to_vec_pretty(&value).map_err(|_| CreatorTemplateError::Invalid)?;
    rendered.push(b'\n');
    Ok(rendered)
}

fn rewrite_strings(value: &mut serde_json::Value, project_id: &str) {
    match value {
        serde_json::Value::String(value) => {
            if value == TEMPLATE_PROJECT_ID {
                *value = project_id.to_owned();
            } else if let Some(suffix) = value.strip_prefix(TEMPLATE_NAMESPACE) {
                *value = format!("{project_id}{suffix}");
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                rewrite_strings(value, project_id);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                rewrite_strings(value, project_id);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

fn valid_project_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (3..=128).contains(&bytes.len())
        && value.contains('.')
        && bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && !value.contains("..")
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
}

fn starter_readme(project_id: &str) -> Vec<u8> {
    format!(
        "# {project_id}\n\n\
         Generated from the Next Engine `rpg-starter` template. The project is a complete\n\
         current-only Project Authoring V7 source with one NPC, ability, quest/dialogue\n\
         interaction and three streamed world chunks.\n\n\
         From the Next Engine repository root:\n\n\
         ```text\n\
         cargo run --locked -p next_cli -- project validate --project <this-directory>\n\
         cargo run --locked -p next_cli -- project run --project <this-directory>\n\
         cargo run --locked -p next_cli -- project package --project <this-directory> --output <absent-package-directory>\n\
         ```\n\n\
         Edit `project.authoring.json`; keep authored IDs inside the `{project_id}` namespace.\n\
         The format is pre-v1/current-only and unsupported revisions fail closed.\n"
    )
    .into_bytes()
}

struct StagingDirectory {
    output: PathBuf,
    path: PathBuf,
}

impl StagingDirectory {
    fn new(requested_output: &Path) -> Result<Self, CreatorTemplateError> {
        let absolute = if requested_output.is_absolute() {
            requested_output.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|_| CreatorTemplateError::Storage)?
                .join(requested_output)
        };
        if !matches!(
            fs::symlink_metadata(&absolute),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        ) {
            return Err(CreatorTemplateError::OutputInvalid);
        }
        let parent = absolute
            .parent()
            .ok_or(CreatorTemplateError::OutputInvalid)?;
        let resolved_parent =
            fs::canonicalize(parent).map_err(|_| CreatorTemplateError::OutputInvalid)?;
        let metadata = fs::symlink_metadata(&resolved_parent)
            .map_err(|_| CreatorTemplateError::OutputInvalid)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(CreatorTemplateError::OutputInvalid);
        }
        let output_name = absolute
            .file_name()
            .and_then(OsStr::to_str)
            .filter(|name| !name.is_empty())
            .ok_or(CreatorTemplateError::OutputInvalid)?;
        let output = resolved_parent.join(output_name);
        for _ in 0..1_024 {
            let ordinal = STAGING_ORDINAL.fetch_add(1, Ordering::Relaxed);
            let path = resolved_parent.join(format!(
                ".{output_name}.creator-template-staging-{}-{ordinal}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { output, path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(CreatorTemplateError::Storage),
            }
        }
        Err(CreatorTemplateError::Storage)
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn publish(self) -> Result<(), CreatorTemplateError> {
        if !matches!(
            fs::symlink_metadata(&self.output),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        ) {
            return Err(CreatorTemplateError::OutputInvalid);
        }
        fs::rename(&self.path, &self.output).map_err(|_| CreatorTemplateError::Storage)?;
        std::mem::forget(self);
        Ok(())
    }
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
