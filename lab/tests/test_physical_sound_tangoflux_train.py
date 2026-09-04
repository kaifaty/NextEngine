import copy
import sys
import tempfile
import unittest
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_tangoflux_train as train


class TangoTrainTest(unittest.TestCase):
    def test_multi_event_corpus_roles_and_captions(self):
        rows = [
            {
                "filename": f"{fold}-{index}.wav",
                "category": key,
                "prompt": prompt,
                "fold": fold,
                "role": "train" if fold == "1" else "development",
                "src_file": f"{fold}-{index}",
                "physical_attributes": None,
            }
            for fold in ("1", "5")
            for index, (key, prompt) in enumerate(train.water.PROMPTS.items())
        ]
        self.assertEqual(train.corpus_split(rows), ([0, 1, 2], [3, 4, 5]))
        for key, value in (
            ("prompt", "invented exact flow rate"),
            ("role", "train"),
            ("physical_attributes", {"force": 1}),
            ("src_file", rows[0]["src_file"]),
            ("fold", "0"),
        ):
            invalid = copy.deepcopy(rows)
            invalid[3][key] = value
            with self.assertRaises(ValueError):
                train.corpus_split(invalid)
        with self.assertRaises(ValueError):
            train.corpus_split(rows + rows[:1])

    def test_five_second_active_region_not_glass_duration(self):
        target = torch.zeros(1, 645, 64)
        prediction = target.clone()
        prediction[:, 33:108] = 1
        parts = train.loss_parts(prediction, target, 108)
        self.assertAlmostEqual(float(parts["active"]), 75 / 108, places=6)
        self.assertEqual(float(parts["tail"]), 0)

    def test_active_sound_not_diluted_by_padding(self):
        target = torch.zeros(1, 645, 64)
        prediction = target.clone()
        prediction[:, :33] = 2
        parts = train.loss_parts(prediction, target)
        self.assertEqual(float(parts["active"]), 4)
        self.assertEqual(float(parts["tail"]), 0)
        self.assertEqual(float(parts["balanced"]), 2)
        self.assertAlmostEqual(float(parts["full"]), 4 * 33 / 645, places=6)

    def test_padding_also_penalized_and_gradient_reaches_both_regions(self):
        prediction = torch.ones(1, 645, 64, requires_grad=True)
        parts = train.loss_parts(prediction, torch.zeros_like(prediction))
        parts["balanced"].backward()
        self.assertTrue((prediction.grad > 0).all())
        self.assertAlmostEqual(float(prediction.grad[:, :33].sum()), 1, places=5)
        self.assertAlmostEqual(float(prediction.grad[:, 33:].sum()), 1, places=5)

    def test_uniform_control_matches_direct_mse_without_region_reweighting(self):
        target = torch.zeros(1, 645, 64)
        prediction = torch.ones_like(target, requires_grad=True)
        full = train.loss_parts(prediction, target)["full"]
        self.assertEqual(
            float(full.detach()), float((prediction - target).square().mean().detach())
        )
        full.backward()
        self.assertTrue(
            torch.equal(
                prediction.grad, torch.full_like(prediction, 2 / prediction.numel())
            )
        )
        self.assertAlmostEqual(
            float(prediction.grad[:, :33].sum()), 2 * 33 / 645, places=6
        )

    def test_invalid_shapes_and_regions(self):
        target = torch.zeros(1, 645, 64)
        for frames in (0, 645, 646):
            with self.assertRaises(ValueError):
                train.loss_parts(target, target, frames)
        with self.assertRaises(ValueError):
            train.loss_parts(target, target[:, :33])

    def test_changed_source_rejected_before_decode(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "761160_13431397-hq.mp3").write_bytes(b"not the source")
            with self.assertRaisesRegex(ValueError, "source hash mismatch"):
                train.prepare_sources(root)


if __name__ == "__main__":
    unittest.main()
