import sys
import tempfile
import unittest
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_tangoflux_train as train


class TangoTrainTest(unittest.TestCase):
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
