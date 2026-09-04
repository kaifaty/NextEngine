use std::path::PathBuf;

use next_contracts::ids::AssetId;
use next_project::{NeutralProjectSourceV7, ProjectCookError};

/// Stable project ID retained behind the historical constant name so the
/// reference-game crate remains a harness while production content lives in
/// `projects/reference-alpha`.
pub const REFERENCE_GAME_PROJECT_ID: &str = "org.nextengine.reference-alpha";

pub const REFERENCE_FLOOR_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x81; 16]);
pub const REFERENCE_HUMANOID_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc1; 16]);
pub const REFERENCE_QUEST_GIVER_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xca; 16]);
pub const REFERENCE_BLADE_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc2; 16]);
pub const REFERENCE_RELAY_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc3; 16]);
pub const REFERENCE_RELAY_APPROACH_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xc6; 16]);
pub const REFERENCE_R5B_COURSE_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xcb; 16]);
pub const REFERENCE_R5B_PUSH_BOX_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0xcc; 16]);
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
pub const REFERENCE_WATER_SURFACE_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x7c; 16]);
pub const REFERENCE_WATER_MATERIAL_ASSET_ID: AssetId = AssetId::from_bytes([0x7d; 16]);
/// ADR-103 vessel surface quads (presentation only), sharing the water material.
pub const REFERENCE_WATER_VESSEL_A_SURFACE_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x7e; 16]);
pub const REFERENCE_WATER_VESSEL_B_SURFACE_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x7f; 16]);
/// ADR-105: the floating crate cube (`0.5 m`) of the basin.
pub const REFERENCE_WATER_CRATE_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x8d; 16]);
/// Plan 16: the compound rim mesh around the basin (five boxes).
pub const REFERENCE_WATER_BASIN_RIM_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x8e; 16]);
/// Plan 32: the pond interior (walls, floor, steps) and its surface quad.
pub const REFERENCE_WATER_POND_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x9d; 16]);
pub const REFERENCE_WATER_POND_SURFACE_MESH_ASSET_ID: AssetId = AssetId::from_bytes([0x9e; 16]);

#[must_use]
pub fn reference_alpha_project_directory() -> PathBuf {
    reference_alpha_project_directory_for_build()
}

#[cfg(debug_assertions)]
fn reference_alpha_project_directory_for_build() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("projects")
        .join("reference-alpha")
}

#[cfg(not(debug_assertions))]
fn reference_alpha_project_directory_for_build() -> PathBuf {
    const RELATIVE_CANDIDATES: [&str; 2] = ["source/reference-alpha", "projects/reference-alpha"];

    for candidate in RELATIVE_CANDIDATES.map(PathBuf::from) {
        if candidate.is_dir() {
            return candidate;
        }
    }
    if let Ok(executable) = std::env::current_exe() {
        for ancestor in executable.ancestors().take(6) {
            for relative in RELATIVE_CANDIDATES {
                let candidate = ancestor.join(relative);
                if candidate.is_dir() {
                    return candidate;
                }
            }
        }
    }
    PathBuf::from(RELATIVE_CANDIDATES[0])
}

pub fn project_source_v7() -> Result<NeutralProjectSourceV7, ProjectCookError> {
    Ok(next_project::load_project_authoring_v7(
        reference_alpha_project_directory(),
    )?)
}

pub fn project_source_v7_with_id(
    project_id: &str,
) -> Result<NeutralProjectSourceV7, ProjectCookError> {
    Ok(next_project::load_project_authoring_v7_with_project_id(
        reference_alpha_project_directory(),
        project_id,
    )?)
}
