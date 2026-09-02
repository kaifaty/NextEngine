#!/usr/bin/env python3
"""Owning deterministic full-entry runner for the V25 M0a neural student."""

from __future__ import annotations

import argparse
import json
import os
import platform
import sys
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import mlflow
import numpy as np
import physical_sound_v25_m0a_common as common
import physical_sound_v25_m0a_evaluate as evaluate
import physical_sound_v25_m0a_model as model_lib
import torch

REPORT_SCHEMA = "nextengine.experimental-physical-sound-v25-m0a.report.v1"
CONTROL_SCHEMA = "nextengine.experimental-physical-sound-v25-m0a.controls.v1"
FREEZE_SCHEMA = "nextengine.experimental-physical-sound-v25-m0a.freeze.v1"
HOLDOUT_SCHEMA = "nextengine.experimental-physical-sound-v25-m0a.method-holdout.v1"
REAL_SCHEMA = "nextengine.experimental-physical-sound-v25-m0a.disclosed-real.v1"
MAX_OUTPUT_BYTES = 256 * 1024 * 1024


@dataclass(frozen=True)
class PreparedSynthetic:
    example: common.SyntheticExample
    waveform: np.ndarray
    mesh: common.Mesh
    coarse_mesh: common.Mesh
    coarse_object_features: np.ndarray
    coarse_contact_features: np.ndarray
    point_metres: np.ndarray
    material_id: str
    support_id: str
    family: str


@dataclass(frozen=True)
class TrainingResult:
    model: model_lib.ContactModalField
    step_metrics: tuple[dict[str, float], ...]


def _tensor(value: np.ndarray) -> torch.Tensor:
    return torch.from_numpy(np.asarray(value, dtype=np.float32)).to(device="cpu", dtype=torch.float32)


def _path_within(path: Path, root: Path, role: str) -> Path:
    resolved = path.resolve(strict=True)
    if not resolved.is_relative_to(root):
        raise common.M0Error(f"{role} escapes its declared external root")
    return resolved


class Preprocessor:
    def __init__(
        self,
        manifest: dict[str, Any],
        combined: dict[str, Any],
        teacher_evidence: dict[str, Any],
        profile: common.ExecutionProfile,
    ) -> None:
        self.manifest = manifest
        self.combined = combined
        self.teacher_evidence = teacher_evidence
        self.profile = profile
        self.t0_root = Path(manifest["t0_root"]).resolve(strict=True)
        self.x0_root = Path(manifest["x0_root"]).resolve(strict=True)
        self.ffmpeg = Path(manifest["artifacts"]["ffmpeg"]["path"]).resolve(strict=True)
        self.ffprobe = Path(manifest["artifacts"]["ffprobe"]["path"]).resolve(strict=True)
        self.object_reports = {item["object_id"]: item for item in teacher_evidence["objects"]}
        self.mesh_cache: dict[str, common.Mesh] = {}
        self.mode_cache: dict[str, common.Modes] = {}
        self.gain_cache: dict[str, common.GainField] = {}
        self.transform_records: dict[str, dict[str, Any]] = {}

    def _data(self, reference: dict[str, Any], root: Path, role: str) -> tuple[Path, bytes]:
        path = _path_within(Path(reference["path"]), root, role)
        checked, data = common.validate_ref({"path": str(path), "sha256": reference["sha256"]}, role)
        return checked, data

    def _record(self, transform_id: str, source_sha256: str, value: np.ndarray, units: str) -> None:
        array = np.asarray(value)
        payload = np.ascontiguousarray(array.astype(array.dtype.newbyteorder("<"), copy=False)).tobytes()
        key = f"{transform_id}:{source_sha256}"
        self.transform_records[key] = {
            "transform_id": transform_id,
            "source_sha256": source_sha256,
            "shape": list(array.shape),
            "dtype": array.dtype.str,
            "units": units,
            "output_sha256": common.sha256_bytes(payload),
        }

    def _mesh(self, reference: dict[str, Any], root: Path, role: str) -> common.Mesh:
        key = reference["sha256"]
        if key not in self.mesh_cache:
            _, data = self._data(reference, root, role)
            self.mesh_cache[key] = common.parse_mesh(data, key)
        return self.mesh_cache[key]

    def _modes(self, reference: dict[str, Any]) -> common.Modes:
        key = reference["sha256"]
        if key not in self.mode_cache:
            _, data = self._data(reference, self.t0_root, "M0a modal target")
            self.mode_cache[key] = common.parse_modes(data, self.profile)
        return self.mode_cache[key]

    def _gains(
        self,
        reference: dict[str, Any],
        mesh: common.Mesh,
        modes: common.Modes,
        role: str,
    ) -> common.GainField:
        key = reference["sha256"]
        if key not in self.gain_cache:
            _, data = self._data(reference, self.t0_root, role)
            self.gain_cache[key] = common.parse_gain_field(data, mesh, modes)
        return self.gain_cache[key]

    def synthetic(self, row: dict[str, Any]) -> PreparedSynthetic:
        axes = row.get("axes", {})
        target = axes.get("teacher_target", {})
        geometry = axes.get("geometry", {})
        impact = axes.get("impact", {})
        material = axes.get("material", {}).get("value_id")
        support = axes.get("support", {}).get("value_id")
        if material not in common.MATERIAL_PHYSICS or support not in common.SUPPORT_IDS:
            raise common.M0Error("M0a teacher material or support is unknown")
        mesh_ref = geometry.get("feature_artifact")
        mesh = self._mesh(mesh_ref, self.t0_root, "M0a teacher mesh")
        modes = self._modes(target.get("modal_parameters"))
        gains = self._gains(target.get("contact_gain_field"), mesh, modes, "M0a teacher gain field")
        point = np.asarray(impact.get("point_metres"), dtype=np.float64)
        vertex = common.nearest_vertex(mesh, point)
        object_features = common.full_object_features(mesh, material, support)
        contact_features = common.contact_descriptor(mesh, point)
        count = len(modes.frequencies_hz)
        frequency = np.empty(self.profile.mode_count, dtype=np.float32)
        decay = np.empty(self.profile.mode_count, dtype=np.float32)
        gain = np.zeros(self.profile.mode_count, dtype=np.float32)
        mask = np.zeros(self.profile.mode_count, dtype=np.float32)
        frequency[:count] = modes.frequencies_hz
        decay[:count] = modes.decay_per_second
        gain[:count] = gains.values[vertex]
        mask[:count] = 1.0
        if count < self.profile.mode_count:
            tail = np.geomspace(
                max(float(frequency[count - 1]) * 1.01, 20.0),
                17_500.0,
                self.profile.mode_count - count + 1,
            )[1:]
            frequency[count:] = tail
            decay[count:] = float(decay[count - 1])
        _, audio_data = self._data(row["audio"], self.t0_root, "M0a teacher audio")
        _, waveform = common.decode_float_wav(audio_data)
        object_id = row["object_group_id"].removeprefix("v24-t0-object-")
        report = self.object_reports.get(object_id)
        if not isinstance(report, dict) or report.get("material_id") != material:
            raise common.M0Error("M0a teacher object report binding changed")
        coarse_mesh_path = self.t0_root / f"objects/{object_id}/mesh-coarse.bin"
        coarse_gain_path = self.t0_root / f"objects/{object_id}/contact-gain-field-coarse.bin"
        coarse_mesh_data = common.external_file(coarse_mesh_path, "M0a coarse mesh").read_bytes()
        coarse_mesh = common.parse_mesh(coarse_mesh_data)
        coarse_gain_data = common.external_file(coarse_gain_path, "M0a coarse gain field").read_bytes()
        common.parse_gain_field(coarse_gain_data, coarse_mesh, modes)
        coarse_vertex = common.nearest_vertex(coarse_mesh, point)
        if not np.array_equal(
            common.parse_gain_field(coarse_gain_data, coarse_mesh, modes).values[coarse_vertex],
            gains.values[vertex],
        ):
            raise common.M0Error("M0a coarse/refined teacher truth changed")
        coarse_object = common.full_object_features(coarse_mesh, material, support)
        coarse_contact = common.contact_descriptor(coarse_mesh, point)
        self._record("geometry-descriptor-v2", mesh.sha256, object_features, "fixed-mixed")
        self._record(
            "contact-descriptor-v1",
            common.sha256_bytes(point.astype("<f8").tobytes()),
            contact_features,
            "normalized",
        )
        self._record("teacher-waveform-f32-v1", row["audio"]["sha256"], waveform, "amplitude")
        example = common.SyntheticExample(
            row["row_id"],
            row["split_role"],
            row["sample_role"],
            object_id,
            object_features,
            contact_features,
            frequency,
            decay,
            gain,
            mask,
        )
        return PreparedSynthetic(
            example,
            waveform.astype(np.float32),
            mesh,
            coarse_mesh,
            coarse_object,
            coarse_contact,
            point,
            material,
            support,
            report["family"],
        )

    def transfer(self, row: dict[str, Any]) -> common.RealTransferExample:
        axes = row.get("axes", {})
        if set(axes) != {"material", "geometry", "impact", "listener"}:
            raise common.M0Error("M0a real transfer axes changed or were fabricated")
        material = axes["material"].get("value_id")
        if material != "glass":
            raise common.M0Error("M0a X0 semantic material changed")
        mesh = self._mesh(axes["geometry"]["feature_artifact"], self.x0_root, "M0a X0 mesh")
        point = axes["impact"].get("point_metres")
        common.nearest_vertex(mesh, point)
        object_features = common.full_object_features(mesh, material, None)
        if not np.array_equal(object_features[29:35], np.zeros(6, dtype=np.float32)):
            raise common.M0Error("M0a fabricated X0 physical material values")
        contact_features = common.contact_descriptor(mesh, point)
        _, data = self._data(row["audio"], self.x0_root, "M0a X0 transfer")
        samples = common.align_transfer(data, self.profile).astype(np.float32)
        self._record("realimpact-peak-anchor-v1", row["audio"]["sha256"], samples, "amplitude")
        return common.RealTransferExample(
            row["row_id"],
            row["sample_role"],
            object_features,
            contact_features,
            samples,
        )

    def recording(self, row: dict[str, Any]) -> common.IdentifiedRecording:
        axes = row.get("axes", {})
        if set(axes) != {"material"} or axes["material"].get("value_id") != "glass":
            raise common.M0Error("M0a identified-recording axes changed or were fabricated")
        path, _ = common.validate_ref(row["audio"], "M0a ObjectFolder container")
        samples = common.decode_objectfolder(path, self.ffmpeg, self.ffprobe).astype(np.float32)
        self._record(
            "objectfolder-stereo44100-to-mono48000-v1",
            row["audio"]["sha256"],
            samples,
            "amplitude",
        )
        return common.IdentifiedRecording(row["row_id"], row["sample_role"], samples)

    def manifest_record(self, source_hashes: dict[str, str]) -> dict[str, Any]:
        return {
            "schema": common.PREPROCESS_SCHEMA,
            "status": "Validated",
            "profile": self.profile.profile_id,
            "protocol_sha256": common.PROTOCOL_SHA256,
            "source_hashes": source_hashes,
            "transform_count": len(self.transform_records),
            "transforms": [self.transform_records[key] for key in sorted(self.transform_records)],
            "x0_material_label_known": True,
            "x0_physical_parameters_known": False,
            "x0_support_known": False,
            "admission_shadow_materialized": False,
            "sealed_realimpact_2407_materialized": False,
        }


def _batch(examples: list[PreparedSynthetic], indices: Iterable[int]) -> tuple[torch.Tensor, ...]:
    selected = [examples[index] for index in indices]
    return (
        _tensor(np.stack([item.example.object_features for item in selected])),
        _tensor(np.stack([item.example.contact_features for item in selected])),
        _tensor(np.stack([item.example.frequencies_hz for item in selected])),
        _tensor(np.stack([item.example.decay_per_second for item in selected])),
        _tensor(np.stack([item.example.gains for item in selected])),
        _tensor(np.stack([item.example.mode_mask for item in selected])),
        _tensor(np.stack([item.waveform for item in selected])),
        _tensor(np.stack([item.coarse_object_features for item in selected])),
        _tensor(np.stack([item.coarse_contact_features for item in selected])),
    )


def _cycle_indices(count: int, batch_size: int, steps: int) -> tuple[tuple[int, ...], ...]:
    if count <= 0:
        raise common.M0Error("M0a training set is empty")
    order = np.random.default_rng(model_lib.SEED).permutation(count).tolist()
    cursor = 0
    result = []
    for _ in range(steps):
        batch = []
        while len(batch) < batch_size:
            batch.append(order[cursor])
            cursor += 1
            if cursor == len(order):
                cursor = 0
        result.append(tuple(batch))
    return tuple(result)


def _model_prediction(
    candidate: model_lib.ContactModalField,
    object_features: np.ndarray,
    contact_features: np.ndarray,
    variant: str,
) -> model_lib.ModalPrediction:
    return candidate(
        _tensor(object_features).reshape(1, -1),
        _tensor(contact_features).reshape(1, -1),
        zero_geometry=variant == "neural-no-geometry-v1",
        zero_contact=variant == "neural-no-contact-v1",
        zero_residual=variant == "neural-no-residual-v1",
    )


def train_variant(
    synthetic: list[PreparedSynthetic],
    transfers: list[common.RealTransferExample],
    recordings: list[common.IdentifiedRecording],
    profile: common.ExecutionProfile,
    variant: str,
) -> TrainingResult:
    environment = model_lib.configure_determinism()
    if environment["device"] != "cpu":
        raise common.M0Error("M0a deterministic environment selected a forbidden device")
    candidate = model_lib.ContactModalField().to(device="cpu", dtype=torch.float32)
    model_lib.validate_model(candidate)
    optimizer = torch.optim.AdamW(
        candidate.parameters(),
        lr=model_lib.SYNTHETIC_LEARNING_RATE,
        weight_decay=model_lib.WEIGHT_DECAY,
    )
    step_metrics: list[dict[str, float]] = []
    schedule = _cycle_indices(len(synthetic), profile.batch_size, profile.synthetic_steps)
    for step, indices in enumerate(schedule, start=1):
        (
            object_x,
            contact_x,
            frequency,
            decay,
            gains,
            mask,
            waveform,
            coarse_object,
            coarse_contact,
        ) = _batch(synthetic, indices)
        prediction = candidate(
            object_x,
            contact_x,
            zero_geometry=variant == "neural-no-geometry-v1",
            zero_contact=variant == "neural-no-contact-v1",
            zero_residual=variant == "neural-no-residual-v1",
        )
        loss, components = model_lib.synthetic_loss(prediction, frequency, decay, gains, mask, waveform)
        coarse = candidate(
            coarse_object,
            coarse_contact,
            zero_geometry=variant == "neural-no-geometry-v1",
            zero_contact=variant == "neural-no-contact-v1",
            zero_residual=variant == "neural-no-residual-v1",
        )
        remesh = torch.mean(torch.abs(prediction.gains - coarse.gains))
        total = loss + 0.25 * remesh
        optimizer.zero_grad(set_to_none=True)
        total.backward()
        torch.nn.utils.clip_grad_norm_(candidate.parameters(), model_lib.GRADIENT_CLIP)
        optimizer.step()
        step_metrics.append(
            {
                "step": float(step),
                "loss": float(total.detach()),
                "remesh": float(remesh.detach()),
                "frequency": float(components["log_frequency_huber"].detach()),
            }
        )
    for group in optimizer.param_groups:
        group["lr"] = model_lib.REAL_LEARNING_RATE
    replay_indices = tuple(index % len(synthetic) for index in range(8))
    for offset in range(profile.real_steps):
        real_losses = []
        rendered_context = []
        for item in transfers:
            prediction = _model_prediction(candidate, item.object_features, item.contact_features, variant)
            rendered = model_lib.render_prediction(prediction, min(len(item.samples), 4096))[0]
            target = _tensor(item.samples[:4096])
            components = model_lib.real_acoustic_components(rendered, target)
            real_losses.append(
                0.5 * components["multi_resolution_log_spectrum"]
                + 0.25 * components["decay_envelope_slope"]
                + 0.25 * components["modal_peak_set"]
            )
            rendered_context.append(rendered)
        representative = torch.mean(torch.stack(rendered_context), dim=0)
        for recording in recordings:
            target = _tensor(recording.samples[:4096])
            components = model_lib.real_acoustic_components(representative, target)
            real_losses.append(
                0.5 * components["multi_resolution_log_spectrum"] + 0.25 * components["decay_envelope_slope"]
            )
        replay = _batch(synthetic, replay_indices)
        replay_prediction = candidate(
            replay[0],
            replay[1],
            zero_geometry=variant == "neural-no-geometry-v1",
            zero_contact=variant == "neural-no-contact-v1",
            zero_residual=variant == "neural-no-residual-v1",
        )
        replay_loss, _ = model_lib.synthetic_loss(
            replay_prediction, replay[2], replay[3], replay[4], replay[5], replay[6]
        )
        total = torch.mean(torch.stack(real_losses)) + replay_loss
        optimizer.zero_grad(set_to_none=True)
        total.backward()
        torch.nn.utils.clip_grad_norm_(candidate.parameters(), model_lib.GRADIENT_CLIP)
        optimizer.step()
        step = profile.synthetic_steps + offset + 1
        step_metrics.append(
            {
                "step": float(step),
                "loss": float(total.detach()),
                "remesh": 0.0,
                "frequency": 0.0,
            }
        )
    final_step = profile.synthetic_steps + profile.real_steps
    if int(step_metrics[-1]["step"]) != final_step:
        raise common.M0Error("M0a final checkpoint step changed")
    model_lib.validate_model(candidate)
    return TrainingResult(candidate, tuple(step_metrics))


def _ridge_prediction(
    ridge: model_lib.RidgeModel,
    item: PreparedSynthetic | common.RealTransferExample,
) -> model_lib.ModalPrediction:
    example = item.example if isinstance(item, PreparedSynthetic) else item
    frequency, decay, gains = model_lib.predict_ridge(ridge, example.object_features, example.contact_features)
    return model_lib.ModalPrediction(
        _tensor(frequency).reshape(1, -1),
        _tensor(decay).reshape(1, -1),
        _tensor(gains).reshape(1, -1),
        torch.zeros((1, model_lib.RESIDUAL_COUNT), dtype=torch.float32),
    )


def synthetic_metrics(
    candidate: model_lib.ContactModalField,
    ridge: model_lib.RidgeModel,
    examples: list[PreparedSynthetic],
    variant: str = "neural-full-v1",
) -> dict[str, float]:
    neural_values = {name: [] for name in ("frequency", "decay", "gain", "spectrum")}
    ridge_values = {name: [] for name in neural_values}
    with torch.no_grad():
        for item in examples:
            target = item.example
            neural = _model_prediction(candidate, target.object_features, target.contact_features, variant)
            control = _ridge_prediction(ridge, item)
            mask = target.mode_mask.astype(bool)
            neural_frequency = neural.frequencies_hz[0].numpy()[mask]
            neural_decay = neural.decay_per_second[0].numpy()[mask]
            neural_gain = neural.gains[0].numpy()[mask]
            target_frequency = target.frequencies_hz[mask]
            target_decay = target.decay_per_second[mask]
            target_gain = target.gains[mask]
            ridge_frequency = control.frequencies_hz[0].numpy()[mask]
            ridge_decay = control.decay_per_second[0].numpy()[mask]
            ridge_gain = control.gains[0].numpy()[mask]
            neural_values["frequency"].append(
                float(np.sqrt(np.mean((np.log(neural_frequency) - np.log(target_frequency)) ** 2)))
            )
            ridge_values["frequency"].append(
                float(np.sqrt(np.mean((np.log(ridge_frequency) - np.log(target_frequency)) ** 2)))
            )
            neural_values["decay"].append(float(np.sqrt(np.mean((np.log(neural_decay) - np.log(target_decay)) ** 2))))
            ridge_values["decay"].append(float(np.sqrt(np.mean((np.log(ridge_decay) - np.log(target_decay)) ** 2))))
            normalizer = max(float(np.sqrt(np.mean(target_gain**2))), 1.0e-12)
            neural_values["gain"].append(float(np.sqrt(np.mean((neural_gain - target_gain) ** 2))) / normalizer)
            ridge_values["gain"].append(float(np.sqrt(np.mean((ridge_gain - target_gain) ** 2))) / normalizer)
            target_wave = _tensor(item.waveform)
            neural_wave = model_lib.render_prediction(neural, len(item.waveform))[0]
            ridge_wave = model_lib.render_prediction(control, len(item.waveform))[0]
            neural_values["spectrum"].append(float(model_lib.multi_resolution_log_spectrum(neural_wave, target_wave)))
            ridge_values["spectrum"].append(float(model_lib.multi_resolution_log_spectrum(ridge_wave, target_wave)))
    result = {}
    for name, values in neural_values.items():
        neural_mean = float(np.mean(values))
        ridge_mean = float(np.mean(ridge_values[name]))
        result[f"neural_{name}"] = neural_mean
        result[f"ridge_{name}"] = ridge_mean
        result[f"ratio_{name}"] = evaluate.safe_ratio(neural_mean, ridge_mean)
    return result


def hard_causal_report(
    candidate: model_lib.ContactModalField,
    examples: list[PreparedSynthetic],
) -> dict[str, Any]:
    maximum_peak = 0.0
    maximum_remesh = 0.0
    gradient_nonconstant = True
    ratios = {name: [] for name in ("youngs", "density", "thickness", "scale")}
    for item in examples:
        base_object = item.example.object_features
        contact = item.example.contact_features
        with torch.no_grad():
            base = _model_prediction(candidate, base_object, contact, "neural-full-v1")
            rendered = model_lib.render_prediction(base, 4096)[0]
            low = model_lib.render_prediction(base, 4096, 0.5)[0].numpy()
            unit = rendered.numpy()
            high = model_lib.render_prediction(base, 4096, 2.0)[0].numpy()
            if not np.array_equal(low, unit * 0.5) or not np.array_equal(high, unit * 2.0):
                raise common.M0Error("M0a force scaling is not exact")
            maximum_peak = max(maximum_peak, float(np.max(np.abs(unit))))
            coarse = _model_prediction(
                candidate,
                item.coarse_object_features,
                item.coarse_contact_features,
                "neural-full-v1",
            )
            maximum_remesh = max(maximum_remesh, float(torch.max(torch.abs(base.gains - coarse.gains))))
            physical = common.MATERIAL_PHYSICS[item.material_id]
            youngs = common.full_object_features(
                item.mesh,
                item.material_id,
                item.support_id,
                physical_override=(physical[0] * 4.0, *physical[1:]),
            )
            density = common.full_object_features(
                item.mesh,
                item.material_id,
                item.support_id,
                physical_override=(physical[0], physical[1] * 4.0, *physical[2:]),
            )
            vertices = item.mesh.vertices.copy()
            vertices[:, 2] *= 2.0
            thick_mesh = common.Mesh(
                vertices,
                item.mesh.triangles,
                common.sha256_bytes(vertices.astype("<f8").tobytes()),
            )
            thick_point = item.point_metres.copy()
            thick_point[2] *= 2.0
            thick = common.full_object_features(thick_mesh, item.material_id, item.support_id)
            scale_vertices = item.mesh.vertices.copy()
            scale_point = item.point_metres.copy()
            scale_vertices[:, 0] *= 2.0
            scale_point[0] *= 2.0
            if item.family == "plate":
                scale_vertices[:, 1] *= 2.0
                scale_point[1] *= 2.0
            scale_mesh = common.Mesh(
                scale_vertices,
                item.mesh.triangles,
                common.sha256_bytes(scale_vertices.astype("<f8").tobytes()),
            )
            scale = common.full_object_features(scale_mesh, item.material_id, item.support_id)
            mutations = {
                "youngs": (youngs, contact),
                "density": (density, contact),
                "thickness": (
                    thick,
                    common.contact_descriptor(thick_mesh, thick_point),
                ),
                "scale": (scale, common.contact_descriptor(scale_mesh, scale_point)),
            }
            for name, (object_value, contact_value) in mutations.items():
                changed = _model_prediction(candidate, object_value, contact_value, "neural-full-v1")
                ratios[name].append(float(torch.median(changed.frequencies_hz / base.frequencies_hz)))
        contact_tensor = _tensor(contact).reshape(1, -1).requires_grad_(True)
        object_tensor = _tensor(base_object).reshape(1, -1)
        prediction = candidate(object_tensor, contact_tensor)
        gradient = torch.autograd.grad(torch.sum(prediction.gains), contact_tensor)[0]
        gradient_nonconstant = (
            gradient_nonconstant
            and bool(torch.all(torch.isfinite(gradient)))
            and float(torch.max(torch.abs(gradient))) > 0.0
        )
    expected = {"youngs": 2.0, "density": 0.5, "thickness": 2.0, "scale": 0.25}
    counterfactuals = {}
    for name, values in ratios.items():
        observed = float(np.median(values))
        counterfactuals[name] = {
            "observed_ratio": observed,
            "expected_ratio": expected[name],
            "relative_error": abs(observed / expected[name] - 1.0),
        }
    return {
        "maximum_render_peak": maximum_peak,
        "maximum_remesh_gain_absolute_difference": maximum_remesh,
        "contact_gradients_finite_nonconstant": gradient_nonconstant,
        "counterfactuals": counterfactuals,
        "force_scaling_exact": True,
    }


def predictions_bytes(
    candidate: model_lib.ContactModalField,
    examples: list[PreparedSynthetic],
    transfers: list[common.RealTransferExample],
) -> bytes:
    tensors: dict[str, np.ndarray] = {}
    with torch.no_grad():
        for index, item in enumerate(examples):
            prediction = _model_prediction(
                candidate,
                item.example.object_features,
                item.example.contact_features,
                "neural-full-v1",
            )
            for name, value in model_lib.prediction_numpy(prediction).items():
                tensors[f"synthetic.{index:04d}.{name}"] = value
        for index, item in enumerate(transfers):
            prediction = _model_prediction(candidate, item.object_features, item.contact_features, "neural-full-v1")
            for name, value in model_lib.prediction_numpy(prediction).items():
                tensors[f"transfer.{index:04d}.{name}"] = value
    return common.canonical_tensor_bytes(dict(sorted(tensors.items())))


def _mlflow_log(
    root: Path,
    manifest: dict[str, Any],
    report: dict[str, Any],
    step_metrics: tuple[dict[str, float], ...],
    artifact_hashes: dict[str, str],
) -> dict[str, str]:
    root.mkdir(parents=True, exist_ok=True)
    tracking_uri = root.resolve(strict=True).as_uri()
    if not tracking_uri.startswith("file://"):
        raise common.M0Error("M0a MLflow tracking URI must be local file://")
    os.environ["MLFLOW_ALLOW_FILE_STORE"] = "true"
    mlflow.autolog(disable=True)
    mlflow.set_tracking_uri(tracking_uri)
    mlflow.set_experiment("nextengine-physical-sound-v25-m0a")
    with mlflow.start_run(run_name="m0a-contact-modal-field-v1") as active:
        mlflow.log_params(
            {
                "model_id": common.MODEL_ID,
                "profile": manifest["profile"],
                "protocol_sha256": common.PROTOCOL_SHA256,
                "implementation_root_sha256": manifest["implementation_root_sha256"],
                "seed": model_lib.SEED,
                "parameter_count": report["parameter_count"],
                "cpu_only": True,
            }
        )
        for item in step_metrics:
            mlflow.log_metric("loss", item["loss"], step=int(item["step"]))
        for name, value in sorted(artifact_hashes.items()):
            mlflow.log_param(f"sha256_{name.replace('.', '_')}", value)
        return {"tracking_uri": tracking_uri, "run_id": active.info.run_id}


def run(manifest_path: Path, output_path: Path) -> dict[str, Any]:
    manifest, manifest_bytes = common.load_manifest(manifest_path)
    common.external_directory(output_path, "M0a output", must_exist=False)
    profile = common.execution_profile(manifest["profile"])
    combined, combined_bytes = common.load_combined(manifest)
    _, teacher_data = common.validate_ref(
        manifest["artifacts"]["teacher_evidence"],
        "M0a teacher evidence",
        common.MAX_JSON_BYTES,
    )
    teacher = common.parse_json_bytes(teacher_data, "M0a teacher evidence")
    if (
        teacher.get("schema") != common.TEACHER_SCHEMA
        or teacher.get("status") != "Validated"
        or teacher.get("implementation_sha256") != common.T0_IMPLEMENTATION_SHA256
        or teacher.get("model_training_authorized") is not False
        or teacher.get("real_material_authorized") is not False
        or teacher.get("runtime_authorized") is not False
    ):
        raise common.M0Error("M0a teacher evidence identity or authority changed")
    _, x0_data = common.validate_ref(manifest["artifacts"]["x0_lineage"], "M0a X0 lineage", common.MAX_JSON_BYTES)
    x0 = common.parse_json_bytes(x0_data, "M0a X0 lineage")
    if (
        x0.get("schema") != common.X0_SCHEMA
        or x0.get("status") != "Validated"
        or x0.get("sealed_contact", {}).get("decoded_sample_count") != 0
        or x0.get("model_training_authorized") is not False
        or x0.get("real_material_admission_authorized") is not False
        or x0.get("runtime_authorized") is not False
    ):
        raise common.M0Error("M0a X0 lineage identity, seal or authority changed")
    common.validate_combined_lineage_bindings(combined, manifest)
    teacher_root = common.reconstruct_teacher_artifact_root(
        Path(manifest["t0_root"]).resolve(strict=True), teacher, combined
    )
    vault = common.EvidenceVault(combined)
    preprocessor = Preprocessor(manifest, combined, teacher, profile)
    vault.begin_training()
    synthetic_train = [preprocessor.synthetic(row) for row in vault.synthetic_train_context()]
    synthetic_train_query = [preprocessor.synthetic(row) for row in vault.synthetic_train_query()]
    transfer_context = [preprocessor.transfer(row) for row in vault.real_context("exact_real_transfer")]
    recording_context = [preprocessor.recording(row) for row in vault.real_context("identified_real_recording")]
    if len(transfer_context) != 3 or len(recording_context) != 2:
        raise common.M0Error("M0a disclosed real context shape changed")
    ridge = model_lib.fit_ridge([item.example for item in synthetic_train])
    candidate_a = train_variant(synthetic_train, transfer_context, recording_context, profile, "neural-full-v1")
    candidate_b = train_variant(synthetic_train, transfer_context, recording_context, profile, "neural-full-v1")
    weights_a = common.canonical_tensor_bytes(model_lib.model_tensors(candidate_a.model))
    weights_b = common.canonical_tensor_bytes(model_lib.model_tensors(candidate_b.model))
    train_predictions_a = predictions_bytes(
        candidate_a.model, [*synthetic_train, *synthetic_train_query], transfer_context
    )
    train_predictions_b = predictions_bytes(
        candidate_b.model, [*synthetic_train, *synthetic_train_query], transfer_context
    )
    if weights_a != weights_b or train_predictions_a != train_predictions_b:
        raise common.M0Error("M0a internal neural A/B canonical bytes differ")
    no_geometry = train_variant(
        synthetic_train,
        transfer_context,
        recording_context,
        profile,
        "neural-no-geometry-v1",
    )
    no_contact = train_variant(
        synthetic_train,
        transfer_context,
        recording_context,
        profile,
        "neural-no-contact-v1",
    )
    no_residual = train_variant(
        synthetic_train,
        transfer_context,
        recording_context,
        profile,
        "neural-no-residual-v1",
    )
    vault.finish_training()
    development = [preprocessor.synthetic(row) for row in vault.synthetic_development()]
    calibration = [preprocessor.synthetic(row) for row in vault.synthetic_calibration()]
    if not development or not calibration:
        raise common.M0Error("M0a synthetic development/calibration is empty")
    full_metrics = synthetic_metrics(candidate_a.model, ridge, development)
    geometry_metrics = synthetic_metrics(no_geometry.model, ridge, development, "neural-no-geometry-v1")
    contact_metrics = synthetic_metrics(no_contact.model, ridge, development, "neural-no-contact-v1")
    residual_metrics = synthetic_metrics(no_residual.model, ridge, development, "neural-no-residual-v1")
    hard = hard_causal_report(candidate_a.model, [*synthetic_train, *development, *calibration])
    parameter_count = model_lib.parameter_count(candidate_a.model)
    official_synthetic_pass = (
        all(full_metrics[f"ratio_{name}"] <= 0.90 for name in ("frequency", "decay", "gain", "spectrum"))
        and geometry_metrics["neural_frequency"] >= full_metrics["neural_frequency"] * 1.05
        and contact_metrics["neural_gain"] >= full_metrics["neural_gain"] * 1.05
        and hard["maximum_render_peak"] < 0.95
        and hard["maximum_remesh_gain_absolute_difference"] <= 1.0e-5
        and hard["contact_gradients_finite_nonconstant"]
        and all(item["relative_error"] <= 0.10 for item in hard["counterfactuals"].values())
    )
    synthetic_pass = official_synthetic_pass or profile.profile_id == "contract-fixture-v1"
    weights_sha256 = common.sha256_bytes(weights_a)
    predictions = predictions_bytes(
        candidate_a.model,
        [*synthetic_train, *synthetic_train_query, *development, *calibration],
        transfer_context,
    )
    predictions_sha256 = common.sha256_bytes(predictions)
    source_hashes = {
        "manifest": common.sha256_bytes(manifest_bytes),
        "combined_manifest": common.sha256_bytes(combined_bytes),
        "teacher_evidence": common.sha256_bytes(teacher_data),
        "x0_lineage": common.sha256_bytes(x0_data),
        "protocol": common.PROTOCOL_SHA256,
        "implementation": manifest["implementation_root_sha256"],
    }
    preprocess_record = preprocessor.manifest_record(source_hashes)
    control_report = {
        "schema": CONTROL_SCHEMA,
        "status": "Validated" if synthetic_pass else "Rejected",
        "profile": profile.profile_id,
        "ridge_lambda": model_lib.RIDGE_LAMBDA,
        "full": full_metrics,
        "no_geometry": geometry_metrics,
        "no_contact": contact_metrics,
        "no_residual": residual_metrics,
        "hard_causal": hard,
        "official_synthetic_gates_pass": official_synthetic_pass,
        "contract_fixture_has_no_quality_claim": profile.profile_id == "contract-fixture-v1",
    }
    staging, output = common.prepare_output(output_path)
    try:
        artifacts = {}
        artifacts["preprocess-manifest.json"] = common.write_artifact(
            staging,
            "preprocess-manifest.json",
            common.canonical_json(preprocess_record),
        )
        artifacts["control-report.json"] = common.write_artifact(
            staging, "control-report.json", common.canonical_json(control_report)
        )
        artifacts["candidate-weights.bin"] = common.write_artifact(staging, "candidate-weights.bin", weights_a)
        artifacts["candidate-predictions.bin"] = common.write_artifact(
            staging, "candidate-predictions.bin", predictions
        )
        decision = "REPRESENTATION_REJECT"
        holdout_report: dict[str, Any] | None = None
        real_report: dict[str, Any] | None = None
        if synthetic_pass:
            freeze = {
                "schema": FREEZE_SCHEMA,
                "status": "Frozen",
                "model_id": common.MODEL_ID,
                "profile": profile.profile_id,
                "seed": model_lib.SEED,
                "final_step": profile.synthetic_steps + profile.real_steps,
                "parameter_count": parameter_count,
                "weights_sha256": weights_sha256,
                "predictions_sha256": predictions_sha256,
                "protocol_sha256": common.PROTOCOL_SHA256,
                "implementation_root_sha256": manifest["implementation_root_sha256"],
                "method_holdout_opened": False,
                "admission_shadow_opened": False,
            }
            artifacts["candidate-freeze.json"] = common.write_artifact(
                staging, "candidate-freeze.json", common.canonical_json(freeze)
            )
            vault.freeze_candidate(weights_sha256)
            holdout = [preprocessor.synthetic(row) for row in vault.synthetic_method_holdout()]
            holdout_metrics = synthetic_metrics(candidate_a.model, ridge, holdout)
            official_holdout_pass = all(
                holdout_metrics[f"ratio_{name}"] <= 0.95 for name in ("frequency", "decay", "gain", "spectrum")
            )
            holdout_pass = official_holdout_pass or profile.profile_id == "contract-fixture-v1"
            holdout_report = {
                "schema": HOLDOUT_SCHEMA,
                "status": "Pass" if holdout_pass else "Reject",
                "candidate_weights_sha256": weights_sha256,
                "metrics": holdout_metrics,
                "official_gates_pass": official_holdout_pass,
                "contract_fixture_has_no_quality_claim": profile.profile_id == "contract-fixture-v1",
                "admission_shadow_opened": False,
            }
            artifacts["method-holdout-report.json"] = common.write_artifact(
                staging,
                "method-holdout-report.json",
                common.canonical_json(holdout_report),
            )
            if holdout_pass:
                transfer_queries = [preprocessor.transfer(row) for row in vault.real_queries("exact_real_transfer")]
                recording_queries = [
                    preprocessor.recording(row) for row in vault.real_queries("identified_real_recording")
                ]
                if len(transfer_queries) != 1 or len(recording_queries) != 1:
                    raise common.M0Error("M0a disclosed real query shape changed")
                real_metrics = evaluate.disclosed_real_metrics(
                    candidate_a.model,
                    ridge,
                    transfer_context,
                    transfer_queries[0],
                    recording_context,
                    recording_queries[0],
                )
                official_real_pass = (
                    real_metrics["full_to_ridge_ratio"] <= 0.95
                    and real_metrics["full_to_nearest_ratio"] <= 0.95
                    and real_metrics["improved_component_count"] >= 2
                    and real_metrics["objectfolder_candidate_distance"] <= real_metrics["objectfolder_nearest_distance"]
                )
                real_pass = official_real_pass or profile.profile_id == "contract-fixture-v1"
                vault.finish_real_queries()
                real_report = {
                    "schema": REAL_SCHEMA,
                    "status": "Pass" if real_pass else "DomainGapReject",
                    "candidate_weights_sha256": weights_sha256,
                    "metrics": real_metrics,
                    "official_gates_pass": official_real_pass,
                    "contract_fixture_has_no_quality_claim": profile.profile_id == "contract-fixture-v1",
                    "realimpact_2407_opened": False,
                    "admission_shadow_opened": False,
                }
                artifacts["disclosed-real-report.json"] = common.write_artifact(
                    staging,
                    "disclosed-real-report.json",
                    common.canonical_json(real_report),
                )
                decision = "PASS" if real_pass else "DOMAIN_GAP_REJECT"
            else:
                decision = "METHOD_HOLDOUT_REJECT"
        output_bytes = sum(item["byte_count"] for item in artifacts.values())
        if output_bytes > MAX_OUTPUT_BYTES:
            raise common.M0Error("M0a canonical output exceeds 256 MiB")
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "profile": profile.profile_id,
            "model_id": common.MODEL_ID,
            "protocol_sha256": common.PROTOCOL_SHA256,
            "implementation_root_sha256": manifest["implementation_root_sha256"],
            "parameter_count": parameter_count,
            "seed": model_lib.SEED,
            "final_step": profile.synthetic_steps + profile.real_steps,
            "environment": {
                "python": platform.python_version(),
                "numpy": np.__version__,
                "torch": torch.__version__,
                "mlflow": mlflow.__version__,
                "cpu_only": True,
                "threads": 1,
            },
            "teacher_aggregate_verification": teacher_root,
            "internal_candidate_ab_byte_exact": True,
            "canonical_payload_bytes_before_report": output_bytes,
            "access_log": vault.access_log,
            "admission_shadow_opened": False,
            "realimpact_2407_opened": False,
            "model_training_executed": True,
            "real_material_admission_authorized": False,
            "runtime_authorized": False,
            "mlflow": {
                "tracking_uri_scheme": "file",
                "diagnostic_only": True,
                "autolog_disabled": True,
                "registry_disabled": True,
                "serving_disabled": True,
                "volatile_run_id_excluded": True,
            },
        }
        artifacts["report.json"] = common.write_artifact(staging, "report.json", common.canonical_json(report))
        artifact_hashes = {name: item["sha256"] for name, item in artifacts.items()}
        _mlflow_log(
            Path(manifest["mlflow_root"]),
            manifest,
            report,
            candidate_a.step_metrics,
            artifact_hashes,
        )
        common.publish_output(staging, output)
        return report
    except BaseException:
        common.abandon_output(staging)
        raise


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = run(arguments.manifest, arguments.output)
    except Exception as error:  # noqa: BLE001 - stable owning CLI boundary
        print(f"physical-sound-v25-m0a: {error}", file=sys.stderr)
        return 1
    print(json.dumps(report, indent=2, sort_keys=True, allow_nan=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
