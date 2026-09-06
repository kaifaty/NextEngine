from __future__ import annotations

import copy
import hashlib
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "lab" / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v44_g0b1c1_ieteasy_corpus_v1 as g0b1c1  # noqa: E402

PROFILE = ROOT / g0b1c1.PROFILE_PATH
PROTOCOL = ROOT / g0b1c1.PROTOCOL_PATH
SCRIPT = ROOT / g0b1c1.OWNER_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def target_for(index: int) -> dict[str, object]:
    tonal = [index + band + 1 for band in range(24)]
    residual = [0] * 24
    residual[0] = 1_000_000 - sum(tonal)
    return {
        "band_edges_millihz": list(range(25)),
        "qualified_peak_count": 0,
        "qualified_peak_count_by_band": [0] * 24,
        "residual_energy_ppm_by_band": residual,
        "schema": g0b1c1.g0b1c0.TARGET_SCHEMA,
        "spectral_flatness_ppm": 100_000 + index,
        "tonal_excess_energy_ppm_by_band": tonal,
        "tonal_excess_mass_ppm": sum(tonal),
        "vector_dimension": 75,
    }


def records_and_objects(unique: bool = True) -> tuple[list[dict], dict[str, bytes]]:
    records = []
    objects = {}
    for index in range(15):
        target = target_for(index if unique else 0)
        data = g0b1c1.canonical_json(target)
        digest = sha256(data)
        path = g0b1c1.content_path("target", digest)
        objects[path] = data
        records.append(
            {
                "canonical_pcm": {
                    "bytes": 44,
                    "object_path": f"objects/pcm/{index:02x}/{'1' * 64}",
                    "sha256": "1" * 64,
                },
                "descriptor": {},
                "lineage": {
                    "physical_parent_id": f"fixture-parent-{index:02d}",
                    "project_id": "fixture-project",
                },
                "material_label": "Steel",
                "record_id": f"fixture-record-{index:02d}",
                "role": "generator_train",
                "sample_id": f"fixture-{index:02d}",
                "source_payload": {"bytes": 1, "sha256": "2" * 64},
                "target": {
                    "bytes": len(data),
                    "object_path": path,
                    "qualified_peak_count": 0,
                    "schema": g0b1c1.g0b1c0.TARGET_SCHEMA,
                    "sha256": digest,
                    "spectral_flatness_ppm": target["spectral_flatness_ppm"],
                    "tonal_excess_mass_ppm": target["tonal_excess_mass_ppm"],
                    "vector_dimension": 75,
                },
            }
        )
    return records, objects


class IeteasyCorpusIncrementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.protocol_bytes, cls.protocol = g0b1c1.read_canonical_json(
            PROTOCOL, "protocol"
        )
        cls.profile_bytes, cls.profile = g0b1c1.read_canonical_json(PROFILE, "profile")

    def test_tracked_protocol_and_profile_are_canonical_and_bind_owner(self) -> None:
        g0b1c1.validate_protocol(self.protocol)
        g0b1c1.validate_profile(self.profile)
        self.assertEqual(self.protocol_bytes, g0b1c1.canonical_json(self.protocol))
        self.assertEqual(self.profile_bytes, g0b1c1.canonical_json(self.profile))
        owner = next(
            item
            for item in self.profile["dependency_bindings"]
            if item["path"] == g0b1c1.OWNER_PATH
        )
        self.assertEqual(SCRIPT.stat().st_size, owner["bytes"])
        self.assertEqual(sha256(SCRIPT.read_bytes()), owner["sha256"])

    def test_frozen_g0b1c0_target_protocol_is_bound_and_valid(self) -> None:
        target_protocol = g0b1c1.load_target_protocol(self.protocol)
        self.assertEqual(g0b1c1.g0b1c0.PROTOCOL_SCHEMA, target_protocol["schema"])
        self.assertEqual(75, target_protocol["algorithm"]["target_vector_dimension"])

    def test_valid_fixed_targets_materialize_one_train_increment(self) -> None:
        records, objects = records_and_objects()
        gates, diagnostics = g0b1c1.dataset_gates(records, objects, self.protocol)
        self.assertTrue(all(gates.values()))
        self.assertEqual(15, diagnostics["unique_target_hashes"])
        self.assertGreaterEqual(diagnostics["varying_target_dimensions"], 24)
        outputs, retained = g0b1c1.build_outputs(
            self.profile, self.protocol, records, objects, 12345
        )
        self.assertEqual(
            "CorpusSuccessorMaterialized", outputs["report.json"]["decision"]
        )
        self.assertIn("manifest.json", outputs)
        self.assertIn("projections/generator_train.json", outputs)
        self.assertEqual(objects, retained)
        self.assertEqual(
            71, outputs["report.json"]["measured"]["supported_parents_after"]
        )
        self.assertEqual(
            34, outputs["report.json"]["measured"]["supported_parent_deficit_after"]
        )

    def test_degenerate_target_matrix_returns_ood_without_content(self) -> None:
        records, objects = records_and_objects(unique=False)
        outputs, retained = g0b1c1.build_outputs(
            self.profile, self.protocol, records, objects, 12345
        )
        self.assertEqual("TargetDomainOOD", outputs["report.json"]["decision"])
        self.assertFalse(outputs["report.json"]["gates"]["target_uniqueness"])
        self.assertFalse(outputs["report.json"]["gates"]["target_variation"])
        self.assertNotIn("manifest.json", outputs)
        self.assertEqual({}, retained)

    def test_target_partition_and_protocol_mutation_fail_closed(self) -> None:
        target = target_for(0)
        g0b1c1.validate_target(target, self.protocol["decision_policy"])
        target["residual_energy_ppm_by_band"][0] -= 1
        with self.assertRaises(g0b1c1.CorpusIncrementError):
            g0b1c1.validate_target(target, self.protocol["decision_policy"])
        mutated = copy.deepcopy(self.protocol)
        mutated["decision_policy"]["minimum_unique_target_hashes"] = 14
        with self.assertRaises(g0b1c1.CorpusIncrementError):
            g0b1c1.validate_protocol(mutated)

    def test_external_input_mutation_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory(dir="/tmp") as temporary:
            root = Path(temporary)
            document = {"schema": "fixture.v1", "value": 1}
            data = g0b1c1.canonical_json(document)
            path = root / "input.json"
            path.write_bytes(data)
            binding = {
                "bytes": len(data),
                "external_relative_path": "input.json",
                "schema": "fixture.v1",
                "sha256": sha256(data),
            }
            self.assertEqual(
                document, g0b1c1.load_external_input(root, binding, "fixture")
            )
            path.write_bytes(data + b"mutation")
            with self.assertRaises(g0b1c1.CorpusIncrementError):
                g0b1c1.load_external_input(root, binding, "fixture")

    def test_content_collision_output_guard_and_atomic_publication(self) -> None:
        objects = {"objects/target/aa/value": b"first"}
        with self.assertRaises(g0b1c1.CorpusIncrementError):
            g0b1c1.add_content(objects, "objects/target/aa/value", b"second")
        with self.assertRaisesRegex(g0b1c1.CorpusIncrementError, "outside"):
            g0b1c1.prepare_output(ROOT / "forbidden-g0b1c1-output")
        with tempfile.TemporaryDirectory(dir="/tmp") as temporary:
            output = Path(temporary) / "published"
            g0b1c1.publish(
                output,
                {"report.json": {"schema": "fixture.v1"}},
                {"objects/target/aa/value": b"value"},
                self.profile_bytes,
                self.protocol_bytes,
            )
            self.assertEqual(
                b"value", (output / "objects/target/aa/value").read_bytes()
            )
            with self.assertRaises(g0b1c1.CorpusIncrementError):
                g0b1c1.publish(
                    output,
                    {},
                    {},
                    self.profile_bytes,
                    self.protocol_bytes,
                )

    def test_forbidden_network_and_model_imports_are_absent(self) -> None:
        source = SCRIPT.read_text()
        for forbidden in (
            "import requests",
            "import torch",
            "import urllib",
            "from urllib",
        ):
            self.assertNotIn(forbidden, source)
        self.assertNotIn("socket", source)
        self.assertNotIn("subprocess", source)


if __name__ == "__main__":
    unittest.main()
