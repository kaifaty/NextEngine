from __future__ import annotations

import json
import unittest
from pathlib import Path

from next_lab.fixed_pd_redundant_contact_research import (
    _validate_profile,
    analyze_redundant_pair,
    canonical_json,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-post-r123-redundant-contact-research.v1.json"


class FixedPdRedundantContactResearchTests(unittest.TestCase):
    def test_profile_is_report_only_and_blocks_every_execution(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        bounded = profile["bounded_acceptance"]
        self.assertTrue(all(value == "NOT_AUTHORIZED" for value in bounded.values()))
        self.assertEqual(profile["scope"]["local_system_reconstructions"], 0)
        self.assertEqual(profile["scope"]["local_system_solves"], 0)

    def test_same_body_heel_forefoot_pair_has_exact_force_gauge(self) -> None:
        result = analyze_redundant_pair(points=_points(), active_point_ordinals=(2, 3))
        self.assertTrue(result["exact_internal_force_gauge_confirmed"])
        self.assertEqual(result["separation_micrometres"], [0, 0, 215_000])
        self.assertEqual(
            result["separation_squared_micrometres_squared"], 46_225_000_000
        )
        self.assertEqual(result["constraint_rank_upper_bound"], 5)
        self.assertEqual(result["minimum_force_nullity"], 1)
        force = result["internal_force_pair_in_common_body_frame"]
        self.assertEqual(force["resultant_force"], [0, 0, 0])
        self.assertEqual(force["resultant_moment"], [0, 0, 0])
        self.assertEqual(force["first_moment"], [-4_006_525_000, 0, 0])
        self.assertEqual(force["second_moment"], [4_006_525_000, 0, 0])

    def test_different_bodies_do_not_confirm_same_body_gauge(self) -> None:
        points = _points()
        points[3] = {**points[3], "body_id": "body.other"}
        result = analyze_redundant_pair(points=points, active_point_ordinals=(2, 3))
        self.assertFalse(result["exact_internal_force_gauge_confirmed"])
        self.assertIsNone(result["constraint_rank_upper_bound"])

    def test_coincident_points_do_not_confirm_line_force_gauge(self) -> None:
        points = _points()
        points[3] = {
            **points[3],
            "local_translation_micrometres": [0, -18_635, -35_000],
        }
        result = analyze_redundant_pair(points=points, active_point_ordinals=(2, 3))
        self.assertFalse(result["separation_is_nonzero"])
        self.assertFalse(result["exact_internal_force_gauge_confirmed"])

    def test_canonical_json_is_order_independent(self) -> None:
        self.assertEqual(canonical_json({"b": 2, "a": 1}), b'{"a":1,"b":2}')


def _points() -> list[dict[str, object]]:
    return [
        {
            "point_ordinal": 0,
            "body_id": "body.left-ankle-roll",
            "effector_id": "effector.left-heel",
            "local_translation_micrometres": [0, -18_635, -35_000],
        },
        {
            "point_ordinal": 1,
            "body_id": "body.left-ankle-roll",
            "effector_id": "effector.left-forefoot",
            "local_translation_micrometres": [0, -18_635, 180_000],
        },
        {
            "point_ordinal": 2,
            "body_id": "body.right-ankle-roll",
            "effector_id": "effector.right-heel",
            "local_translation_micrometres": [0, -18_635, -35_000],
        },
        {
            "point_ordinal": 3,
            "body_id": "body.right-ankle-roll",
            "effector_id": "effector.right-forefoot",
            "local_translation_micrometres": [0, -18_635, 180_000],
        },
    ]


if __name__ == "__main__":
    unittest.main()
