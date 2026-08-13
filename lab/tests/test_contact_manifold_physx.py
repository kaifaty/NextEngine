from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import numpy as np

from next_lab.contact_manifold_physx import (
    ContactPrototypeCase,
    authored_root_state,
    build_fresh_scene_usda,
    compare_reset_paths,
    evaluate_bounded_acceptance,
    overlay_bounded_reference_window,
)


class ContactManifoldPhysxTests(unittest.TestCase):
    def test_bounded_reference_overlay_retains_source_lookahead(self) -> None:
        source = np.asarray(((10,), (20,), (30,), (40,)), dtype=np.int64)
        projected = np.asarray(
            (
                ((100,), (101,), (102,)),
                ((200,), (201,), (202,)),
                ((300,), (301,), (302,)),
                ((400,), (401,), (402,)),
            ),
            dtype=np.int64,
        )
        result = overlay_bounded_reference_window(
            source_values=source,
            projected_by_environment=projected,
            selected_environment_ids=np.arange(4, dtype=np.int64),
            selected_frames=np.asarray((23, 25, 26, 40), dtype=np.int64),
            episode_start_frames=np.asarray((23, 23, 23, 23), dtype=np.int64),
            where=np.where,
        )

        np.testing.assert_array_equal(result[:, 0], (100, 202, 30, 40))

        with self.assertRaisesRegex(ValueError, "differ in shape"):
            overlay_bounded_reference_window(
                source_values=source[:3],
                projected_by_environment=projected,
                selected_environment_ids=np.arange(4, dtype=np.int64),
                selected_frames=np.asarray((23, 25, 26, 40), dtype=np.int64),
                episode_start_frames=np.asarray((23, 23, 23, 23), dtype=np.int64),
                where=np.where,
            )

    def test_fresh_scene_usd_authors_root_link_and_joint_state(self) -> None:
        arrays = _arrays()
        half = np.sqrt(0.5)
        arrays["root_quaternion_q1_30"][0] = np.rint(
            np.asarray((0.0, half, 0.0, half)) * (1 << 30)
        ).astype(np.int64)
        arrays["root_linear_velocity_um_s"][0] = (1_000_000, 0, 0)
        arrays["root_yaw_velocity_urad_s"][0] = 1_000_000
        arrays["joint_position_urad"][0, 0] = 1_570_796
        arrays["joint_velocity_urad_s"][0, 0] = 3_141_593
        descriptor = {
            "bodies": [
                {
                    "body_id": "body.pelvis",
                    "parent_body_slot": None,
                    "center_of_mass_micrometres": (0, 0, -100_000),
                }
            ],
            "joints": [
                {"joint_id": "joint.left-knee", "dof_ordinal": 0},
            ]
        }
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory) / "humanoid.usda"
            base.write_text("#usda 1.0\n", encoding="utf-8")
            payload = build_fresh_scene_usda(
                base_usd_path=base,
                descriptor=descriptor,
                arrays=arrays,
                source_case_ordinal=42,
                artifact_sha256="a" * 64,
            )

        root = authored_root_state(arrays)
        np.testing.assert_allclose(root.linear_velocity_local_m_s, (0.0, -1.0, 0.0), atol=2e-9)
        self.assertIn('prepend apiSchemas = ["PhysicsJointStateAPI:angular"]', payload)
        self.assertIn("float state:angular:physics:position = 89.9999813", payload)
        self.assertIn("float state:angular:physics:velocity = 180.00002", payload)
        self.assertIn("vector3f physics:velocity = (1, -0.1, 0)", payload)
        self.assertIn("vector3f physics:angularVelocity = (0, 0, 57.2957795)", payload)
        self.assertIn('over "joint_left_knee"', payload)
        self.assertIn("nextengine:freshSceneSourceCaseOrdinal = 42", payload)

    def test_reset_comparison_requires_outcomes_and_bounded_impulses(self) -> None:
        fresh = [_row(0, "PASS", (), 11, (100, 200))]
        partial = [_row(0, "PASS", (), 11, (110, 190))]
        comparison = compare_reset_paths(
            fresh_rows=fresh,
            partial_rows=partial,
            maximum_first_tick_impulse_delta=10,
        )
        self.assertEqual(comparison["status"], "PASS")
        partial[0]["first_tick_impulses"] = [111, 190]
        comparison = compare_reset_paths(
            fresh_rows=fresh,
            partial_rows=partial,
            maximum_first_tick_impulse_delta=10,
        )
        self.assertEqual(comparison["status"], "FAIL")

    def test_bounded_acceptance_requires_controls_and_each_category(self) -> None:
        cases = (
            _case(0, "PASS", (), ()),
            _case(1, "FAIL", ("hard_impact",), ("hard_impact",)),
            _case(2, "FAIL", ("hard_rom",), ("hard_rom",)),
            _case(3, "FAIL", ("joint_velocity",), ("joint_velocity",)),
        )
        passing = [
            _row(0, "PASS", (), 11, (0,)),
            _row(1, "PASS", (), 11, (0,)),
            _row(2, "PASS", (), 11, (0,)),
            _row(3, "PASS", (), 11, (0,)),
        ]
        accepted = evaluate_bounded_acceptance(cases=cases, rows=passing)
        self.assertTrue(accepted["accepted_for_full_v19_build"])
        passing[0] = _row(0, "FAIL", ("hard_impact",), 1, (0,))
        rejected = evaluate_bounded_acceptance(cases=cases, rows=passing)
        self.assertFalse(rejected["accepted_for_full_v19_build"])
        self.assertEqual(rejected["passing_control_regression_count"], 1)


def _arrays() -> dict[str, np.ndarray]:
    root_quaternion = np.zeros((2, 4), dtype=np.int64)
    root_quaternion[:, 3] = 1 << 30
    return {
        "root_position_um": np.asarray(((0, 943_500, 0), (0, 943_500, 0)), dtype=np.int64),
        "root_quaternion_q1_30": root_quaternion,
        "root_linear_velocity_um_s": np.zeros((2, 3), dtype=np.int64),
        "root_yaw_velocity_urad_s": np.zeros(2, dtype=np.int64),
        "joint_position_urad": np.zeros((2, 1), dtype=np.int64),
        "joint_velocity_urad_s": np.zeros((2, 1), dtype=np.int64),
    }


def _case(
    ordinal: int,
    baseline_status: str,
    baseline_reasons: tuple[str, ...],
    target_categories: tuple[str, ...],
) -> ContactPrototypeCase:
    return ContactPrototypeCase(
        ordinal=ordinal,
        source_case_ordinal=ordinal,
        clip_id="clip",
        split="train",
        frame_first=ordinal,
        frame_last=ordinal + 11,
        baseline_status=baseline_status,
        baseline_reasons=baseline_reasons,
        target_failure_categories=target_categories,
        artifact_path=Path("unused"),
        artifact_sha256="a" * 64,
    )


def _row(
    ordinal: int,
    status: str,
    reasons: tuple[str, ...],
    terminal_tick: int,
    impulses: tuple[int, ...],
) -> dict[str, object]:
    return {
        "case_ordinal": ordinal,
        "clip_id": "clip",
        "start_frame": ordinal,
        "status": status,
        "observed_motor_ticks": terminal_tick,
        "first_required_safety_violation": (
            {"reasons": list(reasons)} if reasons else None
        ),
        "first_tick_impulses": list(impulses),
    }


if __name__ == "__main__":
    unittest.main()
