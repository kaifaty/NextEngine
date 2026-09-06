"""Fixed shared-student comparison, not a realism validator or tuning loop.

Primary: all three open DEV bodies, four fixed contacts each, versus same-material
nearest geometry and its inverse-size time-scaled response. No audio chooses a
neighbor. Report TRAIN reconstruction separately. Never normalize a candidate
independently of its reference, or treat synthetic vibration as real pressure.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_shared as shared
from physical_sound_sonicgauss_waveform_fit import audio_metrics
from scipy.io import wavfile
from scipy.spatial.distance import cdist


def geometry_distance(a, b, size_std):
    distances = cdist(a["cloud"], b["cloud"])
    chamfer = (distances.min(0).mean() + distances.min(1).mean()) / 2
    size = np.linalg.norm((a["features"][:3] - b["features"][:3]) / size_std)
    return float(chamfer + size)


def scaled_response(source, contact, ratio):
    """Explicit baseline heuristic, not a calibrated physical scaling law.

    Size scaling can cross Nyquist: omit and COUNT those modes only in this
    retuned baseline. Full teacher packing and the shared model stay unchanged.
    """
    if not np.isfinite(ratio) or ratio <= 0:
        raise ValueError("positive finite size ratio required")
    frequency = source["frequency"] * ratio
    keep = (frequency > 0) & (frequency < teacher.RATE / 2)
    return teacher.waveform(
        source["gains"][contact][:, keep],
        frequency[keep],
        source["damping"][keep] * ratio,
        [1, 1, 1],
    ), int((~keep).sum())


def assess(args):
    manifest = json.loads((args.data / "data.json").read_text())
    rendering = json.loads((args.generated / "render.json").read_text())
    if (
        len(manifest["rows"]) != 9
        or {r["object_id"] for r in manifest["rows"]} != shared.TRAIN | shared.DEV
    ):
        raise ValueError("exact nine-object data required")
    data, records, predictions = {}, {}, {}
    for row in manifest["rows"]:
        identity = row["object_id"]
        path = args.data / row["file"]
        if teacher.sha(path) != row["sha256"]:
            raise ValueError("teacher data changed")
        with np.load(path, allow_pickle=False) as saved:
            data[identity] = dict(saved)
        with np.load(
            args.generated / f"object-{identity}.npz", allow_pickle=False
        ) as saved:
            predictions[identity] = dict(saved)
        records[identity] = row
    size_std = np.stack([data[i]["features"][:3] for i in sorted(shared.TRAIN)]).std(
        0, ddof=1
    )
    size_std = np.maximum(size_std, 1e-6)
    if (
        len(rendering["rows"]) != 36
        or len({(r["object_id"], r["contact"]) for r in rendering["rows"]}) != 36
    ):
        raise ValueError("all 36 standalone responses required")
    args.output.mkdir(parents=True)
    rows, galleries, replay_count = [], {}, 0
    for row in rendering["rows"]:
        identity, contact = row["object_id"], row["contact"]
        source, predicted = data[identity], predictions[identity]
        reference = teacher.waveform(
            source["gains"][contact], source["frequency"], source["damping"], [1, 1, 1]
        )
        neural = teacher.waveform(
            predicted["gains"][contact],
            predicted["frequency"],
            predicted["damping"],
            [1, 1, 1],
        )
        path = args.generated / row["file"]
        rate, pcm = wavfile.read(path)
        if (
            teacher.sha(path) != row["sha256"]
            or rate != teacher.RATE
            or not np.array_equal(
                pcm, (neural * rendering["common_gain"]).astype(np.float32)
            )
        ):
            raise ValueError("standalone waveform replay mismatch")
        replay_count += 1
        result = {
            "object_id": identity,
            "contact": contact,
            "role": "train" if identity in shared.TRAIN else "development",
            "source_modes": len(source["frequency"]),
            "predicted_modes": len(predicted["frequency"]),
            "neural": audio_metrics(reference, neural),
        }
        waves = {"reference": reference, "neural": neural}
        if identity in shared.DEV:
            choices = sorted(
                i
                for i in shared.TRAIN
                if records[i]["material"] == records[identity]["material"]
            )
            neighbor = min(
                choices, key=lambda i: geometry_distance(source, data[i], size_std)
            )
            nearest = data[neighbor]
            nearest_contact = int(
                np.linalg.norm(
                    nearest["contacts"] - source["contacts"][contact], axis=1
                ).argmin()
            )
            ratio = records[neighbor]["length"] / records[identity]["length"]
            waves["nearest"], omitted = scaled_response(nearest, nearest_contact, 1.0)
            if omitted:
                raise ValueError("unscaled source unexpectedly aliases")
            waves["size_scaled"], omitted = scaled_response(
                nearest, nearest_contact, ratio
            )
            result.update(
                neighbor=neighbor,
                neighbor_contact=nearest_contact,
                size_ratio=ratio,
                scaled_modes_omitted=omitted,
            )
            for kind in ("nearest", "size_scaled"):
                result[kind] = audio_metrics(reference, waves[kind])
            # One gain for the complete quadruple, no target-matching candidate gain.
            gain = min(1.0, 0.5 / max(float(abs(w).max()) for w in waves.values()))
            gallery = np.concatenate(
                [
                    waves[k] * gain
                    for k in ("reference", "neural", "nearest", "size_scaled")
                ]
            ).astype(np.float32)
            filename = f"object-{identity}-contact-{contact}-comparison.wav"
            wavfile.write(args.output / filename, teacher.RATE, gallery)
            result.update(
                comparison=filename,
                comparison_gain=gain,
                comparison_sha256=teacher.sha(args.output / filename),
            )
            if contact == 0:
                galleries[identity] = np.concatenate([reference, neural])
        rows.append(result)
        print("assessed", identity, contact, result["neural"], flush=True)
    summary = {}
    for role in ("train", "development"):
        selected = [r for r in rows if r["role"] == role]
        kinds = ("neural",) if role == "train" else ("neural", "nearest", "size_scaled")
        summary[role] = {
            kind: {
                metric: float(np.mean([r[kind][metric] for r in selected]))
                for metric in ("spectrum", "envelope", "level")
            }
            for kind in kinds
        }
    dev = summary["development"]
    advantage = all(
        dev["neural"][metric] < dev[control][metric]
        for metric in ("spectrum", "envelope", "level")
        for control in ("nearest", "size_scaled")
    )
    # Fixed first contact of every DEV object. Pair gains differ BETWEEN objects,
    # but are identical within each reference/neural pair; no loudness comparison.
    snippets = []
    for identity in sorted(galleries):
        pair = galleries[identity]
        gain = min(1.0, 0.5 / float(abs(pair).max()))
        name = f"object-{identity}-reference-neural.wav"
        pcm = (pair * gain).astype(np.float32)
        wavfile.write(args.output / name, teacher.RATE, pcm)
        snippets.append(pcm)
    wavfile.write(
        args.output / "three-untrained-objects-reference-neural.wav",
        teacher.RATE,
        np.concatenate(snippets),
    )
    shared.write_json(
        args.output / "assessment.json",
        {
            "rows": rows,
            "summary": summary,
            "quality_advantage": advantage,
            "decision": "DESCRIPTIVE_ADVANTAGE_ONLY"
            if advantage
            else "REJECT_QUALITY_ADVANTAGE",
            "independent_development_objects": 3,
            "replayed_standalone_wavs_exact": replay_count,
            "weights_sha256": rendering["fit_sha256"],
            "scope": "synthetic vibration reconstruction; not perceptual realism, pressure, striker material or unseen-material proof; open DEV",
            "gallery_order": "11 reference/neural, 54 reference/neural, 88 reference/neural; full comparisons reference/neural/nearest/size-scaled",
        },
    )
    print(json.dumps(summary, indent=2), "advantage", advantage, flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--generated", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    assess(parser.parse_args())
