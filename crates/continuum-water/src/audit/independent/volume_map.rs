#![forbid(unsafe_code)]

use super::*;

pub(super) struct VolumeSample {
    pub(super) signed_distance: f64,
    pub(super) volume: f64,
    pub(super) virtual_distance: f64,
    pub(super) displacement: F3,
    pub(super) value: f64,
    pub(super) gradient: F3,
    pub(super) feature_rank: usize,
}

const PARTICLE_RADIUS: f64 = f64::from_bits(0x3f99_9999_9999_999a);
const SUPPORT_RADIUS_SQUARED_F64: f64 = f64::from_bits(0x3f84_7ae1_47ae_147c);
const SUPPORT_RADIUS_CUBED: f64 = f64::from_bits(0x3f50_624d_d2f1_a9fd);
const VOLUME_SCALE: f64 = f64::from_bits(0x3fe9_9999_9999_999a);

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

pub(super) fn sample(position: F3) -> Result<Option<VolumeSample>, WaterError> {
    let (signed_distance, normal, feature_rank) = distance_normal(position)?;
    if signed_distance <= 0.0 || signed_distance >= SUPPORT_RADIUS {
        return Ok(None);
    }
    let volume = integrated_volume(position)?;
    if volume <= 0.0 {
        return Ok(None);
    }
    let offset_distance = finite(
        signed_distance + (0.5 * PARTICLE_RADIUS),
        "independent volume-map virtual offset distance",
    )?;
    let minimum_distance = finite(
        2.0 * PARTICLE_RADIUS,
        "independent volume-map minimum virtual distance",
    )?;
    let virtual_distance = offset_distance.max(minimum_distance);
    let displacement = finite_vec(
        normal.scale(virtual_distance),
        "independent volume-map virtual displacement",
    )?;
    let sampled = kernel_f3(displacement)?;
    Ok(Some(VolumeSample {
        signed_distance,
        volume,
        virtual_distance,
        displacement,
        value: sampled.value,
        gradient: sampled.gradient,
        feature_rank,
    }))
}

fn integrated_volume(position: F3) -> Result<f64, WaterError> {
    let mut result = 0.0;
    for (i, node_x) in GAUSS_NODES.iter().copied().enumerate() {
        let offset_x = finite(
            SUPPORT_RADIUS * node_x,
            "independent volume-map quadrature x",
        )?;
        let sample_x = finite(position.x + offset_x, "independent volume-map sample x")?;
        let weight_i = GAUSS_WEIGHTS[i];
        for (j, node_y) in GAUSS_NODES.iter().copied().enumerate() {
            let offset_y = finite(
                SUPPORT_RADIUS * node_y,
                "independent volume-map quadrature y",
            )?;
            let sample_y = finite(position.y + offset_y, "independent volume-map sample y")?;
            let weight_ij = finite(
                weight_i * GAUSS_WEIGHTS[j],
                "independent volume-map quadrature weight ij",
            )?;
            for (k, node_z) in GAUSS_NODES.iter().copied().enumerate() {
                let offset_z = finite(
                    SUPPORT_RADIUS * node_z,
                    "independent volume-map quadrature z",
                )?;
                let radius_xy = finite(
                    (offset_x * offset_x) + (offset_y * offset_y),
                    "independent volume-map quadrature radius xy",
                )?;
                let radius_squared = finite(
                    radius_xy + (offset_z * offset_z),
                    "independent volume-map quadrature radius squared",
                )?;
                if radius_squared > SUPPORT_RADIUS_SQUARED_F64 {
                    continue;
                }
                let sample_z = finite(position.z + offset_z, "independent volume-map sample z")?;
                let distance = signed_distance(F3::new(sample_x, sample_y, sample_z))?;
                let extension = cubic_extension(distance)?;
                let weight_ijk = finite(
                    weight_ij * GAUSS_WEIGHTS[k],
                    "independent volume-map quadrature weight ijk",
                )?;
                let term = finite(
                    weight_ijk * extension,
                    "independent volume-map quadrature term",
                )?;
                result = finite(result + term, "independent volume-map quadrature reduction")?;
            }
        }
    }
    let integrated = finite(
        result * SUPPORT_RADIUS_CUBED,
        "independent volume-map quadrature domain scaling",
    )?;
    finite(
        VOLUME_SCALE * integrated,
        "independent volume-map reference scaling",
    )
}

fn signed_distance(position: F3) -> Result<f64, WaterError> {
    let lower = [position.x, position.y, position.z];
    let upper = [1.0 - position.x, 1.0 - position.y, 1.0 - position.z];
    for value in lower.into_iter().chain(upper) {
        finite(value, "independent volume-map box clearance")?;
    }
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
    let xy = finite(
        (outside[0] * outside[0]) + (outside[1] * outside[1]),
        "independent volume-map outside distance xy",
    )?;
    let squared = finite(
        xy + (outside[2] * outside[2]),
        "independent volume-map outside distance squared",
    )?;
    finite(
        -squared.sqrt(),
        "independent volume-map outside signed distance",
    )
}

fn distance_normal(position: F3) -> Result<(f64, F3, usize), WaterError> {
    let clearances = [
        finite(position.x, "independent volume-map normal lower x")?,
        finite(1.0 - position.x, "independent volume-map normal upper x")?,
        finite(position.y, "independent volume-map normal lower y")?,
        finite(1.0 - position.y, "independent volume-map normal upper y")?,
        finite(position.z, "independent volume-map normal lower z")?,
        finite(1.0 - position.z, "independent volume-map normal upper z")?,
    ];
    if clearances.iter().any(|distance| *distance < 0.0) {
        return Ok((signed_distance(position)?, F3::ZERO, 0));
    }
    let mut distance = clearances[0];
    for candidate in clearances.into_iter().skip(1) {
        if candidate < distance {
            distance = candidate;
        }
    }
    let directions = [
        F3::new(1.0, 0.0, 0.0),
        F3::new(-1.0, 0.0, 0.0),
        F3::new(0.0, 1.0, 0.0),
        F3::new(0.0, -1.0, 0.0),
        F3::new(0.0, 0.0, 1.0),
        F3::new(0.0, 0.0, -1.0),
    ];
    let mut normal = F3::ZERO;
    let mut feature_rank = 0_usize;
    for (index, clearance) in clearances.into_iter().enumerate() {
        if clearance == distance {
            normal = finite_vec(
                normal.add(directions[index]),
                "independent volume-map tied normal reduction",
            )?;
            feature_rank += 1;
        }
    }
    let divisor = finite(
        (feature_rank as f64).sqrt(),
        "independent volume-map tied normal divisor",
    )?;
    let reciprocal = finite(1.0 / divisor, "independent volume-map normal reciprocal")?;
    let normal = finite_vec(
        normal.scale(reciprocal),
        "independent volume-map normalized normal",
    )?;
    Ok((distance, normal, feature_rank))
}

fn cubic_extension(signed_distance: f64) -> Result<f64, WaterError> {
    if signed_distance <= 0.0 {
        return Ok(1.0);
    }
    if signed_distance >= SUPPORT_RADIUS {
        return Ok(0.0);
    }
    let q = finite(
        signed_distance / SUPPORT_RADIUS,
        "independent volume-map extension q",
    )?;
    let value = if q <= 0.5 {
        let q2 = finite(q * q, "independent volume-map extension q2")?;
        let q3 = finite(q2 * q, "independent volume-map extension q3")?;
        let six_q3 = finite(6.0 * q3, "independent volume-map extension six q3")?;
        let six_q2 = finite(6.0 * q2, "independent volume-map extension six q2")?;
        let polynomial = finite(
            (six_q3 - six_q2) + 1.0,
            "independent volume-map extension polynomial",
        )?;
        finite(
            KERNEL_K * polynomial,
            "independent volume-map extension kernel inner",
        )?
    } else {
        let t = finite(1.0 - q, "independent volume-map extension t")?;
        let t2 = finite(t * t, "independent volume-map extension t2")?;
        let t3 = finite(t2 * t, "independent volume-map extension t3")?;
        finite(
            KERNEL_K * (2.0 * t3),
            "independent volume-map extension kernel outer",
        )?
    };
    finite(
        value / KERNEL_K,
        "independent volume-map normalized extension",
    )
}
