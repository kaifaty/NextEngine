"""Separate AST audio-only diagnostic for the text-to-sound pilot.

Uses Gong et al. AST (BSD-3-Clause), not generator CLAP weights or prompts.
AudioSet pretraining can overlap the generator: not independent data evidence.
Tag scores are uncalibrated, not probabilities of material or naturalness.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_text_pilot as pilot
from scipy.io import wavfile
from scipy.signal import resample_poly

MODEL = "MIT/ast-finetuned-audioset-10-10-0.4593"
REVISION = "f826b80d28226b62986cc218e5cec390b1096902"
# Coarse ontology queries only. Neither "steel" nor the material of the
# striker is an AudioSet label. Keep those limitations visible instead of
# pretending a generic clink certifies a specific material pair.
EXPECTED = {
    "glass-metal": ("Glass", "Chink, clink"),
    "glass-wood": ("Glass", "Chink, clink"),
    "wood-metal": ("Wood", "Knock", "Wood block"),
    "wood-wood": ("Wood", "Knock", "Wood block"),
    "steel-metal": (),
    "steel-wood": (),
    "water-pour": ("Water", "Pour"),
    "rain-light": ("Rain", "Raindrop", "Rain on surface"),
    "rain-heavy": ("Rain", "Raindrop", "Rain on surface"),
    "scrape-wood": ("Scrape",),
    "roll-marble": ("Roll",),
    "break-glass": ("Glass", "Shatter"),
    "real-glass": ("Glass", "Chink, clink"),
    "silence": ("Silence",),
    "white-noise": ("White noise",),
    "pure-tone": ("Sine wave",),
    "empty-prompt": (),
}


def summarize(scores: np.ndarray, labels: list[str], expected: tuple[str, ...]) -> dict:
    if scores.shape != (len(labels),) or not np.isfinite(scores).all():
        raise ValueError("invalid classifier output")
    if not set(expected).issubset(labels):
        raise ValueError("expected tag absent from classifier ontology")
    order = np.argsort(-scores, kind="stable")
    ranks = {labels[index]: rank + 1 for rank, index in enumerate(order)}
    return {
        "top10": [
            {"label": labels[index], "score": float(scores[index])}
            for index in order[:10]
        ],
        "expected_tags": [
            {
                "label": label,
                "score": float(scores[labels.index(label)]),
                "rank": ranks[label],
            }
            for label in expected
        ],
        "expected_in_top5": any(ranks[label] <= 5 for label in expected)
        if expected
        else None,
    }


def load_audio(path: Path, expected_hash: str | None = None) -> np.ndarray:
    if expected_hash and hashlib.sha256(path.read_bytes()).hexdigest() != expected_hash:
        raise ValueError("waveform hash mismatch")
    rate, value = wavfile.read(path)
    if value.dtype != np.int16 or value.ndim != 1 or not 8000 <= rate <= 96000:
        raise ValueError("expected mono PCM16 WAV")
    if not 0 < len(value) <= 10 * rate:
        raise ValueError("expected at most ten seconds of audio")
    divisor = np.gcd(rate, pilot.RATE)
    return resample_poly(
        value.astype(np.float32) / 32768, pilot.RATE // divisor, rate // divisor
    )


def clap_measurement(manifest: dict) -> dict:
    """Same frozen CLAP as AudioLDM2; separate report, not an unbiased judge."""
    import torch
    from transformers import ClapFeatureExtractor, ClapModel, RobertaTokenizer

    options = {"revision": pilot.REVISION, "local_files_only": True}
    model = ClapModel.from_pretrained(
        pilot.MODEL, subfolder="text_encoder", use_safetensors=True, **options
    ).eval()
    tokenizer = RobertaTokenizer.from_pretrained(
        pilot.MODEL, subfolder="tokenizer", **options
    )
    extractor = ClapFeatureExtractor.from_pretrained(
        pilot.MODEL, subfolder="feature_extractor", **options
    )
    records, controls = [], []
    with torch.inference_mode():
        tokens = tokenizer(
            [row["prompt"] for row in manifest["cases"]],
            padding=True,
            return_tensors="pt",
        )
        text = torch.nn.functional.normalize(model.get_text_features(**tokens), dim=-1)
        for row in manifest["rows"] + manifest["controls"]:
            if "wav" not in row:
                continue
            audio = resample_poly(load_audio(Path(row["wav"]), row["sha256"]), 3, 1)
            np.random.seed(0)
            features = extractor([audio], sampling_rate=48000, return_tensors="pt")
            embedding = torch.nn.functional.normalize(
                model.get_audio_features(**features), dim=-1
            )
            scores = (embedding @ text.T)[0].numpy()
            measured = {"seed": row["seed"], "similarities": scores.tolist()}
            if "case" in row:
                records.append(
                    {
                        **measured,
                        "case": row["case"],
                        **pilot.alignment(scores, row["case"]),
                    }
                )
            else:
                controls.append({**measured, "id": row["id"]})
    paired = []
    for seed in manifest["seeds"]:
        rows = sorted(
            (row for row in records if row["seed"] == seed), key=lambda row: row["case"]
        )
        if [row["case"] for row in rows] != list(range(len(manifest["cases"]))):
            raise ValueError("incomplete case matrix")
        matrix = np.array([row["similarities"] for row in rows])
        baseline = next(
            row
            for row in controls
            if row["seed"] == seed and row["id"] == "empty-prompt"
        )
        for index, row in enumerate(rows):
            row["target_gain_over_empty_prompt"] = float(
                matrix[index, index] - baseline["similarities"][index]
            )
        # Pair indices are defined only for the original fixed experiment set.
        if manifest["cases"] == [
            {"id": key, "prompt": prompt} for key, prompt in pilot.CASES
        ]:
            for first, second in pilot.PAIRS:
                paired.append(
                    {
                        "seed": seed,
                        "first": pilot.CASES[first][0],
                        "second": pilot.CASES[second][0],
                        "own_vs_swapped_margin": pilot.pair_margin(
                            matrix, first, second
                        ),
                    }
                )
    return {
        "model": f"{pilot.MODEL}/text_encoder",
        "revision": pilot.REVISION,
        "scope": "prompt alignment, not calibrated quality; potentially biased by generator/pretraining",
        "rows": records,
        "controls": controls,
        "paired_changes": paired,
        "top1_count": sum(row["target_rank"] == 1 for row in records),
        "beats_empty_prompt_count": sum(
            row["target_gain_over_empty_prompt"] > 0 for row in records
        ),
    }


def run(source: Path, output: Path, real_glass: list[Path], with_clap: bool = False):
    import torch
    from transformers import ASTFeatureExtractor, ASTForAudioClassification

    if output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("report must remain outside the repository")
    if output.exists():
        raise ValueError("refusing to overwrite a previous report")
    manifest = json.loads(source.read_text())
    if manifest["status"] != "complete":
        raise ValueError("pilot must complete before this measurement")
    torch.set_num_threads(4)
    model = ASTForAudioClassification.from_pretrained(
        MODEL, revision=REVISION, use_safetensors=True, local_files_only=True
    ).eval()
    extractor = ASTFeatureExtractor.from_pretrained(
        MODEL, revision=REVISION, local_files_only=True
    )
    labels = [model.config.id2label[index] for index in range(model.config.num_labels)]
    report = {
        "model": MODEL,
        "revision": REVISION,
        "license": "BSD-3-Clause",
        "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "text_input": False,
        "generator_weights_shared": False,
        "pretraining_data_disjoint": "not established",
        "status": "diagnostic only; no calibrated acceptance authority",
        "rows": [],
    }
    inputs = []
    for row in manifest["rows"] + manifest["controls"]:
        if "wav" in row:
            key = manifest["cases"][row["case"]]["id"] if "case" in row else row["id"]
            inputs.append(
                (
                    key,
                    {"wav": row["wav"], "sha256": row["sha256"], "seed": row["seed"]},
                    load_audio(Path(row["wav"]), row["sha256"]),
                )
            )
    # Previously disclosed training originals are explicitly controls, not new
    # unseen validation examples. They test whether AST recognizes real glass.
    for path in real_glass:
        inputs.append(
            (
                "real-glass",
                {
                    "wav": str(path),
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                    "role": "disclosed training positive control",
                },
                load_audio(path),
            )
        )
    count = int(manifest["seconds"] * pilot.RATE)
    rng = np.random.default_rng(0)
    inputs.extend(
        (
            ("silence", {}, np.zeros(count, dtype=np.float32)),
            ("white-noise", {}, rng.normal(0, 0.05, count).astype(np.float32)),
            (
                "pure-tone",
                {},
                (0.1 * np.sin(2 * np.pi * 1000 * np.arange(count) / pilot.RATE)).astype(
                    np.float32
                ),
            ),
        )
    )
    with torch.inference_mode():
        for key, metadata, audio in inputs:
            features = extractor(audio, sampling_rate=pilot.RATE, return_tensors="pt")
            scores = model(**features).logits[0].sigmoid().numpy()
            record = {"id": key, **metadata, **summarize(scores, labels, EXPECTED[key])}
            report["rows"].append(record)
            print(json.dumps({"id": key, "top3": record["top10"][:3]}), flush=True)
    if with_clap:
        report["clap"] = clap_measurement(manifest)
    pilot.save_report(output, report)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--real-glass", type=Path, nargs="*", default=[])
    parser.add_argument("--with-clap", action="store_true")
    args = parser.parse_args()
    run(args.source, args.output, args.real_glass, args.with_clap)
