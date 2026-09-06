import copy
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
from scipy.io import wavfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_sonicgauss_cohort as cohort


def fixture():
    records = []
    projection = {"role": "generator_train", "rows": []}
    for object_id, material in cohort.OBJECTS.items():
        parent = (
            "realimpact-6-bowl--objectfolder-real-object-6"
            if object_id == 6
            else f"fixture-object-{object_id}"
        )
        projection["rows"].append(
            {
                "physical_parent_id": parent,
                "material_label": material,
                "family_component_id": "source-component.realimpact-objectfolder-av-msf.v1",
            }
        )
        for index in range(cohort.COUNTS[object_id]):
            records.append(
                {
                    "ply_file": f"datas/OF_Real/ObjectFolderResults/{object_id}/model.ply",
                    "location": f"datas/OF_Real/ObjectFolderResults/{object_id}/audio/{object_id}_{index}_3sec_48000hz.wav",
                    "contact": [0.1, 0.2, index / 100],
                }
            )
    return records, projection


class CohortTests(unittest.TestCase):
    def test_fixed_roster_first_six_and_object_disjoint_local_roles(self):
        records, projection = fixture()
        selected = cohort.select_records(records, projection)
        self.assertEqual([x["object_id"] for x in selected], list(cohort.OBJECTS))
        train = {x["object_id"] for x in selected if x["local_fit_role"] == "train"}
        dev = {x["object_id"] for x in selected if x["local_fit_role"] == "development"}
        self.assertEqual(train, cohort.FIT_TRAIN)
        self.assertEqual(dev, {14, 75, 94, 97})
        self.assertFalse(train & dev)
        for row in selected:
            self.assertEqual([x["index"] for x in row["contacts"]], list(range(6)))
            self.assertEqual(
                [x["contact"][2] for x in row["contacts"]], [x / 100 for x in range(6)]
            )
            self.assertNotIn(row["object_id"], [36, 41, 70, 80, 92])

    def test_wrong_role_membership_identity_and_contact_reject(self):
        records, projection = fixture()
        bad = copy.deepcopy(projection)
        bad["role"] = "validator_calibration"
        with self.assertRaisesRegex(ValueError, "TRAIN"):
            cohort.select_records(records, bad)
        bad = copy.deepcopy(projection)
        bad["rows"][0]["material_label"] = "Unknown"
        with self.assertRaisesRegex(ValueError, "identity"):
            cohort.select_records(records, bad)
        with self.assertRaisesRegex(ValueError, "membership"):
            cohort.select_records(records[1:], projection)
        bad = copy.deepcopy(records)
        bad[0]["contact"][0] = float("nan")
        with self.assertRaisesRegex(ValueError, "contact"):
            cohort.select_records(bad, projection)
        bad = copy.deepcopy(records)
        bad[0]["location"] = (
            "datas/OF_Real/ObjectFolderResults/41/audio/41_0_3sec_48000hz.wav"
        )
        with self.assertRaisesRegex(ValueError, "member"):
            cohort.select_records(bad, projection)

    def test_audio_hash_channels_and_full_resample(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "audio.wav"
            wavfile.write(path, 48000, np.ones(48000, dtype=np.int16) * 1000)
            value = cohort.audio(path, cohort.pilot.sha256(path))
            self.assertEqual(len(value), 44100)
            self.assertTrue(np.isfinite(value).all())
            with self.assertRaisesRegex(ValueError, "hash"):
                cohort.audio(path, "wrong")
            wavfile.write(path, 48000, np.ones((20, 3), dtype=np.float32))
            with self.assertRaisesRegex(ValueError, "stereo"):
                cohort.audio(path, cohort.pilot.sha256(path))

    def test_descriptive_stft_identity_gain_and_full_tail(self):
        time = np.arange(44100) / 44100
        wave = (np.sin(2 * np.pi * 700 * time) * np.exp(-8 * time)).astype(np.float32)
        self.assertEqual(cohort.magnitude_distance(wave, wave.copy()), 0)
        self.assertAlmostEqual(
            cohort.magnitude_distance(wave, wave * 0.5), 0.5, places=5
        )
        altered = wave.copy()
        altered[30000:] += 0.05 * np.sin(2 * np.pi * 3000 * time[30000:])
        self.assertGreater(cohort.magnitude_distance(wave, altered), 0)
        with self.assertRaisesRegex(ValueError, "silent"):
            cohort.magnitude_distance(np.zeros(44100), wave)


if __name__ == "__main__":
    unittest.main()
