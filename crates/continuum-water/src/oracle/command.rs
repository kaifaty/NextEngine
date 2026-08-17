#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

use super::*;

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, WaterError> {
    let material = arguments
        .next()
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "continuum requires water oracle"))?;
    let command = arguments
        .next()
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "continuum water requires oracle"))?;
    if material != "water" || command != "oracle" {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            format!("unsupported continuum command {material} {command}"),
        ));
    }
    let mut scenario_id = None;
    let mut output = None;
    let mut reference = None;
    let mut repeat = 1_u8;
    let mut repeat_seen = false;
    let mut storage_order = StorageOrder::Reverse;
    let mut storage_order_seen = false;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| WaterError::new(SCENARIO_INVALID, format!("{flag} requires a value")))?;
        match flag.as_str() {
            "--scenario" => set_once(&mut scenario_id, value, "--scenario")?,
            "--output" => set_once(&mut output, PathBuf::from(value), "--output")?,
            "--reference" => set_once(&mut reference, PathBuf::from(value), "--reference")?,
            "--repeat" => {
                if repeat_seen {
                    return Err(WaterError::new(
                        SCENARIO_INVALID,
                        "duplicate argument --repeat",
                    ));
                }
                repeat_seen = true;
                repeat = value
                    .parse::<u8>()
                    .map_err(|_| WaterError::new(SCENARIO_INVALID, "--repeat must be 1 or 2"))?;
                if !(1..=2).contains(&repeat) {
                    return Err(WaterError::new(SCENARIO_INVALID, "--repeat must be 1 or 2"));
                }
            }
            "--storage-order" => {
                if storage_order_seen {
                    return Err(WaterError::new(
                        SCENARIO_INVALID,
                        "duplicate argument --storage-order",
                    ));
                }
                storage_order_seen = true;
                storage_order = match value.as_str() {
                    "identity" => StorageOrder::Identity,
                    "reverse" => StorageOrder::Reverse,
                    "affine" => StorageOrder::Affine,
                    _ => {
                        return Err(WaterError::new(
                            SCENARIO_INVALID,
                            "--storage-order must be identity, reverse or affine",
                        ));
                    }
                };
            }
            _ => {
                return Err(WaterError::new(
                    SCENARIO_INVALID,
                    format!("unexpected continuum water oracle argument {flag}"),
                ));
            }
        }
    }
    Ok(Request {
        scenario_id: scenario_id
            .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "oracle requires --scenario <ID>"))?,
        output: output.ok_or_else(|| {
            WaterError::new(SCENARIO_INVALID, "oracle requires --output <absolute-path>")
        })?,
        reference,
        repeat,
        storage_order,
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

pub(super) fn validate_output_path(
    repository_root: &Path,
    output: &Path,
) -> Result<(), WaterError> {
    if !output.is_absolute() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "--output must be an absolute path outside the repository",
        ));
    }
    let repository_root = repository_root.canonicalize().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot canonicalize repository root: {error}"),
        )
    })?;
    let parent = output
        .parent()
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "output path has no parent directory"))?;
    let parent = parent.canonicalize().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("output parent must already exist: {error}"),
        )
    })?;
    if parent.starts_with(&repository_root) {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "oracle reports must be written outside the Git worktree",
        ));
    }
    if output.exists() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            format!("output already exists: {}", output.display()),
        ));
    }
    Ok(())
}

pub(super) fn write_report(output: &Path, state: &mut ReportState) -> Result<(), WaterError> {
    let status = if state.details.terminal.status == "FAILED" {
        "FAIL"
    } else {
        "REPORT_ONLY"
    };
    let envelope = CommandEnvelope {
        schema_version: 1,
        status: status.to_owned(),
        command: "continuum water oracle".to_owned(),
        details: &state.details,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize oracle report: {error}"),
        )
    })?;
    bytes.push(b'\n');
    if let Err(error) = validate_report_capacity(bytes.len()) {
        state.fail(&error);
        state.details.step_summaries.clear();
        state.details.output_metrics.clear();
        state.details.analytical_checks.clear();
        state.details.reference.comparisons.clear();
        let minimal = CommandEnvelope {
            schema_version: 1,
            status: "FAIL".to_owned(),
            command: "continuum water oracle".to_owned(),
            details: &state.details,
        };
        bytes = serde_json::to_vec_pretty(&minimal).map_err(|serialize_error| {
            WaterError::new(
                REPORT_CAPACITY_EXCEEDED,
                format!("cannot serialize bounded failure report: {serialize_error}"),
            )
        })?;
        bytes.push(b'\n');
        validate_report_capacity(bytes.len())?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|error| {
            WaterError::new(
                SCENARIO_INVALID,
                format!("cannot create report {}: {error}", output.display()),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot write report {}: {error}", output.display()),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot sync report {}: {error}", output.display()),
        )
    })
}

pub(super) fn validate_report_capacity(byte_count: usize) -> Result<(), WaterError> {
    scenario::validate_capacity(
        byte_count,
        MAXIMUM_REPORT_BYTES,
        REPORT_CAPACITY_EXCEEDED,
        "report bytes",
    )
}

pub(super) fn preflight_report_plan(
    frame_count: usize,
    output_count: usize,
) -> Result<(), WaterError> {
    let planned = frame_count
        .checked_mul(1_024)
        .and_then(|value| {
            output_count
                .checked_mul(512)
                .and_then(|outputs| value.checked_add(outputs))
        })
        .and_then(|value| value.checked_add(65_536))
        .ok_or_else(|| {
            WaterError::new(REPORT_CAPACITY_EXCEEDED, "report capacity plan overflow")
        })?;
    validate_report_capacity(planned)
}

pub(super) fn report_reserve_error(error: std::collections::TryReserveError) -> WaterError {
    WaterError::new(
        REPORT_CAPACITY_EXCEEDED,
        format!("report metric allocation failed: {error}"),
    )
}

pub(super) fn tool_commit(repository_root: &Path) -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repository_root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_owned())
        .unwrap_or_else(|| "UNKNOWN".to_owned())
}

pub(super) fn tool_tree_state(repository_root: &Path) -> String {
    Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=normal"])
        .current_dir(repository_root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            if output.stdout.is_empty() {
                "CLEAN"
            } else {
                "DIRTY"
            }
            .to_owned()
        })
        .unwrap_or_else(|| "UNKNOWN".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_requires_the_explicit_nested_command_and_output() {
        let request = parse_arguments(
            [
                "water",
                "oracle",
                "--scenario",
                "SMOKE-CW-FREEFALL-001",
                "--output",
                "/tmp/water.json",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(request.scenario_id, "SMOKE-CW-FREEFALL-001");
        assert_eq!(request.storage_order, StorageOrder::Reverse);
        assert_eq!(request.repeat, 1);
    }

    #[test]
    fn parser_rejects_duplicate_scalar_options() {
        let error = parse_arguments(
            [
                "water",
                "oracle",
                "--scenario",
                "SMOKE-CW-FREEFALL-001",
                "--output",
                "/tmp/water.json",
                "--repeat",
                "1",
                "--repeat",
                "2",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap_err();
        assert_eq!(error.code(), SCENARIO_INVALID);
    }

    #[test]
    fn report_capacity_accepts_n_minus_one_and_n() {
        assert!(validate_report_capacity(MAXIMUM_REPORT_BYTES - 1).is_ok());
        assert!(validate_report_capacity(MAXIMUM_REPORT_BYTES).is_ok());
        assert_eq!(
            validate_report_capacity(MAXIMUM_REPORT_BYTES + 1)
                .unwrap_err()
                .code(),
            REPORT_CAPACITY_EXCEEDED
        );
    }

    #[test]
    fn all_smoke_only_geometry_analogues_execute_without_corpus_credit() {
        for id in [
            "SMOKE-CW-HYDRO-001",
            "SMOKE-CW-FREEFALL-001",
            "SMOKE-CW-DAMBREAK-001",
            "SMOKE-CW-STILL-001",
            "SMOKE-CW-ORIFICE-001",
            "SMOKE-CW-SEALED-001",
            "SMOKE-CW-ORDER-001",
        ] {
            let scenario = scenario::find(id).unwrap();
            let boundary = boundary::build(scenario.geometry).unwrap();
            let evidence = run_once(
                &scenario,
                scenario::initial_samples(&scenario, StorageOrder::Reverse).unwrap(),
                &boundary,
                &[7_u8; 32],
                &[8_u8; 32],
                None,
            )
            .unwrap_or_else(|error| panic!("{id} failed: {error}"));
            assert!(evidence.freefall_mismatch.is_none(), "{id}");
        }
    }

    #[test]
    fn nominal_freefall_matches_every_canonical_recurrence_frame() {
        let scenario = scenario::find("CW-FREEFALL-001").unwrap();
        let boundary = boundary::build(scenario.geometry).unwrap();
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let roots = FrozenRoots::verify(repository_root).unwrap();
        let scenario_root = scenario::root_for(&scenario, &roots).unwrap();
        let evidence = run_once(
            &scenario,
            scenario::initial_samples(&scenario, StorageOrder::Reverse).unwrap(),
            &boundary,
            &roots.execution_profile,
            &scenario_root,
            None,
        )
        .unwrap();
        assert_eq!(evidence.freefall_mismatch, None);
        assert_eq!(
            hash::hex(&evidence.trajectory_root),
            "e9ab0e40aff19f919802c7bd59d0e31f623197ebdf46b570f3545f4334f374f6"
        );
    }
}
