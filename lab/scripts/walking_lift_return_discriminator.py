"""Report-only candidate lesson; does not modify any admitted environment.

Run as ``python -m lab.scripts.walking_lift_return_discriminator``. Recorded
native states remain immutable. Synthetic counterfactuals are reward probes,
not physical trajectories or evidence that a controller can execute a gait.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)

from lab.scripts.cpu_walking_contact_audit import analyze, foot_measurements, inputs

PEAK_UM = 60_000
CYCLE_TICKS = 72
START_TICK = 120
Q16 = 65_536


def target_um(action_tick: int, moving: bool) -> tuple[int, int]:
    """Quintic up/down, left swing 6..30, right 42..66; floor to um.

    The action sees tick t; compare its resulting geometry against target(t),
    not target(t+1). These are proposed constants, NOT a runtime profile.
    """
    if action_tick < 0:
        raise ValueError("negative action tick")
    if not moving:
        return (0, 0)
    phase = max(action_tick - START_TICK, 0) % CYCLE_TICKS

    def height(p):
        if not 6 < p < 30:
            return 0
        n = min(p - 6, 30 - p)
        d = 12
        return PEAK_UM * (10 * n**3 * d**2 - 15 * n**4 * d + 6 * n**5) // d**5

    return height(phase), height((phase - 36) % CYCLE_TICKS)


def cost_q16(actual_um: tuple[int, int], action_tick: int, moving: bool) -> int:
    """Nonnegative error, coefficient would be -1; stop adds no new cost.

    Saturate each foot's absolute error at 60 mm. No reward for flight,
    contacts or safe re-contact is inferred from geometry alone.
    """
    if len(actual_um) != 2:
        raise ValueError("expected two sole heights")
    target = target_um(action_tick, moving)
    if not moving:
        return 0
    return sum(
        min(abs(int(actual) - desired), PEAK_UM) * Q16 // PEAK_UM
        for actual, desired in zip(actual_um, target, strict=True)
    )


def cycle_controls():
    """Fixed counterfactuals for a full cycle, never optimizer-selected."""
    cases = {
        name: []
        for name in (
            "exact_lift_and_return",
            "grounded_or_heel_only",
            "wrong_foot",
            "left_held_at_peak",
            "both_held_at_peak",
        )
    }
    for tick in range(START_TICK, START_TICK + CYCLE_TICKS):
        target = target_um(tick, True)
        actuals = (target, (0, 0), target[::-1], (PEAK_UM, 0), (PEAK_UM, PEAK_UM))
        for values, actual in zip(cases.values(), actuals, strict=True):
            values.append(cost_q16(actual, tick, True))
    return {
        key: {"sum_q16": sum(values), "mean": sum(values) / (Q16 * CYCLE_TICKS)}
        for key, values in cases.items()
    }


def native_summary(frames, descriptor):
    clearance, _ = foot_measurements(frames, descriptor)
    # Recorded geometry is floating diagnostic math, rounded ties-even to um.
    # Any future native implementation needs a paired integer geometry check.
    heights = np.rint(clearance * 1_000_000).astype(np.int64)
    records = []
    for frame, actual in zip(frames, heights, strict=True):
        moving = any(frame["command_raw"])
        tick = frame["tick"] - 1
        records.append(
            {
                "action_tick": tick,
                "moving": moving,
                "actual_um": actual.tolist(),
                "target_um": list(target_um(tick, moving)),
                "cost_q16": cost_q16(actual, tick, moving),
                "grounded_cost_q16": cost_q16((0, 0), tick, moving),
            }
        )
    moving = [record for record in records if record["moving"]]
    # Positive reachability tapes have zero velocity command: do not invent a
    # temporal match to this lesson. Test every recorded frame at a FIXED peak
    # target for each side, and report counts, not a cherry-picked best frame.
    peak_comparisons = []
    for side, phase in enumerate((18, 54)):
        costs = [cost_q16(actual, START_TICK + phase, True) for actual in heights]
        peak_comparisons.append(
            {
                "side": ("left", "right")[side],
                "fixed_target_phase": phase,
                "frames_better_than_grounded": sum(value < Q16 for value in costs),
                "frames_total": len(costs),
                "whole_sole_above_30mm": int(np.sum(heights[:, side] > 30_000)),
            }
        )
    return {
        "ticks": len(frames),
        "moving_ticks": len(moving),
        "moving_mean_cost": sum(r["cost_q16"] for r in moving) / (Q16 * len(moving))
        if moving
        else None,
        "moving_grounded_mean_cost": sum(r["grounded_cost_q16"] for r in moving)
        / (Q16 * len(moving))
        if moving
        else None,
        "fixed_peak_geometry_controls_not_gait": peak_comparisons,
        "records": records,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--evaluation-trace", type=Path, required=True)
    parser.add_argument("--reachability-trace", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=1001)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    require_external_path(
        args.output,
        Path(__file__).resolve().parents[2],
        label="candidate evidence",
        must_exist=False,
    )
    if args.output.exists():
        raise ValueError("refusing to replace evidence")
    manifest_path, manifest, descriptor, evaluation_path, evaluation = inputs(
        args.run, args.seed
    )
    trace = json.loads(args.evaluation_trace.read_text())
    expected_profile = (
        "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v6"
    )
    if (
        trace["profile_id"] != expected_profile
        or manifest["profile"]["environment_profile_id"] != expected_profile
    ):
        raise ValueError("this discriminator compares the closed V6 lesson")
    analyze(trace, descriptor, evaluation)  # fail closed on any replay mismatch
    reachability = json.loads(args.reachability_trace.read_text())
    if (
        reachability["profile_id"]
        != "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v5"
        or len(reachability["frames"]) != 3
        or len(reachability["cases"]) != 3
        or any(len(frames) != 105 for frames in reachability["frames"])
        or any(
            case["terminal"] is not None or case["safety_error"] is not None
            for case in reachability["cases"]
        )
    ):
        raise ValueError("expected three safe 105-tick V5 reachability cases")
    report = {
        "status": "REPORT_ONLY_CANDIDATE_NOT_TRAINING_ADMISSION",
        "claim": "geometric signal discrimination, not learnability or safe return",
        "constants": {
            "peak_um": PEAK_UM,
            "cycle_ticks": CYCLE_TICKS,
            "start_tick": START_TICK,
            "rounding": "target floor; geometry ties-even um",
            "zero_command": "no additional cost",
        },
        "inputs": {
            str(path.resolve()): sha256_file(path)
            for path in (
                manifest_path,
                Path(manifest["descriptor_path"]),
                evaluation_path,
                args.evaluation_trace,
                args.reachability_trace,
                Path(__file__),
            )
        },
        "synthetic_cycle_controls_not_physical_trajectories": cycle_controls(),
        "evaluation": native_summary(trace["frames"][0], descriptor),
        "reachability_case_outcomes": reachability["cases"],
        "reachability_geometry": [
            native_summary(frames, descriptor) for frames in reachability["frames"]
        ],
    }
    atomic_write_json(args.output, report)
    print(
        json.dumps(
            {
                "output": str(args.output),
                "sha256": sha256_file(args.output),
                "controls": cycle_controls(),
                "evaluation_mean_cost": report["evaluation"]["moving_mean_cost"],
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
