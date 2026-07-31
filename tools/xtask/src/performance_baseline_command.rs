use std::fs;
use std::path::{Path, PathBuf};

use xtask::performance::{PerformanceBaselineV1, PerformanceRunV1};
use xtask::report::{CommandReportV1, PerformanceDetailsV1};

const PERFORMANCE_REPORT_FILE_NAME: &str = "performance-report-v1.json";
const PERFORMANCE_BASELINE_FILE_NAME: &str = "performance-baseline-v1.json";
const MAX_PERFORMANCE_REPORT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PerformanceBaselineArguments {
    runs: PathBuf,
    output: PathBuf,
}

pub(crate) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<PerformanceBaselineArguments, String> {
    let mut runs = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--runs" if runs.is_none() => runs = Some(PathBuf::from(value)),
            "--output" if output.is_none() => output = Some(PathBuf::from(value)),
            "--runs" | "--output" => return Err(format!("duplicate argument: {flag}")),
            _ => return Err(format!("unknown performance-baseline argument: {flag}")),
        }
    }
    Ok(PerformanceBaselineArguments {
        runs: runs.ok_or_else(|| "performance-baseline requires --runs <directory>".to_owned())?,
        output: output
            .ok_or_else(|| "performance-baseline requires --output <directory>".to_owned())?,
    })
}

pub(crate) fn performance_baseline(
    root: &Path,
    request: &PerformanceBaselineArguments,
) -> Result<(), String> {
    let runs_directory = resolve_from_root(root, &request.runs);
    let output_directory = resolve_from_root(root, &request.output);
    let report_paths = discover_report_paths(&runs_directory)?;
    let runs = report_paths
        .iter()
        .map(|path| read_performance_run(path))
        .collect::<Result<Vec<_>, _>>()?;
    let baseline = PerformanceBaselineV1::from_runs(&runs).map_err(|diagnostics| {
        format!(
            "performance calibration set is invalid: {}",
            diagnostics.join("; ")
        )
    })?;
    let bytes = serde_json::to_vec(&baseline)
        .map_err(|error| format!("failed to serialize PerformanceBaselineV1: {error}"))?;
    write_baseline(&output_directory, &bytes)?;
    println!(
        "{}",
        CommandReportV1::new(
            "performance-baseline",
            "PASS",
            PerformanceBaselinePublishedV1 {
                calibration_runs: baseline.calibration_runs,
                scenario: baseline.scenario.as_str().to_owned(),
                source_commit: baseline.source_commit,
                output: output_directory
                    .join(PERFORMANCE_BASELINE_FILE_NAME)
                    .display()
                    .to_string(),
            },
        )
        .to_json()?
    );
    Ok(())
}

#[derive(serde::Serialize)]
struct PerformanceBaselinePublishedV1 {
    calibration_runs: u32,
    scenario: String,
    source_commit: String,
    output: String,
}

fn resolve_from_root(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}

fn discover_report_paths(runs_directory: &Path) -> Result<Vec<PathBuf>, String> {
    let metadata = fs::symlink_metadata(runs_directory).map_err(|error| {
        format!(
            "failed to inspect calibration directory {}: {error}",
            runs_directory.display()
        )
    })?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "calibration input must be a non-symlink directory: {}",
            runs_directory.display()
        ));
    }
    let mut run_directories = fs::read_dir(runs_directory)
        .map_err(|error| {
            format!(
                "failed to read calibration directory {}: {error}",
                runs_directory.display()
            )
        })?
        .map(|entry| entry.map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    run_directories.sort_by_key(fs::DirEntry::file_name);
    if run_directories.len() != 10 {
        return Err(format!(
            "PERF_BASELINE_REQUIRES_TEN_RUNS: found {} entries in {}",
            run_directories.len(),
            runs_directory.display()
        ));
    }
    run_directories
        .into_iter()
        .map(|entry| {
            let file_type = entry.file_type().map_err(|error| {
                format!(
                    "failed to inspect calibration entry {}: {error}",
                    entry.path().display()
                )
            })?;
            if !file_type.is_dir() {
                return Err(format!(
                    "calibration entry must be a non-symlink directory: {}",
                    entry.path().display()
                ));
            }
            Ok(entry.path().join(PERFORMANCE_REPORT_FILE_NAME))
        })
        .collect()
}

fn read_performance_run(path: &Path) -> Result<PerformanceRunV1, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect report {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_PERFORMANCE_REPORT_BYTES {
        return Err(format!(
            "performance report must be a non-symlink regular file no larger than {MAX_PERFORMANCE_REPORT_BYTES} bytes: {}",
            path.display()
        ));
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read report {}: {error}", path.display()))?;
    let report: CommandReportV1<PerformanceDetailsV1> = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid performance report {}: {error}", path.display()))?;
    if report.schema_version != 1 || report.command != "performance" {
        return Err(format!(
            "calibration input is not a performance CommandReportV1: {}",
            path.display()
        ));
    }
    report
        .details
        .run
        .ok_or_else(|| format!("performance run is missing from report {}", path.display()))
}

fn write_baseline(output_directory: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::create_dir_all(output_directory).map_err(|error| {
        format!(
            "failed to create performance baseline output {}: {error}",
            output_directory.display()
        )
    })?;
    let final_path = output_directory.join(PERFORMANCE_BASELINE_FILE_NAME);
    let temporary_path = output_directory.join(".performance-baseline-v1.json.tmp");
    if final_path.exists() || temporary_path.exists() {
        return Err(format!(
            "performance baseline output already exists: {}",
            final_path.display()
        ));
    }
    fs::write(&temporary_path, bytes).map_err(|error| {
        format!(
            "failed to write performance baseline {}: {error}",
            temporary_path.display()
        )
    })?;
    fs::rename(&temporary_path, &final_path).map_err(|error| {
        format!(
            "failed to publish performance baseline {}: {error}",
            final_path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_cli_requires_each_path_once() {
        let request = parse_arguments(
            ["--runs", "runs", "--output", "baseline"]
                .into_iter()
                .map(str::to_owned),
        )
        .expect("valid arguments");
        assert_eq!(request.runs, PathBuf::from("runs"));
        assert_eq!(request.output, PathBuf::from("baseline"));
        assert!(
            parse_arguments(
                ["--runs", "a", "--runs", "b", "--output", "c"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
        assert!(parse_arguments(std::iter::empty()).is_err());
    }
}
