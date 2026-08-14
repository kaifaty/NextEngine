from __future__ import annotations

import copy
import hashlib
import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.contact_manifold import contact_point_mask
from next_lab.contact_trajectory import hybrid_velocity_stencil
from next_lab.quantization_aware_kto_formulation import (
    _validate_profile,
    audit_hybrid_stencil,
    audit_tracking_prior,
    build_variable_inventory,
    classify_tracking_progress,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-quantization-aware-kto-execution-formulation-r114.v1.json"
)
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"
V9_PROFILE = ROOT / "lab/profiles/humanoid-contact-manifold-prototype.v9.json"


class QuantizationAwareKtoFormulationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_one_r115_in_memory_solve(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(self.profile["scope"]["kto_solves"], 0)
        self.assertFalse(self.profile["scope"]["candidate_construction"])
        self.assertEqual(
            self.profile["decision"]["complete"],
            "PERMIT_R115_SINGLE_BOUNDED_QUANTIZATION_AWARE_KTO_EXECUTION_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["r115_quantization_aware_kto_execution"],
            "AUTHORIZED_ONE_IN_MEMORY_SOLVE_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["additional_kto_solve"],
            "NOT_AUTHORIZED",
        )
        self.assertIn(
            "never a complete trajectory", self.profile["source_roles"]["v7_prior"]
        )

    def test_inventory_contains_complete_floating_base_and_all_joints(self) -> None:
        descriptor = json.loads(DESCRIPTOR.read_bytes())
        v9_profile = json.loads(V9_PROFILE.read_bytes())
        inventory = build_variable_inventory(
            profile=self.profile,
            descriptor=descriptor,
            v9_profile=v9_profile,
        )
        self.assertEqual(inventory["total_scalar_count"], 69_687)
        self.assertEqual(len(inventory["joint_order"]), 23)
        self.assertEqual(inventory["v9_stricter_joint_bound_count"], 10)
        self.assertEqual(
            [row["dof_ordinal"] for row in inventory["effective_joint_limits"]],
            list(range(23)),
        )

    def test_hybrid_stencil_audit_preserves_entry_precedence(self) -> None:
        modes = np.asarray(
            [
                [0, 0],
                [2, 0],
                [2, 0],
                [0, 0],
                [0, 0],
            ],
            dtype=np.uint8,
        )
        indices, coefficients = hybrid_velocity_stencil(contact_point_mask(modes))
        digest = hashlib.sha256()
        digest.update(indices.astype("<i8", copy=False).tobytes())
        digest.update(coefficients.astype("<f8", copy=False).tobytes())
        profile = copy.deepcopy(self.profile)
        profile["kinematic_equalities"]["stencil_sha256"] = digest.hexdigest()
        profile["kinematic_equalities"]["expected_stencil_counts"] = {
            "forward": 2,
            "backward": 2,
            "centered": 1,
        }
        result = audit_hybrid_stencil(profile, modes)
        self.assertEqual(result["status"], "EXACT_FIXED_60_HZ_STENCIL")
        self.assertTrue(result["entry_precedence"])

    def test_tracking_prior_and_progress_are_exact_integer_predicates(self) -> None:
        v9, v7 = _matched_arrays()
        profile = copy.deepcopy(self.profile)
        profile["tracking_progress_gate"].update(
            {
                "expected_differing_cell_count": 1,
                "v9_to_v7_l1_distance_microradians": 4,
                "v9_to_v7_squared_l2_distance_microradians_squared": 16,
            }
        )
        prior = audit_tracking_prior(profile, v9, v7)
        self.assertEqual(prior["status"], "MATCHED_PASSING_LOCAL_PRIOR_ONLY")
        self.assertFalse(prior["independently_admissible_complete_clip"])

        v9_joint = v9["joint_position_urad"][238:250].copy()
        v7_joint = v7["joint_position_urad"].copy()
        emitted = v9_joint.copy()
        emitted[3, 5] += 1
        progress = classify_tracking_progress(
            v9_joint_position_urad=v9_joint,
            v7_joint_position_urad=v7_joint,
            emitted_joint_position_urad=emitted,
        )
        self.assertEqual(progress["status"], "PASS")
        self.assertEqual(progress["directional_dot_microradians_squared"], 4)

        emitted[3, 5] = -1
        no_progress = classify_tracking_progress(
            v9_joint_position_urad=v9_joint,
            v7_joint_position_urad=v7_joint,
            emitted_joint_position_urad=emitted,
        )
        self.assertEqual(no_progress["status"], "FAIL")
        self.assertFalse(no_progress["strict_positive_directional_dot"])

        v7["contacts"][0, 0] = 1
        with self.assertRaisesRegex(ValueError, "matched V9 slice"):
            audit_tracking_prior(profile, v9, v7)


def _matched_arrays() -> tuple[dict[str, np.ndarray], dict[str, np.ndarray]]:
    v9 = {
        "reference_frame": np.arange(801, dtype=np.int64),
        "contact_modes": np.zeros((801, 2), dtype=np.uint8),
        "contacts": np.zeros((801, 7), dtype=np.uint8),
        "root_quaternion_q1_30": np.zeros((801, 4), dtype=np.int64),
        "joint_position_urad": np.zeros((801, 23), dtype=np.int64),
    }
    v7 = {
        "reference_frame": np.arange(238, 250, dtype=np.int64),
        "contact_modes": v9["contact_modes"][238:250].copy(),
        "contacts": v9["contacts"][238:250].copy(),
        "root_quaternion_q1_30": v9["root_quaternion_q1_30"][238:250].copy(),
        "joint_position_urad": v9["joint_position_urad"][238:250].copy(),
    }
    v7["joint_position_urad"][3, 5] = 4
    return v9, v7


if __name__ == "__main__":
    unittest.main()
