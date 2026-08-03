use std::collections::BTreeSet;

use crate::canonical::*;
use crate::ids::*;

use super::actions::PlayerActionPhaseV1;
use super::codec::*;
use super::constants::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PlayerActionValueKindV1 {
    Digital = 1,
    ScalarQ15 = 2,
    Vector2Q15 = 3,
}

impl PlayerActionValueKindV1 {
    fn from_tag(tag: u8) -> Result<Self, InputContractError> {
        match tag {
            1 => Ok(Self::Digital),
            2 => Ok(Self::ScalarQ15),
            3 => Ok(Self::Vector2Q15),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActionConflictPolicyV1 {
    Reject = 1,
    PreferCanonicalBinding = 2,
    CombineVectorQ15 = 3,
}

impl ActionConflictPolicyV1 {
    fn from_tag(tag: u8) -> Result<Self, InputContractError> {
        match tag {
            1 => Ok(Self::Reject),
            2 => Ok(Self::PreferCanonicalBinding),
            3 => Ok(Self::CombineVectorQ15),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActionBindingTransformV1 {
    Digital { pressed_threshold_q15: i16 },
    Vector2ContributionQ15 { contribution_q15: [i16; 2] },
    Vector2PassthroughQ15 { scale_q15: i16 },
    ScalarPassthroughQ15 { scale_q15: i16 },
}

impl ActionBindingTransformV1 {
    fn canonical_payload(self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, nested_type, payload) = match self {
            Self::Digital {
                pressed_threshold_q15,
            } => (
                1,
                CANONICAL_TYPE_I16,
                pressed_threshold_q15.to_le_bytes().to_vec(),
            ),
            Self::Vector2ContributionQ15 { contribution_q15 } => {
                let mut payload = Vec::with_capacity(4);
                payload.extend_from_slice(&contribution_q15[0].to_le_bytes());
                payload.extend_from_slice(&contribution_q15[1].to_le_bytes());
                (2, CANONICAL_TYPE_BYTES, payload)
            }
            Self::Vector2PassthroughQ15 { scale_q15 } => {
                (3, CANONICAL_TYPE_I16, scale_q15.to_le_bytes().to_vec())
            }
            Self::ScalarPassthroughQ15 { scale_q15 } => {
                (4, CANONICAL_TYPE_I16, scale_q15.to_le_bytes().to_vec())
            }
        };
        let mut bytes = vec![tag];
        bytes.extend_from_slice(&encode_nested(nested_type, &payload)?);
        Ok(bytes)
    }

    fn from_canonical_payload(
        payload: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let mut cursor = CanonicalCursor::new(payload);
        let tag = cursor.read_u8()?;
        let (nested_type, nested) = read_nested(&mut cursor, limits)?;
        cursor.finish()?;
        match (tag, nested_type) {
            (1, CANONICAL_TYPE_I16) => Ok(Self::Digital {
                pressed_threshold_q15: i16::from_le_bytes(exact(nested)?),
            }),
            (2, CANONICAL_TYPE_BYTES) if nested.len() == 4 => Ok(Self::Vector2ContributionQ15 {
                contribution_q15: [
                    i16::from_le_bytes(exact(&nested[..2])?),
                    i16::from_le_bytes(exact(&nested[2..])?),
                ],
            }),
            (3, CANONICAL_TYPE_I16) => Ok(Self::Vector2PassthroughQ15 {
                scale_q15: i16::from_le_bytes(exact(nested)?),
            }),
            (4, CANONICAL_TYPE_I16) => Ok(Self::ScalarPassthroughQ15 {
                scale_q15: i16::from_le_bytes(exact(nested)?),
            }),
            (1..=4, _) => Err(InputContractError::FieldType),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }

    fn validate(self) -> Result<(), InputContractError> {
        let valid = match self {
            Self::Digital {
                pressed_threshold_q15,
            } => pressed_threshold_q15 > 0,
            Self::Vector2ContributionQ15 { contribution_q15 } => contribution_q15 != [0, 0],
            Self::Vector2PassthroughQ15 { scale_q15 }
            | Self::ScalarPassthroughQ15 { scale_q15 } => scale_q15 > 0,
        };
        if valid {
            Ok(())
        } else {
            Err(InputContractError::InvalidProfile)
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActionBindingV1 {
    pub binding_slot_id: SchemaId,
    pub device_class: SchemaId,
    pub control_path_id: SchemaId,
    pub required_modifiers: Vec<SchemaId>,
    pub transform: ActionBindingTransformV1,
}

impl ActionBindingV1 {
    pub fn new(
        binding_slot_id: SchemaId,
        device_class: SchemaId,
        control_path_id: SchemaId,
        mut required_modifiers: Vec<SchemaId>,
        transform: ActionBindingTransformV1,
    ) -> Result<Self, InputContractError> {
        required_modifiers.sort();
        let value = Self {
            binding_slot_id,
            device_class,
            control_path_id,
            required_modifiers,
            transform,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), InputContractError> {
        validate_id(&self.binding_slot_id)?;
        validate_id(&self.device_class)?;
        validate_id(&self.control_path_id)?;
        if self.required_modifiers.len() > MAX_ACTION_BINDING_MODIFIERS {
            return Err(InputContractError::ResourceLimit);
        }
        validate_strict_ids(&self.required_modifiers)?;
        self.transform.validate()
    }

    fn resolution_key(&self) -> (&SchemaId, &SchemaId) {
        (&self.device_class, &self.control_path_id)
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_utf8(1, &self.binding_slot_id),
            field_utf8(2, &self.device_class),
            field_utf8(3, &self.control_path_id),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_SEQUENCE,
                encode_id_sequence(&self.required_modifiers)?,
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_TAGGED_UNION,
                self.transform.canonical_payload()?,
            ),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_UTF8_NFC),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_UTF8_NFC),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_TAGGED_UNION),
            ],
        )?;
        let value = Self {
            binding_slot_id: SchemaId::new(read_utf8_field(&fields, 1)?)?,
            device_class: SchemaId::new(read_utf8_field(&fields, 2)?)?,
            control_path_id: SchemaId::new(read_utf8_field(&fields, 3)?)?,
            required_modifiers: decode_id_sequence(&field_from(&fields, 4)?.payload, limits)?,
            transform: ActionBindingTransformV1::from_canonical_payload(
                &field_from(&fields, 5)?.payload,
                limits,
            )?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActionAccessibilityV1 {
    pub semantic_role_id: SchemaId,
    pub supports_hold: bool,
    pub supports_toggle: bool,
}

impl ActionAccessibilityV1 {
    fn validate(&self) -> Result<(), InputContractError> {
        validate_id(&self.semantic_role_id)
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_utf8(1, &self.semantic_role_id),
            CanonicalField::new(2, CANONICAL_TYPE_BOOL, vec![u8::from(self.supports_hold)]),
            CanonicalField::new(3, CANONICAL_TYPE_BOOL, vec![u8::from(self.supports_toggle)]),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_UTF8_NFC),
                (2, CANONICAL_TYPE_BOOL),
                (3, CANONICAL_TYPE_BOOL),
            ],
        )?;
        let value = Self {
            semantic_role_id: SchemaId::new(read_utf8_field(&fields, 1)?)?,
            supports_hold: read_bool_field(&fields, 2)?,
            supports_toggle: read_bool_field(&fields, 3)?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionDefinitionV1 {
    pub action_id: SchemaId,
    pub value_kind: PlayerActionValueKindV1,
    pub allowed_phases: Vec<PlayerActionPhaseV1>,
    pub binding_slots: Vec<ActionBindingV1>,
    pub allowed_context_ids: Vec<SchemaId>,
    pub conflict_policy: ActionConflictPolicyV1,
    pub accessibility: ActionAccessibilityV1,
}

impl ActionDefinitionV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the manifest constructor keeps every public action policy explicit"
    )]
    pub fn new(
        action_id: SchemaId,
        value_kind: PlayerActionValueKindV1,
        mut allowed_phases: Vec<PlayerActionPhaseV1>,
        mut binding_slots: Vec<ActionBindingV1>,
        mut allowed_context_ids: Vec<SchemaId>,
        conflict_policy: ActionConflictPolicyV1,
        accessibility: ActionAccessibilityV1,
    ) -> Result<Self, InputContractError> {
        allowed_phases.sort();
        binding_slots.sort();
        allowed_context_ids.sort();
        let value = Self {
            action_id,
            value_kind,
            allowed_phases,
            binding_slots,
            allowed_context_ids,
            conflict_policy,
            accessibility,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), InputContractError> {
        validate_id(&self.action_id)?;
        if self.binding_slots.len() > MAX_ACTION_BINDINGS_PER_ACTION
            || self.allowed_context_ids.len() > MAX_INPUT_CONTEXTS
        {
            return Err(InputContractError::ResourceLimit);
        }
        if self.allowed_phases.is_empty() || self.binding_slots.is_empty() {
            return Err(InputContractError::InvalidProfile);
        }
        validate_strict(&self.allowed_phases)?;
        validate_strict(&self.binding_slots)?;
        validate_strict_ids(&self.allowed_context_ids)?;
        for binding in &self.binding_slots {
            binding.validate()?;
            let kind_matches = matches!(
                (self.value_kind, binding.transform),
                (
                    PlayerActionValueKindV1::Digital,
                    ActionBindingTransformV1::Digital { .. }
                ) | (
                    PlayerActionValueKindV1::Vector2Q15,
                    ActionBindingTransformV1::Vector2ContributionQ15 { .. }
                        | ActionBindingTransformV1::Vector2PassthroughQ15 { .. }
                ) | (
                    PlayerActionValueKindV1::ScalarQ15,
                    ActionBindingTransformV1::ScalarPassthroughQ15 { .. }
                )
            );
            if !kind_matches {
                return Err(InputContractError::InvalidProfile);
            }
        }
        let has_stateful_binding = self.binding_slots.iter().any(|binding| {
            !matches!(
                binding.transform,
                ActionBindingTransformV1::Vector2PassthroughQ15 { .. }
            )
        });
        if has_stateful_binding
            && [
                PlayerActionPhaseV1::Started,
                PlayerActionPhaseV1::Completed,
                PlayerActionPhaseV1::Cancelled,
            ]
            .into_iter()
            .any(|phase| self.allowed_phases.binary_search(&phase).is_err())
        {
            return Err(InputContractError::InvalidProfile);
        }
        if self
            .binding_slots
            .windows(2)
            .any(|pair| pair[0].binding_slot_id == pair[1].binding_slot_id)
        {
            return Err(InputContractError::DuplicateKey);
        }
        let conflict_matches = match self.value_kind {
            PlayerActionValueKindV1::Digital => !matches!(
                self.conflict_policy,
                ActionConflictPolicyV1::CombineVectorQ15
            ),
            PlayerActionValueKindV1::Vector2Q15 => true,
            PlayerActionValueKindV1::ScalarQ15 => !matches!(
                self.conflict_policy,
                ActionConflictPolicyV1::CombineVectorQ15
            ),
        };
        if !conflict_matches {
            return Err(InputContractError::InvalidProfile);
        }
        self.accessibility.validate()
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_utf8(1, &self.action_id),
            field_u8(2, self.value_kind as u8),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(
                    self.allowed_phases
                        .iter()
                        .map(|phase| vec![*phase as u8])
                        .collect(),
                )?,
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(
                    self.binding_slots
                        .iter()
                        .map(ActionBindingV1::canonical_record)
                        .collect::<Result<Vec<_>, _>>()?,
                )?,
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_SEQUENCE,
                encode_id_sequence(&self.allowed_context_ids)?,
            ),
            field_u8(6, self.conflict_policy as u8),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_STRUCT,
                self.accessibility.canonical_record()?,
            ),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_UTF8_NFC),
                (2, CANONICAL_TYPE_U8),
                (3, CANONICAL_TYPE_SEQUENCE),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_SEQUENCE),
                (6, CANONICAL_TYPE_U8),
                (7, CANONICAL_TYPE_STRUCT),
            ],
        )?;
        let allowed_phases = decode_sequence(&field_from(&fields, 3)?.payload, limits)?
            .into_iter()
            .map(|bytes| {
                if bytes.len() != 1 {
                    return Err(InputContractError::FieldLength);
                }
                PlayerActionPhaseV1::from_tag(bytes[0])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let binding_slots = decode_sequence(&field_from(&fields, 4)?.payload, limits)?
            .into_iter()
            .map(|bytes| ActionBindingV1::from_record(&bytes, limits))
            .collect::<Result<Vec<_>, _>>()?;
        let value = Self {
            action_id: SchemaId::new(read_utf8_field(&fields, 1)?)?,
            value_kind: PlayerActionValueKindV1::from_tag(read_u8_fields(&fields, 2)?)?,
            allowed_phases,
            binding_slots,
            allowed_context_ids: decode_id_sequence(&field_from(&fields, 5)?.payload, limits)?,
            conflict_policy: ActionConflictPolicyV1::from_tag(read_u8_fields(&fields, 6)?)?,
            accessibility: ActionAccessibilityV1::from_record(
                &field_from(&fields, 7)?.payload,
                limits,
            )?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionMapManifestV1 {
    pub schema_version: u16,
    pub action_map_id: SchemaId,
    pub revision: u64,
    pub supported_device_classes: Vec<SchemaId>,
    pub actions: Vec<ActionDefinitionV1>,
    pub content_hash: ContentHash,
}

impl ActionMapManifestV1 {
    pub fn new(
        action_map_id: SchemaId,
        revision: u64,
        mut supported_device_classes: Vec<SchemaId>,
        mut actions: Vec<ActionDefinitionV1>,
    ) -> Result<Self, InputContractError> {
        supported_device_classes.sort();
        actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
        let mut value = Self {
            schema_version: ACTION_MAP_MANIFEST_SCHEMA_VERSION,
            action_map_id,
            revision,
            supported_device_classes,
            actions,
            content_hash: ContentHash::default(),
        };
        value.validate_body()?;
        value.content_hash = value.computed_hash()?;
        Ok(value)
    }

    pub fn core_keyboard_mouse_v1() -> Result<Self, InputContractError> {
        let gameplay = SchemaId::new(CORE_GAMEPLAY_CONTEXT_ID)?;
        let ui_menu = SchemaId::new(CORE_UI_MENU_CONTEXT_ID)?;
        let ui_dialogue = SchemaId::new(CORE_UI_DIALOGUE_CONTEXT_ID)?;
        let keyboard = SchemaId::new(KEYBOARD_DEVICE_CLASS_ID)?;
        let mouse = SchemaId::new(MOUSE_DEVICE_CLASS_ID)?;

        let digital_action = |action_id: &str,
                              role_id: &str,
                              slot_id: &str,
                              control_path: &str,
                              contexts: Vec<SchemaId>|
         -> Result<ActionDefinitionV1, InputContractError> {
            ActionDefinitionV1::new(
                SchemaId::new(action_id)?,
                PlayerActionValueKindV1::Digital,
                vec![
                    PlayerActionPhaseV1::Started,
                    PlayerActionPhaseV1::Completed,
                    PlayerActionPhaseV1::Cancelled,
                ],
                vec![ActionBindingV1::new(
                    SchemaId::new(slot_id)?,
                    keyboard.clone(),
                    SchemaId::new(control_path)?,
                    Vec::new(),
                    ActionBindingTransformV1::Digital {
                        pressed_threshold_q15: 1,
                    },
                )?],
                contexts,
                ActionConflictPolicyV1::PreferCanonicalBinding,
                ActionAccessibilityV1 {
                    semantic_role_id: SchemaId::new(role_id)?,
                    supports_hold: true,
                    supports_toggle: true,
                },
            )
        };

        let movement_bindings = [
            (
                "nextengine.binding.move.a",
                KEYBOARD_A_CONTROL_PATH_ID,
                [-i16::MAX, 0],
            ),
            (
                "nextengine.binding.move.d",
                KEYBOARD_D_CONTROL_PATH_ID,
                [i16::MAX, 0],
            ),
            (
                "nextengine.binding.move.s",
                KEYBOARD_S_CONTROL_PATH_ID,
                [0, -i16::MAX],
            ),
            (
                "nextengine.binding.move.w",
                KEYBOARD_W_CONTROL_PATH_ID,
                [0, i16::MAX],
            ),
        ]
        .into_iter()
        .map(|(slot, path, contribution)| {
            ActionBindingV1::new(
                SchemaId::new(slot)?,
                keyboard.clone(),
                SchemaId::new(path)?,
                Vec::new(),
                ActionBindingTransformV1::Vector2ContributionQ15 {
                    contribution_q15: contribution,
                },
            )
        })
        .collect::<Result<Vec<_>, InputContractError>>()?;

        let ui_navigation_bindings = [
            (
                "nextengine.binding.ui-nav.left",
                KEYBOARD_LEFT_CONTROL_PATH_ID,
                [-i16::MAX, 0],
            ),
            (
                "nextengine.binding.ui-nav.right",
                KEYBOARD_RIGHT_CONTROL_PATH_ID,
                [i16::MAX, 0],
            ),
            (
                "nextengine.binding.ui-nav.down",
                KEYBOARD_DOWN_CONTROL_PATH_ID,
                [0, -i16::MAX],
            ),
            (
                "nextengine.binding.ui-nav.up",
                KEYBOARD_UP_CONTROL_PATH_ID,
                [0, i16::MAX],
            ),
        ]
        .into_iter()
        .map(|(slot, path, contribution)| {
            ActionBindingV1::new(
                SchemaId::new(slot)?,
                keyboard.clone(),
                SchemaId::new(path)?,
                Vec::new(),
                ActionBindingTransformV1::Vector2ContributionQ15 {
                    contribution_q15: contribution,
                },
            )
        })
        .collect::<Result<Vec<_>, InputContractError>>()?;

        Self::new(
            SchemaId::new(CORE_KEYBOARD_MOUSE_ACTION_MAP_ID)?,
            1,
            vec![keyboard.clone(), mouse.clone()],
            vec![
                ActionDefinitionV1::new(
                    SchemaId::new(CORE_MOVE_ACTION_ID)?,
                    PlayerActionValueKindV1::Vector2Q15,
                    vec![
                        PlayerActionPhaseV1::Started,
                        PlayerActionPhaseV1::Performed,
                        PlayerActionPhaseV1::Completed,
                        PlayerActionPhaseV1::Cancelled,
                    ],
                    movement_bindings,
                    vec![gameplay.clone()],
                    ActionConflictPolicyV1::PreferCanonicalBinding,
                    ActionAccessibilityV1 {
                        semantic_role_id: SchemaId::new(
                            "nextengine.accessibility-role.directional-movement",
                        )?,
                        supports_hold: true,
                        supports_toggle: false,
                    },
                )?,
                digital_action(
                    CORE_INTERACT_ACTION_ID,
                    "nextengine.accessibility-role.primary-interaction",
                    "nextengine.binding.interact.e",
                    KEYBOARD_E_CONTROL_PATH_ID,
                    vec![gameplay.clone()],
                )?,
                digital_action(
                    CORE_PICKUP_ACTION_ID,
                    "nextengine.accessibility-role.pickup",
                    "nextengine.binding.pickup.q",
                    KEYBOARD_Q_CONTROL_PATH_ID,
                    vec![gameplay.clone()],
                )?,
                digital_action(
                    CORE_EQUIP_USE_ACTION_ID,
                    "nextengine.accessibility-role.equip-use",
                    "nextengine.binding.equip-use.r",
                    KEYBOARD_R_CONTROL_PATH_ID,
                    vec![gameplay.clone()],
                )?,
                digital_action(
                    CORE_MELEE_ACTION_ID,
                    "nextengine.accessibility-role.melee",
                    "nextengine.binding.melee.f",
                    KEYBOARD_F_CONTROL_PATH_ID,
                    vec![gameplay.clone()],
                )?,
                ActionDefinitionV1::new(
                    SchemaId::new(CORE_CAMERA_ORBIT_ACTION_ID)?,
                    PlayerActionValueKindV1::Vector2Q15,
                    vec![
                        PlayerActionPhaseV1::Performed,
                        PlayerActionPhaseV1::Cancelled,
                    ],
                    vec![ActionBindingV1::new(
                        SchemaId::new("nextengine.binding.camera-orbit.mouse-delta")?,
                        mouse,
                        SchemaId::new(MOUSE_DELTA_CONTROL_PATH_ID)?,
                        Vec::new(),
                        ActionBindingTransformV1::Vector2PassthroughQ15 {
                            scale_q15: i16::MAX,
                        },
                    )?],
                    vec![gameplay.clone()],
                    ActionConflictPolicyV1::CombineVectorQ15,
                    ActionAccessibilityV1 {
                        semantic_role_id: SchemaId::new(
                            "nextengine.accessibility-role.camera-orbit",
                        )?,
                        supports_hold: false,
                        supports_toggle: false,
                    },
                )?,
                ActionDefinitionV1::new(
                    SchemaId::new(CORE_UI_NAVIGATE_ACTION_ID)?,
                    PlayerActionValueKindV1::Vector2Q15,
                    vec![
                        PlayerActionPhaseV1::Started,
                        PlayerActionPhaseV1::Performed,
                        PlayerActionPhaseV1::Completed,
                        PlayerActionPhaseV1::Cancelled,
                    ],
                    ui_navigation_bindings,
                    vec![ui_menu.clone(), ui_dialogue.clone()],
                    ActionConflictPolicyV1::PreferCanonicalBinding,
                    ActionAccessibilityV1 {
                        semantic_role_id: SchemaId::new(
                            "nextengine.accessibility-role.ui-navigation",
                        )?,
                        supports_hold: true,
                        supports_toggle: false,
                    },
                )?,
                digital_action(
                    CORE_UI_CONFIRM_ACTION_ID,
                    "nextengine.accessibility-role.ui-confirm",
                    "nextengine.binding.ui-confirm.return",
                    KEYBOARD_RETURN_CONTROL_PATH_ID,
                    vec![ui_menu.clone(), ui_dialogue.clone()],
                )?,
                digital_action(
                    CORE_UI_INVENTORY_ACTION_ID,
                    "nextengine.accessibility-role.ui-inventory",
                    "nextengine.binding.ui-inventory.i",
                    KEYBOARD_I_CONTROL_PATH_ID,
                    vec![gameplay.clone()],
                )?,
                digital_action(
                    CORE_UI_JOURNAL_ACTION_ID,
                    "nextengine.accessibility-role.ui-journal",
                    "nextengine.binding.ui-journal.j",
                    KEYBOARD_J_CONTROL_PATH_ID,
                    vec![gameplay.clone()],
                )?,
                digital_action(
                    CORE_UI_BACK_ACTION_ID,
                    "nextengine.accessibility-role.ui-back",
                    "nextengine.binding.ui-back.escape",
                    KEYBOARD_ESCAPE_CONTROL_PATH_ID,
                    vec![gameplay, ui_menu, ui_dialogue],
                )?,
            ],
        )
    }

    pub fn validate(&self) -> Result<(), InputContractError> {
        self.validate_body()?;
        if self.computed_hash()? != self.content_hash {
            return Err(InputContractError::HashMismatch);
        }
        Ok(())
    }

    fn validate_body(&self) -> Result<(), InputContractError> {
        if self.schema_version != ACTION_MAP_MANIFEST_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "action map manifest",
                version: u32::from(self.schema_version),
            });
        }
        validate_id(&self.action_map_id)?;
        if self.actions.len() > MAX_ACTION_MAP_ACTIONS
            || self.supported_device_classes.len() > MAX_ACTION_MAP_DEVICE_CLASSES
        {
            return Err(InputContractError::ResourceLimit);
        }
        if self.revision == 0 || self.supported_device_classes.is_empty() || self.actions.is_empty()
        {
            return Err(InputContractError::InvalidProfile);
        }
        validate_strict_ids(&self.supported_device_classes)?;
        if self
            .actions
            .windows(2)
            .any(|pair| pair[0].action_id >= pair[1].action_id)
        {
            return Err(InputContractError::NonCanonicalOrder);
        }
        let mut total_bindings = 0usize;
        let mut resolution_keys = BTreeSet::new();
        for action in &self.actions {
            action.validate()?;
            total_bindings = total_bindings
                .checked_add(action.binding_slots.len())
                .ok_or(InputContractError::ResourceLimit)?;
            for binding in &action.binding_slots {
                if self
                    .supported_device_classes
                    .binary_search(&binding.device_class)
                    .is_err()
                {
                    return Err(InputContractError::InvalidProfile);
                }
                if !resolution_keys.insert(binding.resolution_key()) {
                    return Err(InputContractError::DuplicateKey);
                }
            }
        }
        if total_bindings > MAX_ACTION_MAP_BINDINGS {
            return Err(InputContractError::ResourceLimit);
        }
        Ok(())
    }

    pub fn action(&self, action_id: &SchemaId) -> Option<&ActionDefinitionV1> {
        self.actions
            .binary_search_by(|action| action.action_id.cmp(action_id))
            .ok()
            .map(|index| &self.actions[index])
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            INPUT_OWNER_ID,
            ACTION_MAP_MANIFEST_SCHEMA_ID,
            SEGMENT_V1,
            self.canonical_fields(true)?,
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            INPUT_OWNER_ID,
            ACTION_MAP_MANIFEST_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_SEQUENCE),
                (6, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let actions = decode_sequence(field(&segment, 5)?, limits)?
            .into_iter()
            .map(|record| ActionDefinitionV1::from_record(&record, limits))
            .collect::<Result<Vec<_>, _>>()?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            action_map_id: SchemaId::new(read_utf8(&segment, 2)?)?,
            revision: read_u64(&segment, 3)?,
            supported_device_classes: decode_id_sequence(field(&segment, 4)?, limits)?,
            actions,
            content_hash: read_hash(&segment, 6)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    fn computed_hash(&self) -> Result<ContentHash, CanonicalError> {
        let body = encode_canonical_segment(
            INPUT_OWNER_ID,
            ACTION_MAP_MANIFEST_SCHEMA_ID,
            "body-v1",
            self.canonical_fields(false)?,
        )?;
        hash_canonical_profile(&body)
    }

    fn canonical_fields(&self, include_hash: bool) -> Result<Vec<CanonicalField>, CanonicalError> {
        let mut fields = vec![
            field_u16(1, self.schema_version),
            field_utf8(2, &self.action_map_id),
            field_u64(3, self.revision),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_SEQUENCE,
                encode_id_sequence(&self.supported_device_classes)?,
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(
                    self.actions
                        .iter()
                        .map(ActionDefinitionV1::canonical_record)
                        .collect::<Result<Vec<_>, _>>()?,
                )?,
            ),
        ];
        if include_hash {
            fields.push(field_hash(6, self.content_hash));
        }
        Ok(fields)
    }
}

fn field_utf8(id: u32, value: &SchemaId) -> CanonicalField {
    CanonicalField::new(
        id,
        CANONICAL_TYPE_UTF8_NFC,
        value.as_str().as_bytes().to_vec(),
    )
}

pub(super) fn encode_id_sequence(values: &[SchemaId]) -> Result<Vec<u8>, CanonicalError> {
    encode_sequence(
        values
            .iter()
            .map(|value| value.as_str().as_bytes().to_vec())
            .collect(),
    )
}

pub(super) fn decode_id_sequence(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<SchemaId>, InputContractError> {
    decode_sequence(bytes, limits)?
        .into_iter()
        .map(|bytes| {
            let value = std::str::from_utf8(&bytes)
                .map_err(|_| InputContractError::Canonical(CanonicalDecodeError::InvalidUtf8))?;
            Ok(SchemaId::new(value)?)
        })
        .collect()
}

pub(super) fn validate_id(value: &SchemaId) -> Result<(), InputContractError> {
    if value.as_str().len() > MAX_INPUT_IDENTIFIER_BYTES {
        Err(InputContractError::ResourceLimit)
    } else {
        Ok(())
    }
}

pub(super) fn validate_strict<T: Ord>(values: &[T]) -> Result<(), InputContractError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        Err(InputContractError::NonCanonicalOrder)
    } else {
        Ok(())
    }
}

pub(super) fn validate_strict_ids(values: &[SchemaId]) -> Result<(), InputContractError> {
    if values.iter().any(|value| validate_id(value).is_err()) {
        return Err(InputContractError::ResourceLimit);
    }
    validate_strict(values)
}

fn read_bool_field(fields: &[CanonicalField], id: u32) -> Result<bool, InputContractError> {
    match field_from(fields, id)?.payload.as_slice() {
        [0] => Ok(false),
        [1] => Ok(true),
        _ => Err(InputContractError::InvalidValue),
    }
}
