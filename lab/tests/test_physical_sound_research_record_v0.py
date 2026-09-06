#!/usr/bin/env python3
"""Focused guards for the Physical Sound Research Record V0 protocol."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_research_record_v0 as record_v0  # noqa: E402


def file_map(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class ResearchRecordV0Tests(unittest.TestCase):
    def test_schema_descriptor_is_current_only_and_non_authoritative(self) -> None:
        descriptor = record_v0.schema_descriptor()
        self.assertEqual(descriptor["record_schema"], record_v0.RECORD_SCHEMA)
        self.assertTrue(descriptor["compatibility"]["current_version_roundtrip"])
        self.assertFalse(descriptor["compatibility"]["migration_supported"])
        self.assertFalse(descriptor["public_contract"])
        self.assertFalse(descriptor["runtime_consumer_allowed"])
        self.assertEqual(
            set(descriptor["terminal_states"]),
            {"FallbackOnly", "FallbackOutOfDomain"},
        )

    def test_source_record_has_canonical_byte_exact_roundtrip(self) -> None:
        record = record_v0.source_qualified_record()
        encoded = record_v0.canonical_json(record)
        decoded = record_v0.decode_canonical_json(encoded)
        self.assertEqual(record_v0.validate_record(decoded), record)
        self.assertEqual(record_v0.canonical_json(decoded), encoded)
        self.assertEqual(record["exposure_ledger"]["signal_values_decoded"], 0)
        self.assertTrue(record["fallback"]["required"])

    def test_each_claim_requires_its_frozen_axes(self) -> None:
        canonical = record_v0.source_qualified_record("canonical_impact_field")
        measured = record_v0.source_qualified_record("measured_transfer_field")
        canonical_axes = {item["axis"]: item for item in canonical["axes"]}
        measured_axes = {item["axis"]: item for item in measured["axes"]}
        self.assertEqual(canonical_axes["canonical_excitation"]["state"], "known")
        self.assertEqual(measured_axes["raw_force"]["state"], "known")
        self.assertEqual(measured_axes["common_timebase"]["state"], "known")
        self.assertEqual(
            measured_axes["force_frequency_coverage"]["state"], "known"
        )

        invalid = copy.deepcopy(measured)
        raw_force = next(item for item in invalid["axes"] if item["axis"] == "raw_force")
        raw_force["state"] = "absent"
        raw_force["evidence_sha256"] = None
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "requires known axis"):
            record_v0.validate_record(invalid)

        unknown = copy.deepcopy(canonical)
        unknown["axes"][-1]["axis"] = "unknown_axis"
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "unknown axis"):
            record_v0.validate_record(unknown)

    def test_unknown_fields_hashes_and_missing_fallback_fail_closed(self) -> None:
        cases = []
        unknown = record_v0.source_qualified_record()
        unknown["unexpected"] = True
        cases.append((unknown, "keys changed"))

        malformed = record_v0.source_qualified_record()
        malformed["source"]["provenance_sha256"] = "BAD"
        cases.append((malformed, "lowercase SHA-256"))

        no_fallback = record_v0.source_qualified_record()
        no_fallback["fallback"]["required"] = False
        cases.append((no_fallback, "fallback must remain required"))

        bool_counter = record_v0.source_qualified_record()
        bool_counter["exposure_ledger"]["sample_identity_count"] = True
        cases.append((bool_counter, "non-negative integer"))

        for value, message in cases:
            with self.subTest(message=message):
                with self.assertRaisesRegex(record_v0.ResearchRecordError, message):
                    record_v0.validate_record(value)

    def test_lifecycle_skips_and_parent_drift_fail_closed(self) -> None:
        source = record_v0.source_qualified_record()
        formula = record_v0.formula_validated_record(source)

        skipped = copy.deepcopy(formula)
        skipped["lifecycle"]["state"] = "AtlasAdmitted"
        skipped["lifecycle"]["transition"] = "atlas_admitted"
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "not allowed"):
            record_v0.validate_record(skipped)

        wrong_hash = copy.deepcopy(formula)
        wrong_hash["previous_record_sha256"] = record_v0.synthetic_hash("wrong")
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "hash does not match"):
            record_v0.validate_transition(source, wrong_hash)

        wrong_revision = copy.deepcopy(formula)
        wrong_revision["revision"] = 9
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "revision"):
            record_v0.validate_transition(source, wrong_revision)

    def test_terminal_record_cannot_be_reopened(self) -> None:
        source = record_v0.source_qualified_record()
        terminal = record_v0.fallback_only_record(source)
        otherwise_valid_successor = record_v0.formula_validated_record(source)
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "cannot be reopened"):
            record_v0.validate_transition(terminal, otherwise_valid_successor)

    def test_atlas_requires_independent_validator_and_cooker(self) -> None:
        source = record_v0.source_qualified_record()
        formula = record_v0.formula_validated_record(source)
        atlas = record_v0.atlas_admitted_record(formula)

        no_cooker = copy.deepcopy(atlas)
        no_cooker["evidence"]["cooker_manifest_sha256"] = None
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "cooker_manifest"):
            record_v0.validate_record(no_cooker)

        learned_vote_only = copy.deepcopy(atlas)
        learned_vote_only["validator"]["independent"] = False
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "must be independent"):
            record_v0.validate_record(learned_vote_only)

    def test_successor_cannot_change_source_or_claim(self) -> None:
        source = record_v0.source_qualified_record()
        changed_source = record_v0.formula_validated_record(source)
        changed_source["source"]["object_id"] = "different-object"
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "invariant source"):
            record_v0.validate_transition(source, changed_source)

        changed_claim = record_v0.formula_validated_record(source)
        changed_claim["claim_kind"] = "measured_transfer_field"
        changed_claim["axes"] = record_v0.synthetic_axes("measured_transfer_field")
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "invariant claim_kind"):
            record_v0.validate_transition(source, changed_claim)

    def test_duplicate_and_noncanonical_json_are_rejected(self) -> None:
        duplicate = b'{"schema": "one", "schema": "two"}\n'
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "duplicate JSON key"):
            record_v0.decode_canonical_json(duplicate)

        record = record_v0.source_qualified_record()
        noncanonical = json.dumps(record, separators=(",", ":")).encode()
        with self.assertRaisesRegex(record_v0.ResearchRecordError, "not canonical"):
            record_v0.decode_canonical_json(noncanonical)

    def test_repeated_fixture_builds_are_byte_identical_and_zero_read(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            first = record_v0.build_fixture(root / "fixture-a")
            second = record_v0.build_fixture(root / "fixture-b")
            first_files = file_map(first)
            second_files = file_map(second)
            report = json.loads(first_files["report.json"])
        self.assertEqual(first_files, second_files)
        self.assertEqual(
            report["decision"], "M1A_RESEARCH_RECORD_V0_FIXTURE_PASS"
        )
        self.assertEqual(report["counters"]["network_requests"], 0)
        self.assertEqual(report["counters"]["source_bytes_read"], 0)
        self.assertEqual(report["counters"]["signal_values_decoded"], 0)
        self.assertEqual(report["counters"]["protected_signal_values_decoded"], 0)
        self.assertFalse(report["public_contract"])
        self.assertFalse(report["runtime_consumer_allowed"])

    def test_validate_stage_preserves_current_record_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture = record_v0.build_fixture(root / "fixture")
            output = record_v0.validate_file(
                fixture / "formula-validated.json",
                fixture / "source-qualified.json",
                root / "validated",
            )
            report = json.loads((output / "report.json").read_bytes())
            output_record = (output / "record.json").read_bytes()
            fixture_record = (fixture / "formula-validated.json").read_bytes()
        self.assertEqual(
            output_record,
            fixture_record,
        )
        self.assertEqual(
            report["decision"], "CURRENT_V0_CANONICAL_ROUNDTRIP_PASS"
        )
        self.assertFalse(report["migration_performed"])

    def test_output_inside_repository_is_rejected(self) -> None:
        output = record_v0.repository_root() / "forbidden-research-record-output"
        with self.assertRaisesRegex(
            record_v0.ResearchRecordError, "outside the repository"
        ):
            record_v0.prepare_output(output)


if __name__ == "__main__":
    unittest.main()
