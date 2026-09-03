from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v37-d0r-complete-entry-readiness.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_c0_query_surface_structural_cost_v1 as c0
import physical_sound_v37_d0r_complete_entry_readiness_v1 as d0r
import physical_sound_v37_query_surface_contract_v1 as contract


def tensor_role(
    query_features: list[list[float]], partition_keys: tuple[tuple[int, ...], ...]
) -> c0.TensorRole:
    rows = len(query_features)
    fields = rows
    query = torch.tensor(query_features, dtype=torch.float64)
    node = torch.zeros((fields, 9, 22), dtype=torch.float64)
    for field in range(fields):
        node[field, :, :8] = float(field + 1)
    kernel = torch.full((rows, 9), 1.0 / 9.0, dtype=torch.float64)
    return c0.TensorRole(
        role="artificial",
        node_features=node,
        edge_summaries=torch.zeros((fields, 9, 4), dtype=torch.float64),
        normalized_adjacency=torch.eye(9, dtype=torch.float64),
        query_features=query,
        context_features=torch.zeros((rows, 14), dtype=torch.float64),
        pointwise_features=torch.zeros((rows, 35), dtype=torch.float64),
        query_kernel_weights=kernel,
        row_field_indices=torch.arange(rows, dtype=torch.int64),
        partition_keys=partition_keys,
    )


def write_fake_pass(path: Path) -> None:
    path.mkdir()
    terminal = {
        "trace": {
            "terminal": "Pass",
            "topology_sha256": contract.expected_topology_sha256(
                contract.ProviderKind.SURROGATE_D0
            ),
        }
    }
    evidence = {
        "environment": {
            "device": "cpu",
            "dtype": "float64",
            "numpy": "2.3.5",
            "python": "3.12.13",
            "torch": "2.8.0+cu128",
        },
        "provider_evidence": d0r.zero_forbidden_access(),
    }
    (path / "candidate-bundle.json").write_bytes(d0r.canonical_json({"bundle": 1}))
    (path / "candidate-freeze.json").write_bytes(d0r.canonical_json({"freeze": 1}))
    (path / "evidence.json").write_bytes(d0r.canonical_json(evidence))
    (path / "terminal.json").write_bytes(d0r.canonical_json(terminal))


class CompleteEntryReadinessTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.context = d0r.load_context(PROFILE)

    def test_profile_closes_target_aware_algorithm_and_forbids_official_access(
        self,
    ) -> None:
        self.assertEqual(
            d0r.sha256_bytes(self.context.profile_data), d0r.PROFILE_SHA256
        )
        self.assertEqual(
            self.context.profile["workload"],
            {
                "candidate_and_control_order": [
                    "query-conditioned-surface-operator-v0",
                    "qso-field-only-ablation-v1",
                    "qso-no-interaction-ablation-v1",
                    "qso-no-topology-ablation-v1",
                    "qso-query-only-ablation-v1",
                    "v36-shaped-pointwise-mlp-v1",
                    "continuous-local-interpolation-v1",
                    "fixed-rbf-integral-ridge-v1",
                    "nearest-causal-surface-query-v1",
                ],
                "development_rows": 4320,
                "method_holdout_rows": 0,
                "train_rows": 6480,
            },
        )
        self.assertTrue(
            all(value == 0 for value in d0r.zero_forbidden_access().values())
        )

    def test_ratio_zero_semantics_and_terminal_precedence_are_exact(self) -> None:
        self.assertEqual(d0r.ratio_max(0.0, 0.0, 0.9), (None, True))
        self.assertEqual(d0r.ratio_max(1.0, 0.0, 0.9), (None, False))
        self.assertEqual(d0r.ratio_min(1.0, 0.0, 1.05), (None, True))
        self.assertEqual(d0r.ratio_min(0.0, 0.0, 1.05), (None, False))
        good = {"gate": True}
        bad = {"gate": False}
        self.assertIs(
            d0r.choose_decision(bad, bad, bad),
            contract.TerminalDecision.HARD_GATE_REJECT,
        )
        self.assertIs(
            d0r.choose_decision(good, bad, bad),
            contract.TerminalDecision.METRIC_REJECT,
        )
        self.assertIs(
            d0r.choose_decision(good, good, bad),
            contract.TerminalDecision.RESOURCE_REJECT,
        )
        self.assertIs(
            d0r.choose_decision(good, good, good), contract.TerminalDecision.PASS
        )

    def test_multiplicative_p1_witness_preserves_the_frozen_invariants(self) -> None:
        correction = np.asarray(
            [[0.25, -0.20, 0.25], [-0.25, 0.20, -0.25]] * 8,
            dtype=np.float64,
        )
        self.assertTrue(all(d0r.multiplicative_p1_witness(correction).values()))

    def test_exact_non_neural_controls_fit_and_predict_all_axes(self) -> None:
        query_rows = [[float(row), 0.0, *([0.0] * 22)] for row in range(3)]
        keys = ((1,), (1,), (1,))
        train = tensor_role(query_rows, keys)
        targets = np.asarray(
            [[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [4.0, 8.0, 12.0]],
            dtype=np.float64,
        )
        models = d0r.fit_controls(train, targets)
        query = tensor_role([[1.1, 0.0, *([0.0] * 22)]], ((1,),))
        predictions = d0r.predict_controls(models, query)
        self.assertEqual(
            set(predictions),
            {
                "continuous-local-interpolation-v1",
                "fixed-rbf-integral-ridge-v1",
                "nearest-causal-surface-query-v1",
            },
        )
        np.testing.assert_array_equal(
            predictions["nearest-causal-surface-query-v1"], [[2.0, 4.0, 6.0]]
        )
        self.assertTrue(
            all(
                value.shape == (1, 3) and np.all(np.isfinite(value))
                for value in predictions.values()
            )
        )

    def test_provider_rejects_an_official_capability_without_opening_a_role(
        self,
    ) -> None:
        provider = d0r.FullArtificialProvider(self.context, "test")
        capability = contract.AccessCapabilityV1.official(
            contract.ProviderKind.OFFICIAL_D0, "v37-test", "a" * 64
        )
        with self.assertRaisesRegex(contract.ContractError, "capability mismatch"):
            provider.materialize(contract.RoleKind.TRAIN, capability)

    def test_readiness_seal_requires_two_identical_natural_pass_trees(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v37-d0r-seal-") as raw:
            root = Path(raw)
            run_a = root / "a"
            run_b = root / "b"
            write_fake_pass(run_a)
            write_fake_pass(run_b)
            seal = d0r.build_readiness_seal(run_a, run_b)
            readiness = seal["readiness_seal"]
            self.assertIsInstance(readiness, dict)
            self.assertTrue(readiness["repeat_exact"])
            self.assertEqual(readiness["run_count"], 2)
            (run_b / "terminal.json").write_bytes(
                d0r.canonical_json({"trace": {"terminal": "MetricReject"}})
            )
            with self.assertRaisesRegex(d0r.D0ReadinessError, "not byte-identical"):
                d0r.build_readiness_seal(run_a, run_b)

    def test_checked_readiness_seal_binds_current_owner(self) -> None:
        seal = d0r.validate_checked_readiness_seal(
            LAB / "profiles" / "physical-sound-v37-d0r-readiness-seal.v1.json"
        )
        readiness = seal["readiness_seal"]
        self.assertIsInstance(readiness, dict)
        self.assertEqual(readiness["owner_sha256"], d0r.owner_identity()["sha256"])


if __name__ == "__main__":
    unittest.main()
