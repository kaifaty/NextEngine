#![forbid(unsafe_code)]

mod compiler;
mod control;
mod humanoid;
mod observation;
mod replay;
mod runtime;
mod training;

pub use compiler::{CompiledBodySchemaV1, CompiledPhysicsDescriptorsV1, MotorCompileError};
pub use control::{
    ACTUATOR_EFFORT_CLAMPED, ACTUATOR_RATE_CLAMPED, ACTUATOR_TARGET_CLAMPED, FixedPdController,
    JointControlStateV1, MotorControlError,
};
pub use humanoid::{REFERENCE_HUMANOID_DOF, reference_humanoid_body_schema_v1};
pub use observation::{MotorObservationBuilder, MotorObservationError};
pub use replay::{MOTOR_RUNTIME_CHECKPOINT_SCHEMA_VERSION, MotorReplayCodecError};
pub use runtime::{
    DeterministicHumanoidMotor, HumanoidMotorCheckpoint, MotorFrameResult, MotorRuntimeError,
};
pub use training::{
    MotorVectorRunner, TrainingEnvironmentError, VectorResetOutput, VectorStepInput,
    VectorStepOutput, derive_episode_seed_set,
};
