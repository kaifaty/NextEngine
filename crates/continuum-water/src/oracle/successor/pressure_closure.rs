#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;

use super::*;
use crate::error::AUDIT_MISMATCH;
use crate::hash::AcceleratedPressureRoots;
use crate::oracle::command::{
    tool_commit, tool_tree_state, validate_output_path, validate_report_capacity,
};

#[derive(Serialize)]
struct ClosureEnvelope<'a> {
    schema_version: u32,
    status: &'static str,
    command: &'static str,
    details: &'a ClosureReport,
}

#[derive(Serialize)]
struct ClosureReport {
    report_schema: &'static str,
    classification: &'static str,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: &'static str,
    build_profile: &'static str,
    build_rustflags: String,
    scope: &'static str,
    roots: ClosureRoots,
    production_probe: crate::calibration::AcceleratedPressureFirstStepProbe,
    independent_probe: crate::calibration::AcceleratedPressureFirstStepProbe,
    scenarios: Vec<ClosureScenario>,
    checks: [&'static str; 7],
    product_check: &'static str,
    disposition: &'static str,
}

#[derive(Serialize)]
struct ClosureRoots {
    w0f_execution_profile: String,
    w0g_execution_profile: String,
    w0h_document: String,
    w0h_solver_profile: String,
    w0h_corpus: String,
    w0h_execution_profile: String,
}

#[derive(Serialize)]
struct ClosureScenario {
    id: &'static str,
    w0g_scenario_root: String,
    w0h_scenario_root: String,
}

pub(super) fn run(
    repository_root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let flag = arguments.next().ok_or_else(|| {
        WaterError::new(
            SCENARIO_INVALID,
            "close-accelerated-pressure-profile requires --output <absolute-path>",
        )
    })?;
    let value = arguments
        .next()
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "--output requires an absolute path"))?;
    if flag != "--output" || arguments.next().is_some() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "close-accelerated-pressure-profile accepts only --output <absolute-path>",
        ));
    }
    let output = PathBuf::from(value);
    validate_output_path(repository_root, &output)?;
    profile::validate_execution_profile()?;
    profile::validate_float_environment()?;
    if env!("WATER_BUILD_TARGET") != LINUX_TARGET {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            format!("W0H Linux closure requires target {LINUX_TARGET}"),
        ));
    }

    let roots = AcceleratedPressureRoots::verify(repository_root)?;
    let hydro = scenario::find("CW-HYDRO-001")?;
    let samples = scenario::initial_samples(&hydro, StorageOrder::Reverse)?;
    let boundary = build_density_support(hydro.geometry)?;
    let production = solver::production_accelerated_pressure_first_step_probe(
        &samples,
        hydro.geometry,
        &boundary,
    )?;
    let independent =
        crate::audit::independent_support_complete_accelerated_pressure_first_step_probe()?;
    if production != independent {
        return Err(WaterError::new(
            AUDIT_MISMATCH,
            "W0H production and independent first-step pressure probes differ",
        ));
    }

    let mut scenarios = Vec::new();
    scenarios
        .try_reserve_exact(SCENARIO_IDS.len())
        .map_err(report_reserve_error)?;
    for scenario_id in SCENARIO_IDS {
        scenarios.push(ClosureScenario {
            id: scenario_id,
            w0g_scenario_root: hash::hex(&roots.parent.scenario_root(scenario_id)?),
            w0h_scenario_root: hash::hex(&roots.scenario_root(scenario_id)?),
        });
    }

    let tree_state = tool_tree_state(repository_root);
    let disposition = if tree_state == "CLEAN" {
        "ACCELERATED_PRESSURE_ROOTS_FROZEN / W1_AUTHORIZED"
    } else {
        "ROOTS_AND_INDEPENDENT_PROBE_VERIFIED / CLEAN_EVIDENCE_PENDING"
    };
    let report = ClosureReport {
        report_schema: "nextengine.continuum-water.w0h-closure.v1",
        classification: "W0H_SOLVER_ALGORITHM_RESEARCH_CLOSURE",
        tool_commit: tool_commit(repository_root),
        tool_tree_state: tree_state,
        toolchain_target: env!("WATER_BUILD_TARGET"),
        build_profile: env!("WATER_BUILD_PROFILE"),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        scope: "LINUX_CURRENT_W1 / WINDOWS_DEFERRED_NOT_WAIVED",
        roots: ClosureRoots {
            w0f_execution_profile: hash::hex(&roots.parent.parent.execution_profile),
            w0g_execution_profile: hash::hex(&roots.parent.execution_profile),
            w0h_document: hash::hex(&roots.document),
            w0h_solver_profile: hash::hex(&roots.solver_profile),
            w0h_corpus: hash::hex(&roots.corpus),
            w0h_execution_profile: hash::hex(&roots.execution_profile),
        },
        production_probe: production,
        independent_probe: independent,
        scenarios,
        checks: [
            "W0F_W0G_PARENT_ROOTS=PASS",
            "W0H_DOMAIN_ROOTS=PASS",
            "PRODUCTION_INDEPENDENT_FIRST_STEP=EXACT_MATCH",
            "COMPRESSION_ACCEPTANCE=PASS",
            "PROJECTED_KKT_ACCEPTANCE=PASS",
            "COLD_START_AND_FIXED_OPERATION_BUDGET=PASS",
            "WINDOWS_CURRENT_SCOPE=OUT_OF_SCOPE_BY_USER",
        ],
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN",
        disposition,
    };
    write_closure_report(&output, &report)?;
    serde_json::to_string(&serde_json::json!({
        "schema_version": 1,
        "status": "REPORT_ONLY",
        "command": "continuum water close-accelerated-pressure-profile",
        "details": {
            "report": output.display().to_string(),
            "disposition": disposition,
            "product_check": report.product_check,
        }
    }))
    .map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize W0H command result: {error}"),
        )
    })
}

fn write_closure_report(output: &Path, report: &ClosureReport) -> Result<(), WaterError> {
    let envelope = ClosureEnvelope {
        schema_version: 1,
        status: "REPORT_ONLY",
        command: "continuum water close-accelerated-pressure-profile",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize W0H closure report: {error}"),
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
                format!("cannot create W0H report {}: {error}", output.display()),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot write W0H report {}: {error}", output.display()),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot sync W0H report {}: {error}", output.display()),
        )
    })
}
