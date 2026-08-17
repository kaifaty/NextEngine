#![forbid(unsafe_code)]

use crate::error::{AUDIT_INVALID, WaterError};
use crate::kernel;
use crate::model::{Geometry, Vec3f, checked_scalar};
use crate::profile::{H2, H3, KERNEL_K, PARTICLE_RADIUS, SUPPORT_RADIUS, decode_micrometres};

/// Analytical, immutable form of the Bender et al. 2019 volume-map query used
/// by the W0C discriminator. The regular-grid interpolation from the reference
/// implementation is intentionally replaced by the exact signed-distance
/// field of the selected axis-aligned box; all remaining quadrature and
/// virtual-sample operations retain the pinned reference order.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VolumeMapBoundary {
    minimum: Vec3f,
    maximum: Vec3f,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct VolumeMapSample {
    pub(crate) signed_distance: f64,
    pub(crate) volume: f64,
    pub(crate) virtual_distance: f64,
    pub(crate) displacement: Vec3f,
    pub(crate) value: f64,
    pub(crate) gradient: Vec3f,
    pub(crate) feature_rank: usize,
}

const VOLUME_SCALE: f64 = f64::from_bits(0x3fe9_9999_9999_999a);

// The pinned SPlisHSPlasH GaussQuadrature table maps polynomial degree 30 to
// sixteen one-dimensional points. Values are frozen by IEEE-754 bit pattern.
const GAUSS_NODES: [f64; 16] = [
    f64::from_bits(0xbfef_a92c_264d_787e),
    f64::from_bits(0xbfee_39f5_6616_f9b0),
    f64::from_bits(0xbfeb_b340_3514_e483),
    f64::from_bits(0xbfe8_2c45_dda4_726b),
    f64::from_bits(0xbfe3_c5a4_66d5_e8b8),
    f64::from_bits(0xbfdd_5025_9a43_a772),
    f64::from_bits(0xbfd2_05ca_e642_337c),
    f64::from_bits(0xbfb8_52bd_6676_a9f9),
    f64::from_bits(0x3fb8_52bd_6676_a9f9),
    f64::from_bits(0x3fd2_05ca_e642_337c),
    f64::from_bits(0x3fdd_5025_9a43_a772),
    f64::from_bits(0x3fe3_c5a4_66d5_e8b8),
    f64::from_bits(0x3fe8_2c45_dda4_726b),
    f64::from_bits(0x3feb_b340_3514_e483),
    f64::from_bits(0x3fee_39f5_6616_f9b0),
    f64::from_bits(0x3fef_a92c_264d_787e),
];

const GAUSS_WEIGHTS: [f64; 16] = [
    f64::from_bits(0x3f9b_cdda_b4b7_c211),
    f64::from_bits(0x3faf_dfb1_a2c1_265c),
    f64::from_bits(0x3fb8_5c4e_e79c_c258),
    f64::from_bits(0x3fbf_e7af_2bad_386a),
    f64::from_bits(0x3fc3_25f6_1bca_3cbf),
    f64::from_bits(0x3fc5_a6eb_bb5a_75fc),
    f64::from_bits(0x3fc7_5f8c_77e0_c00f),
    f64::from_bits(0x3fc8_3fea_e80e_4dfc),
    f64::from_bits(0x3fc8_3fea_e80e_4dfc),
    f64::from_bits(0x3fc7_5f8c_77e0_c00f),
    f64::from_bits(0x3fc5_a6eb_bb5a_75fc),
    f64::from_bits(0x3fc3_25f6_1bca_3cbf),
    f64::from_bits(0x3fbf_e7af_2bad_386a),
    f64::from_bits(0x3fb8_5c4e_e79c_c258),
    f64::from_bits(0x3faf_dfb1_a2c1_265c),
    f64::from_bits(0x3f9b_cdda_b4b7_c211),
];

impl VolumeMapBoundary {
    pub(crate) fn new(geometry: Geometry) -> Result<Self, WaterError> {
        if geometry.aperture.is_some() {
            return Err(WaterError::new(
                AUDIT_INVALID,
                "volume-map-box-bender2019-ref-v1 does not define aperture composition",
            ));
        }
        Ok(Self {
            minimum: Vec3f::new(
                decode_micrometres(geometry.bounds.min.x)?,
                decode_micrometres(geometry.bounds.min.y)?,
                decode_micrometres(geometry.bounds.min.z)?,
            ),
            maximum: Vec3f::new(
                decode_micrometres(geometry.bounds.max.x)?,
                decode_micrometres(geometry.bounds.max.y)?,
                decode_micrometres(geometry.bounds.max.z)?,
            ),
        })
    }

    pub(crate) fn sample(self, position: Vec3f) -> Result<Option<VolumeMapSample>, WaterError> {
        let (signed_distance, normal, feature_rank) = self.distance_normal(position)?;
        if signed_distance <= 0.0 || signed_distance >= SUPPORT_RADIUS {
            return Ok(None);
        }
        let volume = self.integrated_volume(position)?;
        if volume <= 0.0 {
            return Ok(None);
        }
        let offset_distance = checked_scalar(
            signed_distance + (0.5 * PARTICLE_RADIUS),
            "volume-map virtual offset distance",
        )?;
        let minimum_distance =
            checked_scalar(2.0 * PARTICLE_RADIUS, "volume-map minimum virtual distance")?;
        let virtual_distance = offset_distance.max(minimum_distance);
        let displacement = normal
            .scale(virtual_distance)
            .checked("volume-map virtual displacement")?;
        let sampled = kernel::sample_metres(displacement)?;
        Ok(Some(VolumeMapSample {
            signed_distance,
            volume,
            virtual_distance,
            displacement,
            value: sampled.value,
            gradient: sampled.gradient,
            feature_rank,
        }))
    }

    fn integrated_volume(self, position: Vec3f) -> Result<f64, WaterError> {
        let mut result = 0.0;
        for (i, node_x) in GAUSS_NODES.iter().copied().enumerate() {
            let offset_x = checked_scalar(SUPPORT_RADIUS * node_x, "volume-map quadrature x")?;
            let sample_x = checked_scalar(position.x + offset_x, "volume-map sample x")?;
            let weight_i = GAUSS_WEIGHTS[i];
            for (j, node_y) in GAUSS_NODES.iter().copied().enumerate() {
                let offset_y = checked_scalar(SUPPORT_RADIUS * node_y, "volume-map quadrature y")?;
                let sample_y = checked_scalar(position.y + offset_y, "volume-map sample y")?;
                let weight_ij = checked_scalar(
                    weight_i * GAUSS_WEIGHTS[j],
                    "volume-map quadrature weight ij",
                )?;
                for (k, node_z) in GAUSS_NODES.iter().copied().enumerate() {
                    let offset_z =
                        checked_scalar(SUPPORT_RADIUS * node_z, "volume-map quadrature z")?;
                    let radius_xy = checked_scalar(
                        (offset_x * offset_x) + (offset_y * offset_y),
                        "volume-map quadrature radius xy",
                    )?;
                    let radius_squared = checked_scalar(
                        radius_xy + (offset_z * offset_z),
                        "volume-map quadrature radius squared",
                    )?;
                    if radius_squared > H2 {
                        continue;
                    }
                    let sample_z = checked_scalar(position.z + offset_z, "volume-map sample z")?;
                    let distance =
                        self.signed_distance(Vec3f::new(sample_x, sample_y, sample_z))?;
                    let extension = cubic_extension(distance)?;
                    let weight_ijk = checked_scalar(
                        weight_ij * GAUSS_WEIGHTS[k],
                        "volume-map quadrature weight ijk",
                    )?;
                    let term =
                        checked_scalar(weight_ijk * extension, "volume-map quadrature term")?;
                    result = checked_scalar(result + term, "volume-map quadrature reduction")?;
                }
            }
        }
        let integrated = checked_scalar(result * H3, "volume-map quadrature domain scaling")?;
        checked_scalar(VOLUME_SCALE * integrated, "volume-map reference scaling")
    }

    fn signed_distance(self, position: Vec3f) -> Result<f64, WaterError> {
        let lower = [
            checked_scalar(position.x - self.minimum.x, "volume-map lower x")?,
            checked_scalar(position.y - self.minimum.y, "volume-map lower y")?,
            checked_scalar(position.z - self.minimum.z, "volume-map lower z")?,
        ];
        let upper = [
            checked_scalar(self.maximum.x - position.x, "volume-map upper x")?,
            checked_scalar(self.maximum.y - position.y, "volume-map upper y")?,
            checked_scalar(self.maximum.z - position.z, "volume-map upper z")?,
        ];
        if lower
            .into_iter()
            .chain(upper)
            .all(|distance| distance >= 0.0)
        {
            let clearances = [lower[0], upper[0], lower[1], upper[1], lower[2], upper[2]];
            let mut distance = clearances[0];
            for candidate in clearances.into_iter().skip(1) {
                if candidate < distance {
                    distance = candidate;
                }
            }
            return Ok(distance);
        }
        let outside = [
            if lower[0] < 0.0 {
                lower[0]
            } else if upper[0] < 0.0 {
                -upper[0]
            } else {
                0.0
            },
            if lower[1] < 0.0 {
                lower[1]
            } else if upper[1] < 0.0 {
                -upper[1]
            } else {
                0.0
            },
            if lower[2] < 0.0 {
                lower[2]
            } else if upper[2] < 0.0 {
                -upper[2]
            } else {
                0.0
            },
        ];
        let xy = checked_scalar(
            (outside[0] * outside[0]) + (outside[1] * outside[1]),
            "volume-map outside distance xy",
        )?;
        let squared = checked_scalar(
            xy + (outside[2] * outside[2]),
            "volume-map outside distance squared",
        )?;
        checked_scalar(-squared.sqrt(), "volume-map outside signed distance")
    }

    fn distance_normal(self, position: Vec3f) -> Result<(f64, Vec3f, usize), WaterError> {
        let clearances = [
            checked_scalar(position.x - self.minimum.x, "volume-map normal lower x")?,
            checked_scalar(self.maximum.x - position.x, "volume-map normal upper x")?,
            checked_scalar(position.y - self.minimum.y, "volume-map normal lower y")?,
            checked_scalar(self.maximum.y - position.y, "volume-map normal upper y")?,
            checked_scalar(position.z - self.minimum.z, "volume-map normal lower z")?,
            checked_scalar(self.maximum.z - position.z, "volume-map normal upper z")?,
        ];
        if clearances.iter().any(|distance| *distance < 0.0) {
            return Ok((self.signed_distance(position)?, Vec3f::ZERO, 0));
        }
        let mut distance = clearances[0];
        for candidate in clearances.into_iter().skip(1) {
            if candidate < distance {
                distance = candidate;
            }
        }
        let directions = [
            Vec3f::new(1.0, 0.0, 0.0),
            Vec3f::new(-1.0, 0.0, 0.0),
            Vec3f::new(0.0, 1.0, 0.0),
            Vec3f::new(0.0, -1.0, 0.0),
            Vec3f::new(0.0, 0.0, 1.0),
            Vec3f::new(0.0, 0.0, -1.0),
        ];
        let mut normal = Vec3f::ZERO;
        let mut feature_rank = 0_usize;
        for (index, clearance) in clearances.into_iter().enumerate() {
            if clearance == distance {
                normal = normal
                    .add(directions[index])
                    .checked("volume-map tied normal reduction")?;
                feature_rank += 1;
            }
        }
        let divisor = checked_scalar(
            (feature_rank as f64).sqrt(),
            "volume-map tied normal divisor",
        )?;
        let normal = normal
            .scale(checked_scalar(
                1.0 / divisor,
                "volume-map normal reciprocal",
            )?)
            .checked("volume-map normalized normal")?;
        Ok((distance, normal, feature_rank))
    }
}

fn cubic_extension(signed_distance: f64) -> Result<f64, WaterError> {
    if signed_distance <= 0.0 {
        return Ok(1.0);
    }
    if signed_distance >= SUPPORT_RADIUS {
        return Ok(0.0);
    }
    let q = checked_scalar(signed_distance / SUPPORT_RADIUS, "volume-map extension q")?;
    let value = if q <= 0.5 {
        let q2 = checked_scalar(q * q, "volume-map extension q2")?;
        let q3 = checked_scalar(q2 * q, "volume-map extension q3")?;
        let six_q3 = checked_scalar(6.0 * q3, "volume-map extension six q3")?;
        let six_q2 = checked_scalar(6.0 * q2, "volume-map extension six q2")?;
        let polynomial =
            checked_scalar((six_q3 - six_q2) + 1.0, "volume-map extension polynomial")?;
        checked_scalar(KERNEL_K * polynomial, "volume-map extension kernel inner")?
    } else {
        let t = checked_scalar(1.0 - q, "volume-map extension t")?;
        let t2 = checked_scalar(t * t, "volume-map extension t2")?;
        let t3 = checked_scalar(t2 * t, "volume-map extension t3")?;
        checked_scalar(KERNEL_K * (2.0 * t3), "volume-map extension kernel outer")?
    };
    checked_scalar(value / KERNEL_K, "volume-map normalized extension")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Box3i, Vec3i};

    fn unit_box() -> Geometry {
        Geometry {
            bounds: Box3i {
                min: Vec3i::new(0, 0, 0),
                max: Vec3i::new(1_000_000, 1_000_000, 1_000_000),
            },
            aperture: None,
        }
    }

    #[test]
    fn analytical_map_is_local_and_feature_symmetric() {
        let map = VolumeMapBoundary::new(unit_box()).unwrap();
        assert!(map.sample(Vec3f::new(0.5, 0.5, 0.5)).unwrap().is_none());

        let face = map
            .sample(Vec3f::new(0.025, 0.375, 0.525))
            .unwrap()
            .unwrap();
        let edge = map
            .sample(Vec3f::new(0.025, 0.025, 0.525))
            .unwrap()
            .unwrap();
        let corner = map
            .sample(Vec3f::new(0.025, 0.025, 0.025))
            .unwrap()
            .unwrap();
        assert_eq!(face.feature_rank, 1);
        assert_eq!(edge.feature_rank, 2);
        assert_eq!(corner.feature_rank, 3);
        assert!(face.volume < edge.volume && edge.volume < corner.volume);
        assert_eq!(face.virtual_distance.to_bits(), (0.05_f64).to_bits());
    }
}
