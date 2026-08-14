from __future__ import annotations

import hashlib
import json
import re
from collections import Counter
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

from next_lab.biomechanics_material_lineage import (
    EXPECTED_COMBINE_PROFILE,
    EXPECTED_MATERIALS,
)
from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.motor_mirror import load_json, validate_current_biomechanics_descriptor
from next_lab.usd_translation import render_usda, validate_translation_bundle

AUDIT_ID = "nextengine.humanoid-derived-usd-material-lineage.v1"
CHECK_ID = "MODEL-MIRROR-P2"

TRACKED_SOURCE_PATHS = {
    "material_lineage": "lab/next_lab/biomechanics_material_lineage.py",
    "motor_mirror": "lab/next_lab/motor_mirror.py",
    "usd_translation": "lab/next_lab/usd_translation.py",
    "isaac_reference_env": "lab/next_lab/isaac_reference_env.py",
    "cli": "lab/next_lab/cli.py",
    "isaac_biomechanics_check": "lab/scripts/isaac_biomechanics_check.py",
    "legacy_mirror_v1": "lab/tests/fixtures/biomechanics_motor_mirror_v1.json",
}


def tracked_source_paths(repository_root: Path) -> dict[str, Path]:
    root = repository_root.resolve()
    return {name: root / relative for name, relative in TRACKED_SOURCE_PATHS.items()}


def build_derived_usd_material_lineage_report(
    *,
    profile_path: Path,
    r111_report_path: Path,
    r111_profile_path: Path,
    descriptor_bytes: bytes,
    translation_manifest_path: Path,
    humanoid_usd_path: Path,
    ground_usd_path: Path,
    tracked_sources: Mapping[str, Path],
    validation_results: Sequence[Mapping[str, Any]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    paths = (
        profile_path.resolve(),
        r111_report_path.resolve(),
        r111_profile_path.resolve(),
        translation_manifest_path.resolve(),
        humanoid_usd_path.resolve(),
        ground_usd_path.resolve(),
        tool_path.resolve(),
    )
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("R112 implementation-audit input is absent")
    sources = {name: path.resolve() for name, path in tracked_sources.items()}
    if set(sources) != set(TRACKED_SOURCE_PATHS) or any(
        not path.is_file() for path in sources.values()
    ):
        raise FileNotFoundError("R112 tracked source closure is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(profile, repository)
    r111 = _validate_r111(profile, r111_report_path, r111_profile_path)
    descriptor = json.loads(descriptor_bytes)
    validate_current_biomechanics_descriptor(descriptor)
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != profile["source"]["mirror_v2_sha256"]
    ):
        raise ValueError("R112 mirror V2 byte identity mismatch")
    if (
        descriptor["body_schema_hash"] != profile["source"]["body_schema_hash"]
        or descriptor["compiled_descriptor_hash"]
        != profile["source"]["compiled_descriptor_v3_hash"]
        or descriptor["material_lineage_hash"]
        != profile["source"]["material_lineage_hash"]
    ):
        raise ValueError("R112 compiled descriptor lineage mismatch")

    manifest = validate_translation_bundle(
        descriptor,
        translation_manifest_path,
        humanoid_usd_path=humanoid_usd_path,
        ground_usd_path=ground_usd_path,
    )
    if (
        sha256(translation_manifest_path)
        != profile["expected_translation"]["translation_manifest_file_sha256"]
    ):
        raise ValueError("R112 translation manifest byte identity mismatch")
    source_audit = audit_r112_sources(
        {name: path.read_text(encoding="utf-8") for name, path in sources.items()}
    )
    translation_audit = audit_translation_outputs(
        profile,
        descriptor,
        manifest,
        humanoid_usd_path.read_bytes(),
        ground_usd_path.read_bytes(),
    )
    legacy_usd = render_usda(load_json(sources["legacy_mirror_v1"]))
    if (
        sha256(sources["legacy_mirror_v1"])
        != profile["source"]["legacy_mirror_v1_sha256"]
        or hashlib.sha256(legacy_usd.encode("utf-8")).hexdigest()
        != profile["source"]["legacy_usd_v1_sha256"]
    ):
        raise ValueError("R112 legacy mirror/USD identity changed")
    validations = _validate_results(profile, validation_results)

    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "audit_id": AUDIT_ID,
        "status": "PASS",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["pass"],
        "scope": profile["scope"],
        "frozen_invariants": profile["frozen_invariants"],
        "source_implementation": source_audit,
        "translation_lineage": translation_audit,
        "source_gate": {
            "r111_status": r111["status"],
            "r111_gate_decision": r111["gate_decision"],
            "r111_report_sha256": r111["report_sha256"],
        },
        "r109_blocker_disposition": {
            "CANONICAL_MATERIAL_DESCRIPTORS_ABSENT": "CLOSED_R111",
            "BACKEND_MATERIAL_DEFAULTS_DIVERGE": "CLOSED_R111_NATIVE_R112_DERIVED",
            "CONTACT_WRENCH_CONVENTION_UNOWNED": "CLOSED_R110_SOLVER_PRIVATE_POINT_FORCES",
            "clean_model_identity_recheck": "R113_REQUIRED",
            "model_identity_pass_claim": False,
            "runtime_correspondence_claim": False,
        },
        "validation_results": validations,
        "next_smallest_action": profile["decision"]["next_smallest_action"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r111_report_file_sha256": sha256(r111_report_path),
            "r111_profile_sha256": sha256(r111_profile_path),
            "mirror_v2_sha256": hashlib.sha256(descriptor_bytes).hexdigest(),
            "humanoid_usd_sha256": sha256(humanoid_usd_path),
            "ground_usd_sha256": sha256(ground_usd_path),
            "translation_manifest_file_sha256": sha256(translation_manifest_path),
            "tracked_source_sha256": {
                name: sha256(path) for name, path in sources.items()
            },
            "tool_sha256": sha256(tool_path),
            "audit_module_sha256": sha256(Path(__file__).resolve()),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "derived_usd_material_implementations": 1,
        "isaac_ground_lineage_implementations": 1,
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


def audit_r112_sources(sources: Mapping[str, str]) -> dict[str, Any]:
    if set(sources) != set(TRACKED_SOURCE_PATHS):
        raise ValueError("R112 tracked source set differs")
    required = {
        "material_lineage": (
            'BIOMECHANICS_TRANSLATOR_ID_V2 = "nextengine.isaac.biomechanics-mirror.v2"',
            "EXPECTED_DESCRIPTOR_FIELDS",
            "validate_current_material_lineage",
            "current biomechanics collider material closure mismatch",
        ),
        "motor_mirror": (
            "BIOMECHANICS_TRANSLATOR_ID_V2",
            "validate_current_biomechanics_descriptor",
            "current biomechanics mirror V2 is required",
        ),
        "usd_translation": (
            "PhysicsMaterialDescriptorV2",
            "PhysicsMaterialCombineProfileV1",
            "material:binding:physics",
            "render_ground_usda",
            "compiled_descriptor_hash",
            "validate_translation_bundle",
            "current translation identity collision",
        ),
        "isaac_reference_env": (
            "validate_current_biomechanics_descriptor",
            "self.translation_manifest = validate_translation_bundle",
            "self.cfg.ground.func",
            "translation-manifest.json",
        ),
        "cli": (
            "BIOMECHANICS_TRANSLATOR_ID_V2",
            "translate_to_store",
        ),
        "isaac_biomechanics_check": (
            'parser.add_argument("--ground-usd"',
            "validate_translation_bundle",
            "sim_utils.UsdFileCfg",
        ),
    }
    for name, tokens in required.items():
        if any(token not in sources[name] for token in tokens):
            raise ValueError(f"R112 implementation token differs: {name}")
    isaac = sources["isaac_reference_env"]
    if any(
        token in isaac
        for token in ("GroundPlaneCfg", "RigidBodyMaterialCfg", "spawn_ground_plane")
    ):
        raise ValueError("R112 Isaac default material path remains reachable")
    if isaac.index(
        "self.translation_manifest = validate_translation_bundle"
    ) > isaac.index("self.cfg.ground.func"):
        raise ValueError("R112 Isaac bundle is not validated before ground spawn")
    return {
        "mirror_v2_required": True,
        "unknown_descriptor_fields_fail_closed": True,
        "material_catalog_and_assignment_counts_exact": True,
        "translation_path_binds_compiled_descriptor_hash": True,
        "translation_identity_collisions_fail_closed": True,
        "isaac_validates_complete_bundle_before_scene": True,
        "isaac_ground_uses_derived_usd": True,
        "isaac_material_defaults_reachable": False,
        "legacy_v1_reinterpreted": False,
        "status": "R112_IMPLEMENTED_R113_PENDING",
    }


def audit_translation_outputs(
    profile: Mapping[str, Any],
    descriptor: Mapping[str, Any],
    manifest: Mapping[str, Any],
    humanoid_usd: bytes,
    ground_usd: bytes,
) -> dict[str, Any]:
    expected = profile["expected_translation"]
    humanoid_hash = hashlib.sha256(humanoid_usd).hexdigest()
    ground_hash = hashlib.sha256(ground_usd).hexdigest()
    if (
        humanoid_hash != expected["humanoid_usd_sha256"]
        or ground_hash != expected["ground_usd_sha256"]
        or manifest.get("schema_version") != expected["manifest_schema_version"]
        or manifest.get("translator_id") != expected["mirror_translator_id"]
        or manifest.get("translator_version") != expected["usd_translator_version"]
        or manifest.get("descriptor_schema_version")
        != expected["descriptor_schema_version"]
        or manifest.get("compiled_descriptor_schema_version")
        != expected["compiled_descriptor_schema_version"]
        or manifest.get("descriptor_content_sha256")
        != expected["descriptor_content_sha256"]
        or manifest.get("usd_path") != expected["humanoid_usd_path"]
        or manifest.get("ground_usd_path") != expected["ground_usd_path"]
    ):
        raise ValueError("R112 translation identity differs")
    humanoid = humanoid_usd.decode("utf-8")
    ground = ground_usd.decode("utf-8")
    humanoid_bindings = re.findall(
        r"rel material:binding:physics = </Humanoid/Materials/([^>]+)>", humanoid
    )
    ground_bindings = re.findall(
        r"rel material:binding:physics = </Ground/Materials/([^>]+)>", ground
    )
    expected_binding_counts = Counter(
        {
            "physics_material_humanoid_body_v1": expected["body_binding_count"],
            "physics_material_humanoid_sole_v1": expected["sole_binding_count"],
        }
    )
    if (
        humanoid.count('def Material "') != expected["humanoid_material_prim_count"]
        or len(humanoid_bindings) != expected["humanoid_physics_binding_count"]
        or Counter(humanoid_bindings) != expected_binding_counts
        or ground.count('def Material "') != expected["ground_material_prim_count"]
        or len(ground_bindings) != expected["ground_physics_binding_count"]
        or ground_bindings != ["physics_material_humanoid_ground_v1"]
        or 'def Plane "Collision"' not in ground
    ):
        raise ValueError("R112 material prim/binding closure differs")
    combined = humanoid + ground
    required_tokens = (
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
    if any(token not in combined for token in required_tokens) or any(
        token in combined
        for token in (
            "physics:staticFriction = 0.5\n",
            "physics:dynamicFriction = 0.5\n",
        )
    ):
        raise ValueError("R112 exact material metadata differs")
    return {
        "body_schema_hash": descriptor["body_schema_hash"],
        "compiled_descriptor_v3_hash": descriptor["compiled_descriptor_hash"],
        "material_lineage_hash": descriptor["material_lineage_hash"],
        "descriptor_content_sha256": manifest["descriptor_content_sha256"],
        "humanoid_usd_sha256": humanoid_hash,
        "ground_usd_sha256": ground_hash,
        "humanoid_material_prim_count": humanoid.count('def Material "'),
        "humanoid_physics_binding_count": len(humanoid_bindings),
        "ground_material_prim_count": ground.count('def Material "'),
        "ground_physics_binding_count": len(ground_bindings),
        "collider_material_binding_counts": dict(
            sorted(Counter(humanoid_bindings).items())
        ),
        "ground_material_binding": ground_bindings[0],
        "material_descriptors": descriptor["materials"],
        "material_combine_profile": descriptor["material_combine_profile"],
        "legacy_bytes_reinterpreted": False,
        "status": "COMPLETE_NEW_DERIVED_LINEAGE",
    }


def _validate_profile(profile: Mapping[str, Any]) -> None:
    if (
        profile.get("schema_version") != 1
        or profile.get("audit_id") != AUDIT_ID
        or profile.get("claim")
        != "DerivedUsdAndIsaacMaterialLineageWithoutSceneExecution"
        or profile.get("scope", {}).get("run_id") != "R112"
        or profile.get("scope", {}).get("product_check") != CHECK_ID
        or profile.get("scope", {}).get("physx_scene_runs") != 0
        or profile.get("expected_materials") != EXPECTED_MATERIALS
        or profile.get("expected_combine_profile") != EXPECTED_COMBINE_PROFILE
        or profile.get("decision", {}).get("pass")
        != "PERMIT_R113_CLEAN_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY"
        or profile.get("bounded_acceptance", {}).get("physx") != "NOT_AUTHORIZED"
        or profile.get("bounded_acceptance", {}).get("training") != "NOT_AUTHORIZED"
    ):
        raise ValueError("R112 profile differs")
    commands = profile.get("validation_commands", ())
    ids = [row.get("id") for row in commands]
    if (
        not commands
        or len(ids) != len(set(ids))
        or any(not row.get("arguments") for row in commands)
    ):
        raise ValueError("R112 validation command closure differs")


def _validate_repository(
    profile: Mapping[str, Any], repository: Mapping[str, Any]
) -> None:
    if (
        repository.get("dirty") is not False
        or repository.get("implementation_commit_is_ancestor") is not True
        or repository.get("implementation_commit")
        != profile["source"]["implementation_commit"]
    ):
        raise ValueError("R112 repository lineage differs")


def _validate_r111(
    profile: Mapping[str, Any], report_path: Path, profile_path: Path
) -> dict[str, Any]:
    expected = profile["source"]["r111"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R112 R111 source identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        report.get("status") != "PASS"
        or report.get("gate_decision")
        != "PERMIT_R112_DERIVED_USD_MATERIAL_LINEAGE_IMPLEMENTATION_ONLY"
        or embedded != expected["report_sha256"]
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("physx_scene_runs") != 0
        or report.get("training_runs") != 0
    ):
        raise ValueError("R112 R111 source contract differs")
    return report


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, Any]]
) -> list[dict[str, Any]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    actual = [row.get("id") for row in results]
    if actual != expected or any(row.get("status") != "PASS" for row in results):
        raise ValueError("R112 validation result differs")
    return [dict(row) for row in results]
