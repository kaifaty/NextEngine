from __future__ import annotations

import hashlib
import io
import json
import os
import subprocess
import tempfile
import zipfile
from copy import deepcopy
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.cmu_motion import AsfSkeleton, AmcFrame, parse_amc, parse_asf
from next_lab.motion_preview import render_clip_preview
from next_lab.motion_math import quaternion_to_matrix
from next_lab.motion_retarget import (
    CONTACT_IDS,
    RetargetedClip,
    canonical_integer_arrays,
    mirror_clip,
    retarget_clip,
    rotate_clip_quarter_yaw,
)


SCHEMA_VERSION = 1
CORPUS_MANIFEST_ID = "nextengine.private-motion-corpus-manifest.v1"
BASE_MOTION_CORPUS_PROFILE_ID = (
    "nextengine.motion-corpus.humanoid-biomechanics-cmu-locomotion.v1"
)
BASE_MOTION_CORPUS_PROFILE_SHA256 = (
    "f281f73773f32ddba506c01aa66301488dadc40d91efbd78e2c1fb79a70951fc"
)
STANCE_CHAIN_ALGORITHM_IDS = frozenset(
    {
        "nextengine.cmu-stance-chain-retarget.v2",
        "nextengine.cmu-stance-chain-retarget.v3",
        "nextengine.cmu-stance-chain-retarget.v4",
        "nextengine.cmu-stance-chain-retarget.v5",
        "nextengine.cmu-stance-chain-retarget.v6",
        "nextengine.cmu-stance-chain-retarget.v7",
        "nextengine.cmu-stance-chain-retarget.v8",
    }
)


def build_motion_corpus(
    *,
    profile_path: Path,
    descriptor_path: Path,
    dataset_root: Path,
    output_store: Path,
) -> tuple[dict[str, Any], Path]:
    profile, profile_bytes = load_motion_corpus_profile(profile_path)
    descriptor_bytes = descriptor_path.read_bytes()
    descriptor = json.loads(descriptor_bytes)
    _validate_closure(profile, descriptor, descriptor_bytes, dataset_root)
    profile_sha256 = _sha256(profile_bytes)
    descriptor_sha256 = _sha256(descriptor_bytes)
    tool_sha256 = _tool_sha256()
    destination = output_store.resolve() / "corpus" / f"{profile_sha256[:16]}-{tool_sha256[:16]}"
    destination.parent.mkdir(parents=True, exist_ok=True)

    skeleton_cache: dict[str, AsfSkeleton] = {}
    frame_cache: dict[tuple[str, str], tuple[AmcFrame, ...]] = {}
    clips: list[RetargetedClip] = []
    admitted_partitions = set(profile["admitted_partitions"])
    expected_scale = profile["source"]["length_scale_metres"]
    scale = float(expected_scale["numerator"]) / float(expected_scale["denominator"])
    raw_root = dataset_root.resolve() / "raw"
    admitted_clip_records = [
        clip_record
        for clip_record in profile["clips"]
        if clip_record["partition"] in admitted_partitions
    ]
    for clip_record in admitted_clip_records:
        subject = clip_record["subject"]
        trial = clip_record["trial"]
        skeleton = skeleton_cache.get(subject)
        if skeleton is None:
            skeleton = parse_asf(raw_root / subject / f"{subject}.asf", expected_length_scale_metres=scale)
            skeleton_cache[subject] = skeleton
        frames = frame_cache.get((subject, trial))
        if frames is None:
            frames = parse_amc(raw_root / subject / f"{subject}_{trial}.amc")
            frame_cache[(subject, trial)] = frames
        retargeted = retarget_clip(
            clip=clip_record,
            skeleton=skeleton,
            frames=frames,
            descriptor=descriptor,
            profile=profile,
        )
        clips.append(retargeted)
        if mirror_id := clip_record.get("derive_mirror_as"):
            clips.append(mirror_clip(retargeted, descriptor, mirror_id))
        for variant in clip_record.get("derive_yaw_variants", []):
            clips.append(
                rotate_clip_quarter_yaw(
                    retargeted,
                    clip_id=variant["clip_id"],
                    skill=variant["skill"],
                    quarter_turns=int(variant["quarter_turns"]),
                )
            )

    split_audit = _split_audit(clips)
    if split_audit["errors"]:
        raise ValueError("motion corpus split closure is invalid: " + "; ".join(split_audit["errors"]))
    coverage_audit = _coverage_audit(clips, profile)

    staging = Path(tempfile.mkdtemp(prefix="nextengine-motion-corpus-", dir=destination.parent))
    try:
        clip_directory = staging / "clips"
        preview_directory = staging / "previews"
        clip_directory.mkdir()
        preview_directory.mkdir()
        clip_results = []
        for clip in clips:
            metadata = _clip_metadata(clip, descriptor, profile_sha256, descriptor_sha256, tool_sha256)
            artifact_bytes = deterministic_npz_bytes(clip, metadata)
            if artifact_bytes != deterministic_npz_bytes(clip, metadata):
                raise RuntimeError(f"non-deterministic serialization for {clip.clip_id}")
            artifact_path = clip_directory / f"{clip.clip_id}.npz"
            artifact_path.write_bytes(artifact_bytes)
            preview_bytes = render_clip_preview(clip, descriptor).encode("utf-8")
            preview_path = preview_directory / f"{clip.clip_id}.svg"
            preview_path.write_bytes(preview_bytes)
            validation = validate_clip(clip, descriptor, profile)
            clip_results.append(
                {
                    "clip_id": clip.clip_id,
                    "source_clip_id": clip.source_clip_id,
                    "mirrored_from": clip.mirrored_from,
                    "derived_from": clip.derived_from,
                    "derivation": clip.derivation,
                    "coverage_class": clip.coverage_class,
                    "skill": clip.skill,
                    "partition": clip.partition,
                    "split": clip.split,
                    "split_group_id": clip.split_group_id,
                    "source_frame_first": int(clip.source_frames[0]),
                    "source_frame_last": int(clip.source_frames[-1]),
                    "reference_frame_count": len(clip.source_frames),
                    "artifact": {
                        "relative_path": str(artifact_path.relative_to(staging)),
                        "sha256": _sha256(artifact_bytes),
                        "bytes": len(artifact_bytes),
                    },
                    "preview": {
                        "relative_path": str(preview_path.relative_to(staging)),
                        "sha256": _sha256(preview_bytes),
                        "bytes": len(preview_bytes),
                        "review_disposition": "pending",
                    },
                    "validation": validation,
                }
            )

        mirror_results = _validate_mirrors(clips, descriptor)
        all_pass = all(result["status"] == "PASS" for result in (entry["validation"] for entry in clip_results))
        all_pass = all_pass and all(result["status"] == "PASS" for result in mirror_results)
        all_pass = all_pass and coverage_audit["status"] == "PASS"
        manifest: dict[str, Any] = {
            "schema_version": SCHEMA_VERSION,
            "manifest_id": CORPUS_MANIFEST_ID,
            "status": "VALIDATED" if all_pass else "FAILED_VALIDATION",
            "profile": {
                "id": profile["profile_id"],
                "sha256": profile_sha256,
            },
            "admitted_partitions": sorted(admitted_partitions),
            "target": {
                "body_schema_id": descriptor["body_schema_id"],
                "body_schema_revision": descriptor["body_schema_revision"],
                "body_schema_hash": descriptor["body_schema_hash"],
                "compiled_descriptor_hash": descriptor["compiled_descriptor_hash"],
                "descriptor_file_sha256": descriptor_sha256,
            },
            "tool_sha256": tool_sha256,
            "source": profile["source"],
            "source_pages": profile["source_pages"],
            "source_files": profile["source_files"],
            "split_audit": split_audit,
            "coverage_audit": coverage_audit,
            "mirror_checks": mirror_results,
            "clips": clip_results,
            "summary": {
                "base_clip_count": len(admitted_clip_records),
                "derived_clip_count": len(clips) - len(admitted_clip_records),
                "total_clip_count": len(clips),
                "validated_clip_count": sum(entry["validation"]["status"] == "PASS" for entry in clip_results),
                "failed_clip_count": sum(entry["validation"]["status"] != "PASS" for entry in clip_results),
                "visual_review_pending_count": len(clips),
                "training_authorized": False,
            },
        }
        manifest["manifest_sha256"] = _canonical_manifest_hash(manifest)
        manifest_bytes = _canonical_json(manifest)
        (staging / "corpus-manifest.json").write_bytes(manifest_bytes)
        (staging / "README.txt").write_text(
            "Private generated TRAIN-4 motion references. Source/retargeted bytes are not distributable engine content.\n",
            encoding="utf-8",
        )
        if destination.exists():
            existing = destination / "corpus-manifest.json"
            if existing.is_file() and existing.read_bytes() == manifest_bytes:
                _remove_tree(staging)
                return manifest, destination
            raise FileExistsError(
                f"refusing to overwrite a different corpus generation: {destination}"
            )
        os.replace(staging, destination)
        return manifest, destination
    except Exception:
        if staging.exists():
            _remove_tree(staging)
        raise


def load_motion_corpus_profile(profile_path: Path) -> tuple[dict[str, Any], bytes]:
    profile_bytes = profile_path.read_bytes()
    document = json.loads(profile_bytes)
    if "base_profile_sha256" not in document:
        return document, profile_bytes
    if (
        document.get("schema_version") != SCHEMA_VERSION
        or document.get("base_profile_id") != BASE_MOTION_CORPUS_PROFILE_ID
        or document.get("base_profile_sha256")
        != BASE_MOTION_CORPUS_PROFILE_SHA256
        or document.get("status") != "Frozen"
        or set(document) != {
            "schema_version",
            "profile_id",
            "status",
            "base_profile_id",
            "base_profile_sha256",
            "variant",
        }
    ):
        raise ValueError("motion corpus overlay identity mismatch")
    base_path = profile_path.parent / "humanoid-motion-corpus-cmu.v1.json"
    base_bytes = base_path.read_bytes()
    if _sha256(base_bytes) != BASE_MOTION_CORPUS_PROFILE_SHA256:
        raise ValueError("motion corpus overlay base hash mismatch")
    base = json.loads(base_bytes)
    if base.get("profile_id") != BASE_MOTION_CORPUS_PROFILE_ID:
        raise ValueError("motion corpus overlay base profile mismatch")
    variant = document.get("variant")
    legacy_variant_keys = {
        frozenset(
            {
                "joint_velocity_limit_basis_points",
                "unidirectional_joint_minimum_microradians",
                "rationale",
            }
        ),
        frozenset(
            {
                "joint_velocity_limit_basis_points",
                "unidirectional_joint_minimum_microradians",
                "hip_roll_minimum_microradians",
                "rationale",
            }
        ),
    }
    temporal_variant_keys = frozenset(
        {
            "joint_velocity_limit_basis_points",
            "unidirectional_joint_minimum_microradians",
            "hip_roll_minimum_microradians",
            "ankle_roll_minimum_microradians",
            "ankle_roll_maximum_microradians",
            "ankle_roll_minimum_hard_reserve_microradians",
            "locomotion_ground_correction_maximum_absolute_micrometres",
            "excluded_clip_ids",
            "clip_source_overrides",
            "temporal_contact_solve",
            "contact_interval_stabilization",
            "rationale",
        }
    )
    stance_chain_variant_keys = temporal_variant_keys | {
        "additional_source_files",
        "clip_derivation_overrides",
    }
    swing_clearance_variant_keys = stance_chain_variant_keys | {
        "ankle_pitch_minimum_microradians",
        "ankle_pitch_minimum_hard_reserve_microradians",
    }
    if not isinstance(variant, dict) or frozenset(variant) not in {
        *legacy_variant_keys,
        temporal_variant_keys,
        stance_chain_variant_keys,
        swing_clearance_variant_keys,
    }:
        raise ValueError("motion corpus overlay variant mismatch")
    velocity_basis_points = int(variant["joint_velocity_limit_basis_points"])
    unidirectional_minimum = int(
        variant["unidirectional_joint_minimum_microradians"]
    )
    hip_roll_minimum = int(variant.get("hip_roll_minimum_microradians", 0))
    ankle_roll_minimum = int(
        variant.get(
            "ankle_roll_minimum_microradians",
            base["retarget"]["locomotion_collision_projection"][
                "ankle_roll_minimum_microradians"
            ],
        )
    )
    ankle_roll_maximum = int(
        variant.get(
            "ankle_roll_maximum_microradians",
            base["retarget"]["locomotion_collision_projection"][
                "ankle_roll_maximum_microradians"
            ],
        )
    )
    ankle_roll_minimum_hard_reserve = int(
        variant.get(
            "ankle_roll_minimum_hard_reserve_microradians",
            base["retarget"]["locomotion_collision_projection"][
                "ankle_roll_minimum_hard_reserve_microradians"
            ],
        )
    )
    ankle_pitch_minimum = int(
        variant.get(
            "ankle_pitch_minimum_microradians",
            base["retarget"]["locomotion_collision_projection"][
                "ankle_pitch_minimum_microradians"
            ],
        )
    )
    ankle_pitch_minimum_hard_reserve = int(
        variant.get(
            "ankle_pitch_minimum_hard_reserve_microradians",
            ankle_pitch_minimum + 698_132,
        )
    )
    if (
        not 0 < velocity_basis_points <= 10_000
        or unidirectional_minimum <= 0
        or not 0 <= hip_roll_minimum <= 523_599
        or ankle_roll_minimum >= ankle_roll_maximum
        or ankle_roll_minimum_hard_reserve < 0
        or not -523_599 <= ankle_pitch_minimum <= -349_066
        or ankle_pitch_minimum_hard_reserve
        != ankle_pitch_minimum + 698_132
        or not isinstance(variant["rationale"], str)
        or not variant["rationale"]
    ):
        raise ValueError("motion corpus overlay reserve is invalid")
    temporal_contact_solve = variant.get("temporal_contact_solve")
    if temporal_contact_solve is not None:
        _validate_temporal_contact_variant(variant, base)
    result = deepcopy(base)
    result["profile_id"] = document["profile_id"]
    result["retarget"]["joint_velocity_limit_basis_points"] = (
        velocity_basis_points
    )
    result["retarget"]["velocity_policy"] = (
        "causal projection to the declared basis-point fraction of each target "
        "joint maximum velocity at 60 Hz after soft-ROM projection"
    )
    projection = result["retarget"]["locomotion_collision_projection"]
    projection["hip_roll_minimum_microradians"] = hip_roll_minimum
    projection["ankle_pitch_minimum_microradians"] = ankle_pitch_minimum
    projection["ankle_roll_minimum_microradians"] = ankle_roll_minimum
    projection["ankle_roll_maximum_microradians"] = ankle_roll_maximum
    projection["ankle_roll_minimum_hard_reserve_microradians"] = (
        ankle_roll_minimum_hard_reserve
    )
    projection["knee_minimum_microradians"] = unidirectional_minimum
    projection["elbow_minimum_microradians"] = unidirectional_minimum
    projection[
        "unidirectional_joint_minimum_hard_reserve_microradians"
    ] = unidirectional_minimum
    projection["rationale"] = variant["rationale"]
    if temporal_contact_solve is not None:
        result["retarget"]["algorithm_id"] = temporal_contact_solve[
            "algorithm_id"
        ]
        result["retarget"]["velocity_policy"] = (
            "symmetric whole-clip temporal/contact solve followed by projection "
            "to the declared basis-point fraction of every joint maximum velocity"
        )
        result["retarget"]["temporal_contact_solve"] = deepcopy(
            temporal_contact_solve
        )
        result["retarget"]["ground_correction"][
            "locomotion_maximum_absolute_micrometres"
        ] = int(
            variant[
                "locomotion_ground_correction_maximum_absolute_micrometres"
            ]
        )
        result["retarget"]["ground_correction"]["algorithm"] = (
            "minimum collision-free Lipschitz root-height majorant over the "
            "temporally solved bilateral sole trajectory"
        )
        result["retarget"]["contact_thresholds"][
            "interval_stabilization"
        ] = deepcopy(variant["contact_interval_stabilization"])
        result["source_files"].extend(
            deepcopy(variant.get("additional_source_files", []))
        )
        result["source_files"].sort(key=lambda item: item["path"])
        excluded_clip_ids = set(variant["excluded_clip_ids"])
        result["clips"] = [
            clip
            for clip in result["clips"]
            if clip["clip_id"] not in excluded_clip_ids
        ]
        clips_by_id = {clip["clip_id"]: clip for clip in result["clips"]}
        for clip_id, override in variant["clip_source_overrides"].items():
            clips_by_id[clip_id].update(deepcopy(override))
        for clip_id, override in variant.get(
            "clip_derivation_overrides", {}
        ).items():
            clips_by_id[clip_id].update(deepcopy(override))
    return result, profile_bytes


def _validate_temporal_contact_variant(
    variant: dict[str, Any], base: dict[str, Any]
) -> None:
    solve = variant["temporal_contact_solve"]
    contacts = variant["contact_interval_stabilization"]
    excluded_clip_ids = variant["excluded_clip_ids"]
    clip_source_overrides = variant["clip_source_overrides"]
    additional_source_files = variant.get("additional_source_files", [])
    clip_derivation_overrides = variant.get("clip_derivation_overrides", {})
    algorithm_id = solve.get("algorithm_id") if isinstance(solve, dict) else None
    expected_solve_fields = {
        "algorithm_id",
        "smoothing",
        "joint_bounds_microradians",
        "support_phase",
        "root_height",
        "root_planar",
        "validation",
    }
    if algorithm_id in STANCE_CHAIN_ALGORITHM_IDS:
        expected_solve_fields.update(
            {"root_orientation", "source_frame_stride", "stance_chain"}
        )
    if (
        not isinstance(solve, dict)
        or set(solve) != expected_solve_fields
        or algorithm_id
        not in {
            "nextengine.cmu-temporal-contact-retarget.v1",
            *STANCE_CHAIN_ALGORITHM_IDS,
        }
        or (
            algorithm_id in STANCE_CHAIN_ALGORITHM_IDS
            and (
                isinstance(solve["source_frame_stride"], bool)
                or solve["source_frame_stride"] != 1
            )
        )
        or not isinstance(contacts, dict)
        or set(contacts)
        != {
            "sole_enter_height_micrometres",
            "sole_exit_height_micrometres",
            "sole_enter_speed_micrometres_per_second",
            "sole_exit_speed_micrometres_per_second",
            "minimum_on_frames",
            "minimum_off_frames",
        }
        or not isinstance(excluded_clip_ids, list)
        or not excluded_clip_ids
        or len(excluded_clip_ids) != len(set(excluded_clip_ids))
        or not isinstance(clip_source_overrides, dict)
        or not clip_source_overrides
        or not isinstance(additional_source_files, list)
        or not isinstance(clip_derivation_overrides, dict)
    ):
        raise ValueError("motion corpus temporal/contact solve identity mismatch")
    smoothing = solve["smoothing"]
    bounds = solve["joint_bounds_microradians"]
    support = solve["support_phase"]
    root_height = solve["root_height"]
    root_planar = solve["root_planar"]
    validation = solve["validation"]
    expected_support_fields = {
        "double_support_height_micrometres",
        "minimum_state_frames",
        "transition_smoothing_passes",
        "stance_knee_microradians",
        "stance_hip_roll_microradians",
        "stance_sole_leveling_iterations",
        "stance_sole_leveling_probe_microradians",
        "swing_knee_lift_microradians",
    }
    if algorithm_id in STANCE_CHAIN_ALGORITHM_IDS:
        expected_support_fields.add(
            "maximum_entry_sole_speed_micrometres_per_second"
        )
    expected_validation_fields = {
        "maximum_protected_joint_acceleration_microradians_per_second_squared",
        "maximum_root_vertical_speed_micrometres_per_second",
        "maximum_planar_root_correction_micrometres",
        "maximum_planar_root_correction_speed_micrometres_per_second",
        "minimum_contact_interval_frames",
    }
    if algorithm_id in STANCE_CHAIN_ALGORITHM_IDS:
        expected_validation_fields.update(
            {
                "maximum_support_sole_planar_speed_micrometres_per_second",
                "maximum_support_sole_vertical_speed_micrometres_per_second",
                "maximum_root_yaw_speed_microradians_per_second",
            }
        )
    if (
        not isinstance(smoothing, dict)
        or set(smoothing)
        != {
            "kernel_weights",
            "whole_body_passes",
            "protected_additional_passes",
            "protected_joint_ids",
        }
        or not isinstance(bounds, dict)
        or not bounds
        or not isinstance(support, dict)
        or set(support) != expected_support_fields
        or not isinstance(root_height, dict)
        or set(root_height)
        != {
            "minimum_clearance_micrometres",
            "maximum_vertical_speed_micrometres_per_second",
        }
        or not isinstance(root_planar, dict)
        or set(root_planar)
        != {
            "kernel_weights",
            "smoothing_passes",
            "endpoint_taper_frames",
            "maximum_absolute_correction_micrometres",
            "maximum_speed_micrometres_per_second",
        }
        or not isinstance(validation, dict)
        or set(validation) != expected_validation_fields
    ):
        raise ValueError("motion corpus temporal/contact solve layout mismatch")
    kernel = smoothing["kernel_weights"]
    protected = smoothing["protected_joint_ids"]
    if (
        not isinstance(kernel, list)
        or not kernel
        or len(kernel) % 2 == 0
        or kernel != list(reversed(kernel))
        or any(isinstance(value, bool) or not isinstance(value, int) or value <= 0 for value in kernel)
        or isinstance(smoothing["whole_body_passes"], bool)
        or not 0 <= int(smoothing["whole_body_passes"]) <= 256
        or isinstance(smoothing["protected_additional_passes"], bool)
        or not 0 <= int(smoothing["protected_additional_passes"]) <= 256
        or not isinstance(protected, list)
        or not protected
        or len(protected) != len(set(protected))
    ):
        raise ValueError("motion corpus temporal smoothing is invalid")
    descriptor_joint_suffixes = {
        "hip-roll",
        "hip-yaw",
        "knee",
        "ankle-pitch",
        "ankle-roll",
    }
    if algorithm_id in STANCE_CHAIN_ALGORITHM_IDS:
        descriptor_joint_suffixes.add("hip-pitch")
    descriptor_joint_ids = {
        joint_id
        for side in ("left", "right")
        for suffix in descriptor_joint_suffixes
        for joint_id in (f"joint.{side}-{suffix}",)
    }
    if set(bounds) != descriptor_joint_ids or any(
        not isinstance(value, list)
        or len(value) != 2
        or any(isinstance(item, bool) or not isinstance(item, int) for item in value)
        or value[0] > value[1]
        for value in bounds.values()
    ):
        raise ValueError("motion corpus temporal joint bounds are invalid")
    stance_chain = solve.get("stance_chain")
    if algorithm_id in STANCE_CHAIN_ALGORITHM_IDS:
        expected_stance_chain_fields = {
            "ordered_joint_suffixes",
            "support_weight_smoothing_passes",
            "final_joint_smoothing_passes",
            "outer_iterations",
            "conjugate_gradient_iterations",
            "jacobian_probe_microradians",
            "sole_normal_lever_micrometres",
            "maximum_joint_update_microradians",
            "maximum_root_update_micrometres",
            "contact_constraint_weight_q16",
            "joint_reference_weight_q16",
            "joint_velocity_weight_q16",
            "joint_acceleration_weight_q16",
            "root_reference_weight_q16",
            "root_velocity_weight_q16",
            "root_acceleration_weight_q16",
        }
        if algorithm_id in {
            "nextengine.cmu-stance-chain-retarget.v3",
            "nextengine.cmu-stance-chain-retarget.v4",
            "nextengine.cmu-stance-chain-retarget.v5",
            "nextengine.cmu-stance-chain-retarget.v6",
            "nextengine.cmu-stance-chain-retarget.v7",
            "nextengine.cmu-stance-chain-retarget.v8",
        }:
            expected_stance_chain_fields.update(
                {
                    "final_sole_pitch_projection_iterations",
                    "final_sole_pitch_smoothing_passes",
                    "final_sole_pitch_probe_microradians",
                    "final_sole_pitch_maximum_update_microradians",
                }
            )
        if algorithm_id in {
            "nextengine.cmu-stance-chain-retarget.v4",
            "nextengine.cmu-stance-chain-retarget.v5",
            "nextengine.cmu-stance-chain-retarget.v6",
            "nextengine.cmu-stance-chain-retarget.v7",
            "nextengine.cmu-stance-chain-retarget.v8",
        }:
            expected_stance_chain_fields.add(
                "final_sole_pitch_inverse_joint_weights_q16"
            )
        if algorithm_id == "nextengine.cmu-stance-chain-retarget.v5":
            expected_stance_chain_fields.add(
                "final_sole_pitch_support_dilation_frames"
            )
        if algorithm_id in {
            "nextengine.cmu-stance-chain-retarget.v6",
            "nextengine.cmu-stance-chain-retarget.v7",
            "nextengine.cmu-stance-chain-retarget.v8",
        }:
            expected_stance_chain_fields.update(
                {
                    "final_swing_clearance_iterations",
                    "final_swing_clearance_smoothing_passes",
                    "final_swing_clearance_probe_microradians",
                    "final_swing_clearance_maximum_update_microradians",
                }
            )
        if algorithm_id in {
            "nextengine.cmu-stance-chain-retarget.v7",
            "nextengine.cmu-stance-chain-retarget.v8",
        }:
            expected_stance_chain_fields.update(
                {
                    "final_sole_roll_projection_iterations",
                    "final_sole_roll_smoothing_passes",
                    "final_sole_roll_probe_microradians",
                    "final_sole_roll_maximum_update_microradians",
                    "final_sole_roll_inverse_joint_weights_q16",
                }
            )
        if algorithm_id == "nextengine.cmu-stance-chain-retarget.v8":
            expected_stance_chain_fields.update(
                {
                    "final_ankle_pitch_minimum_microradians",
                    "final_ankle_pitch_transfer_smoothing_passes",
                }
            )
        positive_integer_fields = expected_stance_chain_fields - {
            "ordered_joint_suffixes",
            "final_sole_pitch_inverse_joint_weights_q16",
            "final_sole_roll_inverse_joint_weights_q16",
            "final_ankle_pitch_minimum_microradians",
        }
        if (
            not isinstance(stance_chain, dict)
            or set(stance_chain) != expected_stance_chain_fields
            or stance_chain.get("ordered_joint_suffixes")
            != [
                "hip-pitch",
                "hip-roll",
                "knee",
                "ankle-pitch",
                "ankle-roll",
            ]
            or any(
                isinstance(stance_chain[field], bool)
                or not isinstance(stance_chain[field], int)
                or stance_chain[field] <= 0
                for field in positive_integer_fields
            )
            or int(stance_chain["support_weight_smoothing_passes"]) > 256
            or int(stance_chain["final_joint_smoothing_passes"]) > 256
            or int(stance_chain["outer_iterations"]) > 8
            or int(stance_chain["conjugate_gradient_iterations"]) > 512
            or int(stance_chain["jacobian_probe_microradians"]) > 100_000
            or int(stance_chain["maximum_joint_update_microradians"])
            > 500_000
            or int(stance_chain["maximum_root_update_micrometres"]) > 100_000
            or (
                algorithm_id
                in {
                    "nextengine.cmu-stance-chain-retarget.v3",
                    "nextengine.cmu-stance-chain-retarget.v4",
                    "nextengine.cmu-stance-chain-retarget.v5",
                    "nextengine.cmu-stance-chain-retarget.v6",
                    "nextengine.cmu-stance-chain-retarget.v7",
                    "nextengine.cmu-stance-chain-retarget.v8",
                }
                and (
                    int(stance_chain["final_sole_pitch_projection_iterations"])
                    > 8
                    or int(stance_chain["final_sole_pitch_smoothing_passes"])
                    > 256
                    or int(stance_chain["final_sole_pitch_probe_microradians"])
                    > 100_000
                    or int(
                        stance_chain[
                            "final_sole_pitch_maximum_update_microradians"
                        ]
                    )
                    > 500_000
                )
            )
            or (
                algorithm_id
                in {
                    "nextengine.cmu-stance-chain-retarget.v4",
                    "nextengine.cmu-stance-chain-retarget.v5",
                    "nextengine.cmu-stance-chain-retarget.v6",
                    "nextengine.cmu-stance-chain-retarget.v7",
                    "nextengine.cmu-stance-chain-retarget.v8",
                }
                and (
                    not isinstance(
                        stance_chain[
                            "final_sole_pitch_inverse_joint_weights_q16"
                        ],
                        list,
                    )
                    or len(
                        stance_chain[
                            "final_sole_pitch_inverse_joint_weights_q16"
                        ]
                    )
                    != 3
                    or any(
                        isinstance(value, bool)
                        or not isinstance(value, int)
                        or not 0 < value <= 65_536
                        for value in stance_chain[
                            "final_sole_pitch_inverse_joint_weights_q16"
                        ]
                    )
                )
            )
            or (
                algorithm_id == "nextengine.cmu-stance-chain-retarget.v5"
                and int(
                    stance_chain["final_sole_pitch_support_dilation_frames"]
                )
                > 256
            )
            or (
                algorithm_id
                in {
                    "nextengine.cmu-stance-chain-retarget.v6",
                    "nextengine.cmu-stance-chain-retarget.v7",
                    "nextengine.cmu-stance-chain-retarget.v8",
                }
                and (
                    int(stance_chain["final_swing_clearance_iterations"]) > 8
                    or int(
                        stance_chain["final_swing_clearance_smoothing_passes"]
                    )
                    > 256
                    or int(
                        stance_chain["final_swing_clearance_probe_microradians"]
                    )
                    > 100_000
                    or int(
                        stance_chain[
                            "final_swing_clearance_maximum_update_microradians"
                        ]
                    )
                    > 500_000
                )
            )
            or (
                algorithm_id
                in {
                    "nextengine.cmu-stance-chain-retarget.v7",
                    "nextengine.cmu-stance-chain-retarget.v8",
                }
                and (
                    int(stance_chain["final_sole_roll_projection_iterations"])
                    > 8
                    or int(stance_chain["final_sole_roll_smoothing_passes"])
                    > 256
                    or int(stance_chain["final_sole_roll_probe_microradians"])
                    > 100_000
                    or int(
                        stance_chain[
                            "final_sole_roll_maximum_update_microradians"
                        ]
                    )
                    > 500_000
                    or not isinstance(
                        stance_chain[
                            "final_sole_roll_inverse_joint_weights_q16"
                        ],
                        list,
                    )
                    or len(
                        stance_chain[
                            "final_sole_roll_inverse_joint_weights_q16"
                        ]
                    )
                    != 2
                    or any(
                        isinstance(value, bool)
                        or not isinstance(value, int)
                        or not 0 < value <= 65_536
                        for value in stance_chain[
                            "final_sole_roll_inverse_joint_weights_q16"
                        ]
                    )
                )
            )
            or (
                algorithm_id == "nextengine.cmu-stance-chain-retarget.v8"
                and (
                    int(
                        stance_chain[
                            "final_ankle_pitch_transfer_smoothing_passes"
                        ]
                    )
                    > 256
                    or not -523_599
                    <= int(
                        stance_chain[
                            "final_ankle_pitch_minimum_microradians"
                        ]
                    )
                    <= -349_066
                    or int(
                        stance_chain[
                            "final_ankle_pitch_minimum_microradians"
                        ]
                    )
                    < int(variant["ankle_pitch_minimum_microradians"])
                )
            )
        ):
            raise ValueError("motion corpus stance-chain solve is invalid")
    elif stance_chain is not None:
        raise ValueError("motion corpus temporal V1 has unexpected stance chain")
    root_orientation = solve.get("root_orientation")
    if algorithm_id in STANCE_CHAIN_ALGORITHM_IDS:
        if (
            not isinstance(root_orientation, dict)
            or set(root_orientation)
            != {
                "kernel_weights",
                "smoothing_passes",
                "minimum_alignment_speed_micrometres_per_second",
                "maximum_yaw_speed_microradians_per_second",
                "velocity_projection_passes",
            }
            or root_orientation["kernel_weights"] != list(kernel)
            or isinstance(root_orientation["smoothing_passes"], bool)
            or not 0 <= int(root_orientation["smoothing_passes"]) <= 256
            or int(root_orientation["minimum_alignment_speed_micrometres_per_second"])
            <= 0
            or int(root_orientation["maximum_yaw_speed_microradians_per_second"])
            <= 0
            or not 1 <= int(root_orientation["velocity_projection_passes"]) <= 16
        ):
            raise ValueError("motion corpus root-orientation solve is invalid")
    elif root_orientation is not None:
        raise ValueError("motion corpus temporal V1 has unexpected root orientation")
    base_projection = base["retarget"]["locomotion_collision_projection"]
    base_clips = {clip["clip_id"]: clip for clip in base["clips"]}
    base_source_paths = {item["path"] for item in base["source_files"]}
    additional_paths = [
        item.get("path") if isinstance(item, dict) else None
        for item in additional_source_files
    ]
    if (
        any(
            not isinstance(item, dict)
            or set(item) != {"path", "sha256"}
            or not isinstance(item["path"], str)
            or not item["path"].endswith((".asf", ".amc"))
            or item["path"].startswith("/")
            or ".." in Path(item["path"]).parts
            or item["path"] in base_source_paths
            or not isinstance(item["sha256"], str)
            or len(item["sha256"]) != 64
            or any(character not in "0123456789abcdef" for character in item["sha256"])
            for item in additional_source_files
        )
        or len(additional_paths) != len(set(additional_paths))
        or additional_paths != sorted(additional_paths)
        or (
            algorithm_id in STANCE_CHAIN_ALGORITHM_IDS
            and not additional_source_files
        )
        or (
            algorithm_id not in STANCE_CHAIN_ALGORITHM_IDS
            and additional_source_files
        )
    ):
        raise ValueError("motion corpus additional source closure is invalid")
    source_paths = base_source_paths | set(additional_paths)
    expected_override_fields = {
        "subject",
        "trial",
        "source_first_frame",
        "source_last_frame",
        "split_group_id",
        "planar_root_scale_basis_points",
        "description",
    }
    if (
        any(clip_id not in base_clips for clip_id in excluded_clip_ids)
        or any(
            clip_id not in base_clips
            or clip_id in excluded_clip_ids
            or not isinstance(override, dict)
            or set(override) != expected_override_fields
            or not isinstance(override["subject"], str)
            or not isinstance(override["trial"], str)
            or f'{override["subject"]}/{override["subject"]}.asf'
            not in source_paths
            or f'{override["subject"]}/{override["subject"]}_{override["trial"]}.amc'
            not in source_paths
            or int(override["source_first_frame"]) <= 0
            or int(override["source_last_frame"])
            < int(override["source_first_frame"])
            or not isinstance(override["split_group_id"], str)
            or not override["split_group_id"]
            or not 0 < int(override["planar_root_scale_basis_points"]) <= 10_000
            or not isinstance(override["description"], str)
            or not override["description"]
            for clip_id, override in clip_source_overrides.items()
        )
        or int(
            variant[
                "locomotion_ground_correction_maximum_absolute_micrometres"
            ]
        )
        <= 0
        or any(
            clip_id not in base_clips
            or clip_id in excluded_clip_ids
            or base_clips[clip_id].get("derive_mirror_as") is not None
            or not isinstance(override, dict)
            or set(override) != {"derive_mirror_as"}
            or not isinstance(override["derive_mirror_as"], str)
            or override["derive_mirror_as"] == clip_id
            or override["derive_mirror_as"] not in excluded_clip_ids
            or override["derive_mirror_as"] not in base_clips
            for clip_id, override in clip_derivation_overrides.items()
        )
        or len(
            {
                override["derive_mirror_as"]
                for override in clip_derivation_overrides.values()
                if isinstance(override, dict)
                and isinstance(override.get("derive_mirror_as"), str)
            }
        )
        != len(clip_derivation_overrides)
        or (
            algorithm_id in STANCE_CHAIN_ALGORITHM_IDS
            and not clip_derivation_overrides
        )
        or (
            algorithm_id not in STANCE_CHAIN_ALGORITHM_IDS
            and clip_derivation_overrides
        )
        or int(support["double_support_height_micrometres"]) < 0
        or (
            algorithm_id in STANCE_CHAIN_ALGORITHM_IDS
            and int(
                support[
                    "maximum_entry_sole_speed_micrometres_per_second"
                ]
            )
            <= 0
        )
        or int(support["minimum_state_frames"]) <= 0
        or int(support["transition_smoothing_passes"]) < 0
        or int(support["stance_knee_microradians"]) < 0
        or int(support["stance_hip_roll_microradians"])
        < int(variant["hip_roll_minimum_microradians"])
        or not 1 <= int(support["stance_sole_leveling_iterations"]) <= 8
        or not 1
        <= int(support["stance_sole_leveling_probe_microradians"])
        <= 100_000
        or int(support["swing_knee_lift_microradians"]) < 0
        or int(root_height["minimum_clearance_micrometres"]) < 0
        or int(root_height["maximum_vertical_speed_micrometres_per_second"])
        <= 0
        or not isinstance(root_planar["kernel_weights"], list)
        or not root_planar["kernel_weights"]
        or len(root_planar["kernel_weights"]) % 2 == 0
        or root_planar["kernel_weights"]
        != list(reversed(root_planar["kernel_weights"]))
        or any(
            isinstance(value, bool) or not isinstance(value, int) or value <= 0
            for value in root_planar["kernel_weights"]
        )
        or isinstance(root_planar["smoothing_passes"], bool)
        or not 0 <= int(root_planar["smoothing_passes"]) <= 256
        or isinstance(root_planar["endpoint_taper_frames"], bool)
        or not 0 <= int(root_planar["endpoint_taper_frames"]) <= 256
        or int(root_planar["maximum_absolute_correction_micrometres"]) <= 0
        or int(root_planar["maximum_speed_micrometres_per_second"]) <= 0
        or int(
            validation[
                "maximum_protected_joint_acceleration_microradians_per_second_squared"
            ]
        )
        <= 0
        or int(validation["maximum_root_vertical_speed_micrometres_per_second"])
        < int(root_height["maximum_vertical_speed_micrometres_per_second"])
        or int(validation["maximum_planar_root_correction_micrometres"])
        != int(root_planar["maximum_absolute_correction_micrometres"])
        or int(
            validation[
                "maximum_planar_root_correction_speed_micrometres_per_second"
            ]
        )
        != int(root_planar["maximum_speed_micrometres_per_second"])
        or int(validation["minimum_contact_interval_frames"]) <= 0
        or (
            algorithm_id in STANCE_CHAIN_ALGORITHM_IDS
            and (
                int(
                    validation[
                        "maximum_support_sole_planar_speed_micrometres_per_second"
                    ]
                )
                <= 0
                or int(
                    validation[
                        "maximum_root_yaw_speed_microradians_per_second"
                    ]
                )
                <= 0
                or int(
                    validation[
                        "maximum_support_sole_vertical_speed_micrometres_per_second"
                    ]
                )
                <= 0
            )
        )
        or int(contacts["sole_enter_height_micrometres"])
        > int(contacts["sole_exit_height_micrometres"])
        or int(contacts["sole_enter_speed_micrometres_per_second"])
        > int(contacts["sole_exit_speed_micrometres_per_second"])
        or int(contacts["minimum_on_frames"]) <= 0
        or int(contacts["minimum_off_frames"]) <= 0
        or int(variant["hip_roll_minimum_microradians"])
        < int(base_projection["hip_roll_minimum_microradians"])
        or int(
            solve["joint_bounds_microradians"][
                "joint.left-ankle-roll"
            ][0]
        )
        != int(variant["ankle_roll_minimum_microradians"])
        or int(
            solve["joint_bounds_microradians"][
                "joint.right-ankle-roll"
            ][0]
        )
        != int(variant["ankle_roll_minimum_microradians"])
        or int(
            solve["joint_bounds_microradians"][
                "joint.left-ankle-roll"
            ][1]
        )
        != int(variant["ankle_roll_maximum_microradians"])
        or int(
            solve["joint_bounds_microradians"][
                "joint.right-ankle-roll"
            ][1]
        )
        != int(variant["ankle_roll_maximum_microradians"])
        or (
            algorithm_id
            in {
                "nextengine.cmu-stance-chain-retarget.v6",
                "nextengine.cmu-stance-chain-retarget.v7",
                "nextengine.cmu-stance-chain-retarget.v8",
            }
            and (
                "ankle_pitch_minimum_microradians" not in variant
                or "ankle_pitch_minimum_hard_reserve_microradians"
                not in variant
                or int(
                    solve["joint_bounds_microradians"][
                        "joint.left-ankle-pitch"
                    ][0]
                )
                != int(variant["ankle_pitch_minimum_microradians"])
                or int(
                    solve["joint_bounds_microradians"][
                        "joint.right-ankle-pitch"
                    ][0]
                )
                != int(variant["ankle_pitch_minimum_microradians"])
            )
        )
        or (
            algorithm_id
            not in {
                "nextengine.cmu-stance-chain-retarget.v6",
                "nextengine.cmu-stance-chain-retarget.v7",
                "nextengine.cmu-stance-chain-retarget.v8",
            }
            and (
                "ankle_pitch_minimum_microradians" in variant
                or "ankle_pitch_minimum_hard_reserve_microradians"
                in variant
            )
        )
    ):
        raise ValueError("motion corpus temporal/contact thresholds are invalid")


def deterministic_npz_bytes(clip: RetargetedClip, metadata: dict[str, Any]) -> bytes:
    arrays: dict[str, NDArray[Any]] = canonical_integer_arrays(clip)
    arrays["metadata_json_utf8"] = np.frombuffer(_canonical_json(metadata), dtype=np.uint8)
    output = io.BytesIO()
    with zipfile.ZipFile(output, mode="w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name in sorted(arrays):
            payload = io.BytesIO()
            np.lib.format.write_array(payload, np.asarray(arrays[name]), allow_pickle=False)
            entry = zipfile.ZipInfo(f"{name}.npy", date_time=(1980, 1, 1, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = 0o100644 << 16
            archive.writestr(entry, payload.getvalue(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    return output.getvalue()


def audit_motion_corpus_physx_poses(
    *,
    runner: Path,
    descriptor_path: Path,
    corpus_root: Path,
    output_store: Path,
    partition: str,
) -> tuple[dict[str, Any], Path]:
    if partition not in {"locomotion", "recovery"}:
        raise ValueError("unsupported motion corpus partition")
    corpus_root = corpus_root.resolve()
    manifest_path = corpus_root / "corpus-manifest.json"
    manifest_bytes = manifest_path.read_bytes()
    manifest = json.loads(manifest_bytes)
    if (
        manifest.get("status") != "VALIDATED"
        or _canonical_manifest_hash(manifest) != manifest.get("manifest_sha256")
    ):
        raise ValueError("motion corpus manifest is not a validated canonical generation")
    descriptor_bytes = descriptor_path.read_bytes()
    descriptor = json.loads(descriptor_bytes)
    target = manifest["target"]
    if (
        _sha256(descriptor_bytes) != target["descriptor_file_sha256"]
        or descriptor.get("body_schema_hash") != target["body_schema_hash"]
        or descriptor.get("compiled_descriptor_hash")
        != target["compiled_descriptor_hash"]
    ):
        raise ValueError("motion corpus PhysX audit descriptor closure mismatch")
    poses: list[dict[str, Any]] = []
    artifact_count = 0
    for entry in manifest["clips"]:
        if entry["partition"] != partition:
            continue
        relative_path = Path(entry["artifact"]["relative_path"])
        if relative_path.is_absolute() or ".." in relative_path.parts:
            raise ValueError("motion corpus artifact path is not canonical relative")
        artifact_path = corpus_root / relative_path
        artifact_bytes = artifact_path.read_bytes()
        if _sha256(artifact_bytes) != entry["artifact"]["sha256"]:
            raise ValueError("motion corpus artifact hash mismatch")
        artifact_count += 1
        with np.load(io.BytesIO(artifact_bytes), allow_pickle=False) as artifact:
            roots = artifact["root_quaternion_q1_30"]
            joints = artifact["joint_position_urad"]
            if len(roots) != len(joints) or joints.shape[1] != len(descriptor["joints"]):
                raise ValueError("motion corpus artifact pose shape mismatch")
            poses.extend(
                {
                    "clip_id": entry["clip_id"],
                    "split": entry["split"],
                    "reference_frame": frame,
                    "root_quaternion_q1_30": roots[frame].tolist(),
                    "joint_position_urad": joints[frame].tolist(),
                }
                for frame in range(len(joints))
            )
    if not poses:
        raise ValueError("motion corpus PhysX audit partition is empty")
    document = {
        "schema_version": 1,
        "schema_id": "nextengine.motor.reference-pose-audit-input.v1",
        "body_schema_hash": target["body_schema_hash"],
        "compiled_descriptor_hash": target["compiled_descriptor_hash"],
        "corpus_manifest_sha256": manifest["manifest_sha256"],
        "poses": poses,
    }
    input_payload = _canonical_json(document)
    runner_payload = runner.read_bytes()
    runner_sha256 = _sha256(runner_payload)
    adapter_tool_sha256 = _sha256(Path(__file__).read_bytes())
    result = subprocess.run(
        [str(runner.resolve())],
        input=input_payload,
        capture_output=True,
        timeout=900,
        check=False,
    )
    if result.returncode != 0:
        diagnostic = result.stderr.decode("utf-8", errors="replace").strip()
        raise ValueError(f"native reference pose audit failed: {diagnostic}")
    report = json.loads(result.stdout)
    if (
        report.get("check") != "TRAIN-4-PHYSX-REFERENCE-POSE-AUDIT"
        or report.get("claim") != "ReferencePoseResetFeasibilityOnly"
        or report.get("body_schema_hash") != target["body_schema_hash"]
        or report.get("compiled_descriptor_hash") != target["compiled_descriptor_hash"]
        or report.get("corpus_manifest_sha256") != manifest["manifest_sha256"]
        or report.get("pose_count") != len(poses)
    ):
        raise ValueError("native reference pose audit output closure mismatch")
    report["partition"] = partition
    report["artifact_count"] = artifact_count
    report["corpus_manifest_file_sha256"] = _sha256(manifest_bytes)
    report["input_sha256"] = _sha256(input_payload)
    report["runner_sha256"] = runner_sha256
    report["adapter_tool_sha256"] = adapter_tool_sha256
    output_payload = _canonical_json(report)
    destination = (
        output_store.resolve()
        / "evaluations"
        / "TRAIN-4"
        / (
            f"reference-pose-audit-{partition}-{manifest['manifest_sha256'][:16]}-"
            f"{runner_sha256[:16]}-{adapter_tool_sha256[:16]}.json"
        )
    )
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists() and destination.read_bytes() != output_payload:
        raise ValueError(f"refusing to overwrite a different pose audit: {destination}")
    if not destination.exists():
        destination.write_bytes(output_payload)
    return report, destination


def validate_clip(
    clip: RetargetedClip, descriptor: dict[str, Any], profile: dict[str, Any]
) -> dict[str, Any]:
    errors: list[str] = []
    joint_minimum = np.empty(len(descriptor["joints"]), dtype=np.int64)
    joint_maximum = np.empty(len(descriptor["joints"]), dtype=np.int64)
    joint_hard_minimum = np.empty(len(descriptor["joints"]), dtype=np.int64)
    joint_hard_maximum = np.empty(len(descriptor["joints"]), dtype=np.int64)
    velocity_maximum = np.empty(len(descriptor["joints"]), dtype=np.int64)
    joint_by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    for joint in descriptor["joints"]:
        ordinal = int(joint["dof_ordinal"])
        joint_minimum[ordinal], joint_maximum[ordinal] = joint["soft_limit_microradians"]
        joint_hard_minimum[ordinal], joint_hard_maximum[ordinal] = joint[
            "hard_limit_microradians"
        ]
        velocity_maximum[ordinal] = int(joint["maximum_velocity_microradians_per_second"])
    below = np.maximum(joint_minimum - clip.joint_position_urad, 0)
    above = np.maximum(clip.joint_position_urad - joint_maximum, 0)
    maximum_rom_violation = int(max(np.max(below), np.max(above)))
    if maximum_rom_violation:
        errors.append("RETARGET_SOFT_ROM_VIOLATION")
    maximum_velocity_ratio = float(
        np.max(np.abs(clip.joint_velocity_urad_s) / velocity_maximum)
    )
    velocity_limit_basis_points = int(
        profile["retarget"].get("joint_velocity_limit_basis_points", 10_000)
    )
    if maximum_velocity_ratio > velocity_limit_basis_points / 10_000.0 + 1.0e-6:
        errors.append("RETARGET_JOINT_VELOCITY_EXCESS")
    maximum_correction = int(np.max(np.abs(clip.ground_correction_um)))
    correction_field = (
        "locomotion_maximum_absolute_micrometres"
        if clip.partition == "locomotion"
        else "recovery_maximum_absolute_micrometres"
    )
    correction_bound = int(profile["retarget"]["ground_correction"][correction_field])
    if maximum_correction > correction_bound:
        errors.append("RETARGET_GROUND_CORRECTION_EXCESS")
    minimum_collider_height = int(np.min(clip.minimum_collider_height_um))
    if minimum_collider_height < -2:
        errors.append("RETARGET_GROUND_PENETRATION")
    minimum_nonfoot_height = int(np.min(clip.minimum_nonfoot_height_um))
    if clip.partition == "locomotion" and minimum_nonfoot_height < -2:
        errors.append("RETARGET_LOCOMOTION_NONFOOT_PENETRATION")

    clamped = clip.raw_soft_rom_excess_urad > 0
    clamped_fraction = float(np.count_nonzero(clamped)) / float(clamped.size)
    maximum_raw_excess = int(np.max(clip.raw_soft_rom_excess_urad))
    if clamped_fraction > 0.35 or maximum_raw_excess > 1_570_796:
        errors.append("RETARGET_EXCESSIVE_SOFT_ROM_PROJECTION")
    velocity_projected = clip.velocity_projection_urad > 0
    velocity_projected_fraction = float(np.count_nonzero(velocity_projected)) / float(
        velocity_projected.size
    )
    maximum_velocity_projection = int(np.max(clip.velocity_projection_urad))
    collision_projected = clip.locomotion_collision_projection_urad > 0
    collision_projected_fraction = float(np.count_nonzero(collision_projected)) / float(
        collision_projected.size
    )
    maximum_collision_projection = int(
        np.max(clip.locomotion_collision_projection_urad)
    )
    if clip.partition == "recovery" and maximum_collision_projection:
        errors.append("RETARGET_RECOVERY_COLLISION_PROJECTION_FORBIDDEN")
    ankle_roll_ordinals = np.asarray(
        [
            int(joint_by_id[f"joint.{side}-ankle-roll"]["dof_ordinal"])
            for side in ("left", "right")
        ],
        dtype=np.int64,
    )
    ankle_roll_positions = clip.joint_position_urad[:, ankle_roll_ordinals]
    ankle_roll_at_soft_boundary = np.logical_or(
        ankle_roll_positions == joint_minimum[ankle_roll_ordinals],
        ankle_roll_positions == joint_maximum[ankle_roll_ordinals],
    )
    ankle_roll_soft_boundary_fraction = float(
        np.count_nonzero(ankle_roll_at_soft_boundary)
    ) / float(ankle_roll_at_soft_boundary.size)
    ankle_roll_hard_reserve = np.minimum(
        ankle_roll_positions - joint_hard_minimum[ankle_roll_ordinals],
        joint_hard_maximum[ankle_roll_ordinals] - ankle_roll_positions,
    )
    minimum_ankle_roll_hard_reserve = int(np.min(ankle_roll_hard_reserve))
    ankle_roll_projected = (
        clip.locomotion_collision_projection_urad[:, ankle_roll_ordinals] > 0
    )
    ankle_roll_projected_fraction = float(np.count_nonzero(ankle_roll_projected)) / float(
        ankle_roll_projected.size
    )
    ankle_pitch_ordinals = np.asarray(
        [
            int(joint_by_id[f"joint.{side}-ankle-pitch"]["dof_ordinal"])
            for side in ("left", "right")
        ],
        dtype=np.int64,
    )
    ankle_pitch_positions = clip.joint_position_urad[:, ankle_pitch_ordinals]
    ankle_pitch_hard_reserve = np.minimum(
        ankle_pitch_positions - joint_hard_minimum[ankle_pitch_ordinals],
        joint_hard_maximum[ankle_pitch_ordinals] - ankle_pitch_positions,
    )
    minimum_ankle_pitch_hard_reserve = int(
        np.min(ankle_pitch_hard_reserve)
    )
    unidirectional_ordinals = np.asarray(
        [
            int(joint_by_id[f"joint.{side}-{kind}"]["dof_ordinal"])
            for side in ("left", "right")
            for kind in ("knee", "elbow")
        ],
        dtype=np.int64,
    )
    unidirectional_positions = clip.joint_position_urad[:, unidirectional_ordinals]
    unidirectional_at_lower_soft_boundary = (
        unidirectional_positions == joint_minimum[unidirectional_ordinals]
    )
    unidirectional_lower_soft_boundary_fraction = float(
        np.count_nonzero(unidirectional_at_lower_soft_boundary)
    ) / float(unidirectional_at_lower_soft_boundary.size)
    unidirectional_hard_reserve = np.minimum(
        unidirectional_positions - joint_hard_minimum[unidirectional_ordinals],
        joint_hard_maximum[unidirectional_ordinals] - unidirectional_positions,
    )
    minimum_unidirectional_hard_reserve = int(
        np.min(unidirectional_hard_reserve)
    )
    if clip.partition == "locomotion":
        projection = profile["retarget"]["locomotion_collision_projection"]
        maximum_boundary_fraction = float(
            projection["ankle_roll_maximum_soft_boundary_fraction"]
        )
        minimum_hard_reserve = int(
            projection["ankle_roll_minimum_hard_reserve_microradians"]
        )
        if ankle_roll_soft_boundary_fraction > maximum_boundary_fraction:
            errors.append("RETARGET_LOCOMOTION_ANKLE_ROLL_SOFT_BOUNDARY_SATURATION")
        if minimum_ankle_roll_hard_reserve < minimum_hard_reserve:
            errors.append("RETARGET_LOCOMOTION_ANKLE_ROLL_HARD_RESERVE_SHORTFALL")
        maximum_unidirectional_lower_boundary_fraction = float(
            projection[
                "unidirectional_joint_maximum_lower_soft_boundary_fraction"
            ]
        )
        minimum_unidirectional_reserve = int(
            projection["unidirectional_joint_minimum_hard_reserve_microradians"]
        )
        temporal_solve = profile["retarget"].get("temporal_contact_solve")
        stance_chain = (
            temporal_solve.get("stance_chain")
            if isinstance(temporal_solve, dict)
            else None
        )
        final_ankle_pitch_minimum = (
            stance_chain.get("final_ankle_pitch_minimum_microradians")
            if isinstance(stance_chain, dict)
            else None
        )
        if final_ankle_pitch_minimum is not None and np.any(
            ankle_pitch_positions < int(final_ankle_pitch_minimum)
        ):
            errors.append(
                "RETARGET_LOCOMOTION_FINAL_ANKLE_PITCH_RESERVE_SHORTFALL"
            )
        if (
            unidirectional_lower_soft_boundary_fraction
            > maximum_unidirectional_lower_boundary_fraction
        ):
            errors.append(
                "RETARGET_LOCOMOTION_UNIDIRECTIONAL_LOWER_SOFT_BOUNDARY_SATURATION"
            )
        if minimum_unidirectional_hard_reserve < minimum_unidirectional_reserve:
            errors.append("RETARGET_LOCOMOTION_UNIDIRECTIONAL_HARD_RESERVE_SHORTFALL")
        for side in ("left", "right"):
            hip_yaw = int(joint_by_id[f"joint.{side}-hip-yaw"]["dof_ordinal"])
            hip_roll = int(joint_by_id[f"joint.{side}-hip-roll"]["dof_ordinal"])
            ankle_pitch = int(
                joint_by_id[f"joint.{side}-ankle-pitch"]["dof_ordinal"]
            )
            ankle_roll = int(
                joint_by_id[f"joint.{side}-ankle-roll"]["dof_ordinal"]
            )
            knee = int(joint_by_id[f"joint.{side}-knee"]["dof_ordinal"])
            elbow = int(joint_by_id[f"joint.{side}-elbow"]["dof_ordinal"])
            shoulder_roll = int(
                joint_by_id[f"joint.{side}-shoulder-roll"]["dof_ordinal"]
            )
            if np.any(
                clip.joint_position_urad[:, hip_yaw]
                != int(projection["hip_yaw_microradians"])
            ):
                errors.append("RETARGET_LOCOMOTION_HIP_YAW_PROJECTION_MISMATCH")
            if np.any(
                clip.joint_position_urad[:, hip_roll]
                < int(projection["hip_roll_minimum_microradians"])
            ):
                errors.append("RETARGET_LOCOMOTION_HIP_CLEARANCE_MISMATCH")
            if np.any(
                clip.joint_position_urad[:, ankle_pitch]
                < int(projection["ankle_pitch_minimum_microradians"])
            ):
                errors.append("RETARGET_LOCOMOTION_ANKLE_RESERVE_MISMATCH")
            if np.any(
                clip.joint_position_urad[:, ankle_roll]
                < int(projection["ankle_roll_minimum_microradians"])
            ) or np.any(
                clip.joint_position_urad[:, ankle_roll]
                > int(projection["ankle_roll_maximum_microradians"])
            ):
                errors.append("RETARGET_LOCOMOTION_ANKLE_ROLL_PROJECTION_MISMATCH")
            if np.any(
                clip.joint_position_urad[:, knee]
                < int(projection["knee_minimum_microradians"])
            ) or np.any(
                clip.joint_position_urad[:, elbow]
                < int(projection["elbow_minimum_microradians"])
            ):
                errors.append("RETARGET_LOCOMOTION_UNIDIRECTIONAL_PROJECTION_MISMATCH")
            if np.any(
                clip.joint_position_urad[:, shoulder_roll]
                < int(projection["shoulder_roll_minimum_microradians"])
            ):
                errors.append("RETARGET_LOCOMOTION_SHOULDER_CLEARANCE_MISMATCH")

    temporal_contact_solve = profile["retarget"].get("temporal_contact_solve")
    maximum_protected_acceleration = 0
    maximum_planar_root_correction = 0
    maximum_planar_root_correction_speed = 0
    maximum_support_sole_planar_speed = 0
    maximum_support_sole_vertical_speed = 0
    maximum_root_vertical_speed = int(
        np.max(np.abs(clip.root_linear_velocity_um_s[:, 1]))
    )
    maximum_root_yaw_speed = int(
        np.max(np.abs(clip.root_yaw_velocity_urad_s))
    )
    minimum_internal_contact_interval = len(clip.contacts)
    if clip.partition == "locomotion" and temporal_contact_solve is not None:
        if clip.planar_root_correction_um is None:
            errors.append("RETARGET_PLANAR_ROOT_CORRECTION_MISSING")
        else:
            maximum_planar_root_correction = int(
                np.rint(
                    np.max(
                        np.linalg.norm(
                            clip.planar_root_correction_um[:, (0, 2)].astype(
                                np.float64
                            ),
                            axis=1,
                        )
                    )
                )
            )
            if len(clip.planar_root_correction_um) > 1:
                maximum_planar_root_correction_speed = int(
                    np.rint(
                        np.max(
                            np.linalg.norm(
                                np.diff(
                                    clip.planar_root_correction_um[:, (0, 2)],
                                    axis=0,
                                ).astype(np.float64)
                                * 60.0,
                                axis=1,
                            )
                        )
                    )
                )
        temporal_bounds = temporal_contact_solve["joint_bounds_microradians"]
        for joint_id, (minimum, maximum) in temporal_bounds.items():
            ordinal = int(joint_by_id[joint_id]["dof_ordinal"])
            if np.any(clip.joint_position_urad[:, ordinal] < int(minimum)) or np.any(
                clip.joint_position_urad[:, ordinal] > int(maximum)
            ):
                errors.append("RETARGET_TEMPORAL_JOINT_BOUND_MISMATCH")
                break
        protected_ordinals = np.asarray(
            [
                int(joint_by_id[joint_id]["dof_ordinal"])
                for joint_id in temporal_contact_solve["smoothing"][
                    "protected_joint_ids"
                ]
            ],
            dtype=np.int64,
        )
        if len(clip.joint_position_urad) > 2:
            acceleration = np.abs(
                np.diff(
                    clip.joint_position_urad[:, protected_ordinals],
                    n=2,
                    axis=0,
                )
                * 3600
            )
            maximum_protected_acceleration = int(np.max(acceleration))
        validation = temporal_contact_solve["validation"]
        if maximum_protected_acceleration > int(
            validation[
                "maximum_protected_joint_acceleration_microradians_per_second_squared"
            ]
        ):
            errors.append("RETARGET_TEMPORAL_JOINT_ACCELERATION_EXCESS")
        if maximum_root_vertical_speed > int(
            validation["maximum_root_vertical_speed_micrometres_per_second"]
        ):
            errors.append("RETARGET_TEMPORAL_ROOT_VERTICAL_SPEED_EXCESS")
        if maximum_planar_root_correction > int(
            validation["maximum_planar_root_correction_micrometres"]
        ):
            errors.append("RETARGET_PLANAR_ROOT_CORRECTION_EXCESS")
        if maximum_planar_root_correction_speed > int(
            validation[
                "maximum_planar_root_correction_speed_micrometres_per_second"
            ]
        ):
            errors.append("RETARGET_PLANAR_ROOT_CORRECTION_SPEED_EXCESS")
        interval_lengths = [
            length
            for column in range(2)
            for length in _internal_run_lengths(clip.contacts[:, column])
        ]
        if interval_lengths:
            minimum_internal_contact_interval = min(interval_lengths)
        if minimum_internal_contact_interval < int(
            validation["minimum_contact_interval_frames"]
        ):
            errors.append("RETARGET_CONTACT_INTERVAL_TOO_SHORT")
        if temporal_contact_solve.get("stance_chain") is not None:
            if maximum_root_yaw_speed > int(
                validation["maximum_root_yaw_speed_microradians_per_second"]
            ):
                errors.append("RETARGET_ROOT_YAW_SPEED_EXCESS")
            if (
                clip.stance_support_state is None
                or clip.stance_support_state.shape != (len(clip.contacts),)
                or np.any(
                    (clip.stance_support_state < 0)
                    | (clip.stance_support_state > 2)
                )
            ):
                errors.append("RETARGET_STANCE_SUPPORT_STATE_MISMATCH")
            else:
                effector_index = {
                    effector_id: index
                    for index, effector_id in enumerate(clip.effector_ids)
                }
                for side_index, side in enumerate(("left", "right")):
                    pair = (
                        effector_index[f"effector.{side}-heel"],
                        effector_index[f"effector.{side}-forefoot"],
                    )
                    center = np.mean(
                        clip.effector_position_um[:, pair, :].astype(np.float64),
                        axis=1,
                    )
                    velocity = np.diff(center, axis=0) * 60.0
                    # The stance-chain state leads the physical touchdown so that
                    # the constrained leg can settle before contact.  It is not a
                    # contact observation: during that lead-in the selected sole
                    # may still be the swing foot.  Measure retained stance speed
                    # against the generated sole-contact channel, which is the
                    # condition consumed by the dynamic admission audit.
                    active = clip.contacts[:, side_index] != 0
                    retained = active[1:] & active[:-1]
                    if np.any(retained):
                        maximum_support_sole_planar_speed = max(
                            maximum_support_sole_planar_speed,
                            int(
                                np.rint(
                                    np.max(
                                        np.linalg.norm(
                                            velocity[retained][:, (0, 2)], axis=1
                                        )
                                    )
                                )
                            ),
                        )
                        maximum_support_sole_vertical_speed = max(
                            maximum_support_sole_vertical_speed,
                            int(np.rint(np.max(np.abs(velocity[retained, 1])))),
                        )
                if maximum_support_sole_planar_speed > int(
                    validation[
                        "maximum_support_sole_planar_speed_micrometres_per_second"
                    ]
                ):
                    errors.append("RETARGET_STANCE_SOLE_PLANAR_SPEED_EXCESS")
                if maximum_support_sole_vertical_speed > int(
                    validation[
                        "maximum_support_sole_vertical_speed_micrometres_per_second"
                    ]
                ):
                    errors.append("RETARGET_STANCE_SOLE_VERTICAL_SPEED_EXCESS")

    planar_velocity = np.linalg.norm(clip.root_linear_velocity_um_s[:, (0, 2)].astype(np.float64), axis=1) / 1_000_000.0
    planar_displacement = float(
        np.linalg.norm((clip.root_position_um[-1, (0, 2)] - clip.root_position_um[0, (0, 2)]).astype(np.float64))
        / 1_000_000.0
    )
    first_speed = float(np.median(planar_velocity[: min(30, len(planar_velocity))]))
    last_speed = float(np.median(planar_velocity[-min(30, len(planar_velocity)) :]))
    mean_speed = float(np.mean(planar_velocity))
    yaw_delta = float(clip.root_yaw_urad[-1] - clip.root_yaw_urad[0]) / 1_000_000.0
    root_height_delta = float(clip.root_position_um[-1, 1] - clip.root_position_um[0, 1]) / 1_000_000.0
    com_height_delta = float(clip.center_of_mass_um[-1, 1] - clip.center_of_mass_um[0, 1]) / 1_000_000.0
    final_com_height = float(clip.center_of_mass_um[-1, 1]) / 1_000_000.0
    final_root_rotation = quaternion_to_matrix(
        clip.root_quaternion_q1_30[-1].astype(np.float64) / float(1 << 30)
    )
    final_root_up_y = float(final_root_rotation[1, 1])
    skill = clip.skill
    if skill in {"neutral_idle", "weight_shift"} and planar_displacement > 0.45:
        errors.append("RETARGET_IDLE_DRIFT")
    if skill.startswith("walk_") and not (0.15 <= mean_speed <= 3.5):
        errors.append("RETARGET_WALK_SPEED_LABEL_MISMATCH")
    if skill.startswith("start_") and not (first_speed <= 0.20 and last_speed >= 0.20):
        errors.append("RETARGET_START_STATE_MISMATCH")
    if skill.startswith("stop_") and not (first_speed >= 0.20 and last_speed <= 0.25):
        errors.append("RETARGET_STOP_STATE_MISMATCH")
    if skill == "turn_gradual_left" and yaw_delta >= -0.08:
        errors.append("RETARGET_LEFT_TURN_FACING_MISMATCH")
    if skill == "turn_gradual_right" and yaw_delta <= 0.08:
        errors.append("RETARGET_RIGHT_TURN_FACING_MISMATCH")
    if skill.startswith("brace_safe_fall") and (
        com_height_delta >= -0.25 or final_root_up_y >= 0.75
    ):
        errors.append("RETARGET_FALL_DIRECTION_MISMATCH")
    if skill.startswith("getup_") and (
        com_height_delta <= 0.15 or final_com_height < 0.50 or final_root_up_y < 0.75
    ):
        errors.append("RETARGET_GETUP_DIRECTION_MISMATCH")
    if skill.startswith("side_transition_") and abs(root_height_delta) > 0.45:
        errors.append("RETARGET_SIDE_TRANSITION_HEIGHT_MISMATCH")
    sole_occupancy = float(np.mean(np.any(clip.contacts[:, :2] != 0, axis=1)))
    recovery_occupancy = float(np.mean(np.any(clip.contacts[:, 2:] != 0, axis=1)))
    if clip.partition == "locomotion" and sole_occupancy < 0.05:
        errors.append("RETARGET_SOLE_CONTACT_EMPTY")
    if clip.partition == "recovery" and max(sole_occupancy, recovery_occupancy) < 0.05:
        errors.append("RETARGET_RECOVERY_CONTACT_EMPTY")
    if clip.loop:
        pose_delta = int(np.max(np.abs(clip.joint_position_urad[-1] - clip.joint_position_urad[0])))
        velocity_delta = int(np.max(np.abs(clip.joint_velocity_urad_s[-1] - clip.joint_velocity_urad_s[0])))
        if pose_delta > int(profile["retarget"]["loop_pose_max_microradians"]):
            errors.append("RETARGET_LOOP_POSE_DISCONTINUITY")
        if velocity_delta > int(profile["retarget"]["loop_velocity_max_microradians_per_second"]):
            errors.append("RETARGET_LOOP_VELOCITY_DISCONTINUITY")

    return {
        "status": "PASS" if not errors else "FAIL",
        "errors": errors,
        "metrics": {
            "maximum_soft_rom_violation_microradians": maximum_rom_violation,
            "maximum_joint_velocity_ratio": round(maximum_velocity_ratio, 6),
            "clamped_channel_fraction": round(clamped_fraction, 6),
            "maximum_raw_soft_rom_excess_microradians": maximum_raw_excess,
            "velocity_projected_channel_fraction": round(velocity_projected_fraction, 6),
            "maximum_velocity_projection_microradians": maximum_velocity_projection,
            "collision_projected_channel_fraction": round(
                collision_projected_fraction, 6
            ),
            "maximum_collision_projection_microradians": maximum_collision_projection,
            "ankle_roll_projected_channel_fraction": round(
                ankle_roll_projected_fraction, 6
            ),
            "ankle_roll_soft_boundary_fraction": round(
                ankle_roll_soft_boundary_fraction, 6
            ),
            "minimum_ankle_roll_hard_reserve_microradians": (
                minimum_ankle_roll_hard_reserve
            ),
            "minimum_ankle_pitch_hard_reserve_microradians": (
                minimum_ankle_pitch_hard_reserve
            ),
            "unidirectional_joint_lower_soft_boundary_fraction": round(
                unidirectional_lower_soft_boundary_fraction, 6
            ),
            "minimum_unidirectional_joint_hard_reserve_microradians": (
                minimum_unidirectional_hard_reserve
            ),
            "maximum_ground_correction_micrometres": maximum_correction,
            "minimum_collider_height_micrometres": minimum_collider_height,
            "minimum_nonfoot_height_micrometres": minimum_nonfoot_height,
            "maximum_protected_joint_acceleration_microradians_per_second_squared": (
                maximum_protected_acceleration
            ),
            "maximum_root_vertical_speed_micrometres_per_second": (
                maximum_root_vertical_speed
            ),
            "maximum_root_yaw_speed_microradians_per_second": (
                maximum_root_yaw_speed
            ),
            "maximum_planar_root_correction_micrometres": (
                maximum_planar_root_correction
            ),
            "maximum_planar_root_correction_speed_micrometres_per_second": (
                maximum_planar_root_correction_speed
            ),
            "minimum_internal_sole_contact_interval_frames": (
                minimum_internal_contact_interval
            ),
            "maximum_support_sole_planar_speed_micrometres_per_second": (
                maximum_support_sole_planar_speed
            ),
            "maximum_support_sole_vertical_speed_micrometres_per_second": (
                maximum_support_sole_vertical_speed
            ),
            "planar_displacement_metres": round(planar_displacement, 6),
            "mean_planar_speed_metres_per_second": round(mean_speed, 6),
            "first_window_speed_metres_per_second": round(first_speed, 6),
            "last_window_speed_metres_per_second": round(last_speed, 6),
            "root_yaw_delta_radians": round(yaw_delta, 6),
            "root_height_delta_metres": round(root_height_delta, 6),
            "center_of_mass_height_delta_metres": round(com_height_delta, 6),
            "final_center_of_mass_height_metres": round(final_com_height, 6),
            "final_root_up_y": round(final_root_up_y, 6),
            "sole_contact_occupancy": round(sole_occupancy, 6),
            "recovery_contact_occupancy": round(recovery_occupancy, 6),
        },
    }


def _validate_closure(
    profile: dict[str, Any],
    descriptor: dict[str, Any],
    descriptor_bytes: bytes,
    dataset_root: Path,
) -> None:
    if profile.get("schema_version") != SCHEMA_VERSION:
        raise ValueError("unsupported motion corpus profile")
    admitted_partitions = profile.get("admitted_partitions")
    if (
        not isinstance(admitted_partitions, list)
        or not admitted_partitions
        or len(admitted_partitions) != len(set(admitted_partitions))
        or any(partition not in {"locomotion", "recovery"} for partition in admitted_partitions)
    ):
        raise ValueError("motion corpus admitted partition closure is invalid")
    target = profile["body_schema"]
    checks = {
        "id": descriptor.get("body_schema_id"),
        "revision": descriptor.get("body_schema_revision"),
        "hash": descriptor.get("body_schema_hash"),
        "compiled_descriptor_hash": descriptor.get("compiled_descriptor_hash"),
    }
    for field, actual in checks.items():
        if target[field] != actual:
            raise ValueError(f"motion corpus target {field} mismatch")
    projection = profile["retarget"]["locomotion_collision_projection"]
    velocity_limit_basis_points = int(
        profile["retarget"].get("joint_velocity_limit_basis_points", 10_000)
    )
    if not 0 < velocity_limit_basis_points <= 10_000:
        raise ValueError("motion corpus joint velocity reserve is invalid")
    ankle_roll_minimum = int(projection["ankle_roll_minimum_microradians"])
    ankle_roll_maximum = int(projection["ankle_roll_maximum_microradians"])
    minimum_hard_reserve = int(
        projection["ankle_roll_minimum_hard_reserve_microradians"]
    )
    maximum_boundary_fraction = float(
        projection["ankle_roll_maximum_soft_boundary_fraction"]
    )
    knee_minimum = int(projection["knee_minimum_microradians"])
    elbow_minimum = int(projection["elbow_minimum_microradians"])
    unidirectional_reserve = int(
        projection["unidirectional_joint_minimum_hard_reserve_microradians"]
    )
    unidirectional_lower_boundary_fraction = float(
        projection[
            "unidirectional_joint_maximum_lower_soft_boundary_fraction"
        ]
    )
    if ankle_roll_minimum >= ankle_roll_maximum:
        raise ValueError("motion corpus ankle-roll projection range is invalid")
    if minimum_hard_reserve < 0 or not 0.0 <= maximum_boundary_fraction <= 1.0:
        raise ValueError("motion corpus ankle-roll audit threshold is invalid")
    if (
        knee_minimum < 0
        or elbow_minimum < 0
        or unidirectional_reserve < 0
        or not 0.0 <= unidirectional_lower_boundary_fraction <= 1.0
    ):
        raise ValueError("motion corpus unidirectional-joint audit threshold is invalid")
    joint_by_id = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    temporal_contact_solve = profile["retarget"].get("temporal_contact_solve")
    if temporal_contact_solve is not None:
        bounds = temporal_contact_solve["joint_bounds_microradians"]
        protected = temporal_contact_solve["smoothing"]["protected_joint_ids"]
        if any(joint_id not in joint_by_id for joint_id in (*bounds, *protected)):
            raise ValueError("motion corpus temporal joint closure mismatch")
        for joint_id, (minimum, maximum) in bounds.items():
            soft_minimum, soft_maximum = map(
                int, joint_by_id[joint_id]["soft_limit_microradians"]
            )
            if not soft_minimum <= int(minimum) <= int(maximum) <= soft_maximum:
                raise ValueError("motion corpus temporal bound exceeds soft ROM")
        support = temporal_contact_solve["support_phase"]
        for side in ("left", "right"):
            knee = bounds[f"joint.{side}-knee"]
            hip_roll = bounds[f"joint.{side}-hip-roll"]
            if not (
                int(knee[0])
                <= int(support["stance_knee_microradians"])
                <= int(knee[1])
                and int(hip_roll[0])
                <= int(support["stance_hip_roll_microradians"])
                <= int(hip_roll[1])
            ):
                raise ValueError("motion corpus stance target exceeds temporal bound")
    for side in ("left", "right"):
        joint = joint_by_id[f"joint.{side}-ankle-roll"]
        soft_minimum, soft_maximum = map(int, joint["soft_limit_microradians"])
        hard_minimum, hard_maximum = map(int, joint["hard_limit_microradians"])
        if ankle_roll_minimum < soft_minimum or ankle_roll_maximum > soft_maximum:
            raise ValueError("motion corpus ankle-roll projection exceeds soft ROM")
        available_reserve = min(
            ankle_roll_minimum - hard_minimum,
            hard_maximum - ankle_roll_maximum,
        )
        if available_reserve < minimum_hard_reserve:
            raise ValueError("motion corpus ankle-roll projection reserve is insufficient")
        for kind, projected_minimum in (
            ("knee", knee_minimum),
            ("elbow", elbow_minimum),
        ):
            joint = joint_by_id[f"joint.{side}-{kind}"]
            soft_minimum, soft_maximum = map(int, joint["soft_limit_microradians"])
            hard_minimum, hard_maximum = map(int, joint["hard_limit_microradians"])
            if not soft_minimum <= projected_minimum <= soft_maximum:
                raise ValueError(
                    "motion corpus unidirectional-joint projection exceeds soft ROM"
                )
            available_reserve = min(
                projected_minimum - hard_minimum,
                hard_maximum - projected_minimum,
            )
            if available_reserve < unidirectional_reserve:
                raise ValueError(
                    "motion corpus unidirectional-joint projection reserve is insufficient"
                )
            projected_maximum = hard_maximum - unidirectional_reserve
            if (
                projected_minimum >= projected_maximum
                or not soft_minimum <= projected_maximum <= soft_maximum
            ):
                raise ValueError(
                    "motion corpus unidirectional-joint symmetric reserve is infeasible"
                )
    if _sha256(descriptor_bytes) != "f1f2be6a486367038f605709ebf54edb4fa6ef400fa797dd772d7590e06f3014":
        raise ValueError("motion corpus target descriptor file hash mismatch")
    raw_root = dataset_root.resolve() / "raw"
    for item in profile["source_files"]:
        path = raw_root / item["path"]
        if not path.is_file() or _sha256(path.read_bytes()) != item["sha256"]:
            raise ValueError(f"motion corpus source hash mismatch: {item['path']}")
    license_root = dataset_root.resolve() / "license-snapshots"
    snapshots = {
        "cmu-mocap-home.html": profile["source"]["rights_snapshot_sha256"],
        "cmu-mocap-info.html": profile["source"]["format_snapshot_sha256"],
        **{item["path"]: item["sha256"] for item in profile["source_pages"]},
    }
    for relative, expected in snapshots.items():
        path = license_root / relative
        if not path.is_file() or _sha256(path.read_bytes()) != expected:
            raise ValueError(f"motion corpus provenance snapshot mismatch: {relative}")
    rights = profile["source"]["rights_statement"] + "\n"
    if _sha256(rights.encode("utf-8")) != profile["source"]["rights_statement_lf_sha256"]:
        raise ValueError("motion corpus rights statement hash mismatch")
    intended = profile["source"]["intended_use"]
    if not intended["commercial_training"] or not intended["model_output_distribution"]:
        raise ValueError("motion corpus rights do not admit the intended candidate")


def _internal_run_lengths(values: NDArray[np.uint8]) -> tuple[int, ...]:
    if values.ndim != 1 or len(values) == 0:
        raise ValueError("contact interval audit requires one non-empty channel")
    boundaries = np.flatnonzero(values[1:] != values[:-1]) + 1
    starts = np.concatenate((np.asarray([0]), boundaries))
    ends = np.concatenate((boundaries, np.asarray([len(values)])))
    if len(starts) <= 2:
        return ()
    return tuple(int(end - start) for start, end in zip(starts[1:-1], ends[1:-1]))


def _split_audit(clips: list[RetargetedClip]) -> dict[str, Any]:
    errors: list[str] = []
    group_splits: dict[str, str] = {}
    subject_splits: dict[str, str] = {}
    split_counts: dict[str, int] = {}
    for clip in clips:
        split_counts[clip.split] = split_counts.get(clip.split, 0) + 1
        previous = group_splits.setdefault(clip.split_group_id, clip.split)
        if previous != clip.split:
            errors.append(f"split group {clip.split_group_id} crosses {previous}/{clip.split}")
        subject = clip.source_clip_id.split("-")[1]
        previous_subject = subject_splits.setdefault(subject, clip.split)
        if previous_subject != clip.split:
            errors.append(f"subject {subject} crosses {previous_subject}/{clip.split}")
    if set(split_counts) != {"train", "validation", "heldout"}:
        errors.append("train/validation/heldout must all be non-empty")
    return {
        "status": "PASS" if not errors else "FAIL",
        "errors": errors,
        "split_clip_counts": dict(sorted(split_counts.items())),
        "subject_assignments": dict(sorted(subject_splits.items())),
        "split_group_assignments": dict(sorted(group_splits.items())),
    }


def _coverage_audit(clips: list[RetargetedClip], profile: dict[str, Any]) -> dict[str, Any]:
    requirements = profile["coverage_requirements"]
    required_splits = tuple(requirements["required_splits"])
    minimum_groups = int(requirements["minimum_independent_groups_per_split"])
    errors: list[str] = []
    classes: dict[str, Any] = {}
    for class_id, requirement in requirements["classes"].items():
        members = [clip for clip in clips if clip.coverage_class == class_id]
        split_groups = {
            split: sorted({clip.split_group_id for clip in members if clip.split == split})
            for split in required_splits
        }
        for split, groups in split_groups.items():
            if len(groups) < minimum_groups:
                errors.append(
                    f"coverage class {class_id} has {len(groups)} independent groups in {split}; "
                    f"requires {minimum_groups}"
                )
        skills = sorted({clip.skill for clip in members})
        missing_skills = sorted(set(requirement["required_skills"]) - set(skills))
        if missing_skills:
            errors.append(
                f"coverage class {class_id} is missing skills {','.join(missing_skills)}"
            )
        minimum_cycles = int(requirement.get("minimum_full_gait_cycles_per_split_group", 0))
        cycles_by_group: dict[str, int] = {}
        if minimum_cycles:
            for clip in members:
                if clip.derived_from is not None:
                    continue
                sole_contacts = clip.contacts[:, :2]
                rising = np.count_nonzero(
                    (sole_contacts[1:] > sole_contacts[:-1]) & (sole_contacts[1:] > 0),
                    axis=0,
                )
                cycles_by_group[clip.split_group_id] = cycles_by_group.get(
                    clip.split_group_id, 0
                ) + int(np.min(rising))
            for group_id in sorted({group for groups in split_groups.values() for group in groups}):
                if cycles_by_group.get(group_id, 0) < minimum_cycles:
                    errors.append(
                        f"coverage class {class_id} group {group_id} has "
                        f"{cycles_by_group.get(group_id, 0)} full gait cycles; requires {minimum_cycles}"
                    )
        cycles_pass = not minimum_cycles or all(
            cycles_by_group.get(group, 0) >= minimum_cycles
            for groups in split_groups.values()
            for group in groups
        )
        classes[class_id] = {
            "required_skills": requirement["required_skills"],
            "observed_skills": skills,
            "split_group_ids": split_groups,
            "full_gait_cycles_by_split_group": dict(sorted(cycles_by_group.items())),
            "status": "PASS"
            if all(len(groups) >= minimum_groups for groups in split_groups.values())
            and not missing_skills
            and cycles_pass
            else "FAIL",
        }
    return {
        "status": "PASS" if not errors else "FAIL",
        "minimum_independent_groups_per_split": minimum_groups,
        "required_splits": list(required_splits),
        "classes": classes,
        "errors": errors,
    }


def _validate_mirrors(clips: list[RetargetedClip], descriptor: dict[str, Any]) -> list[dict[str, Any]]:
    results = []
    by_id = {clip.clip_id: clip for clip in clips}
    for clip in clips:
        if clip.mirrored_from is None:
            continue
        source = by_id[clip.mirrored_from]
        restored = mirror_clip(clip, descriptor, source.clip_id)
        mismatches = [
            name
            for name, values in canonical_integer_arrays(source).items()
            if not np.array_equal(values, canonical_integer_arrays(restored)[name])
        ]
        results.append(
            {
                "source_clip_id": source.clip_id,
                "mirror_clip_id": clip.clip_id,
                "status": "PASS" if not mismatches else "FAIL",
                "mismatched_arrays": mismatches,
            }
        )
    return results


def _clip_metadata(
    clip: RetargetedClip,
    descriptor: dict[str, Any],
    profile_sha256: str,
    descriptor_sha256: str,
    tool_sha256: str,
) -> dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "clip_id": clip.clip_id,
        "source_clip_id": clip.source_clip_id,
        "mirrored_from": clip.mirrored_from,
        "skill": clip.skill,
        "partition": clip.partition,
        "split": clip.split,
        "split_group_id": clip.split_group_id,
        "rate_hz": 60,
        "loop": clip.loop,
        "profile_sha256": profile_sha256,
        "target_descriptor_sha256": descriptor_sha256,
        "tool_sha256": tool_sha256,
        "joint_ids": _ordered_joint_ids(descriptor),
        "effector_ids": clip.effector_ids,
        "contact_ids": CONTACT_IDS,
        "source_overlay_bones": clip.source_overlay_bones,
    }


def _ordered_joint_ids(descriptor: dict[str, Any]) -> tuple[str, ...]:
    names = [""] * len(descriptor["joints"])
    for joint in descriptor["joints"]:
        names[int(joint["dof_ordinal"])] = joint["joint_id"]
    return tuple(names)


def _canonical_manifest_hash(manifest: dict[str, Any]) -> str:
    value = dict(manifest)
    value.pop("manifest_sha256", None)
    return _sha256(_canonical_json(value))


def _tool_sha256() -> str:
    directory = Path(__file__).resolve().parent
    digest = hashlib.sha256()
    for name in ("cmu_motion.py", "motion_math.py", "motion_retarget.py", "motion_preview.py", "motion_corpus.py"):
        payload = (directory / name).read_bytes()
        encoded = name.encode("utf-8")
        digest.update(len(encoded).to_bytes(8, "little"))
        digest.update(encoded)
        digest.update(len(payload).to_bytes(8, "little"))
        digest.update(payload)
    return digest.hexdigest()


def _canonical_json(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode("utf-8")


def _sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def _remove_tree(root: Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_dir():
            path.rmdir()
        else:
            path.unlink()
    root.rmdir()
