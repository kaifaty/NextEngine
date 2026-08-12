use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::ids::PersistentId;
use next_physics_physx::{
    PhysXAdapterError, PhysXArticulationWorldV2, PhysXRawArticulationSnapshot,
};
use next_physics_physx_ffi::{JointState, LinkState};
use serde_json::{Value, json};

use crate::{
    BIOMECHANICS_HUMANOID_DOF, CompiledBodySchemaV2, MotorCompileError,
    biomechanics_humanoid_body_schema_v2,
};

const INPUT_SCHEMA_ID: &str = "nextengine.motor.reference-pose-audit-input.v1";
const MAXIMUM_POSES: usize = 20_000;
const AUDIT_ROOT_HEIGHT_MICROMETRES: i64 = 2_000_000;

#[derive(Debug)]
struct AuditPose {
    clip_id: String,
    split: String,
    reference_frame: usize,
    root_quaternion_q1_30: [i64; 4],
    joint_position_urad: Vec<i64>,
}

pub fn biomechanics_reference_pose_audit_json_v1(
    input_json: &str,
) -> Result<String, ReferencePoseAuditError> {
    let value: Value = serde_json::from_str(input_json).map_err(ReferencePoseAuditError::Json)?;
    if u64_field(&value, "schema_version")? != 1
        || string_field(&value, "schema_id")? != INPUT_SCHEMA_ID
    {
        return Err(ReferencePoseAuditError::InvalidInput);
    }
    let corpus_manifest_sha256 = parse_hash(string_field(&value, "corpus_manifest_sha256")?)?;
    let poses = parse_poses(&value)?;
    let compiled = CompiledBodySchemaV2::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([82; 16]),
    )?;
    if string_field(&value, "body_schema_hash")? != compiled.body_schema_hash.to_hex()
        || string_field(&value, "compiled_descriptor_hash")?
            != compiled.compiled_descriptor_hash.to_hex()
    {
        return Err(ReferencePoseAuditError::InvalidInput);
    }

    let mut scene_profile = compiled.physx_scene_profile;
    scene_profile.gravity_bits = [0.0_f32.to_bits(); 3];
    let maximum_velocity_by_ordinal = maximum_velocity_by_ordinal(&compiled)?;
    let mut failed_pose_count = 0_u64;
    let mut maximum_penetration_micrometres = 0_i64;
    let mut maximum_absolute_impulse_micronewton_seconds = 0_i64;
    let mut maximum_joint_velocity_ratio_basis_points = 0_u64;
    let mut pose_reports = Vec::with_capacity(poses.len());
    for pose in poses {
        let initial = initial_state(&compiled, &pose)?;
        let (mut world, initial_snapshot) = PhysXArticulationWorldV2::create_with_initial_state(
            scene_profile,
            &compiled.physx_catalog,
            &initial,
        )?;
        if initial_snapshot
            .joints
            .iter()
            .zip(&pose.joint_position_urad)
            .any(|(state, reference)| state.position_microradians != *reference)
        {
            return Err(ReferencePoseAuditError::InvalidOutput);
        }
        let snapshot = world.apply_efforts_and_step(&[0; BIOMECHANICS_HUMANOID_DOF])?;
        let active_penetrations = snapshot
            .contacts
            .iter()
            .filter_map(|contact| {
                let maximum_impulse = contact
                    .impulse_micronewton_seconds
                    .into_iter()
                    .map(i64::unsigned_abs)
                    .max()
                    .and_then(|value| i64::try_from(value).ok())
                    .unwrap_or(i64::MAX);
                (contact.separation_micrometres < 0 && maximum_impulse > 0)
                    .then_some((contact, maximum_impulse))
            })
            .collect::<Vec<_>>();
        let pose_maximum_penetration = active_penetrations
            .iter()
            .map(|(contact, _)| contact.separation_micrometres.saturating_neg())
            .max()
            .unwrap_or(0);
        let pose_maximum_impulse = active_penetrations
            .iter()
            .map(|(_, impulse)| *impulse)
            .max()
            .unwrap_or(0);
        let pose_maximum_velocity_ratio = snapshot
            .joints
            .iter()
            .map(|state| {
                state
                    .velocity_microradians_per_second
                    .unsigned_abs()
                    .saturating_mul(10_000)
                    / maximum_velocity_by_ordinal[state.ordinal as usize]
            })
            .max()
            .unwrap_or(0);
        let passed = active_penetrations.is_empty() && pose_maximum_velocity_ratio <= 10_000;
        if !passed {
            failed_pose_count += 1;
        }
        maximum_penetration_micrometres =
            maximum_penetration_micrometres.max(pose_maximum_penetration);
        maximum_absolute_impulse_micronewton_seconds =
            maximum_absolute_impulse_micronewton_seconds.max(pose_maximum_impulse);
        maximum_joint_velocity_ratio_basis_points =
            maximum_joint_velocity_ratio_basis_points.max(pose_maximum_velocity_ratio);
        pose_reports.push(json!({
            "clip_id": pose.clip_id,
            "split": pose.split,
            "reference_frame": pose.reference_frame,
            "status": if passed { "PASS" } else { "FAIL" },
            "active_self_penetration_count": active_penetrations.len(),
            "maximum_penetration_micrometres": pose_maximum_penetration,
            "maximum_absolute_impulse_micronewton_seconds": pose_maximum_impulse,
            "maximum_joint_velocity_ratio_basis_points": pose_maximum_velocity_ratio,
            "active_contact_pairs": active_penetrations.into_iter().map(|(contact, impulse)| json!({
                "shape_a_token": contact.shape_a_token,
                "shape_b_token": contact.shape_b_token,
                "separation_micrometres": contact.separation_micrometres,
                "maximum_absolute_impulse_micronewton_seconds": impulse,
            })).collect::<Vec<_>>(),
        }));
    }
    let output = json!({
        "schema_version": 1,
        "check": "TRAIN-4-PHYSX-REFERENCE-POSE-AUDIT",
        "status": if failed_pose_count == 0 { "PASS" } else { "FAIL" },
        "claim": "ReferencePoseResetFeasibilityOnly",
        "body_schema_hash": compiled.body_schema_hash.to_hex(),
        "compiled_descriptor_hash": compiled.compiled_descriptor_hash.to_hex(),
        "corpus_manifest_sha256": hex(&corpus_manifest_sha256),
        "pose_count": pose_reports.len(),
        "failed_pose_count": failed_pose_count,
        "maximum_penetration_micrometres": maximum_penetration_micrometres,
        "maximum_absolute_impulse_micronewton_seconds": maximum_absolute_impulse_micronewton_seconds,
        "maximum_joint_velocity_ratio_basis_points": maximum_joint_velocity_ratio_basis_points,
        "poses": pose_reports,
        "physics_substeps_per_pose": 1,
        "gravity_enabled": false,
        "ground_reachable": false,
        "optimizer_steps": 0,
        "learned_policy_claim": false,
    });
    let mut text =
        serde_json::to_string_pretty(&output).expect("serde_json::Value serialization cannot fail");
    text.push('\n');
    Ok(text)
}

fn parse_poses(value: &Value) -> Result<Vec<AuditPose>, ReferencePoseAuditError> {
    let rows = value
        .get("poses")
        .and_then(Value::as_array)
        .filter(|rows| !rows.is_empty() && rows.len() <= MAXIMUM_POSES)
        .ok_or(ReferencePoseAuditError::InvalidInput)?;
    rows.iter()
        .map(|row| {
            let split = string_field(row, "split")?.to_owned();
            if !matches!(split.as_str(), "train" | "validation" | "heldout") {
                return Err(ReferencePoseAuditError::InvalidInput);
            }
            Ok(AuditPose {
                clip_id: string_field(row, "clip_id")?.to_owned(),
                split,
                reference_frame: usize::try_from(u64_field(row, "reference_frame")?)
                    .map_err(|_| ReferencePoseAuditError::InvalidInput)?,
                root_quaternion_q1_30: vector::<4>(row, "root_quaternion_q1_30")?,
                joint_position_urad: vector_dynamic(
                    row,
                    "joint_position_urad",
                    BIOMECHANICS_HUMANOID_DOF,
                )?,
            })
        })
        .collect()
}

fn initial_state(
    compiled: &CompiledBodySchemaV2,
    pose: &AuditPose,
) -> Result<PhysXRawArticulationSnapshot, ReferencePoseAuditError> {
    let root_token = compiled.body_tokens[&compiled.construction_order[0]];
    let root = LinkState {
        user_token: root_token,
        position_bits: [
            0.0_f32.to_bits(),
            (AUDIT_ROOT_HEIGHT_MICROMETRES as f32 / 1_000_000.0).to_bits(),
            0.0_f32.to_bits(),
        ],
        rotation_bits: pose
            .root_quaternion_q1_30
            .map(|value| (value as f32 / (1_u64 << 30) as f32).to_bits()),
        linear_velocity_bits: [0.0_f32.to_bits(); 3],
        angular_velocity_bits: [0.0_f32.to_bits(); 3],
    };
    let joints = pose
        .joint_position_urad
        .iter()
        .map(|position| JointState {
            position_bits: (*position as f32 / 1_000_000.0).to_bits(),
            velocity_bits: 0.0_f32.to_bits(),
        })
        .collect();
    Ok(PhysXRawArticulationSnapshot {
        links: vec![root],
        joints,
    })
}

fn maximum_velocity_by_ordinal(
    compiled: &CompiledBodySchemaV2,
) -> Result<Vec<u64>, ReferencePoseAuditError> {
    let mut values = vec![0_u64; BIOMECHANICS_HUMANOID_DOF];
    for descriptor in &compiled.physics_descriptors.joints {
        let ordinal = compiled.joint_dof_ordinals[&descriptor.base.joint_id] as usize;
        values[ordinal] = descriptor.base.maximum_velocity_microradians_per_second;
    }
    if values.contains(&0) {
        return Err(ReferencePoseAuditError::InvalidOutput);
    }
    Ok(values)
}

fn string_field<'a>(value: &'a Value, name: &str) -> Result<&'a str, ReferencePoseAuditError> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or(ReferencePoseAuditError::InvalidInput)
}

fn u64_field(value: &Value, name: &str) -> Result<u64, ReferencePoseAuditError> {
    value
        .get(name)
        .and_then(Value::as_u64)
        .ok_or(ReferencePoseAuditError::InvalidInput)
}

fn vector<const N: usize>(value: &Value, name: &str) -> Result<[i64; N], ReferencePoseAuditError> {
    vector_dynamic(value, name, N)?
        .try_into()
        .map_err(|_| ReferencePoseAuditError::InvalidInput)
}

fn vector_dynamic(
    value: &Value,
    name: &str,
    width: usize,
) -> Result<Vec<i64>, ReferencePoseAuditError> {
    let values = value
        .get(name)
        .and_then(Value::as_array)
        .filter(|values| values.len() == width)
        .ok_or(ReferencePoseAuditError::InvalidInput)?;
    values
        .iter()
        .map(|value| value.as_i64().ok_or(ReferencePoseAuditError::InvalidInput))
        .collect()
}

fn parse_hash(value: &str) -> Result<[u8; 32], ReferencePoseAuditError> {
    if value.len() != 64 || value != value.to_ascii_lowercase() {
        return Err(ReferencePoseAuditError::InvalidInput);
    }
    let mut output = [0_u8; 32];
    for (index, slot) in output.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| ReferencePoseAuditError::InvalidInput)?;
    }
    Ok(output)
}

fn hex(value: &[u8]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug)]
pub enum ReferencePoseAuditError {
    Json(serde_json::Error),
    InvalidInput,
    InvalidOutput,
    Compile(MotorCompileError),
    Physics(PhysXAdapterError),
}

impl Display for ReferencePoseAuditError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Json(_) | Self::InvalidInput => "MOTOR_REFERENCE_POSE_AUDIT_INPUT_INVALID",
            Self::InvalidOutput => "MOTOR_REFERENCE_POSE_AUDIT_OUTPUT_INVALID",
            Self::Compile(_) => "MOTOR_REFERENCE_POSE_AUDIT_COMPILE_FAILED",
            Self::Physics(error) => return Display::fmt(error, formatter),
        })
    }
}

impl Error for ReferencePoseAuditError {}

impl From<MotorCompileError> for ReferencePoseAuditError {
    fn from(value: MotorCompileError) -> Self {
        Self::Compile(value)
    }
}

impl From<PhysXAdapterError> for ReferencePoseAuditError {
    fn from(value: PhysXAdapterError) -> Self {
        Self::Physics(value)
    }
}
