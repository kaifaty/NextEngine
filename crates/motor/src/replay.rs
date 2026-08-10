use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};
use next_physics_physx::PhysXRawArticulationSnapshot;
use next_physics_physx_ffi::{JointState, LinkState};

use crate::{HumanoidMotorCheckpoint, MAX_REPLAY_MOTOR_TICKS, MotorFrameResult, MotorReplayFrame};

const CHECKPOINT_MAGIC: &[u8; 8] = b"NEMOTCP\0";
pub const MOTOR_RUNTIME_CHECKPOINT_SCHEMA_VERSION: u16 = 2;
const MAX_CHECKPOINT_BYTES: usize = 4 * 1_048_576;
const MAX_LINKS: usize = 128;
const MAX_JOINTS: usize = 256;
const MAX_CHANNELS: usize = 4_096;

impl HumanoidMotorCheckpoint {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, MotorReplayCodecError> {
        validate_checkpoint(self)?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(CHECKPOINT_MAGIC);
        bytes.extend_from_slice(&MOTOR_RUNTIME_CHECKPOINT_SCHEMA_VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.motor_tick.to_le_bytes());
        push_i64_values(&mut bytes, &self.applied_action_microradians)?;
        for value in self.command_raw {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        push_i64_values(&mut bytes, &self.previous_efforts_micronewton_metres)?;
        push_physics_snapshot(&mut bytes, &self.replay_origin_physics)?;
        push_len(&mut bytes, self.replay_frames.len())?;
        for frame in &self.replay_frames {
            bytes.extend_from_slice(&frame.motor_tick.to_le_bytes());
            push_i64_values(&mut bytes, &frame.post_safety_efforts_micronewton_metres)?;
            bytes.extend_from_slice(frame.physics_witness_hash.as_bytes());
        }
        push_physics_snapshot(&mut bytes, &self.physics)?;
        if bytes.len() + 32 > MAX_CHECKPOINT_BYTES {
            return Err(MotorReplayCodecError::CapacityExceeded);
        }
        let checksum = sha256(&bytes);
        bytes.extend_from_slice(&checksum);
        Ok(bytes)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, MotorReplayCodecError> {
        if bytes.len() < CHECKPOINT_MAGIC.len() + 2 {
            return Err(MotorReplayCodecError::Truncated);
        }
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(MotorReplayCodecError::CapacityExceeded);
        }
        if &bytes[..8] != CHECKPOINT_MAGIC {
            return Err(MotorReplayCodecError::MagicMismatch);
        }
        let version = u16::from_le_bytes([bytes[8], bytes[9]]);
        if version != MOTOR_RUNTIME_CHECKPOINT_SCHEMA_VERSION {
            return Err(MotorReplayCodecError::UnsupportedVersion(version));
        }
        if bytes.len() < 42 {
            return Err(MotorReplayCodecError::Truncated);
        }
        let (payload, expected_checksum) = bytes.split_at(bytes.len() - 32);
        if sha256(payload) != expected_checksum {
            return Err(MotorReplayCodecError::ChecksumMismatch);
        }
        let mut reader = Reader::new(&payload[10..]);
        let motor_tick = reader.read_u64()?;
        let applied_action_microradians = reader.read_i64_values(MAX_CHANNELS)?;
        let command_raw = [reader.read_i64()?, reader.read_i64()?, reader.read_i64()?];
        let previous_efforts_micronewton_metres = reader.read_i64_values(MAX_CHANNELS)?;
        let replay_origin_physics = reader.read_physics_snapshot()?;
        let replay_frame_count = reader.read_len(MAX_REPLAY_MOTOR_TICKS)?;
        let mut replay_frames = Vec::with_capacity(replay_frame_count);
        for _ in 0..replay_frame_count {
            replay_frames.push(MotorReplayFrame {
                motor_tick: reader.read_u64()?,
                post_safety_efforts_micronewton_metres: reader
                    .read_i64_values(MAX_CHANNELS * next_contracts::motor::STAGE0_SUBSTEPS)?,
                physics_witness_hash: ContentHash::from_bytes(reader.read_u8_array()?),
            });
        }
        let physics = reader.read_physics_snapshot()?;
        if !reader.is_empty() {
            return Err(MotorReplayCodecError::TrailingBytes);
        }
        let value = Self {
            motor_tick,
            applied_action_microradians,
            command_raw,
            previous_efforts_micronewton_metres,
            physics,
            replay_origin_physics,
            replay_frames,
        };
        validate_checkpoint(&value)?;
        if value.canonical_bytes()? != bytes {
            return Err(MotorReplayCodecError::NonCanonical);
        }
        Ok(value)
    }

    pub fn checkpoint_hash(&self) -> Result<ContentHash, MotorReplayCodecError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

impl MotorFrameResult {
    pub fn deterministic_hash(&self) -> Result<ContentHash, MotorReplayCodecError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"nextengine.motor-frame-result.v1\0");
        bytes.extend_from_slice(&self.motor_tick.to_le_bytes());
        push_i64_values(&mut bytes, &self.applied_action_microradians)?;
        push_i64_values(&mut bytes, &self.observation_raw)?;
        push_len(&mut bytes, self.substep_efforts.len())?;
        for substep in &self.substep_efforts {
            push_len(&mut bytes, substep.len())?;
            for effort in substep {
                push_text(&mut bytes, effort.actuator_id.as_str())?;
                bytes.extend_from_slice(&effort.effort_micronewton_metres.to_le_bytes());
                bytes.extend_from_slice(&effort.clamp_flags.to_le_bytes());
            }
        }
        push_len(&mut bytes, self.snapshot.links.len())?;
        for link in &self.snapshot.links {
            bytes.extend_from_slice(&link.user_token.to_le_bytes());
            push_i64_array(&mut bytes, &link.position_micrometres);
            push_i64_array(&mut bytes, &link.rotation_q1_30);
            push_i64_array(&mut bytes, &link.linear_velocity_micrometres_per_second);
            push_i64_array(&mut bytes, &link.angular_velocity_microradians_per_second);
        }
        push_len(&mut bytes, self.snapshot.joints.len())?;
        for joint in &self.snapshot.joints {
            bytes.extend_from_slice(&joint.ordinal.to_le_bytes());
            bytes.extend_from_slice(&joint.position_microradians.to_le_bytes());
            bytes.extend_from_slice(&joint.velocity_microradians_per_second.to_le_bytes());
        }
        push_len(&mut bytes, self.snapshot.contacts.len())?;
        for contact in &self.snapshot.contacts {
            bytes.extend_from_slice(&contact.actor_a_token.to_le_bytes());
            bytes.extend_from_slice(&contact.actor_b_token.to_le_bytes());
            push_i64_array(&mut bytes, &contact.position_micrometres);
            push_i64_array(&mut bytes, &contact.normal_q1_30);
            push_i64_array(&mut bytes, &contact.impulse_micronewton_seconds);
            bytes.extend_from_slice(&contact.separation_micrometres.to_le_bytes());
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

fn validate_checkpoint(value: &HumanoidMotorCheckpoint) -> Result<(), MotorReplayCodecError> {
    if value.applied_action_microradians.is_empty()
        || value.applied_action_microradians.len() > MAX_CHANNELS
        || value.previous_efforts_micronewton_metres.len()
            != value.applied_action_microradians.len()
        || value.replay_frames.len() > MAX_REPLAY_MOTOR_TICKS
        || value.motor_tick != value.replay_frames.len() as u64
    {
        return Err(MotorReplayCodecError::InvalidBounds);
    }
    validate_physics_snapshot(&value.physics, value.applied_action_microradians.len())?;
    validate_physics_snapshot(
        &value.replay_origin_physics,
        value.applied_action_microradians.len(),
    )?;
    if value.physics.links.len() != value.replay_origin_physics.links.len() {
        return Err(MotorReplayCodecError::InvalidBounds);
    }
    let effort_count = value
        .applied_action_microradians
        .len()
        .checked_mul(next_contracts::motor::STAGE0_SUBSTEPS)
        .ok_or(MotorReplayCodecError::CapacityExceeded)?;
    for (index, frame) in value.replay_frames.iter().enumerate() {
        if frame.motor_tick != index as u64 + 1
            || frame.post_safety_efforts_micronewton_metres.len() != effort_count
        {
            return Err(MotorReplayCodecError::InvalidBounds);
        }
    }
    if value.replay_frames.is_empty() && value.physics != value.replay_origin_physics {
        return Err(MotorReplayCodecError::NonCanonical);
    }
    if encoded_checkpoint_size(value)? > MAX_CHECKPOINT_BYTES {
        return Err(MotorReplayCodecError::CapacityExceeded);
    }
    Ok(())
}

fn validate_physics_snapshot(
    value: &PhysXRawArticulationSnapshot,
    expected_joint_count: usize,
) -> Result<(), MotorReplayCodecError> {
    if value.links.is_empty()
        || value.links.len() > MAX_LINKS
        || value.joints.is_empty()
        || value.joints.len() > MAX_JOINTS
        || value.joints.len() != expected_joint_count
        || value
            .links
            .windows(2)
            .any(|pair| pair[0].user_token >= pair[1].user_token)
        || value.links.iter().any(|link| {
            link.position_bits
                .iter()
                .chain(&link.rotation_bits)
                .chain(&link.linear_velocity_bits)
                .chain(&link.angular_velocity_bits)
                .any(|bits| !f32::from_bits(*bits).is_finite())
        })
        || value.joints.iter().any(|joint| {
            !f32::from_bits(joint.position_bits).is_finite()
                || !f32::from_bits(joint.velocity_bits).is_finite()
        })
    {
        return Err(MotorReplayCodecError::InvalidBounds);
    }
    Ok(())
}

fn encoded_checkpoint_size(
    value: &HumanoidMotorCheckpoint,
) -> Result<usize, MotorReplayCodecError> {
    let mut size = CHECKPOINT_MAGIC.len() + 2 + 8 + 24 + 32;
    size = checked_add_i64_vector_size(size, value.applied_action_microradians.len())?;
    size = checked_add_i64_vector_size(size, value.previous_efforts_micronewton_metres.len())?;
    size = checked_add_physics_snapshot_size(size, &value.replay_origin_physics)?;
    size = size
        .checked_add(4)
        .ok_or(MotorReplayCodecError::CapacityExceeded)?;
    for frame in &value.replay_frames {
        size = size
            .checked_add(8 + 32)
            .ok_or(MotorReplayCodecError::CapacityExceeded)?;
        size =
            checked_add_i64_vector_size(size, frame.post_safety_efforts_micronewton_metres.len())?;
    }
    checked_add_physics_snapshot_size(size, &value.physics)
}

fn checked_add_i64_vector_size(size: usize, length: usize) -> Result<usize, MotorReplayCodecError> {
    size.checked_add(4)
        .and_then(|size| {
            length
                .checked_mul(8)
                .and_then(|bytes| size.checked_add(bytes))
        })
        .ok_or(MotorReplayCodecError::CapacityExceeded)
}

fn checked_add_physics_snapshot_size(
    size: usize,
    value: &PhysXRawArticulationSnapshot,
) -> Result<usize, MotorReplayCodecError> {
    let link_bytes = value
        .links
        .len()
        .checked_mul(8 + (3 + 4 + 3 + 3) * 4)
        .ok_or(MotorReplayCodecError::CapacityExceeded)?;
    let joint_bytes = value
        .joints
        .len()
        .checked_mul(8)
        .ok_or(MotorReplayCodecError::CapacityExceeded)?;
    size.checked_add(4)
        .and_then(|size| size.checked_add(link_bytes))
        .and_then(|size| size.checked_add(4))
        .and_then(|size| size.checked_add(joint_bytes))
        .ok_or(MotorReplayCodecError::CapacityExceeded)
}

fn push_physics_snapshot(
    bytes: &mut Vec<u8>,
    physics: &PhysXRawArticulationSnapshot,
) -> Result<(), MotorReplayCodecError> {
    push_len(bytes, physics.links.len())?;
    for link in &physics.links {
        bytes.extend_from_slice(&link.user_token.to_le_bytes());
        push_u32_array(bytes, &link.position_bits);
        push_u32_array(bytes, &link.rotation_bits);
        push_u32_array(bytes, &link.linear_velocity_bits);
        push_u32_array(bytes, &link.angular_velocity_bits);
    }
    push_len(bytes, physics.joints.len())?;
    for joint in &physics.joints {
        bytes.extend_from_slice(&joint.position_bits.to_le_bytes());
        bytes.extend_from_slice(&joint.velocity_bits.to_le_bytes());
    }
    Ok(())
}

fn push_len(bytes: &mut Vec<u8>, length: usize) -> Result<(), MotorReplayCodecError> {
    bytes.extend_from_slice(
        &u32::try_from(length)
            .map_err(|_| MotorReplayCodecError::CapacityExceeded)?
            .to_le_bytes(),
    );
    Ok(())
}

fn push_i64_values(bytes: &mut Vec<u8>, values: &[i64]) -> Result<(), MotorReplayCodecError> {
    push_len(bytes, values.len())?;
    push_i64_array(bytes, values);
    Ok(())
}

fn push_i64_array(bytes: &mut Vec<u8>, values: &[i64]) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn push_u32_array(bytes: &mut Vec<u8>, values: &[u32]) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn push_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), MotorReplayCodecError> {
    push_len(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

struct Reader<'a> {
    remaining: &'a [u8],
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], MotorReplayCodecError> {
        if self.remaining.len() < length {
            return Err(MotorReplayCodecError::Truncated);
        }
        let (value, remaining) = self.remaining.split_at(length);
        self.remaining = remaining;
        Ok(value)
    }

    fn read_u32(&mut self) -> Result<u32, MotorReplayCodecError> {
        Ok(u32::from_le_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| MotorReplayCodecError::Truncated)?,
        ))
    }

    fn read_u64(&mut self) -> Result<u64, MotorReplayCodecError> {
        Ok(u64::from_le_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| MotorReplayCodecError::Truncated)?,
        ))
    }

    fn read_i64(&mut self) -> Result<i64, MotorReplayCodecError> {
        Ok(i64::from_le_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| MotorReplayCodecError::Truncated)?,
        ))
    }

    fn read_len(&mut self, maximum: usize) -> Result<usize, MotorReplayCodecError> {
        let value = self.read_u32()? as usize;
        if value > maximum {
            Err(MotorReplayCodecError::CapacityExceeded)
        } else {
            Ok(value)
        }
    }

    fn read_i64_values(&mut self, maximum: usize) -> Result<Vec<i64>, MotorReplayCodecError> {
        let length = self.read_len(maximum)?;
        (0..length).map(|_| self.read_i64()).collect()
    }

    fn read_u32_array<const N: usize>(&mut self) -> Result<[u32; N], MotorReplayCodecError> {
        let mut output = [0; N];
        for value in &mut output {
            *value = self.read_u32()?;
        }
        Ok(output)
    }

    fn read_u8_array<const N: usize>(&mut self) -> Result<[u8; N], MotorReplayCodecError> {
        self.take(N)?
            .try_into()
            .map_err(|_| MotorReplayCodecError::Truncated)
    }

    fn read_physics_snapshot(
        &mut self,
    ) -> Result<PhysXRawArticulationSnapshot, MotorReplayCodecError> {
        let link_count = self.read_len(MAX_LINKS)?;
        let mut links = Vec::with_capacity(link_count);
        for _ in 0..link_count {
            links.push(LinkState {
                user_token: self.read_u64()?,
                position_bits: self.read_u32_array()?,
                rotation_bits: self.read_u32_array()?,
                linear_velocity_bits: self.read_u32_array()?,
                angular_velocity_bits: self.read_u32_array()?,
            });
        }
        let joint_count = self.read_len(MAX_JOINTS)?;
        let mut joints = Vec::with_capacity(joint_count);
        for _ in 0..joint_count {
            joints.push(JointState {
                position_bits: self.read_u32()?,
                velocity_bits: self.read_u32()?,
            });
        }
        Ok(PhysXRawArticulationSnapshot { links, joints })
    }

    fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotorReplayCodecError {
    UnsupportedVersion(u16),
    MagicMismatch,
    Truncated,
    CapacityExceeded,
    InvalidBounds,
    ChecksumMismatch,
    TrailingBytes,
    NonCanonical,
}

impl MotorReplayCodecError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::UnsupportedVersion(_) => "UNSUPPORTED_MOTOR_CHECKPOINT_VERSION",
            Self::MagicMismatch => "MOTOR_CHECKPOINT_MAGIC_MISMATCH",
            Self::Truncated => "MOTOR_CHECKPOINT_TRUNCATED",
            Self::CapacityExceeded => "MOTOR_CHECKPOINT_CAPACITY_EXCEEDED",
            Self::InvalidBounds => "MOTOR_CHECKPOINT_BOUNDS_INVALID",
            Self::ChecksumMismatch => "MOTOR_CHECKPOINT_CHECKSUM_MISMATCH",
            Self::TrailingBytes => "MOTOR_CHECKPOINT_TRAILING_BYTES",
            Self::NonCanonical => "MOTOR_CHECKPOINT_NON_CANONICAL",
        }
    }
}

impl Display for MotorReplayCodecError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for MotorReplayCodecError {}

#[cfg(all(test, any(feature = "physx-sdk", feature = "mock-abi")))]
mod tests {
    use next_contracts::ids::PersistentId;

    use super::*;
    use crate::{
        CompiledBodySchemaV1, DeterministicHumanoidMotor, reference_humanoid_body_schema_v1,
    };

    fn runtime() -> DeterministicHumanoidMotor {
        let compiled = CompiledBodySchemaV1::compile(
            &reference_humanoid_body_schema_v1(),
            PersistentId::from_bytes([5; 16]),
        )
        .expect("compile");
        DeterministicHumanoidMotor::create(compiled).expect("runtime")
    }

    #[test]
    #[cfg(feature = "mock-abi")]
    fn checkpoint_round_trip_and_fresh_scene_continuation_are_exact() {
        let mut first = runtime();
        for ordinal in 0..5 {
            first
                .step_motor_frame(&[ordinal * 10_000; 23], [100_000, 0, 0])
                .expect("warmup");
        }
        let checkpoint = first.checkpoint();
        let bytes = checkpoint.canonical_bytes().expect("encode");
        let decoded = HumanoidMotorCheckpoint::from_canonical_bytes(&bytes).expect("decode");
        assert_eq!(decoded, checkpoint);
        let mut second = runtime();
        second.restore_fresh(&decoded).expect("restore");
        for ordinal in 0..20 {
            let action = [ordinal * 1_000 - 10_000; 23];
            let left = first
                .step_motor_frame(&action, [0, 0, 50_000])
                .expect("left continuation");
            let right = second
                .step_motor_frame(&action, [0, 0, 50_000])
                .expect("right continuation");
            assert_eq!(
                left.deterministic_hash().expect("left hash"),
                right.deterministic_hash().expect("right hash")
            );
            assert_eq!(left, right);
        }
    }

    #[test]
    #[cfg(feature = "physx-sdk")]
    fn native_restore_reconstructs_solver_continuation_for_ten_thousand_substeps() {
        let mut source = runtime();
        for ordinal in 0..5 {
            source
                .step_motor_frame(&[ordinal * 10_000; 23], [100_000, 0, 0])
                .expect("warmup");
        }
        let checkpoint = source.checkpoint();
        let bytes = checkpoint.canonical_bytes().expect("encode");
        let decoded = HumanoidMotorCheckpoint::from_canonical_bytes(&bytes).expect("decode");
        let mut restored = runtime();
        restored.restore_fresh(&decoded).expect("replay restore");
        for ordinal in 0..2_500 {
            let action_value = i64::from(ordinal % 21) * 1_000 - 10_000;
            let action = [action_value; 23];
            let left = source
                .step_motor_frame(&action, [0, 0, 25_000])
                .expect("source continuation");
            let right = restored
                .step_motor_frame(&action, [0, 0, 25_000])
                .expect("restored continuation");
            assert_eq!(left, right, "first differing motor frame {ordinal}");
        }
    }

    #[test]
    fn tampered_effort_prefix_fails_at_a_witness_before_publication() {
        let mut source = runtime();
        for ordinal in 0..5 {
            source
                .step_motor_frame(&[ordinal * 10_000; 23], [100_000, 0, 0])
                .expect("warmup");
        }
        let mut checkpoint = source.checkpoint();
        checkpoint.replay_frames[2].post_safety_efforts_micronewton_metres[0] += 1_000_000;
        let before = runtime().checkpoint();
        let mut target = runtime();
        let error = target
            .restore_fresh(&checkpoint)
            .expect_err("tampered effort must diverge");
        assert_eq!(error.stable_code(), "MOTOR_RUNTIME_RESTORE_DIVERGENCE");
        assert_eq!(target.checkpoint(), before);
    }

    #[test]
    fn non_contiguous_or_oversized_prefix_is_rejected_before_replay() {
        let mut source = runtime();
        source.step_motor_frame(&[0; 23], [0; 3]).expect("frame");
        let mut checkpoint = source.checkpoint();
        checkpoint.replay_frames[0].motor_tick = 2;
        let mut target = runtime();
        let error = target
            .restore_fresh(&checkpoint)
            .expect_err("non-contiguous prefix");
        assert_eq!(error.stable_code(), "MOTOR_RUNTIME_REPLAY_BOUNDS_INVALID");

        let mut checkpoint = source.checkpoint();
        let frame = checkpoint.replay_frames[0].clone();
        checkpoint.replay_frames = vec![frame; MAX_REPLAY_MOTOR_TICKS + 1];
        checkpoint.motor_tick = (MAX_REPLAY_MOTOR_TICKS + 1) as u64;
        let error = checkpoint.canonical_bytes().expect_err("oversized prefix");
        assert_eq!(error.stable_code(), "MOTOR_CHECKPOINT_BOUNDS_INVALID");
    }

    #[test]
    fn maximum_stage0_prefix_fits_the_declared_four_mebibyte_envelope() {
        let mut source = runtime();
        source.step_motor_frame(&[0; 23], [0; 3]).expect("frame");
        let mut checkpoint = source.checkpoint();
        let template = checkpoint.replay_frames[0].clone();
        checkpoint.replay_frames = (1..=MAX_REPLAY_MOTOR_TICKS)
            .map(|tick| MotorReplayFrame {
                motor_tick: tick as u64,
                ..template.clone()
            })
            .collect();
        checkpoint.motor_tick = MAX_REPLAY_MOTOR_TICKS as u64;
        let bytes = checkpoint.canonical_bytes().expect("maximum prefix");
        assert!(bytes.len() <= MAX_CHECKPOINT_BYTES);
        assert_eq!(
            HumanoidMotorCheckpoint::from_canonical_bytes(&bytes).expect("maximum decode"),
            checkpoint
        );
    }

    #[test]
    fn restoring_the_same_checkpoint_twice_is_idempotent() {
        let mut source = runtime();
        for ordinal in 0..8 {
            source
                .step_motor_frame(&[ordinal * 2_000; 23], [0, 25_000, 0])
                .expect("warmup");
        }
        let checkpoint = source.checkpoint();
        let mut left = runtime();
        let mut right = runtime();
        assert_eq!(
            left.restore_fresh(&checkpoint).expect("left restore"),
            right.restore_fresh(&checkpoint).expect("right restore")
        );
        for ordinal in 0..32 {
            let action = [ordinal * 500 - 8_000; 23];
            assert_eq!(
                left.step_motor_frame(&action, [0, 0, 10_000])
                    .expect("left"),
                right
                    .step_motor_frame(&action, [0, 0, 10_000])
                    .expect("right")
            );
        }
    }

    #[test]
    fn unsupported_version_is_rejected_before_nested_decode_or_checksum() {
        let checkpoint = runtime().checkpoint();
        let mut bytes = checkpoint.canonical_bytes().expect("encode");
        bytes[8..10].copy_from_slice(&1_u16.to_le_bytes());
        bytes.truncate(10);
        let error = HumanoidMotorCheckpoint::from_canonical_bytes(&bytes)
            .expect_err("unsupported old alpha version");
        assert_eq!(error.stable_code(), "UNSUPPORTED_MOTOR_CHECKPOINT_VERSION");
    }
}
