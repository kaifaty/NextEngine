from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from next_lab.canonical_material_lineage_implementation import (
    _validate_profile,
    audit_implementation_sources,
    audit_mirror_v2,
    tracked_source_paths,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-canonical-material-lineage-implementation-r111.v1.json"
)


class CanonicalMaterialLineageImplementationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_static_r112(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(self.profile["scope"]["physx_scene_runs"], 0)
        self.assertEqual(
            self.profile["decision"]["pass"],
            "PERMIT_R112_DERIVED_USD_MATERIAL_LINEAGE_IMPLEMENTATION_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["training"], "NOT_AUTHORIZED"
        )

    def test_repository_sources_close_native_but_not_r112(self) -> None:
        sources = {
            name: path.read_text(encoding="utf-8")
            for name, path in tracked_source_paths(ROOT).items()
        }
        result = audit_implementation_sources(sources)
        self.assertTrue(result["native_ambient_material_removed"])
        self.assertTrue(result["unequal_materials_fail_closed"])
        self.assertFalse(result["derived_usd_material_lineage_implemented"])
        self.assertEqual(result["status"], "R111_COMPLETE_R112_PENDING")

        altered = dict(sources)
        altered["physx_bridge"] += "\ncreateMaterial(0.8F, 0.7F, 0.0F);\n"
        with self.assertRaisesRegex(ValueError, "ambient material"):
            audit_implementation_sources(altered)

    def test_compiled_mirror_audit_is_exact_and_hash_separates_v2(self) -> None:
        descriptor = _minimal_descriptor(self.profile)
        result = audit_mirror_v2(self.profile, descriptor)
        self.assertNotEqual(
            result["compiled_descriptor_v3_hash"],
            result["legacy_compiled_descriptor_hash"],
        )
        self.assertEqual(len(result["material_descriptors"]), 3)
        self.assertFalse(result["legacy_bytes_reinterpreted"])

        altered = copy.deepcopy(descriptor)
        altered["materials"][2]["spinning_friction_q16"] = 1
        with self.assertRaisesRegex(ValueError, "compiled mirror"):
            audit_mirror_v2(self.profile, altered)


def _minimal_descriptor(profile: dict[str, object]) -> dict[str, object]:
    source = profile["source"]
    assert isinstance(source, dict)
    return {
        "schema_version": 2,
        "compiled_descriptor_schema_version": 3,
        "translator_id": "nextengine.isaac.biomechanics-mirror.v2",
        "body_schema_hash": source["body_schema_hash"],
        "compiled_descriptor_hash": source["compiled_descriptor_v3_hash"],
        "material_lineage_hash": source["material_lineage_hash"],
        "ground_material_id": "physics-material.humanoid-ground.v1",
        "materials": profile["expected_materials"],
        "material_combine_profile": profile["expected_combine_profile"],
        "collider_material_assignment_counts": profile[
            "expected_collider_material_assignment_counts"
        ],
    }


if __name__ == "__main__":
    unittest.main()
