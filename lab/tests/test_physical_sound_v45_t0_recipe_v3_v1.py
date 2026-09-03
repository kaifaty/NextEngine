from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "lab" / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v45_t0_recipe_v3_v1 as t0  # noqa: E402

PROFILE = ROOT / t0.PROFILE_PATH
SCRIPT = ROOT / t0.OWNER_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_profile() -> dict[str, object]:
    return json.loads(PROFILE.read_text())


def fixture(profile: dict[str, object], fixture_id: str) -> dict[str, object]:
    return next(item for item in profile["fixtures"] if item["fixture_id"] == fixture_id)


def output_bytes(path: Path) -> dict[str, bytes]:
    return {item.name: item.read_bytes() for item in sorted(path.iterdir())}


class RecipeV3Tests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = load_profile()
        self.stress = fixture(self.profile, "projection-stress-control")
        self.missing = fixture(self.profile, "missing-label-control")

    def test_profile_is_canonical_and_binds_owner_and_r0(self) -> None:
        profile_bytes, profile = t0.read_canonical_json(PROFILE, "profile")
        fixtures = t0.validate_profile(profile)
        self.assertEqual(profile_bytes, t0.canonical_json(profile))
        self.assertEqual(len(fixtures), 2)
        bindings = {item["path"]: item for item in profile["dependency_bindings"]}
        self.assertEqual(bindings[t0.OWNER_PATH]["bytes"], SCRIPT.stat().st_size)
        self.assertEqual(bindings[t0.OWNER_PATH]["sha256"], sha256(SCRIPT.read_bytes()))
        self.assertIn(
            "docs/development/physical-sound-v45-r0-source-claim-ledger-result-2026-09-03.md",
            bindings,
        )

    def test_two_runs_are_byte_identical_and_report_zero_access(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            left = root / "left"
            right = root / "right"
            t0.run(PROFILE, left)
            t0.run(PROFILE, right)
            self.assertEqual(output_bytes(left), output_bytes(right))
            access = json.loads((left / "access.json").read_text())
            report = json.loads((left / "report.json").read_text())
            self.assertTrue(all(value == 0 for value in access["counters"].values()))
            self.assertTrue(all(report["gates"].values()))
            self.assertEqual(report["measured"]["target_dimension"], 189)

    def test_projection_orders_modes_and_enforces_bounds(self) -> None:
        projected = t0.project_recipe(self.stress["raw_recipe"])
        modal = projected["heads"]["modal"]
        excitation = projected["heads"]["excitation"]
        radiation = projected["heads"]["radiation"]
        self.assertEqual(modal["frequency_ratio_ppm"], [1_000_000, 2_000_000, 3_000_000])
        self.assertEqual(modal["rt60_milliseconds"], [5, 60_000, 300])
        self.assertTrue(all(value > 0 for value in modal["rt60_milliseconds"]))
        self.assertEqual(sum(modal["participation_ppm"]), 1_000_000)
        self.assertEqual(sum(radiation["band_energy_ppm"]), 1_000_000)
        self.assertEqual(sum(radiation["residual_envelope_ppm"]), 1_000_000)
        self.assertEqual(excitation["contact_duration_samples"], 960)
        self.assertEqual(excitation["onset_samples"], 4_095)
        self.assertEqual(excitation["impact_gain_ppm"], 2_000_000)
        base = modal["base_frequency_millihz"]
        frequencies = [base * ratio // 1_000_000 for ratio in modal["frequency_ratio_ppm"]]
        self.assertEqual(frequencies, sorted(set(frequencies)))
        self.assertTrue(all(value < t0.NYQUIST_MILLIHZ_EXCLUSIVE for value in frequencies))

    def test_fixed_vector_roundtrip_and_zero_padding(self) -> None:
        projected = t0.project_recipe(self.stress["raw_recipe"])
        vector = t0.flatten_recipe(projected)
        rebuilt = t0.recipe_from_vector(
            vector,
            descriptor_mask=projected["descriptor_mask"],
            recipe_id=projected["recipe_id"],
            source_lane=projected["source_lane"],
        )
        self.assertEqual(rebuilt, projected)
        self.assertEqual(len(vector), t0.TARGET_DIMENSION)
        count = projected["heads"]["modal"]["mode_count"]
        for offset in (2, 34, 66, 98):
            self.assertEqual(vector[offset + count : offset + t0.MAX_MODES], [0] * (t0.MAX_MODES - count))

    def test_missing_labels_contribute_no_loss(self) -> None:
        projected = t0.project_recipe(self.missing["raw_recipe"])
        target = t0.make_target(projected, self.missing["observed_fields"])
        prediction = [987_654_321] * t0.TARGET_DIMENSION
        self.assertEqual(t0.masked_squared_error(prediction, target), {"observed_coordinates": 0, "squared_error": 0})
        self.assertEqual(target["values"], [0] * t0.TARGET_DIMENSION)
        self.assertEqual(target["mask"], [0] * t0.TARGET_DIMENSION)

    def test_lane_masks_cannot_escalate_or_train_on_validator_roles(self) -> None:
        structural = copy.deepcopy(self.stress["raw_recipe"])
        structural["source_lane"] = "structural_transfer"
        projected = t0.project_recipe(structural)
        with self.assertRaisesRegex(t0.RecipeContractError, "exceeds its source lane"):
            t0.make_target(projected, ["radiation.band_energy_ppm"])
        validator = copy.deepcopy(self.stress["raw_recipe"])
        validator["source_lane"] = "validator_calibration"
        projected = t0.project_recipe(validator)
        with self.assertRaisesRegex(t0.RecipeContractError, "cannot create generator targets"):
            t0.make_target(projected, ["support.ood_score_ppm"])

    def test_target_corruption_and_non_integer_values_fail_closed(self) -> None:
        projected = t0.project_recipe(self.stress["raw_recipe"])
        target = t0.make_target(projected, self.stress["observed_fields"])
        changed = copy.deepcopy(target)
        changed["mask"][188] = 0
        with self.assertRaisesRegex(t0.RecipeContractError, "mask does not match"):
            t0.validate_target(changed)
        changed = copy.deepcopy(target)
        changed["values"][0] = True
        with self.assertRaisesRegex(t0.RecipeContractError, "must be an integer"):
            t0.validate_target(changed)
        changed = copy.deepcopy(target)
        missing_index = changed["mask"].index(0)
        changed["values"][missing_index] = 1
        with self.assertRaisesRegex(t0.RecipeContractError, "must be zero"):
            t0.validate_target(changed)

    def test_resource_descriptor_and_profile_corruptions_fail_closed(self) -> None:
        changed = copy.deepcopy(self.stress["raw_recipe"])
        changed["heads"]["modal"]["mode_count"] = t0.MAX_MODES + 1
        with self.assertRaisesRegex(t0.RecipeContractError, "resource envelope"):
            t0.project_recipe(changed)
        changed = copy.deepcopy(self.stress["raw_recipe"])
        changed["descriptor_mask"].append("unknown")
        changed["descriptor_mask"].sort()
        with self.assertRaisesRegex(t0.RecipeContractError, "unknown descriptor"):
            t0.project_recipe(changed)
        changed_profile = copy.deepcopy(self.profile)
        changed_profile["unexpected"] = True
        with self.assertRaisesRegex(t0.RecipeContractError, "profile fields changed"):
            t0.validate_profile(changed_profile)

    def test_output_and_import_guards(self) -> None:
        with self.assertRaisesRegex(t0.RecipeContractError, "outside"):
            t0.prepare_output(ROOT / "forbidden-v45-t0-output")
        source = SCRIPT.read_text()
        for forbidden in (
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
