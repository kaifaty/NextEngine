#![forbid(unsafe_code)]

mod compiler;
mod control;
mod humanoid;
mod mirror;
mod observation;
mod performance;
mod replay;
mod runtime;
mod training;

pub use compiler::{CompiledBodySchemaV1, CompiledPhysicsDescriptorsV1, MotorCompileError};
pub use control::{
    ACTUATOR_EFFORT_CLAMPED, ACTUATOR_RATE_CLAMPED, ACTUATOR_TARGET_CLAMPED, FixedPdController,
    JointControlStateV1, MotorControlError,
};
pub use humanoid::{REFERENCE_HUMANOID_DOF, reference_humanoid_body_schema_v1};
pub use mirror::{stage0_isaac_mirror_descriptor_json_v2, stage0_isaac_mirror_golden_json_v2};
pub use observation::{
    MotorObservationBuilder, MotorObservationError, MotorVelocityFrameV1,
    rotate_world_to_root_local_q1_30,
};
pub use performance::{
    HumanoidPerformanceError, HumanoidPerformanceReportV1, HumanoidWorkerPerformanceV1,
    run_reference_humanoid_performance_v1,
};
pub use replay::{MOTOR_RUNTIME_CHECKPOINT_SCHEMA_VERSION, MotorReplayCodecError};
pub use runtime::{
    DeterministicHumanoidMotor, HumanoidMotorCheckpoint, MAX_REPLAY_MOTOR_TICKS, MotorFrameResult,
    MotorReplayFrame, MotorRuntimeError,
};
pub use training::{
    FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, LOCOMOTION_REWARD_COEFFICIENTS_Q16,
    LOCOMOTION_REWARD_COMPONENT_IDS, MotorEnvironmentProfile, MotorVectorRunner,
    STANDING_ENVIRONMENT_PROFILE_ID, STANDING_REWARD_COMPONENT_IDS, TrainingEnvironmentError,
    VectorPolicyStepInput, VectorResetOutput, VectorStepInput, VectorStepOutput,
    canonical_environment_manifest_v2, derive_episode_seed_set, derive_locomotion_episode_seed_set,
    flat_locomotion_command_profile_v1, flat_locomotion_command_schedule,
};
