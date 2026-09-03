from __future__ import annotations

import copy
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
SCRIPT = SCRIPTS / "physical_sound_v42_r0_domain_information_audit_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v42-r0-domain-information-audit.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v41_b0_grouped_baselines_v1 as b0  # noqa: E402
import physical_sound_v42_r0_domain_information_audit_v1 as r0  # noqa: E402


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
    family_id: str,
    material: str,
    target: dict[str, object],
) -> dict[str, object]:
    return {
        "acoustic_target": target,
        "axis_mask": {},
        "canonical_pcm": {"bytes": 1, "path": "unread.wav", "sha256": "0" * 64},
        "family_component_id": f"component:{family_id}",
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
        "rows_root_sha256": sha256(b0.canonical_json(rows)),
        "schema": b0.PROJECTION_SCHEMA,
    }
    return b0.canonical_json(value)


def write_bound_files(root: Path, files: dict[str, bytes]) -> dict[str, object]:
    result = {}
    for name, data in files.items():
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        result[Path(name).stem] = binding(name, data)
    return result


def build_fixture(root: Path) -> dict[str, Path]:
    corpus = root / "corpus"
    baseline = root / "baseline"
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
                            material,
                            store_feature(corpus, feature(seed)),
                        )
                    )
                    seed += 1

    corpus_files = {
        "corpus-card.json": b0.canonical_json({"fixture": "card"}),
        "profile.json": b0.canonical_json({"fixture": "profile"}),
        "projections/generator_development.json": projection(
            b0.DEVELOPMENT_ROLE, development_rows
        ),
        "projections/generator_train.json": projection(b0.TRAIN_ROLE, train_rows),
        "report.json": b0.canonical_json({"fixture": "report"}),
    }
    write_bound_files(corpus, corpus_files)

    baseline_files = {
        "metrics.json": r0.canonical_json(
            {
                "summary": {
                    "best_baseline": "global_prototype",
                    "reference_floor": {"median_parent_rmse": 1.33759365},
                }
            }
        ),
        "profile.json": r0.canonical_json({"fixture": "baseline-profile"}),
        "report.json": r0.canonical_json(
            {"decision": "B0_GROUPED_BASELINE_SURFACE_FROZEN_M0_AUTHORIZED"}
        ),
        "representation.json": r0.canonical_json({"policy": b0.REPRESENTATION_POLICY}),
    }
    write_bound_files(baseline, baseline_files)

    tracked = json.loads(PROFILE.read_text())
    profile = copy.deepcopy(tracked)
    profile["expected_counts"] = {
        "combined_parents": 16,
        "cross_source_parents": 0,
        "development_parents": 8,
        "development_records": 8,
        "leave_project_out_supported_parents": 16,
        "project_classifier_eligible_parents": 16,
        "project_count": 4,
        "single_source_parents": 16,
        "source_centered_supported_parents": 16,
        "train_parents": 8,
        "train_records": 8,
    }
    profile["input_bindings"] = {
        "baseline": {
            name: binding(f"{name}.json", baseline_files[f"{name}.json"])
            for name in ("metrics", "profile", "report", "representation")
        },
        "corpus": {
            "corpus_card": binding(
                "corpus-card.json", corpus_files["corpus-card.json"]
            ),
            "development_projection": binding(
                "projections/generator_development.json",
                corpus_files["projections/generator_development.json"],
            ),
            "profile": binding("profile.json", corpus_files["profile.json"]),
            "report": binding("report.json", corpus_files["report.json"]),
            "train_projection": binding(
                "projections/generator_train.json",
                corpus_files["projections/generator_train.json"],
            ),
        },
    }
    profile_path = root / "r0-profile.json"
    profile_path.write_bytes(r0.canonical_json(profile))
    return {
        "baseline": baseline,
        "corpus": corpus,
        "profile": profile_path,
    }


class DomainInformationAuditTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_bound(self) -> None:
        data, profile = r0.read_canonical_profile(PROFILE)
        self.assertEqual(data, r0.canonical_json(profile))
        r0.validate_profile(profile)

    def test_decision_precedence_covers_all_three_terminals(self) -> None:
        classifier = {
            "accuracy_lift_over_null_median": 0.4,
            "balanced_accuracy": 0.8,
            "eligible_parent_count": 40,
            "null": {"p_value": 0.001},
        }
        passing = {
            "improved_parent_fraction": 0.7,
            "median_relative_improvement": 0.1,
        }
        failing = {
            "improved_parent_fraction": 0.2,
            "median_relative_improvement": -0.1,
        }
        self.assertEqual(
            r0.decision(classifier, passing, failing, {"low": 0.01})[0],
            "DescriptorSignalPlausible",
        )
        self.assertEqual(
            r0.decision(classifier, failing, passing, {"low": -0.2})[0],
            "DomainNormalizationRequired",
        )
        self.assertEqual(
            r0.decision(classifier, failing, failing, {"low": -0.2})[0],
            "CorpusSignalInsufficient",
        )

    def test_full_synthetic_run_repeats_and_reads_no_pcm(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            paths = build_fixture(Path(temporary))
            output_a = Path(temporary) / "output-a"
            output_b = Path(temporary) / "output-b"
            r0.run(paths["profile"], paths["corpus"], paths["baseline"], output_a)
            r0.run(paths["profile"], paths["corpus"], paths["baseline"], output_b)
            files_a = {
                path.name: path.read_bytes() for path in sorted(output_a.iterdir())
            }
            files_b = {
                path.name: path.read_bytes() for path in sorted(output_b.iterdir())
            }
            self.assertEqual(files_a, files_b)
            report = json.loads(files_a["report.json"])
            self.assertIn(report["decision"], r0.DECISIONS)
            self.assertEqual(report["counts"]["combined_parents"], 16)
            self.assertEqual(report["access"]["pcm_bytes_read"], 0)
            self.assertEqual(report["access"]["validator_projection_bytes_read"], 0)
            self.assertTrue(all(report["gates"].values()))

    def test_bound_input_mutation_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            paths = build_fixture(Path(temporary))
            metrics = paths["baseline"] / "metrics.json"
            metrics.write_bytes(metrics.read_bytes() + b" ")
            output = Path(temporary) / "output"
            with self.assertRaises(r0.AuditError):
                r0.run(paths["profile"], paths["corpus"], paths["baseline"], output)
            self.assertFalse(output.exists())

    def test_cross_role_component_leak_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            paths = build_fixture(Path(temporary))
            profile = json.loads(paths["profile"].read_text())
            development_path = (
                paths["corpus"] / "projections/generator_development.json"
            )
            development = json.loads(development_path.read_text())
            development["rows"][0]["family_component_id"] = "component:train-a"
            development["rows_root_sha256"] = sha256(
                b0.canonical_json(development["rows"])
            )
            development_bytes = b0.canonical_json(development)
            development_path.write_bytes(development_bytes)
            profile["input_bindings"]["corpus"]["development_projection"] = binding(
                "projections/generator_development.json", development_bytes
            )
            paths["profile"].write_bytes(r0.canonical_json(profile))
            output = Path(temporary) / "output"
            with self.assertRaises(b0.BaselineError):
                r0.run(paths["profile"], paths["corpus"], paths["baseline"], output)
            self.assertFalse(output.exists())

    def test_publication_is_atomic_and_repository_output_is_forbidden(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / "result"
            with mock.patch.object(r0.os, "replace", side_effect=OSError("late")):
                with self.assertRaises(OSError):
                    r0.publish_directory(destination, {"audit.json": b"{}\n"})
            self.assertFalse(destination.exists())
        with self.assertRaises(r0.AuditError):
            r0.prepare_output(ROOT / "forbidden-r0-output")

    def test_owner_has_no_candidate_or_validator_input_path(self) -> None:
        source = SCRIPT.read_text()
        self.assertNotIn("validator_calibration.json", source)
        self.assertNotIn("candidate_checkpoint", source)
        self.assertNotIn("urllib", source)
        self.assertNotIn("requests.", source)


if __name__ == "__main__":
    unittest.main()
