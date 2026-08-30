#!/usr/bin/env python3
"""Small deterministic convolutional RVQ codec for R3A V5 controls."""

from __future__ import annotations

import hashlib
import math
from dataclasses import dataclass
from typing import Any

import numpy as np
import physical_sound_contact_field_r3a_v5_common as common
import torch
from torch import nn
from torch.nn import functional

_TORCH_CONFIGURED = False


@dataclass(frozen=True)
class CodecConfig:
    strides: tuple[int, ...]
    base_channels: int
    maximum_channels: int
    latent_dimension: int
    codebook_size: int
    maximum_quantizers: int

    @classmethod
    def frozen(cls) -> CodecConfig:
        return cls(
            strides=tuple(common.MODEL_CONFIG["encoder_strides"]),
            base_channels=common.MODEL_CONFIG["base_channels"],
            maximum_channels=common.MODEL_CONFIG["maximum_channels"],
            latent_dimension=common.MODEL_CONFIG["latent_dimension"],
            codebook_size=common.MODEL_CONFIG["codebook_size"],
            maximum_quantizers=common.MODEL_CONFIG["maximum_quantizers"],
        )

    @classmethod
    def micro(cls) -> CodecConfig:
        return cls(
            strides=(2, 4),
            base_channels=4,
            maximum_channels=8,
            latent_dimension=4,
            codebook_size=8,
            maximum_quantizers=2,
        )


class Snake1d(nn.Module):
    def __init__(self, channels: int) -> None:
        super().__init__()
        self.log_alpha = nn.Parameter(torch.zeros(1, channels, 1))

    def forward(self, value: torch.Tensor) -> torch.Tensor:
        alpha = torch.exp(self.log_alpha).clamp_min(1.0e-4)
        return value + torch.sin(alpha * value).square() / alpha


class ResidualUnit(nn.Module):
    def __init__(self, channels: int, dilation: int) -> None:
        super().__init__()
        self.block = nn.Sequential(
            Snake1d(channels),
            nn.Conv1d(channels, channels, 3, padding=dilation, dilation=dilation),
            Snake1d(channels),
            nn.Conv1d(channels, channels, 1),
        )

    def forward(self, value: torch.Tensor) -> torch.Tensor:
        return value + self.block(value)


class ResidualStack(nn.Module):
    def __init__(self, channels: int) -> None:
        super().__init__()
        self.units = nn.Sequential(
            ResidualUnit(channels, 1),
            ResidualUnit(channels, 3),
            ResidualUnit(channels, 9),
        )

    def forward(self, value: torch.Tensor) -> torch.Tensor:
        return self.units(value)


class ResidualVectorQuantizer(nn.Module):
    def __init__(self, config: CodecConfig) -> None:
        super().__init__()
        self.codebooks = nn.ModuleList(
            [
                nn.Embedding(config.codebook_size, config.latent_dimension)
                for _ in range(config.maximum_quantizers)
            ]
        )
        for codebook in self.codebooks:
            nn.init.uniform_(
                codebook.weight,
                -1.0 / config.codebook_size,
                1.0 / config.codebook_size,
            )

    def forward(
        self, value: torch.Tensor, quantizers: int
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
        if quantizers <= 0 or quantizers > len(self.codebooks):
            raise common.V5Error("RVQ quantizer count is outside the frozen model")
        residual = value
        quantized_sum = torch.zeros_like(value)
        codes = []
        codebook_loss = value.new_zeros(())
        commitment_loss = value.new_zeros(())
        for codebook in self.codebooks[:quantizers]:
            flat = residual.transpose(1, 2).reshape(-1, residual.shape[1])
            weights = codebook.weight
            distance = (
                flat.square().sum(dim=1, keepdim=True)
                + weights.square().sum(dim=1)[None, :]
                - 2.0 * flat @ weights.T
            )
            indices = torch.argmin(distance, dim=1)
            selected = codebook(indices).reshape(
                residual.shape[0], residual.shape[2], residual.shape[1]
            )
            selected = selected.transpose(1, 2)
            codebook_loss = codebook_loss + functional.mse_loss(
                selected, residual.detach()
            )
            commitment_loss = commitment_loss + functional.mse_loss(
                residual, selected.detach()
            )
            straight_through = residual + (selected - residual).detach()
            quantized_sum = quantized_sum + straight_through
            residual = residual - selected.detach()
            codes.append(indices.reshape(value.shape[0], value.shape[2]))
        return (
            quantized_sum,
            torch.stack(codes, dim=1),
            codebook_loss / quantizers,
            commitment_loss / quantizers,
        )


class NeuralImpactCodec(nn.Module):
    def __init__(self, config: CodecConfig) -> None:
        super().__init__()
        self.config = config
        channels = config.base_channels
        encoder: list[nn.Module] = [nn.Conv1d(1, channels, 7, padding=3)]
        encoder_channels = [channels]
        for stride in config.strides:
            encoder.append(ResidualStack(channels))
            next_channels = min(channels * 2, config.maximum_channels)
            encoder.append(
                nn.Conv1d(
                    channels,
                    next_channels,
                    2 * stride + 1,
                    stride=stride,
                    padding=stride,
                )
            )
            channels = next_channels
            encoder_channels.append(channels)
        encoder.extend(
            [
                Snake1d(channels),
                nn.Conv1d(channels, config.latent_dimension, 3, padding=1),
            ]
        )
        self.encoder = nn.Sequential(*encoder)
        self.quantizer = ResidualVectorQuantizer(config)

        decoder: list[nn.Module] = [
            nn.Conv1d(config.latent_dimension, channels, 3, padding=1)
        ]
        for stride, output_channels in zip(
            reversed(config.strides), reversed(encoder_channels[:-1]), strict=True
        ):
            decoder.extend(
                [
                    ResidualStack(channels),
                    Snake1d(channels),
                    nn.ConvTranspose1d(
                        channels,
                        output_channels,
                        2 * stride + 1,
                        stride=stride,
                        padding=stride,
                        output_padding=stride - 1,
                    ),
                ]
            )
            channels = output_channels
        decoder.extend(
            [Snake1d(channels), nn.Conv1d(channels, 1, 7, padding=3), nn.Tanh()]
        )
        self.decoder = nn.Sequential(*decoder)

    def forward(
        self, value: torch.Tensor, quantizers: int
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
        if value.ndim != 3 or value.shape[1] != 1:
            raise common.V5Error("codec input must have shape batch x 1 x samples")
        divisor = math.prod(self.config.strides)
        if value.shape[-1] % divisor:
            raise common.V5Error("codec input length must be divisible by the hop")
        latent = self.encoder(value)
        quantized, codes, codebook_loss, commitment_loss = self.quantizer(
            latent, quantizers
        )
        reconstructed = self.decoder(quantized)
        if reconstructed.shape != value.shape:
            raise common.V5Error("codec inverse changed the sample count")
        return reconstructed, codes, codebook_loss, commitment_loss


def configure_torch() -> None:
    global _TORCH_CONFIGURED
    if _TORCH_CONFIGURED:
        if torch.get_num_threads() != 1 or torch.get_num_interop_threads() != 1:
            raise common.V5Error("V5 torch thread configuration changed")
        return
    torch.set_num_threads(1)
    torch.set_num_interop_threads(1)
    torch.use_deterministic_algorithms(True)
    torch.backends.mkldnn.enabled = False
    _TORCH_CONFIGURED = True


def tensor_sha256(value: torch.Tensor) -> str:
    array = value.detach().cpu().contiguous().numpy()
    return hashlib.sha256(array.tobytes()).hexdigest()


def state_sha256(model: nn.Module) -> str:
    digest = hashlib.sha256()
    for key, value in sorted(model.state_dict().items()):
        array = value.detach().cpu().contiguous().numpy()
        digest.update(key.encode())
        digest.update(str(array.dtype).encode())
        digest.update(np.asarray(array.shape, dtype="<i8").tobytes())
        digest.update(array.tobytes())
    return digest.hexdigest()


def parameter_count(model: nn.Module) -> int:
    return sum(value.numel() for value in model.parameters())


def full_model_control() -> dict[str, Any]:
    configure_torch()
    torch.manual_seed(common.TRAINING_CONFIG["random_seed"])
    model = NeuralImpactCodec(CodecConfig.frozen()).eval()
    samples = math.prod(model.config.strides) * 2
    time = torch.arange(samples, dtype=torch.float32) / common.TARGET_SAMPLE_RATE_HZ
    value = (
        0.7 * torch.exp(-40.0 * time) * torch.sin(2.0 * math.pi * 880.0 * time)
        + 0.2 * torch.exp(-15.0 * time) * torch.sin(2.0 * math.pi * 3_100.0 * time)
    ).reshape(1, 1, -1)
    with torch.inference_mode():
        first = model(value, common.CAPACITIES[-1]["quantizers"])
        second = model(value, common.CAPACITIES[-1]["quantizers"])
    exact = torch.equal(first[0], second[0]) and torch.equal(first[1], second[1])
    if not exact:
        raise common.V5Error("full V5 model control is not exact")
    return {
        "parameter_count": parameter_count(model),
        "state_sha256": state_sha256(model),
        "input_samples": samples,
        "latent_frames": int(first[1].shape[-1]),
        "code_shape": list(first[1].shape),
        "output_sha256": tensor_sha256(first[0]),
        "codes_sha256": tensor_sha256(first[1]),
        "exact_repeat": exact,
    }


def micro_overfit_once(steps: int = 80) -> dict[str, Any]:
    configure_torch()
    torch.manual_seed(17)
    model = NeuralImpactCodec(CodecConfig.micro()).train()
    samples = math.prod(model.config.strides) * 32
    time = torch.arange(samples, dtype=torch.float32) / 8_000.0
    target = (
        0.65 * torch.exp(-18.0 * time) * torch.sin(2.0 * math.pi * 440.0 * time)
        + 0.25 * torch.exp(-8.0 * time) * torch.sin(2.0 * math.pi * 1_170.0 * time)
    ).reshape(1, 1, -1)
    target[..., 8] += 0.8
    optimizer = torch.optim.Adam(model.parameters(), lr=0.01)
    losses = []
    for _ in range(steps):
        reconstructed, _, codebook_loss, commitment_loss = model(target, 2)
        reconstruction_loss = functional.l1_loss(reconstructed, target)
        loss = reconstruction_loss + codebook_loss + 0.25 * commitment_loss
        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        optimizer.step()
        losses.append(float(reconstruction_loss.detach()))
    model.eval()
    with torch.inference_mode():
        output, codes, _, _ = model(target, 2)
    return {
        "steps": steps,
        "initial_reconstruction_l1": losses[0],
        "final_reconstruction_l1": losses[-1],
        "improvement_ratio": losses[-1] / losses[0],
        "state_sha256": state_sha256(model),
        "output_sha256": tensor_sha256(output),
        "codes_sha256": tensor_sha256(codes),
        "code_shape": list(codes.shape),
    }


def micro_overfit_control() -> dict[str, Any]:
    first = micro_overfit_once()
    second = micro_overfit_once()
    exact = first == second
    passed = exact and first["improvement_ratio"] <= 0.45
    if not passed:
        raise common.V5Error(
            f"V5 micro-overfit control failed: exact={exact}, result={first}"
        )
    return {"passed": passed, "exact_repeat": exact, "result": first}
