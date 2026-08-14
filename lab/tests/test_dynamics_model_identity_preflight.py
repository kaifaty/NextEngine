from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from next_lab.dynamics_model_identity_preflight import (
    _validate_profile,
    audit_contact_wrench_ownership,
    audit_descriptor_identity,
    classify_model_identity,
    validate_translation_identity,
)
from next_lab.usd_translation import (
    BIOMECHANICS_TRANSLATOR_VERSION,
    render_usda,
)

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = Path(__file__).parent / "fixtures/biomechanics_motor_mirror_v1.json"
PROFILE = (
    Path(__file__).resolve().parents[1]
    / "profiles"
    / "humanoid-dynamics-model-identity-preflight-r109.v1.json"
)


class DynamicsModelIdentityPreflightTests(unittest.TestCase):
    def test_tracked_profile_freezes_stop_without_execution(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        self.assertEqual(profile["decision"]["expected"], "STOP_INVALID_MODEL_LINEAGE")
        self.assertEqual(profile["scope"]["solver_runs"], 0)
        self.assertFalse(profile["scope"]["candidate_construction"])
        self.assertFalse(profile["scope"]["physx_execution"])

    def test_descriptor_exposes_material_ids_without_coefficients(self) -> None:
        descriptor = json.loads(FIXTURE.read_bytes())
        identity = audit_descriptor_identity(descriptor)
        self.assertEqual(identity["body_count"], 24)
        self.assertEqual(identity["collider_count"], 19)
        self.assertEqual(
            identity["material_id_counts"],
            {
                "physics-material.humanoid-body.v1": 17,
                "physics-material.humanoid-sole.v1": 2,
            },
        )
        self.assertFalse(identity["material_descriptor_array_present"])
        self.assertFalse(identity["material_coefficients_present"])
        self.assertFalse(identity["material_combine_profile_present"])

    def test_stored_translation_can_be_byte_exact_and_still_lack_materials(
        self,
    ) -> None:
        descriptor = json.loads(FIXTURE.read_bytes())
        usd = render_usda(descriptor).encode("utf-8")
        usd_hash = hashlib.sha256(usd).hexdigest()
        manifest = {
            "body_schema_hash": descriptor["body_schema_hash"],
            "compiled_descriptor_hash": descriptor["compiled_descriptor_hash"],
            "schema_version": 2,
            "translator_version": BIOMECHANICS_TRANSLATOR_VERSION,
            "usd_path": "humanoid.usda",
            "usd_sha256": usd_hash,
        }
        manifest_bytes = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode(
            "utf-8"
        )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            usd_path = root / "humanoid.usda"
            manifest_path = root / "translation-manifest.json"
            usd_path.write_bytes(usd)
            manifest_path.write_bytes(manifest_bytes)
            identity = validate_translation_identity(
                descriptor=descriptor,
                manifest_path=manifest_path,
                usd_path=usd_path,
                expected_manifest_sha256=hashlib.sha256(manifest_bytes).hexdigest(),
                expected_usd_sha256=usd_hash,
            )
        self.assertTrue(identity["rendered_usd_matches_stored_bytes"])
        self.assertFalse(identity["material_binding_present"])
        self.assertFalse(identity["scene_gravity_authored_in_usd"])

    def test_missing_material_and_wrench_ownership_blocks_model_identity(
        self,
    ) -> None:
        profile = json.loads(PROFILE.read_bytes())
        descriptor = audit_descriptor_identity(json.loads(FIXTURE.read_bytes()))
        result = classify_model_identity(
            profile=profile,
            descriptor_identity=descriptor,
            translation_identity={"material_binding_present": False},
            wrench_ownership={
                "status": "UNOWNED_IN_CURRENT_ENGINE_CONTRACTS",
                "frame": None,
                "component_order": None,
            },
        )
        self.assertEqual(result["status"], "FAIL")
        self.assertEqual(result["gate_decision"], "STOP_INVALID_MODEL_LINEAGE")
        self.assertFalse(result["single_backend_neutral_dynamics_model_available"])

    def test_engine_contracts_do_not_own_contact_wrench_coordinates(self) -> None:
        identity = audit_contact_wrench_ownership(repository_root=ROOT)
        self.assertEqual(identity["status"], "UNOWNED_IN_CURRENT_ENGINE_CONTRACTS")
        self.assertEqual(identity["contract_owner_matches"], [])


if __name__ == "__main__":
    unittest.main()
