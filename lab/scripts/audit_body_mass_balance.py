"""Measure mass placement and trunk tilt from a closed native walking trace.

This diagnostic does not infer dynamic stability from a static support box.
It changes neither mass nor controller and is not an admission gate.
"""

from __future__ import annotations

import argparse
from pathlib import Path

import numpy as np
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)
from next_lab.motion_math import decode_f32_bits, quaternion_to_matrix

from lab.scripts.cpu_walking_contact_audit import foot_box
from lab.scripts.render_native_walking_video import load_closed_evaluation

ROOT = Path(__file__).resolve().parents[2]


def mass_positions(descriptor, frame=None):
    bodies = descriptor["bodies"]
    tokens = [b["body_token"] for b in bodies]
    if len(set(tokens)) != len(tokens):
        raise ValueError("duplicate descriptor body token")
    links = None if frame is None else {l["body_token"]: l for l in frame["links"]}
    if links is not None and (
        len(links) != len(frame["links"]) or set(links) != set(tokens)
    ):
        raise ValueError("native body token set mismatch")
    rows = []
    for body in bodies:
        mass = body["mass_microkilograms"] / 1e6
        if not np.isfinite(mass) or mass <= 0:
            raise ValueError("invalid body mass")
        if links is None:
            origin = decode_f32_bits(body["initial_translation_f32_bits"])
            quaternion = decode_f32_bits(body["initial_rotation_f32_bits"])
        else:
            link = links[body["body_token"]]
            origin = np.asarray(link["position_um"], dtype=float) / 1e6
            quaternion = np.asarray(link["rotation_q1_30"], dtype=float) / (1 << 30)
        local = np.asarray(body["center_of_mass_micrometres"], dtype=float) / 1e6
        if (
            origin.shape != (3,)
            or local.shape != (3,)
            or quaternion.shape != (4,)
            or not np.isfinite(np.r_[origin, local, quaternion]).all()
            or np.linalg.norm(quaternion) == 0
        ):
            raise ValueError("invalid mass transform")
        rotation = quaternion_to_matrix(quaternion)
        rows.append(
            {
                "id": body["body_id"],
                "mass": mass,
                "origin": origin,
                "com": origin + rotation @ local,
                "rotation": rotation,
                "carrier": body["non_colliding_carrier"],
            }
        )
    return rows


def center_of_mass(rows):
    masses = np.asarray([r["mass"] for r in rows])
    if not len(masses) or not np.isfinite(masses).all() or np.any(masses <= 0):
        raise ValueError("invalid mass collection")
    return (
        np.sum(np.asarray([r["com"] for r in rows]) * masses[:, None], axis=0)
        / masses.sum()
    )


def tilt_degrees(rotation):
    return float(
        np.degrees(np.arctan2(np.linalg.norm(rotation[[0, 2], 1]), rotation[1, 1]))
    )


def carrier_com_counterfactual(rows):
    """Same-pose diagnostic: relocate serial carrier mass to terminal-link COM.

    Not a simulation or dynamically equivalent merged articulation. Isolates
    only the instantaneous COM effect, leaving inertia/control effects unknown.
    """
    by_id = {r["id"]: r for r in rows}
    correction = np.zeros(3)
    mass = sum(r["mass"] for r in rows)
    for row in rows:
        if not row["carrier"]:
            continue
        prefix, axis = row["id"].rsplit("-", 1)
        if axis not in ("pitch", "roll") or prefix + "-yaw" not in by_id:
            raise ValueError("undeclared carrier projection group")
        correction += row["mass"] * (by_id[prefix + "-yaw"]["com"] - row["com"])
    return correction / mass


def analyze_mass(descriptor, frames):
    if not frames or [f["tick"] for f in frames] != list(range(1, len(frames) + 1)):
        raise ValueError("mass audit needs complete contiguous native frames")
    neutral = mass_positions(descriptor)
    total = sum(r["mass"] for r in neutral)
    groups = {"pelvis": 0.0, "torso_head": 0.0, "arms": 0.0, "legs": 0.0}
    for row in neutral:
        name = row["id"]
        key = (
            "pelvis"
            if name == "body.pelvis"
            else "torso_head"
            if name.startswith("body.torso-")
            else "arms"
            if "shoulder" in name or "elbow" in name
            else "legs"
        )
        groups[key] += row["mass"]
    samples = []
    for frame in frames:
        rows = mass_positions(descriptor, frame)
        by_id = {r["id"]: r for r in rows}
        com = center_of_mass(rows)
        links = {l["body_token"]: l for l in frame["links"]}
        feet = [
            b
            for b in descriptor["bodies"]
            if b["body_id"] in ("body.left-ankle-roll", "body.right-ankle-roll")
        ]
        if len(feet) != 2:
            raise ValueError("exact bilateral foot set required")
        # Includes swing foot: potential geometry only, not actual support.
        foot_centers = [foot_box(b, links[b["body_token"]])[1] for b in feet]
        samples.append(
            {
                "tick": frame["tick"],
                "com_world_m": com.tolist(),
                "com_minus_root_world_m": (
                    com - by_id["body.pelvis"]["origin"]
                ).tolist(),
                "com_minus_foot_midpoint_world_m": (
                    com - np.mean(foot_centers, axis=0)
                ).tolist(),
                "pelvis_tilt_deg": tilt_degrees(by_id["body.pelvis"]["rotation"]),
                "torso_tilt_deg": tilt_degrees(by_id["body.torso-yaw"]["rotation"]),
                "carrier_relocation_com_delta_m": carrier_com_counterfactual(
                    rows
                ).tolist(),
            }
        )
    return {
        "status": "report_only",
        "total_mass_kg": total,
        "group_mass_kg": groups,
        "group_mass_fraction": {k: v / total for k, v in groups.items()},
        "neutral_com_world_m": center_of_mass(neutral).tolist(),
        "neutral_carrier_relocation_com_delta_m": carrier_com_counterfactual(
            neutral
        ).tolist(),
        "maximum_same_pose_carrier_com_delta_m": float(
            max(np.linalg.norm(s["carrier_relocation_com_delta_m"]) for s in samples)
        ),
        "maximum_pelvis_tilt_deg": max(s["pelvis_tilt_deg"] for s in samples),
        "maximum_torso_tilt_deg": max(s["torso_tilt_deg"] for s in samples),
        "final": samples[-1],
        "frames": samples,
        "claim_boundary": "same-pose geometric mass accounting; not dynamic stability, loaded support, anthropometric validation or a new physical simulation",
    }


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--evaluation", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    evaluation = require_external_path(args.evaluation, ROOT, label="evaluation")
    output = require_external_path(
        args.output, ROOT, label="mass audit", must_exist=False
    )
    if output.exists() or output.is_relative_to(evaluation):
        raise ValueError("mass audit must be fresh and outside source")
    manifest, descriptor, frames, _, _ = load_closed_evaluation(evaluation, 1001)
    report = analyze_mass(descriptor, frames)
    report.update(
        {
            "source_manifest_sha256": sha256_file(evaluation / "run-manifest.json"),
            "source_descriptor_sha256": manifest["inputs"]["target_descriptor_sha256"],
            "tool_sha256": sha256_file(Path(__file__)),
            "seed": 1001,
        }
    )
    output.mkdir(parents=True, exist_ok=False)
    atomic_write_json(output / "report.json", report)
    print(output / "report.json")


if __name__ == "__main__":
    main()
