use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xtask::performance::{PERFORMANCE_REPORT_FILE_NAME, PerformanceRunV6};
use xtask::performance_codegen::{
    CODEGEN_BUILD_SCHEMA_VERSION, CODEGEN_RUN_PROVENANCE_SCHEMA_VERSION, CODEGEN_SCENARIOS,
    CodegenBuildProvenanceV1, CodegenCandidateV1, CodegenComparisonV1, CodegenComparisonVerdictV1,
    CodegenRunProvenanceV1, CodegenScenarioRunSetV1, compare_codegen_run_sets,
};
use xtask::report::{CommandReportV1, PerformanceDetailsV1};

mod provenance;

use provenance::{
    decode_build_toolchain, decode_effective_rustflags, decode_profile_overrides, hash_file,
    verified_pgo_profile_sha256,
};

const CODEGEN_BUILD_FILE_NAME: &str = "performance-codegen-build-v1.json";
const CODEGEN_RUN_FILE_NAME: &str = "performance-codegen-run-v1.json";
const CODEGEN_COMPARISON_FILE_NAME: &str = "performance-codegen-comparison-v1.json";
const MAX_PERFORMANCE_REPORT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PGO_RAW_FILES: usize = 4_096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PerformanceCodegenArguments {
    Stamp {
        candidate: CodegenCandidateV1,
        output: PathBuf,
    },
    Compare {
        baseline: PathBuf,
        candidate_runs: PathBuf,
        candidate: CodegenCandidateV1,
        output: PathBuf,
    },
    Collect {
        candidate_root: PathBuf,
        candidate: CodegenCandidateV1,
        scenario: xtask::performance::PerformanceScenarioV1,
        output: PathBuf,
    },
    PgoMerge {
        profiles: PathBuf,
        output: PathBuf,
    },
}

pub(crate) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<PerformanceCodegenArguments, String> {
    let operation = arguments.next().ok_or_else(|| {
        "performance-codegen requires stamp, collect, compare or pgo-merge".to_owned()
    })?;
    let mut values = Vec::new();
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        if values.iter().any(|(existing, _)| existing == &flag) {
            return Err(format!("duplicate argument: {flag}"));
        }
        values.push((flag, value));
    }
    match operation.as_str() {
        "stamp" => {
            reject_unknown(&values, &["--candidate-kind", "--output"])?;
            Ok(PerformanceCodegenArguments::Stamp {
                candidate: required_candidate(&values)?,
                output: required_path(&values, "--output")?,
            })
        }
        "compare" => {
            reject_unknown(
                &values,
                &["--baseline", "--candidate", "--candidate-kind", "--output"],
            )?;
            let candidate = required_candidate(&values)?;
            if candidate == CodegenCandidateV1::Baseline {
                return Err(
                    "performance-codegen compare requires thin-lto or pgo candidate-kind"
                        .to_owned(),
                );
            }
            Ok(PerformanceCodegenArguments::Compare {
                baseline: required_path(&values, "--baseline")?,
                candidate_runs: required_path(&values, "--candidate")?,
                candidate,
                output: required_path(&values, "--output")?,
            })
        }
        "collect" => {
            reject_unknown(
                &values,
                &[
                    "--candidate-root",
                    "--candidate-kind",
                    "--scenario",
                    "--output",
                ],
            )?;
            let scenario = xtask::performance::PerformanceScenarioV1::parse(required_value(
                &values,
                "--scenario",
            )?)?;
            if !CODEGEN_SCENARIOS.contains(&scenario) {
                return Err(format!(
                    "performance-codegen collect requires a representative R2-R5 scenario, got {}",
                    scenario.as_str()
                ));
            }
            Ok(PerformanceCodegenArguments::Collect {
                candidate_root: required_path(&values, "--candidate-root")?,
                candidate: required_candidate(&values)?,
                scenario,
                output: required_path(&values, "--output")?,
            })
        }
        "pgo-merge" => {
            reject_unknown(&values, &["--profiles", "--output"])?;
            Ok(PerformanceCodegenArguments::PgoMerge {
                profiles: required_path(&values, "--profiles")?,
                output: required_path(&values, "--output")?,
            })
        }
        _ => Err(format!(
            "unknown performance-codegen operation: {operation}"
        )),
    }
}

pub(crate) fn performance_codegen(
    root: &Path,
    request: &PerformanceCodegenArguments,
) -> Result<(), String> {
    match request {
        PerformanceCodegenArguments::Stamp { candidate, output } => {
            stamp_candidate(root, *candidate, &resolve_from_root(root, output))
        }
        PerformanceCodegenArguments::Compare {
            baseline,
            candidate_runs,
            candidate,
            output,
        } => compare_candidate(
            &resolve_from_root(root, baseline),
            &resolve_from_root(root, candidate_runs),
            *candidate,
            &resolve_from_root(root, output),
        ),
        PerformanceCodegenArguments::Collect {
            candidate_root,
            candidate,
            scenario,
            output,
        } => collect_candidate_run(
            root,
            &resolve_from_root(root, candidate_root),
            *candidate,
            *scenario,
            &resolve_from_root(root, output),
        ),
        PerformanceCodegenArguments::PgoMerge { profiles, output } => merge_pgo_profiles(
            &resolve_from_root(root, profiles),
            &resolve_from_root(root, output),
        ),
    }
}

fn stamp_candidate(
    root: &Path,
    candidate: CodegenCandidateV1,
    output: &Path,
) -> Result<(), String> {
    let provenance = current_provenance(root, candidate)?;
    provenance.validate().map_err(|diagnostics| {
        format!(
            "codegen build provenance is invalid: {}",
            diagnostics.join("; ")
        )
    })?;
    let report = CommandReportV1::new("performance-codegen", "PASS", provenance);
    publish_json(output, CODEGEN_BUILD_FILE_NAME, &report)?;
    println!("{}", report.to_json()?);
    Ok(())
}

fn current_provenance(
    root: &Path,
    candidate: CodegenCandidateV1,
) -> Result<CodegenBuildProvenanceV1, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("failed to resolve current xtask executable: {error}"))?;
    let build_commit = env!("NEXTENGINE_BUILD_COMMIT");
    let current_commit = git_output(root, &["rev-parse", "HEAD"])?;
    let current_worktree_clean = git_output(root, &["status", "--porcelain"])?.is_empty();
    let build_worktree_clean = env!("NEXTENGINE_BUILD_WORKTREE_CLEAN") == "true";
    let profile_path = match candidate {
        CodegenCandidateV1::Baseline => None,
        CodegenCandidateV1::ThinLto => None,
        CodegenCandidateV1::Pgo => {
            let path = env!("NEXTENGINE_PGO_PROFILE_PATH");
            (!path.is_empty()).then(|| resolve_from_root(root, Path::new(path)))
        }
    };
    let pgo_profile_sha256 = profile_path
        .as_deref()
        .map(|path| verified_pgo_profile_sha256(path, env!("NEXTENGINE_PGO_PROFILE_SHA256")))
        .transpose()?;
    Ok(CodegenBuildProvenanceV1 {
        schema_version: CODEGEN_BUILD_SCHEMA_VERSION,
        candidate,
        build_profile: env!("NEXTENGINE_BUILD_PROFILE").to_owned(),
        commit: build_commit.to_owned(),
        worktree_clean: build_worktree_clean
            && current_worktree_clean
            && current_commit == build_commit,
        executable_sha256: hash_file(&executable)?,
        toolchain: decode_build_toolchain()?,
        opt_level: env!("NEXTENGINE_BUILD_OPT_LEVEL").to_owned(),
        effective_rustflags: decode_effective_rustflags()?,
        profile_overrides: decode_profile_overrides()?,
        codegen_mode: env!("NEXTENGINE_CODEGEN_MODE").to_owned(),
        pgo_profile_sha256,
        forbidden_codegen_flags_present: env!("NEXTENGINE_CODEGEN_FORBIDDEN_FLAGS") == "present",
    })
}

fn collect_candidate_run(
    root: &Path,
    candidate_root: &Path,
    candidate: CodegenCandidateV1,
    scenario: xtask::performance::PerformanceScenarioV1,
    output: &Path,
) -> Result<(), String> {
    let stamped = read_codegen_provenance(&candidate_root.join(CODEGEN_BUILD_FILE_NAME))?;
    let current = current_provenance(root, candidate)?;
    current.validate().map_err(|diagnostics| {
        format!(
            "current codegen build provenance is invalid: {}",
            diagnostics.join("; ")
        )
    })?;
    require_exact_build_match(&stamped, &current)?;
    match std::env::var("NEXTENGINE_PERFORMANCE_PROFILER") {
        Ok(value) if value == "1" || value.eq_ignore_ascii_case("on") => {}
        _ => return Err("CODEGEN_COLLECT_REQUIRES_PERFORMANCE_PROFILER_ON".to_owned()),
    }
    let executable = std::env::current_exe()
        .map_err(|error| format!("failed to resolve current xtask executable: {error}"))?;
    let child = Command::new(&executable)
        .args([
            "performance",
            "--scenario",
            scenario.as_str(),
            "--mode",
            "report",
            "--target",
            xtask::performance::LINUX_RELEASE_TARGET_ID,
            "--output",
        ])
        .arg(output)
        .arg("--require-ready-preflight")
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch candidate performance run: {error}"))?;
    if !child.status.success() {
        return Err(format!(
            "candidate performance run failed with {}: stdout={}; stderr={}",
            child.status,
            bounded_output(&child.stdout),
            bounded_output(&child.stderr)
        ));
    }
    let report_path = output.join(PERFORMANCE_REPORT_FILE_NAME);
    let run = read_performance_run(&report_path)?;
    if run.build_profile != candidate.expected_build_profile() || run.scenario != scenario {
        return Err("CODEGEN_COLLECTED_RUN_IDENTITY_MISMATCH".to_owned());
    }
    let run_provenance = CodegenRunProvenanceV1 {
        schema_version: CODEGEN_RUN_PROVENANCE_SCHEMA_VERSION,
        build: current,
        performance_report_sha256: hash_file(&report_path)?,
    };
    publish_json(output, CODEGEN_RUN_FILE_NAME, &run_provenance)?;
    CommandReportV1::emit(
        "performance-codegen",
        "PASS",
        CodegenRunCollectedV1 {
            candidate: candidate.as_str(),
            scenario: scenario.as_str(),
            output: output.display().to_string(),
            performance_verdict: run.verdict.as_str(),
        },
    )
}

fn require_exact_build_match(
    stamped: &CodegenBuildProvenanceV1,
    current: &CodegenBuildProvenanceV1,
) -> Result<(), String> {
    if stamped.executable_sha256 != current.executable_sha256 {
        Err("CODEGEN_CURRENT_EXECUTABLE_HASH_MISMATCH".to_owned())
    } else if stamped != current {
        Err("CODEGEN_CURRENT_BINARY_DOES_NOT_MATCH_BUILD_STAMP".to_owned())
    } else {
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct CodegenRunCollectedV1<'a> {
    candidate: &'a str,
    scenario: &'a str,
    output: String,
    performance_verdict: &'a str,
}

fn compare_candidate(
    baseline_root: &Path,
    candidate_root: &Path,
    candidate: CodegenCandidateV1,
    output: &Path,
) -> Result<(), String> {
    let baseline_provenance =
        read_codegen_provenance(&baseline_root.join(CODEGEN_BUILD_FILE_NAME))?;
    if baseline_provenance.candidate != CodegenCandidateV1::Baseline {
        return Err("CODEGEN_BASELINE_BUILD_KIND_MISMATCH".to_owned());
    }
    let candidate_provenance =
        read_codegen_provenance(&candidate_root.join(CODEGEN_BUILD_FILE_NAME))?;
    if candidate_provenance.candidate != candidate {
        return Err("CODEGEN_BUILD_CANDIDATE_MISMATCH".to_owned());
    }
    let baseline_sets = read_provenanced_run_sets(
        baseline_root,
        &baseline_provenance,
        "codegen baseline run-set root",
    )?;
    let candidate_sets = read_provenanced_run_sets(
        candidate_root,
        &candidate_provenance,
        "codegen candidate run-set root",
    )?;
    let comparison = compare_codegen_run_sets(
        candidate,
        &baseline_provenance,
        &candidate_provenance,
        &baseline_sets,
        &candidate_sets,
    )
    .unwrap_or_else(|diagnostics| CodegenComparisonV1::not_run(candidate, diagnostics));
    let status = match comparison.verdict {
        CodegenComparisonVerdictV1::ReportOnly => "PASS",
        CodegenComparisonVerdictV1::NotRun => "NOT_RUN",
    };
    let report = CommandReportV1::new("performance-codegen", status, comparison);
    publish_json(output, CODEGEN_COMPARISON_FILE_NAME, &report)?;
    println!("{}", report.to_json()?);
    Ok(())
}

fn merge_pgo_profiles(profiles: &Path, output: &Path) -> Result<(), String> {
    let raw_profiles = discover_raw_profiles(profiles)?;
    let llvm_profdata = llvm_profdata_path()?;
    let parent = output
        .parent()
        .ok_or_else(|| format!("PGO output has no parent: {}", output.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create PGO output directory {}: {error}",
            parent.display()
        )
    })?;
    let file_name = output
        .file_name()
        .ok_or_else(|| format!("PGO output has no file name: {}", output.display()))?;
    let mut temporary_name = OsString::from(".");
    temporary_name.push(file_name);
    temporary_name.push(".tmp");
    let temporary = parent.join(temporary_name);
    if output.exists() || temporary.exists() {
        return Err(format!(
            "PGO merge output already exists: {}",
            output.display()
        ));
    }
    let status = Command::new(&llvm_profdata)
        .arg("merge")
        .arg("--failure-mode=all")
        .arg("-o")
        .arg(&temporary)
        .args(&raw_profiles)
        .status()
        .map_err(|error| format!("failed to run {}: {error}", llvm_profdata.display()))?;
    if !status.success() {
        if temporary.is_file() {
            let _ = fs::remove_file(&temporary);
        }
        return Err(format!("llvm-profdata merge failed with {status}"));
    }
    fs::rename(&temporary, output).map_err(|error| {
        format!(
            "failed to publish merged PGO profile {}: {error}",
            output.display()
        )
    })?;
    let report = CommandReportV1::new(
        "performance-codegen",
        "PASS",
        PgoMergeDetailsV1 {
            raw_profile_count: raw_profiles.len(),
            llvm_profdata: llvm_profdata.display().to_string(),
            output: output.display().to_string(),
            profile_sha256: hash_file(output)?,
        },
    );
    println!("{}", report.to_json()?);
    Ok(())
}

#[derive(serde::Serialize)]
struct PgoMergeDetailsV1 {
    raw_profile_count: usize,
    llvm_profdata: String,
    output: String,
    profile_sha256: String,
}

fn required_candidate(values: &[(String, String)]) -> Result<CodegenCandidateV1, String> {
    let value = required_value(values, "--candidate-kind")?;
    CodegenCandidateV1::parse(value)
}

fn required_path(values: &[(String, String)], flag: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(required_value(values, flag)?))
}

fn required_value<'a>(values: &'a [(String, String)], flag: &str) -> Result<&'a str, String> {
    values
        .iter()
        .find_map(|(observed, value)| (observed == flag).then_some(value.as_str()))
        .ok_or_else(|| format!("performance-codegen requires {flag} <value>"))
}

fn reject_unknown(values: &[(String, String)], allowed: &[&str]) -> Result<(), String> {
    if let Some((flag, _)) = values
        .iter()
        .find(|(flag, _)| !allowed.contains(&flag.as_str()))
    {
        Err(format!("unknown performance-codegen argument: {flag}"))
    } else {
        Ok(())
    }
}

fn resolve_from_root(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}

fn git_output(root: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn command_output(program: &str, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .map_err(|error| format!("failed to run {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn read_provenanced_run_sets(
    root: &Path,
    provenance: &CodegenBuildProvenanceV1,
    description: &str,
) -> Result<Vec<CodegenScenarioRunSetV1>, String> {
    require_directory(root, description)?;
    CODEGEN_SCENARIOS
        .into_iter()
        .map(|scenario| {
            let scenario_root = root.join(scenario.as_str());
            let report_paths = discover_report_paths(&scenario_root)?;
            let runs = report_paths
                .iter()
                .map(|path| {
                    validate_run_sidecar(path, provenance)?;
                    read_performance_run(path)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CodegenScenarioRunSetV1 { scenario, runs })
        })
        .collect()
}

fn validate_run_sidecar(
    report_path: &Path,
    provenance: &CodegenBuildProvenanceV1,
) -> Result<(), String> {
    let sidecar_path = report_path
        .parent()
        .ok_or_else(|| {
            format!(
                "performance report has no parent: {}",
                report_path.display()
            )
        })?
        .join(CODEGEN_RUN_FILE_NAME);
    let metadata = fs::symlink_metadata(&sidecar_path).map_err(|error| {
        format!(
            "CODEGEN_RUN_PROVENANCE_MISSING_OR_UNREADABLE: {}: {error}",
            sidecar_path.display()
        )
    })?;
    if !metadata.file_type().is_file() || metadata.len() > 1024 * 1024 {
        return Err(format!(
            "codegen run provenance must be a non-symlink regular file no larger than 1 MiB: {}",
            sidecar_path.display()
        ));
    }
    let bytes = fs::read(&sidecar_path)
        .map_err(|error| format!("failed to read {}: {error}", sidecar_path.display()))?;
    let sidecar: CodegenRunProvenanceV1 = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "invalid codegen run provenance {}: {error}",
            sidecar_path.display()
        )
    })?;
    sidecar.validate_for(provenance).map_err(|diagnostics| {
        format!(
            "codegen run provenance is invalid at {}: {}",
            sidecar_path.display(),
            diagnostics.join("; ")
        )
    })?;
    if sidecar.performance_report_sha256 != hash_file(report_path)? {
        return Err(format!(
            "CODEGEN_RUN_REPORT_HASH_MISMATCH: {}",
            report_path.display()
        ));
    }
    Ok(())
}

fn discover_report_paths(scenario_root: &Path) -> Result<Vec<PathBuf>, String> {
    require_directory(scenario_root, "codegen scenario")?;
    let mut entries = fs::read_dir(scenario_root)
        .map_err(|error| format!("failed to read {}: {error}", scenario_root.display()))?
        .map(|entry| entry.map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    if entries.len() != 10 {
        return Err(format!(
            "CODEGEN_REQUIRES_TEN_RUNS: {}: found {}",
            scenario_root.display(),
            entries.len()
        ));
    }
    entries
        .into_iter()
        .map(|entry| {
            let file_type = entry.file_type().map_err(|error| {
                format!("failed to inspect {}: {error}", entry.path().display())
            })?;
            if !file_type.is_dir() {
                return Err(format!(
                    "codegen run entry must be a non-symlink directory: {}",
                    entry.path().display()
                ));
            }
            Ok(entry.path().join(PERFORMANCE_REPORT_FILE_NAME))
        })
        .collect()
}

fn read_performance_run(path: &Path) -> Result<PerformanceRunV6, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_PERFORMANCE_REPORT_BYTES {
        return Err(format!(
            "performance report must be a non-symlink regular file no larger than {MAX_PERFORMANCE_REPORT_BYTES} bytes: {}",
            path.display()
        ));
    }
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let report: CommandReportV1<PerformanceDetailsV1> = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid performance report {}: {error}", path.display()))?;
    if report.schema_version != 1 || report.command != "performance" {
        return Err(format!(
            "input is not a performance CommandReportV1: {}",
            path.display()
        ));
    }
    let run = report
        .details
        .run
        .ok_or_else(|| format!("performance run is missing from {}", path.display()))?;
    run.validate_wire_version().map_err(|diagnostics| {
        format!(
            "performance run has an incompatible wire version at {}: {}",
            path.display(),
            diagnostics.join(", ")
        )
    })?;
    Ok(run)
}

fn read_codegen_provenance(path: &Path) -> Result<CodegenBuildProvenanceV1, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.len() > 1024 * 1024 {
        return Err(format!(
            "codegen build stamp must be a non-symlink regular file no larger than 1 MiB: {}",
            path.display()
        ));
    }
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let report: CommandReportV1<CodegenBuildProvenanceV1> = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid codegen build stamp {}: {error}", path.display()))?;
    if report.schema_version != 1 || report.command != "performance-codegen" {
        return Err(format!(
            "input is not a performance-codegen CommandReportV1: {}",
            path.display()
        ));
    }
    Ok(report.details)
}

fn require_directory(path: &Path, description: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "{description} must be a non-symlink directory: {}",
            path.display()
        ));
    }
    Ok(())
}

fn discover_raw_profiles(path: &Path) -> Result<Vec<PathBuf>, String> {
    require_directory(path, "PGO raw-profile input")?;
    let mut profiles = fs::read_dir(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?
        .map(|entry| entry.map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    profiles.sort_by_key(fs::DirEntry::file_name);
    if profiles.is_empty() || profiles.len() > MAX_PGO_RAW_FILES {
        return Err(format!(
            "PGO raw-profile input requires 1..={MAX_PGO_RAW_FILES} files: {}",
            path.display()
        ));
    }
    profiles
        .into_iter()
        .map(|entry| {
            let entry_path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("failed to inspect {}: {error}", entry_path.display()))?;
            if !file_type.is_file()
                || entry_path.extension().and_then(|value| value.to_str()) != Some("profraw")
            {
                return Err(format!(
                    "PGO input must contain only direct non-symlink .profraw files: {}",
                    entry_path.display()
                ));
            }
            Ok(entry_path)
        })
        .collect()
}

fn llvm_profdata_path() -> Result<PathBuf, String> {
    let sysroot = PathBuf::from(command_output("rustc", &["--print", "sysroot"])?);
    let version = command_output("rustc", &["-vV"])?;
    let host = version
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| "rustc -vV did not report a host triple".to_owned())?;
    let executable = if cfg!(windows) {
        "llvm-profdata.exe"
    } else {
        "llvm-profdata"
    };
    let path = sysroot
        .join("lib")
        .join("rustlib")
        .join(host)
        .join("bin")
        .join(executable);
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        format!(
            "matching llvm-profdata is unavailable at {} ({error}); install the pinned toolchain's llvm-tools-preview component",
            path.display()
        )
    })?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "matching llvm-profdata must be a non-symlink regular file: {}",
            path.display()
        ));
    }
    Ok(path)
}

fn bounded_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).chars().take(4_096).collect()
}

fn publish_json<T: serde::Serialize>(
    output: &Path,
    file_name: &str,
    value: &T,
) -> Result<(), String> {
    fs::create_dir_all(output)
        .map_err(|error| format!("failed to create {}: {error}", output.display()))?;
    let final_path = output.join(file_name);
    let temporary_path = output.join(format!(".{file_name}.tmp"));
    if final_path.exists() || temporary_path.exists() {
        return Err(format!(
            "performance-codegen output already exists: {}",
            final_path.display()
        ));
    }
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("failed to serialize performance-codegen report: {error}"))?;
    fs::write(&temporary_path, bytes)
        .map_err(|error| format!("failed to write {}: {error}", temporary_path.display()))?;
    fs::rename(&temporary_path, &final_path)
        .map_err(|error| format!("failed to publish {}: {error}", final_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_baseline_build() -> CodegenBuildProvenanceV1 {
        CodegenBuildProvenanceV1 {
            schema_version: CODEGEN_BUILD_SCHEMA_VERSION,
            candidate: CodegenCandidateV1::Baseline,
            build_profile: "release".to_owned(),
            commit: "a".repeat(40),
            worktree_clean: true,
            executable_sha256: "b".repeat(64),
            toolchain: "rustc 1.97.1 (8bab26f4f 2026-07-14)\n\
binary: rustc\n\
commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452\n\
commit-date: 2026-07-14\n\
host: x86_64-pc-windows-msvc\n\
release: 1.97.1\n\
LLVM version: 22.1.6"
                .to_owned(),
            opt_level: "3".to_owned(),
            effective_rustflags: Vec::new(),
            profile_overrides: Vec::new(),
            codegen_mode: "plain".to_owned(),
            pgo_profile_sha256: None,
            forbidden_codegen_flags_present: false,
        }
    }

    #[test]
    fn parse_stamp_compare_and_merge() {
        assert_eq!(
            parse_arguments(
                ["stamp", "--candidate-kind", "baseline", "--output", "out"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .expect("baseline stamp"),
            PerformanceCodegenArguments::Stamp {
                candidate: CodegenCandidateV1::Baseline,
                output: PathBuf::from("out")
            }
        );
        assert_eq!(
            parse_arguments(
                ["stamp", "--candidate-kind", "thin-lto", "--output", "out"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .expect("stamp"),
            PerformanceCodegenArguments::Stamp {
                candidate: CodegenCandidateV1::ThinLto,
                output: PathBuf::from("out")
            }
        );
        assert!(
            parse_arguments(
                [
                    "compare",
                    "--baseline",
                    "base",
                    "--candidate",
                    "candidate",
                    "--candidate-kind",
                    "pgo",
                    "--output",
                    "out",
                ]
                .into_iter()
                .map(str::to_owned)
            )
            .is_ok()
        );
        assert!(
            parse_arguments(
                [
                    "collect",
                    "--candidate-root",
                    "baseline",
                    "--candidate-kind",
                    "baseline",
                    "--scenario",
                    "r2-alpha-render",
                    "--output",
                    "baseline/r2-alpha-render/run-01",
                ]
                .into_iter()
                .map(str::to_owned)
            )
            .is_ok()
        );
        assert!(
            parse_arguments(
                [
                    "collect",
                    "--candidate-root",
                    "candidate",
                    "--candidate-kind",
                    "pgo",
                    "--scenario",
                    "r5-physics-16",
                    "--output",
                    "candidate/r5-physics-16/run-01",
                ]
                .into_iter()
                .map(str::to_owned)
            )
            .is_ok()
        );
        assert!(
            parse_arguments(
                [
                    "pgo-merge",
                    "--profiles",
                    "raw",
                    "--output",
                    "merged.profdata"
                ]
                .into_iter()
                .map(str::to_owned)
            )
            .is_ok()
        );
    }

    #[test]
    fn parse_rejects_duplicates_and_unknown_flags() {
        assert!(
            parse_arguments(
                [
                    "stamp",
                    "--candidate-kind",
                    "thin-lto",
                    "--candidate-kind",
                    "pgo",
                    "--output",
                    "out",
                ]
                .into_iter()
                .map(str::to_owned)
            )
            .is_err()
        );
        assert!(
            parse_arguments(
                ["pgo-merge", "--raw", "raw", "--output", "out"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
        assert!(
            parse_arguments(
                [
                    "compare",
                    "--baseline",
                    "base",
                    "--candidate",
                    "candidate",
                    "--candidate-kind",
                    "baseline",
                    "--output",
                    "out",
                ]
                .into_iter()
                .map(str::to_owned)
            )
            .is_err()
        );
    }

    #[test]
    fn exact_build_match_rejects_replaced_executable() {
        let stamped = valid_baseline_build();
        let mut current = stamped.clone();
        current.executable_sha256 = "c".repeat(64);

        assert_eq!(
            require_exact_build_match(&stamped, &current),
            Err("CODEGEN_CURRENT_EXECUTABLE_HASH_MISMATCH".to_owned())
        );
    }

    #[test]
    fn raw_baseline_report_without_sidecar_is_rejected() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "nextengine-codegen-raw-baseline-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create test directory");
        let report = root.join(PERFORMANCE_REPORT_FILE_NAME);
        fs::write(&report, b"{}").expect("write raw report");

        let error = validate_run_sidecar(&report, &valid_baseline_build())
            .expect_err("raw baseline report must be rejected");
        fs::remove_dir_all(&root).expect("remove test directory");

        assert!(error.contains("CODEGEN_RUN_PROVENANCE_MISSING_OR_UNREADABLE"));
    }

    #[test]
    fn pgo_profile_replacement_after_build_is_rejected() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nextengine-codegen-profile-{}-{unique}.profdata",
            std::process::id()
        ));
        fs::write(&path, b"profile-a").expect("write profile A");
        let build_time_hash = hash_file(&path).expect("hash profile A");
        fs::write(&path, b"profile-b").expect("replace with profile B");

        let error = verified_pgo_profile_sha256(&path, &build_time_hash)
            .expect_err("replaced PGO profile must be rejected");
        fs::remove_file(&path).expect("remove test profile");

        assert!(error.contains("CODEGEN_PGO_PROFILE_CHANGED_SINCE_BUILD"));
    }
}
