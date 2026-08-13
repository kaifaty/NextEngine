use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, content_hash_from_bytes};
use next_contracts::motor::MotorContractError;
use next_contracts::motor::MotorTerminalDispositionV1;
use next_physics_physx::{
    CanonicalPhysXSnapshotV2, PhysXAdapterError, PhysXArticulationWorldV2,
    PhysXRawArticulationSnapshot,
};
use next_physics_physx_ffi::{JointState, LinkState};
use serde_json::{Value, json};

use crate::{
    BiomechanicsContactClassifier, BiomechanicsSafetyController, BiomechanicsSkillContactProfileV1,
    BiomechanicsTerminalEvaluator, CompiledBodySchemaV2, ContactClassificationError,
    JointControlStateV1, MotorCompileError, MotorSafetyError, NORMALIZED_RESIDUAL_ONE_Q1_30,
    REFERENCE_TRACKER_PROFILE_DOCUMENT_SHA256, REFERENCE_TRACKING_ENVIRONMENT_PROFILE_ID,
    TrainingEnvironmentError, biomechanics_humanoid_body_schema_v2,
    biomechanics_reference_environment_manifest_v3, biomechanics_reference_tracking_profile_v1,
};

const INPUT_SCHEMA_ID: &str = "nextengine.motor.reference-baseline-input.v1";
const CORPUS_MANIFEST_SHA256: &str =
    "6f76c1c7d60457d1b10833b6fb840afbb50cd502315f250fd6e44a40d1c0dcbc";
const TRACKING_POSITION_LIMIT_MICROMETRES: i64 = 750_000;
const TRACKING_ORIENTATION_MINIMUM_DOT_Q1_30: i64 = 929_887_697;
const TRACKING_LOSS_TICKS: u32 = 4;
const ACTION_CHANNELS: usize = 23;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BaselineKind {
    ZeroResidual,
    RandomResidual,
}

impl BaselineKind {
    fn parse(value: &str) -> Result<Self, ReferenceBaselineError> {
        match value {
            "zero_residual" => Ok(Self::ZeroResidual),
            "random_residual" => Ok(Self::RandomResidual),
            _ => Err(ReferenceBaselineError::InvalidInput),
        }
    }

    const fn stable_id(self) -> &'static str {
        match self {
            Self::ZeroResidual => "zero_residual",
            Self::RandomResidual => "random_residual",
        }
    }
}

#[derive(Debug)]
struct ReferenceBaselineInput {
    baseline: BaselineKind,
    input_provenance_root: ContentHash,
    clip_id: String,
    skill: String,
    split: String,
    start_frame: usize,
    root_position_um: Vec<[i64; 3]>,
    root_quaternion_q1_30: Vec<[i64; 4]>,
    root_linear_velocity_um_s: Vec<[i64; 3]>,
    root_yaw_velocity_urad_s: Vec<i64>,
    joint_position_urad: Vec<Vec<i64>>,
    joint_velocity_urad_s: Vec<Vec<i64>>,
}

impl ReferenceBaselineInput {
    fn parse(text: &str) -> Result<Self, ReferenceBaselineError> {
        let value: Value = serde_json::from_str(text).map_err(ReferenceBaselineError::Json)?;
        if u64_field(&value, "schema_version")? != 1
            || string_field(&value, "schema_id")? != INPUT_SCHEMA_ID
            || string_field(&value, "profile_id")? != REFERENCE_TRACKING_ENVIRONMENT_PROFILE_ID
            || string_field(&value, "profile_sha256")?
                != hex(&REFERENCE_TRACKER_PROFILE_DOCUMENT_SHA256)
            || string_field(&value, "corpus_manifest_sha256")? != CORPUS_MANIFEST_SHA256
        {
            return Err(ReferenceBaselineError::InvalidInput);
        }
        let baseline = BaselineKind::parse(string_field(&value, "baseline")?)?;
        let input_provenance_root =
            content_hash_from_bytes(parse_hash(string_field(&value, "input_provenance_root")?)?);
        if input_provenance_root == ContentHash::default() {
            return Err(ReferenceBaselineError::InvalidInput);
        }
        let clip_id = string_field(&value, "clip_id")?.to_owned();
        let skill = string_field(&value, "skill")?.to_owned();
        let split = string_field(&value, "split")?.to_owned();
        if !matches!(split.as_str(), "train" | "validation" | "heldout") {
            return Err(ReferenceBaselineError::InvalidInput);
        }
        let start_frame = usize::try_from(u64_field(&value, "start_frame")?)
            .map_err(|_| ReferenceBaselineError::InvalidInput)?;
        let root_position_um = vector_array::<3>(&value, "root_position_um")?;
        let root_quaternion_q1_30 = vector_array::<4>(&value, "root_quaternion_q1_30")?;
        let root_linear_velocity_um_s = vector_array::<3>(&value, "root_linear_velocity_um_s")?;
        let root_yaw_velocity_urad_s = scalar_array(&value, "root_yaw_velocity_urad_s")?;
        let joint_position_urad = matrix(&value, "joint_position_urad", ACTION_CHANNELS)?;
        let joint_velocity_urad_s = matrix(&value, "joint_velocity_urad_s", ACTION_CHANNELS)?;
        let frame_count = root_position_um.len();
        if !(2..=3_600).contains(&frame_count)
            || start_frame >= frame_count - 1
            || root_quaternion_q1_30.len() != frame_count
            || root_linear_velocity_um_s.len() != frame_count
            || root_yaw_velocity_urad_s.len() != frame_count
            || joint_position_urad.len() != frame_count
            || joint_velocity_urad_s.len() != frame_count
        {
            return Err(ReferenceBaselineError::InvalidInput);
        }
        Ok(Self {
            baseline,
            input_provenance_root,
            clip_id,
            skill,
            split,
            start_frame,
            root_position_um,
            root_quaternion_q1_30,
            root_linear_velocity_um_s,
            root_yaw_velocity_urad_s,
            joint_position_urad,
            joint_velocity_urad_s,
        })
    }
}

pub fn biomechanics_reference_baseline_json_v1(
    input_json: &str,
) -> Result<String, ReferenceBaselineError> {
    let input = ReferenceBaselineInput::parse(input_json)?;
    let compiled = CompiledBodySchemaV2::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([81; 16]),
    )?;
    let profile = biomechanics_reference_tracking_profile_v1(&compiled)?;
    let manifest =
        biomechanics_reference_environment_manifest_v3(&compiled, input.input_provenance_root)?;
    let initial_state = initial_state(&compiled, &input)?;
    let (mut world, mut snapshot) = PhysXArticulationWorldV2::create_with_initial_state(
        compiled.physx_scene_profile,
        &compiled.physx_catalog,
        &initial_state,
    )?;
    let mut safety = BiomechanicsSafetyController::new(&compiled)?;
    safety.reset_to_reference(&actuator_ordered(
        &compiled,
        &input.joint_position_urad[input.start_frame],
    )?)?;
    let envelopes = safety.default_skill_envelopes();
    let mut classifier = BiomechanicsContactClassifier::new(&compiled)?;
    let available_ticks = input.root_position_um.len() - input.start_frame - 1;
    let mut terminal = BiomechanicsTerminalEvaluator::new(
        &compiled,
        BiomechanicsSkillContactProfileV1::Locomotion,
        u64::try_from(available_ticks + 1).map_err(|_| ReferenceBaselineError::InvalidInput)?,
    )
    .map_err(ReferenceBaselineError::Terminal)?;
    let mut cursor = input.start_frame;
    let mut completed_ticks = 0_u64;
    let mut tracking_loss_ticks = 0_u32;
    let mut target_clamp_count = 0_u64;
    let mut effort_clamp_count = 0_u64;
    let mut forbidden_contact_count = 0_u64;
    let mut observed_contact_pairs = BTreeMap::<(u64, u64, u64, u64), (i64, i64)>::new();
    let mut joint_safety_error_code = None;
    let mut hard_rom_joint_id = None;
    let mut hard_rom_joint_ordinal = None;
    let mut hard_rom_joint_position = None;
    let mut hard_rom_joint_minimum = None;
    let mut hard_rom_joint_maximum = None;
    let mut maximum_observed_joint_velocity = 0_i64;
    let mut maximum_observed_joint_velocity_ratio_basis_points = 0_u64;
    let mut maximum_velocity_joint_id = None;
    let mut maximum_velocity_joint_ordinal = None;
    let mut maximum_root_error = 0_i64;
    let mut maximum_joint_mean_error = 0_i64;
    let mut minimum_orientation_dot = 1_i64 << 30;
    let mut terminal_reason = None;
    let mut terminal_disposition = MotorTerminalDispositionV1::Running;
    let mut samples = vec![sample(&compiled, &snapshot, cursor, &input)?];

    while cursor + 1 < input.root_position_um.len() {
        let residual = residuals(input.baseline, &input.clip_id, completed_ticks);
        let targets = safety.begin_motor_tick(
            &actuator_ordered(&compiled, &input.joint_position_urad[cursor])?,
            &residual,
            &envelopes,
        );
        let mut contact_frames = Vec::with_capacity(4);
        let mut joint_error = match targets {
            Ok(targets) => {
                target_clamp_count += targets
                    .iter()
                    .filter(|target| target.clamp_flags != 0)
                    .count() as u64;
                None
            }
            Err(error) => Some(error),
        };
        if joint_error.is_none() {
            for _ in 0..4 {
                let states = actuator_states(&compiled, &snapshot)?;
                let efforts = match safety.step_substep(&states) {
                    Ok(efforts) => efforts,
                    Err(error) => {
                        joint_error = Some(error);
                        break;
                    }
                };
                effort_clamp_count += efforts
                    .iter()
                    .filter(|effort| effort.clamp_flags != 0)
                    .count() as u64;
                let mut dof_efforts = vec![0; efforts.len()];
                for (effort, dof) in efforts.iter().zip(&compiled.actuator_dof_ordinals) {
                    dof_efforts[*dof as usize] = effort.effort_micronewton_metres;
                }
                snapshot = world.apply_efforts_and_step(&dof_efforts)?;
                for contact in &snapshot.contacts {
                    let key = (
                        contact.actor_a_token,
                        contact.shape_a_token,
                        contact.actor_b_token,
                        contact.shape_b_token,
                    );
                    let maximum_absolute_impulse = contact
                        .impulse_micronewton_seconds
                        .into_iter()
                        .map(i64::unsigned_abs)
                        .max()
                        .and_then(|value| i64::try_from(value).ok())
                        .unwrap_or(i64::MAX);
                    observed_contact_pairs
                        .entry(key)
                        .and_modify(|diagnostic| {
                            diagnostic.0 = diagnostic.0.min(contact.separation_micrometres);
                            diagnostic.1 = diagnostic.1.max(maximum_absolute_impulse);
                        })
                        .or_insert((contact.separation_micrometres, maximum_absolute_impulse));
                }
                let velocity_diagnostic = joint_velocity_diagnostic(&compiled, &snapshot)?;
                if velocity_diagnostic.ratio_basis_points
                    > maximum_observed_joint_velocity_ratio_basis_points
                {
                    maximum_observed_joint_velocity =
                        velocity_diagnostic.velocity_microradians_per_second;
                    maximum_observed_joint_velocity_ratio_basis_points =
                        velocity_diagnostic.ratio_basis_points;
                    maximum_velocity_joint_id = Some(velocity_diagnostic.joint_id);
                    maximum_velocity_joint_ordinal = Some(velocity_diagnostic.ordinal);
                }
                if let Err(error) =
                    safety.validate_observed_joint_states(&actuator_states(&compiled, &snapshot)?)
                {
                    joint_safety_error_code = Some(error.stable_code().to_owned());
                    if error == MotorSafetyError::HardRangeViolation
                        && let Some(diagnostic) = hard_rom_diagnostic(&compiled, &snapshot)?
                    {
                        hard_rom_joint_id = Some(diagnostic.joint_id);
                        hard_rom_joint_ordinal = Some(diagnostic.ordinal);
                        hard_rom_joint_position = Some(diagnostic.position_microradians);
                        hard_rom_joint_minimum = Some(diagnostic.minimum_microradians);
                        hard_rom_joint_maximum = Some(diagnostic.maximum_microradians);
                    }
                    joint_error = Some(error);
                }
                let contacts = classifier
                    .classify_substep(&snapshot, BiomechanicsSkillContactProfileV1::Locomotion)?;
                forbidden_contact_count += contacts
                    .contacts
                    .iter()
                    .filter(|contact| {
                        contact.class == crate::BiomechanicsContactClassV1::ForbiddenLocomotion
                    })
                    .count() as u64;
                contact_frames.push(contacts);
                if joint_error.is_some() {
                    break;
                }
            }
        }
        while contact_frames.len() < 4 {
            let contacts = classifier
                .classify_substep(&snapshot, BiomechanicsSkillContactProfileV1::Locomotion)?;
            forbidden_contact_count += contacts
                .contacts
                .iter()
                .filter(|contact| {
                    contact.class == crate::BiomechanicsContactClassV1::ForbiddenLocomotion
                })
                .count() as u64;
            contact_frames.push(contacts);
        }
        cursor += 1;
        completed_ticks += 1;
        let errors = tracking_errors(&compiled, &snapshot, cursor, &input)?;
        maximum_root_error = maximum_root_error.max(errors.root_position_micrometres);
        maximum_joint_mean_error =
            maximum_joint_mean_error.max(errors.joint_mean_absolute_microradians);
        minimum_orientation_dot = minimum_orientation_dot.min(errors.orientation_dot_q1_30);
        if errors.root_position_micrometres > TRACKING_POSITION_LIMIT_MICROMETRES
            || errors.orientation_dot_q1_30 < TRACKING_ORIENTATION_MINIMUM_DOT_Q1_30
        {
            tracking_loss_ticks += 1;
        } else {
            tracking_loss_ticks = 0;
        }
        let decision = terminal
            .evaluate_motor_tick(
                completed_ticks,
                &snapshot,
                &contact_frames,
                false,
                joint_error,
            )
            .map_err(ReferenceBaselineError::Terminal)?;
        if decision.disposition != MotorTerminalDispositionV1::Running {
            terminal_disposition = decision.disposition;
            terminal_reason = decision.reason.map(|reason| reason.stable_id().to_owned());
        } else if tracking_loss_ticks >= TRACKING_LOSS_TICKS {
            terminal_disposition = MotorTerminalDispositionV1::Terminated;
            terminal_reason = Some("terminal.reference-tracking-lost".to_owned());
        } else if cursor + 1 == input.root_position_um.len() {
            terminal_disposition = MotorTerminalDispositionV1::Terminated;
            terminal_reason = Some("terminal.reference-complete".to_owned());
        }
        if completed_ticks.is_multiple_of(15)
            || terminal_disposition != MotorTerminalDispositionV1::Running
        {
            samples.push(sample(&compiled, &snapshot, cursor, &input)?);
        }
        if terminal_disposition != MotorTerminalDispositionV1::Running {
            break;
        }
    }
    let reference_completed = terminal_reason.as_deref() == Some("terminal.reference-complete");
    let output = json!({
        "schema_version": 1,
        "check": "MOTOR-REFERENCE-ENV-P1-PHYSX-BASELINE",
        "execution_status": "PASS",
        "claim": "CanonicalPhysXBaselineOnly",
        "baseline": input.baseline.stable_id(),
        "clip_id": input.clip_id,
        "skill": input.skill,
        "split": input.split,
        "start_frame": input.start_frame,
        "final_reference_frame": cursor,
        "completed_motor_ticks": completed_ticks,
        "profile_document_sha256": hex(&REFERENCE_TRACKER_PROFILE_DOCUMENT_SHA256),
        "profile_contract_hash": profile.profile_hash()?.to_hex(),
        "environment_manifest_hash": manifest.manifest_hash()?.to_hex(),
        "input_provenance_root": input.input_provenance_root.to_hex(),
        "terminal_disposition": disposition_id(terminal_disposition),
        "terminal_reason": terminal_reason,
        "reference_completed": reference_completed,
        "maximum_root_position_error_micrometres": maximum_root_error,
        "minimum_root_orientation_absolute_dot_q1_30": minimum_orientation_dot,
        "maximum_joint_mean_absolute_error_microradians": maximum_joint_mean_error,
        "target_clamp_count": target_clamp_count,
        "effort_clamp_count": effort_clamp_count,
        "forbidden_contact_raw_count": forbidden_contact_count,
        "observed_contact_pairs": observed_contact_pairs.into_iter().map(|(pair, diagnostic)| json!({
            "actor_a_token": pair.0,
            "shape_a_token": pair.1,
            "actor_b_token": pair.2,
            "shape_b_token": pair.3,
            "minimum_separation_micrometres": diagnostic.0,
            "maximum_absolute_impulse_micronewton_seconds": diagnostic.1,
        })).collect::<Vec<_>>(),
        "joint_safety_error_code": joint_safety_error_code,
        "hard_rom_joint_id": hard_rom_joint_id,
        "hard_rom_joint_ordinal": hard_rom_joint_ordinal,
        "hard_rom_joint_position_microradians": hard_rom_joint_position,
        "hard_rom_joint_minimum_microradians": hard_rom_joint_minimum,
        "hard_rom_joint_maximum_microradians": hard_rom_joint_maximum,
        "maximum_observed_joint_velocity_microradians_per_second": maximum_observed_joint_velocity,
        "maximum_observed_joint_velocity_ratio_basis_points": maximum_observed_joint_velocity_ratio_basis_points,
        "maximum_velocity_joint_id": maximum_velocity_joint_id,
        "maximum_velocity_joint_ordinal": maximum_velocity_joint_ordinal,
        "samples": samples,
        "optimizer_steps": 0,
        "learned_policy_claim": false,
    });
    let mut text =
        serde_json::to_string_pretty(&output).expect("serde_json::Value serialization cannot fail");
    text.push('\n');
    Ok(text)
}

#[derive(Clone, Copy, Debug)]
struct TrackingErrors {
    root_position_micrometres: i64,
    orientation_dot_q1_30: i64,
    joint_mean_absolute_microradians: i64,
}

fn tracking_errors(
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
    frame: usize,
    input: &ReferenceBaselineInput,
) -> Result<TrackingErrors, ReferenceBaselineError> {
    let root_token = compiled.body_tokens[&compiled.construction_order[0]];
    let root = snapshot
        .links
        .iter()
        .find(|link| link.user_token == root_token)
        .ok_or(ReferenceBaselineError::InvalidOutput)?;
    let delta = root
        .position_micrometres
        .into_iter()
        .zip(input.root_position_um[frame])
        .map(|(current, reference)| i128::from(current) - i128::from(reference))
        .collect::<Vec<_>>();
    let squared = delta
        .iter()
        .try_fold(0_i128, |sum, value| sum.checked_add(value * value))
        .ok_or(ReferenceBaselineError::InvalidOutput)?;
    let root_error = (squared as f64).sqrt().round_ties_even() as i64;
    let dot_q2_60 = root
        .rotation_q1_30
        .into_iter()
        .zip(input.root_quaternion_q1_30[frame])
        .try_fold(0_i128, |sum, (current, reference)| {
            sum.checked_add(i128::from(current) * i128::from(reference))
        })
        .ok_or(ReferenceBaselineError::InvalidOutput)?
        .unsigned_abs();
    let orientation_dot = i64::try_from((dot_q2_60 >> 30).min(1_u128 << 30))
        .map_err(|_| ReferenceBaselineError::InvalidOutput)?;
    let joint_sum = snapshot
        .joints
        .iter()
        .zip(&input.joint_position_urad[frame])
        .map(|(joint, reference)| {
            (i128::from(joint.position_microradians) - i128::from(*reference)).unsigned_abs()
        })
        .sum::<u128>();
    let joint_mean = i64::try_from(joint_sum / ACTION_CHANNELS as u128)
        .map_err(|_| ReferenceBaselineError::InvalidOutput)?;
    Ok(TrackingErrors {
        root_position_micrometres: root_error,
        orientation_dot_q1_30: orientation_dot,
        joint_mean_absolute_microradians: joint_mean,
    })
}

fn sample(
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
    frame: usize,
    input: &ReferenceBaselineInput,
) -> Result<Value, ReferenceBaselineError> {
    let root_token = compiled.body_tokens[&compiled.construction_order[0]];
    let root = snapshot
        .links
        .iter()
        .find(|link| link.user_token == root_token)
        .ok_or(ReferenceBaselineError::InvalidOutput)?;
    let errors = tracking_errors(compiled, snapshot, frame, input)?;
    Ok(json!({
        "reference_frame": frame,
        "root_position_micrometres": root.position_micrometres,
        "root_rotation_q1_30": root.rotation_q1_30,
        "root_position_error_micrometres": errors.root_position_micrometres,
        "root_orientation_absolute_dot_q1_30": errors.orientation_dot_q1_30,
        "joint_mean_absolute_error_microradians": errors.joint_mean_absolute_microradians,
        "contact_count": snapshot.contacts.len(),
    }))
}

fn initial_state(
    compiled: &CompiledBodySchemaV2,
    input: &ReferenceBaselineInput,
) -> Result<PhysXRawArticulationSnapshot, ReferenceBaselineError> {
    let frame = input.start_frame;
    let root_token = compiled.body_tokens[&compiled.construction_order[0]];
    let root = LinkState {
        user_token: root_token,
        position_bits: scaled_vector_bits(input.root_position_um[frame], 1_000_000.0)?,
        rotation_bits: scaled_quaternion_bits(input.root_quaternion_q1_30[frame])?,
        linear_velocity_bits: scaled_vector_bits(
            input.root_linear_velocity_um_s[frame],
            1_000_000.0,
        )?,
        angular_velocity_bits: scaled_vector_bits(
            [0, input.root_yaw_velocity_urad_s[frame], 0],
            1_000_000.0,
        )?,
    };
    let mut joints = vec![JointState::default(); compiled.physx_catalog.joints.len()];
    for dof in &compiled.actuator_dof_ordinals {
        joints[*dof as usize] = JointState {
            position_bits: scaled_bits(
                input.joint_position_urad[frame][*dof as usize],
                1_000_000.0,
            )?,
            velocity_bits: scaled_bits(
                input.joint_velocity_urad_s[frame][*dof as usize],
                1_000_000.0,
            )?,
        };
    }
    Ok(PhysXRawArticulationSnapshot {
        links: vec![root],
        joints,
    })
}

fn actuator_states(
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
) -> Result<Vec<JointControlStateV1>, ReferenceBaselineError> {
    compiled
        .actuator_dof_ordinals
        .iter()
        .map(|ordinal| {
            let joint = snapshot
                .joints
                .get(*ordinal as usize)
                .filter(|joint| joint.ordinal == *ordinal)
                .ok_or(ReferenceBaselineError::InvalidOutput)?;
            Ok(JointControlStateV1 {
                position_microradians: joint.position_microradians,
                velocity_microradians_per_second: joint.velocity_microradians_per_second,
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct JointVelocityDiagnostic {
    joint_id: String,
    ordinal: u32,
    velocity_microradians_per_second: i64,
    ratio_basis_points: u64,
}

#[derive(Clone, Debug)]
struct HardRomDiagnostic {
    joint_id: String,
    ordinal: u32,
    position_microradians: i64,
    minimum_microradians: i64,
    maximum_microradians: i64,
}

fn hard_rom_diagnostic(
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
) -> Result<Option<HardRomDiagnostic>, ReferenceBaselineError> {
    for state in &snapshot.joints {
        let descriptor = compiled
            .physics_descriptors
            .joints
            .iter()
            .find(|descriptor| {
                compiled.joint_dof_ordinals[&descriptor.base.joint_id] == state.ordinal
            })
            .ok_or(ReferenceBaselineError::InvalidOutput)?;
        if state.position_microradians < descriptor.base.limit_min_microradians
            || state.position_microradians > descriptor.base.limit_max_microradians
        {
            return Ok(Some(HardRomDiagnostic {
                joint_id: descriptor.base.joint_id.as_str().to_owned(),
                ordinal: state.ordinal,
                position_microradians: state.position_microradians,
                minimum_microradians: descriptor.base.limit_min_microradians,
                maximum_microradians: descriptor.base.limit_max_microradians,
            }));
        }
    }
    Ok(None)
}

fn joint_velocity_diagnostic(
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
) -> Result<JointVelocityDiagnostic, ReferenceBaselineError> {
    snapshot
        .joints
        .iter()
        .map(|state| {
            let descriptor = compiled
                .physics_descriptors
                .joints
                .iter()
                .find(|descriptor| {
                    compiled.joint_dof_ordinals[&descriptor.base.joint_id] == state.ordinal
                })
                .ok_or(ReferenceBaselineError::InvalidOutput)?;
            let ratio_basis_points = state
                .velocity_microradians_per_second
                .unsigned_abs()
                .saturating_mul(10_000)
                / descriptor.base.maximum_velocity_microradians_per_second;
            Ok::<_, ReferenceBaselineError>(JointVelocityDiagnostic {
                joint_id: descriptor.base.joint_id.as_str().to_owned(),
                ordinal: state.ordinal,
                velocity_microradians_per_second: state.velocity_microradians_per_second,
                ratio_basis_points,
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max_by_key(|value| value.ratio_basis_points)
        .ok_or(ReferenceBaselineError::InvalidOutput)
}

fn actuator_ordered(
    compiled: &CompiledBodySchemaV2,
    dof_ordered: &[i64],
) -> Result<Vec<i64>, ReferenceBaselineError> {
    if dof_ordered.len() != compiled.physx_catalog.joints.len() {
        return Err(ReferenceBaselineError::InvalidInput);
    }
    compiled
        .actuator_dof_ordinals
        .iter()
        .map(|dof| {
            dof_ordered
                .get(*dof as usize)
                .copied()
                .ok_or(ReferenceBaselineError::InvalidInput)
        })
        .collect()
}

fn residuals(kind: BaselineKind, clip_id: &str, motor_tick: u64) -> Vec<i64> {
    (0..ACTION_CHANNELS)
        .map(|channel| match kind {
            BaselineKind::ZeroResidual => 0,
            BaselineKind::RandomResidual => {
                let mut preimage = Vec::new();
                preimage.extend_from_slice(b"nextengine.reference-random-baseline.v1\0");
                preimage.extend_from_slice(clip_id.as_bytes());
                preimage.extend_from_slice(&motor_tick.to_le_bytes());
                preimage.extend_from_slice(&(channel as u32).to_le_bytes());
                let digest = sha256(&preimage);
                let sample = u64::from_le_bytes(digest[..8].try_into().unwrap());
                let width = 2_u64 * NORMALIZED_RESIDUAL_ONE_Q1_30 as u64 + 1;
                i64::try_from(sample % width).unwrap() - NORMALIZED_RESIDUAL_ONE_Q1_30
            }
        })
        .collect()
}

fn scaled_vector_bits<const N: usize>(
    values: [i64; N],
    scale: f64,
) -> Result<[u32; N], ReferenceBaselineError> {
    values
        .map(|value| scaled_bits(value, scale))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| ReferenceBaselineError::InvalidInput)
}

fn scaled_quaternion_bits(values: [i64; 4]) -> Result<[u32; 4], ReferenceBaselineError> {
    scaled_vector_bits(values, (1_u64 << 30) as f64)
}

fn scaled_bits(value: i64, scale: f64) -> Result<u32, ReferenceBaselineError> {
    let value = (value as f64 / scale) as f32;
    if !value.is_finite() {
        return Err(ReferenceBaselineError::InvalidInput);
    }
    Ok(value.to_bits())
}

fn disposition_id(value: MotorTerminalDispositionV1) -> &'static str {
    match value {
        MotorTerminalDispositionV1::Running => "running",
        MotorTerminalDispositionV1::Terminated => "terminated",
        MotorTerminalDispositionV1::Truncated => "truncated",
    }
}

fn string_field<'a>(value: &'a Value, name: &str) -> Result<&'a str, ReferenceBaselineError> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or(ReferenceBaselineError::InvalidInput)
}

fn u64_field(value: &Value, name: &str) -> Result<u64, ReferenceBaselineError> {
    value
        .get(name)
        .and_then(Value::as_u64)
        .ok_or(ReferenceBaselineError::InvalidInput)
}

fn scalar_array(value: &Value, name: &str) -> Result<Vec<i64>, ReferenceBaselineError> {
    value
        .get(name)
        .and_then(Value::as_array)
        .ok_or(ReferenceBaselineError::InvalidInput)?
        .iter()
        .map(|value| value.as_i64().ok_or(ReferenceBaselineError::InvalidInput))
        .collect()
}

fn vector_array<const N: usize>(
    value: &Value,
    name: &str,
) -> Result<Vec<[i64; N]>, ReferenceBaselineError> {
    value
        .get(name)
        .and_then(Value::as_array)
        .ok_or(ReferenceBaselineError::InvalidInput)?
        .iter()
        .map(|row| {
            let row = row.as_array().ok_or(ReferenceBaselineError::InvalidInput)?;
            if row.len() != N {
                return Err(ReferenceBaselineError::InvalidInput);
            }
            row.iter()
                .map(|value| value.as_i64().ok_or(ReferenceBaselineError::InvalidInput))
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| ReferenceBaselineError::InvalidInput)
        })
        .collect()
}

fn matrix(
    value: &Value,
    name: &str,
    width: usize,
) -> Result<Vec<Vec<i64>>, ReferenceBaselineError> {
    value
        .get(name)
        .and_then(Value::as_array)
        .ok_or(ReferenceBaselineError::InvalidInput)?
        .iter()
        .map(|row| {
            let row = row.as_array().ok_or(ReferenceBaselineError::InvalidInput)?;
            if row.len() != width {
                return Err(ReferenceBaselineError::InvalidInput);
            }
            row.iter()
                .map(|value| value.as_i64().ok_or(ReferenceBaselineError::InvalidInput))
                .collect()
        })
        .collect()
}

fn parse_hash(value: &str) -> Result<[u8; 32], ReferenceBaselineError> {
    if value.len() != 64 || value != value.to_ascii_lowercase() {
        return Err(ReferenceBaselineError::InvalidInput);
    }
    let mut output = [0_u8; 32];
    for (index, slot) in output.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| ReferenceBaselineError::InvalidInput)?;
    }
    Ok(output)
}

fn hex(value: &[u8]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug)]
pub enum ReferenceBaselineError {
    Json(serde_json::Error),
    InvalidInput,
    InvalidOutput,
    Compile(MotorCompileError),
    Training(TrainingEnvironmentError),
    Physics(PhysXAdapterError),
    Safety(MotorSafetyError),
    Contact(ContactClassificationError),
    Terminal(crate::BiomechanicsTerminalError),
}

impl Display for ReferenceBaselineError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Json(_) | Self::InvalidInput => "MOTOR_REFERENCE_BASELINE_INPUT_INVALID",
            Self::InvalidOutput => "MOTOR_REFERENCE_BASELINE_OUTPUT_INVALID",
            Self::Compile(_) => "MOTOR_REFERENCE_BASELINE_COMPILE_FAILED",
            Self::Training(error) => return Display::fmt(error, formatter),
            Self::Physics(error) => return Display::fmt(error, formatter),
            Self::Safety(error) => return Display::fmt(error, formatter),
            Self::Contact(error) => return Display::fmt(error, formatter),
            Self::Terminal(error) => return Display::fmt(error, formatter),
        })
    }
}

impl Error for ReferenceBaselineError {}

impl From<MotorCompileError> for ReferenceBaselineError {
    fn from(value: MotorCompileError) -> Self {
        Self::Compile(value)
    }
}

impl From<TrainingEnvironmentError> for ReferenceBaselineError {
    fn from(value: TrainingEnvironmentError) -> Self {
        Self::Training(value)
    }
}

impl From<MotorContractError> for ReferenceBaselineError {
    fn from(value: MotorContractError) -> Self {
        Self::Training(TrainingEnvironmentError::Contract(value))
    }
}

impl From<PhysXAdapterError> for ReferenceBaselineError {
    fn from(value: PhysXAdapterError) -> Self {
        Self::Physics(value)
    }
}

impl From<MotorSafetyError> for ReferenceBaselineError {
    fn from(value: MotorSafetyError) -> Self {
        Self::Safety(value)
    }
}

impl From<ContactClassificationError> for ReferenceBaselineError {
    fn from(value: ContactClassificationError) -> Self {
        Self::Contact(value)
    }
}
