#!/usr/bin/env python3
"""Prove the V35 complete-owner and terminal paths on discarded full-shape data."""

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

import physical_sound_v34_terminal_publication_v1 as terminal
import physical_sound_v35_b0_local_gate_conformance_v1 as b0

B0_OWNER_SHA256 = "e92924fa25269ca28a0afcb8cc581040152ab6795c5e98be010f36dcbcfd6c8d"
B0_RESULT_PATH = (
    "docs/development/physical-sound-v35-b0-local-gate-conformance-result-2026-09-03.md"
)
B0_RESULT_SHA256 = "494494d5ce4bb8317461672e7d57a81967a5ab368930766b72c5a329f52211fc"
TERMINAL_OWNER_PATH = "lab/scripts/physical_sound_v34_terminal_publication_v1.py"
TERMINAL_OWNER_SHA256 = (
    "60c54b392e20f1024a1cfbb3c2f69d3aed925e341495219787b6379fe1268f8d"
)
V34_T0_RESULT_PATH = (
    "docs/development/physical-sound-v34-t0-terminal-path-proof-result-2026-09-02.md"
)
V34_T0_RESULT_SHA256 = (
    "00d1ff517b1c08d59a0abbbf08bccb0fb59b17a54e308c0f16de5a54294d81c0"
)
OWNER_PATH = "lab/scripts/physical_sound_v35_i0_complete_owner_terminal_proof_v1.py"
CLAIM = (
    "DISCARDED_OFFICIAL_SHAPE_COMPLETE_OWNER_AND_TERMINAL_PATH_PROOF_ONLY / "
    "NO_OFFICIAL_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
TRAIN_MODAL_ROWS = 6_480
DEVELOPMENT_MODAL_ROWS = 4_320
OUTPUT_COLUMNS = 3
PREDICTION_VARIANTS = 13
CANDIDATE_PARAMETERS = 1_859
FORBIDDEN_IMPORT_STEMS = {
    "physical_sound_v31_p1_modal_owner_v1",
    "physical_sound_v33_d0_development_tournament_v1",
    "physical_sound_v33_i0_mode_local_spectral_owner_v1",
    "physical_sound_v34_d0_development_tournament_v1",
    "physical_sound_v34_h0_method_holdout_v1",
    "physical_sound_v35_c0_witness_and_coverage_census_v1",
    "torch",
}
OFFICIAL_ZERO_ACCESS = {
    **b0.ZERO_FORBIDDEN_ACCESS,
    "official_feature_rows_materialized": 0,
    "official_role_rows_materialized": 0,
}


class I0ProofError(RuntimeError):
    """The discarded V35 complete-owner terminal proof failed."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise I0ProofError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def validate_bound_file(path_text: str, expected_sha256: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink():
        raise I0ProofError(f"bound file must not be a symlink: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise I0ProofError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def imported_stems(path_text: str) -> list[str]:
    data = (repository_root() / path_text).read_bytes()
    tree = ast.parse(data)
    names: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            names.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            names.add(node.module.split(".")[0])
    return sorted(names)


def validate_import_boundary() -> dict[str, Any]:
    imports = {
        path: imported_stems(path)
        for path in (OWNER_PATH, TERMINAL_OWNER_PATH, b0.OWNER_PATH)
    }
    forbidden = sorted(
        {
            name
            for names in imports.values()
            for name in names
            if name in FORBIDDEN_IMPORT_STEMS
        }
    )
    if forbidden:
        raise I0ProofError(f"forbidden I0 import: {forbidden}")
    required = {
        "physical_sound_v34_terminal_publication_v1",
        "physical_sound_v35_b0_local_gate_conformance_v1",
    }
    if not required.issubset(set(imports[OWNER_PATH])):
        raise I0ProofError("I0 does not import both frozen reusable owners")
    return {"forbidden_imports": [], "imports": imports}


def validate_context(
    profile_path: Path,
) -> tuple[dict[str, Any], dict[str, Any], bytes, dict[str, Any]]:
    overlay, effective = b0.validate_context(profile_path)
    profile_data = profile_path.read_bytes()
    dependencies = {
        "b0_owner": validate_bound_file(b0.OWNER_PATH, B0_OWNER_SHA256),
        "b0_result": validate_bound_file(B0_RESULT_PATH, B0_RESULT_SHA256),
        "terminal_owner": validate_bound_file(
            TERMINAL_OWNER_PATH, TERMINAL_OWNER_SHA256
        ),
        "v34_t0_result": validate_bound_file(V34_T0_RESULT_PATH, V34_T0_RESULT_SHA256),
    }
    counts = effective["corpus"]["counts"]
    model = effective["method_overlay"]["candidate"]["model"]
    controls = effective["method_overlay"]["controls"]
    ablations = effective["method_overlay"]["ablations"]
    if (
        counts["train"]["modal_rows"] != TRAIN_MODAL_ROWS
        or counts["development"]["modal_rows"] != DEVELOPMENT_MODAL_ROWS
        or counts["method_holdout"]["modal_rows"] != DEVELOPMENT_MODAL_ROWS
        or model["parameter_count"] != CANDIDATE_PARAMETERS
        or len(controls) + len(ablations) + 1 != PREDICTION_VARIANTS
    ):
        raise I0ProofError("discarded complete-owner shape drift")
    return overlay, effective, profile_data, dependencies


def discarded_access(phase: str) -> dict[str, Any]:
    result = {
        **OFFICIAL_ZERO_ACCESS,
        "access_phase": phase,
        "discarded_candidate_parameters_initialized": 0,
        "discarded_development_target_rows": 0,
        "discarded_train_target_rows": 0,
        "target_rows_accessed": 0,
    }
    if phase == "post-access":
        result.update(
            {
                "discarded_candidate_parameters_initialized": CANDIDATE_PARAMETERS,
                "discarded_development_target_rows": DEVELOPMENT_MODAL_ROWS,
                "discarded_train_target_rows": TRAIN_MODAL_ROWS,
                "target_rows_accessed": TRAIN_MODAL_ROWS + DEVELOPMENT_MODAL_ROWS,
            }
        )
    elif phase != "pre-access":
        raise I0ProofError("unknown discarded access phase")
    return result


def zero_f64_payload(magic: bytes, rows: int, columns: int) -> bytes:
    if len(magic) != 8 or rows <= 0 or columns <= 0:
        raise I0ProofError("discarded payload shape is invalid")
    return magic + struct.pack("<II", rows, columns) + bytes(rows * columns * 8)


def discarded_control_manifest(effective: dict[str, Any]) -> bytes:
    method = effective["method_overlay"]
    return canonical_json(
        {
            "ablations": sorted(method["ablations"]),
            "candidate": method["candidate"]["candidate_id"],
            "discarded": True,
            "prediction_columns_per_variant": OUTPUT_COLUMNS,
            "prediction_variants": PREDICTION_VARIANTS,
            "quality_authority": False,
            "controls": sorted(method["controls"]),
        }
    )


def discarded_payloads(
    outcome: str,
    effective: dict[str, Any],
    b0_conformance_data: bytes,
) -> dict[str, bytes]:
    if outcome == "ResourceReject":
        return {
            "resource-reject-receipt.json": canonical_json(
                {
                    "complete_owner_shape_checked": True,
                    "discarded": True,
                    "local_gate_conformance_sha256": sha256_bytes(b0_conformance_data),
                    "reason": "injected-resource-reject",
                }
            )
        }
    payloads = {
        "discarded-candidate-weights.bin": zero_f64_payload(
            b"NXV35WGT", CANDIDATE_PARAMETERS, 1
        ),
        "discarded-control-manifest.json": discarded_control_manifest(effective),
        "discarded-development-predictions.bin": zero_f64_payload(
            b"NXV35PRD",
            DEVELOPMENT_MODAL_ROWS,
            PREDICTION_VARIANTS * OUTPUT_COLUMNS,
        ),
        "discarded-development-targets.bin": zero_f64_payload(
            b"NXV35DEV", DEVELOPMENT_MODAL_ROWS, OUTPUT_COLUMNS
        ),
        "discarded-local-gate-conformance.json": b0_conformance_data,
        "discarded-train-targets.bin": zero_f64_payload(
            b"NXV35TRN", TRAIN_MODAL_ROWS, OUTPUT_COLUMNS
        ),
        "owner-report.json": canonical_json(
            {
                "discarded": True,
                "official_candidate_frozen": False,
                "outcome": outcome,
                "quality_authority": False,
            }
        ),
    }
    if outcome == "Pass":
        payloads[terminal.FREEZE_NAME] = canonical_json(
            {
                "candidate_parameter_count": CANDIDATE_PARAMETERS,
                "discarded": True,
                "official_candidate_frozen": False,
                "profile_sha256": b0.f0.PROFILE_SHA256,
                "status": "DiscardedFixtureFreezeOnly",
            }
        )
    return payloads


def validate_discarded_payloads(payloads: dict[str, bytes], outcome: str) -> None:
    has_freeze = terminal.FREEZE_NAME in payloads
    if has_freeze != (outcome == "Pass"):
        raise I0ProofError("discarded candidate freeze/outcome mismatch")
    if outcome == "ResourceReject":
        if set(payloads) != {"resource-reject-receipt.json"}:
            raise I0ProofError("resource reject payload drift")
        return
    expected_sizes = {
        "discarded-candidate-weights.bin": 16 + CANDIDATE_PARAMETERS * 8,
        "discarded-development-predictions.bin": (
            16 + DEVELOPMENT_MODAL_ROWS * PREDICTION_VARIANTS * OUTPUT_COLUMNS * 8
        ),
        "discarded-development-targets.bin": (
            16 + DEVELOPMENT_MODAL_ROWS * OUTPUT_COLUMNS * 8
        ),
        "discarded-train-targets.bin": 16 + TRAIN_MODAL_ROWS * OUTPUT_COLUMNS * 8,
    }
    for name, expected_size in expected_sizes.items():
        if len(payloads[name]) != expected_size:
            raise I0ProofError(f"discarded payload size drift: {name}")


def tree_manifest(root: Path) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for path in sorted(root.rglob("*")):
        if path.is_file():
            data = path.read_bytes()
            name = str(path.relative_to(root))
            result[name] = {"bytes": len(data), "sha256": sha256_bytes(data)}
    return result


def failpoint_probes(
    root: Path, maximum_output_bytes: int
) -> dict[str, dict[str, Any]]:
    failpoint_root = root / "failpoint-probes"
    failpoint_root.mkdir()
    results: dict[str, dict[str, Any]] = {}
    for failpoint in ("after-first-payload", "after-evidence"):
        output = failpoint_root / failpoint
        try:
            terminal.publish_scientific_terminal(
                output,
                repository_root(),
                "MetricReject",
                discarded_access("post-access"),
                {"discarded-a.bin": b"a", "discarded-b.bin": b"b"},
                CLAIM,
                maximum_output_bytes,
                failpoint=failpoint,
            )
        except terminal.TerminalPublicationError as error:
            if "injected publication failure" not in str(error):
                raise
        else:
            raise I0ProofError(f"failpoint unexpectedly published: {failpoint}")
        if output.exists() or any(
            path.name.startswith(".nextengine-v34-terminal-")
            for path in failpoint_root.iterdir()
        ):
            raise I0ProofError(f"failpoint left partial output: {failpoint}")
        results[failpoint] = {
            "output_exists": False,
            "staging_exists": False,
            "status": "Pass",
        }
    return results


def external_output(path: Path) -> Path:
    try:
        return b0.external_output(path)
    except b0.B0ConformanceError as error:
        raise I0ProofError(str(error)) from error


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    output = external_output(output_path)
    _, effective, profile_data, dependencies = validate_context(profile_path)
    import_boundary = validate_import_boundary()
    b0_conformance = b0.build_conformance(profile_path)
    if b0_conformance["status"] != "Pass" or set(b0_conformance["gates"].values()) != {
        "Pass"
    }:
        raise I0ProofError("embedded B0 conformance did not pass")
    b0_conformance_data = canonical_json(b0_conformance)
    resources = effective["resources"]
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v35-i0-", dir=output.parent))
    try:
        terminals: dict[str, Any] = {}
        for outcome in terminal.SCIENTIFIC_OUTCOMES:
            payloads = discarded_payloads(outcome, effective, b0_conformance_data)
            validate_discarded_payloads(payloads, outcome)
            directory_name = outcome.replace("Reject", "-reject").lower()
            report = terminal.publish_scientific_terminal(
                staging / directory_name,
                repository_root(),
                outcome,
                discarded_access("post-access"),
                payloads,
                CLAIM,
                resources["max_output_bytes"],
            )
            terminals[outcome] = {
                "candidate_frozen": report["candidate_frozen"],
                "decision": report["decision"],
                "directory": directory_name,
                "payload_count": len(payloads),
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
            raise I0ProofError("pre-access contract reject published output")
        (staging / "pre-access-contract-reject.json").write_bytes(
            canonical_json(pre_access)
        )
        failpoints = failpoint_probes(staging, resources["max_output_bytes"])
        payload_manifest = tree_manifest(staging)
        owner_data = (repository_root() / OWNER_PATH).read_bytes()
        resource_gates = {
            "peak_rss_within_1_gib": peak_rss_bytes()
            <= resources["max_peak_rss_bytes"],
            "wall_within_300_seconds": time.monotonic() - started
            <= resources["max_wall_seconds"],
        }
        if not all(resource_gates.values()):
            raise I0ProofError("I0 resource envelope exceeded")
        evidence = {
            "b0_conformance_sha256": sha256_bytes(b0_conformance_data),
            "claim": CLAIM,
            "dependencies": dependencies,
            "discarded_access": discarded_access("post-access"),
            "failpoint_probes": failpoints,
            "import_boundary": import_boundary,
            "official_access": OFFICIAL_ZERO_ACCESS,
            "owner_identity": {
                "bytes": len(owner_data),
                "path": OWNER_PATH,
                "sha256": sha256_bytes(owner_data),
            },
            "payload_manifest_before_evidence": payload_manifest,
            "profile_identity": {
                "bytes": len(profile_data),
                "path": b0.f0.PROFILE_PATH,
                "sha256": b0.f0.PROFILE_SHA256,
            },
            "resource_gates": resource_gates,
            "schema": "nextengine.experimental-physical-sound-v35-i0-evidence.v1",
        }
        evidence_data = canonical_json(evidence)
        (staging / "evidence.json").write_bytes(evidence_data)
        report = {
            "claim": CLAIM,
            "decision": "I0_COMPLETE_OWNER_TERMINAL_PATH_PROOF_PASS",
            "evidence_sha256": sha256_bytes(evidence_data),
            "next_authorized_stage": "V35-D0-owner-freeze-and-development",
            "official_target_or_model_values_opened": False,
            "pre_access_contract_reject": pre_access,
            "schema": "nextengine.experimental-physical-sound-v35-i0-report.v1",
            "status": "Pass",
            "terminals": terminals,
        }
        (staging / "report.json").write_bytes(canonical_json(report))
        total_bytes = sum(
            path.stat().st_size for path in staging.rglob("*") if path.is_file()
        )
        if total_bytes > resources["max_output_bytes"]:
            raise I0ProofError("I0 output resource envelope exceeded")
        staging.replace(output)
        print(
            f"i0-resource wall_seconds={time.monotonic() - started:.6f} "
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
        I0ProofError,
        b0.B0ConformanceError,
        b0.f0.F0FreezeError,
        terminal.TerminalPublicationError,
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
                "profile_sha256": b0.f0.PROFILE_SHA256,
            }
        ).decode(),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
