#!/usr/bin/env python3
"""Run frozen colored-residual discovery stages without crossing boundaries."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

PREFLIGHT_SCHEMA = (
    "nextengine.experimental-realimpact-colored-residual-pipeline-preflight.report.v1"
)
STAGE_SCHEMAS = {
    "identity": (
        "nextengine.experimental-realimpact-colored-residual-pipeline-identity.report.v1"
    ),
    "tail": (
        "nextengine.experimental-realimpact-colored-residual-pipeline-tail.report.v1"
    ),
    "header": (
        "nextengine.experimental-realimpact-colored-residual-pipeline-header.report.v1"
    ),
}
REVISION = "colored-residual-boundary-orchestration-v1"
SOURCE_REFERENCES = [
    {
        "path": "lab/scripts/physical_sound_realimpact_colored_residual_preregistration.py",
        "sha256": "fe8a12f516ee8f85dc515c23cf2b7f67cc6bcc4d12a2b7f20c7f69d0626cebf5",
    },
    {
        "path": "lab/scripts/physical_sound_realimpact_colored_residual_discovery.py",
        "sha256": "898d1af13bf4dda4c3efed77b611ad757ccfe71f0548532e29c1a38c1a6c7c58",
    },
    {
        "path": "lab/scripts/physical_sound_realimpact_colored_residual_tail_discovery.py",
        "sha256": "5987b04807fe21177fd017165ef299a87ca8ddac6fa14f818209b67644a8fb74",
    },
    {
        "path": "lab/scripts/physical_sound_realimpact_colored_residual_header_discovery.py",
        "sha256": "5666a78eb8e9806fe0fff0a6215189efd8dbe3175b9d58c305ae7caad6447538",
    },
]
FROZEN_ARTIFACTS = [
    {
        "label": "successor_manifest",
        "sha256": "f03e428ee9b6005b96519869541fd27a301e0343a4e0f800b5d74f770d3e564a",
        "bytes": 7_784,
        "schema": (
            "nextengine.experimental-realimpact-colored-residual-successor.manifest.v1"
        ),
    },
    {
        "label": "successor_preflight",
        "sha256": "4c754f0630d16ade6fd7de3479fc15bb54a193a9f928b72eaf3e1342424da360",
        "bytes": 10_341,
        "decision": "RealImpactColoredResidualSuccessorProtocolFrozen",
    },
    {
        "label": "identity_manifest",
        "sha256": "55be5ed2e71657a344251ccf6068720b0eb0a7f329866e680ca8fffc61b01abe",
        "bytes": 2_405,
        "schema": (
            "nextengine.experimental-realimpact-colored-residual-discovery.manifest.v1"
        ),
    },
    {
        "label": "identity_preflight",
        "sha256": "0d38bfb1e40838880d4f7955a74d8a9c14e0d480cff270fe722bb5cb56bf5d7b",
        "bytes": 2_745,
        "decision": "RealImpactColoredResidualArchiveIdentityDiscoveryFrozen",
    },
]


class PipelineError(RuntimeError):
    """The frozen colored-residual pipeline boundary failed."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--stage", required=True, choices=["preflight", *sorted(STAGE_SCHEMAS)]
    )
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--pipeline-preflight", type=Path)
    parser.add_argument("--successor-manifest", type=Path)
    parser.add_argument("--successor-preflight", type=Path)
    parser.add_argument("--identity-manifest", type=Path)
    parser.add_argument("--identity-preflight", type=Path)
    parser.add_argument("--identity-audit", type=Path)
    parser.add_argument("--tail-audit", type=Path)
    return parser.parse_args()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise PipelineError(f"{label} must be an external file: {resolved}")
    return resolved


def external_output(root: Path, path: Path, *, directory: bool) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise PipelineError(f"output must be new and external: {resolved}")
    if directory:
        resolved.mkdir()
    return resolved


def read_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    data = path.read_bytes()
    try:
        parsed = json.loads(data)
    except json.JSONDecodeError as error:
        raise PipelineError(f"parse {label}: {error}") from error
    return data, parsed


def validate_sources(root: Path) -> list[dict[str, Any]]:
    result = []
    for reference in SOURCE_REFERENCES:
        path = (root / reference["path"]).resolve(strict=True)
        if (
            not path.is_file()
            or not path.is_relative_to(root)
            or sha256_file(path) != reference["sha256"]
        ):
            raise PipelineError(f"bound source changed: {reference['path']}")
        result.append({**reference, "bytes": path.stat().st_size})
    return result


def validate_frozen_artifact(
    root: Path, path: Path, reference: dict[str, Any]
) -> dict[str, Any]:
    resolved = external_file(root, path, reference["label"])
    data, parsed = read_json(resolved, reference["label"])
    if len(data) != reference["bytes"] or sha256_bytes(data) != reference["sha256"]:
        raise PipelineError(f"frozen artifact changed: {reference['label']}")
    if (expected := reference.get("schema")) and parsed.get("schema") != expected:
        raise PipelineError(f"frozen schema changed: {reference['label']}")
    if (expected := reference.get("decision")) and parsed.get("decision") != expected:
        raise PipelineError(f"frozen decision changed: {reference['label']}")
    return {key: value for key, value in reference.items() if key != "schema"}


def frozen_input_paths(arguments: argparse.Namespace) -> list[Path | None]:
    return [
        arguments.successor_manifest,
        arguments.successor_preflight,
        arguments.identity_manifest,
        arguments.identity_preflight,
    ]


def validate_frozen_inputs(
    root: Path, arguments: argparse.Namespace
) -> list[dict[str, Any]]:
    paths = frozen_input_paths(arguments)
    if any(path is None for path in paths):
        raise PipelineError("preflight/identity require all four frozen inputs")
    return [
        validate_frozen_artifact(root, path, reference)
        for path, reference in zip(paths, FROZEN_ARTIFACTS, strict=True)
        if path is not None
    ]


def preflight_report(
    root: Path, runner_sha256: str, artifacts: list[dict[str, Any]]
) -> dict[str, Any]:
    return {
        "schema": PREFLIGHT_SCHEMA,
        "status": "Validated",
        "decision": "RealImpactColoredResidualBoundaryPipelineFrozen",
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "sources": validate_sources(root),
        "artifacts": artifacts,
        "boundaries": [
            {
                "stage": "identity",
                "acquisition": "exactly three body-free HEAD requests once",
                "offline_repeats": ["audit-a", "audit-b"],
                "forbidden": ["range", "member", "shadow"],
            },
            {
                "stage": "tail",
                "prerequisite": "successful byte-identical identity audits",
                "acquisition": "exactly three 65536-byte ranges once",
                "offline_repeats": ["preflight-a", "preflight-b", "audit-a", "audit-b"],
                "forbidden": ["local_header", "member", "shadow"],
            },
            {
                "stage": "header",
                "prerequisite": "successful byte-identical tail audits",
                "acquisition": "exactly three 30-byte ranges once",
                "offline_repeats": ["preflight-a", "preflight-b", "audit-a", "audit-b"],
                "forbidden": ["member", "shadow"],
            },
        ],
        "automatic_retry": False,
        "cross_boundary_auto_continue": False,
        "network_requests": 0,
        "range_requests": 0,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": "run only --stage identity when shell network is available",
    }


def validate_pipeline_preflight(
    root: Path, path: Path, runner_sha256: str
) -> tuple[bytes, dict[str, Any]]:
    resolved = external_file(root, path, "pipeline preflight")
    data, report = read_json(resolved, "pipeline preflight")
    expected = preflight_report(
        root,
        runner_sha256,
        [
            {key: value for key, value in reference.items() if key != "schema"}
            for reference in FROZEN_ARTIFACTS
        ],
    )
    if report != expected:
        raise PipelineError("pipeline preflight changed")
    return data, report


def run_child(root: Path, script: str, arguments: list[str]) -> None:
    path = root / "lab/scripts" / script
    completed = subprocess.run(
        [sys.executable, str(path), *arguments],
        cwd=root,
        capture_output=True,
        check=False,
        text=True,
    )
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip()
        raise PipelineError(
            f"child stage failed: {script}, exit {completed.returncode}: {detail[-1000:]}"
        )


def require_identity_arguments(arguments: argparse.Namespace) -> None:
    if (
        arguments.pipeline_preflight is None
        or any(path is None for path in frozen_input_paths(arguments))
        or arguments.identity_audit is not None
        or arguments.tail_audit is not None
    ):
        raise PipelineError(
            "identity requires pipeline preflight and four frozen inputs only"
        )


def identity_stage(
    root: Path, arguments: argparse.Namespace, runner_sha256: str, output: Path
) -> dict[str, Any]:
    require_identity_arguments(arguments)
    preflight_bytes, _ = validate_pipeline_preflight(
        root, arguments.pipeline_preflight, runner_sha256
    )
    validate_frozen_inputs(root, arguments)
    acquisition_path = output / "acquisition.json"
    child_args = [
        "--stage",
        "acquire",
        "--output",
        str(acquisition_path),
        "--manifest",
        str(arguments.identity_manifest),
        "--preregistration-manifest",
        str(arguments.successor_manifest),
        "--preregistration-preflight",
        str(arguments.successor_preflight),
        "--preflight",
        str(arguments.identity_preflight),
    ]
    run_child(
        root,
        "physical_sound_realimpact_colored_residual_discovery.py",
        child_args,
    )
    acquisition_bytes, acquisition = read_json(acquisition_path, "identity acquisition")
    supported = (
        acquisition.get("decision")
        == "RealImpactColoredResidualArchiveIdentitiesDiscovered"
    )
    audits = []
    if supported:
        for label in ["a", "b"]:
            audit_path = output / f"audit-{label}.json"
            run_child(
                root,
                "physical_sound_realimpact_colored_residual_discovery.py",
                [
                    "--stage",
                    "audit",
                    "--output",
                    str(audit_path),
                    "--manifest",
                    str(arguments.identity_manifest),
                    "--preregistration-manifest",
                    str(arguments.successor_manifest),
                    "--preregistration-preflight",
                    str(arguments.successor_preflight),
                    "--acquisition",
                    str(acquisition_path),
                ],
            )
            audits.append(audit_path.read_bytes())
        if audits[0] != audits[1]:
            raise PipelineError("identity audits are not byte-identical")
    return {
        "schema": STAGE_SCHEMAS["identity"],
        "status": "Validated",
        "decision": (
            "RealImpactColoredResidualIdentityBoundarySupported"
            if supported
            else "RealImpactColoredResidualIdentityBoundaryRejected"
        ),
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "pipeline_preflight_sha256": sha256_bytes(preflight_bytes),
        "acquisition_sha256": sha256_bytes(acquisition_bytes),
        "child_decision": acquisition.get("decision"),
        "audit_sha256": sha256_bytes(audits[0]) if audits else None,
        "network_requests_attempted": acquisition.get("network_requests_attempted", 0),
        "additional_requests_allowed": 0,
        "range_requests": 0,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            "start a separate tail stage from audit-a.json"
            if supported
            else "stop without retry, range request or object substitution"
        ),
    }


def require_single_upstream(
    arguments: argparse.Namespace, *, identity: bool
) -> tuple[Path, Path]:
    upstream = arguments.identity_audit if identity else arguments.tail_audit
    other = arguments.tail_audit if identity else arguments.identity_audit
    if (
        arguments.pipeline_preflight is None
        or upstream is None
        or other is not None
        or any(path is not None for path in frozen_input_paths(arguments))
    ):
        label = "identity-audit" if identity else "tail-audit"
        raise PipelineError(f"stage requires only pipeline preflight and {label}")
    return arguments.pipeline_preflight, upstream


def range_stage(
    root: Path,
    arguments: argparse.Namespace,
    runner_sha256: str,
    output: Path,
    *,
    stage: str,
) -> dict[str, Any]:
    is_tail = stage == "tail"
    pipeline_path, upstream_path = require_single_upstream(arguments, identity=is_tail)
    upstream_path = external_file(
        root, upstream_path, "identity audit" if is_tail else "tail audit"
    )
    preflight_bytes, _ = validate_pipeline_preflight(root, pipeline_path, runner_sha256)
    script = (
        "physical_sound_realimpact_colored_residual_tail_discovery.py"
        if is_tail
        else "physical_sound_realimpact_colored_residual_header_discovery.py"
    )
    upstream_flag = "--identity-audit" if is_tail else "--tail-audit"
    manifest_path = output / "manifest.json"
    run_child(
        root,
        script,
        [
            "--stage",
            "manifest",
            "--output",
            str(manifest_path),
            upstream_flag,
            str(upstream_path),
        ],
    )
    preflights = []
    for label in ["a", "b"]:
        path = output / f"preflight-{label}.json"
        run_child(
            root,
            script,
            [
                "--stage",
                "preflight",
                "--output",
                str(path),
                "--manifest",
                str(manifest_path),
            ],
        )
        preflights.append(path.read_bytes())
    if preflights[0] != preflights[1]:
        raise PipelineError(f"{stage} preflights are not byte-identical")
    acquisition_path = output / "acquisition"
    run_child(
        root,
        script,
        [
            "--stage",
            "acquire",
            "--output",
            str(acquisition_path),
            "--manifest",
            str(manifest_path),
            "--preflight",
            str(output / "preflight-a.json"),
        ],
    )
    acquisition_bytes, acquisition = read_json(
        acquisition_path / "report.json", f"{stage} acquisition"
    )
    supported_decision = (
        "RealImpactColoredResidualExactTailsAcquired"
        if is_tail
        else "RealImpactColoredResidualExactLocalHeadersAcquired"
    )
    supported = acquisition.get("decision") == supported_decision
    audits = []
    if supported:
        for label in ["a", "b"]:
            path = output / f"audit-{label}.json"
            run_child(
                root,
                script,
                [
                    "--stage",
                    "audit",
                    "--output",
                    str(path),
                    "--manifest",
                    str(manifest_path),
                    "--acquisition",
                    str(acquisition_path),
                ],
            )
            audits.append(path.read_bytes())
        if audits[0] != audits[1]:
            raise PipelineError(f"{stage} audits are not byte-identical")
    return {
        "schema": STAGE_SCHEMAS[stage],
        "status": "Validated",
        "decision": (
            f"RealImpactColoredResidual{stage.title()}BoundarySupported"
            if supported
            else f"RealImpactColoredResidual{stage.title()}BoundaryRejected"
        ),
        "revision": REVISION,
        "runner_sha256": runner_sha256,
        "pipeline_preflight_sha256": sha256_bytes(preflight_bytes),
        "upstream_audit_sha256": sha256_file(upstream_path),
        "manifest_sha256": sha256_file(manifest_path),
        "preflight_sha256": sha256_bytes(preflights[0]),
        "acquisition_sha256": sha256_bytes(acquisition_bytes),
        "child_decision": acquisition.get("decision"),
        "audit_sha256": sha256_bytes(audits[0]) if audits else None,
        "network_requests_attempted": acquisition.get("network_requests_attempted", 0),
        "range_requests_attempted": acquisition.get("network_requests_attempted", 0),
        "additional_requests_allowed": 0,
        "member_payload_bytes_read": 0,
        "shadow_payload_bytes_read": 0,
        "quality_domain_or_runtime_admission": False,
        "authored_clip_fallback_required": True,
        "next_action": (
            (
                "start a separate header stage from audit-a.json"
                if is_tail
                else "stop and freeze a payload protocol before any member access"
            )
            if supported
            else "stop without retry, range growth or later-stage access"
        ),
    }


def validate_argument_shape(arguments: argparse.Namespace) -> None:
    if arguments.stage == "preflight":
        if (
            arguments.pipeline_preflight is not None
            or arguments.identity_audit is not None
            or arguments.tail_audit is not None
        ):
            raise PipelineError("preflight accepts only the four frozen inputs")
    elif arguments.stage == "identity":
        require_identity_arguments(arguments)
    elif arguments.stage == "tail":
        require_single_upstream(arguments, identity=True)
    else:
        require_single_upstream(arguments, identity=False)


def main() -> int:
    arguments = parse_arguments()
    validate_argument_shape(arguments)
    root = Path(__file__).resolve().parents[2]
    runner_sha256 = sha256_file(Path(__file__).resolve())
    if arguments.stage == "preflight":
        artifacts = validate_frozen_inputs(root, arguments)
        output = external_output(root, arguments.output, directory=False)
        report = preflight_report(root, runner_sha256, artifacts)
    else:
        output = external_output(root, arguments.output, directory=True)
        if arguments.stage == "identity":
            report = identity_stage(root, arguments, runner_sha256, output)
        else:
            report = range_stage(
                root,
                arguments,
                runner_sha256,
                output,
                stage=arguments.stage,
            )
    report_bytes = canonical_json(report)
    target = output if arguments.stage == "preflight" else output / "report.json"
    target.write_bytes(report_bytes)
    print(f"colored residual pipeline {arguments.stage}: {target}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print(f"network requests: {report.get('network_requests_attempted', 0)}")
    print(
        "range requests: "
        f"{report.get('range_requests_attempted', report.get('range_requests', 0))}"
    )
    print("member payload bytes read: 0")
    print("shadow payload bytes read: 0")
    print("quality/domain/runtime admission: false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
