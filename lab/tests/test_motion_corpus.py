from __future__ import annotations

import io
import json
import tempfile
import unittest
import zipfile
from pathlib import Path

import numpy as np

from next_lab.cmu_motion import parse_amc, parse_asf, source_forward_kinematics
from next_lab.motion_corpus import (
    deterministic_npz_bytes,
    load_motion_corpus_profile,
    validate_clip,
)
from next_lab.motion_retarget import (
    CONTACT_IDS,
    SOURCE_OVERLAY_BONES,
    RetargetedClip,
    canonical_integer_arrays,
    mirror_clip,
    rotate_clip_quarter_yaw,
    _close_bilateral_sole_pitch,
    _close_bilateral_sole_roll,
    _conjugate_gradient_trajectory,
    _continuous_stance_anchor_trajectory,
    _continuous_stance_planar_correction,
    _level_stance_soles,
    _lipschitz_majorant,
    _project_joint_velocity,
    _restore_swing_sole_height,
    _stabilize_binary_intervals,
    _transfer_ankle_pitch_to_proximal_chain,
    _weighted_temporal_smooth,
    target_effectors,
    target_forward_kinematics,
)


FIXTURES = Path(__file__).parent / "fixtures"
PROFILES = Path(__file__).parents[1] / "profiles"
CMU_SCALE_METRES = 127 / 2250


class MotionCorpusTests(unittest.TestCase):
    def test_safety_reserve_overlay_is_base_hash_bound(self) -> None:
        profile, profile_bytes = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-safety-reserve.v2.json"
        )
        self.assertTrue(profile_bytes.startswith(b"{"))
        self.assertEqual(
            profile["profile_id"],
            "nextengine.motion-corpus.humanoid-biomechanics-cmu-locomotion-safety-reserve.v2",
        )
        self.assertEqual(
            profile["retarget"]["joint_velocity_limit_basis_points"], 9500
        )
        projection = profile["retarget"]["locomotion_collision_projection"]
        self.assertEqual(projection["knee_minimum_microradians"], 34906)
        self.assertEqual(projection["elbow_minimum_microradians"], 34906)

    def test_joint_velocity_projection_applies_declared_reserve(self) -> None:
        descriptor = {
            "joints": [
                {
                    "dof_ordinal": 0,
                    "maximum_velocity_microradians_per_second": 8_000_000,
                }
            ]
        }
        values = np.asarray([[0], [500_000], [-500_000]], dtype=np.int64)
        projected = _project_joint_velocity(
            values,
            descriptor,
            60,
            velocity_limit_basis_points=9500,
        )
        maximum_step = 8_000_000 * 9500 // 10_000 // 60
        np.testing.assert_array_equal(
            projected,
            np.asarray([[0], [maximum_step], [0]], dtype=np.int64),
        )

    def test_dynamic_reserve_overlay_adds_causal_rom_and_thigh_clearance(self) -> None:
        profile, _ = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-dynamic-reserve.v3.json"
        )
        self.assertEqual(
            profile["retarget"]["joint_velocity_limit_basis_points"], 9000
        )
        projection = profile["retarget"]["locomotion_collision_projection"]
        self.assertEqual(projection["knee_minimum_microradians"], 209440)
        self.assertEqual(projection["elbow_minimum_microradians"], 209440)
        self.assertEqual(projection["hip_roll_minimum_microradians"], 87266)

    def test_temporal_contact_overlay_materializes_new_algorithm_identity(self) -> None:
        profile, _ = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-temporal-contact.v4.json"
        )
        self.assertEqual(
            profile["profile_id"],
            "nextengine.motion-corpus.humanoid-biomechanics-cmu-locomotion-temporal-contact.v4",
        )
        self.assertEqual(
            profile["retarget"]["algorithm_id"],
            "nextengine.cmu-temporal-contact-retarget.v1",
        )
        self.assertEqual(
            profile["retarget"]["joint_velocity_limit_basis_points"], 2500
        )
        self.assertEqual(
            profile["retarget"]["ground_correction"][
                "locomotion_maximum_absolute_micrometres"
            ],
            250000,
        )
        projection = profile["retarget"]["locomotion_collision_projection"]
        self.assertEqual(projection["ankle_roll_minimum_microradians"], -174533)
        self.assertEqual(
            projection["ankle_roll_minimum_hard_reserve_microradians"],
            174533,
        )
        self.assertEqual(
            profile["retarget"]["contact_thresholds"][
                "interval_stabilization"
            ]["minimum_on_frames"],
            3,
        )

    def test_stance_chain_overlay_restores_required_ankle_reserve(self) -> None:
        profile, _ = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-stance-chain.v5.json"
        )
        self.assertEqual(
            profile["retarget"]["algorithm_id"],
            "nextengine.cmu-stance-chain-retarget.v2",
        )
        projection = profile["retarget"]["locomotion_collision_projection"]
        self.assertEqual(projection["ankle_roll_minimum_microradians"], -87266)
        self.assertEqual(projection["ankle_roll_maximum_microradians"], 87266)
        self.assertEqual(
            projection["ankle_roll_minimum_hard_reserve_microradians"],
            261800,
        )
        self.assertEqual(
            profile["retarget"]["temporal_contact_solve"]["stance_chain"][
                "ordered_joint_suffixes"
            ],
            [
                "hip-pitch",
                "hip-roll",
                "knee",
                "ankle-pitch",
                "ankle-roll",
            ],
        )
        self.assertIn(
            {
                "path": "111/111_34.amc",
                "sha256": "9f0b319349b82afdc6febaaec628b9725a6cc8d6369322573efb0a4737fe2f34",
            },
            profile["source_files"],
        )
        clips = {clip["clip_id"]: clip for clip in profile["clips"]}
        self.assertEqual(clips["cmu16-walk-nominal-b"]["subject"], "111")
        self.assertEqual(clips["cmu16-walk-slow"]["trial"], "34")
        self.assertNotIn("cmu139-turn-right-heldout", clips)
        self.assertEqual(
            clips["cmu139-turn-left-heldout"]["derive_mirror_as"],
            "cmu139-turn-right-heldout",
        )

    def test_sole_pitch_closure_overlay_preserves_declared_reserves(self) -> None:
        profile, _ = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-sole-pitch-closure.v6.json"
        )
        self.assertEqual(
            profile["retarget"]["algorithm_id"],
            "nextengine.cmu-stance-chain-retarget.v3",
        )
        solve = profile["retarget"]["temporal_contact_solve"]
        self.assertEqual(
            solve["joint_bounds_microradians"]["joint.left-ankle-pitch"],
            [-349066, 174533],
        )
        self.assertEqual(
            solve["stance_chain"]["final_sole_pitch_smoothing_passes"], 32
        )
        projection = profile["retarget"]["locomotion_collision_projection"]
        self.assertEqual(projection["ankle_pitch_minimum_microradians"], -349066)
        self.assertEqual(
            projection["ankle_roll_minimum_hard_reserve_microradians"],
            261800,
        )

    def test_support_window_closure_overlay_freezes_selected_window(self) -> None:
        profile, _ = load_motion_corpus_profile(
            PROFILES
            / "humanoid-motion-corpus-cmu-support-window-closure.v8.json"
        )
        self.assertEqual(
            profile["retarget"]["algorithm_id"],
            "nextengine.cmu-stance-chain-retarget.v5",
        )
        chain = profile["retarget"]["temporal_contact_solve"][
            "stance_chain"
        ]
        self.assertEqual(
            chain["final_sole_pitch_inverse_joint_weights_q16"],
            [9830, 19661, 65536],
        )
        self.assertEqual(
            chain["final_sole_pitch_support_dilation_frames"], 18
        )
        self.assertEqual(
            profile["retarget"]["locomotion_collision_projection"][
                "ankle_roll_minimum_hard_reserve_microradians"
            ],
            261800,
        )

    def test_intermediate_closure_profiles_remain_hash_materialized(self) -> None:
        expectations = (
            ("humanoid-motion-corpus-cmu-ankle-priority-closure.v7.json", "v4", None),
            ("humanoid-motion-corpus-cmu-swing-clearance-closure.v9.json", "v6", 32),
            ("humanoid-motion-corpus-cmu-swing-clearance-closure.v10.json", "v6", 64),
            ("humanoid-motion-corpus-cmu-swing-clearance-closure.v11.json", "v6", 80),
            ("humanoid-motion-corpus-cmu-sole-normal-closure.v14.json", "v7", 69),
        )
        for filename, algorithm_suffix, swing_smoothing in expectations:
            with self.subTest(profile=filename):
                profile, profile_bytes = load_motion_corpus_profile(
                    PROFILES / filename
                )
                solve = profile["retarget"]["temporal_contact_solve"]
                self.assertEqual(
                    solve["algorithm_id"],
                    f"nextengine.cmu-stance-chain-retarget.{algorithm_suffix}",
                )
                self.assertTrue(profile_bytes.startswith(b"{"))
                if swing_smoothing is not None:
                    self.assertEqual(
                        solve["stance_chain"][
                            "final_swing_clearance_smoothing_passes"
                        ],
                        swing_smoothing,
                    )

    def test_swing_clearance_overlay_freezes_selected_reserves_and_crop(self) -> None:
        profile, _ = load_motion_corpus_profile(
            PROFILES
            / "humanoid-motion-corpus-cmu-swing-clearance-closure.v12.json"
        )
        self.assertEqual(
            profile["retarget"]["algorithm_id"],
            "nextengine.cmu-stance-chain-retarget.v6",
        )
        projection = profile["retarget"]["locomotion_collision_projection"]
        self.assertEqual(projection["ankle_pitch_minimum_microradians"], -523599)
        self.assertEqual(
            projection["ankle_roll_minimum_hard_reserve_microradians"], 261800
        )
        chain = profile["retarget"]["temporal_contact_solve"][
            "stance_chain"
        ]
        self.assertEqual(chain["final_swing_clearance_iterations"], 4)
        self.assertEqual(chain["final_swing_clearance_smoothing_passes"], 69)
        clips = {clip["clip_id"]: clip for clip in profile["clips"]}
        self.assertEqual(
            clips["cmu139-walk-slow-heldout"]["source_last_frame"], 1400
        )

    def test_contact_seated_overlay_closes_declared_ground_gap(self) -> None:
        profile, _ = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-contact-seated.v13.json"
        )
        self.assertEqual(
            profile["profile_id"],
            "nextengine.motion-corpus.humanoid-biomechanics-cmu-locomotion-contact-seated.v13",
        )
        solve = profile["retarget"]["temporal_contact_solve"]
        self.assertEqual(solve["root_height"]["minimum_clearance_micrometres"], 0)
        self.assertEqual(
            solve["validation"][
                "maximum_root_vertical_speed_micrometres_per_second"
            ],
            200060,
        )
        thresholds = profile["retarget"]["contact_thresholds"]
        self.assertEqual(thresholds["sole_height_micrometres"], 45000)
        self.assertEqual(
            thresholds["sole_speed_micrometres_per_second"], 600000
        )
        self.assertEqual(thresholds["recovery_height_micrometres"], 55000)
        self.assertEqual(
            thresholds["recovery_speed_micrometres_per_second"], 600000
        )

    def test_sole_normal_overlay_closes_lateral_tilt_with_frozen_reserves(self) -> None:
        profile, _ = load_motion_corpus_profile(
            PROFILES
            / "humanoid-motion-corpus-cmu-sole-normal-closure-coverage-restored.v15.json"
        )
        self.assertEqual(
            profile["retarget"]["algorithm_id"],
            "nextengine.cmu-stance-chain-retarget.v7",
        )
        solve = profile["retarget"]["temporal_contact_solve"]
        chain = solve["stance_chain"]
        self.assertEqual(chain["final_sole_roll_projection_iterations"], 4)
        self.assertEqual(chain["final_sole_roll_smoothing_passes"], 4)
        self.assertEqual(
            chain["final_sole_roll_inverse_joint_weights_q16"],
            [65536, 6554],
        )
        self.assertEqual(
            solve["joint_bounds_microradians"]["joint.left-hip-roll"][0],
            87266,
        )
        self.assertEqual(
            solve["joint_bounds_microradians"]["joint.left-ankle-roll"],
            [-87266, 87266],
        )
        clips = {clip["clip_id"]: clip for clip in profile["clips"]}
        self.assertEqual(
            clips["cmu91-walk-slow-validation"]["source_last_frame"], 1650
        )

    def test_ankle_pitch_reserve_overlay_freezes_smooth_transfer(self) -> None:
        profile, _ = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-ankle-pitch-reserve.v16.json"
        )
        self.assertEqual(
            profile["retarget"]["algorithm_id"],
            "nextengine.cmu-stance-chain-retarget.v8",
        )
        chain = profile["retarget"]["temporal_contact_solve"][
            "stance_chain"
        ]
        self.assertEqual(
            chain["final_ankle_pitch_minimum_microradians"], -436332
        )
        self.assertEqual(
            chain["final_ankle_pitch_transfer_smoothing_passes"], 32
        )

    def test_stance_anchor_trajectory_is_continuous_and_time_symmetric(self) -> None:
        centers = np.zeros((11, 2, 3), dtype=np.float64)
        centers[:, 0, 0] = np.linspace(0.0, 1.0, 11)
        centers[:, 1, 2] = np.linspace(1.0, 0.0, 11)
        active = np.zeros((11, 2), dtype=np.bool_)
        active[1:4, 0] = True
        active[7:10, 0] = True
        active[2:5, 1] = True
        active[8:11, 1] = True

        anchors = _continuous_stance_anchor_trajectory(
            centers, active, loop=False
        )
        reversed_anchors = _continuous_stance_anchor_trajectory(
            centers[::-1], active[::-1], loop=False
        )

        np.testing.assert_allclose(anchors, reversed_anchors[::-1])
        np.testing.assert_allclose(
            anchors[1:4, 0], np.repeat(anchors[1:2, 0], 3, axis=0)
        )
        np.testing.assert_allclose(
            anchors[7:10, 0], np.repeat(anchors[7:8, 0], 3, axis=0)
        )
        self.assertTrue(
            np.all(np.diff(anchors[4:8, 0, 0]) >= -1.0e-12)
        )

    def test_fixed_iteration_trajectory_solve_closes_spd_system(self) -> None:
        right_hand_side = np.asarray(
            ((2.0, -4.0), (6.0, 8.0)), dtype=np.float64
        )
        result = _conjugate_gradient_trajectory(
            lambda value: 2.0 * value,
            right_hand_side,
            iterations=4,
        )
        np.testing.assert_allclose(result, right_hand_side / 2.0, atol=1.0e-12)

    def test_temporal_smoothing_is_time_and_sign_symmetric(self) -> None:
        values = np.asarray(
            [[0, 20], [100, -40], [-100, 80], [50, -20], [0, 0]],
            dtype=np.int64,
        )
        smoothed = _weighted_temporal_smooth(
            values, kernel=(1, 4, 6, 4, 1), passes=4
        )
        reversed_result = _weighted_temporal_smooth(
            values[::-1], kernel=(1, 4, 6, 4, 1), passes=4
        )
        negated_result = _weighted_temporal_smooth(
            -values, kernel=(1, 4, 6, 4, 1), passes=4
        )
        np.testing.assert_array_equal(reversed_result[::-1], smoothed)
        np.testing.assert_array_equal(negated_result, -smoothed)
        self.assertLess(
            int(np.max(np.abs(np.diff(smoothed, n=2, axis=0)))),
            int(np.max(np.abs(np.diff(values, n=2, axis=0)))),
        )

    def test_root_height_majorant_is_collision_safe_and_speed_bounded(self) -> None:
        required = np.asarray((1.0, 1.0, 1.2, 1.0, 1.0), dtype=np.float64)
        solved = _lipschitz_majorant(required, maximum_step=0.05)
        self.assertTrue(np.all(solved >= required))
        self.assertLessEqual(float(np.max(np.abs(np.diff(solved)))), 0.050000001)
        np.testing.assert_allclose(solved, solved[::-1])

    def test_contact_hysteresis_rejects_short_pulses_and_short_gaps(self) -> None:
        enter = np.asarray(
            (False, True, False, True, True, True, True, False, False, True, True),
            dtype=np.bool_,
        )
        retain = np.asarray(
            (False, True, False, True, True, True, True, False, True, True, True),
            dtype=np.bool_,
        )
        stabilized = _stabilize_binary_intervals(
            enter=enter,
            retain=retain,
            minimum_on_frames=3,
            minimum_off_frames=2,
        )
        np.testing.assert_array_equal(
            stabilized,
            np.asarray((0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1), dtype=np.uint8),
        )

    def test_stance_planar_correction_is_continuous_and_time_symmetric(self) -> None:
        sole_planar = np.zeros((9, 2, 2), dtype=np.float64)
        sole_planar[:, :, 0] = np.asarray(
            (
                (0.00, 1.00),
                (0.002, 1.001),
                (0.006, 1.003),
                (0.014, 1.006),
                (0.024, 1.011),
                (0.032, 1.019),
                (0.038, 1.029),
                (0.042, 1.041),
                (0.044, 1.055),
            )
        )
        support_state = np.asarray((0, 0, 0, 0, 2, 1, 1, 1, 1), dtype=np.int64)
        profile = {
            "kernel_weights": [1, 4, 6, 4, 1],
            "smoothing_passes": 2,
            "endpoint_taper_frames": 3,
            "maximum_absolute_correction_micrometres": 150_000,
            "maximum_speed_micrometres_per_second": 600_000,
        }

        correction = _continuous_stance_planar_correction(
            sole_planar=sole_planar,
            support_state=support_state,
            profile=profile,
        )
        reversed_correction = _continuous_stance_planar_correction(
            sole_planar=sole_planar[::-1],
            support_state=support_state[::-1],
            profile=profile,
        )

        np.testing.assert_allclose(correction, reversed_correction[::-1], atol=1e-12)
        np.testing.assert_array_equal(correction[[0, -1]], np.zeros((2, 2)))
        self.assertGreater(float(np.max(np.linalg.norm(correction, axis=1))), 0.005)
        self.assertLessEqual(
            float(np.max(np.linalg.norm(correction, axis=1))), 0.149999001
        )
        self.assertLess(
            float(np.max(np.linalg.norm(np.diff(correction, axis=0) * 60.0, axis=1))),
            0.600001,
        )

    def test_stance_sole_leveling_cancels_bilateral_foot_tilt(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        joint_by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
        values = np.zeros((1, len(descriptor["joints"])), dtype=np.float64)
        for side in ("left", "right"):
            values[
                0, int(joint_by_id[f"joint.{side}-hip-roll"]["dof_ordinal"])
            ] = 87_266
            values[
                0, int(joint_by_id[f"joint.{side}-knee"]["dof_ordinal"])
            ] = 261_799
        roots = np.asarray(((0.0, 1.0, 0.0),), dtype=np.float64)
        root_rotations = np.eye(3, dtype=np.float64)[None, :, :]

        leveled = _level_stance_soles(
            values=values,
            descriptor=descriptor,
            root_positions=roots,
            root_rotations=root_rotations,
            support_weight=np.ones((1, 2), dtype=np.float64),
            iterations=2,
            probe_microradians=100,
        )

        _, rotations = target_forward_kinematics(
            descriptor,
            roots[0],
            np.asarray((0.0, 0.0, 0.0, 1.0), dtype=np.float64),
            leveled[0] / 1_000_000.0,
        )
        body_by_id = {body["body_id"]: body for body in descriptor["bodies"]}
        for side in ("left", "right"):
            slot = int(body_by_id[f"body.{side}-ankle-roll"]["body_slot"])
            self.assertLess(
                float(np.linalg.norm(rotations[slot][(0, 2), 1])), 3.0e-6
            )
        np.testing.assert_allclose(
            leveled[0, (4, 5)], leveled[0, (10, 11)], atol=1.0e-6
        )

    def test_final_sole_pitch_closure_uses_available_leg_chain_range(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        profile, _ = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-sole-pitch-closure.v6.json"
        )
        solve = profile["retarget"]["temporal_contact_solve"]
        chain = solve["stance_chain"]
        by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
        values = np.zeros((5, len(descriptor["joints"])), dtype=np.int64)
        for side in ("left", "right"):
            values[:, int(by_id[f"joint.{side}-hip-pitch"]["dof_ordinal"])] = 100_000
            values[:, int(by_id[f"joint.{side}-hip-roll"]["dof_ordinal"])] = 87_266
            values[:, int(by_id[f"joint.{side}-knee"]["dof_ordinal"])] = 486_064
            values[:, int(by_id[f"joint.{side}-ankle-pitch"]["dof_ordinal"])] = -349_066
        roots = np.zeros((5, 3), dtype=np.float64)
        roots[:, 1] = 1.0
        root_rotations = np.repeat(np.eye(3)[None, :, :], 5, axis=0)

        closed = _close_bilateral_sole_pitch(
            values=values,
            descriptor=descriptor,
            root_positions=roots,
            root_rotations=root_rotations,
            bounds=solve["joint_bounds_microradians"],
            iterations=int(chain["final_sole_pitch_projection_iterations"]),
            smoothing_passes=2,
            probe_microradians=int(chain["final_sole_pitch_probe_microradians"]),
            maximum_update_microradians=int(
                chain["final_sole_pitch_maximum_update_microradians"]
            ),
            velocity_limit_basis_points=2500,
        )

        for side in ("left", "right"):
            ankle = int(by_id[f"joint.{side}-ankle-pitch"]["dof_ordinal"])
            self.assertGreaterEqual(int(np.min(closed[:, ankle])), -349_066)
            self.assertTrue(
                np.any(
                    closed[
                        :,
                        int(by_id[f"joint.{side}-hip-pitch"]["dof_ordinal"]),
                    ]
                    != values[
                        :,
                        int(by_id[f"joint.{side}-hip-pitch"]["dof_ordinal"]),
                    ]
                )
            )
            positions, rotations = target_forward_kinematics(
                descriptor,
                roots[0],
                np.asarray((0.0, 0.0, 0.0, 1.0), dtype=np.float64),
                closed[0].astype(np.float64) / 1_000_000.0,
            )
            effectors = target_effectors(descriptor, positions, rotations)
            self.assertLess(
                abs(
                    float(effectors[f"effector.{side}-heel"][1])
                    - float(effectors[f"effector.{side}-forefoot"][1])
                ),
                5.0e-6,
            )

    def test_final_sole_roll_closure_uses_bounded_hip_ankle_chain(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        profile, _ = load_motion_corpus_profile(
            PROFILES
            / "humanoid-motion-corpus-cmu-sole-normal-closure-coverage-restored.v15.json"
        )
        solve = profile["retarget"]["temporal_contact_solve"]
        chain = solve["stance_chain"]
        by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
        body_by_id = {body["body_id"]: body for body in descriptor["bodies"]}
        values = np.zeros((7, len(descriptor["joints"])), dtype=np.int64)
        for side in ("left", "right"):
            values[
                :, int(by_id[f"joint.{side}-hip-roll"]["dof_ordinal"])
            ] = 174_533
            values[
                :, int(by_id[f"joint.{side}-knee"]["dof_ordinal"])
            ] = 486_064
        roots = np.zeros((7, 3), dtype=np.float64)
        roots[:, 1] = 1.0
        root_rotations = np.repeat(np.eye(3)[None, :, :], 7, axis=0)

        closed = _close_bilateral_sole_roll(
            values=values,
            descriptor=descriptor,
            root_positions=roots,
            root_rotations=root_rotations,
            bounds=solve["joint_bounds_microradians"],
            iterations=int(chain["final_sole_roll_projection_iterations"]),
            smoothing_passes=0,
            probe_microradians=int(
                chain["final_sole_roll_probe_microradians"]
            ),
            maximum_update_microradians=int(
                chain["final_sole_roll_maximum_update_microradians"]
            ),
            velocity_limit_basis_points=2500,
            inverse_joint_weights_q16=tuple(
                chain["final_sole_roll_inverse_joint_weights_q16"]
            ),
        )

        for side in ("left", "right"):
            hip = int(by_id[f"joint.{side}-hip-roll"]["dof_ordinal"])
            ankle = int(by_id[f"joint.{side}-ankle-roll"]["dof_ordinal"])
            self.assertGreaterEqual(int(np.min(closed[:, hip])), 87266)
            self.assertGreaterEqual(int(np.min(closed[:, ankle])), -87266)
            self.assertLessEqual(int(np.max(closed[:, ankle])), 87266)
            _, rotations = target_forward_kinematics(
                descriptor,
                roots[3],
                np.asarray((0.0, 0.0, 0.0, 1.0), dtype=np.float64),
                closed[3].astype(np.float64) / 1_000_000.0,
            )
            slot = int(body_by_id[f"body.{side}-ankle-roll"]["body_slot"])
            self.assertLess(abs(float(rotations[slot][0, 1])), 5.0e-6)
            self.assertTrue(
                np.any(closed[:, (hip, ankle)] != values[:, (hip, ankle)])
            )

    def test_ankle_pitch_reserve_transfer_preserves_chain_rotation(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        profile, _ = load_motion_corpus_profile(
            PROFILES
            / "humanoid-motion-corpus-cmu-ankle-pitch-reserve.v16.json"
        )
        bounds = profile["retarget"]["temporal_contact_solve"][
            "joint_bounds_microradians"
        ]
        by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
        values = np.zeros((5, len(descriptor["joints"])), dtype=np.int64)
        for side in ("left", "right"):
            values[
                :, int(by_id[f"joint.{side}-hip-pitch"]["dof_ordinal"])
            ] = -100_000
            values[
                :, int(by_id[f"joint.{side}-hip-roll"]["dof_ordinal"])
            ] = 87_266
            values[
                :, int(by_id[f"joint.{side}-knee"]["dof_ordinal"])
            ] = 600_000
            values[
                :, int(by_id[f"joint.{side}-ankle-pitch"]["dof_ordinal"])
            ] = -523_599

        transferred = _transfer_ankle_pitch_to_proximal_chain(
            values=values,
            descriptor=descriptor,
            bounds=bounds,
            ankle_pitch_minimum_microradians=-436_332,
            smoothing_kernel=(1, 4, 6, 4, 1),
            smoothing_passes=32,
        )

        selected: list[int] = []
        for side in ("left", "right"):
            ordinals = [
                int(by_id[f"joint.{side}-{suffix}"]["dof_ordinal"])
                for suffix in ("hip-pitch", "knee", "ankle-pitch")
            ]
            selected.extend(ordinals)
            np.testing.assert_array_equal(
                np.sum(transferred[:, ordinals], axis=1),
                np.sum(values[:, ordinals], axis=1),
            )
            ankle = ordinals[2]
            self.assertEqual(int(np.min(transferred[:, ankle])), -436_332)
            for ordinal, suffix in zip(
                ordinals, ("hip-pitch", "knee", "ankle-pitch"), strict=True
            ):
                minimum, maximum = bounds[f"joint.{side}-{suffix}"]
                self.assertGreaterEqual(int(np.min(transferred[:, ordinal])), minimum)
                self.assertLessEqual(int(np.max(transferred[:, ordinal])), maximum)
        untouched = sorted(set(range(values.shape[1])) - set(selected))
        np.testing.assert_array_equal(
            transferred[:, untouched], values[:, untouched]
        )

    def test_ankle_pitch_reserve_transfer_smooths_required_envelope(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        profile, _ = load_motion_corpus_profile(
            PROFILES / "humanoid-motion-corpus-cmu-ankle-pitch-reserve.v16.json"
        )
        bounds = profile["retarget"]["temporal_contact_solve"][
            "joint_bounds_microradians"
        ]
        by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
        values = np.zeros((9, len(descriptor["joints"])), dtype=np.int64)
        ankle_input = np.asarray(
            (
                -400_000,
                -410_000,
                -430_000,
                -470_000,
                -523_599,
                -470_000,
                -430_000,
                -410_000,
                -400_000,
            ),
            dtype=np.int64,
        )
        ankle_ordinals: list[int] = []
        for side in ("left", "right"):
            values[
                :, int(by_id[f"joint.{side}-hip-pitch"]["dof_ordinal"])
            ] = -100_000
            values[
                :, int(by_id[f"joint.{side}-knee"]["dof_ordinal"])
            ] = 600_000
            ankle = int(
                by_id[f"joint.{side}-ankle-pitch"]["dof_ordinal"]
            )
            values[:, ankle] = ankle_input
            ankle_ordinals.append(ankle)

        transferred = _transfer_ankle_pitch_to_proximal_chain(
            values=values,
            descriptor=descriptor,
            bounds=bounds,
            ankle_pitch_minimum_microradians=-436_332,
            smoothing_kernel=(1, 4, 6, 4, 1),
            smoothing_passes=32,
        )
        pointwise_transfer = np.maximum(-436_332 - ankle_input, 0)
        pointwise_transfer_acceleration = int(
            np.max(np.abs(np.diff(pointwise_transfer, n=2)))
        )
        for ankle in ankle_ordinals:
            trajectory = transferred[:, ankle]
            self.assertGreaterEqual(int(np.min(trajectory)), -436_332)
            transfer = trajectory - ankle_input
            self.assertLess(
                int(np.max(np.abs(np.diff(transfer, n=2)))),
                pointwise_transfer_acceleration,
            )

    def test_final_sole_pitch_closure_honors_projection_mask(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        profile, _ = load_motion_corpus_profile(
            PROFILES
            / "humanoid-motion-corpus-cmu-support-window-closure.v8.json"
        )
        solve = profile["retarget"]["temporal_contact_solve"]
        chain = solve["stance_chain"]
        by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
        values = np.zeros((5, len(descriptor["joints"])), dtype=np.int64)
        for side in ("left", "right"):
            values[
                :, int(by_id[f"joint.{side}-hip-pitch"]["dof_ordinal"])
            ] = 100_000
            values[
                :, int(by_id[f"joint.{side}-hip-roll"]["dof_ordinal"])
            ] = 87_266
            values[
                :, int(by_id[f"joint.{side}-knee"]["dof_ordinal"])
            ] = 486_064
            values[
                :, int(by_id[f"joint.{side}-ankle-pitch"]["dof_ordinal"])
            ] = -349_066
        roots = np.zeros((5, 3), dtype=np.float64)
        roots[:, 1] = 1.0
        root_rotations = np.repeat(np.eye(3)[None, :, :], 5, axis=0)
        projection_mask = np.zeros((5, 2), dtype=np.bool_)
        projection_mask[2, 0] = True

        closed = _close_bilateral_sole_pitch(
            values=values,
            descriptor=descriptor,
            root_positions=roots,
            root_rotations=root_rotations,
            bounds=solve["joint_bounds_microradians"],
            iterations=1,
            smoothing_passes=0,
            probe_microradians=int(chain["final_sole_pitch_probe_microradians"]),
            maximum_update_microradians=int(
                chain["final_sole_pitch_maximum_update_microradians"]
            ),
            velocity_limit_basis_points=10_000,
            inverse_joint_weights_q16=tuple(
                chain["final_sole_pitch_inverse_joint_weights_q16"]
            ),
            projection_mask=projection_mask,
        )

        left_ordinals = [
            int(by_id[f"joint.left-{suffix}"]["dof_ordinal"])
            for suffix in ("hip-pitch", "knee", "ankle-pitch")
        ]
        right_ordinals = [
            int(by_id[f"joint.right-{suffix}"]["dof_ordinal"])
            for suffix in ("hip-pitch", "knee", "ankle-pitch")
        ]
        self.assertTrue(np.any(closed[2, left_ordinals] != values[2, left_ordinals]))
        np.testing.assert_array_equal(
            closed[[0, 1, 3, 4]][:, left_ordinals],
            values[[0, 1, 3, 4]][:, left_ordinals],
        )
        np.testing.assert_array_equal(
            closed[:, right_ordinals], values[:, right_ordinals]
        )

    def test_swing_height_restore_uses_pitch_null_space_and_mask(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(
                encoding="utf-8"
            )
        )
        profile, _ = load_motion_corpus_profile(
            PROFILES
            / "humanoid-motion-corpus-cmu-swing-clearance-closure.v12.json"
        )
        solve = profile["retarget"]["temporal_contact_solve"]
        by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
        values = np.zeros((7, len(descriptor["joints"])), dtype=np.int64)
        for side in ("left", "right"):
            for suffix, value in (
                ("hip-pitch", 100_000),
                ("hip-roll", 87_266),
                ("knee", 486_064),
                ("ankle-pitch", -349_066),
            ):
                ordinal = int(by_id[f"joint.{side}-{suffix}"]["dof_ordinal"])
                values[:, ordinal] = value
        roots = np.zeros((7, 3), dtype=np.float64)
        roots[:, 1] = 1.0
        root_rotations = np.repeat(np.eye(3)[None, :, :], 7, axis=0)
        closed = _close_bilateral_sole_pitch(
            values=values,
            descriptor=descriptor,
            root_positions=roots,
            root_rotations=root_rotations,
            bounds=solve["joint_bounds_microradians"],
            iterations=4,
            smoothing_passes=0,
            probe_microradians=100,
            maximum_update_microradians=150_000,
            velocity_limit_basis_points=2_500,
            inverse_joint_weights_q16=(6_554, 13_107, 65_536),
        )
        restore_mask = np.zeros((7, 2), dtype=np.bool_)
        restore_mask[:, 0] = True
        restored = _restore_swing_sole_height(
            target_height_values=values,
            values=closed,
            descriptor=descriptor,
            root_positions=roots,
            root_rotations=root_rotations,
            bounds=solve["joint_bounds_microradians"],
            restore_mask=restore_mask,
            iterations=4,
            smoothing_passes=0,
            probe_microradians=100,
            maximum_update_microradians=150_000,
            velocity_limit_basis_points=2_500,
        )

        def sole_height_and_pitch(trajectory: np.ndarray) -> tuple[float, float]:
            positions, rotations = target_forward_kinematics(
                descriptor,
                roots[3],
                np.asarray((0.0, 0.0, 0.0, 1.0), dtype=np.float64),
                trajectory[3].astype(np.float64) / 1_000_000.0,
            )
            effectors = target_effectors(descriptor, positions, rotations)
            heel = float(effectors["effector.left-heel"][1])
            forefoot = float(effectors["effector.left-forefoot"][1])
            return (heel + forefoot) / 2.0, heel - forefoot

        closed_height, _ = sole_height_and_pitch(closed)
        restored_height, restored_pitch = sole_height_and_pitch(restored)
        self.assertGreater(restored_height, closed_height + 0.005)
        self.assertLess(abs(restored_pitch), 0.015)
        right_ordinals = [
            int(by_id[f"joint.right-{suffix}"]["dof_ordinal"])
            for suffix in ("hip-pitch", "knee", "ankle-pitch")
        ]
        np.testing.assert_array_equal(
            restored[:, right_ordinals], closed[:, right_ordinals]
        )
        for suffix in ("hip-pitch", "knee", "ankle-pitch"):
            ordinal = int(by_id[f"joint.left-{suffix}"]["dof_ordinal"])
            minimum, maximum = solve["joint_bounds_microradians"][
                f"joint.left-{suffix}"
            ]
            self.assertGreaterEqual(int(np.min(restored[:, ordinal])), minimum)
            self.assertLessEqual(int(np.max(restored[:, ordinal])), maximum)

    def test_cmu_parser_applies_frozen_units_handedness_and_rate_boundary(self) -> None:
        asf = """
:version 1.10
:units
length 0.45
angle deg
:root
order TX TY TZ RX RY RZ
axis XYZ
position 0 0 0
orientation 0 0 0
:bonedata
begin
id 1
name femur
direction 1 0 0
length 1
axis 0 0 0 XYZ
dof rx
end
:hierarchy
begin
root femur
end
""".strip()
        amc = """
:FULLY-SPECIFIED
:DEGREES
1
root 1 2 3 0 0 0
femur 0
2
root 2 2 3 0 0 0
femur 0
""".strip()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            asf_path = root / "fixture.asf"
            amc_path = root / "fixture.amc"
            asf_path.write_text(asf, encoding="utf-8")
            amc_path.write_text(amc, encoding="utf-8")
            skeleton = parse_asf(
                asf_path,
                expected_length_scale_metres=CMU_SCALE_METRES,
            )
            frames = parse_amc(amc_path)

        self.assertEqual([frame.source_frame for frame in frames], [1, 2])
        pose = source_forward_kinematics(skeleton, frames[0])
        np.testing.assert_allclose(
            pose.root_position,
            np.asarray((-CMU_SCALE_METRES, 2 * CMU_SCALE_METRES, 3 * CMU_SCALE_METRES)),
        )
        np.testing.assert_allclose(
            pose.bone_ends["femur"],
            pose.root_position + np.asarray((-CMU_SCALE_METRES, 0.0, 0.0)),
        )

    def test_cmu_parser_rejects_noncontiguous_frames_and_wrong_scale(self) -> None:
        asf = """
:units
length 0.45
:root
order TX TY TZ RX RY RZ
axis XYZ
:bonedata
begin
name bone
direction 0 1 0
length 1
axis 0 0 0 XYZ
end
:hierarchy
begin
root bone
end
""".strip()
        amc = "1\nroot 0 0 0 0 0 0\n3\nroot 0 0 0 0 0 0\n"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            asf_path = root / "fixture.asf"
            amc_path = root / "fixture.amc"
            asf_path.write_text(asf, encoding="utf-8")
            amc_path.write_text(amc, encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "length scale"):
                parse_asf(asf_path, expected_length_scale_metres=1.0)
            with self.assertRaisesRegex(ValueError, "not contiguous"):
                parse_amc(amc_path)

    def test_mirror_twice_restores_every_canonical_integer_array(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(encoding="utf-8")
        )
        original = _clip(descriptor)
        mirrored = mirror_clip(original, descriptor, "clip.start-left")
        np.testing.assert_array_equal(
            mirrored.stance_support_state,
            np.asarray((1, 2, 0), dtype=np.int64),
        )
        restored = mirror_clip(mirrored, descriptor, original.clip_id)
        for name, expected in canonical_integer_arrays(original).items():
            np.testing.assert_array_equal(canonical_integer_arrays(restored)[name], expected)

    def test_yaw_variant_rotates_world_vectors_and_preserves_group(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(encoding="utf-8")
        )
        original = _clip(descriptor)
        rotated = rotate_clip_quarter_yaw(
            original,
            clip_id="clip.fall-left",
            skill="brace_safe_fall_left",
            quarter_turns=1,
        )
        expected_root = original.root_position_um.copy()
        expected_root[:, 0] = original.root_position_um[:, 2]
        expected_root[:, 2] = -original.root_position_um[:, 0]
        np.testing.assert_array_equal(rotated.root_position_um, expected_root)
        np.testing.assert_array_equal(rotated.contacts, original.contacts)
        np.testing.assert_array_equal(
            rotated.joint_position_urad, original.joint_position_urad
        )
        self.assertEqual(rotated.split_group_id, original.split_group_id)
        self.assertEqual(rotated.derived_from, original.clip_id)
        self.assertEqual(rotated.derivation, "world-yaw-quarter-turns:1")

    def test_npz_serialization_is_byte_exact_and_has_fixed_metadata(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(encoding="utf-8")
        )
        clip = _clip(descriptor)
        metadata = {"schema_version": 1, "clip_id": clip.clip_id}
        first = deterministic_npz_bytes(clip, metadata)
        self.assertEqual(first, deterministic_npz_bytes(clip, metadata))
        with zipfile.ZipFile(io.BytesIO(first)) as archive:
            self.assertTrue(all(item.date_time == (1980, 1, 1, 0, 0, 0) for item in archive.infolist()))
            self.assertEqual(
                sorted(item.filename for item in archive.infolist()),
                sorted(f"{name}.npy" for name in (*canonical_integer_arrays(clip), "metadata_json_utf8")),
            )

    def test_locomotion_ankle_roll_audit_blocks_soft_boundary_saturation(self) -> None:
        descriptor = json.loads(
            (FIXTURES / "biomechanics_motor_mirror_v1.json").read_text(encoding="utf-8")
        )
        profile = json.loads(
            (
                Path(__file__).parents[1]
                / "profiles/humanoid-motion-corpus-cmu.v1.json"
            ).read_text(encoding="utf-8")
        )
        clip = _clip(descriptor)
        clip.joint_position_urad.fill(0)
        clip.joint_velocity_urad_s.fill(0)
        ankle_roll_ordinals = [
            int(joint["dof_ordinal"])
            for joint in descriptor["joints"]
            if joint["joint_id"]
            in {"joint.left-ankle-roll", "joint.right-ankle-roll"}
        ]
        clip.joint_position_urad[:, ankle_roll_ordinals] = 261799

        saturated = validate_clip(clip, descriptor, profile)

        self.assertIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_SOFT_BOUNDARY_SATURATION",
            saturated["errors"],
        )
        self.assertIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_HARD_RESERVE_SHORTFALL",
            saturated["errors"],
        )
        self.assertEqual(saturated["metrics"]["ankle_roll_soft_boundary_fraction"], 1.0)
        self.assertEqual(
            saturated["metrics"]["minimum_ankle_roll_hard_reserve_microradians"],
            87267,
        )

        clip.joint_position_urad[:, ankle_roll_ordinals] = 87266
        unidirectional_ordinals = [
            int(joint["dof_ordinal"])
            for joint in descriptor["joints"]
            if joint["joint_id"].endswith(("-knee", "-elbow"))
        ]
        clip.joint_position_urad[:, unidirectional_ordinals] = 17453
        reserved = validate_clip(clip, descriptor, profile)

        self.assertNotIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_SOFT_BOUNDARY_SATURATION",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_HARD_RESERVE_SHORTFALL",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_ANKLE_ROLL_PROJECTION_MISMATCH",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_UNIDIRECTIONAL_LOWER_SOFT_BOUNDARY_SATURATION",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_UNIDIRECTIONAL_HARD_RESERVE_SHORTFALL",
            reserved["errors"],
        )
        self.assertNotIn(
            "RETARGET_LOCOMOTION_UNIDIRECTIONAL_PROJECTION_MISMATCH",
            reserved["errors"],
        )
        self.assertEqual(reserved["metrics"]["ankle_roll_soft_boundary_fraction"], 0.0)
        self.assertEqual(
            reserved["metrics"]["minimum_ankle_roll_hard_reserve_microradians"],
            261800,
        )
        self.assertEqual(
            reserved["metrics"][
                "unidirectional_joint_lower_soft_boundary_fraction"
            ],
            0.0,
        )
        self.assertEqual(
            reserved["metrics"][
                "minimum_unidirectional_joint_hard_reserve_microradians"
            ],
            17453,
        )


def _clip(descriptor: dict) -> RetargetedClip:
    frame_count = 3
    joint_count = len(descriptor["joints"])
    effector_ids = tuple(sorted(item["effector_id"] for item in descriptor["effectors"]))
    effector_count = len(effector_ids)
    joint_position = np.arange(frame_count * joint_count, dtype=np.int64).reshape(
        frame_count, joint_count
    )
    root_quaternion = np.zeros((frame_count, 4), dtype=np.int64)
    root_quaternion[:, 3] = 1 << 30
    return RetargetedClip(
        clip_id="clip.start-right",
        source_clip_id="cmu-104-14",
        skill="start_right_lead",
        partition="locomotion",
        split="train",
        split_group_id="cmu-subject-104-trial-14",
        source_frames=np.asarray((1, 3, 5), dtype=np.int64),
        root_position_um=np.arange(frame_count * 3, dtype=np.int64).reshape(frame_count, 3),
        root_quaternion_q1_30=root_quaternion,
        root_linear_velocity_um_s=np.arange(frame_count * 3, dtype=np.int64).reshape(
            frame_count, 3
        ),
        root_yaw_urad=np.asarray((1, 2, 3), dtype=np.int64),
        root_yaw_velocity_urad_s=np.asarray((4, 5, 6), dtype=np.int64),
        joint_position_urad=joint_position,
        joint_velocity_urad_s=joint_position * 60,
        center_of_mass_um=np.arange(frame_count * 3, dtype=np.int64).reshape(frame_count, 3),
        effector_ids=effector_ids,
        effector_position_um=np.arange(
            frame_count * effector_count * 3, dtype=np.int64
        ).reshape(frame_count, effector_count, 3),
        contacts=np.arange(frame_count * len(CONTACT_IDS), dtype=np.uint8).reshape(
            frame_count, len(CONTACT_IDS)
        ),
        phase_u16=np.asarray((0, 32768, 65535), dtype=np.uint16),
        ground_correction_um=np.asarray((1, -2, 3), dtype=np.int64),
        minimum_collider_height_um=np.asarray((0, 1, 2), dtype=np.int64),
        minimum_nonfoot_height_um=np.asarray((3, 4, 5), dtype=np.int64),
        raw_soft_rom_excess_urad=joint_position + 100,
        locomotion_collision_projection_urad=joint_position + 200,
        velocity_projection_urad=joint_position,
        source_overlay_bones=SOURCE_OVERLAY_BONES,
        source_overlay_position_um=np.arange(
            frame_count * len(SOURCE_OVERLAY_BONES) * 3, dtype=np.int64
        ).reshape(frame_count, len(SOURCE_OVERLAY_BONES), 3),
        loop=False,
        planar_root_correction_um=np.asarray(
            ((1, 0, 2), (3, 0, 4), (5, 0, 6)), dtype=np.int64
        ),
        stance_support_state=np.asarray((0, 2, 1), dtype=np.int64),
    )


if __name__ == "__main__":
    unittest.main()
