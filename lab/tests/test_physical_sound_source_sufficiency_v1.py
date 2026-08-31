#!/usr/bin/env python3
"""Focused guards for the Physical Sound V15-S0c sufficiency certificate."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from collections import Counter
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_source_sufficiency_v1 as s0c  # noqa: E402


def digest(label: str) -> str:
    return s0c.sha256_bytes(label.encode("utf-8"))


def minimal_documents() -> dict[str, dict[str, object]]:
    basis = {
        "identity_revision": "fixture-v1",
        "origin_revision": "fixture-origin",
        "publisher_material": "Steel",
        "publisher_name": "Fixture_Fork",
        "publisher_object_id": "1",
    }
    group_id = s0c.sha256_bytes(s0c.canonical_json(basis))
    inventory_id = digest("inventory-object")
    identity_map = {
        "alias_edges": [],
        "authority": {},
        "discrepancies": [],
        "groups": [
            {
                "candidate_routes": [
                    {
                        "candidate_state": "metadata_candidate",
                        "inventory_object_id": inventory_id,
                        "source_id": "objectfolder_real_current",
                        "source_tier": "T2_sparse_real_contact",
                    }
                ],
                "identity_basis": basis,
                "material_family": "Metal",
                "members": [
                    {
                        "alias_basis": "identity_origin",
                        "evidence_sha256s": [digest("member-evidence")],
                        "publisher_material": "Steel",
                        "publisher_name": "Fixture_Fork",
                        "publisher_object_id": "1",
                        "revision_id": "fixture-origin",
                        "source_id": "objectfolder_real_current",
                        "source_tier": "T2_sparse_real_contact",
                    }
                ],
                "numeric_object_ids": ["1"],
                "physical_group_id": group_id,
            }
        ],
        "input_identities": {},
        "schema": s0c.IDENTITY_SCHEMA,
    }
    exposure_census = {
        "access": {},
        "authority": {},
        "candidate_counts": {},
        "decision": "fixture",
        "excluded_path_policy": {},
        "groups": [
            {
                "candidate_route_count": 1,
                "candidate_source_ids": ["objectfolder_real_current"],
                "exposure_evidence": {
                    "direct_numeric_record_count": 0,
                    "path_token_record_count": 0,
                    "realimpact_name_record_count": 0,
                },
                "exposure_state": "unexposed",
                "material_family": "Metal",
                "physical_group_id": group_id,
            }
        ],
        "historical_json_manifest": {},
        "input_identities": {},
        "metal_scope_remains_potential": True,
        "realimpact_name_evidence": {},
        "schema": s0c.EXPOSURE_SCHEMA,
    }
    inventory = {
        "access": {},
        "alias_edges": [],
        "archives": [],
        "authority": {},
        "discrepancies": [],
        "input_identities": {},
        "objects": [
            {
                "alias_status": "proven_with_historical_revision",
                "candidate_state": "metadata_candidate",
                "exposure": {"state": "unexposed"},
                "inventory_object_id": inventory_id,
                "material_family": "Metal",
                "member_root_sha256": digest("member-root"),
                "publisher_material": "Steel",
                "publisher_name": "Fixture_Fork",
                "publisher_object_id": "1",
                "reason_codes": ["compact_structure_present"],
                "source_id": "objectfolder_real_current",
                "source_tier": "T2_sparse_real_contact",
                "structural_presence": {},
            }
        ],
        "publisher_claims": {},
        "schema": s0c.INVENTORY_SCHEMA,
        "sources": [],
    }
    axes = {
        "canonical_excitation": "partial",
        "contact_geometry_binding": "unknown",
        "geometry": "partial",
        "geometry_scale": "unknown",
        "listener_condition": "partial",
        "material_identity": "known",
        "object_identity": "known",
        "recorded_response": "partial",
        "support_condition": "partial",
    }
    ycb_capability = {
        "acquisition_facts": {},
        "objects": [
            {
                "capability_axes": axes,
                "evaluation_complete": False,
                "exposure_state": "not_in_adapter_freshness_unassessed",
                "geometry": {
                    "exact_payload_identity": False,
                    "metric_scale_known": False,
                    "route_count": 1,
                    "state": "route_available",
                    "variants": [{"publisher_name": "001_fixture", "routes": [], "variant": None}],
                },
                "material_family": "Metal",
                "missing_evaluation_axes": sorted(
                    axis for axis, state in axes.items() if state != "known"
                ),
                "missing_training_axes": sorted(
                    axis
                    for axis, state in axes.items()
                    if axis != "support_condition" and state != "known"
                ),
                "object_id": "1",
                "object_name": "Fixture fork",
                "primary_material": "Steel",
                "recording_parents": [],
                "secondary_material": "None",
                "training_usable": False,
            }
        ],
        "project": {},
        "schema": s0c.YCB_SCHEMA,
    }
    return {
        "exposure_census": exposure_census,
        "identity_map": identity_map,
        "source_inventory": inventory,
        "ycb_capability": ycb_capability,
    }


def write_inputs(
    root: Path, documents: dict[str, dict[str, object]]
) -> tuple[dict[str, Path], dict[str, str]]:
    root.mkdir(parents=True)
    paths: dict[str, Path] = {}
    hashes: dict[str, str] = {}
    for name, value in documents.items():
        path = root / f"{name}.json"
        data = s0c.canonical_json(value)
        path.write_bytes(data)
        paths[name] = path
        hashes[name] = s0c.sha256_bytes(data)
    return paths, hashes


def run_build(
    root: Path,
    documents: dict[str, dict[str, object]],
    output_name: str = "output",
) -> Path:
    paths, hashes = write_inputs(root / "inputs", documents)
    return s0c.build(
        paths["identity_map"],
        paths["exposure_census"],
        paths["source_inventory"],
        paths["ycb_capability"],
        hashes,
        root / output_name,
    )


def file_map(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def role_candidate(
    index: int, *, evaluation: bool, exposure: str = "unexposed"
) -> dict[str, object]:
    return {
        "candidate_id": f"candidate-{index:02d}",
        "evaluation_complete": evaluation,
        "exposure_state": exposure,
        "physical_group_id": digest(f"group-{index:02d}"),
        "recording_parent_sha256s": [digest(f"parent-{index:02d}")],
        "source_id": "fixture-t2",
        "source_tier": "T2_sparse_real_contact",
        "training_usable": True,
    }


class SourceSufficiencyV1Tests(unittest.TestCase):
    def test_repeat_exact_fixture_is_source_insufficient_and_zero_signal(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            first = run_build(root / "a", minimal_documents())
            second = run_build(root / "b", minimal_documents())
            first_files = file_map(first)
            second_files = file_map(second)
            report = json.loads(first_files["report.json"])
            decision = json.loads(first_files["role-freeze-decision.json"])
        self.assertEqual(first_files, second_files)
        self.assertEqual(
            report["decision"], "S0C_SOURCE_INSUFFICIENT_SOURCE_GROWTH_REQUIRED"
        )
        self.assertEqual(report["counts"]["role_assignment_count"], 0)
        self.assertTrue(all(value == 0 for value in report["access"].values()))
        self.assertTrue(
            all(not row["assignments"] for row in decision["material_decisions"])
        )

    def test_hash_drift_and_noncanonical_json_fail_before_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            paths, hashes = write_inputs(root / "inputs", minimal_documents())
            paths["identity_map"].write_bytes(paths["identity_map"].read_bytes() + b" ")
            with self.assertRaisesRegex(s0c.SourceSufficiencyError, "identity changed"):
                s0c.build(
                    paths["identity_map"], paths["exposure_census"],
                    paths["source_inventory"], paths["ycb_capability"], hashes,
                    root / "drift-output",
                )
            hashes["identity_map"] = s0c.sha256_bytes(paths["identity_map"].read_bytes())
            with self.assertRaisesRegex(s0c.SourceSufficiencyError, "not canonical JSON"):
                s0c.build(
                    paths["identity_map"], paths["exposure_census"],
                    paths["source_inventory"], paths["ycb_capability"], hashes,
                    root / "canonical-output",
                )

    def test_unknown_schema_field_material_exposure_and_candidate_state_fail(self) -> None:
        mutations = []

        def schema(documents: dict[str, dict[str, object]]) -> None:
            documents["identity_map"]["schema"] = "unknown.v9"

        def field(documents: dict[str, dict[str, object]]) -> None:
            documents["exposure_census"]["unexpected"] = True

        def material(documents: dict[str, dict[str, object]]) -> None:
            documents["identity_map"]["groups"][0]["material_family"] = "Ceramic"

        def exposure(documents: dict[str, dict[str, object]]) -> None:
            documents["exposure_census"]["groups"][0]["exposure_state"] = "maybe"

        def candidate(documents: dict[str, dict[str, object]]) -> None:
            documents["identity_map"]["groups"][0]["candidate_routes"][0][
                "candidate_state"
            ] = "ready_by_sound"

        mutations.extend(
            [
                (schema, "unknown identity-map schema"),
                (field, "exposure census keys changed"),
                (material, "unknown material family"),
                (exposure, "unknown exposure state"),
                (candidate, "unknown candidate state"),
            ]
        )
        for mutation, message in mutations:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                documents = minimal_documents()
                mutation(documents)
                with self.assertRaisesRegex(s0c.SourceSufficiencyError, message):
                    run_build(Path(temporary), documents)

    def test_dangling_route_and_cross_document_material_mismatch_fail(self) -> None:
        def dangling(documents: dict[str, dict[str, object]]) -> None:
            documents["identity_map"]["groups"][0]["candidate_routes"][0][
                "inventory_object_id"
            ] = digest("missing")

        def mismatch(documents: dict[str, dict[str, object]]) -> None:
            documents["source_inventory"]["objects"][0]["material_family"] = "Wood"

        for mutation, message in (
            (dangling, "candidate route is dangling"),
            (mismatch, "candidate route material mismatch"),
        ):
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                documents = minimal_documents()
                mutation(documents)
                with self.assertRaisesRegex(s0c.SourceSufficiencyError, message):
                    run_build(Path(temporary), documents)

    def test_ycb_false_fresh_and_axis_promotions_fail_closed(self) -> None:
        def false_fresh(documents: dict[str, dict[str, object]]) -> None:
            documents["ycb_capability"]["objects"][0]["exposure_state"] = "unexposed"

        def missing_axis_training(documents: dict[str, dict[str, object]]) -> None:
            documents["ycb_capability"]["objects"][0]["training_usable"] = True

        def omitted_contact_position(documents: dict[str, dict[str, object]]) -> None:
            item = documents["ycb_capability"]["objects"][0]
            item["capability_axes"] = {axis: "known" for axis in s0c.YCB_AXES}
            item["missing_training_axes"] = []
            item["missing_evaluation_axes"] = []
            item["geometry"]["exact_payload_identity"] = True
            item["geometry"]["metric_scale_known"] = True
            item["training_usable"] = True

        def missing_support_evaluation(documents: dict[str, dict[str, object]]) -> None:
            item = documents["ycb_capability"]["objects"][0]
            item["evaluation_complete"] = True

        for mutation, message in (
            (false_fresh, "unknown YCB exposure state"),
            (missing_axis_training, "contradicts required axes"),
            (omitted_contact_position, "cannot omit contact_position"),
            (missing_support_evaluation, "contradicts required axes"),
        ):
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                documents = minimal_documents()
                mutation(documents)
                with self.assertRaisesRegex(s0c.SourceSufficiencyError, message):
                    run_build(Path(temporary), documents)

    def test_complete_role_selection_is_exact_and_reserves_protected_rows(self) -> None:
        candidates = sorted(
            [
                role_candidate(index, evaluation=index < 3)
                for index in range(8)
            ],
            key=lambda row: row["physical_group_id"],
        )
        decision = s0c.select_roles("Metal", candidates)
        self.assertEqual(decision["state"], "ROLE_FREEZE_COMPLETE")
        self.assertEqual(len(decision["assignments"]), 8)
        self.assertEqual(
            dict(sorted(Counter(row["role"] for row in decision["assignments"]).items())),
            s0c.ROLE_MINIMUM,
        )

        overlapping = sorted(
            [role_candidate(index, evaluation=index < 4) for index in range(9)],
            key=lambda row: row["physical_group_id"],
        )
        evaluation_rows = [row for row in overlapping if row["evaluation_complete"]]
        evaluation_rows[1]["recording_parent_sha256s"] = list(
            evaluation_rows[0]["recording_parent_sha256s"]
        )
        alternative = s0c.select_roles("Metal", overlapping)
        self.assertEqual(alternative["state"], "ROLE_FREEZE_COMPLETE")
        self.assertEqual(len(alternative["assignments"]), 8)

    def test_exposed_realimpact_cannot_be_recycled_and_role_mutations_fail(self) -> None:
        candidates = sorted(
            [
                role_candidate(
                    index,
                    evaluation=index < 3,
                    exposure=("protected_or_unknown_exposed" if index < 3 else "unexposed"),
                )
                for index in range(8)
            ],
            key=lambda row: row["physical_group_id"],
        )
        rejected = s0c.select_roles("Metal", candidates)
        self.assertEqual(rejected["state"], "SOURCE_INSUFFICIENT")
        self.assertEqual(rejected["assignments"], [])

        complete_candidates = sorted(
            [role_candidate(index, evaluation=index < 3) for index in range(8)],
            key=lambda row: row["physical_group_id"],
        )
        complete = s0c.select_roles("Metal", complete_candidates)

        incomplete = copy.deepcopy(complete)
        incomplete["assignments"].pop()
        with self.assertRaisesRegex(s0c.SourceSufficiencyError, "role minimum changed"):
            s0c.validate_material_decision(incomplete)

        partial = copy.deepcopy(complete)
        partial["state"] = "SOURCE_INSUFFICIENT"
        with self.assertRaisesRegex(s0c.SourceSufficiencyError, "partial role assignment"):
            s0c.validate_material_decision(partial)

        reused = copy.deepcopy(complete)
        reused["assignments"][1]["recording_parent_sha256s"] = list(
            reused["assignments"][0]["recording_parent_sha256s"]
        )
        with self.assertRaisesRegex(s0c.SourceSufficiencyError, "recording parent crosses roles"):
            s0c.validate_material_decision(reused)

    def test_output_replacement_and_symlink_input_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = run_build(root, minimal_documents())
            paths, hashes = write_inputs(root / "second-inputs", minimal_documents())
            with self.assertRaisesRegex(s0c.SourceSufficiencyError, "already exists"):
                s0c.build(
                    paths["identity_map"], paths["exposure_census"],
                    paths["source_inventory"], paths["ycb_capability"], hashes, output,
                )

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            paths, hashes = write_inputs(root / "inputs", minimal_documents())
            link = root / "identity-link.json"
            link.symlink_to(paths["identity_map"])
            with self.assertRaisesRegex(s0c.SourceSufficiencyError, "regular file"):
                s0c.build(
                    link, paths["exposure_census"], paths["source_inventory"],
                    paths["ycb_capability"], hashes, root / "output",
                )

    def test_repository_output_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            paths, hashes = write_inputs(root / "inputs", minimal_documents())
            with self.assertRaisesRegex(s0c.SourceSufficiencyError, "outside the repository"):
                s0c.build(
                    paths["identity_map"], paths["exposure_census"],
                    paths["source_inventory"], paths["ycb_capability"], hashes,
                    s0c.repository_root() / "s0c-forbidden-output",
                )


if __name__ == "__main__":
    unittest.main()
