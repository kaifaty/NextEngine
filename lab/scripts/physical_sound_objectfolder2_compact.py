"""Shared signed-band targets with poles common to every contact of an object.

One representation change after the TRAIN-only band discriminator. Six TRAIN
and three open DEV roles remain unchanged; input projection is copied verbatim.
No per-contact pole fitting, no optimization, no teacher/target at generation.
"""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_band_probe as probe
import physical_sound_objectfolder2_shared as shared


def pack(frequency, damping, gains):
    f, d, g = (
        np.asarray(frequency, float),
        np.asarray(damping, float),
        np.asarray(gains, float),
    )
    if (
        f.ndim != 1
        or d.shape != f.shape
        or g.shape != (32, 3, len(f))
        or not len(f)
        or not np.isfinite([f, d]).all()
        or not np.isfinite(g).all()
        or np.any((f < 1) | (f > 22049))
        or np.any(d <= 0)
    ):
        raise ValueError("full finite 32-contact source required")
    edges = (
        np.expm1(np.linspace(np.log1p(1 / 700), np.log1p(22049 / 700), probe.BANDS + 1))
        * 700
    )
    band = np.minimum(np.searchsorted(edges[1:], f, side="right"), probe.BANDS - 1)
    frequencies, dampings, fields, rows = [], [], [], []
    for b in sorted(set(band.tolist())):
        ix = np.flatnonzero(band == b)
        if len(ix) <= 3:
            frequencies.extend(f[ix])
            dampings.extend(d[ix])
            fields.append(g[:, :, ix])
        else:
            # All contacts set ONE representative pole; none is chosen by sound.
            energy = (g[:, :, ix] ** 2).sum((0, 1)) * probe.basis_gram(
                f[ix], d[ix]
            ).diagonal()
            weights = (
                energy / energy.sum()
                if energy.sum() > 0
                else np.full(len(ix), 1 / len(ix))
            )
            frequencies.append(weights @ f[ix])
            dampings.append(weights @ d[ix])
            fields.append(g[:, :, ix].sum(-1, keepdims=True))
        rows.append(
            {
                "band": b,
                "source_modes": len(ix),
                "packed_modes": len(ix) if len(ix) <= 3 else 1,
            }
        )
    return {
        "frequency": np.array(frequencies),
        "damping": np.array(dampings),
        "gains": np.concatenate(fields, axis=-1).astype(np.float32),
    }, rows


def prepare(args):
    rows = json.loads((args.data / "data.json").read_text())["rows"]
    if (
        len(rows) != 9
        or {r["object_id"] for r in rows} != shared.TRAIN | shared.DEV
        or any(
            r["role"] != ("train" if r["object_id"] in shared.TRAIN else "development")
            for r in rows
        )
    ):
        raise ValueError("unchanged nine-object role projection required")
    args.output.mkdir(parents=True)
    records = []
    for row in rows:
        path = args.data / row["file"]
        if teacher.sha(path) != row["sha256"]:
            raise ValueError("full teacher changed")
        with np.load(path, allow_pickle=False) as saved:
            original = dict(saved)
        packed, bands = pack(
            original["frequency"], original["damping"], original["gains"]
        )
        values = {k: original[k] for k in ("cloud", "features", "contacts")}
        np.savez(args.output / row["file"], **values, **packed)
        records.append(
            {
                **row,
                "sha256": teacher.sha(args.output / row["file"]),
                "modes": len(packed["frequency"]),
                "full_teacher_file": str(path),
                "full_teacher_sha256": row["sha256"],
                "full_modes": row["modes"],
                "bands": bands,
            }
        )
        print(
            row["object_id"], row["modes"], "to", len(packed["frequency"]), flush=True
        )
    shared.write_json(
        args.output / "data.json",
        {
            "rows": records,
            "all_modes_preserved": False,
            "all_source_modes_accounted_for": True,
            "scope": "signed 128-band approximation; <=3 modes retained, >3 summed; common poles across32contacts",
        },
    )
    shared.write_json(
        args.output / "train.json",
        {"rows": [r for r in records if r["role"] == "train"]},
    )
    for name in ("inputs.json", "inputs.npz"):
        shutil.copyfile(args.data / name, args.output / name)
        if teacher.sha(args.data / name) != teacher.sha(args.output / name):
            raise ValueError("geometry-only inputs changed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    prepare(parser.parse_args())
