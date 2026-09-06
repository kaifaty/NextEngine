"""Internet numerical air/body teacher probe, NOT a learned guitar model.

Rettberg et al., DaRUS-3248 V1 (CC BY 4.0): one simplified guitar, fixed
bridge force port, two displacements and averaged near-hole internal pressure.
No mesh, recorded audio, arbitrary listener radiation or material-family claim.
Energy-inner-product projection preserves dissipativity for this fixed system.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
from scipy import linalg, signal, sparse
from scipy.io import loadmat, wavfile
from scipy.sparse.linalg import norm, splu

MD5 = "54e6f82be01f30e4d4a29392615a35c9"
SOURCE = "https://doi.org/10.18419/DARUS-3248"
RATE = 44100


def load_source(path):
    if hashlib.md5(path.read_bytes()).hexdigest() != MD5:
        raise ValueError("exact published DaRUS-3248 V1 matrices required")
    raw = loadmat(path, simplify_cells=True)["systemMatrices"]["pH_Vel"]
    d = {k: sparse.csc_matrix(raw[k]) for k in ("E", "J", "D", "Q", "A", "B", "C")}
    if any(not np.isfinite(v.data).all() for v in d.values()):
        raise ValueError("nonfinite source")
    if d["E"].shape != (11248, 11248) or d["C"].shape != (3, 11248):
        raise ValueError("source layout changed")
    if norm(d["A"] - (d["J"] - d["D"]) @ d["Q"]) != 0:
        raise ValueError("descriptor identity failed")
    # State [z(2454), q(3170), zdot(2454), pressure(3170)].
    # The file's C order is top displacement, back displacement, pressure,
    # NOT the ordering of the figure legend in the paper.
    if (
        d["C"][0].nonzero()[1].tolist() != [169]
        or d["C"][1].nonzero()[1].tolist() != [2231]
        or d["C"][2].nonzero()[1].min() < 8078
        or abs(d["C"][2].sum() - 1) > 1e-12
    ):
        raise ValueError("unexpected observation ports")
    return d


def transfer(d, frequency):
    omega = 2j * np.pi * frequency
    x = splu((omega * d["E"] - d["A"]).tocsc()).solve(d["B"].toarray()[:, 0])
    return x, np.asarray(d["C"] @ x).ravel()


def reduce_system(d, frequencies):
    h = (d["Q"].T @ d["E"]).tocsc()
    if norm(h - h.T) / norm(h) > 1e-12:
        raise ValueError("energy matrix not symmetric")
    columns = []
    for f in frequencies:
        x, _ = transfer(d, f)
        for v in (x.real, x.imag):
            energy = v @ (h @ v)
            if energy <= 0 or not np.isfinite(energy):
                raise ValueError("nonpositive response energy")
            columns.append(v / np.sqrt(energy))
        print("basis Hz", float(f), flush=True)
    s = np.column_stack(columns)
    gram = s.T @ (h @ s)
    values, vectors = linalg.eigh((gram + gram.T) / 2)
    keep = values > values[-1] * 1e-12
    v = s @ (vectors[:, keep] / np.sqrt(values[keep]))
    # Re-whiten accumulated floating-point error, not a dynamics repair.
    g = v.T @ (h @ v)
    lower = linalg.cholesky((g + g.T) / 2, lower=True)
    v = linalg.solve_triangular(lower, v.T, lower=True).T
    w = d["Q"] @ v
    a, b, c = w.T @ (d["A"] @ v), w.T @ d["B"].toarray(), d["C"] @ v
    energy_error = np.linalg.norm(v.T @ (h @ v) - np.eye(v.shape[1]))
    dissipative_max = linalg.eigvalsh((a + a.T) / 2).max()
    poles = linalg.eigvals(a)
    if energy_error > 1e-6 or dissipative_max > 1e-6 or poles.real.max() >= 0:
        raise ValueError("reduced passivity/stability check failed")
    return {
        "a": a,
        "b": b,
        "c": np.asarray(c),
        "port": d["B"].T @ w,
        "energy_error": float(energy_error),
        "dissipative_max": float(dissipative_max),
        "pole_real_max": float(poles.real.max()),
        "basis_hz": np.asarray(frequencies),
    }


def midpoint(d, force, dt):
    """Same integrator for full and reduced systems, zero initial state."""
    e, a = sparse.csc_matrix(d["E"]), sparse.csc_matrix(d["A"])
    left = splu(e - dt / 2 * a)
    right = e + dt / 2 * a
    b = np.asarray(d["B"].toarray() if sparse.issparse(d["B"]) else d["B"]).ravel()
    x = np.zeros(a.shape[0])
    y = np.zeros((len(force), d["C"].shape[0]))
    for i in range(1, len(force)):
        x = left.solve(right @ x + dt * b * (force[i - 1] + force[i]) / 2)
        y[i] = d["C"] @ x
    return y


def impulse(t, width):
    if not np.isfinite(width) or width <= 0:
        raise ValueError("positive finite pulse width required")
    # Smooth prescribed force, not an identified striker/contact law.
    return np.where(t < width, np.sin(np.pi * np.minimum(t / width, 1)) ** 2, 0)


def render(rom, width, duration=2.0):
    t = np.arange(round(duration * RATE)) / RATE
    u = impulse(t, width)
    # Exact LTI propagation for piecewise-linear sampled forcing.
    c = np.vstack([rom["c"][2], rom["port"]])
    _, y, _ = signal.lsim((rom["a"], rom["b"], c, np.zeros((2, 1))), u, t)
    if not np.isfinite(y).all():
        raise ValueError("nonfinite rendered response")
    return y


def relative(a, b):
    return (
        np.linalg.norm(a - b, axis=0) / np.maximum(np.linalg.norm(b, axis=0), 1e-30)
    ).tolist()


def full_audio(args):
    """Independent full-state artifact after rejected reduction; no new basis."""
    d = load_source(args.source)
    result = json.loads((args.output / "result.json").read_text())
    with np.load(args.output / "reduced.npz", allow_pickle=False) as saved:
        rom = dict(saved)
    rate = 20000
    time = np.arange(rate + 1) / rate
    # Additional observation only: physical force-port velocity, no state change.
    d["C"] = sparse.vstack([d["C"], d["B"].T @ d["Q"]], format="csc")
    full = midpoint(d, impulse(time, 0.005), 1 / rate)
    print("full one-second pressure calculated", flush=True)
    # Focused time-discretization countercheck, NOT mesh/material convergence.
    fine_time = np.arange(8001) / 40000
    fine = midpoint(d, impulse(fine_time, 0.005), 1 / 40000)
    rd = {
        "E": np.eye(len(rom["a"])),
        "A": rom["a"],
        "B": rom["b"],
        "C": np.vstack([rom["c"], rom["port"]]),
    }
    reduced = midpoint(rd, impulse(time, 0.005), 1 / rate)
    np.savez(
        args.output / "full-audio.npz",
        time=time,
        full=full,
        reduced=reduced,
        fine_time=fine_time,
        fine=fine,
    )
    gains = np.array([result["pressure_pcm_per_pa"], result["velocity_pcm_per_mps"]])
    audio = signal.resample_poly(full[:-1, [2, 3]], 441, 200, axis=0) * gains
    candidate = signal.resample_poly(reduced[:-1, [2, 3]], 441, 200, axis=0) * gains
    waves = {
        "full-pressure-5ms.wav": audio[:, 0],
        "full-velocity-proxy-5ms.wav": audio[:, 1],
        "full-velocity-then-pressure-5ms.wav": np.concatenate(
            [audio[:, 1], np.zeros(RATE // 2), audio[:, 0]]
        ),
        "full-then-reduced-pressure-5ms.wav": np.concatenate(
            [audio[:, 0], np.zeros(RATE // 2), candidate[:, 0]]
        ),
    }
    records = []
    for name, y in waves.items():
        if not np.isfinite(y).all() or abs(y).max() > 0.98:
            raise ValueError("full-reference audio exceeds fixed headroom")
        path = args.output / name
        wavfile.write(path, RATE, y.astype(np.float32))
        records.append(
            {
                "file": name,
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "samples": len(y),
            }
        )
    paper_band = None
    with np.load(args.output / "reference.npz", allow_pickle=False) as ref:
        mask = (ref["held_hz"] >= 82) & (ref["held_hz"] <= 320)
        paper_band = relative(ref["reduced_h"][mask], ref["full_h"][mask])
    check = {
        "full_state_rate": rate,
        "duration_seconds": 1,
        "columns": [*result["columns"], "force port velocity m/s"],
        "full_time_refinement_relative_l2": relative(full[:4001], fine[::2]),
        "full_vs_reduced_relative_l2": relative(reduced, full),
        "paper_band_held_frequency_relative_l2": paper_band,
        "new_fit_or_reduction": False,
        "neural_model": False,
        "gains_same_as_reduced_artifact": gains.tolist(),
        "no_mesh_convergence_or_realism_claim": True,
        "waves": records,
    }
    (args.output / "full-audio.json").write_text(json.dumps(check, indent=2) + "\n")
    print(json.dumps(check, indent=2), flush=True)


def run(args):
    d = load_source(args.source)
    args.output.mkdir(parents=True)
    # One fixed basis, no error-driven order/frequency sweep or audio fitting.
    frequencies = np.geomspace(60, 2000, 24)
    rom = reduce_system(d, frequencies)
    np.savez(args.output / "reduced.npz", **rom)
    rd = {"E": np.eye(len(rom["a"])), "A": rom["a"], "B": rom["b"], "C": rom["c"]}
    held = (frequencies[:-1] + frequencies[1:]) / 2
    full_h, reduced_h = [], []
    for f in held:
        _, y = transfer(d, f)
        full_h.append(y)
        reduced_h.append(
            rom["c"] @ linalg.solve(2j * np.pi * f * rd["E"] - rom["a"], rom["b"])[:, 0]
        )
    t = np.arange(2001) / 10000
    reference, approximate, errors = {}, {}, {}
    for name, force in (
        ("sine100", np.sin(2 * np.pi * 100 * t)),
        ("pulse5ms", impulse(t, 0.005)),
    ):
        reference[name] = midpoint(d, force, 0.0001)
        approximate[name] = midpoint(rd, force, 0.0001)
        errors[name] = relative(approximate[name], reference[name])
        print("full/reduced", name, errors[name], flush=True)
    np.savez(
        args.output / "reference.npz",
        time=t,
        full_h=full_h,
        reduced_h=reduced_h,
        held_hz=held,
        **reference,
        **{k + "_reduced": v for k, v in approximate.items()},
    )
    widths = (0.002, 0.005, 0.010)
    raw = [render(rom, width) for width in widths]
    # Different physical units: one common pressure gain and one velocity gain
    # across all widths. Comparison is timbre only, never absolute loudness.
    gains = 0.5 / np.max(np.abs(np.concatenate(raw)), axis=0)
    waves = {}
    for width, y in zip(widths, raw, strict=True):
        for col, label in enumerate(("pressure", "velocity-proxy")):
            waves[f"{label}-{round(width * 1000)}ms.wav"] = y[:, col] * gains[col]
    silence = np.zeros(RATE // 2)
    waves["velocity-then-pressure-5ms.wav"] = np.concatenate(
        [waves["velocity-proxy-5ms.wav"], silence, waves["pressure-5ms.wav"]]
    )
    waves["pressure-widths-2-5-10ms.wav"] = np.concatenate(
        [
            waves["pressure-2ms.wav"],
            silence,
            waves["pressure-5ms.wav"],
            silence,
            waves["pressure-10ms.wav"],
        ]
    )
    records = []
    for name, y in waves.items():
        if not np.isfinite(y).all() or np.max(np.abs(y)) > 0.98:
            raise ValueError("invalid audible artifact")
        path = args.output / name
        wavfile.write(path, RATE, y.astype(np.float32))
        records.append(
            {
                "file": name,
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "samples": len(y),
            }
        )
    result = {
        "source": SOURCE,
        "license": "CC BY 4.0",
        "source_md5": MD5,
        "source_sha256": hashlib.sha256(args.source.read_bytes()).hexdigest(),
        "scope": "one fixed numerical guitar; no neural fit, recording, mesh or far-field radiation",
        "order": len(rom["a"]),
        "energy_error": rom["energy_error"],
        "dissipative_max": rom["dissipative_max"],
        "pole_real_max": rom["pole_real_max"],
        "columns": [
            "top displacement m",
            "back displacement m",
            "averaged near-hole pressure Pa",
        ],
        "held_frequency_relative_l2": relative(np.array(reduced_h), np.array(full_h)),
        "time_relative_l2": errors,
        "time_check_dt": 0.0001,
        "reduction_check_pass": bool(
            max(x for row in errors.values() for x in row) < 0.05
        ),
        "pressure_pcm_per_pa": float(gains[0]),
        "velocity_pcm_per_mps": float(gains[1]),
        "pressure_gain_is_not_microphone_calibration": True,
        "force": "prescribed 1N sin-squared pulses; not a striker model",
        "paper_excitation_band_hz": [82, 320],
        "basis_band_hz": [60, 2000],
        "outside_paper_band_is_unvalidated": True,
        "waves": records,
    }
    (args.output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--full-audio", action="store_true")
    args = parser.parse_args()
    (full_audio if args.full_audio else run)(args)
