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
    parse_gpu_memory_csv,
    require_external_path,
    sha256_file,
    training_config_hash,
    validate_closed_checkpoint,
    validate_resume_checkpoint,
)


PROFILE = Path(__file__).parents[1] / "profiles/isaac-rsl-rl-rtx3080-poc.v1.json"


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

    def test_zero_override_is_rejected(self) -> None:
        profile = IsaacTrainingProfile.load(PROFILE)
        with self.assertRaisesRegex(ValueError, "num_envs"):
            ResolvedTrainingConfig.from_profile(profile, num_envs=0)

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
