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

import physical_sound_v46_d0_source_preflight_v1 as d0  # noqa: E402

PROFILE = ROOT / d0.PROFILE_PATH
OWNER = ROOT / d0.OWNER_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_profile() -> dict[str, object]:
    return json.loads(PROFILE.read_text())


def write(path: Path, data: bytes) -> None:
    path.write_bytes(data)


def revision_bytes(source: dict[str, object], paths: list[str]) -> bytes:
    return json.dumps(
        {
            "gated": False,
            "id": source["repository_id"],
            "private": False,
            "sha": source["revision"],
            "siblings": [{"rfilename": path} for path in paths],
            "tags": source["expected"]["stable_tags"],
        }
    ).encode()


def root_tree_bytes(paths: list[str]) -> bytes:
    return json.dumps(
        [
            {
                "oid": f"{index + 1:040x}",
                "path": path,
                "size": index,
                "type": "file",
            }
            for index, path in enumerate(paths)
        ]
    ).encode()


def artificial_source(
    source_id: str, directory: Path
) -> tuple[dict[str, object], list[str]]:
    source = copy.deepcopy(
        next(item for item in load_profile()["sources"] if item["source_id"] == source_id)
    )
    if source_id == "nisr-v5":
        sibling_paths = ["1/feat/feat_Glass.npz", "1/voxel.npz", "README.md"]
        root_paths = ["1", "README.md"]
        readme = b"declared pipeline and broken generation document"
        headers = b"HTTP/2 404\r\ncontent-type: text/html\r\n\r\n"
        write(directory / source["metadata_files"]["generation_link_headers"], headers)
        source["expected"]["generation_link_http_status"] = 404
    else:
        sibling_paths = [
            ".gitattributes",
            "README.md",
            "generate/dataset_shard_0000.tar",
            "material_idx_to_params.py",
            "objaverse/dataset_shard_0000.tar",
        ]
        root_paths = [
            ".gitattributes",
            "README.md",
            "generate",
            "material_idx_to_params.py",
            "objaverse",
        ]
        readme = b"high-level geometry modal acoustic chain"
        material = b"MATERIALS = {}\n"
        write(directory / source["metadata_files"]["material_parameters"], material)
        source["expected"]["material_parameters_sha256"] = sha256(material)
    source["expected"]["readme_required_literals"] = [readme.decode()]
    source["expected"]["readme_sha256"] = sha256(readme)
    source["expected"]["root_entry_count"] = len(root_paths)
    source["expected"]["sibling_count"] = len(sibling_paths)
    source["expected"]["sibling_path_root_sha256"] = d0.sibling_path_root(
        sibling_paths
    )
    write(directory / source["metadata_files"]["readme"], readme)
    write(
        directory / source["metadata_files"]["revision"],
        revision_bytes(source, sibling_paths),
    )
    write(
        directory / source["metadata_files"]["root_tree"],
        root_tree_bytes(root_paths),
    )
    return source, sibling_paths


class SourcePreflightTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_binds_dependencies(self) -> None:
        profile_bytes, profile = d0.read_canonical_json(PROFILE, "profile")
        sources = d0.validate_profile(profile)
        self.assertEqual(profile_bytes, d0.canonical_json(profile))
        self.assertEqual(["nisr-v5", "vibraverse"], [item["source_id"] for item in sources])
        bindings = {item["path"]: item for item in profile["dependency_bindings"]}
        for path in (
            d0.OWNER_PATH,
            d0.PROTOCOL_PATH,
            d0.R0_PROFILE_PATH,
            d0.T0_PROFILE_PATH,
        ):
            data = (ROOT / path).read_bytes()
            self.assertEqual(len(data), bindings[path]["bytes"])
            self.assertEqual(sha256(data), bindings[path]["sha256"])

    def test_nisr_broken_declared_generation_document_fails_before_sample(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source, _ = artificial_source("nisr-v5", Path(temporary))
            report, _ = d0.evaluate_source(source, Path(temporary))
        self.assertEqual(d0.UNTRUSTED_DECISION, report["decision"])
        self.assertEqual("DeclaredGenerationDocumentUnavailable", report["reason"])
        self.assertEqual(404, report["observations"]["declared_generation_link_http_status"])
        self.assertFalse(report["sample"]["accessed"])
        self.assertTrue(report["gates"]["dataset_payload_remained_closed"])

    def test_vibraverse_shards_and_material_table_are_not_generation_lineage(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source, sibling_paths = artificial_source("vibraverse", Path(temporary))
            report, _ = d0.evaluate_source(source, Path(temporary))
        self.assertEqual(d0.UNTRUSTED_DECISION, report["decision"])
        self.assertEqual("GenerationLineageIncomplete", report["reason"])
        self.assertEqual(2, report["observations"]["data_shard_count"])
        self.assertEqual(d0.sibling_path_root(sibling_paths), report["observations"]["sibling_path_root_sha256"])
        self.assertFalse(report["sample"]["accessed"])

    def test_revision_or_inventory_drift_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            source, sibling_paths = artificial_source("nisr-v5", directory)
            changed = json.loads(
                (directory / source["metadata_files"]["revision"]).read_text()
            )
            changed["sha"] = "0" * 40
            write(
                directory / source["metadata_files"]["revision"],
                json.dumps(changed).encode(),
            )
            with self.assertRaisesRegex(d0.SourcePreflightError, "identity changed"):
                d0.evaluate_source(source, directory)

            changed["sha"] = source["revision"]
            changed["siblings"] = [
                {"rfilename": path} for path in sibling_paths + ["unexpected.bin"]
            ]
            write(
                directory / source["metadata_files"]["revision"],
                json.dumps(changed).encode(),
            )
            with self.assertRaisesRegex(d0.SourcePreflightError, "sibling count"):
                d0.evaluate_source(source, directory)

    def test_profile_mutation_and_repository_output_fail_closed(self) -> None:
        profile = load_profile()
        changed = copy.deepcopy(profile)
        changed["access_policy"]["bulk_download_allowed"] = True
        with self.assertRaisesRegex(d0.SourcePreflightError, "access policy"):
            d0.validate_profile(changed)
        changed = copy.deepcopy(profile)
        changed["sources"][0]["unexpected"] = True
        with self.assertRaisesRegex(d0.SourcePreflightError, "fields changed"):
            d0.validate_profile(changed)
        with self.assertRaisesRegex(d0.SourcePreflightError, "outside"):
            d0.validate_output(ROOT / "forbidden-v46-d0-output")

    def test_owner_has_no_network_or_model_audio_import(self) -> None:
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
