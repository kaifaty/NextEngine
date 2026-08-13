#!/usr/bin/env python3
"""Exercise directed hard-safety terminal branches in the Isaac reference env."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import traceback
from pathlib import Path

from isaaclab.app import AppLauncher


SCENARIOS = ("hard_impact", "self_collision", "forbidden_contact", "world_bounds", "fall")
EXPECTED_REASONS = {
    "hard_impact": 3,
    "self_collision": 4,
    "forbidden_contact": 5,
    "world_bounds": 6,
    "fall": 7,
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--clip-id", default="cmu104-start-right")
    parser.add_argument("--start-frame", type=int, default=159)
    parser.add_argument("--horizon", type=int, default=11)
    parser.add_argument("--output", type=Path, required=True)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    for path in (args.descriptor, args.profile, args.gate_report, args.usd):
        if not path.resolve().is_file():
            raise FileNotFoundError(path)
    if not args.corpus_root.resolve().is_dir():
        raise FileNotFoundError(args.corpus_root)
    os.environ["NEXTENGINE_HUMANOID_USD"] = str(args.usd.resolve())
    simulation_app = None
    environment = None
    try:
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        import torch

        from next_lab.isaac_reference_env import (
            NextEngineReferenceDirectEnv,
            NextEngineReferenceDirectEnvCfg,
            _terminal_reason_tensor,
        )

        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = len(SCENARIOS)
        cfg.sim.device = args.device
        cfg.seed = 120_812
        cfg.fixed_clip_id = args.clip_id
        cfg.fixed_start_frame = args.start_frame
        cfg.fixed_horizon_motor_ticks = args.horizon
        environment = NextEngineReferenceDirectEnv(
            cfg,
            descriptor_path=str(args.descriptor.resolve()),
            profile_path=str(args.profile.resolve()),
            corpus_root=str(args.corpus_root.resolve()),
            gate_report_path=str(args.gate_report.resolve()),
        )
        environment.reset()

        root_state = environment.robot.data.root_state_w.clone()
        joint_position = environment.robot.data.joint_pos.clone()
        joint_velocity = environment.robot.data.joint_vel.clone()

        # A downward root impulse from an already supported pose must exceed a
        # sole/body impact budget in the actual rigid-contact view.
        root_state[0, 7:10] = 0.0
        root_state[0, 9] = -5.0

        # The pre-reserve corpus failure is reconstructed intentionally: zero
        # both hip-roll coordinates at the phase where the thighs converged.
        left_hip_roll = environment._dof_joint_ids[1]
        right_hip_roll = environment._dof_joint_ids[7]
        joint_position[1, left_hip_roll] = 0.0
        joint_position[1, right_hip_roll] = 0.0

        # Turn the body sideways so the pelvis can touch without the legs being
        # driven through the plane. A shallow downward pelvis contact is above
        # the material threshold and below the hard-impact budget.
        half = math.sqrt(0.5)
        root_state[2, 3:7] = torch.tensor(
            [half, half, 0.0, 0.0], device=environment.device
        )
        root_state[2, 2] = 0.13
        root_state[2, 7:13] = 0.0
        root_state[2, 9] = -0.1
        joint_velocity[2] = 0.0

        # World bounds use environment-local Engine X. Lift the body so this
        # branch is isolated from ground contact while retaining finite state.
        root_state[3, 0] = environment.scene.env_origins[3, 0] + 91.0
        root_state[3, 2] = 2.0
        root_state[3, 7:13] = 0.0
        joint_velocity[3] = 0.0

        # A 90-degree finite root tilt at safe height must select fall.
        root_state[4, 2] = 2.0
        root_state[4, 3:7] = torch.tensor(
            [half, half, 0.0, 0.0], device=environment.device
        )
        root_state[4, 7:13] = 0.0
        joint_velocity[4] = 0.0

        env_ids = torch.arange(len(SCENARIOS), device=environment.device)
        environment.robot.write_root_state_to_sim(root_state, env_ids)
        environment.robot.write_joint_state_to_sim(
            joint_position,
            joint_velocity,
            env_ids=env_ids,
        )
        zero = torch.zeros(
            (len(SCENARIOS), 23), dtype=torch.float32, device=environment.device
        )
        _, _, terminated, truncated, _ = environment.step(zero)

        isolated = {
            name: torch.zeros(len(SCENARIOS), dtype=torch.bool, device=environment.device)
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
        isolated["hard_impact"][0] = True
        isolated["self_collision"][1] = True
        isolated["forbidden_contact"][2] = True
        isolated["world_bounds"][3] = True
        isolated["fall"][4] = True
        isolated_reasons = _terminal_reason_tensor(**isolated)

        results = []
        for index, scenario in enumerate(SCENARIOS):
            hard_pairs = _active_pairs(
                environment, environment.last_step_hard_impact_pair_mask[index]
            )
            self_pairs = _active_pairs(
                environment, environment.last_step_self_collision_pair_mask[index]
            )
            forbidden_pairs = _active_pairs(
                environment, environment.last_step_forbidden_contact_pair_mask[index]
            )
            actual_reason = int(environment.last_step_terminal_reason[index].item())
            branch_name = {
                "hard_impact": "hard_impact",
                "self_collision": "self_collision",
                "forbidden_contact": "forbidden_contact",
                "world_bounds": "world_bounds",
                "fall": "fall",
            }[scenario]
            result = {
                "scenario": scenario,
                "expected_terminal_reason": EXPECTED_REASONS[scenario],
                "actual_terminal_reason": actual_reason,
                "isolated_terminal_reason": int(isolated_reasons[index].item()),
                "terminated": bool(terminated[index].item()),
                "truncated": bool(truncated[index].item()),
                "hard_impact": bool(
                    environment.last_step_failure_hard_impact[index].item()
                ),
                "self_collision": bool(
                    environment.last_step_failure_self_collision[index].item()
                ),
                "forbidden_contact": bool(
                    environment.last_step_failure_forbidden_contact[index].item()
                ),
                "world_bounds": bool(
                    environment.last_step_failure_world_bounds[index].item()
                ),
                "fall": bool(environment.last_step_failure_fall[index].item()),
                "joint_safety": bool(
                    environment.last_step_failure_joint_safety[index].item()
                ),
                "hard_rom": bool(
                    environment.last_step_failure_hard_rom[index].item()
                ),
                "non_finite": bool(
                    environment.last_step_failure_non_finite[index].item()
                ),
                "hard_impact_pairs": hard_pairs,
                "self_collision_pairs": self_pairs,
                "forbidden_contact_pairs": forbidden_pairs,
            }
            observed = {
                name: torch.tensor([bool(result[name])], device=environment.device)
                for name in (
                    "non_finite",
                    "hard_rom",
                    "joint_safety",
                    "hard_impact",
                    "self_collision",
                    "forbidden_contact",
                    "world_bounds",
                    "fall",
                )
            }
            observed["tracking_lost"] = environment.last_step_failure_tracking_lost[
                index : index + 1
            ]
            observed["success"] = environment.last_step_success[index : index + 1]
            observed_reason = int(_terminal_reason_tensor(**observed)[0].item())
            result["physical_expected_branch"] = branch_name
            result["physical_expected_branch_observed"] = bool(result[branch_name])
            result["physical_priority_reduction_reason"] = observed_reason
            result["status"] = (
                "PASS"
                if result["terminated"]
                and not result["truncated"]
                and result["physical_expected_branch_observed"]
                and actual_reason == observed_reason
                and result["isolated_terminal_reason"] == EXPECTED_REASONS[scenario]
                else "FAIL"
            )
            results.append(result)

        status = "PASS" if all(item["status"] == "PASS" for item in results) else "FAIL"
        report = {
            "schema_version": 1,
            "check": "MOTOR-REFERENCE-ENV-P1-ISAAC-DIRECTED-TERMINALS",
            "status": status,
            "claim": "IsaacDirectedHardSafetyTerminalSemanticsOnly",
            "profile_sha256": _sha256(args.profile.resolve()),
            "descriptor_sha256": _sha256(args.descriptor.resolve()),
            "usd_sha256": _sha256(args.usd.resolve()),
            "gate_report_sha256": _sha256(args.gate_report.resolve()),
            "tool_sha256": _sha256(Path(__file__).resolve()),
            "clip_id": args.clip_id,
            "start_frame": args.start_frame,
            "physics_velocity_limit_basis_points": environment.physics_velocity_limit_basis_points,
            "scenarios": results,
            "training_runs": 0,
            "optimizer_steps": 0,
            "learned_policy_claim": False,
        }
        output = args.output.resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
        temporary = output.with_suffix(output.suffix + ".tmp")
        temporary.write_text(payload, encoding="utf-8")
        temporary.replace(output)
        print(json.dumps(report, indent=2, sort_keys=True), flush=True)
        if status != "PASS":
            raise RuntimeError("directed terminal audit failed")
    except BaseException:
        traceback.print_exc()
        raise
    finally:
        if environment is not None:
            environment.close()
        if simulation_app is not None:
            simulation_app.close()


def _active_pairs(environment: object, mask: object) -> list[str]:
    import torch

    indices = torch.nonzero(mask, as_tuple=False).flatten().tolist()
    return [environment.contact_pair_ids[index] for index in indices]


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


if __name__ == "__main__":
    main()
