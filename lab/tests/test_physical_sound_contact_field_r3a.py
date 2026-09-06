from __future__ import annotations

import io
import sys
import unittest
import zlib
from pathlib import Path

import numpy as np

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_common as common  # noqa: E402
import physical_sound_contact_field_r3a_extract as extraction  # noqa: E402
import physical_sound_contact_field_r3a_oracle as oracle  # noqa: E402
import physical_sound_contact_field_r3a_preflight as preflight  # noqa: E402


class PhysicalSoundContactFieldR3ATests(unittest.TestCase):
    def test_implementation_lineage_rejects_post_freeze_runner_change(self) -> None:
        manifest = {
            "implementation_sha256": common.implementation_hashes(SCRIPT_DIRECTORY)
        }
        common.validate_implementation(manifest, SCRIPT_DIRECTORY)
        manifest["implementation_sha256"]["oracle"] = "0" * 64
        with self.assertRaises(common.R3AError):
            common.validate_implementation(manifest, SCRIPT_DIRECTORY)

    def metadata_arrays(self) -> dict[str, np.ndarray]:
        angles = np.repeat(np.arange(0, 200, 20, dtype=np.int64), 60)
        distances = np.tile(np.repeat(np.asarray([0, 333, 666, 1_000]), 15), 10)
        microphones = np.tile(np.arange(15, dtype=np.int64), 40)
        vertex_ids = np.repeat(np.asarray([10, 20, 30, 40, 50]), 600)
        position_table = np.asarray(
            [
                [-0.04, -0.03, 0.01],
                [-0.02, 0.01, 0.04],
                [0.00, -0.02, 0.05],
                [0.03, 0.02, 0.02],
                [0.04, -0.01, -0.01],
            ]
        )
        vertex_xyz = np.repeat(position_table, 600, axis=0)
        listener_xyz = np.tile(
            np.stack(
                [
                    0.23 + distances[:600] / 1_000.0,
                    np.zeros(600),
                    -0.91 + microphones[:600] * 0.01,
                ],
                axis=1,
            ),
            (5, 1),
        )
        return {
            "angle": np.tile(angles, 5),
            "distance": np.tile(distances, 5),
            "microphone_id": np.tile(microphones, 5),
            "vertex_id": vertex_ids,
            "vertex_xyz": vertex_xyz,
            "listener_xyz": listener_xyz,
        }

    def test_metadata_only_split_is_source_ordered_and_seals_last_impact(self) -> None:
        contacts = preflight.derive_contacts(self.metadata_arrays())
        self.assertEqual(
            [item["row_index"] for item in contacts], [7, 607, 1207, 1807, 2407]
        )
        self.assertEqual(
            [item["role"] for item in contacts],
            ["fit", "fit", "fit", "representation_development", "field_holdout"],
        )
        self.assertEqual(contacts[-1]["waveform_access"], "sealed")

    def test_metadata_split_rejects_duplicate_canonical_listener(self) -> None:
        arrays = self.metadata_arrays()
        arrays["microphone_id"][8] = common.CANONICAL_MICROPHONE_ID
        with self.assertRaises(common.R3AError):
            preflight.derive_contacts(arrays)

    def test_streaming_extraction_stops_before_unselected_rows(self) -> None:
        sample_count = 8
        rows = np.arange(5 * sample_count, dtype="<f4").reshape(5, sample_count)
        header = bytes(128)
        compressor = zlib.compressobj(level=6, wbits=-15)
        compressed = compressor.compress(header + rows.tobytes()) + compressor.flush()
        selected, _, raw_bytes = extraction.extract_rows(
            io.BytesIO(compressed),
            [0, 2],
            sample_count=sample_count,
            header_validator=lambda value: self.assertEqual(value, header),
        )
        np.testing.assert_array_equal(selected, rows[[0, 2]])
        self.assertEqual(raw_bytes, 128 + 3 * sample_count * 4)

    def test_equal_budget_query_oracle_passes_synthetic_modal_contact(self) -> None:
        sample_count = common.SAMPLE_COUNT
        time = (
            np.maximum(
                np.arange(sample_count, dtype=np.float64)
                - common.PEAK_ALIGNMENT_SAMPLE,
                0.0,
            )
            / common.SAMPLE_RATE_HZ
        )

        def contact(frequencies: list[float], gains: list[float]) -> np.ndarray:
            value = np.zeros(sample_count, dtype=np.float64)
            value[common.PEAK_ALIGNMENT_SAMPLE] = 0.7
            for frequency, gain in zip(frequencies, gains, strict=True):
                value += (
                    gain * np.exp(-7.0 * time) * np.sin(2.0 * np.pi * frequency * time)
                )
            value[: common.PEAK_ALIGNMENT_SAMPLE] = 0.0
            return value

        waveforms = np.stack(
            [
                contact([700.0, 1_500.0, 3_200.0], [0.30, 0.15, 0.08]),
                contact([760.0, 1_650.0, 3_500.0], [0.25, 0.18, 0.05]),
                contact([820.0, 1_800.0, 3_800.0], [0.20, 0.14, 0.09]),
                contact([960.9375, 2_296.875, 4_101.5625], [0.32, 0.19, 0.07]),
            ]
        )
        positions = np.asarray(
            [[0.0, 0.0, 0.0], [0.02, 0.0, 0.0], [0.04, 0.0, 0.0], [0.03, 0.01, 0.0]]
        )
        result = oracle.evaluate(waveforms, positions)
        self.assertEqual(result["decision"], "READY_FOR_EXACT_OBJECT_FIELD")
        self.assertEqual(
            result["modal_plus_residual"]["record"]["encoded_scalar_count"],
            oracle.SCALAR_BUDGET,
        )
        self.assertEqual(
            result["alternative"]["record"]["encoded_scalar_count"],
            oracle.SCALAR_BUDGET,
        )


if __name__ == "__main__":
    unittest.main()
