from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "lab" / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v44_g0b_iet_prospective_parent_v1 as g0b  # noqa: E402

PROFILE = ROOT / g0b.PROFILE_PATH
CONTRACT = ROOT / g0b.CONTRACT_PATH
SCRIPT = ROOT / g0b.OWNER_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def binding(path: str, data: bytes) -> dict[str, object]:
    return {"bytes": len(data), "path": path, "sha256": sha256(data)}


def write(path: Path, data: bytes) -> bytes:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return data


def bioc(text: str) -> bytes:
    return json.dumps(
        [
            {
                "documents": [
                    {
                        "passages": [{"text": text}],
                    }
                ]
            }
        ]
    ).encode()


def projection(parents: list[str]) -> bytes:
    return g0b.canonical_json(
        {
            "rows": [
                {
                    "family_component_id": f"fixture-component-{index % 6}",
                    "physical_parent_id": parent,
                }
                for index, parent in enumerate(parents)
            ],
            "schema": g0b.PROJECTION_SCHEMA,
        }
    )


def file_list(bridge: dict[str, str]) -> bytes:
    rows = []
    for repetition in range(1, 11):
        file_id = str(
            uuid.uuid5(
                uuid.NAMESPACE_URL,
                f"{bridge['sample_id']}:{repetition}",
            )
        )
        digest = sha256(f"payload:{bridge['sample_id']}:{repetition}".encode())
        size = 119877
        if bridge["sample_id"] == "6082-aluminium" and repetition == 1:
            size += 66
        rows.append(
            {
                "content_details": {
                    "content_type": "audio/mpeg",
                    "download_url": (
                        "https://data.mendeley.com/public-files/datasets/"
                        f"srfp7x6wxm/files/{file_id}/file_downloaded"
                    ),
                    "sha256_hash": digest,
                    "size": size,
                },
                "filename": f"{bridge['filename_stem']} ({repetition}).mp3",
                "folder_id": bridge["folder_id"],
                "id": file_id,
                "size": size,
                "status": "COMPLETED",
            }
        )
    return json.dumps(rows).encode()


def fixture(root: Path) -> tuple[Path, Path, Path]:
    _, tracked = g0b.read_canonical_json(PROFILE, "tracked profile")
    profile = copy.deepcopy(tracked)
    corpus = root / "corpus"
    raw = root / "raw"

    parents = [f"fixture-parent-{index:03d}" for index in range(64)]
    train = projection(parents[:34])
    development = projection(parents[34:])
    write(corpus / "train.json", train)
    write(corpus / "development.json", development)
    profile["input_bindings"] = {
        "development_projection": binding("development.json", development),
        "train_projection": binding("train.json", train),
    }

    page = (
        "10.17632/srfp7x6wxm.1 CC BY 4.0 original audio file dimensions, weight"
    ).encode()
    data_article = bioc(
        "fifteen different materials ten measurements for each sample "
        "suspending the sample on PA6 strings Length (cm) Mass (g) "
        "sampling rate of 48,000"
    )
    hardware_article = bioc(
        "rectangular samples Free-Free boundary conditions two taut nylon strings "
        "mallet is released microphone positioned over the sample"
    )
    bridges = profile["sample_folder_bridge"]
    folder_index = json.dumps(
        [
            {"id": g0b.AUDIO_ROOT_ID, "name": "Audio data"},
            *[
                {
                    "id": bridge["folder_id"],
                    "name": bridge["folder_label"],
                    "parent_id": g0b.AUDIO_ROOT_ID,
                }
                for bridge in bridges
            ],
        ]
    ).encode()
    raw_values = {
        "data_article": ("data-article.json", data_article),
        "dataset_page": ("dataset-page.html", page),
        "folder_index": ("folders.json", folder_index),
        "hardware_article": ("hardware-article.json", hardware_article),
    }
    profile["raw_inputs"] = {}
    for name, (path, data) in raw_values.items():
        write(raw / path, data)
        profile["raw_inputs"][name] = binding(path, data)
    profile["raw_inputs"]["audio_folders"] = []
    for bridge in bridges:
        data = file_list(bridge)
        path = f"audio-files/{bridge['folder_id']}.json"
        write(raw / path, data)
        profile["raw_inputs"]["audio_folders"].append(
            {
                "binding": binding(path, data),
                "folder_id": bridge["folder_id"],
            }
        )

    profile_path = root / "profile.json"
    write(profile_path, g0b.canonical_json(profile))
    return profile_path, corpus, raw


def directory_bytes(path: Path) -> dict[str, bytes]:
    return {item.name: item.read_bytes() for item in sorted(path.iterdir())}


class IeteasyProspectiveParentTests(unittest.TestCase):
    def test_tracked_contract_and_profile_are_canonical_and_bind_owner(self) -> None:
        contract_bytes, contract = g0b.read_canonical_json(CONTRACT, "contract")
        profile_bytes, profile = g0b.read_canonical_json(PROFILE, "profile")
        g0b.validate_profile(profile)
        self.assertEqual(g0b.CONTRACT_SCHEMA, contract["schema"])
        self.assertEqual(contract_bytes, g0b.canonical_json(contract))
        self.assertEqual(profile_bytes, g0b.canonical_json(profile))
        owner = next(
            item
            for item in profile["dependency_bindings"]
            if item["path"] == g0b.OWNER_PATH
        )
        self.assertEqual(len(SCRIPT.read_bytes()), owner["bytes"])
        self.assertEqual(sha256(SCRIPT.read_bytes()), owner["sha256"])

    def test_fixture_repeats_without_payload_access(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile, corpus, raw = fixture(root)
            output_a = root / "output-a"
            output_b = root / "output-b"
            g0b.run(profile, corpus, raw, output_a)
            g0b.run(profile, corpus, raw, output_b)
            self.assertEqual(directory_bytes(output_a), directory_bytes(output_b))
            report = json.loads((output_a / "report.json").read_text())
            self.assertEqual(g0b.DECISION, report["decision"])
            self.assertEqual(15, report["measured"]["prospective_parents"])
            self.assertEqual(150, report["measured"]["publisher_hashed_waveforms"])
            access = json.loads((output_a / "access-ledger.json").read_text())
            for counter in g0b.FORBIDDEN_COUNTERS:
                self.assertEqual(0, access["counters"][counter])

    def test_descriptors_are_bounded_and_psel_stays_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile, corpus, raw = fixture(root)
            output = root / "output"
            g0b.run(profile, corpus, raw, output)
            manifest = json.loads((output / "prospective-parents.json").read_text())
            steel = next(
                parent
                for parent in manifest["parents"]
                if parent["sample_id"] == "aisi-304-steel"
            )
            self.assertEqual([99000, 100500, 8200], steel["descriptor"]["values"][2])
            self.assertEqual(636000, steel["descriptor"]["values"][6])
            self.assertEqual("suspended", steel["descriptor"]["values"][7])
            self.assertIsNone(steel["descriptor"]["values"][8])
            power = json.loads((output / "power.json").read_text())
            self.assertEqual([], power["eligible_psel_packs"])
            self.assertEqual(5, power["target_pack_parent_counts"]["Steel"])
            self.assertEqual(1, power["target_pack_project_counts"]["Steel"])

    def test_bound_source_mutation_fails_without_publication(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile, corpus, raw = fixture(root)
            target = next((raw / "audio-files").iterdir())
            target.write_bytes(target.read_bytes() + b"mutation")
            output = root / "output"
            with self.assertRaisesRegex(g0b.ProspectiveParentError, "binding mismatch"):
                g0b.run(profile, corpus, raw, output)
            self.assertFalse(output.exists())

    def test_duplicate_waveform_hash_rejects_even_when_rebound(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, corpus, raw = fixture(root)
            profile = json.loads(profile_path.read_text())
            folders = profile["raw_inputs"]["audio_folders"]
            first_path = raw / folders[0]["binding"]["path"]
            second_path = raw / folders[1]["binding"]["path"]
            first = json.loads(first_path.read_text())
            second = json.loads(second_path.read_text())
            second[0]["content_details"]["sha256_hash"] = first[0]["content_details"][
                "sha256_hash"
            ]
            changed = json.dumps(second).encode()
            write(second_path, changed)
            folders[1]["binding"] = binding(folders[1]["binding"]["path"], changed)
            write(profile_path, g0b.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(
                g0b.ProspectiveParentError, "waveform hash appears under two records"
            ):
                g0b.run(profile_path, corpus, raw, output)
            self.assertFalse(output.exists())

    def test_material_relabel_rejects(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, corpus, raw = fixture(root)
            profile = json.loads(profile_path.read_text())
            steel = next(
                row
                for row in profile["sample_folder_bridge"]
                if row["sample_id"] == "aisi-304-steel"
            )
            steel["normalized_material_label"] = "Plastic"
            write(profile_path, g0b.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(
                g0b.ProspectiveParentError, "material normalization changed"
            ):
                g0b.run(profile_path, corpus, raw, output)
            self.assertFalse(output.exists())

    def test_c0r_parent_count_change_rejects(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile_path, corpus, raw = fixture(root)
            profile = json.loads(profile_path.read_text())
            development = projection([f"changed-parent-{index}" for index in range(29)])
            write(corpus / "development.json", development)
            profile["input_bindings"]["development_projection"] = binding(
                "development.json", development
            )
            write(profile_path, g0b.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(
                g0b.ProspectiveParentError, "current C0R parent count changed"
            ):
                g0b.run(profile_path, corpus, raw, output)
            self.assertFalse(output.exists())

    def test_output_guards_and_forbidden_imports(self) -> None:
        with self.assertRaisesRegex(
            g0b.ProspectiveParentError, "outside the repository"
        ):
            g0b.prepare_output(ROOT / "forbidden-g0b-output")
        with tempfile.TemporaryDirectory() as temporary:
            occupied = Path(temporary) / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(g0b.ProspectiveParentError, "already exists"):
                g0b.prepare_output(occupied)
        source = SCRIPT.read_text()
        for forbidden in (
            "import librosa",
            "import numpy",
            "import requests",
            "import soundfile",
            "import torch",
            "import urllib.request",
            "import wave",
        ):
            self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
