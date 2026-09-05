"""Report-only SyncFusion: text plus explicit event times, never target audio."""

import argparse
import hashlib
import json
import time
import zipfile
from pathlib import Path

import numpy as np
import physical_sound_mmaudio_timing as timing
import soundfile as sf
import torch

RATE = 48000
LENGTH = 262144
MEMBER = "ckpt-diffusion/epoch=784-valid_loss=0.008.ckpt"
URL = "https://zenodo.org/api/records/12634630/files/ckpt-diffusion.zip/content"
CASES = [
    ("glass", "A drumstick taps glass.", [0.6, 1.5, 2.7, 4.0]),
    ("delayed", "A drumstick taps glass.", [1.0, 1.9, 3.1, 4.4]),
    ("wood", "A drumstick taps wood.", [0.6, 1.5, 2.7, 4.0]),
    ("empty", "A drumstick taps glass.", []),
]


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def prepare(root):
    import fsspec

    root.mkdir(parents=True, exist_ok=False)
    report = {
        "status": "downloading",
        "url": URL,
        "license": "CC-BY-4.0",
        "member": MEMBER,
        "whole_archive_md5_verified": False,
    }
    save(root / "source.json", report)
    # Only the selected checkpoint member; no last checkpoint, visual model or test data.
    with (
        fsspec.open(
            URL, "rb", block_size=32 * 1024**2, cache_type="readahead"
        ) as remote,
        zipfile.ZipFile(remote) as archive,
    ):
        info = archive.getinfo(MEMBER)
        if info.file_size != 3223219889 or info.CRC != 1701792866:
            raise ValueError("Publisher member identity changed")
        with (
            archive.open(info) as source,
            (root / "model.ckpt").open("xb") as target,
        ):
            count = 0
            next_log = 0
            while chunk := source.read(1024**2):
                target.write(chunk)
                count += len(chunk)
                if count >= next_log:
                    print("checkpoint bytes", count, flush=True)
                    next_log += 256 * 1024**2
            if count != info.file_size:
                raise ValueError("Incomplete checkpoint")
    report.update(
        status="complete",
        bytes=count,
        member_crc32=info.CRC,
        checkpoint_sha256=sha(root / "model.ckpt"),
    )
    save(root / "source.json", report)


def event_track(times):
    times = np.asarray(times, dtype=np.float64)
    if times.ndim != 1 or len(times) > 32 or not np.isfinite(times).all():
        raise ValueError("Bounded finite event list required")
    if np.any(times < 0) or np.any(times >= LENGTH / RATE):
        raise ValueError("Events outside generated interval")
    indices = (times * RATE).astype(np.int64)
    if len(np.unique(indices)) != len(indices):
        raise ValueError("Duplicate event samples")
    track = torch.zeros(1, 1, LENGTH)
    track[0, 0, indices] = 1
    return track


def build():
    from audio_diffusion_pytorch import DiffusionModel, UNetV0
    from audio_encoders_pytorch import Encoder1d

    # Exact published exp/model/diffusion.yaml; no Lightning, dataset or audio embedder.
    model = DiffusionModel(
        net_t=UNetV0,
        in_channels=1,
        channels=[8, 32, 64, 128, 256, 512, 1024, 1024],
        factors=[1, 4, 4, 4, 2, 2, 2, 2],
        items=[1, 2, 2, 2, 2, 2, 2, 4],
        attentions=[0, 0, 0, 0, 1, 1, 1, 1],
        attention_heads=8,
        attention_features=64,
        context_channels=[2, 8, 16, 32, 64, 128, 256, 256],
        use_embedding_cfg=True,
        embedding_max_length=1,
        embedding_features=512,
        cross_attentions=[1] * 8,
    )
    encoder = Encoder1d(
        in_channels=1,
        channels=2,
        multipliers=[1, 1, 4, 8, 16, 32, 64, 128, 128],
        factors=[1, 4, 4, 4, 2, 2, 2, 2],
        num_blocks=[2] * 8,
        resnet_groups=2,
        patch_size=1,
    )
    return model.eval().requires_grad_(False), encoder.eval().requires_grad_(False)


def subset(state, prefix):
    selected = {
        k.removeprefix(prefix): v for k, v in state.items() if k.startswith(prefix)
    }
    if not selected or not all(
        isinstance(v, torch.Tensor) and torch.isfinite(v).all()
        for v in selected.values()
    ):
        raise ValueError("Missing or nonfinite checkpoint tensors")
    return selected


def finalize(output):
    """Publish safe cases and retain rejection; never redo inference or clip failures."""
    result = json.loads((output / "result.json").read_text())
    if result["status"].startswith("complete"):
        raise ValueError("Already finalized; do not overwrite evidence")
    previous = {r["id"]: r for r in result["rows"]}
    rows, waves = [], []
    for name, prompt, times in CASES:
        raw = output / f"{name}-raw.wav"
        if name in previous and sha(raw) != previous[name]["raw_sha256"]:
            raise ValueError("Changed previously generated raw signal")
        wave, rate = sf.read(raw, dtype="float32")
        if rate != RATE or wave.shape != (LENGTH,):
            raise ValueError("Invalid raw waveform layout")
        finite = bool(np.isfinite(wave).all())
        peak = float(np.abs(wave).max()) if finite else None
        row = {
            "id": name,
            "prompt": prompt,
            "events_seconds": times,
            "raw_sha256": sha(raw),
            "raw_peak": peak,
            "raw_rms": float(np.sqrt(np.mean(wave**2))) if finite else None,
            "playback_gain": 0.5,
            "seconds": previous.get(name, {}).get("seconds"),
        }
        if finite and peak * 0.5 < 0.98:
            playback = wave * 0.5
            path = output / f"{name}.wav"
            if path.exists():
                if name not in previous or sha(path) != previous[name].get("sha256"):
                    raise ValueError("Changed previously published PCM identity")
                pcm, sr = sf.read(path, dtype="float32")
                if (
                    sr != RATE
                    or pcm.shape != playback.shape
                    or np.max(np.abs(pcm - playback)) > 1 / 32768
                ):
                    raise ValueError("Changed previously published PCM")
            else:
                sf.write(path, playback, RATE, subtype="PCM_16")
            attacks = timing.onsets(playback, RATE)
            row.update(
                status="published",
                wav=str(path),
                sha256=sha(path),
                detected_onsets_s=attacks.tolist(),
                timing=timing.match(times, attacks),
            )
            waves.extend([playback, np.zeros(RATE // 2)])
        else:
            row.update(
                status="rejected",
                reason="nonfinite or fixed-gain headroom failure; no PCM published",
            )
        rows.append(row)
    sf.write(output / "comparison.wav", np.concatenate(waves), RATE, subtype="PCM_16")
    result.update(
        status="complete_with_rejections"
        if any(r["status"] == "rejected" for r in rows)
        else "complete",
        rows=rows,
        comparison_order=[r["id"] for r in rows if r["status"] == "published"],
        claim="Explicit-time research audition; no calibrated physical/material/generalization claim",
    )
    save(output / "result.json", result)


def assess(output):
    import physical_sound_text_tags as tags

    result = json.loads((output / "result.json").read_text())
    if not result["status"].startswith("complete"):
        raise ValueError("Finalize first")
    rows = [r for r in result["rows"] if r["status"] == "published"]
    manifest = {
        "status": "complete",
        "seconds": LENGTH / RATE,
        "controls": [],
        "cases": [
            {
                "id": r["id"],
                "diagnostic_id": "wood-wood" if r["id"] == "wood" else "glass-wood",
            }
            for r in rows
        ],
        "rows": [
            {"case": i, "wav": r["wav"], "sha256": r["sha256"], "seed": 42}
            for i, r in enumerate(rows)
        ],
        "excluded_rejected_cases": [
            r["id"] for r in result["rows"] if r["status"] != "published"
        ],
        "result_sha256": sha(output / "result.json"),
    }
    path = output / "tag-inputs.json"
    if path.exists():
        raise ValueError("Assessment already exists")
    save(path, manifest)
    for name, rms in (("raw", None), ("rms", 0.005)):
        tags.run(path, output / f"tags-{name}.json", [], ast_rms=rms)


def text_conditions(state, prompts):
    """Published frozen CLAP text path, reusable without loading its audio tower."""
    from transformers import RobertaConfig, RobertaModel, RobertaTokenizer

    # RoBERTa-base config, checked against FacebookAI/roberta-base e2da8e2f... .
    # Same LAION-CLAP text path: pooler -> Linear/ReLU/Linear -> L2 normalize.
    config = RobertaConfig(
        vocab_size=50265,
        max_position_embeddings=514,
        type_vocab_size=1,
        layer_norm_eps=1e-5,
    )
    config._attn_implementation = "eager"
    text_model = RobertaModel(config).eval().requires_grad_(False)
    text_state = subset(state, "clap.model.text_branch.")
    positions = text_state.pop("embeddings.position_ids")
    if not torch.equal(positions, torch.arange(514)[None]):
        raise ValueError("Unexpected legacy position buffer")
    text_model.load_state_dict(text_state, strict=True)
    projection = torch.nn.Sequential(
        torch.nn.Linear(768, 512), torch.nn.ReLU(), torch.nn.Linear(512, 512)
    )
    projection.load_state_dict(
        subset(state, "clap.model.text_projection."), strict=True
    )
    projection.eval().requires_grad_(False)
    tokenizer = RobertaTokenizer.from_pretrained(
        "cvssp/audioldm2",
        subfolder="tokenizer",
        revision="c8e7e189d324425c05c4c2f81214041ef4107983",
        local_files_only=True,
    )
    # The cached RoBERTa vocabulary is reused; no AudioLDM weights or audio encoder.
    embeddings = []
    with torch.inference_mode():
        for prompt in prompts:
            tokens = tokenizer(
                [prompt],
                padding="max_length",
                truncation=True,
                max_length=77,
                return_tensors="pt",
            )
            pooled = text_model(
                input_ids=tokens.input_ids, attention_mask=tokens.attention_mask
            ).pooler_output
            embeddings.append(
                torch.nn.functional.normalize(projection(pooled), dim=-1).unsqueeze(1)
            )
    return torch.cat(embeddings)


def render(root, output):
    if output.exists():
        raise ValueError("Refusing to overwrite an existing experiment")
    report = json.loads((root / "source.json").read_text())
    if (
        report["status"] != "complete"
        or sha(root / "model.ckpt") != report["checkpoint_sha256"]
    ):
        raise ValueError("Incomplete or changed checkpoint")
    torch.set_num_threads(4)
    torch.manual_seed(42)
    checkpoint = torch.load(
        root / "model.ckpt", weights_only=True, map_location="cpu", mmap=True
    )
    state = checkpoint["state_dict"]
    model, encoder = build()
    model.load_state_dict(subset(state, "model."), strict=True)
    encoder.load_state_dict(subset(state, "onsets_encoder."), strict=True)
    cases = CASES
    embeddings = text_conditions(state, [prompt for _, prompt, _ in cases]).split(1)
    del checkpoint, state
    model.cuda()
    encoder.cuda()
    output.mkdir(parents=True, exist_ok=False)
    result = {
        "status": "running",
        "checkpoint_sha256": report["checkpoint_sha256"],
        "reference_audio_input": False,
        "video_input": False,
        "training_updates": 0,
        "seed": 42,
        "steps": 150,
        "embedding_scale": 2,
        "sample_rate": RATE,
        "samples": LENGTH,
        "precision": "float32",
        "torch": torch.__version__,
        "cut_prefix": False,
        "post_generation_event_gating": False,
        "rows": [],
    }
    save(output / "result.json", result)
    for (name, prompt, times), embedding in zip(cases, embeddings, strict=True):
        started = time.monotonic()
        with torch.inference_mode():
            _, context = encoder(event_track(times).cuda(), with_info=True)
            noise = torch.randn(
                1,
                1,
                LENGTH,
                device="cuda",
                generator=torch.Generator(device="cuda").manual_seed(42),
            )
            audio = model.sample(
                x_noisy=noise,
                num_steps=150,
                channels=context["xs"][2:-1],
                embedding=embedding.cuda(),
                embedding_scale=2.0,
            )
        wave = audio[0, 0].cpu().numpy()
        sf.write(output / f"{name}-raw.wav", wave, RATE, subtype="FLOAT")
        row = {
            "id": name,
            "prompt": prompt,
            "events_seconds": times,
            "raw_sha256": sha(output / f"{name}-raw.wav"),
            "seconds": time.monotonic() - started,
        }
        result["rows"].append(row)
        save(output / "result.json", result)
        print(json.dumps(row), flush=True)
    result.update(cuda_peak_gib=torch.cuda.max_memory_allocated() / 2**30)
    save(output / "result.json", result)
    finalize(output)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("prepare", "render", "finalize", "assess"))
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.mode != "prepare" and args.output is None:
        parser.error("--output is required except for prepare")
    for path in (args.root, args.output):
        if path is not None and path.resolve().is_relative_to(
            Path(__file__).resolve().parents[2]
        ):
            raise ValueError("Generated assets must stay outside Git")
    if args.mode == "prepare":
        prepare(args.root)
    elif args.mode == "finalize":
        finalize(args.output)
    elif args.mode == "assess":
        assess(args.output)
    else:
        render(args.root, args.output)
