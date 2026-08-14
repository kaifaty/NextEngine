from __future__ import annotations

import hashlib
import json
from collections import Counter
from collections.abc import Mapping
from pathlib import Path
from typing import Any

from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.motor_mirror import validate_biomechanics_descriptor
from next_lab.progressive_kinodynamic_formulation import (
    CHECK_ID as R108_CHECK_ID,
)
from next_lab.progressive_kinodynamic_formulation import (
    FORMULATION_ID as R108_FORMULATION_ID,
)
from next_lab.progressive_kinodynamic_formulation import (
    _validate_profile as _validate_r108_profile,
)
from next_lab.usd_translation import (
    BIOMECHANICS_TRANSLATOR_VERSION,
    render_usda,
)

PREFLIGHT_ID = "nextengine.humanoid-dynamics-model-identity-preflight.v1"
CHECK_ID = "TRAIN-4-DYNAMICS-MODEL-IDENTITY-PREFLIGHT"

TRACKED_SOURCE_PATHS = {
    "spec_26": "docs/architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md",
    "spec_35": "docs/architecture/35-deterministic-humanoid-training-substrate.md",
    "adr_069": "docs/architecture/adr/069-biomechanics-body-schema-v2-and-solver-projection.md",
    "adr_070": "docs/architecture/adr/070-biomechanics-reference-tracking-training-environment.md",
    "biomechanics": "crates/motor/src/biomechanics.rs",
    "compiler_v2": "crates/motor/src/compiler_v2.rs",
    "physics_descriptors": "crates/contracts/src/physics/descriptors.rs",
    "physics_physx": "crates/physics-physx/src/lib.rs",
    "physx_ffi_build": "crates/physics-physx-ffi/build.rs",
    "physx_ffi_lib": "crates/physics-physx-ffi/src/lib.rs",
    "physx_bridge": "crates/physics-physx-ffi/native/nextengine_physx_bridge.cpp",
    "safety_control": "crates/motor/src/safety_control.rs",
    "mirror_v2": "crates/motor/src/mirror_v2.rs",
    "usd_translation": "lab/next_lab/usd_translation.py",
    "isaac_reference_env": "lab/next_lab/isaac_reference_env.py",
    "isaac_reference_env_tests": "lab/tests/test_isaac_reference_env.py",
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


def build_dynamics_model_identity_preflight(
    *,
    profile_path: Path,
    r108_report_path: Path,
    r108_profile_path: Path,
    descriptor_path: Path,
    translation_manifest_path: Path,
    usd_path: Path,
    tracked_sources: Mapping[str, Path],
    isaac_sources: Mapping[str, Path],
    isaac_repository: Mapping[str, Any],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Audit R109's model lineage without constructing or executing a model."""

    direct_paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r108_report_path,
            r108_profile_path,
            descriptor_path,
            translation_manifest_path,
            usd_path,
            tool_path,
        )
    )
    (
        profile_path,
        r108_report_path,
        r108_profile_path,
        descriptor_path,
        translation_manifest_path,
        usd_path,
        tool_path,
    ) = direct_paths
    tracked_sources = {name: path.resolve() for name, path in tracked_sources.items()}
    isaac_sources = {name: path.resolve() for name, path in isaac_sources.items()}
    all_paths = (*direct_paths, *tracked_sources.values(), *isaac_sources.values())
    if any(not path.is_file() for path in all_paths):
        raise FileNotFoundError("dynamics-model identity input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    r108 = validate_r108(
        report_path=r108_report_path,
        profile_path=r108_profile_path,
        expected=profile["source"]["r108"],
    )
    if sha256(descriptor_path) != profile["source"]["descriptor_sha256"]:
        raise ValueError("R109 descriptor identity differs")
    descriptor = json.loads(descriptor_path.read_bytes())
    descriptor_identity = audit_descriptor_identity(descriptor)
    translation_identity = validate_translation_identity(
        descriptor=descriptor,
        manifest_path=translation_manifest_path,
        usd_path=usd_path,
        expected_manifest_sha256=profile["source"]["translation_manifest_sha256"],
        expected_usd_sha256=profile["source"]["usd_sha256"],
    )
    source_identity = audit_source_identity(
        profile=profile,
        tracked_sources=tracked_sources,
        isaac_sources=isaac_sources,
        isaac_repository=isaac_repository,
    )
    wrench_ownership = audit_contact_wrench_ownership(
        repository_root=Path(repository["root"]),
    )
    model_identity = classify_model_identity(
        profile=profile,
        descriptor_identity=descriptor_identity,
        translation_identity=translation_identity,
        wrench_ownership=wrench_ownership,
    )

    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "FAIL",
        "claim": profile["claim"],
        "gate_decision": model_identity["gate_decision"],
        "preflight_id": PREFLIGHT_ID,
        "scope": profile["scope"],
        "required_identity_groups": profile["required_identity_groups"],
        "architecture_contract": {
            "canonical_material_ownership": (
                "SPEC-26 requires exact PhysicsMaterialDescriptorV1 values and "
                "one exact combine profile; a backend default cannot substitute"
            ),
            "mirror_boundary": (
                "SPEC-35 and ADR-070 permit tolerance-based GPU correspondence; "
                "final candidate evaluation returns to canonical CPU PhysX"
            ),
            "material_identity_status": "VIOLATED",
            "gpu_byte_exact_parity_required": False,
        },
        "descriptor_identity": descriptor_identity,
        "translation_identity": translation_identity,
        "shared_identity": {
            **profile["expected_shared_identity"],
            "status": "PASS_EXCEPT_MATERIAL_AND_WRENCH_OWNERSHIP",
            "usd_projection": "BYTE_EXACT",
            "gravity_coordinate_transform": "EXACT",
            "fixed_pd_controller_semantics": "HASH_BOUND_AND_MIRRORED",
        },
        "backend_profiles": {
            "canonical_native": profile["expected_native_backend"],
            "isaac_correspondence_mirror": profile["expected_isaac_backend"],
        },
        "declared_backend_divergences": [
            {
                "field": "solver_position_iterations",
                "canonical_native": 16,
                "isaac_mirror": 8,
                "classification": "DECLARED_MIRROR_NUMERICAL_DIVERGENCE",
                "blocking": False,
                "reason": (
                    "GPU correspondence is tolerance-based, not byte-exact; both "
                    "values must remain separately hash-bound"
                ),
            },
            {
                "field": "scene_determinism_and_contact_cache_flags",
                "canonical_native": (
                    "enhanced determinism on; PCM/contact cache/sleeping off"
                ),
                "isaac_mirror": "Isaac Lab defaults; enhanced determinism off",
                "classification": "DECLARED_MIRROR_NUMERICAL_DIVERGENCE",
                "blocking": False,
                "reason": "final acceptance remains canonical CPU PhysX",
            },
            {
                "field": "material_and_friction",
                "canonical_native": "single hard-coded 0.8/0.7/0.0 material",
                "isaac_mirror": "single default 0.5/0.5/0.0 material",
                "classification": "BLOCKING_UNOWNED_MODEL_DIVERGENCE",
                "blocking": True,
                "reason": (
                    "two descriptor material IDs have no exact coefficient "
                    "descriptors or combine profile and collapse to backend defaults"
                ),
            },
        ],
        "contact_wrench_ownership": wrench_ownership,
        "source_identity": source_identity,
        "model_identity_result": model_identity,
        "next_smallest_action": profile["decision"]["next_smallest_action"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r108_report_sha256": r108["report_sha256"],
            "r108_report_file_sha256": sha256(r108_report_path),
            "r108_profile_sha256": sha256(r108_profile_path),
            "descriptor_sha256": sha256(descriptor_path),
            "translation_manifest_sha256": sha256(translation_manifest_path),
            "usd_sha256": sha256(usd_path),
            "tool_sha256": sha256(tool_path),
            "preflight_module_sha256": sha256(Path(__file__).resolve()),
            "tracked_source_sha256": source_identity["tracked_source_sha256"],
            "isaac_source_sha256": source_identity["isaac_source_sha256"],
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
        "physx_runs": 0,
        "candidate_evaluations": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": {
            key: value for key, value in repository.items() if key != "root"
        },
        "isaac_repository": dict(isaac_repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def validate_r108(
    *,
    report_path: Path,
    profile_path: Path,
    expected: Mapping[str, Any],
) -> dict[str, Any]:
    if sha256(report_path) != expected.get("report_file_sha256"):
        raise ValueError("R108 report file identity differs")
    if sha256(profile_path) != expected.get("profile_sha256"):
        raise ValueError("R108 profile identity differs")
    r108_profile = json.loads(profile_path.read_bytes())
    _validate_r108_profile(r108_profile)
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    bounded = report.get("bounded_acceptance", {})
    if (
        report.get("check") != R108_CHECK_ID
        or report.get("formulation_id") != R108_FORMULATION_ID
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R109_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY"
        or embedded != expected.get("report_sha256")
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("identities", {}).get("profile_sha256")
        != expected.get("profile_sha256")
        or report.get("repository", {}).get("dirty") is not False
        or bounded.get("r109_dynamics_model_identity_preflight") != "AUTHORIZED"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "quantization_aware_kto_solve",
                "inverse_dynamics_solve",
                "kinodynamic_solve",
                "candidate_artifact",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "model_identity_preflights",
                "kto_solves",
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "candidate_artifacts_built",
                "physx_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R108 progressive formulation contract differs")
    return report


def audit_descriptor_identity(descriptor: Mapping[str, Any]) -> dict[str, Any]:
    document = dict(descriptor)
    validate_biomechanics_descriptor(document)
    bodies = document["bodies"]
    joints = document["joints"]
    actuators = document["actuators"]
    colliders = [collider for body in bodies for collider in body["colliders"]]
    material_counts = Counter(str(row["material_id"]) for row in colliders)
    coefficient_fields = {
        "static_friction_q16",
        "dynamic_friction_q16",
        "restitution_q16",
    }
    descriptor_has_material_coefficients = isinstance(
        document.get("materials"), list
    ) and all(coefficient_fields.issubset(row) for row in document["materials"])
    root_bodies = [body for body in bodies if body.get("parent_body_slot") is None]
    if (
        len(root_bodies) != 1
        or root_bodies[0]["body_slot"] != 0
        or sorted(int(joint["dof_ordinal"]) for joint in joints) != list(range(23))
        or sorted(int(actuator["dof_ordinal"]) for actuator in actuators)
        != list(range(23))
        or any(
            int(delta[0]) != -int(delta[1])
            for delta in (
                actuator["target_delta_microradians_per_motor_tick"]
                for actuator in actuators
            )
        )
    ):
        raise ValueError("descriptor generalized-coordinate identity differs")
    return {
        "status": "STRUCTURAL_IDENTITY_PASS_MATERIAL_AUTHORITY_ABSENT",
        "body_schema_hash": document["body_schema_hash"],
        "compiled_descriptor_hash": document["compiled_descriptor_hash"],
        "body_count": len(bodies),
        "joint_count": len(joints),
        "actuator_count": len(actuators),
        "collider_count": len(colliders),
        "non_colliding_carrier_count": sum(
            bool(body["non_colliding_carrier"]) for body in bodies
        ),
        "collision_exclusion_count": len(document["collision_exclusions"]),
        "total_mass_microkilograms": sum(
            int(body["mass_microkilograms"]) for body in bodies
        ),
        "root_body_id": root_bodies[0]["body_id"],
        "root_body_slot": root_bodies[0]["body_slot"],
        "material_id_counts": dict(sorted(material_counts.items())),
        "material_descriptor_array_present": isinstance(
            document.get("materials"), list
        ),
        "material_coefficients_present": descriptor_has_material_coefficients,
        "material_combine_profile_present": ("material_combine_profile" in document),
        "coordinate_mapping": document["coordinate_mapping"],
        "motor_hz": document["motor_hz"],
        "physics_hz": document["physics_hz"],
        "physics_substeps_per_motor_tick": (
            document["physics_hz"] // document["motor_hz"]
        ),
    }


def validate_translation_identity(
    *,
    descriptor: Mapping[str, Any],
    manifest_path: Path,
    usd_path: Path,
    expected_manifest_sha256: str,
    expected_usd_sha256: str,
) -> dict[str, Any]:
    if sha256(manifest_path) != expected_manifest_sha256:
        raise ValueError("translation manifest identity differs")
    if sha256(usd_path) != expected_usd_sha256:
        raise ValueError("derived USD identity differs")
    manifest = json.loads(manifest_path.read_bytes())
    rendered = render_usda(dict(descriptor)).encode("utf-8")
    usd = usd_path.read_bytes()
    if rendered != usd or manifest != {
        "body_schema_hash": descriptor["body_schema_hash"],
        "compiled_descriptor_hash": descriptor["compiled_descriptor_hash"],
        "schema_version": 2,
        "translator_version": BIOMECHANICS_TRANSLATOR_VERSION,
        "usd_path": "humanoid.usda",
        "usd_sha256": expected_usd_sha256,
    }:
        raise ValueError("derived USD translation lineage differs")
    text = usd.decode("utf-8")
    material_tokens = (
        "PhysicsMaterialAPI",
        "material:binding",
        "physics:staticFriction",
        "physics:dynamicFriction",
    )
    return {
        "status": "BYTE_EXACT_DESCRIPTOR_PROJECTION",
        "rendered_usd_matches_stored_bytes": True,
        "body_schema_hash_matches": True,
        "compiled_descriptor_hash_matches": True,
        "joint_prim_count": text.count("def PhysicsRevoluteJoint"),
        "collider_prim_count": text.count(
            'prepend apiSchemas = ["PhysicsCollisionAPI"]'
        ),
        "filtered_pair_body_count": text.count("rel physics:filteredPairs"),
        "material_binding_present": any(token in text for token in material_tokens),
        "scene_gravity_authored_in_usd": "physics:gravity" in text,
        "actuator_drive_authored_in_usd": "PhysicsDriveAPI" in text,
        "runtime_fields_supplied_outside_usd": [
            "gravity",
            "ground",
            "material/friction",
            "solver profile",
            "explicit fixed-PD efforts",
        ],
    }


def audit_source_identity(
    *,
    profile: Mapping[str, Any],
    tracked_sources: Mapping[str, Path],
    isaac_sources: Mapping[str, Path],
    isaac_repository: Mapping[str, Any],
) -> dict[str, Any]:
    if set(tracked_sources) != set(TRACKED_SOURCE_PATHS):
        raise ValueError("tracked model source inventory differs")
    if set(isaac_sources) != set(ISAAC_SOURCE_PATHS):
        raise ValueError("Isaac source inventory differs")
    expected_tracked = {
        **{
            name: profile["source"]["architecture"][f"{name}_sha256"]
            for name in ("spec_26", "spec_35", "adr_069", "adr_070")
        },
        **{
            name: profile["source"]["repository_sources"][f"{name}_sha256"]
            for name in TRACKED_SOURCE_PATHS
            if name not in {"spec_26", "spec_35", "adr_069", "adr_070"}
        },
    }
    actual_tracked = {name: sha256(path) for name, path in tracked_sources.items()}
    if actual_tracked != expected_tracked:
        raise ValueError("tracked dynamics-model source identity differs")
    expected_isaac = {
        name: profile["source"]["isaac_lab"][f"{name}_sha256"]
        for name in ISAAC_SOURCE_PATHS
    }
    actual_isaac = {name: sha256(path) for name, path in isaac_sources.items()}
    if actual_isaac != expected_isaac:
        raise ValueError("Isaac Lab model source identity differs")
    expected_repository = profile["source"]["isaac_lab"]
    if (
        isaac_repository.get("commit") != expected_repository["git_commit"]
        or isaac_repository.get("tag") != expected_repository["git_tag"]
        or isaac_repository.get("dirty") is not False
    ):
        raise ValueError("Isaac Lab repository identity differs")

    text = {
        name: path.read_text(encoding="utf-8") for name, path in tracked_sources.items()
    }
    upstream = {
        name: path.read_text(encoding="utf-8") for name, path in isaac_sources.items()
    }
    _require_fragments(
        text["spec_26"],
        ("PhysicsMaterialDescriptorV1 {", "cannot substitute its default."),
        "SPEC-26 material contract",
    )
    _require_fragments(
        text["spec_35"],
        ("GPU execution is evaluated by correspondence", "not byte-exact"),
        "SPEC-35 mirror boundary",
    )
    _require_fragments(
        text["adr_070"],
        ("GPU floating physics", "canonical PhysX/headless path"),
        "ADR-070 mirror boundary",
    )
    _require_fragments(
        text["compiler_v2"],
        (
            "physx_scene_profile.position_iterations = 16;",
            "physx_scene_profile.velocity_iterations = 4;",
            "centre_bits: [0.0_f32.to_bits(), (-0.5_f32).to_bits(), 0.0_f32.to_bits()]",
        ),
        "native compiler scene profile",
    )
    _require_fragments(
        text["physics_physx"],
        (
            "pub const PHYSX_CPU_TIMESTEP_HZ: u32 = 240;",
            "gravity_bits: [0.0_f32.to_bits(), (-9.81_f32).to_bits(), 0.0_f32.to_bits()]",
        ),
        "native gravity/cadence",
    )
    _require_fragments(
        text["physx_bridge"],
        (
            "createMaterial(0.8F, 0.7F, 0.0F)",
            "PxSolverType::eTGS",
            "PxBroadPhaseType::ePABP",
            "PxFrictionType::ePATCH",
            "eDISABLE_CONTACT_CACHE",
            "eENABLE_ENHANCED_DETERMINISM",
            "eDISABLE_SLEEPING",
        ),
        "native bridge material/scene",
    )
    _require_fragments(
        text["safety_control"],
        (
            "fn intersect_effort_limits(",
            "fn positive_work_charge(",
            "pub(crate) fn round_div_ties_even(",
            "pub fn reset_to_reference(",
        ),
        "native fixed-PD safety controller",
    )
    _require_fragments(
        text["isaac_reference_env"],
        (
            "SimulationCfg(dt=1.0 / 240.0, render_interval=4)",
            "solver_position_iteration_count=8",
            "solver_velocity_iteration_count=4",
            "stiffness=0.0",
            "damping=0.0",
            "GroundPlaneCfg()",
            "def _canonical_pd_requested_effort_tensor(",
            "def _intersect_effort_limits_tensor(",
        ),
        "Isaac reference environment",
    )
    _require_fragments(
        upstream["simulation_cfg"],
        (
            "gravity: tuple[float, float, float] = (0.0, 0.0, -9.81)",
            "enable_enhanced_determinism: bool = False",
            "solver_type: Literal[0, 1] = 1",
        ),
        "Isaac Lab simulation defaults",
    )
    _require_fragments(
        upstream["physics_materials_cfg"],
        (
            "static_friction: float = 0.5",
            "dynamic_friction: float = 0.5",
            "restitution: float = 0.0",
            'friction_combine_mode: Literal["average", "min", "multiply", "max"] = "average"',
        ),
        "Isaac Lab material defaults",
    )
    _require_fragments(
        upstream["from_files_cfg"],
        ("class GroundPlaneCfg", "RigidBodyMaterialCfg()"),
        "Isaac Lab ground defaults",
    )
    if (
        "static_friction" in text["mirror_v2"]
        or "material:binding" in text["usd_translation"]
    ):
        raise ValueError("unexpected canonical material projection appeared")
    return {
        "status": "HASH_BOUND_SOURCE_AUDIT_COMPLETE",
        "tracked_source_sha256": actual_tracked,
        "isaac_source_sha256": actual_isaac,
        "isaac_lab_commit": isaac_repository["commit"],
        "isaac_lab_tag": isaac_repository["tag"],
        "native_physx_version": "5.9.0",
        "controller_correspondence": (
            "integer PD, target slew, effort/rate/power/work intersection and "
            "reference reset semantics are present in both hash-bound paths"
        ),
        "material_projection": (
            "mirror exports material IDs, but neither mirror JSON nor derived "
            "USD supplies exact material coefficient descriptors"
        ),
    }


def audit_contact_wrench_ownership(*, repository_root: Path) -> dict[str, Any]:
    root = repository_root.resolve()
    candidates = [
        *sorted((root / "crates").rglob("*.rs")),
        *sorted((root / "docs" / "architecture").rglob("*.md")),
    ]
    matches: list[str] = []
    needles = ("contact_wrench", "contact-wrench", "ContactWrench")
    for path in candidates:
        text = path.read_text(encoding="utf-8")
        if any(needle in text for needle in needles):
            matches.append(str(path.relative_to(root)))
    return {
        "status": "UNOWNED_IN_CURRENT_ENGINE_CONTRACTS",
        "searched_roots": ["crates/**/*.rs", "docs/architecture/**/*.md"],
        "contract_owner_matches": matches,
        "frame": None,
        "component_order": None,
        "application_point_convention": None,
        "future_formulation_may_not_invent_authoritative_runtime_semantics": True,
    }


def classify_model_identity(
    *,
    profile: Mapping[str, Any],
    descriptor_identity: Mapping[str, Any],
    translation_identity: Mapping[str, Any],
    wrench_ownership: Mapping[str, Any],
) -> dict[str, Any]:
    expected = profile["expected_shared_identity"]
    exact_fields = (
        "body_schema_hash",
        "compiled_descriptor_hash",
        "total_mass_microkilograms",
        "non_colliding_carrier_count",
        "collision_exclusion_count",
        "material_id_counts",
        "coordinate_mapping",
    )
    if any(descriptor_identity[field] != expected[field] for field in exact_fields):
        raise ValueError("shared descriptor identity differs")
    blockers: list[str] = []
    if (
        len(descriptor_identity["material_id_counts"]) > 1
        and not descriptor_identity["material_coefficients_present"]
        and not descriptor_identity["material_combine_profile_present"]
        and not translation_identity["material_binding_present"]
    ):
        blockers.append("CANONICAL_MATERIAL_DESCRIPTORS_ABSENT")
    native = profile["expected_native_backend"]
    isaac = profile["expected_isaac_backend"]
    if (
        native["static_friction"] != isaac["static_friction"]
        or native["dynamic_friction"] != isaac["dynamic_friction"]
        or native["restitution"] != isaac["restitution"]
    ):
        blockers.append("BACKEND_MATERIAL_DEFAULTS_DIVERGE")
    if (
        wrench_ownership.get("status") != "OWNED"
        or wrench_ownership.get("frame") is None
        or wrench_ownership.get("component_order") is None
    ):
        blockers.append("CONTACT_WRENCH_CONVENTION_UNOWNED")
    if blockers != profile["expected_blocking_reasons"]:
        raise ValueError("R109 blocking model-identity reasons differ")
    return {
        "status": "FAIL",
        "blocking_reasons": blockers,
        "blocking_reason_count": len(blockers),
        "nonblocking_declared_mirror_divergence_count": 2,
        "single_backend_neutral_dynamics_model_available": False,
        "canonical_material_contract_satisfied": False,
        "contact_wrench_contract_satisfied": False,
        "kinematic_shared_identity_satisfied": True,
        "gate_decision": profile["decision"]["fail"],
    }


def _require_fragments(text: str, fragments: tuple[str, ...], label: str) -> None:
    missing = [fragment for fragment in fragments if fragment not in text]
    if missing:
        raise ValueError(f"{label} differs: {missing[0]}")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    shared = profile.get("expected_shared_identity", {})
    native = profile.get("expected_native_backend", {})
    isaac = profile.get("expected_isaac_backend", {})
    decision = profile.get("decision", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("preflight_id") != PREFLIGHT_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("claim") != "OptimizerFreeDynamicsModelIdentityPreflightOnly"
        or scope.get("run_id") != "R109"
        or scope.get("body_count") != 24
        or scope.get("joint_count") != 23
        or scope.get("actuator_count") != 23
        or scope.get("collider_count") != 19
        or scope.get("motor_hz") != 60
        or scope.get("physics_hz") != 240
        or scope.get("solver_runs") != 0
        or scope.get("candidate_construction") is not False
        or scope.get("physx_execution") is not False
        or shared.get("total_mass_microkilograms") != 75_337_000
        or shared.get("material_id_counts")
        != {
            "physics-material.humanoid-body.v1": 17,
            "physics-material.humanoid-sole.v1": 2,
        }
        or native.get("position_iterations") != 16
        or native.get("velocity_iterations") != 4
        or native.get("static_friction") != 0.8
        or native.get("dynamic_friction") != 0.7
        or native.get("enhanced_determinism") is not True
        or isaac.get("position_iterations") != 8
        or isaac.get("velocity_iterations") != 4
        or isaac.get("static_friction") != 0.5
        or isaac.get("dynamic_friction") != 0.5
        or isaac.get("enhanced_determinism") is not False
        or profile.get("expected_blocking_reasons")
        != [
            "CANONICAL_MATERIAL_DESCRIPTORS_ABSENT",
            "BACKEND_MATERIAL_DEFAULTS_DIVERGE",
            "CONTACT_WRENCH_CONVENTION_UNOWNED",
        ]
        or decision.get("pass")
        != "PERMIT_SEPARATE_BOUNDED_KTO_EXECUTION_FORMULATION_ONLY"
        or decision.get("fail") != "STOP_INVALID_MODEL_LINEAGE"
        or decision.get("expected") != "STOP_INVALID_MODEL_LINEAGE"
        or bounded.get("material_lineage_research") != "REQUIRED"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
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
        raise ValueError("dynamics-model identity profile is invalid")
