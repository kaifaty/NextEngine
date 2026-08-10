use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};
use next_physics_physx::PhysXRawArticulationSnapshot;
use next_physics_physx_ffi::{JointState, LinkState};

use crate::{HumanoidMotorCheckpoint, MotorFrameResult};

const CHECKPOINT_MAGIC: &[u8; 8] = b"NEMOTCP\0";
pub const MOTOR_RUNTIME_CHECKPOINT_SCHEMA_VERSION: u16 = 1;
const MAX_CHECKPOINT_BYTES: usize = 1_048_576;
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
        push_physics_snapshot(&mut bytes, &self.physics)?;
        if let Some(witness) = &self.restore_witness_physics {
            bytes.push(1);
            push_i64_values(&mut bytes, &self.restore_witness_efforts_micronewton_metres)?;
            push_physics_snapshot(&mut bytes, witness)?;
        } else {
            bytes.push(0);
        }
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
        let physics = reader.read_physics_snapshot()?;
        let (restore_witness_physics, restore_witness_efforts_micronewton_metres) =
            match reader.read_u8()? {
                0 => (None, Vec::new()),
                1 => {
                    let efforts = reader.read_i64_values(MAX_CHANNELS)?;
                    (Some(reader.read_physics_snapshot()?), efforts)
                }
                _ => return Err(MotorReplayCodecError::NonCanonical),
            };
        if !reader.is_empty() {
            return Err(MotorReplayCodecError::TrailingBytes);
        }
        let value = Self {
            motor_tick,
            applied_action_microradians,
            command_raw,
            previous_efforts_micronewton_metres,
            physics,
            restore_witness_physics,
            restore_witness_efforts_micronewton_metres,
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
        || value.physics.links.is_empty()
        || value.physics.links.len() > MAX_LINKS
        || value.physics.joints.is_empty()
        || value.physics.joints.len() > MAX_JOINTS
        || value.physics.joints.len() != value.applied_action_microradians.len()
        || value
            .physics
            .links
            .windows(2)
            .any(|pair| pair[0].user_token >= pair[1].user_token)
    {
        return Err(MotorReplayCodecError::InvalidBounds);
    }
    match &value.restore_witness_physics {
        Some(witness)
            if value.restore_witness_efforts_micronewton_metres.len()
                == value.applied_action_microradians.len()
                && witness.links.len() == value.physics.links.len()
                && witness.joints.len() == value.physics.joints.len() => {}
        None if value.restore_witness_efforts_micronewton_metres.is_empty() => {}
        _ => return Err(MotorReplayCodecError::InvalidBounds),
    }
    Ok(())
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

    fn read_u8(&mut self) -> Result<u8, MotorReplayCodecError> {
        Ok(self.take(1)?[0])
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
    fn native_restore_fails_closed_when_solver_continuation_is_not_representable() {
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
        let error = restored
            .restore_fresh(&decoded)
            .expect_err("unrepresented native solver continuation must not be tolerated");
        assert_eq!(error.stable_code(), "MOTOR_RUNTIME_RESTORE_DIVERGENCE");
    }

    #[test]
    fn unsupported_version_is_rejected_before_nested_decode_or_checksum() {
        let checkpoint = runtime().checkpoint();
        let mut bytes = checkpoint.canonical_bytes().expect("encode");
        bytes[8..10].copy_from_slice(&0_u16.to_le_bytes());
        bytes.truncate(10);
        let error = HumanoidMotorCheckpoint::from_canonical_bytes(&bytes)
            .expect_err("unsupported old alpha version");
        assert_eq!(error.stable_code(), "UNSUPPORTED_MOTOR_CHECKPOINT_VERSION");
    }
}
