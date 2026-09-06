from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from collections import defaultdict
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
V0_PROFILE = LAB / "profiles" / "physical-sound-v32-v0-validator-mechanics.v1.json"
T0_PROFILE = LAB / "profiles" / "physical-sound-v32-t0-truth-mutations.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v32_t0_truth_mutations_v1 as t0  # noqa: E402
import physical_sound_v32_v0_validator_mechanics_v1 as v0  # noqa: E402


class FrozenRejectedValidatorTests(unittest.TestCase):
    def test_complete_entry_rejects_on_clean_causal_alias_without_publication(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v32-v0-reject-") as temporary:
            root = Path(temporary)
            truth = root / "truth"
            output = root / "validation"
            t0.run(T0_PROFILE, truth)
            with self.assertRaisesRegex(
                v0.ValidatorMechanicsError,
                r"decision matrix mismatch: intervention-density: "
                r"Reject \['RetrievalCopyDetected'\]",
            ):
                v0.run(V0_PROFILE, truth, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(path.name.startswith(".nextengine-v32-v0-") for path in root.iterdir())
            )

    def test_clean_hash_aliases_are_modal_equivalent(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v32-v0-alias-") as temporary:
            truth = Path(temporary) / "truth"
            t0.run(T0_PROFILE, truth)
            release = json.loads((truth / "truth-release.json").read_text())
            groups: dict[str, list[tuple[str, list[tuple[object, ...]]]]] = defaultdict(list)
            for summary in release["records"]:
                if summary["kind"] != "Clean":
                    continue
                record = json.loads(
                    (
                        truth
                        / "records"
                        / summary["record_id"]
                        / "record.json"
                    ).read_text()
                )
                signature = [
                    tuple(
                        mode[field]
                        for field in (
                            "family_index_a",
                            "family_index_b",
                            "frequency_hz",
                            "decay_per_second",
                            "signed_gain",
                        )
                    )
                    for mode in record["modal"]["modes"]
                ]
                groups[summary["audio_sha256"]].append(
                    (summary["record_id"], signature)
                )
            aliases = [items for items in groups.values() if len(items) > 1]
            self.assertEqual(
                [[item[0] for item in group] for group in aliases],
                [
                    ["intervention-density", "intervention-uniform-scale"],
                    ["intervention-thickness", "intervention-youngs-modulus"],
                ],
            )
            self.assertTrue(
                all(all(item[1] == group[0][1] for item in group) for group in aliases)
            )

    def test_ignored_labels_do_not_change_candidate_view(self) -> None:
        profile, _ = v0.load_profile(V0_PROFILE)
        with tempfile.TemporaryDirectory(prefix="nextengine-v32-v0-label-") as temporary:
            truth = Path(temporary) / "truth"
            t0.run(T0_PROFILE, truth)
            record = json.loads(
                (
                    truth
                    / "records"
                    / "baseline-plate"
                    / "record.json"
                ).read_text()
            )
            forged = copy.deepcopy(record)
            forged.update(
                {
                    "expected": {"decision": "Forged"},
                    "invariant_checks": {"forged": False},
                    "kind": "Forged",
                    "mutation": {"operation": "forged"},
                    "source_case_id": "forged",
                }
            )
            self.assertEqual(
                v0.canonical_json(v0.candidate_view(record, profile)),
                v0.canonical_json(v0.candidate_view(forged, profile)),
            )


if __name__ == "__main__":
    unittest.main()
