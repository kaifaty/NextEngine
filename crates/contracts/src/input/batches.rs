use std::collections::{BTreeMap, BTreeSet};

use crate::canonical::*;
use crate::command::{CommandPhase, WorldCommand};
use crate::ids::*;

use super::actions::InputSampleV1;
use super::codec::*;
use super::constants::*;
use super::ingress::{IngressAssignmentV1, IngressEquivalenceReceiptV1};
use super::profiles::RuntimeAdmissionLimitsV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedIngressBatchBodyV1 {
    pub schema_version: u16,
    pub queue_generation: u64,
    pub assigned_tick: u64,
    pub input_samples: Vec<InputSampleV1>,
    pub completion_signals: Vec<Vec<u8>>,
    pub input_assignments: Vec<IngressAssignmentV1>,
    pub completion_assignments: Vec<Vec<u8>>,
    pub equivalence_receipts: Vec<IngressEquivalenceReceiptV1>,
}

impl ClosedIngressBatchBodyV1 {
    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != CLOSED_INGRESS_BATCH_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "closed ingress batch body",
                version: u32::from(self.schema_version),
            });
        }
        if !self.completion_signals.is_empty() || !self.completion_assignments.is_empty() {
            return Err(InputContractError::UnsupportedCompletionSignal);
        }
        if self.input_samples.len()
            > usize::try_from(admission.max_commands_per_closed_batch)
                .map_err(|_| InputContractError::ResourceLimit)?
        {
            return Err(InputContractError::ResourceLimit);
        }
        for sample in &self.input_samples {
            sample.validate(admission)?;
        }
        let mut candidates =
            BTreeMap::<(SchemaId, InputSourceId, u64), BTreeSet<ContentHash>>::new();
        let mut previous = None;
        for sample in &self.input_samples {
            let sort_key = sample.sort_key()?;
            if previous
                .as_ref()
                .is_some_and(|previous| previous >= &sort_key)
            {
                return Err(InputContractError::NonCanonicalOrder);
            }
            previous = Some(sort_key);
            candidates
                .entry((
                    sample.source_class.clone(),
                    sample.source_id,
                    sample.source_sequence,
                ))
                .or_default()
                .insert(sample.payload_hash()?);
        }
        let mut expected_assignments = Vec::new();
        let mut expected_receipts = Vec::new();
        for ((source_class, source_id, source_sequence), hashes) in candidates {
            if hashes.len() == 1 {
                expected_assignments.push(IngressAssignmentV1 {
                    schema_version: INGRESS_ASSIGNMENT_SCHEMA_VERSION,
                    queue_generation: self.queue_generation,
                    assigned_tick: self.assigned_tick,
                    source_class,
                    source_id,
                    source_sequence,
                    payload_hash: *hashes
                        .first()
                        .expect("a candidate group always contains a payload hash"),
                });
            } else {
                expected_receipts.push(IngressEquivalenceReceiptV1::for_input_collision(
                    self.assigned_tick,
                    &source_class,
                    source_id,
                    source_sequence,
                    hashes,
                )?);
            }
        }
        if self.input_assignments != expected_assignments
            || self.equivalence_receipts != expected_receipts
        {
            return Err(InputContractError::InvalidValue);
        }
        if self
            .input_assignments
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
            || self
                .equivalence_receipts
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(InputContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            CLOSED_INGRESS_BODY_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u64(2, self.queue_generation),
                field_u64(3, self.assigned_tick),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.input_samples
                            .iter()
                            .map(InputSampleV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
                CanonicalField::new(5, CANONICAL_TYPE_SEQUENCE, encode_sequence(Vec::new())?),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.input_assignments
                            .iter()
                            .map(IngressAssignmentV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
                CanonicalField::new(7, CANONICAL_TYPE_SEQUENCE, encode_sequence(Vec::new())?),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.equivalence_receipts
                            .iter()
                            .map(IngressEquivalenceReceiptV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
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
            CLOSED_INGRESS_BODY_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_SEQUENCE),
                (6, CANONICAL_TYPE_SEQUENCE),
                (7, CANONICAL_TYPE_SEQUENCE),
                (8, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            queue_generation: read_u64(&segment, 2)?,
            assigned_tick: read_u64(&segment, 3)?,
            input_samples: decode_sequence(field(&segment, 4)?, limits)?
                .into_iter()
                .map(|bytes| InputSampleV1::from_canonical_bytes(&bytes, limits, admission))
                .collect::<Result<Vec<_>, _>>()?,
            completion_signals: decode_sequence(field(&segment, 5)?, limits)?,
            input_assignments: decode_sequence(field(&segment, 6)?, limits)?
                .into_iter()
                .map(|bytes| IngressAssignmentV1::from_canonical_bytes(&bytes, limits))
                .collect::<Result<Vec<_>, _>>()?,
            completion_assignments: decode_sequence(field(&segment, 7)?, limits)?,
            equivalence_receipts: decode_sequence(field(&segment, 8)?, limits)?
                .into_iter()
                .map(|bytes| IngressEquivalenceReceiptV1::from_canonical_bytes(&bytes, limits))
                .collect::<Result<Vec<_>, _>>()?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedIngressBatchV1 {
    pub schema_version: u16,
    pub body: ClosedIngressBatchBodyV1,
    pub batch_hash: ContentHash,
}

impl ClosedIngressBatchV1 {
    pub fn from_body(body: ClosedIngressBatchBodyV1) -> Result<Self, CanonicalError> {
        let batch_hash = closed_batch_hash(
            b"nextengine.closed-ingress-batch.v1\0",
            &body.canonical_bytes()?,
        )?;
        Ok(Self {
            schema_version: CLOSED_INGRESS_BATCH_SCHEMA_VERSION,
            body,
            batch_hash,
        })
    }

    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != CLOSED_INGRESS_BATCH_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "closed ingress batch",
                version: u32::from(self.schema_version),
            });
        }
        self.body.validate(admission)?;
        let expected = closed_batch_hash(
            b"nextengine.closed-ingress-batch.v1\0",
            &self.body.canonical_bytes()?,
        )?;
        if expected != self.batch_hash {
            return Err(InputContractError::HashMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            CLOSED_INGRESS_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_BYTES, self.body.canonical_bytes()?),
                field_hash(3, self.batch_hash),
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
            CLOSED_INGRESS_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            body: ClosedIngressBatchBodyV1::from_canonical_bytes(
                field(&segment, 2)?,
                limits,
                admission,
            )?,
            batch_hash: read_hash(&segment, 3)?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedCommandAdmissionBatchBodyV2 {
    pub schema_version: u16,
    pub simulation_tick: u64,
    pub phase: CommandPhase,
    pub batch_ordinal: u32,
    pub envelopes: Vec<WorldCommand>,
}

impl ClosedCommandAdmissionBatchBodyV2 {
    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "closed command batch body",
                version: u32::from(self.schema_version),
            });
        }
        if self.envelopes.len()
            > usize::try_from(admission.max_commands_per_closed_batch)
                .map_err(|_| InputContractError::ResourceLimit)?
        {
            return Err(InputContractError::ResourceLimit);
        }
        let mut previous = None;
        for command in &self.envelopes {
            let key = (
                command.stream_id,
                command.sequence,
                command.body_hash()?,
                command.claimed_command_id,
            );
            if previous.as_ref().is_some_and(|previous| previous > &key) {
                return Err(InputContractError::NonCanonicalOrder);
            }
            previous = Some(key);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let envelopes = self
            .envelopes
            .iter()
            .map(encode_command_envelope)
            .collect::<Result<Vec<_>, _>>()?;
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            CLOSED_COMMAND_BODY_SCHEMA_ID,
            SEGMENT_V2,
            [
                field_u16(1, self.schema_version),
                field_u64(2, self.simulation_tick),
                field_u8(3, self.phase as u8),
                field_u32(4, self.batch_ordinal),
                CanonicalField::new(5, CANONICAL_TYPE_SEQUENCE, encode_sequence(envelopes)?),
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
            CLOSED_COMMAND_BODY_SCHEMA_ID,
            SEGMENT_V2,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U8),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            simulation_tick: read_u64(&segment, 2)?,
            phase: match read_u8(&segment, 3)? {
                0 => CommandPhase::Ingress,
                1 => CommandPhase::Outcome,
                tag => return Err(InputContractError::UnknownTag(tag)),
            },
            batch_ordinal: read_u32(&segment, 4)?,
            envelopes: decode_sequence(field(&segment, 5)?, limits)?
                .into_iter()
                .map(|bytes| decode_command_envelope(&bytes, limits))
                .collect::<Result<Vec<_>, _>>()?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedCommandAdmissionBatchV2 {
    pub schema_version: u16,
    pub body: ClosedCommandAdmissionBatchBodyV2,
    pub batch_hash: ContentHash,
}

impl ClosedCommandAdmissionBatchV2 {
    pub fn from_body(body: ClosedCommandAdmissionBatchBodyV2) -> Result<Self, CanonicalError> {
        Ok(Self {
            schema_version: CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION,
            batch_hash: closed_batch_hash(
                b"nextengine.closed-command-admission-batch.v2\0",
                &body.canonical_bytes()?,
            )?,
            body,
        })
    }

    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "closed command batch",
                version: u32::from(self.schema_version),
            });
        }
        self.body.validate(admission)?;
        if self.batch_hash
            != closed_batch_hash(
                b"nextengine.closed-command-admission-batch.v2\0",
                &self.body.canonical_bytes()?,
            )?
        {
            return Err(InputContractError::HashMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            CLOSED_COMMAND_SCHEMA_ID,
            SEGMENT_V2,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_BYTES, self.body.canonical_bytes()?),
                field_hash(3, self.batch_hash),
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
            CLOSED_COMMAND_SCHEMA_ID,
            SEGMENT_V2,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            body: ClosedCommandAdmissionBatchBodyV2::from_canonical_bytes(
                field(&segment, 2)?,
                limits,
                admission,
            )?,
            batch_hash: read_hash(&segment, 3)?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}
