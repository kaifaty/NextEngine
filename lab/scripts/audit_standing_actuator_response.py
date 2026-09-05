"""Summarize recorded native actuator motion, not a stability/admission gate.

Spectra use the final eight seconds (or the full shorter trace), remove DC,
and use a rectangular window. Frequency power fractions are descriptive only.
The descriptor supplies names/ordinals and geometry tokens, not authorization
to load a trace's policy or a claim that different body hashes are compatible.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np


def spectrum(values, hz=240):
    values = np.asarray(values, dtype=float)
    if values.ndim != 1 or len(values) < 2 or not np.isfinite(values).all():
        raise ValueError("expected a finite one-dimensional time series")
    power = np.abs(np.fft.rfft(values - values.mean())) ** 2
    frequencies = np.fft.rfftfreq(len(values), 1 / hz)
    total = power.sum()
    return {
        "rms": float(np.sqrt(np.mean(values**2))),
        "peak_hz": float(frequencies[1 + np.argmax(power[1:])]) if total else None,
        "power_fraction_20_to_30_hz": float(
            power[(frequencies >= 20) & (frequencies <= 30)].sum() / total
        )
        if total
        else 0.0,
        "power_fraction_above_30_hz": float(power[frequencies > 30].sum() / total)
        if total
        else 0.0,
    }


def summarize(descriptor, trace):
    if descriptor["physics_hz"] != 240 or trace["schema_version"] not in (8, 9):
        raise ValueError("unsupported trace timing/schema")
    channels = descriptor["actuators"]
    if [a["actuator_id"] for a in channels] != trace["ordered_actuator_ids"]:
        raise ValueError("actuator order mismatch")
    ordinals = [a["dof_ordinal"] for a in channels]
    if sorted(ordinals) != list(range(len(channels))):
        raise ValueError("expected complete unique dof ordinals")
    if "actuators" in trace and [
        (a["actuator_id"], a["dof_ordinal"]) for a in trace["actuators"]
    ] != [(a["actuator_id"], a["dof_ordinal"]) for a in channels]:
        raise ValueError("trace dof mapping mismatch")
    steps = trace["substep_samples"]
    if len(steps) < 2 or trace["physics_substeps"] != len(steps):
        raise ValueError("incomplete substep series")
    for index, step in enumerate(steps, 1):
        if step["physics_substep"] != index or [
            j["ordinal"] for j in step["joints"]
        ] != list(range(len(channels))):
            raise ValueError("noncontiguous substep/joint series")
    positions = (
        np.asarray(
            [[j["position_urad"] for j in step["joints"]] for step in steps],
            dtype=float,
        )
        / 1e6
    )
    velocities = (
        np.asarray(
            [[j["velocity_urad_s"] for j in step["joints"]] for step in steps],
            dtype=float,
        )
        / 1e6
    )
    efforts = (
        np.asarray([step["applied_efforts_by_dof_unm"] for step in steps], dtype=float)
        / 1e6
    )
    if efforts.shape != positions.shape or not all(
        np.isfinite(a).all() for a in (positions, velocities, efforts)
    ):
        raise ValueError("invalid joint/effort arrays")
    rows = []
    for channel in channels:
        ordinal = channel["dof_ordinal"]
        rows.append(
            {
                "actuator_id": channel["actuator_id"],
                "dof_ordinal": ordinal,
                "position_range_rad": [
                    float(positions[:, ordinal].min()),
                    float(positions[:, ordinal].max()),
                ],
                "maximum_abs_velocity_rad_s": float(
                    np.abs(velocities[:, ordinal]).max()
                ),
                "maximum_abs_effort_nm": float(np.abs(efforts[:, ordinal]).max()),
                "tail_velocity_spectrum": spectrum(velocities[-1920:, ordinal]),
            }
        )
    return {
        "schema_version": 1,
        "scope": "descriptive native actuator motion; not stability or admission",
        "body_schema_hash": trace["body_schema_hash"],
        "compiled_descriptor_hash": trace["compiled_descriptor_hash"],
        "terminal_reason": trace["reason"],
        "seconds": len(steps) / 240,
        "spectrum_window_seconds": min(1920, len(steps)) / 240,
        "channels": rows,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("descriptor", type=Path)
    parser.add_argument("trace", type=Path)
    args = parser.parse_args()
    descriptor_bytes = args.descriptor.read_bytes()
    trace_bytes = args.trace.read_bytes()
    result = summarize(json.loads(descriptor_bytes), json.loads(trace_bytes))
    result["evidence_sha256"] = {
        "descriptor": hashlib.sha256(descriptor_bytes).hexdigest(),
        "trace": hashlib.sha256(trace_bytes).hexdigest(),
        "script": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    }
    print(json.dumps(result, indent=2, allow_nan=False))


if __name__ == "__main__":
    main()
