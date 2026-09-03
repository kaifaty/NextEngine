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
SCRIPT = SCRIPTS / "physical_sound_v44_b0r_r0r_corrected_baseline_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v44-b0r-r0r-corrected-baseline.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v41_b0_grouped_baselines_v1 as b0  # noqa: E402
import physical_sound_v44_b0r_r0r_corrected_baseline_v1 as corrected  # noqa: E402


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def binding(path: str, value: bytes) -> dict[str, object]:
    return {"bytes": len(value), "path": path, "sha256": sha256(value)}


def feature(seed: int) -> dict[str, object]:
    return {
        "modes": [
            {
                "decay_confidence": 0.8,
                "frequency_hz": 400.0 + seed * 100.0,
                "prominence_db": 12.0,
                "q_factor": 50.0,
                "relative_level_db": 0.0,
                "t60_seconds": 0.7 + seed * 0.01,
            },
            {
                "decay_confidence": 0.7,
                "frequency_hz": 1200.0 + seed * 150.0,
                "prominence_db": 9.0,
                "q_factor": 40.0,
                "relative_level_db": -6.0,
                "t60_seconds": 0.7 + seed * 0.01,
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
            "decay_t20_extrapolated_t60_seconds": 0.7 + seed * 0.01,
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
    data = corrected.canonical_json(value)
    digest = sha256(data)
    relative = f"objects/features/{digest[:2]}/{digest}.json"
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return binding(relative, data)


def row(
    record_id: str,
    parent_id: str,
    family_id: str,
    component_id: str,
    material: str,
    target: dict[str, object],
    observed: bool = True,
) -> dict[str, object]:
    return {
        "acoustic_target": target,
        "axis_mask": {
            "acoustic_pseudo_target": True,
            "material_identity": observed,
            "object_identity": True,
        },
        "canonical_pcm": {"bytes": 1, "path": "unread.wav", "sha256": "0" * 64},
        "family_component_id": component_id,
        "family_id": family_id,
        "material_label": material,
        "physical_parent_id": parent_id,
        "record_id": record_id,
    }


def projection(role: str, rows: list[dict[str, object]]) -> bytes:
    value = {
        "record_count": len(rows),
        "role": role,
        "rows": rows,
        "rows_root_sha256": sha256(corrected.canonical_json(rows)),
        "schema": corrected.PROJECTION_SCHEMA,
    }
    return corrected.canonical_json(value)


def write_file(root: Path, relative: str, data: bytes) -> None:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def build_fixture(root: Path) -> dict[str, Path]:
    corpus = root / "corpus"
    materials = ("Glass", "Wood")
    train_rows = []
    development_rows = []
    seed = 0
    for role, projects, target_rows in (
        ("train", ("train-a", "train-b"), train_rows),
        ("development", ("development-a", "development-b"), development_rows),
    ):
        for project in projects:
            for material in materials:
                for ordinal in range(2):
                    parent_id = f"{role}-{project}-{material}-{ordinal}"
                    target_rows.append(
                        row(
                            f"record-{parent_id}",
                            parent_id,
                            project,
                            f"component:{project}",
                            material,
                            store_feature(corpus, feature(seed)),
                        )
                    )
                    seed += 1
    unknown_parent = "train-unknown-cross-source"
    for family in ("unknown-source-a", "unknown-source-b"):
        train_rows.append(
            row(
                f"record-{family}",
                unknown_parent,
                family,
                "component:unknown-cross-source",
                corrected.UNKNOWN_MATERIAL,
                store_feature(corpus, feature(seed)),
                observed=False,
            )
        )
        seed += 1

    train_data = projection(corrected.TRAIN_ROLE, train_rows)
    development_data = projection(corrected.DEVELOPMENT_ROLE, development_rows)
    projection_bindings = {
        corrected.TRAIN_ROLE: binding("projections/generator_train.json", train_data),
        corrected.DEVELOPMENT_ROLE: binding(
            "projections/generator_development.json", development_data
        ),
    }
    manifest = corrected.canonical_json(
        {
            "manifest_root_sha256": "a" * 64,
            "projections": projection_bindings,
            "schema": "nextengine.experimental-physical-sound-v43-c0r-corpus.v1",
        }
    )
    metadata = {
        "corpus-card.json": corrected.canonical_json(
            {"schema": "nextengine.experimental-physical-sound-v43-c0r-corpus-card.v1"}
        ),
        "lineage-repair.json": corrected.canonical_json(
            {
                "material_quarantine": {
                    "axis": "material_identity",
                    "replacement_label": corrected.UNKNOWN_MATERIAL,
                },
                "schema": "nextengine.experimental-physical-sound-v43-d2-repair-record.v1",
            }
        ),
        "manifest.json": manifest,
        "profile.json": corrected.canonical_json(
            {"schema": "nextengine.experimental-physical-sound-v43-d2-c0r-profile.v1"}
        ),
        "report.json": corrected.canonical_json(
            {
                "artifacts": {"manifest_root_sha256": "a" * 64},
                "decision": "C0R_CORRECTED_CORPUS_REPEATABLE_B0R_R0R_C1_AUTHORIZED",
                "schema": "nextengine.experimental-physical-sound-v43-c0r-report.v1",
            }
        ),
        "projections/generator_development.json": development_data,
        "projections/generator_train.json": train_data,
    }
    for relative, data in metadata.items():
        write_file(corpus, relative, data)

    profile = json.loads(PROFILE.read_text())
    profile["corpus_identity"]["manifest_root_sha256"] = "a" * 64
    profile["expected_counts"] = {
        "audit": {
            "combined_parents": 17,
            "cross_source_parents": 1,
            "leave_project_out_supported_parents": 16,
            "material_unobserved_parents": 1,
            "project_classifier_eligible_parents": 16,
            "project_count": 4,
            "project_statistics_parents": 16,
            "single_source_parents": 16,
            "source_centered_supported_parents": 16,
        },
        "baseline": {
            "coarse_supported_development_parents": 8,
            "development_parents": 8,
            "development_records": 8,
            "exact_supported_development_parents": 8,
            "material_unobserved_train_parents": 1,
            "train_parents": 9,
            "train_records": 10,
        },
    }
    profile["input_bindings"] = {
        "corpus_card": binding("corpus-card.json", metadata["corpus-card.json"]),
        "corpus_profile": binding("profile.json", metadata["profile.json"]),
        "corpus_report": binding("report.json", metadata["report.json"]),
        "development_projection": binding(
            "projections/generator_development.json", development_data
        ),
        "lineage_repair": binding(
            "lineage-repair.json", metadata["lineage-repair.json"]
        ),
        "manifest": binding("manifest.json", manifest),
        "train_projection": binding("projections/generator_train.json", train_data),
    }
    profile_path = root / "profile.json"
    profile_path.write_bytes(corrected.canonical_json(profile))
    return {"corpus": corpus, "profile": profile_path}


def rebind_projection(
    paths: dict[str, Path], role: str, value: dict[str, object]
) -> None:
    profile = json.loads(paths["profile"].read_text())
    relative = f"projections/{role}.json"
    data = corrected.canonical_json(value)
    write_file(paths["corpus"], relative, data)
    input_name = (
        "train_projection" if role == corrected.TRAIN_ROLE else "development_projection"
    )
    profile["input_bindings"][input_name] = binding(relative, data)
    manifest_path = paths["corpus"] / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    manifest["projections"][role] = binding(relative, data)
    manifest_data = corrected.canonical_json(manifest)
    manifest_path.write_bytes(manifest_data)
    profile["input_bindings"]["manifest"] = binding("manifest.json", manifest_data)
    paths["profile"].write_bytes(corrected.canonical_json(profile))


class CorrectedBaselineTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_bound(self) -> None:
        data, profile = corrected.read_canonical_profile(PROFILE)
        self.assertEqual(data, corrected.canonical_json(profile))
        corrected.validate_profile(profile)

    def test_decision_mapping_covers_all_three_terminals(self) -> None:
        classifier = {
            "accuracy_lift_over_null_median": 0.4,
            "balanced_accuracy": 0.8,
            "eligible_parent_count": 40,
            "null": {"p_value": 0.001},
        }
        passing = {"improved_parent_fraction": 0.7, "median_relative_improvement": 0.1}
        failing = {"improved_parent_fraction": 0.2, "median_relative_improvement": -0.1}
        self.assertEqual(
            corrected.corrected_decision(classifier, passing, failing, {"low": 0.01})[
                0
            ],
            "CorrectedDescriptorSignalPlausible",
        )
        self.assertEqual(
            corrected.corrected_decision(classifier, failing, passing, {"low": -0.2})[
                0
            ],
            "CorrectedDomainNormalizationRequired",
        )
        self.assertEqual(
            corrected.corrected_decision(classifier, failing, failing, {"low": -0.2})[
                0
            ],
            "CorrectedCorpusSignalInsufficient",
        )

    def test_full_fixture_repeats_and_quarantines_unknown_material(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v44-b0r-r0r-") as temporary:
            root = Path(temporary)
            paths = build_fixture(root)
            outputs = (root / "run-a", root / "run-b")
            for output in outputs:
                corrected.run(paths["profile"], paths["corpus"], output)
            files_a = {
                path.name: path.read_bytes() for path in sorted(outputs[0].iterdir())
            }
            files_b = {
                path.name: path.read_bytes() for path in sorted(outputs[1].iterdir())
            }
            self.assertEqual(files_a, files_b)
            self.assertEqual(
                set(files_a),
                {
                    "access-ledger.json",
                    "baseline-metrics.json",
                    "baseline-models.json",
                    "baseline-predictions.json",
                    "domain-audit.json",
                    "parent-inventory.json",
                    "power-plan.json",
                    "profile.json",
                    "report.json",
                    "representation.json",
                },
            )
            report = json.loads(files_a["report.json"])
            self.assertIn(report["decision"], corrected.DECISIONS)
            self.assertTrue(all(report["gates"].values()))
            self.assertEqual(report["access"]["pcm_bytes_read"], 0)
            self.assertEqual(report["access"]["validator_projection_bytes_read"], 0)
            model = json.loads(files_a["baseline-models.json"])
            self.assertEqual(model["global_training_parent_count"], 9)
            self.assertEqual(model["conditioning_training_parent_count"], 8)
            self.assertEqual(model["material_unobserved_train_parent_count"], 1)
            self.assertNotIn(corrected.UNKNOWN_MATERIAL, model["conditioning_classes"])
            self.assertNotIn(corrected.UNKNOWN_MATERIAL, model["material_prototypes"])

    def test_bound_input_mutation_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v44-input-") as temporary:
            paths = build_fixture(Path(temporary))
            card = paths["corpus"] / "corpus-card.json"
            card.write_bytes(card.read_bytes() + b" ")
            output = Path(temporary) / "output"
            with self.assertRaisesRegex(
                corrected.CorrectedBaselineError, "bytes or SHA-256"
            ):
                corrected.run(paths["profile"], paths["corpus"], output)
            self.assertFalse(output.exists())

    def test_material_mask_mutation_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v44-mask-") as temporary:
            paths = build_fixture(Path(temporary))
            projection_path = paths["corpus"] / "projections/generator_train.json"
            value = json.loads(projection_path.read_text())
            unknown = next(
                item
                for item in value["rows"]
                if item["material_label"] == corrected.UNKNOWN_MATERIAL
            )
            unknown["axis_mask"]["material_identity"] = True
            value["rows_root_sha256"] = sha256(corrected.canonical_json(value["rows"]))
            rebind_projection(paths, corrected.TRAIN_ROLE, value)
            output = Path(temporary) / "output"
            with self.assertRaisesRegex(
                corrected.CorrectedBaselineError, "marked observed"
            ):
                corrected.run(paths["profile"], paths["corpus"], output)
            self.assertFalse(output.exists())

    def test_cross_role_component_leak_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v44-leak-") as temporary:
            paths = build_fixture(Path(temporary))
            train = json.loads(
                (paths["corpus"] / "projections/generator_train.json").read_text()
            )
            projection_path = paths["corpus"] / "projections/generator_development.json"
            value = json.loads(projection_path.read_text())
            value["rows"][0]["family_component_id"] = train["rows"][0][
                "family_component_id"
            ]
            value["rows_root_sha256"] = sha256(corrected.canonical_json(value["rows"]))
            rebind_projection(paths, corrected.DEVELOPMENT_ROLE, value)
            output = Path(temporary) / "output"
            with self.assertRaisesRegex(
                b0.BaselineError, "crosses train and development"
            ):
                corrected.run(paths["profile"], paths["corpus"], output)
            self.assertFalse(output.exists())

    def test_publication_is_atomic_and_repository_output_is_forbidden(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v44-atomic-") as temporary:
            destination = Path(temporary) / "result"
            with mock.patch.object(
                corrected.os, "replace", side_effect=OSError("late")
            ):
                with self.assertRaises(OSError):
                    corrected.publish_directory(destination, {"report.json": b"{}\n"})
            self.assertFalse(destination.exists())
        with self.assertRaises(corrected.CorrectedBaselineError):
            corrected.prepare_output(ROOT / "forbidden-v44-output")

    def test_owner_has_no_candidate_validator_network_or_pcm_path(self) -> None:
        source = SCRIPT.read_text()
        for forbidden in (
            "validator_calibration.json",
            "candidate_checkpoint",
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
