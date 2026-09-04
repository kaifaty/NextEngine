//! ADR-100 first water consumer: one sealed basin in the reference scene.
//!
//! The basin is authoritative gameplay water. Its extent sits on the floor
//! east of the rock proxies and clear of the R5b course; presentation water
//! (ADR-101) may animate its surface later but never writes here.

use next_contracts::ids::PersistentId;
use next_contracts::physics::PhysicsBodyIdV1;
use next_contracts::physics::{
    PhysicsCanonicalSnapshotV2, PhysicsGeometryV1, PhysicsShapeIdV1, WaterFlowEdgeKindV1,
    WaterFlowEdgeV1, WaterFlowNetworkV1, WaterSubmersionV1, WaterVolumeDefinitionV1,
    WaterVolumeSetV1,
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
/// ADR-105 / plan `continuum-water/08`: the floating crate of the basin,
/// a `0.5 m` cube of `50 kg` resting on the basin floor at activation.
pub const REFERENCE_WATER_CRATE_BODY_ID: PhysicsBodyIdV1 = PhysicsBodyIdV1 {
    subject_id: PersistentId::from_bytes([0x87; 16]),
    body_slot: 0,
};
pub const REFERENCE_WATER_CRATE_HALF_EXTENTS_MICROMETRES: [i64; 3] = [250_000, 250_000, 250_000];
pub const REFERENCE_WATER_CRATE_INITIAL_TRANSLATION_MICROMETRES: [i64; 3] =
    [6_500_000, 250_000, 2_000_000];
pub const REFERENCE_WATER_CRATE_MASS_MICROKILOGRAMS: u64 = 50_000_000;
/// Plan `continuum-water/16`: the authored rim around the basin, one
/// static body with five box shapes (centre, half extents), `0.6 m` high,
/// with an opening on the south side at `x 6.0..7.0 m`.
pub const REFERENCE_WATER_BASIN_RIM_BODY_ID: PhysicsBodyIdV1 = PhysicsBodyIdV1 {
    subject_id: PersistentId::from_bytes([0x89; 16]),
    body_slot: 0,
};
pub const REFERENCE_WATER_BASIN_RIM_BOXES_MICROMETRES: [([i64; 3], [i64; 3]); 5] = [
    ([4425000, 300000, 2000000], [75000, 300000, 1150000]),
    ([8575000, 300000, 2000000], [75000, 300000, 1150000]),
    ([6500000, 300000, 3075000], [2150000, 300000, 75000]),
    ([5175000, 300000, 925000], [825000, 300000, 75000]),
    ([7825000, 300000, 925000], [825000, 300000, 75000]),
];
pub const REFERENCE_WATER_BASIN_PROFILE_REVISION: u32 = 1;

/// Plan 32: the pond sunk into the ground north of the spawn, `10 x 4.5 m`
/// and `1.4 m` deep under a level `10 cm` below the ground; the first body
/// of the scene that holds the third-person camera under the water.
pub const REFERENCE_WATER_POND_ID: PersistentId = PersistentId::from_bytes([0x9a; 16]);
pub const REFERENCE_WATER_POND_MINIMUM_MICROMETRES: [i64; 3] = [-9_000_000, -1_500_000, 5_000_000];
pub const REFERENCE_WATER_POND_MAXIMUM_MICROMETRES: [i64; 3] = [1_000_000, 0, 9_500_000];
pub const REFERENCE_WATER_POND_INITIAL_LEVEL_MICROMETRES: i64 = -100_000;
pub const REFERENCE_WATER_POND_SURFACE_OBJECT_ID: PersistentId =
    PersistentId::from_bytes([0x9c; 16]);
/// Plan 32: the pond floor and the five full-width steps at the west end
/// (centre, half extents), one static body.
pub const REFERENCE_WATER_POND_BODY_ID: PhysicsBodyIdV1 = PhysicsBodyIdV1 {
    subject_id: PersistentId::from_bytes([0x9b; 16]),
    body_slot: 0,
};
pub const REFERENCE_WATER_POND_BOXES_MICROMETRES: [([i64; 3], [i64; 3]); 6] = [
    (
        [-4_000_000, -1_600_000, 7_250_000],
        [5_000_000, 100_000, 2_250_000],
    ),
    (
        [-8_750_000, -875_000, 7_250_000],
        [250_000, 625_000, 2_250_000],
    ),
    (
        [-8_250_000, -1_000_000, 7_250_000],
        [250_000, 500_000, 2_250_000],
    ),
    (
        [-7_750_000, -1_125_000, 7_250_000],
        [250_000, 375_000, 2_250_000],
    ),
    (
        [-7_250_000, -1_250_000, 7_250_000],
        [250_000, 250_000, 2_250_000],
    ),
    (
        [-6_750_000, -1_375_000, 7_250_000],
        [250_000, 125_000, 2_250_000],
    ),
];
/// Plan 32: the ground as four strips around the pond hole (centre, half
/// extents), solid from the pond floor to the ground.
pub const REFERENCE_GROUND_STRIPS_MICROMETRES: [([i64; 3], [i64; 3]); 4] = [
    ([0, -850_000, -2_500_000], [10_000_000, 850_000, 7_500_000]),
    ([0, -850_000, 9_750_000], [10_000_000, 850_000, 250_000]),
    (
        [-9_500_000, -850_000, 7_250_000],
        [500_000, 850_000, 2_250_000],
    ),
    (
        [5_500_000, -850_000, 7_250_000],
        [4_500_000, 850_000, 2_250_000],
    ),
];

/// ADR-103 first flow consumer (plan `continuum-water/07`): vessel A on a
/// `1 m` shelf, `2 x 1.5 m` in plan, filled to `1.5 m`; vessel B on the
/// floor, `2.8 x 1.5 m`, empty; a gated `0.04 m^2` pipe with its invert
/// `0.6 m` above B's floor; a `0.5 L/s` source into A and sink from B.
/// Both vessels sit east of the basin, away from the locomotion walk.
pub const REFERENCE_WATER_VESSEL_A_ID: PersistentId = PersistentId::from_bytes([0x7e; 16]);
pub const REFERENCE_WATER_VESSEL_B_ID: PersistentId = PersistentId::from_bytes([0x7f; 16]);
pub const REFERENCE_WATER_FLOW_GATE_ID: PersistentId = PersistentId::from_bytes([0x80; 16]);
pub const REFERENCE_WATER_FLOW_SOURCE_ID: PersistentId = PersistentId::from_bytes([0x81; 16]);
pub const REFERENCE_WATER_FLOW_SINK_ID: PersistentId = PersistentId::from_bytes([0x82; 16]);
/// The runtime tick rate the flow step integrates at (`TickRateProfileV1::at_30_hz`).
pub const REFERENCE_WATER_FLOW_TICKS_PER_SECOND: u32 = 30;
const VESSEL_A_MINIMUM_MICROMETRES: [i64; 3] = [12_000_000, 1_000_000, 1_000_000];
const VESSEL_A_MAXIMUM_MICROMETRES: [i64; 3] = [14_000_000, 3_000_000, 2_500_000];
const VESSEL_A_INITIAL_LEVEL_MICROMETRES: i64 = 1_500_000;
const VESSEL_B_MINIMUM_MICROMETRES: [i64; 3] = [15_000_000, 0, 1_000_000];
const VESSEL_B_MAXIMUM_MICROMETRES: [i64; 3] = [17_800_000, 2_000_000, 2_500_000];
const VESSEL_B_INITIAL_LEVEL_MICROMETRES: i64 = 0;
const FLOW_GATE_INVERT_MICROMETRES: i64 = 600_000;
const FLOW_GATE_AREA_SQUARE_MILLIMETRES: i64 = 40_000;
/// NGQ8 calibration: short flush opening.
const FLOW_GATE_COEFFICIENT_PERMILLE: u32 = 400;
const FLOW_SOURCE_RATE_CUBIC_MILLIMETRES_PER_SECOND: i64 = 500_000;
const FLOW_SINK_RATE_CUBIC_MILLIMETRES_PER_SECOND: i64 = 500_000;

/// The two flow vessels of the reference scene.
#[must_use]
pub fn reference_water_vessel_definitions() -> [WaterVolumeDefinitionV1; 2] {
    [
        WaterVolumeDefinitionV1 {
            volume_id: REFERENCE_WATER_VESSEL_A_ID,
            minimum_micrometres: VESSEL_A_MINIMUM_MICROMETRES,
            maximum_micrometres: VESSEL_A_MAXIMUM_MICROMETRES,
            initial_level_micrometres: VESSEL_A_INITIAL_LEVEL_MICROMETRES,
            swimming_depth_micrometres: REFERENCE_WATER_BASIN_SWIMMING_DEPTH_MICROMETRES,
            level_ramp: None,
            profile_revision: REFERENCE_WATER_BASIN_PROFILE_REVISION,
        },
        WaterVolumeDefinitionV1 {
            volume_id: REFERENCE_WATER_VESSEL_B_ID,
            minimum_micrometres: VESSEL_B_MINIMUM_MICROMETRES,
            maximum_micrometres: VESSEL_B_MAXIMUM_MICROMETRES,
            initial_level_micrometres: VESSEL_B_INITIAL_LEVEL_MICROMETRES,
            swimming_depth_micrometres: REFERENCE_WATER_BASIN_SWIMMING_DEPTH_MICROMETRES,
            level_ramp: None,
            profile_revision: REFERENCE_WATER_BASIN_PROFILE_REVISION,
        },
    ]
}

/// The authored edges of the reference flow network.
#[must_use]
pub fn reference_water_flow_edges() -> [WaterFlowEdgeV1; 3] {
    [
        WaterFlowEdgeV1 {
            edge_id: REFERENCE_WATER_FLOW_GATE_ID,
            cell_a: REFERENCE_WATER_VESSEL_A_ID,
            cell_b: Some(REFERENCE_WATER_VESSEL_B_ID),
            kind: WaterFlowEdgeKindV1::Gate {
                invert_micrometres: FLOW_GATE_INVERT_MICROMETRES,
                area_square_millimetres: FLOW_GATE_AREA_SQUARE_MILLIMETRES,
                coefficient_permille: FLOW_GATE_COEFFICIENT_PERMILLE,
                initial_opening_permille: 1000,
            },
        },
        WaterFlowEdgeV1 {
            edge_id: REFERENCE_WATER_FLOW_SOURCE_ID,
            cell_a: REFERENCE_WATER_VESSEL_A_ID,
            cell_b: None,
            kind: WaterFlowEdgeKindV1::Source {
                rate_cubic_millimetres_per_second: FLOW_SOURCE_RATE_CUBIC_MILLIMETRES_PER_SECOND,
            },
        },
        WaterFlowEdgeV1 {
            edge_id: REFERENCE_WATER_FLOW_SINK_ID,
            cell_a: REFERENCE_WATER_VESSEL_B_ID,
            cell_b: None,
            kind: WaterFlowEdgeKindV1::Sink {
                rate_cubic_millimetres_per_second: FLOW_SINK_RATE_CUBIC_MILLIMETRES_PER_SECOND,
            },
        },
    ]
}

/// The genesis flow network over the reference water table.
pub fn reference_water_flow(
    volumes: &WaterVolumeSetV1,
) -> Result<WaterFlowNetworkV1, ReferenceGameError> {
    WaterFlowNetworkV1::from_edges(
        REFERENCE_WATER_FLOW_TICKS_PER_SECOND,
        reference_water_flow_edges(),
        volumes,
    )
    .map_err(ReferenceGameError::Physics)
}

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
    Ok(WaterVolumeSetV1::from_definitions(
        [reference_water_basin_definition()]
            .into_iter()
            .chain(reference_water_vessel_definitions())
            .chain([reference_water_pond_definition()]),
    )?)
}

/// Plan 32: the sunken pond, a still body outside the flow network.
#[must_use]
pub fn reference_water_pond_definition() -> WaterVolumeDefinitionV1 {
    WaterVolumeDefinitionV1 {
        volume_id: REFERENCE_WATER_POND_ID,
        minimum_micrometres: REFERENCE_WATER_POND_MINIMUM_MICROMETRES,
        maximum_micrometres: REFERENCE_WATER_POND_MAXIMUM_MICROMETRES,
        initial_level_micrometres: REFERENCE_WATER_POND_INITIAL_LEVEL_MICROMETRES,
        swimming_depth_micrometres: REFERENCE_WATER_BASIN_SWIMMING_DEPTH_MICROMETRES,
        level_ramp: None,
        profile_revision: REFERENCE_WATER_BASIN_PROFILE_REVISION,
    }
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

/// Presentation object ids of the vessel surface quads (plan 09).
pub const REFERENCE_WATER_VESSEL_A_SURFACE_OBJECT_ID: PersistentId =
    PersistentId::from_bytes([0x84; 16]);
pub const REFERENCE_WATER_VESSEL_B_SURFACE_OBJECT_ID: PersistentId =
    PersistentId::from_bytes([0x85; 16]);

/// World translation of an authored surface quad (authored at local
/// `y = 0`): the exact effective level of one volume at `tick`.
pub fn volume_surface_translation(
    water: &WaterVolumeSetV1,
    volume_id: PersistentId,
    tick: u64,
) -> Result<[i64; 3], ReferenceGameError> {
    let level = water
        .effective_level(volume_id, tick)
        .ok_or(ReferenceGameError::PresentationAssetMissing)?;
    Ok([0, level, 0])
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

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::physics::{CAPSULE_MAX_STEP_HEIGHT_MICROMETRES, WaterSubmersionClassV1};

    /// Plan 32 G2: the pond's classification at its floor, its top step and
    /// the ground beside it.
    #[test]
    fn pond_classifies_swimming_on_the_floor_and_wading_on_the_top_step() {
        let volumes = reference_water_volumes().expect("reference volumes");
        let floor = volumes.submersion_at([-4_500_000, -1_500_000, 7_250_000], 0);
        assert_eq!(floor.class, WaterSubmersionClassV1::Swimming);
        assert_eq!(floor.volume_id, Some(REFERENCE_WATER_POND_ID));
        assert_eq!(floor.depth_micrometres, 1_400_000);
        let top_step = volumes.submersion_at([-8_750_000, -250_000, 7_250_000], 0);
        assert_eq!(top_step.class, WaterSubmersionClassV1::Wading);
        let ground = volumes.submersion_at([-4_500_000, 0, 4_000_000], 0);
        assert_eq!(ground.class, WaterSubmersionClassV1::Dry);
    }

    /// Plan 32 G3: step rises within the capsule's limit, the hole inside
    /// the ground strips, no overlap with the other bodies.
    #[test]
    fn pond_steps_and_plan_fit_the_ground_and_the_other_bodies() {
        let mut tops: Vec<i64> = REFERENCE_WATER_POND_BOXES_MICROMETRES[1..]
            .iter()
            .map(|(centre, half)| centre[1] + half[1])
            .collect();
        tops.sort_unstable();
        let mut previous = REFERENCE_WATER_POND_MINIMUM_MICROMETRES[1];
        for top in tops.iter().copied().chain([0]) {
            assert!(top > previous && top - previous <= CAPSULE_MAX_STEP_HEIGHT_MICROMETRES);
            previous = top;
        }
        let floor_top = REFERENCE_WATER_POND_BOXES_MICROMETRES[0].0[1]
            + REFERENCE_WATER_POND_BOXES_MICROMETRES[0].1[1];
        assert_eq!(floor_top, REFERENCE_WATER_POND_MINIMUM_MICROMETRES[1]);
        let pond_min = REFERENCE_WATER_POND_MINIMUM_MICROMETRES;
        let pond_max = REFERENCE_WATER_POND_MAXIMUM_MICROMETRES;
        assert!(pond_min[0] > -10_000_000 && pond_max[0] < 10_000_000);
        assert!(pond_min[2] > -10_000_000 && pond_max[2] < 10_000_000);
        let disjoint = |other: &WaterVolumeDefinitionV1| {
            pond_max[0] <= other.minimum_micrometres[0]
                || other.maximum_micrometres[0] <= pond_min[0]
                || pond_max[2] <= other.minimum_micrometres[2]
                || other.maximum_micrometres[2] <= pond_min[2]
        };
        assert!(disjoint(&reference_water_basin_definition()));
        for vessel in reference_water_vessel_definitions() {
            assert!(disjoint(&vessel));
        }
        // Every ground strip stays outside the hole and inside the floor,
        // reaching the ground at `y 0` and the pond floor below.
        for (centre, half) in REFERENCE_GROUND_STRIPS_MICROMETRES {
            let (x0, x1) = (centre[0] - half[0], centre[0] + half[0]);
            let (z0, z1) = (centre[2] - half[2], centre[2] + half[2]);
            assert!(x0 >= -10_000_000 && x1 <= 10_000_000 && z0 >= -10_000_000 && z1 <= 10_000_000);
            assert!(
                x1 <= pond_min[0] || x0 >= pond_max[0] || z1 <= pond_min[2] || z0 >= pond_max[2]
            );
            assert_eq!(centre[1] + half[1], 0);
            assert!(centre[1] - half[1] <= pond_min[1]);
        }
    }
}
