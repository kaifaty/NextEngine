"""Render a complete closed final or explicit known-candidate native trajectory."""

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

from lab.scripts.cpu_walking_contact_audit import foot_box

ROOT = Path(__file__).resolve().parents[2]
EDGES = (
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
)
COLORS = ("#2684d9", "#e6699b")


def frame_geometry(frame, descriptor):
    """Native body-origin connections and actual foot-box edges, world XYZ."""
    bodies = descriptor["bodies"]
    links = {link["body_token"]: link for link in frame["links"]}
    if len(links) != len(frame["links"]) or set(links) != {
        b["body_token"] for b in bodies
    }:
        raise ValueError("native body token set mismatch")
    positions = [
        np.asarray(links[b["body_token"]]["position_um"], dtype=float) / 1e6
        for b in bodies
    ]
    segments, colors = [], []
    feet = []
    for index, body in enumerate(bodies):
        parent = body["parent_body_slot"]
        color = (
            COLORS[0]
            if ".left-" in body["body_id"]
            else COLORS[1]
            if ".right-" in body["body_id"]
            else "#d8a600"
        )
        if parent is not None:
            if type(parent) is not int or not 0 <= parent < len(bodies):
                raise ValueError("invalid body parent")
            segments.append([positions[parent], positions[index]])
            colors.append(color)
        if body["body_id"] in ("body.left-ankle-roll", "body.right-ankle-roll"):
            corners, center, _, _ = foot_box(body, links[body["body_token"]])
            edges = np.asarray([corners[[a, b]] for a, b in EDGES])
            segments.extend(edges)
            colors.extend([color] * len(EDGES))
            feet.append((body["body_id"], edges, center))
    if len(feet) != 2:
        raise ValueError("video requires exactly two physical sole boxes")
    return np.asarray(segments), colors, sorted(feet), positions[0]


def load_closed_evaluation(evaluation, seed):
    manifest_path = evaluation / "run-manifest.json"
    manifest = json.loads(manifest_path.read_text())
    known = manifest.get("schema") == "nextengine.known-walking-candidate-run.v1"
    if known:
        if manifest.get("status") != "completed":
            raise ValueError("video requires a completed known-candidate evaluation")
        cfg = manifest["profile"]
        if (
            cfg["source_checkpoint_name"] != "model_3999.pt"
            or cfg["source_checkpoint_iteration"] != 3999
        ):
            raise ValueError("video requires the declared known candidate")
        # Read-only projection for shared rendering; never rewrite source manifest.
        manifest = {
            **manifest,
            "matrix": cfg,
            "target_descriptor_path": manifest["target_descriptor"],
            "inputs": {
                "target_descriptor_sha256": cfg["target_descriptor_sha256"],
                "source_checkpoint_sha256": cfg["source_checkpoint_sha256"],
            },
        }
    if (
        not known
        and manifest.get("schema") != "nextengine.corrected-walking-evaluation-run.v1"
        or manifest.get("status") != "completed"
    ):
        raise ValueError("video requires a completed corrected evaluation")
    if not known and manifest["matrix"]["source_checkpoint_name"] != "model_9999.pt":
        raise ValueError("video requires the declared final checkpoint")
    if seed not in manifest["matrix"]["evaluation"]["seeds"]:
        raise ValueError("seed outside declared evaluation matrix")
    paths = (
        [
            evaluation / "evaluation" / f"native-{seed}.json",
            evaluation / "evaluation" / f"support-{seed}.json",
            evaluation / "evaluation.json",
        ]
        if known
        else [
            evaluation / f"native-{seed}.json",
            evaluation / f"support-{seed}/report.json",
            evaluation / "evaluation.json",
        ]
    )
    for path in paths:
        if (
            sha256_file(path)
            != manifest["artifacts"][str(path.relative_to(evaluation))]
        ):
            raise ValueError("closed evaluation artifact hash mismatch")
    descriptor_path = Path(manifest["target_descriptor_path"])
    if sha256_file(descriptor_path) != manifest["inputs"]["target_descriptor_sha256"]:
        raise ValueError("video descriptor hash mismatch")
    trace = json.loads(paths[0].read_text())
    descriptor = json.loads(descriptor_path.read_text())
    support = json.loads(paths[1].read_text())["lifted_support"]["frames"]
    results = json.loads(paths[2].read_text())
    record = next(r for r in results["episodes"] if r["seed"] == seed)
    record = {
        **record,
        "checkpoint_label": manifest["matrix"]["source_checkpoint_name"],
    }
    if (
        len(trace["frames"]) != 1
        or trace["profile_id"] != manifest["matrix"]["target_environment_profile_id"]
    ):
        raise ValueError("video native environment/case mismatch")
    frames = trace["frames"][0]
    if len(frames) != record["ticks"] or len(support) != len(frames) or not frames:
        raise ValueError("video frame count mismatch")
    if descriptor["motor_hz"] != 60:
        raise ValueError("video requires canonical 60 Hz timing")
    for index, (frame, load) in enumerate(zip(frames, support, strict=True)):
        if frame["tick"] != index + 1 or load["tick"] != frame["tick"]:
            raise ValueError("video requires complete contiguous native frames")
    return manifest, descriptor, frames, support, record


def render(descriptor, frames, support, record, output):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from matplotlib.animation import FFMpegWriter
    from matplotlib.collections import LineCollection

    fig, axes = plt.subplots(
        2, 2, figsize=(12, 8), gridspec_kw={"height_ratios": [3, 1]}
    )
    fig.subplots_adjust(top=0.84, bottom=0.08, hspace=0.28, wspace=0.2)
    heading = fig.suptitle("", fontsize=13)
    artists = []
    for index, ax in enumerate(axes.flat):
        collection = LineCollection([], linewidths=2)
        ax.add_collection(collection)
        artists.append(collection)
        ax.axhline(0, color="black", linewidth=0.8)
        ax.set_aspect("equal")
        ax.grid(alpha=0.2)
        ax.set_ylabel("World height, m")
        if index < 2:
            ax.set_ylim(-0.08, 1.85)
            ax.set_xlabel("World forward, m" if index == 0 else "World right, m")
        else:
            ax.set(
                xlim=(-0.2, 0.2),
                ylim=(-0.02, 0.18),
                xlabel="Forward from foot centre, m",
            )
            ax.set_title("Left sole" if index == 2 else "Right sole")
    writer = FFMpegWriter(
        fps=60,
        codec="libx264",
        extra_args=["-pix_fmt", "yuv420p", "-crf", "20", "-threads", "2"],
    )
    preview_indices = {0, len(frames) // 2, len(frames) - 1}
    with writer.saving(fig, str(output / "walking.mp4"), dpi=100):
        for index, (frame, load) in enumerate(zip(frames, support, strict=True)):
            segments, colors, feet, root = frame_geometry(frame, descriptor)
            for view, horizontal in enumerate((2, 0)):
                artists[view].set_segments(segments[:, :, [horizontal, 1]])
                artists[view].set_color(colors)
                axes[0, view].set_xlim(root[horizontal] - 0.75, root[horizontal] + 0.75)
                axes[0, view].set_ylim(
                    min(-0.08, float(segments[:, :, 1].min()) - 0.02),
                    max(1.85, float(segments[:, :, 1].max()) + 0.08),
                )
            for side, (_, edges, centre) in enumerate(feet):
                projected = edges[:, :, [2, 1]].copy()
                projected[:, :, 0] -= centre[2]
                artists[side + 2].set_segments(projected)
                artists[side + 2].set_color(COLORS[side])
                axes[1, side].set_ylim(
                    min(-0.02, float(projected[:, :, 1].min()) - 0.01),
                    max(0.18, float(projected[:, :, 1].max()) + 0.01),
                )
            stance = load["lift_qualified_stance_side_report_only"]
            stance_label = "left" if stance == 0 else "right" if stance == 1 else "none"
            heading.set_text(
                f"{record['checkpoint_label']} | native body origins + physical sole boxes (not a surface mesh)\n"
                f"{frame['tick'] / 60:.2f} s | command {frame['command_raw'][1] / 1e6:.2f} m/s | "
                f"lift-qualified stance: {stance_label}\n"
                f"Full episode, seed {record['seed']} | result: {'PASS' if record['passed'] else 'FAIL'} | {record['terminal_reason']}"
            )
            writer.grab_frame()
            if index in preview_indices:
                fig.savefig(output / f"frame-{frame['tick']:04d}.png", dpi=100)
    plt.close(fig)


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--evaluation", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=1001)
    args = parser.parse_args()
    evaluation = require_external_path(args.evaluation, ROOT, label="evaluation")
    output = require_external_path(args.output, ROOT, label="video", must_exist=False)
    if output.exists() or output.is_relative_to(evaluation):
        raise ValueError("video output must be fresh and outside closed evaluation")
    manifest, descriptor, frames, support, record = load_closed_evaluation(
        evaluation, args.seed
    )
    source_hash = sha256_file(evaluation / "run-manifest.json")
    output.mkdir(parents=True, exist_ok=False)
    render(descriptor, frames, support, record, output)
    if sha256_file(evaluation / "run-manifest.json") != source_hash:
        raise ValueError("source evaluation changed during rendering")
    atomic_write_json(
        output / "video-manifest.json",
        {
            "schema": "nextengine.native-walking-video.v1",
            "source_evaluation": str(evaluation),
            "source_manifest_sha256": source_hash,
            "source_checkpoint_sha256": manifest["inputs"]["source_checkpoint_sha256"],
            "seed": args.seed,
            "frames": len(frames),
            "fps": 60,
            "duration_seconds": len(frames) / 60,
            "all_native_frames_included": True,
            "interpolated": False,
            "tool_sha256": sha256_file(Path(__file__)),
            "artifacts": {
                p.name: sha256_file(p) for p in output.iterdir() if p.is_file()
            },
        },
    )
    print(output / "walking.mp4")


if __name__ == "__main__":
    main()
