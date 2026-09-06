from __future__ import annotations

import copy
import hashlib
import json
import math
import struct
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

import numpy as np

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v41_c0_disclosed_corpus_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v41-c0-disclosed-corpus.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v41_c0_disclosed_corpus_v1 as c0


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def binding(value: bytes) -> dict[str, object]:
    return {"bytes": len(value), "sha256": sha256(value)}


def wav_bytes(samples: np.ndarray, rate: int = 48000) -> bytes:
    quantized = np.rint(np.clip(samples, -1.0, 1.0) * 32767.0).astype("<i2")
    payload = quantized.tobytes()
    return b"".join(
        [
            b"RIFF",
            struct.pack("<I", 36 + len(payload)),
            b"WAVEfmt ",
            struct.pack("<IHHIIHH", 16, 1, 1, rate, rate * 2, 2, 16),
            b"data",
            struct.pack("<I", len(payload)),
            payload,
        ]
    )


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def synthetic_wave(frequency: float) -> np.ndarray:
    rate = c0.CORPUS_POLICY["sample_rate_hz"]
    time = np.arange(rate // 2, dtype=np.float64) / rate
    result = np.zeros_like(time)
    onset = 800
    tail = time[: len(time) - onset]
    result[onset:] = (
        0.7 * np.exp(-8.0 * tail) * np.sin(2.0 * math.pi * frequency * tail)
    )
    result[onset] += 0.9
    return result


def environment() -> dict[str, object]:
    result: dict[str, object] = {
        "numpy_version": np.__version__,
    }
    import scipy

    result["scipy_version"] = scipy.__version__
    for name, path in (
        ("ffmpeg", Path("/usr/bin/ffmpeg")),
        ("libarchive", Path("/usr/lib/x86_64-linux-gnu/libarchive.so.13")),
    ):
        data = path.read_bytes()
        result[name] = {"bytes": len(data), "path": str(path), "sha256": sha256(data)}
    return result


def build_fixture(root: Path) -> dict[str, Path]:
    roles = list(c0.ROLES)
    families = []
    mappings = []
    sources = []
    entries = []
    cache = root / "cache"
    for index, role in enumerate(roles):
        publisher = f"publisher-{index}"
        project = f"project-{index}"
        family_id = f"family-{index}"
        source_id = f"source-{index}"
        revision = f"revision-{index}"
        payload = wav_bytes(synthetic_wave(700.0 + index * 300.0))
        digest = sha256(payload)
        target = cache / "objects" / digest[:2] / digest
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(payload)
        families.append(
            {
                "family_component_id": f"component-{index}",
                "family_id": family_id,
                "protected": False,
                "role": role,
            }
        )
        mappings.append(
            {
                "family_id": family_id,
                "project_id": project,
                "publisher_id": publisher,
            }
        )
        sources.append(
            {
                "adapter_profile": {},
                "artifacts": [],
                "declared_revision": revision,
                "id": source_id,
                "project_id": project,
                "publisher_id": publisher,
            }
        )
        entries.append(
            {
                "audio_file_bytes": len(payload),
                "audio_file_sha256": digest,
                "declared_revision": revision,
                "entry_id": f"entry-{index}",
                "license_expression": "CC0-1.0",
                "material_label": ("Glass", "Wood", "Steel")[index],
                "object_group_id": f"object-group-{index}",
                "object_id": f"object-{index}",
                "project_id": project,
                "publisher_id": publisher,
                "recording_id": "001",
                "redistribution_policy": "external_research_only",
                "sample_rate_hz": 48000,
                "source_id": source_id,
            }
        )

    d1 = {
        "authority": {"authorizes_only": "v41_c0_disclosed_corpus"},
        "families": families,
        "roster_root_sha256": "1" * 64,
    }
    d1_bytes = c0.canonical_json(d1)
    d1_path = root / "d1.json"
    d1_path.write_bytes(d1_bytes)

    identified_manifest = {
        "schema": "nextengine.experimental-physical-sound-identified-corpus.manifest.v1"
    }
    source_manifest = {
        "schema": "nextengine.experimental-physical-sound-internet-sources.manifest.v1",
        "sources": sources,
    }
    identified_report = {
        "entries": entries,
        "recording_count": len(entries),
        "schema": "nextengine.experimental-physical-sound-identified-corpus.report.v1",
    }
    inputs = {}
    for name, value in (
        ("identified_manifest", identified_manifest),
        ("source_manifest", source_manifest),
        ("identified_report", identified_report),
    ):
        data = c0.canonical_json(value)
        path = root / f"{name}.json"
        path.write_bytes(data)
        inputs[name] = (path, binding(data))

    profile = {
        "authority": c0.AUTHORITY,
        "corpus_policy": c0.CORPUS_POLICY,
        "d1_binding": {
            **binding(d1_bytes),
            "roster_root_sha256": d1["roster_root_sha256"],
        },
        "dependency_bindings": [],
        "environment": environment(),
        "expected_counts": {
            "identified_recordings": 3,
            "physical_parents": 3,
            "realimpact_transfers": 0,
            "role_records": {role: 1 for role in roles},
            "total_records": 3,
        },
        "family_mapping": mappings,
        "feature_policy": c0.FEATURE_POLICY,
        "input_bindings": {name: value[1] for name, value in inputs.items()},
        "parent_aliases": [],
        "profile_id": "synthetic-c0-test",
        "realimpact_inputs": [],
        "schema": c0.PROFILE_SCHEMA,
    }
    profile_path = root / "profile.json"
    profile_path.write_bytes(c0.canonical_json(profile))
    return {
        "cache": cache,
        "d1": d1_path,
        "identified_manifest": inputs["identified_manifest"][0],
        "identified_report": inputs["identified_report"][0],
        "profile": profile_path,
        "source_manifest": inputs["source_manifest"][0],
    }


def run_fixture(paths: dict[str, Path], output: Path) -> Path:
    return c0.run(
        paths["profile"],
        paths["d1"],
        paths["identified_manifest"],
        paths["source_manifest"],
        paths["identified_report"],
        paths["cache"],
        [],
        output,
    )


class DisclosedCorpusTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_bound(self) -> None:
        data, profile = c0.read_canonical_profile(PROFILE)
        self.assertEqual(data, c0.canonical_json(profile))
        c0.validate_profile(profile)

    def test_segment_wav_and_acoustic_target_are_deterministic(self) -> None:
        source = synthetic_wave(1200.0)
        first, first_metadata = c0.canonical_segment(source)
        second, second_metadata = c0.canonical_segment(source.copy())
        self.assertTrue(np.array_equal(first, second))
        self.assertEqual(first_metadata, second_metadata)
        self.assertLessEqual(
            abs(int(np.argmax(np.abs(first))) - c0.CORPUS_POLICY["pretrigger_samples"]),
            c0.CORPUS_POLICY["onset_energy_window_samples"],
        )
        wav = c0.wav_pcm16_bytes(first)
        self.assertEqual(wav[:12], b"RIFF" + struct.pack("<I", len(wav) - 8) + b"WAVE")
        target = c0.extract_acoustic_target(first)
        target_repeat = c0.extract_acoustic_target(second)
        self.assertEqual(c0.canonical_json(target), c0.canonical_json(target_repeat))
        self.assertGreaterEqual(len(target["modes"]), 1)
        nearest = min(
            target["modes"], key=lambda row: abs(row["frequency_hz"] - 1200.0)
        )
        self.assertLess(abs(nearest["frequency_hz"] - 1200.0), 12.0)

    def test_zip_archive_member_is_exact_and_missing_member_rejects(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-c0-zip-") as temp:
            root = Path(temp)
            archive = root / "fixture.zip"
            with zipfile.ZipFile(archive, "w") as output:
                output.writestr("audio/hit.wav", b"exact-member")
            reader = c0.ArchiveReader(
                Path("/usr/lib/x86_64-linux-gnu/libarchive.so.13")
            )
            self.assertEqual(
                reader.extract(archive, {"audio/hit.wav"}),
                {"audio/hit.wav": b"exact-member"},
            )
            with self.assertRaisesRegex(c0.CorpusError, "members are absent"):
                reader.extract(archive, {"missing.wav"})

    def test_synthetic_full_run_is_repeat_exact_and_role_isolated(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-c0-run-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            run_fixture(paths, root / "run-a")
            run_fixture(paths, root / "run-b")
            self.assertEqual(
                artifact_tree(root / "run-a"), artifact_tree(root / "run-b")
            )
            report = json.loads((root / "run-a" / "report.json").read_text())
            self.assertEqual(report["decision"], c0.DECISION)
            self.assertTrue(all(report["gates"].values()))
            self.assertEqual(
                report["result"]["role_records"], {role: 1 for role in c0.ROLES}
            )
            self.assertEqual(report["result"]["content_object_count"], 6)
            self.assertEqual(len(artifact_tree(root / "run-a")), 14)

    def test_payload_mutation_rejects_without_publication(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-c0-payload-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            payload = next((paths["cache"] / "objects").rglob("*"))
            while payload.is_dir():
                payload = next(payload.rglob("*"))
            payload.write_bytes(payload.read_bytes() + b"mutation")
            output = root / "output"
            with self.assertRaisesRegex(c0.CorpusError, "bytes or SHA-256"):
                run_fixture(paths, output)
            self.assertFalse(output.exists())

    def test_cross_role_parent_or_component_rejects(self) -> None:
        base = {
            "axis_mask": {axis: False for axis in c0.AXES},
            "family_component_id": "shared-component",
            "physical_parent_id": "shared-parent",
        }
        records = [
            {**base, "role": "generator_train"},
            {**base, "role": "validator_calibration"},
        ]
        counts = {
            "physical_parents": 1,
            "role_records": {
                "generator_development": 0,
                "generator_train": 1,
                "validator_calibration": 1,
            },
        }
        with self.assertRaisesRegex(c0.CorpusError, "crosses C0 roles"):
            c0.validate_role_isolation(records, counts)

    def test_profile_authority_and_d1_mutations_reject(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-c0-mutation-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            profile = json.loads(paths["profile"].read_text())
            changed = copy.deepcopy(profile)
            changed["authority"]["model_training_authority"] = True
            with self.assertRaisesRegex(c0.CorpusError, "authority changed"):
                c0.validate_profile(changed)
            d1_mutation = paths["d1"].read_bytes() + b" "
            paths["d1"].write_bytes(d1_mutation)
            with self.assertRaisesRegex(c0.CorpusError, "bytes or SHA-256"):
                run_fixture(paths, root / "output")

    def test_atomic_publication_and_output_guards(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v41-c0-atomic-") as temp:
            root = Path(temp)
            output = root / "result"
            writes = 0
            original = Path.write_bytes

            def fail_second_write(path: Path, data: bytes) -> int:
                nonlocal writes
                writes += 1
                if writes == 2:
                    raise OSError("injected publication failure")
                return original(path, data)

            with (
                mock.patch.object(Path, "write_bytes", fail_second_write),
                self.assertRaisesRegex(OSError, "injected"),
            ):
                c0.publish_directory(output, {"a": b"1", "nested/b": b"2"})
            self.assertFalse(output.exists())
            self.assertEqual(list(root.iterdir()), [])
            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(c0.CorpusError, "refusing to replace"):
                c0.prepare_output(occupied)
            inside = ROOT / "target" / "v41-c0-forbidden"
            with self.assertRaisesRegex(c0.CorpusError, "outside"):
                c0.prepare_output(inside)

    def test_owner_has_no_network_or_model_dependency(self) -> None:
        source = SCRIPT.read_text()
        for forbidden in (
            "import requests",
            "import socket",
            "import torch",
            "import torchaudio",
            "import urllib",
            "from requests",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
