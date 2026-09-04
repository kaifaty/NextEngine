use serde_json::Value;

use next_contracts::ids::PersistentId;
use next_contracts::motor::MotorTerminalDispositionV1;
use next_physics_physx::PhysXArticulationWorldV3;

use crate::{
    BiomechanicsContactClassV1, BiomechanicsContactClassifier,
    BiomechanicsProceduralStandingControllerV1, BiomechanicsSafetyController,
    BiomechanicsSkillContactProfileV1, BiomechanicsStandingRewardFactsV1,
    BiomechanicsTerminalEvaluator, BiomechanicsTerminalReasonV1, CompiledBodySchemaV3,
    JointControlStateV1, PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS,
    biomechanics_humanoid_body_schema_v2, biomechanics_native_safety_review_json_v1,
    biomechanics_standing_reward_q16_v1,
};

#[test]
fn native_procedural_standing_reaches_exact_thirty_second_timeout() {
    let text = biomechanics_native_safety_review_json_v1().expect("native safety review");
    let review: Value = serde_json::from_str(&text).expect("valid review JSON");
    assert_eq!(review["status"], "PASS");
    assert_eq!(
        review["samples"]
            .as_array()
            .expect("samples")
            .iter()
            .map(|sample| sample["motor_tick"].as_u64().expect("motor tick"))
            .collect::<Vec<_>>(),
        [0, 600, 1_200, PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS]
    );
    let final_sample = review["samples"]
        .as_array()
        .expect("samples")
        .last()
        .expect("final sample");
    assert_eq!(final_sample["terminal"]["disposition"], 2);
    assert_eq!(
        final_sample["terminal"]["reason"],
        BiomechanicsTerminalReasonV1::Timeout as u8
    );
    assert_eq!(final_sample["joints"].as_array().map(Vec::len), Some(23));
    for sample in review["samples"].as_array().expect("samples") {
        for contact in sample["classified_contacts"]
            .as_array()
            .expect("classified contacts")
        {
            assert_eq!(contact["primary_role"], 8);
            assert_eq!(
                contact["class"],
                BiomechanicsContactClassV1::SoleSupport as u8
            );
            assert_eq!(contact["hard_impact_violation"], false);
        }
    }
}

#[test]
fn native_current_biomechanics_reset_step_and_reward_preflight() {
    let compiled = CompiledBodySchemaV3::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([62; 16]),
    )
    .expect("compile current material biomechanics");
    let base = &compiled.base;
    let mut world =
        PhysXArticulationWorldV3::create(base.physx_scene_profile, &compiled.physx_catalog)
            .expect("create current-material PhysX world");
    let mut snapshot = world.capture().expect("reset snapshot");
    let mut safety = BiomechanicsSafetyController::new(base).expect("safety controller");
    let mut classifier = BiomechanicsContactClassifier::new(base).expect("contact classifier");
    let mut terminal = BiomechanicsTerminalEvaluator::new(
        base,
        BiomechanicsSkillContactProfileV1::Locomotion,
        3_600,
    )
    .expect("terminal evaluator");
    let standing =
        BiomechanicsProceduralStandingControllerV1::new(base, &snapshot).expect("standing");
    let residual = vec![0; base.actuator_definitions.len()];
    let envelopes = safety.default_skill_envelopes();
    let root_actor = base.body_tokens[&base.construction_order[0]];

    for motor_tick in 1..=32 {
        let reference = standing
            .reference_targets(&snapshot)
            .expect("standing reference");
        let previous_targets = safety.checkpoint().applied_targets_microradians;
        let applied = safety
            .begin_motor_tick(&reference, &residual, &envelopes)
            .expect("bounded applied targets");
        let applied_targets = applied
            .iter()
            .map(|target| target.target_microradians)
            .collect::<Vec<_>>();
        let mut absolute_effort_sum = 0_u128;
        let mut contact_frames = Vec::with_capacity(4);
        let mut joint_safety_error = None;
        for _ in 0..4 {
            let states = actuator_states(base, &snapshot);
            let efforts = safety.step_substep(&states).expect("safe efforts");
            absolute_effort_sum += efforts
                .iter()
                .map(|effort| effort.effort_micronewton_metres.unsigned_abs() as u128)
                .sum::<u128>();
            let mut dof_efforts = vec![0; efforts.len()];
            for (effort, dof) in efforts.iter().zip(&base.actuator_dof_ordinals) {
                dof_efforts[*dof as usize] = effort.effort_micronewton_metres;
            }
            snapshot = world
                .apply_efforts_and_step(&dof_efforts)
                .expect("native current-material step");
            joint_safety_error = safety
                .validate_observed_joint_states(&actuator_states(base, &snapshot))
                .err();
            contact_frames.push(
                classifier
                    .classify_substep(&snapshot, BiomechanicsSkillContactProfileV1::Locomotion)
                    .expect("classified contacts"),
            );
        }
        let decision = terminal
            .evaluate_motor_tick(
                motor_tick,
                &snapshot,
                &contact_frames,
                false,
                joint_safety_error,
            )
            .expect("terminal decision");
        assert_eq!(decision.disposition, MotorTerminalDispositionV1::Running);

        let root = snapshot
            .links
            .iter()
            .find(|link| link.user_token == root_actor)
            .expect("root link");
        let states = actuator_states(base, &snapshot);
        let positions = states
            .iter()
            .map(|state| state.position_microradians)
            .collect::<Vec<_>>();
        let contacting_soles = contact_frames
            .iter()
            .flat_map(|frame| &frame.contacts)
            .filter(|contact| contact.class == BiomechanicsContactClassV1::SoleSupport)
            .map(|contact| {
                if contact.pair.actor_a_token == crate::HUMANOID_GROUND_ACTOR_TOKEN {
                    contact.pair.actor_b_token
                } else {
                    contact.pair.actor_a_token
                }
            })
            .collect::<std::collections::BTreeSet<_>>();
        let slip_sum = contacting_soles
            .iter()
            .filter_map(|actor| snapshot.links.iter().find(|link| link.user_token == *actor))
            .map(|link| {
                link.linear_velocity_micrometres_per_second[0].unsigned_abs() as u128
                    + link.linear_velocity_micrometres_per_second[2].unsigned_abs() as u128
            })
            .sum();
        let facts = BiomechanicsStandingRewardFactsV1 {
            root_rotation_q1_30: root.rotation_q1_30,
            root_height_micrometres: root.position_micrometres[1],
            joint_positions_microradians: &positions,
            reference_targets_microradians: &reference,
            root_linear_velocity_micrometres_per_second: root
                .linear_velocity_micrometres_per_second,
            root_angular_velocity_microradians_per_second: root
                .angular_velocity_microradians_per_second,
            absolute_applied_effort_sum_micronewton_metres: absolute_effort_sum,
            applied_targets_microradians: &applied_targets,
            previous_applied_targets_microradians: &previous_targets,
            contacting_sole_slip_sum_micrometres_per_second: slip_sum,
            contacting_sole_count: u8::try_from(contacting_soles.len()).expect("at most two soles"),
            fell: false,
        };
        let (components, total) =
            biomechanics_standing_reward_q16_v1(&compiled, &facts).expect("bounded reward");
        assert!(
            components
                .into_iter()
                .all(|value| (0..=65_536).contains(&value))
        );
        assert!((-148_768..=114_688).contains(&total));
    }
}

fn actuator_states(
    compiled: &crate::CompiledBodySchemaV2,
    snapshot: &next_physics_physx::CanonicalPhysXSnapshotV2,
) -> Vec<JointControlStateV1> {
    compiled
        .actuator_dof_ordinals
        .iter()
        .map(|dof| {
            let state = snapshot
                .joints
                .iter()
                .find(|state| state.ordinal == *dof)
                .expect("compiled joint state");
            JointControlStateV1 {
                position_microradians: state.position_microradians,
                velocity_microradians_per_second: state.velocity_microradians_per_second,
            }
        })
        .collect()
}
