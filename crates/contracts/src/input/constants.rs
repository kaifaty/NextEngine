pub const RUNTIME_ADMISSION_LIMITS_SCHEMA_VERSION: u16 = 1;
pub const TICK_RATE_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const INGRESS_ASSIGNMENT_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const PLAYER_ACTION_FRAME_SCHEMA_VERSION: u16 = 1;
pub const INPUT_SAMPLE_SCHEMA_VERSION: u16 = 1;
pub const INGRESS_ASSIGNMENT_SCHEMA_VERSION: u16 = 1;
pub const CLOSED_INGRESS_BATCH_SCHEMA_VERSION: u16 = 1;
pub const CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION: u16 = 2;
pub const PLAYER_CONTROLLER_REGISTRY_SCHEMA_VERSION: u16 = 1;
pub const INPUT_MAPPING_RECEIPT_SCHEMA_VERSION: u16 = 2;
pub const INGRESS_CHECKPOINT_SCHEMA_VERSION: u16 = 1;
pub const ACTION_MAP_MANIFEST_SCHEMA_VERSION: u16 = 1;
pub const INPUT_CONTEXT_SCHEMA_VERSION: u16 = 1;
pub const INPUT_CONTEXT_STACK_SCHEMA_VERSION: u16 = 1;

pub const PLAYER_ACTION_FRAME_SCHEMA_ID: &str = "nextengine.player-action-frame";
pub const PLAYER_ACTION_SOURCE_CLASS: &str = "nextengine.input.player-action";
pub const CORE_MOVE_ACTION_ID: &str = "nextengine.action.move";
pub const CORE_INTERACT_ACTION_ID: &str = "nextengine.action.interact";
pub const CORE_PICKUP_ACTION_ID: &str = "nextengine.action.pickup";
pub const CORE_EQUIP_USE_ACTION_ID: &str = "nextengine.action.equip-use";
pub const CORE_MELEE_ACTION_ID: &str = "nextengine.action.melee";
pub const CORE_CAMERA_ORBIT_ACTION_ID: &str = "nextengine.action.camera-orbit";
pub const CORE_UI_NAVIGATE_ACTION_ID: &str = "nextengine.action.ui-nav";
pub const CORE_UI_CONFIRM_ACTION_ID: &str = "nextengine.action.ui-confirm";
pub const CORE_UI_BACK_ACTION_ID: &str = "nextengine.action.ui-back";
pub const CORE_UI_INVENTORY_ACTION_ID: &str = "nextengine.action.ui-inventory";
pub const CORE_UI_JOURNAL_ACTION_ID: &str = "nextengine.action.ui-journal";
pub const CORE_KEYBOARD_MOUSE_ACTION_MAP_ID: &str = "nextengine.action-map.core-keyboard-mouse";
pub const CORE_GENERIC_CONTROLLER_ACTION_MAP_ID: &str =
    "nextengine.action-map.core-keyboard-mouse-controller";
pub const CORE_GAMEPLAY_CONTEXT_ID: &str = "nextengine.input-context.gameplay";
pub const CORE_GAMEPLAY_CONTEXT_STACK_ID: &str = "nextengine.input-context-stack.gameplay";
pub const CORE_UI_MENU_CONTEXT_ID: &str = "nextengine.input-context.ui-menu";
pub const CORE_UI_MENU_CONTEXT_STACK_ID: &str = "nextengine.input-context-stack.ui-menu";
pub const CORE_UI_DIALOGUE_CONTEXT_ID: &str = "nextengine.input-context.ui-dialogue";
pub const CORE_UI_DIALOGUE_CONTEXT_STACK_ID: &str = "nextengine.input-context-stack.ui-dialogue";
pub const KEYBOARD_DEVICE_CLASS_ID: &str = "nextengine.input.keyboard";
pub const MOUSE_DEVICE_CLASS_ID: &str = "nextengine.input.mouse";
pub const GENERIC_CONTROLLER_DEVICE_CLASS_ID: &str = "nextengine.input.controller.generic";
pub const KEYBOARD_W_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.w";
pub const KEYBOARD_A_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.a";
pub const KEYBOARD_S_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.s";
pub const KEYBOARD_D_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.d";
pub const KEYBOARD_E_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.e";
pub const KEYBOARD_Q_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.q";
pub const KEYBOARD_R_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.r";
pub const KEYBOARD_F_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.f";
pub const KEYBOARD_I_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.i";
pub const KEYBOARD_J_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.j";
pub const KEYBOARD_UP_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.up";
pub const KEYBOARD_DOWN_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.down";
pub const KEYBOARD_LEFT_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.left";
pub const KEYBOARD_RIGHT_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.right";
pub const KEYBOARD_RETURN_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.return";
pub const KEYBOARD_ESCAPE_CONTROL_PATH_ID: &str = "nextengine.input.keyboard.escape";
pub const MOUSE_DELTA_CONTROL_PATH_ID: &str = "nextengine.input.mouse.delta";
pub const CONTROLLER_LEFT_STICK_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.left-stick";
pub const CONTROLLER_RIGHT_STICK_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.right-stick";
pub const CONTROLLER_DPAD_CONTROL_PATH_ID: &str = "nextengine.input.controller.generic.dpad";
pub const CONTROLLER_LEFT_STICK_CLICK_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.left-stick-click";
pub const CONTROLLER_RIGHT_STICK_CLICK_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.right-stick-click";
pub const CONTROLLER_BUTTON_SOUTH_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.button-south";
pub const CONTROLLER_BUTTON_EAST_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.button-east";
pub const CONTROLLER_BUTTON_WEST_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.button-west";
pub const CONTROLLER_BUTTON_NORTH_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.button-north";
pub const CONTROLLER_LEFT_SHOULDER_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.left-shoulder";
pub const CONTROLLER_RIGHT_SHOULDER_CONTROL_PATH_ID: &str =
    "nextengine.input.controller.generic.right-shoulder";
pub const PLAYER_INTERACTION_SYSTEM_ID: &str = "nextengine.system.player-interaction";
pub const MAX_PLAYER_ACTIONS_PER_FRAME: usize = 64;
pub const MAX_ACTION_MAP_ACTIONS: usize = 64;
pub const MAX_ACTION_BINDINGS_PER_ACTION: usize = 32;
pub const MAX_ACTION_MAP_BINDINGS: usize = 256;
pub const MAX_ACTION_MAP_DEVICE_CLASSES: usize = 16;
pub const MAX_ACTION_BINDING_MODIFIERS: usize = 16;
pub const MAX_INPUT_CONTEXTS: usize = 32;
pub const MAX_INPUT_CONTEXT_ACTIONS: usize = 64;
pub const MAX_INPUT_IDENTIFIER_BYTES: usize = 255;
pub const MAX_DERIVED_COMMANDS_PER_INPUT_FRAME: usize = 4_096;

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
pub(super) const ACTION_MAP_MANIFEST_SCHEMA_ID: &str = "nextengine.action-map-manifest";
pub(super) const INPUT_CONTEXT_STACK_SCHEMA_ID: &str = "nextengine.input-context-stack";
pub(super) const SEGMENT_V1: &str = "v1";
pub(super) const SEGMENT_V2: &str = "v2";
