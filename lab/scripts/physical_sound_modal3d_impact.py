"""Sphere impact with both elastic moduli and velocity, plus feedback control.

Hertz half-space force -> existing neural resonator is a one-way approximation.
A separate positive-residue FEM reference tests when feedback matters. Neither
is microphone pressure, a real material identification or a learned contact law.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_modal3d_pilot as pilot
import physical_sound_sonicgauss_waveform_fit as diagnostics
import torch
from numpy.polynomial.legendre import leggauss
from safetensors.torch import load_file
from scipy.integrate import solve_ivp
from scipy.io import wavfile

SHAPE = pilot.DEV_SHAPES[0]
CONTACT = pilot.DEV_CONTACTS[0]
BASE = {
    "radius": 0.003,
    "density": 2500.0,
    "young": 70e9,
    "poisson": 0.3,
    "target_young": 64e9,
    "velocity": 0.5,
}
CASES = [
    ("base", {}),
    ("striker-E2GPa", {"young": 2e9}),
    ("striker-E200GPa", {"young": 200e9}),
    ("speed-0p1", {"velocity": 0.1}),
    ("speed-1p0", {"velocity": 1.0}),
    ("density-1000", {"density": 1000.0}),
    ("density-7800", {"density": 7800.0}),
    ("radius-1mm", {"radius": 0.001}),
    ("radius-5mm", {"radius": 0.005}),
    ("target-E16GPa", {"target_young": 16e9}),
    ("target-E200GPa", {"target_young": 200e9}),
]


def parameters(radius, density, young, poisson, target_young, velocity):
    if (
        not np.isfinite([radius, density, young, poisson, target_young, velocity]).all()
        or not 0 < radius <= 0.02
        or not 0 < density <= 20000
        or not 0 < young <= 1e12
        or not 0 <= poisson < 0.5
        or not 0 < target_young <= 1e12
        or not 0 < velocity <= 2
    ):
        raise ValueError("bounded positive elastic impact parameters required")
    effective = 1 / ((1 - poisson**2) / young + (1 - SHAPE[2] ** 2) / target_young)
    mass = 4 * np.pi * radius**3 * density / 3
    stiffness = 4 * effective * np.sqrt(radius) / 3
    indentation = (5 * mass * velocity**2 / (4 * stiffness)) ** 0.4
    return {
        "mass": mass,
        "stiffness": stiffness,
        "indentation": indentation,
        "time_scale": indentation / velocity,
        "force_scale": stiffness * indentation**1.5,
        "effective_modulus": effective,
    }


def collision(config, omega=None, self_gains=None):
    """First separation only; no adhesion, plasticity, friction or later rebounds.

    Positive indentation points into the target. Normalize x by delta_max,
    time by delta_max/v, modal common responses by F_max*time_scale^2.
    """
    p = parameters(**config)
    coupled = omega is not None
    if coupled:
        omega, self_gains = np.asarray(omega), np.asarray(self_gains)
        if (
            omega.ndim != 1
            or self_gains.shape != omega.shape
            or not np.isfinite(omega).all()
            or not np.isfinite(self_gains).all()
            or np.any(omega <= 0)
            or np.any(self_gains < 0)
        ):
            raise ValueError(
                "positive collocated mechanical residues required; no absolute-value repair"
            )
    else:
        omega, self_gains = np.zeros(0), np.zeros(0)
    count = len(omega)
    damping = 2 + 1e-8 * omega**2
    wt, dt = omega * p["time_scale"], damping * p["time_scale"]
    coupling = 1.25 * p["mass"] * self_gains

    def indentation(y):
        return y[0] - coupling @ y[2 : 2 + count]

    def rhs(t, y):
        h, hp = y[2 : 2 + count], y[2 + count : 2 + 2 * count]
        force = max(float(indentation(y)), 0.0) ** 1.5
        loss = 6.25 * p["mass"] * np.sum(dt * self_gains * hp**2)
        return np.concatenate(
            ([y[1], -1.25 * force], hp, force - 2 * dt * hp - wt**2 * h, [loss])
        )

    def separated(t, y):
        return indentation(y)

    separated.terminal = True
    separated.direction = -1
    initial = np.zeros(3 + 2 * count)
    initial[1] = 1
    solution = solve_ivp(
        rhs,
        (0, 30),
        initial,
        method="DOP853",
        events=separated,
        dense_output=True,
        rtol=1e-10,
        atol=1e-12,
        max_step=0.03,
    )
    if not solution.success or len(solution.t_events[0]) != 1:
        raise ValueError("contact did not reach first separation")
    end = float(solution.t_events[0][0])
    sample = solution.sol(np.linspace(0, end, 1025))
    indent = np.maximum(indentation(sample), 0)
    h, hp = sample[2 : 2 + count], sample[2 + count : 2 + 2 * count]
    energy = sample[1] ** 2 + indent**2.5 + sample[-1]
    energy += (
        1.5625
        * p["mass"]
        * np.sum(self_gains[:, None] * (hp**2 + (wt[:, None] * h) ** 2), axis=0)
    )
    duration = end * p["time_scale"]

    def force_at(time):
        time = np.asarray(time)
        states = solution.sol(np.clip(time / p["time_scale"], 0, end))
        f = p["force_scale"] * np.maximum(indentation(states), 0) ** 1.5
        return np.where((time >= 0) & (time <= duration), f, 0)

    nodes, weights = leggauss(128)
    impulse = float(duration / 2 * (force_at((nodes + 1) * duration / 2) @ weights))
    velocity_out = float(sample[1, -1] * config["velocity"])
    expected_impulse = p["mass"] * (config["velocity"] - velocity_out)
    return {
        **p,
        "duration": duration,
        "peak_force": float(force_at(np.linspace(0, duration, 2049)).max()),
        "impulse": impulse,
        "velocity_out": velocity_out,
        "energy_balance_error": float(abs(energy - 1).max()),
        "momentum_relative_error": abs(impulse / expected_impulse - 1),
        "max_indentation": float(indent.max() * p["indentation"]),
        "dissipated_energy_fraction": float(sample[-1, -1]),
        "force": force_at,
        "solution": solution,
        "mode_count": count,
        "contact_radius": float(
            np.sqrt(config["radius"] * indent.max() * p["indentation"])
        ),
    }


def render_force(omega, gains, impact, quadrature=64):
    """Continuous force convolution, including sub-audio-sample contact pulses."""
    omega, gains = np.asarray(omega), np.asarray(gains)
    if (
        omega.ndim != 1
        or gains.shape != omega.shape
        or not np.isfinite(omega).all()
        or not np.isfinite(gains).all()
        or np.any(omega <= 0)
    ):
        raise ValueError("finite compatible positive frequencies required")
    damping = 2 + 1e-8 * omega**2
    keep = (omega < 2 * np.pi * pilot.RATE * 0.45) & (omega > damping)
    omega, damping, gains = omega[keep], damping[keep], gains[keep]
    wd = np.sqrt(omega**2 - damping**2)
    lam = -damping + 1j * wd
    nodes, weights = leggauss(quadrature)

    def integral(end):
        time = (nodes + 1) * end / 2
        return (
            end / 2 * ((np.exp(-lam[:, None] * time) * impact["force"](time)) @ weights)
        )

    times = np.arange(2 * pilot.RATE) / pilot.RATE
    amplitude = gains * (1 + 1j * damping / wd)
    end = impact["duration"]
    wave = np.real((amplitude * integral(end)) @ np.exp(lam[:, None] * times))
    for i in np.flatnonzero(times < end):
        wave[i] = np.real(
            np.sum(amplitude * np.exp(lam * times[i]) * integral(times[i]))
        )
    if not np.isfinite(wave).all():
        raise ValueError("nonfinite impact response")
    return wave.astype(np.float32)


def summary(impact):
    return {k: v for k, v in impact.items() if k not in ("force", "solution")}


def weak_contact_passes(weak, coupled, spectral_error):
    differences = [
        abs(weak[k] / coupled[k] - 1) for k in ("duration", "peak_force", "impulse")
    ]
    return bool(
        np.isfinite(differences).all()
        and max(differences) <= 0.05
        and spectral_error <= 0.05
    )


def render(args):
    receipt = json.loads((args.fit / "fit.json").read_text())
    if (
        pilot.integrity.sha256(args.fit / "model.safetensors")
        != receipt["weights_sha256"]
    ):
        raise ValueError("weights changed")
    model = pilot.SharedModes().eval()
    model.load_state_dict(load_file(args.fit / "model.safetensors"), strict=True)
    with torch.no_grad():
        w, g = model(
            torch.tensor(pilot.object_input(SHAPE)[None], dtype=torch.float32),
            torch.tensor(CONTACT[None], dtype=torch.float32),
        )
        _, probe = model(
            torch.tensor(pilot.object_input(SHAPE)[None], dtype=torch.float32),
            torch.tensor([[1.0, 0.5]], dtype=torch.float32),
        )
    w, g = w[0].numpy(), g[0].numpy()
    args.output.mkdir(parents=True)
    np.savez(
        args.output / "neural-modes.npz",
        omega=w,
        gains=g,
        probe_self_prediction=probe[0].numpy(),
    )
    rows, waves = [], {}
    for name, changes in CASES:
        config = {**BASE, **changes}
        impact = collision(config)
        wave = render_force(
            *pilot.physical_modes(w, g, 0.18, config["target_young"], 2230), impact
        )
        waves[name] = wave
        row = {"name": name, "config": config, "contact": summary(impact)}
        rows.append(row)
        time = np.linspace(0, impact["duration"], 1025)
        np.savez(
            args.output / (name + "-force.npz"), time=time, force=impact["force"](time)
        )
    gain = min(1.0, 0.98 / max(float(abs(wave).max()) for wave in waves.values()))
    for row in rows:
        path = args.output / (row["name"] + ".wav")
        wavfile.write(path, pilot.RATE, waves[row["name"]] * gain)
        row["wav_sha256"] = pilot.integrity.sha256(path)
    for title, names in (
        ("striker-stiffness", ("striker-E2GPa", "base", "striker-E200GPa")),
        ("impact-speed", ("speed-0p1", "base", "speed-1p0")),
        ("striker-radius", ("radius-1mm", "base", "radius-5mm")),
        ("target-stiffness", ("target-E16GPa", "base", "target-E200GPa")),
    ):
        wavfile.write(
            args.output / (title + ".wav"),
            pilot.RATE,
            np.concatenate(
                [
                    v
                    for name in names
                    for v in (waves[name] * gain, np.zeros(pilot.RATE // 2))
                ]
            ).astype(np.float32),
        )
    pilot.storage.save(
        args.output / "render.json",
        {
            "rows": rows,
            "gain": gain,
            "fit_sha256": pilot.integrity.sha256(args.fit / "fit.json"),
            "script_sha256": pilot.integrity.sha256(Path(__file__)),
            "probe_negative_residues": int((probe < 0).sum()),
            "scope": "one-way elastic Hertz force + unchanged NN; both moduli, striker radius/density/speed; no target recording/FEM at generation; no contact feedback, plasticity, real losses, radiation or real-material admission",
        },
    )


def assess(args):
    args.output.mkdir(parents=True)
    # A separate reference calculation, never opened by neural rendering.
    ref = pilot.solve(SHAPE, CONTACT[None], refinement=4, contact_response=True)
    np.savez(args.output / "reference-modes.npz", **ref)
    manifest = json.loads((args.generated / "render.json").read_text())
    rows = []
    for row in manifest["rows"]:
        config = row["config"]
        omega, g = pilot.physical_modes(
            ref["omega"], ref["gains"][0], 0.18, config["target_young"], 2230
        )
        self_g = ref["self_gains"][0] / (2230 * 0.18**3)
        weak = collision(config)
        coupled = collision(config, omega, self_g)
        a, b = [render_force(omega, g, impact) for impact in (coupled, weak)]
        path = args.generated / (row["name"] + ".wav")
        if pilot.integrity.sha256(path) != row["wav_sha256"]:
            raise ValueError("neural WAV changed")
        sr, neural = wavfile.read(path)
        if sr != pilot.RATE or manifest["gain"] <= 0:
            raise ValueError("invalid generated WAV")
        neural = neural / manifest["gain"]
        error = diagnostics.audio_metrics(a, b)
        gain = min(1.0, 0.98 / max(float(abs(w).max()) for w in (a, b, neural)))
        wavfile.write(
            args.output / (row["name"] + "-comparison.wav"),
            pilot.RATE,
            np.concatenate(
                [
                    v
                    for w in (a, b, neural)
                    for v in (w * gain, np.zeros(pilot.RATE // 2))
                ]
            ).astype(np.float32),
        )
        time = np.linspace(0, coupled["duration"], 1025)
        np.savez(
            args.output / (row["name"] + "-coupled-force.npz"),
            time=time,
            force=coupled["force"](time),
        )
        result = {
            "name": row["name"],
            "weak": summary(weak),
            "coupled": summary(coupled),
            "weak_vs_coupled": error,
            "neural_vs_coupled": diagnostics.audio_metrics(a, neural),
            "weak_contact_passes": weak_contact_passes(
                weak, coupled, error["spectrum"]
            ),
            "comparison_gain": gain,
        }
        rows.append(result)
        print(
            row["name"],
            "weak_contact_passes",
            result["weak_contact_passes"],
            "force impulse ratio",
            weak["impulse"] / coupled["impulse"],
            flush=True,
        )
    pilot.storage.save(
        args.output / "assessment.json",
        {
            "rows": rows,
            "weak_contact_passes": sum(r["weak_contact_passes"] for r in rows),
            "render_sha256": pilot.integrity.sha256(args.generated / "render.json"),
            "script_sha256": pilot.integrity.sha256(Path(__file__)),
            "solver_script_sha256": pilot.integrity.sha256(Path(pilot.__file__)),
            "scope": "first-separation eight-mode FEM feedback discriminator, not real-audio ground truth; no model tuning or release",
        },
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("render", "assess"))
    for name in ("fit", "generated", "output"):
        parser.add_argument("--" + name, type=Path, required=name == "output")
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    if (
        args.stage == "render"
        and args.fit is None
        or args.stage == "assess"
        and args.generated is None
    ):
        parser.error("missing stage input")
    torch.set_num_threads(4)
    {"render": render, "assess": assess}[args.stage](args)


if __name__ == "__main__":
    main()
