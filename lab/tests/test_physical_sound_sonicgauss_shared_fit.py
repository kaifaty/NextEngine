import copy
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch
from scipy.io import wavfile
from torch import nn

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_shared_fit as shared
from test_physical_sound_sonicgauss_cohort import fixture


class DummyFusion(nn.Module):
    def forward(self, gs, pos):
        return torch.cat([gs, gs + pos[:, None]], dim=1)


class SharedFitTests(unittest.TestCase):
    def test_decision_rejects_subgroup_regression_and_incomplete_evidence(self):
        rows = [
            {
                "object_id": o,
                "contact_index": c,
                "local_fit_role": "development",
                "baseline": 1.0,
                "candidate": 0.5,
            }
            for o in (14, 75, 94, 97)
            for c in range(6)
        ]
        decision = shared.nonregression_decision(rows)
        self.assertEqual(decision["decision"], "REPORT_ONLY_NONREGRESSION_PASS")
        self.assertFalse(decision["physical_quality_validated"])
        for row in rows:
            if row["object_id"] == 75:
                row["candidate"] = 1.1
        self.assertEqual(shared.nonregression_decision(rows)["decision"], "REJECT")
        with self.assertRaisesRegex(ValueError, "incomplete"):
            shared.nonregression_decision(rows[:-1])
        rows[0]["candidate"] = float("nan")
        with self.assertRaisesRegex(ValueError, "invalid"):
            shared.nonregression_decision(rows)

    def test_zero_initialization_exact_and_shared_gradients(self):
        torch.manual_seed(42)
        adapter = shared.ContactResidual(width=16, rank=4)
        original = DummyFusion()
        model = shared.AdaptedFusion(original, adapter)
        gs, pos = torch.randn(2, 8, 16), torch.randn(2, 16)
        self.assertTrue(torch.equal(model(gs, pos), original(gs, pos)))
        optimizer = torch.optim.Adam(adapter.parameters(), lr=0.001)
        model(gs, pos).square().mean().backward()
        self.assertGreater(adapter.output.weight.grad.norm().item(), 0)
        optimizer.step()
        optimizer.zero_grad()
        model(gs, pos).square().mean().backward()
        for parameter in adapter.parameters():
            self.assertGreater(parameter.grad.norm().item(), 0)
        self.assertTrue(torch.equal(model(gs, pos)[:, :8], gs))

    def test_geometry_and_contact_counterfactuals(self):
        torch.manual_seed(43)
        adapter = shared.ContactResidual(width=16, rank=4)
        nn.init.normal_(adapter.output.weight)
        gs, pos = torch.randn(2, 8, 16), torch.randn(2, 16)
        result = adapter(gs, pos)
        self.assertFalse(torch.equal(result, adapter(gs, pos.flip(0))))
        self.assertFalse(torch.equal(result, adapter(gs.flip(0), pos)))
        torch.testing.assert_close(result, adapter(gs.flip(1), pos))

    def test_role_swap_and_protected_roster_reject(self):
        records, projection = fixture()
        rows = shared.cohort.select_records(records, projection)
        shared.validate_inputs({"objects": rows})
        bad = copy.deepcopy(rows)
        bad[0]["local_fit_role"] = "development"
        with self.assertRaisesRegex(ValueError, "role"):
            shared.validate_inputs({"objects": bad})
        bad = copy.deepcopy(rows)
        bad[0]["object_id"] = 41
        with self.assertRaisesRegex(ValueError, "roster"):
            shared.validate_inputs({"objects": bad})

    def test_teacher_preserves_stereo_level_and_bounds(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.wav"
            wave = np.stack(
                [np.full(144000, 1000, np.int16), np.full(144000, -2000, np.int16)],
                axis=1,
            )
            wavfile.write(path, 48000, wave)
            value = shared.teacher_audio(path, shared.pilot.sha256(path))
            self.assertEqual(value.shape, (2, 131418))
            self.assertAlmostEqual(
                float(value[0, 1000:-1000].mean()), 1000 / 32768, places=5
            )
            self.assertAlmostEqual(
                float(value[1, 1000:-1000].mean()), -2000 / 32768, places=5
            )
            with self.assertRaisesRegex(ValueError, "hash"):
                shared.teacher_audio(path, "wrong")


if __name__ == "__main__":
    unittest.main()
