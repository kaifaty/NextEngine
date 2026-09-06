"""Disclosed new-surface transfer from category and measured friction coefficients.

Fixed urethane rubber probe. Coefficients measured at10mm/min, not the audio
scan speeds. No geometry, arbitrary contact pair, quality or runtime admission.
Powered by Stability AI; TangoFlux/Hung et al.; Eguchi et al. CC BY4 source.
"""

from __future__ import annotations

import argparse
import json
import shutil
import time
from pathlib import Path

import numpy as np
import physical_sound_texture_event as event
import torch
from safetensors.torch import load_file, save_file

flow = event.flow
source = flow.spectrum.source
TRAIN = (0, 2, 65, 67, 74, 77)
HELD = (4, 66, 76)
SURFACES = tuple(sorted(TRAIN + HELD))
CATEGORIES = ("Wood", "Metals", "Glass")
TABLE_SHA = "6cd7dfc06e61852d49a2256d7fe8d44d222f3619f48fe9a4031f677d1b0eb4ff"
FORMAT = "texture-surface-flow-v1"


def features(category, static, dynamic, speed, force):
    if category not in CATEGORIES or not np.isfinite([static, dynamic]).all():
        raise ValueError("published category and finite coefficients required")
    if not 0 <= dynamic <= static <= 2:
        raise ValueError("invalid coefficients")
    base = event.trace_features(
        flow.spectrum.TEXTURES[CATEGORIES.index(category)], speed, force
    )
    # Fixed dimensionless scaling; not fitted on development statistics.
    return np.concatenate(
        (
            base,
            np.tile([(static - 0.5) / 0.25, (dynamic - 0.5) / 0.25], (len(speed), 1)).T,
        )
    ).astype(np.float32)


def validate_features(physical):
    if physical.ndim != 2 or physical.shape[0] != 7:
        raise ValueError("seven physical features required")
    category = CATEGORIES[int(np.argmax(physical[:3, 0]))]
    rebuilt = features(
        category,
        physical[5, 0] * 0.25 + 0.5,
        physical[6, 0] * 0.25 + 0.5,
        physical[3] * 20 + 40,
        physical[4] * 0.25 + 0.75,
    )
    if not np.allclose(physical, rebuilt, rtol=0, atol=1e-6):
        raise ValueError("changing surface or invalid feature encoding")


def role(row):
    if row["texture_id"] not in SURFACES:
        raise ValueError("outside declared surface split")
    return "unseen_surface" if row["texture_id"] in HELD else flow.spectrum.role(row)


def metadata(path):
    if path.stat().st_size > 2_000_000:
        raise ValueError("oversized surface manifest")
    report = json.loads(path.read_text())
    table_path = flow.spectrum.checked(
        str(path.parent / "texture_list.xlsx"), report, path.parent
    )
    if flow.codec.sha(table_path) != TABLE_SHA:
        raise ValueError("published descriptor table changed")
    table = source.texture_metadata(table_path)
    if (
        not report.get("surface_grid")
        or report.get("selected_texture_ids") != list(SURFACES)
        or report.get("surface_metadata") != [table[t] for t in SURFACES]
    ):
        raise ValueError("declared surface grid mismatch")
    if any(table[t]["category"] not in CATEGORIES for t in SURFACES):
        raise ValueError("unknown category")
    return table


def builder(table):
    def build(texture, speed, force):
        item = table[texture]
        return features(
            item["category"],
            item["static_friction"],
            item["dynamic_friction"],
            speed,
            force,
        )

    return build


def load_data(path):
    table = metadata(path)
    build = builder(table)
    rows, waves, noises, controls, report = flow.spectrum.load_data(
        path,
        textures={t: table[t]["name"] for t in SURFACES},
        feature_builder=lambda t, v, f: build(t, np.full(32, v), np.full(32, f))[:, 0],
    )
    for row in rows:
        row["role"] = role(row)
    return rows, waves, noises, controls, report


def expand(parent):
    model = flow.TextureFlow(7).to(next(parent.parameters()).device)
    state = {k: v.detach().clone() for k, v in parent.state_dict().items()}
    weight = state["context.0.weight"]
    state["context.0.weight"] = torch.cat(
        (weight[:, :5], torch.zeros_like(weight[:, :2]), weight[:, 5:]), 1
    )
    model.load_state_dict(state, strict=True)
    return model


def fit(model, records, descriptors):
    training = [r for r in records if r["row"]["role"] == "train"]
    if len(training) != 48 or {r["row"]["texture_id"] for r in training} != set(TRAIN):
        raise ValueError("exact48-TRAIN split required")
    device = next(model.parameters()).device
    torch.manual_seed(23)
    model.train()
    optimizer = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=1e-4)
    losses = []
    for step in range(2000):
        targets, physical = [], []
        for record in training:
            last = record["valid_frames"] - flow.FRAMES
            first = (
                0
                if step % 3 == 0
                else last
                if step % 3 == 2
                else int(torch.randint(last + 1, (1,)))
            )
            mean = record["mean"][:, first : first + flow.FRAMES]
            std = record["std"][:, first : first + flow.FRAMES]
            targets.append(mean + std * torch.randn_like(std))
            physical.append(record["physical"][:, first : first + flow.FRAMES])
        target = (torch.stack(targets).to(device) - model.center) / model.scale
        controls = torch.tensor(np.stack(physical), device=device)
        if not descriptors:
            controls[:, 5:] = 0
        noise = torch.randn_like(target)
        t = torch.rand(len(target), device=device)
        mixed = noise * (1 - t[:, None, None]) + target * t[:, None, None]
        loss = (model(mixed, t, controls) - (target - noise)).square().mean()
        optimizer.zero_grad()
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1, error_if_nonfinite=True)
        optimizer.step()
        losses.append(float(loss.detach()))
        if (step + 1) % 200 == 0:
            print(
                json.dumps(
                    {
                        "descriptors": descriptors,
                        "step": step + 1,
                        "loss200": float(np.mean(losses[-200:])),
                    }
                ),
                flush=True,
            )
    model.eval()
    return [float(np.mean(losses[i : i + 200])) for i in range(0, 2000, 200)]


@torch.inference_mode()
def generate(model, vae, physical, seed, *, category_only=False):
    validate_features(physical)
    controls = physical.copy()
    if category_only:
        controls[5:] = 0
    latent = flow.sample_features(model, controls[None], seed, controls.shape[-1])
    return vae.decode(latent).sample[0].T.cpu().numpy()


def metrics(wave, real, record):
    row = record["row"]
    result = event.event_metrics(
        wave, real, record["physical"], row["commanded_speed_mm_s"]
    )
    start = round(row["crop_start_seconds"] * event.RATE)
    stop = start + round(0.75 * event.RATE)
    # Explicit common22.05kHz bandwidth and central moving region, not apparatus quiet.
    a, b = [event.resample_poly(w[start:stop].mean(1), 1, 2) for w in (wave, real)]
    delta = flow.spectrum.spectrum(a)[1:] - flow.spectrum.spectrum(b)[1:]
    result.update(
        moving_spectrum_rmse_db=float(np.sqrt(np.mean(delta**2))),
        moving_shape_rmse_db=float(np.std(delta)),
        moving_level_absolute_error_db=float(
            abs(flow.codec.level(a) - flow.codec.level(b))
        ),
    )
    return result


def load_models(directory, device):
    path = directory / "model.json"
    if path.stat().st_size > 100000:
        raise ValueError("oversized model metadata")
    meta = json.loads(path.read_text())
    expected_ids = {
        r["id"]
        for r in source.conditions(True, {t: str(t) for t in SURFACES})
        if role(r) == "train"
    }
    if (
        meta["format"] != FORMAT
        or meta["codec_sha256"] != flow.CODEC_SHA
        or meta["input_gain"] != flow.GAIN
        or meta["table_sha256"] != TABLE_SHA
        or meta["training_surfaces"] != list(TRAIN)
        or meta["held_surfaces"] != list(HELD)
        or len(meta["training_ids"]) != 48
        or set(meta["training_ids"]) != expected_ids
    ):
        raise ValueError("incompatible surface model")
    models = {}
    for arm in ("descriptor", "category_only"):
        weights = directory / f"{arm}.safetensors"
        if (
            weights.stat().st_size > 4_000_000
            or flow.codec.sha(weights) != meta[arm]["sha256"]
        ):
            raise ValueError("model hash/size mismatch")
        state = load_file(weights)
        if (
            not all(torch.isfinite(v).all() for v in state.values())
            or (state["scale"] <= 0).any()
        ):
            raise ValueError("invalid model weights")
        models[arm] = flow.TextureFlow(7).to(device).eval()
        models[arm].load_state_dict(state, strict=True)
    return models, meta


@torch.inference_mode()
def evaluate(models, parent, vae, records, table, output, report):
    preview = []
    for record in records:
        row = record["row"]
        # All60 new-surface cases plus36 original-surface development anchors.
        if row["role"] != "unseen_surface" and not (
            row["texture_id"] in source.TEXTURES and row["role"] != "train"
        ):
            continue
        physical = record["physical"]
        real_entry, real = flow.codec.publish(
            output / f"{row['id']}-real.wav", record["wave"] * event.PLAYBACK
        )
        report["references"].append({"id": row["id"], **real_entry})
        item = table[row["texture_id"]]
        peers = [table[t] for t in TRAIN if table[t]["category"] == item["category"]]
        # Farthest TRAIN coefficients, same category; chosen without listening/scoring.
        wrong = max(
            peers,
            key=lambda p: (
                (p["static_friction"] - item["static_friction"]) ** 2
                + (p["dynamic_friction"] - item["dynamic_friction"]) ** 2
            ),
        )
        wrong_features = features(
            item["category"],
            wrong["static_friction"],
            wrong["dynamic_friction"],
            physical[3] * 20 + 40,
            physical[4] * 0.25 + 0.75,
        )
        for seed in (314, 2718):
            native = {
                "descriptor": generate(models["descriptor"], vae, physical, seed),
                "category_only": generate(
                    models["category_only"], vae, physical, seed, category_only=True
                ),
                "wrong_descriptor": generate(
                    models["descriptor"], vae, wrong_features, seed
                ),
                "parent_category": event.generate(parent, vae, physical[:5], seed),
            }
            pcms = {}
            for arm, wave in native.items():
                entry, pcm = event.publish(
                    output, f"{row['id']}-{arm}-{seed}", wave, len(real)
                )
                pcms[arm] = pcm
                report["rows"].append(
                    {
                        "id": row["id"],
                        "texture_id": row["texture_id"],
                        "role": row["role"],
                        "seed": seed,
                        "variant": arm,
                        "wrong_texture_id": wrong["texture_id"],
                        **entry,
                        **metrics(pcm, real, record),
                    }
                )
            if (
                row["role"] == "unseen_surface"
                and row["commanded_speed_mm_s"] == 40
                and row["commanded_normal_force_N"] == 0.5
                and row["repeat"] == 0
                and seed == 314
            ):
                group = []
                for pcm in (
                    real,
                    pcms["descriptor"],
                    pcms["category_only"],
                    pcms["parent_category"],
                ):
                    group.extend((pcm, np.zeros((round(0.25 * event.RATE), 2))))
                preview.extend(group)
                report[f"surface_{row['texture_id']}_comparison"], _ = (
                    flow.codec.publish(
                        output / f"surface-{row['texture_id']}-comparison.wav",
                        np.concatenate(group),
                    )
                )
        print(json.dumps({"evaluated": row["id"]}), flush=True)
        flow.save(output / "result.json", report)
    report["comparison"], _ = flow.codec.publish(
        output / "comparison.wav", np.concatenate(preview)
    )
    report["comparison_order"] = (
        "Oak/Steel/Frosted glass,40mm/s,.5N,repeat0,seed314: real/descriptor/category-only/parent-category"
    )
    report["summary"] = []
    for texture in HELD + (None,):
        for arm in (
            "descriptor",
            "category_only",
            "wrong_descriptor",
            "parent_category",
        ):
            rr = [
                r
                for r in report["rows"]
                if r["variant"] == arm
                and (
                    r["texture_id"] == texture
                    if texture is not None
                    else r["role"] != "unseen_surface"
                )
            ]
            report["summary"].append(
                {
                    "texture_id": texture,
                    "variant": arm,
                    "count": len(rr),
                    **{
                        key: float(np.mean([r[key] for r in rr]))
                        for key in (
                            "envelope_db_mae",
                            "moving_spectrum_rmse_db",
                            "moving_shape_rmse_db",
                            "moving_level_absolute_error_db",
                        )
                    },
                }
            )


def run(source_path, parent_path, output, existing=None, device="cuda"):
    torch.set_num_threads(4)
    source_path = source_path.resolve()
    output = flow.fresh(output)
    started = time.monotonic()
    report = {
        "status": "running",
        "source_sha256": flow.codec.sha(source_path),
        "rows": [],
        "references": [],
    }
    flow.save(output / "result.json", report)
    try:
        table = metadata(source_path)
        parent, parent_meta = flow.load_model(parent_path, device, event.FORMAT)
        expected_parent = {
            r["id"] for r in source.conditions(True) if flow.spectrum.role(r) == "train"
        }
        if set(parent_meta["training_ids"]) != expected_parent:
            raise ValueError("original24-case timed parent required")
        vae, root = flow.load_codec(device)
        records, corpus, _ = event.prepare(
            source_path, vae, device, loader=load_data, feature_builder=builder(table)
        )
        if existing is None:
            models = {}
            meta = {
                "format": FORMAT,
                "codec_sha256": flow.CODEC_SHA,
                "input_gain": flow.GAIN,
                "table_sha256": TABLE_SHA,
                "source_sha256": report["source_sha256"],
                "parent_checkpoint_sha256": parent_meta["checkpoint_sha256"],
                "training_surfaces": list(TRAIN),
                "held_surfaces": list(HELD),
                "training_ids": [
                    r["row"]["id"] for r in records if r["row"]["role"] == "train"
                ],
                "steps_per_arm": 2000,
                "seed": 23,
                "source_license": corpus["license"],
                "scope": "disclosed development, fixed rubber; no geometry/pair/quality admission",
            }
            for arm in ("descriptor", "category_only"):
                model = expand(parent)
                losses = fit(model, records, arm == "descriptor")
                weights = output / f"{arm}.safetensors"
                save_file(model.state_dict(), weights)
                meta[arm] = {"sha256": flow.codec.sha(weights), "loss200": losses}
                models[arm] = model
                flow.save(output / "model.json", meta)
            for module in (
                __file__,
                event.__file__,
                flow.__file__,
                flow.spectrum.__file__,
                source.__file__,
            ):
                shutil.copyfile(module, output / Path(module).name)
            for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
                shutil.copyfile(root / name, output / name)
        else:
            models, meta = load_models(existing, device)
            if (
                meta["source_sha256"] != report["source_sha256"]
                or meta["parent_checkpoint_sha256"] != parent_meta["checkpoint_sha256"]
            ):
                raise ValueError("evaluation lineage mismatch")
        report.update(
            model=meta,
            new_training_steps=0 if existing else 4000,
            prepared_and_trained_seconds=time.monotonic() - started,
        )
        # Audible primary artifact before the larger evaluation; no source trace/audio.
        physical, request = event.profile()
        item = table[76]
        physical = features(
            item["category"],
            item["static_friction"],
            item["dynamic_friction"],
            physical[3] * 20 + 40,
            physical[4] * 0.25 + 0.75,
        )
        native = generate(models["descriptor"], vae, physical, 314)
        report["standalone"], _ = event.publish(
            output,
            "requested-frosted-glass",
            native,
            round(request["duration_seconds"] * event.RATE),
        )
        report["standalone"].update(
            request={**request, "texture": 76, **item},
            seed=314,
            reference_audio_input=False,
            sensor_input=False,
        )
        flow.save(output / "result.json", report)
        evaluate(models, parent, vae, records, table, output, report)
        report.update(status="complete", elapsed_seconds=time.monotonic() - started)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        flow.save(output / "result.json", report)
        raise
    flow.save(output / "result.json", report)


def render(
    model_path,
    output,
    category,
    static,
    dynamic,
    speed=40.0,
    force=0.5,
    seed=314,
    device="cuda",
):
    torch.set_num_threads(4)
    models, meta = load_models(model_path, device)
    physical, request = event.profile(
        flow.spectrum.TEXTURES[CATEGORIES.index(category)], speed, force
    )
    physical = features(
        category, static, dynamic, physical[3] * 20 + 40, physical[4] * 0.25 + 0.75
    )
    vae, _ = flow.load_codec(device)
    wave = generate(models["descriptor"], vae, physical, seed)
    output = flow.fresh(output)
    entry, _ = event.publish(
        output, "generated", wave, round(request["duration_seconds"] * event.RATE)
    )
    flow.save(
        output / "result.json",
        {
            "status": "complete",
            "model": meta,
            "request": {k: v for k, v in request.items() if k != "texture"},
            "category": category,
            "static_friction": static,
            "dynamic_friction": dynamic,
            "seed": seed,
            "reference_audio_input": False,
            "sensor_input": False,
            **entry,
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path)
    parser.add_argument("--parent", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--evaluate-model", type=Path)
    parser.add_argument("--model", type=Path)
    parser.add_argument("--category", choices=CATEGORIES, default="Glass")
    parser.add_argument("--static", type=float)
    parser.add_argument("--dynamic", type=float)
    parser.add_argument("--speed", type=float, default=40)
    parser.add_argument("--force", type=float, default=0.5)
    parser.add_argument("--seed", type=int, default=314)
    args = parser.parse_args()
    if args.model:
        if args.static is None or args.dynamic is None:
            parser.error("--static and --dynamic required for source-free rendering")
        render(
            args.model,
            args.output,
            args.category,
            args.static,
            args.dynamic,
            args.speed,
            args.force,
            args.seed,
        )
    else:
        if not args.source or not args.parent:
            parser.error("--source and --parent required for fitting/evaluation")
        run(args.source, args.parent, args.output, args.evaluate_model)
