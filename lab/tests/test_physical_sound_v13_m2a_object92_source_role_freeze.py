#!/usr/bin/env python3
"""Focused guards for the V13-M2a object-92 zero-sample freeze."""

from __future__ import annotations

import copy
import hashlib
import json
import struct
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v13_m2a_object92_source_role_freeze as m2a  # noqa: E402


def npy_float64(shape: tuple[int, ...], values: tuple[float, ...]) -> bytes:
    header = str(
        {
            "descr": "<f8",
            "fortran_order": False,
            "shape": shape,
        }
    ).encode("latin1")
    padding = (64 - ((10 + len(header) + 1) % 64)) % 64
    header += b" " * padding + b"\n"
    return (
        b"\x93NUMPY"
        + bytes((1, 0))
        + struct.pack("<H", len(header))
        + header
        + struct.pack(f"<{len(values)}d", *values)
    )


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


def valid_shortlist() -> dict[str, object]:
    return {
        "decision": "FreshGlassCandidatesAvailable",
        "fresh_candidates": [
            {
                "decision": "FreshMetadataOnly",
                "direct_exposure_count": 0,
                "material": material,
                "name": name,
                "object_id": object_id,
                "path_token_exposure_count": 0,
            }
            for object_id, name, material in m2a.SHORTLIST_FRESH
        ],
    }


class Object92SourceRoleFreezeTests(unittest.TestCase):
    def test_frozen_partition_is_complete_contact_parent_disjoint(self) -> None:
        flat = [contact for contacts in m2a.EXPECTED_ROLES.values() for contact in contacts]
        self.assertEqual(len(flat), 36)
        self.assertEqual(len(set(flat)), 36)
        self.assertEqual(set(flat), set(range(36)))
        self.assertEqual(len(m2a.EXPECTED_ROLES["formula_fit"]), 8)
        self.assertEqual(len(m2a.EXPECTED_ROLES["admission_shadow"]), 3)

    def test_small_farthest_point_case_is_deterministic(self) -> None:
        coordinates = {
            0: (0.0, 0.0, 0.0),
            1: (1.0, 0.0, 0.0),
            2: (3.0, 0.0, 0.0),
            3: (7.0, 0.0, 0.0),
        }
        first = m2a.farthest_point_order((0, 1, 2, 3), 3, "test", coordinates)
        second = m2a.farthest_point_order((3, 2, 1, 0), 3, "test", coordinates)
        self.assertEqual(first, second)
        self.assertEqual(len(first), 3)
        self.assertEqual(len(set(first)), 3)

    def test_shortlist_requires_exact_fresh_zero_exposure_set(self) -> None:
        self.assertEqual(m2a.validate_shortlist(valid_shortlist())["decision"], "FreshGlassCandidatesAvailable")

        exposed = valid_shortlist()
        exposed["fresh_candidates"][2]["direct_exposure_count"] = 1
        with self.assertRaisesRegex(m2a.FreezeError, "direct prior exposure"):
            m2a.validate_shortlist(exposed)

        reordered = valid_shortlist()
        reordered["fresh_candidates"].reverse()
        with self.assertRaisesRegex(m2a.FreezeError, "set or order changed"):
            m2a.validate_shortlist(reordered)

    def test_official_split_must_be_exact_complete_and_ordered(self) -> None:
        source = {
            role: [["92", str(contact)] for contact in contacts]
            for role, contacts in m2a.PUBLISHER_SPLIT.items()
        }
        self.assertEqual(m2a.validate_split(source), m2a.PUBLISHER_SPLIT)

        changed = copy.deepcopy(source)
        changed["train"][0], changed["train"][1] = changed["train"][1], changed["train"][0]
        with self.assertRaisesRegex(m2a.FreezeError, "train split changed"):
            m2a.validate_split(changed)

    def test_npy_parser_accepts_only_finite_little_endian_float64_shape(self) -> None:
        payload = npy_float64((3,), (1.0, -2.5, 3.25))
        self.assertEqual(m2a.parse_npy_float64(payload, (3,)), (1.0, -2.5, 3.25))

        wrong_shape = npy_float64((1, 3), (1.0, 2.0, 3.0))
        with self.assertRaisesRegex(m2a.FreezeError, "shape changed"):
            m2a.parse_npy_float64(wrong_shape, (3,))

        nonfinite = npy_float64((3,), (1.0, float("inf"), 3.0))
        with self.assertRaisesRegex(m2a.FreezeError, "non-finite"):
            m2a.parse_npy_float64(nonfinite, (3,))

    def test_wav_parser_reads_header_without_producing_samples(self) -> None:
        payload = pcm16_mono_48k()
        self.assertEqual(len(payload), m2a.EXPECTED_WAV_BYTES)
        self.assertEqual(m2a.parse_pcm_header(payload), m2a.EXPECTED_WAV_HEADER)

        mutated = bytearray(payload)
        mutated[22:24] = struct.pack("<H", 2)
        with self.assertRaisesRegex(m2a.FreezeError, "descriptor changed"):
            m2a.parse_pcm_header(bytes(mutated))

    def test_source_size_and_hash_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "source.bin"
            path.write_bytes(b"exact-source")
            source = {
                "relative_name": "source.bin",
                "bytes": len(b"exact-source"),
                "sha256": hashlib.sha256(b"exact-source").hexdigest(),
            }
            self.assertEqual(m2a.require_source(m2a.repository_root(), path, source), path.resolve())
            source["sha256"] = "0" * 64
            with self.assertRaisesRegex(m2a.FreezeError, "source hash changed"):
                m2a.require_source(m2a.repository_root(), path, source)

    def test_manifest_keeps_all_real_roles_closed_and_zero_sample(self) -> None:
        audio = {
            contact: {
                "path": f"audio/92/{contact}.wav",
                "bytes": m2a.EXPECTED_WAV_BYTES,
                "sha256": hashlib.sha256(f"audio-{contact}".encode()).hexdigest(),
                "header": m2a.EXPECTED_WAV_HEADER,
                "sample_values_decoded": 0,
            }
            for contact in m2a.EXPECTED_CONTACT_IDS
        }
        contacts = {
            contact: {
                "path": f"contacts/92/{contact}.npy",
                "bytes": 152,
                "sha256": hashlib.sha256(f"contact-{contact}".encode()).hexdigest(),
                "coordinate_m": [float(contact), 0.0, 0.0],
            }
            for contact in m2a.EXPECTED_CONTACT_IDS
        }
        point = {
            "path": "global_gt_points/92.npy",
            "bytes": 24_704,
            "sha256": "1" * 64,
            "shape": [1024, 3],
            "dtype": "float64",
            "minimum_m": [0.0, 0.0, 0.0],
            "maximum_m": [1.0, 1.0, 1.0],
        }
        manifest = m2a.build_manifest(
            valid_shortlist(),
            audio,
            contacts,
            point,
            m2a.PUBLISHER_SPLIT,
            m2a.OBJECT_SCALE,
            m2a.EXPECTED_ROLES,
            "2" * 64,
        )
        accounting = manifest["read_accounting"]
        self.assertEqual(accounting["pcm_sample_values_decoded"], 0)
        self.assertEqual(accounting["force_sample_values_decoded"], 0)
        self.assertEqual(accounting["selected_wav_members_hash_committed"], 36)
        self.assertTrue(manifest["all_real_roles_sample_decode_closed"])
        self.assertFalse(manifest["formula_or_model_fit_authorized"])
        self.assertTrue(manifest["authored_clip_fallback_required"])

    def test_output_inside_repository_is_rejected(self) -> None:
        output = m2a.repository_root() / "forbidden-m2a-output"
        with self.assertRaisesRegex(m2a.FreezeError, "outside the repository"):
            m2a.prepare_output(m2a.repository_root(), output)


if __name__ == "__main__":
    unittest.main()
