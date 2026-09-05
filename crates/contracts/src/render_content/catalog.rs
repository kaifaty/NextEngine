use crate::canonical::{
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U32, CanonicalCursor,
    CanonicalDecodeLimits, CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::ContentHash;
use crate::project::AssetRevisionRefV1;

use super::codec::{
    decode_asset_revision, domain_hash, encode_asset_revision, ensure_limit, ensure_unique,
    extend_count, extend_sized, field, read_count, read_sized, validate_envelope,
};
use super::profile::{B0_MAX_MESHLET_TRIANGLES, B0_MAX_MESHLET_VERTICES};
use super::{
    B0RenderContentProfileV1, MaterialAlphaModeV1, MaterialColorSpaceV1, MaterialTextureSlotV1,
    MeshPrimitiveTopologyV1, NeutralBaseSkinningProfileV1, NeutralMaterialTextureBindingV1,
    NeutralMaterialV1, NeutralMeshV1, NeutralTexelEncodingV1, NeutralTextureColorSpaceV1,
    NeutralTextureDimensionV1, NeutralTextureV1, RENDER_CONTENT_CATALOG_SCHEMA_ID,
    RENDER_CONTENT_OWNER_ID, RENDER_CONTENT_SCHEMA_VERSION, RENDER_CONTENT_SEGMENT_ID,
    RenderContentContractError,
};

const COOKED_MESH_SCHEMA_ID: &str = "nextengine.render-content.meshlets";
const MAX_CATALOG_ASSETS: usize = 1_048_576;
const MAX_MESHLETS_PER_MESH: usize = 16_777_216;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct B0MeshletV1 {
    unique_vertex_indices: Vec<u32>,
    local_triangle_indices: Vec<u8>,
}

impl B0MeshletV1 {
    pub fn new(
        unique_vertex_indices: Vec<u32>,
        local_triangle_indices: Vec<u8>,
    ) -> Result<Self, RenderContentContractError> {
        ensure_limit(
            unique_vertex_indices.len(),
            B0_MAX_MESHLET_VERTICES as usize,
        )?;
        ensure_limit(
            local_triangle_indices.len() / 3,
            B0_MAX_MESHLET_TRIANGLES as usize,
        )?;
        if unique_vertex_indices.is_empty()
            || local_triangle_indices.is_empty()
            || !local_triangle_indices.len().is_multiple_of(3)
        {
            return Err(RenderContentContractError::InvalidPrimitive);
        }
        let mut sorted_vertices = unique_vertex_indices.clone();
        sorted_vertices.sort_unstable();
        ensure_unique(&sorted_vertices)?;
        let vertex_count = u8::try_from(unique_vertex_indices.len())
            .map_err(|_| RenderContentContractError::IntegerOverflow)?;
        if local_triangle_indices
            .iter()
            .any(|index| *index >= vertex_count)
        {
            return Err(RenderContentContractError::InvalidIndex);
        }
        let mut used = vec![false; unique_vertex_indices.len()];
        for index in &local_triangle_indices {
            used[usize::from(*index)] = true;
        }
        if used.contains(&false) {
            return Err(RenderContentContractError::InvalidPrimitive);
        }
        Ok(Self {
            unique_vertex_indices,
            local_triangle_indices,
        })
    }

    #[must_use]
    pub fn unique_vertex_indices(&self) -> &[u32] {
        &self.unique_vertex_indices
    }

    #[must_use]
    pub fn local_triangle_indices(&self) -> &[u8] {
        &self.local_triangle_indices
    }

    fn encode(&self) -> Result<Vec<u8>, RenderContentContractError> {
        let mut bytes = Vec::new();
        extend_count(&mut bytes, self.unique_vertex_indices.len())?;
        for index in &self.unique_vertex_indices {
            bytes.extend_from_slice(&index.to_le_bytes());
        }
        extend_count(&mut bytes, self.local_triangle_indices.len())?;
        bytes.extend_from_slice(&self.local_triangle_indices);
        Ok(bytes)
    }

    fn decode(
        cursor: &mut CanonicalCursor<'_>,
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RenderContentContractError> {
        let vertex_count = read_count(cursor, limits, B0_MAX_MESHLET_VERTICES as usize)?;
        let mut vertices = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            vertices.push(cursor.read_u32()?);
        }
        let local_index_count = read_count(cursor, limits, B0_MAX_MESHLET_TRIANGLES as usize * 3)?;
        let local_indices = cursor.read_exact(local_index_count)?.to_vec();
        Self::new(vertices, local_indices)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct B0CookedMeshV1 {
    source_revision: AssetRevisionRefV1,
    meshlets: Vec<B0MeshletV1>,
    payload_sha256: ContentHash,
}

impl B0CookedMeshV1 {
    pub fn cook(mesh: &NeutralMeshV1) -> Result<Self, RenderContentContractError> {
        if mesh.texcoords_q16_16().is_empty()
            || mesh.primitives().iter().any(|primitive| {
                primitive.topology() != MeshPrimitiveTopologyV1::Triangles
                    || primitive.material_slot() != 0
            })
        {
            return Err(RenderContentContractError::UnsupportedB0Feature);
        }
        let mut meshlets = Vec::new();
        let mut vertices: Vec<u32> = Vec::new();
        let mut local_indices = Vec::new();
        for primitive in mesh.primitives() {
            let start = primitive.first_index() as usize;
            let end = start
                .checked_add(primitive.index_count() as usize)
                .ok_or(RenderContentContractError::IntegerOverflow)?;
            for triangle in mesh.indices()[start..end].chunks_exact(3) {
                let additional = triangle
                    .iter()
                    .filter(|index| !vertices.contains(index))
                    .count();
                let triangle_count = local_indices.len() / 3;
                if !local_indices.is_empty()
                    && (vertices.len() + additional > B0_MAX_MESHLET_VERTICES as usize
                        || triangle_count >= B0_MAX_MESHLET_TRIANGLES as usize)
                {
                    meshlets.push(B0MeshletV1::new(vertices, local_indices)?);
                    vertices = Vec::new();
                    local_indices = Vec::new();
                }
                for index in triangle {
                    let local = if let Some(position) =
                        vertices.iter().position(|candidate| candidate == index)
                    {
                        position
                    } else {
                        vertices.push(*index);
                        vertices.len() - 1
                    };
                    local_indices.push(
                        u8::try_from(local)
                            .map_err(|_| RenderContentContractError::IntegerOverflow)?,
                    );
                }
            }
        }
        if !local_indices.is_empty() {
            meshlets.push(B0MeshletV1::new(vertices, local_indices)?);
        }
        ensure_limit(meshlets.len(), MAX_MESHLETS_PER_MESH)?;
        let source_revision = mesh.asset_revision()?;
        let mut value = Self {
            source_revision,
            meshlets,
            payload_sha256: ContentHash::default(),
        };
        value.payload_sha256 = domain_hash(
            "nextengine.render-content.meshlets.v1",
            &value.canonical_body_bytes()?,
        )?;
        Ok(value)
    }

    #[must_use]
    pub const fn source_revision(&self) -> AssetRevisionRefV1 {
        self.source_revision
    }

    #[must_use]
    pub fn meshlets(&self) -> &[B0MeshletV1] {
        &self.meshlets
    }

    #[must_use]
    pub const fn payload_sha256(&self) -> ContentHash {
        self.payload_sha256
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RenderContentContractError> {
        Ok(encode_canonical_segment(
            RENDER_CONTENT_OWNER_ID,
            COOKED_MESH_SCHEMA_ID,
            RENDER_CONTENT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    RENDER_CONTENT_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_STRUCT,
                    encode_asset_revision(self.source_revision).to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_SEQUENCE, encode_meshlets(&self.meshlets)?),
                CanonicalField::new(
                    4,
                    crate::canonical::CANONICAL_TYPE_HASH256,
                    self.payload_sha256.as_bytes().to_vec(),
                ),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RenderContentContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_envelope(&segment, COOKED_MESH_SCHEMA_ID, 4)?;
        let source_revision = decode_asset_revision(field(&segment, 2, CANONICAL_TYPE_STRUCT)?)?;
        let meshlets = decode_meshlets(field(&segment, 3, CANONICAL_TYPE_SEQUENCE)?, limits)?;
        let payload_sha256 = ContentHash::from_bytes(
            field(&segment, 4, crate::canonical::CANONICAL_TYPE_HASH256)?
                .try_into()
                .map_err(|_| RenderContentContractError::InvalidPayload)?,
        );
        let value = Self {
            source_revision,
            meshlets,
            payload_sha256,
        };
        let expected = domain_hash(
            "nextengine.render-content.meshlets.v1",
            &value.canonical_body_bytes()?,
        )?;
        if expected != value.payload_sha256 || value.canonical_bytes()? != bytes {
            return Err(RenderContentContractError::HashMismatch);
        }
        Ok(value)
    }

    fn canonical_body_bytes(&self) -> Result<Vec<u8>, RenderContentContractError> {
        let mut bytes = encode_asset_revision(self.source_revision).to_vec();
        bytes.extend_from_slice(&encode_meshlets(&self.meshlets)?);
        Ok(bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderContentCatalogV1 {
    profile: B0RenderContentProfileV1,
    profile_revision: AssetRevisionRefV1,
    meshes: Vec<NeutralMeshV1>,
    mesh_revisions: Vec<AssetRevisionRefV1>,
    cooked_meshes: Vec<B0CookedMeshV1>,
    materials: Vec<NeutralMaterialV1>,
    material_revisions: Vec<AssetRevisionRefV1>,
    textures: Vec<NeutralTextureV1>,
    texture_revisions: Vec<AssetRevisionRefV1>,
    base_skinning_profiles: Vec<NeutralBaseSkinningProfileV1>,
    base_skinning_profile_revisions: Vec<AssetRevisionRefV1>,
    catalog_sha256: ContentHash,
}

impl RenderContentCatalogV1 {
    pub fn new(
        profile: B0RenderContentProfileV1,
        meshes: Vec<NeutralMeshV1>,
        materials: Vec<NeutralMaterialV1>,
        textures: Vec<NeutralTextureV1>,
        base_skinning_profiles: Vec<NeutralBaseSkinningProfileV1>,
    ) -> Result<Self, RenderContentContractError> {
        ensure_limit(meshes.len(), MAX_CATALOG_ASSETS)?;
        ensure_limit(materials.len(), MAX_CATALOG_ASSETS)?;
        ensure_limit(textures.len(), MAX_CATALOG_ASSETS)?;
        ensure_limit(base_skinning_profiles.len(), MAX_CATALOG_ASSETS)?;
        let profile_revision = profile.asset_revision()?;
        let (mesh_revisions, meshes) = sort_records(meshes, NeutralMeshV1::asset_revision)?;
        let (material_revisions, materials) =
            sort_records(materials, NeutralMaterialV1::asset_revision)?;
        let (texture_revisions, textures) =
            sort_records(textures, NeutralTextureV1::asset_revision)?;
        let (base_skinning_profile_revisions, base_skinning_profiles) = sort_records(
            base_skinning_profiles,
            NeutralBaseSkinningProfileV1::asset_revision,
        )?;
        ensure_unique_asset_ids(
            profile_revision,
            &mesh_revisions,
            &material_revisions,
            &texture_revisions,
            &base_skinning_profile_revisions,
        )?;
        let cooked_meshes = meshes
            .iter()
            .map(B0CookedMeshV1::cook)
            .collect::<Result<Vec<_>, _>>()?;
        let mut value = Self {
            profile,
            profile_revision,
            meshes,
            mesh_revisions,
            cooked_meshes,
            materials,
            material_revisions,
            textures,
            texture_revisions,
            base_skinning_profiles,
            base_skinning_profile_revisions,
            catalog_sha256: ContentHash::default(),
        };
        value.validate_b0()?;
        value.catalog_sha256 = domain_hash(
            "nextengine.render-content.catalog.v1",
            &value.canonical_bytes_without_hash()?,
        )?;
        Ok(value)
    }

    #[must_use]
    pub const fn profile(&self) -> &B0RenderContentProfileV1 {
        &self.profile
    }

    #[must_use]
    pub const fn profile_revision(&self) -> AssetRevisionRefV1 {
        self.profile_revision
    }

    #[must_use]
    pub fn meshes(&self) -> &[NeutralMeshV1] {
        &self.meshes
    }

    #[must_use]
    pub fn materials(&self) -> &[NeutralMaterialV1] {
        &self.materials
    }

    #[must_use]
    pub fn textures(&self) -> &[NeutralTextureV1] {
        &self.textures
    }

    #[must_use]
    pub fn base_skinning_profiles(&self) -> &[NeutralBaseSkinningProfileV1] {
        &self.base_skinning_profiles
    }

    #[must_use]
    pub fn cooked_meshes(&self) -> &[B0CookedMeshV1] {
        &self.cooked_meshes
    }

    #[must_use]
    pub const fn catalog_sha256(&self) -> ContentHash {
        self.catalog_sha256
    }

    #[must_use]
    pub fn mesh(&self, revision: AssetRevisionRefV1) -> Option<&NeutralMeshV1> {
        exact_lookup(&self.mesh_revisions, &self.meshes, revision)
    }

    #[must_use]
    pub fn material(&self, revision: AssetRevisionRefV1) -> Option<&NeutralMaterialV1> {
        exact_lookup(&self.material_revisions, &self.materials, revision)
    }

    #[must_use]
    pub fn texture(&self, revision: AssetRevisionRefV1) -> Option<&NeutralTextureV1> {
        exact_lookup(&self.texture_revisions, &self.textures, revision)
    }

    #[must_use]
    pub fn base_skinning_profile(
        &self,
        revision: AssetRevisionRefV1,
    ) -> Option<&NeutralBaseSkinningProfileV1> {
        exact_lookup(
            &self.base_skinning_profile_revisions,
            &self.base_skinning_profiles,
            revision,
        )
    }

    #[must_use]
    pub fn base_skinning_profile_for_mesh(
        &self,
        revision: AssetRevisionRefV1,
    ) -> Option<&NeutralBaseSkinningProfileV1> {
        self.base_skinning_profiles
            .iter()
            .find(|profile| profile.mesh_revision() == revision)
    }

    #[must_use]
    pub fn cooked_mesh(&self, revision: AssetRevisionRefV1) -> Option<&B0CookedMeshV1> {
        self.cooked_meshes
            .binary_search_by_key(&revision, B0CookedMeshV1::source_revision)
            .ok()
            .map(|index| &self.cooked_meshes[index])
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RenderContentContractError> {
        self.canonical_bytes_without_hash()
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RenderContentContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_envelope(&segment, RENDER_CONTENT_CATALOG_SCHEMA_ID, 7)?;
        let profile = B0RenderContentProfileV1::from_canonical_bytes(
            field(&segment, 2, CANONICAL_TYPE_STRUCT)?,
            limits,
        )?;
        let meshes = decode_records(
            field(&segment, 3, CANONICAL_TYPE_SEQUENCE)?,
            limits,
            NeutralMeshV1::from_canonical_bytes,
        )?;
        let materials = decode_records(
            field(&segment, 4, CANONICAL_TYPE_SEQUENCE)?,
            limits,
            NeutralMaterialV1::from_canonical_bytes,
        )?;
        let textures = decode_records(
            field(&segment, 5, CANONICAL_TYPE_SEQUENCE)?,
            limits,
            NeutralTextureV1::from_canonical_bytes,
        )?;
        let cooked = decode_records(
            field(&segment, 6, CANONICAL_TYPE_SEQUENCE)?,
            limits,
            B0CookedMeshV1::from_canonical_bytes,
        )?;
        let base_skinning_profiles = decode_records(
            field(&segment, 7, CANONICAL_TYPE_SEQUENCE)?,
            limits,
            NeutralBaseSkinningProfileV1::from_canonical_bytes,
        )?;
        let value = Self::new(profile, meshes, materials, textures, base_skinning_profiles)?;
        if value.cooked_meshes != cooked || value.canonical_bytes()? != bytes {
            return Err(RenderContentContractError::NonCanonical);
        }
        Ok(value)
    }

    fn canonical_bytes_without_hash(&self) -> Result<Vec<u8>, RenderContentContractError> {
        Ok(encode_canonical_segment(
            RENDER_CONTENT_OWNER_ID,
            RENDER_CONTENT_CATALOG_SCHEMA_ID,
            RENDER_CONTENT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    RENDER_CONTENT_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_STRUCT, self.profile.canonical_bytes()?),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_records(&self.meshes, NeutralMeshV1::canonical_bytes)?,
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_records(&self.materials, NeutralMaterialV1::canonical_bytes)?,
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_records(&self.textures, NeutralTextureV1::canonical_bytes)?,
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_records(&self.cooked_meshes, B0CookedMeshV1::canonical_bytes)?,
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_records(
                        &self.base_skinning_profiles,
                        NeutralBaseSkinningProfileV1::canonical_bytes,
                    )?,
                ),
            ],
        )?)
    }

    /// Scene look L5 (plan `look/05`): one to three bindings, `BaseColor`
    /// at uv set 0 first, then at most one `MetallicRoughness` and one
    /// `Normal` at uv set 0 over linear textures, every binding's UV
    /// transform the same uniform scale.
    fn validate_b0_bindings(
        &self,
        bindings: &[NeutralMaterialTextureBindingV1],
    ) -> Result<(), RenderContentContractError> {
        let Some(first) = bindings.first() else {
            return Err(RenderContentContractError::UnsupportedB0Feature);
        };
        if bindings.len() > 3
            || first.slot() != MaterialTextureSlotV1::BaseColor
            || first.uv_set() != 0
        {
            return Err(RenderContentContractError::UnsupportedB0Feature);
        }
        let scale = first
            .uv_transform()
            .uniform_scale_q16_16()
            .ok_or(RenderContentContractError::UnsupportedB0Feature)?;
        let mut seen_metallic_roughness = false;
        let mut seen_normal = false;
        for binding in bindings {
            let texture = self
                .texture(binding.texture())
                .ok_or(RenderContentContractError::MissingReference)?;
            if binding.uv_set() != 0 || binding.uv_transform().uniform_scale_q16_16() != Some(scale)
            {
                return Err(RenderContentContractError::UnsupportedB0Feature);
            }
            match binding.slot() {
                MaterialTextureSlotV1::BaseColor if std::ptr::eq(binding, first) => {}
                MaterialTextureSlotV1::MetallicRoughness if !seen_metallic_roughness => {
                    seen_metallic_roughness = true;
                    if texture.color_space() != NeutralTextureColorSpaceV1::Linear {
                        return Err(RenderContentContractError::UnsupportedB0Feature);
                    }
                }
                MaterialTextureSlotV1::Normal if !seen_normal => {
                    seen_normal = true;
                    if texture.color_space() != NeutralTextureColorSpaceV1::Linear {
                        return Err(RenderContentContractError::UnsupportedB0Feature);
                    }
                }
                _ => return Err(RenderContentContractError::UnsupportedB0Feature),
            }
        }
        Ok(())
    }

    fn validate_b0(&self) -> Result<(), RenderContentContractError> {
        let fallback_material = self
            .material(self.profile.fallback_material())
            .ok_or(RenderContentContractError::MissingReference)?;
        self.texture(self.profile.fallback_texture())
            .ok_or(RenderContentContractError::MissingReference)?;
        for material in &self.materials {
            if material.alpha_mode() != MaterialAlphaModeV1::Opaque
                || material.base_color_space() != MaterialColorSpaceV1::Linear
                // Scene look L1 (plan `look/01`): the B0 shading reads
                // metallic and roughness, so the profile admits any value.
                || material.emissive_rgb_unorm16() != [0; 3]
                || material.emissive_color_space() != MaterialColorSpaceV1::Linear
                || material.emissive_intensity_q16_16() != 0
                || material.normal_scale_q16_16() != 65_536
                || material.occlusion_strength_unorm16() != u16::MAX
                || material.alpha_cutoff_unorm16() != 0
                || material.double_sided()
                || !material.feature_tags().is_empty()
            {
                return Err(RenderContentContractError::UnsupportedB0Feature);
            }
            self.validate_b0_bindings(material.texture_bindings())?;
        }
        if fallback_material.texture_bindings()[0].texture() != self.profile.fallback_texture() {
            return Err(RenderContentContractError::MissingReference);
        }
        // Scene look L5 (plan `look/05`): 2D single-layer 8-bit textures in
        // sRGB or linear space with any mip chain the record admits.
        for texture in &self.textures {
            if texture.dimension() != NeutralTextureDimensionV1::D2
                || texture.array_layers() != 1
                || !matches!(
                    texture.color_space(),
                    NeutralTextureColorSpaceV1::Srgb | NeutralTextureColorSpaceV1::Linear
                )
                || !matches!(
                    texture.texel_encoding(),
                    NeutralTexelEncodingV1::Rgba8Unorm
                        | NeutralTexelEncodingV1::Rg8Unorm
                        | NeutralTexelEncodingV1::R8Unorm
                )
            {
                return Err(RenderContentContractError::UnsupportedB0Feature);
            }
        }
        let mut mesh_revisions = Vec::with_capacity(self.base_skinning_profiles.len());
        for skinning in &self.base_skinning_profiles {
            let mesh = self
                .mesh(skinning.mesh_revision())
                .ok_or(RenderContentContractError::MissingReference)?;
            if mesh.positions_micrometres().len() != skinning.vertices().len() {
                return Err(RenderContentContractError::InvalidSkinningProfile);
            }
            mesh_revisions.push(skinning.mesh_revision());
        }
        mesh_revisions.sort();
        ensure_unique(&mesh_revisions)?;
        Ok(())
    }
}

fn sort_records<T>(
    records: Vec<T>,
    revision: impl Fn(&T) -> Result<AssetRevisionRefV1, RenderContentContractError>,
) -> Result<(Vec<AssetRevisionRefV1>, Vec<T>), RenderContentContractError> {
    let mut keyed = records
        .into_iter()
        .map(|record| Ok((revision(&record)?, record)))
        .collect::<Result<Vec<_>, RenderContentContractError>>()?;
    keyed.sort_by_key(|(reference, _)| *reference);
    if keyed
        .windows(2)
        .any(|pair| pair[0].0.asset_id == pair[1].0.asset_id)
    {
        return Err(RenderContentContractError::DuplicateIdentity);
    }
    let (revisions, records) = keyed.into_iter().unzip();
    Ok((revisions, records))
}

fn ensure_unique_asset_ids(
    profile: AssetRevisionRefV1,
    meshes: &[AssetRevisionRefV1],
    materials: &[AssetRevisionRefV1],
    textures: &[AssetRevisionRefV1],
    base_skinning_profiles: &[AssetRevisionRefV1],
) -> Result<(), RenderContentContractError> {
    let mut ids = Vec::with_capacity(
        1 + meshes.len() + materials.len() + textures.len() + base_skinning_profiles.len(),
    );
    ids.push(profile.asset_id);
    ids.extend(meshes.iter().map(|value| value.asset_id));
    ids.extend(materials.iter().map(|value| value.asset_id));
    ids.extend(textures.iter().map(|value| value.asset_id));
    ids.extend(base_skinning_profiles.iter().map(|value| value.asset_id));
    ids.sort();
    ensure_unique(&ids)
}

fn exact_lookup<'a, T>(
    revisions: &[AssetRevisionRefV1],
    records: &'a [T],
    revision: AssetRevisionRefV1,
) -> Option<&'a T> {
    revisions
        .binary_search(&revision)
        .ok()
        .map(|index| &records[index])
}

fn encode_meshlets(meshlets: &[B0MeshletV1]) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    extend_count(&mut bytes, meshlets.len())?;
    for meshlet in meshlets {
        extend_sized(&mut bytes, &meshlet.encode()?)?;
    }
    Ok(bytes)
}

fn decode_meshlets(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<B0MeshletV1>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, MAX_MESHLETS_PER_MESH)?;
    let mut meshlets = Vec::with_capacity(count);
    for _ in 0..count {
        let bytes = read_sized(&mut cursor, limits)?;
        let mut meshlet_cursor = CanonicalCursor::new(bytes);
        meshlets.push(B0MeshletV1::decode(&mut meshlet_cursor, limits)?);
        meshlet_cursor.finish()?;
    }
    cursor.finish()?;
    Ok(meshlets)
}

fn encode_records<T>(
    values: &[T],
    encode: impl Fn(&T) -> Result<Vec<u8>, RenderContentContractError>,
) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    extend_count(&mut bytes, values.len())?;
    for value in values {
        extend_sized(&mut bytes, &encode(value)?)?;
    }
    Ok(bytes)
}

fn decode_records<T>(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    decode: impl Fn(&[u8], CanonicalDecodeLimits) -> Result<T, RenderContentContractError>,
) -> Result<Vec<T>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, MAX_CATALOG_ASSETS)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(decode(read_sized(&mut cursor, limits)?, limits)?);
    }
    cursor.finish()?;
    Ok(values)
}
