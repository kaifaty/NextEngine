#!/usr/bin/env python3
"""Prove every V34 terminal path on discarded full-shape payloads."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import resource
import shutil
import struct
import sys
import tempfile
import time
from pathlib import Path
from typing import Any

import physical_sound_v34_f0_target_safe_profile_freeze_v1 as f0
import physical_sound_v34_terminal_publication_v1 as terminal

C0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v34-c0-structural-witness-census-result-2026-09-02.md"
)
C0_RESULT_SHA256 = "dbfcf031c4ab59a0c7eb3ca6b1b2bb8be3fef4f479d7a7b36eb36b08a12b964e"
C0_OWNER_PATH = "lab/scripts/physical_sound_v34_c0_structural_witness_census_v1.py"
C0_OWNER_SHA256 = "5eedac04fecf2373189be1a19350ab0d0d6e3cec7184fb23bfd66dbfb652c238"
OWNER_PATH = "lab/scripts/physical_sound_v34_t0_terminal_path_proof_v1.py"
TERMINAL_OWNER_PATH = "lab/scripts/physical_sound_v34_terminal_publication_v1.py"
CLAIM = (
    "DISCARDED_FULL_SHAPE_TERMINAL_PUBLICATION_PROOF_ONLY / "
    "NO_OFFICIAL_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
FORBIDDEN_IMPORT_STEMS = {
    "physical_sound_v33_d0_development_tournament_v1",
    "physical_sound_v33_i0_mode_local_spectral_owner_v1",
    "torch",
}
OFFICIAL_ZERO_ACCESS = {
    "development_target_rows": 0,
    "method_holdout_target_rows": 0,
    "network_requests": 0,
    "official_feature_rows_materialized": 0,
    "official_model_parameters_initialized": 0,
    "oracle_values_evaluated": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
    "train_target_rows": 0,
}


class T0ProofError(RuntimeError):
    """The discarded whole-owner terminal-path proof failed."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise T0ProofError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def validate_bound_file(path_text: str, expected_sha256: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink():
        raise T0ProofError(f"bound file must not be a symlink: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise T0ProofError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def validate_import_boundary() -> dict[str, Any]:
    imports: dict[str, list[str]] = {}
    for path_text in (OWNER_PATH, TERMINAL_OWNER_PATH):
        data = (repository_root() / path_text).read_bytes()
        tree = ast.parse(data)
        names: set[str] = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names.update(alias.name.split(".")[0] for alias in node.names)
            elif isinstance(node, ast.ImportFrom) and node.module:
                names.add(node.module.split(".")[0])
        forbidden = names & FORBIDDEN_IMPORT_STEMS
        if forbidden:
            raise T0ProofError(f"forbidden terminal import: {sorted(forbidden)}")
        imports[path_text] = sorted(names)
    return {"forbidden_imports": [], "imports": imports}


def load_context(profile_path: Path) -> tuple[dict[str, Any], dict[str, Any], bytes]:
    overlay, profile_data = f0.load_profile(profile_path)
    f0.validate_dependencies(overlay)
    v33 = f0.load_declared_json(overlay, "v33_profile")
    effective = f0.build_effective_profile(overlay, v33)
    validate_bound_file(C0_OWNER_PATH, C0_OWNER_SHA256)
    validate_bound_file(C0_RESULT_PATH, C0_RESULT_SHA256)
    return overlay, effective, profile_data


def discarded_access(phase: str) -> dict[str, Any]:
    result = {
        **OFFICIAL_ZERO_ACCESS,
        "access_phase": phase,
        "discarded_fixture_target_rows": 0,
        "target_rows_accessed": 0,
    }
    if phase == "post-access":
        result["discarded_fixture_target_rows"] = 10_800
        result["target_rows_accessed"] = 10_800
    return result


def zero_f64_payload(magic: bytes, rows: int, columns: int) -> bytes:
    if len(magic) != 8 or rows <= 0 or columns <= 0:
        raise T0ProofError("discarded payload shape is invalid")
    return magic + struct.pack("<II", rows, columns) + bytes(rows * columns * 8)


def discarded_payloads(outcome: str) -> dict[str, bytes]:
    if outcome == "ResourceReject":
        return {
            "resource-reject-receipt.json": canonical_json(
                {
                    "discarded": True,
                    "reason": "injected-resource-reject",
                    "shape_checked_before_publication": True,
                }
            )
        }
    payloads = {
        "control-report.json": canonical_json(
            {"discarded": True, "outcome": outcome, "quality_authority": False}
        ),
        "discarded-predictions.bin": zero_f64_payload(b"NXV34PRD", 4320, 3),
        "discarded-raw-control-weights.bin": zero_f64_payload(b"NXV34RAW", 1491, 1),
        "discarded-targets.bin": zero_f64_payload(b"NXV34TGT", 10800, 3),
        "discarded-weights.bin": zero_f64_payload(b"NXV34WGT", 1811, 1),
    }
    if outcome == "Pass":
        payloads[terminal.FREEZE_NAME] = canonical_json(
            {
                "discarded": True,
                "official_candidate_frozen": False,
                "status": "DiscardedFixtureFreezeOnly",
            }
        )
    return payloads


def tree_manifest(root: Path) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for path in sorted(root.rglob("*")):
        if path.is_file():
            data = path.read_bytes()
            name = str(path.relative_to(root))
            result[name] = {"bytes": len(data), "sha256": sha256_bytes(data)}
    return result


def external_output(path: Path) -> Path:
    try:
        return f0.external_output(path)
    except f0.F0FreezeError as error:
        raise T0ProofError(str(error)) from error


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    _, effective, profile_data = load_context(profile_path)
    output = external_output(output_path)
    import_boundary = validate_import_boundary()
    resources = effective["resources"]
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v34-t0-", dir=output.parent))
    try:
        terminals: dict[str, Any] = {}
        for outcome in terminal.SCIENTIFIC_OUTCOMES:
            directory_name = outcome.replace("Reject", "-reject").lower()
            report = terminal.publish_scientific_terminal(
                staging / directory_name,
                repository_root(),
                outcome,
                discarded_access("post-access"),
                discarded_payloads(outcome),
                CLAIM,
                resources["max_output_bytes"],
            )
            terminals[outcome] = {
                "candidate_frozen": report["candidate_frozen"],
                "decision": report["decision"],
                "directory": directory_name,
                "status": report["status"],
            }
        pre_access_output = staging / "pre-access-output-must-not-exist"
        pre_access = terminal.pre_access_contract_reject(
            pre_access_output,
            repository_root(),
            discarded_access("pre-access"),
            "injected-pre-access-contract-reject",
        )
        if pre_access_output.exists():
            raise T0ProofError("pre-access contract reject published output")
        (staging / "pre-access-contract-reject.json").write_bytes(
            canonical_json(pre_access)
        )
        payload_manifest = tree_manifest(staging)
        dependencies = {
            "c0_owner": validate_bound_file(C0_OWNER_PATH, C0_OWNER_SHA256),
            "c0_result": validate_bound_file(C0_RESULT_PATH, C0_RESULT_SHA256),
            "f0_profile": {
                "bytes": len(profile_data),
                "path": f0.PROFILE_PATH,
                "sha256": f0.PROFILE_SHA256,
            },
        }
        owner_data = (repository_root() / OWNER_PATH).read_bytes()
        terminal_data = (repository_root() / TERMINAL_OWNER_PATH).read_bytes()
        resource_gates = {
            "peak_rss_within_1_gib": peak_rss_bytes()
            <= resources["max_peak_rss_bytes"],
            "wall_within_300_seconds": time.monotonic() - started
            <= resources["max_wall_seconds"],
        }
        if not all(resource_gates.values()):
            raise T0ProofError("T0 resource envelope exceeded")
        evidence = {
            "claim": CLAIM,
            "dependencies": dependencies,
            "discarded_access": discarded_access("post-access"),
            "import_boundary": import_boundary,
            "official_access": OFFICIAL_ZERO_ACCESS,
            "owner_identity": {
                "bytes": len(owner_data),
                "path": OWNER_PATH,
                "sha256": sha256_bytes(owner_data),
            },
            "payload_manifest_before_evidence": payload_manifest,
            "resource_gates": resource_gates,
            "schema": "nextengine.experimental-physical-sound-v34-t0-evidence.v1",
            "terminal_owner_identity": {
                "bytes": len(terminal_data),
                "path": TERMINAL_OWNER_PATH,
                "sha256": sha256_bytes(terminal_data),
            },
        }
        evidence_data = canonical_json(evidence)
        (staging / "evidence.json").write_bytes(evidence_data)
        report = {
            "claim": CLAIM,
            "decision": "T0_TERMINAL_PATH_PROOF_PASS",
            "evidence_sha256": sha256_bytes(evidence_data),
            "next_authorized_stage": "V34-D0-owner-freeze-and-development",
            "official_target_or_model_values_opened": False,
            "pre_access_contract_reject": pre_access,
            "schema": "nextengine.experimental-physical-sound-v34-t0-report.v1",
            "status": "Pass",
            "terminals": terminals,
        }
        (staging / "report.json").write_bytes(canonical_json(report))
        total_bytes = sum(
            path.stat().st_size for path in staging.rglob("*") if path.is_file()
        )
        if total_bytes > resources["max_output_bytes"]:
            raise T0ProofError("T0 output resource envelope exceeded")
        staging.replace(output)
        print(
            f"t0-resource wall_seconds={time.monotonic() - started:.6f} "
            f"peak_rss_bytes={peak_rss_bytes()} output_bytes={total_bytes}",
            file=sys.stderr,
        )
        return report
    except Exception:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    try:
        report = run(arguments.profile, arguments.output)
    except (
        T0ProofError,
        terminal.TerminalPublicationError,
        f0.F0FreezeError,
        OSError,
        KeyError,
        TypeError,
        ValueError,
    ) as error:
        print(
            canonical_json(
                {"decision": "CONTRACT_REJECT", "error": str(error)}
            ).decode(),
            end="",
        )
        return 2
    print(
        canonical_json(
            {
                "decision": report["decision"],
                "official_target_or_model_values_opened": report[
                    "official_target_or_model_values_opened"
                ],
                "profile_sha256": f0.PROFILE_SHA256,
            }
        ).decode(),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
