#!/usr/bin/env python3
"""Focused guards for the Physical Sound Exposure Ledger V0 builder."""

from __future__ import annotations

import copy
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_exposure_ledger_v0 as ledger_v0  # noqa: E402
import physical_sound_research_record_v0 as record_v0  # noqa: E402


def file_map(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def read_catalog(path: Path) -> dict[str, object]:
    return json.loads(path.read_bytes())


def write_catalog(path: Path, catalog: dict[str, object]) -> None:
    path.write_bytes(ledger_v0.canonical_json(catalog))


def build_mutated(
    root: Path,
    mutate: object,
    *,
    contact_leak: bool = False,
    mutation_leak: bool = False,
) -> Path:
    store, catalog_path = ledger_v0.write_synthetic_inputs(
        root,
        contact_leak=contact_leak,
        mutation_leak=mutation_leak,
    )
    catalog = read_catalog(catalog_path)
    mutate(catalog, store)
    write_catalog(catalog_path, catalog)
    return ledger_v0.build_ledger(catalog_path, store, root / "output")


class ExposureLedgerV0Tests(unittest.TestCase):
    def test_repeated_fixture_builds_are_byte_identical_and_zero_read(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            first = ledger_v0.build_fixture(root / "fixture-a")
            second = ledger_v0.build_fixture(root / "fixture-b")
            first_files = file_map(first)
            second_files = file_map(second)
            report = json.loads(first_files["report.json"])
            ledger = json.loads(first_files["ledger.json"])
            summary = json.loads(first_files["summary.json"])
        self.assertEqual(first_files, second_files)
        self.assertEqual(report["decision"], "M1B_EXPOSURE_LEDGER_V0_FIXTURE_PASS")
        self.assertEqual(report["build_access"]["network_requests"], 0)
        self.assertEqual(report["build_access"]["source_bytes_read"], 0)
        self.assertEqual(report["build_access"]["signal_values_decoded"], 0)
        self.assertEqual(
            report["build_access"]["protected_signal_values_decoded"], 0
        )
        self.assertEqual(report["build_access"]["metadata_files_parsed"], 1)
        self.assertEqual(summary["sha256"], ledger_v0.sha256_bytes(first_files["ledger.json"]))
        self.assertEqual(summary["sample_identity_count"], 4)
        self.assertEqual(summary["signal_values_decoded"], 448)
        self.assertEqual(summary["protected_signal_values_decoded"], 192)
        self.assertEqual(summary["role_root_sha256"], ledger["role_root_sha256"])
        record_v0.validate_ledger(summary)

    def test_artifact_hash_size_schema_and_path_changes_fail_closed(self) -> None:
        mutations = [
            (
                lambda catalog, _store: catalog["artifacts"][0].update(
                    {"sha256": ledger_v0.synthetic_hash("wrong")}
                ),
                "artifact hash changed",
            ),
            (
                lambda catalog, _store: catalog["artifacts"][0].update(
                    {"bytes": catalog["artifacts"][0]["bytes"] + 1}
                ),
                "byte length changed",
            ),
            (
                lambda catalog, _store: catalog["artifacts"][0].update(
                    {"schema_hint": "nextengine.changed-schema.v0"}
                ),
                "artifact schema changed",
            ),
            (
                lambda catalog, _store: catalog["artifacts"][0].update(
                    {"relative_path": "../manifest.json"}
                ),
                "escapes or is not normalized",
            ),
        ]
        for mutation, message in mutations:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, message):
                    build_mutated(Path(temporary), mutation)

    def test_artifact_symlink_is_rejected(self) -> None:
        def mutation(catalog: dict[str, object], store: Path) -> None:
            os.symlink(store / "manifest.json", store / "linked-manifest.json")
            catalog["artifacts"][0]["relative_path"] = "linked-manifest.json"

        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, "symlink"):
                build_mutated(Path(temporary), mutation)

    def test_evidence_hash_pointer_and_binding_fail_closed(self) -> None:
        def changed_hash(catalog: dict[str, object], _store: Path) -> None:
            catalog["exposures"][0]["evidence"][0]["value_sha256"] = (
                ledger_v0.synthetic_hash("wrong-value")
            )

        def unresolved_pointer(catalog: dict[str, object], _store: Path) -> None:
            catalog["exposures"][0]["evidence"][0]["json_pointer"] = "/missing"

        def duplicate_binding(catalog: dict[str, object], _store: Path) -> None:
            evidence = catalog["exposures"][0]["evidence"]
            evidence.append(copy.deepcopy(evidence[0]))

        def missing_binding(catalog: dict[str, object], _store: Path) -> None:
            exposure = catalog["exposures"][0]
            exposure["evidence"] = [
                item for item in exposure["evidence"] if item["binds"] != "project_id"
            ]

        mutations = [
            (changed_hash, "pointed value hash changed"),
            (unresolved_pointer, "unresolved JSON Pointer"),
            (duplicate_binding, "duplicate evidence binding"),
            (missing_binding, "unevidenced identity components"),
        ]
        for mutation, message in mutations:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, message):
                    build_mutated(Path(temporary), mutation)

    def test_bound_identity_and_counter_must_match_pointed_value(self) -> None:
        def identity_mismatch(catalog: dict[str, object], _store: Path) -> None:
            catalog["exposures"][0]["identity"]["object_id"] = "different-object"
            catalog["exposures"].sort(key=ledger_v0.exposure_sort_key)

        def counter_mismatch(catalog: dict[str, object], _store: Path) -> None:
            exposure = catalog["exposures"][0]
            exposure["values_decoded"] += 1
            catalog["exposures"].sort(key=ledger_v0.exposure_sort_key)

        for mutation, message in (
            (identity_mismatch, "does not match identity.object_id"),
            (counter_mismatch, "does not match values_decoded"),
        ):
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, message):
                    build_mutated(Path(temporary), mutation)

    def test_unknown_schema_fields_roles_access_and_partition_fail_closed(self) -> None:
        def unknown_schema(catalog: dict[str, object], _store: Path) -> None:
            catalog["schema"] = "nextengine.unknown.v99"

        def unknown_field(catalog: dict[str, object], _store: Path) -> None:
            catalog["unexpected"] = True

        def unknown_role(catalog: dict[str, object], _store: Path) -> None:
            catalog["exposures"][0]["role"] = "training"
            catalog["exposures"].sort(key=ledger_v0.exposure_sort_key)

        def unknown_access(catalog: dict[str, object], _store: Path) -> None:
            catalog["exposures"][0]["access_kind"] = "opened"
            catalog["exposures"].sort(key=ledger_v0.exposure_sort_key)

        def unknown_partition(catalog: dict[str, object], _store: Path) -> None:
            catalog["partition_policy"] = "best_effort"

        mutations = [
            (unknown_schema, "unknown exposure-catalog schema"),
            (unknown_field, "catalog keys changed"),
            (unknown_role, "unknown exposure role"),
            (unknown_access, "unknown access kind"),
            (unknown_partition, "unknown partition policy"),
        ]
        for mutation, message in mutations:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, message):
                    build_mutated(Path(temporary), mutation)

    def test_duplicate_exact_exposure_is_rejected(self) -> None:
        def mutation(catalog: dict[str, object], _store: Path) -> None:
            catalog["exposures"].append(copy.deepcopy(catalog["exposures"][0]))
            catalog["exposures"].sort(key=ledger_v0.exposure_sort_key)

        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, "duplicate exact"):
                build_mutated(Path(temporary), mutation)

    def test_access_accounting_and_protected_fit_fail_closed(self) -> None:
        def metadata_with_values(catalog: dict[str, object], _store: Path) -> None:
            catalog["exposures"][0]["access_kind"] = "metadata_only"
            catalog["exposures"].sort(key=ledger_v0.exposure_sort_key)

        def protected_fit(catalog: dict[str, object], _store: Path) -> None:
            fit = next(
                item
                for item in catalog["exposures"]
                if item["role"] == "estimator_fit"
            )
            fit["protected"] = True
            catalog["exposures"].sort(key=ledger_v0.exposure_sort_key)

        for mutation, message in (
            (metadata_with_values, "zero values_decoded"),
            (protected_fit, "cannot be protected"),
        ):
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, message):
                    build_mutated(Path(temporary), mutation)

    def test_contact_parent_leak_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            store, catalog = ledger_v0.write_synthetic_inputs(root, contact_leak=True)
            with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, "partition leak"):
                ledger_v0.build_ledger(catalog, store, root / "output")

    def test_mutation_parent_leak_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            store, catalog = ledger_v0.write_synthetic_inputs(root, mutation_leak=True)
            with self.assertRaisesRegex(
                ledger_v0.ExposureLedgerError, "mutation_parent"
            ):
                ledger_v0.build_ledger(catalog, store, root / "output")

    def test_historical_union_records_overlap_without_failing(self) -> None:
        def mutation(catalog: dict[str, object], _store: Path) -> None:
            catalog["partition_policy"] = "historical_union"

        with tempfile.TemporaryDirectory() as temporary:
            output = build_mutated(
                Path(temporary), mutation, contact_leak=True, mutation_leak=True
            )
            ledger = json.loads((output / "ledger.json").read_bytes())
        self.assertGreater(
            ledger["partition_observations"]["cross_side_group_count"], 0
        )

    def test_noncanonical_duplicate_and_oversized_catalog_fail_closed(self) -> None:
        documents, catalog = ledger_v0.synthetic_documents()
        self.assertTrue(documents)
        noncanonical = json.dumps(catalog, separators=(",", ":")).encode()
        with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, "not canonical"):
            ledger_v0.decode_canonical_catalog(noncanonical)

        duplicate = b'{"schema": "one", "schema": "two"}\n'
        with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, "duplicate JSON key"):
            ledger_v0.decode_canonical_catalog(duplicate)

        with mock.patch.object(ledger_v0, "MAX_CATALOG_BYTES", 8):
            with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, "byte limit"):
                ledger_v0.decode_canonical_catalog(b"123456789")

    def test_output_and_store_inside_repository_are_rejected(self) -> None:
        output = ledger_v0.repository_root() / "forbidden-exposure-ledger-output"
        with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, "outside"):
            ledger_v0.prepare_output(output)

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _store, catalog = ledger_v0.write_synthetic_inputs(root)
            with self.assertRaisesRegex(ledger_v0.ExposureLedgerError, "store root"):
                ledger_v0.build_ledger(
                    catalog, ledger_v0.repository_root(), root / "output"
                )


if __name__ == "__main__":
    unittest.main()
