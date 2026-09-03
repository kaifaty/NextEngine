from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
PROFILE = (
    LAB / "profiles" / "physical-sound-v46-b0-lane-aware-control-tournament.v1.json"
)
OWNER = SCRIPTS / "physical_sound_v46_b0_lane_aware_control_tournament_v1.py"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v44_b0r_r0r_corrected_baseline_v1 as b0r  # noqa: E402
import physical_sound_v46_b0_lane_aware_control_tournament_v1 as b0  # noqa: E402
from lab.tests import (  # noqa: E402
    test_physical_sound_v44_b0r_r0r_corrected_baseline_v1 as legacy,
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def binding(
    filename: str, data: bytes, schema: str, source_id: str
) -> dict[str, object]:
    return {
        "bytes": len(data),
        "filename": filename,
        "schema": schema,
        "sha256": sha256(data),
        "source_id": source_id,
    }


def normalized_row(row: dict[str, object], role: str) -> dict[str, object]:
    return {
        "claim_lane": "real_acoustic",
        "material_label": row["material_label"],
        "observation_mask": row["axis_mask"],
        "pcm": row["canonical_pcm"],
        "physical_parent_id": row["physical_parent_id"],
        "project_id": row["family_component_id"],
        "record_id": row["record_id"],
        "role": role,
        "source_artifact": "c0r",
        "source_component_id": row["family_component_id"],
        "target": row["acoustic_target"],
        "target_contract": "c0r_acoustic_pseudo_target",
    }


def clatter_prior() -> tuple[dict[str, object], str]:
    modal = {
        "base_frequency_millihz": 400000,
        "frequency_ratio_ppm": [1000000, 2500000],
        "mode_count": 2,
        "participation_ppm": [700000, 300000],
        "rt60_milliseconds": [700, 500],
        "uncertainty_ppm": [1000000, 1000000],
    }
    rows = []
    for index in range(2):
        recipe = {
            "heads": {"modal": modal},
            "recipe_id": f"glass_{index}",
            "source_lane": "empirical_prior",
        }
        target = {
            "observed_fields": [
                "modal.base_frequency_millihz",
                "modal.frequency_ratio_ppm",
                "modal.mode_count",
                "modal.participation_ppm",
                "modal.rt60_milliseconds",
            ],
            "source_lane": "empirical_prior",
        }
        rows.append(
            {
                "filename": f"glass_{index}_mm.bytes",
                "recipe": recipe,
                "recipe_sha256": sha256(b0.canonical_json(recipe)),
                "target": target,
                "target_sha256": sha256(b0.canonical_json(target)),
            }
        )
    document = {
        "observed_fields": rows[0]["target"]["observed_fields"],
        "profile_sha256": "a" * 64,
        "rows": rows,
        "schema": b0.CLATTER_SCHEMA,
        "source_lane": "empirical_prior",
    }
    digest = sha256(b0.canonical_json(modal))
    return document, digest


def make_fixture(root: Path) -> dict[str, Path]:
    old = legacy.build_fixture(root / "legacy")
    train_projection = json.loads(
        (old["corpus"] / "projections/generator_train.json").read_text()
    )
    development_projection = json.loads(
        (old["corpus"] / "projections/generator_development.json").read_text()
    )
    train_rows = [
        normalized_row(row, "generator_train") for row in train_projection["rows"]
    ]
    development_rows = [
        normalized_row(row, "generator_development")
        for row in development_projection["rows"]
    ]
    iet_row = {
        "claim_lane": "real_acoustic",
        "material_label": "Steel",
        "observation_mask": {"waveform": True},
        "pcm": {"bytes": 1, "path": "objects/pcm/0/0.wav", "sha256": "0" * 64},
        "physical_parent_id": "iet-parent",
        "project_id": "iet-project",
        "record_id": "iet-record",
        "role": "generator_train",
        "source_artifact": "ieteasy_increment",
        "source_component_id": "iet-component",
        "target": {
            "bytes": 1,
            "path": "objects/features/0/0.json",
            "sha256": "0" * 64,
        },
        "target_contract": (
            "nextengine.experimental-physical-sound-v44-noise-robust-fixed-target.v1"
        ),
    }
    prior, modal_digest = clatter_prior()
    group = {
        "group_id": f"clatter-modal-{modal_digest}",
        "members": [{"filename": row["filename"]} for row in prior["rows"]],
        "modal_head_sha256": modal_digest,
    }
    index = {
        "authority": {"authorizes_only": "v46_b0_control_tournament"},
        "claim": "fixture",
        "control_lanes": {
            "empirical_prior": {"authority": "control_only", "groups": [group]},
            "modal_teacher": {"reason": "D0_NoTrustedSyntheticTeacher", "rows": []},
            "structural_transfer": {
                "reason": "NoMaterializedStructuralRowsYet",
                "rows": [],
            },
        },
        "planning_power": {},
        "real_rows": train_rows + development_rows + [iet_row],
        "recipe_contract": {},
        "schema": b0.D1_INDEX_SCHEMA,
        "source_artifacts": [],
        "study_id": "fixture",
    }
    d1_access = {
        "content_objects_opened": 0,
        "model_values_read": 0,
        "pcm_sample_values_decoded": 0,
        "protected_values_read": 0,
        "real_target_values_decoded": 0,
        "schema": b0.D1_ACCESS_SCHEMA,
        "waveform_bytes_read": 0,
    }
    d1_report = {
        "decision": "CorpusIndexCompiled",
        "gates": {"fixture": True},
        "schema": b0.D1_REPORT_SCHEMA,
    }

    expected = {
        "clatter_control_groups": 1,
        "clatter_control_rows": 2,
        "c0r_development_parents": 8,
        "c0r_development_projects": 2,
        "c0r_development_records": 8,
        "c0r_supported_development_parents": 8,
        "c0r_target_bytes": sum(
            row["target"]["bytes"] for row in train_rows + development_rows
        ),
        "c0r_target_objects": len(train_rows + development_rows),
        "c0r_train_parents": 9,
        "c0r_train_projects": 3,
        "c0r_train_records": 10,
        "d1_real_records": len(index["real_rows"]),
        "ieteasy_train_records": 1,
        "modal_teacher_rows": 0,
        "structural_transfer_rows": 0,
    }
    mapped_train, mapped_development, projects = b0.c0r_rows_by_role(
        train_rows + development_rows, expected
    )
    train, _ = b0r.aggregate_parents(mapped_train, old["corpus"], b0.MATERIAL_TAXONOMY)
    development, _ = b0r.aggregate_parents(
        mapped_development, old["corpus"], b0.MATERIAL_TAXONOMY
    )
    current, _ = b0.fit_controls(train, development, projects)
    historical_metrics = {
        "baseline_metrics": {
            name: {
                "coarse_supported": current["baseline_metrics"][name][
                    "coarse_supported"
                ]
            }
            for name in b0.CONTROL_NAMES
        },
        "schema": b0.HISTORICAL_METRICS_SCHEMA,
    }
    historical_report = {
        "access": {
            "pcm_bytes_read": 0,
            "validator_projection_bytes_read": 0,
        },
        "decision": "CorrectedCorpusSignalInsufficient",
        "schema": b0.HISTORICAL_REPORT_SCHEMA,
    }

    documents = {
        "clatter-prior.json": prior,
        "d1-access.json": d1_access,
        "d1-corpus-index.json": index,
        "d1-report.json": d1_report,
        "historical-baseline-metrics.json": historical_metrics,
        "historical-report.json": historical_report,
    }
    input_root = root / "inputs"
    input_root.mkdir()
    input_bindings = []
    source_ids = {name: f"fixture-{name.removesuffix('.json')}" for name in documents}
    for name, document in sorted(documents.items()):
        data = b0.canonical_json(document)
        (input_root / name).write_bytes(data)
        input_bindings.append(binding(name, data, document["schema"], source_ids[name]))

    profile = json.loads(PROFILE.read_text())
    profile["expected"] = expected
    profile["external_inputs"] = input_bindings
    profile_path = root / "profile.json"
    profile_path.write_bytes(b0.canonical_json(profile))
    return {
        "c0r": old["corpus"],
        "inputs": input_root,
        "profile": profile_path,
    }


class LaneAwareControlTournamentTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_bound(self) -> None:
        data = PROFILE.read_bytes()
        profile = json.loads(data)
        self.assertEqual(data, b0.canonical_json(profile))
        b0.validate_profile(profile)

    def test_full_fixture_repeats_and_returns_no_useful_teacher(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v46-b0-") as temporary:
            root = Path(temporary)
            fixture = make_fixture(root)
            outputs = [root / "run-a", root / "run-b"]
            for output in outputs:
                b0.run(fixture["profile"], fixture["inputs"], fixture["c0r"], output)
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
                    "access.json",
                    "control-metrics.json",
                    "lane-matrix.json",
                    "profile.json",
                    "report.json",
                },
            )
            report = json.loads(files_a["report.json"])
            self.assertEqual(report["decision"], "NoUsefulTeacher")
            self.assertEqual(
                report["real_control_floor"]["control_name"], "global_prototype"
            )
            self.assertFalse(
                report["gates"]["teacher_usefulness_independently_scoreable"]
            )
            access = json.loads(files_a["access.json"])
            self.assertTrue(all(access[name] == 0 for name in b0.ZERO_ACCESS_COUNTERS))
            lanes = json.loads(files_a["lane-matrix.json"])
            self.assertEqual(lanes["cross_contract_comparison_count"], 0)
            self.assertEqual(lanes["scoreable_non_real_teacher_count"], 0)

    def test_target_contract_and_clatter_mutations_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v46-b0-mutate-"
        ) as temporary:
            root = Path(temporary)
            fixture = make_fixture(root)
            profile = json.loads(fixture["profile"].read_text())
            documents, _ = b0.load_external_inputs(fixture["inputs"], profile)
            _, rows, _ = b0.validate_d1(documents, profile["expected"])
            changed = copy.deepcopy(rows)
            changed[0]["target_contract"] = "recipe-v3"
            with self.assertRaisesRegex(b0.ControlTournamentError, "contract"):
                b0.c0r_rows_by_role(changed, profile["expected"])
            prior = copy.deepcopy(documents["clatter-prior.json"])
            prior["rows"][0]["recipe"]["heads"]["modal"]["base_frequency_millihz"] += 1
            with self.assertRaisesRegex(b0.ControlTournamentError, "inline hash"):
                b0.clatter_groups(prior)

    def test_profile_external_directory_and_output_guards(self) -> None:
        profile = json.loads(PROFILE.read_text())
        changed = copy.deepcopy(profile)
        changed["access_policy"]["ieteasy_target_values_allowed"] = True
        with self.assertRaisesRegex(b0.ControlTournamentError, "policy"):
            b0.validate_profile(changed)
        with self.assertRaisesRegex(b0.ControlTournamentError, "outside"):
            b0.prepare_output(ROOT / "forbidden-v46-b0-output")
        source = OWNER.read_text()
        for forbidden in (
            "import httpx",
            "import librosa",
            "import requests",
            "import soundfile",
            "import torch",
            "import urllib",
            "import wave",
        ):
            self.assertNotIn(forbidden, source)

    def test_decision_mapping_requires_independent_scoreable_teacher(self) -> None:
        matrix = {"scoreable_non_real_teacher_count": 0}
        self.assertEqual(b0.decide(matrix), "NoUsefulTeacher")
        matrix["scoreable_non_real_teacher_count"] = 1
        self.assertEqual(b0.decide(matrix), "ControlFloorFrozen")


if __name__ == "__main__":
    unittest.main()
