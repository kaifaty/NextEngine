#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;

use super::*;
use crate::hash::ImpactEnergyRoots;
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
    scenarios: Vec<ClosureScenario>,
    checks: [&'static str; 5],
    product_check: &'static str,
    disposition: &'static str,
}

#[derive(Serialize)]
struct ClosureRoots {
    w0f_execution_profile: String,
    w0g_document: String,
    w0g_energy_contract: String,
    w0g_corpus: String,
    w0g_execution_profile: String,
}

#[derive(Serialize)]
struct ClosureScenario {
    id: &'static str,
    energy_class: &'static str,
    w0f_scenario_root: String,
    w0g_scenario_root: String,
}

pub(super) fn run(
    repository_root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let flag = arguments.next().ok_or_else(|| {
        WaterError::new(
            SCENARIO_INVALID,
            "close-impact-energy-profile requires --output <absolute-path>",
        )
    })?;
    let value = arguments
        .next()
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "--output requires an absolute path"))?;
    if flag != "--output" || arguments.next().is_some() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "close-impact-energy-profile accepts only --output <absolute-path>",
        ));
    }
    let output = PathBuf::from(value);
    validate_output_path(repository_root, &output)?;
    profile::validate_execution_profile()?;
    profile::validate_float_environment()?;
    if env!("WATER_BUILD_TARGET") != LINUX_TARGET {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            format!("W0G Linux closure requires target {LINUX_TARGET}"),
        ));
    }
    let roots = ImpactEnergyRoots::verify(repository_root)?;
    let mut scenarios = Vec::new();
    scenarios
        .try_reserve_exact(SCENARIO_IDS.len())
        .map_err(report_reserve_error)?;
    for scenario_id in SCENARIO_IDS {
        if roots.scenario_projection(scenario_id)?
            != energy_contract_projection(scenario_id)?.as_bytes()
        {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!("W0G projection differs for {scenario_id}"),
            ));
        }
        let energy_class = match energy_class(scenario_id)? {
            EnergyClass::ReversibleEquilibrium => "reversible-equilibrium",
            EnergyClass::StaticImpactDissipative => "static-impact-dissipative",
        };
        scenarios.push(ClosureScenario {
            id: scenario_id,
            energy_class,
            w0f_scenario_root: hash::hex(&roots.parent.scenario_root(scenario_id)?),
            w0g_scenario_root: hash::hex(&roots.scenario_root(scenario_id)?),
        });
    }
    let tree_state = tool_tree_state(repository_root);
    let disposition = if tree_state == "CLEAN" {
        "SUCCESSOR_IMPACT_ENERGY_ROOTS_FROZEN / W1_AUTHORIZED"
    } else {
        "ROOTS_VERIFIED / CLEAN_EVIDENCE_PENDING"
    };
    let report = ClosureReport {
        report_schema: "nextengine.continuum-water.w0g-closure.v1",
        classification: "W0G_METRIC_SEMANTICS_ONLY_RESEARCH_CLOSURE",
        tool_commit: tool_commit(repository_root),
        tool_tree_state: tree_state,
        toolchain_target: env!("WATER_BUILD_TARGET"),
        build_profile: env!("WATER_BUILD_PROFILE"),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        scope: "LINUX_CURRENT_W1 / WINDOWS_DEFERRED_NOT_WAIVED",
        roots: ClosureRoots {
            w0f_execution_profile: hash::hex(&roots.parent.execution_profile),
            w0g_document: hash::hex(&roots.document),
            w0g_energy_contract: hash::hex(&roots.contract),
            w0g_corpus: hash::hex(&roots.corpus),
            w0g_execution_profile: hash::hex(&roots.execution_profile),
        },
        scenarios,
        checks: [
            "W0F_PARENT_ROOTS=PASS",
            "W0G_DOMAIN_ROOTS=PASS",
            "SCENARIO_CLASS_PARTITION_4_PLUS_3=PASS",
            "SCENARIO_PROJECTIONS_EXACT=PASS",
            "WINDOWS_CURRENT_SCOPE=OUT_OF_SCOPE_BY_USER",
        ],
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN",
        disposition,
    };
    write_closure_report(&output, &report)?;
    serde_json::to_string(&serde_json::json!({
        "schema_version": 1,
        "status": "REPORT_ONLY",
        "command": "continuum water close-impact-energy-profile",
        "details": {
            "report": output.display().to_string(),
            "disposition": disposition,
            "product_check": report.product_check,
        }
    }))
    .map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize W0G command result: {error}"),
        )
    })
}

fn write_closure_report(output: &Path, report: &ClosureReport) -> Result<(), WaterError> {
    let envelope = ClosureEnvelope {
        schema_version: 1,
        status: "REPORT_ONLY",
        command: "continuum water close-impact-energy-profile",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize W0G closure report: {error}"),
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
                format!("cannot create W0G report {}: {error}", output.display()),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot write W0G report {}: {error}", output.display()),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot sync W0G report {}: {error}", output.display()),
        )
    })
}
