from __future__ import annotations

import hashlib
import io
import json
import struct
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

import numpy as np

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v10_real_source_inventory as inventory


def pcm_wave_payload(frames: int = 288_000) -> bytes:
    channels = 1
    sample_width = 2
    sample_rate = 48_000
    data = bytes(frames * channels * sample_width)
    return (
        b"RIFF"
        + struct.pack("<I", 36 + len(data))
        + b"WAVEfmt "
        + struct.pack(
            "<IHHIIHH",
            16,
            1,
            channels,
            sample_rate,
            sample_rate * channels * sample_width,
            channels * sample_width,
            sample_width * 8,
        )
        + b"data"
        + struct.pack("<I", len(data))
        + data
    )


def npy_payload(values: np.ndarray) -> bytes:
    stream = io.BytesIO()
    np.save(stream, values, allow_pickle=False)
    return stream.getvalue()


def write_tar(path: Path, members: dict[str, bytes]) -> None:
    with tarfile.open(path, "w:gz") as archive:
        for name, payload in members.items():
            info = tarfile.TarInfo(name)
            info.size = len(payload)
            info.mtime = 0
            archive.addfile(info, io.BytesIO(payload))


class PhysicalSoundContactFieldR3AV10RealSourceInventoryTests(unittest.TestCase):
    def test_roles_are_disjoint_complete_and_preserve_official_splits(self) -> None:
        inventory.validate_roles()
        self.assertEqual(
            inventory.OBJECTS["60"]["roles"]["fit"],
            inventory.EXPECTED_OFFICIAL_SPLITS["60"]["train"],
        )
        self.assertEqual(
            inventory.OBJECTS["60"]["roles"]["development"],
            inventory.EXPECTED_OFFICIAL_SPLITS["60"]["val"],
        )
        self.assertEqual(
            inventory.OBJECTS["60"]["roles"]["exact_object_query"],
            inventory.EXPECTED_OFFICIAL_SPLITS["60"]["test"],
        )
        self.assertEqual(
            inventory.OBJECTS["22"]["roles"]["representation_holdout"],
            inventory.EXPECTED_OFFICIAL_SPLITS["22"]["test"],
        )

    def test_pcm_header_parser_accepts_only_frozen_format(self) -> None:
        payload = pcm_wave_payload()
        self.assertEqual(
            inventory.parse_pcm_header(payload[:64], len(payload)),
            inventory.EXPECTED_WAV_HEADER,
        )
        changed = bytearray(payload[:64])
        changed[24:28] = struct.pack("<I", 44_100)
        with self.assertRaises(inventory.InventoryError):
            inventory.parse_pcm_header(bytes(changed), len(payload))

    def test_npy_parser_requires_finite_float64_shape(self) -> None:
        coordinate = np.array([1.0, 2.0, 3.0], dtype=np.float64)
        observed = inventory.parse_npy(npy_payload(coordinate), (3,))
        self.assertTrue(np.array_equal(observed, coordinate))
        with self.assertRaises(inventory.InventoryError):
            inventory.parse_npy(npy_payload(coordinate.astype(np.float32)), (3,))
        coordinate[1] = np.nan
        with self.assertRaises(inventory.InventoryError):
            inventory.parse_npy(npy_payload(coordinate), (3,))

    def test_tar_scan_hashes_only_selected_payload(self) -> None:
        selected = pcm_wave_payload()
        members = {
            "audio/60/0.wav": selected,
            "audio/60/1.wav": b"unselected",
        }
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "fixture.tar.gz"
            write_tar(path, members)
            found, observed = inventory.scan_tar(
                path, {"audio/60/0.wav"}, inventory.parse_audio_member
            )
        self.assertEqual(set(found), {"audio/60/0.wav"})
        self.assertEqual(found["audio/60/0.wav"]["sha256"], hashlib.sha256(selected).hexdigest())
        self.assertEqual(observed["60"], {0, 1})

    def test_official_split_rejects_contact_change(self) -> None:
        source = {"train": [], "val": [], "test": []}
        for object_id, roles in inventory.EXPECTED_OFFICIAL_SPLITS.items():
            for role, contacts in roles.items():
                source[role].extend([[int(object_id), contact] for contact in contacts])
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "split.json"
            path.write_text(json.dumps(source))
            self.assertEqual(inventory.official_splits(path), inventory.EXPECTED_OFFICIAL_SPLITS)
            source["test"].remove([60, 20])
            path.write_text(json.dumps(source))
            with self.assertRaises(inventory.InventoryError):
                inventory.official_splits(path)

    def test_manifest_keeps_protected_roles_closed(self) -> None:
        audio = {
            inventory.RAW_IDENTITY_CONTROL["processed_path"]: {
                "path": inventory.RAW_IDENTITY_CONTROL["processed_path"],
                "bytes": inventory.RAW_IDENTITY_CONTROL["bytes"],
                "sha256": inventory.RAW_IDENTITY_CONTROL["sha256"],
                "descriptor": inventory.EXPECTED_WAV_HEADER,
            }
        }
        contacts = {}
        for object_id in inventory.OBJECTS:
            for contact_id in inventory.EXPECTED_CONTACT_IDS:
                path = f"contacts/{object_id}/{contact_id}.npy"
                contacts[path] = {
                    "path": path,
                    "bytes": 152,
                    "sha256": "0" * 64,
                    "descriptor": {"coordinate_m": [0.0, 0.0, 0.0]},
                }
        points = {
            f"global_gt_points/{object_id}.npy": {
                "path": f"global_gt_points/{object_id}.npy",
                "bytes": 1,
                "sha256": "1" * 64,
                "descriptor": {},
            }
            for object_id in inventory.OBJECTS
        }
        manifest = inventory.build_manifest(
            "2" * 64,
            audio,
            contacts,
            points,
            inventory.EXPECTED_OFFICIAL_SPLITS,
            {object_id: value["scale"] for object_id, value in inventory.OBJECTS.items()},
        )
        self.assertEqual(manifest["waveform_sample_values_decoded"], 0)
        self.assertEqual(manifest["next_authorized_role"], "fit")
        self.assertFalse(manifest["development_authorized"])
        self.assertFalse(manifest["representation_holdout_authorized"])
        self.assertFalse(manifest["exact_object_query_authorized"])
        self.assertFalse(manifest["r3b_authorized"])
        self.assertFalse(manifest["runtime_neural_inference_authorized"])


if __name__ == "__main__":
    unittest.main()
