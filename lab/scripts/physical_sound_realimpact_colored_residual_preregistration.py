#!/usr/bin/env python3
"""Preregister a one-change colored-residual REALIMPACT successor."""

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
import physical_sound_realimpact_residual_representation_diagnostic as diagnostic

MANIFEST_SCHEMA = (
    "nextengine.experimental-realimpact-colored-residual-successor.manifest.v1"
)
REPORT_SCHEMA = (
    "nextengine.experimental-realimpact-colored-residual-successor-preflight.report.v1"
)
REVISION = "seeded-colored-subband-residual-preregistration-v1"
EXECUTION_RUNNER_SHA256 = (
    "8484f9477b6ba9282bce3b6ce08185411c145bc095efa24c8cbe85b53db413cc"
)
DIAGNOSTIC_RUNNER_SHA256 = (
    "d2ab638b7b94d48f94ab2babdb93f292a8d08ce1f8f154863c05b23838443f88"
)
PARENT_DISCOVERY_MANIFEST_SHA256 = (
    "873d82e3ea887f89c05fbb9ada924b43f6087062c7394b1be047e2ca54e23ae4"
)
PARENT_DISCOVERY_PREFLIGHT_SHA256 = (
    "578e143ca137ead516c43d5320df986b4926b3855662322c5325c69ae3eaec31"
)
DIAGNOSTIC_REPORT_SHA256 = (
    "d8acc909d9d0db583a8addab37fb1ac0e4f8a2f38595e093b3d26ec67caba1eb"
)
OFFICIAL_ROSTER_COMMIT = "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987"
OFFICIAL_ROSTER_SHA256 = (
    "3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a"
)

CANDIDATE_ID = "modal_plus_seeded_colored_subband_residual_v1"
CONTROL_ID = "modal_plus_seeded_subband_residual_v0"
COLOR_DCT_COEFFICIENTS = 8
COLOR_COEFFICIENT_LIMIT_DB = 18.0
COLOR_SHAPE_LIMIT_DB = 18.0
COLOR_POWER_FLOOR_RATIO = 1.0e-8
FIT_STOP = execution.protocol.CALIBRATION_STOP_SAMPLE
RENDER_STOP = 32_768

DEVELOPMENT_OBJECTS = [
    "17_IronSkillet",
    "67_IronPlate",
    "90_MetalLadle",
    "43_IronMortar",
    "86_MetalHoledSpoon",
    "91_MetalSpoon",
]
FRESH_OBJECTS = [
    {
        "dataset_object_id": "19_Pan",
        "roster_index": 3,
        "semantic_family_group": "pan",
        "role": "calibration",
    },
    {
        "dataset_object_id": "37_PiePan",
        "roster_index": 14,
        "semantic_family_group": "pan",
        "role": "calibration",
    },
    {
        "dataset_object_id": "22_Cup",
        "roster_index": 4,
        "semantic_family_group": "cup",
        "role": "holdout",
    },
]
SHADOW_OBJECTS = [
    {
        "dataset_object_id": "89_MetalSpatula",
        "roster_index": 41,
        "semantic_family_group": "metal_spatula",
        "role": "shadow",
    },
    {
        "dataset_object_id": "92_MetalSpatula",
        "roster_index": 44,
        "semantic_family_group": "metal_spatula",
        "role": "shadow",
    },
]


class PreregistrationError(RuntimeError):
    """The frozen successor boundary or synthetic control failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["manifest", "preflight"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--parent-manifest", type=Path)
    parser.add_argument("--parent-preflight", type=Path)
    parser.add_argument("--diagnostic", type=Path)
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
        raise PreregistrationError(f"{label} must be an external file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise PreregistrationError(f"output must be a new external file: {resolved}")
    return resolved


def source_references() -> list[dict[str, str]]:
    return [
        {
            "path": "lab/scripts/physical_sound_realimpact_metal_batch_execute.py",
            "sha256": EXECUTION_RUNNER_SHA256,
        },
        {
            "path": "lab/scripts/physical_sound_realimpact_residual_representation_diagnostic.py",
            "sha256": DIAGNOSTIC_RUNNER_SHA256,
        },
    ]


def candidate_profile() -> dict[str, Any]:
    return {
        "candidate_id": CANDIDATE_ID,
        "control_id": CONTROL_ID,
        "single_changed_hypothesis": "within_band_coloration",
        "unchanged": {
            "sample_rate_hz": execution.protocol.SAMPLE_RATE_HZ,
            "analysis_samples": execution.protocol.ANALYSIS_SAMPLES,
            "modal_baseline": "execution runner 8484f947…13cc unchanged",
            "band_edges_hz": execution.protocol.BAND_EDGES_HZ,
            "fit_stop_sample": FIT_STOP,
            "render_stop_sample": RENDER_STOP,
            "envelope": "same nonnegative two-exponential grid fit",
            "listener_gain": "same per-band listener RMS / pooled RMS",
            "seed_fields": ["manifest_sha256", "dataset_object_id", "band_index"],
            "cross_listener_excitation": "same shared rank-one excitation control",
        },
        "changed": {
            "analysis_window": "symmetric Hann, 512 samples, 256-sample hop",
            "target": (
                "mean listener/frame power per 512-point RFFT bin inside each "
                "existing rectangular band, converted to dB and mean-centred"
            ),
            "basis": "orthogonal DCT-II k=1..min(8,N-1)",
            "coefficient_limit_db": COLOR_COEFFICIENT_LIMIT_DB,
            "reconstructed_shape_limit_db": COLOR_SHAPE_LIMIT_DB,
            "power_floor_ratio": COLOR_POWER_FLOOR_RATIO,
            "render": (
                "interpolate the bounded reconstructed dB shape to the 32768-point "
                "RFFT bins, multiply the existing deterministic band excitation, "
                "inverse RFFT and renormalize to unit RMS"
            ),
        },
        "parameter_bound": {
            "maximum_coefficients_per_band": COLOR_DCT_COEFFICIENTS,
            "maximum_bands": len(execution.protocol.BAND_EDGES_HZ) - 1,
            "maximum_color_coefficients_per_object": COLOR_DCT_COEFFICIENTS
            * (len(execution.protocol.BAND_EDGES_HZ) - 1),
            "runtime_learning_or_inference": False,
        },
    }


def archive_url(object_id: str) -> str:
    return f"https://downloads.cs.stanford.edu/viscam/RealImpact/{object_id}.zip"


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    fresh = [
        {
            **item,
            "archive_url": archive_url(item["dataset_object_id"]),
            "archive_identity": "DISCOVERY_REQUIRED_BEFORE_MEMBER_ACCESS",
        }
        for item in FRESH_OBJECTS
    ]
    shadow = [
        {
            **item,
            "archive_url": archive_url(item["dataset_object_id"]),
            "archive_identity": "INHERIT_PARENT_STRUCTURAL_DISCOVERY_ONLY",
            "member_payload_bytes_read": 0,
        }
        for item in SHADOW_OBJECTS
    ]
    return {
        "schema": MANIFEST_SCHEMA,
        "revision": REVISION,
        "runner": {
            "path": "lab/scripts/physical_sound_realimpact_colored_residual_preregistration.py",
            "sha256": runner_sha256,
        },
        "sources": source_references(),
        "lineage": {
            "parent_discovery_manifest_sha256": PARENT_DISCOVERY_MANIFEST_SHA256,
            "parent_discovery_preflight_sha256": PARENT_DISCOVERY_PREFLIGHT_SHA256,
            "rejected_residual_diagnostic_sha256": DIAGNOSTIC_REPORT_SHA256,
        },
        "official_roster": {
            "repository_commit": OFFICIAL_ROSTER_COMMIT,
            "object_names_sha256": OFFICIAL_ROSTER_SHA256,
            "scope": (
                "exact published names and indices only; names do not grant "
                "material identity"
            ),
        },
        "claim": (
            "FRESH_GROUPED_REPRESENTATION_SUCCESSOR_ONLY / NO_MATERIAL_QUALITY_"
            "DOMAIN_RUNTIME_PHYSICS_OR_PRODUCTION_CREDIT"
        ),
        "partition": {
            "development_diagnostic_only": DEVELOPMENT_OBJECTS,
            "calibration": ["19_Pan", "37_PiePan"],
            "holdout": ["22_Cup"],
            "shadow": ["89_MetalSpatula", "92_MetalSpatula"],
            "semantic_family_group_disjoint_across_roles": True,
            "opened_metal_spoon_forbidden_in_calibration_holdout_shadow": True,
            "statistical_release_credit": False,
        },
        "fresh_objects": fresh,
        "shadow_objects": shadow,
        "candidate": candidate_profile(),
        "metrics": {
            "unchanged_gating": execution.protocol.METRICS["gating"],
            "new_representation_gating": [
                "within_band_spectral_shape_mae_db",
                "short_lag_autocorrelation_mae",
            ],
            "diagnostic_only": [
                "waveform_nrmse",
                "modulation_power_mae_db",
                "cross_listener_coherence_mae",
                "effective_rank",
            ],
        },
        "selection_gates": {
            "calibration": {
                "maximum_median_existing_metric_ratio": 0.95,
                "maximum_any_existing_metric_ratio": 1.05,
                "maximum_median_spectral_shape_error_ratio": 0.75,
                "maximum_median_autocorrelation_error_ratio": 0.85,
                "minimum_improved_calibration_object_fraction": 1.0,
            },
            "holdout": {
                "maximum_each_existing_metric_ratio": 0.95,
                "maximum_any_existing_metric_ratio": 1.05,
                "maximum_spectral_shape_error_ratio": 0.80,
                "maximum_autocorrelation_error_ratio": 0.90,
            },
            "shadow_family": {
                "maximum_family_median_existing_metric_ratio": 0.95,
                "maximum_any_object_existing_metric_ratio": 1.05,
                "maximum_family_median_spectral_shape_error_ratio": 0.85,
                "maximum_family_median_autocorrelation_error_ratio": 0.95,
            },
        },
        "stage_plan": [
            "manifest",
            "zero-network-preflight",
            "fresh-archive-identity-discovery",
            "fresh-structure-only-tail-and-local-header-audit",
            "payload-and-decoder-protocol",
            "execution-fixture-preflight",
            "calibration-acquire-decode-select",
            "immutable-selection-commit",
            "holdout-acquire-decode-evaluate",
            "immutable-holdout-commit",
            "shadow-acquire-decode-evaluate-only-if-holdout-supported",
        ],
        "access": {
            "preflight_network_requests": 0,
            "preflight_new_member_payload_bytes": 0,
            "next_discovery_head_requests": 3,
            "next_discovery_objects": ["19_Pan", "37_PiePan", "22_Cup"],
            "shadow_member_access_before_successful_holdout": False,
            "retry_prefix_growth_or_object_substitution": False,
            "quality_domain_or_runtime_admission_allowed": False,
            "authored_clip_fallback_required": True,
        },
        "stop_rule": (
            "any source, identity, structure, decode, repeat, calibration or holdout "
            "failure stops later access; never tune against opened holdout, reuse "
            "Metal Spoon as holdout or fold listener covariance into this revision"
        ),
    }


def dct_color_profile(
    band_signal: np.ndarray, low_hz: float, high_hz: float
) -> dict[str, Any]:
    if band_signal.ndim != 2 or band_signal.shape[1] < FIT_STOP:
        raise PreregistrationError("color profile requires the frozen fit window")
    band = execution.band_project(band_signal[:, :FIT_STOP], low_hz, high_hz)
    window = np.hanning(diagnostic.FRAME_SAMPLES).astype(np.float64)
    starts = range(0, FIT_STOP - diagnostic.FRAME_SAMPLES + 1, diagnostic.HOP_SAMPLES)
    frames = np.stack(
        [
            band[:, start : start + diagnostic.FRAME_SAMPLES] * window
            for start in starts
        ],
        axis=1,
    )
    power = np.mean(np.square(np.abs(np.fft.rfft(frames, axis=2))), axis=(0, 1))
    frequencies = np.fft.rfftfreq(
        diagnostic.FRAME_SAMPLES, d=1.0 / execution.protocol.SAMPLE_RATE_HZ
    )
    mask = (frequencies >= low_hz) & (
        frequencies <= high_hz
        if high_hz == execution.protocol.SAMPLE_RATE_HZ / 2
        else frequencies < high_hz
    )
    selected_frequencies = frequencies[mask]
    selected_power = power[mask]
    if len(selected_power) < 2 or float(np.max(selected_power)) <= 0.0:
        raise PreregistrationError("color profile band has insufficient energy/bins")
    floor = max(float(np.max(selected_power)) * COLOR_POWER_FLOOR_RATIO, 1.0e-300)
    shape = 10.0 * np.log10(np.maximum(selected_power, floor))
    shape -= float(np.mean(shape))
    sample_index = np.arange(len(shape), dtype=np.float64)
    coefficient_count = min(COLOR_DCT_COEFFICIENTS, len(shape) - 1)
    coefficients = []
    reconstructed = np.zeros_like(shape)
    for order in range(1, coefficient_count + 1):
        basis = np.cos(math.pi * (sample_index + 0.5) * order / len(shape))
        coefficient = float(2.0 / len(shape) * np.dot(shape, basis))
        coefficient = float(
            np.clip(
                coefficient,
                -COLOR_COEFFICIENT_LIMIT_DB,
                COLOR_COEFFICIENT_LIMIT_DB,
            )
        )
        coefficients.append(coefficient)
        reconstructed += coefficient * basis
    reconstructed = np.clip(reconstructed, -COLOR_SHAPE_LIMIT_DB, COLOR_SHAPE_LIMIT_DB)
    reconstructed -= float(np.mean(reconstructed))
    return {
        "frequencies_hz": selected_frequencies,
        "coefficients_db": coefficients,
        "shape_db": reconstructed,
    }


def colored_band_noise(
    seed: str,
    length: int,
    low_hz: float,
    high_hz: float,
    profile: dict[str, Any],
) -> np.ndarray:
    excitation = execution.unit_band_noise(seed, length, low_hz, high_hz)
    spectrum = np.fft.rfft(excitation)
    frequencies = np.fft.rfftfreq(length, d=1.0 / execution.protocol.SAMPLE_RATE_HZ)
    mask = execution.frequency_mask(length, low_hz, high_hz)
    shape = np.interp(
        frequencies[mask],
        profile["frequencies_hz"],
        profile["shape_db"],
        left=float(profile["shape_db"][0]),
        right=float(profile["shape_db"][-1]),
    )
    spectrum[mask] *= np.power(10.0, shape / 20.0)
    spectrum[~mask] = 0.0
    output = np.fft.irfft(spectrum, n=length)
    rms = math.sqrt(float(np.mean(np.square(output))))
    if not math.isfinite(rms) or rms <= 1.0e-15:
        raise PreregistrationError("colored excitation has invalid RMS")
    return output / rms


def fixture_report() -> dict[str, Any]:
    time = np.arange(RENDER_STOP, dtype=np.float64) / execution.protocol.SAMPLE_RATE_HZ
    listener_gains = np.linspace(0.75, 1.25, 15, dtype=np.float64)
    observed = np.zeros((15, RENDER_STOP), dtype=np.float64)
    control = np.zeros_like(observed)
    candidate = np.zeros_like(observed)
    profiles = []
    for band_index, (low_hz, high_hz) in enumerate(
        itertools.pairwise(execution.protocol.BAND_EDGES_HZ)
    ):
        source_seed = f"colored-residual-fixture-source-{band_index}"
        source = execution.unit_band_noise(source_seed, RENDER_STOP, low_hz, high_hz)
        spectrum = np.fft.rfft(source)
        mask = execution.frequency_mask(RENDER_STOP, low_hz, high_hz)
        position = np.linspace(-1.0, 1.0, int(np.sum(mask)))
        target_shape_db = (7.5 - 0.4 * band_index) * np.cos(
            math.pi * position
        ) + 3.0 * np.cos(2.0 * math.pi * position)
        spectrum[mask] *= np.power(10.0, target_shape_db / 20.0)
        spectrum[~mask] = 0.0
        source = np.fft.irfft(spectrum, n=RENDER_STOP)
        source /= math.sqrt(float(np.mean(np.square(source))))
        envelope = (0.05 / (band_index + 1)) * (
            0.7 * np.exp(-10.0 * time) + 0.3 * np.exp(-40.0 * time)
        )
        observed += listener_gains[:, None] * envelope[None, :] * source[None, :]

        profile = dct_color_profile(observed[:, :FIT_STOP], low_hz, high_hz)
        render_seed = f"colored-residual-fixture-render-{band_index}"
        white = execution.unit_band_noise(render_seed, RENDER_STOP, low_hz, high_hz)
        colored = colored_band_noise(render_seed, RENDER_STOP, low_hz, high_hz, profile)
        control += listener_gains[:, None] * envelope[None, :] * white[None, :]
        candidate += listener_gains[:, None] * envelope[None, :] * colored[None, :]
        profiles.append(
            {
                "band_index": band_index,
                "coefficient_count": len(profile["coefficients_db"]),
                "maximum_absolute_coefficient_db": max(
                    abs(value) for value in profile["coefficients_db"]
                ),
                "shape_sha256": sha256_bytes(
                    profile["shape_db"].astype("<f8").tobytes()
                ),
            }
        )
    control_shape = float(
        np.mean(
            np.abs(
                diagnostic.spectral_shape(observed) - diagnostic.spectral_shape(control)
            )
        )
    )
    candidate_shape = float(
        np.mean(
            np.abs(
                diagnostic.spectral_shape(observed)
                - diagnostic.spectral_shape(candidate)
            )
        )
    )
    control_autocorrelation = float(
        np.mean(
            np.abs(
                diagnostic.autocorrelation_profile(observed)
                - diagnostic.autocorrelation_profile(control)
            )
        )
    )
    candidate_autocorrelation = float(
        np.mean(
            np.abs(
                diagnostic.autocorrelation_profile(observed)
                - diagnostic.autocorrelation_profile(candidate)
            )
        )
    )
    checks = {
        "all_bands_have_bounded_profiles": len(profiles)
        == len(execution.protocol.BAND_EDGES_HZ) - 1
        and all(
            item["coefficient_count"] <= COLOR_DCT_COEFFICIENTS
            and item["maximum_absolute_coefficient_db"] <= COLOR_COEFFICIENT_LIMIT_DB
            for item in profiles
        ),
        "colored_shape_ratio": candidate_shape / max(control_shape, 1.0e-300) <= 0.75,
        "colored_autocorrelation_ratio": candidate_autocorrelation
        / max(control_autocorrelation, 1.0e-300)
        <= 0.85,
    }
    if not all(checks.values()):
        raise PreregistrationError(
            "colored-residual fixture failed: "
            f"{checks}, shape={control_shape}/{candidate_shape}, "
            f"autocorrelation={control_autocorrelation}/{candidate_autocorrelation}"
        )
    return {
        "observed_sha256": sha256_bytes(observed.astype("<f8").tobytes()),
        "control_sha256": sha256_bytes(control.astype("<f8").tobytes()),
        "candidate_sha256": sha256_bytes(candidate.astype("<f8").tobytes()),
        "profiles": profiles,
        "control_spectral_shape_mae_db": control_shape,
        "candidate_spectral_shape_mae_db": candidate_shape,
        "spectral_shape_error_ratio": candidate_shape / max(control_shape, 1.0e-300),
        "control_autocorrelation_mae": control_autocorrelation,
        "candidate_autocorrelation_mae": candidate_autocorrelation,
        "autocorrelation_error_ratio": candidate_autocorrelation
        / max(control_autocorrelation, 1.0e-300),
        "checks": checks,
    }


def read_exact_json(
    root: Path, path: Path, label: str, expected_sha256: str
) -> tuple[bytes, dict[str, Any]]:
    resolved = external_file(root, path, label)
    data = resolved.read_bytes()
    observed = sha256_bytes(data)
    if observed != expected_sha256:
        raise PreregistrationError(f"{label} hash changed: {observed}")
    try:
        parsed = json.loads(data)
    except json.JSONDecodeError as error:
        raise PreregistrationError(f"parse {label}: {error}") from error
    return data, parsed


def validate_sources(root: Path) -> list[dict[str, Any]]:
    result = []
    for reference in source_references():
        path = (root / reference["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(root)
            or sha256_file(path) != reference["sha256"]
        ):
            raise PreregistrationError(f"source changed: {reference['path']}")
        result.append({**reference, "bytes": path.stat().st_size})
    return result


def preflight(
    root: Path,
    runner_sha256: str,
    manifest_path: Path,
    parent_manifest_path: Path,
    parent_preflight_path: Path,
    diagnostic_path: Path,
) -> dict[str, Any]:
    manifest_bytes, manifest = read_exact_json(
        root,
        manifest_path,
        "successor manifest",
        sha256_bytes(canonical_json(expected_manifest(runner_sha256))),
    )
    if manifest != expected_manifest(runner_sha256):
        raise PreregistrationError("successor manifest content changed")
    parent_manifest_bytes, parent_manifest = read_exact_json(
        root,
        parent_manifest_path,
        "parent discovery manifest",
        PARENT_DISCOVERY_MANIFEST_SHA256,
    )
    parent_preflight_bytes, parent_preflight = read_exact_json(
        root,
        parent_preflight_path,
        "parent discovery preflight",
        PARENT_DISCOVERY_PREFLIGHT_SHA256,
    )
    diagnostic_bytes, diagnostic_report = read_exact_json(
        root,
        diagnostic_path,
        "residual diagnostic",
        DIAGNOSTIC_REPORT_SHA256,
    )
    if (
        parent_manifest.get("official_roster", {}).get("repository_commit")
        != OFFICIAL_ROSTER_COMMIT
        or parent_manifest.get("official_roster", {}).get("object_names_sha256")
        != OFFICIAL_ROSTER_SHA256
        or parent_preflight.get("decision")
        != "RealImpactMetalBatchRolesAndDiscoveryInputsFrozen"
        or parent_preflight.get("network_requests") != 0
    ):
        raise PreregistrationError("parent roster/discovery lineage changed")
    diagnostic_decision = diagnostic_report.get("diagnostic", {})
    if (
        diagnostic_report.get("decision")
        != "RealImpactResidualRepresentationHypothesisSupported"
        or diagnostic_report.get("runner_sha256") != DIAGNOSTIC_RUNNER_SHA256
        or diagnostic_decision.get("leading_hypothesis") != "within_band_coloration"
        or diagnostic_decision.get("supported", {}).get("temporal_modulation")
        is not False
        or diagnostic_report.get("network_requests") != 0
        or diagnostic_report.get("new_member_payload_bytes_read") != 0
        or diagnostic_report.get("shadow_payload_bytes_read") != 0
        or diagnostic_report.get("quality_domain_or_runtime_admission") is not False
    ):
        raise PreregistrationError("diagnostic decision or access boundary changed")
    fixture = fixture_report()
    return {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "RealImpactColoredResidualSuccessorProtocolFrozen",
        "claim": manifest["claim"],
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "sources": validate_sources(root),
        "lineage": {
            "parent_discovery_manifest_sha256": sha256_bytes(parent_manifest_bytes),
            "parent_discovery_preflight_sha256": sha256_bytes(parent_preflight_bytes),
            "diagnostic_report_sha256": sha256_bytes(diagnostic_bytes),
        },
        "partition": manifest["partition"],
        "fresh_objects": manifest["fresh_objects"],
        "shadow_objects": manifest["shadow_objects"],
        "candidate": manifest["candidate"],
        "metrics": manifest["metrics"],
        "selection_gates": manifest["selection_gates"],
        "fixture": fixture,
        "stage_plan": manifest["stage_plan"],
        "access": manifest["access"],
        "network_requests": 0,
        "new_member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            "freeze this repeated report, then discover only the three fresh "
            "archive identities; do not read member payload or shadow audio"
        ),
    }


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    output = external_output(root, arguments.output)
    runner_sha256 = sha256_file(Path(__file__).resolve())
    if arguments.stage == "manifest":
        if any(
            value is not None
            for value in [
                arguments.manifest,
                arguments.parent_manifest,
                arguments.parent_preflight,
                arguments.diagnostic,
            ]
        ):
            raise PreregistrationError("manifest stage accepts only --output")
        data = canonical_json(expected_manifest(runner_sha256))
        output.write_bytes(data)
        print(f"colored residual successor manifest: {output}")
        print(f"manifest sha256: {sha256_bytes(data)}")
        return 0
    if any(
        value is None
        for value in [
            arguments.manifest,
            arguments.parent_manifest,
            arguments.parent_preflight,
            arguments.diagnostic,
        ]
    ):
        raise PreregistrationError("preflight requires all frozen lineage inputs")
    report = preflight(
        root,
        runner_sha256,
        arguments.manifest,
        arguments.parent_manifest,
        arguments.parent_preflight,
        arguments.diagnostic,
    )
    report_bytes = canonical_json(report)
    output.write_bytes(report_bytes)
    print(f"colored residual successor preflight: {output}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("new member payload bytes read: 0")
    print("shadow payload bytes read: 0")
    print("quality/domain/runtime admission: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
