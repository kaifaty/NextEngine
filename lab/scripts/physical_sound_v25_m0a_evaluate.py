"""Frozen real-query evaluation helpers for V25 M0a."""

from __future__ import annotations

import math
from typing import Any

import numpy as np
import physical_sound_v25_m0a_common as common
import physical_sound_v25_m0a_model as model_lib
import torch


def tensor(value: np.ndarray) -> torch.Tensor:
    return torch.from_numpy(np.asarray(value, dtype=np.float32)).to(device="cpu", dtype=torch.float32)


def safe_ratio(candidate: float, control: float) -> float:
    if control == 0.0:
        if candidate != 0.0:
            return math.inf
        return 1.0
    return candidate / control


def model_prediction(
    candidate: model_lib.ContactModalField,
    object_features: np.ndarray,
    contact_features: np.ndarray,
) -> model_lib.ModalPrediction:
    return candidate(tensor(object_features).reshape(1, -1), tensor(contact_features).reshape(1, -1))


def ridge_prediction(
    ridge: model_lib.RidgeModel,
    item: common.RealTransferExample,
) -> model_lib.ModalPrediction:
    frequency, decay, gains = model_lib.predict_ridge(ridge, item.object_features, item.contact_features)
    return model_lib.ModalPrediction(
        tensor(frequency).reshape(1, -1),
        tensor(decay).reshape(1, -1),
        tensor(gains).reshape(1, -1),
        torch.zeros((1, model_lib.RESIDUAL_COUNT), dtype=torch.float32),
    )


def disclosed_real_metrics(
    candidate: model_lib.ContactModalField,
    ridge: model_lib.RidgeModel,
    contexts: list[common.RealTransferExample],
    query: common.RealTransferExample,
    recording_contexts: list[common.IdentifiedRecording],
    recording_query: common.IdentifiedRecording,
) -> dict[str, Any]:
    with torch.no_grad():
        full = model_prediction(candidate, query.object_features, query.contact_features)
        ridge_value = ridge_prediction(ridge, query)
        full_wave = model_lib.render_prediction(full, 4096)[0]
        ridge_wave = model_lib.render_prediction(ridge_value, 4096)[0]
        target = tensor(query.samples[:4096])
        nearest_item = min(
            contexts,
            key=lambda item: float(np.linalg.norm(item.contact_features - query.contact_features)),
        )
        nearest_wave = tensor(nearest_item.samples[:4096])
        full_components = {
            name: float(value) for name, value in model_lib.real_acoustic_components(full_wave, target).items()
        }
        ridge_components = {
            name: float(value) for name, value in model_lib.real_acoustic_components(ridge_wave, target).items()
        }
        nearest_components = {
            name: float(value) for name, value in model_lib.real_acoustic_components(nearest_wave, target).items()
        }
        full_composite = sum(full_components.values())
        ridge_composite = sum(ridge_components.values())
        nearest_composite = sum(nearest_components.values())
        representative = torch.mean(
            torch.stack(
                [
                    model_lib.render_prediction(
                        model_prediction(candidate, item.object_features, item.contact_features),
                        4096,
                    )[0]
                    for item in contexts
                ]
            ),
            dim=0,
        )
        recording_target = tensor(recording_query.samples[:4096])
        candidate_distribution = float(
            model_lib.multi_resolution_log_spectrum(representative, recording_target)
        ) + float(torch.abs(model_lib.decay_slope(representative) - model_lib.decay_slope(recording_target)))
        nearest_recording = min(
            recording_contexts,
            key=lambda item: float(
                model_lib.multi_resolution_log_spectrum(tensor(item.samples[:4096]), recording_target)
            ),
        )
        nearest_distribution = float(
            model_lib.multi_resolution_log_spectrum(tensor(nearest_recording.samples[:4096]), recording_target)
        ) + float(
            torch.abs(
                model_lib.decay_slope(tensor(nearest_recording.samples[:4096]))
                - model_lib.decay_slope(recording_target)
            )
        )
    improved = sum(
        full_components[name] < min(ridge_components[name], nearest_components[name]) for name in full_components
    )
    return {
        "full_components": full_components,
        "ridge_components": ridge_components,
        "nearest_components": nearest_components,
        "full_composite": full_composite,
        "ridge_composite": ridge_composite,
        "nearest_composite": nearest_composite,
        "full_to_ridge_ratio": safe_ratio(full_composite, ridge_composite),
        "full_to_nearest_ratio": safe_ratio(full_composite, nearest_composite),
        "improved_component_count": improved,
        "objectfolder_candidate_distance": candidate_distribution,
        "objectfolder_nearest_distance": nearest_distribution,
    }
