from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_codec as codec


class TextureCodecTests(unittest.TestCase):
    def test_gain_is_common_and_bounded_by_full_signal(self):
        a = np.array([0.01, -0.02])
        b = np.array([0.1, -0.25])
        self.assertEqual(codec.shared_gain([a, b]), 2)
        self.assertEqual(codec.shared_gain([a / 100]), 100)
        for waves in ([], [np.zeros(20)], [a * np.nan], [b * 5]):
            with self.assertRaises(ValueError):
                codec.shared_gain(waves)

    def test_crop_keeps_alignment_and_rejects_incomplete_audio(self):
        wave = np.repeat(np.arange(codec.RATE * 2)[:, None], 2, axis=1)
        out = codec.crop(wave, 0.25)
        np.testing.assert_array_equal(out[0], [11025, 11025])
        self.assertEqual(len(out), round(0.75 * codec.RATE))
        for source, start in ((wave, -1), (wave, 1.9), (wave[:, 0], 0)):
            with self.assertRaises(ValueError):
                codec.crop(source, start)

    def test_pairs_hold_other_conditions_fixed(self):
        rows = [
            r for r in codec.fit.source.conditions(True) if codec.fit.role(r) == "train"
        ]
        pairs = codec.response_pairs(rows)
        self.assertEqual(len(pairs), 30)
        self.assertEqual(sum(p[0] == "speed" for p in pairs), 18)
        index = {r["id"]: r for r in rows}
        for axis, low, high in pairs:
            a, b = index[low], index[high]
            self.assertEqual(a["texture_id"], b["texture_id"])
            fixed = (
                "commanded_normal_force_N"
                if axis == "speed"
                else "commanded_speed_mm_s"
            )
            self.assertEqual(a[fixed], b[fixed])
        for bad in (
            rows[:-1],
            [rows[1]] + rows[1:],
            [dict(rows[0], repeat=1)] + rows[1:],
        ):
            with self.assertRaises(ValueError):
                codec.response_pairs(bad)

    def test_response_does_not_confuse_constant_gain_with_slope(self):
        rows = [
            r for r in codec.fit.source.conditions(True) if codec.fit.role(r) == "train"
        ]
        records = []
        for row in rows:
            level = (
                row["commanded_speed_mm_s"] / 10 + 4 * row["commanded_normal_force_N"]
            )
            for channel in codec.CHANNELS:
                for arm in ("native", "shared"):
                    for variant, value in (
                        ("real", level),
                        ("mean", level + 8),
                        ("sample314", -level),
                    ):
                        records.append(
                            {
                                "id": row["id"],
                                "channel": channel,
                                "arm": arm,
                                "variant": variant,
                                "level_dbfs": value,
                            }
                        )
        result = codec.summarize(rows, records)
        json.dumps(result, allow_nan=False)
        for row in result:
            if row["variant"] == "mean":
                self.assertAlmostEqual(row["delta_mae_db"], 0)
                self.assertEqual(row["same_direction"], row["count"])
            else:
                self.assertEqual(row["same_direction"], 0)
                self.assertGreater(row["delta_mae_db"], 0)

    def test_pcm_no_individual_normalization_and_metrics(self):
        wave = np.random.default_rng(314).normal(0, 0.005, (codec.RATE, 2))
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "sound.wav"
            entry, pcm = codec.publish(path, wave)
            self.assertEqual(entry["sha256"], codec.sha(path))
            self.assertLess(float(np.max(np.abs(pcm - wave))), 2**-23)
            metrics = codec.metrics(pcm, pcm)
            self.assertEqual(metrics["shape_rmse_db"], 0)
            self.assertEqual(metrics["stereo_level_error_db"], 0)
            self.assertAlmostEqual(
                codec.metrics(pcm * 2, pcm)["stereo_level_error_db"], 6.0206, places=4
            )
            with self.assertRaises(ValueError):
                codec.publish(path, np.ones((100, 2)))
            entry, pcm = codec.publish(path, wave * 200, "FLOAT")
            self.assertGreater(entry["peak"], 0.98)


if __name__ == "__main__":
    unittest.main()
