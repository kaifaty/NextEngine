use super::*;

pub(super) fn locomotion_reward_components(
    profile: MotorEnvironmentProfile,
    frame: &MotorFrameResult,
    command_raw: [i64; 3],
    previous_applied_action: &[i64],
    foot_tokens: &[u64],
    maximum_effort_per_frame: u128,
    fell: bool,
) -> Result<(Vec<(SchemaId, i64)>, i64), TrainingEnvironmentError> {
    let root = frame
        .snapshot
        .links
        .first()
        .ok_or(TrainingEnvironmentError::RewardFacts)?;
    if frame.observation_raw.len() != 84
        || previous_applied_action.len() != frame.applied_action_microradians.len()
    {
        return Err(TrainingEnvironmentError::RewardFacts);
    }
    let local_right_velocity = frame.observation_raw[4];
    let local_forward_velocity = frame.observation_raw[6];
    let local_yaw_rate = frame.observation_raw[8];
    let planar_error = abs_sum([
        local_right_velocity.saturating_sub(command_raw[0]),
        local_forward_velocity.saturating_sub(command_raw[1]),
    ]);
    let curriculum = profile == MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2;
    let planar_tracking =
        one_minus_normalized_q16(planar_error, if curriculum { 2_500_000 } else { 6_500_000 });
    let yaw_tracking = one_minus_normalized_q16(
        local_yaw_rate.saturating_sub(command_raw[2]).unsigned_abs() as u128,
        if curriculum { 1_500_000 } else { 3_000_000 },
    );
    let planar_tracking = if curriculum {
        q16_square(planar_tracking)?
    } else {
        planar_tracking
    };
    let yaw_tracking = if curriculum {
        q16_square(yaw_tracking)?
    } else {
        yaw_tracking
    };
    let upright = upright_reward_q16(root.rotation_q1_30)?;
    let height_error = root.position_micrometres[1]
        .saturating_sub(REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES)
        .unsigned_abs() as u128;
    let height_tracking =
        one_minus_normalized_q16(height_error, if curriculum { 400_000 } else { 600_000 });
    let vertical_velocity_cost = ratio_q16(
        root.linear_velocity_micrometres_per_second[1].unsigned_abs() as u128,
        if curriculum { 2_000_000 } else { 3_000_000 },
    )?;
    let roll_pitch_rate_cost = ratio_q16(
        abs_sum([frame.observation_raw[7], frame.observation_raw[9]]),
        if curriculum { 4_000_000 } else { 6_000_000 },
    )?;
    let effort_sum = frame
        .substep_efforts
        .iter()
        .flatten()
        .map(|effort| u128::from(effort.effort_micronewton_metres.unsigned_abs()))
        .sum::<u128>();
    let effort_cost = ratio_q16(effort_sum, maximum_effort_per_frame)?;
    let action_rate_sum = frame
        .applied_action_microradians
        .iter()
        .zip(previous_applied_action)
        .map(|(current, previous)| current.saturating_sub(*previous).unsigned_abs() as u128)
        .sum::<u128>();
    let action_rate_denominator = (frame.applied_action_microradians.len() as u128)
        .checked_mul(2_000_000)
        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
    let action_rate_cost = ratio_q16(action_rate_sum, action_rate_denominator)?;
    let contacting_foot_tokens =
        foot_tokens
            .iter()
            .copied()
            .filter(|token| {
                frame.snapshot.contacts.iter().any(|contact| {
                    contact.actor_a_token == *token || contact.actor_b_token == *token
                })
            })
            .collect::<BTreeSet<_>>();
    let slip_sum = frame
        .snapshot
        .links
        .iter()
        .filter(|link| contacting_foot_tokens.contains(&link.user_token))
        .map(|link| {
            abs_sum([
                link.linear_velocity_micrometres_per_second[0],
                link.linear_velocity_micrometres_per_second[2],
            ])
        })
        .sum::<u128>();
    let slip_denominator = (contacting_foot_tokens.len() as u128)
        .checked_mul(if curriculum { 2_000_000 } else { 4_000_000 })
        .unwrap_or(0);
    let slip_cost = if slip_denominator == 0 {
        0
    } else {
        ratio_q16(slip_sum, slip_denominator)?
    };
    let base_values = [
        planar_tracking,
        yaw_tracking,
        upright,
        height_tracking,
        vertical_velocity_cost,
        roll_pitch_rate_cost,
        effort_cost,
        action_rate_cost,
        slip_cost,
    ];
    let moving = curriculum_command_is_moving(command_raw);
    let support = if (moving && contacting_foot_tokens.len() == 1)
        || (!moving && contacting_foot_tokens.len() == 2)
    {
        65_536
    } else {
        0
    };
    let values = if curriculum {
        base_values
            .into_iter()
            .chain([support, i64::from(fell) * 65_536])
            .collect::<Vec<_>>()
    } else {
        base_values
            .into_iter()
            .chain([i64::from(fell) * 65_536])
            .collect::<Vec<_>>()
    };
    let coefficients: &[i64] = if curriculum {
        &CURRICULUM_LOCOMOTION_REWARD_COEFFICIENTS_Q16
    } else {
        &LOCOMOTION_REWARD_COEFFICIENTS_Q16
    };
    let component_ids: &[&str] = if curriculum {
        &CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS
    } else {
        &LOCOMOTION_REWARD_COMPONENT_IDS
    };
    let reward_total_q16 =
        values
            .iter()
            .zip(coefficients)
            .try_fold(0_i64, |total, (component, coefficient)| {
                let weighted = round_shift_ties_even_i128(
                    i128::from(*component)
                        .checked_mul(i128::from(*coefficient))
                        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?,
                    16,
                )?;
                total
                    .checked_add(weighted)
                    .ok_or(TrainingEnvironmentError::ArithmeticOverflow)
            })?;
    Ok((
        component_ids
            .iter()
            .copied()
            .zip(values)
            .map(|(component_id, value)| (schema_id(component_id), value))
            .collect(),
        reward_total_q16,
    ))
}

pub(super) fn curriculum_command_is_moving(command_raw: [i64; 3]) -> bool {
    match CURRICULUM_SUPPORT_COMMAND_MODE_V2 {
        CurriculumSupportCommandModeV2::ExactZero => command_raw != [0; 3],
    }
}

fn q16_square(value: i64) -> Result<i64, TrainingEnvironmentError> {
    round_shift_ties_even_i128(
        i128::from(value)
            .checked_mul(i128::from(value))
            .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?,
        16,
    )
}

pub(super) fn upright_reward_q16(
    rotation_q1_30: [i64; 4],
) -> Result<i64, TrainingEnvironmentError> {
    let [x, _, z, _] = rotation_q1_30;
    let tilt_reduction_q30 = i128::from(x)
        .checked_mul(i128::from(x))
        .and_then(|value| {
            i128::from(z)
                .checked_mul(i128::from(z))
                .and_then(|other| value.checked_add(other))
        })
        .and_then(|value| value.checked_mul(2))
        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
    let tilt_reduction_q30 = round_shift_ties_even_i128(tilt_reduction_q30, 30)?;
    let upright_q30 = (1_i64 << 30)
        .saturating_sub(tilt_reduction_q30)
        .clamp(0, 1_i64 << 30);
    ratio_q16(upright_q30 as u128, 1_u128 << 30)
}

pub(super) fn standing_reward_components(
    frame: &MotorFrameResult,
    action_microradians: &[i64],
    previous_action_microradians: &[i64],
) -> Vec<(SchemaId, i64)> {
    let root = frame.snapshot.links.first();
    let upright = root.map_or(0, |root| root.rotation_q1_30[3].unsigned_abs() as i64);
    let root_height_tracking = root.map_or(
        -REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES,
        |root| {
            -unsigned_sum([root.position_micrometres[1]
                .saturating_sub(REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES)])
        },
    );
    let standing_pose_tracking = -unsigned_sum(
        frame
            .snapshot
            .joints
            .iter()
            .map(|joint| joint.position_microradians),
    );
    let velocity_penalty = root.map_or(-1, |root| {
        -unsigned_sum(
            root.linear_velocity_micrometres_per_second
                .into_iter()
                .chain(root.angular_velocity_microradians_per_second),
        )
    });
    let effort_penalty = -(frame
        .substep_efforts
        .iter()
        .flatten()
        .map(|effort| effort.effort_micronewton_metres.unsigned_abs() / 1_000_000)
        .sum::<u64>()
        .min(i64::MAX as u64) as i64);
    let action_rate_penalty = -unsigned_sum(
        action_microradians
            .iter()
            .zip(previous_action_microradians)
            .map(|(current, previous)| current.saturating_sub(*previous)),
    );
    let contacting_tokens = frame
        .snapshot
        .contacts
        .iter()
        .flat_map(|contact| [contact.actor_a_token, contact.actor_b_token])
        .filter(|token| *token != 1)
        .collect::<BTreeSet<_>>();
    let foot_slip_penalty = -unsigned_sum(
        frame
            .snapshot
            .links
            .iter()
            .filter(|link| contacting_tokens.contains(&link.user_token))
            .flat_map(|link| {
                [
                    link.linear_velocity_micrometres_per_second[0],
                    link.linear_velocity_micrometres_per_second[2],
                ]
            }),
    );
    let fall_terminal = -i64::from(root.is_none_or(|root| root.position_micrometres[1] <= 250_000));
    [
        upright,
        root_height_tracking,
        standing_pose_tracking,
        velocity_penalty,
        effort_penalty,
        action_rate_penalty,
        foot_slip_penalty,
        fall_terminal,
    ]
    .into_iter()
    .zip(STANDING_REWARD_COMPONENT_IDS.map(schema_id))
    .map(|(value, id)| (id, value))
    .collect()
}
