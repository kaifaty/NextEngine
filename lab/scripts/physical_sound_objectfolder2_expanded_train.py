"""One data-coverage discriminator: same compact student, 11 rather than 6 bodies.

Exactly five newly acquired TRAIN objects, no new/old DEV acoustic reads. Same
representation, optimizer, seed and 2000 updates; no architecture/loss/epoch sweep.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_compact as compact
import physical_sound_objectfolder2_expanded_eval as expanded
import physical_sound_objectfolder2_shared as shared
import torch

NEW_TRAIN = {47, 59, 72, 78, 91}
ALL_TRAIN = shared.TRAIN | NEW_TRAIN


def prepare(args):
    old = json.loads((args.original / "train.json").read_text())["rows"]
    new = json.loads((args.expanded / "data.json").read_text())["rows"]
    new = [r for r in new if r["role"] == "train" and r["status"] == "PREPARED"]
    if (
        len(old) != 6
        or {r["object_id"] for r in old} != shared.TRAIN
        or any(r["role"] != "train" for r in old)
    ):
        raise ValueError("original six TRAIN required")
    if len(new) != 5 or {r["object_id"] for r in new} != NEW_TRAIN:
        raise ValueError("exact five new TRAIN required")
    args.output.mkdir(parents=True)
    rows = []
    for source, entries, needs_pack in (
        (args.original, old, False),
        (args.expanded, new, True),
    ):
        for row in entries:
            data = expanded.read_npz(source / row["file"], row["sha256"])
            if needs_pack:
                parameters, bands = compact.pack(
                    data["frequency"], data["damping"], data["gains"]
                )
                data = {
                    k: data[k] for k in ("cloud", "features", "contacts")
                } | parameters
            else:
                bands = row["bands"]
            np.savez(args.output / row["file"], **data)
            rows.append(
                {
                    **row,
                    "sha256": teacher.sha(args.output / row["file"]),
                    "modes": len(data["frequency"]),
                    "bands": bands,
                    "input_source_sha256": row["sha256"],
                }
            )
            print("TRAIN packed", row["object_id"], len(data["frequency"]), flush=True)
    shared.write_json(
        args.output / "train.json",
        {
            "rows": rows,
            "scope": "same signed compact targets, original six plus five new TRAIN; DEV never packed here",
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("prepare", "fit"))
    for name in ("original", "expanded", "data"):
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    torch.set_num_threads(4)
    if args.stage == "prepare":
        prepare(args)
    else:
        shared.fit(args, train_ids=frozenset(ALL_TRAIN))
