//! Canonical, reference-free load-transfer credit. No pose or physics overrides.

pub const WALKING_CYCLE_TICKS: u64 = 72;
pub const WALKING_CLOCK_WIDTH: usize = 2;
pub const WALKING_PHASE_START_TICK: u64 = 120;
pub const WALKING_LOAD_REWARD_ID: &str = "reward.periodic-load-transfer.v1";

fn phase(motor_tick: u64) -> u64 {
    motor_tick.saturating_sub(WALKING_PHASE_START_TICK) % WALKING_CYCLE_TICKS
}

/// Quadrature triangle waves encode direction as well as position in the cycle.
/// The pair is injective over all 72 ticks and requires no floating-point trig.
#[must_use]
pub fn walking_clock_q1_30(motor_tick: u64, command: [i64; 3]) -> [i64; 2] {
    if command == [0; 3] {
        return [0; 2];
    }
    let triangle = |tick: u64| {
        let tick = (tick % WALKING_CYCLE_TICKS) as i64;
        let distance = (tick - 36).abs();
        (18 - distance) * (1_i64 << 30) / 18
    };
    [
        triangle(phase(motor_tick)),
        triangle(phase(motor_tick) + 18),
    ]
}

/// 24-tick single-support plateaux, separated by 12-tick load transfers.
/// Starts at equal loading, transfers to right support, then to left support.
#[must_use]
pub fn walking_left_load_target_q16(motor_tick: u64) -> i64 {
    let tick = phase(motor_tick) as i64;
    match tick {
        0..=5 => (6 - tick) * 65_536 / 12,
        6..=29 => 0,
        30..=41 => (tick - 30) * 65_536 / 12,
        42..=65 => 65_536,
        _ => (78 - tick) * 65_536 / 12,
    }
}

/// Inputs are canonical ground-contact vertical impulses and foot planar speeds.
/// Both flight and wrong-side unloading earn zero. Balanced loading receives no
/// swing-plateau credit, but unloading improves credit before contact disappears.
#[must_use]
pub fn periodic_load_credit_q16(
    motor_tick: u64,
    command: [i64; 3],
    vertical_impulses: [u128; 2],
    planar_speeds_micrometres_per_second: [u128; 2],
    contact_flags: [i64; 2],
) -> i64 {
    if command == [0; 3] {
        return i64::from(contact_flags == [1, 1]) * 65_536;
    }
    let total = vertical_impulses[0].saturating_add(vertical_impulses[1]);
    if total == 0 {
        return 0;
    }
    // Inputs come from at most four bounded native contact snapshots.
    // Saturating arithmetic keeps diagnostic/untrusted extreme inputs bounded.
    let actual_left = (vertical_impulses[0].saturating_mul(65_536) / total).min(65_536) as i64;
    let target_left = walking_left_load_target_q16(motor_tick);
    let loading = (65_536 - 2 * (actual_left - target_left).abs()).max(0);
    let weighted_speed = planar_speeds_micrometres_per_second[0]
        .saturating_mul(target_left as u128)
        .saturating_add(
            planar_speeds_micrometres_per_second[1].saturating_mul((65_536 - target_left) as u128),
        )
        / 65_536;
    let stance_stationary =
        65_536 - (weighted_speed.saturating_mul(65_536) / 1_000_000).min(65_536) as i64;
    loading * stance_stationary / 65_536
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_is_bounded_injective_periodic_and_off_at_stop() {
        let clocks = (120..192)
            .map(|tick| walking_clock_q1_30(tick, [0, 500_000, 0]))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(clocks.len(), 72);
        for tick in 120..264 {
            let clock = walking_clock_q1_30(tick, [0, 500_000, 0]);
            assert!(clock.iter().all(|value| value.abs() <= 1_i64 << 30));
            assert_eq!(clock, walking_clock_q1_30(tick + 72, [0, 500_000, 0]));
            assert_eq!(walking_clock_q1_30(tick, [0; 3]), [0; 2]);
        }
    }

    #[test]
    fn load_schedule_is_bilateral_and_continuous_at_wrap() {
        for tick in 120..192 {
            // Integer division may contribute a single Q16 unit of rounding.
            assert!(
                (walking_left_load_target_q16(tick) + walking_left_load_target_q16(tick + 36)
                    - 65_536)
                    .abs()
                    <= 1
            );
            assert!(
                (walking_left_load_target_q16(tick + 1) - walking_left_load_target_q16(tick)).abs()
                    <= 5462
            );
        }
    }

    #[test]
    fn unloading_has_credit_before_release_but_flight_and_wrong_side_do_not() {
        let reward = |loads| periodic_load_credit_q16(132, [0, 500_000, 0], loads, [0; 2], [1, 1]);
        assert_eq!(reward([50, 50]), 0);
        assert!(reward([25, 75]) > reward([50, 50]));
        assert!(reward([0, 100]) > reward([25, 75]));
        assert_eq!(reward([0, 100]), 65_536);
        assert_eq!(reward([100, 0]), 0);
        assert_eq!(reward([0, 0]), 0);
        assert_eq!(
            periodic_load_credit_q16(168, [0, 500_000, 0], [100, 0], [0; 2], [1, 0]),
            65_536
        );
        assert_eq!(
            periodic_load_credit_q16(132, [0, 500_000, 0], [0, 100], [0, 1_000_000], [0, 1]),
            0
        );
        assert_eq!(
            periodic_load_credit_q16(132, [0; 3], [0; 2], [0; 2], [1, 1]),
            65_536
        );
        assert_eq!(
            periodic_load_credit_q16(132, [0; 3], [0, 100], [0; 2], [0, 1]),
            0
        );
    }
}
