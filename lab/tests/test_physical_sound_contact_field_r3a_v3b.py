from __future__ import annotations

import io
import math
import sys
import unittest
import zlib
from pathlib import Path

import numpy as np

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v3b_codec_oracle as oracle  # noqa: E402
import physical_sound_contact_field_r3a_v3b_common as common  # noqa: E402
import physical_sound_contact_field_r3a_v3b_extract as extraction  # noqa: E402
import physical_sound_contact_field_r3a_v3b_preflight as preflight  # noqa: E402


class PhysicalSoundContactFieldR3AV3BTests(unittest.TestCase):
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

    def test_metadata_split_is_source_ordered_and_seals_last_contact(self) -> None:
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
        with self.assertRaises(common.R3AV3BError):
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

    def test_native_identity_control_requires_exact_zero(self) -> None:
        exact = {endpoint: 0.0 for endpoint in oracle.PRIMARY_ENDPOINTS}
        self.assertTrue(oracle.identity_gate(exact)["passed"])
        exact["absolute_rms_level_error_db"] = np.finfo(np.float64).eps
        self.assertFalse(oracle.identity_gate(exact)["passed"])

    def test_v3_object_is_absent_from_prior_accessed_roster(self) -> None:
        self.assertNotIn(
            common.DATASET_OBJECT_ID,
            common.PRIOR_ACCESSED_DATASET_OBJECT_IDS,
        )
        self.assertEqual(common.DAC_MODEL_SAMPLE_RATE_HZ, common.SAMPLE_RATE_HZ)

    def test_decode_guard_closes_frozen_eight_sample_deficit(self) -> None:
        without_guard = (
            math.ceil(common.ANALYSIS_SAMPLES / common.DAC_MODEL_HOP_LENGTH)
            * common.DAC_MODEL_HOP_LENGTH
            - common.DAC_DECODE_GUARD_SAMPLES
        )
        guarded = (
            math.ceil(
                (common.ANALYSIS_SAMPLES + common.DAC_DECODE_GUARD_SAMPLES)
                / common.DAC_MODEL_HOP_LENGTH
            )
            * common.DAC_MODEL_HOP_LENGTH
            - common.DAC_DECODE_GUARD_SAMPLES
        )
        self.assertEqual(without_guard, common.ANALYSIS_SAMPLES - 8)
        self.assertGreaterEqual(guarded, common.ANALYSIS_SAMPLES)
        self.assertEqual(common.DAC_ANALYSIS_CODE_FRAMES, 226)

    def test_candidate_gate_requires_absolute_quality_and_baseline_gain(self) -> None:
        candidate = {
            endpoint: 0.1 * threshold
            for endpoint, threshold in oracle.ABSOLUTE_THRESHOLDS.items()
        }
        baseline = {
            endpoint: 2.0 * threshold
            for endpoint, threshold in oracle.ABSOLUTE_THRESHOLDS.items()
        }
        self.assertTrue(oracle.candidate_gate(candidate, baseline)["passed"])
        candidate["normalized_envelope_rmse"] = 2.0
        self.assertFalse(oracle.candidate_gate(candidate, baseline)["passed"])

    def test_implementation_lineage_rejects_post_freeze_change(self) -> None:
        manifest = {
            "implementation_sha256": common.implementation_hashes(SCRIPT_DIRECTORY)
        }
        common.validate_implementation(manifest, SCRIPT_DIRECTORY)
        manifest["implementation_sha256"]["oracle"] = "0" * 64
        with self.assertRaises(common.R3AV3BError):
            common.validate_implementation(manifest, SCRIPT_DIRECTORY)


if __name__ == "__main__":
    unittest.main()
