use crate::cognition::{
    COGNITION_MAX_SPEECH_ACTS, StructuredSpeechActV1, StructuredSpeechExchangeV1,
};

use super::{Reader, SystemicWorkProfileV1, WorldActivityContractError, Writer};

pub(super) fn write_systemic_work(
    writer: &mut Writer,
    value: &SystemicWorkProfileV1,
) -> Result<(), WorldActivityContractError> {
    writer.id(value.employer_character_id);
    writer.id(value.seller_character_id);
    writer.id(value.worker_inventory_id);
    writer.id(value.seller_inventory_id);
    writer.id(value.food_item_id);
    writer.schema_id(&value.currency_resource_id)?;
    writer.schema_id(&value.hunger_resource_id)?;
    writer.schema_id(&value.satiety_resource_id)?;
    writer.i32(value.wage_amount);
    writer.i32(value.food_price);
    writer.i32(value.hunger_restore_amount);
    writer.i32(value.satiety_gain_amount);
    writer.u32(value.listener_trust_q16);
    writer.schema_id(&value.social_action_id)?;
    writer.schema_id(&value.await_activity_action_id)?;
    writer.schema_id(&value.settlement_action_id)?;
    writer.schema_id(&value.social_ready_fact_id)?;
    writer.schema_id(&value.activity_ready_fact_id)?;
    writer.schema_id(&value.settlement_ready_fact_id)?;
    writer.u32(
        u32::try_from(value.work_exchange.acts.len())
            .map_err(|_| WorldActivityContractError::ContentInvalid)?,
    );
    for act in &value.work_exchange.acts {
        writer.bytes(
            &act.canonical_payload_bytes()
                .map_err(|_| WorldActivityContractError::ContentInvalid)?,
        )?;
    }
    writer.bytes(
        &value
            .threat_act
            .canonical_payload_bytes()
            .map_err(|_| WorldActivityContractError::ContentInvalid)?,
    )?;
    Ok(())
}

pub(super) fn read_systemic_work(
    reader: &mut Reader<'_>,
) -> Result<SystemicWorkProfileV1, WorldActivityContractError> {
    let employer_character_id = reader.id()?;
    let seller_character_id = reader.id()?;
    let worker_inventory_id = reader.id()?;
    let seller_inventory_id = reader.id()?;
    let food_item_id = reader.id()?;
    let currency_resource_id = reader.schema_id()?;
    let hunger_resource_id = reader.schema_id()?;
    let satiety_resource_id = reader.schema_id()?;
    let wage_amount = reader.i32()?;
    let food_price = reader.i32()?;
    let hunger_restore_amount = reader.i32()?;
    let satiety_gain_amount = reader.i32()?;
    let listener_trust_q16 = reader.u32()?;
    let social_action_id = reader.schema_id()?;
    let await_activity_action_id = reader.schema_id()?;
    let settlement_action_id = reader.schema_id()?;
    let social_ready_fact_id = reader.schema_id()?;
    let activity_ready_fact_id = reader.schema_id()?;
    let settlement_ready_fact_id = reader.schema_id()?;
    let act_count =
        usize::try_from(reader.u32()?).map_err(|_| WorldActivityContractError::ContentInvalid)?;
    if act_count == 0 || act_count > COGNITION_MAX_SPEECH_ACTS {
        return Err(WorldActivityContractError::ContentInvalid);
    }
    let acts = (0..act_count)
        .map(|_| {
            StructuredSpeechActV1::from_canonical_payload_bytes(reader.bytes()?, reader.limits)
                .map_err(|_| WorldActivityContractError::ContentInvalid)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let threat_act =
        StructuredSpeechActV1::from_canonical_payload_bytes(reader.bytes()?, reader.limits)
            .map_err(|_| WorldActivityContractError::ContentInvalid)?;
    Ok(SystemicWorkProfileV1 {
        employer_character_id,
        seller_character_id,
        worker_inventory_id,
        seller_inventory_id,
        food_item_id,
        currency_resource_id,
        hunger_resource_id,
        satiety_resource_id,
        wage_amount,
        food_price,
        hunger_restore_amount,
        satiety_gain_amount,
        listener_trust_q16,
        social_action_id,
        await_activity_action_id,
        settlement_action_id,
        social_ready_fact_id,
        activity_ready_fact_id,
        settlement_ready_fact_id,
        work_exchange: StructuredSpeechExchangeV1 { acts },
        threat_act,
    })
}
