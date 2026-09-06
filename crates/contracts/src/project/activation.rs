use super::codec::ProjectContractError;
use super::content::{ContentManifestV1, ContentSemanticClassV1};
use super::lock::ProjectLockV3;
use super::schema::SchemaRegistryManifestV2;
use super::world_partition::WorldPartitionManifestV1;
use crate::body::{BODY_SCHEMA_ASSET_SCHEMA_ID, BodySchemaAssetV1};
use crate::cognition::AgentCognitionCatalogV1;
use crate::render_content::{NeutralRenderRecordV1, RenderContentCatalogV1};
use crate::world_activity::WorldActivityCatalogV1;
use crate::world_population::{WorldNavigationCatalogV1, WorldPopulationCatalogV1};
use crate::world_routine::WorldRoutineCatalogV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivatedProjectV8 {
    pub project_lock: ProjectLockV3,
    pub schema_registry: SchemaRegistryManifestV2,
    pub content_manifest: ContentManifestV1,
    pub world_partition: WorldPartitionManifestV1,
    pub neutral_records: Vec<crate::content::NeutralRecordV1>,
    pub text_catalogs: Vec<crate::localization::TextCatalogV1>,
    pub audio_clips: Vec<crate::audio::NeutralAudioV1>,
    pub neutral_skeletons: Vec<crate::animation_content::NeutralSkeletonV1>,
    pub neutral_animations: Vec<crate::animation_content::NeutralAnimationV1>,
    pub body_schema_asset: BodySchemaAssetV1,
    pub rpg_definitions: crate::mechanics::RpgDefinitionRegistryV2,
    pub world_routine_catalog_or_none: Option<WorldRoutineCatalogV1>,
    pub world_navigation_catalog: WorldNavigationCatalogV1,
    pub world_population_catalog: WorldPopulationCatalogV1,
    pub agent_cognition_catalog: AgentCognitionCatalogV1,
    pub world_activity_catalog: WorldActivityCatalogV1,
    pub render_content_catalog: RenderContentCatalogV1,
}

impl ActivatedProjectV8 {
    pub fn validate(&self) -> Result<(), ProjectContractError> {
        self.project_lock.validate()?;
        self.rpg_definitions
            .validate()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        if self
            .neutral_records
            .windows(2)
            .any(|pair| pair[0].asset_id >= pair[1].asset_id)
        {
            return Err(ProjectContractError::DuplicateIdentity);
        }
        for record in &self.neutral_records {
            let record_hash = record
                .record_sha256()
                .map_err(|_| ProjectContractError::HashMismatch)?;
            if !self
                .content_manifest
                .body
                .asset_entries
                .iter()
                .any(|entry| {
                    entry.asset_revision.asset_id == record.asset_id
                        && entry.asset_revision.record_sha256 == record_hash
                })
            {
                return Err(ProjectContractError::HashMismatch);
            }
        }
        if self
            .text_catalogs
            .windows(2)
            .any(|pair| pair[0].catalog_asset_id >= pair[1].catalog_asset_id)
        {
            return Err(ProjectContractError::DuplicateIdentity);
        }
        let mut locales = std::collections::BTreeSet::new();
        let mut root_count = 0_usize;
        for catalog in &self.text_catalogs {
            let record_hash = catalog
                .record_sha256()
                .map_err(|_| ProjectContractError::HashMismatch)?;
            if !self
                .content_manifest
                .body
                .asset_entries
                .iter()
                .any(|entry| {
                    entry.asset_revision.asset_id == catalog.catalog_asset_id
                        && entry.asset_revision.record_sha256 == record_hash
                })
            {
                return Err(ProjectContractError::HashMismatch);
            }
            if !locales.insert(catalog.locale.as_str()) {
                return Err(ProjectContractError::DuplicateIdentity);
            }
            if catalog.fallback_locale_or_none.is_none() {
                root_count += 1;
            }
        }
        if !self.text_catalogs.is_empty() && root_count != 1 {
            return Err(ProjectContractError::MissingReference);
        }
        for catalog in &self.text_catalogs {
            let mut visited = std::collections::BTreeSet::new();
            let mut current = catalog;
            loop {
                if !visited.insert(current.locale.as_str()) {
                    return Err(ProjectContractError::DependencyCycle);
                }
                let Some(fallback) = &current.fallback_locale_or_none else {
                    break;
                };
                current = self
                    .text_catalogs
                    .iter()
                    .find(|candidate| candidate.locale.as_str() == fallback.as_str())
                    .ok_or(ProjectContractError::MissingReference)?;
            }
        }
        if self
            .audio_clips
            .windows(2)
            .any(|pair| pair[0].asset_id >= pair[1].asset_id)
        {
            return Err(ProjectContractError::DuplicateIdentity);
        }
        for clip in &self.audio_clips {
            let record_hash = clip
                .record_sha256()
                .map_err(|_| ProjectContractError::HashMismatch)?;
            if !self
                .content_manifest
                .body
                .asset_entries
                .iter()
                .any(|entry| {
                    entry.asset_revision.asset_id == clip.asset_id
                        && entry.asset_revision.record_sha256 == record_hash
                })
            {
                return Err(ProjectContractError::HashMismatch);
            }
        }
        if self
            .neutral_skeletons
            .windows(2)
            .any(|pair| pair[0].asset_id >= pair[1].asset_id)
            || self
                .neutral_animations
                .windows(2)
                .any(|pair| pair[0].asset_id >= pair[1].asset_id)
        {
            return Err(ProjectContractError::DuplicateIdentity);
        }
        for skeleton in &self.neutral_skeletons {
            let record_hash = skeleton
                .record_sha256()
                .map_err(|_| ProjectContractError::HashMismatch)?;
            if !self
                .content_manifest
                .body
                .asset_entries
                .iter()
                .any(|entry| {
                    entry.asset_revision.asset_id == skeleton.asset_id
                        && entry.asset_revision.record_sha256 == record_hash
                })
            {
                return Err(ProjectContractError::HashMismatch);
            }
        }
        for animation in &self.neutral_animations {
            let record_hash = animation
                .record_sha256()
                .map_err(|_| ProjectContractError::HashMismatch)?;
            if !self
                .content_manifest
                .body
                .asset_entries
                .iter()
                .any(|entry| {
                    entry.asset_revision.asset_id == animation.asset_id
                        && entry.asset_revision.record_sha256 == record_hash
                })
                || !self.neutral_skeletons.iter().any(|skeleton| {
                    skeleton.asset_revision().ok() == Some(animation.skeleton_revision)
                })
            {
                return Err(ProjectContractError::HashMismatch);
            }
        }
        self.body_schema_asset
            .validate()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        let body_schema_revision = self
            .body_schema_asset
            .record_sha256()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        let matching_body_entries = self
            .content_manifest
            .body
            .asset_entries
            .iter()
            .filter(|entry| {
                entry.asset_revision.asset_id == self.body_schema_asset.asset_id
                    && entry.asset_revision.record_sha256 == body_schema_revision
                    && entry.schema_ref.schema_id.as_str() == BODY_SCHEMA_ASSET_SCHEMA_ID
                    && entry.semantic_class == ContentSemanticClassV1::DomainRelevant
            })
            .count();
        let matching_body_roots = self
            .content_manifest
            .body
            .root_assets
            .iter()
            .filter(|root| {
                root.asset_id == self.body_schema_asset.asset_id
                    && root.record_sha256 == body_schema_revision
            })
            .count();
        if matching_body_entries != 1 || matching_body_roots != 1 {
            return Err(ProjectContractError::HashMismatch);
        }
        let catalog_bytes = self
            .render_content_catalog
            .canonical_bytes()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        // Scene look L6a (plan `look/06a`): the catalog's texture field
        // outgrows the default limits.
        let decoded_catalog = RenderContentCatalogV1::from_canonical_bytes(
            &catalog_bytes,
            crate::render_content::RENDER_CONTENT_DECODE_LIMITS,
        )
        .map_err(|_| ProjectContractError::HashMismatch)?;
        if decoded_catalog != self.render_content_catalog {
            return Err(ProjectContractError::HashMismatch);
        }
        let mut catalog_revisions = Vec::with_capacity(
            1 + self.render_content_catalog.meshes().len()
                + self.render_content_catalog.materials().len()
                + self.render_content_catalog.textures().len()
                + self.render_content_catalog.base_skinning_profiles().len(),
        );
        catalog_revisions.push(self.render_content_catalog.profile_revision());
        catalog_revisions.extend(
            self.render_content_catalog
                .meshes()
                .iter()
                .map(|mesh| {
                    mesh.asset_revision()
                        .map_err(|_| ProjectContractError::HashMismatch)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        catalog_revisions.extend(
            self.render_content_catalog
                .materials()
                .iter()
                .map(|material| {
                    material
                        .asset_revision()
                        .map_err(|_| ProjectContractError::HashMismatch)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        catalog_revisions.extend(
            self.render_content_catalog
                .textures()
                .iter()
                .map(|texture| {
                    texture
                        .asset_revision()
                        .map_err(|_| ProjectContractError::HashMismatch)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        catalog_revisions.extend(
            self.render_content_catalog
                .base_skinning_profiles()
                .iter()
                .map(|profile| {
                    let mesh = self
                        .render_content_catalog
                        .mesh(profile.mesh_revision())
                        .ok_or(ProjectContractError::HashMismatch)?;
                    let skeleton = self
                        .neutral_skeletons
                        .iter()
                        .find(|skeleton| {
                            skeleton.asset_revision().ok() == Some(profile.skeleton_revision())
                        })
                        .ok_or(ProjectContractError::HashMismatch)?;
                    profile
                        .validate_against(mesh, skeleton, &self.body_schema_asset)
                        .map_err(|_| ProjectContractError::HashMismatch)?;
                    profile
                        .asset_revision()
                        .map_err(|_| ProjectContractError::HashMismatch)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        catalog_revisions.sort();
        let mut manifest_render_revisions = self
            .content_manifest
            .body
            .asset_entries
            .iter()
            .filter(|entry| NeutralRenderRecordV1::supports_schema_id(&entry.schema_ref.schema_id))
            .map(|entry| entry.asset_revision)
            .collect::<Vec<_>>();
        manifest_render_revisions.sort();
        if catalog_revisions != manifest_render_revisions {
            return Err(ProjectContractError::HashMismatch);
        }
        let conditioned_interactions = self
            .rpg_definitions
            .interactions
            .iter()
            .filter_map(|interaction| interaction.availability_condition_or_none)
            .collect::<Vec<_>>();
        match self.world_routine_catalog_or_none {
            Some(catalog) => {
                catalog
                    .validate()
                    .map_err(|_| ProjectContractError::HashMismatch)?;
                let revision = catalog
                    .revision()
                    .map_err(|_| ProjectContractError::HashMismatch)?;
                let matching_entries = self
                    .content_manifest
                    .body
                    .asset_entries
                    .iter()
                    .filter(|entry| {
                        entry.asset_revision.asset_id == catalog.catalog_asset_id
                            && entry.asset_revision.record_sha256 == revision
                            && entry.schema_ref.schema_id.as_str()
                                == crate::world_routine::WORLD_ROUTINE_CATALOG_SCHEMA_ID
                    })
                    .count();
                let matching_roots = self
                    .content_manifest
                    .body
                    .root_assets
                    .iter()
                    .filter(|root| {
                        root.asset_id == catalog.catalog_asset_id && root.record_sha256 == revision
                    })
                    .count();
                if matching_entries != 1
                    || matching_roots != 1
                    || conditioned_interactions.is_empty()
                    || conditioned_interactions.iter().any(|condition| {
                        condition.subject_id != catalog.routine.subject_id
                            || !matches!(
                                condition.required_activity,
                                crate::world_routine::WorldRoutineActivityV1::Duty
                                    | crate::world_routine::WorldRoutineActivityV1::Rest
                            )
                    })
                {
                    return Err(ProjectContractError::HashMismatch);
                }
            }
            None if conditioned_interactions.is_empty() => {}
            None => return Err(ProjectContractError::HashMismatch),
        }
        self.world_population_catalog
            .validate_against_navigation(&self.world_navigation_catalog)
            .map_err(|_| ProjectContractError::HashMismatch)?;
        self.agent_cognition_catalog
            .validate()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        self.world_activity_catalog
            .validate()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        if self
            .world_population_catalog
            .definition(self.agent_cognition_catalog.subject_id)
            .is_none()
            || self.world_activity_catalog.worker_subject_id
                != self.agent_cognition_catalog.subject_id
            || self
                .world_population_catalog
                .definition(self.world_activity_catalog.worker_subject_id)
                .is_none_or(|definition| {
                    definition.navigation_goal_node_id
                        != self.world_activity_catalog.workplace_node_id
                })
        {
            return Err(ProjectContractError::HashMismatch);
        }
        let navigation_revision = self
            .world_navigation_catalog
            .revision()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        let population_revision = self
            .world_population_catalog
            .revision(&self.world_navigation_catalog)
            .map_err(|_| ProjectContractError::HashMismatch)?;
        let cognition_revision = self
            .agent_cognition_catalog
            .revision()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        let activity_revision = self
            .world_activity_catalog
            .revision()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        for (asset_id, revision, schema_id) in [
            (
                self.world_navigation_catalog.catalog_asset_id,
                navigation_revision,
                crate::world_population::WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
            ),
            (
                self.world_population_catalog.catalog_asset_id,
                population_revision,
                crate::world_population::WORLD_POPULATION_CATALOG_SCHEMA_ID,
            ),
            (
                self.agent_cognition_catalog.catalog_asset_id,
                cognition_revision,
                crate::cognition::AGENT_COGNITION_CATALOG_SCHEMA_ID,
            ),
            (
                self.world_activity_catalog.catalog_asset_id,
                activity_revision,
                crate::world_activity::WORLD_ACTIVITY_CATALOG_SCHEMA_ID,
            ),
        ] {
            let matching_entries = self
                .content_manifest
                .body
                .asset_entries
                .iter()
                .filter(|entry| {
                    entry.asset_revision.asset_id == asset_id
                        && entry.asset_revision.record_sha256 == revision
                        && entry.schema_ref.schema_id.as_str() == schema_id
                        && entry.semantic_class == ContentSemanticClassV1::DomainRelevant
                })
                .count();
            let matching_roots = self
                .content_manifest
                .body
                .root_assets
                .iter()
                .filter(|root| root.asset_id == asset_id && root.record_sha256 == revision)
                .count();
            if matching_entries != 1 || matching_roots != 1 {
                return Err(ProjectContractError::HashMismatch);
            }
        }
        let navigation_chunks = self
            .world_navigation_catalog
            .nodes
            .iter()
            .map(|node| node.chunk_id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let partition_chunks = self
            .world_partition
            .body
            .chunk_bindings
            .iter()
            .map(|binding| binding.chunk_id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if navigation_chunks != partition_chunks
            || self.world_navigation_catalog.nodes.iter().any(|node| {
                self.world_partition
                    .body
                    .chunk_bindings
                    .iter()
                    .find(|binding| binding.chunk_id == node.chunk_id)
                    .is_none_or(|binding| {
                        self.world_navigation_catalog.region_for_node(&node.node_id)
                            != Some(&binding.region_id)
                    })
            })
            || self.world_partition.body.initial_placement_catalog_sha256 != population_revision
            || self.world_partition.body.residency_policy_sha256
                != self
                    .world_population_catalog
                    .residency_policy_revision(&self.world_navigation_catalog)
                    .map_err(|_| ProjectContractError::HashMismatch)?
        {
            return Err(ProjectContractError::HashMismatch);
        }
        if self.project_lock.project_id != self.content_manifest.body.project_id
            || self.project_lock.project_revision != self.content_manifest.body.content_revision
            || self.project_lock.project_revision != self.world_partition.body.topology_revision
            || self.project_lock.schema_registry_manifest_sha256
                != self.schema_registry.schema_registry_manifest_sha256
            || self.project_lock.content_manifest_sha256
                != self.content_manifest.content_manifest_sha256
            || self.project_lock.world_partition_manifest_sha256
                != self.world_partition.world_partition_manifest_sha256
            || self.content_manifest.body.schema_registry_manifest_sha256
                != self.schema_registry.schema_registry_manifest_sha256
            || self.world_partition.body.schema_registry_manifest_sha256
                != self.schema_registry.schema_registry_manifest_sha256
            || self.world_partition.body.content_manifest_sha256
                != self.content_manifest.content_manifest_sha256
            || self.project_lock.mechanics_lock_sha256
                != self.rpg_definitions.mechanics_lock.mechanics_lock_sha256
        {
            return Err(ProjectContractError::HashMismatch);
        }
        self.schema_registry.to_jcs_bytes()?;
        self.content_manifest.to_jcs_bytes()?;
        self.world_partition.to_jcs_bytes()?;
        Ok(())
    }
}
