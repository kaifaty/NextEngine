"""Report-only text -> sound experiment; no reference audio or runtime integration.

AudioLDM2 / Haohe Liu et al., https://huggingface.co/cvssp/audioldm2,
CC-BY-NC-SA-4.0. Weights and generated media stay outside the repository.
The shared CLAP probe measures prompt alignment, NOT independent naturalness,
physical correctness, novelty relative to pretraining, or release eligibility.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import json
import time
from pathlib import Path

import numpy as np
from scipy.io import wavfile
from scipy.signal import resample_poly

MODEL = "cvssp/audioldm2"
REVISION = "c8e7e189d324425c05c4c2f81214041ef4107983"
RATE = 16000
NEGATIVE = "Low quality, music, speech."
# Same seeds across conditions: input changes can be compared without selecting
# a favourable seed. These are qualitative descriptions, not measured physics.
CASES = (
    (
        "glass-metal",
        "A metal spoon tapping a thin glass wine goblet. Clear ringing glass clinks.",
    ),
    (
        "glass-wood",
        "A wooden stick tapping a thin glass wine goblet. Short glass clinks with soft attacks.",
    ),
    (
        "wood-metal",
        "A metal hammer knocking on a solid wooden block. Dry wooden knocks.",
    ),
    (
        "wood-wood",
        "A wooden stick knocking on a solid wooden block. Dry wooden knocks.",
    ),
    (
        "steel-metal",
        "A metal spoon striking an empty steel bucket. Hollow metallic ringing clangs.",
    ),
    (
        "steel-wood",
        "A wooden stick striking an empty steel bucket. Hollow metallic ringing knocks.",
    ),
    (
        "water-pour",
        "Water being poured from a bottle into a glass. Splashing and bubbling liquid.",
    ),
    (
        "rain-light",
        "Gentle light rain with sparse individual raindrops pattering on a glass window.",
    ),
    ("rain-heavy", "Heavy dense rain pounding continuously on a glass window."),
    (
        "scrape-wood",
        "A rough wooden block scraping back and forth across a wooden table.",
    ),
    (
        "roll-marble",
        "A glass marble rolling across a wooden table, rattling and slowing to a stop.",
    ),
    (
        "break-glass",
        "A glass bottle smashing on a concrete floor, followed by small shards tinkling.",
    ),
)
PAIRS = ((0, 1), (2, 3), (4, 5), (7, 8), (0, 4), (2, 4))


def signal_stats(audio: np.ndarray) -> dict:
    audio = np.asarray(audio, dtype=np.float32)
    if audio.ndim != 1 or not audio.size or not np.isfinite(audio).all():
        raise ValueError("expected nonempty finite mono waveform")
    return {
        "peak": float(np.abs(audio).max()),
        "rms": float(np.sqrt(np.mean(audio.astype(np.float64) ** 2))),
        "dc": float(audio.mean()),
        "over_full_scale_fraction": float(np.mean(np.abs(audio) >= 1)),
        "samples": len(audio),
    }


def write_audio(path: Path, audio: np.ndarray) -> dict:
    stats = signal_stats(audio)
    if stats["rms"] < 1e-7:
        raise ValueError("silent generated waveform")
    # Preserve loudness relationships; attenuate only to prevent PCM overload.
    gain = min(1.0, 0.98 / max(stats["peak"], 1e-8))
    pcm = np.round(np.asarray(audio) * gain * 32767).astype(np.int16)
    wavfile.write(path, RATE, pcm)
    return {
        **stats,
        "pcm_gain": gain,
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }


def alignment(scores: np.ndarray, expected: int) -> dict:
    scores = np.asarray(scores)
    if scores.ndim != 1 or scores.size < 2 or not np.isfinite(scores).all():
        raise ValueError("invalid similarity vector")
    if not 0 <= expected < scores.size:
        raise ValueError("invalid target index")
    best = int(scores.argmax())
    return {
        "target_similarity": float(scores[expected]),
        "best_index": best,
        "target_rank": int(1 + np.sum(scores > scores[expected])),
        "target_minus_best_other": float(
            scores[expected] - np.delete(scores, expected).max()
        ),
    }


def pair_margin(matrix: np.ndarray, first: int, second: int) -> float:
    # Positive means the two generated sounds collectively favour their own
    # descriptions over swapped descriptions. Not a probability of correctness.
    return float(
        (
            matrix[first, first]
            + matrix[second, second]
            - matrix[first, second]
            - matrix[second, first]
        )
        / 2
    )


def save_report(path: Path, report: dict):
    pending = path.with_suffix(".pending")
    pending.write_text(json.dumps(report, indent=2, allow_nan=False) + "\n")
    pending.replace(path)


def run(
    output: Path,
    steps: int,
    seeds: tuple[int, ...],
    seconds: float,
    precision: str = "float16",
):
    import torch
    from diffusers import AudioLDM2Pipeline

    if not 1 <= steps <= 300 or not 1 <= seconds <= 10 or not seeds:
        raise ValueError("bounded steps/duration/seeds required")
    if precision not in ("float16", "float32"):
        raise ValueError("unsupported precision")
    if len(seeds) > 8 or len(set(seeds)) != len(seeds) or min(seeds) < 0:
        raise ValueError("one to eight unique nonnegative seeds required")
    repository = Path(__file__).resolve().parents[2]
    output = output.resolve()
    if output.is_relative_to(repository):
        raise ValueError("generated artifacts must remain outside the repository")
    output.mkdir(parents=True, exist_ok=False)
    torch.set_num_threads(4)
    started = time.monotonic()
    report = {
        "status": "running",
        "model": MODEL,
        "revision": REVISION,
        "license": "CC-BY-NC-SA-4.0 / external research only; no engine redistribution",
        "reference_audio_input": False,
        "local_training_steps": 0,
        "validator": "shared generator CLAP; diagnostic only, not independent",
        "steps": steps,
        "precision": precision,
        "seeds": seeds,
        "seconds": seconds,
        "negative_prompt": NEGATIVE,
        "cases": [{"id": key, "prompt": prompt} for key, prompt in CASES],
        "versions": {
            name: importlib.metadata.version(name)
            for name in (
                "torch",
                "diffusers",
                "transformers",
                "accelerate",
                "huggingface-hub",
                "numpy",
                "scipy",
            )
        },
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "rows": [],
        "controls": [],
    }
    save_report(output / "result.json", report)
    try:
        device = "cuda" if torch.cuda.is_available() else "cpu"
        report["device"] = torch.cuda.get_device_name() if device == "cuda" else device
        pipe = AudioLDM2Pipeline.from_pretrained(
            MODEL,
            revision=REVISION,
            torch_dtype=torch.float16
            if device == "cuda" and precision == "float16"
            else torch.float32,
            use_safetensors=True,
            local_files_only=True,
        ).to(device)
        pipe.set_progress_bar_config(disable=True)
        report["load_seconds"] = time.monotonic() - started
        previews = []
        # Every candidate is published before any semantic scoring/selection.
        for seed in seeds:
            for index, (key, prompt) in enumerate(CASES):
                tick = time.monotonic()
                with torch.inference_mode():
                    audio = pipe(
                        prompt,
                        negative_prompt=NEGATIVE,
                        num_inference_steps=steps,
                        audio_length_in_s=seconds,
                        num_waveforms_per_prompt=1,
                        generator=torch.Generator(device).manual_seed(seed),
                    ).audios[0]
                record = write_audio(output / f"{key}-seed{seed}.wav", audio)
                report["rows"].append(
                    {
                        "case": index,
                        "seed": seed,
                        "inference_seconds": time.monotonic() - tick,
                        **record,
                    }
                )
                if seed == seeds[0]:
                    previews.extend(
                        (
                            audio * record["pcm_gain"],
                            np.zeros(RATE // 2, dtype=np.float32),
                        )
                    )
                    write_audio(output / "preview.wav", np.concatenate(previews))
                save_report(output / "result.json", report)
                print(
                    json.dumps(
                        {
                            "generated": key,
                            "seed": seed,
                            "seconds": round(time.monotonic() - tick, 2),
                        }
                    ),
                    flush=True,
                )
            with torch.inference_mode():
                baseline = pipe(
                    "",
                    negative_prompt=NEGATIVE,
                    num_inference_steps=steps,
                    audio_length_in_s=seconds,
                    num_waveforms_per_prompt=1,
                    generator=torch.Generator(device).manual_seed(seed),
                ).audios[0]
            report["controls"].append(
                {
                    "id": "empty-prompt",
                    "seed": seed,
                    **write_audio(output / f"empty-prompt-seed{seed}.wav", baseline),
                }
            )
            save_report(output / "result.json", report)

        # Inference is finished; score PCM as delivered, in FP32 for stability.
        pipe.unet.to("cpu")
        pipe.vae.to("cpu")
        pipe.text_encoder_2.to("cpu")
        pipe.language_model.to("cpu")
        pipe.vocoder.to("cpu")
        pipe.text_encoder.float()
        if device == "cuda":
            torch.cuda.empty_cache()
        with torch.inference_mode():
            tokens = pipe.tokenizer(
                [prompt for _, prompt in CASES], padding=True, return_tensors="pt"
            ).to(device)
            text_features = pipe.text_encoder.get_text_features(**tokens)
            text_features = torch.nn.functional.normalize(text_features, dim=-1)

            def score(audio):
                # CLAP requires 48 kHz, whereas AudioLDM2 emits 16 kHz.
                values = resample_poly(audio, 3, 1)
                np.random.seed(0)  # fixed feature extraction, not seed selection
                features = pipe.feature_extractor(
                    [values], sampling_rate=48000, return_tensors="pt"
                ).to(device)
                embedding = pipe.text_encoder.get_audio_features(**features)
                embedding = torch.nn.functional.normalize(embedding, dim=-1)
                return (embedding @ text_features.T)[0].cpu().numpy()

            for row in report["rows"] + report["controls"]:
                sample_rate, pcm = wavfile.read(row["wav"])
                if sample_rate != RATE:
                    raise ValueError("unexpected output sample rate")
                scores = score(pcm.astype(np.float32) / 32768)
                row["similarities"] = scores.tolist()
                if "case" in row:
                    row.update(alignment(scores, row["case"]))
            count = int(seconds * RATE)
            rng = np.random.default_rng(0)
            for key, value in (
                ("silence", np.zeros(count, dtype=np.float32)),
                ("white-noise", rng.normal(0, 0.05, count).astype(np.float32)),
                (
                    "pure-tone",
                    (0.1 * np.sin(2 * np.pi * 1000 * np.arange(count) / RATE)).astype(
                        np.float32
                    ),
                ),
            ):
                report["controls"].append(
                    {"id": key, "similarities": score(value).tolist()}
                )

        report["paired_changes"] = []
        for seed in seeds:
            rows = [row for row in report["rows"] if row["seed"] == seed]
            matrix = np.array([row["similarities"] for row in rows])
            baseline = next(
                row for row in report["controls"] if row.get("seed") == seed
            )
            for index, row in enumerate(rows):
                row["target_gain_over_empty_prompt"] = float(
                    matrix[index, index] - baseline["similarities"][index]
                )
            for first, second in PAIRS:
                report["paired_changes"].append(
                    {
                        "first": CASES[first][0],
                        "second": CASES[second][0],
                        "seed": seed,
                        "own_vs_swapped_margin": pair_margin(matrix, first, second),
                    }
                )
        report.update(
            {
                "status": "complete",
                "elapsed_seconds": time.monotonic() - started,
                "top1_count": sum(row["target_rank"] == 1 for row in report["rows"]),
                "beats_empty_prompt_count": sum(
                    row["target_gain_over_empty_prompt"] > 0 for row in report["rows"]
                ),
                "preview": str(output / "preview.wav"),
                "claims_not_established": [
                    "independent perceptual quality",
                    "precise geometry/force/velocity control",
                    "new-object or pretraining-unseen generalization",
                    "production admission",
                    "local learning improvement",
                ],
            }
        )
    except BaseException as error:
        report.update(
            {
                "status": "interrupted"
                if isinstance(error, KeyboardInterrupt)
                else "failed",
                "error": f"{type(error).__name__}: {error}",
                "elapsed_seconds": time.monotonic() - started,
            }
        )
        save_report(output / "result.json", report)
        raise
    save_report(output / "result.json", report)
    print(
        json.dumps(
            {
                key: report[key]
                for key in (
                    "status",
                    "elapsed_seconds",
                    "top1_count",
                    "beats_empty_prompt_count",
                    "preview",
                )
            }
        ),
        flush=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--steps", type=int, default=100)
    parser.add_argument("--seeds", type=int, nargs="+", default=[42, 123])
    parser.add_argument("--seconds", type=float, default=5.0)
    parser.add_argument(
        "--precision", choices=("float16", "float32"), default="float16"
    )
    args = parser.parse_args()
    run(args.output, args.steps, tuple(args.seeds), args.seconds, args.precision)
