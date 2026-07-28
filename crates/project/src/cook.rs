use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_assets::{ContentPublicationV1, PublicationFileV1};
use next_contracts::{
    AssetId, AssetRevisionRefV1, ContentAssetEntryV1, ContentDependencyEdgeV1, ContentHash,
    ContentManifestBodyV1, ContentManifestV1, ContentProvenanceV1, ContentSemanticClassV1,
    NeutralPropertyV1, NeutralRecordError, NeutralRecordKindV1, NeutralRecordV1, PersistentId,
    ProjectCatalogRecordV1, ProjectCatalogSnapshotV1, ProjectCompositionLockV1,
    ProjectContractError, ProjectDependencyKindV1, ProjectId, ProjectManifestV1,
    ProjectRequirementV1, SchemaDescriptorV1, SchemaEncodingV1, SchemaId, SchemaRefV1,
    SchemaRegistryManifestBodyV1, SchemaRegistryManifestV1, SchemaRoleV1, SemanticVersionV1,
    WorldChunkBindingV1, WorldPartitionManifestBodyV1, WorldPartitionManifestV1,
    canonical_empty_manifest_hash, domain_hash,
};

use crate::{ProjectResolutionError, resolve_project_records_v1};

pub const PROJECT_MANIFEST_PATH: &str = "manifests/project.json";
pub const PROJECT_CATALOG_PATH: &str = "manifests/catalog.json";
pub const PROJECT_COMPOSITION_LOCK_PATH: &str = "manifests/composition-lock.json";
pub const SCHEMA_REGISTRY_PATH: &str = "manifests/schema-registry.json";
pub const CONTENT_MANIFEST_PATH: &str = "manifests/content.json";
pub const WORLD_PARTITION_PATH: &str = "manifests/world-partition.json";
pub const CONTENT_BLOB_DIRECTORY: &str = "blobs";
pub const CORE_CONTENT_IDENTITY: &str = "org.nextengine.fixture.content";
pub const CORE_RESOLVER_PROFILE_ID: &str = "nextengine.resolver.exact-minimum.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceChunkBindingV1 {
    pub chunk_id: SchemaId,
    pub region_id: SchemaId,
    pub chunk_asset_id: AssetId,
    pub required_asset_ids: Vec<AssetId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralProjectSourceV1 {
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub content_identity: SchemaId,
    pub resolver_profile_id: SchemaId,
    pub resolver_profile_version: u32,
    pub records: Vec<NeutralRecordV1>,
    pub root_asset_ids: Vec<AssetId>,
    pub provenance: ContentProvenanceV1,
    pub license_manifest_sha256: ContentHash,
    pub partition_id: SchemaId,
    pub coordinate_profile_id: SchemaId,
    pub chunks: Vec<SourceChunkBindingV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CookedProjectV1 {
    pub project_manifest: ProjectManifestV1,
    pub catalog_snapshot: ProjectCatalogSnapshotV1,
    pub composition_lock: ProjectCompositionLockV1,
    pub schema_registry: SchemaRegistryManifestV1,
    pub content_manifest: ContentManifestV1,
    pub world_partition: WorldPartitionManifestV1,
    pub blobs: BTreeMap<ContentHash, Vec<u8>>,
}

impl CookedProjectV1 {
    pub fn publication(&self) -> Result<ContentPublicationV1, ProjectCookError> {
        let mut files = vec![
            PublicationFileV1::new(PROJECT_MANIFEST_PATH, self.project_manifest.to_jcs_bytes())?,
            PublicationFileV1::new(PROJECT_CATALOG_PATH, self.catalog_snapshot.to_jcs_bytes())?,
            PublicationFileV1::new(
                PROJECT_COMPOSITION_LOCK_PATH,
                self.composition_lock.to_jcs_bytes(),
            )?,
            PublicationFileV1::new(SCHEMA_REGISTRY_PATH, self.schema_registry.to_jcs_bytes()?)?,
            PublicationFileV1::new(CONTENT_MANIFEST_PATH, self.content_manifest.to_jcs_bytes()?)?,
            PublicationFileV1::new(WORLD_PARTITION_PATH, self.world_partition.to_jcs_bytes()?)?,
        ];
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
            self.composition_lock.composition_lock_sha256,
            files,
        )?)
    }
}

pub fn cook_project_v1(
    mut source: NeutralProjectSourceV1,
) -> Result<CookedProjectV1, ProjectCookError> {
    source.records.sort_by_key(|record| record.asset_id);
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
    let registry_limits_sha256 = domain_hash("nextengine.schema-registry-limits.v1", b"bounded-v1");
    let mut schema_refs: BTreeSet<_> = source
        .records
        .iter()
        .map(|record| record.schema_ref.clone())
        .collect();
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
    let schema_registry = SchemaRegistryManifestV1::new(SchemaRegistryManifestBodyV1 {
        registry_revision: 1,
        canonicalization_profile_sha256,
        ownership_registry_sha256,
        descriptors,
        current_schema_refs: schema_refs.into_iter().collect(),
        migration_dag_sha256: canonical_empty_manifest_hash("nextengine.schema-migration-dag.v1"),
        registry_limits_sha256,
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
        revisions.insert(record.asset_id, revision);
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
            b"next_project::cook_project_v1",
        ),
        cooker_options_sha256: canonical_empty_manifest_hash("nextengine.cooker-options.v1"),
        root_assets,
        provenance_records: vec![source.provenance.clone()],
        asset_entries: entries,
        dependency_edges: edges,
        domain_closure_sha256: ContentHash::default(),
    })?;

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
        initial_placement_catalog_sha256: canonical_empty_manifest_hash(
            "nextengine.initial-placement-catalog.v1",
        ),
        residency_policy_sha256: domain_hash("nextengine.residency-policy.v1", b"fixture-resident"),
        schema_registry_manifest_sha256: schema_registry.schema_registry_manifest_sha256,
        content_manifest_sha256: content_manifest.content_manifest_sha256,
    })?;

    let project_manifest = ProjectManifestV1::new(
        source.project_id.clone(),
        source.project_revision,
        vec![ProjectRequirementV1 {
            kind: ProjectDependencyKindV1::Content,
            identity: source.content_identity.clone(),
            minimum_version: SemanticVersionV1::new(1, 0, 0),
            optional: false,
        }],
    )?;
    let catalog_snapshot = ProjectCatalogSnapshotV1::new(
        source.resolver_profile_id.clone(),
        source.resolver_profile_version,
        source.provenance.provenance_sha256,
        vec![ProjectCatalogRecordV1::new(
            ProjectDependencyKindV1::Content,
            source.content_identity,
            SemanticVersionV1::new(1, 0, 0),
            content_manifest.content_manifest_sha256,
            Vec::new(),
            false,
        )?],
    )?;
    let selected_records = resolve_project_records_v1(&project_manifest, &catalog_snapshot)?;
    let mut resolver_bytes = Vec::new();
    resolver_bytes.extend_from_slice(source.resolver_profile_id.as_str().as_bytes());
    resolver_bytes.extend_from_slice(&source.resolver_profile_version.to_le_bytes());
    let composition_lock = ProjectCompositionLockV1::new(ProjectCompositionLockV1 {
        project_id: source.project_id,
        project_manifest_sha256: project_manifest.manifest_sha256,
        catalog_snapshot_sha256: catalog_snapshot.catalog_snapshot_sha256,
        resolver_profile_sha256: domain_hash(
            "nextengine.project-resolver-profile.v1",
            &resolver_bytes,
        ),
        schema_registry_manifest_sha256: schema_registry.schema_registry_manifest_sha256,
        content_manifest_sha256: content_manifest.content_manifest_sha256,
        world_partition_manifest_sha256: world_partition.world_partition_manifest_sha256,
        mechanics_lock_sha256: canonical_empty_manifest_hash("nextengine.mechanics-lock.v1"),
        selected_records,
        composition_lock_sha256: ContentHash::default(),
    })?;
    Ok(CookedProjectV1 {
        project_manifest,
        catalog_snapshot,
        composition_lock,
        schema_registry,
        content_manifest,
        world_partition,
        blobs,
    })
}

pub fn neutral_vertical_slice_source_v1() -> Result<NeutralProjectSourceV1, ProjectCookError> {
    let kinds = [
        NeutralRecordKindV1::Scene,
        NeutralRecordKindV1::Collider,
        NeutralRecordKindV1::CharacterDefinition,
        NeutralRecordKindV1::ItemDefinition,
        NeutralRecordKindV1::InventoryDefinition,
        NeutralRecordKindV1::EquipmentDefinition,
        NeutralRecordKindV1::DialogueDefinition,
        NeutralRecordKindV1::QuestDefinition,
        NeutralRecordKindV1::RelationshipDefinition,
        NeutralRecordKindV1::InteractionDefinition,
    ];
    let asset_ids: Vec<_> = (1_u8..=10)
        .map(|byte| AssetId::from_bytes([byte; 16]))
        .collect();
    let persistent_ids: Vec<_> = (31_u8..=40)
        .map(|byte| PersistentId::from_bytes([byte; 16]))
        .collect();
    let mut records = Vec::new();
    for (index, kind) in kinds.into_iter().enumerate() {
        let (persistent_references, asset_dependencies) = match kind {
            NeutralRecordKindV1::Scene => (persistent_ids[1..].to_vec(), asset_ids[1..].to_vec()),
            NeutralRecordKindV1::CharacterDefinition => (
                vec![persistent_ids[4], persistent_ids[5]],
                vec![asset_ids[4], asset_ids[5]],
            ),
            NeutralRecordKindV1::InteractionDefinition => (
                vec![persistent_ids[6], persistent_ids[7], persistent_ids[8]],
                vec![asset_ids[6], asset_ids[7], asset_ids[8]],
            ),
            _ => (Vec::new(), Vec::new()),
        };
        records.push(NeutralRecordV1::new(
            schema_ref(
                kind.schema_id(),
                SchemaRoleV1::Definition,
                SchemaEncodingV1::CanonicalBinaryV1,
            )?,
            asset_ids[index],
            kind,
            persistent_ids[index],
            persistent_references,
            asset_dependencies,
            vec![NeutralPropertyV1 {
                property_id: SchemaId::new("nextengine.fixture.role")
                    .expect("engine-owned identifier is valid"),
                value_id: SchemaId::new(format!("nextengine.fixture.{:?}", kind).to_lowercase())
                    .expect("engine-owned identifier is valid"),
            }],
        )?);
    }

    Ok(NeutralProjectSourceV1 {
        project_id: ProjectId::new("org.nextengine.fixture.vertical-slice")?,
        project_revision: 1,
        content_identity: SchemaId::new(CORE_CONTENT_IDENTITY)?,
        resolver_profile_id: SchemaId::new(CORE_RESOLVER_PROFILE_ID)?,
        resolver_profile_version: 1,
        records,
        root_asset_ids: vec![asset_ids[0]],
        provenance: ContentProvenanceV1::new(
            SchemaId::new("nextengine.fixture.provenance.cc0")?,
            SchemaId::new("CC0-1.0")?,
            "Next Engine generated neutral verification fixture; CC0-1.0",
        )?,
        license_manifest_sha256: domain_hash(
            "nextengine.license-manifest.v1",
            b"CC0-1.0\0Next Engine generated verification fixture",
        ),
        partition_id: SchemaId::new("nextengine.fixture.partition.v1")?,
        coordinate_profile_id: SchemaId::new("nextengine.coordinates.right-handed-metres.v1")?,
        chunks: vec![SourceChunkBindingV1 {
            chunk_id: SchemaId::new("nextengine.fixture.chunk.start")?,
            region_id: SchemaId::new("nextengine.fixture.region.start")?,
            chunk_asset_id: asset_ids[0],
            required_asset_ids: asset_ids[1..].to_vec(),
        }],
    })
}

fn validate_source(source: &NeutralProjectSourceV1) -> Result<(), ProjectCookError> {
    if source.project_revision == 0 || source.resolver_profile_version == 0 {
        return Err(ProjectCookError::InvalidRevision);
    }
    ensure_unique(source.records.iter().map(|record| record.asset_id))?;
    ensure_unique(source.records.iter().map(|record| record.record_id))?;
    ensure_unique(source.root_asset_ids.iter().copied())?;
    ensure_unique(source.chunks.iter().map(|chunk| chunk.chunk_id.as_str()))?;
    let assets: BTreeSet<_> = source
        .records
        .iter()
        .map(|record| record.asset_id)
        .collect();
    let records: BTreeSet<_> = source
        .records
        .iter()
        .map(|record| record.record_id)
        .collect();
    for record in &source.records {
        NeutralRecordV1::from_canonical_bytes(
            &record.canonical_bytes()?,
            next_contracts::CanonicalDecodeLimits::default(),
        )?;
        if record
            .asset_dependencies
            .iter()
            .any(|dependency| !assets.contains(dependency))
            || record
                .persistent_references
                .iter()
                .any(|reference| !records.contains(reference))
        {
            return Err(ProjectCookError::MissingReference);
        }
    }
    if source
        .root_asset_ids
        .iter()
        .any(|asset_id| !assets.contains(asset_id))
    {
        return Err(ProjectCookError::MissingReference);
    }
    for chunk in &source.chunks {
        if !assets.contains(&chunk.chunk_asset_id)
            || chunk
                .required_asset_ids
                .iter()
                .any(|asset_id| !assets.contains(asset_id))
        {
            return Err(ProjectCookError::MissingReference);
        }
    }
    Ok(())
}

fn schema_ref(
    schema_id: &str,
    role: SchemaRoleV1,
    encoding: SchemaEncodingV1,
) -> Result<SchemaRefV1, ProjectCookError> {
    Ok(SchemaRefV1 {
        schema_id: SchemaId::new(schema_id)?,
        schema_version: 1,
        descriptor_sha256: domain_hash("nextengine.schema-descriptor.v1", schema_id.as_bytes()),
        role,
        encoding,
    })
}

fn ensure_unique<T: Ord>(values: impl IntoIterator<Item = T>) -> Result<(), ProjectCookError> {
    let mut values: Vec<_> = values.into_iter().collect();
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(ProjectCookError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ProjectCookError {
    Contract(ProjectContractError),
    Neutral(NeutralRecordError),
    Resolution(ProjectResolutionError),
    Store(next_assets::ContentStoreError),
    Identifier(next_contracts::IdentifierError),
    MissingReference,
    DuplicateIdentity,
    InvalidRevision,
    HashCollision,
}

impl ProjectCookError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Contract(_) | Self::Neutral(_) => "CONTENT_SCHEMA_INVALID",
            Self::Resolution(_) => "PROJECT_RESOLUTION_FAILED",
            Self::Store(_) => "CONTENT_PUBLICATION_FAILED",
            Self::Identifier(_) => "CONTENT_IDENTIFIER_INVALID",
            Self::MissingReference => "CONTENT_REFERENCE_MISSING",
            Self::DuplicateIdentity => "CONTENT_ID_DUPLICATE",
            Self::InvalidRevision => "CONTENT_REVISION_INVALID",
            Self::HashCollision => "CONTENT_HASH_COLLISION",
        }
    }
}

impl Display for ProjectCookError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "content contract invalid: {error}"),
            Self::Neutral(error) => write!(formatter, "neutral record invalid: {error}"),
            Self::Resolution(error) => write!(formatter, "project resolution failed: {error}"),
            Self::Store(error) => write!(formatter, "content publication failed: {error}"),
            Self::Identifier(error) => write!(formatter, "content identifier invalid: {error}"),
            Self::MissingReference => formatter.write_str("content reference is missing"),
            Self::DuplicateIdentity => formatter.write_str("content identity is duplicated"),
            Self::InvalidRevision => formatter.write_str("content revision must be positive"),
            Self::HashCollision => formatter.write_str("content hash collision"),
        }
    }
}

impl Error for ProjectCookError {}

impl From<ProjectContractError> for ProjectCookError {
    fn from(error: ProjectContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<NeutralRecordError> for ProjectCookError {
    fn from(error: NeutralRecordError) -> Self {
        Self::Neutral(error)
    }
}

impl From<ProjectResolutionError> for ProjectCookError {
    fn from(error: ProjectResolutionError) -> Self {
        Self::Resolution(error)
    }
}

impl From<next_assets::ContentStoreError> for ProjectCookError {
    fn from(error: next_assets::ContentStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<next_contracts::IdentifierError> for ProjectCookError {
    fn from(error: next_contracts::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}
