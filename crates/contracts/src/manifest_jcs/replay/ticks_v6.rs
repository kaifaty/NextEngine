use super::{decode_command_records, decode_command_results, decode_world_streaming_input};
use crate::canonical::CanonicalDecodeLimits;
use crate::input::{
    ClosedCommandAdmissionBatchV2, ClosedIngressBatchV1, InputMappingReceiptV2,
    RuntimeAdmissionLimitsV1,
};
use crate::persistence::{ManifestValidationError, ReplayTickManifestV7};
use crate::physics::{
    ClosedPhysicsContactBatchV1, PhysicsQueryBatchV1, PhysicsQueryResultV1, PhysicsStepInputV2,
};
use crate::targeting::{AuthoritativeTargetingQueryV1, TargetingIntentV1};
use crate::world_routine::InteractionAvailabilityV1;

use super::super::ManifestCodecError;
use super::super::jcs::{JcsValue, decode_hex, decode_u64_string, into_array, into_object, take};
use super::super::replay_event::decode_domain_events;

pub(super) fn decode_replay_ticks_v6(
    value: JcsValue,
    limits: CanonicalDecodeLimits,
    admission: &RuntimeAdmissionLimitsV1,
) -> Result<Vec<ReplayTickManifestV7>, ManifestCodecError> {
    into_array(value, "ticks")?
        .into_iter()
        .map(|row| {
            let mut object = into_object(row, "ticks[]")?;
            let tick = decode_u64_string(take(&mut object, "tick")?, "ticks[].tick")?;
            let world_streaming_input =
                decode_world_streaming_input(take(&mut object, "world_streaming_input")?)?;
            let closed_ingress_batch = ClosedIngressBatchV1::from_canonical_bytes(
                &decode_hex(
                    take(&mut object, "closed_ingress_batch")?,
                    "ticks[].closed_ingress_batch",
                )?,
                limits,
                admission,
            )
            .map_err(ManifestValidationError::from)?;
            let direct_external_commands =
                decode_command_records(take(&mut object, "direct_external_commands")?)?;
            let expected_ingress_command_batch =
                ClosedCommandAdmissionBatchV2::from_canonical_bytes(
                    &decode_hex(
                        take(&mut object, "expected_ingress_command_batch")?,
                        "ticks[].expected_ingress_command_batch",
                    )?,
                    limits,
                    admission,
                )
                .map_err(ManifestValidationError::from)?;
            let expected_physics_step_input = PhysicsStepInputV2::from_canonical_bytes(
                &decode_hex(
                    take(&mut object, "expected_physics_step_input")?,
                    "ticks[].expected_physics_step_input",
                )?,
                limits,
            )
            .map_err(ManifestValidationError::from)?;
            let expected_contact_batch = ClosedPhysicsContactBatchV1::from_canonical_bytes(
                &decode_hex(
                    take(&mut object, "expected_contact_batch")?,
                    "ticks[].expected_contact_batch",
                )?,
                limits,
            )
            .map_err(ManifestValidationError::from)?;
            let expected_targeting_intents = into_array(
                take(&mut object, "expected_targeting_intents")?,
                "ticks[].expected_targeting_intents",
            )?
            .into_iter()
            .map(|value| {
                TargetingIntentV1::from_canonical_bytes(
                    &decode_hex(value, "ticks[].expected_targeting_intents[]")?,
                    limits,
                )
                .map_err(ManifestValidationError::from)
                .map_err(ManifestCodecError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
            let expected_authoritative_targeting_queries = into_array(
                take(&mut object, "expected_authoritative_targeting_queries")?,
                "ticks[].expected_authoritative_targeting_queries",
            )?
            .into_iter()
            .map(|value| {
                AuthoritativeTargetingQueryV1::from_canonical_bytes(
                    &decode_hex(value, "ticks[].expected_authoritative_targeting_queries[]")?,
                    limits,
                )
                .map_err(ManifestValidationError::from)
                .map_err(ManifestCodecError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
            let expected_physics_query_batch = PhysicsQueryBatchV1::from_canonical_bytes(
                &decode_hex(
                    take(&mut object, "expected_physics_query_batch")?,
                    "ticks[].expected_physics_query_batch",
                )?,
                limits,
            )
            .map_err(ManifestValidationError::from)?;
            let expected_physics_query_results = into_array(
                take(&mut object, "expected_physics_query_results")?,
                "ticks[].expected_physics_query_results",
            )?
            .into_iter()
            .map(|value| {
                PhysicsQueryResultV1::from_canonical_bytes(
                    &decode_hex(value, "ticks[].expected_physics_query_results[]")?,
                    limits,
                )
                .map_err(ManifestValidationError::from)
                .map_err(ManifestCodecError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
            let expected_outcome_command_batch =
                ClosedCommandAdmissionBatchV2::from_canonical_bytes(
                    &decode_hex(
                        take(&mut object, "expected_outcome_command_batch")?,
                        "ticks[].expected_outcome_command_batch",
                    )?,
                    limits,
                    admission,
                )
                .map_err(ManifestValidationError::from)?;
            let expected_mapping_receipts = into_array(
                take(&mut object, "expected_mapping_receipts")?,
                "ticks[].expected_mapping_receipts",
            )?
            .into_iter()
            .map(|value| {
                InputMappingReceiptV2::from_canonical_bytes(
                    &decode_hex(value, "ticks[].expected_mapping_receipts[]")?,
                    limits,
                )
                .map_err(ManifestValidationError::from)
                .map_err(ManifestCodecError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
            let expected_interaction_availability = into_array(
                take(&mut object, "expected_interaction_availability")?,
                "ticks[].expected_interaction_availability",
            )?
            .into_iter()
            .map(|value| {
                InteractionAvailabilityV1::from_canonical_bytes(
                    &decode_hex(value, "ticks[].expected_interaction_availability[]")?,
                    limits,
                )
                .map_err(ManifestValidationError::from)
                .map_err(ManifestCodecError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
            let expected_command_results =
                decode_command_results(take(&mut object, "expected_command_results")?)?;
            let expected_events = decode_domain_events(take(&mut object, "expected_events")?)?;
            if let Some(field) = object.into_keys().next() {
                return Err(ManifestCodecError::UnknownField(format!("ticks[].{field}")));
            }
            Ok(ReplayTickManifestV7 {
                tick,
                world_streaming_input,
                closed_ingress_batch,
                direct_external_commands,
                expected_ingress_command_batch,
                expected_physics_step_input,
                expected_contact_batch,
                expected_targeting_intents,
                expected_authoritative_targeting_queries,
                expected_physics_query_batch,
                expected_physics_query_results,
                expected_outcome_command_batch,
                expected_mapping_receipts,
                expected_interaction_availability,
                expected_command_results,
                expected_events,
            })
        })
        .collect()
}
