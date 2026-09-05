import copy
import unittest

import numpy as np

from lab.scripts.audit_sampled_foot_control import pd_map, response_operators


class SampledFootControlTest(unittest.TestCase):
    def test_known_scalar_stable_and_unstable_maps(self):
        stable = pd_map(np.array([[1.0]]), [1.0], [1.0], hz=1)
        np.testing.assert_allclose(
            stable, [[15 / 32, 15 / 32], [-1, 0]], rtol=0, atol=0
        )
        self.assertAlmostEqual(max(abs(np.linalg.eigvals(stable))), np.sqrt(15 / 32))
        unstable = pd_map(np.array([[1.0]]), [0.0], [3.0], hz=1)
        np.testing.assert_allclose(
            sorted(np.linalg.eigvals(unstable)), [-2, 1], rtol=0, atol=0
        )

    def test_closed_map_matches_sixteen_constant_force_kick_drift_steps(self):
        b = np.array([[2.0, 0.2], [0.2, 1.0]])
        k, d = np.array([3.0, 7.0]), np.array([2.0, 1.0])
        q, v = np.array([0.2, -0.4]), np.array([0.3, -0.1])
        initial = np.concatenate([q, v / 240])
        acceleration = b @ (-k * q - d * v)
        for _ in range(16):
            v += acceleration / (240 * 16)
            q += v / (240 * 16)
        np.testing.assert_allclose(
            pd_map(b, k, d) @ initial, np.concatenate([q, v / 240]), rtol=0, atol=1e-14
        )

    def test_identity_response_and_rejected_bad_native_controls(self):
        trace = self.fixture()
        for b in response_operators(trace):
            np.testing.assert_allclose(b, np.eye(25), rtol=0, atol=0.01)
        for defect in ("missing", "impulse", "ground", "safety", "mapping", "repeat"):
            bad = copy.deepcopy(trace)
            if defect == "missing":
                bad["trials"].pop()
            elif defect in ("impulse", "ground"):
                bad["trials"][-1]["after"]["contacts"] = [
                    {
                        "actors": [1 if defect == "ground" else 1000, 1001],
                        "impulse_uns": [1 if defect == "impulse" else 0, 0, 0],
                    }
                ]
            elif defect == "safety":
                bad["trials"][-1]["observed_safety"] = "invalid"
            elif defect == "mapping":
                bad["trials"][-1]["applied_efforts_by_dof_unm"][0] = 1
            else:
                bad["trials"][1]["after"]["joints"][0]["velocity_urad_s"] = 1
            with self.assertRaises(ValueError, msg=defect):
                response_operators(bad)

    @staticmethod
    def fixture():
        initial = [
            {
                "ordinal": i,
                "position_urad": 100_000 if i in (3, 10, 20, 24) else 0,
                "velocity_urad_s": 0,
            }
            for i in range(25)
        ]
        inputs = [(None, 0), (None, 0)] + [
            (i, sign * mag)
            for mag in (10_000, 20_000)
            for i in range(25)
            for sign in (-1, 1)
        ]
        trials = []
        for dof, value in inputs:
            after = copy.deepcopy(initial)
            efforts = [0] * 25
            if dof is not None:
                efforts[dof] = value
                after[dof]["velocity_urad_s"] = round(value / 240)
            trials.append(
                {
                    "dof": dof,
                    "effort_unm": value,
                    "applied_efforts_by_dof_unm": efforts,
                    "observed_safety": "valid",
                    "after": {"joints": after, "contacts": []},
                }
            )
        return {
            "probe": "FOOT-CONTROL-01.v1",
            "physics_hz": 240,
            "position_iterations": 16,
            "initial": {"joints": initial},
            "trials": trials,
        }


if __name__ == "__main__":
    unittest.main()
