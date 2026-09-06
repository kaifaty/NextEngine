"""Recover missing physical port factors, preserving the prior family/meshes.

Fresh FEM coefficients are checked against prior frequencies/cross-residues.
No prior negative/passive check changes the TRAIN/DEV role or selects a body.
"""

from __future__ import annotations

import argparse
import json
import multiprocessing
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import numpy as np
import physical_sound_modal3d_pilot as pilot


def one(job):
    row, prior, output = job
    path = prior / row["file"]
    if pilot.integrity.sha256(path) != row["sha256"]:
        raise ValueError("prior teacher changed")
    with np.load(path, allow_pickle=False) as d:
        shape, contacts = d["shape"].copy(), d["contacts"].copy()
        modes = pilot.solve(
            shape, contacts, refinement=row["refinement"], contact_response=True
        )
        np.testing.assert_allclose(modes["omega"], d["omega"], rtol=1e-8)
        np.testing.assert_allclose(modes["gains"], d["gains"], rtol=1e-5, atol=1e-7)
    np.savez(output / row["file"], shape=shape, contacts=contacts, **modes)
    return {k: row[k] for k in ("role", "index", "shape", "file", "refinement")} | {
        "sha256": pilot.integrity.sha256(output / row["file"]),
        "prior_sha256": row["sha256"],
        "prior_local_stability": row["pair"]["locally_stable"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prior", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    data = json.loads((args.prior / "data.json").read_text())
    if len(data["rows"]) != 48:
        raise ValueError("full prior family required")
    args.output.mkdir(parents=True)
    rows = []
    with ProcessPoolExecutor(
        max_workers=3, mp_context=multiprocessing.get_context("spawn")
    ) as pool:
        for row in pool.map(one, [(r, args.prior, args.output) for r in data["rows"]]):
            rows.append(row)
            print("port teacher", row["role"], row["index"], flush=True)
    pilot.storage.save(
        args.output / "data.json",
        {
            "rows": rows,
            "prior_sha256": pilot.integrity.sha256(args.prior / "data.json"),
            "script_sha256": pilot.integrity.sha256(Path(__file__)),
            "solver_script_sha256": pilot.integrity.sha256(Path(pilot.__file__)),
            "scope": "same36TRAIN/12DEV and mesh3/4; new physical port factors, no label/role changes",
        },
    )
    pilot.storage.save(
        args.output / "train.json", {"rows": [r for r in rows if r["role"] == "train"]}
    )


if __name__ == "__main__":
    main()
