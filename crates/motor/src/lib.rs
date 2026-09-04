#![forbid(unsafe_code)]

mod biomechanics;
mod biomechanics_standing;
mod biomechanics_standing_runner;
mod capsule_procedural;
#[cfg(test)]
mod capsule_procedural_tests;
mod compiler;
mod compiler_v2;
mod compiler_v3;
mod contact_classifier;
#[cfg(test)]
mod contact_classifier_tests;
mod control;
mod functional_anatomy;
mod humanoid;
mod mirror;
mod mirror_v2;
#[cfg(all(test, feature = "physx-sdk"))]
mod native_safety_scenarios;
mod observation;
mod performance;
mod physical_animation;
mod procedural_standing;
#[cfg(test)]
mod procedural_standing_tests;
mod reference_baseline;
mod reference_pose_audit;
mod reference_tracking;
mod replay;
mod runtime;
mod safety_control;
#[cfg(test)]
mod safety_control_tests;
mod safety_mirror;
#[cfg(feature = "physx-sdk")]
mod safety_review;
mod terminal_v2;
#[cfg(test)]
mod terminal_v2_tests;
mod training;

pub use biomechanics::{
    BIOMECHANICS_HUMANOID_BODY_COUNT, BIOMECHANICS_HUMANOID_COLLIDER_COUNT,
    BIOMECHANICS_HUMANOID_DOF, BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
    BIOMECHANICS_HUMANOID_TOTAL_MASS_MICROKILOGRAMS,
    BIOMECHANICS_HUMANOID_V2_SHOULDER_HALF_WIDTH_MICROMETRES,
    BIOMECHANICS_HUMANOID_V3_CARRIER_INERTIA_MICROKILOGRAM_METRE_SQUARED,
    BIOMECHANICS_HUMANOID_V3_CARRIER_MASS_MICROKILOGRAMS,
    BIOMECHANICS_HUMANOID_V3_SHOULDER_HALF_WIDTH_MICROMETRES, biomechanics_humanoid_body_schema_v2,
    biomechanics_humanoid_body_schema_v3,
};
pub use biomechanics_standing::{
    BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID,
    BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID,
    BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS,
    BIOMECHANICS_FORWARD_START_STOP_OBSERVATION_LAYOUT_ID,
    BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16,
    BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS, BIOMECHANICS_STANDING_ACTION_LAYOUT_ID,
    BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID, BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID_V1,
    BIOMECHANICS_STANDING_MAXIMUM_EPISODE_STEPS, BIOMECHANICS_STANDING_OBSERVATION_LAYOUT_ID,
    BIOMECHANICS_STANDING_REWARD_COEFFICIENTS_Q16, BIOMECHANICS_STANDING_REWARD_COMPONENT_IDS,
    BiomechanicsForwardStartStopRewardFactsV1, BiomechanicsStandingRewardFactsV1,
    biomechanics_forward_start_stop_environment_manifest_v1,
    biomechanics_forward_start_stop_isaac_descriptor_json_v1,
    biomechanics_forward_start_stop_reward_q16_v1, biomechanics_standing_environment_manifest_v1,
    biomechanics_standing_environment_manifest_v2, biomechanics_standing_isaac_descriptor_json_v1,
    biomechanics_standing_isaac_descriptor_json_v2, biomechanics_standing_reward_q16_v1,
};
pub use biomechanics_standing_runner::{
    BiomechanicsStandingFrameResult, BiomechanicsStandingRunnerError,
    BiomechanicsStandingVectorRunner, BiomechanicsStandingVectorStepOutput,
};
pub use capsule_procedural::{
    CAPSULE_PROCEDURAL_MOTOR_PROFILE_VERSION_V1, CapsuleMotorDecisionV1, CapsuleMotorRouteV1,
    CapsuleProceduralMotorControllerV1, CapsuleProceduralMotorError,
};
pub use compiler::{
    CompiledBodySchemaV1, CompiledPhysicsDescriptorsV1, MotorCompileError,
    body_projection_compiler_profile_hash_v1,
};
pub use compiler_v2::{CompiledBodySchemaV2, CompiledPhysicsDescriptorsV2};
pub use compiler_v3::{
    BIOMECHANICS_BODY_MATERIAL_ID, BIOMECHANICS_DYNAMIC_FRICTION_Q16,
    BIOMECHANICS_GROUND_MATERIAL_ID, BIOMECHANICS_MATERIAL_COMBINE_PROFILE_ID,
    BIOMECHANICS_SOLE_MATERIAL_ID, BIOMECHANICS_STATIC_FRICTION_Q16, CompiledBodySchemaV3,
    CompiledPhysicsDescriptorsV3, biomechanics_material_catalog_v2,
    biomechanics_material_combine_profile_v1,
};
pub use contact_classifier::{
    ACTIVE_CONTACT_IMPULSE_MICRONEWTON_SECONDS, BiomechanicsContactClassV1,
    BiomechanicsContactClassifier, BiomechanicsContactFrameV1, BiomechanicsSkillContactProfileV1,
    CONTACT_BRUSH_CEILING_MICRONEWTON_SECONDS, ClassifiedBiomechanicsContactV1,
    ContactClassificationError, ContactPairKeyV1, HUMANOID_GROUND_ACTOR_TOKEN,
    HUMANOID_GROUND_SHAPE_TOKEN, HUMANOID_SAFETY_CONTACT_PROFILE_SHA256,
    LOW_IMPULSE_GRACE_SUBSTEPS,
};
pub use control::{
    ACTUATOR_CAPABILITY_CLAMPED, ACTUATOR_EFFORT_CLAMPED, ACTUATOR_RATE_CLAMPED,
    ACTUATOR_TARGET_CLAMPED, FixedPdController, JointControlStateV1, MotorControlError,
};
pub use functional_anatomy::{FunctionalAnatomyCompileError, compile_body_capability_envelope_v1};
pub use humanoid::{
    REFERENCE_HUMANOID_DOF, REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES,
    neutral_body_instance_projection_v1, reference_humanoid_body_instance_projection_v1,
    reference_humanoid_body_schema_v1,
};
pub use mirror::{stage0_isaac_mirror_descriptor_json_v2, stage0_isaac_mirror_golden_json_v2};
pub use mirror_v2::{
    biomechanics_isaac_mirror_descriptor_json_v1, biomechanics_isaac_mirror_descriptor_json_v2,
    biomechanics_isaac_mirror_descriptor_json_v3,
};
pub use observation::{
    MotorObservationBuilder, MotorObservationError, MotorVelocityFrameV1,
    rotate_world_to_root_local_q1_30,
};
pub use performance::{
    HumanoidPerformanceError, HumanoidPerformanceReportV1, HumanoidWorkerPerformanceV1,
    run_reference_humanoid_performance_v1,
};
pub use physical_animation::{
    PHYSICAL_ANIMATION_MAX_HELD_POSE_AGE_TICKS_V1,
    PHYSICAL_ANIMATION_REDUCED_POSE_CADENCE_TICKS_V1, PhysicalAnimationJointPoseV1,
    PhysicalAnimationLodDecisionV1, PhysicalAnimationLodLevelV1, PhysicalAnimationLodProfileV1,
    PhysicalAnimationLodProjectionV1, PhysicalAnimationLodPublicationModeV1,
    PhysicalAnimationLodRequestV1, PhysicalAnimationLodResourcesV1, PhysicalAnimationOwnerErrorV1,
    PhysicalAnimationOwnerV1, PhysicalAnimationPoseV1, PhysicalAnimationPresentationAvailabilityV1,
    PhysicalAnimationProjectionModeV1,
};
pub use procedural_standing::{
    BiomechanicsProceduralStandingControllerV1, PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS,
    PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS, PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS,
    ProceduralStandingError,
};
pub use reference_baseline::{ReferenceBaselineError, biomechanics_reference_baseline_json_v1};
pub use reference_pose_audit::{
    ReferencePoseAuditError, biomechanics_reference_pose_audit_json_v1,
};
pub use reference_tracking::{
    REFERENCE_REWARD_COEFFICIENTS_Q16, REFERENCE_REWARD_COMPONENT_IDS,
    REFERENCE_TRACKER_PROFILE_DOCUMENT_SHA256, REFERENCE_TRACKING_ENVIRONMENT_PROFILE_ID,
    biomechanics_reference_environment_manifest_v3, biomechanics_reference_tracking_profile_v1,
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
    OBSERVED_MAXIMUM_VELOCITY_QUANTIZATION_TOLERANCE_MICRORADIANS_PER_SECOND,
};
pub use safety_mirror::{SafetyMirrorError, biomechanics_safety_contact_mirror_golden_json_v1};
#[cfg(feature = "physx-sdk")]
pub use safety_review::{SafetyReviewError, biomechanics_native_safety_review_json_v1};
pub use terminal_v2::{
    BIOMECHANICS_FALL_HEIGHT_MICROMETRES, BIOMECHANICS_ROOT_NORM_TOLERANCE_Q2_60,
    BIOMECHANICS_TERMINAL_SUBSTEPS, BIOMECHANICS_WORLD_BOUND_MICROMETRES,
    BiomechanicsTerminalDecisionV1, BiomechanicsTerminalError, BiomechanicsTerminalEvaluator,
    BiomechanicsTerminalReasonV1,
};
pub use training::{
    BOUNDED_STANDING_ENVIRONMENT_PROFILE_ID, BOUNDED_STANDING_REWARD_COEFFICIENTS_Q16,
    BOUNDED_STANDING_REWARD_COMPONENT_IDS, CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
    CURRICULUM_LOCOMOTION_REWARD_COEFFICIENTS_Q16, CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS,
    FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, ISAAC_TRANSLATOR_VERSION,
    LOCOMOTION_REWARD_COEFFICIENTS_Q16, LOCOMOTION_REWARD_COMPONENT_IDS,
    LocomotionCurriculumStageV2, MotorEnvironmentProfile, MotorVectorRunner,
    STANDING_ENVIRONMENT_PROFILE_ID, STANDING_REWARD_COMPONENT_IDS, TrainingEnvironmentError,
    VectorPolicyStepInput, VectorResetOutput, VectorStepInput, VectorStepOutput,
    canonical_environment_manifest_v2, curriculum_locomotion_command_schedule,
    curriculum_locomotion_profile_hash_v2, curriculum_locomotion_stages_v2,
    derive_curriculum_locomotion_episode_seed_set, derive_episode_seed_set,
    derive_locomotion_episode_seed_set, flat_locomotion_command_profile_v1,
    flat_locomotion_command_schedule,
};
