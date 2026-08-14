from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.canonical_material_point_force_formulation import (
    _validate_profile,
    audit_material_repair,
    audit_point_force_coordinates,
)

PROFILE = (
    Path(__file__).resolve().parents[1]
    / "profiles"
    / "humanoid-canonical-material-point-force-repair-r110.v1.json"
)
FIXTURE = Path(__file__).parent / "fixtures/biomechanics_motor_mirror_v1.json"


class CanonicalMaterialPointForceFormulationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())
        self.descriptor = json.loads(FIXTURE.read_bytes())

    def test_profile_authorizes_only_r111_implementation(self) -> None:
        _validate_profile(self.profile)
        self.assertFalse(self.profile["scope"]["runtime_changes"])
        self.assertEqual(self.profile["scope"]["solver_runs"], 0)
        self.assertEqual(
            self.profile["decision"]["complete"],
            "PERMIT_R111_CANONICAL_MATERIAL_LINEAGE_IMPLEMENTATION_ONLY",
        )
        self.assertEqual(self.profile["bounded_acceptance"]["physx"], "NOT_AUTHORIZED")

    def test_material_repair_is_exact_q16_new_lineage(self) -> None:
        result = audit_material_repair(
            profile=self.profile,
            descriptor=self.descriptor,
        )
        self.assertEqual(
            result["source_collider_material_id_counts"],
            {
                "physics-material.humanoid-body.v1": 17,
                "physics-material.humanoid-sole.v1": 2,
            },
        )
        self.assertTrue(result["all_current_coefficients_equal"])
        self.assertEqual(result["quantization_audit"]["static_friction"]["q16"], 52429)
        self.assertEqual(result["quantization_audit"]["dynamic_friction"]["q16"], 45875)
        self.assertEqual(result["status"], "FORMULATED_NEW_LINEAGE")
        self.assertFalse(result["runtime_implemented"])

    def test_missing_source_material_assignment_fails_closed(self) -> None:
        descriptor = copy.deepcopy(self.descriptor)
        descriptor["bodies"][0]["colliders"][0]["material_id"] = (
            "physics-material.unknown.v1"
        )
        with self.assertRaisesRegex(ValueError, "source collider material"):
            audit_material_repair(profile=self.profile, descriptor=descriptor)

    def test_point_force_order_closes_v9_effectors_and_foot_bodies(self) -> None:
        result = audit_point_force_coordinates(
            profile=self.profile,
            descriptor=self.descriptor,
            metadata=_metadata(),
            contact_modes=np.zeros((801, 2), dtype=np.uint8),
        )
        self.assertEqual(
            result["contact_mode_counts"], {"0": 1602, "1": 0, "2": 0, "3": 0}
        )
        self.assertEqual(
            [row["effector_id"] for row in result["application_point_identity"]],
            [
                "effector.left-heel",
                "effector.left-forefoot",
                "effector.right-heel",
                "effector.right-forefoot",
            ],
        )

    def test_point_force_formulation_has_no_independent_moment(self) -> None:
        coordinates = self.profile["point_contact_force_coordinates"]
        self.assertEqual(
            coordinates["force_component_order"],
            ["normal_newtons", "right_newtons", "forward_newtons"],
        )
        self.assertIn(
            "no independent torque, yaw moment, center-of-pressure or six-dimensional wrench variable",
            coordinates["constraints"],
        )
        altered = copy.deepcopy(self.profile)
        altered["point_contact_force_coordinates"]["ordered_points"][0]["body_id"] = (
            "body.pelvis"
        )
        with self.assertRaisesRegex(ValueError, "ordered contact point"):
            audit_point_force_coordinates(
                profile=altered,
                descriptor=self.descriptor,
                metadata=_metadata(),
                contact_modes=np.zeros((801, 2), dtype=np.uint8),
            )


def _metadata() -> dict[str, object]:
    return {
        "artifact_id": "cmu16-walk-nominal-b--complete",
        "clip_id": "cmu16-walk-nominal-b",
        "frame_first": 0,
        "frame_last": 800,
        "effector_ids": [
            "effector.left-forefoot",
            "effector.left-heel",
            "effector.left-palm",
            "effector.right-forefoot",
            "effector.right-heel",
            "effector.right-palm",
        ],
    }


if __name__ == "__main__":
    unittest.main()
