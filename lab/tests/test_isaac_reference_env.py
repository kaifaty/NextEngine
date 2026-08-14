from __future__ import annotations

import unittest
from pathlib import Path

import torch
from next_lab.isaac_env import (
    engine_quaternion_xyzw_from_isaac_wxyz_tensor,
    engine_vector_from_isaac_tensor,
)
from next_lab.isaac_reference_env import (
    OBSERVED_HARD_ROM_TOLERANCE_MICRORADIANS,
    _advance_contact_grace,
    _build_exhaustive_phase_schedule,
    _canonical_pd_requested_effort_tensor,
    _classify_contact_pairs_tensor,
    _contact_body_projections,
    _contact_impact_margin_cost_tensor,
    _contact_impulse_magnitude_micronewton_seconds,
    _contact_pair_layout,
    _engine_to_isaac_vector,
    _engine_xyzw_to_isaac_wxyz,
    _ground_usd_path_from_humanoid,
    _intersect_effort_limits_tensor,
    _minimum_contact_separation_tensor,
    _normalized_xyzw,
    _quaternion_conjugate_xyzw,
    _quaternion_multiply_xyzw,
    _rotate_inverse_xyzw,
    _select_curriculum_episode,
    _soft_rom_excursion_cost_tensor,
    _terminal_reason_tensor,
)
from next_lab.motor_mirror import load_json, round_div_ties_even
from next_lab.safety_contact_mirror import _intersect_effort, _positive_work_charge


class IsaacReferenceEnvironmentTests(unittest.TestCase):
    def test_ground_usd_is_derived_from_same_translation_bundle(self) -> None:
        self.assertEqual(
            _ground_usd_path_from_humanoid("/store/current/humanoid.usda"),
            "/store/current/ground.usda",
        )
        self.assertEqual(_ground_usd_path_from_humanoid(""), "")

    def test_contact_impact_margin_cost_takes_worst_pair(self) -> None:
        actual = _contact_impact_margin_cost_tensor(
            torch.tensor(
                [[5_699_999, 5_850_000], [6_000_000, 6_600_000]],
                dtype=torch.int64,
            ),
            torch.tensor([6_000_000, 6_000_000], dtype=torch.int64),
            9_500,
        )
        torch.testing.assert_close(
            actual, torch.tensor([0.5, 1.0], dtype=torch.float64)
        )

    def test_contact_impulse_diagnostic_uses_vector_magnitude(self) -> None:
        impulse = torch.tensor(
            [[[3_000_000, 4_000_000, 0], [0, 0, 0]]], dtype=torch.int64
        )
        torch.testing.assert_close(
            _contact_impulse_magnitude_micronewton_seconds(impulse),
            torch.tensor([[5_000_000, 0]], dtype=torch.int64),
        )

    def test_contact_body_projection_is_strict_and_pair_complete(self) -> None:
        descriptor = load_json(
            Path(__file__).parent / "fixtures" / "biomechanics_motor_mirror_v1.json"
        )
        projections = _contact_body_projections(descriptor)
        by_body = {body_id: (role, limit) for body_id, role, limit in projections}
        self.assertEqual(len(projections), 14)
        self.assertEqual(by_body["body.torso-yaw"], (3, 1_000_000))
        self.assertEqual(by_body["body.left-knee"], (5, 4_000_000))
        self.assertEqual(by_body["body.right-elbow"], (10, 3_000_000))
        layout = _contact_pair_layout(projections)
        self.assertEqual(len(layout["sensor"]), 14 + 14 * 13 // 2)
        self.assertEqual(sum(bool(value) for value in layout["self_contact"]), 91)
        self.assertEqual(layout["pair_id"][0], "ground:body.pelvis")
        self.assertEqual(len(layout["pair_id"]), len(layout["sensor"]))

    def test_contact_pair_classifier_has_exact_grace_and_strict_limits(self) -> None:
        impulse = torch.tensor(
            [[[100_000, 0, 0], [1_000_001, 0, 0], [300_000, 0, 0]]],
            dtype=torch.int64,
        )
        separation = torch.zeros((1, 3), dtype=torch.int64)
        continuity = torch.tensor([[4, 0, 0]], dtype=torch.int64)
        result = _classify_contact_pairs_tensor(
            impulse,
            separation,
            continuity,
            torch.tensor([3_000_000, 1_000_000, 3_000_000], dtype=torch.int64),
            torch.tensor([False, False, True]),
            torch.tensor([10, 3, 10], dtype=torch.int64),
        )
        torch.testing.assert_close(result["continuity"], torch.tensor([[5, 1, 1]]))
        torch.testing.assert_close(
            result["material"], torch.tensor([[True, True, True]])
        )
        torch.testing.assert_close(
            result["hard_impact"], torch.tensor([[False, True, False]])
        )
        torch.testing.assert_close(
            result["self_collision"], torch.tensor([[False, False, True]])
        )
        torch.testing.assert_close(
            result["forbidden_locomotion"], torch.tensor([[True, True, False]])
        )

    def test_contact_detail_reduction_retains_pair_minimum_separation(self) -> None:
        separation = torch.tensor(
            [[0.002], [-0.003], [0.004], [-0.000_007]], dtype=torch.float32
        )
        count = torch.tensor([[2, 0], [1, 1]], dtype=torch.int32)
        start = torch.tensor([[0, 2], [2, 3]], dtype=torch.int32)
        actual = _minimum_contact_separation_tensor(separation, count, start)
        self.assertEqual(actual[0, 0].item(), -3_000)
        self.assertEqual(actual[1, 0].item(), 4_000)
        self.assertEqual(actual[1, 1].item(), -7)
        self.assertEqual(actual[0, 1].item(), torch.iinfo(torch.int64).max)

    def test_terminal_reason_reducer_is_exhaustive_and_priority_stable(self) -> None:
        false = torch.zeros(11, dtype=torch.bool)
        branches = {
            name: false.clone()
            for name in (
                "non_finite",
                "hard_rom",
                "joint_safety",
                "hard_impact",
                "self_collision",
                "forbidden_contact",
                "world_bounds",
                "fall",
                "tracking_lost",
                "success",
            )
        }
        branches["non_finite"][0] = True
        branches["hard_rom"][1] = True
        branches["joint_safety"][2] = True
        branches["hard_impact"][3] = True
        branches["self_collision"][4] = True
        branches["forbidden_contact"][5] = True
        branches["world_bounds"][6] = True
        branches["fall"][7] = True
        branches["tracking_lost"][8] = True
        branches["success"][9] = True
        for value in branches.values():
            value[10] = True
        torch.testing.assert_close(
            _terminal_reason_tensor(**branches),
            torch.tensor([1, 2, 2, 3, 4, 5, 6, 7, 9, 10, 1]),
        )

    def test_observed_hard_rom_tolerance_matches_engine_contract(self) -> None:
        self.assertEqual(OBSERVED_HARD_ROM_TOLERANCE_MICRORADIANS, 10)

    def test_invalid_quaternion_still_fails_synchronously_on_cpu(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "invalid reference quaternion"):
            _normalized_xyzw(torch.zeros((1, 4)))

    def test_engine_isaac_vector_and_quaternion_mappings_are_inverses(self) -> None:
        vector = torch.tensor([[1.25, -2.5, 3.75]], dtype=torch.float64)
        torch.testing.assert_close(
            engine_vector_from_isaac_tensor(_engine_to_isaac_vector(vector)), vector
        )
        quaternion = torch.tensor([[0.1, -0.2, 0.3, 0.92736185]], dtype=torch.float64)
        quaternion /= torch.linalg.vector_norm(quaternion, dim=-1, keepdim=True)
        torch.testing.assert_close(
            engine_quaternion_xyzw_from_isaac_wxyz_tensor(
                _engine_xyzw_to_isaac_wxyz(quaternion)
            ),
            quaternion,
        )

    def test_root_inverse_rotation_and_relative_identity_are_closed(self) -> None:
        half = 2.0**-0.5
        yaw_quaternion = torch.tensor([[0.0, half, 0.0, half]], dtype=torch.float64)
        forward_world = torch.tensor([[1.0, 0.0, 0.0]], dtype=torch.float64)
        local = _rotate_inverse_xyzw(forward_world, yaw_quaternion)
        torch.testing.assert_close(
            local,
            torch.tensor([[0.0, 0.0, 1.0]], dtype=torch.float64),
            atol=1.0e-12,
            rtol=0.0,
        )
        identity = _quaternion_multiply_xyzw(
            _quaternion_conjugate_xyzw(yaw_quaternion), yaw_quaternion
        )
        torch.testing.assert_close(
            identity,
            torch.tensor([[0.0, 0.0, 0.0, 1.0]], dtype=torch.float64),
            atol=1.0e-12,
            rtol=0.0,
        )

    def test_curriculum_episode_selection_is_deterministic_and_bounded(self) -> None:
        root = bytes.fromhex("12" * 32)
        selections = [
            _select_curriculum_episode(
                run_root=root,
                episode_ordinal=episode,
                vector_slot=slot,
                clip_frame_counts=(90, 120, 180),
                horizon_motor_ticks=32,
            )
            for episode in range(3)
            for slot in range(8)
        ]
        self.assertEqual(
            selections[0],
            _select_curriculum_episode(
                run_root=root,
                episode_ordinal=0,
                vector_slot=0,
                clip_frame_counts=(90, 120, 180),
                horizon_motor_ticks=32,
            ),
        )
        self.assertGreater(len(set(selections)), 1)
        for clip_index, start_frame, terminal_frame in selections:
            self.assertIn(clip_index, range(3))
            self.assertGreaterEqual(start_frame, 0)
            self.assertEqual(terminal_frame - start_frame, 32)
            self.assertLess(terminal_frame, (90, 120, 180)[clip_index])

    def test_curriculum_episode_selection_honors_phase_prefix(self) -> None:
        root = bytes.fromhex("34" * 32)
        for phase_prefix_count in (42, 84, 126, 169):
            selections = [
                _select_curriculum_episode(
                    run_root=root,
                    episode_ordinal=episode,
                    vector_slot=slot,
                    clip_frame_counts=(180,),
                    horizon_motor_ticks=11,
                    phase_prefix_count=phase_prefix_count,
                )
                for episode in range(32)
                for slot in range(64)
            ]
            self.assertTrue(
                all(
                    start_frame < phase_prefix_count for _, start_frame, _ in selections
                )
            )
            self.assertEqual(
                {start_frame for _, start_frame, _ in selections},
                set(range(phase_prefix_count)),
            )

    def test_exhaustive_phase_schedule_covers_every_clip_phase_and_repeat(self) -> None:
        schedule = _build_exhaustive_phase_schedule(
            clip_frame_counts=(14, 16),
            horizon_motor_ticks=11,
            repeats=2,
        )
        self.assertEqual(
            schedule,
            (
                (0, 0, 11, 0),
                (0, 0, 11, 1),
                (0, 1, 12, 0),
                (0, 1, 12, 1),
                (0, 2, 13, 0),
                (0, 2, 13, 1),
                (1, 0, 11, 0),
                (1, 0, 11, 1),
                (1, 1, 12, 0),
                (1, 1, 12, 1),
                (1, 2, 13, 0),
                (1, 2, 13, 1),
                (1, 3, 14, 0),
                (1, 3, 14, 1),
                (1, 4, 15, 0),
                (1, 4, 15, 1),
            ),
        )

    def test_exhaustive_phase_schedule_rejects_incomplete_bounds(self) -> None:
        for arguments in (
            {"clip_frame_counts": (), "horizon_motor_ticks": 11, "repeats": 1},
            {
                "clip_frame_counts": (11,),
                "horizon_motor_ticks": 11,
                "repeats": 1,
            },
            {
                "clip_frame_counts": (12,),
                "horizon_motor_ticks": 0,
                "repeats": 1,
            },
            {
                "clip_frame_counts": (12,),
                "horizon_motor_ticks": 11,
                "repeats": 0,
            },
            {
                "clip_frame_counts": (12,),
                "horizon_motor_ticks": 11,
                "repeats": True,
            },
        ):
            with (
                self.subTest(arguments=arguments),
                self.assertRaisesRegex(
                    ValueError, "invalid exhaustive reference phase schedule"
                ),
            ):
                _build_exhaustive_phase_schedule(**arguments)

    def test_forbidden_contact_grace_expires_after_declared_physics_substeps(
        self,
    ) -> None:
        accumulated = torch.zeros(3, dtype=torch.int64)
        raw = torch.tensor([True, False, True])
        accumulated, terminal = _advance_contact_grace(
            accumulated,
            raw,
            physics_substeps_per_motor_tick=4,
            grace_physics_substeps=4,
        )
        torch.testing.assert_close(accumulated, torch.tensor([4, 0, 4]))
        self.assertFalse(torch.any(terminal))
        accumulated, terminal = _advance_contact_grace(
            accumulated,
            torch.tensor([True, True, False]),
            physics_substeps_per_motor_tick=4,
            grace_physics_substeps=4,
        )
        torch.testing.assert_close(accumulated, torch.tensor([8, 4, 0]))
        torch.testing.assert_close(terminal, torch.tensor([True, False, False]))

    def test_soft_rom_excursion_cost_warns_only_between_soft_and_hard_rom(
        self,
    ) -> None:
        soft_minimum = torch.tensor([-5.0, 0.0])
        soft_maximum = torch.tensor([5.0, 5.0])
        hard_minimum = torch.tensor([-10.0, 0.0])
        hard_maximum = torch.tensor([10.0, 10.0])
        cost = _soft_rom_excursion_cost_tensor(
            torch.tensor([[0.0, 0.0], [7.5, -1.0], [12.0, 7.5]]),
            soft_minimum,
            soft_maximum,
            hard_minimum,
            hard_maximum,
        )
        torch.testing.assert_close(cost, torch.tensor([0.0, 0.5, 1.0]))

    def test_canonical_pd_rounding_matches_engine_integer_arithmetic(self) -> None:
        target = torch.tensor([[150_001.0, -25_000.0]], dtype=torch.float64)
        position = torch.tensor([[-87_266.0, 75_001.0]], dtype=torch.float64)
        velocity = torch.tensor([[-1_750_163.0, 562_311.0]], dtype=torch.float64)
        stiffness = torch.tensor([26_214_400.0, 19_660_800.0], dtype=torch.float64)
        damping = torch.tensor([2_621_440.0, 1_966_080.0], dtype=torch.float64)
        actual = _canonical_pd_requested_effort_tensor(
            target, position, velocity, stiffness, damping
        )
        expected = []
        for channel in range(2):
            error = int(target[0, channel] - position[0, channel])
            expected.append(
                round_div_ties_even(int(stiffness[channel]) * error, 65_536)
                - round_div_ties_even(
                    int(damping[channel]) * int(velocity[0, channel]), 65_536
                )
            )
        torch.testing.assert_close(
            actual, torch.tensor([expected], dtype=torch.float64)
        )

    def test_effort_power_and_work_intersection_matches_cpu_safety_mirror(
        self,
    ) -> None:
        requested = torch.tensor(
            [[300_000_000.0, 300_000_000.0, -300_000_000.0]],
            dtype=torch.float64,
        )
        previous = torch.zeros((1, 3), dtype=torch.float64)
        velocity = torch.tensor([[0.0, 8_000_000.0, -8_000_000.0]], dtype=torch.float64)
        used_work = torch.tensor(
            [[0.0, 13_000_000.0, 13_000_000.0]], dtype=torch.float64
        )
        minimum_effort = torch.full((3,), -220_000_000.0, dtype=torch.float64)
        maximum_effort = torch.full((3,), 220_000_000.0, dtype=torch.float64)
        maximum_rate = torch.full((3,), 6_000_000_000.0, dtype=torch.float64)
        maximum_power = torch.full((3,), 800_000_000.0, dtype=torch.float64)
        maximum_work = torch.full((3,), 13_333_333.0, dtype=torch.float64)
        effort, next_work, infeasible = _intersect_effort_limits_tensor(
            requested,
            previous,
            velocity,
            used_work,
            minimum_effort,
            maximum_effort,
            maximum_rate,
            maximum_power,
            maximum_work,
        )
        expected_effort = []
        expected_work = []
        for channel in range(3):
            actuator = {
                "effort_micronewton_metres": [-220_000_000, 220_000_000],
                "maximum_effort_rate_micronewton_metres_per_second": 6_000_000_000,
                "maximum_power_microwatts": 800_000_000,
                "maximum_positive_work_microjoules_per_motor_tick": 13_333_333,
            }
            actual_effort, _ = _intersect_effort(
                actuator,
                int(requested[0, channel]),
                int(previous[0, channel]),
                int(velocity[0, channel]),
                int(used_work[0, channel]),
            )
            expected_effort.append(actual_effort)
            expected_work.append(
                int(used_work[0, channel])
                + _positive_work_charge(actual_effort, int(velocity[0, channel]))
            )
        torch.testing.assert_close(
            effort, torch.tensor([expected_effort], dtype=torch.float64)
        )
        torch.testing.assert_close(
            next_work, torch.tensor([expected_work], dtype=torch.float64)
        )
        self.assertFalse(torch.any(infeasible))

        _, _, infeasible = _intersect_effort_limits_tensor(
            requested[:, :1],
            previous[:, :1],
            velocity[:, :1],
            torch.tensor([[13_333_334.0]], dtype=torch.float64),
            minimum_effort[:1],
            maximum_effort[:1],
            maximum_rate[:1],
            maximum_power[:1],
            maximum_work[:1],
        )
        self.assertTrue(bool(infeasible.item()))


if __name__ == "__main__":
    unittest.main()
