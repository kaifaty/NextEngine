"""Learn condition-to-resonance trajectories and render reference-free pouring.

Teacher traces are uncertain pseudo-targets, not physical ground truth. A frozen
neural texture is dynamically filtered at the learned frequency. Filter strength
and bandwidth are fixed experiment choices, not learned material parameters.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_pouring_compare as compare
import physical_sound_pouring_pilot as p
import soundfile as sf
import torch
from safetensors.torch import load_file, save_file
from scipy.signal import istft, stft
from torch import nn

FFT = 2048
HOP = 256
STEPS = 1000


class ResonanceHead(nn.Module):
    def __init__(self):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(11, 64), nn.SiLU(), nn.Linear(64, 64), nn.SiLU(), nn.Linear(64, 1)
        )

    def forward(self, controls):
        return self.net(controls).squeeze(-1)


def fit(controls, targets):
    torch.manual_seed(53)
    model = ResonanceHead().cuda()
    x = torch.tensor(controls.reshape(-1, 11), dtype=torch.float32, device="cuda")
    y = torch.tensor(targets.reshape(-1), dtype=torch.float32, device="cuda")
    optimizer = torch.optim.AdamW(model.parameters(), lr=1e-3, weight_decay=1e-4)
    history = []
    for step in range(STEPS):
        indices = torch.randint(len(x), (128,), device="cuda")
        loss = ((model(x[indices]) - y[indices]) ** 2).mean()
        optimizer.zero_grad(set_to_none=True)
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), 1)
        optimizer.step()
        if step % 100 == 0 or step == STEPS - 1:
            history.append({"step": step + 1, "loss": float(loss.detach())})
    return model.eval(), history


@torch.inference_mode()
def predict(model, controls):
    x = np.asarray(controls, dtype=np.float32)
    if x.ndim != 2 or x.shape[1] != 11 or not np.isfinite(x).all():
        raise ValueError("invalid trajectory controls")
    value = model(torch.from_numpy(x).to(next(model.parameters()).device)).cpu().numpy()
    hz = 34000 / (2 ** (value * 2 + 5))
    if not np.isfinite(hz).all() or (hz < 50).any() or (hz > 7800).any():
        raise ValueError("trajectory exceeds numerical frequency range")
    return hz


def grid(controls, times):
    controls = np.asarray(controls, dtype=np.float32)
    times = np.asarray(times)
    if (
        controls.shape != (11,)
        or not np.isfinite(controls).all()
        or not ((controls[:3] > 0) & (controls[:3] < 5)).all()
        or not 1 <= controls[3] * 30 <= 60
        or not 0 <= controls[4] <= 1
        or not np.isin(controls[5:], [0, 1]).all()
        or controls[5:9].sum() != 1
        or controls[9:11].sum() != 1
        or times.ndim != 1
        or not np.isfinite(times).all()
        or not ((times >= 0) & (times <= 60)).all()
    ):
        raise ValueError("invalid controls")
    c = np.tile(controls, (len(times), 1))
    c[:, 4] = np.minimum(1, controls[4] + times / (controls[3] * 30))
    return c


def simple_hz(controls):
    # Cylinder/end-correction approximation, not measured liquid-height labels.
    height = controls[:, 0] * 20
    radius = controls[:, 1] * 10
    return 34000 / (4 * (height * (1 - controls[:, 4]) + 0.62 * radius))


def filter_wave(wave, hz, boost=3):
    if (
        wave.shape != (p.SAMPLES,)
        or not np.isfinite(wave).all()
        or not np.isfinite(boost)
        or not 0 <= boost <= 3
    ):
        raise ValueError("invalid texture/filter strength")
    f, times, spectrum = stft(wave, fs=p.RATE, nperseg=FFT, noverlap=FFT - HOP)
    hz = np.asarray(hz)
    if hz.shape != times.shape or not np.isfinite(hz).all() or (hz <= 0).any():
        raise ValueError("invalid resonance curve")
    cents = 1200 * np.log2(np.maximum(f[:, None], 1) / hz[None])
    response = 1 + boost * np.exp(-0.5 * (cents / 150) ** 2)
    result = istft(spectrum * response, fs=p.RATE, nperseg=FFT, noverlap=FFT - HOP)[1][
        : len(wave)
    ]
    result *= np.sqrt(np.mean(wave**2) / max(np.mean(result**2), 1e-20))
    if not np.isfinite(result).all() or abs(result).max() > 0.98:
        raise ValueError("filter exceeds headroom; no automatic attenuation")
    return result.astype(np.float32)


def variants(model, wave, controls):
    times = np.arange(p.SAMPLES // HOP + 1) * HOP / p.RATE
    c = grid(controls, times)
    neural = predict(model, c)
    simple = simple_hz(c)
    return {
        "base": wave,
        "neural": filter_wave(wave, neural),
        "simple": filter_wave(wave, simple),
    }, {
        "time_seconds": times.tolist(),
        "neural_hz": neural.tolist(),
        "simple_hz": simple.tolist(),
    }


def load_head(directory, parent_meta, device="cuda"):
    path = directory / "model.safetensors"
    meta_path = directory / "model.json"
    if path.stat().st_size > 100000 or meta_path.stat().st_size > 100000:
        raise ValueError("oversized head")
    meta = json.loads(meta_path.read_text())
    if (
        meta["format"] != "pour-resonance-head-v1"
        or hashlib.sha256(path.read_bytes()).hexdigest() != meta["checkpoint_sha256"]
        or meta["parent_checkpoint_sha256"] != parent_meta["checkpoint_sha256"]
    ):
        raise ValueError("head/texture identity mismatch")
    state = load_file(path)
    if not all(torch.isfinite(v).all() for v in state.values()):
        raise ValueError("invalid head weights")
    model = ResonanceHead()
    model.load_state_dict(state, strict=True)
    return model.to(device).eval(), meta


def save_wav(path, wave, gain=1):
    if (
        not np.isfinite(wave).all()
        or not np.isfinite(gain)
        or not 0 < gain <= 100
        or abs(wave).max() * gain > 0.98
    ):
        raise ValueError("unsafe audition gain")
    sf.write(path, wave * gain, p.RATE, subtype="PCM_16")
    return {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "audition_gain": gain,
    }


def render(parent, head, output, controls, seed=2718, audition_gain=1, device="cuda"):
    parent, head, output = (x.resolve() for x in (parent, head, output))
    if (
        output.exists()
        or output.is_relative_to(Path(__file__).resolve().parents[2])
        or not 0 <= seed < 2**32
    ):
        raise ValueError("fresh external output and valid seed required")
    grid(controls, np.array([0, p.SAMPLES / p.RATE]))
    texture, parent_meta = compare.load_model(parent, device)
    model, meta = load_head(head, parent_meta, device)
    wave = p.decode(p.sample(texture, controls, seed), 314)
    sounds, curves = variants(model, wave, controls)
    if (
        not np.isfinite(audition_gain)
        or not 0 < audition_gain <= 100
        or max(abs(w).max() for w in sounds.values()) * audition_gain > 0.98
    ):
        raise ValueError(
            "requested audition gain exceeds headroom; raw model unchanged"
        )
    output.mkdir(parents=True)
    rows = [
        {"kind": k, "seed": seed, **save_wav(output / (k + ".wav"), w, audition_gain)}
        for k, w in sounds.items()
    ]
    p.save_json(
        output / "result.json",
        {
            "reference_audio_input": False,
            "teacher_required_at_inference": False,
            "controls": controls.tolist(),
            "head": meta,
            "curves": curves,
            "rows": rows,
        },
    )
    return rows


def run(source, teacher_probe, parent, base_outputs, output, trained_head=None):
    source, teacher_probe, parent, base_outputs, output = (
        x.resolve() for x in (source, teacher_probe, parent, base_outputs, output)
    )
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("fresh external output required")
    torch.set_num_threads(4)
    rows, provenance = p.load_source(source)
    training = compare.select_rows(rows, True)
    teacher = json.loads((teacher_probe / "result.json").read_text())
    references = {
        r["id"]: r
        for r in teacher["rows"]
        if r["variant"] == "original" and r["id"].startswith("container_")
    }
    if set(references) != {r["container_id"] for r in training}:
        raise ValueError("teacher training roster mismatch")
    x, y = [], []
    fractions = np.linspace(0.02, 0.98, 64)
    for row in training:
        ref = references[row["container_id"]]
        input_path = Path(ref["input_wav"])
        if hashlib.sha256(input_path.read_bytes()).hexdigest() != ref["input_sha256"]:
            raise ValueError("teacher input identity mismatch")
        original, sr = sf.read(input_path, dtype="float32")
        if (
            sr != p.RATE
            or len(original) != len(row["wave"])
            or not np.allclose(original, row["wave"], atol=3.1e-5, rtol=0)
        ):
            raise ValueError("teacher/source alignment mismatch")
        c = p.condition(
            row["dimensions"], row["material"], row["shape"], row["duration"], 0
        )
        c = np.tile(c, (64, 1))
        c[:, 4] = fractions
        hz = np.interp(
            fractions * row["duration"], ref["time_seconds"], ref["axial_hz"]
        )
        x.append(c)
        y.append((np.log2(34000 / hz) - 5) / 2)
    x, y = np.stack(x), np.stack(y)
    _, parent_meta = compare.load_model(parent, "cpu")
    if parent_meta["source_sha256"] != hashlib.sha256(
        (source / "source.json").read_bytes()
    ).hexdigest() or parent_meta["train_ids"] != [
        r["item_id"] for r in rows if r["role"] == "train"
    ]:
        raise ValueError("parent/source exposure mismatch")
    if trained_head is None:
        model, history = fit(x, y)
    else:
        model, previous = load_head(trained_head.resolve(), parent_meta)
        if (
            previous["train_ids"] != [r["item_id"] for r in training]
            or previous["source_sha256"] != parent_meta["source_sha256"]
            or previous["teacher_result_sha256"]
            != hashlib.sha256((teacher_probe / "result.json").read_bytes()).hexdigest()
        ):
            raise ValueError("reused head exposure mismatch")
        history = previous["history"]
    output.mkdir(parents=True)
    save_file(
        {k: v.detach().cpu().contiguous() for k, v in model.state_dict().items()},
        output / "model.safetensors",
    )
    meta = {
        "format": "pour-resonance-head-v1",
        "checkpoint_sha256": hashlib.sha256(
            (output / "model.safetensors").read_bytes()
        ).hexdigest(),
        "parent_checkpoint_sha256": parent_meta["checkpoint_sha256"],
        "source_sha256": parent_meta["source_sha256"],
        "teacher_result_sha256": hashlib.sha256(
            (teacher_probe / "result.json").read_bytes()
        ).hexdigest(),
        "teacher_model": teacher["model"],
        "train_ids": [r["item_id"] for r in training],
        "parameters": sum(v.numel() for v in model.parameters()),
        "steps": STEPS,
        "seed": 53,
        "history": history,
        "scope": "uncertain full-record teacher trajectories;13 recordings/objects; no true physical labels",
        "filter": {
            "fft": FFT,
            "hop": HOP,
            "sigma_cents": 150,
            "boost": 3,
            "rms_preserved": True,
        },
    }
    p.save_json(output / "model.json", meta)
    report = {
        "status": "running",
        "terms": provenance["terms"],
        "model": meta,
        "reference_audio_input_to_generator": False,
        "reused_head_checkpoint": str(trained_head)
        if trained_head is not None
        else None,
        "novel": [],
        "group_validation": [],
        "rows": [],
    }
    # Produce the primary artifact before additional validation infrastructure.
    for name, height, material, duration in [
        ("glass10", 10, "glass", 15),
        ("glass16", 16, "glass", 15),
        ("pet10", 10, "plastic_pet", 15),
        ("glass10fast", 10, "glass", 8),
    ]:
        c = p.condition(
            {"net_height": height, "diameter_top": 7, "diameter_bottom": 7},
            material,
            "cylindrical",
            duration,
            0.1,
        )
        for seed in (314, 2718, 1618):
            report["novel"].append(
                {
                    "name": name,
                    "seed": seed,
                    "controls": c.tolist(),
                    "rows": render(parent, output, output / f"{name}-{seed}", c, seed),
                }
            )
        print({"reference_free_profile": name}, flush=True)
    preview = []
    for record in report["novel"]:
        if record["seed"] == 2718:
            for wav in record["rows"]:
                wave, _ = sf.read(wav["wav"], dtype="float32")
                preview.extend([wave, np.zeros(p.RATE // 2)])
    report["comparison"] = {
        **save_wav(output / "comparison.wav", np.concatenate(preview)),
        "order": "glass10/glass16/PET10/glass10fast; base/neural/simple; seed2718; audition gain1",
    }
    p.save_json(output / "result.json", report)
    print({"primary_artifact": report["comparison"]["wav"]}, flush=True)
    # Object-group exclusion applies to this head, not the published teacher.
    for index, row in enumerate(training):
        mask = np.arange(len(training)) != index
        held, _ = fit(x[mask], y[mask])
        truth = 34000 / (2 ** (y[index] * 2 + 5))
        candidates = {
            "neural": predict(held, x[index]),
            "simple": simple_hz(x[index]),
            "train_mean": 34000 / (2 ** (y[mask].mean(0) * 2 + 5)),
        }
        report["group_validation"].append(
            {
                "container_id": row["container_id"],
                "median_error_cents": {
                    k: float(np.median(abs(1200 * np.log2(v / truth))))
                    for k, v in candidates.items()
                },
            }
        )
        print({"excluded_group": index + 1, "total": len(training)}, flush=True)
    base_report = json.loads((base_outputs / "result.json").read_text())
    if base_report["parent"]["checkpoint_sha256"] != parent_meta["checkpoint_sha256"]:
        raise ValueError("cached texture identity mismatch")
    for index, row in enumerate(base_report["rows"]):
        if row["kind"] not in ("base", "real"):
            continue
        path = Path(row["wav"])
        if hashlib.sha256(path.read_bytes()).hexdigest() != row["sha256"]:
            raise ValueError("cached waveform mismatch")
        if row["kind"] == "real":
            report["rows"].append(row)
            continue
        wave, sr = sf.read(path, dtype="float32")
        if sr != p.RATE:
            raise ValueError("wrong cached rate")
        reference = next(
            r
            for r in base_report["rows"]
            if r["kind"] == "real"
            and r["item_id"] == row["item_id"]
            and r["phase"] == row["phase"]
        )
        real, _ = sf.read(reference["wav"], dtype="float32")
        sounds, _ = variants(model, wave, np.array(row["controls"], dtype=np.float32))
        for kind, sound in sounds.items():
            artifact = (
                {k: row[k] for k in ["wav", "sha256", "audition_gain"]}
                if kind == "base"
                else save_wav(output / f"development-{index}-{kind}.wav", sound)
            )
            report["rows"].append(
                {**row, "kind": kind, **artifact, **p.metrics(sound, real)}
            )
    report["status"] = "complete"
    p.save_json(output / "result.json", report)
    p.save_json(
        output / "tag-input.json",
        {
            "status": "complete",
            "seconds": p.SAMPLES / p.RATE,
            "cases": [
                {"id": f"{i}-{r['kind']}", "diagnostic_id": "water-pour"}
                for i, r in enumerate(report["rows"])
            ],
            "rows": [{**r, "case": i} for i, r in enumerate(report["rows"])],
            "controls": [],
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "teacher-probe", "parent", "base-outputs", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument(
        "--trained-head",
        type=Path,
        help="reuse exact full-fit weights; new output still required",
    )
    args = parser.parse_args()
    run(
        args.source,
        args.teacher_probe,
        args.parent,
        args.base_outputs,
        args.output,
        args.trained_head,
    )
