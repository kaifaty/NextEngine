import copy
import unittest

import numpy as np

from lab.scripts.audit_coupled_effort_response import analyze, response_matrices


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


class CoupledResponseTests(unittest.TestCase):
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
