mod actions;
mod batches;
mod codec;
mod constants;
mod controller;
mod ingress;
mod profiles;

pub use actions::{
    InputSampleV1, PlayerActionFrameV1, PlayerActionPhaseV1, PlayerActionV1, PlayerActionValueV1,
    core_player_action_map_v1_hash, core_player_action_map_v2_hash,
};
pub use batches::{
    ClosedCommandAdmissionBatchBodyV2, ClosedCommandAdmissionBatchV2, ClosedIngressBatchBodyV1,
    ClosedIngressBatchV1,
};
pub use codec::InputContractError;
pub use constants::{
    CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION, CLOSED_INGRESS_BATCH_SCHEMA_VERSION,
    CORE_EQUIP_USE_ACTION_ID, CORE_INTERACT_ACTION_ID, CORE_MELEE_ACTION_ID, CORE_MOVE_ACTION_ID,
    CORE_PICKUP_ACTION_ID, INGRESS_ASSIGNMENT_PROFILE_SCHEMA_VERSION,
    INGRESS_ASSIGNMENT_SCHEMA_VERSION, INGRESS_CHECKPOINT_SCHEMA_VERSION,
    INPUT_SAMPLE_SCHEMA_VERSION, MAX_PLAYER_ACTIONS_PER_FRAME, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS,
    PLAYER_CONTROLLER_REGISTRY_SCHEMA_VERSION, PLAYER_INTERACTION_SYSTEM_ID,
    RUNTIME_ADMISSION_LIMITS_SCHEMA_VERSION, TICK_RATE_PROFILE_SCHEMA_VERSION,
};
pub use controller::{
    InputMappingCodeV1, InputMappingReceiptV1, PlayerControllerBindingV1,
    PlayerControllerRegistryV1,
};
pub use ingress::{
    IngressAssignmentV1, IngressCheckpointV1, IngressEquivalenceReceiptV1, IngressResultCodeV1,
    IngressSubjectKindV1,
};
pub use profiles::{IngressAssignmentProfileV1, RuntimeAdmissionLimitsV1, TickRateProfileV1};

#[cfg(test)]
mod tests;
