from __future__ import annotations

import hashlib
import io
import struct
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v8_real_source_inventory as inventory


def pcm_wave_payload(frames: int = 16) -> bytes:
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


def write_tar(path: Path, members: dict[str, bytes]) -> None:
    with tarfile.open(path, "w:gz") as archive:
        for name, payload in members.items():
            info = tarfile.TarInfo(name)
            info.size = len(payload)
            info.mtime = 0
            archive.addfile(info, io.BytesIO(payload))


class PhysicalSoundContactFieldR3AV8RealSourceInventoryTests(unittest.TestCase):
    def test_pcm_header_parser_accepts_frozen_format(self) -> None:
        payload = pcm_wave_payload(288_000)
        descriptor = inventory.parse_pcm_header(payload[:64], len(payload))
        self.assertEqual(descriptor, inventory.EXPECTED_WAV_HEADER)

    def test_pcm_header_parser_rejects_sample_rate_change(self) -> None:
        payload = bytearray(pcm_wave_payload(288_000))
        payload[24:28] = struct.pack("<I", 44_100)
        with self.assertRaises(inventory.InventoryError):
            inventory.parse_pcm_header(bytes(payload[:64]), len(payload))

    def test_streaming_scan_hashes_only_selected_members(self) -> None:
        selected = pcm_wave_payload(288_000)
        members = {
            "selected.wav": selected,
            "unselected.bin": b"not part of the commitment",
        }
        expected = {
            "selected.wav": (len(selected), hashlib.sha256(selected).hexdigest())
        }
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "fixture.tar.gz"
            write_tar(path, members)
            observed = inventory.scan_selected_members(path, expected)
        self.assertEqual(set(observed), {"selected.wav"})
        self.assertEqual(observed["selected.wav"]["wav_header"]["frame_count"], 288_000)

    def test_streaming_scan_rejects_missing_selected_member(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "fixture.tar.gz"
            write_tar(path, {"other.bin": b"value"})
            with self.assertRaises(inventory.InventoryError):
                inventory.scan_selected_members(
                    path, {"missing.bin": (1, hashlib.sha256(b"x").hexdigest())}
                )

    def test_contact_manifest_preserves_frozen_role_order(self) -> None:
        found = {}
        for contact in inventory.CONTACTS:
            for name in inventory.MEMBER_ORDER:
                path = f"91/audio/{contact['contact_id']}/{name}"
                found[path] = {
                    "path": path,
                    "bytes": 1,
                    "sha256": "0" * 64,
                    "wav_header": None,
                }
        value = inventory.contact_manifest(found)
        self.assertEqual([item["contact_id"] for item in value], ["18", "12", "4", "20", "27"])
        self.assertEqual([item["role"] for item in value], ["fit", "fit", "fit", "development", "sealed"])

    def test_manifest_forbids_decode_development_and_runtime(self) -> None:
        manifest = inventory.build_manifest("0" * 64, [], "1" * 64)
        self.assertEqual(manifest["waveform_sample_values_decoded"], 0)
        self.assertEqual(manifest["development_waveform_sample_values_decoded"], 0)
        self.assertEqual(manifest["sealed_waveform_sample_values_decoded"], 0)
        self.assertFalse(manifest["real_fit_extraction_authorized"])
        self.assertFalse(manifest["real_development_authorized"])
        self.assertFalse(manifest["r3b_authorized"])
        self.assertFalse(manifest["runtime_neural_inference_authorized"])


if __name__ == "__main__":
    unittest.main()
