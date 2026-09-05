import copy
import unittest

import numpy as np

from lab.scripts.audit_coupled_effort_response import (
    analyze,
    analyze_cold,
    response_matrices,
)


def manufactured_response():
    matrix = np.eye(23) * 0.75 + 0.125
    trials = []
    for magnitude in (10000, 20000):
        for dof in range(23):
            for sign in (-1, 1):
                delta = sign * magnitude
                effort = [0] * 23
                effort[dof] = delta
                trials.append(
                    {
                        "dof_ordinal": dof,
                        "delta_effort_unm": delta,
                        "applied_efforts_by_dof_unm": effort,
                        "after": {
                            "joints": [
                                {
                                    "ordinal": row,
                                    "velocity_urad_s": int(matrix[row, dof] * delta),
                                }
                                for row in range(23)
                            ]
                        },
                    }
                )
    return matrix, {
        "contract": "r8b-coupled-effort-response.v1",
        "physics_hz": 240,
        "prefix_steps": 240,
        "exact_reconstructions": 94,
        "control_after": {},
        "repeat_control_after": {},
        "baseline_efforts_by_dof_unm": [0] * 23,
        "trials": trials,
    }


def manufactured_cold_pair():
    _, response = manufactured_response()
    response["contract"] = "r8b-cold-contact-response.v1"
    response["prefix_steps"] = 0
    response["control_after"] = {"contacts": []}
    response["repeat_control_after"] = {"contacts": []}
    for trial in response["trials"]:
        trial["after"]["contacts"] = []
    ground = copy.deepcopy(response)
    ground["control_after"]["contacts"] = [
        {"actors": [1, 2], "impulse_uns": [0, 1000000, 0]}
    ]
    ground["repeat_control_after"] = copy.deepcopy(ground["control_after"])
    return {
        "translation_um": [0, 500000, 0],
        "source_body_schema_hash": "manufactured",
        "cold_response": {"grounded": ground, "raised": response},
    }


class CoupledResponseTests(unittest.TestCase):
    def test_cold_pair_requires_loaded_control_and_ground_free_raised_case(self):
        trace = manufactured_cold_pair()
        result = analyze_cold(trace)
        self.assertEqual(result["grounded_control_support_impulse_ns"], 1)
        self.assertTrue(all(c["criterion_satisfied"] for c in result["cases"].values()))

    def test_invalid_cold_boundaries_fail(self):
        for case in range(6):
            trace = manufactured_cold_pair()
            ground, air = (trace["cold_response"][k] for k in ("grounded", "raised"))
            if case == 0:
                air["trials"][0]["after"]["contacts"] = [
                    {"actors": [1, 2], "impulse_uns": [0, 0, 0]}
                ]
            elif case == 1:
                ground["control_after"]["contacts"][0]["impulse_uns"][1] = 0
            elif case == 2:
                air["native_source_override"] = "unmatched"
            elif case == 3:
                trace["translation_um"][1] = 0
            elif case == 4:
                air["prefix_steps"] = 240
            else:
                ground["baseline_efforts_by_dof_unm"][0] = 1
            with self.subTest(case=case), self.assertRaises(ValueError):
                analyze_cold(trace)

    def test_exact_coupled_linear_response_and_units(self):
        expected, response = manufactured_response()
        for matrix in response_matrices(response):
            np.testing.assert_array_equal(matrix, expected)
        self.assertTrue(analyze(response)["criterion_satisfied"])

    def test_amplitude_dependent_response_fails(self):
        _, response = manufactured_response()
        for trial in response["trials"]:
            if abs(trial["delta_effort_unm"]) == 20000:
                for joint in trial["after"]["joints"]:
                    joint["velocity_urad_s"] *= 2
        self.assertAlmostEqual(analyze(response)["relative_difference"], 0.5)
        self.assertFalse(analyze(response)["criterion_satisfied"])

    def test_missing_duplicate_wrong_coordinate_and_controls_fail(self):
        _, original = manufactured_response()
        for case in range(4):
            response = copy.deepcopy(original)
            if case == 0:
                response["trials"].pop()
            elif case == 1:
                response["trials"].append(response["trials"][0])
            elif case == 2:
                response["trials"][0]["applied_efforts_by_dof_unm"][1] = 1
            else:
                response["repeat_control_after"] = {"changed": True}
            with self.subTest(case=case), self.assertRaises(ValueError):
                analyze(response)


if __name__ == "__main__":
    unittest.main()
