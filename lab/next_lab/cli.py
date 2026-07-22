from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
from pathlib import Path
from typing import Any

import torch

from next_lab.smoke import SmokeConfig, run_smoke


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


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _memory_bytes() -> int:
    try:
        return int(os.sysconf("SC_PAGE_SIZE")) * int(os.sysconf("SC_PHYS_PAGES"))
    except (OSError, ValueError):
        return 0


def _linux_distribution() -> tuple[str | None, str | None]:
    if platform.system() != "Linux":
        return None, None
    try:
        release = platform.freedesktop_os_release()
    except OSError:
        return None, None
    return release.get("ID"), release.get("VERSION_ID")


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
    repository_root = Path(__file__).resolve().parents[2]
    lock_path = repository_root / "lab/uv.lock"
    attestation_path = repository_root / ".local/training/rtx-capability.json"
    attestation: dict[str, Any] = {}
    if attestation_path.is_file():
        try:
            loaded = json.loads(attestation_path.read_text(encoding="utf-8"))
            if isinstance(loaded, dict):
                attestation = loaded
        except (OSError, json.JSONDecodeError):
            attestation = {}

    distribution, distribution_version = _linux_distribution()
    cuda_available = torch.cuda.is_available()
    device_memory = 0
    device_name = None
    if cuda_available:
        properties = torch.cuda.get_device_properties(0)
        device_memory = properties.total_memory
        device_name = properties.name
    driver_version = _nvidia_driver_version()
    lock_hash = _sha256(lock_path)
    memory_bytes = _memory_bytes()
    checks = {
        "linux_x86_64": platform.system() == "Linux" and platform.machine() == "x86_64",
        "supported_ubuntu": distribution == "ubuntu"
        and distribution_version in {"22.04", "24.04"},
        "ram_at_least_32_gib": memory_bytes >= 32 * 1024**3,
        "cuda_available": cuda_available,
        "vram_at_least_16_gib": device_memory >= 16 * 1024**3,
        "driver_pinned": driver_version is not None
        and attestation.get("driver_version") == driver_version,
        "toolchain_lock_pinned": attestation.get("uv_lock_sha256") == lock_hash,
        "python_pinned": attestation.get("python_version") == platform.python_version(),
        "torch_pinned": attestation.get("torch_version") == torch.__version__,
        "offline_cache_verified": attestation.get("offline_cache_verified") is True,
        "provenance_review_passed": attestation.get("provenance_review_passed") is True,
        "license_review_passed": attestation.get("license_review_passed") is True,
    }
    report = {
        "distribution": distribution,
        "distribution_version": distribution_version,
        "ram_bytes": memory_bytes,
        "cuda_available": cuda_available,
        "cuda_device": device_name,
        "cuda_memory_bytes": device_memory,
        "nvidia_driver_version": driver_version,
        "uv_lock_sha256": lock_hash,
        "attestation_manifest_loaded": bool(attestation),
        "checks": checks,
    }
    return report, all(checks.values())


def doctor(profile: str) -> int:
    mps_available, mps_diagnostic = _mps_probe()
    cpu_available = torch.ones(1, device="cpu").item() == 1.0
    report: dict[str, Any] = {
        "schema_version": 1,
        "command": "doctor",
        "profile": profile,
        "python": platform.python_version(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "torch": torch.__version__,
        "cpu_available": cpu_available,
        "mps_built": torch.backends.mps.is_built(),
        "mps_available": mps_available,
        "mps_diagnostic": mps_diagnostic,
    }

    if profile == "local-rtx":
        capability, eligible = _local_rtx_report()
        report.update(
            {
                "capability": capability,
                "status": "PASS" if eligible else "AwaitingCapability",
                "gate": "TRAIN-RTX-01",
            }
        )
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0 if eligible else 3

    report.update(
        {
            "status": "PASS" if cpu_available else "FAIL",
            "gate": "TRAIN-MAC-P0-doctor",
            "automatic_device": "mps" if mps_available else "cpu",
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
    smoke_parser.add_argument("--device", choices=["auto", "mps", "cpu"], default="auto")
    smoke_parser.add_argument("--iterations", type=int, default=256)
    return root


def main() -> int:
    arguments = parser().parse_args()
    if arguments.command == "doctor":
        return doctor(arguments.profile)
    if arguments.command == "smoke":
        config = SmokeConfig(device=arguments.device, iterations=arguments.iterations)
        return run_smoke(config)
    raise AssertionError(f"unhandled command: {arguments.command}")


if __name__ == "__main__":
    sys.exit(main())
