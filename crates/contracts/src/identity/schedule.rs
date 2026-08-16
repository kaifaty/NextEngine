use std::collections::{BTreeMap, BTreeSet};

use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_MAP, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_SET, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_UTF8_NFC, CanonicalCursor,
    CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::cognition::{
    AGENT_COGNITION_CATALOG_OWNER_ID, AGENT_COGNITION_CATALOG_SCHEMA_ID,
    AGENT_COGNITION_SHARD_PLAN_ID, AGENT_COGNITION_SYSTEM_ID, AGENT_MEMORY_SNAPSHOT_OWNER_ID,
    AGENT_MEMORY_SNAPSHOT_SCHEMA_ID, AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
    AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID,
};
use crate::command::CommandPhase;
use crate::ids::{ContentHash, SchemaId, SystemId, content_hash_from_bytes};
use crate::world_population::{
    WORLD_NAVIGATION_CATALOG_OWNER_ID, WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
    WORLD_POPULATION_CATALOG_OWNER_ID, WORLD_POPULATION_CATALOG_SCHEMA_ID,
    WORLD_POPULATION_SHARD_PLAN_ID, WORLD_POPULATION_SNAPSHOT_OWNER_ID,
    WORLD_POPULATION_SNAPSHOT_SCHEMA_ID, WORLD_POPULATION_SYSTEM_ID,
};
use crate::world_routine::{
    WORLD_ROUTINE_CATALOG_OWNER_ID, WORLD_ROUTINE_CATALOG_SCHEMA_ID, WORLD_ROUTINE_SHARD_PLAN_ID,
    WORLD_ROUTINE_SNAPSHOT_OWNER_ID, WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID, WORLD_ROUTINE_SYSTEM_ID,
};

use super::codec::{
    decode_map, decode_nested_struct, encode_map, field, nested_array, nested_payload,
    nested_struct, nested_text, nested_u8, nested_u32, nested_value, read_u16, require_envelope,
    require_fields, require_nested_fields,
};
use super::error::IdentityContractError;

mod core_r4b;
mod stage;

pub use stage::RuntimeStageId;

pub const SCHEDULE_MANIFEST_SCHEMA_VERSION: u16 = 1;
pub const SCHEDULE_MANIFEST_OWNER_ID: &str = "nextengine.runtime";
pub const SCHEDULE_MANIFEST_SCHEMA_ID: &str = "nextengine.schedule-manifest";
pub const SCHEDULE_MANIFEST_SEGMENT_ID: &str = "v1";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AccessKeyV1 {
    pub owner_id: SchemaId,
    pub schema_id: SchemaId,
    pub field_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessSetV1 {
    pub reads: Vec<AccessKeyV1>,
    pub writes: Vec<AccessKeyV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum QueryOrderV1 {
    PersistentId,
    StableRecordKey { key_schema_id: SchemaId },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ReducerKindV1 {
    CheckedSum = 1,
    Minimum = 2,
    Maximum = 3,
    BitAnd = 4,
    BitOr = 5,
    BitXor = 6,
    StableOrderedFold = 7,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ShardPartitionRuleV1 {
    Sha256StableKeyFirstU64LeModulo = 1,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ShardRecordOrderV1 {
    CanonicalStableRecordKey = 1,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DeltaMergeOrderV1 {
    OwnerSchemaRecordFieldSystemShard = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReducerDescriptorV1 {
    pub schema_version: u16,
    pub reducer_id: SchemaId,
    pub target: AccessKeyV1,
    pub value_schema_id: SchemaId,
    pub kind: ReducerKindV1,
    pub ordered_fold_schema_hash: Option<ContentHash>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicalShardPlanV1 {
    pub schema_version: u16,
    pub shard_plan_id: SchemaId,
    pub system_id: SystemId,
    pub logical_shard_count: u32,
    pub partition_rule: ShardPartitionRuleV1,
    pub record_order: ShardRecordOrderV1,
    pub merge_order: DeltaMergeOrderV1,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CommandBarrierSourceV1 {
    AuthenticatedExternalAndQueuedInternal = 1,
    InternalSystemOnly = 2,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandAdmissionBarrierV1 {
    pub schema_version: u16,
    pub phase: CommandPhase,
    pub stage_index: u8,
    pub batch_ordinal: u32,
    pub source: CommandBarrierSourceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemDescriptorV1 {
    pub schema_version: u16,
    pub system_id: SystemId,
    pub owner_id: SchemaId,
    pub stage_id: RuntimeStageId,
    pub before: Vec<SystemId>,
    pub after: Vec<SystemId>,
    pub access: AccessSetV1,
    pub query_order: QueryOrderV1,
    pub shard_plan_id: SchemaId,
    pub reducer_ids: Vec<SchemaId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleManifestV1 {
    pub schema_version: u16,
    pub stage_order: Vec<RuntimeStageId>,
    pub systems: BTreeMap<SystemId, SystemDescriptorV1>,
    pub reducers: BTreeMap<SchemaId, ReducerDescriptorV1>,
    pub shard_plans: BTreeMap<SchemaId, LogicalShardPlanV1>,
    pub command_admission_barriers: Vec<CommandAdmissionBarrierV1>,
}

impl ScheduleManifestV1 {
    pub fn validate(&self) -> Result<(), IdentityContractError> {
        if self.schema_version != SCHEDULE_MANIFEST_SCHEMA_VERSION
            || self.stage_order.is_empty()
            || self.stage_order.windows(2).any(|pair| pair[0] >= pair[1])
            || self
                .command_admission_barriers
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(IdentityContractError::ScheduleClosureInvalid);
        }
        let stages: BTreeSet<_> = self.stage_order.iter().copied().collect();
        for (key, system) in &self.systems {
            if key != &system.system_id
                || system.schema_version != SCHEDULE_MANIFEST_SCHEMA_VERSION
                || !stages.contains(&system.stage_id)
                || !strictly_sorted(&system.before)
                || !strictly_sorted(&system.after)
                || !strictly_sorted(&system.access.reads)
                || !strictly_sorted(&system.access.writes)
                || !strictly_sorted(&system.reducer_ids)
                || system
                    .access
                    .reads
                    .iter()
                    .any(|read| system.access.writes.binary_search(read).is_ok())
            {
                return Err(IdentityContractError::ScheduleClosureInvalid);
            }
            let plan = self
                .shard_plans
                .get(&system.shard_plan_id)
                .ok_or(IdentityContractError::ScheduleClosureInvalid)?;
            if plan.system_id != system.system_id {
                return Err(IdentityContractError::ScheduleClosureInvalid);
            }
            if system
                .before
                .iter()
                .chain(&system.after)
                .any(|id| !self.systems.contains_key(id))
                || system
                    .reducer_ids
                    .iter()
                    .any(|id| !self.reducers.contains_key(id))
            {
                return Err(IdentityContractError::ScheduleClosureInvalid);
            }
        }
        for (key, plan) in &self.shard_plans {
            if key != &plan.shard_plan_id
                || plan.schema_version != SCHEDULE_MANIFEST_SCHEMA_VERSION
                || plan.logical_shard_count == 0
                || !self.systems.contains_key(&plan.system_id)
            {
                return Err(IdentityContractError::ScheduleClosureInvalid);
            }
        }
        for (key, reducer) in &self.reducers {
            if key != &reducer.reducer_id
                || reducer.schema_version != SCHEDULE_MANIFEST_SCHEMA_VERSION
                || matches!(reducer.kind, ReducerKindV1::StableOrderedFold)
                    != reducer.ordered_fold_schema_hash.is_some()
            {
                return Err(IdentityContractError::ScheduleClosureInvalid);
            }
        }
        for barrier in &self.command_admission_barriers {
            if barrier.schema_version != SCHEDULE_MANIFEST_SCHEMA_VERSION
                || barrier.batch_ordinal != 0
                || !stages
                    .iter()
                    .any(|stage| *stage as u8 == barrier.stage_index)
            {
                return Err(IdentityContractError::ScheduleClosureInvalid);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let stage_order = encode_sequence(
            self.stage_order
                .iter()
                .map(|stage| nested_value(CANONICAL_TYPE_U8, &[*stage as u8]))
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let systems = encode_map(
            self.systems
                .iter()
                .map(|(key, value)| {
                    Ok((
                        nested_value(CANONICAL_TYPE_UTF8_NFC, key.as_str().as_bytes())?,
                        encode_system(value)?,
                    ))
                })
                .collect::<Result<Vec<_>, CanonicalError>>()?,
        )?;
        let reducers = encode_map(
            self.reducers
                .iter()
                .map(|(key, value)| {
                    Ok((
                        nested_value(CANONICAL_TYPE_UTF8_NFC, key.as_str().as_bytes())?,
                        encode_reducer(value)?,
                    ))
                })
                .collect::<Result<Vec<_>, CanonicalError>>()?,
        )?;
        let shard_plans = encode_map(
            self.shard_plans
                .iter()
                .map(|(key, value)| {
                    Ok((
                        nested_value(CANONICAL_TYPE_UTF8_NFC, key.as_str().as_bytes())?,
                        encode_shard_plan(value)?,
                    ))
                })
                .collect::<Result<Vec<_>, CanonicalError>>()?,
        )?;
        let barriers = encode_sequence(
            self.command_admission_barriers
                .iter()
                .map(encode_barrier)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        encode_canonical_segment(
            SCHEDULE_MANIFEST_OWNER_ID,
            SCHEDULE_MANIFEST_SCHEMA_ID,
            SCHEDULE_MANIFEST_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_SEQUENCE, stage_order),
                CanonicalField::new(3, CANONICAL_TYPE_MAP, systems),
                CanonicalField::new(4, CANONICAL_TYPE_MAP, reducers),
                CanonicalField::new(5, CANONICAL_TYPE_MAP, shard_plans),
                CanonicalField::new(6, CANONICAL_TYPE_SEQUENCE, barriers),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, IdentityContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        require_envelope(
            &segment,
            SCHEDULE_MANIFEST_OWNER_ID,
            SCHEDULE_MANIFEST_SCHEMA_ID,
            SCHEDULE_MANIFEST_SEGMENT_ID,
        )?;
        require_fields(
            &segment,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_SEQUENCE),
                (3, CANONICAL_TYPE_MAP),
                (4, CANONICAL_TYPE_MAP),
                (5, CANONICAL_TYPE_MAP),
                (6, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let stage_order = decode_sequence(field(&segment, 2)?.payload.as_slice(), limits)?
            .into_iter()
            .map(|value| {
                let (tag, payload) = decode_nested(&value, limits)?;
                if tag != CANONICAL_TYPE_U8 || payload.len() != 1 {
                    return Err(IdentityContractError::ScheduleClosureInvalid);
                }
                RuntimeStageId::from_tag(payload[0])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut systems = BTreeMap::new();
        for (key, value) in decode_map(field(&segment, 3)?.payload.as_slice(), limits)? {
            let key = decode_nested_text(&key, limits)?;
            let key = SystemId::new(key)?;
            if systems
                .insert(key, decode_system(&value, limits)?)
                .is_some()
            {
                return Err(IdentityContractError::DuplicateKey);
            }
        }
        let mut reducers = BTreeMap::new();
        for (key, value) in decode_map(field(&segment, 4)?.payload.as_slice(), limits)? {
            let key = SchemaId::new(decode_nested_text(&key, limits)?)?;
            if reducers
                .insert(key, decode_reducer(&value, limits)?)
                .is_some()
            {
                return Err(IdentityContractError::DuplicateKey);
            }
        }
        let mut shard_plans = BTreeMap::new();
        for (key, value) in decode_map(field(&segment, 5)?.payload.as_slice(), limits)? {
            let key = SchemaId::new(decode_nested_text(&key, limits)?)?;
            if shard_plans
                .insert(key, decode_shard_plan(&value, limits)?)
                .is_some()
            {
                return Err(IdentityContractError::DuplicateKey);
            }
        }
        let command_admission_barriers =
            decode_sequence(field(&segment, 6)?.payload.as_slice(), limits)?
                .into_iter()
                .map(|value| decode_barrier(&value, limits))
                .collect::<Result<Vec<_>, _>>()?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            stage_order,
            systems,
            reducers,
            shard_plans,
            command_admission_barriers,
        };
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(IdentityContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, IdentityContractError> {
        self.validate()?;
        let bytes = self.canonical_bytes()?;
        let mut preimage = b"nextengine.runtime-profile.v1\0".to_vec();
        extend_lp(&mut preimage, SCHEDULE_MANIFEST_SCHEMA_ID.as_bytes())?;
        extend_lp(&mut preimage, &bytes)?;
        Ok(content_hash_from_bytes(sha256(&preimage)))
    }
}

fn encode_access_key(value: &AccessKeyV1) -> Result<Vec<u8>, CanonicalError> {
    nested_struct([
        CanonicalField::new(
            1,
            CANONICAL_TYPE_UTF8_NFC,
            value.owner_id.as_str().as_bytes().to_vec(),
        ),
        CanonicalField::new(
            2,
            CANONICAL_TYPE_UTF8_NFC,
            value.schema_id.as_str().as_bytes().to_vec(),
        ),
        CanonicalField::new(3, CANONICAL_TYPE_U32, value.field_id.to_le_bytes().to_vec()),
    ])
}

fn decode_access_key(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<AccessKeyV1, IdentityContractError> {
    let fields = decode_nested_struct(bytes, limits)?;
    require_nested_fields(
        &fields,
        &[
            (1, CANONICAL_TYPE_UTF8_NFC),
            (2, CANONICAL_TYPE_UTF8_NFC),
            (3, CANONICAL_TYPE_U32),
        ],
    )?;
    Ok(AccessKeyV1 {
        owner_id: SchemaId::new(nested_text(&fields, 1)?)?,
        schema_id: SchemaId::new(nested_text(&fields, 2)?)?,
        field_id: nested_u32(&fields, 3)?,
    })
}

fn encode_access_set(value: &AccessSetV1) -> Result<Vec<u8>, CanonicalError> {
    nested_struct([
        CanonicalField::new(
            1,
            CANONICAL_TYPE_SET,
            encode_set(
                value
                    .reads
                    .iter()
                    .map(encode_access_key)
                    .collect::<Result<Vec<_>, _>>()?,
            )?,
        ),
        CanonicalField::new(
            2,
            CANONICAL_TYPE_SET,
            encode_set(
                value
                    .writes
                    .iter()
                    .map(encode_access_key)
                    .collect::<Result<Vec<_>, _>>()?,
            )?,
        ),
    ])
}

fn decode_access_set(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<AccessSetV1, IdentityContractError> {
    let fields = decode_nested_struct(bytes, limits)?;
    require_nested_fields(&fields, &[(1, CANONICAL_TYPE_SET), (2, CANONICAL_TYPE_SET)])?;
    let mut reads = decode_set(nested_payload(&fields, 1)?, limits)?
        .into_iter()
        .map(|value| decode_access_key(&value, limits))
        .collect::<Result<Vec<_>, _>>()?;
    reads.sort();
    let mut writes = decode_set(nested_payload(&fields, 2)?, limits)?
        .into_iter()
        .map(|value| decode_access_key(&value, limits))
        .collect::<Result<Vec<_>, _>>()?;
    writes.sort();
    Ok(AccessSetV1 { reads, writes })
}

fn encode_system(value: &SystemDescriptorV1) -> Result<Vec<u8>, CanonicalError> {
    let query_order = match &value.query_order {
        QueryOrderV1::PersistentId => vec![1],
        QueryOrderV1::StableRecordKey { key_schema_id } => {
            let mut bytes = vec![2];
            bytes.extend_from_slice(&nested_value(
                CANONICAL_TYPE_UTF8_NFC,
                key_schema_id.as_str().as_bytes(),
            )?);
            bytes
        }
    };
    nested_struct([
        CanonicalField::new(
            1,
            CANONICAL_TYPE_U16,
            value.schema_version.to_le_bytes().to_vec(),
        ),
        CanonicalField::new(
            2,
            CANONICAL_TYPE_UTF8_NFC,
            value.system_id.as_str().as_bytes().to_vec(),
        ),
        CanonicalField::new(
            3,
            CANONICAL_TYPE_UTF8_NFC,
            value.owner_id.as_str().as_bytes().to_vec(),
        ),
        CanonicalField::new(4, CANONICAL_TYPE_U8, vec![value.stage_id as u8]),
        CanonicalField::new(5, CANONICAL_TYPE_SET, encode_system_id_set(&value.before)?),
        CanonicalField::new(6, CANONICAL_TYPE_SET, encode_system_id_set(&value.after)?),
        CanonicalField::new(
            7,
            CANONICAL_TYPE_STRUCT,
            unwrap_nested_struct(encode_access_set(&value.access)?)?,
        ),
        CanonicalField::new(8, CANONICAL_TYPE_TAGGED_UNION, query_order),
        CanonicalField::new(
            9,
            CANONICAL_TYPE_UTF8_NFC,
            value.shard_plan_id.as_str().as_bytes().to_vec(),
        ),
        CanonicalField::new(
            10,
            CANONICAL_TYPE_SET,
            encode_schema_id_set(&value.reducer_ids)?,
        ),
    ])
}

fn decode_system(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<SystemDescriptorV1, IdentityContractError> {
    let fields = decode_nested_struct(bytes, limits)?;
    require_nested_fields(
        &fields,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_UTF8_NFC),
            (3, CANONICAL_TYPE_UTF8_NFC),
            (4, CANONICAL_TYPE_U8),
            (5, CANONICAL_TYPE_SET),
            (6, CANONICAL_TYPE_SET),
            (7, CANONICAL_TYPE_STRUCT),
            (8, CANONICAL_TYPE_TAGGED_UNION),
            (9, CANONICAL_TYPE_UTF8_NFC),
            (10, CANONICAL_TYPE_SET),
        ],
    )?;
    let query_order = decode_query_order(nested_payload(&fields, 8)?, limits)?;
    Ok(SystemDescriptorV1 {
        schema_version: u16::from_le_bytes(nested_array(&fields, 1)?),
        system_id: SystemId::new(nested_text(&fields, 2)?)?,
        owner_id: SchemaId::new(nested_text(&fields, 3)?)?,
        stage_id: RuntimeStageId::from_tag(nested_u8(&fields, 4)?)?,
        before: decode_system_id_set(nested_payload(&fields, 5)?, limits)?,
        after: decode_system_id_set(nested_payload(&fields, 6)?, limits)?,
        access: decode_access_set(
            &nested_value(CANONICAL_TYPE_STRUCT, nested_payload(&fields, 7)?)?,
            limits,
        )?,
        query_order,
        shard_plan_id: SchemaId::new(nested_text(&fields, 9)?)?,
        reducer_ids: decode_schema_id_set(nested_payload(&fields, 10)?, limits)?,
    })
}

fn decode_query_order(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<QueryOrderV1, IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let value = match cursor.read_u8()? {
        1 => QueryOrderV1::PersistentId,
        2 => {
            let tag = cursor.read_u8()?;
            let length = usize::try_from(cursor.read_u64()?)
                .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::LengthOverflow))?;
            if tag != CANONICAL_TYPE_UTF8_NFC || length > limits.max_identifier_bytes {
                return Err(IdentityContractError::ScheduleClosureInvalid);
            }
            QueryOrderV1::StableRecordKey {
                key_schema_id: SchemaId::new(
                    std::str::from_utf8(cursor.read_exact(length)?)
                        .map_err(|_| CanonicalDecodeError::InvalidUtf8)?,
                )?,
            }
        }
        tag => return Err(IdentityContractError::UnknownTag(tag)),
    };
    cursor.finish()?;
    Ok(value)
}

fn encode_shard_plan(value: &LogicalShardPlanV1) -> Result<Vec<u8>, CanonicalError> {
    nested_struct([
        CanonicalField::new(
            1,
            CANONICAL_TYPE_U16,
            value.schema_version.to_le_bytes().to_vec(),
        ),
        CanonicalField::new(
            2,
            CANONICAL_TYPE_UTF8_NFC,
            value.shard_plan_id.as_str().as_bytes().to_vec(),
        ),
        CanonicalField::new(
            3,
            CANONICAL_TYPE_UTF8_NFC,
            value.system_id.as_str().as_bytes().to_vec(),
        ),
        CanonicalField::new(
            4,
            CANONICAL_TYPE_U32,
            value.logical_shard_count.to_le_bytes().to_vec(),
        ),
        CanonicalField::new(5, CANONICAL_TYPE_U8, vec![value.partition_rule as u8]),
        CanonicalField::new(6, CANONICAL_TYPE_U8, vec![value.record_order as u8]),
        CanonicalField::new(7, CANONICAL_TYPE_U8, vec![value.merge_order as u8]),
    ])
}

fn decode_shard_plan(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<LogicalShardPlanV1, IdentityContractError> {
    let fields = decode_nested_struct(bytes, limits)?;
    require_nested_fields(
        &fields,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_UTF8_NFC),
            (3, CANONICAL_TYPE_UTF8_NFC),
            (4, CANONICAL_TYPE_U32),
            (5, CANONICAL_TYPE_U8),
            (6, CANONICAL_TYPE_U8),
            (7, CANONICAL_TYPE_U8),
        ],
    )?;
    if nested_u8(&fields, 5)? != 1 || nested_u8(&fields, 6)? != 1 || nested_u8(&fields, 7)? != 1 {
        return Err(IdentityContractError::ScheduleClosureInvalid);
    }
    Ok(LogicalShardPlanV1 {
        schema_version: u16::from_le_bytes(nested_array(&fields, 1)?),
        shard_plan_id: SchemaId::new(nested_text(&fields, 2)?)?,
        system_id: SystemId::new(nested_text(&fields, 3)?)?,
        logical_shard_count: nested_u32(&fields, 4)?,
        partition_rule: ShardPartitionRuleV1::Sha256StableKeyFirstU64LeModulo,
        record_order: ShardRecordOrderV1::CanonicalStableRecordKey,
        merge_order: DeltaMergeOrderV1::OwnerSchemaRecordFieldSystemShard,
    })
}

fn encode_reducer(value: &ReducerDescriptorV1) -> Result<Vec<u8>, CanonicalError> {
    let optional = match value.ordered_fold_schema_hash {
        None => vec![0],
        Some(hash) => {
            let mut bytes = vec![1];
            bytes.extend_from_slice(&nested_value(CANONICAL_TYPE_HASH256, hash.as_bytes())?);
            bytes
        }
    };
    nested_struct([
        CanonicalField::new(
            1,
            CANONICAL_TYPE_U16,
            value.schema_version.to_le_bytes().to_vec(),
        ),
        CanonicalField::new(
            2,
            CANONICAL_TYPE_UTF8_NFC,
            value.reducer_id.as_str().as_bytes().to_vec(),
        ),
        CanonicalField::new(
            3,
            CANONICAL_TYPE_STRUCT,
            unwrap_nested_struct(encode_access_key(&value.target)?)?,
        ),
        CanonicalField::new(
            4,
            CANONICAL_TYPE_UTF8_NFC,
            value.value_schema_id.as_str().as_bytes().to_vec(),
        ),
        CanonicalField::new(5, CANONICAL_TYPE_U8, vec![value.kind as u8]),
        CanonicalField::new(6, CANONICAL_TYPE_OPTIONAL, optional),
    ])
}

fn decode_reducer(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<ReducerDescriptorV1, IdentityContractError> {
    let fields = decode_nested_struct(bytes, limits)?;
    require_nested_fields(
        &fields,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_UTF8_NFC),
            (3, CANONICAL_TYPE_STRUCT),
            (4, CANONICAL_TYPE_UTF8_NFC),
            (5, CANONICAL_TYPE_U8),
            (6, CANONICAL_TYPE_OPTIONAL),
        ],
    )?;
    let kind = match nested_u8(&fields, 5)? {
        1 => ReducerKindV1::CheckedSum,
        2 => ReducerKindV1::Minimum,
        3 => ReducerKindV1::Maximum,
        4 => ReducerKindV1::BitAnd,
        5 => ReducerKindV1::BitOr,
        6 => ReducerKindV1::BitXor,
        7 => ReducerKindV1::StableOrderedFold,
        tag => return Err(IdentityContractError::UnknownTag(tag)),
    };
    Ok(ReducerDescriptorV1 {
        schema_version: u16::from_le_bytes(nested_array(&fields, 1)?),
        reducer_id: SchemaId::new(nested_text(&fields, 2)?)?,
        target: decode_access_key(
            &nested_value(CANONICAL_TYPE_STRUCT, nested_payload(&fields, 3)?)?,
            limits,
        )?,
        value_schema_id: SchemaId::new(nested_text(&fields, 4)?)?,
        kind,
        ordered_fold_schema_hash: decode_optional_hash(nested_payload(&fields, 6)?)?,
    })
}

fn encode_barrier(value: &CommandAdmissionBarrierV1) -> Result<Vec<u8>, CanonicalError> {
    nested_struct([
        CanonicalField::new(
            1,
            CANONICAL_TYPE_U16,
            value.schema_version.to_le_bytes().to_vec(),
        ),
        CanonicalField::new(2, CANONICAL_TYPE_U8, vec![value.phase as u8]),
        CanonicalField::new(3, CANONICAL_TYPE_U8, vec![value.stage_index]),
        CanonicalField::new(
            4,
            CANONICAL_TYPE_U32,
            value.batch_ordinal.to_le_bytes().to_vec(),
        ),
        CanonicalField::new(5, CANONICAL_TYPE_U8, vec![value.source as u8]),
    ])
}

fn decode_barrier(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<CommandAdmissionBarrierV1, IdentityContractError> {
    let fields = decode_nested_struct(bytes, limits)?;
    require_nested_fields(
        &fields,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_U8),
            (3, CANONICAL_TYPE_U8),
            (4, CANONICAL_TYPE_U32),
            (5, CANONICAL_TYPE_U8),
        ],
    )?;
    let phase = match nested_u8(&fields, 2)? {
        0 => CommandPhase::Ingress,
        1 => CommandPhase::Outcome,
        tag => return Err(IdentityContractError::UnknownTag(tag)),
    };
    let source = match nested_u8(&fields, 5)? {
        1 => CommandBarrierSourceV1::AuthenticatedExternalAndQueuedInternal,
        2 => CommandBarrierSourceV1::InternalSystemOnly,
        tag => return Err(IdentityContractError::UnknownTag(tag)),
    };
    Ok(CommandAdmissionBarrierV1 {
        schema_version: u16::from_le_bytes(nested_array(&fields, 1)?),
        phase,
        stage_index: nested_u8(&fields, 3)?,
        batch_ordinal: nested_u32(&fields, 4)?,
        source,
    })
}

fn encode_system_id_set(values: &[SystemId]) -> Result<Vec<u8>, CanonicalError> {
    encode_set(
        values
            .iter()
            .map(|value| nested_value(CANONICAL_TYPE_UTF8_NFC, value.as_str().as_bytes()))
            .collect::<Result<Vec<_>, _>>()?,
    )
}

fn decode_system_id_set(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<SystemId>, IdentityContractError> {
    let mut values = decode_set(bytes, limits)?
        .into_iter()
        .map(|value| Ok(SystemId::new(decode_nested_text(&value, limits)?)?))
        .collect::<Result<Vec<_>, IdentityContractError>>()?;
    values.sort();
    Ok(values)
}

fn encode_schema_id_set(values: &[SchemaId]) -> Result<Vec<u8>, CanonicalError> {
    encode_set(
        values
            .iter()
            .map(|value| nested_value(CANONICAL_TYPE_UTF8_NFC, value.as_str().as_bytes()))
            .collect::<Result<Vec<_>, _>>()?,
    )
}

fn decode_schema_id_set(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<SchemaId>, IdentityContractError> {
    let mut values = decode_set(bytes, limits)?
        .into_iter()
        .map(|value| Ok(SchemaId::new(decode_nested_text(&value, limits)?)?))
        .collect::<Result<Vec<_>, IdentityContractError>>()?;
    values.sort();
    Ok(values)
}

fn encode_sequence(values: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(values.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for value in values {
        bytes.extend_from_slice(&value);
    }
    Ok(bytes)
}

fn encode_set(mut values: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    encode_sequence(values)
}

fn decode_sequence(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<Vec<u8>>, IdentityContractError> {
    decode_values(bytes, limits, false)
}

fn decode_set(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<Vec<u8>>, IdentityContractError> {
    decode_values(bytes, limits, true)
}

fn decode_values(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    sorted: bool,
) -> Result<Vec<Vec<u8>>, IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let tag = cursor.read_u8()?;
        let length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::LengthOverflow))?;
        if length > limits.max_field_payload_bytes {
            return Err(IdentityContractError::ScheduleClosureInvalid);
        }
        values.push(nested_value(tag, cursor.read_exact(length)?)?);
    }
    cursor.finish()?;
    if sorted && values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(IdentityContractError::DuplicateKey);
    }
    Ok(values)
}

fn decode_nested(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<(u8, &[u8]), IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let tag = cursor.read_u8()?;
    let length = usize::try_from(cursor.read_u64()?)
        .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::LengthOverflow))?;
    if length > limits.max_field_payload_bytes {
        return Err(IdentityContractError::ScheduleClosureInvalid);
    }
    let payload = cursor.read_exact(length)?;
    cursor.finish()?;
    Ok((tag, payload))
}

fn decode_nested_text(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<&str, IdentityContractError> {
    let (tag, payload) = decode_nested(bytes, limits)?;
    if tag != CANONICAL_TYPE_UTF8_NFC || payload.len() > limits.max_identifier_bytes {
        return Err(IdentityContractError::ScheduleClosureInvalid);
    }
    std::str::from_utf8(payload)
        .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::InvalidUtf8))
}

fn unwrap_nested_struct(bytes: Vec<u8>) -> Result<Vec<u8>, CanonicalError> {
    let mut cursor = CanonicalCursor::new(&bytes);
    let tag = cursor
        .read_u8()
        .map_err(|_| CanonicalError::LengthOverflow)?;
    if tag != CANONICAL_TYPE_STRUCT {
        return Err(CanonicalError::LengthOverflow);
    }
    let length = usize::try_from(
        cursor
            .read_u64()
            .map_err(|_| CanonicalError::LengthOverflow)?,
    )
    .map_err(|_| CanonicalError::LengthOverflow)?;
    let payload = cursor
        .read_exact(length)
        .map_err(|_| CanonicalError::LengthOverflow)?
        .to_vec();
    cursor
        .finish()
        .map_err(|_| CanonicalError::LengthOverflow)?;
    Ok(payload)
}

fn decode_optional_hash(bytes: &[u8]) -> Result<Option<ContentHash>, IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let value = match cursor.read_u8()? {
        0 => None,
        1 => {
            let tag = cursor.read_u8()?;
            let length = cursor.read_u64()?;
            if tag != CANONICAL_TYPE_HASH256 || length != 32 {
                return Err(IdentityContractError::ScheduleClosureInvalid);
            }
            Some(ContentHash::from_bytes(
                cursor
                    .read_exact(32)?
                    .try_into()
                    .map_err(|_| IdentityContractError::ScheduleClosureInvalid)?,
            ))
        }
        tag => return Err(IdentityContractError::UnknownTag(tag)),
    };
    cursor.finish()?;
    Ok(value)
}

fn extend_lp(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), CanonicalError> {
    bytes.extend_from_slice(
        &u64::try_from(value.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value);
    Ok(())
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[cfg(test)]
mod tests;
