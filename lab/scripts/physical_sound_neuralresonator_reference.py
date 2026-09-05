"""Numerical 2D reference pairs for the published Neural Resonator, report-only.

Own polygons; no imported dataset or retraining. Implements the author's clamped
plane-strain displacement proxy, not 3D acoustic radiation or calibrated impacts.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_neuralresonator_pilot as pilot
import torch
from scipy.io import wavfile
from scipy.sparse.linalg import eigsh
from skfem import Basis, BilinearForm, ElementTriP2, ElementVector, MeshTri, asm
from skfem.helpers import dot
from skfem.models.elasticity import linear_elasticity

POLYGONS = {
    "octagon": [
        [0.1, 0.3],
        [0.3, 0.1],
        [0.7, 0.1],
        [0.9, 0.3],
        [0.9, 0.7],
        [0.7, 0.9],
        [0.3, 0.9],
        [0.1, 0.7],
    ],
    "rectangle": [[0.1, 0.2], [0.9, 0.2], [0.9, 0.8], [0.1, 0.8]],
    "skewed": [[0.1, 0.2], [0.7, 0.1], [0.9, 0.65], [0.4, 0.9]],
}
MATERIALS = {"base": pilot.BASE, "combined": [2100.0, 1.7e10, 0.31, 6.0, 1e-6]}


def mask_for(polygon: np.ndarray) -> torch.Tensor:
    y, x = np.mgrid[:64, :64] / 64
    mask = np.ones((64, 64), bool)
    for a, b in zip(polygon, np.roll(polygon, -1, axis=0)):
        mask &= (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0]) >= -1e-12
    return torch.tensor(mask, dtype=torch.float32)


def solve_modes(
    polygon: np.ndarray, material, refinement: int, coordinate_scale: float = 2.0
) -> dict:
    center = polygon.mean(0)
    vertices = np.vstack([center, polygon])
    triangles = np.array(
        [[0, i + 1, (i + 1) % len(polygon) + 1] for i in range(len(polygon))]
    )
    mesh = MeshTri((coordinate_scale * (vertices - 0.5)).T, triangles.T).refined(
        refinement
    )
    basis = Basis(mesh, ElementVector(ElementTriP2()))
    rho, young, poisson, alpha, beta = material
    mu = young / (2 * (1 + poisson))
    lam = young * poisson / ((1 + poisson) * (1 - 2 * poisson))
    stiffness = asm(linear_elasticity(lam, mu), basis)
    mass = asm(BilinearForm(lambda u, v, w: rho * dot(u, v)), basis)
    free = basis.complement_dofs(basis.get_dofs())
    k, m = stiffness[free][:, free], mass[free][:, free]
    values, vectors = eigsh(
        k, k=32, M=m, sigma=0.0, which="LM", v0=np.linspace(1, 2, len(free))
    )
    order = np.argsort(values)
    values, vectors = values[order], vectors[:, order]
    kv, mv = k @ vectors, m @ vectors
    residual = np.linalg.norm(kv - mv * values, axis=0) / (
        np.linalg.norm(kv, axis=0) + np.linalg.norm(mv * values, axis=0)
    )
    full = np.zeros((basis.N, 32))
    full[free] = vectors
    mode_gain = np.linalg.norm(full[basis.nodal_dofs], axis=0)
    damping = 0.5 * (alpha + beta * values)
    frequencies = np.sqrt(np.maximum(values - damping**2, 0)) / (2 * np.pi)
    if (
        not np.isfinite(frequencies).all()
        or np.any(frequencies <= 0)
        or np.any(frequencies >= pilot.RATE / 2)
    ):
        raise ValueError("reference modes outside audible renderer bounds")
    return {
        "frequencies": frequencies,
        "eigenvalues": values,
        "damping": damping,
        "gains": mode_gain,
        "points": mesh.p.T / coordinate_scale + 0.5,
        "max_relative_residual": float(residual.max()),
        "dofs": int(basis.N),
    }


def render_reference(modes: dict, contact: np.ndarray) -> np.ndarray:
    distance = np.linalg.norm(modes["points"] - contact, axis=1)
    index = int(distance.argmin())
    if distance[index] > 1e-10:
        raise ValueError("contact must be an exact generated mesh vertex")
    t = np.arange(pilot.RATE) / pilot.RATE
    return np.sum(
        modes["gains"][index, :, None]
        * np.exp(-modes["damping"][:, None] * t)
        * np.cos(2 * np.pi * modes["frequencies"][:, None] * t),
        axis=0,
    )


def spectrum_distance(reference: np.ndarray, neural: np.ndarray) -> float:
    # Fixed full-second shape descriptor; not an acceptance threshold or judge.
    spectra = [abs(np.fft.rfft(w))[20:] for w in (reference, neural)]
    logs = [20 * np.log10(s / np.linalg.norm(s) + 1e-6) for s in spectra]
    return float(np.sqrt(np.mean((logs[0] - logs[1]) ** 2)))


def run(assets: Path, output: Path) -> dict:
    if output.exists():
        raise ValueError("preserve previous outputs; use a new directory")
    output.mkdir(parents=True)
    torch.set_num_threads(4)
    torch.manual_seed(42)
    model, encoder, _, _ = pilot.load_models(assets)
    rows, waveform_pairs, convergence = [], [], []
    for shape, vertices in POLYGONS.items():
        polygon = np.asarray(vertices)
        center = polygon.mean(0)
        contacts = {"center": center, "off-center": (center + polygon[0]) / 2}
        mask = mask_for(polygon)
        np.save(output / f"{shape}-mask.npy", mask.numpy())
        with torch.inference_mode():
            features = encoder(mask[None, None].repeat(1, 3, 1, 1))
        for material_name, material in MATERIALS.items():
            coarse = solve_modes(polygon, material, 3)
            fine = solve_modes(polygon, material, 4)
            convergence.append(
                {
                    "shape": shape,
                    "material": material_name,
                    "max_32_eigenfrequency_relative_change": float(
                        abs(
                            np.sqrt(coarse["eigenvalues"] / fine["eigenvalues"]) - 1
                        ).max()
                    ),
                    "max_relative_eigen_residual": fine["max_relative_residual"],
                    "fine_dofs": fine["dofs"],
                }
            )
            np.savez(output / f"{shape}-{material_name}-modes.npz", **fine)
            for position, contact in contacts.items():
                name = f"{shape}-{material_name}-{position}"
                with torch.inference_mode():
                    inputs = torch.cat(
                        [
                            features,
                            torch.tensor(contact[None], dtype=torch.float32),
                            pilot.material_input(material)[None],
                        ],
                        -1,
                    )
                    coefficients = model(inputs)[0].numpy()
                neural, radius = pilot.render_coefficients(coefficients)
                reference = render_reference(fine, contact)
                np.save(output / f"{name}-coefficients.npy", coefficients)
                np.save(output / f"{name}-neural-raw.npy", neural)
                np.save(output / f"{name}-reference-raw.npy", reference)
                r, n = (
                    pilot.waveform_observations(reference),
                    pilot.waveform_observations(neural),
                )
                row = {
                    "name": name,
                    "material": material,
                    "contact": contact.tolist(),
                    "reference": r,
                    "neural": n,
                    "max_pole_radius": radius,
                    "dominant_peak_relative_error": n["dominant_hz"] / r["dominant_hz"]
                    - 1,
                    "full_spectrum_shape_rmse_db": spectrum_distance(reference, neural),
                }
                rows.append(row)
                waveform_pairs.append((reference, neural))
                print(json.dumps(row), flush=True)
    # Preserve the coordinate ambiguity exposed by the author's two example paths.
    polygon = np.asarray(POLYGONS["octagon"])
    unit = solve_modes(polygon, pilot.BASE, 4, coordinate_scale=1.0)
    double = solve_modes(polygon, pilot.BASE, 4, coordinate_scale=2.0)
    control = {
        "scale1_vs_scale2_undamped_mode_ratio": np.sqrt(
            unit["eigenvalues"] / double["eigenvalues"]
        ).tolist(),
        "scale1_reference_peak_hz": pilot.waveform_observations(
            render_reference(unit, polygon.mean(0))
        )["dominant_hz"],
        "scale2_reference_peak_hz": pilot.waveform_observations(
            render_reference(double, polygon.mean(0))
        )["dominant_hz"],
        "neural_input_unchanged": True,
    }
    # Single shared gain preserves level discrepancies as well as spectral ones.
    gain = 0.5 / max(float(abs(w).max()) for pair in waveform_pairs for w in pair)
    for row, (reference, neural) in zip(rows, waveform_pairs):
        parts = [(wave * gain).astype(np.float32) for wave in (reference, neural)]
        for kind, pcm in zip(("reference", "neural"), parts):
            wavfile.write(output / f"{row['name']}-{kind}.wav", pilot.RATE, pcm)
        comparison = np.concatenate(
            [parts[0], np.zeros(pilot.RATE // 2, np.float32), parts[1]]
        )
        path = output / f"{row['name']}-comparison.wav"
        wavfile.write(path, pilot.RATE, comparison)
        row["comparison_sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
    result = {
        "claim": "OWN_POLYGON_NUMERICAL_REFERENCE / NOT_REAL_OBJECT_OR_HOLDOUT_ADMISSION",
        "coordinate_frame": "world=2*(normalized-.5), matching published results notebook; generator default differs",
        "coordinate_control": control,
        "convergence": convergence,
        "records": rows,
        "shared_gain": gain,
        "checkpoint_sha256": pilot.CHECKPOINT_SHA,
        "sample_rate": pilot.RATE,
        "samples": pilot.RATE,
        "trained_here": False,
    }
    (output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(
        json.dumps(
            {"convergence": convergence, "coordinate_control": control}, indent=2
        )
    )
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run(args.assets, args.output)
