use super::*;
use crate::geometry::{APERTURE_Y_MIN_EDGE_FEATURE_ID, INTERNAL_PATCH_FEATURE_ID};
use crate::model::StorageOrder;
use crate::{scenario, solver};

const DENSITY_FIXTURES: [(&str, u32); 4] = [
    ("outer-face", 3_000),
    ("internal-face", 1_019),
    ("aperture-edge", 1_819),
    ("aperture-corner", 1_779),
];
const CONTACT_FIXTURES: [ContactFixtureInput; 8] = [
    ContactFixtureInput {
        id: "face-separating",
        position_um: Vec3i::new(975_000, 100_000, 500_000),
        velocity_um_s: Vec3i::new(-1_000_000, 0, 0),
    },
    ContactFixtureInput {
        id: "face-resting",
        position_um: Vec3i::new(975_000, 100_000, 500_000),
        velocity_um_s: Vec3i::new(0, 0, 0),
    },
    ContactFixtureInput {
        id: "face-direct-impact",
        position_um: Vec3i::new(975_000, 100_000, 500_000),
        velocity_um_s: Vec3i::new(1_000_000, 0, 0),
    },
    ContactFixtureInput {
        id: "face-high-speed-crossing",
        position_um: Vec3i::new(900_000, 100_000, 500_000),
        velocity_um_s: Vec3i::new(30_000_000, 0, 0),
    },
    ContactFixtureInput {
        id: "aperture-pass",
        position_um: Vec3i::new(900_000, 300_000, 500_000),
        velocity_um_s: Vec3i::new(30_000_000, 0, 0),
    },
    ContactFixtureInput {
        id: "aperture-edge-graze",
        position_um: Vec3i::new(900_000, 225_000, 500_000),
        velocity_um_s: Vec3i::new(30_000_000, 0, 0),
    },
    ContactFixtureInput {
        id: "aperture-edge-impact",
        position_um: Vec3i::new(900_000, 220_000, 500_000),
        velocity_um_s: Vec3i::new(30_000_000, 0, 0),
    },
    ContactFixtureInput {
        id: "aperture-corner-simultaneous",
        position_um: Vec3i::new(900_000, 220_000, 420_000),
        velocity_um_s: Vec3i::new(30_000_000, 0, 0),
    },
];

#[test]
fn selected_outer_extent_has_the_derived_24704_samples() {
    let sealed = scenario::find("CW-SEALED-001").unwrap();
    let support = build_density_support(sealed.geometry).unwrap();
    assert_eq!(support.len(), 24_704);
}

#[test]
fn orifice_support_is_two_sided_oriented_and_opening_free() {
    let orifice = scenario::find("CW-ORIFICE-001").unwrap();
    let support = build_density_support(orifice.geometry).unwrap();
    assert_eq!(support.len(), 10_880);
    let internal: Vec<_> = support
        .iter()
        .filter(|sample| sample.feature_id == INTERNAL_PATCH_FEATURE_ID)
        .collect();
    assert_eq!(internal.len(), 1_536);
    assert!(internal.iter().all(|sample| {
        !(sample.position_um.y >= 200_000
            && sample.position_um.y <= 400_000
            && sample.position_um.z >= 400_000
            && sample.position_um.z <= 600_000)
    }));
    assert_eq!(
        internal
            .iter()
            .filter(|sample| matches!(sample.support, BoundarySupport::FluidXLessThan(_)))
            .count(),
        768
    );
    assert_eq!(
        internal
            .iter()
            .filter(|sample| matches!(sample.support, BoundarySupport::FluidXGreaterThan(_)))
            .count(),
        768
    );
}

#[test]
fn production_and_independent_support_records_match_exactly() {
    let orifice = scenario::find("CW-ORIFICE-001").unwrap();
    let production = density_support_records(&build_density_support(orifice.geometry).unwrap());
    let independent = independent::density_support_records(orifice.geometry).unwrap();
    assert_eq!(production, independent);
}

#[test]
fn production_and_independent_geometry_classification_features_and_roots_match() {
    let orifice = scenario::find("CW-ORIFICE-001").unwrap();
    let manifest = AxisAlignedGeometryManifest::from_geometry(orifice.geometry).unwrap();
    let positions = [
        Vec3i::new(25_000, 500_000, 500_000),
        Vec3i::new(1_000_000, 100_000, 500_000),
        Vec3i::new(1_000_000, 300_000, 500_000),
        Vec3i::new(975_000, 225_000, 500_000),
        Vec3i::new(1_000_000, 225_000, 425_000),
    ];
    let production: Vec<_> = positions
        .iter()
        .copied()
        .map(|position| manifest.observe(position).unwrap())
        .collect();
    let independent = independent::geometry_observations(orifice.geometry, &positions).unwrap();
    assert_eq!(production, independent);
    assert_eq!(
        manifest.canonical_bytes(),
        independent::geometry_bytes(orifice.geometry)
    );
    assert_eq!(
        manifest.root(),
        independent::geometry_root(orifice.geometry)
    );
    assert_eq!(production[1].classification, "INTERNAL_SOLID");
    assert_eq!(production[2].classification, "OPENING");
    assert_eq!(
        production[4].closest_feature_id,
        APERTURE_Y_MIN_EDGE_FEATURE_ID
    );
}

#[test]
fn outer_face_internal_face_aperture_edge_and_corner_fixtures_match() {
    let orifice = scenario::find("CW-ORIFICE-001").unwrap();
    let samples = scenario::initial_samples(&orifice, StorageOrder::Reverse).unwrap();
    let boundary = build_density_support(orifice.geometry).unwrap();
    let production = solver::production_successor_density_fixtures(
        &samples,
        orifice.geometry,
        &boundary,
        &DENSITY_FIXTURES,
    )
    .unwrap();
    let independent =
        independent::density_fixtures(orifice.geometry, &samples, &DENSITY_FIXTURES).unwrap();
    assert_eq!(production, independent);
    assert_eq!(production.len(), DENSITY_FIXTURES.len());
    assert_ne!(
        production[1].boundary_gradient_bits,
        production[2].boundary_gradient_bits
    );
    assert_ne!(
        production[2].boundary_gradient_bits,
        production[3].boundary_gradient_bits
    );
}

#[test]
fn production_and_independent_swept_contact_fixtures_match_exactly() {
    let orifice = scenario::find("CW-ORIFICE-001").unwrap();
    let production =
        solver::production_successor_contact_fixtures(orifice.geometry, &CONTACT_FIXTURES).unwrap();
    let independent = independent::contact_fixtures(orifice.geometry, &CONTACT_FIXTURES).unwrap();
    assert_eq!(production, independent);
    assert_eq!(production[0].active_rows, 0);
    assert_eq!(production[1].active_rows, 0);
    assert_eq!(
        production[2].features[0].feature_id,
        INTERNAL_PATCH_FEATURE_ID
    );
    assert_eq!(production[4].active_rows, 0);
    assert_eq!(production[5].active_rows, 0);
    assert_eq!(
        production[6].features[0].feature_id,
        APERTURE_Y_MIN_EDGE_FEATURE_ID
    );
    assert_eq!(production[7].features.len(), 2);
}

#[test]
fn successor_orifice_profile_accepts_the_local_preflight() {
    let orifice = scenario::find("CW-ORIFICE-001").unwrap();
    let samples = scenario::initial_samples(&orifice, StorageOrder::Reverse).unwrap();
    let boundary = build_density_support(orifice.geometry).unwrap();
    let (mut frame, _) =
        solver::initial_frame(samples, orifice.geometry, &boundary, &[0; 32], &[1; 32]).unwrap();
    for step in 1..=24 {
        let next = solver::successor_pcg_constrained_substep(
            &frame,
            orifice.geometry,
            &boundary,
            &[0; 32],
            &[1; 32],
        )
        .unwrap_or_else(|error| panic!("orifice step {step} failed: {error}"));
        assert!(next.outcome.summary.density_error_ppb <= 100_000);
        let manifest = AxisAlignedGeometryManifest::from_geometry(orifice.geometry).unwrap();
        for sample in &next.outcome.frame.samples {
            let observation = manifest.observe(sample.position_um).unwrap();
            assert!(
                observation.closest_distance_squared_um2
                    >= i128::from(PARTICLE_RADIUS_UM) * i128::from(PARTICLE_RADIUS_UM),
                "sample {} has successor clearance {:?} at step {step}",
                sample.id,
                observation
            );
        }
        frame = next.outcome.frame;
    }
    assert_eq!(frame.step, 24);
    assert_eq!(frame.samples.len(), 6_000);
    assert!(
        frame
            .samples
            .iter()
            .any(|sample| sample.position_um.x >= 1_000_000),
        "the local preflight did not exercise one legal aperture crossing"
    );
}

#[test]
fn successor_static_capacity_accepts_below_and_equal_but_rejects_above() {
    for value in [32_767, 32_768] {
        assert!(
            validate_capacity(
                value,
                SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES,
                BOUNDARY_CAPACITY_EXCEEDED,
                "successor static boundary samples",
            )
            .is_ok()
        );
    }
    assert_eq!(
        validate_capacity(
            32_769,
            SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES,
            BOUNDARY_CAPACITY_EXCEEDED,
            "successor static boundary samples",
        )
        .unwrap_err()
        .code(),
        BOUNDARY_CAPACITY_EXCEEDED
    );
}
