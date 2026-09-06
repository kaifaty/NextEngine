"""TRAIN-only frozen hidden-feature subspace diagnostic with an untrained control.

No neural fit. Larger numerical Ritz spaces are not equal-cost replacements for
the32-column head. Exact P1 reference modes are solved only during assessment.
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as source
import physical_sound_objectfolder2_physical_shared as shared
import physical_sound_objectfolder2_ritz_probe as probe
import torch
from safetensors.torch import load_file, save_file
from scipy.io import wavfile
from scipy.linalg import eigh
from scipy.sparse import load_npz
from scipy.sparse.linalg import eigsh


def extract(model, geometry):
    captured = []
    hook = model.field[-1].register_forward_pre_hook(
        lambda _, args: captured.append(args[0].detach())
    )
    try:
        with torch.no_grad():
            _, actual = model(
                *[
                    torch.tensor(geometry[k][None])
                    for k in ("cloud", "features", "points")
                ]
            )
    finally:
        hook.remove()
    h = np.c_[captured[0][0].numpy().astype(float), np.ones(len(geometry["points"]))]
    weight = np.c_[
        model.field[-1].weight.detach().numpy(), model.field[-1].bias.detach().numpy()
    ]
    v = (h @ weight.T).reshape(len(h), 3, 32) * model.field_scale.numpy()
    relative = float(np.linalg.norm(v - actual[0].numpy()) / np.linalg.norm(v))
    if relative > 1e-6:
        raise ValueError("hidden linear-head reconstruction failed")
    return h, v.reshape(-1, 32), relative


def vector_dictionary(hidden):
    result = np.zeros((len(hidden) * 3, hidden.shape[1] * 3))
    for axis in range(3):
        result[axis::3, axis * hidden.shape[1] : (axis + 1) * hidden.shape[1]] = hidden
    return result


def expanded_basis(dictionary, head, m, rigid):
    """Preserve the head exactly; add only numerically resolved extra directions."""
    q = probe.mass_basis(probe.remove_rigid(head, rigid, m), m)
    before = np.sqrt(np.einsum("ij,ij->j", dictionary, m @ dictionary))
    extra = probe.remove_rigid(dictionary, rigid, m)
    extra -= q @ (q.T @ (m @ extra))
    norms = np.sqrt(np.maximum(0, np.einsum("ij,ij->j", extra, m @ extra)))
    keep = norms > before * 1e-10
    extra = extra[:, keep] / norms[keep]
    gram = extra.T @ (m @ extra)
    values, axes = eigh((gram + gram.T) / 2)
    keep = values > values[-1] * 1e-10
    extra = extra @ (axes[:, keep] / np.sqrt(values[keep]))
    # Reorthogonalize once after rank-revealing extraction; no diagonal loading.
    combined = probe.mass_basis(np.c_[q, extra], m)
    error = float(abs(combined.T @ (m @ combined) - np.eye(combined.shape[1])).max())
    if error > 1e-7:
        raise ValueError("expanded mass orthogonality failed")
    return combined, {
        "dimension": combined.shape[1],
        "mass_error": error,
        "extra_relative_rank_cutoff": 1e-10,
        "smallest_retained_relative_eigenvalue": float(values[keep][0] / values[-1]),
    }


def lowest(v, k, m, rigid, count=32):
    values, u, residual = probe.ritz(v, k, m, rigid)
    return values[:count], u[:, :count], residual[:count]


def load_inputs(root, identity):
    data = root / "objectfolder2-operator-data-fixed-2026-09-06"
    rows = json.loads((data / "operators.json").read_text())["rows"]
    if identity not in shared.TRAIN:
        raise ValueError("TRAIN-only feature diagnostic")
    row = next(r for r in rows if r["object_id"] == identity)
    if row["role"] != "train":
        raise ValueError("TRAIN role required")
    paths = {}
    for name, record in row["files"].items():
        path = data / record["file"]
        if source.sha(path) != record["sha256"]:
            raise ValueError("changed TRAIN input")
        paths[name] = path
    with np.load(paths["geometry"], allow_pickle=False) as d:
        g = dict(d)
    query_data = root / "objectfolder2-physical-shared-data-2026-09-06"
    query_row = next(
        r
        for r in json.loads((query_data / "train.json").read_text())["rows"]
        if r["object_id"] == identity
    )
    path = query_data / query_row["geometry_file"]
    if source.sha(path) != query_row["geometry_sha256"]:
        raise ValueError("changed audition geometry")
    with np.load(path, allow_pickle=False) as d:
        audition = dict(d)
    ids = np.array(query_row["point_node_ids"])
    np.testing.assert_array_equal(g["points"][ids], audition["points"])
    return load_npz(paths["k"]), load_npz(paths["m"]), g, ids, audition


def generate(args):
    fit = args.root / "objectfolder2-operator-fit-2026-09-06"
    receipt = json.loads((fit / "fit.json").read_text())
    if source.sha(fit / "model.safetensors") != receipt["weights_sha256"]:
        raise ValueError("changed frozen weights")
    trained = shared.PhysicalStudent().eval().requires_grad_(False)
    trained.load_state_dict(load_file(fit / "model.safetensors"))
    torch.manual_seed(42)
    random = shared.PhysicalStudent().eval().requires_grad_(False)
    args.output.mkdir(parents=True)
    save_file(random.state_dict(), args.output / "untrained.safetensors")
    results = []
    for identity in sorted(shared.TRAIN):
        k, m, g, ids, _ = load_inputs(args.root, identity)
        for name, model in (("trained", trained), ("untrained", random)):
            started = time.monotonic()
            h, head, reconstruction = extract(model, g)
            base_values, base, base_residual = lowest(head, k, m, g["rigid"])
            basis, statistics = expanded_basis(
                vector_dictionary(h), head, m, g["rigid"]
            )
            values, u, residual = lowest(basis, k, m, g["rigid"])
            if np.any(values > base_values * (1 + 1e-7)):
                raise ValueError("expanded subspace worsens Ritz eigenvalue bound")
            for suffix, w, v in (("head", base_values, base), ("features", values, u)):
                np.savez(
                    args.output / f"object-{identity}-{name}-{suffix}.npz",
                    omega=np.sqrt(w),
                    field=v.reshape(-1, 3, 32)[ids],
                )
            row = {
                "object_id": identity,
                "name": name,
                **statistics,
                "head_trace": float(base_values.sum()),
                "feature_trace": float(values.sum()),
                "head_max_residual": float(base_residual.max()),
                "feature_max_residual": float(residual.max()),
                "linear_head_reconstruction_relative_error": reconstruction,
                "seconds": time.monotonic() - started,
                "files": {
                    suffix: {
                        "file": f"object-{identity}-{name}-{suffix}.npz",
                        "sha256": source.sha(
                            args.output / f"object-{identity}-{name}-{suffix}.npz"
                        ),
                    }
                    for suffix in ("head", "features")
                },
            }
            results.append(row)
            print({k: v for k, v in row.items() if k != "files"}, flush=True)
    shared.write(
        args.output / "generation.json",
        {
            "rows": results,
            "weights_sha256": receipt["weights_sha256"],
            "untrained_weights_sha256": source.sha(
                args.output / "untrained.safetensors"
            ),
            "script_sha256": source.sha(Path(__file__)),
            "scope": "TRAIN P1 geometry/operators only; frozen learned and one seed42 untrained network;32head vs up-to387hidden columns, numerical rank truncation explicit; no inverse correction or reference modal data, no new neural fit; not equal-cost or generalization claim",
        },
    )


def assess(args):
    report = json.loads((args.generated / "generation.json").read_text())
    args.output.mkdir(parents=True)
    results = []
    for identity in sorted(shared.TRAIN):
        k, m, _g, ids, audition = load_inputs(args.root, identity)
        values, u = eigsh(
            k,
            k=38,
            M=m,
            sigma=-1e-4,
            which="LM",
            tol=1e-9,
            v0=np.linspace(1, 2, k.shape[0]),
        )
        order = np.argsort(values)
        values, u = values[order], u[:, order]
        if max(abs(values[:6])) > values[6] * 1e-6 or values[6] <= 0:
            raise ValueError("reference rigid/elastic separation failed")
        variants = {
            "reference": {
                "omega": np.sqrt(values[6:]),
                "field": u[:, 6:].reshape(-1, 3, 32)[ids],
            }
        }
        np.savez(
            args.output / f"object-{identity}-reference.npz", **variants["reference"]
        )
        for row in (r for r in report["rows"] if r["object_id"] == identity):
            for suffix, record in row["files"].items():
                if row["name"] == "untrained" and suffix == "head":
                    continue
                path = args.generated / record["file"]
                if source.sha(path) != record["sha256"]:
                    raise ValueError("changed frozen feature result")
                with np.load(path, allow_pickle=False) as d:
                    variants[row["name"] + "-" + suffix] = dict(d)
        waves, rejected = probe.render_variants(variants, audition)
        gain = 0.5 / max(abs(w).max() for w in waves.values())
        scores = {}
        for name, wave in waves.items():
            if name == "reference":
                continue
            metrics = [
                shared.audio_metrics(a * gain, b * gain)
                for a, b in zip(waves["reference"], wave, strict=True)
            ]
            scores[name] = {
                "mean": {
                    key: float(np.mean([r[key] for r in metrics])) for key in metrics[0]
                },
                "trace_to_lower_bound": float(
                    sum(variants[name]["omega"] ** 2) / sum(values[6:])
                ),
                "frequency_mean_relative_error": float(
                    np.mean(
                        abs(
                            variants[name]["omega"] / variants["reference"]["omega"] - 1
                        )
                    )
                ),
            }
        order = list(waves)
        for contact in (0, 24, 47):
            for name, wave in waves.items():
                wavfile.write(
                    args.output / f"object-{identity}-contact-{contact}-{name}.wav",
                    source.RATE,
                    (wave[contact] * gain).astype(np.float32),
                )
            wavfile.write(
                args.output / f"object-{identity}-contact-{contact}-comparison.wav",
                source.RATE,
                (np.concatenate([waves[n][contact] for n in order]) * gain).astype(
                    np.float32
                ),
            )
        row = {
            "object_id": identity,
            "scores": scores,
            "order": order,
            "not_renderable": rejected,
            "common_gain_all_contacts_variants": gain,
        }
        results.append(row)
        print(row, flush=True)
    shared.write(
        args.output / "assessment.json",
        {
            "rows": results,
            "scope": "48contacts each on three TRAIN bodies, same P1 reference, common gain perbody; feature-oracle diagnosis NOT independentlytrained heads/DEV/pressure/realism",
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("generate", "assess"))
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--generated", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    torch.set_num_threads(4)
    {"generate": generate, "assess": assess}[args.stage](args)
