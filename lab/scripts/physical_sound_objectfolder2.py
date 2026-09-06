"""Bounded ObjectFolder2 AudioNet teacher, not a shared/newly trained model.

Gao et al., CVPR2022, upstream CC BY4.0; preserve original mesh-source terms.
Models encode modal vibration, NOT calibrated pressure or a striker material.
Unlike the author's WAV exporter, retain linear force scaling and finite zero.
No downloaded imports, CUDA extensions, optimizer execution or unsafe pickle.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
from pathlib import Path

import numpy as np
import torch
from scipy.io import wavfile
from torch import nn
from torch.nn import functional as F

REVISION = "3c6cd8930b2dcbadb6d94dadf2745c956bdcd236"
MODEL_SOURCE = "c54c434ec895601ff42963fa15f1c260ca0ea68d7b782a430388e7ca4567673b"
DEMO_WEIGHTS = "593d80908a7cd7be8648af388f148b9e3f2fc701e891e663136607c095190ac7"
RATE = 44100
NUMPY_GLOBALS = {
    "numpy.dtype",
    "numpy.ndarray",
    "numpy.core.multiarray.scalar",
    "numpy.core.multiarray._reconstruct",
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_definitions(path):
    if sha(path) != MODEL_SOURCE:
        raise ValueError("unreviewed AudioNet source")
    selected = {"DenseLayer", "Embedder", "get_embedder", "AudioNeRF", "NeRF"}
    tree = ast.parse(path.read_text())
    definitions = [
        n
        for n in tree.body
        if isinstance(n, (ast.ClassDef, ast.FunctionDef)) and n.name in selected
    ]
    if {n.name for n in definitions} != selected:
        raise ValueError("incomplete reviewed definitions")
    namespace = {"torch": torch, "nn": nn, "np": np, "F": F}
    # Only hash-pinned, directly reviewed declarations; no circular imports,
    # global anomaly setting, filesystem helpers, downloads or device setup.
    exec(  # noqa: S102 -- Hash-pinned, audited declarations only; imports excluded.
        compile(ast.Module(body=definitions, type_ignores=[]), str(path), "exec"),
        namespace,
    )
    return namespace


def load_audio(path, expected):
    if sha(path) != expected:
        raise ValueError("checkpoint changed")
    if set(torch.serialization.get_unsafe_globals_in_checkpoint(path)) != NUMPY_GLOBALS:
        raise ValueError("unreviewed serialized globals")
    # Explicit numeric NumPy carriers only; weights_only remains enabled.
    with torch.serialization.safe_globals(
        [
            (np._core.multiarray._reconstruct, "numpy.core.multiarray._reconstruct"),
            (np._core.multiarray.scalar, "numpy.core.multiarray.scalar"),
            np.ndarray,
            np.dtype,
            np.dtypes.Float32DType,
            np.dtypes.Float64DType,
            np.dtypes.UInt32DType,
            np.dtypes.Int64DType,
            np.dtypes.Int32DType,
        ]
    ):
        audio = torch.load(path, weights_only=True, map_location="cpu")["AudioNet"]
    frequency, damping = np.asarray(audio["frequencies"]), np.asarray(audio["dampings"])
    if (
        frequency.ndim != 1
        or damping.shape != frequency.shape
        or not 1 <= len(frequency) <= 4096
        or not np.isfinite(frequency).all()
        or not np.isfinite(damping).all()
        or np.any((frequency <= 0) | (frequency >= RATE / 2))
        or np.any(damping <= 0)
    ):
        raise ValueError("invalid audible modal metadata")
    norm = audio["normalizer"]
    keys = {
        "xyz_min",
        "xyz_max",
        "f1_min",
        "f1_max",
        "f2_min",
        "f2_max",
        "f3_min",
        "f3_max",
    }
    if set(norm) != keys or not all(
        np.ndim(v) == 0 and np.isfinite(v) for v in norm.values()
    ):
        raise ValueError("invalid scalar normalizer")
    if any(norm[p + "_max"] <= norm[p + "_min"] for p in ("xyz", "f1", "f2", "f3")):
        raise ValueError("empty normalization interval")
    weights = audio["model_state_dict"]
    if not all(
        k.startswith("module.")
        and isinstance(v, torch.Tensor)
        and torch.isfinite(v).all()
        for k, v in weights.items()
    ):
        raise ValueError("invalid AudioNet weights")
    return audio


def point_gains(audio, definitions, points):
    points = np.asarray(points, dtype=np.float64)
    norm = audio["normalizer"]
    normalized = (points - norm["xyz_min"]) / (norm["xyz_max"] - norm["xyz_min"])
    if (
        points.ndim != 2
        or points.shape[1] != 3
        or not np.isfinite(points).all()
        or np.any((normalized < 0) | (normalized > 1))
    ):
        raise ValueError("points outside declared coordinate support")
    embed, channels = definitions["get_embedder"](10, 0)
    model = definitions["AudioNeRF"](
        D=8, input_ch=channels, output_ch=len(audio["frequencies"])
    )
    model.load_state_dict(
        {k.removeprefix("module."): v for k, v in audio["model_state_dict"].items()},
        strict=True,
    )
    model.eval().requires_grad_(False)
    with torch.no_grad():
        x = embed(torch.tensor(normalized, dtype=torch.float32))
        predictions = model(x, x, x)
        gains = torch.stack(
            [
                value * (norm[f"f{i}_max"] - norm[f"f{i}_min"]) + norm[f"f{i}_min"]
                for i, value in enumerate(predictions, 1)
            ],
            dim=1,
        ).numpy()
    if not np.isfinite(gains).all():
        raise ValueError("nonfinite neural gains")
    return gains


def waveform(gains, frequency, damping, force):
    force = np.asarray(force, dtype=np.float64)
    if force.shape != (3,) or not np.isfinite(force).all():
        raise ValueError("finite three-axis excitation required")
    combined = force @ gains
    t = np.arange(RATE * 2, dtype=np.float64) / RATE
    y = combined @ (
        np.exp(-np.asarray(damping)[:, None] * t)
        * np.sin(2 * np.pi * np.asarray(frequency)[:, None] * t)
    )
    # Full author's two-second ringing horizon plus one second of silence,
    # directly evaluating paper Eq8 without its two-sample FFT padding delay.
    return np.pad(y, (0, RATE))


def pcm_scaling_check(base, scaled, factor):
    """Report bit identity separately from explicit PCM rounding uncertainty.

    Two quantized signals need not scale bitwise in the subnormal tail. One
    ULP per operand is a conservative representational bound, not an acoustic
    tolerance. Zero-force silence must still be checked exactly and separately.
    """
    if (
        base.dtype != np.float32
        or scaled.dtype != np.float32
        or base.shape != scaled.shape
    ):
        raise ValueError("matching float32 PCM required")
    if (
        not np.isfinite(base).all()
        or not np.isfinite(scaled).all()
        or not np.isfinite(factor)
    ):
        raise ValueError("finite PCM and scale required")
    expected = base.astype(np.float64) * factor
    error = np.abs(scaled.astype(np.float64) - expected)
    bound = abs(np.spacing(scaled)).astype(np.float64) + abs(factor) * abs(
        np.spacing(base)
    ).astype(np.float64)
    return {
        "bit_exact": bool(np.array_equal(scaled, base * factor)),
        "within_pcm_rounding_bound": bool(np.all(error <= bound)),
        "max_absolute_error": float(error.max()),
    }


def run(args):
    torch.set_num_threads(4)
    audio = load_audio(args.data / "ObjectFile.pth", args.weights_sha256)
    definitions = source_definitions(args.source / "AudioNet_model.py")
    vertices = np.array(
        [
            [float(x) for x in line.split()[1:]]
            for line in (args.data / "model.obj").read_text().splitlines()
            if line.startswith("v ")
        ]
    )
    if vertices.ndim != 2 or vertices.shape[1] != 3 or not np.isfinite(vertices).all():
        raise ValueError("invalid OBJ vertices")
    indices = np.arange(4) * (len(vertices) // 4)
    points = vertices[indices]
    gains = point_gains(audio, definitions, points)
    conditions = [(i, np.ones(3), f"contact-{i}") for i in range(4)]
    conditions += [(0, np.ones(3) * x, f"force-{x:g}") for x in (0, 0.25, 0.5, 2)]
    conditions += [
        (0, f, name)
        for f, name in (
            ([1, 0, 0], "force-x"),
            ([0, 1, 0], "force-y"),
            ([0, 0, 1], "force-z"),
            ([-1, -1, -1], "force-negative"),
        )
    ]
    raw = {
        name: waveform(gains[i], audio["frequencies"], audio["dampings"], force)
        for i, force, name in conditions
    }
    peak = max(float(abs(y).max()) for y in raw.values())
    if not np.isfinite(peak) or peak <= 0:
        raise ValueError("no finite audible response")
    common_gain = 0.5 / peak
    args.output.mkdir(parents=True)
    np.savez(
        args.output / "generated.npz",
        points=points,
        vertex_indices=indices,
        gains=gains,
        frequency=audio["frequencies"],
        damping=audio["dampings"],
    )
    waves = {name + ".wav": y * common_gain for name, y in raw.items()}
    gap = np.zeros(RATE // 2)
    waves["force-quarter-half-one-two.wav"] = np.concatenate(
        [
            waves["force-0.25.wav"],
            gap,
            waves["force-0.5.wav"],
            gap,
            waves["contact-0.wav"],
            gap,
            waves["force-2.wav"],
        ]
    )
    waves["four-mesh-contacts.wav"] = np.concatenate(
        [
            waves["contact-0.wav"],
            gap,
            waves["contact-1.wav"],
            gap,
            waves["contact-2.wav"],
            gap,
            waves["contact-3.wav"],
        ]
    )
    records = []
    for name, y in waves.items():
        if not np.isfinite(y).all() or abs(y).max() > 0.98:
            raise ValueError("invalid WAV")
        path = args.output / name
        wavfile.write(path, RATE, y.astype(np.float32))
        records.append({"file": name, "samples": len(y), "sha256": sha(path)})
    checks = {
        "zero_exact": bool(np.all(raw["force-0"] == 0)),
        "half_exact": bool(np.array_equal(raw["force-0.5"], raw["contact-0"] * 0.5)),
        "double_exact": bool(np.array_equal(raw["force-2"], raw["contact-0"] * 2)),
        "negative_exact": bool(
            np.array_equal(raw["force-negative"], -raw["contact-0"])
        ),
        "axis_superposition_relative_l2": float(
            np.linalg.norm(
                raw["force-x"] + raw["force-y"] + raw["force-z"] - raw["contact-0"]
            )
            / np.linalg.norm(raw["contact-0"])
        ),
    }
    checks["pcm_scaling"] = {
        name: pcm_scaling_check(
            waves["contact-0.wav"].astype(np.float32),
            waves[name + ".wav"].astype(np.float32),
            factor,
        )
        for name, factor in (("force-0.5", 0.5), ("force-2", 2), ("force-negative", -1))
    }
    checks["pcm_zero_half_double_sign_pass"] = checks["zero_exact"] and all(
        check["within_pcm_rounding_bound"] for check in checks["pcm_scaling"].values()
    )
    result = {
        "revision": REVISION,
        "weights_sha256": args.weights_sha256,
        "mesh_sha256": sha(args.data / "model.obj"),
        "license": "upstream CC BY4.0; retain original mesh-source terms",
        "scope": "published object-specific neural vibration teacher; NOT pressure, new-object generalization or our training",
        "material_identity": "not inferred from demo timbre/geometry",
        "force_units": "upstream axis coefficients; not calibrated newton-second or striker material",
        "common_pcm_gain": common_gain,
        "upstream_per_wave_peak_normalization": "omitted to preserve force scaling and zero",
        "coordinate_mapping": "upstream scalar min/max to [0,1], despite [-1,1] comment",
        "vertex_indices": indices.tolist(),
        "points": points.tolist(),
        "mode_count": len(audio["frequencies"]),
        "checks": checks,
        "waves": records,
    }
    (args.output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--weights-sha256", default=DEMO_WEIGHTS)
    run(parser.parse_args())
