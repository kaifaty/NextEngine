"""R2 baseline estimators fitted from frozen reliability replay outputs.

Implements the §12 baseline ladder of ``DEV-SPEECH-RELIABILITY-001`` on top
of replay ``results.jsonl`` files whose rows embed the service-side
recognition-reliability feature payload plus the deterministic
``exact_match`` score:

1. a constant base-rate predictor;
2. a deterministic revision-stability rule with no learned weights;
3. an L2-regularized logistic regression for ``exact_match``.

Discipline:

- Only ``completed`` turns with ``completeness == "complete"`` enter the
  table; everything else is counted and excluded, never imputed.
- Fitting uses ``split == "train"`` rows exclusively; ``calibration`` rows
  are reported; ``held_out`` rows are refused outright to keep the
  evaluation partition pristine.
- Wall-clock features (``*_wall_ms``) are excluded on purpose: they differ
  between identical replays and would leak run identity into weights.
- Missing optional timings become documented sentinels (``-1``).
- The logistic model standardizes with train-only statistics and solves a
  fixed 25-step Newton-Raphson ridge iteration — fully deterministic,
  numpy-only.
- The exported candidate carries bounded coefficients, normalization
  constants and complete lineage; it publishes atomically with private
  permissions outside this repository.

None of this claims calibrated reliability yet: §16 gates require held-out
evaluation that has not been run.
"""

from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path
from typing import Sequence

import numpy as np

from .profile import REPOSITORY_ROOT
from .reliability import _atomic_write


BASELINE_KIND = "nextengine.speech-reliability.baseline-report"
CANDIDATE_KIND = "nextengine.speech-reliability.baseline-candidate"
SCHEMA_VERSION = 0

LOGISTIC_L2_LAMBDA = 1.0
LOGISTIC_NEWTON_STEPS = 25
ECE_BINS = 10
MAX_RESULTS_FILES = 16
MAX_ROWS = 400_000

FEATURE_NAMES: tuple[str, ...] = (
    "revisions_received",
    "nonempty_revisions",
    "cumulative_churn",
    "max_churn",
    "stable_prefix_ratio",
    "final_to_previous_edit_distance",
    "final_char_count",
    "final_word_count",
    "utterance_duration_ms",
    "first_text_audio_ms_sentinel",
    "last_text_change_audio_ms_sentinel",
    "all_revisions_empty",
    "final_empty",
    "text_appeared_then_vanished",
    "final_matches_previous",
    "speech_samples",
    "speech_ratio",
    "vad_segment_count",
    "raw_rms_dbfs",
    "raw_peak_dbfs",
    "raw_nonzero_ratio",
    "raw_clipping_ratio",
    "ingress_frames",
    "discontinuous_frames",
    "route_sample_deficit_abs",
    "scheduler_overloads",
    "asr_job_failures",
    "preprocessor_active",
    "algorithmic_delay_ms",
)

RULE_CONSTANTS = {
    "max_revisions_for_stable": 2,
    "min_speech_ratio": 0.15,
    "max_clipping_ratio": 0.01,
}


class ReliabilityBaselineError(RuntimeError):
    """A stable fail-closed baseline-fitting error."""


def _finite(value: object, default: float = -1.0) -> float:
    if isinstance(value, bool) or value is None:
        return default
    if isinstance(value, (int, float)):
        numeric = float(value)
        return numeric if math.isfinite(numeric) else default
    return default


def _bool01(value: object) -> float:
    return 1.0 if value is True else 0.0


def feature_vector(payload: dict[str, object]) -> dict[str, float]:
    transcript = payload.get("transcript")
    acoustic = payload.get("acoustic")
    runtime = payload.get("runtime")
    raw = acoustic.get("raw") if isinstance(acoustic, dict) else None
    if not all(
        isinstance(part, dict) for part in (transcript, acoustic, runtime, raw)
    ):
        raise ReliabilityBaselineError("feature payload sections are incomplete")

    values: dict[str, float] = {
        "revisions_received": _finite(transcript.get("revisions_received")),
        "nonempty_revisions": _finite(transcript.get("nonempty_revisions")),
        "cumulative_churn": _finite(transcript.get("cumulative_churn")),
        "max_churn": _finite(transcript.get("max_churn")),
        "stable_prefix_ratio": _finite(transcript.get("stable_prefix_ratio")),
        "final_to_previous_edit_distance": _finite(
            transcript.get("final_to_previous_edit_distance")
        ),
        "final_char_count": _finite(transcript.get("final_char_count")),
        "final_word_count": _finite(transcript.get("final_word_count")),
        "utterance_duration_ms": _finite(transcript.get("utterance_duration_ms")),
        "first_text_audio_ms_sentinel": _finite(transcript.get("first_text_audio_ms")),
        "last_text_change_audio_ms_sentinel": _finite(
            transcript.get("last_change_audio_ms")
        ),
        "all_revisions_empty": _bool01(transcript.get("all_revisions_empty")),
        "final_empty": _bool01(transcript.get("final_empty")),
        "text_appeared_then_vanished": _bool01(
            transcript.get("text_appeared_then_vanished")
        ),
        "final_matches_previous": _bool01(transcript.get("final_matches_previous")),
        "speech_samples": _finite(acoustic.get("speech_samples")),
        "speech_ratio": _finite(acoustic.get("speech_ratio")),
        "vad_segment_count": _finite(acoustic.get("vad_segment_count")),
        "raw_rms_dbfs": _finite(raw.get("rms_dbfs")),
        "raw_peak_dbfs": _finite(raw.get("peak_dbfs")),
        "raw_nonzero_ratio": _finite(raw.get("nonzero_ratio")),
        "raw_clipping_ratio": _finite(raw.get("clipping_ratio")),
        "ingress_frames": _finite(runtime.get("ingress_frames")),
        "discontinuous_frames": _finite(runtime.get("discontinuous_frames")),
        "route_sample_deficit_abs": abs(_finite(runtime.get("route_sample_deficit"))),
        "scheduler_overloads": _finite(runtime.get("scheduler_overloads")),
        "asr_job_failures": _finite(runtime.get("asr_job_failures")),
        "preprocessor_active": _bool01(runtime.get("preprocessor_active")),
        "algorithmic_delay_ms": _finite(runtime.get("algorithmic_delay_ms")),
    }
    assert set(values) == set(FEATURE_NAMES)
    return values


def load_training_table(
    results_paths: Sequence[Path],
) -> tuple[np.ndarray, np.ndarray, np.ndarray, dict[str, int]]:
    """Build (X, y, split_code, counts) from one or more results files."""

    if not results_paths:
        raise ReliabilityBaselineError("at least one results file is required")
    if len(results_paths) > MAX_RESULTS_FILES:
        raise ReliabilityBaselineError("too many results files")

    xs: list[list[float]] = []
    ys: list[float] = []
    splits: list[str] = []
    excluded = {
        "incomplete_payload": 0,
        "not_completed": 0,
        "missing_score": 0,
        "held_out_refused": 0,
    }
    for path in results_paths:
        resolved = path.expanduser().resolve()
        if resolved == REPOSITORY_ROOT or resolved.is_relative_to(REPOSITORY_ROOT):
            raise ReliabilityBaselineError("results files must be external")
        try:
            lines = resolved.read_text(encoding="utf-8").splitlines()
        except OSError as error:
            raise ReliabilityBaselineError(f"cannot read results: {error}") from error
        for line in lines:
            if len(xs) >= MAX_ROWS:
                raise ReliabilityBaselineError("training table exceeds row cap")
            if not line.strip():
                continue
            row = json.loads(line)
            payload = row.get("feature_payload")
            if not isinstance(payload, dict) or payload.get("completeness") != (
                "complete"
            ):
                excluded["incomplete_payload"] += 1
                continue
            if row.get("outcome") != "completed":
                excluded["not_completed"] += 1
                continue
            scores = row.get("scores")
            exact = scores.get("exact_match") if isinstance(scores, dict) else None
            if exact is None:
                excluded["missing_score"] += 1
                continue
            split = str(row.get("split"))
            if split == "held_out":
                excluded["held_out_refused"] += 1
                continue
            vector = feature_vector(payload)
            xs.append([vector[name] for name in FEATURE_NAMES])
            ys.append(1.0 if exact is True else 0.0)
            splits.append(split)
    if not xs:
        raise ReliabilityBaselineError("no admissible rows in the inputs")
    x = np.asarray(xs, dtype=np.float64)
    y = np.asarray(ys, dtype=np.float64)
    codes = np.asarray([{"train": 0, "calibration": 1}.get(s, -1) for s in splits])
    if bool(np.any(codes == -1)):
        raise ReliabilityBaselineError("unknown split label encountered")
    return x, y, codes, excluded


def brier_score(y: np.ndarray, p: np.ndarray) -> float:
    return float(np.mean((p - y) ** 2))


def expected_calibration_error(y: np.ndarray, p: np.ndarray) -> float:
    edges = np.linspace(0.0, 1.0, ECE_BINS + 1)
    total = float(y.size)
    ece = 0.0
    for index in range(ECE_BINS):
        low, high = edges[index], edges[index + 1]
        mask = (p > low) & (p <= high) if index else (p >= low) & (p <= high)
        count = int(np.sum(mask))
        if count == 0:
            continue
        ece += (count / total) * abs(float(np.mean(p[mask])) - float(np.mean(y[mask])))
    return ece


def accuracy_at_threshold(y: np.ndarray, p: np.ndarray, threshold: float = 0.5) -> float:
    return float(np.mean((p >= threshold).astype(np.float64) == y))


def constant_predictor(y_train: np.ndarray) -> float:
    return float(np.clip(np.mean(y_train), 1e-6, 1 - 1e-6))


def stability_rule_predict(x: np.ndarray) -> np.ndarray:
    """Fixed a-priori thresholds; no learned weights, no tuning."""

    by = {name: x[:, index] for index, name in enumerate(FEATURE_NAMES)}
    stable = (
        (by["revisions_received"] <= RULE_CONSTANTS["max_revisions_for_stable"])
        & (by["final_empty"] == 0.0)
        & (by["all_revisions_empty"] == 0.0)
        & (by["text_appeared_then_vanished"] == 0.0)
        & (by["raw_clipping_ratio"] <= RULE_CONSTANTS["max_clipping_ratio"])
        & (by["speech_ratio"] >= RULE_CONSTANTS["min_speech_ratio"])
        & (by["asr_job_failures"] == 0.0)
    )
    return np.where(stable, 0.95, 0.05)


def _sigmoid(z: np.ndarray) -> np.ndarray:
    return 1.0 / (1.0 + np.exp(-np.clip(z, -30.0, 30.0)))


def fit_logistic_l2(
    x_train: np.ndarray, y_train: np.ndarray
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Standardize and solve ridge logistic via fixed Newton iterations."""

    mean = x_train.mean(axis=0)
    std = x_train.std(axis=0)
    std[std == 0.0] = 1.0
    standardized = (x_train - mean) / std
    design = np.column_stack([np.ones(standardized.shape[0]), standardized])
    weights = np.zeros(design.shape[1])
    penalty = np.diag(
        np.concatenate([[0.0], np.full(design.shape[1] - 1, LOGISTIC_L2_LAMBDA)])
    )
    for _ in range(LOGISTIC_NEWTON_STEPS):
        probabilities = _sigmoid(design @ weights)
        gradient = design.T @ (probabilities - y_train) + penalty @ weights
        hessian = (design.T * (probabilities * (1.0 - probabilities))) @ design + (
            penalty
        )
        try:
            step = np.linalg.solve(hessian, gradient)
        except np.linalg.LinAlgError as error:
            raise ReliabilityBaselineError(
                f"logistic Hessian is singular: {error}"
            ) from error
        weights -= step
        if float(np.max(np.abs(step))) < 1e-10:
            break
    return mean, std, weights


def logistic_predict(mean: np.ndarray, std: np.ndarray, weights: np.ndarray, x: np.ndarray) -> np.ndarray:
    standardized = (x - mean) / std
    return _sigmoid(weights[0] + standardized @ weights[1:])


def _metrics(y: np.ndarray, p: np.ndarray) -> dict[str, float]:
    return {
        "brier": round(brier_score(y, p), 6),
        "ece_10bin": round(expected_calibration_error(y, p), 6),
        "accuracy_at_05": round(accuracy_at_threshold(y, p), 6),
        "n": int(y.size),
    }


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"


def _write_candidate(path: Path, payload: dict[str, object]) -> str:
    destination = path.expanduser().resolve()
    if destination == REPOSITORY_ROOT or destination.is_relative_to(REPOSITORY_ROOT):
        raise ReliabilityBaselineError("candidates must stay outside the repository")
    destination.parent.mkdir(parents=True, exist_ok=True)
    encoded = json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True).encode(
        "utf-8"
    )
    _atomic_write(destination, encoded)
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def fit_baselines(
    results_paths: Sequence[Path],
    *,
    out_dir: Path,
    lineage: dict[str, object] | None = None,
) -> dict[str, object]:
    """Fit the §12 ladder and publish candidates plus an honest report."""

    directory = out_dir.expanduser().resolve()
    if directory == REPOSITORY_ROOT or directory.is_relative_to(REPOSITORY_ROOT):
        raise ReliabilityBaselineError("output directory must be external")
    directory.mkdir(parents=True, exist_ok=True)

    x, y, codes, excluded = load_training_table(results_paths)
    train_mask = codes == 0
    calibration_mask = codes == 1
    if int(train_mask.sum()) < 10:
        raise ReliabilityBaselineError(
            "fewer than ten admissible train rows; refusing to fit"
        )

    x_train, y_train = x[train_mask], y[train_mask]
    lineage_payload = {
        "results_sha256": {str(path): _sha256_file(path) for path in results_paths},
        **(lineage or {}),
    }

    baselines: dict[str, dict[str, object]] = {}
    constant_rate = constant_predictor(y_train)

    def evaluate(name: str, p_all: np.ndarray) -> None:
        entry: dict[str, object] = {
            "train": _metrics(y_train, p_all[train_mask]),
        }
        if bool(calibration_mask.any()):
            entry["calibration"] = _metrics(y[calibration_mask], p_all[calibration_mask])
        else:
            entry["calibration"] = "not_run_no_calibration_rows_in_input"
        baselines[name] = entry

    evaluate("constant", np.full(y.size, constant_rate))
    evaluate("stability_rule", stability_rule_predict(x))

    mean, std, weights = fit_logistic_l2(x_train, y_train)
    evaluate("logistic_l2", logistic_predict(mean, std, weights, x))

    def calibration_brier(name: str) -> float | None:
        entry = baselines[name].get("calibration")  # type: ignore[union-attr]
        if isinstance(entry, str):
            return None
        return float(entry["brier"])  # type: ignore[index]

    improvements: dict[str, object] = {}
    constant_calibration = calibration_brier("constant")
    for name in ("stability_rule", "logistic_l2"):
        value = calibration_brier(name)
        improvements[name] = (
            "not_run_no_calibration_rows"
            if value is None or constant_calibration is None
            else round((constant_calibration - value) / constant_calibration, 4)
        )

    candidates: dict[str, str] = {}
    candidates["constant"] = _write_candidate(
        directory / "constant.candidate.json",
        {
            "kind": CANDIDATE_KIND,
            "schema_version": SCHEMA_VERSION,
            "baseline": "constant",
            "feature_names": list(FEATURE_NAMES),
            "base_rate": constant_rate,
            "lineage": lineage_payload,
        },
    )
    candidates["stability_rule"] = _write_candidate(
        directory / "stability-rule.candidate.json",
        {
            "kind": CANDIDATE_KIND,
            "schema_version": SCHEMA_VERSION,
            "baseline": "stability_rule",
            "feature_names": list(FEATURE_NAMES),
            "constants": RULE_CONSTANTS,
            "positive_probability": 0.95,
            "negative_probability": 0.05,
            "lineage": lineage_payload,
        },
    )
    candidates["logistic_l2"] = _write_candidate(
        directory / "logistic-l2.candidate.json",
        {
            "kind": CANDIDATE_KIND,
            "schema_version": SCHEMA_VERSION,
            "baseline": "logistic_l2",
            "feature_names": list(FEATURE_NAMES),
            "standardization_mean": [round(v, 8) for v in mean],
            "standardization_std": [round(v, 8) for v in std],
            "bias": round(float(weights[0]), 8),
            "coefficients": [round(float(v), 8) for v in weights[1:]],
            "l2_lambda": LOGISTIC_L2_LAMBDA,
            "newton_steps": LOGISTIC_NEWTON_STEPS,
            "lineage": lineage_payload,
        },
    )

    report: dict[str, object] = {
        "kind": BASELINE_KIND,
        "schema_version": SCHEMA_VERSION,
        "status": "complete",
        "rows_admissible": int(y.size),
        "excluded_counts": excluded,
        "split_counts": {
            "train": int(train_mask.sum()),
            "calibration": int(calibration_mask.sum()),
        },
        "baselines": baselines,
        "brier_improvement_vs_constant_calibration": improvements,
        "gate_min_10_percent_brier_improvement": {
            name: (
                value if isinstance(value, str) else bool(value >= 0.10)
            )
            for name, value in improvements.items()
        },
        "candidates": candidates,
        "lineage": lineage_payload,
        "honesty": "in-sample/calibration-only fitting evidence; no held-out claim",
    }
    report_path = directory / "baseline-report.json"
    encoded = json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True).encode(
        "utf-8"
    )
    _atomic_write(report_path, encoded)
    report["report_path"] = str(report_path)
    return report
