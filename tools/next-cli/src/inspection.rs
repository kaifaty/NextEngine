use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use next_contracts::content::{NeutralRecordKindV1, NeutralRecordV1};
use next_contracts::platform::PresentationTargetKindV1;
use next_contracts::project::{
    ActivatedProjectV8, AssetRevisionRefV1, ContentSemanticClassV1, SchemaEncodingV1, SchemaRoleV1,
};
use next_project::CookedProjectV7;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CreatorProjectInput {
    Project(PathBuf),
    Package(PathBuf),
}

pub(crate) fn exclusive_project_input(
    project: Option<PathBuf>,
    package: Option<PathBuf>,
) -> Option<CreatorProjectInput> {
    match (project, package) {
        (Some(project), None) => Some(CreatorProjectInput::Project(project)),
        (None, Some(package)) => Some(CreatorProjectInput::Package(package)),
        _ => None,
    }
}

use crate::{
    CreatorCommandFailureReportV1, CreatorProjectIdentityV1, project_identity_from_activated,
    project_identity_from_cooked,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatorInspectCommandReportV1 {
    Pass(Box<CreatorInspectCommandPassReportV1>),
    Fail(CreatorCommandFailureReportV1),
}

impl CreatorInspectCommandReportV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorInspectCommandPassReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: CreatorInspectDetailsV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorInspectDetailsV1 {
    pub source: String,
    pub project: CreatorProjectIdentityV1,
    pub projection: CreatorProjectProjectionV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatorDiffCommandReportV1 {
    Pass(Box<CreatorDiffCommandPassReportV1>),
    Fail(CreatorCommandFailureReportV1),
}

impl CreatorDiffCommandReportV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorDiffCommandPassReportV1 {
    pub schema_version: u32,
    pub status: String,
    pub command: String,
    pub details: CreatorProjectDiffV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorProjectProjectionV1 {
    pub summary: CreatorProjectProjectionSummaryV1,
    pub composition: CreatorCompositionProjectionV1,
    pub schemas: Vec<CreatorSchemaProjectionV1>,
    pub root_assets: Vec<CreatorAssetRevisionProjectionV1>,
    pub assets: Vec<CreatorAssetProjectionV1>,
    pub dependencies: Vec<CreatorDependencyProjectionV1>,
    pub world: CreatorWorldProjectionV1,
    pub mechanics: CreatorMechanicsProjectionV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorProjectProjectionSummaryV1 {
    pub schema_count: u32,
    pub root_asset_count: u32,
    pub asset_count: u32,
    pub dependency_count: u32,
    pub neutral_record_count: u32,
    pub world_chunk_count: u32,
    pub mechanic_package_count: u32,
    pub granted_capability_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorCompositionProjectionV1 {
    pub content_domain_closure_sha256: String,
    pub render_content_catalog_sha256: String,
    pub runtime_determinism_profile_sha256: String,
    pub launch_profiles_sha256: String,
    pub platform_capability_profile_sha256: String,
    pub platform_timebase_profile_sha256: String,
    pub allowed_presentation_targets: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorSchemaProjectionV1 {
    pub schema_id: String,
    pub schema_version: u32,
    pub descriptor_sha256: String,
    pub role: String,
    pub encoding: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorAssetRevisionProjectionV1 {
    pub asset_id: String,
    pub record_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorAssetProjectionV1 {
    pub asset_id: String,
    pub record_sha256: String,
    pub neutral_record_blob_sha256: String,
    pub schema_id: String,
    pub schema_version: u32,
    pub descriptor_sha256: String,
    pub schema_role: String,
    pub schema_encoding: String,
    pub semantic_class: String,
    pub owning_bundle_id: String,
    pub provenance_sha256: String,
    pub license_manifest_sha256: String,
    pub is_root: bool,
    pub neutral_kind_or_none: Option<String>,
    pub persistent_record_id_or_none: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorDependencyProjectionV1 {
    pub source_asset_id: String,
    pub target_asset_id: String,
    pub dependency_kind: String,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorWorldProjectionV1 {
    pub partition_id: String,
    pub coordinate_profile_id: String,
    pub topology_revision: u64,
    pub root_region_ids: Vec<String>,
    pub initial_placement_catalog_sha256: String,
    pub residency_policy_sha256: String,
    pub chunks: Vec<CreatorWorldChunkProjectionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorWorldChunkProjectionV1 {
    pub chunk_id: String,
    pub region_id: String,
    pub chunk_asset_id: String,
    pub chunk_record_sha256: String,
    pub required_asset_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorMechanicsProjectionV1 {
    pub registry_sha256: String,
    pub packages: Vec<CreatorMechanicPackageProjectionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorMechanicPackageProjectionV1 {
    pub package_id: String,
    pub package_manifest_sha256: String,
    pub granted_capabilities: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorProjectDiffV1 {
    pub base: CreatorDiffOperandV1,
    pub candidate: CreatorDiffOperandV1,
    pub different: bool,
    pub summary: CreatorProjectDiffSummaryV1,
    pub changes: CreatorProjectDiffChangesV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorDiffOperandV1 {
    pub source: String,
    pub project: CreatorProjectIdentityV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorProjectDiffSummaryV1 {
    pub project_field_changes: u32,
    pub root_changes: u32,
    pub presentation_target_changes: u32,
    pub schema_changes: u32,
    pub root_asset_changes: u32,
    pub asset_changes: u32,
    pub dependency_changes: u32,
    pub world_field_changes: u32,
    pub world_region_changes: u32,
    pub world_chunk_changes: u32,
    pub mechanic_package_changes: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorProjectDiffChangesV1 {
    pub project_fields: Vec<CreatorScalarChangeV1>,
    pub roots: Vec<CreatorHashChangeV1>,
    pub presentation_targets: CreatorStringSetDeltaV1,
    pub schemas: Vec<CreatorCollectionChangeV1<CreatorSchemaProjectionV1>>,
    pub root_assets: Vec<CreatorCollectionChangeV1<CreatorAssetRevisionProjectionV1>>,
    pub assets: Vec<CreatorCollectionChangeV1<CreatorAssetProjectionV1>>,
    pub dependencies: Vec<CreatorCollectionChangeV1<CreatorDependencyProjectionV1>>,
    pub world_fields: Vec<CreatorScalarChangeV1>,
    pub world_regions: CreatorStringSetDeltaV1,
    pub world_chunks: Vec<CreatorCollectionChangeV1<CreatorWorldChunkProjectionV1>>,
    pub mechanic_packages: Vec<CreatorCollectionChangeV1<CreatorMechanicPackageProjectionV1>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorScalarChangeV1 {
    pub field: String,
    pub base: String,
    pub candidate: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorHashChangeV1 {
    pub root: String,
    pub base_sha256: String,
    pub candidate_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorStringSetDeltaV1 {
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatorCollectionChangeV1<T> {
    pub change: String,
    pub key: String,
    pub base: Option<T>,
    pub candidate: Option<T>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct InspectionProjectionError;

pub(super) fn inspect_cooked(
    cooked: &CookedProjectV7,
    neutral_records: &[NeutralRecordV1],
) -> Result<CreatorInspectDetailsV1, InspectionProjectionError> {
    Ok(CreatorInspectDetailsV1 {
        source: "authoring".to_owned(),
        project: project_identity_from_cooked(cooked),
        projection: project_projection(
            &cooked.project_lock,
            &cooked.schema_registry,
            &cooked.content_manifest,
            &cooked.world_partition,
            &cooked.rpg_definitions,
            &cooked.render_content_catalog,
            neutral_records,
        )?,
    })
}

pub(super) fn inspect_activated(
    activated: &ActivatedProjectV8,
) -> Result<CreatorInspectDetailsV1, InspectionProjectionError> {
    Ok(CreatorInspectDetailsV1 {
        source: "package".to_owned(),
        project: project_identity_from_activated(activated),
        projection: project_projection(
            &activated.project_lock,
            &activated.schema_registry,
            &activated.content_manifest,
            &activated.world_partition,
            &activated.rpg_definitions,
            &activated.render_content_catalog,
            &activated.neutral_records,
        )?,
    })
}

fn project_projection(
    lock: &next_contracts::project::ProjectLockV3,
    schemas: &next_contracts::project::SchemaRegistryManifestV2,
    content: &next_contracts::project::ContentManifestV1,
    world: &next_contracts::project::WorldPartitionManifestV1,
    mechanics: &next_contracts::mechanics::RpgDefinitionRegistryV2,
    render: &next_contracts::render_content::RenderContentCatalogV1,
    neutral_records: &[NeutralRecordV1],
) -> Result<CreatorProjectProjectionV1, InspectionProjectionError> {
    let neutral_by_asset = neutral_records
        .iter()
        .map(|record| (record.asset_id, record))
        .collect::<BTreeMap<_, _>>();
    if neutral_by_asset.len() != neutral_records.len() {
        return Err(InspectionProjectionError);
    }

    let mut schema_projection = schemas
        .body
        .current_schema_refs
        .iter()
        .map(|schema| CreatorSchemaProjectionV1 {
            schema_id: schema.schema_id.as_str().to_owned(),
            schema_version: schema.schema_version,
            descriptor_sha256: schema.descriptor_sha256.to_hex(),
            role: schema_role(schema.role).to_owned(),
            encoding: schema_encoding(schema.encoding).to_owned(),
        })
        .collect::<Vec<_>>();
    schema_projection.sort_by(|left, right| left.schema_id.cmp(&right.schema_id));

    let root_assets = content
        .body
        .root_assets
        .iter()
        .map(asset_revision_projection)
        .collect::<Vec<_>>();
    let root_asset_set = content
        .body
        .root_assets
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let assets = content
        .body
        .asset_entries
        .iter()
        .map(|entry| {
            let neutral = neutral_by_asset
                .get(&entry.asset_revision.asset_id)
                .copied();
            CreatorAssetProjectionV1 {
                asset_id: entry.asset_revision.asset_id.to_hex(),
                record_sha256: entry.asset_revision.record_sha256.to_hex(),
                neutral_record_blob_sha256: entry.neutral_record_blob_sha256.to_hex(),
                schema_id: entry.schema_ref.schema_id.as_str().to_owned(),
                schema_version: entry.schema_ref.schema_version,
                descriptor_sha256: entry.schema_ref.descriptor_sha256.to_hex(),
                schema_role: schema_role(entry.schema_ref.role).to_owned(),
                schema_encoding: schema_encoding(entry.schema_ref.encoding).to_owned(),
                semantic_class: semantic_class(entry.semantic_class).to_owned(),
                owning_bundle_id: entry.owning_bundle_id.as_str().to_owned(),
                provenance_sha256: entry.provenance_sha256.to_hex(),
                license_manifest_sha256: entry.license_manifest_sha256.to_hex(),
                is_root: root_asset_set.contains(&entry.asset_revision),
                neutral_kind_or_none: neutral.map(|record| neutral_kind(record.kind).to_owned()),
                persistent_record_id_or_none: neutral.map(|record| record.record_id.to_hex()),
            }
        })
        .collect::<Vec<_>>();
    let dependencies = content
        .body
        .dependency_edges
        .iter()
        .map(|edge| CreatorDependencyProjectionV1 {
            source_asset_id: edge.source_asset_id.to_hex(),
            target_asset_id: edge.target_asset_id.to_hex(),
            dependency_kind: edge.dependency_kind.as_str().to_owned(),
            required: edge.required,
        })
        .collect::<Vec<_>>();
    let chunks = world
        .body
        .chunk_bindings
        .iter()
        .map(|chunk| CreatorWorldChunkProjectionV1 {
            chunk_id: chunk.chunk_id.as_str().to_owned(),
            region_id: chunk.region_id.as_str().to_owned(),
            chunk_asset_id: chunk.chunk_asset.asset_id.to_hex(),
            chunk_record_sha256: chunk.chunk_asset.record_sha256.to_hex(),
            required_asset_ids: chunk
                .required_asset_ids
                .iter()
                .map(|asset_id| asset_id.to_hex())
                .collect(),
        })
        .collect::<Vec<_>>();
    let packages = mechanics
        .mechanics_lock
        .packages
        .iter()
        .map(|package| CreatorMechanicPackageProjectionV1 {
            package_id: package.package_id.as_str().to_owned(),
            package_manifest_sha256: package.package_manifest_sha256.to_hex(),
            granted_capabilities: package
                .granted_capabilities
                .iter()
                .map(|capability| capability.as_str().to_owned())
                .collect(),
        })
        .collect::<Vec<_>>();
    let granted_capability_count = mechanics
        .mechanics_lock
        .packages
        .iter()
        .try_fold(0_usize, |total, package| {
            total.checked_add(package.granted_capabilities.len())
        })
        .ok_or(InspectionProjectionError)?;

    Ok(CreatorProjectProjectionV1 {
        summary: CreatorProjectProjectionSummaryV1 {
            schema_count: report_count(schema_projection.len())?,
            root_asset_count: report_count(root_assets.len())?,
            asset_count: report_count(assets.len())?,
            dependency_count: report_count(dependencies.len())?,
            neutral_record_count: report_count(neutral_records.len())?,
            world_chunk_count: report_count(chunks.len())?,
            mechanic_package_count: report_count(packages.len())?,
            granted_capability_count: report_count(granted_capability_count)?,
        },
        composition: CreatorCompositionProjectionV1 {
            content_domain_closure_sha256: content.body.domain_closure_sha256.to_hex(),
            render_content_catalog_sha256: render.catalog_sha256().to_hex(),
            runtime_determinism_profile_sha256: lock.runtime_determinism_profile_sha256.to_hex(),
            launch_profiles_sha256: lock.launch_profiles_sha256.to_hex(),
            platform_capability_profile_sha256: lock.platform_capability_profile_sha256.to_hex(),
            platform_timebase_profile_sha256: lock.platform_timebase_profile_sha256.to_hex(),
            allowed_presentation_targets: lock
                .allowed_presentation_targets
                .iter()
                .map(|target| presentation_target(*target).to_owned())
                .collect(),
        },
        schemas: schema_projection,
        root_assets,
        assets,
        dependencies,
        world: CreatorWorldProjectionV1 {
            partition_id: world.body.partition_id.as_str().to_owned(),
            coordinate_profile_id: world.body.coordinate_profile_id.as_str().to_owned(),
            topology_revision: world.body.topology_revision,
            root_region_ids: world
                .body
                .root_region_ids
                .iter()
                .map(|region| region.as_str().to_owned())
                .collect(),
            initial_placement_catalog_sha256: world.body.initial_placement_catalog_sha256.to_hex(),
            residency_policy_sha256: world.body.residency_policy_sha256.to_hex(),
            chunks,
        },
        mechanics: CreatorMechanicsProjectionV1 {
            registry_sha256: mechanics.registry_sha256.to_hex(),
            packages,
        },
    })
}

pub(super) fn diff_projects(
    base: &CreatorInspectDetailsV1,
    candidate: &CreatorInspectDetailsV1,
) -> Result<CreatorProjectDiffV1, InspectionProjectionError> {
    let mut project_fields = Vec::new();
    push_scalar_change(
        &mut project_fields,
        "project-id",
        &base.project.project_id,
        &candidate.project.project_id,
    );
    push_scalar_change(
        &mut project_fields,
        "project-revision",
        &base.project.project_revision.to_string(),
        &candidate.project.project_revision.to_string(),
    );

    let roots = diff_roots(&root_values(base), &root_values(candidate));
    let presentation_targets = string_set_delta(
        &base.projection.composition.allowed_presentation_targets,
        &candidate
            .projection
            .composition
            .allowed_presentation_targets,
    );
    let schemas = diff_collection(
        &base.projection.schemas,
        &candidate.projection.schemas,
        |schema| schema.schema_id.clone(),
    );
    let root_assets = diff_collection(
        &base.projection.root_assets,
        &candidate.projection.root_assets,
        |asset| asset.asset_id.clone(),
    );
    let assets = diff_collection(
        &base.projection.assets,
        &candidate.projection.assets,
        |asset| asset.asset_id.clone(),
    );
    let dependencies = diff_collection(
        &base.projection.dependencies,
        &candidate.projection.dependencies,
        dependency_key,
    );
    let mut world_fields = Vec::new();
    push_scalar_change(
        &mut world_fields,
        "partition-id",
        &base.projection.world.partition_id,
        &candidate.projection.world.partition_id,
    );
    push_scalar_change(
        &mut world_fields,
        "coordinate-profile-id",
        &base.projection.world.coordinate_profile_id,
        &candidate.projection.world.coordinate_profile_id,
    );
    push_scalar_change(
        &mut world_fields,
        "topology-revision",
        &base.projection.world.topology_revision.to_string(),
        &candidate.projection.world.topology_revision.to_string(),
    );
    let world_regions = string_set_delta(
        &base.projection.world.root_region_ids,
        &candidate.projection.world.root_region_ids,
    );
    let world_chunks = diff_collection(
        &base.projection.world.chunks,
        &candidate.projection.world.chunks,
        |chunk| chunk.chunk_id.clone(),
    );
    let mechanic_packages = diff_collection(
        &base.projection.mechanics.packages,
        &candidate.projection.mechanics.packages,
        |package| package.package_id.clone(),
    );

    let summary = CreatorProjectDiffSummaryV1 {
        project_field_changes: report_count(project_fields.len())?,
        root_changes: report_count(roots.len())?,
        presentation_target_changes: report_count(
            presentation_targets
                .added
                .len()
                .checked_add(presentation_targets.removed.len())
                .ok_or(InspectionProjectionError)?,
        )?,
        schema_changes: report_count(schemas.len())?,
        root_asset_changes: report_count(root_assets.len())?,
        asset_changes: report_count(assets.len())?,
        dependency_changes: report_count(dependencies.len())?,
        world_field_changes: report_count(world_fields.len())?,
        world_region_changes: report_count(
            world_regions
                .added
                .len()
                .checked_add(world_regions.removed.len())
                .ok_or(InspectionProjectionError)?,
        )?,
        world_chunk_changes: report_count(world_chunks.len())?,
        mechanic_package_changes: report_count(mechanic_packages.len())?,
    };
    let different = summary.project_field_changes != 0
        || summary.root_changes != 0
        || summary.presentation_target_changes != 0
        || summary.schema_changes != 0
        || summary.root_asset_changes != 0
        || summary.asset_changes != 0
        || summary.dependency_changes != 0
        || summary.world_field_changes != 0
        || summary.world_region_changes != 0
        || summary.world_chunk_changes != 0
        || summary.mechanic_package_changes != 0;

    Ok(CreatorProjectDiffV1 {
        base: CreatorDiffOperandV1 {
            source: base.source.clone(),
            project: base.project.clone(),
        },
        candidate: CreatorDiffOperandV1 {
            source: candidate.source.clone(),
            project: candidate.project.clone(),
        },
        different,
        summary,
        changes: CreatorProjectDiffChangesV1 {
            project_fields,
            roots,
            presentation_targets,
            schemas,
            root_assets,
            assets,
            dependencies,
            world_fields,
            world_regions,
            world_chunks,
            mechanic_packages,
        },
    })
}

fn root_values(details: &CreatorInspectDetailsV1) -> BTreeMap<&'static str, String> {
    BTreeMap::from([
        ("authoring", details.project.authoring_sha256.clone()),
        ("project-lock", details.project.project_lock_sha256.clone()),
        (
            "schema-registry",
            details.project.schema_registry_sha256.clone(),
        ),
        (
            "content-manifest",
            details.project.content_manifest_sha256.clone(),
        ),
        (
            "content-domain-closure",
            details
                .projection
                .composition
                .content_domain_closure_sha256
                .clone(),
        ),
        (
            "world-partition",
            details.project.world_partition_sha256.clone(),
        ),
        (
            "world-initial-placement-catalog",
            details
                .projection
                .world
                .initial_placement_catalog_sha256
                .clone(),
        ),
        (
            "world-residency-policy",
            details.projection.world.residency_policy_sha256.clone(),
        ),
        (
            "mechanics-lock",
            details.project.mechanics_lock_sha256.clone(),
        ),
        (
            "mechanics-definition-registry",
            details.projection.mechanics.registry_sha256.clone(),
        ),
        (
            "render-content-catalog",
            details
                .projection
                .composition
                .render_content_catalog_sha256
                .clone(),
        ),
        (
            "runtime-determinism-profile",
            details
                .projection
                .composition
                .runtime_determinism_profile_sha256
                .clone(),
        ),
        (
            "launch-profiles",
            details
                .projection
                .composition
                .launch_profiles_sha256
                .clone(),
        ),
        (
            "platform-capability-profile",
            details
                .projection
                .composition
                .platform_capability_profile_sha256
                .clone(),
        ),
        (
            "platform-timebase-profile",
            details
                .projection
                .composition
                .platform_timebase_profile_sha256
                .clone(),
        ),
    ])
}

fn diff_roots(
    base: &BTreeMap<&'static str, String>,
    candidate: &BTreeMap<&'static str, String>,
) -> Vec<CreatorHashChangeV1> {
    base.iter()
        .filter_map(|(root, base_sha256)| {
            let candidate_sha256 = candidate.get(root)?;
            (base_sha256 != candidate_sha256).then(|| CreatorHashChangeV1 {
                root: (*root).to_owned(),
                base_sha256: base_sha256.clone(),
                candidate_sha256: candidate_sha256.clone(),
            })
        })
        .collect()
}

fn push_scalar_change(
    output: &mut Vec<CreatorScalarChangeV1>,
    field: &str,
    base: &str,
    candidate: &str,
) {
    if base != candidate {
        output.push(CreatorScalarChangeV1 {
            field: field.to_owned(),
            base: base.to_owned(),
            candidate: candidate.to_owned(),
        });
    }
}

fn string_set_delta(base: &[String], candidate: &[String]) -> CreatorStringSetDeltaV1 {
    let base = base.iter().cloned().collect::<BTreeSet<_>>();
    let candidate = candidate.iter().cloned().collect::<BTreeSet<_>>();
    CreatorStringSetDeltaV1 {
        added: candidate.difference(&base).cloned().collect(),
        removed: base.difference(&candidate).cloned().collect(),
    }
}

fn diff_collection<T, F>(base: &[T], candidate: &[T], key: F) -> Vec<CreatorCollectionChangeV1<T>>
where
    T: Clone + Eq,
    F: Fn(&T) -> String,
{
    let base = base
        .iter()
        .map(|value| (key(value), value))
        .collect::<BTreeMap<_, _>>();
    let candidate = candidate
        .iter()
        .map(|value| (key(value), value))
        .collect::<BTreeMap<_, _>>();
    base.keys()
        .chain(candidate.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(
            |item_key| match (base.get(&item_key), candidate.get(&item_key)) {
                (Some(base), Some(candidate)) if *base == *candidate => None,
                (Some(base), Some(candidate)) => Some(CreatorCollectionChangeV1 {
                    change: "changed".to_owned(),
                    key: item_key,
                    base: Some((*base).clone()),
                    candidate: Some((*candidate).clone()),
                }),
                (Some(base), None) => Some(CreatorCollectionChangeV1 {
                    change: "removed".to_owned(),
                    key: item_key,
                    base: Some((*base).clone()),
                    candidate: None,
                }),
                (None, Some(candidate)) => Some(CreatorCollectionChangeV1 {
                    change: "added".to_owned(),
                    key: item_key,
                    base: None,
                    candidate: Some((*candidate).clone()),
                }),
                (None, None) => None,
            },
        )
        .collect()
}

fn dependency_key(dependency: &CreatorDependencyProjectionV1) -> String {
    format!(
        "{}|{}|{}",
        dependency.source_asset_id, dependency.target_asset_id, dependency.dependency_kind
    )
}

fn asset_revision_projection(asset: &AssetRevisionRefV1) -> CreatorAssetRevisionProjectionV1 {
    CreatorAssetRevisionProjectionV1 {
        asset_id: asset.asset_id.to_hex(),
        record_sha256: asset.record_sha256.to_hex(),
    }
}

fn report_count(value: usize) -> Result<u32, InspectionProjectionError> {
    u32::try_from(value).map_err(|_| InspectionProjectionError)
}

const fn semantic_class(value: ContentSemanticClassV1) -> &'static str {
    match value {
        ContentSemanticClassV1::DomainRelevant => "domain-relevant",
        ContentSemanticClassV1::PresentationOnly => "presentation-only",
    }
}

const fn schema_role(value: SchemaRoleV1) -> &'static str {
    match value {
        SchemaRoleV1::Definition => "definition",
        SchemaRoleV1::Manifest => "manifest",
        SchemaRoleV1::NeutralContent => "neutral-content",
    }
}

const fn schema_encoding(value: SchemaEncodingV1) -> &'static str {
    match value {
        SchemaEncodingV1::CanonicalBinaryV1 => "canonical-binary-v1",
        SchemaEncodingV1::JcsRfc8785 => "jcs-rfc8785",
    }
}

const fn presentation_target(value: PresentationTargetKindV1) -> &'static str {
    match value {
        PresentationTargetKindV1::None => "None",
        PresentationTargetKindV1::Interactive => "Interactive",
        PresentationTargetKindV1::DisplaylessOffscreen => "DisplaylessOffscreen",
    }
}

const fn neutral_kind(value: NeutralRecordKindV1) -> &'static str {
    match value {
        NeutralRecordKindV1::Scene => "scene",
        NeutralRecordKindV1::Collider => "collider",
        NeutralRecordKindV1::CharacterDefinition => "character-definition",
        NeutralRecordKindV1::ItemDefinition => "item-definition",
        NeutralRecordKindV1::InventoryDefinition => "inventory-definition",
        NeutralRecordKindV1::EquipmentDefinition => "equipment-definition",
        NeutralRecordKindV1::DialogueDefinition => "dialogue-definition",
        NeutralRecordKindV1::QuestDefinition => "quest-definition",
        NeutralRecordKindV1::RelationshipDefinition => "relationship-definition",
        NeutralRecordKindV1::InteractionDefinition => "interaction-definition",
        NeutralRecordKindV1::AbilityDefinition => "ability-definition",
        NeutralRecordKindV1::WorldChunk => "world-chunk",
    }
}
