use std::fs::OpenOptions;
use std::io::Write;

use super::*;
use crate::oracle::command::validate_report_capacity;

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, WaterError> {
    let mut output = None;
    let mut scenario_id = None;
    let mut reference = None;
    let mut hydro_reference = None;
    let mut dam_break_reference = None;
    let mut orifice_reference = None;
    let mut solver_mode = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| WaterError::new(SCENARIO_INVALID, format!("{flag} requires a value")))?;
        match flag.as_str() {
            "--output" => set_once(&mut output, PathBuf::from(value), "--output")?,
            "--scenario" => set_once(&mut scenario_id, value, "--scenario")?,
            "--reference" => set_once(&mut reference, PathBuf::from(value), "--reference")?,
            "--hydro-reference" => set_once(
                &mut hydro_reference,
                PathBuf::from(value),
                "--hydro-reference",
            )?,
            "--dam-break-reference" => set_once(
                &mut dam_break_reference,
                PathBuf::from(value),
                "--dam-break-reference",
            )?,
            "--orifice-reference" => set_once(
                &mut orifice_reference,
                PathBuf::from(value),
                "--orifice-reference",
            )?,
            "--solver" => {
                let parsed = match value.as_str() {
                    "frozen-successor" => W1SolverMode::FrozenSuccessor,
                    "diagnostic-frozen-observe-energy" => {
                        W1SolverMode::FrozenObserveEnergyDiagnostic
                    }
                    _ => {
                        return Err(WaterError::new(
                            SCENARIO_INVALID,
                            format!("unknown run-w1-linux solver mode {value:?}"),
                        ));
                    }
                };
                set_once(&mut solver_mode, parsed, "--solver")?;
            }
            _ => {
                return Err(WaterError::new(
                    SCENARIO_INVALID,
                    format!("unexpected run-w1-linux argument {flag}"),
                ));
            }
        }
    }
    let output = output.ok_or_else(|| {
        WaterError::new(
            SCENARIO_INVALID,
            "run-w1-linux requires --output <absolute-path>",
        )
    })?;
    if let Some(id) = &scenario_id
        && !SCENARIO_IDS.contains(&id.as_str())
    {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            format!("run-w1-linux does not admit scenario {id:?}"),
        ));
    }
    if scenario_id.is_some()
        && (hydro_reference.is_some()
            || dam_break_reference.is_some()
            || orifice_reference.is_some())
    {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "scenario mode uses --reference, not named full-corpus reference flags",
        ));
    }
    if scenario_id.is_none() && reference.is_some() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "full-corpus mode requires named reference flags",
        ));
    }
    let solver_mode = solver_mode.unwrap_or(W1SolverMode::FrozenSuccessor);
    if solver_mode != W1SolverMode::FrozenSuccessor && scenario_id.is_none() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "diagnostic solver mode requires one explicit --scenario",
        ));
    }
    Ok(Request {
        output,
        scenario_id,
        reference,
        hydro_reference,
        dam_break_reference,
        orifice_reference,
        solver_mode,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), WaterError> {
    if slot.replace(value).is_some() {
        Err(WaterError::new(
            SCENARIO_INVALID,
            format!("duplicate argument {flag}"),
        ))
    } else {
        Ok(())
    }
}

pub(super) fn validate_reference_paths(
    repository_root: &Path,
    request: &Request,
) -> Result<(), WaterError> {
    for path in [
        request.reference.as_deref(),
        request.hydro_reference.as_deref(),
        request.dam_break_reference.as_deref(),
        request.orifice_reference.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if !path.is_absolute() {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                "W1 reference paths must be absolute",
            ));
        }
        let canonical = path.canonicalize().map_err(|error| {
            WaterError::new(
                SCENARIO_INVALID,
                format!("cannot canonicalize reference {}: {error}", path.display()),
            )
        })?;
        let repository = repository_root.canonicalize().map_err(|error| {
            WaterError::new(
                SCENARIO_INVALID,
                format!("cannot canonicalize repository root: {error}"),
            )
        })?;
        if canonical.starts_with(repository) {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                "W1 external references must remain outside the Git worktree",
            ));
        }
        let _ = reference::preflight_size(&canonical)?;
    }
    Ok(())
}

pub(super) fn write_report(
    output: &Path,
    report: &W1Report,
    succeeded: bool,
) -> Result<(), WaterError> {
    let envelope = Envelope {
        schema_version: 1,
        status: if succeeded { "REPORT_ONLY" } else { "FAIL" },
        command: "continuum water run-w1-linux",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize W1 Linux report: {error}"),
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
                format!("cannot create W1 report {}: {error}", output.display()),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot write W1 report {}: {error}", output.display()),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot sync W1 report {}: {error}", output.display()),
        )
    })
}

pub(super) fn command_result(request: &Request, report: &W1Report) -> Result<String, WaterError> {
    #[derive(Serialize)]
    struct ResultDetails<'a> {
        report: String,
        disposition: &'a str,
        product_check: &'a str,
        corpus_run_root: Option<&'a str>,
        windows: &'a str,
    }
    let envelope = serde_json::json!({
        "schema_version": 1,
        "status": "REPORT_ONLY",
        "command": "continuum water run-w1-linux",
        "details": ResultDetails {
            report: request.output.display().to_string(),
            disposition: &report.disposition,
            product_check: &report.product_check,
            corpus_run_root: report.corpus_run_root.as_deref(),
            windows: &report.scope.windows,
        }
    });
    serde_json::to_string(&envelope).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize W1 command result: {error}"),
        )
    })
}
