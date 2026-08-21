"""Bounded streaming feature capture for utterance-level ASR reliability.

This module implements the R1 increment of
``DEV-SPEECH-RELIABILITY-001``.  It accumulates O(1) transcript-revision
statistics for one explicit-finish utterance and assembles a fail-closed,
versioned internal feature payload from them plus separately accumulated
acoustic and runtime counters.

Documented formulas and boundary cases:

- Normalized text is produced by ``ru-asr-normalize-v0``
  (``reliability.normalize_ru_asr``).
- ``churn(a, b) = lev(a, b) / max(1, len(a), len(b))`` over normalized
  texts, always within ``[0, 1]``.  The documented initial previous state is
  the empty string, so the first non-empty revision has churn ``1.0``.
- ``stable_prefix_ratio = common_prefix(norm(stable_prefix), norm(text)) /
  len(norm(text))``, ``0.0`` while the normalized text is empty.
- ``final_to_previous_edit_distance`` compares the final normalized text
  with the last previously observed revision (the empty string when no
  earlier revision existed).
- ``text_appeared_then_vanished`` requires at least one non-empty revision
  and an empty final text; a transient disappearance followed by recovery
  does not set it.
- Clipping counts int16-saturated samples (magnitude >= 32767) as a ratio of
  all received samples of that signal branch.

Boundary notes:

- The payload is an internal ``tools/speech-timeline`` diagnostic.  It is
  not a public engine contract and never becomes gameplay authority.
- No transcript history is retained.  Only bounded scalars plus exactly one
  retained normalized text buffer (the previous revision) are kept, so
  memory does not grow with the number of revisions.
- Raw and ASR-route PCM levels are accumulated by the caller as separate,
  explicitly labeled branches; neither stores audio.
- Every emitted numeric value must be finite and range-checked; NaN or
  Infinity, unknown schema versions, incomplete vectors or identity
  mismatches fail closed with typed reasons instead of degrading into a low
  confidence guess.
"""

from __future__ import annotations

from dataclasses import dataclass
import math

from .reliability import normalize_ru_asr


FEATURE_KIND = "nextengine.speech-reliability.features"
FEATURE_SCHEMA = "speech-reliability-features-v0"
FEATURE_SCHEMA_VERSION = 0

SAMPLE_RATE_HZ = 16_000
#: int16 saturation magnitude treated as a clipped sample.
CLIPPING_SAMPLE_MAGNITUDE = 32_767
MAX_REVISIONS = 4_096
MAX_TEXT_BYTES = 4_096

MIN_DBFS = -120.0
MAX_DBFS = 6.0


class ReliabilityFeatureError(RuntimeError):
    """Typed fail-closed error for reliability feature construction."""

    def __init__(self, reason: str, detail: str) -> None:
        super().__init__(f"{reason}: {detail}")
        self.reason = reason


def _levenshtein_distance(left: str, right: str) -> int:
    """Two-row Levenshtein distance over Unicode scalars."""

    if left == right:
        return 0
    if not left:
        return len(right)
    if not right:
        return len(left)
    previous = list(range(len(right) + 1))
    for left_index, left_unit in enumerate(left, start=1):
        current = [left_index]
        for right_index, right_unit in enumerate(right, start=1):
            substitution = previous[right_index - 1]
            if left_unit != right_unit:
                substitution += 1
            deletion = previous[right_index] + 1
            insertion = current[right_index - 1] + 1
            current.append(min(substitution, deletion, insertion))
        previous = current
    return previous[-1]


def normalized_churn(previous: str, current: str) -> float:
    """Normalized Levenshtein churn between two normalized texts."""

    denominator = max(1, len(previous), len(current))
    return _levenshtein_distance(previous, current) / denominator


def common_prefix_length(left: str, right: str) -> int:
    limit = min(len(left), len(right))
    index = 0
    while index < limit and left[index] == right[index]:
        index += 1
    return index


@dataclass(frozen=True)
class RevisionObservation:
    """One admitted ASR transcript revision (partial or final)."""

    full_text: str
    stable_prefix: str
    final: bool
    audio_end_sample: int
    wall_monotonic_ns: int


class TranscriptRevisionState:
    """Mutable O(1) accumulator over ASR transcript revisions.

    Memory is bounded by fixed scalars plus exactly one retained normalized
    text buffer (the previous revision) capped at ``MAX_TEXT_BYTES`` UTF-8
    bytes after normalization.  No structure grows with revision count.
    """

    def __init__(self) -> None:
        self.revisions_received = 0
        self.nonempty_revisions = 0
        self.first_text_audio_ms: float | None = None
        self.first_text_wall_ms: float | None = None
        self.last_change_audio_ms: float | None = None
        self.last_change_wall_ms: float | None = None
        self.cumulative_churn = 0.0
        self.max_churn = 0.0
        self.last_stable_prefix_ratio = 0.0
        self.final_to_previous_edit_distance: int | None = None
        self.final_matches_previous = False
        self.all_revisions_empty = True
        self._saw_nonempty_revision = False
        self._start_wall_monotonic_ns: int | None = None
        self._previous_normalized: str | None = None

    def observe(self, observation: RevisionObservation) -> None:
        if self.revisions_received >= MAX_REVISIONS:
            raise ReliabilityFeatureError(
                "TOO_MANY_REVISIONS",
                f"more than {MAX_REVISIONS} revisions in one utterance",
            )
        full_text = observation.full_text
        stable_prefix = observation.stable_prefix
        if not isinstance(full_text, str) or not isinstance(stable_prefix, str):
            raise ReliabilityFeatureError(
                "INVALID_REVISION", "revision texts must be strings"
            )
        if (
            len(full_text.encode("utf-8")) > MAX_TEXT_BYTES
            or len(stable_prefix.encode("utf-8")) > MAX_TEXT_BYTES
        ):
            raise ReliabilityFeatureError(
                "TRANSCRIPT_TOO_LARGE",
                "revision text exceeds the bounded transcript size",
            )
        if observation.audio_end_sample < 0:
            raise ReliabilityFeatureError(
                "INVALID_AUDIO_CLOCK", "audio_end_sample must be non-negative"
            )
        try:
            normalized = normalize_ru_asr(full_text)
            stable_normalized = normalize_ru_asr(stable_prefix)
        except Exception as error:
            raise ReliabilityFeatureError(
                "NORMALIZATION_FAILED", f"transcript normalization failed: {error}"
            ) from error
        if self._start_wall_monotonic_ns is None:
            self._start_wall_monotonic_ns = observation.wall_monotonic_ns
        wall_offset_ms = round(
            (observation.wall_monotonic_ns - self._start_wall_monotonic_ns) / 1_000_000,
            3,
        )
        audio_ms = round(observation.audio_end_sample * 1_000 / SAMPLE_RATE_HZ, 3)

        previous = self._previous_normalized
        self.cumulative_churn += normalized_churn(previous or "", normalized)
        self.max_churn = max(self.max_churn, normalized_churn(previous or "", normalized))
        self.revisions_received += 1
        if normalized:
            self.nonempty_revisions += 1
            self.all_revisions_empty = False
            self._saw_nonempty_revision = True
            if self.first_text_audio_ms is None:
                self.first_text_audio_ms = audio_ms
                self.first_text_wall_ms = wall_offset_ms
        if previous is None or normalized != previous:
            self.last_change_audio_ms = audio_ms
            self.last_change_wall_ms = wall_offset_ms
        prefix_units = common_prefix_length(stable_normalized, normalized)
        self.last_stable_prefix_ratio = (
            prefix_units / len(normalized) if normalized else 0.0
        )
        self.final_matches_previous = False
        if observation.final:
            self.final_to_previous_edit_distance = _levenshtein_distance(
                previous or "", normalized
            )
            self.final_matches_previous = previous == normalized
        self._previous_normalized = normalized

    def transcript_features(self, *, utterance_duration_ms: float) -> dict[str, object]:
        """Return the bounded transcript feature mapping."""

        final_normalized = self._previous_normalized or ""
        appeared_then_vanished = self._saw_nonempty_revision and final_normalized == ""
        return {
            "revisions_received": self.revisions_received,
            "nonempty_revisions": self.nonempty_revisions,
            "first_text_audio_ms": self.first_text_audio_ms,
            "first_text_wall_ms": self.first_text_wall_ms,
            "last_change_audio_ms": self.last_change_audio_ms,
            "last_change_wall_ms": self.last_change_wall_ms,
            "cumulative_churn": round(self.cumulative_churn, 6),
            "max_churn": round(self.max_churn, 6),
            "stable_prefix_ratio": round(self.last_stable_prefix_ratio, 6),
            "final_to_previous_edit_distance": self.final_to_previous_edit_distance,
            "final_char_count": len(final_normalized.replace(" ", "")),
            "final_word_count": len(final_normalized.split()),
            "utterance_duration_ms": round(max(0.0, utterance_duration_ms), 3),
            "all_revisions_empty": self.all_revisions_empty,
            "final_empty": final_normalized == "",
            "text_appeared_then_vanished": appeared_then_vanished,
            "final_matches_previous": self.final_matches_previous,
        }


def _finite_or_none(value: object) -> float | None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return None
    numeric = float(value)
    return numeric if math.isfinite(numeric) else None


def _require_level_group(name: str, group: dict[str, object]) -> dict[str, object]:
    resolved = dict(group)
    samples = group.get("samples")
    if isinstance(samples, bool) or not isinstance(samples, int) or samples < 0:
        raise ReliabilityFeatureError(
            "NON_FINITE_FEATURE", f"{name}.samples is missing or invalid"
        )
    for key in ("rms_dbfs", "peak_dbfs"):
        numeric = _finite_or_none(group.get(key))
        if numeric is None:
            # An empty signal branch legitimately reports no measurable
            # level; pin it to the documented silence floor instead of
            # failing the whole vector.
            if samples == 0:
                resolved[key] = MIN_DBFS
                continue
            raise ReliabilityFeatureError(
                "NON_FINITE_FEATURE", f"{name}.{key} is missing or non-finite"
            )
        if not MIN_DBFS <= numeric <= MAX_DBFS:
            raise ReliabilityFeatureError(
                "OUT_OF_RANGE_FEATURE",
                f"{name}.{key}={numeric} outside [{MIN_DBFS}, {MAX_DBFS}]",
            )
    for key in ("nonzero_ratio", "clipping_ratio"):
        numeric = _finite_or_none(group.get(key))
        if numeric is None:
            if samples == 0:
                resolved[key] = 0.0
                continue
            raise ReliabilityFeatureError(
                "NON_FINITE_FEATURE", f"{name}.{key} is missing or non-finite"
            )
        if not 0.0 <= numeric <= 1.0:
            raise ReliabilityFeatureError(
                "OUT_OF_RANGE_FEATURE", f"{name}.{key}={numeric} outside [0, 1]"
            )
    return resolved


def _check_int(name: str, value: object) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ReliabilityFeatureError(
            "NON_FINITE_FEATURE", f"{name} is missing or not an integer"
        )
    if not 0 <= value <= 2**31 - 1:
        raise ReliabilityFeatureError(
            "OUT_OF_RANGE_FEATURE", f"{name}={value} outside u31 bounds"
        )
    return value


def build_feature_payload(
    *,
    identities: dict[str, object],
    transcript: dict[str, object],
    raw_signal: dict[str, object],
    route_signal: dict[str, object],
    noise_floor_dbfs: float | None,
    noise_floor_source: str,
    speech_samples: int,
    speech_ratio: float,
    vad_segment_count: int,
    ingress_frames: int,
    discontinuous_frames: int,
    route_sample_deficit: int,
    scheduler_overloads: int,
    asr_job_failures: int,
    preprocessor_active: bool,
    algorithmic_delay_ms: int,
    expected_adapter_id: str | None = None,
) -> dict[str, object]:
    """Validate and assemble one fail-closed reliability feature payload.

    Raises :class:`ReliabilityFeatureError` on unknown schema versions,
    missing/mismatched identity, non-finite or out-of-range values, so a
    broken vector never becomes a silently low-confidence guess.
    """

    if identities.get("feature_schema") != FEATURE_SCHEMA:
        raise ReliabilityFeatureError(
            "UNKNOWN_FEATURE_SCHEMA",
            f"unsupported feature schema: {identities.get('feature_schema')!r}",
        )
    for key in ("asr_model", "asr_adapter_id", "audio_route"):
        value = identities.get(key)
        if not isinstance(value, str) or not value:
            raise ReliabilityFeatureError(
                "IDENTITY_MISMATCH", f"missing identity field {key}"
            )
    if expected_adapter_id is not None and identities["asr_adapter_id"] != (
        expected_adapter_id
    ):
        raise ReliabilityFeatureError(
            "IDENTITY_MISMATCH",
            f"adapter identity changed: expected {expected_adapter_id!r}, "
            f"got {identities['asr_adapter_id']!r}",
        )
    if identities.get("sample_rate_hz") != SAMPLE_RATE_HZ:
        raise ReliabilityFeatureError(
            "IDENTITY_MISMATCH",
            f"sample rate must be {SAMPLE_RATE_HZ}, got "
            f"{identities.get('sample_rate_hz')!r}",
        )

    raw_signal = _require_level_group("raw_signal", raw_signal)
    route_signal = _require_level_group("route_signal", route_signal)

    checked_transcript: dict[str, object] = {}
    for key, value in transcript.items():
        if isinstance(value, float):
            numeric = _finite_or_none(value)
            if numeric is None or abs(numeric) > 1e12:
                raise ReliabilityFeatureError(
                    "NON_FINITE_FEATURE" if numeric is None else "OUT_OF_RANGE_FEATURE",
                    f"transcript.{key} is non-finite"
                    if numeric is None
                    else f"transcript.{key} exceeds the bounded magnitude",
                )
        checked_transcript[key] = value

    acoustic: dict[str, object] = {
        "noise_floor_source": noise_floor_source,
        "noise_floor_dbfs": None,
        "speech_samples": _check_int("speech_samples", speech_samples),
        "speech_ratio": None,
        "vad_segment_count": _check_int("vad_segment_count", vad_segment_count),
        "raw": dict(raw_signal),
        "asr_route": dict(route_signal),
        "route_minus_raw_rms_dbfs": None,
        "route_minus_raw_peak_dbfs": None,
    }
    if noise_floor_dbfs is not None:
        floor = _finite_or_none(noise_floor_dbfs)
        if floor is None or not MIN_DBFS <= floor <= MAX_DBFS:
            raise ReliabilityFeatureError(
                "NON_FINITE_FEATURE" if floor is None else "OUT_OF_RANGE_FEATURE",
                "noise_floor_dbfs is non-finite"
                if floor is None
                else "noise_floor_dbfs outside the calibrated range",
            )
        acoustic["noise_floor_dbfs"] = floor
    speech_ratio_value = _finite_or_none(speech_ratio)
    if speech_ratio_value is None or not 0.0 <= speech_ratio_value <= 1.0:
        raise ReliabilityFeatureError(
            "NON_FINITE_FEATURE" if speech_ratio_value is None else "OUT_OF_RANGE_FEATURE",
            "speech_ratio is non-finite" if speech_ratio_value is None else "speech_ratio outside [0, 1]",
        )
    acoustic["speech_ratio"] = speech_ratio_value
    raw_rms = _finite_or_none(raw_signal.get("rms_dbfs"))
    route_rms = _finite_or_none(route_signal.get("rms_dbfs"))
    raw_peak = _finite_or_none(raw_signal.get("peak_dbfs"))
    route_peak = _finite_or_none(route_signal.get("peak_dbfs"))
    if raw_rms is not None and route_rms is not None:
        acoustic["route_minus_raw_rms_dbfs"] = round(route_rms - raw_rms, 3)
    if raw_peak is not None and route_peak is not None:
        acoustic["route_minus_raw_peak_dbfs"] = round(route_peak - raw_peak, 3)

    deficit = _finite_or_none(route_sample_deficit)
    if deficit is None or abs(deficit) > 2**31 - 1:
        raise ReliabilityFeatureError(
            "NON_FINITE_FEATURE", "route_sample_deficit is missing or unbounded"
        )

    runtime: dict[str, object] = {
        "ingress_frames": _check_int("ingress_frames", ingress_frames),
        "discontinuous_frames": _check_int(
            "discontinuous_frames", discontinuous_frames
        ),
        "route_sample_deficit": route_sample_deficit,
        "scheduler_overloads": _check_int("scheduler_overloads", scheduler_overloads),
        "asr_job_failures": _check_int("asr_job_failures", asr_job_failures),
        "preprocessor_active": bool(preprocessor_active),
        "algorithmic_delay_ms": _check_int(
            "algorithmic_delay_ms", algorithmic_delay_ms
        ),
    }

    payload: dict[str, object] = {
        "kind": FEATURE_KIND,
        "schema_version": FEATURE_SCHEMA_VERSION,
        "feature_schema": FEATURE_SCHEMA,
        "completeness": "complete",
        "invalid_reason": None,
        "identity": dict(identities),
        "transcript": checked_transcript,
        "acoustic": acoustic,
        "runtime": runtime,
    }
    # Coarse wire-bound guard: the payload rides inside terminal metrics.
    if len(repr(payload)) > 48_000:
        raise ReliabilityFeatureError(
            "PAYLOAD_TOO_LARGE", "feature payload exceeds its bounded size"
        )
    return payload


def incomplete_feature_payload(reason: str, detail: str) -> dict[str, object]:
    """Typed incomplete payload for fail-closed capture paths."""

    if not reason or not isinstance(reason, str):
        raise ValueError("incomplete payload needs a reason code")
    return {
        "kind": FEATURE_KIND,
        "schema_version": FEATURE_SCHEMA_VERSION,
        "feature_schema": FEATURE_SCHEMA,
        "completeness": "incomplete",
        "invalid_reason": reason,
        "detail": str(detail)[:256],
    }
