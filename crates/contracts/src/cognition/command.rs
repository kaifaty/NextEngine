use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AgentCognitionCommandV1 {
    CommitDecision {
        expected_agent_revision: u64,
        expected_memory_revision: u64,
        cognition_catalog_revision: ContentHash,
        epistemic_view_hash: ContentHash,
        next_agent_snapshot: AgentCognitionSnapshotV1,
        next_memory_snapshot: AgentMemorySnapshotV1,
    },
}

impl AgentCognitionCommandV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        match self {
            Self::CommitDecision {
                expected_agent_revision,
                expected_memory_revision,
                cognition_catalog_revision,
                epistemic_view_hash,
                next_agent_snapshot,
                next_memory_snapshot,
            } => {
                next_agent_snapshot.validate()?;
                next_memory_snapshot.validate()?;
                if next_agent_snapshot.subject_id != next_memory_snapshot.subject_id
                    || expected_agent_revision.checked_add(1) != Some(next_agent_snapshot.revision)
                    || expected_memory_revision.checked_add(1)
                        != Some(next_memory_snapshot.revision)
                    || *cognition_catalog_revision == ContentHash::default()
                    || *epistemic_view_hash == ContentHash::default()
                    || next_agent_snapshot.last_epistemic_hash != *epistemic_view_hash
                {
                    return Err(CognitionContractError::CommandInvalid);
                }
            }
        }
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate()
            .map_err(|_| CanonicalError::LengthOverflow)?;
        let mut writer = Writer::with_domain(b"nextengine.agent-cognition-command.v1\0");
        writer.u16(COGNITION_SCHEMA_VERSION);
        match self {
            Self::CommitDecision {
                expected_agent_revision,
                expected_memory_revision,
                cognition_catalog_revision,
                epistemic_view_hash,
                next_agent_snapshot,
                next_memory_snapshot,
            } => {
                writer.u8(1);
                writer.u64(*expected_agent_revision);
                writer.u64(*expected_memory_revision);
                writer.hash(*cognition_catalog_revision);
                writer.hash(*epistemic_view_hash);
                writer.bytes(
                    &next_agent_snapshot
                        .canonical_bytes()
                        .map_err(|_| CanonicalError::LengthOverflow)?,
                )?;
                writer.bytes(
                    &next_memory_snapshot
                        .canonical_bytes()
                        .map_err(|_| CanonicalError::LengthOverflow)?,
                )?;
            }
        }
        Ok(writer.into_bytes())
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CognitionContractError> {
        let mut reader = Reader::new(bytes, limits);
        reader.domain(b"nextengine.agent-cognition-command.v1\0")?;
        if reader.u16()? != COGNITION_SCHEMA_VERSION || reader.u8()? != 1 {
            return Err(CognitionContractError::CommandInvalid);
        }
        let value = Self::CommitDecision {
            expected_agent_revision: reader.u64()?,
            expected_memory_revision: reader.u64()?,
            cognition_catalog_revision: reader.hash()?,
            epistemic_view_hash: reader.hash()?,
            next_agent_snapshot: AgentCognitionSnapshotV1::from_canonical_bytes(
                reader.bytes()?,
                limits,
            )?,
            next_memory_snapshot: AgentMemorySnapshotV1::from_canonical_bytes(
                reader.bytes()?,
                limits,
            )?,
        };
        reader.finish()?;
        value.validate()?;
        if value.canonical_payload_bytes()? != bytes {
            return Err(CognitionContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AgentDecisionCommittedV1 {
    pub schema_version: u16,
    pub subject_id: PersistentId,
    pub agent_revision: u64,
    pub memory_revision: u64,
    pub active_goal_id: SchemaId,
    pub intent_id: ContentHash,
}

impl AgentDecisionCommittedV1 {
    pub fn validate(&self) -> Result<(), CognitionContractError> {
        if self.schema_version != COGNITION_SCHEMA_VERSION
            || self.agent_revision == 0
            || self.memory_revision == 0
            || self.intent_id == ContentHash::default()
        {
            return Err(CognitionContractError::EventInvalid);
        }
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate()
            .map_err(|_| CanonicalError::LengthOverflow)?;
        let mut writer = Writer::with_domain(b"nextengine.agent-decision-committed.v1\0");
        writer.u16(self.schema_version);
        writer.id(self.subject_id);
        writer.u64(self.agent_revision);
        writer.u64(self.memory_revision);
        writer
            .text(self.active_goal_id.as_str())
            .map_err(|_| CanonicalError::LengthOverflow)?;
        writer.hash(self.intent_id);
        Ok(writer.into_bytes())
    }
}
