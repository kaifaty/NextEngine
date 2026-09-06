#!/usr/bin/env python3
"""Frozen R2E low-rank listener-coordinate field protocol and model."""

from __future__ import annotations

import math
import sys
from typing import Any

import torch
from torch import nn

import physical_sound_listener_field_r2d_v2_common as r2d_v2

v1 = r2d_v2.v1
r2c = r2d_v2.r2c
r2b = r2c.r2b

TRAIN_MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2e-train.manifest.v1"
)
TRAIN_FREEZE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2e-train-freeze.report.v1"
)
TRAIN_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2e-train.report.v1"
)
MLFLOW_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2e-mlflow-lineage.v1"
)
EVALUATION_MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2e-evaluation.manifest.v1"
)
EVALUATION_FREEZE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2e-evaluation-"
    "freeze.report.v1"
)
EVALUATION_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2e-evaluation.report.v1"
)
REVISION = "green-goblet-low-rank-harmonic-listener-field-r2e-v1"
CANDIDATE_ID = "group_conditioned_cosine_coefficient_field_v1"
R2D_V2_MANIFEST_SHA256 = (
    "47920fce0dac93035d19a3b69ddbbd7c6eda4b5db2996887942a0765b683f6d1"
)
R2D_V2_RUN_REPORT_SHA256 = (
    "898ee201018c4f9375f838e2f6de9e62a05f8b59700258571ed9ee7a5100875c"
)
R2B_REPORT_SHA256 = (
    "e3db03ac75efc2d7153b8c3f02172a47eb5ad384cb85ff489810f2f239d31696"
)
DISTANCE_OFFSETS_MILLIMETRES = (0, 333, 666, 1_000)
MICROPHONE_IDS = tuple(range(r2b.MICROPHONES_PER_COLUMN))
CONTEXT_ANGLES_DEGREES = (0, 20, 60, 80, 120, 140, 180)
QUERY_ANGLES_DEGREES = r2b.QUERY_ANGLES
ANGULAR_BASIS_COUNT = len(CONTEXT_ANGLES_DEGREES)
GROUP_COUNT = len(DISTANCE_OFFSETS_MILLIMETRES) * len(MICROPHONE_IDS)
RANK = v1.RANK
STEPS = 1_600
TF_CHUNK = v1.TF_CHUNK
COOK_ROW_CHUNK = 208
SEED = r2d_v2.SEED
LOG_INTERVAL = r2d_v2.LOG_INTERVAL
GRADIENT_CLIP_NORM = r2d_v2.GRADIENT_CLIP_NORM
PRIMARY_ENDPOINTS = r2c.PRIMARY_ENDPOINTS


class R2EError(RuntimeError):
    """The frozen R2E coordinate-field boundary failed closed."""


def profile() -> dict[str, Any]:
    return {
        "profile_id": "group-conditioned-cosine-low-rank-field-v1",
        "revision": REVISION,
        "candidate_id": CANDIDATE_ID,
        "seed": SEED,
        "claim_domain": {
            "object": "realimpact-green-goblet",
            "impact": "fixed-mesh-vertex-31676",
            "azimuth_degrees": [0, 180],
            "distance_offsets_millimetres": list(
                DISTANCE_OFFSETS_MILLIMETRES
            ),
            "microphone_ids": list(MICROPHONE_IDS),
            "held_query_angles_degrees": list(QUERY_ANGLES_DEGREES),
            "arbitrary_distance_or_height_generalization": False,
        },
        "representation": {
            "source": "passed_r2d_v2_context_rank96_complex_basis",
            "rank": RANK,
            "target": "normalized_complex_basis_coefficients",
            "basis_learning": False,
            "query_audio_used_for_training_or_selection": False,
        },
        "model": {
            "family": "fixed_cosine_encoder_plus_group_conditioned_linear_head",
            "inputs": [
                "published_azimuth_degrees",
                "published_gantry_distance_offset_millimetres",
                "published_microphone_id",
            ],
            "angular_features": "cos(k*azimuth_radians)_for_k_0_through_6",
            "grouping": "exact_distance_offset_cross_microphone_id",
            "group_count": GROUP_COUNT,
            "angular_basis_count": ANGULAR_BASIS_COUNT,
            "output": "rank96_complex_normalized_coefficients",
            "initialization": "all_zero_equal_global_mean_field",
            "parameter_count": GROUP_COUNT * ANGULAR_BASIS_COUNT * RANK * 2,
        },
        "optimizer": {
            "name": "torch_adamw_half_cosine_decay_fixed_steps",
            "steps": STEPS,
            "learning_rate_schedule": r2d_v2.profile()["optimizer"][
                "learning_rate_schedule"
            ],
            "betas": list(r2d_v2.ADAM_BETAS),
            "epsilon": r2d_v2.ADAM_EPSILON,
            "weight_decay": r2d_v2.WEIGHT_DECAY,
            "gradient_clip_norm": GRADIENT_CLIP_NORM,
            "checkpoint_policy": "final_step_only_without_query_feedback",
        },
        "objective": v1.profile()["objective"],
        "context_gate": v1.profile()["gates"],
        "repetition": {
            "count": 2,
            "required": "byte_identical_report_checkpoint_and_predictions",
        },
        "query_evaluation": {
            "runs_authorized": 1,
            "rows": r2b.QUERY_ROWS,
            "controls": list(r2b.CONTROL_IDS),
            "primary_endpoints": list(PRIMARY_ENDPOINTS),
            "selection": (
                "strictly_lower_than_every_control_on_all_five_endpoints"
            ),
        },
        "method_holdout_or_shadow_available": False,
    }


def distance_index(offset_millimetres: int) -> int:
    try:
        return DISTANCE_OFFSETS_MILLIMETRES.index(offset_millimetres)
    except ValueError as error:
        raise R2EError("R2E row uses an unsupported distance offset") from error


def encode_rows(
    rows: list[dict[str, Any]],
    device: torch.device,
    *,
    expected_angles: tuple[int, ...],
) -> tuple[torch.Tensor, torch.Tensor]:
    angles = []
    groups = []
    seen = set()
    for row in rows:
        angle = row.get("azimuth_degrees")
        offset = row.get("gantry_distance_offset_millimetres")
        microphone = row.get("microphone_id")
        if (
            not isinstance(angle, int)
            or angle not in expected_angles
            or not isinstance(offset, int)
            or not isinstance(microphone, int)
            or microphone not in MICROPHONE_IDS
        ):
            raise R2EError("R2E row escaped the frozen published coordinate grid")
        group = distance_index(offset) * len(MICROPHONE_IDS) + microphone
        identity = (angle, group)
        if identity in seen:
            raise R2EError("R2E coordinate grid contains a duplicate row")
        seen.add(identity)
        angles.append(math.radians(angle))
        groups.append(group)
    return (
        torch.tensor(angles, dtype=torch.float32, device=device),
        torch.tensor(groups, dtype=torch.int64, device=device),
    )


def angular_features(angles_radians: torch.Tensor) -> torch.Tensor:
    orders = torch.arange(
        ANGULAR_BASIS_COUNT,
        dtype=angles_radians.dtype,
        device=angles_radians.device,
    )
    return torch.cos(angles_radians[:, None] * orders[None, :])


class HarmonicCoefficientField(nn.Module):
    """One bounded neural coordinate field over the published listener grid."""

    def __init__(self) -> None:
        super().__init__()
        self.weights = nn.Parameter(
            torch.zeros((GROUP_COUNT, ANGULAR_BASIS_COUNT, RANK, 2))
        )

    def forward(
        self, angles_radians: torch.Tensor, group_indices: torch.Tensor
    ) -> torch.Tensor:
        features = angular_features(angles_radians)
        selected = self.weights[group_indices]
        return torch.einsum("nf,nfrc->nrc", features, selected)


def learning_rate_for_step(step_index: int) -> float:
    return r2d_v2.learning_rate_for_step(step_index, STEPS)


if __name__ == "__main__":
    sys.exit("physical_sound_listener_field_r2e_common.py is a library module")
