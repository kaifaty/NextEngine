use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use next_contracts::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CanonicalDecodeLimits, CanonicalField, decode_canonical_segment,
    encode_canonical_segment, sha256,
};
use next_contracts::ids::{ApplicationSessionId, ContentHash, content_hash_from_bytes};

#[cfg(test)]
use crate::content::ContentPublishFault;
use crate::content::{ContentPublicationV1, ContentStore, ContentStoreError, PublicationFileV1};

const SESSION_INDEX_FILE: &str = "session/index.bin";
const SESSION_SNAPSHOT_FILE: &str = "session/snapshot.bin";
const SESSION_OBJECT_DIRECTORY: &str = "objects";
const SESSION_INDEX_SCHEMA_VERSION: u32 = 1;
const SESSION_MAX_OBJECTS: usize = 4_096;
const SESSION_MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionObjectV1 {
    pub content_hash: ContentHash,
    pub bytes: Vec<u8>,
}

impl SessionObjectV1 {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            content_hash: content_hash_from_bytes(sha256(&bytes)),
            bytes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionPublicationV1 {
    pub generation_id: ContentHash,
    pub sequence: u64,
    pub project_composition_lock_hash: ContentHash,
    pub live_session_id: Option<ApplicationSessionId>,
    pub expected_previous_generation: Option<ContentHash>,
    pub superseded_session_id: Option<ApplicationSessionId>,
    pub snapshot: Vec<u8>,
    pub objects: Vec<SessionObjectV1>,
}

impl SessionPublicationV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the atomic publication binds prior generation and live-session registry"
    )]
    pub fn new(
        sequence: u64,
        project_composition_lock_hash: ContentHash,
        live_session_id: Option<ApplicationSessionId>,
        expected_previous_generation: Option<ContentHash>,
        superseded_session_id: Option<ApplicationSessionId>,
        snapshot: Vec<u8>,
        mut objects: Vec<SessionObjectV1>,
    ) -> Result<Self, SessionStoreError> {
        if snapshot.is_empty() || snapshot.len() > SESSION_MAX_SNAPSHOT_BYTES {
            return Err(SessionStoreError::SnapshotInvalid);
        }
        if objects.len() > SESSION_MAX_OBJECTS {
            return Err(SessionStoreError::ObjectLimitExceeded);
        }
        objects.sort_by_key(|object| object.content_hash);
        for object in &objects {
            if content_hash_from_bytes(sha256(&object.bytes)) != object.content_hash {
                return Err(SessionStoreError::ObjectHashMismatch);
            }
        }
        if objects
            .windows(2)
            .any(|pair| pair[0].content_hash == pair[1].content_hash)
        {
            return Err(SessionStoreError::DuplicateObject);
        }
        if superseded_session_id.is_some()
            && (live_session_id.is_none() || superseded_session_id == live_session_id)
        {
            return Err(SessionStoreError::LiveSessionConflict);
        }
        let snapshot_hash = content_hash_from_bytes(sha256(&snapshot));
        let object_hashes: Vec<_> = objects.iter().map(|object| object.content_hash).collect();
        let index = encode_index(
            sequence,
            project_composition_lock_hash,
            live_session_id,
            snapshot_hash,
            expected_previous_generation,
            superseded_session_id,
            &object_hashes,
        )?;
        let mut preimage = b"nextengine.session-publication.v1\0".to_vec();
        preimage.extend_from_slice(&index);
        preimage.extend_from_slice(snapshot_hash.as_bytes());
        for hash in &object_hashes {
            preimage.extend_from_slice(hash.as_bytes());
        }
        Ok(Self {
            generation_id: content_hash_from_bytes(sha256(&preimage)),
            sequence,
            project_composition_lock_hash,
            live_session_id,
            expected_previous_generation,
            superseded_session_id,
            snapshot,
            objects,
        })
    }

    fn index_bytes(&self) -> Result<Vec<u8>, SessionStoreError> {
        encode_index(
            self.sequence,
            self.project_composition_lock_hash,
            self.live_session_id,
            content_hash_from_bytes(sha256(&self.snapshot)),
            self.expected_previous_generation,
            self.superseded_session_id,
            &self
                .objects
                .iter()
                .map(|object| object.content_hash)
                .collect::<Vec<_>>(),
        )
    }

    fn content_publication(&self) -> Result<ContentPublicationV1, SessionStoreError> {
        let mut files = vec![
            PublicationFileV1::new(SESSION_INDEX_FILE, self.index_bytes()?)?,
            PublicationFileV1::new(SESSION_SNAPSHOT_FILE, self.snapshot.clone())?,
        ];
        files.extend(
            self.objects
                .iter()
                .map(|object| {
                    PublicationFileV1::new(
                        format!(
                            "{SESSION_OBJECT_DIRECTORY}/{}.bin",
                            object.content_hash.to_hex()
                        ),
                        object.bytes.clone(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(ContentPublicationV1::new(self.generation_id, files)?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedSessionGenerationV1 {
    pub generation_id: ContentHash,
    pub sequence: u64,
    pub project_composition_lock_hash: ContentHash,
    pub live_session_id: Option<ApplicationSessionId>,
    pub expected_previous_generation: Option<ContentHash>,
    pub superseded_session_id: Option<ApplicationSessionId>,
    pub snapshot: Vec<u8>,
    pub objects: BTreeMap<ContentHash, Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct SessionStore {
    root: PathBuf,
    content: ContentStore,
}

impl SessionStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            content: ContentStore::new(&root),
            root,
        }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn publish(
        &self,
        publication: &SessionPublicationV1,
    ) -> Result<ContentHash, SessionStoreError> {
        let content_publication = publication.content_publication()?;
        self.validate_registry_transition(publication)?;
        self.prepare_superseded_generations(publication)?;
        self.content.publish(&content_publication)?;
        self.prune_superseded_generations_after_commit(publication);
        Ok(publication.generation_id)
    }

    pub fn load_current(&self) -> Result<PublishedSessionGenerationV1, SessionStoreError> {
        let generation = self.content.load_current()?;
        let index_bytes = generation
            .file(SESSION_INDEX_FILE)
            .ok_or(SessionStoreError::IndexInvalid)?;
        let index = decode_index(index_bytes)?;
        let snapshot = generation
            .file(SESSION_SNAPSHOT_FILE)
            .ok_or(SessionStoreError::SnapshotInvalid)?
            .to_vec();
        if content_hash_from_bytes(sha256(&snapshot)) != index.snapshot_hash {
            return Err(SessionStoreError::SnapshotInvalid);
        }
        let mut objects = BTreeMap::new();
        for hash in &index.object_hashes {
            let path = format!("{SESSION_OBJECT_DIRECTORY}/{}.bin", hash.to_hex());
            let bytes = generation
                .file(&path)
                .ok_or(SessionStoreError::ObjectMissing)?
                .to_vec();
            if content_hash_from_bytes(sha256(&bytes)) != *hash {
                return Err(SessionStoreError::ObjectHashMismatch);
            }
            objects.insert(*hash, bytes);
        }
        let expected_paths = 2 + index.object_hashes.len();
        if generation.files.len() != expected_paths {
            return Err(SessionStoreError::UnexpectedObject);
        }
        Ok(PublishedSessionGenerationV1 {
            generation_id: generation.generation_id,
            sequence: index.sequence,
            project_composition_lock_hash: index.project_composition_lock_hash,
            live_session_id: index.live_session_id,
            expected_previous_generation: index.expected_previous_generation,
            superseded_session_id: index.superseded_session_id,
            snapshot,
            objects,
        })
    }

    fn validate_registry_transition(
        &self,
        publication: &SessionPublicationV1,
    ) -> Result<(), SessionStoreError> {
        let current = if self.root.join(crate::CONTENT_CURRENT_FILE).exists() {
            Some(self.load_current()?)
        } else {
            None
        };
        match current {
            None => {
                if publication.expected_previous_generation.is_some()
                    || publication.superseded_session_id.is_some()
                {
                    return Err(SessionStoreError::PriorGenerationMismatch);
                }
            }
            Some(current) => {
                if publication.expected_previous_generation != Some(current.generation_id) {
                    return Err(SessionStoreError::PriorGenerationMismatch);
                }
                if let Some(current_live) = current.live_session_id {
                    if publication.project_composition_lock_hash
                        != current.project_composition_lock_hash
                    {
                        return Err(SessionStoreError::PriorGenerationMismatch);
                    }
                    let same = publication.live_session_id == Some(current_live)
                        && publication.superseded_session_id.is_none();
                    let recovery = publication.superseded_session_id == Some(current_live)
                        && publication.live_session_id != Some(current_live);
                    let close = publication.live_session_id.is_none()
                        && publication.superseded_session_id.is_none();
                    if !same && !recovery && !close {
                        return Err(SessionStoreError::LiveSessionConflict);
                    }
                } else if publication.superseded_session_id.is_some() {
                    return Err(SessionStoreError::LiveSessionConflict);
                }
                if publication.sequence <= current.sequence {
                    return Err(SessionStoreError::SequenceStale);
                }
            }
        }
        Ok(())
    }

    fn prune_superseded_generations_after_commit(&self, publication: &SessionPublicationV1) {
        let retained = retained_generations(publication);
        // CURRENT is the publication commit point. Cleanup is preflighted
        // before that switch, but an external I/O/TOCTOU failure afterwards
        // must not report the already committed publication as rolled back.
        // Any retained obsolete generation is non-authoritative and the next
        // publication preflight retries the bounded cleanup.
        let _cleanup_result = self.content.prune_generations_except(&retained);
    }

    fn prepare_superseded_generations(
        &self,
        publication: &SessionPublicationV1,
    ) -> Result<(), SessionStoreError> {
        let mut retained = retained_generations(publication);
        if self.root.join(crate::CONTENT_CURRENT_FILE).exists()
            && let Some(previous_generation) = self.load_current()?.expected_previous_generation
        {
            retained.push(previous_generation);
        }
        // Before a new commit, remove only generations older than the current
        // and its declared previous generation. This preserves both recovery
        // points if the new publication later fails, while preventing a prior
        // post-CURRENT cleanup failure from growing history without bound.
        self.content.prune_generations_except(&retained)?;
        Ok(())
    }

    #[cfg(test)]
    fn publish_with_fault(
        &self,
        publication: &SessionPublicationV1,
        fault: SessionPublishFault,
    ) -> Result<(), SessionStoreError> {
        let content_publication = publication.content_publication()?;
        self.validate_registry_transition(publication)?;
        self.prepare_superseded_generations(publication)?;
        let content_fault = match fault {
            SessionPublishFault::BeforeGenerationCommit => {
                ContentPublishFault::BeforeGenerationCommit
            }
            SessionPublishFault::BeforePointerSwitch => ContentPublishFault::BeforeCurrentSwitch,
        };
        self.content
            .publish_with_fault(&content_publication, content_fault)?;
        self.prune_superseded_generations_after_commit(publication);
        Ok(())
    }
}

fn retained_generations(publication: &SessionPublicationV1) -> Vec<ContentHash> {
    let mut retained = vec![publication.generation_id];
    if let Some(previous_generation) = publication.expected_previous_generation {
        retained.push(previous_generation);
    }
    retained
}

struct SessionIndexV1 {
    sequence: u64,
    project_composition_lock_hash: ContentHash,
    live_session_id: Option<ApplicationSessionId>,
    snapshot_hash: ContentHash,
    expected_previous_generation: Option<ContentHash>,
    superseded_session_id: Option<ApplicationSessionId>,
    object_hashes: Vec<ContentHash>,
}

#[allow(
    clippy::too_many_arguments,
    reason = "the canonical store index binds every publication and registry reference"
)]
fn encode_index(
    sequence: u64,
    project_composition_lock_hash: ContentHash,
    live_session_id: Option<ApplicationSessionId>,
    snapshot_hash: ContentHash,
    expected_previous_generation: Option<ContentHash>,
    superseded_session_id: Option<ApplicationSessionId>,
    object_hashes: &[ContentHash],
) -> Result<Vec<u8>, SessionStoreError> {
    let hash_sequence = object_hashes
        .iter()
        .flat_map(|hash| hash.as_bytes().iter().copied())
        .collect();
    Ok(encode_canonical_segment(
        "nextengine.assets",
        "nextengine.session-store-index.v1",
        "current",
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                SESSION_INDEX_SCHEMA_VERSION.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(2, CANONICAL_TYPE_U64, sequence.to_le_bytes().to_vec()),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_HASH256,
                project_composition_lock_hash.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_OPTIONAL,
                live_session_id.map_or_else(Vec::new, |id| id.as_bytes().to_vec()),
            ),
            CanonicalField::new(5, CANONICAL_TYPE_HASH256, snapshot_hash.as_bytes().to_vec()),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_OPTIONAL,
                expected_previous_generation.map_or_else(Vec::new, |hash| hash.as_bytes().to_vec()),
            ),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_OPTIONAL,
                superseded_session_id.map_or_else(Vec::new, |id| id.as_bytes().to_vec()),
            ),
            CanonicalField::new(8, CANONICAL_TYPE_SEQUENCE, hash_sequence),
        ],
    )?)
}

fn decode_index(bytes: &[u8]) -> Result<SessionIndexV1, SessionStoreError> {
    let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())?;
    if decoded.owner_id != "nextengine.assets"
        || decoded.schema_id != "nextengine.session-store-index.v1"
        || decoded.segment_id != "current"
        || decoded.fields.len() != 8
    {
        return Err(SessionStoreError::IndexInvalid);
    }
    let field = |id, tag| -> Result<&[u8], SessionStoreError> {
        let field = decoded.field(id).ok_or(SessionStoreError::IndexInvalid)?;
        if field.type_tag != tag {
            return Err(SessionStoreError::IndexInvalid);
        }
        Ok(&field.payload)
    };
    let schema = u32::from_le_bytes(
        field(1, CANONICAL_TYPE_U32)?
            .try_into()
            .map_err(|_| SessionStoreError::IndexInvalid)?,
    );
    if schema != SESSION_INDEX_SCHEMA_VERSION {
        return Err(SessionStoreError::IndexInvalid);
    }
    let sequence = u64::from_le_bytes(
        field(2, CANONICAL_TYPE_U64)?
            .try_into()
            .map_err(|_| SessionStoreError::IndexInvalid)?,
    );
    let object_bytes = field(8, CANONICAL_TYPE_SEQUENCE)?;
    if object_bytes.len() % 32 != 0 || object_bytes.len() / 32 > SESSION_MAX_OBJECTS {
        return Err(SessionStoreError::IndexInvalid);
    }
    let object_hashes = object_bytes
        .chunks_exact(32)
        .map(|bytes| ContentHash::from_bytes(bytes.try_into().expect("exact hash chunk")))
        .collect::<Vec<_>>();
    if object_hashes.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(SessionStoreError::IndexInvalid);
    }
    Ok(SessionIndexV1 {
        sequence,
        project_composition_lock_hash: decode_hash(field(3, CANONICAL_TYPE_HASH256)?)?,
        live_session_id: decode_optional_id(field(4, CANONICAL_TYPE_OPTIONAL)?)?,
        snapshot_hash: decode_hash(field(5, CANONICAL_TYPE_HASH256)?)?,
        expected_previous_generation: decode_optional_hash(field(6, CANONICAL_TYPE_OPTIONAL)?)?,
        superseded_session_id: decode_optional_id(field(7, CANONICAL_TYPE_OPTIONAL)?)?,
        object_hashes,
    })
}

fn decode_hash(bytes: &[u8]) -> Result<ContentHash, SessionStoreError> {
    Ok(ContentHash::from_bytes(
        bytes
            .try_into()
            .map_err(|_| SessionStoreError::IndexInvalid)?,
    ))
}

fn decode_optional_hash(bytes: &[u8]) -> Result<Option<ContentHash>, SessionStoreError> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        decode_hash(bytes).map(Some)
    }
}

fn decode_optional_id(bytes: &[u8]) -> Result<Option<ApplicationSessionId>, SessionStoreError> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ApplicationSessionId::from_bytes(
            bytes
                .try_into()
                .map_err(|_| SessionStoreError::IndexInvalid)?,
        )))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum SessionStoreError {
    Content(ContentStoreError),
    Canonical(next_contracts::canonical::CanonicalError),
    CanonicalDecode(next_contracts::canonical::CanonicalDecodeError),
    SnapshotInvalid,
    IndexInvalid,
    ObjectLimitExceeded,
    ObjectHashMismatch,
    DuplicateObject,
    ObjectMissing,
    UnexpectedObject,
    PriorGenerationMismatch,
    LiveSessionConflict,
    SequenceStale,
}

impl SessionStoreError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::LiveSessionConflict => "SESSION_LIVE_REGISTRY_CONFLICT",
            Self::PriorGenerationMismatch | Self::SequenceStale => {
                "SESSION_EXPECTED_REVISION_STALE"
            }
            Self::Content(_)
            | Self::Canonical(_)
            | Self::CanonicalDecode(_)
            | Self::SnapshotInvalid
            | Self::IndexInvalid
            | Self::ObjectLimitExceeded
            | Self::ObjectHashMismatch
            | Self::DuplicateObject
            | Self::ObjectMissing
            | Self::UnexpectedObject => "SESSION_STORAGE_UNAVAILABLE",
        }
    }
}

impl Display for SessionStoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Content(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
            Self::CanonicalDecode(error) => write!(formatter, "{error}"),
            Self::SnapshotInvalid => formatter.write_str("session snapshot is invalid"),
            Self::IndexInvalid => formatter.write_str("session index is invalid"),
            Self::ObjectLimitExceeded => formatter.write_str("session object limit exceeded"),
            Self::ObjectHashMismatch => formatter.write_str("session object hash mismatch"),
            Self::DuplicateObject => formatter.write_str("session object is duplicated"),
            Self::ObjectMissing => formatter.write_str("session object is missing"),
            Self::UnexpectedObject => formatter.write_str("unexpected session object exists"),
            Self::PriorGenerationMismatch => {
                formatter.write_str("session prior generation does not match")
            }
            Self::LiveSessionConflict => formatter.write_str("another session is live"),
            Self::SequenceStale => formatter.write_str("session store sequence is stale"),
        }
    }
}

impl Error for SessionStoreError {}

impl From<ContentStoreError> for SessionStoreError {
    fn from(value: ContentStoreError) -> Self {
        Self::Content(value)
    }
}

impl From<next_contracts::canonical::CanonicalError> for SessionStoreError {
    fn from(value: next_contracts::canonical::CanonicalError) -> Self {
        Self::Canonical(value)
    }
}

impl From<next_contracts::canonical::CanonicalDecodeError> for SessionStoreError {
    fn from(value: next_contracts::canonical::CanonicalDecodeError) -> Self {
        Self::CanonicalDecode(value)
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum SessionPublishFault {
    BeforeGenerationCommit,
    BeforePointerSwitch,
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn atomic_faults_keep_prior_generation_and_one_live_session() {
        for fault in [
            SessionPublishFault::BeforeGenerationCommit,
            SessionPublishFault::BeforePointerSwitch,
        ] {
            let root = test_root("fault");
            let store = SessionStore::new(&root);
            let first = publication(0, None, 1, None);
            store.publish(&first).expect("initial");
            let second = publication(1, Some(first.generation_id), 1, None);
            assert!(store.publish_with_fault(&second, fault).is_err());
            let loaded = store.load_current().expect("prior remains");
            assert_eq!(loaded.generation_id, first.generation_id);
            assert_eq!(
                loaded.live_session_id,
                Some(ApplicationSessionId::from_bytes([1; 16]))
            );
            std::fs::remove_dir_all(root).expect("cleanup");
        }
    }

    #[test]
    fn recovery_atomically_replaces_live_session() {
        let root = test_root("recovery");
        let store = SessionStore::new(&root);
        let first = publication(0, None, 1, None);
        store.publish(&first).expect("initial");
        let recovered = publication(1, Some(first.generation_id), 2, Some(1));
        store.publish(&recovered).expect("recovery");
        assert_eq!(
            store.load_current().expect("current").live_session_id,
            Some(ApplicationSessionId::from_bytes([2; 16]))
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn successful_publication_retains_only_current_and_previous_generations() {
        let root = test_root("bounded-generations");
        let store = SessionStore::new(&root);
        let mut previous = None;
        let mut prior_target: Option<ContentHash> = None;
        for sequence in 0..12 {
            let next = publication(sequence, previous, 1, None);
            store.publish(&next).expect("session publication");

            let generations = generation_names(&root);
            let mut expected = vec![next.generation_id.to_hex()];
            if let Some(prior_target) = prior_target {
                expected.push(prior_target.to_hex());
            }
            expected.sort();
            assert_eq!(generations, expected);

            previous = Some(next.generation_id);
            prior_target = Some(next.generation_id);
        }
        assert_eq!(
            store.load_current().expect("current").generation_id,
            previous.expect("published generation")
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn failed_third_publication_preserves_current_and_previous_generations() {
        let root = test_root("failed-third-preserves-history");
        let store = SessionStore::new(&root);
        let first = publication(0, None, 1, None);
        store.publish(&first).expect("first generation");
        let second = publication(1, Some(first.generation_id), 1, None);
        store.publish(&second).expect("second generation");
        let third = publication(2, Some(second.generation_id), 1, None);

        assert!(matches!(
            store.publish_with_fault(&third, SessionPublishFault::BeforeGenerationCommit),
            Err(SessionStoreError::Content(ContentStoreError::InjectedFault))
        ));
        assert_eq!(
            store
                .load_current()
                .expect("second remains current")
                .generation_id,
            second.generation_id
        );
        let mut retained = vec![first.generation_id.to_hex(), second.generation_id.to_hex()];
        retained.sort();
        assert_eq!(generation_names(&root), retained);

        store.publish(&third).expect("third retry");
        let mut retained = vec![second.generation_id.to_hex(), third.generation_id.to_hex()];
        retained.sort();
        assert_eq!(generation_names(&root), retained);
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn next_publication_repairs_a_prior_post_commit_cleanup_miss() {
        let root = test_root("repair-post-commit-cleanup");
        let store = SessionStore::new(&root);
        let first = publication(0, None, 1, None);
        store.publish(&first).expect("first generation");
        let second = publication(1, Some(first.generation_id), 1, None);
        store.publish(&second).expect("second generation");
        let third = publication(2, Some(second.generation_id), 1, None);

        store
            .content
            .publish(
                &third
                    .content_publication()
                    .expect("third content publication"),
            )
            .expect("simulate CURRENT commit before cleanup");
        assert_eq!(
            store.load_current().expect("third current").generation_id,
            third.generation_id
        );
        assert_eq!(generation_names(&root).len(), 3);

        let fourth = publication(3, Some(third.generation_id), 1, None);
        store
            .publish(&fourth)
            .expect("next publication repairs history");
        let mut retained = vec![third.generation_id.to_hex(), fourth.generation_id.to_hex()];
        retained.sort();
        assert_eq!(generation_names(&root), retained);
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn target_left_by_pointer_fault_is_retained_for_retry() {
        let root = test_root("pointer-retry");
        let store = SessionStore::new(&root);
        let first = publication(0, None, 1, None);
        store.publish(&first).expect("initial");
        let second = publication(1, Some(first.generation_id), 1, None);
        assert!(matches!(
            store.publish_with_fault(&second, SessionPublishFault::BeforePointerSwitch),
            Err(SessionStoreError::Content(ContentStoreError::InjectedFault))
        ));
        assert_eq!(generation_names(&root).len(), 2);

        store.publish(&second).expect("idempotent retry");
        assert_eq!(
            store.load_current().expect("retried current").generation_id,
            second.generation_id
        );
        assert_eq!(generation_names(&root).len(), 2);
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn invalid_generation_path_blocks_publish_before_current_switch() {
        let root = test_root("invalid-generation-path");
        let store = SessionStore::new(&root);
        let first = publication(0, None, 1, None);
        store.publish(&first).expect("initial");
        std::fs::create_dir(
            root.join(crate::CONTENT_GENERATIONS_DIRECTORY)
                .join("escape"),
        )
        .expect("invalid generation entry");

        let second = publication(1, Some(first.generation_id), 1, None);
        assert!(matches!(
            store.publish(&second),
            Err(SessionStoreError::Content(ContentStoreError::InvalidPath))
        ));
        assert_eq!(
            store
                .load_current()
                .expect("prior current remains")
                .generation_id,
            first.generation_id
        );
        assert!(
            !root
                .join(crate::CONTENT_GENERATIONS_DIRECTORY)
                .join(second.generation_id.to_hex())
                .exists()
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn crash_staging_at_partial_depths_is_removed_without_switching_current() {
        let root = test_root("crash-staging-retry");
        let store = SessionStore::new(&root);
        let first = publication(0, None, 1, None);
        store.publish(&first).expect("initial");
        let generations = root.join(crate::CONTENT_GENERATIONS_DIRECTORY);

        for (ordinal, relative_path) in [
            (1_u8, None),
            (2, Some("session")),
            (3, Some("objects/nested/partial.bin")),
        ] {
            let staging = generations.join(format!(
                ".{}.staging.{}",
                ContentHash::from_bytes([ordinal; 32]).to_hex(),
                4_000_000_000_u32 + u32::from(ordinal)
            ));
            std::fs::create_dir(&staging).expect("staging root");
            if let Some(relative_path) = relative_path {
                let partial = staging.join(relative_path);
                if partial.extension().is_some() {
                    std::fs::create_dir_all(partial.parent().expect("partial parent"))
                        .expect("partial parent tree");
                    std::fs::write(partial, b"partial").expect("partial file");
                } else {
                    std::fs::create_dir_all(partial).expect("partial directory");
                }
            }
        }

        let second = publication(1, Some(first.generation_id), 1, None);
        assert!(matches!(
            store.publish_with_fault(&second, SessionPublishFault::BeforePointerSwitch),
            Err(SessionStoreError::Content(ContentStoreError::InjectedFault))
        ));
        assert_eq!(
            store
                .load_current()
                .expect("fault keeps prior CURRENT")
                .generation_id,
            first.generation_id
        );
        let mut expected_generations =
            vec![first.generation_id.to_hex(), second.generation_id.to_hex()];
        expected_generations.sort();
        assert_eq!(generation_names(&root), expected_generations);

        store.publish(&second).expect("retry after crash cleanup");
        assert_eq!(
            store.load_current().expect("retried current").generation_id,
            second.generation_id
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn staging_shaped_non_directory_fails_closed_without_switching_current() {
        let root = test_root("crash-staging-file");
        let store = SessionStore::new(&root);
        let first = publication(0, None, 1, None);
        store.publish(&first).expect("initial");
        let staging = root
            .join(crate::CONTENT_GENERATIONS_DIRECTORY)
            .join(format!(
                ".{}.staging.4000000001",
                ContentHash::from_bytes([0x7a; 32]).to_hex()
            ));
        std::fs::write(&staging, b"not a directory").expect("staging-shaped file");

        let second = publication(1, Some(first.generation_id), 1, None);
        assert!(matches!(
            store.publish(&second),
            Err(SessionStoreError::Content(ContentStoreError::InvalidPath))
        ));
        assert_eq!(
            store
                .load_current()
                .expect("prior current remains")
                .generation_id,
            first.generation_id
        );
        assert!(
            !root
                .join(crate::CONTENT_GENERATIONS_DIRECTORY)
                .join(second.generation_id.to_hex())
                .exists()
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn one_thousand_open_close_cycles_keep_one_live_session_and_unique_receipts() {
        let root = test_root("thousand-cycles");
        let store = SessionStore::new(&root);
        let mut previous = None;
        let mut sequence = 0_u64;
        let mut receipts = std::collections::BTreeSet::new();
        for ordinal in 0..1_000_u64 {
            let mut id_bytes = [0_u8; 16];
            id_bytes[..8].copy_from_slice(&ordinal.to_le_bytes());
            let session_id = ApplicationSessionId::from_bytes(id_bytes);
            let opened = SessionPublicationV1::new(
                sequence,
                ContentHash::from_bytes([9; 32]),
                Some(session_id),
                previous,
                None,
                format!("open-{ordinal}").into_bytes(),
                vec![],
            )
            .expect("open publication");
            store.publish(&opened).expect("open");
            let current = store.load_current().expect("opened current");
            assert_eq!(current.live_session_id, Some(session_id));
            previous = Some(opened.generation_id);
            sequence += 1;

            let receipt = SessionObjectV1::new(format!("receipt-{ordinal}").into_bytes());
            assert!(receipts.insert(receipt.content_hash));
            let closed = SessionPublicationV1::new(
                sequence,
                ContentHash::from_bytes([9; 32]),
                None,
                previous,
                None,
                format!("closed-{ordinal}").into_bytes(),
                vec![receipt],
            )
            .expect("close publication");
            store.publish(&closed).expect("close");
            let current = store.load_current().expect("closed current");
            assert_eq!(current.live_session_id, None);
            assert_eq!(current.objects.len(), 1);
            previous = Some(closed.generation_id);
            sequence += 1;
        }
        assert_eq!(receipts.len(), 1_000);
        assert_eq!(generation_names(&root).len(), 2);
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    fn publication(
        sequence: u64,
        previous: Option<ContentHash>,
        live: u8,
        superseded: Option<u8>,
    ) -> SessionPublicationV1 {
        SessionPublicationV1::new(
            sequence,
            ContentHash::from_bytes([9; 32]),
            Some(ApplicationSessionId::from_bytes([live; 16])),
            previous,
            superseded.map(|id| ApplicationSessionId::from_bytes([id; 16])),
            vec![sequence as u8 + 1],
            vec![SessionObjectV1::new(vec![42, sequence as u8])],
        )
        .expect("publication")
    }

    fn test_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "nextengine-session-store-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn generation_names(root: &Path) -> Vec<String> {
        let mut names = std::fs::read_dir(root.join(crate::CONTENT_GENERATIONS_DIRECTORY))
            .expect("generation directory")
            .map(|entry| {
                entry
                    .expect("generation entry")
                    .file_name()
                    .into_string()
                    .expect("utf-8 generation name")
            })
            .collect::<Vec<_>>();
        names.sort();
        names
    }
}
