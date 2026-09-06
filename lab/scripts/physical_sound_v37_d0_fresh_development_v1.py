#!/usr/bin/env python3
"""Run the sealed V37 owner once on fresh official train/development roles."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, cast

import physical_sound_v37_d0_official_provider_v1 as official
import physical_sound_v37_d0r_complete_entry_readiness_v1 as numeric
import physical_sound_v37_query_surface_contract_v1 as contract
import physical_sound_v37_terminal_publisher_v1 as publisher

PROFILE_PATH = "lab/profiles/physical-sound-v37-d0-fresh-development.v1.json"
PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v37-d0-fresh-development-profile.v1"
)
PROFILE_ID = "physical-sound-v37-d0-fresh-development-v1"
PROFILE_SHA256 = "2baf41bcb3734744c380c37effa2c1545cdde79afb219c5f994daa54b1ec305a"
OWNER_PATH = "lab/scripts/physical_sound_v37_d0_fresh_development_v1.py"
CLAIM = (
    "FRESH_V37_QSO_D0_ONE_SHOT_SYNTHETIC_DEVELOPMENT_EVIDENCE_ONLY / "
    "NO_METHOD_HOLDOUT_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_"
    "OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 1_048_576
SCIENTIFIC_TERMINALS = {
    contract.TerminalDecision.PASS,
    contract.TerminalDecision.METRIC_REJECT,
    contract.TerminalDecision.HARD_GATE_REJECT,
    contract.TerminalDecision.RESOURCE_REJECT,
}


class D0RunnerError(RuntimeError):
    """The frozen V37 D0 runner profile or execution is invalid."""


@dataclass(frozen=True, slots=True)
class D0Context:
    profile: dict[str, Any]
    profile_data: bytes
    dependencies: dict[str, dict[str, object]]


@dataclass(frozen=True, slots=True)
class PreparedD0:
    context: D0Context
    verified: official.VerifiedD0PreAccess
    capability: contract.AccessCapabilityV1
    provider: official.OfficialD0Provider
    publication: numeric.PublicationIdentity
    output: Path


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise D0RunnerError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return cast(str, numeric.sha256_bytes(data))


def load_json_bytes(data: bytes, label: str) -> dict[str, Any]:
    try:
        value = cast(object, json.loads(data))
    except json.JSONDecodeError as error:
        raise D0RunnerError(f"invalid JSON: {label}") from error
    if not isinstance(value, dict):
        raise D0RunnerError(f"JSON root must be an object: {label}")
    result = cast(dict[str, Any], value)
    if canonical_json(result) != data:
        raise D0RunnerError(f"JSON is not canonical: {label}")
    return result


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


def load_context(profile_path: Path) -> D0Context:
    if not profile_path.is_file() or profile_path.is_symlink():
        raise D0RunnerError("D0 profile must be a regular file")
    data = profile_path.read_bytes()
    if (
        not data
        or len(data) > MAX_PROFILE_BYTES
        or sha256_bytes(data) != PROFILE_SHA256
    ):
        raise D0RunnerError("D0 profile size or identity drift")
    profile = load_json_bytes(data, "D0 profile")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("claim") != CLAIM
        or profile.get("baseline_commit") != "d1e30550"
        or profile.get("scientific_profile_sha256") != numeric.PROFILE_SHA256
    ):
        raise D0RunnerError("D0 profile identity drift")
    declarations = cast(dict[str, dict[str, str]], profile["dependencies"])
    if list(declarations) != sorted(declarations):
        raise D0RunnerError("D0 dependencies are not canonical")
    dependencies = {
        name: bound_file(row["path"], row["sha256"])
        for name, row in declarations.items()
    }
    seals = cast(dict[str, Any], profile["execution_seals"])
    d0r_seal = cast(dict[str, Any], seals["d0r"])
    t0_seal = cast(dict[str, Any], seals["t0"])
    if (
        d0r_seal
        != {
            "document_sha256": official.D0R_SEAL_DOCUMENT_SHA256,
            "owner_sha256": official.D0R_OWNER_SHA256,
            "payload_sha256": official.D0R_SEAL_PAYLOAD_SHA256,
            "status": "SealedBeforeOfficialTargetAccess",
        }
        or seals.get("e0_payload_sha256") != official.E0_SEAL_PAYLOAD_SHA256
        or t0_seal
        != {
            "document_sha256": official.T0_SEAL_DOCUMENT_SHA256,
            "payload_sha256": official.T0_SEAL_PAYLOAD_SHA256,
            "status": "SealedBeforeOfficialTargetAccess",
        }
        or profile["environment"]
        != numeric.load_context(repository_root() / numeric.PROFILE_PATH).environment
    ):
        raise D0RunnerError("D0 seal or environment projection drift")
    expected = cast(dict[str, int], profile["expected_access_per_process"])
    if expected != {
        "development_target_rows": 4320,
        "fresh_v37_official_truth_values_evaluated": 32400,
        "method_holdout_target_rows": 0,
        "network_requests": 0,
        "official_capabilities_issued": 1,
        "official_d0_target_rows": 10800,
        "official_h0_target_rows": 0,
        "official_truth_rows_evaluated": 10800,
        "protected_signal_values_decoded": 0,
        "provider_calls": 2,
        "real_signal_values_decoded": 0,
        "target_rows_accessed": 10800,
        "train_target_rows": 6480,
    }:
        raise D0RunnerError("D0 expected access drift")
    if profile["one_shot"] != {
        "candidate_required_only_for": "Pass",
        "method_holdout_allowed_only_after_exact_d0_pass": True,
        "processes": 2,
        "repeat_or_tune_after_post_access_terminal": False,
        "stderr_empty": True,
        "stdout_and_tree_byte_exact": True,
    } or profile["terminal_outcomes"] != [
        "Pass",
        "MetricReject",
        "HardGateReject",
        "ResourceReject",
        "OwnerFault",
    ]:
        raise D0RunnerError("D0 one-shot or terminal contract drift")
    return D0Context(profile, data, dependencies)


def publication_identity(context: D0Context) -> numeric.PublicationIdentity:
    publication = cast(dict[str, str], context.profile["publication"])
    resources = cast(dict[str, int], context.profile["resources"])
    return numeric.PublicationIdentity(
        claim=CLAIM,
        profile_sha256=PROFILE_SHA256,
        evidence_schema=publication["evidence_schema"],
        report_schema=publication["report_schema"],
        report_status=publication["report_status"],
        next_authorized_action=publication["next_authorized_action"],
        maximum_output_bytes=resources["maximum_output_bytes"],
    )


def prepare(profile_path: Path, output: Path) -> PreparedD0:
    resolved_output, _staging = publisher.validate_output(output, repository_root())
    context = load_context(profile_path)
    verified, capability, provider = official.prepare_official_d0()
    if (
        verified.receipt["official_capabilities_issued"] != 0
        or verified.receipt["official_d0_target_rows"] != 0
        or verified.receipt["official_truth_rows_evaluated"] != 0
        or capability.provider_kind is not contract.ProviderKind.OFFICIAL_D0
        or capability.execution_seal_sha256 != official.D0R_SEAL_PAYLOAD_SHA256
        or provider.evidence()["official_d0_target_rows"] != 0
    ):
        raise D0RunnerError("official D0 preparation opened or changed authority")
    return PreparedD0(
        context,
        verified,
        capability,
        provider,
        publication_identity(context),
        resolved_output,
    )


def preflight(profile_path: Path) -> dict[str, object]:
    context = load_context(profile_path)
    verified = official.verify_pre_access()
    return {
        "claim": CLAIM,
        "dependencies": context.dependencies,
        "official_access": verified.receipt,
        "owner": owner_identity(),
        "profile_sha256": PROFILE_SHA256,
        "schema": "nextengine.experimental-physical-sound-v37-d0-preflight.v1",
        "status": "D0_PREFLIGHT_PASS_ZERO_OFFICIAL_VALUES",
    }


def complete_access_record(
    result: numeric.OwnerResult, provider: official.OfficialD0Provider
) -> dict[str, object]:
    access = publisher.access_record(result.trace.access)
    evidence = provider.evidence()
    return {**access, **evidence}


def execute_prepared(prepared: PreparedD0) -> tuple[dict[str, object], int]:
    result = numeric.execute_with_provider(
        prepared.verified.d0r_context,
        prepared.output,
        prepared.provider,
        prepared.capability,
        prepared.publication,
    )
    trace = result.trace
    if (
        trace.pipeline is not contract.PipelineKind.D0
        or trace.provider_kind is not contract.ProviderKind.OFFICIAL_D0
        or trace.terminal not in SCIENTIFIC_TERMINALS
    ):
        raise D0RunnerError("official D0 returned an invalid terminal trace")
    observed = complete_access_record(result, prepared.provider)
    expected = cast(
        dict[str, int], prepared.context.profile["expected_access_per_process"]
    )
    if any(observed.get(name) != value for name, value in expected.items()):
        raise D0RunnerError("official D0 access receipt drift")
    files = numeric.file_tree(prepared.output)
    candidate_frozen = trace.terminal is contract.TerminalDecision.PASS
    expected_files = (
        {
            "candidate-bundle.json",
            "candidate-freeze.json",
            "evidence.json",
            "terminal.json",
        }
        if candidate_frozen
        else {"evidence.json", "rejected-candidate.json", "terminal.json"}
    )
    if set(files) != expected_files:
        raise D0RunnerError("official D0 terminal payload closure drift")
    report: dict[str, object] = {
        "access": observed,
        "candidate_frozen": candidate_frozen,
        "claim": CLAIM,
        "decision": trace.terminal.value,
        "d0r_owner_sha256": official.D0R_OWNER_SHA256,
        "d0r_seal_payload_sha256": official.D0R_SEAL_PAYLOAD_SHA256,
        "owner": owner_identity(),
        "profile_sha256": PROFILE_SHA256,
        "provider_sha256": prepared.context.profile["dependencies"][
            "official_provider"
        ]["sha256"],
        "receipt": numeric.e0.receipt_record(result.receipt),
        "schema": "nextengine.experimental-physical-sound-v37-d0-process.v1",
        "status": "D0_POST_ACCESS_TERMINAL",
        "t0_seal_payload_sha256": official.T0_SEAL_PAYLOAD_SHA256,
    }
    return report, 0 if candidate_frozen else 1


def contract_reject(error: BaseException) -> dict[str, object]:
    return {
        "claim": CLAIM,
        "decision": contract.TerminalDecision.CONTRACT_REJECT.value,
        "error_class": type(error).__name__,
        "official_d0_target_rows": 0,
        "official_truth_rows_evaluated": 0,
        "owner": owner_identity(),
        "profile_sha256": PROFILE_SHA256,
        "reason": str(error),
        "schema": "nextengine.experimental-physical-sound-v37-d0-process.v1",
        "status": "D0_PRE_ACCESS_CONTRACT_REJECT",
    }


def owner_fault(
    error: BaseException, provider: official.OfficialD0Provider
) -> dict[str, object]:
    return {
        "access": provider.evidence(),
        "candidate_frozen": False,
        "claim": CLAIM,
        "decision": contract.TerminalDecision.OWNER_FAULT.value,
        "error_class": type(error).__name__,
        "owner": owner_identity(),
        "profile_sha256": PROFILE_SHA256,
        "reason": str(error),
        "schema": "nextengine.experimental-physical-sound-v37-d0-process.v1",
        "status": "D0_POST_ACCESS_OWNER_FAULT",
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", type=Path, default=Path(PROFILE_PATH))
    parser.add_argument("--output", type=Path)
    parser.add_argument("--preflight", action="store_true")
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    if arguments.preflight:
        try:
            report = preflight(arguments.profile)
        except (
            D0RunnerError,
            official.OfficialProviderError,
            numeric.D0ReadinessError,
            contract.ContractError,
            publisher.PublicationError,
        ) as error:
            report = contract_reject(error)
            print(canonical_json(report).decode(), end="")
            return 2
        print(canonical_json(report).decode(), end="")
        return 0
    if arguments.output is None:
        report = contract_reject(D0RunnerError("D0 execution requires --output"))
        print(canonical_json(report).decode(), end="")
        return 2
    try:
        prepared = prepare(arguments.profile, arguments.output.absolute())
    except (
        D0RunnerError,
        official.OfficialProviderError,
        numeric.D0ReadinessError,
        contract.ContractError,
        publisher.PublicationError,
    ) as error:
        print(canonical_json(contract_reject(error)).decode(), end="")
        return 2
    try:
        report, exit_code = execute_prepared(prepared)
    except (
        D0RunnerError,
        official.OfficialProviderError,
        numeric.D0ReadinessError,
        contract.ContractError,
        publisher.PublicationError,
    ) as error:
        report = owner_fault(error, prepared.provider)
        exit_code = 1
    print(canonical_json(report).decode(), end="")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
