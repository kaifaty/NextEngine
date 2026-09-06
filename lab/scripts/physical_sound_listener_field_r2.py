#!/usr/bin/env python3
"""Preregister and train the frozen R2 fixed-impact listener-field pilot."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import struct
import sys
import wave
from pathlib import Path
from typing import Any

import mlflow
import numpy as np
import torch
from torch import nn

MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2.manifest.v1"
)
PREFLIGHT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-preflight.report.v1"
)
TRAINING_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-training.report.v1"
)
MLFLOW_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-mlflow-lineage.v1"
)
PROJECTION_SCHEMA = (
    "nextengine.experimental-physical-sound-neural-data-plane.projection.v2"
)
BASELINE_SCHEMA = (
    "nextengine.experimental-physical-sound-transfer-field-baseline.report.v1"
)
BASELINE_DECISION = "FrozenTransferControlsAvailable"
BASELINE_METRIC_PROFILE = "transfer-listener-field-r1-v1"
TASK_SCOPE = "exact_object_few_shot_impact_listener_field"
TRANSFER_SEMANTICS = "force_deconvolved_transfer_response"
REVISION = "green-goblet-fixed-impact-listener-field-r2-v2"
SEED = 2_608_300_101
EPOCHS = 8_000
LEARNING_RATE = 0.01
WEIGHT_DECAY = 1.0e-6
SMOOTHNESS_WEIGHT = 1.0e-4
HIDDEN_WIDTH = 32
HIDDEN_LAYERS = 2
GRID_POINTS = 65
DTYPE = torch.float64
DB_FLOOR = -240.0
CANDIDATES = (
    {
        "candidate_id": "listener_tanh_rank4_modal_latent_v1",
        "rank": 4,
        "representation": "mean_plus_rank4_listener_latent",
    },
    {
        "candidate_id": "listener_tanh_rank7_modal_plus_residual_v1",
        "rank": 7,
        "representation": "mean_plus_full_centered_rank_listener_latent",
    },
)


class ListenerFieldError(RuntimeError):
    """The frozen listener-field boundary or training run failed."""


class ListenerField(nn.Module):
    """Small coordinate network mapping one listener axis to latent gains."""

    def __init__(self, output_count: int) -> None:
        super().__init__()
        layers: list[nn.Module] = []
        input_count = 1
        for _ in range(HIDDEN_LAYERS):
            layers.extend((nn.Linear(input_count, HIDDEN_WIDTH), nn.Tanh()))
            input_count = HIDDEN_WIDTH
        layers.append(nn.Linear(input_count, output_count))
        self.network = nn.Sequential(*layers)

    def forward(self, coordinate: torch.Tensor) -> torch.Tensor:
        return self.network(coordinate)


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["preflight", "train"])
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
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


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise ListenerFieldError(f"{label} must be an external file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise ListenerFieldError(f"output must be a new external directory: {resolved}")
    resolved.mkdir()
    return resolved


def read_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise ListenerFieldError(f"parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise ListenerFieldError(f"{label} must contain one JSON object")
    return data, value


def require_file_ref(value: Any, label: str) -> dict[str, str]:
    if (
        not isinstance(value, dict)
        or set(value) != {"path", "sha256"}
        or not isinstance(value["path"], str)
        or not isinstance(value["sha256"], str)
        or len(value["sha256"]) != 64
        or any(character not in "0123456789abcdef" for character in value["sha256"])
    ):
        raise ListenerFieldError(f"invalid {label} file reference")
    return value


def resolve_ref(
    root: Path, manifest_directory: Path, value: Any, label: str
) -> tuple[Path, bytes]:
    reference = require_file_ref(value, label)
    unresolved = Path(reference["path"])
    path = unresolved if unresolved.is_absolute() else manifest_directory / unresolved
    resolved = external_file(root, path, label)
    data = resolved.read_bytes()
    actual = sha256_bytes(data)
    if actual != reference["sha256"]:
        raise ListenerFieldError(
            f"{label} hash mismatch: expected {reference['sha256']}, got {actual}"
        )
    return resolved, data


def source_path(root: Path) -> Path:
    path = Path(__file__).resolve(strict=True)
    if not path.is_relative_to(root):
        raise ListenerFieldError("listener-field source must remain repository-local")
    return path


def environment_profile() -> dict[str, Any]:
    return {
        "python": platform.python_version(),
        "numpy": np.__version__,
        "torch": torch.__version__,
        "mlflow": mlflow.__version__,
        "device": "cpu",
        "dtype": "float64",
        "torch_threads": 1,
        "deterministic_algorithms": True,
        "cuda_available_but_unused": torch.cuda.is_available(),
    }


def candidate_profile() -> dict[str, Any]:
    return {
        "profile_id": "listener-field-r2-tanh-latent-v1",
        "seed": SEED,
        "epochs": EPOCHS,
        "learning_rate": LEARNING_RATE,
        "weight_decay": WEIGHT_DECAY,
        "smoothness_weight": SMOOTHNESS_WEIGHT,
        "hidden_width": HIDDEN_WIDTH,
        "hidden_layers": HIDDEN_LAYERS,
        "activation": "tanh",
        "coordinate": "listener_z_metres_normalized_by_context_max_abs",
        "temporal_basis": "canonical_sign_centered_context_gram_eigendecomposition",
        "latent_standardization": "per_component_population_std_floor_1e-12",
        "smoothness_grid_points": GRID_POINTS,
        "optimizer": "torch_adam_fixed_steps",
        "cooker": "reject_nonfinite_or_abs_ge_1_then_round_clamp_pcm16",
        "candidate_ladder": list(CANDIDATES),
        "selection": (
            "smallest_rank_candidate_strictly_below_both_r1_controls_on_every_"
            "frozen_primary_endpoint_else_reject_listener_field"
        ),
        "query_audio_available_to_trainer": False,
        "method_holdout_or_shadow_available": False,
    }


def validate_environment(manifest: dict[str, Any]) -> None:
    if manifest.get("environment") != environment_profile():
        raise ListenerFieldError("listener-field environment differs from manifest")
    if manifest.get("candidate_profile") != candidate_profile():
        raise ListenerFieldError("listener-field candidate profile differs from manifest")


def validate_manifest_shape(manifest: dict[str, Any]) -> None:
    expected = {
        "schema",
        "revision",
        "runner_sha256",
        "projection",
        "baseline_report",
        "context_bindings",
        "query_row_ids",
        "environment",
        "candidate_profile",
        "mlflow_experiment",
    }
    if set(manifest) != expected:
        raise ListenerFieldError("listener-field manifest fields changed")
    if manifest.get("schema") != MANIFEST_SCHEMA or manifest.get("revision") != REVISION:
        raise ListenerFieldError("listener-field manifest schema or revision changed")
    if manifest.get("runner_sha256") != sha256_file(Path(__file__).resolve(strict=True)):
        raise ListenerFieldError("listener-field runner source hash changed")
    if manifest.get("mlflow_experiment") != "nextengine-physical-sound-r2":
        raise ListenerFieldError("listener-field MLflow experiment changed")
    validate_environment(manifest)
    bindings = manifest.get("context_bindings")
    query_ids = manifest.get("query_row_ids")
    if not isinstance(bindings, list) or not isinstance(query_ids, list):
        raise ListenerFieldError("listener-field bindings and query IDs must be arrays")
    binding_ids = [binding.get("row_id") for binding in bindings]
    if binding_ids != sorted(binding_ids) or len(set(binding_ids)) != len(binding_ids):
        raise ListenerFieldError("context bindings must be strictly sorted and unique")
    if query_ids != sorted(query_ids) or len(set(query_ids)) != len(query_ids):
        raise ListenerFieldError("query row IDs must be strictly sorted and unique")
    if len(bindings) < 3 or not query_ids:
        raise ListenerFieldError("listener field requires context and query coverage")
    for binding in bindings:
        if set(binding) != {"row_id", "audio"} or not isinstance(
            binding["row_id"], str
        ):
            raise ListenerFieldError("invalid listener-field context binding")
        require_file_ref(binding["audio"], "context audio")


def validate_projection_and_baseline(
    root: Path, manifest_path: Path, manifest: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any], bytes, bytes]:
    directory = manifest_path.parent
    _, projection_bytes = resolve_ref(
        root, directory, manifest["projection"], "listener-field projection"
    )
    _, baseline_bytes = resolve_ref(
        root, directory, manifest["baseline_report"], "listener-field baseline report"
    )
    projection = json.loads(projection_bytes)
    baseline = json.loads(baseline_bytes)
    if (
        projection.get("schema") != PROJECTION_SCHEMA
        or projection.get("task_scope") != TASK_SCOPE
        or "development" not in projection.get("role_scope", [])
    ):
        raise ListenerFieldError("listener-field projection semantics changed")
    if (
        baseline.get("schema") != BASELINE_SCHEMA
        or baseline.get("status") != "Validated"
        or baseline.get("decision") != BASELINE_DECISION
        or baseline.get("metric_profile", {}).get("id") != BASELINE_METRIC_PROFILE
        or baseline.get("admission_shadow_opened") is not False
    ):
        raise ListenerFieldError("listener-field baseline semantics changed")
    rows = {
        row["row_id"]: row
        for row in projection.get("rows", [])
        if row.get("split_role") == "development"
        and row.get("corpus_role") == "target"
        and row.get("audio_semantics") == TRANSFER_SEMANTICS
    }
    context_ids = [binding["row_id"] for binding in manifest["context_bindings"]]
    query_ids = manifest["query_row_ids"]
    expected_context = sorted(
        row_id for row_id, row in rows.items() if row.get("sample_role") == "context"
    )
    expected_query = sorted(
        row_id for row_id, row in rows.items() if row.get("sample_role") == "query"
    )
    if context_ids != expected_context or query_ids != expected_query:
        raise ListenerFieldError("manifest does not exactly bind projected context/query rows")
    baseline_query = sorted(row["row_id"] for row in baseline.get("rows", []))
    if baseline_query != expected_query:
        raise ListenerFieldError("baseline query identity differs from projection")
    impact = None
    coordinate_profile = None
    for row in rows.values():
        axes = row.get("axes", {})
        if axes.get("impact") is None or axes.get("listener") is None:
            raise ListenerFieldError("listener-field row lacks impact/listener coordinates")
        row_impact = axes["impact"]
        row_profile = axes["listener"].get("coordinate_profile")
        if impact is None:
            impact = row_impact
            coordinate_profile = row_profile
        elif row_impact != impact or row_profile != coordinate_profile:
            raise ListenerFieldError("listener-field rows do not share impact/profile")
    return projection, baseline, projection_bytes, baseline_bytes


def read_context(
    root: Path,
    manifest_path: Path,
    manifest: dict[str, Any],
    projection: dict[str, Any],
) -> tuple[np.ndarray, np.ndarray, int, list[dict[str, Any]]]:
    projected = {row["row_id"]: row for row in projection["rows"]}
    signals = []
    coordinates = []
    records = []
    sample_rate = None
    sample_count = None
    for binding in manifest["context_bindings"]:
        row = projected[binding["row_id"]]
        path, data = resolve_ref(
            root, manifest_path.parent, binding["audio"], "listener-field context WAV"
        )
        if (
            sha256_bytes(data) != row["audio"]["sha256"]
            or len(data) != row["audio"]["byte_count"]
        ):
            raise ListenerFieldError(f"context identity changed for {binding['row_id']}")
        with wave.open(str(path), "rb") as source:
            if (
                source.getnchannels() != 1
                or source.getsampwidth() != 2
                or source.getcomptype() != "NONE"
            ):
                raise ListenerFieldError("context WAV must be uncompressed mono PCM16")
            row_rate = source.getframerate()
            row_count = source.getnframes()
            payload = source.readframes(row_count)
        if len(payload) != row_count * 2:
            raise ListenerFieldError("context WAV payload is truncated")
        if sample_rate is None:
            sample_rate = row_rate
            sample_count = row_count
        elif row_rate != sample_rate or row_count != sample_count:
            raise ListenerFieldError("context WAV grid changed")
        signal = np.frombuffer(payload, dtype="<i2").astype(np.float64) / 32768.0
        if not np.isfinite(signal).all():
            raise ListenerFieldError("context WAV contains non-finite samples")
        point = row["axes"]["listener"]["point_metres"]
        signals.append(signal)
        coordinates.append(float(point[2]))
        records.append(
            {
                "row_id": binding["row_id"],
                "audio_sha256": sha256_bytes(data),
                "listener_point_metres": point,
            }
        )
    assert sample_rate is not None and sample_count is not None
    coordinate_array = np.asarray(coordinates, dtype=np.float64)
    if len(np.unique(coordinate_array)) != len(coordinate_array):
        raise ListenerFieldError("context listener coordinates are not unique")
    return np.stack(signals), coordinate_array, sample_rate, records


def query_coordinates(
    manifest: dict[str, Any], projection: dict[str, Any]
) -> tuple[np.ndarray, list[dict[str, Any]]]:
    rows = {row["row_id"]: row for row in projection["rows"]}
    coordinates = []
    records = []
    for row_id in manifest["query_row_ids"]:
        point = rows[row_id]["axes"]["listener"]["point_metres"]
        coordinates.append(float(point[2]))
        records.append({"row_id": row_id, "listener_point_metres": point})
    return np.asarray(coordinates, dtype=np.float64), records


def configure_determinism() -> None:
    os.environ["CUBLAS_WORKSPACE_CONFIG"] = ":4096:8"
    torch.use_deterministic_algorithms(True)
    torch.set_num_threads(1)
    torch.set_num_interop_threads(1)
    torch.manual_seed(SEED)
    np.random.seed(SEED & 0xFFFF_FFFF)


def canonical_latent_basis(
    signals: torch.Tensor, requested_rank: int
) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, int]:
    mean = signals.mean(dim=0)
    centered = signals - mean
    gram = centered @ centered.T
    eigenvalues, eigenvectors = torch.linalg.eigh(gram)
    order = torch.argsort(eigenvalues, descending=True)
    eigenvalues = eigenvalues[order]
    eigenvectors = eigenvectors[:, order]
    positive = eigenvalues > max(float(eigenvalues[0]) * 1.0e-12, 1.0e-18)
    available_rank = int(positive.sum().item())
    rank = min(requested_rank, available_rank)
    if rank < 1:
        raise ListenerFieldError("context listener field has no non-constant rank")
    eigenvalues = eigenvalues[:rank]
    eigenvectors = eigenvectors[:, :rank].clone()
    for column in range(rank):
        vector = eigenvectors[:, column]
        anchor = int(torch.argmax(torch.abs(vector)).item())
        if float(vector[anchor]) < 0.0:
            eigenvectors[:, column] *= -1.0
    singular = torch.sqrt(torch.clamp(eigenvalues, min=1.0e-24))
    coefficients = eigenvectors * singular
    basis = (eigenvectors.T @ centered) / singular[:, None]
    return mean, coefficients, basis, available_rank


def train_candidate(
    signals: np.ndarray,
    context_coordinate: np.ndarray,
    query_coordinate: np.ndarray,
    requested_rank: int,
) -> tuple[ListenerField, np.ndarray, dict[str, Any]]:
    torch.manual_seed(SEED + requested_rank)
    signal_tensor = torch.from_numpy(signals).to(dtype=DTYPE)
    mean, coefficients, basis, available_rank = canonical_latent_basis(
        signal_tensor, requested_rank
    )
    rank = coefficients.shape[1]
    scale = torch.std(coefficients, dim=0, correction=0).clamp_min(1.0e-12)
    target = coefficients / scale
    coordinate_scale = float(np.max(np.abs(context_coordinate)))
    if not math.isfinite(coordinate_scale) or coordinate_scale <= 0.0:
        raise ListenerFieldError("context coordinate scale is invalid")
    context = torch.from_numpy(context_coordinate[:, None] / coordinate_scale).to(
        dtype=DTYPE
    )
    query = torch.from_numpy(query_coordinate[:, None] / coordinate_scale).to(
        dtype=DTYPE
    )
    grid = torch.linspace(-1.0, 1.0, GRID_POINTS, dtype=DTYPE)[:, None]
    model = ListenerField(rank).to(dtype=DTYPE)
    optimizer = torch.optim.Adam(
        model.parameters(), lr=LEARNING_RATE, weight_decay=WEIGHT_DECAY
    )
    initial_loss = None
    final_data_loss = None
    final_smoothness = None
    for _ in range(EPOCHS):
        prediction = model(context)
        data_loss = torch.mean((prediction - target) ** 2)
        grid_prediction = model(grid)
        second = grid_prediction[2:] - 2.0 * grid_prediction[1:-1] + grid_prediction[:-2]
        smoothness = torch.mean(second**2)
        loss = data_loss + SMOOTHNESS_WEIGHT * smoothness
        if initial_loss is None:
            initial_loss = float(loss.detach())
        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        optimizer.step()
        final_data_loss = float(data_loss.detach())
        final_smoothness = float(smoothness.detach())
    with torch.no_grad():
        trained_context = model(context) * scale
        query_coefficients = model(query) * scale
        context_reconstruction = mean + trained_context @ basis
        query_prediction = mean + query_coefficients @ basis
    train_error = context_reconstruction - signal_tensor
    train_nrmse = torch.sqrt(torch.sum(train_error**2) / torch.sum(signal_tensor**2))
    training = {
        "requested_rank": requested_rank,
        "effective_rank": int(rank),
        "available_centered_rank": available_rank,
        "coordinate_scale_metres": coordinate_scale,
        "initial_objective": initial_loss,
        "final_latent_mse": final_data_loss,
        "final_smoothness_penalty": final_smoothness,
        "train_normalized_waveform_rmse_db": max(
            DB_FLOOR, 20.0 * math.log10(float(train_nrmse))
        ),
    }
    return model, query_prediction.numpy(force=True), training


def encode_wav(samples: np.ndarray, sample_rate: int) -> bytes:
    if samples.ndim != 1 or not np.isfinite(samples).all():
        raise ListenerFieldError("candidate prediction must be finite mono audio")
    peak = float(np.max(np.abs(samples)))
    if peak >= 1.0:
        raise ListenerFieldError(f"candidate prediction peak {peak} is not cookable")
    quantized = np.clip(np.rint(samples * 32768.0), -32768, 32767).astype("<i2")
    payload = quantized.tobytes()
    riff_size = 36 + len(payload)
    return b"".join(
        (
            b"RIFF",
            struct.pack("<I", riff_size),
            b"WAVEfmt ",
            struct.pack("<IHHIIHH", 16, 1, 1, sample_rate, sample_rate * 2, 2, 16),
            b"data",
            struct.pack("<I", len(payload)),
            payload,
        )
    )


def write_checkpoint(
    output: Path, candidate_id: str, model: ListenerField
) -> dict[str, Any]:
    directory = output / "checkpoints" / candidate_id
    directory.mkdir(parents=True)
    offset = 0
    records = []
    payload = bytearray()
    for name, tensor in sorted(model.state_dict().items()):
        array = tensor.detach().cpu().numpy().astype("<f8", copy=False)
        data = array.tobytes(order="C")
        records.append(
            {
                "name": name,
                "shape": list(array.shape),
                "dtype": "float64le",
                "offset_bytes": offset,
                "byte_count": len(data),
            }
        )
        payload.extend(data)
        offset += len(data)
    weights_path = directory / "weights.f64le"
    weights_path.write_bytes(payload)
    descriptor = {
        "schema": "nextengine.experimental-listener-field-checkpoint.v1",
        "candidate_id": candidate_id,
        "weights_sha256": sha256_bytes(payload),
        "weights_byte_count": len(payload),
        "parameters": records,
    }
    descriptor_path = directory / "checkpoint.json"
    descriptor_bytes = canonical_json(descriptor)
    descriptor_path.write_bytes(descriptor_bytes)
    return {
        "descriptor_file": str(descriptor_path.relative_to(output)),
        "descriptor_sha256": sha256_bytes(descriptor_bytes),
        "weights_file": str(weights_path.relative_to(output)),
        "weights_sha256": descriptor["weights_sha256"],
        "weights_byte_count": len(payload),
    }


def preflight(
    root: Path,
    manifest_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    output: Path,
) -> None:
    projection, baseline, projection_bytes, baseline_bytes = (
        validate_projection_and_baseline(root, manifest_path, manifest)
    )
    signals, coordinates, sample_rate, context = read_context(
        root, manifest_path, manifest, projection
    )
    query, query_records = query_coordinates(manifest, projection)
    if signals.shape[1] > sample_rate * 30:
        raise ListenerFieldError("listener-field audio exceeds bounded duration")
    report = {
        "schema": PREFLIGHT_SCHEMA,
        "status": "Validated",
        "decision": "ListenerFieldR2ProtocolFrozen",
        "claim": (
            "FIXED_IMPACT_LISTENER_FIELD_PROTOCOL_ONLY / NO_TRAINED_MODEL_"
            "QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
        ),
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "runner_sha256": sha256_file(source_path(root)),
        "projection_sha256": sha256_bytes(projection_bytes),
        "baseline_report_sha256": sha256_bytes(baseline_bytes),
        "baseline_metric_profile": baseline["metric_profile"]["id"],
        "candidate_profile": candidate_profile(),
        "environment": environment_profile(),
        "sample_rate_hz": sample_rate,
        "sample_count": int(signals.shape[1]),
        "context_rows": context,
        "query_rows_without_audio": query_records,
        "context_coordinate_z_metres": coordinates.tolist(),
        "query_coordinate_z_metres": query.tolist(),
        "optimizer_steps": 0,
        "query_audio_bytes_read": 0,
        "method_holdout_or_shadow_bytes_read": 0,
        "training_authorized_by_this_report": True,
        "quality_or_admission_authorized": False,
    }
    report_bytes = canonical_json(report)
    (output / "preflight-report.json").write_bytes(report_bytes)
    print(report_bytes.decode(), end="")


def validate_preflight(
    root: Path,
    manifest_path: Path,
    manifest_bytes: bytes,
    preflight_path: Path | None,
) -> tuple[Path, bytes, dict[str, Any]]:
    if preflight_path is None:
        raise ListenerFieldError("train stage requires --preflight")
    path = external_file(root, preflight_path, "listener-field preflight")
    data, report = read_json(path, "listener-field preflight")
    if (
        report.get("schema") != PREFLIGHT_SCHEMA
        or report.get("status") != "Validated"
        or report.get("decision") != "ListenerFieldR2ProtocolFrozen"
        or report.get("manifest_sha256") != sha256_bytes(manifest_bytes)
        or report.get("runner_sha256") != sha256_file(source_path(root))
        or report.get("optimizer_steps") != 0
        or report.get("query_audio_bytes_read") != 0
        or report.get("method_holdout_or_shadow_bytes_read") != 0
    ):
        raise ListenerFieldError("listener-field preflight lineage changed")
    if path.parent == manifest_path.parent:
        raise ListenerFieldError("preflight and manifest require distinct external outputs")
    return path, data, report


def log_mlflow_candidate(
    tracking_directory: Path,
    experiment_name: str,
    candidate_id: str,
    profile: dict[str, Any],
    training: dict[str, Any],
    output: Path,
    checkpoint: dict[str, Any],
    prediction_records: list[dict[str, Any]],
) -> dict[str, Any]:
    database = tracking_directory / "mlflow.db"
    artifacts = tracking_directory / "artifacts"
    artifacts.mkdir(exist_ok=True)
    tracking_uri = f"sqlite:///{database}"
    mlflow.set_tracking_uri(tracking_uri)
    client = mlflow.tracking.MlflowClient(tracking_uri=tracking_uri)
    experiment = client.get_experiment_by_name(experiment_name)
    if experiment is None:
        experiment_id = client.create_experiment(
            experiment_name, artifact_location=artifacts.as_uri()
        )
    else:
        experiment_id = experiment.experiment_id
    with mlflow.start_run(experiment_id=experiment_id, run_name=candidate_id) as run:
        mlflow.log_params(
            {
                "revision": REVISION,
                "candidate_id": candidate_id,
                "rank": profile["rank"],
                "seed": SEED,
                "epochs": EPOCHS,
                "learning_rate": LEARNING_RATE,
                "weight_decay": WEIGHT_DECAY,
                "smoothness_weight": SMOOTHNESS_WEIGHT,
                "dtype": "float64",
                "device": "cpu",
            }
        )
        for key, value in training.items():
            if isinstance(value, (int, float)) and math.isfinite(float(value)):
                mlflow.log_metric(key, float(value))
        for relative in [
            checkpoint["descriptor_file"],
            checkpoint["weights_file"],
            *(record["prediction_file"] for record in prediction_records),
        ]:
            mlflow.log_artifact(str(output / relative))
        return {
            "candidate_id": candidate_id,
            "run_id": run.info.run_id,
            "experiment_id": run.info.experiment_id,
        }


def train(
    root: Path,
    manifest_path: Path,
    manifest_bytes: bytes,
    manifest: dict[str, Any],
    preflight_path: Path | None,
    output: Path,
) -> None:
    _, preflight_bytes, _ = validate_preflight(
        root, manifest_path, manifest_bytes, preflight_path
    )
    projection, _, projection_bytes, baseline_bytes = (
        validate_projection_and_baseline(root, manifest_path, manifest)
    )
    signals, context_coordinate, sample_rate, context_records = read_context(
        root, manifest_path, manifest, projection
    )
    query_coordinate, query_records = query_coordinates(manifest, projection)
    configure_determinism()
    (output / "predictions").mkdir()
    tracking_directory = output / "mlflow"
    tracking_directory.mkdir()
    candidate_reports = []
    mlflow_runs = []
    for profile in CANDIDATES:
        model, predictions, training = train_candidate(
            signals, context_coordinate, query_coordinate, profile["rank"]
        )
        checkpoint = write_checkpoint(output, profile["candidate_id"], model)
        prediction_records = []
        for row, samples in zip(query_records, predictions, strict=True):
            relative = (
                Path("predictions")
                / profile["candidate_id"]
                / f"{row['row_id']}.wav"
            )
            path = output / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            wav = encode_wav(samples, sample_rate)
            path.write_bytes(wav)
            prediction_records.append(
                {
                    "row_id": row["row_id"],
                    "listener_point_metres": row["listener_point_metres"],
                    "prediction_file": str(relative),
                    "prediction_sha256": sha256_bytes(wav),
                    "prediction_byte_count": len(wav),
                    "pre_cook_peak_abs": float(np.max(np.abs(samples))),
                }
            )
        candidate_reports.append(
            {
                **profile,
                "training": training,
                "checkpoint": checkpoint,
                "predictions": prediction_records,
            }
        )
        mlflow_runs.append(
            log_mlflow_candidate(
                tracking_directory,
                manifest["mlflow_experiment"],
                profile["candidate_id"],
                profile,
                training,
                output,
                checkpoint,
                prediction_records,
            )
        )
    report = {
        "schema": TRAINING_SCHEMA,
        "status": "Trained",
        "decision": "ListenerFieldCandidatesCookedForFrozenDevelopmentEvaluation",
        "claim": (
            "FIXED_IMPACT_LISTENER_FIELD_TRAINING_AND_PCM_COOK_ONLY / NO_"
            "QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY"
        ),
        "revision": REVISION,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "preflight_sha256": sha256_bytes(preflight_bytes),
        "runner_sha256": sha256_file(source_path(root)),
        "projection_sha256": sha256_bytes(projection_bytes),
        "baseline_report_sha256": sha256_bytes(baseline_bytes),
        "candidate_profile": candidate_profile(),
        "environment": environment_profile(),
        "sample_rate_hz": sample_rate,
        "sample_count": int(signals.shape[1]),
        "context_rows": context_records,
        "candidates": candidate_reports,
        "query_audio_bytes_read": 0,
        "method_holdout_or_shadow_bytes_read": 0,
        "quality_or_admission_authorized": False,
    }
    report_bytes = canonical_json(report)
    (output / "training-report.json").write_bytes(report_bytes)
    mlflow_lineage = {
        "schema": MLFLOW_SCHEMA,
        "mlflow_version": mlflow.__version__,
        "tracking_database": "mlflow/mlflow.db",
        "artifact_directory": "mlflow/artifacts",
        "experiment_name": manifest["mlflow_experiment"],
        "runs": mlflow_runs,
        "deterministic_training_report_sha256": sha256_bytes(report_bytes),
    }
    (output / "mlflow-lineage.json").write_bytes(canonical_json(mlflow_lineage))
    print(report_bytes.decode(), end="")


def main() -> None:
    arguments = parse_arguments()
    root = Path(__file__).resolve(strict=True).parents[2]
    source_path(root)
    manifest_path = external_file(root, arguments.manifest, "listener-field manifest")
    manifest_bytes, manifest = read_json(manifest_path, "listener-field manifest")
    validate_manifest_shape(manifest)
    output = external_output(root, arguments.output)
    try:
        if arguments.stage == "preflight":
            if arguments.preflight is not None:
                raise ListenerFieldError("preflight stage does not accept --preflight")
            preflight(root, manifest_path, manifest_bytes, manifest, output)
        else:
            train(
                root,
                manifest_path,
                manifest_bytes,
                manifest,
                arguments.preflight,
                output,
            )
    except Exception:
        if output.is_dir() and not any(output.iterdir()):
            output.rmdir()
        raise


if __name__ == "__main__":
    main()
