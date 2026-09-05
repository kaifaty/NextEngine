import copy
import tempfile
import unittest
from pathlib import Path

from next_lab.isaac_training import atomic_write_json, sha256_file

from lab.scripts.evaluate_known_walking_candidate import candidate_checkpoint


class KnownCandidateTests(unittest.TestCase):
    def test_exact_closed_candidate_only_and_corrupt_source_rejection(self):
        with tempfile.TemporaryDirectory() as directory:
            run = Path(directory)
            checkpoint = run / "model_3999.pt"
            atomic_write_json(checkpoint, {"fixture": "not loaded as torch"})
            cfg = {
                "source_checkpoint_iteration": 3999,
                "source_checkpoint_name": checkpoint.name,
                "source_checkpoint_sha256": sha256_file(checkpoint),
            }
            manifest = {"artifacts": {checkpoint.name: sha256_file(checkpoint)}}
            self.assertEqual(candidate_checkpoint(run, manifest, cfg), checkpoint)
            for name, iteration in (
                ("model_9999.pt", 9999),
                ("../model_3999.pt", 3999),
                ("model_3999.pt", 9999),
            ):
                bad = {
                    **cfg,
                    "source_checkpoint_name": name,
                    "source_checkpoint_iteration": iteration,
                }
                with (
                    self.subTest(name=name, iteration=iteration),
                    self.assertRaises(ValueError),
                ):
                    candidate_checkpoint(run, manifest, bad)
            bad = copy.deepcopy(manifest)
            bad["artifacts"].clear()
            with self.assertRaisesRegex(ValueError, "not closed"):
                candidate_checkpoint(run, bad, cfg)
            atomic_write_json(checkpoint, {"corrupted": True})
            with self.assertRaisesRegex(ValueError, "hash mismatch"):
                candidate_checkpoint(run, manifest, cfg)


if __name__ == "__main__":
    unittest.main()
