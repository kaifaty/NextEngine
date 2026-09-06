from __future__ import annotations

import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_dc_audit as audit


class DCAuditTests(unittest.TestCase):
    def test_dc_ac_energy_decomposition(self):
        x = np.tile([1.0, -1.0], 441)
        result = audit.components(x + 2)
        self.assertEqual(result["mean"], 2)
        self.assertEqual(result["total_power"], 5)
        self.assertEqual(result["ac_power"], 1)
        self.assertEqual(result["dc_power"], 4)
        self.assertEqual(result["dc_fraction"], 0.8)
        silent = audit.components(np.zeros(882))
        self.assertEqual(silent["dc_fraction"], 0)
        self.assertEqual(silent["total_dbfs"], -180)

    def test_global_offset_diagnostic_not_an_audio_gain(self):
        x = np.random.default_rng(1).normal(0, 0.01, 44100)
        result = audit.compare(x + 0.1, x)
        self.assertGreater(result["original_envelope_mae_db"], 15)
        self.assertLess(result["event_mean_removed_envelope_mae_db"], 1e-12)
        self.assertLess(result["block_means_removed_envelope_mae_db"], 1e-12)

    def test_invalid_requests(self):
        for x in (np.zeros(440), np.zeros((441, 2)), np.full(441, np.nan)):
            with self.assertRaises(ValueError):
                audit.components(x)
        with self.assertRaises(ValueError):
            audit.compare(np.zeros(441), np.zeros(442))
        with self.assertRaises(ValueError):
            audit.envelope(np.zeros(441), "tuned")

    def test_distribution_is_order_independent_not_a_timing_test(self):
        a = np.repeat(np.linspace(0.001, 0.01, 37), 441)
        b = a.reshape(37, 441)[::-1].ravel()
        a, b = [np.pad(x, (0, 16538 - len(x))) for x in (a, b)]
        result = audit.moving_comparison(a, b)
        self.assertEqual(result["bins"], 37)
        self.assertGreater(result["ordered_envelope_mae_db"], 5)
        self.assertEqual(result["envelope_distribution_w1_db"], 0)
        with self.assertRaises(ValueError):
            audit.moving_comparison(a[:-1], b[:-1])


if __name__ == "__main__":
    unittest.main()
