use super::*;
use sha2::{Digest, Sha256};

pub fn causal_provenance_hash(
    identity_kind: CausalIdentityKind,
    canonical_provenance_bytes: &[u8],
) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.causal-provenance.v1\0");
    preimage.push(identity_kind as u8);
    preimage.extend_from_slice(
        &u64::try_from(canonical_provenance_bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(canonical_provenance_bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}
pub(super) fn collision_candidates_are_canonical(
    candidates: &[CommandCollisionCandidateV1],
) -> bool {
    !candidates.is_empty()
        && candidates
            .iter()
            .all(|candidate| candidate.body_hash == candidate.canonical_body_ref)
        && candidates.windows(2).all(|pair| {
            pair[0].body_hash != pair[1].body_hash
                && (&pair[0].body_hash, &pair[0].command_id)
                    < (&pair[1].body_hash, &pair[1].command_id)
        })
}

pub fn command_body_archive_root(
    entries: &BTreeMap<CommandBodyHash, Arc<[u8]>>,
) -> Result<ContentHash, CanonicalError> {
    let mut nodes = Vec::with_capacity(entries.len());
    for (body_hash, body_bytes) in entries {
        nodes.push(command_body_archive_leaf_hash(
            *body_hash,
            body_bytes.as_ref(),
        )?);
    }
    command_body_archive_root_from_leaves(
        nodes,
        u64::try_from(entries.len()).map_err(|_| CanonicalError::LengthOverflow)?,
    )
}

pub(super) fn command_body_archive_leaf_hash(
    body_hash: CommandBodyHash,
    body_bytes: &[u8],
) -> Result<[u8; 32], CanonicalError> {
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.command-body-archive-leaf.v1\0");
    hasher.update(body_hash.as_bytes());
    hasher.update(
        u64::try_from(body_bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    hasher.update(body_bytes);
    Ok(hasher.finalize().into())
}

pub(super) fn command_body_archive_root_from_leaves(
    leaves: impl IntoIterator<Item = [u8; 32]>,
    entry_count: u64,
) -> Result<ContentHash, CanonicalError> {
    let mut nodes: Vec<_> = leaves.into_iter().collect();
    let merkle_root = if nodes.is_empty() {
        sha256(b"nextengine.command-body-archive-empty.v1\0")
    } else {
        while nodes.len() > 1 {
            let mut parents = Vec::with_capacity(nodes.len().div_ceil(2));
            for pair in nodes.chunks(2) {
                let mut hasher = Sha256::new();
                if let [left, right] = pair {
                    hasher.update(b"nextengine.command-body-archive-node.v1\0");
                    hasher.update(left);
                    hasher.update(right);
                } else {
                    hasher.update(b"nextengine.command-body-archive-carry.v1\0");
                    hasher.update(pair[0]);
                }
                parents.push(hasher.finalize().into());
            }
            nodes = parents;
        }
        nodes[0]
    };
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.command-body-archive-root.v1\0");
    hasher.update(entry_count.to_le_bytes());
    hasher.update(merkle_root);
    Ok(content_hash_from_bytes(hasher.finalize().into()))
}

pub fn command_identity_index_root(
    body: &CommandIdentityIndexBodyV1,
) -> Result<ContentHash, CanonicalError> {
    let layout = body.canonical_layout()?;
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.command-identity-index.v1\0");
    hasher.update(
        u64::try_from(layout.total_bytes)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    body.visit_canonical_bytes(layout, &mut |chunk| hasher.update(chunk))?;
    Ok(content_hash_from_bytes(hasher.finalize().into()))
}

#[must_use]
pub fn command_receipt_chain_genesis() -> ContentHash {
    content_hash_from_bytes(sha256(b"nextengine.command-ledger-chain.genesis.v1\0"))
}

pub fn command_receipt_digest(receipt: &CommandReceiptV1) -> Result<ContentHash, CanonicalError> {
    let bytes = receipt.canonical_bytes()?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-receipt.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(&bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

pub fn command_receipt_chain_next(
    previous_root: ContentHash,
    receipt: &CommandReceiptV1,
) -> Result<ContentHash, CanonicalError> {
    let receipt_digest = command_receipt_digest(receipt)?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-ledger-chain.v1\0");
    preimage.extend_from_slice(previous_root.as_bytes());
    preimage.extend_from_slice(receipt_digest.as_bytes());
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

pub fn command_collision_candidates_root(
    candidates: &[CommandCollisionCandidateV1],
) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-collision-set.v1\0");
    preimage.extend_from_slice(
        &u32::try_from(candidates.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for candidate in candidates {
        preimage.extend_from_slice(candidate.body_hash.as_bytes());
        preimage.extend_from_slice(candidate.command_id.as_bytes());
    }
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

#[must_use]
pub fn command_collision_incident_digest(
    stream_id: CommandStreamId,
    sequence: u64,
    candidates_root: ContentHash,
) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-collision-incident.v1\0");
    preimage.extend_from_slice(stream_id.as_bytes());
    preimage.extend_from_slice(&sequence.to_le_bytes());
    preimage.extend_from_slice(candidates_root.as_bytes());
    content_hash_from_bytes(sha256(&preimage))
}

pub(super) fn validate_body_reference(
    archive_hashes: &BTreeSet<CommandBodyHash>,
    command_id: CommandId,
    body_hash: CommandBodyHash,
    index: &CommandIdentityIndexV1,
) -> Result<(), CommandLedgerError> {
    if !archive_hashes.contains(&body_hash) {
        return Err(CommandLedgerError::BodyReferenceMissing);
    }
    let Some(binding) = index.body.bindings.get(&command_id) else {
        return Err(CommandLedgerError::IdentityReferenceMissing);
    };
    if !binding
        .occurrences
        .iter()
        .any(|occurrence| occurrence.body_hash == body_hash)
    {
        return Err(CommandLedgerError::IdentityReferenceMissing);
    }
    Ok(())
}

pub(super) fn validate_receipt_references(
    receipt: &CommandReceiptV1,
    archive_hashes: &BTreeSet<CommandBodyHash>,
    index: &CommandIdentityIndexV1,
) -> Result<(), CommandLedgerError> {
    match &receipt.subject {
        CommandReceiptSubjectV1::Command {
            command_id,
            body_hash,
            canonical_body_ref,
            ..
        } => {
            if body_hash != canonical_body_ref {
                return Err(CommandLedgerError::BodyReferenceMissing);
            }
            validate_body_reference(archive_hashes, *command_id, *body_hash, index)
        }
        CommandReceiptSubjectV1::CollisionSet { candidates, .. } => {
            for candidate in candidates {
                if candidate.body_hash != candidate.canonical_body_ref {
                    return Err(CommandLedgerError::BodyReferenceMissing);
                }
                validate_body_reference(
                    archive_hashes,
                    candidate.command_id,
                    candidate.body_hash,
                    index,
                )?;
            }
            Ok(())
        }
    }
}

pub(super) fn validate_receipt_against_reservation(
    receipt: &CommandReceiptV1,
    reservation: &CommandReservationV1,
) -> Result<(), CommandLedgerError> {
    let CommandReceiptSubjectV1::Command {
        stream_id,
        issuer,
        sequence,
        command_id,
        body_hash,
        canonical_body_ref,
    } = &receipt.subject
    else {
        return Err(CommandLedgerError::ReservationMismatch);
    };
    if *stream_id != reservation.stream_id
        || issuer != &reservation.issuer
        || *sequence != reservation.sequence
        || *command_id != reservation.command_id
        || *body_hash != reservation.body_hash
        || *canonical_body_ref != reservation.canonical_body_ref
        || receipt.phase != reservation.phase
        || receipt.target_tick != reservation.target_tick
        || receipt.priority_class != reservation.priority_class
        || receipt.command_kind_registry_hash != reservation.command_kind_registry_hash
    {
        return Err(CommandLedgerError::ReservationMismatch);
    }
    Ok(())
}

pub(super) fn receipt_matches_collision_incident(
    receipt: &CommandReceiptV1,
    incident: Option<&CommandCollisionIncidentV1>,
) -> bool {
    let (
        CommandReceiptSubjectV1::CollisionSet {
            stream_id,
            issuer,
            sequence,
            candidates_root,
            candidates,
            ..
        },
        Some(incident),
    ) = (&receipt.subject, incident)
    else {
        return false;
    };
    *stream_id == incident.stream_id
        && issuer == &incident.issuer
        && *sequence == incident.sequence
        && *candidates_root == incident.candidates_root
        && candidates == &incident.candidates
}

pub(super) fn nested_value(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.push(type_tag);
    bytes.extend_from_slice(
        &u64::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

pub(super) fn struct_record(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    if let Some(pair) = fields
        .windows(2)
        .find(|pair| pair[0].field_id == pair[1].field_id)
    {
        return Err(CanonicalError::DuplicateField(pair[0].field_id));
    }
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for field in fields {
        payload.extend_from_slice(&field.field_id.to_le_bytes());
        payload.push(field.type_tag);
        payload.extend_from_slice(
            &u64::try_from(field.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        payload.extend_from_slice(&field.payload);
    }
    nested_value(CANONICAL_TYPE_STRUCT, &payload)
}

pub(super) fn encode_sequence(records: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(records.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for record in records {
        payload.extend_from_slice(&record);
    }
    Ok(payload)
}

#[cfg(test)]
pub(super) fn encode_map(mut entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<Vec<u8>, CanonicalError> {
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(entries.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for (key, value) in entries {
        payload.extend_from_slice(&key);
        payload.extend_from_slice(&value);
    }
    Ok(payload)
}

pub(super) fn option_hash(value: Option<&ContentHash>) -> Result<Vec<u8>, CanonicalError> {
    let Some(value) = value else {
        return Ok(vec![0]);
    };
    let mut payload = vec![1];
    payload.extend_from_slice(&nested_value(CANONICAL_TYPE_HASH256, value.as_bytes())?);
    Ok(payload)
}

pub(super) fn principal_payload(principal: &IssuerPrincipal) -> Result<Vec<u8>, CanonicalError> {
    let encoded = principal.canonical_bytes()?;
    let header_length = 1 + std::mem::size_of::<u64>();
    encoded
        .get(header_length..)
        .map(ToOwned::to_owned)
        .ok_or(CanonicalError::LengthOverflow)
}

// Canonical encoding visitors and merged-view root helpers for the
// command identity index. The identity index body methods and the
// prepared-update root path share these through `use super::hashes::*`.
pub(super) fn encode_identity_bindings(
    bindings: &BTreeMap<CommandId, CommandIdentityBindingV1>,
) -> Result<Vec<u8>, CanonicalError> {
    let expected_bytes = identity_bindings_byte_len(bindings)?;
    let mut bytes = Vec::with_capacity(expected_bytes);
    visit_identity_bindings(&mut |chunk| bytes.extend_from_slice(chunk), bindings)?;
    debug_assert_eq!(bytes.len(), expected_bytes);
    Ok(bytes)
}

pub(super) fn identity_bindings_byte_len(
    bindings: &BTreeMap<CommandId, CommandIdentityBindingV1>,
) -> Result<usize, CanonicalError> {
    u32::try_from(bindings.len()).map_err(|_| CanonicalError::LengthOverflow)?;
    bindings
        .values()
        .try_fold(std::mem::size_of::<u32>(), |length, binding| {
            let binding_bytes = identity_binding_byte_len(binding)?;
            length
                .checked_add(CANONICAL_NESTED_HEADER_BYTES + std::mem::size_of::<u128>())
                .and_then(|length| length.checked_add(binding_bytes))
                .ok_or(CanonicalError::LengthOverflow)
        })
}

pub(super) fn visit_identity_bindings(
    write: &mut impl FnMut(&[u8]),
    bindings: &BTreeMap<CommandId, CommandIdentityBindingV1>,
) -> Result<(), CanonicalError> {
    write(
        &u32::try_from(bindings.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for (command_id, binding) in bindings {
        visit_nested_value(write, CANONICAL_TYPE_ID128, command_id.as_bytes())?;
        visit_identity_binding(write, binding)?;
    }
    Ok(())
}

pub(super) fn identity_binding_byte_len(
    binding: &CommandIdentityBindingV1,
) -> Result<usize, CanonicalError> {
    let occurrences_bytes = std::mem::size_of::<u32>()
        .checked_add(
            binding
                .occurrences
                .len()
                .checked_mul(IDENTITY_OCCURRENCE_RECORD_BYTES)
                .ok_or(CanonicalError::LengthOverflow)?,
        )
        .ok_or(CanonicalError::LengthOverflow)?;
    let payload_bytes = std::mem::size_of::<u32>()
        .checked_add(canonical_field_bytes(binding.command_id.as_bytes().len())?)
        .and_then(|length| length.checked_add(canonical_field_bytes(occurrences_bytes).ok()?))
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u8>()).ok()?)
        })
        .ok_or(CanonicalError::LengthOverflow)?;
    CANONICAL_NESTED_HEADER_BYTES
        .checked_add(payload_bytes)
        .ok_or(CanonicalError::LengthOverflow)
}

pub(super) fn visit_identity_binding(
    write: &mut impl FnMut(&[u8]),
    binding: &CommandIdentityBindingV1,
) -> Result<(), CanonicalError> {
    let binding_bytes = identity_binding_byte_len(binding)?;
    let payload_bytes = binding_bytes
        .checked_sub(CANONICAL_NESTED_HEADER_BYTES)
        .ok_or(CanonicalError::LengthOverflow)?;
    let occurrences_bytes = std::mem::size_of::<u32>()
        .checked_add(
            binding
                .occurrences
                .len()
                .checked_mul(IDENTITY_OCCURRENCE_RECORD_BYTES)
                .ok_or(CanonicalError::LengthOverflow)?,
        )
        .ok_or(CanonicalError::LengthOverflow)?;
    visit_nested_header(write, CANONICAL_TYPE_STRUCT, payload_bytes)?;
    write(&3_u32.to_le_bytes());
    visit_field(
        write,
        1,
        CANONICAL_TYPE_ID128,
        binding.command_id.as_bytes(),
    )?;
    visit_field_header(write, 2, CANONICAL_TYPE_SEQUENCE, occurrences_bytes)?;
    write(
        &u32::try_from(binding.occurrences.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for occurrence in &binding.occurrences {
        visit_identity_occurrence(write, occurrence)?;
    }
    visit_field(write, 3, CANONICAL_TYPE_U8, &[binding.state as u8])
}

pub(super) fn visit_identity_occurrence(
    write: &mut impl FnMut(&[u8]),
    occurrence: &CommandIdentityOccurrenceV1,
) -> Result<(), CanonicalError> {
    let payload_bytes = std::mem::size_of::<u32>()
        .checked_add(canonical_field_bytes(
            occurrence.body_hash.as_bytes().len(),
        )?)
        .and_then(|length| {
            length.checked_add(
                canonical_field_bytes(occurrence.first_stream_id.as_bytes().len()).ok()?,
            )
        })
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u64>()).ok()?)
        })
        .ok_or(CanonicalError::LengthOverflow)?;
    visit_nested_header(write, CANONICAL_TYPE_STRUCT, payload_bytes)?;
    write(&3_u32.to_le_bytes());
    visit_field(
        write,
        1,
        CANONICAL_TYPE_HASH256,
        occurrence.body_hash.as_bytes(),
    )?;
    visit_field(
        write,
        2,
        CANONICAL_TYPE_ID128,
        occurrence.first_stream_id.as_bytes(),
    )?;
    visit_field(
        write,
        3,
        CANONICAL_TYPE_U64,
        &occurrence.first_sequence.to_le_bytes(),
    )
}

pub(super) const CANONICAL_NESTED_HEADER_BYTES: usize =
    std::mem::size_of::<u8>() + std::mem::size_of::<u64>();
pub(super) const CANONICAL_FIELD_HEADER_BYTES: usize =
    std::mem::size_of::<u32>() + std::mem::size_of::<u8>() + std::mem::size_of::<u64>();
pub(super) const IDENTITY_OCCURRENCE_RECORD_BYTES: usize = CANONICAL_NESTED_HEADER_BYTES
    + std::mem::size_of::<u32>()
    + (3 * CANONICAL_FIELD_HEADER_BYTES)
    + 32
    + 16
    + 8;

pub(super) fn canonical_field_bytes(payload_bytes: usize) -> Result<usize, CanonicalError> {
    std::mem::size_of::<u32>()
        .checked_add(std::mem::size_of::<u8>())
        .and_then(|length| length.checked_add(std::mem::size_of::<u64>()))
        .and_then(|length| length.checked_add(payload_bytes))
        .ok_or(CanonicalError::LengthOverflow)
}

pub(super) fn visit_u32_length_prefixed(
    write: &mut impl FnMut(&[u8]),
    payload: &[u8],
) -> Result<(), CanonicalError> {
    write(
        &u32::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    write(payload);
    Ok(())
}

pub(super) fn visit_nested_value(
    write: &mut impl FnMut(&[u8]),
    type_tag: u8,
    payload: &[u8],
) -> Result<(), CanonicalError> {
    visit_nested_header(write, type_tag, payload.len())?;
    write(payload);
    Ok(())
}

pub(super) fn visit_nested_header(
    write: &mut impl FnMut(&[u8]),
    type_tag: u8,
    payload_bytes: usize,
) -> Result<(), CanonicalError> {
    write(&[type_tag]);
    write(
        &u64::try_from(payload_bytes)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

pub(super) fn visit_field(
    write: &mut impl FnMut(&[u8]),
    field_id: u32,
    type_tag: u8,
    payload: &[u8],
) -> Result<(), CanonicalError> {
    visit_field_header(write, field_id, type_tag, payload.len())?;
    write(payload);
    Ok(())
}

pub(super) fn visit_field_header(
    write: &mut impl FnMut(&[u8]),
    field_id: u32,
    type_tag: u8,
    payload_bytes: usize,
) -> Result<(), CanonicalError> {
    write(&field_id.to_le_bytes());
    write(&[type_tag]);
    write(
        &u64::try_from(payload_bytes)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

pub(super) fn command_identity_index_root_with_replacements(
    schema_version: u16,
    base: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    replacements: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    command_id_count: u64,
    occurrence_count: u64,
) -> Result<ContentHash, CanonicalError> {
    let bindings_bytes = merged_identity_bindings_byte_len(base, replacements)?;
    let total_bytes = crate::canonical::CANONICAL_BINARY_V1_MAGIC
        .len()
        .checked_add(std::mem::size_of::<u16>())
        .and_then(|length| {
            length.checked_add(
                std::mem::size_of::<u32>() + COMMAND_IDENTITY_INDEX_BODY_OWNER_ID.len(),
            )
        })
        .and_then(|length| {
            length.checked_add(
                std::mem::size_of::<u32>() + COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID.len(),
            )
        })
        .and_then(|length| {
            length.checked_add(
                std::mem::size_of::<u32>() + COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID.len(),
            )
        })
        .and_then(|length| length.checked_add(std::mem::size_of::<u32>()))
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u16>()).ok()?)
        })
        .and_then(|length| length.checked_add(canonical_field_bytes(bindings_bytes).ok()?))
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u64>()).ok()?)
        })
        .and_then(|length| {
            length.checked_add(canonical_field_bytes(std::mem::size_of::<u64>()).ok()?)
        })
        .ok_or(CanonicalError::LengthOverflow)?;
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.command-identity-index.v1\0");
    hasher.update(
        u64::try_from(total_bytes)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    let mut write = |chunk: &[u8]| hasher.update(chunk);
    write(&crate::canonical::CANONICAL_BINARY_V1_MAGIC);
    write(&crate::canonical::CANONICAL_BINARY_V1_VERSION.to_le_bytes());
    visit_u32_length_prefixed(&mut write, COMMAND_IDENTITY_INDEX_BODY_OWNER_ID.as_bytes())?;
    visit_u32_length_prefixed(&mut write, COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID.as_bytes())?;
    visit_u32_length_prefixed(
        &mut write,
        COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID.as_bytes(),
    )?;
    write(&4_u32.to_le_bytes());
    visit_field(
        &mut write,
        1,
        CANONICAL_TYPE_U16,
        &schema_version.to_le_bytes(),
    )?;
    visit_field_header(&mut write, 2, CANONICAL_TYPE_MAP, bindings_bytes)?;
    write(
        &u32::try_from(command_id_count)
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for_each_merged_identity_binding(base, replacements, |command_id, binding| {
        visit_nested_value(&mut write, CANONICAL_TYPE_ID128, command_id.as_bytes())?;
        visit_identity_binding(&mut write, binding)
    })?;
    visit_field(
        &mut write,
        3,
        CANONICAL_TYPE_U64,
        &command_id_count.to_le_bytes(),
    )?;
    visit_field(
        &mut write,
        4,
        CANONICAL_TYPE_U64,
        &occurrence_count.to_le_bytes(),
    )?;
    Ok(content_hash_from_bytes(hasher.finalize().into()))
}

pub(super) fn merged_identity_bindings_byte_len(
    base: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    replacements: &BTreeMap<CommandId, CommandIdentityBindingV1>,
) -> Result<usize, CanonicalError> {
    let count = base
        .len()
        .checked_add(
            replacements
                .keys()
                .filter(|command_id| !base.contains_key(command_id))
                .count(),
        )
        .ok_or(CanonicalError::LengthOverflow)?;
    u32::try_from(count).map_err(|_| CanonicalError::LengthOverflow)?;
    let mut length = std::mem::size_of::<u32>();
    for_each_merged_identity_binding(base, replacements, |_, binding| {
        length = length
            .checked_add(CANONICAL_NESTED_HEADER_BYTES + std::mem::size_of::<u128>())
            .and_then(|length| length.checked_add(identity_binding_byte_len(binding).ok()?))
            .ok_or(CanonicalError::LengthOverflow)?;
        Ok(())
    })?;
    Ok(length)
}

pub(super) fn for_each_merged_identity_binding(
    base: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    replacements: &BTreeMap<CommandId, CommandIdentityBindingV1>,
    mut visit: impl FnMut(&CommandId, &CommandIdentityBindingV1) -> Result<(), CanonicalError>,
) -> Result<(), CanonicalError> {
    let mut base = base.iter().peekable();
    let mut replacements = replacements.iter().peekable();
    loop {
        match (base.peek(), replacements.peek()) {
            (Some((base_id, base_binding)), Some((replacement_id, replacement_binding))) => {
                match base_id.cmp(replacement_id) {
                    std::cmp::Ordering::Less => {
                        visit(base_id, base_binding)?;
                        base.next();
                    }
                    std::cmp::Ordering::Equal => {
                        visit(replacement_id, replacement_binding)?;
                        base.next();
                        replacements.next();
                    }
                    std::cmp::Ordering::Greater => {
                        visit(replacement_id, replacement_binding)?;
                        replacements.next();
                    }
                }
            }
            (Some((command_id, binding)), None) => {
                visit(command_id, binding)?;
                base.next();
            }
            (None, Some((command_id, binding))) => {
                visit(command_id, binding)?;
                replacements.next();
            }
            (None, None) => return Ok(()),
        }
    }
}
