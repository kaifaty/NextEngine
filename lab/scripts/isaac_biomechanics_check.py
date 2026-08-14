"""Load the exact biomechanics USD in Isaac and verify its articulation closure."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import traceback
from pathlib import Path

from isaaclab.app import AppLauncher


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--ground-usd", type=Path, required=True)
    parser.add_argument("--steps", type=int, default=4)
    parser.add_argument("--output", type=Path)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.steps <= 0:
        raise ValueError("steps must be positive")
    descriptor_path = args.descriptor.resolve()
    usd_path = args.usd.resolve()
    ground_usd_path = args.ground_usd.resolve()
    descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))

    simulation_app = None
    try:
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        import isaaclab.sim as sim_utils
        import torch
        from isaaclab.actuators import ImplicitActuatorCfg
        from isaaclab.assets import Articulation, ArticulationCfg
        from isaaclab.sim import SimulationContext
        from next_lab.isaac_env import isaac_actuator_limits_from_descriptor
        from next_lab.motor_mirror import validate_current_biomechanics_descriptor
        from next_lab.usd_translation import validate_translation_bundle

        validate_current_biomechanics_descriptor(descriptor)
        translation_manifest = validate_translation_bundle(
            descriptor,
            usd_path.with_name("translation-manifest.json"),
            humanoid_usd_path=usd_path,
            ground_usd_path=ground_usd_path,
        )
        limits = isaac_actuator_limits_from_descriptor(descriptor)
        sim = SimulationContext(
            sim_utils.SimulationCfg(dt=1.0 / 240.0, device=args.device)
        )
        ground_cfg = sim_utils.UsdFileCfg(usd_path=str(ground_usd_path))
        ground_cfg.func("/World/ground", ground_cfg)
        cfg = ArticulationCfg(
            prim_path="/World/Humanoid",
            spawn=sim_utils.UsdFileCfg(
                usd_path=str(usd_path),
                activate_contact_sensors=True,
                articulation_props=sim_utils.ArticulationRootPropertiesCfg(
                    enabled_self_collisions=True,
                    solver_position_iteration_count=8,
                    solver_velocity_iteration_count=4,
                ),
            ),
            actuators={
                "engine_effort": ImplicitActuatorCfg(
                    joint_names_expr=[".*"],
                    stiffness=0.0,
                    damping=0.0,
                    effort_limit_sim=limits["effort_limit_sim"],
                    velocity_limit_sim=limits["velocity_limit_sim"],
                )
            },
        )
        cfg.spawn.func(cfg.prim_path, cfg.spawn)
        robot = Articulation(cfg)
        sim.reset()
        canonical_joint_names = [
            _prim(actuator["joint_id"]) for actuator in descriptor["actuators"]
        ]
        joint_ids, resolved_joint_names = robot.find_joints(
            canonical_joint_names, preserve_order=True
        )
        if resolved_joint_names != canonical_joint_names or len(joint_ids) != 23:
            raise RuntimeError(
                "Isaac articulation does not preserve canonical DoF order"
            )
        if robot.num_bodies != 24 or robot.num_joints != 23:
            raise RuntimeError(
                "Isaac articulation topology does not match the descriptor"
            )
        initial_root = robot.data.root_state_w.clone()
        initial_joint = robot.data.joint_pos[:, joint_ids].clone()
        zero_effort = torch.zeros_like(initial_joint)
        maximum_joint_motion = 0.0
        for _ in range(args.steps):
            robot.set_joint_effort_target(zero_effort, joint_ids=joint_ids)
            robot.write_data_to_sim()
            sim.step(render=False)
            robot.update(sim.get_physics_dt())
            for name, value in (
                ("root_state", robot.data.root_state_w),
                ("joint_position", robot.data.joint_pos),
                ("joint_velocity", robot.data.joint_vel),
            ):
                if not torch.isfinite(value).all():
                    raise RuntimeError(f"non-finite Isaac {name}")
            maximum_joint_motion = max(
                maximum_joint_motion,
                float(
                    torch.max(
                        torch.abs(robot.data.joint_pos[:, joint_ids] - initial_joint)
                    ).item()
                ),
            )
        root_motion = float(
            torch.max(
                torch.abs(robot.data.root_state_w[:, :7] - initial_root[:, :7])
            ).item()
        )
        if not math.isfinite(root_motion) or not math.isfinite(maximum_joint_motion):
            raise RuntimeError("non-finite Isaac motion diagnostic")
        report = {
            "schema_version": 1,
            "check": "MOTOR-BIOMECHANICS-ISAAC-P1",
            "status": "PASS",
            "claim": "IsaacUsdLoadAndTopologyOnly",
            "descriptor_sha256": _sha256(descriptor_path),
            "usd_sha256": _sha256(usd_path),
            "ground_usd_sha256": _sha256(ground_usd_path),
            "translation_manifest_sha256": _sha256(
                usd_path.with_name("translation-manifest.json")
            ),
            "tool_sha256": _sha256(Path(__file__).resolve()),
            "body_schema_hash": descriptor["body_schema_hash"],
            "compiled_descriptor_hash": descriptor["compiled_descriptor_hash"],
            "material_lineage_hash": descriptor["material_lineage_hash"],
            "translation_manifest": translation_manifest,
            "device": args.device,
            "body_count": robot.num_bodies,
            "joint_count": robot.num_joints,
            "canonical_joint_names": resolved_joint_names,
            "physics_steps": args.steps,
            "maximum_root_pose_motion": root_motion,
            "maximum_joint_motion_radians": maximum_joint_motion,
            "training_runs": 0,
            "optimizer_steps": 0,
            "learned_policy_claim": False,
        }
        if args.output is not None:
            output = args.output.resolve()
            output.parent.mkdir(parents=True, exist_ok=True)
            payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
            temporary = output.with_suffix(output.suffix + ".tmp")
            temporary.write_text(payload, encoding="utf-8")
            temporary.replace(output)
        print(json.dumps(report, indent=2, sort_keys=True), flush=True)
    except BaseException:
        traceback.print_exc()
        raise
    finally:
        if simulation_app is not None:
            simulation_app.close()


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _prim(identifier: str) -> str:
    return "".join(
        character if character.isalnum() else "_" for character in identifier
    )


if __name__ == "__main__":
    main()
