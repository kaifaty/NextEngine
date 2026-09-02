"""Prefix-bounded synthetic loss for the V28 M0c execution successor."""

from __future__ import annotations

import physical_sound_v25_m0a_common as common
import physical_sound_v25_m0a_model as inherited
import torch
from torch.nn import functional

TRAINING_SPECTRUM_WINDOWS = (256, 1024, 4096)
TRAINING_RENDER_FRAMES = 4096

SEED = inherited.SEED
SAMPLE_RATE_HZ = inherited.SAMPLE_RATE_HZ
MODE_COUNT = inherited.MODE_COUNT
RESIDUAL_COUNT = inherited.RESIDUAL_COUNT
OBJECT_FEATURE_COUNT = inherited.OBJECT_FEATURE_COUNT
CONTACT_FEATURE_COUNT = inherited.CONTACT_FEATURE_COUNT
PARAMETER_LIMIT = inherited.PARAMETER_LIMIT
SYNTHETIC_LEARNING_RATE = inherited.SYNTHETIC_LEARNING_RATE
REAL_LEARNING_RATE = inherited.REAL_LEARNING_RATE
WEIGHT_DECAY = inherited.WEIGHT_DECAY
GRADIENT_CLIP = inherited.GRADIENT_CLIP
RIDGE_LAMBDA = inherited.RIDGE_LAMBDA

ModalPrediction = inherited.ModalPrediction
RidgeModel = inherited.RidgeModel
ContactModalField = inherited.ContactModalField
configure_determinism = inherited.configure_determinism
parameter_count = inherited.parameter_count
validate_model = inherited.validate_model
render_prediction = inherited.render_prediction
multi_resolution_log_spectrum = inherited.multi_resolution_log_spectrum
decay_slope = inherited.decay_slope
real_acoustic_components = inherited.real_acoustic_components
fit_ridge = inherited.fit_ridge
predict_ridge = inherited.predict_ridge
prediction_numpy = inherited.prediction_numpy
model_tensors = inherited.model_tensors


def synthetic_loss(
    prediction: ModalPrediction,
    target_frequency: torch.Tensor,
    target_decay: torch.Tensor,
    target_gains: torch.Tensor,
    mode_mask: torch.Tensor,
    target_waveform: torch.Tensor,
) -> tuple[torch.Tensor, dict[str, torch.Tensor]]:
    if target_waveform.shape[-1] < TRAINING_RENDER_FRAMES:
        raise common.M0Error(
            f"M0c synthetic target must contain at least {TRAINING_RENDER_FRAMES} frames"
        )
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
    gains = (
        torch.sum(
            functional.huber_loss(prediction.gains, target_gains, reduction="none")
            * mode_mask
        )
        / denominator
    )
    rendered = render_prediction(prediction, TRAINING_RENDER_FRAMES)
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
