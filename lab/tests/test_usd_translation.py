from __future__ import annotations

import copy
import hashlib
import tempfile
import unittest
from pathlib import Path

from next_lab.biomechanics_material_lineage import (
    BIOMECHANICS_TRANSLATOR_ID_V2,
    BIOMECHANICS_USDA_TRANSLATOR_VERSION_V2,
    EXPECTED_ASSIGNMENT_COUNTS,
    EXPECTED_COMBINE_PROFILE,
    EXPECTED_MATERIALS,
)
from next_lab.motor_mirror import (
    CURRENT_TRANSLATOR_VERSION,
    LEGACY_TRANSLATOR_PROFILE_ID,
    load_json,
    validate_current_biomechanics_descriptor,
)
from next_lab.usd_translation import (
    BIOMECHANICS_TRANSLATOR_VERSION,
    render_ground_usda,
    render_usda,
    translate_to_store,
    validate_translation_bundle,
)

BIOMECHANICS_FIXTURE = (
    Path(__file__).parent / "fixtures/biomechanics_motor_mirror_v1.json"
)


def biomechanics_descriptor_v2() -> dict:
    value = load_json(BIOMECHANICS_FIXTURE)
    value.update(
        {
            "schema_version": 2,
            "translator_id": BIOMECHANICS_TRANSLATOR_ID_V2,
            "compiled_descriptor_schema_version": 3,
            "compiled_descriptor_hash": "6751853a812f549866f1db9d3662d8115b18db9b6d73beabd7221bb9f972f027",
            "material_lineage_hash": "2d13e197f766e6a24090edf396dfc2fb6cbbf4c578ea9868dffa06ab7adab751",
            "ground_material_id": "physics-material.humanoid-ground.v1",
            "materials": copy.deepcopy(EXPECTED_MATERIALS),
            "material_combine_profile": copy.deepcopy(EXPECTED_COMBINE_PROFILE),
            "collider_material_assignment_counts": copy.deepcopy(
                EXPECTED_ASSIGNMENT_COUNTS
            ),
        }
    )
    return value


def descriptor() -> dict:
    actuator_ids = [f"actuator.{index:02}" for index in range(23)]
    return {
        "schema_version": 2,
        "translator_version": "nextengine.isaac-usda-translator.v3",
        "observation_width": 84,
        "action_width": 23,
        "body_schema_hash": "12" * 32,
        "physics_hz": 240,
        "motor_hz": 60,
        "ordered_body_ids": ["body.root", "body.child"],
        "ordered_actuator_ids": actuator_ids,
        "bodies": [
            {
                "body_id": "body.child",
                "parent_body_id": "body.root",
                "local_bind_translation_micrometres": [100_000, 200_000, 300_000],
                "mass_microkilograms": 1_000_000,
                "center_of_mass_micrometres": [10_000, 20_000, 30_000],
                "inertia_microkilogram_metre_squared": [100_000, 200_000, 300_000],
                "colliders": [
                    {
                        "collider_id": "collider.child",
                        "geometry": {"kind": "sphere", "radius_micrometres": 100_000},
                    }
                ],
            },
            {
                "body_id": "body.root",
                "parent_body_id": None,
                "local_bind_translation_micrometres": [0, 1_050_000, 0],
                "mass_microkilograms": 10_000_000,
                "center_of_mass_micrometres": [0, 0, 0],
                "inertia_microkilogram_metre_squared": [1_000_000] * 3,
                "colliders": [
                    {
                        "collider_id": "collider.root",
                        "geometry": {"kind": "sphere", "radius_micrometres": 200_000},
                    }
                ],
            },
        ],
        "joints": [
            {
                "joint_id": "joint.child",
                "parent_body_id": "body.root",
                "child_body_id": "body.child",
                "parent_translation_micrometres": [100_000, 200_000, 300_000],
                "child_translation_micrometres": [0, 0, 0],
                "limit_min_microradians": -1_000_000,
                "limit_max_microradians": 1_000_000,
            }
        ],
        "actuators": [{"actuator_id": value} for value in actuator_ids],
        "environment_profiles": [
            _profile("nextengine.motor.env.humanoid-standing.v1", "world", 3_600, 8),
            _profile(
                "nextengine.motor.env.humanoid-flat-command.v1", "root-local", 1_200, 10
            ),
            _profile(
                "nextengine.motor.env.humanoid-flat-command-curriculum.v2",
                "root-local",
                1_200,
                11,
            ),
        ],
    }


def _profile(
    profile_id: str, velocity_frame: str, maximum_steps: int, reward_count: int
) -> dict:
    standing = [
        "reward.upright",
        "reward.root-height-tracking",
        "reward.standing-pose-tracking",
        "reward.velocity-penalty",
        "reward.effort-penalty",
        "reward.action-rate-penalty",
        "reward.foot-slip-penalty",
        "reward.fall-terminal",
    ]
    locomotion = [
        "reward.planar-command-tracking",
        "reward.yaw-rate-tracking",
        "reward.upright-yaw-invariant",
        "reward.root-height-tracking",
        "reward.vertical-velocity-cost",
        "reward.roll-pitch-rate-cost",
        "reward.normalized-applied-effort-cost",
        "reward.applied-action-rate-cost",
        "reward.contacting-foot-tangential-slip-cost",
        "reward.fall-component",
    ]
    curriculum = [
        *locomotion[:-1],
        "reward.command-conditioned-support",
        "reward.fall-component",
    ]
    profile = {
        "profile_id": profile_id,
        "velocity_frame": velocity_frame,
        "maximum_episode_steps": maximum_steps,
        "observation_source_ids": [f"observation.{index}" for index in range(84)],
        "reward_components": [
            {"component_id": value}
            for value in (
                standing
                if reward_count == 8
                else curriculum
                if reward_count == 11
                else locomotion
            )
        ],
    }
    for field in (
        "manifest_hash",
        "observation_layout_hash",
        "action_layout_hash",
        "command_schedule_profile_hash",
        "reward_profile_hash",
        "translator_version_hash",
        "termination_profile_hash",
        "rng_derivation_profile_hash",
        "correspondence_profile_hash",
    ):
        profile[field] = "12" * 32
    translator_identity = (
        CURRENT_TRANSLATOR_VERSION
        if reward_count == 11
        else LEGACY_TRANSLATOR_PROFILE_ID
    )
    profile["translator_version_hash"] = hashlib.sha256(
        translator_identity.encode("utf-8") + b"\0" + bytes.fromhex("12" * 32)
    ).hexdigest()
    if reward_count == 10:
        profile["command_profile"] = {
            "warmup_ticks": 60,
            "segment_ticks": 120,
            "episode_ticks": 1_200,
            "mode_weights_basis_points": [2_500, 3_500, 2_000, 2_000],
        }
    elif reward_count == 11:
        profile["command_profile"] = {
            "kind": "sha256-counter-episode-curriculum-v2",
            "episode_ticks": 1_200,
            "stages": [
                {
                    "first_episode_ordinal": first,
                    "warmup_ticks": warmup,
                    "segment_ticks": segment,
                    "episode_ticks": 1_200,
                    "mode_weights_basis_points": weights,
                    "right_velocity_range_raw": right,
                    "forward_velocity_range_raw": forward,
                    "yaw_rate_range_raw": yaw,
                    "linear_rate_limit_raw_per_second_squared": linear_rate,
                    "yaw_rate_limit_raw_per_second_squared": yaw_rate,
                }
                for first, warmup, segment, weights, right, forward, yaw, linear_rate, yaw_rate in (
                    (
                        0,
                        120,
                        240,
                        [4_000, 6_000, 0, 0],
                        [0, 0],
                        [0, 750_000],
                        [0, 0],
                        1_000_000,
                        500_000,
                    ),
                    (
                        32,
                        90,
                        180,
                        [2_500, 5_500, 1_500, 500],
                        [-350_000, 350_000],
                        [0, 1_250_000],
                        [-600_000, 600_000],
                        1_500_000,
                        750_000,
                    ),
                    (
                        96,
                        60,
                        120,
                        [1_500, 4_500, 1_500, 2_500],
                        [-1_000_000, 1_000_000],
                        [-500_000, 2_000_000],
                        [-1_000_000, 1_000_000],
                        2_000_000,
                        1_000_000,
                    ),
                )
            ],
        }
    return profile


class UsdTranslationTests(unittest.TestCase):
    def test_translation_is_stable_and_maps_engine_y_up_to_usd_z_up(self) -> None:
        first = render_usda(descriptor())
        second = render_usda(descriptor())
        self.assertEqual(first, second)
        self.assertIn('upAxis = "Z"', first)
        self.assertIn("double3 xformOp:translate = (0, 0, 1.05)", first)
        self.assertIn("double3 xformOp:translate = (0.1, -0.3, 1.25)", first)
        self.assertIn("point3f physics:centerOfMass = (0.01, -0.03, 0.02)", first)
        self.assertIn("float3 physics:diagonalInertia = (0.1, 0.3, 0.2)", first)
        self.assertIn("rel physics:body0 = </Humanoid/Bodies/body_root>", first)
        self.assertIn("point3f physics:localPos0 = (0.1, -0.3, 0.2)", first)
        self.assertIn("point3f physics:localPos1 = (0, 0, 0)", first)
        self.assertEqual(first.count("{"), first.count("}"))
        self.assertNotIn("\n{\n    {\n", first)

    def test_generated_usd_and_manifest_live_in_external_store(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            manifest = translate_to_store(
                descriptor(), store, Path(__file__).parents[2]
            )
            output = store / "derived" / ("12" * 32)
            self.assertTrue((output / "humanoid.usda").is_file())
            self.assertTrue((output / "translation-manifest.json").is_file())
            self.assertEqual(len(manifest["usd_sha256"]), 64)

    def test_biomechanics_translation_projects_solver_frames_and_filters(self) -> None:
        source = load_json(BIOMECHANICS_FIXTURE)
        first = render_usda(source)
        self.assertEqual(first, render_usda(source))
        self.assertEqual(
            hashlib.sha256(first.encode("utf-8")).hexdigest(),
            "951fc04bed0ff3e15414bfd06d067636ee8db18ef70e6e4a0ced28c624fcefa4",
        )
        self.assertIn(BIOMECHANICS_TRANSLATOR_VERSION, first)
        self.assertEqual(first.count("def PhysicsRevoluteJoint"), 23)
        self.assertEqual(
            first.count('prepend apiSchemas = ["PhysicsCollisionAPI"]'), 19
        )
        self.assertIn('def PhysicsRevoluteJoint "joint_left_hip_yaw"', first)
        self.assertIn(
            "quatf physics:localRot0 = (0.707106781, 0, 0.707106781, 0)",
            first,
        )
        self.assertIn(
            "rel physics:filteredPairs = [</Humanoid/Bodies/body_left_hip_pitch>",
            first,
        )
        self.assertIn(
            'custom string nextengine:semanticId = "collider.left-foot"', first
        )
        self.assertIn(
            'uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:orient", "xformOp:scale"]',
            first,
        )
        self.assertEqual(first.count("{"), first.count("}"))

    def test_current_biomechanics_translation_binds_exact_materials(self) -> None:
        source = biomechanics_descriptor_v2()
        validate_current_biomechanics_descriptor(source)
        humanoid = render_usda(source)
        ground = render_ground_usda(source)

        self.assertIn(BIOMECHANICS_USDA_TRANSLATOR_VERSION_V2, humanoid)
        self.assertIn(
            'nextengine:materialContract = "PhysicsMaterialDescriptorV2"', humanoid
        )
        self.assertEqual(humanoid.count('def Material "'), 2)
        self.assertEqual(humanoid.count("rel material:binding:physics"), 19)
        self.assertEqual(humanoid.count("physics:staticFriction = 0.800003052"), 2)
        self.assertEqual(humanoid.count("physics:dynamicFriction = 0.699996948"), 2)
        self.assertNotIn('def Material "physics_material_humanoid_ground_v1"', humanoid)
        self.assertIn('def Material "physics_material_humanoid_ground_v1"', ground)
        self.assertEqual(ground.count("rel material:binding:physics"), 1)
        self.assertIn('def Plane "Collision"', ground)
        self.assertNotIn("physics:staticFriction = 0.5\n", humanoid + ground)
        self.assertNotIn("physics:dynamicFriction = 0.5\n", humanoid + ground)

    def test_current_material_translation_rejects_nonzero_extension(self) -> None:
        source = biomechanics_descriptor_v2()
        source["materials"][0]["rolling_friction_q16"] = 1
        with self.assertRaisesRegex(ValueError, "material catalog"):
            render_usda(source)

        source = biomechanics_descriptor_v2()
        source["materials"][0]["descriptor_revision"] = True
        with self.assertRaisesRegex(ValueError, "numeric type"):
            render_usda(source)

        source = biomechanics_descriptor_v2()
        source["ambient_material_fallback"] = True
        with self.assertRaisesRegex(ValueError, "identity mismatch"):
            render_usda(source)

    def test_biomechanics_manifest_uses_separate_translator_identity(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            manifest = translate_to_store(
                load_json(BIOMECHANICS_FIXTURE),
                Path(temporary),
                Path(__file__).parents[2],
            )
            self.assertEqual(
                manifest["translator_version"], BIOMECHANICS_TRANSLATOR_VERSION
            )
            self.assertEqual(
                manifest["compiled_descriptor_hash"],
                "b6f8b1260b24ad401453942c9a4303d99ce378a8f71c6490db79bb78d85a1782",
            )

    def test_current_bundle_uses_compiled_identity_and_explicit_ground(self) -> None:
        source = biomechanics_descriptor_v2()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = translate_to_store(source, root, Path(__file__).parents[2])
            output = (
                root
                / "derived"
                / source["body_schema_hash"]
                / source["compiled_descriptor_hash"]
            )
            self.assertTrue((output / "humanoid.usda").is_file())
            self.assertTrue((output / "ground.usda").is_file())
            self.assertEqual(manifest["schema_version"], 3)
            self.assertEqual(
                manifest["material_lineage_hash"], source["material_lineage_hash"]
            )
            self.assertEqual(
                validate_translation_bundle(
                    source,
                    output / "translation-manifest.json",
                    humanoid_usd_path=output / "humanoid.usda",
                    ground_usd_path=output / "ground.usda",
                ),
                manifest,
            )

            (output / "ground.usda").write_text("tampered", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "translated USD identity"):
                validate_translation_bundle(
                    source, output / "translation-manifest.json"
                )
            with self.assertRaisesRegex(FileExistsError, "identity collision"):
                translate_to_store(source, root, Path(__file__).parents[2])


if __name__ == "__main__":
    unittest.main()
