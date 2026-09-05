import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_neuralresonator_pilot as pilot


class NeuralResonatorTests(unittest.TestCase):
    def test_checkpoint_hash_before_loading(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.ckpt"
            path.write_bytes(b"untrusted")
            with mock.patch.object(torch, "load") as loader:
                with self.assertRaisesRegex(ValueError, "hash"):
                    pilot.load_checkpoint(path)
                loader.assert_not_called()

    def test_inert_metadata_does_not_call_embedded_function(self):
        function = mock.Mock(side_effect=AssertionError("must not execute"))
        carrier = pilot.InertMetadata(function)
        carrier.__setstate__((function, (), {}, None))
        function.assert_not_called()

    def test_unknown_global_rejected_before_loading(self):
        with (
            mock.patch.object(pilot, "checked_bytes"),
            mock.patch.object(
                torch.serialization,
                "get_unsafe_globals_in_checkpoint",
                return_value=["unknown.code"],
            ),
            mock.patch.object(torch, "load") as loader,
        ):
            with self.assertRaisesRegex(ValueError, "globals"):
                pilot.load_checkpoint(Path("unused"))
            loader.assert_not_called()

    def test_material_ranges_and_shape(self):
        torch.testing.assert_close(
            pilot.material_input(pilot.BASE), torch.full((5,), 0.5)
        )
        for value in ([0] * 5, [float("nan")] * 5, [1, 2]):
            with self.assertRaises(ValueError):
                pilot.material_input(value)
        normal, narrow = pilot.shape_mask(), pilot.shape_mask(True)
        self.assertEqual(normal.shape, (64, 64))
        self.assertLess(float(narrow.sum()), float(normal.sum()))
        self.assertEqual(float(narrow[32, 32]), 1)

    def test_filter_identity_and_instability(self):
        coefficients = np.zeros((32, 2, 6))
        coefficients[..., 0] = coefficients[..., 3] = 1
        wave, radius = pilot.render_coefficients(coefficients)
        self.assertEqual(radius, 0)
        self.assertEqual(wave[0], 32)
        self.assertEqual(np.count_nonzero(wave), 1)
        coefficients[0, 0, 4] = -1.1
        with self.assertRaisesRegex(ValueError, "unstable"):
            pilot.render_coefficients(coefficients)

    def test_full_waveform_observation(self):
        time = np.arange(pilot.RATE) / pilot.RATE
        wave = np.sin(2 * np.pi * 800 * time) * np.exp(-20 * time)
        result = pilot.waveform_observations(wave)
        self.assertEqual(result["dominant_hz"], 800)
        self.assertAlmostEqual(result["energy_centroid_seconds"], 0.025, places=4)
        with self.assertRaisesRegex(ValueError, "full"):
            pilot.waveform_observations(wave[:400])


if __name__ == "__main__":
    unittest.main()
