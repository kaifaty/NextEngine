#!/usr/bin/env python3
"""Focused guards for Physical Sound V14-N1a Dataset Contract V1."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_dataset_contract_v1 as contract_v1  # noqa: E402


def file_map(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def load(path: Path) -> dict[str, object]:
    return json.loads(path.read_bytes())


def write(path: Path, value: object) -> None:
    path.write_bytes(contract_v1.canonical_json(value))


def sorted_objects(contract: dict[str, object]) -> None:
    contract["objects"].sort(key=lambda item: item["object_group_sha256"])


def build_mutated(
    root: Path,
    mutate: object,
) -> Path:
    contract_path, ledger_path = contract_v1.write_synthetic_inputs(root / "inputs")
    contract = load(contract_path)
    ledger = load(ledger_path)
    mutate(contract, ledger)
    write(contract_path, contract)
    write(ledger_path, ledger)
    return contract_v1.build_contract(
        contract_path, ledger_path, root / "output"
    )


def object_with_role(contract: dict[str, object], role: str) -> dict[str, object]:
    return next(item for item in contract["objects"] if item["role"] == role)


class DatasetContractV1Tests(unittest.TestCase):
    def test_repeat_fixture_is_exact_with_zero_source_and_signal_access(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            first = contract_v1.build_fixture(root / "run-a")
            second = contract_v1.build_fixture(root / "run-b")
            first_files = file_map(first)
            second_files = file_map(second)
            report = json.loads(first_files["report.json"])
            contract = json.loads(first_files["contract.json"])
        self.assertEqual(first_files, second_files)
        self.assertEqual(
            report["decision"], "N1A_DATASET_CONTRACT_V1_FIXTURE_PASS"
        )
        self.assertEqual(report["counts"]["source_count"], 3)
        self.assertEqual(report["counts"]["object_row_count"], 24)
        self.assertEqual(report["counts"]["physical_object_group_count"], 24)
        self.assertEqual(report["counts"]["recording_parent_count"], 24)
        self.assertEqual(report["counts"]["source_revision_cluster_count"], 3)
        for material in contract_v1.MATERIALS:
            self.assertEqual(
                report["counts"]["eligible_role_counts"][material],
                contract_v1.MINIMUM_PER_MATERIAL,
            )
        for key in (
            "network_requests",
            "source_artifact_bytes_read",
            "pcm_sample_values_decoded",
            "force_sample_values_decoded",
            "protected_signal_values_decoded",
        ):
            self.assertEqual(report["build_access"][key], 0)
        training_usable = [
            item
            for item in contract["objects"]
            if item["quality_mask"]["classification"] == "training_usable"
        ]
        self.assertEqual(len(training_usable), 3)
        self.assertTrue(
            all(item["role"] == "generator_train" for item in training_usable)
        )
        self.assertTrue(
            all(item["axes"][-1]["state"] == "absent" for item in training_usable)
        )

    def test_parent_ledger_hash_role_root_and_schema_are_bound(self) -> None:
        mutations = []

        def wrong_hash(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["parent_exposure_ledger"]["sha256"] = contract_v1.synthetic_hash(
                "wrong-ledger"
            )

        def wrong_role_root(
            _contract: dict[str, object], ledger: dict[str, object]
        ) -> None:
            ledger["role_root_sha256"] = contract_v1.synthetic_hash("wrong-root")

        def wrong_schema(
            _contract: dict[str, object], ledger: dict[str, object]
        ) -> None:
            ledger["schema"] = "nextengine.unknown-ledger.v9"

        def inconsistent_roles(
            contract: dict[str, object], ledger: dict[str, object]
        ) -> None:
            ledger["entries"][0]["roles"] = ["historical_unknown"]
            ledger["role_root_sha256"] = contract_v1.ledger_v0.role_root(
                ledger["entries"]
            )
            ledger_bytes = contract_v1.canonical_json(ledger)
            contract["parent_exposure_ledger"]["sha256"] = contract_v1.sha256_bytes(
                ledger_bytes
            )
            contract["parent_exposure_ledger"]["role_root_sha256"] = ledger[
                "role_root_sha256"
            ]

        mutations.extend(
            [
                (wrong_hash, "parent ledger hash changed"),
                (wrong_role_root, "ledger role root changed"),
                (wrong_schema, "unknown exposure-ledger schema"),
                (inconsistent_roles, "ledger entry roles do not match exposures"),
            ]
        )
        for mutation, message in mutations:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(contract_v1.DatasetContractError, message):
                    build_mutated(Path(temporary), mutation)

    def test_unknown_schema_field_role_tier_material_quality_and_axis_fail(self) -> None:
        def unknown_schema(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["schema"] = "nextengine.unknown.v9"

        def unknown_field(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["unexpected"] = True

        def unknown_role(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["objects"][0]["role"] = "training"

        def unknown_tier(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["sources"][0]["tier"] = "T9_magic"

        def unknown_material(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["objects"][0]["material_family"] = "Ceramic"

        def unknown_quality(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["objects"][0]["quality_mask"]["classification"] = "maybe"

        def unknown_axis(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["objects"][0]["axes"][0]["axis"] = "magic_axis"

        cases = (
            (unknown_schema, "unknown dataset-contract schema"),
            (unknown_field, "contract keys changed"),
            (unknown_role, "unknown dataset role"),
            (unknown_tier, "unknown source tier"),
            (unknown_material, "unknown material family"),
            (unknown_quality, "unknown quality class"),
            (unknown_axis, "axes are missing, unknown or not sorted"),
        )
        for mutation, message in cases:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(contract_v1.DatasetContractError, message):
                    build_mutated(Path(temporary), mutation)

    def test_derived_source_object_and_prior_group_hashes_fail_closed(self) -> None:
        def source_hash(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["sources"][0]["source_revision_group_sha256"] = (
                contract_v1.synthetic_hash("wrong-source-group")
            )

        def object_hash(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["objects"][0]["object_group_sha256"] = contract_v1.synthetic_hash(
                "wrong-object-group"
            )
            sorted_objects(contract)

        def prior_hash(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["objects"][0]["prior_exposure"]["object_group_sha256"] = (
                contract_v1.synthetic_hash("wrong-prior-group")
            )

        def alias_query_set(
            contract: dict[str, object], _ledger: dict[str, object]
        ) -> None:
            contract["objects"][0]["prior_exposure"][
                "queried_object_group_sha256s"
            ] = [contract_v1.synthetic_hash("wrong-alias-query")]

        cases = (
            (source_hash, "source revision group hash changed"),
            (object_hash, "object group hash changed"),
            (prior_hash, "prior exposure object group hash changed"),
            (alias_query_set, "prior exposure alias query set changed"),
        )
        for mutation, message in cases:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(contract_v1.DatasetContractError, message):
                    build_mutated(Path(temporary), mutation)

    def test_duplicate_object_and_cross_role_physical_or_recording_groups_fail(self) -> None:
        def duplicate_object(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["objects"].append(copy.deepcopy(contract["objects"][0]))
            sorted_objects(contract)

        def physical_leak(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            first = object_with_role(contract, "generator_train")
            protected = object_with_role(contract, "admission_shadow")
            protected["physical_object_group_id"] = first["physical_object_group_id"]

        def recording_leak(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            first = object_with_role(contract, "generator_train")
            protected = object_with_role(contract, "validator_method_holdout")
            protected["recording_parent_sha256s"] = list(
                first["recording_parent_sha256s"]
            )

        cases = (
            (duplicate_object, "objects are duplicate or not sorted"),
            (physical_leak, "physical object group crosses dataset roles"),
            (recording_leak, "recording parent crosses dataset roles"),
        )
        for mutation, message in cases:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(contract_v1.DatasetContractError, message):
                    build_mutated(Path(temporary), mutation)

    def test_protected_quality_axes_tier_and_recording_parent_fail_closed(self) -> None:
        def training_quality(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            item = object_with_role(contract, "admission_shadow")
            item["quality_mask"]["classification"] = "training_usable"

        def absent_axis(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            item = object_with_role(contract, "validator_calibration")
            item["axes"][-1] = {
                "axis": "support_condition",
                "state": "absent",
                "evidence_sha256": None,
            }

        def synthetic_tier(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            source_id = object_with_role(contract, "admission_shadow")["source_id"]
            next(item for item in contract["sources"] if item["source_id"] == source_id)[
                "tier"
            ] = "T1_synthetic_teacher"

        def no_recording_parent(
            contract: dict[str, object], _ledger: dict[str, object]
        ) -> None:
            object_with_role(contract, "validator_method_holdout")[
                "recording_parent_sha256s"
            ] = []

        cases = (
            (training_quality, "training-usable object cannot receive evaluation role"),
            (absent_axis, "quality class has absent required axes"),
            (synthetic_tier, "protected role requires a real-contact tier"),
            (no_recording_parent, "active role requires a recording parent"),
        )
        for mutation, message in cases:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(contract_v1.DatasetContractError, message):
                    build_mutated(Path(temporary), mutation)

    def test_prior_exposure_cannot_be_promoted_to_protected_or_recycled(self) -> None:
        def generator_to_shadow(
            contract: dict[str, object], _ledger: dict[str, object]
        ) -> None:
            item = next(
                item
                for item in contract["objects"]
                if item["publisher_object_id"] == "glass-00"
            )
            item["role"] = "admission_shadow"

        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(
                contract_v1.DatasetContractError,
                "prior exposure cannot enter a protected role",
            ):
                build_mutated(Path(temporary), generator_to_shadow)

        def exposed_alias_to_shadow(
            contract: dict[str, object], _ledger: dict[str, object]
        ) -> None:
            exposed = next(
                item
                for item in contract["objects"]
                if item["publisher_object_id"] == "glass-00"
            )
            protected = next(
                item
                for item in contract["objects"]
                if item["material_family"] == "Glass"
                and item["role"] == "validator_method_holdout"
            )
            aliases = sorted(
                {
                    protected["object_group_sha256"],
                    exposed["object_group_sha256"],
                }
            )
            protected["known_alias_object_group_sha256s"] = aliases
            protected["prior_exposure"].update(
                {
                    "queried_object_group_sha256s": aliases,
                    "ledger_roles": ["estimator_fit"],
                    "state": "generator_exposed",
                }
            )

        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(
                contract_v1.DatasetContractError,
                "prior exposure cannot enter a protected role",
            ):
                build_mutated(Path(temporary), exposed_alias_to_shadow)

        def historical_unknown(
            contract: dict[str, object], ledger: dict[str, object]
        ) -> None:
            entry = ledger["entries"][0]
            entry["roles"] = ["historical_unknown"]
            entry["exposures"][0]["role"] = "historical_unknown"
            ledger["role_root_sha256"] = contract_v1.ledger_v0.role_root(
                ledger["entries"]
            )
            ledger_bytes = contract_v1.canonical_json(ledger)
            contract["parent_exposure_ledger"]["sha256"] = contract_v1.sha256_bytes(
                ledger_bytes
            )
            contract["parent_exposure_ledger"]["role_root_sha256"] = ledger[
                "role_root_sha256"
            ]
            item = next(
                item
                for item in contract["objects"]
                if item["publisher_object_id"] == "glass-00"
            )
            item["prior_exposure"]["ledger_roles"] = ["historical_unknown"]
            item["prior_exposure"]["state"] = "protected_or_unknown_exposed"

        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(
                contract_v1.DatasetContractError,
                "protected or unknown exposure cannot enter generator data",
            ):
                build_mutated(Path(temporary), historical_unknown)

    def test_quality_ood_failed_evidence_and_signal_accounting_fail_closed(self) -> None:
        def active_ood(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            item = object_with_role(contract, "generator_train")
            item["quality_mask"] = {
                "classification": "source_ood",
                "reason_code": "artifact_incomplete",
                "evidence": contract_v1.quality_evidence("active-ood", failed=True),
            }

        def active_failed_evidence(
            contract: dict[str, object], _ledger: dict[str, object]
        ) -> None:
            item = object_with_role(contract, "generator_train")
            item["quality_mask"]["evidence"][0]["decision"] = "Fail"

        def decoded_quality_signal(
            contract: dict[str, object], _ledger: dict[str, object]
        ) -> None:
            contract["objects"][0]["quality_mask"]["evidence"][0][
                "signal_values_decoded"
            ] = 1

        cases = (
            (active_ood, "source OOD cannot be active data"),
            (active_failed_evidence, "active quality mask contains failed evidence"),
            (decoded_quality_signal, "quality evidence decoded signal values"),
        )
        for mutation, message in cases:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(contract_v1.DatasetContractError, message):
                    build_mutated(Path(temporary), mutation)

    def test_source_ood_exclusion_is_valid_and_does_not_add_role_credit(self) -> None:
        def mutation(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            source = next(
                item
                for item in contract["sources"]
                if item["source_id"] == "synthetic-glass-source"
            )
            publisher_object_id = "glass-source-ood"
            group = contract_v1.object_parent_hash(source, publisher_object_id)
            item = copy.deepcopy(contract["objects"][0])
            item.update(
                {
                    "object_group_sha256": group,
                    "known_alias_object_group_sha256s": [group],
                    "physical_object_group_id": "synthetic:glass:source-ood",
                    "source_id": source["source_id"],
                    "publisher_object_id": publisher_object_id,
                    "material_family": "Glass",
                    "role": "excluded",
                    "quality_mask": {
                        "classification": "source_ood",
                        "reason_code": "artifact_incomplete",
                        "evidence": contract_v1.quality_evidence(
                            "glass-source-ood", failed=True
                        ),
                    },
                    "recording_parent_sha256s": [],
                    "source_member_root_sha256": contract_v1.synthetic_hash(
                        "members:glass-source-ood"
                    ),
                    "prior_exposure": {
                        "object_group_sha256": group,
                        "queried_object_group_sha256s": [group],
                        "ledger_roles": [],
                        "state": "unexposed",
                    },
                }
            )
            item["axes"][-1] = {
                "axis": "support_condition",
                "state": "absent",
                "evidence_sha256": None,
            }
            contract["objects"].append(item)
            sorted_objects(contract)

        with tempfile.TemporaryDirectory() as temporary:
            output = build_mutated(Path(temporary), mutation)
            report = load(output / "report.json")
        self.assertEqual(report["counts"]["object_row_count"], 25)
        self.assertEqual(
            report["counts"]["eligible_role_counts"]["Glass"],
            contract_v1.MINIMUM_PER_MATERIAL,
        )

    def test_minimum_material_role_coverage_fails_closed(self) -> None:
        def mutation(contract: dict[str, object], _ledger: dict[str, object]) -> None:
            contract["objects"] = [
                item
                for item in contract["objects"]
                if not (
                    item["material_family"] == "Metal"
                    and item["role"] == "admission_shadow"
                )
            ]

        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(
                contract_v1.DatasetContractError,
                "minimum role coverage not met: Metal/admission_shadow",
            ):
                build_mutated(Path(temporary), mutation)

    def test_noncanonical_duplicate_key_and_oversized_inputs_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            contract_path, ledger_path = contract_v1.write_synthetic_inputs(
                root / "inputs"
            )
            value = load(contract_path)
            contract_path.write_bytes(
                json.dumps(value, separators=(",", ":"), sort_keys=True).encode()
            )
            with self.assertRaisesRegex(
                contract_v1.DatasetContractError, "not canonical JSON"
            ):
                contract_v1.build_contract(contract_path, ledger_path, root / "output")

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            contract_path, ledger_path = contract_v1.write_synthetic_inputs(
                root / "inputs"
            )
            contract_path.write_bytes(b'{"schema":"one","schema":"two"}\n')
            with self.assertRaisesRegex(contract_v1.DatasetContractError, "duplicate"):
                contract_v1.build_contract(contract_path, ledger_path, root / "output")

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            contract_path, _ledger_path = contract_v1.write_synthetic_inputs(
                root / "inputs"
            )
            with mock.patch.object(contract_v1, "MAX_INPUT_BYTES", 8):
                with self.assertRaisesRegex(
                    contract_v1.DatasetContractError, "byte limit"
                ):
                    contract_v1.parse_canonical_document(contract_path, "contract")

    def test_output_inside_repository_is_rejected(self) -> None:
        output = contract_v1.repository_root() / "forbidden-dataset-contract-output"
        with self.assertRaisesRegex(contract_v1.DatasetContractError, "outside"):
            contract_v1.prepare_output(output)


if __name__ == "__main__":
    unittest.main()
