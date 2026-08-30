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

RUN_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-capacity-run.v1"
STATE_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-capacity-state.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v5-capacity.report.v1"
REVISION = "three-capacity-training-v1"
TRAINING_PREFLIGHT_MANIFEST_SHA256 = (
    "75ed5e06cff62bc546645314cf1708259b91f2d07bba6972349780b848aa2679"
)
V4_MANIFEST_SHA256 = "6f9fe00ea14c99da2b2fc8b71b22af6772725ac5f28430a2417b8717819386b1"
METRIC_INTERVAL_STEPS = 100
FIT_CONTACTS_PER_OBJECT = 3
INTERNET_TRAIN_CLIP_COUNT = 56
INTERNET_VALIDATION_CLIP_COUNT = 17
TOTAL_TRAIN_ITEM_COUNT = 68


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


def _train_step(
    model: codec_model.NeuralImpactCodec,
    optimizer: torch.optim.Optimizer,
    target: torch.Tensor,
    mask: torch.Tensor,
    quantizers: int,
    step: int,
) -> dict[str, float]:
    rate = learning_rate_for_step(step)
    for group in optimizer.param_groups:
        group["lr"] = rate
    model.train()
    output, _, codebook_loss, commitment_loss = model(target, quantizers)
    total, terms = training.frozen_reconstruction_loss(
        output * mask,
        target * mask,
        codebook_loss,
        commitment_loss,
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
        "learning_rate": rate,
        "total": float(total.detach()),
        "gradient_norm": float(gradient_norm.detach()),
        **{name: float(value.detach()) for name, value in terms.items()},
    }


def evaluate_validation(
    model: codec_model.NeuralImpactCodec,
    validation: list[WaveformItem],
    quantizers: int,
    device: torch.device,
) -> dict[str, Any]:
    totals: dict[str, list[float]] = {}
    unique_by_quantizer: list[set[int]] = [set() for _ in range(quantizers)]
    model.eval()
    with torch.inference_mode():
        for item in validation:
            segment, mask = _aligned_segment(item.samples)
            target = _tensor(segment, device)
            mask_tensor = _tensor(mask, device)
            output, codes, codebook_loss, commitment_loss = model(target, quantizers)
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
    return {
        "clip_count": len(validation),
        "mean": {name: float(np.mean(values)) for name, values in totals.items()},
        "maximum": {name: float(np.max(values)) for name, values in totals.items()},
        "unique_codes_per_quantizer": [len(values) for values in unique_by_quantizer],
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


def _implementation_sha256() -> str:
    return common.sha256_file(Path(__file__).resolve())


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
    return (
        model,
        optimizer,
        int(loaded["step"]),
        state.get("best_validation_loss"),
        state.get("best_checkpoint"),
    )


def run(root: Path, arguments: argparse.Namespace) -> Path:
    output = arguments.output.resolve()
    if output.is_relative_to(root):
        raise common.V5Error("V5 capacity output must stay outside the repository")
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
        model, optimizer, completed_step, best_loss, best_checkpoint = _load_resume(
            output, run_manifest_sha256, arguments.capacity_id, device
        )
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
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
        }
        _write_json_atomic(output / "state.json", state)

    maximum_steps = common.TRAINING_CONFIG["maximum_steps"]
    checkpoint_interval = common.TRAINING_CONFIG["checkpoint_interval_steps"]
    last_metrics: dict[str, Any] | None = None
    for step in range(completed_step + 1, maximum_steps + 1):
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
                f"{arguments.capacity_id} step {step}/{maximum_steps} "
                f"loss={last_metrics['total']:.6f} item={item.id} "
                f"segment={segment_policy}",
                flush=True,
            )
        if step % checkpoint_interval != 0 and step != maximum_steps:
            continue
        validation_metrics = evaluate_validation(
            model, validation, capacity["quantizers"], device
        )
        checkpoint_name = f"checkpoint-step-{step:08d}.pt"
        checkpoint_path = output / checkpoint_name
        checkpoint = {
            "schema": training.CHECKPOINT_SCHEMA,
            "capacity_id": arguments.capacity_id,
            "quantizers": capacity["quantizers"],
            "step": step,
            "run_manifest_sha256": run_manifest_sha256,
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
        if best_loss is None or validation_loss < best_loss:
            best_loss = validation_loss
            best_checkpoint = checkpoint_record
        record = {
            "step": step,
            "train": last_metrics,
            "validation": validation_metrics,
            "checkpoint": checkpoint_record,
            "best_validation_loss": best_loss,
            "best_checkpoint": best_checkpoint,
            "last_item": item.id,
            "last_segment_policy": segment_policy,
        }
        _append_metric(metrics_path, record)
        state = {
            "schema": STATE_SCHEMA,
            "status": "InProgress",
            "capacity_id": arguments.capacity_id,
            "run_manifest_sha256": run_manifest_sha256,
            "completed_step": step,
            "latest_checkpoint": checkpoint_name,
            "best_validation_loss": best_loss,
            "best_checkpoint": best_checkpoint,
            "development_waveform_samples_decoded": 0,
            "sealed_waveform_samples_decoded": 0,
        }
        _write_json_atomic(output / "state.json", state)

    if best_checkpoint is None or last_metrics is None:
        raise common.V5Error("V5 capacity training produced no checkpoint")
    metrics_sha256 = common.sha256_file(metrics_path)
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Complete",
        "decision": "CAPACITY_TRAINING_COMPLETE",
        "capacity_id": arguments.capacity_id,
        "quantizers": capacity["quantizers"],
        "run_manifest_sha256": run_manifest_sha256,
        "metrics_sha256": metrics_sha256,
        "completed_steps": maximum_steps,
        "best_validation_loss": best_loss,
        "best_checkpoint": best_checkpoint,
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
        "status": "Complete",
        "capacity_id": arguments.capacity_id,
        "run_manifest_sha256": run_manifest_sha256,
        "completed_step": maximum_steps,
        "latest_checkpoint": f"checkpoint-step-{maximum_steps:08d}.pt",
        "best_validation_loss": best_loss,
        "best_checkpoint": best_checkpoint,
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
    if state["status"] == "Complete":
        print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
