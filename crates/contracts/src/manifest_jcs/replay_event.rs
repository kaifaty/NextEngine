use super::ManifestCodecError;
use super::jcs::{
    JcsValue, decode_fixed_hex, decode_hex, decode_i32_string, decode_i64_string, decode_u32,
    decode_u64_string, ensure_no_more, hex_bytes, into_array, into_string, next, string,
};
use crate::command::{CommandPhase, DomainEvent, EventPayload};
use crate::ids::{CommandId, PersistentId, SchemaId};
use crate::persistence::ManifestValidationError;
use crate::physics::{PhysicalEventV1, PhysicsPoseV1};
use crate::rpg::RpgEventV1;
use crate::rpg::SkillProficiency;

pub(super) fn encode_domain_event(event: &DomainEvent) -> Result<JcsValue, ManifestCodecError> {
    event.validate()?;
    Ok(JcsValue::Array(vec![
        JcsValue::Number(u64::from(event.phase as u8)),
        string(event.tick.to_string()),
        string(event.causal_command_id.to_hex()),
        JcsValue::Number(u64::from(event.event_slot)),
        encode_event_payload(&event.payload),
        string(hex_bytes(&event.canonical_bytes()?)),
    ]))
}

fn encode_event_payload(payload: &EventPayload) -> JcsValue {
    match payload {
        EventPayload::CommandCommitted { command_sequence } => JcsValue::Array(vec![
            string("command"),
            string(command_sequence.to_string()),
        ]),
        EventPayload::Rpg(RpgEventV1::DialogueAdvanced {
            dialogue_id,
            node_id,
        }) => JcsValue::Array(vec![
            string("rpg_dialogue"),
            string(dialogue_id.to_hex()),
            string(node_id.as_str()),
        ]),
        EventPayload::Rpg(RpgEventV1::QuestTransitioned { quest_id, state_id }) => {
            JcsValue::Array(vec![
                string("rpg_quest"),
                string(quest_id.to_hex()),
                string(state_id.as_str()),
            ])
        }
        EventPayload::Rpg(RpgEventV1::RelationshipAdjusted {
            relationship_id,
            dimension_id,
            value,
        }) => JcsValue::Array(vec![
            string("rpg_relationship"),
            string(relationship_id.to_hex()),
            string(dimension_id.as_str()),
            string(value.to_string()),
        ]),
        EventPayload::Rpg(RpgEventV1::ItemTransferred {
            item_id,
            source_inventory_id,
            destination_inventory_id,
            quantity,
        }) => JcsValue::Array(vec![
            string("rpg_item"),
            string(item_id.to_hex()),
            encode_optional_persistent_id(*source_inventory_id),
            encode_optional_persistent_id(*destination_inventory_id),
            JcsValue::Number(u64::from(*quantity)),
        ]),
        EventPayload::Rpg(RpgEventV1::SkillProficiencySet {
            character_id,
            skill_id,
            value,
        }) => JcsValue::Array(vec![
            string("rpg_skill"),
            string(character_id.to_hex()),
            string(skill_id.as_str()),
            JcsValue::Number(u64::from(value.get())),
        ]),
        EventPayload::Rpg(RpgEventV1::EquipmentAssigned {
            equipment_id,
            item_id,
            slot_id,
        }) => JcsValue::Array(vec![
            string("rpg_equipment"),
            string(equipment_id.to_hex()),
            string(item_id.to_hex()),
            string(slot_id.as_str()),
        ]),
        EventPayload::Rpg(RpgEventV1::InteractiveObjectTransitioned {
            object_id,
            state_id,
        }) => JcsValue::Array(vec![
            string("rpg_object"),
            string(object_id.to_hex()),
            string(state_id.as_str()),
        ]),
        EventPayload::Rpg(RpgEventV1::CharacterResourceAdjusted {
            character_id,
            resource_id,
            value,
        }) => JcsValue::Array(vec![
            string("rpg_character_resource"),
            string(character_id.to_hex()),
            string(resource_id.as_str()),
            string(value.to_string()),
        ]),
        EventPayload::Physical(PhysicalEventV1::CapsuleStepApplied {
            body_id,
            physics_tick,
            before,
            after,
        }) => JcsValue::Array(vec![
            string("physical_capsule_step"),
            string(body_id.to_hex()),
            string(physics_tick.to_string()),
            encode_pose(before),
            encode_pose(after),
        ]),
    }
}

pub(super) fn decode_domain_events(
    value: JcsValue,
) -> Result<Vec<DomainEvent>, ManifestCodecError> {
    into_array(value, "ticks[].expected_events")?
        .into_iter()
        .map(|row| {
            let mut columns = into_array(row, "ticks[].expected_events[]")?.into_iter();
            let phase = match decode_u32(
                next(&mut columns, "ticks[].expected_events[].phase")?,
                "ticks[].expected_events[].phase",
            )? {
                0 => CommandPhase::Ingress,
                1 => CommandPhase::Outcome,
                _ => {
                    return Err(ManifestCodecError::InvalidInteger(
                        "ticks[].expected_events[].phase".to_owned(),
                    ));
                }
            };
            let tick = decode_u64_string(
                next(&mut columns, "ticks[].expected_events[].tick")?,
                "ticks[].expected_events[].tick",
            )?;
            let command_id = CommandId::from_bytes(decode_fixed_hex::<16>(
                next(&mut columns, "ticks[].expected_events[].causal_command_id")?,
                "ticks[].expected_events[].causal_command_id",
            )?);
            let event_slot = decode_u32(
                next(&mut columns, "ticks[].expected_events[].event_slot")?,
                "ticks[].expected_events[].event_slot",
            )?;
            let payload =
                decode_event_payload(next(&mut columns, "ticks[].expected_events[].payload")?)?;
            let canonical_bytes = decode_hex(
                next(&mut columns, "ticks[].expected_events[].canonical_bytes")?,
                "ticks[].expected_events[].canonical_bytes",
            )?;
            ensure_no_more(columns, "ticks[].expected_events[]")?;
            let event = match payload {
                EventPayload::CommandCommitted { command_sequence } => {
                    DomainEvent::command_committed(tick, phase, command_id, command_sequence)?
                }
                EventPayload::Rpg(payload) => {
                    DomainEvent::rpg(tick, phase, command_id, event_slot, payload)?
                }
                EventPayload::Physical(payload) => {
                    DomainEvent::physical(tick, phase, command_id, event_slot, payload)?
                }
            };
            if event.event_slot != event_slot || event.canonical_bytes()? != canonical_bytes {
                return Err(ManifestCodecError::Validation(
                    ManifestValidationError::ReplayBatchMismatch,
                ));
            }
            Ok(event)
        })
        .collect()
}

fn decode_event_payload(value: JcsValue) -> Result<EventPayload, ManifestCodecError> {
    let mut columns = into_array(value, "ticks[].expected_events[].payload")?.into_iter();
    let tag = into_string(
        next(&mut columns, "ticks[].expected_events[].payload.tag")?,
        "ticks[].expected_events[].payload.tag",
    )?;
    let payload = match tag.as_str() {
        "command" => EventPayload::CommandCommitted {
            command_sequence: decode_u64_string(
                next(
                    &mut columns,
                    "ticks[].expected_events[].payload.command_sequence",
                )?,
                "ticks[].expected_events[].payload.command_sequence",
            )?,
        },
        "rpg_dialogue" => EventPayload::Rpg(RpgEventV1::DialogueAdvanced {
            dialogue_id: decode_persistent_id(next(&mut columns, "event.dialogue_id")?)?,
            node_id: decode_schema_id(next(&mut columns, "event.dialogue_node_id")?)?,
        }),
        "rpg_quest" => EventPayload::Rpg(RpgEventV1::QuestTransitioned {
            quest_id: decode_persistent_id(next(&mut columns, "event.quest_id")?)?,
            state_id: decode_schema_id(next(&mut columns, "event.quest_state_id")?)?,
        }),
        "rpg_relationship" => EventPayload::Rpg(RpgEventV1::RelationshipAdjusted {
            relationship_id: decode_persistent_id(next(&mut columns, "event.relationship_id")?)?,
            dimension_id: decode_schema_id(next(&mut columns, "event.dimension_id")?)?,
            value: decode_i32_string(
                next(&mut columns, "event.relationship_value")?,
                "event.relationship_value",
            )?,
        }),
        "rpg_item" => EventPayload::Rpg(RpgEventV1::ItemTransferred {
            item_id: decode_persistent_id(next(&mut columns, "event.item_id")?)?,
            source_inventory_id: decode_optional_persistent_id(next(
                &mut columns,
                "event.source_inventory_id",
            )?)?,
            destination_inventory_id: decode_optional_persistent_id(next(
                &mut columns,
                "event.destination_inventory_id",
            )?)?,
            quantity: decode_u32(next(&mut columns, "event.quantity")?, "event.quantity")?,
        }),
        "rpg_skill" => {
            let character_id = decode_persistent_id(next(&mut columns, "event.character_id")?)?;
            let skill_id = decode_schema_id(next(&mut columns, "event.skill_id")?)?;
            let raw = decode_u32(
                next(&mut columns, "event.proficiency")?,
                "event.proficiency",
            )?;
            let proficiency =
                SkillProficiency::new(u16::try_from(raw).map_err(|_| {
                    ManifestCodecError::InvalidInteger("event.proficiency".to_owned())
                })?)
                .map_err(|_| ManifestCodecError::InvalidInteger("event.proficiency".to_owned()))?;
            EventPayload::Rpg(RpgEventV1::SkillProficiencySet {
                character_id,
                skill_id,
                value: proficiency,
            })
        }
        "rpg_equipment" => EventPayload::Rpg(RpgEventV1::EquipmentAssigned {
            equipment_id: decode_persistent_id(next(&mut columns, "event.equipment_id")?)?,
            item_id: decode_persistent_id(next(&mut columns, "event.item_id")?)?,
            slot_id: decode_schema_id(next(&mut columns, "event.slot_id")?)?,
        }),
        "rpg_object" => EventPayload::Rpg(RpgEventV1::InteractiveObjectTransitioned {
            object_id: decode_persistent_id(next(&mut columns, "event.object_id")?)?,
            state_id: decode_schema_id(next(&mut columns, "event.state_id")?)?,
        }),
        "physical_capsule_step" => EventPayload::Physical(PhysicalEventV1::CapsuleStepApplied {
            body_id: decode_persistent_id(next(&mut columns, "event.body_id")?)?,
            physics_tick: decode_u64_string(
                next(&mut columns, "event.physics_tick")?,
                "event.physics_tick",
            )?,
            before: decode_pose(next(&mut columns, "event.before")?)?,
            after: decode_pose(next(&mut columns, "event.after")?)?,
        }),
        _ => {
            return Err(ManifestCodecError::UnknownField(format!(
                "ticks[].expected_events[].payload.{tag}"
            )));
        }
    };
    ensure_no_more(columns, "ticks[].expected_events[].payload")?;
    Ok(payload)
}

fn encode_pose(pose: &PhysicsPoseV1) -> JcsValue {
    JcsValue::Array(
        pose.translation_micrometres
            .iter()
            .map(|value| string(value.to_string()))
            .chain(
                pose.rotation_q1_30
                    .iter()
                    .map(|value| string(value.to_string())),
            )
            .collect(),
    )
}

fn decode_pose(value: JcsValue) -> Result<PhysicsPoseV1, ManifestCodecError> {
    let mut columns = into_array(value, "event.pose")?.into_iter();
    let pose = PhysicsPoseV1 {
        translation_micrometres: [
            decode_i64_string(
                next(&mut columns, "event.pose.translation_x")?,
                "event.pose",
            )?,
            decode_i64_string(
                next(&mut columns, "event.pose.translation_y")?,
                "event.pose",
            )?,
            decode_i64_string(
                next(&mut columns, "event.pose.translation_z")?,
                "event.pose",
            )?,
        ],
        rotation_q1_30: [
            decode_i32_string(next(&mut columns, "event.pose.rotation_x")?, "event.pose")?,
            decode_i32_string(next(&mut columns, "event.pose.rotation_y")?, "event.pose")?,
            decode_i32_string(next(&mut columns, "event.pose.rotation_z")?, "event.pose")?,
            decode_i32_string(next(&mut columns, "event.pose.rotation_w")?, "event.pose")?,
        ],
    };
    ensure_no_more(columns, "event.pose")?;
    pose.validate()
        .map_err(ManifestValidationError::from)
        .map_err(ManifestCodecError::from)?;
    Ok(pose)
}

fn encode_optional_persistent_id(value: Option<PersistentId>) -> JcsValue {
    match value {
        Some(value) => JcsValue::Array(vec![string(value.to_hex())]),
        None => JcsValue::Array(Vec::new()),
    }
}

fn decode_optional_persistent_id(
    value: JcsValue,
) -> Result<Option<PersistentId>, ManifestCodecError> {
    let values = into_array(value, "event.optional_persistent_id")?;
    match values.as_slice() {
        [] => Ok(None),
        [_] => Ok(Some(decode_persistent_id(
            values.into_iter().next().expect("single value"),
        )?)),
        _ => Err(ManifestCodecError::InvalidInteger(
            "event.optional_persistent_id".to_owned(),
        )),
    }
}

fn decode_persistent_id(value: JcsValue) -> Result<PersistentId, ManifestCodecError> {
    Ok(PersistentId::from_bytes(decode_fixed_hex::<16>(
        value,
        "event.persistent_id",
    )?))
}

fn decode_schema_id(value: JcsValue) -> Result<SchemaId, ManifestCodecError> {
    Ok(SchemaId::new(into_string(value, "event.schema_id")?)?)
}
