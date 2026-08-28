use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgContractErrorV1 {
    Canonical(CanonicalDecodeError),
    Canonicalize(CanonicalError),
    Identifier(IdentifierError),
    UnsupportedSchemaVersion(u32),
    UnknownAggregateKind(u8),
    UnknownOperationTag(u8),
    UnknownEventTag(u8),
    InputTooLarge,
    EnvelopeMismatch,
    FieldSetMismatch,
    NonCanonicalEncoding,
    ZeroSchemaVersion,
    PayloadKindMismatch,
    PayloadHashMismatch,
    CollectionLimitExceeded,
    AggregateOrderInvalid,
    OperationCountInvalid,
    OperationOrderInvalid,
    TargetSetInvalid,
    DefinitionPolicySetInvalid,
    PhysicalFactInvalid,
    InvalidTag(u8),
    PayloadInvariant(&'static str),
    PlanOrderInvalid,
    PlanWriteInvalid,
    PlanHashMismatch,
    EventOrderInvalid,
    RevisionExhausted,
}

impl RpgContractErrorV1 {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion(_) => "RPG_SCHEMA_UNSUPPORTED",
            Self::UnknownAggregateKind(_) | Self::PayloadKindMismatch => {
                "RPG_AGGREGATE_KIND_INVALID"
            }
            Self::UnknownOperationTag(_) | Self::OperationCountInvalid => "RPG_OPERATION_INVALID",
            Self::UnknownEventTag(_) => "RPG_EVENT_ORDER_INVALID",
            Self::InputTooLarge | Self::CollectionLimitExceeded => "RPG_INPUT_LIMIT_EXCEEDED",
            Self::EnvelopeMismatch | Self::FieldSetMismatch => "RPG_SCHEMA_INVALID",
            Self::NonCanonicalEncoding => "RPG_NON_CANONICAL",
            Self::ZeroSchemaVersion => "RPG_SCHEMA_INVALID",
            Self::PayloadHashMismatch => "RPG_PAYLOAD_HASH_MISMATCH",
            Self::AggregateOrderInvalid | Self::PlanOrderInvalid => "RPG_ORDER_INVALID",
            Self::OperationOrderInvalid => "RPG_OPERATION_ORDER_INVALID",
            Self::TargetSetInvalid => "RPG_TARGET_SET_INVALID",
            Self::DefinitionPolicySetInvalid => "RPG_DEFINITION_MISMATCH",
            Self::PhysicalFactInvalid => "RPG_PHYSICAL_PRECONDITION_MISSING",
            Self::InvalidTag(_) => "RPG_SCHEMA_INVALID",
            Self::PayloadInvariant(code) => code,
            Self::PlanWriteInvalid => "RPG_PLAN_WRITE_INVALID",
            Self::PlanHashMismatch => "RPG_PLAN_HASH_MISMATCH",
            Self::EventOrderInvalid => "RPG_EVENT_ORDER_INVALID",
            Self::RevisionExhausted => "RPG_REVISION_EXHAUSTED",
            Self::Canonical(_) | Self::Canonicalize(_) | Self::Identifier(_) => {
                "RPG_CANONICALIZATION_FAILED"
            }
        }
    }
}

impl Display for RpgContractErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "{}: {version}", self.stable_code())
            }
            Self::UnknownAggregateKind(tag)
            | Self::UnknownOperationTag(tag)
            | Self::UnknownEventTag(tag) => {
                write!(formatter, "{}: {tag}", self.stable_code())
            }
            Self::InvalidTag(tag) => write!(formatter, "{}: {tag}", self.stable_code()),
            Self::Canonical(error) => write!(formatter, "{}: {error}", self.stable_code()),
            Self::Canonicalize(error) => write!(formatter, "{}: {error}", self.stable_code()),
            Self::Identifier(error) => write!(formatter, "{}: {error}", self.stable_code()),
            _ => formatter.write_str(self.stable_code()),
        }
    }
}

impl Error for RpgContractErrorV1 {}

impl From<CanonicalDecodeError> for RpgContractErrorV1 {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for RpgContractErrorV1 {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalize(error)
    }
}

impl From<IdentifierError> for RpgContractErrorV1 {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

pub(super) fn validate_payload(payload: &RpgAggregatePayloadV1) -> Result<(), RpgContractErrorV1> {
    match payload {
        RpgAggregatePayloadV1::Character(payload) => {
            if payload.resources.len() > RPG_MAX_COLLECTION_ENTRIES
                || !strictly_ordered_by(&payload.resources, |entry| entry.resource_id.clone())
                || payload.resources.iter().any(|entry| {
                    entry.minimum_value > entry.maximum_value
                        || entry.current_value < entry.minimum_value
                        || entry.current_value > entry.maximum_value
                })
                || !strictly_ordered_by(&payload.skills, |entry| entry.skill_id.clone())
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_CHARACTER_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::Item(payload) => {
            if payload.quantity == 0 {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_ITEM_QUANTITY_ZERO",
                ));
            }
        }
        RpgAggregatePayloadV1::Inventory(payload) => {
            if payload.item_ids.len() > RPG_MAX_COLLECTION_ENTRIES
                || payload.reservations.len() > RPG_MAX_COLLECTION_ENTRIES
                || !strictly_ordered_unique(&payload.item_ids)
                || !strictly_ordered_by(&payload.reservations, |entry| entry.reservation_id)
                || payload.reservations.iter().any(|entry| entry.quantity == 0)
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_INVENTORY_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::Equipment(payload) => {
            if payload.assignments.len() > RPG_MAX_COLLECTION_ENTRIES
                || !strictly_ordered_by(&payload.assignments, |entry| entry.slot_id.clone())
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_EQUIPMENT_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::Faction(payload) => {
            if payload.directed_policy_refs.len() > RPG_MAX_COLLECTION_ENTRIES
                || !strictly_ordered_unique(&payload.directed_policy_refs)
            {
                return Err(RpgContractErrorV1::PayloadInvariant("RPG_FACTION_INVALID"));
            }
        }
        RpgAggregatePayloadV1::FactionMembership(payload) => {
            if payload.character_id == payload.faction_id {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_FACTION_MEMBERSHIP_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::Relationship(payload) => {
            if payload.source_id == payload.target_id
                || !strictly_ordered_by(&payload.dimensions, |entry| entry.dimension_id.clone())
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_RELATIONSHIP_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::DivineStanding(payload) => {
            if payload.offer_ids.len() > RPG_MAX_COLLECTION_ENTRIES
                || payload.warning_ids.len() > RPG_MAX_COLLECTION_ENTRIES
                || !strictly_ordered_unique(&payload.offer_ids)
                || !strictly_ordered_unique(&payload.warning_ids)
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_DIVINE_STANDING_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::Commitment(payload) => {
            if payload.issuer_character_id == payload.recipient_character_id
                || payload.wage_amount <= 0
            {
                return Err(RpgContractErrorV1::PayloadInvariant(
                    "RPG_COMMITMENT_INVALID",
                ));
            }
        }
        RpgAggregatePayloadV1::BodyCondition(payload) => payload.validate()?,
        RpgAggregatePayloadV1::Quest(_)
        | RpgAggregatePayloadV1::Dialogue(_)
        | RpgAggregatePayloadV1::InteractiveObject(_) => {}
    }
    Ok(())
}

pub(super) fn validate_event_drafts(
    operations: &[RpgOperationV1],
    drafts: &[RpgEventDraftV1],
) -> Result<(), RpgContractErrorV1> {
    for operation in operations {
        let operation_drafts = drafts
            .iter()
            .filter(|draft| draft.operation_slot == operation.operation_slot)
            .collect::<Vec<_>>();
        if operation_drafts.is_empty() {
            return Err(RpgContractErrorV1::EventOrderInvalid);
        }
        for (expected_slot, draft) in operation_drafts.into_iter().enumerate() {
            let expected_slot =
                u16::try_from(expected_slot).map_err(|_| RpgContractErrorV1::EventOrderInvalid)?;
            let primary = draft.event.primary_aggregate();
            if draft.event_local_slot != expected_slot
                || draft.event_schema_id.as_str() != draft.event.schema_id()
                || (draft.primary_aggregate_kind, draft.primary_persistent_id) != primary
                || !operation
                    .targets
                    .iter()
                    .any(|target| (target.aggregate_kind, target.persistent_id) == primary)
            {
                return Err(RpgContractErrorV1::EventOrderInvalid);
            }
        }
    }
    if drafts.iter().any(|draft| {
        usize::try_from(draft.operation_slot).map_or(true, |slot| slot >= operations.len())
    }) {
        return Err(RpgContractErrorV1::EventOrderInvalid);
    }
    Ok(())
}

pub(super) fn extend_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), CanonicalError> {
    let count = u32::try_from(count).map_err(|_| CanonicalError::LengthOverflow)?;
    bytes.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

pub(super) fn extend_schema_id(
    bytes: &mut Vec<u8>,
    value: &SchemaId,
) -> Result<(), CanonicalError> {
    extend_u32_length_prefixed(bytes, value.as_str().as_bytes())
}

pub(super) fn extend_ids(
    bytes: &mut Vec<u8>,
    values: &[PersistentId],
) -> Result<(), CanonicalError> {
    extend_count(bytes, values.len())?;
    for value in values {
        bytes.extend_from_slice(value.as_bytes());
    }
    Ok(())
}

pub(super) fn extend_hashes(
    bytes: &mut Vec<u8>,
    values: &[ContentHash],
) -> Result<(), CanonicalError> {
    extend_count(bytes, values.len())?;
    for value in values {
        bytes.extend_from_slice(value.as_bytes());
    }
    Ok(())
}

pub(super) fn extend_definition_ref(bytes: &mut Vec<u8>, value: &DefinitionRefV1) {
    match value {
        DefinitionRefV1::None => bytes.push(0),
        DefinitionRefV1::Exact {
            asset_id,
            content_hash,
        } => {
            bytes.push(1);
            bytes.extend_from_slice(asset_id.as_bytes());
            bytes.extend_from_slice(content_hash.as_bytes());
        }
    }
}

pub(super) fn extend_provenance(bytes: &mut Vec<u8>, value: &ProvenanceBindingV1) {
    match value {
        ProvenanceBindingV1::None => bytes.push(0),
        ProvenanceBindingV1::Exact(value) => {
            bytes.push(1);
            bytes.extend_from_slice(value.as_bytes());
        }
    }
}

pub(super) fn extend_optional_id(bytes: &mut Vec<u8>, value: Option<PersistentId>) {
    match value {
        None => bytes.push(0),
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(value.as_bytes());
        }
    }
}

pub(super) fn read_id(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<PersistentId, CanonicalDecodeError> {
    Ok(PersistentId::from_bytes(read_array(cursor)?))
}

pub(super) fn read_array<const N: usize>(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<[u8; N], CanonicalDecodeError> {
    cursor
        .read_exact(N)?
        .try_into()
        .map_err(|_| CanonicalDecodeError::UnexpectedEnd)
}

pub(super) fn read_schema_id(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<SchemaId, RpgContractErrorV1> {
    let bytes = cursor.read_u32_length_prefixed(limits.max_identifier_bytes)?;
    let text = std::str::from_utf8(bytes).map_err(|_| CanonicalDecodeError::InvalidUtf8)?;
    Ok(SchemaId::new(text)?)
}

pub(super) fn read_optional_id(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<Option<PersistentId>, RpgContractErrorV1> {
    match cursor.read_u8()? {
        0 => Ok(None),
        1 => Ok(Some(read_id(cursor)?)),
        tag => Err(RpgContractErrorV1::InvalidTag(tag)),
    }
}

pub(super) fn read_provenance(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<ProvenanceBindingV1, RpgContractErrorV1> {
    match cursor.read_u8()? {
        0 => Ok(ProvenanceBindingV1::None),
        1 => Ok(ProvenanceBindingV1::Exact(ContentHash::from_bytes(
            read_array(cursor)?,
        ))),
        tag => Err(RpgContractErrorV1::InvalidTag(tag)),
    }
}

pub(super) fn read_definition_ref(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<DefinitionRefV1, RpgContractErrorV1> {
    match cursor.read_u8()? {
        0 => Ok(DefinitionRefV1::None),
        1 => Ok(DefinitionRefV1::Exact {
            asset_id: AssetId::from_bytes(read_array(cursor)?),
            content_hash: ContentHash::from_bytes(read_array(cursor)?),
        }),
        tag => Err(RpgContractErrorV1::InvalidTag(tag)),
    }
}

pub(super) fn read_definition_refs(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<Vec<DefinitionRefV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count).map(|_| read_definition_ref(cursor)).collect()
}

pub(super) fn read_hashes(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<Vec<ContentHash>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| Ok(ContentHash::from_bytes(read_array(cursor)?)))
        .collect()
}

pub(super) fn read_ids(
    cursor: &mut CanonicalCursor<'_>,
    _limits: CanonicalDecodeLimits,
) -> Result<Vec<PersistentId>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count).map(|_| Ok(read_id(cursor)?)).collect()
}

pub(super) fn read_skills(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<SkillProficiencyEntryV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| {
            Ok(SkillProficiencyEntryV1 {
                skill_id: read_schema_id(cursor, limits)?,
                proficiency: SkillProficiency::new(cursor.read_u16()?).map_err(|_| {
                    RpgContractErrorV1::PayloadInvariant("RPG_SKILL_PROFICIENCY_OUT_OF_RANGE")
                })?,
            })
        })
        .collect()
}

pub(super) fn read_resources(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CharacterResourceEntryV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| {
            Ok(CharacterResourceEntryV1 {
                resource_id: read_schema_id(cursor, limits)?,
                current_value: read_i32(cursor)?,
                minimum_value: read_i32(cursor)?,
                maximum_value: read_i32(cursor)?,
            })
        })
        .collect()
}

pub(super) fn read_reservations(
    cursor: &mut CanonicalCursor<'_>,
    _limits: CanonicalDecodeLimits,
) -> Result<Vec<InventoryReservationV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| {
            Ok(InventoryReservationV1 {
                reservation_id: read_id(cursor)?,
                item_id: read_id(cursor)?,
                quantity: cursor.read_u32()?,
            })
        })
        .collect()
}

pub(super) fn read_assignments(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<EquipmentSlotAssignmentV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| {
            Ok(EquipmentSlotAssignmentV1 {
                slot_id: read_schema_id(cursor, limits)?,
                item_id: read_id(cursor)?,
            })
        })
        .collect()
}

pub(super) fn read_dimensions(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<RelationshipDimensionV1>, RpgContractErrorV1> {
    let count = read_bounded_count(cursor, RPG_MAX_COLLECTION_ENTRIES)?;
    (0..count)
        .map(|_| {
            Ok(RelationshipDimensionV1 {
                dimension_id: read_schema_id(cursor, limits)?,
                value: read_i32(cursor)?,
            })
        })
        .collect()
}

pub(super) fn read_i32(cursor: &mut CanonicalCursor<'_>) -> Result<i32, CanonicalDecodeError> {
    Ok(i32::from_le_bytes(read_array(cursor)?))
}

pub(super) fn read_bounded_count(
    cursor: &mut CanonicalCursor<'_>,
    limit: usize,
) -> Result<usize, CanonicalDecodeError> {
    cursor.read_count(limit, |actual, limit| CanonicalDecodeError::InputTooLarge {
        actual,
        limit,
    })
}

pub(super) fn strictly_ordered_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

pub(super) fn strictly_ordered_by<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

pub(super) fn contract_as_canonical(error: RpgContractErrorV1) -> CanonicalError {
    match error {
        RpgContractErrorV1::Canonicalize(error) => error,
        _ => CanonicalError::LengthOverflow,
    }
}
