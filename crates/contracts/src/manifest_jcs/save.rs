use std::collections::BTreeMap;

use super::ManifestCodecError;
use super::jcs::{
    JcsValue, Parser, decode_fixed_hex, decode_hash, decode_u32, decode_u64_string, encode_value,
    ensure_no_more, into_array, into_object, into_string, next, string, take,
};
use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{SchemaId, WorldNamespaceId};
use crate::persistence::{
    CommandLedgerDescriptorV2, HashBinding, SaveCompatibility, SaveManifestV2,
    SaveSegmentDescriptor, SchemaBinding, TickSettings,
};

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
pub(super) fn encode_compatibility(compatibility: &SaveCompatibility) -> JcsValue {
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

pub(super) fn decode_compatibility(
    value: JcsValue,
) -> Result<SaveCompatibility, ManifestCodecError> {
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

pub(super) fn encode_segment_descriptor(segment: &SaveSegmentDescriptor) -> JcsValue {
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

pub(super) fn decode_segment_descriptor(
    value: JcsValue,
) -> Result<SaveSegmentDescriptor, ManifestCodecError> {
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
