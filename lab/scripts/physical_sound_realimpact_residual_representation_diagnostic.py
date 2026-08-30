#!/usr/bin/env python3
"""Diagnose the rejected REALIMPACT metal residual without opening new data."""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import math
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_realimpact_metal_batch_execute as execution

SCHEMA = (
    "nextengine.experimental-realimpact-residual-representation-diagnostic.report.v1"
)
REVISION = "opened-metal-residual-representation-diagnostic-v1"
EXECUTION_RUNNER_SHA256 = (
    "8484f9477b6ba9282bce3b6ce08185411c145bc095efa24c8cbe85b53db413cc"
)
MANIFEST_SHA256 = "bffaa21c58a14058ad4d05597c4406b24dd0b969a10d9c6b5b447e92058a8664"
FIT_DECODE_SHA256 = "af7af9fb84824e3046e90c201f464f904cd05b58b513ec9cc6cfb567d94e068a"
SELECTION_SHA256 = "83f858bf8822f12649677b21845d3755ffcc12580417e9779aab9974d1ce43f2"
HOLDOUT_DECODE_SHA256 = (
    "5fd5ec04af4bd719419671ec026b6ea13ba827de11483317872463457238a633"
)
HOLDOUT_SHA256 = "fef9dd344836b8ab05935d9c70892e544f306e6844ea8c09528add67ad00cb73"
CANDIDATE_ID = "modal_plus_seeded_subband_residual_v0"

ANALYSIS_STOP = 32_768
FRAME_SAMPLES = 512
HOP_SAMPLES = 256
MODULATION_EDGES_HZ = [0.0, 3.0, 12.0, 30.0, 60.0, 93.75]
AUTOCORRELATION_LAGS = [48, 96, 192, 384]
POWER_FLOOR_RATIO = 1.0e-8

# Frozen diagnostic thresholds. They classify a missing statistic; they never
# admit a candidate, alter the rejected holdout or authorize shadow access.
HYPOTHESIS_THRESHOLDS = {
    "within_band_coloration": {
        "minimum_median_spectral_shape_mae_db": 3.0,
        "minimum_median_autocorrelation_mae": 0.05,
    },
    "temporal_modulation": {
        "minimum_median_modulation_power_mae_db": 3.0,
        "minimum_median_adjacent_band_envelope_correlation_mae": 0.15,
    },
    "cross_listener_coupling": {
        "minimum_median_coherence_mae": 0.15,
        "minimum_median_effective_rank_gap": 1.0,
        "maximum_median_candidate_effective_rank": 1.5,
    },
}
HYPOTHESIS_PRIORITY = [
    "within_band_coloration",
    "temporal_modulation",
    "cross_listener_coupling",
]


class DiagnosticError(RuntimeError):
    """The frozen diagnostic lineage or numeric envelope failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["preflight", "diagnose"])
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--fit-decode", required=True, type=Path)
    parser.add_argument("--selection", required=True, type=Path)
    parser.add_argument("--holdout-decode", required=True, type=Path)
    parser.add_argument("--holdout-report", required=True, type=Path)
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


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise DiagnosticError(f"output must be a new external file: {resolved}")
    return resolved


def report_from_directory(
    root: Path,
    path: Path,
    label: str,
    decisions: set[str],
    expected_sha256: str,
) -> tuple[bytes, dict[str, Any]]:
    _, report_bytes, report = execution.report_from_directory(
        root,
        path,
        label,
        manifest_sha256=MANIFEST_SHA256,
        decisions=decisions,
    )
    observed = sha256_bytes(report_bytes)
    if observed != expected_sha256:
        raise DiagnosticError(f"{label} hash changed: {observed}")
    return report_bytes, report


def validate_lineage(
    root: Path, arguments: argparse.Namespace
) -> tuple[bytes, dict[str, Any], dict[str, Any]]:
    execution_runner = (
        root / "lab/scripts/physical_sound_realimpact_metal_batch_execute.py"
    )
    if sha256_file(execution_runner) != EXECUTION_RUNNER_SHA256:
        raise DiagnosticError("bound metal-batch execution runner changed")
    manifest_path = execution.external_file(root, arguments.manifest, "manifest")
    manifest_bytes, manifest = execution.load_manifest(
        manifest_path, EXECUTION_RUNNER_SHA256
    )
    if sha256_bytes(manifest_bytes) != MANIFEST_SHA256:
        raise DiagnosticError("execution manifest hash changed")

    fit_bytes, _ = report_from_directory(
        root,
        arguments.fit_decode,
        "fit decode",
        {"RealImpactMetalBatchFitInputsDecoded"},
        FIT_DECODE_SHA256,
    )
    selection_bytes, selection = report_from_directory(
        root,
        arguments.selection,
        "candidate selection",
        {"RealImpactMetalBatchCandidateSelected"},
        SELECTION_SHA256,
    )
    holdout_decode_bytes, _ = report_from_directory(
        root,
        arguments.holdout_decode,
        "holdout decode",
        {"RealImpactMetalBatchHoldoutInputsDecoded"},
        HOLDOUT_DECODE_SHA256,
    )
    holdout_bytes, holdout = report_from_directory(
        root,
        arguments.holdout_report,
        "holdout evaluation",
        {"RealImpactMetalBatchHoldoutRejected"},
        HOLDOUT_SHA256,
    )
    if (
        selection.get("selected_candidate_id") != CANDIDATE_ID
        or holdout.get("selected_candidate_id") != CANDIDATE_ID
        or holdout.get("gate", {}).get("passed") is not False
    ):
        raise DiagnosticError("selected candidate or rejected holdout decision changed")
    expected_objects = {
        item["dataset_object_id"]: item for item in selection["objects"]
    }
    expected_objects.update(
        {item["dataset_object_id"]: item for item in holdout["objects"]}
    )
    if len(expected_objects) != 6:
        raise DiagnosticError("diagnostic requires exactly six already opened objects")
    return (
        manifest_bytes,
        manifest,
        {
            "fit_decode_report_sha256": sha256_bytes(fit_bytes),
            "selection_report_sha256": sha256_bytes(selection_bytes),
            "holdout_decode_report_sha256": sha256_bytes(holdout_decode_bytes),
            "holdout_report_sha256": sha256_bytes(holdout_bytes),
            "expected_objects": expected_objects,
        },
    )


def frame_starts() -> list[int]:
    return list(range(0, ANALYSIS_STOP - FRAME_SAMPLES + 1, HOP_SAMPLES))


def projected_frames(signal: np.ndarray, low_hz: float, high_hz: float) -> np.ndarray:
    band = execution.band_project(signal[:, :ANALYSIS_STOP], low_hz, high_hz)
    window = np.hanning(FRAME_SAMPLES).astype(np.float64)
    return np.stack(
        [band[:, start : start + FRAME_SAMPLES] * window for start in frame_starts()],
        axis=1,
    )


def frame_rms_by_listener(
    signal: np.ndarray, low_hz: float, high_hz: float
) -> np.ndarray:
    band = execution.band_project(signal[:, :ANALYSIS_STOP], low_hz, high_hz)
    return np.stack(
        [
            np.sqrt(np.mean(np.square(band[:, start : start + FRAME_SAMPLES]), axis=1))
            for start in frame_starts()
        ],
        axis=1,
    )


def upper_triangle(values: np.ndarray) -> np.ndarray:
    return values[np.triu_indices(values.shape[0], 1)]


def spatial_statistics(signal: np.ndarray) -> dict[str, Any]:
    coherences = []
    effective_ranks = []
    near_unity = []
    for low_hz, high_hz in itertools.pairwise(execution.protocol.BAND_EDGES_HZ):
        frames = projected_frames(signal, low_hz, high_hz)
        spectra = np.fft.rfft(frames, axis=2)
        frequencies = np.fft.rfftfreq(
            FRAME_SAMPLES, d=1.0 / execution.protocol.SAMPLE_RATE_HZ
        )
        mask = (frequencies >= low_hz) & (
            frequencies <= high_hz
            if high_hz == execution.protocol.SAMPLE_RATE_HZ / 2
            else frequencies < high_hz
        )
        values = spectra[:, :, mask].reshape(signal.shape[0], -1)
        covariance = values @ values.conj().T
        diagonal = np.maximum(np.real(np.diag(covariance)), 1.0e-300)
        coherence = np.square(np.abs(covariance)) / np.outer(diagonal, diagonal)
        coherence = np.clip(np.real(coherence), 0.0, 1.0)
        pair_values = upper_triangle(coherence)
        coherences.append(pair_values)
        near_unity.append(float(np.mean(pair_values >= 0.98)))

        eigenvalues = np.maximum(np.linalg.eigvalsh(covariance).real, 0.0)
        total = float(np.sum(eigenvalues))
        probabilities = eigenvalues / max(total, 1.0e-300)
        nonzero = probabilities[probabilities > 1.0e-15]
        effective_ranks.append(float(np.exp(-np.sum(nonzero * np.log(nonzero)))))
    return {
        "coherence": np.stack(coherences),
        "median_pair_coherence": float(np.median(np.concatenate(coherences))),
        "near_unity_pair_fraction": float(np.mean(near_unity)),
        "median_effective_rank": float(np.median(effective_ranks)),
    }


def modulation_distribution(signal: np.ndarray) -> np.ndarray:
    rows = []
    frame_rate = execution.protocol.SAMPLE_RATE_HZ / HOP_SAMPLES
    for low_hz, high_hz in itertools.pairwise(execution.protocol.BAND_EDGES_HZ):
        envelope = frame_rms_by_listener(signal, low_hz, high_hz)
        mean = np.maximum(np.mean(envelope, axis=1, keepdims=True), 1.0e-300)
        normalized = envelope / mean - 1.0
        power = np.square(np.abs(np.fft.rfft(normalized, axis=1)))
        frequencies = np.fft.rfftfreq(normalized.shape[1], d=1.0 / frame_rate)
        band_power = []
        for first, second in itertools.pairwise(MODULATION_EDGES_HZ):
            mask = (frequencies > first) & (
                frequencies <= second
                if second == MODULATION_EDGES_HZ[-1]
                else frequencies < second
            )
            band_power.append(np.sum(power[:, mask], axis=1))
        values = np.stack(band_power, axis=1)
        values /= np.maximum(np.sum(values, axis=1, keepdims=True), 1.0e-300)
        rows.append(10.0 * np.log10(np.maximum(values, POWER_FLOOR_RATIO)))
    return np.stack(rows)


def adjacent_band_envelope_correlations(signal: np.ndarray) -> np.ndarray:
    envelopes = [
        frame_rms_by_listener(signal, low_hz, high_hz)
        for low_hz, high_hz in itertools.pairwise(execution.protocol.BAND_EDGES_HZ)
    ]
    result = []
    for first, second in itertools.pairwise(envelopes):
        first_c = first - np.mean(first, axis=1, keepdims=True)
        second_c = second - np.mean(second, axis=1, keepdims=True)
        denominator = np.sqrt(
            np.sum(np.square(first_c), axis=1) * np.sum(np.square(second_c), axis=1)
        )
        result.append(
            np.sum(first_c * second_c, axis=1) / np.maximum(denominator, 1.0e-300)
        )
    return np.stack(result)


def spectral_shape(signal: np.ndarray) -> np.ndarray:
    rows = []
    for low_hz, high_hz in itertools.pairwise(execution.protocol.BAND_EDGES_HZ):
        frames = projected_frames(signal, low_hz, high_hz)
        power = np.mean(np.square(np.abs(np.fft.rfft(frames, axis=2))), axis=(0, 1))
        frequencies = np.fft.rfftfreq(
            FRAME_SAMPLES, d=1.0 / execution.protocol.SAMPLE_RATE_HZ
        )
        mask = (frequencies >= low_hz) & (
            frequencies <= high_hz
            if high_hz == execution.protocol.SAMPLE_RATE_HZ / 2
            else frequencies < high_hz
        )
        selected = power[mask]
        floor = max(float(np.max(selected)) * POWER_FLOOR_RATIO, 1.0e-300)
        decibels = 10.0 * np.log10(np.maximum(selected, floor))
        rows.append(decibels - np.mean(decibels))
    return np.concatenate(rows)


def autocorrelation_profile(signal: np.ndarray) -> np.ndarray:
    rows = []
    for low_hz, high_hz in itertools.pairwise(execution.protocol.BAND_EDGES_HZ):
        band = execution.band_project(signal[:, :ANALYSIS_STOP], low_hz, high_hz)
        energy = np.sum(np.square(band), axis=1)
        for lag in AUTOCORRELATION_LAGS:
            value = np.sum(band[:, :-lag] * band[:, lag:], axis=1)
            rows.append(value / np.maximum(energy, 1.0e-300))
    return np.stack(rows)


def residual_flatness_error(observed: np.ndarray, candidate: np.ndarray) -> float:
    errors = []
    for start, stop in execution.protocol.EVALUATION_WINDOWS:
        errors.extend(
            np.abs(
                execution.spectral_flatness_db(observed[:, start:stop])
                - execution.spectral_flatness_db(candidate[:, start:stop])
            )
        )
    return float(np.mean(errors))


def diagnose_pair(observed: np.ndarray, candidate: np.ndarray) -> dict[str, float]:
    observed_spatial = spatial_statistics(observed)
    candidate_spatial = spatial_statistics(candidate)
    return {
        "spectral_shape_mae_db": float(
            np.mean(np.abs(spectral_shape(observed) - spectral_shape(candidate)))
        ),
        "autocorrelation_mae": float(
            np.mean(
                np.abs(
                    autocorrelation_profile(observed)
                    - autocorrelation_profile(candidate)
                )
            )
        ),
        "modulation_power_mae_db": float(
            np.mean(
                np.abs(
                    modulation_distribution(observed)
                    - modulation_distribution(candidate)
                )
            )
        ),
        "adjacent_band_envelope_correlation_mae": float(
            np.mean(
                np.abs(
                    adjacent_band_envelope_correlations(observed)
                    - adjacent_band_envelope_correlations(candidate)
                )
            )
        ),
        "cross_listener_coherence_mae": float(
            np.mean(
                np.abs(observed_spatial["coherence"] - candidate_spatial["coherence"])
            )
        ),
        "observed_median_pair_coherence": observed_spatial["median_pair_coherence"],
        "candidate_median_pair_coherence": candidate_spatial["median_pair_coherence"],
        "observed_near_unity_pair_fraction": observed_spatial[
            "near_unity_pair_fraction"
        ],
        "candidate_near_unity_pair_fraction": candidate_spatial[
            "near_unity_pair_fraction"
        ],
        "observed_median_effective_rank": observed_spatial["median_effective_rank"],
        "candidate_median_effective_rank": candidate_spatial["median_effective_rank"],
        "residual_flatness_absolute_error_db": residual_flatness_error(
            observed, candidate
        ),
    }


def fixture_report() -> dict[str, Any]:
    length = ANALYSIS_STOP
    time = np.arange(length, dtype=np.float64) / execution.protocol.SAMPLE_RATE_HZ
    base = execution.deterministic_normal("residual-diagnostic-fixture-base", length)
    base /= math.sqrt(float(np.mean(np.square(base))))
    shared = np.linspace(0.7, 1.3, 15)[:, None] * base[None, :]
    independent = np.stack(
        [
            execution.deterministic_normal(
                f"residual-diagnostic-fixture-independent-{index}", length
            )
            for index in range(15)
        ]
    )
    smooth = shared * np.exp(-12.0 * time)[None, :]
    modulated = smooth * (1.0 + 0.65 * np.sin(2.0 * np.pi * 20.0 * time))[None, :]
    shared_spatial = spatial_statistics(shared)
    independent_spatial = spatial_statistics(independent)
    modulation_error = float(
        np.mean(
            np.abs(modulation_distribution(smooth) - modulation_distribution(modulated))
        )
    )
    checks = {
        "shared_noise_near_unity_coherence": shared_spatial["median_pair_coherence"]
        >= 0.98,
        "independent_noise_lower_coherence": independent_spatial[
            "median_pair_coherence"
        ]
        <= 0.20,
        "shared_noise_lower_effective_rank": shared_spatial["median_effective_rank"]
        < independent_spatial["median_effective_rank"],
        "twenty_hz_modulation_is_detected": modulation_error >= 1.5,
    }
    if not all(checks.values()):
        raise DiagnosticError(f"diagnostic fixture failed: {checks}")
    return {
        "shared_signal_sha256": sha256_bytes(shared.astype("<f8").tobytes()),
        "independent_signal_sha256": sha256_bytes(independent.astype("<f8").tobytes()),
        "shared_spatial": {
            key: value for key, value in shared_spatial.items() if key != "coherence"
        },
        "independent_spatial": {
            key: value
            for key, value in independent_spatial.items()
            if key != "coherence"
        },
        "modulation_power_mae_db": modulation_error,
        "checks": checks,
    }


def load_opened_objects(
    root: Path,
    arguments: argparse.Namespace,
    manifest: dict[str, Any],
) -> list[tuple[dict[str, Any], np.ndarray]]:
    base = arguments.manifest.resolve(strict=True).parent
    objects = execution.load_existing_observations(base, manifest)
    _, _, fit = execution.decoded_objects(
        root, arguments.fit_decode, MANIFEST_SHA256, "fit"
    )
    _, _, holdout = execution.decoded_objects(
        root, arguments.holdout_decode, MANIFEST_SHA256, "holdout"
    )
    objects.extend(fit)
    objects.extend(holdout)
    if (
        len(objects) != 6
        or len({item[0]["dataset_object_id"] for item in objects}) != 6
    ):
        raise DiagnosticError("opened object roster changed")
    return objects


def object_diagnostic(
    reference: dict[str, Any],
    raw_channels: np.ndarray,
    expected: dict[str, Any],
) -> dict[str, Any]:
    object_id = reference["dataset_object_id"]
    onset = execution.detect_onset(raw_channels)
    observation = np.asarray(
        raw_channels[:, onset : onset + execution.protocol.ANALYSIS_SAMPLES],
        dtype=np.float64,
    )
    baseline, modal = execution.modal_baseline(observation)
    residual = observation - baseline
    rendered, _ = execution.fit_and_render_residual(
        residual, MANIFEST_SHA256, object_id, CANDIDATE_ID
    )
    identities = {
        "input_slice_sha256": sha256_bytes(observation.astype("<f8").tobytes()),
        "modal_reconstruction_sha256": modal["reconstruction_sha256"],
        "residual_sha256": sha256_bytes(residual.astype("<f8").tobytes()),
        "rendered_residual_sha256": sha256_bytes(rendered.astype("<f8").tobytes()),
    }
    expected_identities = {
        "input_slice_sha256": expected["input_slice_sha256"],
        "modal_reconstruction_sha256": expected["modal"]["reconstruction_sha256"],
        "residual_sha256": expected["residual_sha256"],
        "rendered_residual_sha256": expected["candidates"][CANDIDATE_ID][
            "rendered_residual_sha256"
        ],
    }
    if identities != expected_identities:
        raise DiagnosticError(f"{object_id} recomputation left frozen lineage")
    return {
        "dataset_object_id": object_id,
        "family_group": reference.get("family_group"),
        "role": reference["role"],
        "onset_sample": onset,
        "identities": identities,
        "metrics": diagnose_pair(residual, rendered),
    }


def median_metric(objects: list[dict[str, Any]], name: str) -> float:
    return float(np.median([item["metrics"][name] for item in objects]))


def hypothesis_decision(objects: list[dict[str, Any]]) -> dict[str, Any]:
    aggregates = {name: median_metric(objects, name) for name in objects[0]["metrics"]}
    aggregates["median_effective_rank_gap"] = (
        aggregates["observed_median_effective_rank"]
        - aggregates["candidate_median_effective_rank"]
    )
    checks = {
        "within_band_coloration": {
            "spectral_shape": aggregates["spectral_shape_mae_db"]
            >= HYPOTHESIS_THRESHOLDS["within_band_coloration"][
                "minimum_median_spectral_shape_mae_db"
            ],
            "autocorrelation": aggregates["autocorrelation_mae"]
            >= HYPOTHESIS_THRESHOLDS["within_band_coloration"][
                "minimum_median_autocorrelation_mae"
            ],
        },
        "temporal_modulation": {
            "modulation_power": aggregates["modulation_power_mae_db"]
            >= HYPOTHESIS_THRESHOLDS["temporal_modulation"][
                "minimum_median_modulation_power_mae_db"
            ],
            "adjacent_band_correlation": aggregates[
                "adjacent_band_envelope_correlation_mae"
            ]
            >= HYPOTHESIS_THRESHOLDS["temporal_modulation"][
                "minimum_median_adjacent_band_envelope_correlation_mae"
            ],
        },
        "cross_listener_coupling": {
            "coherence": aggregates["cross_listener_coherence_mae"]
            >= HYPOTHESIS_THRESHOLDS["cross_listener_coupling"][
                "minimum_median_coherence_mae"
            ],
            "effective_rank_gap": aggregates["median_effective_rank_gap"]
            >= HYPOTHESIS_THRESHOLDS["cross_listener_coupling"][
                "minimum_median_effective_rank_gap"
            ],
            "candidate_rank_bound": aggregates["candidate_median_effective_rank"]
            <= HYPOTHESIS_THRESHOLDS["cross_listener_coupling"][
                "maximum_median_candidate_effective_rank"
            ],
        },
    }
    supported = {name: all(values.values()) for name, values in checks.items()}
    leading = next((name for name in HYPOTHESIS_PRIORITY if supported[name]), None)
    return {
        "aggregates": aggregates,
        "thresholds": HYPOTHESIS_THRESHOLDS,
        "checks": checks,
        "supported": supported,
        "priority": HYPOTHESIS_PRIORITY,
        "leading_hypothesis": leading,
    }


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    output = external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    manifest_bytes, manifest, lineage = validate_lineage(root, arguments)
    fixture = fixture_report()
    common = {
        "schema": SCHEMA,
        "status": "Validated",
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "execution_runner_sha256": EXECUTION_RUNNER_SHA256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "lineage": {
            key: value for key, value in lineage.items() if key != "expected_objects"
        },
        "claim": (
            "OPENED_ROW_REPRESENTATION_DIAGNOSTIC_ONLY / NO_RETUNING_CANDIDATE_"
            "SELECTION_SHADOW_QUALITY_DOMAIN_RUNTIME_PHYSICS_OR_PLANTER_CREDIT"
        ),
        "fixtures": fixture,
        "network_requests": 0,
        "new_member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
    }
    if arguments.stage == "preflight":
        report = {
            **common,
            "decision": "RealImpactResidualRepresentationDiagnosticFrozen",
            "objects": sorted(lineage["expected_objects"]),
            "diagnostic_profile": {
                "analysis_stop": ANALYSIS_STOP,
                "frame_samples": FRAME_SAMPLES,
                "hop_samples": HOP_SAMPLES,
                "band_edges_hz": execution.protocol.BAND_EDGES_HZ,
                "modulation_edges_hz": MODULATION_EDGES_HZ,
                "autocorrelation_lags": AUTOCORRELATION_LAGS,
                "hypothesis_thresholds": HYPOTHESIS_THRESHOLDS,
                "hypothesis_priority": HYPOTHESIS_PRIORITY,
            },
            "next_action": "freeze this report, then run diagnose twice without changing the runner",
        }
    else:
        objects = [
            object_diagnostic(
                reference,
                channels,
                lineage["expected_objects"][reference["dataset_object_id"]],
            )
            for reference, channels in load_opened_objects(root, arguments, manifest)
        ]
        decision = hypothesis_decision(objects)
        label = decision["leading_hypothesis"]
        report = {
            **common,
            "decision": (
                "RealImpactResidualRepresentationHypothesisSupported"
                if label is not None
                else "RealImpactResidualRepresentationDiagnosticInconclusive"
            ),
            "objects": objects,
            "diagnostic": decision,
            "next_action": (
                f"preregister a fresh grouped successor for {label}; do not reuse Metal Spoon as holdout"
                if label is not None
                else "stop and research a new measurable representation statistic before any new audio access"
            ),
        }
    report_bytes = canonical_json(report)
    output.write_bytes(report_bytes)
    print(f"REALIMPACT residual representation {arguments.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("new member payload bytes read: 0")
    print("shadow payload bytes read: 0")
    print("quality/domain/runtime admission: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
