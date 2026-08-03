use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_assets::{ContentPublicationV1, PublicationFileV1};
use next_contracts::content::{NeutralRecordError, NeutralRecordKindV1, NeutralRecordV1};
use next_contracts::ids::{
    AssetId, CapabilityId, ContentHash, MechanicPackageId, ProjectId, SchemaId,
};
use next_contracts::localization::{TEXT_CATALOG_SCHEMA_ID, TextCatalogErrorV1, TextCatalogV1};
use next_contracts::mechanics::{
    AbilityDefinitionV1, AbilityTargetKindV1, CooldownSpecV1, DialogueDefinitionV1,
    InteractionDefinitionV1, LockedMechanicPackageV1, MECHANICS_EFFECT_PROPOSE_CAPABILITY_ID,
    MechanicAffordanceV1, MechanicPackageManifestV1, MechanicsContractError, MechanicsLockV1,
    PHYSICS_QUERY_CONTACT_CAPABILITY_ID, QuestDefinitionV1, RelationshipDefinitionV1,
    RpgDefinitionRegistryV1, StateTransitionV1, ability_definition_hash,
    interaction_definition_hash,
};
use next_contracts::platform::PresentationTargetKindV1;
use next_contracts::project::{
    AssetRevisionRefV1, ContentAssetEntryV1, ContentDependencyEdgeV1, ContentManifestBodyV1,
    ContentManifestV1, ContentProvenanceV1, ContentSemanticClassV1, ProjectCatalogRecordV1,
    ProjectCatalogSnapshotV1, ProjectCompositionLockV2, ProjectContractError,
    ProjectDependencyKindV1, ProjectManifestV1, ProjectRequirementV1, SchemaDescriptorV1,
    SchemaEncodingV1, SchemaRefV1, SchemaRegistryManifestBodyV1, SchemaRegistryManifestV1,
    SchemaRoleV1, SemanticVersionV1, WorldChunkBindingV1, WorldPartitionManifestBodyV1,
    WorldPartitionManifestV1, canonical_empty_manifest_hash, domain_hash,
};
use next_contracts::render_content::{
    NeutralRenderRecordV1, RenderContentCatalogV1, RenderContentContractError,
};
use next_contracts::rpg::RPG_COMMAND_CAPABILITY_ID;
use next_contracts::session::{RecoveryPolicyV1, ShutdownPolicyV1};

use crate::{ProjectResolutionError, resolve_project_records_v1};

pub const PROJECT_MANIFEST_PATH: &str = "manifests/project.json";
pub const PROJECT_CATALOG_PATH: &str = "manifests/catalog.json";
pub const PROJECT_COMPOSITION_LOCK_PATH: &str = "manifests/composition-lock.json";
pub const SCHEMA_REGISTRY_PATH: &str = "manifests/schema-registry.json";
pub const CONTENT_MANIFEST_PATH: &str = "manifests/content.json";
pub const WORLD_PARTITION_PATH: &str = "manifests/world-partition.json";
pub const CONTENT_BLOB_DIRECTORY: &str = "blobs";
pub const RENDER_CONTENT_CATALOG_PATH: &str = "render-content/catalog.bin";
pub const RENDER_CONTENT_MESH_DIRECTORY: &str = "render-content/meshes";
pub const CORE_INTERACTION_PACKAGE_ID: &str = "org.nextengine.core.interaction";
pub const CORE_COMBAT_PACKAGE_ID: &str = "org.nextengine.core.combat";

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
    pub render_records: Vec<NeutralRenderRecordV1>,
    pub text_catalogs: Vec<TextCatalogV1>,
    pub root_asset_ids: Vec<AssetId>,
    pub provenance: ContentProvenanceV1,
    pub license_manifest_sha256: ContentHash,
    pub partition_id: SchemaId,
    pub coordinate_profile_id: SchemaId,
    pub chunks: Vec<SourceChunkBindingV1>,
    pub recovery_policy: RecoveryPolicyV1,
    pub shutdown_policy: ShutdownPolicyV1,
    pub allowed_presentation_targets: Vec<PresentationTargetKindV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CookedProjectV1 {
    pub project_manifest: ProjectManifestV1,
    pub catalog_snapshot: ProjectCatalogSnapshotV1,
    pub composition_lock: ProjectCompositionLockV2,
    pub schema_registry: SchemaRegistryManifestV1,
    pub content_manifest: ContentManifestV1,
    pub world_partition: WorldPartitionManifestV1,
    pub rpg_definitions: RpgDefinitionRegistryV1,
    pub render_content_catalog: RenderContentCatalogV1,
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
            PublicationFileV1::new(
                RENDER_CONTENT_CATALOG_PATH,
                self.render_content_catalog.canonical_bytes()?,
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
            self.composition_lock.composition_lock_sha256,
            files,
        )?)
    }
}

pub fn cook_project_v1(
    mut source: NeutralProjectSourceV1,
) -> Result<CookedProjectV1, ProjectCookError> {
    source.records.sort_by_key(|record| record.asset_id);
    source
        .render_records
        .sort_by_key(NeutralRenderRecordV1::asset_id);
    source
        .text_catalogs
        .sort_by_key(|catalog| catalog.catalog_asset_id);
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
        let bytes = catalog.canonical_bytes()?;
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
    let rpg_definitions = compile_rpg_definitions_v1(&source.records)?;

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
    let composition_lock = ProjectCompositionLockV2::new(ProjectCompositionLockV2 {
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
        mechanics_lock_sha256: rpg_definitions.mechanics_lock.mechanics_lock_sha256,
        runtime_determinism_profile_sha256: domain_hash(
            "nextengine.runtime-determinism-profile.v1",
            b"fixed-stage-order+adr-022-ingress",
        ),
        launch_profiles_sha256: domain_hash(
            "nextengine.launch-profiles.v1",
            b"game:interactive|none;headless:none;tools:none|interactive;capture-worker:displayless-offscreen",
        ),
        recovery_policy_sha256: source.recovery_policy.canonical_hash,
        shutdown_policy_sha256: source.shutdown_policy.canonical_hash,
        recovery_permit_required_save: source.recovery_policy.permit_required_save_recovery,
        recovery_preserve_prior_history: source.recovery_policy.preserve_prior_history,
        shutdown_maximum_attempts: source.shutdown_policy.maximum_attempts,
        shutdown_failure_disposition: source.shutdown_policy.failure_disposition,
        platform_capability_profile_sha256: domain_hash(
            "nextengine.platform-capability-profile.v1",
            b"normalized-engine-owned-capabilities",
        ),
        platform_timebase_profile_sha256: domain_hash(
            "nextengine.platform-timebase-profile.v1",
            b"diagnostic-only-monotonic-v1",
        ),
        allowed_presentation_targets: source.allowed_presentation_targets,
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
        rpg_definitions,
        render_content_catalog,
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

pub(crate) fn compile_rpg_definitions_v1(
    records: &[NeutralRecordV1],
) -> Result<RpgDefinitionRegistryV1, ProjectCookError> {
    let mut by_kind = BTreeMap::new();
    for record in records {
        if !matches!(
            record.kind,
            NeutralRecordKindV1::DialogueDefinition
                | NeutralRecordKindV1::QuestDefinition
                | NeutralRecordKindV1::RelationshipDefinition
                | NeutralRecordKindV1::InteractionDefinition
                | NeutralRecordKindV1::AbilityDefinition
                | NeutralRecordKindV1::ItemDefinition
        ) {
            continue;
        }
        if by_kind.insert(record.kind, record).is_some() {
            return Err(ProjectCookError::DuplicateIdentity);
        }
    }
    let dialogue_record = by_kind
        .get(&NeutralRecordKindV1::DialogueDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let quest_record = by_kind
        .get(&NeutralRecordKindV1::QuestDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let relationship_record = by_kind
        .get(&NeutralRecordKindV1::RelationshipDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let interaction_record = by_kind
        .get(&NeutralRecordKindV1::InteractionDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let ability_record = by_kind
        .get(&NeutralRecordKindV1::AbilityDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let item_record = by_kind
        .get(&NeutralRecordKindV1::ItemDefinition)
        .ok_or(ProjectCookError::MissingReference)?;
    let dialogue_revision = asset_revision(dialogue_record)?;
    let quest_revision = asset_revision(quest_record)?;
    let relationship_revision = asset_revision(relationship_record)?;
    let interaction_revision = asset_revision(interaction_record)?;
    let ability_revision = asset_revision(ability_record)?;
    let dialogue_transition_id = property_id(
        interaction_record,
        "nextengine.interaction.dialogue-transition",
    )?;
    let quest_transition_id = property_id(
        interaction_record,
        "nextengine.interaction.quest-transition",
    )?;
    let dialogue = DialogueDefinitionV1 {
        asset_revision: dialogue_revision,
        entry_node_id: property_id(dialogue_record, "nextengine.dialogue.entry-node")?,
        transitions: vec![StateTransitionV1 {
            transition_id: dialogue_transition_id.clone(),
            source_state_id: property_id(dialogue_record, "nextengine.dialogue.entry-node")?,
            target_state_id: property_id(dialogue_record, "nextengine.dialogue.accepted-node")?,
        }],
    };
    let quest = QuestDefinitionV1 {
        asset_revision: quest_revision,
        entry_state_id: property_id(quest_record, "nextengine.quest.entry-state")?,
        transitions: vec![StateTransitionV1 {
            transition_id: quest_transition_id.clone(),
            source_state_id: property_id(quest_record, "nextengine.quest.entry-state")?,
            target_state_id: property_id(quest_record, "nextengine.quest.active-state")?,
        }],
    };
    let relationship = RelationshipDefinitionV1 {
        asset_revision: relationship_revision,
        dimension_id: property_id(relationship_record, "nextengine.relationship.dimension")?,
        minimum_value: -100,
        maximum_value: 100,
    };
    let dependency_revisions: BTreeMap<_, _> = interaction_record
        .asset_dependencies
        .iter()
        .map(|asset_id| {
            let record = records
                .iter()
                .find(|record| record.asset_id == *asset_id)
                .ok_or(ProjectCookError::MissingReference)?;
            Ok((record.kind, asset_revision(record)?))
        })
        .collect::<Result<_, ProjectCookError>>()?;
    let interaction = InteractionDefinitionV1 {
        asset_revision: interaction_revision,
        interaction_id: property_id(interaction_record, "nextengine.interaction.definition-id")?,
        dialogue_definition: *dependency_revisions
            .get(&NeutralRecordKindV1::DialogueDefinition)
            .ok_or(ProjectCookError::MissingReference)?,
        dialogue_transition_id,
        quest_definition: *dependency_revisions
            .get(&NeutralRecordKindV1::QuestDefinition)
            .ok_or(ProjectCookError::MissingReference)?,
        quest_transition_id,
        relationship_definition: *dependency_revisions
            .get(&NeutralRecordKindV1::RelationshipDefinition)
            .ok_or(ProjectCookError::MissingReference)?,
        relationship_source_value: 0,
        relationship_delta: property_i32(
            interaction_record,
            "nextengine.interaction.relationship-delta",
        )?,
    };
    let interaction_hash = interaction_definition_hash(&interaction);
    let rpg_capability = next_contracts::ids::CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
        .expect("engine-owned RPG capability is valid");
    let interaction_package = MechanicPackageManifestV1::new(
        MechanicPackageId::new(CORE_INTERACTION_PACKAGE_ID)?,
        1,
        vec![rpg_capability.clone()],
        vec![interaction_hash],
        Vec::new(),
    )?;
    let effect_capability = CapabilityId::new(MECHANICS_EFFECT_PROPOSE_CAPABILITY_ID)
        .expect("engine-owned effect capability is valid");
    let contact_capability = CapabilityId::new(PHYSICS_QUERY_CONTACT_CAPABILITY_ID)
        .expect("engine-owned contact capability is valid");
    let ability = AbilityDefinitionV1 {
        asset_revision: ability_revision,
        package_id: MechanicPackageId::new(CORE_COMBAT_PACKAGE_ID)?,
        ability_id: property_id(ability_record, "nextengine.ability.definition-id")?,
        required_item_definition: asset_revision(item_record)?,
        required_equipment_slot_id: property_id(
            ability_record,
            "nextengine.ability.equipment-slot",
        )?,
        target_kind: AbilityTargetKindV1::ContactCharacter,
        resource_id: property_id(ability_record, "nextengine.ability.resource")?,
        resource_delta: property_i32(ability_record, "nextengine.ability.resource-delta")?,
        cooldown: CooldownSpecV1 {
            duration_ticks: property_u32(ability_record, "nextengine.ability.cooldown-ticks")?,
            group_id: property_id(ability_record, "nextengine.ability.cooldown-group")?,
        },
        required_capabilities: vec![effect_capability.clone(), contact_capability.clone()],
        affordance: MechanicAffordanceV1 {
            semantic_action_id: property_id(ability_record, "nextengine.ability.semantic-action")?,
            planner_visible: true,
            requires_equipped_item: true,
            requires_contact: true,
            expected_resource_delta_minimum: -25,
            expected_resource_delta_maximum: -25,
            failure_modes: vec![
                SchemaId::new("nextengine.mechanics.failure.contact-required")?,
                SchemaId::new("nextengine.mechanics.failure.cooldown-active")?,
                SchemaId::new("nextengine.mechanics.failure.equipment-required")?,
            ],
        },
    };
    let ability_hash = ability_definition_hash(&ability);
    let combat_package = MechanicPackageManifestV1::new(
        MechanicPackageId::new(CORE_COMBAT_PACKAGE_ID)?,
        1,
        vec![effect_capability.clone(), contact_capability.clone()],
        Vec::new(),
        vec![ability_hash],
    )?;
    let mechanics_lock = MechanicsLockV1::new(vec![
        LockedMechanicPackageV1 {
            package_id: interaction_package.package_id.clone(),
            package_manifest_sha256: interaction_package.package_manifest_sha256,
            granted_capabilities: vec![rpg_capability],
        },
        LockedMechanicPackageV1 {
            package_id: combat_package.package_id.clone(),
            package_manifest_sha256: combat_package.package_manifest_sha256,
            granted_capabilities: vec![effect_capability, contact_capability],
        },
    ])?;
    Ok(RpgDefinitionRegistryV1::new(
        vec![dialogue],
        vec![quest],
        vec![relationship],
        vec![interaction],
        vec![ability],
        vec![interaction_package, combat_package],
        mechanics_lock,
    )?)
}

fn asset_revision(record: &NeutralRecordV1) -> Result<AssetRevisionRefV1, ProjectCookError> {
    Ok(AssetRevisionRefV1 {
        asset_id: record.asset_id,
        record_sha256: record.record_sha256()?,
    })
}

fn property_id(record: &NeutralRecordV1, property_id: &str) -> Result<SchemaId, ProjectCookError> {
    record
        .properties
        .iter()
        .find(|property| property.property_id.as_str() == property_id)
        .map(|property| property.value_id.clone())
        .ok_or(ProjectCookError::MissingReference)
}

fn property_i32(record: &NeutralRecordV1, property_key: &str) -> Result<i32, ProjectCookError> {
    let value = property_id(record, property_key)?;
    value
        .as_str()
        .strip_prefix("nextengine.value.i32.")
        .ok_or(ProjectCookError::InvalidValue)?
        .parse()
        .map_err(|_| ProjectCookError::InvalidValue)
}

fn property_u32(record: &NeutralRecordV1, property_key: &str) -> Result<u32, ProjectCookError> {
    let value = property_id(record, property_key)?;
    value
        .as_str()
        .strip_prefix("nextengine.value.u32.")
        .ok_or(ProjectCookError::InvalidValue)?
        .parse()
        .map_err(|_| ProjectCookError::InvalidValue)
}

fn validate_source(source: &NeutralProjectSourceV1) -> Result<(), ProjectCookError> {
    if source.project_revision == 0 || source.resolver_profile_version == 0 {
        return Err(ProjectCookError::InvalidRevision);
    }
    source
        .recovery_policy
        .validate()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    source
        .shutdown_policy
        .validate()
        .map_err(|_| ProjectCookError::InvalidValue)?;
    if source.allowed_presentation_targets.is_empty() {
        return Err(ProjectCookError::InvalidValue);
    }
    ensure_unique(
        source
            .records
            .iter()
            .map(|record| record.asset_id)
            .chain(
                source
                    .render_records
                    .iter()
                    .map(NeutralRenderRecordV1::asset_id),
            )
            .chain(
                source
                    .text_catalogs
                    .iter()
                    .map(|catalog| catalog.catalog_asset_id),
            ),
    )?;
    ensure_unique(source.records.iter().map(|record| record.record_id))?;
    ensure_unique(source.root_asset_ids.iter().copied())?;
    ensure_unique(source.chunks.iter().map(|chunk| chunk.chunk_id.as_str()))?;
    let assets: BTreeSet<_> = source
        .records
        .iter()
        .map(|record| record.asset_id)
        .chain(
            source
                .render_records
                .iter()
                .map(NeutralRenderRecordV1::asset_id),
        )
        .collect();
    let mut revisions = BTreeMap::new();
    for record in &source.records {
        revisions.insert(record.asset_id, asset_revision(record)?);
    }
    for record in &source.render_records {
        revisions.insert(record.asset_id(), record.asset_revision()?);
    }
    let records: BTreeSet<_> = source
        .records
        .iter()
        .map(|record| record.record_id)
        .collect();
    for record in &source.records {
        NeutralRecordV1::from_canonical_bytes(
            &record.canonical_bytes()?,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
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
    for record in &source.render_records {
        let bytes = record.canonical_bytes()?;
        if NeutralRenderRecordV1::from_canonical_bytes(
            &bytes,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )? != record.clone()
        {
            return Err(ProjectCookError::Render(
                RenderContentContractError::NonCanonical,
            ));
        }
        for dependency in record.dependencies() {
            if revisions.get(&dependency.asset_id) != Some(&dependency) {
                return Err(ProjectCookError::MissingReference);
            }
        }
    }
    compile_render_content_catalog_v1(&source.render_records)?;
    for catalog in &source.text_catalogs {
        TextCatalogV1::from_canonical_bytes(
            &catalog.canonical_bytes()?,
            next_contracts::canonical::CanonicalDecodeLimits::default(),
        )?;
    }
    validate_text_catalog_closure(&source.text_catalogs)?;
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

fn validate_text_catalog_closure(catalogs: &[TextCatalogV1]) -> Result<(), ProjectCookError> {
    if catalogs.is_empty() {
        return Ok(());
    }
    let mut locales = BTreeSet::new();
    let mut root_count = 0_usize;
    for catalog in catalogs {
        if !locales.insert(catalog.locale.as_str()) {
            return Err(ProjectCookError::DuplicateIdentity);
        }
        if catalog.fallback_locale_or_none.is_none() {
            root_count += 1;
        }
    }
    if root_count != 1 {
        return Err(ProjectCookError::LocalizationClosureInvalid);
    }
    for catalog in catalogs {
        let mut visited = BTreeSet::new();
        let mut current = catalog;
        loop {
            if !visited.insert(current.locale.as_str()) {
                return Err(ProjectCookError::LocalizationClosureInvalid);
            }
            let Some(fallback) = &current.fallback_locale_or_none else {
                break;
            };
            current = catalogs
                .iter()
                .find(|candidate| candidate.locale.as_str() == fallback.as_str())
                .ok_or(ProjectCookError::MissingReference)?;
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
    Render(RenderContentContractError),
    Localization(TextCatalogErrorV1),
    Resolution(ProjectResolutionError),
    Store(next_assets::ContentStoreError),
    Identifier(next_contracts::ids::IdentifierError),
    MissingReference,
    DuplicateIdentity,
    InvalidRevision,
    HashCollision,
    LocalizationClosureInvalid,
    Mechanics(MechanicsContractError),
    InvalidValue,
}

impl ProjectCookError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Contract(_) | Self::Neutral(_) => "CONTENT_SCHEMA_INVALID",
            Self::Render(error) => error.diagnostic_code(),
            Self::Localization(_) => "CONTENT_SCHEMA_INVALID",
            Self::Resolution(_) => "PROJECT_RESOLUTION_FAILED",
            Self::Store(_) => "CONTENT_PUBLICATION_FAILED",
            Self::Identifier(_) => "CONTENT_IDENTIFIER_INVALID",
            Self::MissingReference => "CONTENT_REFERENCE_MISSING",
            Self::DuplicateIdentity => "CONTENT_ID_DUPLICATE",
            Self::InvalidRevision => "CONTENT_REVISION_INVALID",
            Self::HashCollision => "CONTENT_HASH_COLLISION",
            Self::LocalizationClosureInvalid => "LOCALIZATION_CLOSURE_INVALID",
            Self::Mechanics(_) => "MECHANICS_MANIFEST_INVALID",
            Self::InvalidValue => "CONTENT_VALUE_INVALID",
        }
    }
}

impl Display for ProjectCookError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "content contract invalid: {error}"),
            Self::Neutral(error) => write!(formatter, "neutral record invalid: {error}"),
            Self::Render(error) => write!(formatter, "render content invalid: {error}"),
            Self::Localization(error) => write!(formatter, "text catalog invalid: {error}"),
            Self::Resolution(error) => write!(formatter, "project resolution failed: {error}"),
            Self::Store(error) => write!(formatter, "content publication failed: {error}"),
            Self::Identifier(error) => write!(formatter, "content identifier invalid: {error}"),
            Self::MissingReference => formatter.write_str("content reference is missing"),
            Self::DuplicateIdentity => formatter.write_str("content identity is duplicated"),
            Self::InvalidRevision => formatter.write_str("content revision must be positive"),
            Self::HashCollision => formatter.write_str("content hash collision"),
            Self::LocalizationClosureInvalid => {
                formatter.write_str("localization fallback closure is invalid")
            }
            Self::Mechanics(error) => write!(formatter, "mechanics contract invalid: {error}"),
            Self::InvalidValue => formatter.write_str("content property value is invalid"),
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

impl From<RenderContentContractError> for ProjectCookError {
    fn from(error: RenderContentContractError) -> Self {
        Self::Render(error)
    }
}

impl From<TextCatalogErrorV1> for ProjectCookError {
    fn from(error: TextCatalogErrorV1) -> Self {
        Self::Localization(error)
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

impl From<next_contracts::ids::IdentifierError> for ProjectCookError {
    fn from(error: next_contracts::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<MechanicsContractError> for ProjectCookError {
    fn from(error: MechanicsContractError) -> Self {
        Self::Mechanics(error)
    }
}
