from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np


LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v32-m1-execution-mechanics.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v32_m1_known_truth_tournament_v1 as m1  # noqa: E402


class KnownTruthTournamentOwnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, cls.profile_data = m1.load_execution_profile(PROFILE)
        root = m1.repository_root()
        cls.m0_profile, _ = m1.m0.load_profile(
            root / cls.profile["dependencies"]["m0_profile"]["path"]
        )
        cls.p0_profile, _ = m1.m0.p1.p0.load_profile(
            root / cls.m0_profile["parent"]["p0_profile"]["path"]
        )

    def test_profile_and_dependency_closure_are_exact(self) -> None:
        dependencies = m1.validate_dependencies(self.profile)
        self.assertEqual(
            set(dependencies), set(self.profile["dependencies"]) | {"m1_protocol"}
        )
        self.assertEqual(m1.sha256_bytes(self.profile_data), m1.PROFILE_SHA256)
        self.assertEqual(self.profile["role_order"], ["train", "development", "method_holdout"])
        self.assertFalse(self.profile["execution"]["run_b_is_retry"])

    def test_complete_fixture_enumeration_is_valid_without_solving_roles(self) -> None:
        corpus = self.m0_profile["corpus"]
        counts: dict[str, int] = {}
        groups: dict[str, int] = {}
        for role, cells in corpus["role_geometry_cells"].items():
            case_count = 0
            group_ids: set[tuple[int, int, int]] = set()
            for family_index in range(len(corpus["families"])):
                for material_index in range(len(corpus["materials"])):
                    for cell_index in cells:
                        group_ids.add((family_index, material_index, cell_index))
                        for contact_index in range(len(corpus["contacts"])):
                            fixture, multipliers = m1.build_fixture(
                                self.m0_profile,
                                self.p0_profile,
                                family_index,
                                material_index,
                                cell_index,
                                contact_index,
                            )
                            validated = m1.m0.p1.validate_fixture(fixture, self.p0_profile)
                            self.assertEqual(validated["mode_count"], 10)
                            self.assertEqual(len(multipliers), 3)
                            case_count += 1
            counts[role] = case_count
            groups[role] = len(group_ids)
            self.assertEqual(case_count, corpus["counts"][role]["cases"])
            self.assertEqual(len(group_ids), corpus["counts"][role]["object_groups"])
        self.assertEqual(sum(counts.values()), 576)
        self.assertEqual(sum(groups.values()), 72)

    def test_ridge_nearest_metrics_and_zero_denominator_are_frozen(self) -> None:
        train_x = np.asarray([[-1.0], [0.0], [1.0]], dtype=np.float64)
        train_y = np.asarray([-2.0, 1.0, 4.0], dtype=np.float64)
        beta, intercept = m1.fit_ridge(train_x, train_y)
        self.assertAlmostEqual(float(beta[0]), 2.99999850000075)
        self.assertAlmostEqual(intercept, 1.0)
        nearest = m1.predict_nearest(
            train_x,
            train_y,
            np.asarray([[-0.5], [0.5]], dtype=np.float64),
        )
        np.testing.assert_array_equal(nearest, np.asarray([-2.0, 1.0]))
        self.assertEqual(m1.ratio(0.0, 0.0), (0.0, True))
        self.assertEqual(m1.ratio(1.0, 0.0), (0.0, False))
        metric = m1.metric_set(np.zeros((2, 3)), np.ones((2, 3)))
        self.assertAlmostEqual(metric["branch_rmse"]["decay"], 1.0)
        self.assertAlmostEqual(
            metric["aggregate_normalized_rmse"],
            np.mean([1.0 / 0.20, 1.0 / 0.16, 1.0 / 0.22]),
        )

    def test_binary_serialization_is_stable_and_ordered(self) -> None:
        model_a = m1.m0.ResidualModel(self.m0_profile)
        model_b = m1.m0.ResidualModel(self.m0_profile)
        weights_a = m1.encode_weights(model_a)
        weights_b = m1.encode_weights(model_b)
        self.assertEqual(weights_a, weights_b)
        self.assertTrue(weights_a.startswith(m1.WEIGHTS_MAGIC))
        predictions = np.asarray([[0.1, -0.2, 0.3], [0.0, 0.0, 0.0]])
        encoded_a = m1.encode_predictions(["a", "b"], predictions)
        encoded_b = m1.encode_predictions(["a", "b"], predictions.copy())
        self.assertEqual(encoded_a, encoded_b)
        self.assertTrue(encoded_a.startswith(m1.PREDICTIONS_MAGIC))

    def test_repeat_comparator_detects_exact_and_drift(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v32-m1-compare-") as temporary:
            root = Path(temporary)
            a = root / "a"
            b = root / "b"
            a.mkdir()
            b.mkdir()
            payload = m1.canonical_json({"decision": "REJECT_DEVELOPMENT"})
            (a / "report.json").write_bytes(payload)
            (b / "report.json").write_bytes(payload)
            exact = m1.compare_runs(a, b)
            self.assertEqual(exact["decision"], "REPEAT_EXACT")
            (b / "report.json").write_text(json.dumps({"decision": "drift"}))
            drift = m1.compare_runs(a, b)
            self.assertEqual(drift["decision"], "REJECT_NONDETERMINISTIC")


if __name__ == "__main__":
    unittest.main()
