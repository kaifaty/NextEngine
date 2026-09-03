from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v44_c1_runtime_descriptor_intake_v1.py"
CONTRACT = (
    LAB / "profiles" / "physical-sound-v44-c1-runtime-descriptor-contract.v1.json"
)
PROFILE = LAB / "profiles" / "physical-sound-v44-c1-runtime-descriptor-intake.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v44_c1_runtime_descriptor_intake_v1 as c1  # noqa: E402


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def binding(path: str, data: bytes) -> dict[str, object]:
    return {"bytes": len(data), "path": path, "sha256": sha256(data)}


def row(
    record_id: str,
    parent_id: str,
    family_id: str,
    component_id: str,
    material: str,
    observed: bool,
) -> dict[str, object]:
    return {
        "acoustic_target": {
            "bytes": 1,
            "path": "objects/features/unread.json",
            "sha256": "1" * 64,
        },
        "axis_mask": {
            "acoustic_pseudo_target": True,
            "material_identity": observed,
            "object_identity": True,
            "waveform": True,
        },
        "canonical_pcm": {
            "bytes": 1,
            "path": "objects/pcm/unread.wav",
            "sha256": "2" * 64,
        },
        "family_component_id": component_id,
        "family_id": family_id,
        "material_label": material,
        "physical_parent_id": parent_id,
        "record_id": record_id,
    }


def write_json(path: Path, value: dict[str, object]) -> bytes:
    data = c1.canonical_json(value)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return data


def projection(role: str, rows: list[dict[str, object]]) -> dict[str, object]:
    return {
        "record_count": len(rows),
        "role": role,
        "rows": rows,
        "rows_root_sha256": sha256(c1.canonical_json(rows)),
        "schema": c1.PROJECTION_SCHEMA,
    }


def build_fixture(root: Path) -> tuple[Path, Path, dict[str, object]]:
    corpus = root / "corpus"
    train_rows = [
        row("train-glass-1", "parent-glass", "family-a", "component-a", "Glass", True),
        row("train-glass-2", "parent-glass", "family-a", "component-a", "Glass", True),
        row(
            "train-unknown",
            "parent-quarantine",
            "family-a",
            "component-a",
            "Unknown",
            False,
        ),
    ]
    development_rows = [
        row("dev-wood-1", "parent-wood", "family-b", "component-b", "Wood", True)
    ]
    values = {
        "profile.json": {
            "schema": "nextengine.experimental-physical-sound-v43-d2-c0r-profile.v1"
        },
        "report.json": {
            "artifacts": {
                "manifest_root_sha256": "2539d5ac14b6b52616aaca9dcba0557e799afdd237b215581adb5f1447656cdd"
            },
            "decision": "C0R_CORRECTED_CORPUS_REPEATABLE_B0R_R0R_C1_AUTHORIZED",
            "schema": "nextengine.experimental-physical-sound-v43-c0r-report.v1",
        },
        "lineage-repair.json": {
            "schema": "nextengine.experimental-physical-sound-v43-d2-repair-record.v1"
        },
        "projections/generator_train.json": projection(c1.TRAIN_ROLE, train_rows),
        "projections/generator_development.json": projection(
            c1.DEVELOPMENT_ROLE, development_rows
        ),
    }
    file_bindings = {}
    for relative, value in values.items():
        data = write_json(corpus / relative, value)
        file_bindings[relative] = binding(relative, data)

    profile = json.loads(PROFILE.read_text())
    profile["expected_counts"] = {
        "corpus_parents": 3,
        "corpus_records": 4,
        "descriptor_complete_parents": 0,
        "descriptor_complete_records": 0,
        "material_observed_parents": 2,
        "material_observed_records": 3,
        "source_artifacts": 0,
        "source_inputs": 0,
        "source_observations": 0,
    }
    profile["input_bindings"] = {
        "corpus_profile": file_bindings["profile.json"],
        "corpus_report": file_bindings["report.json"],
        "development_projection": file_bindings[
            "projections/generator_development.json"
        ],
        "lineage_repair": file_bindings["lineage-repair.json"],
        "train_projection": file_bindings["projections/generator_train.json"],
    }
    profile_path = root / "profile.json"
    write_json(profile_path, profile)
    return profile_path, corpus, profile


def claim(
    value: object,
    confidence: int | None,
    artifact_id: str = "metadata",
) -> dict[str, object]:
    return {
        "confidence_observed": confidence is not None,
        "confidence_ppm": confidence,
        "evidence_artifact_ids": [artifact_id],
        "value": value,
    }


def write_source(
    parent: Path,
    root_name: str,
    object_fields: dict[str, dict[str, object]],
    record_fields: dict[str, dict[str, object]] | None = None,
    record_id: str = "train-glass-1",
    source_component: str | None = None,
) -> tuple[Path, dict[str, object], int]:
    root = parent / root_name
    metadata = c1.canonical_json(
        {"declared": "synthetic metadata evidence", "source": root_name}
    )
    metadata_path = root / "objects" / "metadata.json"
    metadata_path.parent.mkdir(parents=True, exist_ok=True)
    metadata_path.write_bytes(metadata)
    observations = [
        {
            "fields": object_fields,
            "observation_id": "object-observation",
            "physical_parent_id": "parent-glass",
            "record_id": None,
        }
    ]
    if record_fields:
        observations.append(
            {
                "fields": record_fields,
                "observation_id": "record-observation",
                "physical_parent_id": "parent-glass",
                "record_id": record_id,
            }
        )
    manifest = {
        "artifacts": [
            {
                "artifact_id": "metadata",
                "bytes": len(metadata),
                "media_type": "application/json",
                "path": "objects/metadata.json",
                "sha256": sha256(metadata),
                "source_url": f"https://example.invalid/{root_name}/metadata.json",
            }
        ],
        "declared_revision": "revision-1",
        "landing_page_url": f"https://example.invalid/{root_name}",
        "license_expression": "CC0-1.0",
        "observations": observations,
        "project_id": f"project-{root_name}",
        "publisher_id": f"publisher-{root_name}",
        "redistribution_policy": "redistributable_with_notice",
        "schema": c1.SOURCE_SCHEMA,
        "source_component_id": source_component or f"component-{root_name}",
        "source_id": f"source-{root_name}",
        "terms_url": None,
    }
    manifest_data = write_json(root / "manifest.json", manifest)
    source_input = {
        "manifest": binding("manifest.json", manifest_data),
        "root_name": root_name,
    }
    return root, source_input, len(observations)


def complete_object_fields(mass: int = 240000) -> dict[str, dict[str, object]]:
    return {
        "cavity_kind": claim("single", 900000),
        "characteristic_wall_thickness_micrometres": claim(1800, 800000),
        "extent_micrometres": claim([80000, 80000, 180000], 950000),
        "mass_milligrams": claim(mass, 1000000),
        "material_label": claim("Glass", None),
        "opening_kind": claim("single", 900000),
        "shape_topology": claim("open_shell", 950000),
    }


def complete_record_fields() -> dict[str, dict[str, object]]:
    return {
        "impact_zone": claim("rim", 750000),
        "support_condition": claim("suspended", 750000),
    }


def directory_bytes(path: Path) -> dict[str, bytes]:
    return {item.name: item.read_bytes() for item in sorted(path.iterdir())}


class RuntimeDescriptorIntakeTests(unittest.TestCase):
    def test_tracked_contract_and_profile_are_canonical_and_bound(self) -> None:
        contract_bytes, contract = c1.read_canonical_json(CONTRACT, "contract")
        c1.validate_contract(contract)
        profile_bytes, profile = c1.read_canonical_json(PROFILE, "profile")
        c1.validate_profile(profile)
        loaded_bytes, loaded = c1.load_contract(profile)
        self.assertEqual(contract_bytes, loaded_bytes)
        self.assertEqual(contract, loaded)
        owner = next(
            item
            for item in profile["dependency_bindings"]
            if item["path"]
            == "lab/scripts/physical_sound_v44_c1_runtime_descriptor_intake_v1.py"
        )
        self.assertEqual(len(SCRIPT.read_bytes()), owner["bytes"])
        self.assertEqual(sha256(SCRIPT.read_bytes()), owner["sha256"])
        self.assertEqual(profile_bytes, c1.canonical_json(profile))

    def test_base_fixture_repeats_and_preserves_quarantine(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile, corpus, _ = build_fixture(root)
            output_a = root / "output-a"
            output_b = root / "output-b"
            c1.run(profile, corpus, [], output_a)
            c1.run(profile, corpus, [], output_b)
            self.assertEqual(directory_bytes(output_a), directory_bytes(output_b))
            report = json.loads((output_a / "report.json").read_text())
            self.assertEqual(c1.DECISION, report["decision"])
            self.assertTrue(all(report["gates"].values()))
            self.assertEqual(0, report["result"]["descriptor_complete_records"])
            records = json.loads((output_a / "descriptor-records.json").read_text())[
                "records"
            ]
            unknown = next(
                item for item in records if item["record_id"] == "train-unknown"
            )
            self.assertFalse(unknown["model_input"]["observed_mask"]["material_label"])
            self.assertEqual(
                "source_conflict",
                unknown["field_state"]["material_label"]["missing_reason"],
            )
            serialized_model_inputs = json.dumps(
                [item["model_input"] for item in records], sort_keys=True
            ).lower()
            for token in c1.FORBIDDEN_MODEL_TOKENS:
                self.assertNotIn(f'"{token}"', serialized_model_inputs)

    def test_verified_source_populates_one_complete_descriptor(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _profile_path, corpus, profile = build_fixture(root)
            source_root, source_input, observations = write_source(
                root,
                "source-a",
                complete_object_fields(),
                complete_record_fields(),
            )
            profile["source_inputs"] = [source_input]
            profile["expected_counts"].update(
                {
                    "descriptor_complete_parents": 1,
                    "descriptor_complete_records": 1,
                    "source_artifacts": 1,
                    "source_inputs": 1,
                    "source_observations": observations,
                }
            )
            profile_path = root / "source-profile.json"
            write_json(profile_path, profile)
            output = root / "output"
            c1.run(profile_path, corpus, [source_root], output)
            report = json.loads((output / "report.json").read_text())
            self.assertEqual(1, report["result"]["descriptor_complete_records"])
            records = json.loads((output / "descriptor-records.json").read_text())[
                "records"
            ]
            complete = next(
                item for item in records if item["record_id"] == "train-glass-1"
            )
            self.assertTrue(all(complete["model_input"]["observed_mask"].values()))
            self.assertEqual(
                [80000, 80000, 180000],
                complete["model_input"]["values"]["extent_micrometres"],
            )
            self.assertFalse(
                complete["model_input"]["confidence_mask"]["material_label"]
            )

    def test_conflicting_sources_mask_field_without_order_selection(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _profile_path, corpus, profile = build_fixture(root)
            source_a, input_a, observations_a = write_source(
                root,
                "source-a",
                complete_object_fields(240000),
                complete_record_fields(),
            )
            source_b, input_b, observations_b = write_source(
                root,
                "source-b",
                {"mass_milligrams": claim(260000, 1000000)},
            )
            profile["source_inputs"] = [input_a, input_b]
            profile["expected_counts"].update(
                {
                    "source_artifacts": 2,
                    "source_inputs": 2,
                    "source_observations": observations_a + observations_b,
                }
            )
            profile_path = root / "conflict-profile.json"
            write_json(profile_path, profile)
            output = root / "output"
            c1.run(profile_path, corpus, [source_b, source_a], output)
            records = json.loads((output / "descriptor-records.json").read_text())[
                "records"
            ]
            record = next(
                item for item in records if item["record_id"] == "train-glass-1"
            )
            self.assertIsNone(record["model_input"]["values"]["mass_milligrams"])
            self.assertFalse(record["model_input"]["observed_mask"]["mass_milligrams"])
            state = record["field_state"]["mass_milligrams"]
            self.assertEqual("source_conflict", state["missing_reason"])
            self.assertEqual(2, len(state["evidence_refs"]))

    def test_source_artifact_mutation_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _profile_path, corpus, profile = build_fixture(root)
            source_root, source_input, observations = write_source(
                root,
                "source-a",
                complete_object_fields(),
                complete_record_fields(),
            )
            profile["source_inputs"] = [source_input]
            profile["expected_counts"].update(
                {
                    "descriptor_complete_parents": 1,
                    "descriptor_complete_records": 1,
                    "source_artifacts": 1,
                    "source_inputs": 1,
                    "source_observations": observations,
                }
            )
            profile_path = root / "source-profile.json"
            write_json(profile_path, profile)
            (source_root / "objects" / "metadata.json").write_bytes(b"mutated")
            output = root / "output"
            with self.assertRaisesRegex(c1.DescriptorIntakeError, "bytes or SHA-256"):
                c1.run(profile_path, corpus, [source_root], output)
            self.assertFalse(output.exists())

    def test_material_override_and_field_scope_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _profile_path, corpus, profile = build_fixture(root)
            source_root, source_input, observations = write_source(
                root,
                "source-a",
                {"material_label": claim("Steel", None)},
            )
            profile["source_inputs"] = [source_input]
            profile["expected_counts"].update(
                {
                    "source_artifacts": 1,
                    "source_inputs": 1,
                    "source_observations": observations,
                }
            )
            profile_path = root / "override-profile.json"
            write_json(profile_path, profile)
            output = root / "override-output"
            with self.assertRaisesRegex(c1.DescriptorIntakeError, "disagrees"):
                c1.run(profile_path, corpus, [source_root], output)
            self.assertFalse(output.exists())

            manifest_path = source_root / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            manifest["observations"][0]["fields"] = {
                "support_condition": claim("free", None)
            }
            manifest_data = write_json(manifest_path, manifest)
            profile["source_inputs"][0]["manifest"] = binding(
                "manifest.json", manifest_data
            )
            write_json(profile_path, profile)
            with self.assertRaisesRegex(c1.DescriptorIntakeError, "scope"):
                c1.run(profile_path, corpus, [source_root], root / "scope-output")

    def test_cross_role_component_and_bound_input_mutation_publish_nothing(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, corpus, profile = build_fixture(root)
            development_path = corpus / "projections" / "generator_development.json"
            development = json.loads(development_path.read_text())
            development["rows"][0]["family_component_id"] = "component-a"
            development["rows_root_sha256"] = sha256(
                c1.canonical_json(development["rows"])
            )
            development_data = write_json(development_path, development)
            profile["input_bindings"]["development_projection"] = binding(
                "projections/generator_development.json", development_data
            )
            write_json(profile_path, profile)
            output = root / "component-output"
            with self.assertRaisesRegex(c1.DescriptorIntakeError, "component crosses"):
                c1.run(profile_path, corpus, [], output)
            self.assertFalse(output.exists())

            profile_path, corpus, _profile = build_fixture(root / "mutation")
            report_path = corpus / "report.json"
            report_path.write_bytes(report_path.read_bytes() + b" ")
            output = root / "mutation-output"
            with self.assertRaisesRegex(c1.DescriptorIntakeError, "bytes or SHA-256"):
                c1.run(profile_path, corpus, [], output)
            self.assertFalse(output.exists())

    def test_publication_is_atomic_and_repository_output_is_forbidden(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile, corpus, _ = build_fixture(root)
            output = root / "output"
            with mock.patch.object(
                c1.os, "replace", side_effect=OSError("late failure")
            ):
                with self.assertRaises(OSError):
                    c1.run(profile, corpus, [], output)
            self.assertFalse(output.exists())
            self.assertEqual([], list(root.glob(".output.*")))
            with self.assertRaisesRegex(c1.DescriptorIntakeError, "outside"):
                c1.run(profile, corpus, [], ROOT / "forbidden-c1-output")

    def test_owner_has_no_audio_feature_validator_or_network_reader(self) -> None:
        source = SCRIPT.read_text()
        for forbidden in (
            "objects/pcm/",
            "objects/features/",
            "validator_calibration.json",
            "urllib",
            "urlopen",
            "requests.",
            "socket.",
        ):
            self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
