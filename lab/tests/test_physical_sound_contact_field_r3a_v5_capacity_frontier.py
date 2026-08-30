from __future__ import annotations

import sys
import unittest
from pathlib import Path

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v5_capacity_frontier as frontier


class PhysicalSoundContactFieldR3AV5CapacityFrontierTests(unittest.TestCase):
    def test_parse_requires_each_frozen_capacity_once(self) -> None:
        values = [
            "rvq-6kbps=/tmp/a",
            "rvq-12kbps=/tmp/b",
            "rvq-24kbps=/tmp/c",
        ]
        self.assertEqual(sorted(frontier.parse_runs(values)), sorted(frontier.common.CAPACITIES[index]["id"] for index in range(3)))
        with self.assertRaises(frontier.common.V5Error):
            frontier.parse_runs(values[:2])

    def test_ranking_uses_validation_then_rate_then_id(self) -> None:
        records = [
            {"capacity_id": "rvq-24kbps", "validation_total": 9.0, "nominal_bits_per_second": 24_000},
            {"capacity_id": "rvq-6kbps", "validation_total": 10.0, "nominal_bits_per_second": 6_000},
            {"capacity_id": "rvq-12kbps", "validation_total": 9.0, "nominal_bits_per_second": 12_000},
        ]
        self.assertEqual(
            [item["capacity_id"] for item in frontier.rank_capacities(records)],
            ["rvq-12kbps", "rvq-24kbps", "rvq-6kbps"],
        )


if __name__ == "__main__":
    unittest.main()
