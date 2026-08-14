#!/usr/bin/env python3
"""Build the hash-closed, optimizer-free TRAIN-4 contact prototype bundle."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import shutil
import subprocess
import tempfile
import zipfile
from pathlib import Path
from typing import Any, Sequence

import numpy as np

from next_lab import contact_manifold
from next_lab.contact_manifold import (
    ColliderClosure,
    ContactManifoldTolerances,
    project_reference_contact_manifold,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-audit", type=Path, required=True)
    parser.add_argument("--prototype-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--case-scope",
        choices=("all", "discriminator"),
        default="all",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    source_audit_path = args.source_audit.resolve()
    profile_path = args.prototype_profile.resolve()
    descriptor_path = args.descriptor.resolve()
    corpus_root = args.corpus_root.resolve()
    manifest_path = corpus_root / "corpus-manifest.json"
    for path in (source_audit_path, profile_path, descriptor_path, manifest_path):
        if not path.is_file():
            raise FileNotFoundError(path)
    output = _external_directory(args.output)
    profile = json.loads(profile_path.read_bytes())
    source_audit = json.loads(source_audit_path.read_bytes())
    descriptor = json.loads(descriptor_path.read_bytes())
    corpus_manifest = json.loads(manifest_path.read_bytes())
    _validate_inputs(
        profile=profile,
        profile_path=profile_path,
        source_audit=source_audit,
        source_audit_path=source_audit_path,
        descriptor_path=descriptor_path,
        corpus_manifest=corpus_manifest,
        manifest_path=manifest_path,
    )
    inventory = _select_cases(source_audit["results"]["phase_results"], profile)
    selected = _select_case_scope(inventory, profile, args.case_scope)
    tolerances = _tolerances(profile["projection"])
    collider_closure = _collider_closure(profile["projection"])
    clip_manifest = {entry["clip_id"]: entry for entry in corpus_manifest["clips"]}
    clip_cache: dict[str, tuple[dict[str, np.ndarray], dict[str, Any]]] = {}
    staging = Path(
        tempfile.mkdtemp(prefix="nextengine-contact-prototype-", dir=output.parent)
    )
    try:
        artifact_directory = staging / "cases"
        artifact_directory.mkdir()
        case_records: list[dict[str, Any]] = []
        for selection in selected:
            clip_id = selection["clip_id"]
            if clip_id not in clip_cache:
                clip_cache[clip_id] = _load_clip(
                    corpus_root, clip_manifest[clip_id]
                )
            arrays, metadata = clip_cache[clip_id]
            row = selection["source_row"]
            frame_first = int(row["start_frame"])
            frame_last = int(row["terminal_frame"])
            projection = project_reference_contact_manifold(
                descriptor=descriptor,
                effector_ids=tuple(metadata["effector_ids"]),
                root_position_um=arrays["root_position_um"],
                root_quaternion_q1_30=arrays["root_quaternion_q1_30"],
                root_yaw_urad=arrays["root_yaw_urad"],
                joint_position_urad=arrays["joint_position_urad"],
                effector_position_um=arrays["effector_position_um"],
                contacts=arrays["contacts"],
                support_state=arrays["stance_support_state"],
                frame_first=frame_first,
                frame_last=frame_last,
                tolerances=tolerances,
                collider_closure=collider_closure,
            )
            artifact_id = f"{clip_id}--start-{frame_first:04d}"
            artifact_metadata = {
                "schema_version": 1,
                "artifact_id": artifact_id,
                "clip_id": clip_id,
                "split": row["split"],
                "frame_first": frame_first,
                "frame_last": frame_last,
                "source_case_ordinal": int(row["case_ordinal"]),
                "baseline_status": row["status"],
                "target_failure_categories": selection[
                    "target_failure_categories"
                ],
                "control_for": selection["control_for"],
                "effector_ids": metadata["effector_ids"],
                "source_clip_artifact_sha256": clip_manifest[clip_id]["artifact"][
                    "sha256"
                ],
            }
            artifact_arrays = {
                "center_of_mass_um": projection.center_of_mass_um,
                "contact_modes": projection.contact_modes,
                "contacts": projection.contacts,
                "effector_position_um": projection.effector_position_um,
                "joint_position_urad": projection.joint_position_urad,
                "joint_velocity_urad_s": projection.joint_velocity_urad_s,
                "phase_u16": arrays["phase_u16"][frame_first : frame_last + 1],
                "reference_frame": np.arange(
                    frame_first, frame_last + 1, dtype=np.int64
                ),
                "root_linear_velocity_um_s": (
                    projection.root_linear_velocity_um_s
                ),
                "root_position_um": projection.root_position_um,
                "root_quaternion_q1_30": arrays["root_quaternion_q1_30"][
                    frame_first : frame_last + 1
                ],
                "root_yaw_urad": arrays["root_yaw_urad"][
                    frame_first : frame_last + 1
                ],
                "root_yaw_velocity_urad_s": (
                    projection.root_yaw_velocity_urad_s
                ),
            }
            artifact_bytes = _deterministic_npz_bytes(
                artifact_arrays, artifact_metadata
            )
            if artifact_bytes != _deterministic_npz_bytes(
                artifact_arrays, artifact_metadata
            ):
                raise RuntimeError("contact prototype serialization is unstable")
            artifact_path = artifact_directory / f"{artifact_id}.npz"
            artifact_path.write_bytes(artifact_bytes)
            violation = row["first_required_safety_violation"]
            case_records.append(
                {
                    **artifact_metadata,
                    "baseline_reasons": sorted(violation["reasons"])
                    if violation
                    else [],
                    "artifact": {
                        "relative_path": str(artifact_path.relative_to(staging)),
                        "sha256": hashlib.sha256(artifact_bytes).hexdigest(),
                        "bytes": len(artifact_bytes),
                    },
                    "projection_diagnostics": projection.diagnostics,
                }
            )

        all_pass = all(
            case["projection_diagnostics"]["status"] == "PASS"
            for case in case_records
        )
        report = {
            "schema_version": 1,
            "check": "TRAIN-4-CONTACT-MANIFOLD-PROTOTYPE-BUILD",
            "status": "PASS" if all_pass else "FAIL",
            "claim": "OptimizerFreeResearchOnly",
            "gate_decision": "NO_CHANGE",
            "prototype_id": profile["prototype_id"],
            "scope": {
                "case_count": len(case_records),
                "failure_case_count": sum(
                    bool(case["target_failure_categories"])
                    for case in case_records
                ),
                "control_case_count": sum(
                    not case["target_failure_categories"]
                    for case in case_records
                ),
                "case_scope": args.case_scope,
                "prototype_inventory_case_count": len(inventory),
                "clip_ids": [
                    clip_id
                    for clip_id in profile["selection"]["ordered_clip_ids"]
                    if any(case["clip_id"] == clip_id for case in selected)
                ],
                "selection": profile["selection"],
                "discriminator": profile.get("discriminator"),
                "projection": profile["projection"],
            },
            "identities": {
                "source_audit_sha256": _sha256(source_audit_path),
                "prototype_profile_sha256": _sha256(profile_path),
                "descriptor_sha256": _sha256(descriptor_path),
                "corpus_manifest_file_sha256": _sha256(manifest_path),
                "corpus_manifest_sha256": corpus_manifest["manifest_sha256"],
                "tool_sha256": _sha256(Path(__file__).resolve()),
                "projector_sha256": _sha256(
                    Path(contact_manifold.__file__).resolve()
                ),
            },
            "cases": case_records,
            "optimizer_steps": 0,
            "training_runs": 0,
            "learned_policy_claim": False,
            "repository": _repository_state(),
        }
        report["manifest_sha256"] = hashlib.sha256(
            _canonical_json(report)
        ).hexdigest()
        (staging / "prototype-manifest.json").write_bytes(
            (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
        )
        os.replace(staging, output)
        print(
            json.dumps(
                {
                    "output": str(output),
                    "status": report["status"],
                    "case_count": len(case_records),
                    "manifest_sha256": report["manifest_sha256"],
                    "optimizer_steps": 0,
                    "training_runs": 0,
                },
                indent=2,
                sort_keys=True,
            )
        )
        if not all_pass:
            raise SystemExit(1)
    except BaseException:
        if staging.exists():
            shutil.rmtree(staging)
        raise


def _select_cases(
    source_rows: Sequence[dict[str, Any]], profile: dict[str, Any]
) -> list[dict[str, Any]]:
    selection = profile["selection"]
    selected: dict[tuple[str, int], dict[str, Any]] = {}
    for clip_id in selection["ordered_clip_ids"]:
        clip_rows = [row for row in source_rows if row["clip_id"] == clip_id]
        if not clip_rows:
            raise ValueError(f"selected clip is absent from source audit: {clip_id}")
        passing = [row for row in clip_rows if row["status"] == "PASS"]
        for category in selection["ordered_target_failure_categories"]:
            failing = [
                row
                for row in clip_rows
                if row["status"] == "FAIL"
                and category
                in (row["first_required_safety_violation"] or {}).get(
                    "reasons", ()
                )
            ]
            if not failing:
                continue
            failure = min(failing, key=lambda row: int(row["start_frame"]))
            failure_key = (clip_id, int(failure["start_frame"]))
            target = selected.setdefault(
                failure_key,
                _selection_record(failure),
            )
            target["target_failure_categories"].append(category)
            before = max(
                (
                    row
                    for row in passing
                    if int(row["start_frame"]) < int(failure["start_frame"])
                ),
                key=lambda row: int(row["start_frame"]),
                default=None,
            )
            after = min(
                (
                    row
                    for row in passing
                    if int(row["start_frame"]) > int(failure["start_frame"])
                ),
                key=lambda row: int(row["start_frame"]),
                default=None,
            )
            if before is None or after is None:
                raise ValueError("selected failure lacks two-sided passing controls")
            for position, control in (("before", before), ("after", after)):
                control_key = (clip_id, int(control["start_frame"]))
                record = selected.setdefault(
                    control_key,
                    _selection_record(control),
                )
                record["control_for"].append(
                    {
                        "category": category,
                        "failure_start_frame": int(failure["start_frame"]),
                        "position": position,
                    }
                )
    result = sorted(
        selected.values(),
        key=lambda record: (
            record["clip_id"].encode("utf-8"),
            int(record["source_row"]["start_frame"]),
        ),
    )
    for record in result:
        record["target_failure_categories"].sort()
        record["control_for"].sort(
            key=lambda value: (
                value["category"],
                value["failure_start_frame"],
                value["position"],
            )
        )
    failure_count = sum(bool(item["target_failure_categories"]) for item in result)
    control_count = len(result) - failure_count
    if (
        len(result) != int(selection["expected_case_count"])
        or failure_count != int(selection["expected_failure_case_count"])
        or control_count != int(selection["expected_control_case_count"])
    ):
        raise ValueError("selected prototype case counts differ from frozen profile")
    return result


def _select_case_scope(
    inventory: Sequence[dict[str, Any]],
    profile: dict[str, Any],
    case_scope: str,
) -> list[dict[str, Any]]:
    if case_scope == "all":
        return list(inventory)
    discriminator = profile.get("discriminator")
    if not isinstance(discriminator, dict):
        raise ValueError("prototype profile has no discriminator case scope")
    ordered_ordinals = discriminator.get("ordered_source_case_ordinals")
    if (
        not isinstance(ordered_ordinals, list)
        or not ordered_ordinals
        or any(
            isinstance(value, bool) or not isinstance(value, int)
            for value in ordered_ordinals
        )
        or len(ordered_ordinals) != len(set(ordered_ordinals))
    ):
        raise ValueError("prototype discriminator inventory is invalid")
    by_ordinal = {
        int(record["source_row"]["case_ordinal"]): record
        for record in inventory
    }
    if any(ordinal not in by_ordinal for ordinal in ordered_ordinals):
        raise ValueError("prototype discriminator case is outside the inventory")
    selected = [by_ordinal[ordinal] for ordinal in ordered_ordinals]
    failure_count = sum(bool(item["target_failure_categories"]) for item in selected)
    if (
        len(selected) != int(discriminator["expected_case_count"])
        or failure_count != int(discriminator["expected_failure_case_count"])
        or len(selected) - failure_count
        != int(discriminator["expected_control_case_count"])
    ):
        raise ValueError("prototype discriminator counts differ from profile")
    return selected


def _selection_record(row: dict[str, Any]) -> dict[str, Any]:
    return {
        "clip_id": row["clip_id"],
        "source_row": row,
        "target_failure_categories": [],
        "control_for": [],
    }


def _load_clip(
    corpus_root: Path, manifest_entry: dict[str, Any]
) -> tuple[dict[str, np.ndarray], dict[str, Any]]:
    artifact_path = corpus_root / manifest_entry["artifact"]["relative_path"]
    if _sha256(artifact_path) != manifest_entry["artifact"]["sha256"]:
        raise ValueError("source clip artifact hash mismatch")
    with np.load(artifact_path, allow_pickle=False) as archive:
        metadata = json.loads(archive["metadata_json_utf8"].tobytes())
        arrays = {
            name: np.array(archive[name], copy=True)
            for name in archive.files
            if name != "metadata_json_utf8"
        }
    return arrays, metadata


def _validate_inputs(
    *,
    profile: dict[str, Any],
    profile_path: Path,
    source_audit: dict[str, Any],
    source_audit_path: Path,
    descriptor_path: Path,
    corpus_manifest: dict[str, Any],
    manifest_path: Path,
) -> None:
    source = profile.get("source", {})
    prototype_id = profile.get("prototype_id")
    projection = profile.get("projection", {})
    discriminator = profile.get("discriminator")
    version_shape_is_valid = (
        prototype_id == "nextengine.humanoid-contact-manifold-prototype.v1"
        and "collider_closure" not in projection
        and discriminator is None
    ) or (
        prototype_id == "nextengine.humanoid-contact-manifold-prototype.v2"
        and isinstance(projection.get("collider_closure"), dict)
        and isinstance(discriminator, dict)
        and discriminator.get("ordered_source_case_ordinals")
        == [3749, 3750, 3753, 8144]
    ) or (
        prototype_id == "nextengine.humanoid-contact-manifold-prototype.v3"
        and isinstance(projection.get("collider_closure"), dict)
        and isinstance(discriminator, dict)
        and discriminator.get("ordered_source_case_ordinals")
        == [3749, 3750, 3753, 8144]
    ) or (
        prototype_id == "nextengine.humanoid-contact-manifold-prototype.v4"
        and isinstance(projection.get("collider_closure"), dict)
        and isinstance(discriminator, dict)
        and discriminator.get("ordered_source_case_ordinals")
        == [3749, 3750, 3753, 8144]
    ) or (
        prototype_id == "nextengine.humanoid-contact-manifold-prototype.v5"
        and isinstance(projection.get("collider_closure"), dict)
        and isinstance(discriminator, dict)
        and discriminator.get("ordered_source_case_ordinals")
        == [3749, 3750, 3753, 8144]
    )
    if (
        profile.get("schema_version") != 1
        or profile.get("status") != "FrozenResearchOnly"
        or not version_shape_is_valid
        or source_audit.get("check")
        != "TRAIN-4-ISAAC-EXHAUSTIVE-DYNAMIC-REFERENCE-FEASIBILITY"
        or source_audit.get("status") != "FAIL"
        or _sha256(source_audit_path) != source.get("audit_sha256")
        or _sha256(descriptor_path) != source.get("descriptor_sha256")
        or _sha256(manifest_path)
        != source.get("corpus_manifest_file_sha256")
        or corpus_manifest.get("manifest_sha256")
        != source.get("corpus_manifest_sha256")
        or corpus_manifest.get("profile", {}).get("sha256")
        != source.get("corpus_profile_sha256")
        or source_audit.get("scope", {}).get("horizon_motor_ticks") != 11
        or source_audit.get("scope", {}).get("repeat_count_per_start_phase")
        != 1
        or profile["acceptance"].get("optimizer_authorized") is not False
        or profile["acceptance"].get("full_corpus_build_authorized_by_this_profile")
        is not False
    ):
        raise ValueError(
            f"prototype input closure is invalid for {profile_path.name}"
        )


def _tolerances(projection: dict[str, Any]) -> ContactManifoldTolerances:
    return ContactManifoldTolerances(
        maximum_tangential_step_micrometres=int(
            projection["maximum_tangential_step_micrometres"]
        ),
        maximum_normal_step_micrometres=int(
            projection["maximum_normal_step_micrometres"]
        ),
        maximum_normal_residual_micrometres=int(
            projection["maximum_normal_residual_micrometres"]
        ),
        maximum_mode_inference_height_micrometres=int(
            projection["maximum_mode_inference_height_micrometres"]
        ),
        maximum_mode_inference_speed_micrometres_per_second=int(
            projection["maximum_mode_inference_speed_micrometres_per_second"]
        ),
        maximum_mode_retention_height_micrometres=int(
            projection["maximum_mode_retention_height_micrometres"]
        ),
        maximum_mode_retention_speed_micrometres_per_second=int(
            projection["maximum_mode_retention_speed_micrometres_per_second"]
        ),
        minimum_mode_on_frames=int(projection["minimum_mode_on_frames"]),
        minimum_mode_off_frames=int(projection["minimum_mode_off_frames"]),
    )


def _collider_closure(projection: dict[str, Any]) -> ColliderClosure | None:
    document = projection.get("collider_closure")
    if document is None:
        return None
    algorithm_id = document.get("algorithm_id")
    if (
        not isinstance(document, dict)
        or algorithm_id
        not in {
            "nextengine.bounded-flight-collider-closure.v1",
            "nextengine.bounded-support-precedence-closure.v2",
            "nextengine.support-conditioned-collider-closure.v3",
            "nextengine.final-contact-root-closure.v4",
        }
        or (
            algorithm_id != "nextengine.final-contact-root-closure.v4"
            and document.get("root_vertical_policy")
            != "upward-only residual all-collider floor after leg-chain correction"
        )
        or (
            algorithm_id == "nextengine.final-contact-root-closure.v4"
            and document.get("root_vertical_policy")
            != (
                "upward-only all-collider floor plus final 60 Hz root-velocity "
                "closure after active-contact reprojection"
            )
        )
        or (
            algorithm_id == "nextengine.bounded-flight-collider-closure.v1"
            and any(
                field in document
                for field in (
                    "active_contact_anchor_target_micrometres",
                    "unsupported_flight_clearance_target_micrometres",
                    "unsupported_correction_smoothing_passes",
                    "final_contact_root_velocity_closure_enabled",
                )
            )
        )
        or (
            algorithm_id == "nextengine.bounded-support-precedence-closure.v2"
            and (
                document.get("active_contact_anchor_target_micrometres") != 0
                or "unsupported_flight_clearance_target_micrometres" in document
                or "unsupported_correction_smoothing_passes" in document
                or "final_contact_root_velocity_closure_enabled" in document
            )
        )
        or (
            algorithm_id == "nextengine.support-conditioned-collider-closure.v3"
            and (
                document.get("active_contact_anchor_target_micrometres") != 0
                or document.get(
                    "unsupported_flight_clearance_target_micrometres"
                )
                != 5_000
                or document.get("unsupported_correction_smoothing_passes")
                != 2
                or "final_contact_root_velocity_closure_enabled" in document
            )
        )
        or (
            algorithm_id == "nextengine.final-contact-root-closure.v4"
            and (
                document.get("active_contact_anchor_target_micrometres") != 0
                or document.get(
                    "unsupported_flight_clearance_target_micrometres"
                )
                != 5_000
                or document.get("unsupported_correction_smoothing_passes")
                != 2
                or document.get("correction_smoothing_passes") != 8
                or document.get(
                    "final_contact_root_velocity_closure_enabled"
                )
                is not True
            )
        )
    ):
        raise ValueError("collider-closure profile identity is invalid")
    suffixes = tuple(document["ordered_joint_suffixes"])
    bounds = document["joint_bounds_microradians"]
    closure = ColliderClosure(
        minimum_collider_height_micrometres=document[
            "minimum_collider_height_micrometres"
        ],
        swing_clearance_target_micrometres=document[
            "swing_clearance_target_micrometres"
        ],
        maximum_root_vertical_velocity_micrometres_per_second=document[
            "maximum_root_vertical_velocity_micrometres_per_second"
        ],
        joint_velocity_limit_basis_points=document[
            "joint_velocity_limit_basis_points"
        ],
        ordered_joint_suffixes=suffixes,
        joint_bounds_microradians=tuple(
            tuple(
                tuple(bounds[f"joint.{side}-{suffix}"])
                for suffix in suffixes
            )
            for side in ("left", "right")
        ),
        outer_iterations=document["outer_iterations"],
        jacobian_probe_microradians=document[
            "jacobian_probe_microradians"
        ],
        maximum_joint_update_microradians=document[
            "maximum_joint_update_microradians"
        ],
        correction_smoothing_kernel_weights=tuple(
            document["correction_smoothing_kernel_weights"]
        ),
        correction_smoothing_passes=document[
            "correction_smoothing_passes"
        ],
        active_contact_anchor_target_micrometres=document.get(
            "active_contact_anchor_target_micrometres"
        ),
        unsupported_flight_clearance_target_micrometres=document.get(
            "unsupported_flight_clearance_target_micrometres"
        ),
        unsupported_correction_smoothing_passes=document.get(
            "unsupported_correction_smoothing_passes"
        ),
        final_contact_root_velocity_closure_enabled=document.get(
            "final_contact_root_velocity_closure_enabled", False
        ),
    )
    closure.validate()
    return closure


def _deterministic_npz_bytes(
    arrays: dict[str, np.ndarray], metadata: dict[str, Any]
) -> bytes:
    values = dict(arrays)
    values["metadata_json_utf8"] = np.frombuffer(
        _canonical_json(metadata), dtype=np.uint8
    )
    output = io.BytesIO()
    with zipfile.ZipFile(
        output, mode="w", compression=zipfile.ZIP_DEFLATED, compresslevel=9
    ) as archive:
        for name in sorted(values):
            payload = io.BytesIO()
            np.lib.format.write_array(
                payload, np.asarray(values[name]), allow_pickle=False
            )
            entry = zipfile.ZipInfo(
                f"{name}.npy", date_time=(1980, 1, 1, 0, 0, 0)
            )
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = 0o100644 << 16
            archive.writestr(
                entry,
                payload.getvalue(),
                compress_type=zipfile.ZIP_DEFLATED,
                compresslevel=9,
            )
    return output.getvalue()


def _external_directory(candidate: Path) -> Path:
    output = candidate.resolve()
    repository = Path(__file__).resolve().parents[2]
    if output == repository or repository in output.parents:
        raise ValueError("contact prototype bundle must stay outside the repository")
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(output)
    return output


def _canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _repository_state() -> dict[str, Any]:
    repository = Path(__file__).resolve().parents[2]
    commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repository,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    dirty_paths = subprocess.run(
        ["git", "status", "--short"],
        cwd=repository,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    return {
        "commit": commit,
        "dirty": bool(dirty_paths),
        "dirty_paths": dirty_paths,
    }


if __name__ == "__main__":
    main()
