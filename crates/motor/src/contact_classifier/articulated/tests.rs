use super::*;
use crate::{
    BiomechanicsProceduralStandingControllerV3, BiomechanicsTerminalEvaluator, CompiledBodySchemaV4,
};
use next_contracts::ids::PersistentId;
use next_physics_physx::CanonicalPhysXLinkState;

fn compiled(subject: u8) -> CompiledBodySchemaV4 {
    CompiledBodySchemaV4::compile(
        &crate::biomechanics_humanoid_body_schema_v8(),
        PersistentId::from_bytes([subject; 16]),
    )
    .unwrap()
}

fn snapshot(contacts: Vec<CanonicalPhysXContactV2>) -> CanonicalPhysXSnapshotV2 {
    CanonicalPhysXSnapshotV2 {
        links: vec![CanonicalPhysXLinkState {
            user_token: 1000,
            position_micrometres: [0, 1_000_000, 0],
            rotation_q1_30: [0, 0, 0, 1 << 30],
            linear_velocity_micrometres_per_second: [0; 3],
            angular_velocity_microradians_per_second: [0; 3],
        }],
        joints: vec![],
        contacts,
    }
}

fn contact(
    body: &CompiledBodySchemaV4,
    side: &str,
    segment: &str,
    impulse: i64,
) -> CanonicalPhysXContactV2 {
    let base = &body.base.base;
    let actor = *base
        .body_tokens
        .iter()
        .find(|(id, _)| id.as_str() == format!("body.{side}-{segment}"))
        .unwrap()
        .1;
    let shape = *base
        .collider_body_tokens
        .iter()
        .find(|(_, a)| **a == actor)
        .unwrap()
        .0;
    CanonicalPhysXContactV2 {
        actor_a_token: 1,
        shape_a_token: 0,
        actor_b_token: actor,
        shape_b_token: shape,
        position_micrometres: [0; 3],
        normal_q1_30: [0, 1 << 30, 0],
        impulse_micronewton_seconds: [0, impulse, 0],
        separation_micrometres: 0,
    }
}

fn classify(
    classifier: &mut BiomechanicsContactClassifierV2,
    contacts: Vec<CanonicalPhysXContactV2>,
) -> BiomechanicsContactFrameV1 {
    classifier
        .classify_substep(
            &snapshot(contacts),
            BiomechanicsSkillContactProfileV1::Locomotion,
        )
        .unwrap()
}

#[test]
fn anatomical_limit_is_inclusive_separate_by_side_and_includes_inactive_pairs() {
    let body = compiled(0);
    for (rear, toe, violation) in [
        (3_000_000, 3_000_000, false),
        (3_000_000, 3_000_001, true),
        (5_990_000, 20_000, true),
        (6_000_001, -1_000_000, true),
    ] {
        let mut c = BiomechanicsContactClassifierV2::new(&body).unwrap();
        let frame = classify(
            &mut c,
            vec![
                contact(&body, "left", "ankle-roll", rear),
                contact(&body, "left", "mtp", toe),
            ],
        );
        assert_eq!(
            frame.contacts.iter().any(|c| c.hard_impact_violation),
            violation
        );
        assert_eq!(
            c.foot_impulses_micronewton_seconds()[0],
            [0, i128::from(rear + toe), 0]
        );
    }
    let mut c = BiomechanicsContactClassifierV2::new(&body).unwrap();
    let frame = classify(
        &mut c,
        vec![
            contact(&body, "left", "mtp", 4_000_000),
            contact(&body, "right", "mtp", 4_000_000),
        ],
    );
    assert!(!frame.contacts.iter().any(|c| c.hard_impact_violation));
}

#[test]
fn reversal_manifold_partition_and_order_do_not_change_foot_evidence() {
    let body = compiled(0);
    let rear = contact(&body, "right", "ankle-roll", 2_000_000);
    let toe = contact(&body, "right", "mtp", 4_000_001);
    let mut reversed = toe.clone();
    std::mem::swap(&mut reversed.actor_a_token, &mut reversed.actor_b_token);
    std::mem::swap(&mut reversed.shape_a_token, &mut reversed.shape_b_token);
    reversed.impulse_micronewton_seconds = reversed.impulse_micronewton_seconds.map(|v| -v);
    reversed.normal_q1_30 = reversed.normal_q1_30.map(|v| -v);
    let mut half = rear.clone();
    half.impulse_micronewton_seconds[1] /= 2;
    let mut a = BiomechanicsContactClassifierV2::new(&body).unwrap();
    let mut b = a.clone();
    assert_eq!(
        classify(&mut a, vec![rear, toe]),
        classify(&mut b, vec![reversed, half.clone(), half])
    );
    assert_eq!(a, b);
}

#[test]
fn support_transfer_preserves_anatomical_continuity_and_invalid_input_is_atomic() {
    let body = compiled(0);
    let mut c = BiomechanicsContactClassifierV2::new(&body).unwrap();
    let initial = c.clone();
    classify(&mut c, vec![contact(&body, "left", "ankle-roll", 100_000)]);
    classify(&mut c, vec![contact(&body, "left", "mtp", 100_000)]);
    assert_eq!(c.foot_active_substeps(), [2, 0]);
    let before = c.clone();
    let mut bad = contact(&body, "left", "mtp", 100_000);
    bad.shape_b_token = u64::MAX;
    assert!(
        c.classify_substep(
            &snapshot(vec![contact(&body, "left", "ankle-roll", 100_000), bad]),
            BiomechanicsSkillContactProfileV1::Locomotion
        )
        .is_err()
    );
    assert_eq!(c, before);
    let mut huge = contact(&body, "left", "mtp", i64::MAX);
    huge.impulse_micronewton_seconds = [i64::MAX; 3];
    assert_eq!(
        c.classify_substep(
            &snapshot(vec![huge.clone(), huge]),
            BiomechanicsSkillContactProfileV1::Locomotion
        ),
        Err(ContactClassificationError::NumericOverflow)
    );
    assert_eq!(c, before);
    classify(&mut c, vec![]);
    assert_eq!(c.foot_active_substeps(), [0, 0]);
    c.reset();
    assert_eq!(c, initial);
}

#[test]
fn terminal_consumes_aggregate_violation_and_rejects_legacy_contact_profile() {
    let body = compiled(0);
    let skill = BiomechanicsSkillContactProfileV1::Locomotion;
    let mut c = BiomechanicsContactClassifierV2::new(&body).unwrap();
    let frame = classify(
        &mut c,
        vec![
            contact(&body, "right", "ankle-roll", 3_000_000),
            contact(&body, "right", "mtp", 3_000_001),
        ],
    );
    let mut terminal = BiomechanicsTerminalEvaluator::new_articulated(&body, skill, 1800).unwrap();
    let decision = terminal
        .evaluate_motor_tick(1, &snapshot(vec![]), &vec![frame.clone(); 4], false, None)
        .unwrap();
    assert_eq!(
        decision.reason,
        Some(crate::BiomechanicsTerminalReasonV1::ContactImpact)
    );
    terminal.reset();
    let old = BiomechanicsContactClassifier::new(&body.base.base)
        .unwrap()
        .classify_substep(&snapshot(vec![]), skill)
        .unwrap();
    let before = terminal.clone();
    assert_eq!(
        terminal.evaluate_motor_tick(1, &snapshot(vec![]), &vec![old; 4], false, None),
        Err(crate::BiomechanicsTerminalError::ContactProfileMismatch)
    );
    assert_eq!(terminal, before);
    assert_eq!(
        BiomechanicsTerminalEvaluator::new(&body.base.base, skill, 1800)
            .unwrap()
            .evaluate_motor_tick(1, &snapshot(vec![]), &vec![frame; 4], false, None),
        Err(crate::BiomechanicsTerminalError::ContactProfileMismatch)
    );
}

#[test]
fn articulated_consumers_bind_exact_body_subject_reset_and_neutral_mtp_targets() {
    let body = compiled(0);
    let reset = snapshot(vec![]);
    let controller = BiomechanicsProceduralStandingControllerV3::new(&body, &reset).unwrap();
    let mut shifted = reset.clone();
    shifted.links[0].position_micrometres[2] += 1;
    assert_ne!(
        controller.state_root(),
        BiomechanicsProceduralStandingControllerV3::new(&body, &shifted)
            .unwrap()
            .state_root()
    );
    let other = compiled(1);
    assert_ne!(
        controller.state_root(),
        BiomechanicsProceduralStandingControllerV3::new(&other, &reset)
            .unwrap()
            .state_root()
    );
    assert_ne!(
        BiomechanicsContactClassifierV2::new(&body)
            .unwrap()
            .continuity_root(),
        BiomechanicsContactClassifierV2::new(&other)
            .unwrap()
            .continuity_root()
    );
    let base =
        crate::BiomechanicsProceduralStandingControllerV1::new(&body.base.base, &reset).unwrap();
    for qx in [-50_000_000, 0, 50_000_000] {
        let mut pose = reset.clone();
        pose.links[0].rotation_q1_30[0] = qx;
        let mut expected = base.reference_targets(&pose).unwrap();
        for (target, a) in expected
            .iter_mut()
            .zip(&body.base.base.actuator_definitions)
        {
            if a.joint_id.as_str().ends_with("-hip-pitch") {
                *target += (-2 * (i128::from(qx) * 2_000_000 / (1_i128 << 30))) as i64;
            }
            if a.joint_id.as_str().ends_with("-mtp") {
                assert_eq!(*target, 0);
            }
        }
        assert_eq!(controller.reference_targets(&pose).unwrap(), expected);
    }
    let old = CompiledBodySchemaV4::compile(
        &crate::biomechanics_humanoid_body_schema_v7(),
        PersistentId::from_bytes([0; 16]),
    )
    .unwrap();
    let mut bad = body.clone();
    bad.base.base.actuator_definitions[0].damping_q16 += 1;
    for input in [old, bad] {
        assert!(BiomechanicsContactClassifierV2::new(&input).is_err());
        assert!(BiomechanicsProceduralStandingControllerV3::new(&input, &reset).is_err());
        assert!(
            BiomechanicsTerminalEvaluator::new_articulated(
                &input,
                BiomechanicsSkillContactProfileV1::Locomotion,
                1800
            )
            .is_err()
        );
    }
}
