#!/usr/bin/env python3
"""Focused guards for the V13-M2c metadata-only RealImpact control."""

from __future__ import annotations

import struct
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v13_m2c_realimpact_five_impact_control as m2c  # noqa: E402


def npy_payload(dtype: str, shape: tuple[int, ...], values: tuple[int | float, ...]) -> bytes:
    header = str({"descr": dtype, "fortran_order": False, "shape": shape}).encode("latin1")
    padding = (64 - ((10 + len(header) + 1) % 64)) % 64
    header += b" " * padding + b"\n"
    code = "d" if dtype == "<f8" else "q"
    return (
        b"\x93NUMPY"
        + bytes((1, 0))
        + struct.pack("<H", len(header))
        + header
        + struct.pack(f"<{len(values)}{code}", *values)
    )


def synthetic_metadata():
    vertex_xyz = []
    vertex_id = []
    listener_xyz = []
    microphone_id = []
    distance = []
    angle = []
    for impact in range(5):
        for local in range(600):
            microphone = local % 15
            distance_value = (local // 15) % 4 * 222
            angle_value = local // 60 * 40
            vertex_xyz.append((float(impact), float(impact + 1), float(impact + 2)))
            vertex_id.append(impact + 10)
            listener_xyz.append(
                (float(angle_value), float(distance_value), float(microphone))
            )
            microphone_id.append(microphone)
            distance.append(distance_value)
            angle.append(angle_value)
    return (
        tuple(vertex_xyz),
        tuple(vertex_id),
        tuple(listener_xyz),
        tuple(microphone_id),
        tuple(distance),
        tuple(angle),
    )


class RealImpactFiveImpactControlTests(unittest.TestCase):
    def test_metadata_ranges_never_overlap_forbidden_audio(self) -> None:
        m2c.require_no_audio_overlap(
            m2c.CENTRAL["offset"],
            m2c.CENTRAL["offset"] + m2c.CENTRAL["compressed_bytes"] - 1,
        )
        for spec in m2c.ENTRIES.values():
            m2c.require_no_audio_overlap(
                spec["offset"], spec["offset"] + spec["compressed_bytes"] - 1
            )
        start, _ = m2c.audio_interval()
        with self.assertRaisesRegex(m2c.ControlError, "overlaps"):
            m2c.require_no_audio_overlap(start, start)

    def test_folds_hold_every_impact_exactly_once(self) -> None:
        held = [fold["held_control_parent"] for fold in m2c.FOLDS.values()]
        self.assertEqual(held, [0, 1, 2, 3, 4])
        for fold in m2c.FOLDS.values():
            self.assertEqual(len(fold["fit_impact_parents"]), 4)
            self.assertNotIn(fold["held_control_parent"], fold["fit_impact_parents"])

    def test_npy_parser_requires_exact_dtype_shape_and_finite_values(self) -> None:
        payload = npy_payload("<f8", (3,), (1.0, 2.0, 3.0))
        self.assertEqual(m2c.parse_npy(payload, "<f8", (3,)), (1.0, 2.0, 3.0))
        with self.assertRaisesRegex(m2c.ControlError, "shape changed"):
            m2c.parse_npy(payload, "<f8", (1, 3))

        integers = npy_payload("<i8", (3,), (1, 2, 3))
        self.assertEqual(m2c.parse_npy(integers, "<i8", (3,)), (1, 2, 3))

        nonfinite = npy_payload("<f8", (3,), (1.0, float("inf"), 3.0))
        with self.assertRaisesRegex(m2c.ControlError, "non-finite"):
            m2c.parse_npy(nonfinite, "<f8", (3,))

    def test_five_impact_grid_and_canonical_rows_validate(self) -> None:
        values = synthetic_metadata()
        impacts, listener = m2c.validate_metadata(*values, mesh_vertex_count=100)
        self.assertEqual(len(impacts), 5)
        self.assertEqual(
            [impact["canonical_listener_row"] for impact in impacts],
            list(m2c.EXPECTED_CANONICAL_ROWS),
        )
        self.assertEqual(listener["microphone_id"], 7)
        self.assertEqual(listener["angle"], 0)
        self.assertEqual(listener["distance"], 0)

    def test_repeated_or_changed_impact_parent_fails_closed(self) -> None:
        values = list(synthetic_metadata())
        vertex_ids = list(values[1])
        vertex_positions = list(values[0])
        vertex_ids[600:1200] = vertex_ids[0:600]
        vertex_positions[600:1200] = vertex_positions[0:600]
        values[0] = tuple(vertex_positions)
        values[1] = tuple(vertex_ids)
        with self.assertRaisesRegex(m2c.ControlError, "not distinct"):
            m2c.validate_metadata(*values, mesh_vertex_count=100)

    def test_mesh_parser_rejects_empty_or_nonfinite_geometry(self) -> None:
        descriptor = m2c.parse_mesh(b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n")
        self.assertEqual(descriptor["vertex_count"], 3)
        self.assertEqual(descriptor["face_count"], 1)
        with self.assertRaisesRegex(m2c.ControlError, "no vertices or faces"):
            m2c.parse_mesh(b"v 0 0 0\n")

    def test_output_inside_repository_is_rejected(self) -> None:
        with self.assertRaisesRegex(m2c.ControlError, "outside the repository"):
            m2c.prepare_output(m2c.repository_root() / "forbidden-m2c-output")
        with tempfile.TemporaryDirectory() as temporary:
            output, staging = m2c.prepare_output(Path(temporary) / "allowed")
            self.assertNotEqual(output, staging)


if __name__ == "__main__":
    unittest.main()
