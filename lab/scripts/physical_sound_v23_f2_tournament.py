#!/usr/bin/env python3
"""Official V23 F2 fixed-feature ridge tournament and evaluator adapter."""

from __future__ import annotations

import argparse
import inspect
import resource
import shutil
import sys
import time
from collections.abc import Callable
from pathlib import Path
from typing import Any

import physical_sound_v21_f1_tournament as evaluator
import physical_sound_v23_f2_common as common
import physical_sound_v23_f2_model as model

METHOD_ORDER = evaluator.METHOD_ORDER
MUTATION_ORDER = evaluator.MUTATION_ORDER
STRUCTURAL_ORDER = evaluator.STRUCTURAL_ORDER
OUTPUT_FILES = evaluator.OUTPUT_FILES

REQUIRED_SIGNATURES = {
    "candidate_specs": "() -> 'tuple[CandidateSpec, ...]'",
    "compatible_predictions": "(trained: 'FixedFeatureModel', gain_scale: 'np.ndarray', item: 'common.FieldObject', spec: 'CandidateSpec') -> 'dict[str, np.ndarray]'",
    "decode_models": "(payload: 'bytes') -> 'tuple[tuple[FixedFeatureModel, ...], FixedFeatureModel, np.ndarray]'",
    "encode_models": "(candidates: 'tuple[TrainingResult, ...]', control: 'TrainingResult') -> 'bytes'",
    "fit_candidates": "(train: 'tuple[common.FieldObject, ...]') -> 'tuple[TrainingResult, ...]'",
    "parameter_bytes": "(trained: 'FixedFeatureModel') -> 'int'",
    "predict_candidate": "(trained: 'FixedFeatureModel', gain_scale: 'np.ndarray', item: 'common.FieldObject', spec: 'CandidateSpec', observed_context: 'np.ndarray | None' = None) -> 'np.ndarray'",
    "predict_direct_probes": "(trained: 'FixedFeatureModel', gain_scale: 'np.ndarray', item: 'common.FieldObject', spec: 'CandidateSpec') -> 'np.ndarray'",
    "validate_request": "(item: 'common.FieldObject', spec: 'CandidateSpec', *, expected_role: 'str', declared_role: 'str | None' = None, declared_basis_revision: 'str' = 'topology-native-continuous-tensor-v1', declared_regularization: 'float | None' = None, coefficient: 'np.ndarray | None' = None, available_model_bytes: 'int' = 131072) -> 'None'",
}


def api_surface_report() -> dict[str, Any]:
    if tuple(sorted(model.REQUIRED_MODEL_API)) != tuple(sorted(REQUIRED_SIGNATURES)):
        raise common.F1Error("F2 required API member set changed")
    observed = {}
    for name, expected in sorted(REQUIRED_SIGNATURES.items()):
        value = getattr(model, name, None)
        if not callable(value):
            raise common.F1Error(f"F2 required API missing: {name}")
        signature = str(inspect.signature(value))
        if signature != expected:
            raise common.F1Error(
                f"F2 required API signature changed for {name}: {signature}"
            )
        observed[name] = signature
    return {"members": observed, "passed": True}


def _selection_key(
    result: model.TrainingResult,
    metrics: dict[str, list[dict[str, Any]]],
    remesh: list[dict[str, Any]],
) -> tuple[Any, ...]:
    if result.spec is None:
        raise common.F1Error("F2 selection candidate identity changed")
    summary = evaluator._summary(metrics["candidate"])
    topology_gradient = evaluator._topology_means(
        metrics["candidate"], "edge_gradient_p99"
    )
    return (
        max(row["gain_metric_drift"] for row in remesh),
        max(row["probe_disagreement_nrmse"] for row in remesh),
        max(float(value) for value in topology_gradient.values() if value is not None),
        summary["gain_nrmse_mean"],
        summary["edge_gradient_p99_mean"],
        result.spec.dimension,
        result.spec.bandwidth,
        result.spec.order,
        result.spec.candidate_id,
    )


def _invoke(function: Callable[..., dict[str, Any]], *arguments: Any) -> dict[str, Any]:
    previous_common = evaluator.common
    previous_model = evaluator.model
    previous_selection = evaluator._selection_key
    evaluator.common = common
    evaluator.model = model
    evaluator._selection_key = _selection_key
    try:
        return function(*arguments)
    finally:
        evaluator.common = previous_common
        evaluator.model = previous_model
        evaluator._selection_key = previous_selection


def _validated_target(output: Path) -> Path:
    staging, target = common.prepare_output(output)
    common.abandon_output(staging)
    return target


def run(output: Path, f0_root: Path) -> dict[str, Any]:
    started = time.monotonic()
    common.verify_protocol_environment()
    api_surface_report()
    target = _validated_target(output)
    inner = target.with_name(f".{target.name}.f2-inner")
    if inner.exists() or inner.is_symlink():
        raise common.F1Error("F2 inner publication target already exists")
    try:
        result = _invoke(evaluator.run, inner, f0_root)
        elapsed = time.monotonic() - started
        rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        if elapsed >= 300.0:
            raise common.F1Error("F2 runtime ceiling exceeded")
        if rss > 2 * 1024 * 1024:
            raise common.F1Error("F2 RSS ceiling exceeded")
        inner.rename(target)
        result["output"] = str(target)
        result["tree_digest"] = common.tree_digest(common.directory_file_map(target))
        return result
    except Exception:
        if inner.is_dir() and not inner.is_symlink():
            shutil.rmtree(inner)
        raise


def compare(left: Path, right: Path) -> dict[str, Any]:
    return _invoke(evaluator.compare, left, right)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("--output", type=Path, required=True)
    run_parser.add_argument("--f0-root", type=Path, required=True)
    compare_parser = subparsers.add_parser("compare")
    compare_parser.add_argument("--left", type=Path, required=True)
    compare_parser.add_argument("--right", type=Path, required=True)
    return parser


def main() -> int:
    arguments = _parser().parse_args()
    try:
        if arguments.command == "run":
            result = run(arguments.output, arguments.f0_root)
        else:
            result = compare(arguments.left, arguments.right)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0
    except (AttributeError, common.F1Error, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
