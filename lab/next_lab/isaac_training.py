from __future__ import annotations

import hashlib
import json
import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any


PROFILE_SCHEMA_VERSION = 1
RUN_MANIFEST_SCHEMA = "nextengine.isaac-training-run.v1"
EVALUATION_MANIFEST_SCHEMA = "nextengine.isaac-policy-evaluation.v1"
CHECKPOINT_FILE_PATTERN = re.compile(r"model_(\d+)\.pt")


@dataclass(frozen=True)
class IsaacTrainingProfile:
    profile_id: str
    environment_profile_id: str
    num_envs: int
    steps_per_env: int
    iterations: int
    save_interval: int
    seed: int
    device: str
    min_free_gpu_memory_mib: int
    policy: dict[str, Any]
    algorithm: dict[str, Any]
    evaluation: dict[str, Any]
    profile_hash: str

    @classmethod
    def load(cls, path: Path) -> "IsaacTrainingProfile":
        value: dict[str, Any] = json.loads(path.read_text(encoding="utf-8"))
        if value.get("schema_version") != PROFILE_SCHEMA_VERSION:
            raise ValueError("unsupported Isaac training profile")
        if value.get("status") != "Proposed":
            raise ValueError("Isaac training profile must remain Proposed")
        profile = cls(
            profile_id=_nonempty_string(value.get("profile_id"), "profile_id"),
            environment_profile_id=_nonempty_string(
                value.get("environment_profile_id"), "environment_profile_id"
            ),
            num_envs=_positive_int(value.get("num_envs"), "num_envs"),
            steps_per_env=_positive_int(value.get("steps_per_env"), "steps_per_env"),
            iterations=_positive_int(value.get("iterations"), "iterations"),
            save_interval=_positive_int(value.get("save_interval"), "save_interval"),
            seed=_seed(value.get("seed")),
            device=_device(value.get("device")),
            min_free_gpu_memory_mib=_positive_int(
                value.get("min_free_gpu_memory_mib"), "min_free_gpu_memory_mib"
            ),
            policy=_mapping(value.get("policy"), "policy"),
            algorithm=_mapping(value.get("algorithm"), "algorithm"),
            evaluation=_mapping(value.get("evaluation"), "evaluation"),
            profile_hash=canonical_json_hash(value),
        )
        _validate_policy(profile.policy)
        _validate_algorithm(profile.algorithm)
        _validate_evaluation(profile.evaluation)
        return profile


@dataclass(frozen=True)
class ResolvedTrainingConfig:
    profile: IsaacTrainingProfile
    num_envs: int
    steps_per_env: int
    iterations: int
    save_interval: int
    seed: int
    device: str

    @classmethod
    def from_profile(
        cls,
        profile: IsaacTrainingProfile,
        *,
        num_envs: int | None = None,
        steps_per_env: int | None = None,
        iterations: int | None = None,
        save_interval: int | None = None,
        seed: int | None = None,
        device: str | None = None,
    ) -> "ResolvedTrainingConfig":
        return cls(
            profile=profile,
            num_envs=_positive_int(
                profile.num_envs if num_envs is None else num_envs, "num_envs"
            ),
            steps_per_env=_positive_int(
                profile.steps_per_env if steps_per_env is None else steps_per_env,
                "steps_per_env",
            ),
            iterations=_positive_int(
                profile.iterations if iterations is None else iterations, "iterations"
            ),
            save_interval=_positive_int(
                profile.save_interval if save_interval is None else save_interval,
                "save_interval",
            ),
            seed=_seed(profile.seed if seed is None else seed),
            device=_device(profile.device if device is None else device),
        )

    @property
    def run_root_hex(self) -> str:
        digest = hashlib.sha256()
        digest.update(b"nextengine.isaac-run-root.v1\0")
        digest.update(self.profile.profile_id.encode("utf-8"))
        digest.update(b"\0")
        digest.update(self.seed.to_bytes(8, "little", signed=False))
        return digest.hexdigest()

    def as_dict(self) -> dict[str, Any]:
        return {
            "profile_id": self.profile.profile_id,
            "profile_hash": self.profile.profile_hash,
            "environment_profile_id": self.profile.environment_profile_id,
            "num_envs": self.num_envs,
            "steps_per_env": self.steps_per_env,
            "iterations": self.iterations,
            "save_interval": self.save_interval,
            "seed": self.seed,
            "device": self.device,
            "run_root_hex": self.run_root_hex,
            "policy": self.profile.policy,
            "algorithm": self.profile.algorithm,
        }


def canonical_json_hash(value: Any) -> str:
    payload = json.dumps(
        value,
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def training_config_hash(
    config: ResolvedTrainingConfig,
    descriptor_sha256: str,
    usd_sha256: str,
) -> str:
    return canonical_json_hash(
        {
            "config": config.as_dict(),
            "descriptor_sha256": _hash(descriptor_sha256, "descriptor_sha256"),
            "usd_sha256": _hash(usd_sha256, "usd_sha256"),
        }
    )


def require_external_path(
    path: Path,
    repository_root: Path,
    *,
    label: str,
    must_exist: bool = True,
) -> Path:
    resolved = path.resolve()
    repository = repository_root.resolve()
    if resolved == repository or repository in resolved.parents:
        raise ValueError(f"{label} must be outside the repository")
    if must_exist and not resolved.exists():
        raise FileNotFoundError(f"{label} does not exist: {resolved}")
    return resolved


def atomic_write_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    os.replace(temporary, path)


def validate_resume_checkpoint(
    checkpoint: Path,
    expected_training_config_hash: str,
) -> dict[str, Any]:
    manifest = validate_closed_checkpoint(checkpoint)
    if manifest.get("training_config_hash") != expected_training_config_hash:
        raise ValueError("resume checkpoint training configuration mismatch")
    return manifest


def validate_closed_checkpoint(checkpoint: Path) -> dict[str, Any]:
    checkpoint = checkpoint.resolve()
    if not checkpoint.is_file():
        raise FileNotFoundError(f"checkpoint does not exist: {checkpoint}")
    manifest_path = checkpoint.parent / "run-manifest.json"
    if not manifest_path.is_file():
        raise ValueError("checkpoint has no run-manifest.json")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema") != RUN_MANIFEST_SCHEMA or manifest.get("status") != "completed":
        raise ValueError("checkpoint belongs to an incomplete or unsupported run")
    actual_hash = sha256_file(checkpoint)
    records = manifest.get("checkpoints")
    if not isinstance(records, list) or not any(
        record.get("file") == checkpoint.name and record.get("sha256") == actual_hash
        for record in records
        if isinstance(record, dict)
    ):
        raise ValueError("checkpoint hash is not closed by its run manifest")
    return manifest


def latest_closed_checkpoint(runs_root: Path) -> Path:
    """Return the newest hash-closed checkpoint from completed external runs."""
    candidates: list[tuple[str, int, Path]] = []
    for manifest_path in runs_root.resolve().glob("*/run-manifest.json"):
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if manifest.get("schema") != RUN_MANIFEST_SCHEMA or manifest.get("status") != "completed":
            continue
        records = manifest.get("checkpoints")
        if not isinstance(records, list):
            continue
        for record in records:
            if not isinstance(record, dict):
                continue
            name = record.get("file")
            match = CHECKPOINT_FILE_PATTERN.fullmatch(name) if isinstance(name, str) else None
            if match is None or Path(name).name != name:
                continue
            checkpoint = manifest_path.parent / name
            try:
                validate_closed_checkpoint(checkpoint)
            except (FileNotFoundError, ValueError):
                continue
            candidates.append((manifest_path.parent.name, int(match.group(1)), checkpoint))
    if not candidates:
        raise FileNotFoundError(f"no closed checkpoint found below: {runs_root.resolve()}")
    return max(candidates, key=lambda value: (value[0], value[1]))[2].resolve()


def closed_checkpoint_history(checkpoint: Path) -> list[Path]:
    """Return every hash-closed checkpoint in the selected checkpoint's run."""
    selected = checkpoint.resolve()
    manifest = validate_closed_checkpoint(selected)
    records = manifest.get("checkpoints")
    if not isinstance(records, list):
        raise ValueError("checkpoint run manifest has no checkpoint records")
    candidates: list[tuple[int, Path]] = []
    for record in records:
        if not isinstance(record, dict):
            continue
        name = record.get("file")
        match = CHECKPOINT_FILE_PATTERN.fullmatch(name) if isinstance(name, str) else None
        if match is None or Path(name).name != name:
            continue
        candidate = selected.parent / name
        try:
            validate_closed_checkpoint(candidate)
        except (FileNotFoundError, ValueError):
            continue
        candidates.append((int(match.group(1)), candidate.resolve()))
    if not any(candidate == selected for _, candidate in candidates):
        raise ValueError("selected checkpoint is absent from its closed run history")
    return [candidate for _, candidate in sorted(candidates)]


def validate_checkpoint_artifacts(
    parent: dict[str, Any],
    profile: IsaacTrainingProfile,
    descriptor: Path,
    usd: Path,
) -> None:
    """Require a checkpoint to use the selected profile and exact mirror artifacts."""
    training = parent.get("training_config")
    artifacts = parent.get("artifacts")
    if not isinstance(training, dict) or training.get("profile_hash") != profile.profile_hash:
        raise ValueError("checkpoint training profile does not match selected profile")
    if not isinstance(artifacts, dict):
        raise ValueError("checkpoint manifest has no artifact closure")
    expected = {
        "descriptor": sha256_file(descriptor),
        "usd": sha256_file(usd),
    }
    for name, digest in expected.items():
        record = artifacts.get(name)
        if not isinstance(record, dict) or record.get("sha256") != digest:
            raise ValueError(f"checkpoint {name} does not match selected artifact")


def checkpoint_records(run_dir: Path) -> list[dict[str, Any]]:
    return [
        {
            "file": path.name,
            "bytes": path.stat().st_size,
            "sha256": sha256_file(path),
        }
        for path in sorted(run_dir.glob("model_*.pt"))
        if path.is_file()
    ]


def parse_gpu_memory_csv(value: str) -> dict[str, int | str]:
    fields = [field.strip() for field in value.strip().split(",")]
    if len(fields) != 4:
        raise ValueError("unexpected nvidia-smi memory response")
    index, name, total, free = fields
    try:
        return {
            "index": int(index),
            "name": name,
            "memory_total_mib": int(total),
            "memory_free_mib": int(free),
        }
    except ValueError as error:
        raise ValueError("invalid nvidia-smi memory response") from error


def equal_episode_quota(episodes: int, num_envs: int) -> int:
    episodes = _positive_int(episodes, "evaluation.episodes")
    num_envs = _positive_int(num_envs, "evaluation.num_envs")
    quota, remainder = divmod(episodes, num_envs)
    if quota == 0 or remainder != 0:
        raise ValueError(
            "evaluation episodes must be a positive multiple of num_envs "
            "for unbiased per-slot sampling"
        )
    return quota


def _validate_policy(value: dict[str, Any]) -> None:
    _positive_number(value.get("init_noise_std"), "policy.init_noise_std")
    for field in ("actor_hidden_dims", "critic_hidden_dims"):
        dimensions = value.get(field)
        if not isinstance(dimensions, list) or not dimensions:
            raise ValueError(f"{field} must be a non-empty array")
        for dimension in dimensions:
            _positive_int(dimension, field)
    if value.get("activation") not in {"elu", "relu", "tanh"}:
        raise ValueError("unsupported policy activation")
    for field in ("actor_obs_normalization", "critic_obs_normalization"):
        if not isinstance(value.get(field), bool):
            raise ValueError(f"{field} must be boolean")


def _validate_algorithm(value: dict[str, Any]) -> None:
    for field in ("num_learning_epochs", "num_mini_batches"):
        _positive_int(value.get(field), f"algorithm.{field}")
    for field in (
        "value_loss_coef",
        "clip_param",
        "learning_rate",
        "gamma",
        "lam",
        "desired_kl",
        "max_grad_norm",
    ):
        _positive_number(value.get(field), f"algorithm.{field}")
    entropy = value.get("entropy_coef")
    if not isinstance(entropy, (int, float)) or entropy < 0:
        raise ValueError("algorithm.entropy_coef must be non-negative")
    if not isinstance(value.get("use_clipped_value_loss"), bool):
        raise ValueError("algorithm.use_clipped_value_loss must be boolean")
    if value.get("schedule") not in {"adaptive", "fixed"}:
        raise ValueError("unsupported learning-rate schedule")


def _validate_evaluation(value: dict[str, Any]) -> None:
    equal_episode_quota(value.get("episodes"), value.get("num_envs"))
    _positive_int(value.get("max_steps"), "evaluation.max_steps")
    _nonnegative_int(
        value.get("episode_ordinal_start", 0),
        "evaluation.episode_ordinal_start",
    )
    seeds = value.get("seeds")
    if not isinstance(seeds, list) or not seeds:
        raise ValueError("evaluation.seeds must be a non-empty array")
    for seed in seeds:
        _seed(seed)


def _mapping(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{label} must be an object")
    return value


def _nonempty_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{label} must be a non-empty string")
    return value


def _positive_int(value: Any, label: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise ValueError(f"{label} must be a positive integer")
    return value


def _nonnegative_int(value: Any, label: str) -> int:
    if (
        not isinstance(value, int)
        or isinstance(value, bool)
        or not 0 <= value < 2**63
    ):
        raise ValueError(f"{label} must be a non-negative 63-bit integer")
    return value


def _positive_number(value: Any, label: str) -> float:
    if not isinstance(value, (int, float)) or isinstance(value, bool) or value <= 0:
        raise ValueError(f"{label} must be positive")
    return float(value)


def _seed(value: Any) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not 0 <= value < 2**63:
        raise ValueError("seed must be an unsigned 63-bit integer")
    return value


def _device(value: Any) -> str:
    if not isinstance(value, str) or not value.startswith("cuda:"):
        raise ValueError("Isaac training device must be an explicit cuda:N device")
    suffix = value.removeprefix("cuda:")
    if not suffix.isdigit():
        raise ValueError("Isaac training device must be an explicit cuda:N device")
    return value


def _hash(value: Any, label: str) -> str:
    if not isinstance(value, str) or len(value) != 64:
        raise ValueError(f"{label} must be a SHA-256 hex string")
    try:
        bytes.fromhex(value)
    except ValueError as error:
        raise ValueError(f"{label} must be a SHA-256 hex string") from error
    return value
