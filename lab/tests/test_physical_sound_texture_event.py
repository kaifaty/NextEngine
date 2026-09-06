from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_event as event


class TextureEventTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)

    def test_constant_condition_matches_time_broadcast(self):
        torch.manual_seed(23)
        model = event.flow.TextureFlow()
        torch.nn.init.normal_(model.output.weight, std=0.01)
        x = torch.randn(2, 64, 32)
        t = torch.ones(2) * 0.5
        physical = torch.tensor(
            np.stack([event.flow.spectrum.features(74, 40, 0.5)] * 2)
        )
        a = model(x, t, physical)
        b = model(x, t, physical[:, :, None].expand(-1, -1, 32))
        torch.testing.assert_close(a, b, rtol=1e-5, atol=1e-6)

    def test_profile_distance_and_quiet_boundaries(self):
        physical, request = event.profile()
        speed = physical[3] * 20 + 40
        self.assertAlmostEqual(
            float(speed.sum() * event.HOP / event.RATE), 90, delta=0.1
        )
        self.assertEqual(speed[0], 0)
        self.assertEqual(speed[-1], 0)
        self.assertAlmostEqual(request["duration_seconds"], 3.15)
        for kwargs in (
            {"speed": 61},
            {"distance": 20},
            {"tail": 0},
            {"start": float("nan")},
        ):
            with self.assertRaises(ValueError):
                event.profile(**kwargs)

    def test_domain_and_average_speed(self):
        pos = np.zeros(301, dtype=[("time", float), ("Y", float)])
        pos["time"] = np.linspace(0, 3, 301)
        pos["Y"] = 45 - 30 * pos["time"]
        np.testing.assert_allclose(
            event.position_speed(pos, np.linspace(0.1, 2.9, 32)), 30
        )
        for speed, force in (
            (np.full(32, 81), np.ones(32)),
            (np.zeros(32), np.full(32, 1.6)),
            (np.full(32, np.nan), np.ones(32)),
        ):
            with self.assertRaises(ValueError):
                event.trace_features(74, speed, force)

    def test_generation_needs_no_audio_or_sensor_files(self):
        physical, _ = event.profile()
        model = event.flow.TextureFlow()
        vae = SimpleNamespace(
            decode=lambda z: SimpleNamespace(
                sample=z[:, :2].repeat_interleave(event.HOP, dim=2) * 0.001
            )
        )
        with (
            patch.object(event.sf, "read", side_effect=AssertionError("no reference")),
            patch.object(
                event.np, "genfromtxt", side_effect=AssertionError("no sensors")
            ),
        ):
            a = event.generate(model, vae, physical, 314)
            b = event.generate(model, vae, physical, 314)
        np.testing.assert_array_equal(a, b)
        self.assertEqual(a.shape, (physical.shape[1] * event.HOP, 2))
        physical[0, 20] = 1
        with self.assertRaises(ValueError):
            event.generate(model, vae, physical, 314)

    def test_event_metrics_timing_and_weak_source(self):
        physical, request = event.profile()
        n = round(request["duration_seconds"] * event.RATE)
        times = (np.arange(n) + 0.5) / event.RATE
        v = np.interp(
            times,
            (np.arange(physical.shape[1]) + 0.5) * event.HOP / event.RATE,
            physical[3] * 20 + 40,
        )
        noise = np.random.default_rng(314).normal(size=n)
        wave = noise * (0.001 + 0.01 * np.sqrt(v / 40))
        real = np.repeat(wave[:, None], 2, axis=1)
        metrics = event.event_metrics(real, real, physical, 40)
        json.dumps(metrics, allow_nan=False)
        self.assertEqual(metrics["envelope_db_mae"], 0)
        self.assertEqual(metrics["onset_error_seconds"], 0)
        self.assertEqual(metrics["offset_error_seconds"], 0)
        delayed = np.pad(real, ((round(0.2 * event.RATE), 0), (0, 0)))[:n]
        self.assertGreater(
            event.event_metrics(delayed, real, physical, 40)["onset_error_seconds"],
            0.15,
        )
        quiet = np.repeat((noise * 0.001)[:, None], 2, axis=1)
        weak = event.event_metrics(quiet, quiet, physical, 40)
        json.dumps(weak, allow_nan=False)
        self.assertIsNone(weak["onset_error_seconds"])
        silent = event.event_metrics(np.zeros_like(real), real, physical, 40)
        json.dumps(silent, allow_nan=False)
        self.assertIsNone(silent["envelope_correlation"])

    def test_full_publication_has_one_gain_and_keeps_horizon(self):
        wave = np.random.default_rng(314).normal(0, 0.001, (40 * event.HOP, 2))
        with tempfile.TemporaryDirectory() as tmp:
            row, pcm = event.publish(Path(tmp), "event", wave, 80000)
            self.assertEqual(len(pcm), 80000)
            self.assertEqual(row["full"]["frames"], len(wave))
            self.assertLess(
                float(np.max(np.abs(pcm - wave[:80000] * event.PLAYBACK))), 2**-23
            )


if __name__ == "__main__":
    unittest.main()
