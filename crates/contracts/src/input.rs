mod action_map;
mod actions;
mod batches;
mod codec;
mod constants;
mod context;
mod controller;
mod ingress;
mod profiles;

pub use action_map::{
    ActionAccessibilityV1, ActionBindingTransformV1, ActionBindingV1, ActionConflictPolicyV1,
    ActionDefinitionV1, ActionMapManifestV1, PlayerActionValueKindV1,
};
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
    ACTION_MAP_MANIFEST_SCHEMA_VERSION, CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION,
    CLOSED_INGRESS_BATCH_SCHEMA_VERSION, CORE_CAMERA_ORBIT_ACTION_ID, CORE_EQUIP_USE_ACTION_ID,
    CORE_GAMEPLAY_CONTEXT_ID, CORE_GAMEPLAY_CONTEXT_STACK_ID, CORE_INTERACT_ACTION_ID,
    CORE_KEYBOARD_MOUSE_ACTION_MAP_ID, CORE_MELEE_ACTION_ID, CORE_MOVE_ACTION_ID,
    CORE_PICKUP_ACTION_ID, INGRESS_ASSIGNMENT_PROFILE_SCHEMA_VERSION,
    INGRESS_ASSIGNMENT_SCHEMA_VERSION, INGRESS_CHECKPOINT_SCHEMA_VERSION,
    INPUT_CONTEXT_SCHEMA_VERSION, INPUT_CONTEXT_STACK_SCHEMA_VERSION,
    INPUT_MAPPING_RECEIPT_SCHEMA_VERSION, INPUT_SAMPLE_SCHEMA_VERSION, KEYBOARD_A_CONTROL_PATH_ID,
    KEYBOARD_D_CONTROL_PATH_ID, KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_E_CONTROL_PATH_ID,
    KEYBOARD_F_CONTROL_PATH_ID, KEYBOARD_Q_CONTROL_PATH_ID, KEYBOARD_R_CONTROL_PATH_ID,
    KEYBOARD_S_CONTROL_PATH_ID, KEYBOARD_W_CONTROL_PATH_ID, MAX_ACTION_BINDING_MODIFIERS,
    MAX_ACTION_BINDINGS_PER_ACTION, MAX_ACTION_MAP_ACTIONS, MAX_ACTION_MAP_BINDINGS,
    MAX_ACTION_MAP_DEVICE_CLASSES, MAX_DERIVED_COMMANDS_PER_INPUT_FRAME, MAX_INPUT_CONTEXT_ACTIONS,
    MAX_INPUT_CONTEXTS, MAX_INPUT_IDENTIFIER_BYTES, MAX_PLAYER_ACTIONS_PER_FRAME,
    MOUSE_DELTA_CONTROL_PATH_ID, MOUSE_DEVICE_CLASS_ID, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS,
    PLAYER_CONTROLLER_REGISTRY_SCHEMA_VERSION, PLAYER_INTERACTION_SYSTEM_ID,
    RUNTIME_ADMISSION_LIMITS_SCHEMA_VERSION, TICK_RATE_PROFILE_SCHEMA_VERSION,
};
pub use context::{InputContextCapturePolicyV1, InputContextStackV1, InputContextV1};
pub use controller::{
    InputActionMappingResultV2, InputDerivedCommandRefV2, InputMappingCodeV1,
    InputMappingReceiptV1, InputMappingReceiptV2, PlayerControllerBindingV1,
    PlayerControllerRegistryV1,
};
pub use ingress::{
    IngressAssignmentV1, IngressCheckpointV1, IngressEquivalenceReceiptV1, IngressResultCodeV1,
    IngressSubjectKindV1,
};
pub use profiles::{IngressAssignmentProfileV1, RuntimeAdmissionLimitsV1, TickRateProfileV1};

#[cfg(test)]
mod tests;
