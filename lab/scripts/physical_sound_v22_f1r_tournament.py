#!/usr/bin/env python3
"""Official resource-bounded V22 F1r tournament entry point."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any

import physical_sound_v21_f1_tournament as predecessor
import physical_sound_v22_f1r_common as common
import physical_sound_v22_f1r_model as model

METHOD_ORDER = predecessor.METHOD_ORDER
MUTATION_ORDER = predecessor.MUTATION_ORDER
STRUCTURAL_ORDER = predecessor.STRUCTURAL_ORDER
OUTPUT_FILES = predecessor.OUTPUT_FILES


def _invoke(function: Any, *arguments: Any) -> dict[str, Any]:
    """Use the frozen evaluator without leaking module state to other tests."""

    previous_common = predecessor.common
    previous_model = predecessor.model
    predecessor.common = common
    predecessor.model = model
    try:
        return function(*arguments)
    finally:
        predecessor.common = previous_common
        predecessor.model = previous_model


def run(output: Path, f0_root: Path) -> dict[str, Any]:
    common.verify_protocol_environment()
    return _invoke(predecessor.run, output, f0_root)


def compare(left: Path, right: Path) -> dict[str, Any]:
    return _invoke(predecessor.compare, left, right)


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
    except (common.F1Error, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
