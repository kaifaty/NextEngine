//! ADR-100 first water consumer: one sealed basin in the reference scene.
//!
//! The basin is authoritative gameplay water. Its extent sits on the floor
//! east of the rock proxies and clear of the R5b course; presentation water
//! (ADR-101) may animate its surface later but never writes here.

use next_contracts::ids::PersistentId;
use next_contracts::physics::{
    PhysicsCanonicalSnapshotV2, PhysicsGeometryV1, PhysicsShapeIdV1, WaterSubmersionV1,
    WaterVolumeDefinitionV1, WaterVolumeSetV1,
};

use crate::ReferenceGameError;
use crate::session::ReferenceGameSession;

/// Durable identity of the reference basin.
pub const REFERENCE_WATER_BASIN_ID: PersistentId = PersistentId::from_bytes([0x7a; 16]);
/// Basin footprint: `4 x 2 m` on the floor, `2 m` deep, filled to `0.5 m`.
pub const REFERENCE_WATER_BASIN_MINIMUM_MICROMETRES: [i64; 3] = [4_500_000, 0, 1_000_000];
pub const REFERENCE_WATER_BASIN_MAXIMUM_MICROMETRES: [i64; 3] = [8_500_000, 2_000_000, 3_000_000];
pub const REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES: i64 = 500_000;
/// A point at least this deep below the surface counts as swimming.
pub const REFERENCE_WATER_BASIN_SWIMMING_DEPTH_MICROMETRES: i64 = 1_200_000;
pub const REFERENCE_WATER_BASIN_PROFILE_REVISION: u32 = 1;

#[must_use]
pub fn reference_water_basin_definition() -> WaterVolumeDefinitionV1 {
    WaterVolumeDefinitionV1 {
        volume_id: REFERENCE_WATER_BASIN_ID,
        minimum_micrometres: REFERENCE_WATER_BASIN_MINIMUM_MICROMETRES,
        maximum_micrometres: REFERENCE_WATER_BASIN_MAXIMUM_MICROMETRES,
        initial_level_micrometres: REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES,
        swimming_depth_micrometres: REFERENCE_WATER_BASIN_SWIMMING_DEPTH_MICROMETRES,
        level_ramp: None,
        profile_revision: REFERENCE_WATER_BASIN_PROFILE_REVISION,
    }
}

pub fn reference_water_volumes() -> Result<WaterVolumeSetV1, ReferenceGameError> {
    Ok(WaterVolumeSetV1::from_definitions([
        reference_water_basin_definition(),
    ])?)
}

/// Exact submersion of the player's capsule foot point: the committed
/// capsule pose minus its half segment and radius, queried against the
/// committed water table at the given tick. Presentation and HUD read this
/// projection; it never writes gameplay state.
pub fn player_submersion(
    fixture: &ReferenceGameSession,
    physics: &PhysicsCanonicalSnapshotV2,
    water: &WaterVolumeSetV1,
    tick: u64,
) -> Result<WaterSubmersionV1, ReferenceGameError> {
    if water.is_empty() {
        return Ok(WaterSubmersionV1::dry());
    }
    let body = physics
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or(ReferenceGameError::BodyMissing)?;
    let capsule = fixture
        .bootstrap
        .physics_checkpoint
        .catalog
        .bodies
        .get(&fixture.physics_body_id)
        .and_then(|descriptor| {
            descriptor.shapes.get(&PhysicsShapeIdV1 {
                body_id: fixture.physics_body_id,
                shape_slot: 0,
            })
        })
        .ok_or(ReferenceGameError::BodyMissing)?;
    let PhysicsGeometryV1::Capsule {
        radius_micrometres,
        half_segment_micrometres,
    } = capsule.geometry
    else {
        return Err(ReferenceGameError::BodyMissing);
    };
    let foot_offset = radius_micrometres
        .checked_add(half_segment_micrometres)
        .ok_or(ReferenceGameError::PresentationTransformOverflow)?;
    let translation = body.pose.translation_micrometres;
    let foot = [
        translation[0],
        translation[1]
            .checked_sub(foot_offset)
            .ok_or(ReferenceGameError::PresentationTransformOverflow)?,
        translation[2],
    ];
    Ok(water.submersion_at(foot, tick))
}

/// World translation of the authored basin surface quad (authored at local
/// `y = 0`): the exact effective level of the reference basin at `tick`.
pub fn water_surface_translation(
    water: &WaterVolumeSetV1,
    tick: u64,
) -> Result<[i64; 3], ReferenceGameError> {
    let level = water
        .effective_level(REFERENCE_WATER_BASIN_ID, tick)
        .ok_or(ReferenceGameError::PresentationAssetMissing)?;
    Ok([0, level, 0])
}
