from __future__ import annotations

import unittest
from pathlib import Path

import numpy as np

from next_lab.reference_tracker import (
    ACTION_CHANNELS,
    CONTACT_IMPACT_MARGIN_PROFILE_SHA256,
    DYNAMIC_RESERVE_PROFILE_SHA256,
    OBSERVATION_CHANNELS,
    PHYSICS_VELOCITY_GUARD_PROFILE_SHA256,
    PROFILE_SHA256,
    PREDICTIVE_ROM_COST_PROFILE_SHA256,
    SAFETY_RESERVE_PROFILE_SHA256,
    SOFT_ROM_COST_PROFILE_SHA256,
    DescriptorLimits,
    ReferenceClip,
    ReferenceTrackerProfile,
    _contact_impact_margin_cost,
    TrackingState,
    build_observation,
    compute_reward,
    derive_named_seed,
    reference_state,
)


PROFILE = Path(__file__).parents[1] / "profiles/humanoid-reference-tracker.v1.json"
SOFT_ROM_COST_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-tracker-soft-rom-cost.v1.json"
)
PREDICTIVE_ROM_COST_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-tracker-predictive-rom-cost.v1.json"
)
SAFETY_RESERVE_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-tracker-safety-reserve.v2.json"
)
DYNAMIC_RESERVE_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-tracker-dynamic-reserve.v3.json"
)
PHYSICS_VELOCITY_GUARD_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-tracker-physics-velocity-guard.v4.json"
)
CONTACT_IMPACT_MARGIN_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/humanoid-reference-tracker-contact-impact-margin.v5.json"
)


class ReferenceTrackerTests(unittest.TestCase):
    def test_contact_impact_margin_cost_uses_last_five_percent(self) -> None:
        self.assertEqual(_contact_impact_margin_cost(9_499, 9_500), 0.0)
        self.assertEqual(_contact_impact_margin_cost(9_500, 9_500), 0.0)
        self.assertEqual(_contact_impact_margin_cost(9_750, 9_500), 0.5)
        self.assertEqual(_contact_impact_margin_cost(10_000, 9_500), 1.0)
        self.assertEqual(_contact_impact_margin_cost(11_000, 9_500), 1.0)

    def test_contact_impact_margin_profile_preserves_guarded_lineage(self) -> None:
        profile = ReferenceTrackerProfile.load(CONTACT_IMPACT_MARGIN_PROFILE)
        self.assertEqual(
            profile.document_sha256, CONTACT_IMPACT_MARGIN_PROFILE_SHA256
        )
        self.assertEqual(
            profile.document["corpus"]["manifest_sha256"],
            "33546488a73db25557c23fdb1a54b066ac3d02384aaca9acb529dab5d4cc81fd",
        )
        self.assertEqual(
            profile.document["termination"][
                "isaac_physics_velocity_limit_basis_points"
            ],
            9_000,
        )
        clip = _clip()
        limits = _limits()
        state = reference_state(clip, 2)
        warned = compute_reward(
            profile,
            limits,
            TrackingState(
                **{
                    **state.__dict__,
                    "maximum_contact_impulse_basis_points": 9_750,
                }
            ),
            clip,
            2,
            terminal_failure=False,
        )
        self.assertEqual(
            dict(warned.component_values_q16)[
                "reward.contact-impact-margin-cost"
            ],
            32_768,
        )

    def test_safety_reserve_profile_replaces_only_input_and_terminal_lineage(self) -> None:
        base = ReferenceTrackerProfile.load(PROFILE)
        profile = ReferenceTrackerProfile.load(SAFETY_RESERVE_PROFILE)
        self.assertEqual(profile.document_sha256, SAFETY_RESERVE_PROFILE_SHA256)
        self.assertEqual(
            profile.document["termination"]["safety_contact_profile_sha256"],
            "ba9d368e075f389a4dbff4a0ed9299b737edf4907be10ae6cf3aeb60b348729f",
        )
        self.assertEqual(
            profile.document["corpus"]["manifest_sha256"],
            "2afcd10a61c60ec8d7715758eae6f5a87a98e7f0906620977602cf1b259f7ed4",
        )
        self.assertEqual(profile.document["reward"], base.document["reward"])
        self.assertIn(
            "terminal.self-collision", profile.document["termination"]["failure_reasons"]
        )

    def test_dynamic_reserve_profile_closes_new_data_gate_without_optimizer(self) -> None:
        profile = ReferenceTrackerProfile.load(DYNAMIC_RESERVE_PROFILE)
        self.assertEqual(profile.document_sha256, DYNAMIC_RESERVE_PROFILE_SHA256)
        self.assertEqual(
            profile.document["corpus"]["manifest_sha256"],
            "33546488a73db25557c23fdb1a54b066ac3d02384aaca9acb529dab5d4cc81fd",
        )
        self.assertIn(
            "optimizer-free", profile.document["training_authorization"]["scope"]
        )

    def test_physics_velocity_guard_is_hash_bound_without_relaxing_outer_limit(self) -> None:
        profile = ReferenceTrackerProfile.load(PHYSICS_VELOCITY_GUARD_PROFILE)
        self.assertEqual(
            profile.document_sha256, PHYSICS_VELOCITY_GUARD_PROFILE_SHA256
        )
        projection = profile.contract_projection()
        self.assertEqual(
            projection["isaac_velocity_guard_profile_sha256"],
            "a5c8448a71f0adc2c821b8d833332e0ecd1e003110ce47d969f56cac13881676",
        )
        self.assertEqual(
            projection["isaac_physics_velocity_limit_basis_points"], 9_000
        )

    def test_frozen_profile_closes_exact_layout_and_subprofiles(self) -> None:
        profile = ReferenceTrackerProfile.load(PROFILE)
        self.assertEqual(profile.document_sha256, PROFILE_SHA256)
        projection = profile.contract_projection()
        self.assertEqual(projection["actor_channel_count"], OBSERVATION_CHANNELS)
        self.assertEqual(projection["action_channel_count"], ACTION_CHANNELS)
        self.assertEqual(projection["horizon_offsets_motor_ticks"], [0, 4, 8, 16])
        self.assertEqual(projection["reset_mode_weights_basis_points"], [7000, 2000, 1000, 0])
        self.assertEqual(len(projection["reward_profile_hash"]), 64)
        action = profile.document["action"]
        joints = profile.document["observation"]["joint_state_order"]
        self.assertEqual(
            sorted(action["reference_joint_dof_ordinal_by_action_channel"]),
            list(range(ACTION_CHANNELS)),
        )
        self.assertEqual(
            [
                joints[dof].removeprefix("joint.")
                for dof in action["reference_joint_dof_ordinal_by_action_channel"]
            ],
            [
                actuator.removeprefix("actuator.")
                for actuator in action["ordered_actuator_ids"]
            ],
        )

    def test_named_seed_is_repeatable_and_purpose_separated(self) -> None:
        run_root = bytes(range(32))
        first = derive_named_seed(run_root, "train", 9, 3, "randomization.reference-clip")
        self.assertEqual(
            first,
            derive_named_seed(run_root, "train", 9, 3, "randomization.reference-clip"),
        )
        self.assertNotEqual(
            first,
            derive_named_seed(run_root, "train", 9, 3, "randomization.reference-phase"),
        )
        self.assertNotEqual(
            first,
            derive_named_seed(run_root, "heldout", 9, 3, "randomization.reference-clip"),
        )

    def test_soft_rom_cost_profile_is_distinct_and_warns_before_hard_rom(self) -> None:
        profile = ReferenceTrackerProfile.load(SOFT_ROM_COST_PROFILE)
        self.assertEqual(profile.document_sha256, SOFT_ROM_COST_PROFILE_SHA256)
        clip = _clip()
        limits = _limits()
        state = reference_state(clip, 2)
        inside = compute_reward(
            profile, limits, state, clip, 2, terminal_failure=False
        )
        self.assertEqual(
            dict(inside.component_values_q16)["reward.soft-rom-excursion-cost"], 0
        )
        outside = TrackingState(
            **{
                **state.__dict__,
                "joint_position_urad": np.asarray(
                    [750_000, *state.joint_position_urad[1:]], dtype=np.int64
                ),
            }
        )
        warned = compute_reward(
            profile, limits, outside, clip, 2, terminal_failure=False
        )
        self.assertEqual(
            dict(warned.component_values_q16)["reward.soft-rom-excursion-cost"],
            32_768,
        )
        self.assertLess(warned.total_q16, inside.total_q16)

    def test_predictive_rom_cost_warns_one_motor_tick_before_excursion(self) -> None:
        profile = ReferenceTrackerProfile.load(PREDICTIVE_ROM_COST_PROFILE)
        self.assertEqual(
            profile.document_sha256, PREDICTIVE_ROM_COST_PROFILE_SHA256
        )
        clip = _clip()
        limits = _limits()
        state = reference_state(clip, 2)
        approaching = TrackingState(
            **{
                **state.__dict__,
                "joint_position_urad": np.asarray(
                    [500_000, *state.joint_position_urad[1:]], dtype=np.int64
                ),
                "joint_velocity_urad_s": np.asarray(
                    [15_000_000, *state.joint_velocity_urad_s[1:]], dtype=np.int64
                ),
            }
        )
        warned = compute_reward(
            profile, limits, approaching, clip, 2, terminal_failure=False
        )
        self.assertEqual(
            dict(warned.component_values_q16)[
                "reward.predictive-rom-excursion-cost"
            ],
            32_768,
        )

    def test_observation_has_435_channels_and_clamped_reference_horizon(self) -> None:
        clip = _clip()
        state = reference_state(clip, 0)
        observation = build_observation(state, clip, 0)
        self.assertEqual(observation.shape, (OBSERVATION_CHANNELS,))
        np.testing.assert_array_equal(observation[:4], np.asarray([0, 0, 0, 1 << 30]))
        self.assertEqual(observation[86], 0)
        np.testing.assert_array_equal(observation[87:90], np.zeros(3, dtype=np.int64))
        np.testing.assert_array_equal(
            observation[174:177], np.asarray([4_000, 0, 0], dtype=np.int64)
        )

        final = build_observation(reference_state(clip, clip.frame_count - 1), clip, clip.frame_count - 1)
        for start in (87, 174, 261, 348):
            np.testing.assert_array_equal(final[start : start + 3], np.zeros(3, dtype=np.int64))

    def test_perfect_reward_and_cost_signs_are_not_ambiguous(self) -> None:
        profile = ReferenceTrackerProfile.load(PROFILE)
        clip = _clip()
        limits = _limits()
        state = reference_state(clip, 2)
        perfect = compute_reward(profile, limits, state, clip, 2, terminal_failure=False)
        self.assertEqual(perfect.total_q16, 458_752)
        self.assertTrue(all(value == 65_536 for _, value in perfect.component_values_q16[:9]))
        self.assertTrue(all(value == 0 for _, value in perfect.component_values_q16[9:]))

        degraded = TrackingState(
            **{
                **state.__dict__,
                "root_position_um": state.root_position_um
                + np.asarray([0, 250_000, 0], dtype=np.int64),
                "applied_effort_unm": np.full(ACTION_CHANNELS, 1_000, dtype=np.int64),
                "applied_target_urad": state.applied_target_urad
                + np.full(ACTION_CHANNELS, 1_000, dtype=np.int64),
            }
        )
        result = compute_reward(profile, limits, degraded, clip, 2, terminal_failure=True)
        values = dict(result.component_values_q16)
        self.assertEqual(values["reward.reference-root-height"], 0)
        self.assertGreater(values["reward.normalized-applied-effort-cost"], 0)
        self.assertGreater(values["reward.applied-target-rate-cost"], 0)
        self.assertEqual(values["reward.terminal-failure"], 65_536)
        self.assertLess(result.total_q16, perfect.total_q16)


def _clip() -> ReferenceClip:
    frames = 20
    root_position = np.zeros((frames, 3), dtype=np.int64)
    root_position[:, 0] = np.arange(frames, dtype=np.int64) * 1_000
    root_quaternion = np.zeros((frames, 4), dtype=np.int64)
    root_quaternion[:, 3] = 1 << 30
    joint_position = np.arange(frames * ACTION_CHANNELS, dtype=np.int64).reshape(
        frames, ACTION_CHANNELS
    )
    arrays = {
        "root_position_um": root_position,
        "root_quaternion_q1_30": root_quaternion,
        "root_linear_velocity_um_s": np.zeros((frames, 3), dtype=np.int64),
        "root_yaw_velocity_urad_s": np.zeros(frames, dtype=np.int64),
        "joint_position_urad": joint_position,
        "joint_velocity_urad_s": np.zeros((frames, ACTION_CHANNELS), dtype=np.int64),
        "center_of_mass_um": root_position + np.asarray([0, 800_000, 0], dtype=np.int64),
        "effector_position_um": np.repeat(root_position[:, None, :], 6, axis=1),
        "contacts": np.ones((frames, 7), dtype=np.uint8),
        "phase_u16": np.rint(np.linspace(0, 65_535, frames)).astype(np.uint16),
    }
    return ReferenceClip(
        clip_id="fixture.walk",
        skill="walk_nominal",
        split="train",
        split_group_id="fixture.subject",
        arrays=arrays,
        metadata={},
    )


def _limits() -> DescriptorLimits:
    return DescriptorLimits(
        soft_rom_spans_urad=np.full(ACTION_CHANNELS, 1_000_000, dtype=np.int64),
        soft_minimum_urad=np.full(ACTION_CHANNELS, -500_000, dtype=np.int64),
        soft_maximum_urad=np.full(ACTION_CHANNELS, 500_000, dtype=np.int64),
        hard_minimum_urad=np.full(ACTION_CHANNELS, -1_000_000, dtype=np.int64),
        hard_maximum_urad=np.full(ACTION_CHANNELS, 1_000_000, dtype=np.int64),
        maximum_velocity_urad_s=np.full(ACTION_CHANNELS, 1_000_000, dtype=np.int64),
        maximum_effort_unm=np.full(ACTION_CHANNELS, 10_000, dtype=np.int64),
        maximum_target_delta_urad=np.full(ACTION_CHANNELS, 10_000, dtype=np.int64),
        document_sha256="00" * 32,
    )


if __name__ == "__main__":
    unittest.main()
