use std::collections::BTreeMap;

use super::ManifestCodecError;
use super::jcs::{
    JcsValue, Parser, decode_fixed_hex, decode_hash, decode_hex, decode_optional_id_hex,
    decode_u32, decode_u64_string, encode_optional_id_hex, encode_value, ensure_no_more, hex_bytes,
    into_array, into_object, into_string, next, string, take,
};
use super::replay_event::encode_domain_event;
use super::save::{
    decode_compatibility, decode_segment_descriptor, encode_compatibility,
    encode_segment_descriptor,
};
use crate::canonical::CanonicalDecodeLimits;
use crate::command::IssuerPrincipal;
use crate::ids::{CapabilityId, CommandId, CommandLedgerHash, SchemaId, StateRoot};
use crate::persistence::{
    AuthorityGrant, ManifestValidationError, REPLAY_MANIFEST_V9_SCHEMA_VERSION,
    REPLAY_MANIFEST_V10_SCHEMA_VERSION, ReplayCommandRecord, ReplayCommandResultV2,
    ReplayComparePointV9, ReplayManifestV9, ReplayManifestV10, ReplayOwnerSegmentV2,
    ReplayTickManifestV9, WorldStreamingReplayInputV1,
};
use crate::snapshot::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SEGMENT_ID,
    RuntimeSnapshotV3,
};

mod ticks_v6;

use ticks_v6::decode_replay_ticks_v6;

pub(crate) fn encode_replay_manifest_v9(
    manifest: &ReplayManifestV9,
) -> Result<Vec<u8>, ManifestCodecError> {
    manifest.validate_and_decode(CanonicalDecodeLimits::default())?;
    encode_replay_manifest_fields(manifest)
}

pub(crate) fn encode_replay_manifest_v10(
    manifest: &ReplayManifestV10,
) -> Result<Vec<u8>, ManifestCodecError> {
    manifest.validate_and_decode(CanonicalDecodeLimits::default())?;
    encode_replay_manifest_fields(&ReplayManifestV9 {
        schema_version: manifest.schema_version,
        compatibility: manifest.compatibility.clone(),
        initial_owner_segments: manifest.initial_owner_segments.clone(),
        initial_state_root: manifest.initial_state_root,
        authority: manifest.authority.clone(),
        ticks: manifest.ticks.clone(),
        compare_points: manifest.compare_points.clone(),
    })
}

fn encode_replay_manifest_fields(
    manifest: &ReplayManifestV9,
) -> Result<Vec<u8>, ManifestCodecError> {
    let mut object = BTreeMap::new();
    object.insert(
        "authority".to_owned(),
        JcsValue::Array(
            manifest
                .authority
                .iter()
                .map(encode_authority_grant)
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    object.insert(
        "compare_points".to_owned(),
        JcsValue::Array(
            manifest
                .compare_points
                .iter()
                .map(encode_compare_point_v6)
                .collect(),
        ),
    );
    object.insert(
        "compatibility".to_owned(),
        encode_compatibility(&manifest.compatibility),
    );
    object.insert(
        "initial_owner_segments".to_owned(),
        JcsValue::Array(
            manifest
                .initial_owner_segments
                .iter()
                .map(encode_owner_segment)
                .collect(),
        ),
    );
    object.insert(
        "initial_state_root".to_owned(),
        string(manifest.initial_state_root.to_hex()),
    );
    object.insert(
        "schema_version".to_owned(),
        JcsValue::Number(u64::from(manifest.schema_version)),
    );
    object.insert(
        "ticks".to_owned(),
        JcsValue::Array(
            manifest
                .ticks
                .iter()
                .map(encode_replay_tick_v6)
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    Ok(encode_value(&JcsValue::Object(object)).into_bytes())
}

pub(crate) fn decode_replay_manifest_v9(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<ReplayManifestV9, ManifestCodecError> {
    let manifest = decode_replay_manifest_fields(bytes, limits, REPLAY_MANIFEST_V9_SCHEMA_VERSION)?;
    manifest.validate_and_decode(limits)?;
    Ok(manifest)
}

pub(crate) fn decode_replay_manifest_v10(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<ReplayManifestV10, ManifestCodecError> {
    let raw = decode_replay_manifest_fields(bytes, limits, REPLAY_MANIFEST_V10_SCHEMA_VERSION)?;
    let manifest = ReplayManifestV10 {
        schema_version: raw.schema_version,
        compatibility: raw.compatibility,
        initial_owner_segments: raw.initial_owner_segments,
        initial_state_root: raw.initial_state_root,
        authority: raw.authority,
        ticks: raw.ticks,
        compare_points: raw.compare_points,
    };
    manifest.validate_and_decode(limits)?;
    Ok(manifest)
}

fn decode_replay_manifest_fields(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    expected_schema_version: u32,
) -> Result<ReplayManifestV9, ManifestCodecError> {
    if bytes.len() > limits.max_total_bytes {
        return Err(ManifestCodecError::InputTooLarge {
            actual: bytes.len(),
            limit: limits.max_total_bytes,
        });
    }
    let mut parser = Parser::new(bytes, limits.max_sequence_items);
    let value = parser.parse_value(0)?;
    parser.finish()?;
    if encode_value(&value).as_bytes() != bytes {
        return Err(ManifestCodecError::NonCanonicalJcs);
    }
    let mut object = into_object(value, "root")?;
    let schema_version = decode_u32(take(&mut object, "schema_version")?, "schema_version")?;
    if schema_version != expected_schema_version {
        return Err(ManifestValidationError::UnsupportedReplayVersion(schema_version).into());
    }

    let compatibility = decode_compatibility(take(&mut object, "compatibility")?)?;
    let initial_owner_segments =
        decode_owner_segments(take(&mut object, "initial_owner_segments")?)?;
    let initial_state_root = StateRoot::from_bytes(decode_fixed_hex::<32>(
        take(&mut object, "initial_state_root")?,
        "initial_state_root",
    )?);
    let runtime_segment = initial_owner_segments
        .iter()
        .find(|segment| {
            segment.descriptor.owner_id.as_str() == RUNTIME_SNAPSHOT_OWNER_ID
                && segment.descriptor.schema_id.as_str() == RUNTIME_SNAPSHOT_SCHEMA_ID
                && segment.descriptor.segment_id.as_str() == RUNTIME_SNAPSHOT_SEGMENT_ID
        })
        .ok_or(ManifestValidationError::ReplayInitialSegmentsInvalid)?;
    let runtime_snapshot =
        RuntimeSnapshotV3::from_canonical_bytes(&runtime_segment.canonical_bytes, limits)
            .map_err(ManifestValidationError::from)?;
    let authority = decode_authority(take(&mut object, "authority")?, limits)?;
    let ticks = decode_replay_ticks_v6(
        take(&mut object, "ticks")?,
        limits,
        &runtime_snapshot.admission_limits,
    )?;
    let compare_points = decode_compare_points_v6(take(&mut object, "compare_points")?)?;
    if let Some(field) = object.into_keys().next() {
        return Err(ManifestCodecError::UnknownField(field));
    }
    let manifest = ReplayManifestV9 {
        schema_version,
        compatibility,
        initial_owner_segments,
        initial_state_root,
        authority,
        ticks,
        compare_points,
    };
    Ok(manifest)
}
fn encode_owner_segment(segment: &ReplayOwnerSegmentV2) -> JcsValue {
    JcsValue::Array(vec![
        encode_segment_descriptor(&segment.descriptor),
        string(hex_bytes(&segment.canonical_bytes)),
    ])
}

fn decode_owner_segments(value: JcsValue) -> Result<Vec<ReplayOwnerSegmentV2>, ManifestCodecError> {
    into_array(value, "initial_owner_segments")?
        .into_iter()
        .map(|row| {
            let mut columns = into_array(row, "initial_owner_segments[]")?.into_iter();
            let descriptor = decode_segment_descriptor(next(
                &mut columns,
                "initial_owner_segments[].descriptor",
            )?)?;
            let canonical_bytes = decode_hex(
                next(&mut columns, "initial_owner_segments[].bytes")?,
                "initial_owner_segments[].bytes",
            )?;
            ensure_no_more(columns, "initial_owner_segments[]")?;
            Ok(ReplayOwnerSegmentV2 {
                descriptor,
                canonical_bytes,
            })
        })
        .collect()
}

fn encode_authority_grant(grant: &AuthorityGrant) -> Result<JcsValue, ManifestCodecError> {
    Ok(JcsValue::Array(vec![
        string(hex_bytes(&grant.principal.canonical_bytes()?)),
        JcsValue::Array(
            grant
                .capabilities
                .iter()
                .map(|capability| string(capability.as_str()))
                .collect(),
        ),
    ]))
}

fn decode_authority(
    value: JcsValue,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<AuthorityGrant>, ManifestCodecError> {
    into_array(value, "authority")?
        .into_iter()
        .map(|row| {
            let mut columns = into_array(row, "authority[]")?.into_iter();
            let principal_bytes = decode_hex(
                next(&mut columns, "authority[].principal")?,
                "authority[].principal",
            )?;
            let principal = IssuerPrincipal::from_canonical_bytes(&principal_bytes, limits)?;
            let capabilities = into_array(
                next(&mut columns, "authority[].capabilities")?,
                "authority[].capabilities",
            )?
            .into_iter()
            .map(|value| {
                CapabilityId::new(into_string(value, "authority[].capabilities[]")?)
                    .map_err(ManifestCodecError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
            ensure_no_more(columns, "authority[]")?;
            Ok(AuthorityGrant {
                principal,
                capabilities,
            })
        })
        .collect()
}

fn encode_replay_tick_v6(tick: &ReplayTickManifestV9) -> Result<JcsValue, ManifestCodecError> {
    let mut object = BTreeMap::new();
    object.insert(
        "expected_interaction_availability".to_owned(),
        JcsValue::Array(
            tick.expected_interaction_availability
                .iter()
                .map(|availability| {
                    availability
                        .canonical_bytes()
                        .map(|bytes| string(hex_bytes(&bytes)))
                        .map_err(ManifestValidationError::from)
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    object.insert(
        "closed_ingress_batch".to_owned(),
        string(hex_bytes(&tick.closed_ingress_batch.canonical_bytes()?)),
    );
    object.insert(
        "direct_external_commands".to_owned(),
        JcsValue::Array(
            tick.direct_external_commands
                .iter()
                .map(encode_command_record)
                .collect(),
        ),
    );
    object.insert(
        "expected_authoritative_targeting_queries".to_owned(),
        JcsValue::Array(
            tick.expected_authoritative_targeting_queries
                .iter()
                .map(|query| {
                    query
                        .canonical_bytes()
                        .map(|bytes| string(hex_bytes(&bytes)))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    object.insert(
        "expected_command_results".to_owned(),
        JcsValue::Array(
            tick.expected_command_results
                .iter()
                .map(encode_command_result)
                .collect(),
        ),
    );
    object.insert(
        "expected_contact_batch".to_owned(),
        string(hex_bytes(&tick.expected_contact_batch.canonical_bytes()?)),
    );
    object.insert(
        "expected_events".to_owned(),
        JcsValue::Array(
            tick.expected_events
                .iter()
                .map(encode_domain_event)
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    object.insert(
        "expected_ingress_command_batch".to_owned(),
        string(hex_bytes(
            &tick.expected_ingress_command_batch.canonical_bytes()?,
        )),
    );
    object.insert(
        "expected_mapping_receipts".to_owned(),
        JcsValue::Array(
            tick.expected_mapping_receipts
                .iter()
                .map(|receipt| {
                    receipt
                        .canonical_bytes()
                        .map(|bytes| string(hex_bytes(&bytes)))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    object.insert(
        "expected_outcome_command_batch".to_owned(),
        string(hex_bytes(
            &tick.expected_outcome_command_batch.canonical_bytes()?,
        )),
    );
    object.insert(
        "expected_physics_query_batch".to_owned(),
        string(hex_bytes(
            &tick.expected_physics_query_batch.canonical_bytes()?,
        )),
    );
    object.insert(
        "expected_physics_query_results".to_owned(),
        JcsValue::Array(
            tick.expected_physics_query_results
                .iter()
                .map(|result| {
                    result
                        .canonical_bytes()
                        .map(|bytes| string(hex_bytes(&bytes)))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    object.insert(
        "expected_physics_step_input".to_owned(),
        string(hex_bytes(
            &tick.expected_physics_step_input.canonical_bytes()?,
        )),
    );
    object.insert(
        "expected_targeting_intents".to_owned(),
        JcsValue::Array(
            tick.expected_targeting_intents
                .iter()
                .map(|intent| {
                    intent
                        .canonical_bytes()
                        .map(|bytes| string(hex_bytes(&bytes)))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    object.insert("tick".to_owned(), string(tick.tick.to_string()));
    object.insert(
        "world_streaming_input".to_owned(),
        encode_world_streaming_input(&tick.world_streaming_input),
    );
    Ok(JcsValue::Object(object))
}

fn encode_world_streaming_input(input: &WorldStreamingReplayInputV1) -> JcsValue {
    match input {
        WorldStreamingReplayInputV1::None => JcsValue::Array(vec![JcsValue::Number(0)]),
        WorldStreamingReplayInputV1::BeginTransition {
            target_chunk_id,
            expected_base_world_state_hash,
            expected_next_world_state_hash,
        } => JcsValue::Array(vec![
            JcsValue::Number(1),
            string(target_chunk_id.as_str()),
            string(expected_base_world_state_hash.to_hex()),
            string(expected_next_world_state_hash.to_hex()),
        ]),
        WorldStreamingReplayInputV1::CompletePendingTransition {
            target_chunk_id,
            expected_base_world_state_hash,
            expected_loaded_result_hash,
            expected_next_world_state_hash,
        } => JcsValue::Array(vec![
            JcsValue::Number(2),
            string(target_chunk_id.as_str()),
            string(expected_base_world_state_hash.to_hex()),
            string(expected_loaded_result_hash.to_hex()),
            string(expected_next_world_state_hash.to_hex()),
        ]),
    }
}

pub(super) fn decode_world_streaming_input(
    value: JcsValue,
) -> Result<WorldStreamingReplayInputV1, ManifestCodecError> {
    let mut columns = into_array(value, "ticks[].world_streaming_input")?.into_iter();
    let tag = decode_u32(
        next(&mut columns, "ticks[].world_streaming_input.tag")?,
        "ticks[].world_streaming_input.tag",
    )?;
    let input = match tag {
        0 => WorldStreamingReplayInputV1::None,
        1 => WorldStreamingReplayInputV1::BeginTransition {
            target_chunk_id: SchemaId::new(into_string(
                next(
                    &mut columns,
                    "ticks[].world_streaming_input.target_chunk_id",
                )?,
                "ticks[].world_streaming_input.target_chunk_id",
            )?)?,
            expected_base_world_state_hash: decode_hash(
                next(&mut columns, "ticks[].world_streaming_input.base_hash")?,
                "ticks[].world_streaming_input.base_hash",
            )?,
            expected_next_world_state_hash: decode_hash(
                next(&mut columns, "ticks[].world_streaming_input.next_hash")?,
                "ticks[].world_streaming_input.next_hash",
            )?,
        },
        2 => WorldStreamingReplayInputV1::CompletePendingTransition {
            target_chunk_id: SchemaId::new(into_string(
                next(
                    &mut columns,
                    "ticks[].world_streaming_input.target_chunk_id",
                )?,
                "ticks[].world_streaming_input.target_chunk_id",
            )?)?,
            expected_base_world_state_hash: decode_hash(
                next(&mut columns, "ticks[].world_streaming_input.base_hash")?,
                "ticks[].world_streaming_input.base_hash",
            )?,
            expected_loaded_result_hash: decode_hash(
                next(&mut columns, "ticks[].world_streaming_input.loaded_hash")?,
                "ticks[].world_streaming_input.loaded_hash",
            )?,
            expected_next_world_state_hash: decode_hash(
                next(&mut columns, "ticks[].world_streaming_input.next_hash")?,
                "ticks[].world_streaming_input.next_hash",
            )?,
        },
        _ => return Err(ManifestValidationError::ReplayWorldStreamingInputInvalid.into()),
    };
    ensure_no_more(columns, "ticks[].world_streaming_input")?;
    input.validate()?;
    Ok(input)
}

fn encode_command_record(record: &ReplayCommandRecord) -> JcsValue {
    JcsValue::Array(vec![
        JcsValue::Number(u64::from(record.envelope_schema_version)),
        encode_optional_id_hex(record.claimed_command_id.as_ref().map(CommandId::as_bytes)),
        string(record.command_id.to_hex()),
        string(hex_bytes(&record.canonical_command_bytes)),
    ])
}

fn decode_command_records(value: JcsValue) -> Result<Vec<ReplayCommandRecord>, ManifestCodecError> {
    into_array(value, "ticks[].direct_external_commands")?
        .into_iter()
        .map(|row| {
            let mut columns = into_array(row, "ticks[].direct_external_commands[]")?.into_iter();
            let envelope_schema_version = u16::try_from(decode_u32(
                next(
                    &mut columns,
                    "ticks[].direct_external_commands[].envelope_schema_version",
                )?,
                "ticks[].direct_external_commands[].envelope_schema_version",
            )?)
            .map_err(|_| {
                ManifestCodecError::InvalidInteger(
                    "ticks[].direct_external_commands[].envelope_schema_version".to_owned(),
                )
            })?;
            let claimed_command_id = decode_optional_id_hex::<16>(
                next(
                    &mut columns,
                    "ticks[].direct_external_commands[].claimed_command_id",
                )?,
                "ticks[].direct_external_commands[].claimed_command_id",
            )?
            .map(CommandId::from_bytes);
            let command_id = CommandId::from_bytes(decode_fixed_hex::<16>(
                next(
                    &mut columns,
                    "ticks[].direct_external_commands[].command_id",
                )?,
                "ticks[].direct_external_commands[].command_id",
            )?);
            let canonical_command_bytes = decode_hex(
                next(
                    &mut columns,
                    "ticks[].direct_external_commands[].canonical_command_bytes",
                )?,
                "ticks[].direct_external_commands[].canonical_command_bytes",
            )?;
            ensure_no_more(columns, "ticks[].direct_external_commands[]")?;
            Ok(ReplayCommandRecord {
                envelope_schema_version,
                claimed_command_id,
                command_id,
                canonical_command_bytes,
            })
        })
        .collect()
}

fn encode_command_result(result: &ReplayCommandResultV2) -> JcsValue {
    JcsValue::Array(vec![
        string(result.command_id.to_hex()),
        string(result.sequence.to_string()),
        string(&result.disposition_code),
    ])
}

fn decode_command_results(
    value: JcsValue,
) -> Result<Vec<ReplayCommandResultV2>, ManifestCodecError> {
    into_array(value, "ticks[].expected_command_results")?
        .into_iter()
        .map(|row| {
            let mut columns = into_array(row, "ticks[].expected_command_results[]")?.into_iter();
            let result = ReplayCommandResultV2 {
                command_id: CommandId::from_bytes(decode_fixed_hex::<16>(
                    next(
                        &mut columns,
                        "ticks[].expected_command_results[].command_id",
                    )?,
                    "ticks[].expected_command_results[].command_id",
                )?),
                sequence: decode_u64_string(
                    next(&mut columns, "ticks[].expected_command_results[].sequence")?,
                    "ticks[].expected_command_results[].sequence",
                )?,
                disposition_code: into_string(
                    next(
                        &mut columns,
                        "ticks[].expected_command_results[].disposition_code",
                    )?,
                    "ticks[].expected_command_results[].disposition_code",
                )?,
            };
            ensure_no_more(columns, "ticks[].expected_command_results[]")?;
            Ok(result)
        })
        .collect()
}

fn encode_compare_point_v6(point: &ReplayComparePointV9) -> JcsValue {
    JcsValue::Array(vec![
        string(point.tick.to_string()),
        string(point.state_root.to_hex()),
        string(point.command_ledger_hash.to_hex()),
        JcsValue::Array(
            point
                .owner_segments
                .iter()
                .map(encode_segment_descriptor)
                .collect(),
        ),
        string(point.closed_ingress_batch_hash.to_hex()),
        string(point.ingress_command_batch_hash.to_hex()),
        string(point.physics_step_input_hash.to_hex()),
        string(point.contact_batch_hash.to_hex()),
        string(point.physics_query_batch_hash.to_hex()),
        string(point.physics_query_results_hash.to_hex()),
        string(point.targeting_query_trace_hash.to_hex()),
        string(point.outcome_command_batch_hash.to_hex()),
        string(point.interaction_availability_hash.to_hex()),
    ])
}

fn decode_compare_points_v6(
    value: JcsValue,
) -> Result<Vec<ReplayComparePointV9>, ManifestCodecError> {
    into_array(value, "compare_points")?
        .into_iter()
        .map(|row| {
            let mut columns = into_array(row, "compare_points[]")?.into_iter();
            let point = ReplayComparePointV9 {
                tick: decode_u64_string(
                    next(&mut columns, "compare_points[].tick")?,
                    "compare_points[].tick",
                )?,
                state_root: StateRoot::from_bytes(decode_fixed_hex::<32>(
                    next(&mut columns, "compare_points[].state_root")?,
                    "compare_points[].state_root",
                )?),
                command_ledger_hash: CommandLedgerHash::from_bytes(decode_fixed_hex::<32>(
                    next(&mut columns, "compare_points[].command_ledger_hash")?,
                    "compare_points[].command_ledger_hash",
                )?),
                owner_segments: into_array(
                    next(&mut columns, "compare_points[].owner_segments")?,
                    "compare_points[].owner_segments",
                )?
                .into_iter()
                .map(decode_segment_descriptor)
                .collect::<Result<Vec<_>, _>>()?,
                closed_ingress_batch_hash: decode_hash(
                    next(&mut columns, "compare_points[].closed_ingress_batch_hash")?,
                    "compare_points[].closed_ingress_batch_hash",
                )?,
                ingress_command_batch_hash: decode_hash(
                    next(&mut columns, "compare_points[].ingress_command_batch_hash")?,
                    "compare_points[].ingress_command_batch_hash",
                )?,
                physics_step_input_hash: decode_hash(
                    next(&mut columns, "compare_points[].physics_step_input_hash")?,
                    "compare_points[].physics_step_input_hash",
                )?,
                contact_batch_hash: decode_hash(
                    next(&mut columns, "compare_points[].contact_batch_hash")?,
                    "compare_points[].contact_batch_hash",
                )?,
                physics_query_batch_hash: decode_hash(
                    next(&mut columns, "compare_points[].physics_query_batch_hash")?,
                    "compare_points[].physics_query_batch_hash",
                )?,
                physics_query_results_hash: decode_hash(
                    next(&mut columns, "compare_points[].physics_query_results_hash")?,
                    "compare_points[].physics_query_results_hash",
                )?,
                targeting_query_trace_hash: decode_hash(
                    next(&mut columns, "compare_points[].targeting_query_trace_hash")?,
                    "compare_points[].targeting_query_trace_hash",
                )?,
                outcome_command_batch_hash: decode_hash(
                    next(&mut columns, "compare_points[].outcome_command_batch_hash")?,
                    "compare_points[].outcome_command_batch_hash",
                )?,
                interaction_availability_hash: decode_hash(
                    next(
                        &mut columns,
                        "compare_points[].interaction_availability_hash",
                    )?,
                    "compare_points[].interaction_availability_hash",
                )?,
            };
            ensure_no_more(columns, "compare_points[]")?;
            Ok(point)
        })
        .collect()
}
