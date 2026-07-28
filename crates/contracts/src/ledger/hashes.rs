use super::*;

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
    entries: &BTreeMap<CommandBodyHash, Vec<u8>>,
) -> Result<ContentHash, CanonicalError> {
    let mut nodes = Vec::with_capacity(entries.len());
    for (body_hash, body_bytes) in entries {
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.command-body-archive-leaf.v1\0");
        preimage.extend_from_slice(body_hash.as_bytes());
        preimage.extend_from_slice(
            &u64::try_from(body_bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(body_bytes);
        nodes.push(sha256(&preimage));
    }
    let merkle_root = if nodes.is_empty() {
        sha256(b"nextengine.command-body-archive-empty.v1\0")
    } else {
        while nodes.len() > 1 {
            let mut parents = Vec::with_capacity(nodes.len().div_ceil(2));
            for pair in nodes.chunks(2) {
                let mut preimage = Vec::new();
                if let [left, right] = pair {
                    preimage.extend_from_slice(b"nextengine.command-body-archive-node.v1\0");
                    preimage.extend_from_slice(left);
                    preimage.extend_from_slice(right);
                } else {
                    preimage.extend_from_slice(b"nextengine.command-body-archive-carry.v1\0");
                    preimage.extend_from_slice(&pair[0]);
                }
                parents.push(sha256(&preimage));
            }
            nodes = parents;
        }
        nodes[0]
    };
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-body-archive-root.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(entries.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(&merkle_root);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

pub fn command_identity_index_root(
    body: &CommandIdentityIndexBodyV1,
) -> Result<ContentHash, CanonicalError> {
    let bytes = body.canonical_bytes()?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-identity-index.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(&bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
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
