"""Pure signal/measurement checks; no model download needed."""

import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
from scipy.io import wavfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_text_pilot as pilot
import physical_sound_text_tags as tags


class TextPilotTest(unittest.TestCase):
    def test_ast_rejects_unknown_device_before_loading_artifacts(self):
        with (
            tempfile.TemporaryDirectory() as directory,
            self.assertRaisesRegex(ValueError, "unsupported AST device"),
        ):
            tags.run(
                Path("missing-source"),
                Path(directory) / "report.json",
                [],
                device="invalid",
            )

    def test_ast_level_control_is_opt_in_and_gain_invariant(self):
        wave = np.array([0.01, -0.02, 0.03, -0.04], dtype=np.float32)
        raw, record = tags.ast_level_control(wave, None)
        self.assertIs(raw, wave)
        self.assertEqual(record["gain"], 1)
        a, record = tags.ast_level_control(wave, 0.005)
        b, _ = tags.ast_level_control(wave * 10, 0.005)
        np.testing.assert_allclose(a, b, atol=1e-9)
        self.assertAlmostEqual(float(np.sqrt(np.mean(a * a))), 0.005, places=8)
        np.testing.assert_array_equal(
            wave, np.array([0.01, -0.02, 0.03, -0.04], dtype=np.float32)
        )

    def test_ast_level_control_preserves_silence_and_rejects_clipping(self):
        silence, record = tags.ast_level_control(np.zeros(100, dtype=np.float32), 0.005)
        self.assertTrue(record["zero_energy"])
        self.assertEqual(float(abs(silence).max()), 0)
        impulse = np.zeros(10000, dtype=np.float32)
        impulse[0] = 0.1
        with self.assertRaises(ValueError):
            tags.ast_level_control(impulse, 0.02)
        for level in [-1, 0, float("nan"), 1]:
            with self.assertRaises(ValueError):
                tags.ast_level_control(silence, level)

    def test_clap_wrapper_retains_case_and_empty_control_pairing(self):
        manifest = {
            "cases": [{"id": "a", "prompt": "glass"}, {"id": "b", "prompt": "wood"}],
            "rows": [
                {"case": 1, "seed": 42, "wav": "wood.wav"},
                {"case": 0, "seed": 42, "wav": "glass.wav"},
            ],
            "controls": [{"id": "empty-prompt", "seed": 42, "wav": "empty.wav"}],
            "seeds": [42],
        }
        scores = [[0.1, 0.8], [0.6, 0.3], [0.4, 0.5]]
        with patch.object(tags, "clap_similarities", return_value=scores):
            result = tags.clap_measurement(manifest)
        self.assertEqual(result["top1_count"], 2)
        self.assertEqual(result["beats_empty_prompt_count"], 2)
        self.assertAlmostEqual(result["rows"][0]["target_gain_over_empty_prompt"], 0.3)
        self.assertAlmostEqual(result["rows"][1]["target_gain_over_empty_prompt"], 0.2)
        self.assertEqual(result["paired_changes"], [])

    def test_empty_clap_batch_fails_before_loading_models(self):
        for prompts, recordings in (([], [{}]), (["glass"], [])):
            with self.assertRaisesRegex(ValueError, "nonempty"):
                tags.clap_similarities(prompts, recordings)

    def test_unknown_ontology_axis_is_unscored(self):
        self.assertEqual(tags.EXPECTED["steel-metal"], ())
        result = tags.summarize(np.array([0.7, 0.2]), ["Glass", "Rain"], ())
        self.assertIsNone(result["expected_in_top5"])
        with self.assertRaises(ValueError):
            tags.summarize(np.array([0.7, 0.2]), ["Glass", "Rain"], ("Steel",))

    def test_waveform_hash_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "test.wav"
            pilot.write_audio(path, np.ones(1000, dtype=np.float32) * 0.1)
            with self.assertRaises(ValueError):
                tags.load_audio(path, "wrong")

    def test_bad_signal_rejected(self):
        for value in ([], [float("nan")], [float("inf")], [[1, 2]]):
            with self.assertRaises(ValueError):
                pilot.signal_stats(np.asarray(value))

    def test_pcm_no_amplification_and_no_clipping(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "test.wav"
            for amplitude in (0.1, 1.5):
                report = pilot.write_audio(
                    path, np.array([amplitude, -amplitude], dtype=np.float32)
                )
                rate, pcm = wavfile.read(path)
                self.assertEqual(rate, pilot.RATE)
                self.assertLessEqual(report["pcm_gain"], 1)
                self.assertLessEqual(np.abs(pcm).max(), 32113)
            with self.assertRaises(ValueError):
                pilot.write_audio(path, np.zeros(100))

    def test_alignment_detects_swapped_label(self):
        self.assertEqual(pilot.alignment(np.array([0.8, 0.2]), 0)["target_rank"], 1)
        self.assertLess(
            pilot.alignment(np.array([0.8, 0.2]), 1)["target_minus_best_other"], 0
        )
        with self.assertRaises(ValueError):
            pilot.alignment(np.array([0.8, np.nan]), 0)
        with self.assertRaises(ValueError):
            pilot.alignment(np.array([0.8, 0.2]), -1)

    def test_pair_contrast_and_no_effect_control(self):
        matrix = np.array([[0.8, 0.2], [0.3, 0.7]])
        self.assertAlmostEqual(pilot.pair_margin(matrix, 0, 1), 0.5)
        self.assertLess(pilot.pair_margin(matrix[::-1], 0, 1), 0)
        self.assertAlmostEqual(
            pilot.pair_margin(np.array([[0.8, 0.2], [0.8, 0.2]]), 0, 1), 0
        )


if __name__ == "__main__":
    unittest.main()
