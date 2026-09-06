"""Real-recording controls for existing acoustic metrics; never a realism gate."""

import argparse
import collections
import json
from pathlib import Path

import numpy as np
import physical_sound_syncfusion_adapter as adapter
import physical_sound_syncfusion_condition as oracle
import physical_sound_syncfusion_pilot as pilot
import soundfile as sf
import torch

MATERIALS = adapter.MATERIALS


def validate_rows(rows):
    roles = collections.defaultdict(set)
    for row in rows:
        if row["material"] not in MATERIALS or row["role"] not in (
            "train",
            "recording-dev",
            "combination-dev",
        ):
            raise ValueError("Unsupported label/role")
        roles[row["recording"]].add(row["role"])
    if any(len(value) != 1 for value in roles.values()):
        raise ValueError("Recording leakage")


def bank(features, rows, pooling, excluded=None):
    validate_rows(rows)
    if pooling not in ("material", "recording"):
        raise ValueError("Unknown pooling")
    if (
        features.ndim != 2
        or len(features) != len(rows)
        or not np.isfinite(features).all()
    ):
        raise ValueError("Invalid feature matrix")
    groups = collections.defaultdict(list)
    for i, row in enumerate(rows):
        if row["role"] == "train" and row["recording"] != excluded:
            groups[(row["material"], row["recording"])].append(i)
    vectors, labels = [], []
    for material in MATERIALS:
        selected = [
            features[indices].mean(0)
            for (m, _), indices in sorted(groups.items())
            if m == material
        ]
        if not selected:
            raise ValueError("Missing TRAIN material after recording exclusion")
        for vector in (
            [np.mean(selected, axis=0)] if pooling == "material" else selected
        ):
            vectors.append(vector)
            labels.append(material)
    return np.stack(vectors), labels


def classify(features, prototypes, labels, representation):
    if representation == "shape":
        distance = np.abs(features[:, None] - prototypes[None]).mean(-1)
    elif representation == "embedding":

        def norm(x):
            return x / np.maximum(np.linalg.norm(x, axis=-1, keepdims=True), 1e-12)

        distance = 1 - norm(features) @ norm(prototypes).T
    else:
        raise ValueError("Unknown representation")
    if not np.isfinite(distance).all():
        raise ValueError("Invalid distance")
    return [labels[i] for i in distance.argmin(-1)]


def metrics(rows, predicted):
    if len(rows) != len(predicted) or not rows:
        raise ValueError("Empty or mismatched assessment")
    confusion = np.zeros((3, 3), dtype=int)
    records = collections.defaultdict(list)
    for row, prediction in zip(rows, predicted, strict=True):
        confusion[MATERIALS.index(row["material"]), MATERIALS.index(prediction)] += 1
        records[(row["material"], row["recording"])].append(
            prediction == row["material"]
        )
    recall = {
        m: float(
            np.mean(
                [np.mean(v) for (material, _), v in records.items() if material == m]
            )
        )
        if any(material == m for material, _ in records)
        else None
        for m in MATERIALS
    }
    return {
        "event_confusion_true_rows_predicted_columns": confusion.tolist(),
        "label_order": list(MATERIALS),
        "events": len(rows),
        "recordings": len({r["recording"] for r in rows}),
        "material_recording_support": {
            m: sum(material == m for material, _ in records) for m in MATERIALS
        },
        "recording_balanced_recall": recall,
        "macro_recall": float(np.mean(list(recall.values())))
        if all(v is not None for v in recall.values())
        else None,
        "recording_details": [
            {
                "material": m,
                "recording": key,
                "events": len(v),
                "recall": float(np.mean(v)),
            }
            for (m, key), v in sorted(records.items())
        ],
    }


def run(data, generated, assets, package, output, reference_data=None):
    report = json.loads((data / "data.json").read_text())
    if (
        report["status"] != "complete"
        or pilot.sha(data / "embeddings.pt") != report["embeddings_sha256"]
    ):
        raise ValueError("Changed real feature cache")
    rows = report["rows"]
    validate_rows(rows)
    output.mkdir(parents=True, exist_ok=False)
    shapes, waves = [], []
    for row in rows:
        if pilot.sha(Path(row["wav"])) != row["sha256"]:
            raise ValueError("Changed real recording")
        wave, rate = sf.read(row["wav"], dtype="float32")
        if rate != pilot.RATE:
            raise ValueError("Real sample rate")
        shapes.append(oracle.spectral_shape(wave[:9600]))
        waves.append(wave)
    features = {
        "shape": np.stack(shapes),
        "embedding": torch.load(data / "embeddings.pt", weights_only=True).numpy(),
    }
    reference_rows = rows
    if reference_data is not None:
        reference = json.loads((reference_data / "data.json").read_text())
        if reference["status"] != "complete":
            raise ValueError("Incomplete fixed references")
        reference_rows = adapter.fixed_reference_rows(rows, reference["rows"])
    held = [i for i, r in enumerate(reference_rows) if r["role"] != "train"]
    result = {
        "status": "real_controls_complete_generation_pending",
        "claim": "diagnostic only; no realism/material acceptance authority; recording-held, not known novel objects",
        "data_sha256": pilot.sha(data / "data.json"),
        "reference_data_sha256": pilot.sha((reference_data or data) / "data.json"),
        "methods": {},
        "auditions": [],
        "generated": [],
    }
    fitted = {}
    for representation, values in features.items():
        for pooling in ("material", "recording"):
            name = representation + "/" + pooling
            prototypes, labels = bank(values, rows, pooling)
            fitted[name] = (prototypes, labels)
            prediction = classify(values[held], prototypes, labels, representation)
            held_rows = [rows[i] for i in held]
            training_rows, training_predictions = [], []
            for key in sorted({r["recording"] for r in rows if r["role"] == "train"}):
                indices = [i for i, r in enumerate(rows) if r["recording"] == key]
                p, names = bank(values, rows, pooling, excluded=key)
                training_rows.extend(rows[i] for i in indices)
                training_predictions.extend(
                    classify(values[indices], p, names, representation)
                )
            result["methods"][name] = {
                "train_leave_recording_out": metrics(
                    training_rows, training_predictions
                ),
                "held": metrics(held_rows, prediction),
                "held_predictions": [
                    {"row": i, "target": rows[i]["material"], "prediction": p}
                    for i, p in zip(held, prediction, strict=True)
                ],
            }
            # First real success/failure per material, never auditory cherry-picking.
            for material in MATERIALS:
                order, parts = [], []
                for correct in (True, False):
                    chosen = next(
                        (
                            i
                            for i, p in zip(held, prediction, strict=True)
                            if rows[i]["material"] == material
                            and (p == material) == correct
                        ),
                        None,
                    )
                    if chosen is None:
                        continue
                    sound = waves[chosen] * 0.5
                    entry = {
                        "row": chosen,
                        "correct": correct,
                        "prediction": prediction[held.index(chosen)],
                    }
                    if np.abs(sound).max() >= 0.98:
                        order.append(dict(entry, status="not_published_headroom"))
                        continue
                    parts.extend([sound, np.zeros(pilot.RATE // 2)])
                    order.append(dict(entry, status="published"))
                if parts:
                    path = (
                        output
                        / f"{representation}-{pooling}-{material}-real-controls.wav"
                    )
                    sf.write(path, np.concatenate(parts), pilot.RATE, subtype="PCM_16")
                    result["auditions"].append(
                        {
                            "method": name,
                            "target": material,
                            "order": order,
                            "wav": str(path),
                            "sha256": pilot.sha(path),
                        }
                    )
    pilot.save(output / "result.json", result)
    if pilot.sha(assets / "model.ckpt") != report["checkpoint_sha256"]:
        raise ValueError("Changed frozen audio encoder")
    torch.set_num_threads(4)
    state = torch.load(
        assets / "model.ckpt", weights_only=True, mmap=True, map_location="cpu"
    )["state_dict"]
    encoder, projection = oracle.audio_encoder(package, state)
    encoder.cuda()
    projection.cuda()
    del state
    for directory in generated:
        source = json.loads((directory / "result.json").read_text())
        if source["status"] != "complete":
            raise ValueError("Incomplete generation")
        for row in source["rows"]:
            path = Path(row["wav"])
            raw = directory / (path.stem + "-raw.wav")
            if pilot.sha(path) != row["sha256"] or pilot.sha(raw) != row["raw_sha256"]:
                raise ValueError("Changed generated audio")
            wave, rate = sf.read(raw, dtype="float32")
            if rate != pilot.RATE or wave.shape != (pilot.LENGTH,):
                raise ValueError("Generated layout")
            times = source["events_seconds"]
            for j, start in enumerate(times):
                end = times[j + 1] if j + 1 < len(times) else len(wave) / rate
                event = wave[int(start * rate) : int(end * rate)]
                with torch.inference_mode():
                    embedding = (
                        projection(
                            encoder(
                                {"waveform": oracle.repeatpad(event)[None].cuda()},
                                device="cuda",
                            )["embedding"]
                        )
                        .cpu()
                        .numpy()
                    )
                values = {
                    "shape": oracle.spectral_shape(event[:9600])[None],
                    "embedding": embedding,
                }
                predicted = {
                    name: classify(
                        values[name.split("/")[0]], p, labels, name.split("/")[0]
                    )[0]
                    for name, (p, labels) in fitted.items()
                }
                result["generated"].append(
                    {
                        "wav": str(path),
                        "target": row["material"],
                        "kind": row["kind"],
                        "attack": j,
                        "predicted": predicted,
                    }
                )
    result["status"] = "complete_diagnostic_not_admission"
    pilot.save(output / "result.json", result)
    print(
        json.dumps(
            {
                name: value["held"]["recording_balanced_recall"]
                for name, value in result["methods"].items()
            }
        ),
        flush=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("data", "assets", "package", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--generated", type=Path, nargs="+", required=True)
    parser.add_argument("--reference-data", type=Path)
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        parser.error("Artifacts must remain external")
    run(
        args.data,
        args.generated,
        args.assets,
        args.package,
        args.output,
        args.reference_data,
    )
