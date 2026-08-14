from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

from next_lab.contact_target_knot_formulation import canonical_json, sha256

AUDIT_ID = "nextengine.humanoid-canonical-material-lineage-implementation.v1"
CHECK_ID = "TRAIN-4-CANONICAL-MATERIAL-LINEAGE-IMPLEMENTATION"

TRACKED_SOURCE_PATHS = {
    "adr_071": "docs/architecture/adr/071-canonical-physics-material-lineage.md",
    "spec_26": "docs/architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md",
    "spec_35": "docs/architecture/35-deterministic-humanoid-training-substrate.md",
    "material_contract": "crates/contracts/src/physics/material.rs",
    "compiler_v3": "crates/motor/src/compiler_v3.rs",
    "mirror_v2": "crates/motor/src/mirror_v2.rs",
    "physics_physx": "crates/physics-physx/src/lib.rs",
    "physx_ffi": "crates/physics-physx-ffi/src/lib.rs",
    "physx_bridge": "crates/physics-physx-ffi/native/nextengine_physx_bridge.cpp",
    "physx_build": "crates/physics-physx-ffi/build.rs",
    "physx_xtask": "tools/xtask/src/physx.rs",
    "legacy_mirror_v1": "lab/tests/fixtures/biomechanics_motor_mirror_v1.json",
    "usd_translation": "lab/next_lab/usd_translation.py",
    "isaac_reference_env": "lab/next_lab/isaac_reference_env.py",
}


def tracked_source_paths(repository_root: Path) -> dict[str, Path]:
    root = repository_root.resolve()
    return {name: root / relative for name, relative in TRACKED_SOURCE_PATHS.items()}


def build_canonical_material_lineage_implementation(
    *,
    profile_path: Path,
    r110_report_path: Path,
    r110_profile_path: Path,
    physx_sdk_manifest_path: Path,
    mirror_v2_bytes: bytes,
    tracked_sources: Mapping[str, Path],
    validation_results: Sequence[Mapping[str, Any]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    paths = (
        profile_path.resolve(),
        r110_report_path.resolve(),
        r110_profile_path.resolve(),
        physx_sdk_manifest_path.resolve(),
        tool_path.resolve(),
    )
    profile_path, r110_report_path, r110_profile_path, manifest_path, tool_path = paths
    sources = {name: path.resolve() for name, path in tracked_sources.items()}
    if any(not path.is_file() for path in (*paths, *sources.values())):
        raise FileNotFoundError("R111 implementation-audit input is absent")
    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(profile, repository)
    r110 = _validate_r110(profile, r110_report_path, r110_profile_path)
    source_audit = audit_implementation_sources(
        {name: path.read_text(encoding="utf-8") for name, path in sources.items()}
    )
    mirror_audit = audit_mirror_v2(profile, json.loads(mirror_v2_bytes))
    sdk_audit = _audit_sdk(profile, manifest_path)
    validations = _validate_results(profile, validation_results)
    source_hashes = {name: sha256(path) for name, path in sources.items()}
    if (
        source_hashes["legacy_mirror_v1"]
        != profile["source"]["legacy_mirror_v1_sha256"]
    ):
        raise ValueError("R111 legacy mirror V1 identity changed")

    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "audit_id": AUDIT_ID,
        "status": "PASS",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["pass"],
        "scope": profile["scope"],
        "frozen_invariants": profile["frozen_invariants"],
        "source_formulation": {
            "r110_status": r110["status"],
            "r110_gate_decision": r110["gate_decision"],
            "r110_report_sha256": r110["report_sha256"],
        },
        "architecture_and_schema_boundary": {
            "adr": "ADR-071",
            "material_contract": "PhysicsMaterialDescriptorV2",
            "combine_contract": "PhysicsMaterialCombineProfileV1",
            "compiled_descriptor": "CompiledBodySchemaV3",
            "mirror": "nextengine.isaac.biomechanics-mirror.v2",
            "legacy_v1_v2_reinterpreted": False,
            "body_schema_hash_changed": False,
            "status": "COMPLETE",
        },
        "implementation_audit": source_audit,
        "compiled_mirror_lineage": mirror_audit,
        "physx_sdk": sdk_audit,
        "validation_results": validations,
        "r109_blocker_disposition": {
            "CANONICAL_MATERIAL_DESCRIPTORS_ABSENT": "CLOSED_BY_R111",
            "BACKEND_MATERIAL_DEFAULTS_DIVERGE": "NATIVE_SIDE_CLOSED_R111_DERIVED_USD_ISAAC_R112_PENDING",
            "CONTACT_WRENCH_CONVENTION_UNOWNED": "CLOSED_BY_R110_SOLVER_PRIVATE_POINT_FORCES",
        },
        "remaining_model_identity_gap": {
            "derived_usd_material_prims_and_bindings": "ABSENT_R112_REQUIRED",
            "explicit_isaac_ground_material_lineage": "ABSENT_R112_REQUIRED",
            "clean_model_identity_preflight": "R113_REQUIRED_AFTER_R112",
            "model_identity_pass_claim": False,
            "runtime_correspondence_claim": False,
        },
        "next_smallest_action": profile["decision"]["next_smallest_action"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r110_report_file_sha256": sha256(r110_report_path),
            "r110_profile_sha256": sha256(r110_profile_path),
            "mirror_v2_sha256": hashlib.sha256(mirror_v2_bytes).hexdigest(),
            "physx_sdk_manifest_file_sha256": sha256(manifest_path),
            "tracked_source_sha256": source_hashes,
            "tool_sha256": sha256(tool_path),
            "audit_module_sha256": sha256(Path(__file__).resolve()),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "material_lineage_implementations": 1,
        "derived_usd_material_implementations": 0,
        "model_identity_preflights": 0,
        "solver_runs": 0,
        "kto_solves": 0,
        "inverse_dynamics_solves": 0,
        "kinodynamic_solves": 0,
        "candidate_artifacts_built": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def audit_implementation_sources(sources: Mapping[str, str]) -> dict[str, Any]:
    if set(sources) != set(TRACKED_SOURCE_PATHS):
        raise ValueError("R111 tracked source set differs")
    required = {
        "adr_071": ("ADR-071", "Bridge ABI 4", "CompiledBodySchemaV3"),
        "material_contract": (
            "struct PhysicsMaterialDescriptorV2",
            "rolling_friction_q16",
            "spinning_friction_q16",
            "surface_velocity_micrometres_per_second",
            "struct PhysicsMaterialCombineProfileV1",
        ),
        "compiler_v3": (
            "struct CompiledBodySchemaV3",
            "physics-material.humanoid-body.v1",
            "physics-material.humanoid-ground.v1",
            "physics-material.humanoid-sole.v1",
            "52_429",
            "45_875",
            "validate_material_closure",
            "NEXTENGINE_PHYSX_ABI_VERSION",
        ),
        "mirror_v2": (
            "biomechanics_isaac_mirror_descriptor_json_v2",
            "nextengine.isaac.biomechanics-mirror.v2",
            'descriptor["materials"]',
            'descriptor["material_combine_profile"]',
        ),
        "physics_physx": (
            "PhysXSharedMaterialProfileV1",
            "from_material_catalog",
            "descriptor.rolling_friction_q16 != first.rolling_friction_q16",
            "MATERIAL_COEFFICIENT_ENCODING_Q16",
            "legacy_stage0_material_input",
            "PhysXArticulationWorldV3",
        ),
        "physx_ffi": (
            "NEXTENGINE_PHYSX_ABI_VERSION: u32 = 4",
            "struct MaterialProfileInput",
            "!self.material_configured",
            "world_configure_material",
        ),
        "physx_bridge": (
            "kAbiVersion = 4",
            "ne_physx_world_configure_material",
            "createMaterial(static_friction, dynamic_friction, restitution)",
            "setFrictionCombineMode(physx::PxCombineMode::eAVERAGE)",
            "setRestitutionCombineMode(physx::PxCombineMode::eAVERAGE)",
            "setTorsionalPatchRadius(0.0F)",
            "setMinTorsionalPatchRadius(0.0F)",
        ),
        "physx_build": ('\\"bridge_abi\\": 4',),
        "physx_xtask": ("PHYSX_BRIDGE_ABI: u32 = 4",),
    }
    for name, tokens in required.items():
        if any(token not in sources[name] for token in tokens):
            raise ValueError(f"R111 implementation token differs: {name}")
    bridge = sources["physx_bridge"]
    if "createMaterial(0.8F, 0.7F, 0.0F)" in bridge:
        raise ValueError("R111 native ambient material literal remains")
    usd_material_ready = all(
        token in sources["usd_translation"]
        for token in ("PhysicsMaterialDescriptorV2", "material:binding:physics")
    )
    isaac_ground_ready = (
        "GroundPlaneCfg()" not in sources["isaac_reference_env"]
        and "physics-material.humanoid-ground.v1" in sources["isaac_reference_env"]
    )
    if usd_material_ready or isaac_ground_ready:
        raise ValueError("R111 unexpectedly includes the separately gated R112 change")
    return {
        "material_contract_v2_complete": True,
        "combine_profile_v1_complete": True,
        "compiled_descriptor_v3_hash_binds_materials": True,
        "mirror_v2_exports_material_lineage": True,
        "native_ambient_material_removed": True,
        "native_requires_explicit_material_before_scene": True,
        "native_q16_shared_material_projection": True,
        "unequal_materials_fail_closed": True,
        "nonzero_extended_fields_fail_closed": True,
        "torsional_patch_radii_explicit_zero": True,
        "derived_usd_material_lineage_implemented": False,
        "explicit_isaac_ground_lineage_implemented": False,
        "status": "R111_COMPLETE_R112_PENDING",
    }


def audit_mirror_v2(
    profile: Mapping[str, Any], descriptor: Mapping[str, Any]
) -> dict[str, Any]:
    source = profile["source"]
    if (
        descriptor.get("schema_version") != 2
        or descriptor.get("compiled_descriptor_schema_version") != 3
        or descriptor.get("translator_id") != "nextengine.isaac.biomechanics-mirror.v2"
        or descriptor.get("body_schema_hash") != source["body_schema_hash"]
        or descriptor.get("compiled_descriptor_hash")
        != source["compiled_descriptor_v3_hash"]
        or descriptor.get("compiled_descriptor_hash")
        == source["legacy_compiled_descriptor_hash"]
        or descriptor.get("material_lineage_hash") != source["material_lineage_hash"]
        or descriptor.get("ground_material_id") != "physics-material.humanoid-ground.v1"
        or descriptor.get("materials") != profile["expected_materials"]
        or descriptor.get("material_combine_profile")
        != profile["expected_combine_profile"]
        or descriptor.get("collider_material_assignment_counts")
        != profile["expected_collider_material_assignment_counts"]
    ):
        raise ValueError("R111 compiled mirror lineage differs")
    return {
        "body_schema_hash": descriptor["body_schema_hash"],
        "legacy_compiled_descriptor_hash": source["legacy_compiled_descriptor_hash"],
        "compiled_descriptor_v3_hash": descriptor["compiled_descriptor_hash"],
        "material_lineage_hash": descriptor["material_lineage_hash"],
        "mirror_v2_sha256": source["mirror_v2_sha256"],
        "legacy_mirror_v1_sha256": source["legacy_mirror_v1_sha256"],
        "ground_material_id": descriptor["ground_material_id"],
        "material_descriptors": descriptor["materials"],
        "material_combine_profile": descriptor["material_combine_profile"],
        "collider_material_assignment_counts": descriptor[
            "collider_material_assignment_counts"
        ],
        "legacy_bytes_reinterpreted": False,
        "status": "COMPLETE_NEW_LINEAGE",
    }


def _validate_profile(profile: Mapping[str, Any]) -> None:
    if (
        profile.get("schema_version") != 1
        or profile.get("audit_id") != AUDIT_ID
        or profile.get("claim")
        != "CanonicalMaterialLineageImplementationWithoutPhysXExecution"
        or profile.get("scope", {}).get("run_id") != "R111"
        or profile.get("scope", {}).get("physx_scene_runs") != 0
        or profile.get("decision", {}).get("pass")
        != "PERMIT_R112_DERIVED_USD_MATERIAL_LINEAGE_IMPLEMENTATION_ONLY"
        or profile.get("bounded_acceptance", {}).get("physx") != "NOT_AUTHORIZED"
        or profile.get("bounded_acceptance", {}).get("training") != "NOT_AUTHORIZED"
    ):
        raise ValueError("R111 profile differs")
    commands = profile.get("validation_commands", ())
    ids = [row.get("id") for row in commands]
    if (
        not commands
        or len(ids) != len(set(ids))
        or any(not row.get("arguments") for row in commands)
    ):
        raise ValueError("R111 validation command closure differs")


def _validate_repository(
    profile: Mapping[str, Any], repository: Mapping[str, Any]
) -> None:
    if (
        repository.get("dirty") is not False
        or repository.get("architecture_commit_is_ancestor") is not True
        or repository.get("implementation_commit_is_ancestor") is not True
        or repository.get("architecture_commit")
        != profile["source"]["architecture_commit"]
        or repository.get("implementation_commit")
        != profile["source"]["implementation_commit"]
    ):
        raise ValueError("R111 repository lineage differs")


def _validate_r110(
    profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r110"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R111 R110 source identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R111_CANONICAL_MATERIAL_LINEAGE_IMPLEMENTATION_ONLY"
        or embedded != expected["report_sha256"]
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("physx_runs") != 0
        or report.get("training_runs") != 0
    ):
        raise ValueError("R111 R110 source contract differs")
    return report


def _audit_sdk(profile: Mapping[str, Any], manifest_path: Path) -> dict[str, Any]:
    expected = profile["source"]["physx_sdk"]
    if sha256(manifest_path) != expected["manifest_file_sha256"]:
        raise ValueError("R111 PhysX SDK manifest identity differs")
    manifest = json.loads(manifest_path.read_bytes())
    for key in ("physx_version", "source_revision", "bridge_abi", "profile_hash"):
        if manifest.get(key) != expected[key]:
            raise ValueError("R111 PhysX SDK profile differs")
    return {
        **expected,
        "material_input_layout_bytes": 72,
        "native_scene_executed": False,
        "status": "ABI4_PREPARED_AND_COMPILE_ONLY",
    }


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, Any]]
) -> list[dict[str, Any]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R111 validation result differs")
    return [dict(row) for row in results]
