from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path

from next_lab.biomechanics_material_lineage import BIOMECHANICS_TRANSLATOR_ID_V2
from next_lab.derived_usd_material_lineage import (
    _validate_profile,
    audit_r112_sources,
    audit_translation_outputs,
    tracked_source_paths,
)
from next_lab.motor_mirror import load_json
from next_lab.usd_translation import translate_to_store

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-derived-usd-material-lineage-r112.v1.json"
LEGACY = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"


class DerivedUsdMaterialLineageTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_report_only_r113(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(self.profile["scope"]["physx_scene_runs"], 0)
        self.assertEqual(
            self.profile["decision"]["pass"],
            "PERMIT_R113_CLEAN_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["quantization_aware_kto"],
            "NOT_AUTHORIZED",
        )

    def test_repository_sources_remove_isaac_material_defaults(self) -> None:
        sources = {
            name: path.read_text(encoding="utf-8")
            for name, path in tracked_source_paths(ROOT).items()
        }
        result = audit_r112_sources(sources)
        self.assertTrue(result["isaac_validates_complete_bundle_before_scene"])
        self.assertFalse(result["isaac_material_defaults_reachable"])

        altered = dict(sources)
        altered["isaac_reference_env"] += "\nGroundPlaneCfg()\n"
        with self.assertRaisesRegex(ValueError, "default material path"):
            audit_r112_sources(altered)

    def test_translation_audit_closes_all_twenty_bindings(self) -> None:
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
            humanoid = (bundle / manifest["usd_path"]).read_bytes()
            ground = (bundle / manifest["ground_usd_path"]).read_bytes()
            result = audit_translation_outputs(
                self.profile, descriptor, manifest, humanoid, ground
            )
            self.assertEqual(result["humanoid_physics_binding_count"], 19)
            self.assertEqual(result["ground_physics_binding_count"], 1)
            self.assertFalse(result["legacy_bytes_reinterpreted"])

            altered = humanoid.replace(
                b"physics_material_humanoid_sole_v1>",
                b"physics_material_humanoid_body_v1>",
                1,
            )
            with self.assertRaisesRegex(ValueError, "translation identity"):
                audit_translation_outputs(
                    self.profile, descriptor, manifest, altered, ground
                )


def _descriptor_v2(profile: dict[str, object]) -> dict[str, object]:
    descriptor = load_json(LEGACY)
    source = profile["source"]
    assert isinstance(source, dict)
    descriptor.update(
        {
            "schema_version": 2,
            "translator_id": BIOMECHANICS_TRANSLATOR_ID_V2,
            "compiled_descriptor_schema_version": 3,
            "compiled_descriptor_hash": source["compiled_descriptor_v3_hash"],
            "material_lineage_hash": source["material_lineage_hash"],
            "ground_material_id": "physics-material.humanoid-ground.v1",
            "materials": copy.deepcopy(profile["expected_materials"]),
            "material_combine_profile": copy.deepcopy(
                profile["expected_combine_profile"]
            ),
            "collider_material_assignment_counts": [
                {
                    "collider_count": 17,
                    "material_id": "physics-material.humanoid-body.v1",
                },
                {
                    "collider_count": 2,
                    "material_id": "physics-material.humanoid-sole.v1",
                },
            ],
        }
    )
    return descriptor


if __name__ == "__main__":
    unittest.main()
