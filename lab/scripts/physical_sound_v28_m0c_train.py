#!/usr/bin/env python3
"""Owning deterministic full-entry runner for the V28 M0c neural student."""

from __future__ import annotations

import argparse
import os
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Any

import mlflow
import physical_sound_v25_m0a_common as base
import physical_sound_v25_m0a_train as inherited_train
import physical_sound_v26_m0b_train as inherited_owner
import physical_sound_v28_m0c_common as contract
import physical_sound_v28_m0c_model as model_lib

REPORT_SCHEMA = "nextengine.experimental-physical-sound-v28-m0c.report.v1"
CONTROL_SCHEMA = "nextengine.experimental-physical-sound-v28-m0c.controls.v1"
FREEZE_SCHEMA = "nextengine.experimental-physical-sound-v28-m0c.freeze.v1"
HOLDOUT_SCHEMA = "nextengine.experimental-physical-sound-v28-m0c.method-holdout.v1"
REAL_SCHEMA = "nextengine.experimental-physical-sound-v28-m0c.disclosed-real.v1"


def _mlflow_log(
    root: Path,
    manifest: dict[str, Any],
    report: dict[str, Any],
    step_metrics: tuple[dict[str, float], ...],
    artifact_hashes: dict[str, str],
) -> dict[str, str]:
    root.mkdir(parents=True, exist_ok=True)
    tracking_uri = root.resolve(strict=True).as_uri()
    if not tracking_uri.startswith("file://"):
        raise base.M0Error("M0c MLflow tracking URI must be local file://")
    os.environ["MLFLOW_ALLOW_FILE_STORE"] = "true"
    mlflow.autolog(disable=True)
    mlflow.set_tracking_uri(tracking_uri)
    mlflow.set_experiment("nextengine-physical-sound-v28-m0c")
    with mlflow.start_run(run_name=contract.EXPERIMENT_ID) as active:
        mlflow.log_params(
            {
                "model_id": contract.MODEL_ID,
                "profile": manifest["profile"],
                "protocol_sha256": contract.PROTOCOL_SHA256,
                "implementation_root_sha256": manifest["implementation_root_sha256"],
                "seed": model_lib.SEED,
                "parameter_count": report["parameter_count"],
                "training_render_frames": model_lib.TRAINING_RENDER_FRAMES,
                "cpu_only": True,
            }
        )
        for item in step_metrics:
            mlflow.log_metric("loss", item["loss"], step=int(item["step"]))
        for name, value in sorted(artifact_hashes.items()):
            mlflow.log_param(f"sha256_{name.replace('.', '_')}", value)
        return {"tracking_uri": tracking_uri, "run_id": active.info.run_id}


@contextmanager
def _bound_owner() -> Iterator[None]:
    bindings = {
        "contract": contract,
        "model_lib": model_lib,
        "REPORT_SCHEMA": REPORT_SCHEMA,
        "CONTROL_SCHEMA": CONTROL_SCHEMA,
        "FREEZE_SCHEMA": FREEZE_SCHEMA,
        "HOLDOUT_SCHEMA": HOLDOUT_SCHEMA,
        "REAL_SCHEMA": REAL_SCHEMA,
        "_mlflow_log": _mlflow_log,
    }
    saved_owner = {name: getattr(inherited_owner, name) for name in bindings}
    saved_train_model = inherited_train.model_lib
    try:
        for name, value in bindings.items():
            setattr(inherited_owner, name, value)
        inherited_train.model_lib = model_lib
        yield
    finally:
        inherited_train.model_lib = saved_train_model
        for name, value in saved_owner.items():
            setattr(inherited_owner, name, value)


def run(manifest_path: Path, output_path: Path) -> dict[str, Any]:
    with _bound_owner():
        return inherited_owner.run(manifest_path, output_path)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    arguments = parser.parse_args()
    try:
        report = run(arguments.manifest, arguments.output)
    except (base.M0Error, OSError, ValueError) as error:
        print(f"physical-sound-v28-m0c: {error}", file=os.sys.stderr)
        return 2
    print(base.canonical_json(report).decode("utf-8"), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
