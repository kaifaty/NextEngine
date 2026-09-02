"""Deterministic bounded zero-padded transfer alignment for V26 M0b."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass, replace
from typing import Any

import numpy as np
import physical_sound_v25_m0a_common as base

OFFICIAL_TRANSFER_SAMPLES = 230_215
OFFICIAL_OUTPUT_SAMPLES = 144_000
OFFICIAL_ANCHOR = 512
MAXIMUM_PADDING = 512
MINIMUM_COPIED_SOURCE_SAMPLES = 142_976


@dataclass(frozen=True)
class AlignmentLimits:
    source_samples: int
    output_samples: int
    anchor: int
    maximum_leading_padding: int
    maximum_trailing_padding: int
    minimum_copied_source_samples: int


@dataclass(frozen=True)
class AlignedTransfer:
    samples: np.ndarray
    source_peak_index: int
    raw_start: int
    source_start: int
    destination_start: int
    copied_source_samples: int
    leading_padding_samples: int
    trailing_padding_samples: int
    output_sha256: str

    def canonical_record(self, source_sha256: str) -> dict[str, Any]:
        return {
            "transform_id": "realimpact-peak-anchor-padded-v1",
            "source_sha256": source_sha256,
            "source_peak_index": self.source_peak_index,
            "raw_start": self.raw_start,
            "source_start": self.source_start,
            "destination_start": self.destination_start,
            "copied_source_samples": self.copied_source_samples,
            "leading_padding_samples": self.leading_padding_samples,
            "trailing_padding_samples": self.trailing_padding_samples,
            "output_sha256": self.output_sha256,
        }


def limits(profile: base.ExecutionProfile) -> AlignmentLimits:
    if profile.profile_id == "official-v1" and (
        profile.transfer_samples,
        profile.transfer_window,
        profile.transfer_anchor,
    ) == (OFFICIAL_TRANSFER_SAMPLES, OFFICIAL_OUTPUT_SAMPLES, OFFICIAL_ANCHOR):
        return AlignmentLimits(
            OFFICIAL_TRANSFER_SAMPLES,
            OFFICIAL_OUTPUT_SAMPLES,
            OFFICIAL_ANCHOR,
            MAXIMUM_PADDING,
            MAXIMUM_PADDING,
            MINIMUM_COPIED_SOURCE_SAMPLES,
        )
    if profile.profile_id == "contract-fixture-v1" and (
        profile.transfer_samples,
        profile.transfer_window,
        profile.transfer_anchor,
    ) == (8_192, 4_096, 128):
        return AlignmentLimits(8_192, 4_096, 128, 128, 128, 3_840)
    raise base.M0Error("M0b alignment execution profile changed")


def _positive_zero_padding(
    samples: np.ndarray, destination_start: int, copied: int
) -> bool:
    bits = samples.view(np.uint32)
    before = bits[:destination_start]
    after = bits[destination_start + copied :]
    return bool(np.all(before == 0) and np.all(after == 0))


def _align_source(source: np.ndarray, selected: AlignmentLimits) -> AlignedTransfer:
    if source.dtype != np.dtype("<f4") or source.shape != (selected.source_samples,):
        raise base.M0Error("M0b transfer source shape or dtype changed")
    if not np.all(np.isfinite(source)):
        raise base.M0Error("M0b transfer is non-finite")
    source_bits = source.view(np.uint32)
    if np.any(source_bits == np.uint32(0x80000000)):
        raise base.M0Error("M0b transfer contains forbidden signed zero")
    absolute = np.abs(source)
    maximum = float(np.max(absolute))
    if maximum <= 0.0:
        raise base.M0Error("M0b transfer is silent")
    peak = int(np.argmax(absolute))
    raw_start = peak - selected.anchor
    source_start = max(0, raw_start)
    destination_start = max(0, -raw_start)
    copied = min(
        selected.source_samples - source_start,
        selected.output_samples - destination_start,
    )
    if copied <= 0:
        raise base.M0Error("M0b transfer alignment has no source overlap")
    output = np.zeros(selected.output_samples, dtype="<f4")
    output[destination_start : destination_start + copied] = source[
        source_start : source_start + copied
    ]
    leading = destination_start
    trailing = selected.output_samples - destination_start - copied
    if (
        leading > selected.maximum_leading_padding
        or trailing > selected.maximum_trailing_padding
    ):
        raise base.M0Error("M0b transfer alignment exceeds the frozen padding ceiling")
    if copied < selected.minimum_copied_source_samples:
        raise base.M0Error("M0b transfer alignment copied too few source samples")
    if int(np.argmax(np.abs(output))) != selected.anchor:
        raise base.M0Error("M0b transfer peak did not land on the frozen anchor")
    if not _positive_zero_padding(output, destination_start, copied):
        raise base.M0Error("M0b transfer padding is not positive float32 zero")
    payload = output.tobytes()
    return AlignedTransfer(
        output,
        peak,
        raw_start,
        source_start,
        destination_start,
        copied,
        leading,
        trailing,
        base.sha256_bytes(payload),
    )


def align_transfer(data: bytes, profile: base.ExecutionProfile) -> AlignedTransfer:
    selected = limits(profile)
    if len(data) != selected.source_samples * 4:
        raise base.M0Error("M0b transfer sample count changed")
    return _align_source(np.frombuffer(data, dtype="<f4"), selected)


def _expect_reject(operation: Callable[[], object], role: str) -> None:
    try:
        operation()
    except base.M0Error:
        return
    raise base.M0Error(f"M0b alignment mutation did not reject: {role}")


def conformance_report(implementation_root_sha256: str) -> dict[str, Any]:
    profile = base.execution_profile("official-v1")
    selected = limits(profile)
    valid_cases = (0, 39, 73, 87, 511, 512, 72_000, 87_239)
    records = []
    for case_index, peak in enumerate(valid_cases):
        source = np.zeros(selected.source_samples, dtype="<f4")
        source[1_024 + case_index] = np.float32(
            np.nextafter(0.0, 1.0, dtype=np.float32)
        )
        source[2_048 + case_index] = np.float32(-0.125)
        source[peak] = np.float32(-1.0 if case_index % 2 else 1.0)
        result = _align_source(source, selected)
        copied_source = source[
            result.source_start : result.source_start + result.copied_source_samples
        ]
        copied_output = result.samples[
            result.destination_start : result.destination_start
            + result.copied_source_samples
        ]
        if not np.array_equal(
            copied_source.view(np.uint32), copied_output.view(np.uint32)
        ):
            raise base.M0Error(
                "M0b alignment fixture did not preserve copied source bytes"
            )
        records.append(
            {
                "case": f"peak-{peak}",
                "peak": peak,
                "leading": result.leading_padding_samples,
                "trailing": result.trailing_padding_samples,
                "copied": result.copied_source_samples,
                "output_sha256": result.output_sha256,
            }
        )
    tie = np.zeros(selected.source_samples, dtype="<f4")
    tie[73] = np.float32(-1.0)
    tie[87] = np.float32(1.0)
    tie_result = _align_source(tie, selected)
    if tie_result.source_peak_index != 73:
        raise base.M0Error(
            "M0b alignment fixture did not choose the first absolute maximum"
        )
    records.append(
        {
            "case": "equal-absolute-maxima",
            "peak": tie_result.source_peak_index,
            "leading": tie_result.leading_padding_samples,
            "trailing": tie_result.trailing_padding_samples,
            "copied": tie_result.copied_source_samples,
            "output_sha256": tie_result.output_sha256,
        }
    )

    mutations: list[tuple[str, Callable[[], object]]] = []
    zero = np.zeros(selected.source_samples, dtype="<f4")
    nan = zero.copy()
    nan[10] = np.float32(np.nan)
    infinity = zero.copy()
    infinity[10] = np.float32(np.inf)
    signed_zero = zero.copy()
    signed_zero[10] = np.float32(-0.0)
    late = zero.copy()
    late[87_240] = np.float32(1.0)
    early = zero.copy()
    early[0] = np.float32(1.0)
    below_copy_floor = zero.copy()
    below_copy_floor[87_752] = np.float32(1.0)
    mutations.extend(
        (
            ("all-zero", lambda: _align_source(zero, selected)),
            ("nan", lambda: _align_source(nan, selected)),
            ("inf", lambda: _align_source(infinity, selected)),
            ("signed-zero", lambda: _align_source(signed_zero, selected)),
            ("wrong-byte-count", lambda: align_transfer(zero.tobytes()[:-1], profile)),
            ("trailing-padding-plus-one", lambda: _align_source(late, selected)),
            (
                "leading-padding-limit-minus-one",
                lambda: _align_source(
                    early, replace(selected, maximum_leading_padding=511)
                ),
            ),
            (
                "copied-source-floor-minus-one",
                lambda: _align_source(
                    below_copy_floor,
                    replace(selected, maximum_trailing_padding=2_000),
                ),
            ),
        )
    )
    for role, operation in mutations:
        _expect_reject(operation, role)
    return {
        "fixture_id": "padded-absolute-peak-alignment-v1",
        "protocol_profile": "official-v1",
        "implementation_root_sha256": implementation_root_sha256,
        "source_samples": selected.source_samples,
        "output_samples": selected.output_samples,
        "anchor": selected.anchor,
        "maximum_leading_padding": selected.maximum_leading_padding,
        "maximum_trailing_padding": selected.maximum_trailing_padding,
        "minimum_copied_source_samples": selected.minimum_copied_source_samples,
        "valid_case_count": len(records),
        "rejection_case_count": len(mutations),
        "canonical_case_root_sha256": base.sha256_bytes(base.canonical_json(records)),
        "model_values_opened": False,
        "protected_roles_opened": False,
        "runtime_authorized": False,
    }
