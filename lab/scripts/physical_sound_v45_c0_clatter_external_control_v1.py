#!/usr/bin/env python3
"""Run the V45 C0 Clatter empirical-prior control outside the repository."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import struct
import subprocess
import tempfile
import wave
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_v45_t0_recipe_v3_v1 as t0

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v45-c0-profile.v1"
INVENTORY_SCHEMA = "nextengine.experimental-physical-sound-v45-c0-inventory.v1"
PRIOR_SCHEMA = "nextengine.experimental-physical-sound-v45-c0-prior.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v45-c0-access.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v45-c0-report.v1"

PROFILE_PATH = "lab/profiles/physical-sound-v45-c0-clatter-external-control.v1.json"
OWNER_PATH = "lab/scripts/physical_sound_v45_c0_clatter_external_control_v1.py"
PROTOCOL_PATH = (
    "docs/development/physical-sound-v45-c0-clatter-external-control-protocol-"
    "2026-09-03.md"
)
DECISION = "ClatterExternalControlAvailable"
FAILURE = "ExternalPriorUnavailableOrIncompatible"
CLAIM = (
    "EXTERNAL_CLATTER_EMPIRICAL_MODAL_PRIOR_CONTROL / NO_REAL_PARENT_MATERIAL_"
    "TRUTH_TRAINING_VALIDATOR_PROTECTED_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)

MATERIALS = (
    "cardboard",
    "ceramic",
    "fabric",
    "glass",
    "leather",
    "metal",
    "paper",
    "plastic_hard",
    "plastic_soft_foam",
    "rubber",
    "stone",
    "wood_hard",
    "wood_medium",
    "wood_soft",
)
SOURCE_RELATIVE = Path("Clatter/Clatter.Core/Data/ImpactMaterials")
OBSERVED_FIELDS = (
    "modal.base_frequency_millihz",
    "modal.frequency_ratio_ppm",
    "modal.mode_count",
    "modal.participation_ppm",
    "modal.rt60_milliseconds",
)
FORBIDDEN_ZERO_COUNTERS = (
    "model_values_read",
    "network_requests",
    "protected_values_read",
    "real_audio_values_read",
    "runtime_values_read",
    "validator_values_read",
)
AUTHORITY = {
    "admission_authority": False,
    "candidate_training_authority": False,
    "empirical_prior_control_authority": True,
    "material_truth_authority": False,
    "product_authority": False,
    "public_contract": False,
    "real_acoustic_parent_credit": 0,
    "runtime_consumer_allowed": False,
    "validator_authority": False,
}
MAX_FILE_BYTES = 4 * 1024 * 1024
MAX_ARRAY_VALUES = 16_384


class ClatterControlError(RuntimeError):
    """The external source or C0 publication violates the frozen protocol."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True, allow_nan=False)
            + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise ClatterControlError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ClatterControlError(f"{context} must be an object")
    return value


def require_string(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        raise ClatterControlError(f"{context} must be a non-empty string")
    return value


def require_int(value: Any, context: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        raise ClatterControlError(f"{context} must be an integer")
    return value


def read_regular(path: Path, context: str, maximum: int = 2 * 1024 * 1024) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ClatterControlError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise ClatterControlError(f"{context} has an invalid byte count")
    return path.read_bytes()


def read_canonical_json(path: Path, context: str) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, context)
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ClatterControlError(f"{context} is not valid UTF-8 JSON") from error
    document = require_dict(value, context)
    if data != canonical_json(document):
        raise ClatterControlError(f"{context} is not canonical JSON")
    return data, document


def validate_binding(value: Any, context: str) -> dict[str, Any]:
    binding = require_dict(value, context)
    if set(binding) != {"bytes", "path", "sha256"}:
        raise ClatterControlError(f"{context} fields changed")
    relative = Path(require_string(binding["path"], f"{context}.path"))
    count = require_int(binding["bytes"], f"{context}.bytes")
    digest = require_string(binding["sha256"], f"{context}.sha256")
    if relative.is_absolute() or ".." in relative.parts or count <= 0:
        raise ClatterControlError(f"{context} path or byte count is invalid")
    if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
        raise ClatterControlError(f"{context} hash is invalid")
    data = read_regular(repository_root() / relative, context)
    if len(data) != count or sha256_bytes(data) != digest:
        raise ClatterControlError(f"{context} binding mismatch")
    return binding


def expected_filenames() -> list[str]:
    return sorted(f"{material}_{size}_mm.bytes" for material in MATERIALS for size in range(6))


def validate_profile(profile: dict[str, Any]) -> None:
    if set(profile) != {
        "authority", "claim", "decision", "decoder", "dependency_bindings",
        "expected_source", "projection", "renderer", "resources", "schema",
    }:
        raise ClatterControlError("profile fields changed")
    if profile["schema"] != PROFILE_SCHEMA or profile["claim"] != CLAIM or profile["decision"] != DECISION:
        raise ClatterControlError("profile identity changed")
    if profile["authority"] != AUTHORITY:
        raise ClatterControlError("authority changed")
    bindings = profile["dependency_bindings"]
    if not isinstance(bindings, list) or not bindings:
        raise ClatterControlError("dependency bindings must be non-empty")
    checked = [validate_binding(item, f"dependency {index}") for index, item in enumerate(bindings)]
    paths = [item["path"] for item in checked]
    if paths != sorted(set(paths)) or OWNER_PATH not in paths or PROTOCOL_PATH not in paths:
        raise ClatterControlError("dependencies must be sorted, unique and bind owner/protocol")
    if profile["decoder"] != {
        "arrays": ["cf_hz", "op_db", "rt60_seconds"],
        "byte_order": "little",
        "count_type": "int32",
        "header_bytes": 12,
        "mode_rows_used": 10,
        "require_equal_counts": True,
        "scalar_type": "ieee754_binary64",
        "trailing_bytes_allowed": False,
    }:
        raise ClatterControlError("decoder semantics changed")
    if profile["projection"] != {
        "frequency_unit": "millihertz",
        "observed_fields": list(OBSERVED_FIELDS),
        "participation": "normalize_10_pow_op_db_over_20_to_ppm",
        "ratio_scale": 1_000_000,
        "rounding": "nearest_ties_to_even",
        "source_lane": "empirical_prior",
        "t0_profile_sha256": "6cf7e1fe6c141ac324b9896defcadc033047fcbd7d33bc8f6b245d99cb0c9994",
    }:
        raise ClatterControlError("projection semantics changed")
    if profile["renderer"] != {
        "amplitude_ppm": 500_000,
        "contact_pulse_samples": 48,
        "frequency_relative_std_ppm": 100_000,
        "frequency_retry_floor_hz": 20,
        "mode_count": 10,
        "onset_level_std_millidb": 10_000,
        "output_encoding": "mono_pcm16le_wav",
        "prng": "splitmix64_box_muller_v1",
        "render_frames": 48_000,
        "resonance_ppm": 1_000_000,
        "root_seed": 4_503_599_627_370_517,
        "rt60_relative_std_ppm": 100_000,
        "rt60_retry_floor_microseconds": 1_000,
        "sample_rate_hz": 48_000,
    }:
        raise ClatterControlError("renderer semantics changed")
    if profile["resources"] != {
        "expected_payload_files": 84,
        "max_array_values": MAX_ARRAY_VALUES,
        "max_file_bytes": MAX_FILE_BYTES,
        "max_output_bytes": 32 * 1024 * 1024,
        "max_retry_per_draw": 128,
    }:
        raise ClatterControlError("resource envelope changed")
    source = require_dict(profile["expected_source"], "expected source")
    if set(source) != {
        "commit", "filename_count", "filename_rule", "semantic_files",
        "tree_root_sha256", "url", "usage_expression",
    }:
        raise ClatterControlError("source fields changed")
    if source["commit"] != "79cac6cbe3f7c452ba28b56c7da4a0124ad04806":
        raise ClatterControlError("source commit changed")
    if source["url"] != "https://github.com/alters-mit/clatter":
        raise ClatterControlError("source URL changed")
    if source["filename_count"] != 84 or source["filename_rule"] != "14_materials_x_6_size_buckets":
        raise ClatterControlError("source inventory changed")
    if source["tree_root_sha256"] != "8230a6192f189806b899a9113c08fefaf458e9e7f26322e2fc6fe5434a4c065e":
        raise ClatterControlError("source tree root changed")
    semantic = source["semantic_files"]
    if not isinstance(semantic, list) or not semantic:
        raise ClatterControlError("source semantic bindings must be non-empty")
    semantic_paths = [require_string(item.get("path"), "semantic path") for item in semantic if isinstance(item, dict)]
    if len(semantic_paths) != len(semantic) or semantic_paths != sorted(set(semantic_paths)):
        raise ClatterControlError("source semantic paths must be sorted and unique")


def git_head(source: Path) -> str:
    try:
        result = subprocess.run(
            ["git", "-C", str(source), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise ClatterControlError(f"cannot resolve external source commit: {error}") from error
    return result.stdout.strip()


def validate_external_source(source_value: Path, profile: dict[str, Any]) -> tuple[Path, list[dict[str, Any]], dict[str, int]]:
    source = source_value.resolve(strict=True)
    if source.is_relative_to(repository_root().resolve()):
        raise ClatterControlError("Clatter source must remain outside the repository")
    expected = profile["expected_source"]
    if git_head(source) != expected["commit"]:
        raise ClatterControlError("external source commit mismatch")
    semantic_rows = []
    semantic_bytes = 0
    for item in expected["semantic_files"]:
        if set(item) != {"bytes", "path", "sha256"}:
            raise ClatterControlError("source semantic binding fields changed")
        path = source / item["path"]
        data = read_regular(path, f"source semantic file {item['path']}", MAX_FILE_BYTES)
        if len(data) != item["bytes"] or sha256_bytes(data) != item["sha256"]:
            raise ClatterControlError(f"source semantic binding mismatch: {item['path']}")
        semantic_bytes += len(data)
        semantic_rows.append(dict(item))
    payload_root = source / SOURCE_RELATIVE
    actual_paths = sorted(payload_root.glob("*.bytes"), key=lambda path: path.name.encode())
    names = [path.name for path in actual_paths]
    if names != expected_filenames():
        raise ClatterControlError("impact-material payload inventory mismatch")
    rows = []
    tree_lines = bytearray()
    payload_bytes = 0
    for path in actual_paths:
        data = read_regular(path, f"payload {path.name}", MAX_FILE_BYTES)
        digest = sha256_bytes(data)
        tree_lines.extend(f"{digest}  {path.name}\n".encode())
        payload_bytes += len(data)
        rows.append({"bytes": len(data), "filename": path.name, "sha256": digest})
    if sha256_bytes(bytes(tree_lines)) != expected["tree_root_sha256"]:
        raise ClatterControlError("impact-material tree root mismatch")
    return payload_root, rows, {
        "parameter_bytes_hashed": payload_bytes,
        "parameter_files_hashed": len(rows),
        "source_code_bytes_hashed": semantic_bytes,
        "source_code_files_hashed": len(semantic_rows),
    }


def decode_payload(data: bytes, context: str, maximum_values: int = MAX_ARRAY_VALUES) -> tuple[list[float], list[float], list[float]]:
    if len(data) < 12:
        raise ClatterControlError(f"{context} is truncated before the header")
    counts = struct.unpack_from("<iii", data, 0)
    if any(count < 10 or count > maximum_values for count in counts):
        raise ClatterControlError(f"{context} array count is outside the envelope")
    if len(set(counts)) != 1:
        raise ClatterControlError(f"{context} array counts differ")
    expected = 12 + 8 * sum(counts)
    if len(data) != expected:
        raise ClatterControlError(f"{context} byte count does not match its header")
    offset = 12
    arrays = []
    for count in counts:
        values = list(struct.unpack_from(f"<{count}d", data, offset))
        offset += 8 * count
        if any(not math.isfinite(value) for value in values):
            raise ClatterControlError(f"{context} contains a non-finite value")
        arrays.append(values)
    cf, op, rt = arrays
    if any(value <= 0 for value in cf) or any(value <= 0 for value in rt):
        raise ClatterControlError(f"{context} frequency or RT60 is not positive")
    return cf, op, rt


def nearest_even(value: float) -> int:
    if not math.isfinite(value):
        raise ClatterControlError("cannot quantize a non-finite value")
    return int(round(value))


def largest_remainder(weights: list[float], total: int = 1_000_000) -> list[int]:
    if not weights or any(not math.isfinite(value) or value < 0 for value in weights):
        raise ClatterControlError("participation weights are invalid")
    denominator = math.fsum(weights)
    if not denominator > 0:
        raise ClatterControlError("participation weights have zero mass")
    exact = [value * total / denominator for value in weights]
    result = [math.floor(value) for value in exact]
    missing = total - sum(result)
    for index in sorted(range(len(weights)), key=lambda item: (-(exact[item] - result[item]), item))[:missing]:
        result[index] += 1
    return result


def project_prior(filename: str, cf: list[float], op: list[float], rt: list[float]) -> tuple[dict[str, Any], dict[str, Any]]:
    rows = sorted(
        [(cf[index], op[index], rt[index], index) for index in range(10)],
        key=lambda item: (item[0], item[3]),
    )
    frequency_millihz = [nearest_even(item[0] * 1000) for item in rows]
    base = frequency_millihz[0]
    ratios = [nearest_even(value * 1_000_000 / base) for value in frequency_millihz]
    rt60 = [nearest_even(item[2] * 1000) for item in rows]
    participation = largest_remainder([10.0 ** (item[1] / 20.0) for item in rows])
    recipe_id = filename.removesuffix("_mm.bytes")
    raw = {
        "descriptor_mask": ["geometry_extents", "material_class"],
        "heads": {
            "excitation": {
                "contact_duration_samples": 48,
                "impact_gain_ppm": 500_000,
                "location_modifier_ppm": [0] * 8,
                "onset_samples": 0,
                "support_modifier_ppm": [0] * 4,
                "uncertainty_ppm": 1_000_000,
            },
            "modal": {
                "base_frequency_millihz": base,
                "frequency_ratio_ppm": ratios,
                "mode_count": 10,
                "participation_ppm": participation,
                "rt60_milliseconds": rt60,
                "uncertainty_ppm": [1_000_000] * 10,
            },
            "radiation": {
                "band_energy_ppm": [0] * 24,
                "residual_envelope_ppm": [0] * 16,
                "spectral_tilt_millidb_per_octave": 0,
                "uncertainty_ppm": 1_000_000,
            },
            "support": {"ood_score_ppm": 1_000_000},
        },
        "recipe_id": recipe_id,
        "source_lane": "empirical_prior",
    }
    try:
        projected = t0.project_recipe(raw)
        target = t0.make_target(projected, list(OBSERVED_FIELDS))
    except t0.RecipeContractError as error:
        raise ClatterControlError(f"Recipe V3 projection failed for {filename}: {error}") from error
    modal = projected["heads"]["modal"]
    if (
        modal["base_frequency_millihz"] != base
        or modal["frequency_ratio_ppm"] != ratios
        or modal["rt60_milliseconds"] != rt60
        or modal["participation_ppm"] != participation
    ):
        raise ClatterControlError(f"Recipe V3 would clamp or rewrite source values: {filename}")
    return projected, target


class SplitMixNormal:
    def __init__(self, seed: int) -> None:
        self.state = seed & ((1 << 64) - 1)
        self.spare: float | None = None

    def next_u64(self) -> int:
        self.state = (self.state + 0x9E3779B97F4A7C15) & ((1 << 64) - 1)
        value = self.state
        value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & ((1 << 64) - 1)
        value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & ((1 << 64) - 1)
        return value ^ (value >> 31)

    def uniform_open(self) -> float:
        return ((self.next_u64() >> 11) + 0.5) / (1 << 53)

    def normal(self) -> float:
        if self.spare is not None:
            value = self.spare
            self.spare = None
            return value
        radius = math.sqrt(-2.0 * math.log(self.uniform_open()))
        angle = 2.0 * math.pi * self.uniform_open()
        self.spare = radius * math.sin(angle)
        return radius * math.cos(angle)


def seeded_value(rng: SplitMixNormal, mean: float, stddev: float, floor: float, retries: int) -> float:
    for _ in range(retries):
        value = mean + stddev * rng.normal()
        if math.isfinite(value) and value >= floor:
            return value
    raise ClatterControlError("seeded parameter retry limit exceeded")


def render_control(
    filename: str,
    cf: list[float],
    op: list[float],
    rt: list[float],
    renderer: dict[str, Any],
) -> bytes:
    seed_material = f"{renderer['root_seed']}:{filename}".encode()
    seed = int.from_bytes(hashlib.sha256(seed_material).digest()[:8], "little")
    rng = SplitMixNormal(seed)
    frames = renderer["render_frames"]
    rate = renderer["sample_rate_hz"]
    seconds = np.arange(frames, dtype=np.float64) / float(rate)
    response = np.zeros(frames, dtype=np.float64)
    retries = 128
    for index in range(10):
        frequency = seeded_value(rng, cf[index], cf[index] / 10.0, 20.0, retries)
        level = op[index] + 10.0 * rng.normal()
        decay = seeded_value(rng, rt[index], rt[index] / 10.0, 0.001, retries)
        amplitude = 10.0 ** (level / 20.0)
        response += amplitude * np.cos(2.0 * math.pi * frequency * seconds) * np.power(10.0, -3.0 * seconds / decay)
    force = np.sin(np.linspace(0.0, math.pi, renderer["contact_pulse_samples"], dtype=np.float64))
    samples = np.convolve(response, force, mode="full")[:frames]
    peak = float(np.max(np.abs(samples)))
    if not math.isfinite(peak) or peak <= 0 or not np.all(np.isfinite(samples)):
        raise ClatterControlError(f"render produced invalid output: {filename}")
    samples *= (renderer["amplitude_ppm"] / 1_000_000.0) / peak
    pcm = np.rint(np.clip(samples, -1.0, 1.0) * 32767.0).astype("<i2")
    with tempfile.SpooledTemporaryFile(max_size=256 * 1024) as stream:
        with wave.open(stream, "wb") as writer:
            writer.setnchannels(1)
            writer.setsampwidth(2)
            writer.setframerate(rate)
            writer.setnframes(frames)
            writer.writeframes(pcm.tobytes())
        stream.seek(0)
        return stream.read()


def prepare_output(output: Path) -> Path:
    resolved = output.resolve(strict=False)
    if resolved.is_relative_to(repository_root().resolve()):
        raise ClatterControlError("output must remain outside the repository")
    if output.exists() or output.is_symlink():
        raise ClatterControlError("output path already exists")
    output.parent.mkdir(parents=True, exist_ok=True)
    return Path(tempfile.mkdtemp(prefix=f".{output.name}.staging-", dir=output.parent))


def run(profile_path: Path, source: Path, output: Path) -> None:
    profile_bytes, profile = read_canonical_json(profile_path, "profile")
    validate_profile(profile)
    payload_root, inventory_rows, integrity = validate_external_source(source, profile)
    stage = prepare_output(output)
    decoded_values = 0
    prior_rows = []
    try:
        renders = stage / "renders"
        renders.mkdir()
        for inventory in inventory_rows:
            filename = inventory["filename"]
            data = read_regular(payload_root / filename, f"payload {filename}", MAX_FILE_BYTES)
            cf, op, rt = decode_payload(data, filename)
            decoded_values += len(cf) + len(op) + len(rt)
            recipe, target = project_prior(filename, cf, op, rt)
            wav = render_control(filename, cf, op, rt, profile["renderer"])
            wav_name = filename.removesuffix("_mm.bytes") + ".wav"
            (renders / wav_name).write_bytes(wav)
            prior_rows.append({
                "filename": filename,
                "recipe": recipe,
                "recipe_sha256": sha256_bytes(canonical_json(recipe)),
                "target": target,
                "target_sha256": sha256_bytes(canonical_json(target)),
                "wav_bytes": len(wav),
                "wav_path": f"renders/{wav_name}",
                "wav_sha256": sha256_bytes(wav),
            })
        access_counters = {
            **integrity,
            "generated_pcm_samples": len(prior_rows) * profile["renderer"]["render_frames"],
            "model_values_read": 0,
            "network_requests": 0,
            "numeric_payload_files_decoded": len(prior_rows),
            "numeric_payload_values_decoded": decoded_values,
            "protected_values_read": 0,
            "real_audio_values_read": 0,
            "runtime_values_read": 0,
            "validator_values_read": 0,
        }
        inventory = {
            "commit": profile["expected_source"]["commit"],
            "payloads": inventory_rows,
            "schema": INVENTORY_SCHEMA,
            "semantic_files": profile["expected_source"]["semantic_files"],
            "tree_root_sha256": profile["expected_source"]["tree_root_sha256"],
        }
        prior = {
            "observed_fields": list(OBSERVED_FIELDS),
            "profile_sha256": sha256_bytes(profile_bytes),
            "rows": prior_rows,
            "schema": PRIOR_SCHEMA,
            "source_lane": "empirical_prior",
        }
        gates = {
            "all_84_payloads_integrity_bound": len(inventory_rows) == 84,
            "all_84_priors_project_without_clamp": len(prior_rows) == 84,
            "all_84_seeded_wavs_canonical": all(row["wav_bytes"] == 96_044 for row in prior_rows),
            "forbidden_access_zero": all(access_counters[key] == 0 for key in FORBIDDEN_ZERO_COUNTERS),
            "no_real_parent_or_project_credit": True,
            "recipe_v3_profile_bound": profile["projection"]["t0_profile_sha256"] == sha256_bytes((repository_root() / t0.PROFILE_PATH).read_bytes()),
        }
        report = {
            "authority": AUTHORITY,
            "claim": CLAIM,
            "decision": DECISION if all(gates.values()) else FAILURE,
            "gates": gates,
            "measured": {
                "decoded_numeric_values": decoded_values,
                "material_classes": len(MATERIALS),
                "payload_files": len(prior_rows),
                "rendered_pcm_samples": access_counters["generated_pcm_samples"],
                "size_buckets_per_material": 6,
            },
            "next_action": "freeze_s0_nisr_and_vibraverse_bounded_preflights_without_using_c0_as_real_or_validator_evidence",
            "schema": REPORT_SCHEMA,
        }
        if not all(gates.values()):
            raise ClatterControlError("C0 report gate failed")
        (stage / "access.json").write_bytes(canonical_json({"counters": access_counters, "schema": ACCESS_SCHEMA}))
        (stage / "inventory.json").write_bytes(canonical_json(inventory))
        (stage / "prior.json").write_bytes(canonical_json(prior))
        (stage / "report.json").write_bytes(canonical_json(report))
        total_bytes = sum(path.stat().st_size for path in stage.rglob("*") if path.is_file())
        if total_bytes > profile["resources"]["max_output_bytes"]:
            raise ClatterControlError("external output exceeds the resource envelope")
        os.replace(stage, output)
    except BaseException:
        shutil.rmtree(stage, ignore_errors=True)
        raise


def main() -> None:
    arguments = parse_arguments()
    run(arguments.profile, arguments.source, arguments.output)


if __name__ == "__main__":
    main()
