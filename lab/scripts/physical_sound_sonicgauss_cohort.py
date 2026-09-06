"""Shared pretrained 3D audio baseline on ten disclosed TRAIN objects.

Source recording access is confined to acquisition/assessment. Render consumes a
separate geometry/contact manifest, never reference audio. A prospective local
fine-tuning split is not an unseen-pretraining-object test. No per-object fitting.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import zipfile
import zlib
from pathlib import Path

import numpy as np
import physical_sound_sonicgauss_condition_probe as condition
import physical_sound_sonicgauss_pilot as pilot
import torch
from scipy.io import wavfile
from scipy.signal import resample_poly, stft

OBJECTS = {
    2: "Ceramic",
    6: "Glass",
    12: "Wood",
    14: "Wood",
    24: "Plastic",
    66: "Iron",
    75: "Ceramic",
    94: "Glass",
    95: "Glass",
    97: "Plastic",
}
FIT_TRAIN = {2, 6, 12, 24, 66, 95}
COUNTS = {2: 28, 6: 39, 12: 30, 14: 26, 24: 33, 66: 26, 75: 46, 94: 40, 95: 42, 97: 36}
RANGE_HASHES = {
    "probe.py": "7aa499faee844832b4bd42a2d4a15cc64e230fe41a6dc634c64a288f2ff6294d",
    "directory.json": "b97a7a410418eb561c8d42aa23becce16e12b7fa6176e00f6e8bd296e716775c",
    "objectfolder_real_train.json": "555e811d432ff77d071f04d09c1e82a189e53092efe81e408d8526a3aec9a6c1",
}
PROJECTION_SHA = "93d397c18c3c8df49f7f687caf79d8e346b7691c0ab1726dd3488bcd8f2bc00e"


def save(path, data):
    path.write_text(json.dumps(data, indent=2) + "\n")


def select_records(records, projection):
    if projection["role"] != "generator_train":
        raise ValueError("corrected TRAIN projection required")
    selected = []
    for object_id, material in OBJECTS.items():
        parents = {
            x["physical_parent_id"]
            for x in projection["rows"]
            if x["family_component_id"]
            == "source-component.realimpact-objectfolder-av-msf.v1"
            and x["material_label"] == material
            and (
                x["physical_parent_id"]
                == "realimpact-6-bowl--objectfolder-real-object-6"
                if object_id == 6
                else re.search(rf"object-{object_id}$", x["physical_parent_id"])
            )
        }
        if len(parents) != 1:
            raise ValueError(f"ambiguous/missing disclosed identity: {object_id}")
        member = f"OF_Real/ObjectFolderResults/{object_id}/model.ply"
        rows = [x for x in records if x["ply_file"] == "datas/" + member]
        if len(rows) != COUNTS[object_id]:
            raise ValueError(f"unexpected author TRAIN membership: {object_id}")
        contacts = []
        for index, row in enumerate(rows[:6]):
            p = np.asarray(row["contact"], dtype=np.float64)
            if p.shape != (3,) or not np.isfinite(p).all() or abs(p).max() > 10000:
                raise ValueError("invalid contact")
            audio = row["location"].removeprefix("datas/")
            if not re.fullmatch(
                rf"OF_Real/ObjectFolderResults/{object_id}/audio/{object_id}_\d+_3sec_48000hz\.wav",
                audio,
            ):
                raise ValueError("unexpected source member")
            contacts.append(
                {"index": index, "contact": p.tolist(), "reference_member": audio}
            )
        selected.append(
            {
                "object_id": object_id,
                "material_label": material,
                "physical_parent_id": parents.pop(),
                "source_role": "disclosed_generator_train",
                "pretrained_split": "author_train",
                "local_fit_role": "train" if object_id in FIT_TRAIN else "development",
                "ply_member": member,
                "author_train_count": len(rows),
                "contacts": contacts,
            }
        )
    return selected


def acquire(range_root, projection_path, output):
    pilot.verify(range_root, RANGE_HASHES)
    if pilot.sha256(projection_path) != PROJECTION_SHA:
        raise ValueError("corrected projection hash mismatch")
    selected = select_records(
        json.loads((range_root / "objectfolder_real_train.json").read_text()),
        json.loads(projection_path.read_text()),
    )
    spec = importlib.util.spec_from_file_location(
        "sonic_pinned_range", range_root / "probe.py"
    )
    reader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reader)
    directory = {
        x["name"]: x for x in json.loads((range_root / "directory.json").read_text())
    }
    if (output / "inputs.json").exists() or (output / "corpus.json").exists():
        raise ValueError("completed cohort already exists")
    output.mkdir(parents=True, exist_ok=True)
    receipts = []

    def extract(member, path):
        record = directory[member]
        if not 0 < record["size"] < 128 * 1024 * 1024:
            raise ValueError("oversized entry")
        if path.exists():
            data = path.read_bytes()
            if len(data) != record["size"] or zlib.crc32(data) != record["crc"]:
                raise ValueError(f"invalid existing partial acquisition: {path}")
        else:
            info = zipfile.ZipInfo(member)
            for key, value in {
                "volume": record["disk"],
                "header_offset": record["offset"],
                "file_size": record["size"],
                "compress_size": record["compressed"],
                "CRC": record["crc"],
                "compress_type": 8,
                "flag_bits": 0,
            }.items():
                setattr(info, key, value)
            data = reader.entry(info)
            path.parent.mkdir(parents=True, exist_ok=True)
            with path.open("xb") as stream:
                stream.write(data)
        result = {
            "path": str(path),
            "bytes": len(data),
            "sha256": pilot.sha256(path),
            "member": member,
            "zip_crc32": record["crc"],
        }
        receipts.append(result)
        save(
            output / "acquisition-progress.json",
            {"files": receipts, "ranges_this_process": reader.RECEIPTS},
        )
        return result

    for row in selected:
        object_id = row["object_id"]
        row["ply"] = extract(
            row["ply_member"], output / f"objects/{object_id}/model.ply"
        )
        for contact in row["contacts"]:
            contact["reference"] = extract(
                contact["reference_member"],
                output / f"references/{object_id}/{contact['index']}.wav",
            )
        print(
            "acquired",
            object_id,
            row["material_label"],
            len(row["contacts"]),
            "contacts",
            flush=True,
        )
    corpus = {
        "dataset_revision": reader.REV,
        "range_source_hashes": RANGE_HASHES,
        "projection_sha256": PROJECTION_SHA,
        "objects": selected,
        "full_archive_sha_verified": False,
        "scope": "report-only; no protected roles; no product admission",
    }
    save(output / "corpus.json", corpus)
    # Intentionally no reference waveform path/bytes/hash in the render input.
    inputs = {
        **corpus,
        "objects": [
            {k: v for k, v in row.items() if k != "contacts"}
            | {
                "contacts": [
                    {k: c[k] for k in ["index", "contact"]} for c in row["contacts"]
                ]
            }
            for row in selected
        ],
    }
    save(output / "inputs.json", inputs)
    print(
        "complete acquisition",
        len(receipts),
        "files",
        sum(x["bytes"] for x in receipts),
        "bytes",
        flush=True,
    )


def render(source, assets, data, output):
    if output.exists():
        raise ValueError("new render output required")
    manifest = json.loads((data / "inputs.json").read_text())
    if [x["object_id"] for x in manifest["objects"]] != list(OBJECTS):
        raise ValueError("cohort membership mismatch")
    for row in manifest["objects"]:
        if pilot.sha256(Path(row["ply"]["path"])) != row["ply"]["sha256"]:
            raise ValueError("geometry hash mismatch")
        if (
            row["source_role"] != "disclosed_generator_train"
            or len(row["contacts"]) != 6
        ):
            raise ValueError("unexpected object role/contacts")
    torch.set_num_threads(4)
    torch.manual_seed(0)
    ns = pilot.load_definitions(source)
    models = pilot.build_models(source, assets, ns)
    _, encoder, _, fusion, vae, _ = models
    output.mkdir(parents=True)
    records = []
    waves = []
    for row in manifest["objects"]:
        object_id = row["object_id"]
        gs = ns["load_ply"](row["ply"]["path"])
        if not all(torch.isfinite(x).all() for x in gs.values()):
            raise ValueError("nonfinite geometry")
        normalized, scaler = ns["preprocess_gaussian"](
            gs, "cuda", scaler_class=ns["MinMaxScaler"], return_scaler=True
        )
        contacts = [
            ns["normalize_position"](x["contact"], scaler, "cuda")
            for x in row["contacts"]
        ]
        capture = condition.CaptureGaussian(encoder)
        for index in [0, 1]:
            ge = (
                capture
                if index == 0
                else condition.CachedGaussian(capture.features, capture.rng)
            )
            result = condition.render(
                ns,
                models,
                ge,
                condition.FusionProbe(fusion),
                normalized,
                contacts[index],
            )
            name = f"object-{object_id:02d}-contact-{index}"
            np.savez(output / f"{name}.npz", **result)
            wave = result["wave"]
            if not all(np.isfinite(x).all() for x in result.values()):
                raise ValueError("nonfinite result; raw retained")
            record = {
                "name": name,
                "object_id": object_id,
                "material_label": row["material_label"],
                "contact_index": index,
                "normalized_contact": contacts[index].tolist(),
                "raw_peak": float(np.abs(wave).max()),
                "raw_rms": float(np.sqrt(np.mean(wave.astype(float) ** 2))),
                "source_free": True,
                "local_fit_role": row["local_fit_role"],
            }
            if record["raw_rms"] < 1e-7:
                raise ValueError("silent output; raw retained")
            records.append(record)
            waves.append(wave)
            print(
                name,
                row["material_label"],
                record["raw_peak"],
                record["raw_rms"],
                flush=True,
            )
        np.savez(
            output / f"object-{object_id:02d}-geometry.npz",
            features=capture.features.cpu().numpy(),
            contacts=torch.stack(contacts).cpu().numpy(),
            cpu_rng=capture.rng[0].numpy(),
            cuda_rng=capture.rng[1].numpy(),
        )
    gain = min(1.0, 0.98 / max(row["raw_peak"] for row in records))
    rate = vae.config.sampling_rate
    for row, wave in zip(records, waves, strict=True):
        path = output / f"{row['name']}.wav"
        wavfile.write(path, rate, (wave.T * gain).astype(np.float32))
        row.update(wav=str(path), wav_sha256=pilot.sha256(path))
    for i, row in enumerate(manifest["objects"]):
        pair = np.concatenate(
            [waves[i * 2].T * gain, np.zeros((rate // 2, 2)), waves[i * 2 + 1].T * gain]
        ).astype(np.float32)
        wavfile.write(output / f"object-{row['object_id']:02d}-pair.wav", rate, pair)
    gallery = np.concatenate(
        [
            part
            for wave in waves[::2]
            for part in [wave.T * gain, np.zeros((rate // 2, 2))]
        ]
    ).astype(np.float32)
    wavfile.write(output / "gallery.wav", rate, gallery)
    save(
        output / "result.json",
        {
            "status": "REPORT_ONLY_SHARED_PRETRAINED_COHORT",
            "rows": records,
            "shared_gain": gain,
            "rate": rate,
            "input_manifest_sha256": pilot.sha256(data / "inputs.json"),
            "script_sha256": pilot.sha256(Path(__file__)),
            "weight_hashes": pilot.WEIGHTS,
            "source_hashes": pilot.SOURCES,
            "steps": 50,
            "seed": 0,
            "guidance": -1,
            "full_decode_retained": True,
            "target_audio_read": False,
            "limits": "All objects in published generator TRAIN; no unseen-object or calibrated force/striker/size claim.",
        },
    )


def audio(path, expected_hash):
    if pilot.sha256(path) != expected_hash:
        raise ValueError("waveform hash mismatch")
    rate, value = wavfile.read(path)
    if value.dtype == np.int16:
        value = value.astype(np.float32) / 32768
    elif value.dtype != np.float32:
        raise ValueError("expected PCM16 or float32")
    if value.ndim == 2:
        if value.shape[1] != 2:
            raise ValueError("expected mono/stereo")
        value = value.mean(axis=1)
    if (
        value.ndim != 1
        or not np.isfinite(value).all()
        or not 8000 <= rate <= 96000
        or not 0 < len(value) <= 4 * rate
    ):
        raise ValueError("invalid bounded audio")
    divisor = np.gcd(rate, 44100)
    return resample_poly(value, 44100 // divisor, rate // divisor).astype(np.float32)


def magnitude_distance(reference, generated):
    """Descriptive multi-resolution STFT L1; not perceptual acceptance."""
    if any(
        x.ndim != 1 or not len(x) or not np.isfinite(x).all()
        for x in (reference, generated)
    ):
        raise ValueError("expected nonempty finite mono signals")
    length = max(len(reference), len(generated))
    signals = [np.pad(x, (0, length - len(x))) for x in (reference, generated)]
    errors = []
    for size in (512, 1024, 2048):
        mags = [
            np.abs(stft(x, 44100, nperseg=size, noverlap=size * 3 // 4)[2])
            for x in signals
        ]
        denominator = float(mags[0].sum())
        if denominator <= 0:
            raise ValueError("silent reference")
        errors.append(float(np.abs(mags[0] - mags[1]).sum() / denominator))
    return float(np.mean(errors))


def assess(data, generated, output):
    import physical_sound_text_tags as tags

    if output.exists():
        raise ValueError("new assessment output required")
    corpus = json.loads((data / "corpus.json").read_text())
    result = json.loads((generated / "result.json").read_text())
    if result["input_manifest_sha256"] != pilot.sha256(data / "inputs.json"):
        raise ValueError("render input binding changed")
    output.mkdir(parents=True)
    originals = {}
    candidates = {}
    for row in corpus["objects"]:
        for contact in row["contacts"][:2]:
            ref = contact["reference"]
            originals[(row["object_id"], contact["index"])] = audio(
                Path(ref["path"]), ref["sha256"]
            )
    for row in result["rows"]:
        candidates[(row["object_id"], row["contact_index"])] = audio(
            Path(row["wav"]), row["wav_sha256"]
        )
    if originals.keys() != candidates.keys() or len(candidates) != 20:
        raise ValueError("incomplete baseline cohort")
    # One gain for every reference and generated audition in this assessment.
    gain = min(
        1.0,
        0.98
        / max(
            float(abs(x).max())
            for x in list(originals.values()) + list(candidates.values())
        ),
    )
    manifest = {
        "status": "complete",
        "seconds": 3.0,
        "controls": [],
        "cases": [],
        "rows": [],
    }
    rows = []
    for key, reference in originals.items():
        object_id, index = key
        candidate = candidates[key]
        name = f"object-{object_id:02d}-contact-{index}"
        material = OBJECTS[object_id]
        record = {
            "object_id": object_id,
            "contact_index": index,
            "material_label": material,
            "matched_mrstft_l1": magnitude_distance(reference, candidate),
            "swapped_contact_mrstft_l1": magnitude_distance(
                originals[(object_id, 1 - index)], candidate
            ),
            "reference_rms": float(np.sqrt(np.mean(reference.astype(float) ** 2))),
            "generated_rms": float(np.sqrt(np.mean(candidate.astype(float) ** 2))),
        }
        rows.append(record)
        wavfile.write(
            output / f"{name}-comparison.wav",
            44100,
            np.concatenate(
                [reference * gain, np.zeros(22050), candidate * gain, np.zeros(22050)]
            ).astype(np.float32),
        )
        for kind, value in [("reference", reference), ("generated", candidate)]:
            path = output / f"{name}-{kind}.wav"
            wavfile.write(path, 44100, np.round(value * gain * 32767).astype(np.int16))
            diagnostic = (
                "real-glass"
                if material == "Glass"
                else "wood-material"
                if material == "Wood"
                else "unmapped-material"
            )
            manifest["cases"].append(
                {"id": name + "-" + kind, "diagnostic_id": diagnostic}
            )
            manifest["rows"].append(
                {
                    "case": len(manifest["cases"]) - 1,
                    "wav": str(path),
                    "sha256": pilot.sha256(path),
                    "seed": 0,
                }
            )
    save(
        output / "signal.json",
        {
            "rows": rows,
            "shared_gain": gain,
            "reference_after_generation_only": True,
            "mean_mrstft_l1": float(np.mean([x["matched_mrstft_l1"] for x in rows])),
            "matched_beats_swapped": sum(
                x["matched_mrstft_l1"] < x["swapped_contact_mrstft_l1"] for x in rows
            ),
            "scope": "Known pretraining TRAIN; descriptive error, not calibrated contact/material/force acceptance.",
        },
    )
    save(output / "tag-inputs.json", manifest)
    tags.EXPECTED["wood-material"] = ("Wood", "Knock", "Wood block")
    for name, rms in [("raw", None), ("rms", 0.005)]:
        tags.run(
            output / "tag-inputs.json",
            output / f"tags-{name}.json",
            [],
            ast_rms=rms,
            device="cuda",
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=["acquire", "render", "assess"])
    for name in [
        "range-root",
        "projection",
        "source",
        "assets",
        "data",
        "generated",
        "output",
    ]:
        parser.add_argument("--" + name, type=Path, required=name == "output")
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("output must stay outside repository")
    if args.stage == "acquire":
        if args.range_root is None or args.projection is None:
            parser.error("acquire needs --range-root --projection")
        acquire(args.range_root, args.projection, args.output)
    elif args.stage == "render":
        if any(x is None for x in [args.source, args.assets, args.data]):
            parser.error("render needs --source --assets --data")
        render(args.source, args.assets, args.data, args.output)
    else:
        if args.data is None or args.generated is None:
            parser.error("assess needs --data --generated")
        assess(args.data, args.generated, args.output)


if __name__ == "__main__":
    main()
