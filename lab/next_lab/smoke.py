from __future__ import annotations

import hashlib
import json
import math
import os
import platform
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import numpy as np
import onnx
import onnxruntime
import torch
from torch import nn

SEEDS = (11, 29, 47, 83)
ENVIRONMENTS = 8
STEPS = 256
PARITY_OBSERVATIONS = 1_000
MAX_PARITY_ERROR = 1e-5
MAX_DURATION_SECONDS = 600.0


@dataclass(frozen=True)
class SmokeConfig:
    device: str = "auto"
    iterations: int = 256


class TinyMotorPolicy(nn.Module):
    def __init__(self) -> None:
        super().__init__()
        self.layers = nn.Sequential(
            nn.Linear(6, 16),
            nn.Tanh(),
            nn.Linear(16, 2),
            nn.Tanh(),
        )

    def forward(self, observation: torch.Tensor) -> torch.Tensor:
        return self.layers(observation)


def _canonical_array_hash(array: np.ndarray[Any, Any]) -> str:
    contiguous = np.ascontiguousarray(array)
    digest = hashlib.sha256()
    digest.update(contiguous.dtype.str.encode("ascii"))
    digest.update(json.dumps(list(contiguous.shape), separators=(",", ":")).encode("ascii"))
    digest.update(contiguous.tobytes(order="C"))
    return digest.hexdigest()


def _file_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _json_hash(value: Any) -> str:
    payload = json.dumps(value, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def generate_corpus() -> tuple[np.ndarray[Any, Any], np.ndarray[Any, Any]]:
    observations = []
    actions = []
    samples_per_seed = ENVIRONMENTS * STEPS
    for seed in SEEDS:
        random = np.random.default_rng(seed)
        position = random.uniform(-1.0, 1.0, size=(samples_per_seed, 2))
        velocity = random.uniform(-0.5, 0.5, size=(samples_per_seed, 2))
        target = random.uniform(-1.0, 1.0, size=(samples_per_seed, 2))
        observation = np.concatenate((position, velocity, target), axis=1).astype(np.float32)
        expert_action = np.tanh(2.0 * (target - position) - 0.25 * velocity).astype(np.float32)
        observations.append(observation)
        actions.append(expert_action)
    return np.concatenate(observations), np.concatenate(actions)


def generate_parity_observations() -> np.ndarray[Any, Any]:
    random = np.random.default_rng(99_173)
    return random.uniform(-1.0, 1.0, size=(PARITY_OBSERVATIONS, 6)).astype(np.float32)


def _mps_available() -> tuple[bool, str | None]:
    if not torch.backends.mps.is_built() or not torch.backends.mps.is_available():
        return False, "MPS backend unavailable"
    try:
        probe = torch.tensor([1.0, 2.0], device="mps")
        if not torch.isfinite(probe.square()).all().cpu().item():
            return False, "MPS finite-output probe failed"
    except Exception as error:
        return False, f"{type(error).__name__}: {error}"
    return True, None


def _selected_device(requested: str) -> tuple[str, str | None]:
    available, diagnostic = _mps_available()
    if requested == "cpu":
        return "cpu", None
    if requested == "mps":
        if not available:
            raise RuntimeError(diagnostic or "MPS unavailable")
        return "mps", None
    if available:
        return "mps", None
    return "cpu", diagnostic


def _train(
    observations: np.ndarray[Any, Any],
    actions: np.ndarray[Any, Any],
    device: str,
    iterations: int,
) -> tuple[TinyMotorPolicy, float, float]:
    torch.manual_seed(20_260_722)
    if device == "mps":
        torch.mps.manual_seed(20_260_722)
    model = TinyMotorPolicy().to(device)
    input_tensor = torch.from_numpy(observations).to(device)
    target_tensor = torch.from_numpy(actions).to(device)
    optimizer = torch.optim.Adam(model.parameters(), lr=0.02)

    with torch.no_grad():
        initial_loss = nn.functional.mse_loss(model(input_tensor), target_tensor).item()
    for _ in range(iterations):
        optimizer.zero_grad(set_to_none=True)
        prediction = model(input_tensor)
        loss = nn.functional.mse_loss(prediction, target_tensor)
        loss.backward()
        optimizer.step()
    with torch.no_grad():
        final_loss = nn.functional.mse_loss(model(input_tensor), target_tensor).item()
    return model.to("cpu").eval(), initial_loss, final_loss


def _atomic_json(path: Path, value: Any) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temporary, path)


def run_smoke(config: SmokeConfig) -> int:
    if config.iterations < 1 or config.iterations > 2_048:
        raise ValueError("iterations must be in 1..2048")

    started = time.monotonic()
    observations, actions = generate_corpus()
    parity_observations = generate_parity_observations()
    selected_device, fallback_reason = _selected_device(config.device)

    try:
        model, initial_loss, final_loss = _train(
            observations,
            actions,
            selected_device,
            config.iterations,
        )
    except Exception as error:
        if selected_device != "mps" or config.device == "mps":
            raise
        fallback_reason = f"MPS training fallback: {type(error).__name__}: {error}"
        selected_device = "cpu"
        model, initial_loss, final_loss = _train(
            observations,
            actions,
            selected_device,
            config.iterations,
        )

    repository_root = Path(__file__).resolve().parents[2]
    output_root = repository_root / ".local/training/smoke"
    output_root.mkdir(parents=True, exist_ok=True)
    model_path = output_root / "policy.onnx"

    dummy = torch.from_numpy(parity_observations[:1])
    torch.onnx.export(
        model,
        dummy,
        model_path,
        export_params=True,
        opset_version=17,
        do_constant_folding=True,
        input_names=["motor_observation"],
        output_names=["motor_action"],
        dynamic_axes={
            "motor_observation": {0: "batch"},
            "motor_action": {0: "batch"},
        },
        dynamo=False,
    )
    onnx_model = onnx.load(model_path)
    onnx.checker.check_model(onnx_model)

    with torch.no_grad():
        torch_actions = model(torch.from_numpy(parity_observations)).numpy()
    session = onnxruntime.InferenceSession(
        str(model_path),
        providers=["CPUExecutionProvider"],
    )
    ort_actions = session.run(None, {"motor_observation": parity_observations})[0]
    max_absolute_error = float(np.max(np.abs(torch_actions - ort_actions)))
    finite = bool(np.isfinite(torch_actions).all() and np.isfinite(ort_actions).all())
    duration_seconds = time.monotonic() - started
    trained = math.isfinite(final_loss) and final_loss < initial_loss * 0.25
    passed = (
        finite
        and trained
        and max_absolute_error <= MAX_PARITY_ERROR
        and duration_seconds <= MAX_DURATION_SECONDS
    )

    manifest = {
        "schema_version": 1,
        "command": "smoke",
        "check": "training-smoke",
        "status": "passed" if passed else "failed",
        "scope": "training/export/inference toolchain smoke only",
        "requested_device": config.device,
        "selected_device": selected_device,
        "fallback_reason": fallback_reason,
        "fixture": {
            "kind": "generated-2dof-control",
            "seeds": list(SEEDS),
            "environments": ENVIRONMENTS,
            "steps": STEPS,
            "training_observations": int(observations.shape[0]),
            "parity_observations": PARITY_OBSERVATIONS,
            "training_corpus_hash": _json_hash(
                {
                    "observations": _canonical_array_hash(observations),
                    "actions": _canonical_array_hash(actions),
                }
            ),
            "parity_input_hash": _canonical_array_hash(parity_observations),
        },
        "config": asdict(config),
        "config_hash": _json_hash(asdict(config)),
        "toolchain": {
            "python": platform.python_version(),
            "python_implementation": platform.python_implementation(),
            "platform": platform.system(),
            "machine": platform.machine(),
            "torch": torch.__version__,
            "onnx": onnx.__version__,
            "onnxruntime": onnxruntime.__version__,
            "opset": 17,
        },
        "metrics": {
            "initial_loss": initial_loss,
            "final_loss": final_loss,
            "max_absolute_action_error": max_absolute_error,
            "max_absolute_action_error_threshold": MAX_PARITY_ERROR,
            "finite_outputs": finite,
            "duration_seconds": duration_seconds,
            "duration_threshold_seconds": MAX_DURATION_SECONDS,
        },
        "artifacts": [
            {
                "path": "policy.onnx",
                "sha256": _file_hash(model_path),
                "bytes": model_path.stat().st_size,
            }
        ],
    }
    manifest_path = output_root / "run-manifest.json"
    _atomic_json(manifest_path, manifest)

    summary = {
        "command": manifest["command"],
        "check": manifest["check"],
        "status": manifest["status"],
        "selected_device": selected_device,
        "fallback_reason": fallback_reason,
        "max_absolute_action_error": max_absolute_error,
        "final_loss": final_loss,
        "duration_seconds": duration_seconds,
        "manifest": str(manifest_path.relative_to(repository_root)),
    }
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0 if passed else 4
