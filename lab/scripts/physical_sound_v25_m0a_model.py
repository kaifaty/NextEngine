"""Frozen compact PyTorch model and acoustic primitives for V25 M0a."""

from __future__ import annotations

import math
import os
import random
from collections.abc import Iterable
from dataclasses import dataclass

import numpy as np
import physical_sound_v25_m0a_common as common
import torch
from torch import nn
from torch.nn import functional

SEED = 3_101
SAMPLE_RATE_HZ = 48_000
MODE_COUNT = 10
RESIDUAL_COUNT = 8
OBJECT_FEATURE_COUNT = 38
CONTACT_FEATURE_COUNT = 24
PARAMETER_LIMIT = 40_000
SYNTHETIC_LEARNING_RATE = 1.0e-3
REAL_LEARNING_RATE = 2.0e-4
WEIGHT_DECAY = 1.0e-5
GRADIENT_CLIP = 1.0
RIDGE_LAMBDA = 1.0e-3


@dataclass(frozen=True)
class ModalPrediction:
    frequencies_hz: torch.Tensor
    decay_per_second: torch.Tensor
    gains: torch.Tensor
    residual_gains: torch.Tensor


@dataclass(frozen=True)
class RidgeModel:
    mean: np.ndarray
    scale: np.ndarray
    coefficients: np.ndarray


def configure_determinism(seed: int = SEED) -> dict[str, object]:
    if seed != SEED:
        raise common.M0Error(f"M0 seed must be {SEED}")
    for name in (
        "OMP_NUM_THREADS",
        "OPENBLAS_NUM_THREADS",
        "MKL_NUM_THREADS",
        "NUMEXPR_NUM_THREADS",
    ):
        observed = os.environ.get(name)
        if observed not in {None, "1"}:
            raise common.M0Error(f"M0 {name} must be unset or 1, got {observed}")
        os.environ[name] = "1"
    random.seed(seed)
    np.random.seed(seed)
    torch.manual_seed(seed)
    torch.set_num_threads(1)
    try:
        torch.set_num_interop_threads(1)
    except RuntimeError:
        if torch.get_num_interop_threads() != 1:
            raise common.M0Error("M0 inter-op thread count cannot be fixed at one") from None
    torch.use_deterministic_algorithms(True)
    torch.set_default_dtype(torch.float32)
    if torch.get_num_threads() != 1 or torch.get_num_interop_threads() != 1:
        raise common.M0Error("M0 torch thread policy changed")
    if torch.backends.cuda.matmul.allow_tf32 or torch.backends.cudnn.allow_tf32:
        torch.backends.cuda.matmul.allow_tf32 = False
        torch.backends.cudnn.allow_tf32 = False
    return {
        "seed": seed,
        "device": "cpu",
        "parameter_dtype": "float32",
        "intra_op_threads": torch.get_num_threads(),
        "inter_op_threads": torch.get_num_interop_threads(),
        "deterministic_algorithms": torch.are_deterministic_algorithms_enabled(),
        "cuda_used": False,
        "tf32": False,
        "amp": False,
        "compile": False,
        "data_loader_workers": 0,
    }


class ContactModalField(nn.Module):
    def __init__(self) -> None:
        super().__init__()
        self.object_trunk = nn.Sequential(
            nn.Linear(OBJECT_FEATURE_COUNT, 64),
            nn.SiLU(),
            nn.Linear(64, 64),
            nn.SiLU(),
        )
        self.global_head = nn.Linear(64, MODE_COUNT * 2)
        self.contact_trunk = nn.Sequential(
            nn.Linear(64 + CONTACT_FEATURE_COUNT, 64),
            nn.SiLU(),
            nn.Linear(64, 64),
            nn.SiLU(),
            nn.Linear(64, 64),
            nn.SiLU(),
        )
        self.contact_head = nn.Linear(64, MODE_COUNT)
        self.residual_head = nn.Linear(64, RESIDUAL_COUNT)

    def forward(
        self,
        object_features: torch.Tensor,
        contact_features: torch.Tensor,
        *,
        zero_geometry: bool = False,
        zero_contact: bool = False,
        zero_residual: bool = False,
    ) -> ModalPrediction:
        if object_features.shape[-1] != OBJECT_FEATURE_COUNT or contact_features.shape[-1] != CONTACT_FEATURE_COUNT:
            raise common.M0Error("M0 model input feature shape changed")
        if object_features.device.type != "cpu" or contact_features.device.type != "cpu":
            raise common.M0Error("M0 model is CPU-only")
        object_input = object_features.clone()
        if zero_geometry:
            object_input[..., :24] = 0.0
        contact_input = torch.zeros_like(contact_features) if zero_contact else contact_features
        encoded_object = self.object_trunk(object_input)
        global_values = self.global_head(encoded_object)
        gap_logits, decay_logits = torch.split(global_values, MODE_COUNT, dim=-1)
        gaps = functional.softplus(gap_logits) + 1.0e-5
        cumulative = torch.cumsum(gaps, dim=-1)
        unit = cumulative / (1.0 + cumulative[..., -1:])
        log_minimum = math.log(20.0)
        log_maximum = math.log(18_000.0)
        frequencies = torch.exp(log_minimum + unit * (log_maximum - log_minimum))
        decay = functional.softplus(decay_logits) + 1.0e-4
        encoded_contact = self.contact_trunk(torch.cat((encoded_object, contact_input), dim=-1))
        gains = torch.tanh(self.contact_head(encoded_contact))
        residual = torch.tanh(self.residual_head(encoded_contact))
        if zero_residual:
            residual = torch.zeros_like(residual)
        return ModalPrediction(frequencies, decay, gains, residual)


def parameter_count(model: nn.Module) -> int:
    count = sum(parameter.numel() for parameter in model.parameters() if parameter.requires_grad)
    if count > PARAMETER_LIMIT:
        raise common.M0Error(f"M0 parameter count exceeds {PARAMETER_LIMIT}: {count}")
    return count


def validate_model(model: ContactModalField) -> int:
    count = parameter_count(model)
    if any(parameter.device.type != "cpu" or parameter.dtype != torch.float32 for parameter in model.parameters()):
        raise common.M0Error("M0 model parameters must be CPU float32")
    with torch.no_grad():
        prediction = model(
            torch.zeros((2, OBJECT_FEATURE_COUNT), dtype=torch.float32),
            torch.zeros((2, CONTACT_FEATURE_COUNT), dtype=torch.float32),
        )
    frequencies = prediction.frequencies_hz.numpy()
    decay = prediction.decay_per_second.numpy()
    if (
        not np.all(np.isfinite(frequencies))
        or not np.all(np.diff(frequencies, axis=1) > 0.0)
        or float(np.min(frequencies)) < 20.0
        or float(np.max(frequencies)) > 18_000.0
        or not np.all(np.isfinite(decay))
        or not np.all(decay > 0.0)
    ):
        raise common.M0Error("M0 sorted-frequency or positive-decay transform failed")
    return count


def residual_parameters(device: torch.device, dtype: torch.dtype) -> tuple[torch.Tensor, torch.Tensor]:
    frequencies = torch.logspace(
        math.log10(180.0),
        math.log10(12_000.0),
        RESIDUAL_COUNT,
        device=device,
        dtype=dtype,
    )
    decay = torch.logspace(math.log10(18.0), math.log10(240.0), RESIDUAL_COUNT, device=device, dtype=dtype)
    return frequencies, decay


def render_prediction(prediction: ModalPrediction, frames: int, impulse: float = 1.0) -> torch.Tensor:
    if frames <= 0 or not math.isfinite(impulse):
        raise common.M0Error("M0 render request is invalid")
    frequency = prediction.frequencies_hz
    decay = prediction.decay_per_second
    gains = prediction.gains
    residual_gains = prediction.residual_gains
    time = torch.arange(frames, dtype=frequency.dtype, device=frequency.device) / SAMPLE_RATE_HZ
    modal_basis = torch.exp(-decay[..., :, None] * time) * torch.sin(2.0 * math.pi * frequency[..., :, None] * time)
    residual_frequency, residual_decay = residual_parameters(frequency.device, frequency.dtype)
    residual_basis = torch.exp(-residual_decay[:, None] * time) * torch.cos(
        2.0 * math.pi * residual_frequency[:, None] * time
    )
    modal = torch.sum(gains[..., :, None] * modal_basis, dim=-2)
    residual = torch.sum(residual_gains[..., :, None] * residual_basis, dim=-2)
    scale = 0.0125
    result = impulse * scale * (modal + 0.125 * residual)
    if not torch.all(torch.isfinite(result)):
        raise common.M0Error("M0 render is non-finite")
    return result


def _windowed(samples: torch.Tensor, size: int) -> torch.Tensor:
    if samples.shape[-1] >= size:
        return samples[..., :size]
    return functional.pad(samples, (0, size - samples.shape[-1]))


def log_spectrum(samples: torch.Tensor, size: int) -> torch.Tensor:
    value = _windowed(samples, size)
    window = torch.hann_window(size, periodic=False, dtype=value.dtype, device=value.device)
    return torch.log(torch.abs(torch.fft.rfft(value * window, n=size)) + 1.0e-7)


def closed_form_log_gain(predicted: torch.Tensor, target: torch.Tensor, size: int = 4096) -> torch.Tensor:
    predicted_log = log_spectrum(predicted, size)
    target_log = log_spectrum(target, size)
    return torch.mean(target_log - predicted_log, dim=-1, keepdim=True)


def multi_resolution_log_spectrum(predicted: torch.Tensor, target: torch.Tensor) -> torch.Tensor:
    losses = []
    for size in (256, 1024, 4096):
        predicted_log = log_spectrum(predicted, size)
        target_log = log_spectrum(target, size)
        nuisance = torch.mean(target_log - predicted_log, dim=-1, keepdim=True)
        losses.append(torch.mean(torch.abs(predicted_log + nuisance - target_log)))
    return torch.mean(torch.stack(losses))


def decay_slope(samples: torch.Tensor, blocks: int = 16) -> torch.Tensor:
    usable = samples.shape[-1] - (samples.shape[-1] % blocks)
    if usable < blocks:
        raise common.M0Error("M0 decay envelope is too short")
    energy = torch.mean(
        samples[..., :usable].reshape(*samples.shape[:-1], blocks, usable // blocks) ** 2,
        dim=-1,
    )
    log_energy = torch.log(energy + 1.0e-10)
    x = torch.linspace(-1.0, 1.0, blocks, dtype=samples.dtype, device=samples.device)
    return torch.sum(
        (x - torch.mean(x)) * (log_energy - torch.mean(log_energy, dim=-1, keepdim=True)),
        dim=-1,
    ) / torch.sum((x - torch.mean(x)) ** 2)


def modal_peak_vector(samples: torch.Tensor, count: int = MODE_COUNT) -> torch.Tensor:
    spectrum = log_spectrum(samples, 4096)
    values, indices = torch.topk(spectrum[..., 1:], count, dim=-1, largest=True, sorted=True)
    frequencies = (indices.to(samples.dtype) + 1.0) * SAMPLE_RATE_HZ / 4096.0
    order = torch.argsort(frequencies, dim=-1)
    return torch.cat(
        (
            torch.gather(frequencies, -1, order) / 18_000.0,
            torch.gather(values, -1, order) / 20.0,
        ),
        dim=-1,
    )


def real_acoustic_components(predicted: torch.Tensor, target: torch.Tensor) -> dict[str, torch.Tensor]:
    return {
        "multi_resolution_log_spectrum": multi_resolution_log_spectrum(predicted, target),
        "decay_envelope_slope": torch.mean(torch.abs(decay_slope(predicted) - decay_slope(target))),
        "modal_peak_set": torch.mean(torch.abs(modal_peak_vector(predicted) - modal_peak_vector(target))),
    }


def synthetic_loss(
    prediction: ModalPrediction,
    target_frequency: torch.Tensor,
    target_decay: torch.Tensor,
    target_gains: torch.Tensor,
    mode_mask: torch.Tensor,
    target_waveform: torch.Tensor,
) -> tuple[torch.Tensor, dict[str, torch.Tensor]]:
    denominator = torch.sum(mode_mask).clamp_min(1.0)
    frequency = (
        torch.sum(
            functional.huber_loss(
                torch.log(prediction.frequencies_hz),
                torch.log(target_frequency),
                reduction="none",
            )
            * mode_mask
        )
        / denominator
    )
    decay = (
        torch.sum(
            functional.huber_loss(
                torch.log(prediction.decay_per_second),
                torch.log(target_decay),
                reduction="none",
            )
            * mode_mask
        )
        / denominator
    )
    gains = torch.sum(functional.huber_loss(prediction.gains, target_gains, reduction="none") * mode_mask) / denominator
    rendered = render_prediction(prediction, target_waveform.shape[-1])
    spectrum = multi_resolution_log_spectrum(rendered, target_waveform)
    residual = torch.mean(prediction.residual_gains**2)
    components = {
        "log_frequency_huber": frequency,
        "log_decay_huber": decay,
        "signed_gain_huber": gains,
        "rendered_log_spectrum": spectrum,
        "residual_zero": residual,
    }
    total = frequency + 0.5 * decay + gains + 0.5 * spectrum + 0.1 * residual
    return total, components


def fit_ridge(examples: Iterable[common.SyntheticExample]) -> RidgeModel:
    rows = tuple(examples)
    if not rows:
        raise common.M0Error("M0 ridge has no training examples")
    features = np.stack([np.concatenate((row.object_features, row.contact_features)) for row in rows]).astype(
        np.float64
    )
    targets = []
    for row in rows:
        frequency = np.log(np.maximum(row.frequencies_hz, 20.0))
        decay = np.log(np.maximum(row.decay_per_second, 1.0e-4))
        targets.append(np.concatenate((frequency, decay, row.gains)).astype(np.float64))
    target = np.stack(targets)
    mean = np.mean(features, axis=0)
    scale = np.std(features, axis=0)
    scale = np.where(scale > 1.0e-8, scale, 1.0)
    standardized = (features - mean) / scale
    design = np.concatenate((np.ones((len(rows), 1)), standardized, standardized**2), axis=1)
    gram = design.T @ design + RIDGE_LAMBDA * np.eye(design.shape[1])
    coefficients = np.linalg.solve(gram, design.T @ target)
    if not np.all(np.isfinite(coefficients)):
        raise common.M0Error("M0 ridge coefficients are non-finite")
    return RidgeModel(mean, scale, coefficients)


def predict_ridge(
    model: RidgeModel, object_features: np.ndarray, contact_features: np.ndarray
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    features = np.concatenate((object_features, contact_features)).astype(np.float64)
    standardized = (features - model.mean) / model.scale
    design = np.concatenate(([1.0], standardized, standardized**2))
    output = design @ model.coefficients
    frequencies = np.sort(np.clip(np.exp(output[:MODE_COUNT]), 20.0, 18_000.0))
    for index in range(1, len(frequencies)):
        frequencies[index] = max(frequencies[index], frequencies[index - 1] + 1.0e-3)
    decay = np.maximum(np.exp(output[MODE_COUNT : 2 * MODE_COUNT]), 1.0e-4)
    gains = np.tanh(output[2 * MODE_COUNT : 3 * MODE_COUNT])
    return frequencies, decay, gains


def model_tensors(model: ContactModalField) -> dict[str, np.ndarray]:
    result = {}
    for name, value in sorted(model.state_dict().items()):
        array = value.detach().cpu().numpy().astype("<f4", copy=True)
        if not np.all(np.isfinite(array)):
            raise common.M0Error(f"M0 model tensor is non-finite: {name}")
        result[name] = array
    return result


def prediction_numpy(prediction: ModalPrediction) -> dict[str, np.ndarray]:
    return {
        "decay_per_second": prediction.decay_per_second.detach().cpu().numpy().astype("<f4", copy=True),
        "frequencies_hz": prediction.frequencies_hz.detach().cpu().numpy().astype("<f4", copy=True),
        "gains": prediction.gains.detach().cpu().numpy().astype("<f4", copy=True),
        "residual_gains": prediction.residual_gains.detach().cpu().numpy().astype("<f4", copy=True),
    }
