import hashlib
import sys
import unittest
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_input_probe as probe


class InputProbeTests(unittest.TestCase):
    def test_hash_check_precedes_execution(self):
        with self.assertRaisesRegex(ValueError, "hash"):
            probe.load_definitions({"common.py": b"raise RuntimeError('EXECUTED')"})
        payload = b"reviewed source"
        self.assertEqual(
            probe.verified(payload, hashlib.sha256(payload).hexdigest()), payload
        )

    def test_input_comparison_does_not_hide_small_differences(self):
        a = {"x": torch.tensor([0.0, 1.0])}
        b = {"x": torch.tensor([0.0, 1.0 + 2**-23])}
        self.assertFalse(probe.input_changes(a, b)["x"]["exact_equal"])
        with self.assertRaisesRegex(ValueError, "keys"):
            probe.input_changes(a, {})

    def test_fixture_is_non_degenerate(self):
        fixture = probe.gaussian_fixture()
        self.assertEqual(
            set(fixture),
            {"means", "scales", "opacities", "quats", "features_dc", "features_rest"},
        )
        self.assertTrue(torch.all(fixture["means"].amax(0) > fixture["means"].amin(0)))

    def test_modal_control_frequency_intervention_and_bounds(self):
        profile = {
            "modes": [
                {
                    "damped_frequency_hz": 1000.0,
                    "damping_per_second": 20.0,
                    "ridge_amplitude": 0.1,
                }
            ]
        }
        for ratio, expected in ((1.0, 1000), (0.5, 500)):
            wave = probe.modal_control(profile, ratio)
            self.assertEqual(len(wave), 32_000)
            self.assertTrue(np.isfinite(wave).all())
            self.assertEqual(int(abs(np.fft.rfft(wave)).argmax()), expected)
        for ratio in (0.0, -1.0, 2.0, np.nan):
            with self.assertRaises(ValueError):
                probe.modal_control(profile, ratio)


if __name__ == "__main__":
    unittest.main()
