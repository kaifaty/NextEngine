"""FOOT-CONTROL-01 local response and explicitly bounded linear PD model."""

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np


def pd_map(response, stiffness, damping, hz=240, iterations=16):
    """State [q, h*v], constant outer-step effort, n kick/drift integrations."""
    h = 1 / hz
    alpha = (iterations + 1) / (2 * iterations)
    identity = np.eye(len(response))
    k = h * h * response @ np.diag(stiffness)
    d = h * response @ np.diag(damping)
    return np.block([[identity - alpha * k, identity - alpha * d], [-k, identity - d]])


def response_operators(trace):
    if trace["probe"] != "FOOT-CONTROL-01.v1" or (
        trace["physics_hz"],
        trace["position_iterations"],
    ) != (240, 16):
        raise ValueError("profile mismatch")
    expected = [(None, 0), (None, 0)] + [
        (dof, sign * magnitude)
        for magnitude in (10_000, 20_000)
        for dof in range(25)
        for sign in (-1, 1)
    ]
    if [(t["dof"], t["effort_unm"]) for t in trace["trials"]] != expected:
        raise ValueError("trial census/order")
    initial = trace["initial"]["joints"]
    if [j["ordinal"] for j in initial] != list(range(25)):
        raise ValueError("initial DOFs")
    for j in initial:
        if j["velocity_urad_s"] != 0 or j["position_urad"] != (
            100_000 if j["ordinal"] in (3, 10, 20, 24) else 0
        ):
            raise ValueError("interior reset pose")
    outputs = {}
    for trial in trace["trials"]:
        if trial["observed_safety"] != "valid":
            raise ValueError("native observed safety failure")
        expected_efforts = [0] * 25
        if trial["dof"] is not None:
            expected_efforts[trial["dof"]] = trial["effort_unm"]
        if expected_efforts != trial["applied_efforts_by_dof_unm"]:
            raise ValueError("effort mapping")
        after = trial["after"]
        if [j["ordinal"] for j in after["joints"]] != list(range(25)):
            raise ValueError("observed DOFs")
        if any(1 in c["actors"] or any(c["impulse_uns"]) for c in after["contacts"]):
            raise ValueError("contact-contaminated response")
        outputs[trial["dof"], trial["effort_unm"]] = np.array(
            [j["velocity_urad_s"] for j in after["joints"]], dtype=np.float64
        )
    if trace["trials"][0] != trace["trials"][1]:
        raise ValueError("zero repeat mismatch")
    zero = trace["trials"][0]["after"]["joints"]
    if any(
        abs(z["position_urad"] - q["position_urad"]) > 1
        or abs(z["velocity_urad_s"]) > 1
        for z, q in zip(zero, initial, strict=True)
    ):
        raise ValueError("zero motion control")
    return [
        np.column_stack(
            [
                (outputs[dof, magnitude] - outputs[dof, -magnitude])
                * 240
                / (2 * magnitude)
                for dof in range(25)
            ]
        )
        for magnitude in (10_000, 20_000)
    ]


def analyze(trace, descriptor):
    if trace["body_schema_hash"] != descriptor["body_schema_hash"]:
        raise ValueError("body identity")
    operators = response_operators(trace)
    norm = max(float(np.max(np.abs(b))) for b in operators)
    if norm == 0:
        raise ValueError("empty response")
    difference = float(np.max(np.abs(operators[0] - operators[1]))) / norm
    reciprocity = [
        float(np.max(np.abs(b - b.T))) / float(np.max(np.abs(b))) for b in operators
    ]
    positive = [np.linalg.eigvalsh((b + b.T) / 2).tolist() for b in operators]
    adequate = (
        difference <= 0.01
        and max(reciprocity) <= 0.01
        and min(min(p) for p in positive) > 0
    )
    actuators = sorted(descriptor["actuators"], key=lambda a: a["dof_ordinal"])
    if [a["dof_ordinal"] for a in actuators] != list(range(25)):
        raise ValueError("actuator coverage")
    k = [a["stiffness_q16"] / 65536 for a in actuators]
    d = [a["damping_q16"] / 65536 for a in actuators]
    models = []
    if adequate:
        for b in operators:
            matrix = pd_map(b, k, d)
            values, vectors = np.linalg.eig(matrix)
            dominant = int(np.argmax(abs(values)))
            participation = (
                abs(vectors[:25, dominant]) ** 2 + abs(vectors[25:, dominant]) ** 2
            )
            participation /= sum(participation)
            models.append(
                {
                    "spectral_radius": float(max(abs(values))),
                    "eigenvalues_re_im": [
                        [float(v.real), float(v.imag)] for v in values
                    ],
                    "dominant_mode_participation": {
                        a["joint_id"]: float(weight)
                        for a, weight in zip(actuators, participation, strict=True)
                    },
                    "matrix_q_hv": matrix.tolist(),
                }
            )
    return {
        "probe": trace["probe"],
        "response_operators": [b.tolist() for b in operators],
        "relative_amplitude_difference": difference,
        "relative_reciprocity": reciprocity,
        "symmetric_part_eigenvalues": positive,
        "adequacy_pass": adequate,
        "model": models,
        "model_decision": "not_fit"
        if not adequate
        else (
            "outside_margin"
            if min(m["spectral_radius"] for m in models) > 1.1
            else (
                "inside_margin"
                if max(m["spectral_radius"] for m in models) < 0.99
                else "inconclusive"
            )
        ),
        "scope": "finite unloaded interior-pose response; unclipped frozen linear model only",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("descriptor", "trace", "output"):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    descriptor, trace = [
        json.loads(p.read_text()) for p in (args.descriptor, args.trace)
    ]
    expected_outer = hashlib.sha256(
        b"nextengine.compiled-body-schema.v4\0"
        + bytes.fromhex(descriptor["compiled_descriptor_hash"])
        + b"nextengine.physics.humanoid-per-iteration-external-forces.v1\0"
        + (1).to_bytes(4, "little")
        + (1).to_bytes(4, "little")
    ).hexdigest()
    if trace["compiled_descriptor_hash"] != expected_outer:
        raise ValueError("compiled identity")
    result = analyze(trace, descriptor)
    result["numpy_version"] = np.__version__
    result["sha256"] = {
        str(p): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in (args.descriptor, args.trace, Path(__file__))
    }
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
