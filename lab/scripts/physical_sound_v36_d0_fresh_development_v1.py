#!/usr/bin/env python3
"""Run the sealed V36 owner once on fresh official train/development roles."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, cast

import physical_sound_v36_d0_official_provider_v1 as official
import physical_sound_v36_e0_full_surrogate_seal_v1 as numeric
import physical_sound_v36_owner_contract_v1 as contract
import physical_sound_v36_terminal_publisher_v1 as publisher

PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v36-d0-fresh-development-profile.v1"
)
PROFILE_ID = "physical-sound-v36-d0-fresh-development-v1"
PROFILE_PATH = "lab/profiles/physical-sound-v36-d0-fresh-development.v1.json"
PROFILE_SHA256 = "f15835e665fb3a511dcda962e395d2db219ade0dc7a580cf9880ecedb4224456"
OWNER_PATH = "lab/scripts/physical_sound_v36_d0_fresh_development_v1.py"
CLAIM = numeric.OFFICIAL_D0_CLAIM
SCIENTIFIC_TERMINALS = {
    contract.TerminalDecision.PASS,
    contract.TerminalDecision.METRIC_REJECT,
    contract.TerminalDecision.HARD_GATE_REJECT,
    contract.TerminalDecision.RESOURCE_REJECT,
}


class D0RunnerError(RuntimeError):
    """The frozen D0 runner profile or pre-access boundary is invalid."""


@dataclass(frozen=True, slots=True)
class D0Context:
    profile: dict[str, Any]
    profile_data: bytes
    dependencies: dict[str, dict[str, object]]


@dataclass(frozen=True, slots=True)
class PreparedD0:
    context: D0Context
    verified: official.VerifiedD0PreAccess
    capability: contract.AccessCapability
    provider: official.OfficialD0Provider
    output: Path


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise D0RunnerError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    result: str = numeric.sha256_bytes(data)
    return result


def load_json_bytes(data: bytes, label: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except json.JSONDecodeError as error:
        raise D0RunnerError(f"invalid JSON: {label}") from error
    if not isinstance(value, dict):
        raise D0RunnerError(f"JSON must be an object: {label}")
    return cast(dict[str, Any], value)


def bound_file(path_text: str, expected_sha256: str) -> dict[str, object]:
    path = repository_root() / path_text
    if not path.is_file() or path.is_symlink():
        raise D0RunnerError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise D0RunnerError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def owner_identity() -> dict[str, object]:
    data = (repository_root() / OWNER_PATH).read_bytes()
    return {"bytes": len(data), "path": OWNER_PATH, "sha256": sha256_bytes(data)}


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise D0RunnerError("D0 profile must not be a symlink")
    data = path.read_bytes()
    profile = load_json_bytes(data, "D0 profile")
    if data != canonical_json(profile) or sha256_bytes(data) != PROFILE_SHA256:
        raise D0RunnerError("D0 profile is noncanonical or hash-drifted")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("claim") != CLAIM
    ):
        raise D0RunnerError("D0 profile identity drift")
    return profile, data


def load_context(profile_path: Path) -> D0Context:
    profile, data = load_profile(profile_path)
    dependencies = {
        name: bound_file(declaration["path"], declaration["sha256"])
        for name, declaration in sorted(profile["dependencies"].items())
    }
    protocol = profile["protocol"]
    dependencies["protocol"] = bound_file(protocol["path"], protocol["sha256"])
    if (
        profile["dependencies"]["official_provider"]["sha256"]
        != sha256_bytes((repository_root() / official.__file__).read_bytes())
        or profile["execution_seal"]["document_sha256"] != official.SEAL_DOCUMENT_SHA256
        or profile["execution_seal"]["payload_sha256"] != official.SEAL_PAYLOAD_SHA256
        or profile["dependencies"]["e0_numeric_owner"]["sha256"]
        != numeric.owner_identity()["sha256"]
        or profile["environment"]
        != numeric.load_context(repository_root() / numeric.PROFILE_PATH).profile[
            "environment"
        ]
    ):
        raise D0RunnerError("D0 bound identity projection drift")
    expected_access = profile["expected_access_per_process"]
    if (
        expected_access["train_target_rows"] != 6480
        or expected_access["development_target_rows"] != 4320
        or expected_access["official_d0_target_rows"] != 10800
        or expected_access["fresh_v36_truth_values_evaluated"] != 32400
        or profile["one_shot"]
        != {
            "candidate_freeze_status": "OfficialD0CandidateFrozenAfterPass",
            "candidate_required_only_for": "Pass",
            "processes": 2,
            "repeat_or_tune_after_post_access_terminal": False,
            "stdout_and_tree_byte_exact": True,
        }
        or profile["terminal_outcomes"]
        != [
            "Pass",
            "MetricReject",
            "HardGateReject",
            "ResourceReject",
            "OwnerFault",
        ]
    ):
        raise D0RunnerError("D0 one-shot contract drift")
    return D0Context(profile, data, dependencies)


def prepare(profile_path: Path, output: Path) -> PreparedD0:
    resolved_output, _ = publisher.validate_output(output, repository_root())
    context = load_context(profile_path)
    verified, capability, provider = official.prepare_official_d0()
    sealed = context.profile["execution_seal"]
    if (
        verified.seal.owner_sha256 != sealed["owner_sha256"]
        or verified.seal.profile_sha256 != sealed["profile_sha256"]
        or verified.seal.d0_rehearsal_root_sha256 != sealed["d0_rehearsal_root_sha256"]
        or verified.seal.h0_rehearsal_root_sha256 != sealed["h0_rehearsal_root_sha256"]
        or verified.receipt["fresh_v36_target_rows"] != 0
        or verified.receipt["official_d0_target_rows"] != 0
        or capability.provider_kind is not contract.ProviderKind.OFFICIAL_D0
        or capability.execution_seal != verified.seal
        or provider.kind is not contract.ProviderKind.OFFICIAL_D0
    ):
        raise D0RunnerError("official D0 preparation drift")
    return PreparedD0(context, verified, capability, provider, resolved_output)


def pre_access_report(prepared: PreparedD0) -> dict[str, object]:
    return {
        "claim": CLAIM,
        "dependencies": prepared.context.dependencies,
        "fresh_v36_target_rows": 0,
        "official_capabilities_issued": prepared.verified.receipt[
            "official_capabilities_issued"
        ],
        "official_d0_target_rows": 0,
        "owner": owner_identity(),
        "profile_sha256": PROFILE_SHA256,
        "provider": {
            "kind": prepared.provider.kind.value,
            "namespace": prepared.provider.namespace,
            "sha256": prepared.context.profile["dependencies"]["official_provider"][
                "sha256"
            ],
        },
        "schema": "nextengine.experimental-physical-sound-v36-d0-pre-access.v1",
        "seal_document_sha256": official.SEAL_DOCUMENT_SHA256,
        "seal_payload_sha256": official.SEAL_PAYLOAD_SHA256,
        "status": "PreparedBeforeFreshRoleMaterialization",
    }


def run(profile_path: Path, output: Path) -> tuple[dict[str, object], int]:
    prepared = prepare(profile_path, output)
    owner_run = numeric.execute_owner(
        prepared.verified.context,
        prepared.capability,
        prepared.provider,
        prepared.output,
        candidate_bundle=None,
    )
    trace = owner_run.trace
    if (
        trace.pipeline is not contract.PipelineKind.D0
        or trace.provider_kind is not contract.ProviderKind.OFFICIAL_D0
    ):
        raise D0RunnerError("official owner returned the wrong trace identity")
    if trace.terminal in SCIENTIFIC_TERMINALS:
        expected = prepared.context.profile["expected_access_per_process"]
        access = publisher.access_record(trace.access)
        official_access = numeric.official_access_record(trace)
        observed = {**access, **official_access}
        if any(observed.get(name) != value for name, value in expected.items()):
            raise D0RunnerError("complete D0 access receipt drift")
    if (trace.terminal is contract.TerminalDecision.PASS) != (
        owner_run.candidate_bundle is not None
    ):
        raise D0RunnerError("D0 candidate/terminal mismatch")
    report: dict[str, object] = {
        "access": publisher.access_record(trace.access),
        "candidate_frozen": owner_run.candidate_bundle is not None,
        "claim": CLAIM,
        "decision": trace.terminal.value,
        "numeric_owner_sha256": prepared.verified.seal.owner_sha256,
        "official_access": numeric.official_access_record(trace),
        "owner": owner_identity(),
        "profile_sha256": PROFILE_SHA256,
        "provider_sha256": prepared.context.profile["dependencies"][
            "official_provider"
        ]["sha256"],
        "schema": "nextengine.experimental-physical-sound-v36-d0-process.v1",
        "seal_payload_sha256": official.SEAL_PAYLOAD_SHA256,
        "status": "D0_POST_ACCESS_TERMINAL",
        "terminal_sha256": owner_run.report.get("terminal_sha256"),
        "tree_sha256": owner_run.report.get("tree_sha256"),
    }
    return report, contract.terminal_exit_code(trace.terminal)


def contract_reject(error: BaseException) -> dict[str, object]:
    return {
        "claim": CLAIM,
        "decision": contract.TerminalDecision.CONTRACT_REJECT.value,
        "error_class": type(error).__name__,
        "fresh_v36_target_rows": 0,
        "official_d0_target_rows": 0,
        "owner": owner_identity(),
        "profile_sha256": PROFILE_SHA256,
        "reason": str(error),
        "schema": "nextengine.experimental-physical-sound-v36-d0-process.v1",
        "status": "D0_PRE_ACCESS_CONTRACT_REJECT",
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", type=Path, default=Path(PROFILE_PATH))
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    try:
        report, exit_code = run(arguments.profile, arguments.output.absolute())
    except (
        D0RunnerError,
        official.OfficialProviderError,
        contract.ContractError,
        publisher.PublicationError,
    ) as error:
        report = contract_reject(error)
        exit_code = contract.terminal_exit_code(
            contract.TerminalDecision.CONTRACT_REJECT
        )
    print(canonical_json(report).decode(), end="")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
