use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::motor::MotorContractError;

use crate::{MotorReplayCodecError, MotorRuntimeError};

#[derive(Debug)]
pub enum TrainingEnvironmentError {
    Runtime(MotorRuntimeError),
    Replay(MotorReplayCodecError),
    Contract(MotorContractError),
    Compile,
    UnsupportedProfile,
    SlotCount,
    SlotIdentity,
    SlotNotReset,
    IncompleteBatch,
    StaleEpisode,
    TerminalSlot,
    ExternalCommandForbidden,
    ActionLength,
    EpisodeOverflow,
    SeedProfile,
    SeedCollision,
    ScheduleBounds,
    RewardFacts,
    CheckpointIdentity,
    ArithmeticOverflow,
}

impl TrainingEnvironmentError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Runtime(error) => error.stable_code(),
            Self::Replay(error) => (*error).stable_code(),
            Self::Contract(error) => error.stable_code(),
            Self::Compile => "MOTOR_ENV_COMPILE_FAILED",
            Self::UnsupportedProfile => "UNSUPPORTED_MOTOR_ENVIRONMENT_PROFILE",
            Self::SlotCount => "MOTOR_ENV_SLOT_COUNT_INVALID",
            Self::SlotIdentity => "MOTOR_ENV_SLOT_IDENTITY_INVALID",
            Self::SlotNotReset => "MOTOR_ENV_SLOT_NOT_RESET",
            Self::IncompleteBatch => "MOTOR_ENV_BATCH_INCOMPLETE",
            Self::StaleEpisode => "MOTOR_ENV_EPISODE_STALE",
            Self::TerminalSlot => "MOTOR_ENV_SLOT_TERMINAL",
            Self::ExternalCommandForbidden => "MOTOR_ENV_EXTERNAL_COMMAND_FORBIDDEN",
            Self::ActionLength => "MOTOR_ENV_ACTION_LENGTH_INVALID",
            Self::EpisodeOverflow => "MOTOR_ENV_EPISODE_OVERFLOW",
            Self::SeedProfile => "MOTOR_ENV_SEED_PROFILE_INVALID",
            Self::SeedCollision => "MOTOR_ENV_SEED_COLLISION",
            Self::ScheduleBounds => "MOTOR_ENV_COMMAND_SCHEDULE_BOUNDS",
            Self::RewardFacts => "MOTOR_ENV_REWARD_FACTS_INVALID",
            Self::CheckpointIdentity => "MOTOR_ENV_CHECKPOINT_IDENTITY_MISMATCH",
            Self::ArithmeticOverflow => "MOTOR_ENV_ARITHMETIC_OVERFLOW",
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

impl From<MotorReplayCodecError> for TrainingEnvironmentError {
    fn from(value: MotorReplayCodecError) -> Self {
        Self::Replay(value)
    }
}

impl From<MotorContractError> for TrainingEnvironmentError {
    fn from(value: MotorContractError) -> Self {
        Self::Contract(value)
    }
}
