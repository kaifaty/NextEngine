"""Post-generation onset diagnostic, not a calibrated visual-contact/quality judge."""

import argparse
import json
import subprocess
from pathlib import Path

import numpy as np
import physical_sound_mmaudio_pilot as pilot
import soundfile as sf
from scipy.optimize import linear_sum_assignment
from scipy.signal import find_peaks


def onsets(wave, rate):
    """Fixed 10ms energy / 5ms hop positive flux; relative thresholds, no fitting."""
    if wave.ndim != 1 or not np.isfinite(wave).all() or len(wave) < rate:
        raise ValueError("At least one finite mono second required")
    hop, window = round(rate * 0.005), round(rate * 0.010)
    frames = np.lib.stride_tricks.sliding_window_view(wave, window)[::hop]
    rms = np.sqrt(np.mean(frames.astype(np.float64) ** 2, axis=1))
    flux = np.maximum(0, np.diff(rms, prepend=0))
    if flux.max() <= 1e-8:
        return np.zeros(0)
    peaks, _ = find_peaks(
        flux,
        height=0.15 * flux.max(),
        prominence=0.10 * flux.max(),
        distance=round(0.1 * rate / hop),
    )
    return (peaks * hop + window / 2) / rate


def match(reference, predicted, tolerance=0.1):
    reference, predicted = np.asarray(reference), np.asarray(predicted)
    if (
        reference.ndim != 1
        or predicted.ndim != 1
        or not all(np.isfinite(x).all() for x in (reference, predicted))
    ):
        raise ValueError("Finite time vectors required")
    if tolerance <= 0 or not np.isfinite(tolerance):
        raise ValueError("Positive finite tolerance required")
    distance = np.abs(reference[:, None] - predicted[None])
    penalty = (len(reference) + len(predicted) + 1) * tolerance
    row, col = linear_sum_assignment(np.where(distance <= tolerance, distance, penalty))
    accepted = distance[row, col] <= tolerance
    errors = distance[row[accepted], col[accepted]]
    count = len(errors)
    return {
        "matched": count,
        "missed": len(reference) - count,
        "extra": len(predicted) - count,
        "recall": count / len(reference) if len(reference) else None,
        "precision": count / len(predicted) if len(predicted) else None,
        "matched_mean_error_s": float(errors.mean()) if count else None,
    }


def run(generated, reference_video):
    if generated.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("Generated media must remain outside the repository")
    result = json.loads((generated / "result.json").read_text())
    if result["status"] != "complete" or result["time_shift_seconds"] != 1:
        raise ValueError("Complete fixed-shift experiment required")
    reference_path = generated / "reference-evaluation-only.wav"
    subprocess.run(
        [
            "ffmpeg",
            "-v",
            "error",
            "-nostdin",
            "-n",
            "-i",
            str(reference_video),
            "-t",
            "8",
            "-map",
            "0:a:0",
            "-ac",
            "1",
            "-ar",
            "44100",
            "-c:a",
            "pcm_s16le",
            str(reference_path),
        ],
        check=True,
    )
    wave, rate = sf.read(reference_path)
    reference = onsets(wave, rate)
    shifted_reference = reference[reference + 1 < 8] + 1
    report = {
        "status": "complete",
        "scope": "audio-onset diagnostic, not measured physical contacts",
        "tolerance_s": 0.1,
        "reference_video_sha256": pilot.digest(reference_video),
        "result_sha256": pilot.digest(generated / "result.json"),
        "reference_sha256": pilot.digest(reference_path),
        "reference_onsets_s": reference.tolist(),
        "rows": {},
    }
    for key, item in result["arms"].items():
        path = Path(item["path"])
        if pilot.digest(path) != item["sha256"]:
            raise ValueError("Changed generated audio")
        wave, rate = sf.read(path)
        predicted = onsets(wave, rate)
        report["rows"][key] = {
            "onsets_s": predicted.tolist(),
            "original_timeline": match(reference, predicted),
            "delayed_timeline": match(shifted_reference, predicted),
        }
    pilot.write_json(generated / "timing.json", report)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--generated", type=Path, required=True)
    parser.add_argument("--reference-video", type=Path, required=True)
    args = parser.parse_args()
    run(args.generated, args.reference_video)
