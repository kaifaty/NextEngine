from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from next_lab.isaac_training import (
    RUN_MANIFEST_SCHEMA,
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    atomic_write_json,
    canonical_json_hash,
    closed_checkpoint_history,
    equal_episode_quota,
    latest_closed_checkpoint,
    parse_gpu_memory_csv,
    require_external_path,
    sha256_file,
    training_config_hash,
    validate_checkpoint_artifacts,
    validate_closed_checkpoint,
    validate_resume_checkpoint,
)


PROFILE = Path(__file__).parents[1] / "profiles/isaac-rsl-rl-rtx3080-poc.v1.json"
CURRICULUM_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/isaac-rsl-rl-rtx3080-locomotion-curriculum.v2.json"
)


class IsaacTrainingTests(unittest.TestCase):
    def test_rtx3080_profile_is_explicit_and_reproducible(self) -> None:
        profile = IsaacTrainingProfile.load(PROFILE)
        config = ResolvedTrainingConfig.from_profile(profile)
        self.assertEqual(config.num_envs, 128)
        self.assertEqual(config.steps_per_env, 32)
        self.assertEqual(len(config.run_root_hex), 64)
        self.assertEqual(
            config.run_root_hex,
            ResolvedTrainingConfig.from_profile(profile).run_root_hex,
        )
        different = ResolvedTrainingConfig.from_profile(profile, seed=43)
        self.assertNotEqual(config.run_root_hex, different.run_root_hex)

    def test_curriculum_profile_scales_samples_and_retains_held_out_seeds(self) -> None:
        profile = IsaacTrainingProfile.load(CURRICULUM_PROFILE)
        config = ResolvedTrainingConfig.from_profile(profile)
        self.assertEqual(config.num_envs, 512)
        self.assertEqual(config.steps_per_env, 24)
        self.assertEqual(config.iterations, 3_000)
        self.assertEqual(config.num_envs * config.steps_per_env * config.iterations, 36_864_000)
        self.assertEqual(
            profile.environment_profile_id,
            "nextengine.motor.env.humanoid-flat-command-curriculum.v2",
        )
        self.assertEqual(profile.evaluation["seeds"], [1001, 1002, 1003, 1004, 1005])
        self.assertEqual(profile.evaluation["episode_ordinal_start"], 96)
        self.assertGreater(profile.algorithm["entropy_coef"], 0)

    def test_config_hash_binds_artifacts_and_overrides(self) -> None:
        profile = IsaacTrainingProfile.load(PROFILE)
        first = ResolvedTrainingConfig.from_profile(profile)
        second = ResolvedTrainingConfig.from_profile(profile, num_envs=64)
        descriptor = "12" * 32
        usd = "34" * 32
        self.assertNotEqual(
            training_config_hash(first, descriptor, usd),
            training_config_hash(second, descriptor, usd),
        )
        self.assertNotEqual(
            training_config_hash(first, descriptor, usd),
            training_config_hash(first, descriptor, "56" * 32),
        )

    def test_repository_artifact_path_is_rejected(self) -> None:
        repository = Path(__file__).parents[2]
        with self.assertRaisesRegex(ValueError, "outside the repository"):
            require_external_path(
                repository / "lab",
                repository,
                label="training store",
            )

    def test_resume_requires_matching_closed_checkpoint(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            checkpoint = run / "model_7.pt"
            checkpoint.write_bytes(b"checkpoint")
            config_hash = canonical_json_hash({"config": "one"})
            manifest = {
                "schema": RUN_MANIFEST_SCHEMA,
                "status": "completed",
                "training_config_hash": config_hash,
                "checkpoints": [
                    {
                        "file": checkpoint.name,
                        "bytes": checkpoint.stat().st_size,
                        "sha256": sha256_file(checkpoint),
                    }
                ],
            }
            atomic_write_json(run / "run-manifest.json", manifest)
            self.assertEqual(
                validate_resume_checkpoint(checkpoint, config_hash)["status"],
                "completed",
            )
            self.assertEqual(validate_closed_checkpoint(checkpoint)["status"], "completed")
            checkpoint.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "hash"):
                validate_resume_checkpoint(checkpoint, config_hash)

    def test_latest_closed_checkpoint_ignores_running_and_corrupt_runs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile = IsaacTrainingProfile.load(PROFILE)

            def write_run(name: str, iteration: int, status: str, payload: bytes) -> Path:
                run = root / name
                run.mkdir()
                checkpoint = run / f"model_{iteration}.pt"
                checkpoint.write_bytes(payload)
                atomic_write_json(
                    run / "run-manifest.json",
                    {
                        "schema": RUN_MANIFEST_SCHEMA,
                        "status": status,
                        "training_config": {"profile_hash": profile.profile_hash},
                        "checkpoints": [
                            {
                                "file": checkpoint.name,
                                "bytes": checkpoint.stat().st_size,
                                "sha256": sha256_file(checkpoint),
                            }
                        ],
                    },
                )
                return checkpoint

            expected = write_run("20260811T100000Z-old", 10, "completed", b"old")
            write_run("20260811T110000Z-running", 20, "running", b"running")
            corrupt = write_run("20260811T120000Z-corrupt", 30, "completed", b"before")
            corrupt.write_bytes(b"after")
            self.assertEqual(latest_closed_checkpoint(root), expected.resolve())

    def test_closed_checkpoint_history_is_numeric_and_hash_checked(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            checkpoints = []
            for iteration in (10, 2, 7):
                checkpoint = run / f"model_{iteration}.pt"
                checkpoint.write_bytes(str(iteration).encode("ascii"))
                checkpoints.append(checkpoint)
            atomic_write_json(
                run / "run-manifest.json",
                {
                    "schema": RUN_MANIFEST_SCHEMA,
                    "status": "completed",
                    "checkpoints": [
                        {
                            "file": checkpoint.name,
                            "bytes": checkpoint.stat().st_size,
                            "sha256": sha256_file(checkpoint),
                        }
                        for checkpoint in checkpoints
                    ],
                },
            )
            self.assertEqual(
                closed_checkpoint_history(run / "model_7.pt"),
                [
                    (run / "model_2.pt").resolve(),
                    (run / "model_7.pt").resolve(),
                    (run / "model_10.pt").resolve(),
                ],
            )
            (run / "model_10.pt").write_bytes(b"corrupt")
            self.assertEqual(
                closed_checkpoint_history(run / "model_7.pt"),
                [
                    (run / "model_2.pt").resolve(),
                    (run / "model_7.pt").resolve(),
                ],
            )

    def test_checkpoint_artifacts_bind_profile_descriptor_and_usd(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            descriptor = root / "descriptor.json"
            usd = root / "humanoid.usda"
            descriptor.write_bytes(b"descriptor")
            usd.write_bytes(b"usd")
            profile = IsaacTrainingProfile.load(PROFILE)
            parent = {
                "training_config": {"profile_hash": profile.profile_hash},
                "artifacts": {
                    "descriptor": {"sha256": sha256_file(descriptor)},
                    "usd": {"sha256": sha256_file(usd)},
                },
            }
            validate_checkpoint_artifacts(parent, profile, descriptor, usd)
            usd.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "usd"):
                validate_checkpoint_artifacts(parent, profile, descriptor, usd)

    def test_zero_override_is_rejected(self) -> None:
        profile = IsaacTrainingProfile.load(PROFILE)
        with self.assertRaisesRegex(ValueError, "num_envs"):
            ResolvedTrainingConfig.from_profile(profile, num_envs=0)

    def test_evaluation_quota_is_equal_per_slot(self) -> None:
        self.assertEqual(equal_episode_quota(256, 64), 4)
        with self.assertRaisesRegex(ValueError, "multiple of num_envs"):
            equal_episode_quota(17, 16)
        with self.assertRaisesRegex(ValueError, "multiple of num_envs"):
            equal_episode_quota(8, 16)

    def test_atomic_json_and_gpu_memory_parser_are_bounded(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "manifest.json"
            atomic_write_json(output, {"status": "running"})
            self.assertEqual(json.loads(output.read_text())["status"], "running")
            self.assertFalse(output.with_suffix(".json.tmp").exists())
        self.assertEqual(
            parse_gpu_memory_csv("0, NVIDIA GeForce RTX 3080, 10240, 8192"),
            {
                "index": 0,
                "name": "NVIDIA GeForce RTX 3080",
                "memory_total_mib": 10240,
                "memory_free_mib": 8192,
            },
        )


if __name__ == "__main__":
    unittest.main()
