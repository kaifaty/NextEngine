from __future__ import annotations

import hashlib
import json
import re
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

from next_lab.biomechanics_material_lineage import (
    EXPECTED_COMBINE_PROFILE,
    EXPECTED_MATERIALS,
)
from next_lab.canonical_material_point_force_formulation import (
    CHECK_ID as R110_CHECK_ID,
)
from next_lab.canonical_material_point_force_formulation import (
    FORMULATION_ID as R110_FORMULATION_ID,
)
from next_lab.canonical_material_point_force_formulation import (
    _validate_profile as _validate_r110_profile,
)
from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.derived_usd_material_lineage import AUDIT_ID as R112_AUDIT_ID
from next_lab.derived_usd_material_lineage import CHECK_ID as R112_CHECK_ID
from next_lab.derived_usd_material_lineage import (
    _validate_profile as _validate_r112_profile,
)
from next_lab.dynamics_model_identity_preflight import (
    audit_descriptor_identity as audit_structural_descriptor_identity,
)
from next_lab.motor_mirror import validate_current_biomechanics_descriptor
from next_lab.usd_translation import validate_translation_bundle

PREFLIGHT_ID = "nextengine.humanoid-clean-dynamics-model-identity-preflight.v1"
CHECK_ID = "TRAIN-4-CLEAN-DYNAMICS-MODEL-IDENTITY-PREFLIGHT"

TRACKED_SOURCE_PATHS = {
    "spec_26": "docs/architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md",
    "spec_35": "docs/architecture/35-deterministic-humanoid-training-substrate.md",
    "adr_069": "docs/architecture/adr/069-biomechanics-body-schema-v2-and-solver-projection.md",
    "adr_070": "docs/architecture/adr/070-biomechanics-reference-tracking-training-environment.md",
    "adr_071": "docs/architecture/adr/071-canonical-physics-material-lineage.md",
    "biomechanics": "crates/motor/src/biomechanics.rs",
    "compiler_v2": "crates/motor/src/compiler_v2.rs",
    "compiler_v3": "crates/motor/src/compiler_v3.rs",
    "material_contract": "crates/contracts/src/physics/material.rs",
    "physics_physx": "crates/physics-physx/src/lib.rs",
    "physics_physx_material": "crates/physics-physx/src/material.rs",
    "physx_ffi_build": "crates/physics-physx-ffi/build.rs",
    "physx_ffi_lib": "crates/physics-physx-ffi/src/lib.rs",
    "physx_bridge": "crates/physics-physx-ffi/native/nextengine_physx_bridge.cpp",
    "safety_control": "crates/motor/src/safety_control.rs",
    "mirror_v2": "crates/motor/src/mirror_v2.rs",
    "material_lineage": "lab/next_lab/biomechanics_material_lineage.py",
    "motor_mirror": "lab/next_lab/motor_mirror.py",
    "usd_translation": "lab/next_lab/usd_translation.py",
    "isaac_reference_env": "lab/next_lab/isaac_reference_env.py",
}

ISAAC_SOURCE_PATHS = {
    "simulation_cfg": "source/isaaclab/isaaclab/sim/simulation_cfg.py",
    "physics_materials_cfg": (
        "source/isaaclab/isaaclab/sim/spawners/materials/physics_materials_cfg.py"
    ),
    "from_files_cfg": (
        "source/isaaclab/isaaclab/sim/spawners/from_files/from_files_cfg.py"
    ),
}


def tracked_source_paths(repository_root: Path) -> dict[str, Path]:
    root = repository_root.resolve()
    return {name: root / relative for name, relative in TRACKED_SOURCE_PATHS.items()}


def isaac_source_paths(isaac_lab_root: Path) -> dict[str, Path]:
    root = isaac_lab_root.resolve()
    return {name: root / relative for name, relative in ISAAC_SOURCE_PATHS.items()}


def build_clean_dynamics_model_identity_preflight(
    *,
    profile_path: Path,
    r112_report_path: Path,
    r112_profile_path: Path,
    r110_report_path: Path,
    r110_profile_path: Path,
    descriptor_bytes: bytes,
    translation_manifest_path: Path,
    humanoid_usd_path: Path,
    ground_usd_path: Path,
    tracked_sources: Mapping[str, Path],
    isaac_sources: Mapping[str, Path],
    isaac_repository: Mapping[str, Any],
    validation_results: Sequence[Mapping[str, Any]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Audit R113's exact model inputs without constructing a scene or solving."""

    direct_paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r112_report_path,
            r112_profile_path,
            r110_report_path,
            r110_profile_path,
            translation_manifest_path,
            humanoid_usd_path,
            ground_usd_path,
            tool_path,
        )
    )
    (
        profile_path,
        r112_report_path,
        r112_profile_path,
        r110_report_path,
        r110_profile_path,
        translation_manifest_path,
        humanoid_usd_path,
        ground_usd_path,
        tool_path,
    ) = direct_paths
    sources = {name: path.resolve() for name, path in tracked_sources.items()}
    upstream = {name: path.resolve() for name, path in isaac_sources.items()}
    if any(
        not path.is_file()
        for path in (*direct_paths, *sources.values(), *upstream.values())
    ):
        raise FileNotFoundError("R113 model-identity input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    r112 = validate_r112(
        profile=profile,
        report_path=r112_report_path,
        profile_path=r112_profile_path,
    )
    r110 = validate_r110(
        profile=profile,
        report_path=r110_report_path,
        profile_path=r110_profile_path,
    )
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != profile["source"]["mirror_v2_sha256"]
    ):
        raise ValueError("R113 mirror V2 byte identity differs")
    descriptor = json.loads(descriptor_bytes)
    descriptor_identity = audit_descriptor_identity(profile, descriptor)
    translation_identity = audit_translation_identity(
        profile,
        descriptor,
        translation_manifest_path=translation_manifest_path,
        humanoid_usd_path=humanoid_usd_path,
        ground_usd_path=ground_usd_path,
    )
    source_identity = audit_source_identity(
        profile=profile,
        tracked_sources=sources,
        isaac_sources=upstream,
        isaac_repository=isaac_repository,
    )
    point_force_identity = audit_point_force_identity(profile, r110)
    model_identity = classify_model_identity(
        profile=profile,
        descriptor_identity=descriptor_identity,
        translation_identity=translation_identity,
        source_identity=source_identity,
        point_force_identity=point_force_identity,
    )
    validations = _validate_results(profile, validation_results)

    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "preflight_id": PREFLIGHT_ID,
        "status": "PASS",
        "claim": profile["claim"],
        "gate_decision": model_identity["gate_decision"],
        "scope": profile["scope"],
        "required_identity_groups": profile["required_identity_groups"],
        "frozen_invariants": profile["frozen_invariants"],
        "source_gates": {
            "r112_status": r112["status"],
            "r112_gate_decision": r112["gate_decision"],
            "r112_report_sha256": r112["report_sha256"],
            "r110_status": r110["status"],
            "r110_gate_decision": r110["gate_decision"],
            "r110_report_sha256": r110["report_sha256"],
        },
        "architecture_contract": {
            "canonical_material_ownership": "SATISFIED_BY_ADR_071_SUCCESSOR",
            "derived_material_ownership": "SATISFIED_BY_EXACT_R112_BUNDLE",
            "point_force_ownership": "SATISFIED_BY_HASH_BOUND_SOLVER_PRIVATE_R110_FORMULATION",
            "gpu_byte_exact_parity_required": False,
            "runtime_correspondence_claim": False,
        },
        "descriptor_identity": descriptor_identity,
        "translation_identity": translation_identity,
        "shared_identity": {
            **profile["expected_shared_identity"],
            "status": "COMPLETE_BACKEND_NEUTRAL_MODEL_INPUT",
            "usd_projection": "BYTE_EXACT",
            "gravity_coordinate_transform": "EXACT",
            "fixed_pd_controller_semantics": "HASH_BOUND_AND_MIRRORED",
            "runtime_numerical_equivalence_claim": False,
        },
        "backend_profiles": {
            "canonical_native": profile["expected_native_backend"],
            "isaac_correspondence_mirror": profile["expected_isaac_backend"],
        },
        "declared_backend_divergences": _declared_backend_divergences(),
        "point_contact_force_identity": point_force_identity,
        "source_identity": source_identity,
        "model_identity_result": model_identity,
        "validation_results": validations,
        "next_smallest_action": profile["decision"]["next_smallest_action"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r112_report_file_sha256": sha256(r112_report_path),
            "r112_profile_sha256": sha256(r112_profile_path),
            "r110_report_file_sha256": sha256(r110_report_path),
            "r110_profile_sha256": sha256(r110_profile_path),
            "mirror_v2_sha256": hashlib.sha256(descriptor_bytes).hexdigest(),
            "translation_manifest_file_sha256": sha256(translation_manifest_path),
            "humanoid_usd_sha256": sha256(humanoid_usd_path),
            "ground_usd_sha256": sha256(ground_usd_path),
            "tracked_source_sha256": source_identity["tracked_source_sha256"],
            "isaac_source_sha256": source_identity["isaac_source_sha256"],
            "tool_sha256": sha256(tool_path),
            "preflight_module_sha256": sha256(Path(__file__).resolve()),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "model_identity_preflights": 1,
        "solver_runs": 0,
        "kto_solves": 0,
        "inverse_dynamics_solves": 0,
        "kinodynamic_solves": 0,
        "candidate_target_constructions": 0,
        "candidate_artifacts_built": 0,
        "offline_candidate_evaluations": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
        "isaac_repository": dict(isaac_repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def validate_r112(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r112"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R113 R112 source identity differs")
    source_profile = json.loads(profile_path.read_bytes())
    _validate_r112_profile(source_profile)
    report = json.loads(report_path.read_bytes())
    _validate_canonical_report(report, expected["report_sha256"], "R112")
    bounded = report.get("bounded_acceptance", {})
    identities = report.get("identities", {})
    if (
        report.get("check") != R112_CHECK_ID
        or report.get("audit_id") != R112_AUDIT_ID
        or report.get("status") != "PASS"
        or report.get("gate_decision")
        != "PERMIT_R113_CLEAN_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY"
        or bounded.get("r113_model_identity_preflight") != "AUTHORIZED_REPORT_ONLY"
        or report.get("repository", {}).get("dirty") is not False
        or identities.get("mirror_v2_sha256") != profile["source"]["mirror_v2_sha256"]
        or identities.get("humanoid_usd_sha256")
        != profile["source"]["translation"]["humanoid_usd_sha256"]
        or identities.get("ground_usd_sha256")
        != profile["source"]["translation"]["ground_usd_sha256"]
        or identities.get("translation_manifest_file_sha256")
        != profile["source"]["translation"]["manifest_file_sha256"]
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "model_identity_preflights",
                "solver_runs",
                "kto_solves",
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "candidate_artifacts_built",
                "physx_scene_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R113 R112 gate contract differs")
    return report


def validate_r110(
    *, profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r110"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R113 R110 source identity differs")
    source_profile = json.loads(profile_path.read_bytes())
    _validate_r110_profile(source_profile)
    report = json.loads(report_path.read_bytes())
    _validate_canonical_report(report, expected["report_sha256"], "R110")
    if (
        report.get("check") != R110_CHECK_ID
        or report.get("formulation_id") != R110_FORMULATION_ID
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R111_CANONICAL_MATERIAL_LINEAGE_IMPLEMENTATION_ONLY"
        or report.get("repository", {}).get("dirty") is not False
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "solver_runs",
                "kto_solves",
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "physx_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R113 R110 point-force gate contract differs")
    return report


def audit_descriptor_identity(
    profile: Mapping[str, Any], descriptor: Mapping[str, Any]
) -> dict[str, Any]:
    document = dict(descriptor)
    validate_current_biomechanics_descriptor(document)
    structural = audit_structural_descriptor_identity(document)
    expected = profile["expected_shared_identity"]
    exact_fields = (
        "body_schema_hash",
        "compiled_descriptor_hash",
        "total_mass_microkilograms",
        "non_colliding_carrier_count",
        "collision_exclusion_count",
        "material_id_counts",
        "coordinate_mapping",
        "motor_hz",
        "physics_hz",
    )
    if (
        any(structural[field] != expected[field] for field in exact_fields)
        or document.get("material_lineage_hash") != expected["material_lineage_hash"]
        or document.get("ground_material_id") != expected["ground_material_id"]
        or document.get("materials") != profile["expected_materials"]
        or document.get("material_combine_profile")
        != profile["expected_combine_profile"]
        or document.get("collider_material_assignment_counts")
        != profile["expected_collider_material_assignment_counts"]
        or structural["material_coefficients_present"] is not True
        or structural["material_combine_profile_present"] is not True
    ):
        raise ValueError("R113 complete descriptor identity differs")
    return {
        **structural,
        "status": "COMPLETE_STRUCTURAL_AND_MATERIAL_IDENTITY",
        "material_lineage_hash": document["material_lineage_hash"],
        "ground_material_id": document["ground_material_id"],
        "material_descriptors": document["materials"],
        "material_combine_profile": document["material_combine_profile"],
        "collider_material_assignment_counts": document[
            "collider_material_assignment_counts"
        ],
        "legacy_descriptor_reinterpreted": False,
    }


def audit_translation_identity(
    profile: Mapping[str, Any],
    descriptor: Mapping[str, Any],
    *,
    translation_manifest_path: Path,
    humanoid_usd_path: Path,
    ground_usd_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["translation"]
    if (
        sha256(translation_manifest_path) != expected["manifest_file_sha256"]
        or sha256(humanoid_usd_path) != expected["humanoid_usd_sha256"]
        or sha256(ground_usd_path) != expected["ground_usd_sha256"]
    ):
        raise ValueError("R113 R112 translation byte identity differs")
    manifest = validate_translation_bundle(
        dict(descriptor),
        translation_manifest_path,
        humanoid_usd_path=humanoid_usd_path,
        ground_usd_path=ground_usd_path,
    )
    humanoid = humanoid_usd_path.read_text(encoding="utf-8")
    ground = ground_usd_path.read_text(encoding="utf-8")
    humanoid_bindings = re.findall(
        r"rel material:binding:physics = </Humanoid/Materials/([^>]+)>", humanoid
    )
    ground_bindings = re.findall(
        r"rel material:binding:physics = </Ground/Materials/([^>]+)>", ground
    )
    if (
        manifest.get("descriptor_content_sha256")
        != expected["descriptor_content_sha256"]
        or humanoid.count('def Material "') != 2
        or humanoid_bindings.count("physics_material_humanoid_body_v1") != 17
        or humanoid_bindings.count("physics_material_humanoid_sole_v1") != 2
        or len(humanoid_bindings) != 19
        or ground.count('def Material "') != 1
        or ground_bindings != ["physics_material_humanoid_ground_v1"]
    ):
        raise ValueError("R113 physics-purpose material binding closure differs")
    combined = humanoid + ground
    required = (
        "physics:staticFriction = 0.800003052",
        "physics:dynamicFriction = 0.699996948",
        "physics:restitution = 0",
        'physxMaterial:frictionCombineMode = "average"',
        'physxMaterial:restitutionCombineMode = "average"',
        "nextengine:rollingFrictionQ16 = 0",
        "nextengine:spinningFrictionQ16 = 0",
        "nextengine:surfaceVelocityMicrometresPerSecond = (0, 0, 0)",
        descriptor["material_lineage_hash"],
    )
    if any(token not in combined for token in required) or any(
        token in combined
        for token in (
            "physics:staticFriction = 0.5\n",
            "physics:dynamicFriction = 0.5\n",
        )
    ):
        raise ValueError("R113 exact derived material values differ")
    return {
        "status": "COMPLETE_EXACT_R112_TRANSLATION_IDENTITY",
        "manifest_schema_version": manifest["schema_version"],
        "descriptor_content_sha256": manifest["descriptor_content_sha256"],
        "humanoid_usd_sha256": sha256(humanoid_usd_path),
        "ground_usd_sha256": sha256(ground_usd_path),
        "humanoid_material_prim_count": 2,
        "humanoid_physics_binding_count": len(humanoid_bindings),
        "ground_material_prim_count": 1,
        "ground_physics_binding_count": len(ground_bindings),
        "isaac_default_material_reachable": False,
        "legacy_usd_reinterpreted": False,
    }


def audit_source_identity(
    *,
    profile: Mapping[str, Any],
    tracked_sources: Mapping[str, Path],
    isaac_sources: Mapping[str, Path],
    isaac_repository: Mapping[str, Any],
) -> dict[str, Any]:
    if set(tracked_sources) != set(TRACKED_SOURCE_PATHS):
        raise ValueError("R113 tracked source inventory differs")
    if set(isaac_sources) != set(ISAAC_SOURCE_PATHS):
        raise ValueError("R113 Isaac source inventory differs")
    tracked_hashes = {name: sha256(path) for name, path in tracked_sources.items()}
    isaac_hashes = {name: sha256(path) for name, path in isaac_sources.items()}
    if tracked_hashes != profile["source"]["tracked_source_sha256"]:
        raise ValueError("R113 tracked model source identity differs")
    if isaac_hashes != profile["source"]["isaac_lab"]["source_sha256"]:
        raise ValueError("R113 Isaac Lab source identity differs")
    expected_isaac = profile["source"]["isaac_lab"]
    if (
        isaac_repository.get("commit") != expected_isaac["git_commit"]
        or isaac_repository.get("tag") != expected_isaac["git_tag"]
        or isaac_repository.get("dirty") is not False
    ):
        raise ValueError("R113 Isaac Lab repository identity differs")

    text = {
        name: path.read_text(encoding="utf-8") for name, path in tracked_sources.items()
    }
    upstream = {
        name: path.read_text(encoding="utf-8") for name, path in isaac_sources.items()
    }
    required = {
        "spec_26": ("PhysicsMaterialDescriptorV2", "cannot substitute its default."),
        "spec_35": (
            "CompiledBodySchemaV3",
            "GPU execution is evaluated by correspondence",
        ),
        "adr_071": ("Bridge ABI 4", "Derived USD/Isaac material prims"),
        "compiler_v3": ("struct CompiledBodySchemaV3", "validate_material_closure"),
        "material_contract": (
            "struct PhysicsMaterialDescriptorV2",
            "struct PhysicsMaterialCombineProfileV1",
        ),
        "physics_physx": (
            "pub const PHYSX_CPU_TIMESTEP_HZ: u32 = 240;",
            "PhysXArticulationWorldV3",
            "native.configure_material(catalog.shared_material.ffi())?;",
            "native.configure_scene(profile.ffi())?;",
        ),
        "physics_physx_material": (
            "PhysXSharedMaterialProfileV1",
            "MATERIAL_COEFFICIENT_ENCODING_Q16",
            "descriptor.rolling_friction_q16 != first.rolling_friction_q16",
        ),
        "physx_ffi_lib": (
            "NEXTENGINE_PHYSX_ABI_VERSION: u32 = 4",
            "pub fn configure_material",
        ),
        "physx_bridge": (
            "ne_physx_world_configure_material",
            "createMaterial(static_friction, dynamic_friction, restitution)",
            "setFrictionCombineMode(physx::PxCombineMode::eAVERAGE)",
            "setRestitutionCombineMode(physx::PxCombineMode::eAVERAGE)",
            "setTorsionalPatchRadius(0.0F)",
            "setMinTorsionalPatchRadius(0.0F)",
        ),
        "safety_control": (
            "fn intersect_effort_limits(",
            "fn positive_work_charge(",
            "pub fn reset_to_reference(",
        ),
        "material_lineage": (
            "BIOMECHANICS_TRANSLATOR_ID_V2",
            "validate_current_material_lineage",
        ),
        "motor_mirror": ("validate_current_biomechanics_descriptor",),
        "usd_translation": (
            "material:binding:physics",
            "render_ground_usda",
            "validate_translation_bundle",
        ),
        "isaac_reference_env": (
            "SimulationCfg(dt=1.0 / 240.0, render_interval=4)",
            "solver_position_iteration_count=8",
            "stiffness=0.0",
            "damping=0.0",
            "self.translation_manifest = validate_translation_bundle",
            "self.cfg.ground.func",
        ),
    }
    for name, fragments in required.items():
        _require_fragments(text[name], fragments, f"R113 {name}")
    _require_fragments(
        upstream["simulation_cfg"],
        (
            "gravity: tuple[float, float, float] = (0.0, 0.0, -9.81)",
            "enable_enhanced_determinism: bool = False",
            "solver_type: Literal[0, 1] = 1",
        ),
        "R113 Isaac simulation defaults",
    )
    _require_fragments(
        upstream["physics_materials_cfg"],
        ("static_friction: float = 0.5", "dynamic_friction: float = 0.5"),
        "R113 unreachable Isaac material defaults",
    )
    _require_fragments(
        upstream["from_files_cfg"],
        ("class GroundPlaneCfg", "RigidBodyMaterialCfg()"),
        "R113 unreachable Isaac ground defaults",
    )
    if "createMaterial(0.8F, 0.7F, 0.0F)" in text["physx_bridge"]:
        raise ValueError("R113 native ambient material remains reachable")
    if any(
        token in text["isaac_reference_env"]
        for token in ("GroundPlaneCfg", "RigidBodyMaterialCfg", "spawn_ground_plane")
    ):
        raise ValueError("R113 Isaac ambient material path remains reachable")
    v3_world = text["physics_physx"].split("impl PhysXArticulationWorldV3", 1)[1]
    if v3_world.index(
        "native.configure_material(catalog.shared_material.ffi())?;"
    ) > v3_world.index("native.configure_scene(profile.ffi())?;"):
        raise ValueError("R113 native material is not configured before scene")
    if text["isaac_reference_env"].index(
        "self.translation_manifest = validate_translation_bundle"
    ) > text["isaac_reference_env"].index("self.cfg.ground.func"):
        raise ValueError("R113 Isaac bundle is not validated before scene setup")
    return {
        "status": "COMPLETE_HASH_BOUND_SOURCE_IDENTITY",
        "tracked_source_sha256": tracked_hashes,
        "isaac_source_sha256": isaac_hashes,
        "isaac_lab_commit": isaac_repository["commit"],
        "isaac_lab_tag": isaac_repository["tag"],
        "native_material_configured_before_scene": True,
        "native_ambient_material_reachable": False,
        "isaac_bundle_validated_before_scene": True,
        "isaac_ambient_material_reachable": False,
        "fixed_pd_safety_identity_hash_bound": True,
    }


def audit_point_force_identity(
    profile: Mapping[str, Any], r110_report: Mapping[str, Any]
) -> dict[str, Any]:
    point_force = r110_report.get("point_contact_force_coordinates", {})
    projection = {
        "status": point_force.get("status"),
        "owner": point_force.get("owner"),
        "engine_world_axes": point_force.get("engine_world_axes"),
        "contact_frame": point_force.get("contact_frame"),
        "force_component_order": point_force.get("force_component_order"),
        "world_force_mapping": point_force.get("world_force_mapping"),
        "ordered_points": point_force.get("ordered_points"),
        "mode_encoding": point_force.get("mode_encoding"),
        "cadence": point_force.get("cadence"),
        "application": point_force.get("application"),
        "constraints": point_force.get("constraints"),
        "application_point_identity": point_force.get("application_point_identity"),
        "runtime_contract_change": point_force.get("runtime_contract_change"),
    }
    if projection != profile["expected_point_force_identity"]:
        raise ValueError("R113 solver-private point-force identity differs")
    return {**projection, "model_identity_satisfied": True}


def classify_model_identity(
    *,
    profile: Mapping[str, Any],
    descriptor_identity: Mapping[str, Any],
    translation_identity: Mapping[str, Any],
    source_identity: Mapping[str, Any],
    point_force_identity: Mapping[str, Any],
) -> dict[str, Any]:
    blockers: list[str] = []
    if descriptor_identity.get("status") != "COMPLETE_STRUCTURAL_AND_MATERIAL_IDENTITY":
        blockers.append("COMPLETE_DESCRIPTOR_IDENTITY_ABSENT")
    if translation_identity.get("status") != "COMPLETE_EXACT_R112_TRANSLATION_IDENTITY":
        blockers.append("DERIVED_TRANSLATION_IDENTITY_ABSENT")
    if (
        source_identity.get("native_ambient_material_reachable") is not False
        or source_identity.get("isaac_ambient_material_reachable") is not False
    ):
        blockers.append("BACKEND_MATERIAL_DEFAULT_REACHABLE")
    if point_force_identity.get("model_identity_satisfied") is not True:
        blockers.append("POINT_FORCE_COORDINATES_UNOWNED")
    if blockers != profile["expected_blocking_reasons"]:
        raise ValueError("R113 blocking model-identity reasons differ")
    return {
        "status": "PASS",
        "blocking_reasons": blockers,
        "blocking_reason_count": len(blockers),
        "closed_r109_blocking_reasons": [
            "CANONICAL_MATERIAL_DESCRIPTORS_ABSENT",
            "BACKEND_MATERIAL_DEFAULTS_DIVERGE",
            "CONTACT_WRENCH_CONVENTION_UNOWNED",
        ],
        "nonblocking_declared_mirror_divergence_count": 3,
        "single_backend_neutral_dynamics_model_available": True,
        "canonical_material_contract_satisfied": True,
        "derived_material_contract_satisfied": True,
        "point_force_contract_satisfied": True,
        "kinematic_shared_identity_satisfied": True,
        "runtime_correspondence_claim": False,
        "physx_behavior_claim": False,
        "gate_decision": profile["decision"]["pass"],
    }


def _declared_backend_divergences() -> list[dict[str, Any]]:
    return [
        {
            "field": "solver_position_iterations",
            "canonical_native": 16,
            "isaac_mirror": 8,
            "classification": "DECLARED_MIRROR_NUMERICAL_DIVERGENCE",
            "blocking": False,
        },
        {
            "field": "scene_determinism_and_contact_cache_flags",
            "canonical_native": "enhanced determinism on; PCM/contact cache/sleeping off",
            "isaac_mirror": "Isaac Lab defaults; enhanced determinism off",
            "classification": "DECLARED_MIRROR_NUMERICAL_DIVERGENCE",
            "blocking": False,
        },
        {
            "field": "flat_ground_representation",
            "canonical_native": "100 m x 100 m top face of static box at engine Y=0",
            "isaac_mirror": "derived infinite plane at Isaac Z=0",
            "classification": "DECLARED_MIRROR_GEOMETRY_REPRESENTATION",
            "blocking": False,
        },
    ]


def _validate_canonical_report(
    report: Mapping[str, Any], expected_hash: str, label: str
) -> None:
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        embedded != expected_hash
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
    ):
        raise ValueError(f"R113 {label} canonical report identity differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    shared = profile.get("expected_shared_identity", {})
    native = profile.get("expected_native_backend", {})
    isaac = profile.get("expected_isaac_backend", {})
    decision = profile.get("decision", {})
    bounded = profile.get("bounded_acceptance", {})
    source = profile.get("source", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("preflight_id") != PREFLIGHT_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim")
        != "CleanOptimizerFreeDynamicsModelIdentityPreflightOnly"
        or scope.get("run_id") != "R113"
        or scope.get("body_count") != 24
        or scope.get("joint_count") != 23
        or scope.get("actuator_count") != 23
        or scope.get("collider_count") != 19
        or scope.get("motor_hz") != 60
        or scope.get("physics_hz") != 240
        or scope.get("model_identity_preflights") != 1
        or scope.get("solver_runs") != 0
        or scope.get("physx_scene_runs") != 0
        or scope.get("candidate_construction") is not False
        or shared.get("compiled_descriptor_hash")
        != "6751853a812f549866f1db9d3662d8115b18db9b6d73beabd7221bb9f972f027"
        or shared.get("material_lineage_hash")
        != "2d13e197f766e6a24090edf396dfc2fb6cbbf4c578ea9868dffa06ab7adab751"
        or source.get("r112", {}).get("report_sha256")
        != "dfb3bd892b04023054ce947127743e7ada78b40000a93e22887101c544c493f2"
        or source.get("r110", {}).get("report_sha256")
        != "83408b97b6ba13dc801b4d9b4f68e55146f9f39b09a451c238b24cc4c8c7d88d"
        or source.get("mirror_v2_sha256")
        != "7928fe23affaf9dd16a0c82db1d7da85e61af2ad0f071f9423c6df1121ba50a3"
        or profile.get("expected_materials") != EXPECTED_MATERIALS
        or profile.get("expected_combine_profile") != EXPECTED_COMBINE_PROFILE
        or native.get("static_friction_q16") != 52_429
        or native.get("dynamic_friction_q16") != 45_875
        or native.get("restitution_q16") != 0
        or native.get("ambient_material_reachable") is not False
        or isaac.get("static_friction_q16") != native.get("static_friction_q16")
        or isaac.get("dynamic_friction_q16") != native.get("dynamic_friction_q16")
        or isaac.get("restitution_q16") != native.get("restitution_q16")
        or isaac.get("ambient_material_reachable") is not False
        or profile.get("expected_blocking_reasons") != []
        or decision.get("pass")
        != "PERMIT_SEPARATE_BOUNDED_KTO_EXECUTION_FORMULATION_ONLY"
        or decision.get("fail") != "STOP_INVALID_MODEL_LINEAGE"
        or decision.get("expected")
        != "PERMIT_SEPARATE_BOUNDED_KTO_EXECUTION_FORMULATION_ONLY"
        or bounded.get("r114_quantization_aware_kto_formulation")
        != "AUTHORIZED_FORMULATION_ONLY"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
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
        or set(source.get("tracked_source_sha256", {})) != set(TRACKED_SOURCE_PATHS)
        or set(source.get("isaac_lab", {}).get("source_sha256", {}))
        != set(ISAAC_SOURCE_PATHS)
    ):
        raise ValueError("R113 clean model-identity profile differs")
    commands = profile.get("validation_commands", ())
    ids = [row.get("id") for row in commands]
    if (
        not commands
        or len(ids) != len(set(ids))
        or any(not row.get("arguments") for row in commands)
    ):
        raise ValueError("R113 validation command closure differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    if repository.get("dirty") is not False:
        raise ValueError("R113 repository must be clean")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, Any]]
) -> list[dict[str, Any]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R113 validation result differs")
    return [dict(row) for row in results]


def _require_fragments(text: str, fragments: tuple[str, ...], label: str) -> None:
    missing = [fragment for fragment in fragments if fragment not in text]
    if missing:
        raise ValueError(f"{label} differs: {missing[0]}")
