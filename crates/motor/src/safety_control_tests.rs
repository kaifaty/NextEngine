use next_contracts::ids::PersistentId;

use crate::safety_control::*;
use crate::{
    ACTUATOR_TARGET_CLAMPED, CompiledBodySchemaV2, JointControlStateV1,
    biomechanics_humanoid_body_schema_v2,
};

fn controller() -> BiomechanicsSafetyController {
    let compiled = CompiledBodySchemaV2::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([31; 16]),
    )
    .expect("compile biomechanics schema");
    BiomechanicsSafetyController::new(&compiled).expect("construct safety controller")
}

fn neutral_tick(
    controller: &mut BiomechanicsSafetyController,
) -> (Vec<i64>, Vec<JointTargetEnvelopeV1>) {
    let neutral = controller
        .channels
        .iter()
        .map(|channel| channel.joint.neutral_position_microradians)
        .collect::<Vec<_>>();
    let envelopes = controller.default_skill_envelopes();
    (neutral, envelopes)
}

fn neutral_states(controller: &BiomechanicsSafetyController) -> Vec<JointControlStateV1> {
    controller
        .channels
        .iter()
        .map(|channel| JointControlStateV1 {
            position_microradians: channel.joint.neutral_position_microradians,
            velocity_microradians_per_second: 0,
        })
        .collect()
}

#[test]
fn reference_reset_sets_first_tick_slew_origin_atomically() {
    let mut controller = controller();
    let mut reference = controller
        .channels
        .iter()
        .map(|channel| channel.joint.neutral_position_microradians)
        .collect::<Vec<_>>();
    reference[0] += 50_000;
    controller
        .reset_to_reference(&reference)
        .expect("valid reference reset");
    assert_eq!(
        controller.checkpoint().applied_targets_microradians,
        reference
    );

    let before = controller.checkpoint();
    let mut invalid = reference;
    invalid[0] = i64::MAX;
    assert_eq!(
        controller.reset_to_reference(&invalid),
        Err(MotorSafetyError::InvalidSkillEnvelope)
    );
    assert_eq!(controller.checkpoint(), before);
}

#[test]
fn residual_target_intersects_soft_skill_and_slew_envelopes() {
    let mut controller = controller();
    let (mut reference, mut envelopes) = neutral_tick(&mut controller);
    let mut residuals = vec![0; controller.channel_count()];
    reference[0] = i64::MAX;
    residuals[1] = NORMALIZED_RESIDUAL_ONE_Q1_30;
    envelopes[2].minimum_microradians = 25_000;
    envelopes[2].maximum_microradians = 25_000;

    let applied = controller
        .begin_motor_tick(&reference, &residuals, &envelopes)
        .expect("bounded action");

    assert_ne!(applied[0].clamp_flags & ACTUATOR_TARGET_CLAMPED, 0);
    assert_ne!(applied[0].clamp_flags & ACTUATOR_TARGET_SLEW_CLAMPED, 0);
    assert_eq!(applied[2].target_microradians, 25_000);
    for (target, channel) in applied.iter().zip(&controller.channels) {
        assert!(
            (channel.joint.soft_limit_min_microradians..=channel.joint.soft_limit_max_microradians)
                .contains(&target.target_microradians)
        );
    }
}

#[test]
fn walking_action_multiplier_widens_residual_before_the_same_safety_envelope() {
    let mut baseline = controller();
    let mut widened = controller();
    let (reference, envelopes) = neutral_tick(&mut baseline);
    let mut residuals = vec![0; baseline.channel_count()];
    residuals[0] = NORMALIZED_RESIDUAL_ONE_Q1_30 / 4;

    let baseline_targets = baseline
        .begin_motor_tick(&reference, &residuals, &envelopes)
        .expect("baseline action");
    let widened_targets = widened
        .begin_motor_tick_with_residual_scale_multiplier(
            &reference,
            &residuals,
            &envelopes,
            2 * 65_536,
        )
        .expect("widened walking action");
    assert_eq!(
        baseline_targets[0].target_microradians - reference[0],
        37_500
    );
    assert_eq!(
        widened_targets[0].target_microradians - reference[0],
        75_000
    );
}

#[test]
fn walking_multiplier_rejects_out_of_range_values_without_mutation() {
    for multiplier in [-1, 0, 4 * 65_536 + 1, i64::MAX] {
        let mut controller = controller();
        let (reference, envelopes) = neutral_tick(&mut controller);
        let before = controller.checkpoint();
        let residuals = vec![NORMALIZED_RESIDUAL_ONE_Q1_30; controller.channel_count()];
        assert_eq!(
            controller.begin_motor_tick_with_residual_scale_multiplier(
                &reference, &residuals, &envelopes, multiplier,
            ),
            Err(MotorSafetyError::InvalidNormalizedResidual)
        );
        assert_eq!(controller.checkpoint(), before);
    }
}

#[test]
fn action_rejection_is_atomic_for_late_invalid_channel() {
    let mut controller = controller();
    let (reference, mut envelopes) = neutral_tick(&mut controller);
    let residuals = vec![0; controller.channel_count()];
    let before = controller.checkpoint();
    let last = envelopes.len() - 1;
    envelopes[last].joint_id = envelopes[0].joint_id.clone();

    assert_eq!(
        controller.begin_motor_tick(&reference, &residuals, &envelopes),
        Err(MotorSafetyError::InvalidSkillEnvelope)
    );
    assert_eq!(controller.checkpoint(), before);
}

#[test]
fn disjoint_skill_and_slew_envelopes_reject_atomically() {
    let mut controller = controller();
    let (reference, mut envelopes) = neutral_tick(&mut controller);
    let residuals = vec![0; controller.channel_count()];
    envelopes[0].minimum_microradians = 100_000;
    envelopes[0].maximum_microradians = 200_000;
    let before = controller.checkpoint();

    assert_eq!(
        controller.begin_motor_tick(&reference, &residuals, &envelopes),
        Err(MotorSafetyError::InfeasibleTargetEnvelope)
    );
    assert_eq!(controller.checkpoint(), before);
}

#[test]
fn every_maximum_action_remains_inside_soft_rom_across_ticks() {
    let mut controller = controller();
    let (reference, envelopes) = neutral_tick(&mut controller);
    let residuals = vec![NORMALIZED_RESIDUAL_ONE_Q1_30; controller.channel_count()];
    let states = neutral_states(&controller);
    for _ in 0..32 {
        let targets = controller
            .begin_motor_tick(&reference, &residuals, &envelopes)
            .expect("maximum bounded action");
        for (target, channel) in targets.iter().zip(&controller.channels) {
            assert!(
                (channel.joint.soft_limit_min_microradians
                    ..=channel.joint.soft_limit_max_microradians)
                    .contains(&target.target_microradians)
            );
        }
        for _ in 0..PHYSICS_SUBSTEPS_PER_MOTOR_TICK {
            controller.step_substep(&states).expect("safe substep");
        }
    }
}

#[test]
fn pd_effort_rate_power_and_positive_work_stay_inside_authored_bounds() {
    let mut controller = controller();
    let (reference, envelopes) = neutral_tick(&mut controller);
    let residuals = vec![NORMALIZED_RESIDUAL_ONE_Q1_30; controller.channel_count()];
    let mut states = neutral_states(&controller);
    for (state, channel) in states.iter_mut().zip(&controller.channels) {
        state.position_microradians = channel.joint.soft_limit_min_microradians;
        state.velocity_microradians_per_second =
            i64::try_from(channel.joint.base.maximum_velocity_microradians_per_second)
                .expect("profile velocity fits i64");
    }
    let mut prior = vec![0_i64; controller.channel_count()];
    let mut saw_power_clamp = false;
    let mut saw_work_clamp = false;
    for _ in 0..24 {
        controller
            .begin_motor_tick(&reference, &residuals, &envelopes)
            .expect("prepare tick");
        for _ in 0..PHYSICS_SUBSTEPS_PER_MOTOR_TICK {
            let efforts = controller.step_substep(&states).expect("bounded effort");
            for (index, ((effort, channel), state)) in efforts
                .iter()
                .zip(&controller.channels)
                .zip(&states)
                .enumerate()
            {
                assert!(
                    (channel.actuator.minimum_effort_micronewton_metres
                        ..=channel.actuator.maximum_effort_micronewton_metres)
                        .contains(&effort.effort_micronewton_metres)
                );
                let maximum_delta = round_div_ties_even(
                    i128::from(
                        channel
                            .actuator
                            .base
                            .maximum_effort_rate_micronewton_metres_per_second,
                    ),
                    PHYSICS_SUBSTEPS_PER_SECOND,
                );
                assert!(
                    i128::from(effort.effort_micronewton_metres - prior[index]).abs()
                        <= maximum_delta
                );
                let power = i128::from(effort.effort_micronewton_metres)
                    * i128::from(state.velocity_microradians_per_second);
                assert!(
                    power.unsigned_abs() / MICRO_SCALE as u128
                        <= u128::from(channel.actuator.maximum_power_microwatts)
                );
                saw_power_clamp |= effort.clamp_flags & ACTUATOR_POWER_CLAMPED != 0;
                saw_work_clamp |= effort.clamp_flags & ACTUATOR_WORK_CLAMPED != 0;
                prior[index] = effort.effort_micronewton_metres;
            }
        }
        let checkpoint = controller.checkpoint();
        for (work, channel) in checkpoint
            .positive_work_microjoules
            .iter()
            .zip(&controller.channels)
        {
            assert!(
                *work
                    <= channel
                        .actuator
                        .maximum_positive_work_microjoules_per_motor_tick
            );
        }
    }
    assert!(
        saw_power_clamp,
        "corpus must exercise the power intersection"
    );
    assert!(saw_work_clamp, "corpus must exercise the work intersection");
}

#[test]
fn hard_rom_and_velocity_faults_publish_no_partial_effort() {
    let mut controller = controller();
    let (reference, envelopes) = neutral_tick(&mut controller);
    let residuals = vec![0; controller.channel_count()];
    controller
        .begin_motor_tick(&reference, &residuals, &envelopes)
        .expect("prepare tick");
    let mut states = neutral_states(&controller);
    let last = states.len() - 1;
    states[last].position_microradians =
        controller.channels[last].joint.base.limit_max_microradians + 10;
    assert_eq!(controller.validate_observed_joint_states(&states), Ok(()));
    states[last].position_microradians =
        controller.channels[last].joint.base.limit_max_microradians + 11;
    let before = controller.checkpoint();
    assert_eq!(
        controller.step_substep(&states),
        Err(MotorSafetyError::HardRangeViolation)
    );
    assert_eq!(controller.checkpoint(), before);

    states[last].position_microradians = controller.channels[last]
        .joint
        .neutral_position_microradians;
    states[last].velocity_microradians_per_second = i64::try_from(
        controller.channels[last]
            .joint
            .base
            .maximum_velocity_microradians_per_second,
    )
    .expect("profile velocity fits i64")
        + i64::try_from(OBSERVED_MAXIMUM_VELOCITY_QUANTIZATION_TOLERANCE_MICRORADIANS_PER_SECOND)
            .expect("velocity tolerance fits i64")
        + 1;
    assert_eq!(
        controller.step_substep(&states),
        Err(MotorSafetyError::VelocityViolation)
    );
    assert_eq!(controller.checkpoint(), before);
}

#[test]
fn observed_hard_rom_tolerance_is_exactly_ten_microradians() {
    let controller = controller();
    let mut states = neutral_states(&controller);
    let first = 0;
    let minimum = controller.channels[first].joint.base.limit_min_microradians;
    states[first].position_microradians = minimum - 10;
    assert_eq!(controller.validate_observed_joint_states(&states), Ok(()));
    states[first].position_microradians = minimum - 11;
    assert_eq!(
        controller.validate_observed_joint_states(&states),
        Err(MotorSafetyError::HardRangeViolation)
    );
}

#[test]
fn observed_velocity_tolerance_is_exactly_one_thousand_microradians_per_second() {
    let controller = controller();
    let mut states = neutral_states(&controller);
    let first = 0;
    let maximum = i64::try_from(
        controller.channels[first]
            .joint
            .base
            .maximum_velocity_microradians_per_second,
    )
    .expect("profile velocity fits i64");
    states[first].velocity_microradians_per_second = maximum + 1_000;
    assert_eq!(controller.validate_observed_joint_states(&states), Ok(()));
    states[first].velocity_microradians_per_second = maximum + 1_001;
    assert_eq!(
        controller.validate_observed_joint_states(&states),
        Err(MotorSafetyError::VelocityViolation)
    );
}

#[test]
fn post_step_observation_validation_is_read_only() {
    let controller = controller();
    let mut states = neutral_states(&controller);
    let before = controller.checkpoint_root();
    assert_eq!(controller.validate_observed_joint_states(&states), Ok(()));
    let last = states.len() - 1;
    states[last].velocity_microradians_per_second = i64::try_from(
        controller.channels[last]
            .joint
            .base
            .maximum_velocity_microradians_per_second,
    )
    .expect("profile velocity fits i64")
        + i64::try_from(OBSERVED_MAXIMUM_VELOCITY_QUANTIZATION_TOLERANCE_MICRORADIANS_PER_SECOND)
            .expect("velocity tolerance fits i64")
        + 1;
    assert_eq!(
        controller.validate_observed_joint_states(&states),
        Err(MotorSafetyError::VelocityViolation)
    );
    assert_eq!(controller.checkpoint_root(), before);
}

#[test]
fn reset_clears_targets_efforts_work_and_tick_state_exactly() {
    let mut controller = controller();
    let pristine = controller.checkpoint();
    let pristine_root = controller.checkpoint_root();
    let (reference, envelopes) = neutral_tick(&mut controller);
    let residuals = vec![NORMALIZED_RESIDUAL_ONE_Q1_30; controller.channel_count()];
    controller
        .begin_motor_tick(&reference, &residuals, &envelopes)
        .expect("prepare tick");
    controller
        .step_substep(&neutral_states(&controller))
        .expect("substep");
    assert_ne!(controller.checkpoint(), pristine);
    assert_ne!(controller.checkpoint_root(), pristine_root);
    controller.reset();
    assert_eq!(controller.checkpoint(), pristine);
    assert_eq!(controller.checkpoint_root(), pristine_root);
}

#[test]
fn ties_to_even_residual_scaling_is_sign_symmetric() {
    assert_eq!(round_div_ties_even(5, 2), 2);
    assert_eq!(round_div_ties_even(7, 2), 4);
    assert_eq!(round_div_ties_even(-5, 2), -2);
    assert_eq!(round_div_ties_even(-7, 2), -4);
}
