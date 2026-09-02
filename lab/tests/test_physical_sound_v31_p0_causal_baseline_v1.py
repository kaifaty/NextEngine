from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v31_p0_causal_baseline_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v31-p0-causal-baseline.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v31_p0_causal_baseline_v1 as p0  # noqa: E402


class ProfileTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_text(encoding="utf-8"))

    def test_checked_in_profile_is_canonical_complete_and_zero_signal(self) -> None:
        profile, data = p0.load_profile(PROFILE)
        validated = p0.validate_profile(copy.deepcopy(profile))
        self.assertEqual(data, p0.canonical_json(profile))
        self.assertEqual(p0.sha256_bytes(data), p0.PROFILE_SHA256)
        self.assertEqual(
            [item["axis"] for item in validated["interventions"]],
            sorted(
                [
                    "youngs_modulus",
                    "density",
                    "thickness",
                    "uniform_scale",
                    "impulse",
                    "contact",
                    "support",
                ]
            ),
        )
        self.assertEqual(profile["authority"], p0.AUTHORITY)
        self.assertTrue(profile["fallback"]["authored_clip_class"])

    def test_bound_protocol_and_baseline_commit_exist(self) -> None:
        protocol = self.profile["protocol"]
        data = (ROOT / protocol["path"]).read_bytes()
        self.assertEqual(p0.sha256_bytes(data), protocol["sha256"])
        subprocess.run(
            ["git", "cat-file", "-e", f"{self.profile['baseline_commit']}^{{commit}}"],
            cwd=ROOT,
            check=True,
        )

    def test_unknown_authority_resource_and_formula_drift_are_rejected(self) -> None:
        cases: list[tuple[dict[str, object], str]] = []

        unknown = copy.deepcopy(self.profile)
        unknown["unexpected"] = True
        cases.append((unknown, "profile fields changed"))

        authority = copy.deepcopy(self.profile)
        authority["authority"]["runtime_authority"] = True
        cases.append((authority, "external-only authority changed"))

        resource = copy.deepcopy(self.profile)
        resource["resources"]["max_modes"] = 65
        cases.append((resource, "resource envelope changed"))

        formula = copy.deepcopy(self.profile)
        formula["formulae"][0]["formula_id"] = "tuned-after-values"
        cases.append((formula, "analytic formula table changed"))

        modal = copy.deepcopy(self.profile)
        modal["modal_output"]["mode_count"] = 11
        cases.append((modal, "modal output contract changed"))

        for value, message in cases:
            with self.subTest(message=message):
                with self.assertRaisesRegex(p0.CausalBaselineError, message):
                    p0.validate_profile(value)

    def test_missing_duplicate_or_nonisolated_intervention_is_rejected(self) -> None:
        missing = copy.deepcopy(self.profile)
        missing["interventions"].pop()
        with self.assertRaisesRegex(p0.CausalBaselineError, "exactly seven"):
            p0.validate_profile(missing)

        duplicate = copy.deepcopy(self.profile)
        duplicate["interventions"][-1] = copy.deepcopy(duplicate["interventions"][0])
        with self.assertRaisesRegex(p0.CausalBaselineError, "duplicate intervention"):
            p0.validate_profile(duplicate)

        simultaneous = copy.deepcopy(self.profile)
        simultaneous["interventions"][0]["also_changes"] = "density"
        with self.assertRaisesRegex(p0.CausalBaselineError, "fields changed"):
            p0.validate_profile(simultaneous)

    def test_wrong_frequency_support_or_contact_expectation_is_rejected(self) -> None:
        for axis, key, value in (
            ("density", "expected_frequency_ratio", "2"),
            ("support", "value", "free-free"),
        ):
            changed = copy.deepcopy(self.profile)
            item = next(entry for entry in changed["interventions"] if entry["axis"] == axis)
            item[key] = value
            with self.subTest(axis=axis):
                with self.assertRaisesRegex(p0.CausalBaselineError, "frozen isolated mutation"):
                    p0.validate_profile(changed)

        contact = copy.deepcopy(self.profile)
        item = next(entry for entry in contact["interventions"] if entry["axis"] == "contact")
        item["target"]["u"] = "0.49"
        with self.assertRaisesRegex(p0.CausalBaselineError, "frozen isolated mutation"):
            p0.validate_profile(contact)

    def test_profile_parser_rejects_duplicates_nonfinite_and_noncanonical_json(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)

            duplicate = root / "duplicate.json"
            duplicate.write_text('{"schema":"x","schema":"y"}\n', encoding="utf-8")
            with self.assertRaisesRegex(p0.CausalBaselineError, "duplicate JSON key"):
                p0.load_profile(duplicate)

            nonfinite = root / "nonfinite.json"
            nonfinite.write_text('{"value":NaN}\n', encoding="utf-8")
            with self.assertRaisesRegex(p0.CausalBaselineError, "non-finite JSON number"):
                p0.load_profile(nonfinite)

            noncanonical = root / "noncanonical.json"
            noncanonical.write_text(PROFILE.read_text(encoding="utf-8").rstrip(), encoding="utf-8")
            with self.assertRaisesRegex(p0.CausalBaselineError, "canonical JSON"):
                p0.load_profile(noncanonical)


class PublicationTests(unittest.TestCase):
    def test_full_owner_cli_repeats_byte_exactly_without_opening_values(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            outputs = [root / "run-a", root / "run-b"]
            stdout: list[bytes] = []
            for output in outputs:
                completed = subprocess.run(
                    [
                        sys.executable,
                        str(SCRIPT),
                        "--profile",
                        str(PROFILE),
                        "--output",
                        str(output),
                    ],
                    cwd=ROOT,
                    check=True,
                    capture_output=True,
                )
                stdout.append(completed.stdout)
                self.assertEqual(completed.stderr, b"")

            self.assertEqual(stdout[0], stdout[1])
            self.assertEqual(
                (outputs[0] / "contract.json").read_bytes(),
                (outputs[1] / "contract.json").read_bytes(),
            )
            self.assertEqual(
                (outputs[0] / "report.json").read_bytes(),
                (outputs[1] / "report.json").read_bytes(),
            )
            report = json.loads((outputs[0] / "report.json").read_text())
            contract = json.loads((outputs[0] / "contract.json").read_text())
            self.assertEqual(report["status"], "ProtocolFrozen")
            self.assertEqual(report["access"], p0.ZERO_ACCESS)
            self.assertEqual(report["next_authorized_stage"], "V31-P1-deterministic-modal-owner")
            self.assertEqual(
                contract["owner_identity"]["sha256"],
                p0.sha256_bytes(SCRIPT.read_bytes()),
            )
            self.assertFalse(any(outputs[0].glob("*.wav")))

    def test_output_must_be_fresh_external_and_not_a_symlink(self) -> None:
        with self.assertRaisesRegex(p0.CausalBaselineError, "fresh external"):
            p0.publish(PROFILE, ROOT / "p0-illegal-output")

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            existing = root / "existing"
            existing.mkdir()
            with self.assertRaisesRegex(p0.CausalBaselineError, "fresh external"):
                p0.publish(PROFILE, existing)

            target = root / "target"
            target.mkdir()
            link = root / "link"
            link.symlink_to(target, target_is_directory=True)
            with self.assertRaisesRegex(p0.CausalBaselineError, "symlink"):
                p0.publish(PROFILE, link)

    def test_protocol_hash_drift_rejects_before_publication(self) -> None:
        changed = json.loads(PROFILE.read_text(encoding="utf-8"))
        changed["protocol"]["sha256"] = "0" * 64
        with self.assertRaisesRegex(p0.CausalBaselineError, "protocol hash drift"):
            p0.validate_profile(changed)


if __name__ == "__main__":
    unittest.main()
