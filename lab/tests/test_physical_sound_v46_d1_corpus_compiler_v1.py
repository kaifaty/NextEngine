from __future__ import annotations

import copy
import hashlib
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "lab" / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v46_d1_corpus_compiler_v1 as d1  # noqa: E402

PROFILE = ROOT / d1.PROFILE_PATH
OWNER = ROOT / d1.OWNER_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_profile() -> dict[str, object]:
    return json.loads(PROFILE.read_text())


def clatter_row(filename: str, modal: dict[str, object]) -> dict[str, object]:
    recipe = {
        "heads": {"modal": modal},
        "source_lane": "empirical_prior",
    }
    target = {
        "observed_fields": ["modal.frequencies_hz"],
        "source_lane": "empirical_prior",
    }
    return {
        "filename": filename,
        "recipe": recipe,
        "recipe_sha256": sha256(d1.canonical_json(recipe)),
        "target": target,
        "target_sha256": sha256(d1.canonical_json(target)),
    }


class CorpusCompilerTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_binds_dependencies(self) -> None:
        profile_bytes, profile = d1.read_canonical_json(PROFILE, "profile")
        inputs, expected = d1.validate_profile(profile)
        self.assertEqual(profile_bytes, d1.canonical_json(profile))
        self.assertEqual(12, len(inputs))
        self.assertEqual(154, expected["real_records"])
        bindings = {item["path"]: item for item in profile["dependency_bindings"]}
        for path in (
            d1.OWNER_PATH,
            d1.PROTOCOL_PATH,
            d1.D0_PROFILE_PATH,
            d1.D0_RESULT_PATH,
        ):
            data = (ROOT / path).read_bytes()
            self.assertEqual(len(data), bindings[path]["bytes"])
            self.assertEqual(sha256(data), bindings[path]["sha256"])

    def test_clatter_groups_by_modal_values_not_filename(self) -> None:
        modal_a = {"frequencies_hz": [100.0, 200.0], "gain": 0.5}
        modal_b = {"frequencies_hz": [100.0, 250.0], "gain": 0.5}
        document = {
            "observed_fields": ["modal.frequencies_hz"],
            "rows": [
                clatter_row("a.json", modal_a),
                clatter_row("b.json", modal_a),
                clatter_row("c.json", modal_b),
            ],
            "schema": d1.C0_SCHEMA,
            "source_lane": "empirical_prior",
        }
        groups, rows, scalars = d1.compile_clatter_groups(document)
        self.assertEqual(2, len(groups))
        self.assertEqual(3, rows)
        self.assertEqual(9, scalars)
        self.assertEqual([1, 2], sorted(len(item["members"]) for item in groups))

    def test_clatter_inline_hash_drift_fails_closed(self) -> None:
        row = clatter_row("a.json", {"frequencies_hz": [100.0]})
        row["recipe_sha256"] = "0" * 64
        document = {
            "observed_fields": ["modal.frequencies_hz"],
            "rows": [row],
            "schema": d1.C0_SCHEMA,
            "source_lane": "empirical_prior",
        }
        with self.assertRaisesRegex(d1.CorpusCompilerError, "inline hash"):
            d1.compile_clatter_groups(document)

    def test_d0_cannot_silently_promote_an_untrusted_source(self) -> None:
        document = {
            "schema": d1.D0_SCHEMA,
            "sources": [
                {
                    "decision": "SyntheticTeacherUntrusted",
                    "reason": "DeclaredGenerationDocumentUnavailable",
                    "sample": {"accessed": False},
                    "source_id": "nisr-v5",
                },
                {
                    "decision": "SyntheticTeacherUntrusted",
                    "reason": "GenerationLineageIncomplete",
                    "sample": {"accessed": False},
                    "source_id": "vibraverse",
                },
            ],
        }
        d1.validate_d0_sources(document)
        changed = copy.deepcopy(document)
        changed["sources"][0]["decision"] = "SyntheticTeacherTrusted"
        with self.assertRaisesRegex(d1.CorpusCompilerError, "decisions changed"):
            d1.validate_d0_sources(changed)

    def test_content_reference_and_role_leakage_fail_closed(self) -> None:
        digest = "1" * 64
        reference = d1.validate_content_reference(
            {
                "bytes": 10,
                "path": f"objects/{digest}.json",
                "sha256": digest,
            },
            "target",
        )
        self.assertEqual(digest, reference["sha256"])
        with self.assertRaisesRegex(d1.CorpusCompilerError, "content addressed"):
            d1.validate_content_reference(
                {"bytes": 10, "path": "objects/target.json", "sha256": digest},
                "target",
            )
        rows = [
            {
                "physical_parent_id": "parent-a",
                "project_id": "project-a",
                "record_id": "record-a",
                "role": "generator_train",
                "source_component_id": "component-a",
            },
            {
                "physical_parent_id": "parent-a",
                "project_id": "project-b",
                "record_id": "record-b",
                "role": "validator_calibration",
                "source_component_id": "component-b",
            },
        ]
        with self.assertRaisesRegex(d1.CorpusCompilerError, "parent crosses roles"):
            d1.validate_role_isolation(rows)

    def test_profile_mutation_repository_output_and_imports_fail_closed(self) -> None:
        profile = load_profile()
        changed = copy.deepcopy(profile)
        changed["access_policy"]["network_allowed"] = True
        with self.assertRaisesRegex(d1.CorpusCompilerError, "access policy"):
            d1.validate_profile(changed)
        changed = copy.deepcopy(profile)
        changed["external_inputs"][0]["unexpected"] = True
        with self.assertRaisesRegex(d1.CorpusCompilerError, "fields changed"):
            d1.validate_profile(changed)
        with self.assertRaisesRegex(d1.CorpusCompilerError, "outside"):
            d1.validate_output(ROOT / "forbidden-v46-d1-output")
        source = OWNER.read_text()
        for forbidden in (
            "import httpx",
            "import librosa",
            "import numpy",
            "import requests",
            "import soundfile",
            "import torch",
            "import urllib",
            "import wave",
        ):
            self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
