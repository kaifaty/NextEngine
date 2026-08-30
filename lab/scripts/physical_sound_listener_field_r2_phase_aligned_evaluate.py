#!/usr/bin/env python3
"""Evaluate the preregistered phase-aligned R2 successor."""

from __future__ import annotations

import physical_sound_listener_field_r2_evaluate as evaluator

evaluator.MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "evaluation.manifest.v1"
)
evaluator.TRAINING_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "training.report.v1"
)
evaluator.MLFLOW_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "mlflow-lineage.v1"
)
evaluator.REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-listener-field-r2-phase-aligned-"
    "evaluation.report.v1"
)
evaluator.REVISION = (
    "green-goblet-fixed-impact-listener-field-r2-phase-aligned-evaluation-v1"
)
evaluator.TRAINING_REVISION = (
    "green-goblet-fixed-impact-listener-field-r2-phase-aligned-v1"
)


def main() -> None:
    evaluator.__file__ = __file__
    evaluator.main()


if __name__ == "__main__":
    main()
