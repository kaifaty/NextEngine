import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
from scipy.io import wavfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_bridge_compare as c


class CompareTests(unittest.TestCase):
    def test_relative_profile_gain_invariance_and_invalid_signal(self):
        wave = np.random.default_rng(53).normal(0, 0.01, c.p.SAMPLES)
        target = c.relative_power_profile(wave)
        for gain in (0.01, 0.1, 10):
            np.testing.assert_allclose(
                c.relative_power_profile(wave * gain), target, rtol=0, atol=1e-10
            )
        for wave in (np.zeros(c.p.SAMPLES), np.ones(10), np.full(c.p.SAMPLES, np.nan)):
            with self.assertRaises(ValueError):
                c.relative_power_profile(wave)

    def test_audio_identity_and_pcm_bounds(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "clip.wav"
            row = c.b.train.pilot.write_audio(path, np.ones(c.p.SAMPLES) * 0.01)
            self.assertEqual(c.checked_audio(row).shape, (c.p.SAMPLES,))
            with self.assertRaisesRegex(ValueError, "hash"):
                c.checked_audio({**row, "sha256": "wrong"})
            wavfile.write(path, 16000, np.full(c.p.SAMPLES, 32767, dtype=np.int16))
            row["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
            with self.assertRaisesRegex(ValueError, "PCM"):
                c.checked_audio(row)

    def test_source_order_and_roles(self):
        rows = [
            {"item_id": f"{key}-{i}", "container_id": key, "role": "unseen_container"}
            for key in c.p.HELDOUT
            for i in (0, 1)
        ]
        with patch.object(c.c, "phase_crop", return_value=None):
            chosen = c.select_patches(rows)
            self.assertEqual(
                [r[0]["item_id"] for r in chosen],
                [rows[0]["item_id"]] * 2 + [rows[2]["item_id"]] * 2,
            )
            rows[0]["role"] = "train"
            with self.assertRaisesRegex(ValueError, "relabelled"):
                c.select_patches(rows)

    def test_end_to_end_publication_and_failed_generation_status(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.json").write_text("{}")
            sha = hashlib.sha256(b"{}").hexdigest()
            controls = np.array([0.5, 0.3, 0.3, 0.5, 0.0, 1, 0, 0, 0, 1, 0], np.float32)
            real = np.ones(c.p.SAMPLES, np.float32) * 0.01
            control_by_object = {key: controls.copy() for key in c.p.HELDOUT}
            control_by_object[c.p.HELDOUT[1]][0] = 0.8
            rows = [
                {
                    "item_id": key,
                    "container_id": key,
                    "role": "unseen_container",
                    "setting": setting,
                }
                for key, setting in zip(c.p.HELDOUT, ("ws-room", "ws-kitchen"))
            ]
            prior_audio = c.b.train.pilot.write_audio(root / "prior.wav", real)
            previous = {
                "status": "complete",
                "source_sha256": sha,
                "rows": [
                    {
                        **prior_audio,
                        "target_item_id": key,
                        "target_phase": phase,
                        "seed": seed,
                        "kind": kind,
                        "controls": control_by_object[key].tolist(),
                    }
                    for key in c.p.HELDOUT
                    for phase in ("first", "middle")
                    for seed in (314, 2718)
                    for kind in ("base", "matched")
                ],
            }
            (root / "result.json").write_text(json.dumps(previous))
            meta = {
                "source": {"source_sha256": sha, "train_ids": []},
                "bridge_sha256": "bridge",
                "frozen_model_sha256_after": "frozen",
            }

            def generate(model, vae, output, name, *, controls, bridge, seed, **extra):
                return {
                    **c.b.train.pilot.write_audio(output / f"{name}.wav", real),
                    "seed": seed,
                    "reference_audio_input": False,
                    **extra,
                }, real

            with (
                patch.object(
                    c.p, "load_source", return_value=(rows, {"terms": "test"})
                ),
                patch.object(
                    c.c,
                    "phase_crop",
                    side_effect=lambda r, phase: (
                        None,
                        control_by_object[r["container_id"]],
                        real,
                        0,
                    ),
                ),
                patch.object(c.b, "load_bridge", return_value=(None, meta)),
                patch.object(c.b.train.tango, "load_models", return_value=(None, None)),
                patch.object(c.b, "digest", return_value="frozen"),
                patch.object(c.b, "generate", side_effect=generate) as renderer,
            ):
                c.run(root, root, root, root / "out")
                report = json.loads((root / "out/result.json").read_text())
                self.assertEqual(report["status"], "complete")
                self.assertEqual(len(report["rows"]), 32)
                self.assertEqual(renderer.call_count, 8)
                self.assertFalse(report["reference_audio_input_to_generator"])
                rate, pcm = wavfile.read(report["comparison"]["wav"])
                self.assertEqual((rate, len(pcm)), (16000, 8 * (c.p.SAMPLES + 8000)))
                meta["bridge_kind"] = "setting-text"
                renderer.reset_mock()
                c.run(root, root, root, root / "setting")
                setting_report = json.loads((root / "setting/result.json").read_text())
                self.assertEqual(len(setting_report["rows"]), 40)
                self.assertEqual(renderer.call_count, 24)
                calls = renderer.call_args_list
                for j in range(0, len(calls), 3):
                    matched, swapped, style = [call.kwargs for call in calls[j : j + 3]]
                    self.assertEqual(matched["setting"], swapped["setting"])
                    self.assertEqual(matched["setting"], style["setting"])
                    self.assertFalse(
                        np.array_equal(matched["controls"], swapped["controls"])
                    )
                    self.assertTrue(
                        np.array_equal(matched["controls"], style["controls"])
                    )
                    self.assertTrue(matched["physical"])
                    self.assertTrue(swapped["physical"])
                    self.assertFalse(style["physical"])
                renderer.side_effect = RuntimeError("expected failure")
                with self.assertRaisesRegex(RuntimeError, "expected failure"):
                    c.run(root, root, root, root / "failed")
                failed = json.loads((root / "failed/result.json").read_text())
                self.assertEqual(failed["status"], "failed")
                self.assertNotIn("comparison", failed)


if __name__ == "__main__":
    unittest.main()
