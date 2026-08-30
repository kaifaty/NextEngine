#!/usr/bin/env python3
"""Train one frozen R3A V5 neural representation capacity externally."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import os
import wave
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_contact_field_r3a_v5_common as common
import physical_sound_contact_field_r3a_v5_model as codec_model
import physical_sound_contact_field_r3a_v5_training as training
import physical_sound_contact_field_r3a_v5_training_preflight as training_preflight
import scipy.signal
import torch
from torch.nn import functional

RUN_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-capacity-run.v1"
STATE_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-capacity-state.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-capacity.report.v1"
REVISION = "three-capacity-training-v4-frozen-quantizer-ramp"
TRAINING_PREFLIGHT_MANIFEST_SHA256 = (
    "d53561fdd4cc4a662dfe2d750a2f3109fb5d63dc154b14f73ce92cb475d60b95"
)
V4_MANIFEST_SHA256 = "6f9fe00ea14c99da2b2fc8b71b22af6772725ac5f28430a2417b8717819386b1"
METRIC_INTERVAL_STEPS = 100
FIT_CONTACTS_PER_OBJECT = 3
INTERNET_TRAIN_CLIP_COUNT = 56
INTERNET_VALIDATION_CLIP_COUNT = 17
TOTAL_TRAIN_ITEM_COUNT = 68
IMPLEMENTATION_FILES = {
    "common": "physical_sound_contact_field_r3a_v5_common.py",
    "model": "physical_sound_contact_field_r3a_v5_model.py",
    "training": "physical_sound_contact_field_r3a_v5_training.py",
    "training_preflight": "physical_sound_contact_field_r3a_v5_training_preflight.py",
    "capacity_train": "physical_sound_contact_field_r3a_v5_capacity_train.py",
}
CURRICULUM = {
    "continuous_bootstrap_end_step": 2_000,
    "quantizer_ramp_end_step": 4_000,
    "full_loss_ramp_end_step": 6_000,
    "codebook_initialization_segments": 8,
    "bootstrap_loss": {
        "relative_waveform_l1_weight": 1.0,
        "relative_derivative_l1_weight": 1.0,
        "relative_complex_stft_weight": 2.0,
        "complex_stft_window_samples": [512, 2_048, 8_192],
    },
    "quantized_latent_match_weight": 1.0,
    "quantizer_ramp_trainable_modules": ["quantizer"],
    "full_loss_after_quantized_gate": True,
    "bootstrap_auxiliary_retained": True,
}
BOOTSTRAP_GATE = {
    "evaluation_step": CURRICULUM["continuous_bootstrap_end_step"],
    "minimum_mean_output_target_rms_ratio": 0.10,
    "minimum_log_spectrum_relative_improvement": 0.005,
    "maximum_encoder_latent_rms": 10.0,
    "maximum_encoder_latent_absolute": 100.0,
    "minimum_output_diversity_ratio": 0.01,
}
ANTI_COLLAPSE_GATE = {
    "evaluation_step": CURRICULUM["quantizer_ramp_end_step"],
    "minimum_mean_output_target_rms_ratio": 0.10,
    "minimum_log_spectrum_relative_improvement": 0.005,
    "minimum_unique_codes_per_quantizer": 2,
    "maximum_encoder_latent_rms": 10.0,
    "maximum_encoder_latent_absolute": 100.0,
    "minimum_output_diversity_ratio": 0.01,
}


@dataclass(frozen=True)
class WaveformItem:
    id: str
    role: str
    source_kind: str
    source_group: str
    samples: np.ndarray
    samples_sha256: str
    output_gain: float


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--v5-manifest", required=True, type=Path)
    parser.add_argument("--training-preflight-manifest", required=True, type=Path)
    parser.add_argument("--v4-manifest", required=True, type=Path)
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument(
        "--fit-extraction",
        action="append",
        default=[],
        metavar="OBJECT_ID=EXTERNAL_DIRECTORY",
    )
    parser.add_argument(
        "--capacity-id",
        required=True,
        choices=[item["id"] for item in common.CAPACITIES],
    )
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--resume", action="store_true")
    parser.add_argument(
        "--stop-after-step",
        type=int,
        help="finish a bounded research discriminator at this checkpoint",
    )
    return parser.parse_args()


def parse_fit_extractions(values: list[str]) -> dict[str, Path]:
    result: dict[str, Path] = {}
    for value in values:
        object_id, separator, path = value.partition("=")
        if not separator or not object_id or not path or object_id in result:
            raise common.V5Error("invalid or duplicate --fit-extraction")
        result[object_id] = Path(path)
    if sorted(result) != sorted(common.V4_OBJECT_IDS):
        raise common.V5Error("fit extraction object set changed")
    return result


def _require_external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise common.V5Error(f"{label} must be an external directory")
    return resolved


def load_parent_manifest(
    root: Path,
    path: Path,
    label: str,
    expected_sha256: str,
) -> dict[str, Any]:
    resolved = common.require_external_file(root, path, label)
    payload, value = common.load_json(resolved, label, canonical=True)
    if common.sha256_bytes(payload) != expected_sha256:
        raise common.V5Error(f"{label} hash changed")
    return value


def _decode_internet_member(
    archive: zipfile.ZipFile,
    descriptor: dict[str, Any],
) -> WaveformItem:
    payload = archive.read(descriptor["member"])
    if common.sha256_bytes(payload) != descriptor["member_sha256"]:
        raise common.V5Error("V5 internet clip payload changed")
    with wave.open(io.BytesIO(payload), "rb") as source:
        if (
            source.getnchannels() != common.SOURCE_CHANNELS
            or source.getsampwidth() != common.SOURCE_SAMPLE_WIDTH_BYTES
            or source.getframerate() != common.SOURCE_SAMPLE_RATE_HZ
            or source.getcomptype() != "NONE"
            or source.getnframes() != descriptor["frames"]
        ):
            raise common.V5Error("V5 internet clip PCM identity changed")
        raw = source.readframes(source.getnframes())
    scalars = np.frombuffer(raw, dtype="<i2")
    if scalars.size != descriptor["frames"] * common.SOURCE_CHANNELS:
        raise common.V5Error("V5 internet clip scalar count changed")
    stereo = scalars.astype(np.float64).reshape(-1, common.SOURCE_CHANNELS)
    mono = stereo.mean(axis=1) / 32768.0
    resampled = scipy.signal.resample_poly(mono, 160, 147)
    if not np.isfinite(resampled).all() or not np.any(resampled):
        raise common.V5Error("V5 internet clip is silent or non-finite")
    peak = float(np.max(np.abs(resampled)))
    output_gain = 0.92 / peak
    normalized = np.asarray(resampled * output_gain, dtype="<f4")
    return WaveformItem(
        id=f"internet:{descriptor['member']}",
        role=descriptor["role"],
        source_kind="internet_impact",
        source_group=descriptor["event_group"],
        samples=normalized,
        samples_sha256=common.sha256_bytes(normalized.tobytes()),
        output_gain=output_gain,
    )


def load_internet_waveforms(
    archive_path: Path,
    corpus: dict[str, Any],
) -> tuple[list[WaveformItem], list[WaveformItem]]:
    with zipfile.ZipFile(archive_path) as archive:
        items = [
            _decode_internet_member(archive, descriptor)
            for descriptor in sorted(corpus["clips"], key=lambda item: item["member"])
        ]
    train_items = [item for item in items if item.role == "train"]
    validation_items = [item for item in items if item.role == "internal_validation"]
    if (
        len(train_items) != INTERNET_TRAIN_CLIP_COUNT
        or len(validation_items) != INTERNET_VALIDATION_CLIP_COUNT
    ):
        raise common.V5Error("V5 internet role cardinality changed")
    if {item.source_group for item in train_items} & {
        item.source_group for item in validation_items
    }:
        raise common.V5Error("V5 internet groups leak across roles")
    return train_items, validation_items


def extract_fit_rows(
    values: np.ndarray,
    metadata: dict[str, Any],
    object_id: str,
) -> list[WaveformItem]:
    contacts = metadata.get("contacts", [])
    if (
        values.ndim != 2
        or values.shape[0] != 4
        or len(contacts) != 4
        or [item.get("role") for item in contacts]
        != ["fit", "fit", "fit", "representation_development"]
    ):
        raise common.V5Error(f"V5 fit extraction changed for {object_id}")
    result = []
    for index in range(FIT_CONTACTS_PER_OBJECT):
        samples = np.asarray(values[index], dtype="<f4")
        if not np.isfinite(samples).all() or not np.any(samples):
            raise common.V5Error(
                f"V5 fit waveform is silent or non-finite: {object_id}"
            )
        peak = float(np.max(np.abs(samples)))
        output_gain = 0.92 / peak
        normalized = np.asarray(samples * output_gain, dtype="<f4")
        result.append(
            WaveformItem(
                id=f"fit:{object_id}:{index}",
                role="fit",
                source_kind="authorized_fit_contact",
                source_group=object_id,
                samples=normalized,
                samples_sha256=common.sha256_bytes(normalized.tobytes()),
                output_gain=output_gain,
            )
        )
    return result


def load_fit_waveforms(
    root: Path,
    v4_manifest: dict[str, Any],
    directories: dict[str, Path],
) -> tuple[list[WaveformItem], list[dict[str, Any]]]:
    by_id = {item["id"]: item for item in v4_manifest["objects"]}
    waveforms = []
    lineage = []
    for object_id in common.V4_OBJECT_IDS:
        descriptor = by_id[object_id]
        directory = _require_external_directory(
            root, directories[object_id], f"{object_id} fit extraction"
        )
        metadata_path = directory / "contacts-metadata.json"
        contacts_path = directory / "contacts.npy"
        report_path = directory / "report.json"
        if not all(
            path.is_file() for path in (metadata_path, contacts_path, report_path)
        ):
            raise common.V5Error(f"V5 fit extraction files missing for {object_id}")
        metadata_payload, metadata = common.load_json(
            metadata_path, f"{object_id} fit metadata", canonical=True
        )
        report_payload, report = common.load_json(
            report_path, f"{object_id} fit report", canonical=True
        )
        if (
            common.sha256_bytes(metadata_payload) != descriptor["metadata_sha256"]
            or common.sha256_bytes(report_payload) != descriptor["report_sha256"]
            or common.sha256_file(contacts_path) != descriptor["contacts_sha256"]
            or metadata.get("sample_rate_hz") != common.TARGET_SAMPLE_RATE_HZ
            or metadata.get("sample_count") != descriptor["sample_count"]
            or report.get("sealed_waveform_samples_decoded") != 0
        ):
            raise common.V5Error(f"V5 fit extraction lineage changed for {object_id}")
        values = np.load(contacts_path, mmap_mode="r", allow_pickle=False)
        waveforms.extend(extract_fit_rows(values, metadata, object_id))
        lineage.append(
            {
                "object_id": object_id,
                "directory": str(directory),
                "metadata_sha256": descriptor["metadata_sha256"],
                "report_sha256": descriptor["report_sha256"],
                "contacts_sha256": descriptor["contacts_sha256"],
                "fit_rows_decoded": FIT_CONTACTS_PER_OBJECT,
                "development_rows_decoded": 0,
                "sealed_rows_decoded": 0,
            }
        )
    if len(waveforms) != 12:
        raise common.V5Error("V5 fit waveform cardinality changed")
    return waveforms, lineage


def _aligned_segment(samples: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    peak_index = int(np.argmax(np.abs(samples)))
    prefix = max(0, 512 - peak_index)
    start = max(0, peak_index - 512)
    available = np.pad(samples[start:], (prefix, 0))
    valid = np.pad(np.ones(samples[start:].size, dtype="<f4"), (prefix, 0))
    if available.size < common.TRAINING_SEGMENT_SAMPLES:
        padding = common.TRAINING_SEGMENT_SAMPLES - available.size
        available = np.pad(available, (0, padding))
        valid = np.pad(valid, (0, padding))
    return (
        np.asarray(available[: common.TRAINING_SEGMENT_SAMPLES], dtype="<f4"),
        np.asarray(valid[: common.TRAINING_SEGMENT_SAMPLES], dtype="<f4"),
    )


def segment_for_step(
    items: list[WaveformItem],
    capacity_id: str,
    step: int,
) -> tuple[WaveformItem, np.ndarray, np.ndarray, str]:
    seed_bytes = hashlib.sha256(
        f"{common.TRAINING_CONFIG['random_seed']}\0{capacity_id}\0{step}".encode()
    ).digest()[:16]
    rng = np.random.default_rng(int.from_bytes(seed_bytes, "little"))
    item = items[int(rng.integers(0, len(items)))]
    if (
        float(rng.random())
        < common.TRAINING_CONFIG["segment_policy"]["onset_aligned_probability"]
    ):
        segment, mask = _aligned_segment(item.samples)
        return item, segment, mask, "onset_aligned"
    if item.samples.size <= common.TRAINING_SEGMENT_SAMPLES:
        segment = np.pad(
            item.samples,
            (0, common.TRAINING_SEGMENT_SAMPLES - item.samples.size),
        ).astype("<f4", copy=False)
        mask = np.pad(
            np.ones(item.samples.size, dtype="<f4"),
            (0, common.TRAINING_SEGMENT_SAMPLES - item.samples.size),
        )
        return item, segment, mask, "uniform_short_masked"
    maximum_start = item.samples.size - common.TRAINING_SEGMENT_SAMPLES
    start = int(rng.integers(0, maximum_start + 1))
    segment = np.asarray(
        item.samples[start : start + common.TRAINING_SEGMENT_SAMPLES], dtype="<f4"
    )
    mask = np.ones(common.TRAINING_SEGMENT_SAMPLES, dtype="<f4")
    return item, segment, mask, "uniform"


def learning_rate_for_step(step: int) -> float:
    base = common.TRAINING_CONFIG["learning_rate"]
    warmup = common.TRAINING_CONFIG["warmup_steps"]
    maximum = common.TRAINING_CONFIG["maximum_steps"]
    if step <= 0 or step > maximum:
        raise common.V5Error("V5 learning-rate step is outside the frozen run")
    if step <= warmup:
        return base * step / warmup
    progress = (step - warmup) / (maximum - warmup)
    multiplier = 0.05 + 0.95 * 0.5 * (1.0 + math.cos(math.pi * progress))
    return base * multiplier


def _tensor(value: np.ndarray, device: torch.device) -> torch.Tensor:
    return torch.from_numpy(value.copy()).reshape(1, 1, -1).to(device)


def curriculum_for_step(step: int) -> dict[str, float | str]:
    continuous_end = CURRICULUM["continuous_bootstrap_end_step"]
    quantizer_end = CURRICULUM["quantizer_ramp_end_step"]
    full_end = CURRICULUM["full_loss_ramp_end_step"]
    if step <= 0 or step > common.TRAINING_CONFIG["maximum_steps"]:
        raise common.V5Error("V5 curriculum step is outside the frozen run")
    if step <= continuous_end:
        return {"phase": "continuous_bootstrap", "quantizer_mix": 0.0, "full_loss_weight": 0.0}
    if step <= quantizer_end:
        return {
            "phase": "quantizer_ramp_bootstrap",
            "quantizer_mix": (step - continuous_end) / (quantizer_end - continuous_end),
            "full_loss_weight": 0.0,
        }
    return {
        "phase": "quantized_full_loss_ramp",
        "quantizer_mix": 1.0,
        "full_loss_weight": min(1.0, (step - quantizer_end) / (full_end - quantizer_end)),
    }


def _forward_curriculum(
    model: codec_model.NeuralImpactCodec,
    target: torch.Tensor,
    quantizers: int,
    quantizer_mix: float,
) -> tuple[
    torch.Tensor,
    torch.Tensor,
    torch.Tensor,
    torch.Tensor,
    torch.Tensor,
    torch.Tensor,
]:
    latent = model.encoder(target)
    if quantizer_mix == 0.0:
        codes = torch.zeros(
            target.shape[0],
            quantizers,
            latent.shape[-1],
            dtype=torch.long,
            device=target.device,
        )
        zero = latent.new_zeros(())
        return model.decoder(latent), codes, zero, zero, latent, latent
    quantized, codes, codebook_loss, commitment_loss = model.quantizer(
        latent, quantizers
    )
    mixed = torch.lerp(latent, quantized, quantizer_mix)
    return (
        model.decoder(mixed),
        codes,
        codebook_loss,
        commitment_loss,
        latent,
        quantized,
    )


def normalized_bootstrap_loss(
    output: torch.Tensor, target: torch.Tensor
) -> tuple[torch.Tensor, dict[str, torch.Tensor]]:
    def relative_l1(first: torch.Tensor, second: torch.Tensor) -> torch.Tensor:
        return functional.l1_loss(first, second) / second.abs().mean().clamp_min(
            1.0e-4
        )

    terms = {
        "relative_waveform_l1": relative_l1(output, target),
        "relative_derivative_l1": relative_l1(
            torch.diff(output, dim=-1), torch.diff(target, dim=-1)
        ),
    }
    complex_losses = []
    for window_samples in CURRICULUM["bootstrap_loss"][
        "complex_stft_window_samples"
    ]:
        output_stft = training._stft(output, window_samples, window_samples // 4)
        target_stft = training._stft(target, window_samples, window_samples // 4)
        complex_losses.append(
            (output_stft - target_stft).abs().mean()
            / target_stft.abs().mean().clamp_min(1.0e-5)
        )
    terms["relative_complex_stft"] = torch.stack(complex_losses).mean()
    total = (
        CURRICULUM["bootstrap_loss"]["relative_waveform_l1_weight"]
        * terms["relative_waveform_l1"]
        + CURRICULUM["bootstrap_loss"]["relative_derivative_l1_weight"]
        * terms["relative_derivative_l1"]
        + CURRICULUM["bootstrap_loss"]["relative_complex_stft_weight"]
        * terms["relative_complex_stft"]
    )
    return total, terms


def configure_phase_trainability(
    model: codec_model.NeuralImpactCodec, phase: str
) -> None:
    quantizer_only = phase == "quantizer_ramp_bootstrap"
    model.encoder.requires_grad_(not quantizer_only)
    model.decoder.requires_grad_(not quantizer_only)
    model.quantizer.requires_grad_(True)


def _train_step(
    model: codec_model.NeuralImpactCodec,
    optimizer: torch.optim.Optimizer,
    target: torch.Tensor,
    mask: torch.Tensor,
    quantizers: int,
    step: int,
) -> dict[str, float | str]:
    rate = learning_rate_for_step(step)
    for group in optimizer.param_groups:
        group["lr"] = rate
    curriculum = curriculum_for_step(step)
    configure_phase_trainability(model, str(curriculum["phase"]))
    model.train()
    (
        output,
        _,
        codebook_loss,
        commitment_loss,
        latent,
        quantized,
    ) = _forward_curriculum(
        model, target, quantizers, float(curriculum["quantizer_mix"])
    )
    masked_output = output * mask
    masked_target = target * mask
    bootstrap_total, bootstrap_terms = normalized_bootstrap_loss(
        masked_output, masked_target
    )
    quantizer_regularization = codebook_loss + 0.25 * commitment_loss
    if float(curriculum["quantizer_mix"]) > 0.0:
        latent_match = functional.mse_loss(quantized, latent.detach()) / latent.detach().square().mean().clamp_min(1.0e-6)
    else:
        latent_match = latent.new_zeros(())
    full_loss_weight = float(curriculum["full_loss_weight"])
    if full_loss_weight > 0.0:
        full_total, full_terms = training.frozen_reconstruction_loss(
            masked_output,
            masked_target,
            codebook_loss,
            commitment_loss,
        )
    else:
        full_total = latent.new_zeros(())
        full_terms = {}
    total = (
        bootstrap_total
        + quantizer_regularization
        + CURRICULUM["quantized_latent_match_weight"] * latent_match
        + full_loss_weight * full_total
    )
    optimizer.zero_grad(set_to_none=True)
    total.backward()
    gradient_norm = torch.nn.utils.clip_grad_norm_(
        model.parameters(), common.TRAINING_CONFIG["gradient_clip_norm"]
    )
    if not torch.isfinite(gradient_norm):
        raise common.V5Error("V5 capacity gradient is non-finite")
    optimizer.step()
    return {
        "phase": str(curriculum["phase"]),
        "quantizer_mix": float(curriculum["quantizer_mix"]),
        "full_loss_weight": full_loss_weight,
        "learning_rate": rate,
        "total": float(total.detach()),
        "gradient_norm": float(gradient_norm.detach()),
        "bootstrap_total": float(bootstrap_total.detach()),
        "quantizer_regularization": float(quantizer_regularization.detach()),
        "quantized_latent_match": float(latent_match.detach()),
        "full_loss_total": float(full_total.detach()),
        **{
            f"bootstrap_{name}": float(value.detach())
            for name, value in bootstrap_terms.items()
        },
        **{
            f"full_{name}": float(value.detach())
            for name, value in full_terms.items()
        },
    }


def evaluate_validation(
    model: codec_model.NeuralImpactCodec,
    validation: list[WaveformItem],
    quantizers: int,
    device: torch.device,
    quantizer_mix: float,
) -> dict[str, Any]:
    totals: dict[str, list[float]] = {}
    unique_by_quantizer: list[set[int]] = [set() for _ in range(quantizers)]
    output_rms_values = []
    target_rms_values = []
    rms_ratios = []
    absolute_correlations = []
    latent_rms_values = []
    latent_absolute_values = []
    normalized_outputs = []
    normalized_targets = []
    output_hashes = set()
    model.eval()
    with torch.inference_mode():
        for item in validation:
            segment, mask = _aligned_segment(item.samples)
            target = _tensor(segment, device)
            mask_tensor = _tensor(mask, device)
            (
                output,
                codes,
                codebook_loss,
                commitment_loss,
                latent,
                _,
            ) = _forward_curriculum(
                model, target, quantizers, quantizer_mix
            )
            total, terms = training.frozen_reconstruction_loss(
                output * mask_tensor,
                target * mask_tensor,
                codebook_loss,
                commitment_loss,
            )
            values = {
                "total": float(total),
                **{name: float(value) for name, value in terms.items()},
            }
            for name, value in values.items():
                totals.setdefault(name, []).append(value)
            for index in range(quantizers):
                unique_by_quantizer[index].update(
                    int(value) for value in torch.unique(codes[:, index]).cpu()
                )
            valid = mask.astype(bool)
            target_values = np.asarray(segment, dtype=np.float64)
            output_values = output[0, 0].detach().cpu().numpy().astype(np.float64)
            target_valid = target_values[valid]
            output_valid = output_values[valid]
            target_rms = float(np.sqrt(np.mean(np.square(target_valid))))
            output_rms = float(np.sqrt(np.mean(np.square(output_valid))))
            if target_rms <= 0.0:
                raise common.V5Error("V5 validation target RMS is zero")
            target_centered = target_valid - np.mean(target_valid)
            output_centered = output_valid - np.mean(output_valid)
            correlation_denominator = float(
                np.linalg.norm(target_centered) * np.linalg.norm(output_centered)
            )
            correlation = (
                float(np.dot(target_centered, output_centered))
                / correlation_denominator
                if correlation_denominator > 1.0e-12
                else 0.0
            )
            normalized_output = np.zeros_like(output_values)
            normalized_target = np.zeros_like(target_values)
            normalized_output[valid] = output_valid / max(output_rms, 1.0e-12)
            normalized_target[valid] = target_valid / target_rms
            output_rms_values.append(output_rms)
            target_rms_values.append(target_rms)
            rms_ratios.append(output_rms / target_rms)
            absolute_correlations.append(abs(correlation))
            latent_rms_values.append(float(torch.sqrt(torch.mean(latent.square()))))
            latent_absolute_values.append(float(torch.max(torch.abs(latent))))
            normalized_outputs.append(normalized_output)
            normalized_targets.append(normalized_target)
            output_hashes.add(
                common.sha256_bytes(
                    np.asarray(output_values * mask, dtype="<f4").tobytes()
                )
            )
    output_pairwise = []
    target_pairwise = []
    for first in range(len(normalized_outputs)):
        for second in range(first + 1, len(normalized_outputs)):
            output_pairwise.append(
                float(
                    np.mean(
                        np.abs(normalized_outputs[first] - normalized_outputs[second])
                    )
                )
            )
            target_pairwise.append(
                float(
                    np.mean(
                        np.abs(normalized_targets[first] - normalized_targets[second])
                    )
                )
            )
    mean_target_pairwise = float(np.mean(target_pairwise))
    if mean_target_pairwise <= 0.0:
        raise common.V5Error("V5 validation target diversity is zero")
    return {
        "clip_count": len(validation),
        "mean": {name: float(np.mean(values)) for name, values in totals.items()},
        "maximum": {name: float(np.max(values)) for name, values in totals.items()},
        "unique_codes_per_quantizer": [len(values) for values in unique_by_quantizer],
        "waveform_diagnostics": {
            "mean_output_rms": float(np.mean(output_rms_values)),
            "mean_target_rms": float(np.mean(target_rms_values)),
            "mean_output_target_rms_ratio": float(np.mean(rms_ratios)),
            "minimum_output_target_rms_ratio": float(np.min(rms_ratios)),
            "mean_absolute_correlation": float(np.mean(absolute_correlations)),
            "unique_output_sha256_count": len(output_hashes),
            "mean_pairwise_normalized_output_l1": float(np.mean(output_pairwise)),
            "mean_pairwise_normalized_target_l1": mean_target_pairwise,
            "output_diversity_ratio": float(np.mean(output_pairwise))
            / mean_target_pairwise,
        },
        "latent_diagnostics": {
            "mean_rms": float(np.mean(latent_rms_values)),
            "maximum_rms": float(np.max(latent_rms_values)),
            "maximum_absolute": float(np.max(latent_absolute_values)),
        },
    }


def _assess_signal_gate(
    initial: dict[str, Any],
    current: dict[str, Any],
    quantizers: int,
    step: int,
    policy: dict[str, Any],
    require_codes: bool,
) -> dict[str, Any]:
    evaluation_step = policy["evaluation_step"]
    if step < evaluation_step:
        return {
            "status": "PendingWarmup",
            "step": step,
            "evaluation_step": evaluation_step,
            "passed": None,
            "failed_checks": [],
        }
    initial_spectrum = float(initial["mean"]["log_spectrum"])
    current_spectrum = float(current["mean"]["log_spectrum"])
    if initial_spectrum <= 0.0:
        raise common.V5Error("V5 initial validation spectrum loss is invalid")
    spectrum_improvement = (initial_spectrum - current_spectrum) / initial_spectrum
    unique_codes = current["unique_codes_per_quantizer"]
    if len(unique_codes) != quantizers:
        raise common.V5Error("V5 validation quantizer count changed")
    observed = {
        "mean_output_target_rms_ratio": current["waveform_diagnostics"][
            "mean_output_target_rms_ratio"
        ],
        "log_spectrum_relative_improvement": spectrum_improvement,
        "maximum_encoder_latent_rms": current["latent_diagnostics"][
            "maximum_rms"
        ],
        "maximum_encoder_latent_absolute": current["latent_diagnostics"][
            "maximum_absolute"
        ],
        "output_diversity_ratio": current["waveform_diagnostics"][
            "output_diversity_ratio"
        ],
    }
    if require_codes:
        observed["minimum_unique_codes_per_quantizer"] = min(unique_codes)
    checks = {
        "mean_output_target_rms_ratio": observed[
            "mean_output_target_rms_ratio"
        ]
        >= policy["minimum_mean_output_target_rms_ratio"],
        "log_spectrum_relative_improvement": observed[
            "log_spectrum_relative_improvement"
        ]
        >= policy["minimum_log_spectrum_relative_improvement"],
        "maximum_encoder_latent_rms": observed["maximum_encoder_latent_rms"]
        <= policy["maximum_encoder_latent_rms"],
        "maximum_encoder_latent_absolute": observed[
            "maximum_encoder_latent_absolute"
        ]
        <= policy["maximum_encoder_latent_absolute"],
        "output_diversity_ratio": observed["output_diversity_ratio"]
        >= policy["minimum_output_diversity_ratio"],
    }
    if require_codes:
        checks["minimum_unique_codes_per_quantizer"] = observed[
            "minimum_unique_codes_per_quantizer"
        ] >= policy["minimum_unique_codes_per_quantizer"]
    failed = sorted(name for name, passed in checks.items() if not passed)
    return {
        "status": "Passed" if not failed else "Rejected",
        "step": step,
        "evaluation_step": evaluation_step,
        "passed": not failed,
        "policy": policy,
        "observed": observed,
        "checks": checks,
        "failed_checks": failed,
    }


def assess_anti_collapse_gate(
    initial: dict[str, Any],
    current: dict[str, Any],
    quantizers: int,
    step: int,
) -> dict[str, Any]:
    return _assess_signal_gate(
        initial,
        current,
        quantizers,
        step,
        ANTI_COLLAPSE_GATE,
        require_codes=True,
    )


def assess_curriculum_gate(
    initial: dict[str, Any],
    current: dict[str, Any],
    quantizers: int,
    step: int,
) -> dict[str, Any]:
    bootstrap_step = CURRICULUM["continuous_bootstrap_end_step"]
    quantized_step = CURRICULUM["quantizer_ramp_end_step"]
    if step < bootstrap_step:
        return {
            "status": "PendingContinuousBootstrap",
            "step": step,
            "evaluation_step": bootstrap_step,
            "passed": None,
            "admissible_for_checkpoint_selection": False,
            "failed_checks": [],
        }
    if step < quantized_step:
        if step == bootstrap_step:
            result = _assess_signal_gate(
                initial,
                current,
                quantizers,
                step,
                BOOTSTRAP_GATE,
                require_codes=False,
            )
            result["stage"] = "continuous_bootstrap"
            result["admissible_for_checkpoint_selection"] = False
            return result
        return {
            "status": "PendingQuantizerRamp",
            "step": step,
            "evaluation_step": quantized_step,
            "passed": None,
            "admissible_for_checkpoint_selection": False,
            "failed_checks": [],
        }
    result = assess_anti_collapse_gate(initial, current, quantizers, step)
    result["stage"] = "fully_quantized"
    result["admissible_for_checkpoint_selection"] = result["passed"]
    return result


def initialize_codebooks_from_items(
    model: codec_model.NeuralImpactCodec,
    items: list[WaveformItem],
    capacity_id: str,
    quantizers: int,
    device: torch.device,
) -> dict[str, Any]:
    selected: list[tuple[WaveformItem, np.ndarray]] = []
    selected_ids = set()
    step = 1
    while len(selected) < CURRICULUM["codebook_initialization_segments"]:
        item, segment, _, _ = segment_for_step(items, capacity_id, step)
        if item.id not in selected_ids:
            selected.append((item, segment))
            selected_ids.add(item.id)
        step += 1
        if step > 10_000:
            raise common.V5Error("V5 cannot select distinct codebook segments")
    model.eval()
    with torch.inference_mode():
        residuals = [
            model.encoder(_tensor(segment, device)) for _, segment in selected
        ]
        for layer in model.quantizer.layers[:quantizers]:
            projected = [layer.input_projection(residual) for residual in residuals]
            candidates = torch.cat(
                [
                    value.transpose(1, 2).reshape(-1, value.shape[1])
                    for value in projected
                ],
                dim=0,
            )
            entries = layer.codebook.num_embeddings
            if candidates.shape[0] >= entries:
                indices = torch.div(
                    torch.arange(entries, device=device) * candidates.shape[0],
                    entries,
                    rounding_mode="floor",
                )
                values = candidates[indices]
            else:
                repeats = math.ceil(entries / candidates.shape[0])
                values = candidates.repeat(repeats, 1)[:entries]
            layer.codebook.weight.copy_(values)
            next_residuals = []
            for residual in residuals:
                quantized, _, _, _ = layer(residual)
                next_residuals.append(residual - quantized)
            residuals = next_residuals
        unique_by_quantizer = [set() for _ in range(quantizers)]
        for item, segment in selected:
            latent = model.encoder(_tensor(segment, device))
            _, codes, _, _ = model.quantizer(latent, quantizers)
            for index in range(quantizers):
                unique_by_quantizer[index].update(
                    int(value) for value in torch.unique(codes[:, index]).cpu()
                )
    return {
        "revision": "eight-distinct-trained-latent-sequential-projection-v1",
        "item_ids": [item.id for item, _ in selected],
        "unique_codes_per_quantizer": [
            len(values) for values in unique_by_quantizer
        ],
        "model_state_sha256": codec_model.state_sha256(model),
    }


def _optimizer(model: torch.nn.Module) -> torch.optim.Optimizer:
    return torch.optim.AdamW(
        model.parameters(),
        lr=common.TRAINING_CONFIG["learning_rate"],
        betas=tuple(common.TRAINING_CONFIG["betas"]),
        weight_decay=common.TRAINING_CONFIG["weight_decay"],
    )


def _write_json_atomic(path: Path, value: dict[str, Any]) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_bytes(common.canonical_json(value))
    os.replace(temporary, path)


def _save_checkpoint_atomic(path: Path, value: dict[str, Any]) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    torch.save(value, temporary)
    os.replace(temporary, path)


def _append_metric(path: Path, value: dict[str, Any]) -> None:
    line = json.dumps(common.json_ready(value), sort_keys=True, allow_nan=False) + "\n"
    with path.open("a", encoding="utf-8") as handle:
        handle.write(line)
        handle.flush()
        os.fsync(handle.fileno())


def _waveform_descriptors(items: list[WaveformItem]) -> list[dict[str, Any]]:
    return [
        {
            "id": item.id,
            "role": item.role,
            "source_kind": item.source_kind,
            "source_group": item.source_group,
            "samples": int(item.samples.size),
            "samples_sha256": item.samples_sha256,
            "output_gain": item.output_gain,
        }
        for item in items
    ]


def _implementation_sha256() -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    return {
        name: common.sha256_file(directory / filename)
        for name, filename in IMPLEMENTATION_FILES.items()
    }


def _load_resume(
    output: Path,
    run_manifest_sha256: str,
    capacity_id: str,
    device: torch.device,
) -> tuple[
    codec_model.NeuralImpactCodec,
    torch.optim.Optimizer,
    int,
    float | None,
    dict[str, Any] | None,
    dict[str, Any],
    dict[str, Any] | None,
]:
    _, state = common.load_json(output / "state.json", "V5 capacity state", True)
    if (
        state.get("schema") != STATE_SCHEMA
        or state.get("run_manifest_sha256") != run_manifest_sha256
        or state.get("capacity_id") != capacity_id
        or state.get("status") != "InProgress"
    ):
        raise common.V5Error("V5 capacity resume state changed")
    checkpoint_path = output / state["latest_checkpoint"]
    loaded = torch.load(checkpoint_path, map_location=device, weights_only=True)
    model = codec_model.NeuralImpactCodec(codec_model.CodecConfig.frozen()).to(device)
    optimizer = _optimizer(model)
    model.load_state_dict(loaded["model"], strict=True)
    optimizer.load_state_dict(loaded["optimizer"])
    initial_validation = state.get("initial_validation")
    if not isinstance(initial_validation, dict):
        raise common.V5Error("V5 capacity resume lacks initial validation")
    codebook_transition = state.get("codebook_transition")
    if int(loaded["step"]) >= CURRICULUM["continuous_bootstrap_end_step"] and not isinstance(
        codebook_transition, dict
    ):
        raise common.V5Error("V5 capacity resume lacks codebook transition")
    return (
        model,
        optimizer,
        int(loaded["step"]),
        state.get("best_validation_loss"),
        state.get("best_checkpoint"),
        initial_validation,
        codebook_transition,
    )


def run(root: Path, arguments: argparse.Namespace) -> Path:
    output = arguments.output.resolve()
    if output.is_relative_to(root):
        raise common.V5Error("V5 capacity output must stay outside the repository")
    frozen_maximum_steps = common.TRAINING_CONFIG["maximum_steps"]
    checkpoint_interval = common.TRAINING_CONFIG["checkpoint_interval_steps"]
    execution_steps = arguments.stop_after_step or frozen_maximum_steps
    if (
        execution_steps < ANTI_COLLAPSE_GATE["evaluation_step"]
        or execution_steps > frozen_maximum_steps
        or execution_steps % checkpoint_interval != 0
    ):
        raise common.V5Error(
            "V5 bounded stop must be a checkpoint from anti-collapse evaluation "
            "through the frozen maximum"
        )
    v5_manifest = load_parent_manifest(
        root,
        arguments.v5_manifest,
        "V5 preflight manifest",
        training_preflight.V5_PREFLIGHT_MANIFEST_SHA256,
    )
    training_manifest = load_parent_manifest(
        root,
        arguments.training_preflight_manifest,
        "V5 training preflight manifest",
        TRAINING_PREFLIGHT_MANIFEST_SHA256,
    )
    v4_manifest = load_parent_manifest(
        root, arguments.v4_manifest, "V4 manifest", V4_MANIFEST_SHA256
    )
    if (
        training_manifest.get("frozen_capacity_training_authorized") is not True
        or training_manifest.get("development_access_authorized") is not False
        or v4_manifest.get("new_object_or_sealed_waveform_access_authorized")
        is not False
    ):
        raise common.V5Error("V5 capacity training authorization changed")
    capacity = next(
        item for item in common.CAPACITIES if item["id"] == arguments.capacity_id
    )
    archive = common.require_external_file(root, arguments.archive, "Heller archive")
    if common.sha256_file(archive) != common.ARCHIVE_SHA256:
        raise common.V5Error("V5 capacity archive changed")
    fit_directories = parse_fit_extractions(arguments.fit_extraction)
    environment = training_preflight.training_environment()
    training.configure_training_determinism(common.TRAINING_CONFIG["random_seed"])

    internet_train, validation = load_internet_waveforms(archive, v5_manifest["corpus"])
    fit_train, fit_lineage = load_fit_waveforms(root, v4_manifest, fit_directories)
    train_items = internet_train + fit_train
    if len(train_items) != TOTAL_TRAIN_ITEM_COUNT:
        raise common.V5Error("V5 capacity train item cardinality changed")

    run_manifest = {
        "schema": RUN_SCHEMA,
        "status": "Frozen",
        "study_id": common.STUDY_ID,
        "revision": REVISION,
        "capacity": capacity,
        "v5_preflight_manifest_sha256": training_preflight.V5_PREFLIGHT_MANIFEST_SHA256,
        "training_preflight_manifest_sha256": TRAINING_PREFLIGHT_MANIFEST_SHA256,
        "v4_manifest_sha256": V4_MANIFEST_SHA256,
        "archive_sha256": common.ARCHIVE_SHA256,
        "implementation_sha256": _implementation_sha256(),
        "environment": environment,
        "model": common.MODEL_CONFIG,
        "loss": common.LOSS_CONFIG,
        "loss_implementation_revision": training.LOSS_IMPLEMENTATION_REVISION,
        "training": common.TRAINING_CONFIG,
        "execution": {
            "frozen_maximum_steps": frozen_maximum_steps,
            "requested_stop_after_step": arguments.stop_after_step,
            "curriculum": CURRICULUM,
            "bootstrap_gate": BOOTSTRAP_GATE,
            "anti_collapse_gate": ANTI_COLLAPSE_GATE,
        },
        "codebook_initialization_revision": training.CODEBOOK_INITIALIZATION_REVISION,
        "sampling": "stateless_sha256_seeded_item_uniform_v1",
        "train_items": _waveform_descriptors(train_items),
        "validation_items": _waveform_descriptors(validation),
        "fit_lineage": fit_lineage,
        "development_waveform_samples_decoded": 0,
        "sealed_waveform_samples_decoded": 0,
        "development_access_authorized": False,
        "holdout_access_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }
    run_manifest_bytes = common.canonical_json(run_manifest)
    run_manifest_sha256 = common.sha256_bytes(run_manifest_bytes)
    device = torch.device("cuda:0")
    metrics_path = output / "metrics.jsonl"

    if output.exists():
        if not arguments.resume or not output.is_dir():
            raise common.V5Error("V5 capacity output exists without --resume")
        existing = (output / "run-manifest.json").read_bytes()
        if existing != run_manifest_bytes:
            raise common.V5Error("V5 capacity run manifest changed on resume")
        (
            model,
            optimizer,
            completed_step,
            best_loss,
            best_checkpoint,
            initial_validation,
            codebook_transition,
        ) = _load_resume(output, run_manifest_sha256, arguments.capacity_id, device)
    else:
        if arguments.resume:
            raise common.V5Error("V5 capacity --resume output does not exist")
        output.parent.mkdir(parents=True, exist_ok=True)
        output.mkdir()
        (output / "run-manifest.json").write_bytes(run_manifest_bytes)
        model = codec_model.NeuralImpactCodec(codec_model.CodecConfig.frozen()).to(
            device
        )
        first_item, first_segment, _, _ = segment_for_step(
            train_items, arguments.capacity_id, 1
        )
        initialized_state = training.initialize_codebooks_from_batch(
            model,
            _tensor(first_segment, device),
            capacity["quantizers"],
        )
        optimizer = _optimizer(model)
        completed_step = 0
        best_loss = None
        best_checkpoint = None
        codebook_transition = None
        initial_validation = evaluate_validation(
            model, validation, capacity["quantizers"], device, quantizer_mix=0.0
        )
        state = {
            "schema": STATE_SCHEMA,
            "status": "InProgress",
            "capacity_id": arguments.capacity_id,
            "run_manifest_sha256": run_manifest_sha256,
            "completed_step": 0,
            "initialization_item": first_item.id,
            "initialized_state_sha256": initialized_state,
            "latest_checkpoint": None,
            "best_validation_loss": None,
            "best_checkpoint": None,
            "initial_validation": initial_validation,
            "codebook_transition": None,
            "anti_collapse_gate": {
                "status": "PendingContinuousBootstrap",
                "evaluation_step": BOOTSTRAP_GATE["evaluation_step"],
            },
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
        }
        _write_json_atomic(output / "state.json", state)

    last_metrics: dict[str, Any] | None = None
    last_gate: dict[str, Any] | None = None
    for step in range(completed_step + 1, execution_steps + 1):
        item, segment, mask, segment_policy = segment_for_step(
            train_items, arguments.capacity_id, step
        )
        last_metrics = _train_step(
            model,
            optimizer,
            _tensor(segment, device),
            _tensor(mask, device),
            capacity["quantizers"],
            step,
        )
        if step % METRIC_INTERVAL_STEPS == 0:
            print(
                f"{arguments.capacity_id} step {step}/{execution_steps} "
                f"loss={last_metrics['total']:.6f} item={item.id} "
                f"segment={segment_policy}",
                flush=True,
            )
        if step % checkpoint_interval != 0 and step != execution_steps:
            continue
        curriculum = curriculum_for_step(step)
        validation_metrics = evaluate_validation(
            model,
            validation,
            capacity["quantizers"],
            device,
            quantizer_mix=float(curriculum["quantizer_mix"]),
        )
        last_gate = assess_curriculum_gate(
            initial_validation,
            validation_metrics,
            capacity["quantizers"],
            step,
        )
        if (
            step == CURRICULUM["continuous_bootstrap_end_step"]
            and last_gate["passed"] is True
        ):
            codebook_transition = initialize_codebooks_from_items(
                model,
                train_items,
                arguments.capacity_id,
                capacity["quantizers"],
                device,
            )
        checkpoint_name = f"checkpoint-step-{step:08d}.pt"
        checkpoint_path = output / checkpoint_name
        checkpoint = {
            "schema": training.CHECKPOINT_SCHEMA,
            "capacity_id": arguments.capacity_id,
            "quantizers": capacity["quantizers"],
            "step": step,
            "run_manifest_sha256": run_manifest_sha256,
            "curriculum": curriculum,
            "codebook_transition": codebook_transition,
            "model": model.state_dict(),
            "optimizer": optimizer.state_dict(),
        }
        _save_checkpoint_atomic(checkpoint_path, checkpoint)
        checkpoint_record = {
            "path": checkpoint_name,
            "sha256": common.sha256_file(checkpoint_path),
            "bytes": checkpoint_path.stat().st_size,
            "step": step,
        }
        validation_loss = validation_metrics["mean"]["total"]
        if last_gate["admissible_for_checkpoint_selection"] is True and (
            best_loss is None or validation_loss < best_loss
        ):
            best_loss = validation_loss
            best_checkpoint = checkpoint_record
        record = {
            "step": step,
            "train": last_metrics,
            "validation": validation_metrics,
            "anti_collapse_gate": last_gate,
            "curriculum": curriculum,
            "codebook_transition": codebook_transition,
            "checkpoint": checkpoint_record,
            "best_validation_loss": best_loss,
            "best_checkpoint": best_checkpoint,
            "last_item": item.id,
            "last_segment_policy": segment_policy,
        }
        _append_metric(metrics_path, record)
        state = {
            "schema": STATE_SCHEMA,
            "status": "Rejected" if last_gate["passed"] is False else "InProgress",
            "capacity_id": arguments.capacity_id,
            "run_manifest_sha256": run_manifest_sha256,
            "completed_step": step,
            "latest_checkpoint": checkpoint_name,
            "best_validation_loss": best_loss,
            "best_checkpoint": best_checkpoint,
            "initial_validation": initial_validation,
            "codebook_transition": codebook_transition,
            "anti_collapse_gate": last_gate,
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
        }
        _write_json_atomic(output / "state.json", state)
        if last_gate["passed"] is False:
            report = {
                "schema": REPORT_SCHEMA,
                "status": "Rejected",
                "decision": "REJECT_TRAINING_SUBSTRATE",
                "capacity_id": arguments.capacity_id,
                "quantizers": capacity["quantizers"],
                "run_manifest_sha256": run_manifest_sha256,
                "metrics_sha256": common.sha256_file(metrics_path),
                "completed_steps": step,
                "rejected_checkpoint": checkpoint_record,
                "anti_collapse_gate": last_gate,
                "curriculum": CURRICULUM,
                "codebook_transition": codebook_transition,
                "capacity_training_complete": False,
                "development_waveform_samples_decoded": 0,
                "sealed_waveform_samples_decoded": 0,
                "development_access_authorized": False,
                "holdout_access_authorized": False,
                "runtime_neural_inference_authorized": False,
                "authored_clip_fallback_required": True,
            }
            report_bytes = common.canonical_json(report)
            (output / "report.json").write_bytes(report_bytes)
            state["report_sha256"] = common.sha256_bytes(report_bytes)
            _write_json_atomic(output / "state.json", state)
            return output

    if best_checkpoint is None or last_metrics is None or last_gate is None:
        raise common.V5Error("V5 capacity training produced no checkpoint")
    metrics_sha256 = common.sha256_file(metrics_path)
    full_training_complete = execution_steps == frozen_maximum_steps
    terminal_status = "Complete" if full_training_complete else "ResearchComplete"
    terminal_decision = (
        "CAPACITY_TRAINING_COMPLETE"
        if full_training_complete
        else "RESEARCH_DISCRIMINATOR_PASSED"
    )
    report = {
        "schema": REPORT_SCHEMA,
        "status": terminal_status,
        "decision": terminal_decision,
        "capacity_id": arguments.capacity_id,
        "quantizers": capacity["quantizers"],
        "run_manifest_sha256": run_manifest_sha256,
        "metrics_sha256": metrics_sha256,
        "completed_steps": execution_steps,
        "best_validation_loss": best_loss,
        "best_checkpoint": best_checkpoint,
        "anti_collapse_gate": last_gate,
        "curriculum": CURRICULUM,
        "codebook_transition": codebook_transition,
        "capacity_training_complete": full_training_complete,
        "final_model_state_sha256": codec_model.state_sha256(model),
        "internet_train_clip_count": len(internet_train),
        "fit_contact_count": len(fit_train),
        "internet_validation_clip_count": len(validation),
        "development_waveform_samples_decoded": 0,
        "sealed_waveform_samples_decoded": 0,
        "development_access_authorized": False,
        "holdout_access_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }
    report_bytes = common.canonical_json(report)
    (output / "report.json").write_bytes(report_bytes)
    state = {
        "schema": STATE_SCHEMA,
        "status": terminal_status,
        "capacity_id": arguments.capacity_id,
        "run_manifest_sha256": run_manifest_sha256,
        "completed_step": execution_steps,
        "latest_checkpoint": f"checkpoint-step-{execution_steps:08d}.pt",
        "best_validation_loss": best_loss,
        "best_checkpoint": best_checkpoint,
        "initial_validation": initial_validation,
        "codebook_transition": codebook_transition,
        "anti_collapse_gate": last_gate,
        "report_sha256": common.sha256_bytes(report_bytes),
        "development_waveform_samples_decoded": 0,
        "sealed_waveform_samples_decoded": 0,
    }
    _write_json_atomic(output / "state.json", state)
    return output


def main() -> None:
    arguments = parse_arguments()
    output = run(common.repository_root(), arguments)
    _, state = common.load_json(output / "state.json", "V5 capacity state", True)
    print(f"R3A V5 capacity run: {output}")
    print(f"status: {state['status']}")
    if state["status"] in {"Complete", "ResearchComplete", "Rejected"}:
        print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
