import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np
import pandas as pd
from scipy.io import wavfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_epic_slice as s


class EpicTests(unittest.TestCase):
    def frame(self):
        return pd.DataFrame(
            [
                {
                    "annotation_id": f"P{i:02d}_01_{j}",
                    "participant_id": f"P{i:02d}",
                    "video_id": f"P{i:02d}_01",
                    "class": label,
                    "start_sample": j * s.RATE * 5,
                    "stop_sample": (j * 5 + 1) * s.RATE,
                }
                for j, label in enumerate(s.CLASSES)
                for i in range(1, 6)
            ]
        )

    def test_selection_coverage_overlap_and_failure(self):
        frame = self.frame()
        rows = s.select(frame)
        self.assertEqual(len(rows), 24)
        self.assertEqual(
            [r["participant_id"] for r in rows[:4]], ["P01", "P02", "P03", "P04"]
        )
        overlap = frame.iloc[0].copy()
        overlap["annotation_id"] = "P01_01_99"
        overlap["class"] = "background"
        rows = s.select(pd.concat([frame, pd.DataFrame([overlap])]))
        self.assertEqual(
            [r["participant_id"] for r in rows[:4]], ["P02", "P03", "P04", "P05"]
        )
        with self.assertRaisesRegex(ValueError, "duplicate"):
            s.select(pd.concat([frame, frame.iloc[:1]]))
        with self.assertRaisesRegex(ValueError, "coverage"):
            s.select(frame[frame.participant_id == "P01"])
        frame["start_sample"] = frame["start_sample"].astype(float) + 0.5
        with self.assertRaisesRegex(ValueError, "integer"):
            s.select(frame)

    def test_original_and_extension_urls_mixed_string_versions(self):
        splits = pd.DataFrame([{"video_id": "P01_01", "split": "train"}])
        checksums = pd.DataFrame(
            [
                {
                    "file_remote_path": "videos/train/P01/P01_01.MP4",
                    "md5": "a" * 32,
                    "version": "55",
                },
                {
                    "file_remote_path": "P02/videos/P02_101.MP4",
                    "md5": "b" * 32,
                    "version": "100",
                },
                {
                    "file_remote_path": "irrelevant",
                    "md5": "c" * 32,
                    "version": "errata",
                },
            ]
        )
        for video in ("P01_01", "P02_101"):
            row = {"video_id": video, "participant_id": video.split("_")[0]}
            source = s.video_source(row, splits, checksums)
            self.assertTrue(source["url"].endswith(video + ".MP4"))
            self.assertFalse(source["whole_video_md5_verified"])
        with self.assertRaises(ValueError):
            s.video_source(
                {"video_id": "../bad", "participant_id": "P01"}, splits, checksums
            )

    def test_extract_tls_sample_bounds_no_invented_physics(self):
        row = self.frame().iloc[0].to_dict()
        source = {"url": s.BASE55 + "/videos/train/P01/P01_01.MP4"}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def decode(command, **kwargs):
                self.assertIn("-tls_verify", command)
                self.assertEqual(command[command.index("-tls_verify") + 1], "1")
                self.assertEqual(kwargs["timeout"], 60)
                wavfile.write(command[-1], s.RATE, np.full(s.RATE, 32767, np.int16))

            with patch.object(s.subprocess, "run", side_effect=decode):
                result = s.extract(row, source, root)
            self.assertIsNone(result["striker_material"])
            self.assertIsNone(result["physical_object_id"])
            self.assertEqual(result["materials_unordered"], ["metal", "glass"])
            self.assertLess(result["pcm_gain"], 1)
            rate, pcm = wavfile.read(result["wav"])
            self.assertEqual((rate, len(pcm)), (s.RATE, s.RATE))
            self.assertLessEqual(int(pcm.max()), round(0.98 * 32767))
            with patch.object(s.subprocess, "run"):
                row["stop_sample"] += 1
                with self.assertRaisesRegex(ValueError, "sample bounds"):
                    s.extract(row, source, root)


if __name__ == "__main__":
    unittest.main()
