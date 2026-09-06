#!/usr/bin/env python3
"""Official confound-resistant metric calibration for Physical Sound V20 M0b."""

from __future__ import annotations

import argparse
import json
import math
import resource
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v20_m0_metric_lab as m0_lab
import physical_sound_v20_m0b_common as common
from scipy.signal import hilbert
from scipy.stats import spearmanr

SAMPLE_RATE_HZ = m0_lab.SAMPLE_RATE_HZ
SAMPLE_COUNT = m0_lab.SAMPLE_COUNT
CASE_COUNT = m0_lab.CASE_COUNT
BATCH_SIZE = m0_lab.BATCH_SIZE
IDENTITY_TOLERANCE = m0_lab.IDENTITY_TOLERANCE
SEPARATION_MARGIN = m0_lab.SEPARATION_MARGIN
SPEARMAN_MINIMUM = m0_lab.SPEARMAN_MINIMUM
EDC_ACTIVE_EPSILON = 1.0e-12
EDC_LOG_FLOOR = 1.0e-10
STFT_RESOLUTIONS = m0_lab.STFT_RESOLUTIONS
ACOUSTIC_METRICS = (
    "mrsc",
    "mean_centered_log_magnitude",
    "decay_slope_residual",
    "transient_energy",
)
SUMMARY_METRICS = (
    *ACOUSTIC_METRICS,
    "raw_mrlm",
    "absolute_decay_energy",
    "early_waveform_nrmse",
    "full_waveform_nrmse",
    "hilbert_envelope_nrmse",
    "frequency_cents_median",
    "frequency_cents_p95",
    "frequency_cents_max",
    "damping_relative_median",
    "damping_relative_p95",
    "gain_nrmse",
    "missing_mode_count",
)
OUTPUT_FILES = {
    "access-ledger.json",
    "corpus.json",
    "manifest.json",
    "metrics.jsonl",
    "report.json",
}


@dataclass(frozen=True)
class TruthCache:
    base: m0_lab.TruthCache
    edc_slope: np.ndarray
    edc_active: np.ndarray


def control_specs() -> tuple[m0_lab.ControlSpec, ...]:
    return m0_lab.control_specs()


def build_corpus(
    dependencies: common.m0_common.i0_common.LoadedDependencies,
) -> m0_lab.EvaluationCorpus:
    corpus = m0_lab.build_corpus(dependencies)
    if any(item.row.role != "development" for item in corpus.objects):
        raise common.M0bError("M0b attempted to generate a non-development role")
    return corpus


def _centered_log_magnitude(magnitude: np.ndarray) -> np.ndarray:
    value = np.log(np.asarray(magnitude, dtype=np.float64) + 1.0e-7)
    return value - np.mean(value, axis=2, keepdims=True)


def _edc_slope(magnitude: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    frequency = np.fft.rfftfreq(512, d=1.0 / SAMPLE_RATE_HZ)
    band_energy = []
    for band_index, (start, end) in enumerate(m0_lab.DECAY_FREQUENCY_BANDS_HZ):
        if band_index == len(m0_lab.DECAY_FREQUENCY_BANDS_HZ) - 1:
            mask = (frequency >= start) & (frequency <= end)
        else:
            mask = (frequency >= start) & (frequency < end)
        band_energy.append(np.sum(magnitude[:, :, mask] ** 2, axis=2))
    energy = np.stack(band_energy, axis=1)
    cumulative = np.cumsum(energy[:, :, ::-1], axis=2)[:, :, ::-1]
    active = cumulative[:, :, 0] > EDC_ACTIVE_EPSILON
    denominator = np.maximum(cumulative[:, :, :1], 1.0e-300)
    normalized = cumulative / denominator
    log_curve = np.log(np.maximum(normalized, EDC_LOG_FLOOR))
    slope = np.diff(log_curve, axis=2)
    expected_frames = m0_lab._stft_plan(512, 128)[0].size - 1
    if slope.shape[1:] != (3, expected_frames):
        raise common.M0bError(f"M0b EDC slope shape changed: {slope.shape}")
    if not np.isfinite(slope).all():
        raise common.M0bError("M0b EDC slope is non-finite")
    return slope, active


def build_truth_cache(corpus: m0_lab.EvaluationCorpus) -> TruthCache:
    base = m0_lab.build_truth_cache(corpus)
    slope, active = _edc_slope(base.stft_magnitudes[512])
    if np.any(np.sum(active, axis=1) == 0):
        raise common.M0bError("M0b truth has no active EDC band")
    return TruthCache(base=base, edc_slope=slope, edc_active=active)


def _acoustic_batch(
    prediction: np.ndarray, truth: TruthCache, start: int, end: int
) -> dict[str, np.ndarray]:
    base = truth.base
    normalized = prediction / base.rms[start:end, None]
    mrsc_rows = []
    raw_mrlm_rows = []
    centered_rows = []
    predicted_512 = None
    for fft_length, window_length, hop in STFT_RESOLUTIONS:
        predicted = m0_lab._stft_magnitude(normalized, window_length, hop)
        expected = base.stft_magnitudes[fft_length][start:end]
        delta = predicted - expected
        denominator = np.maximum(np.sqrt(np.sum(expected**2, axis=(1, 2))), 1.0e-12)
        mrsc_rows.append(np.sqrt(np.sum(delta**2, axis=(1, 2))) / denominator)
        raw_mrlm_rows.append(
            np.mean(
                np.abs(np.log(predicted + 1.0e-7) - np.log(expected + 1.0e-7)),
                axis=(1, 2),
            )
        )
        predicted_centered = _centered_log_magnitude(predicted)
        expected_centered = _centered_log_magnitude(expected)
        centered_rows.append(
            np.mean(np.abs(predicted_centered - expected_centered), axis=(1, 2))
        )
        if fft_length == 512:
            predicted_512 = predicted
    if predicted_512 is None:
        raise common.M0bError("M0b decay STFT resolution is absent")

    predicted_decay = m0_lab._decay_cells(predicted_512)
    decay_active = base.decay_active[start:end]
    absolute_decay_delta = np.log(predicted_decay + 1.0e-10) - np.log(
        base.decay_energy[start:end] + 1.0e-10
    )
    absolute_decay = np.sqrt(
        np.sum(np.where(decay_active, absolute_decay_delta**2, 0.0), axis=1)
        / np.sum(decay_active, axis=1)
    )

    predicted_slope, _predicted_active = _edc_slope(predicted_512)
    edc_active = truth.edc_active[start:end]
    slope_mask = np.repeat(edc_active[:, :, None], predicted_slope.shape[2], axis=2)
    slope_delta = predicted_slope - truth.edc_slope[start:end]
    decay_slope = np.sqrt(
        np.sum(np.where(slope_mask, slope_delta**2, 0.0), axis=(1, 2))
        / np.sum(slope_mask, axis=(1, 2))
    )

    transient = m0_lab._transient_fractions(normalized)
    transient_energy = np.sqrt(
        np.mean((transient - base.transient_fractions[start:end]) ** 2, axis=1)
    )
    truth_normalized = base.normalized[start:end]
    early_denominator = np.sqrt(np.mean(truth_normalized[:, :1_024] ** 2, axis=1))
    full_denominator = np.sqrt(np.mean(truth_normalized**2, axis=1))
    predicted_envelope = np.abs(hilbert(normalized, axis=1))
    envelope_denominator = np.sqrt(np.mean(base.envelope[start:end] ** 2, axis=1))
    result = {
        "absolute_decay_energy": absolute_decay,
        "decay_active_bands": np.sum(edc_active, axis=1),
        "decay_omitted_bands": 3 - np.sum(edc_active, axis=1),
        "decay_slope_residual": decay_slope,
        "early_waveform_nrmse": np.sqrt(
            np.mean(
                (normalized[:, :1_024] - truth_normalized[:, :1_024]) ** 2,
                axis=1,
            )
        )
        / early_denominator,
        "full_waveform_nrmse": np.sqrt(
            np.mean((normalized - truth_normalized) ** 2, axis=1)
        )
        / full_denominator,
        "hilbert_envelope_nrmse": np.sqrt(
            np.mean((predicted_envelope - base.envelope[start:end]) ** 2, axis=1)
        )
        / envelope_denominator,
        "mean_centered_log_magnitude": np.mean(np.stack(centered_rows, axis=1), axis=1),
        "mrsc": np.mean(np.stack(mrsc_rows, axis=1), axis=1),
        "raw_mrlm": np.mean(np.stack(raw_mrlm_rows, axis=1), axis=1),
        "transient_energy": transient_energy,
    }
    if not all(np.isfinite(value).all() for value in result.values()):
        raise common.M0bError("M0b acoustic metric produced a non-finite value")
    return result


def evaluate_controls(
    corpus: m0_lab.EvaluationCorpus,
    truth: TruthCache,
    controls: tuple[m0_lab.ControlSpec, ...],
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for spec in controls:
        for start in range(0, CASE_COUNT, BATCH_SIZE):
            end = min(start + BATCH_SIZE, CASE_COUNT)
            waveform, frequencies, damping, gains = m0_lab._render_control(
                corpus, truth.base, spec, start, end
            )
            acoustic = _acoustic_batch(waveform, truth, start, end)
            physical = m0_lab._physical_batch(
                corpus, frequencies, damping, gains, start, end
            )
            for local_index, case_index in enumerate(range(start, end)):
                rows.append(
                    {
                        **corpus.case_records[case_index],
                        "classification": spec.classification,
                        "control": spec.name,
                        "control_family": spec.family,
                        "control_severity": spec.severity,
                        **{
                            name: int(value[local_index])
                            if name
                            in (
                                "decay_active_bands",
                                "decay_omitted_bands",
                                "missing_mode_count",
                            )
                            else float(value[local_index])
                            for name, value in {**acoustic, **physical}.items()
                        },
                        "schema": common.METRIC_SCHEMA,
                    }
                )
    if len(rows) != CASE_COUNT * len(controls):
        raise common.M0bError("M0b metric cell count changed")
    return rows


def _summaries(
    rows: list[dict[str, Any]], controls: tuple[m0_lab.ControlSpec, ...]
) -> dict[str, dict[str, Any]]:
    by_control = {item.name: [] for item in controls}
    for row in rows:
        by_control[row["control"]].append(row)
    result: dict[str, dict[str, Any]] = {}
    for spec in controls:
        values = by_control[spec.name]
        if len(values) != CASE_COUNT:
            raise common.M0bError(f"M0b control {spec.name} is incomplete")
        metrics = {}
        for name in SUMMARY_METRICS:
            array = np.asarray([row[name] for row in values], dtype=np.float64)
            metrics[name] = {
                "max": float(np.max(array)),
                "mean": float(np.mean(array)),
                "median": float(np.median(array)),
                "min": float(np.min(array)),
                "p95": float(np.quantile(array, 0.95)),
            }
        result[spec.name] = {
            **spec.record(),
            "case_count": len(values),
            "decay_active_band_count": int(
                sum(row["decay_active_bands"] for row in values)
            ),
            "decay_omitted_band_count": int(
                sum(row["decay_omitted_bands"] for row in values)
            ),
            "metrics": metrics,
        }
    return result


def _family_specs(
    controls: tuple[m0_lab.ControlSpec, ...],
    family: str,
    classification: str | None = None,
) -> list[m0_lab.ControlSpec]:
    return [
        item
        for item in controls
        if item.family == family
        and (classification is None or item.classification == classification)
    ]


def _separation_records(
    summaries: dict[str, dict[str, Any]],
    controls: tuple[m0_lab.ControlSpec, ...],
) -> dict[str, dict[str, Any]]:
    base = ["identity", "b0-only", "f0-only", "combined"]
    assignments = (
        ("frequency-uniform", "mrsc"),
        ("frequency-uniform", "mean_centered_log_magnitude"),
        ("frequency-alternating", "mrsc"),
        ("frequency-alternating", "mean_centered_log_magnitude"),
        ("damping-positive", "decay_slope_residual"),
        ("damping-negative", "decay_slope_residual"),
        ("mode-removal", "mrsc"),
        ("mode-removal", "mean_centered_log_magnitude"),
        ("onset-delay", "transient_energy"),
        ("sample-zero-impulse", "transient_energy"),
    )
    result = {}
    for family, metric in assignments:
        acceptable_names = base + [
            item.name for item in _family_specs(controls, family, "acceptable")
        ]
        harmful_names = [
            item.name for item in _family_specs(controls, family, "harmful")
        ]
        acceptable_values = [
            summaries[name]["metrics"][metric]["p95"] for name in acceptable_names
        ]
        harmful_values = [
            summaries[name]["metrics"][metric]["p95"] for name in harmful_names
        ]
        if not harmful_values:
            raise common.M0bError(
                f"M0b separation family has no harmful control: {family}"
            )
        maximum_acceptable = max(acceptable_values)
        minimum_harmful = min(harmful_values)
        ratio = (
            math.inf
            if minimum_harmful <= 0.0 and maximum_acceptable > 0.0
            else maximum_acceptable / max(minimum_harmful, 1.0e-300)
        )
        key = f"{family}:{metric}"
        result[key] = {
            "acceptable_controls": sorted(set(acceptable_names)),
            "aggregate": "corpus_p95",
            "family": family,
            "harmful_controls": sorted(harmful_names),
            "margin": SEPARATION_MARGIN,
            "maximum_acceptable": maximum_acceptable,
            "metric": metric,
            "minimum_harmful": minimum_harmful,
            "pass": bool(maximum_acceptable <= SEPARATION_MARGIN * minimum_harmful),
            "ratio": ratio,
        }
    return result


def _severity_correlations(
    summaries: dict[str, dict[str, Any]],
    controls: tuple[m0_lab.ControlSpec, ...],
) -> dict[str, dict[str, Any]]:
    assignments = (
        ("frequency-uniform", "mrsc"),
        ("frequency-uniform", "mean_centered_log_magnitude"),
        ("frequency-alternating", "mrsc"),
        ("frequency-alternating", "mean_centered_log_magnitude"),
        ("damping-positive", "decay_slope_residual"),
        ("damping-negative", "decay_slope_residual"),
        ("gain-scale", "gain_nrmse"),
        ("mode-removal", "mrsc"),
        ("mode-removal", "mean_centered_log_magnitude"),
        ("onset-delay", "transient_energy"),
        ("sample-zero-impulse", "transient_energy"),
    )
    result = {}
    for family, metric in assignments:
        ladder = _family_specs(controls, family)
        severity = np.asarray([item.severity for item in ladder], dtype=np.float64)
        values = np.asarray(
            [summaries[item.name]["metrics"][metric]["p95"] for item in ladder],
            dtype=np.float64,
        )
        correlation = float(spearmanr(severity, values).statistic)
        if not math.isfinite(correlation):
            raise common.M0bError(f"M0b severity correlation is non-finite: {family}")
        key = f"{family}:{metric}"
        result[key] = {
            "aggregate": "corpus_p95",
            "correlation": correlation,
            "family": family,
            "metric": metric,
            "minimum": SPEARMAN_MINIMUM,
            "pass": correlation >= SPEARMAN_MINIMUM,
        }
    return result


def _predecessor_reproduction(
    summaries: dict[str, dict[str, Any]], predecessor: common.M0Predecessor
) -> dict[str, Any]:
    predecessor_summaries = predecessor.report["control_summaries"]
    metric_map = {
        "mrsc": "mrsc",
        "raw_mrlm": "mrlm",
        "absolute_decay_energy": "decay_energy",
        "transient_energy": "transient_energy",
    }
    deltas = []
    for control_name, summary in summaries.items():
        expected = predecessor_summaries[control_name]
        for current_metric, old_metric in metric_map.items():
            for aggregate in ("max", "mean", "median", "min", "p95"):
                deltas.append(
                    abs(
                        summary["metrics"][current_metric][aggregate]
                        - expected["metrics"][old_metric][aggregate]
                    )
                )
    maximum_delta = max(deltas)
    failed = {
        key: value["ratio"]
        for key, value in predecessor.report["separation"].items()
        if not value["pass"]
    }
    expected_failed = {
        "damping-negative:decay_energy",
        "damping-positive:decay_energy",
        "frequency-alternating:mrlm",
        "frequency-uniform:mrlm",
    }
    return {
        "failed_separation_ratios": failed,
        "maximum_absolute_delta": maximum_delta,
        "pass": bool(
            maximum_delta <= IDENTITY_TOLERANCE
            and set(failed) == expected_failed
            and predecessor.report["gates"]["metric_separation"] is False
        ),
        "predecessor_tree_digest": common.M0_TREE_DIGEST,
    }


def _finite(value: Any) -> bool:
    if value is None or isinstance(value, (str, bool, int)):
        return True
    if isinstance(value, float):
        return math.isfinite(value)
    if isinstance(value, dict):
        return all(_finite(item) for item in value.values())
    if isinstance(value, (list, tuple)):
        return all(_finite(item) for item in value)
    return False


def _metric_roundtrip_exact(rows: list[dict[str, Any]]) -> bool:
    payload = common.canonical_json_lines(rows)
    decoded = [json.loads(line) for line in payload.decode("utf-8").splitlines()]
    return decoded == rows


def _gates(
    rows: list[dict[str, Any]],
    summaries: dict[str, dict[str, Any]],
    controls: tuple[m0_lab.ControlSpec, ...],
    separation: dict[str, dict[str, Any]],
    correlations: dict[str, dict[str, Any]],
    components: dict[str, Any],
    legacy: dict[str, Any],
    predecessor: dict[str, Any],
    roundtrip_exact: bool,
) -> tuple[dict[str, bool], dict[str, bool]]:
    identity = summaries["identity"]["metrics"]
    polarity = summaries["global-polarity"]["metrics"]
    physical = m0_lab._physical_control_gates(summaries)
    polarity_gain = polarity["gain_nrmse"]
    physical["global_polarity_signed_gain_rejects"] = not (
        polarity_gain["mean"] <= 0.25 and polarity_gain["max"] <= 0.40
    )
    expected_cells = CASE_COUNT * len(controls)
    gates = {
        "component_hard_physical": bool(components["pass"]),
        "complete_finite": bool(
            len(rows) == expected_cells
            and len(summaries) == len(controls)
            and all(value["case_count"] == CASE_COUNT for value in summaries.values())
            and _finite(rows)
            and _finite(summaries)
        ),
        "harmful_physical_owners": all(physical.values()),
        "identity_zero": all(
            identity[name]["max"] <= IDENTITY_TOLERANCE for name in ACOUSTIC_METRICS
        ),
        "legacy_attribution": bool(legacy["pass"]),
        "metric_separation": all(value["pass"] for value in separation.values()),
        "polarity_invariance": bool(
            all(
                polarity[name]["max"] <= IDENTITY_TOLERANCE for name in ACOUSTIC_METRICS
            )
            and polarity["full_waveform_nrmse"]["min"] >= 1.9
        ),
        "predecessor_failure_reproduced": bool(predecessor["pass"]),
        "resource_ceiling_enforced": True,
        "serialization_roundtrip": roundtrip_exact,
        "severity_monotonicity": all(value["pass"] for value in correlations.values()),
        "zero_sealed_access": all(value == 0 for value in common.ZERO_ACCESS.values()),
    }
    return gates, physical


def _corpus_record(corpus: m0_lab.EvaluationCorpus) -> dict[str, Any]:
    record = m0_lab._corpus_record(corpus)
    record["schema"] = common.CORPUS_SCHEMA
    record["study_id"] = common.STUDY_ID
    return record


def run(m0_root: Path, b0_root: Path, f0_root: Path, output: Path) -> dict[str, Any]:
    started = time.monotonic()
    environment = common.verify_protocol_environment()
    implementation_commit = common.verify_committed_implementation()
    implementation = common.implementation_hashes()
    predecessor_artifact = common.load_predecessor(m0_root)
    dependencies = common.load_dependencies(b0_root, f0_root)
    staging, target = common.prepare_output(output)
    try:
        corpus = build_corpus(dependencies)
        components = m0_lab._component_evidence(dependencies, corpus)
        legacy = m0_lab._legacy_attribution(corpus)
        controls = control_specs()
        truth = build_truth_cache(corpus)
        rows = evaluate_controls(corpus, truth, controls)
        summaries = _summaries(rows, controls)
        separation = _separation_records(summaries, controls)
        correlations = _severity_correlations(summaries, controls)
        predecessor = _predecessor_reproduction(summaries, predecessor_artifact)
        roundtrip_exact = _metric_roundtrip_exact(rows)
        gates, physical_controls = _gates(
            rows,
            summaries,
            controls,
            separation,
            correlations,
            components,
            legacy,
            predecessor,
            roundtrip_exact,
        )
        report = {
            "component_evidence": components,
            "control_summaries": summaries,
            "gates": gates,
            "legacy_attribution": legacy,
            "metric_contract": {
                "edc_active_epsilon": EDC_ACTIVE_EPSILON,
                "edc_log_floor": EDC_LOG_FLOOR,
                "separation_aggregate": "corpus_p95",
                "separation_margin": SEPARATION_MARGIN,
                "severity_aggregate": "corpus_p95",
                "severity_spearman_minimum": SPEARMAN_MINIMUM,
            },
            "physical_control_gates": physical_controls,
            "predecessor_reproduction": predecessor,
            "schema": common.REPORT_SCHEMA,
            "separation": separation,
            "severity_correlations": correlations,
            "single_run_pass": all(gates.values()),
            "study_id": common.STUDY_ID,
        }
        access = {
            **common.ZERO_ACCESS,
            "b0_artifact_bytes_read": dependencies.b0_bytes_read,
            "development_control_count": len(controls),
            "development_views_generated": len(corpus.objects),
            "development_waveform_cases_evaluated": len(corpus.case_records),
            "f0_artifact_bytes_read": dependencies.f0_bytes_read,
            "m0_predecessor_artifact_bytes_read": predecessor_artifact.bytes_read,
            "schema": common.ACCESS_SCHEMA,
        }
        files = {
            "access-ledger.json": common.canonical_json(access),
            "corpus.json": common.canonical_json(_corpus_record(corpus)),
            "metrics.jsonl": common.canonical_json_lines(rows),
            "report.json": common.canonical_json(report),
        }
        artifact_hashes = {
            name: common.sha256_bytes(payload)
            for name, payload in sorted(files.items())
        }
        manifest = {
            "artifact_hashes": artifact_hashes,
            "b0_tree_digest": common.B0_TREE_DIGEST,
            "development_row_root": common.DEVELOPMENT_ROW_ROOT,
            "environment": environment,
            "f0_tree_digest": common.F0_TREE_DIGEST,
            "implementation_commit": implementation_commit,
            "implementation_hashes": implementation,
            "m0_implementation_commit": common.M0_IMPLEMENTATION_COMMIT,
            "m0_implementation_hashes": common.M0_IMPLEMENTATION_HASHES,
            "m0_predecessor_tree_digest": common.M0_TREE_DIGEST,
            "protocol_sha256": common.PROTOCOL_SHA256,
            "revision": common.REVISION,
            "schema": common.MANIFEST_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        files["manifest.json"] = common.canonical_json(manifest)
        if set(files) != OUTPUT_FILES:
            raise common.M0bError(f"M0b output file set changed: {sorted(files)}")
        total_bytes = sum(len(payload) for payload in files.values())
        if total_bytes > 100 * 1024 * 1024:
            raise common.M0bError("M0b output byte ceiling exceeded")
        if time.monotonic() - started > 600.0:
            raise common.M0bError("M0b runtime ceiling exceeded")
        if resource.getrusage(resource.RUSAGE_SELF).ru_maxrss > 4 * 1024 * 1024:
            raise common.M0bError("M0b RSS ceiling exceeded")
        common.write_files(staging, files)
        if time.monotonic() - started > 600.0:
            raise common.M0bError("M0b runtime ceiling exceeded after serialization")
        common.publish_output(staging, target)
        return {
            "file_count": len(files),
            "output": str(target),
            "passed": all(gates.values()),
            "report_sha256": artifact_hashes["report.json"],
            "tree_digest": common.tree_digest(common.directory_file_map(target)),
        }
    except Exception:
        common.abandon_output(staging)
        raise


def compare(left: Path, right: Path) -> dict[str, Any]:
    result = common.compare_directories(left, right)
    if result["file_count"] != len(OUTPUT_FILES):
        raise common.M0bError("M0b compared output file count changed")
    return result


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("--m0-root", type=Path, required=True)
    run_parser.add_argument("--b0-root", type=Path, required=True)
    run_parser.add_argument("--f0-root", type=Path, required=True)
    run_parser.add_argument("--output", type=Path, required=True)
    compare_parser = subparsers.add_parser("compare")
    compare_parser.add_argument("--left", type=Path, required=True)
    compare_parser.add_argument("--right", type=Path, required=True)
    return parser


def main() -> int:
    arguments = _parser().parse_args()
    try:
        if arguments.command == "run":
            result = run(
                arguments.m0_root,
                arguments.b0_root,
                arguments.f0_root,
                arguments.output,
            )
        else:
            result = compare(arguments.left, arguments.right)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0
    except (common.M0bError, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
