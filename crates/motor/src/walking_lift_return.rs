//! V7 geometric swing/return objective; never changes contact or physics facts.

use crate::{MotorObservationError, rotate_world_to_root_local_q1_30};

pub const WALKING_LIFT_RETURN_REWARD_IDS: [&str; 2] = [
    "reward.periodic-left-sole-height-error.v1",
    "reward.periodic-right-sole-height-error.v1",
];
pub const WALKING_SWING_HEIGHT_MICROMETRES: i64 = 60_000;

#[must_use]
pub fn walking_sole_height_targets_um(motor_tick: u64, command: [i64; 3]) -> [i64; 2] {
    if command == [0; 3] {
        return [0; 2];
    }
    let phase =
        motor_tick.saturating_sub(crate::WALKING_PHASE_START_TICK) % crate::WALKING_CYCLE_TICKS;
    let height = |phase: u64| {
        if !(7..30).contains(&phase) {
            return 0;
        }
        let n = (phase - 6).min(30 - phase) as i64;
        let d = 12_i64;
        // n <= d; the complete numerator is below 15e9. Integer floor is
        // part of the profile, matching the independent rational oracle.
        WALKING_SWING_HEIGHT_MICROMETRES
            * (10 * n.pow(3) * d.pow(2) - 15 * n.pow(4) * d + 6 * n.pow(5))
            / d.pow(5)
    };
    [
        height(phase),
        height((phase + 36) % crate::WALKING_CYCLE_TICKS),
    ]
}

#[must_use]
pub fn walking_sole_height_cost_q16(
    motor_tick: u64,
    command: [i64; 3],
    actual_height_um: [i64; 2],
) -> i64 {
    walking_sole_height_costs_q16(motor_tick, command, actual_height_um)
        .into_iter()
        .sum()
}

#[must_use]
pub fn walking_sole_height_costs_q16(
    motor_tick: u64,
    command: [i64; 3],
    actual_height_um: [i64; 2],
) -> [i64; 2] {
    if command == [0; 3] {
        return [0; 2];
    }
    let targets = walking_sole_height_targets_um(motor_tick, command);
    std::array::from_fn(|side| {
        let (actual, target) = (actual_height_um[side], targets[side]);
        let error = (i128::from(actual) - i128::from(target))
            .abs()
            .min(i128::from(WALKING_SWING_HEIGHT_MICROMETRES));
        (error * 65_536 / i128::from(WALKING_SWING_HEIGHT_MICROMETRES)) as i64
    })
}

/// Minimum world Y of an identity-local-rotation box, floor-rounded to um.
/// R's world-Y row projects the offset and the box support radius. Collider
/// shape/local rotation validation belongs to the schema-consuming caller.
pub fn walking_box_minimum_y_um(
    position_um: [i64; 3],
    rotation_q1_30: [i64; 4],
    local_offset_um: [i64; 3],
    half_extents_um: [i64; 3],
) -> Result<i64, MotorObservationError> {
    if half_extents_um.iter().any(|value| *value <= 0) {
        return Err(MotorObservationError::ArithmeticOverflow);
    }
    let row = rotate_world_to_root_local_q1_30(rotation_q1_30, [0, 1 << 30, 0])?;
    let mut offset = 0_i128;
    for ((coefficient, translation), half) in
        row.into_iter().zip(local_offset_um).zip(half_extents_um)
    {
        // Q30 coefficients and i64 geometry fit in i128, including all three
        // accumulated terms. The final world coordinate still fails closed.
        offset += i128::from(coefficient) * i128::from(translation)
            - i128::from(coefficient).abs() * i128::from(half);
    }
    i64::try_from(i128::from(position_um[1]) + offset.div_euclid(1 << 30))
        .map_err(|_| MotorObservationError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    const MOVE: [i64; 3] = [0, 500_000, 0];

    #[test]
    fn phase_targets_and_cost_reject_ground_wrong_side_and_held_lift() {
        let mut ground = 0;
        let mut wrong = 0;
        let mut held = 0;
        for tick in 120..192 {
            let target = walking_sole_height_targets_um(tick, MOVE);
            assert_eq!(target, walking_sole_height_targets_um(tick + 72, MOVE));
            assert_eq!(
                [target[1], target[0]],
                walking_sole_height_targets_um(tick + 36, MOVE)
            );
            assert_eq!(walking_sole_height_cost_q16(tick, MOVE, target), 0);
            ground += walking_sole_height_cost_q16(tick, MOVE, [0; 2]);
            wrong += walking_sole_height_cost_q16(tick, MOVE, [target[1], target[0]]);
            held += walking_sole_height_cost_q16(tick, MOVE, [60_000, 0]);
        }
        assert_eq!((ground, wrong, held), (1_572_820, 3_145_640, 4_718_572));
        assert_eq!(walking_sole_height_targets_um(138, MOVE), [60_000, 0]);
        assert_eq!(walking_sole_height_targets_um(150, MOVE), [0; 2]);
        assert_eq!(
            walking_sole_height_cost_q16(138, MOVE, [i64::MIN, i64::MAX]),
            131_072
        );
        assert_eq!(
            walking_sole_height_cost_q16(138, [0; 3], [i64::MIN, i64::MAX]),
            0
        );
    }

    #[test]
    fn actual_box_geometry_includes_offset_orientation_and_translation() {
        let half = [55_000, 30_000, 130_000];
        let offset = [0, 11_365, 80_000];
        let identity = [0, 0, 0, 1 << 30];
        assert_eq!(
            walking_box_minimum_y_um([0, 18_635, 0], identity, offset, half).unwrap(),
            0
        );
        assert_eq!(
            walking_box_minimum_y_um([900_000, 78_635, -200_000], identity, offset, half).unwrap(),
            60_000
        );
        // Exact 180-degree x rotation puts the former box top at its minimum.
        assert_eq!(
            walking_box_minimum_y_um([0, 41_365, 0], [1 << 30, 0, 0, 0], offset, half).unwrap(),
            0
        );
        assert!(walking_box_minimum_y_um([0; 3], [i64::MAX; 4], offset, half).is_err());
        assert!(walking_box_minimum_y_um([0; 3], identity, offset, [0; 3]).is_err());
    }
}
