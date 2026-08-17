#![forbid(unsafe_code)]

use super::*;
use crate::boundary;
use crate::model::{Box3i, StorageOrder};
use crate::scenario;

#[test]
fn convergence_thresholds_are_inclusive_and_minimum_iterations_hold() {
    assert!(!converged(1, 2, 99_999, 100_000));
    assert!(converged(2, 2, 99_999, 100_000));
    assert!(converged(2, 2, 100_000, 100_000));
    assert!(!converged(20, 2, 100_001, 100_000));
    assert!(converged(1, 1, 999_999, 1_000_000));
    assert!(converged(1, 1, 1_000_000, 1_000_000));
    assert!(!converged(20, 1, 1_000_001, 1_000_000));
}

#[test]
fn fluid_row_capacity_accepts_n_minus_one_and_n() {
    for value in [
        MAXIMUM_NEIGHBORS_PER_FLUID_ROW - 1,
        MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
    ] {
        assert!(
            validate_capacity(
                value,
                MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
                NEIGHBOR_CAPACITY_EXCEEDED,
                "row"
            )
            .is_ok()
        );
    }
    assert_eq!(
        validate_capacity(
            MAXIMUM_NEIGHBORS_PER_FLUID_ROW + 1,
            MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
            NEIGHBOR_CAPACITY_EXCEEDED,
            "row"
        )
        .unwrap_err()
        .code(),
        NEIGHBOR_CAPACITY_EXCEEDED
    );
}

#[test]
fn zero_and_one_particle_frames_are_well_defined() {
    let geometry = Geometry {
        bounds: Box3i {
            min: Vec3i::new(-100_000, -100_000, -100_000),
            max: Vec3i::new(100_000, 100_000, 100_000),
        },
        aperture: None,
    };
    let profile = [1_u8; 32];
    let scenario = [2_u8; 32];
    let (empty, _) = initial_frame(Vec::new(), geometry, &[], &profile, &scenario).unwrap();
    let empty_next = substep(&empty, geometry, &[], &profile, &scenario).unwrap();
    assert!(empty_next.frame.samples.is_empty());

    let singleton = CanonicalSample {
        id: 42,
        position_um: Vec3i::new(0, 0, 0),
        velocity_um_s: Vec3i::new(0, 0, 0),
    };
    let (one, _) = initial_frame(vec![singleton], geometry, &[], &profile, &scenario).unwrap();
    let one_next = substep(&one, geometry, &[], &profile, &scenario).unwrap();
    assert_eq!(one_next.frame.samples.len(), 1);
    assert_eq!(one_next.summary.divergence_iterations, 1);
    assert_eq!(one_next.summary.density_iterations, 2);
}

#[test]
fn smoke_storage_orders_publish_the_same_frame() {
    let scenario = scenario::find("SMOKE-CW-ORDER-001").unwrap();
    let boundary = boundary::build(scenario.geometry).unwrap();
    let profile = [3_u8; 32];
    let scenario_root = [4_u8; 32];
    let mut roots = Vec::new();
    for order in [
        StorageOrder::Identity,
        StorageOrder::Reverse,
        StorageOrder::Affine,
    ] {
        let samples = scenario::initial_samples(&scenario, order).unwrap();
        let (initial, _) = initial_frame(
            samples,
            scenario.geometry,
            &boundary,
            &profile,
            &scenario_root,
        )
        .unwrap();
        let next = substep(
            &initial,
            scenario.geometry,
            &boundary,
            &profile,
            &scenario_root,
        )
        .unwrap();
        roots.push(next.frame.frame_root);
    }
    assert_eq!(roots[0], roots[1]);
    assert_eq!(roots[0], roots[2]);
}
