#!/usr/bin/env python3
"""Execute the frozen REALIMPACT object-grouped metal representation batch."""

from __future__ import annotations

import argparse
from collections.abc import Iterable
import hashlib
import io
import ipaddress
import itertools
import json
import math
import os
from pathlib import Path
import shutil
import socket
import struct
import tempfile
from typing import Any
import urllib.error
import urllib.parse
import urllib.request
import zlib

import numpy as np
import physical_sound_broadband_common_pole_control as broadband
import physical_sound_dense_broadband_scaling_control as dense
import physical_sound_realimpact_broadband_counterfactual as broadband_v1
import physical_sound_realimpact_metal_batch_discovery as discovery
import physical_sound_realimpact_metal_batch_protocol as protocol
import scipy


MANIFEST_SCHEMA = "nextengine.experimental-realimpact-metal-batch-execution.manifest.v1"
REPORT_SCHEMAS = {
    "preflight": "nextengine.experimental-realimpact-metal-batch-execution-preflight.report.v1",
    "acquire-fit": "nextengine.experimental-realimpact-metal-batch-fit-acquisition.report.v1",
    "decode-fit": "nextengine.experimental-realimpact-metal-batch-fit-decode.report.v1",
    "select": "nextengine.experimental-realimpact-metal-batch-selection.report.v1",
    "acquire-holdout": "nextengine.experimental-realimpact-metal-batch-holdout-acquisition.report.v1",
    "decode-holdout": "nextengine.experimental-realimpact-metal-batch-holdout-decode.report.v1",
    "holdout": "nextengine.experimental-realimpact-metal-batch-holdout.report.v1",
    "acquire-shadow": "nextengine.experimental-realimpact-metal-batch-shadow-acquisition.report.v1",
    "decode-shadow": "nextengine.experimental-realimpact-metal-batch-shadow-decode.report.v1",
    "shadow": "nextengine.experimental-realimpact-metal-batch-shadow.report.v1",
}
STUDY_ID = protocol.STUDY_ID
REVISION = "staged-modal-transient-residual-representation-execution-v1"

PROTOCOL_RUNNER_SHA256 = (
    "c3d0b406c8f96d86508e5db2ee3e5a2c2b37763d7c4190f65780fac00c77e2ea"
)
PROTOCOL_MANIFEST_SHA256 = (
    "2423197420b3c77b67441043323f89a6dc539ff00292007f8255856a08ab724b"
)
PROTOCOL_PREFLIGHT_SHA256 = (
    "bd7557418546495ccae9689e00c17ad59e94348b4aa18a896ab073e1a55709e3"
)

SOURCE_HASHES = {
    "lab/scripts/physical_sound_realimpact_metal_batch_protocol.py": PROTOCOL_RUNNER_SHA256,
    "lab/scripts/physical_sound_realimpact_metal_batch_discovery.py": "ecbaa72478c6cc4475fc204d699e334059707f0474cac284cc09e02f2ccacbfc",
    "lab/scripts/physical_sound_broadband_common_pole_control.py": "317fa3a2e4f05f6c212ccdbee0575554d3610753573db359a68f3e756a73ec8d",
    "lab/scripts/physical_sound_dense_broadband_scaling_control.py": "8e4cc18ba25f90adb43483e5d4ff1ea7be46869e55c28ebd7c9bb5831b29fbbe",
    "lab/scripts/physical_sound_realimpact_broadband_counterfactual.py": "63d46bf476798e540cfc726a0e2bca58beaacc0f829ffff7ecc2b1083efb2e76",
    "lab/scripts/physical_sound_realimpact_broadband_v2_counterfactual.py": "3f1c07b501dec97645cc0f1ad2fec5b3960312eacde88a291b64cc740a056aa5",
}

ROLE_STAGE_OBJECTS = {
    "fit": ["86_MetalHoledSpoon"],
    "holdout": ["91_MetalSpoon"],
    "shadow": ["89_MetalSpatula", "92_MetalSpatula"],
}
SELECTABLE_CANDIDATES = [
    "modal_plus_parametric_transient_v0",
    "modal_plus_seeded_subband_residual_v0",
]

FRAME_SAMPLES = 512
HOP_SAMPLES = 256
ONSET_PEAK_FRACTION = 0.05
MAXIMUM_REGION_COUNT = 32
MAXIMUM_ANALYSIS_BIN_COUNT = 96
MINIMUM_PREPRUNE_CLUSTER_COUNT = 6
MAXIMUM_PREPRUNE_CLUSTER_COUNT = 64
MINIMUM_RETAINED_MODE_COUNT = 6
MAXIMUM_RETAINED_MODE_COUNT = 64
METRIC_DENOMINATOR_FLOOR = 1.0e-12
POWER_FLOOR_RATIO = 1.0e-8


class ExecutionError(RuntimeError):
    """The execution lineage, leakage order, or bounded DSP invariant failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--stage", required=True, choices=["manifest", *sorted(REPORT_SCHEMAS)]
    )
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--input", type=Path)
    parser.add_argument("--selection", type=Path)
    parser.add_argument("--holdout", type=Path)
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
        raise ExecutionError(f"{label} must be an external regular file: {resolved}")
    return resolved


def external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise ExecutionError(f"{label} must be an external directory: {resolved}")
    return resolved


def external_output(root: Path, path: Path, *, directory: bool) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise ExecutionError(f"output must remain outside the repository: {resolved}")
    if directory:
        if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
            raise ExecutionError(f"output must be absent or empty: {resolved}")
    elif resolved.exists():
        raise ExecutionError(f"refusing to replace existing output: {resolved}")
    return resolved


def protocol_references() -> dict[str, dict[str, Any]]:
    return {
        "runner": {
            "path": "lab/scripts/physical_sound_realimpact_metal_batch_protocol.py",
            "sha256": PROTOCOL_RUNNER_SHA256,
        },
        "manifest": {
            "path": "../ps2-realimpact-metal-batch-protocol-v1/manifest.json",
            "sha256": PROTOCOL_MANIFEST_SHA256,
            "bytes": 13_558,
        },
        "preflight": {
            "path": "../ps2-realimpact-metal-batch-protocol-v1/preflight-a.json",
            "sha256": PROTOCOL_PREFLIGHT_SHA256,
            "bytes": 6_279,
            "decision": "RealImpactMetalBatchPayloadAndCandidateProtocolFrozen",
        },
    }


def dsp_profile() -> dict[str, Any]:
    return {
        "sample_rate_hz": protocol.SAMPLE_RATE_HZ,
        "analysis_samples": protocol.ANALYSIS_SAMPLES,
        "onset": {
            "rule": "first spatial-l2 sample at or above 5 percent of maximum",
            "peak_fraction": ONSET_PEAK_FRACTION,
        },
        "modal_baseline": {
            "gabor_source": "physical_sound_broadband_common_pole_control.py",
            "partial_svd_source": "physical_sound_dense_broadband_scaling_control.py",
            "amplitude_prune_source": "physical_sound_realimpact_broadband_counterfactual.py",
            "partial_svd_rank": dense.PARTIAL_SVD_RANK,
            "opened_region_ranking": "forbidden",
            "maximum_region_count": MAXIMUM_REGION_COUNT,
            "maximum_analysis_bin_count": MAXIMUM_ANALYSIS_BIN_COUNT,
            "preprune_cluster_count": [
                MINIMUM_PREPRUNE_CLUSTER_COUNT,
                MAXIMUM_PREPRUNE_CLUSTER_COUNT,
            ],
            "retained_mode_count": [
                MINIMUM_RETAINED_MODE_COUNT,
                MAXIMUM_RETAINED_MODE_COUNT,
            ],
        },
        "band_projection": {
            "kind": "numpy_rfft_rectangular_bins_v1",
            "band_edges_hz": protocol.BAND_EDGES_HZ,
            "lower_edge_inclusive": True,
            "upper_edge_exclusive_except_nyquist": True,
            "inverse": "numpy_irfft_same_length",
        },
        "frames": {
            "samples": FRAME_SAMPLES,
            "hop_samples": HOP_SAMPLES,
            "timestamp": "frame_center_seconds",
            "partial_frame": "excluded",
        },
        "envelope_fit": {
            "target": "pooled_listener_band_rms_per_frame",
            "decay_grid_per_second": protocol.DECAY_GRID_PER_SECOND,
            "one_exponential": "nonnegative scalar least squares",
            "two_exponential": (
                "all decay pairs with replacement; exact two-column nonnegative "
                "least squares boundary enumeration; minimum (sse,d1,d2)"
            ),
            "listener_gain": (
                "listener band RMS divided by pooled listener band RMS over fit window"
            ),
        },
        "excitation": {
            "uniform": (
                "SHA256(seed_utf8 || u64be(counter)); four open-interval u64 "
                "uniforms per digest"
            ),
            "normal": "Box-Muller pairs in digest order",
            "band_limit": "the frozen rectangular RFFT projection",
            "normalization": "unit RMS after band projection",
            "seed_fields": ["manifest_sha256", "dataset_object_id", "band_index"],
        },
        "metrics": {
            "log_band_envelope_mae_db": (
                "mean absolute 20log10 frame-RMS error over complete frames whose "
                "centres are inside both frozen evaluation windows; per-band floor "
                "is observed peak RMS times 10^(-80/20)"
            ),
            "listener_window_energy_mae_db": (
                "mean absolute full-band listener RMS dB error over both frozen windows"
            ),
            "spectral_flatness_absolute_error_db": (
                "mean absolute 10log10 geometric/arithmetic power-ratio error over "
                "listeners and frozen windows; power floor is window maximum times 1e-8"
            ),
            "ratio_denominator_floor": METRIC_DENOMINATOR_FLOOR,
            "diagnostic_waveform_nrmse": True,
        },
    }


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner": {
            "path": "lab/scripts/physical_sound_realimpact_metal_batch_execute.py",
            "sha256": runner_sha256,
        },
        "protocol": protocol_references(),
        "sources": [
            {"path": path, "sha256": digest} for path, digest in SOURCE_HASHES.items()
        ],
        "existing_observations": discovery.EXISTING_OBSERVATIONS,
        "partition": protocol.PARTITION,
        "new_objects": protocol.OBJECTS,
        "stage_plan": [
            {
                "stage": "acquire-fit",
                "objects": ROLE_STAGE_OBJECTS["fit"],
                "requires": ["preflight"],
            },
            {
                "stage": "decode-fit",
                "objects": ROLE_STAGE_OBJECTS["fit"],
                "requires": ["acquire-fit"],
            },
            {
                "stage": "select",
                "roles": ["development", "calibration"],
                "requires": ["decode-fit"],
            },
            {
                "stage": "acquire-holdout",
                "objects": ROLE_STAGE_OBJECTS["holdout"],
                "requires": ["frozen successful select report"],
            },
            {
                "stage": "decode-holdout",
                "objects": ROLE_STAGE_OBJECTS["holdout"],
                "requires": ["acquire-holdout"],
            },
            {
                "stage": "holdout",
                "roles": ["holdout"],
                "requires": ["decode-holdout", "frozen successful select report"],
            },
            {
                "stage": "acquire-shadow",
                "objects": ROLE_STAGE_OBJECTS["shadow"],
                "requires": ["frozen successful holdout report"],
            },
            {
                "stage": "decode-shadow",
                "objects": ROLE_STAGE_OBJECTS["shadow"],
                "requires": ["acquire-shadow"],
            },
            {
                "stage": "shadow",
                "roles": ["shadow"],
                "requires": [
                    "decode-shadow",
                    "frozen successful select report",
                    "frozen successful holdout report",
                ],
            },
        ],
        "dsp": dsp_profile(),
        "candidates": protocol.CANDIDATES,
        "metrics": protocol.METRICS,
        "selection_gates": protocol.SELECTION_GATES,
        "runtime": {"numpy": np.__version__, "scipy": scipy.__version__},
        "access": protocol.access_summary(),
        "data_policy": {
            "new_payload_only_in_declared_role_stage": True,
            "holdout_before_selection": False,
            "shadow_before_frozen_successful_holdout": False,
            "retry_prefix_growth_or_object_substitution": False,
            "per_object_human_admission": False,
            "planter_payload_allowed": False,
            "physics_solver_allowed": False,
            "quality_domain_or_runtime_admission_allowed": False,
            "authored_clip_fallback_required": True,
        },
        "stop_rule": (
            "publish one immutable report per stage; any lineage, request, decode, "
            "modal-capacity, repeat, calibration, holdout or shadow failure stops "
            "later access without retry, prefix growth, threshold tuning or object "
            "substitution and retains authored_clip_required"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 256 * 1024:
        raise ExecutionError("metal batch execution manifest exceeds 256 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise ExecutionError(
            f"parse metal batch execution manifest: {error}"
        ) from error
    if manifest != expected_manifest(runner_sha256):
        raise ExecutionError("metal batch execution manifest changed")
    return data, manifest


def resolve_store_reference(
    base: Path, reference: dict[str, Any], label: str
) -> tuple[Path, bytes]:
    path = (base / reference["path"]).resolve(strict=True)
    store = base.parent.resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(store):
        raise ExecutionError(f"{label} escapes the external physical-sound store")
    data = path.read_bytes()
    if sha256_bytes(data) != reference["sha256"] or (
        "bytes" in reference and len(data) != reference["bytes"]
    ):
        raise ExecutionError(f"{label} identity changed")
    return path, data


def validate_protocol(
    root: Path, base: Path, manifest: dict[str, Any]
) -> dict[str, Any]:
    refs = manifest["protocol"]
    runner = (root / refs["runner"]["path"]).resolve(strict=True)
    if (
        not runner.is_file()
        or not runner.is_relative_to(root)
        or sha256_file(runner) != refs["runner"]["sha256"]
    ):
        raise ExecutionError("bound metal batch protocol runner changed")
    manifest_path, protocol_manifest_bytes = resolve_store_reference(
        base, refs["manifest"], "protocol manifest"
    )
    _, preflight_bytes = resolve_store_reference(
        base, refs["preflight"], "protocol preflight"
    )
    protocol_manifest = json.loads(protocol_manifest_bytes)
    protocol_preflight = json.loads(preflight_bytes)
    if (
        protocol_manifest != protocol.expected_manifest(PROTOCOL_RUNNER_SHA256)
        or protocol_preflight.get("decision") != refs["preflight"]["decision"]
        or protocol_preflight.get("manifest_sha256") != PROTOCOL_MANIFEST_SHA256
        or protocol_preflight.get("runner_sha256") != PROTOCOL_RUNNER_SHA256
        or protocol_preflight.get("network_requests") != 0
        or protocol_preflight.get("new_member_payload_bytes_read") != 0
        or protocol_preflight.get("quality_domain_or_runtime_admission") is not False
    ):
        raise ExecutionError("metal batch protocol lineage changed")
    return {
        "runner_sha256": sha256_file(runner),
        "manifest_path": str(manifest_path),
        "manifest_sha256": sha256_bytes(protocol_manifest_bytes),
        "preflight_sha256": sha256_bytes(preflight_bytes),
        "decision": protocol_preflight["decision"],
    }


def validate_sources(root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    result = []
    for reference in manifest["sources"]:
        path = (root / reference["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(root)
            or sha256_file(path) != reference["sha256"]
        ):
            raise ExecutionError(f"bound source changed: {reference['path']}")
        result.append(
            {
                "path": reference["path"],
                "sha256": reference["sha256"],
                "bytes": path.stat().st_size,
            }
        )
    return result


def validate_existing_observations(
    base: Path, manifest: dict[str, Any]
) -> list[dict[str, Any]]:
    result = []
    for reference in manifest["existing_observations"]:
        path = (base / reference["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(base.parent)
            or path.stat().st_size != reference["bytes"]
            or sha256_file(path) != reference["sha256"]
        ):
            raise ExecutionError(
                f"existing observation changed: {reference['dataset_object_id']}"
            )
        result.append(
            {
                "dataset_object_id": reference["dataset_object_id"],
                "family_group": reference["family_group"],
                "role": reference["role"],
                "path": reference["path"],
                "sha256": reference["sha256"],
                "bytes": path.stat().st_size,
            }
        )
    return result


def common_report(manifest_bytes: bytes, runner_sha256: str) -> dict[str, Any]:
    return {
        "study_id": STUDY_ID,
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "planter_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
    }


def write_new_file(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        raise ExecutionError(f"refusing to replace existing output: {path}")
    descriptor, staging_name = tempfile.mkstemp(
        prefix=f".{path.name}.", dir=path.parent
    )
    staging = Path(staging_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(staging, path)
    finally:
        staging.unlink(missing_ok=True)


def publish_directory(
    output: Path, report: dict[str, Any], files: dict[str, bytes] | None = None
) -> bytes:
    report_bytes = canonical_json(report)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent)
    )
    try:
        (staging / "report.json").write_bytes(report_bytes)
        for name, data in (files or {}).items():
            target = staging / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return report_bytes


def deterministic_normal(seed: str, count: int) -> np.ndarray:
    if count < 0:
        raise ExecutionError("normal sample count is negative")
    result = np.empty(count, dtype=np.float64)
    written = 0
    counter = 0
    seed_bytes = seed.encode("utf-8")
    denominator = float(1 << 64)
    while written < count:
        digest = hashlib.sha256(seed_bytes + counter.to_bytes(8, "big")).digest()
        values = [
            (int.from_bytes(digest[index : index + 8], "big") + 0.5) / denominator
            for index in range(0, 32, 8)
        ]
        normals = []
        for first, second in ((values[0], values[1]), (values[2], values[3])):
            radius = math.sqrt(-2.0 * math.log(first))
            angle = 2.0 * math.pi * second
            normals.extend([radius * math.cos(angle), radius * math.sin(angle)])
        take = min(len(normals), count - written)
        result[written : written + take] = normals[:take]
        written += take
        counter += 1
    return result


def frequency_mask(length: int, low_hz: float, high_hz: float) -> np.ndarray:
    if length <= 0 or not 0 <= low_hz < high_hz <= protocol.SAMPLE_RATE_HZ / 2:
        raise ExecutionError("invalid frozen frequency band")
    frequencies = np.fft.rfftfreq(length, d=1.0 / protocol.SAMPLE_RATE_HZ)
    if high_hz == protocol.SAMPLE_RATE_HZ / 2:
        return (frequencies >= low_hz) & (frequencies <= high_hz)
    return (frequencies >= low_hz) & (frequencies < high_hz)


def band_project(signal: np.ndarray, low_hz: float, high_hz: float) -> np.ndarray:
    if signal.ndim not in (1, 2) or signal.shape[-1] <= 0:
        raise ExecutionError("band projection expects one or two dimensional signal")
    spectrum = np.fft.rfft(signal, axis=-1)
    mask = frequency_mask(signal.shape[-1], low_hz, high_hz)
    spectrum[..., ~mask] = 0.0
    return np.fft.irfft(spectrum, n=signal.shape[-1], axis=-1)


def unit_band_noise(seed: str, count: int, low_hz: float, high_hz: float) -> np.ndarray:
    projected = band_project(deterministic_normal(seed, count), low_hz, high_hz)
    rms = float(np.sqrt(np.mean(np.square(projected))))
    if not math.isfinite(rms) or rms <= 1.0e-15:
        raise ExecutionError("deterministic band excitation has zero or nonfinite RMS")
    return projected / rms


def frame_rms(signal: np.ndarray, stop: int) -> tuple[np.ndarray, np.ndarray]:
    if signal.ndim != 2 or stop < FRAME_SAMPLES or stop > signal.shape[1]:
        raise ExecutionError("invalid frame-RMS input")
    starts = np.arange(0, stop - FRAME_SAMPLES + 1, HOP_SAMPLES, dtype=np.int64)
    values = np.empty(len(starts), dtype=np.float64)
    for index, start in enumerate(starts):
        frame = signal[:, start : start + FRAME_SAMPLES]
        values[index] = math.sqrt(float(np.mean(np.square(frame))))
    centres = (
        starts.astype(np.float64) + FRAME_SAMPLES / 2.0
    ) / protocol.SAMPLE_RATE_HZ
    return centres, values


def nonnegative_one_column(
    design: np.ndarray, target: np.ndarray
) -> tuple[float, float]:
    denominator = float(np.dot(design, design))
    coefficient = max(float(np.dot(design, target)) / max(denominator, 1.0e-300), 0.0)
    error = target - coefficient * design
    return coefficient, float(np.dot(error, error))


def nonnegative_two_columns(
    first: np.ndarray, second: np.ndarray, target: np.ndarray
) -> tuple[list[float], float]:
    design = np.column_stack([first, second])
    candidates: list[tuple[float, float, float]] = []
    solution, _, _, _ = np.linalg.lstsq(design, target, rcond=None)
    if solution[0] >= 0.0 and solution[1] >= 0.0:
        residual = target - design @ solution
        candidates.append(
            (float(np.dot(residual, residual)), float(solution[0]), float(solution[1]))
        )
    first_coefficient, first_sse = nonnegative_one_column(first, target)
    second_coefficient, second_sse = nonnegative_one_column(second, target)
    candidates.extend(
        [
            (first_sse, first_coefficient, 0.0),
            (second_sse, 0.0, second_coefficient),
            (float(np.dot(target, target)), 0.0, 0.0),
        ]
    )
    sse, coefficient_a, coefficient_b = min(candidates)
    return [coefficient_a, coefficient_b], sse


def fit_envelope(
    band_signal: np.ndarray, fit_stop: int, term_count: int
) -> dict[str, Any]:
    times, target = frame_rms(band_signal, fit_stop)
    choices: list[tuple[Any, ...]] = []
    if term_count == 1:
        for decay in protocol.DECAY_GRID_PER_SECOND:
            basis = np.exp(-float(decay) * times)
            coefficient, sse = nonnegative_one_column(basis, target)
            choices.append((sse, decay, [coefficient]))
        sse, decay, coefficients = min(choices)
        decays = [decay]
    elif term_count == 2:
        for first_decay, second_decay in itertools.combinations_with_replacement(
            protocol.DECAY_GRID_PER_SECOND, 2
        ):
            first = np.exp(-float(first_decay) * times)
            second = np.exp(-float(second_decay) * times)
            coefficients, sse = nonnegative_two_columns(first, second, target)
            choices.append((sse, first_decay, second_decay, coefficients))
        sse, first_decay, second_decay, coefficients = min(choices)
        decays = [first_decay, second_decay]
    else:
        raise ExecutionError("frozen envelope supports one or two terms")
    pooled_rms = math.sqrt(float(np.mean(np.square(band_signal[:, :fit_stop]))))
    if pooled_rms <= 1.0e-15:
        listener_gains = [0.0 for _ in range(band_signal.shape[0])]
    else:
        listener_gains = [
            math.sqrt(float(np.mean(np.square(row[:fit_stop])))) / pooled_rms
            for row in band_signal
        ]
    return {
        "decays_per_second": decays,
        "coefficients": coefficients,
        "sse": sse,
        "listener_gains": listener_gains,
        "frame_count": len(times),
    }


def fit_and_render_residual(
    residual: np.ndarray,
    manifest_sha256: str,
    object_id: str,
    candidate_id: str,
) -> tuple[np.ndarray, list[dict[str, Any]]]:
    if residual.shape != (len(protocol.ANALYSIS_ROWS), protocol.ANALYSIS_SAMPLES):
        raise ExecutionError("residual shape left the frozen 15x60000 envelope")
    if candidate_id == "modal_plus_parametric_transient_v0":
        fit_stop = 4_096
        render_stop = 8_192
        term_count = 1
    elif candidate_id == "modal_plus_seeded_subband_residual_v0":
        fit_stop = protocol.CALIBRATION_STOP_SAMPLE
        render_stop = 32_768
        term_count = 2
    else:
        raise ExecutionError(f"unsupported candidate: {candidate_id}")
    output = np.zeros_like(residual)
    fits = []
    for band_index, (low_hz, high_hz) in enumerate(
        itertools.pairwise(protocol.BAND_EDGES_HZ)
    ):
        band_signal = band_project(residual[:, :fit_stop], low_hz, high_hz)
        fit = fit_envelope(band_signal, fit_stop, term_count)
        seed = f"{manifest_sha256}\0{object_id}\0{band_index}"
        excitation = unit_band_noise(seed, render_stop, low_hz, high_hz)
        times = np.arange(render_stop, dtype=np.float64) / protocol.SAMPLE_RATE_HZ
        envelope = np.zeros(render_stop, dtype=np.float64)
        for coefficient, decay in zip(
            fit["coefficients"], fit["decays_per_second"], strict=True
        ):
            envelope += coefficient * np.exp(-float(decay) * times)
        for listener, gain in enumerate(fit["listener_gains"]):
            output[listener, :render_stop] += gain * envelope * excitation
        fits.append(
            {
                "band_index": band_index,
                "band_hz": [low_hz, high_hz],
                "seed_sha256": sha256_bytes(seed.encode()),
                "excitation_sha256": sha256_bytes(excitation.astype("<f8").tobytes()),
                **fit,
            }
        )
    return output, fits


def complete_frame_starts(window_start: int, window_stop: int) -> Iterable[int]:
    first = ((window_start + HOP_SAMPLES - 1) // HOP_SAMPLES) * HOP_SAMPLES
    return range(first, window_stop - FRAME_SAMPLES + 1, HOP_SAMPLES)


def log_band_envelope_mae_db(observed: np.ndarray, candidate: np.ndarray) -> float:
    errors = []
    for low_hz, high_hz in itertools.pairwise(protocol.BAND_EDGES_HZ):
        observed_band = band_project(observed[:, :32_768], low_hz, high_hz)
        candidate_band = band_project(candidate[:, :32_768], low_hz, high_hz)
        observed_frame_rms = []
        candidate_frame_rms = []
        for window_start, window_stop in protocol.EVALUATION_WINDOWS:
            for start in complete_frame_starts(window_start, window_stop):
                observed_frame_rms.extend(
                    np.sqrt(
                        np.mean(
                            np.square(observed_band[:, start : start + FRAME_SAMPLES]),
                            axis=1,
                        )
                    )
                )
                candidate_frame_rms.extend(
                    np.sqrt(
                        np.mean(
                            np.square(candidate_band[:, start : start + FRAME_SAMPLES]),
                            axis=1,
                        )
                    )
                )
        observed_values = np.asarray(observed_frame_rms, dtype=np.float64)
        candidate_values = np.asarray(candidate_frame_rms, dtype=np.float64)
        floor = max(float(np.max(observed_values)) * 1.0e-4, 1.0e-300)
        errors.extend(
            np.abs(
                20.0 * np.log10(np.maximum(observed_values, floor))
                - 20.0 * np.log10(np.maximum(candidate_values, floor))
            )
        )
    return float(np.mean(errors))


def listener_window_energy_mae_db(observed: np.ndarray, candidate: np.ndarray) -> float:
    errors = []
    for start, stop in protocol.EVALUATION_WINDOWS:
        observed_rms = np.sqrt(np.mean(np.square(observed[:, start:stop]), axis=1))
        candidate_rms = np.sqrt(np.mean(np.square(candidate[:, start:stop]), axis=1))
        floor = max(float(np.max(observed_rms)) * 1.0e-4, 1.0e-300)
        errors.extend(
            np.abs(
                20.0 * np.log10(np.maximum(observed_rms, floor))
                - 20.0 * np.log10(np.maximum(candidate_rms, floor))
            )
        )
    return float(np.mean(errors))


def spectral_flatness_db(signal: np.ndarray) -> np.ndarray:
    spectrum = np.fft.rfft(signal, axis=1)[:, 1:]
    power = np.square(np.abs(spectrum))
    floor = np.maximum(
        np.max(power, axis=1, keepdims=True) * POWER_FLOOR_RATIO, 1.0e-300
    )
    bounded = np.maximum(power, floor)
    geometric = np.exp(np.mean(np.log(bounded), axis=1))
    arithmetic = np.mean(bounded, axis=1)
    return 10.0 * np.log10(geometric / arithmetic)


def spectral_flatness_absolute_error_db(
    observed: np.ndarray, candidate: np.ndarray
) -> float:
    errors = []
    for start, stop in protocol.EVALUATION_WINDOWS:
        errors.extend(
            np.abs(
                spectral_flatness_db(observed[:, start:stop])
                - spectral_flatness_db(candidate[:, start:stop])
            )
        )
    return float(np.mean(errors))


def normalized_rms_error(observed: np.ndarray, candidate: np.ndarray) -> float:
    denominator = math.sqrt(float(np.mean(np.square(observed))))
    numerator = math.sqrt(float(np.mean(np.square(observed - candidate))))
    return numerator / max(denominator, 1.0e-300)


def metrics(observed: np.ndarray, candidate: np.ndarray) -> dict[str, float]:
    return {
        "log_band_envelope_mae_db": log_band_envelope_mae_db(observed, candidate),
        "listener_window_energy_mae_db": listener_window_energy_mae_db(
            observed, candidate
        ),
        "spectral_flatness_absolute_error_db": spectral_flatness_absolute_error_db(
            observed, candidate
        ),
        "waveform_nrmse": normalized_rms_error(observed, candidate),
    }


def metric_ratios(
    baseline: dict[str, float], candidate: dict[str, float]
) -> dict[str, float]:
    return {
        name: candidate[name] / max(baseline[name], METRIC_DENOMINATOR_FLOOR)
        for name in protocol.METRICS["gating"]
    }


def median(values: Iterable[float]) -> float:
    return float(np.median(np.asarray(list(values), dtype=np.float64)))


def calibration_decision(objects: list[dict[str, Any]]) -> dict[str, Any]:
    candidates = []
    thresholds = protocol.SELECTION_GATES["calibration"]
    for candidate_id in SELECTABLE_CANDIDATES:
        rows = [
            item["candidates"][candidate_id]["ratios"]
            for item in objects
            if item["role"] == "calibration"
        ]
        if len(rows) != 2:
            raise ExecutionError("calibration decision requires exactly two objects")
        aggregates = {
            name: median(row[name] for row in rows)
            for name in protocol.METRICS["gating"]
        }
        any_ratio = max(value for row in rows for value in row.values())
        improved_fraction = sum(median(row.values()) < 1.0 for row in rows) / len(rows)
        checks = [
            aggregates["log_band_envelope_mae_db"]
            <= thresholds["maximum_median_log_band_envelope_error_ratio"],
            aggregates["listener_window_energy_mae_db"]
            <= thresholds["maximum_median_listener_energy_error_ratio"],
            aggregates["spectral_flatness_absolute_error_db"]
            <= thresholds["maximum_median_flatness_error_ratio"],
            any_ratio <= thresholds["maximum_any_object_metric_ratio"],
            improved_fraction >= thresholds["minimum_improved_object_fraction"],
        ]
        candidates.append(
            {
                "candidate_id": candidate_id,
                "aggregate_ratios": aggregates,
                "maximum_object_metric_ratio": any_ratio,
                "improved_object_fraction": improved_fraction,
                "passed": all(checks),
            }
        )
    passing = [item for item in candidates if item["passed"]]
    selected = min(
        passing,
        key=lambda item: (
            item["aggregate_ratios"]["log_band_envelope_mae_db"],
            item["aggregate_ratios"]["listener_window_energy_mae_db"],
            item["aggregate_ratios"]["spectral_flatness_absolute_error_db"],
            item["candidate_id"],
        ),
        default=None,
    )
    return {
        "candidates": candidates,
        "selected_candidate_id": (
            selected["candidate_id"] if selected is not None else None
        ),
        "passed": selected is not None,
    }


def detect_onset(channels: np.ndarray) -> int:
    if channels.ndim != 2 or channels.shape[0] != len(protocol.ANALYSIS_ROWS):
        raise ExecutionError("onset detector expects the frozen listener rows")
    spatial_norm = np.linalg.norm(channels, axis=0)
    peak = float(np.max(spatial_norm))
    if not math.isfinite(peak) or peak <= 0.0:
        raise ExecutionError("observation onset is absent")
    candidates = np.flatnonzero(spatial_norm >= ONSET_PEAK_FRACTION * peak)
    if len(candidates) == 0:
        raise ExecutionError("observation onset threshold has no crossing")
    onset = int(candidates[0])
    if onset + protocol.ANALYSIS_SAMPLES > channels.shape[1]:
        raise ExecutionError("observation onset leaves fewer than 60000 samples")
    return onset


def modal_baseline(observation: np.ndarray) -> tuple[np.ndarray, dict[str, Any]]:
    gabor_stop = (
        broadband.WINDOW_SAMPLES
        + (broadband.GABOR_FRAME_COUNT - 1) * broadband.HOP_SAMPLES
    )
    coefficients = broadband.gabor_coefficients(observation[:, :gabor_stop])
    discovered = broadband.discover_regions(coefficients)
    if (
        not 1 <= len(discovered["regions"]) <= MAXIMUM_REGION_COUNT
        or len(discovered["analysis_bins"]) > MAXIMUM_ANALYSIS_BIN_COUNT
    ):
        raise ExecutionError(
            "modal discovery left the frozen 1..32 region / 96-bin envelope"
        )
    raw, bin_analyses = dense.extract_partial_estimates(coefficients, discovered)
    clusters = broadband.cluster_duplicates(raw)
    if (
        not MINIMUM_PREPRUNE_CLUSTER_COUNT
        <= len(clusters)
        <= MAXIMUM_PREPRUNE_CLUSTER_COUNT
    ):
        raise ExecutionError("modal clusters left the frozen 6..64 envelope")
    fitted, reconstruction, undamped = broadband_v1.fit_and_prune(observation, clusters)
    retained = [item for item in fitted if item["retained"]]
    if not MINIMUM_RETAINED_MODE_COUNT <= len(retained) <= MAXIMUM_RETAINED_MODE_COUNT:
        raise ExecutionError("retained modes left the frozen 6..64 envelope")
    return reconstruction, {
        "gabor_coefficients_sha256": sha256_bytes(
            coefficients.astype("<c16", copy=False).tobytes()
        ),
        "regions": discovered["regions"],
        "analysis_bins": discovered["analysis_bins"],
        "all_discovered_regions_analyzed": len(bin_analyses)
        == len(discovered["analysis_bins"]),
        "raw_estimate_count": len(raw),
        "duplicate_estimates_removed": len(raw) - len(clusters),
        "preprune_cluster_count": len(clusters),
        "retained_mode_count": len(retained),
        "pruned_mode_count": len(fitted) - len(retained),
        "reconstruction_sha256": sha256_bytes(
            reconstruction.astype("<f8", copy=False).tobytes()
        ),
        "undamped_reconstruction_sha256": sha256_bytes(
            undamped.astype("<f8", copy=False).tobytes()
        ),
        "damped_nrmse": normalized_rms_error(observation, reconstruction),
        "undamped_nrmse": normalized_rms_error(observation, undamped),
    }


def analyze_observation(
    raw_channels: np.ndarray,
    manifest_sha256: str,
    object_id: str,
    role: str,
    candidate_ids: list[str],
) -> dict[str, Any]:
    if raw_channels.ndim != 2 or raw_channels.shape[0] != 15:
        raise ExecutionError(f"{object_id} observation has an invalid shape")
    if not np.all(np.isfinite(raw_channels)):
        raise ExecutionError(f"{object_id} observation contains nonfinite values")
    onset = detect_onset(raw_channels)
    observation = np.asarray(
        raw_channels[:, onset : onset + protocol.ANALYSIS_SAMPLES], dtype=np.float64
    )
    baseline, modal = modal_baseline(observation)
    baseline_metrics = metrics(observation, baseline)
    residual = observation - baseline
    candidates = {}
    for candidate_id in candidate_ids:
        rendered_residual, fits = fit_and_render_residual(
            residual, manifest_sha256, object_id, candidate_id
        )
        candidate = baseline + rendered_residual
        candidate_metrics = metrics(observation, candidate)
        candidates[candidate_id] = {
            "metrics": candidate_metrics,
            "ratios": metric_ratios(baseline_metrics, candidate_metrics),
            "rendered_residual_sha256": sha256_bytes(
                rendered_residual.astype("<f8", copy=False).tobytes()
            ),
            "candidate_sha256": sha256_bytes(
                candidate.astype("<f8", copy=False).tobytes()
            ),
            "band_fits": fits,
        }
    return {
        "dataset_object_id": object_id,
        "role": role,
        "onset_sample": onset,
        "input_slice_sha256": sha256_bytes(
            observation.astype("<f8", copy=False).tobytes()
        ),
        "modal": modal,
        "baseline_metrics": baseline_metrics,
        "residual_sha256": sha256_bytes(residual.astype("<f8").tobytes()),
        "candidates": candidates,
    }


def fixture_report(manifest_sha256: str) -> dict[str, Any]:
    length = protocol.ANALYSIS_SAMPLES
    times = np.arange(length, dtype=np.float64) / protocol.SAMPLE_RATE_HZ
    listener_gains = np.linspace(0.75, 1.25, 15, dtype=np.float64)
    baseline = listener_gains[:, None] * (
        0.008 * np.exp(-35.0 * times) * np.sin(2.0 * np.pi * 1_125.0 * times)
        + 0.005 * np.exp(-80.0 * times) * np.sin(2.0 * np.pi * 4_250.0 * times)
    )
    true_residual = np.zeros_like(baseline)
    fixture_fits = []
    for band_index, (low_hz, high_hz) in enumerate(
        itertools.pairwise(protocol.BAND_EDGES_HZ)
    ):
        seed = f"{manifest_sha256}\0fixture_calibration_a\0{band_index}"
        excitation = unit_band_noise(seed, 32_768, low_hz, high_hz)
        amplitude = 0.04 / (band_index + 1)
        envelope = amplitude * (
            0.72 * np.exp(-10.0 * times[:32_768])
            + 0.28 * np.exp(-40.0 * times[:32_768])
        )
        true_residual[:, :32_768] += (
            listener_gains[:, None] * envelope[None, :] * excitation[None, :]
        )
        fitted = fit_envelope(
            band_project(
                true_residual[:, : protocol.CALIBRATION_STOP_SAMPLE], low_hz, high_hz
            ),
            protocol.CALIBRATION_STOP_SAMPLE,
            2,
        )
        fixture_fits.append(
            {
                "band_index": band_index,
                "decays_per_second": fitted["decays_per_second"],
                "coefficients": fitted["coefficients"],
            }
        )
    observed = baseline + true_residual
    baseline_metrics = metrics(observed, baseline)
    candidates = {}
    for candidate_id in SELECTABLE_CANDIDATES:
        residual, fits = fit_and_render_residual(
            true_residual,
            manifest_sha256,
            "fixture_calibration_a",
            candidate_id,
        )
        candidate_metrics = metrics(observed, baseline + residual)
        candidates[candidate_id] = {
            "metrics": candidate_metrics,
            "ratios": metric_ratios(baseline_metrics, candidate_metrics),
            "rendered_sha256": sha256_bytes(residual.astype("<f8").tobytes()),
            "fit_decays": [item["decays_per_second"] for item in fits],
        }
    calibration_objects = [
        {
            "dataset_object_id": "fixture_calibration_a",
            "role": "calibration",
            "candidates": candidates,
        },
        {
            "dataset_object_id": "fixture_calibration_b",
            "role": "calibration",
            "candidates": candidates,
        },
    ]
    decision = calibration_decision(calibration_objects)
    normal = deterministic_normal("nextengine-metal-batch-fixture-v1", 4_096)
    checks = {
        "normal_finite": bool(np.all(np.isfinite(normal))),
        "normal_mean_absolute_maximum": abs(float(np.mean(normal))) <= 0.08,
        "normal_rms_range": 0.90
        <= math.sqrt(float(np.mean(np.square(normal))))
        <= 1.10,
        "two_exponential_fit_uses_two_terms": all(
            len(item["decays_per_second"]) == 2 for item in fixture_fits
        ),
        "candidate_selection_passed": decision["passed"],
        "seeded_residual_selected": decision["selected_candidate_id"]
        == "modal_plus_seeded_subband_residual_v0",
    }
    if not all(checks.values()):
        raise ExecutionError(
            f"deterministic execution fixture failed: {checks}; selection={decision}"
        )
    return {
        "normal_sha256": sha256_bytes(normal.astype("<f8").tobytes()),
        "normal_mean": float(np.mean(normal)),
        "normal_rms": math.sqrt(float(np.mean(np.square(normal)))),
        "observed_sha256": sha256_bytes(observed.astype("<f8").tobytes()),
        "true_residual_sha256": sha256_bytes(true_residual.astype("<f8").tobytes()),
        "candidate_results": candidates,
        "selection": decision,
        "checks": checks,
    }


def preflight(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
) -> dict[str, Any]:
    lineage = validate_protocol(root, base, manifest)
    sources = validate_sources(root, manifest)
    existing = validate_existing_observations(base, manifest)
    fixtures = fixture_report(sha256_bytes(manifest_bytes))
    return {
        "schema": REPORT_SCHEMAS["preflight"],
        "status": "Validated",
        "decision": "RealImpactMetalBatchExecutionAndFixturesFrozen",
        "claim": (
            "HASH_CLOSED_STAGED_EXECUTION_AND_DETERMINISTIC_DSP_FIXTURES / "
            "ZERO_NEW_NETWORK_OR_MEMBER_PAYLOAD_ACCESS / NO_QUALITY_DOMAIN_"
            "RUNTIME_OR_PHYSICS_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "protocol": lineage,
        "bound_sources": sources,
        "existing_observations": existing,
        "fixtures": fixtures,
        "stage_plan": manifest["stage_plan"],
        "dsp": manifest["dsp"],
        "runtime": manifest["runtime"],
        "network_requests": 0,
        "new_member_payload_bytes_read": 0,
        "new_metadata_payload_bytes_read": 0,
        "holdout_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "next_action": (
            "commit this repeated byte-identical preflight, then execute only "
            "acquire-fit for 86_MetalHoledSpoon"
        ),
    }


def object_by_id(manifest: dict[str, Any], object_id: str) -> dict[str, Any]:
    matches = [
        item
        for item in manifest["new_objects"]
        if item["dataset_object_id"] == object_id
    ]
    if len(matches) != 1:
        raise ExecutionError(f"new object identity changed: {object_id}")
    return matches[0]


def report_from_directory(
    root: Path,
    path: Path,
    label: str,
    *,
    manifest_sha256: str,
    decisions: set[str],
) -> tuple[Path, bytes, dict[str, Any]]:
    directory = external_directory(root, path, label)
    report_path = directory / "report.json"
    if not report_path.is_file():
        raise ExecutionError(f"{label} has no report.json")
    report_bytes = report_path.read_bytes()
    try:
        report = json.loads(report_bytes)
    except json.JSONDecodeError as error:
        raise ExecutionError(f"parse {label}: {error}") from error
    if (
        report.get("manifest_sha256") != manifest_sha256
        or report.get("decision") not in decisions
        or report.get("quality_domain_or_runtime_admission") is not False
        or report.get("authored_clip_fallback_required") is not True
    ):
        raise ExecutionError(f"{label} lineage or decision changed")
    return directory, report_bytes, report


def ensure_public_archive_host(url: str) -> None:
    parsed = urllib.parse.urlparse(url)
    if (
        parsed.scheme != "https"
        or parsed.hostname is None
        or parsed.username is not None
        or parsed.password is not None
    ):
        raise ExecutionError("archive URL is not credential-free HTTPS")
    addresses = {
        item[4][0]
        for item in socket.getaddrinfo(parsed.hostname, 443, type=socket.SOCK_STREAM)
    }
    if not addresses or any(
        not ipaddress.ip_address(value).is_global for value in addresses
    ):
        raise ExecutionError(f"archive host did not resolve only publicly: {addresses}")


def download_range(
    item: dict[str, Any], start: int, end: int, target: Path
) -> dict[str, Any]:
    archive = item["archive"]
    length = end - start + 1
    if start < 0 or length <= 0 or end >= archive["bytes"]:
        raise ExecutionError("bounded metal-batch range is invalid")
    ensure_public_archive_host(archive["url"])
    request = urllib.request.Request(
        archive["url"],
        headers={"Range": f"bytes={start}-{end}", "Accept-Encoding": "identity"},
        method="GET",
    )
    digest = hashlib.sha256()
    count = 0
    try:
        with (
            urllib.request.urlopen(request, timeout=60) as response,
            target.open("wb") as output,
        ):
            expected_content_range = f"bytes {start}-{end}/{archive['bytes']}"
            observed = {
                "status": response.status,
                "final_url": response.geturl(),
                "content_range": response.headers.get("Content-Range"),
                "content_length": response.headers.get("Content-Length"),
                "etag": response.headers.get("ETag"),
                "last_modified": response.headers.get("Last-Modified"),
            }
            if observed != {
                "status": 206,
                "final_url": archive["url"],
                "content_range": expected_content_range,
                "content_length": str(length),
                "etag": archive["etag"],
                "last_modified": archive["last_modified_http"],
            }:
                raise ExecutionError(f"archive response identity changed: {observed}")
            while count < length:
                block = response.read(min(1024 * 1024, length - count))
                if not block:
                    break
                digest.update(block)
                output.write(block)
                count += len(block)
            if response.read(1):
                raise ExecutionError("archive response exceeded the frozen range")
    except urllib.error.HTTPError as error:
        raise ExecutionError(
            f"archive range request failed: HTTP {error.code}"
        ) from error
    if count != length:
        raise ExecutionError(f"archive range truncated: expected {length}, got {count}")
    return {
        "dataset_object_id": item["dataset_object_id"],
        "start": start,
        "end": end,
        "bytes": count,
        "sha256": digest.hexdigest(),
        "content_range": f"bytes {start}-{end}/{archive['bytes']}",
        "etag": archive["etag"],
        "last_modified_http": archive["last_modified_http"],
    }


def acquisition_artifacts(item: dict[str, Any]) -> list[dict[str, Any]]:
    artifacts = []
    for index, value in enumerate(item["metadata_ranges"]):
        artifacts.append(
            {
                "name": f"{item['dataset_object_id']}-metadata-{index:02d}.bin",
                "kind": "metadata",
                "range": value,
            }
        )
    audio_start = item["audio_entry"]["data_offset"]
    artifacts.append(
        {
            "name": f"{item['dataset_object_id']}-audio-prefix-32m.deflate",
            "kind": "audio",
            "range": [audio_start, audio_start + protocol.AUDIO_PREFIX_BYTES - 1],
        }
    )
    return artifacts


def acquisition_stage(
    root: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    stage: str,
    output: Path,
    input_path: Path | None,
    selection_path: Path | None,
    holdout_path: Path | None,
) -> tuple[dict[str, Any], bytes]:
    manifest_sha256 = sha256_bytes(manifest_bytes)
    role = stage.removeprefix("acquire-")
    prerequisites = {}
    if role == "fit":
        if input_path is None:
            raise ExecutionError("acquire-fit requires --input repeated preflight")
        _, prerequisite_bytes, prerequisite = report_from_directory(
            root,
            input_path,
            "execution preflight",
            manifest_sha256=manifest_sha256,
            decisions={"RealImpactMetalBatchExecutionAndFixturesFrozen"},
        )
        prerequisites["preflight_sha256"] = sha256_bytes(prerequisite_bytes)
        if prerequisite.get("network_requests") != 0:
            raise ExecutionError("execution preflight was not zero-access")
    elif role == "holdout":
        if selection_path is None:
            raise ExecutionError("acquire-holdout requires --selection")
        _, selection_bytes, selection = report_from_directory(
            root,
            selection_path,
            "candidate selection",
            manifest_sha256=manifest_sha256,
            decisions={"RealImpactMetalBatchCandidateSelected"},
        )
        prerequisites["selection_report_sha256"] = sha256_bytes(selection_bytes)
        prerequisites["selected_candidate_id"] = selection["selected_candidate_id"]
    elif role == "shadow":
        if selection_path is None or holdout_path is None:
            raise ExecutionError("acquire-shadow requires --selection and --holdout")
        _, selection_bytes, selection = report_from_directory(
            root,
            selection_path,
            "candidate selection",
            manifest_sha256=manifest_sha256,
            decisions={"RealImpactMetalBatchCandidateSelected"},
        )
        _, holdout_bytes, holdout = report_from_directory(
            root,
            holdout_path,
            "holdout evaluation",
            manifest_sha256=manifest_sha256,
            decisions={"RealImpactMetalBatchHoldoutRepresentationTransferSupported"},
        )
        if holdout.get("selected_candidate_id") != selection.get(
            "selected_candidate_id"
        ):
            raise ExecutionError("holdout and selection candidate identities differ")
        prerequisites.update(
            {
                "selection_report_sha256": sha256_bytes(selection_bytes),
                "holdout_report_sha256": sha256_bytes(holdout_bytes),
                "selected_candidate_id": selection["selected_candidate_id"],
            }
        )
    else:
        raise ExecutionError(f"unknown acquisition role: {role}")

    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent)
    )
    responses = []
    attempted = 0
    failure = None
    objects = [object_by_id(manifest, value) for value in ROLE_STAGE_OBJECTS[role]]
    try:
        for item in objects:
            for artifact in acquisition_artifacts(item):
                attempted += 1
                response = download_range(
                    item,
                    artifact["range"][0],
                    artifact["range"][1],
                    staging / artifact["name"],
                )
                responses.append({**artifact, **response})
    except (OSError, RuntimeError) as error:
        failure = f"{type(error).__name__}: {error}"
    decision = (
        f"RealImpactMetalBatch{role.title()}InputsAcquired"
        if failure is None
        else f"RealImpactMetalBatch{role.title()}AcquisitionRejected"
    )
    report = {
        "schema": REPORT_SCHEMAS[stage],
        "status": "Validated",
        "decision": decision,
        "claim": (
            f"FROZEN_{role.upper()}_ROLE_RANGES_ONLY / NO_RETRY_PREFIX_GROWTH_"
            "QUALITY_DOMAIN_RUNTIME_PHYSICS_OR_PLANTER_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "role_stage": role,
        "objects": ROLE_STAGE_OBJECTS[role],
        "prerequisites": prerequisites,
        "network_requests_attempted": attempted,
        "responses": responses,
        "failure": failure,
        "additional_requests_allowed": 0,
        "new_member_payload_bytes_read": sum(item["bytes"] for item in responses),
        "next_action": (
            f"decode only the immutable {role} cache"
            if failure is None
            else "stop with authored-clip fallback; do not retry, grow a prefix or substitute an object"
        ),
    }
    try:
        report_bytes = canonical_json(report)
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return report, report_bytes


def decode_local_record(
    block: bytes,
    block_start: int,
    item: dict[str, Any],
    entry: list[Any],
) -> bytes:
    basename, expected_crc32, compressed_bytes, uncompressed_bytes, local_offset = entry
    offset = local_offset - block_start
    if offset < 0 or offset + 30 > len(block):
        raise ExecutionError(f"metadata local header is outside range: {basename}")
    values = struct.unpack_from("<4s5H3L2H", block, offset)
    signature, flags, method = values[0], values[2], values[3]
    crc32, compressed, uncompressed = values[6], values[7], values[8]
    name_bytes, extra_bytes = values[9], values[10]
    name_start = offset + 30
    data_start = name_start + name_bytes + extra_bytes
    data_stop = data_start + compressed
    if data_stop > len(block):
        raise ExecutionError(f"metadata member is outside range: {basename}")
    try:
        name = block[name_start : name_start + name_bytes].decode("utf-8")
    except UnicodeDecodeError as error:
        raise ExecutionError("metadata member name is not UTF-8") from error
    expected_name = f"{item['dataset_object_id']}/preprocessed/{basename}"
    if (
        signature != b"PK\x03\x04"
        or flags != 0
        or method != 8
        or f"{crc32:08x}" != expected_crc32
        or compressed != compressed_bytes
        or uncompressed != uncompressed_bytes
        or name != expected_name
    ):
        raise ExecutionError(f"metadata local member changed: {basename}")
    try:
        payload = zlib.decompress(block[data_start:data_stop], -15)
    except zlib.error as error:
        raise ExecutionError(f"inflate metadata {basename}: {error}") from error
    if (
        len(payload) != uncompressed_bytes
        or f"{zlib.crc32(payload) & 0xFFFFFFFF:08x}" != expected_crc32
    ):
        raise ExecutionError(f"metadata integrity changed: {basename}")
    return payload


def load_metadata_npy(payload: bytes, label: str) -> np.ndarray:
    stream = io.BytesIO(payload)
    try:
        array = np.load(stream, allow_pickle=False)
    except (ValueError, OSError) as error:
        raise ExecutionError(f"decode metadata {label}: {error}") from error
    if (
        stream.tell() != len(payload)
        or array.shape != (3_000,)
        or array.dtype.str != "<i8"
    ):
        raise ExecutionError(
            f"metadata NPY changed: {label}, {array.shape}, {array.dtype.str}"
        )
    return array


def validate_condition(arrays: dict[str, np.ndarray]) -> dict[str, Any]:
    rows = slice(0, 15)
    angles = arrays["angle.npy"][rows]
    distances = arrays["distance.npy"][rows]
    microphones = arrays["micID.npy"][rows]
    vertices = arrays["vertexID.npy"][rows]
    if (
        len(np.unique(angles)) != 1
        or len(np.unique(distances)) != 1
        or len(np.unique(vertices)) != 1
        or not np.array_equal(microphones, np.arange(15, dtype=np.int64))
    ):
        raise ExecutionError("frozen impact-zero listener condition changed")
    return {
        "angle": int(angles[0]),
        "distance": int(distances[0]),
        "vertex_id": int(vertices[0]),
        "microphone_ids": microphones.tolist(),
    }


def validate_audio_header(header: bytes, shape: list[int]) -> None:
    if (
        len(header) != protocol.NPY_HEADER_BYTES
        or header[:6] != b"\x93NUMPY"
        or header[6:8] != b"\x01\x00"
        or int.from_bytes(header[8:10], "little") != 118
    ):
        raise ExecutionError("audio NPY header identity changed")
    text = header[10:].decode("latin1")
    if (
        "'descr': '<f4'" not in text
        or "'fortran_order': False" not in text
        or f"'shape': ({shape[0]}, {shape[1]})" not in text
        or not text.endswith("\n")
    ):
        raise ExecutionError(f"audio NPY descriptor changed: {text!r}")


def decode_audio_prefix(prefix: Path, target: Path, shape: list[int]) -> dict[str, Any]:
    decoder = zlib.decompressobj(-15)
    header = bytearray()
    digest = hashlib.sha256()
    decoded = 0
    decoded_bytes = 15 * shape[1] * np.dtype("<f4").itemsize
    target_total = protocol.NPY_HEADER_BYTES + decoded_bytes
    with prefix.open("rb") as source, target.open("wb") as output:
        while len(header) + decoded < target_total:
            compressed = source.read(1024 * 1024)
            if not compressed:
                break
            pending = compressed
            while pending and len(header) + decoded < target_total:
                remaining = target_total - len(header) - decoded
                block = decoder.decompress(pending, remaining)
                pending = decoder.unconsumed_tail
                if len(header) < protocol.NPY_HEADER_BYTES:
                    take = min(protocol.NPY_HEADER_BYTES - len(header), len(block))
                    header.extend(block[:take])
                    block = block[take:]
                if block:
                    digest.update(block)
                    output.write(block)
                    decoded += len(block)
                if not pending:
                    break
    validate_audio_header(bytes(header), shape)
    if decoded != decoded_bytes:
        raise ExecutionError(
            f"audio prefix decoded {decoded} bytes, expected {decoded_bytes}"
        )
    return {
        "header_sha256": sha256_bytes(bytes(header)),
        "decoded_sha256": digest.hexdigest(),
        "decoded_bytes": decoded,
        "decoded_shape": [15, shape[1]],
    }


def decode_stage(
    root: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    stage: str,
    acquisition_path: Path,
    output: Path,
) -> tuple[dict[str, Any], bytes]:
    role = stage.removeprefix("decode-")
    manifest_sha256 = sha256_bytes(manifest_bytes)
    acquisition, acquisition_bytes, acquisition_report = report_from_directory(
        root,
        acquisition_path,
        f"{role} acquisition",
        manifest_sha256=manifest_sha256,
        decisions={f"RealImpactMetalBatch{role.title()}InputsAcquired"},
    )
    if acquisition_report.get("role_stage") != role:
        raise ExecutionError("acquisition role changed before decode")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent)
    )
    decoded_objects = []
    try:
        for object_id in ROLE_STAGE_OBJECTS[role]:
            item = object_by_id(manifest, object_id)
            range_blocks = []
            for index, (start, end) in enumerate(item["metadata_ranges"]):
                path = acquisition / f"{object_id}-metadata-{index:02d}.bin"
                if not path.is_file() or path.stat().st_size != end - start + 1:
                    raise ExecutionError(
                        f"metadata acquisition changed: {object_id}/{index}"
                    )
                range_blocks.append((start, end, path.read_bytes()))
            arrays = {}
            metadata_hashes = {}
            for entry in item["metadata_entries"]:
                local_offset = entry[4]
                matches = [
                    (start, block)
                    for start, end, block in range_blocks
                    if start <= local_offset <= end
                ]
                if len(matches) != 1:
                    raise ExecutionError(f"metadata range mapping changed: {entry[0]}")
                payload = decode_local_record(matches[0][1], matches[0][0], item, entry)
                arrays[entry[0]] = load_metadata_npy(payload, entry[0])
                metadata_hashes[entry[0]] = sha256_bytes(payload)
            condition = validate_condition(arrays)
            audio_prefix = acquisition / f"{object_id}-audio-prefix-32m.deflate"
            if (
                not audio_prefix.is_file()
                or audio_prefix.stat().st_size != protocol.AUDIO_PREFIX_BYTES
            ):
                raise ExecutionError(f"audio prefix changed: {object_id}")
            decoded_name = f"{object_id}-impact000-mics00-14.f32le"
            decoded = decode_audio_prefix(
                audio_prefix, staging / decoded_name, item["audio_entry"]["shape"]
            )
            decoded_objects.append(
                {
                    "dataset_object_id": object_id,
                    "family_group": item["family_group"],
                    "role": item["role"],
                    "condition": condition,
                    "metadata_payload_sha256": metadata_hashes,
                    "decoded_path": decoded_name,
                    **decoded,
                }
            )
        report = {
            "schema": REPORT_SCHEMAS[stage],
            "status": "Validated",
            "decision": f"RealImpactMetalBatch{role.title()}InputsDecoded",
            "claim": (
                f"FROZEN_{role.upper()}_ROLE_CONDITION_AND_15_LISTENER_ROWS_ONLY / "
                "NO_QUALITY_DOMAIN_RUNTIME_PHYSICS_OR_PLANTER_CREDIT"
            ),
            **common_report(manifest_bytes, runner_sha256),
            "role_stage": role,
            "acquisition_report_sha256": sha256_bytes(acquisition_bytes),
            "objects": decoded_objects,
            "network_requests": 0,
            "additional_member_payload_bytes_read": 0,
            "next_action": (
                "run development/calibration selection only"
                if role == "fit"
                else (
                    "evaluate only the already selected candidate on holdout"
                    if role == "holdout"
                    else "evaluate only the already selected candidate on the shadow family"
                )
            ),
        }
        report_bytes = canonical_json(report)
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    return report, report_bytes


def load_raw_block(
    path: Path, rows: int, samples: int, batch_rows: list[int]
) -> np.ndarray:
    expected_bytes = rows * samples * np.dtype("<f4").itemsize
    if not path.is_file() or path.stat().st_size != expected_bytes:
        raise ExecutionError(f"observation block size changed: {path}")
    mapped = np.memmap(path, dtype="<f4", mode="r", shape=(rows, samples), order="C")
    selected = np.asarray(mapped[batch_rows], dtype=np.float64)
    if selected.shape != (15, samples) or not np.all(np.isfinite(selected)):
        raise ExecutionError(f"observation block values changed: {path}")
    return selected


def load_existing_observations(
    base: Path, manifest: dict[str, Any]
) -> list[tuple[dict[str, Any], np.ndarray]]:
    result = []
    for reference in manifest["existing_observations"]:
        path = (base / reference["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(base.parent)
            or path.stat().st_size != reference["bytes"]
            or sha256_file(path) != reference["sha256"]
        ):
            raise ExecutionError(
                f"existing observation changed: {reference['dataset_object_id']}"
            )
        result.append(
            (
                reference,
                load_raw_block(
                    path,
                    reference["stored_rows"],
                    reference["samples_per_row"],
                    reference["batch_rows"],
                ),
            )
        )
    return result


def decoded_objects(
    root: Path,
    path: Path,
    manifest_sha256: str,
    role: str,
) -> tuple[bytes, dict[str, Any], list[tuple[dict[str, Any], np.ndarray]]]:
    directory, report_bytes, report = report_from_directory(
        root,
        path,
        f"{role} decode",
        manifest_sha256=manifest_sha256,
        decisions={f"RealImpactMetalBatch{role.title()}InputsDecoded"},
    )
    if report.get("role_stage") != role:
        raise ExecutionError("decode role changed")
    result = []
    for item in report["objects"]:
        decoded = directory / item["decoded_path"]
        rows, samples = item["decoded_shape"]
        if rows != 15 or sha256_file(decoded) != item["decoded_sha256"]:
            raise ExecutionError(f"decoded object changed: {item['dataset_object_id']}")
        result.append((item, load_raw_block(decoded, rows, samples, list(range(15)))))
    return report_bytes, report, result


def compact_object_analysis(item: dict[str, Any]) -> dict[str, Any]:
    result = dict(item)
    for candidate in result["candidates"].values():
        for fit in candidate["band_fits"]:
            fit["listener_gains"] = {
                "minimum": min(fit["listener_gains"]),
                "median": median(fit["listener_gains"]),
                "maximum": max(fit["listener_gains"]),
                "sha256": sha256_bytes(
                    np.asarray(fit["listener_gains"], dtype="<f8").tobytes()
                ),
            }
    return result


def selection_stage(
    root: Path,
    base: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    runner_sha256: str,
    decode_path: Path,
) -> dict[str, Any]:
    manifest_sha256 = sha256_bytes(manifest_bytes)
    decode_bytes, _, new_objects = decoded_objects(
        root, decode_path, manifest_sha256, "fit"
    )
    objects = []
    for reference, channels in load_existing_observations(base, manifest):
        objects.append(
            analyze_observation(
                channels,
                manifest_sha256,
                reference["dataset_object_id"],
                reference["role"],
                SELECTABLE_CANDIDATES,
            )
        )
    for reference, channels in new_objects:
        objects.append(
            analyze_observation(
                channels,
                manifest_sha256,
                reference["dataset_object_id"],
                reference["role"],
                SELECTABLE_CANDIDATES,
            )
        )
    decision = calibration_decision(objects)
    passed = decision["passed"]
    return {
        "schema": REPORT_SCHEMAS["select"],
        "status": "Validated",
        "decision": (
            "RealImpactMetalBatchCandidateSelected"
            if passed
            else "RealImpactMetalBatchCalibrationRejected"
        ),
        "claim": (
            "DEVELOPMENT_CALIBRATION_ONLY_REPRESENTATION_SELECTION / NO_"
            "HOLDOUT_SHADOW_QUALITY_DOMAIN_RUNTIME_PHYSICS_OR_PLANTER_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "fit_decode_report_sha256": sha256_bytes(decode_bytes),
        "objects": [compact_object_analysis(item) for item in objects],
        "calibration": decision,
        "selected_candidate_id": decision["selected_candidate_id"],
        "holdout_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "network_requests": 0,
        "next_action": (
            "freeze this report, then acquire only the one-object holdout"
            if passed
            else "stop with authored-clip fallback; preserve the immutable calibration rejection"
        ),
    }


def evaluate_selected_stage(
    root: Path,
    manifest_bytes: bytes,
    runner_sha256: str,
    stage: str,
    decode_path: Path,
    selection_path: Path,
    holdout_path: Path | None,
) -> dict[str, Any]:
    manifest_sha256 = sha256_bytes(manifest_bytes)
    _, selection_bytes, selection = report_from_directory(
        root,
        selection_path,
        "candidate selection",
        manifest_sha256=manifest_sha256,
        decisions={"RealImpactMetalBatchCandidateSelected"},
    )
    candidate_id = selection["selected_candidate_id"]
    role = stage
    prerequisites = {"selection_report_sha256": sha256_bytes(selection_bytes)}
    if role == "shadow":
        if holdout_path is None:
            raise ExecutionError("shadow evaluation requires --holdout")
        _, holdout_bytes, holdout = report_from_directory(
            root,
            holdout_path,
            "holdout evaluation",
            manifest_sha256=manifest_sha256,
            decisions={"RealImpactMetalBatchHoldoutRepresentationTransferSupported"},
        )
        if holdout["selected_candidate_id"] != candidate_id:
            raise ExecutionError("holdout candidate changed before shadow")
        prerequisites["holdout_report_sha256"] = sha256_bytes(holdout_bytes)
    decode_bytes, _, objects = decoded_objects(root, decode_path, manifest_sha256, role)
    analyses = [
        analyze_observation(
            channels,
            manifest_sha256,
            reference["dataset_object_id"],
            reference["role"],
            [candidate_id],
        )
        for reference, channels in objects
    ]
    ratios = [item["candidates"][candidate_id]["ratios"] for item in analyses]
    if role == "holdout":
        thresholds = protocol.SELECTION_GATES["holdout"]
        per_metric = ratios[0]
        checks = {
            "each_gating_metric_ratio": all(
                value <= thresholds["maximum_each_gating_metric_ratio"]
                for value in per_metric.values()
            ),
            "maximum_any_gating_metric_ratio": max(per_metric.values())
            <= thresholds["maximum_any_gating_metric_ratio"],
        }
        supported_decision = (
            "RealImpactMetalBatchHoldoutRepresentationTransferSupported"
        )
        rejected_decision = "RealImpactMetalBatchHoldoutRejected"
        next_pass = (
            "freeze this report, then acquire only the two-object Spatula shadow family"
        )
    else:
        thresholds = protocol.SELECTION_GATES["shadow_family"]
        family_medians = {
            name: median(row[name] for row in ratios)
            for name in protocol.METRICS["gating"]
        }
        checks = {
            "family_median_each_gating_metric_ratio": all(
                value <= thresholds["maximum_family_median_each_gating_metric_ratio"]
                for value in family_medians.values()
            ),
            "maximum_any_object_gating_metric_ratio": max(
                value for row in ratios for value in row.values()
            )
            <= thresholds["maximum_any_object_gating_metric_ratio"],
        }
        supported_decision = "RealImpactMetalBatchShadowRepresentationTransferSupported"
        rejected_decision = "RealImpactMetalBatchShadowRejected"
        next_pass = (
            "record representation-transfer evidence only; quality/domain/runtime "
            "admission remains disabled"
        )
    passed = all(checks.values())
    return {
        "schema": REPORT_SCHEMAS[role],
        "status": "Validated",
        "decision": supported_decision if passed else rejected_decision,
        "claim": (
            f"FROZEN_{role.upper()}_REPRESENTATION_TRANSFER_ONLY / NO_QUALITY_"
            "DOMAIN_RUNTIME_PHYSICS_OR_PLANTER_CREDIT"
        ),
        **common_report(manifest_bytes, runner_sha256),
        "prerequisites": prerequisites,
        "decode_report_sha256": sha256_bytes(decode_bytes),
        "selected_candidate_id": candidate_id,
        "objects": [compact_object_analysis(item) for item in analyses],
        "gate": {"passed": passed, "checks": checks},
        "network_requests": 0,
        "additional_member_payload_bytes_read": 0,
        "next_action": (
            next_pass
            if passed
            else "stop with authored-clip fallback; preserve the immutable rejection without retuning"
        ),
    }


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    runner_sha256 = sha256_file(Path(__file__).resolve())
    if arguments.stage == "manifest":
        if any(
            value is not None
            for value in [
                arguments.manifest,
                arguments.input,
                arguments.selection,
                arguments.holdout,
            ]
        ):
            raise ExecutionError("manifest stage accepts only --output")
        output = external_output(root, arguments.output, directory=False)
        data = canonical_json(expected_manifest(runner_sha256))
        write_new_file(output, data)
        print(f"REALIMPACT metal batch execution manifest: {output}")
        print(f"manifest sha256: {sha256_bytes(data)}")
        return 0

    if arguments.manifest is None:
        raise ExecutionError(f"{arguments.stage} requires --manifest")
    manifest_path = external_file(root, arguments.manifest, "execution manifest")
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    base = manifest_path.parent
    output = external_output(root, arguments.output, directory=True)

    if arguments.stage == "preflight":
        if any(
            value is not None
            for value in [arguments.input, arguments.selection, arguments.holdout]
        ):
            raise ExecutionError("preflight accepts no input stage")
        report = preflight(root, base, manifest_bytes, manifest, runner_sha256)
        report_bytes = publish_directory(output, report)
    elif arguments.stage.startswith("acquire-"):
        report, report_bytes = acquisition_stage(
            root,
            manifest_bytes,
            manifest,
            runner_sha256,
            arguments.stage,
            output,
            arguments.input,
            arguments.selection,
            arguments.holdout,
        )
    elif arguments.stage.startswith("decode-"):
        if arguments.input is None:
            raise ExecutionError(f"{arguments.stage} requires --input acquisition")
        if arguments.selection is not None or arguments.holdout is not None:
            raise ExecutionError(f"{arguments.stage} accepts only --input")
        report, report_bytes = decode_stage(
            root,
            manifest_bytes,
            manifest,
            runner_sha256,
            arguments.stage,
            arguments.input,
            output,
        )
    elif arguments.stage == "select":
        if arguments.input is None:
            raise ExecutionError("select requires --input decode-fit")
        if arguments.selection is not None or arguments.holdout is not None:
            raise ExecutionError("select accepts only --input")
        report = selection_stage(
            root,
            base,
            manifest_bytes,
            manifest,
            runner_sha256,
            arguments.input,
        )
        report_bytes = publish_directory(output, report)
    elif arguments.stage in {"holdout", "shadow"}:
        if arguments.input is None or arguments.selection is None:
            raise ExecutionError(
                f"{arguments.stage} requires --input decode and --selection"
            )
        if arguments.stage == "holdout" and arguments.holdout is not None:
            raise ExecutionError("holdout evaluation does not accept --holdout")
        report = evaluate_selected_stage(
            root,
            manifest_bytes,
            runner_sha256,
            arguments.stage,
            arguments.input,
            arguments.selection,
            arguments.holdout,
        )
        report_bytes = publish_directory(output, report)
    else:
        raise ExecutionError(f"unsupported stage: {arguments.stage}")

    print(f"REALIMPACT metal batch execution {arguments.stage}: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report.get('network_requests', 0)}")
    print(
        "new member payload bytes read: "
        f"{report.get('new_member_payload_bytes_read', 0)}"
    )
    print("quality/domain/runtime admission: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
