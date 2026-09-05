import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import numpy as np
import torch
from safetensors.torch import save_file

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_pilot as pilot


class SonicGaussPilotTests(unittest.TestCase):
    def test_hash_mutation_rejects(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "source.py"
            path.write_text("x = 1\n")
            binding = {path.name: pilot.sha256(path)}
            pilot.verify(root, binding)
            path.write_text("x = 2\n")
            with self.assertRaisesRegex(ValueError, "corrupt"):
                pilot.verify(root, binding)

    def test_csr_matches_explicit_reductions(self):
        x = torch.tensor([[1.0, 2.0], [-3.0, 4.0], [9.0, 1.0], [6.0, 0.0], [3.0, 8.0]])
        pointers = torch.tensor([0, 2, 3, 5])
        for reduction in ["sum", "mean", "min", "max"]:
            expected = []
            for start, end in [(0, 2), (2, 3), (3, 5)]:
                operation = getattr(x[start:end], reduction)(dim=0)
                expected.append(
                    operation.values if reduction in {"min", "max"} else operation
                )
            actual = pilot.segment_csr(x, pointers, reduce=reduction)
            torch.testing.assert_close(actual, torch.stack(expected), rtol=0, atol=0)
        with self.assertRaises(ValueError):
            pilot.segment_csr(x, torch.tensor([0, 0, 5]))
        with self.assertRaises(ValueError):
            pilot.segment_csr(x, pointers, reduce="median")

    def test_ragged_attention_preserves_partition_scale_and_order(self):
        generator = torch.Generator().manual_seed(73)
        qkv = torch.randn(11, 3, 2, 4, generator=generator, dtype=torch.float64)
        offsets = torch.tensor([0, 3, 10, 11], dtype=torch.int32)
        actual = pilot.sdpa_varlen(qkv, offsets, 7, softmax_scale=0.37)
        expected = []
        for start, end in [(0, 3), (3, 10), (10, 11)]:
            q, k, v = qkv[start:end].permute(1, 2, 0, 3).unbind(0)
            value = ((q * 0.37) @ k.transpose(-1, -2)).softmax(-1) @ v
            expected.append(value.transpose(0, 1))
        torch.testing.assert_close(actual, torch.cat(expected), rtol=1e-12, atol=1e-12)
        changed = qkv.clone()
        changed[3:10] *= 12
        second = pilot.sdpa_varlen(changed, offsets, 7, softmax_scale=0.37)
        torch.testing.assert_close(actual[:3], second[:3], rtol=0, atol=0)
        for invalid in [
            torch.tensor([1, 11]),
            torch.tensor([0, 12]),
            torch.tensor([0, 0, 11]),
        ]:
            with self.assertRaises(ValueError):
                pilot.sdpa_varlen(qkv, invalid, 11)
        with self.assertRaises(ValueError):
            pilot.sdpa_varlen(qkv, offsets, 7, dropout_p=0.1)

    def test_select_definitions_does_not_execute_initializer(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.py").write_text(
                "raise RuntimeError('initializer')\ndef selected():\n    return 7\n"
            )
            namespace = {}
            pilot.definitions(root, "source.py", ["selected"], namespace)
            self.assertEqual(namespace["selected"](), 7)
            with self.assertRaisesRegex(ValueError, "missing reviewed"):
                pilot.definitions(root, "source.py", ["absent"], {})

    def test_strict_weights_only_unused_text_is_excluded(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            model = torch.nn.Linear(2, 1)
            tensors = {
                "weight": torch.ones(1, 2),
                "bias": torch.zeros(1),
                "text_encoder.unused": torch.zeros(1),
            }
            save_file(tensors, root / "valid.safetensors")
            with mock.patch.object(
                torch, "load", side_effect=AssertionError("no pickle")
            ):
                report = pilot.load_weights(
                    model, root / "valid.safetensors", ignore_text=True
                )
            self.assertEqual(
                report,
                {"loaded_keys": 2, "unused_text_encoder_keys": 1, "strict": True},
            )
            self.assertTrue(all(not x.requires_grad for x in model.parameters()))
            self.assertFalse(model.training)
            tensors["unknown"] = torch.zeros(1)
            save_file(tensors, root / "unexpected.safetensors")
            with self.assertRaises(RuntimeError):
                pilot.load_weights(
                    model, root / "unexpected.safetensors", ignore_text=True
                )
            save_file(
                {"weight": torch.full((1, 2), np.nan), "bias": torch.zeros(1)},
                root / "nonfinite.safetensors",
            )
            with self.assertRaisesRegex(ValueError, "nonfinite"):
                pilot.load_weights(model, root / "nonfinite.safetensors")


if __name__ == "__main__":
    unittest.main()
