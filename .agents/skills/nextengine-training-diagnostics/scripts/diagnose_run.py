#!/usr/bin/env python3
"""Verify and diagnose a closed NextEngine reference PPO run."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import statistics
import sys
from pathlib import Path
from typing import Any, Iterable


MAX_JSON_BYTES = 64 * 1024 * 1024
MAX_METRICS_BYTES = 512 * 1024 * 1024
MAX_METRIC_RECORDS = 250_000
MAX_JSONL_LINE_BYTES = 1024 * 1024
METRIC_FIELDS = (
    "action_standard_deviation_mean",
    "approximate_kl",
    "clip_fraction",
    "early_stop_kl",
    "entropy_estimate",
    "gradient_norm",
    "policy_loss",
    "rollout_failure_count",
    "rollout_mean_reward",
    "rollout_reference_complete_count",
    "update_minibatches",
    "value_loss",
)
COUNTER_FIELDS = ("iteration", "optimizer_steps", "samples")


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def finite_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(float(value))


def nonfinite_paths(value: Any, prefix: str = "$") -> list[str]:
    found: list[str] = []
    if isinstance(value, float) and not math.isfinite(value):
        found.append(prefix)
    elif isinstance(value, dict):
        for key, item in value.items():
            found.extend(nonfinite_paths(item, f"{prefix}.{key}"))
    elif isinstance(value, list):
        for index, item in enumerate(value):
            found.extend(nonfinite_paths(item, f"{prefix}[{index}]"))
    return found


def is_below(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return path != parent
    except ValueError:
        return False


def relative_artifact(run_dir: Path, value: Any, label: str, issues: list[dict[str, Any]]) -> Path | None:
    if not isinstance(value, str) or not value:
        issues.append(
            {
                "severity": "error",
                "code": f"artifact.{label}.path",
                "message": f"{label} path must be a non-empty relative string",
            }
        )
        return None
    candidate = (run_dir / value).resolve()
    if not is_below(candidate, run_dir):
        issues.append(
            {
                "severity": "error",
                "code": f"artifact.{label}.escape",
                "message": f"{label} path escapes the run directory",
                "actual": value,
            }
        )
        return None
    return candidate


def load_json_object(path: Path, label: str, issues: list[dict[str, Any]]) -> dict[str, Any] | None:
    if not path.is_file():
        issues.append({"severity": "error", "code": f"artifact.{label}.missing", "message": f"missing {label}", "actual": str(path)})
        return None
    if path.stat().st_size > MAX_JSON_BYTES:
        issues.append(
            {
                "severity": "error",
                "code": f"artifact.{label}.too_large",
                "message": f"{label} exceeds the bounded JSON size",
                "actual": path.stat().st_size,
            }
        )
        return None
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        issues.append({"severity": "error", "code": f"artifact.{label}.invalid_json", "message": str(exc)})
        return None
    if not isinstance(value, dict):
        issues.append({"severity": "error", "code": f"artifact.{label}.not_object", "message": f"{label} must be a JSON object"})
        return None
    return value


def verify_declared_hash(
    path: Path | None,
    expected: Any,
    label: str,
    issues: list[dict[str, Any]],
) -> str | None:
    if path is None:
        return None
    if not path.is_file():
        issues.append({"severity": "error", "code": f"artifact.{label}.missing", "message": f"missing {label}", "actual": str(path)})
        return None
    try:
        actual = file_sha256(path)
    except OSError as exc:
        issues.append({"severity": "error", "code": f"artifact.{label}.unreadable", "message": str(exc)})
        return None
    if actual != expected:
        issues.append(
            {
                "severity": "error",
                "code": f"artifact.{label}.sha256",
                "message": f"{label} hash differs from run manifest",
                "expected": expected,
                "actual": actual,
            }
        )
    return actual


def require_stable_hash(
    path: Path | None,
    initial: str | None,
    label: str,
    issues: list[dict[str, Any]],
) -> None:
    if path is None or initial is None or not path.is_file():
        return
    try:
        final = file_sha256(path)
    except OSError as exc:
        issues.append({"severity": "error", "code": f"artifact.{label}.stability", "message": str(exc)})
        return
    if final != initial:
        issues.append(
            {
                "severity": "error",
                "code": f"artifact.{label}.changed_during_diagnosis",
                "message": f"{label} changed while it was being diagnosed",
                "expected": initial,
                "actual": final,
            }
        )


def read_metrics(path: Path | None, issues: list[dict[str, Any]]) -> list[dict[str, Any]]:
    if path is None or not path.is_file():
        if path is not None:
            issues.append({"severity": "error", "code": "artifact.metrics.missing", "message": "metrics file is missing", "actual": str(path)})
        return []
    if path.stat().st_size > MAX_METRICS_BYTES:
        issues.append(
            {
                "severity": "error",
                "code": "artifact.metrics.too_large",
                "message": "metrics file exceeds the diagnostic bound",
                "actual": path.stat().st_size,
            }
        )
        return []
    records: list[dict[str, Any]] = []
    try:
        with path.open("rb") as handle:
            for line_number, raw in enumerate(handle, start=1):
                if len(raw) > MAX_JSONL_LINE_BYTES:
                    issues.append(
                        {
                            "severity": "error",
                            "code": "metrics.line_too_large",
                            "message": "metrics line exceeds the diagnostic bound",
                            "line": line_number,
                        }
                    )
                    break
                if not raw.strip():
                    continue
                if len(records) >= MAX_METRIC_RECORDS:
                    issues.append(
                        {
                            "severity": "error",
                            "code": "metrics.too_many_records",
                            "message": "metrics record count exceeds the diagnostic bound",
                            "actual": len(records) + 1,
                        }
                    )
                    break
                try:
                    value = json.loads(raw)
                except (UnicodeError, json.JSONDecodeError) as exc:
                    issues.append(
                        {
                            "severity": "error",
                            "code": "metrics.invalid_jsonl",
                            "message": str(exc),
                            "line": line_number,
                        }
                    )
                    continue
                if not isinstance(value, dict):
                    issues.append(
                        {
                            "severity": "error",
                            "code": "metrics.not_object",
                            "message": "metrics record must be a JSON object",
                            "line": line_number,
                        }
                    )
                    continue
                records.append(value)
    except OSError as exc:
        issues.append({"severity": "error", "code": "artifact.metrics.unreadable", "message": str(exc)})
    return records


def validate_metrics(records: list[dict[str, Any]], issues: list[dict[str, Any]]) -> None:
    previous: dict[str, float] | None = None
    for ordinal, record in enumerate(records, start=1):
        for field in COUNTER_FIELDS + METRIC_FIELDS:
            value = record.get(field)
            if field == "early_stop_kl" and isinstance(value, bool):
                value = float(value)
            if not finite_number(value):
                issues.append(
                    {
                        "severity": "error",
                        "code": "metrics.nonfinite_or_missing",
                        "message": f"metric {field} is missing, non-numeric or non-finite",
                        "record": ordinal,
                        "actual": repr(record.get(field)),
                    }
                )
        if previous is not None:
            if finite_number(record.get("iteration")) and record["iteration"] <= previous["iteration"]:
                issues.append(
                    {
                        "severity": "error",
                        "code": "metrics.iteration_order",
                        "message": "iteration must increase strictly",
                        "record": ordinal,
                        "previous": previous["iteration"],
                        "actual": record["iteration"],
                    }
                )
            if finite_number(record.get("samples")) and record["samples"] <= previous["samples"]:
                issues.append(
                    {
                        "severity": "error",
                        "code": "metrics.sample_order",
                        "message": "samples must increase strictly",
                        "record": ordinal,
                        "previous": previous["samples"],
                        "actual": record["samples"],
                    }
                )
            if finite_number(record.get("optimizer_steps")) and record["optimizer_steps"] < previous["optimizer_steps"]:
                issues.append(
                    {
                        "severity": "error",
                        "code": "metrics.optimizer_step_order",
                        "message": "optimizer steps must not decrease",
                        "record": ordinal,
                        "previous": previous["optimizer_steps"],
                        "actual": record["optimizer_steps"],
                    }
                )
        if all(finite_number(record.get(field)) for field in COUNTER_FIELDS):
            previous = {field: float(record[field]) for field in COUNTER_FIELDS}


def mean_for(records: Iterable[dict[str, Any]], field: str) -> float | None:
    values = [float(record[field]) for record in records if finite_number(record.get(field))]
    return statistics.fmean(values) if values else None


def slope(records: list[dict[str, Any]], field: str) -> float | None:
    points = [
        (float(record["iteration"]), float(record[field]))
        for record in records
        if finite_number(record.get("iteration")) and finite_number(record.get(field))
    ]
    if len(points) < 2:
        return None
    x_mean = statistics.fmean(point[0] for point in points)
    y_mean = statistics.fmean(point[1] for point in points)
    denominator = sum((x - x_mean) ** 2 for x, _ in points)
    if denominator == 0.0:
        return 0.0
    return sum((x - x_mean) * (y - y_mean) for x, y in points) / denominator


def window_summary(records: list[dict[str, Any]], window: int) -> dict[str, Any]:
    early = records[:window]
    final = records[-window:]
    return {
        "record_count": len(records),
        "window": min(window, len(records)),
        "first_iteration": records[0].get("iteration") if records else None,
        "last_iteration": records[-1].get("iteration") if records else None,
        "early_mean": {field: mean_for(early, field) for field in METRIC_FIELDS},
        "final_mean": {field: mean_for(final, field) for field in METRIC_FIELDS},
        "slope_per_iteration": {field: slope(records, field) for field in METRIC_FIELDS},
    }


def alert(
    alerts: list[dict[str, Any]],
    severity: str,
    code: str,
    message: str,
    evidence: dict[str, Any],
    action: str,
) -> None:
    alerts.append(
        {
            "severity": severity,
            "code": code,
            "message": message,
            "evidence": evidence,
            "action": action,
        }
    )


def evaluation_summary(value: Any) -> dict[str, Any] | None:
    if not isinstance(value, dict):
        return None
    reasons = value.get("failure_reason_counts") if isinstance(value.get("failure_reason_counts"), dict) else {}
    return {
        "episode_matrix": value.get("episode_matrix"),
        "episodes": value.get("episodes"),
        "selection_episode_matrix_hash": value.get("selection_episode_matrix_hash"),
        "mean_episode_length": value.get("mean_episode_length"),
        "mean_return": value.get("mean_return"),
        "failure_count": value.get("failure_count"),
        "reference_complete_count": value.get("reference_complete_count"),
        "failure_reason_counts": reasons,
        "maximum_hard_rom_action_channel": value.get("maximum_hard_rom_action_channel"),
        "maximum_hard_rom_excess_microradians": value.get("maximum_hard_rom_excess_microradians"),
        "maximum_hard_rom_selection": value.get("maximum_hard_rom_selection"),
    }


def diagnose_evaluation(manifest: dict[str, Any], alerts: list[dict[str, Any]]) -> dict[str, Any]:
    initial_raw = manifest.get("initial_evaluation")
    final_raw = manifest.get("final_evaluation")
    initial = evaluation_summary(initial_raw)
    final = evaluation_summary(final_raw)
    if initial is None or final is None:
        alert(
            alerts,
            "high",
            "evaluation.missing",
            "closed run is missing initial or final evaluation",
            {"initial_present": initial is not None, "final_present": final is not None},
            "Re-run the declared evaluation closure in a new artifact directory.",
        )
        return {"initial": initial, "final": final}

    nonfinite = nonfinite_paths(final_raw, "$.final_evaluation")
    if nonfinite:
        alert(
            alerts,
            "critical",
            "evaluation.nonfinite",
            "final evaluation contains non-finite values",
            {"paths": nonfinite[:20], "truncated": len(nonfinite) > 20},
            "Stop promotion and isolate the first non-finite environment fact.",
        )

    same_matrix = (
        manifest.get("evaluation_selection_matrix_identical") is True
        and initial.get("selection_episode_matrix_hash") == final.get("selection_episode_matrix_hash")
        and initial.get("episode_matrix") == final.get("episode_matrix")
    )
    if not same_matrix:
        alert(
            alerts,
            "critical",
            "evaluation.matrix_mismatch",
            "initial and final evaluations are not the same declared selection matrix",
            {
                "manifest_flag": manifest.get("evaluation_selection_matrix_identical"),
                "initial_hash": initial.get("selection_episode_matrix_hash"),
                "final_hash": final.get("selection_episode_matrix_hash"),
            },
            "Repeat both evaluations with one frozen matrix before comparing policy quality.",
        )

    reasons = final["failure_reason_counts"]
    for reason in ("non_finite", "hard_rom", "forbidden_contact"):
        count = reasons.get(reason, 0)
        if isinstance(count, int) and count > 0:
            detail = {
                "reason": reason,
                "count": count,
                "episodes": final.get("episodes"),
                "maximum_hard_rom_action_channel": final.get("maximum_hard_rom_action_channel"),
                "maximum_hard_rom_excess_microradians": final.get("maximum_hard_rom_excess_microradians"),
                "maximum_hard_rom_selection": final.get("maximum_hard_rom_selection"),
            }
            alert(
                alerts,
                "critical",
                f"safety.{reason}",
                f"final evaluation contains {reason} failures",
                detail,
                "Run the smallest phase/action-channel safety diagnostic before changing PPO.",
            )

    tracking_lost = reasons.get("reference_tracking_lost", 0)
    if isinstance(tracking_lost, int) and tracking_lost > 0:
        alert(
            alerts,
            "high",
            "environment.reference_tracking_lost",
            "final evaluation loses reference tracking",
            {"count": tracking_lost, "episodes": final.get("episodes")},
            "Stratify failures by clip and start phase under the unchanged final checkpoint.",
        )

    initial_complete = initial.get("reference_complete_count")
    final_complete = final.get("reference_complete_count")
    initial_length = initial.get("mean_episode_length")
    final_length = final.get("mean_episode_length")
    if all(finite_number(value) for value in (initial_complete, final_complete, initial_length, final_length)):
        if not (final_complete > initial_complete and final_length > initial_length):
            alert(
                alerts,
                "high",
                "evaluation.no_joint_improvement",
                "final checkpoint does not improve both frozen acceptance measures",
                {
                    "initial_reference_complete": initial_complete,
                    "final_reference_complete": final_complete,
                    "initial_mean_episode_length": initial_length,
                    "final_mean_episode_length": final_length,
                },
                "Keep the matrix fixed and diagnose the failing phase distribution before another run.",
            )
    return {"initial": initial, "final": final}


def diagnose_optimizer(
    summary: dict[str, Any],
    records: list[dict[str, Any]],
    training: dict[str, Any] | None,
    alerts: list[dict[str, Any]],
) -> None:
    if not records:
        return
    ppo = training.get("ppo") if training and isinstance(training.get("ppo"), dict) else {}
    distribution = (
        training.get("policy_distribution")
        if training and isinstance(training.get("policy_distribution"), dict)
        else {}
    )
    target_kl = ppo.get("target_approximate_kl")
    max_grad = ppo.get("maximum_gradient_norm")
    clip_ratio = ppo.get("clip_ratio")
    final = summary["final_mean"]

    early_stop_rate = final.get("early_stop_kl")
    if finite_number(early_stop_rate) and early_stop_rate >= 0.5:
        alert(
            alerts,
            "high",
            "optimization.kl_early_stop_pressure",
            "KL early stopping dominates the final diagnostic window",
            {"final_window_rate": early_stop_rate, "target_approximate_kl": target_kl},
            "After safety is clean, test one new frozen profile changing only learning rate.",
        )

    approximate_kl = final.get("approximate_kl")
    if finite_number(target_kl) and finite_number(approximate_kl) and approximate_kl > 1.5 * target_kl:
        alert(
            alerts,
            "high",
            "optimization.kl_above_target",
            "final-window approximate KL materially exceeds the frozen target",
            {"final_window_mean": approximate_kl, "target": target_kl},
            "After higher-priority blockers, test a learning-rate-only profile revision.",
        )

    fraction_above_grad: float | None = None
    if finite_number(max_grad):
        grad_values = [float(record["gradient_norm"]) for record in records if finite_number(record.get("gradient_norm"))]
        if grad_values:
            fraction_above_grad = sum(value > float(max_grad) for value in grad_values) / len(grad_values)
    if fraction_above_grad is not None and fraction_above_grad >= 0.5:
        alert(
            alerts,
            "medium",
            "optimization.gradient_clipping_pressure",
            "pre-clip gradient norm exceeds the frozen maximum in most updates",
            {"fraction_above_maximum": fraction_above_grad, "maximum_gradient_norm": max_grad},
            "Inspect reward/value scales; if safety is clean, change one optimizer variable in a new profile.",
        )

    clip_fraction = final.get("clip_fraction")
    if finite_number(clip_fraction) and finite_number(clip_ratio) and clip_fraction > max(0.3, float(clip_ratio) * 1.5):
        alert(
            alerts,
            "medium",
            "optimization.clip_fraction_high",
            "many final-window samples lie outside the PPO clipping interval",
            {"final_window_mean": clip_fraction, "clip_ratio": clip_ratio},
            "Pair with KL evidence before testing a learning-rate-only profile revision.",
        )

    min_log_std = distribution.get("minimum_log_standard_deviation")
    action_std = final.get("action_standard_deviation_mean")
    if finite_number(min_log_std) and finite_number(action_std):
        floor = math.exp(float(min_log_std))
        if action_std <= floor * 1.5:
            alert(
                alerts,
                "high",
                "optimization.action_std_collapse",
                "policy action standard deviation is near its frozen floor",
                {"final_window_mean": action_std, "floor": floor},
                "Check phase coverage before changing the distribution floor in a new profile.",
            )

    early_value = summary["early_mean"].get("value_loss")
    final_value = final.get("value_loss")
    if finite_number(early_value) and finite_number(final_value) and final_value > max(1.0, early_value * 1.25):
        alert(
            alerts,
            "medium",
            "optimization.value_loss_growth",
            "value loss grows materially from the early to final window",
            {"early_window_mean": early_value, "final_window_mean": final_value},
            "Validate return scale and truncation bootstrap before changing critic capacity.",
        )


def choose_primary(valid: bool, alerts: list[dict[str, Any]]) -> tuple[str, str]:
    if not valid:
        return "Artifact", "Regenerate a new hash-closed run; do not repair the existing artifacts in place."
    codes = {item["code"] for item in alerts if item["severity"] in {"critical", "high"}}
    if any(code.startswith("safety.") for code in codes):
        return "Safety", "Hold PPO fixed and isolate the first failing phase and action channel."
    if "evaluation.matrix_mismatch" in codes or "evaluation.missing" in codes:
        return "Evaluation", "Recreate the unchanged deterministic evaluation matrix before comparison."
    if any(code.startswith("environment.") for code in codes):
        return "Environment", "Stratify the unchanged checkpoint by clip and start phase."
    if any(code.startswith("optimization.") for code in codes):
        return "Optimization", "Create one frozen profile revision changing only the best-supported optimizer variable."
    return "Evaluation", "No high-severity signal found; extend only the predeclared evaluation matrix before tuning."


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run_directory", type=Path)
    parser.add_argument("--training-profile", type=Path)
    parser.add_argument("--window", type=int, default=20)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if not 1 <= args.window <= 5000:
        raise SystemExit("--window must be between 1 and 5000")

    run_dir = args.run_directory.resolve()
    issues: list[dict[str, Any]] = []
    alerts: list[dict[str, Any]] = []
    manifest_path = run_dir / "run-manifest.json"
    manifest_file_hash: str | None = None
    if manifest_path.is_file():
        try:
            manifest_file_hash = file_sha256(manifest_path)
        except OSError as exc:
            issues.append({"severity": "error", "code": "artifact.run_manifest.unreadable", "message": str(exc)})
    manifest = load_json_object(manifest_path, "run_manifest", issues)
    if manifest is None:
        result = {
            "schema": "nextengine.skill.training-diagnostic.v1",
            "valid": False,
            "run_directory": str(run_dir),
            "integrity_issues": issues,
            "alerts": alerts,
            "primary_diagnosis": {
                "class": "Artifact",
                "next_experiment": "Regenerate a new hash-closed run; do not repair this directory in place.",
            },
        }
        json.dump(result, sys.stdout, indent=2, sort_keys=True)
        sys.stdout.write("\n")
        return 2

    if manifest.get("schema_version") != 1 or manifest.get("schema_id") != "nextengine.training.humanoid-reference-overfit-run.v1":
        issues.append(
            {
                "severity": "error",
                "code": "artifact.run_manifest.schema",
                "message": "unsupported reference run manifest schema",
                "actual": {"schema_id": manifest.get("schema_id"), "schema_version": manifest.get("schema_version")},
            }
        )

    metrics_binding = manifest.get("metrics") if isinstance(manifest.get("metrics"), dict) else {}
    metrics_path = relative_artifact(run_dir, metrics_binding.get("file"), "metrics", issues)
    metrics_hash = verify_declared_hash(metrics_path, metrics_binding.get("sha256"), "metrics", issues)
    records = read_metrics(metrics_path, issues)
    validate_metrics(records, issues)
    if metrics_binding.get("record_count") != len(records):
        issues.append(
            {
                "severity": "error",
                "code": "artifact.metrics.record_count",
                "message": "metrics record count differs from run manifest",
                "expected": metrics_binding.get("record_count"),
                "actual": len(records),
            }
        )

    checkpoint_binding = manifest.get("checkpoint") if isinstance(manifest.get("checkpoint"), dict) else {}
    checkpoint_path = relative_artifact(run_dir, checkpoint_binding.get("file"), "checkpoint", issues)
    checkpoint_hash = verify_declared_hash(checkpoint_path, checkpoint_binding.get("sha256"), "checkpoint", issues)

    throughput_binding = manifest.get("throughput_report")
    throughput_path: Path | None = None
    throughput_hash: str | None = None
    if isinstance(throughput_binding, dict):
        throughput_path = relative_artifact(run_dir, throughput_binding.get("file"), "throughput_report", issues)
        throughput_hash = verify_declared_hash(
            throughput_path,
            throughput_binding.get("sha256"),
            "throughput_report",
            issues,
        )

    training: dict[str, Any] | None = None
    training_path: Path | None = None
    training_hash: str | None = None
    if args.training_profile:
        training_path = args.training_profile.resolve()
        training = load_json_object(training_path, "training_profile", issues)
        if training is not None:
            training_hash = file_sha256(training_path)
            expected = manifest.get("training_profile_sha256")
            if training_hash != expected:
                issues.append(
                    {
                        "severity": "error",
                        "code": "artifact.training_profile.sha256",
                        "message": "supplied training profile differs from run manifest",
                        "expected": expected,
                        "actual": training_hash,
                    }
                )

    summary = window_summary(records, args.window)
    if records and isinstance(manifest.get("final_metrics"), dict) and manifest["final_metrics"] != records[-1]:
        issues.append(
            {
                "severity": "error",
                "code": "artifact.final_metrics",
                "message": "run manifest final_metrics differs from the final JSONL record",
            }
        )

    status = manifest.get("status")
    if status != "completed":
        alert(
            alerts,
            "high",
            "run.not_completed",
            "run status is not completed",
            {"status": status},
            "Diagnose the stable failure prefix; do not retry unchanged inputs.",
        )

    evaluations = diagnose_evaluation(manifest, alerts)
    diagnose_optimizer(summary, records, training, alerts)

    for label, path, digest in (
        ("run_manifest", manifest_path, manifest_file_hash),
        ("metrics", metrics_path, metrics_hash),
        ("checkpoint", checkpoint_path, checkpoint_hash),
        ("throughput_report", throughput_path, throughput_hash),
        ("training_profile", training_path, training_hash),
    ):
        require_stable_hash(path, digest, label, issues)

    valid = not any(item["severity"] == "error" for item in issues)
    primary_class, next_experiment = choose_primary(valid, alerts)
    result = {
        "schema": "nextengine.skill.training-diagnostic.v1",
        "valid": valid,
        "run_directory": str(run_dir),
        "run_identity": {
            "run_id": manifest.get("run_id"),
            "status": status,
            "generation_id": manifest.get("training_generation_id"),
            "generation_manifest_hash": manifest.get("training_generation_manifest_hash"),
            "repository": manifest.get("repository"),
            "training_profile_sha256": manifest.get("training_profile_sha256"),
            "run_manifest_file_sha256": manifest_file_hash,
            "metrics_sha256": metrics_hash,
            "checkpoint_sha256": checkpoint_hash,
            "throughput_report_sha256": throughput_hash,
        },
        "metrics_summary": summary,
        "evaluation_summary": evaluations,
        "integrity_issues": issues,
        "alerts": alerts,
        "primary_diagnosis": {"class": primary_class, "next_experiment": next_experiment},
        "claim": {
            "declared": manifest.get("claim"),
            "overfit_acceptance": manifest.get("overfit_acceptance"),
            "learned_policy_claim": manifest.get("learned_policy_claim"),
            "ceiling": "DeclaredRunStageOnly; no CPU/Isaac parity, runtime parity, TRAIN advancement, or R5 closure",
        },
        "analysis_mode": "read-only; dashboards and notebooks are non-authoritative projections",
    }
    json.dump(result, sys.stdout, indent=2, sort_keys=True, allow_nan=False)
    sys.stdout.write("\n")
    return 0 if valid else 2


if __name__ == "__main__":
    raise SystemExit(main())
