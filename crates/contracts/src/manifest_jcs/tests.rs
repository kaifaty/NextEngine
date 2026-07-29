use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{SchemaId, WorldNamespaceId, content_hash_from_bytes};
use crate::persistence::{CommandLedgerDescriptorV2, ManifestCodecError};
use crate::persistence::{SaveCompatibility, SaveManifestV2, SaveSegmentDescriptor, TickSettings};

fn manifest() -> SaveManifestV2 {
    SaveManifestV2 {
        schema_version: crate::persistence::SAVE_MANIFEST_SCHEMA_VERSION,
        generation: u64::MAX,
        world_revision: 7,
        compatibility: SaveCompatibility {
            engine_build_hash: content_hash_from_bytes([1; 32]),
            game_build_hash: content_hash_from_bytes([2; 32]),
            project_id: SchemaId::new("nextengine.test").expect("valid project"),
            schema_registry_hash: content_hash_from_bytes([3; 32]),
            content_manifest_hash: content_hash_from_bytes([4; 32]),
            mechanics_lock_hash: content_hash_from_bytes([5; 32]),
            tick_settings: TickSettings {
                gameplay_hz: 30,
                physics_hz: 120,
                motor_hz: 60,
            },
            loaded_chunk_revisions: vec![],
            rng_stream_states: vec![],
            physical_bindings: vec![],
            policy_state_schemas: vec![],
            plugin_script_bindings: vec![],
        },
        command_ledger: CommandLedgerDescriptorV2 {
            world_namespace: WorldNamespaceId::from_bytes([6; 16]),
            stream_count: 0,
            archive_root: content_hash_from_bytes([7; 32]),
            identity_index_root: content_hash_from_bytes([8; 32]),
            runtime_snapshot_segment_hash: content_hash_from_bytes([9; 32]),
        },
        segments: vec![
            SaveSegmentDescriptor::for_bytes(
                SchemaId::new(crate::snapshot::RUNTIME_SNAPSHOT_OWNER_ID).expect("valid owner"),
                SchemaId::new(crate::snapshot::RUNTIME_SNAPSHOT_SCHEMA_ID).expect("valid schema"),
                SchemaId::new(crate::snapshot::RUNTIME_SNAPSHOT_SEGMENT_ID).expect("valid segment"),
                crate::snapshot::RUNTIME_SNAPSHOT_SCHEMA_VERSION,
                b"snapshot",
            )
            .expect("valid segment"),
        ],
    }
}

#[test]
fn save_manifest_jcs_round_trip_is_byte_exact() {
    let manifest = manifest();
    let bytes = manifest.to_jcs_bytes().expect("manifest encodes");
    let decoded = SaveManifestV2::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
        .expect("manifest decodes");
    assert_eq!(decoded, manifest);
    assert_eq!(decoded.to_jcs_bytes().expect("manifest re-encodes"), bytes);
}

#[test]
fn noncanonical_whitespace_and_duplicate_keys_are_rejected() {
    let bytes = manifest().to_jcs_bytes().expect("manifest encodes");
    let mut whitespace = bytes.clone();
    whitespace.insert(1, b' ');
    assert!(SaveManifestV2::from_jcs_bytes(&whitespace, CanonicalDecodeLimits::default()).is_err());

    assert!(matches!(
        super::jcs::Parser::new(br#"{"a":1,"a":2}"#, 10).parse_value(0),
        Err(ManifestCodecError::DuplicateObjectKey(key)) if key == "a"
    ));
}
