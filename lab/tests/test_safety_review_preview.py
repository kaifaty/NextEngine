from pathlib import Path
import unittest

from next_lab.motor_mirror import load_json
from next_lab.safety_review_preview import (
    render_contact_semantics_review,
    render_native_standing_review,
)


FIXTURES = Path(__file__).parent / "fixtures"


class SafetyReviewPreviewTests(unittest.TestCase):
    def test_contact_semantics_preview_is_deterministic_and_complete(self) -> None:
        descriptor = load_json(FIXTURES / "biomechanics_motor_mirror_v1.json")
        golden = load_json(FIXTURES / "biomechanics_safety_contact_mirror_v1.json")
        first = render_contact_semantics_review(descriptor, golden)
        self.assertEqual(first, render_contact_semantics_review(descriptor, golden))
        for name in (
            "locomotion-hand-grace",
            "locomotion-knee-material",
            "locomotion-torso-material",
            "getup-knee-support",
            "locomotion-head-impact",
            "locomotion-self-collision",
            "locomotion-timeout",
        ):
            self.assertIn(name, first)

    def test_native_preview_rejects_nonsole_review_contact(self) -> None:
        descriptor = load_json(FIXTURES / "biomechanics_motor_mirror_v1.json")
        review = {
            "schema_version": 1,
            "status": "PASS",
            "body_schema_hash": descriptor["body_schema_hash"],
            "compiled_descriptor_hash": descriptor["compiled_descriptor_hash"],
            "samples": [],
        }
        with self.assertRaisesRegex(ValueError, "sample closure"):
            render_native_standing_review(descriptor, review)


if __name__ == "__main__":
    unittest.main()
