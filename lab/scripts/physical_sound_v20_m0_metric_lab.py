#!/usr/bin/env python3
"""Deterministic development-only metric laboratory for Physical Sound V20 M0."""

from __future__ import annotations

import argparse
import json
import math
import resource
import sys
import time
from dataclasses import dataclass
from functools import cache
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_v19_f0_oracle as field_oracle
import physical_sound_v19_i0_oracle as i0_oracle
import physical_sound_v20_m0_common as common
from scipy.signal import hilbert
from scipy.stats import spearmanr

SAMPLE_RATE_HZ = 16_000
SAMPLE_COUNT = 8_192
MODE_COUNT = 8
CASE_COUNT = 192
CASE_SELECTION_COUNT = 8
BATCH_SIZE = 32
IDENTITY_TOLERANCE = 1.0e-12
SEPARATION_MARGIN = 0.90
SPEARMAN_MINIMUM = 0.90
DECAY_ACTIVE_EPSILON = 1.0e-12
STFT_RESOLUTIONS = (
    (256, 256, 64),
    (512, 512, 128),
    (1_024, 1_024, 256),
    (2_048, 2_048, 512),
)
DECAY_FREQUENCY_BANDS_HZ = ((0.0, 500.0), (500.0, 2_000.0), (2_000.0, 8_000.0))
DECAY_SAMPLE_INTERVALS = ((0, 256), (256, 1_024), (1_024, 4_096), (4_096, 8_192))
TRANSIENT_PREFIXES = (64, 256, 1_024)
ACOUSTIC_METRICS = ("mrsc", "mrlm", "decay_energy", "transient_energy")
SUMMARY_METRICS = (
    *ACOUSTIC_METRICS,
    "early_waveform_nrmse",
    "full_waveform_nrmse",
    "hilbert_envelope_nrmse",
    "frequency_cents_median",
    "frequency_cents_p95",
    "frequency_cents_max",
    "damping_relative_median",
    "damping_relative_p95",
    "gain_nrmse",
    "missing_mode_count",
)
OUTPUT_FILES = {
    "access-ledger.json",
    "corpus.json",
    "manifest.json",
    "metrics.jsonl",
    "report.json",
}

LEGACY_EXPECTED = {
    "combined": {
        "early_waveform_nrmse": 0.44468013433359327,
        "envelope_nrmse_p95": 0.47792052424213516,
        "full_waveform_nrmse": 0.6476855496667725,
        "spectrum_rmse_db_median": 2.3246230204176306,
    },
    "field_only": {
        "early_waveform_nrmse": 0.1809337431676388,
        "envelope_nrmse_p95": 0.23741730132208194,
        "full_waveform_nrmse": 0.17689206640365496,
        "spectrum_rmse_db_median": 1.3881230458592098,
    },
    "global_only": {
        "early_waveform_nrmse": 0.4139467233391932,
        "envelope_nrmse_p95": 0.4669793291861611,
        "full_waveform_nrmse": 0.632584459054322,
        "spectrum_rmse_db_median": 1.7299308450891062,
    },
}


@dataclass(frozen=True)
class ControlSpec:
    name: str
    family: str
    severity: float
    classification: str

    def record(self) -> dict[str, Any]:
        return {
            "classification": self.classification,
            "family": self.family,
            "name": self.name,
            "severity": self.severity,
        }


@dataclass(frozen=True)
class EvaluationCorpus:
    objects: tuple[common.i0_common.f0_common.FieldObject, ...]
    modes: tuple[common.i0_common.GlobalModes, ...]
    candidate: tuple[np.ndarray, ...]
    case_records: tuple[dict[str, Any], ...]
    case_object_index: np.ndarray
    query_vertices: np.ndarray
    truth_frequencies: np.ndarray
    truth_damping: np.ndarray
    truth_gains: np.ndarray
    predicted_frequencies: np.ndarray
    predicted_damping: np.ndarray
    predicted_gains: np.ndarray
    field_metrics: dict[str, list[dict[str, Any]]]


@dataclass(frozen=True)
class TruthCache:
    waveforms: np.ndarray
    rms: np.ndarray
    normalized: np.ndarray
    stft_magnitudes: dict[int, np.ndarray]
    decay_energy: np.ndarray
    decay_active: np.ndarray
    transient_fractions: np.ndarray
    envelope: np.ndarray


def _severity_token(value: float) -> str:
    if float(value).is_integer():
        return f"{int(value):03d}"
    return f"{value:.2f}".replace(".", "p")


def control_specs() -> tuple[ControlSpec, ...]:
    controls = [
        ControlSpec("identity", "base", 0.0, "acceptable"),
        ControlSpec("b0-only", "base", 0.0, "acceptable"),
        ControlSpec("f0-only", "base", 0.0, "acceptable"),
        ControlSpec("combined", "base", 0.0, "acceptable"),
        ControlSpec("global-polarity", "polarity", 1.0, "invariant"),
        ControlSpec("alternating-gain-sign", "gain-sign", 1.0, "harmful"),
    ]
    frequency = (0.0, 5.0, 10.0, 20.0, 40.0, 60.0, 90.0, 120.0)
    for family in ("frequency-uniform", "frequency-alternating"):
        for value in frequency:
            controls.append(
                ControlSpec(
                    f"{family}-{_severity_token(value)}-cents",
                    family,
                    value,
                    "acceptable" if value <= 20.0 else "harmful",
                )
            )
    damping = (0.0, 0.02, 0.05, 0.08, 0.12, 0.20, 0.35, 0.50)
    for family in ("damping-positive", "damping-negative"):
        for value in damping:
            controls.append(
                ControlSpec(
                    f"{family}-{_severity_token(value)}",
                    family,
                    value,
                    "acceptable" if value <= 0.08 else "harmful",
                )
            )
    for value in (0.0, 0.05, 0.10, 0.20, 0.25, 0.40, 0.60):
        controls.append(
            ControlSpec(
                f"gain-scale-{_severity_token(value)}",
                "gain-scale",
                value,
                "acceptable" if value <= 0.25 else "harmful",
            )
        )
    for value in (0.0, 1.0, 2.0, 4.0):
        controls.append(
            ControlSpec(
                f"mode-removal-{int(value)}",
                "mode-removal",
                value,
                "acceptable" if value == 0.0 else "harmful",
            )
        )
    for value in (0.0, 16.0, 32.0, 64.0, 128.0):
        classification = "acceptable" if value == 0.0 else "diagnostic"
        if value >= 64.0:
            classification = "harmful"
        controls.append(
            ControlSpec(
                f"onset-delay-{int(value)}-samples",
                "onset-delay",
                value,
                classification,
            )
        )
    for value in (0.0, 0.5, 2.0, 8.0):
        classification = "acceptable" if value == 0.0 else "diagnostic"
        if value == 8.0:
            classification = "harmful"
        controls.append(
            ControlSpec(
                f"sample-zero-impulse-{_severity_token(value)}",
                "sample-zero-impulse",
                value,
                classification,
            )
        )
    names = [item.name for item in controls]
    if len(controls) != 58 or len(set(names)) != len(names):
        raise common.M0Error("M0 control matrix changed")
    return tuple(controls)


def selected_queries(accepted: np.ndarray) -> np.ndarray:
    values = np.asarray(accepted, dtype=np.int64)
    if values.ndim != 1 or values.size < CASE_SELECTION_COUNT:
        raise common.M0Error("M0 view has fewer than eight accepted queries")
    positions = np.asarray(
        [math.floor(index * (values.size - 1) / 7) for index in range(8)],
        dtype=np.int64,
    )
    if np.unique(positions).size != CASE_SELECTION_COUNT:
        raise common.M0Error("M0 query selection produced duplicate positions")
    selected = values[positions]
    if np.unique(selected).size != CASE_SELECTION_COUNT:
        raise common.M0Error("M0 query selection produced duplicate vertices")
    return selected


def build_corpus(
    dependencies: common.i0_common.LoadedDependencies,
) -> EvaluationCorpus:
    objects = common.development_objects()
    modes = tuple(
        common.i0_common.global_modes(dependencies.b0, item.row) for item in objects
    )
    predictions, field_metrics, _decisions = field_oracle._prediction_evaluation(
        dependencies.f0_model, dependencies.gain_scale, objects
    )
    candidate = predictions["candidate"]
    case_records: list[dict[str, Any]] = []
    object_indices: list[int] = []
    vertices: list[int] = []
    truth_frequencies: list[np.ndarray] = []
    truth_damping: list[np.ndarray] = []
    truth_gains: list[np.ndarray] = []
    predicted_frequencies: list[np.ndarray] = []
    predicted_damping: list[np.ndarray] = []
    predicted_gains: list[np.ndarray] = []
    for object_index, (item, mode, field) in enumerate(
        zip(objects, modes, candidate, strict=True)
    ):
        selected = selected_queries(item.accepted_query)
        for query_vertex in selected:
            query = int(query_vertex)
            case_index = len(case_records)
            case_records.append(
                {
                    "case_index": case_index,
                    "is_twin": item.is_twin,
                    "object_id": item.row.object_id,
                    "physical_group_id": item.row.physical_group_id,
                    "query_vertex": query,
                }
            )
            object_indices.append(object_index)
            vertices.append(query)
            truth_frequencies.append(mode.truth_frequencies)
            truth_damping.append(mode.truth_damping)
            truth_gains.append(item.gains[query])
            predicted_frequencies.append(mode.predicted_frequencies)
            predicted_damping.append(mode.predicted_damping)
            predicted_gains.append(field[query])
    if len(case_records) != CASE_COUNT:
        raise common.M0Error(f"M0 waveform case count changed: {len(case_records)}")
    return EvaluationCorpus(
        objects=objects,
        modes=modes,
        candidate=candidate,
        case_records=tuple(case_records),
        case_object_index=np.asarray(object_indices, dtype=np.int64),
        query_vertices=np.asarray(vertices, dtype=np.int64),
        truth_frequencies=np.stack(truth_frequencies),
        truth_damping=np.stack(truth_damping),
        truth_gains=np.stack(truth_gains),
        predicted_frequencies=np.stack(predicted_frequencies),
        predicted_damping=np.stack(predicted_damping),
        predicted_gains=np.stack(predicted_gains),
        field_metrics=field_metrics,
    )


def _render_modal_batch(
    frequencies: np.ndarray, damping: np.ndarray, gains: np.ndarray
) -> np.ndarray:
    frequencies = np.asarray(frequencies, dtype=np.float64)
    damping = np.asarray(damping, dtype=np.float64)
    gains = np.asarray(gains, dtype=np.float64)
    if (
        frequencies.ndim != 2
        or frequencies.shape != damping.shape
        or frequencies.shape != gains.shape
        or frequencies.shape[1] != MODE_COUNT
    ):
        raise common.M0Error("M0 modal batch shape changed")
    time_axis = np.arange(SAMPLE_COUNT, dtype=np.float64) / SAMPLE_RATE_HZ
    basis = np.exp(-damping[:, :, None] * time_axis[None, None, :]) * np.sin(
        2.0 * np.pi * frequencies[:, :, None] * time_axis[None, None, :]
    )
    waveforms = np.sum(gains[:, :, None] * basis, axis=1)
    if waveforms.shape != (frequencies.shape[0], SAMPLE_COUNT):
        raise common.M0Error("M0 modal renderer shape changed")
    if not np.isfinite(waveforms).all():
        raise common.M0Error("M0 modal renderer produced a non-finite value")
    return waveforms


@cache
def _stft_plan(
    window_length: int, hop: int
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    starts = np.arange(0, SAMPLE_COUNT, hop, dtype=np.int64)
    sample_index = starts[:, None] + np.arange(window_length, dtype=np.int64)[None, :]
    index = np.arange(window_length, dtype=np.float64)
    window = 0.5 - 0.5 * np.cos(2.0 * np.pi * index / window_length)
    return starts, sample_index, window


def _stft_magnitude(waveforms: np.ndarray, window_length: int, hop: int) -> np.ndarray:
    values = np.asarray(waveforms, dtype=np.float64)
    if values.ndim != 2 or values.shape[1] != SAMPLE_COUNT:
        raise common.M0Error("M0 STFT waveform shape changed")
    _starts, sample_index, window = _stft_plan(window_length, hop)
    pad = int(sample_index[-1, -1] + 1 - SAMPLE_COUNT)
    padded = np.pad(values, ((0, 0), (0, pad)), mode="constant")
    frames = padded[:, sample_index] * window[None, None, :]
    magnitude = np.abs(np.fft.rfft(frames, n=window_length, axis=2))
    if not np.isfinite(magnitude).all():
        raise common.M0Error("M0 STFT produced a non-finite value")
    return magnitude


def _decay_cells(magnitude: np.ndarray) -> np.ndarray:
    starts, _sample_index, _window = _stft_plan(512, 128)
    frequencies = np.fft.rfftfreq(512, d=1.0 / SAMPLE_RATE_HZ)
    cells = []
    for time_start, time_end in DECAY_SAMPLE_INTERVALS:
        time_mask = (starts >= time_start) & (starts < time_end)
        for band_index, (frequency_start, frequency_end) in enumerate(
            DECAY_FREQUENCY_BANDS_HZ
        ):
            if band_index == len(DECAY_FREQUENCY_BANDS_HZ) - 1:
                frequency_mask = (frequencies >= frequency_start) & (
                    frequencies <= frequency_end
                )
            else:
                frequency_mask = (frequencies >= frequency_start) & (
                    frequencies < frequency_end
                )
            cells.append(
                np.sum(magnitude[:, time_mask][:, :, frequency_mask] ** 2, axis=(1, 2))
            )
    result = np.stack(cells, axis=1)
    if result.shape[1] != 12:
        raise common.M0Error("M0 decay cell count changed")
    return result


def _transient_fractions(waveforms: np.ndarray) -> np.ndarray:
    total = np.sum(waveforms**2, axis=1)
    if np.any(total <= 1.0e-12):
        raise common.M0Error("M0 transient metric has a degenerate complete clip")
    return np.stack(
        [
            np.sum(waveforms[:, :prefix] ** 2, axis=1) / total
            for prefix in TRANSIENT_PREFIXES
        ],
        axis=1,
    )


def build_truth_cache(corpus: EvaluationCorpus) -> TruthCache:
    waveforms = _render_modal_batch(
        corpus.truth_frequencies, corpus.truth_damping, corpus.truth_gains
    )
    rms = np.sqrt(np.mean(waveforms**2, axis=1))
    if np.any(rms <= 1.0e-12):
        raise common.M0Error("M0 truth RMS normalization is degenerate")
    normalized = waveforms / rms[:, None]
    stft = {
        fft_length: _stft_magnitude(normalized, window_length, hop)
        for fft_length, window_length, hop in STFT_RESOLUTIONS
    }
    decay_energy = _decay_cells(stft[512])
    decay_active = decay_energy > DECAY_ACTIVE_EPSILON
    if np.any(np.sum(decay_active, axis=1) == 0):
        raise common.M0Error("M0 truth has no active decay cell")
    envelope = np.abs(hilbert(normalized, axis=1))
    return TruthCache(
        waveforms=waveforms,
        rms=rms,
        normalized=normalized,
        stft_magnitudes=stft,
        decay_energy=decay_energy,
        decay_active=decay_active,
        transient_fractions=_transient_fractions(normalized),
        envelope=envelope,
    )


def _control_parameters(
    corpus: EvaluationCorpus, spec: ControlSpec, start: int, end: int
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    frequencies = corpus.truth_frequencies[start:end].copy()
    damping = corpus.truth_damping[start:end].copy()
    gains = corpus.truth_gains[start:end].copy()
    if spec.name in ("b0-only", "combined"):
        frequencies = corpus.predicted_frequencies[start:end].copy()
        damping = corpus.predicted_damping[start:end].copy()
    if spec.name in ("f0-only", "combined"):
        gains = corpus.predicted_gains[start:end].copy()
    if spec.family == "polarity":
        gains *= -1.0
    elif spec.family == "gain-sign":
        gains[:, 1::2] *= -1.0
    elif spec.family == "frequency-uniform":
        frequencies *= 2.0 ** (spec.severity / 1_200.0)
    elif spec.family == "frequency-alternating":
        offsets = np.where(
            np.arange(MODE_COUNT) % 2 == 0, -spec.severity, spec.severity
        )
        frequencies *= 2.0 ** (offsets[None, :] / 1_200.0)
    elif spec.family == "damping-positive":
        damping *= 1.0 + spec.severity
    elif spec.family == "damping-negative":
        damping *= 1.0 - spec.severity
    elif spec.family == "gain-scale":
        gains *= 1.0 + spec.severity
    elif spec.family == "mode-removal" and spec.severity > 0.0:
        remove_count = int(spec.severity)
        for row in range(gains.shape[0]):
            indices = np.argsort(-np.abs(gains[row]), kind="stable")[:remove_count]
            gains[row, indices] = 0.0
    return frequencies, damping, gains


def _render_control(
    corpus: EvaluationCorpus,
    truth: TruthCache,
    spec: ControlSpec,
    start: int,
    end: int,
) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    frequencies, damping, gains = _control_parameters(corpus, spec, start, end)
    if spec.name == "identity" or (
        spec.family
        in (
            "frequency-uniform",
            "frequency-alternating",
            "damping-positive",
            "damping-negative",
            "gain-scale",
            "mode-removal",
            "onset-delay",
            "sample-zero-impulse",
        )
        and spec.severity == 0.0
    ):
        waveform = truth.waveforms[start:end].copy()
    elif spec.family == "polarity":
        waveform = -truth.waveforms[start:end]
    else:
        waveform = _render_modal_batch(frequencies, damping, gains)
    if spec.family == "onset-delay" and spec.severity > 0.0:
        delay = int(spec.severity)
        shifted = np.zeros_like(waveform)
        shifted[:, delay:] = waveform[:, :-delay]
        waveform = shifted
    elif spec.family == "sample-zero-impulse" and spec.severity > 0.0:
        waveform = waveform.copy()
        peak = np.max(np.abs(truth.waveforms[start:end]), axis=1)
        waveform[:, 0] += spec.severity * peak
    if not np.isfinite(waveform).all():
        raise common.M0Error(f"M0 control {spec.name} produced non-finite waveform")
    return waveform, frequencies, damping, gains


def _acoustic_batch(
    prediction: np.ndarray, truth: TruthCache, start: int, end: int
) -> dict[str, np.ndarray]:
    normalized = prediction / truth.rms[start:end, None]
    mrsc_rows = []
    mrlm_rows = []
    predicted_512 = None
    for fft_length, window_length, hop in STFT_RESOLUTIONS:
        predicted = _stft_magnitude(normalized, window_length, hop)
        expected = truth.stft_magnitudes[fft_length][start:end]
        delta = predicted - expected
        denominator = np.maximum(np.sqrt(np.sum(expected**2, axis=(1, 2))), 1.0e-12)
        mrsc_rows.append(np.sqrt(np.sum(delta**2, axis=(1, 2))) / denominator)
        mrlm_rows.append(
            np.mean(
                np.abs(np.log(predicted + 1.0e-7) - np.log(expected + 1.0e-7)),
                axis=(1, 2),
            )
        )
        if fft_length == 512:
            predicted_512 = predicted
    if predicted_512 is None:
        raise common.M0Error("M0 decay STFT resolution is absent")
    predicted_decay = _decay_cells(predicted_512)
    active = truth.decay_active[start:end]
    decay_delta = np.log(predicted_decay + 1.0e-10) - np.log(
        truth.decay_energy[start:end] + 1.0e-10
    )
    decay_energy = np.sqrt(
        np.sum(np.where(active, decay_delta**2, 0.0), axis=1) / np.sum(active, axis=1)
    )
    transient = _transient_fractions(normalized)
    transient_energy = np.sqrt(
        np.mean((transient - truth.transient_fractions[start:end]) ** 2, axis=1)
    )
    truth_normalized = truth.normalized[start:end]
    early_denominator = np.sqrt(np.mean(truth_normalized[:, :1_024] ** 2, axis=1))
    full_denominator = np.sqrt(np.mean(truth_normalized**2, axis=1))
    predicted_envelope = np.abs(hilbert(normalized, axis=1))
    envelope_denominator = np.sqrt(np.mean(truth.envelope[start:end] ** 2, axis=1))
    result = {
        "decay_active_cells": np.sum(active, axis=1),
        "decay_energy": decay_energy,
        "decay_omitted_cells": 12 - np.sum(active, axis=1),
        "early_waveform_nrmse": np.sqrt(
            np.mean((normalized[:, :1_024] - truth_normalized[:, :1_024]) ** 2, axis=1)
        )
        / early_denominator,
        "full_waveform_nrmse": np.sqrt(
            np.mean((normalized - truth_normalized) ** 2, axis=1)
        )
        / full_denominator,
        "hilbert_envelope_nrmse": np.sqrt(
            np.mean((predicted_envelope - truth.envelope[start:end]) ** 2, axis=1)
        )
        / envelope_denominator,
        "mrlm": np.mean(np.stack(mrlm_rows, axis=1), axis=1),
        "mrsc": np.mean(np.stack(mrsc_rows, axis=1), axis=1),
        "transient_energy": transient_energy,
    }
    if not all(np.isfinite(value).all() for value in result.values()):
        raise common.M0Error("M0 acoustic metric produced a non-finite value")
    return result


def _physical_batch(
    corpus: EvaluationCorpus,
    frequencies: np.ndarray,
    damping: np.ndarray,
    gains: np.ndarray,
    start: int,
    end: int,
) -> dict[str, np.ndarray]:
    frequency = np.abs(
        1_200.0 * np.log2(frequencies / corpus.truth_frequencies[start:end])
    )
    damping_error = (
        np.abs(damping - corpus.truth_damping[start:end])
        / corpus.truth_damping[start:end]
    )
    truth_gain = corpus.truth_gains[start:end]
    gain_denominator = np.sqrt(np.mean(truth_gain**2, axis=1))
    missing = np.sum((np.abs(truth_gain) > 1.0e-12) & (gains == 0.0), axis=1)
    result = {
        "damping_relative_median": np.median(damping_error, axis=1),
        "damping_relative_p95": np.quantile(damping_error, 0.95, axis=1),
        "frequency_cents_max": np.max(frequency, axis=1),
        "frequency_cents_median": np.median(frequency, axis=1),
        "frequency_cents_p95": np.quantile(frequency, 0.95, axis=1),
        "gain_nrmse": np.sqrt(np.mean((gains - truth_gain) ** 2, axis=1))
        / gain_denominator,
        "missing_mode_count": missing,
    }
    if not all(np.isfinite(value).all() for value in result.values()):
        raise common.M0Error("M0 physical metric produced a non-finite value")
    return result


def evaluate_controls(
    corpus: EvaluationCorpus, truth: TruthCache, controls: tuple[ControlSpec, ...]
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for spec in controls:
        for start in range(0, CASE_COUNT, BATCH_SIZE):
            end = min(start + BATCH_SIZE, CASE_COUNT)
            waveform, frequencies, damping, gains = _render_control(
                corpus, truth, spec, start, end
            )
            acoustic = _acoustic_batch(waveform, truth, start, end)
            physical = _physical_batch(corpus, frequencies, damping, gains, start, end)
            for local_index, case_index in enumerate(range(start, end)):
                rows.append(
                    {
                        **corpus.case_records[case_index],
                        "classification": spec.classification,
                        "control": spec.name,
                        "control_family": spec.family,
                        "control_severity": spec.severity,
                        **{
                            name: int(value[local_index])
                            if name
                            in (
                                "decay_active_cells",
                                "decay_omitted_cells",
                                "missing_mode_count",
                            )
                            else float(value[local_index])
                            for name, value in {**acoustic, **physical}.items()
                        },
                        "schema": common.METRIC_SCHEMA,
                    }
                )
    if len(rows) != CASE_COUNT * len(controls):
        raise common.M0Error("M0 metric cell count changed")
    return rows


def _summaries(
    rows: list[dict[str, Any]], controls: tuple[ControlSpec, ...]
) -> dict[str, dict[str, Any]]:
    by_control = {item.name: [] for item in controls}
    for row in rows:
        by_control[row["control"]].append(row)
    result: dict[str, dict[str, Any]] = {}
    for spec in controls:
        values = by_control[spec.name]
        if len(values) != CASE_COUNT:
            raise common.M0Error(f"M0 control {spec.name} is incomplete")
        metrics = {}
        for name in SUMMARY_METRICS:
            array = np.asarray([row[name] for row in values], dtype=np.float64)
            metrics[name] = {
                "max": float(np.max(array)),
                "mean": float(np.mean(array)),
                "median": float(np.median(array)),
                "min": float(np.min(array)),
                "p95": float(np.quantile(array, 0.95)),
            }
        result[spec.name] = {
            **spec.record(),
            "case_count": len(values),
            "decay_active_cell_count": int(
                sum(row["decay_active_cells"] for row in values)
            ),
            "decay_omitted_cell_count": int(
                sum(row["decay_omitted_cells"] for row in values)
            ),
            "metrics": metrics,
        }
    return result


def _family_specs(
    controls: tuple[ControlSpec, ...], family: str, classification: str | None = None
) -> list[ControlSpec]:
    return [
        item
        for item in controls
        if item.family == family
        and (classification is None or item.classification == classification)
    ]


def _separation_records(
    summaries: dict[str, dict[str, Any]], controls: tuple[ControlSpec, ...]
) -> dict[str, dict[str, Any]]:
    base = ["identity", "b0-only", "f0-only", "combined"]
    assignments = (
        ("frequency-uniform", "mrsc"),
        ("frequency-uniform", "mrlm"),
        ("frequency-alternating", "mrsc"),
        ("frequency-alternating", "mrlm"),
        ("damping-positive", "decay_energy"),
        ("damping-negative", "decay_energy"),
        ("mode-removal", "mrsc"),
        ("mode-removal", "mrlm"),
        ("onset-delay", "transient_energy"),
        ("sample-zero-impulse", "transient_energy"),
    )
    result = {}
    for family, metric in assignments:
        acceptable_names = base + [
            item.name for item in _family_specs(controls, family, "acceptable")
        ]
        harmful_names = [
            item.name for item in _family_specs(controls, family, "harmful")
        ]
        acceptable_values = [
            summaries[name]["metrics"][metric]["p95"] for name in acceptable_names
        ]
        harmful_values = [
            summaries[name]["metrics"][metric]["p95"] for name in harmful_names
        ]
        if not harmful_values:
            raise common.M0Error(
                f"M0 separation family has no harmful control: {family}"
            )
        maximum_acceptable = max(acceptable_values)
        minimum_harmful = min(harmful_values)
        ratio = (
            math.inf
            if minimum_harmful <= 0.0 and maximum_acceptable > 0.0
            else maximum_acceptable / max(minimum_harmful, 1.0e-300)
        )
        key = f"{family}:{metric}"
        result[key] = {
            "acceptable_controls": sorted(set(acceptable_names)),
            "aggregate": "corpus_p95",
            "family": family,
            "harmful_controls": sorted(harmful_names),
            "margin": SEPARATION_MARGIN,
            "maximum_acceptable": maximum_acceptable,
            "metric": metric,
            "minimum_harmful": minimum_harmful,
            "pass": bool(maximum_acceptable <= SEPARATION_MARGIN * minimum_harmful),
            "ratio": ratio,
        }
    return result


def _severity_correlations(
    summaries: dict[str, dict[str, Any]], controls: tuple[ControlSpec, ...]
) -> dict[str, dict[str, Any]]:
    assignments = (
        ("frequency-uniform", "mrsc"),
        ("frequency-uniform", "mrlm"),
        ("frequency-alternating", "mrsc"),
        ("frequency-alternating", "mrlm"),
        ("damping-positive", "decay_energy"),
        ("damping-negative", "decay_energy"),
        ("gain-scale", "gain_nrmse"),
        ("mode-removal", "mrsc"),
        ("mode-removal", "mrlm"),
        ("onset-delay", "transient_energy"),
        ("sample-zero-impulse", "transient_energy"),
    )
    result = {}
    for family, metric in assignments:
        ladder = _family_specs(controls, family)
        severity = np.asarray([item.severity for item in ladder], dtype=np.float64)
        values = np.asarray(
            [summaries[item.name]["metrics"][metric]["p95"] for item in ladder],
            dtype=np.float64,
        )
        correlation = float(spearmanr(severity, values).statistic)
        if not math.isfinite(correlation):
            raise common.M0Error(f"M0 severity correlation is non-finite: {family}")
        key = f"{family}:{metric}"
        result[key] = {
            "aggregate": "corpus_p95",
            "correlation": correlation,
            "family": family,
            "metric": metric,
            "minimum": SPEARMAN_MINIMUM,
            "pass": correlation >= SPEARMAN_MINIMUM,
        }
    return result


def _component_evidence(
    dependencies: common.i0_common.LoadedDependencies, corpus: EvaluationCorpus
) -> dict[str, Any]:
    remesh = field_oracle._remesh_metrics(
        corpus.objects, corpus.candidate, corpus.field_metrics["candidate"]
    )
    mutations = field_oracle._mutation_metrics(
        dependencies.f0_model,
        dependencies.gain_scale,
        corpus.objects,
        corpus.candidate,
    )
    structural = field_oracle._structural_metrics(corpus.objects)
    field_gates = field_oracle._gates(
        corpus.objects,
        corpus.field_metrics,
        remesh,
        mutations,
        structural,
        True,
    )
    global_metrics = i0_oracle._global_metrics(corpus.modes)
    fallback = i0_oracle._fallback_records(corpus.objects)
    rejected = {
        (item.row.object_id, int(vertex))
        for item in corpus.objects
        for vertex in item.rejected_query
    }
    fallback_complete = i0_oracle._fallback_complete(rejected, fallback)
    hard_mutations = i0_oracle._hard_mutations(corpus.objects, corpus.modes, fallback)
    global_gates = {
        "damping": global_metrics["damping_relative_median"] <= 0.08
        and global_metrics["damping_relative_p95"] <= 0.20,
        "frequency": global_metrics["frequency_cents_median"] <= 20.0
        and global_metrics["frequency_cents_p95"] <= 60.0,
        "modal_peak": global_metrics["frequency_cents_max"] <= 60.0,
    }
    return {
        "dependency_identity": i0_oracle._dependency_claim_valid(
            common.B0_TREE_DIGEST, common.F0_TREE_DIGEST
        ),
        "fallback_complete": fallback_complete,
        "field_gates": field_gates,
        "field_metrics": {
            name: field_oracle._metric_summary(rows)
            for name, rows in corpus.field_metrics.items()
        },
        "global_gates": global_gates,
        "global_metrics": global_metrics,
        "hard_mutations": hard_mutations,
        "pass": bool(
            all(field_gates.values())
            and all(global_gates.values())
            and fallback_complete
            and all(value == 12 for value in hard_mutations.values())
        ),
    }


def _legacy_attribution(corpus: EvaluationCorpus) -> dict[str, Any]:
    combined, _ = i0_oracle._waveform_metrics(
        corpus.objects, corpus.modes, corpus.candidate
    )
    truth_modes = tuple(
        common.i0_common.GlobalModes(
            value.truth_frequencies,
            value.truth_damping,
            value.truth_frequencies,
            value.truth_damping,
        )
        for value in corpus.modes
    )
    field_only, _ = i0_oracle._waveform_metrics(
        corpus.objects, truth_modes, corpus.candidate
    )
    global_only, _ = i0_oracle._waveform_metrics(
        corpus.objects,
        corpus.modes,
        tuple(item.gains for item in corpus.objects),
    )
    observed = {
        "combined": combined,
        "field_only": field_only,
        "global_only": global_only,
    }
    maximum_delta = max(
        abs(observed[group][metric] - LEGACY_EXPECTED[group][metric])
        for group in LEGACY_EXPECTED
        for metric in LEGACY_EXPECTED[group]
    )
    causal_ordering = bool(
        field_only["early_waveform_nrmse"] < global_only["early_waveform_nrmse"]
        and combined["early_waveform_nrmse"] > 0.35
        and combined["envelope_nrmse_p95"] > 0.15
        and combined["spectrum_rmse_db_median"] <= 2.5
    )
    return {
        "causal_ordering_reproduced": causal_ordering,
        "expected": LEGACY_EXPECTED,
        "maximum_absolute_delta": maximum_delta,
        "observed": observed,
        "pass": maximum_delta <= IDENTITY_TOLERANCE and causal_ordering,
    }


def _physical_control_gates(summaries: dict[str, dict[str, Any]]) -> dict[str, bool]:
    def frequency_pass(name: str) -> bool:
        metrics = summaries[name]["metrics"]
        return bool(
            metrics["frequency_cents_median"]["median"] <= 20.0
            and metrics["frequency_cents_p95"]["p95"] <= 60.0
            and metrics["frequency_cents_max"]["max"] <= 60.0
        )

    def damping_pass(name: str) -> bool:
        metrics = summaries[name]["metrics"]
        return bool(
            metrics["damping_relative_median"]["median"] <= 0.08
            and metrics["damping_relative_p95"]["p95"] <= 0.20
        )

    def gain_pass(name: str) -> bool:
        value = summaries[name]["metrics"]["gain_nrmse"]
        return bool(value["mean"] <= 0.25 and value["max"] <= 0.40)

    return {
        "alternating_gain_sign_rejects": not gain_pass("alternating-gain-sign"),
        "damping_negative_035_rejects": not damping_pass("damping-negative-0p35"),
        "damping_positive_035_rejects": not damping_pass("damping-positive-0p35"),
        "frequency_alternating_090_rejects": not frequency_pass(
            "frequency-alternating-090-cents"
        ),
        "frequency_uniform_090_rejects": not frequency_pass(
            "frequency-uniform-090-cents"
        ),
        "mode_removal_two_rejects": bool(
            summaries["mode-removal-2"]["metrics"]["missing_mode_count"]["min"] >= 2.0
            or not gain_pass("mode-removal-2")
        ),
    }


def _finite(value: Any) -> bool:
    if value is None or isinstance(value, (str, bool, int)):
        return True
    if isinstance(value, float):
        return math.isfinite(value)
    if isinstance(value, dict):
        return all(_finite(item) for item in value.values())
    if isinstance(value, (list, tuple)):
        return all(_finite(item) for item in value)
    return False


def _metric_roundtrip_exact(rows: list[dict[str, Any]]) -> bool:
    payload = common.canonical_json_lines(rows)
    decoded = [json.loads(line) for line in payload.decode("utf-8").splitlines()]
    return decoded == rows


def _gates(
    rows: list[dict[str, Any]],
    summaries: dict[str, dict[str, Any]],
    controls: tuple[ControlSpec, ...],
    separation: dict[str, dict[str, Any]],
    correlations: dict[str, dict[str, Any]],
    components: dict[str, Any],
    legacy: dict[str, Any],
    roundtrip_exact: bool,
) -> tuple[dict[str, bool], dict[str, bool]]:
    identity = summaries["identity"]["metrics"]
    polarity = summaries["global-polarity"]["metrics"]
    physical = _physical_control_gates(summaries)
    expected_cells = CASE_COUNT * len(controls)
    gates = {
        "component_hard_physical": bool(components["pass"]),
        "complete_finite": bool(
            len(rows) == expected_cells
            and len(summaries) == len(controls)
            and all(value["case_count"] == CASE_COUNT for value in summaries.values())
            and _finite(rows)
            and _finite(summaries)
        ),
        "harmful_physical_owners": all(physical.values()),
        "identity_zero": all(
            identity[name]["max"] <= IDENTITY_TOLERANCE for name in ACOUSTIC_METRICS
        ),
        "legacy_attribution": bool(legacy["pass"]),
        "metric_separation": all(value["pass"] for value in separation.values()),
        "polarity_invariance": bool(
            all(
                polarity[name]["max"] <= IDENTITY_TOLERANCE for name in ACOUSTIC_METRICS
            )
            and polarity["full_waveform_nrmse"]["min"] >= 1.9
        ),
        "resource_ceiling_enforced": True,
        "serialization_roundtrip": roundtrip_exact,
        "severity_monotonicity": all(value["pass"] for value in correlations.values()),
        "zero_sealed_access": all(value == 0 for value in common.ZERO_ACCESS.values()),
    }
    return gates, physical


def _corpus_record(corpus: EvaluationCorpus) -> dict[str, Any]:
    cases_by_object: dict[str, list[int]] = {}
    for row in corpus.case_records:
        cases_by_object.setdefault(row["object_id"], []).append(row["query_vertex"])
    objects = []
    for item, mode, prediction in zip(
        corpus.objects, corpus.modes, corpus.candidate, strict=True
    ):
        selected = cases_by_object[item.row.object_id]
        objects.append(
            {
                **item.record(),
                "predicted_damping_hash": common.identity_hash(mode.predicted_damping),
                "predicted_frequency_hash": common.identity_hash(
                    mode.predicted_frequencies
                ),
                "predicted_gain_hash": common.identity_hash(prediction),
                "selected_query_count": len(selected),
                "selected_query_hash": common.identity_hash(
                    np.asarray(selected, dtype=np.int64)
                ),
                "selected_query_vertices": selected,
                "truth_damping_hash": common.identity_hash(mode.truth_damping),
                "truth_frequency_hash": common.identity_hash(mode.truth_frequencies),
            }
        )
    return {
        "b0_tree_digest": common.B0_TREE_DIGEST,
        "case_count": len(corpus.case_records),
        "case_root": common.sha256_bytes(common.canonical_json(corpus.case_records)),
        "development_record_root": common.DEVELOPMENT_RECORD_ROOT,
        "development_row_root": common.DEVELOPMENT_ROW_ROOT,
        "f0_tree_digest": common.F0_TREE_DIGEST,
        "objects": objects,
        "schema": common.CORPUS_SCHEMA,
        "study_id": common.STUDY_ID,
        "view_count": len(corpus.objects),
    }


def run(b0_root: Path, f0_root: Path, output: Path) -> dict[str, Any]:
    started = time.monotonic()
    environment = common.verify_protocol_environment()
    implementation_commit = common.verify_committed_implementation()
    implementation = common.implementation_hashes()
    dependencies = common.load_dependencies(b0_root, f0_root)
    staging, target = common.prepare_output(output)
    try:
        corpus = build_corpus(dependencies)
        components = _component_evidence(dependencies, corpus)
        legacy = _legacy_attribution(corpus)
        controls = control_specs()
        truth = build_truth_cache(corpus)
        rows = evaluate_controls(corpus, truth, controls)
        summaries = _summaries(rows, controls)
        separation = _separation_records(summaries, controls)
        correlations = _severity_correlations(summaries, controls)
        roundtrip_exact = _metric_roundtrip_exact(rows)
        gates, physical_controls = _gates(
            rows,
            summaries,
            controls,
            separation,
            correlations,
            components,
            legacy,
            roundtrip_exact,
        )
        report = {
            "component_evidence": components,
            "control_summaries": summaries,
            "gates": gates,
            "legacy_attribution": legacy,
            "metric_contract": {
                "decay_active_epsilon": DECAY_ACTIVE_EPSILON,
                "separation_aggregate": "corpus_p95",
                "separation_margin": SEPARATION_MARGIN,
                "severity_aggregate": "corpus_p95",
                "severity_spearman_minimum": SPEARMAN_MINIMUM,
            },
            "physical_control_gates": physical_controls,
            "schema": common.REPORT_SCHEMA,
            "separation": separation,
            "severity_correlations": correlations,
            "single_run_pass": all(gates.values()),
            "study_id": common.STUDY_ID,
        }
        access = {
            **common.ZERO_ACCESS,
            "b0_artifact_bytes_read": dependencies.b0_bytes_read,
            "development_control_count": len(controls),
            "development_views_generated": len(corpus.objects),
            "development_waveform_cases_evaluated": len(corpus.case_records),
            "f0_artifact_bytes_read": dependencies.f0_bytes_read,
            "schema": common.ACCESS_SCHEMA,
        }
        files = {
            "access-ledger.json": common.canonical_json(access),
            "corpus.json": common.canonical_json(_corpus_record(corpus)),
            "metrics.jsonl": common.canonical_json_lines(rows),
            "report.json": common.canonical_json(report),
        }
        artifact_hashes = {
            name: common.sha256_bytes(payload)
            for name, payload in sorted(files.items())
        }
        manifest = {
            "artifact_hashes": artifact_hashes,
            "b0_tree_digest": common.B0_TREE_DIGEST,
            "development_row_root": common.DEVELOPMENT_ROW_ROOT,
            "environment": environment,
            "f0_tree_digest": common.F0_TREE_DIGEST,
            "i0_implementation_commit": common.I0_IMPLEMENTATION_COMMIT,
            "i0_implementation_hashes": common.I0_IMPLEMENTATION_HASHES,
            "implementation_commit": implementation_commit,
            "implementation_hashes": implementation,
            "protocol_sha256": common.PROTOCOL_SHA256,
            "revision": common.REVISION,
            "schema": common.MANIFEST_SCHEMA,
            "study_id": common.STUDY_ID,
        }
        files["manifest.json"] = common.canonical_json(manifest)
        if set(files) != OUTPUT_FILES:
            raise common.M0Error(f"M0 output file set changed: {sorted(files)}")
        total_bytes = sum(len(payload) for payload in files.values())
        if total_bytes > 100 * 1024 * 1024:
            raise common.M0Error("M0 output byte ceiling exceeded")
        if time.monotonic() - started > 600.0:
            raise common.M0Error("M0 runtime ceiling exceeded")
        if resource.getrusage(resource.RUSAGE_SELF).ru_maxrss > 4 * 1024 * 1024:
            raise common.M0Error("M0 RSS ceiling exceeded")
        common.write_files(staging, files)
        if time.monotonic() - started > 600.0:
            raise common.M0Error("M0 runtime ceiling exceeded after serialization")
        common.publish_output(staging, target)
        return {
            "file_count": len(files),
            "output": str(target),
            "passed": all(gates.values()),
            "report_sha256": artifact_hashes["report.json"],
            "tree_digest": common.tree_digest(common.directory_file_map(target)),
        }
    except Exception:
        common.abandon_output(staging)
        raise


def compare(left: Path, right: Path) -> dict[str, Any]:
    result = common.compare_directories(left, right)
    if result["file_count"] != len(OUTPUT_FILES):
        raise common.M0Error("M0 compared output file count changed")
    return result


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("--b0-root", type=Path, required=True)
    run_parser.add_argument("--f0-root", type=Path, required=True)
    run_parser.add_argument("--output", type=Path, required=True)
    compare_parser = subparsers.add_parser("compare")
    compare_parser.add_argument("--left", type=Path, required=True)
    compare_parser.add_argument("--right", type=Path, required=True)
    return parser


def main() -> int:
    arguments = _parser().parse_args()
    try:
        if arguments.command == "run":
            result = run(arguments.b0_root, arguments.f0_root, arguments.output)
        else:
            result = compare(arguments.left, arguments.right)
        sys.stdout.buffer.write(common.canonical_json(result))
        return 0
    except (common.M0Error, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
