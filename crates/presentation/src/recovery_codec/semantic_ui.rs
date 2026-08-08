use super::*;

pub(super) fn encode_semantic_ui_record(
    index: usize,
    record: &SemanticUiPresentationRecordV1,
) -> Result<Vec<u8>, ()> {
    record.validate().map_err(|_| ())?;
    let text_or_none = record
        .element
        .text_or_none
        .as_ref()
        .map_or_else(|| Ok(Vec::new()), encode_ui_text_ref)?;
    let affordances = encode_sequence(
        record
            .element
            .affordances
            .iter()
            .map(encode_ui_affordance)
            .collect(),
    )?;
    encode_canonical_segment(
        RECOVERY_OWNER,
        SEMANTIC_UI_RECORD_SCHEMA,
        &format!("semantic-ui-{index}"),
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                record.schema_version.to_le_bytes().to_vec(),
            ),
            hash_field(2, record.snapshot_epoch),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_BYTES,
                record.surface_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_BYTES,
                record.semantic_path_id.as_str().as_bytes().to_vec(),
            ),
            hash_field(5, record.source_snapshot_hash),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_BYTES,
                record.element.element_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(7, CANONICAL_TYPE_U8, vec![record.element.role as u8]),
            CanonicalField::new(8, CANONICAL_TYPE_U8, vec![record.element.style_role as u8]),
            CanonicalField::new(
                9,
                CANONICAL_TYPE_U8,
                vec![record.element.accessibility_role as u8],
            ),
            CanonicalField::new(
                10,
                CANONICAL_TYPE_BOOL,
                vec![u8::from(record.element.enabled)],
            ),
            CanonicalField::new(
                11,
                CANONICAL_TYPE_BOOL,
                vec![u8::from(record.element.visible)],
            ),
            CanonicalField::new(
                12,
                CANONICAL_TYPE_BOOL,
                vec![u8::from(record.element.selected)],
            ),
            CanonicalField::new(13, CANONICAL_TYPE_OPTIONAL, text_or_none),
            CanonicalField::new(
                14,
                CANONICAL_TYPE_BYTES,
                encode_ui_value(record.element.value),
            ),
            CanonicalField::new(15, CANONICAL_TYPE_SEQUENCE, affordances),
            hash_field(16, record.element.canonical_hash),
            hash_field(17, record.canonical_hash),
        ],
    )
    .map_err(|_| ())
}

pub(super) fn decode_semantic_ui_record(
    index: usize,
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<SemanticUiPresentationRecordV1, ()> {
    let segment = decode_canonical_segment(bytes, limits).map_err(|_| ())?;
    ensure_segment(
        &segment,
        RECOVERY_OWNER,
        SEMANTIC_UI_RECORD_SCHEMA,
        &format!("semantic-ui-{index}"),
        17,
    )?;
    let text_or_none = {
        let payload = field(&segment, 13, CANONICAL_TYPE_OPTIONAL)?;
        if payload.is_empty() {
            None
        } else {
            Some(decode_ui_text_ref(payload, limits)?)
        }
    };
    let element = UiSemanticElementV1 {
        element_id: decode_schema_id(field(&segment, 6, CANONICAL_TYPE_BYTES)?)?,
        role: decode_ui_element_role(decode_u8(field(&segment, 7, CANONICAL_TYPE_U8)?)?)?,
        style_role: decode_ui_style_role(decode_u8(field(&segment, 8, CANONICAL_TYPE_U8)?)?)?,
        accessibility_role: decode_ui_accessibility_role(decode_u8(field(
            &segment,
            9,
            CANONICAL_TYPE_U8,
        )?)?)?,
        enabled: decode_bool(field(&segment, 10, CANONICAL_TYPE_BOOL)?)?,
        visible: decode_bool(field(&segment, 11, CANONICAL_TYPE_BOOL)?)?,
        selected: decode_bool(field(&segment, 12, CANONICAL_TYPE_BOOL)?)?,
        text_or_none,
        value: decode_ui_value(field(&segment, 14, CANONICAL_TYPE_BYTES)?)?,
        affordances: decode_sequence(
            field(&segment, 15, CANONICAL_TYPE_SEQUENCE)?,
            UI_MAX_AFFORDANCES_PER_ELEMENT,
            limits.max_field_payload_bytes,
        )?
        .into_iter()
        .map(decode_ui_affordance)
        .collect::<Result<Vec<_>, _>>()?,
        canonical_hash: decode_hash(field(&segment, 16, CANONICAL_TYPE_HASH256)?)?,
    };
    element.validate().map_err(|_| ())?;
    let record = SemanticUiPresentationRecordV1 {
        schema_version: decode_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)?,
        snapshot_epoch: decode_hash(field(&segment, 2, CANONICAL_TYPE_HASH256)?)?,
        surface_id: decode_schema_id(field(&segment, 3, CANONICAL_TYPE_BYTES)?)?,
        semantic_path_id: decode_schema_id(field(&segment, 4, CANONICAL_TYPE_BYTES)?)?,
        source_snapshot_hash: decode_hash(field(&segment, 5, CANONICAL_TYPE_HASH256)?)?,
        element,
        canonical_hash: decode_hash(field(&segment, 17, CANONICAL_TYPE_HASH256)?)?,
    };
    if record.schema_version != SEMANTIC_UI_PRESENTATION_RECORD_SCHEMA_VERSION {
        return Err(());
    }
    record.validate().map_err(|_| ())?;
    if encode_semantic_ui_record(index, &record)? != bytes {
        return Err(());
    }
    Ok(record)
}

fn encode_ui_text_ref(text: &UiTextRefV1) -> Result<Vec<u8>, ()> {
    text.validate().map_err(|_| ())?;
    encode_canonical_segment(
        RECOVERY_OWNER,
        UI_TEXT_REF_SCHEMA,
        "text",
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_BYTES,
                text.text_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(text.arguments.iter().map(encode_ui_text_argument).collect())?,
            ),
        ],
    )
    .map_err(|_| ())
}

fn decode_ui_text_ref(bytes: &[u8], limits: CanonicalDecodeLimits) -> Result<UiTextRefV1, ()> {
    let segment = decode_canonical_segment(bytes, limits).map_err(|_| ())?;
    ensure_segment(&segment, RECOVERY_OWNER, UI_TEXT_REF_SCHEMA, "text", 2)?;
    let text = UiTextRefV1 {
        text_id: decode_schema_id(field(&segment, 1, CANONICAL_TYPE_BYTES)?)?,
        arguments: decode_sequence(
            field(&segment, 2, CANONICAL_TYPE_SEQUENCE)?,
            UI_MAX_TEXT_ARGUMENTS,
            limits.max_field_payload_bytes,
        )?
        .into_iter()
        .map(decode_ui_text_argument)
        .collect::<Result<Vec<_>, _>>()?,
    };
    text.validate().map_err(|_| ())?;
    if encode_ui_text_ref(&text)? != bytes {
        return Err(());
    }
    Ok(text)
}

fn encode_ui_text_argument(argument: &UiTextArgumentV1) -> Vec<u8> {
    match argument {
        UiTextArgumentV1::SignedInteger(value) => {
            let mut bytes = Vec::with_capacity(9);
            bytes.push(0);
            bytes.extend_from_slice(&value.to_le_bytes());
            bytes
        }
        UiTextArgumentV1::TextId(text_id) => {
            let mut bytes = Vec::with_capacity(text_id.as_str().len() + 1);
            bytes.push(1);
            bytes.extend_from_slice(text_id.as_str().as_bytes());
            bytes
        }
    }
}

fn decode_ui_text_argument(bytes: &[u8]) -> Result<UiTextArgumentV1, ()> {
    let (&kind, payload) = bytes.split_first().ok_or(())?;
    match kind {
        0 => Ok(UiTextArgumentV1::SignedInteger(i64::from_le_bytes(
            payload.try_into().map_err(|_| ())?,
        ))),
        1 => Ok(UiTextArgumentV1::TextId(decode_schema_id(payload)?)),
        _ => Err(()),
    }
}

fn encode_ui_value(value: UiElementValueV1) -> Vec<u8> {
    match value {
        UiElementValueV1::None => vec![0],
        UiElementValueV1::Scalar { current, maximum } => {
            let mut bytes = Vec::with_capacity(17);
            bytes.push(1);
            bytes.extend_from_slice(&current.to_le_bytes());
            bytes.extend_from_slice(&maximum.to_le_bytes());
            bytes
        }
    }
}

fn decode_ui_value(bytes: &[u8]) -> Result<UiElementValueV1, ()> {
    if bytes == [0] {
        return Ok(UiElementValueV1::None);
    }
    if bytes.len() == 17 && bytes[0] == 1 {
        return Ok(UiElementValueV1::Scalar {
            current: i64::from_le_bytes(bytes[1..9].try_into().map_err(|_| ())?),
            maximum: i64::from_le_bytes(bytes[9..17].try_into().map_err(|_| ())?),
        });
    }
    Err(())
}

fn encode_ui_affordance(affordance: &UiActionAffordanceV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(affordance.action_id.as_str().len() + 1);
    bytes.extend_from_slice(affordance.action_id.as_str().as_bytes());
    bytes.push(u8::from(affordance.enabled));
    bytes
}

fn decode_ui_affordance(bytes: &[u8]) -> Result<UiActionAffordanceV1, ()> {
    let (&enabled, action_id) = bytes.split_last().ok_or(())?;
    let enabled = match enabled {
        0 => false,
        1 => true,
        _ => return Err(()),
    };
    Ok(UiActionAffordanceV1 {
        action_id: decode_schema_id(action_id)?,
        enabled,
    })
}

fn decode_schema_id(bytes: &[u8]) -> Result<SchemaId, ()> {
    SchemaId::new(String::from_utf8(bytes.to_vec()).map_err(|_| ())?).map_err(|_| ())
}

fn decode_ui_element_role(value: u8) -> Result<UiElementRoleV1, ()> {
    match value {
        0 => Ok(UiElementRoleV1::Label),
        1 => Ok(UiElementRoleV1::Value),
        2 => Ok(UiElementRoleV1::Meter),
        3 => Ok(UiElementRoleV1::Button),
        4 => Ok(UiElementRoleV1::List),
        5 => Ok(UiElementRoleV1::ListItem),
        6 => Ok(UiElementRoleV1::Panel),
        7 => Ok(UiElementRoleV1::Hint),
        8 => Ok(UiElementRoleV1::Subtitle),
        _ => Err(()),
    }
}

fn decode_ui_style_role(value: u8) -> Result<UiStyleRoleV1, ()> {
    match value {
        0 => Ok(UiStyleRoleV1::Default),
        1 => Ok(UiStyleRoleV1::Muted),
        2 => Ok(UiStyleRoleV1::Accent),
        3 => Ok(UiStyleRoleV1::Warning),
        4 => Ok(UiStyleRoleV1::Danger),
        _ => Err(()),
    }
}

fn decode_ui_accessibility_role(value: u8) -> Result<UiAccessibilityRoleV1, ()> {
    match value {
        0 => Ok(UiAccessibilityRoleV1::Standard),
        1 => Ok(UiAccessibilityRoleV1::Heading),
        2 => Ok(UiAccessibilityRoleV1::Status),
        3 => Ok(UiAccessibilityRoleV1::Alert),
        _ => Err(()),
    }
}
