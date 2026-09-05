"""Pinned upstream input-collision diagnostic; NOT SonicGauss audio inference.

Runs only reviewed normalization definitions, not repository imports or weights.
The optional modal audition is a frequency-control illustration, NOT neural
output, calibrated size transfer, or a realism/admission test.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import itertools
import json
from pathlib import Path
from urllib.request import urlopen

import numpy as np
import torch
from scipy.io import wavfile

SONIC = "AiEson/SonicGauss/7a5687afbe6d4338f8e569b3738c3fa7fa62304a"
SPLAT = "ChenYutongTHU/SplatFormer/446ffb5dd1c35b4b8f94953a22046bde5714a094"
SOURCES = {
    "common.py": (
        f"{SONIC}/stage2/common.py",
        "d57fc0e5b6bbe54ea4fe7b1d2d9ec409ea60d46c9dfad83cc2518bb110c32650",
    ),
    "transform_utils.py": (
        f"{SPLAT}/utils/transform_utils.py",
        "ced86c22947a8050deee9b95b9c159b9bab77da7cb919212993872a6848b7797",
    ),
    "ptv3.gin": (
        f"{SPLAT}/configs/model/ptv3.gin",
        "010a8c459a204fd37e69f5693082e40e14d1c4ce1a1e87a30e44b96fadf81e62",
    ),
    "infer_3.py": (
        f"{SONIC}/stage3/infer_3.py",
        "9cccda8ad6940bc1e4b02491302dd1674191c70ba05855f738d8a2cd154df88b",
    ),
}
PROFILE_SHA = "7939326bb4b1e09f9b6b89c3a2f4099708edbe35c953d710db3c100c6bd007f7"


def verified(data: bytes, expected: str) -> bytes:
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError("source hash mismatch; review changed code before execution")
    return data


def load_definitions(sources: dict[str, bytes]) -> dict:
    """Hash-check BEFORE AST execution; no arbitrary upstream entry point."""
    for name, (_, expected) in SOURCES.items():
        verified(sources[name], expected)
    if b"MinMaxScaler" in sources["ptv3.gin"]:
        raise ValueError("unexpected scaler gin binding")
    namespace = {"torch": torch}
    for filename, names in (
        ("transform_utils.py", {"MinMaxScaler"}),
        ("common.py", {"preprocess_gaussian", "normalize_position"}),
    ):
        nodes = [
            node
            for node in ast.parse(sources[filename]).body
            if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name in names
        ]
        if len(nodes) != len(names):
            raise ValueError("missing or duplicate reviewed definition")
        for node in nodes:
            # This published gin config has no scaler overrides. Use defaults.
            node.decorator_list = []
        # Exact hash-pinned, reviewed definitions only; no imports/entry points.
        exec(  # noqa: S102
            compile(ast.Module(body=nodes, type_ignores=[]), filename, "exec"),
            namespace,
        )
    return namespace


def gaussian_fixture() -> dict[str, torch.Tensor]:
    """Own tensor fixture, NOT a recovered real object or a metered 3D scan."""
    means = torch.tensor(
        list(itertools.product((-0.5, 0.5), (-0.25, 0.25), (-0.125, 0.125)))
    )
    return {
        "means": means,
        "scales": torch.zeros(8, 3),
        "opacities": torch.ones(8, 1),
        "quats": torch.tensor([[1.0, 0.0, 0.0, 0.0]]).repeat(8, 1),
        "features_dc": torch.zeros(8, 3),
        "features_rest": torch.zeros(8, 3, 3),
    }


def input_changes(base: dict, candidate: dict) -> dict:
    if base.keys() != candidate.keys():
        raise ValueError("different input keys")
    return {
        key: {
            "exact_equal": torch.equal(base[key], candidate[key]),
            "max_abs_delta": float((base[key] - candidate[key]).abs().max()),
        }
        for key in base
    }


def probe(namespace: dict) -> dict:
    def inputs(gs, position):
        result, scaler = namespace["preprocess_gaussian"](
            gs, "cpu", scaler_class=namespace["MinMaxScaler"], return_scaler=True
        )
        result["position"] = namespace["normalize_position"](position, scaler, "cpu")
        result["grid_coord"] = torch.floor(result["means"] * 384).int()
        return result

    original = gaussian_fixture()
    point = torch.tensor([0.5, 0.25, 0.125])
    base = inputs(original, point)
    doubled = {key: value.clone() for key, value in original.items()}
    doubled["means"] *= 2
    doubled["scales"] += torch.log(torch.tensor(2.0))
    stretched = {key: value.clone() for key, value in original.items()}
    stretched["means"][:, 1] *= 2
    stretched["scales"][:, 1] += torch.log(torch.tensor(2.0))
    changed_appearance = {key: value.clone() for key, value in original.items()}
    changed_appearance["features_dc"][:, 0] = 0.25
    cases = {
        "uniform_scale_2": inputs(doubled, point * 2),
        "relative_contact": inputs(original, [-0.5, 0.25, 0.125]),
        "relative_shape": inputs(stretched, [0.5, 0.5, 0.125]),
        "appearance": inputs(changed_appearance, point),
    }
    return {name: input_changes(base, candidate) for name, candidate in cases.items()}


def modal_control(profile: dict, frequency_ratio: float) -> np.ndarray:
    """Deliberate frequency-only intervention; damping/gains remain fixed."""
    if not np.isfinite(frequency_ratio) or not 0 < frequency_ratio <= 1:
        raise ValueError("frequency ratio must be in (0, 1]")
    clock = np.arange(32_000, dtype=np.float64) / 32_000
    wave = np.zeros_like(clock)
    for mode in profile["modes"]:
        frequency = mode["damped_frequency_hz"] * frequency_ratio
        damping, amplitude = mode["damping_per_second"], mode["ridge_amplitude"]
        if not np.isfinite([frequency, damping, amplitude]).all():
            raise ValueError("nonfinite modal parameter")
        if not 0 < frequency < 16_000 or damping <= 0:
            raise ValueError("invalid modal frequency or damping")
        wave += (
            amplitude * np.exp(-damping * clock) * np.sin(2 * np.pi * frequency * clock)
        )
    if not np.isfinite(wave).all() or abs(wave).max() < 1e-8:
        raise ValueError("invalid modal control audio")
    # No target WAV, sampled transient, learned decoding, gating or tail crop.
    return wave


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--modal-profile", type=Path)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    if (args.output / "result.json").exists():
        raise ValueError("result already exists; preserve the previous experiment")
    snapshots = args.output / "sources"
    snapshots.mkdir(exist_ok=True)
    sources = {}
    for name, (path, expected) in SOURCES.items():
        target = snapshots / name
        if target.exists():
            data = target.read_bytes()
        elif args.offline:
            raise ValueError(f"missing offline source: {name}")
        else:
            with urlopen(
                f"https://raw.githubusercontent.com/{path}", timeout=30
            ) as response:
                data = response.read(100_000)
            verified(data, expected)
            target.write_bytes(data)
        sources[name] = verified(data, expected)
    changes = probe(load_definitions(sources))
    result = {
        "claim": "INPUT_DIAGNOSTIC_ONLY / NO_NEURAL_INFERENCE_OR_REALISM_ADMISSION",
        "sources": SOURCES,
        "torch": torch.__version__,
        "device": "cpu",
        "dtype": "float32",
        "cases": changes,
        "scale_collision": all(
            row["exact_equal"] for row in changes["uniform_scale_2"].values()
        ),
        "positive_controls_change": all(
            any(not row["exact_equal"] for row in changes[name].values())
            for name in ("relative_contact", "relative_shape", "appearance")
        ),
        "weights_loaded": False,
        "dataset_payload_opened": False,
    }
    if args.modal_profile:
        profile = json.loads(verified(args.modal_profile.read_bytes(), PROFILE_SHA))
        waves = [modal_control(profile, ratio) for ratio in (1.0, 0.5)]
        gain = 0.5 / max(float(abs(wave).max()) for wave in waves)
        segments = [(wave * gain).astype(np.float32) for wave in waves]
        for index, wave in enumerate(segments):
            np.save(args.output / f"modal-control-{index}-raw.npy", waves[index])
            wavfile.write(args.output / f"modal-control-{index}.wav", 32_000, wave)
        comparison = np.concatenate(
            [segments[0], np.zeros(16_000, np.float32), segments[1]]
        )
        path = args.output / "modal-frequency-control-NOT-NEURAL.wav"
        wavfile.write(path, 32_000, comparison)
        result["audition"] = {
            "claim": "OLD_FITTED_MODAL_BANK / FREQUENCY_CONTROL_ONLY / NOT_PHYSICAL_SIZE_VALIDATION",
            "source_profile": str(args.modal_profile.resolve()),
            "source_sha256": PROFILE_SHA,
            "frequency_ratios": [1.0, 0.5],
            "fixed_damping_and_gains": True,
            "transient_included": False,
            "shared_gain": gain,
            "comparison_seconds": len(comparison) / 32_000,
            "comparison_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        }
    (args.output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
