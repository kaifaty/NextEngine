use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_U32, CanonicalDecodeLimits, CanonicalField, encode_canonical_segment,
};
use crate::command::IssuerPrincipal;
use crate::identity::{
    CommandStreamRegistryV1, PrincipalRecordV1, PrincipalRegistryV1, PrincipalStatus,
    RuntimeDeterminismBundleV1, WorldIdentityManifestV1,
};
use crate::ids::{ContentHash, PlayerPrincipalId, ProjectId, SchemaId};
use crate::input::{
    IngressAssignmentProfileV1, IngressCheckpointV1, PlayerControllerRegistryV1,
    RuntimeAdmissionLimitsV1, TickRateProfileV1,
};
use crate::ledger::{
    CausalIdentityKey, CausalIdentityKind, CommandLedgerV2, CommandStreamLedgerV2,
};
use crate::physics::{AuthoritativeNumericProfileV1, PhysicsQuantizationProfileV1};
use crate::rpg::RpgRuntimeBindingsV1;

use super::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SEGMENT_ID,
    RuntimeSnapshotV3, SnapshotDecodeError,
};

fn fixture() -> RuntimeSnapshotV3 {
    let bundle = RuntimeDeterminismBundleV1::core_r4a().expect("determinism bundle");
    let command_hash = bundle.command_kind_registry_hash();
    let profile = bundle.runtime_profile();
    let world = WorldIdentityManifestV1::new(
        ProjectId::new("nextengine.snapshot-fixture").expect("project id"),
        [1; 32],
        [2; 32],
        profile.profile_hash().expect("profile hash"),
    )
    .expect("world identity");
    let principal = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([3; 16]));
    let mut principals = PrincipalRegistryV1::empty(world.world_namespace);
    principals
        .register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash: ContentHash::from_bytes([4; 32]),
                capability_subject_id: SchemaId::new("fixture.player").expect("subject"),
                status: PrincipalStatus::Active,
            },
        )
        .expect("principal");
    let mut streams = CommandStreamRegistryV1::empty(world.world_namespace);
    let stream_id = streams
        .allocate_stream(principal.clone())
        .expect("stream allocation");
    let (mut ledger, archive) = CommandLedgerV2::empty(
        world.world_namespace,
        command_hash,
        profile.profile_hash().expect("profile hash"),
    )
    .expect("empty ledger");
    ledger.streams.insert(
        stream_id,
        CommandStreamLedgerV2::genesis(stream_id, principal.clone(), 0, 0),
    );
    ledger
        .causal_identity_registry
        .compare_or_insert(
            CausalIdentityKey {
                identity_kind: CausalIdentityKind::PlayerPrincipal,
                identity_bytes: match &principal {
                    IssuerPrincipal::Player(id) => *id.as_bytes(),
                    _ => unreachable!("fixture principal is a player"),
                },
            },
            ContentHash::from_bytes([4; 32]),
        )
        .expect("player provenance");
    let principal_bytes = principal.canonical_bytes().expect("principal bytes");
    let mut provenance = Vec::new();
    provenance.extend_from_slice(world.world_namespace.as_bytes());
    provenance.extend_from_slice(&(principal_bytes.len() as u64).to_le_bytes());
    provenance.extend_from_slice(&principal_bytes);
    provenance.extend_from_slice(&0_u32.to_le_bytes());
    provenance.extend_from_slice(&0_u32.to_le_bytes());
    ledger
        .causal_identity_registry
        .compare_or_insert_provenance(
            CausalIdentityKind::CommandStream,
            *stream_id.as_bytes(),
            &provenance,
        )
        .expect("stream provenance");
    let admission_limits = RuntimeAdmissionLimitsV1::default();
    let tick_rate_profile = TickRateProfileV1::at_30_hz();
    let ingress_assignment_profile =
        IngressAssignmentProfileV1::core_v1(&admission_limits).expect("ingress profile");
    let physics_quantization_profile =
        PhysicsQuantizationProfileV1::capsule_reference_v1().expect("physics profile");
    let authoritative_numeric_profile =
        AuthoritativeNumericProfileV1::capsule_reference_v1(&physics_quantization_profile)
            .expect("numeric profile");
    let world_namespace = world.world_namespace;
    RuntimeSnapshotV3 {
        next_tick: 5,
        committed_event_count: 0,
        authoritative_revision: 3,
        world_identity: world,
        principal_registry: principals,
        stream_registry: streams,
        runtime_profile: profile,
        admission_limits,
        tick_rate_profile,
        ingress_assignment_profile,
        authoritative_numeric_profile,
        physics_quantization_profile,
        player_controller_registry: PlayerControllerRegistryV1 {
            schema_version: 1,
            world_namespace,
            bindings: BTreeMap::new(),
        },
        ingress_checkpoint: IngressCheckpointV1 {
            schema_version: 1,
            current_tick: 5,
            current_generation: 5,
            current_samples: Vec::new(),
            next_samples: Vec::new(),
            last_closed_batch_hash: None,
        },
        rpg_runtime_bindings: RpgRuntimeBindingsV1 {
            project_composition_lock_hash: ContentHash::from_bytes([21; 32]),
            schema_registry_hash: ContentHash::from_bytes([22; 32]),
            budget_policy_hash: ContentHash::from_bytes([23; 32]),
            active_definition_policy_hashes: vec![ContentHash::from_bytes([24; 32])],
        },
        command_ledger: ledger,
        body_archive: archive,
    }
}

#[test]
fn v3_snapshot_round_trip_is_byte_exact() {
    let snapshot = fixture();
    let bytes = snapshot.canonical_bytes().expect("snapshot bytes");
    let decoded = RuntimeSnapshotV3::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
        .expect("snapshot decodes");
    assert_eq!(decoded, snapshot);
    assert_eq!(decoded.canonical_bytes().expect("decoded bytes"), bytes);
}

#[test]
fn v3_snapshot_rejects_ingress_controller_and_profile_closure_corruption() {
    let mut corrupt_ingress = fixture();
    corrupt_ingress.ingress_checkpoint.current_tick -= 1;
    assert_eq!(
        corrupt_ingress.validate(),
        Err(SnapshotDecodeError::ProfileClosureMismatch)
    );

    let mut corrupt_controller = fixture();
    corrupt_controller
        .player_controller_registry
        .world_namespace = crate::ids::WorldNamespaceId::from_bytes([0xff; 16]);
    assert_eq!(
        corrupt_controller.validate(),
        Err(SnapshotDecodeError::ClosureMismatch)
    );

    let mut corrupt_profile = fixture();
    corrupt_profile
        .tick_rate_profile
        .physics_substeps_per_gameplay_tick = 4;
    assert_eq!(
        corrupt_profile.validate(),
        Err(SnapshotDecodeError::ProfileClosureMismatch)
    );
}

#[test]
fn v1_is_rejected_before_nested_state_decoding() {
    let bytes = encode_canonical_segment(
        RUNTIME_SNAPSHOT_OWNER_ID,
        RUNTIME_SNAPSHOT_SCHEMA_ID,
        RUNTIME_SNAPSHOT_SEGMENT_ID,
        [CanonicalField::new(
            1,
            CANONICAL_TYPE_U32,
            1_u32.to_le_bytes().to_vec(),
        )],
    )
    .expect("legacy-shaped bytes");
    let error = RuntimeSnapshotV3::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
        .expect_err("V1 must be rejected");
    assert_eq!(error.stable_code(), "UNSUPPORTED_RUNTIME_SNAPSHOT_VERSION");
}
