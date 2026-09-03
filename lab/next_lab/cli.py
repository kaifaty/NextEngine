from __future__ import annotations

import argparse
import json
import os
import platform
import subprocess
import sys
from pathlib import Path
from typing import Any

import torch

from next_lab.biomechanics_material_lineage import BIOMECHANICS_TRANSLATOR_ID_V2
from next_lab.correspondence import evaluate_files
from next_lab.isaac_profile import IsaacProfile, doctor_report
from next_lab.motion_corpus import audit_motion_corpus_physx_poses, build_motion_corpus
from next_lab.motor_lab_client import (
    BOUNDED_STANDING_PROFILE_ID,
    CURRICULUM_LOCOMOTION_PROFILE_ID,
    FLAT_LOCOMOTION_PROFILE_ID,
    STANDING_PROFILE_ID,
)
from next_lab.physical_sound_glass_corpus import (
    default_recipe_path as default_physical_sound_glass_recipe_path,
)
from next_lab.physical_sound_glass_corpus import solve_controlled_glass_corpus
from next_lab.motor_mirror import (
    BIOMECHANICS_TRANSLATOR_ID,
    load_json,
    validate_biomechanics_descriptor,
    validate_descriptor,
    validate_golden,
)
from next_lab.reference_tracker import audit_reference_inputs, run_physx_baseline
from next_lab.smoke import SmokeConfig, run_smoke
from next_lab.trajectory_recorder import record_canonical_cpu_trajectories
from next_lab.usd_translation import translate_to_store


def _mps_probe() -> tuple[bool, str | None]:
    if not torch.backends.mps.is_built():
        return False, "torch build has no MPS backend"
    if not torch.backends.mps.is_available():
        return False, "MPS backend is not available on this host"
    try:
        value = torch.tensor([1.0], device="mps")
        result = (value * 2.0).cpu().item()
        if result != 2.0:
            return False, "MPS arithmetic probe returned an unexpected result"
    except Exception as error:  # capability diagnostics must survive backend faults
        return False, f"{type(error).__name__}: {error}"
    return True, None


def _nvidia_driver_version() -> str | None:
    try:
        result = subprocess.run(
            [
                "nvidia-smi",
                "--query-gpu=driver_version",
                "--format=csv,noheader",
            ],
            check=True,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (FileNotFoundError, subprocess.SubprocessError):
        return None
    return result.stdout.splitlines()[0].strip() if result.stdout.strip() else None


def _local_rtx_report() -> tuple[dict[str, Any], bool]:
    cuda_available = torch.cuda.is_available()
    device_memory = 0
    device_name = None
    if cuda_available:
        properties = torch.cuda.get_device_properties(0)
        device_memory = properties.total_memory
        device_name = properties.name
    driver_version = _nvidia_driver_version()
    checks = {
        "linux_x86_64": platform.system() == "Linux" and platform.machine() == "x86_64",
        "cuda_available": cuda_available,
        "driver_detected": driver_version is not None,
        "vram_at_least_8_gib": device_memory >= 8 * 1024**3,
    }
    report = {
        "system": platform.system(),
        "machine": platform.machine(),
        "cuda_available": cuda_available,
        "cuda_device": device_name,
        "cuda_memory_bytes": device_memory,
        "nvidia_driver_version": driver_version,
        "checks": checks,
    }
    return report, all(checks.values())


def doctor(profile: str) -> int:
    report: dict[str, Any] = {
        "schema_version": 1,
        "command": "doctor",
        "profile": profile,
        "python": platform.python_version(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "torch": torch.__version__,
    }

    if profile == "local-rtx":
        capability, available = _local_rtx_report()
        report.update(
            {
                "capability": capability,
                "status": "available" if available else "unavailable",
                "check": "local-rtx",
            }
        )
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0 if available else 3

    mps_available, mps_diagnostic = _mps_probe()
    cpu_available = torch.ones(1, device="cpu").item() == 1.0
    report.update(
        {
            "status": "available" if cpu_available else "unavailable",
            "check": "developer-host",
            "automatic_device": "mps" if mps_available else "cpu",
            "cpu_available": cpu_available,
            "mps_built": torch.backends.mps.is_built(),
            "mps_available": mps_available,
            "mps_diagnostic": mps_diagnostic,
        }
    )
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if cpu_available else 4


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(prog="python -m next_lab")
    commands = root.add_subparsers(dest="command", required=True)

    doctor_parser = commands.add_parser("doctor")
    doctor_parser.add_argument(
        "--profile",
        choices=["developer", "local-rtx"],
        default="developer",
    )

    smoke_parser = commands.add_parser("smoke")
    smoke_parser.add_argument(
        "--device", choices=["auto", "mps", "cpu"], default="auto"
    )
    smoke_parser.add_argument("--iterations", type=int, default=256)

    repository_root = Path(__file__).resolve().parents[2]
    mirror_parser = commands.add_parser("motor-mirror-check")
    mirror_parser.add_argument(
        "--golden",
        type=Path,
        default=repository_root / "lab/tests/fixtures/stage0_motor_mirror_v2.json",
    )
    mirror_parser.add_argument("--descriptor", type=Path)

    translate_parser = commands.add_parser("translate-body")
    translate_parser.add_argument("--descriptor", type=Path, required=True)
    translate_parser.add_argument("--store", type=Path)
    translate_parser.add_argument(
        "--golden",
        type=Path,
        default=repository_root / "lab/tests/fixtures/stage0_motor_mirror_v2.json",
    )

    isaac_parser = commands.add_parser("isaac-doctor")
    isaac_parser.add_argument(
        "--profile-file",
        type=Path,
        default=repository_root / "lab/profiles/isaac-lab-physx-stage0.v1.json",
    )

    correspondence_parser = commands.add_parser("correspondence")
    correspondence_parser.add_argument("--cpu", type=Path, required=True)
    correspondence_parser.add_argument("--gpu", type=Path, required=True)
    correspondence_parser.add_argument("--store", type=Path)

    recorder_parser = commands.add_parser("record-trajectories")
    recorder_parser.add_argument("--headless", type=Path, required=True)
    recorder_parser.add_argument(
        "--profile",
        choices=[
            BOUNDED_STANDING_PROFILE_ID,
            CURRICULUM_LOCOMOTION_PROFILE_ID,
            FLAT_LOCOMOTION_PROFILE_ID,
            STANDING_PROFILE_ID,
        ],
        default=CURRICULUM_LOCOMOTION_PROFILE_ID,
    )
    recorder_parser.add_argument("--slots", type=int, default=1)
    recorder_parser.add_argument("--episodes-per-slot", type=int, default=1)
    recorder_parser.add_argument("--run-root", required=True)
    recorder_parser.add_argument("--store", type=Path)
    recorder_parser.add_argument("--output-name")

    corpus_parser = commands.add_parser("motion-corpus-build")
    corpus_parser.add_argument(
        "--profile",
        type=Path,
        default=repository_root / "lab/profiles/humanoid-motion-corpus-cmu.v1.json",
    )
    corpus_parser.add_argument(
        "--descriptor",
        type=Path,
        default=repository_root
        / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json",
    )
    corpus_parser.add_argument("--dataset-root", type=Path, required=True)
    corpus_parser.add_argument("--store", type=Path)

    pose_audit_parser = commands.add_parser("motion-corpus-physx-pose-audit")
    pose_audit_parser.add_argument("--runner", type=Path, required=True)
    pose_audit_parser.add_argument("--descriptor", type=Path, required=True)
    pose_audit_parser.add_argument("--corpus-root", type=Path, required=True)
    pose_audit_parser.add_argument(
        "--partition", choices=["locomotion", "recovery"], required=True
    )
    pose_audit_parser.add_argument("--store", type=Path)

    reference_parser = commands.add_parser("reference-input-audit")
    reference_parser.add_argument(
        "--profile",
        type=Path,
        default=repository_root / "lab/profiles/humanoid-reference-tracker.v1.json",
    )
    reference_parser.add_argument("--descriptor", type=Path, required=True)
    reference_parser.add_argument("--corpus-root", type=Path, required=True)
    reference_parser.add_argument("--gate-report", type=Path, required=True)
    reference_parser.add_argument("--store", type=Path)

    baseline_parser = commands.add_parser("reference-physx-baseline")
    baseline_parser.add_argument("--runner", type=Path, required=True)
    baseline_parser.add_argument(
        "--profile",
        type=Path,
        default=repository_root / "lab/profiles/humanoid-reference-tracker.v1.json",
    )
    baseline_parser.add_argument("--corpus-root", type=Path, required=True)
    baseline_parser.add_argument("--gate-report", type=Path, required=True)
    baseline_parser.add_argument("--store", type=Path)
    baseline_parser.add_argument(
        "--split", choices=["train", "validation", "heldout"], required=True
    )
    baseline_parser.add_argument("--clip-id", required=True)
    baseline_parser.add_argument("--start-frame", type=int, default=0)
    baseline_parser.add_argument(
        "--baseline", choices=["zero_residual", "random_residual"], required=True
    )

    glass_corpus_parser = commands.add_parser(
        "physical-sound-glass-corpus-solve"
    )
    glass_corpus_parser.add_argument(
        "--profile",
        type=Path,
        default=default_physical_sound_glass_recipe_path(),
    )
    glass_corpus_parser.add_argument("--output", type=Path, required=True)

    return root


def _configured_store(value: Path | None) -> Path:
    candidate = value or (
        Path(path) if (path := os.environ.get("NEXTENGINE_TRAINING_STORE")) else None
    )
    if candidate is None:
        raise ValueError("--store or NEXTENGINE_TRAINING_STORE is required")
    repository_root = Path(__file__).resolve().parents[2]
    resolved = candidate.resolve()
    if resolved == repository_root or repository_root in resolved.parents:
        raise ValueError("training artifacts require an external configured store")
    return resolved


def main() -> int:
    arguments = parser().parse_args()
    if arguments.command == "doctor":
        return doctor(arguments.profile)
    if arguments.command == "smoke":
        config = SmokeConfig(device=arguments.device, iterations=arguments.iterations)
        return run_smoke(config)
    if arguments.command == "motor-mirror-check":
        golden = load_json(arguments.golden)
        descriptor_bytes = (
            arguments.descriptor.read_bytes() if arguments.descriptor else None
        )
        if golden.get("translator_id") in {
            BIOMECHANICS_TRANSLATOR_ID,
            BIOMECHANICS_TRANSLATOR_ID_V2,
        }:
            validate_biomechanics_descriptor(golden)
            if (
                descriptor_bytes is not None
                and descriptor_bytes != arguments.golden.read_bytes()
            ):
                raise ValueError(
                    "biomechanics descriptor does not match its exact golden bytes"
                )
        else:
            validate_golden(golden, descriptor_bytes)
        if descriptor_bytes is not None:
            descriptor = json.loads(descriptor_bytes)
            if descriptor.get("translator_id") in {
                BIOMECHANICS_TRANSLATOR_ID,
                BIOMECHANICS_TRANSLATOR_ID_V2,
            }:
                validate_biomechanics_descriptor(descriptor)
            else:
                validate_descriptor(descriptor)
        print(
            json.dumps({"check": "MODEL-MIRROR-GOLDEN", "status": "passed"}, indent=2)
        )
        return 0
    if arguments.command == "translate-body":
        descriptor_bytes = arguments.descriptor.read_bytes()
        descriptor = json.loads(descriptor_bytes)
        if descriptor.get("translator_id") in {
            BIOMECHANICS_TRANSLATOR_ID,
            BIOMECHANICS_TRANSLATOR_ID_V2,
        }:
            validate_biomechanics_descriptor(descriptor)
        else:
            validate_golden(load_json(arguments.golden), descriptor_bytes)
        manifest = translate_to_store(
            descriptor,
            _configured_store(arguments.store),
            Path(__file__).resolve().parents[2],
        )
        print(json.dumps(manifest, indent=2, sort_keys=True))
        return 0
    if arguments.command == "isaac-doctor":
        report, available = doctor_report(IsaacProfile.load(arguments.profile_file))
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0 if available else 3
    if arguments.command == "correspondence":
        report, path = evaluate_files(
            arguments.cpu,
            arguments.gpu,
            _configured_store(arguments.store),
        )
        summary = {
            "check": report["check"],
            "status": report["status"],
            "report": str(path),
        }
        print(json.dumps(summary, indent=2, sort_keys=True))
        return 0 if report["status"] == "passed" else 4
    if arguments.command == "record-trajectories":
        output = record_canonical_cpu_trajectories(
            headless_executable=arguments.headless,
            training_store=_configured_store(arguments.store),
            run_root=arguments.run_root,
            slots=arguments.slots,
            episodes_per_slot=arguments.episodes_per_slot,
            profile_id=arguments.profile,
            output_name=arguments.output_name,
        )
        print(
            json.dumps(
                {
                    "check": "MODEL-DATAPLANE-CPU-TRAJECTORY-V2",
                    "status": "passed",
                    "artifact": str(output),
                },
                indent=2,
                sort_keys=True,
            )
        )
        return 0
    if arguments.command == "motion-corpus-build":
        manifest, output = build_motion_corpus(
            profile_path=arguments.profile,
            descriptor_path=arguments.descriptor,
            dataset_root=arguments.dataset_root,
            output_store=_configured_store(arguments.store),
        )
        print(
            json.dumps(
                {
                    "check": "TRAIN-4-MOTION-CORPUS",
                    "status": manifest["status"],
                    "manifest_sha256": manifest["manifest_sha256"],
                    "output": str(output),
                },
                indent=2,
                sort_keys=True,
            )
        )
        return 0 if manifest["status"] == "VALIDATED" else 4
    if arguments.command == "motion-corpus-physx-pose-audit":
        report, output = audit_motion_corpus_physx_poses(
            runner=arguments.runner,
            descriptor_path=arguments.descriptor,
            corpus_root=arguments.corpus_root,
            output_store=_configured_store(arguments.store),
            partition=arguments.partition,
        )
        print(
            json.dumps(
                {
                    "check": report["check"],
                    "status": report["status"],
                    "pose_count": report["pose_count"],
                    "failed_pose_count": report["failed_pose_count"],
                    "output": str(output),
                },
                indent=2,
                sort_keys=True,
            )
        )
        return 0 if report["status"] == "PASS" else 4
    if arguments.command == "reference-input-audit":
        report, output = audit_reference_inputs(
            profile_path=arguments.profile,
            descriptor_path=arguments.descriptor,
            corpus_root=arguments.corpus_root,
            gate_report_path=arguments.gate_report,
            output_store=_configured_store(arguments.store),
        )
        print(
            json.dumps(
                {
                    "check": report["check"],
                    "status": report["status"],
                    "claim": report["claim"],
                    "output": str(output),
                },
                indent=2,
                sort_keys=True,
            )
        )
        return 0 if report["status"] == "PASS" else 4
    if arguments.command == "reference-physx-baseline":
        report, output = run_physx_baseline(
            runner=arguments.runner,
            profile_path=arguments.profile,
            corpus_root=arguments.corpus_root,
            gate_report_path=arguments.gate_report,
            output_store=_configured_store(arguments.store),
            split=arguments.split,
            clip_id=arguments.clip_id,
            start_frame=arguments.start_frame,
            baseline=arguments.baseline,
        )
        print(
            json.dumps(
                {
                    "check": report["check"],
                    "execution_status": report["execution_status"],
                    "reference_completed": report["reference_completed"],
                    "terminal_reason": report["terminal_reason"],
                    "output": str(output),
                },
                indent=2,
                sort_keys=True,
            )
        )
        return 0
    if arguments.command == "physical-sound-glass-corpus-solve":
        report, _ = solve_controlled_glass_corpus(
            profile_path=arguments.profile,
            output=arguments.output,
        )
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0
    raise AssertionError(f"unhandled command: {arguments.command}")


if __name__ == "__main__":
    sys.exit(main())
