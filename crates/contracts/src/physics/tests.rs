use std::collections::BTreeMap;

use crate::canonical::{CanonicalDecodeLimits, encode_canonical_segment};
use crate::{ContentHash, PhysicsWorldId, SchemaId, TickRateProfileV1, content_hash_from_bytes};

use super::codec::field_u16;
use super::*;

fn hash(byte: u8) -> ContentHash {
    content_hash_from_bytes([byte; 32])
}

#[test]
fn physical_command_accepts_only_axial_q15() {
    PhysicalCommandV1::SetCapsuleLocomotionIntent {
        direction_q15: [0, 32_767],
    }
    .validate()
    .expect("forward is valid");
    assert_eq!(
        PhysicalCommandV1::SetCapsuleLocomotionIntent {
            direction_q15: [32_767, 32_767],
        }
        .validate(),
        Err(PhysicsContractError::DirectionOutOfProfile)
    );
}

#[test]
fn profiles_and_snapshot_round_trip() {
    let tick = TickRateProfileV1::at_30_hz();
    let quantization = PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
    let numeric =
        AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
    numeric.validate().expect("numeric valid");
    let catalog = PhysicsWorldCatalogV1::new(
        PhysicsWorldId::from_bytes([1; 16]),
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("coordinate"),
            limits: PhysicsLimitsProfileV1::reference_v1().expect("limits"),
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("solver"),
            tick_rate_hash: tick.profile_hash().expect("tick hash"),
            authoritative_numeric_hash: numeric.profile_hash().expect("numeric hash"),
            quantization_hash: quantization.profile_hash().expect("quantization hash"),
        },
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .expect("catalog");
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(&catalog, &tick, &numeric, &quantization)
        .expect("snapshot");
    snapshot
        .validate_profile_closure(&catalog, &tick, &numeric, &quantization)
        .expect("profile closure");
    let bytes = snapshot.canonical_bytes().expect("snapshot encode");
    assert_eq!(
        PhysicsCanonicalSnapshotV2::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("snapshot decode"),
        snapshot
    );
    let mut corrupt_profile = snapshot;
    corrupt_profile.tick_rate_profile_hash = hash(10);
    assert_eq!(
        corrupt_profile.validate_profile_closure(&catalog, &tick, &numeric, &quantization),
        Err(PhysicsContractError::ProfileMismatch)
    );
}

#[test]
fn grounded_capsule_v2_quantization_is_structured_and_fail_closed() {
    let profile = PhysicsQuantizationProfileV1::grounded_capsule_v2().expect("v2 profile");
    profile.validate().expect("v2 profile valid");
    let bytes = profile.canonical_bytes().expect("encode v2 profile");
    assert_eq!(
        PhysicsQuantizationProfileV1::from_canonical_bytes(
            &bytes,
            CanonicalDecodeLimits::default()
        )
        .expect("decode v2 profile"),
        profile
    );
    assert_ne!(
        profile.profile_hash().expect("v2 hash"),
        PhysicsQuantizationProfileV1::capsule_reference_v1()
            .expect("v1 profile")
            .profile_hash()
            .expect("v1 hash")
    );

    let mut invalid_rule = profile.clone();
    invalid_rule
        .rules
        .get_mut(&SchemaId::new(PHYSICS_SWEEP_DISTANCE_FIELD_ID).expect("distance field ID"))
        .expect("distance rule")
        .scale_denominator = 0;
    assert_eq!(
        invalid_rule.validate(),
        Err(PhysicsContractError::InvalidProfile)
    );

    let mut mismatched_key = profile;
    let distance_id = SchemaId::new(PHYSICS_SWEEP_DISTANCE_FIELD_ID).expect("distance field ID");
    let distance = mismatched_key
        .rules
        .remove(&distance_id)
        .expect("distance rule");
    mismatched_key.rules.insert(
        SchemaId::new("nextengine.physics.raw.wrong-key").expect("wrong key"),
        distance,
    );
    assert_eq!(
        mismatched_key.validate(),
        Err(PhysicsContractError::DuplicateKey)
    );
}

#[test]
fn legacy_physics_snapshot_v1_is_rejected_from_header_only() {
    let bytes = encode_canonical_segment(
        PHYSICS_SNAPSHOT_OWNER_ID,
        PHYSICS_SNAPSHOT_SCHEMA_ID,
        LEGACY_PHYSICS_SNAPSHOT_SEGMENT_ID,
        [field_u16(1, LEGACY_PHYSICS_SNAPSHOT_SCHEMA_VERSION)],
    )
    .expect("legacy header");
    assert_eq!(
        PhysicsCanonicalSnapshotV2::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default()),
        Err(PhysicsContractError::UnsupportedVersion(1))
    );
}
