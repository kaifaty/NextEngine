from __future__ import annotations

import copy
import hashlib
import json
import math
import subprocess
import sys
import tempfile
import unittest
import wave
from array import array
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "lab" / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v44_g0b1a_iet_payload_probe_v1 as g0b1a  # noqa: E402

PROFILE = ROOT / g0b1a.PROFILE_PATH
SELECTION = ROOT / g0b1a.SELECTION_PATH
SCRIPT = ROOT / g0b1a.OWNER_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def binding(path: str, data: bytes) -> dict[str, object]:
    return {"bytes": len(data), "path": path, "sha256": sha256(data)}


def write(path: Path, data: bytes) -> bytes:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return data


def make_mp3(root: Path, sample_rate: int = 48000) -> bytes:
    wav_path = root / "source.wav"
    mp3_path = root / "source.mp3"
    sample_count = sample_rate * 2
    samples = array("h")
    for index in range(sample_count):
        if index < sample_rate // 2:
            value = 0.0
        else:
            time = (index - sample_rate // 2) / sample_rate
            value = math.exp(-5.0 * time) * (
                0.55 * math.sin(2.0 * math.pi * 730.0 * time)
                + 0.25 * math.sin(2.0 * math.pi * 1960.0 * time)
            )
        samples.append(round(max(-1.0, min(1.0, value)) * 32767.0))
    with wave.open(str(wav_path), "wb") as stream:
        stream.setnchannels(1)
        stream.setsampwidth(2)
        stream.setframerate(sample_rate)
        stream.writeframes(samples.tobytes())
    completed = subprocess.run(
        [
            "/usr/bin/ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-i",
            str(wav_path),
            "-map",
            "0:a:0",
            "-ac",
            "1",
            "-ar",
            str(sample_rate),
            "-c:a",
            "libmp3lame",
            "-b:a",
            "96k",
            str(mp3_path),
        ],
        check=False,
        capture_output=True,
        timeout=30,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.decode(errors="replace"))
    return mp3_path.read_bytes()


def fixture(
    root: Path, sample_rate: int = 48000
) -> tuple[Path, Path, Path, Path, bytes]:
    _, tracked_selection = g0b1a.read_canonical_json(SELECTION, "selection")
    _, tracked_profile = g0b1a.read_canonical_json(PROFILE, "profile")
    selection = copy.deepcopy(tracked_selection)
    profile = copy.deepcopy(tracked_profile)
    payload = make_mp3(root, sample_rate)
    digest = sha256(payload)
    manifest = {
        "claim": g0b1a.PROSPECTIVE_CLAIM,
        "parents": [
            {
                "descriptor": {"values": ["Steel"]},
                "lineage": {
                    "physical_parent_id": (
                        "mendeley-data--10.17632-srfp7x6wxm--v1--sample-fixture"
                    ),
                    "project_id": selection["project"]["project_id"],
                    "revision_id": selection["project"]["revision_id"],
                    "source_component_id": selection["project"]["source_component_id"],
                },
                "prospective_catalogue_eligible": True,
                "sample_id": "fixture",
                "waveforms": [
                    {
                        "bytes": len(payload),
                        "media_type": "audio/mpeg",
                        "publisher_file_id": "fixture-file",
                        "record_id": "identified:ieteasy-fixture--impact-001",
                        "repetition_index": 1,
                        "sha256": digest,
                        "source_url": "https://data.mendeley.com/fixture",
                    }
                ],
            }
        ],
        "schema": g0b1a.PROSPECTIVE_SCHEMA,
    }
    manifest_bytes = g0b1a.canonical_json(manifest)
    manifest_path = root / "prospective.json"
    write(manifest_path, manifest_bytes)
    selection["expected_selection"] = {
        "material_counts": {"Steel": 1},
        "physical_parents": 1,
        "project_revisions": 1,
        "selected_payload_bytes": len(payload),
        "selected_waveforms": 1,
    }
    selection["input_binding"] = {
        "bytes": len(manifest_bytes),
        "external_relative_path": "fixture/prospective.json",
        "schema": g0b1a.PROSPECTIVE_SCHEMA,
        "sha256": sha256(manifest_bytes),
    }
    selection_bytes = g0b1a.canonical_json(selection)
    selection_path = root / "selection.json"
    write(selection_path, selection_bytes)
    profile["expected_result"] = {
        "decoded_waveforms": 1,
        "physical_parents": 1,
        "project_revisions": 1,
        "selected_payload_bytes": len(payload),
        "steel_parents": 1,
        "target_probes": 1,
    }
    profile["selection_profile"] = binding(g0b1a.SELECTION_PATH, selection_bytes)
    profile_path = root / "profile.json"
    write(profile_path, g0b1a.canonical_json(profile))
    payload_root = root / "external-payload"
    write(payload_root / digest, payload)
    return profile_path, selection_path, manifest_path, payload_root, payload


def directory_bytes(path: Path) -> dict[str, bytes]:
    return {item.name: item.read_bytes() for item in sorted(path.iterdir())}


class IeteasyPayloadProbeTests(unittest.TestCase):
    def test_tracked_profiles_are_canonical_and_bind_owner(self) -> None:
        selection_bytes, selection = g0b1a.read_canonical_json(SELECTION, "selection")
        profile_bytes, profile = g0b1a.read_canonical_json(PROFILE, "profile")
        g0b1a.validate_execution_profile(profile)
        g0b1a.validate_selection_profile(selection)
        self.assertEqual(selection_bytes, g0b1a.canonical_json(selection))
        self.assertEqual(profile_bytes, g0b1a.canonical_json(profile))
        owner = next(
            item
            for item in profile["dependency_bindings"]
            if item["path"] == g0b1a.OWNER_PATH
        )
        self.assertEqual(SCRIPT.stat().st_size, owner["bytes"])
        self.assertEqual(sha256(SCRIPT.read_bytes()), owner["sha256"])

    def test_fixture_repeats_with_hash_media_onset_and_target_probe(self) -> None:
        with tempfile.TemporaryDirectory(dir="/tmp") as temporary:
            root = Path(temporary)
            profile, selection, manifest, payload, _ = fixture(root)
            output_a = root / "output-a"
            output_b = root / "output-b"
            g0b1a.run(profile, selection, manifest, payload, output_a)
            g0b1a.run(profile, selection, manifest, payload, output_b)
            self.assertEqual(directory_bytes(output_a), directory_bytes(output_b))
            report = json.loads((output_a / "report.json").read_text())
            self.assertEqual(g0b1a.DECISION, report["decision"])
            self.assertTrue(report["gates"]["media_and_onset_probe_passed"])
            self.assertFalse(report["gates"]["corpus_materialization_eligible"])
            self.assertFalse(
                report["gates"]["target_extractor_reuse_without_audit_allowed"]
            )
            media = json.loads((output_a / "media-probe.json").read_text())
            row = media["records"][0]
            self.assertEqual(48000, row["media"]["sample_rate_hz"])
            self.assertEqual(1, row["media"]["channel_count"])
            self.assertGreater(row["normalization"]["source_onset_sample_48k"], 0)
            self.assertEqual(g0b1a.c0.FEATURE_SCHEMA, row["target_probe"]["schema"])
            access = json.loads((output_a / "access-ledger.json").read_text())
            for counter in g0b1a.FORBIDDEN_COUNTERS:
                self.assertEqual(0, access["counters"][counter])

    def test_payload_hash_mutation_rejects_without_publication(self) -> None:
        with tempfile.TemporaryDirectory(dir="/tmp") as temporary:
            root = Path(temporary)
            profile, selection, manifest, payload_root, payload = fixture(root)
            digest = sha256(payload)
            write(payload_root / digest, payload[:-1] + bytes([payload[-1] ^ 1]))
            output = root / "output"
            with self.assertRaisesRegex(g0b1a.PayloadProbeError, "byte/hash"):
                g0b1a.run(profile, selection, manifest, payload_root, output)
            self.assertFalse(output.exists())

    def test_wrong_source_rate_rejects_without_publication(self) -> None:
        with tempfile.TemporaryDirectory(dir="/tmp") as temporary:
            root = Path(temporary)
            profile, selection, manifest, payload, _ = fixture(root, 44100)
            output = root / "output"
            with self.assertRaisesRegex(g0b1a.PayloadProbeError, "channel/rate"):
                g0b1a.run(profile, selection, manifest, payload, output)
            self.assertFalse(output.exists())

    def test_duplicate_repetition_one_rejects_before_payload_read(self) -> None:
        with tempfile.TemporaryDirectory(dir="/tmp") as temporary:
            root = Path(temporary)
            profile_path, selection_path, manifest_path, payload, _ = fixture(root)
            manifest = json.loads(manifest_path.read_text())
            duplicate = copy.deepcopy(manifest["parents"][0]["waveforms"][0])
            duplicate["record_id"] += "-duplicate"
            manifest["parents"][0]["waveforms"].append(duplicate)
            manifest_bytes = g0b1a.canonical_json(manifest)
            write(manifest_path, manifest_bytes)
            selection = json.loads(selection_path.read_text())
            selection["input_binding"]["bytes"] = len(manifest_bytes)
            selection["input_binding"]["sha256"] = sha256(manifest_bytes)
            selection_bytes = g0b1a.canonical_json(selection)
            write(selection_path, selection_bytes)
            profile = json.loads(profile_path.read_text())
            profile["selection_profile"] = binding(
                g0b1a.SELECTION_PATH, selection_bytes
            )
            write(profile_path, g0b1a.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(g0b1a.PayloadProbeError, "exactly one"):
                g0b1a.run(profile_path, selection_path, manifest_path, payload, output)
            self.assertFalse(output.exists())

    def test_output_payload_root_and_network_import_guards(self) -> None:
        with self.assertRaisesRegex(g0b1a.PayloadProbeError, "outside"):
            g0b1a.prepare_output(ROOT / "forbidden-g0b1a-output")
        with tempfile.TemporaryDirectory(dir=ROOT) as temporary:
            root = Path(temporary)
            profile, selection, manifest, _, payload = fixture(root)
            payload_root = root / "payload-inside-repository"
            write(payload_root / sha256(payload), payload)
            output = Path("/tmp") / f"g0b1a-{root.name}"
            with self.assertRaisesRegex(g0b1a.PayloadProbeError, "external"):
                g0b1a.run(profile, selection, manifest, payload_root, output)
            self.assertFalse(output.exists())
        source = SCRIPT.read_text()
        for forbidden in (
            "import requests",
            "import torch",
            "import urllib",
            "from urllib",
        ):
            self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
