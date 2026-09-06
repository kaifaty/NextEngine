"""Frozen image features plus learned acoustic residual; report-only impact pilot."""

import argparse
import collections
import json
import shutil
import tarfile
import time
from pathlib import Path

import numpy as np
import physical_sound_syncfusion_adapter as categorical
import physical_sound_syncfusion_pilot as pilot
import soundfile as sf
import torch
from PIL import Image, ImageOps
from torch.nn import functional as F

DINO = "facebook/dinov2-small"
REVISION = "ed25f3a31f01632728cabb09d1542f84ab7b0056"
DINO_SHA = "ae1e99fcefd534ed978cdeb8326f08030c96e28b7a81ffcbc98a857c84d14be1"


def frame_index(start, fps):
    if not np.isfinite(start) or start < 0.1 or fps != 15:
        raise ValueError("Expected eligible event and author15fps metadata")
    return int((start - 0.1) * fps) + 1


def image_inputs(processor, image, preprocessing="author-center-crop"):
    if preprocessing == "author-center-crop":
        return processor(images=image.convert("RGB"), return_tensors="pt")
    if preprocessing != "full-frame-letterbox-224":
        raise ValueError("Unknown visual preprocessing")
    # One causal counterfactual: preserve boundary interactions discarded by the
    # stock center crop. Not contact localization or calibrated physical shape.
    contained = ImageOps.contain(
        image.convert("RGB"), (224, 224), Image.Resampling.BICUBIC
    )
    fill = tuple(round(255 * v) for v in processor.image_mean)
    canvas = Image.new("RGB", (224, 224), fill)
    canvas.paste(
        contained, ((224 - contained.width) // 2, (224 - contained.height) // 2)
    )
    return processor(
        images=canvas, return_tensors="pt", do_resize=False, do_center_crop=False
    )


def prepare(
    data, archive_path, output, preprocessing="author-center-crop", backbone=None
):
    from huggingface_hub import hf_hub_download
    from transformers import AutoImageProcessor, Dinov2Model

    report = json.loads((data / "data.json").read_text())
    if (
        report["status"] != "complete"
        or pilot.sha(archive_path) != report["archive_sha256"]
    ):
        raise ValueError("Incomplete or changed author TRAIN source")
    output.mkdir(parents=True, exist_ok=False)
    for name in (
        "README.md",
        "config.json",
        "preprocessor_config.json",
        "model.safetensors",
    ):
        if backbone is None:
            hf_hub_download(DINO, name, revision=REVISION, local_dir=output / "dino")
        else:
            (output / "dino").mkdir(exist_ok=True)
            shutil.copyfile(backbone / name, output / "dino" / name)
    if pilot.sha(output / "dino/model.safetensors") != DINO_SHA:
        raise ValueError("Visual backbone identity mismatch")
    torch.set_num_threads(4)
    processor = AutoImageProcessor.from_pretrained(
        output / "dino", local_files_only=True
    )
    model = (
        Dinov2Model.from_pretrained(
            output / "dino", local_files_only=True, use_safetensors=True
        )
        .eval()
        .requires_grad_(False)
        .cuda()
    )
    rows, vectors = [], []
    receipt = {
        "status": "extracting",
        "data_sha256": pilot.sha(data / "data.json"),
        "dino_repo": DINO,
        "dino_revision": REVISION,
        "dino_sha256": DINO_SHA,
        "dino_license": "Apache-2.0",
        "preprocessing": preprocessing,
        "frame_rule": "one-based floor((onset-.1)*processed_fps)+1; metadata15fps; pre-impact nominal frame,not calibrated contact timestamp",
        "rows": rows,
    }
    pilot.save(output / "frames.json", receipt)
    with tarfile.open(archive_path, "r:") as archive:
        members = {m.name: m for m in archive if m.isfile()}
        for i, row in enumerate(report["rows"]):
            key = row["recording"].removesuffix(".times.csv")
            meta_member = members[key + ".metadata.json"]
            if meta_member.size > 10000:
                raise ValueError("Oversized frame metadata")
            metadata = json.loads(archive.extractfile(meta_member).read())
            fps = metadata["processed"]["video_frame_rate"]
            index = frame_index(row["start"], fps)
            name = key + f".frame_{index:06d}.jpg"
            member = members[name]
            if member.size > 1024**2:
                raise ValueError("Oversized frame")
            path = output / f"frame-{i:03d}.jpg"
            path.write_bytes(archive.extractfile(member).read())
            with Image.open(path) as image:
                if image.size != (320, 240):
                    raise ValueError("Unexpected processed image layout")
                inputs = image_inputs(processor, image, preprocessing)
            with torch.inference_mode():
                vector = F.normalize(
                    model(**inputs.to("cuda")).last_hidden_state[:, 0], dim=-1
                ).cpu()
            vectors.append(vector)
            rows.append(
                {
                    "index": i,
                    "recording": row["recording"],
                    "role": row["role"],
                    "material": row["material"],
                    "motion": row["motion"],
                    "frame": str(path),
                    "sha256": pilot.sha(path),
                    "member": name,
                    "fps": fps,
                    "nominal_time": (index - 1) / fps,
                    "event_time": row["start"],
                }
            )
            if i % 50 == 0:
                print("visual features", i + 1, "/", len(report["rows"]), flush=True)
        value = torch.cat(vectors)
        if value.shape != (len(rows), 384) or not torch.isfinite(value).all():
            raise ValueError("Invalid visual features")
        torch.save(value, output / "visual.pt")
        receipt.update(status="complete", visual_sha256=pilot.sha(output / "visual.pt"))
        pilot.save(output / "frames.json", receipt)


def mismatch_indices(rows):
    result = []
    for row in rows:
        candidates = [
            i
            for i, other in enumerate(rows)
            if other["recording"] != row["recording"]
            and (other["material"], other["motion"]) == (row["material"], row["motion"])
        ]
        if not candidates:
            raise ValueError("No same-descriptor different-recording frame control")
        result.append(candidates[0])
    return result


def ridge_residual(visual, baseline, targets, rows):
    train = torch.tensor([r["role"] == "train" for r in rows])
    training_keys = {r["recording"] for r in rows if r["role"] == "train"}
    if training_keys & {r["recording"] for r in rows if r["role"] != "train"}:
        raise ValueError("Recording leakage")
    counts = collections.Counter(
        (r["material"], r["motion"]) for r in rows if r["role"] == "train"
    )
    weights = torch.tensor(
        [
            1 / counts[(r["material"], r["motion"])]
            for r in rows
            if r["role"] == "train"
        ],
        dtype=torch.float64,
    )
    weights /= weights.sum()
    mean = (visual[train].double() * weights[:, None]).sum(0)
    x = visual[train].double() - mean
    y = (targets[train] - baseline[train]).double()
    # One fixed group-balanced ridge solve; no regularization/feature/capacity sweep.
    matrix = x.T @ (weights[:, None] * x) + 0.01 * torch.eye(
        x.shape[1], dtype=torch.float64
    )
    mapping = torch.linalg.solve(matrix, x.T @ (weights[:, None] * y))
    return mean.float(), mapping.float()


def fit(data, frames, base, output):
    output.mkdir(parents=True, exist_ok=False)
    original = json.loads((data / "data.json").read_text())
    image_report = json.loads((frames / "frames.json").read_text())
    base_report = json.loads((base / "fit.json").read_text())
    if (
        image_report["status"] != "complete"
        or pilot.sha(data / "data.json") != image_report["data_sha256"]
        or base_report["data_sha256"] != image_report["data_sha256"]
        or pilot.sha(frames / "visual.pt") != image_report["visual_sha256"]
        or pilot.sha(data / "embeddings.pt") != original["embeddings_sha256"]
        or pilot.sha(base / "adapter.pt") != base_report["adapter_sha256"]
    ):
        raise ValueError("Changed input identities")
    rows = original["rows"]
    visual = torch.load(frames / "visual.pt", weights_only=True)
    targets = torch.load(data / "embeddings.pt", weights_only=True)
    trained = torch.load(base / "adapter.pt", weights_only=True)
    adapter = categorical.Adapter().eval().requires_grad_(False)
    adapter.load_state_dict(trained["state_dict"], strict=True)
    with torch.inference_mode():
        baseline = adapter(
            torch.stack(
                [categorical.features(r["material"], r["motion"]) for r in rows]
            )
        )
    torch.set_num_threads(4)
    mean, mapping = ridge_residual(visual, baseline, targets, rows)
    wrong = mismatch_indices(rows)
    predictions = {
        "descriptor": baseline,
        "visual": F.normalize(baseline + (visual - mean) @ mapping, dim=-1),
        "wrong-frame": F.normalize(baseline + (visual[wrong] - mean) @ mapping, dim=-1),
    }
    torch.save(
        {
            "base_state": adapter.state_dict(),
            "visual_mean": mean,
            "visual_mapping": mapping,
        },
        output / "visual-adapter.pt",
    )
    validation = []
    for role in ("train", "recording-dev", "combination-dev"):
        for material, motion in (
            (m, s) for m in categorical.MATERIALS for s in categorical.MOTIONS
        ):
            indices = [
                i
                for i, r in enumerate(rows)
                if r["role"] == role
                and (r["material"], r["motion"]) == (material, motion)
            ]
            if indices:
                validation.append(
                    {
                        "role": role,
                        "material": material,
                        "motion": motion,
                        "n": len(indices),
                        **{
                            kind: float(
                                (1 - (pred[indices] * targets[indices]).sum(-1)).mean()
                            )
                            for kind, pred in predictions.items()
                        },
                    }
                )
    cases = []
    for material, motion in categorical.AUDITIONS:
        i = next(
            i
            for i, r in enumerate(rows)
            if r["role"] != "train"
            and (r["material"], r["motion"]) == (material, motion)
        )
        cases.append(
            {
                "material": material,
                "motion": motion,
                "row": i,
                "frame": image_report["rows"][i],
                "wrong_frame": image_report["rows"][wrong[i]],
            }
        )
    pilot.save(
        output / "fit.json",
        {
            "status": "complete",
            "weights_sha256": pilot.sha(output / "visual-adapter.pt"),
            "data_sha256": pilot.sha(data / "data.json"),
            "frames_sha256": pilot.sha(frames / "frames.json"),
            "base_sha256": base_report["adapter_sha256"],
            "checkpoint_sha256": base_report["checkpoint_sha256"],
            "dino_sha256": DINO_SHA,
            "preprocessing": image_report.get("preprocessing", "author-center-crop"),
            "ridge": 0.01,
            "new_parameters": mapping.numel(),
            "validation": validation,
            "cases": cases,
            "claim": "TRAIN-fitted visual residual diagnostic, not object/geometry/physical calibration",
        },
    )


def render(assets, frames, fitted, output):
    from transformers import AutoImageProcessor, Dinov2Model

    output.mkdir(parents=True, exist_ok=False)
    report = json.loads((fitted / "fit.json").read_text())
    if (
        report["status"] != "complete"
        or pilot.sha(fitted / "visual-adapter.pt") != report["weights_sha256"]
        or pilot.sha(assets / "model.ckpt") != report["checkpoint_sha256"]
        or pilot.sha(frames / "dino/model.safetensors") != report["dino_sha256"]
    ):
        raise ValueError("Changed inference weights")
    torch.set_num_threads(4)
    torch.manual_seed(42)
    trained = torch.load(fitted / "visual-adapter.pt", weights_only=True)
    adapter = categorical.Adapter().eval().requires_grad_(False)
    adapter.load_state_dict(trained["base_state"], strict=True)
    processor = AutoImageProcessor.from_pretrained(
        frames / "dino", local_files_only=True
    )
    image_model = (
        Dinov2Model.from_pretrained(
            frames / "dino", local_files_only=True, use_safetensors=True
        )
        .eval()
        .requires_grad_(False)
        .cuda()
    )
    conditions = []
    for case in report["cases"]:
        with torch.inference_mode():
            base = adapter(categorical.features(case["material"], case["motion"])[None])
        conditions.append((case, "descriptor", base[:, None]))
        for kind, reference in (
            ("visual", case["frame"]),
            ("wrong-frame", case["wrong_frame"]),
        ):
            path = Path(reference["frame"])
            if pilot.sha(path) != reference["sha256"]:
                raise ValueError("Changed input image")
            with Image.open(path) as image:
                inputs = image_inputs(
                    processor, image, report.get("preprocessing", "author-center-crop")
                )
            with torch.inference_mode():
                vector = F.normalize(
                    image_model(**inputs.to("cuda")).last_hidden_state[:, 0], dim=-1
                ).cpu()
                condition = F.normalize(
                    base
                    + (vector - trained["visual_mean"]) @ trained["visual_mapping"],
                    dim=-1,
                )
            conditions.append((case, kind, condition[:, None]))
    del image_model
    state = torch.load(
        assets / "model.ckpt", weights_only=True, mmap=True, map_location="cpu"
    )["state_dict"]
    model, encoder = pilot.build()
    model.load_state_dict(pilot.subset(state, "model."), strict=True)
    encoder.load_state_dict(pilot.subset(state, "onsets_encoder."), strict=True)
    del state
    model.cuda()
    encoder.cuda()
    result = {
        "status": "running",
        "reference_audio_input": False,
        "image_input": True,
        "preprocessing": report.get("preprocessing", "author-center-crop"),
        "audio_encoder_instantiated": False,
        "weights_sha256": report["weights_sha256"],
        "events_seconds": categorical.oracle.TIMES,
        "seed": 42,
        "steps": 150,
        "embedding_scale": 2,
        "playback_gain": 0.5,
        "rows": [],
    }
    pilot.save(output / "result.json", result)
    for case, kind, condition in conditions:
        started = time.monotonic()
        with torch.inference_mode():
            _, context = encoder(
                pilot.event_track(result["events_seconds"]).cuda(), with_info=True
            )
            noise = torch.randn(
                1,
                1,
                pilot.LENGTH,
                device="cuda",
                generator=torch.Generator(device="cuda").manual_seed(42),
            )
            wave = (
                model.sample(
                    x_noisy=noise,
                    num_steps=150,
                    channels=context["xs"][2:-1],
                    embedding=condition.cuda(),
                    embedding_scale=2,
                )[0, 0]
                .cpu()
                .numpy()
            )
        name = f"{case['material']}-{case['motion']}-{kind}"
        raw = output / f"{name}-raw.wav"
        sf.write(raw, wave, pilot.RATE, subtype="FLOAT")
        finite = bool(np.isfinite(wave).all())
        peak = float(np.abs(wave).max()) if finite else None
        row = {
            "id": name,
            "material": case["material"],
            "motion": case["motion"],
            "kind": kind,
            "evaluation_row": case["row"],
            "raw_sha256": pilot.sha(raw),
            "raw_peak": peak,
            "seconds": time.monotonic() - started,
        }
        if finite and peak * 0.5 < 0.98:
            path = output / f"{name}.wav"
            sf.write(path, wave * 0.5, pilot.RATE, subtype="PCM_16")
            row.update(
                status="published",
                wav=str(path),
                sha256=pilot.sha(path),
                timing=pilot.timing.match(
                    result["events_seconds"],
                    pilot.timing.onsets(wave * 0.5, pilot.RATE),
                ),
            )
        else:
            row.update(status="rejected", reason="finite/headroom")
        result["rows"].append(row)
        pilot.save(output / "result.json", result)
        print(json.dumps(row), flush=True)
    result.update(
        status="complete_with_rejections"
        if any(r["status"] == "rejected" for r in result["rows"])
        else "complete",
        cuda_peak_gib=torch.cuda.max_memory_allocated() / 2**30,
    )
    pilot.save(output / "result.json", result)


def assess(data, fitted, previous, output):
    if (output / "assessment.json").exists():
        raise ValueError("Assessment exists")
    result = json.loads((output / "result.json").read_text())
    fit_report = json.loads((fitted / "fit.json").read_text())
    if (
        not result["status"].startswith("complete")
        or pilot.sha(data / "data.json") != fit_report["data_sha256"]
        or result["weights_sha256"] != fit_report["weights_sha256"]
    ):
        raise ValueError("Changed or incomplete experiment")
    source_rows = json.loads((data / "data.json").read_text())["rows"]
    old = json.loads((previous / "result.json").read_text())
    report = {
        "scope": "three preselected held event/image cases; one reference event each; no object/material/quality admission",
        "rows": [],
        "comparisons": [],
    }
    for case in fit_report["cases"]:
        material, motion = case["material"], case["motion"]
        reference = source_rows[case["row"]]
        if (
            reference["role"] == "train"
            or pilot.sha(Path(reference["wav"])) != reference["sha256"]
        ):
            raise ValueError("Changed or training evaluation reference")
        wave, rate = sf.read(reference["wav"], dtype="float32")
        if rate != pilot.RATE or np.abs(wave).max() * 0.5 >= 0.98:
            raise ValueError("Reference rate/headroom")
        target = categorical.oracle.spectral_shape(wave[:9600])
        name = f"{material}-{motion}"
        reference_path = output / f"{name}-reference.wav"
        sf.write(reference_path, wave * 0.5, rate, subtype="PCM_16")
        waves, order = (
            [sf.read(reference_path)[0], np.zeros(rate // 2)],
            ["held-reference"],
        )
        for row in result["rows"]:
            if (row["material"], row["motion"]) != (material, motion):
                continue
            if row["status"] != "published":
                report["rows"].append({"id": row["id"], "status": "rejected"})
                continue
            path = Path(row["wav"])
            if pilot.sha(path) != row["sha256"]:
                raise ValueError("Changed generated PCM")
            wave, rate = sf.read(path, dtype="float32")
            if rate != pilot.RATE or wave.shape != (pilot.LENGTH,):
                raise ValueError("Generated layout")
            features = np.stack(
                [
                    categorical.oracle.spectral_shape(
                        wave[int(t * rate) : int(t * rate) + 9600]
                    )
                    for t in result["events_seconds"]
                ]
            )
            measured = {
                "id": row["id"],
                "material": material,
                "motion": motion,
                "kind": row["kind"],
                "status": "measured",
                "shape_distance_db": float(np.abs(features - target).mean()),
                "per_generated_attack_db": np.abs(features - target).mean(-1).tolist(),
                "timing": row["timing"],
                "reference_index": case["row"],
            }
            if row["kind"] == "descriptor":
                control = next(
                    r
                    for r in old["rows"]
                    if (r["material"], r["motion"], r["kind"])
                    == (material, motion, "adapter")
                )
                measured["previous_descriptor_pcm_exact"] = (
                    row["sha256"] == control["sha256"]
                )
            report["rows"].append(measured)
            waves.extend([wave, np.zeros(rate // 2)])
            order.append(row["kind"])
        comparison = output / f"{name}-comparison.wav"
        sf.write(comparison, np.concatenate(waves), pilot.RATE, subtype="PCM_16")
        report["comparisons"].append(
            {
                "id": name,
                "wav": str(comparison),
                "sha256": pilot.sha(comparison),
                "order": order,
                "correct_frame": case["frame"],
                "wrong_frame": case["wrong_frame"],
            }
        )
    pilot.save(output / "assessment.json", report)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "mode", choices=("prepare", "prepare-full-frame", "fit", "render", "assess")
    )
    for name in ("data", "archive", "frames", "base", "assets", "fitted", "previous"):
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--backbone", type=Path)
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        parser.error("Artifacts must remain external")
    required = {
        "prepare": (args.data, args.archive),
        "prepare-full-frame": (args.data, args.archive),
        "fit": (args.data, args.frames, args.base),
        "render": (args.assets, args.frames, args.fitted),
        "assess": (args.data, args.fitted, args.previous),
    }[args.mode]
    if any(p is None for p in required):
        parser.error("Missing mode-specific input")
    if args.mode == "prepare-full-frame":
        prepare(
            *required,
            args.output,
            preprocessing="full-frame-letterbox-224",
            backbone=args.backbone,
        )
    else:
        {"prepare": prepare, "fit": fit, "render": render, "assess": assess}[args.mode](
            *required, args.output
        )
