use super::codec::*;
use super::hashes::*;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandLedgerV2 {
    pub schema_version: u16,
    pub world_namespace: WorldNamespaceId,
    pub streams: BTreeMap<CommandStreamId, CommandStreamLedgerV2>,
    pub command_kind_registry_hash: ContentHash,
    pub runtime_determinism_profile_hash: ContentHash,
    pub body_archive: CommandBodyArchiveManifestV1,
    pub identity_index: CommandIdentityIndexV1,
    pub causal_identity_registry: CausalIdentityRegistryV1,
}

impl CommandLedgerV2 {
    pub fn empty(
        world_namespace: WorldNamespaceId,
        command_kind_registry_hash: ContentHash,
        runtime_determinism_profile_hash: ContentHash,
    ) -> Result<(Self, CommandBodyArchiveV1), CommandLedgerError> {
        let archive = CommandBodyArchiveV1::default();
        let ledger = Self {
            schema_version: COMMAND_LEDGER_SCHEMA_VERSION,
            world_namespace,
            streams: BTreeMap::new(),
            command_kind_registry_hash,
            runtime_determinism_profile_hash,
            body_archive: archive.manifest()?,
            identity_index: CommandIdentityIndexV1::empty()?,
            causal_identity_registry: CausalIdentityRegistryV1 {
                schema_version: CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION,
                world_namespace,
                bindings: Arc::new(BTreeMap::new()),
            },
        };
        ledger.validate(&archive)?;
        Ok((ledger, archive))
    }

    pub fn synchronize_archive(
        &mut self,
        archive: &CommandBodyArchiveV1,
    ) -> Result<(), CommandLedgerError> {
        archive.validate()?;
        let mut next = self.clone();
        next.body_archive = archive.manifest()?;
        next.validate(archive)?;
        *self = next;
        Ok(())
    }

    /// Publishes exact archive metadata for a transaction assembled only
    /// through the incremental archive/identity/stream mutation APIs.
    ///
    /// Full historical closure validation remains mandatory when decoding,
    /// serializing or restoring a durable public snapshot.
    pub fn synchronize_archive_incremental(
        &mut self,
        archive: &CommandBodyArchiveV1,
    ) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedLedgerVersion(
                self.schema_version,
            ));
        }
        if self.identity_index.schema_version != COMMAND_IDENTITY_INDEX_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedIdentityIndexVersion(
                self.identity_index.schema_version,
            ));
        }
        if self.identity_index.body.schema_version != COMMAND_IDENTITY_INDEX_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedIdentityIndexVersion(
                self.identity_index.body.schema_version,
            ));
        }
        let entry_count = u64::try_from(archive.entries().len())
            .map_err(|_| CommandLedgerError::CountOverflow)?;
        if entry_count != self.identity_index.body.occurrence_count {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
        }
        self.body_archive = archive.manifest()?;
        Ok(())
    }

    pub fn validate(&self, archive: &CommandBodyArchiveV1) -> Result<(), CommandLedgerError> {
        if self.schema_version != COMMAND_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedLedgerVersion(
                self.schema_version,
            ));
        }
        if self.causal_identity_registry.schema_version != CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION
            || self.causal_identity_registry.world_namespace != self.world_namespace
        {
            return Err(CommandLedgerError::CausalIdentityRegistryMismatch);
        }
        archive.validate()?;
        if archive.manifest()? != self.body_archive {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
        }
        self.identity_index.validate()?;
        let archive_hashes: BTreeSet<_> = archive.entries().keys().copied().collect();
        let occurrence_hashes: BTreeSet<_> = self
            .identity_index
            .body
            .bindings
            .values()
            .flat_map(|binding| {
                binding
                    .occurrences
                    .iter()
                    .map(|occurrence| occurrence.body_hash)
            })
            .collect();
        if archive_hashes != occurrence_hashes {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
        }
        for (command_id, binding) in self.identity_index.body.bindings.iter() {
            for occurrence in &binding.occurrences {
                let body_bytes = archive
                    .entries()
                    .get(&occurrence.body_hash)
                    .ok_or(CommandLedgerError::BodyReferenceMissing)?;
                if compute_command_id_from_body_bytes(body_bytes)? != *command_id {
                    return Err(CommandLedgerError::IdentityCommandIdMismatch);
                }
            }
        }
        for (stream_id, stream) in &self.streams {
            if stream_id != &stream.stream_id {
                return Err(CommandLedgerError::StreamKeyMismatch);
            }
            stream.validate()?;
            for reservation in stream.pending.values() {
                validate_body_reference(
                    &archive_hashes,
                    reservation.command_id,
                    reservation.body_hash,
                    &self.identity_index,
                )?;
            }
            for receipt in stream.receipt_window.iter() {
                validate_receipt_references(receipt, &archive_hashes, &self.identity_index)?;
            }
            if let Some(incident) = &stream.collision_incident {
                for candidate in &incident.candidates {
                    validate_body_reference(
                        &archive_hashes,
                        candidate.command_id,
                        candidate.body_hash,
                        &self.identity_index,
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(
        &self,
        archive: &CommandBodyArchiveV1,
    ) -> Result<Vec<u8>, CommandLedgerError> {
        self.validate(archive)?;
        self.canonical_bytes_validated()
    }

    pub(crate) fn canonical_bytes_validated(&self) -> Result<Vec<u8>, CommandLedgerError> {
        encode_canonical_segment(
            COMMAND_LEDGER_OWNER_ID,
            COMMAND_LEDGER_SCHEMA_ID,
            COMMAND_LEDGER_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_ID128,
                    self.world_namespace.as_bytes().to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_streams(&self.streams)?),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_HASH256,
                    self.command_kind_registry_hash.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_HASH256,
                    self.runtime_determinism_profile_hash.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_STRUCT,
                    encode_archive_manifest(self.body_archive),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_BYTES,
                    encode_identity_index(&self.identity_index)?,
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_MAP,
                    encode_causal_registry(&self.causal_identity_registry)?,
                ),
            ],
        )
        .map_err(Into::into)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        archive: &CommandBodyArchiveV1,
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CommandLedgerError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != COMMAND_LEDGER_OWNER_ID
            || segment.schema_id != COMMAND_LEDGER_SCHEMA_ID
            || segment.segment_id != COMMAND_LEDGER_SEGMENT_ID
        {
            return Err(CommandLedgerError::WrongEnvelope);
        }
        require_ledger_fields(
            &segment,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_MAP),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_HASH256),
                (6, CANONICAL_TYPE_STRUCT),
                (7, CANONICAL_TYPE_BYTES),
                (8, CANONICAL_TYPE_MAP),
            ],
        )?;
        let schema_version = u16::from_le_bytes(fixed_field(&segment, 1)?);
        if schema_version != COMMAND_LEDGER_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedLedgerVersion(schema_version));
        }
        let world_namespace = WorldNamespaceId::from_bytes(fixed_field(&segment, 2)?);
        let ledger = Self {
            schema_version,
            world_namespace,
            streams: decode_streams(&ledger_field(&segment, 3)?.payload, limits)?,
            command_kind_registry_hash: ContentHash::from_bytes(fixed_field(&segment, 4)?),
            runtime_determinism_profile_hash: ContentHash::from_bytes(fixed_field(&segment, 5)?),
            body_archive: decode_archive_manifest(&ledger_field(&segment, 6)?.payload)?,
            identity_index: decode_identity_index(&ledger_field(&segment, 7)?.payload, limits)?,
            causal_identity_registry: decode_causal_registry(
                &ledger_field(&segment, 8)?.payload,
                world_namespace,
                limits,
            )?,
        };
        ledger.validate(archive)?;
        if ledger.canonical_bytes_validated()? != bytes {
            return Err(CommandLedgerError::NonCanonicalEncoding);
        }
        Ok(ledger)
    }

    pub fn command_ledger_hash(
        &self,
        archive: &CommandBodyArchiveV1,
    ) -> Result<CommandLedgerHash, CommandLedgerError> {
        let bytes = self.canonical_bytes(archive)?;
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.command-ledger.v2\0");
        preimage.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| CommandLedgerError::CountOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&bytes);
        Ok(command_ledger_hash_from_bytes(sha256(&preimage)))
    }
}
