"""Audit a closed walking body's proportions and sole mechanics without training."""

from __future__ import annotations

import argparse
from pathlib import Path

import numpy as np
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)

from lab.scripts.cpu_walking_contact_audit import foot_measurements, plot, sole_support
from lab.scripts.native_body_geometry import neutral_proportions, physical_geometry
from lab.scripts.render_native_walking_video import (
    frame_geometry,
    load_closed_evaluation,
)

ROOT = Path(__file__).resolve().parents[2]


def plot_comparison(descriptor, frame, output):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from matplotlib.collections import LineCollection

    proportions = neutral_proportions(descriptor)
    fig, axes = plt.subplots(1, 4, figsize=(15, 7), sharey=True)
    fig.subplots_adjust(top=0.82, bottom=0.18, wspace=0.18)
    old_segments, old_colors, _, root = frame_geometry(frame, descriptor)
    for index, ax in enumerate(axes):
        horizontal = 2 if index == 3 else 0
        if index == 0:
            segments, colors, origin = old_segments, old_colors, root
        else:
            shapes, origins = physical_geometry(
                descriptor, None if index > 1 else frame
            )
            segments = np.concatenate([s["segments"] for s in shapes])
            colors = [s["color"] for s in shapes for _ in s["segments"]]
            origin = origins["body.pelvis"]
        points = segments[:, :, [horizontal, 1]].copy()
        points[:, :, 0] -= origin[horizontal]
        ax.add_collection(LineCollection(points, colors=colors, linewidths=1.2))
        ax.set(xlim=(-0.65, 0.65), ylim=(-0.03, 1.82), aspect="equal")
        ax.axhline(0, color="black", linewidth=0.8)
        ax.grid(alpha=0.2)
        ax.set_xlabel("Forward, m" if index == 3 else "Right, m")
        ax.set_title(
            (
                "Old origin lines",
                "Same recorded frame",
                "Initial body: front",
                "Initial body: side",
            )[index]
        )
        if index > 1:
            for name in ("hip", "shoulder"):
                height = proportions["sides"]["left"][f"{name}_height_m"]
                ax.axhline(height, color="#555555", linestyle=":", linewidth=0.7)
                if index == 3:
                    ax.text(
                        0.02,
                        height + 0.012,
                        f"{name}: {height * 100:.1f} cm",
                        fontsize=8,
                    )
    axes[0].set_ylabel("World height, m")
    fig.suptitle(
        "Body proportions: missing torso/head in the old stick view\nPhysical colliders from the unchanged descriptor; not a skin or anatomical bone mesh",
        fontsize=14,
    )
    side = proportions["sides"]["left"]
    fig.text(
        0.5,
        0.06,
        f"Stature {proportions['stature_m'] * 100:.1f} cm | hip height {side['hip_height_m'] * 100:.1f} cm | "
        f"hip to shoulder {side['hip_to_shoulder_vertical_m'] * 100:.1f} cm\n"
        "Left = blue, right = pink. Initial body is not a learned standing result. No physical dimensions changed.",
        ha="center",
        fontsize=11,
    )
    fig.savefig(output / "body-proportions.png", dpi=150)
    plt.close(fig)


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--evaluation", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=1001)
    args = parser.parse_args()
    evaluation = require_external_path(args.evaluation, ROOT, label="evaluation")
    output = require_external_path(args.output, ROOT, label="audit", must_exist=False)
    if output.exists() or output.is_relative_to(evaluation):
        raise ValueError("audit output must be fresh and outside source evaluation")
    manifest, descriptor, frames, _, _ = load_closed_evaluation(evaluation, args.seed)
    source_hash = sha256_file(evaluation / "run-manifest.json")
    report = neutral_proportions(descriptor)
    report.update(
        {
            "source_evaluation_manifest_sha256": source_hash,
            "source_descriptor_sha256": manifest["inputs"]["target_descriptor_sha256"],
            "native_ticks": len(frames),
            "sole_mechanics": sole_support(frames, descriptor),
            "physics_or_weights_changed": False,
            "tools": {
                name: sha256_file(Path(__file__).with_name(name))
                for name in (
                    "audit_body_proportions.py",
                    "native_body_geometry.py",
                    "render_native_walking_video.py",
                    "cpu_walking_contact_audit.py",
                )
            },
        }
    )
    output.mkdir(parents=True, exist_ok=False)
    plot_comparison(descriptor, frames[0], output)
    root_token = next(
        b["body_token"] for b in descriptor["bodies"] if b["body_id"] == "body.pelvis"
    )
    plot_evaluation = {
        "command": np.asarray([f["command_raw"] for f in frames]) / 1e6,
        "root_velocity_mps": np.asarray(
            [
                next(
                    link["linear_velocity_um_s"]
                    for link in f["links"]
                    if link["body_token"] == root_token
                )
                for f in frames
            ]
        )
        / 1e6,
    }
    clearance, impulses = foot_measurements(frames, descriptor)
    plot(descriptor, plot_evaluation, {"frames": [frames]}, clearance, impulses, output)
    report["artifacts"] = {
        p.name: sha256_file(p) for p in output.iterdir() if p.is_file()
    }
    if sha256_file(evaluation / "run-manifest.json") != source_hash:
        raise ValueError("source evaluation changed during audit")
    atomic_write_json(output / "report.json", report)
    print(output / "body-proportions.png")


if __name__ == "__main__":
    main()
