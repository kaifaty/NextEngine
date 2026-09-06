from __future__ import annotations

import math
import struct
import sys
import unittest
from pathlib import Path

import torch

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v5_common as common
import physical_sound_contact_field_r3a_v5_model as model
import physical_sound_contact_field_r3a_v5_preflight as preflight


class PhysicalSoundContactFieldR3AV5Tests(unittest.TestCase):
    def test_capacity_rates_and_records_are_bounded(self) -> None:
        descriptors = preflight.capacity_descriptors()
        self.assertEqual(
            [item["nominal_bits_per_second"] for item in descriptors],
            [6_000, 12_000, 24_000],
        )
        self.assertEqual([item["quantizers"] for item in descriptors], [4, 8, 16])
        self.assertTrue(
            all(item["analysis_encoded_bytes"] < 64 * 1024 for item in descriptors)
        )

    def test_group_split_is_deterministic_and_disjoint(self) -> None:
        groups = [f"group-{index}" for index in range(17)]
        first = {group: common.group_role(group, groups) for group in groups}
        second = {
            group: common.group_role(group, list(reversed(groups))) for group in groups
        }
        self.assertEqual(first, second)
        self.assertEqual(
            sum(role == "internal_validation" for role in first.values()), 4
        )
        self.assertEqual(sum(role == "train" for role in first.values()), 13)

    def test_pcm_header_parser_accepts_frozen_stereo_format(self) -> None:
        frames = 20
        data = bytes(frames * common.SOURCE_CHANNELS * common.SOURCE_SAMPLE_WIDTH_BYTES)
        fmt = struct.pack(
            "<HHIIHH",
            1,
            common.SOURCE_CHANNELS,
            common.SOURCE_SAMPLE_RATE_HZ,
            common.SOURCE_SAMPLE_RATE_HZ * 4,
            4,
            16,
        )
        payload = (
            b"RIFF"
            + struct.pack("<I", 4 + 8 + len(fmt) + 8 + len(data))
            + b"WAVEfmt "
            + struct.pack("<I", len(fmt))
            + fmt
            + b"data"
            + struct.pack("<I", len(data))
            + data
        )
        descriptor = common.parse_pcm_wave_header(payload, "fixture")
        self.assertEqual(descriptor["frames"], frames)
        broken = bytearray(payload)
        broken[22:24] = struct.pack("<H", 1)
        with self.assertRaises(common.V5Error):
            common.parse_pcm_wave_header(bytes(broken), "broken")

    def test_micro_codec_inverse_preserves_sample_count_and_code_shape(self) -> None:
        model.configure_torch()
        torch.manual_seed(5)
        codec = model.NeuralImpactCodec(model.CodecConfig.micro()).eval()
        samples = math.prod(codec.config.strides) * 8
        value = torch.zeros(1, 1, samples)
        with torch.inference_mode():
            output, codes, _, _ = codec(value, 2)
        self.assertEqual(output.shape, value.shape)
        self.assertEqual(
            codes.shape, (1, 2, samples // math.prod(codec.config.strides))
        )

    def test_codec_uses_factorized_normalized_quantizers(self) -> None:
        codec = model.NeuralImpactCodec(model.CodecConfig.micro())
        self.assertEqual(len(codec.quantizer.layers), 2)
        for layer in codec.quantizer.layers:
            self.assertEqual(layer.codebook.embedding_dim, 2)
            self.assertEqual(layer.input_projection.kernel_size, (1,))
            self.assertEqual(layer.output_projection.kernel_size, (1,))
            self.assertTrue(hasattr(layer.input_projection, "weight_g"))
            self.assertTrue(hasattr(layer.input_projection, "weight_v"))

    def test_rvq_produces_finite_gradients_through_sequential_residuals(self) -> None:
        torch.manual_seed(31)
        config = model.CodecConfig.micro()
        quantizer = model.ResidualVectorQuantizer(config)
        value = torch.randn(1, config.latent_dimension, 8, requires_grad=True)
        quantized, codes, codebook_loss, commitment_loss = quantizer(value, 2)
        loss = quantized.square().mean() + codebook_loss + commitment_loss
        loss.backward()
        self.assertEqual(quantized.shape, value.shape)
        self.assertEqual(codes.shape, (1, 2, 8))
        self.assertIsNotNone(value.grad)
        self.assertTrue(torch.isfinite(value.grad).all())

    def test_state_hash_is_exact_for_seeded_initialization(self) -> None:
        torch.manual_seed(12)
        first = model.NeuralImpactCodec(model.CodecConfig.micro())
        torch.manual_seed(12)
        second = model.NeuralImpactCodec(model.CodecConfig.micro())
        self.assertEqual(model.state_sha256(first), model.state_sha256(second))

    def test_frozen_controls_are_exact_and_micro_model_learns(self) -> None:
        full = model.full_model_control()
        micro = model.micro_overfit_control()
        self.assertTrue(full["exact_repeat"])
        self.assertTrue(micro["exact_repeat"])
        self.assertTrue(micro["passed"])
        self.assertLessEqual(micro["result"]["improvement_ratio"], 0.45)

    def test_manifest_forbids_training_and_development_access(self) -> None:
        manifest = preflight.build_manifest(
            {key: "0" * 64 for key in common.IMPLEMENTATION_FILES},
            {"control": True},
            {"clips": [], "groups": []},
            {
                "development_waveform_samples_decoded": 0,
                "sealed_waveform_samples_decoded": 0,
            },
            {"full_model": {"exact_repeat": True}, "micro_overfit": {"passed": True}},
        )
        self.assertFalse(manifest["neural_training_authorized"])
        self.assertEqual(manifest["development_waveform_samples_decoded"], 0)
        self.assertEqual(
            manifest["training_environment"]["status"],
            "MUST_FREEZE_BEFORE_REAL_TRAINING",
        )


if __name__ == "__main__":
    unittest.main()
