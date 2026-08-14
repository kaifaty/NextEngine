from __future__ import annotations

import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from typing import Any

import numpy as np

from next_lab.contact_manifold_physx import (
    ContactPrototypeCase,
    authored_root_state,
    build_fresh_scene_usda,
    complete_clip_probe_shape_is_valid,
    compare_reset_paths,
    counterfactual_probe_shape_is_valid,
    evaluate_bounded_acceptance,
    evaluate_counterfactual_acceptance,
    minimum_normalized_quaternion_dot_q1_30,
    native_dynamics_trace_probe_shape_is_valid,
    overlay_bounded_reference_window,
)


class ContactManifoldPhysxTests(unittest.TestCase):
    def test_native_trace_profile_has_no_acceptance_authority(self) -> None:
        counterfactual_cases = (
            replace(_case(0, "PASS", (), ()), source_case_ordinal=25),
            replace(_case(1, "PASS", (), ()), source_case_ordinal=7967),
        )
        counterfactual_profile = _native_trace_profile(
            role="v11-emitted-acceleration", worker_case_ordinals=[0, 1]
        )
        self.assertTrue(
            native_dynamics_trace_probe_shape_is_valid(
                profile=counterfactual_profile,
                prototype_manifest=_counterfactual_manifest(),
                cases=counterfactual_cases,
            )
        )
        counterfactual_profile["bounded_acceptance"]["pass_gate_decision"] = (
            "PERMIT_MERGED_OFFLINE_COUNTERFACTUAL_ONLY"
        )
        self.assertFalse(
            native_dynamics_trace_probe_shape_is_valid(
                profile=counterfactual_profile,
                prototype_manifest=_counterfactual_manifest(),
                cases=counterfactual_cases,
            )
        )

        baseline_cases = tuple(
            replace(
                _case(ordinal, "PASS", (), ()),
                source_case_ordinal=7967 if ordinal == 10 else ordinal,
            )
            for ordinal in range(17)
        )
        self.assertTrue(
            native_dynamics_trace_probe_shape_is_valid(
                profile=_native_trace_profile(
                    role="v9-baseline", worker_case_ordinals=[10]
                ),
                prototype_manifest=_v9_manifest(),
                cases=baseline_cases,
            )
        )

    def test_counterfactual_probe_requires_exact_two_case_bundle(self) -> None:
        cases = (
            replace(_case(0, "PASS", (), ()), source_case_ordinal=25),
            replace(_case(1, "PASS", (), ()), source_case_ordinal=7967),
        )
        profile = {
            "execution": {
                "indexed_partial_reset": {
                    "enabled": False,
                    "evidence_role": "report-only",
                }
            },
            "bounded_acceptance": {
                "acceptance_authority": "fresh-scene",
                "evaluation_mode": "two-counterfactual-controls-must-pass",
                "pass_gate_decision": (
                    "PERMIT_MERGED_OFFLINE_COUNTERFACTUAL_ONLY"
                ),
                "fail_gate_decision": "STOP_AND_RESEARCH",
            },
        }
        manifest = {
            "check": "TRAIN-4-CONTACT-MANIFOLD-COUNTERFACTUAL-BUNDLE",
            "prototype_id": (
                "nextengine.humanoid-contact-counterfactual-bundle.v1"
            ),
            "scope": {
                "case_scope": "two-independent-counterfactuals",
                "case_count": 2,
                "failure_case_count": 0,
                "control_case_count": 2,
                "ordered_r95_case_ordinals": [2, 10],
                "ordered_source_case_ordinals": [25, 7967],
            },
            "identities": {
                "source_counterfactuals": [
                    {"role": "contact-reserve", "r95_case_ordinal": 2},
                    {
                        "role": "emitted-acceleration",
                        "r95_case_ordinal": 10,
                    },
                ]
            },
            "cases": [
                {
                    "counterfactual_role": "contact-reserve",
                    "r95_case_ordinal": 2,
                    "exact_complete_clip_slice_status": "PASS",
                },
                {
                    "counterfactual_role": "emitted-acceleration",
                    "r95_case_ordinal": 10,
                    "exact_complete_clip_slice_status": "PASS",
                },
            ],
        }
        self.assertTrue(
            counterfactual_probe_shape_is_valid(
                profile=profile,
                prototype_manifest=manifest,
                cases=cases,
            )
        )
        manifest["scope"]["ordered_r95_case_ordinals"] = [10, 2]
        self.assertFalse(
            counterfactual_probe_shape_is_valid(
                profile=profile,
                prototype_manifest=manifest,
                cases=cases,
            )
        )

    def test_complete_clip_probe_requires_exact_v9_offline_pass(self) -> None:
        cases = tuple(_case(ordinal, "PASS", (), ()) for ordinal in range(17))
        profile = {
            "execution": {
                "indexed_partial_reset": {
                    "enabled": False,
                    "evidence_role": "report-only",
                }
            },
            "bounded_acceptance": {
                "acceptance_authority": "fresh-scene",
                "pass_gate_decision": "PERMIT_FULL_V19_DATA_BUILD_ONLY",
                "fail_gate_decision": "STOP_AND_RESEARCH",
            },
        }
        manifest = {
            "prototype_id": "nextengine.humanoid-contact-manifold-prototype.v9",
            "scope": {
                "case_scope": "all",
                "failure_case_count": 7,
                "control_case_count": 10,
                "projection": {
                    "trajectory_closure": {
                        "algorithm_id": (
                            "nextengine.dimensionless-contact-trajectory-qp.v9"
                        )
                    }
                },
            },
            "complete_clips": [
                {
                    "solve_count": 1,
                    "projection_diagnostics": {
                        "status": "PASS",
                        "contact_point_deletion_count": 0,
                    },
                }
                for _ in range(3)
            ],
            "cases": [
                {"exact_complete_clip_slice_status": "PASS"}
                for _ in range(17)
            ],
            "exact_slice_identity": {
                "status": "PASS",
                "disagreement_count": 0,
            },
        }

        self.assertTrue(
            complete_clip_probe_shape_is_valid(
                profile=profile,
                prototype_manifest=manifest,
                cases=cases,
            )
        )
        manifest["complete_clips"][2]["projection_diagnostics"][
            "contact_point_deletion_count"
        ] = 1
        self.assertFalse(
            complete_clip_probe_shape_is_valid(
                profile=profile,
                prototype_manifest=manifest,
                cases=cases,
            )
        )

    def test_quaternion_dot_normalizes_physx_float_scale(self) -> None:
        expected = np.asarray(((1.0, 0.0, 0.0, 0.0),), dtype=np.float64)
        actual = np.asarray(((0.9999999, 0.0, 0.0, 0.0),), dtype=np.float32)

        self.assertEqual(
            minimum_normalized_quaternion_dot_q1_30(actual, expected), 1 << 30
        )
        with self.assertRaisesRegex(ValueError, "invalid value"):
            minimum_normalized_quaternion_dot_q1_30(
                np.zeros((1, 4), dtype=np.float64), expected
            )

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

    def test_counterfactual_acceptance_never_authorizes_all17(self) -> None:
        cases = (
            _case(0, "PASS", (), ()),
            _case(1, "PASS", (), ()),
        )
        rows = [
            _row(0, "PASS", (), 11, (0,)),
            _row(1, "PASS", (), 11, (0,)),
        ]
        accepted = evaluate_counterfactual_acceptance(cases=cases, rows=rows)
        self.assertEqual(accepted["status"], "PASS")
        self.assertTrue(accepted["counterfactuals_supported"])
        self.assertFalse(accepted["all_17_fresh_probe_authorized"])
        rows[1] = _row(1, "FAIL", ("hard_rom",), 10, (0,))
        rejected = evaluate_counterfactual_acceptance(cases=cases, rows=rows)
        self.assertEqual(rejected["status"], "FAIL")


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


def _native_trace_profile(
    *, role: str, worker_case_ordinals: list[int]
) -> dict[str, Any]:
    return {
        "execution": {
            "worker_case_ordinals": worker_case_ordinals,
            "native_dynamics_trace": {
                "enabled": True,
                "evidence_role": "report-only",
                "comparison_role": role,
                "physics_substeps_per_motor_tick": 4,
                "action_channel_scope": "all-ordered-action-channels",
                "contact_pair_scope": "all-frozen-contact-pairs",
            },
            "indexed_partial_reset": {
                "enabled": False,
                "evidence_role": "report-only",
            },
        },
        "bounded_acceptance": {
            "acceptance_authority": "fresh-scene",
            "evaluation_mode": "report-only-native-dynamics-trace",
            "pass_gate_decision": "STOP_AND_RESEARCH",
            "fail_gate_decision": "STOP_AND_RESEARCH",
        },
    }


def _counterfactual_manifest() -> dict[str, Any]:
    return {
        "check": "TRAIN-4-CONTACT-MANIFOLD-COUNTERFACTUAL-BUNDLE",
        "prototype_id": "nextengine.humanoid-contact-counterfactual-bundle.v1",
        "scope": {
            "case_scope": "two-independent-counterfactuals",
            "case_count": 2,
            "failure_case_count": 0,
            "control_case_count": 2,
            "ordered_r95_case_ordinals": [2, 10],
            "ordered_source_case_ordinals": [25, 7967],
        },
        "identities": {
            "source_counterfactuals": [
                {"role": "contact-reserve", "r95_case_ordinal": 2},
                {"role": "emitted-acceleration", "r95_case_ordinal": 10},
            ]
        },
        "cases": [
            {
                "counterfactual_role": "contact-reserve",
                "r95_case_ordinal": 2,
                "exact_complete_clip_slice_status": "PASS",
            },
            {
                "counterfactual_role": "emitted-acceleration",
                "r95_case_ordinal": 10,
                "exact_complete_clip_slice_status": "PASS",
            },
        ],
    }


def _v9_manifest() -> dict[str, Any]:
    return {
        "prototype_id": "nextengine.humanoid-contact-manifold-prototype.v9",
        "scope": {
            "case_scope": "all",
            "failure_case_count": 7,
            "control_case_count": 10,
            "projection": {
                "trajectory_closure": {
                    "algorithm_id": "nextengine.dimensionless-contact-trajectory-qp.v9"
                }
            },
        },
        "complete_clips": [
            {
                "solve_count": 1,
                "projection_diagnostics": {
                    "status": "PASS",
                    "contact_point_deletion_count": 0,
                },
            }
            for _ in range(3)
        ],
        "cases": [
            {"exact_complete_clip_slice_status": "PASS"} for _ in range(17)
        ],
        "exact_slice_identity": {"status": "PASS", "disagreement_count": 0},
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
