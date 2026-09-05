use super::*;

pub(super) fn build_render_records(
    project_directory: &Path,
    authored: &[AuthoringRenderRecordV1],
    skeletons: &[NeutralSkeletonV1],
    body_schema_asset: &BodySchemaAssetV1,
    referenced_sources: &[String],
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
            AuthoringRenderRecordV1::TexturePng {
                asset_id: id,
                record_revision,
                relative_path,
                color_space,
                alpha,
                mip_levels,
                ..
            } => {
                // Scene look L5: the file must be a declared referenced
                // source (its bytes and license ride the composition lock).
                if !referenced_sources.iter().any(|path| path == relative_path) {
                    return Err(ProjectAuthoringError::MissingReference(
                        relative_path.clone(),
                    ));
                }
                let bytes = read_file(&safe_join(project_directory, relative_path)?)?;
                let decoded = super::png::decode_png(&bytes)
                    .map_err(|_| ProjectAuthoringError::InvalidValue)?;
                let srgb = matches!(color_space, AuthoringTextureColorSpaceV1::Srgb);
                let levels = match mip_levels {
                    AuthoringTextureMipLevelsV1::Full => {
                        super::png::mip_chain(decoded.width, decoded.height, &decoded.rgba8, srgb)
                    }
                    AuthoringTextureMipLevelsV1::None => {
                        vec![([decoded.width, decoded.height, 1], decoded.rgba8.clone())]
                    }
                };
                let texture = NeutralTextureV1::new(
                    schema_ref(
                        NEUTRAL_TEXTURE_SCHEMA_ID,
                        SchemaRoleV1::NeutralContent,
                        SchemaEncodingV1::CanonicalBinaryV1,
                    )?,
                    asset_id(id)?,
                    *record_revision,
                    NeutralTextureDimensionV1::D2,
                    [decoded.width, decoded.height, 1],
                    1,
                    texture_color_space(*color_space),
                    texture_alpha(*alpha),
                    NeutralTexelEncodingV1::Rgba8Unorm,
                    levels
                        .into_iter()
                        .map(|(extent, texels)| NeutralTextureMipLevelV1::new(extent, texels))
                        .collect(),
                )?;
                insert_revision(&mut revisions, texture.asset_revision()?)?;
                records.push(texture.into());
            }
            AuthoringRenderRecordV1::MeshGltf {
                asset_id: id,
                record_revision,
                relative_path,
                mesh: mesh_index,
                primitive,
                node,
                double_sided,
                ..
            } => {
                // Scene look L5b: the glTF file and every buffer or image it
                // names must be declared referenced sources.
                let document = super::gltf::load_document(
                    project_directory,
                    relative_path,
                    referenced_sources,
                )?;
                let geometry = document
                    .geometry(*mesh_index, *primitive, *node)
                    .map_err(ProjectAuthoringError::Gltf)?;
                let mut indices = geometry.indices.clone();
                if *double_sided {
                    let reversed = indices
                        .chunks_exact(3)
                        .flat_map(|triangle| [triangle[0], triangle[2], triangle[1]])
                        .collect::<Vec<_>>();
                    indices.extend(reversed);
                }
                let tangents = match &geometry.tangents_snorm16 {
                    Some(tangents) => Some(
                        tangents
                            .iter()
                            .map(|(components, handedness)| {
                                NeutralTangentV1::new(*components, *handedness)
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                    None => None,
                };
                let mesh = NeutralMeshV1::new(
                    schema_ref(
                        NEUTRAL_MESH_SCHEMA_ID,
                        SchemaRoleV1::NeutralContent,
                        SchemaEncodingV1::CanonicalBinaryV1,
                    )?,
                    asset_id(id)?,
                    *record_revision,
                    // The neutral bounds are half-open at their maximum.
                    AabbI64V1::new(
                        geometry.bounds_min,
                        geometry.bounds_max.map(|value| value.saturating_add(1)),
                    )?,
                    geometry.positions_micrometres.clone(),
                    geometry.normals_snorm16.clone(),
                    tangents,
                    if geometry.uv0_q16.is_empty() {
                        Vec::new()
                    } else {
                        vec![geometry.uv0_q16.clone()]
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
            AuthoringRenderRecordV1::Material { .. }
            | AuthoringRenderRecordV1::B0Profile { .. }
            | AuthoringRenderRecordV1::BaseSkinningProfile { .. } => {}
        }
    }
    for record in authored {
        if let AuthoringRenderRecordV1::Material {
            asset_id: id,
            record_revision,
            texture_asset_id,
            metallic_roughness_texture_asset_id,
            normal_texture_asset_id,
            uv_scale,
            base_color_rgba_u16,
            metallic_u16,
            roughness_u16,
            emissive_rgb_u16,
            double_sided,
            ..
        } = record
        {
            let texture = revision(&revisions, texture_asset_id)?;
            // Scene look L5: a uniform UV scale shared by the bindings.
            if !(*uv_scale > 0.0 && *uv_scale <= 1024.0) {
                return Err(ProjectAuthoringError::InvalidValue);
            }
            let scale_q16_16 = (uv_scale * 65_536.0).round() as i32;
            let uv_transform = UvTransformV1::new([scale_q16_16, 0, 0, 0, scale_q16_16, 0])?;
            let mut bindings = vec![NeutralMaterialTextureBindingV1::new(
                MaterialTextureSlotV1::BaseColor,
                texture,
                0,
                uv_transform,
            )?];
            if let Some(map) = metallic_roughness_texture_asset_id {
                bindings.push(NeutralMaterialTextureBindingV1::new(
                    MaterialTextureSlotV1::MetallicRoughness,
                    revision(&revisions, map)?,
                    0,
                    uv_transform,
                )?);
            }
            if let Some(map) = normal_texture_asset_id {
                bindings.push(NeutralMaterialTextureBindingV1::new(
                    MaterialTextureSlotV1::Normal,
                    revision(&revisions, map)?,
                    0,
                    uv_transform,
                )?);
            }
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
                bindings,
                Vec::new(),
            )?;
            insert_revision(&mut revisions, material.asset_revision()?)?;
            records.push(material.into());
        }
    }
    for record in authored {
        if let AuthoringRenderRecordV1::BaseSkinningProfile {
            asset_id: id,
            record_revision,
            mesh_asset_id,
            skeleton_asset_id,
            body_schema_asset_id,
            mesh_origin_in_skeleton_micrometres,
            max_instances_per_frame,
            render_joints,
            vertex_joint_ranges,
            pose_correctives,
            ..
        } = record
        {
            let mesh_revision = revision(&revisions, mesh_asset_id)?;
            let mesh = records
                .iter()
                .find_map(|record| match record {
                    NeutralRenderRecordV1::Mesh(mesh)
                        if mesh.asset_id() == mesh_revision.asset_id =>
                    {
                        Some(mesh)
                    }
                    _ => None,
                })
                .ok_or(ProjectAuthoringError::InvalidValue)?;
            let skeleton_id = asset_id(skeleton_asset_id)?;
            let skeleton = skeletons
                .iter()
                .find(|skeleton| skeleton.asset_id == skeleton_id)
                .ok_or(ProjectAuthoringError::InvalidValue)?;
            if asset_id(body_schema_asset_id)? != body_schema_asset.asset_id {
                return Err(ProjectAuthoringError::InvalidValue);
            }
            let mut vertices = vec![None; mesh.positions_micrometres().len()];
            for range in vertex_joint_ranges {
                let start = usize::try_from(range.first_vertex)
                    .map_err(|_| ProjectAuthoringError::InvalidValue)?;
                let count = usize::try_from(range.vertex_count)
                    .map_err(|_| ProjectAuthoringError::InvalidValue)?;
                let end = start
                    .checked_add(count)
                    .ok_or(ProjectAuthoringError::InvalidValue)?;
                if count == 0 || end > vertices.len() {
                    return Err(ProjectAuthoringError::InvalidValue);
                }
                let vertex = NeutralSkinVertexV1::new(vec![NeutralSkinInfluenceV1 {
                    render_joint_id: SchemaId::new(&range.render_joint_id)?,
                    weight_unorm16: u16::MAX,
                }])?;
                for slot in &mut vertices[start..end] {
                    if slot.replace(vertex.clone()).is_some() {
                        return Err(ProjectAuthoringError::InvalidValue);
                    }
                }
            }
            let vertices = vertices
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .ok_or(ProjectAuthoringError::InvalidValue)?;
            pose_correctives
                .iter()
                .try_fold(0_usize, |total, corrective| {
                    corrective
                        .vertex_delta_ranges
                        .iter()
                        .try_fold(total, |total, range| {
                            let start = usize::try_from(range.first_vertex)
                                .map_err(|_| ProjectAuthoringError::InvalidValue)?;
                            let count = usize::try_from(range.vertex_count)
                                .map_err(|_| ProjectAuthoringError::InvalidValue)?;
                            let end = start
                                .checked_add(count)
                                .ok_or(ProjectAuthoringError::InvalidValue)?;
                            if count == 0 || end > mesh.positions_micrometres().len() {
                                return Err(ProjectAuthoringError::InvalidValue);
                            }
                            total
                                .checked_add(count)
                                .filter(|count| *count <= POSE_CORRECTIVE_MAX_VERTEX_DELTAS_V1)
                                .ok_or(ProjectAuthoringError::InvalidValue)
                        })
                })?;
            let profile = NeutralBaseSkinningProfileV1::new(
                schema_ref(
                    NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID,
                    SchemaRoleV1::NeutralContent,
                    SchemaEncodingV1::CanonicalBinaryV1,
                )?,
                asset_id(id)?,
                *record_revision,
                mesh_revision,
                skeleton.asset_revision()?,
                AssetRevisionRefV1 {
                    asset_id: body_schema_asset.asset_id,
                    record_sha256: body_schema_asset
                        .record_sha256()
                        .map_err(|_| ProjectAuthoringError::InvalidValue)?,
                },
                *mesh_origin_in_skeleton_micrometres,
                BaseSkinningMethodV1::LinearBlend,
                BaseSkinningFallbackV1::BindPose,
                *max_instances_per_frame,
                render_joints
                    .iter()
                    .map(|joint| {
                        Ok(NeutralRenderJointV1 {
                            render_joint_id: SchemaId::new(&joint.render_joint_id)?,
                            parent_render_joint_id: joint
                                .parent_render_joint_id
                                .as_deref()
                                .map(SchemaId::new)
                                .transpose()?,
                            animation_joint_id: SchemaId::new(&joint.animation_joint_id)?,
                            body_semantic_id: SchemaId::new(&joint.body_semantic_id)?,
                            bind_transform: NeutralTransformV1::translated(
                                joint.bind_translation_micrometres,
                            ),
                        })
                    })
                    .collect::<Result<Vec<_>, ProjectAuthoringError>>()?,
                pose_correctives
                    .iter()
                    .map(|corrective| {
                        let mut vertex_deltas = Vec::new();
                        for range in &corrective.vertex_delta_ranges {
                            if range.vertex_count == 0 {
                                return Err(ProjectAuthoringError::InvalidValue);
                            }
                            let end = range
                                .first_vertex
                                .checked_add(range.vertex_count)
                                .ok_or(ProjectAuthoringError::InvalidValue)?;
                            vertex_deltas.extend((range.first_vertex..end).map(|vertex_index| {
                                NeutralPoseCorrectiveVertexDeltaV1 {
                                    vertex_index,
                                    delta_micrometres: range.delta_micrometres,
                                }
                            }));
                        }
                        NeutralPoseCorrectiveV1::new(
                            SchemaId::new(&corrective.corrective_id)?,
                            SchemaId::new(&corrective.driver_render_joint_id)?,
                            match corrective.driver_axis {
                                AuthoringPoseCorrectiveDriverAxisV1::X => {
                                    PoseCorrectiveDriverAxisV1::X
                                }
                                AuthoringPoseCorrectiveDriverAxisV1::Y => {
                                    PoseCorrectiveDriverAxisV1::Y
                                }
                                AuthoringPoseCorrectiveDriverAxisV1::Z => {
                                    PoseCorrectiveDriverAxisV1::Z
                                }
                            },
                            corrective.activation_start_delta_micrometres,
                            corrective.activation_full_delta_micrometres,
                            match corrective.lod_class {
                                AuthoringPoseCorrectiveLodClassV1::Essential => {
                                    PoseCorrectiveLodClassV1::Essential
                                }
                                AuthoringPoseCorrectiveLodClassV1::Detail => {
                                    PoseCorrectiveLodClassV1::Detail
                                }
                            },
                            vertex_deltas,
                        )
                        .map_err(ProjectAuthoringError::from)
                    })
                    .collect::<Result<Vec<_>, ProjectAuthoringError>>()?,
                vertices,
            )?;
            profile.validate_against(mesh, skeleton, body_schema_asset)?;
            insert_revision(&mut revisions, profile.asset_revision()?)?;
            records.push(profile.into());
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
