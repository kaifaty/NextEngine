from __future__ import annotations

import argparse
import json
import platform
import subprocess
import sys
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
