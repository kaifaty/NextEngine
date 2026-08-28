#!/usr/bin/env python3
"""Audit the frozen Pitcher result against the pre-existing V2 observation gate."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
from typing import Any


PITCHER_REPORT_SHA256 = (
    "8bd5323cabdb4319f8465631c3c7a54438669ee10afa9cac1f6bb122537b1aea"
)
PITCHER_REPORT_SCHEMA = "nextengine.experimental-realimpact-pitcher-calibration.report.v1"
REPORT_SCHEMA = "nextengine.experimental-realimpact-observation-admission-audit.report.v1"
TRANSFER_CALIBRATION_PATH = Path(
    "tools/xtask/src/physical_sound_registry_command/transfer_calibration.rs"
)
TRANSFER_CALIBRATION_SHA256 = (
    "2624656ebefbdf20d210ddef3138824219ab8044435dc5440365908a720cbf80"
)
TRANSFER_DSP_PATH = Path(
    "tools/xtask/src/physical_sound_registry_command/transfer_calibration/dsp.rs"
)
TRANSFER_DSP_SHA256 = (
    "131bbf42d01e633b6cac0c4e0340178f3f5a0a83324bb64630d8423da8179ca4"
)

# These are the already-published REALIMPACT transfer V2 holdout thresholds.
# This audit does not select or tune them from Pitcher.
MINIMUM_SELECTED_MODES = 6
MINIMUM_PERSISTENT_MODE_RECALL = 0.50
MAXIMUM_MEDIAN_FREQUENCY_ERROR_CENTS = 40.0
MINIMUM_DECAYING_MODE_FRACTION = 0.50
MAXIMUM_MEDIAN_TAIL_PREDICTION_RMSE_DB = 24.0

# The REALIMPACT paper reports RT60 below 0.2 s above 500 Hz and explicitly
# identifies the lower band as the less-anechoic part of the room.
ROOM_CAVEAT_UPPER_HZ = 500.0


class AuditError(RuntimeError):
    """A hash, schema, bound or output invariant was violated."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pitcher-report", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def external_file(root: Path, path: Path) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise AuditError(f"Pitcher report must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root):
        raise AuditError(f"audit output must remain outside the repository: {resolved}")
    if resolved.exists() and (not resolved.is_dir() or any(resolved.iterdir())):
        raise AuditError(f"audit output must be absent or empty: {resolved}")
    return resolved


def finite_number(value: Any, name: str) -> float:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise AuditError(f"{name} is not numeric")
    result = float(value)
    if not (-float("inf") < result < float("inf")):
        raise AuditError(f"{name} is not finite")
    return result


def load_pitcher_report(path: Path) -> tuple[bytes, dict[str, Any]]:
    report_bytes = path.read_bytes()
    actual = sha256_bytes(report_bytes)
    if actual != PITCHER_REPORT_SHA256:
        raise AuditError(
            f"Pitcher report identity changed: expected {PITCHER_REPORT_SHA256}, got {actual}"
        )
    try:
        report = json.loads(report_bytes)
    except json.JSONDecodeError as error:
        raise AuditError(f"parse Pitcher report: {error}") from error
    if (
        report.get("schema") != PITCHER_REPORT_SCHEMA
        or report.get("decision") != "PitcherGeometrySpatialCalibrationRejected"
        or report.get("network_requests") != 0
        or report.get("additional_reserved_audio_payload_bytes_read") != 0
        or report.get("planter_audio_payload_bytes_read") != 0
        or report.get("gate", {}).get("passed") is not False
    ):
        raise AuditError("Pitcher report contract changed")
    return report_bytes, report


def validate_bound_sources(root: Path) -> dict[str, str]:
    sources = {
        str(TRANSFER_CALIBRATION_PATH): TRANSFER_CALIBRATION_SHA256,
        str(TRANSFER_DSP_PATH): TRANSFER_DSP_SHA256,
    }
    for relative, expected in sources.items():
        path = root / relative
        actual = sha256_bytes(path.read_bytes())
        if actual != expected:
            raise AuditError(
                f"bound observation-gate source changed: {relative}: "
                f"expected {expected}, got {actual}"
            )
    return sources


def check(name: str, observed: float | int, relation: str, threshold: float | int) -> dict[str, Any]:
    if relation == ">=":
        passed = observed >= threshold
    elif relation == "<=":
        passed = observed <= threshold
    else:
        raise AuditError(f"unsupported gate relation: {relation}")
    return {
        "name": name,
        "observed": observed,
        "relation": relation,
        "threshold": threshold,
        "passed": passed,
    }


def audit(report: dict[str, Any], sources: dict[str, str]) -> dict[str, Any]:
    extraction = report.get("extractor")
    if not isinstance(extraction, dict) or not isinstance(extraction.get("modes"), list):
        raise AuditError("Pitcher extractor report is absent")
    selected = extraction.get("selected_mode_count")
    persistent = finite_number(
        extraction.get("persistent_mode_recall"), "persistent_mode_recall"
    )
    frequency_error = finite_number(
        extraction.get("median_frequency_error_cents"),
        "median_frequency_error_cents",
    )
    decaying = finite_number(
        extraction.get("decaying_mode_fraction"), "decaying_mode_fraction"
    )
    tail_rmse = finite_number(
        extraction.get("median_tail_prediction_rmse_db"),
        "median_tail_prediction_rmse_db",
    )
    if not isinstance(selected, int) or isinstance(selected, bool) or selected <= 0:
        raise AuditError("selected_mode_count is invalid")
    modes = extraction["modes"]
    if len(modes) != selected:
        raise AuditError("Pitcher mode list length changed")
    frequencies = [
        finite_number(mode.get("frequency_hz"), f"modes[{index}].frequency_hz")
        for index, mode in enumerate(modes)
        if isinstance(mode, dict)
    ]
    if len(frequencies) != selected:
        raise AuditError("Pitcher mode record shape changed")

    checks = [
        check("selected_mode_count", selected, ">=", MINIMUM_SELECTED_MODES),
        check(
            "persistent_mode_recall",
            persistent,
            ">=",
            MINIMUM_PERSISTENT_MODE_RECALL,
        ),
        check(
            "median_frequency_error_cents",
            frequency_error,
            "<=",
            MAXIMUM_MEDIAN_FREQUENCY_ERROR_CENTS,
        ),
        check(
            "decaying_mode_fraction",
            decaying,
            ">=",
            MINIMUM_DECAYING_MODE_FRACTION,
        ),
        check(
            "median_tail_prediction_rmse_db",
            tail_rmse,
            "<=",
            MAXIMUM_MEDIAN_TAIL_PREDICTION_RMSE_DB,
        ),
    ]
    observation_passed = all(item["passed"] for item in checks)
    low_band = [frequency for frequency in frequencies if frequency < ROOM_CAVEAT_UPPER_HZ]
    original_coverage = report.get("gate", {}).get("minimum_mode_coverage_passed")
    if original_coverage is not True:
        raise AuditError("Pitcher numeric mode-coverage result changed")
    return {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "PitcherPhysicalRejectionCausallyConfoundedByObservationAdmissionFailure",
        "claim": "POST_HOC_CAUSAL_AUDIT_OF_FROZEN_REPORT_ONLY / ORIGINAL_REJECTION_UNCHANGED / NO_RETUNING_NO_NEW_AUDIO_NO_PLANTER_ACCESS",
        "pitcher_report_sha256": PITCHER_REPORT_SHA256,
        "audit_script_sha256": sha256_bytes(Path(__file__).read_bytes()),
        "bound_observation_gate_sources": sources,
        "observation_profile": {
            "id": "realimpact-transfer-v2-holdout-thresholds-reused-without-selection",
            "checks": checks,
            "passed": observation_passed,
        },
        "room_band_diagnostic": {
            "published_caveat_upper_hz": ROOM_CAVEAT_UPPER_HZ,
            "selected_mode_count_below_caveat": len(low_band),
            "selected_mode_fraction_below_caveat": len(low_band) / selected,
            "frequencies_hz_below_caveat": low_band,
        },
        "gate_gap": {
            "original_numeric_mode_coverage_passed": original_coverage,
            "observation_admission_passed": observation_passed,
            "physical_frequency_and_field_gates_consumed_unadmitted_observation": (
                original_coverage and not observation_passed
            ),
        },
        "interpretation": {
            "still_supported": "the frozen combined scalar-proxy/extractor/transfer protocol is rejected on Pitcher",
            "not_supported": "the result uniquely attributes the rejection to scalar mechanics or acoustic transfer",
            "smallest_next_action": "freeze observation admission before any physics comparison on one unopened development object; evaluate mechanics only if that observation passes",
        },
        "network_requests": 0,
        "audio_payload_bytes_read": 0,
        "planter_audio_payload_bytes_read": 0,
    }


def publish(output: Path, report_bytes: bytes) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    try:
        (staging / "report.json").write_bytes(report_bytes)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def main() -> int:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    pitcher_path = external_file(root, arguments.pitcher_report)
    output = external_output(root, arguments.output)
    _, pitcher_report = load_pitcher_report(pitcher_path)
    sources = validate_bound_sources(root)
    report = audit(pitcher_report, sources)
    report_bytes = canonical_json(report)
    publish(output, report_bytes)
    print(f"Pitcher observation audit: {output.resolve()}")
    print(f"decision: {report['decision']}")
    print(f"report sha256: {sha256_bytes(report_bytes)}")
    print("network requests: 0")
    print("audio payload bytes read: 0")
    print("Planter audio payload bytes read: 0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
