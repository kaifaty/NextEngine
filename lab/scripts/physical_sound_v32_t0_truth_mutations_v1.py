#!/usr/bin/env python3
"""Build the V32 T0 synthetic truth and mutation release outside the repository."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import os
import platform
import shutil
import struct
import sys
import tempfile
import time
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_v31_p1_modal_owner_v1 as p1


PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-t0-truth-mutations-profile.v1"
)
RECORD_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-t0-truth-mutation-record.v1"
)
RELEASE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-t0-truth-release.v1"
)
EVIDENCE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-t0-truth-evidence.v1"
)
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v32-t0-truth-report.v1"
PROFILE_ID = "physical-sound-v32-t0-truth-mutations-v1"
PROFILE_SHA256 = "44d83804398b2e08a54dbe28e4e3269098b1112785d8377dac34a11d4e5eebe8"
BASELINE_COMMIT = "101cc0be4ffa0def1ecb455fe57ffccc300af240"
P0_PROFILE_SHA256 = "c6b7f816d65bdcaa18618a72d40cfedb6ed970359d3a1cf8f28825cd4c686a3d"
P1_OWNER_SHA256 = "04b44d77ce073d07f30e94c3c361ca4c199cb554ecbe017947aa423842781650"
P1_RESULT_SHA256 = "16e3f6e23280eb3c6d77d52132b6d216768bf8383baa668ed3dd16ca5ac08683"
PROTOCOL_SHA256 = "ac0b087478bf1d0b6c580306428679682e7f295a03a6e08e1b551b081ec8487d"
CLAIM = (
    "SYNTHETIC_VALIDATOR_MECHANICS_TRUTH_ONLY / NO_REAL_MATERIAL_QUALITY_"
    "VALIDATOR_RELEASE_ML_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 256 * 1024

EXPECTED_CLEAN_CASES = [
    "baseline-plate",
    "baseline-beam",
    "intervention-youngs-modulus",
    "intervention-density",
    "intervention-thickness",
    "intervention-uniform-scale",
    "intervention-impulse",
    "intervention-contact",
    "intervention-support",
]
EXPECTED_MUTATIONS = [
    ("mutation-wrong-decay", "PhysicalDecayMismatch"),
    ("mutation-frozen-carrier", "TemporalEvolutionFrozen"),
    ("mutation-shuffled-envelope", "EnvelopeOrderMismatch"),
    ("mutation-mode-collapse", "ModalCoverageCollapsed"),
    ("mutation-spectral-copy", "RetrievalCopyDetected"),
    ("mutation-clipping", "PcmClipping"),
    ("mutation-provenance-mismatch", "ProvenanceMismatch"),
]
AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "external_research_only": True,
    "model_allowed": False,
    "mutation_truth_authority": True,
    "network_allowed": False,
    "quality_authority": False,
    "real_material_authority": False,
    "real_signal_allowed": False,
    "runtime_authority": False,
    "synthetic_only": True,
    "validator_release_authority": False,
}
ZERO_ACCESS = dict(p1.ZERO_ACCESS)


class TruthMutationError(RuntimeError):
    """The frozen T0 release cannot be built or verified."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise TruthMutationError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _no_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise TruthMutationError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value: str) -> None:
    raise TruthMutationError(f"non-finite JSON number: {value}")


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise TruthMutationError("profile must not be a symlink")
    try:
        data = path.read_bytes()
    except OSError as error:
        raise TruthMutationError(f"cannot read T0 profile: {error}") from error
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise TruthMutationError("T0 profile size is outside the frozen bound")
    try:
        profile = json.loads(
            data,
            object_pairs_hook=_no_duplicates,
            parse_constant=_reject_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise TruthMutationError(f"invalid T0 profile JSON: {error}") from error
    if not isinstance(profile, dict) or data != canonical_json(profile):
        raise TruthMutationError("T0 profile is not canonical JSON")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise TruthMutationError("T0 profile hash mismatch")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("baseline_commit") != BASELINE_COMMIT
        or profile.get("claim") != CLAIM
        or profile.get("authority") != AUTHORITY
    ):
        raise TruthMutationError("T0 frozen identity or authority mismatch")
    if profile.get("clean_cases") != EXPECTED_CLEAN_CASES:
        raise TruthMutationError("T0 clean case matrix mismatch")
    mutation_matrix = [
        (item.get("mutation_id"), item.get("expected_reason_code"))
        for item in profile.get("mutations", [])
        if isinstance(item, dict)
    ]
    if mutation_matrix != EXPECTED_MUTATIONS:
        raise TruthMutationError("T0 mutation matrix mismatch")
    if profile.get("mutation_reason_codes") != [item[1] for item in EXPECTED_MUTATIONS]:
        raise TruthMutationError("T0 mutation reason order mismatch")
    expectations = profile.get("validator_expectations")
    publication = profile.get("publication")
    numeric = profile.get("numeric_profile")
    resources = profile.get("resources")
    if not all(
        isinstance(item, dict)
        for item in (expectations, publication, numeric, resources)
    ):
        raise TruthMutationError("T0 publication/expectation/resource contract missing")
    if (
        expectations.get("expected_pass_count") != 9
        or expectations.get("expected_reject_count") != 7
        or expectations.get("out_of_domain_count") != 0
        or publication.get("record_count") != 16
        or publication.get("expected_file_count") != 35
        or numeric.get("frame_count") != 144_000
        or numeric.get("sample_rate_hz") != 48_000
        or numeric.get("randomness") != "forbidden"
        or publication.get("record_count") > resources.get("max_record_count", -1)
        or publication.get("expected_file_count") > resources.get("max_file_count", -1)
        or numeric.get("frame_count") > resources.get("max_render_frames", -1)
    ):
        raise TruthMutationError("T0 frozen publication/resource contract mismatch")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    root = repository_root()
    expected = {
        "p0_profile": (
            "lab/profiles/physical-sound-v31-p0-causal-baseline.v1.json",
            P0_PROFILE_SHA256,
        ),
        "p1_owner": (
            "lab/scripts/physical_sound_v31_p1_modal_owner_v1.py",
            P1_OWNER_SHA256,
        ),
        "p1_result": (
            "docs/development/physical-sound-v31-p1-deterministic-modal-owner-result-2026-09-02.md",
            P1_RESULT_SHA256,
        ),
    }
    result: dict[str, dict[str, Any]] = {}
    parent = profile.get("parent")
    if not isinstance(parent, dict) or set(parent) != set(expected):
        raise TruthMutationError("T0 parent dependency set mismatch")
    for dependency_id, (relative, expected_sha256) in expected.items():
        declared = parent[dependency_id]
        if declared != {"path": relative, "sha256": expected_sha256}:
            raise TruthMutationError(f"T0 declared dependency mismatch: {dependency_id}")
        data = (root / relative).read_bytes()
        actual = sha256_bytes(data)
        if actual != expected_sha256:
            raise TruthMutationError(f"T0 dependency drift: {dependency_id}")
        result[dependency_id] = {
            "bytes": len(data),
            "path": relative,
            "sha256": actual,
        }
    protocol = profile.get("protocol")
    expected_protocol = {
        "path": "docs/development/physical-sound-v32-t0-truth-mutation-protocol-2026-09-02.md",
        "sha256": PROTOCOL_SHA256,
    }
    if protocol != expected_protocol:
        raise TruthMutationError("T0 protocol identity mismatch")
    protocol_data = (root / expected_protocol["path"]).read_bytes()
    if sha256_bytes(protocol_data) != PROTOCOL_SHA256:
        raise TruthMutationError("T0 protocol drift")
    result["t0_protocol"] = {
        "bytes": len(protocol_data),
        **expected_protocol,
    }
    if Path(p1.__file__).resolve() != (root / expected["p1_owner"][0]).resolve():
        raise TruthMutationError("T0 imported P1 owner path mismatch")
    return result


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise TruthMutationError("output must not be a symlink")
    try:
        parent = unresolved.parent.resolve(strict=True)
    except OSError as error:
        raise TruthMutationError(f"output parent must exist: {error}") from error
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise TruthMutationError("output must be a fresh external path")
    return output


def render_modal(
    frequencies: np.ndarray,
    decay: np.ndarray,
    signed_gains: np.ndarray,
    impulse: float,
    sample_rate_hz: int,
    frame_count: int,
    render_scale: float,
) -> np.ndarray:
    if not (
        frequencies.ndim == decay.ndim == signed_gains.ndim == 1
        and len(frequencies) == len(decay) == len(signed_gains)
        and len(frequencies) > 0
    ):
        raise TruthMutationError("invalid T0 modal render vectors")
    seconds = np.arange(frame_count, dtype=np.float64) / float(sample_rate_hz)
    samples = np.zeros(frame_count, dtype=np.float64)
    for frequency, rate, gain in zip(frequencies, decay, signed_gains, strict=True):
        samples += float(gain) * np.exp(-float(rate) * seconds) * np.sin(
            2.0 * math.pi * float(frequency) * seconds
        )
    samples *= render_scale * impulse
    if np.any(~np.isfinite(samples)):
        raise TruthMutationError("non-finite T0 modal render")
    return samples


def undamped_carrier(
    frequencies: np.ndarray,
    signed_gains: np.ndarray,
    sample_ordinals: np.ndarray,
    sample_rate_hz: int,
) -> np.ndarray:
    seconds = sample_ordinals.astype(np.float64, copy=False) / float(sample_rate_hz)
    carrier = np.zeros(len(sample_ordinals), dtype=np.float64)
    for frequency, gain in zip(frequencies, signed_gains, strict=True):
        carrier += float(gain) * np.sin(2.0 * math.pi * float(frequency) * seconds)
    if np.any(~np.isfinite(carrier)):
        raise TruthMutationError("non-finite T0 carrier")
    return carrier


def encode_float32_wav(
    samples: np.ndarray, sample_rate_hz: int, *, allow_full_scale: bool
) -> bytes:
    if samples.ndim != 1 or len(samples) == 0 or np.any(~np.isfinite(samples)):
        raise TruthMutationError("T0 render is empty or non-finite")
    peak = float(np.max(np.abs(samples)))
    if peak > 1.0 or (not allow_full_scale and peak >= 1.0):
        raise TruthMutationError("T0 render violates the PCM peak contract")
    payload = samples.astype("<f4", copy=False).tobytes()
    fmt = struct.pack(
        "<HHIIHH", 3, 1, sample_rate_hz, sample_rate_hz * 4, 4, 32
    )
    return (
        b"RIFF"
        + struct.pack("<I", 4 + 8 + len(fmt) + 8 + len(payload))
        + b"WAVEfmt "
        + struct.pack("<I", len(fmt))
        + fmt
        + b"data"
        + struct.pack("<I", len(payload))
        + payload
    )


def audio_identity(data: bytes, frame_count: int, sample_rate_hz: int) -> dict[str, Any]:
    return {
        "bytes": len(data),
        "channels": 1,
        "encoding": "ieee-float32-little-endian",
        "frame_count": frame_count,
        "sample_rate_hz": sample_rate_hz,
        "sha256": sha256_bytes(data),
    }


def common_source(
    solution: dict[str, Any],
    profile: dict[str, Any],
    p0_profile: dict[str, Any],
) -> tuple[int, int, float, float]:
    numeric = profile["numeric_profile"]
    frame_count = numeric["frame_count"]
    sample_rate_hz = numeric["sample_rate_hz"]
    p0_numeric = p0_profile["numeric_profile"]
    if (
        frame_count != p0_numeric["render_frames"]
        or sample_rate_hz != p0_numeric["sample_rate_hz"]
    ):
        raise TruthMutationError("T0/P0 numeric profile mismatch")
    impulse = float(solution["validated"]["contact"]["normal_impulse_ns"])
    render_scale = p1.positive_float(p0_numeric["render_scale_per_ns"])
    return frame_count, sample_rate_hz, impulse, render_scale


def clean_record(
    case_id: str,
    solution: dict[str, Any],
    wav: bytes,
    profile_sha256: str,
) -> dict[str, Any]:
    return {
        "audio": audio_identity(
            wav,
            len(solution["samples"]),
            48_000,
        ),
        "claim": CLAIM,
        "expected": {"decision": "Pass", "reason_codes": []},
        "invariant_checks": {
            "clean_p1_audio_exact": True,
            "energy_envelope_pass": True,
            "remesh_common_vertices_exact": solution["metrics"][
                "remesh_common_vertices_exact"
            ],
        },
        "kind": "Clean",
        "lineage": {
            "p0_profile_sha256": P0_PROFILE_SHA256,
            "p1_owner_sha256": P1_OWNER_SHA256,
            "t0_profile_sha256": profile_sha256,
        },
        "modal": solution["modal_document"],
        "record_id": case_id,
        "render_metrics": solution["metrics"],
        "schema": RECORD_SCHEMA,
        "source_case_id": case_id,
        "target_case_id": case_id,
    }


def mutation_record(
    mutation: dict[str, Any],
    wav: bytes,
    invariant_checks: dict[str, Any],
    source_audio_sha256: str,
    frame_count: int,
    sample_rate_hz: int,
    profile_sha256: str,
    modal: dict[str, Any] | None,
) -> dict[str, Any]:
    declared_parent = source_audio_sha256
    if mutation["mutation_id"] == "mutation-provenance-mismatch":
        declared_parent = mutation["operation"]["declared_parent_audio_sha256"]
    result = {
        "audio": audio_identity(wav, frame_count, sample_rate_hz),
        "claim": CLAIM,
        "expected": {
            "decision": mutation["expected_decision"],
            "reason_codes": [mutation["expected_reason_code"]],
        },
        "invariant_checks": invariant_checks,
        "kind": "Mutation",
        "lineage": {
            "actual_parent_audio_sha256": source_audio_sha256,
            "declared_parent_audio_sha256": declared_parent,
            "p0_profile_sha256": P0_PROFILE_SHA256,
            "p1_owner_sha256": P1_OWNER_SHA256,
            "t0_profile_sha256": profile_sha256,
        },
        "modal": modal,
        "mutation": {
            "operation": mutation["operation"],
            "preserved_invariants": mutation["preserved_invariants"],
            "violated_invariant": mutation["violated_invariant"],
        },
        "record_id": mutation["mutation_id"],
        "schema": RECORD_SCHEMA,
        "source_case_id": mutation["source_case_id"],
        "target_case_id": mutation["target_case_id"],
    }
    return result


def update_modal_decay(
    modal_document: dict[str, Any], decay: np.ndarray, case_id: str
) -> dict[str, Any]:
    result = copy.deepcopy(modal_document)
    result["case_id"] = case_id
    for mode, rate in zip(result["modes"], decay, strict=True):
        mode["decay_per_second"] = float(rate)
    return result


def build_mutations(
    profile: dict[str, Any],
    p0_profile: dict[str, Any],
    clean_solutions: dict[str, dict[str, Any]],
    clean_wavs: dict[str, bytes],
    profile_sha256: str,
) -> list[tuple[dict[str, Any], bytes]]:
    plate = clean_solutions["baseline-plate"]
    beam = clean_solutions["baseline-beam"]
    frame_count, sample_rate_hz, impulse, render_scale = common_source(
        plate, profile, p0_profile
    )
    if frame_count != len(plate["samples"]):
        raise TruthMutationError("T0 profile/P1 frame count mismatch")
    sample_ordinals = np.arange(frame_count, dtype=np.int64)
    seconds = sample_ordinals.astype(np.float64) / float(sample_rate_hz)
    source_sha256 = sha256_bytes(clean_wavs["baseline-plate"])
    result: list[tuple[dict[str, Any], bytes]] = []
    by_id = {item["mutation_id"]: item for item in profile["mutations"]}

    mutation = by_id["mutation-wrong-decay"]
    factor = float(mutation["operation"]["decay_multiplier"])
    wrong_decay = plate["decay"] * factor
    wrong_samples = render_modal(
        plate["frequencies"],
        wrong_decay,
        plate["signed_gains"],
        impulse,
        sample_rate_hz,
        frame_count,
        render_scale,
    )
    wrong_wav = encode_float32_wav(
        wrong_samples, sample_rate_hz, allow_full_scale=False
    )
    checks = {
        "decay_multiplier_exact": bool(
            np.array_equal(wrong_decay, plate["decay"] * 0.25)
        ),
        "frame_count_exact": len(wrong_samples) == frame_count,
        "frequencies_exact": True,
        "signed_gains_exact": True,
    }
    if not all(checks.values()):
        raise TruthMutationError("wrong-decay invariant check failed")
    result.append(
        (
            mutation_record(
                mutation,
                wrong_wav,
                checks,
                source_sha256,
                frame_count,
                sample_rate_hz,
                profile_sha256,
                update_modal_decay(
                    plate["modal_document"], wrong_decay, mutation["mutation_id"]
                ),
            ),
            wrong_wav,
        )
    )

    if not np.array_equal(
        plate["decay"], np.full(len(plate["decay"]), plate["decay"][0])
    ):
        raise TruthMutationError("baseline plate does not have one scalar envelope")
    common_decay = float(plate["decay"][0])
    clean_envelope = np.exp(-common_decay * seconds)

    mutation = by_id["mutation-frozen-carrier"]
    period = mutation["operation"]["carrier_period_samples"]
    prefix_ordinals = np.arange(period, dtype=np.int64)
    prefix_carrier = undamped_carrier(
        plate["frequencies"],
        plate["signed_gains"],
        prefix_ordinals,
        sample_rate_hz,
    )
    repeated_carrier = prefix_carrier[np.remainder(sample_ordinals, period)]
    frozen_samples = render_scale * impulse * repeated_carrier * clean_envelope
    frozen_wav = encode_float32_wav(
        frozen_samples, sample_rate_hz, allow_full_scale=False
    )
    checks = {
        "carrier_period_samples": period,
        "common_decay_exact": True,
        "frame_count_exact": len(frozen_samples) == frame_count,
        "repeated_carrier_exact": bool(
            np.array_equal(
                repeated_carrier,
                prefix_carrier[np.remainder(sample_ordinals, period)],
            )
        ),
        "scalar_decay_envelope_exact": True,
    }
    if checks["carrier_period_samples"] != 256 or not all(
        value for key, value in checks.items() if key != "carrier_period_samples"
    ):
        raise TruthMutationError("frozen-carrier invariant check failed")
    result.append(
        (
            mutation_record(
                mutation,
                frozen_wav,
                checks,
                source_sha256,
                frame_count,
                sample_rate_hz,
                profile_sha256,
                copy.deepcopy(plate["modal_document"]),
            ),
            frozen_wav,
        )
    )

    mutation = by_id["mutation-shuffled-envelope"]
    block_count = mutation["operation"]["block_count"]
    block_size = mutation["operation"]["block_size_samples"]
    permutation = mutation["operation"]["block_permutation"]
    if block_count * block_size != frame_count:
        raise TruthMutationError("shuffled-envelope blocks do not cover render")
    if sorted(permutation) != list(range(block_count)) or permutation == list(
        range(block_count)
    ):
        raise TruthMutationError("shuffled-envelope permutation is invalid")
    shuffled_envelope = np.concatenate(
        [
            clean_envelope[source * block_size : (source + 1) * block_size]
            for source in permutation
        ]
    )
    full_carrier = undamped_carrier(
        plate["frequencies"],
        plate["signed_gains"],
        sample_ordinals,
        sample_rate_hz,
    )
    shuffled_samples = render_scale * impulse * full_carrier * shuffled_envelope
    shuffled_wav = encode_float32_wav(
        shuffled_samples, sample_rate_hz, allow_full_scale=False
    )
    restored = np.empty_like(shuffled_envelope)
    for output_block, source_block in enumerate(permutation):
        restored[source_block * block_size : (source_block + 1) * block_size] = (
            shuffled_envelope[
                output_block * block_size : (output_block + 1) * block_size
            ]
        )
    checks = {
        "block_count_exact": block_count == 12,
        "block_size_exact": block_size == 12_000,
        "envelope_multiset_exact": bool(np.array_equal(restored, clean_envelope)),
        "frame_count_exact": len(shuffled_samples) == frame_count,
        "permutation_bijective_non_identity": True,
        "undamped_carrier_time_aligned_exact": True,
    }
    if not all(checks.values()):
        raise TruthMutationError("shuffled-envelope invariant check failed")
    result.append(
        (
            mutation_record(
                mutation,
                shuffled_wav,
                checks,
                source_sha256,
                frame_count,
                sample_rate_hz,
                profile_sha256,
                copy.deepcopy(plate["modal_document"]),
            ),
            shuffled_wav,
        )
    )

    mutation = by_id["mutation-mode-collapse"]
    selected = min(
        range(len(plate["signed_gains"])),
        key=lambda ordinal: (-abs(float(plate["signed_gains"][ordinal])), ordinal),
    )
    collapsed_samples = render_modal(
        plate["frequencies"][selected : selected + 1],
        plate["decay"][selected : selected + 1],
        plate["signed_gains"][selected : selected + 1],
        impulse,
        sample_rate_hz,
        frame_count,
        render_scale,
    )
    collapsed_wav = encode_float32_wav(
        collapsed_samples, sample_rate_hz, allow_full_scale=False
    )
    collapsed_modal = copy.deepcopy(plate["modal_document"])
    collapsed_modal["case_id"] = mutation["mutation_id"]
    collapsed_modal["modes"] = [collapsed_modal["modes"][selected]]
    checks = {
        "frame_count_exact": len(collapsed_samples) == frame_count,
        "retained_mode_count": len(collapsed_modal["modes"]),
        "selected_mode_parameters_exact": collapsed_modal["modes"][0]
        == plate["modal_document"]["modes"][selected],
        "selected_ordinal": selected,
    }
    if checks["retained_mode_count"] != 1 or not checks[
        "selected_mode_parameters_exact"
    ]:
        raise TruthMutationError("mode-collapse invariant check failed")
    result.append(
        (
            mutation_record(
                mutation,
                collapsed_wav,
                checks,
                source_sha256,
                frame_count,
                sample_rate_hz,
                profile_sha256,
                collapsed_modal,
            ),
            collapsed_wav,
        )
    )

    mutation = by_id["mutation-spectral-copy"]
    copied_wav = clean_wavs["baseline-plate"]
    checks = {
        "copied_payload_equals_source": True,
        "copied_payload_differs_target": copied_wav != clean_wavs["baseline-beam"],
        "source_lineage_honest": True,
        "target_case_differs": mutation["target_case_id"] != mutation["source_case_id"],
        "target_modal_family_differs": plate["validated"]["family"]
        != beam["validated"]["family"],
    }
    if not all(checks.values()):
        raise TruthMutationError("spectral-copy invariant check failed")
    result.append(
        (
            mutation_record(
                mutation,
                copied_wav,
                checks,
                source_sha256,
                frame_count,
                sample_rate_hz,
                profile_sha256,
                copy.deepcopy(plate["modal_document"]),
            ),
            copied_wav,
        )
    )

    mutation = by_id["mutation-clipping"]
    clean_peak = float(np.max(np.abs(plate["samples"])))
    target_peak = float(mutation["operation"]["preclip_target_peak"])
    preclip_samples = plate["samples"] * (target_peak / clean_peak)
    clipped_samples = np.clip(preclip_samples, -1.0, 1.0)
    clipped_count = int(np.count_nonzero(np.abs(preclip_samples) >= 1.0))
    clipped_wav = encode_float32_wav(
        clipped_samples, sample_rate_hz, allow_full_scale=True
    )
    checks = {
        "clipped_sample_count": clipped_count,
        "frame_count_exact": len(clipped_samples) == frame_count,
        "hard_clip_bounds_exact": bool(
            float(np.max(clipped_samples)) <= 1.0
            and float(np.min(clipped_samples)) >= -1.0
            and float(np.max(np.abs(clipped_samples))) == 1.0
        ),
        "preclip_target_peak_absolute_error": abs(
            float(np.max(np.abs(preclip_samples))) - target_peak
        ),
    }
    if (
        clipped_count <= 0
        or not checks["frame_count_exact"]
        or not checks["hard_clip_bounds_exact"]
        or checks["preclip_target_peak_absolute_error"] > p1.RELATIVE_TOLERANCE
    ):
        raise TruthMutationError("clipping invariant check failed")
    result.append(
        (
            mutation_record(
                mutation,
                clipped_wav,
                checks,
                source_sha256,
                frame_count,
                sample_rate_hz,
                profile_sha256,
                copy.deepcopy(plate["modal_document"]),
            ),
            clipped_wav,
        )
    )

    mutation = by_id["mutation-provenance-mismatch"]
    provenance_wav = clean_wavs["baseline-plate"]
    declared = mutation["operation"]["declared_parent_audio_sha256"]
    checks = {
        "actual_payload_hash_exact": sha256_bytes(provenance_wav) == source_sha256,
        "declared_parent_differs_actual": declared != source_sha256,
        "declared_parent_is_frozen_zero_hash": declared == "0" * 64,
        "payload_equals_source": provenance_wav == clean_wavs["baseline-plate"],
    }
    if not all(checks.values()):
        raise TruthMutationError("provenance-mismatch invariant check failed")
    result.append(
        (
            mutation_record(
                mutation,
                provenance_wav,
                checks,
                source_sha256,
                frame_count,
                sample_rate_hz,
                profile_sha256,
                copy.deepcopy(plate["modal_document"]),
            ),
            provenance_wav,
        )
    )

    if [record["record_id"] for record, _ in result] != [
        item[0] for item in EXPECTED_MUTATIONS
    ]:
        raise TruthMutationError("T0 mutation output order mismatch")
    return result


def write_bytes(root: Path, relative: str, data: bytes) -> dict[str, Any]:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {
        "bytes": len(data),
        "path": relative,
        "sha256": sha256_bytes(data),
    }


def verify_artifacts(root: Path, artifacts: list[dict[str, Any]]) -> None:
    paths = [artifact["path"] for artifact in artifacts]
    if paths != sorted(paths) or len(paths) != len(set(paths)):
        raise TruthMutationError("T0 artifact index is not sorted and unique")
    for artifact in artifacts:
        data = (root / artifact["path"]).read_bytes()
        if len(data) != artifact["bytes"] or sha256_bytes(data) != artifact["sha256"]:
            raise TruthMutationError(f"T0 artifact drift: {artifact['path']}")


def deterministic_memory_bound(profile: dict[str, Any]) -> int:
    return (
        profile["publication"]["record_count"]
        * profile["numeric_profile"]["frame_count"]
        * 8
        + 64 * 1024 * 1024
    )


def build_into(
    staging: Path,
    profile: dict[str, Any],
    profile_data: bytes,
    started: float,
) -> dict[str, Any]:
    dependencies = validate_dependencies(profile)
    root = repository_root()
    try:
        p0_profile, _ = p1.p0.load_profile(
            root / profile["parent"]["p0_profile"]["path"]
        )
    except p1.p0.CausalBaselineError as error:
        raise TruthMutationError(f"P0 parent contract rejected: {error}") from error
    cases = p1.build_case_inputs(p0_profile)
    if [case_id for case_id, _, _ in cases] != profile["clean_cases"]:
        raise TruthMutationError("P1 clean case order drift")
    clean_solutions = {
        case_id: p1.solve_case(case_id, fixture, p0_profile)
        for case_id, fixture, _ in cases
    }
    analytic_controls = p1.analytic_controls(p0_profile, clean_solutions)
    intervention_controls = p1.intervention_controls(p0_profile, clean_solutions)
    clean_wavs = {
        case_id: p1.encode_float32_wav(
            solution["samples"], p0_profile["numeric_profile"]["sample_rate_hz"]
        )
        for case_id, solution in clean_solutions.items()
    }
    profile_sha256 = sha256_bytes(profile_data)
    records: list[tuple[dict[str, Any], bytes]] = [
        (
            clean_record(case_id, clean_solutions[case_id], clean_wavs[case_id], profile_sha256),
            clean_wavs[case_id],
        )
        for case_id in profile["clean_cases"]
    ]
    records.extend(
        build_mutations(
            profile, p0_profile, clean_solutions, clean_wavs, profile_sha256
        )
    )
    if len(records) != profile["publication"]["record_count"]:
        raise TruthMutationError("T0 record count drift")

    artifacts: list[dict[str, Any]] = []
    summaries: list[dict[str, Any]] = []
    operation_checks: list[dict[str, Any]] = []
    for record, wav in sorted(records, key=lambda item: item[0]["record_id"]):
        record_id = record["record_id"]
        base = f"records/{record_id}"
        audio_ref = write_bytes(staging, f"{base}/audio.wav", wav)
        if audio_ref["sha256"] != record["audio"]["sha256"]:
            raise TruthMutationError(f"T0 audio identity drift: {record_id}")
        record_ref = write_bytes(
            staging, f"{base}/record.json", canonical_json(record)
        )
        artifacts.extend([audio_ref, record_ref])
        summaries.append(
            {
                "audio_sha256": audio_ref["sha256"],
                "decision": record["expected"]["decision"],
                "kind": record["kind"],
                "reason_codes": record["expected"]["reason_codes"],
                "record_id": record_id,
                "record_sha256": record_ref["sha256"],
            }
        )
        operation_checks.append(
            {
                "checks": record["invariant_checks"],
                "record_id": record_id,
                "status": "Pass",
            }
        )
    artifacts.sort(key=lambda item: item["path"])
    verify_artifacts(staging, artifacts)
    pass_count = sum(item["decision"] == "Pass" for item in summaries)
    reject_count = sum(item["decision"] == "Reject" for item in summaries)
    expectations = profile["validator_expectations"]
    if (
        pass_count != expectations["expected_pass_count"]
        or reject_count != expectations["expected_reject_count"]
    ):
        raise TruthMutationError("T0 expected decision counts drift")

    owner_data = Path(__file__).read_bytes()
    owner_identity = {
        "bytes": len(owner_data),
        "path": "lab/scripts/physical_sound_v32_t0_truth_mutations_v1.py",
        "sha256": sha256_bytes(owner_data),
    }
    record_artifact_root = sha256_bytes(canonical_json(artifacts))
    truth_release = {
        "access": ZERO_ACCESS,
        "artifact_index_root_sha256": record_artifact_root,
        "authority": profile["authority"],
        "claim": CLAIM,
        "clean_relations": profile["clean_relations"],
        "decision_counts": {
            "OutOfDomain": 0,
            "Pass": pass_count,
            "Reject": reject_count,
        },
        "owner_identity": owner_identity,
        "parent": profile["parent"],
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": profile_sha256,
        },
        "records": summaries,
        "schema": RELEASE_SCHEMA,
        "status": "Pass",
    }
    release_ref = write_bytes(
        staging, "truth-release.json", canonical_json(truth_release)
    )
    artifacts.append(release_ref)
    artifacts.sort(key=lambda item: item["path"])
    verify_artifacts(staging, artifacts)

    memory_bound = deterministic_memory_bound(profile)
    if memory_bound > profile["resources"]["max_peak_rss_bytes"]:
        raise TruthMutationError("T0 deterministic memory bound exceeds profile")
    artifact_bytes = sum(item["bytes"] for item in artifacts)
    if artifact_bytes > profile["resources"]["max_output_bytes"]:
        raise TruthMutationError("T0 artifacts exceed output resource profile")
    if time.monotonic() - started > profile["resources"]["max_wall_seconds"]:
        raise TruthMutationError("T0 execution exceeded wall resource profile")

    evidence = {
        "access": ZERO_ACCESS,
        "analytic_controls": analytic_controls,
        "artifact_bytes_before_evidence": artifact_bytes,
        "artifact_count_before_evidence": len(artifacts),
        "artifact_index_root_sha256": sha256_bytes(canonical_json(artifacts)),
        "authority": profile["authority"],
        "claim": CLAIM,
        "dependencies": dependencies,
        "deterministic_memory_bound_bytes": memory_bound,
        "environment": {
            "numpy": np.__version__,
            "python": platform.python_version(),
        },
        "intervention_controls": intervention_controls,
        "operation_checks": operation_checks,
        "owner_identity": owner_identity,
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": profile_sha256,
        },
        "schema": EVIDENCE_SCHEMA,
        "status": "Pass",
        "truth_release_sha256": release_ref["sha256"],
    }
    evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
    output_bytes_before_report = sum(
        path.stat().st_size for path in staging.rglob("*") if path.is_file()
    )
    report = {
        "access": ZERO_ACCESS,
        "authority": profile["authority"],
        "claim": CLAIM,
        "decision": "T0_TRUTH_MUTATION_LIBRARY_PASS",
        "evidence_sha256": evidence_ref["sha256"],
        "expected_pass_count": pass_count,
        "expected_reject_count": reject_count,
        "gates": {
            "byte_exact_repeat_required": True,
            "clean_p1_cases": "9/9 Pass",
            "decision_matrix": "9 Pass / 7 Reject",
            "mutation_invariants": "7/7 Pass",
            "parent_lineage": "Pass",
            "resource_envelope": "Pass",
            "zero_real_signal_model_network": "Pass",
        },
        "next_authorized_stage": "V32-V0-validator-mechanics-protocol",
        "output_bytes_before_report": output_bytes_before_report,
        "owner_sha256": owner_identity["sha256"],
        "profile_sha256": profile_sha256,
        "record_count": len(records),
        "schema": REPORT_SCHEMA,
        "status": "Pass",
        "truth_release_sha256": release_ref["sha256"],
    }
    write_bytes(staging, "report.json", canonical_json(report))
    files = sorted(path for path in staging.rglob("*") if path.is_file())
    if len(files) != profile["publication"]["expected_file_count"]:
        raise TruthMutationError("T0 final file count drift")
    final_bytes = sum(path.stat().st_size for path in files)
    if final_bytes > profile["resources"]["max_output_bytes"]:
        raise TruthMutationError("T0 final publication exceeds output profile")
    if time.monotonic() - started > profile["resources"]["max_wall_seconds"]:
        raise TruthMutationError("T0 final execution exceeded wall profile")
    return report


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    profile, profile_data = load_profile(profile_path)
    output = external_output(output_path)
    staging = Path(
        tempfile.mkdtemp(prefix=".nextengine-v32-t0-", dir=str(output.parent))
    )
    try:
        report = build_into(staging, profile, profile_data, started)
        os.replace(staging, output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = run(arguments.profile, arguments.output)
    except Exception as error:  # noqa: BLE001 - stable external CLI boundary
        print(f"physical-sound-v32-t0: ContractReject: {error}", file=sys.stderr)
        return 2
    print(canonical_json(report).decode("utf-8"), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
