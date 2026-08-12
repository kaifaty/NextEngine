#!/usr/bin/env python3
"""Read-only preflight for a hash-closed NextEngine reference PPO run."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


MAX_JSON_BYTES = 64 * 1024 * 1024
RUN_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,126}[A-Za-z0-9]$|^[A-Za-z0-9]$")
GENERATION_SCHEMA = "nextengine.humanoid-training-generation.v1"
INDEX_SCHEMA = "nextengine.active-training-generation.v1"
REFERENCE_PROFILE_PREFIX = "nextengine.motor.env.humanoid-reference-tracker"
TRAINING_PROFILE_KINDS = {
    "tiny_overfit_profile",
    "reference_curriculum_stage_profile",
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

    def issue(
        self,
        severity: str,
        code: str,
        message: str,
        *,
        expected: Any | None = None,
        actual: Any | None = None,
    ) -> None:
        item: dict[str, Any] = {
            "severity": severity,
            "code": code,
            "message": message,
        }
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
            self.issue(
                "error",
                code,
                message,
                expected=expected,
                actual=actual,
            )
        return condition

    @property
    def ready(self) -> bool:
        return not any(item["severity"] == "error" for item in self.issues)


def load_json_object(path: Path, audit: Audit, label: str) -> dict[str, Any] | None:
    if not path.is_file():
        audit.issue("error", f"{label}.missing", f"{label} is not a file", actual=str(path))
        return None
    size = path.stat().st_size
    if size > MAX_JSON_BYTES:
        audit.issue(
            "error",
            f"{label}.too_large",
            f"{label} exceeds the bounded JSON size",
            expected=f"<= {MAX_JSON_BYTES}",
            actual=size,
        )
        return None
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        audit.issue("error", f"{label}.invalid_json", f"cannot read {label}: {exc}")
        return None
    if not isinstance(value, dict):
        audit.issue("error", f"{label}.not_object", f"{label} must contain a JSON object")
        return None
    return value


def checked_file_hash(path: Path, audit: Audit, label: str) -> str | None:
    if not path.is_file():
        audit.issue("error", f"{label}.missing", f"{label} is not a file", actual=str(path))
        return None
    try:
        digest = file_sha256(path)
    except OSError as exc:
        audit.issue("error", f"{label}.unreadable", f"cannot hash {label}: {exc}")
        return None
    audit.identities[f"{label}_file_sha256"] = digest
    return digest


def require_stable_hash(path: Path, initial: str | None, audit: Audit, label: str) -> None:
    if initial is None or not path.is_file():
        return
    try:
        final = file_sha256(path)
    except OSError as exc:
        audit.issue("error", f"{label}.stability", f"cannot re-hash {label}: {exc}")
        return
    audit.require(
        final == initial,
        f"{label}.changed_during_preflight",
        f"{label} changed while it was being audited",
        expected=initial,
        actual=final,
    )


def git_identity(repository: Path, audit: Audit) -> None:
    required = (repository / "AGENTS.md", repository / "lab", repository / "docs/architecture")
    audit.require(
        all(path.exists() for path in required),
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
        status = subprocess.run(
            ["git", "status", "--porcelain=v1", "--untracked-files=all"],
            cwd=repository,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
    except (OSError, subprocess.CalledProcessError) as exc:
        audit.issue("error", "repository.git", f"cannot inspect repository Git identity: {exc}")
        return
    audit.identities["repository_commit"] = commit
    audit.identities["repository_dirty_entry_count"] = len(status)
    audit.require(
        not status,
        "repository.dirty",
        "training requires a clean recorded repository commit",
        expected=0,
        actual=len(status),
    )


def require_match(
    audit: Audit,
    actual: Any,
    expected: Any,
    code: str,
    message: str,
) -> None:
    audit.require(actual == expected, code, message, expected=expected, actual=actual)


def admitted(
    generation: dict[str, Any],
    *,
    kind: str | set[str],
    sha256: str | None,
) -> bool:
    kinds = {kind} if isinstance(kind, str) else kind
    return any(
        isinstance(item, dict)
        and item.get("kind") in kinds
        and item.get("sha256") == sha256
        for item in generation.get("admitted_inputs", [])
    )


def audit_generation(
    path: Path,
    document: dict[str, Any],
    file_hash: str | None,
    audit: Audit,
) -> None:
    require_match(
        audit,
        document.get("schema"),
        GENERATION_SCHEMA,
        "generation.schema",
        "unsupported generation manifest schema",
    )
    require_match(
        audit,
        document.get("schema_version"),
        1,
        "generation.schema_version",
        "unsupported generation manifest schema version",
    )
    require_match(
        audit,
        document.get("status"),
        "train-5-preacceptance",
        "generation.status",
        "reference PPO preflight requires the active TRAIN-5 generation status",
    )
    embedded = canonical_embedded_hash(document, "manifest_hash")
    require_match(
        audit,
        document.get("manifest_hash"),
        embedded,
        "generation.manifest_hash",
        "generation manifest embedded hash is not canonical",
    )
    audit.identities.update(
        {
            "generation_id": document.get("training_generation_id"),
            "generation_manifest_hash": embedded,
            "generation_manifest_file_sha256": file_hash,
            "generation_root": str(path.parent),
        }
    )
    audit.require(
        isinstance(document.get("admitted_inputs"), list),
        "generation.admitted_inputs",
        "generation admitted_inputs must be an array",
    )


def audit_index(
    path: Path,
    document: dict[str, Any],
    generation_path: Path,
    generation: dict[str, Any],
    generation_file_hash: str | None,
    audit: Audit,
) -> None:
    require_match(audit, document.get("schema"), INDEX_SCHEMA, "index.schema", "unsupported index schema")
    require_match(
        audit,
        document.get("schema_version"),
        1,
        "index.schema_version",
        "unsupported index schema version",
    )
    embedded = canonical_embedded_hash(document, "index_hash")
    require_match(audit, document.get("index_hash"), embedded, "index.index_hash", "index hash is not canonical")
    relative = document.get("generation_manifest")
    resolved: Path | None = None
    if isinstance(relative, str) and relative:
        resolved = (path.parent / relative).resolve()
        audit.require(
            is_below(resolved, path.parent.resolve()),
            "index.generation_manifest_escape",
            "generation manifest path escapes the training store",
            actual=relative,
        )
        require_match(
            audit,
            str(resolved),
            str(generation_path),
            "index.generation_manifest_path",
            "index does not select the supplied generation manifest",
        )
    else:
        audit.issue("error", "index.generation_manifest", "index generation_manifest must be a non-empty string")
    require_match(
        audit,
        document.get("generation_manifest_sha256"),
        generation_file_hash,
        "index.generation_manifest_sha256",
        "index generation manifest file hash mismatch",
    )
    require_match(
        audit,
        document.get("active_training_generation_id"),
        generation.get("training_generation_id"),
        "index.generation_id",
        "index and generation IDs differ",
    )
    audit.identities["generation_index_hash"] = embedded


def audit_checkpoint(
    checkpoint_path: Path | None,
    initialization: Any,
    generation: dict[str, Any],
    audit: Audit,
) -> None:
    if initialization is None:
        audit.require(
            checkpoint_path is None,
            "checkpoint.unbound",
            "a checkpoint was supplied but the training profile has no initialization closure",
        )
        return
    if not isinstance(initialization, dict):
        audit.issue("error", "checkpoint.initialization", "training profile initialization must be an object")
        return
    require_match(
        audit,
        initialization.get("mode"),
        "model-weights-only",
        "checkpoint.mode",
        "the current reference trainer only admits model-weights-only initialization",
    )
    if checkpoint_path is None:
        audit.issue("error", "checkpoint.required", "the training profile binds an initial checkpoint")
        return
    checkpoint_hash = checked_file_hash(checkpoint_path, audit, "initial_checkpoint")
    require_match(
        audit,
        checkpoint_hash,
        initialization.get("checkpoint_sha256"),
        "checkpoint.sha256",
        "initial checkpoint hash differs from the training profile",
    )
    audit.require(
        admitted(
            generation,
            kind="reference_curriculum_initial_checkpoint",
            sha256=checkpoint_hash,
        ),
        "checkpoint.not_admitted",
        "initial checkpoint is not admitted by the generation",
        actual=checkpoint_hash,
    )

    source_manifest = checkpoint_path.parent / "run-manifest.json"
    if not source_manifest.is_file():
        audit.issue(
            "error",
            "checkpoint.source_manifest_missing",
            "initial checkpoint must have a sibling closed run-manifest.json",
            actual=str(source_manifest),
        )
        return
    source = load_json_object(source_manifest, audit, "checkpoint_source_manifest")
    if source is None:
        return
    source_checkpoint = source.get("checkpoint")
    source_hash = source_checkpoint.get("sha256") if isinstance(source_checkpoint, dict) else None
    require_match(
        audit,
        source_hash,
        checkpoint_hash,
        "checkpoint.source_manifest_hash",
        "source run manifest does not close the selected checkpoint",
    )
    require_match(
        audit,
        source.get("training_profile_sha256"),
        initialization.get("training_profile_sha256"),
        "checkpoint.source_profile",
        "source run training profile differs from initialization lineage",
    )
    require_match(
        audit,
        source.get("status"),
        "completed",
        "checkpoint.source_status",
        "initial checkpoint source run is not completed",
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=Path.cwd())
    parser.add_argument("--generation-manifest", type=Path, required=True)
    parser.add_argument("--generation-index", type=Path)
    parser.add_argument("--training-profile", type=Path, required=True)
    parser.add_argument("--environment-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--initial-checkpoint", type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    audit = Audit()

    repository = args.repository.resolve()
    generation_path = args.generation_manifest.resolve()
    training_path = args.training_profile.resolve()
    environment_path = args.environment_profile.resolve()
    descriptor_path = args.descriptor.resolve()
    corpus_root = args.corpus_root.resolve()
    gate_path = args.gate_report.resolve()
    usd_path = args.usd.resolve()
    output_root = args.output_root.resolve()
    checkpoint_path = args.initial_checkpoint.resolve() if args.initial_checkpoint else None

    git_identity(repository, audit)
    profiles_root = (repository / "lab/profiles").resolve()
    audit.require(
        is_below(training_path, profiles_root),
        "training_profile.location",
        "training profile must be tracked below lab/profiles",
        actual=str(training_path),
    )
    audit.require(
        is_below(environment_path, profiles_root),
        "environment_profile.location",
        "environment profile must be tracked below lab/profiles",
        actual=str(environment_path),
    )

    generation_hash = checked_file_hash(generation_path, audit, "generation_manifest")
    training_hash = checked_file_hash(training_path, audit, "training_profile")
    environment_hash = checked_file_hash(environment_path, audit, "environment_profile")
    descriptor_hash = checked_file_hash(descriptor_path, audit, "descriptor")
    gate_hash = checked_file_hash(gate_path, audit, "gate_report")
    usd_hash = checked_file_hash(usd_path, audit, "usd")

    generation = load_json_object(generation_path, audit, "generation_manifest")
    training = load_json_object(training_path, audit, "training_profile")
    environment = load_json_object(environment_path, audit, "environment_profile")
    descriptor = load_json_object(descriptor_path, audit, "descriptor")
    gate = load_json_object(gate_path, audit, "gate_report")

    if generation is not None:
        audit_generation(generation_path, generation, generation_hash, audit)
        if args.generation_index:
            index_path = args.generation_index.resolve()
            index = load_json_object(index_path, audit, "generation_index")
            if index is not None:
                audit_index(index_path, index, generation_path, generation, generation_hash, audit)

    corpus_manifest_path = corpus_root / "corpus-manifest.json"
    corpus_file_hash = checked_file_hash(corpus_manifest_path, audit, "corpus_manifest")
    corpus = load_json_object(corpus_manifest_path, audit, "corpus_manifest")
    corpus_manifest_hash = corpus.get("manifest_sha256") if corpus else None
    if corpus:
        require_match(
            audit,
            corpus.get("status"),
            "VALIDATED",
            "corpus.status",
            "corpus is not validated",
        )
        require_match(
            audit,
            canonical_corpus_hash(corpus),
            corpus_manifest_hash,
            "corpus.manifest_sha256",
            "corpus manifest embedded hash is not canonical",
        )
        audit.identities["corpus_manifest_hash"] = corpus_manifest_hash

    if training:
        require_match(audit, training.get("schema_version"), 1, "training.schema_version", "unsupported training profile")
        require_match(audit, training.get("status"), "Frozen", "training.status", "training profile is not frozen")
        environment_profile_id = environment.get("profile_id") if environment else None
        require_match(
            audit,
            training.get("environment_profile_id"),
            environment_profile_id,
            "training.environment_profile_id",
            "training profile targets a different environment",
        )
        audit.require(
            is_reference_profile_id(environment_profile_id),
            "training.environment_profile_family",
            "this preflight only supports the reference-tracker environment family",
            actual=environment_profile_id,
        )
        require_match(
            audit,
            training.get("environment_profile_sha256"),
            environment_hash,
            "training.environment_profile_sha256",
            "training and environment profile hashes differ",
        )
        require_match(audit, training.get("descriptor_sha256"), descriptor_hash, "training.descriptor", "descriptor hash mismatch")
        require_match(audit, training.get("usd_sha256"), usd_hash, "training.usd", "USD hash mismatch")
        require_match(
            audit,
            training.get("corpus_manifest_sha256"),
            corpus_manifest_hash,
            "training.corpus",
            "corpus manifest hash mismatch",
        )

    if environment:
        require_match(audit, environment.get("schema_version"), 1, "environment.schema_version", "unsupported environment profile")
        require_match(audit, environment.get("status"), "Frozen", "environment.status", "environment profile is not frozen")
        audit.require(
            is_reference_profile_id(environment.get("profile_id")),
            "environment.profile_id",
            "unexpected reference environment profile family",
            actual=environment.get("profile_id"),
        )
        body = environment.get("body_schema") if isinstance(environment.get("body_schema"), dict) else {}
        require_match(
            audit,
            body.get("descriptor_file_sha256"),
            descriptor_hash,
            "environment.descriptor",
            "environment body descriptor file hash mismatch",
        )
        corpus_binding = environment.get("corpus") if isinstance(environment.get("corpus"), dict) else {}
        require_match(
            audit,
            corpus_binding.get("manifest_sha256"),
            corpus_manifest_hash,
            "environment.corpus",
            "environment corpus hash mismatch",
        )
        authorization = environment.get("training_authorization") if isinstance(environment.get("training_authorization"), dict) else {}
        require_match(audit, authorization.get("gate_id"), "TRAIN-4", "environment.gate_id", "unexpected training authorization gate")
        require_match(audit, authorization.get("decision"), "Advance", "environment.gate_decision", "environment is not authorized to advance")
        require_match(
            audit,
            authorization.get("gate_report_sha256"),
            gate_hash,
            "environment.gate_report",
            "environment gate report hash mismatch",
        )

    if descriptor and environment:
        body = environment.get("body_schema") if isinstance(environment.get("body_schema"), dict) else {}
        for field, env_field in (
            ("body_schema_id", "id"),
            ("body_schema_revision", "revision"),
            ("body_schema_hash", "hash"),
            ("compiled_descriptor_hash", "compiled_descriptor_hash"),
        ):
            require_match(
                audit,
                descriptor.get(field),
                body.get(env_field),
                f"descriptor.{field}",
                f"descriptor {field} differs from environment body identity",
            )

    if gate:
        require_match(audit, gate.get("gate_id"), "TRAIN-4", "gate.id", "unexpected gate report")
        require_match(audit, gate.get("decision"), "Advance", "gate.decision", "TRAIN-4 did not advance")
        require_match(audit, gate.get("exact_checks"), "Pass", "gate.exact_checks", "TRAIN-4 exact checks did not pass")
        identities = gate.get("identities") if isinstance(gate.get("identities"), dict) else {}
        require_match(audit, identities.get("descriptor_file_sha256"), descriptor_hash, "gate.descriptor", "gate descriptor hash mismatch")
        require_match(audit, identities.get("corpus_manifest_sha256"), corpus_manifest_hash, "gate.corpus", "gate corpus hash mismatch")
        artifacts = gate.get("artifacts") if isinstance(gate.get("artifacts"), dict) else {}
        corpus_artifact = (
            artifacts.get("motion_corpus_manifest")
            if isinstance(artifacts.get("motion_corpus_manifest"), dict)
            else {}
        )
        require_match(
            audit,
            corpus_artifact.get("file_sha256"),
            corpus_file_hash,
            "gate.corpus_file",
            "gate corpus manifest file hash mismatch",
        )

    if generation:
        bindings = (
            ("reference_tracker_profile", environment_hash, "generation.environment_not_admitted"),
            ("compiled_descriptor_file", descriptor_hash, "generation.descriptor_not_admitted"),
            ("isaac_biomechanics_usd", usd_hash, "generation.usd_not_admitted"),
            ("train_4_gate_report", gate_hash, "generation.gate_not_admitted"),
            ("motion_corpus_manifest", corpus_manifest_hash, "generation.corpus_not_admitted"),
        )
        for kind, digest, code in bindings:
            audit.require(
                admitted(generation, kind=kind, sha256=digest),
                code,
                f"{kind} identity is not admitted by the generation",
                actual=digest,
            )
        audit.require(
            admitted(generation, kind=TRAINING_PROFILE_KINDS, sha256=training_hash),
            "generation.training_profile_not_admitted",
            "training profile is not admitted by the generation",
            actual=training_hash,
        )
        initialization = training.get("initialization") if training else None
        audit_checkpoint(checkpoint_path, initialization, generation, audit)

    generation_root = generation_path.parent
    audit.require(output_root.is_dir(), "output_root.missing", "output root must already exist", actual=str(output_root))
    audit.require(
        is_below(output_root, generation_root),
        "output_root.generation",
        "output root must be below the selected external generation",
        actual=str(output_root),
    )
    audit.require(
        not is_below(output_root, repository) and output_root != repository,
        "output_root.repository",
        "training artifacts must stay outside the Git repository",
        actual=str(output_root),
    )
    audit.require(bool(RUN_ID_RE.fullmatch(args.run_id)), "run_id.invalid", "run ID must be one safe path segment", actual=args.run_id)
    target = (output_root / args.run_id).resolve()
    audit.require(
        target.parent == output_root,
        "run_id.escape",
        "run ID resolves outside the output root",
        actual=str(target),
    )
    audit.require(not target.exists(), "run_target.exists", "run directory already exists", actual=str(target))

    command = [
        "python",
        "lab/scripts/isaac_reference_overfit.py",
        "--generation-manifest",
        str(generation_path),
        "--training-profile",
        str(training_path),
        "--environment-profile",
        str(environment_path),
        "--descriptor",
        str(descriptor_path),
        "--corpus-root",
        str(corpus_root),
        "--gate-report",
        str(gate_path),
        "--usd",
        str(usd_path),
        "--output-root",
        str(output_root),
        "--run-id",
        args.run_id,
    ]
    if checkpoint_path is not None:
        command.extend(("--initial-checkpoint", str(checkpoint_path)))

    for label, path, digest in (
        ("generation_manifest", generation_path, generation_hash),
        ("training_profile", training_path, training_hash),
        ("environment_profile", environment_path, environment_hash),
        ("descriptor", descriptor_path, descriptor_hash),
        ("corpus_manifest", corpus_manifest_path, corpus_file_hash),
        ("gate_report", gate_path, gate_hash),
        ("usd", usd_path, usd_hash),
        ("initial_checkpoint", checkpoint_path, audit.identities.get("initial_checkpoint_file_sha256")),
    ):
        if path is not None:
            require_stable_hash(path, digest, audit, label)

    result = {
        "schema": "nextengine.skill.training-preflight.v1",
        "ready": audit.ready,
        "mode": "read-only",
        "identities": audit.identities,
        "planned_run_directory": str(target),
        "planned_command": command,
        "issues": audit.issues,
        "claim_boundary": "PreflightOnly; no Isaac startup, training, evaluation, or stage advancement",
    }
    json.dump(result, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0 if audit.ready else 2


if __name__ == "__main__":
    raise SystemExit(main())
