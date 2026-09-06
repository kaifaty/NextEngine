from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v29_q1m_role_power_v1 as q1m  # noqa: E402


def canonical(value: object) -> bytes:
    return q1m.canonical_json(value)


def access(metadata_bytes: int = 1) -> dict[str, int]:
    return {
        "archive_member_bodies_read": 0,
        "audio_headers_parsed": 0,
        "force_sample_values_decoded": 0,
        "mesh_values_decoded": 0,
        "metadata_bytes_read": metadata_bytes,
        "network_requests": 0,
        "pcm_sample_values_decoded": 0,
        "protected_signal_values_decoded": 0,
        "source_payload_bytes_read": 0,
    }


def q0_group(
    *,
    source: str,
    object_id: int,
    material: str,
    available: bool,
) -> dict[str, object]:
    if source == "objectfolder":
        project = "objectfolder-real"
        publisher = "stanford-objectfolder"
        revision = "rendered-table-sha256-0111f57a"
        group_id = f"of-group-{object_id:03}"
        parent_id = None
        name = f"OF Object {object_id}"
    else:
        project = "ycb-impact-sounds"
        publisher = "iri-csic-upc-ctu"
        revision = "osf-bj5w8-2022-09-27"
        group_id = f"ycb-group-{object_id:03}"
        parent_id = f"ycb-parent-{object_id:03}"
        name = f"YCB Object {object_id}"
    group: dict[str, object] = {
        "acquisition_axes": {"material_identity": "known"},
        "candidate_available_after_historical_exclusion": available,
        "group_id": group_id,
        "historical_exclusion": None
        if available
        else {
            "historical_partition": "dev",
            "historical_role": "reject_parent",
            "historical_source_id": f"old-{group_id}",
            "reason": "historical_glass_split_identity",
        },
        "material_relation": q1m.material_relation(material),
        "object_id": str(object_id),
        "object_name": name,
        "primary_material": material,
        "project_id": project,
        "publisher_id": publisher,
        "recording_parent_id": parent_id,
        "revision": revision,
        "role_state": "unassigned",
        "secondary_material": None,
        "source_id": f"{source}-source-{object_id:03}",
    }
    if source == "ycb":
        group["missing_evaluation_axes"] = []
        group["missing_training_axes"] = []
    return group


def build_q0() -> tuple[dict[str, object], dict[str, object]]:
    groups: list[dict[str, object]] = []
    objectfolder_available = ["Steel"] * 17 + ["Iron"] * 15 + ["Ceramic"] * 63
    for object_id, material in enumerate(objectfolder_available, 1):
        groups.append(
            q0_group(
                source="objectfolder",
                object_id=object_id,
                material=material,
                available=True,
            )
        )
    for object_id in range(96, 101):
        groups.append(
            q0_group(
                source="objectfolder",
                object_id=object_id,
                material="Glass",
                available=False,
            )
        )

    ycb_available = ["Steel"] * 6 + ["Aluminium"] + ["Wood"] * 7
    for object_id, material in enumerate(ycb_available, 1):
        groups.append(
            q0_group(
                source="ycb",
                object_id=object_id,
                material=material,
                available=True,
            )
        )
    for object_id in range(15, 40):
        groups.append(
            q0_group(
                source="ycb",
                object_id=object_id,
                material="Ceramic",
                available=False,
            )
        )
    groups.sort(key=lambda group: str(group["group_id"]))
    q0_access = access()
    inventory = {
        "access": q0_access,
        "authority": (
            "SOURCE_IDENTITY_AND_DECLARED_AXIS_INVENTORY_ONLY / "
            "NO_ROLE_OR_ADMISSION_AUTHORITY"
        ),
        "groups": groups,
        "historical_glass_split": {
            "manifest_sha256": "a" * 64,
            "matched_objectfolder_groups": 5,
            "matched_ycb_groups": 25,
            "policy": "exclude_without_relabelling_or_threshold_use",
            "target_material_label": "Glass",
        },
        "schema": q1m.Q0_INVENTORY_SCHEMA,
        "sources": [
            {
                "landing_page_url": "https://objectfolder.example",
                "license_expression": "NOASSERTION",
                "project_id": "objectfolder-real",
                "publisher_id": "stanford-objectfolder",
                "redistribution_policy": "external_research_only",
                "revision": "rendered-table-sha256-0111f57a",
            },
            {
                "landing_page_url": "https://ycb.example",
                "license_expression": "NOASSERTION",
                "project_id": "ycb-impact-sounds",
                "publisher_id": "iri-csic-upc-ctu",
                "redistribution_policy": "external_research_only",
                "revision": "osf-bj5w8-2022-09-27",
            },
        ],
    }
    inventory_data = canonical(inventory)
    report = {
        "access": q0_access,
        "available_counts": {
            "broad_metal_candidates": 39,
            "exact_steel_candidates": 23,
            "non_metal_candidates": 70,
            "other_metal_candidates": 16,
            "primary_materials": {},
        },
        "decision": "Q0M_METAL_SOURCE_INVENTORY_FEASIBLE_Q1M_NEXT",
        "excluded_group_count": 30,
        "group_count": 139,
        "input_identities": {},
        "inventory_sha256": q1m.sha256_bytes(inventory_data),
        "minimums": {},
        "project_revision_counts": {},
        "q1_blockers": [],
        "schema": q1m.Q0_REPORT_SCHEMA,
    }
    return inventory, report


def build_exposure_and_identity() -> tuple[dict[str, object], dict[str, object]]:
    exposure_groups: list[dict[str, object]] = []
    identity_groups: list[dict[str, object]] = []
    for object_id in range(1, 101):
        physical_group_id = f"physical-{object_id:03}"
        if object_id <= 17:
            material = "Steel"
        elif object_id <= 32:
            material = "Iron"
        elif object_id <= 95:
            material = "Ceramic"
        else:
            material = "Glass"
        exposure_groups.append(
            {
                "candidate_route_count": 1,
                "candidate_source_ids": ["objectfolder_real_current"],
                "exposure_evidence": {},
                "exposure_state": "unexposed",
                "material_family": "Metal"
                if material in {"Steel", "Iron"}
                else material,
                "physical_group_id": physical_group_id,
            }
        )
        identity_groups.append(
            {
                "candidate_routes": [],
                "identity_basis": {},
                "material_family": "Metal"
                if material in {"Steel", "Iron"}
                else material,
                "members": [
                    {
                        "alias_basis": "identity_origin",
                        "evidence_sha256s": [],
                        "publisher_material": material,
                        "publisher_name": f"OF Object {object_id}",
                        "publisher_object_id": str(object_id),
                        "revision_id": "rendered-table-sha256-0111f57a",
                        "source_id": "objectfolder_real_current",
                        "source_tier": "T2_sparse_real_contact",
                    }
                ],
                "numeric_object_ids": [str(object_id)],
                "physical_group_id": physical_group_id,
            }
        )
    exposure = {
        "access": {
            **access(1000),
            "json_bytes_read": 1000,
            "json_files_parsed": 1,
        },
        "authority": {"public_contract": False, "runtime_consumer_allowed": False},
        "candidate_counts": {},
        "decision": "S0A_IDENTITY_EXPOSURE_PASS_METAL_SCOPE_REMAINS_POTENTIAL",
        "excluded_path_policy": {},
        "groups": exposure_groups,
        "historical_json_manifest": [],
        "input_identities": {},
        "metal_scope_remains_potential": True,
        "realimpact_name_evidence": {},
        "schema": q1m.EXPOSURE_SCHEMA,
    }
    identity = {
        "alias_edges": [],
        "authority": {"public_contract": False, "runtime_consumer_allowed": False},
        "discrepancies": [],
        "groups": identity_groups,
        "input_identities": {},
        "schema": q1m.IDENTITY_MAP_SCHEMA,
    }
    return exposure, identity


def build_ycb() -> dict[str, object]:
    available_materials = ["Steel"] * 6 + ["Aluminium"] + ["Wood"] * 7
    objects: list[dict[str, object]] = []
    for object_id in range(1, 78):
        if object_id <= len(available_materials):
            material = available_materials[object_id - 1]
        else:
            material = "Ceramic"
        parents = []
        if object_id <= 39:
            parents = [
                {
                    "group_id": f"ycb-group-{object_id:03}",
                    "parent_id": f"ycb-parent-{object_id:03}",
                }
            ]
        objects.append(
            {
                "exposure_state": "not_in_adapter_freshness_unassessed",
                "object_id": str(object_id),
                "primary_material": material,
                "recording_parents": parents,
            }
        )
    return {
        "acquisition_facts": {},
        "objects": objects,
        "project": {
            "component_revision": "2022-09-27T11:36:47.167853",
            "id": "ycb-impact-sounds",
            "publisher": "iri-csic-upc-ctu",
        },
        "schema": q1m.YCB_SCHEMA,
    }


def fixture_documents() -> tuple[dict[str, object], ...]:
    inventory, report = build_q0()
    exposure, identity = build_exposure_and_identity()
    return inventory, report, exposure, identity, build_ycb()


def build_fixture() -> dict[str, bytes]:
    inventory, report, exposure, identity, ycb = fixture_documents()
    inventory_data = canonical(inventory)
    report["inventory_sha256"] = q1m.sha256_bytes(inventory_data)
    return q1m.build_documents(
        inventory_data,
        canonical(report),
        canonical(exposure),
        canonical(identity),
        canonical(ycb),
    )


class Q1MRolePowerTests(unittest.TestCase):
    def test_repeat_exact_two_project_result_is_zero_signal_source_ood(self) -> None:
        first = build_fixture()
        second = build_fixture()
        self.assertEqual(first, second)
        report = json.loads(first["report.json"])
        self.assertEqual(
            report["decision"],
            "Q1M_SOURCE_POWER_INSUFFICIENT_SOURCE_GROWTH_REQUIRED",
        )
        self.assertEqual(report["role_assignment_count"], 0)
        self.assertTrue(
            all(
                value == 0
                for key, value in report["access"].items()
                if key != "metadata_bytes_read"
            )
        )

    def test_role_topology_floors_are_nine_full_and_four_protected(self) -> None:
        self.assertEqual(q1m.FULL_ROLE_PROJECT_FLOOR, 9)
        self.assertEqual(q1m.PROTECTED_PROJECT_FLOOR, 4)
        report = json.loads(build_fixture()["report.json"])
        self.assertIn("full_role_project_topology_requires_9_has_2", report["blockers"])
        self.assertIn(
            "protected_role_project_topology_requires_4_has_2", report["blockers"]
        )

    def test_each_protected_evaluation_gets_its_own_group_minimum(self) -> None:
        report = json.loads(build_fixture()["report.json"])
        self.assertEqual(
            report["protected_only_floor"],
            {"exact_steel_groups": 32, "non_metal_reject_groups": 70, "project_revisions": 4},
        )
        self.assertIn(
            "exact_steel_protected_minimum_requires_32_has_23", report["blockers"]
        )

    def test_steel_is_not_pooled_with_iron_or_aluminium(self) -> None:
        audit = json.loads(build_fixture()["metal-role-power-audit.json"])
        self.assertEqual(audit["target_policy"]["positive_material"], "Steel")
        self.assertEqual(audit["power"]["raw_available_counts"], {
            "broad_metal_candidates_diagnostic_only": 39,
            "exact_steel_candidates": 23,
            "non_metal_candidates": 70,
            "other_metal_candidates": 16,
        })

    def test_failed_gate_assigns_no_group_or_project_role(self) -> None:
        audit = json.loads(build_fixture()["metal-role-power-audit.json"])
        self.assertEqual(audit["role_assignment"], [])
        self.assertEqual(
            {group["role_state"] for group in audit["groups"]},
            {"unassigned_source_power_ood"},
        )

    def test_every_available_group_has_one_exposure_account(self) -> None:
        audit = json.loads(build_fixture()["metal-role-power-audit.json"])
        accounting = audit["exposure_accounting"]
        self.assertEqual(accounting["available_group_count"], 109)
        self.assertEqual(accounting["accounted_available_groups"], 109)
        self.assertEqual(len(audit["groups"]), 109)

    def test_historical_unexposed_never_becomes_current_freshness(self) -> None:
        audit = json.loads(build_fixture()["metal-role-power-audit.json"])
        self.assertEqual(
            audit["exposure_accounting"]["states"][
                "historical_unexposed_not_currently_certified"
            ],
            95,
        )
        self.assertTrue(all(not group["freshness_eligible"] for group in audit["groups"]))

    def test_duplicate_q0_group_is_rejected(self) -> None:
        inventory, report, exposure, identity, ycb = fixture_documents()
        inventory["groups"].append(copy.deepcopy(inventory["groups"][0]))
        report["group_count"] += 1
        inventory_data = canonical(inventory)
        report["inventory_sha256"] = q1m.sha256_bytes(inventory_data)
        with self.assertRaisesRegex(q1m.Q1MRolePowerError, "duplicate Q0"):
            q1m.build_documents(
                inventory_data,
                canonical(report),
                canonical(exposure),
                canonical(identity),
                canonical(ycb),
            )

    def test_ycb_parent_identity_mismatch_is_rejected(self) -> None:
        inventory, report, exposure, identity, ycb = fixture_documents()
        ycb["objects"][0]["recording_parents"][0]["parent_id"] = "different-parent"
        inventory_data = canonical(inventory)
        report["inventory_sha256"] = q1m.sha256_bytes(inventory_data)
        with self.assertRaisesRegex(q1m.Q1MRolePowerError, "exact parent identity mismatch"):
            q1m.build_documents(
                inventory_data,
                canonical(report),
                canonical(exposure),
                canonical(identity),
                canonical(ycb),
            )

    def test_ambiguous_objectfolder_join_is_rejected(self) -> None:
        inventory, report, exposure, identity, ycb = fixture_documents()
        duplicate = copy.deepcopy(identity["groups"][0])
        duplicate["physical_group_id"] = "different-physical-group"
        identity["groups"].append(duplicate)
        exposure["groups"].append(
            {
                **copy.deepcopy(exposure["groups"][0]),
                "physical_group_id": "different-physical-group",
            }
        )
        inventory_data = canonical(inventory)
        report["inventory_sha256"] = q1m.sha256_bytes(inventory_data)
        with self.assertRaisesRegex(q1m.Q1MRolePowerError, "ambiguous ObjectFolder"):
            q1m.build_documents(
                inventory_data,
                canonical(report),
                canonical(exposure),
                canonical(identity),
                canonical(ycb),
            )

    def test_q0_signal_access_drift_is_rejected(self) -> None:
        inventory, report, exposure, identity, ycb = fixture_documents()
        inventory["access"]["pcm_sample_values_decoded"] = 1
        report["access"]["pcm_sample_values_decoded"] = 1
        inventory_data = canonical(inventory)
        report["inventory_sha256"] = q1m.sha256_bytes(inventory_data)
        with self.assertRaisesRegex(q1m.Q1MRolePowerError, "signal-bearing"):
            q1m.build_documents(
                inventory_data,
                canonical(report),
                canonical(exposure),
                canonical(identity),
                canonical(ycb),
            )

    def test_symlink_input_and_existing_output_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.json"
            source.write_text("{}")
            link = root / "link.json"
            link.symlink_to(source)
            with self.assertRaisesRegex(q1m.Q1MRolePowerError, "symlink"):
                q1m.ensure_external_path(link, "fixture input", False)
            output = root / "output"
            output.mkdir()
            with self.assertRaisesRegex(q1m.Q1MRolePowerError, "refusing to replace"):
                q1m.ensure_external_path(output, "fixture output", True)


if __name__ == "__main__":
    unittest.main()
