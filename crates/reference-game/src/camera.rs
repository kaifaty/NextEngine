//! Deterministic integer third-person camera math for the live driver.
//!
//! The pinned CORDIC profile keeps orbit offsets exact and reproducible
//! across platforms without float tolerances; the live driver binds these
//! offsets into the presentation camera record every publication.

use next_contracts::input::{
    CORE_CAMERA_ORBIT_ACTION_ID, PlayerActionFrameV1, PlayerActionValueV1,
};

use crate::ReferenceGameError;

pub(crate) const CAMERA_DISTANCE_MICROMETRES: u64 = 4_000_000;
pub(crate) const CAMERA_SHOULDER_MICROMETRES: i64 = 350_000;
const CAMERA_MOUSE_MILLIDEGREES_PER_UNIT: i32 = 120;
const CAMERA_TRIG_FRACTIONAL_BITS: u32 = 52;
const CAMERA_TRIG_ONE_Q52: i64 = 1_i64 << CAMERA_TRIG_FRACTIONAL_BITS;
const CAMERA_CORDIC_GAIN_INVERSE_Q52: i64 = 2_734_824_091_825_638;
// Pinned deterministic CORDIC profile: atan(2^-i) uses Q62 turns while the
// rotated sine/cosine vector uses Q52. These integers are canonical inputs.
const CAMERA_CORDIC_ATAN_TURN_Q62: [i64; 53] = [
    576_460_752_303_423_488,
    340_304_653_033_718_272,
    179_807_632_645_220_256,
    91_273_161_881_380_496,
    45_813_697_873_323_712,
    22_929_182_573_009_056,
    11_467_389_120_678_284,
    5_734_044_481_687_724,
    2_867_065_987_018_958,
    1_433_538_461_969_102,
    716_769_914_547_871,
    358_385_042_719_534,
    179_192_532_040_472,
    89_596_267_355_325,
    44_798_133_844_548,
    22_399_066_943_135,
    11_199_533_474_175,
    5_599_766_737_413,
    2_799_883_368_747,
    1_399_941_684_379,
    699_970_842_190,
    349_985_421_095,
    174_992_710_548,
    87_496_355_274,
    43_748_177_637,
    21_874_088_818,
    10_937_044_409,
    5_468_522_205,
    2_734_261_102,
    1_367_130_551,
    683_565_276,
    341_782_638,
    170_891_319,
    85_445_659,
    42_722_830,
    21_361_415,
    10_680_707,
    5_340_354,
    2_670_177,
    1_335_088,
    667_544,
    333_772,
    166_886,
    83_443,
    41_722,
    20_861,
    10_430,
    5_215,
    2_608,
    1_304,
    652,
    326,
    163,
];

pub(crate) fn update_camera_state(
    frame: &PlayerActionFrameV1,
    camera_yaw_millidegrees: &mut i32,
    camera_pitch_millidegrees: &mut i32,
) {
    let Some(PlayerActionValueV1::Vector2Q15([yaw, pitch])) = frame
        .actions
        .iter()
        .find(|action| action.action_id.as_str() == CORE_CAMERA_ORBIT_ACTION_ID)
        .map(|action| action.value)
    else {
        return;
    };
    let yaw_delta = i32::from(yaw).saturating_mul(CAMERA_MOUSE_MILLIDEGREES_PER_UNIT);
    let unwrapped = camera_yaw_millidegrees.saturating_add(yaw_delta);
    *camera_yaw_millidegrees = (unwrapped.saturating_add(180_000)).rem_euclid(360_000) - 180_000;
    let pitch_delta = i32::from(pitch).saturating_mul(CAMERA_MOUSE_MILLIDEGREES_PER_UNIT);
    *camera_pitch_millidegrees = camera_pitch_millidegrees
        .saturating_add(pitch_delta)
        .clamp(-75_000, 75_000);
}

pub(crate) fn camera_orbit_offset_micrometres(
    yaw_millidegrees: i32,
    pitch_millidegrees: i32,
) -> Result<[i64; 3], ReferenceGameError> {
    let (yaw_sine, yaw_cosine) = deterministic_sin_cos_q52(yaw_millidegrees)?;
    let (pitch_sine, pitch_cosine) = deterministic_sin_cos_q52(pitch_millidegrees)?;
    let scale = i128::from(CAMERA_TRIG_ONE_Q52);
    let scale_squared = scale * scale;
    let distance = i128::from(CAMERA_DISTANCE_MICROMETRES);
    let shoulder = i128::from(CAMERA_SHOULDER_MICROMETRES);
    let yaw_sine = i128::from(yaw_sine);
    let yaw_cosine = i128::from(yaw_cosine);
    let pitch_sine = i128::from(pitch_sine);
    let pitch_cosine = i128::from(pitch_cosine);

    Ok([
        round_fixed_ratio_to_i64(
            yaw_sine * pitch_cosine * distance + yaw_cosine * shoulder * scale,
            scale_squared,
        )?,
        round_fixed_ratio_to_i64(-pitch_sine * distance, scale)?,
        round_fixed_ratio_to_i64(
            -yaw_cosine * pitch_cosine * distance + yaw_sine * shoulder * scale,
            scale_squared,
        )?,
    ])
}

fn deterministic_sin_cos_q52(angle_millidegrees: i32) -> Result<(i64, i64), ReferenceGameError> {
    if !(-180_000..=180_000).contains(&angle_millidegrees) {
        return Err(ReferenceGameError::CountOverflow);
    }
    let (reduced_angle, quadrant_sign) = if angle_millidegrees > 90_000 {
        (angle_millidegrees - 180_000, -1_i64)
    } else if angle_millidegrees < -90_000 {
        (angle_millidegrees + 180_000, -1_i64)
    } else {
        (angle_millidegrees, 1_i64)
    };
    match reduced_angle {
        0 => return Ok((0, quadrant_sign * CAMERA_TRIG_ONE_Q52)),
        90_000 => return Ok((quadrant_sign * CAMERA_TRIG_ONE_Q52, 0)),
        -90_000 => return Ok((-quadrant_sign * CAMERA_TRIG_ONE_Q52, 0)),
        _ => {}
    }

    let mut residual_turn_q62 =
        round_fixed_ratio_to_i64(i128::from(reduced_angle) * (1_i128 << 62), 360_000)?;
    let mut cosine = CAMERA_CORDIC_GAIN_INVERSE_Q52;
    let mut sine = 0_i64;
    for (shift, angle) in CAMERA_CORDIC_ATAN_TURN_Q62.iter().copied().enumerate() {
        let prior_cosine = cosine;
        let prior_sine = sine;
        if residual_turn_q62 >= 0 {
            cosine = prior_cosine - (prior_sine >> shift);
            sine = prior_sine + (prior_cosine >> shift);
            residual_turn_q62 -= angle;
        } else {
            cosine = prior_cosine + (prior_sine >> shift);
            sine = prior_sine - (prior_cosine >> shift);
            residual_turn_q62 += angle;
        }
    }
    Ok((
        (sine * quadrant_sign).clamp(-CAMERA_TRIG_ONE_Q52, CAMERA_TRIG_ONE_Q52),
        (cosine * quadrant_sign).clamp(-CAMERA_TRIG_ONE_Q52, CAMERA_TRIG_ONE_Q52),
    ))
}

fn round_fixed_ratio_to_i64(numerator: i128, denominator: i128) -> Result<i64, ReferenceGameError> {
    debug_assert!(denominator > 0);
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let rounded = if remainder.abs() * 2 >= denominator {
        quotient + numerator.signum()
    } else {
        quotient
    };
    i64::try_from(rounded).map_err(|_| ReferenceGameError::CountOverflow)
}

#[cfg(test)]
mod tests {
    use super::camera_orbit_offset_micrometres;

    #[test]
    fn integer_camera_orbit_matches_the_pinned_micrometre_golden_vectors() {
        let cases = [
            ((0, -15_000), [350_000, 1_035_276, -3_863_703]),
            ((90_000, 0), [4_000_000, 0, 350_000]),
            ((-90_000, 0), [-4_000_000, 0, -350_000]),
            ((-180_000, 0), [-350_000, 0, 4_000_000]),
            ((42_480, 12_360), [2_896_849, -856_214, -2_645_310]),
            ((-42_480, -12_360), [-2_380_590, 856_214, -3_118_042]),
            ((179_880, 75_000), [-347_831, -3_863_703, 1_036_007]),
            ((-180_000, -75_000), [-350_000, 3_863_703, 1_035_276]),
        ];
        for ((yaw, pitch), expected) in cases {
            assert_eq!(
                camera_orbit_offset_micrometres(yaw, pitch).expect("bounded camera orbit"),
                expected,
                "yaw={yaw}, pitch={pitch}"
            );
        }
    }

    #[test]
    fn integer_camera_orbit_preserves_opposite_yaw_symmetry_without_float_tolerance() {
        for pitch in [-75_000, -15_000, 0, 12_360, 75_000] {
            let forward = camera_orbit_offset_micrometres(0, pitch).expect("forward camera orbit");
            let backward =
                camera_orbit_offset_micrometres(-180_000, pitch).expect("backward camera orbit");
            assert_eq!(forward[0], -backward[0]);
            assert_eq!(forward[1], backward[1]);
            assert_eq!(forward[2], -backward[2]);
        }
    }
}
