from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
import soundfile as sf

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_pouring_compare as compare
import physical_sound_pouring_pilot as p


def row(container="container_18"):
    wave = np.linspace(-0.01, 0.01, p.SAMPLES * 2, dtype=np.float32)
    return {
        "item_id": container + "-example",
        "container_id": container,
        "wave": wave,
        "spectrogram": p.encode(wave),
        "dimensions": {"net_height": 10, "diameter_top": 7, "diameter_bottom": 7},
        "material": "glass",
        "shape": "cylindrical",
        "duration": len(wave) / p.RATE,
    }


class CompareTests(unittest.TestCase):
    def test_training_controls_select_first_per_object_and_exclude_development(self):
        rows = [
            row(x)
            for x in ("train_a", "container_18", "train_a", "train_b", "container_30")
        ]
        self.assertEqual(
            [x["container_id"] for x in compare.select_rows(rows, False)],
            list(p.HELDOUT),
        )
        selected = compare.select_rows(rows, True)
        self.assertEqual([x["container_id"] for x in selected], ["train_a", "train_b"])
        self.assertIs(selected[0], rows[0])

    def test_phase_alignment_and_short_padding(self):
        source = row()
        for phase in ("first", "middle"):
            spec, controls, real, offset = compare.phase_crop(source, phase)
            self.assertEqual(spec.shape, (p.FRAMES, p.FRAMES))
            self.assertEqual(offset % p.HOP, 0)
            self.assertAlmostEqual(controls[4], offset / len(source["wave"]), places=6)
            np.testing.assert_array_equal(
                real, source["wave"][offset : offset + p.SAMPLES]
            )
        self.assertGreater(compare.phase_crop(source, "middle")[3], 0)
        source["wave"] = source["wave"][:100]
        source["spectrogram"] = np.full((256, 2), -2, dtype=np.float32)
        self.assertEqual(len(compare.phase_crop(source, "middle")[2]), p.SAMPLES)
        with self.assertRaises(ValueError):
            compare.phase_crop(source, "end")

    def test_seed_validation_precedes_io(self):
        for seeds in ([], [1, 1], [-1], [2**32]):
            with self.assertRaises(ValueError):
                compare.run(
                    Path("missing"),
                    Path("missing"),
                    Path("missing"),
                    Path("missing"),
                    seeds,
                )

    def test_complete_matrix_and_reference_free_sampling(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.json").write_text("{}")
            sources = [row(x) for x in ("training", *p.HELDOUT)]
            meta = {
                "train_ids": [sources[0]["item_id"]],
                "heldout_containers": list(p.HELDOUT),
                "source_sha256": hashlib.sha256(b"{}").hexdigest(),
            }
            sampled = []

            def sample(model, controls, seed):
                self.assertEqual(controls.shape, (11,))
                sampled.append(seed)
                return np.full((256, 256), -2, dtype=np.float32)

            with (
                patch.object(
                    p, "load_source", return_value=(sources, {"terms": "test only"})
                ),
                patch.object(compare, "load_model", return_value=(object(), meta)),
                patch.object(p, "sample", side_effect=sample),
            ):
                compare.run(root, root, root, root / "out", [1, 2], "cpu")
            result = json.loads((root / "out/result.json").read_text())
            self.assertEqual(result["status"], "complete")
            self.assertEqual(len(result["rows"]), 24)
            self.assertEqual(len(sampled), 16)
            self.assertFalse(result["reference_audio_input_to_generator"])
            self.assertEqual(len(result["previews"]), 2)
            for record in result["rows"] + result["previews"]:
                path = Path(record["wav"])
                self.assertEqual(
                    hashlib.sha256(path.read_bytes()).hexdigest(), record["sha256"]
                )
                self.assertEqual(sf.info(path).samplerate, p.RATE)
            with self.assertRaises(ValueError):
                compare.run(root, root, root, root / "out", [1, 2], "cpu")


if __name__ == "__main__":
    unittest.main()
