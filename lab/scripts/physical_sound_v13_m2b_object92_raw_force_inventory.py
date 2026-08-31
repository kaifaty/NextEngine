#!/usr/bin/env python3
"""Inventory bounded object-92 raw microphone/force quartets without sample decode."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
import shutil
import struct
import tarfile
from pathlib import Path
from typing import Any

OBJECT_ID = "92"
EXPECTED_CONTACT_IDS = tuple(range(36))
MEMBER_KINDS = ("Force.wav", "metadata.yaml", "mic.wav", "striking_force.yaml")
EXPECTED_WAV_BYTES = 576_044
EXPECTED_WAV_HEADER = {
    "audio_format": 1,
    "channels": 1,
    "sample_rate_hz": 48_000,
    "sample_width_bytes": 2,
    "frame_count": 288_000,
}
GIB = 1_073_741_824
CHECKPOINTS_GIB = (4, 8, 12)
PARENT_MANIFEST_SHA256 = "b30f88978bd6dc749661c92a13093a97d6eda98771c75b058cc6ff149b5ea6b1"
PARENT_REPORT_SHA256 = "aafcbfdd551a0f18b0ec6fb390b093abb19825162455f49822fde718bacc0276"
PARENT_ROLE_ROOT_SHA256 = "a271bcd6d5cbdc2c0d61114b40f3080909ddd224d8930414524131597157b867"
ACQUISITION_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-object92-raw-acquisition.report.v0"
)
MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-object92-raw-force-inventory.v0"
)
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-object92-raw-force-inventory.report.v0"
)
EXPECTED_ROLES = {
    "formula_fit": (8, 32, 19, 26, 16, 21, 29, 1),
    "generator_development": (10, 23, 6, 3, 34, 15),
    "representation_holdout": (0, 4, 5, 7, 14, 17, 18, 20, 24, 25, 27, 30),
    "validator_calibration": (9, 11, 13),
    "validator_method_holdout": (2, 12, 31, 33),
    "admission_shadow": (22, 28, 35),
}


class InventoryError(RuntimeError):
    """The preregistered M2b raw inventory boundary was violated."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(8 * 1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def require_external_file(path: Path, context: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(repository_root()) or not resolved.is_file():
        raise InventoryError(f"{context} must be an external regular file")
    return resolved


def require_exact_file(path: Path, expected_sha256: str, context: str) -> Path:
    resolved = require_external_file(path, context)
    if sha256_file(resolved) != expected_sha256:
        raise InventoryError(f"{context} identity changed")
    return resolved


def load_json(path: Path, context: str) -> Any:
    try:
        return json.loads(path.read_bytes())
    except (UnicodeDecodeError, json.JSONDecodeError, OSError) as error:
        raise InventoryError(f"cannot decode {context} JSON") from error


def validate_parent(manifest: Any, report: Any) -> tuple[dict[int, str], dict[int, str]]:
    if not isinstance(manifest, dict) or not isinstance(report, dict):
        raise InventoryError("parent evidence must be JSON objects")
    if manifest.get("role_root_sha256") != PARENT_ROLE_ROOT_SHA256:
        raise InventoryError("parent role root changed")
    if report.get("manifest_sha256") != PARENT_MANIFEST_SHA256:
        raise InventoryError("parent report manifest identity changed")
    accounting = report.get("read_accounting", {})
    if accounting.get("pcm_sample_values_decoded") != 0:
        raise InventoryError("parent PCM sample counter is not zero")
    if accounting.get("force_sample_values_decoded") != 0:
        raise InventoryError("parent force sample counter is not zero")
    observed_roles = {
        role: tuple(contacts) for role, contacts in manifest.get("roles", {}).items()
    }
    if observed_roles != EXPECTED_ROLES:
        raise InventoryError("parent roles changed")
    compact_hashes: dict[int, str] = {}
    roles: dict[int, str] = {}
    for record in manifest.get("contacts", []):
        try:
            contact = int(record["contact_id"])
            compact_hashes[contact] = record["microphone"]["sha256"]
            roles[contact] = record["role"]
        except (KeyError, TypeError, ValueError) as error:
            raise InventoryError("parent contact record changed") from error
    if set(compact_hashes) != set(EXPECTED_CONTACT_IDS):
        raise InventoryError("parent compact contact set changed")
    return compact_hashes, roles


def validate_acquisition(report: Any, prefix: Path) -> tuple[int, str]:
    if not isinstance(report, dict) or report.get("schema") != ACQUISITION_REPORT_SCHEMA:
        raise InventoryError("acquisition report schema changed")
    if report.get("decision") != "READY_FOR_ZERO_SAMPLE_INVENTORY":
        raise InventoryError("acquisition report does not authorize inventory")
    checkpoint = report.get("checkpoint_gib")
    if checkpoint not in CHECKPOINTS_GIB:
        raise InventoryError("acquisition checkpoint changed")
    expected_bytes = checkpoint * GIB
    descriptor = report.get("prefix", {})
    if descriptor.get("bytes") != expected_bytes or prefix.stat().st_size != expected_bytes:
        raise InventoryError("acquired prefix byte count changed")
    prefix_sha256 = sha256_file(prefix)
    if descriptor.get("sha256") != prefix_sha256:
        raise InventoryError("acquired prefix hash changed")
    if report.get("microphone_sample_values_decoded") != 0:
        raise InventoryError("acquisition microphone counter is not zero")
    if report.get("force_sample_values_decoded") != 0:
        raise InventoryError("acquisition force counter is not zero")
    return checkpoint, prefix_sha256


def parse_pcm_header(payload: bytes) -> dict[str, int]:
    if len(payload) != EXPECTED_WAV_BYTES:
        raise InventoryError("selected WAV byte count changed")
    header = payload[:64]
    if len(header) < 44 or header[:4] != b"RIFF" or header[8:12] != b"WAVE":
        raise InventoryError("selected WAV has invalid RIFF/WAVE identity")
    if header[12:16] != b"fmt " or struct.unpack_from("<I", header, 16)[0] != 16:
        raise InventoryError("selected WAV fmt chunk changed")
    if header[36:40] != b"data":
        raise InventoryError("selected WAV data chunk changed")
    audio_format, channels, sample_rate = struct.unpack_from("<HHI", header, 20)
    block_align, bits_per_sample = struct.unpack_from("<HH", header, 32)
    data_bytes = struct.unpack_from("<I", header, 40)[0]
    if block_align == 0 or data_bytes % block_align:
        raise InventoryError("selected WAV block alignment changed")
    descriptor = {
        "audio_format": audio_format,
        "channels": channels,
        "sample_rate_hz": sample_rate,
        "sample_width_bytes": bits_per_sample // 8,
        "frame_count": data_bytes // block_align,
    }
    if descriptor != EXPECTED_WAV_HEADER or data_bytes + 44 != len(payload):
        raise InventoryError("selected WAV PCM descriptor changed")
    return descriptor


def selected_paths() -> set[str]:
    return {
        f"{OBJECT_ID}/audio/{contact}/{kind}"
        for contact in EXPECTED_CONTACT_IDS
        for kind in MEMBER_KINDS
    }


def member_record(archive: tarfile.TarFile, member: tarfile.TarInfo) -> dict[str, Any]:
    if not member.isfile():
        raise InventoryError(f"selected member is not a regular file: {member.name}")
    stream = archive.extractfile(member)
    if stream is None:
        raise InventoryError(f"cannot read selected member: {member.name}")
    payload = stream.read()
    if len(payload) != member.size:
        raise InventoryError(f"selected member was truncated: {member.name}")
    kind = Path(member.name).name
    result = {
        "path": member.name,
        "bytes": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "sample_values_decoded": 0,
    }
    if kind in ("mic.wav", "Force.wav"):
        result["header"] = parse_pcm_header(payload)
    else:
        result["numeric_values_decoded"] = 0
    return result


def scan_prefix(
    prefix: Path,
) -> tuple[dict[str, dict[str, Any]], int, bool, set[int], bool]:
    expected = selected_paths()
    found: dict[str, dict[str, Any]] = {}
    observed_contacts: set[int] = set()
    reached_next_object = False
    tail_truncated = False
    with prefix.open("rb") as raw:
        try:
            with tarfile.open(fileobj=raw, mode="r|gz") as archive:
                for member in archive:
                    parts = member.name.split("/")
                    if parts and parts[0] == "93":
                        reached_next_object = True
                        break
                    if len(parts) >= 3 and parts[0] == OBJECT_ID and parts[1] == "audio":
                        if parts[2].isdecimal():
                            observed_contacts.add(int(parts[2]))
                    if member.name not in expected:
                        continue
                    if member.name in found:
                        raise InventoryError(f"duplicate selected member: {member.name}")
                    found[member.name] = member_record(archive, member)
        except (tarfile.ReadError, EOFError, gzip.BadGzipFile, OSError):
            if raw.tell() < prefix.stat().st_size - 16 * 1024 * 1024:
                raise InventoryError("raw prefix failed before the expected truncated tail")
            tail_truncated = True
        compressed_bytes_consumed = raw.tell()
    return (
        found,
        compressed_bytes_consumed,
        tail_truncated,
        observed_contacts,
        reached_next_object,
    )


def prepare_output(output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(repository_root()):
        raise InventoryError("output must stay outside the repository")
    if resolved.exists():
        raise InventoryError("output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        raise InventoryError("staging output already exists")
    staging.mkdir()
    return resolved, staging


def inventory_decision(complete: bool, reached_next_object: bool, checkpoint_gib: int) -> str:
    if complete:
        return "READY_FOR_M2C_REALIMPACT_CONTROL_FREEZE"
    if reached_next_object:
        return "SOURCE_INCOMPLETE_OBJECT92"
    if checkpoint_gib < max(CHECKPOINTS_GIB):
        return "DATA_INSUFFICIENT_CHECKPOINT_EXTEND_WITHIN_PROTOCOL"
    return "DATA_INSUFFICIENT_ACQUISITION"


def run(
    parent_manifest_argument: Path,
    parent_report_argument: Path,
    acquisition_report_argument: Path,
    prefix_argument: Path,
    output_argument: Path,
) -> Path:
    parent_manifest_path = require_exact_file(
        parent_manifest_argument, PARENT_MANIFEST_SHA256, "parent manifest"
    )
    parent_report_path = require_exact_file(
        parent_report_argument, PARENT_REPORT_SHA256, "parent report"
    )
    acquisition_report_path = require_external_file(
        acquisition_report_argument, "acquisition report"
    )
    prefix_path = require_external_file(prefix_argument, "raw prefix")
    parent_manifest = load_json(parent_manifest_path, "parent manifest")
    parent_report = load_json(parent_report_path, "parent report")
    compact_hashes, roles = validate_parent(parent_manifest, parent_report)
    acquisition_report = load_json(acquisition_report_path, "acquisition report")
    checkpoint_gib, prefix_sha256 = validate_acquisition(acquisition_report, prefix_path)
    (
        found,
        compressed_consumed,
        tail_truncated,
        observed_contacts,
        reached_next_object,
    ) = scan_prefix(prefix_path)

    expected = selected_paths()
    complete = set(found) == expected
    if complete and observed_contacts != set(EXPECTED_CONTACT_IDS):
        raise InventoryError("raw object-92 contact set changed")
    if complete:
        for contact in EXPECTED_CONTACT_IDS:
            raw_microphone = found[f"{OBJECT_ID}/audio/{contact}/mic.wav"]
            if raw_microphone["sha256"] != compact_hashes[contact]:
                raise InventoryError(f"raw/compact microphone mismatch for contact {contact}")

    selected_bytes = sum(record["bytes"] for record in found.values())
    records = []
    for contact in EXPECTED_CONTACT_IDS:
        members = {
            kind: found.get(f"{OBJECT_ID}/audio/{contact}/{kind}")
            for kind in MEMBER_KINDS
        }
        records.append(
            {
                "object_id": OBJECT_ID,
                "contact_id": contact,
                "role": roles[contact],
                "complete": all(value is not None for value in members.values()),
                "members": members,
                "raw_compact_microphone_identity": (
                    members["mic.wav"] is not None
                    and members["mic.wav"]["sha256"] == compact_hashes[contact]
                ),
            }
        )

    decision = inventory_decision(complete, reached_next_object, checkpoint_gib)
    read_accounting = {
        "network_requests": 0,
        "prefix_bytes_hash_verified": prefix_path.stat().st_size,
        "compressed_prefix_bytes_consumed": compressed_consumed,
        "selected_member_bytes_hashed": selected_bytes,
        "selected_members_hash_committed": len(found),
        "wav_header_bytes_parsed": sum(
            64 for path in found if path.endswith(("mic.wav", "Force.wav"))
        ),
        "raw_compact_microphone_identity_comparisons": 36 if complete else 0,
        "microphone_sample_values_decoded": 0,
        "force_sample_values_decoded": 0,
        "striking_force_numeric_values_decoded": 0,
        "protected_role_signal_values_decoded": 0,
        "member_payloads_emitted": 0,
    }
    manifest = {
        "schema": MANIFEST_SCHEMA,
        "status": (
            "CompleteZeroSampleInventory"
            if complete
            else "RejectedIncompleteSource"
            if reached_next_object
            else "IncompleteCheckpoint"
        ),
        "decision": decision,
        "object_id": OBJECT_ID,
        "claim_ceiling": "canonical_impact_field",
        "parent_manifest_sha256": PARENT_MANIFEST_SHA256,
        "parent_report_sha256": PARENT_REPORT_SHA256,
        "parent_role_root_sha256": PARENT_ROLE_ROOT_SHA256,
        "acquisition_report_sha256": sha256_file(acquisition_report_path),
        "prefix": {
            "checkpoint_gib": checkpoint_gib,
            "bytes": prefix_path.stat().st_size,
            "sha256": prefix_sha256,
            "tail_truncated": tail_truncated,
            "reached_next_object_header": reached_next_object,
        },
        "roles": EXPECTED_ROLES,
        "contacts": records,
        "read_accounting": read_accounting,
        "raw_force_use": "future_onset_and_scalar_canonical_energy_only",
        "measured_transfer_field_authorized": False,
        "all_real_roles_sample_decode_closed": True,
        "runtime_consumer_allowed": False,
        "public_contract": False,
        "authored_clip_fallback_required": True,
        "implementation_sha256": sha256_file(Path(__file__).resolve()),
    }
    manifest_bytes = canonical_json(manifest)
    missing = sorted(expected - set(found))
    report = {
        "schema": REPORT_SCHEMA,
        "status": (
            "Validated"
            if complete
            else "Rejected"
            if reached_next_object
            else "DataInsufficient"
        ),
        "decision": decision,
        "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
        "checkpoint_gib": checkpoint_gib,
        "complete_contact_parents": sum(record["complete"] for record in records),
        "selected_member_count": len(found),
        "missing_member_count": len(missing),
        "missing_member_paths": missing,
        "raw_compact_microphone_identity_passed": complete,
        "read_accounting": read_accounting,
        "formula_or_model_fit_authorized": False,
        "real_quality_credit": False,
        "runtime_or_public_contract_changed": False,
        "authored_clip_fallback_required": True,
    }
    output, staging = prepare_output(output_argument)
    try:
        (staging / "manifest.json").write_bytes(manifest_bytes)
        (staging / "report.json").write_bytes(canonical_json(report))
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--parent-manifest", required=True, type=Path)
    parser.add_argument("--parent-report", required=True, type=Path)
    parser.add_argument("--acquisition-report", required=True, type=Path)
    parser.add_argument("--prefix", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    run(
        arguments.parent_manifest,
        arguments.parent_report,
        arguments.acquisition_report,
        arguments.prefix,
        arguments.output,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
