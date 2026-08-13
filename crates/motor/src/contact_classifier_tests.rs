use next_contracts::body::BodyContactRoleV2;
use next_contracts::ids::PersistentId;
use next_physics_physx::{CanonicalPhysXContactV2, CanonicalPhysXSnapshotV2};

use crate::contact_classifier::*;
use crate::{CompiledBodySchemaV2, biomechanics_humanoid_body_schema_v2};

struct Harness {
    compiled: CompiledBodySchemaV2,
    classifier: BiomechanicsContactClassifier,
}

impl Harness {
    fn new() -> Self {
        let compiled = CompiledBodySchemaV2::compile(
            &biomechanics_humanoid_body_schema_v2(),
            PersistentId::from_bytes([41; 16]),
        )
        .expect("compile biomechanics profile");
        let classifier =
            BiomechanicsContactClassifier::new(&compiled).expect("construct classifier");
        Self {
            compiled,
            classifier,
        }
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

    fn classify(
        &mut self,
        contacts: Vec<CanonicalPhysXContactV2>,
        profile: BiomechanicsSkillContactProfileV1,
    ) -> BiomechanicsContactFrameV1 {
        self.classifier
            .classify_substep(&snapshot(contacts), profile)
            .expect("classify contacts")
    }
}

fn snapshot(contacts: Vec<CanonicalPhysXContactV2>) -> CanonicalPhysXSnapshotV2 {
    CanonicalPhysXSnapshotV2 {
        links: Vec::new(),
        joints: Vec::new(),
        contacts,
    }
}

fn ground_contact(
    actor: u64,
    shape: u64,
    impulse: [i64; 3],
    separation: i64,
) -> CanonicalPhysXContactV2 {
    CanonicalPhysXContactV2 {
        actor_a_token: HUMANOID_GROUND_ACTOR_TOKEN,
        actor_b_token: actor,
        shape_a_token: HUMANOID_GROUND_SHAPE_TOKEN,
        shape_b_token: shape,
        position_micrometres: [0; 3],
        normal_q1_30: [0, 1 << 30, 0],
        impulse_micronewton_seconds: impulse,
        separation_micrometres: separation,
    }
}

fn reversed(mut contact: CanonicalPhysXContactV2) -> CanonicalPhysXContactV2 {
    std::mem::swap(&mut contact.actor_a_token, &mut contact.actor_b_token);
    std::mem::swap(&mut contact.shape_a_token, &mut contact.shape_b_token);
    contact.normal_q1_30 = contact.normal_q1_30.map(|value| -value);
    contact.impulse_micronewton_seconds = contact.impulse_micronewton_seconds.map(|value| -value);
    contact
}

#[test]
fn soles_support_while_speculative_pairs_are_not_contacts() {
    let mut harness = Harness::new();
    let (actor, shape) = harness.shape(BodyContactRoleV2::FootWithSoleFeature);
    let frame = harness.classify(
        vec![ground_contact(actor, shape, [50_000, 0, 0], 1)],
        BiomechanicsSkillContactProfileV1::Locomotion,
    );
    assert_eq!(frame.contacts.len(), 1);
    assert_eq!(
        frame.contacts[0].class,
        BiomechanicsContactClassV1::SoleSupport
    );
    let empty = harness.classify(
        vec![ground_contact(actor, shape, [0; 3], 1)],
        BiomechanicsSkillContactProfileV1::Locomotion,
    );
    assert!(empty.contacts.is_empty());
    assert_eq!(
        empty.continuity_root,
        Harness::new().classifier.continuity_root()
    );
}

#[test]
fn fifth_low_impulse_hand_substep_is_forbidden_and_one_gap_clears_grace() {
    let mut harness = Harness::new();
    let (actor, shape) = harness.shape(BodyContactRoleV2::HandGround);
    let contact = ground_contact(actor, shape, [100_000, 0, 0], 0);
    for expected in 1..=LOW_IMPULSE_GRACE_SUBSTEPS {
        let frame = harness.classify(
            vec![contact.clone()],
            BiomechanicsSkillContactProfileV1::Locomotion,
        );
        assert_eq!(frame.contacts[0].consecutive_active_substeps, expected);
        assert_eq!(
            frame.contacts[0].class,
            BiomechanicsContactClassV1::TransientAllowed
        );
    }
    let forbidden = harness.classify(
        vec![contact.clone()],
        BiomechanicsSkillContactProfileV1::Locomotion,
    );
    assert_eq!(
        forbidden.contacts[0].class,
        BiomechanicsContactClassV1::ForbiddenLocomotion
    );
    harness.classify(Vec::new(), BiomechanicsSkillContactProfileV1::Locomotion);
    let restarted = harness.classify(vec![contact], BiomechanicsSkillContactProfileV1::Locomotion);
    assert_eq!(restarted.contacts[0].consecutive_active_substeps, 1);
    assert_eq!(
        restarted.contacts[0].class,
        BiomechanicsContactClassV1::TransientAllowed
    );
}

#[test]
fn all_nonsole_roles_become_immediately_forbidden_above_brush_ceiling() {
    let mut harness = Harness::new();
    let roles = [
        BodyContactRoleV2::PelvisGround,
        BodyContactRoleV2::TorsoGround,
        BodyContactRoleV2::HeadGround,
        BodyContactRoleV2::ThighGround,
        BodyContactRoleV2::ShankGround,
        BodyContactRoleV2::KneeGround,
        BodyContactRoleV2::AnkleGround,
        BodyContactRoleV2::UpperArmGround,
        BodyContactRoleV2::ForearmGround,
        BodyContactRoleV2::HandGround,
    ];
    for role in roles {
        harness.classifier.reset();
        let (actor, shape) = harness.shape(role);
        let frame = harness.classify(
            vec![ground_contact(actor, shape, [250_001, 0, 0], 0)],
            BiomechanicsSkillContactProfileV1::Locomotion,
        );
        let projected_role = match role {
            BodyContactRoleV2::TorsoGround => BodyContactRoleV2::HeadGround,
            BodyContactRoleV2::KneeGround => BodyContactRoleV2::ShankGround,
            BodyContactRoleV2::HandGround => BodyContactRoleV2::ForearmGround,
            _ => role,
        };
        assert_eq!(frame.contacts[0].primary_role, projected_role);
        assert_eq!(
            frame.contacts[0].class,
            BiomechanicsContactClassV1::ForbiddenLocomotion
        );
        assert!(!frame.contacts[0].hard_impact_violation);
    }
}

#[test]
fn shared_bodies_use_the_strict_projected_role_in_recovery_reports() {
    let mut harness = Harness::new();
    let (hand_actor, hand_shape) = harness.shape(BodyContactRoleV2::HandGround);
    let brace = harness.classify(
        vec![ground_contact(hand_actor, hand_shape, [300_000, 0, 0], 0)],
        BiomechanicsSkillContactProfileV1::BraceFall,
    );
    assert_eq!(
        brace.contacts[0].class,
        BiomechanicsContactClassV1::BraceSupport
    );
    assert_eq!(
        brace.contacts[0].primary_role,
        BodyContactRoleV2::ForearmGround
    );

    harness.classifier.reset();
    let (knee_actor, knee_shape) = harness.shape(BodyContactRoleV2::KneeGround);
    let getup = harness.classify(
        vec![ground_contact(knee_actor, knee_shape, [300_000, 0, 0], 0)],
        BiomechanicsSkillContactProfileV1::GetUp,
    );
    assert_eq!(
        getup.contacts[0].class,
        BiomechanicsContactClassV1::TransientAllowed
    );
    assert_eq!(
        getup.contacts[0].primary_role,
        BodyContactRoleV2::ShankGround
    );

    harness.classifier.reset();
    let (torso_actor, torso_shape) = harness.shape(BodyContactRoleV2::TorsoGround);
    let recovery = harness.classify(
        vec![ground_contact(torso_actor, torso_shape, [300_000, 0, 0], 0)],
        BiomechanicsSkillContactProfileV1::GetUp,
    );
    assert_eq!(
        recovery.contacts[0].class,
        BiomechanicsContactClassV1::TransientAllowed
    );
    assert_eq!(
        recovery.contacts[0].primary_role,
        BodyContactRoleV2::HeadGround
    );
}

#[test]
fn authored_shapes_on_one_body_reduce_to_one_strict_body_pair() {
    let mut harness = Harness::new();
    let (torso_actor, torso_shape) = harness.shape(BodyContactRoleV2::TorsoGround);
    let (head_actor, head_shape) = harness.shape(BodyContactRoleV2::HeadGround);
    assert_eq!(torso_actor, head_actor);

    let frame = harness.classify(
        vec![
            ground_contact(torso_actor, torso_shape, [600_000, 0, 0], 2),
            ground_contact(head_actor, head_shape, [600_001, 0, 0], -3),
        ],
        BiomechanicsSkillContactProfileV1::Locomotion,
    );

    assert_eq!(frame.contacts.len(), 1);
    assert_eq!(
        frame.contacts[0].primary_role,
        BodyContactRoleV2::HeadGround
    );
    assert_eq!(
        frame.contacts[0].impulse_micronewton_seconds,
        [1_200_001, 0, 0]
    );
    assert_eq!(frame.contacts[0].minimum_separation_micrometres, -3);
    assert!(frame.contacts[0].hard_impact_violation);
}

#[test]
fn hard_impact_budget_has_no_grace_and_self_contact_uses_material_rule() {
    let mut harness = Harness::new();
    let (head_actor, head_shape) = harness.shape(BodyContactRoleV2::HeadGround);
    let head = harness.classify(
        vec![ground_contact(head_actor, head_shape, [1_000_001, 0, 0], 0)],
        BiomechanicsSkillContactProfileV1::GetUp,
    );
    assert!(head.contacts[0].hard_impact_violation);
    assert_eq!(head.contacts[0].consecutive_active_substeps, 1);

    harness.classifier.reset();
    let (hand_actor, hand_shape) = harness.shape(BodyContactRoleV2::HandGround);
    let (torso_actor, torso_shape) = harness.shape(BodyContactRoleV2::TorsoGround);
    let self_contact = CanonicalPhysXContactV2 {
        actor_a_token: hand_actor,
        actor_b_token: torso_actor,
        shape_a_token: hand_shape,
        shape_b_token: torso_shape,
        position_micrometres: [0; 3],
        normal_q1_30: [1 << 30, 0, 0],
        impulse_micronewton_seconds: [300_000, 0, 0],
        separation_micrometres: -1,
    };
    let frame = harness.classify(
        vec![self_contact],
        BiomechanicsSkillContactProfileV1::BraceFall,
    );
    assert_eq!(
        frame.contacts[0].class,
        BiomechanicsContactClassV1::SelfCollisionViolation
    );
}

#[test]
fn manifold_points_and_callback_order_produce_one_exact_pair_root() {
    let mut forward = Harness::new();
    let mut reverse = Harness::new();
    let (actor, shape) = forward.shape(BodyContactRoleV2::HandGround);
    let first = ground_contact(actor, shape, [30_000, 0, 0], 4);
    let second = ground_contact(actor, shape, [40_000, 0, 0], -1);
    let forward_frame = forward.classify(
        vec![first.clone(), second.clone()],
        BiomechanicsSkillContactProfileV1::Locomotion,
    );
    let reverse_frame = reverse.classify(
        vec![reversed(second), reversed(first)],
        BiomechanicsSkillContactProfileV1::Locomotion,
    );
    assert_eq!(forward_frame, reverse_frame);
    assert_eq!(forward_frame.contacts.len(), 1);
    assert_eq!(
        forward_frame.contacts[0].impulse_micronewton_seconds,
        [70_000, 0, 0]
    );
    assert_eq!(forward_frame.contacts[0].consecutive_active_substeps, 1);
}

#[test]
fn invalid_late_pair_does_not_mutate_continuity() {
    let mut harness = Harness::new();
    let (actor, shape) = harness.shape(BodyContactRoleV2::HandGround);
    harness.classify(
        vec![ground_contact(actor, shape, [100_000, 0, 0], 0)],
        BiomechanicsSkillContactProfileV1::Locomotion,
    );
    let before = harness.classifier.continuity_root();
    let invalid = ground_contact(actor, u64::MAX, [100_000, 0, 0], 0);
    assert_eq!(
        harness.classifier.classify_substep(
            &snapshot(vec![
                ground_contact(actor, shape, [100_000, 0, 0], 0),
                invalid,
            ]),
            BiomechanicsSkillContactProfileV1::Locomotion,
        ),
        Err(ContactClassificationError::UnknownShape)
    );
    assert_eq!(harness.classifier.continuity_root(), before);
}

#[test]
fn reset_clears_penetration_only_continuity_exactly() {
    let mut harness = Harness::new();
    let pristine = harness.classifier.continuity_root();
    let (actor, shape) = harness.shape(BodyContactRoleV2::KneeGround);
    let frame = harness.classify(
        vec![ground_contact(actor, shape, [0; 3], -1)],
        BiomechanicsSkillContactProfileV1::GetUp,
    );
    assert_ne!(frame.continuity_root, pristine);
    harness.classifier.reset();
    assert_eq!(harness.classifier.continuity_root(), pristine);
}
