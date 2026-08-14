from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path

from next_lab.biomechanics_material_lineage import BIOMECHANICS_TRANSLATOR_ID_V2
from next_lab.clean_dynamics_model_identity_preflight import (
    _validate_profile,
    audit_descriptor_identity,
    audit_point_force_identity,
    audit_translation_identity,
    classify_model_identity,
)
from next_lab.motor_mirror import load_json
from next_lab.usd_translation import translate_to_store

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT / "lab/profiles/humanoid-clean-dynamics-model-identity-preflight-r113.v1.json"
)
LEGACY = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"


class CleanDynamicsModelIdentityPreflightTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_r114_formulation(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(self.profile["scope"]["physx_scene_runs"], 0)
        self.assertEqual(
            self.profile["decision"]["expected"],
            "PERMIT_SEPARATE_BOUNDED_KTO_EXECUTION_FORMULATION_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["quantization_aware_kto_solve"],
            "NOT_AUTHORIZED",
        )

    def test_current_descriptor_closes_the_r109_material_gap(self) -> None:
        descriptor = _descriptor_v2(self.profile)
        result = audit_descriptor_identity(self.profile, descriptor)
        self.assertEqual(result["status"], "COMPLETE_STRUCTURAL_AND_MATERIAL_IDENTITY")
        self.assertTrue(result["material_coefficients_present"])
        self.assertTrue(result["material_combine_profile_present"])

        altered = copy.deepcopy(descriptor)
        altered["materials"][2]["dynamic_friction_q16"] -= 1
        with self.assertRaisesRegex(ValueError, "material"):
            audit_descriptor_identity(self.profile, altered)

    def test_exact_r112_bundle_closes_all_material_bindings(self) -> None:
        descriptor = _descriptor_v2(self.profile)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = translate_to_store(descriptor, root, ROOT)
            bundle = (
                root
                / "derived"
                / descriptor["body_schema_hash"]
                / descriptor["compiled_descriptor_hash"]
            )
            result = audit_translation_identity(
                self.profile,
                descriptor,
                translation_manifest_path=bundle / "translation-manifest.json",
                humanoid_usd_path=bundle / manifest["usd_path"],
                ground_usd_path=bundle / manifest["ground_usd_path"],
            )
            self.assertEqual(result["humanoid_physics_binding_count"], 19)
            self.assertEqual(result["ground_physics_binding_count"], 1)
            self.assertFalse(result["isaac_default_material_reachable"])

            humanoid_path = bundle / manifest["usd_path"]
            humanoid_path.write_bytes(humanoid_path.read_bytes() + b"\n")
            with self.assertRaisesRegex(ValueError, "translation byte identity"):
                audit_translation_identity(
                    self.profile,
                    descriptor,
                    translation_manifest_path=bundle / "translation-manifest.json",
                    humanoid_usd_path=humanoid_path,
                    ground_usd_path=bundle / manifest["ground_usd_path"],
                )

    def test_point_force_and_final_classification_are_fail_closed(self) -> None:
        r110 = {
            "point_contact_force_coordinates": copy.deepcopy(
                self.profile["expected_point_force_identity"]
            )
        }
        point_force = audit_point_force_identity(self.profile, r110)
        result = classify_model_identity(
            profile=self.profile,
            descriptor_identity={"status": "COMPLETE_STRUCTURAL_AND_MATERIAL_IDENTITY"},
            translation_identity={"status": "COMPLETE_EXACT_R112_TRANSLATION_IDENTITY"},
            source_identity={
                "native_ambient_material_reachable": False,
                "isaac_ambient_material_reachable": False,
            },
            point_force_identity=point_force,
        )
        self.assertEqual(result["status"], "PASS")
        self.assertEqual(result["blocking_reasons"], [])
        self.assertFalse(result["runtime_correspondence_claim"])

        altered = copy.deepcopy(r110)
        altered["point_contact_force_coordinates"]["force_component_order"] = [
            "right_newtons",
            "normal_newtons",
            "forward_newtons",
        ]
        with self.assertRaisesRegex(ValueError, "point-force identity"):
            audit_point_force_identity(self.profile, altered)


def _descriptor_v2(profile: dict[str, object]) -> dict[str, object]:
    descriptor = load_json(LEGACY)
    shared = profile["expected_shared_identity"]
    assert isinstance(shared, dict)
    descriptor.update(
        {
            "schema_version": 2,
            "translator_id": BIOMECHANICS_TRANSLATOR_ID_V2,
            "compiled_descriptor_schema_version": 3,
            "compiled_descriptor_hash": shared["compiled_descriptor_hash"],
            "material_lineage_hash": shared["material_lineage_hash"],
            "ground_material_id": shared["ground_material_id"],
            "materials": copy.deepcopy(profile["expected_materials"]),
            "material_combine_profile": copy.deepcopy(
                profile["expected_combine_profile"]
            ),
            "collider_material_assignment_counts": copy.deepcopy(
                profile["expected_collider_material_assignment_counts"]
            ),
        }
    )
    return descriptor


if __name__ == "__main__":
    unittest.main()
