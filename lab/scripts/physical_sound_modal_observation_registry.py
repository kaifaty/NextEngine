#!/usr/bin/env python3
"""Build a deterministic, report-only physical-sound modal registry."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import tempfile
from pathlib import Path
from typing import Any

MANIFEST_SCHEMA = "nextengine.experimental-modal-observation-registry.manifest.v0"
PREFLIGHT_SCHEMA = (
    "nextengine.experimental-modal-observation-registry-preflight.report.v0"
)
REGISTRY_SCHEMA = "nextengine.experimental-modal-observation-registry.v0"
ENTRY_SCHEMA = "nextengine.experimental-modal-observation.v0"
BUILD_REPORT_SCHEMA = (
    "nextengine.experimental-modal-observation-registry-build.report.v0"
)
STUDY_ID = "physical-sound-modal-observation-registry"
REVISION = "iron-skillet-mortar-report-only-seed-v0"
SAMPLE_RATE_HZ = 48_000
LISTENER_OUTPUT_INDICES = list(range(15))

INPUTS = [
    {
        "entry_id": "realimpact-17-iron-skillet-impact-000",
        "path": (
            "../ps2-realimpact-iron-skillet-broadband-v2-counterfactual-v1/"
            "analysis-a/report.json"
        ),
        "sha256": ("f2fb359faaccb36544203780f07cc1d359c5d529be7f7f0d533ae39bd50b6a73"),
        "bytes": 924_196,
        "report_schema": (
            "nextengine.experimental-realimpact-broadband-v2-counterfactual.report.v1"
        ),
        "producer_runner_sha256": (
            "3f1c07b501dec97645cc0f1ad2fec5b3960312eacde88a291b64cc740a056aa5"
        ),
        "producer_manifest_sha256": (
            "876e125dcef4a8c38e5bf7630b1115013cc02deb0c1dd5303d5bed244a5aa4bf"
        ),
        "required_decision": "ExistingIronBroadbandV2MethodTransferSupported",
        "object": {
            "dataset_object_id": "17_IronSkillet",
            "impact_ordinal": 0,
            "role": "opened_development_counterfactual_only",
        },
        "listener_output_indices": LISTENER_OUTPUT_INDICES,
        "expected_counts": {
            "regions": 31,
            "analysis_bins": 91,
            "raw_estimates": 59,
            "duplicate_estimates_removed": 28,
            "preprune_clusters": 31,
            "retained_modes": 30,
            "pruned_modes": 1,
        },
    },
    {
        "entry_id": "realimpact-43-iron-mortar-impact-000",
        "path": (
            "../ps2-realimpact-iron-mortar-broadband-holdout-v1/analysis-a/report.json"
        ),
        "sha256": ("64bc63aa0cd02cfe9ad3959670102e883893f7d53a1427fddb5f660a8951d0cc"),
        "bytes": 696_263,
        "report_schema": (
            "nextengine.experimental-realimpact-broadband-holdout-execution-"
            "analysis.report.v1"
        ),
        "producer_runner_sha256": (
            "53f637789b2fa2f665f3f6eee1bddc8b9cfc1af6ea91dd70510d792ab6b7a8e9"
        ),
        "producer_manifest_sha256": (
            "6915846dc63687ca6f9982a90eb43866203c8c79114c5a70b80fd4156737caaf"
        ),
        "required_decision": "BroadbandIndependentHoldoutMethodTransferSupported",
        "object": {
            "dataset_object_id": "43_IronMortar",
            "impact_ordinal": 0,
            "role": "independent_method_holdout",
        },
        "listener_output_indices": LISTENER_OUTPUT_INDICES,
        "expected_counts": {
            "regions": 26,
            "analysis_bins": 77,
            "raw_estimates": 27,
            "duplicate_estimates_removed": 12,
            "preprune_clusters": 15,
            "retained_modes": 13,
            "pruned_modes": 2,
        },
    },
]


class RegistryError(RuntimeError):
    """A frozen registry input or invariant changed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--stage", required=True, choices=["manifest", "preflight", "build"]
    )
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--preflight", type=Path)
    return parser.parse_args()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def expected_manifest(runner_sha256: str) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "runner": {
            "path": "lab/scripts/physical_sound_modal_observation_registry.py",
            "sha256": runner_sha256,
        },
        "entry_order": [item["entry_id"] for item in INPUTS],
        "inputs": INPUTS,
        "output": {
            "registry_schema": REGISTRY_SCHEMA,
            "entry_schema": ENTRY_SCHEMA,
            "report_schema": BUILD_REPORT_SCHEMA,
            "record_kind": "report_only_experimental_observation",
            "public_contract": False,
            "runtime_consumer_allowed": False,
        },
        "admission_policy": {
            "method_state": "supported_by_frozen_source_gate",
            "quality_admission_state": (
                "disabled_no_validated_absolute_residual_threshold"
            ),
            "domain_admission_state": "disabled_no_exact_domain_evidence",
            "runtime_admission_state": "disabled_proposed_report_only_experiment",
            "fallback_state": "authored_clip_required",
        },
        "access_policy": {
            "source_report_json_only": True,
            "audio_or_waveform_payload_allowed": False,
            "network_allowed": False,
            "physics_solver_allowed": False,
            "planter_payload_allowed": False,
            "threshold_or_variant_tuning_allowed": False,
        },
        "stop_rule": (
            "stop if source identity, report schema, method gate, counts, modal "
            "shape, spatial partition identity, residual fields or zero-credit "
            "state differ; never convert method support into quality, domain or "
            "runtime admission"
        ),
    }


def load_manifest(path: Path, runner_sha256: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    if len(data) > 128 * 1024:
        raise RegistryError("registry manifest exceeds 128 KiB")
    try:
        manifest = json.loads(data)
    except json.JSONDecodeError as error:
        raise RegistryError(f"parse registry manifest: {error}") from error
    if manifest != expected_manifest(runner_sha256):
        raise RegistryError("registry manifest changed")
    return data, manifest


def resolve_input(base: Path, reference: dict[str, Any]) -> Path:
    path = (base / reference["path"]).resolve(strict=True)
    store = base.parent.resolve(strict=True)
    if not path.is_file() or not path.is_relative_to(store):
        raise RegistryError(f"input escapes physical-sound store: {reference['path']}")
    if path.stat().st_size != reference["bytes"]:
        raise RegistryError(f"input byte length changed: {reference['entry_id']}")
    if sha256_file(path) != reference["sha256"]:
        raise RegistryError(f"input hash changed: {reference['entry_id']}")
    return path


def write_new_file(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        raise RegistryError(f"refusing to replace existing output: {path}")
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


def publish_directory(output: Path, files: dict[str, bytes]) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise RegistryError(f"refusing to replace existing output: {output}")
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    try:
        for name, data in files.items():
            path = staging / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        os.replace(staging, output)
    finally:
        if staging.exists():
            shutil.rmtree(staging)


def expected_preflight(
    manifest_bytes: bytes, runner_sha256: str, inputs: list[dict[str, Any]]
) -> dict[str, Any]:
    return {
        "schema": PREFLIGHT_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "decision": "ModalObservationRegistryV0InputsFrozen",
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": runner_sha256,
        "entry_order": [item["entry_id"] for item in inputs],
        "inputs": [
            {
                "entry_id": item["entry_id"],
                "path": item["path"],
                "sha256": item["sha256"],
                "bytes": item["bytes"],
                "report_payload_parsed": False,
            }
            for item in inputs
        ],
        "report_bytes_hashed": sum(item["bytes"] for item in inputs),
        "network_requests": 0,
        "audio_or_waveform_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
    }


def load_json_object(path: Path, maximum_bytes: int) -> dict[str, Any]:
    data = path.read_bytes()
    if len(data) > maximum_bytes:
        raise RegistryError(f"JSON input exceeds frozen size: {path}")
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise RegistryError(f"parse JSON input {path}: {error}") from error
    if not isinstance(value, dict):
        raise RegistryError(f"JSON input is not an object: {path}")
    return value


def require_finite_positive(value: Any, label: str) -> float:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise RegistryError(f"{label} is not numeric")
    result = float(value)
    if not math.isfinite(result) or result <= 0.0:
        raise RegistryError(f"{label} is not finite and positive")
    return result


def compact_spatial_partitions(report: dict[str, Any]) -> dict[str, Any]:
    partitions = report.get("spatial_partitions")
    if not isinstance(partitions, dict) or set(partitions) != {
        "even_microphones",
        "odd_microphones",
    }:
        raise RegistryError("spatial partition shape changed")
    result = {}
    all_indices = []
    for name in ["even_microphones", "odd_microphones"]:
        partition = partitions[name]
        indices = partition.get("output_indices")
        matching = partition.get("retained_full_output_matching")
        if not isinstance(indices, list) or not isinstance(matching, dict):
            raise RegistryError(f"spatial partition fields changed: {name}")
        all_indices.extend(indices)
        result[name] = {
            "output_indices": indices,
            "raw_estimate_count": partition.get("raw_estimate_count"),
            "retained_full_output_matching": matching,
        }
    if sorted(all_indices) != LISTENER_OUTPUT_INDICES:
        raise RegistryError("listener output identity changed")
    return result


def validate_cluster(cluster: dict[str, Any], listener_count: int) -> None:
    representative = cluster.get("representative")
    amplitudes = cluster.get("complex_amplitudes")
    members = cluster.get("members")
    if not isinstance(representative, dict):
        raise RegistryError("cluster representative missing")
    if not isinstance(amplitudes, list) or len(amplitudes) != listener_count:
        raise RegistryError("cluster amplitude/listener shape changed")
    if not isinstance(members, list) or len(members) != cluster.get("member_count"):
        raise RegistryError("cluster member provenance changed")
    require_finite_positive(representative.get("frequency_hz"), "mode frequency")
    require_finite_positive(representative.get("decay_per_second"), "mode decay")
    for amplitude in amplitudes:
        if not isinstance(amplitude, dict):
            raise RegistryError("complex amplitude is not an object")
        for field in ["real", "imaginary", "magnitude", "phase_radians"]:
            value = amplitude.get(field)
            if not isinstance(value, (int, float)) or not math.isfinite(float(value)):
                raise RegistryError(f"complex amplitude {field} is not finite")


def build_entry(reference: dict[str, Any], report: dict[str, Any]) -> dict[str, Any]:
    exact_fields = {
        "schema": reference["report_schema"],
        "runner_sha256": reference["producer_runner_sha256"],
        "manifest_sha256": reference["producer_manifest_sha256"],
        "decision": reference["required_decision"],
        **reference["object"],
    }
    for field, expected in exact_fields.items():
        if report.get(field) != expected:
            raise RegistryError(f"source report identity changed: {field}")
    if report.get("quality_admission_or_runtime_credit") is not False:
        raise RegistryError("source report gained quality or runtime credit")
    if report.get("opened_region_ranking_used") is not False:
        raise RegistryError("source report used opened-region ranking")
    if report.get("all_discovered_regions_analyzed") is not True:
        raise RegistryError("source report omitted discovered regions")
    gate = report.get("gate")
    if not isinstance(gate, dict) or gate.get("passed") is not True:
        raise RegistryError("source method gate is not supported")
    checks = gate.get("checks")
    if (
        not isinstance(checks, list)
        or not checks
        or not all(
            isinstance(check, dict) and check.get("passed") is True for check in checks
        )
    ):
        raise RegistryError("source method gate checks changed")

    discovery = report.get("discovery")
    clusters = report.get("fitted_clusters")
    if not isinstance(discovery, dict) or not isinstance(clusters, list):
        raise RegistryError("source extraction fields changed")
    expected = reference["expected_counts"]
    observed = {
        "regions": len(discovery.get("regions", [])),
        "analysis_bins": len(discovery.get("analysis_bins", [])),
        "raw_estimates": report.get("raw_estimate_count"),
        "duplicate_estimates_removed": report.get("duplicate_estimates_removed"),
        "preprune_clusters": len(clusters),
        "retained_modes": report.get("retained_mode_count"),
        "pruned_modes": report.get("pruned_mode_count"),
    }
    if observed != expected:
        raise RegistryError(
            f"source extraction counts changed: {reference['entry_id']}"
        )
    if [cluster.get("cluster_index") for cluster in clusters] != list(
        range(len(clusters))
    ):
        raise RegistryError("cluster order or identity changed")
    for cluster in clusters:
        validate_cluster(cluster, len(LISTENER_OUTPUT_INDICES))
    retained = [cluster for cluster in clusters if cluster.get("retained") is True]
    pruned = [cluster for cluster in clusters if cluster.get("retained") is False]
    if (
        len(retained) != expected["retained_modes"]
        or len(pruned) != expected["pruned_modes"]
    ):
        raise RegistryError("cluster retention state changed")

    spatial = compact_spatial_partitions(report)
    reconstruction = report.get("reconstruction")
    prediction = report.get("prediction_control")
    if not isinstance(reconstruction, dict) or not isinstance(prediction, dict):
        raise RegistryError("absolute residual fields missing")
    require_finite_positive(reconstruction.get("candidate_nrmse"), "candidate NRMSE")
    windows = prediction.get("windows")
    if not isinstance(windows, list) or len(windows) != 2:
        raise RegistryError("prediction residual windows changed")
    for index, window in enumerate(windows):
        if not isinstance(window, dict):
            raise RegistryError("prediction window is not an object")
        require_finite_positive(
            window.get("damped_nrmse"), f"prediction window {index} damped NRMSE"
        )

    access = {
        "network_requests": report.get("network_requests", 0),
        "additional_payload_bytes_read": report.get(
            "additional_payload_bytes_read",
            report.get("additional_audio_payload_bytes_read", 0),
        ),
        "planter_payload_bytes_read": report.get("planter_payload_bytes_read"),
        "physics_solver_runs": report.get("physics_solver_runs"),
        "opened_region_ranking_used": report.get("opened_region_ranking_used"),
        "all_discovered_regions_analyzed": report.get(
            "all_discovered_regions_analyzed"
        ),
    }
    if access != {
        "network_requests": 0,
        "additional_payload_bytes_read": 0,
        "planter_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "opened_region_ranking_used": False,
        "all_discovered_regions_analyzed": True,
    }:
        raise RegistryError("source access boundary changed")

    observation_samples = reconstruction.get(
        "observation_samples", report.get("observation_samples")
    )
    if observation_samples != 60_000:
        raise RegistryError("observation sample count changed")
    return {
        "schema": ENTRY_SCHEMA,
        "entry_id": reference["entry_id"],
        "record_kind": "report_only_experimental_observation",
        "source": {
            "report_path": reference["path"],
            "report_sha256": reference["sha256"],
            "report_schema": report["schema"],
            "producer_runner_sha256": report["runner_sha256"],
            "producer_manifest_sha256": report["manifest_sha256"],
            "decision": report["decision"],
            "study_id": report.get("study_id"),
            "revision": report.get("revision"),
            "claim": report.get("claim"),
        },
        "observation_identity": {
            **reference["object"],
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "observation_samples": observation_samples,
            "onset_sample": report.get("onset_sample"),
            "listener_group": {
                "identity_kind": "ordered_source_output_indices",
                "output_indices": LISTENER_OUTPUT_INDICES,
                "physical_coordinates_state": "not_available_in_source_report",
            },
            "input_slice_sha256": report.get("input_slice_sha256"),
            "gabor_coefficients_sha256": report.get("gabor_coefficients_sha256"),
            "decoded_observation_sha256": report.get("decoded_observation_sha256"),
        },
        "extraction_provenance": {
            "discovery": discovery,
            "counts": observed,
            "scale_invariance": report.get("scale_invariance"),
            "method_gate": gate,
            "deterministic_work_envelope": report.get("deterministic_work_envelope"),
        },
        "modal_fit": {
            "retained_modes": retained,
            "pruned_modes": pruned,
        },
        "spatial_replication": spatial,
        "absolute_residual_evaluation": {
            "reconstruction": reconstruction,
            "prediction_control": prediction,
        },
        "admission": {
            "method_state": "supported_by_frozen_source_gate",
            "quality_admission_state": (
                "disabled_no_validated_absolute_residual_threshold"
            ),
            "domain_admission_state": "disabled_no_exact_domain_evidence",
            "runtime_admission_state": ("disabled_proposed_report_only_experiment"),
            "fallback_state": "authored_clip_required",
        },
        "access": access,
    }


def run_manifest(output: Path, runner_sha256: str) -> None:
    write_new_file(output, canonical_json(expected_manifest(runner_sha256)))


def run_preflight(manifest_path: Path, output: Path, runner_sha256: str) -> None:
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    for reference in manifest["inputs"]:
        resolve_input(manifest_path.parent, reference)
    report = expected_preflight(manifest_bytes, runner_sha256, manifest["inputs"])
    publish_directory(output, {"report.json": canonical_json(report)})


def run_build(
    manifest_path: Path,
    preflight_path: Path,
    output: Path,
    runner_sha256: str,
) -> None:
    manifest_bytes, manifest = load_manifest(manifest_path, runner_sha256)
    preflight_bytes = preflight_path.read_bytes()
    try:
        preflight = json.loads(preflight_bytes)
    except json.JSONDecodeError as error:
        raise RegistryError(f"parse registry preflight: {error}") from error
    expected = expected_preflight(manifest_bytes, runner_sha256, manifest["inputs"])
    if preflight != expected:
        raise RegistryError("registry preflight changed")

    entries = []
    for reference in manifest["inputs"]:
        path = resolve_input(manifest_path.parent, reference)
        report = load_json_object(path, reference["bytes"])
        entries.append(build_entry(reference, report))
    registry = {
        "schema": REGISTRY_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "record_kind": "report_only_experimental_observation_registry",
        "builder": {
            "runner_sha256": runner_sha256,
            "manifest_sha256": sha256_bytes(manifest_bytes),
            "preflight_sha256": sha256_bytes(preflight_bytes),
        },
        "entry_order": manifest["entry_order"],
        "entries": entries,
        "public_contract": False,
        "runtime_consumer_allowed": False,
    }
    registry_bytes = canonical_json(registry)
    summaries = []
    for entry in entries:
        modes = entry["modal_fit"]["retained_modes"]
        frequencies = [mode["representative"]["frequency_hz"] for mode in modes]
        prediction_windows = entry["absolute_residual_evaluation"][
            "prediction_control"
        ]["windows"]
        summaries.append(
            {
                "entry_id": entry["entry_id"],
                "retained_mode_count": len(modes),
                "pruned_mode_count": len(entry["modal_fit"]["pruned_modes"]),
                "frequency_range_hz": [min(frequencies), max(frequencies)],
                "full_candidate_nrmse": entry["absolute_residual_evaluation"][
                    "reconstruction"
                ]["candidate_nrmse"],
                "predictive_damped_nrmse": [
                    window["damped_nrmse"] for window in prediction_windows
                ],
                "admission": entry["admission"],
            }
        )
    report = {
        "schema": BUILD_REPORT_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "decision": "ModalObservationRegistryV0BuiltFallbackOnly",
        "runner_sha256": runner_sha256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "preflight_sha256": sha256_bytes(preflight_bytes),
        "registry_sha256": sha256_bytes(registry_bytes),
        "registry_bytes": len(registry_bytes),
        "entry_count": len(entries),
        "entries": summaries,
        "all_source_method_gates_supported": True,
        "all_quality_domain_runtime_admissions_disabled": True,
        "all_fallbacks_required": True,
        "network_requests": 0,
        "audio_or_waveform_payload_bytes_read": 0,
        "physics_solver_runs": 0,
        "planter_payload_bytes_read": 0,
        "next_action": (
            "freeze an object-grouped batch manifest and evaluate explicit "
            "transient/residual candidates without per-object human admission"
        ),
    }
    publish_directory(
        output,
        {
            "registry.json": registry_bytes,
            "report.json": canonical_json(report),
        },
    )


def main() -> None:
    arguments = parse_arguments()
    runner_sha256 = sha256_file(Path(__file__).resolve())
    if arguments.stage == "manifest":
        if arguments.manifest is not None or arguments.preflight is not None:
            raise RegistryError("manifest stage accepts only --output")
        run_manifest(arguments.output, runner_sha256)
        return
    if arguments.manifest is None:
        raise RegistryError(f"{arguments.stage} stage requires --manifest")
    if arguments.stage == "preflight":
        if arguments.preflight is not None:
            raise RegistryError("preflight stage does not accept --preflight")
        run_preflight(arguments.manifest, arguments.output, runner_sha256)
        return
    if arguments.preflight is None:
        raise RegistryError("build stage requires --preflight")
    run_build(
        arguments.manifest,
        arguments.preflight,
        arguments.output,
        runner_sha256,
    )


if __name__ == "__main__":
    main()
