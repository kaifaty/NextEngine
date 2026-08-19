#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::boundary::{BoundarySample, BoundarySupport};
use crate::error::{
    AUDIT_INVALID, BOUNDARY_CAPACITY_EXCEEDED, BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
    NEIGHBOR_CAPACITY_EXCEEDED, REPORT_CAPACITY_EXCEEDED, SCENARIO_INVALID, WaterError,
};
use crate::geometry::AxisAlignedGeometryManifest;
use crate::hash::{self, FrozenRoots, SuccessorRoots};
use crate::model::{AcceptedFrame, CanonicalSample, StorageOrder, Vec3f, Vec3i};
use crate::oracle::command::{
    tool_commit, tool_tree_state, validate_output_path, validate_report_capacity,
};
use crate::profile::{
    DT, GRAVITY_MAGNITUDE, MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW, MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
    PARTICLE_RADIUS_UM, SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES, UNIFORM_MASS,
};
use crate::scenario::validate_capacity;
use crate::{scenario, solver};

mod independent;
mod support;

pub(crate) use support::build_density_support;

const PREFLIGHT_STEPS: u32 = 24;
const HYDRO_EXTENDED_STEPS: u32 = 1_200;
const W0E_STEP_24_ROOT: &str = "399705af374393503581dd9f3d03bc0bba8e44af732544c5ea3c19d7dbd90b4f";
const W0E_STEP_1200_ROOT: &str = "026d26585edbda74aff93ef126810b0ced0a7c9f5d623b4dbf60260b48554b18";
const W0E_FREEFALL_ROOT: &str = "cbe47b57dbb819e44eabe53049a1b9cb44c6a94626d560421c73deb6b1db4011";

#[derive(Serialize)]
struct SuccessorEnvelope<'a> {
    schema_version: u32,
    status: &'a str,
    command: &'a str,
    details: &'a SuccessorClosureReport,
}

#[derive(Serialize)]
struct SuccessorClosureReport {
    report_schema: String,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: String,
    build_profile: String,
    build_rustflags: String,
    classification: String,
    disposition: String,
    corpus_credit: String,
    product_check: String,
    conclusion: String,
    roots: SuccessorRootReport,
    geometry_comparison: String,
    geometry_observations: Vec<crate::geometry::GeometryObservation>,
    density_support_comparison: String,
    density_support_count: usize,
    density_support_root: String,
    density_fixture_comparison: String,
    density_fixtures: Vec<DensityFixtureObservation>,
    contact_fixture_comparison: String,
    contact_fixtures: Vec<ContactFixtureObservation>,
    scenario_boundary_counts: Vec<ScenarioBoundaryCount>,
    capacity_thresholds: CapacityThresholds,
    hydro_regression: HydroRegression,
    freefall_regression: FreefallRegression,
    orifice_preflight: OrificePreflight,
    repeatability: String,
    timing_classification: String,
    wall_clock_nanoseconds: u64,
}

#[derive(Serialize)]
struct SuccessorRootReport {
    document: String,
    float_profile: String,
    execution_manifest: String,
    corpus: String,
    fixtures: String,
    geometry: String,
    execution_profile: String,
    scenarios: Vec<ScenarioRootReport>,
}

#[derive(Serialize)]
struct ScenarioRootReport {
    scenario_id: String,
    root: String,
}

#[derive(Serialize)]
struct ScenarioBoundaryCount {
    scenario_id: String,
    sample_count: usize,
    admitted: bool,
}

#[derive(Serialize)]
struct CapacityThresholds {
    static_boundary_32767: String,
    static_boundary_32768: String,
    static_boundary_32769: String,
    fluid_row_127: String,
    fluid_row_128: String,
    fluid_row_129: String,
    boundary_row_127: String,
    boundary_row_128: String,
    boundary_row_129: String,
}

#[derive(Serialize)]
struct HydroRegression {
    status: String,
    completed_steps: u32,
    maximum_density_iterations: u8,
    maximum_density_error_ppb: i64,
    step_24_root: String,
    step_1200_root: String,
}

#[derive(Serialize)]
struct FreefallRegression {
    status: String,
    compared_frames: u32,
    final_root: String,
    active_contact_constraints: u32,
}

#[derive(Serialize)]
struct OrificePreflight {
    status: String,
    completed_steps: u32,
    initial_left_count: usize,
    initial_right_count: usize,
    final_left_count: usize,
    final_right_count: usize,
    left_to_right_crossings: u64,
    right_to_left_crossings: u64,
    minimum_clearance_squared_um2: i128,
    maximum_density_iterations: u8,
    maximum_density_error_ppb: i64,
    maximum_contact_constraints: usize,
    pressure_fluid_impulse_bits: [String; 3],
    contact_fluid_impulse_bits: [String; 3],
    feature_contact_fluid_impulse_bits: [String; 3],
    feature_sum_difference_ppb: i64,
    momentum_residual_ppb: i64,
    final_frame_root: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DensityFixtureObservation {
    pub(crate) role: String,
    pub(crate) sample_id: u32,
    pub(crate) position_um: Vec3i,
    pub(crate) fluid_neighbor_count: usize,
    pub(crate) boundary_neighbor_count: usize,
    pub(crate) rho_ratio_bits: String,
    pub(crate) alpha_bits: String,
    pub(crate) fluid_gradient_bits: [String; 3],
    pub(crate) boundary_gradient_bits: [String; 3],
    pub(crate) total_gradient_bits: [String; 3],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DensitySupportRecord {
    pub(crate) id: u32,
    pub(crate) position_um: Vec3i,
    pub(crate) volume_bits: String,
    pub(crate) feature_id: u32,
    pub(crate) support: String,
}

#[derive(Clone, Copy)]
pub(crate) struct ContactFixtureInput {
    pub(crate) id: &'static str,
    pub(crate) position_um: Vec3i,
    pub(crate) velocity_um_s: Vec3i,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FeatureActivation {
    pub(crate) feature_id: u32,
    pub(crate) active_constraints: u32,
    pub(crate) fluid_impulse_bits: [String; 3],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ContactFixtureObservation {
    pub(crate) id: String,
    pub(crate) position_um: Vec3i,
    pub(crate) velocity_before_um_s: Vec3i,
    pub(crate) velocity_after_bits: [String; 3],
    pub(crate) fluid_impulse_bits: [String; 3],
    pub(crate) active_rows: usize,
    pub(crate) active_components: usize,
    pub(crate) features: Vec<FeatureActivation>,
}

pub(crate) fn density_support_records(boundary: &[BoundarySample]) -> Vec<DensitySupportRecord> {
    boundary
        .iter()
        .map(|sample| DensitySupportRecord {
            id: sample.id,
            position_um: sample.position_um,
            volume_bits: format!("0x{:016x}", sample.volume.to_bits()),
            feature_id: sample.feature_id,
            support: match sample.support {
                BoundarySupport::Unrestricted => "UNRESTRICTED".to_owned(),
                BoundarySupport::FluidXLessThan(coordinate_um) => {
                    format!("FLUID_X_LESS_THAN:{coordinate_um}")
                }
                BoundarySupport::FluidXGreaterThan(coordinate_um) => {
                    format!("FLUID_X_GREATER_THAN:{coordinate_um}")
                }
            },
        })
        .collect()
}

pub(crate) fn run_xtask(
    repository_root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let output_flag = arguments.next().ok_or_else(argument_error)?;
    let output = arguments.next().ok_or_else(argument_error)?;
    if output_flag != "--output" || arguments.next().is_some() {
        return Err(argument_error());
    }
    let output = PathBuf::from(output);
    validate_output_path(repository_root, &output)?;
    crate::profile::validate_execution_profile()?;
    crate::profile::validate_float_environment()?;
    let rejected_roots = FrozenRoots::verify(repository_root)?;
    let successor_roots = SuccessorRoots::verify(repository_root)?;
    let started = Instant::now();

    let orifice = scenario::find("CW-ORIFICE-001")?;
    let geometry_manifest = AxisAlignedGeometryManifest::from_geometry(orifice.geometry)?;
    let geometry_positions = [
        Vec3i::new(25_000, 500_000, 500_000),
        Vec3i::new(1_000_000, 100_000, 500_000),
        Vec3i::new(1_000_000, 300_000, 500_000),
        Vec3i::new(975_000, 225_000, 500_000),
        Vec3i::new(1_000_000, 225_000, 425_000),
    ];
    let mut geometry_observations = Vec::new();
    for position in geometry_positions {
        geometry_observations.push(geometry_manifest.observe(position)?);
    }
    let independent_geometry =
        independent::geometry_observations(orifice.geometry, &geometry_positions)?;
    let geometry_matches = geometry_observations == independent_geometry
        && geometry_manifest.canonical_bytes() == independent::geometry_bytes(orifice.geometry)
        && geometry_manifest.root() == independent::geometry_root(orifice.geometry)
        && geometry_manifest.root() == successor_roots.geometry;

    let orifice_boundary = build_density_support(orifice.geometry)?;
    let production_support_records = density_support_records(&orifice_boundary);
    let independent_support_records = independent::density_support_records(orifice.geometry)?;
    let support_matches = production_support_records == independent_support_records;
    let density_support_root = serializable_root(
        b"nextengine.continuum-water.successor-density-support.v1\0",
        &production_support_records,
    )?;

    let orifice_samples = scenario::initial_samples(&orifice, StorageOrder::Reverse)?;
    let density_selections = [
        ("outer-face", 3_000),
        ("internal-face", 1_019),
        ("aperture-edge", 1_819),
        ("aperture-corner", 1_779),
    ];
    let density_fixtures = solver::production_successor_density_fixtures(
        &orifice_samples,
        orifice.geometry,
        &orifice_boundary,
        &density_selections,
    )?;
    let independent_density =
        independent::density_fixtures(orifice.geometry, &orifice_samples, &density_selections)?;
    let density_matches = density_fixtures == independent_density;

    let contact_inputs = contact_fixture_inputs();
    let contact_fixtures =
        solver::production_successor_contact_fixtures(orifice.geometry, &contact_inputs)?;
    let independent_contact = independent::contact_fixtures(orifice.geometry, &contact_inputs)?;
    let contact_matches = contact_fixtures == independent_contact;

    let scenario_ids = [
        "CW-HYDRO-001",
        "CW-FREEFALL-001",
        "CW-DAMBREAK-001",
        "CW-STILL-001",
        "CW-ORIFICE-001",
        "CW-SEALED-001",
        "CW-ORDER-001",
    ];
    let mut scenario_boundary_counts = Vec::new();
    let mut scenario_roots = Vec::new();
    let mut all_scenarios_admitted = true;
    for scenario_id in scenario_ids {
        let selected = scenario::find(scenario_id)?;
        let source_projection = successor_roots.scenario_projection(scenario_id)?;
        let implementation_projection = scenario::successor_projection(&selected)?;
        if source_projection != implementation_projection.as_bytes() {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!(
                    "scenario {scenario_id} implementation differs from its successor hash-bound projection"
                ),
            ));
        }
        let count = build_density_support(selected.geometry)?.len();
        let admitted = count <= SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES;
        all_scenarios_admitted &= admitted;
        scenario_boundary_counts.push(ScenarioBoundaryCount {
            scenario_id: scenario_id.to_owned(),
            sample_count: count,
            admitted,
        });
        scenario_roots.push(ScenarioRootReport {
            scenario_id: scenario_id.to_owned(),
            root: hash::hex(&successor_roots.scenario_root(scenario_id)?),
        });
    }
    let capacity_thresholds = capacity_thresholds();
    let capacities_match = all_scenarios_admitted
        && capacity_thresholds.static_boundary_32767 == "PASS"
        && capacity_thresholds.static_boundary_32768 == "PASS"
        && capacity_thresholds.static_boundary_32769 == "EXPECTED_REJECTION"
        && capacity_thresholds.fluid_row_127 == "PASS"
        && capacity_thresholds.fluid_row_128 == "PASS"
        && capacity_thresholds.fluid_row_129 == "EXPECTED_REJECTION"
        && capacity_thresholds.boundary_row_127 == "PASS"
        && capacity_thresholds.boundary_row_128 == "PASS"
        && capacity_thresholds.boundary_row_129 == "EXPECTED_REJECTION";

    let hydro_regression = run_hydro_regression(&rejected_roots)?;
    let freefall_regression = run_freefall_regression(&rejected_roots)?;
    let orifice_preflight = run_orifice_preflight(
        &orifice,
        &orifice_samples,
        &orifice_boundary,
        &successor_roots,
    )?;
    let survived = geometry_matches
        && support_matches
        && density_matches
        && contact_matches
        && capacities_match
        && hydro_regression.status == "EXACT_W0E_TRANSCRIPT"
        && freefall_regression.status == "EXACT_W0E_TRANSCRIPT"
        && orifice_preflight.status == "PREFLIGHT_PASS";
    let disposition = if survived {
        "SUCCESSOR_PROFILE_ROOTS_FROZEN / W1_AUTHORIZED"
    } else {
        "SUCCESSOR_PROFILE_REJECTED"
    };
    let initial_tree_state = tool_tree_state(repository_root);
    let mut report = SuccessorClosureReport {
        report_schema: "nextengine.continuum-water.successor-profile-closure.v1".to_owned(),
        tool_commit: tool_commit(repository_root),
        tool_tree_state: initial_tree_state,
        toolchain_target: env!("WATER_BUILD_TARGET").to_owned(),
        build_profile: env!("WATER_BUILD_PROFILE").to_owned(),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        classification: "W0F_RESEARCH_ONLY_PROFILE_CLOSURE".to_owned(),
        disposition: disposition.to_owned(),
        corpus_credit: "NO_W1_CORPUS_CREDIT".to_owned(),
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned(),
        conclusion: if survived {
            "the geometry, density support, swept contact, capacity, W0E regression, freefall and orifice preflight gates pass under hash-bound successor inputs; reproduce the bounded roots in two clean runs before handoff".to_owned()
        } else {
            "at least one W0F discriminator failed; issue no successor roots and do not start W1"
                .to_owned()
        },
        roots: SuccessorRootReport {
            document: hash::hex(&successor_roots.document),
            float_profile: hash::hex(&successor_roots.float_profile),
            execution_manifest: hash::hex(&successor_roots.execution_manifest),
            corpus: hash::hex(&successor_roots.corpus),
            fixtures: hash::hex(&successor_roots.fixtures),
            geometry: hash::hex(&successor_roots.geometry),
            execution_profile: hash::hex(&successor_roots.execution_profile),
            scenarios: scenario_roots,
        },
        geometry_comparison: comparison(geometry_matches),
        geometry_observations,
        density_support_comparison: comparison(support_matches),
        density_support_count: production_support_records.len(),
        density_support_root,
        density_fixture_comparison: comparison(density_matches),
        density_fixtures,
        contact_fixture_comparison: comparison(contact_matches),
        contact_fixtures,
        scenario_boundary_counts,
        capacity_thresholds,
        hydro_regression,
        freefall_regression,
        orifice_preflight,
        repeatability: "ROOT_SET_EMITTED_FOR_CLEAN_PAIR_COMPARISON".to_owned(),
        timing_classification: "DIAGNOSTIC_ONLY".to_owned(),
        wall_clock_nanoseconds: u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX),
    };
    let ending_tree_state = tool_tree_state(repository_root);
    if ending_tree_state != report.tool_tree_state {
        report.tool_tree_state = format!("{}->{ending_tree_state}", report.tool_tree_state);
    }
    write_report(&output, &report, survived)?;
    serde_json::to_string(&serde_json::json!({
        "schema_version": 1,
        "status": if survived { "REPORT_ONLY" } else { "FAIL" },
        "command": "continuum water close-successor-profile",
        "details": {
            "report": output.display().to_string(),
            "disposition": disposition,
            "geometry": report.geometry_comparison,
            "density_support": report.density_support_comparison,
            "density_fixtures": report.density_fixture_comparison,
            "contact_fixtures": report.contact_fixture_comparison,
            "hydro_regression": report.hydro_regression.status,
            "freefall_regression": report.freefall_regression.status,
            "orifice_preflight": report.orifice_preflight.status,
            "product_check": report.product_check,
        }
    }))
    .map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize successor closure command result: {error}"),
        )
    })
}

fn argument_error() -> WaterError {
    WaterError::new(
        SCENARIO_INVALID,
        "close-successor-profile requires --output <absolute-path>",
    )
}

fn comparison(matches: bool) -> String {
    if matches { "EXACT_MATCH" } else { "MISMATCH" }.to_owned()
}

fn contact_fixture_inputs() -> [ContactFixtureInput; 8] {
    [
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
    ]
}

fn run_hydro_regression(rejected_roots: &FrozenRoots) -> Result<HydroRegression, WaterError> {
    let hydro = scenario::find("CW-HYDRO-001")?;
    let scenario_root = scenario::root_for(&hydro, rejected_roots)?;
    let samples = scenario::initial_samples(&hydro, StorageOrder::Reverse)?;
    let boundary = build_density_support(hydro.geometry)?;
    let (mut frame, _) = solver::initial_frame(
        samples,
        hydro.geometry,
        &boundary,
        &rejected_roots.execution_profile,
        &scenario_root,
    )?;
    let mut step_24_root = String::new();
    let mut maximum_density_iterations = 0_u8;
    let mut maximum_density_error_ppb = 0_i64;
    for step in 1..=HYDRO_EXTENDED_STEPS {
        let next = solver::successor_pcg_constrained_substep(
            &frame,
            hydro.geometry,
            &boundary,
            &rejected_roots.execution_profile,
            &scenario_root,
        )?;
        maximum_density_iterations =
            maximum_density_iterations.max(next.outcome.summary.density_iterations);
        maximum_density_error_ppb =
            maximum_density_error_ppb.max(next.outcome.summary.density_error_ppb);
        frame = next.outcome.frame;
        if step == PREFLIGHT_STEPS {
            step_24_root = hash::hex(&frame.frame_root);
        }
    }
    let step_1200_root = hash::hex(&frame.frame_root);
    let exact = step_24_root == W0E_STEP_24_ROOT && step_1200_root == W0E_STEP_1200_ROOT;
    Ok(HydroRegression {
        status: if exact {
            "EXACT_W0E_TRANSCRIPT".to_owned()
        } else {
            "REGRESSION".to_owned()
        },
        completed_steps: frame.step,
        maximum_density_iterations,
        maximum_density_error_ppb,
        step_24_root,
        step_1200_root,
    })
}

fn run_freefall_regression(rejected_roots: &FrozenRoots) -> Result<FreefallRegression, WaterError> {
    let freefall = scenario::find("CW-FREEFALL-001")?;
    let scenario_root = scenario::root_for(&freefall, rejected_roots)?;
    let samples = scenario::initial_samples(&freefall, StorageOrder::Reverse)?;
    let boundary = build_density_support(freefall.geometry)?;
    let (mut frame, _) = solver::initial_frame(
        samples,
        freefall.geometry,
        &boundary,
        &rejected_roots.execution_profile,
        &scenario_root,
    )?;
    let mut active_contact_constraints = 0_u32;
    for _ in 0..freefall.steps {
        let next = solver::successor_pcg_constrained_substep(
            &frame,
            freefall.geometry,
            &boundary,
            &rejected_roots.execution_profile,
            &scenario_root,
        )?;
        for count in next.projection.feature_active_constraints {
            active_contact_constraints = active_contact_constraints
                .checked_add(count)
                .ok_or_else(|| WaterError::new(AUDIT_INVALID, "freefall contact count overflow"))?;
        }
        frame = next.outcome.frame;
    }
    let final_root = hash::hex(&frame.frame_root);
    Ok(FreefallRegression {
        status: if final_root == W0E_FREEFALL_ROOT && active_contact_constraints == 0 {
            "EXACT_W0E_TRANSCRIPT".to_owned()
        } else {
            "REGRESSION".to_owned()
        },
        compared_frames: freefall.steps + 1,
        final_root,
        active_contact_constraints,
    })
}

fn run_orifice_preflight(
    orifice: &crate::model::Scenario,
    samples: &[CanonicalSample],
    boundary: &[BoundarySample],
    roots: &SuccessorRoots,
) -> Result<OrificePreflight, WaterError> {
    let scenario_root = roots.scenario_root(orifice.id)?;
    let (mut frame, _) = solver::initial_frame(
        samples.to_vec(),
        orifice.geometry,
        boundary,
        &roots.execution_profile,
        &scenario_root,
    )?;
    let manifest = AxisAlignedGeometryManifest::from_geometry(orifice.geometry)?;
    let patch = manifest
        .internal_patch()
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "orifice preflight has no internal plane"))?;
    let (initial_left_count, initial_right_count) = chamber_counts(&frame, patch.coordinate_um);
    let initial_momentum = frame_momentum(&frame)?;
    let mut gravity_impulse = Vec3f::ZERO;
    let mut pressure_impulse = Vec3f::ZERO;
    let mut contact_impulse = Vec3f::ZERO;
    let mut feature_impulses = [Vec3f::ZERO; 25];
    let mut left_to_right_crossings = 0_u64;
    let mut right_to_left_crossings = 0_u64;
    let mut minimum_clearance_squared = i128::MAX;
    let mut maximum_density_iterations = 0_u8;
    let mut maximum_density_error_ppb = 0_i64;
    let mut maximum_contact_constraints = 0_usize;
    update_minimum_clearance(&manifest, &frame, &mut minimum_clearance_squared)?;
    for _step in 1..=PREFLIGHT_STEPS {
        let next = solver::successor_pcg_constrained_substep(
            &frame,
            orifice.geometry,
            boundary,
            &roots.execution_profile,
            &scenario_root,
        )?;
        count_strict_crossings(
            &frame,
            &next.outcome.frame,
            patch,
            &mut left_to_right_crossings,
            &mut right_to_left_crossings,
        )?;
        for _sample in &next.outcome.frame.samples {
            let gravity_row = Vec3f::new(0.0, UNIFORM_MASS * DT * -GRAVITY_MAGNITUDE, 0.0);
            gravity_impulse = gravity_impulse
                .add(gravity_row)
                .checked("orifice gravity impulse reduction")?;
        }
        for impulse in next.outcome.boundary_impulses {
            pressure_impulse = pressure_impulse
                .add(impulse)
                .checked("orifice pressure impulse reduction")?;
        }
        contact_impulse = contact_impulse
            .add(next.projection.fluid_impulse)
            .checked("orifice contact impulse reduction")?;
        let mut step_constraints = 0_usize;
        for (feature, count) in next
            .projection
            .feature_active_constraints
            .iter()
            .copied()
            .enumerate()
        {
            step_constraints = step_constraints
                .checked_add(usize::try_from(count).map_err(|_| {
                    WaterError::new(AUDIT_INVALID, "orifice constraint conversion overflow")
                })?)
                .ok_or_else(|| {
                    WaterError::new(AUDIT_INVALID, "orifice constraint count overflow")
                })?;
            feature_impulses[feature] = feature_impulses[feature]
                .add(next.projection.feature_fluid_impulses[feature])
                .checked("orifice feature impulse reduction")?;
        }
        maximum_contact_constraints = maximum_contact_constraints.max(step_constraints);
        maximum_density_iterations =
            maximum_density_iterations.max(next.outcome.summary.density_iterations);
        maximum_density_error_ppb =
            maximum_density_error_ppb.max(next.outcome.summary.density_error_ppb);
        update_minimum_clearance(
            &manifest,
            &next.outcome.frame,
            &mut minimum_clearance_squared,
        )?;
        frame = next.outcome.frame;
    }
    let (final_left_count, final_right_count) = chamber_counts(&frame, patch.coordinate_um);
    let final_momentum = frame_momentum(&frame)?;
    let residual = final_momentum
        .sub(initial_momentum)
        .sub(gravity_impulse)
        .sub(pressure_impulse)
        .sub(contact_impulse)
        .checked("orifice momentum residual")?;
    let denominator = (vector_norm(initial_momentum)?
        + vector_norm(gravity_impulse)?
        + vector_norm(pressure_impulse)?
        + vector_norm(contact_impulse)?)
    .max(1.0);
    let momentum_residual_ppb = crate::profile::quantize_ppb(crate::model::checked_scalar(
        vector_norm(residual)? / denominator,
        "orifice normalized momentum residual",
    )?)?;
    let mut feature_impulse_sum = Vec3f::ZERO;
    for impulse in feature_impulses {
        feature_impulse_sum = feature_impulse_sum
            .add(impulse)
            .checked("orifice feature impulse sum")?;
    }
    let feature_difference = feature_impulse_sum
        .sub(contact_impulse)
        .checked("orifice feature impulse difference")?;
    let feature_sum_difference_ppb = crate::profile::quantize_ppb(crate::model::checked_scalar(
        vector_norm(feature_difference)? / vector_norm(contact_impulse)?.max(1.0),
        "orifice feature impulse normalized difference",
    )?)?;
    let radius_squared = i128::from(PARTICLE_RADIUS_UM) * i128::from(PARTICLE_RADIUS_UM);
    let partition_exact = final_left_count + final_right_count == samples.len();
    let legal_transfer = left_to_right_crossings > 0 && final_right_count > 0;
    let passed = frame.step == PREFLIGHT_STEPS
        && partition_exact
        && legal_transfer
        && minimum_clearance_squared >= radius_squared
        && maximum_density_error_ppb <= 100_000
        && momentum_residual_ppb <= 10_000_000
        && feature_sum_difference_ppb <= 1_000;
    Ok(OrificePreflight {
        status: if passed {
            "PREFLIGHT_PASS".to_owned()
        } else {
            "PREFLIGHT_FAILED".to_owned()
        },
        completed_steps: frame.step,
        initial_left_count,
        initial_right_count,
        final_left_count,
        final_right_count,
        left_to_right_crossings,
        right_to_left_crossings,
        minimum_clearance_squared_um2: minimum_clearance_squared,
        maximum_density_iterations,
        maximum_density_error_ppb,
        maximum_contact_constraints,
        pressure_fluid_impulse_bits: crate::audit::vector_bits(pressure_impulse),
        contact_fluid_impulse_bits: crate::audit::vector_bits(contact_impulse),
        feature_contact_fluid_impulse_bits: crate::audit::vector_bits(feature_impulse_sum),
        feature_sum_difference_ppb,
        momentum_residual_ppb,
        final_frame_root: hash::hex(&frame.frame_root),
    })
}

fn chamber_counts(frame: &AcceptedFrame, wall_x_um: i64) -> (usize, usize) {
    let right = frame
        .samples
        .iter()
        .filter(|sample| sample.position_um.x >= wall_x_um)
        .count();
    (frame.samples.len() - right, right)
}

fn count_strict_crossings(
    prior: &AcceptedFrame,
    next: &AcceptedFrame,
    patch: crate::geometry::InternalPlanePatch,
    left_to_right: &mut u64,
    right_to_left: &mut u64,
) -> Result<(), WaterError> {
    for (prior, next) in prior.samples.iter().zip(&next.samples) {
        let prior_side = prior.position_um.x - patch.coordinate_um;
        let next_side = next.position_um.x - patch.coordinate_um;
        let direction = if prior_side < 0 && next_side >= 0 {
            1_i8
        } else if prior_side > 0 && next_side <= 0 {
            -1_i8
        } else {
            0_i8
        };
        if direction == 0 {
            continue;
        }
        validate_strict_crossing(prior.position_um, next.position_um, patch)?;
        if direction > 0 {
            *left_to_right = left_to_right
                .checked_add(1)
                .ok_or_else(|| WaterError::new(AUDIT_INVALID, "left crossing count overflow"))?;
        } else {
            *right_to_left = right_to_left
                .checked_add(1)
                .ok_or_else(|| WaterError::new(AUDIT_INVALID, "right crossing count overflow"))?;
        }
    }
    Ok(())
}

fn validate_strict_crossing(
    start: Vec3i,
    end: Vec3i,
    patch: crate::geometry::InternalPlanePatch,
) -> Result<(), WaterError> {
    let denominator = i128::from(end.x) - i128::from(start.x);
    let numerator = i128::from(patch.coordinate_um) - i128::from(start.x);
    let mut denominator = denominator;
    let mut numerator = numerator;
    if denominator < 0 {
        denominator = -denominator;
        numerator = -numerator;
    }
    if denominator == 0 {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "strict aperture crossing has zero denominator",
        ));
    }
    let y =
        i128::from(start.y) * denominator + (i128::from(end.y) - i128::from(start.y)) * numerator;
    let z =
        i128::from(start.z) * denominator + (i128::from(end.z) - i128::from(start.z)) * numerator;
    let opening = patch.opening;
    let y_min = i128::from(opening.y_min_um + PARTICLE_RADIUS_UM) * denominator;
    let y_max = i128::from(opening.y_max_um - PARTICLE_RADIUS_UM) * denominator;
    let z_min = i128::from(opening.z_min_um + PARTICLE_RADIUS_UM) * denominator;
    let z_max = i128::from(opening.z_max_um - PARTICLE_RADIUS_UM) * denominator;
    if y < y_min || y > y_max || z < z_min || z > z_max {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "successor contact allowed a crossing without radius clearance",
        ));
    }
    Ok(())
}

fn update_minimum_clearance(
    manifest: &AxisAlignedGeometryManifest,
    frame: &AcceptedFrame,
    minimum: &mut i128,
) -> Result<(), WaterError> {
    for sample in &frame.samples {
        let observation = manifest.observe(sample.position_um)?;
        *minimum = (*minimum).min(observation.closest_distance_squared_um2);
    }
    Ok(())
}

fn frame_momentum(frame: &AcceptedFrame) -> Result<Vec3f, WaterError> {
    let mut momentum = Vec3f::ZERO;
    for sample in &frame.samples {
        let velocity = Vec3f::new(
            crate::profile::decode_velocity(sample.velocity_um_s.x)?,
            crate::profile::decode_velocity(sample.velocity_um_s.y)?,
            crate::profile::decode_velocity(sample.velocity_um_s.z)?,
        );
        momentum = momentum
            .add(velocity.scale(UNIFORM_MASS))
            .checked("successor momentum reduction")?;
    }
    Ok(momentum)
}

fn vector_norm(vector: Vec3f) -> Result<f64, WaterError> {
    crate::model::checked_scalar(vector.dot(vector), "successor vector norm").map(f64::sqrt)
}

fn capacity_thresholds() -> CapacityThresholds {
    CapacityThresholds {
        static_boundary_32767: capacity_outcome(
            32_767,
            SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES,
            BOUNDARY_CAPACITY_EXCEEDED,
        ),
        static_boundary_32768: capacity_outcome(
            32_768,
            SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES,
            BOUNDARY_CAPACITY_EXCEEDED,
        ),
        static_boundary_32769: capacity_outcome(
            32_769,
            SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES,
            BOUNDARY_CAPACITY_EXCEEDED,
        ),
        fluid_row_127: capacity_outcome(
            127,
            MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
            NEIGHBOR_CAPACITY_EXCEEDED,
        ),
        fluid_row_128: capacity_outcome(
            128,
            MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
            NEIGHBOR_CAPACITY_EXCEEDED,
        ),
        fluid_row_129: capacity_outcome(
            129,
            MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
            NEIGHBOR_CAPACITY_EXCEEDED,
        ),
        boundary_row_127: capacity_outcome(
            127,
            MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW,
            BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
        ),
        boundary_row_128: capacity_outcome(
            128,
            MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW,
            BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
        ),
        boundary_row_129: capacity_outcome(
            129,
            MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW,
            BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED,
        ),
    }
}

fn capacity_outcome(value: usize, maximum: usize, code: &'static str) -> String {
    match validate_capacity(value, maximum, code, "successor threshold") {
        Ok(()) => "PASS".to_owned(),
        Err(error) if error.code() == code => "EXPECTED_REJECTION".to_owned(),
        Err(error) => format!("UNEXPECTED:{}", error.code()),
    }
}

fn serializable_root<T: Serialize>(domain: &[u8], value: &T) -> Result<String, WaterError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("cannot serialize successor rooted fixture: {error}"),
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    Ok(hash::hex(&hasher.finalize().into()))
}

fn write_report(
    output: &Path,
    report: &SuccessorClosureReport,
    survived: bool,
) -> Result<(), WaterError> {
    let envelope = SuccessorEnvelope {
        schema_version: 1,
        status: if survived { "REPORT_ONLY" } else { "FAIL" },
        command: "continuum water close-successor-profile",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("cannot serialize successor closure report: {error}"),
        )
    })?;
    bytes.push(b'\n');
    validate_report_capacity(bytes.len())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|error| {
            WaterError::new(
                SCENARIO_INVALID,
                format!(
                    "cannot create successor report {}: {error}",
                    output.display()
                ),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot write successor report {}: {error}",
                output.display()
            ),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot sync successor report {}: {error}", output.display()),
        )
    })
}

#[cfg(test)]
mod tests;
