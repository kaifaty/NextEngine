from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np

from next_lab.contact_projected_direction_audit import (
    _validate_profile,
    construct_candidate_positions,
    quantize_projected_direction,
    selected_slice_identity,
    validate_candidate_position_scope,
)


class ContactProjectedDirectionAuditTests(unittest.TestCase):
    def test_tracked_profile_freezes_one_exact_offline_candidate(self) -> None:
        path = (
            Path(__file__).resolve().parents[1]
            / "profiles"
            / "humanoid-contact-projected-direction-exact-audit-r107.v1.json"
        )
        profile = json.loads(path.read_bytes())
        _validate_profile(profile)
        self.assertEqual(profile["scope"]["candidate_count"], 1)
        self.assertEqual(profile["method"]["physx_runs"], 0)
        self.assertEqual(profile["method"]["optimizer_steps"], 0)

    def test_quantized_candidate_changes_only_declared_cells(self) -> None:
        local_scale = np.ones(5, dtype=np.float64)
        projected = np.asarray(
            [
                1.5e-6,
                2.5e-6,
                -1.5e-6,
                3.5e-6,
                -2.5e-6,
                0.0,
                -0.5e-6,
                0.5e-6,
                1.5e-6,
                -3.5e-6,
            ],
            dtype=np.float64,
        )
        root_delta, joint_delta = quantize_projected_direction(
            projected=projected,
            local_scale=local_scale,
            support_frame_count=2,
        )
        np.testing.assert_array_equal(
            root_delta,
            np.asarray([[2, 2, -2], [0, 0, 0]], dtype=np.int64),
        )
        np.testing.assert_array_equal(
            joint_delta,
            np.asarray([[4, -2], [2, -4]], dtype=np.int64),
        )

        source_root = np.zeros((6, 3), dtype=np.int64)
        source_joint = np.zeros((6, 4), dtype=np.int64)
        selected = np.asarray([1, 3], dtype=np.int64)
        candidate_root, candidate_joint = construct_candidate_positions(
            source_root=source_root,
            source_joint=source_joint,
            root_delta_um=root_delta,
            joint_delta_urad=joint_delta,
            selected_dof_ordinals=selected,
            support_frame_first=2,
            support_frame_last=3,
        )
        facts = validate_candidate_position_scope(
            source_root=source_root,
            source_joint=source_joint,
            candidate_root=candidate_root,
            candidate_joint=candidate_joint,
            selected_dof_ordinals=selected,
            support_frame_first=2,
            support_frame_last=3,
        )
        self.assertEqual(facts["status"], "PASS")
        self.assertTrue(np.array_equal(candidate_root[:2], source_root[:2]))
        self.assertEqual(np.count_nonzero(candidate_joint[:, [0, 2]]), 0)

    def test_selected_slice_identity_is_direct_and_typed(self) -> None:
        report = selected_slice_identity(
            arrays={
                "a": np.arange(18, dtype=np.int64).reshape(6, 3),
                "b": np.arange(6, dtype=np.uint8),
            },
            frame_first=2,
            frame_last=4,
        )
        self.assertEqual(report["status"], "PASS")
        self.assertEqual(report["frame_count"], 3)
        self.assertEqual(report["arrays"]["a"]["shape"], [3, 3])
        self.assertEqual(report["arrays"]["b"]["dtype"], "|u1")


if __name__ == "__main__":
    unittest.main()
