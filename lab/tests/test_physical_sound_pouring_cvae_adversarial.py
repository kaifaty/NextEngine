import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_cvae_adversarial as a


class AdversarialTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)
        torch.manual_seed(53)

    def test_hinge_and_feature_matching(self):
        real = [(torch.ones(1), [torch.ones(1, 2, requires_grad=True)])]
        features = torch.zeros(2, 2, requires_grad=True)
        fake = [(-torch.ones(2), [features])]
        self.assertEqual(float(a.discriminator_loss(real, fake)), 0)
        adv, fm = a.generator_losses(fake, real, 1)
        self.assertEqual(float(adv), 1)
        self.assertEqual(float(fm.detach()), 1)
        fm.backward()
        self.assertIsNone(real[0][1][0].grad)
        self.assertTrue(torch.equal(features.grad[1], torch.zeros(2)))
        self.assertTrue(torch.all(features.grad[0] < 0))

    def test_frozen_critic_passes_generator_gradients_only(self):
        critic = a.Critic().requires_grad_(False)
        wave = torch.randn(1, 1, 64, 64, requires_grad=True)
        outputs = critic(wave)
        sum(x[0].mean() for x in outputs).backward()
        self.assertTrue(torch.isfinite(wave.grad).all())
        self.assertGreater(float(wave.grad.abs().sum()), 0)
        self.assertTrue(all(p.grad is None for p in critic.parameters()))

    def test_step_changes_only_decoder_and_optional_critic(self):
        for adversarial in (False, True):
            with self.subTest(adversarial=adversarial):
                model, critic = a.v.PourCVAE(), a.Critic()
                trainable = a.freeze_representation(model)
                before = {k: v.clone() for k, v in model.state_dict().items()}
                critic_before = {k: v.clone() for k, v in critic.state_dict().items()}
                go = torch.optim.Adam(trainable, lr=1e-4)
                do = torch.optim.Adam(critic.parameters(), lr=1e-4)
                values = a.train_step(
                    model,
                    critic,
                    go,
                    do,
                    torch.randn(1, a.v.p.SAMPLES) * 0.01,
                    torch.tensor(a.v.phase.d.controls_for()[None]),
                    adversarial,
                )
                self.assertTrue(torch.isfinite(torch.tensor(values)).all())
                for name, parameter in model.named_parameters():
                    if not a.decoder_parameter(name):
                        self.assertTrue(torch.equal(parameter, before[name]))
                        self.assertIsNone(parameter.grad)
                self.assertFalse(
                    torch.equal(model.output.weight, before["output.weight"])
                )
                critic_changed = any(
                    not torch.equal(value, critic_before[name])
                    for name, value in critic.state_dict().items()
                )
                self.assertEqual(critic_changed, adversarial)
                self.assertTrue(all(p.grad is None for p in critic.parameters()))

    def test_probe_rejects_wrong_source_before_output(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            (source / "source.json").write_text("{}")
            with (
                patch.object(
                    a.v.p,
                    "load_source",
                    return_value=([{"role": "train", "item_id": "one"}], {}),
                ),
                patch.object(
                    a.v,
                    "load",
                    return_value=(
                        None,
                        {"source_sha256": "wrong", "train_ids": ["one"]},
                    ),
                ),
                self.assertRaisesRegex(ValueError, "source mismatch"),
            ):
                a.phase_budget_probe(source, source, source / "output", "cpu")
            self.assertFalse((source / "output").exists())


if __name__ == "__main__":
    unittest.main()
