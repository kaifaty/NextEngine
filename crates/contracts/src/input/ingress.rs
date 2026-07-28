use std::collections::BTreeSet;

use crate::canonical::*;
use crate::ids::*;

use super::actions::InputSampleV1;
use super::codec::*;
use super::constants::*;
use super::profiles::RuntimeAdmissionLimitsV1;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IngressAssignmentV1 {
    pub schema_version: u16,
    pub queue_generation: u64,
    pub assigned_tick: u64,
    pub source_class: SchemaId,
    pub source_id: InputSourceId,
    pub source_sequence: u64,
    pub payload_hash: ContentHash,
}

impl IngressAssignmentV1 {
    pub fn from_sample(
        queue_generation: u64,
        assigned_tick: u64,
        sample: &InputSampleV1,
    ) -> Result<Self, CanonicalError> {
        Ok(Self {
            schema_version: INGRESS_ASSIGNMENT_SCHEMA_VERSION,
            queue_generation,
            assigned_tick,
            source_class: sample.source_class.clone(),
            source_id: sample.source_id,
            source_sequence: sample.source_sequence,
            payload_hash: sample.payload_hash()?,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INGRESS_ASSIGNMENT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u64(2, self.queue_generation),
                field_u64(3, self.assigned_tick),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.source_class.as_str().as_bytes().to_vec(),
                ),
                field_id(5, self.source_id.as_bytes()),
                field_u64(6, self.source_sequence),
                field_hash(7, self.payload_hash),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INGRESS_ASSIGNMENT_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_UTF8_NFC),
                (5, CANONICAL_TYPE_ID128),
                (6, CANONICAL_TYPE_U64),
                (7, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            queue_generation: read_u64(&segment, 2)?,
            assigned_tick: read_u64(&segment, 3)?,
            source_class: SchemaId::new(read_utf8(&segment, 4)?)?,
            source_id: InputSourceId::from_bytes(exact(field(&segment, 5)?)?),
            source_sequence: read_u64(&segment, 6)?,
            payload_hash: read_hash(&segment, 7)?,
        };
        if value.schema_version != INGRESS_ASSIGNMENT_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "ingress assignment",
                version: u32::from(value.schema_version),
            });
        }
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum IngressSubjectKindV1 {
    Input = 1,
    Completion = 2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum IngressResultCodeV1 {
    InputSequenceCollision = 1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IngressEquivalenceReceiptV1 {
    pub assigned_tick: u64,
    pub subject_kind: IngressSubjectKindV1,
    pub subject_id: Vec<u8>,
    pub candidate_hashes: BTreeSet<ContentHash>,
    pub result_code: IngressResultCodeV1,
}

impl IngressEquivalenceReceiptV1 {
    pub fn for_input_collision(
        assigned_tick: u64,
        source_class: &SchemaId,
        source_id: InputSourceId,
        source_sequence: u64,
        candidate_hashes: BTreeSet<ContentHash>,
    ) -> Result<Self, CanonicalError> {
        let source_class = source_class.as_str().as_bytes();
        let mut subject_id = Vec::new();
        subject_id.extend_from_slice(
            &u32::try_from(source_class.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        subject_id.extend_from_slice(source_class);
        subject_id.extend_from_slice(source_id.as_bytes());
        subject_id.extend_from_slice(&source_sequence.to_le_bytes());
        Ok(Self {
            assigned_tick,
            subject_kind: IngressSubjectKindV1::Input,
            subject_id,
            candidate_hashes,
            result_code: IngressResultCodeV1::InputSequenceCollision,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let hashes = encode_sequence(
            self.candidate_hashes
                .iter()
                .map(|hash| hash.as_bytes().to_vec())
                .collect(),
        )?;
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INGRESS_RECEIPT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u64(1, self.assigned_tick),
                field_u8(2, self.subject_kind as u8),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, self.subject_id.clone()),
                CanonicalField::new(4, CANONICAL_TYPE_SEQUENCE, hashes),
                field_u8(5, self.result_code as u8),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INGRESS_RECEIPT_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U64),
                (2, CANONICAL_TYPE_U8),
                (3, CANONICAL_TYPE_BYTES),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_U8),
            ],
        )?;
        let candidate_hashes = decode_sequence(field(&segment, 4)?, limits)?
            .into_iter()
            .map(|bytes| Ok(content_hash_from_bytes(exact(&bytes)?)))
            .collect::<Result<BTreeSet<_>, InputContractError>>()?;
        if candidate_hashes.len() < 2 {
            return Err(InputContractError::InvalidValue);
        }
        let value = Self {
            assigned_tick: read_u64(&segment, 1)?,
            subject_kind: match read_u8(&segment, 2)? {
                1 => IngressSubjectKindV1::Input,
                2 => IngressSubjectKindV1::Completion,
                tag => return Err(InputContractError::UnknownTag(tag)),
            },
            subject_id: field(&segment, 3)?.to_vec(),
            candidate_hashes,
            result_code: match read_u8(&segment, 5)? {
                1 => IngressResultCodeV1::InputSequenceCollision,
                tag => return Err(InputContractError::UnknownTag(tag)),
            },
        };
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]

pub struct IngressCheckpointV1 {
    pub schema_version: u16,
    pub current_tick: u64,
    pub current_generation: u64,
    pub current_samples: Vec<InputSampleV1>,
    pub next_samples: Vec<InputSampleV1>,
    pub last_closed_batch_hash: Option<ContentHash>,
}

impl IngressCheckpointV1 {
    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != INGRESS_CHECKPOINT_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "ingress checkpoint",
                version: u32::from(self.schema_version),
            });
        }
        let max = usize::try_from(admission.max_commands_per_closed_batch)
            .map_err(|_| InputContractError::ResourceLimit)?;
        if self.current_samples.len() > max || self.next_samples.len() > max {
            return Err(InputContractError::ResourceLimit);
        }
        for sample in self.current_samples.iter().chain(&self.next_samples) {
            sample.validate(admission)?;
        }
        if !is_non_decreasing_by(&self.current_samples, InputSampleV1::sort_key)
            || !is_non_decreasing_by(&self.next_samples, InputSampleV1::sort_key)
        {
            return Err(InputContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INGRESS_CHECKPOINT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u64(2, self.current_tick),
                field_u64(3, self.current_generation),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.current_samples
                            .iter()
                            .map(InputSampleV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.next_samples
                            .iter()
                            .map(InputSampleV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_OPTIONAL,
                    encode_optional_hash(self.last_closed_batch_hash.as_ref())?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
        admission: &RuntimeAdmissionLimitsV1,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INGRESS_CHECKPOINT_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_SEQUENCE),
                (6, CANONICAL_TYPE_OPTIONAL),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            current_tick: read_u64(&segment, 2)?,
            current_generation: read_u64(&segment, 3)?,
            current_samples: decode_sequence(field(&segment, 4)?, limits)?
                .into_iter()
                .map(|bytes| InputSampleV1::from_canonical_bytes(&bytes, limits, admission))
                .collect::<Result<Vec<_>, _>>()?,
            next_samples: decode_sequence(field(&segment, 5)?, limits)?
                .into_iter()
                .map(|bytes| InputSampleV1::from_canonical_bytes(&bytes, limits, admission))
                .collect::<Result<Vec<_>, _>>()?,
            last_closed_batch_hash: decode_optional_hash(field(&segment, 6)?, limits)?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}
