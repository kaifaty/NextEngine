use std::collections::BTreeMap;

use crate::canonical::{CanonicalDecodeLimits, encode_canonical_segment};
use crate::ids::{ContentHash, PersistentId, PhysicsWorldId, SchemaId, content_hash_from_bytes};
use crate::input::TickRateProfileV1;

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
fn grounded_capsule_r5b_profile_constants_are_exact() {
    assert_eq!(CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND, 3_000_000);
    assert_eq!(CAPSULE_MAX_STEP_HEIGHT_MICROMETRES, 300_000);
    assert_eq!(CAPSULE_GROUND_SNAP_DISTANCE_MICROMETRES, 300_000);
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

#[test]
fn complete_material_and_combine_contracts_round_trip_and_hash_every_field() {
    let material = PhysicsMaterialDescriptorV2 {
        schema_version: PHYSICS_MATERIAL_DESCRIPTOR_V2_SCHEMA_VERSION,
        base: PhysicsMaterialDescriptorV1 {
            material_id: SchemaId::new("physics-material.humanoid-body.v1").expect("material ID"),
            descriptor_revision: 1,
            static_friction_q16: 52_429,
            dynamic_friction_q16: 45_875,
            restitution_q16: 0,
            canonical_material_tags: Vec::new(),
        },
        rolling_friction_q16: 0,
        spinning_friction_q16: 0,
        surface_velocity_micrometres_per_second: [0; 3],
    };
    material.validate().expect("complete material");
    let bytes = material.canonical_bytes().expect("material bytes");
    assert_eq!(
        PhysicsMaterialDescriptorV2::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("material round trip"),
        material
    );
    let mut changed = material.clone();
    changed.spinning_friction_q16 = 1;
    assert_ne!(
        material.descriptor_hash().expect("material hash"),
        changed.descriptor_hash().expect("changed material hash")
    );

    let combine = PhysicsMaterialCombineProfileV1 {
        schema_version: PHYSICS_MATERIAL_COMBINE_PROFILE_V1_SCHEMA_VERSION,
        profile_id: SchemaId::new("nextengine.physics-material-combine.humanoid-motor.v1")
            .expect("combine ID"),
        profile_revision: 1,
        static_friction: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        dynamic_friction: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        restitution: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        rolling_friction: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        spinning_friction: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        surface_velocity: PhysicsSurfaceVelocityCombineRuleV1::CanonicalParticipantOrder,
    };
    combine.validate().expect("combine profile");
    let bytes = combine.canonical_bytes().expect("combine bytes");
    assert_eq!(
        PhysicsMaterialCombineProfileV1::from_canonical_bytes(
            &bytes,
            CanonicalDecodeLimits::default()
        )
        .expect("combine round trip"),
        combine
    );
    let mut changed = combine.clone();
    changed.dynamic_friction = PhysicsMaterialCombineRuleV1::Minimum;
    assert_ne!(
        combine.profile_hash().expect("combine hash"),
        changed.profile_hash().expect("changed combine hash")
    );
}

#[test]
fn water_volume_set_round_trips_queries_and_commands() {
    use super::{
        WaterLevelRampV1, WaterSubmersionClassV1, WaterVolumeCommandV1, WaterVolumeDefinitionV1,
        WaterVolumeRejectionV1, WaterVolumeSetV1,
    };
    use crate::canonical::CanonicalDecodeLimits;
    use crate::ids::PersistentId;

    let basin = WaterVolumeDefinitionV1 {
        volume_id: PersistentId::from_bytes([0x21; 16]),
        minimum_micrometres: [0, 0, 0],
        maximum_micrometres: [4_000_000, 2_000_000, 2_000_000],
        initial_level_micrometres: 500_000,
        swimming_depth_micrometres: 1_200_000,
        level_ramp: Some(WaterLevelRampV1 {
            start_tick: 10,
            end_tick: 20,
            start_level_micrometres: 500_000,
            end_level_micrometres: 1_500_000,
        }),
        profile_revision: 1,
    };
    let set = WaterVolumeSetV1::from_definitions([basin.clone()]).expect("valid basin");
    let bytes = set.canonical_record().expect("encode");
    let decoded =
        WaterVolumeSetV1::from_record(&bytes, CanonicalDecodeLimits::default()).expect("decode");
    assert_eq!(decoded, set);
    assert_eq!(
        set.set_hash().expect("hash"),
        decoded.set_hash().expect("hash")
    );

    // Ramp: exact integer interpolation, clamped outside the window.
    assert_eq!(set.effective_level(basin.volume_id, 0), Some(500_000));
    assert_eq!(set.effective_level(basin.volume_id, 15), Some(1_000_000));
    assert_eq!(set.effective_level(basin.volume_id, 99), Some(1_500_000));
    let probe = set.submersion_at([1_000_000, 0, 1_000_000], 15);
    assert_eq!(probe.depth_micrometres, 1_000_000);
    assert_eq!(probe.class, WaterSubmersionClassV1::Wading);
    assert_eq!(
        set.submersion_at([9_000_000, 0, 0], 0).class,
        WaterSubmersionClassV1::Dry
    );

    // A committed command suspends the ramp and bumps the record revision.
    let (next, event) = set
        .apply_command(
            &WaterVolumeCommandV1::SetLevel {
                volume_id: basin.volume_id,
                expected_record_revision: 0,
                level_micrometres: 1_800_000,
            },
            15,
        )
        .expect("commit");
    assert_eq!(event.previous_level_micrometres, 1_000_000);
    assert_eq!(event.record_revision, 1);
    assert_eq!(next.effective_level(basin.volume_id, 15), Some(1_800_000));
    assert_eq!(
        next.submersion_at([1_000_000, 0, 1_000_000], 15).class,
        WaterSubmersionClassV1::Swimming
    );
    assert_eq!(
        next.apply_command(
            &WaterVolumeCommandV1::SetLevel {
                volume_id: basin.volume_id,
                expected_record_revision: 0,
                level_micrometres: 1_000_000,
            },
            16,
        )
        .expect_err("stale"),
        WaterVolumeRejectionV1::RevisionStale
    );
    assert_eq!(
        next.apply_command(
            &WaterVolumeCommandV1::SetLevel {
                volume_id: basin.volume_id,
                expected_record_revision: 1,
                level_micrometres: 2_000_001,
            },
            16,
        )
        .expect_err("out of extent"),
        WaterVolumeRejectionV1::LevelOutOfExtent
    );

    // Overlapping volumes reject; command and event payloads round-trip.
    let mut overlapping = basin.clone();
    overlapping.volume_id = PersistentId::from_bytes([0x22; 16]);
    assert!(WaterVolumeSetV1::from_definitions([basin.clone(), overlapping]).is_err());
    let command = WaterVolumeCommandV1::SetLevel {
        volume_id: basin.volume_id,
        expected_record_revision: 3,
        level_micrometres: -7,
    };
    let command_bytes = command.canonical_payload_bytes().expect("command bytes");
    assert_eq!(
        WaterVolumeCommandV1::from_canonical_payload_bytes(
            &command_bytes,
            CanonicalDecodeLimits::default()
        )
        .expect("command decode"),
        command
    );
    let event_bytes = event.canonical_payload_bytes().expect("event bytes");
    assert_eq!(
        super::WaterVolumeChangedV1::from_canonical_payload_bytes(&event_bytes).expect("event"),
        event
    );
}

#[test]
fn body_descriptor_mass_follows_the_motion_kind_and_round_trips() {
    let tick = TickRateProfileV1::at_30_hz();
    let quantization = PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
    let numeric =
        AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
    let material_id = SchemaId::new("nextengine.physics.material.test-zero").expect("material id");
    let material = PhysicsMaterialDescriptorV1 {
        material_id: material_id.clone(),
        descriptor_revision: 1,
        static_friction_q16: 0,
        dynamic_friction_q16: 0,
        restitution_q16: 0,
        canonical_material_tags: Vec::new(),
    };
    let body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x31; 16]),
        body_slot: 0,
    };
    let shape_id = PhysicsShapeIdV1 {
        body_id,
        shape_slot: 0,
    };
    let shape = PhysicsShapeDescriptorV1 {
        shape_id,
        descriptor_revision: 1,
        local_pose: PhysicsPoseV1::default(),
        geometry: PhysicsGeometryV1::Box {
            half_extents_micrometres: [200_000, 300_000, 200_000],
        },
        material_id,
        collision_layer: 1,
        collision_mask: u64::MAX,
        participation: PhysicsParticipationV1::Solid,
        contact_reporting: PhysicsContactReportingV1::Disabled,
    };
    let body = |motion_kind, mass_microkilograms| PhysicsBodyDescriptorV1 {
        body_id,
        descriptor_revision: 1,
        motion_kind,
        initial_pose: PhysicsPoseV1 {
            translation_micrometres: [0, 300_000, 0],
            ..PhysicsPoseV1::default()
        },
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        mass_microkilograms,
        shapes: BTreeMap::from([(shape_id, shape.clone())]),
    };
    for (motion_kind, mass) in [
        (PhysicsMotionKindV1::Dynamic, 0),
        (
            PhysicsMotionKindV1::Dynamic,
            MAXIMUM_BODY_MASS_MICROKILOGRAMS + 1,
        ),
        (PhysicsMotionKindV1::Static, 1),
        (PhysicsMotionKindV1::Kinematic, 20_000_000),
    ] {
        assert_eq!(
            body(motion_kind, mass).validate(),
            Err(PhysicsContractError::InvalidDescriptor)
        );
    }
    let dynamic = body(PhysicsMotionKindV1::Dynamic, 20_000_000);
    dynamic.validate().expect("20 kg dynamic body");
    let catalog = PhysicsWorldCatalogV1::new(
        PhysicsWorldId::from_bytes([2; 16]),
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("coordinate"),
            limits: PhysicsLimitsProfileV1::reference_v1().expect("limits"),
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("solver"),
            tick_rate_hash: tick.profile_hash().expect("tick hash"),
            authoritative_numeric_hash: numeric.profile_hash().expect("numeric hash"),
            quantization_hash: quantization.profile_hash().expect("quantization hash"),
        },
        BTreeMap::from([(material.material_id.clone(), material)]),
        BTreeMap::from([(body_id, dynamic)]),
        BTreeMap::new(),
    )
    .expect("catalog with one dynamic body");
    let bytes = catalog.canonical_bytes().expect("catalog encode");
    let decoded =
        PhysicsWorldCatalogV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("catalog decode");
    assert_eq!(decoded, catalog);
    assert_eq!(decoded.bodies[&body_id].mass_microkilograms, 20_000_000);
}
