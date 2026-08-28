use super::ManifestCodecError;
use super::jcs::{
    JcsValue, decode_fixed_hex, decode_hex, decode_i32_string, decode_i64_string, decode_u32,
    decode_u64_string, ensure_no_more, hex_bytes, into_array, into_string, next, string,
};
use crate::cognition::{AgentDecisionCommittedV1, COGNITION_SCHEMA_VERSION};
use crate::command::{CommandPhase, DomainEvent, EventPayload};
use crate::ids::{CommandId, ContentHash, PersistentId, SchemaId};
use crate::persistence::ManifestValidationError;
use crate::physics::{PhysicalEventV1, PhysicsPoseV1};
use crate::rpg::SkillProficiency;
use crate::rpg::{CommitmentStateV1, RpgEventV1};
use crate::world_activity::{WorldActivityChangedV1, WorldActivityStateV1};
use crate::world_population::{PopulationTierV1, WorldPopulationChangedV1};
use crate::world_routine::{WorldRoutineActivityChangedV1, WorldRoutineActivityV1};

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
        EventPayload::Rpg(RpgEventV1::CommitmentTransitioned {
            commitment_id,
            state,
        }) => JcsValue::Array(vec![
            string("rpg_commitment"),
            string(commitment_id.to_hex()),
            JcsValue::Number(u64::from(*state as u8)),
        ]),
        EventPayload::Rpg(RpgEventV1::BodyConditionChanged {
            condition_id,
            character_id,
            impairment,
            recovery_stage,
            systemic_condition,
        }) => JcsValue::Array(vec![
            string("rpg_body_condition"),
            string(condition_id.to_hex()),
            string(character_id.to_hex()),
            JcsValue::Number(u64::from(*impairment as u8)),
            JcsValue::Number(u64::from(*recovery_stage as u8)),
            JcsValue::Number(u64::from(*systemic_condition as u8)),
        ]),
        EventPayload::Rpg(RpgEventV1::BodyTreatmentAdvanced {
            condition_id,
            character_id,
            channel,
            impairment,
            recovery_stage,
            systemic_condition,
        }) => JcsValue::Array(vec![
            string("rpg_body_treatment"),
            string(condition_id.to_hex()),
            string(character_id.to_hex()),
            JcsValue::Number(u64::from(*channel as u8)),
            JcsValue::Number(u64::from(*impairment as u8)),
            JcsValue::Number(u64::from(*recovery_stage as u8)),
            JcsValue::Number(u64::from(*systemic_condition as u8)),
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
        EventPayload::WorldRoutine(event) => JcsValue::Array(vec![
            string("world_routine_activity_changed"),
            string(event.subject_id.to_hex()),
            JcsValue::Number(u64::from(event.previous_activity as u8)),
            JcsValue::Number(u64::from(event.current_activity as u8)),
            string(event.boundary_world_tick.to_string()),
            string(event.record_revision.to_string()),
        ]),
        EventPayload::WorldPopulation(WorldPopulationChangedV1::TierTransitioned {
            subject_id,
            previous_tier,
            current_tier,
            transition_tick,
            record_revision,
        }) => JcsValue::Array(vec![
            string("world_population_tier_transitioned"),
            string(subject_id.to_hex()),
            JcsValue::Number(u64::from(*previous_tier as u8)),
            JcsValue::Number(u64::from(*current_tier as u8)),
            string(transition_tick.to_string()),
            string(record_revision.to_string()),
        ]),
        EventPayload::WorldPopulation(WorldPopulationChangedV1::AbstractTransferred {
            subject_id,
            source_region_id,
            source_node_id,
            target_region_id,
            target_node_id,
            route_plan_hash,
            record_revision,
        }) => JcsValue::Array(vec![
            string("world_population_abstract_transferred"),
            string(subject_id.to_hex()),
            string(source_region_id.as_str()),
            string(source_node_id.as_str()),
            string(target_region_id.as_str()),
            string(target_node_id.as_str()),
            string(route_plan_hash.to_hex()),
            string(record_revision.to_string()),
        ]),
        EventPayload::WorldActivity(event) => JcsValue::Array(vec![
            string("world_activity_changed"),
            string(event.subject_id.to_hex()),
            JcsValue::Number(u64::from(event.previous_state as u8)),
            JcsValue::Number(u64::from(event.current_state as u8)),
            string(event.boundary_tick.to_string()),
            string(event.record_revision.to_string()),
            string(event.evidence_hash.to_hex()),
        ]),
        EventPayload::AgentCognition(event) => JcsValue::Array(vec![
            string("agent_cognition_decision_committed"),
            string(event.subject_id.to_hex()),
            string(event.agent_revision.to_string()),
            string(event.memory_revision.to_string()),
            string(event.active_goal_id.as_str()),
            string(event.intent_id.to_hex()),
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
                EventPayload::WorldRoutine(payload) => {
                    DomainEvent::world_routine(tick, phase, command_id, event_slot, payload)?
                }
                EventPayload::WorldPopulation(payload) => {
                    DomainEvent::world_population(tick, phase, command_id, event_slot, payload)?
                }
                EventPayload::WorldActivity(payload) => {
                    DomainEvent::world_activity(tick, phase, command_id, event_slot, payload)?
                }
                EventPayload::AgentCognition(payload) => {
                    DomainEvent::agent_cognition(tick, phase, command_id, event_slot, payload)?
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
        "rpg_character_resource" => EventPayload::Rpg(RpgEventV1::CharacterResourceAdjusted {
            character_id: decode_persistent_id(next(&mut columns, "event.character_id")?)?,
            resource_id: decode_schema_id(next(&mut columns, "event.resource_id")?)?,
            value: decode_i32_string(
                next(&mut columns, "event.resource_value")?,
                "event.resource_value",
            )?,
        }),
        "rpg_commitment" => EventPayload::Rpg(RpgEventV1::CommitmentTransitioned {
            commitment_id: decode_persistent_id(next(&mut columns, "event.commitment_id")?)?,
            state: decode_commitment_state(
                next(&mut columns, "event.commitment_state")?,
                "event.commitment_state",
            )?,
        }),
        "rpg_body_condition" => EventPayload::Rpg(RpgEventV1::BodyConditionChanged {
            condition_id: decode_persistent_id(next(&mut columns, "event.condition_id")?)?,
            character_id: decode_persistent_id(next(&mut columns, "event.character_id")?)?,
            impairment: decode_body_impairment(
                next(&mut columns, "event.impairment")?,
                "event.impairment",
            )?,
            recovery_stage: decode_body_recovery_stage(
                next(&mut columns, "event.recovery_stage")?,
                "event.recovery_stage",
            )?,
            systemic_condition: decode_systemic_condition(
                next(&mut columns, "event.systemic_condition")?,
                "event.systemic_condition",
            )?,
        }),
        "rpg_body_treatment" => EventPayload::Rpg(RpgEventV1::BodyTreatmentAdvanced {
            condition_id: decode_persistent_id(next(&mut columns, "event.condition_id")?)?,
            character_id: decode_persistent_id(next(&mut columns, "event.character_id")?)?,
            channel: decode_body_treatment_channel(
                next(&mut columns, "event.treatment_channel")?,
                "event.treatment_channel",
            )?,
            impairment: decode_body_impairment(
                next(&mut columns, "event.impairment")?,
                "event.impairment",
            )?,
            recovery_stage: decode_body_recovery_stage(
                next(&mut columns, "event.recovery_stage")?,
                "event.recovery_stage",
            )?,
            systemic_condition: decode_systemic_condition(
                next(&mut columns, "event.systemic_condition")?,
                "event.systemic_condition",
            )?,
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
        "world_routine_activity_changed" => {
            EventPayload::WorldRoutine(WorldRoutineActivityChangedV1 {
                subject_id: decode_persistent_id(next(&mut columns, "event.subject_id")?)?,
                previous_activity: decode_world_routine_activity(
                    next(&mut columns, "event.previous_activity")?,
                    "event.previous_activity",
                )?,
                current_activity: decode_world_routine_activity(
                    next(&mut columns, "event.current_activity")?,
                    "event.current_activity",
                )?,
                boundary_world_tick: decode_u64_string(
                    next(&mut columns, "event.boundary_world_tick")?,
                    "event.boundary_world_tick",
                )?,
                record_revision: decode_u64_string(
                    next(&mut columns, "event.record_revision")?,
                    "event.record_revision",
                )?,
            })
        }
        "world_population_tier_transitioned" => {
            EventPayload::WorldPopulation(WorldPopulationChangedV1::TierTransitioned {
                subject_id: decode_persistent_id(next(&mut columns, "event.subject_id")?)?,
                previous_tier: decode_population_tier(
                    next(&mut columns, "event.previous_tier")?,
                    "event.previous_tier",
                )?,
                current_tier: decode_population_tier(
                    next(&mut columns, "event.current_tier")?,
                    "event.current_tier",
                )?,
                transition_tick: decode_u64_string(
                    next(&mut columns, "event.transition_tick")?,
                    "event.transition_tick",
                )?,
                record_revision: decode_u64_string(
                    next(&mut columns, "event.record_revision")?,
                    "event.record_revision",
                )?,
            })
        }
        "world_population_abstract_transferred" => {
            EventPayload::WorldPopulation(WorldPopulationChangedV1::AbstractTransferred {
                subject_id: decode_persistent_id(next(&mut columns, "event.subject_id")?)?,
                source_region_id: decode_schema_id(next(&mut columns, "event.source_region_id")?)?,
                source_node_id: decode_schema_id(next(&mut columns, "event.source_node_id")?)?,
                target_region_id: decode_schema_id(next(&mut columns, "event.target_region_id")?)?,
                target_node_id: decode_schema_id(next(&mut columns, "event.target_node_id")?)?,
                route_plan_hash: ContentHash::from_bytes(decode_fixed_hex::<32>(
                    next(&mut columns, "event.route_plan_hash")?,
                    "event.route_plan_hash",
                )?),
                record_revision: decode_u64_string(
                    next(&mut columns, "event.record_revision")?,
                    "event.record_revision",
                )?,
            })
        }
        "world_activity_changed" => EventPayload::WorldActivity(WorldActivityChangedV1 {
            subject_id: decode_persistent_id(next(&mut columns, "event.subject_id")?)?,
            previous_state: decode_world_activity_state(
                next(&mut columns, "event.previous_state")?,
                "event.previous_state",
            )?,
            current_state: decode_world_activity_state(
                next(&mut columns, "event.current_state")?,
                "event.current_state",
            )?,
            boundary_tick: decode_u64_string(
                next(&mut columns, "event.boundary_tick")?,
                "event.boundary_tick",
            )?,
            record_revision: decode_u64_string(
                next(&mut columns, "event.record_revision")?,
                "event.record_revision",
            )?,
            evidence_hash: ContentHash::from_bytes(decode_fixed_hex::<32>(
                next(&mut columns, "event.evidence_hash")?,
                "event.evidence_hash",
            )?),
        }),
        "agent_cognition_decision_committed" => {
            EventPayload::AgentCognition(AgentDecisionCommittedV1 {
                schema_version: COGNITION_SCHEMA_VERSION,
                subject_id: decode_persistent_id(next(&mut columns, "event.subject_id")?)?,
                agent_revision: decode_u64_string(
                    next(&mut columns, "event.agent_revision")?,
                    "event.agent_revision",
                )?,
                memory_revision: decode_u64_string(
                    next(&mut columns, "event.memory_revision")?,
                    "event.memory_revision",
                )?,
                active_goal_id: decode_schema_id(next(&mut columns, "event.active_goal_id")?)?,
                intent_id: ContentHash::from_bytes(decode_fixed_hex::<32>(
                    next(&mut columns, "event.intent_id")?,
                    "event.intent_id",
                )?),
            })
        }
        _ => {
            return Err(ManifestCodecError::UnknownField(format!(
                "ticks[].expected_events[].payload.{tag}"
            )));
        }
    };
    ensure_no_more(columns, "ticks[].expected_events[].payload")?;
    Ok(payload)
}

fn decode_world_routine_activity(
    value: JcsValue,
    path: &'static str,
) -> Result<WorldRoutineActivityV1, ManifestCodecError> {
    match decode_u32(value, path)? {
        1 => Ok(WorldRoutineActivityV1::Duty),
        2 => Ok(WorldRoutineActivityV1::Rest),
        _ => Err(ManifestCodecError::InvalidInteger(path.to_owned())),
    }
}

fn decode_population_tier(
    value: JcsValue,
    path: &'static str,
) -> Result<PopulationTierV1, ManifestCodecError> {
    match decode_u32(value, path)? {
        1 => Ok(PopulationTierV1::Dormant),
        2 => Ok(PopulationTierV1::Abstract),
        3 => Ok(PopulationTierV1::Simulated),
        4 => Ok(PopulationTierV1::Active),
        _ => Err(ManifestCodecError::InvalidInteger(path.to_owned())),
    }
}

fn decode_commitment_state(
    value: JcsValue,
    path: &'static str,
) -> Result<CommitmentStateV1, ManifestCodecError> {
    match decode_u32(value, path)? {
        1 => Ok(CommitmentStateV1::Offered),
        2 => Ok(CommitmentStateV1::Accepted),
        3 => Ok(CommitmentStateV1::Fulfilled),
        4 => Ok(CommitmentStateV1::Cancelled),
        _ => Err(ManifestCodecError::InvalidInteger(path.to_owned())),
    }
}

fn decode_body_impairment(
    value: JcsValue,
    path: &'static str,
) -> Result<crate::rpg::BodyImpairmentV1, ManifestCodecError> {
    match decode_u32(value, path)? {
        1 => Ok(crate::rpg::BodyImpairmentV1::Intact),
        2 => Ok(crate::rpg::BodyImpairmentV1::PartialKneeExtensor),
        3 => Ok(crate::rpg::BodyImpairmentV1::TendonTransmissionLost),
        4 => Ok(crate::rpg::BodyImpairmentV1::NerveControlLost),
        _ => Err(ManifestCodecError::InvalidInteger(path.to_owned())),
    }
}

fn decode_body_recovery_stage(
    value: JcsValue,
    path: &'static str,
) -> Result<crate::rpg::BodyRecoveryStageV1, ManifestCodecError> {
    match decode_u32(value, path)? {
        1 => Ok(crate::rpg::BodyRecoveryStageV1::Untreated),
        2 => Ok(crate::rpg::BodyRecoveryStageV1::Stabilized),
        3 => Ok(crate::rpg::BodyRecoveryStageV1::Repaired),
        4 => Ok(crate::rpg::BodyRecoveryStageV1::Rehabilitated),
        _ => Err(ManifestCodecError::InvalidInteger(path.to_owned())),
    }
}

fn decode_body_treatment_channel(
    value: JcsValue,
    path: &'static str,
) -> Result<crate::rpg::BodyTreatmentChannelV1, ManifestCodecError> {
    match decode_u32(value, path)? {
        1 => Ok(crate::rpg::BodyTreatmentChannelV1::Medical),
        2 => Ok(crate::rpg::BodyTreatmentChannelV1::Magical),
        _ => Err(ManifestCodecError::InvalidInteger(path.to_owned())),
    }
}

fn decode_systemic_condition(
    value: JcsValue,
    path: &'static str,
) -> Result<crate::rpg::SystemicConditionV1, ManifestCodecError> {
    match decode_u32(value, path)? {
        1 => Ok(crate::rpg::SystemicConditionV1::Stable),
        2 => Ok(crate::rpg::SystemicConditionV1::Impaired),
        _ => Err(ManifestCodecError::InvalidInteger(path.to_owned())),
    }
}

fn decode_world_activity_state(
    value: JcsValue,
    path: &'static str,
) -> Result<WorldActivityStateV1, ManifestCodecError> {
    match decode_u32(value, path)? {
        1 => Ok(WorldActivityStateV1::Unassigned),
        2 => Ok(WorldActivityStateV1::Assigned),
        3 => Ok(WorldActivityStateV1::Working),
        4 => Ok(WorldActivityStateV1::Completed),
        _ => Err(ManifestCodecError::InvalidInteger(path.to_owned())),
    }
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
