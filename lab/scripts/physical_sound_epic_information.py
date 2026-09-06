"""EPIC TRAIN-only participant-separated material information discriminator.

Huh/Chalk et al., EPIC-SOUNDS CC-BY-NC4; local noncommercial research only.
No automatic quality admission, physical measurements or generator fitting.
"""

import argparse
import hashlib
import json
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import pandas as pd
import physical_sound_epic_pair_bridge as pair
import physical_sound_epic_slice as source
import physical_sound_text_tags as tags
import torch
from scipy.io import wavfile
from scipy.signal import resample_poly
from sklearn.linear_model import LogisticRegression
from sklearn.preprocessing import StandardScaler


def select(frame):
    """Source-order caps, overlap tested against ALL annotations, not just pairs."""
    if (
        frame.annotation_id.duplicated().any()
        or not frame.annotation_id.str.fullmatch(r"P\d{2}_\d{2,3}_\d+").all()
    ):
        raise ValueError("invalid annotation identity")
    if not ((frame.start_sample >= 0) & (frame.stop_sample > frame.start_sample)).all():
        raise ValueError("invalid annotation bounds")
    if any(
        not np.equal(frame[k], np.floor(frame[k])).all()
        for k in ("start_sample", "stop_sample")
    ):
        raise ValueError("integer samples required")
    nonoverlap = set()
    for _, video in frame.groupby("video_id", sort=False):
        video = video.sort_values("start_sample", kind="stable")
        start, end = video.start_sample.to_numpy(), video.stop_sample.to_numpy()
        previous = np.r_[False, np.maximum.accumulate(end)[:-1] > start[1:]]
        following = np.r_[start[1:] < end[:-1], False]
        nonoverlap.update(video.loc[~(previous | following), "annotation_id"])
    duration = (frame.stop_sample - frame.start_sample) / source.RATE
    eligible = frame[
        frame.annotation_id.isin(nonoverlap)
        & duration.between(0.25, 3)
        & ~frame.participant_id.isin(["P04", "P07"])
        & frame["class"].isin(source.CLASSES)
    ]
    rows = []
    for label in source.CLASSES:
        subset = eligible[eligible["class"] == label]
        participants = subset.participant_id.drop_duplicates().iloc[:12]
        chosen = (
            subset[subset.participant_id.isin(participants)]
            .groupby("participant_id", sort=False)
            .head(3)
        )
        if chosen.participant_id.nunique() < 3:
            raise ValueError("at least three participants per class required")
        rows.extend(chosen.to_dict("records"))
    return rows, eligible.groupby("class").size().to_dict()


def summary(labels, predictions):
    matrix = np.zeros((6, 6), dtype=int)
    np.add.at(matrix, (labels, predictions), 1)
    counts = matrix.sum(axis=1)
    if (counts == 0).any():
        raise ValueError("all six classes required for aggregate recall")
    recalls = matrix.diagonal() / counts
    return {
        "accuracy": float(np.mean(labels == predictions)),
        "macro_recall": float(recalls.mean()),
        "recalls": recalls.tolist(),
        "counts": counts.tolist(),
        "confusion": matrix.tolist(),
    }


def cross_predict(features, labels, groups):
    """No participant, test moments, labels or tuning enters the fitted fold."""
    if (
        features.ndim != 2
        or features.shape[0] != len(labels)
        or not np.isfinite(features).all()
    ):
        raise ValueError("finite aligned feature matrix required")
    predictions = np.full(len(labels), -1, int)
    folds = []
    for group in np.unique(groups):
        held = groups == group
        if set(labels[~held]) != set(range(6)):
            raise ValueError("every training fold must cover all six classes")
        scale = StandardScaler().fit(features[~held])
        model = LogisticRegression(C=1.0, class_weight="balanced", max_iter=1000)
        model.fit(scale.transform(features[~held]), labels[~held])
        if np.max(model.n_iter_) >= 1000:
            raise ValueError("logistic probe did not converge; no silent acceptance")
        predictions[held] = model.predict(scale.transform(features[held]))
        folds.append(
            {
                "held_participant": str(group),
                "train_count": int((~held).sum()),
                "evaluation_count": int(held.sum()),
                "iterations": model.n_iter_.tolist(),
            }
        )
    return predictions, folds


def evaluate(features, rows):
    labels = np.array([source.CLASSES.index(r["class"]) for r in rows])
    groups = np.array([r["participant_id"] for r in rows])
    report = {
        "scope": "TRAIN cross-validation diagnostic, not a calibrated acceptance classifier",
        "classes": list(source.CLASSES),
        "methods": {},
    }
    for name, values in features.items():
        predicted, folds = cross_predict(values, labels, groups)
        report["methods"][name] = {
            **summary(labels, predicted),
            "predictions": predicted.tolist(),
            "folds": folds,
        }
    # One fixed family of within-participant permutations preserves recording
    # context / class-frequency confounds. No selection of a favourable null.
    rng = np.random.default_rng(20260905)
    null = []
    for i in range(32):
        shuffled = labels.copy()
        for group in np.unique(groups):
            mask = groups == group
            shuffled[mask] = rng.permutation(labels[mask])
        predicted, _ = cross_predict(features["ast_raw"], shuffled, groups)
        null.append(summary(shuffled, predicted)["macro_recall"])
    observed = report["methods"]["ast_raw"]["macro_recall"]
    report["within_participant_permutation"] = {
        "seed": 20260905,
        "macro_recalls": null,
        "mean": float(np.mean(null)),
        "max": float(np.max(null)),
        "monte_carlo_p": (1 + sum(x >= observed for x in null)) / 33,
        "scope": "32 fixed permutations; exploratory, no multiple-comparison or quality guarantee",
    }
    return report


@torch.no_grad()
def embeddings(rows, device):
    from transformers import ASTFeatureExtractor, ASTForAudioClassification

    options = {"revision": tags.REVISION, "local_files_only": True}
    model = (
        ASTForAudioClassification.from_pretrained(
            tags.MODEL, use_safetensors=True, **options
        )
        .eval()
        .to(device)
    )
    model.requires_grad_(False)
    extractor = ASTFeatureExtractor.from_pretrained(tags.MODEL, **options)
    features = {
        "ast_raw": [],
        "ast_rms005": [],
        "relative_spectrum": [],
        "duration_gain_only": [],
    }
    for i, row in enumerate(rows):
        wave = resample_poly(pair.checked_wave(row), 2, 3)
        rms = float(np.sqrt(np.mean(wave.astype(np.float64) ** 2)))
        features["relative_spectrum"].append(pair.profile(wave, 16000))
        features["duration_gain_only"].append(
            [
                len(wave) / 16000,
                np.log(max(rms, 1e-12)),
                np.log(max(float(abs(wave).max()), 1e-12)),
            ]
        )
        for key, level in (("ast_raw", None), ("ast_rms005", 0.005)):
            audio, _ = tags.ast_level_control(wave, level)
            inputs = extractor(audio, sampling_rate=16000, return_tensors="pt").to(
                device
            )
            vector = (
                model.audio_spectrogram_transformer(**inputs)
                .pooler_output[0]
                .cpu()
                .numpy()
            )
            if vector.shape != (768,) or not np.isfinite(vector).all():
                raise ValueError("invalid AST pooled feature")
            features[key].append(vector)
        if (i + 1) % 10 == 0:
            print({"embedded": i + 1, "total": len(rows)}, flush=True)
    return {key: np.array(value) for key, value in features.items()}


def run(root, bridge_root, output, device="cuda"):
    output = pair.b.flow.c.v.phase.d.fresh_output(output)
    manifest = json.loads((root / "result.json").read_text())
    bridge = json.loads((bridge_root / "result.json").read_text())
    if (
        manifest["status"] != "complete"
        or manifest["annotation_revision"] != source.ANNOTATIONS
        or bridge["status"] != "complete"
    ):
        raise ValueError("completed matching source/checkpoint required")
    for row in manifest["files"]:
        if (
            hashlib.sha256((root / row["path"]).read_bytes()).hexdigest()
            != row["sha256"]
        ):
            raise ValueError("source metadata changed")
    frame = pd.read_csv(root / "train.csv")
    selected, census = select(frame)
    known = {r["annotation_id"]: r for r in manifest["rows"] + bridge["train_rows"]}
    splits, hashes = (
        pd.read_csv(root / "video-splits55.csv"),
        pd.read_csv(root / "video-md5.csv"),
    )
    report = {
        "status": "running",
        "annotation_revision": source.ANNOTATIONS,
        "license": manifest["license"],
        "source": manifest["source"],
        "source_result_sha256": hashlib.sha256(
            (root / "result.json").read_bytes()
        ).hexdigest(),
        "bridge_result_sha256": hashlib.sha256(
            (bridge_root / "result.json").read_bytes()
        ).hexdigest(),
        "selection": "first 12 source-order participants per pair; first 3 nonoverlapping .25–3s TRAIN clips each; P04/P07 excluded",
        "eligible_census": census,
        "selected_annotations": [r["annotation_id"] for r in selected],
        "scope": "real-positive discriminator, NOT new generator, object-independent proof or quality admission; no author val/test",
        "rows": [],
        "ast_model": tags.MODEL,
        "ast_revision": tags.REVISION,
        "ast_device": device,
        "pretraining_overlap": "unknown",
    }
    save = lambda: source.save(output / "result.json", report)
    save()
    try:

        def acquire(row):
            if row["annotation_id"] in known:
                old = known[row["annotation_id"]]
                if any(old[k] != row[k] for k in row):
                    raise ValueError("cached annotation differs")
                pair.checked_wave(old)
                return old
            return source.extract(row, source.video_source(row, splits, hashes), output)

        with ThreadPoolExecutor(max_workers=2) as pool:
            for row in pool.map(acquire, selected):
                report["rows"].append(row)
                save()
                print(
                    {"acquired": len(report["rows"]), "total": len(selected)},
                    flush=True,
                )
        # Whole source audition and a short fixed first-new-participant preview.
        preview, all_pcm, preview_pcm = [], [], []
        for row in report["rows"]:
            _, pcm = wavfile.read(row["wav"])
            all_pcm.extend([pcm, np.zeros(12000, np.int16)])
            if (
                row["participant_id"] not in ("P01", "P02", "P03")
                and sum(r["class"] == row["class"] for r in preview) < 2
            ):
                preview.append(row)
                preview_pcm.extend([pcm, np.zeros(12000, np.int16)])
        for name, pieces in (("all-sources", all_pcm), ("preview", preview_pcm)):
            path = output / (name + ".wav")
            pcm = np.concatenate(pieces)
            wavfile.write(path, 24000, pcm)
            report[name] = {
                "wav": str(path),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "seconds": len(pcm) / 24000,
            }
        report["preview_order"] = [r["annotation_id"] for r in preview]
        save()
        torch.set_num_threads(4)
        features = embeddings(report["rows"], device)
        np.savez(output / "features.npz", **features)
        report["features_sha256"] = hashlib.sha256(
            (output / "features.npz").read_bytes()
        ).hexdigest()
        save()
        report["evaluation"] = evaluate(features, report["rows"])
        report["status"] = "complete"
        save()
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--source", type=Path, required=True)
    p.add_argument("--bridge", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--device", choices=("cpu", "cuda"), default="cuda")
    a = p.parse_args()
    run(a.source, a.bridge, a.output, a.device)
