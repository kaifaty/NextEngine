use std::collections::{BTreeMap, BTreeSet};

use next_contracts::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_OPTIONAL,
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U8, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64,
    CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits, CanonicalField, DecodedCanonicalSegment,
    decode_canonical_segment, encode_canonical_segment,
};
use next_contracts::ids::{ContentHash, InputSourceId, PersistentId, SchemaId};
use next_contracts::input::{ActionMapManifestV1, InputContextStackV1, PlayerActionValueV1};
use next_contracts::platform::{PLATFORM_MAX_CONTROL_COMPONENTS, PLATFORM_MAX_MODIFIERS};

use super::{
    ControlIdentity, MAX_HELD_CONTROLS, MAX_PLATFORM_INPUT_SOURCES, PlatformSourceCursor,
    PlatformSourceKey, PlayerInputError, PlayerInputSessionV1, validate_context_compatibility,
};

const RECOVERY_SCHEMA_VERSION_V1: u32 = 1;
const RECOVERY_SCHEMA_VERSION_V2: u32 = 2;
const RECOVERY_OWNER_ID: &str = "nextengine.player";
const RECOVERY_SCHEMA_ID_V1: &str = "nextengine.player-input-session-recovery.v1";
const RECOVERY_SCHEMA_ID_V2: &str = "nextengine.player-input-session-recovery.v2";
const RECOVERY_SEGMENT_ID_V1: &str = "v1";
const RECOVERY_SEGMENT_ID_V2: &str = "v2";
const RECORD_SEGMENT_ID: &str = "v1";
const SOURCE_CURSOR_SCHEMA_ID: &str = "nextengine.player-input-source-cursor.v1";
const HELD_CONTROL_SCHEMA_ID: &str = "nextengine.player-held-control.v1";
const ACTIVE_ACTION_SCHEMA_ID: &str = "nextengine.player-active-action.v1";
const RECOVERY_FIELD_COUNT_V1: usize = 9;
const RECOVERY_FIELD_COUNT_V2: usize = 10;

impl PlayerInputSessionV1 {
    /// Encodes the exact committed input-session state at a logical-frame
    /// boundary. Transient, not-yet-closed frame state is deliberately not
    /// recoverable and therefore fails closed.
    pub fn recovery_bytes(&self) -> Result<Vec<u8>, PlayerInputError> {
        self.recovery_bytes_for_version(RECOVERY_SCHEMA_VERSION_V2)
    }

    #[cfg(test)]
    pub(super) fn recovery_bytes_v1_for_test(&self) -> Result<Vec<u8>, PlayerInputError> {
        self.recovery_bytes_for_version(RECOVERY_SCHEMA_VERSION_V1)
    }

    fn recovery_bytes_for_version(&self, version: u32) -> Result<Vec<u8>, PlayerInputError> {
        self.validate_recovery_boundary()?;
        let (schema_id, segment_id) = match version {
            RECOVERY_SCHEMA_VERSION_V1 if self.cancelled_action_ids.is_empty() => {
                (RECOVERY_SCHEMA_ID_V1, RECOVERY_SEGMENT_ID_V1)
            }
            RECOVERY_SCHEMA_VERSION_V2 => (RECOVERY_SCHEMA_ID_V2, RECOVERY_SEGMENT_ID_V2),
            _ => return Err(PlayerInputError::RecoveryInvalid),
        };
        let mut fields = vec![
            CanonicalField::new(1, CANONICAL_TYPE_U32, version.to_le_bytes().to_vec()),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_ID128,
                self.controller_id.as_bytes().to_vec(),
            ),
            CanonicalField::new(3, CANONICAL_TYPE_ID128, self.source_id.as_bytes().to_vec()),
            CanonicalField::new(4, CANONICAL_TYPE_BYTES, self.action_map.canonical_bytes()?),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_BYTES,
                self.context_stack.canonical_bytes()?,
            ),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_OPTIONAL,
                self.last_logical_frame_sequence
                    .map_or_else(Vec::new, |value| value.to_le_bytes().to_vec()),
            ),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_SEQUENCE,
                encode_records(
                    self.source_cursors
                        .iter()
                        .map(|(key, cursor)| encode_source_cursor(key, cursor)),
                )?,
            ),
            CanonicalField::new(
                8,
                CANONICAL_TYPE_SEQUENCE,
                encode_records(
                    self.held_controls
                        .iter()
                        .map(|(identity, value)| encode_held_control(identity, value)),
                )?,
            ),
            CanonicalField::new(
                9,
                CANONICAL_TYPE_SEQUENCE,
                encode_records(
                    self.active_actions
                        .iter()
                        .map(|(action_id, value)| encode_active_action(action_id, *value)),
                )?,
            ),
        ];
        if version == RECOVERY_SCHEMA_VERSION_V2 {
            let cancelled_action_ids = self
                .cancelled_action_ids
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            fields.push(CanonicalField::new(
                10,
                CANONICAL_TYPE_SEQUENCE,
                encode_identifier_sequence(&cancelled_action_ids, self.action_map.actions.len())?,
            ));
        }
        Ok(encode_canonical_segment(
            RECOVERY_OWNER_ID,
            schema_id,
            segment_id,
            fields,
        )?)
    }

    pub fn restore_from_recovery_bytes(bytes: &[u8]) -> Result<Self, PlayerInputError> {
        let limits = CanonicalDecodeLimits::default();
        let segment = decode_canonical_segment(bytes, limits)
            .map_err(|_| PlayerInputError::RecoveryInvalid)?;
        let version = match (segment.schema_id.as_str(), segment.segment_id.as_str()) {
            (RECOVERY_SCHEMA_ID_V1, RECOVERY_SEGMENT_ID_V1) => {
                ensure_segment(
                    &segment,
                    RECOVERY_SCHEMA_ID_V1,
                    RECOVERY_SEGMENT_ID_V1,
                    RECOVERY_FIELD_COUNT_V1,
                )?;
                RECOVERY_SCHEMA_VERSION_V1
            }
            (RECOVERY_SCHEMA_ID_V2, RECOVERY_SEGMENT_ID_V2) => {
                ensure_segment(
                    &segment,
                    RECOVERY_SCHEMA_ID_V2,
                    RECOVERY_SEGMENT_ID_V2,
                    RECOVERY_FIELD_COUNT_V2,
                )?;
                RECOVERY_SCHEMA_VERSION_V2
            }
            _ => return Err(PlayerInputError::RecoveryInvalid),
        };
        if decode_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)? != version {
            return Err(PlayerInputError::RecoveryInvalid);
        }
        let controller_id =
            PersistentId::from_bytes(decode_array(field(&segment, 2, CANONICAL_TYPE_ID128)?)?);
        let source_id =
            InputSourceId::from_bytes(decode_array(field(&segment, 3, CANONICAL_TYPE_ID128)?)?);
        let action_map = ActionMapManifestV1::from_canonical_bytes(
            field(&segment, 4, CANONICAL_TYPE_BYTES)?,
            limits,
        )
        .map_err(|_| PlayerInputError::RecoveryInvalid)?;
        let context_stack = InputContextStackV1::from_canonical_bytes(
            field(&segment, 5, CANONICAL_TYPE_BYTES)?,
            limits,
        )
        .map_err(|_| PlayerInputError::RecoveryInvalid)?;
        validate_context_compatibility(&action_map, &context_stack)
            .map_err(|_| PlayerInputError::RecoveryInvalid)?;
        let last_logical_frame_sequence =
            decode_optional_u64(field(&segment, 6, CANONICAL_TYPE_OPTIONAL)?)?;

        let source_records = decode_records(
            field(&segment, 7, CANONICAL_TYPE_SEQUENCE)?,
            MAX_PLATFORM_INPUT_SOURCES,
            limits,
        )?;
        let mut source_cursors = BTreeMap::new();
        for record in source_records {
            let (key, cursor) = decode_source_cursor(record, limits)?;
            if source_cursors.insert(key, cursor).is_some() {
                return Err(PlayerInputError::RecoveryInvalid);
            }
        }
        let held_records = decode_records(
            field(&segment, 8, CANONICAL_TYPE_SEQUENCE)?,
            MAX_HELD_CONTROLS,
            limits,
        )?;
        let mut held_controls = BTreeMap::new();
        for record in held_records {
            let (identity, value) = decode_held_control(record, limits)?;
            if held_controls.insert(identity, value).is_some() {
                return Err(PlayerInputError::RecoveryInvalid);
            }
        }
        let active_records = decode_records(
            field(&segment, 9, CANONICAL_TYPE_SEQUENCE)?,
            action_map.actions.len(),
            limits,
        )?;
        let mut active_actions = BTreeMap::new();
        for record in active_records {
            let (action_id, value) = decode_active_action(record, limits)?;
            if action_map.action(&action_id).is_none()
                || active_actions.insert(action_id, value).is_some()
            {
                return Err(PlayerInputError::RecoveryInvalid);
            }
        }
        let cancelled_action_ids = if version == RECOVERY_SCHEMA_VERSION_V2 {
            decode_identifier_sequence(
                field(&segment, 10, CANONICAL_TYPE_SEQUENCE)?,
                action_map.actions.len(),
            )?
            .into_iter()
            .collect()
        } else {
            BTreeSet::new()
        };

        let value = Self {
            controller_id,
            source_id,
            action_map,
            context_stack,
            pending_action_map: None,
            pending_context_stack: None,
            source_cursors,
            pending_platform_events: Vec::new(),
            last_logical_frame_sequence,
            held_controls,
            started_controls: BTreeSet::new(),
            pending_deltas: BTreeMap::new(),
            active_actions,
            cancelled_action_ids,
            diagnostics: BTreeSet::new(),
        };
        value.validate_recovery_boundary()?;
        value.validate_active_action_closure()?;
        if value.recovery_bytes_for_version(version)? != bytes {
            return Err(PlayerInputError::RecoveryInvalid);
        }
        Ok(value)
    }

    /// Converts controls inherited from a previous host lifetime into normal
    /// one-shot cancellation actions at the next logical frame boundary and
    /// retires source cursors that the replacement host can never continue.
    ///
    /// A fresh desktop adapter has a new host identity and cannot reliably
    /// deliver key-up events for controls that were held by the dead process.
    pub fn cancel_recovered_controls(&mut self) -> Result<(), PlayerInputError> {
        self.validate_recovery_boundary()?;
        self.cancel_all_active();
        self.source_cursors.clear();
        Ok(())
    }

    fn validate_recovery_boundary(&self) -> Result<(), PlayerInputError> {
        if self.pending_action_map.is_some()
            || self.pending_context_stack.is_some()
            || !self.pending_platform_events.is_empty()
            || !self.started_controls.is_empty()
            || !self.pending_deltas.is_empty()
            || !self.diagnostics.is_empty()
            || self.source_cursors.len() > MAX_PLATFORM_INPUT_SOURCES
            || self.held_controls.len() > MAX_HELD_CONTROLS
            || self.active_actions.len() > self.action_map.actions.len()
            || self.cancelled_action_ids.len() > self.action_map.actions.len()
        {
            return Err(PlayerInputError::RecoveryInvalid);
        }
        self.action_map
            .validate()
            .map_err(|_| PlayerInputError::RecoveryInvalid)?;
        self.context_stack
            .validate()
            .map_err(|_| PlayerInputError::RecoveryInvalid)?;
        validate_context_compatibility(&self.action_map, &self.context_stack)
            .map_err(|_| PlayerInputError::RecoveryInvalid)?;
        self.validate_active_action_closure()
    }

    fn validate_active_action_closure(&self) -> Result<(), PlayerInputError> {
        if !self.cancelled_action_ids.is_empty() {
            let active_action_ids = self.active_actions.keys().cloned().collect::<BTreeSet<_>>();
            if !self.held_controls.is_empty()
                || self.cancelled_action_ids != active_action_ids
                || self
                    .cancelled_action_ids
                    .iter()
                    .any(|action_id| self.action_map.action(action_id).is_none())
            {
                return Err(PlayerInputError::RecoveryInvalid);
            }
            return Ok(());
        }
        let persisted = self.active_actions.clone();
        let mut rebuilt = self.clone();
        rebuilt.active_actions.clear();
        rebuilt
            .resolve_actions()
            .map_err(|_| PlayerInputError::RecoveryInvalid)?;
        if rebuilt.active_actions != persisted {
            return Err(PlayerInputError::RecoveryInvalid);
        }
        Ok(())
    }
}

fn encode_source_cursor(
    key: &PlatformSourceKey,
    cursor: &PlatformSourceCursor,
) -> Result<Vec<u8>, PlayerInputError> {
    Ok(encode_canonical_segment(
        RECOVERY_OWNER_ID,
        SOURCE_CURSOR_SCHEMA_ID,
        RECORD_SEGMENT_ID,
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_ID128,
                key.host_instance_id.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_UTF8_NFC,
                key.source_class.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_U64,
                cursor.source_sequence.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_HASH256,
                cursor.last_event_id.as_bytes().to_vec(),
            ),
        ],
    )?)
}

fn decode_source_cursor(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<(PlatformSourceKey, PlatformSourceCursor), PlayerInputError> {
    let segment =
        decode_canonical_segment(bytes, limits).map_err(|_| PlayerInputError::RecoveryInvalid)?;
    ensure_segment(&segment, SOURCE_CURSOR_SCHEMA_ID, RECORD_SEGMENT_ID, 4)?;
    let value = (
        PlatformSourceKey {
            host_instance_id: PersistentId::from_bytes(decode_array(field(
                &segment,
                1,
                CANONICAL_TYPE_ID128,
            )?)?),
            source_class: decode_schema_id(field(&segment, 2, CANONICAL_TYPE_UTF8_NFC)?)?,
        },
        PlatformSourceCursor {
            source_sequence: decode_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?,
            last_event_id: ContentHash::from_bytes(decode_array(field(
                &segment,
                4,
                CANONICAL_TYPE_HASH256,
            )?)?),
        },
    );
    if encode_source_cursor(&value.0, &value.1)? != bytes {
        return Err(PlayerInputError::RecoveryInvalid);
    }
    Ok(value)
}

fn encode_held_control(
    identity: &ControlIdentity,
    value: &[i16],
) -> Result<Vec<u8>, PlayerInputError> {
    if value.is_empty() || value.len() > PLATFORM_MAX_CONTROL_COMPONENTS {
        return Err(PlayerInputError::RecoveryInvalid);
    }
    let mut values = Vec::with_capacity(value.len() * 2);
    for component in value {
        values.extend_from_slice(&component.to_le_bytes());
    }
    Ok(encode_canonical_segment(
        RECOVERY_OWNER_ID,
        HELD_CONTROL_SCHEMA_ID,
        RECORD_SEGMENT_ID,
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                identity.device_class.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_ID128,
                identity.device_instance_nonce.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_UTF8_NFC,
                identity.control_path_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_SEQUENCE,
                encode_identifier_sequence(&identity.modifier_set, PLATFORM_MAX_MODIFIERS)?,
            ),
            CanonicalField::new(5, CANONICAL_TYPE_BYTES, values),
        ],
    )?)
}

fn decode_held_control(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<(ControlIdentity, Vec<i16>), PlayerInputError> {
    let segment =
        decode_canonical_segment(bytes, limits).map_err(|_| PlayerInputError::RecoveryInvalid)?;
    ensure_segment(&segment, HELD_CONTROL_SCHEMA_ID, RECORD_SEGMENT_ID, 5)?;
    let identity = ControlIdentity {
        device_class: decode_schema_id(field(&segment, 1, CANONICAL_TYPE_UTF8_NFC)?)?,
        device_instance_nonce: PersistentId::from_bytes(decode_array(field(
            &segment,
            2,
            CANONICAL_TYPE_ID128,
        )?)?),
        control_path_id: decode_schema_id(field(&segment, 3, CANONICAL_TYPE_UTF8_NFC)?)?,
        modifier_set: decode_identifier_sequence(
            field(&segment, 4, CANONICAL_TYPE_SEQUENCE)?,
            PLATFORM_MAX_MODIFIERS,
        )?,
    };
    let value_bytes = field(&segment, 5, CANONICAL_TYPE_BYTES)?;
    if value_bytes.is_empty()
        || value_bytes.len() % 2 != 0
        || value_bytes.len() / 2 > PLATFORM_MAX_CONTROL_COMPONENTS
    {
        return Err(PlayerInputError::RecoveryInvalid);
    }
    let value = value_bytes
        .chunks_exact(2)
        .map(|bytes| {
            i16::from_le_bytes(
                bytes
                    .try_into()
                    .expect("chunks_exact(2) always yields two bytes"),
            )
        })
        .collect::<Vec<_>>();
    if encode_held_control(&identity, &value)? != bytes {
        return Err(PlayerInputError::RecoveryInvalid);
    }
    Ok((identity, value))
}

fn encode_active_action(
    action_id: &SchemaId,
    value: PlayerActionValueV1,
) -> Result<Vec<u8>, PlayerInputError> {
    let (tag, bytes) = encode_action_value(value);
    Ok(encode_canonical_segment(
        RECOVERY_OWNER_ID,
        ACTIVE_ACTION_SCHEMA_ID,
        RECORD_SEGMENT_ID,
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                action_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(2, CANONICAL_TYPE_U8, vec![tag]),
            CanonicalField::new(3, CANONICAL_TYPE_BYTES, bytes),
        ],
    )?)
}

fn decode_active_action(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<(SchemaId, PlayerActionValueV1), PlayerInputError> {
    let segment =
        decode_canonical_segment(bytes, limits).map_err(|_| PlayerInputError::RecoveryInvalid)?;
    ensure_segment(&segment, ACTIVE_ACTION_SCHEMA_ID, RECORD_SEGMENT_ID, 3)?;
    let action_id = decode_schema_id(field(&segment, 1, CANONICAL_TYPE_UTF8_NFC)?)?;
    let tag = decode_u8(field(&segment, 2, CANONICAL_TYPE_U8)?)?;
    let value = decode_action_value(tag, field(&segment, 3, CANONICAL_TYPE_BYTES)?)?;
    if encode_active_action(&action_id, value)? != bytes {
        return Err(PlayerInputError::RecoveryInvalid);
    }
    Ok((action_id, value))
}

fn encode_action_value(value: PlayerActionValueV1) -> (u8, Vec<u8>) {
    match value {
        PlayerActionValueV1::Digital(value) => (1, vec![u8::from(value)]),
        PlayerActionValueV1::ScalarQ15(value) => (2, value.to_le_bytes().to_vec()),
        PlayerActionValueV1::Vector2Q15(value) => {
            let mut bytes = Vec::with_capacity(4);
            bytes.extend_from_slice(&value[0].to_le_bytes());
            bytes.extend_from_slice(&value[1].to_le_bytes());
            (3, bytes)
        }
    }
}

fn decode_action_value(tag: u8, bytes: &[u8]) -> Result<PlayerActionValueV1, PlayerInputError> {
    match (tag, bytes) {
        (1, [0]) => Ok(PlayerActionValueV1::Digital(false)),
        (1, [1]) => Ok(PlayerActionValueV1::Digital(true)),
        (2, bytes) if bytes.len() == 2 => Ok(PlayerActionValueV1::ScalarQ15(i16::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| PlayerInputError::RecoveryInvalid)?,
        ))),
        (3, bytes) if bytes.len() == 4 => Ok(PlayerActionValueV1::Vector2Q15([
            i16::from_le_bytes(
                bytes[..2]
                    .try_into()
                    .map_err(|_| PlayerInputError::RecoveryInvalid)?,
            ),
            i16::from_le_bytes(
                bytes[2..]
                    .try_into()
                    .map_err(|_| PlayerInputError::RecoveryInvalid)?,
            ),
        ])),
        _ => Err(PlayerInputError::RecoveryInvalid),
    }
}

fn encode_records(
    records: impl IntoIterator<Item = Result<Vec<u8>, PlayerInputError>>,
) -> Result<Vec<u8>, PlayerInputError> {
    let records = records.into_iter().collect::<Result<Vec<_>, _>>()?;
    let mut bytes = Vec::new();
    extend_u32(
        &mut bytes,
        u32::try_from(records.len()).map_err(|_| PlayerInputError::RecoveryInvalid)?,
    );
    for record in records {
        extend_u32(
            &mut bytes,
            u32::try_from(record.len()).map_err(|_| PlayerInputError::RecoveryInvalid)?,
        );
        bytes.extend_from_slice(&record);
    }
    Ok(bytes)
}

fn decode_records(
    bytes: &[u8],
    maximum: usize,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<&[u8]>, PlayerInputError> {
    let mut cursor = RecoveryCursor::new(bytes);
    let count = cursor.read_count(maximum)?;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        records.push(cursor.read_length_prefixed(limits.max_field_payload_bytes)?);
    }
    cursor.finish()?;
    Ok(records)
}

fn encode_identifier_sequence(
    values: &[SchemaId],
    maximum: usize,
) -> Result<Vec<u8>, PlayerInputError> {
    if values.len() > maximum || values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(PlayerInputError::RecoveryInvalid);
    }
    let mut bytes = Vec::new();
    extend_u32(
        &mut bytes,
        u32::try_from(values.len()).map_err(|_| PlayerInputError::RecoveryInvalid)?,
    );
    for value in values {
        let encoded = value.as_str().as_bytes();
        extend_u32(
            &mut bytes,
            u32::try_from(encoded.len()).map_err(|_| PlayerInputError::RecoveryInvalid)?,
        );
        bytes.extend_from_slice(encoded);
    }
    Ok(bytes)
}

fn decode_identifier_sequence(
    bytes: &[u8],
    maximum: usize,
) -> Result<Vec<SchemaId>, PlayerInputError> {
    let mut cursor = RecoveryCursor::new(bytes);
    let count = cursor.read_count(maximum)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let value =
            cursor.read_length_prefixed(CanonicalDecodeLimits::default().max_identifier_bytes)?;
        let value = std::str::from_utf8(value).map_err(|_| PlayerInputError::RecoveryInvalid)?;
        values.push(SchemaId::new(value).map_err(|_| PlayerInputError::RecoveryInvalid)?);
    }
    cursor.finish()?;
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(PlayerInputError::RecoveryInvalid);
    }
    Ok(values)
}

fn ensure_segment(
    segment: &DecodedCanonicalSegment,
    schema_id: &str,
    segment_id: &str,
    field_count: usize,
) -> Result<(), PlayerInputError> {
    if segment.owner_id != RECOVERY_OWNER_ID
        || segment.schema_id != schema_id
        || segment.segment_id != segment_id
        || segment.fields.len() != field_count
        || segment
            .fields
            .iter()
            .enumerate()
            .any(|(index, field)| field.field_id != u32::try_from(index + 1).unwrap_or(u32::MAX))
    {
        return Err(PlayerInputError::RecoveryInvalid);
    }
    Ok(())
}

fn field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
    type_tag: u8,
) -> Result<&[u8], PlayerInputError> {
    let field = segment
        .field(field_id)
        .ok_or(PlayerInputError::RecoveryInvalid)?;
    if field.type_tag != type_tag {
        return Err(PlayerInputError::RecoveryInvalid);
    }
    Ok(&field.payload)
}

fn decode_schema_id(bytes: &[u8]) -> Result<SchemaId, PlayerInputError> {
    let value = std::str::from_utf8(bytes).map_err(|_| PlayerInputError::RecoveryInvalid)?;
    SchemaId::new(value).map_err(|_| PlayerInputError::RecoveryInvalid)
}

fn decode_u8(bytes: &[u8]) -> Result<u8, PlayerInputError> {
    match bytes {
        [value] => Ok(*value),
        _ => Err(PlayerInputError::RecoveryInvalid),
    }
}

fn decode_u32(bytes: &[u8]) -> Result<u32, PlayerInputError> {
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| PlayerInputError::RecoveryInvalid)?,
    ))
}

fn decode_u64(bytes: &[u8]) -> Result<u64, PlayerInputError> {
    Ok(u64::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| PlayerInputError::RecoveryInvalid)?,
    ))
}

fn decode_optional_u64(bytes: &[u8]) -> Result<Option<u64>, PlayerInputError> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        decode_u64(bytes).map(Some)
    }
}

fn decode_array<const N: usize>(bytes: &[u8]) -> Result<[u8; N], PlayerInputError> {
    bytes
        .try_into()
        .map_err(|_| PlayerInputError::RecoveryInvalid)
}

fn extend_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

struct RecoveryCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> RecoveryCursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_count(&mut self, maximum: usize) -> Result<usize, PlayerInputError> {
        let value =
            usize::try_from(self.read_u32()?).map_err(|_| PlayerInputError::RecoveryInvalid)?;
        if value > maximum {
            return Err(PlayerInputError::RecoveryInvalid);
        }
        Ok(value)
    }

    fn read_length_prefixed(&mut self, maximum: usize) -> Result<&'a [u8], PlayerInputError> {
        let length =
            usize::try_from(self.read_u32()?).map_err(|_| PlayerInputError::RecoveryInvalid)?;
        if length > maximum {
            return Err(PlayerInputError::RecoveryInvalid);
        }
        self.read_exact(length)
    }

    fn read_u32(&mut self) -> Result<u32, PlayerInputError> {
        Ok(u32::from_le_bytes(
            self.read_exact(4)?
                .try_into()
                .map_err(|_| PlayerInputError::RecoveryInvalid)?,
        ))
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], PlayerInputError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(PlayerInputError::RecoveryInvalid)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(PlayerInputError::RecoveryInvalid)?;
        self.offset = end;
        Ok(value)
    }

    fn finish(self) -> Result<(), PlayerInputError> {
        if self.offset != self.bytes.len() {
            return Err(PlayerInputError::RecoveryInvalid);
        }
        Ok(())
    }
}
