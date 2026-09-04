#!/usr/bin/env python3
"""Validate and activate one exact Isaac training input closure."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from next_lab.isaac_training import (
    IsaacTrainingProfile,
    activate_training_generation,
    require_external_path,
    sha256_file,
)
from next_lab.motor_mirror import (
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2,
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
    validate_biomechanics_forward_start_stop_descriptor,
    validate_biomechanics_standing_descriptor,
    validate_descriptor,
)
from next_lab.usd_translation import render_usda


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generation-index", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    arguments = parse_args()
    generation_index = require_external_path(
        arguments.generation_index,
        REPOSITORY_ROOT,
        label="active training generation",
    )
    descriptor_path = require_external_path(
        arguments.descriptor,
        REPOSITORY_ROOT,
        label="engine descriptor",
    )
    usd_path = require_external_path(
        arguments.usd,
        REPOSITORY_ROOT,
        label="derived humanoid USD",
    )
    profile = IsaacTrainingProfile.load(arguments.profile.resolve())
    descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
    if "training_descriptor_id" in descriptor:
        if profile.environment_profile_id in {
            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2,
            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
            BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
        }:
            validate_biomechanics_forward_start_stop_descriptor(descriptor)
        else:
            validate_biomechanics_standing_descriptor(descriptor)
    else:
        validate_descriptor(descriptor)
    profile_ids = {
        item.get("profile_id") for item in descriptor["environment_profiles"]
    }
    if profile.environment_profile_id not in profile_ids:
        raise ValueError("training environment is absent from the descriptor")
    expected_usd_hash = hashlib.sha256(
        render_usda(descriptor).encode("utf-8")
    ).hexdigest()
    actual_usd_hash = sha256_file(usd_path)
    if actual_usd_hash != expected_usd_hash:
        raise ValueError("USD bytes do not match the selected descriptor translation")

    activated = activate_training_generation(
        index_path=generation_index,
        profile=profile,
        descriptor_sha256=sha256_file(descriptor_path),
        usd_sha256=actual_usd_hash,
    )
    print(
        json.dumps(
            {
                "status": activated.manifest.status,
                "training_generation_id": activated.manifest.generation_id,
                "generation_manifest": str(activated.manifest_path),
                "generation_manifest_hash": activated.manifest.manifest_hash,
                "profile_id": profile.profile_id,
                "profile_hash": profile.profile_hash,
                "environment_profile_id": profile.environment_profile_id,
                "descriptor_sha256": sha256_file(descriptor_path),
                "usd_sha256": actual_usd_hash,
            },
            indent=2,
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
