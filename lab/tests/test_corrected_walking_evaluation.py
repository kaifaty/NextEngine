from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
from next_lab.isaac_training import atomic_write_json, sha256_file

from lab.scripts.evaluate_corrected_walking import (
    check_compatibility,
    corrected_episode,
    validate_closed_source,
)
from lab.tests.test_canonical_ppo import descriptor


def descriptors():
    source = descriptor()
    source["observation_width"] = 88
    source["training_descriptor_id"] = (
        "nextengine.canonical.humanoid-biomechanics-forward-start-stop.v7"
    )
    env = source["environment_profiles"][0]
    env["profile_id"] = (
        "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v7"
    )
    env["command_profile"] = {
        "kind": "fixed-forward-start-stop-v2",
        "ramp_down_tick": 991,
        "profile_id": "nextengine.motor.command.biomechanics-forward-start-stop.v2",
        "maximum_delta": 16666,
    }
    target = copy.deepcopy(source)
    target["training_descriptor_id"] = (
        "nextengine.canonical.humanoid-biomechanics-forward-start-stop.v8"
    )
    env = target["environment_profiles"][0]
    env["profile_id"] = (
        "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v8"
    )
    env["command_profile"].update(
        kind="fixed-forward-start-stop-v3",
        ramp_down_tick=990,
        profile_id="nextengine.motor.command.biomechanics-forward-start-stop.v3",
        applied_action_indices_inclusive=[0, 1199],
        final_zero_action_indices_inclusive=[1020, 1199],
    )
    for key in (
        "manifest_hash",
        "command_schedule_profile_hash",
        "correspondence_profile_hash",
    ):
        env[key] = "34" * 32
    return source, target


class CompatibilityTests(unittest.TestCase):
    def test_only_declared_command_and_identity_changes_are_compatible(self):
        source, target = descriptors()
        saved = copy.deepcopy(target)
        check_compatibility(source, target)
        self.assertEqual(target, saved)

    def test_body_action_observation_reward_safety_and_command_drift_rejected(self):
        for field in (
            "joints",
            "actuators",
            "observation_width",
            "reward_profile_hash",
            "termination_profile_hash",
            "maximum_delta",
            "extra",
            "profile_id",
        ):
            source, target = descriptors()
            if field in ("joints", "actuators"):
                target[field].reverse()
            elif field == "observation_width":
                target[field] = 86
            elif field == "maximum_delta":
                target["environment_profiles"][0]["command_profile"][field] += 1
            elif field == "extra":
                target[field] = True
            else:
                target["environment_profiles"][0][field] = "bad"
            with (
                self.subTest(field=field),
                self.assertRaises((ValueError, AssertionError)),
            ):
                check_compatibility(source, target)


class SourceClosureTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.run = root / "run"
        self.run.mkdir()
        self.gen = root / "generation.json"
        self.training = root / "training.json"
        self.source_path = root / "source.json"
        self.target_path = root / "target.json"
        source, target = descriptors()
        atomic_write_json(self.source_path, source)
        atomic_write_json(self.target_path, target)
        cfg = {"seeds": [1001, 1002, 1003, 1004, 1005]}
        profile = {
            "num_envs": 2,
            "steps_per_env": 2,
            "iterations": 3,
            "evaluation": cfg,
        }
        atomic_write_json(self.training, profile)
        closure = {
            "repository_commit": "abc",
            "profile_sha256": sha256_file(self.training),
            "descriptor_sha256": sha256_file(self.source_path),
            "headless_sha256": "12" * 32,
            "dependencies": {},
        }
        atomic_write_json(
            self.gen,
            {
                "schema": "nextengine.canonical-ppo-generation.v1",
                "run_path": str(self.run),
                "profile": profile,
                "closure": closure,
            },
        )
        self.matrix = {
            "source_generation_sha256": sha256_file(self.gen),
            "source_training_profile_sha256": sha256_file(self.training),
            "source_descriptor_sha256": sha256_file(self.source_path),
            "target_descriptor_sha256": sha256_file(self.target_path),
            "source_checkpoint_name": "model_2.pt",
            "source_checkpoint_iteration": 2,
            "evaluation": cfg,
        }
        with (self.run / "metrics.jsonl").open("w") as stream:
            for i in range(3):
                stream.write(
                    json.dumps({"iteration": i, "total_timesteps": (i + 1) * 4}) + "\n"
                )
        for name in ["model_2.pt", "evaluation.json"] + [
            f"evaluation-{s}.npz" for s in cfg["seeds"]
        ]:
            atomic_write_json(self.run / name, {})
        self.manifest = {
            **closure,
            "schema": "nextengine.canonical-ppo-run.v1",
            "status": "completed",
            "mode": "train",
            "repository_dirty": False,
            "input_checkpoint": None,
            "profile": profile,
            "descriptor_path": str(self.source_path),
            "generation_manifest_sha256": sha256_file(self.gen),
            "artifacts": {p.name: sha256_file(p) for p in self.run.iterdir()},
        }
        self.save_manifest()
        self.patcher = patch(
            "lab.scripts.evaluate_corrected_walking.TRAINING_PROFILE", self.training
        )
        self.patcher.start()
        self.addCleanup(self.patcher.stop)

    def save_manifest(self):
        atomic_write_json(self.run / "run-manifest.json", self.manifest)

    def validate(self):
        return validate_closed_source(self.run, self.gen, self.target_path, self.matrix)

    def test_closed_exact_source_passes_without_loading_any_weights(self):
        self.assertEqual(
            self.validate()["source_checkpoint_sha256"],
            self.manifest["artifacts"]["model_2.pt"],
        )

    def test_running_failed_dirty_initialized_and_wrong_mode_rejected(self):
        baseline = copy.deepcopy(self.manifest)
        for key, value in [
            ("status", "running"),
            ("status", "failed"),
            ("repository_dirty", True),
            ("input_checkpoint", "other.pt"),
            ("mode", "freeze"),
        ]:
            self.manifest = {**baseline, key: value}
            self.save_manifest()
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                self.validate()

    def test_corrupted_artifact_and_incomplete_closure_rejected(self):
        path = self.run / "model_2.pt"
        atomic_write_json(path, {"corrupt": True})
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            self.validate()
        del self.manifest["artifacts"]["model_2.pt"]
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, "incomplete"):
            self.validate()

    def test_artifact_escape_rejected_even_with_valid_external_hash(self):
        self.manifest["artifacts"]["../source.json"] = sha256_file(self.source_path)
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, "escapes"):
            self.validate()

    def test_missing_updates_cannot_be_called_final(self):
        path = self.run / "metrics.jsonl"
        with path.open("w") as stream:
            stream.write(json.dumps({"iteration": 0, "total_timesteps": 4}) + "\n")
        self.manifest["artifacts"][path.name] = sha256_file(path)
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, "final checkpoint"):
            self.validate()

    def test_existing_intermediate_weights_cannot_replace_final(self):
        self.matrix["source_checkpoint_name"] = "model_1.pt"
        self.matrix["source_checkpoint_iteration"] = 1
        atomic_write_json(self.run / "model_1.pt", {})
        self.manifest["artifacts"]["model_1.pt"] = sha256_file(self.run / "model_1.pt")
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, "final checkpoint"):
            self.validate()


class CorrectedGateTests(unittest.TestCase):
    def setUp(self):
        self.cfg = json.loads(
            (
                Path(__file__).resolve().parents[1]
                / "profiles/canonical-walking-corrected-evaluation.v1.json"
            ).read_text()
        )["evaluation"]
        commands, speed = [], 0
        for tick in range(1200):
            target = 500_000 if 120 <= tick < 990 else 0
            speed = (
                min(speed + 16_666, target)
                if speed < target
                else max(speed - 16_666, target)
            )
            commands.append([0, speed, 0])
        self.frames = [
            {"command_raw": c, "completed_physics_substeps": 4} for c in commands
        ]
        self.trace = {
            "frames": [self.frames],
            "cases": [
                {"ticks": 1200, "terminal": "terminal.timeout", "safety_error": None}
            ],
        }
        self.raw = {
            "seed": 1001,
            "ticks": 1200,
            "terminal_reason": "terminal.timeout",
            "gates": {"all_old_flags": False},
        }
        positions = np.zeros((1200, 3))
        positions[:, 2] = np.linspace(0, 3, 1200)
        command = np.asarray(commands) / 1e6
        velocities = np.zeros((1200, 3))
        velocities[:, 2] = command[:, 1]
        self.evaluation = {
            "root_position_m": positions,
            "root_velocity_mps": velocities,
            "command": command,
        }
        self.support = {
            "load_and_lift": {
                "longest_continuous_ticks_by_stance_side": [8, 8],
                "switches_between_runs_of_at_least_eight_ticks": 2,
            }
        }

    def result(self):
        with (
            patch(
                "lab.scripts.evaluate_corrected_walking.analyze",
                return_value=({}, None, None),
            ),
            patch(
                "lab.scripts.evaluate_corrected_walking.lifted_support_report",
                return_value=self.support,
            ),
        ):
            return corrected_episode(
                self.raw, self.trace, {}, self.evaluation, self.cfg
            )[0]

    def test_exact_matrix_recomputes_metrics_not_old_presence_flags(self):
        self.assertTrue(self.result()["passed"])
        self.assertEqual(self.raw["gates"], {"all_old_flags": False})

    def test_early_terminal_and_partial_substep_never_pass(self):
        self.frames[-1]["completed_physics_substeps"] = 3
        self.assertFalse(self.result()["passed"])
        self.frames[-1]["completed_physics_substeps"] = 4
        self.trace["cases"][0]["terminal"] = self.raw["terminal_reason"] = (
            "terminal.joint-safety"
        )
        self.assertFalse(self.result()["passed"])

    def test_179_zero_commands_are_not_rounded_to_180(self):
        self.frames[1020]["command_raw"] = [0, 20, 0]
        self.evaluation["command"][1020] = [0, 0.000020, 0]
        with self.assertRaises(AssertionError):
            self.result()

    def test_short_episode_cannot_claim_full_timeout(self):
        self.frames.pop()
        self.trace["cases"][0]["ticks"] = self.raw["ticks"] = 1199
        self.evaluation = {k: v[:-1] for k, v in self.evaluation.items()}
        self.assertFalse(self.result()["gates"]["complete_safe_episode"])

    def test_unchanged_travel_tracking_stop_and_gait_thresholds(self):
        self.evaluation["root_position_m"][-1, 2] = 2.999
        self.assertFalse(self.result()["gates"]["forward_travel"])
        self.evaluation["root_velocity_mps"][:, 2] += 0.201
        self.assertFalse(self.result()["gates"]["velocity_tracking"])
        self.assertFalse(self.result()["gates"]["stopped"])
        self.support["load_and_lift"]["longest_continuous_ticks_by_stance_side"] = [
            8,
            7,
        ]
        self.support["load_and_lift"][
            "switches_between_runs_of_at_least_eight_ticks"
        ] = 1
        self.assertFalse(self.result()["gates"]["bilateral_single_support"])
        self.assertFalse(self.result()["gates"]["alternating_support"])


if __name__ == "__main__":
    unittest.main()
