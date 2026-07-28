pub const RUNTIME_ADMISSION_LIMITS_SCHEMA_VERSION: u16 = 1;
pub const TICK_RATE_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const INGRESS_ASSIGNMENT_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const PLAYER_ACTION_FRAME_SCHEMA_VERSION: u16 = 1;
pub const INPUT_SAMPLE_SCHEMA_VERSION: u16 = 1;
pub const INGRESS_ASSIGNMENT_SCHEMA_VERSION: u16 = 1;
pub const CLOSED_INGRESS_BATCH_SCHEMA_VERSION: u16 = 1;
pub const CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION: u16 = 2;
pub const PLAYER_CONTROLLER_REGISTRY_SCHEMA_VERSION: u16 = 1;
pub const INGRESS_CHECKPOINT_SCHEMA_VERSION: u16 = 1;

pub const PLAYER_ACTION_FRAME_SCHEMA_ID: &str = "nextengine.player-action-frame";
pub const PLAYER_ACTION_SOURCE_CLASS: &str = "nextengine.input.player-action";
pub const CORE_MOVE_ACTION_ID: &str = "nextengine.action.move";
pub const CORE_INTERACT_ACTION_ID: &str = "nextengine.action.interact";
pub const CORE_PICKUP_ACTION_ID: &str = "nextengine.action.pickup";
pub const CORE_EQUIP_USE_ACTION_ID: &str = "nextengine.action.equip-use";
pub const CORE_MELEE_ACTION_ID: &str = "nextengine.action.melee";
pub const PLAYER_INTERACTION_SYSTEM_ID: &str = "nextengine.system.player-interaction";
pub const MAX_PLAYER_ACTIONS_PER_FRAME: usize = 64;

pub(super) const RUNTIME_OWNER_ID: &str = "nextengine.runtime";
pub(super) const INPUT_OWNER_ID: &str = "nextengine.input";
pub(super) const RUNTIME_ADMISSION_LIMITS_SCHEMA_ID: &str = "nextengine.runtime-admission-limits";
pub(super) const TICK_RATE_PROFILE_SCHEMA_ID: &str = "nextengine.tick-rate-profile";
pub(super) const INGRESS_ASSIGNMENT_PROFILE_SCHEMA_ID: &str =
    "nextengine.ingress-assignment-profile";
pub(super) const INPUT_SAMPLE_SCHEMA_ID: &str = "nextengine.input-sample";
pub(super) const INGRESS_ASSIGNMENT_SCHEMA_ID: &str = "nextengine.ingress-assignment";
pub(super) const INGRESS_RECEIPT_SCHEMA_ID: &str = "nextengine.ingress-equivalence-receipt";
pub(super) const CLOSED_INGRESS_BODY_SCHEMA_ID: &str = "nextengine.closed-ingress-batch-body";
pub(super) const CLOSED_INGRESS_SCHEMA_ID: &str = "nextengine.closed-ingress-batch";
pub(super) const CLOSED_COMMAND_BODY_SCHEMA_ID: &str =
    "nextengine.closed-command-admission-batch-body";
pub(super) const CLOSED_COMMAND_SCHEMA_ID: &str = "nextengine.closed-command-admission-batch";
pub(super) const PLAYER_CONTROLLER_REGISTRY_SCHEMA_ID: &str =
    "nextengine.player-controller-registry";
pub(super) const INPUT_MAPPING_RECEIPT_SCHEMA_ID: &str = "nextengine.input-mapping-receipt";
pub(super) const INGRESS_CHECKPOINT_SCHEMA_ID: &str = "nextengine.ingress-checkpoint";
pub(super) const SEGMENT_V1: &str = "v1";
pub(super) const SEGMENT_V2: &str = "v2";
