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

import physical_sound_v44_c1_runtime_descriptor_intake_v1 as c1  # noqa: E402
import physical_sound_v44_g0_structured_source_v1 as g0  # noqa: E402

PROFILE = ROOT / g0.PROFILE_PATH
SCRIPT = ROOT / g0.OWNER_PATH
C1_SUCCESSOR_PROFILE = (
    ROOT / "lab/profiles/physical-sound-v44-c1-runtime-descriptor-intake-g0-ycb.v1.json"
)

PLATE_ROWS = """
S 75 7708.30 1.05 0.98 1535.15 7.24 0.95 −138.84 −0.62 25.98 21.41 −6.00
S 150 7708.30 0.86 0.98 773.44 6.89 1.45 −45.61 −0.96 24.41 19.90 −5.90
S 300 7708.30 0.90 0.98 386.72 6.17 1.81 −25.04 −1.24 23.27 20.64 −2.85
S 600 7708.30 0.37 0.98 187.50 6.80 3.15 −11.96 −2.50 24.10 21.69 −2.63
S 1200 7708.30 0.27 0.98 93.75 5.84 3.88 −4.78 −2.93 23.83 20.35 −2.97
G 75 2301.70 1.52 0.52 1406.25 8.09 1.25 −153.39 −1.17 25.38 22.48 −7.70
G 150 2301.70 4.46 0.47 750.00 7.97 1.09 −175.29 −0.93 23.76 18.59 −31.70
G 300 2301.70 2.59 0.63 386.72 8.58 1.50 −105.47 −1.12 23.07 19.33 −5.72
G 600 2301.70 1.68 0.98 187.50 7.06 1.47 −42.14 −0.69 22.96 16.86 −7.26
G 1200 2301.70 2.55 0.94 105.47 6.59 1.34 −38.41 −0.54 22.56 17.20 −5.56
W 75 718.33 19.29 0.17 527.34 5.19 0.93 −175.51 −1.40 23.51 18.18 −171.61
W 150 718.33 22.33 0.19 257.81 4.55 0.95 −131.95 −2.03 22.34 16.44 −102.38
W 300 718.33 19.03 0.30 128.91 4.56 0.83 −121.13 −1.12 21.40 15.75 −44.01
W 600 718.33 19.78 0.16 58.60 4.15 1.07 −104.69 −3.41 20.98 16.69 −52.39
W 1200 718.33 17.55 0.23 23.44 4.06 1.05 −64.57 −2.64 21.00 16.10 −36.04
P 75 1413.30 26.09 0.10 527.34 4.78 1.11 −176.74 −3.99 23.26 18.36 −153.04
P 150 1413.30 39.62 0.10 281.25 4.05 1.15 −127.72 −4.52 22.11 17.25 −148.12
P 300 1413.30 41.03 0.13 140.63 3.83 1.10 −110.59 −3.19 21.33 16.84 −114.28
P 600 1413.30 31.03 0.16 70.31 3.71 0.91 −99.08 −2.50 20.98 16.46 −123.87
P 1200 1413.30 24.50 0.17 35.16 3.79 0.91 −84.09 −2.46 20.80 15.38 −85.39
""".strip()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def binding(path: str, data: bytes) -> dict[str, object]:
    return {"bytes": len(data), "path": path, "sha256": sha256(data)}


def write(path: Path, data: bytes) -> bytes:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return data


def projection(role: str, ids: list[int]) -> bytes:
    return g0.canonical_json(
        {
            "role": role,
            "rows": [
                {
                    "family_id": g0.YCB_FAMILY,
                    "physical_parent_id": f"{g0.YCB_PARENT_PREFIX}{object_id}",
                }
                for object_id in ids
            ],
            "schema": g0.PROJECTION_SCHEMA,
        }
    )


def fixture(root: Path) -> tuple[Path, Path, Path]:
    _, tracked = g0.read_canonical_json(PROFILE, "tracked profile")
    profile = copy.deepcopy(tracked)
    corpus = root / "corpus"
    raw = root / "raw"
    development = projection("generator_development", sorted(g0.EXPECTED_C0R_YCB_IDS))
    train = projection("generator_train", [])
    write(corpus / "development.json", development)
    write(corpus / "train.json", train)
    profile["input_bindings"] = {
        "development_projection": binding("development.json", development),
        "train_projection": binding("train.json", train),
    }
    ycb_pdf = b"%PDF-1.4 synthetic reviewed YCB paper\n"
    ycb_text = (
        "Table I: Object Set Items and Properties\nWine glass\nWood Block\n"
    ).encode()
    plate_pdf = b"%PDF-1.4 synthetic reviewed plate paper\n"
    plate_text = (
        "2-mm-thick square\n45 cm from the center\n"
        "TABLE I. Acoustical descriptors\n" + PLATE_ROWS + "\n"
    ).encode()
    raw_values = {
        "plate_paper_pdf": ("plate.pdf", plate_pdf),
        "plate_paper_text": ("plate.txt", plate_text),
        "ycb_paper_pdf": ("ycb.pdf", ycb_pdf),
        "ycb_paper_text": ("ycb.txt", ycb_text),
    }
    profile["raw_inputs"] = {}
    for name, (filename, data) in raw_values.items():
        write(raw / filename, data)
        profile["raw_inputs"][name] = binding(filename, data)
    profile_path = root / "profile.json"
    write(profile_path, g0.canonical_json(profile))
    return profile_path, corpus, raw


def directory_bytes(path: Path) -> dict[str, bytes]:
    return {item.name: item.read_bytes() for item in sorted(path.iterdir())}


class StructuredSourceTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_binds_owner(self) -> None:
        profile_bytes, profile = g0.read_canonical_json(PROFILE, "tracked profile")
        g0.validate_profile(profile)
        owner = next(
            item
            for item in profile["dependency_bindings"]
            if item["path"] == g0.OWNER_PATH
        )
        self.assertEqual(len(SCRIPT.read_bytes()), owner["bytes"])
        self.assertEqual(sha256(SCRIPT.read_bytes()), owner["sha256"])
        self.assertEqual(profile_bytes, g0.canonical_json(profile))

    def test_c1_successor_profile_is_canonical_and_accepts_one_source(self) -> None:
        profile_bytes, profile = c1.read_canonical_json(
            C1_SUCCESSOR_PROFILE, "C1 successor profile"
        )
        c1.validate_profile(profile)
        self.assertEqual(profile_bytes, c1.canonical_json(profile))
        self.assertEqual(
            ["ycb-object-properties"],
            [source["root_name"] for source in profile["source_inputs"]],
        )
        self.assertEqual(26, profile["expected_counts"]["source_observations"])

    def test_plate_table_builds_twenty_fixed_point_prospects(self) -> None:
        plates = g0.parse_plate_table(PLATE_ROWS)
        self.assertEqual(20, len(plates))
        self.assertEqual(15, sum(row["priority_material"] for row in plates))
        glass = next(row for row in plates if row["prospect_id"].endswith("g-0075"))
        self.assertEqual("Glass", glass["descriptor"]["material_label"])
        self.assertEqual(34526, glass["descriptor"]["mass_milligrams"])
        self.assertEqual(
            1520, glass["acoustic_targets_fixed_point"]["auditory_loss_factor_micro"]
        )

    def test_fixture_repeats_and_manifest_is_accepted_by_c1(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, corpus, raw = fixture(root)
            output_a = root / "output-a"
            output_b = root / "output-b"
            g0.run(profile_path, corpus, raw, output_a)
            g0.run(profile_path, corpus, raw, output_b)
            self.assertEqual(directory_bytes(output_a), directory_bytes(output_b))
            report = json.loads((output_a / "report.json").read_text())
            self.assertEqual(g0.DECISION, report["decision"])
            self.assertFalse(report["gates"]["psel_power_sufficient"])
            manifest_data = (output_a / "manifest.json").read_bytes()
            manifest = json.loads(manifest_data)
            object_61 = next(
                observation
                for observation in manifest["observations"]
                if observation["physical_parent_id"].endswith("object-61")
            )
            self.assertIn("source-table-060", object_61["observation_id"])
            source_input = {"manifest": binding("manifest.json", manifest_data)}
            _, contract = c1.read_canonical_json(
                ROOT
                / "lab/profiles/physical-sound-v44-c1-runtime-descriptor-contract.v1.json",
                "contract",
            )
            parent_records = {
                f"{g0.YCB_PARENT_PREFIX}{object_id}": set()
                for object_id in g0.YCB_OBJECTS
            }
            base_material = {parent: None for parent in parent_records}
            _, _, inventory, observations = c1.validate_source_manifest(
                "ycb-object-properties",
                output_a,
                source_input,
                contract,
                parent_records,
                base_material,
                set(),
            )
            self.assertEqual(26, inventory["observation_count"])
            self.assertEqual(26, len(observations))

    def test_bound_source_mutation_fails_without_publication(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, corpus, raw = fixture(root)
            with (raw / "plate.txt").open("ab") as stream:
                stream.write(b"mutation")
            output = root / "output"
            with self.assertRaisesRegex(g0.StructuredSourceError, "binding mismatch"):
                g0.run(profile_path, corpus, raw, output)
            self.assertFalse(output.exists())

    def test_c0r_parent_change_fails_without_publication(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, corpus, raw = fixture(root)
            profile = json.loads(profile_path.read_text())
            changed = projection(
                "generator_development", sorted(g0.EXPECTED_C0R_YCB_IDS - {71})
            )
            write(corpus / "development.json", changed)
            profile["input_bindings"]["development_projection"] = binding(
                "development.json", changed
            )
            write(profile_path, g0.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(
                g0.StructuredSourceError, "identity set changed"
            ):
                g0.run(profile_path, corpus, raw, output)
            self.assertFalse(output.exists())

    def test_output_guards_and_forbidden_imports(self) -> None:
        with self.assertRaisesRegex(g0.StructuredSourceError, "outside the repository"):
            g0.prepare_output(ROOT / "forbidden-g0-output")
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
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / "target"
            target.mkdir()
            alias = root / "alias"
            alias.symlink_to(target, target_is_directory=True)
            with self.assertRaisesRegex(g0.StructuredSourceError, "symlinks"):
                g0.prepare_output(alias / "output")


if __name__ == "__main__":
    unittest.main()
