use std::path::PathBuf;

use next_contracts::ids::AssetId;
use next_project::{NeutralProjectSourceV3, ProjectCookError};

/// Stable project ID retained behind the historical constant name so the
/// reference-game crate remains a harness while production content lives in
/// `projects/reference-alpha`.
pub const REFERENCE_GAME_PROJECT_ID: &str = "org.nextengine.reference-alpha";

pub const REFERENCE_FLOOR_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x81; 16]);
pub const REFERENCE_HUMANOID_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc1; 16]);
pub const REFERENCE_ENEMY_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc9; 16]);
pub const REFERENCE_QUEST_GIVER_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xca; 16]);
pub const REFERENCE_BLADE_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc2; 16]);
pub const REFERENCE_RELAY_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc3; 16]);
pub const REFERENCE_RELAY_APPROACH_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc6; 16]);
pub const REFERENCE_FOCUS_RING_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc7; 16]);
pub const REFERENCE_QUEST_MARKER_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc8; 16]);
pub const REFERENCE_BASE_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0x85; 16]);
pub const REFERENCE_PLAYER_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xd1; 16]);
pub const REFERENCE_ENEMY_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xd2; 16]);
pub const REFERENCE_QUEST_GIVER_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xd3; 16]);
pub const REFERENCE_BLADE_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xd4; 16]);
pub const REFERENCE_RELAY_INACTIVE_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xd5; 16]);
pub const REFERENCE_RELAY_ACTIVE_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xd6; 16]);
pub const REFERENCE_DEFEATED_ENEMY_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xd8; 16]);
pub const REFERENCE_RELAY_APPROACH_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xd9; 16]);
pub const REFERENCE_INDICATOR_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0xd7; 16]);

#[must_use]
pub fn reference_alpha_project_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("projects")
        .join("reference-alpha")
}

pub fn project_source_v3() -> Result<NeutralProjectSourceV3, ProjectCookError> {
    Ok(next_project::load_project_authoring_v3(
        reference_alpha_project_directory(),
    )?)
}

pub fn project_source_v3_with_id(
    project_id: &str,
) -> Result<NeutralProjectSourceV3, ProjectCookError> {
    Ok(next_project::load_project_authoring_v3_with_project_id(
        reference_alpha_project_directory(),
        project_id,
    )?)
}
