use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::ids::PersistentId;
use next_contracts::motor::MotorTerminalDispositionV1;
use next_physics_physx::{CanonicalPhysXSnapshotV2, PhysXAdapterError, PhysXArticulationWorldV2};
use serde_json::{Value, json};

use crate::{
    BiomechanicsContactClassifier, BiomechanicsProceduralStandingControllerV1,
    BiomechanicsSafetyController, BiomechanicsSkillContactProfileV1, BiomechanicsTerminalEvaluator,
    CompiledBodySchemaV2, ContactClassificationError, HUMANOID_SAFETY_CONTACT_PROFILE_SHA256,
    JointControlStateV1, MotorCompileError, MotorSafetyError,
    PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS, ProceduralStandingError,
    biomechanics_humanoid_body_schema_v2,
};

pub fn biomechanics_native_safety_review_json_v1() -> Result<String, SafetyReviewError> {
    let compiled = CompiledBodySchemaV2::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([61; 16]),
    )?;
    let mut world =
        PhysXArticulationWorldV2::create(compiled.physx_scene_profile, &compiled.physx_catalog)?;
    let mut safety = BiomechanicsSafetyController::new(&compiled)?;
    let mut classifier = BiomechanicsContactClassifier::new(&compiled)?;
    let mut terminal = BiomechanicsTerminalEvaluator::new(
        &compiled,
        BiomechanicsSkillContactProfileV1::Locomotion,
        PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS,
    )
    .map_err(SafetyReviewError::Terminal)?;
    let mut snapshot = world.capture()?;
    let standing = BiomechanicsProceduralStandingControllerV1::new(&compiled, &snapshot)?;
    let residual = vec![0; compiled.actuator_definitions.len()];
    let envelopes = safety.default_skill_envelopes();
    let mut samples = vec![sample_json(
        0,
        &compiled,
        &snapshot,
        &[],
        None,
        safety.checkpoint_root().to_hex(),
    )];
    let mut final_decision = None;
    for motor_tick in 1..=PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS {
        let reference = standing.reference_targets(&snapshot)?;
        safety.begin_motor_tick(&reference, &residual, &envelopes)?;
        let mut contact_frames = Vec::with_capacity(4);
        let mut joint_error = None;
        for _ in 0..4 {
            let states = actuator_states(&compiled, &snapshot)?;
            let efforts = safety.step_substep(&states)?;
            let mut dof_efforts = vec![0; efforts.len()];
            for (effort, dof) in efforts.iter().zip(&compiled.actuator_dof_ordinals) {
                dof_efforts[*dof as usize] = effort.effort_micronewton_metres;
            }
            snapshot = world.apply_efforts_and_step(&dof_efforts)?;
            if let Err(error) =
                safety.validate_observed_joint_states(&actuator_states(&compiled, &snapshot)?)
            {
                joint_error = Some(error);
            }
            contact_frames.push(
                classifier
                    .classify_substep(&snapshot, BiomechanicsSkillContactProfileV1::Locomotion)?,
            );
            if joint_error.is_some() {
                break;
            }
        }
        while contact_frames.len() < 4 {
            contact_frames.push(
                classifier
                    .classify_substep(&snapshot, BiomechanicsSkillContactProfileV1::Locomotion)?,
            );
        }
        let decision = terminal
            .evaluate_motor_tick(motor_tick, &snapshot, &contact_frames, false, joint_error)
            .map_err(SafetyReviewError::Terminal)?;
        if motor_tick.is_multiple_of(600)
            || decision.disposition != MotorTerminalDispositionV1::Running
        {
            samples.push(sample_json(
                motor_tick,
                &compiled,
                &snapshot,
                &contact_frames,
                Some(&decision),
                safety.checkpoint_root().to_hex(),
            ));
        }
        if decision.disposition != MotorTerminalDispositionV1::Running {
            final_decision = Some(decision);
            break;
        }
    }
    let decision = final_decision.ok_or(SafetyReviewError::MissingTerminal)?;
    if decision.disposition != MotorTerminalDispositionV1::Truncated
        || decision.reason != Some(crate::BiomechanicsTerminalReasonV1::Timeout)
        || samples
            .last()
            .and_then(|sample| sample["motor_tick"].as_u64())
            != Some(PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS)
    {
        return Err(SafetyReviewError::UnexpectedTerminal);
    }
    let output = json!({
        "schema_version": 1,
        "review_id": "nextengine.motor.humanoid-safety-native-review.v1",
        "status": "PASS",
        "safety_contact_profile_sha256": hex(&HUMANOID_SAFETY_CONTACT_PROFILE_SHA256),
        "body_schema_hash": compiled.body_schema_hash.to_hex(),
        "compiled_descriptor_hash": compiled.compiled_descriptor_hash.to_hex(),
        "physics_hz": 240,
        "motor_hz": 60,
        "scenario_motor_ticks": PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS,
        "procedural_standing_state_root": standing.state_root().to_hex(),
        "samples": samples,
        "final_terminal_state_root": terminal.terminal_state_root().to_hex(),
    });
    let mut text =
        serde_json::to_string_pretty(&output).expect("serde_json::Value serialization cannot fail");
    text.push('\n');
    Ok(text)
}

fn sample_json(
    motor_tick: u64,
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
    contact_frames: &[crate::BiomechanicsContactFrameV1],
    decision: Option<&crate::BiomechanicsTerminalDecisionV1>,
    safety_root: String,
) -> Value {
    let links = compiled
        .construction_order
        .iter()
        .map(|body_id| {
            let actor = compiled.body_tokens[body_id];
            let link = snapshot
                .links
                .iter()
                .find(|link| link.user_token == actor)
                .expect("native snapshot contains every compiled link");
            json!({
                "body_id": body_id.as_str(),
                "actor_token": actor,
                "position_micrometres": link.position_micrometres,
                "rotation_q1_30": link.rotation_q1_30,
            })
        })
        .collect::<Vec<_>>();
    let mut joints = snapshot.joints.clone();
    joints.sort_by_key(|joint| joint.ordinal);
    let classified_contacts = contact_frames
        .iter()
        .enumerate()
        .flat_map(|(substep, frame)| {
            frame.contacts.iter().map(move |contact| {
                json!({
                    "substep": substep,
                    "primary_role": contact.primary_role as u8,
                    "secondary_role": contact.secondary_role.map(|role| role as u8),
                    "class": contact.class as u8,
                    "material": contact.material,
                    "hard_impact_violation": contact.hard_impact_violation,
                    "consecutive_active_substeps": contact.consecutive_active_substeps,
                    "impulse_magnitude_squared": contact.impulse_magnitude_squared,
                })
            })
        })
        .collect::<Vec<_>>();
    json!({
        "motor_tick": motor_tick,
        "simulated_seconds": motor_tick / 60,
        "links": links,
        "joints": joints.iter().map(|joint| json!({
            "ordinal": joint.ordinal,
            "position_microradians": joint.position_microradians,
            "velocity_microradians_per_second": joint.velocity_microradians_per_second,
        })).collect::<Vec<_>>(),
        "classified_contacts": classified_contacts,
        "contact_frame_roots": contact_frames.iter().map(|frame| frame.classification_root.to_hex()).collect::<Vec<_>>(),
        "safety_checkpoint_root": safety_root,
        "terminal": decision.map(|decision| json!({
            "disposition": decision.disposition as u8,
            "reason": decision.reason.map(|reason| reason as u8),
            "reason_id": decision.reason.map(|reason| reason.stable_id()),
            "decision_root": decision.decision_root.to_hex(),
        })),
    })
}

fn actuator_states(
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
) -> Result<Vec<JointControlStateV1>, SafetyReviewError> {
    compiled
        .actuator_dof_ordinals
        .iter()
        .map(|dof| {
            let state = snapshot
                .joints
                .iter()
                .find(|state| state.ordinal == *dof)
                .ok_or(SafetyReviewError::JointState)?;
            Ok(JointControlStateV1 {
                position_microradians: state.position_microradians,
                velocity_microradians_per_second: state.velocity_microradians_per_second,
            })
        })
        .collect()
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
pub enum SafetyReviewError {
    Compile(MotorCompileError),
    Physics(PhysXAdapterError),
    Safety(MotorSafetyError),
    Contact(ContactClassificationError),
    Terminal(crate::BiomechanicsTerminalError),
    Standing(ProceduralStandingError),
    JointState,
    MissingTerminal,
    UnexpectedTerminal,
}

impl From<MotorCompileError> for SafetyReviewError {
    fn from(value: MotorCompileError) -> Self {
        Self::Compile(value)
    }
}

impl From<PhysXAdapterError> for SafetyReviewError {
    fn from(value: PhysXAdapterError) -> Self {
        Self::Physics(value)
    }
}

impl From<MotorSafetyError> for SafetyReviewError {
    fn from(value: MotorSafetyError) -> Self {
        Self::Safety(value)
    }
}

impl From<ContactClassificationError> for SafetyReviewError {
    fn from(value: ContactClassificationError) -> Self {
        Self::Contact(value)
    }
}

impl From<ProceduralStandingError> for SafetyReviewError {
    fn from(value: ProceduralStandingError) -> Self {
        Self::Standing(value)
    }
}

impl Display for SafetyReviewError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for SafetyReviewError {}
