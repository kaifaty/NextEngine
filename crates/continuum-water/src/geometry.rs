#![forbid(unsafe_code)]

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{NUMERIC_OVERFLOW, SCENARIO_INVALID, WaterError};
use crate::model::{Aperture, Geometry, Vec3i};

pub(crate) const OUTER_X_MIN_FEATURE_ID: u32 = 0;
pub(crate) const OUTER_X_MAX_FEATURE_ID: u32 = 1;
pub(crate) const OUTER_Y_MIN_FEATURE_ID: u32 = 2;
pub(crate) const OUTER_Y_MAX_FEATURE_ID: u32 = 3;
pub(crate) const OUTER_Z_MIN_FEATURE_ID: u32 = 4;
pub(crate) const OUTER_Z_MAX_FEATURE_ID: u32 = 5;
pub(crate) const INTERNAL_PATCH_FEATURE_ID: u32 = 16;
pub(crate) const APERTURE_Y_MIN_EDGE_FEATURE_ID: u32 = 17;
pub(crate) const APERTURE_Y_MAX_EDGE_FEATURE_ID: u32 = 18;
pub(crate) const APERTURE_Z_MIN_EDGE_FEATURE_ID: u32 = 19;
pub(crate) const APERTURE_Z_MAX_EDGE_FEATURE_ID: u32 = 20;
pub(crate) const APERTURE_MIN_MIN_CORNER_FEATURE_ID: u32 = 21;
pub(crate) const APERTURE_MIN_MAX_CORNER_FEATURE_ID: u32 = 22;
pub(crate) const APERTURE_MAX_MIN_CORNER_FEATURE_ID: u32 = 23;
pub(crate) const APERTURE_MAX_MAX_CORNER_FEATURE_ID: u32 = 24;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum PlaneAxis {
    X,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RectangularOpening {
    pub(crate) opening_id: u32,
    pub(crate) y_min_um: i64,
    pub(crate) y_max_um: i64,
    pub(crate) z_min_um: i64,
    pub(crate) z_max_um: i64,
}

impl RectangularOpening {
    pub(crate) fn contains_closed(self, y_um: i64, z_um: i64) -> bool {
        y_um >= self.y_min_um
            && y_um <= self.y_max_um
            && z_um >= self.z_min_um
            && z_um <= self.z_max_um
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct InternalPlanePatch {
    pub(crate) feature_id: u32,
    pub(crate) axis: PlaneAxis,
    pub(crate) coordinate_um: i64,
    pub(crate) y_min_um: i64,
    pub(crate) y_max_um: i64,
    pub(crate) z_min_um: i64,
    pub(crate) z_max_um: i64,
    pub(crate) opening: RectangularOpening,
}

impl InternalPlanePatch {
    pub(crate) fn projection_is_solid(self, y_um: i64, z_um: i64) -> bool {
        y_um >= self.y_min_um
            && y_um <= self.y_max_um
            && z_um >= self.z_min_um
            && z_um <= self.z_max_um
            && !self.opening.contains_closed(y_um, z_um)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct AxisAlignedGeometryManifest {
    pub(crate) schema: &'static str,
    pub(crate) bounds_min_um: Vec3i,
    pub(crate) bounds_max_um: Vec3i,
    pub(crate) internal_plane_patches: Vec<InternalPlanePatch>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct GeometryObservation {
    pub(crate) position_um: Vec3i,
    pub(crate) classification: String,
    pub(crate) closest_feature_id: u32,
    pub(crate) closest_distance_squared_um2: i128,
}

impl AxisAlignedGeometryManifest {
    pub(crate) fn from_geometry(geometry: Geometry) -> Result<Self, WaterError> {
        let mut internal_plane_patches = Vec::new();
        if let Some(aperture) = geometry.aperture {
            internal_plane_patches
                .try_reserve_exact(1)
                .map_err(|error| {
                    WaterError::new(
                        SCENARIO_INVALID,
                        format!("cannot reserve internal geometry manifest: {error}"),
                    )
                })?;
            internal_plane_patches.push(patch_from_aperture(geometry, aperture)?);
        }
        let manifest = Self {
            schema: "nextengine.continuum-water.axis-aligned-geometry.v1",
            bounds_min_um: geometry.bounds.min,
            bounds_max_um: geometry.bounds.max,
            internal_plane_patches,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub(crate) fn validate(&self) -> Result<(), WaterError> {
        if self.bounds_min_um.x >= self.bounds_max_um.x
            || self.bounds_min_um.y >= self.bounds_max_um.y
            || self.bounds_min_um.z >= self.bounds_max_um.z
        {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                "axis-aligned geometry has an empty outer box",
            ));
        }
        let mut prior_id = None;
        for patch in &self.internal_plane_patches {
            if prior_id.is_some_and(|prior| prior >= patch.feature_id) {
                return Err(WaterError::new(
                    SCENARIO_INVALID,
                    "internal plane feature IDs are not strictly ascending",
                ));
            }
            prior_id = Some(patch.feature_id);
            if patch.axis != PlaneAxis::X
                || patch.coordinate_um <= self.bounds_min_um.x
                || patch.coordinate_um >= self.bounds_max_um.x
                || patch.y_min_um != self.bounds_min_um.y
                || patch.y_max_um != self.bounds_max_um.y
                || patch.z_min_um != self.bounds_min_um.z
                || patch.z_max_um != self.bounds_max_um.z
            {
                return Err(WaterError::new(
                    SCENARIO_INVALID,
                    format!(
                        "internal plane feature {} is outside the supported W0F manifest",
                        patch.feature_id
                    ),
                ));
            }
            let opening = patch.opening;
            if opening.y_min_um >= opening.y_max_um
                || opening.z_min_um >= opening.z_max_um
                || opening.y_min_um <= patch.y_min_um
                || opening.y_max_um >= patch.y_max_um
                || opening.z_min_um <= patch.z_min_um
                || opening.z_max_um >= patch.z_max_um
            {
                return Err(WaterError::new(
                    SCENARIO_INVALID,
                    format!(
                        "opening {} is not strictly inside plane feature {}",
                        opening.opening_id, patch.feature_id
                    ),
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn internal_patch(&self) -> Option<InternalPlanePatch> {
        self.internal_plane_patches.first().copied()
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        let mut text = String::new();
        text.push_str("GEOMETRY_MANIFEST_V1_BEGIN\n");
        text.push_str(&format!(
            "outer.feature_ids={},{},{},{},{},{}\n",
            OUTER_X_MIN_FEATURE_ID,
            OUTER_X_MAX_FEATURE_ID,
            OUTER_Y_MIN_FEATURE_ID,
            OUTER_Y_MAX_FEATURE_ID,
            OUTER_Z_MIN_FEATURE_ID,
            OUTER_Z_MAX_FEATURE_ID
        ));
        text.push_str(&format!(
            "outer.box_um=({},{},{})..({},{},{})\n",
            self.bounds_min_um.x,
            self.bounds_min_um.y,
            self.bounds_min_um.z,
            self.bounds_max_um.x,
            self.bounds_max_um.y,
            self.bounds_max_um.z
        ));
        for patch in &self.internal_plane_patches {
            text.push_str(&format!(
                "patch.{}.axis=X;coordinate_um={};y={}..{};z={}..{}\n",
                patch.feature_id,
                patch.coordinate_um,
                patch.y_min_um,
                patch.y_max_um,
                patch.z_min_um,
                patch.z_max_um
            ));
            text.push_str(&format!(
                "patch.{}.opening.{}=y={}..{};z={}..{};membership=closed\n",
                patch.feature_id,
                patch.opening.opening_id,
                patch.opening.y_min_um,
                patch.opening.y_max_um,
                patch.opening.z_min_um,
                patch.opening.z_max_um
            ));
            text.push_str(&format!(
                "patch.{}.edge_feature_ids={},{},{},{};corner_feature_ids={},{},{},{}\n",
                patch.feature_id,
                APERTURE_Y_MIN_EDGE_FEATURE_ID,
                APERTURE_Y_MAX_EDGE_FEATURE_ID,
                APERTURE_Z_MIN_EDGE_FEATURE_ID,
                APERTURE_Z_MAX_EDGE_FEATURE_ID,
                APERTURE_MIN_MIN_CORNER_FEATURE_ID,
                APERTURE_MIN_MAX_CORNER_FEATURE_ID,
                APERTURE_MAX_MIN_CORNER_FEATURE_ID,
                APERTURE_MAX_MAX_CORNER_FEATURE_ID
            ));
        }
        text.push_str("GEOMETRY_MANIFEST_V1_END\n");
        text.into_bytes()
    }

    pub(crate) fn root(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"nextengine.continuum-water.geometry-manifest.v1\0");
        hasher.update(self.canonical_bytes());
        hasher.finalize().into()
    }

    pub(crate) fn observe(&self, position: Vec3i) -> Result<GeometryObservation, WaterError> {
        let outside = position.x < self.bounds_min_um.x
            || position.x > self.bounds_max_um.x
            || position.y < self.bounds_min_um.y
            || position.y > self.bounds_max_um.y
            || position.z < self.bounds_min_um.z
            || position.z > self.bounds_max_um.z;
        let classification = if outside {
            "OUTER_SOLID"
        } else if let Some(patch) = self.internal_patch() {
            if position.x == patch.coordinate_um {
                if patch.opening.contains_closed(position.y, position.z) {
                    "OPENING"
                } else {
                    "INTERNAL_SOLID"
                }
            } else {
                "FLUID_DOMAIN"
            }
        } else {
            "FLUID_DOMAIN"
        };

        let outer_candidates = [
            (
                OUTER_X_MIN_FEATURE_ID,
                absolute_difference(position.x, self.bounds_min_um.x),
            ),
            (
                OUTER_X_MAX_FEATURE_ID,
                absolute_difference(position.x, self.bounds_max_um.x),
            ),
            (
                OUTER_Y_MIN_FEATURE_ID,
                absolute_difference(position.y, self.bounds_min_um.y),
            ),
            (
                OUTER_Y_MAX_FEATURE_ID,
                absolute_difference(position.y, self.bounds_max_um.y),
            ),
            (
                OUTER_Z_MIN_FEATURE_ID,
                absolute_difference(position.z, self.bounds_min_um.z),
            ),
            (
                OUTER_Z_MAX_FEATURE_ID,
                absolute_difference(position.z, self.bounds_max_um.z),
            ),
        ];
        let mut closest = (u32::MAX, i128::MAX);
        for (feature_id, distance) in outer_candidates {
            select_closest(&mut closest, feature_id, square(distance)?)?;
        }
        if let Some(patch) = self.internal_patch() {
            let x_distance = absolute_difference(position.x, patch.coordinate_um);
            let x_squared = square(x_distance)?;
            if patch.opening.contains_closed(position.y, position.z) {
                for (feature_id, distance) in [
                    (
                        APERTURE_Y_MIN_EDGE_FEATURE_ID,
                        absolute_difference(position.y, patch.opening.y_min_um),
                    ),
                    (
                        APERTURE_Y_MAX_EDGE_FEATURE_ID,
                        absolute_difference(position.y, patch.opening.y_max_um),
                    ),
                    (
                        APERTURE_Z_MIN_EDGE_FEATURE_ID,
                        absolute_difference(position.z, patch.opening.z_min_um),
                    ),
                    (
                        APERTURE_Z_MAX_EDGE_FEATURE_ID,
                        absolute_difference(position.z, patch.opening.z_max_um),
                    ),
                ] {
                    let distance_squared =
                        x_squared.checked_add(square(distance)?).ok_or_else(|| {
                            WaterError::new(NUMERIC_OVERFLOW, "geometry feature distance overflow")
                        })?;
                    select_closest(&mut closest, feature_id, distance_squared)?;
                }
            } else if position.y >= patch.y_min_um
                && position.y <= patch.y_max_um
                && position.z >= patch.z_min_um
                && position.z <= patch.z_max_um
            {
                select_closest(&mut closest, patch.feature_id, x_squared)?;
            }
        }
        Ok(GeometryObservation {
            position_um: position,
            classification: classification.to_owned(),
            closest_feature_id: closest.0,
            closest_distance_squared_um2: closest.1,
        })
    }

    /// Returns true when the centre-to-centre segment crosses the solid part
    /// of an internal patch. The wall intersection is evaluated as an exact
    /// rational number; no floating-point epsilon participates in admission.
    pub(crate) fn blocks_segment(&self, start: Vec3i, end: Vec3i) -> Result<bool, WaterError> {
        for patch in &self.internal_plane_patches {
            let start_side = start
                .x
                .checked_sub(patch.coordinate_um)
                .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "geometry start-side overflow"))?;
            let end_side = end
                .x
                .checked_sub(patch.coordinate_um)
                .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "geometry end-side overflow"))?;
            if !((start_side < 0 && end_side > 0)
                || (start_side > 0 && end_side < 0)
                || start_side == 0
                || end_side == 0)
            {
                continue;
            }
            if start_side == 0 && end_side == 0 {
                if patch.projection_is_solid(start.y, start.z)
                    || patch.projection_is_solid(end.y, end.z)
                {
                    return Ok(true);
                }
                continue;
            }
            let denominator = i128::from(end.x)
                .checked_sub(i128::from(start.x))
                .ok_or_else(|| {
                    WaterError::new(NUMERIC_OVERFLOW, "geometry crossing denominator overflow")
                })?;
            if denominator == 0 {
                continue;
            }
            let numerator = i128::from(patch.coordinate_um)
                .checked_sub(i128::from(start.x))
                .ok_or_else(|| {
                    WaterError::new(NUMERIC_OVERFLOW, "geometry crossing numerator overflow")
                })?;
            let mut denominator = denominator;
            let mut numerator = numerator;
            if denominator < 0 {
                denominator = denominator.checked_neg().ok_or_else(|| {
                    WaterError::new(NUMERIC_OVERFLOW, "geometry denominator negation overflow")
                })?;
                numerator = numerator.checked_neg().ok_or_else(|| {
                    WaterError::new(NUMERIC_OVERFLOW, "geometry numerator negation overflow")
                })?;
            }
            let y_numerator = interpolate_numerator(start.y, end.y, numerator, denominator)?;
            let z_numerator = interpolate_numerator(start.z, end.z, numerator, denominator)?;
            let opening = patch.opening;
            let inside_opening = rational_in_closed_range(
                y_numerator,
                opening.y_min_um,
                opening.y_max_um,
                denominator,
            )? && rational_in_closed_range(
                z_numerator,
                opening.z_min_um,
                opening.z_max_um,
                denominator,
            )?;
            if !inside_opening {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn absolute_difference(left: i64, right: i64) -> i128 {
    (i128::from(left) - i128::from(right)).abs()
}

fn square(value: i128) -> Result<i128, WaterError> {
    value
        .checked_mul(value)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "geometry squared distance overflow"))
}

fn select_closest(
    closest: &mut (u32, i128),
    feature_id: u32,
    distance_squared: i128,
) -> Result<(), WaterError> {
    if distance_squared < 0 {
        return Err(WaterError::new(
            NUMERIC_OVERFLOW,
            "geometry distance is negative",
        ));
    }
    if distance_squared < closest.1 || (distance_squared == closest.1 && feature_id < closest.0) {
        *closest = (feature_id, distance_squared);
    }
    Ok(())
}

fn patch_from_aperture(
    geometry: Geometry,
    aperture: Aperture,
) -> Result<InternalPlanePatch, WaterError> {
    let patch = InternalPlanePatch {
        feature_id: INTERNAL_PATCH_FEATURE_ID,
        axis: PlaneAxis::X,
        coordinate_um: aperture.wall_x_um,
        y_min_um: geometry.bounds.min.y,
        y_max_um: geometry.bounds.max.y,
        z_min_um: geometry.bounds.min.z,
        z_max_um: geometry.bounds.max.z,
        opening: RectangularOpening {
            opening_id: INTERNAL_PATCH_FEATURE_ID + 1,
            y_min_um: aperture.y_min_um,
            y_max_um: aperture.y_max_um,
            z_min_um: aperture.z_min_um,
            z_max_um: aperture.z_max_um,
        },
    };
    Ok(patch)
}

fn interpolate_numerator(
    start: i64,
    end: i64,
    numerator: i128,
    denominator: i128,
) -> Result<i128, WaterError> {
    let start_term = i128::from(start)
        .checked_mul(denominator)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "geometry start term overflow"))?;
    let delta = i128::from(end)
        .checked_sub(i128::from(start))
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "geometry delta overflow"))?;
    start_term
        .checked_add(
            delta
                .checked_mul(numerator)
                .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "geometry delta term overflow"))?,
        )
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "geometry interpolation overflow"))
}

fn rational_in_closed_range(
    value_numerator: i128,
    minimum: i64,
    maximum: i64,
    denominator: i128,
) -> Result<bool, WaterError> {
    let minimum = i128::from(minimum)
        .checked_mul(denominator)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "geometry range minimum overflow"))?;
    let maximum = i128::from(maximum)
        .checked_mul(denominator)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "geometry range maximum overflow"))?;
    Ok(value_numerator >= minimum && value_numerator <= maximum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario;

    #[test]
    fn closed_opening_is_the_only_unblocked_crossing_set() {
        let geometry = scenario::find("CW-ORIFICE-001").unwrap().geometry;
        let manifest = AxisAlignedGeometryManifest::from_geometry(geometry).unwrap();
        assert!(
            !manifest
                .blocks_segment(
                    Vec3i::new(975_000, 300_000, 500_000),
                    Vec3i::new(1_025_000, 300_000, 500_000),
                )
                .unwrap()
        );
        assert!(
            !manifest
                .blocks_segment(
                    Vec3i::new(975_000, 200_000, 400_000),
                    Vec3i::new(1_025_000, 200_000, 400_000),
                )
                .unwrap()
        );
        assert!(
            manifest
                .blocks_segment(
                    Vec3i::new(975_000, 199_999, 400_000),
                    Vec3i::new(1_025_000, 199_999, 400_000),
                )
                .unwrap()
        );
    }

    #[test]
    fn rational_crossing_does_not_round_into_the_opening() {
        let geometry = scenario::find("CW-ORIFICE-001").unwrap().geometry;
        let manifest = AxisAlignedGeometryManifest::from_geometry(geometry).unwrap();
        assert!(
            manifest
                .blocks_segment(
                    Vec3i::new(950_000, 199_999, 500_000),
                    Vec3i::new(1_100_000, 200_001, 500_000),
                )
                .unwrap()
        );
    }
}
