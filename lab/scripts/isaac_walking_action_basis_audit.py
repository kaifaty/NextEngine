#!/usr/bin/env python3
"""Replay the canonical walking action-basis tape in the Isaac GPU mirror."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import subprocess
import sys
from pathlib import Path

import numpy as np

from next_lab.isaac_training import atomic_write_json, require_external_path
from next_lab.walking_action_basis import (
    ACTION_BASIS_CASES,
    ACTION_BASIS_PROFILE_ID,
    Q1_30_ONE,
    action_tape_sha256,
    evaluate_action_basis,
    validate_action_basis_tape,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    from isaaclab.app import AppLauncher

    parser = argparse.ArgumentParser()
    parser.add_argument("--audit-worker", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--action-tape", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=44)
    parser.add_argument("--canonical-damping-probe", action="store_true",
                        help="Report-only counterfactual: apply CPU compiler's 0.05 link damping")
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def supervise_audit(command: list[str], output: Path) -> int:
    """Keep the gate status outside Kit, whose shutdown may exit Python directly."""
    if output.exists() or output.with_suffix(".npz").exists():
        raise ValueError("audit supervisor requires fresh output paths")
    completed = subprocess.run(command, check=False)
    if completed.returncode != 0:
        return completed.returncode if completed.returncode > 0 else 1
    try:
        report = json.loads(output.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return 4
    if not isinstance(report, dict):
        return 4
    gates = report.get("gates")
    return 0 if (
        report.get("status") == "passed"
        and isinstance(gates, dict)
        and bool(gates)
        and all(value is True for value in gates.values())
    ) else 4


def main() -> int:
    args = parse_args()
    descriptor_path = require_external_path(
        args.descriptor, REPOSITORY_ROOT, label="walking descriptor"
    )
    usd_path = require_external_path(
        args.usd, REPOSITORY_ROOT, label="walking humanoid USD"
    )
    cpu_path = require_external_path(
        args.action_tape, REPOSITORY_ROOT, label="canonical CPU action-basis tape"
    )
    output = require_external_path(
        args.output,
        REPOSITORY_ROOT,
        label="walking action-basis Isaac report",
        must_exist=False,
    )
    if output.suffix != ".json" or output.exists() or output.with_suffix(".npz").exists():
        raise ValueError("audit output must be a new .json path with a new .npz companion")
    if not args.audit_worker:
        return supervise_audit(
            [sys.executable, str(Path(__file__).resolve()), *sys.argv[1:], "--audit-worker"],
            output,
        )
    from isaaclab.app import AppLauncher

    descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
    with np.load(cpu_path, allow_pickle=False) as archive:
        cpu = {key: archive[key] for key in archive.files}
    if str(cpu["profile_id"].item()) != ACTION_BASIS_PROFILE_ID:
        raise ValueError("CPU action tape profile identity mismatch")
    if tuple(cpu["case_ids"].tolist()) != ACTION_BASIS_CASES:
        raise ValueError("CPU action tape case order mismatch")
    tape = cpu["action_raw"]
    validate_action_basis_tape(tape)
    if cpu["root_position_m"].shape[:2] != tape.shape[:2] or np.any(cpu["done"]):
        raise ValueError("CPU control must complete the frozen tape without a terminal")
    case_count, motor_steps, action_width = tape.shape
    if case_count != len(ACTION_BASIS_CASES) or action_width != 23:
        raise ValueError("CPU action tape shape mismatch")
    profile = next(
        (
            candidate
            for candidate in descriptor["environment_profiles"]
            if candidate["profile_id"] == ACTION_BASIS_PROFILE_ID
        ),
        None,
    )
    if profile is None:
        raise ValueError("descriptor does not contain the walking action-basis profile")
    if not np.array_equal(
        cpu["manifest_hash"],
        np.frombuffer(bytes.fromhex(profile["manifest_hash"]), dtype=np.uint8),
    ):
        raise ValueError("CPU/descriptor manifest identity mismatch")
    if not np.array_equal(
        cpu["action_layout_hash"],
        np.frombuffer(bytes.fromhex(profile["action_layout_hash"]), dtype=np.uint8),
    ):
        raise ValueError("CPU/descriptor action layout identity mismatch")

    os.environ["NEXTENGINE_HUMANOID_USD"] = str(usd_path)
    simulation_app = None
    wrapped = None
    try:
        args.device = args.device or "cuda:0"
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        import torch
        from isaaclab_rl.rsl_rl import RslRlVecEnvWrapper

        from next_lab.isaac_env import (
            NextEngineHumanoidDirectEnv,
            NextEngineHumanoidDirectEnvCfg,
        )

        class RecordingEnvironment(NextEngineHumanoidDirectEnv):
            def _get_dones(self):
                result = super()._get_dones()
                self.audit_action = self._action.clone()
                self.audit_targets = self._applied_target.clone()
                self.audit_contacts = self._canonical_facts()[-1].clone()
                return result

        env_cfg = NextEngineHumanoidDirectEnvCfg()
        env_cfg.scene.num_envs = case_count
        env_cfg.seed = args.seed
        env_cfg.run_root_hex = action_tape_sha256(tape)
        env_cfg.environment_profile_id = ACTION_BASIS_PROFILE_ID
        env_cfg.episode_ordinal_start = 0
        env_cfg.sim.device = args.device
        if args.canonical_damping_probe:
            import isaaclab.sim as sim_utils
            env_cfg.asset.spawn.rigid_props = sim_utils.RigidBodyPropertiesCfg(
                linear_damping=0.05, angular_damping=0.05,
            )
        environment = RecordingEnvironment(
            env_cfg, descriptor_path=str(descriptor_path)
        )
        wrapped = RslRlVecEnvWrapper(environment, clip_actions=1.0)
        wrapped.reset()

        root_position = np.empty((case_count, motor_steps, 3), dtype=np.float64)
        root_velocity = np.empty_like(root_position)
        joint_position = np.empty((case_count, motor_steps, action_width), dtype=np.float64)
        contacts = np.empty((case_count, motor_steps, 2), dtype=np.bool_)
        done = np.zeros((case_count, motor_steps), dtype=np.bool_)
        targets = np.empty_like(tape)
        terminal_facts = {}
        exact_action = True
        with torch.inference_mode():
            for tick in range(motor_steps):
                expected = torch.from_numpy(tape[:, tick]).to(args.device)
                actions = expected.to(torch.float32) / float(Q1_30_ONE)
                _, _, dones, _ = wrapped.step(actions)
                exact_action &= bool(torch.equal(environment.audit_action, expected))
                done[:, tick] = dones.detach().cpu().numpy()
                joint_position[:, tick] = (
                    environment.last_step_joint_position.cpu().numpy() / 1_000_000.0
                )
                root_position[:, tick] = (
                    environment.last_step_root_position.cpu().numpy() / 1_000_000.0
                )
                root_velocity[:, tick] = (
                    environment.last_step_root_linear_velocity.cpu().numpy() / 1_000_000.0
                )
                contacts[:, tick] = environment.audit_contacts.cpu().numpy()
                targets[:, tick] = environment.audit_targets.cpu().numpy()
                if np.any(done[:, tick]):
                    for slot in np.flatnonzero(done[:, tick]):
                        terminal_facts[ACTION_BASIS_CASES[slot]] = {
                            name: bool(getattr(environment, "last_step_failure_" + name)[slot])
                            for name in ("joint_safety", "self_collision", "hard_impact", "forbidden_contact", "fall")
                        }
                    # Preserve the terminal commit; never mix a reset episode into this tape.
                    break

        root_position, root_velocity, joint_position, contacts, done, targets = (
            value[:, :tick + 1] for value in
            (root_position, root_velocity, joint_position, contacts, done, targets)
        )
        gate = evaluate_action_basis(
            contact_occupancy=contacts,
            root_position_m=root_position,
            done=done,
        )
        paired_metrics = {
            "joint_position_rmse_rad": _rmse(
                cpu["joint_position_rad"][:, :tick + 1], joint_position
            ),
            "root_position_rmse_m": _rmse(cpu["root_position_m"][:, :tick + 1], root_position),
            "root_velocity_rmse_mps": _rmse(
                cpu["root_velocity_mps"][:, :tick + 1], root_velocity
            ),
            "contact_occupancy_agreement": float(
                np.mean(cpu["contact_occupancy"][:, :tick + 1] == contacts)
            ),
            "done_agreement": float(np.mean(cpu["done"][:, :tick + 1] == done)),
        }
        gates = dict(gate["gates"])
        gates["exact_q1_30_action_consumption"] = exact_action
        status = "passed" if all(gates.values()) else "failed"
        report = {
            "schema_version": 1,
            **gate,
            "status": status,
            "plane": "isaac-gpu-mirror" if args.device.startswith("cuda") else "isaac-cpu-diagnostic",
            "canonical_damping_probe": args.canonical_damping_probe,
            "gates": gates,
            "action_tape_sha256": action_tape_sha256(tape),
            "cpu_trajectory_sha256": _sha256(cpu_path),
            "descriptor_sha256": _sha256(descriptor_path),
            "usd_sha256": _sha256(usd_path),
            "paired_metrics": paired_metrics,
            "model_mirror_p1_claim": "not-evaluated-sample-floor-not-met",
            "device": args.device,
            "seed": args.seed,
            "training_runs": 0,
            "optimizer_steps": 0,
            "tool_sha256": _sha256(Path(__file__).resolve()),
            "environment_source_sha256": _sha256(REPOSITORY_ROOT / "lab/next_lab/isaac_env.py"),
            "tape_module_sha256": _sha256(REPOSITORY_ROOT / "lab/next_lab/walking_action_basis.py"),
            "observed_motor_steps": tick + 1,
            "terminal_facts": terminal_facts,
            "paired_metrics_scope": "observed prefix including first terminal commit; incomplete tape fails the horizon gate",
        }
        trajectory_path = output.with_suffix(".npz")
        output.parent.mkdir(parents=True, exist_ok=True)
        np.savez_compressed(
            trajectory_path, action_raw=tape, applied_targets_raw=targets,
            root_position_m=root_position, root_velocity_mps=root_velocity,
            joint_position_rad=joint_position, contact_occupancy=contacts,
            done=done, observed_motor_steps=np.asarray(tick + 1),
        )
        report["trajectory_path"] = str(trajectory_path)
        report["trajectory_sha256"] = _sha256(trajectory_path)
        atomic_write_json(output, report)
        print(json.dumps(report, indent=2, sort_keys=True), flush=True)
    finally:
        if wrapped is not None:
            wrapped.close()
        if simulation_app is not None:
            simulation_app.close()
    return 0 if status == "passed" else 4


def _rmse(left: np.ndarray, right: np.ndarray) -> float:
    difference = left.astype(np.float64) - right.astype(np.float64)
    return math.sqrt(float(np.mean(np.square(difference))))


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


if __name__ == "__main__":
    raise SystemExit(main())
