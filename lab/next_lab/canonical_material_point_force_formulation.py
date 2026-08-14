from __future__ import annotations

import hashlib
import json
import struct
from collections import Counter
from collections.abc import Mapping
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.dynamics_model_identity_preflight import (
    CHECK_ID as R109_CHECK_ID,
)
from next_lab.dynamics_model_identity_preflight import (
    PREFLIGHT_ID as R109_PREFLIGHT_ID,
)
from next_lab.dynamics_model_identity_preflight import (
    _validate_profile as _validate_r109_profile,
)
from next_lab.motor_mirror import validate_biomechanics_descriptor

FORMULATION_ID = "nextengine.humanoid-canonical-material-point-force-repair.v1"
CHECK_ID = "TRAIN-4-CANONICAL-MATERIAL-POINT-FORCE-REPAIR-FORMULATION"

TRACKED_SOURCE_PATHS = {
    "spec_26": (
        "docs/architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md"
    ),
    "physics_descriptors": "crates/contracts/src/physics/descriptors.rs",
    "physics_catalog": "crates/contracts/src/physics/catalog.rs",
    "compiler_v2": "crates/motor/src/compiler_v2.rs",
    "mirror_v2": "crates/motor/src/mirror_v2.rs",
    "physics_physx": "crates/physics-physx/src/lib.rs",
    "physx_ffi_lib": "crates/physics-physx-ffi/src/lib.rs",
    "physx_bridge": ("crates/physics-physx-ffi/native/nextengine_physx_bridge.cpp"),
    "usd_translation": "lab/next_lab/usd_translation.py",
    "isaac_reference_env": "lab/next_lab/isaac_reference_env.py",
    "contact_manifold": "lab/next_lab/contact_manifold.py",
}

EXPECTED_R109_BLOCKERS = (
    "CANONICAL_MATERIAL_DESCRIPTORS_ABSENT",
    "BACKEND_MATERIAL_DEFAULTS_DIVERGE",
    "CONTACT_WRENCH_CONVENTION_UNOWNED",
)
EXPECTED_EFFECTOR_ORDER = (
    "effector.left-forefoot",
    "effector.left-heel",
    "effector.left-palm",
    "effector.right-forefoot",
    "effector.right-heel",
    "effector.right-palm",
)


def tracked_source_paths(repository_root: Path) -> dict[str, Path]:
    root = repository_root.resolve()
    return {name: root / relative for name, relative in TRACKED_SOURCE_PATHS.items()}


def build_canonical_material_point_force_formulation(
    *,
    profile_path: Path,
    r109_report_path: Path,
    r109_profile_path: Path,
    descriptor_path: Path,
    v9_profile_path: Path,
    v9_manifest_path: Path,
    v9_complete_clip_path: Path,
    physx_sdk_manifest_path: Path,
    px_material_header_path: Path,
    tracked_sources: Mapping[str, Path],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R110's repair formulation without changing or executing a model."""

    direct_paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r109_report_path,
            r109_profile_path,
            descriptor_path,
            v9_profile_path,
            v9_manifest_path,
            v9_complete_clip_path,
            physx_sdk_manifest_path,
            px_material_header_path,
            tool_path,
        )
    )
    (
        profile_path,
        r109_report_path,
        r109_profile_path,
        descriptor_path,
        v9_profile_path,
        v9_manifest_path,
        v9_complete_clip_path,
        physx_sdk_manifest_path,
        px_material_header_path,
        tool_path,
    ) = direct_paths
    tracked_sources = {name: path.resolve() for name, path in tracked_sources.items()}
    if any(not path.is_file() for path in (*direct_paths, *tracked_sources.values())):
        raise FileNotFoundError("R110 formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    r109 = validate_r109(
        report_path=r109_report_path,
        profile_path=r109_profile_path,
        expected=profile["source"]["r109"],
    )
    if sha256(descriptor_path) != profile["source"]["descriptor_sha256"]:
        raise ValueError("R110 descriptor identity differs")
    descriptor = json.loads(descriptor_path.read_bytes())
    validate_biomechanics_descriptor(descriptor)
    source_identity = validate_source_identity(
        profile=profile,
        tracked_sources=tracked_sources,
    )
    material_repair = audit_material_repair(profile=profile, descriptor=descriptor)
    physx_lineage = audit_physx_lineage(
        profile=profile,
        manifest_path=physx_sdk_manifest_path,
        header_path=px_material_header_path,
    )
    v9_lineage = audit_v9_contact_lineage(
        profile=profile,
        descriptor=descriptor,
        v9_profile_path=v9_profile_path,
        manifest_path=v9_manifest_path,
        complete_clip_path=v9_complete_clip_path,
    )
    current_gap = audit_current_implementation_gap(tracked_sources)

    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "formulation_id": FORMULATION_ID,
        "scope": profile["scope"],
        "research_basis": profile["research_basis"],
        "frozen_invariants": profile["frozen_invariants"],
        "source_stop": {
            "r109_status": r109["status"],
            "r109_gate_decision": r109["gate_decision"],
            "blocking_reasons": r109["model_identity_result"]["blocking_reasons"],
        },
        "current_implementation_gap": current_gap,
        "canonical_material_repair": material_repair,
        "physx_material_lineage": physx_lineage,
        "point_contact_force_coordinates": v9_lineage,
        "implementation_sequence": profile["implementation_sequence"],
        "rollback_and_non_regression": profile["rollback_and_non_regression"],
        "repair_assessment": {
            "status": "FORMULATED_NOT_IMPLEMENTED",
            "r109_blocker_disposition": {
                "CANONICAL_MATERIAL_DESCRIPTORS_ABSENT": (
                    "CLOSED_BY_HASH_BOUND_FORMULATION_PENDING_R111_IMPLEMENTATION"
                ),
                "BACKEND_MATERIAL_DEFAULTS_DIVERGE": (
                    "R111_R112_IMPLEMENTATION_AND_R113_RECHECK_REQUIRED"
                ),
                "CONTACT_WRENCH_CONVENTION_UNOWNED": (
                    "REPLACED_BY_HASH_BOUND_SOLVER_PRIVATE_POINT_CONTACT_FORCES"
                ),
            },
            "model_identity_pass_claim": False,
            "runtime_correspondence_claim": False,
        },
        "next_smallest_action": profile["decision"]["next_smallest_action"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r109_report_sha256": r109["report_sha256"],
            "r109_report_file_sha256": sha256(r109_report_path),
            "r109_profile_sha256": sha256(r109_profile_path),
            "descriptor_sha256": sha256(descriptor_path),
            "v9_profile_sha256": sha256(v9_profile_path),
            "v9_manifest_file_sha256": sha256(v9_manifest_path),
            "v9_complete_clip_sha256": sha256(v9_complete_clip_path),
            "physx_sdk_manifest_file_sha256": sha256(physx_sdk_manifest_path),
            "px_material_header_sha256": sha256(px_material_header_path),
            "tracked_source_sha256": source_identity,
            "tool_sha256": sha256(tool_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "research_reports": 1,
        "material_repair_formulations": 1,
        "point_force_formulations": 1,
        "runtime_changes": 0,
        "model_identity_preflights": 0,
        "solver_runs": 0,
        "kto_solves": 0,
        "inverse_dynamics_solves": 0,
        "kinodynamic_solves": 0,
        "candidate_target_constructions": 0,
        "candidate_artifacts_built": 0,
        "offline_candidate_evaluations": 0,
        "physx_runs": 0,
        "candidate_evaluations": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def validate_r109(
    *,
    report_path: Path,
    profile_path: Path,
    expected: Mapping[str, Any],
) -> dict[str, Any]:
    if sha256(report_path) != expected.get("report_file_sha256"):
        raise ValueError("R109 report file identity differs")
    if sha256(profile_path) != expected.get("profile_sha256"):
        raise ValueError("R109 profile identity differs")
    r109_profile = json.loads(profile_path.read_bytes())
    _validate_r109_profile(r109_profile)
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    blockers = report.get("model_identity_result", {}).get("blocking_reasons")
    if (
        report.get("check") != R109_CHECK_ID
        or report.get("preflight_id") != R109_PREFLIGHT_ID
        or report.get("status") != "FAIL"
        or report.get("gate_decision") != "STOP_INVALID_MODEL_LINEAGE"
        or embedded != expected.get("report_sha256")
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or blockers != list(EXPECTED_R109_BLOCKERS)
        or report.get("repository", {}).get("dirty") is not False
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "solver_runs",
                "kto_solves",
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "candidate_target_constructions",
                "candidate_artifacts_built",
                "physx_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R109 stop contract differs")
    return report


def validate_source_identity(
    *,
    profile: Mapping[str, Any],
    tracked_sources: Mapping[str, Path],
) -> dict[str, str]:
    if set(tracked_sources) != set(TRACKED_SOURCE_PATHS):
        raise ValueError("R110 tracked source set differs")
    expected = {
        "spec_26": profile["source"]["architecture"]["spec_26_sha256"],
        **{
            key.removesuffix("_sha256"): value
            for key, value in profile["source"]["repository_sources"].items()
        },
    }
    actual = {name: sha256(path) for name, path in tracked_sources.items()}
    if actual != expected:
        raise ValueError("R110 tracked source identity differs")
    return actual


def audit_material_repair(
    *, profile: Mapping[str, Any], descriptor: Mapping[str, Any]
) -> dict[str, Any]:
    repair = profile["canonical_material_repair"]
    material_rows = repair["material_descriptors"]
    material_ids = [row["material_id"] for row in material_rows]
    if material_ids != sorted(material_ids, key=lambda value: value.encode("utf-8")):
        raise ValueError("R110 material descriptor order is non-canonical")
    if len(material_ids) != len(set(material_ids)):
        raise ValueError("R110 material descriptor identity is duplicated")
    coefficient_rows = []
    for row in material_rows:
        coefficients = tuple(
            int(row[key])
            for key in (
                "static_friction_q16",
                "dynamic_friction_q16",
                "restitution_q16",
            )
        )
        if (
            row.get("descriptor_revision") != 1
            or coefficients[1] > coefficients[0]
            or coefficients[2] > 65_536
            or any(value < 0 for value in coefficients)
            or row.get("canonical_material_tags") != []
        ):
            raise ValueError("R110 material descriptor is invalid")
        coefficient_rows.append(coefficients)

    source_mapping = repair["source_float_mapping"]
    quantization = {}
    for name in ("static_friction", "dynamic_friction", "restitution"):
        row = source_mapping[name]
        source = float(row["source_decimal"])
        source_f32 = struct.unpack("<f", struct.pack("<f", source))[0]
        source_f32_bits = struct.unpack("<I", struct.pack("<f", source))[0]
        q16 = round(source * 65_536)
        if source_f32_bits != row["source_f32_bits"] or q16 != row["q16"]:
            raise ValueError("R110 source material quantization differs")
        quantization[name] = {
            "source_f32_bits": source_f32_bits,
            "q16": q16,
            "exact_q16_value": q16 / 65_536,
            "q16_minus_source_f32": q16 / 65_536 - source_f32,
        }

    counts = Counter(
        collider["material_id"]
        for body in descriptor["bodies"]
        for collider in body["colliders"]
    )
    expected_counts = {
        "physics-material.humanoid-body.v1": 17,
        "physics-material.humanoid-sole.v1": 2,
    }
    if counts != expected_counts:
        raise ValueError("R110 source collider material assignment differs")
    assignment_ids = set(repair["assignments"].values())
    if assignment_ids != set(material_ids):
        raise ValueError("R110 material assignment closure differs")
    if len(set(coefficient_rows)) != 1:
        raise ValueError("R110 current material coefficients must be identical")

    combine = repair["combine_profile"]
    expected_rule = "ArithmeticMeanTiesToEven"
    if any(
        combine.get(key) != expected_rule
        for key in ("static_friction", "dynamic_friction", "restitution")
    ):
        raise ValueError("R110 combine profile differs")
    table_root = hashlib.sha256(
        canonical_json(
            {
                "material_descriptors": material_rows,
                "combine_profile": combine,
            }
        )
    ).hexdigest()
    return {
        **repair,
        "source_collider_material_id_counts": dict(sorted(counts.items())),
        "all_current_coefficients_equal": True,
        "quantization_audit": quantization,
        "material_and_combine_table_sha256": table_root,
        "status": "FORMULATED_NEW_LINEAGE",
        "runtime_implemented": False,
    }


def audit_physx_lineage(
    *,
    profile: Mapping[str, Any],
    manifest_path: Path,
    header_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["physx_sdk"]
    if (
        sha256(manifest_path) != expected["manifest_file_sha256"]
        or sha256(header_path) != expected["px_material_header_sha256"]
    ):
        raise ValueError("R110 PhysX SDK file identity differs")
    manifest = json.loads(manifest_path.read_bytes())
    for key in (
        "physx_version",
        "source_revision",
        "bridge_abi",
        "profile_hash",
    ):
        if manifest.get(key) != expected[key]:
            raise ValueError("R110 PhysX SDK manifest differs")
    header = header_path.read_text(encoding="utf-8")
    required_tokens = (
        "eAVERAGE",
        "eMIN",
        "eMULTIPLY",
        "eMAX",
        "PxCombineMode::eAVERAGE",
        "setFrictionCombineMode",
        "setRestitutionCombineMode",
    )
    if any(token not in header for token in required_tokens):
        raise ValueError("R110 PhysX material API evidence differs")
    return {
        "status": "PINNED_API_SUPPORTS_EXPLICIT_REPAIR",
        "physx_version": manifest["physx_version"],
        "source_revision": manifest["source_revision"],
        "bridge_abi": manifest["bridge_abi"],
        "profile_hash": manifest["profile_hash"],
        "supported_combine_modes": ["average", "minimum", "multiply", "maximum"],
        "physx_default_combine_mode": "average",
        "selected_combine_mode": "average",
        "default_is_authority": False,
    }


def audit_v9_contact_lineage(
    *,
    profile: Mapping[str, Any],
    descriptor: Mapping[str, Any],
    v9_profile_path: Path,
    manifest_path: Path,
    complete_clip_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["v9_contact_lineage"]
    if (
        sha256(v9_profile_path) != expected["prototype_profile_sha256"]
        or sha256(manifest_path) != expected["prototype_manifest_file_sha256"]
        or sha256(complete_clip_path) != expected["complete_clip_artifact_sha256"]
    ):
        raise ValueError("R110 V9 contact file identity differs")
    manifest = json.loads(manifest_path.read_bytes())
    embedded = manifest.get("manifest_sha256")
    without_hash = dict(manifest)
    without_hash.pop("manifest_sha256", None)
    if (
        manifest.get("status") != "PASS"
        or embedded != expected["prototype_manifest_sha256"]
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
    ):
        raise ValueError("R110 V9 contact manifest differs")
    clip_row = next(
        (
            row
            for row in manifest.get("complete_clips", ())
            if row.get("clip_id") == "cmu16-walk-nominal-b"
        ),
        None,
    )
    if (
        clip_row is None
        or clip_row.get("artifact", {}).get("sha256")
        != expected["complete_clip_artifact_sha256"]
        or clip_row.get("projection_diagnostics", {}).get(
            "contact_point_deletion_count"
        )
        != 0
    ):
        raise ValueError("R110 V9 complete clip row differs")
    with np.load(complete_clip_path, allow_pickle=False) as arrays:
        metadata = json.loads(arrays["metadata_json_utf8"].tobytes().decode("utf-8"))
        modes = np.asarray(arrays["contact_modes"], dtype=np.uint8)
    point_coordinates = audit_point_force_coordinates(
        profile=profile,
        descriptor=descriptor,
        metadata=metadata,
        contact_modes=modes,
    )
    return {
        **profile["point_contact_force_coordinates"],
        "source_clip": {
            "artifact_id": metadata["artifact_id"],
            "clip_id": metadata["clip_id"],
            "frame_first": int(metadata["frame_first"]),
            "frame_last": int(metadata["frame_last"]),
            "frame_count": len(modes),
            "contact_mode_counts": point_coordinates["contact_mode_counts"],
            "contact_point_deletion_count": 0,
        },
        "application_point_identity": point_coordinates["application_point_identity"],
        "status": "FROZEN_SOLVER_PRIVATE_POINT_FORCE_COORDINATES",
        "runtime_contract_change": False,
    }


def audit_point_force_coordinates(
    *,
    profile: Mapping[str, Any],
    descriptor: Mapping[str, Any],
    metadata: Mapping[str, Any],
    contact_modes: NDArray[np.uint8],
) -> dict[str, Any]:
    coordinates = profile["point_contact_force_coordinates"]
    if (
        tuple(metadata.get("effector_ids", ())) != EXPECTED_EFFECTOR_ORDER
        or metadata.get("clip_id") != "cmu16-walk-nominal-b"
        or metadata.get("frame_first") != 0
        or metadata.get("frame_last") != 800
        or contact_modes.shape != (801, 2)
        or np.any(contact_modes > 3)
    ):
        raise ValueError("R110 contact schedule identity differs")
    effector_by_id = {
        row["effector_id"]: row for row in descriptor.get("effectors", ())
    }
    ordered_points = coordinates["ordered_points"]
    expected_points = (
        (0, "effector.left-heel", "body.left-ankle-roll", 0, 0, [1, 3]),
        (1, "effector.left-forefoot", "body.left-ankle-roll", 0, 1, [2, 3]),
        (2, "effector.right-heel", "body.right-ankle-roll", 1, 0, [1, 3]),
        (3, "effector.right-forefoot", "body.right-ankle-roll", 1, 1, [2, 3]),
    )
    application_identity = []
    for row, expected in zip(ordered_points, expected_points, strict=True):
        actual = (
            row.get("point_ordinal"),
            row.get("effector_id"),
            row.get("body_id"),
            row.get("side_index"),
            row.get("point_index"),
            row.get("active_mode_bits"),
        )
        if actual != expected:
            raise ValueError("R110 ordered contact point differs")
        effector = effector_by_id.get(row["effector_id"])
        if effector is None or effector.get("body_id") != row["body_id"]:
            raise ValueError("R110 contact application body differs")
        application_identity.append(
            {
                "point_ordinal": row["point_ordinal"],
                "effector_id": row["effector_id"],
                "body_id": row["body_id"],
                "local_translation_micrometres": effector[
                    "local_translation_micrometres"
                ],
            }
        )
    counts = Counter(int(value) for value in contact_modes.ravel())
    return {
        "contact_mode_counts": {
            str(mode): int(counts.get(mode, 0)) for mode in range(4)
        },
        "application_point_identity": application_identity,
    }


def audit_current_implementation_gap(
    tracked_sources: Mapping[str, Path],
) -> dict[str, Any]:
    descriptors = tracked_sources["physics_descriptors"].read_text(encoding="utf-8")
    catalog = tracked_sources["physics_catalog"].read_text(encoding="utf-8")
    compiler = tracked_sources["compiler_v2"].read_text(encoding="utf-8")
    bridge = tracked_sources["physx_bridge"].read_text(encoding="utf-8")
    usd = tracked_sources["usd_translation"].read_text(encoding="utf-8")
    isaac = tracked_sources["isaac_reference_env"].read_text(encoding="utf-8")
    result = {
        "physics_material_descriptor_v1_present": (
            "struct PhysicsMaterialDescriptorV1" in descriptors
        ),
        "world_material_catalog_present": (
            "pub materials: BTreeMap<SchemaId, PhysicsMaterialDescriptorV1>" in catalog
        ),
        "material_combine_contract_present": (
            "PhysicsMaterialCombineProfileV1" in descriptors
            or "PhysicsMaterialCombineProfileV1" in catalog
        ),
        "compiled_v2_material_catalog_present": (
            "materials: BTreeMap" in compiler or "material_descriptors:" in compiler
        ),
        "native_hard_coded_material_present": (
            "createMaterial(0.8F, 0.7F, 0.0F)" in bridge
        ),
        "usd_physics_material_binding_present": (
            "PhysicsMaterialAPI" in usd and "MaterialBindingAPI" in usd
        ),
        "isaac_default_ground_plane_present": ("GroundPlaneCfg()" in isaac),
    }
    if result != {
        "physics_material_descriptor_v1_present": True,
        "world_material_catalog_present": True,
        "material_combine_contract_present": False,
        "compiled_v2_material_catalog_present": False,
        "native_hard_coded_material_present": True,
        "usd_physics_material_binding_present": False,
        "isaac_default_ground_plane_present": True,
    }:
        raise ValueError("R110 current implementation gap differs")
    return {**result, "status": "REPAIR_NOT_IMPLEMENTED"}


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    invariants = profile.get("frozen_invariants", {})
    repair = profile.get("canonical_material_repair", {})
    point_force = profile.get("point_contact_force_coordinates", {})
    decision = profile.get("decision", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("claim") != "CanonicalMaterialAndPointForceRepairFormulationOnly"
        or scope.get("run_id") != "R110"
        or scope.get("initial_clip_id") != "cmu16-walk-nominal-b"
        or scope.get("initial_source_case_ordinal") != 7967
        or scope.get("body_count") != 24
        or scope.get("joint_count") != 23
        or scope.get("actuator_count") != 23
        or scope.get("motor_hz") != 60
        or scope.get("physics_hz") != 240
        or scope.get("research_reports") != 1
        or scope.get("runtime_changes") is not False
        or scope.get("solver_runs") != 0
        or scope.get("candidate_construction") is not False
        or scope.get("physx_execution") is not False
        or invariants.get("controller")
        != "fixed zero-residual PD with zero desired velocity and descriptor gains/caps"
        or invariants.get("fresh_scene_authority") != "ADR-070"
        or invariants.get("partial_reset") != "REPORT_ONLY"
        or invariants.get("contact_schedule_changes") != "FORBIDDEN"
        or invariants.get("contact_point_deletion") != "FORBIDDEN"
        or invariants.get("limit_changes") != "FORBIDDEN"
        or repair.get("coefficient_encoding") != "unsigned Q16 exact integers"
        or len(repair.get("material_descriptors", ())) != 3
        or repair.get("lineage_policy", {}).get("compiled_descriptor_hash")
        != "MUST_CHANGE"
        or repair.get("lineage_policy", {}).get("old_runtime_equivalence_claim")
        != "FORBIDDEN"
        or point_force.get("engine_world_axes") != "+X right, +Y up, +Z forward"
        or point_force.get("force_component_order")
        != ["normal_newtons", "right_newtons", "forward_newtons"]
        or len(point_force.get("ordered_points", ())) != 4
        or decision.get("complete")
        != "PERMIT_R111_CANONICAL_MATERIAL_LINEAGE_IMPLEMENTATION_ONLY"
        or bounded.get("r111_canonical_material_lineage_implementation") != "AUTHORIZED"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "r112_usd_isaac_material_lineage_implementation",
                "r113_model_identity_preflight",
                "quantization_aware_kto_formulation",
                "quantization_aware_kto_solve",
                "inverse_dynamics_solve",
                "kinodynamic_solve",
                "candidate_target_construction",
                "candidate_artifact",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
    ):
        raise ValueError("R110 repair formulation profile differs")
