"""Check the finite amplitude-consistency contract of a native response probe.

This does not identify an inverse mass matrix or certify closed-loop stability.
Canonical velocity and effort both use micro-units, which cancel in the central
difference. The resulting units are (rad/s)/(N m).
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np


def response_matrices(response):
    if response["contract"] != "r8b-coupled-effort-response.v1":
        raise ValueError("unsupported research contract")
    if response["physics_hz"] != 240 or response["prefix_steps"] != 240:
        raise ValueError("wrong time/prefix profile")
    if response["exact_reconstructions"] != 94:
        raise ValueError("wrong reconstruction count")
    if response["control_after"] != response["repeat_control_after"]:
        raise ValueError("zero controls differ")
    baseline = response["baseline_efforts_by_dof_unm"]
    if len(baseline) != 23 or any(type(x) is not int for x in baseline):
        raise ValueError("invalid baseline effort vector")
    trials = response["trials"]
    expected_keys = {
        (dof, sign * m) for dof in range(23) for m in (10000, 20000) for sign in (-1, 1)
    }
    values = {}
    for trial in trials:
        key = trial["dof_ordinal"], trial["delta_effort_unm"]
        if key not in expected_keys or key in values:
            raise ValueError("unexpected or duplicate perturbation")
        command = baseline.copy()
        command[key[0]] += key[1]
        if trial["applied_efforts_by_dof_unm"] != command:
            raise ValueError("perturbation changes the wrong command")
        joints = trial["after"]["joints"]
        if [j["ordinal"] for j in joints] != list(range(23)):
            raise ValueError("joint order mismatch")
        velocity = [j["velocity_urad_s"] for j in joints]
        if any(type(v) is not int for v in velocity):
            raise ValueError("noncanonical velocity")
        values[key] = np.asarray(velocity, dtype=float)
    if set(values) != expected_keys:
        raise ValueError("missing perturbation")
    matrices = []
    for magnitude in (10000, 20000):
        matrices.append(
            np.column_stack(
                [
                    (values[dof, magnitude] - values[dof, -magnitude]) / (2 * magnitude)
                    for dof in range(23)
                ]
            )
        )
    if not all(np.isfinite(m).all() for m in matrices):
        raise ValueError("nonfinite response matrix")
    return matrices


def analyze(response):
    small, large = response_matrices(response)
    norms = [float(np.linalg.norm(m)) for m in (small, large)]
    difference = float(np.linalg.norm(small - large))
    denominator = max(norms)
    if denominator == 0:
        raise ValueError("zero response cannot resolve this contract")
    relative = difference / denominator
    columns = []
    for dof in range(23):
        norm = max(np.linalg.norm(small[:, dof]), np.linalg.norm(large[:, dof]))
        delta = float(np.linalg.norm(small[:, dof] - large[:, dof]))
        columns.append(
            {
                "dof_ordinal": dof,
                "absolute_difference": delta,
                "relative_difference": delta / float(norm) if norm else None,
            }
        )
    return {
        "schema_version": 1,
        "contract": response["contract"],
        "scope": "finite amplitude consistency only; not physical or control stability",
        "criterion": "relative Frobenius difference <= 0.05",
        "criterion_satisfied": relative <= 0.05,
        "matrix_units": "(rad/s)/(N m)",
        "matrix_norms": norms,
        "absolute_difference": difference,
        "relative_difference": relative,
        "off_diagonal_norms": [
            float(np.linalg.norm(m - np.diag(np.diag(m)))) for m in (small, large)
        ],
        "columns": columns,
        "matrices": [m.tolist() for m in (small, large)],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path)
    args = parser.parse_args()
    data = args.trace.read_bytes()
    trace = json.loads(data)
    response = trace["response_probe"]
    if response["compiled_descriptor_hash"] != trace["compiled_descriptor_hash"]:
        raise ValueError("compiled identity mismatch")
    if (
        trace["physics_substeps"] != 240
        or trace["reason"] != "diagnostic.response-prefix"
    ):
        raise ValueError("incomplete recorded prefix")
    result = analyze(response)
    result["compiled_descriptor_hash"] = trace["compiled_descriptor_hash"]
    result["body_schema_hash"] = trace["body_schema_hash"]
    result["evidence_sha256"] = {
        "trace": hashlib.sha256(data).hexdigest(),
        "script": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    }
    print(json.dumps(result, indent=2, allow_nan=False))


if __name__ == "__main__":
    main()
