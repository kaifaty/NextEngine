from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from next_lab.isaac_training import (
    ACTIVE_TRAINING_GENERATION_SCHEMA,
    INCOMPATIBLE_TRAINING_GENERATION,
    RUN_MANIFEST_SCHEMA,
    TRAINING_GENERATION_MANIFEST_SCHEMA,
    TRAINING_GENERATION_NOT_ACTIVE,
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    TrainingGenerationError,
    activate_training_generation,
    atomic_write_json,
    canonical_json_hash,
    closed_checkpoint_history,
    equal_episode_quota,
    initialize_training_generation,
    latest_closed_checkpoint,
    load_active_training_generation,
    metrics_record,
    parse_gpu_memory_csv,
    require_external_path,
    require_run_id,
    sha256_file,
    training_config_hash,
    validate_checkpoint_artifacts,
    validate_closed_checkpoint,
    validate_closed_metrics,
    validate_resume_checkpoint,
)


PROFILE = Path(__file__).parents[1] / "profiles/isaac-rsl-rl-rtx3080-poc.v1.json"
CURRICULUM_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/isaac-rsl-rl-rtx3080-locomotion-curriculum.v2.json"
)
STANDING_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/isaac-rsl-rl-rtx3080-standing.v1.json"
)
BOUNDED_STANDING_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/isaac-rsl-rl-rtx3080-standing.v2.json"
)
BIOMECHANICS_STANDING_PROFILE = (
    Path(__file__).parents[1]
    / "profiles/isaac-rsl-rl-rtx3080-biomechanics-standing.v1.json"
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

    def test_standing_profile_is_bounded_to_the_first_r8b_gate(self) -> None:
        profile = IsaacTrainingProfile.load(STANDING_PROFILE)
        config = ResolvedTrainingConfig.from_profile(profile)
        self.assertEqual(
            profile.environment_profile_id,
            "nextengine.motor.env.humanoid-standing.v1",
        )
        self.assertEqual(
            config.num_envs * config.steps_per_env * config.iterations,
            4_096_000,
        )
        self.assertEqual(profile.evaluation["max_steps"], 3_600)
        self.assertEqual(profile.evaluation["episodes"], 1)
        self.assertEqual(profile.evaluation["num_envs"], 1)
        self.assertEqual(
            profile.evaluation["seeds"],
            [1001, 1002, 1003, 1004, 1005],
        )

    def test_bounded_standing_v2_changes_only_environment_identity(self) -> None:
        legacy = IsaacTrainingProfile.load(STANDING_PROFILE)
        bounded = IsaacTrainingProfile.load(BOUNDED_STANDING_PROFILE)
        self.assertEqual(
            bounded.environment_profile_id,
            "nextengine.motor.env.humanoid-standing.v2",
        )
        self.assertNotEqual(bounded.profile_hash, legacy.profile_hash)
        self.assertEqual(bounded.num_envs, legacy.num_envs)
        self.assertEqual(bounded.steps_per_env, legacy.steps_per_env)
        self.assertEqual(bounded.iterations, legacy.iterations)
        self.assertEqual(bounded.policy, legacy.policy)
        self.assertEqual(bounded.algorithm, legacy.algorithm)
        self.assertEqual(bounded.evaluation, legacy.evaluation)

    def test_biomechanics_standing_is_a_small_independent_successor_run(self) -> None:
        profile = IsaacTrainingProfile.load(BIOMECHANICS_STANDING_PROFILE)
        config = ResolvedTrainingConfig.from_profile(profile)
        self.assertEqual(
            profile.environment_profile_id,
            "nextengine.motor.env.humanoid-biomechanics-standing.v1",
        )
        self.assertEqual(config.num_envs * config.steps_per_env * config.iterations, 1_024_000)
        self.assertEqual(profile.policy["init_noise_std"], 0.35)
        self.assertEqual(profile.evaluation["max_steps"], 3_600)
        self.assertEqual(profile.evaluation["seeds"], [1001, 1002, 1003, 1004, 1005])

    def test_config_hash_binds_artifacts_and_overrides(self) -> None:
        profile = IsaacTrainingProfile.load(PROFILE)
        first = ResolvedTrainingConfig.from_profile(profile)
        second = ResolvedTrainingConfig.from_profile(profile, num_envs=64)
        descriptor = "12" * 32
        usd = "34" * 32
        self.assertNotEqual(
            training_config_hash(first, descriptor, usd, "78" * 32),
            training_config_hash(second, descriptor, usd, "78" * 32),
        )
        self.assertNotEqual(
            training_config_hash(first, descriptor, usd, "78" * 32),
            training_config_hash(first, descriptor, "56" * 32, "78" * 32),
        )
        self.assertNotEqual(
            training_config_hash(first, descriptor, usd, "78" * 32),
            training_config_hash(first, descriptor, usd, "9a" * 32),
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
            metrics = run / "metrics.jsonl"
            metrics.write_text('{"iteration": 0}\n', encoding="utf-8")
            config_hash = canonical_json_hash({"config": "one"})
            manifest = {
                "schema": RUN_MANIFEST_SCHEMA,
                "status": "completed",
                "training_generation_id": "nextengine.training.generation.test.v1",
                "training_config_hash": config_hash,
                "metrics": metrics_record(metrics),
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
                validate_resume_checkpoint(
                    checkpoint,
                    config_hash,
                    "nextengine.training.generation.test.v1",
                )["status"],
                "completed",
            )
            self.assertEqual(validate_closed_checkpoint(checkpoint)["status"], "completed")
            metrics.write_text('{"iteration": 1}\n', encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "metrics sha256"):
                validate_closed_metrics(run, manifest)
            metrics.write_text('{"iteration": 0}\n', encoding="utf-8")
            checkpoint.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "hash"):
                validate_resume_checkpoint(
                    checkpoint,
                    config_hash,
                    "nextengine.training.generation.test.v1",
                )

    def test_resume_rejects_legacy_or_other_generation_before_use(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            checkpoint = run / "model_1.pt"
            checkpoint.write_bytes(b"checkpoint")
            config_hash = canonical_json_hash({"config": "one"})
            atomic_write_json(
                run / "run-manifest.json",
                {
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
                },
            )
            with self.assertRaisesRegex(
                TrainingGenerationError, INCOMPATIBLE_TRAINING_GENERATION
            ):
                validate_resume_checkpoint(
                    checkpoint,
                    config_hash,
                    "nextengine.training.generation.new.v1",
                )

    def test_latest_closed_checkpoint_ignores_running_and_corrupt_runs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile = IsaacTrainingProfile.load(PROFILE)

            def write_run(name: str, iteration: int, status: str, payload: bytes) -> Path:
                run = root / name
                run.mkdir()
                checkpoint = run / f"model_{iteration}.pt"
                checkpoint.write_bytes(payload)
                metrics = run / "metrics.jsonl"
                metrics.write_text('{"iteration": 0}\n', encoding="utf-8")
                atomic_write_json(
                    run / "run-manifest.json",
                    {
                        "schema": RUN_MANIFEST_SCHEMA,
                        "status": status,
                        "training_config": {"profile_hash": profile.profile_hash},
                        "metrics": metrics_record(metrics),
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
            metrics = run / "metrics.jsonl"
            metrics.write_text('{"iteration": 0}\n', encoding="utf-8")
            atomic_write_json(
                run / "run-manifest.json",
                {
                    "schema": RUN_MANIFEST_SCHEMA,
                    "status": "completed",
                    "metrics": metrics_record(metrics),
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

    def test_run_id_is_one_safe_bounded_path_segment(self) -> None:
        self.assertEqual(require_run_id("r8b-standing-seed42-v1"), "r8b-standing-seed42-v1")
        for invalid in ("", ".", "../escape", "nested/run", " space", "x" * 129):
            with self.subTest(invalid=invalid):
                with self.assertRaisesRegex(ValueError, "run ID"):
                    require_run_id(invalid)

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

    def test_metrics_record_closes_jsonl_bytes_hash_and_count(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            metrics = Path(temporary) / "metrics.jsonl"
            metrics.write_text('{"iteration": 0}\n\n{"iteration": 1}\n', encoding="utf-8")
            self.assertEqual(
                metrics_record(metrics),
                {
                    "file": "metrics.jsonl",
                    "bytes": metrics.stat().st_size,
                    "record_count": 2,
                    "sha256": sha256_file(metrics),
                },
            )

    def test_generation_index_is_hash_closed_and_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile = IsaacTrainingProfile.load(PROFILE)
            descriptor_hash = "12" * 32
            usd_hash = "34" * 32
            manifest_path = root / "generations/new/generation-manifest.json"
            manifest = {
                "schema": TRAINING_GENERATION_MANIFEST_SCHEMA,
                "schema_version": 1,
                "status": "prepared",
                "training_generation_id": "nextengine.training.generation.new.v1",
                "candidate_id": "HumanoidFlatRecoveryCandidateV1",
                "requirements_baseline_sha256": "56" * 32,
                "retired_inventory_sha256": "78" * 32,
                "admitted_inputs": [
                    {
                        "profile_id": profile.profile_id,
                        "profile_hash": profile.profile_hash,
                        "environment_profile_id": profile.environment_profile_id,
                        "descriptor_sha256": descriptor_hash,
                        "usd_sha256": usd_hash,
                    }
                ],
            }
            manifest["manifest_hash"] = canonical_json_hash(manifest)
            atomic_write_json(manifest_path, manifest)
            index = {
                "schema": ACTIVE_TRAINING_GENERATION_SCHEMA,
                "schema_version": 1,
                "active_training_generation_id": manifest["training_generation_id"],
                "generation_manifest": "generations/new/generation-manifest.json",
                "generation_manifest_sha256": sha256_file(manifest_path),
            }
            index["index_hash"] = canonical_json_hash(index)
            index_path = root / "active-generation.json"
            atomic_write_json(index_path, index)
            loaded = load_active_training_generation(index_path)
            with self.assertRaisesRegex(
                TrainingGenerationError, TRAINING_GENERATION_NOT_ACTIVE
            ):
                loaded.manifest.require_input(profile, descriptor_hash, usd_hash)

            manifest["status"] = "active"
            manifest.pop("manifest_hash")
            manifest["manifest_hash"] = canonical_json_hash(manifest)
            atomic_write_json(manifest_path, manifest)
            index["generation_manifest_sha256"] = sha256_file(manifest_path)
            index.pop("index_hash")
            index["index_hash"] = canonical_json_hash(index)
            atomic_write_json(index_path, index)
            loaded = load_active_training_generation(index_path)
            loaded.manifest.require_input(profile, descriptor_hash, usd_hash)
            with self.assertRaisesRegex(
                TrainingGenerationError, INCOMPATIBLE_TRAINING_GENERATION
            ):
                loaded.manifest.require_input(profile, descriptor_hash, "ab" * 32)

            manifest_path.write_text("{}\n", encoding="utf-8")
            with self.assertRaisesRegex(
                TrainingGenerationError, INCOMPATIBLE_TRAINING_GENERATION
            ):
                load_active_training_generation(index_path)

    def test_generation_initialization_inventories_without_deleting(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary) / "training-store"
            old_runs = store / "runs/legacy"
            old_evaluations = store / "evaluations/legacy"
            old_runs.mkdir(parents=True)
            old_evaluations.mkdir(parents=True)
            (old_runs / "run-manifest.json").write_text("{}\n", encoding="utf-8")
            (old_evaluations / "evaluation-manifest.json").write_text(
                "{}\n", encoding="utf-8"
            )
            baseline = Path(temporary) / "requirements.md"
            baseline.write_text("requirements\n", encoding="utf-8")
            result = initialize_training_generation(
                training_store=store,
                repository_root=Path(__file__).parents[2],
                generation_directory="humanoid-motor-rebuild-v1",
                generation_id="nextengine.training.generation.humanoid-motor-rebuild.v1",
                candidate_id="HumanoidFlatRecoveryCandidateV1",
                requirements_baseline=baseline,
                retired_generation_id="nextengine.training.generation.legacy-stage0.v1",
                retired_roots=[old_runs, old_evaluations],
            )
            generation_root = Path(result["generation_root"])
            self.assertEqual(
                [path.name for path in generation_root.iterdir()],
                ["generation-manifest.json"],
            )
            self.assertEqual(result["inventory_file_count"], "2")
            self.assertTrue((old_runs / "run-manifest.json").is_file())
            loaded = load_active_training_generation(Path(result["active_index"]))
            self.assertEqual(loaded.manifest.status, "prepared")
            self.assertEqual(loaded.manifest.admitted_inputs, ())

    def test_generation_activation_admits_exact_input_and_is_idempotent(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary) / "training-store"
            retired = store / "retired"
            retired.mkdir(parents=True)
            baseline = Path(temporary) / "requirements.md"
            baseline.write_text("requirements\n", encoding="utf-8")
            initialized = initialize_training_generation(
                training_store=store,
                repository_root=Path(__file__).parents[2],
                generation_directory="standing-v1",
                generation_id="nextengine.training.generation.standing.v1",
                candidate_id="HumanoidStage0StandingCandidateV1",
                requirements_baseline=baseline,
                retired_generation_id="nextengine.training.generation.none.v1",
                retired_roots=[retired],
            )
            profile = IsaacTrainingProfile.load(STANDING_PROFILE)
            index = Path(initialized["active_index"])
            descriptor_hash = "12" * 32
            usd_hash = "34" * 32
            first = activate_training_generation(
                index_path=index,
                profile=profile,
                descriptor_sha256=descriptor_hash,
                usd_sha256=usd_hash,
            )
            self.assertEqual(first.manifest.status, "active")
            first.manifest.require_input(profile, descriptor_hash, usd_hash)
            second = activate_training_generation(
                index_path=index,
                profile=profile,
                descriptor_sha256=descriptor_hash,
                usd_sha256=usd_hash,
            )
            self.assertEqual(second.manifest.manifest_hash, first.manifest.manifest_hash)
            with self.assertRaisesRegex(
                TrainingGenerationError, INCOMPATIBLE_TRAINING_GENERATION
            ):
                activate_training_generation(
                    index_path=index,
                    profile=profile,
                    descriptor_sha256=descriptor_hash,
                    usd_sha256="56" * 32,
                )


if __name__ == "__main__":
    unittest.main()
