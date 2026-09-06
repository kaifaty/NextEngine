"""Two bounded train/development iterations with audible outputs and controls.

The whole 761162 recording is excluded from fitting and model statistics.
Its previously disclosed later strikes are development data, not an untouched
scientific holdout. Validation measures reconstruction, not material identity.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import time
from pathlib import Path

import numpy as np
import physical_sound_audible_glass as glass
import physical_sound_audible_glass_unseen as unseen
import torch
from torch import nn
from torch.nn import functional as F

PEAKS = {
    "761160": (0.58, 2.90, 6.89, 9.75, 11.79),
    "761161": (0.77, 2.51, 4.19, 6.02, 7.91, 10.16, 11.99, 13.68, 15.58, 17.32, 19.39),
    "761162": (2.76, 4.47, 6.13, 7.65, 9.24, 10.90, 12.43, 14.34, 16.02, 18.06, 20.62),
}
DEVELOPMENT_ID = "761162"


def split_rows(rows: list[dict]) -> tuple[list[int], list[int]]:
    train = [i for i, row in enumerate(rows) if row["sound_id"] != DEVELOPMENT_ID]
    development = [i for i, row in enumerate(rows) if row["sound_id"] == DEVELOPMENT_ID]
    if not train or not development:
        raise ValueError("both training and development recordings are required")
    return train, development


def eligible(candidate: dict, baseline: dict) -> bool:
    # A relative development decision, not a calibrated perceptual certificate.
    return bool(
        all(np.isfinite(value) for value in candidate.values())
        and candidate["spectral"] <= 0.9 * baseline["spectral"]
        and candidate["envelope"] <= 1.1 * baseline["envelope"]
        and candidate["attack"] <= 1.1 * baseline["attack"]
    )


def metrics(
    targets: torch.Tensor, predictions: torch.Tensor, onsets: list[int]
) -> tuple[dict, list[dict]]:
    with torch.inference_mode():
        spectral, envelope = glass.AudioLoss(targets).parts(predictions)
        records = []
        for i, onset in enumerate(onsets):
            target = targets[i, onset : onset + 1600]
            prediction = predictions[i, onset : onset + 1600]
            target_rms = (
                F.avg_pool1d(target[None, None].square(), 32, 32)
                .clamp_min(1e-10)
                .sqrt()
            )
            predicted_rms = (
                F.avg_pool1d(prediction[None, None].square(), 32, 32)
                .clamp_min(1e-10)
                .sqrt()
            )
            attack = float(
                (predicted_rms - target_rms).norm() / target_rms.norm().clamp_min(1e-8)
            )
            records.append(
                {
                    "spectral": float(spectral[i]),
                    "envelope": float(envelope[i]),
                    "attack": attack,
                }
            )
    return {
        key: float(np.mean([row[key] for row in records])) for key in records[0]
    }, records


def publish(
    output: Path,
    name: str,
    rows: list[dict],
    targets: torch.Tensor,
    baseline: torch.Tensor,
    predictions: torch.Tensor,
) -> str:
    combined = []
    for i, row in enumerate(rows):
        values = (targets[i], baseline[i], predictions[i])
        gain = min(1.0, 0.98 / max(float(value.abs().max()) for value in values))
        for label, value in zip(("original", "previous", name), values):
            audio = value.detach().numpy() * gain
            glass.write_wav(output / f"{name}-{row['id']}-{label}.wav", audio)
            combined.extend((audio, np.zeros(glass.RATE // 2, dtype=np.float32)))
    glass.write_wav(output / f"{name}-comparison.wav", np.concatenate(combined))
    preview = output / f"{name}-preview.wav"
    glass.write_wav(preview, np.concatenate(combined[:18]))
    return str(preview)


def fit_teacher(
    initial: torch.Tensor,
    target: torch.Tensor,
    onset: int,
    synth: glass.Synthesizer,
    deadline: float,
) -> tuple[torch.Tensor, dict]:
    frequencies = torch.arange(0, glass.MODES * 5, 5)
    others = torch.tensor(
        [i for i in range(glass.PARAMETERS) if i % 5 != 0 or i >= glass.MODES * 5]
    )
    model = nn.ParameterDict(
        {
            "frequency": nn.Parameter(initial[frequencies].clone()),
            "other": nn.Parameter(initial[others].clone()),
        }
    )

    def parameters():
        return initial.scatter(0, frequencies, model["frequency"]).scatter(
            0, others, model["other"]
        )

    loss = glass.AudioLoss(target[None])
    optimizer = torch.optim.Adam(
        [
            {"params": [model["frequency"]], "lr": 1e-5},
            {"params": [model["other"]], "lr": 0.02},
        ]
    )
    report = glass.optimize(
        model,
        lambda: loss(synth(parameters(), [onset])),
        optimizer,
        300,
        deadline,
        300,
        lambda step, value: None,
    )
    return parameters().detach(), report


def run(source_root: Path, trained: Path, output: Path, budget: float = 600) -> dict:
    output = output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("output must be a new external directory")
    if not np.isfinite(budget) or budget <= 0:
        raise ValueError("invalid budget")
    start = time.monotonic()
    torch.set_num_threads(4)
    torch.manual_seed(42)
    rows, values, onsets = [], [], []
    for sound_id, _, expected in glass.SOURCES:
        path = source_root / f"{sound_id}_13431397-hq.mp3"
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError("source hash changed")
        decoded = glass.decode(path)
        starts = [round((peak - 0.05) * glass.RATE) for peak in PEAKS[sound_id]]
        unseen.check_disjoint(starts[1:], starts[0])
        for index, peak in enumerate(PEAKS[sound_id]):
            target, onset, info = unseen.crop(decoded, peak)
            rows.append(
                {
                    "id": f"{sound_id}-crop{index + 1:02}",
                    "sound_id": sound_id,
                    "source_sha256": expected,
                    **info,
                }
            )
            values.append(target)
            onsets.append(onset)
    targets = torch.stack(values)
    train, development = split_rows(rows)
    development_rows = [rows[i] for i in development]
    development_onsets = [onsets[i] for i in development]
    inputs = glass.features(targets)
    initial = torch.stack(
        [
            glass.initialize(target.numpy(), onset)
            for target, onset in zip(targets, onsets)
        ]
    )
    synth = glass.Synthesizer()
    checkpoint = trained / "encoder.pt"
    checkpoint_hash = hashlib.sha256(checkpoint.read_bytes()).hexdigest()
    previous = glass.Encoder(torch.zeros(3, 2048), torch.zeros(3, glass.PARAMETERS))
    previous.load_state_dict(
        torch.load(checkpoint, weights_only=True, map_location="cpu")
    )
    previous.eval()
    with torch.inference_mode():
        baseline = synth(previous(inputs[development]), development_onsets)
        analytic = synth(initial[development], development_onsets)
    baseline_metrics, _ = metrics(targets[development], baseline, development_onsets)
    analytic_metrics, _ = metrics(targets[development], analytic, development_onsets)
    output.mkdir(parents=True)
    report = {
        "claim": "DEVELOPMENT_RECONSTRUCTION_ONLY / NO_INDEPENDENT_MATERIAL_VALIDATION",
        "training_sound_ids": sorted({rows[i]["sound_id"] for i in train}),
        "development_sound_ids": [DEVELOPMENT_ID],
        "train_count": len(train),
        "development_count": len(development),
        "checkpoint_sha256": checkpoint_hash,
        "budget_seconds": budget,
        "validator": "mean spectral <=90% of previous; envelope and 50ms attack <=110%; finite output; development selection only",
        "baseline": baseline_metrics,
        "analytic_control": analytic_metrics,
        "rows": rows,
        "train_indices": train,
        "development_indices": development,
        "teachers": [],
        "iterations": [],
        "status": "running",
    }
    report["analytic_preview"] = publish(
        output, "analytic", development_rows, targets[development], baseline, analytic
    )
    glass.write_json(output / "result.json", report)
    teacher_parameters = []
    for order, index in enumerate(train):
        parameters, teacher_report = fit_teacher(
            initial[index],
            targets[index],
            onsets[index],
            synth,
            start + min(420, budget * 0.7) * (order + 1) / len(train),
        )
        teacher_parameters.append(parameters)
        report["teachers"].append({"row_id": rows[index]["id"], **teacher_report})
        glass.write_json(output / "result.json", report)
        print(
            json.dumps(
                {
                    "teacher": order + 1,
                    "total": len(train),
                    "loss": teacher_report["best_loss"],
                    "status": teacher_report["status"],
                }
            ),
            flush=True,
        )
        if teacher_report["status"] == "interrupted":
            report["status"] = "interrupted"
            glass.write_json(output / "result.json", report)
            return report
    teachers = torch.stack(teacher_parameters)
    # Both candidates use exactly the same expanded training set and teachers.
    # The second predicts corrections around per-input analytic modal estimates;
    # its frequencies stay analytic. Its advantage must be measured against
    # the analytic-only control before attributing anything to learning.
    mask = torch.ones(glass.PARAMETERS)
    mask[torch.arange(0, glass.MODES * 5, 5)] = 0
    for iteration, name in enumerate(("expanded", "anchored")):
        torch.manual_seed(42)
        target_parameters = (
            teachers if name == "expanded" else (teachers - initial[train]) * mask
        )
        encoder = glass.Encoder(inputs[train], target_parameters)

        def predict(model=encoder, kind=name):
            raw = model(inputs[development])
            return raw if kind == "expanded" else initial[development] + raw * mask

        def objective(model=encoder, teacher=target_parameters):
            return (
                ((model(inputs[train]) - teacher) / model.output_scale).square().mean()
            )

        def save(step, loss_value, model=encoder, kind=name, prediction=predict):
            with torch.inference_mode():
                audio = synth(prediction(), development_onsets)
            preview = publish(
                output, kind, development_rows, targets[development], baseline, audio
            )
            torch.save(model.state_dict(), output / f"{kind}.pt")
            print(
                json.dumps(
                    {
                        "candidate": kind,
                        "step": step,
                        "training_loss": loss_value,
                        "preview": preview,
                    }
                ),
                flush=True,
            )

        training_result = glass.optimize(
            encoder,
            objective,
            torch.optim.Adam(encoder.parameters(), lr=0.001),
            1500,
            start + budget * (0.85 if iteration == 0 else 1),
            300,
            save,
        )
        with torch.inference_mode():
            prediction = predict()
            audio = synth(prediction, development_onsets)
        scores, individual = metrics(targets[development], audio, development_onsets)
        eligible_result = (
            eligible(scores, baseline_metrics) and training_result["steps"] > 0
        )
        result = {
            "name": name,
            "training": training_result,
            "scores": scores,
            "per_example": individual,
            "eligible_against_previous": eligible_result,
            "beats_analytic_spectral": scores["spectral"]
            < analytic_metrics["spectral"],
            "preview": str(output / f"{name}-preview.wav"),
        }
        report["iterations"].append(result)
        glass.write_json(
            output / f"{name}-parameters.json",
            {"raw": prediction.tolist(), "onsets": development_onsets},
        )
        glass.write_json(output / "result.json", report)
        if training_result["status"] == "interrupted":
            break
    valid = [row for row in report["iterations"] if row["eligible_against_previous"]]
    best = min(valid, key=lambda row: row["scores"]["spectral"]) if valid else None
    report["best_eligible_neural"] = best["name"] if best else None
    report["learning_beats_analytic"] = bool(best and best["beats_analytic_spectral"])
    # A good analytic reconstruction is not relabelled a learned improvement.
    choices = [(baseline_metrics["spectral"], "previous")]
    if eligible(analytic_metrics, baseline_metrics):
        choices.append((analytic_metrics["spectral"], "analytic"))
    choices.extend((row["scores"]["spectral"], row["name"]) for row in valid)
    report["recommended_reconstruction"] = min(choices)[1]
    report["checkpoint_unchanged"] = (
        hashlib.sha256(checkpoint.read_bytes()).hexdigest() == checkpoint_hash
    )
    report["elapsed_seconds"] = time.monotonic() - start
    completed = all(item["status"] == "complete" for item in report["teachers"])
    completed &= len(report["iterations"]) == 2 and all(
        item["training"]["status"] == "complete" for item in report["iterations"]
    )
    report["status"] = "complete" if completed else "partial"
    glass.write_json(output / "result.json", report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--trained", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--budget-seconds", type=float, default=600)
    args = parser.parse_args()
    result = run(args.source_root, args.trained, args.output, args.budget_seconds)
    print(
        json.dumps(
            {
                key: result[key]
                for key in (
                    "status",
                    "best_eligible_neural",
                    "learning_beats_analytic",
                    "recommended_reconstruction",
                    "elapsed_seconds",
                )
            }
        ),
        flush=True,
    )
