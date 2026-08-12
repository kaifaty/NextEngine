#!/usr/bin/env python3
"""Read-only identity and evidence audit for the NextEngine Isaac mirror."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
import sys
from pathlib import Path
from typing import Any


MAX_JSON_BYTES = 64 * 1024 * 1024
GENERATION_SCHEMA = "nextengine.humanoid-training-generation.v1"
REFERENCE_PROFILE_PREFIX = "nextengine.motor.env.humanoid-reference-tracker"
EXPECTED_ISAAC_PROFILE = {
    "profile_id": "nextengine.isaac-lab-physx-stage0.v1",
    "status": "Proposed",
    "isaac_lab_version": "2.3.2",
    "isaac_lab_distribution_version": "0.54.2",
    "isaac_sim_version": "5.1.0",
    "isaac_sim_distribution_version": "5.1.0.0",
    "engine_physx_version": "5.9.0",
    "python_version": "3.11",
    "backend": "physx",
}
P1_THRESHOLDS = {
    "joint_position_rmse_rad": ("maximum", 0.02),
    "root_position_rmse_m": ("maximum", 0.03),
    "root_velocity_rmse_mps": ("maximum", 0.05),
    "contact_occupancy_agreement": ("minimum", 0.98),
    "done_tick_agreement": ("minimum", 0.95),
    "reward_total_mae": ("maximum", 0.05),
}
P1_EXACT_KEYS = {
    "command_raw",
    "profile_id",
    "manifest_hash",
    "observation_layout_hash",
    "action_layout_hash",
    "command_schedule_profile_hash",
    "reward_profile_hash",
    "termination_profile_hash",
    "rng_derivation_profile_hash",
    "correspondence_profile_hash",
    "reward_component_ids",
}


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def canonical_embedded_hash(document: dict[str, Any], field: str) -> str:
    payload = {key: value for key, value in document.items() if key != field}
    encoded = json.dumps(
        payload,
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def canonical_corpus_hash(document: dict[str, Any]) -> str:
    payload = {key: value for key, value in document.items() if key != "manifest_sha256"}
    encoded = (
        json.dumps(
            payload,
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        )
        + "\n"
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def is_below(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return path != parent
    except ValueError:
        return False


def is_reference_profile_id(value: Any) -> bool:
    return isinstance(value, str) and value.startswith(REFERENCE_PROFILE_PREFIX)


class Audit:
    def __init__(self) -> None:
        self.issues: list[dict[str, Any]] = []
        self.identities: dict[str, Any] = {}

    def add(
        self,
        severity: str,
        code: str,
        message: str,
        *,
        expected: Any | None = None,
        actual: Any | None = None,
    ) -> None:
        item: dict[str, Any] = {"severity": severity, "code": code, "message": message}
        if expected is not None:
            item["expected"] = expected
        if actual is not None:
            item["actual"] = actual
        self.issues.append(item)

    def require(
        self,
        condition: bool,
        code: str,
        message: str,
        *,
        expected: Any | None = None,
        actual: Any | None = None,
    ) -> bool:
        if not condition:
            self.add("error", code, message, expected=expected, actual=actual)
        return condition

    @property
    def valid(self) -> bool:
        return not any(item["severity"] == "error" for item in self.issues)


def load_json(path: Path, label: str, audit: Audit) -> dict[str, Any] | None:
    if not path.is_file():
        audit.add("error", f"{label}.missing", f"{label} is not a file", actual=str(path))
        return None
    if path.stat().st_size > MAX_JSON_BYTES:
        audit.add("error", f"{label}.too_large", f"{label} exceeds the bounded JSON size", actual=path.stat().st_size)
        return None
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        audit.add("error", f"{label}.invalid_json", f"cannot read {label}: {exc}")
        return None
    if not isinstance(value, dict):
        audit.add("error", f"{label}.not_object", f"{label} must contain a JSON object")
        return None
    return value


def checked_hash(path: Path, label: str, audit: Audit) -> str | None:
    if not path.is_file():
        audit.add("error", f"{label}.missing", f"{label} is not a file", actual=str(path))
        return None
    try:
        value = file_sha256(path)
    except OSError as exc:
        audit.add("error", f"{label}.unreadable", f"cannot hash {label}: {exc}")
        return None
    audit.identities[f"{label}_file_sha256"] = value
    return value


def require_stable_hash(path: Path, initial: str | None, audit: Audit, label: str) -> None:
    if initial is None or not path.is_file():
        return
    try:
        final = file_sha256(path)
    except OSError as exc:
        audit.add("error", f"{label}.stability", f"cannot re-hash {label}: {exc}")
        return
    audit.require(
        final == initial,
        f"{label}.changed_during_audit",
        f"{label} changed while it was being audited",
        expected=initial,
        actual=final,
    )


def require_equal(
    audit: Audit,
    actual: Any,
    expected: Any,
    code: str,
    message: str,
) -> None:
    audit.require(actual == expected, code, message, expected=expected, actual=actual)


def admitted(generation: dict[str, Any], kind: str, digest: str | None) -> bool:
    return any(
        isinstance(item, dict)
        and item.get("kind") == kind
        and item.get("sha256") == digest
        for item in generation.get("admitted_inputs", [])
    )


def admitted_with_any_kind(generation: dict[str, Any], digest: str | None) -> bool:
    return any(
        isinstance(item, dict) and item.get("sha256") == digest
        for item in generation.get("admitted_inputs", [])
    )


def audit_repository(repository: Path, audit: Audit) -> None:
    audit.require(
        (repository / "AGENTS.md").is_file() and (repository / "lab/profiles").is_dir(),
        "repository.layout",
        "repository does not have the expected NextEngine layout",
        actual=str(repository),
    )
    try:
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=repository,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        dirty = subprocess.run(
            ["git", "status", "--porcelain=v1", "--untracked-files=all"],
            cwd=repository,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
    except (OSError, subprocess.CalledProcessError) as exc:
        audit.add("error", "repository.git", f"cannot inspect Git identity: {exc}")
        return
    audit.identities["repository_commit"] = commit
    audit.identities["repository_dirty_entry_count"] = len(dirty)
    if dirty:
        audit.add(
            "warning",
            "repository.dirty",
            "identity inspection is valid, but new correspondence evidence must use a clean recorded commit",
            actual=len(dirty),
        )


def audit_p1_report(
    report: dict[str, Any],
    report_path: Path,
    cpu_path: Path | None,
    gpu_path: Path | None,
    require_bound_inputs: bool,
    audit: Audit,
) -> tuple[bool, bool]:
    start_error_count = sum(item["severity"] == "error" for item in audit.issues)
    require_equal(audit, report.get("schema_version"), 2, "report.schema_version", "unsupported MODEL-MIRROR-P1 report schema")
    require_equal(audit, report.get("check"), "MODEL-MIRROR-P1", "report.check", "report is not MODEL-MIRROR-P1")
    require_equal(audit, report.get("status"), "passed", "report.status", "correspondence report did not pass")

    episodes = report.get("episodes")
    motor_steps = report.get("motor_steps_per_episode")
    audit.require(
        isinstance(episodes, int) and not isinstance(episodes, bool) and episodes >= 256,
        "report.episode_floor",
        "correspondence report does not meet the episode floor",
        expected=">= 256",
        actual=episodes,
    )
    audit.require(
        isinstance(motor_steps, int) and not isinstance(motor_steps, bool) and motor_steps >= 600,
        "report.motor_step_floor",
        "correspondence report does not meet the motor-step floor",
        expected=">= 600",
        actual=motor_steps,
    )

    metrics = report.get("metrics") if isinstance(report.get("metrics"), dict) else {}
    thresholds = report.get("thresholds") if isinstance(report.get("thresholds"), dict) else {}
    for name, (direction, limit) in P1_THRESHOLDS.items():
        actual = metrics.get(name)
        declared = thresholds.get(name)
        require_equal(
            audit,
            declared,
            limit,
            f"report.threshold.{name}",
            f"report threshold for {name} differs from the repository contract",
        )
        numeric = isinstance(actual, (int, float)) and not isinstance(actual, bool) and math.isfinite(float(actual))
        passed = numeric and (float(actual) <= limit if direction == "maximum" else float(actual) >= limit)
        audit.require(
            passed,
            f"report.metric.{name}",
            f"correspondence metric {name} violates its threshold",
            expected=f"{direction} {limit}",
            actual=actual,
        )

    exact = report.get("exact_matches") if isinstance(report.get("exact_matches"), dict) else {}
    missing_exact = sorted(P1_EXACT_KEYS - set(exact))
    audit.require(
        not missing_exact,
        "report.exact_matches.incomplete",
        "report omits required exact-match fields",
        actual=missing_exact,
    )
    for name, value in exact.items():
        audit.require(
            value is True,
            f"report.exact_matches.{name}",
            f"exact correspondence field {name} did not match",
            expected=True,
            actual=value,
        )

    gates = report.get("gates") if isinstance(report.get("gates"), dict) else {}
    audit.require(bool(gates), "report.gates.missing", "report has no correspondence gates")
    for name, value in gates.items():
        audit.require(
            value is True,
            f"report.gate.{name}",
            f"correspondence gate {name} did not pass",
            expected=True,
            actual=value,
        )

    inputs = report.get("inputs") if isinstance(report.get("inputs"), dict) else {}
    both_paths = cpu_path is not None and gpu_path is not None
    neither_path = cpu_path is None and gpu_path is None
    audit.require(
        both_paths or neither_path,
        "report.inputs.pair",
        "CPU and GPU trajectories must be supplied together",
    )
    inputs_verified = False
    if both_paths:
        cpu_hash = checked_hash(cpu_path, "cpu_trajectory", audit)
        gpu_hash = checked_hash(gpu_path, "gpu_trajectory", audit)
        require_equal(audit, inputs.get("cpu_sha256"), cpu_hash, "report.inputs.cpu", "report CPU trajectory hash mismatch")
        require_equal(audit, inputs.get("gpu_sha256"), gpu_hash, "report.inputs.gpu", "report GPU trajectory hash mismatch")
        inputs_verified = cpu_hash is not None and gpu_hash is not None
    elif require_bound_inputs:
        audit.add(
            "error",
            "report.inputs.not_verified",
            "a required passing report must be checked against both trajectory files",
        )
    else:
        audit.add(
            "warning",
            "report.inputs.not_verified",
            "report input hashes were not checked against trajectory files",
        )

    audit.identities["correspondence_report_file_sha256"] = file_sha256(report_path)
    end_error_count = sum(item["severity"] == "error" for item in audit.issues)
    return end_error_count == start_error_count, inputs_verified


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=Path.cwd())
    parser.add_argument("--generation-manifest", type=Path, required=True)
    parser.add_argument("--environment-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--isaac-profile", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path)
    parser.add_argument("--gate-report", type=Path)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--cpu", type=Path)
    parser.add_argument("--gpu", type=Path)
    parser.add_argument("--require-passed-report", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    audit = Audit()
    repository = args.repository.resolve()
    generation_path = args.generation_manifest.resolve()
    environment_path = args.environment_profile.resolve()
    descriptor_path = args.descriptor.resolve()
    usd_path = args.usd.resolve()
    isaac_path = args.isaac_profile.resolve()
    corpus_root = args.corpus_root.resolve() if args.corpus_root else None
    gate_path = args.gate_report.resolve() if args.gate_report else None
    report_path = args.report.resolve() if args.report else None
    cpu_path = args.cpu.resolve() if args.cpu else None
    gpu_path = args.gpu.resolve() if args.gpu else None

    audit_repository(repository, audit)
    profiles_root = (repository / "lab/profiles").resolve()
    for label, path in (("environment_profile", environment_path), ("isaac_profile", isaac_path)):
        audit.require(
            is_below(path, profiles_root),
            f"{label}.location",
            f"{label} must be tracked below lab/profiles",
            actual=str(path),
        )

    generation_hash = checked_hash(generation_path, "generation_manifest", audit)
    environment_hash = checked_hash(environment_path, "environment_profile", audit)
    descriptor_hash = checked_hash(descriptor_path, "descriptor", audit)
    usd_hash = checked_hash(usd_path, "usd", audit)
    isaac_hash = checked_hash(isaac_path, "isaac_profile", audit)

    generation = load_json(generation_path, "generation_manifest", audit)
    environment = load_json(environment_path, "environment_profile", audit)
    descriptor = load_json(descriptor_path, "descriptor", audit)
    isaac_profile = load_json(isaac_path, "isaac_profile", audit)

    if generation:
        require_equal(audit, generation.get("schema"), GENERATION_SCHEMA, "generation.schema", "unexpected generation schema")
        require_equal(audit, generation.get("schema_version"), 1, "generation.schema_version", "unsupported generation schema version")
        audit.require(
            generation.get("status") in {"train-5-preacceptance", "accepted"},
            "generation.status",
            "generation status does not admit mirror evidence",
            actual=generation.get("status"),
        )
        embedded = canonical_embedded_hash(generation, "manifest_hash")
        require_equal(audit, generation.get("manifest_hash"), embedded, "generation.manifest_hash", "generation embedded hash is not canonical")
        audit.identities.update(
            {
                "generation_id": generation.get("training_generation_id"),
                "generation_manifest_hash": embedded,
                "generation_manifest_file_sha256": generation_hash,
            }
        )

    reference_profile = bool(environment and is_reference_profile_id(environment.get("profile_id")))
    required_check = "MODEL-MIRROR-P2" if reference_profile else "MODEL-MIRROR-P1"
    audit.identities["required_check"] = required_check

    if environment:
        require_equal(audit, environment.get("schema_version"), 1, "environment.schema_version", "unsupported environment profile")
        require_equal(audit, environment.get("status"), "Frozen", "environment.status", "environment profile is not frozen")
        body = environment.get("body_schema") if isinstance(environment.get("body_schema"), dict) else {}
        require_equal(audit, body.get("descriptor_file_sha256"), descriptor_hash, "environment.descriptor", "environment descriptor hash mismatch")
        audit.identities.update(
            {
                "environment_profile_id": environment.get("profile_id"),
                "body_schema_id": body.get("id"),
                "body_schema_revision": body.get("revision"),
                "body_schema_hash": body.get("hash"),
                "compiled_descriptor_hash": body.get("compiled_descriptor_hash"),
            }
        )

    if descriptor and environment:
        body = environment.get("body_schema") if isinstance(environment.get("body_schema"), dict) else {}
        for descriptor_field, environment_field in (
            ("body_schema_id", "id"),
            ("body_schema_revision", "revision"),
            ("body_schema_hash", "hash"),
            ("compiled_descriptor_hash", "compiled_descriptor_hash"),
        ):
            require_equal(
                audit,
                descriptor.get(descriptor_field),
                body.get(environment_field),
                f"descriptor.{descriptor_field}",
                f"descriptor {descriptor_field} differs from the frozen environment",
            )

    if isaac_profile:
        for field, expected in EXPECTED_ISAAC_PROFILE.items():
            require_equal(
                audit,
                isaac_profile.get(field),
                expected,
                f"isaac_profile.{field}",
                f"tracked Isaac profile {field} differs from the audited skill contract; update the skill with the profile",
            )
        require_equal(audit, isaac_profile.get("schema_version"), 1, "isaac_profile.schema_version", "unsupported Isaac profile")
        audit.identities["isaac_profile_sha256"] = isaac_hash

    if generation:
        environment_admitted = (
            admitted(generation, "reference_tracker_profile", environment_hash)
            if reference_profile
            else admitted_with_any_kind(generation, environment_hash)
        )
        audit.require(
            environment_admitted,
            "generation.environment_not_admitted",
            "environment profile is not admitted by the generation",
            actual=environment_hash,
        )
        for kind, digest, code in (
            ("compiled_descriptor_file", descriptor_hash, "generation.descriptor_not_admitted"),
            ("isaac_biomechanics_usd", usd_hash, "generation.usd_not_admitted"),
        ):
            audit.require(
                admitted(generation, kind, digest),
                code,
                f"{kind} is not admitted by the generation",
                actual=digest,
            )

    closure_complete = True
    corpus_manifest_hash: str | None = None
    corpus_file_hash: str | None = None
    if corpus_root:
        corpus_path = corpus_root / "corpus-manifest.json"
        corpus_file_hash = checked_hash(corpus_path, "corpus_manifest", audit)
        corpus = load_json(corpus_path, "corpus_manifest", audit)
        if corpus:
            corpus_manifest_hash = corpus.get("manifest_sha256")
            require_equal(
                audit,
                canonical_corpus_hash(corpus),
                corpus_manifest_hash,
                "corpus.manifest_sha256",
                "corpus embedded hash is not canonical",
            )
            if environment:
                binding = environment.get("corpus") if isinstance(environment.get("corpus"), dict) else {}
                require_equal(audit, binding.get("manifest_sha256"), corpus_manifest_hash, "environment.corpus", "environment corpus hash mismatch")
            if generation:
                audit.require(
                    admitted(generation, "motion_corpus_manifest", corpus_manifest_hash),
                    "generation.corpus_not_admitted",
                    "corpus manifest is not admitted by the generation",
                    actual=corpus_manifest_hash,
                )
    else:
        closure_complete = False
        audit.add("warning", "corpus.not_supplied", "corpus closure was not audited")

    gate_hash: str | None = None
    if gate_path:
        gate_hash = checked_hash(gate_path, "gate_report", audit)
        gate = load_json(gate_path, "gate_report", audit)
        if gate:
            require_equal(audit, gate.get("gate_id"), "TRAIN-4", "gate.id", "unexpected gate report")
            require_equal(audit, gate.get("decision"), "Advance", "gate.decision", "TRAIN-4 did not advance")
            identities = gate.get("identities") if isinstance(gate.get("identities"), dict) else {}
            require_equal(audit, identities.get("descriptor_file_sha256"), descriptor_hash, "gate.descriptor", "gate descriptor hash mismatch")
            if corpus_manifest_hash:
                require_equal(audit, identities.get("corpus_manifest_sha256"), corpus_manifest_hash, "gate.corpus", "gate corpus hash mismatch")
            artifacts = gate.get("artifacts") if isinstance(gate.get("artifacts"), dict) else {}
            corpus_artifact = (
                artifacts.get("motion_corpus_manifest")
                if isinstance(artifacts.get("motion_corpus_manifest"), dict)
                else {}
            )
            if corpus_file_hash:
                require_equal(
                    audit,
                    corpus_artifact.get("file_sha256"),
                    corpus_file_hash,
                    "gate.corpus_file",
                    "gate corpus manifest file hash mismatch",
                )
            if environment:
                authorization = environment.get("training_authorization") if isinstance(environment.get("training_authorization"), dict) else {}
                require_equal(audit, authorization.get("gate_report_sha256"), gate_hash, "environment.gate", "environment gate hash mismatch")
            if generation:
                audit.require(
                    admitted(generation, "train_4_gate_report", gate_hash),
                    "generation.gate_not_admitted",
                    "gate report is not admitted by the generation",
                    actual=gate_hash,
                )
    else:
        closure_complete = False
        audit.add("warning", "gate.not_supplied", "TRAIN-4 gate closure was not audited")

    for label, path, digest in (
        ("generation_manifest", generation_path, generation_hash),
        ("environment_profile", environment_path, environment_hash),
        ("descriptor", descriptor_path, descriptor_hash),
        ("usd", usd_path, usd_hash),
        ("isaac_profile", isaac_path, isaac_hash),
        ("corpus_manifest", corpus_root / "corpus-manifest.json" if corpus_root else None, corpus_file_hash),
        ("gate_report", gate_path, gate_hash),
    ):
        if path is not None:
            require_stable_hash(path, digest, audit, label)

    errors_before_report = sum(item["severity"] == "error" for item in audit.issues)
    report_valid = False
    report_inputs_verified = False
    report_check: str | None = None
    evidence_status: str
    if report_path is None:
        evidence_status = "NotRun(no correspondence report supplied)"
        if args.require_passed_report:
            audit.add("error", "report.required", "a passing correspondence report is required")
    else:
        report = load_json(report_path, "correspondence_report", audit)
        report_check = report.get("check") if report else None
        if report and report_check == "MODEL-MIRROR-P1":
            report_valid, report_inputs_verified = audit_p1_report(
                report,
                report_path,
                cpu_path,
                gpu_path,
                args.require_passed_report,
                audit,
            )
            if required_check == "MODEL-MIRROR-P2":
                evidence_status = "Insufficient(MODEL-MIRROR-P1 does not satisfy MODEL-MIRROR-P2)"
                if args.require_passed_report:
                    audit.add(
                        "error",
                        "report.p2_required",
                        "the reference tracker requires repository-owned MODEL-MIRROR-P2 evidence",
                    )
            else:
                evidence_status = "Pass(MODEL-MIRROR-P1)" if report_valid else "Fail(MODEL-MIRROR-P1)"
        elif report and report_check == "MODEL-MIRROR-P2":
            evidence_status = "Unsupported(MODEL-MIRROR-P2 producer/schema is not implemented in this repository)"
            audit.add(
                "error" if args.require_passed_report else "warning",
                "report.p2_schema_unavailable",
                "do not accept a P2 label until a repository-owned producer and schema close its required semantics",
            )
        else:
            evidence_status = f"Fail(unexpected report check: {report_check!r})"
            audit.add("error", "report.check", "unsupported correspondence report check", actual=report_check)

    for label, path, identity_key in (
        ("correspondence_report", report_path, "correspondence_report_file_sha256"),
        ("cpu_trajectory", cpu_path, "cpu_trajectory_file_sha256"),
        ("gpu_trajectory", gpu_path, "gpu_trajectory_file_sha256"),
    ):
        if path is not None:
            require_stable_hash(path, audit.identities.get(identity_key), audit, label)

    identity_ready = errors_before_report == 0
    promotion_ready = (
        audit.valid
        and identity_ready
        and closure_complete
        and report_valid
        and report_inputs_verified
        and report_check == required_check
    )
    result = {
        "schema": "nextengine.skill.isaac-correspondence-audit.v1",
        "identity_ready": identity_ready,
        "closure_complete": closure_complete,
        "required_check": required_check,
        "evidence_status": evidence_status,
        "report_check": report_check,
        "report_valid": report_valid,
        "report_inputs_verified": report_inputs_verified,
        "promotion_ready": promotion_ready,
        "identities": audit.identities,
        "issues": audit.issues,
        "claim_boundary": "CorrespondenceOnly; no policy quality, runtime parity, TRAIN advancement, or R5 closure",
    }
    json.dump(result, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    if not audit.valid or (args.require_passed_report and not promotion_ready):
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
