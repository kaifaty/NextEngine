from __future__ import annotations

import copy
import hashlib
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "lab" / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v44_g0b1b_target_audit_v1 as g0b1b  # noqa: E402

PROFILE = ROOT / g0b1b.PROFILE_PATH
PROTOCOL = ROOT / g0b1b.PROTOCOL_PATH
SCRIPT = ROOT / g0b1b.OWNER_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def passing_metrics() -> dict[str, int]:
    return {
        "modal_histogram_cosine_similarity_ppm": 1_000_000,
        "modal_histogram_normalized_l1_ppm": 0,
        "retained_linear_modal_amplitude_ppm": 1_000_000,
    }


def failing_metrics() -> dict[str, int]:
    return {
        "modal_histogram_cosine_similarity_ppm": 800_000,
        "modal_histogram_normalized_l1_ppm": 300_000,
        "retained_linear_modal_amplitude_ppm": 700_000,
    }


def decision_record(
    passing_caps: set[int], *, normalized: bool = True, saturated: bool = False
) -> dict[str, object]:
    mutations = [
        {
            "canonical_pcm_identical": normalized,
            "scale_ppm": scale,
            "target_identical": normalized,
        }
        for scale in (500_000, 2_000_000)
    ]
    return {
        "candidate_caps": [
            {
                "cap": cap,
                "metrics": passing_metrics()
                if cap in passing_caps
                else failing_metrics(),
                "mode_count": cap,
                "passes": cap in passing_caps,
                "target_sha256": "0" * 64,
            }
            for cap in (12, 24, 48, 96)
        ],
        "lineage": {"physical_parent_id": "fixture-parent"},
        "normalization": {
            "post_normalization_clipped_samples": 0 if normalized else 1,
            "scale_mutations": mutations,
        },
        "record_id": "fixture-record",
        "reference": {
            "cap": 512,
            "mode_count": 512 if saturated else 180,
            "saturated": saturated,
            "target_sha256": "1" * 64,
        },
        "source_payload_sha256": "2" * 64,
    }


class TargetAuditTests(unittest.TestCase):
    def test_tracked_protocol_and_profile_are_canonical_and_bind_owner(self) -> None:
        protocol_bytes, protocol = g0b1b.read_canonical_json(PROTOCOL, "protocol")
        profile_bytes, profile = g0b1b.read_canonical_json(PROFILE, "profile")
        g0b1b.validate_protocol(protocol)
        g0b1b.validate_profile(profile)
        self.assertEqual(protocol_bytes, g0b1b.canonical_json(protocol))
        self.assertEqual(profile_bytes, g0b1b.canonical_json(profile))
        owner = next(
            item
            for item in profile["dependency_bindings"]
            if item["path"] == g0b1b.OWNER_PATH
        )
        self.assertEqual(SCRIPT.stat().st_size, owner["bytes"])
        self.assertEqual(sha256(SCRIPT.read_bytes()), owner["sha256"])

    def test_identical_target_modal_histograms_are_exact(self) -> None:
        target = {
            "modes": [
                {
                    "frequency_hz": 1000.0,
                    "relative_level_db": -3.0,
                }
            ],
            "spectral": {
                "band_energy_db": [-20.0] * 24,
                "bandwidth_hz": 1000.0,
                "centroid_hz": 1200.0,
                "flatness": 0.1,
                "rolloff_95_hz": 3000.0,
            },
            "time": {
                "crest_factor": 2.0,
                "decay_t20_extrapolated_t60_seconds": 0.5,
                "peak_offset_ms": 1.0,
                "temporal_centroid_seconds": 0.1,
                "zero_crossing_rate": 0.2,
            },
            "transient_envelope_db": [-30.0] * 48,
        }
        metrics = g0b1b.compare_targets(target, copy.deepcopy(target))
        self.assertEqual(passing_metrics(), metrics)

    def test_smallest_cap_passing_every_record_is_selected(self) -> None:
        _, protocol = g0b1b.read_canonical_json(PROTOCOL, "protocol")
        records = [
            decision_record({24, 48, 96}),
            decision_record({24, 48, 96}),
        ]
        decision, cap, gates = g0b1b.terminal_decision(records, protocol)
        self.assertEqual("VersionedModeCapTargetRequired", decision)
        self.assertEqual(24, cap)
        self.assertEqual([24, 48, 96], gates["candidate_caps_passing_all_records"])

    def test_normalization_failure_has_priority(self) -> None:
        _, protocol = g0b1b.read_canonical_json(PROTOCOL, "protocol")
        decision, cap, gates = g0b1b.terminal_decision(
            [decision_record({12, 24, 48, 96}, normalized=False)], protocol
        )
        self.assertEqual("NormalizationPolicyReject", decision)
        self.assertIsNone(cap)
        self.assertFalse(gates["normalization_passed"])

    def test_reference_saturation_has_priority_over_candidate_pass(self) -> None:
        _, protocol = g0b1b.read_canonical_json(PROTOCOL, "protocol")
        decision, cap, gates = g0b1b.terminal_decision(
            [decision_record({12, 24, 48, 96}, saturated=True)], protocol
        )
        self.assertEqual("TargetReferenceInconclusive", decision)
        self.assertIsNone(cap)
        self.assertFalse(gates["reference_non_saturating"])

    def test_target_cap_global_is_restored_on_failure(self) -> None:
        original = g0b1b.c0.FEATURE_POLICY["maximum_modes"]
        invalid = np.zeros(32, dtype=np.float64)
        with self.assertRaises(g0b1b.TargetAuditError):
            g0b1b.extract_with_cap(invalid, 96)
        self.assertEqual(original, g0b1b.c0.FEATURE_POLICY["maximum_modes"])

    def test_input_mutation_output_guard_and_network_imports(self) -> None:
        with tempfile.TemporaryDirectory(dir="/tmp") as temporary:
            root = Path(temporary)
            document = {"schema": "fixture.v1", "value": 1}
            data = g0b1b.canonical_json(document)
            path = root / "input.json"
            path.write_bytes(data)
            binding = {
                "bytes": len(data),
                "external_relative_path": "input.json",
                "schema": "fixture.v1",
                "sha256": sha256(data),
            }
            self.assertEqual(
                document, g0b1b.load_external_input(root, binding, "fixture")
            )
            path.write_bytes(data + b"mutation")
            with self.assertRaises(g0b1b.TargetAuditError):
                g0b1b.load_external_input(root, binding, "fixture")
        with self.assertRaisesRegex(g0b1b.TargetAuditError, "outside"):
            g0b1b.prepare_output(ROOT / "forbidden-g0b1b-output")
        source = SCRIPT.read_text()
        for forbidden in (
            "import requests",
            "import torch",
            "import urllib",
            "from urllib",
        ):
            self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
