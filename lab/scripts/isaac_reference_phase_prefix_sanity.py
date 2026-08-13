#!/usr/bin/env python3
"""Audit the frozen TRAIN-5 phase-prefix curriculum without optimization."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import traceback
from collections import Counter
from pathlib import Path
from typing import Any

from isaaclab.app import AppLauncher


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--training-profile", type=Path, required=True)
    parser.add_argument("--environment-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--num-envs", type=int, default=16)
    parser.add_argument("--reset-rounds-per-stage", type=int, default=8)
    parser.add_argument("--expected-evaluation-matrix-sha256", required=True)
    parser.add_argument("--output", type=Path, required=True)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.num_envs <= 0 or args.reset_rounds_per_stage <= 0:
        raise ValueError("invalid phase-prefix audit bounds")
    for path in (
        args.training_profile,
        args.environment_profile,
        args.descriptor,
        args.gate_report,
        args.usd,
    ):
        if not path.resolve().is_file():
            raise FileNotFoundError(path)
    if not args.corpus_root.resolve().is_dir():
        raise FileNotFoundError(args.corpus_root)
    if len(args.expected_evaluation_matrix_sha256) != 64:
        raise ValueError("invalid expected evaluation matrix hash")

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
            _select_curriculum_episode,
        )
        from next_lab.reference_ppo import (
            TinyReferencePpoProfile,
            _phase_prefix_count_for_iteration,
            _selection_episode_matrix_hash,
        )

        profile = TinyReferencePpoProfile.load(args.training_profile.resolve())
        document = profile.document
        schedule = document["phase_curriculum"]
        scope = document["scope"]
        execution = document["execution"]
        if (
            _sha256(args.environment_profile.resolve())
            != document["environment_profile_sha256"]
            or _sha256(args.descriptor.resolve()) != document["descriptor_sha256"]
            or _sha256(args.usd.resolve()) != document["usd_sha256"]
            or document["evaluation"]["episode_matrix"]
            != "fixed-vector-waves-v1"
        ):
            raise ValueError("phase-prefix audit input hash mismatch")

        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = args.num_envs
        cfg.sim.device = execution["device"]
        cfg.seed = int(execution["seed"])
        cfg.fixed_horizon_motor_ticks = int(scope["horizon_motor_ticks"])
        cfg.eligible_clip_ids = tuple(scope["eligible_clip_ids"])
        cfg.phase_randomization = True
        cfg.rng_run_root_hex = scope["rng_run_root_hex"]
        environment = NextEngineReferenceDirectEnv(
            cfg,
            descriptor_path=str(args.descriptor.resolve()),
            profile_path=str(args.environment_profile.resolve()),
            corpus_root=str(args.corpus_root.resolve()),
            gate_report_path=str(args.gate_report.resolve()),
        )
        frame_counts = tuple(clip.frame_count for clip in environment.reference_clips)
        run_root = bytes.fromhex(scope["rng_run_root_hex"])
        stages: list[dict[str, Any]] = []
        for stage in schedule["stages"]:
            count = _phase_prefix_count_for_iteration(
                schedule, int(stage["iteration_first"])
            )
            if count != int(stage["eligible_phase_count"]):
                raise RuntimeError("phase-prefix stage lookup mismatch")
            environment.set_curriculum_phase_prefix_count(count)
            environment.reset_episode_sequence()
            actual_rows: list[tuple[int, int, int]] = []
            for episode_ordinal in range(args.reset_rounds_per_stage):
                observation, _ = environment.reset()
                if not torch.isfinite(observation["policy"]).all():
                    raise RuntimeError("phase-prefix reset produced a non-finite observation")
                actual = list(
                    zip(
                        environment._clip_index.detach().cpu().tolist(),
                        environment._cursor.detach().cpu().tolist(),
                        environment._terminal_frame.detach().cpu().tolist(),
                        strict=True,
                    )
                )
                for vector_slot, row in enumerate(actual):
                    expected = _select_curriculum_episode(
                        run_root=run_root,
                        episode_ordinal=episode_ordinal,
                        vector_slot=vector_slot,
                        clip_frame_counts=frame_counts,
                        horizon_motor_ticks=int(scope["horizon_motor_ticks"]),
                        phase_prefix_count=count,
                    )
                    if row != expected:
                        raise RuntimeError("Isaac phase-prefix selection mismatch")
                    if row[1] >= count:
                        raise RuntimeError("Isaac selection escaped its phase prefix")
                actual_rows.extend(actual)

            exhaustive_rows = _selection_rows(
                run_root=run_root,
                clip_frame_counts=frame_counts,
                horizon_motor_ticks=int(scope["horizon_motor_ticks"]),
                phase_prefix_count=count,
                episode_ordinals=64,
                vector_slots=64,
            )
            repeat_rows = _selection_rows(
                run_root=run_root,
                clip_frame_counts=frame_counts,
                horizon_motor_ticks=int(scope["horizon_motor_ticks"]),
                phase_prefix_count=count,
                episode_ordinals=64,
                vector_slots=64,
            )
            if exhaustive_rows != repeat_rows:
                raise RuntimeError("phase-prefix selection is not reproducible")
            covered = {row[1] for row in exhaustive_rows}
            if covered != set(range(count)):
                raise RuntimeError("phase-prefix audit did not cover every eligible phase")
            stages.append(
                {
                    **stage,
                    "isaac_checked_selection_count": len(actual_rows),
                    "isaac_selection_sha256": _canonical_hash(actual_rows),
                    "exhaustive_probe_selection_count": len(exhaustive_rows),
                    "covered_phase_count": len(covered),
                    "minimum_start_phase": min(covered),
                    "maximum_start_phase": max(covered),
                    "exhaustive_selection_sha256": _canonical_hash(exhaustive_rows),
                    "repeat_byte_identical": True,
                }
            )

        environment.set_curriculum_phase_prefix_count(None)
        evaluation_counts: Counter[str] = Counter()
        evaluation_episodes = int(document["evaluation"]["episodes"])
        evaluation_num_envs = int(document["evaluation"]["num_envs"])
        for episode in range(evaluation_episodes):
            wave_ordinal = episode // evaluation_num_envs
            vector_slot = episode % evaluation_num_envs
            clip_index, start_frame, _ = _select_curriculum_episode(
                run_root=run_root,
                episode_ordinal=wave_ordinal,
                vector_slot=vector_slot,
                clip_frame_counts=frame_counts,
                horizon_motor_ticks=int(scope["horizon_motor_ticks"]),
            )
            selection_id = f"{scope['eligible_clip_ids'][clip_index]}:{start_frame}"
            evaluation_counts[selection_id] += 1
        selection_results = {
            selection_id: {"episodes": episodes}
            for selection_id, episodes in sorted(evaluation_counts.items())
        }
        evaluation_matrix_sha256 = _selection_episode_matrix_hash(selection_results)
        if evaluation_matrix_sha256 != args.expected_evaluation_matrix_sha256:
            raise RuntimeError("phase-prefix profile changed the fixed evaluation matrix")

        report = {
            "schema_version": 1,
            "check": "MOTOR-REFERENCE-ENV-P1-PHASE-PREFIX-CURRICULUM-SANITY",
            "status": "PASS",
            "claim": "OptimizerFreeCurriculumScheduleEvidenceOnly",
            "training_profile_sha256": profile.sha256,
            "environment_profile_sha256": _sha256(
                args.environment_profile.resolve()
            ),
            "descriptor_sha256": _sha256(args.descriptor.resolve()),
            "usd_sha256": _sha256(args.usd.resolve()),
            "tool_sha256": _sha256(Path(__file__).resolve()),
            "implementation_sha256": {
                "isaac_reference_env.py": _sha256(
                    Path(__file__).parents[1]
                    / "next_lab"
                    / "isaac_reference_env.py"
                ),
                "reference_ppo.py": _sha256(
                    Path(__file__).parents[1] / "next_lab" / "reference_ppo.py"
                ),
            },
            "repository": _repository_state(),
            "rng_run_root_hex": scope["rng_run_root_hex"],
            "horizon_motor_ticks": int(scope["horizon_motor_ticks"]),
            "stages": stages,
            "evaluation": {
                "episode_matrix": "fixed-vector-waves-v1",
                "episodes": evaluation_episodes,
                "vector_envs": evaluation_num_envs,
                "phase_prefix_disabled": True,
                "distinct_selection_count": len(evaluation_counts),
                "selection_episode_matrix_sha256": evaluation_matrix_sha256,
            },
            "optimizer_steps": 0,
            "training_runs": 0,
            "learned_policy_claim": False,
        }
        output = args.output.resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        temporary = output.with_suffix(output.suffix + ".tmp")
        temporary.write_text(
            json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        temporary.replace(output)
        print(json.dumps(report, indent=2, sort_keys=True), flush=True)
    except BaseException:
        traceback.print_exc()
        raise
    finally:
        if environment is not None:
            environment.close()
        if simulation_app is not None:
            simulation_app.close()


def _selection_rows(
    *,
    run_root: bytes,
    clip_frame_counts: tuple[int, ...],
    horizon_motor_ticks: int,
    phase_prefix_count: int,
    episode_ordinals: int,
    vector_slots: int,
) -> list[tuple[int, int, int]]:
    from next_lab.isaac_reference_env import _select_curriculum_episode

    return [
        _select_curriculum_episode(
            run_root=run_root,
            episode_ordinal=episode_ordinal,
            vector_slot=vector_slot,
            clip_frame_counts=clip_frame_counts,
            horizon_motor_ticks=horizon_motor_ticks,
            phase_prefix_count=phase_prefix_count,
        )
        for episode_ordinal in range(episode_ordinals)
        for vector_slot in range(vector_slots)
    ]


def _canonical_hash(value: Any) -> str:
    return hashlib.sha256(
        json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode(
            "utf-8"
        )
    ).hexdigest()


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _repository_state() -> dict[str, object]:
    root = Path(__file__).parents[2]
    commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    dirty = bool(
        subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    return {"root": str(root), "commit": commit, "dirty": dirty}


if __name__ == "__main__":
    main()
