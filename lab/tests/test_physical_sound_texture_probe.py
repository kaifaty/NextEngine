from __future__ import annotations

import io
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

import numpy as np
import soundfile as sf

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_texture_probe as probe


class TextureProbeTests(unittest.TestCase):
    def wav(self, data, rate=44100):
        buffer = io.BytesIO()
        sf.write(buffer, data, rate, format="WAV", subtype="FLOAT")
        return buffer.getvalue()

    def test_fixed_physical_grid(self):
        rows = probe.conditions()
        self.assertEqual(len(rows), 12)
        self.assertEqual(len({r["id"] for r in rows}), 12)
        self.assertEqual({r["texture_id"] for r in rows}, {0, 65, 74})
        self.assertEqual({r["commanded_normal_force_N"] for r in rows}, {0.5, 1})
        self.assertEqual({r["commanded_speed_mm_s"] for r in rows}, {20, 60})
        self.assertTrue(all(r["probe_material"] == "urethane rubber" for r in rows))

    def test_raw_channels_kept_separate(self):
        data = np.tile([0.25, 0.125], (4410, 1))
        result = probe.validate_audio(self.wav(data), 2)
        self.assertEqual(result["channels"], 2)
        self.assertEqual(result["rms"], [0.25, 0.125])
        self.assertEqual(result["seconds"], 0.1)

    def test_training_grid_has_repeats_and_middle_speed(self):
        rows = probe.conditions(True)
        self.assertEqual(len(rows), 60)
        self.assertEqual(len({row["id"] for row in rows}), 60)
        self.assertEqual({row["repeat"] for row in rows}, {0, 1})
        self.assertEqual(
            {row["commanded_speed_mm_s"] for row in rows}, {20, 30, 40, 50, 60}
        )

    def test_invalid_audio_rejected(self):
        for data, rate, channels in [
            (np.zeros(441), 16000, 1),
            (np.zeros((441, 2)), 44100, 1),
            (np.zeros(0), 44100, 1),
            (np.zeros(44100 * 15 + 1), 44100, 1),
            (np.full(441, np.nan), 44100, 1),
        ]:
            with (
                self.subTest(rate=rate, shape=data.shape),
                self.assertRaises(ValueError),
            ):
                probe.validate_audio(self.wav(data, rate), channels)

    def test_no_source_amplitude_invention(self):
        result = probe.validate_audio(self.wav(np.zeros(441)), 1)
        self.assertEqual(result["rms"], [0.0])
        # Silence is evidence, not a reason to amplify or omit a source.

    def test_bounded_selected_surface_grid(self):
        self.assertEqual(len(probe.conditions(True, {2: "Elm", 4: "Oak"})), 40)
        for bad in ({}, {118: "unknown"}, {True: "boolean"}, {2: ""}):
            with self.assertRaises(ValueError):
                probe.conditions(True, bad)

    def test_table_cached_values_and_rejection(self):
        ns = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
        strings = f'<sst xmlns="{ns}"><si><t>Wood</t></si></sst>'
        header = '<row><c r="A1"><v>Texture_id</v></c><c r="F1"><v>Static friction coefficient</v></c><c r="G1"><v>Dynamic friction coefficient</v></c></row>'
        rows = "".join(
            f'<row><c r="A{i + 2}"><v>{i}</v></c><c r="B{i + 2}"><v>surface{i}</v></c><c r="D{i + 2}" t="s"><v>0</v></c><c r="F{i + 2}"><v>0.5</v></c><c r="G{i + 2}"><v>0.4</v></c></row>'
            for i in range(118)
        )
        sheet = (
            f'<worksheet xmlns="{ns}"><sheetData>{header}{rows}</sheetData></worksheet>'
        )
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "metadata.xlsx"

            def write(data):
                with zipfile.ZipFile(path, "w") as archive:
                    archive.writestr("xl/sharedStrings.xml", strings)
                    archive.writestr("xl/worksheets/sheet1.xml", data)
                    archive.writestr(
                        "xl/styles.xml", "deliberately malformed unused styles"
                    )

            write(sheet)
            result = probe.texture_metadata(path)
            self.assertEqual(len(result), 118)
            self.assertEqual(result[76]["dynamic_friction"], 0.4)
            for bad in (
                sheet.replace("<v>0.4</v>", "<v>0.6</v>", 1),
                sheet.replace("<v>0.4</v>", "<f>1+1</f><v>0.4</v>", 1),
                "<!DOCTYPE a>" + sheet,
                f'<worksheet xmlns="{ns}"><sheetData/></worksheet>',
            ):
                write(bad)
                with self.assertRaises(ValueError):
                    probe.texture_metadata(path)


if __name__ == "__main__":
    unittest.main()
