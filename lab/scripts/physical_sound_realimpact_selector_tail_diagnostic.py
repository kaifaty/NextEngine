#!/usr/bin/env python3
"""Diagnose selector composition and fixed-tail timing on the opened Iron block."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
from typing import Any

import numpy as np

import physical_sound_multioutput_decay_control as multioutput
import physical_sound_realimpact_independent_observation_execute as execution
import physical_sound_realimpact_independent_observation_protocol as protocol
import physical_sound_realimpact_pitcher_calibration as single
import physical_sound_salience_selector_control as salience


MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-selector-tail-diagnostic.manifest.v1"
)
REPORT_SCHEMAS = {
    "preflight": (
        "nextengine.experimental-realimpact-selector-tail-diagnostic-preflight.report.v1"
    ),
    "analyze": "nextengine.experimental-realimpact-selector-tail-diagnostic.report.v1",
}
STUDY_ID = "physical-sound-realimpact-selector-tail-diagnostic"
REVISION = "iron-skillet-selector-tail-ablation-v1"
OBJECT_ID = protocol.OBJECT_ID

PARENT_REPORT_SHA256 = (
    "a9c4ae36ae04bf9a9772cb8a1a6c55e2862ed918eaff3ae6d9a2043acb07206d"
)
DECODE_REPORT_SHA256 = (
    "47acdc35449390f639cda34990520c5f74cb8627d0932079345ca35735ffc099"
)
DECODED_SHA256 = (
    "e26d1df1d547d059bbcbc57453c522b8c23402461feec09ff6149bc37a9c7eeb"
)
DECODED_BYTES = protocol.DECODED_BYTES
DECODED_NAME = execution.DECODED_NAME
ANALYSIS_ROWS = list(range(15))
TAIL_STARTS_MS = [100, 200, 400, 900]
BASELINE_TAIL_START_MS = 900
PERSISTENCE_THRESHOLD = protocol.GATES[
    "minimum_onset_to_tail_persistence_recall"
]
ADAPTIVE_VALID_THRESHOLD = protocol.GATES["minimum_valid_adaptive_fit_fraction"]

SOURCE_HASHES = {
    "lab/scripts/physical_sound_realimpact_independent_observation_execute.py": (
        "0c53553a5288fabd6beb14edc18cb13c1a5494bf5da07ef3ace1049da8be0988"
    ),
    "lab/scripts/physical_sound_realimpact_independent_observation_protocol.py": (
        "7385241d75abe219430267ca21fe82ab15b6706b3368b67cb15127f6c7838273"
    ),
    "lab/scripts/physical_sound_salience_selector_control.py": (
        "62043142a91e5bd4fe815a0b11c76678935a867d57dae90262336c59b5004ed2"
    ),
    "lab/scripts/physical_sound_multioutput_decay_control.py": (
        "f0483c397843994d23793a4f3392dd49d9ce5d90ec218caa1b10f2a846a2068e"
    ),
    "lab/scripts/physical_sound_adaptive_decay_control.py": (
        "fe59b3d6c8845ed9433bab6f225625d53485f4245e7dac5968c4952eb8459c20"
    ),
    "lab/scripts/physical_sound_realimpact_pitcher_calibration.py": (
        "cf93b1fc436d1e04965142e7c846d665529565025b6421f0ea6164c47fd06772"
    ),
}

RESEARCH_SOURCES = [
    {
        "id": "audio-dspy-find-freqs",
        "url": (
            "https://raw.githubusercontent.com/jatinchowdhury18/audio_dspy/"
            "2ad0b05f81b014c27612f6f91087265ee52e9238/audio_dspy/modal_tools.py"
        ),
        "retrieved_sha256": (
            "a1e5b99112a06a7172c8dbaf270954f33d6e69f3041b15bf976e0eb585ccc494"
        ),
        "bounded_claim": (
            "the transferred source selector suppresses a local peak when a stronger "
            "FFT bin exists within plus or minus ten percent"
        ),
    },
    {
        "id": "realimpact-modal-notebook",
        "url": (
            "https://raw.githubusercontent.com/samuel-clarke/RealImpact/"
            "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/modes_dsp_sweep.ipynb"
        ),
        "retrieved_sha256": (
            "2a8ce3c65e6fb3ac304602d3e2841779d917a3cbfdaedf72c9ee5146561fc7b4"
        ),
        "bounded_claim": (
            "REALIMPACT applies the audio_dspy modal workflow to measured impact data"
        ),
    },
    {
        "id": "gabor-esprit-impact-analysis",
        "url": "https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf",
        "retrieved_sha256": (
            "d52c9a3fd1e9d14e23cdabca7c03ac39374a38c05f13ea7836236063d979bdd0"
        ),
        "bounded_claim": (
            "real impact sounds are modeled as damped sinusoids; subband Gabor ESPRIT "
            "extends the analysis horizon, uses ESTER order estimation and discards "
            "insignificant modes after estimation"
        ),
    },
]


class DiagnosticError(RuntimeError):
    """The frozen diagnostic contract or immutable parent lineage failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--stage", required=True, choices=sorted(REPORT_SCHEMAS))
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def json_ready(value: Any) -> Any:
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, dict):
        return {key: json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise DiagnosticError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise DiagnosticError(f"output must remain outside repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise DiagnosticError(f"output must be absent or empty: {resolved}")
    return resolved


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "object": {
            "dataset_object_id": OBJECT_ID,
            "role": "opened_development_diagnostic_only",
            "impact_ordinal": 0,
            "analysis_rows": ANALYSIS_ROWS,
        },
        "parents": {
            "rejected_analysis_a": {
                "path": (
                    "../ps2-realimpact-iron-skillet-observation-v1/"
                    "observation-analysis-a/report.json"
                ),
                "sha256": PARENT_REPORT_SHA256,
            },
            "rejected_analysis_b": {
                "path": (
                    "../ps2-realimpact-iron-skillet-observation-v1/"
                    "observation-analysis-b/report.json"
                ),
                "sha256": PARENT_REPORT_SHA256,
            },
            "decode_report": {
                "path": (
                    "../ps2-realimpact-iron-skillet-observation-v1/"
                    "observation-decode/report.json"
                ),
                "sha256": DECODE_REPORT_SHA256,
            },
            "decoded_block": {
                "path": (
                    "../ps2-realimpact-iron-skillet-observation-v1/"
                    f"observation-decode/{DECODED_NAME}"
                ),
                "sha256": DECODED_SHA256,
                "bytes": DECODED_BYTES,
                "shape": [protocol.ROW_COUNT, protocol.SAMPLE_COUNT],
                "dtype": "<f4",
            },
        },
        "bound_sources": [
            {"path": path, "sha256": digest}
            for path, digest in SOURCE_HASHES.items()
        ],
        "ablation": {
            "onset_rule": (
                "first 15-output spatial-norm sample at or above five percent of maximum"
            ),
            "onset_expected_sample": 29,
            "candidate_selector": {
                "id": salience.REVISION,
                "role": "source-derived plus-or-minus-ten-percent dominance",
            },
            "comparator_selector": {
                "id": "spatial-modal-power-15-v1-top16-fixed-separation",
                "role": "existing non-admission comparator",
            },
            "adaptive_estimator": execution.adaptive.REVISION,
            "tail_start_ms": TAIL_STARTS_MS,
            "baseline_tail_start_ms": BASELINE_TAIL_START_MS,
            "fft_size_samples": single.FFT_SIZE,
            "fft_support_ms": single.FFT_SIZE * 1_000.0 / protocol.SAMPLE_RATE_HZ,
            "matching": "unchanged injective maximum-40-cents assignment",
        },
        "classification": {
            "minimum_valid_adaptive_fit_fraction": ADAPTIVE_VALID_THRESHOLD,
            "minimum_persistence_recall": PERSISTENCE_THRESHOLD,
            "selector_composition_mismatch_rule": (
                "source onset adaptive-valid fraction below threshold while comparator "
                "onset adaptive-valid fraction meets threshold"
            ),
            "fixed_tail_timing_mismatch_rule": (
                "source baseline-900ms persistence below threshold while at least one "
                "predeclared earlier tail meets threshold"
            ),
            "outcomes": [
                "CombinedSelectorAndTailMismatchSupported",
                "SourceSelectorCompositionMismatchSupported",
                "FixedTailTimingMismatchSupported",
                "ExistingBlockAblationInconclusive",
            ],
        },
        "research_sources": RESEARCH_SOURCES,
        "data_policy": {
            "existing_decoded_rows_only": True,
            "new_network_or_payload_access_allowed": False,
            "tail_grid_or_threshold_tuning_allowed": False,
            "best_variant_selection_allowed": False,
            "physics_solver_or_planter_access_allowed": False,
            "quality_admission_or_runtime_credit_allowed": False,
        },
        "stop_rule": (
            "publish the frozen four-way diagnosis twice; do not promote a tail, tune "
            "selectors, fetch another object, run physics or access Planter"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise DiagnosticError("selector-tail diagnostic manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise DiagnosticError(f"parse selector-tail diagnostic manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise DiagnosticError("selector-tail diagnostic manifest changed")
    return data, manifest


def resolve_reference(
    base: Path, ref: dict[str, Any], label: str
) -> tuple[Path, bytes]:
    path = (base / ref["path"]).resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(base.parent):
        raise DiagnosticError(f"{label} escapes Iron diagnostic store")
    if "bytes" in ref:
        if path.stat().st_size != ref["bytes"] or sha256_file(path) != ref["sha256"]:
            raise DiagnosticError(f"{label} identity changed")
        return path, b""
    data = path.read_bytes()
    if sha256_bytes(data) != ref["sha256"]:
        raise DiagnosticError(f"{label} identity changed")
    return path, data


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    result = []
    for ref in manifest["bound_sources"]:
        path = (root / ref["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(root)
            or sha256_file(path) != ref["sha256"]
        ):
            raise DiagnosticError(f"bound source changed: {ref['path']}")
        result.append(
            {"path": ref["path"], "sha256": ref["sha256"], "bytes": path.stat().st_size}
        )
    return result


def validate_parents(base: Path, manifest: dict[str, Any]) -> tuple[dict[str, Any], Path]:
    refs = manifest["parents"]
    _, analysis_a_bytes = resolve_reference(
        base, refs["rejected_analysis_a"], "rejected analysis A"
    )
    _, analysis_b_bytes = resolve_reference(
        base, refs["rejected_analysis_b"], "rejected analysis B"
    )
    _, decode_bytes = resolve_reference(base, refs["decode_report"], "decode report")
    block, _ = resolve_reference(base, refs["decoded_block"], "decoded block")
    analysis_a = json.loads(analysis_a_bytes)
    analysis_b = json.loads(analysis_b_bytes)
    decode = json.loads(decode_bytes)
    candidate = analysis_a.get("candidate_analysis", {})
    comparator = analysis_a.get("diagnostic_comparator", {})
    if (
        analysis_a_bytes != analysis_b_bytes
        or analysis_a.get("decision") != "IndependentObservationMethodTransferRejected"
        or analysis_a.get("gate", {}).get("passed") is not False
        or analysis_a.get("onset_sample") != 29
        or candidate.get("persistence_recall") != 6 / 17
        or candidate.get("valid_adaptive_fit_fraction") != 0.5
        or comparator.get("valid_adaptive_fit_fraction") != 0.9375
        or comparator.get("decaying_mode_fraction") != 0.9375
        or analysis_a.get("network_requests") != 0
        or analysis_a.get("physics_solver_runs") != 0
        or analysis_a.get("planter_payload_bytes_read") != 0
        or decode.get("decision") != "IndependentImpactZeroObservationDecoded"
        or decode.get("decoded_sha256") != DECODED_SHA256
        or decode.get("decoded_bytes") != DECODED_BYTES
        or decode.get("row_count") != protocol.ROW_COUNT
        or decode.get("sample_count") != protocol.SAMPLE_COUNT
        or decode.get("additional_audio_payload_bytes_read") != 0
        or decode.get("planter_payload_bytes_read") != 0
    ):
        raise DiagnosticError("Iron observation parent lineage changed")
    return {
        "rejected_analysis_a_sha256": sha256_bytes(analysis_a_bytes),
        "rejected_analysis_b_sha256": sha256_bytes(analysis_b_bytes),
        "decode_report_sha256": sha256_bytes(decode_bytes),
        "decoded_block_sha256": sha256_file(block),
        "decoded_block_bytes": block.stat().st_size,
    }, block


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "dataset_object_id": OBJECT_ID,
        "impact_ordinal": 0,
        "analysis_rows": ANALYSIS_ROWS,
        "network_requests": 0,
        "additional_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "quality_admission_or_runtime_credit": False,
    }


def serialize_peak(peak: single.SpectralPeak) -> dict[str, float]:
    return {
        "frequency_hz": peak.frequency_hz,
        "relative_level_db": peak.relative_level_db,
    }


def adaptive_summary(
    channels: np.ndarray, peaks: list[single.SpectralPeak]
) -> dict[str, Any]:
    modes = [execution.adaptive_fit(channels, peak) for peak in peaks]
    valid = [mode for mode in modes if mode["valid"]]
    count = len(modes)
    return {
        "selected_mode_count": count,
        "frequencies_hz": [peak.frequency_hz for peak in peaks],
        "valid_fit_count": len(valid),
        "valid_fit_fraction": len(valid) / count if count else 0.0,
        "decaying_mode_fraction": (
            sum(
                mode["valid"] and mode["fit_decay_db_per_second"] < -1.0
                for mode in modes
            )
            / count
            if count
            else 0.0
        ),
        "median_fit_r_squared": (
            single.median([mode["fit_r_squared"] for mode in valid]) if valid else None
        ),
        "modes": modes,
    }


def persistence_summary(
    onset_peaks: list[single.SpectralPeak], tail_peaks: list[single.SpectralPeak]
) -> dict[str, Any]:
    assignments = single.injective_matches(onset_peaks, tail_peaks)
    errors = [
        abs(
            single.cents_between(
                onset_peaks[index].frequency_hz,
                tail_peaks[assignment].frequency_hz,
            )
        )
        for index, assignment in enumerate(assignments)
        if assignment is not None
    ]
    matched = len(errors)
    return {
        "tail_candidate_count": len(tail_peaks),
        "tail_frequencies_hz": [peak.frequency_hz for peak in tail_peaks],
        "assignments": assignments,
        "persistent_mode_count": matched,
        "persistence_recall": matched / len(onset_peaks) if onset_peaks else 0.0,
        "median_frequency_error_cents": single.median(errors) if errors else None,
    }


def tail_ablation(
    channels: np.ndarray,
    onset: int,
    source_onset: list[dict[str, Any]],
    comparator_onset: list[single.SpectralPeak],
) -> list[dict[str, Any]]:
    source_peaks = [
        single.SpectralPeak(item["frequency_hz"], item["relative_level_db"])
        for item in source_onset
    ]
    rows = []
    for tail_start_ms in TAIL_STARTS_MS:
        start = onset + tail_start_ms * protocol.SAMPLE_RATE_HZ // 1_000
        source_local, source_tail = salience.source_derived_candidates(channels, start)
        source_tail_peaks = [
            single.SpectralPeak(item["frequency_hz"], item["relative_level_db"])
            for item in source_tail
        ]
        comparator_tail = multioutput.spatial_modal_peaks(channels, start)
        rows.append(
            {
                "tail_start_ms": tail_start_ms,
                "fft_end_ms_relative_to_onset": (
                    tail_start_ms
                    + single.FFT_SIZE * 1_000.0 / protocol.SAMPLE_RATE_HZ
                ),
                "source_selector": {
                    "local_candidate_count": len(source_local),
                    "salient_candidate_count": len(source_tail),
                    **persistence_summary(source_peaks, source_tail_peaks),
                },
                "comparator_selector": persistence_summary(
                    comparator_onset, comparator_tail
                ),
            }
        )
    return rows


def classify(
    source_adaptive: dict[str, Any],
    comparator_adaptive: dict[str, Any],
    tails: list[dict[str, Any]],
) -> dict[str, Any]:
    baseline = next(
        row for row in tails if row["tail_start_ms"] == BASELINE_TAIL_START_MS
    )
    earlier = [
        row for row in tails if row["tail_start_ms"] < BASELINE_TAIL_START_MS
    ]
    selector_mismatch = (
        source_adaptive["valid_fit_fraction"] < ADAPTIVE_VALID_THRESHOLD
        and comparator_adaptive["valid_fit_fraction"] >= ADAPTIVE_VALID_THRESHOLD
    )
    tail_mismatch = (
        baseline["source_selector"]["persistence_recall"] < PERSISTENCE_THRESHOLD
        and any(
            row["source_selector"]["persistence_recall"] >= PERSISTENCE_THRESHOLD
            for row in earlier
        )
    )
    if selector_mismatch and tail_mismatch:
        decision = "CombinedSelectorAndTailMismatchSupported"
    elif selector_mismatch:
        decision = "SourceSelectorCompositionMismatchSupported"
    elif tail_mismatch:
        decision = "FixedTailTimingMismatchSupported"
    else:
        decision = "ExistingBlockAblationInconclusive"
    return {
        "decision": decision,
        "source_selector_composition_mismatch_supported": selector_mismatch,
        "fixed_tail_timing_mismatch_supported": tail_mismatch,
        "minimum_valid_adaptive_fit_fraction": ADAPTIVE_VALID_THRESHOLD,
        "minimum_persistence_recall": PERSISTENCE_THRESHOLD,
        "baseline_tail_start_ms": BASELINE_TAIL_START_MS,
        "baseline_source_persistence_recall": baseline["source_selector"][
            "persistence_recall"
        ],
        "earlier_source_persistence_recalls": [
            {
                "tail_start_ms": row["tail_start_ms"],
                "persistence_recall": row["source_selector"]["persistence_recall"],
            }
            for row in earlier
        ],
    }


def preflight(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parents, _ = validate_parents(base, manifest)
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "IronSkilletSelectorTailDiagnosticFrozen",
        "claim": (
            "EXISTING_BLOCK_SELECTOR_AND_TAIL_ABLATION_PREFLIGHT_ONLY / "
            "NO_TUNING_SELECTION_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parents": parents,
        "bound_sources": validate_sources(root, manifest),
        "ablation": manifest["ablation"],
        "classification": manifest["classification"],
        "research_sources": manifest["research_sources"],
        "decoded_payload_bytes_verified": DECODED_BYTES,
        "next_action": (
            "commit this preflight, then execute the unchanged four-way diagnostic twice"
        ),
    }


def analyze(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    parents, block = validate_parents(base, manifest)
    validate_sources(root, manifest)
    rows = np.memmap(
        block,
        dtype="<f4",
        mode="r",
        shape=(protocol.ROW_COUNT, protocol.SAMPLE_COUNT),
    )
    channels = np.asarray(rows[:15], dtype=np.float64)
    onset = execution.detect_onset(channels)
    if onset != manifest["ablation"]["onset_expected_sample"]:
        raise DiagnosticError("Iron diagnostic onset changed")
    source_local, source_onset = salience.source_derived_candidates(channels, onset)
    source_peaks = [
        single.SpectralPeak(item["frequency_hz"], item["relative_level_db"])
        for item in source_onset
    ]
    comparator_peaks = multioutput.spatial_modal_peaks(channels, onset)
    source_adaptive = adaptive_summary(channels, source_peaks)
    comparator_adaptive = adaptive_summary(channels, comparator_peaks)
    tails = tail_ablation(channels, onset, source_onset, comparator_peaks)
    classification = classify(source_adaptive, comparator_adaptive, tails)
    baseline = next(
        row for row in tails if row["tail_start_ms"] == BASELINE_TAIL_START_MS
    )
    if baseline["source_selector"]["persistence_recall"] != 6 / 17:
        raise DiagnosticError("baseline source persistence no longer matches parent")
    return {
        "schema": REPORT_SCHEMAS["analyze"],
        "status": "Validated",
        "decision": classification["decision"],
        "claim": (
            "EXISTING_BLOCK_SELECTOR_AND_TAIL_CAUSAL_DIAGNOSTIC_ONLY / "
            "NO_VARIANT_PROMOTION_TUNING_PHYSICS_QUALITY_ADMISSION_OR_RUNTIME_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "parents": parents,
        "decoded_payload_bytes_verified": DECODED_BYTES,
        "onset_sample": onset,
        "source_selector": {
            "local_onset_candidate_count": len(source_local),
            "salient_onset_candidate_count": len(source_onset),
            "salient_onset": source_onset,
            "adaptive": source_adaptive,
        },
        "comparator_selector": {
            "admission_role": False,
            "onset_peaks": [serialize_peak(peak) for peak in comparator_peaks],
            "adaptive": comparator_adaptive,
        },
        "tail_ablation": tails,
        "classification": classification,
        "next_action": (
            "interpret only the frozen four-way classification; do not promote an "
            "earlier tail or selector variant from this opened object"
        ),
    }


def publish(output: Path, report: dict[str, Any]) -> bytes:
    report_bytes = canonical_json(report)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    try:
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return report_bytes


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    manifest_path = external_file(root, arguments.manifest, "manifest")
    output = external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    report = (
        preflight(root, base, manifest_bytes, manifest, runner_sha256)
        if arguments.stage == "preflight"
        else analyze(root, base, manifest_bytes, manifest, runner_sha256)
    )
    report_bytes = publish(output, report)
    print(f"Iron selector-tail diagnostic {arguments.stage}: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("additional payload bytes read: 0")
    print("physics solver runs: 0")
    print("Planter payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
