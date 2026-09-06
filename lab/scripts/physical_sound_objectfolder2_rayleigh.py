"""Published OF2 Rayleigh manifold: frozen neural frequencies/gains, analytic decay.

Gao et al., CVPR2022 main Eq6-7 and supplementary Table1. No fitted material
coefficients or target poles at rendering. This reproduces an assigned synthetic
material law, NOT measured damping for arbitrary real objects or striker pairs.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_channels as channels
import physical_sound_objectfolder2_shared as shared
import torch

SOURCE = "https://ai.stanford.edu/~rhgao/objectfolder2.0/ObjectFolderV2_Supp.pdf"
COEFFICIENTS = {
    "Ceramic": (6.0, 1e-7),
    "Glass": (1.0, 1e-7),
    "Wood": (60.0, 2e-6),
    "Plastic": (30.0, 1e-6),
    "Iron": (5.0, 1e-7),
    "Polycarbonate": (0.5, 4e-7),
    "Steel": (5.0, 3e-8),
}


def damping(frequency, material):
    f = np.asarray(frequency, dtype=np.float64)
    if (
        material not in COEFFICIENTS
        or f.ndim != 1
        or not np.isfinite(f).all()
        or np.any((f <= 0) | (f >= teacher.RATE / 2))
    ):
        raise ValueError(
            "known assigned material and finite sub-Nyquist frequencies required"
        )
    alpha, beta = COEFFICIENTS[material]
    omega2 = (2 * np.pi * f) ** 2
    discriminant = 1 - alpha * beta - beta**2 * omega2
    if np.any(discriminant <= 0):
        raise ValueError("outside low-branch Rayleigh frequency domain")
    # d=(alpha+beta*lambda)/2, lambda=(2*pi*f_damped)^2+d^2.
    # Rationalized smaller root avoids subtracting nearly equal numbers.
    return (alpha + beta * omega2) / (1 + np.sqrt(discriminant))


def predict(model, cloud, features, contacts):
    onehot = features[3:]
    if (
        onehot.shape != (len(shared.MATERIALS),)
        or not np.isin(onehot, [0, 1]).all()
        or onehot.sum() != 1
    ):
        raise ValueError("one known material required at generation")
    material = shared.MATERIALS[int(onehot.argmax())]
    result = channels.predict(model, cloud, features, contacts)
    result["learned_damping"] = result["damping"].copy()
    result["damping"] = damping(result["frequency"], material)
    result["rayleigh_alpha_beta"] = np.array(COEFFICIENTS[material])
    return result


def verify(args):
    rows = json.loads((args.data / "data.json").read_text())["rows"]
    if len(rows) != 9 or {r["object_id"] for r in rows} != shared.TRAIN | shared.DEV:
        raise ValueError("fixed nine-object source audit required")
    results = []
    for row in rows:
        path = args.data / row["file"]
        if teacher.sha(path) != row["sha256"]:
            raise ValueError("full teacher changed")
        with np.load(path, allow_pickle=False) as d:
            calculated = damping(d["frequency"], row["material"])
            relative = float(np.max(abs(calculated / d["damping"] - 1)))
            # Validate BOTH oscillator equations, not an undamped-frequency shortcut.
            alpha, beta = COEFFICIENTS[row["material"]]
            shortcut = (alpha + beta * (2 * np.pi * d["frequency"]) ** 2) / 2
            shortcut_error = float(np.max(abs(shortcut / d["damping"] - 1)))
        if relative > 1e-12:
            raise ValueError("published material law does not reproduce source")
        results.append(
            {
                "object_id": row["object_id"],
                "material": row["material"],
                "modes": row["modes"],
                "max_relative_error": relative,
                "undamped_shortcut_max_relative_error": shortcut_error,
            }
        )
    args.output.mkdir(parents=True)
    shared.write_json(
        args.output / "verification.json",
        {
            "rows": results,
            "source": SOURCE,
            "coefficients": COEFFICIENTS,
            "tolerance": 1e-12,
            "scope": "source-model algebra verification, no coefficient fitting, not new holdout/perceptual evidence",
        },
    )
    print(json.dumps(results, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("verify", "render"))
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--fit", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    torch.set_num_threads(4)
    if args.stage == "verify":
        verify(args)
    else:
        shared.render(args, model_class=channels.ChannelStudent, prediction_fn=predict)
        shared.write_json(
            args.output / "analytic-decay.json",
            {
                "source": SOURCE,
                "coefficients": COEFFICIENTS,
                "scope": "posthoc frozen network discriminator; only damping replaced, frequencies/gains/occupancy untouched; no target acoustics at generation",
            },
        )
