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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommandBodyArchiveV1 {
    pub(super) entries: BTreeMap<CommandBodyHash, Vec<u8>>,
}

impl CommandBodyArchiveV1 {
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<CommandBodyHash, Vec<u8>> {
        &self.entries
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
        let body_hash = command_body_hash_from_bytes(sha256(&body_bytes));
        match self.entries.get(&body_hash) {
            Some(existing) if existing == &body_bytes => {
                Ok(ArchiveInsertResult::Existing(body_hash))
            }
            Some(_) => Err(CommandLedgerError::CommandBodyHashCollision),
            None => {
                self.entries.insert(body_hash, body_bytes);
                Ok(ArchiveInsertResult::Inserted(body_hash))
            }
        }
    }

    pub fn insert_command(
        &mut self,
        command: &WorldCommand,
    ) -> Result<ArchiveInsertResult, CommandLedgerError> {
        self.insert_body_bytes(command.canonical_bytes()?)
    }

    pub fn manifest(&self) -> Result<CommandBodyArchiveManifestV1, CommandLedgerError> {
        let entry_count =
            u64::try_from(self.entries.len()).map_err(|_| CommandLedgerError::CountOverflow)?;
        Ok(CommandBodyArchiveManifestV1 {
            schema_version: COMMAND_BODY_ARCHIVE_SCHEMA_VERSION,
            entry_count,
            archive_root: command_body_archive_root(&self.entries)?,
        })
    }

    pub fn validate(&self) -> Result<(), CommandLedgerError> {
        for (declared_hash, body_bytes) in &self.entries {
            let command =
                WorldCommand::from_canonical_bytes(body_bytes, CanonicalDecodeLimits::default())?;
            if command.canonical_bytes()? != *body_bytes {
                return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
            }
            let computed_hash = command_body_hash_from_bytes(sha256(body_bytes));
            if declared_hash != &computed_hash {
                return Err(CommandLedgerError::CommandBodyArchiveCorrupt);
            }
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
        for (body_hash, body_bytes) in &self.entries {
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
        let mut entries = BTreeMap::new();
        let mut previous = None;
        for _ in 0..count {
            let body_hash = CommandBodyHash::from_bytes(read_array(&mut cursor)?);
            if previous.is_some_and(|prior| prior >= body_hash) {
                return Err(CommandLedgerError::MapNotStrictlySorted);
            }
            let body_bytes = cursor
                .read_u32_length_prefixed(limits.max_total_bytes)?
                .to_vec();
            entries.insert(body_hash, body_bytes);
            previous = Some(body_hash);
        }
        cursor.finish()?;
        let archive = Self { entries };
        archive.validate()?;
        if archive.canonical_bytes()? != bytes {
            return Err(CommandLedgerError::NonCanonicalEncoding);
        }
        Ok(archive)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInsertResult {
    Inserted(CommandBodyHash),
    Existing(CommandBodyHash),
}
