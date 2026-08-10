from __future__ import annotations

import importlib.metadata
import json
import platform
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class IsaacProfile:
    profile_id: str
    isaac_lab_version: str
    isaac_sim_version: str
    python_version: str
    engine_physx_version: str
    backend: str
    environments: int

    @classmethod
    def load(cls, path: Path) -> "IsaacProfile":
        value: dict[str, Any] = json.loads(path.read_text(encoding="utf-8"))
        if value.get("schema_version") != 1 or value.get("status") != "Proposed":
            raise ValueError("unsupported Isaac profile")
        profile = cls(
            profile_id=value["profile_id"],
            isaac_lab_version=value["isaac_lab_version"],
            isaac_sim_version=value["isaac_sim_version"],
            python_version=value["python_version"],
            engine_physx_version=value["engine_physx_version"],
            backend=value["backend"],
            environments=value["environments"],
        )
        if profile.backend != "physx" or profile.environments != 4_096:
            raise ValueError("Isaac profile is not the Stage 0 PhysX mirror")
        return profile


def doctor_report(profile: IsaacProfile) -> tuple[dict[str, Any], bool]:
    installed = {}
    for distribution in ("isaaclab", "isaacsim"):
        try:
            installed[distribution] = importlib.metadata.version(distribution)
        except importlib.metadata.PackageNotFoundError:
            installed[distribution] = None
    driver = None
    try:
        result = subprocess.run(
            ["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"],
            check=True,
            capture_output=True,
            text=True,
            timeout=10,
        )
        driver = result.stdout.splitlines()[0].strip()
    except (FileNotFoundError, subprocess.SubprocessError, IndexError):
        pass
    checks = {
        "linux_x86_64": platform.system() == "Linux" and platform.machine() == "x86_64",
        "python": platform.python_version().startswith(profile.python_version + "."),
        "isaac_lab": installed["isaaclab"] == profile.isaac_lab_version,
        "isaac_sim": installed["isaacsim"] == profile.isaac_sim_version,
        "nvidia_driver": driver is not None,
    }
    return (
        {
            "schema_version": 1,
            "profile_id": profile.profile_id,
            "status": "available" if all(checks.values()) else "unavailable",
            "checks": checks,
            "installed": installed,
            "nvidia_driver_version": driver,
        },
        all(checks.values()),
    )
