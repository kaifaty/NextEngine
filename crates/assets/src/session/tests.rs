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
fn session_objects_roundtrip_through_one_generation_pack() {
    let root = test_root("object-pack");
    let store = SessionStore::new(&root);
    let first_bytes = patterned_bytes(32 * 1024, 0x21);
    let first = object_publication(0, None, first_bytes.clone());
    store.publish(&first).expect("packed generation");
    let loaded = store.load_current().expect("packed generation roundtrip");
    assert_eq!(
        loaded.objects.get(&first.objects[0].content_hash),
        Some(&first_bytes)
    );
    let physical = store
        .content
        .load_generation_by_id(first.generation_id)
        .expect("physical generation");
    assert!(physical.file(SESSION_OBJECT_PACK_INDEX_FILE).is_some());
    assert_eq!(
        physical.file(&object_pack_path(0)),
        Some(first_bytes.as_slice())
    );
    assert!(
        physical
            .file(&format!(
                "{SESSION_OBJECT_DIRECTORY}/{}.bin",
                first.objects[0].content_hash.to_hex()
            ))
            .is_none()
    );
    assert_eq!(physical.files.len(), 4);
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn object_pack_corruption_fails_closed_before_returning_a_generation() {
    let root = test_root("pack-corruption");
    let store = SessionStore::new(&root);
    let publication = object_publication(0, None, patterned_bytes(16 * 1024, 0x31));
    store.publish(&publication).expect("packed generation");
    let pack = root
        .join(crate::CONTENT_GENERATIONS_DIRECTORY)
        .join(publication.generation_id.to_hex())
        .join(object_pack_path(0));
    std::fs::write(pack, vec![0x5a; 16 * 1024]).expect("corrupt pack");

    let reopened = SessionStore::new(&root);
    assert!(matches!(
        reopened.load_current(),
        Err(SessionStoreError::Content(ContentStoreError::HashMismatch))
    ));
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn missing_object_pack_fails_closed_before_returning_a_generation() {
    let root = test_root("pack-missing");
    let store = SessionStore::new(&root);
    let publication = object_publication(0, None, patterned_bytes(8 * 1024, 0x41));
    store.publish(&publication).expect("packed generation");
    let pack = root
        .join(crate::CONTENT_GENERATIONS_DIRECTORY)
        .join(publication.generation_id.to_hex())
        .join(object_pack_path(0));
    std::fs::remove_file(pack).expect("remove pack");

    let reopened = SessionStore::new(&root);
    assert!(matches!(
        reopened.load_current(),
        Err(SessionStoreError::Content(ContentStoreError::MissingFile(
            _
        )))
    ));
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn pointer_fault_keeps_prior_packed_closure_and_retry_is_exact() {
    let root = test_root("packed-pointer-retry");
    let store = SessionStore::new(&root);
    let first = object_publication(0, None, patterned_bytes(12 * 1024, 0x51));
    store.publish(&first).expect("first");
    let mut second_bytes = patterned_bytes(12 * 1024, 0x51);
    second_bytes[4 * 1024 + 7] ^= 0x77;
    let second = object_publication(1, Some(first.generation_id), second_bytes.clone());

    assert!(matches!(
        store.publish_with_fault(&second, SessionPublishFault::BeforePointerSwitch),
        Err(SessionStoreError::Content(ContentStoreError::InjectedFault))
    ));
    assert_eq!(
        store
            .load_current()
            .expect("prior packed current")
            .generation_id,
        first.generation_id
    );

    store.publish(&second).expect("packed retry");
    let loaded = store.load_current().expect("retried packed current");
    assert_eq!(
        loaded.objects.get(&second.objects[0].content_hash),
        Some(&second_bytes)
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn malformed_object_pack_index_fails_closed() {
    let root = test_root("pack-index-invalid");
    let store = SessionStore::new(&root);
    let publication = object_publication(0, None, patterned_bytes(4 * 1024, 0x61));
    let plan = publication.packing_plan().expect("packing plan");
    let mut files = plan.content.files;
    let pack_index = files
        .iter_mut()
        .find(|file| file.relative_path() == SESSION_OBJECT_PACK_INDEX_FILE)
        .expect("pack index");
    let mut invalid = pack_index.bytes().to_vec();
    invalid[0] ^= 0xff;
    *pack_index = PublicationFileV1::new(SESSION_OBJECT_PACK_INDEX_FILE, invalid)
        .expect("invalid indexed bytes are still a valid content file");
    let content =
        ContentPublicationV1::new(publication.generation_id, files).expect("content publication");
    store
        .content
        .publish(&content)
        .expect("physical publication");

    assert!(matches!(
        store.load_current(),
        Err(SessionStoreError::ObjectPackIndexInvalid)
    ));
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn unreferenced_object_pack_tail_fails_closed() {
    let root = test_root("pack-tail-invalid");
    let store = SessionStore::new(&root);
    let publication = object_publication(0, None, patterned_bytes(4 * 1024, 0x69));
    let plan = publication.packing_plan().expect("packing plan");
    let mut files = plan.content.files;
    let pack = files
        .iter_mut()
        .find(|file| file.relative_path() == object_pack_path(0))
        .expect("object pack");
    let mut bytes = pack.bytes().to_vec();
    bytes.push(0xff);
    *pack = PublicationFileV1::new(object_pack_path(0), bytes)
        .expect("tailed pack remains a valid content file");
    let content =
        ContentPublicationV1::new(publication.generation_id, files).expect("content publication");
    store
        .content
        .publish(&content)
        .expect("physical publication");

    assert!(matches!(
        store.load_current(),
        Err(SessionStoreError::ObjectPackIndexInvalid)
    ));
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn legacy_raw_object_generation_remains_readable() {
    let root = test_root("legacy-raw");
    let store = SessionStore::new(&root);
    let bytes = patterned_bytes(8 * 1024, 0x71);
    let publication = object_publication(0, None, bytes.clone());
    let object = &publication.objects[0];
    let content = ContentPublicationV1::new(
        publication.generation_id,
        vec![
            PublicationFileV1::new(
                SESSION_INDEX_FILE,
                publication.index_bytes().expect("index"),
            )
            .expect("index file"),
            PublicationFileV1::new(SESSION_SNAPSHOT_FILE, publication.snapshot.clone())
                .expect("snapshot file"),
            PublicationFileV1::new(
                format!(
                    "{SESSION_OBJECT_DIRECTORY}/{}.bin",
                    object.content_hash.to_hex()
                ),
                bytes.clone(),
            )
            .expect("raw object"),
        ],
    )
    .expect("legacy content publication");
    store.content.publish(&content).expect("legacy publication");

    let loaded = store.load_current().expect("legacy raw load");
    assert_eq!(loaded.objects.get(&object.content_hash), Some(&bytes));
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

fn object_publication(
    sequence: u64,
    previous: Option<ContentHash>,
    bytes: Vec<u8>,
) -> SessionPublicationV1 {
    SessionPublicationV1::new(
        sequence,
        ContentHash::from_bytes([9; 32]),
        Some(ApplicationSessionId::from_bytes([1; 16])),
        previous,
        None,
        vec![sequence as u8 + 1],
        vec![SessionObjectV1::new(bytes)],
    )
    .expect("object publication")
}

fn patterned_bytes(length: usize, seed: u8) -> Vec<u8> {
    let mut bytes = vec![seed; length];
    for (ordinal, byte) in bytes.iter_mut().enumerate().step_by(257) {
        *byte ^= ordinal as u8;
    }
    bytes
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
