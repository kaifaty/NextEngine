import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_water_sources as sources

META = "filename,fold,target,category,src_file,take\n1-101-A-10.wav,1,10,rain,101,A\n5-202-A-10.wav,5,10,rain,202,A\n"
NOTICES = "- [1-101-A.ogg]: https://freesound.org/people/a/sounds/101/ by a [CC0]\n- [5-202-A.ogg]: https://freesound.org/people/b/sounds/202/ by b [CC-BY]\n"


class SourcesTest(unittest.TestCase):
    def test_roles_are_source_disjoint_and_no_physical_attributes_invented(self):
        rows = sources.select(META, NOTICES)
        self.assertEqual([row["role"] for row in rows], ["train", "development"])
        self.assertTrue(all(row["physical_attributes"] is None for row in rows))
        self.assertTrue(all(row["attribution"] for row in rows))

    def test_same_source_across_folds_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "source-disjoint"):
            sources.select(META.replace("202", "101"), NOTICES.replace("202", "101"))

    def test_missing_and_unknown_notices_rejected(self):
        for notice in ("", NOTICES.replace("[CC0]", "[UNKNOWN]")):
            with self.assertRaises(ValueError):
                sources.select(META, notice)

    def test_only_named_unreviewed_sampling_sources_are_deferred(self):
        meta = META + "1-67152-A-17.wav,1,17,pouring_water,67152,A\n"
        notices = (
            NOTICES
            + "- [1-67152-A.ogg]: https://freesound.org/people/c/sounds/67152/ by c [CC-Sampling+]\n"
        )
        self.assertEqual(len(sources.select(meta, notices)), 2)
        with self.assertRaisesRegex(ValueError, "unknown source terms"):
            sources.select(
                meta.replace("67152", "99999"), notices.replace("67152", "99999")
            )

    def test_metadata_identity_and_unsafe_name_rejected(self):
        for meta in (
            META.replace("rain,101", "rain,999"),
            META.replace("1-101-A-10.wav", "../1-101-A-10.wav"),
        ):
            with self.assertRaises(ValueError):
                sources.select(meta, NOTICES)

    def test_existing_source_requires_role_review(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "legacy.json").write_text(
                json.dumps({"url": "https://freesound.org/people/a/sounds/101/"})
            )
            with self.assertRaisesRegex(ValueError, "role review"):
                sources.prior_source_check(root, [{"src_file": "101"}])


if __name__ == "__main__":
    unittest.main()
