from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.kto_linearization_repair_conformance import (
    AnchorDefinition,
    _validate_profile,
    hybrid_velocity_stencil,
    nearest_yaw_delta,
    rotation_exp,
    select_anchor_definitions,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT / "lab/profiles/humanoid-kto-linearization-repair-conformance-r118.v1.json"
)


class KtoLinearizationRepairConformanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_report_only_r119_on_pass(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(self.profile["scope"]["qp_solves"], 0)
        self.assertEqual(self.profile["scope"]["kto_solves"], 0)
        self.assertEqual(
            self.profile["decision"]["pass"],
            "PERMIT_SEPARATE_REPORT_ONLY_R119_REPAIRED_KTO_EXECUTION_FORMULATION_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["additional_kto_solve"],
            "NOT_AUTHORIZED",
        )

    def test_profile_freezes_norm_squared_not_component_box(self) -> None:
        self.assertEqual(
            self.profile["function_contract"]["tangential_constraint"],
            "velocity_x_m_s^2 + velocity_z_m_s^2 <= 0.12^2",
        )
        invalid = copy.deepcopy(self.profile)
        invalid["function_contract"]["tangential_constraint"] = "component box"
        with self.assertRaisesRegex(ValueError, "profile differs"):
            _validate_profile(invalid)

    def test_solver_free_stencil_uses_entry_exit_and_centered_rows(self) -> None:
        active = np.zeros((801, 2, 2), dtype=np.bool_)
        active[10:14, 0, 1] = True
        active[20:24, 1, 0] = True
        indices, coefficients = hybrid_velocity_stencil(active)
        self.assertEqual(indices[10].tolist(), [10, 11])
        self.assertEqual(coefficients[10].tolist(), [-60.0, 60.0])
        self.assertEqual(indices[13].tolist(), [12, 13])
        self.assertEqual(coefficients[13].tolist(), [-60.0, 60.0])
        self.assertEqual(indices[11].tolist(), [10, 12])
        self.assertEqual(coefficients[11].tolist(), [-30.0, 30.0])

    def test_anchor_selection_is_deterministic_for_each_role(self) -> None:
        active = np.zeros((801, 2, 2), dtype=np.bool_)
        active[10:14, 0, 1] = True
        active[328, 0, 1] = True
        active[20:24, 1, 0] = True
        indices, coefficients = hybrid_velocity_stencil(active)
        expected = (
            "frame-328:left-forefoot",
            "frame-10:left-forefoot",
            "frame-13:left-forefoot",
            "frame-11:left-forefoot",
            "frame-20:right-heel",
            "frame-23:right-heel",
            "frame-21:right-heel",
        )
        selected = select_anchor_definitions(
            active=active,
            stencil_indices=indices,
            stencil_coefficients=coefficients,
            expected=expected,
        )
        self.assertEqual(tuple(row.point_id for row in selected), expected)
        self.assertTrue(all(isinstance(row, AnchorDefinition) for row in selected))

    def test_nearest_yaw_delta_keeps_continuous_source_branch(self) -> None:
        source = rotation_exp(np.asarray((0.0, np.pi - 0.001, 0.0)))
        candidate = rotation_exp(np.asarray((0.0, np.pi + 0.001, 0.0)))
        self.assertAlmostEqual(nearest_yaw_delta(candidate, source), 0.002, places=9)


if __name__ == "__main__":
    unittest.main()
