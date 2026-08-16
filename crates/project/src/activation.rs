use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_assets::{ContentStore, ContentStoreError, PinnedContentGeneration};
use next_contracts::animation_content::{
    NEUTRAL_ANIMATION_SCHEMA_ID, NEUTRAL_SKELETON_SCHEMA_ID, NeutralAnimationContentErrorV1,
    NeutralAnimationV1, NeutralSkeletonV1,
};
use next_contracts::audio::{NEUTRAL_AUDIO_SCHEMA_ID, NeutralAudioErrorV1, NeutralAudioV1};
use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::cognition::{AGENT_COGNITION_CATALOG_SCHEMA_ID, AgentCognitionCatalogV1};
use next_contracts::content::{NeutralRecordError, NeutralRecordV1};
use next_contracts::identity::RuntimeDeterminismBundleV1;
use next_contracts::ids::AssetId;
use next_contracts::localization::{TEXT_CATALOG_SCHEMA_ID, TextCatalogErrorV1, TextCatalogV1};
use next_contracts::project::{
    ActivatedProjectV7, ContentManifestV1, ContentSemanticClassV1, ProjectContractError,
    ProjectLockV3, SchemaEncodingV1, SchemaRefV1, SchemaRegistryManifestV2, SchemaRoleV1,
    WorldPartitionManifestV1, domain_hash,
};
use next_contracts::render_content::{
    B0CookedMeshV1, NeutralRenderRecordV1, RenderContentCatalogV1, RenderContentContractError,
};
use next_contracts::world_activity::{WORLD_ACTIVITY_CATALOG_SCHEMA_ID, WorldActivityCatalogV1};
use next_contracts::world_population::{
    WORLD_NAVIGATION_CATALOG_SCHEMA_ID, WORLD_POPULATION_CATALOG_SCHEMA_ID,
    WorldNavigationCatalogV1, WorldPopulationCatalogV1,
};

use crate::cook::{
    CONTENT_BLOB_DIRECTORY, CONTENT_MANIFEST_PATH, PROJECT_LOCK_PATH, RENDER_CONTENT_CATALOG_PATH,
    RENDER_CONTENT_MESH_DIRECTORY, RPG_DEFINITIONS_PATH, SCHEMA_REGISTRY_PATH,
    WORLD_PARTITION_PATH, compile_render_content_catalog_v1, launch_profiles_sha256,
    platform_capability_profile_sha256, platform_timebase_profile_sha256,
};
use crate::cook_rpg::activate_rpg_definitions_v2;

#[derive(Clone, Debug)]
pub struct ActivatedProjectPackage {
    pub project: ActivatedProjectV7,
    pub content_generation: PinnedContentGeneration,
}

pub fn activate_project(
    store: &ContentStore,
) -> Result<ActivatedProjectV7, ProjectActivationError> {
    Ok(activate_project_package(store)?.project)
}

pub fn activate_project_package(
    store: &ContentStore,
) -> Result<ActivatedProjectPackage, ProjectActivationError> {
    let content_generation = store.pin_current_generation()?;
    let project = activate_pinned_project(&content_generation)?;
    Ok(ActivatedProjectPackage {
        project,
        content_generation,
    })
}

fn activate_pinned_project(
    content_generation: &PinnedContentGeneration,
) -> Result<ActivatedProjectV7, ProjectActivationError> {
    let generation = content_generation.load_all_verified()?;
    let limits = CanonicalDecodeLimits::default();
    let project_lock = ProjectLockV3::from_jcs_bytes(
        required_file(&generation.files, PROJECT_LOCK_PATH)?,
        limits,
    )?;
    let schema_registry = SchemaRegistryManifestV2::from_jcs_bytes(
        required_file(&generation.files, SCHEMA_REGISTRY_PATH)?,
        limits,
    )?;
    let content_manifest = ContentManifestV1::from_jcs_bytes(
        required_file(&generation.files, CONTENT_MANIFEST_PATH)?,
        limits,
    )?;
    let world_partition = WorldPartitionManifestV1::from_jcs_bytes(
        required_file(&generation.files, WORLD_PARTITION_PATH)?,
        limits,
    )?;

    if generation.generation_id != project_lock.project_lock_sha256
        || project_lock.runtime_determinism_profile_sha256
            != RuntimeDeterminismBundleV1::core_r4d()
                .expect("the engine-owned determinism bundle is canonical")
                .runtime_profile_hash()
        || project_lock.launch_profiles_sha256 != launch_profiles_sha256()
        || project_lock.platform_capability_profile_sha256 != platform_capability_profile_sha256()
        || project_lock.platform_timebase_profile_sha256 != platform_timebase_profile_sha256()
    {
        return Err(ProjectActivationError::HashMismatch);
    }

    let current_schemas: BTreeSet<_> = schema_registry
        .body
        .current_schema_refs
        .iter()
        .cloned()
        .collect();
    require_schema(&current_schemas, &content_manifest.body.schema_ref)?;
    let entries: BTreeMap<_, _> = content_manifest
        .body
        .asset_entries
        .iter()
        .map(|entry| (entry.asset_revision.asset_id, entry))
        .collect();
    let mut expected_files: BTreeSet<String> = [
        PROJECT_LOCK_PATH,
        SCHEMA_REGISTRY_PATH,
        CONTENT_MANIFEST_PATH,
        WORLD_PARTITION_PATH,
        RENDER_CONTENT_CATALOG_PATH,
        RPG_DEFINITIONS_PATH,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let mut record_dependencies = BTreeMap::<AssetId, BTreeSet<AssetId>>::new();
    let mut neutral_records = Vec::new();
    let mut render_records = Vec::new();
    let mut text_catalogs = Vec::new();
    let mut audio_clips = Vec::new();
    let mut neutral_skeletons = Vec::new();
    let mut neutral_animations = Vec::new();
    let mut world_routine_catalog_or_none = None;
    let mut world_navigation_catalog_or_none = None;
    let mut world_population_catalog_or_none = None;
    let mut agent_cognition_catalog_or_none = None;
    let mut world_activity_catalog_or_none = None;
    for entry in &content_manifest.body.asset_entries {
        require_schema(&current_schemas, &entry.schema_ref)?;
        let blob_path = format!(
            "{CONTENT_BLOB_DIRECTORY}/{}.bin",
            entry.neutral_record_blob_sha256.to_hex()
        );
        expected_files.insert(blob_path.clone());
        let blob = required_file(&generation.files, &blob_path)?;
        if entry.schema_ref.schema_id.as_str() == TEXT_CATALOG_SCHEMA_ID {
            let catalog = TextCatalogV1::from_canonical_bytes(blob, limits)?;
            let expected_schema_ref = SchemaRefV1 {
                schema_id: entry.schema_ref.schema_id.clone(),
                schema_version: catalog.schema_version,
                descriptor_sha256: domain_hash(
                    "nextengine.schema-descriptor.v1",
                    TEXT_CATALOG_SCHEMA_ID.as_bytes(),
                ),
                role: SchemaRoleV1::NeutralContent,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            };
            if catalog.catalog_asset_id != entry.asset_revision.asset_id
                || expected_schema_ref != entry.schema_ref
                || catalog.record_sha256()? != entry.asset_revision.record_sha256
                || entry.semantic_class != ContentSemanticClassV1::PresentationOnly
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(catalog.catalog_asset_id, BTreeSet::new());
            text_catalogs.push(catalog);
        } else if entry.schema_ref.schema_id.as_str() == NEUTRAL_AUDIO_SCHEMA_ID {
            let clip = NeutralAudioV1::from_canonical_bytes(blob, limits)?;
            let expected_schema_ref = SchemaRefV1 {
                schema_id: entry.schema_ref.schema_id.clone(),
                schema_version: clip.schema_version,
                descriptor_sha256: domain_hash(
                    "nextengine.schema-descriptor.v1",
                    NEUTRAL_AUDIO_SCHEMA_ID.as_bytes(),
                ),
                role: SchemaRoleV1::NeutralContent,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            };
            if clip.asset_id != entry.asset_revision.asset_id
                || expected_schema_ref != entry.schema_ref
                || clip.record_sha256()? != entry.asset_revision.record_sha256
                || entry.semantic_class != ContentSemanticClassV1::PresentationOnly
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(clip.asset_id, BTreeSet::new());
            audio_clips.push(clip);
        } else if entry.schema_ref.schema_id.as_str() == NEUTRAL_SKELETON_SCHEMA_ID {
            let skeleton = NeutralSkeletonV1::from_canonical_bytes(blob, limits)?;
            let expected_schema_ref = SchemaRefV1 {
                schema_id: entry.schema_ref.schema_id.clone(),
                schema_version: skeleton.schema_version,
                descriptor_sha256: domain_hash(
                    "nextengine.schema-descriptor.v1",
                    NEUTRAL_SKELETON_SCHEMA_ID.as_bytes(),
                ),
                role: SchemaRoleV1::NeutralContent,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            };
            if skeleton.asset_id != entry.asset_revision.asset_id
                || expected_schema_ref != entry.schema_ref
                || skeleton.record_sha256()? != entry.asset_revision.record_sha256
                || entry.semantic_class != ContentSemanticClassV1::DomainRelevant
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(skeleton.asset_id, BTreeSet::new());
            neutral_skeletons.push(skeleton);
        } else if entry.schema_ref.schema_id.as_str() == NEUTRAL_ANIMATION_SCHEMA_ID {
            let animation = NeutralAnimationV1::from_canonical_bytes(blob, limits)?;
            let expected_schema_ref = SchemaRefV1 {
                schema_id: entry.schema_ref.schema_id.clone(),
                schema_version: animation.schema_version,
                descriptor_sha256: domain_hash(
                    "nextengine.schema-descriptor.v1",
                    NEUTRAL_ANIMATION_SCHEMA_ID.as_bytes(),
                ),
                role: SchemaRoleV1::NeutralContent,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            };
            if animation.asset_id != entry.asset_revision.asset_id
                || expected_schema_ref != entry.schema_ref
                || animation.record_sha256()? != entry.asset_revision.record_sha256
                || entry.semantic_class != ContentSemanticClassV1::DomainRelevant
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(
                animation.asset_id,
                BTreeSet::from([animation.skeleton_revision.asset_id]),
            );
            neutral_animations.push(animation);
        } else if entry.schema_ref.schema_id.as_str()
            == next_contracts::world_routine::WORLD_ROUTINE_CATALOG_SCHEMA_ID
        {
            let catalog =
                next_contracts::world_routine::WorldRoutineCatalogV1::from_canonical_bytes(
                    blob, limits,
                )
                .map_err(|_| ProjectActivationError::HashMismatch)?;
            let expected_schema_ref = SchemaRefV1 {
                schema_id: entry.schema_ref.schema_id.clone(),
                schema_version: u32::from(catalog.schema_version),
                descriptor_sha256: domain_hash(
                    "nextengine.schema-descriptor.v1",
                    next_contracts::world_routine::WORLD_ROUTINE_CATALOG_SCHEMA_ID.as_bytes(),
                ),
                role: SchemaRoleV1::NeutralContent,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            };
            if catalog.catalog_asset_id != entry.asset_revision.asset_id
                || expected_schema_ref != entry.schema_ref
                || catalog
                    .revision()
                    .map_err(|_| ProjectActivationError::HashMismatch)?
                    != entry.asset_revision.record_sha256
                || entry.semantic_class != ContentSemanticClassV1::DomainRelevant
                || world_routine_catalog_or_none.replace(catalog).is_some()
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(catalog.catalog_asset_id, BTreeSet::new());
        } else if entry.schema_ref.schema_id.as_str() == WORLD_NAVIGATION_CATALOG_SCHEMA_ID {
            let catalog = WorldNavigationCatalogV1::from_canonical_bytes(blob, limits)
                .map_err(|_| ProjectActivationError::HashMismatch)?;
            let expected_schema_ref = SchemaRefV1 {
                schema_id: entry.schema_ref.schema_id.clone(),
                schema_version: u32::from(catalog.schema_version),
                descriptor_sha256: domain_hash(
                    "nextengine.schema-descriptor.v1",
                    WORLD_NAVIGATION_CATALOG_SCHEMA_ID.as_bytes(),
                ),
                role: SchemaRoleV1::NeutralContent,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            };
            if catalog.catalog_asset_id != entry.asset_revision.asset_id
                || expected_schema_ref != entry.schema_ref
                || catalog
                    .revision()
                    .map_err(|_| ProjectActivationError::HashMismatch)?
                    != entry.asset_revision.record_sha256
                || entry.semantic_class != ContentSemanticClassV1::DomainRelevant
                || world_navigation_catalog_or_none
                    .replace(catalog.clone())
                    .is_some()
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(catalog.catalog_asset_id, BTreeSet::new());
        } else if entry.schema_ref.schema_id.as_str() == WORLD_POPULATION_CATALOG_SCHEMA_ID {
            let catalog = WorldPopulationCatalogV1::from_canonical_bytes(blob, limits)
                .map_err(|_| ProjectActivationError::HashMismatch)?;
            let expected_schema_ref = SchemaRefV1 {
                schema_id: entry.schema_ref.schema_id.clone(),
                schema_version: u32::from(catalog.schema_version),
                descriptor_sha256: domain_hash(
                    "nextengine.schema-descriptor.v1",
                    WORLD_POPULATION_CATALOG_SCHEMA_ID.as_bytes(),
                ),
                role: SchemaRoleV1::NeutralContent,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            };
            if catalog.catalog_asset_id != entry.asset_revision.asset_id
                || expected_schema_ref != entry.schema_ref
                || entry.semantic_class != ContentSemanticClassV1::DomainRelevant
                || world_population_catalog_or_none
                    .replace(catalog.clone())
                    .is_some()
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(
                catalog.catalog_asset_id,
                BTreeSet::from([catalog.navigation_catalog_asset_id]),
            );
        } else if entry.schema_ref.schema_id.as_str() == AGENT_COGNITION_CATALOG_SCHEMA_ID {
            let catalog = AgentCognitionCatalogV1::from_canonical_bytes(blob, limits)
                .map_err(|_| ProjectActivationError::HashMismatch)?;
            let expected_schema_ref = SchemaRefV1 {
                schema_id: entry.schema_ref.schema_id.clone(),
                schema_version: u32::from(catalog.schema_version),
                descriptor_sha256: domain_hash(
                    "nextengine.schema-descriptor.v1",
                    AGENT_COGNITION_CATALOG_SCHEMA_ID.as_bytes(),
                ),
                role: SchemaRoleV1::NeutralContent,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            };
            if catalog.catalog_asset_id != entry.asset_revision.asset_id
                || expected_schema_ref != entry.schema_ref
                || catalog
                    .revision()
                    .map_err(|_| ProjectActivationError::HashMismatch)?
                    != entry.asset_revision.record_sha256
                || entry.semantic_class != ContentSemanticClassV1::DomainRelevant
                || agent_cognition_catalog_or_none
                    .replace(catalog.clone())
                    .is_some()
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(catalog.catalog_asset_id, BTreeSet::new());
        } else if entry.schema_ref.schema_id.as_str() == WORLD_ACTIVITY_CATALOG_SCHEMA_ID {
            let catalog = WorldActivityCatalogV1::from_canonical_bytes(blob, limits)
                .map_err(|_| ProjectActivationError::HashMismatch)?;
            let expected_schema_ref = SchemaRefV1 {
                schema_id: entry.schema_ref.schema_id.clone(),
                schema_version: u32::from(catalog.schema_version),
                descriptor_sha256: domain_hash(
                    "nextengine.schema-descriptor.v1",
                    WORLD_ACTIVITY_CATALOG_SCHEMA_ID.as_bytes(),
                ),
                role: SchemaRoleV1::NeutralContent,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            };
            if catalog.catalog_asset_id != entry.asset_revision.asset_id
                || expected_schema_ref != entry.schema_ref
                || catalog
                    .revision()
                    .map_err(|_| ProjectActivationError::HashMismatch)?
                    != entry.asset_revision.record_sha256
                || entry.semantic_class != ContentSemanticClassV1::DomainRelevant
                || world_activity_catalog_or_none
                    .replace(catalog.clone())
                    .is_some()
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(catalog.catalog_asset_id, BTreeSet::new());
        } else if NeutralRenderRecordV1::supports_schema_id(&entry.schema_ref.schema_id) {
            let record = NeutralRenderRecordV1::from_canonical_bytes(blob, limits)?;
            if record.asset_id() != entry.asset_revision.asset_id
                || record.schema_ref() != &entry.schema_ref
                || record.record_sha256()? != entry.asset_revision.record_sha256
                || entry.semantic_class != record.semantic_class()
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(
                record.asset_id(),
                record
                    .dependencies()
                    .into_iter()
                    .map(|reference| reference.asset_id)
                    .collect(),
            );
            render_records.push(record);
        } else {
            let record = NeutralRecordV1::from_canonical_bytes(blob, limits)?;
            if record.asset_id != entry.asset_revision.asset_id
                || record.schema_ref != entry.schema_ref
                || record.record_sha256()? != entry.asset_revision.record_sha256
                || entry.semantic_class
                    != next_contracts::project::ContentSemanticClassV1::DomainRelevant
            {
                return Err(ProjectActivationError::HashMismatch);
            }
            record_dependencies.insert(
                record.asset_id,
                record.asset_dependencies.iter().copied().collect(),
            );
            neutral_records.push(record);
        }
    }
    let cognition_catalog_asset_id = agent_cognition_catalog_or_none
        .as_ref()
        .ok_or(ProjectActivationError::MissingReference)?
        .catalog_asset_id;
    let population_catalog_asset_id = world_population_catalog_or_none
        .as_ref()
        .ok_or(ProjectActivationError::MissingReference)?
        .catalog_asset_id;
    let activity_catalog_asset_id = world_activity_catalog_or_none
        .as_ref()
        .ok_or(ProjectActivationError::MissingReference)?
        .catalog_asset_id;
    record_dependencies.insert(
        cognition_catalog_asset_id,
        BTreeSet::from([population_catalog_asset_id]),
    );
    record_dependencies.insert(
        activity_catalog_asset_id,
        BTreeSet::from([population_catalog_asset_id]),
    );
    for entry in &content_manifest.body.asset_entries {
        let declared: BTreeSet<_> = content_manifest
            .body
            .dependency_edges
            .iter()
            .filter(|edge| edge.source_asset_id == entry.asset_revision.asset_id && edge.required)
            .map(|edge| edge.target_asset_id)
            .collect();
        if declared
            != *record_dependencies
                .get(&entry.asset_revision.asset_id)
                .expect("decoded every content entry")
        {
            return Err(ProjectActivationError::DependencyMismatch);
        }
    }
    for chunk in &world_partition.body.chunk_bindings {
        if entries
            .get(&chunk.chunk_asset.asset_id)
            .map(|entry| entry.asset_revision)
            != Some(chunk.chunk_asset)
            || chunk
                .required_asset_ids
                .iter()
                .any(|asset_id| !entries.contains_key(asset_id))
        {
            return Err(ProjectActivationError::MissingReference);
        }
    }

    neutral_records.sort_by_key(|record| record.asset_id);
    render_records.sort_by_key(NeutralRenderRecordV1::asset_id);
    text_catalogs.sort_by_key(|catalog| catalog.catalog_asset_id);
    audio_clips.sort_by_key(|clip| clip.asset_id);
    neutral_skeletons.sort_by_key(|record| record.asset_id);
    neutral_animations.sort_by_key(|record| record.asset_id);
    let render_content_catalog = compile_render_content_catalog_v1(&render_records)?;
    let world_navigation_catalog =
        world_navigation_catalog_or_none.ok_or(ProjectActivationError::MissingReference)?;
    let world_population_catalog =
        world_population_catalog_or_none.ok_or(ProjectActivationError::MissingReference)?;
    let agent_cognition_catalog =
        agent_cognition_catalog_or_none.ok_or(ProjectActivationError::MissingReference)?;
    let world_activity_catalog =
        world_activity_catalog_or_none.ok_or(ProjectActivationError::MissingReference)?;
    world_population_catalog
        .validate_against_navigation(&world_navigation_catalog)
        .map_err(|_| ProjectActivationError::HashMismatch)?;
    if world_population_catalog
        .revision(&world_navigation_catalog)
        .map_err(|_| ProjectActivationError::HashMismatch)?
        != entries
            .get(&world_population_catalog.catalog_asset_id)
            .ok_or(ProjectActivationError::MissingReference)?
            .asset_revision
            .record_sha256
    {
        return Err(ProjectActivationError::HashMismatch);
    }
    let published_catalog = RenderContentCatalogV1::from_canonical_bytes(
        required_file(&generation.files, RENDER_CONTENT_CATALOG_PATH)?,
        limits,
    )?;
    if published_catalog != render_content_catalog {
        return Err(ProjectActivationError::HashMismatch);
    }
    for cooked_mesh in render_content_catalog.cooked_meshes() {
        let path = format!(
            "{RENDER_CONTENT_MESH_DIRECTORY}/{}.bin",
            cooked_mesh.payload_sha256().to_hex()
        );
        expected_files.insert(path.clone());
        let published =
            B0CookedMeshV1::from_canonical_bytes(required_file(&generation.files, &path)?, limits)?;
        if &published != cooked_mesh {
            return Err(ProjectActivationError::HashMismatch);
        }
    }
    if generation.files.keys().cloned().collect::<BTreeSet<_>>() != expected_files {
        return Err(ProjectActivationError::UnexpectedArtifact);
    }
    let activated = ActivatedProjectV7 {
        project_lock,
        schema_registry,
        content_manifest,
        world_partition,
        neutral_records: neutral_records.clone(),
        text_catalogs,
        audio_clips,
        neutral_skeletons,
        neutral_animations,
        rpg_definitions: activate_rpg_definitions_v2(
            &neutral_records,
            world_routine_catalog_or_none.as_ref(),
            required_file(&generation.files, RPG_DEFINITIONS_PATH)?,
        )
        .map_err(ProjectActivationError::Cook)?,
        world_routine_catalog_or_none,
        world_navigation_catalog,
        world_population_catalog,
        agent_cognition_catalog,
        world_activity_catalog,
        render_content_catalog,
    };
    activated.validate()?;
    Ok(activated)
}

fn required_file<'a>(
    files: &'a BTreeMap<String, Vec<u8>>,
    path: &str,
) -> Result<&'a [u8], ProjectActivationError> {
    files
        .get(path)
        .map(Vec::as_slice)
        .ok_or_else(|| ProjectActivationError::MissingArtifact(path.to_owned()))
}

fn require_schema(
    current_schemas: &BTreeSet<SchemaRefV1>,
    schema_ref: &SchemaRefV1,
) -> Result<(), ProjectActivationError> {
    if current_schemas.contains(schema_ref) {
        Ok(())
    } else {
        Err(ProjectActivationError::MissingSchema)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ProjectActivationError {
    Store(ContentStoreError),
    Contract(ProjectContractError),
    Neutral(NeutralRecordError),
    Render(RenderContentContractError),
    Localization(TextCatalogErrorV1),
    Audio(NeutralAudioErrorV1),
    Animation(NeutralAnimationContentErrorV1),
    Cook(crate::ProjectCookError),
    MissingArtifact(String),
    MissingReference,
    MissingSchema,
    HashMismatch,
    DependencyMismatch,
    UnexpectedArtifact,
}

impl ProjectActivationError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Store(_) | Self::MissingArtifact(_) => "PROJECT_ARTIFACT_MISSING",
            Self::Contract(ProjectContractError::UnsupportedFormat { .. }) => {
                "UNSUPPORTED_PROJECT_FORMAT"
            }
            Self::Contract(_)
            | Self::Neutral(_)
            | Self::Localization(_)
            | Self::Audio(_)
            | Self::Animation(_) => "PROJECT_SCHEMA_INVALID",
            Self::Render(error) => error.diagnostic_code(),
            Self::Cook(_) => "PROJECT_DEFINITION_INVALID",
            Self::MissingReference => "PROJECT_REFERENCE_MISSING",
            Self::MissingSchema => "PROJECT_SCHEMA_MISSING",
            Self::HashMismatch => "PROJECT_HASH_MISMATCH",
            Self::DependencyMismatch => "PROJECT_DEPENDENCY_MISMATCH",
            Self::UnexpectedArtifact => "PROJECT_ARTIFACT_UNEXPECTED",
        }
    }
}

impl Display for ProjectActivationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "project store load failed: {error}"),
            Self::Contract(error) => write!(formatter, "project contract invalid: {error}"),
            Self::Neutral(error) => write!(formatter, "project record invalid: {error}"),
            Self::Render(error) => write!(formatter, "project render content invalid: {error}"),
            Self::Localization(error) => write!(formatter, "project text catalog invalid: {error}"),
            Self::Audio(error) => write!(formatter, "project audio clip invalid: {error}"),
            Self::Animation(error) => {
                write!(formatter, "project animation content invalid: {error}")
            }
            Self::Cook(error) => write!(formatter, "project definition compile failed: {error}"),
            Self::MissingArtifact(path) => write!(formatter, "project artifact missing: {path}"),
            Self::MissingReference => formatter.write_str("project reference missing"),
            Self::MissingSchema => formatter.write_str("project schema missing"),
            Self::HashMismatch => formatter.write_str("project hash mismatch"),
            Self::DependencyMismatch => formatter.write_str("project dependency closure mismatch"),
            Self::UnexpectedArtifact => {
                formatter.write_str("project generation has extra artifact")
            }
        }
    }
}

impl Error for ProjectActivationError {}

impl From<ContentStoreError> for ProjectActivationError {
    fn from(error: ContentStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<ProjectContractError> for ProjectActivationError {
    fn from(error: ProjectContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<NeutralRecordError> for ProjectActivationError {
    fn from(error: NeutralRecordError) -> Self {
        Self::Neutral(error)
    }
}

impl From<RenderContentContractError> for ProjectActivationError {
    fn from(error: RenderContentContractError) -> Self {
        Self::Render(error)
    }
}

impl From<TextCatalogErrorV1> for ProjectActivationError {
    fn from(error: TextCatalogErrorV1) -> Self {
        Self::Localization(error)
    }
}

impl From<NeutralAudioErrorV1> for ProjectActivationError {
    fn from(error: NeutralAudioErrorV1) -> Self {
        Self::Audio(error)
    }
}

impl From<NeutralAnimationContentErrorV1> for ProjectActivationError {
    fn from(error: NeutralAnimationContentErrorV1) -> Self {
        Self::Animation(error)
    }
}
