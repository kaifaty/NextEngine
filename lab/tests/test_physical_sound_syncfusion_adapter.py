import io
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_syncfusion_adapter as adapter


class AdapterTests(unittest.TestCase):
    def test_structured_features_and_ood(self):
        self.assertEqual(
            adapter.features("glass", "rigid-motion").tolist(), [1, 0, 0, 0, 1]
        )
        for material, motion in [("rubber", "static"), ("wood", "deform")]:
            with self.assertRaises(ValueError):
                adapter.features(material, motion)

    def test_neural_interface_is_finite_unit_length_without_audio(self):
        model = adapter.Adapter()
        value = model(
            torch.stack(
                [
                    adapter.features("glass", "static"),
                    adapter.features("wood", "rigid-motion"),
                ]
            )
        )
        self.assertEqual(tuple(value.shape), (2, 512))
        self.assertTrue(torch.isfinite(value).all())
        torch.testing.assert_close(
            torch.linalg.vector_norm(value, dim=-1), torch.ones(2)
        )

    def test_invalid_event_times_fail_before_filesystem_or_model_load(self):
        missing = Path("/nonexistent-syncfusion-test-input")
        with self.assertRaises(ValueError):
            adapter.render(missing, missing, missing, times=[-1])

    def test_combination_holdout_excludes_entire_recording(self):
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode="w") as archive:
            annotation = b"0,glass hit rigid-motion\n0.5,wood hit static\n1,metal hit static\n1.5,None None None\n"
            info = tarfile.TarInfo("train/held.times.csv")
            info.size = len(annotation)
            archive.addfile(info, io.BytesIO(annotation))
        buffer.seek(0)
        with tarfile.open(fileobj=buffer, mode="r:") as archive:
            rows, _ = adapter.selection(archive)
        self.assertEqual(len(rows), 3)
        self.assertEqual({r["role"] for r in rows}, {"combination-dev"})

    def test_prototype_cannot_consume_development_targets(self):
        rows = [
            {"role": "train", "material": "glass", "motion": "static"},
            {"role": "combination-dev", "material": "glass", "motion": "rigid-motion"},
        ]
        targets = torch.tensor([[1.0, 0.0], [0.0, 1.0]])
        actual, kind = adapter.prototype(rows, targets, "glass", "rigid-motion")
        torch.testing.assert_close(actual, targets[0])
        self.assertEqual(kind, "material-only-fallback")
        with self.assertRaises(ValueError):
            adapter.prototype(rows, targets, "wood", "static")

    def test_development_target_change_cannot_change_fit_or_prototypes(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            rows = [
                {
                    "recording": f"train/{m}-{motion}",
                    "role": "train",
                    "material": m,
                    "motion": motion,
                }
                for m in adapter.MATERIALS
                for motion in adapter.MOTIONS
                if (m, motion) != ("glass", "rigid-motion")
            ] + [
                {
                    "recording": "train/held",
                    "role": "combination-dev",
                    "material": "glass",
                    "motion": "rigid-motion",
                }
            ]
            torch.manual_seed(12)
            target = torch.nn.functional.normalize(torch.randn(len(rows), 512), dim=-1)
            saved = []
            for variant in range(2):
                data, output = root / f"data-{variant}", root / f"fit-{variant}"
                data.mkdir()
                changed = target.clone()
                if variant:
                    changed[-1] *= -1
                torch.save(changed, data / "embeddings.pt")
                adapter.pilot.save(
                    data / "data.json",
                    {
                        "status": "complete",
                        "rows": rows,
                        "checkpoint_sha256": "synthetic-test",
                        "embeddings_sha256": adapter.pilot.sha(data / "embeddings.pt"),
                    },
                )
                adapter.fit(data, output)
                saved.append(torch.load(output / "adapter.pt", weights_only=True))
            for group in ("state_dict", "prototypes"):
                for name in saved[0][group]:
                    self.assertTrue(
                        torch.equal(saved[0][group][name], saved[1][group][name])
                    )


if __name__ == "__main__":
    unittest.main()
