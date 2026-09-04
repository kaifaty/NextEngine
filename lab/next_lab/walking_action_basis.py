from __future__ import annotations

import hashlib
from typing import Any

import numpy as np


ACTION_BASIS_PROFILE_ID = (
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v5"
)
ACTION_BASIS_CHECK_ID = "WALKING-ACTION-REACHABILITY-P0"
ACTION_BASIS_CASES = ("zero", "left-swing", "right-swing")
ACTION_BASIS_MOTOR_STEPS = 105
ACTION_WIDTH = 23
Q1_30_ONE = 1 << 30
MINIMUM_SINGLE_SUPPORT_TICKS = 8
MINIMUM_FORWARD_DISPLACEMENT_M = 0.01

_BILATERAL_ACTION_PAIRS = (
    (0, 10),
    (1, 11),
    (2, 12),
    (3, 13),
    (4, 14),
    (5, 15),
    (6, 16),
    (7, 17),
    (8, 18),
    (9, 19),
)


def walking_action_basis_tape() -> np.ndarray:
    """Return the frozen zero/left/right Q1.30 reachability tape.

    The source values are float32 policy values, so converting the resulting
    integer tape back through Isaac's float32 action input is byte-exact.
    """

    prep = np.zeros(ACTION_WIDTH, dtype=np.float32)
    peak = np.zeros(ACTION_WIDTH, dtype=np.float32)
    for index, value in {
        1: -0.081,
        4: -0.022,
        11: -0.008,
        14: -0.077,
        21: 0.106,
    }.items():
        prep[index] = np.float32(value)
    peak[:] = prep
    for index, value in {0: -0.834, 3: 0.700, 4: 0.385, 6: 0.796}.items():
        peak[index] = np.float32(value)

    left = np.empty((ACTION_BASIS_MOTOR_STEPS, ACTION_WIDTH), dtype=np.float32)
    for index in range(ACTION_BASIS_MOTOR_STEPS):
        tick = index + 1
        if tick <= 40:
            value = prep * np.float32(tick / 40.0)
        elif tick <= 55:
            value = prep
        elif tick <= 70:
            alpha = np.float32((tick - 55) / 15.0)
            value = prep + (peak - prep) * alpha
        else:
            value = peak
        left[index] = value

    left_raw = np.rint(left.astype(np.float64) * Q1_30_ONE).astype(np.int64)
    right_raw = mirror_action_tape(left_raw)
    zero_raw = np.zeros_like(left_raw)
    tape = np.stack((zero_raw, left_raw, right_raw))
    round_trip = np.rint(
        tape.astype(np.float32).astype(np.float64) / Q1_30_ONE * Q1_30_ONE
    ).astype(np.int64)
    if not np.array_equal(tape, round_trip):
        raise AssertionError("walking action tape is not float32/Q1.30 reversible")
    return tape


def mirror_action_tape(action_raw: np.ndarray) -> np.ndarray:
    mirrored = np.array(action_raw, dtype=np.int64, copy=True)
    for left, right in _BILATERAL_ACTION_PAIRS:
        mirrored[..., left] = action_raw[..., right]
        mirrored[..., right] = action_raw[..., left]
    mirrored[..., 21] = -action_raw[..., 21]
    mirrored[..., 22] = -action_raw[..., 22]
    return mirrored


def action_tape_sha256(action_raw: np.ndarray) -> str:
    canonical = np.ascontiguousarray(action_raw, dtype="<i8")
    return hashlib.sha256(canonical.tobytes(order="C")).hexdigest()


def validate_action_basis_tape(action_raw: np.ndarray) -> None:
    if action_raw.dtype != np.dtype("int64") or not np.array_equal(
        action_raw, walking_action_basis_tape()
    ):
        raise ValueError("action tape does not match the frozen reachability probe")


def evaluate_action_basis(
    *,
    contact_occupancy: np.ndarray,
    root_position_m: np.ndarray,
    done: np.ndarray,
) -> dict[str, Any]:
    contacts = np.asarray(contact_occupancy, dtype=np.bool_)
    positions = np.asarray(root_position_m, dtype=np.float64)
    ended = np.asarray(done, dtype=np.bool_)
    if contacts.ndim != 3 or not 1 <= contacts.shape[1] <= ACTION_BASIS_MOTOR_STEPS:
        raise ValueError("invalid observed action-basis horizon")
    observed_steps = contacts.shape[1]
    expected_shape = (len(ACTION_BASIS_CASES), observed_steps)
    if contacts.shape != (*expected_shape, 2):
        raise ValueError(f"contact occupancy must have shape {(*expected_shape, 2)}")
    if positions.shape != (*expected_shape, 3):
        raise ValueError(f"root position must have shape {(*expected_shape, 3)}")
    if ended.shape != expected_shape:
        raise ValueError(f"done must have shape {expected_shape}")
    if not np.isfinite(positions).all():
        raise ValueError("root position contains non-finite values")

    summaries: dict[str, dict[str, Any]] = {}
    expected_support = {
        "left-swing": np.array([False, True]),
        "right-swing": np.array([True, False]),
    }
    for case_index, case_id in enumerate(ACTION_BASIS_CASES):
        case_contacts = contacts[case_index]
        case_done = ended[case_index]
        forward_displacement = float(
            positions[case_index, -1, 2] - positions[case_index, 0, 2]
        )
        record: dict[str, Any] = {
            "ended": bool(np.any(case_done)),
            "first_done_tick": (
                int(np.flatnonzero(case_done)[0] + 1) if np.any(case_done) else None
            ),
            "forward_displacement_m": forward_displacement,
            "two_sole_ticks": int(np.sum(np.all(case_contacts, axis=1))),
        }
        if case_id != "zero":
            matches = np.all(case_contacts == expected_support[case_id], axis=1)
            record["single_support_ticks"] = int(np.sum(matches))
            record["longest_single_support_run"] = _longest_true_run(matches)
        summaries[case_id] = record

    gates = {
        "complete_horizon": observed_steps == ACTION_BASIS_MOTOR_STEPS,
        "no_early_terminal": not bool(np.any(ended)),
        "zero_control_two_sole_contact": summaries["zero"]["two_sole_ticks"]
        >= observed_steps - 1,
        "left_single_support": summaries["left-swing"]["longest_single_support_run"]
        >= MINIMUM_SINGLE_SUPPORT_TICKS,
        "right_single_support": summaries["right-swing"]["longest_single_support_run"]
        >= MINIMUM_SINGLE_SUPPORT_TICKS,
        "left_forward_progress": summaries["left-swing"]["forward_displacement_m"]
        >= MINIMUM_FORWARD_DISPLACEMENT_M,
        "right_forward_progress": summaries["right-swing"]["forward_displacement_m"]
        >= MINIMUM_FORWARD_DISPLACEMENT_M,
    }
    return {
        "check": ACTION_BASIS_CHECK_ID,
        "status": "passed" if all(gates.values()) else "failed",
        "claim": "ActionReachabilityOnly",
        "profile_id": ACTION_BASIS_PROFILE_ID,
        "cases": summaries,
        "gates": gates,
        "motor_steps": ACTION_BASIS_MOTOR_STEPS,
        "observed_motor_steps": observed_steps,
        "minimum_single_support_ticks": MINIMUM_SINGLE_SUPPORT_TICKS,
        "minimum_forward_displacement_m": MINIMUM_FORWARD_DISPLACEMENT_M,
    }


def _longest_true_run(values: np.ndarray) -> int:
    longest = 0
    current = 0
    for value in values:
        current = current + 1 if bool(value) else 0
        longest = max(longest, current)
    return longest
