from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import numpy as np

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v41_b0_grouped_baselines_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v41-b0-grouped-baselines.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v41_b0_grouped_baselines_v1 as b0  # noqa: E402


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def binding(path: str, value: bytes) -> dict[str, object]:
    return {"bytes": len(value), "path": path, "sha256": sha256(value)}


def feature(seed: int, decay: float | None = 0.8) -> dict[str, object]:
    return {
        "modes": [
            {
                "decay_confidence": 0.8,
                "frequency_hz": 400.0 + seed * 100.0,
                "prominence_db": 12.0,
                "q_factor": 50.0,
                "relative_level_db": 0.0,
                "t60_seconds": decay,
            },
            {
                "decay_confidence": 0.7,
                "frequency_hz": 1200.0 + seed * 150.0,
                "prominence_db": 9.0,
                "q_factor": 40.0,
                "relative_level_db": -6.0,
                "t60_seconds": decay,
            },
        ],
        "sample_count": 144000,
        "sample_rate_hz": 48000,
        "schema": b0.FEATURE_SCHEMA,
        "spectral": {
            "band_edges_hz": [float(index + 1) for index in range(25)],
            "band_energy_db": [-20.0 - seed - index * 0.2 for index in range(24)],
            "bandwidth_hz": 2200.0 + seed * 30.0,
            "centroid_hz": 1800.0 + seed * 25.0,
            "flatness": 0.08 + seed * 0.002,
            "rolloff_95_hz": 6000.0 + seed * 50.0,
        },
        "time": {
            "crest_factor": 4.0 + seed * 0.1,
            "decay_t20_extrapolated_t60_seconds": decay,
            "peak_offset_ms": 2.0 + seed * 0.1,
            "rms": 0.1,
            "temporal_centroid_seconds": 0.4 + seed * 0.01,
            "zero_crossing_rate": 0.08 + seed * 0.001,
        },
        "transient_envelope_db": [
            -float(index) * 0.5 - seed * 0.1 for index in range(48)
        ],
        "uncertainty": {},
    }


def store_feature(root: Path, value: dict[str, object]) -> dict[str, object]:
    data = b0.canonical_json(value)
    digest = sha256(data)
    relative = f"objects/features/{digest[:2]}/{digest}.json"
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return binding(relative, data)


def row(
    record_id: str,
    parent_id: str,
    material: str,
    component: str,
    target: dict[str, object],
) -> dict[str, object]:
    return {
        "acoustic_target": target,
        "axis_mask": {},
        "canonical_pcm": {"bytes": 1, "path": "unread.wav", "sha256": "0" * 64},
        "family_component_id": component,
        "family_id": component,
        "material_label": material,
        "physical_parent_id": parent_id,
        "record_id": record_id,
    }


def projection(role: str, rows: list[dict[str, object]]) -> bytes:
    value = {
        "record_count": len(rows),
        "role": role,
        "rows": rows,
        "rows_root_sha256": sha256(b0.canonical_json(rows)),
        "schema": b0.PROJECTION_SCHEMA,
    }
    return b0.canonical_json(value)


def build_fixture(root: Path) -> dict[str, Path]:
    corpus = root / "corpus"
    train_rows = [
        row(
            "train-glass-a",
            "train-glass",
            "Glass",
            "train-glass-component",
            store_feature(corpus, feature(0)),
        ),
        row(
            "train-glass-b",
            "train-glass",
            "Glass",
            "train-glass-component",
            store_feature(corpus, feature(1)),
        ),
        row(
            "train-wood",
            "train-wood",
            "Wood",
            "train-wood-component",
            store_feature(corpus, feature(2)),
        ),
    ]
    development_rows = [
        row(
            "dev-glass",
            "dev-glass",
            "Glass",
            "dev-glass-component",
            store_feature(corpus, feature(3)),
        ),
        row(
            "dev-wood",
            "dev-wood",
            "Wood",
            "dev-wood-component",
            store_feature(corpus, feature(4)),
        ),
        row(
            "dev-ceramic",
            "dev-ceramic",
            "Ceramic",
            "dev-ceramic-component",
            store_feature(corpus, feature(5, None)),
        ),
    ]
    files: dict[str, bytes] = {
        "corpus-card.json": b0.canonical_json({"fixture": "card"}),
        "profile.json": b0.canonical_json({"fixture": "profile"}),
        "report.json": b0.canonical_json({"fixture": "report"}),
        "projections/generator_train.json": projection(b0.TRAIN_ROLE, train_rows),
        "projections/generator_development.json": projection(
            b0.DEVELOPMENT_ROLE, development_rows
        ),
    }
    for relative, data in files.items():
        path = corpus / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    profile = {
        "authority": b0.AUTHORITY,
        "baseline_policy": b0.BASELINE_POLICY,
        "dependency_bindings": [],
        "environment": {"numpy_version": np.__version__},
        "evaluation_policy": b0.EVALUATION_POLICY,
        "expected_counts": {
            "coarse_supported_development_parents": 2,
            "development_parents": 3,
            "development_records": 3,
            "exact_supported_development_parents": 2,
            "train_parents": 2,
            "train_records": 3,
        },
        "input_bindings": {
            "corpus_card": binding("corpus-card.json", files["corpus-card.json"]),
            "corpus_profile": binding("profile.json", files["profile.json"]),
            "corpus_report": binding("report.json", files["report.json"]),
            "development_projection": binding(
                "projections/generator_development.json",
                files["projections/generator_development.json"],
            ),
            "train_projection": binding(
                "projections/generator_train.json",
                files["projections/generator_train.json"],
            ),
        },
        "material_taxonomy": {
            "Ceramic": "ceramic",
            "Glass": "glass",
            "Wood": "wood",
        },
        "profile_id": "synthetic-b0-test",
        "representation_policy": b0.REPRESENTATION_POLICY,
        "schema": b0.PROFILE_SCHEMA,
    }
    profile_path = root / "profile.json"
    profile_path.write_bytes(b0.canonical_json(profile))
    return {"corpus": corpus, "profile": profile_path}


class GroupedBaselineTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_bound(self) -> None:
        data, profile = b0.read_canonical_profile(PROFILE)
        self.assertEqual(data, b0.canonical_json(profile))
        b0.validate_profile(profile)

    def test_feature_vector_has_frozen_shape_and_decay_mask(self) -> None:
        vector, mask = b0.feature_vector(feature(1))
        self.assertEqual(vector.shape, (105,))
        self.assertTrue(np.all(mask))
        missing_vector, missing_mask = b0.feature_vector(feature(2, None))
        self.assertEqual(missing_vector.shape, (105,))
        self.assertFalse(missing_mask[100])
        self.assertEqual(len(b0.representation_names()), 105)

    def test_full_fixture_is_repeat_exact_grouped_and_ood_safe(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-b0-run-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            outputs = [root / "run-a", root / "run-b"]
            for output in outputs:
                b0.run(paths["profile"], paths["corpus"], output)
            left = {path.name: path.read_bytes() for path in outputs[0].iterdir()}
            right = {path.name: path.read_bytes() for path in outputs[1].iterdir()}
            self.assertEqual(left, right)
            report = json.loads((outputs[0] / "report.json").read_text())
            self.assertEqual(report["decision"], b0.DECISION)
            self.assertTrue(all(report["gates"].values()))
            self.assertEqual(
                report["counts"]["coarse_supported_development_parents"], 2
            )
            self.assertEqual(report["access"]["validator_projection_bytes_read"], 0)
            self.assertEqual(report["access"]["pcm_bytes_read"], 0)
            metrics = json.loads((outputs[0] / "metrics.json").read_text())
            self.assertEqual(set(metrics["baseline_metrics"]), set(b0.BASELINE_NAMES))
            for value in metrics["baseline_metrics"].values():
                self.assertEqual(value["ood_parent_count"], 1)

    def test_ridge_and_mlp_fit_are_repeat_exact(self) -> None:
        inputs = np.eye(3, dtype=np.float64)
        targets = np.arange(3 * 105, dtype=np.float64).reshape(3, 105) / 100.0
        masks = np.ones_like(targets, dtype=bool)
        first_ridge = b0.fit_ridge(inputs, targets, masks)
        second_ridge = b0.fit_ridge(inputs.copy(), targets.copy(), masks.copy())
        for first, second in zip(first_ridge, second_ridge, strict=True):
            self.assertTrue(np.array_equal(first, second))
        first_mlp, first_loss = b0.fit_mlp(inputs, targets, masks)
        second_mlp, second_loss = b0.fit_mlp(
            inputs.copy(), targets.copy(), masks.copy()
        )
        self.assertEqual(first_loss, second_loss)
        for name in first_mlp:
            self.assertTrue(np.array_equal(first_mlp[name], second_mlp[name]))

    def test_split_rejects_parent_component_and_record_leakage(self) -> None:
        train = [
            {
                "family_component_id": "train-component",
                "physical_parent_id": "train-parent",
                "record_id": "train-record",
            }
        ]
        mutations = [
            {**train[0], "record_id": "development-record"},
            {
                **train[0],
                "physical_parent_id": "development-parent",
                "record_id": "development-record",
            },
            {
                "family_component_id": "development-component",
                "physical_parent_id": "development-parent",
                "record_id": "train-record",
            },
        ]
        for mutation in mutations:
            with (
                self.subTest(mutation=mutation),
                self.assertRaisesRegex(
                    b0.BaselineError, "crosses train and development"
                ),
            ):
                b0.validate_split(train, [mutation])

    def test_feature_hash_mutation_rejects_before_publication(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-b0-feature-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            feature_path = next(
                (paths["corpus"] / "objects" / "features").rglob("*.json")
            )
            feature_path.write_bytes(feature_path.read_bytes() + b"mutation")
            output = root / "output"
            with self.assertRaisesRegex(b0.BaselineError, "bytes or SHA-256"):
                b0.run(paths["profile"], paths["corpus"], output)
            self.assertFalse(output.exists())

    def test_authority_and_atomic_output_mutations_reject(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-b0-atomic-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            profile = json.loads(paths["profile"].read_text())
            changed = copy.deepcopy(profile)
            changed["authority"]["validator_access_authority"] = True
            with self.assertRaisesRegex(b0.BaselineError, "authority changed"):
                b0.validate_profile(changed)
            output = root / "result"
            writes = 0
            original = Path.write_bytes

            def fail_second(path: Path, data: bytes) -> int:
                nonlocal writes
                writes += 1
                if writes == 2:
                    raise OSError("injected publication failure")
                return original(path, data)

            with (
                mock.patch.object(Path, "write_bytes", fail_second),
                self.assertRaisesRegex(OSError, "injected"),
            ):
                b0.publish_directory(output, {"a.json": b"1", "b.json": b"2"})
            self.assertFalse(output.exists())
            self.assertEqual(
                sorted(path.name for path in root.iterdir()),
                ["corpus", "profile.json"],
            )

    def test_owner_has_no_validator_network_pcm_or_candidate_dependency(self) -> None:
        source = SCRIPT.read_text()
        for forbidden in (
            "validator_calibration.json",
            "import requests",
            "import socket",
            "import torch",
            "import urllib",
            'canonical_pcm"]["path',
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
