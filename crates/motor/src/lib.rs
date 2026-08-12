#![forbid(unsafe_code)]

mod biomechanics;
mod compiler;
mod compiler_v2;
mod contact_classifier;
#[cfg(test)]
mod contact_classifier_tests;
mod control;
mod humanoid;
mod mirror;
mod mirror_v2;
mod observation;
mod performance;
mod replay;
mod runtime;
mod safety_control;
#[cfg(test)]
mod safety_control_tests;
mod training;

pub use biomechanics::{
    BIOMECHANICS_HUMANOID_BODY_COUNT, BIOMECHANICS_HUMANOID_COLLIDER_COUNT,
    BIOMECHANICS_HUMANOID_DOF, BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
    BIOMECHANICS_HUMANOID_TOTAL_MASS_MICROKILOGRAMS, biomechanics_humanoid_body_schema_v2,
};
pub use compiler::{CompiledBodySchemaV1, CompiledPhysicsDescriptorsV1, MotorCompileError};
pub use compiler_v2::{CompiledBodySchemaV2, CompiledPhysicsDescriptorsV2};
pub use contact_classifier::{
    ACTIVE_CONTACT_IMPULSE_MICRONEWTON_SECONDS, BiomechanicsContactClassV1,
    BiomechanicsContactClassifier, BiomechanicsContactFrameV1, BiomechanicsSkillContactProfileV1,
    CONTACT_BRUSH_CEILING_MICRONEWTON_SECONDS, ClassifiedBiomechanicsContactV1,
    ContactClassificationError, ContactPairKeyV1, HUMANOID_GROUND_ACTOR_TOKEN,
    HUMANOID_SAFETY_CONTACT_PROFILE_SHA256, LOW_IMPULSE_GRACE_SUBSTEPS,
};
pub use control::{
    ACTUATOR_EFFORT_CLAMPED, ACTUATOR_RATE_CLAMPED, ACTUATOR_TARGET_CLAMPED, FixedPdController,
    JointControlStateV1, MotorControlError,
};
pub use humanoid::{
    REFERENCE_HUMANOID_DOF, REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES,
    reference_humanoid_body_schema_v1,
};
pub use mirror::{stage0_isaac_mirror_descriptor_json_v2, stage0_isaac_mirror_golden_json_v2};
pub use mirror_v2::biomechanics_isaac_mirror_descriptor_json_v1;
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
pub use safety_control::{
    ACTUATOR_POWER_CLAMPED, ACTUATOR_TARGET_SLEW_CLAMPED, ACTUATOR_WORK_CLAMPED,
    AppliedJointTargetV1, BiomechanicsSafetyCheckpointV1, BiomechanicsSafetyController,
    JointTargetEnvelopeV1, MotorSafetyError, NORMALIZED_RESIDUAL_ONE_Q1_30,
};
pub use training::{
    CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID, CURRICULUM_LOCOMOTION_REWARD_COEFFICIENTS_Q16,
    CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS, FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
    ISAAC_TRANSLATOR_VERSION, LOCOMOTION_REWARD_COEFFICIENTS_Q16, LOCOMOTION_REWARD_COMPONENT_IDS,
    LocomotionCurriculumStageV2, MotorEnvironmentProfile, MotorVectorRunner,
    STANDING_ENVIRONMENT_PROFILE_ID, STANDING_REWARD_COMPONENT_IDS, TrainingEnvironmentError,
    VectorPolicyStepInput, VectorResetOutput, VectorStepInput, VectorStepOutput,
    canonical_environment_manifest_v2, curriculum_locomotion_command_schedule,
    curriculum_locomotion_profile_hash_v2, curriculum_locomotion_stages_v2,
    derive_curriculum_locomotion_episode_seed_set, derive_episode_seed_set,
    derive_locomotion_episode_seed_set, flat_locomotion_command_profile_v1,
    flat_locomotion_command_schedule,
};
