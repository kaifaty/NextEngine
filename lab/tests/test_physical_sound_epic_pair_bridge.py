import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
import pandas as pd
import torch
from scipy.io import wavfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_epic_pair_bridge as e


class PairTests(unittest.TestCase):
    def setUp(self):
        torch.set_num_threads(2)

    def test_factorized_controls_and_centered_gradients(self):
        a = e.controls("metal / glass collision")
        self.assertEqual(a.tolist(), [0.5, 0.5, 0, 0, 0])
        with self.assertRaises(ValueError):
            e.controls("steel / glass collision")
        model = e.PairBridge()
        self.assertEqual(sum(p.numel() for p in model.parameters()), 10240)
        bank = torch.tensor(
            np.stack([e.controls(k) for k in e.source.CLASSES if k != e.HELD_PAIR])
        )
        model.center_controls = bank
        hidden, pooled = torch.zeros(2, 4, 1024), torch.zeros(2, 1024)
        h, p = model.condition(bank[:1], hidden, pooled, cfg=True)
        self.assertTrue(torch.equal(h, hidden))
        self.assertTrue(torch.equal(p, pooled))
        with torch.no_grad():
            model.network.weight.normal_(0, 0.01)
        h, p = model.condition(bank[:1], hidden, pooled, cfg=True)
        p.square().mean().backward()
        self.assertGreater(float(model.network.weight.grad.abs().sum()), 0)
        self.assertTrue(torch.equal(h[0], hidden[0]))
        self.assertTrue(torch.equal(h[:, -1], hidden[:, -1]))
        before = model.delta(bank[:1])
        model.freeze_centering()
        self.assertTrue(torch.equal(before, model.delta(bank[:1])))
        # Linear material factors compose, not one learned lookup per pair.
        x = torch.tensor(e.controls(e.HELD_PAIR))[None]
        self.assertTrue(torch.isfinite(model.delta(x)).all())

        def delta(label):
            return model.delta(torch.tensor(e.controls(label))[None])

        torch.testing.assert_close(
            delta(e.HELD_PAIR),
            delta("metal / glass collision")
            + delta("plastic / wood collision")
            - delta("metal / plastic collision"),
        )
        np.testing.assert_allclose(
            e.profile(np.sin(np.arange(24000) * 0.2), 24000),
            e.profile(0.01 * np.sin(np.arange(24000) * 0.2), 24000),
            atol=1e-9,
        )

    def test_source_roles_coverage_and_overlap(self):
        rows = []
        for j, label in enumerate(e.source.CLASSES):
            for part in ("P01", "P02", "P03", "P04"):
                for i in range(4):
                    start = (j * 10 + i * 2) * 24000
                    rows.append(
                        {
                            "annotation_id": f"{part}_01_{j * 4 + i}",
                            "video_id": part + "_01",
                            "participant_id": part,
                            "class": label,
                            "start_sample": start,
                            "stop_sample": start + 12000,
                        }
                    )
        frame = pd.DataFrame(rows)
        selected = e.select_training(frame)
        self.assertEqual(len(selected), 45)
        self.assertNotIn(e.HELD_PAIR, {r["class"] for r in selected})
        self.assertEqual({r["participant_id"] for r in selected}, {"P01", "P02", "P03"})
        overlap = frame.iloc[0].to_dict()
        overlap["annotation_id"] = "extra"
        overlap["class"] = "background"
        selected = e.select_training(pd.concat([frame, pd.DataFrame([overlap])]))
        self.assertNotEqual(selected[0]["annotation_id"], rows[0]["annotation_id"])
        with self.assertRaisesRegex(ValueError, "three TRAIN"):
            e.select_training(frame[frame.participant_id != "P03"])

    def test_source_pcm_identity_and_bounds(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.wav"
            wavfile.write(path, 24000, np.ones(12000, np.int16))
            row = {
                "wav": str(path),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "start_sample": 0,
                "stop_sample": 12000,
            }
            self.assertEqual(e.checked_wave(row).shape, (12000,))
            with self.assertRaisesRegex(ValueError, "identity"):
                e.checked_wave({**row, "sha256": "bad"})
            with self.assertRaisesRegex(ValueError, "PCM"):
                e.checked_wave({**row, "stop_sample": 12001})

    def test_checkpoint_roundtrip_and_failures(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            model = e.PairBridge()
            e.save_file(
                {**model.state_dict(), "offset": torch.ones(1, 2048)},
                root / "bridge.safetensors",
            )
            meta = {
                "format": e.FORMAT,
                "status": "complete",
                "revision": e.b.train.tango.REVISION,
                "materials": list(e.MATERIALS),
                "bridge_sha256": hashlib.sha256(
                    (root / "bridge.safetensors").read_bytes()
                ).hexdigest(),
            }
            (root / "result.json").write_text(json.dumps(meta))
            with patch.object(e.PairBridge, "to", lambda self, *a, **kw: self):
                restored, _ = e.load_bridge(root)
            self.assertTrue(torch.equal(restored.offset, torch.ones(1, 2048)))
            self.assertTrue(torch.equal(restored.network.weight, model.network.weight))
            meta["materials"].reverse()
            (root / "result.json").write_text(json.dumps(meta))
            with self.assertRaisesRegex(ValueError, "identity"):
                e.load_bridge(root)

    def test_event_publication_preserves_horizon_and_rejects_noise(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rate = e.b.train.tango.RATE
            wave = np.full((2, 6 * rate), 1e-5, np.float32)
            wave[:, 4 * rate : 5 * rate] = 0.2 * np.sin(np.arange(rate) * 0.1)
            row = e.publish_generation(root, "late", wave, event_matched=True)
            self.assertGreater(row["event_window"]["crop_start_seconds"], 3)
            self.assertGreater(row["native_rms"], 0.05)
            self.assertLess(row["raw_prefix"]["native_rms"], 2e-5)
            _, pcm = wavfile.read(row["full_horizon"]["native_wav"])
            self.assertEqual(pcm.shape, (6 * rate, 2))
            expected, _ = e.b.train.tango.event_window(
                pcm.T.astype(np.float32) / 32767, e.SECONDS
            )
            _, actual = wavfile.read(row["native_wav"])
            np.testing.assert_array_equal(
                actual, np.round(expected.T * 32767).astype(np.int16)
            )
            # Identical policy applies to baseline and conditioned candidates.
            again = e.publish_generation(root, "base", wave, event_matched=True)
            self.assertEqual(row["native_sha256"], again["native_sha256"])
            with self.assertRaisesRegex(ValueError, "no detected event"):
                e.publish_generation(
                    root, "noise", np.full_like(wave, 1e-5), event_matched=True
                )
            self.assertTrue((root / "noise-full-stereo.wav").is_file())
            rejection = json.loads((root / "noise-extraction.json").read_text())
            self.assertEqual(rejection["event_window"]["status"], "no_detected_event")

    def test_matrix_uses_same_event_policy_without_training(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            checkpoint = root / "model"
            checkpoint.mkdir()
            (checkpoint / "result.json").write_text("{}")
            meta = {
                "bridge_sha256": "fixed",
                "development_rows": [],
                "frozen_model_after": "same",
            }

            def fake_generate(*args, **kwargs):
                self.assertTrue(kwargs["event_matched"])
                seed = args[6] if len(args) > 6 else kwargs["seed"]
                return {
                    "seed": seed,
                    "material_pair": args[5] if len(args) > 5 else None,
                }

            with (
                patch.object(e, "load_bridge", return_value=(object(), meta)),
                patch.object(
                    e.b.train.tango, "load_models", return_value=(object(), object())
                ),
                patch.object(e.b, "digest", return_value="same"),
                patch.object(e, "generate", side_effect=fake_generate) as generation,
                patch.object(e, "compare_development"),
            ):
                e.event_matrix(checkpoint, root / "out")
                e.event_matrix(checkpoint, root / "text", text_control=True)
            report = json.loads((root / "out" / "result.json").read_text())
            self.assertEqual(generation.call_count, 28)
            self.assertEqual(report["status"], "complete")
            self.assertFalse(report["training_performed"])
            self.assertEqual({r["seed"] for r in report["rows"]}, {314, 2718})
            text_report = json.loads((root / "text" / "result.json").read_text())
            self.assertFalse(text_report["bridge_applied"])
            self.assertTrue(
                all(
                    r["training_pair"] is None
                    for r in text_report["rows"]
                    if r["kind"] == "matched"
                )
            )
            for call in generation.call_args_list[14:]:
                if len(call.args) > 4:
                    self.assertIsNone(call.args[4])
                    self.assertEqual(
                        call.kwargs["prompt"], e.material_prompt(call.args[5])
                    )
            self.assertEqual(
                e.material_prompt(e.HELD_PAIR),
                "The sound of an object made of wood colliding with an object made of glass.",
            )
            with self.assertRaises(ValueError):
                e.material_prompt("unknown")

    def test_expanded_data_keeps_pair_and_participant_exclusions(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expanded = root / "expanded"
            expanded.mkdir()
            records = []

            def record(label, part):
                i = len(records)
                row = {
                    "annotation_id": f"{part}_01_{i}",
                    "participant_id": part,
                    "video_id": part + "_01",
                    "class": label,
                    "start_sample": i * 48000,
                    "stop_sample": i * 48000 + 12000,
                }
                records.append(row)
                path = root / (row["annotation_id"] + ".wav")
                wavfile.write(path, 24000, np.ones(12000, np.int16))
                return {
                    **row,
                    "wav": str(path),
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                }

            train = [
                record(label, "P01")
                for label in e.source.CLASSES
                if label != e.HELD_PAIR
                for _ in range(12)
            ]
            held = record(e.HELD_PAIR, "P02")
            dev = [record(label, "P04") for label in e.source.CLASSES] + [
                record(e.HELD_PAIR, "P07")
            ]
            pd.DataFrame(records).to_csv(root / "train.csv", index=False)
            original = {
                "status": "complete",
                "annotation_revision": e.source.ANNOTATIONS,
                "files": [
                    {
                        "path": "train.csv",
                        "sha256": hashlib.sha256(
                            (root / "train.csv").read_bytes()
                        ).hexdigest(),
                    }
                ],
                "rows": dev,
            }
            e.source.save(root / "result.json", original)
            manifest = {
                "status": "complete",
                "annotation_revision": e.source.ANNOTATIONS,
                "source_result_sha256": hashlib.sha256(
                    (root / "result.json").read_bytes()
                ).hexdigest(),
                "rows": train + [held],
            }
            e.source.save(expanded / "result.json", manifest)
            actual, actual_dev = e.expanded_data(root, expanded)
            self.assertEqual(actual, train)
            self.assertEqual(actual_dev, dev)
            manifest["rows"][0] = dev[0]
            e.source.save(expanded / "result.json", manifest)
            with self.assertRaisesRegex(ValueError, "roles"):
                e.expanded_data(root, expanded)


if __name__ == "__main__":
    unittest.main()
