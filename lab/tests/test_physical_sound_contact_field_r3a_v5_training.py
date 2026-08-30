from __future__ import annotations

import io
import math
import sys
import tempfile
import unittest
import wave
import zipfile
from pathlib import Path

import numpy as np
import torch

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v5_common as common
import physical_sound_contact_field_r3a_v5_model as codec_model
import physical_sound_contact_field_r3a_v5_training as training


class PhysicalSoundContactFieldR3AV5TrainingTests(unittest.TestCase):
    def test_control_selection_uses_one_clip_from_each_frozen_role(self) -> None:
        corpus = {
            "clips": [
                {"member": "z.wav", "event_group": "z", "role": "train"},
                {
                    "member": "v.wav",
                    "event_group": "validation",
                    "role": "internal_validation",
                },
                {"member": "a.wav", "event_group": "a", "role": "train"},
            ]
        }
        selected = training.select_control_clips(corpus)
        self.assertEqual([item["member"] for item in selected], ["a.wav", "v.wav"])

    def test_control_decode_is_bounded_aligned_and_float32(self) -> None:
        frames = 4_410
        time = np.arange(frames, dtype=np.float64) / common.SOURCE_SAMPLE_RATE_HZ
        mono = 0.3 * np.sin(2.0 * math.pi * 730.0 * time)
        mono[1_000] = 0.9
        pcm = np.round(mono * 32767.0).astype("<i2")
        stereo = np.column_stack((pcm, pcm)).astype("<i2").tobytes()
        wave_buffer = io.BytesIO()
        with wave.open(wave_buffer, "wb") as output:
            output.setnchannels(common.SOURCE_CHANNELS)
            output.setsampwidth(common.SOURCE_SAMPLE_WIDTH_BYTES)
            output.setframerate(common.SOURCE_SAMPLE_RATE_HZ)
            output.writeframes(stereo)
        payload = wave_buffer.getvalue()
        descriptor = {
            "member": "corpus/control.wav",
            "member_sha256": common.sha256_bytes(payload),
            "role": "train",
            "event_group": "control",
        }
        with tempfile.TemporaryDirectory() as directory:
            archive_path = Path(directory) / "control.zip"
            with zipfile.ZipFile(archive_path, "w") as archive:
                archive.writestr(descriptor["member"], payload)
            with zipfile.ZipFile(archive_path) as archive:
                decoded = training.decode_control_clip(archive, descriptor)
        self.assertEqual(decoded.samples.dtype, np.dtype("<f4"))
        self.assertEqual(decoded.samples.size, common.TRAINING_SEGMENT_SAMPLES)
        self.assertEqual(int(np.argmax(np.abs(decoded.samples))), 512)
        self.assertLessEqual(float(np.max(np.abs(decoded.samples))), 0.920001)
        self.assertEqual(decoded.source_scalar_samples_decoded, frames * 2)

    def test_full_loss_is_finite_and_zero_for_matching_waveform_terms(self) -> None:
        time = torch.arange(common.TRAINING_SEGMENT_SAMPLES, dtype=torch.float32)
        target = (
            0.5
            * torch.exp(-time / 12_000.0)
            * torch.sin(2.0 * math.pi * 880.0 * time / common.TARGET_SAMPLE_RATE_HZ)
        ).reshape(1, 1, -1)
        zero = target.new_zeros(())
        loss, terms = training.frozen_reconstruction_loss(target, target, zero, zero)
        self.assertTrue(torch.isfinite(loss))
        self.assertEqual(float(terms["waveform_l1"]), 0.0)
        self.assertEqual(float(terms["complex_stft"]), 0.0)
        self.assertEqual(float(terms["log_spectrum"]), 0.0)
        self.assertEqual(float(terms["target_peak_emphasis"]), 0.0)
        self.assertEqual(float(terms["log_mel"]), 0.0)
        self.assertEqual(float(terms["envelope"]), 0.0)
        self.assertEqual(float(terms["decay_energy_curve"]), 0.0)

    def test_codebook_warm_start_activates_each_micro_residual_stage(self) -> None:
        torch.manual_seed(23)
        model = codec_model.NeuralImpactCodec(codec_model.CodecConfig.micro())
        samples = math.prod(model.config.strides) * 16
        time = torch.arange(samples, dtype=torch.float32)
        target = (
            torch.sin(2.0 * math.pi * time / 17.0)
            + 0.3 * torch.sin(2.0 * math.pi * time / 7.0)
        ).reshape(1, 1, -1)
        state_hash = training.initialize_codebooks_from_batch(model, target, 2)
        with torch.inference_mode():
            _, codes, _, _ = model(target, 2)
        self.assertEqual(len(state_hash), 64)
        self.assertTrue(
            all(torch.unique(codes[:, index]).numel() >= 2 for index in range(2))
        )


if __name__ == "__main__":
    unittest.main()
