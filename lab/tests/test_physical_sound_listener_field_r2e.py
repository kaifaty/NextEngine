from __future__ import annotations

import sys
import unittest
from pathlib import Path

import torch

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_listener_field_r2e_common as common  # noqa: E402
import physical_sound_listener_field_r2e_evaluate as evaluate  # noqa: E402
import physical_sound_listener_field_r2e_failure_diagnostic as diagnostic  # noqa: E402
import physical_sound_listener_field_r2e_train as train  # noqa: E402


def row(angle: int, offset: int = 0, microphone: int = 0) -> dict[str, int]:
    return {
        "azimuth_degrees": angle,
        "gantry_distance_offset_millimetres": offset,
        "microphone_id": microphone,
    }


class PhysicalSoundListenerFieldR2ETests(unittest.TestCase):
    def test_profile_freezes_one_query_isolated_candidate(self) -> None:
        profile = common.profile()
        self.assertEqual(profile["candidate_id"], common.CANDIDATE_ID)
        self.assertEqual(profile["representation"]["rank"], 96)
        self.assertFalse(
            profile["representation"]["query_audio_used_for_training_or_selection"]
        )
        self.assertEqual(profile["repetition"]["count"], 2)
        self.assertEqual(profile["query_evaluation"]["runs_authorized"], 1)
        self.assertEqual(
            profile["objective"], common.v1.profile()["objective"]
        )
        self.assertEqual(
            profile["context_gate"], common.v1.profile()["gates"]
        )

    def test_context_cosine_basis_is_well_conditioned_and_complete(self) -> None:
        rows = [row(angle) for angle in common.CONTEXT_ANGLES_DEGREES]
        angles, groups = common.encode_rows(
            rows,
            torch.device("cpu"),
            expected_angles=common.CONTEXT_ANGLES_DEGREES,
        )
        matrix = common.angular_features(angles).to(torch.float64)
        self.assertEqual(tuple(matrix.shape), (7, 7))
        self.assertEqual(groups.tolist(), [0] * 7)
        self.assertEqual(int(torch.linalg.matrix_rank(matrix)), 7)
        self.assertLess(float(torch.linalg.cond(matrix)), 3.0)

    def test_field_can_represent_each_context_angle_exactly(self) -> None:
        torch.manual_seed(common.SEED)
        rows = [row(angle) for angle in common.CONTEXT_ANGLES_DEGREES]
        angles, groups = common.encode_rows(
            rows,
            torch.device("cpu"),
            expected_angles=common.CONTEXT_ANGLES_DEGREES,
        )
        target = torch.randn((7, common.RANK, 2), dtype=torch.float32) * 0.2
        basis = common.angular_features(angles)
        solved = torch.linalg.solve(basis, target.reshape(7, -1)).reshape(
            7, common.RANK, 2
        )
        model = common.HarmonicCoefficientField()
        with torch.no_grad():
            model.weights[0] = solved
        torch.testing.assert_close(model(angles, groups), target)

    def test_coordinate_encoder_rejects_duplicate_or_unknown_axes(self) -> None:
        with self.assertRaises(common.R2EError):
            common.encode_rows(
                [row(0), row(0)],
                torch.device("cpu"),
                expected_angles=common.CONTEXT_ANGLES_DEGREES,
            )
        with self.assertRaises(common.R2EError):
            common.encode_rows(
                [row(40, offset=500)],
                torch.device("cpu"),
                expected_angles=common.QUERY_ANGLES_DEGREES,
            )

    def test_complete_context_and_query_grids_are_disjoint(self) -> None:
        context = []
        query = []
        for angle in common.CONTEXT_ANGLES_DEGREES:
            for offset in common.DISTANCE_OFFSETS_MILLIMETRES:
                for microphone in common.MICROPHONE_IDS:
                    context.append(row(angle, offset, microphone))
        for angle in common.QUERY_ANGLES_DEGREES:
            for offset in common.DISTANCE_OFFSETS_MILLIMETRES:
                for microphone in common.MICROPHONE_IDS:
                    query.append(row(angle, offset, microphone))
        train.validate_coordinate_rows(context, query)
        self.assertEqual(len(context), common.r2b.CONTEXT_ROWS)
        self.assertEqual(len(query), common.r2b.QUERY_ROWS)

    def test_synthetic_harmonic_field_converges_under_frozen_objective(self) -> None:
        torch.manual_seed(common.SEED)
        rows = [row(angle) for angle in common.CONTEXT_ANGLES_DEGREES]
        angles, groups = common.encode_rows(
            rows,
            torch.device("cpu"),
            expected_angles=common.CONTEXT_ANGLES_DEGREES,
        )
        teacher = common.HarmonicCoefficientField()
        with torch.no_grad():
            teacher.weights[0].normal_(mean=0.0, std=0.15)
            target = teacher(angles, groups).detach()
        model = common.HarmonicCoefficientField()
        optimizer = torch.optim.AdamW(
            model.parameters(),
            lr=common.r2d_v2.INITIAL_LEARNING_RATE,
            betas=common.r2d_v2.ADAM_BETAS,
            eps=common.r2d_v2.ADAM_EPSILON,
            weight_decay=common.r2d_v2.WEIGHT_DECAY,
        )
        rms = torch.linspace(0.5, 1.5, common.RANK)
        mean_energy = torch.tensor(50.0)
        projection = torch.zeros((common.RANK, 2))
        for step in range(common.STEPS):
            for group in optimizer.param_groups:
                group["lr"] = common.learning_rate_for_step(step)
            objective, _ = common.v1.coefficient_objective(
                model(angles, groups), target, rms, mean_energy, projection
            )
            optimizer.zero_grad(set_to_none=True)
            objective.backward()
            torch.nn.utils.clip_grad_norm_(
                model.parameters(), common.GRADIENT_CLIP_NORM
            )
            optimizer.step()
        final, parts = common.v1.coefficient_objective(
            model(angles, groups), target, rms, mean_energy, projection
        )
        self.assertLess(float(final.detach()), 1.0e-10)
        self.assertLess(
            float(parts["mean_absolute_log_energy_error"].detach()), 1.0e-6
        )

    def test_training_sources_bind_passed_r2d_v2(self) -> None:
        paths = train.implementation_paths()
        self.assertIn(
            "lab/scripts/physical_sound_listener_field_r2d_v2_trainability.py",
            paths,
        )
        self.assertIn("lab/scripts/physical_sound_listener_field_r2e_train.py", paths)
        self.assertEqual(len(common.R2D_V2_MANIFEST_SHA256), 64)
        self.assertEqual(len(common.R2D_V2_RUN_REPORT_SHA256), 64)
        self.assertIn(
            "lab/scripts/physical_sound_listener_field_r2e_evaluate.py",
            evaluate.implementation_paths(),
        )
        self.assertEqual(
            evaluate.SELECTION_POLICY,
            "candidate_must_strictly_beat_all_three_frozen_controls_on_all_"
            "five_primary_endpoints_else_reject_low_rank_coefficient_field",
        )

    def test_post_reject_diagnostic_is_non_authoritative(self) -> None:
        self.assertEqual(len(diagnostic.R2E_EVALUATION_REPORT_SHA256), 64)
        self.assertIn("failure-diagnostic", diagnostic.REPORT_SCHEMA)
        self.assertTrue(diagnostic.ORACLE_ID.startswith("post_reject_"))

    def test_projection_oracle_classifies_representation_ceiling(self) -> None:
        controls = [
            {
                "control_id": "control",
                "aggregate": {endpoint: 1.0 for endpoint in common.PRIMARY_ENDPOINTS},
            }
        ]
        oracle = {
            "aggregate": {
                endpoint: (1.1 if index == 2 else 0.5)
                for index, endpoint in enumerate(common.PRIMARY_ENDPOINTS)
            }
        }
        decision, compared = diagnostic.classify(oracle, controls)
        self.assertEqual(decision, "RepresentationAndInterpolationBothLimited")
        self.assertFalse(compared["passes_frozen_r2c_rule"])

        passing = {
            "aggregate": {endpoint: 0.5 for endpoint in common.PRIMARY_ENDPOINTS}
        }
        decision, compared = diagnostic.classify(passing, controls)
        self.assertEqual(
            decision, "RepresentationSufficientInterpolationOrSamplingLimited"
        )
        self.assertTrue(compared["passes_frozen_r2c_rule"])


if __name__ == "__main__":
    unittest.main()
