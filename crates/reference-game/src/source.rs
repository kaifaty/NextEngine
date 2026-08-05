use next_contracts::content::{NeutralPropertyV1, NeutralRecordKindV1, NeutralRecordV1};
use next_contracts::ids::{AssetId, PersistentId, ProjectId, SchemaId};
use next_contracts::localization::{TextCatalogEntryV1, TextCatalogV1, TextLocaleTagV1};
use next_contracts::platform::PresentationTargetKindV1;
use next_contracts::project::{
    ContentProvenanceV1, SchemaEncodingV1, SchemaRefV1, SchemaRoleV1, domain_hash,
};
use next_contracts::render_content::{
    AabbI64V1, B0_RENDER_CONTENT_PROFILE_SCHEMA_ID, B0RenderContentProfileV1, MaterialAlphaModeV1,
    MaterialColorSpaceV1, MaterialTextureSlotV1, MeshPrimitiveTopologyV1,
    NEUTRAL_MATERIAL_SCHEMA_ID, NEUTRAL_MESH_SCHEMA_ID, NEUTRAL_TEXTURE_SCHEMA_ID,
    NeutralMaterialTextureBindingV1, NeutralMaterialV1, NeutralMeshPrimitiveV1, NeutralMeshV1,
    NeutralRenderRecordV1, NeutralTexelEncodingV1, NeutralTextureAlphaSemanticsV1,
    NeutralTextureColorSpaceV1, NeutralTextureDimensionV1, NeutralTextureMipLevelV1,
    NeutralTextureV1, UvTransformV1, b0_shader_interface_manifest_sha256,
};
use next_contracts::session::{RecoveryPolicyV1, ShutdownPolicyV1};
use next_project::{NeutralProjectSourceV1, ProjectCookError, SourceChunkBindingV1};

pub const REFERENCE_GAME_PROJECT_ID: &str = "org.nextengine.reference-game";
const REFERENCE_CONTENT_IDENTITY: &str = "org.nextengine.reference-game.content";
const REFERENCE_RESOLVER_PROFILE_ID: &str = "nextengine.resolver.exact-minimum.v1";

pub const REFERENCE_FLOOR_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x81; 16]);
pub const REFERENCE_MARKER_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x82; 16]);
pub const REFERENCE_BASE_TEXTURE_ASSET_ID: AssetId = AssetId::from_bytes([0x83; 16]);
pub const REFERENCE_FALLBACK_TEXTURE_ASSET_ID: AssetId = AssetId::from_bytes([0x84; 16]);
pub const REFERENCE_BASE_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0x85; 16]);
pub const REFERENCE_FALLBACK_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0x86; 16]);
pub const REFERENCE_RENDER_PROFILE_ASSET_ID: AssetId = AssetId::from_bytes([0x87; 16]);
pub const REFERENCE_TEXT_CATALOG_EN_ASSET_ID: AssetId = AssetId::from_bytes([0x91; 16]);
pub const REFERENCE_TEXT_CATALOG_PSEUDO_ASSET_ID: AssetId = AssetId::from_bytes([0x92; 16]);

pub fn project_source_v2() -> Result<NeutralProjectSourceV1, ProjectCookError> {
    project_source_v2_with_id(REFERENCE_GAME_PROJECT_ID)
}

pub fn project_source_v2_with_id(
    project_id: &str,
) -> Result<NeutralProjectSourceV1, ProjectCookError> {
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
        NeutralRecordKindV1::AbilityDefinition,
        NeutralRecordKindV1::WorldChunk,
        NeutralRecordKindV1::WorldChunk,
    ];
    let asset_ids: Vec<_> = (1_u8..=13)
        .map(|byte| AssetId::from_bytes([byte; 16]))
        .collect();
    let persistent_ids: Vec<_> = (31_u8..=43)
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
            NeutralRecordKindV1::AbilityDefinition => (vec![persistent_ids[3]], vec![asset_ids[3]]),
            NeutralRecordKindV1::WorldChunk if index == 11 => {
                (persistent_ids[1..=5].to_vec(), asset_ids[1..=5].to_vec())
            }
            NeutralRecordKindV1::WorldChunk => {
                (persistent_ids[2..=10].to_vec(), asset_ids[2..=10].to_vec())
            }
            _ => (Vec::new(), Vec::new()),
        };
        let mut properties = vec![NeutralPropertyV1 {
            property_id: SchemaId::new("nextengine.reference.role")?,
            value_id: SchemaId::new(format!("nextengine.reference.{kind:?}").to_lowercase())?,
        }];
        properties.extend(definition_properties(kind)?);
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
            properties,
        )?);
    }

    let render_records = reference_render_records()?;
    let render_root_asset_ids = [
        REFERENCE_FLOOR_MESH_ASSET_ID,
        REFERENCE_MARKER_MESH_ASSET_ID,
        REFERENCE_BASE_MATERIAL_ASSET_ID,
        REFERENCE_RENDER_PROFILE_ASSET_ID,
    ];
    let mut root_asset_ids = vec![asset_ids[0]];
    root_asset_ids.extend(render_root_asset_ids);
    Ok(NeutralProjectSourceV1 {
        project_id: ProjectId::new(project_id)?,
        project_revision: 2,
        content_identity: SchemaId::new(REFERENCE_CONTENT_IDENTITY)?,
        resolver_profile_id: SchemaId::new(REFERENCE_RESOLVER_PROFILE_ID)?,
        resolver_profile_version: 1,
        records,
        render_records,
        text_catalogs: reference_text_catalogs()?,
        audio_records: Vec::new(),
        root_asset_ids,
        provenance: ContentProvenanceV1::new(
            SchemaId::new("nextengine.reference.provenance.cc0")?,
            SchemaId::new("CC0-1.0")?,
            "Next Engine generated reference-game content; CC0-1.0",
        )?,
        license_manifest_sha256: domain_hash(
            "nextengine.license-manifest.v1",
            b"CC0-1.0\0Next Engine generated reference-game content",
        ),
        partition_id: SchemaId::new("nextengine.reference.partition.v1")?,
        coordinate_profile_id: SchemaId::new("nextengine.coordinates.right-handed-metres.v1")?,
        chunks: vec![
            SourceChunkBindingV1 {
                chunk_id: SchemaId::new("nextengine.reference.chunk.start")?,
                region_id: SchemaId::new("nextengine.reference.region.start")?,
                chunk_asset_id: asset_ids[11],
                required_asset_ids: asset_ids[1..=5].to_vec(),
            },
            SourceChunkBindingV1 {
                chunk_id: SchemaId::new("nextengine.reference.chunk.frontier")?,
                region_id: SchemaId::new("nextengine.reference.region.frontier")?,
                chunk_asset_id: asset_ids[12],
                required_asset_ids: asset_ids[2..=10].to_vec(),
            },
        ],
        recovery_policy: RecoveryPolicyV1::reference_game_default(),
        shutdown_policy: ShutdownPolicyV1::reference_game_default(),
        allowed_presentation_targets: vec![
            PresentationTargetKindV1::None,
            PresentationTargetKindV1::Interactive,
            PresentationTargetKindV1::DisplaylessOffscreen,
        ],
    })
}

/// Reference text catalogs: the `en` source locale covers every text ID the
/// semantic UI projection emits plus the quest-state TextId argument targets;
/// the `qps-ploc` pseudo-locale intentionally omits
/// `nextengine.ui.text.pause-menu.load` so resolution exercises the declared
/// `qps-ploc -> en` fallback chain deterministically.
fn reference_text_catalogs() -> Result<Vec<TextCatalogV1>, ProjectCookError> {
    let en_entries: &[(&str, &str)] = &[
        ("nextengine.ui.text.hud.health", "Health {0}/{1}"),
        ("nextengine.ui.text.hud.quest-state", "Quest: {0}"),
        ("nextengine.ui.text.pause-menu.title", "Paused"),
        ("nextengine.ui.text.pause-menu.resume", "Resume"),
        ("nextengine.ui.text.pause-menu.save", "Save game"),
        ("nextengine.ui.text.pause-menu.load", "Load game"),
        ("nextengine.reference.quest.available", "Available"),
        ("nextengine.reference.quest.active", "Active"),
        (
            "nextengine.reference.item.training-sword",
            "Worn training sword",
        ),
        (
            "nextengine.reference.quest.a-helping-hand",
            "A Helping Hand",
        ),
        ("nextengine.reference.dialogue.offer", "Will you help me?"),
        ("nextengine.reference.dialogue.accepted", "Thank you!"),
        ("nextengine.ui.text.inventory.title", "Inventory"),
        ("nextengine.ui.text.inventory.item-row", "{0} x{1}"),
        ("nextengine.ui.text.inventory.empty", "(empty)"),
        ("nextengine.ui.text.equipment.title", "Equipment"),
        ("nextengine.ui.text.equipment.slot-row", "{0}: {1}"),
        ("nextengine.rpg.equipment-slot.main-hand", "Main hand"),
        ("nextengine.ui.text.journal.title", "Quests"),
        ("nextengine.ui.text.journal.entry", "{0} - {1}"),
        ("nextengine.ui.text.dialogue.title", "Dialogue"),
        ("nextengine.ui.text.dialogue.choice-accept", "Accept"),
        ("nextengine.ui.text.dialogue.choice-leave", "Leave"),
    ];
    let pseudo_entries: &[(&str, &str)] = &[
        ("nextengine.ui.text.hud.health", "⟦Ħēåłŧħ⟧ {0}/{1}"),
        ("nextengine.ui.text.hud.quest-state", "⟦Qųēșŧ⟧: {0}"),
        ("nextengine.ui.text.pause-menu.title", "⟦Påųșēḑ⟧"),
        ("nextengine.ui.text.pause-menu.resume", "⟦Rēșųmē⟧"),
        ("nextengine.ui.text.pause-menu.save", "⟦Șåvē ĝåmē⟧"),
        ("nextengine.reference.quest.available", "⟦Åvēåíłåbłē⟧"),
        ("nextengine.reference.quest.active", "⟦Åćŧívē⟧"),
        (
            "nextengine.reference.item.training-sword",
            "⟦Wårn ŧråíníng șwårḑ⟧",
        ),
        (
            "nextengine.reference.quest.a-helping-hand",
            "⟦Å Ħēłpíng Ħånḑ⟧",
        ),
        ("nextengine.reference.dialogue.offer", "⟦Wíłł you ħēłp mē?⟧"),
        ("nextengine.reference.dialogue.accepted", "⟦ŧhank you!⟧"),
        ("nextengine.ui.text.inventory.title", "⟦Ínvēnŧåry⟧"),
        ("nextengine.ui.text.inventory.item-row", "⟦{0} x{1}⟧"),
        ("nextengine.ui.text.inventory.empty", "⟦(ēmpŧå)⟧"),
        ("nextengine.ui.text.equipment.title", "⟦Ēqųípmēnŧ⟧"),
        ("nextengine.ui.text.equipment.slot-row", "⟦{0}: {1}⟧"),
        ("nextengine.rpg.equipment-slot.main-hand", "⟦Måín ħånḑ⟧"),
        ("nextengine.ui.text.journal.title", "⟦Qųēșŧș⟧"),
        ("nextengine.ui.text.journal.entry", "⟦{0} - {1}⟧"),
        ("nextengine.ui.text.dialogue.title", "⟦ḑíåłåĝųē⟧"),
        ("nextengine.ui.text.dialogue.choice-accept", "⟦Åććēpŧ⟧"),
        ("nextengine.ui.text.dialogue.choice-leave", "⟦Łēåvē⟧"),
    ];
    Ok(vec![
        reference_text_catalog(REFERENCE_TEXT_CATALOG_EN_ASSET_ID, "en", None, en_entries)?,
        reference_text_catalog(
            REFERENCE_TEXT_CATALOG_PSEUDO_ASSET_ID,
            "qps-ploc",
            Some("en"),
            pseudo_entries,
        )?,
    ])
}

fn reference_text_catalog(
    asset_id: AssetId,
    locale: &str,
    fallback: Option<&str>,
    entries: &[(&str, &str)],
) -> Result<TextCatalogV1, ProjectCookError> {
    let entries = entries
        .iter()
        .map(|(text_id, template)| {
            Ok(TextCatalogEntryV1::new(
                SchemaId::new(*text_id)?,
                (*template).to_owned(),
            )?)
        })
        .collect::<Result<Vec<_>, ProjectCookError>>()?;
    Ok(TextCatalogV1::new(
        asset_id,
        1,
        TextLocaleTagV1::new(locale)?,
        fallback.map(TextLocaleTagV1::new).transpose()?,
        entries,
    )?)
}

fn reference_render_records() -> Result<Vec<NeutralRenderRecordV1>, ProjectCookError> {
    let base_texture = NeutralTextureV1::new(
        schema_ref(
            NEUTRAL_TEXTURE_SCHEMA_ID,
            SchemaRoleV1::NeutralContent,
            SchemaEncodingV1::CanonicalBinaryV1,
        )?,
        REFERENCE_BASE_TEXTURE_ASSET_ID,
        1,
        NeutralTextureDimensionV1::D2,
        [2, 2, 1],
        1,
        NeutralTextureColorSpaceV1::Srgb,
        NeutralTextureAlphaSemanticsV1::Opaque,
        NeutralTexelEncodingV1::Rgba8Unorm,
        vec![NeutralTextureMipLevelV1::new(
            [2, 2, 1],
            vec![
                255, 255, 255, 255, 220, 235, 255, 255, 220, 235, 255, 255, 255, 255, 255, 255,
            ],
        )],
    )?;
    let fallback_texture = NeutralTextureV1::new(
        schema_ref(
            NEUTRAL_TEXTURE_SCHEMA_ID,
            SchemaRoleV1::NeutralContent,
            SchemaEncodingV1::CanonicalBinaryV1,
        )?,
        REFERENCE_FALLBACK_TEXTURE_ASSET_ID,
        1,
        NeutralTextureDimensionV1::D2,
        [2, 2, 1],
        1,
        NeutralTextureColorSpaceV1::Srgb,
        NeutralTextureAlphaSemanticsV1::Opaque,
        NeutralTexelEncodingV1::Rgba8Unorm,
        vec![NeutralTextureMipLevelV1::new(
            [2, 2, 1],
            vec![
                255, 0, 255, 255, 16, 16, 16, 255, 16, 16, 16, 255, 255, 0, 255, 255,
            ],
        )],
    )?;
    let base_material = reference_material(
        REFERENCE_BASE_MATERIAL_ASSET_ID,
        base_texture.asset_revision()?,
    )?;
    let fallback_material = reference_material(
        REFERENCE_FALLBACK_MATERIAL_ASSET_ID,
        fallback_texture.asset_revision()?,
    )?;
    let floor_mesh = reference_floor_mesh(REFERENCE_FLOOR_MESH_ASSET_ID)?;
    let marker_mesh = reference_quad_mesh(
        REFERENCE_MARKER_MESH_ASSET_ID,
        [[-80_000, -120_000, 0], [80_000, 120_000, 0]],
    )?;
    let profile = B0RenderContentProfileV1::new(
        schema_ref(
            B0_RENDER_CONTENT_PROFILE_SCHEMA_ID,
            SchemaRoleV1::NeutralContent,
            SchemaEncodingV1::CanonicalBinaryV1,
        )?,
        REFERENCE_RENDER_PROFILE_ASSET_ID,
        1,
        b0_shader_interface_manifest_sha256(),
        fallback_material.asset_revision()?,
        fallback_texture.asset_revision()?,
    )?;
    Ok(vec![
        floor_mesh.into(),
        marker_mesh.into(),
        base_texture.into(),
        fallback_texture.into(),
        base_material.into(),
        fallback_material.into(),
        profile.into(),
    ])
}

fn reference_material(
    asset_id: AssetId,
    texture: next_contracts::project::AssetRevisionRefV1,
) -> Result<NeutralMaterialV1, ProjectCookError> {
    Ok(NeutralMaterialV1::new(
        schema_ref(
            NEUTRAL_MATERIAL_SCHEMA_ID,
            SchemaRoleV1::NeutralContent,
            SchemaEncodingV1::CanonicalBinaryV1,
        )?,
        asset_id,
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
        vec![NeutralMaterialTextureBindingV1::new(
            MaterialTextureSlotV1::BaseColor,
            texture,
            0,
            UvTransformV1::identity(),
        )?],
        Vec::new(),
    )?)
}

fn reference_quad_mesh(
    asset_id: AssetId,
    corners: [[i64; 3]; 2],
) -> Result<NeutralMeshV1, ProjectCookError> {
    let [minimum, maximum] = corners;
    let bounds = AabbI64V1::new(
        [minimum[0], minimum[1], -1],
        [
            maximum[0].saturating_add(1),
            maximum[1].saturating_add(1),
            1,
        ],
    )?;
    Ok(NeutralMeshV1::new(
        schema_ref(
            NEUTRAL_MESH_SCHEMA_ID,
            SchemaRoleV1::NeutralContent,
            SchemaEncodingV1::CanonicalBinaryV1,
        )?,
        asset_id,
        1,
        bounds,
        vec![
            [minimum[0], minimum[1], 0],
            [maximum[0], minimum[1], 0],
            [maximum[0], maximum[1], 0],
            [minimum[0], maximum[1], 0],
        ],
        None,
        None,
        vec![vec![[0, 0], [65_536, 0], [65_536, 65_536], [0, 65_536]]],
        // Presentation markers are camera-facing placeholders, not physical
        // surfaces. Emit both windings so they remain visible while orbiting
        // the third-person camera under the B0 back-face-culling pipeline.
        vec![0, 1, 2, 2, 3, 0, 0, 2, 1, 2, 0, 3],
        vec![NeutralMeshPrimitiveV1::new(
            MeshPrimitiveTopologyV1::Triangles,
            0,
            12,
            0,
        )?],
    )?)
}

fn reference_floor_mesh(asset_id: AssetId) -> Result<NeutralMeshV1, ProjectCookError> {
    // The reference physics profile is Y-up. The floor body is centred at
    // y=-100 mm with a 100 mm half-height, so its walkable top is local
    // y=+100 mm and world y=0. Match the 20 m x 20 m physical footprint.
    const HALF_EXTENT: i64 = 10_000_000;
    const LOCAL_TOP_Y: i64 = 100_000;
    let bounds = AabbI64V1::new(
        [-HALF_EXTENT, LOCAL_TOP_Y - 1, -HALF_EXTENT],
        [HALF_EXTENT + 1, LOCAL_TOP_Y + 1, HALF_EXTENT + 1],
    )?;
    Ok(NeutralMeshV1::new(
        schema_ref(
            NEUTRAL_MESH_SCHEMA_ID,
            SchemaRoleV1::NeutralContent,
            SchemaEncodingV1::CanonicalBinaryV1,
        )?,
        asset_id,
        1,
        bounds,
        // Counter-clockwise from above (+Y), matching the B0 front-face
        // contract after the Vulkan projection Y flip.
        vec![
            [-HALF_EXTENT, LOCAL_TOP_Y, -HALF_EXTENT],
            [-HALF_EXTENT, LOCAL_TOP_Y, HALF_EXTENT],
            [HALF_EXTENT, LOCAL_TOP_Y, HALF_EXTENT],
            [HALF_EXTENT, LOCAL_TOP_Y, -HALF_EXTENT],
        ],
        None,
        None,
        vec![vec![[0, 0], [0, 65_536], [65_536, 65_536], [65_536, 0]]],
        vec![0, 1, 2, 2, 3, 0],
        vec![NeutralMeshPrimitiveV1::new(
            MeshPrimitiveTopologyV1::Triangles,
            0,
            6,
            0,
        )?],
    )?)
}

fn definition_properties(
    kind: NeutralRecordKindV1,
) -> Result<Vec<NeutralPropertyV1>, ProjectCookError> {
    let pairs: &[(&str, &str)] = match kind {
        NeutralRecordKindV1::DialogueDefinition => &[
            (
                "nextengine.dialogue.entry-node",
                "nextengine.reference.dialogue.offer",
            ),
            (
                "nextengine.dialogue.accepted-node",
                "nextengine.reference.dialogue.accepted",
            ),
            (
                "nextengine.display-name.text-id",
                "nextengine.ui.text.dialogue.title",
            ),
        ],
        NeutralRecordKindV1::ItemDefinition => &[(
            "nextengine.display-name.text-id",
            "nextengine.reference.item.training-sword",
        )],
        NeutralRecordKindV1::QuestDefinition => &[
            (
                "nextengine.quest.entry-state",
                "nextengine.reference.quest.available",
            ),
            (
                "nextengine.quest.active-state",
                "nextengine.reference.quest.active",
            ),
            (
                "nextengine.display-name.text-id",
                "nextengine.reference.quest.a-helping-hand",
            ),
        ],
        NeutralRecordKindV1::RelationshipDefinition => &[(
            "nextengine.relationship.dimension",
            "nextengine.reference.relationship.trust",
        )],
        NeutralRecordKindV1::InteractionDefinition => &[
            (
                "nextengine.interaction.definition-id",
                "nextengine.reference.interaction.accept-help",
            ),
            (
                "nextengine.interaction.dialogue-transition",
                "nextengine.reference.transition.dialogue.accept",
            ),
            (
                "nextengine.interaction.quest-transition",
                "nextengine.reference.transition.quest.accept",
            ),
            (
                "nextengine.interaction.relationship-delta",
                "nextengine.value.i32.7",
            ),
        ],
        NeutralRecordKindV1::AbilityDefinition => &[
            (
                "nextengine.ability.definition-id",
                "nextengine.reference.ability.training-melee",
            ),
            (
                "nextengine.ability.semantic-action",
                "nextengine.action.melee",
            ),
            (
                "nextengine.ability.resource",
                "nextengine.rpg.resource.health",
            ),
            (
                "nextengine.ability.resource-delta",
                "nextengine.value.i32.-25",
            ),
            (
                "nextengine.ability.cooldown-ticks",
                "nextengine.value.u32.2",
            ),
            (
                "nextengine.ability.cooldown-group",
                "nextengine.cooldown.melee",
            ),
            (
                "nextengine.ability.equipment-slot",
                "nextengine.rpg.equipment-slot.main-hand",
            ),
        ],
        _ => &[],
    };
    pairs
        .iter()
        .map(|(key, value)| {
            Ok(NeutralPropertyV1 {
                property_id: SchemaId::new(*key)?,
                value_id: SchemaId::new(*value)?,
            })
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::{REFERENCE_FLOOR_MESH_ASSET_ID, reference_floor_mesh};

    #[test]
    fn reference_floor_matches_the_y_up_physics_surface() {
        let mesh =
            reference_floor_mesh(REFERENCE_FLOOR_MESH_ASSET_ID).expect("reference floor mesh");
        assert_eq!(
            mesh.positions_micrometres(),
            &[
                [-10_000_000, 100_000, -10_000_000],
                [-10_000_000, 100_000, 10_000_000],
                [10_000_000, 100_000, 10_000_000],
                [10_000_000, 100_000, -10_000_000],
            ]
        );

        let [a, b, c, _] = mesh.positions_micrometres() else {
            panic!("floor is one quad");
        };
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let normal_y = ab[2] * ac[0] - ab[0] * ac[2];
        assert!(normal_y > 0, "floor front face must point toward +Y");
    }
}
