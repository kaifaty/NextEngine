use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use next_contracts::canonical::sha256;
use next_contracts::ids::ContentHash;
use next_presentation::physical_sound_lab::{
    OfflineModalMode, cook_offline_q30_modal_bank, decode_offline_q30_samples,
    render_offline_modal_recurrence_unscaled, render_offline_q30_modal_recurrence,
};
use serde::{Deserialize, Serialize};

mod support;

use support::*;

const PROFILE_SCHEMA: &str = "nextengine.external-controlled-glass-modal-corpus.v0";
const REPORT_SCHEMA: &str =
    "nextengine.experimental-physical-sound-controlled-glass-corpus.report.v0";
const QUALITY_MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-quality.manifest.v0";
const REQUIRED_CLAIM: &str =
    "EXTERNAL_CONTROLLED_P0_ONLY / NO_PHYSICAL_IDENTIFICATION_OR_P1_PROMOTION";
const MAX_PROFILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_MODE_COUNT: usize = 128;
const MAX_CONDITION_COUNT: usize = 128;
const FORCE_RELATION_TOLERANCE: f64 = 1.0e-9;
const POSITION_COSINE_DISTANCE_MINIMUM: f64 = 0.02;
const Q30_MAXIMUM_ABSOLUTE_RESIDUAL_LIMIT: f64 = 1.0e-3;
const Q30_RMS_RESIDUAL_LIMIT: f64 = 1.0e-4;
const Q30_CORRELATION_MINIMUM: f64 = 0.999_99;
const AUDITION_SILENCE_MILLISECONDS: u32 = 200;

pub(super) struct Request {
    profile: PathBuf,
    output: PathBuf,
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, String> {
    let mut profile = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--profile" => set_once(&mut profile, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    Ok(Request {
        profile: profile
            .ok_or_else(|| "physical-sound-corpus requires --profile <external-json>".to_owned())?,
        output: output.ok_or_else(|| {
            "physical-sound-corpus requires --output <external-empty-directory>".to_owned()
        })?,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        return Err(format!("duplicate argument: {flag}"));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ExternalProfile {
    schema: String,
    claim: String,
    object_id: String,
    source_recipe: ExternalFile,
    solver: ExternalSolver,
    geometry: ExternalGeometry,
    material: ExternalMaterial,
    boundary: ExternalBoundary,
    modal_basis: ExternalModalBasis,
    pickup: ExternalPoint,
    sample_rate_hz: u32,
    frame_count: usize,
    global_amplitude_gain: f64,
    target_global_peak: f64,
    modes: Vec<ExternalMode>,
    strikes: Vec<ExternalStrike>,
    impulses: Vec<ExternalImpulse>,
    heldout: ExternalHeldout,
    conditions: Vec<ExternalCondition>,
}

#[derive(Debug, Deserialize)]
struct ExternalFile {
    file: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct ExternalSolver {
    id: String,
    python: String,
    numpy: String,
    scipy: String,
    method: String,
}

#[derive(Debug, Deserialize)]
struct ExternalGeometry {
    kind: String,
    outer_size_m: [f64; 3],
    wall_thickness_m: f64,
    bottom_thickness_m: f64,
    cell_size_m: f64,
    axes: String,
    tetrahedra_per_cell: u32,
    node_count: usize,
    tetrahedron_count: usize,
    fixed_node_count: usize,
    node_file: String,
    node_sha256: String,
    tetrahedron_file: String,
    tetrahedron_sha256: String,
    fixed_node_file: String,
    fixed_node_sha256: String,
}

#[derive(Debug, Deserialize)]
struct ExternalMaterial {
    name: String,
    density_kg_m3: f64,
    youngs_modulus_pa: f64,
    poisson_ratio: f64,
    damping: ExternalDamping,
}

#[derive(Debug, Deserialize)]
struct ExternalDamping {
    kind: String,
    base_per_second: f64,
    slope_per_hz: f64,
}

#[derive(Debug, Deserialize)]
struct ExternalBoundary {
    kind: String,
    patch_size_m: f64,
}

#[derive(Debug, Deserialize)]
struct ExternalModalBasis {
    file: String,
    sha256: String,
    normalization: String,
    sign_rule: String,
}

#[derive(Debug, Deserialize)]
struct ExternalPoint {
    point_m: [f64; 3],
    direction: [f64; 3],
    mesh_node_index: usize,
}

#[derive(Debug, Deserialize)]
struct ExternalMode {
    index: usize,
    undamped_frequency_hz: f64,
    damped_frequency_hz: f64,
    damping_per_second: f64,
    t60_seconds: f64,
    relative_eigen_residual: f64,
}

#[derive(Debug, Deserialize)]
struct ExternalStrike {
    id: String,
    role: String,
    point_m: [f64; 3],
    direction: [f64; 3],
    mesh_node_index: usize,
}

#[derive(Debug, Deserialize)]
struct ExternalImpulse {
    id: String,
    newton_seconds: f64,
}

#[derive(Debug, Deserialize)]
struct ExternalHeldout {
    position_interpolator: String,
    train_force_ids: Vec<String>,
    force_holdout_id: String,
    position_holdout_strike_id: String,
}

#[derive(Debug, Deserialize)]
struct ExternalCondition {
    id: String,
    split: String,
    strike_id: String,
    force_id: String,
    impulse_newton_seconds: f64,
    amplitudes: Vec<f64>,
}

#[derive(Debug, Serialize)]
struct CorpusReport {
    schema: &'static str,
    status: &'static str,
    claim: &'static str,
    fallback: &'static str,
    source_profile: String,
    source_profile_sha256: String,
    source_recipe_sha256: String,
    solver: SolverReport,
    object: ObjectReport,
    sample_rate_hz: u32,
    frame_count: usize,
    mode_count: usize,
    condition_count: usize,
    split_counts: BTreeMap<String, usize>,
    repeated_render_identical: bool,
    q30_transfer: Q30TransferReport,
    force_control: ForceControlReport,
    position_control: PositionControlReport,
    heldout_interpolation: HeldoutInterpolationReport,
    audition: AuditionReport,
    quality_manifest_file: &'static str,
}

#[derive(Debug, Serialize)]
struct SolverReport {
    id: String,
    python: String,
    numpy: String,
    scipy: String,
    method: String,
    maximum_relative_eigen_residual: f64,
}

#[derive(Debug, Serialize)]
struct ObjectReport {
    object_id: String,
    geometry_kind: String,
    outer_size_m: [f64; 3],
    wall_thickness_m: f64,
    bottom_thickness_m: f64,
    cell_size_m: f64,
    node_count: usize,
    tetrahedron_count: usize,
    fixed_node_count: usize,
    density_kg_m3: f64,
    youngs_modulus_pa: f64,
    poisson_ratio: f64,
    minimum_mode_frequency_hz: f64,
    maximum_mode_frequency_hz: f64,
}

#[derive(Debug, Serialize)]
struct Q30TransferReport {
    status: &'static str,
    renderer: &'static str,
    maximum_absolute_residual: f64,
    maximum_rms_residual: f64,
    minimum_correlation: f64,
    acceptance: Q30AcceptanceReport,
}

#[derive(Debug, Serialize)]
struct Q30AcceptanceReport {
    maximum_absolute_residual_at_most: f64,
    rms_residual_at_most: f64,
    correlation_at_least: f64,
}

#[derive(Debug, Serialize)]
struct ForceControlReport {
    status: &'static str,
    relation: &'static str,
    maximum_modal_scale_relative_error: f64,
    maximum_waveform_rms_ratio_relative_error: f64,
}

#[derive(Debug, Serialize)]
struct PositionControlReport {
    status: &'static str,
    relation: &'static str,
    pair_count: usize,
    minimum_cosine_distance: f64,
    maximum_cosine_distance: f64,
    required_minimum_cosine_distance: f64,
}

#[derive(Debug, Serialize)]
struct HeldoutInterpolationReport {
    status: &'static str,
    claim: &'static str,
    strike_id: String,
    weights: BTreeMap<String, f64>,
    conditions: Vec<HeldoutConditionReport>,
}

#[derive(Debug, Serialize)]
struct HeldoutConditionReport {
    condition_id: String,
    force_id: String,
    residual: ResidualReport,
}

#[derive(Clone, Debug, Serialize)]
struct ResidualReport {
    maximum_absolute: f64,
    rms: f64,
    signal_to_noise_db: Option<f64>,
    correlation: f64,
}

#[derive(Debug, Serialize)]
struct AuditionReport {
    heldout_exact_then_idw_file: &'static str,
    heldout_exact_then_idw_sha256: String,
    heldout_force_ladder_file: &'static str,
    heldout_force_ladder_sha256: String,
    order: &'static str,
}

#[derive(Debug, Serialize)]
struct QualityManifest {
    schema: &'static str,
    split: &'static str,
    entries: Vec<QualityManifestEntry>,
}

#[derive(Debug, Serialize)]
struct QualityManifestEntry {
    id: String,
    object_id: String,
    material: &'static str,
    impact_position: String,
    force_band: String,
    candidate: QualityAudioRef,
    reference: Option<QualityAudioRef>,
}

#[derive(Debug, Serialize)]
struct QualityAudioRef {
    path: String,
    sha256: String,
}

#[derive(Clone)]
struct RenderedCondition {
    reference: Vec<f64>,
    reference_file: String,
    reference_sha256: String,
    repeated_q30: bool,
}

pub(super) fn run(root: &Path, request: &Request) -> Result<(), String> {
    let profile_path = canonical_external_input(root, &request.profile, "solver profile")?;
    let profile_bytes = bounded_read(&profile_path, MAX_PROFILE_BYTES, "solver profile")?;
    let profile: ExternalProfile = serde_json::from_slice(&profile_bytes)
        .map_err(|error| format!("parse {}: {error}", profile_path.display()))?;
    let profile_parent = profile_path
        .parent()
        .ok_or_else(|| "solver profile has no parent directory".to_owned())?;
    validate_profile(&profile)?;
    validate_external_artifacts(profile_parent, &profile)?;

    let output = resolve_external_output(root, &request.output)?;
    require_empty_output(&output)?;
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;

    let mut rendered = BTreeMap::new();
    let mut manifest_entries = Vec::new();
    let mut repeated_render_identical = true;
    let mut maximum_absolute_residual = 0.0_f64;
    let mut maximum_rms_residual = 0.0_f64;
    let mut minimum_correlation = 1.0_f64;
    for condition in &profile.conditions {
        let modes = condition_modes(&profile, condition, 1.0)?;
        let reference = render_offline_modal_recurrence_unscaled(
            profile.sample_rate_hz,
            profile.frame_count,
            &modes,
            &[],
        )
        .map_err(|error| error.to_string())?;
        repeated_render_identical &= reference
            == render_offline_modal_recurrence_unscaled(
                profile.sample_rate_hz,
                profile.frame_count,
                &modes,
                &[],
            )
            .map_err(|error| error.to_string())?;
        validate_pcm_range(&reference, &condition.id)?;

        let bank = cook_offline_q30_modal_bank(profile.sample_rate_hz, &modes, &[])
            .map_err(|error| error.to_string())?;
        let q30_raw = render_offline_q30_modal_recurrence(profile.frame_count, &bank)
            .map_err(|error| error.to_string())?;
        let repeated_q30 = q30_raw
            == render_offline_q30_modal_recurrence(profile.frame_count, &bank)
                .map_err(|error| error.to_string())?;
        let q30 = decode_offline_q30_samples(&q30_raw).map_err(|error| error.to_string())?;
        validate_pcm_range(&q30, &condition.id)?;
        let residual = calculate_residual(&reference, &q30)?;
        maximum_absolute_residual = maximum_absolute_residual.max(residual.maximum_absolute);
        maximum_rms_residual = maximum_rms_residual.max(residual.rms);
        minimum_correlation = minimum_correlation.min(residual.correlation);

        let reference_file = format!("reference-{}.wav", condition.id);
        let q30_file = format!("candidate-q30-{}.wav", condition.id);
        let reference_wav = encode_float32_mono_wav(profile.sample_rate_hz, &reference)?;
        let q30_wav = encode_float32_mono_wav(profile.sample_rate_hz, &q30)?;
        write_output(&output, &reference_file, &reference_wav)?;
        write_output(&output, &q30_file, &q30_wav)?;
        let reference_sha256 = sha256_hex(&reference_wav);
        let q30_sha256 = sha256_hex(&q30_wav);
        manifest_entries.push(QualityManifestEntry {
            id: format!("q30--{}", condition.id),
            object_id: profile.object_id.clone(),
            material: "glass",
            impact_position: condition.strike_id.clone(),
            force_band: condition.force_id.clone(),
            candidate: QualityAudioRef {
                path: output.join(&q30_file).display().to_string(),
                sha256: q30_sha256.clone(),
            },
            reference: Some(QualityAudioRef {
                path: output.join(&reference_file).display().to_string(),
                sha256: reference_sha256.clone(),
            }),
        });
        rendered.insert(
            condition.id.clone(),
            RenderedCondition {
                reference,
                reference_file,
                reference_sha256,
                repeated_q30,
            },
        );
    }

    let force_control = evaluate_force_control(&profile, &rendered)?;
    let position_control = evaluate_position_control(&profile)?;
    let (heldout_interpolation, heldout_auditions) =
        render_heldout_interpolation(&profile, &rendered, &output, &mut manifest_entries)?;
    let audition = write_auditions(&profile, &rendered, &heldout_auditions, &output)?;

    manifest_entries.sort_by(|left, right| left.id.cmp(&right.id));
    let manifest = QualityManifest {
        schema: QUALITY_MANIFEST_SCHEMA,
        split: "controlled-glass-position-force-holdout-v0",
        entries: manifest_entries,
    };
    let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?;
    write_output(&output, "quality-manifest.json", &manifest_json)?;

    let q30_status = if maximum_absolute_residual <= Q30_MAXIMUM_ABSOLUTE_RESIDUAL_LIMIT
        && maximum_rms_residual <= Q30_RMS_RESIDUAL_LIMIT
        && minimum_correlation >= Q30_CORRELATION_MINIMUM
        && rendered.values().all(|condition| condition.repeated_q30)
    {
        "PASS"
    } else {
        "RESIDUAL_TOO_LARGE"
    };
    let status = if repeated_render_identical
        && q30_status == "PASS"
        && force_control.status == "PASS"
        && position_control.status == "PASS"
    {
        "PASS"
    } else {
        "CONTROL_RELATION_FAILED"
    };
    let split_counts = profile.conditions.iter().fold(
        BTreeMap::<String, usize>::new(),
        |mut counts, condition| {
            *counts.entry(condition.split.clone()).or_default() += 1;
            counts
        },
    );
    let maximum_relative_eigen_residual = profile
        .modes
        .iter()
        .map(|mode| mode.relative_eigen_residual)
        .fold(0.0_f64, f64::max);
    let report = CorpusReport {
        schema: REPORT_SCHEMA,
        status,
        claim: REQUIRED_CLAIM,
        fallback: "ordinary authored clip path and the unchanged Glass-H laboratory control",
        source_profile: profile_path.display().to_string(),
        source_profile_sha256: sha256_hex(&profile_bytes),
        source_recipe_sha256: profile.source_recipe.sha256,
        solver: SolverReport {
            id: profile.solver.id,
            python: profile.solver.python,
            numpy: profile.solver.numpy,
            scipy: profile.solver.scipy,
            method: profile.solver.method,
            maximum_relative_eigen_residual,
        },
        object: ObjectReport {
            object_id: profile.object_id,
            geometry_kind: profile.geometry.kind,
            outer_size_m: profile.geometry.outer_size_m,
            wall_thickness_m: profile.geometry.wall_thickness_m,
            bottom_thickness_m: profile.geometry.bottom_thickness_m,
            cell_size_m: profile.geometry.cell_size_m,
            node_count: profile.geometry.node_count,
            tetrahedron_count: profile.geometry.tetrahedron_count,
            fixed_node_count: profile.geometry.fixed_node_count,
            density_kg_m3: profile.material.density_kg_m3,
            youngs_modulus_pa: profile.material.youngs_modulus_pa,
            poisson_ratio: profile.material.poisson_ratio,
            minimum_mode_frequency_hz: profile.modes[0].undamped_frequency_hz,
            maximum_mode_frequency_hz: profile.modes[profile.modes.len() - 1].undamped_frequency_hz,
        },
        sample_rate_hz: profile.sample_rate_hz,
        frame_count: profile.frame_count,
        mode_count: profile.modes.len(),
        condition_count: profile.conditions.len(),
        split_counts,
        repeated_render_identical,
        q30_transfer: Q30TransferReport {
            status: q30_status,
            renderer: "engine-owned signed-Q30 second-order recurrence; no per-condition normalization",
            maximum_absolute_residual,
            maximum_rms_residual,
            minimum_correlation,
            acceptance: Q30AcceptanceReport {
                maximum_absolute_residual_at_most: Q30_MAXIMUM_ABSOLUTE_RESIDUAL_LIMIT,
                rms_residual_at_most: Q30_RMS_RESIDUAL_LIMIT,
                correlation_at_least: Q30_CORRELATION_MINIMUM,
            },
        },
        force_control,
        position_control,
        heldout_interpolation,
        audition,
        quality_manifest_file: "quality-manifest.json",
    };
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    write_output(&output, "report.json", &report_json)?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    if status != "PASS" {
        return Err("controlled glass corpus failed a predeclared numeric control".to_owned());
    }
    Ok(())
}

fn validate_profile(profile: &ExternalProfile) -> Result<(), String> {
    if profile.schema != PROFILE_SCHEMA || profile.claim != REQUIRED_CLAIM {
        return Err("unsupported controlled glass profile schema or claim".to_owned());
    }
    if profile.object_id.is_empty()
        || profile.sample_rate_hz != 48_000
        || profile.frame_count == 0
        || profile.frame_count > profile.sample_rate_hz as usize * 2
        || !finite_positive(profile.global_amplitude_gain)
        || !finite_positive(profile.target_global_peak)
        || profile.target_global_peak >= 0.95
    {
        return Err("controlled glass profile identity or render bounds are invalid".to_owned());
    }
    validate_geometry(profile)?;
    validate_material(profile)?;
    validate_modes(profile)?;
    validate_strikes_and_conditions(profile)?;
    validate_point(&profile.pickup.point_m, &profile.pickup.direction, "pickup")?;
    if profile.pickup.mesh_node_index >= profile.geometry.node_count {
        return Err("pickup mesh node is outside the geometry".to_owned());
    }
    Ok(())
}

fn validate_geometry(profile: &ExternalProfile) -> Result<(), String> {
    let geometry = &profile.geometry;
    if geometry.kind != "open_rectangular_vessel"
        || geometry.axes != "+X right, +Y up, +Z forward"
        || geometry.tetrahedra_per_cell != 6
        || geometry.node_count == 0
        || geometry.tetrahedron_count == 0
        || geometry.fixed_node_count < 4
        || geometry.fixed_node_count > geometry.node_count
        || geometry
            .outer_size_m
            .iter()
            .any(|value| !finite_positive(*value))
        || !finite_positive(geometry.wall_thickness_m)
        || !finite_positive(geometry.bottom_thickness_m)
        || !finite_positive(geometry.cell_size_m)
        || geometry.wall_thickness_m * 2.0 >= geometry.outer_size_m[0]
        || geometry.wall_thickness_m * 2.0 >= geometry.outer_size_m[2]
        || geometry.bottom_thickness_m >= geometry.outer_size_m[1]
    {
        return Err("controlled glass geometry is invalid or unbounded".to_owned());
    }
    if profile.boundary.kind != "fixed_bottom_back_left_patch"
        || !finite_positive(profile.boundary.patch_size_m)
    {
        return Err("controlled glass boundary is invalid".to_owned());
    }
    Ok(())
}

fn validate_material(profile: &ExternalProfile) -> Result<(), String> {
    let material = &profile.material;
    if material.name.is_empty()
        || !finite_positive(material.density_kg_m3)
        || !finite_positive(material.youngs_modulus_pa)
        || !material.poisson_ratio.is_finite()
        || !(0.0..0.49).contains(&material.poisson_ratio)
        || material.damping.kind != "explicit_frequency_calibration"
        || !finite_positive(material.damping.base_per_second)
        || !finite_positive(material.damping.slope_per_hz)
    {
        return Err("controlled glass material or damping calibration is invalid".to_owned());
    }
    Ok(())
}

fn validate_modes(profile: &ExternalProfile) -> Result<(), String> {
    if profile.modes.is_empty() || profile.modes.len() > MAX_MODE_COUNT {
        return Err("controlled glass mode count is invalid or unbounded".to_owned());
    }
    let mut previous = 0.0_f64;
    for (index, mode) in profile.modes.iter().enumerate() {
        let expected_t60 = 1000.0_f64.ln() / mode.damping_per_second;
        let expected_damped = ((2.0 * std::f64::consts::PI * mode.undamped_frequency_hz).powi(2)
            - mode.damping_per_second.powi(2))
        .sqrt()
            / (2.0 * std::f64::consts::PI);
        if mode.index != index
            || !finite_positive(mode.undamped_frequency_hz)
            || !finite_positive(mode.damped_frequency_hz)
            || mode.undamped_frequency_hz <= previous
            || mode.damped_frequency_hz >= f64::from(profile.sample_rate_hz) * 0.5
            || !finite_positive(mode.damping_per_second)
            || !finite_positive(mode.t60_seconds)
            || (mode.t60_seconds - expected_t60).abs() > expected_t60 * 1.0e-8
            || (mode.damped_frequency_hz - expected_damped).abs()
                > mode.damped_frequency_hz * 1.0e-8
            || !mode.relative_eigen_residual.is_finite()
            || !(0.0..=1.0e-5).contains(&mode.relative_eigen_residual)
        {
            return Err(format!("controlled glass mode {index} is inconsistent"));
        }
        previous = mode.undamped_frequency_hz;
    }
    Ok(())
}

fn validate_strikes_and_conditions(profile: &ExternalProfile) -> Result<(), String> {
    if profile.strikes.len() < 3
        || profile.conditions.is_empty()
        || profile.conditions.len() > MAX_CONDITION_COUNT
        || profile.impulses.len() != 3
        || profile.heldout.position_interpolator != "inverse_distance_squared"
        || profile.heldout.train_force_ids != ["low", "high"]
        || profile.heldout.force_holdout_id != "medium"
    {
        return Err("controlled glass split protocol is invalid".to_owned());
    }
    let impulse_ids = profile
        .impulses
        .iter()
        .map(|impulse| impulse.id.as_str())
        .collect::<Vec<_>>();
    if impulse_ids != ["low", "medium", "high"]
        || profile
            .impulses
            .iter()
            .any(|impulse| !finite_positive(impulse.newton_seconds))
        || profile.impulses[0].newton_seconds >= profile.impulses[1].newton_seconds
        || profile.impulses[1].newton_seconds >= profile.impulses[2].newton_seconds
    {
        return Err("controlled glass impulse levels are invalid".to_owned());
    }
    let impulse_map = profile
        .impulses
        .iter()
        .map(|impulse| (impulse.id.as_str(), impulse.newton_seconds))
        .collect::<BTreeMap<_, _>>();
    let mut strike_ids = BTreeSet::new();
    let mut heldout_count = 0_usize;
    for strike in &profile.strikes {
        validate_point(&strike.point_m, &strike.direction, &strike.id)?;
        if !strike_ids.insert(strike.id.as_str())
            || strike.mesh_node_index >= profile.geometry.node_count
            || !matches!(strike.role.as_str(), "train" | "heldout")
        {
            return Err("controlled glass strike identity or bounds are invalid".to_owned());
        }
        heldout_count += usize::from(strike.role == "heldout");
    }
    if heldout_count != 1
        || !strike_ids.contains(profile.heldout.position_holdout_strike_id.as_str())
    {
        return Err("controlled glass profile must have one declared heldout strike".to_owned());
    }

    let mut previous_id = None;
    let mut condition_keys = BTreeSet::new();
    let strike_roles = profile
        .strikes
        .iter()
        .map(|strike| (strike.id.as_str(), strike.role.as_str()))
        .collect::<BTreeMap<_, _>>();
    for condition in &profile.conditions {
        if previous_id.is_some_and(|previous| previous >= condition.id.as_str()) {
            return Err("controlled glass conditions must be strictly sorted".to_owned());
        }
        previous_id = Some(condition.id.as_str());
        let impulse = impulse_map
            .get(condition.force_id.as_str())
            .ok_or_else(|| format!("unknown condition force: {}", condition.force_id))?;
        let role = strike_roles
            .get(condition.strike_id.as_str())
            .ok_or_else(|| format!("unknown condition strike: {}", condition.strike_id))?;
        let expected_split = expected_split(role, &condition.force_id);
        if condition.id != format!("{}--{}", condition.strike_id, condition.force_id)
            || condition.split != expected_split
            || !condition_keys.insert((condition.strike_id.as_str(), condition.force_id.as_str()))
            || (condition.impulse_newton_seconds - impulse).abs() > f64::EPSILON
            || condition.amplitudes.len() != profile.modes.len()
            || condition
                .amplitudes
                .iter()
                .any(|amplitude| !amplitude.is_finite() || amplitude.abs() > 16.0)
            || condition
                .amplitudes
                .iter()
                .all(|amplitude| *amplitude == 0.0)
        {
            return Err(format!(
                "controlled glass condition {} is invalid",
                condition.id
            ));
        }
    }
    if condition_keys.len() != profile.strikes.len() * profile.impulses.len() {
        return Err("controlled glass condition matrix is incomplete".to_owned());
    }
    Ok(())
}

fn validate_point(point: &[f64; 3], direction: &[f64; 3], name: &str) -> Result<(), String> {
    if point.iter().any(|value| !value.is_finite())
        || direction.iter().any(|value| !value.is_finite())
        || (vector_norm(direction) - 1.0).abs() > 1.0e-12
    {
        return Err(format!(
            "controlled glass point/direction is invalid: {name}"
        ));
    }
    Ok(())
}

fn validate_external_artifacts(parent: &Path, profile: &ExternalProfile) -> Result<(), String> {
    let artifacts = [
        (&profile.source_recipe.file, &profile.source_recipe.sha256),
        (&profile.geometry.node_file, &profile.geometry.node_sha256),
        (
            &profile.geometry.tetrahedron_file,
            &profile.geometry.tetrahedron_sha256,
        ),
        (
            &profile.geometry.fixed_node_file,
            &profile.geometry.fixed_node_sha256,
        ),
        (&profile.modal_basis.file, &profile.modal_basis.sha256),
    ];
    for (file, expected_hash) in artifacts {
        validate_sibling_file_name(file)?;
        if !is_lower_hex(expected_hash, 64) {
            return Err(format!("invalid artifact SHA-256 for {file}"));
        }
        let path = canonical_sibling(parent, file)?;
        let bytes = bounded_read(&path, MAX_ARTIFACT_BYTES, "corpus artifact")?;
        let actual = sha256_hex(&bytes);
        if actual != *expected_hash {
            return Err(format!(
                "corpus artifact hash mismatch for {file}: expected {expected_hash}, got {actual}"
            ));
        }
    }
    if profile.modal_basis.normalization.is_empty()
        || profile.modal_basis.sign_rule.is_empty()
        || profile.solver.id.is_empty()
        || profile.solver.method.is_empty()
    {
        return Err("controlled glass solver lineage is incomplete".to_owned());
    }
    Ok(())
}

fn condition_modes(
    profile: &ExternalProfile,
    condition: &ExternalCondition,
    scale: f64,
) -> Result<Vec<OfflineModalMode>, String> {
    if !scale.is_finite() {
        return Err("condition amplitude scale is non-finite".to_owned());
    }
    Ok(profile
        .modes
        .iter()
        .zip(&condition.amplitudes)
        .map(|(mode, amplitude)| OfflineModalMode {
            damped_frequency_hz: mode.damped_frequency_hz,
            damping_per_second: mode.damping_per_second,
            amplitude: amplitude * scale,
        })
        .collect())
}

#[cfg(test)]
mod tests;
