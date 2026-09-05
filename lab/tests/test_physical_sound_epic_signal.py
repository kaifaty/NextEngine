import hashlib
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_epic_signal as s


class SignalTests(unittest.TestCase):
    def test_regions_partition_and_source_alignment(self):
        wave = np.zeros(24000, np.float32)
        wave[4800:] = 0.1
        masks, info = s.regions(wave)
        self.assertEqual(info["onset_seconds"], 0.2)
        self.assertAlmostEqual(info["attack_end_seconds"], 0.3)
        self.assertEqual(info["source_seconds"], 1)
        self.assertTrue(np.all(np.stack(list(masks.values())).sum(0) == 1))
        self.assertEqual(sum(info["counts"].values()), 645)
        self.assertGreater(info["counts"]["padded_tail"], 600)
        self.assertGreater(info["counts"]["attack"], 0)
        with self.assertRaisesRegex(ValueError, "energy"):
            s.regions(np.zeros(24000))
        with self.assertRaisesRegex(ValueError, "finite"):
            s.regions(wave * np.nan)

    def test_error_components_reconstruct_full_loss(self):
        masks, _ = s.regions(np.ones(24000, np.float32))
        prediction = torch.zeros(1, 645, 64)
        for i, mask in enumerate(masks.values()):
            prediction[:, mask] = i + 1
        measured = s.measure(prediction, torch.zeros_like(prediction), masks)
        expected = (
            sum(measured[k] * mask.sum() for k, mask in masks.items() if mask.any())
            / 645
        )
        self.assertAlmostEqual(measured["full"], expected)
        self.assertIsNone(measured["pre_onset"])
        with self.assertRaisesRegex(ValueError, "shapes"):
            s.measure(prediction[:, :1], torch.zeros_like(prediction), masks)

    def test_all_wrong_conditions_not_one_convenient_swap(self):
        _, regions = s.regions(np.ones(24000, np.float32))
        label = s.pair.source.CLASSES[0]
        errors = {
            name: {k: 3.0 for k in ("full", *s.REGIONS)}
            for name in ("base", *s.pair.source.CLASSES)
        }
        errors[label] = dict.fromkeys(("full", *s.REGIONS), 1.0)
        errors[s.pair.source.CLASSES[-1]] = dict.fromkeys(("full", *s.REGIONS), 0.5)
        result = s.aggregate(
            [
                {
                    "precision": "fp32",
                    "regions": regions,
                    "errors": errors,
                    "label": label,
                }
            ]
        )
        self.assertEqual(result["fp32"]["attack"]["beats_base"], 1)
        self.assertEqual(result["fp32"]["attack"]["beats_all_wrong"], 0)
        self.assertGreater(result["fp32"]["attack"]["wrong_minus_matched"], 0)
        self.assertEqual(result["bf16"], {})
        self.assertAlmostEqual(
            sum(
                v["contribution_to_full_gain"]
                for k, v in result["fp32"].items()
                if k != "full"
            ),
            result["fp32"]["full"]["base_minus_matched"],
        )

    def test_diagnostic_playlist_keeps_actual_seeds_and_pcm(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            path = output / "input.wav"
            pcm = np.full(1600, 1000, np.int16)
            s.wavfile.write(path, 16000, pcm)
            rows = [
                {
                    "wav": str(path),
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                    "seed": 10000 + i,
                }
                for i in range(20)
            ]
            result = s.write_diagnostic_comparison(output, rows)
            self.assertTrue(result["reference_audio_input"])
            rate, actual = s.wavfile.read(result["wav"])
            self.assertEqual(rate, 16000)
            np.testing.assert_array_equal(
                actual, np.tile(np.r_[pcm, np.zeros(8000, np.int16)], 20)
            )
            with self.assertRaisesRegex(ValueError, "groups"):
                s.write_diagnostic_comparison(output, rows[:4])
            rows[0]["sha256"] = "bad"
            with self.assertRaisesRegex(ValueError, "identity"):
                s.write_diagnostic_comparison(output, rows)

    def test_gain_decomposition_includes_zero_length_regions(self):
        rows = []
        for onset in (0, 4800):
            wave = np.ones(24000, np.float32)
            wave[:onset] = 0
            masks, regions = s.regions(wave)
            zeros = torch.zeros(1, 645, 64)
            one = s.measure(torch.ones_like(zeros), zeros, masks)
            errors = {k: one for k in ("base", *s.pair.source.CLASSES)}
            label = s.pair.source.CLASSES[0]
            errors[label] = s.measure(zeros, zeros, masks)
            rows.append(
                {
                    "precision": "fp32",
                    "label": label,
                    "regions": regions,
                    "errors": errors,
                }
            )
        result = s.aggregate(rows)["fp32"]
        self.assertEqual(result["full"]["base_minus_matched"], 1)
        self.assertAlmostEqual(
            sum(
                v["contribution_to_full_gain"] for k, v in result.items() if k != "full"
            ),
            1,
        )


if __name__ == "__main__":
    unittest.main()
