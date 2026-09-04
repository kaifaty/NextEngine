"""Verify V7 native tapes against immutable predecessor physics and height oracle."""

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

from lab.scripts.cpu_walking_contact_audit import foot_measurements
from lab.scripts.walking_lift_return_discriminator import cost_q16


def verify(source, successor, descriptor):
    if (
        successor["profile_id"]
        != "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v7"
    ):
        raise ValueError("expected native V7")
    if source["run_root"] != successor["run_root"]:
        raise ValueError("run root mismatch")
    if len(source["frames"]) != len(successor["frames"]):
        raise ValueError("case count mismatch")
    max_error_um = 0.0
    ticks = 0
    for old, new in zip(source["frames"], successor["frames"], strict=True):
        if len(old) != len(new):
            raise ValueError("physical horizon mismatch")
        heights, _ = foot_measurements(new, descriptor)
        for a, b, height in zip(old, new, heights, strict=True):
            for key in (
                "tick",
                "command_raw",
                "contact_flags",
                "links",
                "contacts",
                "joint_position_urad",
                "physics_root",
            ):
                if a[key] != b[key]:
                    raise ValueError(f"physical mismatch at {b['tick']}: {key}")
            if a["reward_components_q16"] != b["reward_components_q16"][:11]:
                raise ValueError("legacy reward component mismatch")
            raw = b["observation_raw"]
            if len(raw) != 88 or len(b["reward_components_q16"]) != 13:
                raise ValueError("V7 layout mismatch")
            if (
                "observation_raw" in a
                and a["observation_raw"] != raw[: len(a["observation_raw"])]
            ):
                raise ValueError("legacy observation mismatch")
            error = float(np.max(np.abs(np.asarray(raw[86:]) - height * 1_000_000)))
            max_error_um = max(max_error_um, error)
            if error > 1.01:
                raise ValueError(
                    "integer geometry exceeds 1 um floor/rotation agreement"
                )
            expected = cost_q16(tuple(raw[86:]), b["tick"] - 1, any(b["command_raw"]))
            if sum(b["reward_components_q16"][11:]) != expected:
                raise ValueError("native/Python phase cost mismatch")
            if any(not 0 <= cost <= 65536 for cost in b["reward_components_q16"][11:]):
                raise ValueError("per-foot protocol cost bound")
            ticks += 1
    for a, b in zip(source["cases"], successor["cases"], strict=True):
        if {k: v for k, v in a.items() if k != "step_root"} != {
            k: v for k, v in b.items() if k != "step_root"
        }:
            raise ValueError("terminal/safety outcome mismatch")
    return {
        "status": "passed",
        "ticks": ticks,
        "maximum_geometry_error_um": max_error_um,
        "claim": "identical-action physics/safety and native height-cost agreement; not learned gait",
    }


def main():
    parser = argparse.ArgumentParser(__doc__)
    for name in ("source", "successor", "descriptor", "output"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    args = parser.parse_args()
    require_external_path(
        args.output,
        Path(__file__).resolve().parents[2],
        label="native comparison",
        must_exist=False,
    )
    if args.output.exists():
        raise ValueError("output must be new")
    report = verify(
        *(
            json.loads(path.read_text())
            for path in (
                args.source,
                args.successor,
                args.descriptor,
            )
        )
    )
    report["inputs"] = {
        str(path.resolve()): sha256_file(path)
        for path in (
            args.source,
            args.successor,
            args.descriptor,
            Path(__file__),
        )
    }
    atomic_write_json(args.output, report)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
