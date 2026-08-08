use std::error::Error;
use std::fmt::{Display, Formatter};
#[cfg(not(windows))]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};

const SLOT_COUNT: u64 = 2;
const CURRENT_FILE: &str = "CURRENT";
const CURRENT_STAGING_FILE: &str = "CURRENT.new";
const SNAPSHOT_FILE: &str = "session.snapshot.v4.bin";
const MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionPublicationV2 {
    pub generation_id: ContentHash,
    pub sequence: u64,
    pub expected_previous_generation: Option<ContentHash>,
    pub snapshot: Vec<u8>,
}

impl SessionPublicationV2 {
    pub fn new(
        sequence: u64,
        expected_previous_generation: Option<ContentHash>,
        snapshot: Vec<u8>,
    ) -> Result<Self, SessionStoreError> {
        validate_snapshot(&snapshot)?;
        let generation_id = generation_id(sequence, &snapshot);
        Ok(Self {
            generation_id,
            sequence,
            expected_previous_generation,
            snapshot,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedSessionGenerationV2 {
    pub generation_id: ContentHash,
    pub sequence: u64,
    pub slot: u8,
    pub snapshot: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct SessionStore {
    root: PathBuf,
}

impl SessionStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn current_path(&self) -> PathBuf {
        self.root.join(CURRENT_FILE)
    }

    pub fn publish(
        &self,
        publication: &SessionPublicationV2,
    ) -> Result<ContentHash, SessionStoreError> {
        validate_snapshot(&publication.snapshot)?;
        if generation_id(publication.sequence, &publication.snapshot) != publication.generation_id {
            return Err(SessionStoreError::SnapshotInvalid);
        }
        let current = self.load_current_optional()?;
        if current.as_ref().map(|value| value.generation_id)
            != publication.expected_previous_generation
        {
            return Err(SessionStoreError::PriorGenerationMismatch);
        }
        if current
            .as_ref()
            .is_some_and(|value| publication.sequence <= value.sequence)
        {
            return Err(SessionStoreError::SequenceStale);
        }

        fs::create_dir_all(&self.root)
            .map_err(|source| SessionStoreError::io("create session root", &self.root, source))?;
        let slot = u8::try_from(publication.sequence % SLOT_COUNT)
            .map_err(|_| SessionStoreError::SequenceStale)?;
        let slot_path = self.slot_path(slot);
        let staging = self.root.join(format!("slot-{slot}.new"));
        remove_directory(&staging)?;
        fs::create_dir_all(&staging)
            .map_err(|source| SessionStoreError::io("create session staging", &staging, source))?;
        write_new_synced(&staging.join(SNAPSHOT_FILE), &publication.snapshot)?;
        sync_directory(&staging)?;

        remove_directory(&slot_path)?;
        fs::rename(&staging, &slot_path)
            .map_err(|source| SessionStoreError::io("publish session slot", &slot_path, source))?;
        sync_directory(&self.root)?;

        let pointer_staging = self.root.join(CURRENT_STAGING_FILE);
        remove_file(&pointer_staging)?;
        write_new_synced(
            &pointer_staging,
            format!(
                "{} {slot} {}\n",
                publication.sequence,
                publication.generation_id.to_hex()
            )
            .as_bytes(),
        )?;
        remove_file(&self.current_path())?;
        fs::rename(&pointer_staging, self.current_path()).map_err(|source| {
            SessionStoreError::io("publish session pointer", self.current_path(), source)
        })?;
        sync_directory(&self.root)?;
        Ok(publication.generation_id)
    }

    pub fn load_current(&self) -> Result<PublishedSessionGenerationV2, SessionStoreError> {
        self.load_current_optional()?
            .ok_or(SessionStoreError::NoCurrent)
    }

    fn load_current_optional(
        &self,
    ) -> Result<Option<PublishedSessionGenerationV2>, SessionStoreError> {
        if !self.current_path().exists() {
            return Ok(None);
        }
        let pointer = fs::read_to_string(self.current_path()).map_err(|source| {
            SessionStoreError::io("read session pointer", self.current_path(), source)
        })?;
        let mut fields = pointer.split_whitespace();
        let sequence = fields
            .next()
            .ok_or(SessionStoreError::PointerInvalid)?
            .parse::<u64>()
            .map_err(|_| SessionStoreError::PointerInvalid)?;
        let slot = fields
            .next()
            .ok_or(SessionStoreError::PointerInvalid)?
            .parse::<u8>()
            .map_err(|_| SessionStoreError::PointerInvalid)?;
        let expected_hash = parse_hash(fields.next().ok_or(SessionStoreError::PointerInvalid)?)?;
        if fields.next().is_some()
            || u64::from(slot) >= SLOT_COUNT
            || slot != (sequence % SLOT_COUNT) as u8
        {
            return Err(SessionStoreError::PointerInvalid);
        }
        let snapshot_path = self.slot_path(slot).join(SNAPSHOT_FILE);
        let snapshot = fs::read(&snapshot_path).map_err(|source| {
            SessionStoreError::io("read session snapshot", &snapshot_path, source)
        })?;
        validate_snapshot(&snapshot)?;
        let actual_hash = generation_id(sequence, &snapshot);
        if actual_hash != expected_hash {
            return Err(SessionStoreError::SnapshotInvalid);
        }
        Ok(Some(PublishedSessionGenerationV2 {
            generation_id: actual_hash,
            sequence,
            slot,
            snapshot,
        }))
    }

    fn slot_path(&self, slot: u8) -> PathBuf {
        self.root.join(format!("slot-{slot}"))
    }
}

fn generation_id(sequence: u64, snapshot: &[u8]) -> ContentHash {
    let mut preimage = b"nextengine.session-snapshot.v4\0".to_vec();
    preimage.extend_from_slice(&sequence.to_le_bytes());
    preimage.extend_from_slice(snapshot);
    content_hash_from_bytes(sha256(&preimage))
}

fn validate_snapshot(snapshot: &[u8]) -> Result<(), SessionStoreError> {
    if snapshot.is_empty() || snapshot.len() > MAX_SNAPSHOT_BYTES {
        return Err(SessionStoreError::SnapshotInvalid);
    }
    Ok(())
}

fn parse_hash(value: &str) -> Result<ContentHash, SessionStoreError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SessionStoreError::PointerInvalid);
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    Ok(ContentHash::from_bytes(bytes))
}

fn nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => unreachable!("hex was validated"),
    }
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), SessionStoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| SessionStoreError::io("create session file", path, source))?;
    file.write_all(bytes)
        .map_err(|source| SessionStoreError::io("write session file", path, source))?;
    file.sync_all()
        .map_err(|source| SessionStoreError::io("sync session file", path, source))
}

#[cfg(not(windows))]
fn sync_directory(path: &Path) -> Result<(), SessionStoreError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| SessionStoreError::io("sync session directory", path, source))
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<(), SessionStoreError> {
    Ok(())
}

fn remove_file(path: &Path) -> Result<(), SessionStoreError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(SessionStoreError::io("remove session file", path, source)),
    }
}

fn remove_directory(path: &Path) -> Result<(), SessionStoreError> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(SessionStoreError::io(
            "remove session directory",
            path,
            source,
        )),
    }
}

#[derive(Debug)]
pub enum SessionStoreError {
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    NoCurrent,
    PointerInvalid,
    SnapshotInvalid,
    PriorGenerationMismatch,
    SequenceStale,
}

impl SessionStoreError {
    fn io(operation: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }

    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Io { .. } => "SESSION_STORE_IO",
            Self::NoCurrent => "SESSION_STORE_EMPTY",
            Self::PointerInvalid | Self::SnapshotInvalid => "SESSION_SNAPSHOT_INVALID",
            Self::PriorGenerationMismatch | Self::SequenceStale => "SESSION_PUBLICATION_STALE",
        }
    }
}

impl Display for SessionStoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io {
                operation, path, ..
            } => write!(formatter, "{operation}: {}", path.display()),
            Self::NoCurrent => formatter.write_str("no current session snapshot exists"),
            Self::PointerInvalid => formatter.write_str("session CURRENT pointer is invalid"),
            Self::SnapshotInvalid => formatter.write_str("session snapshot is invalid"),
            Self::PriorGenerationMismatch => {
                formatter.write_str("session generation precondition failed")
            }
            Self::SequenceStale => formatter.write_str("session sequence is stale"),
        }
    }
}

impl Error for SessionStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternates_two_slots_and_round_trips_current_snapshot() {
        let root = std::env::temp_dir().join(format!("next-session-store-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let store = SessionStore::new(&root);
        let first = SessionPublicationV2::new(0, None, vec![1, 2, 3]).expect("first");
        store.publish(&first).expect("publish first");
        let second =
            SessionPublicationV2::new(1, Some(first.generation_id), vec![4, 5]).expect("second");
        store.publish(&second).expect("publish second");
        let current = store.load_current().expect("load current");
        assert_eq!(current.snapshot, vec![4, 5]);
        assert_eq!(current.slot, 1);
        assert!(root.join("slot-0").join(SNAPSHOT_FILE).exists());
        assert!(root.join("slot-1").join(SNAPSHOT_FILE).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_stale_publication() {
        let root =
            std::env::temp_dir().join(format!("next-session-store-stale-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let store = SessionStore::new(&root);
        let first = SessionPublicationV2::new(0, None, vec![1]).expect("first");
        store.publish(&first).expect("publish first");
        let stale = SessionPublicationV2::new(1, None, vec![2]).expect("stale");
        assert!(matches!(
            store.publish(&stale),
            Err(SessionStoreError::PriorGenerationMismatch)
        ));
        let _ = fs::remove_dir_all(root);
    }
}
