use super::wire::*;
use super::*;

pub(super) fn encode_streams(
    streams: &BTreeMap<CommandStreamId, CommandStreamLedgerV2>,
) -> Result<Vec<u8>, CommandLedgerError> {
    let mut writer = LedgerWriter::default();
    writer.count(streams.len())?;
    for (stream_id, stream) in streams {
        writer.bytes(stream_id.as_bytes());
        encode_stream(&mut writer, stream)?;
    }
    Ok(writer.finish())
}

pub(super) fn decode_streams(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<BTreeMap<CommandStreamId, CommandStreamLedgerV2>, CommandLedgerError> {
    let mut reader = LedgerReader::new(bytes, limits);
    let count = reader.count()?;
    let mut streams = BTreeMap::new();
    let mut previous = None;
    for _ in 0..count {
        let stream_id = CommandStreamId::from_bytes(reader.array()?);
        if previous.is_some_and(|prior| prior >= stream_id) {
            return Err(CommandLedgerError::MapNotStrictlySorted);
        }
        let stream = decode_stream(&mut reader)?;
        if stream.stream_id != stream_id || streams.insert(stream_id, stream).is_some() {
            return Err(CommandLedgerError::StreamKeyMismatch);
        }
        previous = Some(stream_id);
    }
    reader.finish()?;
    Ok(streams)
}

fn encode_stream(
    writer: &mut LedgerWriter,
    stream: &CommandStreamLedgerV2,
) -> Result<(), CommandLedgerError> {
    writer.u16(stream.schema_version);
    writer.bytes(stream.stream_id.as_bytes());
    writer.principal(&stream.issuer)?;
    writer.u32(stream.stream_slot);
    writer.u32(stream.stream_epoch);
    writer.u8(stream.state as u8);
    writer.option_u64(stream.admission_high_watermark);
    writer.option_u64(stream.greatest_reserved_target_tick);
    writer.count(stream.pending.len())?;
    for (sequence, reservation) in &stream.pending {
        writer.u64(*sequence);
        encode_reservation(writer, reservation)?;
    }
    writer.count(stream.receipt_window.len())?;
    for receipt in &stream.receipt_window {
        encode_receipt(writer, receipt)?;
    }
    writer.u64(stream.finalized_receipt_count);
    writer.bytes(stream.receipt_chain_root.as_bytes());
    match &stream.collision_incident {
        None => writer.u8(0),
        Some(incident) => {
            writer.u8(1);
            encode_incident(writer, incident)?;
        }
    }
    Ok(())
}

fn decode_stream(
    reader: &mut LedgerReader<'_>,
) -> Result<CommandStreamLedgerV2, CommandLedgerError> {
    let schema_version = reader.u16()?;
    if schema_version != COMMAND_STREAM_LEDGER_SCHEMA_VERSION {
        return Err(CommandLedgerError::UnsupportedStreamLedgerVersion(
            schema_version,
        ));
    }
    let stream_id = CommandStreamId::from_bytes(reader.array()?);
    let issuer = reader.principal()?;
    let stream_slot = reader.u32()?;
    let stream_epoch = reader.u32()?;
    let state = match reader.u8()? {
        0 => CommandStreamStateV1::Open,
        1 => CommandStreamStateV1::CollisionLocked,
        2 => CommandStreamStateV1::Exhausted,
        3 => CommandStreamStateV1::Closed,
        tag => return Err(CommandLedgerError::UnknownTag(tag)),
    };
    let admission_high_watermark = reader.option_u64()?;
    let greatest_reserved_target_tick = reader.option_u64()?;
    let pending_count = reader.count()?;
    if pending_count > COMMAND_PENDING_CAPACITY {
        return Err(CommandLedgerError::PendingLimit);
    }
    let mut pending = BTreeMap::new();
    let mut previous = None;
    for _ in 0..pending_count {
        let sequence = reader.u64()?;
        if previous.is_some_and(|prior| prior >= sequence) {
            return Err(CommandLedgerError::MapNotStrictlySorted);
        }
        let reservation = decode_reservation(reader)?;
        if reservation.sequence != sequence || pending.insert(sequence, reservation).is_some() {
            return Err(CommandLedgerError::ReservationMismatch);
        }
        previous = Some(sequence);
    }
    let receipt_count = reader.count()?;
    if receipt_count > COMMAND_RECEIPT_WINDOW_CAPACITY {
        return Err(CommandLedgerError::ReceiptWindowLengthMismatch);
    }
    let mut receipt_window = Vec::with_capacity(receipt_count);
    for _ in 0..receipt_count {
        receipt_window.push(decode_receipt(reader)?);
    }
    let finalized_receipt_count = reader.u64()?;
    let receipt_chain_root = ContentHash::from_bytes(reader.array()?);
    let collision_incident = match reader.u8()? {
        0 => None,
        1 => Some(decode_incident(reader)?),
        tag => return Err(CommandLedgerError::UnknownTag(tag)),
    };
    Ok(CommandStreamLedgerV2 {
        schema_version,
        stream_id,
        issuer,
        stream_slot,
        stream_epoch,
        state,
        admission_high_watermark,
        greatest_reserved_target_tick,
        pending,
        receipt_window,
        finalized_receipt_count,
        receipt_chain_root,
        collision_incident,
    })
}

fn encode_reservation(
    writer: &mut LedgerWriter,
    reservation: &CommandReservationV1,
) -> Result<(), CommandLedgerError> {
    writer.u16(reservation.schema_version);
    writer.bytes(reservation.stream_id.as_bytes());
    writer.principal(&reservation.issuer)?;
    writer.u64(reservation.sequence);
    writer.bytes(reservation.command_id.as_bytes());
    writer.bytes(reservation.body_hash.as_bytes());
    writer.bytes(reservation.canonical_body_ref.as_bytes());
    writer.u64(reservation.reserved_at_tick);
    writer.u64(reservation.target_tick);
    writer.u8(reservation.phase as u8);
    writer.u16(reservation.priority_class);
    writer.bytes(reservation.command_kind_registry_hash.as_bytes());
    Ok(())
}

fn decode_reservation(
    reader: &mut LedgerReader<'_>,
) -> Result<CommandReservationV1, CommandLedgerError> {
    let schema_version = reader.u16()?;
    if schema_version != COMMAND_RESERVATION_SCHEMA_VERSION {
        return Err(CommandLedgerError::ReservationMismatch);
    }
    Ok(CommandReservationV1 {
        schema_version,
        stream_id: CommandStreamId::from_bytes(reader.array()?),
        issuer: reader.principal()?,
        sequence: reader.u64()?,
        command_id: CommandId::from_bytes(reader.array()?),
        body_hash: CommandBodyHash::from_bytes(reader.array()?),
        canonical_body_ref: CommandBodyHash::from_bytes(reader.array()?),
        reserved_at_tick: reader.u64()?,
        target_tick: reader.u64()?,
        phase: decode_phase(reader.u8()?)?,
        priority_class: reader.u16()?,
        command_kind_registry_hash: ContentHash::from_bytes(reader.array()?),
    })
}

fn encode_candidate(writer: &mut LedgerWriter, candidate: &CommandCollisionCandidateV1) {
    writer.bytes(candidate.command_id.as_bytes());
    writer.bytes(candidate.body_hash.as_bytes());
    writer.bytes(candidate.canonical_body_ref.as_bytes());
}

fn decode_candidate(
    reader: &mut LedgerReader<'_>,
) -> Result<CommandCollisionCandidateV1, CommandLedgerError> {
    Ok(CommandCollisionCandidateV1 {
        command_id: CommandId::from_bytes(reader.array()?),
        body_hash: CommandBodyHash::from_bytes(reader.array()?),
        canonical_body_ref: CommandBodyHash::from_bytes(reader.array()?),
    })
}

fn encode_incident(
    writer: &mut LedgerWriter,
    incident: &CommandCollisionIncidentV1,
) -> Result<(), CommandLedgerError> {
    writer.bytes(incident.stream_id.as_bytes());
    writer.principal(&incident.issuer)?;
    writer.u64(incident.sequence);
    writer.count(incident.candidates.len())?;
    for candidate in &incident.candidates {
        encode_candidate(writer, candidate);
    }
    writer.bytes(incident.candidates_root.as_bytes());
    writer.bytes(incident.incident_digest.as_bytes());
    Ok(())
}

fn decode_incident(
    reader: &mut LedgerReader<'_>,
) -> Result<CommandCollisionIncidentV1, CommandLedgerError> {
    let stream_id = CommandStreamId::from_bytes(reader.array()?);
    let issuer = reader.principal()?;
    let sequence = reader.u64()?;
    let count = reader.count()?;
    let mut candidates = Vec::with_capacity(count);
    for _ in 0..count {
        candidates.push(decode_candidate(reader)?);
    }
    let incident = CommandCollisionIncidentV1 {
        stream_id,
        issuer,
        sequence,
        candidates,
        candidates_root: ContentHash::from_bytes(reader.array()?),
        incident_digest: ContentHash::from_bytes(reader.array()?),
    };
    incident.validate()?;
    Ok(incident)
}

fn encode_receipt(
    writer: &mut LedgerWriter,
    receipt: &CommandReceiptV1,
) -> Result<(), CommandLedgerError> {
    writer.u16(receipt.schema_version);
    writer.u64(receipt.finalization_ordinal);
    match &receipt.subject {
        CommandReceiptSubjectV1::Command {
            stream_id,
            issuer,
            sequence,
            command_id,
            body_hash,
            canonical_body_ref,
        } => {
            writer.u8(1);
            writer.bytes(stream_id.as_bytes());
            writer.principal(issuer)?;
            writer.u64(*sequence);
            writer.bytes(command_id.as_bytes());
            writer.bytes(body_hash.as_bytes());
            writer.bytes(canonical_body_ref.as_bytes());
        }
        CommandReceiptSubjectV1::CollisionSet {
            stream_id,
            issuer,
            sequence,
            candidates_root,
            candidate_count,
            candidates,
        } => {
            writer.u8(2);
            writer.bytes(stream_id.as_bytes());
            writer.principal(issuer)?;
            writer.u64(*sequence);
            writer.bytes(candidates_root.as_bytes());
            writer.u32(*candidate_count);
            writer.count(candidates.len())?;
            for candidate in candidates {
                encode_candidate(writer, candidate);
            }
        }
    }
    writer.u8(receipt.phase as u8);
    writer.u64(receipt.target_tick);
    writer.u64(receipt.finalized_at_tick);
    writer.u16(receipt.priority_class);
    writer.bytes(receipt.command_kind_registry_hash.as_bytes());
    match &receipt.result {
        CommandFinalResultV1::Committed => writer.u8(1),
        CommandFinalResultV1::Rejected { code } => {
            writer.u8(2);
            writer.text(code.as_str())?;
        }
        CommandFinalResultV1::Collision { code } => {
            writer.u8(3);
            writer.text(code.as_str())?;
        }
    }
    writer.option_hash(receipt.diagnostic_digest);
    writer.count(receipt.event_ids.len())?;
    for event_id in &receipt.event_ids {
        writer.bytes(event_id.as_bytes());
    }
    writer.bytes(receipt.transaction_result_root.as_bytes());
    Ok(())
}

fn decode_receipt(reader: &mut LedgerReader<'_>) -> Result<CommandReceiptV1, CommandLedgerError> {
    let schema_version = reader.u16()?;
    if schema_version != COMMAND_RECEIPT_SCHEMA_VERSION {
        return Err(CommandLedgerError::UnsupportedReceiptVersion(
            schema_version,
        ));
    }
    let finalization_ordinal = reader.u64()?;
    let subject = match reader.u8()? {
        1 => CommandReceiptSubjectV1::Command {
            stream_id: CommandStreamId::from_bytes(reader.array()?),
            issuer: reader.principal()?,
            sequence: reader.u64()?,
            command_id: CommandId::from_bytes(reader.array()?),
            body_hash: CommandBodyHash::from_bytes(reader.array()?),
            canonical_body_ref: CommandBodyHash::from_bytes(reader.array()?),
        },
        2 => {
            let stream_id = CommandStreamId::from_bytes(reader.array()?);
            let issuer = reader.principal()?;
            let sequence = reader.u64()?;
            let candidates_root = ContentHash::from_bytes(reader.array()?);
            let candidate_count = reader.u32()?;
            let count = reader.count()?;
            let mut candidates = Vec::with_capacity(count);
            for _ in 0..count {
                candidates.push(decode_candidate(reader)?);
            }
            CommandReceiptSubjectV1::CollisionSet {
                stream_id,
                issuer,
                sequence,
                candidates_root,
                candidate_count,
                candidates,
            }
        }
        tag => return Err(CommandLedgerError::UnknownTag(tag)),
    };
    let phase = decode_phase(reader.u8()?)?;
    let target_tick = reader.u64()?;
    let finalized_at_tick = reader.u64()?;
    let priority_class = reader.u16()?;
    let command_kind_registry_hash = ContentHash::from_bytes(reader.array()?);
    let result = match reader.u8()? {
        1 => CommandFinalResultV1::Committed,
        2 => CommandFinalResultV1::Rejected {
            code: SchemaId::new(reader.text()?)?,
        },
        3 => CommandFinalResultV1::Collision {
            code: SchemaId::new(reader.text()?)?,
        },
        tag => return Err(CommandLedgerError::UnknownTag(tag)),
    };
    let diagnostic_digest = reader.option_hash()?;
    let event_count = reader.count()?;
    let mut event_ids = Vec::with_capacity(event_count);
    for _ in 0..event_count {
        event_ids.push(EventId::from_bytes(reader.array()?));
    }
    let receipt = CommandReceiptV1 {
        schema_version,
        finalization_ordinal,
        subject,
        phase,
        target_tick,
        finalized_at_tick,
        priority_class,
        command_kind_registry_hash,
        result,
        diagnostic_digest,
        event_ids,
        transaction_result_root: ContentHash::from_bytes(reader.array()?),
    };
    receipt.validate()?;
    Ok(receipt)
}

pub(super) fn encode_archive_manifest(manifest: CommandBodyArchiveManifestV1) -> Vec<u8> {
    let mut writer = LedgerWriter::default();
    writer.u16(manifest.schema_version);
    writer.u64(manifest.entry_count);
    writer.bytes(manifest.archive_root.as_bytes());
    writer.finish()
}

pub(super) fn decode_archive_manifest(
    bytes: &[u8],
) -> Result<CommandBodyArchiveManifestV1, CommandLedgerError> {
    let mut reader = LedgerReader::new(bytes, CanonicalDecodeLimits::default());
    let manifest = CommandBodyArchiveManifestV1 {
        schema_version: reader.u16()?,
        entry_count: reader.u64()?,
        archive_root: ContentHash::from_bytes(reader.array()?),
    };
    reader.finish()?;
    if manifest.schema_version != COMMAND_BODY_ARCHIVE_SCHEMA_VERSION {
        return Err(CommandLedgerError::UnsupportedArchiveVersion(
            manifest.schema_version,
        ));
    }
    Ok(manifest)
}

pub(super) fn encode_identity_index(
    index: &CommandIdentityIndexV1,
) -> Result<Vec<u8>, CommandLedgerError> {
    let mut writer = LedgerWriter::default();
    writer.u16(index.schema_version);
    writer.u16(index.body.schema_version);
    writer.u64(index.body.command_id_count);
    writer.u64(index.body.occurrence_count);
    writer.count(index.body.bindings.len())?;
    for (command_id, binding) in &index.body.bindings {
        writer.bytes(command_id.as_bytes());
        writer.bytes(binding.command_id.as_bytes());
        writer.u8(binding.state as u8);
        writer.count(binding.occurrences.len())?;
        for occurrence in &binding.occurrences {
            writer.bytes(occurrence.body_hash.as_bytes());
            writer.bytes(occurrence.first_stream_id.as_bytes());
            writer.u64(occurrence.first_sequence);
        }
    }
    writer.bytes(index.index_root.as_bytes());
    Ok(writer.finish())
}

pub(super) fn decode_identity_index(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<CommandIdentityIndexV1, CommandLedgerError> {
    let mut reader = LedgerReader::new(bytes, limits);
    let schema_version = reader.u16()?;
    let body_schema_version = reader.u16()?;
    let command_id_count = reader.u64()?;
    let occurrence_count = reader.u64()?;
    let count = reader.count()?;
    let mut bindings = BTreeMap::new();
    let mut previous = None;
    for _ in 0..count {
        let key = CommandId::from_bytes(reader.array()?);
        if previous.is_some_and(|prior| prior >= key) {
            return Err(CommandLedgerError::MapNotStrictlySorted);
        }
        let command_id = CommandId::from_bytes(reader.array()?);
        let state = match reader.u8()? {
            0 => CommandIdentityBindingState::Unique,
            1 => CommandIdentityBindingState::Collision,
            tag => return Err(CommandLedgerError::UnknownTag(tag)),
        };
        let occurrence_len = reader.count()?;
        let mut occurrences = Vec::with_capacity(occurrence_len);
        for _ in 0..occurrence_len {
            occurrences.push(CommandIdentityOccurrenceV1 {
                body_hash: CommandBodyHash::from_bytes(reader.array()?),
                first_stream_id: CommandStreamId::from_bytes(reader.array()?),
                first_sequence: reader.u64()?,
            });
        }
        bindings.insert(
            key,
            CommandIdentityBindingV1 {
                command_id,
                occurrences,
                state,
            },
        );
        previous = Some(key);
    }
    let index = CommandIdentityIndexV1 {
        schema_version,
        body: CommandIdentityIndexBodyV1 {
            schema_version: body_schema_version,
            bindings,
            command_id_count,
            occurrence_count,
        },
        index_root: ContentHash::from_bytes(reader.array()?),
    };
    reader.finish()?;
    index.validate()?;
    Ok(index)
}

pub(super) fn encode_causal_registry(
    registry: &CausalIdentityRegistryV1,
) -> Result<Vec<u8>, CommandLedgerError> {
    let mut writer = LedgerWriter::default();
    writer.u16(registry.schema_version);
    writer.bytes(registry.world_namespace.as_bytes());
    writer.count(registry.bindings.len())?;
    for (key, hash) in &registry.bindings {
        writer.u8(key.identity_kind as u8);
        writer.bytes(&key.identity_bytes);
        writer.bytes(hash.as_bytes());
    }
    Ok(writer.finish())
}

pub(super) fn decode_causal_registry(
    bytes: &[u8],
    expected_world: WorldNamespaceId,
    limits: CanonicalDecodeLimits,
) -> Result<CausalIdentityRegistryV1, CommandLedgerError> {
    let mut reader = LedgerReader::new(bytes, limits);
    let schema_version = reader.u16()?;
    let world_namespace = WorldNamespaceId::from_bytes(reader.array()?);
    let count = reader.count()?;
    let mut bindings = BTreeMap::new();
    let mut previous = None;
    for _ in 0..count {
        let identity_kind = match reader.u8()? {
            1 => CausalIdentityKind::PlayerPrincipal,
            2 => CausalIdentityKind::CommandStream,
            3 => CausalIdentityKind::DomainEvent,
            4 => CausalIdentityKind::PersistentRecord,
            5 => CausalIdentityKind::RngStream,
            tag => return Err(CommandLedgerError::UnknownTag(tag)),
        };
        let key = CausalIdentityKey {
            identity_kind,
            identity_bytes: reader.array()?,
        };
        if previous.is_some_and(|prior| prior >= key) {
            return Err(CommandLedgerError::MapNotStrictlySorted);
        }
        bindings.insert(key, ContentHash::from_bytes(reader.array()?));
        previous = Some(key);
    }
    reader.finish()?;
    if schema_version != CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION
        || world_namespace != expected_world
    {
        return Err(CommandLedgerError::CausalIdentityRegistryMismatch);
    }
    Ok(CausalIdentityRegistryV1 {
        schema_version,
        world_namespace,
        bindings,
    })
}

fn decode_phase(tag: u8) -> Result<CommandPhase, CommandLedgerError> {
    match tag {
        0 => Ok(CommandPhase::Ingress),
        1 => Ok(CommandPhase::Outcome),
        _ => Err(CommandLedgerError::UnknownTag(tag)),
    }
}

pub(super) fn extend_u32_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), CanonicalError> {
    output.extend_from_slice(
        &u32::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    output.extend_from_slice(bytes);
    Ok(())
}

pub(super) fn require_ledger_fields(
    segment: &crate::canonical::DecodedCanonicalSegment,
    expected: &[(u32, u8)],
) -> Result<(), CommandLedgerError> {
    for actual in &segment.fields {
        if !expected.iter().any(|(id, _)| *id == actual.field_id) {
            return Err(CommandLedgerError::UnknownField(actual.field_id));
        }
    }
    for (id, tag) in expected {
        let actual = ledger_field(segment, *id)?;
        if actual.type_tag != *tag {
            return Err(CommandLedgerError::WrongFieldType {
                field_id: *id,
                expected: *tag,
                actual: actual.type_tag,
            });
        }
    }
    Ok(())
}

pub(super) fn ledger_field(
    segment: &crate::canonical::DecodedCanonicalSegment,
    id: u32,
) -> Result<&CanonicalField, CommandLedgerError> {
    segment
        .field(id)
        .ok_or(CommandLedgerError::MissingField(id))
}

pub(super) fn fixed_field<const N: usize>(
    segment: &crate::canonical::DecodedCanonicalSegment,
    id: u32,
) -> Result<[u8; N], CommandLedgerError> {
    ledger_field(segment, id)?
        .payload
        .as_slice()
        .try_into()
        .map_err(|_| CommandLedgerError::InvalidFieldLength {
            field_id: id,
            expected: N,
            actual: ledger_field(segment, id).map_or(0, |field| field.payload.len()),
        })
}
