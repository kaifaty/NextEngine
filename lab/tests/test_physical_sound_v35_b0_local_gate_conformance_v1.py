from __future__ import annotations

import math
import struct
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v35-f0-geometry-conditioned-hybrid.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v35_b0_local_gate_conformance_v1 as b0


class LocalGateConformanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.overlay, cls.effective = b0.validate_context(PROFILE)
        cls.rows = b0.build_discarded_official_shape()
        cls.interpolator = b0.LocalInterpolator(cls.rows)

    def test_dependencies_imports_and_exact_contact_boundary_are_closed(self) -> None:
        boundary = b0.validate_owner_import_boundary()
        self.assertEqual(boundary["forbidden_imports"], [])
        self.assertIn(
            "physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1",
            boundary["imports"],
        )
        equality = b0.validate_no_exact_contact_equality_branch()
        self.assertEqual(equality["exact_equality_comparisons"], 0)
        self.assertEqual(b0.ZERO_FORBIDDEN_ACCESS["oracle_values_evaluated"], 0)
        self.assertEqual(b0.ZERO_FORBIDDEN_ACCESS["model_parameters_initialized"], 0)

    def test_discarded_corpus_matches_official_shape_without_official_roles(
        self,
    ) -> None:
        self.assertEqual(len(self.rows), 6480)
        self.assertEqual(len(self.interpolator.partition_counts), 30)
        self.assertEqual(set(self.interpolator.partition_counts.values()), {216})
        self.assertTrue(
            all(row.trace_id.startswith("discarded/b0/") for row in self.rows)
        )
        self.assertTrue(all(len(row.causal_key) == 15 for row in self.rows))
        self.assertTrue(all(abs(row.target) < 0.22 for row in self.rows))

    def test_prediction_is_permutation_exact_and_excludes_complete_group(self) -> None:
        query = self.rows[107]
        forward = self.interpolator.predict(query, exclude_query_group=True)
        reverse = b0.LocalInterpolator(list(reversed(self.rows)))
        backward = reverse.predict(query, exclude_query_group=True)
        self.assertEqual(
            self.interpolator.canonical_train_sha256, reverse.canonical_train_sha256
        )
        self.assertEqual(
            struct.pack("<d", forward.value), struct.pack("<d", backward.value)
        )
        self.assertEqual(forward.contributor_count, 215)
        self.assertNotIn(query.group_key, forward.contributor_group_keys)

        source = b0.discarded_row(
            family=0,
            ordinal=0,
            material=b0.DISCARDED_MATERIALS[0],
            geometry=b0.DISCARDED_GEOMETRY[0],
            contact=b0.DISCARDED_CONTACTS[0],
            trace_id="discarded/remesh-a",
            group_suffix=("shared-remesh-group",),
        )
        remesh = b0.LocalRow(
            trace_id="discarded/remesh-b",
            partition=source.partition,
            causal_key=source.causal_key,
            group_key=source.group_key,
            target=source.target,
        )
        other_a = b0.discarded_row(
            family=0,
            ordinal=0,
            material=b0.DISCARDED_MATERIALS[1],
            geometry=b0.DISCARDED_GEOMETRY[1],
            contact=b0.DISCARDED_CONTACTS[1],
            trace_id="discarded/other-a",
        )
        other_b = b0.discarded_row(
            family=0,
            ordinal=0,
            material=b0.DISCARDED_MATERIALS[2],
            geometry=b0.DISCARDED_GEOMETRY[2],
            contact=b0.DISCARDED_CONTACTS[2],
            trace_id="discarded/other-b",
        )
        remesh_prediction = b0.LocalInterpolator(
            [source, remesh, other_a, other_b]
        ).predict(source, exclude_query_group=True)
        self.assertEqual(remesh_prediction.contributor_count, 2)
        self.assertNotIn(source.group_key, remesh_prediction.contributor_group_keys)

    def test_zero_distance_uses_same_kernel_and_ood_boundary_is_strict(self) -> None:
        result = b0.ood_and_zero_distance_conformance(self.interpolator, self.rows)
        self.assertEqual(
            result["exact_local"]["nearest_squared_distance_hex"], "0x0.0p+0"
        )
        self.assertEqual(result["exact_local"]["contributor_count"], 216)
        self.assertNotEqual(
            result["exact_local"]["prediction_hex"], result["exact_local"]["target_hex"]
        )
        self.assertEqual(result["exact_gate"]["local_weight_hex"], (0.8).hex())
        self.assertEqual(result["ood"]["inside"]["decision"], "InDomain")
        self.assertEqual(result["ood"]["threshold"]["decision"], "FallbackOutOfDomain")
        self.assertEqual(result["ood"]["outside"]["decision"], "FallbackOutOfDomain")

    def test_continuity_refines_and_blend_stays_bounded(self) -> None:
        result = b0.continuity_conformance(self.overlay, self.interpolator)
        self.assertEqual(
            set(result["probe_axes"]),
            {
                "geometry_0",
                "geometry_1",
                "geometry_2",
                "contact_u",
                "contact_v",
            },
        )
        for scales in result["probe_axes"].values():
            self.assertEqual(len(scales), 3)
            local_deltas = [float.fromhex(row["local_delta_hex"]) for row in scales]
            self.assertLessEqual(local_deltas[1], local_deltas[0] + 2.0**-48)
            self.assertLessEqual(local_deltas[2], local_deltas[1] + 2.0**-48)
        for local, neural, weight in (
            (0.22, 0.25, 0.8),
            (-0.22, -0.25, 0.8),
            (0.0, 0.25, 0.0),
        ):
            self.assertLessEqual(abs(b0.blend_contact(local, neural, weight)), 0.25)

    def test_invalid_rows_bandwidth_blend_and_output_fail_closed(self) -> None:
        source = self.rows[0]
        conflicting = b0.LocalRow(
            trace_id="discarded/conflicting",
            partition=source.partition,
            causal_key=source.causal_key,
            group_key=source.group_key,
            target=source.target + 0.01,
        )
        with self.assertRaisesRegex(b0.B0ConformanceError, "conflicting targets"):
            b0.LocalInterpolator([source, conflicting])
        with self.assertRaisesRegex(b0.B0ConformanceError, "blend input"):
            b0.blend_contact(0.23, 0.0, 0.5)
        with self.assertRaisesRegex(b0.B0ConformanceError, "finite-positive"):
            b0.coverage_gate((0, 0, 0), (0, 0), [(0, 0, 0)], [(0, 0)], 0.0, 1.0)
        invalid = b0.LocalRow(
            trace_id="discarded/nonfinite",
            partition=source.partition,
            causal_key=source.causal_key,
            group_key=source.group_key,
            target=math.nan,
        )
        with self.assertRaisesRegex(b0.B0ConformanceError, "local target"):
            invalid.validated()
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v35-b0-output-"
        ) as temporary:
            occupied = Path(temporary) / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(b0.B0ConformanceError, "fresh external path"):
                b0.external_output(occupied)


if __name__ == "__main__":
    unittest.main()
