#!/usr/bin/env python3
"""Focused guards for the V13-M2b bounded raw-force acquisition/inventory."""

from __future__ import annotations

import io
import json
import struct
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v13_m2b_object92_raw_force_acquire as acquire  # noqa: E402
import physical_sound_v13_m2b_object92_raw_force_inventory as inventory  # noqa: E402


def pcm16_mono_48k() -> bytes:
    data_bytes = 288_000 * 2
    return b"".join(
        (
            b"RIFF",
            struct.pack("<I", 36 + data_bytes),
            b"WAVEfmt ",
            struct.pack("<IHHIIHH", 16, 1, 1, 48_000, 96_000, 2, 16),
            b"data",
            struct.pack("<I", data_bytes),
            bytes(data_bytes),
        )
    )


class FakeResponse:
    status = 206

    def __init__(self, start: int, end: int) -> None:
        self.headers = {
            "Content-Range": f"bytes {start}-{end}/{acquire.FULL_BYTES}",
            "Content-Length": str(end - start + 1),
            "ETag": acquire.ETAG,
            "Last-Modified": acquire.LAST_MODIFIED,
        }

    def geturl(self) -> str:
        return acquire.URL


class RawForceM2bTests(unittest.TestCase):
    def test_checkpoints_and_hard_ceiling_are_frozen(self) -> None:
        self.assertEqual(acquire.CHECKPOINTS_GIB, (4, 8, 12))
        self.assertEqual(max(acquire.CHECKPOINTS_GIB) * acquire.GIB, 12_884_901_888)
        self.assertEqual(inventory.CHECKPOINTS_GIB, acquire.CHECKPOINTS_GIB)
        self.assertEqual(len(inventory.selected_paths()), 144)

    def test_exact_range_headers_are_required(self) -> None:
        response = FakeResponse(10, 19)
        acquire.validate_response(response, 10, 19)
        response.headers["Content-Range"] = "bytes 0-9/10"
        with self.assertRaisesRegex(acquire.AcquisitionError, "Content-Range"):
            acquire.validate_response(response, 10, 19)

    def test_acquisition_root_has_exact_marker_and_is_reentrant(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "m2b"
            first = acquire.prepare_root(root)
            second = acquire.prepare_root(root)
            self.assertEqual(first, second)
            marker = json.loads((root / "acquisition-marker.json").read_bytes())
            self.assertEqual(marker["schema"], acquire.MARKER_SCHEMA)

            (root / "acquisition-marker.json").write_text("{}")
            with self.assertRaisesRegex(acquire.AcquisitionError, "not an exact M2b root"):
                acquire.prepare_root(root)

    def test_wav_header_parser_never_returns_sample_values(self) -> None:
        payload = pcm16_mono_48k()
        self.assertEqual(inventory.parse_pcm_header(payload), inventory.EXPECTED_WAV_HEADER)
        changed = bytearray(payload)
        changed[24:28] = struct.pack("<I", 44_100)
        with self.assertRaisesRegex(inventory.InventoryError, "descriptor changed"):
            inventory.parse_pcm_header(bytes(changed))

    def test_parent_requires_exact_roles_and_zero_sample_counters(self) -> None:
        contacts = []
        role_by_contact = {}
        for role, contact_ids in inventory.EXPECTED_ROLES.items():
            for contact in contact_ids:
                role_by_contact[contact] = role
        for contact in inventory.EXPECTED_CONTACT_IDS:
            contacts.append(
                {
                    "contact_id": contact,
                    "role": role_by_contact[contact],
                    "microphone": {"sha256": f"{contact:064x}"},
                }
            )
        manifest = {
            "role_root_sha256": inventory.PARENT_ROLE_ROOT_SHA256,
            "roles": inventory.EXPECTED_ROLES,
            "contacts": contacts,
        }
        report = {
            "manifest_sha256": inventory.PARENT_MANIFEST_SHA256,
            "read_accounting": {
                "pcm_sample_values_decoded": 0,
                "force_sample_values_decoded": 0,
            },
        }
        hashes, roles = inventory.validate_parent(manifest, report)
        self.assertEqual(len(hashes), 36)
        self.assertEqual(roles[35], role_by_contact[35])

        report["read_accounting"]["force_sample_values_decoded"] = 1
        with self.assertRaisesRegex(inventory.InventoryError, "force sample counter"):
            inventory.validate_parent(manifest, report)

    def test_truncated_tar_inventory_is_structural_and_zero_decode(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            prefix = Path(temporary) / "prefix.tar.gz"
            with tarfile.open(prefix, "w:gz") as archive:
                payload = b"gain: 1\n"
                member = tarfile.TarInfo("92/audio/0/metadata.yaml")
                member.size = len(payload)
                archive.addfile(member, io.BytesIO(payload))
                sentinel = tarfile.TarInfo("93/audio/0/metadata.yaml")
                sentinel.size = len(payload)
                archive.addfile(sentinel, io.BytesIO(payload))
            found, consumed, truncated, contacts, reached_next = inventory.scan_prefix(prefix)
        self.assertEqual(set(found), {"92/audio/0/metadata.yaml"})
        self.assertEqual(found["92/audio/0/metadata.yaml"]["numeric_values_decoded"], 0)
        self.assertGreater(consumed, 0)
        self.assertFalse(truncated)
        self.assertEqual(contacts, {0})
        self.assertTrue(reached_next)

    def test_next_object_proves_terminal_source_incompleteness(self) -> None:
        self.assertEqual(
            inventory.inventory_decision(False, True, 8),
            "SOURCE_INCOMPLETE_OBJECT92",
        )
        self.assertEqual(
            inventory.inventory_decision(False, False, 8),
            "DATA_INSUFFICIENT_CHECKPOINT_EXTEND_WITHIN_PROTOCOL",
        )
        self.assertEqual(
            inventory.inventory_decision(False, False, 12),
            "DATA_INSUFFICIENT_ACQUISITION",
        )

    def test_repository_outputs_are_rejected(self) -> None:
        with self.assertRaisesRegex(inventory.InventoryError, "outside the repository"):
            inventory.prepare_output(inventory.repository_root() / "forbidden-m2b")
        with self.assertRaisesRegex(acquire.AcquisitionError, "outside the repository"):
            acquire.prepare_root(acquire.repository_root() / "forbidden-m2b-acquire")


if __name__ == "__main__":
    unittest.main()
