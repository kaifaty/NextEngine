use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::motor::{MOTOR_EPISODE_SEED_SET_V1_SCHEMA_VERSION, MotorEpisodeSeedSetV1};

use crate::{
    CompiledBodySchemaV1, DeterministicHumanoidMotor, MotorFrameResult, MotorRuntimeError,
    reference_humanoid_body_schema_v1,
};

pub const MAX_CPU_VECTOR_SLOTS: u32 = 256;
pub const DEFAULT_MAX_EPISODE_MOTOR_STEPS: u64 = 60 * 60;

const RANDOMIZATION_PURPOSES: [&str; 4] = [
    "randomization.action-noise",
    "randomization.friction",
    "randomization.initial-pose",
    "randomization.terrain",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorStepInput {
    pub vector_slot: u32,
    pub action_microradians: Vec<i64>,
    pub command_raw: [i64; 3],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorResetOutput {
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub seed_set: MotorEpisodeSeedSetV1,
    pub observation_raw: Vec<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorStepOutput {
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub frame: MotorFrameResult,
    pub reward_components_raw: Vec<(SchemaId, i64)>,
    pub terminal_reason_id: Option<SchemaId>,
}

#[derive(Debug)]
struct VectorSlot {
    episode_ordinal: u64,
    runtime: DeterministicHumanoidMotor,
}

#[derive(Debug)]
pub struct MotorVectorRunner {
    run_root: ContentHash,
    slots: Vec<VectorSlot>,
    maximum_episode_steps: u64,
}

impl MotorVectorRunner {
    pub fn create(
        slot_count: u32,
        run_root: ContentHash,
    ) -> Result<Self, TrainingEnvironmentError> {
        if slot_count == 0 || slot_count > MAX_CPU_VECTOR_SLOTS {
            return Err(TrainingEnvironmentError::SlotCount);
        }
        let schema = reference_humanoid_body_schema_v1();
        let mut slots = Vec::with_capacity(slot_count as usize);
        for vector_slot in 0..slot_count {
            let subject_id = subject_id(run_root, vector_slot);
            let compiled = CompiledBodySchemaV1::compile(&schema, subject_id)
                .map_err(|_| TrainingEnvironmentError::Compile)?;
            slots.push(VectorSlot {
                episode_ordinal: 0,
                runtime: DeterministicHumanoidMotor::create(compiled)?,
            });
        }
        Ok(Self {
            run_root,
            slots,
            maximum_episode_steps: DEFAULT_MAX_EPISODE_MOTOR_STEPS,
        })
    }

    #[must_use]
    pub fn slot_count(&self) -> u32 {
        self.slots.len() as u32
    }

    pub fn reset_all(
        &mut self,
        episode_ordinal: u64,
    ) -> Result<Vec<VectorResetOutput>, TrainingEnvironmentError> {
        let mut output = Vec::with_capacity(self.slots.len());
        for (slot_index, slot) in self.slots.iter_mut().enumerate() {
            slot.episode_ordinal = episode_ordinal;
            let vector_slot = slot_index as u32;
            output.push(VectorResetOutput {
                episode_ordinal,
                vector_slot,
                seed_set: derive_episode_seed_set(self.run_root, episode_ordinal, vector_slot)?,
                observation_raw: slot.runtime.reset()?,
            });
        }
        Ok(output)
    }

    pub fn step_lockstep(
        &mut self,
        inputs: Vec<VectorStepInput>,
    ) -> Result<Vec<VectorStepOutput>, TrainingEnvironmentError> {
        if inputs.len() != self.slots.len() {
            return Err(TrainingEnvironmentError::IncompleteBatch);
        }
        let mut by_slot = BTreeMap::new();
        for input in inputs {
            if input.vector_slot >= self.slot_count()
                || by_slot.insert(input.vector_slot, input).is_some()
            {
                return Err(TrainingEnvironmentError::SlotIdentity);
            }
        }
        let mut output = Vec::with_capacity(self.slots.len());
        for (slot_index, slot) in self.slots.iter_mut().enumerate() {
            let vector_slot = slot_index as u32;
            let input = by_slot
                .remove(&vector_slot)
                .ok_or(TrainingEnvironmentError::IncompleteBatch)?;
            let frame = slot
                .runtime
                .step_motor_frame(&input.action_microradians, input.command_raw)?;
            let terminal_reason_id = terminal_reason(&frame, self.maximum_episode_steps);
            let reward_components_raw = standing_reward_components(&frame, input.command_raw);
            output.push(VectorStepOutput {
                episode_ordinal: slot.episode_ordinal,
                vector_slot,
                frame,
                reward_components_raw,
                terminal_reason_id,
            });
        }
        output.sort_by_key(|value| (value.episode_ordinal, value.vector_slot));
        Ok(output)
    }
}

pub fn derive_episode_seed_set(
    run_root: ContentHash,
    episode_ordinal: u64,
    vector_slot: u32,
) -> Result<MotorEpisodeSeedSetV1, TrainingEnvironmentError> {
    let mut purpose_seeds = RANDOMIZATION_PURPOSES
        .into_iter()
        .map(|purpose| {
            let purpose_id = schema_id(purpose);
            let purpose_bytes = purpose.as_bytes();
            let mut preimage = Vec::new();
            preimage.extend_from_slice(b"nextengine.motor-episode-seed.v1\0");
            preimage.extend_from_slice(run_root.as_bytes());
            preimage.extend_from_slice(&episode_ordinal.to_le_bytes());
            preimage.extend_from_slice(&vector_slot.to_le_bytes());
            preimage.extend_from_slice(&(purpose_bytes.len() as u32).to_le_bytes());
            preimage.extend_from_slice(purpose_bytes);
            (purpose_id, sha256(&preimage))
        })
        .collect::<Vec<_>>();
    purpose_seeds.sort_by(|left, right| left.0.cmp(&right.0));
    if purpose_seeds
        .iter()
        .map(|(_, seed)| seed)
        .collect::<BTreeSet<_>>()
        .len()
        != purpose_seeds.len()
    {
        return Err(TrainingEnvironmentError::SeedCollision);
    }
    let value = MotorEpisodeSeedSetV1 {
        schema_version: MOTOR_EPISODE_SEED_SET_V1_SCHEMA_VERSION,
        run_root,
        episode_ordinal,
        vector_slot,
        purpose_seeds,
    };
    value
        .validate()
        .map_err(|_| TrainingEnvironmentError::SeedProfile)?;
    Ok(value)
}

fn subject_id(run_root: ContentHash, vector_slot: u32) -> PersistentId {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.motor-vector-subject.v1\0");
    preimage.extend_from_slice(run_root.as_bytes());
    preimage.extend_from_slice(&vector_slot.to_le_bytes());
    let digest = sha256(&preimage);
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    PersistentId::from_bytes(bytes)
}

fn standing_reward_components(
    frame: &MotorFrameResult,
    command_raw: [i64; 3],
) -> Vec<(SchemaId, i64)> {
    let root = frame.snapshot.links.first();
    let alive = i64::from(root.is_some_and(|root| root.position_micrometres[1] > 250_000));
    let command_tracking = root.map_or(-20_000_000, |root| {
        -((root.linear_velocity_micrometres_per_second[0] - command_raw[0])
            .unsigned_abs()
            .min(i64::MAX as u64) as i64)
    });
    let energy = -(frame
        .substep_efforts
        .iter()
        .flatten()
        .map(|effort| effort.effort_micronewton_metres.unsigned_abs() / 1_000_000)
        .sum::<u64>()
        .min(i64::MAX as u64) as i64);
    let upright = root.map_or(0, |root| root.rotation_q1_30[3].unsigned_abs() as i64);
    vec![
        (schema_id("reward.alive"), alive),
        (schema_id("reward.command-tracking"), command_tracking),
        (schema_id("reward.energy"), energy),
        (schema_id("reward.upright"), upright),
    ]
}

fn terminal_reason(frame: &MotorFrameResult, maximum_steps: u64) -> Option<SchemaId> {
    if frame
        .snapshot
        .links
        .first()
        .is_none_or(|root| root.position_micrometres[1] <= 250_000)
    {
        Some(schema_id("terminal.fall"))
    } else if frame.motor_tick >= maximum_steps {
        Some(schema_id("terminal.timeout"))
    } else {
        None
    }
}

fn schema_id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("engine-owned training identifiers are valid")
}

#[derive(Debug)]
pub enum TrainingEnvironmentError {
    Runtime(MotorRuntimeError),
    Compile,
    SlotCount,
    SlotIdentity,
    IncompleteBatch,
    SeedProfile,
    SeedCollision,
}

impl TrainingEnvironmentError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Runtime(error) => error.stable_code(),
            Self::Compile => "MOTOR_ENV_COMPILE_FAILED",
            Self::SlotCount => "MOTOR_ENV_SLOT_COUNT_INVALID",
            Self::SlotIdentity => "MOTOR_ENV_SLOT_IDENTITY_INVALID",
            Self::IncompleteBatch => "MOTOR_ENV_BATCH_INCOMPLETE",
            Self::SeedProfile => "MOTOR_ENV_SEED_PROFILE_INVALID",
            Self::SeedCollision => "MOTOR_ENV_SEED_COLLISION",
        }
    }
}

impl Display for TrainingEnvironmentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for TrainingEnvironmentError {}

impl From<MotorRuntimeError> for TrainingEnvironmentError {
    fn from(value: MotorRuntimeError) -> Self {
        Self::Runtime(value)
    }
}

#[cfg(all(test, any(feature = "physx-sdk", feature = "mock-abi")))]
mod tests {
    use super::*;

    fn input(slot: u32, action: i64) -> VectorStepInput {
        VectorStepInput {
            vector_slot: slot,
            action_microradians: vec![action; 23],
            command_raw: [100_000, 0, 0],
        }
    }

    #[test]
    fn publication_order_and_results_ignore_input_worker_permutation() {
        let run_root = ContentHash::from_bytes([4; 32]);
        let mut first = MotorVectorRunner::create(4, run_root).expect("first runner");
        let mut second = MotorVectorRunner::create(4, run_root).expect("second runner");
        assert_eq!(
            first.reset_all(17).expect("first reset"),
            second.reset_all(17).expect("second reset")
        );
        let ascending = vec![input(0, 1), input(1, 2), input(2, 3), input(3, 4)];
        let descending = vec![input(3, 4), input(2, 3), input(1, 2), input(0, 1)];
        let left = first.step_lockstep(ascending).expect("ascending");
        let right = second.step_lockstep(descending).expect("descending");
        assert_eq!(left, right);
        assert_eq!(
            left.iter()
                .map(|value| value.vector_slot)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
    }

    #[test]
    fn purpose_slot_and_episode_seeds_are_domain_separated() {
        let run_root = ContentHash::from_bytes([6; 32]);
        let first = derive_episode_seed_set(run_root, 1, 0).expect("first");
        let second = derive_episode_seed_set(run_root, 1, 1).expect("slot");
        let third = derive_episode_seed_set(run_root, 2, 0).expect("episode");
        assert_ne!(first.purpose_seeds, second.purpose_seeds);
        assert_ne!(first.purpose_seeds, third.purpose_seeds);
        assert_eq!(
            first
                .purpose_seeds
                .iter()
                .map(|(_, seed)| seed)
                .collect::<BTreeSet<_>>()
                .len(),
            RANDOMIZATION_PURPOSES.len()
        );
    }
}
