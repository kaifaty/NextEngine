use crate::PersistentId;
use crate::canonical::{
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U32, CanonicalCursor, CanonicalDecodeError,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, DecodedCanonicalSegment,
    decode_canonical_segment, encode_canonical_segment,
};

use super::model::{
    CharacterSnapshot, DialogueSnapshot, FactionSnapshot, InteractiveObjectSnapshot, ItemSnapshot,
    QuestSnapshot, RPG_SNAPSHOT_OWNER_ID, RPG_SNAPSHOT_SCHEMA_ID, RPG_SNAPSHOT_SCHEMA_VERSION,
    RPG_SNAPSHOT_SEGMENT_ID, RelationshipEntry, RpgDecodeError, RpgSnapshot, SkillProficiency,
    SkillProficiencyEntry, WorldChunkRecordSnapshot,
};
use super::payload::{
    extend_optional_id, extend_text, read_i32, read_id, read_optional_id, read_text, read_u16,
};

impl RpgSnapshot {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RPG_SNAPSHOT_OWNER_ID,
            RPG_SNAPSHOT_SCHEMA_ID,
            RPG_SNAPSHOT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    RPG_SNAPSHOT_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_characters(&self.characters)?,
                ),
                CanonicalField::new(3, CANONICAL_TYPE_SEQUENCE, encode_items(&self.items)?),
                CanonicalField::new(4, CANONICAL_TYPE_SEQUENCE, encode_quests(&self.quests)?),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_dialogues(&self.dialogues)?,
                ),
                CanonicalField::new(6, CANONICAL_TYPE_SEQUENCE, encode_factions(&self.factions)?),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_interactive_objects(&self.interactive_objects)?,
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_world_chunk_records(&self.world_chunk_records)?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RpgDecodeError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_snapshot_envelope(&segment)?;
        validate_snapshot_fields(&segment)?;
        let version = decode_u32_field(&segment, 1)?;
        if version != RPG_SNAPSHOT_SCHEMA_VERSION {
            return Err(RpgDecodeError::UnsupportedSnapshotVersion(version));
        }
        let snapshot = Self {
            characters: decode_characters(field_payload(&segment, 2)?, limits)?,
            items: decode_items(field_payload(&segment, 3)?, limits)?,
            quests: decode_quests(field_payload(&segment, 4)?, limits)?,
            dialogues: decode_dialogues(field_payload(&segment, 5)?, limits)?,
            factions: decode_factions(field_payload(&segment, 6)?, limits)?,
            interactive_objects: decode_interactive_objects(field_payload(&segment, 7)?, limits)?,
            world_chunk_records: decode_world_chunk_records(field_payload(&segment, 8)?, limits)?,
        };
        if snapshot.canonical_bytes()? != bytes {
            return Err(RpgDecodeError::NonCanonicalEncoding);
        }
        Ok(snapshot)
    }
}

fn encode_characters(records: &[CharacterSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.archetype_id)?;

        let mut skills = record.skills.clone();
        skills.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));
        ensure_unique_by(&skills, |entry| entry.skill_id.clone())?;
        extend_count(bytes, skills.len())?;
        for skill in skills {
            extend_text(bytes, &skill.skill_id)?;
            bytes.extend_from_slice(&skill.proficiency.get().to_le_bytes());
        }

        let mut relationships = record.relationships.clone();
        relationships.sort_by(|left, right| {
            (&left.target, &left.dimension_id).cmp(&(&right.target, &right.dimension_id))
        });
        ensure_unique_by(&relationships, |entry| {
            (entry.target, entry.dimension_id.clone())
        })?;
        extend_count(bytes, relationships.len())?;
        for relationship in relationships {
            bytes.extend_from_slice(relationship.target.as_bytes());
            extend_text(bytes, &relationship.dimension_id)?;
            bytes.extend_from_slice(&relationship.value.to_le_bytes());
        }
        Ok(())
    })
}

fn encode_items(records: &[ItemSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.archetype_id)?;
        extend_optional_id(bytes, record.owner);
        bytes.extend_from_slice(&record.quantity.to_le_bytes());
        Ok(())
    })
}

fn encode_quests(records: &[QuestSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.definition_id)?;
        extend_text(bytes, &record.state_id)?;
        Ok(())
    })
}

fn encode_dialogues(records: &[DialogueSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.definition_id)?;
        bytes.extend_from_slice(record.speaker.as_bytes());
        bytes.extend_from_slice(record.listener.as_bytes());
        extend_text(bytes, &record.node_id)?;
        Ok(())
    })
}

fn encode_factions(records: &[FactionSnapshot]) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.definition_id)?;
        let mut members = record.members.clone();
        members.sort();
        ensure_unique_by(&members, |member| *member)?;
        extend_count(bytes, members.len())?;
        for member in members {
            bytes.extend_from_slice(member.as_bytes());
        }
        Ok(())
    })
}

fn encode_interactive_objects(
    records: &[InteractiveObjectSnapshot],
) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.archetype_id)?;
        extend_text(bytes, &record.state_id)?;
        Ok(())
    })
}

fn encode_world_chunk_records(
    records: &[WorldChunkRecordSnapshot],
) -> Result<Vec<u8>, CanonicalError> {
    encode_records(records, |bytes, record| {
        bytes.extend_from_slice(record.id.as_bytes());
        bytes.extend_from_slice(&record.revision.to_le_bytes());
        extend_text(bytes, &record.record_schema_id)?;
        extend_text(bytes, &record.state_id)?;
        Ok(())
    })
}

fn encode_records<T>(
    records: &[T],
    id_and_encode: impl Fn(&mut Vec<u8>, &T) -> Result<(), CanonicalError>,
) -> Result<Vec<u8>, CanonicalError>
where
    T: Clone + HasPersistentId,
{
    let mut records = records.to_vec();
    records.sort_by_key(HasPersistentId::persistent_id);
    ensure_unique_by(&records, HasPersistentId::persistent_id)?;
    let mut bytes = Vec::new();
    extend_count(&mut bytes, records.len())?;
    for record in &records {
        id_and_encode(&mut bytes, record)?;
    }
    Ok(bytes)
}

trait HasPersistentId {
    fn persistent_id(&self) -> PersistentId;
}

macro_rules! impl_has_persistent_id {
    ($($type:ty),+ $(,)?) => {
        $(
            impl HasPersistentId for $type {
                fn persistent_id(&self) -> PersistentId {
                    self.id
                }
            }
        )+
    };
}

impl_has_persistent_id!(
    CharacterSnapshot,
    ItemSnapshot,
    QuestSnapshot,
    DialogueSnapshot,
    FactionSnapshot,
    InteractiveObjectSnapshot,
    WorldChunkRecordSnapshot,
);

fn decode_characters(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CharacterSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        let id = read_id(cursor)?;
        let revision = cursor.read_u64()?;
        let archetype_id = read_text(cursor, limits)?;
        let skill_count = read_count(cursor, limits)?;
        let mut skills = Vec::with_capacity(skill_count);
        for _ in 0..skill_count {
            let skill_id = read_text(cursor, limits)?;
            let raw = read_u16(cursor)?;
            let proficiency = SkillProficiency::new(raw)
                .map_err(|_| RpgDecodeError::InvalidSkillProficiency(raw))?;
            skills.push(SkillProficiencyEntry {
                skill_id,
                proficiency,
            });
        }
        let relationship_count = read_count(cursor, limits)?;
        let mut relationships = Vec::with_capacity(relationship_count);
        for _ in 0..relationship_count {
            relationships.push(RelationshipEntry {
                target: read_id(cursor)?,
                dimension_id: read_text(cursor, limits)?,
                value: read_i32(cursor)?,
            });
        }
        Ok(CharacterSnapshot {
            id,
            revision,
            archetype_id,
            skills,
            relationships,
        })
    })
}

fn decode_items(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<ItemSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(ItemSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            archetype_id: read_text(cursor, limits)?,
            owner: read_optional_id(cursor)?,
            quantity: cursor.read_u32()?,
        })
    })
}

fn decode_quests(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<QuestSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(QuestSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            definition_id: read_text(cursor, limits)?,
            state_id: read_text(cursor, limits)?,
        })
    })
}

fn decode_dialogues(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<DialogueSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(DialogueSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            definition_id: read_text(cursor, limits)?,
            speaker: read_id(cursor)?,
            listener: read_id(cursor)?,
            node_id: read_text(cursor, limits)?,
        })
    })
}

fn decode_factions(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<FactionSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        let id = read_id(cursor)?;
        let revision = cursor.read_u64()?;
        let definition_id = read_text(cursor, limits)?;
        let member_count = read_count(cursor, limits)?;
        let mut members = Vec::with_capacity(member_count);
        for _ in 0..member_count {
            members.push(read_id(cursor)?);
        }
        Ok(FactionSnapshot {
            id,
            revision,
            definition_id,
            members,
        })
    })
}

fn decode_interactive_objects(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<InteractiveObjectSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(InteractiveObjectSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            archetype_id: read_text(cursor, limits)?,
            state_id: read_text(cursor, limits)?,
        })
    })
}

fn decode_world_chunk_records(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<WorldChunkRecordSnapshot>, RpgDecodeError> {
    decode_records(bytes, limits, |cursor| {
        Ok(WorldChunkRecordSnapshot {
            id: read_id(cursor)?,
            revision: cursor.read_u64()?,
            record_schema_id: read_text(cursor, limits)?,
            state_id: read_text(cursor, limits)?,
        })
    })
}

fn decode_records<T>(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    decode: impl Fn(&mut CanonicalCursor<'_>) -> Result<T, RpgDecodeError>,
) -> Result<Vec<T>, RpgDecodeError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits)?;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        records.push(decode(&mut cursor)?);
    }
    cursor.finish()?;
    Ok(records)
}

fn validate_snapshot_envelope(segment: &DecodedCanonicalSegment) -> Result<(), RpgDecodeError> {
    if segment.owner_id != RPG_SNAPSHOT_OWNER_ID
        || segment.schema_id != RPG_SNAPSHOT_SCHEMA_ID
        || segment.segment_id != RPG_SNAPSHOT_SEGMENT_ID
    {
        return Err(RpgDecodeError::WrongSnapshotEnvelope);
    }
    Ok(())
}

fn validate_snapshot_fields(segment: &DecodedCanonicalSegment) -> Result<(), RpgDecodeError> {
    const EXPECTED: [(u32, u8); 8] = [
        (1, CANONICAL_TYPE_U32),
        (2, CANONICAL_TYPE_SEQUENCE),
        (3, CANONICAL_TYPE_SEQUENCE),
        (4, CANONICAL_TYPE_SEQUENCE),
        (5, CANONICAL_TYPE_SEQUENCE),
        (6, CANONICAL_TYPE_SEQUENCE),
        (7, CANONICAL_TYPE_SEQUENCE),
        (8, CANONICAL_TYPE_SEQUENCE),
    ];
    for field in &segment.fields {
        if !EXPECTED
            .iter()
            .any(|(field_id, _)| *field_id == field.field_id)
        {
            return Err(RpgDecodeError::UnknownSnapshotField(field.field_id));
        }
    }
    for (field_id, expected_type) in EXPECTED {
        let field = segment
            .field(field_id)
            .ok_or(RpgDecodeError::MissingSnapshotField(field_id))?;
        if field.type_tag != expected_type {
            return Err(RpgDecodeError::SnapshotFieldType {
                field_id,
                expected: expected_type,
                actual: field.type_tag,
            });
        }
    }
    Ok(())
}

fn field_payload(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<&[u8], RpgDecodeError> {
    segment
        .field(field_id)
        .map(|field| field.payload.as_slice())
        .ok_or(RpgDecodeError::MissingSnapshotField(field_id))
}

fn decode_u32_field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<u32, RpgDecodeError> {
    let payload = field_payload(segment, field_id)?;
    let bytes: [u8; 4] = payload
        .try_into()
        .map_err(|_| RpgDecodeError::SnapshotFieldLength {
            field_id,
            expected: 4,
            actual: payload.len(),
        })?;
    Ok(u32::from_le_bytes(bytes))
}

fn extend_count(target: &mut Vec<u8>, count: usize) -> Result<(), CanonicalError> {
    target.extend_from_slice(
        &u32::try_from(count)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

fn ensure_unique_by<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> Result<(), CanonicalError> {
    if values.windows(2).any(|pair| key(&pair[0]) == key(&pair[1])) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    Ok(())
}

fn read_count(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<usize, RpgDecodeError> {
    let count =
        usize::try_from(cursor.read_u32()?).map_err(|_| CanonicalDecodeError::LengthOverflow)?;
    if count > limits.max_sequence_items {
        return Err(RpgDecodeError::TooManyRecords {
            actual: count,
            limit: limits.max_sequence_items,
        });
    }
    Ok(count)
}
