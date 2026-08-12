use next_contracts::body::BodyContactRoleV2;
use next_contracts::ids::PersistentId;
use next_contracts::motor::MotorTerminalDispositionV1;
use next_physics_physx::{
    CanonicalPhysXContactV2, CanonicalPhysXLinkState, CanonicalPhysXSnapshotV2,
};

use crate::contact_classifier::{
    BiomechanicsContactClassifier, HUMANOID_GROUND_ACTOR_TOKEN, HUMANOID_GROUND_SHAPE_TOKEN,
};
use crate::terminal_v2::*;
use crate::{
    BiomechanicsContactFrameV1, BiomechanicsSkillContactProfileV1, CompiledBodySchemaV2,
    LOW_IMPULSE_GRACE_SUBSTEPS, MotorSafetyError, biomechanics_humanoid_body_schema_v2,
};

struct Harness {
    compiled: CompiledBodySchemaV2,
    classifier: BiomechanicsContactClassifier,
    root_actor: u64,
}

impl Harness {
    fn new() -> Self {
        let compiled = CompiledBodySchemaV2::compile(
            &biomechanics_humanoid_body_schema_v2(),
            PersistentId::from_bytes([51; 16]),
        )
        .expect("compile biomechanics profile");
        let classifier =
            BiomechanicsContactClassifier::new(&compiled).expect("construct classifier");
        let root_actor = compiled.body_tokens[&compiled.construction_order[0]];
        Self {
            compiled,
            classifier,
            root_actor,
        }
    }

    fn evaluator(
        &self,
        profile: BiomechanicsSkillContactProfileV1,
        maximum_ticks: u64,
    ) -> BiomechanicsTerminalEvaluator {
        BiomechanicsTerminalEvaluator::new(&self.compiled, profile, maximum_ticks)
            .expect("construct terminal evaluator")
    }

    fn shape(&self, role: BodyContactRoleV2) -> (u64, u64) {
        let shape = self
            .compiled
            .collider_contact_roles
            .iter()
            .find_map(|(shape, candidate)| (*candidate == role).then_some(*shape))
            .expect("profile contains requested role");
        (self.compiled.collider_body_tokens[&shape], shape)
    }

    fn frames(
        &mut self,
        contacts: &[CanonicalPhysXContactV2],
        profile: BiomechanicsSkillContactProfileV1,
    ) -> Vec<BiomechanicsContactFrameV1> {
        (0..BIOMECHANICS_TERMINAL_SUBSTEPS)
            .map(|_| {
                self.classifier
                    .classify_substep(&snapshot(Vec::new(), contacts.to_vec()), profile)
                    .expect("classify substep")
            })
            .collect()
    }

    fn empty_frames(
        &mut self,
        profile: BiomechanicsSkillContactProfileV1,
    ) -> Vec<BiomechanicsContactFrameV1> {
        self.frames(&[], profile)
    }
}

fn snapshot(
    links: Vec<CanonicalPhysXLinkState>,
    contacts: Vec<CanonicalPhysXContactV2>,
) -> CanonicalPhysXSnapshotV2 {
    CanonicalPhysXSnapshotV2 {
        links,
        joints: Vec::new(),
        contacts,
    }
}

fn upright_root(actor: u64) -> CanonicalPhysXLinkState {
    CanonicalPhysXLinkState {
        user_token: actor,
        position_micrometres: [0, 1_095_000, 0],
        rotation_q1_30: [0, 0, 0, 1 << 30],
        linear_velocity_micrometres_per_second: [0; 3],
        angular_velocity_microradians_per_second: [0; 3],
    }
}

fn ground_contact(actor: u64, shape: u64, impulse: i64) -> CanonicalPhysXContactV2 {
    CanonicalPhysXContactV2 {
        actor_a_token: HUMANOID_GROUND_ACTOR_TOKEN,
        actor_b_token: actor,
        shape_a_token: HUMANOID_GROUND_SHAPE_TOKEN,
        shape_b_token: shape,
        position_micrometres: [0; 3],
        normal_q1_30: [0, 1 << 30, 0],
        impulse_micronewton_seconds: [impulse, 0, 0],
        separation_micrometres: 0,
    }
}

fn self_contact(actor_a: u64, shape_a: u64, actor_b: u64, shape_b: u64) -> CanonicalPhysXContactV2 {
    CanonicalPhysXContactV2 {
        actor_a_token: actor_a,
        actor_b_token: actor_b,
        shape_a_token: shape_a,
        shape_b_token: shape_b,
        position_micrometres: [0; 3],
        normal_q1_30: [1 << 30, 0, 0],
        impulse_micronewton_seconds: [300_000, 0, 0],
        separation_micrometres: -1,
    }
}

#[test]
fn deliberate_hand_knee_and_torso_contacts_terminate_locomotion() {
    for role in [
        BodyContactRoleV2::HandGround,
        BodyContactRoleV2::KneeGround,
        BodyContactRoleV2::TorsoGround,
    ] {
        let mut harness = Harness::new();
        let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1_200);
        let (actor, shape) = harness.shape(role);
        let frames = harness.frames(
            &[ground_contact(actor, shape, 300_000)],
            BiomechanicsSkillContactProfileV1::Locomotion,
        );
        let decision = evaluator
            .evaluate_motor_tick(
                1,
                &snapshot(vec![upright_root(harness.root_actor)], Vec::new()),
                &frames,
                false,
                None,
            )
            .expect("terminal decision");
        assert_eq!(decision.disposition, MotorTerminalDispositionV1::Terminated);
        assert_eq!(
            decision.reason,
            Some(BiomechanicsTerminalReasonV1::ForbiddenLocomotionContact)
        );
    }
}

#[test]
fn same_hand_knee_and_torso_contacts_remain_running_in_recovery_profiles() {
    let cases = [
        (
            BodyContactRoleV2::HandGround,
            BiomechanicsSkillContactProfileV1::BraceFall,
        ),
        (
            BodyContactRoleV2::KneeGround,
            BiomechanicsSkillContactProfileV1::GetUp,
        ),
        (
            BodyContactRoleV2::TorsoGround,
            BiomechanicsSkillContactProfileV1::GetUp,
        ),
    ];
    for (role, profile) in cases {
        let mut harness = Harness::new();
        let mut evaluator = harness.evaluator(profile, 1_200);
        let (actor, shape) = harness.shape(role);
        let frames = harness.frames(&[ground_contact(actor, shape, 300_000)], profile);
        let decision = evaluator
            .evaluate_motor_tick(
                1,
                &snapshot(vec![upright_root(harness.root_actor)], Vec::new()),
                &frames,
                false,
                None,
            )
            .expect("running decision");
        assert_eq!(decision.disposition, MotorTerminalDispositionV1::Running);
        assert_eq!(decision.reason, None);
    }
}

#[test]
fn earlier_substep_violation_cannot_disappear_before_motor_commit() {
    let mut harness = Harness::new();
    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1_200);
    let (actor, shape) = harness.shape(BodyContactRoleV2::HandGround);
    for _ in 0..LOW_IMPULSE_GRACE_SUBSTEPS {
        harness
            .classifier
            .classify_substep(
                &snapshot(Vec::new(), vec![ground_contact(actor, shape, 100_000)]),
                BiomechanicsSkillContactProfileV1::Locomotion,
            )
            .expect("prime grace state");
    }
    let violating = harness
        .classifier
        .classify_substep(
            &snapshot(Vec::new(), vec![ground_contact(actor, shape, 100_000)]),
            BiomechanicsSkillContactProfileV1::Locomotion,
        )
        .expect("fifth substep");
    let mut frames = vec![violating];
    for _ in 1..BIOMECHANICS_TERMINAL_SUBSTEPS {
        frames.push(
            harness
                .classifier
                .classify_substep(
                    &snapshot(Vec::new(), Vec::new()),
                    BiomechanicsSkillContactProfileV1::Locomotion,
                )
                .expect("contact disappears"),
        );
    }
    let decision = evaluator
        .evaluate_motor_tick(
            2,
            &snapshot(vec![upright_root(harness.root_actor)], Vec::new()),
            &frames,
            false,
            None,
        )
        .expect("terminal decision");
    assert_eq!(
        decision.reason,
        Some(BiomechanicsTerminalReasonV1::ForbiddenLocomotionContact)
    );
}

#[test]
fn terminal_priority_is_exact_across_joint_contact_and_pose_failures() {
    let mut harness = Harness::new();
    let (head_actor, head_shape) = harness.shape(BodyContactRoleV2::HeadGround);
    let impact_frames = harness.frames(
        &[ground_contact(head_actor, head_shape, 1_000_001)],
        BiomechanicsSkillContactProfileV1::Locomotion,
    );
    let world_and_fall = CanonicalPhysXLinkState {
        position_micrometres: [90_000_000, 100_000, 0],
        ..upright_root(harness.root_actor)
    };

    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1);
    let nonfinite = evaluator
        .evaluate_motor_tick(
            1,
            &snapshot(vec![world_and_fall.clone()], Vec::new()),
            &impact_frames,
            true,
            Some(MotorSafetyError::VelocityViolation),
        )
        .expect("non-finite priority");
    assert_eq!(
        nonfinite.reason,
        Some(BiomechanicsTerminalReasonV1::NonFiniteState)
    );

    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1);
    let joint = evaluator
        .evaluate_motor_tick(
            1,
            &snapshot(vec![world_and_fall.clone()], Vec::new()),
            &impact_frames,
            false,
            Some(MotorSafetyError::VelocityViolation),
        )
        .expect("joint priority");
    assert_eq!(
        joint.reason,
        Some(BiomechanicsTerminalReasonV1::JointSafety)
    );

    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1);
    let impact = evaluator
        .evaluate_motor_tick(
            1,
            &snapshot(vec![world_and_fall.clone()], Vec::new()),
            &impact_frames,
            false,
            None,
        )
        .expect("impact priority");
    assert_eq!(
        impact.reason,
        Some(BiomechanicsTerminalReasonV1::ContactImpact)
    );

    harness.classifier.reset();
    let (hand_actor, hand_shape) = harness.shape(BodyContactRoleV2::HandGround);
    let (torso_actor, torso_shape) = harness.shape(BodyContactRoleV2::TorsoGround);
    let self_and_forbidden = harness.frames(
        &[
            self_contact(hand_actor, hand_shape, torso_actor, torso_shape),
            ground_contact(hand_actor, hand_shape, 300_000),
        ],
        BiomechanicsSkillContactProfileV1::Locomotion,
    );
    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1);
    let self_collision = evaluator
        .evaluate_motor_tick(
            1,
            &snapshot(vec![world_and_fall.clone()], Vec::new()),
            &self_and_forbidden,
            false,
            None,
        )
        .expect("self-collision priority");
    assert_eq!(
        self_collision.reason,
        Some(BiomechanicsTerminalReasonV1::SelfCollision)
    );

    harness.classifier.reset();
    let empty = harness.empty_frames(BiomechanicsSkillContactProfileV1::Locomotion);
    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1);
    let world = evaluator
        .evaluate_motor_tick(
            1,
            &snapshot(vec![world_and_fall], Vec::new()),
            &empty,
            false,
            None,
        )
        .expect("world priority");
    assert_eq!(
        world.reason,
        Some(BiomechanicsTerminalReasonV1::WorldBounds)
    );
}

#[test]
fn fall_orientation_and_timeout_have_distinct_dispositions() {
    let mut harness = Harness::new();
    let empty = harness.empty_frames(BiomechanicsSkillContactProfileV1::Locomotion);
    let mut tilted = upright_root(harness.root_actor);
    tilted.rotation_q1_30 = [1 << 29, 0, 0, 929_887_697];
    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1_200);
    let fall = evaluator
        .evaluate_motor_tick(1, &snapshot(vec![tilted], Vec::new()), &empty, false, None)
        .expect("fall decision");
    assert_eq!(fall.disposition, MotorTerminalDispositionV1::Terminated);
    assert_eq!(fall.reason, Some(BiomechanicsTerminalReasonV1::Fall));

    let mut harness = Harness::new();
    let empty = harness.empty_frames(BiomechanicsSkillContactProfileV1::Locomotion);
    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1_200);
    let timeout = evaluator
        .evaluate_motor_tick(
            1_200,
            &snapshot(vec![upright_root(harness.root_actor)], Vec::new()),
            &empty,
            false,
            None,
        )
        .expect("timeout decision");
    assert_eq!(timeout.disposition, MotorTerminalDispositionV1::Truncated);
    assert_eq!(timeout.reason, Some(BiomechanicsTerminalReasonV1::Timeout));
}

#[test]
fn malformed_or_missing_root_is_nonfinite_state() {
    let mut harness = Harness::new();
    let empty = harness.empty_frames(BiomechanicsSkillContactProfileV1::Locomotion);
    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1_200);
    let decision = evaluator
        .evaluate_motor_tick(1, &snapshot(Vec::new(), Vec::new()), &empty, false, None)
        .expect("missing root terminal");
    assert_eq!(
        decision.reason,
        Some(BiomechanicsTerminalReasonV1::NonFiniteState)
    );

    let mut harness = Harness::new();
    let empty = harness.empty_frames(BiomechanicsSkillContactProfileV1::Locomotion);
    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1_200);
    let mut malformed = upright_root(harness.root_actor);
    malformed.rotation_q1_30 = [0; 4];
    let decision = evaluator
        .evaluate_motor_tick(
            1,
            &snapshot(vec![malformed], Vec::new()),
            &empty,
            false,
            None,
        )
        .expect("malformed root terminal");
    assert_eq!(
        decision.reason,
        Some(BiomechanicsTerminalReasonV1::NonFiniteState)
    );
}

#[test]
fn contact_frames_from_another_skill_profile_are_rejected_atomically() {
    let mut harness = Harness::new();
    let recovery_frames = harness.empty_frames(BiomechanicsSkillContactProfileV1::GetUp);
    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1_200);
    let pristine = evaluator.terminal_state_root();
    assert_eq!(
        evaluator.evaluate_motor_tick(
            1,
            &snapshot(vec![upright_root(harness.root_actor)], Vec::new()),
            &recovery_frames,
            false,
            None,
        ),
        Err(BiomechanicsTerminalError::ContactProfileMismatch)
    );
    assert_eq!(evaluator.terminal_state_root(), pristine);
}

#[test]
fn terminal_latches_until_reset_and_invalid_substep_count_is_atomic() {
    let mut harness = Harness::new();
    let empty = harness.empty_frames(BiomechanicsSkillContactProfileV1::Locomotion);
    let mut evaluator = harness.evaluator(BiomechanicsSkillContactProfileV1::Locomotion, 1);
    let pristine = evaluator.terminal_state_root();
    assert_eq!(
        evaluator.evaluate_motor_tick(
            1,
            &snapshot(vec![upright_root(harness.root_actor)], Vec::new()),
            &empty[..3],
            false,
            None,
        ),
        Err(BiomechanicsTerminalError::SubstepCount)
    );
    assert_eq!(evaluator.terminal_state_root(), pristine);
    evaluator
        .evaluate_motor_tick(
            1,
            &snapshot(vec![upright_root(harness.root_actor)], Vec::new()),
            &empty,
            false,
            None,
        )
        .expect("latch timeout");
    assert_ne!(evaluator.terminal_state_root(), pristine);
    assert_eq!(
        evaluator.evaluate_motor_tick(
            2,
            &snapshot(vec![upright_root(harness.root_actor)], Vec::new()),
            &empty,
            false,
            None,
        ),
        Err(BiomechanicsTerminalError::EpisodeFinished)
    );
    evaluator.reset();
    assert_eq!(evaluator.terminal_state_root(), pristine);
}

#[test]
fn terminal_reason_ids_are_stable() {
    assert_eq!(
        BiomechanicsTerminalReasonV1::NonFiniteState.stable_id(),
        "terminal.non-finite-state"
    );
    assert_eq!(
        BiomechanicsTerminalReasonV1::ForbiddenLocomotionContact.stable_id(),
        "terminal.forbidden-locomotion-contact"
    );
    assert_eq!(
        BiomechanicsTerminalReasonV1::Timeout.stable_id(),
        "terminal.timeout"
    );
}
