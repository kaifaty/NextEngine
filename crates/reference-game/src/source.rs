use next_contracts::content::{NeutralPropertyV1, NeutralRecordKindV1, NeutralRecordV1};
use next_contracts::ids::{AssetId, PersistentId, ProjectId, SchemaId};
use next_contracts::platform::PresentationTargetKindV1;
use next_contracts::project::{
    ContentProvenanceV1, SchemaEncodingV1, SchemaRefV1, SchemaRoleV1, domain_hash,
};
use next_contracts::session::{RecoveryPolicyV1, ShutdownPolicyV1};
use next_project::{NeutralProjectSourceV1, ProjectCookError, SourceChunkBindingV1};

pub const REFERENCE_GAME_PROJECT_ID: &str = "org.nextengine.reference-game";
const REFERENCE_CONTENT_IDENTITY: &str = "org.nextengine.reference-game.content";
const REFERENCE_RESOLVER_PROFILE_ID: &str = "nextengine.resolver.exact-minimum.v1";

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

    Ok(NeutralProjectSourceV1 {
        project_id: ProjectId::new(project_id)?,
        project_revision: 2,
        content_identity: SchemaId::new(REFERENCE_CONTENT_IDENTITY)?,
        resolver_profile_id: SchemaId::new(REFERENCE_RESOLVER_PROFILE_ID)?,
        resolver_profile_version: 1,
        records,
        root_asset_ids: vec![asset_ids[0]],
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
        ],
        NeutralRecordKindV1::QuestDefinition => &[
            (
                "nextengine.quest.entry-state",
                "nextengine.reference.quest.available",
            ),
            (
                "nextengine.quest.active-state",
                "nextengine.reference.quest.active",
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
