#!/usr/bin/env python3
"""Frozen R2D V2 optimizer-only successor to the V1 trainability gate."""

from __future__ import annotations

import copy
import math
import sys
from typing import Any

import physical_sound_listener_field_r2d_common as v1

r2c = v1.r2c

MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2d-v2.manifest.v1"
)
FREEZE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2d-v2-freeze.report.v1"
)
RUN_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2d-v2-run.report.v1"
)
MLFLOW_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2d-v2-mlflow-lineage.v1"
)
REVISION = "green-goblet-context-trainability-r2d-v2-cosine-decay"
V1_MANIFEST_SHA256 = (
    "3fe4129584da5395b1502c0f55e6061603c530ed9e214c48bfd13f3166c05f28"
)
INITIAL_LEARNING_RATE = v1.LEARNING_RATE
FINAL_LEARNING_RATE = 1.0e-5
SCHEDULE = "half_cosine_inclusive_endpoints_no_warmup"

# These aliases are deliberate: every non-optimizer input remains V1-exact.
RANK = v1.RANK
SEED = v1.SEED
TF_CHUNK = v1.TF_CHUNK
TASKS = v1.TASKS
LOG_INTERVAL = v1.LOG_INTERVAL
ADAM_BETAS = v1.ADAM_BETAS
ADAM_EPSILON = v1.ADAM_EPSILON
WEIGHT_DECAY = v1.WEIGHT_DECAY
GRADIENT_CLIP_NORM = v1.GRADIENT_CLIP_NORM
ALL_PROBE_CACHE_INDICES = v1.ALL_PROBE_CACHE_INDICES
CoefficientTable = v1.CoefficientTable
R2DError = v1.R2DError


def profile() -> dict[str, Any]:
    """Return V1 with only revision identity and LR scheduling changed."""

    result = copy.deepcopy(v1.profile())
    result["profile_id"] = (
        "context-low-rank-coefficient-trainability-rank96-cosine-decay-v2"
    )
    result["revision"] = REVISION
    optimizer = result["optimizer"]
    optimizer["name"] = "torch_adamw_half_cosine_decay_fixed_steps"
    optimizer.pop("learning_rate")
    optimizer["learning_rate_schedule"] = {
        "kind": SCHEDULE,
        "initial": INITIAL_LEARNING_RATE,
        "final": FINAL_LEARNING_RATE,
        "progress": "optimizer_step_index_divided_by_total_steps_minus_one",
        "applied": "before_each_optimizer_step",
    }
    result["selection"] = (
        "all_three_unchanged_context_tasks_and_cooker_comparisons_pass_else_"
        "reject_training_substrate_without_query_audio"
    )
    return result


def learning_rate_for_step(step_index: int, total_steps: int) -> float:
    """Inclusive-endpoint half-cosine schedule fixed before real-data training."""

    if total_steps < 2:
        raise R2DError("R2D V2 schedule requires at least two optimizer steps")
    if step_index < 0 or step_index >= total_steps:
        raise R2DError("R2D V2 schedule step is outside the frozen task")
    progress = step_index / (total_steps - 1)
    cosine = 0.5 * (1.0 + math.cos(math.pi * progress))
    return FINAL_LEARNING_RATE + (
        INITIAL_LEARNING_RATE - FINAL_LEARNING_RATE
    ) * cosine


if __name__ == "__main__":
    sys.exit("physical_sound_listener_field_r2d_v2_common.py is a library module")
