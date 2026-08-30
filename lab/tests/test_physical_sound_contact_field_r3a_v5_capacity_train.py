from __future__ import annotations

import math
import sys
import unittest
from pathlib import Path

import numpy as np
import torch

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v5_capacity_train as capacity_train
import physical_sound_contact_field_r3a_v5_common as common
import physical_sound_contact_field_r3a_v5_model as codec_model


class PhysicalSoundContactFieldR3AV5CapacityTrainTests(unittest.TestCase):
    @staticmethod
    def _validation_metrics(
        *,
        log_spectrum: float,
        rms_ratio: float,
        unique_codes: list[int],
        latent_rms: float = 0.5,
        latent_absolute: float = 2.0,
        diversity_ratio: float = 0.5,
    ) -> dict[str, object]:
        return {
            "mean": {"log_spectrum": log_spectrum},
            "unique_codes_per_quantizer": unique_codes,
            "waveform_diagnostics": {
                "mean_output_target_rms_ratio": rms_ratio,
                "output_diversity_ratio": diversity_ratio,
            },
            "latent_diagnostics": {
                "maximum_rms": latent_rms,
                "maximum_absolute": latent_absolute,
            },
        }

    def test_learning_rate_matches_frozen_warmup_and_cosine_endpoints(self) -> None:
        base = common.TRAINING_CONFIG["learning_rate"]
        self.assertEqual(
            capacity_train.learning_rate_for_step(1),
            base / common.TRAINING_CONFIG["warmup_steps"],
        )
        self.assertEqual(
            capacity_train.learning_rate_for_step(
                common.TRAINING_CONFIG["warmup_steps"]
            ),
            base,
        )
        self.assertTrue(
            math.isclose(
                capacity_train.learning_rate_for_step(
                    common.TRAINING_CONFIG["maximum_steps"]
                ),
                base * 0.05,
            )
        )

    def test_stateless_sampler_repeats_item_crop_and_mask(self) -> None:
        items = [
            capacity_train.WaveformItem(
                id=f"item-{index}",
                role="train",
                source_kind="fixture",
                source_group=f"group-{index}",
                samples=np.linspace(-0.9, 0.9, 60_000 + index, dtype="<f4"),
                samples_sha256="0" * 64,
                output_gain=1.0,
            )
            for index in range(3)
        ]
        first = capacity_train.segment_for_step(items, "rvq-6kbps", 77)
        second = capacity_train.segment_for_step(items, "rvq-6kbps", 77)
        self.assertEqual(first[0].id, second[0].id)
        self.assertEqual(first[3], second[3])
        np.testing.assert_array_equal(first[1], second[1])
        np.testing.assert_array_equal(first[2], second[2])

    def test_frontier_capacities_share_the_same_sampling_stream(self) -> None:
        items = [
            capacity_train.WaveformItem(
                id=f"item-{index}",
                role="train",
                source_kind="fixture",
                source_group=f"group-{index}",
                samples=np.linspace(-0.9, 0.9, 60_000 + index, dtype="<f4"),
                samples_sha256="0" * 64,
                output_gain=1.0,
            )
            for index in range(3)
        ]
        first = capacity_train.segment_for_step(items, "rvq-6kbps", 77)
        second = capacity_train.segment_for_step(items, "rvq-24kbps", 77)
        self.assertEqual(first[0].id, second[0].id)
        self.assertEqual(first[3], second[3])
        np.testing.assert_array_equal(first[1], second[1])
        np.testing.assert_array_equal(first[2], second[2])

    def test_fit_projection_decodes_only_first_three_rows(self) -> None:
        values = np.zeros((4, 64), dtype="<f4")
        values[0, 3] = 0.2
        values[1, 4] = 0.4
        values[2, 5] = 0.6
        values[3] = np.nan
        metadata = {
            "contacts": [
                {"role": "fit"},
                {"role": "fit"},
                {"role": "fit"},
                {"role": "representation_development"},
            ]
        }
        items = capacity_train.extract_fit_rows(values, metadata, "fixture")
        self.assertEqual(len(items), 3)
        self.assertTrue(all(np.isfinite(item.samples).all() for item in items))
        self.assertTrue(all(item.role == "fit" for item in items))

    def test_short_segment_mask_excludes_right_padding(self) -> None:
        item = capacity_train.WaveformItem(
            id="short",
            role="train",
            source_kind="fixture",
            source_group="fixture",
            samples=np.ones(1_000, dtype="<f4") * 0.2,
            samples_sha256="0" * 64,
            output_gain=1.0,
        )
        _, segment, mask, _ = capacity_train.segment_for_step([item], "rvq-12kbps", 4)
        self.assertEqual(segment.size, common.TRAINING_SEGMENT_SAMPLES)
        self.assertEqual(mask.size, common.TRAINING_SEGMENT_SAMPLES)
        self.assertEqual(float(mask.sum()), 1_000.0)
        self.assertEqual(float((segment * (1.0 - mask)).sum()), 0.0)

    def test_curriculum_phases_have_exact_transition_endpoints(self) -> None:
        continuous = capacity_train.curriculum_for_step(2_000)
        ramp_start = capacity_train.curriculum_for_step(2_001)
        quantized = capacity_train.curriculum_for_step(4_000)
        refinement_start = capacity_train.curriculum_for_step(4_001)
        refinement = capacity_train.curriculum_for_step(6_000)
        self.assertEqual(continuous["phase"], "continuous_bootstrap")
        self.assertEqual(continuous["quantizer_mix"], 0.0)
        self.assertTrue(math.isclose(ramp_start["quantizer_mix"], 0.0005))
        self.assertEqual(quantized["quantizer_mix"], 1.0)
        self.assertEqual(quantized["full_loss_weight"], 0.0)
        self.assertEqual(refinement_start["phase"], "quantized_decoder_refinement")
        self.assertEqual(refinement_start["full_loss_weight"], 0.0)
        self.assertEqual(refinement["full_loss_weight"], 0.0)

    def test_normalized_bootstrap_loss_is_zero_for_matching_signal(self) -> None:
        target = torch.linspace(-0.4, 0.4, 8_192).reshape(1, 1, -1)
        total, terms = capacity_train.normalized_bootstrap_loss(target, target)
        self.assertEqual(float(total), 0.0)
        self.assertTrue(all(float(value) == 0.0 for value in terms.values()))

    def test_quantizer_ramp_freezes_codec_and_refinement_unfreezes_decoder(self) -> None:
        model = codec_model.NeuralImpactCodec(codec_model.CodecConfig.micro())
        capacity_train.configure_phase_trainability(
            model, "quantizer_ramp_bootstrap"
        )
        self.assertFalse(any(value.requires_grad for value in model.encoder.parameters()))
        self.assertFalse(any(value.requires_grad for value in model.decoder.parameters()))
        self.assertTrue(all(value.requires_grad for value in model.quantizer.parameters()))
        capacity_train.configure_phase_trainability(
            model, "quantized_decoder_refinement"
        )
        self.assertFalse(any(value.requires_grad for value in model.encoder.parameters()))
        self.assertTrue(all(value.requires_grad for value in model.decoder.parameters()))
        self.assertTrue(all(value.requires_grad for value in model.quantizer.parameters()))

    def test_anti_collapse_gate_waits_for_quantizer_then_passes_signal(self) -> None:
        initial = self._validation_metrics(
            log_spectrum=20.0,
            rms_ratio=0.001,
            unique_codes=[8, 8, 8, 8],
        )
        current = self._validation_metrics(
            log_spectrum=19.0,
            rms_ratio=0.4,
            unique_codes=[7, 6, 5, 4],
        )
        pending = capacity_train.assess_anti_collapse_gate(
            initial,
            current,
            4,
            capacity_train.CURRICULUM["quantizer_ramp_end_step"] - 1,
        )
        passed = capacity_train.assess_anti_collapse_gate(
            initial,
            current,
            4,
            capacity_train.CURRICULUM["quantizer_ramp_end_step"],
        )
        self.assertEqual(pending["status"], "PendingWarmup")
        self.assertIsNone(pending["passed"])
        self.assertEqual(passed["status"], "Passed")
        self.assertTrue(passed["passed"])

    def test_curriculum_bootstrap_gate_ignores_uninitialized_codes(self) -> None:
        initial = self._validation_metrics(
            log_spectrum=20.0,
            rms_ratio=0.001,
            unique_codes=[0, 0, 0, 0],
        )
        current = self._validation_metrics(
            log_spectrum=18.0,
            rms_ratio=0.5,
            unique_codes=[0, 0, 0, 0],
        )
        result = capacity_train.assess_curriculum_gate(
            initial,
            current,
            4,
            capacity_train.CURRICULUM["continuous_bootstrap_end_step"],
        )
        self.assertTrue(result["passed"])
        self.assertFalse(result["admissible_for_checkpoint_selection"])
        self.assertNotIn("minimum_unique_codes_per_quantizer", result["checks"])

    def test_anti_collapse_gate_rejects_silent_constant_code_collapse(self) -> None:
        initial = self._validation_metrics(
            log_spectrum=20.0,
            rms_ratio=0.001,
            unique_codes=[8, 8, 8, 8],
        )
        collapsed = self._validation_metrics(
            log_spectrum=20.0,
            rms_ratio=0.03,
            unique_codes=[2, 2, 1, 1],
            diversity_ratio=0.001,
        )
        result = capacity_train.assess_anti_collapse_gate(
            initial,
            collapsed,
            4,
            capacity_train.CURRICULUM["quantizer_ramp_end_step"],
        )
        self.assertEqual(result["status"], "Rejected")
        self.assertFalse(result["passed"])
        self.assertEqual(
            result["failed_checks"],
            [
                "log_spectrum_relative_improvement",
                "mean_output_target_rms_ratio",
                "minimum_unique_codes_per_quantizer",
                "output_diversity_ratio",
            ],
        )


if __name__ == "__main__":
    unittest.main()
