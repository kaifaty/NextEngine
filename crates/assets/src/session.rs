use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use next_contracts::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CanonicalDecodeLimits, CanonicalField, decode_canonical_segment,
    encode_canonical_segment, sha256,
};
use next_contracts::ids::{ApplicationSessionId, ContentHash, content_hash_from_bytes};

#[cfg(test)]
use crate::content::ContentPublishFault;
use crate::content::{
    CONTENT_MAX_FILE_BYTES, ContentPublicationV1, ContentStore, ContentStoreError,
    PublicationFileV1, PublishedContentGenerationV1,
};

const SESSION_INDEX_FILE: &str = "session/index.bin";
const SESSION_SNAPSHOT_FILE: &str = "session/snapshot.bin";
const SESSION_OBJECT_DIRECTORY: &str = "objects";
const SESSION_OBJECT_PACK_INDEX_FILE: &str = "session/object-pack-index.bin";
const SESSION_INDEX_SCHEMA_VERSION: u32 = 1;
const SESSION_OBJECT_PACK_INDEX_SCHEMA_VERSION: u32 = 1;
const SESSION_MAX_OBJECTS: usize = 4_096;
const SESSION_MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;
const SESSION_OBJECT_PACK_LOCATION_BYTES: usize = 32 + 4 + 8 + 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionObjectV1 {
    content_hash: ContentHash,
    bytes: Arc<[u8]>,
}

impl SessionObjectV1 {
    #[must_use]
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes = bytes.into();
        Self {
            content_hash: content_hash_from_bytes(sha256(&bytes)),
            bytes,
        }
    }

    /// Reconstructs an object from a previously computed content hash.
    ///
    /// The caller must guarantee that `content_hash` equals the SHA-256 of
    /// `bytes` (for example a map key produced by [`Self::new`]). This
    /// avoids re-hashing every object on each publication; the invariant is
    /// still checked in debug builds.
    #[must_use]
    pub fn from_shared_parts(content_hash: ContentHash, bytes: Arc<[u8]>) -> Self {
        debug_assert_eq!(content_hash, content_hash_from_bytes(sha256(&bytes)));
        Self {
            content_hash,
            bytes,
        }
    }

    #[must_use]
    pub const fn content_hash(&self) -> ContentHash {
        self.content_hash
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn shared_bytes(&self) -> Arc<[u8]> {
        self.bytes.clone()
    }

    #[must_use]
    pub fn into_shared_bytes(self) -> Arc<[u8]> {
        self.bytes
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
        objects.sort_by_key(SessionObjectV1::content_hash);
        if objects
            .windows(2)
            .any(|pair| pair[0].content_hash() == pair[1].content_hash())
        {
            return Err(SessionStoreError::DuplicateObject);
        }
        if superseded_session_id.is_some()
            && (live_session_id.is_none() || superseded_session_id == live_session_id)
        {
            return Err(SessionStoreError::LiveSessionConflict);
        }
        let snapshot_hash = content_hash_from_bytes(sha256(&snapshot));
        let object_hashes: Vec<_> = objects.iter().map(SessionObjectV1::content_hash).collect();
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
                .map(SessionObjectV1::content_hash)
                .collect::<Vec<_>>(),
        )
    }

    #[cfg(test)]
    fn content_publication(&self) -> Result<ContentPublicationV1, SessionStoreError> {
        Ok(self.packing_plan()?.content)
    }

    fn packing_plan(&self) -> Result<SessionPackingPlanV1, SessionStoreError> {
        let mut files = vec![
            PublicationFileV1::new(SESSION_INDEX_FILE, self.index_bytes()?)?,
            PublicationFileV1::new(SESSION_SNAPSHOT_FILE, self.snapshot.clone())?,
        ];
        let mut packs: Vec<Vec<u8>> = Vec::new();
        let mut locations = Vec::with_capacity(self.objects.len());
        for object in &self.objects {
            if object.bytes().len() > CONTENT_MAX_FILE_BYTES {
                return Err(ContentStoreError::LimitExceeded {
                    actual: object.bytes().len(),
                    limit: CONTENT_MAX_FILE_BYTES,
                }
                .into());
            }
            let needs_new_pack = packs.last().is_none_or(|pack| {
                !pack.is_empty()
                    && pack
                        .len()
                        .checked_add(object.bytes().len())
                        .is_none_or(|length| length > CONTENT_MAX_FILE_BYTES)
            });
            if needs_new_pack {
                packs.push(Vec::with_capacity(object.bytes().len()));
            }
            let pack_index = packs.len() - 1;
            let pack = packs.last_mut().expect("pack was created");
            let offset = pack.len();
            pack.extend_from_slice(object.bytes());
            locations.push(SessionObjectPackLocationV1 {
                object_hash: object.content_hash(),
                pack_index,
                offset,
                length: object.bytes().len(),
            });
        }
        files.push(PublicationFileV1::new(
            SESSION_OBJECT_PACK_INDEX_FILE,
            encode_object_pack_index(packs.len(), &locations)?,
        )?);
        for (ordinal, bytes) in packs.into_iter().enumerate() {
            files.push(PublicationFileV1::new(object_pack_path(ordinal), bytes)?);
        }
        Ok(SessionPackingPlanV1 {
            content: ContentPublicationV1::new(self.generation_id, files)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SessionObjectPackLocationV1 {
    object_hash: ContentHash,
    pack_index: usize,
    offset: usize,
    length: usize,
}

#[derive(Debug)]
struct SessionPackingPlanV1 {
    content: ContentPublicationV1,
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
    pub objects: BTreeMap<ContentHash, Arc<[u8]>>,
}

#[derive(Clone, Debug)]
pub struct SessionStore {
    root: PathBuf,
    content: ContentStore,
    cache: Arc<Mutex<SessionStoreCacheV1>>,
}

#[derive(Debug, Default)]
struct SessionStoreCacheV1 {
    current: Option<PublishedSessionGenerationV1>,
}

impl SessionStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            content: ContentStore::new(&root),
            root,
            cache: Arc::new(Mutex::new(SessionStoreCacheV1::default())),
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
        let plan = publication.packing_plan()?;
        let mut cache = self.cache()?;
        let current = self.current_for_publish(&mut cache)?;
        self.validate_registry_transition(publication, current.as_ref())?;
        self.prepare_superseded_generations(publication, current.as_ref())?;
        self.content.publish(&plan.content)?;
        self.prune_superseded_generations_after_commit(publication);
        cache.current = Some(published_generation(publication));
        Ok(publication.generation_id)
    }

    pub fn load_current(&self) -> Result<PublishedSessionGenerationV1, SessionStoreError> {
        let generation_id = self.content.current_generation_id()?;
        let publication = self.load_generation_closure(generation_id)?;
        let mut cache = self.cache()?;
        cache.current = Some(publication.clone());
        Ok(publication)
    }

    fn decode_generation(
        &self,
        generation: PublishedContentGenerationV1,
    ) -> Result<PublishedSessionGenerationV1, SessionStoreError> {
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
        if let Some(pack_index_bytes) = generation.file(SESSION_OBJECT_PACK_INDEX_FILE) {
            let pack_index = decode_object_pack_index(pack_index_bytes, &index.object_hashes)?;
            let packs = (0..pack_index.pack_count)
                .map(|ordinal| {
                    generation
                        .file(&object_pack_path(ordinal))
                        .ok_or(SessionStoreError::ObjectMissing)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut consumed_pack_bytes = vec![0_usize; pack_index.pack_count];
            for location in pack_index.locations {
                let pack = packs
                    .get(location.pack_index)
                    .ok_or(SessionStoreError::ObjectPackIndexInvalid)?;
                let end = location
                    .offset
                    .checked_add(location.length)
                    .ok_or(SessionStoreError::ObjectPackIndexInvalid)?;
                let bytes = pack
                    .get(location.offset..end)
                    .ok_or(SessionStoreError::ObjectPackIndexInvalid)?
                    .to_vec();
                if content_hash_from_bytes(sha256(&bytes)) != location.object_hash {
                    return Err(SessionStoreError::ObjectHashMismatch);
                }
                consumed_pack_bytes[location.pack_index] = end;
                objects.insert(location.object_hash, Arc::from(bytes));
            }
            if packs
                .iter()
                .zip(consumed_pack_bytes)
                .any(|(pack, consumed)| pack.len() != consumed)
            {
                return Err(SessionStoreError::ObjectPackIndexInvalid);
            }
            let expected_paths = 3 + pack_index.pack_count;
            if generation.files.len() != expected_paths {
                return Err(SessionStoreError::UnexpectedObject);
            }
        } else {
            for hash in &index.object_hashes {
                let raw_path = format!("{SESSION_OBJECT_DIRECTORY}/{}.bin", hash.to_hex());
                let bytes = generation
                    .file(&raw_path)
                    .ok_or(SessionStoreError::ObjectMissing)?
                    .to_vec();
                if content_hash_from_bytes(sha256(&bytes)) != *hash {
                    return Err(SessionStoreError::ObjectHashMismatch);
                }
                objects.insert(*hash, Arc::from(bytes));
            }
            let expected_paths = 2 + index.object_hashes.len();
            if generation.files.len() != expected_paths {
                return Err(SessionStoreError::UnexpectedObject);
            }
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

    fn load_generation_closure(
        &self,
        generation_id: ContentHash,
    ) -> Result<PublishedSessionGenerationV1, SessionStoreError> {
        let generation = self.content.load_generation_by_id(generation_id)?;
        let loaded = self.decode_generation(generation)?;
        if let Some(previous_generation) = loaded.expected_previous_generation {
            let previous = self.content.load_generation_by_id(previous_generation)?;
            self.decode_generation(previous)?;
        }
        Ok(loaded)
    }

    fn current_for_publish(
        &self,
        cache: &mut SessionStoreCacheV1,
    ) -> Result<Option<PublishedSessionGenerationV1>, SessionStoreError> {
        if !self.root.join(crate::CONTENT_CURRENT_FILE).exists() {
            *cache = SessionStoreCacheV1::default();
            return Ok(None);
        }
        let current_id = self.content.current_generation_id()?;
        if cache
            .current
            .as_ref()
            .is_some_and(|current| current.generation_id == current_id)
        {
            return Ok(cache.current.clone());
        }
        let publication = self.load_generation_closure(current_id)?;
        cache.current = Some(publication.clone());
        Ok(Some(publication))
    }

    fn validate_registry_transition(
        &self,
        publication: &SessionPublicationV1,
        current: Option<&PublishedSessionGenerationV1>,
    ) -> Result<(), SessionStoreError> {
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
        current: Option<&PublishedSessionGenerationV1>,
    ) -> Result<(), SessionStoreError> {
        let mut retained = retained_generations(publication);
        if let Some(previous_generation) =
            current.and_then(|current| current.expected_previous_generation)
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

    fn cache(&self) -> Result<MutexGuard<'_, SessionStoreCacheV1>, SessionStoreError> {
        self.cache
            .lock()
            .map_err(|_| SessionStoreError::CacheUnavailable)
    }

    #[cfg(test)]
    fn publish_with_fault(
        &self,
        publication: &SessionPublicationV1,
        fault: SessionPublishFault,
    ) -> Result<(), SessionStoreError> {
        let plan = publication.packing_plan()?;
        let mut cache = self.cache()?;
        let current = self.current_for_publish(&mut cache)?;
        self.validate_registry_transition(publication, current.as_ref())?;
        self.prepare_superseded_generations(publication, current.as_ref())?;
        let content_fault = match fault {
            SessionPublishFault::BeforeGenerationCommit => {
                ContentPublishFault::BeforeGenerationCommit
            }
            SessionPublishFault::BeforePointerSwitch => ContentPublishFault::BeforeCurrentSwitch,
        };
        self.content
            .publish_with_fault(&plan.content, content_fault)?;
        self.prune_superseded_generations_after_commit(publication);
        cache.current = Some(published_generation(publication));
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

fn published_generation(publication: &SessionPublicationV1) -> PublishedSessionGenerationV1 {
    PublishedSessionGenerationV1 {
        generation_id: publication.generation_id,
        sequence: publication.sequence,
        project_composition_lock_hash: publication.project_composition_lock_hash,
        live_session_id: publication.live_session_id,
        expected_previous_generation: publication.expected_previous_generation,
        superseded_session_id: publication.superseded_session_id,
        snapshot: publication.snapshot.clone(),
        objects: publication
            .objects
            .iter()
            .map(|object| (object.content_hash(), object.shared_bytes()))
            .collect(),
    }
}

struct SessionObjectPackIndexV1 {
    pack_count: usize,
    locations: Vec<SessionObjectPackLocationV1>,
}

fn object_pack_path(ordinal: usize) -> String {
    format!("{SESSION_OBJECT_DIRECTORY}/pack-{ordinal:04}.bin")
}

fn encode_object_pack_index(
    pack_count: usize,
    locations: &[SessionObjectPackLocationV1],
) -> Result<Vec<u8>, SessionStoreError> {
    if pack_count > SESSION_MAX_OBJECTS || locations.len() > SESSION_MAX_OBJECTS {
        return Err(SessionStoreError::ObjectPackIndexInvalid);
    }
    let mut records = Vec::with_capacity(locations.len() * SESSION_OBJECT_PACK_LOCATION_BYTES);
    for location in locations {
        if location.pack_index >= pack_count
            || location.length > CONTENT_MAX_FILE_BYTES
            || location
                .offset
                .checked_add(location.length)
                .is_none_or(|end| end > CONTENT_MAX_FILE_BYTES)
        {
            return Err(SessionStoreError::ObjectPackIndexInvalid);
        }
        records.extend_from_slice(location.object_hash.as_bytes());
        records.extend_from_slice(
            &u32::try_from(location.pack_index)
                .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?
                .to_le_bytes(),
        );
        records.extend_from_slice(
            &u64::try_from(location.offset)
                .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?
                .to_le_bytes(),
        );
        records.extend_from_slice(
            &u64::try_from(location.length)
                .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?
                .to_le_bytes(),
        );
    }
    Ok(encode_canonical_segment(
        "nextengine.assets",
        "nextengine.session-object-pack-index.v1",
        "current",
        [
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                SESSION_OBJECT_PACK_INDEX_SCHEMA_VERSION
                    .to_le_bytes()
                    .to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_U32,
                u32::try_from(pack_count)
                    .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?
                    .to_le_bytes()
                    .to_vec(),
            ),
            CanonicalField::new(3, CANONICAL_TYPE_SEQUENCE, records),
        ],
    )?)
}

fn decode_object_pack_index(
    bytes: &[u8],
    expected_hashes: &[ContentHash],
) -> Result<SessionObjectPackIndexV1, SessionStoreError> {
    let decoded = decode_canonical_segment(bytes, CanonicalDecodeLimits::default())
        .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?;
    if decoded.owner_id != "nextengine.assets"
        || decoded.schema_id != "nextengine.session-object-pack-index.v1"
        || decoded.segment_id != "current"
        || decoded.fields.len() != 3
    {
        return Err(SessionStoreError::ObjectPackIndexInvalid);
    }
    let field = |id, tag| -> Result<&[u8], SessionStoreError> {
        let field = decoded
            .field(id)
            .ok_or(SessionStoreError::ObjectPackIndexInvalid)?;
        if field.type_tag != tag {
            return Err(SessionStoreError::ObjectPackIndexInvalid);
        }
        Ok(&field.payload)
    };
    let schema = u32::from_le_bytes(
        field(1, CANONICAL_TYPE_U32)?
            .try_into()
            .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?,
    );
    let pack_count = usize::try_from(u32::from_le_bytes(
        field(2, CANONICAL_TYPE_U32)?
            .try_into()
            .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?,
    ))
    .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?;
    let records = field(3, CANONICAL_TYPE_SEQUENCE)?;
    if schema != SESSION_OBJECT_PACK_INDEX_SCHEMA_VERSION
        || pack_count > SESSION_MAX_OBJECTS
        || records.len() % SESSION_OBJECT_PACK_LOCATION_BYTES != 0
        || records.len() / SESSION_OBJECT_PACK_LOCATION_BYTES != expected_hashes.len()
    {
        return Err(SessionStoreError::ObjectPackIndexInvalid);
    }
    let locations = records
        .chunks_exact(SESSION_OBJECT_PACK_LOCATION_BYTES)
        .zip(expected_hashes)
        .map(|(record, expected_hash)| {
            let object_hash = ContentHash::from_bytes(
                record[..32]
                    .try_into()
                    .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?,
            );
            let pack_index = usize::try_from(u32::from_le_bytes(
                record[32..36]
                    .try_into()
                    .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?,
            ))
            .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?;
            let offset = usize::try_from(u64::from_le_bytes(
                record[36..44]
                    .try_into()
                    .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?,
            ))
            .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?;
            let length = usize::try_from(u64::from_le_bytes(
                record[44..52]
                    .try_into()
                    .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?,
            ))
            .map_err(|_| SessionStoreError::ObjectPackIndexInvalid)?;
            if object_hash != *expected_hash
                || pack_index >= pack_count
                || length > CONTENT_MAX_FILE_BYTES
                || offset
                    .checked_add(length)
                    .is_none_or(|end| end > CONTENT_MAX_FILE_BYTES)
            {
                return Err(SessionStoreError::ObjectPackIndexInvalid);
            }
            Ok(SessionObjectPackLocationV1 {
                object_hash,
                pack_index,
                offset,
                length,
            })
        })
        .collect::<Result<Vec<_>, SessionStoreError>>()?;
    if expected_hashes.is_empty() != (pack_count == 0) {
        return Err(SessionStoreError::ObjectPackIndexInvalid);
    }
    let mut expected_pack_index = 0_usize;
    let mut expected_offset = 0_usize;
    for location in &locations {
        if location.pack_index == expected_pack_index {
            if location.offset != expected_offset {
                return Err(SessionStoreError::ObjectPackIndexInvalid);
            }
        } else if location.pack_index == expected_pack_index + 1 && location.offset == 0 {
            expected_pack_index = location.pack_index;
        } else {
            return Err(SessionStoreError::ObjectPackIndexInvalid);
        }
        expected_offset = location
            .offset
            .checked_add(location.length)
            .ok_or(SessionStoreError::ObjectPackIndexInvalid)?;
    }
    if !locations.is_empty() && expected_pack_index + 1 != pack_count {
        return Err(SessionStoreError::ObjectPackIndexInvalid);
    }
    Ok(SessionObjectPackIndexV1 {
        pack_count,
        locations,
    })
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
    ObjectPackIndexInvalid,
    CacheUnavailable,
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
            | Self::UnexpectedObject
            | Self::ObjectPackIndexInvalid
            | Self::CacheUnavailable => "SESSION_STORAGE_UNAVAILABLE",
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
            Self::ObjectPackIndexInvalid => {
                formatter.write_str("session object pack index is invalid")
            }
            Self::CacheUnavailable => formatter.write_str("session store cache is unavailable"),
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
mod tests;
