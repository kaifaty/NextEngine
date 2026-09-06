"""Common-noise object/contact separation with recorded-audio positive control.

No fit, seed search, per-object correction or physical acceptance. Material,
geometry, appearance and valid contact all covary between object conditions.
Object signatures are diagnostics, not a perceptual or physical validator.
"""

from __future__ import annotations

import argparse
import itertools
import json
from pathlib import Path

import numpy as np
import physical_sound_sonicgauss_codec_probe as codec
import physical_sound_sonicgauss_cohort as cohort
import physical_sound_sonicgauss_energy_probe as energy
import physical_sound_sonicgauss_objective_probe as objective
import physical_sound_sonicgauss_pilot as pilot
import physical_sound_sonicgauss_shared_fit as shared
import torch
from scipy.io import wavfile


def signature(wave):
    """Independent IIR + time-averaged, unit-L1 magnitude per resolution."""
    if wave.ndim != 2 or wave.shape[0] != 2 or not np.isfinite(wave).all():
        raise ValueError("finite stereo required")
    mono = codec.audible_component(wave).mean(0)
    if np.sqrt(np.mean(mono.astype(float) ** 2)) < 1e-8:
        raise ValueError("silent signal has no identifiable spectral signature")
    spectra = [m.mean(1) for m in objective.magnitudes(mono)]
    return np.concatenate([s / s.sum() / 3 for s in spectra])


def shape_distance(a, b):
    return float(np.abs(a - b).sum())


def retrieval(queries, candidates, ids):
    if set(queries) != set(ids) or set(candidates) != set(ids):
        raise ValueError("retrieval identities differ")
    rows = []
    for oid in ids:
        distances = {
            other: shape_distance(queries[oid], candidates[other]) for other in ids
        }
        correct = distances[oid]
        # Ties never become automatic identity wins.
        rank = 1 + sum(v < correct for v in distances.values())
        unique_best = all(correct < v for other, v in distances.items() if other != oid)
        rows.append(
            {
                "object_id": oid,
                "rank": rank,
                "unique_top1": unique_best,
                "distances": distances,
            }
        )
    return {
        "top1": sum(r["unique_top1"] for r in rows),
        "count": len(ids),
        "rows": rows,
    }


def assess(args):
    corpus = json.loads((args.data / "corpus.json").read_text())
    shared.validate_inputs(corpus)
    receipt = json.loads((args.generated / "result.json").read_text())
    if not receipt.get("common_noise_across_conditions") or receipt[
        "input_sha256"
    ] != pilot.sha256(args.data / "inputs.json"):
        raise ValueError("common-noise generation binding required")
    ids = list(cohort.OBJECTS)
    expected = {(o, c, s) for o in ids for c in range(6) for s in range(2)}
    records = {
        (r["object_id"], r["contact_index"], r["sample"]): r for r in receipt["rows"]
    }
    if set(records) != expected or len(receipt["rows"]) != len(expected):
        raise ValueError("incomplete common-noise cohort")
    waves, signatures, spectra, noises = {}, {}, {}, {}
    with torch.no_grad():
        for key, record in records.items():
            path = args.generated / (record["name"] + ".npz")
            if pilot.sha256(path) != record["npz_sha256"]:
                raise ValueError("generation hash mismatch")
            with np.load(path, allow_pickle=False) as saved:
                wave, noise = saved["wave"].copy(), saved["initial_noise"].copy()
            if wave.shape != (2, 131072) or not np.isfinite(wave).all():
                raise ValueError("invalid full waveform")
            if key[2] in noises and not np.array_equal(noise, noises[key[2]]):
                raise ValueError("condition intervention changed noise")
            noises[key[2]] = noise
            waves[key] = wave
            signatures[key] = signature(wave)
            spectra[key] = energy.features(torch.from_numpy(wave)[None])
    if np.array_equal(noises[0], noises[1]):
        raise ValueError("sampling control has identical noise")

    def distances(a, b):
        return {
            "spectral_energy_distance": energy.distance(spectra[a], spectra[b]).item(),
            "signature_distance": shape_distance(signatures[a], signatures[b]),
        }

    contacts, sampling, objects = [], [], []
    for oid in ids:
        for c in range(6):
            sampling.append(
                {"object_id": oid, "contact": c, **distances((oid, c, 0), (oid, c, 1))}
            )
        for a, b in itertools.combinations(range(6), 2):
            for s in range(2):
                contacts.append(
                    {
                        "object_id": oid,
                        "contact_a": a,
                        "contact_b": b,
                        "sample": s,
                        **distances((oid, a, s), (oid, b, s)),
                    }
                )
    for a, b in itertools.combinations(ids, 2):
        for c in range(6):
            for s in range(2):
                objects.append(
                    {
                        "object_a": a,
                        "object_b": b,
                        "author_row_index": c,
                        "sample": s,
                        **distances((a, c, s), (b, c, s)),
                    }
                )
    summaries = {}
    for metric in ("spectral_energy_distance", "signature_distance"):
        summaries[metric] = {
            name: float(np.mean([r[metric] for r in rows]))
            for name, rows in (
                ("noise", sampling),
                ("contact", contacts),
                ("object_configuration", objects),
            )
        }
        summaries[metric].update(
            {
                name + "_to_noise_ratio": summaries[metric][name]
                / summaries[metric]["noise"]
                for name in ("contact", "object_configuration")
            }
        )
    per_object = []
    for oid in ids:
        item = {"object_id": oid}
        for metric in summaries:
            noise = np.mean([r[metric] for r in sampling if r["object_id"] == oid])
            contact = np.mean([r[metric] for r in contacts if r["object_id"] == oid])
            item[metric] = {
                "noise": float(noise),
                "contact": float(contact),
                "contact_to_noise_ratio": float(contact / noise),
            }
        per_object.append(item)

    real, recorded = {}, {}
    for obj in corpus["objects"]:
        oid = obj["object_id"]
        for contact in obj["contacts"]:
            ref = contact["reference"]
            wave = shared.teacher_audio(Path(ref["path"]), ref["sha256"])
            real[oid, contact["index"]] = signature(wave)
            recorded[oid, contact["index"]] = wave
    queries = {o: np.mean([real[o, c] for c in (0, 2, 4)], axis=0) for o in ids}
    controls = {o: np.mean([real[o, c] for c in (1, 3, 5)], axis=0) for o in ids}
    predictions = {
        o: np.mean([signatures[o, c, s] for c in range(6) for s in range(2)], axis=0)
        for o in ids
    }
    identity = {
        "recorded_odd_contact_control": retrieval(queries, controls, ids),
        "generated_all_contacts": retrieval(queries, predictions, ids),
    }
    # Full samples in every gallery: no per-object normalization/crop.
    auditions = [("objects-same-noise.wav", [waves[o, 0, 0].mean(0) for o in ids])]
    for oid in ids:
        auditions.append(
            (
                f"object-{oid:02d}-contact-noise-comparison.wav",
                [waves[oid, c, 0].mean(0) for c in range(6)]
                + [waves[oid, 0, 1].mean(0)],
            )
        )
        auditions.append(
            (
                f"object-{oid:02d}-recorded-generated.wav",
                [recorded[oid, 0].mean(0), waves[oid, 0, 0].mean(0)],
            )
        )
    gain = min(
        1.0, 0.98 / max(float(abs(w).max()) for _, parts in auditions for w in parts)
    )
    args.output.mkdir(parents=True)
    for name, parts in auditions:
        wavfile.write(
            args.output / name,
            44100,
            np.concatenate(
                [v for w in parts for v in (w * gain, np.zeros(22050))]
            ).astype(np.float32),
        )
    result = {
        "summary": summaries,
        "per_object": per_object,
        "identity": identity,
        "sampling_rows": sampling,
        "contact_rows": contacts,
        "object_rows": objects,
        "common_noise_verified_all_conditions": True,
        "comparison_gain": gain,
        "generated_receipt_sha256": pilot.sha256(args.generated / "result.json"),
        "script_sha256": pilot.sha256(Path(__file__)),
        "scope": "two-noise report-only intervention; object configuration jointly changes appearance/geometry/contact; contact indices across objects are not physical correspondences; neither ratio nor retrieval proves realism or missing physical variables",
    }
    cohort.save(args.output / "result.json", result)
    print(
        json.dumps(
            {
                "summary": summaries,
                "identity_top1": {k: v["top1"] for k, v in identity.items()},
            },
            indent=2,
        ),
        flush=True,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("data", "generated", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    torch.set_num_threads(4)
    assess(args)


if __name__ == "__main__":
    main()
