from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v37-e0-full-surrogate-seal.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_e0_full_surrogate_seal_v1 as e0
import physical_sound_v37_query_surface_contract_v1 as contract


def fake_evidence() -> dict[str, object]:
    return {
        "official_access": e0.zero_official_access(),
        "rehearsal": {
            "d0": {
                "receipt": {"tree_sha256": "a" * 64},
                "topology_sha256": contract.expected_topology_sha256(
                    contract.ProviderKind.SURROGATE_D0
                ),
            },
            "environment_sha256": "b" * 64,
            "h0": {
                "receipt": {"tree_sha256": "c" * 64},
                "topology_sha256": contract.expected_topology_sha256(
                    contract.ProviderKind.SURROGATE_H0
                ),
            },
            "owner": {"sha256": "d" * 64},
            "profile_sha256": e0.PROFILE_SHA256,
            "publisher": {"sha256": "e" * 64},
        },
    }


def write_fake_run(path: Path) -> None:
    path.mkdir()
    (path / "candidate-bundle.json").write_bytes(e0.canonical_json({"bundle": 1}))
    (path / "candidate-freeze.json").write_bytes(e0.canonical_json({"freeze": 1}))
    (path / "evidence.json").write_bytes(e0.canonical_json(fake_evidence()))
    (path / "terminal.json").write_bytes(e0.canonical_json({"terminal": "Pass"}))


class FullSurrogateSealTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.context = e0.load_context(PROFILE)

    def test_profile_binds_full_frozen_workload_and_zero_authority(self) -> None:
        workload = self.context.profile["workload"]
        self.assertEqual(
            workload["role_rows"],
            {
                "development": 4320,
                "method-holdout": 4320,
                "train": 6480,
            },
        )
        self.assertEqual(workload["neural_steps_each"], 96)
        self.assertEqual(len(workload["candidate_and_control_order"]), 9)
        self.assertTrue(all(value == 0 for value in e0.zero_official_access().values()))

    def test_all_six_neural_models_encode_and_decode_byte_exactly(self) -> None:
        models = e0.model_factories()
        encoded = e0.encode_models(models)
        decoded = e0.decode_models(encoded)
        self.assertEqual(e0.encode_models(decoded), encoded)
        self.assertEqual(
            set(decoded),
            {name for name, _seed in e0.QSO_VARIANTS} | {e0.POINTWISE_NAME},
        )

    def test_model_bundle_binds_exact_binary_identity(self) -> None:
        encoded = e0.encode_models(e0.model_factories())
        document = e0.bundle_document(encoded)
        restored, models, identity = e0.load_bundle_document(document)
        self.assertEqual(restored, encoded)
        self.assertEqual(e0.encode_models(models), encoded)
        self.assertEqual(identity, e0.sha256_bytes(encoded))
        with self.assertRaises(e0.E0Error):
            e0.load_bundle_document(
                document.replace(b"weights_sha256", b"weights_shb256")
            )

    def test_seal_requires_two_byte_identical_complete_external_trees(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-e0-seal-") as raw:
            root = Path(raw)
            run_a = root / "a"
            run_b = root / "b"
            write_fake_run(run_a)
            write_fake_run(run_b)
            seal = e0.build_execution_seal(run_a, run_b)
            execution = seal["execution_seal"]
            self.assertIsInstance(execution, dict)
            self.assertEqual(execution["rehearsal_run_count"], 2)
            self.assertTrue(execution["repeat_exact"])
            self.assertEqual(execution["forbidden_access_count"], 0)
            (run_b / "terminal.json").write_bytes(
                e0.canonical_json({"terminal": "mutated"})
            )
            with self.assertRaisesRegex(e0.E0Error, "not byte-identical"):
                e0.build_execution_seal(run_a, run_b)

    def test_seal_rejects_nested_symlink_and_dependency_drift(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-e0-link-") as raw:
            run = Path(raw) / "run"
            write_fake_run(run)
            (run / "alias.json").symlink_to(run / "terminal.json")
            with self.assertRaisesRegex(e0.E0Error, "symlink"):
                e0.file_tree(run)
        first = self.context.profile["dependencies"][0]
        with self.assertRaisesRegex(e0.E0Error, "bound file drift"):
            e0.bound_file(first["path"], "0" * 64)


if __name__ == "__main__":
    unittest.main()
