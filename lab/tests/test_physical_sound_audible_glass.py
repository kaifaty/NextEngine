"""Focused reconstruction, training, publication and failure tests."""

import json
import math
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

import numpy as np
import torch
from scipy.io import wavfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_audible_glass as glass


def tone_parameters(frequency=1000.0):
    modal = np.zeros((glass.MODES, 5))
    modal[:, 0] = glass.logit(
        np.log(np.linspace(200, 10000, glass.MODES) / 80) / math.log(15000 / 80)
    )
    modal[0, 0] = glass.logit(np.log(frequency / 80) / math.log(15000 / 80))
    modal[:, 1] = glass.logit(np.log(0.5 / 0.02) / math.log(4 / 0.02))
    modal[:, 2] = -25
    modal[0, 2] = np.log(np.expm1(0.2))
    modal[:, 4] = 1
    return torch.tensor(
        np.r_[modal.ravel(), np.full(glass.BANDS, -25)], dtype=torch.float32
    )


class AudibleGlassTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        torch.set_num_threads(2)
        torch.manual_seed(42)

    def test_frequency_decay_and_gradient(self):
        synth = glass.Synthesizer()
        parameters = tone_parameters().requires_grad_()
        audio = synth(parameters, [0])[0]
        hz = np.fft.rfftfreq(len(audio), 1 / glass.RATE)
        peak = hz[np.argmax(abs(np.fft.rfft(audio.detach().numpy())))]
        self.assertAlmostEqual(peak, 1000, delta=1)
        first = audio[0:640].square().mean().sqrt()
        later = audio[8000:8640].square().mean().sqrt()
        self.assertAlmostEqual(float((later / first).detach()), 10**-1.5, delta=0.001)
        audio.square().mean().backward()
        self.assertTrue(torch.isfinite(parameters.grad).all())
        self.assertGreater(float(parameters.grad.abs().max()), 0)

    def test_encoder_updates_reload_and_wav(self):
        synth = glass.Synthesizer()
        params = torch.stack([tone_parameters(f) for f in (900, 1100, 1300)])
        targets = synth(params, [0, 0, 0]).detach()
        inputs = glass.features(targets)
        encoder = glass.Encoder(inputs, params)
        before = encoder.net[0].weight.detach().clone()
        objective = lambda: (
            ((encoder(inputs) - params) / encoder.output_scale).square().mean()
        )
        result = glass.optimize(
            encoder,
            objective,
            torch.optim.Adam(encoder.parameters(), lr=0.001),
            20,
            time.monotonic() + 20,
            10,
            lambda step, value: None,
        )
        self.assertLess(result["best_loss"], result["initial_loss"])
        self.assertFalse(torch.equal(before, encoder.net[0].weight))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            torch.save(encoder.state_dict(), root / "model.pt")
            restored = glass.Encoder(inputs, params)
            restored.load_state_dict(torch.load(root / "model.pt", weights_only=True))
            with torch.no_grad():
                actual = synth(encoder(inputs), [0, 0, 0])
                self.assertTrue(torch.equal(actual, synth(restored(inputs), [0, 0, 0])))
            bundle = glass.publish_audio(root, targets, targets, actual)
            rate, audio = wavfile.read(bundle["paths"]["761160-neural"])
            self.assertEqual(rate, glass.RATE)
            self.assertEqual(len(audio), glass.SAMPLES)
            self.assertGreater(abs(audio.astype(float)).max(), 0)
            self.assertLess(abs(audio.astype(float)).max(), 32767)

    def test_invalid_input_and_output(self):
        for value in (np.array([]), np.zeros(100), np.array([np.nan]), np.ones((2, 3))):
            with self.assertRaises(ValueError):
                glass.validate_audio(value)
        with tempfile.TemporaryDirectory() as directory:
            broken = Path(directory) / "broken.mp3"
            broken.write_bytes(b"not audio")
            with self.assertRaises(ValueError):
                glass.decode(broken)
            with self.assertRaises(ValueError):
                glass.run(Path(directory), Path(directory), 1)

    def test_time_limit_and_interrupt_publish_best(self):
        for interrupt in (False, True):
            model = torch.nn.Linear(1, 1)
            calls, published = [], []

            def objective(calls=calls, interrupt=interrupt, model=model):
                calls.append(1)
                if interrupt and len(calls) == 4:
                    raise KeyboardInterrupt
                return model(torch.ones(1, 1)).square().mean()

            result = glass.optimize(
                model,
                objective,
                torch.optim.Adam(model.parameters(), lr=0.01),
                20,
                time.monotonic() + (20 if interrupt else -1),
                10,
                lambda step, value, published=published: published.append(
                    (step, value)
                ),
            )
            self.assertEqual(
                result["status"], "interrupted" if interrupt else "time_limit"
            )
            self.assertEqual(published[-1][1], result["best_loss"])
            self.assertTrue(all(math.isfinite(value) for _, value in published))

    def test_full_run_exports_on_short_budget(self):
        params = torch.stack([tone_parameters(f) for f in (900, 1100, 1300)])
        targets = glass.Synthesizer()(params, [0, 0, 0]).detach()
        records = [{"sound_id": sound_id} for sound_id, _, _ in glass.SOURCES]
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "run"
            with (
                mock.patch.object(
                    glass, "load_sources", return_value=(targets, [0, 0, 0], records)
                ),
                mock.patch.object(glass, "initialize", side_effect=list(params)),
            ):
                result = glass.run(Path(directory), output, 0.001)
            self.assertTrue(result["reload_exact"])
            self.assertFalse(result["technical_success"])
            self.assertEqual(result["encoder"]["status"], "time_limit")
            self.assertTrue((output / "encoder.pt").is_file())
            self.assertEqual(
                json.loads((output / "result.json").read_text())["status"], "partial"
            )
            for path in result["audio"]["paths"].values():
                self.assertTrue(Path(path).is_file())


if __name__ == "__main__":
    unittest.main()
