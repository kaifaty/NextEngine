use crate::animation_content::NeutralTransformV1;
use crate::canonical::{CanonicalDecodeLimits, sha256};
use crate::content::{NeutralRecordKindV1, NeutralRecordV1};
use crate::ids::{AssetId, ContentHash, PersistentId, SchemaId, content_hash_from_bytes};
use crate::project::{
    AssetRevisionRefV1, SchemaEncodingV1, SchemaRefV1, SchemaRoleV1, domain_hash,
};

use super::{
    AabbI64V1, B0CookedMeshV1, B0RenderContentProfileV1, BaseSkinningFallbackV1,
    BaseSkinningMethodV1, MaterialAlphaModeV1, MaterialColorSpaceV1, MaterialTextureSlotV1,
    MeshPrimitiveTopologyV1, NeutralBaseSkinningProfileV1, NeutralMaterialTextureBindingV1,
    NeutralMaterialV1, NeutralMeshPrimitiveV1, NeutralMeshV1, NeutralPoseCorrectiveV1,
    NeutralPoseCorrectiveVertexDeltaV1, NeutralRenderJointV1, NeutralRenderRecordV1,
    NeutralSkinInfluenceV1, NeutralSkinVertexV1, NeutralTangentV1, NeutralTexelEncodingV1,
    NeutralTextureAlphaSemanticsV1, NeutralTextureColorSpaceV1, NeutralTextureDimensionV1,
    NeutralTextureMipLevelV1, NeutralTextureV1, PoseCorrectiveDriverAxisV1,
    PoseCorrectiveLodClassV1, RenderContentCatalogV1, RenderContentContractError, UvTransformV1,
    b0_shader_interface_manifest_sha256,
};
use super::{
    B0_RENDER_CONTENT_PROFILE_SCHEMA_ID, NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID,
    NEUTRAL_MATERIAL_SCHEMA_ID, NEUTRAL_MESH_SCHEMA_ID, NEUTRAL_TEXTURE_SCHEMA_ID,
};

#[test]
fn typed_records_and_catalog_round_trip_canonically() {
    let (profile, mesh, material, texture) = fixture();
    for record in [
        NeutralRenderRecordV1::Mesh(mesh.clone()),
        NeutralRenderRecordV1::Material(material.clone()),
        NeutralRenderRecordV1::Texture(texture.clone()),
        NeutralRenderRecordV1::Profile(profile.clone()),
    ] {
        let bytes = record.canonical_bytes().expect("record encodes");
        let decoded =
            NeutralRenderRecordV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("record decodes");
        assert_eq!(decoded, record);
        assert_eq!(decoded.canonical_bytes().expect("record re-encodes"), bytes);
        assert_ne!(
            decoded.record_sha256().expect("record hash"),
            content_hash_from_bytes(sha256(&bytes))
        );
    }

    let catalog = RenderContentCatalogV1::new(
        profile,
        vec![mesh],
        vec![material],
        vec![texture],
        Vec::new(),
    )
    .expect("catalog validates");
    let bytes = catalog.canonical_bytes().expect("catalog encodes");
    let decoded =
        RenderContentCatalogV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("catalog decodes");

    assert_eq!(decoded, catalog);
    assert_eq!(
        decoded.canonical_bytes().expect("catalog re-encodes"),
        bytes
    );
    assert_eq!(decoded.cooked_meshes().len(), 1);
    assert_eq!(decoded.cooked_meshes()[0].meshlets().len(), 1);
    assert_eq!(
        decoded.cooked_meshes()[0].meshlets()[0].local_triangle_indices(),
        &[0, 1, 2]
    );
}

#[test]
fn catalog_sorting_and_exact_lookup_are_order_independent() {
    let (profile, first_mesh, material, texture) = fixture();
    let second_mesh = mesh(
        5,
        vec![[0, 0, 0], [0, 1_000_000, 0], [1_000_000, 0, 0]],
        vec![0, 2, 1],
    );
    let first = RenderContentCatalogV1::new(
        profile.clone(),
        vec![second_mesh.clone(), first_mesh.clone()],
        vec![material.clone()],
        vec![texture.clone()],
        Vec::new(),
    )
    .expect("first catalog");
    let second = RenderContentCatalogV1::new(
        profile,
        vec![first_mesh.clone(), second_mesh],
        vec![material],
        vec![texture],
        Vec::new(),
    )
    .expect("second catalog");

    assert_eq!(first.catalog_sha256(), second.catalog_sha256());
    assert_eq!(
        first.canonical_bytes().expect("first bytes"),
        second.canonical_bytes().expect("second bytes")
    );
    let revision = first_mesh.asset_revision().expect("mesh revision");
    assert_eq!(first.mesh(revision), Some(&first_mesh));
    assert!(first.cooked_mesh(revision).is_some());
    let stale = AssetRevisionRefV1 {
        record_sha256: hash(99),
        ..revision
    };
    assert!(first.mesh(stale).is_none());
}

#[test]
fn base_skinning_profile_round_trips_and_rejects_incomplete_weights() {
    let (render_profile, mesh, material, texture) = fixture();
    let mesh_revision = mesh.asset_revision().expect("mesh revision");
    let joint_id = SchemaId::new("test.render-joint.root").expect("joint id");
    let joint = NeutralRenderJointV1 {
        render_joint_id: joint_id.clone(),
        parent_render_joint_id: None,
        animation_joint_id: SchemaId::new("test.animation-joint.root").expect("animation joint"),
        body_semantic_id: SchemaId::new("test.body.root").expect("body semantic"),
        bind_transform: NeutralTransformV1::translated([0, 0, 0]),
    };
    let vertex = NeutralSkinVertexV1::new(vec![NeutralSkinInfluenceV1 {
        render_joint_id: joint_id.clone(),
        weight_unorm16: u16::MAX,
    }])
    .expect("complete weight");
    let corrective = NeutralPoseCorrectiveV1::new(
        SchemaId::new("test.corrective.root-x").expect("corrective id"),
        joint_id.clone(),
        PoseCorrectiveDriverAxisV1::X,
        0,
        1,
        PoseCorrectiveLodClassV1::Essential,
        vec![NeutralPoseCorrectiveVertexDeltaV1 {
            vertex_index: 0,
            delta_micrometres: [1, 0, 0],
        }],
    )
    .expect("corrective");
    let profile = NeutralBaseSkinningProfileV1::new(
        schema_ref(NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID),
        asset(9),
        1,
        mesh_revision,
        AssetRevisionRefV1 {
            asset_id: asset(10),
            record_sha256: hash(10),
        },
        AssetRevisionRefV1 {
            asset_id: asset(11),
            record_sha256: hash(11),
        },
        [0, 0, 0],
        BaseSkinningMethodV1::LinearBlend,
        BaseSkinningFallbackV1::BindPose,
        2,
        vec![joint],
        vec![corrective.clone()],
        vec![vertex; mesh.positions_micrometres().len()],
    )
    .expect("base skinning profile");
    let record = NeutralRenderRecordV1::BaseSkinningProfile(profile.clone());
    let bytes = record.canonical_bytes().expect("profile encodes");
    assert_eq!(
        NeutralRenderRecordV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("profile decodes"),
        record
    );
    let catalog = RenderContentCatalogV1::new(
        render_profile,
        vec![mesh],
        vec![material],
        vec![texture],
        vec![profile.clone()],
    )
    .expect("profile closes over exact mesh");
    assert_eq!(
        catalog.base_skinning_profile(profile.asset_revision().expect("profile revision")),
        Some(&profile)
    );
    assert_eq!(
        profile.pose_correctives(),
        std::slice::from_ref(&corrective)
    );
    assert_eq!(
        NeutralPoseCorrectiveV1::new(
            SchemaId::new("test.corrective.duplicate-vertex").expect("corrective id"),
            joint_id.clone(),
            PoseCorrectiveDriverAxisV1::X,
            0,
            1,
            PoseCorrectiveLodClassV1::Essential,
            vec![
                NeutralPoseCorrectiveVertexDeltaV1 {
                    vertex_index: 0,
                    delta_micrometres: [1, 0, 0],
                },
                NeutralPoseCorrectiveVertexDeltaV1 {
                    vertex_index: 0,
                    delta_micrometres: [0, 1, 0],
                },
            ],
        ),
        Err(RenderContentContractError::InvalidPoseCorrective)
    );
    assert_eq!(
        NeutralPoseCorrectiveV1::new(
            SchemaId::new("test.corrective.zero-interval").expect("corrective id"),
            joint_id.clone(),
            PoseCorrectiveDriverAxisV1::X,
            1,
            1,
            PoseCorrectiveLodClassV1::Essential,
            vec![NeutralPoseCorrectiveVertexDeltaV1 {
                vertex_index: 0,
                delta_micrometres: [1, 0, 0],
            }],
        ),
        Err(RenderContentContractError::InvalidPoseCorrective)
    );
    assert_eq!(
        NeutralBaseSkinningProfileV1::new(
            profile.schema_ref().clone(),
            profile.asset_id(),
            profile.record_revision(),
            profile.mesh_revision(),
            profile.skeleton_revision(),
            profile.body_schema_revision(),
            profile.mesh_origin_in_skeleton_micrometres(),
            profile.method(),
            profile.fallback(),
            profile.max_instances_per_frame(),
            profile.render_joints().to_vec(),
            vec![corrective.clone(), corrective],
            profile.vertices().to_vec(),
        ),
        Err(RenderContentContractError::InvalidPoseCorrective)
    );
    assert_eq!(
        NeutralSkinVertexV1::new(vec![NeutralSkinInfluenceV1 {
            render_joint_id: joint_id,
            weight_unorm16: u16::MAX - 1,
        }]),
        Err(RenderContentContractError::InvalidSkinWeights)
    );
}

#[test]
fn strict_mesh_validation_rejects_bad_bounds_streams_indices_and_tangents() {
    assert_eq!(
        AabbI64V1::new([0, 0, 0], [0, 1, 1]),
        Err(RenderContentContractError::InvalidBounds)
    );
    assert_eq!(
        NeutralTangentV1::new([32_767, 0, 0], 0),
        Err(RenderContentContractError::InvalidTangent)
    );
    let error = NeutralMeshV1::new(
        schema_ref(NEUTRAL_MESH_SCHEMA_ID),
        asset(3),
        1,
        AabbI64V1::new([0, 0, 0], [2, 2, 2]).expect("bounds"),
        vec![[0, 0, 0], [1, 0, 0], [0, 1, 0]],
        Some(vec![[0, 0, 0]; 3]),
        None,
        vec![vec![[0, 0]; 3]],
        vec![0, 1, 3],
        vec![
            NeutralMeshPrimitiveV1::new(MeshPrimitiveTopologyV1::Triangles, 0, 3, 0)
                .expect("primitive"),
        ],
    )
    .expect_err("invalid normal is rejected before index validation");
    assert_eq!(error, RenderContentContractError::InvalidNormal);

    let error = NeutralMeshV1::new(
        schema_ref(NEUTRAL_MESH_SCHEMA_ID),
        asset(3),
        1,
        AabbI64V1::new([0, 0, 0], [2, 2, 2]).expect("bounds"),
        vec![[0, 0, 0], [1, 0, 0], [0, 1, 0]],
        None,
        None,
        vec![vec![[0, 0]; 2]],
        vec![0, 1, 2],
        vec![
            NeutralMeshPrimitiveV1::new(MeshPrimitiveTopologyV1::Triangles, 0, 3, 0)
                .expect("primitive"),
        ],
    )
    .expect_err("short UV stream is rejected");
    assert_eq!(error, RenderContentContractError::AttributeLengthMismatch);
}

#[test]
fn texture_validation_rejects_wrong_byte_length_and_noncanonical_half_float() {
    let wrong_length = NeutralTextureV1::new(
        schema_ref(NEUTRAL_TEXTURE_SCHEMA_ID),
        asset(1),
        1,
        NeutralTextureDimensionV1::D2,
        [2, 2, 1],
        1,
        NeutralTextureColorSpaceV1::Srgb,
        NeutralTextureAlphaSemanticsV1::Straight,
        NeutralTexelEncodingV1::Rgba8Unorm,
        vec![NeutralTextureMipLevelV1::new([2, 2, 1], vec![0; 15])],
    )
    .expect_err("wrong byte length");
    assert_eq!(wrong_length, RenderContentContractError::InvalidTextureData);

    let half_nan = NeutralTextureV1::new(
        schema_ref(NEUTRAL_TEXTURE_SCHEMA_ID),
        asset(1),
        1,
        NeutralTextureDimensionV1::D2,
        [1, 1, 1],
        1,
        NeutralTextureColorSpaceV1::Linear,
        NeutralTextureAlphaSemanticsV1::Straight,
        NeutralTexelEncodingV1::Rgba16FloatCanonical,
        vec![NeutralTextureMipLevelV1::new(
            [1, 1, 1],
            [0x00, 0x7c, 0, 0, 0, 0, 0, 0].to_vec(),
        )],
    )
    .expect_err("infinite binary16 is rejected");
    assert_eq!(half_nan, RenderContentContractError::InvalidHalfFloat);
}

#[test]
fn texture_and_material_alpha_values_require_one_canonical_representation() {
    for (semantics, texels) in [
        (NeutralTextureAlphaSemanticsV1::Opaque, vec![1, 2, 3, 254]),
        (NeutralTextureAlphaSemanticsV1::Straight, vec![1, 2, 3, 0]),
    ] {
        assert_eq!(
            NeutralTextureV1::new(
                schema_ref(NEUTRAL_TEXTURE_SCHEMA_ID),
                asset(1),
                1,
                NeutralTextureDimensionV1::D2,
                [1, 1, 1],
                1,
                NeutralTextureColorSpaceV1::Srgb,
                semantics,
                NeutralTexelEncodingV1::Rgba8Unorm,
                vec![NeutralTextureMipLevelV1::new([1, 1, 1], texels)],
            ),
            Err(RenderContentContractError::InvalidTextureData)
        );
    }

    let texture_ref = texture(1).asset_revision().expect("texture ref");
    assert_eq!(
        NeutralMaterialV1::new(
            schema_ref(NEUTRAL_MATERIAL_SCHEMA_ID),
            asset(2),
            1,
            [1, 2, 3, 0],
            MaterialColorSpaceV1::Linear,
            0,
            u16::MAX,
            [0; 3],
            MaterialColorSpaceV1::Linear,
            0,
            65_536,
            u16::MAX,
            MaterialAlphaModeV1::Blend,
            0,
            false,
            vec![
                NeutralMaterialTextureBindingV1::new(
                    MaterialTextureSlotV1::BaseColor,
                    texture_ref,
                    0,
                    UvTransformV1::identity(),
                )
                .expect("binding"),
            ],
            Vec::new(),
        ),
        Err(RenderContentContractError::InvalidMaterial)
    );
}

#[test]
fn catalog_requires_exact_declared_fallback_and_b0_material_profile() {
    let (profile, mesh, material, texture) = fixture();
    let missing_texture = RenderContentCatalogV1::new(
        profile.clone(),
        vec![mesh.clone()],
        vec![material],
        Vec::new(),
        Vec::new(),
    )
    .expect_err("fallback texture must be present");
    assert_eq!(
        missing_texture,
        RenderContentContractError::MissingReference
    );

    let texture_ref = texture.asset_revision().expect("texture ref");
    let blend_material = NeutralMaterialV1::new(
        schema_ref(NEUTRAL_MATERIAL_SCHEMA_ID),
        asset(2),
        1,
        [u16::MAX; 4],
        MaterialColorSpaceV1::Linear,
        0,
        u16::MAX,
        [0; 3],
        MaterialColorSpaceV1::Linear,
        0,
        65_536,
        u16::MAX,
        MaterialAlphaModeV1::Blend,
        0,
        false,
        vec![
            NeutralMaterialTextureBindingV1::new(
                MaterialTextureSlotV1::BaseColor,
                texture_ref,
                0,
                UvTransformV1::identity(),
            )
            .expect("binding"),
        ],
        Vec::new(),
    )
    .expect("neutral blend is valid");
    let blend_profile = B0RenderContentProfileV1::new(
        schema_ref(B0_RENDER_CONTENT_PROFILE_SCHEMA_ID),
        asset(4),
        1,
        b0_shader_interface_manifest_sha256(),
        blend_material.asset_revision().expect("material ref"),
        texture_ref,
    )
    .expect("profile");
    let unsupported = RenderContentCatalogV1::new(
        blend_profile,
        vec![mesh],
        vec![blend_material],
        vec![texture],
        Vec::new(),
    )
    .expect_err("blend is outside B0");
    assert_eq!(
        unsupported,
        RenderContentContractError::UnsupportedB0Feature
    );
}

#[test]
fn b0_admission_rejects_semantics_the_backend_does_not_represent() {
    let texture = texture(1);
    let texture_ref = texture.asset_revision().expect("texture ref");
    let unsupported_materials = [
        b0_material(
            texture_ref,
            B0MaterialOverrides {
                base_color_space: MaterialColorSpaceV1::Srgb,
                ..B0MaterialOverrides::default()
            },
        ),
        b0_material(
            texture_ref,
            B0MaterialOverrides {
                double_sided: true,
                ..B0MaterialOverrides::default()
            },
        ),
        b0_material(
            texture_ref,
            B0MaterialOverrides {
                uv_set: 1,
                ..B0MaterialOverrides::default()
            },
        ),
        b0_material(
            texture_ref,
            B0MaterialOverrides {
                uv_transform: UvTransformV1::new([65_536, 0, 1, 0, 65_536, 0])
                    .expect("bounded transform"),
                ..B0MaterialOverrides::default()
            },
        ),
    ];
    for material in unsupported_materials {
        let profile = B0RenderContentProfileV1::new(
            schema_ref(B0_RENDER_CONTENT_PROFILE_SCHEMA_ID),
            asset(4),
            1,
            b0_shader_interface_manifest_sha256(),
            material.asset_revision().expect("material ref"),
            texture_ref,
        )
        .expect("profile");
        assert_eq!(
            RenderContentCatalogV1::new(
                profile,
                vec![mesh(
                    3,
                    vec![[0, 0, 0], [1_000_000, 0, 0], [0, 1_000_000, 0]],
                    vec![0, 1, 2],
                )],
                vec![material],
                vec![texture.clone()],
                Vec::new(),
            ),
            Err(RenderContentContractError::UnsupportedB0Feature)
        );
    }

    let mesh_with_material_slot = NeutralMeshV1::new(
        schema_ref(NEUTRAL_MESH_SCHEMA_ID),
        asset(3),
        1,
        AabbI64V1::new([-1; 3], [1_000_001; 3]).expect("bounds"),
        vec![[0, 0, 0], [1_000_000, 0, 0], [0, 1_000_000, 0]],
        None,
        None,
        vec![vec![[0, 0], [65_536, 0], [0, 65_536]]],
        vec![0, 1, 2],
        vec![
            NeutralMeshPrimitiveV1::new(MeshPrimitiveTopologyV1::Triangles, 0, 3, 1)
                .expect("primitive"),
        ],
    )
    .expect("neutral mesh");
    assert_eq!(
        B0CookedMeshV1::cook(&mesh_with_material_slot),
        Err(RenderContentContractError::UnsupportedB0Feature)
    );
}

#[test]
fn typed_schema_dispatch_and_legacy_record_encoding_remain_separate() {
    assert!(NeutralRenderRecordV1::supports_schema_id(
        &SchemaId::new(NEUTRAL_MESH_SCHEMA_ID).expect("schema")
    ));
    assert!(!NeutralRenderRecordV1::supports_schema_id(
        &SchemaId::new("nextengine.content.scene.v1").expect("schema")
    ));

    let legacy = NeutralRecordV1::new(
        SchemaRefV1 {
            schema_id: SchemaId::new(NeutralRecordKindV1::Scene.schema_id())
                .expect("legacy schema"),
            schema_version: 1,
            descriptor_sha256: hash(42),
            role: SchemaRoleV1::Definition,
            encoding: SchemaEncodingV1::CanonicalBinaryV1,
        },
        asset(8),
        NeutralRecordKindV1::Scene,
        PersistentId::from_bytes([9; 16]),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .expect("legacy record");
    let bytes = legacy.canonical_bytes().expect("legacy bytes");
    let decoded = NeutralRecordV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
        .expect("legacy record still decodes");
    assert_eq!(decoded.canonical_bytes().expect("legacy re-encode"), bytes);
}

#[test]
fn b0_shader_interface_hash_is_a_stable_neutral_golden_vector() {
    assert_eq!(
        b0_shader_interface_manifest_sha256().to_hex(),
        "860a946d909b88edca130dcaf9c6378916f3c6da7aab01d5b46a92768c1a649a"
    );
}

fn fixture() -> (
    B0RenderContentProfileV1,
    NeutralMeshV1,
    NeutralMaterialV1,
    NeutralTextureV1,
) {
    let texture = texture(1);
    let texture_ref = texture.asset_revision().expect("texture ref");
    let material = NeutralMaterialV1::new(
        schema_ref(NEUTRAL_MATERIAL_SCHEMA_ID),
        asset(2),
        1,
        [u16::MAX; 4],
        MaterialColorSpaceV1::Linear,
        0,
        u16::MAX,
        [0; 3],
        MaterialColorSpaceV1::Linear,
        0,
        65_536,
        u16::MAX,
        MaterialAlphaModeV1::Opaque,
        0,
        false,
        vec![
            NeutralMaterialTextureBindingV1::new(
                MaterialTextureSlotV1::BaseColor,
                texture_ref,
                0,
                UvTransformV1::identity(),
            )
            .expect("binding"),
        ],
        Vec::new(),
    )
    .expect("material");
    let profile = B0RenderContentProfileV1::new(
        schema_ref(B0_RENDER_CONTENT_PROFILE_SCHEMA_ID),
        asset(4),
        1,
        b0_shader_interface_manifest_sha256(),
        material.asset_revision().expect("material ref"),
        texture_ref,
    )
    .expect("profile");
    (
        profile,
        mesh(
            3,
            vec![[0, 0, 0], [1_000_000, 0, 0], [0, 1_000_000, 0]],
            vec![0, 1, 2],
        ),
        material,
        texture,
    )
}

fn mesh(id: u8, positions: Vec<[i64; 3]>, indices: Vec<u32>) -> NeutralMeshV1 {
    NeutralMeshV1::new(
        schema_ref(NEUTRAL_MESH_SCHEMA_ID),
        asset(id),
        1,
        AabbI64V1::new([-1, -1, -1], [1_000_001, 1_000_001, 1]).expect("bounds"),
        positions,
        None,
        None,
        vec![vec![[0, 0], [65_536, 0], [0, 65_536]]],
        indices,
        vec![
            NeutralMeshPrimitiveV1::new(MeshPrimitiveTopologyV1::Triangles, 0, 3, 0)
                .expect("primitive"),
        ],
    )
    .expect("mesh")
}

fn texture(id: u8) -> NeutralTextureV1 {
    NeutralTextureV1::new(
        schema_ref(NEUTRAL_TEXTURE_SCHEMA_ID),
        asset(id),
        1,
        NeutralTextureDimensionV1::D2,
        [2, 2, 1],
        1,
        NeutralTextureColorSpaceV1::Srgb,
        NeutralTextureAlphaSemanticsV1::Straight,
        NeutralTexelEncodingV1::Rgba8Unorm,
        vec![NeutralTextureMipLevelV1::new(
            [2, 2, 1],
            vec![
                255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255,
            ],
        )],
    )
    .expect("texture")
}

fn linear_texture(id: u8) -> NeutralTextureV1 {
    NeutralTextureV1::new(
        schema_ref(NEUTRAL_TEXTURE_SCHEMA_ID),
        asset(id),
        1,
        NeutralTextureDimensionV1::D2,
        [2, 2, 1],
        1,
        NeutralTextureColorSpaceV1::Linear,
        NeutralTextureAlphaSemanticsV1::Straight,
        NeutralTexelEncodingV1::Rgba8Unorm,
        vec![
            NeutralTextureMipLevelV1::new(
                [2, 2, 1],
                vec![
                    128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255,
                ],
            ),
            NeutralTextureMipLevelV1::new([1, 1, 1], vec![128, 128, 255, 255]),
        ],
    )
    .expect("texture")
}

fn b0_material_with_bindings(
    bindings: Vec<NeutralMaterialTextureBindingV1>,
) -> Result<NeutralMaterialV1, RenderContentContractError> {
    NeutralMaterialV1::new(
        schema_ref(NEUTRAL_MATERIAL_SCHEMA_ID),
        asset(2),
        1,
        [u16::MAX; 4],
        MaterialColorSpaceV1::Linear,
        0,
        u16::MAX,
        [0; 3],
        MaterialColorSpaceV1::Linear,
        0,
        65_536,
        u16::MAX,
        MaterialAlphaModeV1::Opaque,
        0,
        false,
        bindings,
        Vec::new(),
    )
}

/// Plan look/05 G3: the profile admits a base, metallic-roughness and normal
/// binding under one uniform UV scale over a mip-mapped linear texture, and
/// rejects a duplicate slot, an emissive binding, an sRGB normal map and a
/// non-uniform scale.
#[test]
fn b0_admits_three_bindings_under_a_uniform_scale() {
    let base = texture(1);
    let linear = linear_texture(9);
    let base_ref = base.asset_revision().expect("texture ref");
    let linear_ref = linear.asset_revision().expect("texture ref");
    let scale = UvTransformV1::new([2 * 65_536, 0, 0, 0, 2 * 65_536, 0]).expect("scale");
    let binding = |slot: MaterialTextureSlotV1, texture: AssetRevisionRefV1, transform| {
        NeutralMaterialTextureBindingV1::new(slot, texture, 0, transform).expect("binding")
    };
    let catalog = |material: NeutralMaterialV1| {
        let profile = B0RenderContentProfileV1::new(
            schema_ref(B0_RENDER_CONTENT_PROFILE_SCHEMA_ID),
            asset(4),
            1,
            b0_shader_interface_manifest_sha256(),
            material.asset_revision().expect("material ref"),
            base_ref,
        )
        .expect("profile");
        RenderContentCatalogV1::new(
            profile,
            vec![mesh(
                3,
                vec![[0, 0, 0], [1_000_000, 0, 0], [0, 1_000_000, 0]],
                vec![0, 1, 2],
            )],
            vec![material],
            vec![base.clone(), linear.clone()],
            Vec::new(),
        )
    };
    let accepted = b0_material_with_bindings(vec![
        binding(MaterialTextureSlotV1::BaseColor, base_ref, scale),
        binding(MaterialTextureSlotV1::MetallicRoughness, linear_ref, scale),
        binding(MaterialTextureSlotV1::Normal, linear_ref, scale),
    ])
    .expect("material");
    assert!(catalog(accepted).is_ok());
    let rejected = [
        vec![
            binding(MaterialTextureSlotV1::BaseColor, base_ref, scale),
            binding(MaterialTextureSlotV1::Normal, linear_ref, scale),
            binding(MaterialTextureSlotV1::Normal, linear_ref, scale),
        ],
        vec![
            binding(MaterialTextureSlotV1::BaseColor, base_ref, scale),
            binding(MaterialTextureSlotV1::Emissive, base_ref, scale),
        ],
        vec![
            binding(MaterialTextureSlotV1::BaseColor, base_ref, scale),
            binding(MaterialTextureSlotV1::Normal, base_ref, scale),
        ],
        vec![binding(
            MaterialTextureSlotV1::BaseColor,
            base_ref,
            UvTransformV1::new([2 * 65_536, 0, 0, 0, 65_536, 0]).expect("scale"),
        )],
        vec![
            binding(MaterialTextureSlotV1::BaseColor, base_ref, scale),
            binding(
                MaterialTextureSlotV1::Normal,
                linear_ref,
                UvTransformV1::identity(),
            ),
        ],
    ];
    for bindings in rejected {
        // The record itself refuses a duplicate slot; the profile refuses
        // the rest.
        let Ok(material) = b0_material_with_bindings(bindings) else {
            continue;
        };
        assert_eq!(
            catalog(material),
            Err(RenderContentContractError::UnsupportedB0Feature)
        );
    }
    assert_eq!(scale.uniform_scale_q16_16(), Some(2 * 65_536));
    assert_eq!(
        UvTransformV1::new([65_536, 0, 1, 0, 65_536, 0])
            .expect("bounded transform")
            .uniform_scale_q16_16(),
        None
    );
}

#[derive(Clone, Copy)]
struct B0MaterialOverrides {
    base_color_space: MaterialColorSpaceV1,
    metallic_unorm16: u16,
    double_sided: bool,
    uv_set: u8,
    uv_transform: UvTransformV1,
}

impl Default for B0MaterialOverrides {
    fn default() -> Self {
        Self {
            base_color_space: MaterialColorSpaceV1::Linear,
            metallic_unorm16: 0,
            double_sided: false,
            uv_set: 0,
            uv_transform: UvTransformV1::identity(),
        }
    }
}

fn b0_material(texture: AssetRevisionRefV1, overrides: B0MaterialOverrides) -> NeutralMaterialV1 {
    NeutralMaterialV1::new(
        schema_ref(NEUTRAL_MATERIAL_SCHEMA_ID),
        asset(2),
        1,
        [u16::MAX; 4],
        overrides.base_color_space,
        overrides.metallic_unorm16,
        u16::MAX,
        [0; 3],
        MaterialColorSpaceV1::Linear,
        0,
        65_536,
        u16::MAX,
        MaterialAlphaModeV1::Opaque,
        0,
        overrides.double_sided,
        vec![
            NeutralMaterialTextureBindingV1::new(
                MaterialTextureSlotV1::BaseColor,
                texture,
                overrides.uv_set,
                overrides.uv_transform,
            )
            .expect("binding"),
        ],
        Vec::new(),
    )
    .expect("neutral material")
}

fn schema_ref(schema_id: &str) -> SchemaRefV1 {
    SchemaRefV1 {
        schema_id: SchemaId::new(schema_id).expect("schema id"),
        schema_version: 1,
        descriptor_sha256: domain_hash("nextengine.schema-descriptor.v1", schema_id.as_bytes()),
        role: SchemaRoleV1::NeutralContent,
        encoding: SchemaEncodingV1::CanonicalBinaryV1,
    }
}

const fn asset(value: u8) -> AssetId {
    AssetId::from_bytes([value; 16])
}

const fn hash(value: u8) -> ContentHash {
    ContentHash::from_bytes([value; 32])
}
