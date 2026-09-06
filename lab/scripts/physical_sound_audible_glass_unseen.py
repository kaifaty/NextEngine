"""Evaluate the frozen audible-glass encoder on later, non-overlapping hits.

All inputs remain disclosed generator-train recordings. These are unseen
windows for this checkpoint, not independent objects or protected holdouts.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from itertools import pairwise
from pathlib import Path

import numpy as np
import physical_sound_audible_glass as glass
import torch

PEAKS = {
    "761160": (2.90, 6.89, 9.75),
    "761161": (2.51, 4.19, 6.02),
    "761162": (2.76, 4.47, 6.13),
}


def crop(decoded: np.ndarray, peak_seconds: float) -> tuple[torch.Tensor, int, dict]:
    start = round((peak_seconds - 0.05) * glass.RATE)
    if start < 0 or start + glass.SAMPLES > len(decoded):
        raise ValueError("crop outside source")
    value = decoded[start : start + glass.SAMPLES].copy()
    value -= value.mean()
    glass.validate_audio(value)
    gain = 0.8 / float(np.max(np.abs(value)))
    value *= gain
    rms = np.sqrt(np.convolve(value[:3200] ** 2, np.ones(32) / 32, mode="same"))
    onset = int(np.flatnonzero(rms > rms.max() * 0.1)[0])
    return (
        torch.tensor(value),
        onset,
        {
            "crop_start_sample": start,
            "crop_samples": glass.SAMPLES,
            "normalization_gain": gain,
            "onset_sample": onset,
        },
    )


def check_disjoint(starts: list[int], training_start: int) -> None:
    ordered = sorted([training_start, *starts])
    if any(right < left + glass.SAMPLES for left, right in pairwise(ordered)):
        raise ValueError("evaluation windows overlap each other or the training window")


def run(source_root: Path, trained: Path, output: Path) -> dict:
    output = output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("output must be a new external directory")
    torch.set_num_threads(4)
    checkpoint = trained / "encoder.pt"
    checkpoint_hash = hashlib.sha256(checkpoint.read_bytes()).hexdigest()
    old_report = json.loads((trained / "result.json").read_text())
    old_parameters = json.loads((trained / "parameters.json").read_text())
    training, training_onsets, sources = glass.load_sources(source_root)
    training_features = glass.features(training)
    encoder = glass.Encoder(torch.zeros(3, 2048), torch.zeros(3, glass.PARAMETERS))
    encoder.load_state_dict(
        torch.load(checkpoint, map_location="cpu", weights_only=True)
    )
    encoder.eval()
    synth = glass.Synthesizer()
    # Positive control: same checkpoint and preprocessing must retain the
    # already published parameter predictions before measuring transfer.
    with torch.inference_mode():
        control = encoder(training_features)
        expected = torch.tensor(old_parameters["neural_raw"], dtype=torch.float32)
        if not torch.equal(control, expected):
            raise ValueError("published training prediction changed")
        control_audio = synth(control, training_onsets)
        training_spectral, _ = glass.AudioLoss(training).parts(control_audio)

    rows, signals, onsets = [], [], []
    for source, (sound_id, first_peak, _) in zip(sources, glass.SOURCES):
        decoded = glass.decode(Path(source["path"]))
        starts = [round((peak - 0.05) * glass.RATE) for peak in PEAKS[sound_id]]
        check_disjoint(starts, round((first_peak - 0.05) * glass.RATE))
        # Also ensure this evaluation is disjoint from the actual recorded
        # training windows, not just this version's default constants.
        previous = next(
            item for item in old_report["sources"] if item["sound_id"] == sound_id
        )
        if (
            previous["sha256"] != source["sha256"]
            or previous["crop_samples"] != glass.SAMPLES
        ):
            raise ValueError("training source or crop identity changed")
        check_disjoint(starts, round(previous["crop_start_seconds"] * glass.RATE))
        for index, peak in enumerate(PEAKS[sound_id], start=2):
            value, onset, metadata = crop(decoded, peak)
            rows.append(
                {
                    "id": f"{sound_id}-hit{index}",
                    "sound_id": sound_id,
                    "source_sha256": source["sha256"],
                    **metadata,
                }
            )
            signals.append(value)
            onsets.append(onset)
    targets = torch.stack(signals)
    with torch.inference_mode():
        inputs = glass.features(targets)
        parameters = encoder(inputs)
        predictions = synth(parameters, onsets)
        # Retrieval sees the same audio features; its library contains only
        # the three old fitted parameter vectors. No new target is fitted.
        distances = (
            ((inputs[:, None] - training_features[None]) / encoder.input_scale) ** 2
        ).mean(2)
        nearest_indices = distances.argmin(1)
        nearest_parameters = torch.tensor(old_parameters["direct_raw"])[nearest_indices]
        nearest = synth(nearest_parameters, onsets)
        loss = glass.AudioLoss(targets)
        neural_spectral, neural_envelope = loss.parts(predictions)
        nearest_spectral, nearest_envelope = loss.parts(nearest)
    if not torch.isfinite(predictions).all() or not torch.isfinite(nearest).all():
        raise ValueError("nonfinite output from frozen model")
    output.mkdir(parents=True)
    pairs, triplets = [], []
    for i, row in enumerate(rows):
        variants = {
            "original": targets[i],
            "neural": predictions[i],
            "nearest": nearest[i],
        }
        peak = max(float(value.abs().max()) for value in variants.values())
        gain = min(1.0, 0.98 / max(peak, 1e-8))
        audio = {key: value.numpy() * gain for key, value in variants.items()}
        row["common_gain"] = gain
        row["paths"] = {}
        for key, value in audio.items():
            path = output / f"{row['id']}-{key}.wav"
            glass.write_wav(path, value)
            row["paths"][key] = str(path)
        silence = np.zeros(glass.RATE // 2, dtype=np.float32)
        pairs.extend((audio["original"], silence, audio["neural"], silence))
        triplets.extend(
            (
                audio["original"],
                silence,
                audio["neural"],
                silence,
                audio["nearest"],
                silence,
            )
        )
        training_row = next(
            j
            for j, source in enumerate(sources)
            if source["sound_id"] == row["sound_id"]
        )
        row.update(
            {
                "neural_spectral": float(neural_spectral[i]),
                "neural_envelope": float(neural_envelope[i]),
                "nearest_spectral": float(nearest_spectral[i]),
                "nearest_envelope": float(nearest_envelope[i]),
                "nearest_training_sound_id": sources[int(nearest_indices[i])][
                    "sound_id"
                ],
                "versus_same_recording_train_error": float(
                    neural_spectral[i] / training_spectral[training_row]
                ),
                "neural_beats_retrieval": bool(
                    neural_spectral[i] < nearest_spectral[i]
                ),
                "original_diagnostics": glass.diagnostics(targets[i].numpy()),
                "neural_diagnostics": glass.diagnostics(predictions[i].numpy()),
            }
        )
    glass.write_wav(output / "comparison.wav", np.concatenate(pairs))
    glass.write_wav(output / "comparison-with-retrieval.wav", np.concatenate(triplets))
    # Short, fixed-order preview: first unseen strike from each recording.
    preview = np.concatenate(
        [np.concatenate(pairs[i * 4 : (i + 1) * 4]) for i in (0, 3, 6)]
    )
    glass.write_wav(output / "preview.wav", preview)
    if hashlib.sha256(checkpoint.read_bytes()).hexdigest() != checkpoint_hash:
        raise ValueError("checkpoint changed during evaluation")
    report = {
        "claim": "UNSEEN_WINDOWS_SAME_RECORDINGS / REPORT_ONLY / NO_NEW_OBJECT_CLAIM",
        "checkpoint_sha256": checkpoint_hash,
        "checkpoint": str(checkpoint.resolve()),
        "training_prediction_exact": True,
        "training_steps": 0,
        "selection": "first three previously listed later RMS peaks per recording; no score-based selection",
        "comparison_order": ["original", "neural"],
        "retrieval_order": ["original", "neural", "nearest"],
        "mean_training_spectral": float(training_spectral.mean()),
        "mean_unseen_spectral": float(neural_spectral.mean()),
        "mean_retrieval_spectral": float(nearest_spectral.mean()),
        "neural_beats_retrieval_count": int((neural_spectral < nearest_spectral).sum()),
        "examples": rows,
        "comparison": str(output / "comparison.wav"),
        "preview": str(output / "preview.wav"),
    }
    glass.write_json(output / "result.json", report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--trained", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    result = run(arguments.source_root, arguments.trained, arguments.output)
    print(
        json.dumps(
            {key: value for key, value in result.items() if key != "examples"}, indent=2
        )
    )
