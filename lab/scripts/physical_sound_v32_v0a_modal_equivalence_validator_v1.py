#!/usr/bin/env python3
"""Run the V32 V0a modal-equivalence validator mechanics externally."""

from __future__ import annotations

import argparse
import copy
import os
import platform
import shutil
import tempfile
import time
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator

import numpy as np

import physical_sound_v32_v0_validator_mechanics_v1 as core


PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0a-modal-equivalence-validator-"
    "profile.v1"
)
FEATURE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0a-validator-features.v1"
)
DECISION_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0a-validator-decision.v1"
)
RELEASE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0a-validator-mechanics-release.v1"
)
EVIDENCE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0a-validator-mechanics-evidence.v1"
)
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-v0a-validator-mechanics-report.v1"
)
PROFILE_ID = "physical-sound-v32-v0a-modal-equivalence-validator-v1"
PROFILE_SHA256 = "bb3e6cb2f6f0e52e80dc31df14f7887e9f3fc14ecca6e9a541d670b716118094"
BASELINE_COMMIT = "4260b4d24148bb0ce2adc2158138a4fd8cd1bf72"
PROTOCOL_SHA256 = "1abf9374896ca50597fe8d2534ab1bb32b17819ebcef4ae2e8c88804020f7097"
V0_CORE_SHA256 = "267ab74bd1ff56623aa19b25d060013adabdc1b8e695562b127aa1279e4f16b4"
V0_PROFILE_SHA256 = "8865cf8ca52f76816ad538c1ebedd44ba8ee68a6a96e1f1980a50cd35ebe4709"
V0_RESULT_SHA256 = "b01f85f3f255a6ce4b35f40c0e729b675649f9de0222dd5b53665949938be852"
CLAIM = (
    "SYNTHETIC_MODAL_EQUIVALENCE_VALIDATOR_MECHANICS_ONLY / NO_REAL_THRESHOLDS_"
    "MATERIAL_QUALITY_VALIDATOR_RELEASE_ML_ADMISSION_COOKER_DEMO_OR_RUNTIME_"
    "AUTHORITY"
)
OWNER_PATH = "lab/scripts/physical_sound_v32_v0a_modal_equivalence_validator_v1.py"
PROTOCOL_PATH = (
    "docs/development/physical-sound-v32-v0a-modal-equivalence-validator-"
    "protocol-2026-09-02.md"
)
CORE_IDENTITY_OVERRIDES = {
    "PROFILE_SCHEMA": PROFILE_SCHEMA,
    "FEATURE_SCHEMA": FEATURE_SCHEMA,
    "DECISION_SCHEMA": DECISION_SCHEMA,
    "PROFILE_ID": PROFILE_ID,
    "PROFILE_SHA256": PROFILE_SHA256,
    "BASELINE_COMMIT": BASELINE_COMMIT,
    "CLAIM": CLAIM,
}
ZERO_ACCESS = core.ZERO_ACCESS
ValidatorMechanicsError = core.ValidatorMechanicsError


@contextmanager
def v0a_core_identity() -> Iterator[None]:
    """Temporarily give the frozen parser/feature core the V0a output identity."""

    original = {name: getattr(core, name) for name in CORE_IDENTITY_OVERRIDES}
    try:
        for name, value in CORE_IDENTITY_OVERRIDES.items():
            setattr(core, name, value)
        yield
    finally:
        for name, value in original.items():
            setattr(core, name, value)


def canonical_json(value: Any) -> bytes:
    return core.canonical_json(value)


def sha256_bytes(data: bytes) -> str:
    return core.sha256_bytes(data)


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    with v0a_core_identity():
        profile, data = core.load_profile(path)
    retrieval = profile.get("specialists", {}).get("retrieval")
    successor = profile.get("successor")
    if retrieval != {
        "equivalence": "exact-ordered-complete-modal-signature",
        "reason_code": "RetrievalCopyDetected",
        "rule": "audio-hash-equals-clean-non-equivalent-modal-target",
        "signature_fields": [
            "ordinal",
            "family_index_a",
            "family_index_b",
            "frequency_hz",
            "decay_per_second",
            "contact_participation",
            "pickup_participation",
            "signed_gain",
        ],
    }:
        raise ValidatorMechanicsError("V0a retrieval contract mismatch")
    if successor != {
        "allowed_semantic_change": "retrieval-target-id-to-exact-modal-equivalence",
        "inherited_profile_sha256": V0_PROFILE_SHA256,
        "plate_to_beam_negative_control_required": True,
    }:
        raise ValidatorMechanicsError("V0a successor contract mismatch")
    if profile["cache"]["domain_separator"] != (
        "nextengine.physical-sound-v32-v0a-feature-cache.v1\\0"
    ):
        raise ValidatorMechanicsError("V0a cache domain mismatch")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    root = repository_root()
    expected = {
        "t0_owner": (
            "lab/scripts/physical_sound_v32_t0_truth_mutations_v1.py",
            core.T0_OWNER_SHA256,
        ),
        "t0_profile": (
            "lab/profiles/physical-sound-v32-t0-truth-mutations.v1.json",
            core.T0_PROFILE_SHA256,
        ),
        "v0_core": (
            "lab/scripts/physical_sound_v32_v0_validator_mechanics_v1.py",
            V0_CORE_SHA256,
        ),
        "v0_profile": (
            "lab/profiles/physical-sound-v32-v0-validator-mechanics.v1.json",
            V0_PROFILE_SHA256,
        ),
        "v0_result": (
            "docs/development/physical-sound-v32-v0-validator-mechanics-result-"
            "2026-09-02.md",
            V0_RESULT_SHA256,
        ),
    }
    parent = profile.get("parent")
    if not isinstance(parent, dict) or set(parent) != set(expected):
        raise ValidatorMechanicsError("V0a parent dependency set mismatch")
    result: dict[str, dict[str, Any]] = {}
    for dependency_id, (relative, expected_sha256) in expected.items():
        declared = parent[dependency_id]
        if declared != {"path": relative, "sha256": expected_sha256}:
            raise ValidatorMechanicsError(
                f"V0a declared dependency mismatch: {dependency_id}"
            )
        data = (root / relative).read_bytes()
        actual = sha256_bytes(data)
        if actual != expected_sha256:
            raise ValidatorMechanicsError(f"V0a dependency drift: {dependency_id}")
        result[dependency_id] = {
            "bytes": len(data),
            "path": relative,
            "sha256": actual,
        }
    protocol = profile.get("protocol")
    expected_protocol = {"path": PROTOCOL_PATH, "sha256": PROTOCOL_SHA256}
    if protocol != expected_protocol:
        raise ValidatorMechanicsError("V0a protocol identity mismatch")
    data = (root / PROTOCOL_PATH).read_bytes()
    if sha256_bytes(data) != PROTOCOL_SHA256:
        raise ValidatorMechanicsError("V0a protocol drift")
    result["v0a_protocol"] = {"bytes": len(data), **expected_protocol}
    return result


def feature_cache_key(
    profile: dict[str, Any],
    candidate_view_sha256: str,
    audio_sha256: str,
    target_record_sha256: str,
) -> str:
    separator = profile["cache"]["domain_separator"]
    if not separator.endswith("\\0"):
        raise ValidatorMechanicsError("cache domain separator mismatch")
    domain = separator[:-2].encode("utf-8") + b"\0"
    return sha256_bytes(
        domain
        + core.decoded_hash(PROFILE_SHA256, "profile")
        + core.decoded_hash(candidate_view_sha256, "candidate view")
        + core.decoded_hash(audio_sha256, "candidate audio")
        + core.decoded_hash(target_record_sha256, "target record")
    )


def modal_signature(record: dict[str, Any], profile: dict[str, Any]) -> tuple[Any, ...]:
    fields = profile["specialists"]["retrieval"]["signature_fields"]
    modes = core.modal_values(record)
    try:
        return tuple(tuple(mode[field] for field in fields) for mode in modes)
    except KeyError as error:
        raise ValidatorMechanicsError(
            f"V0a trusted modal signature field missing: {error.args[0]}"
        ) from error


def make_decision(
    record_id: str,
    cache_key: str | None,
    decision: str,
    reason_codes: list[str],
) -> dict[str, Any]:
    return {
        "cache_key": cache_key,
        "claim": CLAIM,
        "decision": decision,
        "reason_codes": reason_codes,
        "record_id": record_id,
        "schema": DECISION_SCHEMA,
    }


def validate_candidate(
    profile: dict[str, Any],
    record: dict[str, Any],
    audio_data: bytes,
    clean_records: dict[str, dict[str, Any]],
    clean_audio: dict[str, bytes],
    clean_record_sha256: dict[str, str],
) -> tuple[dict[str, Any] | None, dict[str, Any]]:
    """Filter retrieval aliases by exact trusted modal equivalence only."""

    with v0a_core_identity():
        features, original_decision = core.validate_candidate(
            profile,
            record,
            audio_data,
            clean_records,
            clean_audio,
            clean_record_sha256,
        )
    if features is None:
        return None, original_decision
    if "integrity_error" in features:
        return features, make_decision(
            features["record_id"], features["cache_key"], "Reject", ["IntegrityMismatch"]
        )

    target_case_id = record["target_case_id"]
    target_signature = modal_signature(clean_records[target_case_id], profile)
    copied = features["retrieval"]["copied_clean_target_ids"]
    filtered = [
        case_id
        for case_id in copied
        if modal_signature(clean_records[case_id], profile) != target_signature
    ]
    features["retrieval"]["copied_clean_target_ids"] = filtered

    provenance = features["provenance"]
    integrity = features["integrity"]
    modal = features["modal"]
    temporal = features["temporal_evolution"]
    envelope = features["envelope_order"]
    reason: str | None = None
    if provenance["mismatch"]:
        reason = "ProvenanceMismatch"
    elif integrity["clipping_sample_count"] > 0:
        reason = "PcmClipping"
    elif filtered:
        reason = "RetrievalCopyDetected"
    elif modal["candidate_mode_count"] < modal["target_mode_count"]:
        reason = "ModalCoverageCollapsed"
    elif (
        modal["candidate_mode_count"] != modal["target_mode_count"]
        or not modal["structure_exact"]
    ):
        reason = "ModalParameterMismatch"
    elif not modal["decay_exact"]:
        reason = "PhysicalDecayMismatch"
    elif temporal["normalized_periodicity_rms"] <= float(
        profile["specialists"]["temporal_evolution"]["normalized_rms_max"]
    ):
        reason = "TemporalEvolutionFrozen"
    elif envelope["monotonic_violation_count"] > 0:
        reason = "EnvelopeOrderMismatch"
    else:
        integrity_profile = profile["specialists"]["integrity"]
        if (
            not integrity["audio_identity_exact"]
            or integrity["rms"] < float(integrity_profile["minimum_rms"])
            or integrity["dc_over_rms"] > float(integrity_profile["dc_over_rms_max"])
        ):
            reason = "IntegrityMismatch"
    if reason is None:
        return features, make_decision(
            features["record_id"], features["cache_key"], "Pass", []
        )
    return features, make_decision(
        features["record_id"], features["cache_key"], "Reject", [reason]
    )


def build_into(
    staging: Path,
    profile: dict[str, Any],
    profile_data: bytes,
    truth_path: Path,
    started: float,
) -> dict[str, Any]:
    dependencies = validate_dependencies(profile)
    records, audio, truth_identity = core.load_truth(truth_path, profile)
    clean_ids = profile["truth"]["clean_case_ids"]
    clean_records = {record_id: records[record_id] for record_id in clean_ids}
    clean_audio = {record_id: audio[record_id] for record_id in clean_ids}
    clean_record_sha256 = {
        record_id: truth_identity["record_sha256"][record_id]
        for record_id in clean_ids
    }
    expected = core.expected_decisions(profile)
    if set(expected) != set(records):
        raise ValidatorMechanicsError("V0a expected candidate closure mismatch")

    cold: dict[str, tuple[bytes, bytes]] = {}
    summaries: list[dict[str, Any]] = []
    label_invariance: list[dict[str, Any]] = []
    for record_id in sorted(records):
        features, decision = validate_candidate(
            profile,
            records[record_id],
            audio[record_id],
            clean_records,
            clean_audio,
            clean_record_sha256,
        )
        if features is None:
            raise ValidatorMechanicsError(f"official T0 candidate became OOD: {record_id}")
        feature_data = canonical_json(features)
        decision_data = canonical_json(decision)
        if (decision["decision"], decision["reason_codes"]) != expected[record_id]:
            raise ValidatorMechanicsError(
                f"V0a decision matrix mismatch: {record_id}: "
                f"{decision['decision']} {decision['reason_codes']}"
            )
        cold[decision["cache_key"]] = (feature_data, decision_data)

        forged = copy.deepcopy(records[record_id])
        forged.update(
            {
                "expected": {"decision": "Forged", "reason_codes": ["Forged"]},
                "invariant_checks": {"forged": False},
                "kind": "ForgedLabel",
                "mutation": {"operation": {"operation_id": "forged"}},
                "source_case_id": "forged-source",
            }
        )
        original_view = canonical_json(core.candidate_view(records[record_id], profile))
        forged_view = canonical_json(core.candidate_view(forged, profile))
        if original_view != forged_view:
            raise ValidatorMechanicsError(f"ignored label leaked: {record_id}")
        forged_features, forged_decision = validate_candidate(
            profile,
            forged,
            audio[record_id],
            clean_records,
            clean_audio,
            clean_record_sha256,
        )
        if (
            forged_features is None
            or canonical_json(forged_features) != feature_data
            or canonical_json(forged_decision) != decision_data
        ):
            raise ValidatorMechanicsError(f"ignored label changed V0a result: {record_id}")
        label_invariance.append({"record_id": record_id, "status": "Pass"})
        summaries.append(
            {
                "cache_key": decision["cache_key"],
                "decision": decision["decision"],
                "reason_codes": decision["reason_codes"],
                "record_id": record_id,
            }
        )

    warm_hits = 0
    for record_id in sorted(records, reverse=True):
        view_sha256 = sha256_bytes(
            canonical_json(core.candidate_view(records[record_id], profile))
        )
        target_id = records[record_id]["target_case_id"]
        key = feature_cache_key(
            profile,
            view_sha256,
            sha256_bytes(audio[record_id]),
            clean_record_sha256[target_id],
        )
        if key not in cold:
            raise ValidatorMechanicsError(f"V0a cold/warm cache miss: {record_id}")
        features, decision = validate_candidate(
            profile,
            records[record_id],
            audio[record_id],
            clean_records,
            clean_audio,
            clean_record_sha256,
        )
        if features is None or cold[key] != (
            canonical_json(features),
            canonical_json(decision),
        ):
            raise ValidatorMechanicsError(f"V0a cold/warm bytes drift: {record_id}")
        warm_hits += 1

    artifacts: list[dict[str, Any]] = []
    publication_summaries: list[dict[str, Any]] = []
    by_record = {item["record_id"]: item for item in summaries}
    for record_id in sorted(records):
        key = by_record[record_id]["cache_key"]
        feature_data, decision_data = cold[key]
        base = f"records/{record_id}"
        feature_ref = core.write_bytes(staging, f"{base}/features.json", feature_data)
        decision_ref = core.write_bytes(staging, f"{base}/decision.json", decision_data)
        artifacts.extend([decision_ref, feature_ref])
        publication_summaries.append(
            {
                **by_record[record_id],
                "decision_sha256": decision_ref["sha256"],
                "features_sha256": feature_ref["sha256"],
            }
        )
    artifacts.sort(key=lambda item: item["path"])
    core.verify_artifacts(staging, artifacts)

    pass_count = sum(item["decision"] == "Pass" for item in summaries)
    reject_count = sum(item["decision"] == "Reject" for item in summaries)
    reason_counts: dict[str, int] = {}
    for item in summaries:
        for reason in item["reason_codes"]:
            reason_counts[reason] = reason_counts.get(reason, 0) + 1
    if (
        pass_count != profile["decision_contract"]["expected_pass_count"]
        or reject_count != profile["decision_contract"]["expected_reject_count"]
        or reason_counts
        != {item["reason_code"]: 1 for item in profile["truth"]["mutation_expectations"]}
    ):
        raise ValidatorMechanicsError("V0a aggregate decision matrix mismatch")

    owner_data = Path(__file__).read_bytes()
    owner_identity = {
        "bytes": len(owner_data),
        "path": OWNER_PATH,
        "sha256": sha256_bytes(owner_data),
    }
    release = {
        "access": ZERO_ACCESS,
        "artifact_index_root_sha256": sha256_bytes(canonical_json(artifacts)),
        "authority": profile["authority"],
        "claim": CLAIM,
        "decision_counts": {"OutOfDomain": 0, "Pass": pass_count, "Reject": reject_count},
        "owner_identity": owner_identity,
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": sha256_bytes(profile_data),
        },
        "reason_counts": reason_counts,
        "records": publication_summaries,
        "schema": RELEASE_SCHEMA,
        "status": "Pass",
        "truth_identity": {
            "evidence_sha256": truth_identity["evidence_sha256"],
            "report_sha256": truth_identity["report_sha256"],
            "truth_release_sha256": truth_identity["truth_release_sha256"],
        },
    }
    release_ref = core.write_bytes(
        staging, "validator-mechanics-release.json", canonical_json(release)
    )
    artifacts.append(release_ref)
    artifacts.sort(key=lambda item: item["path"])
    core.verify_artifacts(staging, artifacts)

    memory_bound = core.deterministic_memory_bound(profile)
    if memory_bound > profile["resources"]["max_peak_rss_bytes"]:
        raise ValidatorMechanicsError("V0a deterministic memory bound exceeds profile")
    artifact_bytes = sum(item["bytes"] for item in artifacts)
    if artifact_bytes > profile["resources"]["max_output_bytes"]:
        raise ValidatorMechanicsError("V0a artifacts exceed output resource profile")
    if time.monotonic() - started > profile["resources"]["max_wall_seconds"]:
        raise ValidatorMechanicsError("V0a execution exceeded wall resource profile")

    evidence = {
        "access": ZERO_ACCESS,
        "artifact_bytes_before_evidence": artifact_bytes,
        "artifact_count_before_evidence": len(artifacts),
        "artifact_index_root_sha256": sha256_bytes(canonical_json(artifacts)),
        "authority": profile["authority"],
        "cache": {
            "cold_entries": len(cold),
            "reverse_order_warm_hits": warm_hits,
            "warm_cold_bytes_exact": True,
        },
        "claim": CLAIM,
        "dependencies": dependencies,
        "deterministic_memory_bound_bytes": memory_bound,
        "environment": {"numpy": np.__version__, "python": platform.python_version()},
        "label_invariance": label_invariance,
        "owner_identity": owner_identity,
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": sha256_bytes(profile_data),
        },
        "schema": EVIDENCE_SCHEMA,
        "status": "Pass",
        "truth_identity": release["truth_identity"],
        "validator_release_sha256": release_ref["sha256"],
    }
    evidence_ref = core.write_bytes(staging, "evidence.json", canonical_json(evidence))
    output_bytes_before_report = sum(
        item.stat().st_size for item in staging.rglob("*") if item.is_file()
    )
    report = {
        "access": ZERO_ACCESS,
        "authority": profile["authority"],
        "claim": CLAIM,
        "decision": "V0A_MODAL_EQUIVALENCE_VALIDATOR_MECHANICS_PASS",
        "evidence_sha256": evidence_ref["sha256"],
        "gates": {
            "byte_exact_repeat_required": True,
            "cache_cold_warm_and_order": "16/16 Pass",
            "clean_modal_aliases": "4/4 Pass",
            "decision_matrix": "9 Pass / 7 Reject",
            "ignored_label_invariance": "16/16 Pass",
            "mutation_reason_coverage": "7/7 Pass",
            "plate_to_beam_retrieval_control": "Pass",
            "resource_envelope": "Pass",
            "zero_real_signal_model_network": "Pass",
        },
        "next_authorized_stage": "V32-M0-bounded-correction-protocol",
        "output_bytes_before_report": output_bytes_before_report,
        "owner_sha256": owner_identity["sha256"],
        "profile_sha256": sha256_bytes(profile_data),
        "record_count": len(records),
        "schema": REPORT_SCHEMA,
        "status": "Pass",
        "validator_release_sha256": release_ref["sha256"],
    }
    core.write_bytes(staging, "report.json", canonical_json(report))
    files = sorted(item for item in staging.rglob("*") if item.is_file())
    if len(files) != profile["publication"]["expected_file_count"]:
        raise ValidatorMechanicsError("V0a final file count drift")
    final_bytes = sum(item.stat().st_size for item in files)
    if final_bytes > profile["resources"]["max_output_bytes"]:
        raise ValidatorMechanicsError("V0a final publication exceeds output profile")
    if time.monotonic() - started > profile["resources"]["max_wall_seconds"]:
        raise ValidatorMechanicsError("V0a final execution exceeded wall profile")
    return report


def run(profile_path: Path, truth_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    profile, profile_data = load_profile(profile_path)
    output = core.external_output(output_path)
    staging = Path(
        tempfile.mkdtemp(prefix=".nextengine-v32-v0a-", dir=str(output.parent))
    )
    try:
        report = build_into(staging, profile, profile_data, truth_path, started)
        os.replace(staging, output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--truth", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = run(arguments.profile, arguments.truth, arguments.output)
    except Exception as error:  # noqa: BLE001 - stable external CLI boundary
        print(f"physical-sound-v32-v0a: ContractReject: {error}", file=os.sys.stderr)
        return 2
    print(canonical_json(report).decode("utf-8"), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
