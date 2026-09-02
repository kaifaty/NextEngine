#!/usr/bin/env python3
"""Value-independent equivalence and resource oracles for V28 M0c."""

from __future__ import annotations

import argparse
import gc
import os
import resource as process_resource
import time
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v25_m0a_common as base
import physical_sound_v25_m0a_model as inherited_model
import physical_sound_v25_m0a_train as inherited_train
import physical_sound_v28_m0c_common as contract
import physical_sound_v28_m0c_model as model_lib
import torch

EQUIVALENCE_SCHEMA = "nextengine.experimental-physical-sound-v28-m0c.equivalence.v1"
RESOURCE_SCHEMA = "nextengine.experimental-physical-sound-v28-m0c.resource.v1"
RESOURCE_PROFILE_ID = "m0c-resource-oracle-v1"
RESOURCE_SYNTHETIC_EXAMPLES = 32
RESOURCE_TRANSFER_CONTEXTS = 3
RESOURCE_RECORDING_CONTEXTS = 2
RESOURCE_WALL_LIMIT_SECONDS = 600.0
RESOURCE_RSS_LIMIT_BYTES = 3_758_096_384
RESOURCE_VARIANTS = (
    "neural-full-v1",
    "neural-full-v1",
    "neural-no-geometry-v1",
    "neural-no-contact-v1",
    "neural-no-residual-v1",
)


def _features(rng: np.random.Generator, count: int) -> np.ndarray:
    return rng.normal(0.0, 0.35, count).astype(np.float32)


def _resource_fixture() -> tuple[
    list[inherited_train.PreparedSynthetic],
    list[base.RealTransferExample],
    list[base.IdentifiedRecording],
    base.ExecutionProfile,
]:
    rng = np.random.default_rng(model_lib.SEED)
    profile = base.ExecutionProfile(
        RESOURCE_PROFILE_ID,
        1_500,
        500,
        16,
        230_215,
        144_000,
        512,
        model_lib.MODE_COUNT,
        True,
    )
    vertices = np.asarray(
        [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        dtype=np.float64,
    )
    triangles = np.asarray([[0, 1, 2]], dtype=np.uint32)
    mesh = base.Mesh(vertices, triangles, base.sha256_bytes(vertices.tobytes()))
    time_prefix = np.arange(
        model_lib.TRAINING_RENDER_FRAMES, dtype=np.float32
    ) / np.float32(model_lib.SAMPLE_RATE_HZ)
    synthetic: list[inherited_train.PreparedSynthetic] = []
    for index in range(RESOURCE_SYNTHETIC_EXAMPLES):
        base_frequency = np.geomspace(
            90.0 + index, 12_000.0 + index, model_lib.MODE_COUNT
        ).astype(np.float32)
        decay = np.linspace(3.0, 75.0, model_lib.MODE_COUNT, dtype=np.float32)
        gains = np.tanh(rng.normal(0.0, 0.5, model_lib.MODE_COUNT)).astype(np.float32)
        waveform = np.zeros(profile.transfer_window, dtype=np.float32)
        waveform[: model_lib.TRAINING_RENDER_FRAMES] = np.sum(
            gains[:, None]
            * np.exp(-decay[:, None] * time_prefix)
            * np.sin(2.0 * np.pi * base_frequency[:, None] * time_prefix),
            axis=0,
            dtype=np.float32,
        ) * np.float32(0.0125)
        object_features = _features(rng, model_lib.OBJECT_FEATURE_COUNT)
        contact_features = _features(rng, model_lib.CONTACT_FEATURE_COUNT)
        example = base.SyntheticExample(
            f"m0c-resource-synthetic-{index:02d}",
            "train",
            "context",
            f"resource-object-{index:02d}",
            object_features,
            contact_features,
            base_frequency,
            decay,
            gains,
            np.ones(model_lib.MODE_COUNT, dtype=np.float32),
        )
        synthetic.append(
            inherited_train.PreparedSynthetic(
                example,
                waveform,
                mesh,
                mesh,
                object_features + np.float32(1.0e-4),
                contact_features + np.float32(1.0e-4),
                np.asarray([0.25, 0.25, 0.0], dtype=np.float64),
                "elastic-a",
                "simply-supported-all-edges",
                "resource-fixture",
            )
        )
    transfers = []
    for index in range(RESOURCE_TRANSFER_CONTEXTS):
        samples = np.zeros(model_lib.TRAINING_RENDER_FRAMES, dtype=np.float32)
        samples[64 + index] = np.float32(0.5)
        transfers.append(
            base.RealTransferExample(
                f"m0c-resource-transfer-{index}",
                "context",
                _features(rng, model_lib.OBJECT_FEATURE_COUNT),
                _features(rng, model_lib.CONTACT_FEATURE_COUNT),
                samples,
            )
        )
    recordings = []
    for index in range(RESOURCE_RECORDING_CONTEXTS):
        samples = np.zeros(model_lib.TRAINING_RENDER_FRAMES, dtype=np.float32)
        samples[96 + index] = np.float32(0.25)
        recordings.append(
            base.IdentifiedRecording(
                f"m0c-resource-recording-{index}", "context", samples
            )
        )
    return synthetic, transfers, recordings, profile


def _prediction_targets(
    batch: int, *, extreme: bool = False
) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
    if extreme:
        frequency_row = torch.logspace(
            np.log10(20.0), np.log10(18_000.0), model_lib.MODE_COUNT
        )
        decay_row = torch.logspace(-4.0, 2.0, model_lib.MODE_COUNT)
        gains_row = torch.linspace(-1.0, 1.0, model_lib.MODE_COUNT)
    else:
        frequency_row = torch.logspace(
            np.log10(80.0), np.log10(12_000.0), model_lib.MODE_COUNT
        )
        decay_row = torch.linspace(2.0, 80.0, model_lib.MODE_COUNT)
        gains_row = torch.tanh(torch.randn(model_lib.MODE_COUNT))
    return (
        frequency_row.repeat(batch, 1),
        decay_row.repeat(batch, 1),
        gains_row.repeat(batch, 1),
        torch.ones(batch, model_lib.MODE_COUNT),
    )


def _loss_and_gradients(
    candidate: model_lib.ContactModalField,
    object_features: torch.Tensor,
    contact_features: torch.Tensor,
    targets: tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor],
    waveform: torch.Tensor,
    *,
    prefix_bounded: bool,
) -> tuple[torch.Tensor, dict[str, torch.Tensor], tuple[torch.Tensor, ...]]:
    candidate.zero_grad(set_to_none=True)
    prediction = candidate(object_features, contact_features)
    loss_owner = (
        model_lib.synthetic_loss if prefix_bounded else inherited_model.synthetic_loss
    )
    loss, components = loss_owner(prediction, *targets, waveform)
    loss.backward()
    gradients = tuple(
        parameter.grad.detach().clone() for parameter in candidate.parameters()
    )
    return (
        loss.detach(),
        {key: value.detach() for key, value in components.items()},
        gradients,
    )


def _equivalence_case(
    case_id: str,
    object_features: torch.Tensor,
    contact_features: torch.Tensor,
    *,
    extreme: bool,
) -> dict[str, Any]:
    batch = object_features.shape[0]
    targets = _prediction_targets(batch, extreme=extreme)
    teacher = model_lib.ModalPrediction(
        targets[0],
        targets[1],
        targets[2],
        torch.zeros(batch, model_lib.RESIDUAL_COUNT),
    )
    with torch.no_grad():
        target_waveform = model_lib.render_prediction(teacher, 144_000)
    full_model = model_lib.ContactModalField()
    prefix_model = model_lib.ContactModalField()
    prefix_model.load_state_dict(full_model.state_dict())
    with torch.no_grad():
        prediction = full_model(object_features, contact_features)
        full_render = model_lib.render_prediction(prediction, 144_000)
        prefix_render = model_lib.render_prediction(
            prediction, model_lib.TRAINING_RENDER_FRAMES
        )
        render_equal = torch.equal(
            full_render[..., : model_lib.TRAINING_RENDER_FRAMES], prefix_render
        )
    full_loss, full_components, full_gradients = _loss_and_gradients(
        full_model,
        object_features,
        contact_features,
        targets,
        target_waveform,
        prefix_bounded=False,
    )
    prefix_loss, prefix_components, prefix_gradients = _loss_and_gradients(
        prefix_model,
        object_features,
        contact_features,
        targets,
        target_waveform,
        prefix_bounded=True,
    )
    component_equal = all(
        torch.equal(full_components[key], prefix_components[key])
        for key in full_components
    )
    gradient_equal = all(
        torch.equal(full, prefix)
        for full, prefix in zip(full_gradients, prefix_gradients, strict=True)
    )
    result = {
        "case_id": case_id,
        "render_prefix_exact": bool(render_equal),
        "loss_exact": bool(torch.equal(full_loss, prefix_loss)),
        "components_exact": bool(component_equal),
        "gradients_exact": bool(gradient_equal),
    }
    del target_waveform, full_render, prefix_render
    gc.collect()
    return result


def equivalence_report() -> dict[str, Any]:
    model_lib.configure_determinism()
    cases = []
    for offset in range(5):
        torch.manual_seed(model_lib.SEED + offset)
        cases.append(
            _equivalence_case(
                f"random-{offset}",
                torch.randn(3, model_lib.OBJECT_FEATURE_COUNT),
                torch.randn(3, model_lib.CONTACT_FEATURE_COUNT),
                extreme=False,
            )
        )
    for label, value in (("negative", -1.0), ("zero", 0.0), ("positive", 1.0)):
        torch.manual_seed(model_lib.SEED)
        cases.append(
            _equivalence_case(
                f"bounded-{label}",
                torch.full((3, model_lib.OBJECT_FEATURE_COUNT), value),
                torch.full((3, model_lib.CONTACT_FEATURE_COUNT), value),
                extreme=True,
            )
        )

    torch.manual_seed(model_lib.SEED)
    object_features = torch.randn(2, model_lib.OBJECT_FEATURE_COUNT)
    contact_features = torch.randn(2, model_lib.CONTACT_FEATURE_COUNT)
    targets = _prediction_targets(2)
    teacher = model_lib.ModalPrediction(
        targets[0], targets[1], targets[2], torch.zeros(2, model_lib.RESIDUAL_COUNT)
    )
    with torch.no_grad():
        target = model_lib.render_prediction(teacher, 144_000)
    late = target.clone()
    late[..., 5_000] += 1.0
    early = target.clone()
    early[..., 100] += 1.0
    baseline_model = model_lib.ContactModalField()
    late_model = model_lib.ContactModalField()
    early_model = model_lib.ContactModalField()
    late_model.load_state_dict(baseline_model.state_dict())
    early_model.load_state_dict(baseline_model.state_dict())
    baseline_loss, baseline_components, baseline_gradients = _loss_and_gradients(
        baseline_model,
        object_features,
        contact_features,
        targets,
        target,
        prefix_bounded=True,
    )
    late_loss, late_components, late_gradients = _loss_and_gradients(
        late_model,
        object_features,
        contact_features,
        targets,
        late,
        prefix_bounded=True,
    )
    _, early_components, _ = _loss_and_gradients(
        early_model,
        object_features,
        contact_features,
        targets,
        early,
        prefix_bounded=True,
    )
    late_exact = (
        torch.equal(baseline_loss, late_loss)
        and all(
            torch.equal(baseline_components[key], late_components[key])
            for key in baseline_components
        )
        and all(
            torch.equal(baseline, changed)
            for baseline, changed in zip(
                baseline_gradients, late_gradients, strict=True
            )
        )
    )
    early_detected = not torch.equal(
        baseline_components["rendered_log_spectrum"],
        early_components["rendered_log_spectrum"],
    )

    invalid_rejections = {}
    prediction = baseline_model(object_features, contact_features)
    try:
        model_lib.synthetic_loss(prediction, *targets, target[..., :4095])
    except base.M0Error:
        invalid_rejections["short_target"] = True
    else:
        invalid_rejections["short_target"] = False
    invalid_prediction = model_lib.ModalPrediction(
        torch.full_like(prediction.frequencies_hz, float("nan")),
        prediction.decay_per_second,
        prediction.gains,
        prediction.residual_gains,
    )
    try:
        model_lib.synthetic_loss(invalid_prediction, *targets, target)
    except base.M0Error:
        invalid_rejections["non_finite_prediction"] = True
    else:
        invalid_rejections["non_finite_prediction"] = False
    try:
        model_lib.render_prediction(prediction, 0)
    except base.M0Error:
        invalid_rejections["non_positive_frames"] = True
    else:
        invalid_rejections["non_positive_frames"] = False

    all_exact = all(
        row["render_prefix_exact"]
        and row["loss_exact"]
        and row["components_exact"]
        and row["gradients_exact"]
        for row in cases
    )
    passed = (
        all_exact and late_exact and early_detected and all(invalid_rejections.values())
    )
    return {
        "schema": EQUIVALENCE_SCHEMA,
        "status": "Pass" if passed else "ImplementationConformanceReject",
        "protocol_sha256": contract.PROTOCOL_SHA256,
        "implementation_root_sha256": contract.implementation_root_sha256(),
        "full_frames": 144_000,
        "prefix_frames": model_lib.TRAINING_RENDER_FRAMES,
        "case_count": len(cases),
        "cases": cases,
        "late_target_mutation_exact": bool(late_exact),
        "early_target_mutation_detected": bool(early_detected),
        "invalid_rejections": invalid_rejections,
        "official_values_opened": False,
        "model_values_published": False,
    }


def resource_report() -> dict[str, Any]:
    started = time.perf_counter()
    synthetic, transfers, recordings, profile = _resource_fixture()
    saved_model_lib = inherited_train.model_lib
    results = []
    candidate_a_bytes: bytes | None = None
    candidate_b_exact = False
    try:
        inherited_train.model_lib = model_lib
        for index, variant in enumerate(RESOURCE_VARIANTS):
            result = inherited_train.train_variant(
                synthetic, transfers, recordings, profile, variant
            )
            if index == 0:
                candidate_a_bytes = base.canonical_tensor_bytes(
                    model_lib.model_tensors(result.model)
                )
            elif index == 1:
                candidate_b_bytes = base.canonical_tensor_bytes(
                    model_lib.model_tensors(result.model)
                )
                candidate_b_exact = candidate_a_bytes == candidate_b_bytes
            results.append(result)
    finally:
        inherited_train.model_lib = saved_model_lib
    elapsed = time.perf_counter() - started
    peak_rss_bytes = (
        process_resource.getrusage(process_resource.RUSAGE_SELF).ru_maxrss * 1024
    )
    completed_steps = sum(len(result.step_metrics) for result in results)
    expected_steps = len(RESOURCE_VARIANTS) * (
        profile.synthetic_steps + profile.real_steps
    )
    wall_pass = elapsed <= RESOURCE_WALL_LIMIT_SECONDS
    memory_pass = peak_rss_bytes < RESOURCE_RSS_LIMIT_BYTES
    passed = (
        completed_steps == expected_steps
        and candidate_b_exact
        and wall_pass
        and memory_pass
    )
    return {
        "schema": RESOURCE_SCHEMA,
        "status": "Pass" if passed else "ResourceReject",
        "protocol_sha256": contract.PROTOCOL_SHA256,
        "implementation_root_sha256": contract.implementation_root_sha256(),
        "fixture": {
            "profile": RESOURCE_PROFILE_ID,
            "seed": model_lib.SEED,
            "synthetic_examples": len(synthetic),
            "stored_target_frames": profile.transfer_window,
            "training_render_frames": model_lib.TRAINING_RENDER_FRAMES,
            "transfer_contexts": len(transfers),
            "recording_contexts": len(recordings),
            "variant_count": len(RESOURCE_VARIANTS),
            "synthetic_steps": len(RESOURCE_VARIANTS) * profile.synthetic_steps,
            "real_replay_steps": len(RESOURCE_VARIANTS) * profile.real_steps,
            "expected_optimizer_steps": expected_steps,
        },
        "completed_optimizer_steps": completed_steps,
        "candidate_ab_exact": bool(candidate_b_exact),
        "wall_seconds": elapsed,
        "wall_limit_seconds": RESOURCE_WALL_LIMIT_SECONDS,
        "wall_gate_pass": bool(wall_pass),
        "peak_rss_bytes": peak_rss_bytes,
        "peak_rss_limit_bytes": RESOURCE_RSS_LIMIT_BYTES,
        "memory_gate_pass": bool(memory_pass),
        "network_allowed": False,
        "official_values_opened": False,
        "model_values_published": False,
        "loss_values_published": False,
    }


def _publish(report: dict[str, Any], output_path: Path) -> None:
    base.external_directory(output_path, "M0c resource output", must_exist=False)
    staging, output = base.prepare_output(output_path)
    try:
        base.write_artifact(staging, "report.json", base.canonical_json(report))
        base.publish_output(staging, output)
    except Exception:
        base.abandon_output(staging)
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=("equivalence", "resource"), required=True)
    parser.add_argument("--output", required=True, type=Path)
    arguments = parser.parse_args()
    try:
        report = (
            equivalence_report()
            if arguments.mode == "equivalence"
            else resource_report()
        )
        _publish(report, arguments.output)
    except (base.M0Error, OSError, ValueError) as error:
        print(f"physical-sound-v28-m0c-resource: {error}", file=os.sys.stderr)
        return 2
    print(base.canonical_json(report).decode("utf-8"), end="")
    return 0 if report["status"] == "Pass" else 3


if __name__ == "__main__":
    raise SystemExit(main())
