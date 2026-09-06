"""New OF2 shapes: prepare full targets, separately render frozen NN, assess.

No training or selection of examples by sound. Identity/family roles are fixed
before new audio. Evaluated model training IDs distinguish transfer from
reconstruction. Synthetic teachers and diagnostic metrics are not realism.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_expand as expansion
import physical_sound_objectfolder2_shared as shared
import physical_sound_objectfolder2_shared_assess as assess_lib
import torch
from safetensors.torch import load_file
from scipy.io import wavfile

WEIGHTS = "5b569809d3340a28f5d9aa361b29d6db9c0f0d56628e881a706e615e1074b56f"


def cohort_role(identity):
    # Conservative metadata-family guard BEFORE opening new audio: YCB065 cups
    # include IDs40/45 in provisional DEV; Top Paw53 resembles old DEV54.
    # This is a possible-family exclusion, not proof of identical geometry.
    if 37 <= identity <= 46 or identity == 53:
        return "development"
    return expansion.role(identity)


def read_npz(path, expected):
    if teacher.sha(path) != expected:
        raise ValueError("artifact changed")
    with np.load(path, allow_pickle=False) as d:
        return dict(d)


def check_geometry(values):
    if set(values) != {"cloud", "features", "contacts"}:
        raise ValueError("geometry-only input required")
    if any(
        values[k].shape != shape
        for k, shape in (("cloud", (512, 3)), ("features", (8,)), ("contacts", (32, 3)))
    ):
        raise ValueError("invalid geometry shape")
    if not all(v.dtype == np.float32 and np.isfinite(v).all() for v in values.values()):
        raise ValueError("finite float32 geometry required")
    material = values["features"][3:]
    if not np.isin(material, [0, 1]).all() or material.sum() != 1:
        raise ValueError("known one-hot material required")


def prepare(args):
    manifest = json.loads((args.expansion / "expansion.json").read_text())
    original = json.loads((args.original / "data.json").read_text())["rows"]
    old_hashes = {r["mesh_sha256"] for r in original}
    definitions = teacher.source_definitions(args.source / "AudioNet_model.py")
    args.output.mkdir(parents=True)
    rows, inputs = [], []
    for entry in manifest["rows"]:
        identity = entry["object_id"]
        if identity in expansion.EXISTING or entry["role"] != expansion.role(identity):
            raise ValueError("new role/identity mismatch")
        mesh_record, model_record = (
            entry["files"]["model.obj"],
            entry["files"]["ObjectFile.pth"],
        )
        mesh = Path(mesh_record["path"])
        if teacher.sha(mesh) != mesh_record["sha256"]:
            raise ValueError("mesh changed")
        row = {
            k: entry[k]
            for k in ("object_id", "role", "name", "material", "mesh_source")
        }
        row["acquisition_role"] = row["role"]
        row["role"] = cohort_role(identity)
        row.update(
            mesh_sha256=mesh_record["sha256"],
            source_model_sha256=model_record["sha256"],
        )
        row["duplicate_exact_mesh"] = mesh_record["sha256"] in old_hashes
        old_hashes.add(mesh_record["sha256"])
        if entry["material"] not in shared.MATERIALS:
            row["status"] = "UNTRAINED_MATERIAL_VOCABULARY"
            rows.append(row)
            print(identity, row["status"], flush=True)
            continue
        vertices = np.array(
            [
                [float(x) for x in line.split()[1:]]
                for line in mesh.read_text().splitlines()
                if line.startswith("v ")
            ]
        )
        cloud, features, center, length = shared.geometry(vertices, entry["material"])
        audio = teacher.load_audio(Path(model_record["path"]), model_record["sha256"])
        norm = audio["normalizer"]
        support = np.flatnonzero(
            np.all(
                (vertices >= norm["xyz_min"]) & (vertices <= norm["xyz_max"]), axis=1
            )
        )
        if len(support) < 32:
            raise ValueError("not enough supported geometry points")
        ids = support[np.linspace(0, len(support) - 1, 32).astype(int)]
        gains = teacher.point_gains(audio, definitions, vertices[ids])
        geometry = {
            "cloud": cloud,
            "features": features,
            "contacts": ((vertices[ids] - center) / length).astype(np.float32),
        }
        check_geometry(geometry)
        target = {
            **geometry,
            "frequency": audio["frequencies"],
            "damping": audio["dampings"],
            "gains": gains,
        }
        name = f"object-{identity}.npz"
        np.savez(args.output / name, **target)
        input_name = f"object-{identity}-geometry.npz"
        np.savez(args.output / input_name, **geometry)
        row.update(
            status="PREPARED",
            file=name,
            sha256=teacher.sha(args.output / name),
            modes=len(audio["frequencies"]),
            length=float(length),
            query_vertex_indices=ids.tolist(),
            out_of_support_vertices=len(vertices) - len(support),
        )
        inputs.append(
            {k: row[k] for k in ("object_id", "role", "material", "name")}
            | {"file": input_name, "sha256": teacher.sha(args.output / input_name)}
        )
        rows.append(row)
        print("prepared", identity, row["modes"], entry["material"], flush=True)
    shared.write_json(
        args.output / "data.json",
        {
            "rows": rows,
            "scope": "complete full modal targets; new ID-only roles; no protected evaluation; exact mesh duplicates flagged, not full semantic identity proof",
        },
    )
    shared.write_json(args.output / "inputs.json", {"rows": inputs})


def render(args, *, model_class=shared.SharedStudent, prediction_fn=shared.predict):
    if teacher.sha(args.fit / "model.safetensors") != args.weights_sha256:
        raise ValueError("explicitly pinned student required")
    fit = json.loads((args.fit / "fit.json").read_text())
    if fit["weights_sha256"] != args.weights_sha256:
        raise ValueError("fit weight identity mismatch")
    model = model_class()
    model.load_state_dict(load_file(args.fit / "model.safetensors"), strict=True)
    model.eval().requires_grad_(False)
    inputs = json.loads((args.data / "inputs.json").read_text())["rows"]
    args.output.mkdir(parents=True)
    rows = []
    for row in inputs:
        geometry = read_npz(args.data / row["file"], row["sha256"])
        check_geometry(geometry)
        predicted = prediction_fn(
            model, geometry["cloud"], geometry["features"], geometry["contacts"]
        )
        name = f"object-{row['object_id']}.npz"
        np.savez(args.output / name, **predicted)
        waves = [
            teacher.waveform(
                predicted["gains"][c],
                predicted["frequency"],
                predicted["damping"],
                [1, 1, 1],
            )
            for c in range(4)
        ]
        peak = max(float(abs(w).max()) for w in waves)
        gain = min(1.0, 0.5 / peak) if peak else 1.0
        files = []
        for c, wave in enumerate(waves):
            filename = f"object-{row['object_id']}-contact-{c}.wav"
            wavfile.write(
                args.output / filename, teacher.RATE, (wave * gain).astype(np.float32)
            )
            files.append(
                {"file": filename, "sha256": teacher.sha(args.output / filename)}
            )
        rows.append(
            {
                **row,
                "file": name,
                "sha256": teacher.sha(args.output / name),
                "common_gain": gain,
                "waves": files,
            }
        )
        print("standalone", row["object_id"], len(predicted["frequency"]), flush=True)
    shared.write_json(
        args.output / "render.json",
        {
            "rows": rows,
            "weights_sha256": args.weights_sha256,
            "train_ids": fit["train_ids"],
            "script_sha256": teacher.sha(Path(__file__)),
            "scope": "pinned compact NN in inference mode, geometry/own weights only; no target acoustic input",
        },
    )


def assess(args):
    targets = json.loads((args.data / "data.json").read_text())["rows"]
    generated = json.loads((args.generated / "render.json").read_text())
    if generated["weights_sha256"] != args.weights_sha256:
        raise ValueError("unexpected evaluated weights")
    by_id = {r["object_id"]: r for r in generated["rows"]}
    original = json.loads((args.original / "data.json").read_text())["rows"]
    train_rows = {r["object_id"]: r for r in original if r["role"] == "train"}
    if set(train_rows) != shared.TRAIN:
        raise ValueError("six original TRAIN neighbors required")
    train = {
        i: read_npz(args.original / r["file"], r["sha256"])
        for i, r in train_rows.items()
    }
    size_std = np.maximum(
        np.stack([train[i]["features"][:3] for i in sorted(train)]).std(0, ddof=1), 1e-6
    )
    args.output.mkdir(parents=True)
    results, previews = [], []
    for row in targets:
        identity = row["object_id"]
        if row["status"] != "PREPARED":
            continue
        source = read_npz(args.data / row["file"], row["sha256"])
        gen = by_id[identity]
        predicted = read_npz(args.generated / gen["file"], gen["sha256"])
        choices = sorted(
            i for i in train if train_rows[i]["material"] == row["material"]
        )
        neighbor = min(
            choices,
            key=lambda i: assess_lib.geometry_distance(source, train[i], size_std),
        )
        for contact in range(4):
            waves = {
                "reference": teacher.waveform(
                    source["gains"][contact],
                    source["frequency"],
                    source["damping"],
                    [1, 1, 1],
                ),
                "neural": teacher.waveform(
                    predicted["gains"][contact],
                    predicted["frequency"],
                    predicted["damping"],
                    [1, 1, 1],
                ),
            }
            standalone = gen["waves"][contact]
            path = args.generated / standalone["file"]
            rate, pcm = wavfile.read(path)
            if (
                teacher.sha(path) != standalone["sha256"]
                or rate != teacher.RATE
                or not np.array_equal(
                    pcm, (waves["neural"] * gen["common_gain"]).astype(np.float32)
                )
            ):
                raise ValueError("standalone replay mismatch")
            nc = int(
                np.linalg.norm(
                    train[neighbor]["contacts"] - source["contacts"][contact], axis=1
                ).argmin()
            )
            ratio = train_rows[neighbor]["length"] / row["length"]
            waves["nearest"], _ = assess_lib.scaled_response(train[neighbor], nc, 1.0)
            waves["size_scaled"], omitted = assess_lib.scaled_response(
                train[neighbor], nc, ratio
            )
            metrics = {
                kind: assess_lib.audio_metrics(waves["reference"], waves[kind])
                for kind in ("neural", "nearest", "size_scaled")
            }
            gain = min(1.0, 0.5 / max(float(abs(w).max()) for w in waves.values()))
            name = f"object-{identity}-contact-{contact}-comparison.wav"
            wavfile.write(
                args.output / name,
                teacher.RATE,
                (np.concatenate(list(waves.values())) * gain).astype(np.float32),
            )
            if contact == 0:
                previews.append(
                    (
                        np.concatenate([waves["reference"], waves["neural"]]) * gain
                    ).astype(np.float32)
                )
            results.append(
                {
                    "object_id": identity,
                    "role": row["role"],
                    "contact": contact,
                    "neighbor": neighbor,
                    "neighbor_contact": nc,
                    "size_ratio": ratio,
                    "scaled_modes_omitted": omitted,
                    "comparison": name,
                    "common_gain": gain,
                    **metrics,
                }
            )
        print("assessed", identity, results[-1]["neural"], flush=True)
    summary = {}
    for role in ("expanded_cohort", "train", "development"):
        selected = [
            r for r in results if role == "expanded_cohort" or r["role"] == role
        ]
        summary[role] = (
            {
                kind: {
                    metric: float(np.mean([r[kind][metric] for r in selected]))
                    for metric in ("spectrum", "envelope", "level")
                }
                for kind in ("neural", "nearest", "size_scaled")
            }
            if selected
            else None
        )
    wavfile.write(
        args.output / "new-shapes-reference-neural.wav",
        teacher.RATE,
        np.concatenate(previews),
    )
    shared.write_json(
        args.output / "assessment.json",
        {
            "rows": results,
            "summary": summary,
            "prepared_objects": len(by_id),
            "all_downloaded_pairs": len(targets),
            "unsupported": [r for r in targets if r["status"] != "PREPARED"],
            "exact_mesh_duplicates": [
                r["object_id"] for r in targets if r["duplicate_exact_mesh"]
            ],
            "weights_sha256": args.weights_sha256,
            "evaluated_train_ids": generated.get("train_ids", sorted(shared.TRAIN)),
            "gallery_order": "sorted new IDs, first contact each: full synthetic teacher then frozen NN; full comparison reference/neural/nearest/size-scaled",
            "scope": "descriptive OF2 comparison, not perceptual validation or engine admission; inspect evaluated_train_ids to distinguish reconstruction from transfer",
        },
    )
    print(json.dumps(summary, indent=2), flush=True)


def compare(args):
    """Same-gain audible before/after on ALL new supported DEV bodies."""
    data = json.loads((args.data / "data.json").read_text())["rows"]
    roots = (args.baseline, args.generated)
    manifests = [json.loads((root / "render.json").read_text()) for root in roots]
    baseline_weights = args.baseline_weights_sha256
    if [m["weights_sha256"] for m in manifests] != [
        baseline_weights,
        args.weights_sha256,
    ]:
        raise ValueError("before/after weight identities changed")
    indices = [{r["object_id"]: r for r in m["rows"]} for m in manifests]
    selected = [
        r for r in data if r["role"] == "development" and r["status"] == "PREPARED"
    ]
    if {r["object_id"] for r in selected} != {37, 40, 53}:
        raise ValueError("all three new supported DEV required")
    args.output.mkdir(parents=True)
    previews, rows = [], []
    label = (
        "reference-six-eleven"
        if baseline_weights == WEIGHTS
        else "reference-baseline-candidate"
    )
    for row in selected:
        identity = row["object_id"]
        parameters = [read_npz(args.data / row["file"], row["sha256"])]
        for root, index in zip(roots, indices, strict=True):
            r = index[identity]
            parameters.append(read_npz(root / r["file"], r["sha256"]))
        waves = [
            teacher.waveform(p["gains"][0], p["frequency"], p["damping"], [1, 1, 1])
            for p in parameters
        ]
        gain = min(1.0, 0.5 / max(float(abs(w).max()) for w in waves))
        pcm = (np.concatenate(waves) * gain).astype(np.float32)
        name = f"object-{identity}-{label}.wav"
        wavfile.write(args.output / name, teacher.RATE, pcm)
        previews.append(pcm)
        rows.append(
            {
                "object_id": identity,
                "file": name,
                "common_gain": gain,
                "sha256": teacher.sha(args.output / name),
            }
        )
    wavfile.write(
        args.output / f"new-development-{label}.wav",
        teacher.RATE,
        np.concatenate(previews),
    )
    shared.write_json(
        args.output / "comparison.json",
        {
            "rows": rows,
            "weights_sha256": [baseline_weights, args.weights_sha256],
            "order": "37/40/53, each reference then pinned baseline NN then pinned candidate NN,3seconds per item",
            "scope": "all three new supported DEV objects but TWO conservative families; same gain per triple, not calibrated loudness across objects; no target acoustics at either NN generation",
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("prepare", "render", "assess", "compare"))
    for name in (
        "expansion",
        "original",
        "source",
        "data",
        "fit",
        "generated",
        "baseline",
    ):
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--weights-sha256", default=WEIGHTS)
    parser.add_argument("--baseline-weights-sha256", default=WEIGHTS)
    args = parser.parse_args()
    torch.set_num_threads(4)
    {"prepare": prepare, "render": render, "assess": assess, "compare": compare}[
        args.stage
    ](args)
