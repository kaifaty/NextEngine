use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::persistence::{
    AuthorityGrant, CommandLedgerDescriptorV2, HashBinding, ManifestValidationError,
    REPLAY_MANIFEST_V3_SCHEMA_VERSION, ReplayCommandRecord, ReplayCommandResultV2,
    ReplayComparePointV3, ReplayManifestV3, ReplayOwnerSegmentV2, ReplayTickManifestV3,
    SaveCompatibility, SaveManifestV2, SaveSegmentDescriptor, SchemaBinding, TickSettings,
};
use crate::{
    CanonicalDecodeLimits, CanonicalError, CapabilityId, ClosedCommandAdmissionBatchV2,
    ClosedIngressBatchV1, ClosedPhysicsContactBatchV1, CommandId, CommandLedgerHash, CommandPhase,
    ContentHash, DomainEvent, EventPayload, InputMappingReceiptV1, IssuerPrincipal, PersistentId,
    PhysicalEventV1, PhysicsPoseV1, PhysicsStepInputV2, PrincipalDecodeError,
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SEGMENT_ID, RpgEvent,
    RuntimeAdmissionLimitsV1, RuntimeSnapshot, SchemaId, SkillProficiency, StateRoot,
    WorldNamespaceId, content_hash_from_bytes,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ManifestCodecError {
    Validation(ManifestValidationError),
    Canonicalization(CanonicalError),
    Principal(PrincipalDecodeError),
    Identifier(crate::IdentifierError),
    InputTooLarge {
        actual: usize,
        limit: usize,
    },
    UnexpectedEnd,
    InvalidUtf8,
    InvalidSyntax,
    InvalidEscape,
    InvalidUnicodeEscape,
    DuplicateObjectKey(String),
    TooManyItems {
        limit: usize,
    },
    NestingTooDeep,
    MissingField(String),
    UnknownField(String),
    WrongType {
        field: String,
        expected: &'static str,
    },
    InvalidInteger(String),
    InvalidHex(String),
    NonCanonicalJcs,
}

impl Display for ManifestCodecError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(error) => write!(formatter, "manifest validation failed: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "manifest canonicalization failed: {error}")
            }
            Self::Principal(error) => write!(formatter, "manifest principal is invalid: {error}"),
            Self::Identifier(error) => write!(formatter, "manifest identifier is invalid: {error}"),
            Self::InputTooLarge { actual, limit } => {
                write!(formatter, "manifest has {actual} bytes; limit is {limit}")
            }
            Self::UnexpectedEnd => formatter.write_str("manifest ended unexpectedly"),
            Self::InvalidUtf8 => formatter.write_str("manifest contains invalid UTF-8"),
            Self::InvalidSyntax => formatter.write_str("manifest JSON syntax is invalid"),
            Self::InvalidEscape => formatter.write_str("manifest JSON escape is invalid"),
            Self::InvalidUnicodeEscape => {
                formatter.write_str("manifest JSON Unicode escape is invalid")
            }
            Self::DuplicateObjectKey(key) => write!(formatter, "duplicate manifest key {key}"),
            Self::TooManyItems { limit } => {
                write!(formatter, "manifest exceeds item limit {limit}")
            }
            Self::NestingTooDeep => formatter.write_str("manifest nesting is too deep"),
            Self::MissingField(field) => write!(formatter, "manifest field {field} is missing"),
            Self::UnknownField(field) => write!(formatter, "manifest field {field} is unknown"),
            Self::WrongType { field, expected } => {
                write!(formatter, "manifest field {field} must be {expected}")
            }
            Self::InvalidInteger(field) => {
                write!(formatter, "manifest field {field} is not a valid integer")
            }
            Self::InvalidHex(field) => {
                write!(
                    formatter,
                    "manifest field {field} is not canonical lowercase hex"
                )
            }
            Self::NonCanonicalJcs => {
                formatter.write_str("manifest bytes are valid JSON but not canonical JCS")
            }
        }
    }
}

impl Error for ManifestCodecError {}

impl From<ManifestValidationError> for ManifestCodecError {
    fn from(error: ManifestValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<CanonicalError> for ManifestCodecError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<PrincipalDecodeError> for ManifestCodecError {
    fn from(error: PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}

impl From<crate::IdentifierError> for ManifestCodecError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum JcsValue {
    String(String),
    Number(u64),
    Array(Vec<JcsValue>),
    Object(BTreeMap<String, JcsValue>),
}

pub(crate) fn encode_save_manifest(
    manifest: &SaveManifestV2,
) -> Result<Vec<u8>, ManifestCodecError> {
    manifest.validate()?;
    let mut object = BTreeMap::new();
    object.insert(
        "command_ledger".to_owned(),
        encode_command_ledger(manifest.command_ledger),
    );
    object.insert(
        "content_manifest_hash".to_owned(),
        string(manifest.compatibility.content_manifest_hash.to_hex()),
    );
    object.insert(
        "engine_build_hash".to_owned(),
        string(manifest.compatibility.engine_build_hash.to_hex()),
    );
    object.insert(
        "game_build_hash".to_owned(),
        string(manifest.compatibility.game_build_hash.to_hex()),
    );
    object.insert(
        "generation".to_owned(),
        string(manifest.generation.to_string()),
    );
    object.insert(
        "loaded_chunk_revisions".to_owned(),
        encode_hash_bindings(&manifest.compatibility.loaded_chunk_revisions),
    );
    object.insert(
        "mechanics_lock_hash".to_owned(),
        string(manifest.compatibility.mechanics_lock_hash.to_hex()),
    );
    object.insert(
        "physical_bindings".to_owned(),
        encode_hash_bindings(&manifest.compatibility.physical_bindings),
    );
    object.insert(
        "plugin_script_bindings".to_owned(),
        encode_hash_bindings(&manifest.compatibility.plugin_script_bindings),
    );
    object.insert(
        "policy_state_schemas".to_owned(),
        encode_schema_bindings(&manifest.compatibility.policy_state_schemas),
    );
    object.insert(
        "project_id".to_owned(),
        string(manifest.compatibility.project_id.as_str()),
    );
    object.insert(
        "rng_stream_states".to_owned(),
        encode_hash_bindings(&manifest.compatibility.rng_stream_states),
    );
    object.insert(
        "schema_registry_hash".to_owned(),
        string(manifest.compatibility.schema_registry_hash.to_hex()),
    );
    object.insert(
        "schema_version".to_owned(),
        JcsValue::Number(u64::from(manifest.schema_version)),
    );
    object.insert("segments".to_owned(), encode_segments(&manifest.segments));
    object.insert(
        "tick_settings".to_owned(),
        JcsValue::Array(vec![
            JcsValue::Number(u64::from(manifest.compatibility.tick_settings.gameplay_hz)),
            JcsValue::Number(u64::from(manifest.compatibility.tick_settings.physics_hz)),
            JcsValue::Number(u64::from(manifest.compatibility.tick_settings.motor_hz)),
        ]),
    );
    object.insert(
        "world_revision".to_owned(),
        string(manifest.world_revision.to_string()),
    );
    Ok(encode_value(&JcsValue::Object(object)).into_bytes())
}

pub(crate) fn decode_save_manifest(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<SaveManifestV2, ManifestCodecError> {
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
    let command_ledger = decode_command_ledger(take(&mut object, "command_ledger")?)?;
    let compatibility = SaveCompatibility {
        engine_build_hash: decode_hash(
            take(&mut object, "engine_build_hash")?,
            "engine_build_hash",
        )?,
        game_build_hash: decode_hash(take(&mut object, "game_build_hash")?, "game_build_hash")?,
        project_id: SchemaId::new(into_string(take(&mut object, "project_id")?, "project_id")?)?,
        schema_registry_hash: decode_hash(
            take(&mut object, "schema_registry_hash")?,
            "schema_registry_hash",
        )?,
        content_manifest_hash: decode_hash(
            take(&mut object, "content_manifest_hash")?,
            "content_manifest_hash",
        )?,
        mechanics_lock_hash: decode_hash(
            take(&mut object, "mechanics_lock_hash")?,
            "mechanics_lock_hash",
        )?,
        tick_settings: decode_tick_settings(take(&mut object, "tick_settings")?)?,
        loaded_chunk_revisions: decode_hash_bindings(
            take(&mut object, "loaded_chunk_revisions")?,
            "loaded_chunk_revisions",
        )?,
        rng_stream_states: decode_hash_bindings(
            take(&mut object, "rng_stream_states")?,
            "rng_stream_states",
        )?,
        physical_bindings: decode_hash_bindings(
            take(&mut object, "physical_bindings")?,
            "physical_bindings",
        )?,
        policy_state_schemas: decode_schema_bindings(take(&mut object, "policy_state_schemas")?)?,
        plugin_script_bindings: decode_hash_bindings(
            take(&mut object, "plugin_script_bindings")?,
            "plugin_script_bindings",
        )?,
    };
    let manifest = SaveManifestV2 {
        schema_version: decode_u32(take(&mut object, "schema_version")?, "schema_version")?,
        generation: decode_u64_string(take(&mut object, "generation")?, "generation")?,
        world_revision: decode_u64_string(take(&mut object, "world_revision")?, "world_revision")?,
        compatibility,
        command_ledger,
        segments: decode_segments(take(&mut object, "segments")?)?,
    };
    if let Some(field) = object.into_keys().next() {
        return Err(ManifestCodecError::UnknownField(field));
    }
    manifest.validate()?;
    Ok(manifest)
}

pub(crate) fn encode_replay_manifest_v3(
    manifest: &ReplayManifestV3,
) -> Result<Vec<u8>, ManifestCodecError> {
    manifest.validate_and_decode(CanonicalDecodeLimits::default())?;
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
                .map(encode_compare_point)
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
                .map(encode_replay_tick)
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    Ok(encode_value(&JcsValue::Object(object)).into_bytes())
}

pub(crate) fn decode_replay_manifest_v3(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<ReplayManifestV3, ManifestCodecError> {
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
    if schema_version != REPLAY_MANIFEST_V3_SCHEMA_VERSION {
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
        RuntimeSnapshot::from_canonical_bytes(&runtime_segment.canonical_bytes, limits)
            .map_err(ManifestValidationError::from)?;
    let authority = decode_authority(take(&mut object, "authority")?, limits)?;
    let ticks = decode_replay_ticks(
        take(&mut object, "ticks")?,
        limits,
        &runtime_snapshot.admission_limits,
    )?;
    let compare_points = decode_compare_points(take(&mut object, "compare_points")?)?;
    if let Some(field) = object.into_keys().next() {
        return Err(ManifestCodecError::UnknownField(field));
    }
    let manifest = ReplayManifestV3 {
        schema_version,
        compatibility,
        initial_owner_segments,
        initial_state_root,
        authority,
        ticks,
        compare_points,
    };
    manifest.validate_and_decode(limits)?;
    Ok(manifest)
}

fn encode_compatibility(compatibility: &SaveCompatibility) -> JcsValue {
    let mut object = BTreeMap::new();
    object.insert(
        "content_manifest_hash".to_owned(),
        string(compatibility.content_manifest_hash.to_hex()),
    );
    object.insert(
        "engine_build_hash".to_owned(),
        string(compatibility.engine_build_hash.to_hex()),
    );
    object.insert(
        "game_build_hash".to_owned(),
        string(compatibility.game_build_hash.to_hex()),
    );
    object.insert(
        "loaded_chunk_revisions".to_owned(),
        encode_hash_bindings(&compatibility.loaded_chunk_revisions),
    );
    object.insert(
        "mechanics_lock_hash".to_owned(),
        string(compatibility.mechanics_lock_hash.to_hex()),
    );
    object.insert(
        "physical_bindings".to_owned(),
        encode_hash_bindings(&compatibility.physical_bindings),
    );
    object.insert(
        "plugin_script_bindings".to_owned(),
        encode_hash_bindings(&compatibility.plugin_script_bindings),
    );
    object.insert(
        "policy_state_schemas".to_owned(),
        encode_schema_bindings(&compatibility.policy_state_schemas),
    );
    object.insert(
        "project_id".to_owned(),
        string(compatibility.project_id.as_str()),
    );
    object.insert(
        "rng_stream_states".to_owned(),
        encode_hash_bindings(&compatibility.rng_stream_states),
    );
    object.insert(
        "schema_registry_hash".to_owned(),
        string(compatibility.schema_registry_hash.to_hex()),
    );
    object.insert(
        "tick_settings".to_owned(),
        JcsValue::Array(vec![
            JcsValue::Number(u64::from(compatibility.tick_settings.gameplay_hz)),
            JcsValue::Number(u64::from(compatibility.tick_settings.physics_hz)),
            JcsValue::Number(u64::from(compatibility.tick_settings.motor_hz)),
        ]),
    );
    JcsValue::Object(object)
}

fn decode_compatibility(value: JcsValue) -> Result<SaveCompatibility, ManifestCodecError> {
    let mut object = into_object(value, "compatibility")?;
    let compatibility = SaveCompatibility {
        engine_build_hash: decode_hash(
            take(&mut object, "engine_build_hash")?,
            "compatibility.engine_build_hash",
        )?,
        game_build_hash: decode_hash(
            take(&mut object, "game_build_hash")?,
            "compatibility.game_build_hash",
        )?,
        project_id: SchemaId::new(into_string(
            take(&mut object, "project_id")?,
            "compatibility.project_id",
        )?)?,
        schema_registry_hash: decode_hash(
            take(&mut object, "schema_registry_hash")?,
            "compatibility.schema_registry_hash",
        )?,
        content_manifest_hash: decode_hash(
            take(&mut object, "content_manifest_hash")?,
            "compatibility.content_manifest_hash",
        )?,
        mechanics_lock_hash: decode_hash(
            take(&mut object, "mechanics_lock_hash")?,
            "compatibility.mechanics_lock_hash",
        )?,
        tick_settings: decode_tick_settings(take(&mut object, "tick_settings")?)?,
        loaded_chunk_revisions: decode_hash_bindings(
            take(&mut object, "loaded_chunk_revisions")?,
            "compatibility.loaded_chunk_revisions",
        )?,
        rng_stream_states: decode_hash_bindings(
            take(&mut object, "rng_stream_states")?,
            "compatibility.rng_stream_states",
        )?,
        physical_bindings: decode_hash_bindings(
            take(&mut object, "physical_bindings")?,
            "compatibility.physical_bindings",
        )?,
        policy_state_schemas: decode_schema_bindings(take(&mut object, "policy_state_schemas")?)?,
        plugin_script_bindings: decode_hash_bindings(
            take(&mut object, "plugin_script_bindings")?,
            "compatibility.plugin_script_bindings",
        )?,
    };
    if let Some(field) = object.into_keys().next() {
        return Err(ManifestCodecError::UnknownField(format!(
            "compatibility.{field}"
        )));
    }
    Ok(compatibility)
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

fn encode_replay_tick(tick: &ReplayTickManifestV3) -> Result<JcsValue, ManifestCodecError> {
    let mut object = BTreeMap::new();
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
        "expected_physics_step_input".to_owned(),
        string(hex_bytes(
            &tick.expected_physics_step_input.canonical_bytes()?,
        )),
    );
    object.insert("tick".to_owned(), string(tick.tick.to_string()));
    Ok(JcsValue::Object(object))
}

fn decode_replay_ticks(
    value: JcsValue,
    limits: CanonicalDecodeLimits,
    admission: &RuntimeAdmissionLimitsV1,
) -> Result<Vec<ReplayTickManifestV3>, ManifestCodecError> {
    into_array(value, "ticks")?
        .into_iter()
        .map(|row| {
            let mut object = into_object(row, "ticks[]")?;
            let tick = decode_u64_string(take(&mut object, "tick")?, "ticks[].tick")?;
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
                InputMappingReceiptV1::from_canonical_bytes(
                    &decode_hex(value, "ticks[].expected_mapping_receipts[]")?,
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
            Ok(ReplayTickManifestV3 {
                tick,
                closed_ingress_batch,
                direct_external_commands,
                expected_ingress_command_batch,
                expected_physics_step_input,
                expected_contact_batch,
                expected_outcome_command_batch,
                expected_mapping_receipts,
                expected_command_results,
                expected_events,
            })
        })
        .collect()
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

fn encode_compare_point(point: &ReplayComparePointV3) -> JcsValue {
    JcsValue::Array(vec![
        string(point.tick.to_string()),
        string(point.state_root.to_hex()),
        string(point.command_ledger_hash.to_hex()),
        string(point.runtime_segment_hash.to_hex()),
        string(point.rpg_segment_hash.to_hex()),
        string(point.physics_segment_hash.to_hex()),
        string(point.closed_ingress_batch_hash.to_hex()),
        string(point.ingress_command_batch_hash.to_hex()),
        string(point.physics_step_input_hash.to_hex()),
        string(point.contact_batch_hash.to_hex()),
        string(point.outcome_command_batch_hash.to_hex()),
    ])
}

fn decode_compare_points(value: JcsValue) -> Result<Vec<ReplayComparePointV3>, ManifestCodecError> {
    into_array(value, "compare_points")?
        .into_iter()
        .map(|row| {
            let mut columns = into_array(row, "compare_points[]")?.into_iter();
            let point = ReplayComparePointV3 {
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
                runtime_segment_hash: decode_hash(
                    next(&mut columns, "compare_points[].runtime_segment_hash")?,
                    "compare_points[].runtime_segment_hash",
                )?,
                rpg_segment_hash: decode_hash(
                    next(&mut columns, "compare_points[].rpg_segment_hash")?,
                    "compare_points[].rpg_segment_hash",
                )?,
                physics_segment_hash: decode_hash(
                    next(&mut columns, "compare_points[].physics_segment_hash")?,
                    "compare_points[].physics_segment_hash",
                )?,
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
                outcome_command_batch_hash: decode_hash(
                    next(&mut columns, "compare_points[].outcome_command_batch_hash")?,
                    "compare_points[].outcome_command_batch_hash",
                )?,
            };
            ensure_no_more(columns, "compare_points[]")?;
            Ok(point)
        })
        .collect()
}

fn encode_domain_event(event: &DomainEvent) -> Result<JcsValue, ManifestCodecError> {
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
        EventPayload::Rpg(RpgEvent::DialogueQuestAdvanced {
            dialogue_id,
            dialogue_node_id,
            quest_id,
            quest_state_id,
            relationship_source,
            relationship_target,
            relationship_dimension_id,
            relationship_value,
        }) => JcsValue::Array(vec![
            string("rpg_dialogue"),
            string(dialogue_id.to_hex()),
            string(dialogue_node_id.as_str()),
            string(quest_id.to_hex()),
            string(quest_state_id.as_str()),
            string(relationship_source.to_hex()),
            string(relationship_target.to_hex()),
            string(relationship_dimension_id.as_str()),
            string(relationship_value.to_string()),
        ]),
        EventPayload::Rpg(RpgEvent::ItemTransferred {
            item_id,
            previous_owner,
            new_owner,
        }) => JcsValue::Array(vec![
            string("rpg_item"),
            string(item_id.to_hex()),
            encode_optional_persistent_id(*previous_owner),
            encode_optional_persistent_id(*new_owner),
        ]),
        EventPayload::Rpg(RpgEvent::SkillLearned {
            character_id,
            skill_id,
            proficiency,
        }) => JcsValue::Array(vec![
            string("rpg_skill"),
            string(character_id.to_hex()),
            string(skill_id.as_str()),
            JcsValue::Number(u64::from(proficiency.get())),
        ]),
        EventPayload::Rpg(RpgEvent::InteractiveObjectStateChanged {
            object_id,
            state_id,
        }) => JcsValue::Array(vec![
            string("rpg_object"),
            string(object_id.to_hex()),
            string(state_id.as_str()),
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

fn decode_domain_events(value: JcsValue) -> Result<Vec<DomainEvent>, ManifestCodecError> {
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
        "rpg_dialogue" => EventPayload::Rpg(RpgEvent::DialogueQuestAdvanced {
            dialogue_id: decode_persistent_id(next(&mut columns, "event.dialogue_id")?)?,
            dialogue_node_id: decode_schema_id(next(&mut columns, "event.dialogue_node_id")?)?,
            quest_id: decode_persistent_id(next(&mut columns, "event.quest_id")?)?,
            quest_state_id: decode_schema_id(next(&mut columns, "event.quest_state_id")?)?,
            relationship_source: decode_persistent_id(next(
                &mut columns,
                "event.relationship_source",
            )?)?,
            relationship_target: decode_persistent_id(next(
                &mut columns,
                "event.relationship_target",
            )?)?,
            relationship_dimension_id: decode_schema_id(next(
                &mut columns,
                "event.relationship_dimension_id",
            )?)?,
            relationship_value: decode_i32_string(
                next(&mut columns, "event.relationship_value")?,
                "event.relationship_value",
            )?,
        }),
        "rpg_item" => EventPayload::Rpg(RpgEvent::ItemTransferred {
            item_id: decode_persistent_id(next(&mut columns, "event.item_id")?)?,
            previous_owner: decode_optional_persistent_id(next(
                &mut columns,
                "event.previous_owner",
            )?)?,
            new_owner: decode_optional_persistent_id(next(&mut columns, "event.new_owner")?)?,
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
            EventPayload::Rpg(RpgEvent::SkillLearned {
                character_id,
                skill_id,
                proficiency,
            })
        }
        "rpg_object" => EventPayload::Rpg(RpgEvent::InteractiveObjectStateChanged {
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

fn encode_command_ledger(ledger: CommandLedgerDescriptorV2) -> JcsValue {
    JcsValue::Array(vec![
        string(ledger.world_namespace.to_hex()),
        string(ledger.stream_count.to_string()),
        string(ledger.archive_root.to_hex()),
        string(ledger.identity_index_root.to_hex()),
        string(ledger.runtime_snapshot_segment_hash.to_hex()),
    ])
}

fn encode_hash_bindings(bindings: &[HashBinding]) -> JcsValue {
    JcsValue::Array(
        bindings
            .iter()
            .map(|binding| {
                JcsValue::Array(vec![
                    string(binding.binding_id.as_str()),
                    string(binding.content_hash.to_hex()),
                ])
            })
            .collect(),
    )
}

fn encode_schema_bindings(bindings: &[SchemaBinding]) -> JcsValue {
    JcsValue::Array(
        bindings
            .iter()
            .map(|binding| {
                JcsValue::Array(vec![
                    string(binding.schema_id.as_str()),
                    JcsValue::Number(u64::from(binding.schema_version)),
                    string(binding.content_hash.to_hex()),
                ])
            })
            .collect(),
    )
}

fn encode_segments(segments: &[SaveSegmentDescriptor]) -> JcsValue {
    JcsValue::Array(segments.iter().map(encode_segment_descriptor).collect())
}

fn encode_segment_descriptor(segment: &SaveSegmentDescriptor) -> JcsValue {
    JcsValue::Array(vec![
        string(segment.owner_id.as_str()),
        string(segment.schema_id.as_str()),
        string(segment.segment_id.as_str()),
        JcsValue::Number(u64::from(segment.schema_version)),
        string(segment.byte_length.to_string()),
        string(segment.content_hash.to_hex()),
    ])
}

fn decode_command_ledger(value: JcsValue) -> Result<CommandLedgerDescriptorV2, ManifestCodecError> {
    let mut columns = into_array(value, "command_ledger")?.into_iter();
    let descriptor = CommandLedgerDescriptorV2 {
        world_namespace: WorldNamespaceId::from_bytes(decode_fixed_hex::<16>(
            next(&mut columns, "command_ledger.world_namespace")?,
            "command_ledger.world_namespace",
        )?),
        stream_count: decode_u64_string(
            next(&mut columns, "command_ledger.stream_count")?,
            "command_ledger.stream_count",
        )?,
        archive_root: decode_hash(
            next(&mut columns, "command_ledger.archive_root")?,
            "command_ledger.archive_root",
        )?,
        identity_index_root: decode_hash(
            next(&mut columns, "command_ledger.identity_index_root")?,
            "command_ledger.identity_index_root",
        )?,
        runtime_snapshot_segment_hash: decode_hash(
            next(&mut columns, "command_ledger.runtime_snapshot_segment_hash")?,
            "command_ledger.runtime_snapshot_segment_hash",
        )?,
    };
    ensure_no_more(columns, "command_ledger")?;
    Ok(descriptor)
}

fn decode_hash_bindings(
    value: JcsValue,
    field: &'static str,
) -> Result<Vec<HashBinding>, ManifestCodecError> {
    let rows = into_array(value, field)?;
    let mut bindings = Vec::with_capacity(rows.len());
    for row in rows {
        let mut columns = into_array(row, field)?.into_iter();
        let binding_id = SchemaId::new(into_string(next(&mut columns, field)?, field)?)?;
        let content_hash = decode_hash(next(&mut columns, field)?, field)?;
        ensure_no_more(columns, field)?;
        bindings.push(HashBinding {
            binding_id,
            content_hash,
        });
    }
    Ok(bindings)
}

fn decode_schema_bindings(value: JcsValue) -> Result<Vec<SchemaBinding>, ManifestCodecError> {
    let rows = into_array(value, "policy_state_schemas")?;
    let mut bindings = Vec::with_capacity(rows.len());
    for row in rows {
        let mut columns = into_array(row, "policy_state_schemas[]")?.into_iter();
        let schema_id = SchemaId::new(into_string(
            next(&mut columns, "policy_state_schemas[].id")?,
            "policy_state_schemas[].id",
        )?)?;
        let schema_version = decode_u32(
            next(&mut columns, "policy_state_schemas[].version")?,
            "policy_state_schemas[].version",
        )?;
        let content_hash = decode_hash(
            next(&mut columns, "policy_state_schemas[].hash")?,
            "policy_state_schemas[].hash",
        )?;
        ensure_no_more(columns, "policy_state_schemas[]")?;
        bindings.push(SchemaBinding {
            schema_id,
            schema_version,
            content_hash,
        });
    }
    Ok(bindings)
}

fn decode_segments(value: JcsValue) -> Result<Vec<SaveSegmentDescriptor>, ManifestCodecError> {
    into_array(value, "segments")?
        .into_iter()
        .map(decode_segment_descriptor)
        .collect()
}

fn decode_segment_descriptor(value: JcsValue) -> Result<SaveSegmentDescriptor, ManifestCodecError> {
    let mut columns = into_array(value, "segment_descriptor")?.into_iter();
    let descriptor = SaveSegmentDescriptor {
        owner_id: SchemaId::new(into_string(
            next(&mut columns, "segment_descriptor.owner")?,
            "segment_descriptor.owner",
        )?)?,
        schema_id: SchemaId::new(into_string(
            next(&mut columns, "segment_descriptor.schema")?,
            "segment_descriptor.schema",
        )?)?,
        segment_id: SchemaId::new(into_string(
            next(&mut columns, "segment_descriptor.segment")?,
            "segment_descriptor.segment",
        )?)?,
        schema_version: decode_u32(
            next(&mut columns, "segment_descriptor.version")?,
            "segment_descriptor.version",
        )?,
        byte_length: decode_u64_string(
            next(&mut columns, "segment_descriptor.byte_length")?,
            "segment_descriptor.byte_length",
        )?,
        content_hash: decode_hash(
            next(&mut columns, "segment_descriptor.hash")?,
            "segment_descriptor.hash",
        )?,
    };
    ensure_no_more(columns, "segment_descriptor")?;
    Ok(descriptor)
}

fn decode_tick_settings(value: JcsValue) -> Result<TickSettings, ManifestCodecError> {
    let mut values = into_array(value, "tick_settings")?.into_iter();
    let gameplay_hz = decode_u32(
        next(&mut values, "tick_settings.gameplay_hz")?,
        "tick_settings",
    )?;
    let physics_hz = decode_u32(
        next(&mut values, "tick_settings.physics_hz")?,
        "tick_settings",
    )?;
    let motor_hz = decode_u32(
        next(&mut values, "tick_settings.motor_hz")?,
        "tick_settings",
    )?;
    ensure_no_more(values, "tick_settings")?;
    Ok(TickSettings {
        gameplay_hz,
        physics_hz,
        motor_hz,
    })
}

fn string(value: impl Into<String>) -> JcsValue {
    JcsValue::String(value.into())
}

fn take(
    object: &mut BTreeMap<String, JcsValue>,
    field: &'static str,
) -> Result<JcsValue, ManifestCodecError> {
    object
        .remove(field)
        .ok_or_else(|| ManifestCodecError::MissingField(field.to_owned()))
}

fn next(
    values: &mut impl Iterator<Item = JcsValue>,
    field: &'static str,
) -> Result<JcsValue, ManifestCodecError> {
    values
        .next()
        .ok_or_else(|| ManifestCodecError::MissingField(field.to_owned()))
}

fn ensure_no_more(
    mut values: impl Iterator<Item = JcsValue>,
    field: &'static str,
) -> Result<(), ManifestCodecError> {
    if values.next().is_some() {
        Err(ManifestCodecError::UnknownField(field.to_owned()))
    } else {
        Ok(())
    }
}

fn into_object(
    value: JcsValue,
    field: &'static str,
) -> Result<BTreeMap<String, JcsValue>, ManifestCodecError> {
    if let JcsValue::Object(object) = value {
        Ok(object)
    } else {
        Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an object",
        })
    }
}

fn into_array(value: JcsValue, field: &'static str) -> Result<Vec<JcsValue>, ManifestCodecError> {
    if let JcsValue::Array(array) = value {
        Ok(array)
    } else {
        Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an array",
        })
    }
}

fn into_string(value: JcsValue, field: &'static str) -> Result<String, ManifestCodecError> {
    if let JcsValue::String(value) = value {
        Ok(value)
    } else {
        Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "a string",
        })
    }
}

fn decode_u32(value: JcsValue, field: &'static str) -> Result<u32, ManifestCodecError> {
    let JcsValue::Number(value) = value else {
        return Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an unsigned JSON integer",
        });
    };
    u32::try_from(value).map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()))
}

fn decode_u64_string(value: JcsValue, field: &'static str) -> Result<u64, ManifestCodecError> {
    let value = into_string(value, field)?;
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ManifestCodecError::InvalidInteger(field.to_owned()));
    }
    value
        .parse()
        .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()))
}

fn decode_i64_string(value: JcsValue, field: &'static str) -> Result<i64, ManifestCodecError> {
    let value = into_string(value, field)?;
    validate_signed_integer_string(&value, field)?;
    value
        .parse()
        .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()))
}

fn decode_i32_string(value: JcsValue, field: &'static str) -> Result<i32, ManifestCodecError> {
    let value = into_string(value, field)?;
    validate_signed_integer_string(&value, field)?;
    value
        .parse()
        .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()))
}

fn validate_signed_integer_string(
    value: &str,
    field: &'static str,
) -> Result<(), ManifestCodecError> {
    let digits = value.strip_prefix('-').unwrap_or(value);
    if digits.is_empty()
        || (digits.len() > 1 && digits.starts_with('0'))
        || (value.starts_with('-') && digits == "0")
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ManifestCodecError::InvalidInteger(field.to_owned()));
    }
    Ok(())
}

fn decode_hash(value: JcsValue, field: &'static str) -> Result<ContentHash, ManifestCodecError> {
    Ok(content_hash_from_bytes(decode_fixed_hex::<32>(
        value, field,
    )?))
}

fn decode_fixed_hex<const LENGTH: usize>(
    value: JcsValue,
    field: &'static str,
) -> Result<[u8; LENGTH], ManifestCodecError> {
    let bytes = decode_hex(value, field)?;
    bytes
        .try_into()
        .map_err(|_| ManifestCodecError::InvalidHex(field.to_owned()))
}

fn decode_hex(value: JcsValue, field: &'static str) -> Result<Vec<u8>, ManifestCodecError> {
    let value = into_string(value, field)?;
    if !value.len().is_multiple_of(2)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ManifestCodecError::InvalidHex(field.to_owned()));
    }
    let mut output = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        output.push((hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]));
    }
    Ok(output)
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn encode_optional_id_hex<const LENGTH: usize>(value: Option<&[u8; LENGTH]>) -> JcsValue {
    match value {
        Some(value) => JcsValue::Array(vec![string(hex_bytes(value))]),
        None => JcsValue::Array(Vec::new()),
    }
}

fn decode_optional_id_hex<const LENGTH: usize>(
    value: JcsValue,
    field: &'static str,
) -> Result<Option<[u8; LENGTH]>, ManifestCodecError> {
    let values = into_array(value, field)?;
    match values.as_slice() {
        [] => Ok(None),
        [_] => Ok(Some(decode_fixed_hex::<LENGTH>(
            values.into_iter().next().expect("single value"),
            field,
        )?)),
        _ => Err(ManifestCodecError::InvalidHex(field.to_owned())),
    }
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

fn encode_value(value: &JcsValue) -> String {
    match value {
        JcsValue::String(value) => encode_string(value),
        JcsValue::Number(value) => value.to_string(),
        JcsValue::Array(values) => {
            let mut output = String::from("[");
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                output.push_str(&encode_value(value));
            }
            output.push(']');
            output
        }
        JcsValue::Object(object) => {
            let mut output = String::from("{");
            for (index, (key, value)) in object.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                output.push_str(&encode_string(key));
                output.push(':');
                output.push_str(&encode_value(value));
            }
            output.push('}');
            output
        }
    }
}

fn encode_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\u{0c}' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            '\u{00}'..='\u{1f}' => {
                output.push_str(&format!("\\u{:04x}", u32::from(character)));
            }
            _ => output.push(character),
        }
    }
    output.push('"');
    output
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
    item_limit: usize,
    remaining_items: usize,
}

impl<'a> Parser<'a> {
    const fn new(bytes: &'a [u8], max_items: usize) -> Self {
        Self {
            bytes,
            position: 0,
            item_limit: max_items,
            remaining_items: max_items,
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<JcsValue, ManifestCodecError> {
        if depth > 32 {
            return Err(ManifestCodecError::NestingTooDeep);
        }
        match self.peek()? {
            b'"' => Ok(JcsValue::String(self.parse_string()?)),
            b'[' => self.parse_array(depth + 1),
            b'{' => self.parse_object(depth + 1),
            b'0'..=b'9' => self.parse_number(),
            _ => Err(ManifestCodecError::InvalidSyntax),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JcsValue, ManifestCodecError> {
        self.expect(b'[')?;
        let mut values = Vec::new();
        if self.consume(b']') {
            return Ok(JcsValue::Array(values));
        }
        loop {
            self.consume_item()?;
            values.push(self.parse_value(depth)?);
            if self.consume(b']') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(JcsValue::Array(values))
    }

    fn parse_object(&mut self, depth: usize) -> Result<JcsValue, ManifestCodecError> {
        self.expect(b'{')?;
        let mut object = BTreeMap::new();
        if self.consume(b'}') {
            return Ok(JcsValue::Object(object));
        }
        loop {
            self.consume_item()?;
            let key = self.parse_string()?;
            self.expect(b':')?;
            let value = self.parse_value(depth)?;
            if object.insert(key.clone(), value).is_some() {
                return Err(ManifestCodecError::DuplicateObjectKey(key));
            }
            if self.consume(b'}') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(JcsValue::Object(object))
    }

    fn parse_string(&mut self) -> Result<String, ManifestCodecError> {
        self.expect(b'"')?;
        let mut output = String::new();
        loop {
            let byte = self.peek()?;
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(output);
                }
                b'\\' => {
                    self.position += 1;
                    self.parse_escape(&mut output)?;
                }
                0x00..=0x1f => return Err(ManifestCodecError::InvalidSyntax),
                0x20..=0x7f => {
                    output.push(char::from(byte));
                    self.position += 1;
                }
                _ => {
                    let remaining = std::str::from_utf8(&self.bytes[self.position..])
                        .map_err(|_| ManifestCodecError::InvalidUtf8)?;
                    let character = remaining
                        .chars()
                        .next()
                        .ok_or(ManifestCodecError::UnexpectedEnd)?;
                    output.push(character);
                    self.position += character.len_utf8();
                }
            }
        }
    }

    fn parse_escape(&mut self, output: &mut String) -> Result<(), ManifestCodecError> {
        let escaped = self.take()?;
        match escaped {
            b'"' => output.push('"'),
            b'\\' => output.push('\\'),
            b'/' => output.push('/'),
            b'b' => output.push('\u{08}'),
            b'f' => output.push('\u{0c}'),
            b'n' => output.push('\n'),
            b'r' => output.push('\r'),
            b't' => output.push('\t'),
            b'u' => {
                let first = self.parse_hex_quad()?;
                let scalar = if (0xd800..=0xdbff).contains(&first) {
                    self.expect(b'\\')?;
                    self.expect(b'u')?;
                    let second = self.parse_hex_quad()?;
                    if !(0xdc00..=0xdfff).contains(&second) {
                        return Err(ManifestCodecError::InvalidUnicodeEscape);
                    }
                    0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
                } else if (0xdc00..=0xdfff).contains(&first) {
                    return Err(ManifestCodecError::InvalidUnicodeEscape);
                } else {
                    u32::from(first)
                };
                output
                    .push(char::from_u32(scalar).ok_or(ManifestCodecError::InvalidUnicodeEscape)?);
            }
            _ => return Err(ManifestCodecError::InvalidEscape),
        }
        Ok(())
    }

    fn parse_hex_quad(&mut self) -> Result<u16, ManifestCodecError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let byte = self.take()?;
            let nibble = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a' + 10),
                b'A'..=b'F' => u16::from(byte - b'A' + 10),
                _ => return Err(ManifestCodecError::InvalidUnicodeEscape),
            };
            value = (value << 4) | nibble;
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<JcsValue, ManifestCodecError> {
        let start = self.position;
        while self
            .bytes
            .get(self.position)
            .is_some_and(u8::is_ascii_digit)
        {
            self.position += 1;
        }
        let value = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| ManifestCodecError::InvalidUtf8)?
            .parse()
            .map_err(|_| ManifestCodecError::InvalidSyntax)?;
        Ok(JcsValue::Number(value))
    }

    fn consume_item(&mut self) -> Result<(), ManifestCodecError> {
        if self.remaining_items == 0 {
            return Err(ManifestCodecError::TooManyItems {
                limit: self.item_limit,
            });
        }
        self.remaining_items -= 1;
        Ok(())
    }

    fn peek(&self) -> Result<u8, ManifestCodecError> {
        self.bytes
            .get(self.position)
            .copied()
            .ok_or(ManifestCodecError::UnexpectedEnd)
    }

    fn take(&mut self) -> Result<u8, ManifestCodecError> {
        let value = self.peek()?;
        self.position += 1;
        Ok(value)
    }

    fn expect(&mut self, expected: u8) -> Result<(), ManifestCodecError> {
        if self.take()? == expected {
            Ok(())
        } else {
            Err(ManifestCodecError::InvalidSyntax)
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.bytes.get(self.position) == Some(&expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn finish(self) -> Result<(), ManifestCodecError> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err(ManifestCodecError::InvalidSyntax)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::{
        SaveCompatibility, SaveManifestV2, SaveSegmentDescriptor, TickSettings,
    };
    use crate::{
        CanonicalDecodeLimits, CommandLedgerDescriptorV2, ManifestCodecError, SchemaId,
        WorldNamespaceId, content_hash_from_bytes,
    };

    fn manifest() -> SaveManifestV2 {
        SaveManifestV2 {
            schema_version: crate::SAVE_MANIFEST_SCHEMA_VERSION,
            generation: u64::MAX,
            world_revision: 7,
            compatibility: SaveCompatibility {
                engine_build_hash: content_hash_from_bytes([1; 32]),
                game_build_hash: content_hash_from_bytes([2; 32]),
                project_id: SchemaId::new("nextengine.test").expect("valid project"),
                schema_registry_hash: content_hash_from_bytes([3; 32]),
                content_manifest_hash: content_hash_from_bytes([4; 32]),
                mechanics_lock_hash: content_hash_from_bytes([5; 32]),
                tick_settings: TickSettings {
                    gameplay_hz: 30,
                    physics_hz: 120,
                    motor_hz: 60,
                },
                loaded_chunk_revisions: vec![],
                rng_stream_states: vec![],
                physical_bindings: vec![],
                policy_state_schemas: vec![],
                plugin_script_bindings: vec![],
            },
            command_ledger: CommandLedgerDescriptorV2 {
                world_namespace: WorldNamespaceId::from_bytes([6; 16]),
                stream_count: 0,
                archive_root: content_hash_from_bytes([7; 32]),
                identity_index_root: content_hash_from_bytes([8; 32]),
                runtime_snapshot_segment_hash: content_hash_from_bytes([9; 32]),
            },
            segments: vec![
                SaveSegmentDescriptor::for_bytes(
                    SchemaId::new(crate::RUNTIME_SNAPSHOT_OWNER_ID).expect("valid owner"),
                    SchemaId::new(crate::RUNTIME_SNAPSHOT_SCHEMA_ID).expect("valid schema"),
                    SchemaId::new(crate::RUNTIME_SNAPSHOT_SEGMENT_ID).expect("valid segment"),
                    crate::RUNTIME_SNAPSHOT_SCHEMA_VERSION,
                    b"snapshot",
                )
                .expect("valid segment"),
            ],
        }
    }

    #[test]
    fn save_manifest_jcs_round_trip_is_byte_exact() {
        let manifest = manifest();
        let bytes = manifest.to_jcs_bytes().expect("manifest encodes");
        let decoded = SaveManifestV2::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("manifest decodes");
        assert_eq!(decoded, manifest);
        assert_eq!(decoded.to_jcs_bytes().expect("manifest re-encodes"), bytes);
    }

    #[test]
    fn noncanonical_whitespace_and_duplicate_keys_are_rejected() {
        let bytes = manifest().to_jcs_bytes().expect("manifest encodes");
        let mut whitespace = bytes.clone();
        whitespace.insert(1, b' ');
        assert!(
            SaveManifestV2::from_jcs_bytes(&whitespace, CanonicalDecodeLimits::default()).is_err()
        );

        assert!(matches!(
            super::Parser::new(br#"{"a":1,"a":2}"#, 10).parse_value(0),
            Err(ManifestCodecError::DuplicateObjectKey(key)) if key == "a"
        ));
    }
}
