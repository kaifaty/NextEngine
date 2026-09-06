"""TRAIN-only audible representation discriminator; no training/promotion.

Full-band modal packing inspired by Deep-Modal (Jin et al., MM2020), but not
its published 32-band/100-10000Hz recipe or a reproduction of its network.
Compare signed-sum packing with a three-axis response-energy Gram matrix.
Gram is integrated waveform covariance, NOT a mechanical power/admittance port.
Both controls consume target modal arrays and are oracle RECONSTRUCTIONS.
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

BANDS = 128
SAMPLES = 2 * teacher.RATE


def basis_gram(frequency, damping):
    """Exact finite discrete-sample sine Gram via geometric series, float64."""
    f, d = np.asarray(frequency, float), np.asarray(damping, float)
    if (
        f.ndim != 1
        or d.shape != f.shape
        or not len(f)
        or not np.isfinite([f, d]).all()
        or np.any(d <= 0)
        or np.any((f <= 0) | (f >= teacher.RATE / 2))
    ):
        raise ValueError("finite positive sub-Nyquist poles required")
    decay = -d[:, None] - d[None, :]
    pieces = []
    for angular in (
        2 * np.pi * (f[:, None] - f[None, :]),
        2 * np.pi * (f[:, None] + f[None, :]),
    ):
        exponent = (decay + 1j * angular) / teacher.RATE
        pieces.append((np.expm1(SAMPLES * exponent) / np.expm1(exponent)).real)
    return (pieces[0] - pieces[1]) / (2 * teacher.RATE)


def psd_root(matrix, inverse=False):
    matrix = np.asarray(matrix, float)
    if matrix.shape != (3, 3) or not np.isfinite(matrix).all():
        raise ValueError("finite 3x3 Gram required")
    values, vectors = np.linalg.eigh((matrix + matrix.T) / 2)
    # Explicit floating-point eigendecomposition allowance, not an audio gate.
    bound = (
        64
        * np.finfo(float).eps
        * max(float(np.linalg.norm(matrix, 2)), np.finfo(float).tiny)
    )
    if values.min() < -bound:
        raise ValueError("indefinite response-energy Gram")
    if inverse:
        if values.min() <= 0 or values.max() / values.min() > 1e12:
            raise ValueError("carrier Gram ill-conditioned")
        diagonal = 1 / np.sqrt(values)
    else:
        diagonal = np.sqrt(np.maximum(values, 0))
    return (vectors * diagonal) @ vectors.T


def orthogonal_carriers(frequency, damping):
    """Sample-space QR avoids squaring the condition number in Gram inversion."""
    t = np.arange(SAMPLES) / teacher.RATE
    basis = np.exp(-np.asarray(damping)[:, None] * t) * np.sin(
        2 * np.pi * np.asarray(frequency)[:, None] * t
    )
    q, r = np.linalg.qr(basis.T, mode="reduced")
    if np.min(abs(r.diagonal())) <= 64 * np.finfo(float).eps * np.linalg.norm(basis):
        raise ValueError("rank-deficient carrier basis")
    # Fix QR column signs, not a cross-machine bit reproducibility contract.
    q *= np.sign(r.diagonal())
    return q.T * np.sqrt(teacher.RATE)


def axis_responses(values):
    f, d, g = values["frequency"], values["damping"], values["gains"]
    sizes = values.get("group_sizes", np.array([len(f)]))
    orthogonalized = values.get("orthogonalized", np.zeros(len(sizes), dtype=bool))
    output = np.zeros((3, SAMPLES))
    offset = 0
    t = np.arange(SAMPLES) / teacher.RATE
    for size, orthogonal in zip(sizes, orthogonalized, strict=True):
        selected = slice(offset, offset + size)
        if orthogonal:
            output += g[:, selected] @ orthogonal_carriers(f[selected], d[selected])
        else:
            for start in range(offset, offset + size, 128):
                chunk = slice(start, min(start + 128, offset + size))
                basis = np.exp(-d[chunk, None] * t) * np.sin(
                    2 * np.pi * f[chunk, None] * t
                )
                output += g[:, chunk] @ basis
        offset += size
    if offset != len(f) or not np.isfinite(output).all():
        raise ValueError("invalid response groups")
    return np.pad(output, ((0, 0), (0, teacher.RATE)))


def compress(frequency, damping, gains):
    f, d, g = (
        np.asarray(frequency, float),
        np.asarray(damping, float),
        np.asarray(gains, float),
    )
    basis_gram(f[:1], d[:1])
    if (
        g.shape != (3, len(f))
        or d.shape != f.shape
        or not np.isfinite([f, d]).all()
        or not np.isfinite(g).all()
        or np.any((f < 1) | (f > 22049))
        or np.any(d <= 0)
    ):
        raise ValueError("bounded full modal field required")
    # Fixed before observation, covers every source mode; no frequency crop.
    edges = (
        np.expm1(np.linspace(np.log1p(1 / 700), np.log1p(22049 / 700), BANDS + 1)) * 700
    )
    band = np.minimum(np.searchsorted(edges[1:], f, side="right"), BANDS - 1)
    output = {
        k: {
            "frequency": [],
            "damping": [],
            "gains": [],
            "group_sizes": [],
            "orthogonalized": [],
        }
        for k in ("signed_sum", "energy_gram")
    }
    records = []
    for b in sorted(set(band.tolist())):
        ix = np.flatnonzero(band == b)
        fb, db, gb = f[ix], d[ix], g[:, ix]
        kernel = basis_gram(fb, db)
        gram = gb @ kernel @ gb.T
        energy = (gb**2).sum(0) * kernel.diagonal()
        weights = (
            energy / energy.sum() if energy.sum() > 0 else np.full(len(ix), 1 / len(ix))
        )
        if len(ix) <= 3:
            packed = {k: (fb, db, gb) for k in output}
        else:
            damping_mean = float(weights @ db)
            center = float(weights @ fb)
            carrier_f = fb.min() + np.array([1 / 6, 1 / 2, 5 / 6]) * np.ptp(fb)
            carrier_d = np.full(3, damping_mean)
            coefficients = psd_root(gram)
            packed = {
                "signed_sum": (
                    np.array([center]),
                    np.array([damping_mean]),
                    gb.sum(1)[:, None],
                ),
                "energy_gram": (carrier_f, carrier_d, coefficients),
            }
        pf, pd, pg = packed["energy_gram"]
        if len(ix) > 3:
            response = pg @ orthogonal_carriers(pf, pd)
            reproduced = response @ response.T / teacher.RATE
        else:
            reproduced = pg @ basis_gram(pf, pd) @ pg.T
        error = float(
            np.linalg.norm(reproduced - gram)
            / max(np.linalg.norm(gram), np.finfo(float).tiny)
        )
        # Narrow numerical contract only: within-band covariance, not total
        # cross-band interference, perceptual fidelity or calibrated energy.
        if error > 1e-9:
            raise ValueError("within-band energy reconstruction error")
        records.append(
            {
                "band": b,
                "source_modes": len(ix),
                "energy_modes": len(pf),
                "gram_relative_error": error,
            }
        )
        for kind, (pf, pd, pg) in packed.items():
            output[kind]["frequency"].extend(pf.tolist())
            output[kind]["damping"].extend(pd.tolist())
            output[kind]["gains"].append(pg)
            output[kind]["group_sizes"].append(len(pf))
            output[kind]["orthogonalized"].append(kind == "energy_gram" and len(ix) > 3)
    for values in output.values():
        values["frequency"] = np.array(values["frequency"])
        values["damping"] = np.array(values["damping"])
        values["gains"] = np.concatenate(values["gains"], axis=1)
        values["group_sizes"] = np.array(values["group_sizes"])
        values["orthogonalized"] = np.array(values["orthogonalized"])
    return output, records


def run(args):
    rows = json.loads((args.data / "train.json").read_text())["rows"]
    if (
        len(rows) != 6
        or {r["object_id"] for r in rows} != shared.TRAIN
        or any(r["role"] != "train" for r in rows)
    ):
        raise ValueError("exact TRAIN-only projection required")
    args.output.mkdir(parents=True)
    results, snippets = [], []
    for row in rows:
        path = args.data / row["file"]
        if teacher.sha(path) != row["sha256"]:
            raise ValueError("teacher changed")
        with np.load(path, allow_pickle=False) as saved:
            data = dict(saved)
        identity = row["object_id"]
        packed, bands = compress(data["frequency"], data["damping"], data["gains"][0])
        parameters = {
            "teacher": {
                "frequency": data["frequency"],
                "damping": data["damping"],
                "gains": data["gains"][0],
            },
            **packed,
        }
        for kind, values in parameters.items():
            np.savez(args.output / f"object-{identity}-{kind}.npz", **values)
        responses = {kind: axis_responses(v) for kind, v in parameters.items()}
        waves = {kind: np.ones(3) @ response for kind, response in responses.items()}
        metrics = {
            kind: audio_metrics(waves["teacher"], waves[kind]) for kind in packed
        }
        total_grams = {
            kind: response @ response.T / teacher.RATE
            for kind, response in responses.items()
        }
        gram_errors = {
            kind: float(
                np.linalg.norm(total_grams[kind] - total_grams["teacher"])
                / max(np.linalg.norm(total_grams["teacher"]), np.finfo(float).tiny)
            )
            for kind in packed
        }
        # Total covariance is evaluated, not forced to match after packing.
        # Preserve failures from cross-band interference instead of normalizing.
        gain = min(1.0, 0.5 / max(float(abs(w).max()) for w in waves.values()))
        gallery = np.concatenate([w * gain for w in waves.values()]).astype(np.float32)
        if not np.isfinite(gallery).all():
            raise ValueError("nonfinite comparison")
        name = f"object-{identity}-teacher-signed-energy.wav"
        wavfile.write(args.output / name, teacher.RATE, gallery)
        snippets.append(np.concatenate([waves["teacher"], waves["energy_gram"]]) * gain)
        result = {
            "object_id": identity,
            "source_modes": len(data["frequency"]),
            "packed_modes": {k: len(v["frequency"]) for k, v in packed.items()},
            "bands": bands,
            "metrics": metrics,
            "total_gram_relative_error": gram_errors,
            "comparison": name,
            "common_gain": gain,
            "sha256": teacher.sha(args.output / name),
        }
        results.append(result)
        print(json.dumps({k: v for k, v in result.items() if k != "bands"}), flush=True)
    summary = {
        kind: {
            metric: float(np.mean([r["metrics"][kind][metric] for r in results]))
            for metric in ("spectrum", "envelope", "level")
        }
        for kind in packed
    }
    wavfile.write(
        args.output / "six-train-teacher-energy-reconstruction.wav",
        teacher.RATE,
        np.concatenate(snippets).astype(np.float32),
    )
    shared.write_json(
        args.output / "probe.json",
        {
            "rows": results,
            "summary": summary,
            "bands": BANDS,
            "band_gram_numerical_tolerance": 1e-9,
            "scope": "target-parameter reconstruction; fixed six TRAIN first contacts; no fit, not source-free sound; within-band energy only, no global normalization",
            "source": "https://hellojxt.github.io/DeepModal/ACMMM20_ModalSound.pdf",
            "script_sha256": teacher.sha(Path(__file__)),
        },
    )
    print(json.dumps(summary, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    run(parser.parse_args())
