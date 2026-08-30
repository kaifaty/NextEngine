#!/usr/bin/env python3
"""Frozen CUDA runner primitives for R3A V5 neural representation controls."""

from __future__ import annotations

import io
import math
import os
import random
import wave
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_contact_field_r3a_v5_common as common
import physical_sound_contact_field_r3a_v5_model as codec_model
import scipy.signal
import torch
from torch import nn
from torch.nn import functional

CONTROL_CAPACITY_ID = "rvq-6kbps"
CONTROL_QUANTIZERS = 4
CONTROL_OPTIMIZATION_STEPS = 3
CONTROL_MINIMUM_L1_IMPROVEMENT = 0.01
CONTROL_MINIMUM_TRAIN_UNIQUE_CODES_PER_QUANTIZER = 2
CONTROL_MINIMUM_VALIDATION_ACTIVE_QUANTIZERS = 2
LOSS_IMPLEMENTATION_REVISION = "v5-full-loss-center-false-hann-v1"
CODEBOOK_INITIALIZATION_REVISION = "sequential_projected_first-train-latent-v2"
CHECKPOINT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5.checkpoint.v1"


@dataclass(frozen=True)
class DecodedClip:
    member: str
    role: str
    event_group: str
    samples: np.ndarray
    source_scalar_samples_decoded: int
    output_gain: float


def configure_training_determinism(seed: int) -> None:
    if os.environ.get("CUBLAS_WORKSPACE_CONFIG") != ":4096:8":
        raise common.V5Error("CUBLAS_WORKSPACE_CONFIG must be :4096:8")
    random.seed(seed)
    np.random.seed(seed)
    torch.manual_seed(seed)
    torch.cuda.manual_seed_all(seed)
    torch.use_deterministic_algorithms(True)
    torch.backends.cudnn.benchmark = False
    torch.backends.cudnn.deterministic = True
    torch.backends.cuda.matmul.allow_tf32 = False
    torch.backends.cudnn.allow_tf32 = False


def select_control_clips(corpus: dict[str, Any]) -> list[dict[str, Any]]:
    selected = []
    for role in ("train", "internal_validation"):
        matches = sorted(
            (clip for clip in corpus["clips"] if clip["role"] == role),
            key=lambda clip: (clip["event_group"], clip["member"]),
        )
        if not matches:
            raise common.V5Error(f"V5 corpus lacks a {role} control clip")
        selected.append(matches[0])
    if selected[0]["event_group"] == selected[1]["event_group"]:
        raise common.V5Error("V5 control roles share an event group")
    return selected


def decode_control_clip(
    archive: zipfile.ZipFile, descriptor: dict[str, Any]
) -> DecodedClip:
    payload = archive.read(descriptor["member"])
    if common.sha256_bytes(payload) != descriptor["member_sha256"]:
        raise common.V5Error("V5 control clip payload changed")
    with wave.open(io.BytesIO(payload), "rb") as source:
        if (
            source.getnchannels() != common.SOURCE_CHANNELS
            or source.getsampwidth() != common.SOURCE_SAMPLE_WIDTH_BYTES
            or source.getframerate() != common.SOURCE_SAMPLE_RATE_HZ
            or source.getcomptype() != "NONE"
        ):
            raise common.V5Error("V5 control clip wave identity changed")
        frames = source.getnframes()
        raw = source.readframes(frames)
    scalars = np.frombuffer(raw, dtype="<i2")
    if scalars.size != frames * common.SOURCE_CHANNELS:
        raise common.V5Error("V5 control clip sample count changed")
    stereo = scalars.astype(np.float64).reshape(-1, common.SOURCE_CHANNELS)
    mono = stereo.mean(axis=1) / 32768.0
    resampled = scipy.signal.resample_poly(mono, 160, 147)
    if not np.isfinite(resampled).all() or not np.any(resampled):
        raise common.V5Error("V5 control clip decode is silent or non-finite")
    peak = float(np.max(np.abs(resampled)))
    output_gain = 0.92 / peak
    normalized = resampled * output_gain
    peak_index = int(np.argmax(np.abs(normalized)))
    prefix = max(0, 512 - peak_index)
    start = max(0, peak_index - 512)
    segment = np.pad(normalized[start:], (prefix, 0))
    if segment.size < common.TRAINING_SEGMENT_SAMPLES:
        segment = np.pad(segment, (0, common.TRAINING_SEGMENT_SAMPLES - segment.size))
    segment = np.asarray(segment[: common.TRAINING_SEGMENT_SAMPLES], dtype="<f4")
    if int(np.argmax(np.abs(segment))) != 512:
        raise common.V5Error("V5 control clip alignment changed")
    return DecodedClip(
        member=descriptor["member"],
        role=descriptor["role"],
        event_group=descriptor["event_group"],
        samples=segment,
        source_scalar_samples_decoded=int(scalars.size),
        output_gain=output_gain,
    )


def decode_control_pair(
    archive_path: Path, corpus: dict[str, Any]
) -> list[DecodedClip]:
    descriptors = select_control_clips(corpus)
    with zipfile.ZipFile(archive_path) as archive:
        return [decode_control_clip(archive, item) for item in descriptors]


def _stft(value: torch.Tensor, window_samples: int, hop: int) -> torch.Tensor:
    window = torch.hann_window(
        window_samples,
        periodic=True,
        device=value.device,
        dtype=value.dtype,
    )
    return torch.stft(
        value[:, 0],
        n_fft=window_samples,
        hop_length=hop,
        win_length=window_samples,
        window=window,
        center=False,
        normalized=True,
        return_complex=True,
    )


def _band_mask(n_fft: int, device: torch.device) -> torch.Tensor:
    frequencies = torch.fft.rfftfreq(
        n_fft,
        d=1.0 / common.TARGET_SAMPLE_RATE_HZ,
        device=device,
    )
    config = common.LOSS_CONFIG["log_spectrum"]
    return (frequencies >= config["minimum_hz"]) & (frequencies <= config["maximum_hz"])


def _log_amplitude(value: torch.Tensor) -> torch.Tensor:
    floor_db = common.LOSS_CONFIG["log_spectrum"]["floor_db"]
    floor = 10.0 ** (floor_db / 20.0)
    return 20.0 * torch.log10(value.abs().clamp_min(floor))


def _hz_to_mel(value: torch.Tensor) -> torch.Tensor:
    return 2595.0 * torch.log10(1.0 + value / 700.0)


def _mel_to_hz(value: torch.Tensor) -> torch.Tensor:
    return 700.0 * (torch.pow(10.0, value / 2595.0) - 1.0)


def _mel_filter_bank(
    n_fft: int,
    mel_bins: int,
    device: torch.device,
    dtype: torch.dtype,
) -> torch.Tensor:
    minimum = torch.tensor(0.0, device=device, dtype=dtype)
    maximum = torch.tensor(
        common.TARGET_SAMPLE_RATE_HZ / 2.0,
        device=device,
        dtype=dtype,
    )
    mel_points = torch.linspace(
        _hz_to_mel(minimum),
        _hz_to_mel(maximum),
        mel_bins + 2,
        device=device,
        dtype=dtype,
    )
    hz_points = _mel_to_hz(mel_points)
    frequencies = torch.fft.rfftfreq(
        n_fft,
        d=1.0 / common.TARGET_SAMPLE_RATE_HZ,
        device=device,
        dtype=dtype,
    )
    lower = hz_points[:-2, None]
    center = hz_points[1:-1, None]
    upper = hz_points[2:, None]
    rising = (frequencies[None, :] - lower) / (center - lower).clamp_min(1.0e-12)
    falling = (upper - frequencies[None, :]) / (upper - center).clamp_min(1.0e-12)
    return torch.minimum(rising, falling).clamp(0.0, 1.0)


def _si_sdr_loss(output: torch.Tensor, target: torch.Tensor) -> torch.Tensor:
    epsilon = common.LOSS_CONFIG["si_sdr"]["epsilon"]
    output_flat = output.flatten(1)
    target_flat = target.flatten(1)
    output_flat = output_flat - output_flat.mean(dim=1, keepdim=True)
    target_flat = target_flat - target_flat.mean(dim=1, keepdim=True)
    scale = (output_flat * target_flat).sum(dim=1, keepdim=True) / (
        target_flat.square().sum(dim=1, keepdim=True) + epsilon
    )
    projected = scale * target_flat
    noise = output_flat - projected
    ratio = (projected.square().sum(dim=1) + epsilon) / (
        noise.square().sum(dim=1) + epsilon
    )
    return -10.0 * torch.log10(ratio).mean()


def frozen_reconstruction_loss(
    output: torch.Tensor,
    target: torch.Tensor,
    codebook_loss: torch.Tensor,
    commitment_loss: torch.Tensor,
) -> tuple[torch.Tensor, dict[str, torch.Tensor]]:
    if (
        output.shape != target.shape
        or output.shape[-1] != common.TRAINING_SEGMENT_SAMPLES
    ):
        raise common.V5Error("V5 loss input shape changed")
    terms: dict[str, torch.Tensor] = {}
    terms["waveform_l1"] = functional.l1_loss(output, target)
    terms["si_sdr"] = _si_sdr_loss(output, target)

    complex_losses = []
    for window_samples in common.LOSS_CONFIG["complex_stft"]["window_samples"]:
        hop = window_samples // common.LOSS_CONFIG["complex_stft"]["hop_divisor"]
        output_stft = _stft(output, window_samples, hop)
        target_stft = _stft(target, window_samples, hop)
        complex_losses.append(
            functional.l1_loss(output_stft.real, target_stft.real)
            + functional.l1_loss(output_stft.imag, target_stft.imag)
        )
    terms["complex_stft"] = torch.stack(complex_losses).mean()

    spectrum_losses = []
    peak_losses = []
    for window_samples in common.LOSS_CONFIG["log_spectrum"]["window_samples"]:
        hop = window_samples // common.LOSS_CONFIG["log_spectrum"]["hop_divisor"]
        output_stft = _stft(output, window_samples, hop)
        target_stft = _stft(target, window_samples, hop)
        mask = _band_mask(window_samples, output.device)
        output_log = _log_amplitude(output_stft[:, mask])
        target_log = _log_amplitude(target_stft[:, mask])
        difference = (output_log - target_log).abs()
        spectrum_losses.append(difference.mean())
        target_strength = target_stft[:, mask].abs().mean(dim=-1)
        peak_count = min(
            common.LOSS_CONFIG["target_peak_emphasis"]["top_bins_per_scale"],
            target_strength.shape[1],
        )
        peak_indices = torch.topk(target_strength, peak_count, dim=1).indices
        expanded = peak_indices[:, :, None].expand(-1, -1, difference.shape[-1])
        peak_losses.append(torch.gather(difference, 1, expanded).mean())
    terms["log_spectrum"] = torch.stack(spectrum_losses).mean()
    terms["target_peak_emphasis"] = torch.stack(peak_losses).mean()

    mel_losses = []
    mel_config = common.LOSS_CONFIG["log_mel"]
    for window_samples, mel_bins in zip(
        mel_config["window_samples"], mel_config["mel_bins"], strict=True
    ):
        hop = window_samples // mel_config["hop_divisor"]
        output_magnitude = _stft(output, window_samples, hop).abs()
        target_magnitude = _stft(target, window_samples, hop).abs()
        filters = _mel_filter_bank(
            window_samples,
            mel_bins,
            output.device,
            output.dtype,
        )
        output_mel = torch.einsum("mf,bft->bmt", filters, output_magnitude)
        target_mel = torch.einsum("mf,bft->bmt", filters, target_magnitude)
        mel_losses.append(
            functional.l1_loss(
                torch.log(output_mel.clamp_min(1.0e-5)),
                torch.log(target_mel.clamp_min(1.0e-5)),
            )
        )
    terms["log_mel"] = torch.stack(mel_losses).mean()

    smoothing = common.LOSS_CONFIG["envelope"]["smoothing_samples"]
    output_envelope = functional.avg_pool1d(
        output.abs(), smoothing, stride=smoothing // 2
    )
    target_envelope = functional.avg_pool1d(
        target.abs(), smoothing, stride=smoothing // 2
    )
    terms["envelope"] = functional.l1_loss(output_envelope, target_envelope)

    output_energy = output_envelope.square()
    target_energy = target_envelope.square()
    output_decay = torch.flip(
        torch.cumsum(torch.flip(output_energy, dims=(-1,)), dim=-1), dims=(-1,)
    )
    target_decay = torch.flip(
        torch.cumsum(torch.flip(target_energy, dims=(-1,)), dim=-1), dims=(-1,)
    )
    output_decay = output_decay / output_decay[..., :1].clamp_min(1.0e-12)
    target_decay = target_decay / target_decay[..., :1].clamp_min(1.0e-12)
    decay_floor = 10.0 ** (common.LOSS_CONFIG["decay_energy_curve"]["floor_db"] / 10.0)
    terms["decay_energy_curve"] = functional.l1_loss(
        10.0 * torch.log10(output_decay.clamp_min(decay_floor)),
        10.0 * torch.log10(target_decay.clamp_min(decay_floor)),
    )
    terms["rvq_codebook"] = codebook_loss
    terms["rvq_commitment"] = commitment_loss

    total = output.new_zeros(())
    for name, value in terms.items():
        if not torch.isfinite(value):
            raise common.V5Error(f"V5 loss term is non-finite: {name}")
        total = total + common.LOSS_CONFIG[name]["weight"] * value
    if not torch.isfinite(total):
        raise common.V5Error("V5 total loss is non-finite")
    return total, terms


def _optimizer(model: nn.Module) -> torch.optim.Optimizer:
    return torch.optim.AdamW(
        model.parameters(),
        lr=common.TRAINING_CONFIG["learning_rate"],
        betas=tuple(common.TRAINING_CONFIG["betas"]),
        weight_decay=common.TRAINING_CONFIG["weight_decay"],
    )


def initialize_codebooks_from_batch(
    model: codec_model.NeuralImpactCodec,
    target: torch.Tensor,
    quantizers: int,
) -> str:
    model.eval()
    with torch.inference_mode():
        latent = model.encoder(target)
        residual = latent
        for layer in model.quantizer.layers[:quantizers]:
            projected = layer.input_projection(residual)
            candidates = projected.transpose(1, 2).reshape(-1, projected.shape[1])
            if candidates.shape[0] < 2:
                raise common.V5Error("V5 codebook initialization lacks latent frames")
            repeats = math.ceil(
                layer.codebook.num_embeddings / candidates.shape[0]
            )
            values = candidates.repeat(repeats, 1)[: layer.codebook.num_embeddings]
            layer.codebook.weight.copy_(values)
            selected = layer.codebook(
                torch.arange(projected.shape[-1], device=projected.device)
                % layer.codebook.num_embeddings
            ).transpose(0, 1)[None, ...]
            residual = residual - layer.output_projection(selected)
    return codec_model.state_sha256(model)


def _step(
    model: codec_model.NeuralImpactCodec,
    optimizer: torch.optim.Optimizer,
    target: torch.Tensor,
) -> dict[str, float]:
    model.train()
    output, _, codebook_loss, commitment_loss = model(target, CONTROL_QUANTIZERS)
    loss, terms = frozen_reconstruction_loss(
        output, target, codebook_loss, commitment_loss
    )
    optimizer.zero_grad(set_to_none=True)
    loss.backward()
    gradient_norm = torch.nn.utils.clip_grad_norm_(
        model.parameters(), common.TRAINING_CONFIG["gradient_clip_norm"]
    )
    if not torch.isfinite(gradient_norm):
        raise common.V5Error("V5 gradient norm is non-finite")
    optimizer.step()
    return {
        "total": float(loss.detach()),
        "gradient_norm": float(gradient_norm.detach()),
        **{name: float(value.detach()) for name, value in terms.items()},
    }


def _inference(
    model: codec_model.NeuralImpactCodec, target: torch.Tensor
) -> tuple[torch.Tensor, torch.Tensor, float]:
    model.eval()
    with torch.inference_mode():
        output, codes, _, _ = model(target, CONTROL_QUANTIZERS)
        l1 = float(functional.l1_loss(output, target))
    return output, codes, l1


def _unique_codes(codes: torch.Tensor) -> list[int]:
    return [
        int(torch.unique(codes[:, index]).numel()) for index in range(codes.shape[1])
    ]


def run_training_controls(
    decoded: list[DecodedClip], checkpoint_path: Path
) -> dict[str, Any]:
    if [clip.role for clip in decoded] != ["train", "internal_validation"]:
        raise common.V5Error("V5 control clip roles changed")
    configure_training_determinism(common.TRAINING_CONFIG["random_seed"])
    device = torch.device("cuda:0")
    model = codec_model.NeuralImpactCodec(codec_model.CodecConfig.frozen()).to(device)
    train = torch.from_numpy(decoded[0].samples.copy()).reshape(1, 1, -1).to(device)
    validation = (
        torch.from_numpy(decoded[1].samples.copy()).reshape(1, 1, -1).to(device)
    )
    initialized_state = initialize_codebooks_from_batch(
        model, train, CONTROL_QUANTIZERS
    )
    optimizer = _optimizer(model)
    _, _, initial_l1 = _inference(model, train)
    steps = [_step(model, optimizer, train) for _ in range(CONTROL_OPTIMIZATION_STEPS)]
    trained_output, trained_codes, final_l1 = _inference(model, train)
    _, validation_codes, validation_l1 = _inference(model, validation)
    unique_train = _unique_codes(trained_codes)
    unique_validation = _unique_codes(validation_codes)
    validation_active_quantizers = sum(value >= 2 for value in unique_validation)
    if min(unique_train) < CONTROL_MINIMUM_TRAIN_UNIQUE_CODES_PER_QUANTIZER:
        raise common.V5Error("V5 train RVQ control collapsed to one code")
    if validation_active_quantizers < CONTROL_MINIMUM_VALIDATION_ACTIVE_QUANTIZERS:
        raise common.V5Error("V5 validation RVQ control lacks active stages")
    improvement = (initial_l1 - final_l1) / initial_l1
    if improvement < CONTROL_MINIMUM_L1_IMPROVEMENT:
        raise common.V5Error(f"V5 full-model control did not improve: {improvement}")

    checkpoint = {
        "schema": CHECKPOINT_SCHEMA,
        "step": CONTROL_OPTIMIZATION_STEPS,
        "capacity_id": CONTROL_CAPACITY_ID,
        "quantizers": CONTROL_QUANTIZERS,
        "codebook_initialization_revision": CODEBOOK_INITIALIZATION_REVISION,
        "initialized_state_sha256": initialized_state,
        "model": model.state_dict(),
        "optimizer": optimizer.state_dict(),
    }
    torch.save(checkpoint, checkpoint_path)
    checkpoint_sha256 = common.sha256_file(checkpoint_path)

    reference_step = _step(model, optimizer, train)
    reference_output, reference_codes, _ = _inference(model, train)
    reference_state = codec_model.state_sha256(model)

    configure_training_determinism(common.TRAINING_CONFIG["random_seed"])
    resumed = codec_model.NeuralImpactCodec(codec_model.CodecConfig.frozen()).to(device)
    resumed_optimizer = _optimizer(resumed)
    loaded = torch.load(checkpoint_path, map_location=device, weights_only=True)
    if (
        loaded.get("schema") != CHECKPOINT_SCHEMA
        or loaded.get("step") != CONTROL_OPTIMIZATION_STEPS
        or loaded.get("capacity_id") != CONTROL_CAPACITY_ID
        or loaded.get("quantizers") != CONTROL_QUANTIZERS
    ):
        raise common.V5Error("V5 checkpoint identity changed")
    resumed.load_state_dict(loaded["model"], strict=True)
    resumed_optimizer.load_state_dict(loaded["optimizer"])
    resumed_step = _step(resumed, resumed_optimizer, train)
    resumed_output, resumed_codes, _ = _inference(resumed, train)
    exact_resume = (
        reference_state == codec_model.state_sha256(resumed)
        and torch.equal(reference_output, resumed_output)
        and torch.equal(reference_codes, resumed_codes)
        and reference_step == resumed_step
    )
    if not exact_resume:
        raise common.V5Error("V5 checkpoint resume is not exact")

    return {
        "capacity_id": CONTROL_CAPACITY_ID,
        "quantizers": CONTROL_QUANTIZERS,
        "optimization_steps_before_checkpoint": CONTROL_OPTIMIZATION_STEPS,
        "resume_steps": 1,
        "initial_waveform_l1": initial_l1,
        "final_waveform_l1": final_l1,
        "relative_l1_improvement": improvement,
        "validation_waveform_l1": validation_l1,
        "step_metrics": steps,
        "train_unique_codes_per_quantizer": unique_train,
        "validation_unique_codes_per_quantizer": unique_validation,
        "validation_active_quantizers": validation_active_quantizers,
        "trained_output_sha256": codec_model.tensor_sha256(trained_output),
        "trained_codes_sha256": codec_model.tensor_sha256(trained_codes),
        "checkpoint_sha256": checkpoint_sha256,
        "checkpoint_bytes": checkpoint_path.stat().st_size,
        "exact_checkpoint_resume": exact_resume,
        "resumed_state_sha256": codec_model.state_sha256(resumed),
        "resumed_output_sha256": codec_model.tensor_sha256(resumed_output),
        "resumed_codes_sha256": codec_model.tensor_sha256(resumed_codes),
    }
