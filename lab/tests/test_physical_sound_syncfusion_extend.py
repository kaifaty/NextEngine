import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import physical_sound_syncfusion_extend as extension


class ExtensionTests(unittest.TestCase):
    def test_fixed_reference_prefix_cannot_be_reassigned(self):
        old = [self.row("train/old", "combination-dev")]
        current = extension.append_rows(old, [self.row("train/new")])
        self.assertEqual(extension.adapter.fixed_reference_rows(current, old), old)
        altered = [self.row("train/old", "train")]
        with self.assertRaisesRegex(ValueError, "reference"):
            extension.adapter.fixed_reference_rows(current, altered)

    def row(self, name, role="train", material="wood", motion="static"):
        return {"recording": name, "role": role, "material": material, "motion": motion}

    def test_preserves_old_rows_and_roles(self):
        old = [self.row("train/old", "combination-dev")]
        new = [self.row("train/new")]
        result = extension.append_rows(old, new)
        self.assertEqual(result[: len(old)], old)
        self.assertEqual(len(old), 1)

    def test_duplicate_reassignment_rejected(self):
        with self.assertRaisesRegex(ValueError, "Duplicate"):
            extension.append_rows(
                [self.row("train/a", "recording-dev")], [self.row("train/a")]
            )

    def test_author_test_combination_and_mixed_roles_rejected(self):
        for rows in (
            [self.row("test/a")],
            [self.row("train/a", material="glass", motion="rigid-motion")],
            [self.row("train/a"), self.row("train/a", "combination-dev")],
        ):
            with self.assertRaises(ValueError):
                extension.append_rows([], rows)


if __name__ == "__main__":
    unittest.main()
