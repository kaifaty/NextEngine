#!/usr/bin/env python3
"""Focused guards for the preregistered V12-C3 zero-decode importer."""

from __future__ import annotations

import io
import json
import sys
import tarfile
import tempfile
import unittest
import wave
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_contact_field_r3a_v12_c3_source_role_inventory as inventory  # noqa: E402


def pcm16_wave() -> bytes:
    output = io.BytesIO()
    with wave.open(output, "wb") as stream:
        stream.setnchannels(1)
        stream.setsampwidth(2)
        stream.setframerate(48_000)
        stream.writeframes(b"\0\0" * 288_000)
    return output.getvalue()


def add_bytes(archive: tarfile.TarFile, name: str, payload: bytes) -> None:
    member = tarfile.TarInfo(name)
    member.size = len(payload)
    archive.addfile(member, io.BytesIO(payload))


def raw_fixture(path: Path, contacts: tuple[int, ...], omit: str | None = None) -> None:
    wav = pcm16_wave()
    with tarfile.open(path, "w:gz") as archive:
        for contact_id in contacts:
            values = {
                "mic.wav": wav,
                "Force.wav": wav,
                "metadata.yaml": b"opaque metadata\n",
                "striking_force.yaml": b"not parsed by C3\n",
            }
            for name, payload in values.items():
                member_name = f"41/audio/{contact_id}/{name}"
                if member_name != omit:
                    add_bytes(archive, member_name, payload)


class SourceRoleInventoryTests(unittest.TestCase):
    def test_role_partition_is_exact_disjoint_and_complete(self) -> None:
        partition, root, order = inventory.role_partition()
        self.assertEqual(partition, inventory.EXPECTED_ROLE_IDS)
        self.assertEqual(root, inventory.ROLE_ORDER_ROOT)
        self.assertEqual(len(order), 35)
        self.assertEqual(set(order), set(range(35)))
        self.assertEqual(len(inventory.role_for_contact(partition)), 35)

    def test_sample_budgets_follow_frozen_role_sizes(self) -> None:
        partition, _, _ = inventory.role_partition()
        budgets = inventory.sample_budgets(partition)
        self.assertEqual(budgets["estimator_fit"]["contact_count"], 16)
        self.assertEqual(budgets["estimator_fit"]["microphone_sample_values"], 4_608_000)
        self.assertEqual(budgets["admission_shadow"]["force_sample_values"], 864_000)

    def test_pcm_header_parser_checks_container_without_sample_decode(self) -> None:
        payload = pcm16_wave()
        self.assertEqual(inventory.parse_wav_header(payload, len(payload)), inventory.EXPECTED_WAV_HEADER)

    def test_raw_scanner_accepts_complete_paired_members(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "prefix.tar.gz"
            raw_fixture(path, (0, 1))
            report = inventory.scan_raw_prefix(path, (0, 1))
        self.assertTrue(report["complete"])
        self.assertEqual(report["complete_contact_ids"], [0, 1])
        self.assertEqual(report["missing_members"], [])
        self.assertEqual(len(report["records"]), 8)

    def test_raw_scanner_reports_incomplete_prefix_without_role_repair(self) -> None:
        missing = "41/audio/1/Force.wav"
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "prefix.tar.gz"
            raw_fixture(path, (0, 1), omit=missing)
            report = inventory.scan_raw_prefix(path, (0, 1))
        self.assertFalse(report["complete"])
        self.assertEqual(report["complete_contact_ids"], [0])
        self.assertEqual(report["missing_members"], [missing])

    def test_freeze_and_preflights_read_zero_source_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            frozen = inventory.freeze("0" * 64, root / "freeze")
            first = inventory.preflight(frozen / "manifest.json", root / "preflight-a")
            second = inventory.preflight(frozen / "manifest.json", root / "preflight-b")
            manifest = json.loads((frozen / "manifest.json").read_text())
            report = json.loads((first / "report.json").read_text())
            first_bytes = (first / "report.json").read_bytes()
            second_bytes = (second / "report.json").read_bytes()
        self.assertEqual(manifest["raw_prefix"]["bytes"], 4_294_967_296)
        self.assertEqual(report["decision"], "C3_SOURCE_ROLE_FREEZE_FROZEN")
        self.assertEqual(report["counters"]["source_bytes_read"], 0)
        self.assertEqual(report["counters"]["microphone_sample_values_decoded"], 0)
        self.assertFalse(any(report["role_decode_authorized"].values()))
        self.assertEqual(first_bytes, second_bytes)

    def test_manifest_rejects_role_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            frozen = inventory.freeze("1" * 64, root / "freeze")
            manifest_path = frozen / "manifest.json"
            value = json.loads(manifest_path.read_text())
            value["roles"]["estimator_fit"][0] = 0
            manifest_path.write_bytes(inventory.canonical_json(value))
            with self.assertRaisesRegex(inventory.InventoryError, "roles changed"):
                inventory.validate_manifest(manifest_path)


if __name__ == "__main__":
    unittest.main()
