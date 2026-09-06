#!/usr/bin/env python3
"""Focused guards for the V10 A1R raw-force zero-decode inventory."""

from __future__ import annotations

import hashlib
import io
import struct
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_contact_field_r3a_v10_a1r_force_source_inventory as inventory  # noqa: E402


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


def write_tar(path: Path, members: list[tuple[str, bytes]]) -> None:
    with tarfile.open(path, "w:gz") as archive:
        for name, payload in members:
            info = tarfile.TarInfo(name)
            info.size = len(payload)
            info.mtime = 0
            archive.addfile(info, io.BytesIO(payload))


class A1RForceSourceInventoryTests(unittest.TestCase):
    def test_pcm_header_accepts_frozen_native_format(self) -> None:
        payload = pcm_wave_payload()
        self.assertEqual(
            inventory.parse_pcm_header(payload[:64], len(payload)),
            inventory.EXPECTED_WAV_HEADER,
        )

    def test_pcm_header_rejects_resampling(self) -> None:
        payload = bytearray(pcm_wave_payload())
        payload[24:28] = struct.pack("<I", 44_100)
        with self.assertRaises(inventory.InventoryError):
            inventory.parse_pcm_header(payload[:64], len(payload))

    def test_official_page_requires_force_coordinate_and_object_identity(self) -> None:
        text = "\n".join(
            (
                "ground-truth contact force profile",
                "coordinate of the striking location on the object mesh",
                "|  51   |     Fruit_Bowl     |     Glass",
                "audio_data_[X+1]_[X+10].tar.gz",
            )
        )
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "page.md"
            path.write_text(text)
            descriptor = inventory.validate_official_page(path)
        self.assertEqual(descriptor["commit"], inventory.OFFICIAL_SITE_COMMIT)

    def test_raw_scan_hashes_only_selected_members_and_preserves_order(self) -> None:
        wav = pcm_wave_payload()
        members = [
            ("51/audio/27/unselected.bin", b"not hashed"),
            ("51/audio/27/mic.wav", wav),
            ("51/audio/15/mic.wav", wav),
        ]
        expected = {
            name: (len(payload), hashlib.sha256(payload).hexdigest())
            for name, payload in members
            if name.endswith("mic.wav")
        }
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "prefix.tar.gz"
            write_tar(path, members)
            found, order = inventory.scan_raw_prefix(path, expected, (27, 15))
        self.assertEqual(set(found), set(expected))
        self.assertEqual(order, [27, 15])

    def test_raw_scan_rejects_archive_order_change(self) -> None:
        wav = pcm_wave_payload()
        members = [
            ("51/audio/15/mic.wav", wav),
            ("51/audio/27/mic.wav", wav),
        ]
        expected = {
            name: (len(payload), hashlib.sha256(payload).hexdigest())
            for name, payload in members
        }
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "prefix.tar.gz"
            write_tar(path, members)
            with self.assertRaisesRegex(inventory.InventoryError, "order changed"):
                inventory.scan_raw_prefix(path, expected, (27, 15))

    def test_manifest_keeps_protected_and_runtime_paths_closed(self) -> None:
        manifest = inventory.build_manifest({}, [], {}, {}, "0" * 64)
        self.assertEqual(manifest["microphone_sample_values_decoded"], 0)
        self.assertEqual(manifest["force_sample_values_decoded"], 0)
        self.assertEqual(
            manifest["development_microphone_sample_values_decoded"], 0
        )
        self.assertEqual(manifest["sealed_force_sample_values_decoded"], 0)
        self.assertEqual(manifest["next_authorized_role"], "fit")
        self.assertFalse(manifest["development_authorized"])
        self.assertFalse(manifest["sealed_authorized"])
        self.assertFalse(manifest["r3b_authorized"])
        self.assertFalse(manifest["runtime_neural_inference_authorized"])

    def test_role_order_is_frozen_before_decode(self) -> None:
        self.assertEqual(
            [(item["contact_id"], item["role"]) for item in inventory.CONTACTS],
            [
                (27, "fit"),
                (15, "fit"),
                (4, "fit"),
                (3, "fit"),
                (9, "development"),
                (18, "sealed"),
            ],
        )


if __name__ == "__main__":
    unittest.main()
