from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import numpy as np
import torch
from safetensors.torch import save_file

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_flow as flow


class TextureFlowTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)

    def test_normalization_includes_posterior_variance_and_rejects_bad_tensors(self):
        mean = torch.ones(24, 64, flow.FRAMES) * 2
        std = torch.ones_like(mean) * 3
        center, scale = flow.normalization(mean, std)
        torch.testing.assert_close(center, torch.ones(1, 64, 1) * 2)
        torch.testing.assert_close(scale, torch.ones(1, 64, 1) * 3)
        for bad in (std * -1, std[:, :, :-1], std * float("nan")):
            with self.assertRaises(ValueError):
                flow.normalization(mean, bad)

    def test_network_shape_and_gradient(self):
        model = flow.TextureFlow()
        x = torch.randn(2, 64, flow.FRAMES)
        physical = torch.tensor(np.stack([flow.spectrum.features(74, 40, 0.5)] * 2))
        output = model(x, torch.ones(2) * 0.5, physical)
        self.assertEqual(output.shape, x.shape)
        self.assertEqual(float(output.detach().abs().max()), 0)
        (output - x).square().mean().backward()
        self.assertTrue(torch.isfinite(model.output.weight.grad).all())
        self.assertGreater(float(model.output.weight.grad.abs().sum()), 0)

    def test_source_free_sampling_and_common_noise(self):
        model = flow.TextureFlow()
        features = np.stack(
            [flow.spectrum.features(74, 40, 0.5), flow.spectrum.features(0, 60, 1)]
        )
        with (
            patch.object(
                flow.sf, "read", side_effect=AssertionError("no source audio")
            ),
            patch.object(
                flow.spectrum,
                "load_data",
                side_effect=AssertionError("no source manifest"),
            ),
        ):
            first = flow.sample(model, features, 314)
            repeated = flow.sample(model, features, 314)
            other = flow.sample(model, features, 2718)
        torch.testing.assert_close(first, repeated, rtol=0, atol=0)
        torch.testing.assert_close(first[0], first[1], rtol=0, atol=0)
        self.assertFalse(torch.equal(first, other))
        # Zero field integrates to the exact initial noise, with no source target.
        expected = torch.randn(
            (1, 64, flow.FRAMES), generator=torch.Generator().manual_seed(314)
        )
        torch.testing.assert_close(first[:1], expected, rtol=0, atol=0)

    def test_domain_and_window_bounds(self):
        good = flow.spectrum.features(74, 40, 0.5)[None]
        flow.validate_controls(good)
        for bad in (
            good[:, :4],
            good * float("nan"),
            np.ones((1, 5)),
            np.array([[0, 0, 1, 1.1, 0]]),
        ):
            with self.assertRaises(ValueError):
                flow.validate_controls(bad)
        first = flow.window_start({"crop_start_seconds": 1.0}, 100)
        self.assertGreaterEqual(first, 0)
        for row, frames in (
            ({"crop_start_seconds": -1}, 100),
            ({"crop_start_seconds": 1}, 20),
        ):
            with self.assertRaises(ValueError):
                flow.window_start(row, frames)

    def test_checkpoint_identity_and_scale(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            model = flow.TextureFlow()
            save_file(model.state_dict(), root / "model.safetensors")
            meta = {
                "format": flow.FORMAT,
                "codec_sha256": flow.CODEC_SHA,
                "input_gain": flow.GAIN,
                "checkpoint_sha256": flow.codec.sha(root / "model.safetensors"),
            }
            (root / "model.json").write_text(json.dumps(meta))
            loaded, _ = flow.load_model(root, "cpu")
            self.assertFalse(loaded.training)
            meta["input_gain"] = 1
            (root / "model.json").write_text(json.dumps(meta))
            with self.assertRaises(ValueError):
                flow.load_model(root, "cpu")

    def test_core_publication_and_temporal_metrics(self):
        native = np.random.default_rng(314).normal(
            0, 0.001, (flow.FRAMES * flow.HOP, 2)
        )
        with tempfile.TemporaryDirectory() as tmp:
            row, pcm = flow.publish_decode(Path(tmp), "sound", native)
            self.assertEqual(len(pcm), flow.CORE)
            self.assertEqual(row["full"]["frames"], flow.FRAMES * flow.HOP)
            expected = (
                native[flow.CONTEXT * flow.HOP : flow.CONTEXT * flow.HOP + flow.CORE]
                * flow.PLAYBACK
            )
            self.assertLess(float(np.max(np.abs(pcm - expected))), 2**-23)
            metrics = flow.waveform_metrics(pcm, pcm)
            self.assertEqual(metrics["envelope_acf_mae"], 0)
            self.assertEqual(metrics["absolute_level_error_db"], 0)

    def test_legacy_baseline_uses_exact_recorded_roles(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            save_file(
                flow.spectrum.SpectrumNet(4).state_dict(), root / "model.safetensors"
            )
            meta = {
                "format": "texture-spectrum-v1",
                "rank": 4,
                "corpus_sha256": "source",
                "checkpoint_sha256": flow.codec.sha(root / "model.safetensors"),
            }
            flow.save(root / "model.json", meta)
            rows = [
                {**r, "role": flow.spectrum.role(r)}
                for r in flow.spectrum.source.conditions(True)
            ]
            report = {"status": "complete", "model": meta, "rows": rows}
            flow.save(root / "result.json", report)
            _, checked = flow.load_baseline(root, "source")
            self.assertEqual(
                checked["verified_role_report_sha256"],
                flow.codec.sha(root / "result.json"),
            )
            rows[0]["role"] = "unseen_speed"
            flow.save(root / "result.json", report)
            with self.assertRaises(ValueError):
                flow.load_baseline(root, "source")

    def test_paired_response_separates_repeats_and_constant_gain(self):
        conditions = flow.spectrum.source.conditions(True)
        report = {"conditions": conditions, "references": [], "rows": []}
        for row in conditions:
            level = row["commanded_speed_mm_s"] / 10 + row["commanded_normal_force_N"]
            report["references"].append(
                {"id": row["id"], "real": {"level_dbfs": level}}
            )
            for variant in ("flow", "previous_spectrum", "interpolation"):
                for seed in (314, 2718):
                    report["rows"].append(
                        {
                            "id": row["id"],
                            "variant": variant,
                            "seed": seed,
                            "level_dbfs": level + 10,
                        }
                    )
        output = flow.paired_responses(report)
        self.assertEqual(len(output), 24)
        for row in output:
            self.assertEqual(row["count"], 24 if row["axis"] == "speed" else 15)
            self.assertAlmostEqual(row["delta_mae_db"], 0)
            self.assertEqual(row["same_direction"], row["count"])

    def test_shared_band_envelope_metric(self):
        wave = np.random.default_rng(314).normal(size=16384)
        np.testing.assert_allclose(
            flow.envelope_acf_mono(wave), flow.envelope_acf_mono(wave * 10), atol=1e-12
        )
        np.testing.assert_array_equal(
            flow.envelope_acf_mono(np.ones(16384)), np.zeros(4)
        )
        with self.assertRaises(ValueError):
            flow.envelope_acf_mono(wave[:100])

    def test_standalone_render_reads_only_weights_and_its_own_pcm(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            save_file(flow.TextureFlow().state_dict(), root / "model.safetensors")
            meta = {
                "format": flow.FORMAT,
                "codec_sha256": flow.CODEC_SHA,
                "input_gain": flow.GAIN,
                "checkpoint_sha256": flow.codec.sha(root / "model.safetensors"),
            }
            flow.save(root / "model.json", meta)
            for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
                (root / name).write_text("test notice")
            vae = SimpleNamespace(
                decode=lambda latent: SimpleNamespace(
                    sample=latent[:, :2].repeat_interleave(flow.HOP, dim=2) * 0.001
                )
            )
            original_read = flow.sf.read
            output = root / "generated"

            def read(path, *args, **kwargs):
                self.assertTrue(Path(path).is_relative_to(output))
                return original_read(path, *args, **kwargs)

            with (
                patch.object(flow, "load_codec", return_value=(vae, root)),
                patch.object(
                    flow, "prepare", side_effect=AssertionError("no reference input")
                ),
                patch.object(flow.sf, "read", side_effect=read),
            ):
                result = flow.render(root, output, 74, 40, 0.5, 314, device="cpu")
            self.assertFalse(result["reference_audio_input"])
            self.assertTrue((output / "generated.wav").exists())


if __name__ == "__main__":
    unittest.main()
