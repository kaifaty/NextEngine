//! File-backed project authoring input.
//!
//! The authoring manifest is deliberately a tool-side schema. It carries
//! source spans and provenance references, then normalizes into the bounded
//! neutral records consumed by the existing resolve/cook/activate path. The
//! runtime never opens this file or any referenced source path.

mod cognition;
mod error_impl;
mod schema;

use std::collections::BTreeMap;
use std::path::{Component, Path};

use self::cognition::build_agent_cognition_catalog;
use self::schema::{
    AUTHORING_FORMAT_V5, AuthoringAnimationPropertyV1, AuthoringAudioRecordV1,
    AuthoringHumanoidCatalogV1, AuthoringNeutralRecordKindV1, AuthoringPresentationTargetV1,
    AuthoringRenderRecordV1, AuthoringSourceReferenceV1, AuthoringSourceSpanV1,
    AuthoringTextureAlphaV1, AuthoringTextureColorSpaceV1, AuthoringWorldRoutineActivityV1,
    ProjectAuthoringManifestV5,
};
use crate::cook::{NeutralProjectSourceV5, SourceChunkBindingV1};
use crate::cook_support::schema_ref;
use next_contracts::animation_content::{
    AnimationInterpolationV1, AnimationPropertyV1, AnimationWrapModeV1, NeutralAnimationChannelV1,
    NeutralAnimationContentErrorV1, NeutralAnimationKeyV1, NeutralAnimationV1,
    NeutralAnimationValueV1, NeutralSkeletonJointV1, NeutralSkeletonV1, NeutralTransformV1,
};
use next_contracts::audio::{
    AudioLoudnessMetadataV1, AudioPcmEncodingV1, NeutralAudioErrorV1, NeutralAudioV1,
};
use next_contracts::cognition::{
    AgentCognitionCatalogV1, BeliefContradictionV1, BeliefSourceV1, COGNITION_SCHEMA_VERSION,
    SemanticBeliefV1,
};
use next_contracts::content::{
    NeutralPropertyV1, NeutralRecordError, NeutralRecordKindV1, NeutralRecordV1,
};
use next_contracts::ids::{
    AssetId, ContentHash, IdentifierError, PersistentId, ProjectId, SchemaId,
};
use next_contracts::localization::{
    TextCatalogEntryV1, TextCatalogErrorV1, TextCatalogV1, TextLocaleTagV1,
};
use next_contracts::platform::PresentationTargetKindV1;
use next_contracts::project::{
    AssetRevisionRefV1, ContentProvenanceV1, ProjectContractError, SchemaEncodingV1, SchemaRoleV1,
    domain_hash,
};
use next_contracts::render_content::{
    AabbI64V1, B0_RENDER_CONTENT_PROFILE_SCHEMA_ID, B0RenderContentProfileV1, MaterialAlphaModeV1,
    MaterialColorSpaceV1, MaterialTextureSlotV1, MeshPrimitiveTopologyV1,
    NEUTRAL_MATERIAL_SCHEMA_ID, NEUTRAL_MESH_SCHEMA_ID, NEUTRAL_TEXTURE_SCHEMA_ID,
    NeutralMaterialTextureBindingV1, NeutralMaterialV1, NeutralMeshPrimitiveV1, NeutralMeshV1,
    NeutralRenderRecordV1, NeutralTexelEncodingV1, NeutralTextureAlphaSemanticsV1,
    NeutralTextureColorSpaceV1, NeutralTextureDimensionV1, NeutralTextureMipLevelV1,
    NeutralTextureV1, RenderContentContractError, UvTransformV1,
    b0_shader_interface_manifest_sha256,
};
use next_contracts::world_population::{
    NavigationCapabilityV1, PopulationCadenceClassV1, PopulationTierV1,
    WORLD_POPULATION_ACTIVE_COUNT_V1, WORLD_POPULATION_BACKGROUND_COUNT_V1,
    WORLD_POPULATION_COUNT_V1, WORLD_POPULATION_NEAR_COUNT_V1, WorldNavigationCatalogV1,
    WorldNavigationEdgeV1, WorldNavigationNodeV1, WorldNavigationTileV1, WorldPopulationCatalogV1,
    WorldPopulationDefinitionV1, navigation_tile_revision, population_cadence_phase,
};
use next_contracts::world_routine::{
    WorldRoutineActivityV1, WorldRoutineCatalogV1, WorldRoutineDefinitionV1,
    WorldRoutineInteractionBindingV1, WorldRoutineProfileV1,
};

pub const PROJECT_AUTHORING_MANIFEST_FILE: &str = "project.authoring.json";

#[derive(serde::Deserialize)]
struct ProjectAuthoringFormatProbe {
    format: String,
}

pub fn load_project_authoring_v5(
    project_directory: impl AsRef<Path>,
) -> Result<NeutralProjectSourceV5, ProjectAuthoringError> {
    load_project_authoring_with_override(project_directory.as_ref(), None)
}

pub fn load_project_authoring_v5_with_project_id(
    project_directory: impl AsRef<Path>,
    project_id: &str,
) -> Result<NeutralProjectSourceV5, ProjectAuthoringError> {
    load_project_authoring_with_override(project_directory.as_ref(), Some(project_id))
}

fn load_project_authoring_with_override(
    project_directory: &Path,
    project_id_override: Option<&str>,
) -> Result<NeutralProjectSourceV5, ProjectAuthoringError> {
    let manifest_path = project_directory.join(PROJECT_AUTHORING_MANIFEST_FILE);
    let bytes = read_file(&manifest_path)?;
    let format: ProjectAuthoringFormatProbe = serde_json::from_slice(&bytes)?;
    if format.format != AUTHORING_FORMAT_V5 {
        return Err(ProjectAuthoringError::UnsupportedFormat(format.format));
    }
    let manifest: ProjectAuthoringManifestV5 = serde_json::from_slice(&bytes)?;
    if manifest.format != AUTHORING_FORMAT_V5 {
        return Err(ProjectAuthoringError::UnsupportedFormat(manifest.format));
    }
    validate_span(project_directory, &manifest.provenance.source_span)?;
    let license_manifest_sha256 = validate_provenance(project_directory, &manifest)?;

    let records = manifest
        .records
        .iter()
        .map(|record| {
            validate_span(project_directory, &record.source_span)?;
            NeutralRecordV1::new(
                schema_ref(
                    neutral_kind(record.kind).schema_id(),
                    SchemaRoleV1::Definition,
                    SchemaEncodingV1::CanonicalBinaryV1,
                )?,
                asset_id(&record.asset_id)?,
                neutral_kind(record.kind),
                persistent_id(&record.record_id)?,
                record
                    .persistent_references
                    .iter()
                    .map(|value| persistent_id(value))
                    .collect::<Result<Vec<_>, _>>()?,
                record
                    .asset_dependencies
                    .iter()
                    .map(|value| asset_id(value))
                    .collect::<Result<Vec<_>, _>>()?,
                record
                    .properties
                    .iter()
                    .map(|property| {
                        Ok(NeutralPropertyV1 {
                            property_id: SchemaId::new(&property.property_id)?,
                            value_id: SchemaId::new(&property.value_id)?,
                        })
                    })
                    .collect::<Result<Vec<_>, ProjectAuthoringError>>()?,
            )
            .map_err(ProjectAuthoringError::from)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let render_records = build_render_records(project_directory, &manifest.render_records)?;
    let text_catalogs = build_text_catalogs(project_directory, &manifest.text_catalogs)?;
    let audio_records = build_audio_records(project_directory, &manifest.audio_records)?;
    let (skeletons, animations) = build_animation_catalogs(project_directory, &manifest)?;
    let chunks = manifest
        .partition
        .chunks
        .iter()
        .map(|chunk| {
            validate_span(project_directory, &chunk.source_span)?;
            Ok(SourceChunkBindingV1 {
                chunk_id: SchemaId::new(&chunk.chunk_id)?,
                region_id: SchemaId::new(&chunk.region_id)?,
                chunk_asset_id: asset_id(&chunk.chunk_asset_id)?,
                required_asset_ids: chunk
                    .required_asset_ids
                    .iter()
                    .map(|value| asset_id(value))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect::<Result<Vec<_>, ProjectAuthoringError>>()?;
    let allowed_presentation_targets = manifest
        .allowed_presentation_targets
        .iter()
        .map(|target| match target {
            AuthoringPresentationTargetV1::None => PresentationTargetKindV1::None,
            AuthoringPresentationTargetV1::Interactive => PresentationTargetKindV1::Interactive,
            AuthoringPresentationTargetV1::DisplaylessOffscreen => {
                PresentationTargetKindV1::DisplaylessOffscreen
            }
        })
        .collect();
    let activity = |value: AuthoringWorldRoutineActivityV1| match value {
        AuthoringWorldRoutineActivityV1::Duty => WorldRoutineActivityV1::Duty,
        AuthoringWorldRoutineActivityV1::Rest => WorldRoutineActivityV1::Rest,
    };
    let world_routine_catalog_or_none = manifest
        .world_routine_catalog
        .as_ref()
        .map(|catalog| {
            let value = WorldRoutineCatalogV1 {
                schema_version: catalog.schema_version,
                catalog_asset_id: asset_id(&catalog.catalog_asset_id)?,
                profile: WorldRoutineProfileV1 {
                    schema_version: catalog.profile.schema_version,
                    anchor_simulation_tick: catalog.profile.anchor_simulation_tick,
                    anchor_world_tick: catalog.profile.anchor_world_tick,
                    world_ticks_per_simulation_tick_num: catalog
                        .profile
                        .world_ticks_per_simulation_tick_num,
                    world_ticks_per_simulation_tick_den: catalog
                        .profile
                        .world_ticks_per_simulation_tick_den,
                },
                routine: WorldRoutineDefinitionV1 {
                    schema_version: catalog.routine.schema_version,
                    subject_id: persistent_id(&catalog.routine.subject_id)?,
                    initial_activity: activity(catalog.routine.initial_activity),
                    transition_world_tick: catalog.routine.transition_world_tick,
                    next_activity: activity(catalog.routine.next_activity),
                },
            };
            value
                .validate()
                .map_err(|_| ProjectAuthoringError::InvalidValue)?;
            Ok::<_, ProjectAuthoringError>(value)
        })
        .transpose()?;
    let world_routine_interaction_binding_or_none = manifest
        .world_routine_interaction_binding
        .as_ref()
        .map(|binding| {
            Ok::<_, ProjectAuthoringError>(WorldRoutineInteractionBindingV1 {
                interaction_id: SchemaId::new(&binding.interaction_id)?,
                subject_id: persistent_id(&binding.subject_id)?,
                required_activity: activity(binding.required_activity),
            })
        })
        .transpose()?;
    match (
        &world_routine_catalog_or_none,
        &world_routine_interaction_binding_or_none,
    ) {
        (Some(catalog), Some(binding)) => binding
            .validate_against(catalog)
            .map_err(|_| ProjectAuthoringError::InvalidValue)?,
        (None, None) => {}
        _ => return Err(ProjectAuthoringError::InvalidValue),
    }
    let world_navigation_catalog = build_world_navigation_catalog(&manifest, &chunks)?;
    let world_population_catalog =
        build_world_population_catalog(&manifest, &world_navigation_catalog)?;
    let agent_cognition_catalog =
        build_agent_cognition_catalog(&manifest, &world_population_catalog)?;
    Ok(NeutralProjectSourceV5 {
        project_id: ProjectId::new(
            project_id_override.unwrap_or(manifest.project.project_id.as_str()),
        )?,
        project_revision: manifest.project.project_revision,
        authoring_sha256: domain_hash(AUTHORING_FORMAT_V5, &bytes),
        records,
        render_records,
        text_catalogs,
        audio_records,
        skeletons,
        animations,
        world_routine_catalog_or_none,
        world_routine_interaction_binding_or_none,
        world_navigation_catalog,
        world_population_catalog,
        agent_cognition_catalog,
        root_asset_ids: manifest
            .root_asset_ids
            .iter()
            .map(|value| asset_id(value))
            .collect::<Result<Vec<_>, _>>()?,
        provenance: ContentProvenanceV1::new(
            SchemaId::new(&manifest.provenance.provenance_id)?,
            SchemaId::new(&manifest.provenance.license_id)?,
            manifest.provenance.attribution,
        )?,
        license_manifest_sha256,
        partition_id: SchemaId::new(&manifest.partition.partition_id)?,
        coordinate_profile_id: SchemaId::new(&manifest.partition.coordinate_profile_id)?,
        chunks,
        allowed_presentation_targets,
    })
}

fn build_world_navigation_catalog(
    manifest: &ProjectAuthoringManifestV5,
    chunks: &[SourceChunkBindingV1],
) -> Result<WorldNavigationCatalogV1, ProjectAuthoringError> {
    let authored = &manifest.world_navigation_catalog;
    if authored.schema_version != next_contracts::world_population::WORLD_POPULATION_SCHEMA_VERSION
        || authored.topology_revision != manifest.project.project_revision
        || authored.edge_cost == 0
        || chunks.len() < 2
    {
        return Err(ProjectAuthoringError::InvalidValue);
    }
    let mut nodes = chunks
        .iter()
        .map(|chunk| {
            Ok(WorldNavigationNodeV1 {
                node_id: chunk.chunk_id.clone(),
                tile_id: SchemaId::new(format!("{}.navigation-tile", chunk.region_id.as_str()))?,
                chunk_id: chunk.chunk_id.clone(),
            })
        })
        .collect::<Result<Vec<_>, ProjectAuthoringError>>()?;
    nodes.sort();
    let mut region_tiles = BTreeMap::<SchemaId, SchemaId>::new();
    for chunk in chunks {
        region_tiles.insert(
            chunk.region_id.clone(),
            SchemaId::new(format!("{}.navigation-tile", chunk.region_id.as_str()))?,
        );
    }
    let mut tiles = region_tiles
        .into_iter()
        .map(|(region_id, tile_id)| {
            let node_ids = nodes
                .iter()
                .filter(|node| node.tile_id == tile_id)
                .map(|node| &node.node_id)
                .collect::<Vec<_>>();
            Ok(WorldNavigationTileV1 {
                tile_revision: navigation_tile_revision(&tile_id, &region_id, &node_ids)
                    .map_err(|_| ProjectAuthoringError::InvalidValue)?,
                tile_id,
                region_id,
            })
        })
        .collect::<Result<Vec<_>, ProjectAuthoringError>>()?;
    tiles.sort();
    let mut edges = Vec::with_capacity(nodes.len());
    for index in 0..nodes.len() {
        let left = &nodes[index].node_id;
        let right = &nodes[(index + 1) % nodes.len()].node_id;
        let (node_low, node_high) = if left < right {
            (left.clone(), right.clone())
        } else {
            (right.clone(), left.clone())
        };
        edges.push(WorldNavigationEdgeV1 {
            node_low,
            node_high,
            cost: authored.edge_cost,
            capability: NavigationCapabilityV1::AbstractTransfer,
        });
    }
    edges.sort();
    if edges.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ProjectAuthoringError::DuplicateIdentity);
    }
    let value = WorldNavigationCatalogV1 {
        schema_version: authored.schema_version,
        catalog_asset_id: asset_id(&authored.catalog_asset_id)?,
        topology_revision: authored.topology_revision,
        tiles,
        nodes,
        edges,
    };
    value
        .validate()
        .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    Ok(value)
}

fn build_world_population_catalog(
    manifest: &ProjectAuthoringManifestV5,
    navigation: &WorldNavigationCatalogV1,
) -> Result<WorldPopulationCatalogV1, ProjectAuthoringError> {
    let authored = &manifest.world_population_catalog;
    if authored.schema_version != next_contracts::world_population::WORLD_POPULATION_SCHEMA_VERSION
        || authored.identity_domain.is_empty()
        || usize::try_from(authored.courier_ordinal)
            .ok()
            .is_none_or(|ordinal| ordinal >= WORLD_POPULATION_COUNT_V1)
    {
        return Err(ProjectAuthoringError::InvalidValue);
    }
    let courier_initial_node_id = SchemaId::new(&authored.courier_initial_node_id)?;
    let courier_goal_node_id = SchemaId::new(&authored.courier_goal_node_id)?;
    let node_count = navigation.nodes.len();
    let mut records = Vec::with_capacity(WORLD_POPULATION_COUNT_V1);
    for ordinal in 0..WORLD_POPULATION_COUNT_V1 {
        let subject_id = population_subject_id(&authored.identity_domain, ordinal)?;
        let (cadence_class, initial_tier) = if ordinal < WORLD_POPULATION_ACTIVE_COUNT_V1 {
            (PopulationCadenceClassV1::Active, PopulationTierV1::Active)
        } else if ordinal < WORLD_POPULATION_ACTIVE_COUNT_V1 + WORLD_POPULATION_NEAR_COUNT_V1 {
            (PopulationCadenceClassV1::Near, PopulationTierV1::Simulated)
        } else if ordinal
            == usize::try_from(authored.courier_ordinal)
                .map_err(|_| ProjectAuthoringError::InvalidValue)?
        {
            (
                PopulationCadenceClassV1::Background,
                PopulationTierV1::Dormant,
            )
        } else {
            (
                PopulationCadenceClassV1::Background,
                PopulationTierV1::Abstract,
            )
        };
        let initial_node_id = if ordinal
            == usize::try_from(authored.courier_ordinal)
                .map_err(|_| ProjectAuthoringError::InvalidValue)?
        {
            courier_initial_node_id.clone()
        } else {
            navigation.nodes[ordinal % node_count].node_id.clone()
        };
        let navigation_goal_node_id = if ordinal
            == usize::try_from(authored.courier_ordinal)
                .map_err(|_| ProjectAuthoringError::InvalidValue)?
        {
            courier_goal_node_id.clone()
        } else {
            navigation.nodes[(ordinal + node_count / 2) % node_count]
                .node_id
                .clone()
        };
        records.push(WorldPopulationDefinitionV1 {
            subject_id,
            home_node_id: initial_node_id.clone(),
            initial_node_id,
            navigation_goal_node_id,
            cadence_class,
            cadence_phase: population_cadence_phase(subject_id, cadence_class),
            initial_tier,
        });
    }
    records.sort();
    let courier_ordinal = usize::try_from(authored.courier_ordinal)
        .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    let courier_subject_id = records
        .iter()
        .find(|record| {
            record.initial_node_id == courier_initial_node_id
                && record.initial_tier == PopulationTierV1::Dormant
        })
        .map(|record| record.subject_id)
        .ok_or(ProjectAuthoringError::InvalidValue)?;
    let _ = courier_ordinal;
    debug_assert_eq!(
        records
            .iter()
            .filter(|record| record.cadence_class == PopulationCadenceClassV1::Background)
            .count(),
        WORLD_POPULATION_BACKGROUND_COUNT_V1
    );
    let value = WorldPopulationCatalogV1 {
        schema_version: authored.schema_version,
        catalog_asset_id: asset_id(&authored.catalog_asset_id)?,
        navigation_catalog_asset_id: navigation.catalog_asset_id,
        navigation_catalog_revision: navigation
            .revision()
            .map_err(|_| ProjectAuthoringError::InvalidValue)?,
        courier_subject_id,
        courier_transition_start_tick: authored.courier_transition_start_tick,
        records,
    };
    value
        .validate_against_navigation(navigation)
        .map_err(|_| ProjectAuthoringError::InvalidValue)?;
    Ok(value)
}

fn population_subject_id(
    identity_domain: &str,
    ordinal: usize,
) -> Result<PersistentId, ProjectAuthoringError> {
    let mut preimage = b"nextengine.population-subject.v1\0".to_vec();
    append_string(&mut preimage, identity_domain)?;
    preimage.extend_from_slice(
        &u32::try_from(ordinal)
            .map_err(|_| ProjectAuthoringError::InvalidValue)?
            .to_le_bytes(),
    );
    let digest = next_contracts::canonical::sha256(&preimage);
    let mut subject_bytes = [0_u8; 16];
    subject_bytes.copy_from_slice(&digest[..16]);
    Ok(PersistentId::from_bytes(subject_bytes))
}

fn build_render_records(
    project_directory: &Path,
    authored: &[AuthoringRenderRecordV1],
) -> Result<Vec<NeutralRenderRecordV1>, ProjectAuthoringError> {
    let mut records = Vec::new();
    let mut revisions = BTreeMap::<AssetId, AssetRevisionRefV1>::new();
    for record in authored {
        validate_span(project_directory, record.source_span())?;
        match record {
            AuthoringRenderRecordV1::Mesh {
                asset_id: id,
                record_revision,
                bounds_min,
                bounds_max,
                positions_micrometres,
                uv0_q16,
                normals_snorm16,
                indices,
                double_sided,
                ..
            } => {
                let mut indices = indices.clone();
                if *double_sided {
                    let reversed = indices
                        .chunks_exact(3)
                        .flat_map(|triangle| [triangle[0], triangle[2], triangle[1]])
                        .collect::<Vec<_>>();
                    indices.extend(reversed);
                }
                let mesh = NeutralMeshV1::new(
                    schema_ref(
                        NEUTRAL_MESH_SCHEMA_ID,
                        SchemaRoleV1::NeutralContent,
                        SchemaEncodingV1::CanonicalBinaryV1,
                    )?,
                    asset_id(id)?,
                    *record_revision,
                    AabbI64V1::new(*bounds_min, *bounds_max)?,
                    positions_micrometres.clone(),
                    normals_snorm16.clone(),
                    None,
                    if uv0_q16.is_empty() {
                        Vec::new()
                    } else {
                        vec![uv0_q16.clone()]
                    },
                    indices.clone(),
                    vec![NeutralMeshPrimitiveV1::new(
                        MeshPrimitiveTopologyV1::Triangles,
                        0,
                        u32::try_from(indices.len())
                            .map_err(|_| ProjectAuthoringError::InvalidValue)?,
                        0,
                    )?],
                )?;
                insert_revision(&mut revisions, mesh.asset_revision()?)?;
                records.push(mesh.into());
            }
            AuthoringRenderRecordV1::TextureRgba8 {
                asset_id: id,
                record_revision,
                extent,
                color_space,
                alpha,
                texels,
                ..
            } => {
                let texture = NeutralTextureV1::new(
                    schema_ref(
                        NEUTRAL_TEXTURE_SCHEMA_ID,
                        SchemaRoleV1::NeutralContent,
                        SchemaEncodingV1::CanonicalBinaryV1,
                    )?,
                    asset_id(id)?,
                    *record_revision,
                    NeutralTextureDimensionV1::D2,
                    *extent,
                    1,
                    texture_color_space(*color_space),
                    texture_alpha(*alpha),
                    NeutralTexelEncodingV1::Rgba8Unorm,
                    vec![NeutralTextureMipLevelV1::new(*extent, texels.clone())],
                )?;
                insert_revision(&mut revisions, texture.asset_revision()?)?;
                records.push(texture.into());
            }
            AuthoringRenderRecordV1::Material { .. }
            | AuthoringRenderRecordV1::B0Profile { .. } => {}
        }
    }
    for record in authored {
        if let AuthoringRenderRecordV1::Material {
            asset_id: id,
            record_revision,
            texture_asset_id,
            base_color_rgba_u16,
            metallic_u16,
            roughness_u16,
            emissive_rgb_u16,
            double_sided,
            ..
        } = record
        {
            let texture = revision(&revisions, texture_asset_id)?;
            let material = NeutralMaterialV1::new(
                schema_ref(
                    NEUTRAL_MATERIAL_SCHEMA_ID,
                    SchemaRoleV1::NeutralContent,
                    SchemaEncodingV1::CanonicalBinaryV1,
                )?,
                asset_id(id)?,
                *record_revision,
                *base_color_rgba_u16,
                MaterialColorSpaceV1::Linear,
                *metallic_u16,
                *roughness_u16,
                *emissive_rgb_u16,
                MaterialColorSpaceV1::Linear,
                0,
                65_536,
                u16::MAX,
                MaterialAlphaModeV1::Opaque,
                0,
                *double_sided,
                vec![NeutralMaterialTextureBindingV1::new(
                    MaterialTextureSlotV1::BaseColor,
                    texture,
                    0,
                    UvTransformV1::identity(),
                )?],
                Vec::new(),
            )?;
            insert_revision(&mut revisions, material.asset_revision()?)?;
            records.push(material.into());
        }
    }
    for record in authored {
        if let AuthoringRenderRecordV1::B0Profile {
            asset_id: id,
            record_revision,
            fallback_material_asset_id,
            fallback_texture_asset_id,
            ..
        } = record
        {
            let profile = B0RenderContentProfileV1::new(
                schema_ref(
                    B0_RENDER_CONTENT_PROFILE_SCHEMA_ID,
                    SchemaRoleV1::NeutralContent,
                    SchemaEncodingV1::CanonicalBinaryV1,
                )?,
                asset_id(id)?,
                *record_revision,
                b0_shader_interface_manifest_sha256(),
                revision(&revisions, fallback_material_asset_id)?,
                revision(&revisions, fallback_texture_asset_id)?,
            )?;
            insert_revision(&mut revisions, profile.asset_revision()?)?;
            records.push(profile.into());
        }
    }
    Ok(records)
}

fn build_text_catalogs(
    project_directory: &Path,
    authored: &[schema::AuthoringTextCatalogV1],
) -> Result<Vec<TextCatalogV1>, ProjectAuthoringError> {
    authored
        .iter()
        .map(|catalog| {
            validate_span(project_directory, &catalog.source_span)?;
            TextCatalogV1::new(
                asset_id(&catalog.asset_id)?,
                catalog.revision,
                TextLocaleTagV1::new(&catalog.locale)?,
                catalog
                    .fallback_locale
                    .as_deref()
                    .map(TextLocaleTagV1::new)
                    .transpose()?,
                catalog
                    .entries
                    .iter()
                    .map(|entry| {
                        Ok(TextCatalogEntryV1::new(
                            SchemaId::new(&entry.text_id)?,
                            entry.template.clone(),
                        )?)
                    })
                    .collect::<Result<Vec<_>, ProjectAuthoringError>>()?,
            )
            .map_err(ProjectAuthoringError::from)
        })
        .collect()
}

fn build_audio_records(
    project_directory: &Path,
    authored: &[AuthoringAudioRecordV1],
) -> Result<Vec<NeutralAudioV1>, ProjectAuthoringError> {
    authored
        .iter()
        .map(|record| {
            validate_span(project_directory, record.source_span())?;
            let (id, revision, sample_rate, samples) = match record {
                AuthoringAudioRecordV1::NoiseBurst {
                    asset_id: id,
                    record_revision,
                    sample_rate_hz,
                    frames,
                    amplitude,
                    seed,
                    ..
                } => (
                    id,
                    *record_revision,
                    *sample_rate_hz,
                    synthesize_noise_burst(*frames, *amplitude, *seed),
                ),
                AuthoringAudioRecordV1::TwoTone {
                    asset_id: id,
                    record_revision,
                    sample_rate_hz,
                    frames,
                    amplitude,
                    first_period,
                    second_period,
                    ..
                } => (
                    id,
                    *record_revision,
                    *sample_rate_hz,
                    synthesize_two_tone(*frames, *first_period, *second_period, *amplitude),
                ),
                AuthoringAudioRecordV1::Thud {
                    asset_id: id,
                    record_revision,
                    sample_rate_hz,
                    frames,
                    amplitude,
                    period,
                    ..
                } => (
                    id,
                    *record_revision,
                    *sample_rate_hz,
                    synthesize_thud(*frames, *period, *amplitude),
                ),
            };
            build_audio_clip(asset_id(id)?, revision, sample_rate, &samples)
        })
        .collect()
}

fn build_animation_catalogs(
    project_directory: &Path,
    manifest: &ProjectAuthoringManifestV5,
) -> Result<(Vec<NeutralSkeletonV1>, Vec<NeutralAnimationV1>), ProjectAuthoringError> {
    let mut skeletons = Vec::new();
    let mut animations = Vec::new();
    for reference in &manifest.neutral_animation_catalogs {
        validate_span(project_directory, &reference.source_span)?;
        let provenance = manifest
            .provenance
            .referenced_sources
            .iter()
            .find(|candidate| candidate.relative_path == reference.relative_path)
            .ok_or_else(|| {
                ProjectAuthoringError::MissingReference(reference.relative_path.clone())
            })?;
        let path = safe_join(project_directory, &reference.relative_path)?;
        let catalog: AuthoringHumanoidCatalogV1 = serde_json::from_slice(&read_file(&path)?)?;
        if catalog.format != "nextengine.neutral-humanoid-authoring.v1"
            || catalog.creator.trim().is_empty()
            || catalog.source != provenance.logical_source
            || catalog.license_expression != provenance.license_expression
        {
            return Err(ProjectAuthoringError::InvalidProvenance);
        }
        let asset_set_id = SchemaId::new(&catalog.asset_set_id)?;
        let joint_id = |key: &str| {
            SchemaId::new(format!("{}.joint.{key}", asset_set_id.as_str()))
                .map_err(ProjectAuthoringError::from)
        };
        let joints = catalog
            .skeleton
            .joints
            .iter()
            .map(|joint| {
                Ok(NeutralSkeletonJointV1 {
                    joint_key: joint_id(&joint.key)?,
                    parent_joint_key: joint.parent.as_deref().map(&joint_id).transpose()?,
                    bind_transform: NeutralTransformV1::translated(joint.translation_micrometres),
                    semantic_roles: Vec::new(),
                })
            })
            .collect::<Result<Vec<_>, ProjectAuthoringError>>()?;
        let roots = catalog
            .skeleton
            .joints
            .iter()
            .filter(|joint| joint.parent.is_none())
            .map(|joint| joint_id(&joint.key))
            .collect::<Result<Vec<_>, _>>()?;
        let skeleton = NeutralSkeletonV1::new(
            asset_id(&catalog.skeleton.asset_id)?,
            catalog.skeleton.record_revision,
            SchemaId::new(&manifest.partition.coordinate_profile_id)?,
            roots,
            joints,
        )?;
        let skeleton_revision = skeleton.asset_revision()?;
        for clip in &catalog.clips {
            let channels = clip
                .channels
                .iter()
                .map(|channel| {
                    let property = match channel.property {
                        AuthoringAnimationPropertyV1::Translation => {
                            AnimationPropertyV1::Translation
                        }
                    };
                    Ok(NeutralAnimationChannelV1 {
                        joint_key: joint_id(&channel.joint)?,
                        property,
                        interpolation: AnimationInterpolationV1::Linear,
                        keys: channel
                            .keys
                            .iter()
                            .map(|key| NeutralAnimationKeyV1 {
                                time_microseconds: key.time_microseconds,
                                value: NeutralAnimationValueV1::Translation(key.value),
                            })
                            .collect(),
                    })
                })
                .collect::<Result<Vec<_>, ProjectAuthoringError>>()?;
            let animation = NeutralAnimationV1::new(
                asset_id(&clip.asset_id)?,
                clip.record_revision,
                SchemaId::new(&clip.clip_id)?,
                skeleton_revision,
                clip.duration_microseconds,
                AnimationWrapModeV1::Loop,
                channels,
                Vec::new(),
                Vec::new(),
            )?;
            for channel in &animation.channels {
                if !skeleton
                    .joints
                    .iter()
                    .any(|joint| joint.joint_key == channel.joint_key)
                {
                    return Err(ProjectAuthoringError::MissingReference(
                        channel.joint_key.as_str().to_owned(),
                    ));
                }
            }
            animations.push(animation);
        }
        skeletons.push(skeleton);
    }
    Ok((skeletons, animations))
}

fn validate_provenance(
    project_directory: &Path,
    manifest: &ProjectAuthoringManifestV5,
) -> Result<ContentHash, ProjectAuthoringError> {
    if manifest.provenance.source_identity.is_empty()
        || manifest.provenance.referenced_sources.is_empty()
    {
        return Err(ProjectAuthoringError::InvalidProvenance);
    }
    let mut references = manifest.provenance.referenced_sources.clone();
    references.sort_by(|left, right| left.logical_source.cmp(&right.logical_source));
    if references
        .windows(2)
        .any(|pair| pair[0].logical_source == pair[1].logical_source)
    {
        return Err(ProjectAuthoringError::InvalidProvenance);
    }
    let mut closure = Vec::new();
    append_string(&mut closure, &manifest.provenance.source_identity)?;
    for reference in &references {
        validate_source_reference(project_directory, reference, &mut closure)?;
    }
    let computed = domain_hash("nextengine.license-manifest.v1", &closure);
    let declared = content_hash(&manifest.provenance.license_manifest_sha256)?;
    if computed != declared {
        return Err(ProjectAuthoringError::HashMismatch(
            "license manifest".to_owned(),
        ));
    }
    Ok(computed)
}

fn validate_source_reference(
    project_directory: &Path,
    reference: &AuthoringSourceReferenceV1,
    closure: &mut Vec<u8>,
) -> Result<(), ProjectAuthoringError> {
    validate_span(project_directory, &reference.source_span)?;
    if reference.logical_source.is_empty() || reference.license_expression.is_empty() {
        return Err(ProjectAuthoringError::InvalidProvenance);
    }
    let source_path = safe_join(project_directory, &reference.relative_path)?;
    let source_bytes = read_file(&source_path)?;
    let actual = ContentHash::from_bytes(next_contracts::canonical::sha256(&source_bytes));
    let expected = content_hash(&reference.sha256)?;
    if actual != expected {
        return Err(ProjectAuthoringError::HashMismatch(
            reference.relative_path.clone(),
        ));
    }
    let notice_path = safe_join(project_directory, &reference.notice_path)?;
    let notice = read_file(&notice_path)?;
    if notice.is_empty() {
        return Err(ProjectAuthoringError::InvalidProvenance);
    }
    append_string(closure, &reference.logical_source)?;
    append_string(closure, &reference.relative_path)?;
    closure.extend_from_slice(expected.as_bytes());
    append_string(closure, &reference.license_expression)?;
    append_string(closure, &reference.notice_path)?;
    closure.extend_from_slice(&next_contracts::canonical::sha256(&notice));
    Ok(())
}

fn validate_span(
    project_directory: &Path,
    span: &AuthoringSourceSpanV1,
) -> Result<(), ProjectAuthoringError> {
    if span.line == 0 || span.column == 0 {
        return Err(ProjectAuthoringError::InvalidSourceSpan);
    }
    let path = safe_join(project_directory, &span.relative_path)?;
    if !path.is_file() {
        return Err(ProjectAuthoringError::InvalidSourceSpan);
    }
    Ok(())
}

fn safe_join(root: &Path, relative: &str) -> Result<std::path::PathBuf, ProjectAuthoringError> {
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ProjectAuthoringError::UnsafePath(
            relative.display().to_string(),
        ));
    }
    Ok(root.join(relative))
}

fn read_file(path: &Path) -> Result<Vec<u8>, ProjectAuthoringError> {
    std::fs::read(path).map_err(|source| ProjectAuthoringError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn asset_id(value: &str) -> Result<AssetId, ProjectAuthoringError> {
    Ok(AssetId::from_bytes(hex_fixed(value)?))
}

fn persistent_id(value: &str) -> Result<PersistentId, ProjectAuthoringError> {
    Ok(PersistentId::from_bytes(hex_fixed(value)?))
}

fn content_hash(value: &str) -> Result<ContentHash, ProjectAuthoringError> {
    Ok(ContentHash::from_bytes(hex_fixed(value)?))
}

fn hex_fixed<const LENGTH: usize>(value: &str) -> Result<[u8; LENGTH], ProjectAuthoringError> {
    if value.len() != LENGTH * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ProjectAuthoringError::InvalidHex);
    }
    let mut output = [0_u8; LENGTH];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(pair).map_err(|_| ProjectAuthoringError::InvalidHex)?;
        output[index] =
            u8::from_str_radix(text, 16).map_err(|_| ProjectAuthoringError::InvalidHex)?;
    }
    Ok(output)
}

mod helpers;
use helpers::*;

#[derive(Debug)]
#[non_exhaustive]
pub enum ProjectAuthoringError {
    Io {
        path: String,
        source: std::io::Error,
    },
    Json(serde_json::Error),
    Identifier(IdentifierError),
    Contract(ProjectContractError),
    Neutral(NeutralRecordError),
    Render(RenderContentContractError),
    Localization(TextCatalogErrorV1),
    Audio(NeutralAudioErrorV1),
    Animation(NeutralAnimationContentErrorV1),
    UnsupportedFormat(String),
    UnsafePath(String),
    InvalidHex,
    InvalidSourceSpan,
    InvalidProvenance,
    InvalidValue,
    DuplicateIdentity,
    MissingReference(String),
    HashMismatch(String),
}
