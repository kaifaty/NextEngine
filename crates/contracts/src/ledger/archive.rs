use super::codec::{extend_u32_bytes, fixed_field, ledger_field, require_ledger_fields};
use super::hashes::*;
use super::wire::*;
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandBodyArchiveManifestV1 {
    pub schema_version: u16,
    pub entry_count: u64,
    pub archive_root: ContentHash,
}

#[derive(Clone, Debug, Default)]
pub struct CommandBodyArchiveV1 {
    entries: Arc<BTreeMap<CommandBodyHash, Arc<[u8]>>>,
    leaf_hashes: Arc<Vec<(CommandBodyHash, [u8; 32])>>,
    command_ids: Arc<BTreeMap<CommandBodyHash, CommandId>>,
    manifest: Arc<std::sync::OnceLock<CommandBodyArchiveManifestV1>>,
}

impl PartialEq for CommandBodyArchiveV1 {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
            && self.leaf_hashes == other.leaf_hashes
            && self.command_ids == other.command_ids
    }
}

impl Eq for CommandBodyArchiveV1 {}

/// Opaque, fully checked append-only update for a command-body archive.
///
/// The update keeps only the new entries. Applying it to the archive it was
/// prepared from performs no fallible validation or canonicalization work.
#[derive(Clone, Debug)]
pub struct PreparedCommandBodyArchiveUpdate {
    base_entry_count: usize,
    base_leaf_hashes: Arc<Vec<(CommandBodyHash, [u8; 32])>>,
    base_command_ids: Arc<BTreeMap<CommandBodyHash, CommandId>>,
    additions: BTreeMap<CommandBodyHash, Arc<[u8]>>,
    addition_leaf_hashes: Vec<(CommandBodyHash, [u8; 32])>,
    addition_command_ids: Vec<(CommandBodyHash, CommandId)>,
    entry_count: u64,
    manifest: std::sync::OnceLock<CommandBodyArchiveManifestV1>,
}

impl PreparedCommandBodyArchiveUpdate {
    #[must_use]
    pub fn manifest(&self) -> CommandBodyArchiveManifestV1 {
        *self.manifest.get_or_init(|| {
            let mut base = self.base_leaf_hashes.iter().peekable();
            let mut added = self.addition_leaf_hashes.iter().peekable();
            let mut merged = Vec::with_capacity(
                usize::try_from(self.entry_count)
                    .expect("validated archive entry count fits usize"),
            );
            loop {
                match (base.peek(), added.peek()) {
                    (Some((base_hash, base_leaf)), Some((added_hash, added_leaf))) => {
                        if base_hash < added_hash {
                            merged.push(*base_leaf);
                            base.next();
                        } else if added_hash < base_hash {
                            merged.push(*added_leaf);
                            added.next();
                        } else {
                            unreachable!("validated archive additions are disjoint from the base");
                        }
                    }
                    (Some((_, base_leaf)), None) => {
                        merged.push(*base_leaf);
                        base.next();
                    }
                    (None, Some((_, added_leaf))) => {
                        merged.push(*added_leaf);
                        added.next();
                    }
                    (None, None) => break,
                }
            }
            CommandBodyArchiveManifestV1 {
                schema_version: COMMAND_BODY_ARCHIVE_SCHEMA_VERSION,
                entry_count: self.entry_count,
                archive_root: command_body_archive_root_from_leaves(merged, self.entry_count)
                    .expect("validated archive leaves have a representable root"),
            }
        })
    }

    #[must_use]
    pub const fn entry_count(&self) -> u64 {
        self.entry_count
    }

    #[must_use]
    pub fn additions(&self) -> &BTreeMap<CommandBodyHash, Arc<[u8]>> {
        &self.additions
    }
}

impl CommandBodyArchiveV1 {
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<CommandBodyHash, Arc<[u8]>> {
        &self.entries
    }

    pub(crate) fn command_id_for_body_hash(
        &self,
        body_hash: &CommandBodyHash,
    ) -> Option<CommandId> {
        self.command_ids.get(body_hash).copied()
    }

    pub fn insert_body_bytes(
        &mut self,
        body_bytes: Vec<u8>,
    ) -> Result<ArchiveInsertResult, CommandLedgerError> {
        let command =
            WorldCommand::from_canonical_bytes(&body_bytes, CanonicalDecodeLimits::default())?;
        if command.canonical_bytes()? != body_bytes {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
        }
        self.insert_validated_body_bytes(body_bytes)
    }

    fn insert_validated_body_bytes(
        &mut self,
        body_bytes: Vec<u8>,
    ) -> Result<ArchiveInsertResult, CommandLedgerError> {
        let body_hash = command_body_hash_from_bytes(sha256(&body_bytes));
        match self.entries.get(&body_hash) {
            Some(existing) if existing.as_ref() == body_bytes.as_slice() => {
                Ok(ArchiveInsertResult::Existing(body_hash))
            }
            Some(_) => Err(CommandLedgerError::CommandBodyHashCollision),
            None => {
                let leaf_hash = command_body_archive_leaf_hash(body_hash, &body_bytes)?;
                let command_id = compute_command_id_from_body_bytes(&body_bytes)?;
                let insertion_index = match self
                    .leaf_hashes
                    .binary_search_by_key(&body_hash, |(hash, _)| *hash)
                {
                    Ok(_) => return Err(CommandLedgerError::CommandBodyArchiveCorrupt),
                    Err(insertion_index) => insertion_index,
                };
                Arc::make_mut(&mut self.entries).insert(body_hash, Arc::from(body_bytes));
                Arc::make_mut(&mut self.leaf_hashes)
                    .insert(insertion_index, (body_hash, leaf_hash));
                Arc::make_mut(&mut self.command_ids).insert(body_hash, command_id);
                self.manifest = Arc::new(std::sync::OnceLock::new());
                Ok(ArchiveInsertResult::Inserted(body_hash))
            }
        }
    }

    pub fn insert_command(
        &mut self,
        command: &WorldCommand,
    ) -> Result<ArchiveInsertResult, CommandLedgerError> {
        self.insert_validated_body_bytes(command.canonical_bytes()?)
    }

    /// Validates a bounded set of new canonical bodies against this archive.
    /// The exact resulting manifest remains lazy until publication needs it.
    pub fn prepare_additions(
        &self,
        additions: BTreeMap<CommandBodyHash, Arc<[u8]>>,
    ) -> Result<PreparedCommandBodyArchiveUpdate, CommandLedgerError> {
        let mut addition_leaf_hashes = Vec::with_capacity(additions.len());
        let mut addition_command_ids = Vec::with_capacity(additions.len());
        for (body_hash, body_bytes) in &additions {
            if self.entries.contains_key(body_hash) {
                return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
            }
            let (leaf_hash, command_id) = validate_archive_entry(*body_hash, body_bytes)?;
            addition_leaf_hashes.push((*body_hash, leaf_hash));
            addition_command_ids.push((*body_hash, command_id));
        }
        let entry_count = self
            .entries
            .len()
            .checked_add(additions.len())
            .ok_or(CommandLedgerError::CountOverflow)?;
        let entry_count =
            u64::try_from(entry_count).map_err(|_| CommandLedgerError::CountOverflow)?;
        Ok(PreparedCommandBodyArchiveUpdate {
            base_entry_count: self.entries.len(),
            base_leaf_hashes: self.leaf_hashes.clone(),
            base_command_ids: self.command_ids.clone(),
            additions,
            addition_leaf_hashes,
            addition_command_ids,
            entry_count,
            manifest: std::sync::OnceLock::new(),
        })
    }

    /// Applies an update that was validated against the current archive.
    /// Runtime generation validation guarantees that the base cannot change
    /// between preparation and this infallible commit point.
    pub fn commit_prepared_additions(&mut self, update: PreparedCommandBodyArchiveUpdate) {
        let PreparedCommandBodyArchiveUpdate {
            base_entry_count,
            base_leaf_hashes,
            base_command_ids,
            additions,
            addition_leaf_hashes,
            addition_command_ids,
            entry_count,
            manifest,
        } = update;
        assert_eq!(
            self.entries.len(),
            base_entry_count,
            "prepared archive update belongs to a different base generation"
        );
        assert!(
            Arc::ptr_eq(&self.leaf_hashes, &base_leaf_hashes),
            "prepared archive update belongs to a different base generation"
        );
        drop(base_leaf_hashes);
        assert!(
            Arc::ptr_eq(&self.command_ids, &base_command_ids),
            "prepared archive update belongs to a different base generation"
        );
        drop(base_command_ids);
        let entries = Arc::make_mut(&mut self.entries);
        for (body_hash, body_bytes) in additions {
            let replaced = entries.insert(body_hash, body_bytes);
            debug_assert!(replaced.is_none());
        }
        let leaf_hashes = Arc::make_mut(&mut self.leaf_hashes);
        for (body_hash, leaf_hash) in addition_leaf_hashes {
            let insertion_index = leaf_hashes
                .binary_search_by_key(&body_hash, |(hash, _)| *hash)
                .expect_err("validated archive addition is absent from the base");
            leaf_hashes.insert(insertion_index, (body_hash, leaf_hash));
        }
        let command_ids = Arc::make_mut(&mut self.command_ids);
        for (body_hash, command_id) in addition_command_ids {
            let replaced = command_ids.insert(body_hash, command_id);
            debug_assert!(replaced.is_none());
        }
        debug_assert_eq!(self.entries.len(), self.leaf_hashes.len());
        debug_assert_eq!(self.entries.len(), self.command_ids.len());
        debug_assert_eq!(self.entries.len(), entry_count as usize);
        // Preserve the prepared update's exact lazy state. Ordinary live ticks
        // intentionally transfer an empty cache; a checkpoint path may
        // initialize the update once before cloning/commit and transfers that
        // already computed value without recomputing the complete history.
        self.manifest = Arc::new(manifest);
    }

    #[cfg(test)]
    pub(crate) fn manifest_is_materialized(&self) -> bool {
        self.manifest.get().is_some()
    }

    pub fn manifest(&self) -> Result<CommandBodyArchiveManifestV1, CommandLedgerError> {
        if self.entries.len() != self.leaf_hashes.len()
            || self.entries.len() != self.command_ids.len()
        {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
        }
        if let Some(manifest) = self.manifest.get() {
            return Ok(*manifest);
        }
        let entry_count =
            u64::try_from(self.entries.len()).map_err(|_| CommandLedgerError::CountOverflow)?;
        let manifest = CommandBodyArchiveManifestV1 {
            schema_version: COMMAND_BODY_ARCHIVE_SCHEMA_VERSION,
            entry_count,
            archive_root: command_body_archive_root_from_leaves(
                self.leaf_hashes.iter().map(|(_, leaf_hash)| *leaf_hash),
                entry_count,
            )?,
        };
        let _ = self.manifest.set(manifest);
        Ok(self.manifest.get().copied().unwrap_or(manifest))
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        // Entries and leaves are private and every construction/mutation path
        // validates canonical bodies, declared hashes, and leaf hashes before
        // publishing a `CommandBodyArchiveV1`. Revalidating every body here
        // would decode the complete append-only history at each checkpoint.
        if self.entries.len() != self.leaf_hashes.len()
            || self.entries.len() != self.command_ids.len()
            || self
                .entries
                .keys()
                .zip(self.leaf_hashes.iter().map(|(body_hash, _)| body_hash))
                .any(|(entry_hash, leaf_hash)| entry_hash != leaf_hash)
            || self.entries.keys().ne(self.command_ids.keys())
        {
            return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
        }
        let _ = self.manifest()?;
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut entries = Vec::new();
        entries.extend_from_slice(
            &u32::try_from(self.entries.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        for (body_hash, body_bytes) in self.entries.iter() {
            entries.extend_from_slice(body_hash.as_bytes());
            extend_u32_bytes(&mut entries, body_bytes)?;
        }
        encode_canonical_segment(
            COMMAND_BODY_ARCHIVE_OWNER_ID,
            COMMAND_BODY_ARCHIVE_SCHEMA_ID,
            COMMAND_BODY_ARCHIVE_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    COMMAND_BODY_ARCHIVE_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_MAP, entries),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CommandLedgerError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != COMMAND_BODY_ARCHIVE_OWNER_ID
            || segment.schema_id != COMMAND_BODY_ARCHIVE_SCHEMA_ID
            || segment.segment_id != COMMAND_BODY_ARCHIVE_SEGMENT_ID
        {
            return Err(CommandLedgerError::WrongEnvelope);
        }
        require_ledger_fields(
            &segment,
            &[(1, CANONICAL_TYPE_U16), (2, CANONICAL_TYPE_MAP)],
        )?;
        let version = u16::from_le_bytes(fixed_field(&segment, 1)?);
        if version != COMMAND_BODY_ARCHIVE_SCHEMA_VERSION {
            return Err(CommandLedgerError::UnsupportedArchiveVersion(version));
        }
        let mut cursor = CanonicalCursor::new(&ledger_field(&segment, 2)?.payload);
        let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
            CanonicalDecodeError::TooManyFields { actual, limit }
        })?;
        let mut entries: BTreeMap<CommandBodyHash, Arc<[u8]>> = BTreeMap::new();
        let mut previous = None;
        for _ in 0..count {
            let body_hash = CommandBodyHash::from_bytes(read_array(&mut cursor)?);
            if previous.is_some_and(|prior| prior >= body_hash) {
                return Err(CommandLedgerError::MapNotStrictlySorted);
            }
            let body_bytes = cursor
                .read_u32_length_prefixed(limits.max_total_bytes)?
                .to_vec();
            entries.insert(body_hash, Arc::from(body_bytes));
            previous = Some(body_hash);
        }
        cursor.finish()?;
        let validated_entries = entries
            .iter()
            .map(|(body_hash, body_bytes)| {
                validate_archive_entry(*body_hash, body_bytes)
                    .map(|(leaf_hash, command_id)| (*body_hash, leaf_hash, command_id))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let leaf_hashes = validated_entries
            .iter()
            .map(|(body_hash, leaf_hash, _)| (*body_hash, *leaf_hash))
            .collect();
        let command_ids = validated_entries
            .into_iter()
            .map(|(body_hash, _, command_id)| (body_hash, command_id))
            .collect();
        let archive = Self {
            entries: Arc::new(entries),
            leaf_hashes: Arc::new(leaf_hashes),
            command_ids: Arc::new(command_ids),
            manifest: Arc::new(std::sync::OnceLock::new()),
        };
        archive.validate()?;
        if archive.canonical_bytes()? != bytes {
            return Err(CommandLedgerError::NonCanonicalEncoding);
        }
        Ok(archive)
    }
}

fn validate_archive_entry(
    body_hash: CommandBodyHash,
    body_bytes: &[u8],
) -> Result<([u8; 32], CommandId), CommandLedgerError> {
    let command = WorldCommand::from_canonical_bytes(body_bytes, CanonicalDecodeLimits::default())?;
    if command.canonical_bytes()?.as_slice() != body_bytes
        || command_body_hash_from_bytes(sha256(body_bytes)) != body_hash
    {
        return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
    }
    Ok((
        command_body_archive_leaf_hash(body_hash, body_bytes)?,
        compute_command_id_from_body_bytes(body_bytes)?,
    ))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInsertResult {
    Inserted(CommandBodyHash),
    Existing(CommandBodyHash),
}
