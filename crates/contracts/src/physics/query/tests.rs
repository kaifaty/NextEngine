use super::*;
use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{CommandStreamId, PersistentId, PhysicsWorldId, content_hash_from_bytes};

fn request(slot: u32) -> PhysicsQueryRequestV1 {
    PhysicsQueryRequestV1 {
        schema_version: PHYSICS_QUERY_SCHEMA_VERSION,
        query_id: PhysicsQueryIdV1 {
            physics_tick: 7,
            query_slot: slot,
            issuer_stream_id: CommandStreamId::from_bytes([3; 16]),
        },
        world_id: PhysicsWorldId::from_bytes([4; 16]),
        snapshot_selector: PhysicsSnapshotSelectorV1 {
            physics_tick: 7,
            completed_substep: 0,
            physics_snapshot_hash: content_hash_from_bytes([5; 32]),
        },
        geometry: PhysicsQueryGeometryV1::ClosestPoint {
            point_micrometres: [1, 2, 3],
            maximum_distance_micrometres: 10_000,
        },
        filter: PhysicsQueryFilterV1 {
            query_collision_layer: 2,
            query_collision_mask: 1 << 4,
            include_solid: true,
            include_sensor: false,
            include_query_only: true,
            excluded_bodies: vec![PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([6; 16]),
                body_slot: 0,
            }],
            excluded_shapes: Vec::new(),
        },
        cardinality: PhysicsQueryCardinalityV1::Closest,
        maximum_published_hits: 1,
    }
}

fn hit(subject: u8, distance_micrometres: i64, fraction_q0_32: Option<u32>) -> PhysicsQueryHitV1 {
    PhysicsQueryHitV1 {
        distance_micrometres,
        shape_id: PhysicsShapeIdV1 {
            body_id: PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([subject; 16]),
                body_slot: 0,
            },
            shape_slot: 0,
        },
        feature_id: 1,
        point_micrometres: [0; 3],
        outward_normal_q1_30: [Q1_30_ONE, 0, 0],
        fraction_q0_32,
    }
}

fn all_result(
    request: &PhysicsQueryRequestV1,
    eligible_hit_count: u32,
    truncated: bool,
    hits: Vec<PhysicsQueryHitV1>,
) -> PhysicsQueryResultV1 {
    PhysicsQueryResultV1 {
        schema_version: PHYSICS_QUERY_RESULT_SCHEMA_VERSION,
        query_id: request.query_id,
        snapshot_selector: request.snapshot_selector,
        request_hash: request.request_hash().expect("request hash"),
        payload: PhysicsQueryResultPayloadV1::All {
            eligible_hit_count,
            truncated,
            hits,
        },
    }
}

#[test]
fn closest_point_request_and_batch_round_trip_exactly() {
    let first = request(1);
    let second = request(0);
    let batch = PhysicsQueryBatchV1::new(first.snapshot_selector, vec![first.clone(), second])
        .expect("valid batch");
    assert_eq!(batch.requests[0].query_id.query_slot, 0);

    let request_bytes = first.canonical_bytes().expect("request bytes");
    assert_eq!(
        PhysicsQueryRequestV1::from_canonical_bytes(
            &request_bytes,
            CanonicalDecodeLimits::default(),
        )
        .expect("request round trip"),
        first,
    );
    let batch_bytes = batch.canonical_bytes().expect("batch bytes");
    assert_eq!(
        PhysicsQueryBatchV1::from_canonical_bytes(&batch_bytes, CanonicalDecodeLimits::default(),)
            .expect("batch round trip"),
        batch,
    );
}

#[test]
fn query_validation_rejects_stale_tick_bad_cardinality_and_exclusion_order() {
    let mut stale = request(0);
    stale.snapshot_selector.physics_tick = 8;
    assert_eq!(stale.validate(), Err(PhysicsContractError::InvalidQuery));

    let mut bad_cardinality = request(0);
    bad_cardinality.maximum_published_hits = 0;
    assert_eq!(
        bad_cardinality.validate(),
        Err(PhysicsContractError::InvalidQuery)
    );

    let mut unordered = request(0);
    let high = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([9; 16]),
        body_slot: 0,
    };
    let low = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([1; 16]),
        body_slot: 0,
    };
    unordered.filter.excluded_bodies = vec![high, low];
    assert_eq!(
        unordered.validate(),
        Err(PhysicsContractError::NonCanonicalOrder)
    );
}

#[test]
fn query_result_rejects_wrong_request_binding_and_noncanonical_hits() {
    let request = PhysicsQueryRequestV1 {
        cardinality: PhysicsQueryCardinalityV1::All,
        maximum_published_hits: 2,
        ..request(0)
    };
    let body = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([7; 16]),
        body_slot: 0,
    };
    let hit = PhysicsQueryHitV1 {
        distance_micrometres: 5,
        shape_id: PhysicsShapeIdV1 {
            body_id: body,
            shape_slot: 0,
        },
        feature_id: 1,
        point_micrometres: [0; 3],
        outward_normal_q1_30: [Q1_30_ONE, 0, 0],
        fraction_q0_32: None,
    };
    let result = PhysicsQueryResultV1 {
        schema_version: PHYSICS_QUERY_RESULT_SCHEMA_VERSION,
        query_id: request.query_id,
        snapshot_selector: request.snapshot_selector,
        request_hash: request.request_hash().expect("request hash"),
        payload: PhysicsQueryResultPayloadV1::All {
            eligible_hit_count: 1,
            truncated: false,
            hits: vec![hit.clone()],
        },
    };
    result
        .validate_against_request(&request)
        .expect("valid result");
    let bytes = result.canonical_bytes().expect("result bytes");
    assert_eq!(
        PhysicsQueryResultV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("result round trip"),
        result,
    );

    let mut wrong_hash = result.clone();
    wrong_hash.request_hash = content_hash_from_bytes([0; 32]);
    assert_eq!(
        wrong_hash.validate_against_request(&request),
        Err(PhysicsContractError::InvalidQuery)
    );

    let mut duplicate = result;
    duplicate.payload = PhysicsQueryResultPayloadV1::All {
        eligible_hit_count: 2,
        truncated: false,
        hits: vec![hit.clone(), hit],
    };
    assert_eq!(
        duplicate.validate_against_request(&request),
        Err(PhysicsContractError::InvalidQuery)
    );
}

#[test]
fn all_result_requires_the_complete_published_prefix() {
    let request = PhysicsQueryRequestV1 {
        cardinality: PhysicsQueryCardinalityV1::All,
        maximum_published_hits: 2,
        ..request(0)
    };
    let first = hit(1, 5, None);
    let second = hit(2, 6, None);

    assert_eq!(
        all_result(&request, 1, true, Vec::new()).validate_against_request(&request),
        Err(PhysicsContractError::InvalidQuery)
    );
    assert_eq!(
        all_result(&request, 3, true, vec![first.clone()]).validate_against_request(&request),
        Err(PhysicsContractError::InvalidQuery)
    );
    all_result(&request, 3, true, vec![first, second])
        .validate_against_request(&request)
        .expect("the first max-published hits form the complete published prefix");
}

#[test]
fn ray_and_shape_cast_hits_order_by_fraction_before_distance() {
    let request = PhysicsQueryRequestV1 {
        geometry: PhysicsQueryGeometryV1::RayCast {
            origin_micrometres: [0; 3],
            unit_direction_q1_30: [Q1_30_ONE, 0, 0],
            maximum_distance_micrometres: 10_000,
        },
        cardinality: PhysicsQueryCardinalityV1::All,
        maximum_published_hits: 2,
        ..request(0)
    };
    let earlier_fraction = hit(2, 9_000, Some(1));
    let later_fraction = hit(1, 1, Some(2));

    all_result(
        &request,
        2,
        false,
        vec![earlier_fraction.clone(), later_fraction.clone()],
    )
    .validate_against_request(&request)
    .expect("ray hits use fraction as the leading canonical key");
    assert_eq!(
        all_result(&request, 2, false, vec![later_fraction, earlier_fraction])
            .validate_against_request(&request),
        Err(PhysicsContractError::InvalidQuery)
    );

    assert_eq!(
        hit(2, 9_000, Some(1))
            .cmp_canonical_for_kind(&hit(1, 1, Some(2)), PhysicsQueryKindV1::ShapeCast,),
        std::cmp::Ordering::Less
    );
}

#[test]
fn overlap_hits_order_by_identity_without_numeric_tie_breakers() {
    let request = PhysicsQueryRequestV1 {
        geometry: PhysicsQueryGeometryV1::Overlap {
            query_shape: PhysicsQueryShapeV1::Primitive(PhysicsGeometryV1::Sphere {
                radius_micrometres: 10,
            }),
            pose: PhysicsPoseV1::default(),
        },
        cardinality: PhysicsQueryCardinalityV1::All,
        maximum_published_hits: 2,
        ..request(0)
    };
    let lower_identity = hit(1, 9_000, None);
    let higher_identity = hit(2, 1, None);

    all_result(
        &request,
        2,
        false,
        vec![lower_identity.clone(), higher_identity.clone()],
    )
    .validate_against_request(&request)
    .expect("overlap hits use stable identity as the canonical key");
    assert_eq!(
        all_result(&request, 2, false, vec![higher_identity, lower_identity])
            .validate_against_request(&request),
        Err(PhysicsContractError::InvalidQuery)
    );
}

#[test]
fn query_result_decoder_rejects_unknown_schema_version() {
    let request = PhysicsQueryRequestV1 {
        cardinality: PhysicsQueryCardinalityV1::All,
        maximum_published_hits: 1,
        ..request(0)
    };
    let mut result = all_result(&request, 0, false, Vec::new());
    result.schema_version = PHYSICS_QUERY_RESULT_SCHEMA_VERSION + 1;
    let bytes = result.canonical_bytes().expect("encoded unknown version");
    assert!(matches!(
        PhysicsQueryResultV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default()),
        Err(PhysicsContractError::UnsupportedVersion(_))
    ));
}
