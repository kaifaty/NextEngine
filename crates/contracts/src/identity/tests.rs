use crate::canonical::CanonicalDecodeLimits;
use crate::command::IssuerPrincipal;
use crate::ids::{ContentHash, PlayerPrincipalId, ProjectId, SchemaId, WorldNamespaceId};

use super::*;

fn hash(byte: u8) -> ContentHash {
    ContentHash::from_bytes([byte; 32])
}

#[test]
fn world_player_and_stream_ids_are_stable_and_body_sensitive() {
    let project = ProjectId::new("nextengine.fixture").expect("fixture id");
    let world =
        WorldIdentityManifestV1::new(project, [1; 32], [2; 32], hash(3)).expect("world identity");
    assert_eq!(
        world.world_namespace.to_hex(),
        "b4911c2f5b2b7cc4371e28a522408982"
    );
    assert_eq!(
        derive_player_principal_id(world.world_namespace, 0).to_hex(),
        "7fd6241b679a81c7a1407665aa40f340"
    );
    let principal = IssuerPrincipal::Player(derive_player_principal_id(world.world_namespace, 0));
    let stream =
        derive_command_stream_id(world.world_namespace, &principal, 0, 0).expect("stream id");
    assert_eq!(stream.to_hex(), "d23d8b2c25c0744daab76ef54bd38038");
    assert_ne!(
        stream,
        derive_command_stream_id(world.world_namespace, &principal, 1, 0).expect("second stream")
    );
}

#[test]
fn identity_contracts_round_trip_and_reject_v1_drift() {
    let profile = RuntimeDeterminismBundleV1::core_r4c()
        .expect("determinism bundle")
        .runtime_profile();
    let profile_bytes = profile.canonical_bytes().expect("profile bytes");
    assert_eq!(
        RuntimeDeterminismProfileV1::from_canonical_bytes(
            &profile_bytes,
            CanonicalDecodeLimits::default()
        )
        .expect("profile round trip"),
        profile
    );
    let world = WorldIdentityManifestV1::new(
        ProjectId::new("nextengine.fixture").expect("fixture id"),
        [1; 32],
        [2; 32],
        profile.profile_hash().expect("profile hash"),
    )
    .expect("world");
    let world_bytes = world.canonical_bytes().expect("world bytes");
    assert_eq!(
        WorldIdentityManifestV1::from_canonical_bytes(
            &world_bytes,
            CanonicalDecodeLimits::default()
        )
        .expect("world round trip"),
        world
    );
}

#[test]
fn registries_are_order_independent_and_allocate_monotonic_slots() {
    let namespace = WorldNamespaceId::from_bytes([4; 16]);
    let first = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([1; 16]));
    let second = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16]));
    let record = |byte| PrincipalRecordV1 {
        provenance_hash: hash(byte),
        capability_subject_id: SchemaId::new(format!("fixture.subject.{byte}"))
            .expect("subject id"),
        status: PrincipalStatus::Active,
    };
    let mut left = PrincipalRegistryV1::empty(namespace);
    left.register(first.clone(), record(1)).expect("first");
    left.register(second.clone(), record(2)).expect("second");
    let mut right = PrincipalRegistryV1::empty(namespace);
    right.register(second.clone(), record(2)).expect("second");
    right.register(first.clone(), record(1)).expect("first");
    assert_eq!(
        left.canonical_bytes().expect("left"),
        right.canonical_bytes().expect("right")
    );

    let mut streams = CommandStreamRegistryV1::empty(namespace);
    let stream0 = streams.allocate_stream(first.clone()).expect("slot zero");
    let stream1 = streams.allocate_stream(first).expect("slot one");
    assert_ne!(stream0, stream1);
    streams.validate().expect("registry closure");
    let bytes = streams.canonical_bytes().expect("stream bytes");
    assert_eq!(
        CommandStreamRegistryV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("stream round trip"),
        streams
    );
}
