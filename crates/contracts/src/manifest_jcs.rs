use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::persistence::{
    CommandLedgerDescriptor, HashBinding, ManifestValidationError, SaveCompatibility,
    SaveManifestV1, SaveSegmentDescriptor, SchemaBinding, TickSettings,
};
use crate::{
    CanonicalDecodeLimits, CanonicalError, CommandId, CommandStreamId, ContentHash,
    IssuerPrincipal, PrincipalDecodeError, SchemaId, content_hash_from_bytes,
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
    manifest: &SaveManifestV1,
) -> Result<Vec<u8>, ManifestCodecError> {
    manifest.validate()?;
    let mut object = BTreeMap::new();
    object.insert(
        "command_ledgers".to_owned(),
        encode_command_ledgers(&manifest.command_ledgers)?,
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
) -> Result<SaveManifestV1, ManifestCodecError> {
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
    let command_ledgers = decode_command_ledgers(take(&mut object, "command_ledgers")?, limits)?;
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
    let manifest = SaveManifestV1 {
        schema_version: decode_u32(take(&mut object, "schema_version")?, "schema_version")?,
        generation: decode_u64_string(take(&mut object, "generation")?, "generation")?,
        world_revision: decode_u64_string(take(&mut object, "world_revision")?, "world_revision")?,
        compatibility,
        command_ledgers,
        segments: decode_segments(take(&mut object, "segments")?)?,
    };
    if let Some(field) = object.into_keys().next() {
        return Err(ManifestCodecError::UnknownField(field));
    }
    manifest.validate()?;
    Ok(manifest)
}

fn encode_command_ledgers(
    ledgers: &[CommandLedgerDescriptor],
) -> Result<JcsValue, ManifestCodecError> {
    let mut values = Vec::with_capacity(ledgers.len());
    for ledger in ledgers {
        values.push(JcsValue::Array(vec![
            string(ledger.stream_id.to_hex()),
            string(hex(&ledger.issuer.canonical_bytes()?)),
            string(ledger.last_sequence.to_string()),
            string(ledger.command_id.to_hex()),
        ]));
    }
    Ok(JcsValue::Array(values))
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
    JcsValue::Array(
        segments
            .iter()
            .map(|segment| {
                JcsValue::Array(vec![
                    string(segment.owner_id.as_str()),
                    string(segment.schema_id.as_str()),
                    string(segment.segment_id.as_str()),
                    JcsValue::Number(u64::from(segment.schema_version)),
                    string(segment.byte_length.to_string()),
                    string(segment.content_hash.to_hex()),
                ])
            })
            .collect(),
    )
}

fn decode_command_ledgers(
    value: JcsValue,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CommandLedgerDescriptor>, ManifestCodecError> {
    let rows = into_array(value, "command_ledgers")?;
    let mut ledgers = Vec::with_capacity(rows.len());
    for row in rows {
        let mut columns = into_array(row, "command_ledgers[]")?.into_iter();
        let stream_id = CommandStreamId::from_bytes(decode_fixed_hex::<16>(
            next(&mut columns, "command_ledgers[].stream")?,
            "command_ledgers[].stream",
        )?);
        let principal_bytes = decode_hex(
            next(&mut columns, "command_ledgers[].principal")?,
            "command_ledgers[].principal",
        )?;
        let issuer = IssuerPrincipal::from_canonical_bytes(&principal_bytes, limits)?;
        let last_sequence = decode_u64_string(
            next(&mut columns, "command_ledgers[].sequence")?,
            "command_ledgers[].sequence",
        )?;
        let command_id = CommandId::from_bytes(decode_fixed_hex::<16>(
            next(&mut columns, "command_ledgers[].command_id")?,
            "command_ledgers[].command_id",
        )?);
        ensure_no_more(columns, "command_ledgers[]")?;
        ledgers.push(CommandLedgerDescriptor {
            stream_id,
            issuer,
            last_sequence,
            command_id,
        });
    }
    Ok(ledgers)
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
    let rows = into_array(value, "segments")?;
    let mut segments = Vec::with_capacity(rows.len());
    for row in rows {
        let mut columns = into_array(row, "segments[]")?.into_iter();
        let owner_id = SchemaId::new(into_string(
            next(&mut columns, "segments[].owner")?,
            "segments[].owner",
        )?)?;
        let schema_id = SchemaId::new(into_string(
            next(&mut columns, "segments[].schema")?,
            "segments[].schema",
        )?)?;
        let segment_id = SchemaId::new(into_string(
            next(&mut columns, "segments[].segment")?,
            "segments[].segment",
        )?)?;
        let schema_version = decode_u32(
            next(&mut columns, "segments[].version")?,
            "segments[].version",
        )?;
        let byte_length = decode_u64_string(
            next(&mut columns, "segments[].byte_length")?,
            "segments[].byte_length",
        )?;
        let content_hash = decode_hash(next(&mut columns, "segments[].hash")?, "segments[].hash")?;
        ensure_no_more(columns, "segments[]")?;
        segments.push(SaveSegmentDescriptor {
            owner_id,
            schema_id,
            segment_id,
            schema_version,
            byte_length,
            content_hash,
        });
    }
    Ok(segments)
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
    if value.len() % 2 != 0
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

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
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
        SaveCompatibility, SaveManifestV1, SaveSegmentDescriptor, TickSettings,
    };
    use crate::{CanonicalDecodeLimits, ManifestCodecError, SchemaId, content_hash_from_bytes};

    fn manifest() -> SaveManifestV1 {
        SaveManifestV1 {
            schema_version: 1,
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
            command_ledgers: vec![],
            segments: vec![
                SaveSegmentDescriptor::for_bytes(
                    SchemaId::new("runtime").expect("valid owner"),
                    SchemaId::new("nextengine.runtime.snapshot").expect("valid schema"),
                    SchemaId::new("command-ledger").expect("valid segment"),
                    1,
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
        let decoded = SaveManifestV1::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
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
            SaveManifestV1::from_jcs_bytes(&whitespace, CanonicalDecodeLimits::default()).is_err()
        );

        assert!(matches!(
            super::Parser::new(br#"{"a":1,"a":2}"#, 10).parse_value(0),
            Err(ManifestCodecError::DuplicateObjectKey(key)) if key == "a"
        ));
    }
}
