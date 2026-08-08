use crate::ids::SchemaId;

use super::super::codec::InputContractError;
use super::super::constants::*;
use super::{ActionBindingTransformV1, ActionBindingV1, ActionMapManifestV1};

impl ActionMapManifestV1 {
    /// Core first-party profile with keyboard/mouse fallback and a generic
    /// controller. Both device families bind the same semantic action IDs.
    pub fn core_keyboard_mouse_controller_v1() -> Result<Self, InputContractError> {
        let keyboard_mouse = Self::core_keyboard_mouse_v1()?;
        let controller = SchemaId::new(GENERIC_CONTROLLER_DEVICE_CLASS_ID)?;
        let mut actions = keyboard_mouse.actions;
        for action in &mut actions {
            let (slot_id, control_path_id, transform) = match action.action_id.as_str() {
                CORE_MOVE_ACTION_ID => (
                    "nextengine.binding.move.controller-left-stick",
                    CONTROLLER_LEFT_STICK_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Vector2PassthroughQ15 {
                        scale_q15: i16::MAX,
                    },
                ),
                CORE_CAMERA_ORBIT_ACTION_ID => (
                    "nextengine.binding.camera-orbit.controller-right-stick",
                    CONTROLLER_RIGHT_STICK_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Vector2PassthroughQ15 {
                        scale_q15: i16::MAX,
                    },
                ),
                CORE_UI_NAVIGATE_ACTION_ID => (
                    "nextengine.binding.ui-nav.controller-dpad",
                    CONTROLLER_DPAD_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Vector2PassthroughQ15 {
                        scale_q15: i16::MAX,
                    },
                ),
                CORE_INTERACT_ACTION_ID => (
                    "nextengine.binding.interact.controller-south",
                    CONTROLLER_BUTTON_SOUTH_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Digital {
                        pressed_threshold_q15: 1,
                    },
                ),
                CORE_PICKUP_ACTION_ID => (
                    "nextengine.binding.pickup.controller-west",
                    CONTROLLER_BUTTON_WEST_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Digital {
                        pressed_threshold_q15: 1,
                    },
                ),
                CORE_EQUIP_USE_ACTION_ID => (
                    "nextengine.binding.equip-use.controller-north",
                    CONTROLLER_BUTTON_NORTH_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Digital {
                        pressed_threshold_q15: 1,
                    },
                ),
                CORE_MELEE_ACTION_ID => (
                    "nextengine.binding.melee.controller-right-shoulder",
                    CONTROLLER_RIGHT_SHOULDER_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Digital {
                        pressed_threshold_q15: 1,
                    },
                ),
                CORE_UI_CONFIRM_ACTION_ID => (
                    "nextengine.binding.ui-confirm.controller-left-stick-click",
                    CONTROLLER_LEFT_STICK_CLICK_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Digital {
                        pressed_threshold_q15: 1,
                    },
                ),
                CORE_UI_BACK_ACTION_ID => (
                    "nextengine.binding.ui-back.controller-east",
                    CONTROLLER_BUTTON_EAST_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Digital {
                        pressed_threshold_q15: 1,
                    },
                ),
                CORE_UI_INVENTORY_ACTION_ID => (
                    "nextengine.binding.ui-inventory.controller-left-shoulder",
                    CONTROLLER_LEFT_SHOULDER_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Digital {
                        pressed_threshold_q15: 1,
                    },
                ),
                CORE_UI_JOURNAL_ACTION_ID => (
                    "nextengine.binding.ui-journal.controller-right-stick-click",
                    CONTROLLER_RIGHT_STICK_CLICK_CONTROL_PATH_ID,
                    ActionBindingTransformV1::Digital {
                        pressed_threshold_q15: 1,
                    },
                ),
                _ => return Err(InputContractError::InvalidProfile),
            };
            action.binding_slots.push(ActionBindingV1::new(
                SchemaId::new(slot_id)?,
                controller.clone(),
                SchemaId::new(control_path_id)?,
                Vec::new(),
                transform,
            )?);
            action.binding_slots.sort();
        }
        Self::new(
            SchemaId::new(CORE_GENERIC_CONTROLLER_ACTION_MAP_ID)?,
            1,
            vec![
                SchemaId::new(KEYBOARD_DEVICE_CLASS_ID)?,
                SchemaId::new(MOUSE_DEVICE_CLASS_ID)?,
                controller,
            ],
            actions,
        )
    }
}
