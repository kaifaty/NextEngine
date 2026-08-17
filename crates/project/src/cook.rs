use self::validate::validate_source;
use crate::cook_rpg::{compile_rpg_definitions_v2, rpg_definitions_v2_manifest_bytes};
use crate::cook_support::{
    ensure_unique, schema_ref, schema_ref_with_version, validate_text_catalog_closure,
};
use next_assets::{ContentPublicationV1, PublicationFileV1};
use next_contracts::animation_content::{
    NEUTRAL_ANIMATION_SCHEMA_ID, NEUTRAL_SKELETON_SCHEMA_ID, NeutralAnimationV1, NeutralSkeletonV1,
};
use next_contracts::audio::{NEUTRAL_AUDIO_SCHEMA_ID, NeutralAudioV1};
use next_contracts::body::{BODY_SCHEMA_ASSET_SCHEMA_ID, BodySchemaAssetV1};
use next_contracts::cognition::{
    AGENT_COGNITION_CATALOG_SCHEMA_ID, AgentCognitionCatalogV1, COGNITION_SCHEMA_VERSION,
};
use next_contracts::content::{NeutralRecordKindV1, NeutralRecordV1};
use next_contracts::identity::RuntimeDeterminismBundleV1;
use next_contracts::ids::{AssetId, ContentHash, ProjectId, SchemaId};
use next_contracts::localization::{TEXT_CATALOG_SCHEMA_ID, TextCatalogV1};
use next_contracts::mechanics::RpgDefinitionRegistryV2;
use next_contracts::platform::PresentationTargetKindV1;
use next_contracts::project::{
    AssetRevisionRefV1, ContentAssetEntryV1, ContentDependencyEdgeV1, ContentManifestBodyV1,
    ContentManifestV1, ContentProvenanceV1, ContentSemanticClassV1, ProjectLockV3,
    SchemaDescriptorV1, SchemaEncodingV1, SchemaRegistryManifestBodyV2, SchemaRegistryManifestV2,
    SchemaRoleV1, WorldChunkBindingV1, WorldPartitionManifestBodyV1, WorldPartitionManifestV1,
    canonical_empty_manifest_hash, domain_hash,
};
use next_contracts::render_content::{
    NeutralRenderRecordV1, RenderContentCatalogV1, RenderContentContractError,
};
use next_contracts::world_activity::{WORLD_ACTIVITY_CATALOG_SCHEMA_ID, WorldActivityCatalogV1};
use next_contracts::world_population::{
    WORLD_NAVIGATION_CATALOG_SCHEMA_ID, WORLD_POPULATION_CATALOG_SCHEMA_ID,
    WorldNavigationCatalogV1, WorldPopulationCatalogV1,
};
use next_contracts::world_routine::{
    WORLD_ROUTINE_CATALOG_SCHEMA_ID, WorldRoutineCatalogV1, WorldRoutineInteractionBindingV1,
};
use std::collections::{BTreeMap, BTreeSet};
pub const PROJECT_LOCK_PATH: &str = "manifests/project-lock.json";
pub const SCHEMA_REGISTRY_PATH: &str = "manifests/schema-registry.json";
pub const CONTENT_MANIFEST_PATH: &str = "manifests/content.json";
pub const WORLD_PARTITION_PATH: &str = "manifests/world-partition.json";
pub const CONTENT_BLOB_DIRECTORY: &str = "blobs";
pub const RENDER_CONTENT_CATALOG_PATH: &str = "render-content/catalog.bin";
pub const RENDER_CONTENT_MESH_DIRECTORY: &str = "render-content/meshes";
pub const RPG_DEFINITIONS_PATH: &str = "mechanics/rpg-definitions-v2.bin";
pub const CORE_INTERACTION_PACKAGE_ID: &str = "org.nextengine.core.interaction";
pub const CORE_COMBAT_PACKAGE_ID: &str = "org.nextengine.core.combat";

pub(crate) fn launch_profiles_sha256() -> ContentHash {
    domain_hash(
        "nextengine.launch-profiles.v1",
        b"game:interactive|none;headless:none;tools:none|interactive;capture-worker:displayless-offscreen",
    )
}

pub(crate) fn platform_capability_profile_sha256() -> ContentHash {
    domain_hash(
        "nextengine.platform-capability-profile.v1",
        b"normalized-engine-owned-capabilities",
    )
}

pub(crate) fn platform_timebase_profile_sha256() -> ContentHash {
    domain_hash(
        "nextengine.platform-timebase-profile.v1",
        b"diagnostic-only-monotonic-v1",
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceChunkBindingV1 {
    pub chunk_id: SchemaId,
    pub region_id: SchemaId,
    pub chunk_asset_id: AssetId,
    pub required_asset_ids: Vec<AssetId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralProjectSourceV7 {
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub authoring_sha256: ContentHash,
    pub records: Vec<NeutralRecordV1>,
    pub render_records: Vec<NeutralRenderRecordV1>,
    pub text_catalogs: Vec<TextCatalogV1>,
    pub audio_records: Vec<NeutralAudioV1>,
    pub skeletons: Vec<NeutralSkeletonV1>,
    pub animations: Vec<NeutralAnimationV1>,
    pub body_schema_asset: BodySchemaAssetV1,
    pub world_routine_catalog_or_none: Option<WorldRoutineCatalogV1>,
    pub world_routine_interaction_binding_or_none: Option<WorldRoutineInteractionBindingV1>,
    pub world_navigation_catalog: WorldNavigationCatalogV1,
    pub world_population_catalog: WorldPopulationCatalogV1,
    pub agent_cognition_catalog: AgentCognitionCatalogV1,
    pub world_activity_catalog: WorldActivityCatalogV1,
    pub root_asset_ids: Vec<AssetId>,
    pub provenance: ContentProvenanceV1,
    pub license_manifest_sha256: ContentHash,
    pub partition_id: SchemaId,
    pub coordinate_profile_id: SchemaId,
    pub chunks: Vec<SourceChunkBindingV1>,
    pub allowed_presentation_targets: Vec<PresentationTargetKindV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CookedProjectV7 {
    pub project_lock: ProjectLockV3,
    pub schema_registry: SchemaRegistryManifestV2,
    pub content_manifest: ContentManifestV1,
    pub world_partition: WorldPartitionManifestV1,
    pub rpg_definitions: RpgDefinitionRegistryV2,
    pub world_routine_catalog_or_none: Option<WorldRoutineCatalogV1>,
    pub world_navigation_catalog: WorldNavigationCatalogV1,
    pub world_population_catalog: WorldPopulationCatalogV1,
    pub agent_cognition_catalog: AgentCognitionCatalogV1,
    pub world_activity_catalog: WorldActivityCatalogV1,
    pub render_content_catalog: RenderContentCatalogV1,
    pub body_schema_asset: BodySchemaAssetV1,
    pub blobs: BTreeMap<ContentHash, Vec<u8>>,
}

impl CookedProjectV7 {
    pub fn publication(&self) -> Result<ContentPublicationV1, ProjectCookError> {
        let mut files = vec![
            PublicationFileV1::new(PROJECT_LOCK_PATH, self.project_lock.to_jcs_bytes())?,
            PublicationFileV1::new(SCHEMA_REGISTRY_PATH, self.schema_registry.to_jcs_bytes()?)?,
            PublicationFileV1::new(CONTENT_MANIFEST_PATH, self.content_manifest.to_jcs_bytes()?)?,
            PublicationFileV1::new(WORLD_PARTITION_PATH, self.world_partition.to_jcs_bytes()?)?,
            PublicationFileV1::new(
                RENDER_CONTENT_CATALOG_PATH,
                self.render_content_catalog.canonical_bytes()?,
            )?,
            PublicationFileV1::new(
                RPG_DEFINITIONS_PATH,
                rpg_definitions_v2_manifest_bytes(&self.rpg_definitions)?,
            )?,
        ];
        files.extend(
            self.render_content_catalog
                .cooked_meshes()
                .iter()
                .map(|mesh| {
                    Ok(PublicationFileV1::new(
                        format!(
                            "{RENDER_CONTENT_MESH_DIRECTORY}/{}.bin",
                            mesh.payload_sha256().to_hex()
                        ),
                        mesh.canonical_bytes()?,
                    )?)
                })
                .collect::<Result<Vec<_>, ProjectCookError>>()?,
        );
        files.extend(
            self.blobs
                .iter()
                .map(|(hash, bytes)| {
                    PublicationFileV1::new(
                        format!("{CONTENT_BLOB_DIRECTORY}/{}.bin", hash.to_hex()),
                        bytes.clone(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(ContentPublicationV1::new(
            self.project_lock.project_lock_sha256,
            files,
        )?)
    }
}

pub fn cook_project_v7(
    mut source: NeutralProjectSourceV7,
) -> Result<CookedProjectV7, ProjectCookError> {
    source.records.sort_by_key(|record| record.asset_id);
    source
        .render_records
        .sort_by_key(NeutralRenderRecordV1::asset_id);
    source
        .text_catalogs
        .sort_by_key(|catalog| catalog.catalog_asset_id);
    source.audio_records.sort_by_key(|record| record.asset_id);
    source.skeletons.sort_by_key(|record| record.asset_id);
    source.animations.sort_by_key(|record| record.asset_id);
    source.root_asset_ids.sort();
    source
        .chunks
        .sort_by(|left, right| left.chunk_id.as_str().cmp(right.chunk_id.as_str()));
    validate_source(&source)?;

    let canonicalization_profile_sha256 = domain_hash(
        "nextengine.canonicalization-profile.v1",
        b"jcs+canonical-binary-v1",
    );
    let ownership_registry_sha256 = domain_hash(
        "nextengine.schema-ownership-registry.v1",
        b"nextengine.assets",
    );
    let mut schema_refs: BTreeSet<_> = source
        .records
        .iter()
        .map(|record| record.schema_ref.clone())
        .collect();
    schema_refs.extend(
        source
            .render_records
            .iter()
            .map(|record| record.schema_ref().clone()),
    );
    let text_catalog_schema_ref = schema_ref(
        TEXT_CATALOG_SCHEMA_ID,
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    if !source.text_catalogs.is_empty() {
        schema_refs.insert(text_catalog_schema_ref.clone());
    }
    let audio_schema_ref = schema_ref(
        NEUTRAL_AUDIO_SCHEMA_ID,
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    if !source.audio_records.is_empty() {
        schema_refs.insert(audio_schema_ref.clone());
    }
    let skeleton_schema_ref = schema_ref(
        NEUTRAL_SKELETON_SCHEMA_ID,
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    if !source.skeletons.is_empty() {
        schema_refs.insert(skeleton_schema_ref.clone());
    }
    let animation_schema_ref = schema_ref(
        NEUTRAL_ANIMATION_SCHEMA_ID,
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    if !source.animations.is_empty() {
        schema_refs.insert(animation_schema_ref.clone());
    }
    let body_schema_asset_schema_ref = schema_ref(
        BODY_SCHEMA_ASSET_SCHEMA_ID,
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    schema_refs.insert(body_schema_asset_schema_ref.clone());
    let world_routine_catalog_schema_ref = schema_ref(
        WORLD_ROUTINE_CATALOG_SCHEMA_ID,
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    if source.world_routine_catalog_or_none.is_some() {
        schema_refs.insert(world_routine_catalog_schema_ref.clone());
    }
    let world_navigation_catalog_schema_ref = schema_ref(
        WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    schema_refs.insert(world_navigation_catalog_schema_ref.clone());
    let world_population_catalog_schema_ref = schema_ref(
        WORLD_POPULATION_CATALOG_SCHEMA_ID,
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    schema_refs.insert(world_population_catalog_schema_ref.clone());
    let agent_cognition_catalog_schema_ref = schema_ref_with_version(
        AGENT_COGNITION_CATALOG_SCHEMA_ID,
        u32::from(COGNITION_SCHEMA_VERSION),
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    schema_refs.insert(agent_cognition_catalog_schema_ref.clone());
    let world_activity_catalog_schema_ref = schema_ref(
        WORLD_ACTIVITY_CATALOG_SCHEMA_ID,
        SchemaRoleV1::NeutralContent,
        SchemaEncodingV1::CanonicalBinaryV1,
    )?;
    schema_refs.insert(world_activity_catalog_schema_ref.clone());
    let content_schema_ref = schema_ref(
        "nextengine.content.manifest",
        SchemaRoleV1::Manifest,
        SchemaEncodingV1::JcsRfc8785,
    )?;
    schema_refs.insert(content_schema_ref.clone());
    let descriptors: Vec<_> = schema_refs
        .iter()
        .cloned()
        .map(|schema_ref| SchemaDescriptorV1 {
            field_registry_sha256: domain_hash(
                "nextengine.schema-field-registry.v1",
                schema_ref.schema_id.as_str().as_bytes(),
            ),
            owner_context_id: SchemaId::new("nextengine.assets")
                .expect("engine-owned identifier is valid"),
            schema_ref,
        })
        .collect();
    let schema_registry = SchemaRegistryManifestV2::new(SchemaRegistryManifestBodyV2 {
        registry_revision: 1,
        canonicalization_profile_sha256,
        ownership_registry_sha256,
        descriptors,
        current_schema_refs: schema_refs.into_iter().collect(),
    })?;

    let mut blobs = BTreeMap::new();
    let mut revisions = BTreeMap::new();
    let mut entries = Vec::new();
    let mut edges = Vec::new();
    for record in &source.records {
        let bytes = record.canonical_bytes()?;
        let record_hash = record.record_sha256()?;
        if blobs.insert(record_hash, bytes).is_some() {
            return Err(ProjectCookError::HashCollision);
        }
        let revision = AssetRevisionRefV1 {
            asset_id: record.asset_id,
            record_sha256: record_hash,
        };
        if revisions.insert(record.asset_id, revision).is_some() {
            return Err(ProjectCookError::DuplicateIdentity);
        }
        entries.push(ContentAssetEntryV1 {
            asset_revision: revision,
            schema_ref: record.schema_ref.clone(),
            neutral_record_blob_sha256: record_hash,
            semantic_class: ContentSemanticClassV1::DomainRelevant,
            provenance_sha256: source.provenance.provenance_sha256,
            license_manifest_sha256: source.license_manifest_sha256,
            owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
                .expect("engine-owned identifier is valid"),
        });
        for target in &record.asset_dependencies {
            edges.push(ContentDependencyEdgeV1 {
                source_asset_id: record.asset_id,
                target_asset_id: *target,
                dependency_kind: SchemaId::new("nextengine.content.required")
                    .expect("engine-owned identifier is valid"),
                required: true,
            });
        }
    }
    for record in &source.render_records {
        let bytes = record.canonical_bytes()?;
        let record_hash = record.record_sha256()?;
        if blobs.insert(record_hash, bytes).is_some() {
            return Err(ProjectCookError::HashCollision);
        }
        let revision = record.asset_revision()?;
        if revisions.insert(record.asset_id(), revision).is_some() {
            return Err(ProjectCookError::DuplicateIdentity);
        }
        entries.push(ContentAssetEntryV1 {
            asset_revision: revision,
            schema_ref: record.schema_ref().clone(),
            neutral_record_blob_sha256: record_hash,
            semantic_class: record.semantic_class(),
            provenance_sha256: source.provenance.provenance_sha256,
            license_manifest_sha256: source.license_manifest_sha256,
            owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
                .expect("engine-owned identifier is valid"),
        });
        for target in record.dependencies() {
            edges.push(ContentDependencyEdgeV1 {
                source_asset_id: record.asset_id(),
                target_asset_id: target.asset_id,
                dependency_kind: SchemaId::new("nextengine.content.required")
                    .expect("engine-owned identifier is valid"),
                required: true,
            });
        }
    }
    let render_content_catalog = compile_render_content_catalog_v1(&source.render_records)?;
    for catalog in &source.text_catalogs {
        let bytes = catalog
            .canonical_bytes()
            .map_err(|_| ProjectCookError::InvalidValue)?;
        let record_hash = catalog.record_sha256()?;
        if blobs.insert(record_hash, bytes).is_some() {
            return Err(ProjectCookError::HashCollision);
        }
        let revision = AssetRevisionRefV1 {
            asset_id: catalog.catalog_asset_id,
            record_sha256: record_hash,
        };
        if revisions
            .insert(catalog.catalog_asset_id, revision)
            .is_some()
        {
            return Err(ProjectCookError::DuplicateIdentity);
        }
        entries.push(ContentAssetEntryV1 {
            asset_revision: revision,
            schema_ref: text_catalog_schema_ref.clone(),
            neutral_record_blob_sha256: record_hash,
            semantic_class: ContentSemanticClassV1::PresentationOnly,
            provenance_sha256: source.provenance.provenance_sha256,
            license_manifest_sha256: source.license_manifest_sha256,
            owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
                .expect("engine-owned identifier is valid"),
        });
    }
    for record in &source.audio_records {
        let bytes = record.canonical_bytes()?;
        let record_hash = record.record_sha256()?;
        if blobs.insert(record_hash, bytes).is_some() {
            return Err(ProjectCookError::HashCollision);
        }
        let revision = AssetRevisionRefV1 {
            asset_id: record.asset_id,
            record_sha256: record_hash,
        };
        if revisions.insert(record.asset_id, revision).is_some() {
            return Err(ProjectCookError::DuplicateIdentity);
        }
        entries.push(ContentAssetEntryV1 {
            asset_revision: revision,
            schema_ref: audio_schema_ref.clone(),
            neutral_record_blob_sha256: record_hash,
            semantic_class: ContentSemanticClassV1::PresentationOnly,
            provenance_sha256: source.provenance.provenance_sha256,
            license_manifest_sha256: source.license_manifest_sha256,
            owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
                .expect("engine-owned identifier is valid"),
        });
    }
    for record in &source.skeletons {
        let bytes = record.canonical_bytes()?;
        let record_hash = record.record_sha256()?;
        if blobs.insert(record_hash, bytes).is_some() {
            return Err(ProjectCookError::HashCollision);
        }
        let revision = record.asset_revision()?;
        if revisions.insert(record.asset_id, revision).is_some() {
            return Err(ProjectCookError::DuplicateIdentity);
        }
        entries.push(ContentAssetEntryV1 {
            asset_revision: revision,
            schema_ref: skeleton_schema_ref.clone(),
            neutral_record_blob_sha256: record_hash,
            semantic_class: ContentSemanticClassV1::DomainRelevant,
            provenance_sha256: source.provenance.provenance_sha256,
            license_manifest_sha256: source.license_manifest_sha256,
            owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
                .expect("engine-owned identifier is valid"),
        });
    }
    for record in &source.animations {
        let bytes = record.canonical_bytes()?;
        let record_hash = record.record_sha256()?;
        if blobs.insert(record_hash, bytes).is_some() {
            return Err(ProjectCookError::HashCollision);
        }
        let revision = record.asset_revision()?;
        if revisions.insert(record.asset_id, revision).is_some() {
            return Err(ProjectCookError::DuplicateIdentity);
        }
        entries.push(ContentAssetEntryV1 {
            asset_revision: revision,
            schema_ref: animation_schema_ref.clone(),
            neutral_record_blob_sha256: record_hash,
            semantic_class: ContentSemanticClassV1::DomainRelevant,
            provenance_sha256: source.provenance.provenance_sha256,
            license_manifest_sha256: source.license_manifest_sha256,
            owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
                .expect("engine-owned identifier is valid"),
        });
        edges.push(ContentDependencyEdgeV1 {
            source_asset_id: record.asset_id,
            target_asset_id: record.skeleton_revision.asset_id,
            dependency_kind: SchemaId::new("nextengine.content.required")
                .expect("engine-owned identifier is valid"),
            required: true,
        });
    }
    let body_schema_bytes = source
        .body_schema_asset
        .canonical_bytes()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    let body_schema_hash = source
        .body_schema_asset
        .record_sha256()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    if blobs.insert(body_schema_hash, body_schema_bytes).is_some() {
        return Err(ProjectCookError::HashCollision);
    }
    let body_schema_revision = AssetRevisionRefV1 {
        asset_id: source.body_schema_asset.asset_id,
        record_sha256: body_schema_hash,
    };
    if revisions
        .insert(source.body_schema_asset.asset_id, body_schema_revision)
        .is_some()
    {
        return Err(ProjectCookError::DuplicateIdentity);
    }
    entries.push(ContentAssetEntryV1 {
        asset_revision: body_schema_revision,
        schema_ref: body_schema_asset_schema_ref,
        neutral_record_blob_sha256: body_schema_hash,
        semantic_class: ContentSemanticClassV1::DomainRelevant,
        provenance_sha256: source.provenance.provenance_sha256,
        license_manifest_sha256: source.license_manifest_sha256,
        owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
            .expect("engine-owned identifier is valid"),
    });
    let navigation_bytes = source
        .world_navigation_catalog
        .canonical_bytes()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    let navigation_hash = source
        .world_navigation_catalog
        .revision()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    if blobs.insert(navigation_hash, navigation_bytes).is_some() {
        return Err(ProjectCookError::HashCollision);
    }
    let navigation_revision = AssetRevisionRefV1 {
        asset_id: source.world_navigation_catalog.catalog_asset_id,
        record_sha256: navigation_hash,
    };
    if revisions
        .insert(
            source.world_navigation_catalog.catalog_asset_id,
            navigation_revision,
        )
        .is_some()
    {
        return Err(ProjectCookError::DuplicateIdentity);
    }
    entries.push(ContentAssetEntryV1 {
        asset_revision: navigation_revision,
        schema_ref: world_navigation_catalog_schema_ref,
        neutral_record_blob_sha256: navigation_hash,
        semantic_class: ContentSemanticClassV1::DomainRelevant,
        provenance_sha256: source.provenance.provenance_sha256,
        license_manifest_sha256: source.license_manifest_sha256,
        owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
            .expect("engine-owned identifier is valid"),
    });

    let population_bytes = source
        .world_population_catalog
        .canonical_bytes()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    let population_hash = source
        .world_population_catalog
        .revision(&source.world_navigation_catalog)
        .map_err(|_| ProjectCookError::InvalidValue)?;
    if blobs.insert(population_hash, population_bytes).is_some() {
        return Err(ProjectCookError::HashCollision);
    }
    let population_revision = AssetRevisionRefV1 {
        asset_id: source.world_population_catalog.catalog_asset_id,
        record_sha256: population_hash,
    };
    if revisions
        .insert(
            source.world_population_catalog.catalog_asset_id,
            population_revision,
        )
        .is_some()
    {
        return Err(ProjectCookError::DuplicateIdentity);
    }
    entries.push(ContentAssetEntryV1 {
        asset_revision: population_revision,
        schema_ref: world_population_catalog_schema_ref,
        neutral_record_blob_sha256: population_hash,
        semantic_class: ContentSemanticClassV1::DomainRelevant,
        provenance_sha256: source.provenance.provenance_sha256,
        license_manifest_sha256: source.license_manifest_sha256,
        owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
            .expect("engine-owned identifier is valid"),
    });
    edges.push(ContentDependencyEdgeV1 {
        source_asset_id: source.world_population_catalog.catalog_asset_id,
        target_asset_id: source.world_navigation_catalog.catalog_asset_id,
        dependency_kind: SchemaId::new("nextengine.content.required")
            .expect("engine-owned identifier is valid"),
        required: true,
    });
    let cognition_bytes = source
        .agent_cognition_catalog
        .canonical_bytes()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    let cognition_hash = source
        .agent_cognition_catalog
        .revision()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    if blobs.insert(cognition_hash, cognition_bytes).is_some() {
        return Err(ProjectCookError::HashCollision);
    }
    let cognition_revision = AssetRevisionRefV1 {
        asset_id: source.agent_cognition_catalog.catalog_asset_id,
        record_sha256: cognition_hash,
    };
    if revisions
        .insert(
            source.agent_cognition_catalog.catalog_asset_id,
            cognition_revision,
        )
        .is_some()
    {
        return Err(ProjectCookError::DuplicateIdentity);
    }
    entries.push(ContentAssetEntryV1 {
        asset_revision: cognition_revision,
        schema_ref: agent_cognition_catalog_schema_ref,
        neutral_record_blob_sha256: cognition_hash,
        semantic_class: ContentSemanticClassV1::DomainRelevant,
        provenance_sha256: source.provenance.provenance_sha256,
        license_manifest_sha256: source.license_manifest_sha256,
        owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
            .expect("engine-owned identifier is valid"),
    });
    edges.push(ContentDependencyEdgeV1 {
        source_asset_id: source.agent_cognition_catalog.catalog_asset_id,
        target_asset_id: source.world_population_catalog.catalog_asset_id,
        dependency_kind: SchemaId::new("nextengine.content.required")
            .expect("engine-owned identifier is valid"),
        required: true,
    });
    let activity_bytes = source
        .world_activity_catalog
        .canonical_bytes()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    let activity_hash = source
        .world_activity_catalog
        .revision()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    if blobs.insert(activity_hash, activity_bytes).is_some() {
        return Err(ProjectCookError::HashCollision);
    }
    let activity_revision = AssetRevisionRefV1 {
        asset_id: source.world_activity_catalog.catalog_asset_id,
        record_sha256: activity_hash,
    };
    if revisions
        .insert(
            source.world_activity_catalog.catalog_asset_id,
            activity_revision,
        )
        .is_some()
    {
        return Err(ProjectCookError::DuplicateIdentity);
    }
    entries.push(ContentAssetEntryV1 {
        asset_revision: activity_revision,
        schema_ref: world_activity_catalog_schema_ref,
        neutral_record_blob_sha256: activity_hash,
        semantic_class: ContentSemanticClassV1::DomainRelevant,
        provenance_sha256: source.provenance.provenance_sha256,
        license_manifest_sha256: source.license_manifest_sha256,
        owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
            .expect("engine-owned identifier is valid"),
    });
    edges.push(ContentDependencyEdgeV1 {
        source_asset_id: source.world_activity_catalog.catalog_asset_id,
        target_asset_id: source.world_population_catalog.catalog_asset_id,
        dependency_kind: SchemaId::new("nextengine.content.required")
            .expect("engine-owned identifier is valid"),
        required: true,
    });
    if let Some(catalog) = source.world_routine_catalog_or_none {
        let bytes = catalog
            .canonical_bytes()
            .map_err(|_| ProjectCookError::InvalidValue)?;
        let record_hash = catalog
            .revision()
            .map_err(|_| ProjectCookError::InvalidValue)?;
        if blobs.insert(record_hash, bytes).is_some() {
            return Err(ProjectCookError::HashCollision);
        }
        let revision = AssetRevisionRefV1 {
            asset_id: catalog.catalog_asset_id,
            record_sha256: record_hash,
        };
        if revisions
            .insert(catalog.catalog_asset_id, revision)
            .is_some()
        {
            return Err(ProjectCookError::DuplicateIdentity);
        }
        entries.push(ContentAssetEntryV1 {
            asset_revision: revision,
            schema_ref: world_routine_catalog_schema_ref,
            neutral_record_blob_sha256: record_hash,
            semantic_class: ContentSemanticClassV1::DomainRelevant,
            provenance_sha256: source.provenance.provenance_sha256,
            license_manifest_sha256: source.license_manifest_sha256,
            owning_bundle_id: SchemaId::new("nextengine.fixture.bundle.v1")
                .expect("engine-owned identifier is valid"),
        });
    }
    let root_assets = source
        .root_asset_ids
        .iter()
        .map(|asset_id| {
            revisions
                .get(asset_id)
                .copied()
                .ok_or(ProjectCookError::MissingReference)
        })
        .collect::<Result<_, _>>()?;
    let content_manifest = ContentManifestV1::new(ContentManifestBodyV1 {
        schema_ref: content_schema_ref,
        manifest_id: SchemaId::new("nextengine.fixture.content-manifest.v1")
            .expect("engine-owned identifier is valid"),
        project_id: source.project_id.clone(),
        content_revision: source.project_revision,
        schema_registry_manifest_sha256: schema_registry.schema_registry_manifest_sha256,
        canonicalization_profile_sha256,
        content_admission_limits_sha256: domain_hash(
            "nextengine.content-admission-limits.v1",
            b"fixture-bounded-v1",
        ),
        cooker_contract_sha256: domain_hash(
            "nextengine.cooker-contract.v1",
            b"next_project::cook_project_v7",
        ),
        cooker_options_sha256: canonical_empty_manifest_hash("nextengine.cooker-options.v1"),
        root_assets,
        provenance_records: vec![source.provenance.clone()],
        asset_entries: entries,
        dependency_edges: edges,
        domain_closure_sha256: ContentHash::default(),
    })?;
    let rpg_definitions = compile_rpg_definitions_v2(
        &source.records,
        source.world_routine_interaction_binding_or_none.as_ref(),
    )?;

    let mut root_region_ids: Vec<_> = source
        .chunks
        .iter()
        .map(|chunk| chunk.region_id.clone())
        .collect();
    root_region_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    root_region_ids.dedup();
    let chunk_bindings = source
        .chunks
        .iter()
        .map(|chunk| {
            Ok(WorldChunkBindingV1 {
                chunk_id: chunk.chunk_id.clone(),
                region_id: chunk.region_id.clone(),
                chunk_asset: revisions
                    .get(&chunk.chunk_asset_id)
                    .copied()
                    .ok_or(ProjectCookError::MissingReference)?,
                required_asset_ids: chunk.required_asset_ids.clone(),
            })
        })
        .collect::<Result<_, ProjectCookError>>()?;
    let world_partition = WorldPartitionManifestV1::new(WorldPartitionManifestBodyV1 {
        partition_id: source.partition_id,
        coordinate_profile_id: source.coordinate_profile_id,
        topology_revision: source.project_revision,
        root_region_ids,
        chunk_bindings,
        initial_placement_catalog_sha256: population_hash,
        residency_policy_sha256: source
            .world_population_catalog
            .residency_policy_revision(&source.world_navigation_catalog)
            .map_err(|_| ProjectCookError::InvalidValue)?,
        schema_registry_manifest_sha256: schema_registry.schema_registry_manifest_sha256,
        content_manifest_sha256: content_manifest.content_manifest_sha256,
    })?;

    let project_lock = ProjectLockV3::new(ProjectLockV3 {
        project_id: source.project_id,
        project_revision: source.project_revision,
        authoring_sha256: source.authoring_sha256,
        schema_registry_manifest_sha256: schema_registry.schema_registry_manifest_sha256,
        content_manifest_sha256: content_manifest.content_manifest_sha256,
        world_partition_manifest_sha256: world_partition.world_partition_manifest_sha256,
        mechanics_lock_sha256: rpg_definitions.mechanics_lock.mechanics_lock_sha256,
        runtime_determinism_profile_sha256: RuntimeDeterminismBundleV1::core_r5c()
            .expect("the engine-owned determinism bundle is canonical")
            .runtime_profile_hash(),
        launch_profiles_sha256: launch_profiles_sha256(),
        platform_capability_profile_sha256: platform_capability_profile_sha256(),
        platform_timebase_profile_sha256: platform_timebase_profile_sha256(),
        allowed_presentation_targets: source.allowed_presentation_targets,
        project_lock_sha256: ContentHash::default(),
    })?;
    Ok(CookedProjectV7 {
        project_lock,
        schema_registry,
        content_manifest,
        world_partition,
        rpg_definitions,
        world_routine_catalog_or_none: source.world_routine_catalog_or_none,
        world_navigation_catalog: source.world_navigation_catalog,
        world_population_catalog: source.world_population_catalog,
        agent_cognition_catalog: source.agent_cognition_catalog,
        world_activity_catalog: source.world_activity_catalog,
        render_content_catalog,
        body_schema_asset: source.body_schema_asset,
        blobs,
    })
}

pub(crate) fn compile_render_content_catalog_v1(
    records: &[NeutralRenderRecordV1],
) -> Result<RenderContentCatalogV1, RenderContentContractError> {
    let mut profile = None;
    let mut meshes = Vec::new();
    let mut materials = Vec::new();
    let mut textures = Vec::new();
    for record in records {
        match record {
            NeutralRenderRecordV1::Mesh(value) => meshes.push(value.clone()),
            NeutralRenderRecordV1::Material(value) => materials.push(value.clone()),
            NeutralRenderRecordV1::Texture(value) => textures.push(value.clone()),
            NeutralRenderRecordV1::Profile(value) => {
                if profile.replace(value.clone()).is_some() {
                    return Err(RenderContentContractError::DuplicateIdentity);
                }
            }
        }
    }
    let profile = profile.ok_or(RenderContentContractError::MissingReference)?;
    RenderContentCatalogV1::new(profile, meshes, materials, textures)
}

pub(crate) fn asset_revision(
    record: &NeutralRecordV1,
) -> Result<AssetRevisionRefV1, ProjectCookError> {
    Ok(AssetRevisionRefV1 {
        asset_id: record.asset_id,
        record_sha256: record.record_sha256()?,
    })
}

mod error;
mod validate;

pub use error::ProjectCookError;
