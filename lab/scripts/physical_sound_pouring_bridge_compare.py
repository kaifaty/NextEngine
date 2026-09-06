"""Reference-free bridge comparison on two already-disclosed development objects.

Real audio enters metrics, never generation. Local research, no physical gate.
"""

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_pouring_bridge as b
import physical_sound_pouring_compare as c
import torch
from scipy.io import wavfile

p = c.p


def relative_power_profile(wave):
    """Shape with a RELATIVE floor; the legacy fixed dB floor depends on gain."""
    wave = np.asarray(wave, dtype=np.float64)
    if wave.shape != (p.SAMPLES,) or not np.isfinite(wave).all():
        raise ValueError("invalid profile waveform")
    power = np.mean(abs(p.transform(wave)[1:]) ** 2, axis=1)
    if power.max() <= 0:
        raise ValueError("zero-energy profile")
    db = 10 * np.log10(np.maximum(power / power.max(), 1e-10))
    return db - db.mean()


def checked_audio(row):
    path = Path(row["wav"])
    if path.stat().st_size > 1_000_000:
        raise ValueError("oversized comparison audio")
    if hashlib.sha256(path.read_bytes()).hexdigest() != row["sha256"]:
        raise ValueError("comparison audio hash mismatch")
    rate, pcm = wavfile.read(path)
    if (
        rate != p.RATE
        or pcm.dtype != np.int16
        or pcm.shape != (p.SAMPLES,)
        or abs(pcm.astype(np.int32)).max() > round(0.98 * 32767)
    ):
        raise ValueError("invalid published comparison PCM")
    return pcm.astype(np.float32) / 32768


def select_patches(rows):
    selected = []
    for key in p.HELDOUT:
        matches = [r for r in rows if r["container_id"] == key]
        if not matches or any(r["role"] != "unseen_container" for r in matches):
            raise ValueError("missing or relabelled development object")
        selected.append(matches[0])
    return [
        (r, phase, c.phase_crop(r, phase))
        for r in selected
        for phase in ("first", "middle")
    ]


def run(source, directory, previous_root, output):
    rows, provenance = p.load_source(source)
    patches = select_patches(rows)
    source_sha = hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
    bridge, meta = b.load_bridge(directory)
    setting_aware = meta.get("bridge_kind") == "setting-text"
    if setting_aware and any(r["setting"] not in b.SETTINGS for r, _, _ in patches):
        raise ValueError("unknown development setting")
    previous = json.loads((previous_root / "result.json").read_text())
    if (
        meta["source"]["source_sha256"] != source_sha
        or meta["source"]["train_ids"]
        != [r["item_id"] for r in rows if r["role"] == "train"]
        or previous["status"] != "complete"
        or previous["source_sha256"] != source_sha
    ):
        raise ValueError("model/source/previous split mismatch")
    prior = {}
    for row in previous["rows"]:
        key = (row["target_item_id"], row["target_phase"], row["seed"], row["kind"])
        if key in prior:
            raise ValueError("duplicate previous comparison key")
        checked_audio(row)
        prior[key] = row
    for r, phase, (_, controls, _, _) in patches:
        for seed in (314, 2718):
            matched = prior[r["item_id"], phase, seed, "matched"]
            if not np.array_equal(np.array(matched["controls"], np.float32), controls):
                raise ValueError("previous target controls differ")
            prior[r["item_id"], phase, seed, "base"]
    output = b.flow.c.v.phase.d.fresh_output(output)
    torch.set_num_threads(4)
    model, vae = b.train.tango.load_models(output)
    if b.digest(model) != meta["frozen_model_sha256_after"]:
        raise ValueError("frozen generator mismatch")
    report = {
        "status": "running",
        "scope": "first source-order recording per disclosed excluded object18/30; first/middle; two seeds, not independent test",
        "source_sha256": source_sha,
        "bridge_sha256": meta["bridge_sha256"],
        "previous_result_sha256": hashlib.sha256(
            (previous_root / "result.json").read_bytes()
        ).hexdigest(),
        "terms": provenance["terms"],
        "reference_audio_input_to_generator": False,
        "swap": "all11 controls from other object, same phase; not isolated material",
        "setting_policy": "target setting held fixed for matched/swapped/style-only"
        if setting_aware
        else None,
        "rows": [],
    }
    save = lambda: p.save_json(output / "result.json", report)
    save()
    generated, real_rows = {}, []
    try:
        for i, (r, phase, (_, controls, real, start)) in enumerate(patches):
            real_rows.append(
                {
                    **b.train.pilot.write_audio(output / f"{i}-real.wav", real),
                    "item_id": r["item_id"],
                    "container_id": r["container_id"],
                    "phase": phase,
                    "kind": "real",
                    "seed": None,
                }
            )
        for seed in (314, 2718):
            for i, (r, phase, (_, controls, _, start)) in enumerate(patches):
                variants = [("matched", controls, True)]
                if setting_aware:
                    variants += [
                        ("swapped", patches[(i + 2) % 4][2][1], True),
                        ("style-only", controls, False),
                    ]
                for kind, vector, physical in variants:
                    extra = (
                        {"setting": r["setting"], "physical": physical}
                        if setting_aware
                        else {}
                    )
                    row, _ = b.generate(
                        model,
                        vae,
                        output,
                        f"{i}-{kind}-{seed}",
                        controls=vector,
                        bridge=bridge,
                        seed=seed,
                        **extra,
                    )
                    generated[i, seed, kind] = {
                        **row,
                        "item_id": r["item_id"],
                        "container_id": r["container_id"],
                        "phase": phase,
                        "controls": vector.tolist(),
                        "start_sample": start,
                    }
                    report["generated_so_far"] = list(generated.values())
                    save()
        for i, (r, phase, (_, _, real, _)) in enumerate(patches):
            for seed in (314, 2718):
                pending = [
                    ("base", prior[r["item_id"], phase, seed, "base"]),
                    ("previous", prior[r["item_id"], phase, seed, "matched"]),
                    ("matched", generated[i, seed, "matched"]),
                    (
                        "swapped",
                        generated[i, seed, "swapped"]
                        if setting_aware
                        else generated[(i + 2) % 4, seed, "matched"],
                    ),
                ]
                if setting_aware:
                    pending.append(("style-only", generated[i, seed, "style-only"]))
                for kind, row in pending:
                    wave = checked_audio(row)
                    report["rows"].append(
                        {
                            **row,
                            "kind": kind,
                            "target_item_id": r["item_id"],
                            "target_container_id": r["container_id"],
                            "target_phase": phase,
                            **p.metrics(wave, real),
                            "relative_power_shape_rmse_db": float(
                                np.sqrt(
                                    np.mean(
                                        (
                                            relative_power_profile(wave)
                                            - relative_power_profile(real)
                                        )
                                        ** 2
                                    )
                                )
                            ),
                        }
                    )
        preview = []
        for i in (1, 3):
            preview.append({**real_rows[i], "seed": 2718})
            kinds = (
                ("previous", "matched", "swapped", "style-only")
                if setting_aware
                else ("previous", "matched", "swapped")
            )
            for kind in kinds:
                preview.append(
                    next(
                        r
                        for r in report["rows"]
                        if r["target_item_id"] == real_rows[i]["item_id"]
                        and r["target_phase"] == "middle"
                        and r["seed"] == 2718
                        and r["kind"] == kind
                    )
                )
        report["comparison"] = {
            **b.write_comparison(output, preview),
            "order": "glass18 thenPET30,middle,real/previous/new/swapped"
            + ("/style-only" if setting_aware else "")
            + ",seed2718,published gains",
        }
        report["real_rows"] = real_rows
        report.pop("generated_so_far")
        report["status"] = "complete"
        save()
        tagrows = list(generated.values())
        p.save_json(
            output / "tag-input.json",
            {
                "status": "complete",
                "seconds": b.SECONDS,
                "cases": [
                    {"id": Path(r["wav"]).stem, "diagnostic_id": "water-pour"}
                    for r in tagrows
                ],
                "rows": [{**r, "case": i} for i, r in enumerate(tagrows)],
                "controls": [],
            },
        )
        print({"primary_artifact": report["comparison"]["wav"]}, flush=True)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "model", "previous", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    run(args.source, args.model, args.previous, args.output)
