use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_ID128, CANONICAL_TYPE_U16, CanonicalCursor,
    CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::ids::{
    AssetId, ContentHash, IdentifierError, PersistentId, SchemaId, content_hash_from_bytes,
};

pub const COGNITION_SCHEMA_VERSION: u16 = 2;
pub const AGENT_COGNITION_COMMAND_SCHEMA_VERSION: u32 = 2;
pub const AGENT_COGNITION_EVENT_SCHEMA_VERSION: u32 = 2;

pub const AGENT_COGNITION_CATALOG_OWNER_ID: &str = "nextengine.assets";
pub const AGENT_COGNITION_CATALOG_SCHEMA_ID: &str = "nextengine.content.agent-cognition-catalog";
pub const AGENT_COGNITION_CATALOG_SEGMENT_ID: &str = "nextengine.agent-cognition-catalog.v2";

pub const AGENT_MEMORY_SNAPSHOT_OWNER_ID: &str = "nextengine.memory-service";
pub const AGENT_MEMORY_SNAPSHOT_SCHEMA_ID: &str = "nextengine.agent-memory-snapshot";
pub const AGENT_MEMORY_SNAPSHOT_SEGMENT_ID: &str = "agent-memory";
pub const AGENT_RUNTIME_SNAPSHOT_OWNER_ID: &str = "nextengine.agent-runtime";
pub const AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID: &str = "nextengine.agent-cognition-snapshot";
pub const AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID: &str = "agent-cognition";

pub const AGENT_COGNITION_COMMAND_SCHEMA_ID: &str = "nextengine.command.agent-cognition";
pub const AGENT_COGNITION_COMMAND_KIND_ID: &str = "nextengine.command-kind.agent-cognition";
pub const AGENT_COGNITION_EVENT_SCHEMA_ID: &str = "nextengine.event.agent-decision-committed";
pub const AGENT_COGNITION_CAPABILITY_ID: &str = "nextengine.capability.agent-cognition-commit";
pub const AGENT_COGNITION_SYSTEM_ID: &str = "nextengine.system.agent-cognition-boundary";
pub const AGENT_COGNITION_CAPABILITY_SUBJECT_ID: &str =
    "nextengine.capability-subject.agent-cognition-boundary";
pub const AGENT_COGNITION_SHARD_PLAN_ID: &str = "nextengine.shard-plan.agent-cognition-single";
pub const AGENT_COGNITION_PRIORITY_CLASS: u16 = 270;

pub const COGNITION_MAX_BELIEFS: usize = 32;
pub const COGNITION_MAX_RETRIEVED_BELIEFS: usize = 8;
pub const COGNITION_MAX_FACTS: usize = 16;
pub const COGNITION_MAX_AFFORDANCES: usize = 8;
pub const COGNITION_MAX_GOALS: usize = 8;
pub const COGNITION_MAX_PLAN_STEPS: usize = 8;
pub const COGNITION_MAX_SUSPENDED_GOALS: usize = 4;
pub const COGNITION_MAX_SPEECH_ACTS: usize = 16;
pub const COGNITION_MAX_CLAIM_PROVENANCE: usize = 8;
pub const COGNITION_MAX_TEXT_BYTES: usize = 4 * 1024;
pub const COGNITION_Q16_ONE: i32 = 1 << 16;
const COGNITION_MAX_SCORE_Q16: i32 = 256 << 16;

mod belief;
mod command;
mod error;
mod executive;
mod social;

pub use belief::*;
pub use command::*;
pub use error::*;
pub use executive::*;
pub use social::*;

struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn with_domain(domain: &[u8]) -> Self {
        Self {
            bytes: domain.to_vec(),
        }
    }

    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn id(&mut self, value: PersistentId) {
        self.bytes.extend_from_slice(value.as_bytes());
    }

    fn hash(&mut self, value: ContentHash) {
        self.bytes.extend_from_slice(value.as_bytes());
    }

    fn optional_hash(&mut self, value: Option<ContentHash>) {
        match value {
            None => self.u8(0),
            Some(value) => {
                self.u8(1);
                self.hash(value);
            }
        }
    }

    fn count(&mut self, count: usize) -> Result<(), CognitionContractError> {
        self.u32(u32::try_from(count).map_err(|_| CognitionContractError::LimitExceeded)?);
        Ok(())
    }

    fn text(&mut self, value: &str) -> Result<(), CognitionContractError> {
        if value.len() > COGNITION_MAX_TEXT_BYTES {
            return Err(CognitionContractError::LimitExceeded);
        }
        self.count(value.len())?;
        self.bytes.extend_from_slice(value.as_bytes());
        Ok(())
    }

    fn bytes(&mut self, value: &[u8]) -> Result<(), CanonicalError> {
        self.bytes.extend_from_slice(
            &u32::try_from(value.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn schema_ids(&mut self, values: &[SchemaId]) -> Result<(), CognitionContractError> {
        self.count(values.len())?;
        for value in values {
            self.text(value.as_str())?;
        }
        Ok(())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    fn finish_hash(self) -> ContentHash {
        content_hash_from_bytes(sha256(&self.bytes))
    }
}

struct Reader<'a> {
    cursor: CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], limits: CanonicalDecodeLimits) -> Self {
        Self {
            cursor: CanonicalCursor::new(bytes),
            limits,
        }
    }

    fn domain(&mut self, expected: &[u8]) -> Result<(), CognitionContractError> {
        if self.cursor.read_exact(expected.len())? != expected {
            return Err(CognitionContractError::NonCanonicalEncoding);
        }
        Ok(())
    }

    fn u8(&mut self) -> Result<u8, CognitionContractError> {
        Ok(self.cursor.read_u8()?)
    }

    fn u16(&mut self) -> Result<u16, CognitionContractError> {
        Ok(self.cursor.read_u16()?)
    }

    fn u32(&mut self) -> Result<u32, CognitionContractError> {
        Ok(self.cursor.read_u32()?)
    }

    fn u64(&mut self) -> Result<u64, CognitionContractError> {
        Ok(self.cursor.read_u64()?)
    }

    fn i32(&mut self) -> Result<i32, CognitionContractError> {
        Ok(i32::from_le_bytes(read_exact(self.cursor.read_exact(4)?)?))
    }

    fn id(&mut self) -> Result<PersistentId, CognitionContractError> {
        Ok(PersistentId::from_bytes(read_exact(
            self.cursor.read_exact(16)?,
        )?))
    }

    fn hash(&mut self) -> Result<ContentHash, CognitionContractError> {
        Ok(ContentHash::from_bytes(read_exact(
            self.cursor.read_exact(32)?,
        )?))
    }

    fn optional_hash(&mut self) -> Result<Option<ContentHash>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.hash()?)),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn count(&mut self, limit: usize) -> Result<usize, CognitionContractError> {
        Ok(self
            .cursor
            .read_count(limit, |actual, limit| CanonicalDecodeError::TooManyFields {
                actual,
                limit,
            })?)
    }

    fn text(&mut self) -> Result<&'a str, CognitionContractError> {
        let bytes = self
            .cursor
            .read_u32_length_prefixed(COGNITION_MAX_TEXT_BYTES)?;
        Ok(std::str::from_utf8(bytes).map_err(|_| CanonicalDecodeError::InvalidUtf8)?)
    }

    fn schema_id(&mut self) -> Result<SchemaId, CognitionContractError> {
        Ok(SchemaId::new(self.text()?)?)
    }

    fn bytes(&mut self) -> Result<&'a [u8], CognitionContractError> {
        let length =
            usize::try_from(self.u32()?).map_err(|_| CognitionContractError::LimitExceeded)?;
        if length > self.limits.max_field_payload_bytes {
            return Err(CognitionContractError::LimitExceeded);
        }
        Ok(self.cursor.read_exact(length)?)
    }

    fn beliefs(&mut self, limit: usize) -> Result<Vec<SemanticBeliefV1>, CognitionContractError> {
        (0..self.count(limit)?).map(|_| read_belief(self)).collect()
    }

    fn speech_acts(
        &mut self,
        limit: usize,
    ) -> Result<Vec<StructuredSpeechActV1>, CognitionContractError> {
        (0..self.count(limit)?)
            .map(|_| {
                StructuredSpeechActV1::from_canonical_payload_bytes(self.bytes()?, self.limits)
            })
            .collect()
    }

    fn optional_active_goal(&mut self) -> Result<Option<ActiveGoalV1>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(read_active_goal(self)?)),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn suspended_goals(&mut self) -> Result<Vec<SuspendedGoalV1>, CognitionContractError> {
        (0..self.count(COGNITION_MAX_SUSPENDED_GOALS)?)
            .map(|_| {
                Ok(SuspendedGoalV1 {
                    goal: read_active_goal(self)?,
                    suspended_tick: self.u64()?,
                })
            })
            .collect()
    }

    fn optional_plan(&mut self) -> Result<Option<StrategicPlanV1>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(read_plan(self)?)),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn optional_task(&mut self) -> Result<Option<PrivateTaskStateV1>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(PrivateTaskStateV1 {
                action_id: self.schema_id()?,
                lifecycle: TaskLifecycleV1::from_tag(self.u8()?)?,
                started_tick: self.u64()?,
                attempt: self.u16()?,
            })),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn optional_intent(
        &mut self,
    ) -> Result<Option<StrategicAgentIntentV1>, CognitionContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(read_intent(self)?)),
            value => Err(CognitionContractError::UnknownTag(value)),
        }
    }

    fn finish(self) -> Result<(), CognitionContractError> {
        Ok(self.cursor.finish()?)
    }
}

fn write_belief_identity(
    writer: &mut Writer,
    belief: &SemanticBeliefV1,
) -> Result<(), CognitionContractError> {
    writer.id(belief.subject_id);
    writer.text(belief.predicate_id.as_str())?;
    writer.text(belief.value_id.as_str())?;
    writer.u32(belief.confidence_q16);
    writer.u8(belief.source as u8);
    writer.optional_hash(belief.source_act_id_or_none);
    writer.u64(belief.learned_tick);
    writer.u64(belief.last_verified_revision);
    writer.u8(belief.contradiction as u8);
    Ok(())
}

fn write_belief(
    writer: &mut Writer,
    belief: &SemanticBeliefV1,
) -> Result<(), CognitionContractError> {
    belief.validate()?;
    writer.hash(belief.belief_id);
    write_belief_identity(writer, belief)
}

fn read_belief(reader: &mut Reader<'_>) -> Result<SemanticBeliefV1, CognitionContractError> {
    let value = SemanticBeliefV1 {
        belief_id: reader.hash()?,
        subject_id: reader.id()?,
        predicate_id: reader.schema_id()?,
        value_id: reader.schema_id()?,
        confidence_q16: reader.u32()?,
        source: BeliefSourceV1::from_tag(reader.u8()?)?,
        source_act_id_or_none: reader.optional_hash()?,
        learned_tick: reader.u64()?,
        last_verified_revision: reader.u64()?,
        contradiction: BeliefContradictionV1::from_tag(reader.u8()?)?,
    };
    value.validate()?;
    Ok(value)
}

fn write_affordance(
    writer: &mut Writer,
    value: &SemanticAffordanceV1,
) -> Result<(), CognitionContractError> {
    value.validate()?;
    writer.text(value.action_id.as_str())?;
    writer.u64(value.owner_revision);
    writer.schema_ids(&value.precondition_fact_ids)?;
    writer.schema_ids(&value.effect_fact_ids)?;
    writer.u32(value.cost_q16);
    writer.u8(value.execution as u8);
    writer.optional_hash(value.route_plan_hash_or_none);
    write_optional_schema_id(writer, value.target_id_or_none.as_ref())
}

fn write_goal_candidate(
    writer: &mut Writer,
    value: &GoalCandidateV1,
) -> Result<(), CognitionContractError> {
    value.validate()?;
    writer.text(value.goal_id.as_str())?;
    writer.u8(value.priority_band as u8);
    write_optional_schema_id(writer, value.target_id_or_none.as_ref())?;
    writer.text(value.desired_fact_id.as_str())?;
    writer.i32(value.drive_utility_q16);
    writer.i32(value.aspiration_utility_q16);
    writer.i32(value.inertia_utility_q16);
    writer.i32(value.risk_cost_q16);
    writer.i32(value.expected_cost_q16);
    writer.i32(value.total_utility_q16);
    writer.count(value.cited_belief_ids.len())?;
    for belief_id in &value.cited_belief_ids {
        writer.hash(*belief_id);
    }
    Ok(())
}

fn write_active_goal(
    writer: &mut Writer,
    value: &ActiveGoalV1,
) -> Result<(), CognitionContractError> {
    writer.text(value.goal_id.as_str())?;
    writer.u8(value.priority_band as u8);
    write_optional_schema_id(writer, value.target_id_or_none.as_ref())?;
    writer.text(value.desired_fact_id.as_str())?;
    writer.i32(value.selected_utility_q16);
    writer.u64(value.selected_tick);
    Ok(())
}

fn read_active_goal(reader: &mut Reader<'_>) -> Result<ActiveGoalV1, CognitionContractError> {
    Ok(ActiveGoalV1 {
        goal_id: reader.schema_id()?,
        priority_band: GoalPriorityBandV1::from_tag(reader.u8()?)?,
        target_id_or_none: read_optional_schema_id(reader)?,
        desired_fact_id: reader.schema_id()?,
        selected_utility_q16: reader.i32()?,
        selected_tick: reader.u64()?,
    })
}

fn write_optional_active_goal(
    writer: &mut Writer,
    value: Option<&ActiveGoalV1>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            write_active_goal(writer, value)?;
        }
    }
    Ok(())
}

fn write_plan(writer: &mut Writer, value: &StrategicPlanV1) -> Result<(), CognitionContractError> {
    value.validate()?;
    writer.text(value.goal_id.as_str())?;
    writer.text(value.desired_fact_id.as_str())?;
    writer.count(value.steps.len())?;
    for step in &value.steps {
        writer.text(step.action_id.as_str())?;
        writer.u64(step.owner_revision);
        writer.u32(step.cost_q16);
        writer.u8(step.execution as u8);
        writer.optional_hash(step.route_plan_hash_or_none);
        write_optional_schema_id(writer, step.target_id_or_none.as_ref())?;
    }
    writer.u8(value.cursor);
    writer.u16(value.expanded_nodes);
    writer.u8(value.planner_max_depth);
    writer.u16(value.planner_max_expanded_nodes);
    Ok(())
}

fn read_plan(reader: &mut Reader<'_>) -> Result<StrategicPlanV1, CognitionContractError> {
    let goal_id = reader.schema_id()?;
    let desired_fact_id = reader.schema_id()?;
    let steps = (0..reader.count(COGNITION_MAX_PLAN_STEPS)?)
        .map(|_| {
            Ok(StrategicPlanStepV1 {
                action_id: reader.schema_id()?,
                owner_revision: reader.u64()?,
                cost_q16: reader.u32()?,
                execution: AffordanceExecutionV1::from_tag(reader.u8()?)?,
                route_plan_hash_or_none: reader.optional_hash()?,
                target_id_or_none: read_optional_schema_id(reader)?,
            })
        })
        .collect::<Result<Vec<_>, CognitionContractError>>()?;
    let value = StrategicPlanV1 {
        goal_id,
        desired_fact_id,
        steps,
        cursor: reader.u8()?,
        expanded_nodes: reader.u16()?,
        planner_max_depth: reader.u8()?,
        planner_max_expanded_nodes: reader.u16()?,
    };
    value.validate()?;
    Ok(value)
}

fn write_optional_plan(
    writer: &mut Writer,
    value: Option<&StrategicPlanV1>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            write_plan(writer, value)?;
        }
    }
    Ok(())
}

fn write_optional_task(
    writer: &mut Writer,
    value: Option<&PrivateTaskStateV1>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            writer.text(value.action_id.as_str())?;
            writer.u8(value.lifecycle as u8);
            writer.u64(value.started_tick);
            writer.u16(value.attempt);
        }
    }
    Ok(())
}

fn write_intent_identity(
    writer: &mut Writer,
    value: &StrategicAgentIntentV1,
) -> Result<(), CognitionContractError> {
    writer.u16(value.schema_version);
    writer.id(value.subject_id);
    writer.text(value.goal_id.as_str())?;
    writer.text(value.action_id.as_str())?;
    writer.u64(value.creation_tick);
    writer.u64(value.expiry_tick);
    match &value.kind {
        StrategicAgentIntentKindV1::RequestLogicalRoute {
            start_node_id,
            goal_node_id,
            route_plan_hash,
        } => {
            writer.u8(1);
            writer.text(start_node_id.as_str())?;
            writer.text(goal_node_id.as_str())?;
            writer.hash(*route_plan_hash);
        }
        StrategicAgentIntentKindV1::HoldPosition => writer.u8(2),
        StrategicAgentIntentKindV1::CommitSocialExchange {
            exchange_hash,
            commitment_id,
        } => {
            writer.u8(3);
            writer.hash(*exchange_hash);
            writer.id(*commitment_id);
        }
        StrategicAgentIntentKindV1::AwaitActivity {
            commitment_id,
            expected_activity_revision,
        } => {
            writer.u8(4);
            writer.id(*commitment_id);
            writer.u64(*expected_activity_revision);
        }
        StrategicAgentIntentKindV1::SettleSystemicExchange {
            commitment_id,
            expected_activity_revision,
        } => {
            writer.u8(5);
            writer.id(*commitment_id);
            writer.u64(*expected_activity_revision);
        }
    }
    Ok(())
}

fn write_intent(
    writer: &mut Writer,
    value: &StrategicAgentIntentV1,
) -> Result<(), CognitionContractError> {
    value.validate()?;
    writer.hash(value.intent_id);
    write_intent_identity(writer, value)
}

fn read_intent(reader: &mut Reader<'_>) -> Result<StrategicAgentIntentV1, CognitionContractError> {
    let intent_id = reader.hash()?;
    let schema_version = reader.u16()?;
    let subject_id = reader.id()?;
    let goal_id = reader.schema_id()?;
    let action_id = reader.schema_id()?;
    let creation_tick = reader.u64()?;
    let expiry_tick = reader.u64()?;
    let kind = match reader.u8()? {
        1 => StrategicAgentIntentKindV1::RequestLogicalRoute {
            start_node_id: reader.schema_id()?,
            goal_node_id: reader.schema_id()?,
            route_plan_hash: reader.hash()?,
        },
        2 => StrategicAgentIntentKindV1::HoldPosition,
        3 => StrategicAgentIntentKindV1::CommitSocialExchange {
            exchange_hash: reader.hash()?,
            commitment_id: reader.id()?,
        },
        4 => StrategicAgentIntentKindV1::AwaitActivity {
            commitment_id: reader.id()?,
            expected_activity_revision: reader.u64()?,
        },
        5 => StrategicAgentIntentKindV1::SettleSystemicExchange {
            commitment_id: reader.id()?,
            expected_activity_revision: reader.u64()?,
        },
        value => return Err(CognitionContractError::UnknownTag(value)),
    };
    let value = StrategicAgentIntentV1 {
        schema_version,
        intent_id,
        subject_id,
        goal_id,
        action_id,
        creation_tick,
        expiry_tick,
        kind,
    };
    value.validate()?;
    Ok(value)
}

fn write_optional_intent(
    writer: &mut Writer,
    value: Option<&StrategicAgentIntentV1>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            write_intent(writer, value)?;
        }
    }
    Ok(())
}

fn write_optional_schema_id(
    writer: &mut Writer,
    value: Option<&SchemaId>,
) -> Result<(), CognitionContractError> {
    match value {
        None => writer.u8(0),
        Some(value) => {
            writer.u8(1);
            writer.text(value.as_str())?;
        }
    }
    Ok(())
}

fn read_optional_schema_id(
    reader: &mut Reader<'_>,
) -> Result<Option<SchemaId>, CognitionContractError> {
    match reader.u8()? {
        0 => Ok(None),
        1 => Ok(Some(reader.schema_id()?)),
        value => Err(CognitionContractError::UnknownTag(value)),
    }
}

fn decode_snapshot_segment(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment_id: &str,
) -> Result<crate::canonical::DecodedCanonicalSegment, CognitionContractError> {
    let segment = decode_canonical_segment(bytes, limits)?;
    require_envelope(
        &segment.owner_id,
        &segment.schema_id,
        &segment.segment_id,
        owner,
        schema,
        segment_id,
    )?;
    require_fields(
        &segment.fields,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_ID128),
            (3, CANONICAL_TYPE_BYTES),
        ],
    )?;
    Ok(segment)
}

fn require_envelope(
    owner: &str,
    schema: &str,
    segment: &str,
    expected_owner: &str,
    expected_schema: &str,
    expected_segment: &str,
) -> Result<(), CognitionContractError> {
    if owner != expected_owner || schema != expected_schema || segment != expected_segment {
        return Err(CognitionContractError::WrongEnvelope);
    }
    Ok(())
}

fn require_fields(
    actual: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), CognitionContractError> {
    for field in actual {
        let Some((_, tag)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(CognitionContractError::UnknownField(field.field_id));
        };
        if field.type_tag != *tag {
            return Err(CognitionContractError::FieldType);
        }
    }
    for (id, _) in expected {
        if !actual.iter().any(|field| field.field_id == *id) {
            return Err(CognitionContractError::MissingField(*id));
        }
    }
    Ok(())
}

fn field(fields: &[CanonicalField], id: u32) -> Result<&[u8], CognitionContractError> {
    Ok(&fields
        .iter()
        .find(|field| field.field_id == id)
        .ok_or(CognitionContractError::MissingField(id))?
        .payload)
}

fn read_exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], CognitionContractError> {
    bytes
        .try_into()
        .map_err(|_| CognitionContractError::NonCanonicalEncoding)
}

fn read_u16_exact(bytes: &[u8]) -> Result<u16, CognitionContractError> {
    Ok(u16::from_le_bytes(read_exact(bytes)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test schema id")
    }

    fn catalog() -> AgentCognitionCatalogV1 {
        let subject_id = PersistentId::from_bytes([7; 16]);
        let belief = SemanticBeliefV1::new(
            subject_id,
            schema("knowledge.route-purpose"),
            schema("knowledge.frontier-duty"),
            u32::try_from(COGNITION_Q16_ONE).expect("positive"),
            BeliefSourceV1::AuthoredSeed,
            0,
            0,
            BeliefContradictionV1::Consistent,
        )
        .expect("belief");
        AgentCognitionCatalogV1 {
            schema_version: COGNITION_SCHEMA_VERSION,
            catalog_asset_id: AssetId::from_bytes([8; 16]),
            subject_id,
            evaluation_start_tick: 1,
            evaluation_period_ticks: 3,
            retrieval_limit: 4,
            goal_switch_threshold_q16: COGNITION_Q16_ONE / 4,
            emergency_health_threshold: 30,
            planner_max_depth: 2,
            planner_max_expanded_nodes: 8,
            ordinary_goal_id: schema("goal.reach-frontier"),
            emergency_goal_id: schema("goal.preserve-self"),
            navigate_action_id: schema("affordance.request-logical-route"),
            hold_action_id: schema("affordance.hold-position"),
            route_known_fact_id: schema("fact.route-known"),
            travel_needed_fact_id: schema("fact.travel-needed"),
            emergency_fact_id: schema("fact.emergency"),
            navigate_ready_fact_id: schema("fact.navigate-intent-ready"),
            hold_ready_fact_id: schema("fact.hold-intent-ready"),
            seed_beliefs: vec![belief],
        }
    }

    #[test]
    fn cognition_catalog_and_owner_snapshots_round_trip() {
        let catalog = catalog();
        let bytes = catalog.canonical_bytes().expect("catalog bytes");
        assert_eq!(
            AgentCognitionCatalogV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("catalog round trip"),
            catalog
        );

        let memory = AgentMemorySnapshotV1::initial(&catalog).expect("memory");
        let memory_bytes = memory.canonical_bytes().expect("memory bytes");
        assert_eq!(
            AgentMemorySnapshotV1::from_canonical_bytes(
                &memory_bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("memory round trip"),
            memory
        );

        let agent = AgentCognitionSnapshotV1::initial(catalog.subject_id, 17);
        let agent_bytes = agent.canonical_bytes().expect("agent bytes");
        assert_eq!(
            AgentCognitionSnapshotV1::from_canonical_bytes(
                &agent_bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("agent round trip"),
            agent
        );
    }

    #[test]
    fn memory_retrieval_is_bounded_and_canonical() {
        let catalog = catalog();
        let memory = AgentMemorySnapshotV1::initial(&catalog).expect("memory");
        let first = memory.retrieve(1).expect("retrieve");
        assert_eq!(first, catalog.seed_beliefs);
        assert_eq!(
            memory.retrieve(0),
            Err(CognitionContractError::LimitExceeded)
        );
    }
}
