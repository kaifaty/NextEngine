"""Source-free audition of Rodrigo Diaz et al.'s published Neural Resonator.

External, report-only 2D synthetic-object experiment, not real-material admission.
No Lightning execution, target audio, dataset, or training at inference.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import time
from pathlib import Path
from typing import Optional

import numpy as np
import torch
from scipy import signal
from scipy.io import wavfile
from torch import nn

CHECKPOINT_SHA = "fa46fa2291ed595ec1daf07c5aa290aabb763d8e4e61aa8a3421cf0e7aa6e232"
SOURCE_HASHES = {
    "models.py": "0fcc3df024a8d073ef248fb5453a3c5ef66314ee2d1edb2add922faf7db794de",
    "dsp.py": "6dfb28a4a5ae543f6fcb1b6d92cdc21391a29d1720839d4a6b8328c987d8893a",
    "utilities.py": "315e52688b2a4519e82216f669a1e5ee135904173cd7873d04a5a67787d382bc",
}
INERT_GLOBALS = {
    "torch.optim.lr_scheduler.ExponentialLR",
    "torch.optim.adam.Adam",
    "torch.nn.modules.container.Sequential",
    "torch.nn.modules.linear.Identity",
    "torch.nn.modules.activation.LeakyReLU",
    "torch.nn.modules.linear.Linear",
    "neuralresonator.utilities.MelScaleLoss",
    "neuralresonator.models.FC",
    "neuralresonator.models.FCBlock",
    "neuralresonator.models.CoefficientsFC",
    "functools.partial",
}
# Published nbs/results.ipynb ranges; not an independently verified training manifest.
RANGES = np.array([(500, 15000), (8e9, 5e10), (0.1, 0.4), (1, 10), (3e-7, 2e-6)])
BASE = [7750.0, 2.9e10, 0.25, 5.5, 1.15e-6]
RATE = 32000


class InertMetadata:
    """Only carries data. Never calls a pickle-specified class, function or hook."""

    def __init__(self, *args, **kwargs):
        self.args, self.kwargs = args, kwargs

    def __setstate__(self, state):
        self.state = state


def checked_bytes(path: Path, expected: str) -> bytes:
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError(f"hash mismatch: {path.name}")
    return data


def load_checkpoint(path: Path) -> dict:
    checked_bytes(path, CHECKPOINT_SHA)
    names = set(torch.serialization.get_unsafe_globals_in_checkpoint(path))
    if names != INERT_GLOBALS:
        raise ValueError("unreviewed checkpoint globals")
    # All nonstandard references become inert carriers, NOT the original objects.
    with torch.serialization.safe_globals(
        [(InertMetadata, name) for name in sorted(names)]
    ):
        checkpoint = torch.load(path, weights_only=True, map_location="cpu")
    if any(
        not isinstance(v, torch.Tensor) or not torch.isfinite(v).all()
        for v in checkpoint["state_dict"].values()
    ):
        raise ValueError("invalid state tensor")
    return checkpoint


def upstream_definitions(assets: Path) -> dict:
    namespace = {"torch": torch, "nn": nn, "np": np, "List": list, "Optional": Optional}
    selections = {
        "dsp.py": {
            "mtanh",
            "constrain_complex_pole_or_zero",
            "pole_or_zero_to_iir_coeff",
            "apply_filter",
        },
        "utilities.py": {"to_zpk"},
        "models.py": {"FCBlock", "FC", "CoefficientsFC"},
    }
    for filename, names in selections.items():
        tree = ast.parse(checked_bytes(assets / filename, SOURCE_HASHES[filename]))
        nodes = [
            node
            for node in tree.body
            if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name in names
        ]
        if len(nodes) != len(names):
            raise ValueError("missing reviewed upstream definition")
        for node in nodes:
            node.decorator_list = []
        # Reviewed hash-pinned definitions only, not imports or package entry points.
        exec(  # noqa: S102
            compile(ast.Module(body=nodes, type_ignores=[]), filename, "exec"),
            namespace,
        )
    return namespace


def load_models(assets: Path):
    from torchvision.models import efficientnet_b0

    checkpoint = load_checkpoint(assets / "ethereal_dust-317-2.ckpt")
    definitions = upstream_definitions(assets)
    hp = checkpoint["hyper_parameters"]["model"].state
    expected = {
        "n_parallel": 32,
        "n_biquads": 2,
        "initial_gain_scale": 1.0,
        "tahn_c": 1.0,
        "tanh_d": 1.0,
        "use_zp_init": True,
    }
    if any(hp[key] != value for key, value in expected.items()):
        raise ValueError("unexpected checkpoint model hyperparameters")
    model = definitions["CoefficientsFC"](
        input_size=1007, hidden_sizes=[1024] * 6, **expected
    )
    encoder = efficientnet_b0(weights=None)
    state = checkpoint["state_dict"]
    model.load_state_dict(
        {
            k.removeprefix("model."): v
            for k, v in state.items()
            if k.startswith("model.")
        },
        strict=True,
    )
    encoder.load_state_dict(
        {
            k.removeprefix("encoder."): v
            for k, v in state.items()
            if k.startswith("encoder.")
        },
        strict=True,
    )
    if {k for k in state if not k.startswith(("model.", "encoder."))} != {
        "criterion.fb"
    }:
        raise ValueError("unexpected unused weights")
    for module in (model, encoder):
        module.eval().requires_grad_(False)
    return model, encoder, definitions, checkpoint["global_step"]


def material_input(values) -> torch.Tensor:
    values = np.asarray(values, dtype=np.float64)
    if values.shape != (5,) or not np.isfinite(values).all():
        raise ValueError("five finite material parameters required")
    scaled = (values - RANGES[:, 0]) / (RANGES[:, 1] - RANGES[:, 0])
    if np.any((scaled < 0) | (scaled > 1)):
        raise ValueError("outside published example material ranges")
    return torch.tensor(scaled, dtype=torch.float32)


def shape_mask(narrow: bool = False) -> torch.Tensor:
    # Own convex octagon on the published 64x64 occupancy grid; no image input.
    y, x = np.mgrid[:64, :64] / 64
    dx = abs(x - 0.5) / (0.25 if narrow else 0.4)
    dy = abs(y - 0.5) / 0.4
    return torch.tensor((dx <= 1) & (dy <= 1) & (dx + dy <= 1.5), dtype=torch.float32)


def render_coefficients(
    ba: np.ndarray, samples: int = RATE
) -> tuple[np.ndarray, float]:
    ba = np.asarray(ba, dtype=np.float64)
    if ba.shape != (32, 2, 6) or not np.isfinite(ba).all():
        raise ValueError("invalid IIR bank")
    if not np.all(ba[..., 3] == 1):
        raise ValueError("denominator must have unit leading coefficient")
    radius = max(float(abs(np.roots(a)).max()) for a in ba[..., 3:].reshape(-1, 3))
    if radius >= 1:
        raise ValueError("unstable or marginally stable IIR bank")
    impulse = np.zeros(samples)
    impulse[0] = 1.0
    audio = sum(signal.sosfilt(branch, impulse) for branch in ba)
    if not np.isfinite(audio).all() or abs(audio).max() < 1e-10:
        raise ValueError("nonfinite or silent output")
    return audio, radius


def waveform_observations(wave: np.ndarray) -> dict:
    wave = np.asarray(wave, dtype=np.float64)
    if wave.ndim != 1 or len(wave) != RATE or not np.isfinite(wave).all():
        raise ValueError("one full finite second required")
    power = wave**2
    if power.sum() <= 0:
        raise ValueError("silent observation")
    spectrum = abs(np.fft.rfft(wave))
    spectrum[0] = 0  # exclude DC from the diagnostic peak search only
    return {
        "dominant_hz": float(spectrum.argmax()),
        "energy_centroid_seconds": float(
            np.sum(power * np.arange(RATE) / RATE) / power.sum()
        ),
        "energy_after_200ms_fraction": float(power[RATE // 5 :].sum() / power.sum()),
    }


def material_relation_observations(records: list[dict]) -> dict:
    rows = {row["name"]: row for row in records}
    base = rows["base"]
    ratios = {}
    for name in ("lower-density", "higher-density", "stiffer"):
        row = rows[name]
        # Derived from K(E) u = omega^2 M(rho) u at fixed geometry/nu.
        # Undamped modal relation, not an exact law for a damped spectrum's peak.
        expected = np.sqrt(
            (row["material"][1] / row["material"][0])
            / (base["material"][1] / base["material"][0])
        )
        observed = row["dominant_hz"] / base["dominant_hz"]
        ratios[name] = {
            "undamped_expected_ratio": float(expected),
            "observed_dominant_peak_ratio": observed,
            "relative_ratio_error": float(observed / expected - 1),
        }
    return {
        "authority": "DESCRIPTIVE_SINGLE_SHAPE_RELATIONS / NOT_REALISM_OR_GENERALIZATION_GATE",
        "peak_ratios": ratios,
        "density_peak_order": rows["lower-density"]["dominant_hz"]
        > base["dominant_hz"]
        > rows["higher-density"]["dominant_hz"],
        "stiffness_peak_order": rows["stiffer"]["dominant_hz"] > base["dominant_hz"],
        "damping_energy_centroid_shorter": rows["more-damping"][
            "energy_centroid_seconds"
        ]
        < base["energy_centroid_seconds"],
    }


def run(assets: Path, output: Path) -> dict:
    if output.exists():
        raise ValueError("use a new output directory; preserve all candidates")
    output.mkdir(parents=True)
    torch.set_num_threads(4)
    torch.manual_seed(42)
    model, encoder, definitions, step = load_models(assets)
    cases = [("base", BASE, [0.5, 0.5], False)]
    for name, index, value in (
        ("lower-density", 0, 1500.0),
        ("higher-density", 0, 14000.0),
        ("stiffer", 1, 4.5e10),
        ("more-damping", 4, 1.8e-6),
    ):
        values = BASE.copy()
        values[index] = value
        cases.append((name, values, [0.5, 0.5], False))
    cases += [
        ("off-center", BASE, [0.65, 0.5], False),
        ("narrow-shape", BASE, [0.5, 0.5], True),
    ]
    waves, records = [], []
    for name, values, contact, narrow in cases:
        started = time.perf_counter()
        mask = shape_mask(narrow)
        with torch.inference_mode():
            features = encoder(mask[None, None].repeat(1, 3, 1, 1))
            inputs = torch.cat(
                [features, torch.tensor([contact]), material_input(values)[None]], -1
            )
            ba = model(inputs)[0].numpy()
            repeated = model(inputs)[0].numpy()
        if not np.array_equal(ba, repeated):
            raise ValueError("coefficient repeat mismatch")
        np.save(output / f"{name}-coefficients.npy", ba)
        np.save(output / f"{name}-mask.npy", mask.numpy())
        wave, radius = render_coefficients(ba)
        impulse = np.zeros(128)
        impulse[0] = 1.0
        reference = definitions["apply_filter"](impulse, ba[..., 3:], ba[..., :3])
        error = float(abs(reference - wave[:128]).max())
        if error > 1e-10:
            raise ValueError("scipy/upstream recurrence mismatch")
        np.save(output / f"{name}-raw.npy", wave)
        waves.append(wave)
        record = {
            "name": name,
            "material": values,
            "contact": contact,
            "narrow_shape": narrow,
            "max_pole_radius": radius,
            "raw_peak": float(abs(wave).max()),
            "rms": float(np.sqrt(np.mean(wave**2))),
            "recurrence_max_error": error,
            "seconds": time.perf_counter() - started,
            **waveform_observations(wave),
        }
        records.append(record)
        print(json.dumps(record), flush=True)
    gain = 0.5 / max(float(abs(wave).max()) for wave in waves)
    pcm = [(wave * gain).astype(np.float32) for wave in waves]
    for record, wave in zip(records, pcm):
        path = output / f"{record['name']}.wav"
        wavfile.write(path, RATE, wave)
        record["wav_sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        record["relative_wave_delta_vs_base"] = float(
            np.linalg.norm(wave - pcm[0]) / np.linalg.norm(pcm[0])
        )
    comparison = np.concatenate(
        [part for wave in pcm for part in (wave, np.zeros(RATE // 2, np.float32))]
    )
    wavfile.write(output / "comparison.wav", RATE, comparison)
    result = {
        "claim": "SOURCE_FREE_PUBLISHED_NEURAL_2D_SYNTHETIC_PRIOR / NO_REALISM_OR_NEW_OBJECT_ADMISSION",
        "checkpoint_sha256": CHECKPOINT_SHA,
        "upstream_global_step": step,
        "records": records,
        "sample_rate": RATE,
        "samples_per_case": RATE,
        "shared_playback_gain": gain,
        "inference_device": "cpu",
        "torch": torch.__version__,
        "target_audio_input": False,
        "dataset_payload_opened": False,
        "trained_here": False,
        "cases_per_model": len(cases),
        "material_relations": material_relation_observations(records),
    }
    (output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run(args.assets, args.output)
