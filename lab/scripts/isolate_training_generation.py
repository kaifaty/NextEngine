#!/usr/bin/env python3
"""Create one non-destructive active generation and retire exact old roots."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from next_lab.isaac_training import initialize_training_generation


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--training-store", type=Path, required=True)
    parser.add_argument("--generation-directory", required=True)
    parser.add_argument("--generation-id", required=True)
    parser.add_argument("--candidate-id", required=True)
    parser.add_argument("--requirements-baseline", type=Path, required=True)
    parser.add_argument("--retired-generation-id", required=True)
    parser.add_argument(
        "--retired-root",
        type=Path,
        action="append",
        required=True,
        help="Exact old artifact root to inventory; repeat for each root.",
    )
    return parser.parse_args()


def main() -> None:
    arguments = parse_args()
    result = initialize_training_generation(
        training_store=arguments.training_store,
        repository_root=REPOSITORY_ROOT,
        generation_directory=arguments.generation_directory,
        generation_id=arguments.generation_id,
        candidate_id=arguments.candidate_id,
        requirements_baseline=arguments.requirements_baseline,
        retired_generation_id=arguments.retired_generation_id,
        retired_roots=arguments.retired_root,
    )
    print(
        json.dumps(
            {key: str(value) for key, value in result.items()},
            indent=2,
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
