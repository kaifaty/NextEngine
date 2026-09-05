import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
import soundfile as sf
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_syncfusion_condition as condition


class ConditionTests(unittest.TestCase):
    def test_selection_uses_first_annotated_hit_not_waveform(self):
        annotation = "0,wood scratch static\n0.5,glass hit static\n1.2,wood hit deform\n2.0,None None None\n"
        self.assertEqual(
            condition.first_hit(annotation, "glass"), (0.5, 1.2, "glass hit static")
        )
        self.assertEqual(
            condition.first_hit(annotation, "wood"), (1.2, 2.0, "wood hit deform")
        )
        self.assertIsNone(condition.first_hit(annotation, "metal"))

    def test_selection_does_not_cross_long_gaps_or_use_final_unbounded_hit(self):
        annotation = "0,glass hit static\n4,wood hit static\n"
        self.assertIsNone(condition.first_hit(annotation, "glass"))
        self.assertIsNone(condition.first_hit(annotation, "wood"))

    def test_repeatpad_matches_author_quantization_and_filling(self):
        wave = np.linspace(-1.1, 1.1, 70001, dtype=np.float32)
        x = torch.from_numpy(wave)
        quantized = (torch.clip(x, -1, 1) * 32767).to(torch.int16).to(
            torch.float32
        ) / 32767
        reference = torch.nn.functional.pad(
            quantized.repeat(6), (0, 480000 - 6 * len(x))
        )
        self.assertTrue(torch.equal(condition.repeatpad(wave), reference))

    def test_repeatpad_rejects_invalid_conditioning(self):
        for wave in ([], [np.nan], [[1]], np.zeros(480001)):
            with self.assertRaises(ValueError):
                condition.repeatpad(wave)

    def test_spectral_shape_is_gain_invariant_and_separates_tones(self):
        time = np.arange(9600) / 48000
        first = np.sin(2 * np.pi * 1000 * time)
        second = np.sin(2 * np.pi * 4000 * time)
        np.testing.assert_allclose(
            condition.spectral_shape(first),
            condition.spectral_shape(first * 0.2),
            atol=1e-8,
        )
        self.assertGreater(
            np.abs(
                condition.spectral_shape(first) - condition.spectral_shape(second)
            ).mean(),
            10,
        )
        with self.assertRaises(ValueError):
            condition.spectral_shape(np.zeros(9600))

    def test_assessment_keeps_reference_and_generator_roles_separate(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source, baseline, output = [
                root / name for name in ("source", "baseline", "output")
            ]
            for path in (source, baseline, output):
                path.mkdir()
            generator = np.random.default_rng(42)
            selected, old_rows, new_rows = [], [], []
            for material in condition.MATERIALS:
                for index in range(2):
                    name = f"{material}-{index}"
                    path = source / f"{name}-event.wav"
                    sf.write(
                        path,
                        generator.normal(0, 0.01, 12000),
                        condition.pilot.RATE,
                        subtype="FLOAT",
                    )
                    selected.append(
                        {
                            "id": name,
                            "material": material,
                            "event_sha256": condition.pilot.sha(path),
                        }
                    )
                for kind, directory, rows in (
                    (material, baseline, old_rows),
                    (material, output, new_rows),
                    (material + "-individual", output, new_rows),
                ):
                    path = directory / f"{kind}.wav"
                    sf.write(
                        path,
                        generator.normal(0, 0.01, condition.pilot.LENGTH),
                        condition.pilot.RATE,
                        subtype="PCM_16",
                    )
                    rows.append(
                        {
                            "id": kind,
                            "wav": str(path),
                            "sha256": condition.pilot.sha(path),
                            "status": "published",
                        }
                    )
            condition.pilot.save(source / "source.json", {"selected": selected})
            condition.pilot.save(
                baseline / "result.json", {"status": "complete", "rows": old_rows}
            )
            condition.pilot.save(
                output / "result.json",
                {
                    "status": "complete",
                    "rows": new_rows,
                    "source_sha256": condition.pilot.sha(source / "source.json"),
                },
            )
            with patch("physical_sound_text_tags.run") as classifier:
                condition.assess(source, baseline, output)
                self.assertEqual(classifier.call_count, 2)
            report = condition.json.loads((output / "assessment.json").read_text())
            self.assertEqual(len(report["rows"]), 6)
            self.assertEqual(
                report["comparisons"]["glass"]["order"],
                [
                    "reference-glass-0",
                    "reference-glass-1",
                    "text",
                    "prototype",
                    "individual",
                ],
            )
            with self.assertRaises(ValueError):
                condition.assess(source, baseline, output)


if __name__ == "__main__":
    unittest.main()
