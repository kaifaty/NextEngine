"""Shared 3D elastic-mode surrogate with separate contact participation.

Own clamped rectangular solids, quadratic tetrahedral FEM teachers. Learns
dimensionless geometry/Poisson response; material/size scaling and damped modal
rendering are analytical. Output is a fixed-point vibration-velocity proxy,
NOT calibrated microphone pressure, a two-body contact solver or real material
identification. No per-object fitting, target audio at inference or runtime use.
"""

from __future__ import annotations

import argparse
import itertools
import json
from pathlib import Path

import numpy as np
import physical_sound_sonicgauss_cohort as storage
import physical_sound_sonicgauss_pilot as integrity
import torch
from safetensors.torch import load_file, save_file
from scipy.io import wavfile
from scipy.signal import fftconvolve
from torch import nn

MODES = 8
RATE = 44100
TRAIN_SHAPES = list(
    itertools.product(
        (0.12, 0.16, 0.20, 0.24), (0.045, 0.065, 0.085), (0.20, 0.28, 0.36)
    )
)
DEV_SHAPES = list(itertools.product((0.14, 0.18, 0.22), (0.055, 0.075), (0.24, 0.32)))
TRAIN_CONTACTS = np.array(list(itertools.product((0.4, 0.7, 1.0), (0.2, 0.5, 0.8))))
DEV_CONTACTS = np.array(list(itertools.product((0.55, 0.85), (0.35, 0.65))))


def solve(
    shape,
    contacts,
    refinement=1,
    length=1.0,
    young=1.0,
    density=1.0,
    *,
    mode_count=MODES,
    sample_points=None,
):
    # Imports stay out of standalone neural rendering.
    from scipy.sparse.linalg import eigsh
    from skfem import Basis, BilinearForm, ElementTetP2, ElementVector, MeshTet, asm
    from skfem.helpers import dot
    from skfem.models.elasticity import linear_elasticity

    ry, rz, poisson = shape
    if not (0 < ry < 1 and 0 < rz < 1 and 0 < poisson < 0.49):
        raise ValueError("invalid isotropic shape")
    mesh = MeshTet.init_tensor(
        np.linspace(0, length, 6 * refinement + 1),
        np.linspace(0, length * ry, 2 * refinement + 1),
        np.linspace(0, length * rz, 2 * refinement + 1),
    )
    basis = Basis(mesh, ElementVector(ElementTetP2()))
    mu = young / (2 * (1 + poisson))
    lam = young * poisson / ((1 + poisson) * (1 - 2 * poisson))
    k = asm(linear_elasticity(lam, mu), basis)
    m = asm(BilinearForm(lambda u, v, w: density * dot(u, v)), basis)
    free = basis.complement_dofs(basis.get_dofs(lambda x: np.isclose(x[0], 0)))
    k, m = k[free][:, free], m[free][:, free]
    if not isinstance(mode_count, int) or not 1 <= mode_count <= 32:
        raise ValueError("bounded mode count required")
    values, vectors = eigsh(
        k,
        k=mode_count,
        M=m,
        sigma=0,
        which="LM",
        v0=np.linspace(1, 2, len(free)),
        tol=1e-10,
    )
    order = np.argsort(values)
    values, vectors = values[order], vectors[:, order]
    residual = np.linalg.norm(k @ vectors - (m @ vectors) * values, axis=0) / (
        np.linalg.norm(k @ vectors, axis=0)
        + np.linalg.norm((m @ vectors) * values, axis=0)
    )
    if not np.isfinite(values).all() or np.any(values <= 0) or residual.max() > 1e-7:
        raise ValueError("invalid elastic eigenpairs")
    full = np.zeros((basis.N, mode_count))
    full[free] = vectors
    # Top-surface +z force, fixed +z velocity probe at (1,.5,1).
    points = np.vstack([np.asarray(contacts), [1.0, 0.5]])
    xyz = (
        np.vstack([points[:, 0], points[:, 1] * ry, np.full(len(points), rz)]) * length
    )
    sampled = (basis.probes(xyz) @ full).reshape(3, len(points), mode_count)[2]
    gains = sampled[:-1] * sampled[-1]
    result = {
        "omega": np.sqrt(values),
        "gains": gains,
        "max_residual": float(residual.max()),
        "dofs": basis.N,
    }
    if sample_points is not None:
        samples = np.asarray(sample_points)
        if (
            samples.ndim != 2
            or samples.shape[1] != 3
            or not np.isfinite(samples).all()
            or np.any((samples < 0) | (samples > 1))
        ):
            raise ValueError("normalized interior modal samples required")
        result["mode_samples"] = (
            basis.probes((samples * [length, length * ry, length * rz]).T) @ full
        )
    return result


def make_data(args):
    args.output.mkdir(parents=True)
    rows = []
    for role, shapes, contacts in (
        ("train", TRAIN_SHAPES, TRAIN_CONTACTS),
        ("development", DEV_SHAPES, DEV_CONTACTS),
    ):
        for index, shape in enumerate(shapes):
            modes = solve(shape, contacts)
            name = f"{role}-{index:02d}.npz"
            np.savez(args.output / name, shape=shape, contacts=contacts, **modes)
            rows.append(
                {
                    "role": role,
                    "index": index,
                    "shape": shape,
                    "file": name,
                    "sha256": integrity.sha256(args.output / name),
                    "max_residual": modes["max_residual"],
                }
            )
            print("3D teacher", role, index, flush=True)
    coarse = solve(DEV_SHAPES[0], DEV_CONTACTS)
    fine = solve(DEV_SHAPES[0], DEV_CONTACTS, refinement=2)
    np.savez(args.output / "refined-control.npz", **fine)
    storage.save(
        args.output / "data.json",
        {
            "rows": rows,
            "solver": "scikit-fem12.0.2 vector quadratic tetrahedra, clamped x=0; E=rho=L=1",
            "refined_frequency_relative_change": (
                coarse["omega"] / fine["omega"] - 1
            ).tolist(),
            "script_sha256": integrity.sha256(Path(__file__)),
            "scope": "new own synthetic family, not a real recording corpus or pretrained dataset",
        },
    )


def object_input(shape):
    return (np.asarray(shape, dtype=np.float32) - [0.18, 0.065, 0.28]) / [
        0.06,
        0.02,
        0.08,
    ]


class SharedModes(nn.Module):
    def __init__(self):
        super().__init__()
        self.body = nn.Sequential(
            nn.Linear(3, 64),
            nn.SiLU(),
            nn.Linear(64, 64),
            nn.SiLU(),
            nn.Linear(64, MODES),
        )
        self.contact = nn.Sequential(
            nn.Linear(5, 64),
            nn.SiLU(),
            nn.Linear(64, 64),
            nn.SiLU(),
            nn.Linear(64, MODES),
        )
        self.register_buffer("log_mean", torch.zeros(MODES))
        self.register_buffer("log_std", torch.ones(MODES))
        self.register_buffer("gain_scale", torch.ones(MODES))

    def forward(self, objects, contacts):
        omega = (self.body(objects) * self.log_std + self.log_mean).exp()
        gains = (
            self.contact(torch.cat([objects, contacts * 2 - 1], -1)).sinh()
            * self.gain_scale
        )
        return omega, gains


def fit(args):
    receipt = json.loads((args.data / "data.json").read_text())
    x, c, frequencies, gains = [], [], [], []
    for row in receipt["rows"]:
        if row["role"] != "train":
            continue
        path = args.data / row["file"]
        if integrity.sha256(path) != row["sha256"]:
            raise ValueError("teacher hash mismatch")
        with np.load(path, allow_pickle=False) as d:
            for contact, g in zip(d["contacts"], d["gains"], strict=True):
                x.append(object_input(d["shape"]))
                c.append(contact)
                frequencies.append(d["omega"])
                gains.append(g)
    x, c, frequencies, gains = [
        torch.tensor(np.array(v), dtype=torch.float32)
        for v in (x, c, frequencies, gains)
    ]
    torch.manual_seed(42)
    model = SharedModes()
    model.log_mean.copy_(frequencies.log().mean(0))
    model.log_std.copy_(frequencies.log().std(0).clamp_min(0.01))
    model.gain_scale.copy_(gains.square().mean(0).sqrt().clamp_min(1e-6))
    targets = (
        (frequencies.log() - model.log_mean) / model.log_std,
        (gains / model.gain_scale).asinh(),
    )
    optimizer = torch.optim.Adam(model.parameters(), lr=1e-3)
    args.output.mkdir(parents=True)
    losses = []
    for step in range(1500):
        optimizer.zero_grad(set_to_none=True)
        predicted = (model.body(x), model.contact(torch.cat([x, c * 2 - 1], -1)))
        loss = sum(
            (a - b).square().mean() for a, b in zip(predicted, targets, strict=True)
        )
        if not torch.isfinite(loss):
            raise ValueError("nonfinite fit")
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1, error_if_nonfinite=True)
        optimizer.step()
        if step == 0 or (step + 1) % 250 == 0:
            losses.append({"step": step + 1, "loss": loss.item()})
            print("shared 3D fit", losses[-1], flush=True)
    save_file(model.state_dict(), args.output / "model.safetensors")
    storage.save(
        args.output / "fit.json",
        {
            "steps": 1500,
            "seed": 42,
            "train_shapes": len(TRAIN_SHAPES),
            "train_contacts": len(x),
            "parameters": sum(p.numel() for p in model.parameters()),
            "data_sha256": integrity.sha256(args.data / "data.json"),
            "weights_sha256": integrity.sha256(args.output / "model.safetensors"),
            "losses": losses,
            "script_sha256": integrity.sha256(Path(__file__)),
            "scope": "one final checkpoint, no DEV reads/selection; geometry modes and participation learned, physical scaling analytical",
        },
    )


def physical_modes(omega, gains, length, young, density):
    if (
        min(length, young, density) <= 0
        or not np.isfinite([length, young, density]).all()
    ):
        raise ValueError("positive finite physical parameters required")
    return omega * np.sqrt(young / density) / length, gains / (density * length**3)


def render_wave(omega, gains, impulse=0.002, duration=0.0005):
    if (
        not (0 < duration <= 0.02 and 0 <= impulse <= 0.1)
        or not np.isfinite(omega).all()
        or not np.isfinite(gains).all()
        or np.any(omega <= 0)
    ):
        raise ValueError("invalid modes/excitation")
    # Declared Rayleigh damping, not inferred material losses.
    damping = 2.0 + 1e-8 * omega**2
    retained = (omega < 2 * np.pi * RATE * 0.45) & (omega > damping)
    w, d, g = omega[retained], damping[retained], gains[retained]
    wd = np.sqrt(w**2 - d**2)
    t = np.arange(2 * RATE) / RATE
    velocity = np.sum(
        g[:, None]
        * np.exp(-d[:, None] * t)
        * (np.cos(wd[:, None] * t) - (d / wd)[:, None] * np.sin(wd[:, None] * t)),
        axis=0,
    )
    count = max(2, round(duration * RATE))
    pulse = np.sin(np.pi * (np.arange(count) + 0.5) / count)
    pulse *= impulse / pulse.sum()
    return fftconvolve(velocity, pulse)[: len(t)].astype(np.float32)


def render(args):
    receipt = json.loads((args.fit / "fit.json").read_text())
    if integrity.sha256(args.fit / "model.safetensors") != receipt["weights_sha256"]:
        raise ValueError("weights changed")
    model = SharedModes().eval()
    model.load_state_dict(load_file(args.fit / "model.safetensors"), strict=True)
    args.output.mkdir(parents=True)
    rows, waves = [], []
    with torch.no_grad():
        for index, shape in enumerate(DEV_SHAPES):
            for ci, contact in enumerate(DEV_CONTACTS):
                omega, gains = model(
                    torch.tensor(object_input(shape)[None], dtype=torch.float32),
                    torch.tensor(contact[None], dtype=torch.float32),
                )
                omega, gains = omega[0].numpy(), gains[0].numpy()
                physical = physical_modes(omega, gains, 0.18, 64e9, 2230)
                name = f"case-{index:02d}-contact-{ci}"
                np.savez(args.output / (name + ".npz"), omega=omega, gains=gains)
                waves.append(render_wave(*physical))
                rows.append({"shape_index": index, "contact_index": ci, "name": name})
    # One held shape/contact: numeric material/size/impulse/pulse-duration controls.
    first = args.output / "case-00-contact-0.npz"
    with np.load(first, allow_pickle=False) as values:
        variants = [
            ("base", 0.18, 64e9, 2230, 0.002, 0.0005),
            ("double-size", 0.36, 64e9, 2230, 0.002, 0.0005),
            ("quarter-stiffness", 0.18, 16e9, 2230, 0.002, 0.0005),
            ("double-impulse", 0.18, 64e9, 2230, 0.004, 0.0005),
            ("soft-long-pulse", 0.18, 64e9, 2230, 0.002, 0.005),
        ]
        for label, size, young, density, impulse, duration in variants:
            waves.append(
                render_wave(
                    *physical_modes(
                        values["omega"], values["gains"], size, young, density
                    ),
                    impulse,
                    duration,
                )
            )
            rows.append(
                {
                    "name": label,
                    "length_m": size,
                    "young_pa": young,
                    "density": density,
                    "impulse_ns": impulse,
                    "pulse_seconds": duration,
                }
            )
    gain = min(1.0, 0.98 / max(float(abs(w).max()) for w in waves))
    for row, wave in zip(rows, waves, strict=True):
        path = args.output / (row["name"] + ".wav")
        wavfile.write(path, RATE, wave * gain)
        row.update(wav=str(path), sha256=integrity.sha256(path))
    wavfile.write(
        args.output / "physical-controls.wav",
        RATE,
        np.concatenate(
            [v for w in waves[-5:] for v in (w * gain, np.zeros(RATE // 2))]
        ).astype(np.float32),
    )
    storage.save(
        args.output / "render.json",
        {
            "rows": rows,
            "gain": gain,
            "fit_sha256": integrity.sha256(args.fit / "fit.json"),
            "script_sha256": integrity.sha256(Path(__file__)),
            "scope": "no FEM/teacher files/audio read; 12 new shape/Poisson combinations x4 new contacts; same cuboid topology; velocity proxy, not pressure; size/E/rho laws analytical, pulse softness not identified striker material",
        },
    )


def interpolation_baseline(train):
    from scipy.interpolate import RegularGridInterpolator

    axes = [sorted({shape[i] for shape in TRAIN_SHAPES}) for i in range(3)]
    contact_axes = [sorted(set(TRAIN_CONTACTS[:, i])) for i in range(2)]
    omega = np.empty((4, 3, 3, MODES))
    gains = np.empty((4, 3, 3, 3, 3, MODES))
    if len(train) != len(TRAIN_SHAPES) or {tuple(row["shape"]) for row in train} != set(
        TRAIN_SHAPES
    ):
        raise ValueError("complete training grid required")
    for row in train:
        index = tuple(axes[j].index(float(row["shape"][j])) for j in range(3))
        omega[index] = row["omega"]
        gains[index] = row["gains"].reshape(3, 3, MODES)
    return RegularGridInterpolator(axes, omega), RegularGridInterpolator(
        axes + contact_axes, gains
    )


def beats_baseline(summary, baseline):
    return all(
        summary[m]["neural"] < summary[m][baseline]
        for m in (
            "frequency_relative_mean",
            "participation_relative_l1",
            "spectrum",
            "envelope",
            "level",
        )
    )


def evaluate(args):
    import physical_sound_sonicgauss_waveform_fit as diagnostics

    data = json.loads((args.data / "data.json").read_text())
    rendered = json.loads((args.generated / "render.json").read_text())
    train = []
    dev = {}
    for row in data["rows"]:
        path = args.data / row["file"]
        if integrity.sha256(path) != row["sha256"]:
            raise ValueError("teacher hash mismatch")
        with np.load(path, allow_pickle=False) as values:
            d = {k: values[k].copy() for k in values.files}
        if row["role"] == "train":
            train.append(d)
        else:
            dev[row["index"]] = d
    interpolate_omega, interpolate_gains = interpolation_baseline(train)
    records = [r for r in rendered["rows"] if "shape_index" in r]
    if len(records) != 48 or {
        (r["shape_index"], r["contact_index"]) for r in records
    } != set(itertools.product(range(12), range(4))):
        raise ValueError("incomplete held combination render")
    results, comparisons = [], []
    for row in records:
        index, ci = row["shape_index"], row["contact_index"]
        target = dev[index]
        nearest = min(
            train,
            key=lambda d: np.linalg.norm(
                object_input(d["shape"]) - object_input(target["shape"])
            ),
        )
        nc = np.linalg.norm(
            nearest["contacts"] - target["contacts"][ci], axis=1
        ).argmin()
        with np.load(args.generated / (row["name"] + ".npz"), allow_pickle=False) as n:
            predictions = {
                "nearest": (nearest["omega"], nearest["gains"][nc]),
                "interpolation": (
                    interpolate_omega(target["shape"][None])[0],
                    interpolate_gains(
                        np.concatenate([target["shape"], target["contacts"][ci]])[None]
                    )[0],
                ),
                "neural": (n["omega"].copy(), n["gains"].copy()),
            }
        reference = render_wave(
            *physical_modes(target["omega"], target["gains"][ci], 0.18, 64e9, 2230)
        )
        result = {"shape_index": index, "contact_index": ci}
        parts = [reference]
        for name, (omega, gains) in predictions.items():
            wave = render_wave(*physical_modes(omega, gains, 0.18, 64e9, 2230))
            if name == "neural":
                if integrity.sha256(Path(row["wav"])) != row["sha256"]:
                    raise ValueError("neural WAV hash mismatch")
                sr, saved = wavfile.read(row["wav"])
                if sr != RATE or not np.array_equal(saved, wave * rendered["gain"]):
                    raise ValueError("standalone waveform replay differs")
            result[name] = {
                "frequency_relative_mean": float(
                    np.mean(abs(omega / target["omega"] - 1))
                ),
                "frequency_relative_max": float(
                    np.max(abs(omega / target["omega"] - 1))
                ),
                "participation_relative_l1": float(
                    abs(gains - target["gains"][ci]).sum()
                    / max(abs(target["gains"][ci]).sum(), 1e-10)
                ),
                **diagnostics.audio_metrics(reference, wave),
            }
            if name != "nearest":
                parts.append(wave)
        results.append(result)
        comparisons.append((row["name"], parts))
    summary = {
        metric: {
            name: float(np.mean([r[name][metric] for r in results]))
            for name in ("nearest", "interpolation", "neural")
        }
        for metric in results[0]["neural"]
    }
    for metric, values in summary.items():
        values["wins"] = sum(
            r["neural"][metric] < r["nearest"][metric] for r in results
        )
        values["wins_over_interpolation"] = sum(
            r["neural"][metric] < r["interpolation"][metric] for r in results
        )
    passed = beats_baseline(summary, "nearest") and beats_baseline(
        summary, "interpolation"
    )
    gain = min(
        1.0, 0.98 / max(float(abs(w).max()) for _, parts in comparisons for w in parts)
    )
    args.output.mkdir(parents=True)
    for name, parts in comparisons:
        wavfile.write(
            args.output / (name + "-comparison.wav"),
            RATE,
            np.concatenate(
                [v for w in parts for v in (w * gain, np.zeros(RATE // 2))]
            ).astype(np.float32),
        )
    output = {
        "rows": results,
        "summary": summary,
        "comparison_gain": gain,
        "decision": "REPORT_ONLY_DISCRETE_TEACHER_BASELINE_IMPROVEMENT"
        if passed
        else "REPORT_ONLY_PROTOTYPE_BASELINE_ADVANTAGE_NOT_ESTABLISHED",
        "scope": "reference -> interpolated TRAIN examples -> shared neural; metrics also retain nearest baseline; coarse discrete FEM only, not real sound; no rollout or physical admission",
        "render_sha256": integrity.sha256(args.generated / "render.json"),
        "data_sha256": integrity.sha256(args.data / "data.json"),
        "standalone_replay_exact": True,
    }
    storage.save(args.output / "evaluation.json", output)
    print(
        json.dumps({"summary": summary, "decision": output["decision"]}, indent=2),
        flush=True,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("data", "fit", "render", "evaluate"))
    for name in ("data", "fit", "generated", "output"):
        parser.add_argument("--" + name, type=Path, required=name == "output")
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    if (
        args.stage == "fit"
        and args.data is None
        or args.stage == "render"
        and args.fit is None
    ):
        parser.error("missing stage input")
    if args.stage == "evaluate" and (args.data is None or args.generated is None):
        parser.error("evaluate needs --data and --generated")
    torch.set_num_threads(4)
    {"data": make_data, "fit": fit, "render": render, "evaluate": evaluate}[args.stage](
        args
    )


if __name__ == "__main__":
    main()
