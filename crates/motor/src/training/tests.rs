use super::*;

fn standing_input(slot: u32, action: i64) -> VectorStepInput {
    VectorStepInput {
        vector_slot: slot,
        action_microradians: vec![action; 23],
        command_raw: [100_000, 0, 0],
    }
}

fn locomotion_input(slot: u32, episode: u64, action: i64) -> VectorPolicyStepInput {
    VectorPolicyStepInput {
        vector_slot: slot,
        episode_ordinal: episode,
        action_microradians: vec![action; 23],
    }
}

#[test]
fn standing_publication_order_and_behavior_ignore_input_permutation() {
    let run_root = ContentHash::from_bytes([4; 32]);
    let mut first = MotorVectorRunner::create(4, run_root).expect("first runner");
    let mut second = MotorVectorRunner::create(4, run_root).expect("second runner");
    assert_eq!(
        first.reset_all(17).expect("first reset"),
        second.reset_all(17).expect("second reset")
    );
    let ascending = vec![
        standing_input(0, 1),
        standing_input(1, 2),
        standing_input(2, 3),
        standing_input(3, 4),
    ];
    let descending = vec![
        standing_input(3, 4),
        standing_input(2, 3),
        standing_input(1, 2),
        standing_input(0, 1),
    ];
    let left = first.step_lockstep(ascending).expect("ascending");
    let right = second.step_lockstep(descending).expect("descending");
    assert_eq!(left, right);
}

#[test]
fn command_schedule_has_warmup_bounds_modes_and_rate_limits() {
    let schedule = flat_locomotion_command_schedule([9; 32]).expect("schedule");
    assert_eq!(schedule.len(), 1_201);
    assert!(schedule[..60].iter().all(|command| *command == [0; 3]));
    for pair in schedule.windows(2) {
        assert!(pair[1][0].abs_diff(pair[0][0]) <= 50_000);
        assert!(pair[1][1].abs_diff(pair[0][1]) <= 50_000);
        assert!(pair[1][2].abs_diff(pair[0][2]) <= 25_000);
        assert!((-2_000_000..=2_000_000).contains(&pair[1][0]));
        assert!((-1_500_000..=3_000_000).contains(&pair[1][1]));
        assert!((-1_500_000..=1_500_000).contains(&pair[1][2]));
    }
    assert_eq!(
        schedule,
        flat_locomotion_command_schedule([9; 32]).expect("repeat")
    );
    assert_eq!(
        [schedule[60], schedule[61], schedule[180], schedule[1_200]],
        [
            [50_000, -50_000, 0],
            [100_000, -100_000, 0],
            [621_567, -592_654, 0],
            [-1_191_367, 2_721_543, -1_298_134],
        ]
    );
}

#[test]
fn one_million_aggregate_command_steps_remain_bounded() {
    let mut aggregate_steps = 0_u64;
    for ordinal in 0_u64..834 {
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"motor-locomotion-million-step-test\0");
        preimage.extend_from_slice(&ordinal.to_le_bytes());
        let schedule = flat_locomotion_command_schedule(sha256(&preimage)).expect("schedule");
        aggregate_steps += schedule.len() as u64;
        assert!(schedule.iter().all(|command| {
            (-2_000_000..=2_000_000).contains(&command[0])
                && (-1_500_000..=3_000_000).contains(&command[1])
                && (-1_500_000..=1_500_000).contains(&command[2])
        }));
    }
    assert!(aggregate_steps >= 1_000_000);
}

#[test]
fn reward_tracking_is_monotonic_and_upright_ignores_yaw() {
    assert!(
        one_minus_normalized_q16(100_000, 3_000_000) > one_minus_normalized_q16(500_000, 3_000_000)
    );
    let yaw_half_sqrt_q30 = 759_250_125;
    assert_eq!(
        upright_reward_q16([0, 0, 0, 1 << 30]).expect("identity"),
        upright_reward_q16([0, yaw_half_sqrt_q30, 0, yaw_half_sqrt_q30]).expect("yaw")
    );
    assert!(
        upright_reward_q16([yaw_half_sqrt_q30, 0, 0, yaw_half_sqrt_q30]).expect("roll") < 65_536
    );
    assert!(!LOCOMOTION_REWARD_COMPONENT_IDS.contains(&"reward.standing-pose-tracking"));
    assert!(
        LOCOMOTION_REWARD_COEFFICIENTS_Q16[4..]
            .iter()
            .all(|value| *value < 0)
    );
}

#[test]
fn action_rate_reward_uses_applied_not_requested_action() {
    let mut runner = MotorVectorRunner::create_profile(
        FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
        1,
        ContentHash::from_bytes([13; 32]),
    )
    .expect("runner");
    runner.reset_slots(&[0]).expect("reset");
    let output = runner
        .step_actions_lockstep(vec![locomotion_input(0, 1, 2_000_000)])
        .expect("step");
    assert!(
        output[0]
            .frame
            .applied_action_microradians
            .iter()
            .all(|value| *value == 1_000_000)
    );
    assert_eq!(output[0].reward_components_raw[7].1, 32_768);
}

#[test]
fn terminal_slots_reject_steps_until_partial_reset() {
    let mut runner = MotorVectorRunner::create_profile(
        FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
        1,
        ContentHash::from_bytes([15; 32]),
    )
    .expect("runner");
    runner.reset_slots(&[0]).expect("reset");
    let mut terminal = None;
    for _ in 0..FLAT_LOCOMOTION_MAX_EPISODE_MOTOR_STEPS {
        let output = runner
            .step_actions_lockstep(vec![locomotion_input(0, 1, 0)])
            .expect("step");
        if output[0].terminated || output[0].truncated {
            terminal = Some((output[0].terminated, output[0].truncated));
            break;
        }
    }
    assert!(terminal.is_some());
    let error = runner
        .step_actions_lockstep(vec![locomotion_input(0, 1, 0)])
        .expect_err("post-terminal step");
    assert_eq!(error.stable_code(), "MOTOR_ENV_SLOT_TERMINAL");
    assert_eq!(
        runner.reset_slots(&[0]).expect("reset again")[0].episode_ordinal,
        2
    );
}

#[test]
fn partial_resets_increment_only_selected_episode_ordinals() {
    let mut runner = MotorVectorRunner::create_profile(
        FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
        3,
        ContentHash::from_bytes([7; 32]),
    )
    .expect("runner");
    let first = runner.reset_slots(&[2, 0, 1]).expect("first reset");
    assert!(first.iter().all(|value| value.episode_ordinal == 1));
    let second = runner.reset_slots(&[1]).expect("partial reset");
    assert_eq!(second[0].episode_ordinal, 2);
    let error = runner
        .step_actions_lockstep(vec![
            locomotion_input(0, 1, 0),
            locomotion_input(1, 1, 0),
            locomotion_input(2, 1, 0),
        ])
        .expect_err("stale episode");
    assert_eq!(error.stable_code(), "MOTOR_ENV_EPISODE_STALE");
}

#[test]
fn locomotion_input_permutations_match_and_invalid_batches_are_atomic() {
    let run_root = ContentHash::from_bytes([10; 32]);
    let mut first =
        MotorVectorRunner::create_profile(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, 3, run_root)
            .expect("first");
    let mut second =
        MotorVectorRunner::create_profile(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, 3, run_root)
            .expect("second");
    assert_eq!(
        first.reset_slots(&[0, 1, 2]).expect("first reset"),
        second.reset_slots(&[2, 1, 0]).expect("second reset")
    );
    let before = first
        .checkpoint_slot(0, 1)
        .expect("before")
        .canonical_bytes()
        .expect("encode");
    assert_eq!(
        first
            .step_actions_lockstep(vec![
                locomotion_input(0, 1, 1),
                locomotion_input(0, 1, 2),
                locomotion_input(2, 1, 3),
            ])
            .expect_err("duplicate")
            .stable_code(),
        "MOTOR_ENV_SLOT_IDENTITY_INVALID"
    );
    assert_eq!(
        first
            .checkpoint_slot(0, 1)
            .expect("after")
            .canonical_bytes()
            .expect("encode"),
        before
    );
    let left = first
        .step_actions_lockstep(vec![
            locomotion_input(2, 1, 3),
            locomotion_input(0, 1, 1),
            locomotion_input(1, 1, 2),
        ])
        .expect("permuted");
    let right = second
        .step_actions_lockstep(vec![
            locomotion_input(0, 1, 1),
            locomotion_input(1, 1, 2),
            locomotion_input(2, 1, 3),
        ])
        .expect("ordered");
    assert_eq!(left, right);
}

#[test]
fn checkpoint_restore_continues_byte_exact_and_is_idempotent() {
    let run_root = ContentHash::from_bytes([11; 32]);
    let mut source =
        MotorVectorRunner::create_profile(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, 1, run_root)
            .expect("source");
    source.reset_slots(&[0]).expect("reset");
    for action in [100, -200, 300] {
        source
            .step_actions_lockstep(vec![locomotion_input(0, 1, action)])
            .expect("prefix");
    }
    let checkpoint = source.checkpoint_slot(0, 1).expect("checkpoint");
    let expected = source
        .step_actions_lockstep(vec![locomotion_input(0, 1, 400)])
        .expect("source continuation");

    let mut restored =
        MotorVectorRunner::create_profile(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, 1, run_root)
            .expect("restored");
    restored.restore_slot(&checkpoint).expect("restore");
    restored
        .restore_slot(&checkpoint)
        .expect("idempotent restore");
    let actual = restored
        .step_actions_lockstep(vec![locomotion_input(0, 1, 400)])
        .expect("restored continuation");
    assert_eq!(actual, expected);
}

#[test]
fn purpose_slot_and_episode_seeds_are_domain_separated() {
    let run_root = ContentHash::from_bytes([6; 32]);
    let first = derive_episode_seed_set(run_root, 1, 0).expect("first");
    let second = derive_episode_seed_set(run_root, 1, 1).expect("slot");
    let third = derive_episode_seed_set(run_root, 2, 0).expect("episode");
    assert_ne!(first.purpose_seeds, second.purpose_seeds);
    assert_ne!(first.purpose_seeds, third.purpose_seeds);
    assert_eq!(
        first
            .purpose_seeds
            .iter()
            .map(|(_, seed)| seed)
            .collect::<BTreeSet<_>>()
            .len(),
        RANDOMIZATION_PURPOSES.len()
    );
}
