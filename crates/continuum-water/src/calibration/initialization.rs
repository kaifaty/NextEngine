#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

use crate::audit::{
    AuditFluidInput, boundary_input_root, fluid_input_root, independent_zero_velocity_settling,
    production_fluid_input,
};
use crate::error::{
    AUDIT_INVALID, AUDIT_MISMATCH, REPORT_CAPACITY_EXCEEDED, SCENARIO_INVALID, WaterError,
};
use crate::hash::{self, FrozenRoots};
use crate::model::{CanonicalSample, Geometry, StorageOrder, Vec3i};
use crate::oracle::command::{
    tool_commit, tool_tree_state, validate_output_path, validate_report_capacity,
};
use crate::{profile, scenario, solver};

use super::candidate::{
    FreefallControl, HydroSoak, audit_boundary_input, compare_freefall, ghost_cell_shell,
    run_hydro_soak,
};
use super::{
    SETTLING_DENSITY_CEILING, SETTLING_MAX_PASSES, SETTLING_MAXIMUM_DISPLACEMENT_UM,
    SETTLING_READY_STREAK, SETTLING_TREND_TRANSITIONS, SettlingComputation, SettlingPassTrace,
    SettlingTrace,
};

const CANDIDATE_ID: &str = "zero-velocity-settle-v1";

#[derive(Serialize)]
struct InitializationEnvelope<'a> {
    schema_version: u32,
    status: &'a str,
    command: &'a str,
    details: &'a InitializationReport,
}

#[derive(Serialize)]
struct InitializationReport {
    report_schema: String,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: String,
    build_profile: String,
    build_rustflags: String,
    roots: InitializationRoots,
    candidate_id: String,
    candidate_definition: String,
    classification: String,
    corpus_credit: String,
    selection_status: String,
    comparison: String,
    first_mismatch: Option<String>,
    local_disposition: String,
    conclusion: String,
    product_check: String,
    settling_configuration: SettlingConfiguration,
    fluid_sample_count: usize,
    candidate_boundary_sample_count: usize,
    input_fluid_root: String,
    candidate_boundary_input_root: String,
    production_last_state_root: String,
    independent_last_state_root: String,
    last_state_classification: String,
    production_generator: SettlingTrace,
    independent_generator: SettlingTrace,
    post_generator_hydro_soak: HydroSoak,
    freefall_control: FreefallControl,
    timing_classification: String,
    wall_clock_nanoseconds: u64,
}

#[derive(Serialize)]
struct InitializationRoots {
    original_w0b_document_root: String,
    original_float_profile_root: String,
    original_corpus_root: String,
    original_execution_profile_root: String,
    original_hydro_scenario_root: String,
    original_freefall_scenario_root: String,
}

#[derive(Serialize)]
struct SettlingConfiguration {
    boundary_candidate: String,
    velocity_policy: String,
    maximum_passes: u8,
    diagnostic_density_ceiling: u8,
    success_maximum_displacement_um: i64,
    success_required_consecutive_passes: u8,
    adverse_trend_transitions: usize,
    production_density_ceiling: u8,
}

pub(crate) fn run_xtask(
    repository_root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let candidate_flag = arguments.next().ok_or_else(argument_error)?;
    let candidate = arguments.next().ok_or_else(argument_error)?;
    let output_flag = arguments.next().ok_or_else(argument_error)?;
    let output = arguments.next().ok_or_else(argument_error)?;
    if candidate_flag != "--candidate"
        || candidate != CANDIDATE_ID
        || output_flag != "--output"
        || arguments.next().is_some()
    {
        return Err(argument_error());
    }
    let output = PathBuf::from(output);
    validate_output_path(repository_root, &output)?;
    profile::validate_execution_profile()?;
    profile::validate_float_environment()?;
    let roots = FrozenRoots::verify(repository_root)?;
    let hydro = scenario::find("CW-HYDRO-001")?;
    let hydro_root = scenario::root_for(&hydro, &roots)?;
    let freefall = scenario::find("CW-FREEFALL-001")?;
    let freefall_root = scenario::root_for(&freefall, &roots)?;
    let initial_samples = scenario::initial_samples(&hydro, StorageOrder::Reverse)?;
    let boundary = ghost_cell_shell(hydro.geometry)?;
    let boundary_input = audit_boundary_input(&boundary)?;
    let mut initial_fluid = production_fluid_input(&initial_samples)?;
    initial_fluid.sort_unstable_by_key(|sample| sample.id);

    let started = Instant::now();
    let production = production_settling(
        &initial_samples,
        hydro.geometry,
        &boundary,
        &roots.execution_profile,
        &hydro_root,
    )?;
    let independent = independent_zero_velocity_settling()?;
    let mismatch = settling_mismatch(&production, &independent);
    let production_last_state_root = fluid_input_root(&production.final_fluid);
    let independent_last_state_root = fluid_input_root(&independent.final_fluid);
    let post_generator_samples = canonical_samples(&production.final_fluid)?;
    let post_generator_hydro_soak = run_hydro_soak(
        &post_generator_samples,
        hydro.geometry,
        &boundary,
        &roots.execution_profile,
        &hydro_root,
        20,
    )?;
    let freefall_control = compare_freefall(&freefall, &roots.execution_profile, &freefall_root)?;
    let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let comparison = if mismatch.is_none() {
        "EXACT_MATCH"
    } else {
        "MISMATCH"
    };
    let survived = mismatch.is_none()
        && production.trace.status == "CONVERGED"
        && post_generator_hydro_soak.status == "SOAK_PASS"
        && freefall_control.status == "EXACT_MATCH";
    let local_disposition = if survived {
        "LOCAL_DISCRIMINATOR_SURVIVED"
    } else {
        "CANDIDATE_REJECTED"
    };
    let initial_tree_state = tool_tree_state(repository_root);
    let mut report = InitializationReport {
        report_schema: "nextengine.continuum-water.hydro-initialization-candidate.v1".to_owned(),
        tool_commit: tool_commit(repository_root),
        tool_tree_state: initial_tree_state,
        toolchain_target: env!("WATER_BUILD_TARGET").to_owned(),
        build_profile: env!("WATER_BUILD_PROFILE").to_owned(),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        roots: InitializationRoots {
            original_w0b_document_root: hash::hex(&roots.document),
            original_float_profile_root: hash::hex(&roots.float_profile),
            original_corpus_root: hash::hex(&roots.corpus),
            original_execution_profile_root: hash::hex(&roots.execution_profile),
            original_hydro_scenario_root: hash::hex(&hydro_root),
            original_freefall_scenario_root: hash::hex(&freefall_root),
        },
        candidate_id: CANDIDATE_ID.to_owned(),
        candidate_definition: "start from the regular hydro lattice and ghost-cell-shell-v1; before each bounded relaxation pass set every canonical velocity to zero, run one serial DFSPH step with ceiling 160, publish positions, discard velocity, and stop on convergence, an adverse trend, a typed solver failure or pass 24".to_owned(),
        classification: "COUNTERFACTUAL_INITIALIZATION_CANDIDATE".to_owned(),
        corpus_credit: "NO_CORPUS_CREDIT".to_owned(),
        selection_status: "NOT_SELECTED".to_owned(),
        comparison: comparison.to_owned(),
        first_mismatch: mismatch.clone(),
        local_disposition: local_disposition.to_owned(),
        conclusion: if survived {
            "the bounded zero-velocity settling generator survives its local discriminator; external aggregate comparison, successor roots and the full corpus are still required before selection".to_owned()
        } else {
            "the zero-velocity settling generator fails its local discriminator and must not define a successor initial state".to_owned()
        },
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned(),
        settling_configuration: SettlingConfiguration {
            boundary_candidate: "ghost-cell-shell-v1".to_owned(),
            velocity_policy: "set all canonical velocities to zero before every pass and discard every accepted pass velocity".to_owned(),
            maximum_passes: SETTLING_MAX_PASSES,
            diagnostic_density_ceiling: SETTLING_DENSITY_CEILING,
            success_maximum_displacement_um: SETTLING_MAXIMUM_DISPLACEMENT_UM,
            success_required_consecutive_passes: SETTLING_READY_STREAK,
            adverse_trend_transitions: SETTLING_TREND_TRANSITIONS,
            production_density_ceiling: 20,
        },
        fluid_sample_count: initial_fluid.len(),
        candidate_boundary_sample_count: boundary_input.len(),
        input_fluid_root: fluid_input_root(&initial_fluid),
        candidate_boundary_input_root: boundary_input_root(&boundary_input),
        production_last_state_root,
        independent_last_state_root,
        last_state_classification:
            "DIAGNOSTIC_LAST_ACCEPTED_PASS / NOT_A_GENERATOR_OUTPUT / NO_CORPUS_CREDIT"
                .to_owned(),
        production_generator: production.trace,
        independent_generator: independent.trace,
        post_generator_hydro_soak,
        freefall_control,
        timing_classification: "DIAGNOSTIC_ONLY".to_owned(),
        wall_clock_nanoseconds: elapsed,
    };
    let ending_tree_state = tool_tree_state(repository_root);
    if ending_tree_state != report.tool_tree_state {
        report.tool_tree_state = format!("{}->{ending_tree_state}", report.tool_tree_state);
    }
    write_report(&output, &report, mismatch.is_none())?;
    if let Some(detail) = mismatch {
        return Err(WaterError::new(
            AUDIT_MISMATCH,
            format!(
                "{detail}; initialization report written to {}",
                output.display()
            ),
        ));
    }
    let command = serde_json::json!({
        "schema_version": 1,
        "status": "REPORT_ONLY",
        "command": "continuum water evaluate-hydro-initialization",
        "details": {
            "report": output.display().to_string(),
            "candidate": report.candidate_id,
            "comparison": report.comparison,
            "generator_status": report.production_generator.status,
            "generator_completed_passes": report.production_generator.completed_passes,
            "generator_terminal_code": report.production_generator.terminal_code,
            "post_generator_hydro_soak": report.post_generator_hydro_soak.status,
            "freefall_control": report.freefall_control.status,
            "local_disposition": report.local_disposition,
            "selection_status": report.selection_status,
            "product_check": report.product_check,
        }
    });
    serde_json::to_string(&command).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize initialization command report: {error}"),
        )
    })
}

fn argument_error() -> WaterError {
    WaterError::new(
        SCENARIO_INVALID,
        format!(
            "evaluate-hydro-initialization requires --candidate {CANDIDATE_ID} --output <absolute-path>"
        ),
    )
}

fn production_settling(
    initial_samples: &[CanonicalSample],
    geometry: Geometry,
    boundary: &[crate::boundary::BoundarySample],
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<SettlingComputation, WaterError> {
    let mut samples = zero_velocity_samples(initial_samples)?;
    let mut final_fluid = production_fluid_input(&samples)?;
    final_fluid.sort_unstable_by_key(|sample| sample.id);
    let mut passes = Vec::new();
    passes
        .try_reserve_exact(usize::from(SETTLING_MAX_PASSES))
        .map_err(|error| {
            WaterError::new(
                AUDIT_INVALID,
                format!("settling transcript allocation failed: {error}"),
            )
        })?;
    let mut ready_streak = 0_u8;
    for pass in 1..=SETTLING_MAX_PASSES {
        let prior = samples.clone();
        let (frame, _) =
            solver::initial_frame(samples, geometry, boundary, execution_root, scenario_root)?;
        let outcome = match solver::counterfactual_substep(
            &frame,
            geometry,
            boundary,
            execution_root,
            scenario_root,
            SETTLING_DENSITY_CEILING,
        ) {
            Ok(outcome) => outcome,
            Err(error) => {
                return Ok(SettlingComputation {
                    trace: SettlingTrace {
                        status: "GENERATOR_REJECTED".to_owned(),
                        requested_maximum_passes: SETTLING_MAX_PASSES,
                        completed_passes: pass - 1,
                        terminal_pass: Some(pass),
                        terminal_code: error.code().to_owned(),
                        terminal_detail: error.detail().to_owned(),
                        passes,
                    },
                    final_fluid,
                });
            }
        };
        let maximum_displacement_um = maximum_displacement(&prior, &outcome.frame.samples)?;
        samples = zero_velocity_samples(&outcome.frame.samples)?;
        final_fluid = production_fluid_input(&samples)?;
        final_fluid.sort_unstable_by_key(|sample| sample.id);
        let production_ceiling_ready = outcome.summary.density_iterations <= 20
            && maximum_displacement_um <= SETTLING_MAXIMUM_DISPLACEMENT_UM;
        if production_ceiling_ready {
            ready_streak = ready_streak
                .checked_add(1)
                .ok_or_else(|| WaterError::new(AUDIT_INVALID, "settling ready streak overflow"))?;
        } else {
            ready_streak = 0;
        }
        passes.push(SettlingPassTrace {
            pass,
            density_iterations: outcome.summary.density_iterations,
            density_error_ppb: outcome.summary.density_error_ppb,
            maximum_displacement_um,
            maximum_penetration_um: outcome.summary.maximum_penetration_um,
            centre_of_mass_y_um: outcome.summary.centre_of_mass_um.y,
            zero_velocity_state_root: fluid_input_root(&final_fluid),
            production_ceiling_ready,
        });
        if ready_streak >= SETTLING_READY_STREAK {
            return Ok(SettlingComputation {
                trace: SettlingTrace {
                    status: "CONVERGED".to_owned(),
                    requested_maximum_passes: SETTLING_MAX_PASSES,
                    completed_passes: pass,
                    terminal_pass: Some(pass),
                    terminal_code: "COMPLETED".to_owned(),
                    terminal_detail: format!(
                        "settling met the production ceiling and displacement target for {SETTLING_READY_STREAK} consecutive passes"
                    ),
                    passes,
                },
                final_fluid,
            });
        }
        if has_adverse_trend(&passes) {
            return Ok(SettlingComputation {
                trace: SettlingTrace {
                    status: "GENERATOR_REJECTED".to_owned(),
                    requested_maximum_passes: SETTLING_MAX_PASSES,
                    completed_passes: pass,
                    terminal_pass: Some(pass),
                    terminal_code: "SETTLING_ADVERSE_TREND".to_owned(),
                    terminal_detail: format!(
                        "density iteration demand and penetration increased for {SETTLING_TREND_TRANSITIONS} consecutive transitions"
                    ),
                    passes,
                },
                final_fluid,
            });
        }
    }
    Ok(SettlingComputation {
        trace: SettlingTrace {
            status: "GENERATOR_REJECTED".to_owned(),
            requested_maximum_passes: SETTLING_MAX_PASSES,
            completed_passes: SETTLING_MAX_PASSES,
            terminal_pass: Some(SETTLING_MAX_PASSES),
            terminal_code: "SETTLING_PASS_LIMIT_EXHAUSTED".to_owned(),
            terminal_detail: format!(
                "settling did not meet its convergence rule in {SETTLING_MAX_PASSES} passes"
            ),
            passes,
        },
        final_fluid,
    })
}

fn zero_velocity_samples(samples: &[CanonicalSample]) -> Result<Vec<CanonicalSample>, WaterError> {
    let mut result = Vec::new();
    result.try_reserve_exact(samples.len()).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("zero-velocity state allocation failed: {error}"),
        )
    })?;
    for sample in samples {
        result.push(CanonicalSample {
            id: sample.id,
            position_um: sample.position_um,
            velocity_um_s: Vec3i::new(0, 0, 0),
        });
    }
    result.sort_unstable_by_key(|sample| sample.id);
    Ok(result)
}

fn canonical_samples(fluid: &[AuditFluidInput]) -> Result<Vec<CanonicalSample>, WaterError> {
    let mut result = Vec::new();
    result.try_reserve_exact(fluid.len()).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("settled canonical state allocation failed: {error}"),
        )
    })?;
    for sample in fluid {
        result.push(CanonicalSample {
            id: sample.id,
            position_um: sample.position_um,
            velocity_um_s: sample.velocity_um_s,
        });
    }
    Ok(result)
}

fn maximum_displacement(
    prior: &[CanonicalSample],
    next: &[CanonicalSample],
) -> Result<i64, WaterError> {
    if prior.len() != next.len() {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "settling displacement sample counts differ",
        ));
    }
    let mut maximum = 0_u64;
    for (prior, next) in prior.iter().zip(next) {
        if prior.id != next.id {
            return Err(WaterError::new(
                AUDIT_INVALID,
                "settling displacement SampleId order differs",
            ));
        }
        for (left, right) in [
            (prior.position_um.x, next.position_um.x),
            (prior.position_um.y, next.position_um.y),
            (prior.position_um.z, next.position_um.z),
        ] {
            maximum = maximum.max(left.abs_diff(right));
        }
    }
    i64::try_from(maximum)
        .map_err(|_| WaterError::new(AUDIT_INVALID, "settling displacement exceeds i64"))
}

fn has_adverse_trend(passes: &[SettlingPassTrace]) -> bool {
    let required = SETTLING_TREND_TRANSITIONS + 1;
    if passes.len() < required {
        return false;
    }
    passes[passes.len() - required..].windows(2).all(|pair| {
        !pair[1].production_ceiling_ready
            && pair[1].density_iterations > pair[0].density_iterations
            && pair[1].maximum_penetration_um > pair[0].maximum_penetration_um
    })
}

fn settling_mismatch(
    production: &SettlingComputation,
    independent: &SettlingComputation,
) -> Option<String> {
    if production.trace.passes.len() != independent.trace.passes.len() {
        return Some(format!(
            "settling pass count {} != {}",
            production.trace.passes.len(),
            independent.trace.passes.len()
        ));
    }
    if let Some(index) = production
        .trace
        .passes
        .iter()
        .zip(&independent.trace.passes)
        .position(|(left, right)| left != right)
    {
        return Some(format!("settling pass {} differs", index + 1));
    }
    if production.trace != independent.trace {
        return Some("settling terminal trace differs".to_owned());
    }
    if production.final_fluid.len() != independent.final_fluid.len() {
        return Some(format!(
            "settling final sample count {} != {}",
            production.final_fluid.len(),
            independent.final_fluid.len()
        ));
    }
    if let Some(index) = production
        .final_fluid
        .iter()
        .zip(&independent.final_fluid)
        .position(|(left, right)| left != right)
    {
        return Some(format!("settling final sample differs at index {index}"));
    }
    None
}

fn write_report(
    output: &Path,
    report: &InitializationReport,
    matched: bool,
) -> Result<(), WaterError> {
    let envelope = InitializationEnvelope {
        schema_version: 1,
        status: if matched { "REPORT_ONLY" } else { "FAIL" },
        command: "continuum water evaluate-hydro-initialization",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("cannot serialize hydro initialization report: {error}"),
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
                    "cannot create initialization report {}: {error}",
                    output.display()
                ),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot write initialization report {}: {error}",
                output.display()
            ),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot sync initialization report {}: {error}",
                output.display()
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization_cli_rejects_an_unregistered_candidate() {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let error = crate::run_xtask(
            repository_root,
            [
                "water".to_owned(),
                "evaluate-hydro-initialization".to_owned(),
                "--candidate".to_owned(),
                "unknown".to_owned(),
                "--output".to_owned(),
                "/tmp/not-created.json".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap_err();
        assert!(error.starts_with("WATER_SCENARIO_INVALID:"));
    }

    #[test]
    fn zero_velocity_settling_matches_independent_and_hits_the_trend_guard() {
        let hydro = scenario::find("CW-HYDRO-001").unwrap();
        let samples = scenario::initial_samples(&hydro, StorageOrder::Reverse).unwrap();
        let boundary = ghost_cell_shell(hydro.geometry).unwrap();
        let production =
            production_settling(&samples, hydro.geometry, &boundary, &[0; 32], &[1; 32]).unwrap();
        let independent = independent_zero_velocity_settling().unwrap();

        assert_eq!(settling_mismatch(&production, &independent), None);
        assert_eq!(production.trace.status, "GENERATOR_REJECTED");
        assert_eq!(production.trace.completed_passes, 4);
        assert_eq!(production.trace.terminal_code, "SETTLING_ADVERSE_TREND");
        assert_eq!(
            fluid_input_root(&production.final_fluid),
            "373af3e7270c46741078b08335629821a50853b7e3bb1fca3c8c462748af1df9"
        );
        assert_eq!(
            production
                .trace
                .passes
                .iter()
                .map(|pass| pass.density_iterations)
                .collect::<Vec<_>>(),
            [2, 33, 40, 47]
        );
        assert_eq!(
            production
                .trace
                .passes
                .iter()
                .map(|pass| pass.density_error_ppb)
                .collect::<Vec<_>>(),
            [95_755, 98_983, 99_961, 98_517]
        );
        assert_eq!(
            production
                .trace
                .passes
                .iter()
                .map(|pass| pass.maximum_displacement_um)
                .collect::<Vec<_>>(),
            [171, 231, 201, 189]
        );
        assert_eq!(
            production
                .trace
                .passes
                .iter()
                .map(|pass| pass.maximum_penetration_um)
                .collect::<Vec<_>>(),
            [171, 340, 521, 710]
        );
    }
}
