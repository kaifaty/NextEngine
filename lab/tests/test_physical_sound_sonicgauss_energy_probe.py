import sys
import unittest
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_energy_probe as probe


class EnergyProbeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        torch.set_num_threads(4)
        t = torch.arange(probe.LENGTH) / 44100
        cls.waves = [
            (0.03 * torch.sin(2 * torch.pi * frequency * t) * torch.exp(-8 * t))[
                None, None
            ].repeat(1, 2, 1)
            for frequency in (700, 1400, 2800)
        ]

    def test_symmetry_identity_and_finite_gradients(self):
        a = probe.features(self.waves[0])
        b = probe.features(self.waves[1])
        self.assertEqual(probe.distance(a, a).item(), 0)
        self.assertEqual(probe.distance(a, b).item(), probe.distance(b, a).item())
        gain = torch.tensor(1.0, requires_grad=True)
        score, parts = probe.energy(a, probe.features(self.waves[1] * gain), b)
        gradient = torch.autograd.grad(score, gain)[0]
        self.assertTrue(torch.isfinite(gradient))
        self.assertEqual(parts["repulsion"].item(), 0)
        self.assertGreater(score.item(), 0)

    def test_empirical_distribution_beats_silence_and_single_tone(self):
        tones = [probe.features(w) for w in self.waves]
        silent = probe.features(torch.zeros_like(self.waves[0]))
        correct = torch.stack(
            [probe.energy(x, y, z)[0] for x in tones for y in tones for z in tones]
        ).mean()
        for collapsed in (silent, tones[0]):
            wrong = torch.stack(
                [probe.energy(x, collapsed, collapsed)[0] for x in tones]
            ).mean()
            self.assertLess(correct.item(), wrong.item())

    def test_correct_quiet_output_is_pushed_up_not_down(self):
        target = probe.features(self.waves[0])
        gain = torch.tensor(0.5, requires_grad=True)
        predicted = probe.features(self.waves[0] * gain)
        loss = probe.energy(target, predicted, predicted)[0]
        self.assertLess(torch.autograd.grad(loss, gain)[0].item(), 0)

    def test_both_independent_branches_receive_finite_gradients(self):
        first = self.waves[1].clone().requires_grad_(True)
        second = self.waves[2].clone().requires_grad_(True)
        value, _ = probe.energy(
            probe.features(self.waves[0]), probe.features(first), probe.features(second)
        )
        for grad in torch.autograd.grad(value, (first, second)):
            self.assertTrue(torch.isfinite(grad).all())
            self.assertGreater(grad.norm().item(), 0)

    def test_correct_overloud_output_is_pushed_down(self):
        target = probe.features(self.waves[0])
        gain = torch.tensor(2.0, requires_grad=True)
        predicted = probe.features(self.waves[0] * gain)
        loss = probe.energy(target, predicted, predicted)[0]
        self.assertGreater(torch.autograd.grad(loss, gain)[0].item(), 0)

    def test_padding_keeps_full_signal_and_rejects_overlong(self):
        original = self.waves[0][..., :131072]
        padded = torch.nn.functional.pad(original, (0, probe.LENGTH - 131072))
        self.assertEqual(
            probe.distance(probe.features(original), probe.features(padded)).item(), 0
        )
        with self.assertRaises(ValueError):
            probe.features(torch.zeros(1, 2, probe.LENGTH + 1))


if __name__ == "__main__":
    unittest.main()
