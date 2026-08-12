use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::body::BodyContactRoleV2;
use next_contracts::ids::PersistentId;
use next_physics_physx::{
    CanonicalPhysXContactV2, CanonicalPhysXLinkState, CanonicalPhysXSnapshotV2,
};
use serde_json::{Value, json};

use crate::{
    AppliedJointTargetV1, BiomechanicsContactClassifier, BiomechanicsContactFrameV1,
    BiomechanicsSafetyController, BiomechanicsSkillContactProfileV1,
    BiomechanicsTerminalDecisionV1, BiomechanicsTerminalEvaluator, CompiledBodySchemaV2,
    ContactClassificationError, HUMANOID_GROUND_ACTOR_TOKEN, HUMANOID_GROUND_SHAPE_TOKEN,
    HUMANOID_SAFETY_CONTACT_PROFILE_SHA256, JointControlStateV1, MotorCompileError,
    MotorSafetyError, NORMALIZED_RESIDUAL_ONE_Q1_30, biomechanics_humanoid_body_schema_v2,
};

pub fn biomechanics_safety_contact_mirror_golden_json_v1() -> Result<String, SafetyMirrorError> {
    let compiled = CompiledBodySchemaV2::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([0; 16]),
    )?;
    let value = json!({
        "schema_version": 1,
        "mirror_id": "nextengine.isaac.humanoid-safety-contact-mirror.v1",
        "safety_contact_profile_sha256": hex(&HUMANOID_SAFETY_CONTACT_PROFILE_SHA256),
        "body_schema_hash": compiled.body_schema_hash.to_hex(),
        "compiled_descriptor_hash": compiled.compiled_descriptor_hash.to_hex(),
        "physics_hz": 240,
        "motor_hz": 60,
        "action_width": compiled.actuator_definitions.len(),
        "safety_scenario": safety_scenario(&compiled)?,
        "contact_terminal_scenarios": contact_terminal_scenarios(&compiled)?,
    });
    let mut output =
        serde_json::to_string_pretty(&value).expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

fn safety_scenario(compiled: &CompiledBodySchemaV2) -> Result<Value, SafetyMirrorError> {
    let mut controller = BiomechanicsSafetyController::new(compiled)?;
    let references = compiled
        .physics_descriptors
        .actuators
        .iter()
        .map(|actuator| actuator.base.neutral_position_microradians)
        .collect::<Vec<_>>();
    let residuals = vec![NORMALIZED_RESIDUAL_ONE_Q1_30; controller.channel_count()];
    let envelopes = controller.default_skill_envelopes();
    let joint_by_id = compiled
        .physics_descriptors
        .joints
        .iter()
        .map(|joint| (joint.base.joint_id.clone(), joint))
        .collect::<std::collections::BTreeMap<_, _>>();
    let states = compiled
        .physics_descriptors
        .actuators
        .iter()
        .map(|actuator| {
            let joint = joint_by_id[&actuator.base.joint_id];
            Ok(JointControlStateV1 {
                position_microradians: joint.soft_limit_min_microradians,
                velocity_microradians_per_second: i64::try_from(
                    joint.base.maximum_velocity_microradians_per_second,
                )
                .map_err(|_| SafetyMirrorError::NumericOverflow)?,
            })
        })
        .collect::<Result<Vec<_>, SafetyMirrorError>>()?;
    let mut final_targets = Vec::new();
    let mut final_efforts = Vec::new();
    for _ in 0..24 {
        final_targets = controller.begin_motor_tick(&references, &residuals, &envelopes)?;
        final_efforts.clear();
        for _ in 0..4 {
            final_efforts.push(controller.step_substep(&states)?);
        }
    }
    let checkpoint = controller.checkpoint();
    Ok(json!({
        "motor_ticks": 24,
        "reference_targets_microradians": references,
        "normalized_residuals_q1_30": residuals,
        "skill_envelopes": envelopes.iter().map(|envelope| json!({
            "joint_id": envelope.joint_id.as_str(),
            "minimum_microradians": envelope.minimum_microradians,
            "maximum_microradians": envelope.maximum_microradians,
        })).collect::<Vec<_>>(),
        "joint_states": states.iter().map(|state| [
            state.position_microradians,
            state.velocity_microradians_per_second,
        ]).collect::<Vec<_>>(),
        "final_applied_targets": target_json(&final_targets),
        "final_tick_efforts": final_efforts.iter().map(|substep| substep.iter().map(|effort| json!({
            "actuator_id": effort.actuator_id.as_str(),
            "effort_micronewton_metres": effort.effort_micronewton_metres,
            "clamp_flags": effort.clamp_flags,
        })).collect::<Vec<_>>()).collect::<Vec<_>>(),
        "final_checkpoint": {
            "applied_targets_microradians": checkpoint.applied_targets_microradians,
            "previous_efforts_micronewton_metres": checkpoint.previous_efforts_micronewton_metres,
            "positive_work_microjoules": checkpoint.positive_work_microjoules,
            "completed_substeps": checkpoint.completed_substeps,
            "motor_tick_prepared": checkpoint.motor_tick_prepared,
        },
        "safety_checkpoint_root": controller.checkpoint_root().to_hex(),
    }))
}

fn target_json(targets: &[AppliedJointTargetV1]) -> Vec<Value> {
    targets
        .iter()
        .map(|target| {
            json!({
                "actuator_id": target.actuator_id.as_str(),
                "target_microradians": target.target_microradians,
                "clamp_flags": target.clamp_flags,
            })
        })
        .collect()
}

fn contact_terminal_scenarios(
    compiled: &CompiledBodySchemaV2,
) -> Result<Vec<Value>, SafetyMirrorError> {
    let hand = role_pair(compiled, BodyContactRoleV2::HandGround)?;
    let knee = role_pair(compiled, BodyContactRoleV2::KneeGround)?;
    let head = role_pair(compiled, BodyContactRoleV2::HeadGround)?;
    let torso = role_pair(compiled, BodyContactRoleV2::TorsoGround)?;
    Ok(vec![
        run_terminal_scenario(
            compiled,
            "locomotion-hand-grace",
            BiomechanicsSkillContactProfileV1::Locomotion,
            1_200,
            vec![
                vec![vec![ground_contact(hand, 100_000)]; 4],
                vec![
                    vec![ground_contact(hand, 100_000)],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ],
            ],
        )?,
        run_terminal_scenario(
            compiled,
            "getup-knee-support",
            BiomechanicsSkillContactProfileV1::GetUp,
            1_200,
            vec![vec![vec![ground_contact(knee, 300_000)]; 4]],
        )?,
        run_terminal_scenario(
            compiled,
            "locomotion-head-impact",
            BiomechanicsSkillContactProfileV1::Locomotion,
            1_200,
            vec![vec![vec![ground_contact(head, 1_000_001)]; 4]],
        )?,
        run_terminal_scenario(
            compiled,
            "locomotion-self-collision",
            BiomechanicsSkillContactProfileV1::Locomotion,
            1_200,
            vec![vec![vec![self_contact(hand, torso, 300_000)]; 4]],
        )?,
        run_terminal_scenario(
            compiled,
            "locomotion-timeout",
            BiomechanicsSkillContactProfileV1::Locomotion,
            1,
            vec![vec![Vec::new(); 4]],
        )?,
    ])
}

fn run_terminal_scenario(
    compiled: &CompiledBodySchemaV2,
    name: &str,
    profile: BiomechanicsSkillContactProfileV1,
    maximum_ticks: u64,
    ticks: Vec<Vec<Vec<CanonicalPhysXContactV2>>>,
) -> Result<Value, SafetyMirrorError> {
    let mut classifier = BiomechanicsContactClassifier::new(compiled)?;
    let mut evaluator = BiomechanicsTerminalEvaluator::new(compiled, profile, maximum_ticks)
        .map_err(SafetyMirrorError::Terminal)?;
    let snapshot = upright_snapshot(compiled);
    let mut tick_records = Vec::new();
    for (index, substeps) in ticks.into_iter().enumerate() {
        let mut frames = Vec::new();
        let mut input = Vec::new();
        for contacts in substeps {
            input.push(contacts.iter().map(contact_json).collect::<Vec<_>>());
            frames.push(classifier.classify_substep(
                &CanonicalPhysXSnapshotV2 {
                    links: Vec::new(),
                    joints: Vec::new(),
                    contacts,
                },
                profile,
            )?);
        }
        let motor_tick =
            u64::try_from(index + 1).map_err(|_| SafetyMirrorError::NumericOverflow)?;
        let decision = evaluator
            .evaluate_motor_tick(motor_tick, &snapshot, &frames, false, None)
            .map_err(SafetyMirrorError::Terminal)?;
        tick_records.push(json!({
            "motor_tick": motor_tick,
            "contact_substeps": input,
            "expected_frames": frames.iter().map(contact_frame_json).collect::<Vec<_>>(),
            "expected_decision": decision_json(&decision),
        }));
    }
    Ok(json!({
        "name": name,
        "skill_profile": profile as u8,
        "maximum_episode_motor_ticks": maximum_ticks,
        "root": {
            "actor_token": snapshot.links[0].user_token,
            "position_micrometres": snapshot.links[0].position_micrometres,
            "rotation_q1_30": snapshot.links[0].rotation_q1_30,
        },
        "ticks": tick_records,
    }))
}

fn contact_frame_json(frame: &BiomechanicsContactFrameV1) -> Value {
    json!({
        "classification_root": frame.classification_root.to_hex(),
        "continuity_root": frame.continuity_root.to_hex(),
        "contacts": frame.contacts.iter().map(|contact| json!({
            "pair": [contact.pair.actor_a_token, contact.pair.shape_a_token, contact.pair.actor_b_token, contact.pair.shape_b_token],
            "primary_role": contact.primary_role as u8,
            "secondary_role": contact.secondary_role.map(|role| role as u8),
            "impulse_micronewton_seconds": contact.impulse_micronewton_seconds,
            "impulse_magnitude_squared": contact.impulse_magnitude_squared,
            "minimum_separation_micrometres": contact.minimum_separation_micrometres,
            "consecutive_active_substeps": contact.consecutive_active_substeps,
            "material": contact.material,
            "hard_impact_violation": contact.hard_impact_violation,
            "class": contact.class as u8,
        })).collect::<Vec<_>>(),
    })
}

fn decision_json(decision: &BiomechanicsTerminalDecisionV1) -> Value {
    json!({
        "disposition": decision.disposition as u8,
        "reason": decision.reason.map(|reason| reason as u8),
        "reason_id": decision.reason.map(|reason| reason.stable_id()),
        "decision_root": decision.decision_root.to_hex(),
    })
}

fn contact_json(contact: &CanonicalPhysXContactV2) -> Value {
    json!({
        "actor_a_token": contact.actor_a_token,
        "shape_a_token": contact.shape_a_token,
        "actor_b_token": contact.actor_b_token,
        "shape_b_token": contact.shape_b_token,
        "impulse_micronewton_seconds": contact.impulse_micronewton_seconds,
        "separation_micrometres": contact.separation_micrometres,
    })
}

fn role_pair(
    compiled: &CompiledBodySchemaV2,
    role: BodyContactRoleV2,
) -> Result<(u64, u64), SafetyMirrorError> {
    let shape = compiled
        .collider_contact_roles
        .iter()
        .find_map(|(shape, candidate)| (*candidate == role).then_some(*shape))
        .ok_or(SafetyMirrorError::Profile)?;
    Ok((compiled.collider_body_tokens[&shape], shape))
}

fn ground_contact(pair: (u64, u64), impulse: i64) -> CanonicalPhysXContactV2 {
    CanonicalPhysXContactV2 {
        actor_a_token: HUMANOID_GROUND_ACTOR_TOKEN,
        actor_b_token: pair.0,
        shape_a_token: HUMANOID_GROUND_SHAPE_TOKEN,
        shape_b_token: pair.1,
        position_micrometres: [0; 3],
        normal_q1_30: [0, 1 << 30, 0],
        impulse_micronewton_seconds: [impulse, 0, 0],
        separation_micrometres: 0,
    }
}

fn self_contact(first: (u64, u64), second: (u64, u64), impulse: i64) -> CanonicalPhysXContactV2 {
    CanonicalPhysXContactV2 {
        actor_a_token: first.0,
        actor_b_token: second.0,
        shape_a_token: first.1,
        shape_b_token: second.1,
        position_micrometres: [0; 3],
        normal_q1_30: [1 << 30, 0, 0],
        impulse_micronewton_seconds: [impulse, 0, 0],
        separation_micrometres: -1,
    }
}

fn upright_snapshot(compiled: &CompiledBodySchemaV2) -> CanonicalPhysXSnapshotV2 {
    CanonicalPhysXSnapshotV2 {
        links: vec![CanonicalPhysXLinkState {
            user_token: compiled.body_tokens[&compiled.construction_order[0]],
            position_micrometres: [0, 1_095_000, 0],
            rotation_q1_30: [0, 0, 0, 1 << 30],
            linear_velocity_micrometres_per_second: [0; 3],
            angular_velocity_microradians_per_second: [0; 3],
        }],
        joints: Vec::new(),
        contacts: Vec::new(),
    }
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[derive(Debug)]
pub enum SafetyMirrorError {
    Compile(MotorCompileError),
    Safety(MotorSafetyError),
    Contact(ContactClassificationError),
    Terminal(crate::BiomechanicsTerminalError),
    NumericOverflow,
    Profile,
}

impl From<MotorCompileError> for SafetyMirrorError {
    fn from(value: MotorCompileError) -> Self {
        Self::Compile(value)
    }
}

impl From<MotorSafetyError> for SafetyMirrorError {
    fn from(value: MotorSafetyError) -> Self {
        Self::Safety(value)
    }
}

impl From<ContactClassificationError> for SafetyMirrorError {
    fn from(value: ContactClassificationError) -> Self {
        Self::Contact(value)
    }
}

impl Display for SafetyMirrorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for SafetyMirrorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_contact_mirror_is_complete_and_matches_tracked_golden() {
        let actual = biomechanics_safety_contact_mirror_golden_json_v1()
            .expect("generate safety/contact golden");
        let value: Value = serde_json::from_str(&actual).expect("valid JSON");
        assert_eq!(value["action_width"], 23);
        assert_eq!(
            value["contact_terminal_scenarios"].as_array().map(Vec::len),
            Some(5)
        );
        assert_eq!(
            actual,
            include_str!("../../../lab/tests/fixtures/biomechanics_safety_contact_mirror_v1.json")
        );
    }
}
