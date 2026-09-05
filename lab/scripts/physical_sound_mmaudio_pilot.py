"""External, research-only MMAudio video/text counterfactual; no target audio input."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import subprocess
import time
from pathlib import Path
from unittest.mock import patch

import numpy as np
import soundfile as sf

SOURCE_REV = "974010a026c731054592d8f777218bd9d85a6c24"
WATER_REV = "12575460ee39d6adaebbe5aff531a5f4a24a627b"
ASSETS = {
    "model": (
        "hkchengrex/MMAudio",
        "eb13a1a98fdbec91753775c57b074ccdfc60587c",
        [
            "README.md",
            "weights/mmaudio_large_44k_v2.pth",
            "ext_weights/v1-44.pth",
            "ext_weights/synchformer_state_dict.pth",
        ],
    ),
    "clip": (
        "apple/DFN5B-CLIP-ViT-H-14-384",
        "01b771ed0d1395ca5ffdd279897d665ebe00dfd2",
        [
            "README.md",
            "LICENSE",
            "open_clip_config.json",
            "open_clip_pytorch_model.bin",
        ],
    ),
    "vocoder": (
        "nvidia/bigvgan_v2_44khz_128band_512x",
        "95a9d1dcb12906c03edd938d77b9333d6ded7dfb",
        ["README.md", "LICENSE", "config.json", "bigvgan_generator.pt"],
    ),
}


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, data):
    path.write_text(json.dumps(data, indent=2) + "\n")


def probe(path):
    return json.loads(
        subprocess.check_output(
            ["ffprobe", "-v", "error", "-show_streams", "-of", "json", str(path)]
        )
    )


def require_silent_video(path):
    streams = probe(path)["streams"]
    if len(streams) != 1 or streams[0]["codec_type"] != "video":
        raise ValueError("Generator input must contain one video stream and NO audio")


def publish(path, wave, playback_gain=1.0):
    if wave.ndim != 1 or not 7 * 44100 <= wave.size <= 9 * 44100:
        raise ValueError("Expected approximately eight seconds of mono audio")
    if not np.isfinite(playback_gain) or not 0 < playback_gain <= 1:
        raise ValueError("Only explicit attenuation is allowed")
    if not np.isfinite(wave).all() or np.max(np.abs(wave * playback_gain)) >= 0.98:
        raise ValueError(
            "Invalid waveform or headroom; no silent clipping/normalization"
        )
    sf.write(path, wave * playback_gain, 44100, subtype="PCM_16")
    sf.write(path.with_stem(path.stem + "-float"), wave, 44100, subtype="FLOAT")
    return {
        "path": str(path),
        "sha256": digest(path),
        "samples": wave.size,
        "peak": float(np.abs(wave).max()),
        "playback_gain": playback_gain,
        "rms": float(np.sqrt(np.mean(wave**2))),
    }


def prepare(root, water):
    from huggingface_hub import HfApi, hf_hub_download

    root.mkdir(parents=True, exist_ok=False)
    receipt = {"status": "preparing", "research_only": True, "assets": {}}
    write_json(root / "assets.json", receipt)
    for key, (repo, rev, names) in ASSETS.items():
        metadata = HfApi().model_info(repo, revision=rev, files_metadata=True)
        files = {f.rfilename: f for f in metadata.siblings}
        for name in names:
            print("download", repo, name, files[name].size, flush=True)
            path = Path(hf_hub_download(repo, name, revision=rev, local_dir=root / key))
            sha = digest(path)
            info = files[name]
            if path.stat().st_size != info.size or (
                info.lfs and sha != info.lfs.sha256
            ):
                raise ValueError("Asset identity mismatch")
            receipt["assets"][f"{key}/{name}"] = {
                "repo": repo,
                "revision": rev,
                "sha256": sha,
                "bytes": info.size,
            }
            write_json(root / "assets.json", receipt)
    # Existing publisher TRAIN, already opened for pouring experiments. Not a clean test.
    opened = json.loads((water / "source.json").read_text())
    known = {
        Path(f["path"]).stem for f in opened["files"] if f["path"].startswith("audios/")
    }
    with (water / "splits/train.csv").open() as stream:
        row = next(
            r
            for r in csv.DictReader(stream)
            if r["item_id"] in known
            and r["material"] == "glass"
            and r["clean"] == "yes"
            and r["bg-noise"] == "no"
            and r["container_id"] not in ("container_18", "container_30")
            and float(r["end_time"]) - float(r["start_time"]) >= 8
        )
    video = Path(
        hf_hub_download(
            "bpiyush/sound-of-water",
            row["file_name"],
            repo_type="dataset",
            revision=WATER_REV,
            local_dir=root / "water",
        )
    )
    silent = root / "water-silent.mp4"
    subprocess.run(
        [
            "ffmpeg",
            "-v",
            "error",
            "-nostdin",
            "-n",
            "-i",
            str(video),
            "-t",
            "8",
            "-map",
            "0:v:0",
            "-an",
            "-vf",
            "scale=512:-2",
            "-c:v",
            "libx264",
            "-crf",
            "18",
            str(silent),
        ],
        check=True,
    )
    require_silent_video(silent)
    receipt.update(
        status="complete",
        water={
            "row": row,
            "revision": WATER_REV,
            "source_sha256": digest(video),
            "silent_sha256": digest(silent),
            "role": "opened publisher TRAIN; pretraining overlap unknown",
            "terms": "dataset redistribution unspecified; local research only",
        },
    )
    write_json(root / "assets.json", receipt)


def render(root, source, output):
    import torch
    from mmaudio.eval_utils import generate, load_video
    from mmaudio.ext.bigvgan_v2.bigvgan import BigVGAN
    from mmaudio.model.flow_matching import FlowMatching
    from mmaudio.model.networks import get_my_mmaudio
    from mmaudio.model.sequence_config import CONFIG_44K
    from mmaudio.model.utils import features_utils

    receipt = json.loads((root / "assets.json").read_text())
    if receipt["status"] != "complete":
        raise ValueError("Incomplete assets")
    actual = subprocess.check_output(
        ["git", "-C", str(source), "rev-parse", "HEAD"], text=True
    ).strip()
    if actual != SOURCE_REV:
        raise ValueError("Unexpected upstream revision")
    for key, item in receipt["assets"].items():
        if digest(root / key) != item["sha256"]:
            raise ValueError(f"Changed asset: {key}")
    silent = root / "water-silent.mp4"
    require_silent_video(silent)
    if digest(silent) != receipt["water"]["silent_sha256"]:
        raise ValueError("Changed silent input")
    output.mkdir(parents=True, exist_ok=False)
    result = {
        "status": "running",
        "upstream_revision": actual,
        "assets_sha256": digest(root / "assets.json"),
        "seed": 42,
        "steps": 25,
        "cfg": 4.5,
        "training_updates": 0,
        "reference_audio_input": False,
        "prompt": "Water is being poured into a glass container.",
        "negative_prompt": "",
        "torch": torch.__version__,
        "arms": {},
    }
    write_json(output / "result.json", result)
    torch.set_num_threads(4)
    torch.backends.cuda.matmul.allow_tf32 = True
    torch.backends.cudnn.allow_tf32 = True
    dtype = torch.bfloat16
    model = root / "model"
    net = get_my_mmaudio("large_44k_v2").eval().requires_grad_(False)
    net.load_weights(
        torch.load(
            model / "weights/mmaudio_large_44k_v2.pth",
            map_location="cpu",
            weights_only=True,
        )
    )
    net.to("cuda", dtype)
    clip_factory = features_utils.create_model_from_pretrained
    vocoder_factory = BigVGAN.from_pretrained
    # Redirect only asset locations, not model algorithms; no network during rendering.
    with (
        patch.object(
            features_utils,
            "create_model_from_pretrained",
            lambda *a, **kw: clip_factory(f"local-dir:{root / 'clip'}", **kw),
        ),
        patch.object(
            BigVGAN,
            "from_pretrained",
            lambda *a, **kw: vocoder_factory(root / "vocoder", **kw),
        ),
    ):
        features = features_utils.FeaturesUtils(
            tod_vae_ckpt=model / "ext_weights/v1-44.pth",
            synchformer_ckpt=model / "ext_weights/synchformer_state_dict.pth",
            enable_conditions=True,
            mode="44k",
            need_vae_encoder=False,
        )
    features.eval().requires_grad_(False).to("cuda", dtype)
    video = load_video(silent, 8, load_all_frames=False)
    CONFIG_44K.duration = video.duration_sec
    net.update_seq_lengths(
        CONFIG_44K.latent_seq_len, CONFIG_44K.clip_seq_len, CONFIG_44K.sync_seq_len
    )
    fm = FlowMatching(min_sigma=0, inference_mode="euler", num_steps=25)
    waves = []
    # Same seed/prompt/sampler. Static preserves first-frame appearance but removes motion.
    for arm in ("text", "video", "static"):
        start = time.monotonic()
        clip, sync = None, None
        if arm != "text":
            clip, sync = video.clip_frames, video.sync_frames
            if arm == "static":
                clip = clip[:1].expand_as(clip)
                sync = sync[:1].expand_as(sync)
            clip, sync = clip.unsqueeze(0), sync.unsqueeze(0)
        with torch.inference_mode():
            audio = generate(
                clip,
                sync,
                [result["prompt"]],
                negative_text=[""],
                feature_utils=features,
                net=net,
                fm=fm,
                rng=torch.Generator(device="cuda").manual_seed(42),
                cfg_strength=4.5,
                clip_batch_size_multiplier=2,
                sync_batch_size_multiplier=1,
            )
        wave = audio[0].float().cpu().numpy().reshape(-1)
        print(
            arm,
            "raw shape/finite/peak",
            wave.shape,
            bool(np.isfinite(wave).all()),
            float(np.max(np.abs(wave))),
            flush=True,
        )
        sf.write(output / f"{arm}-raw-diagnostic.wav", wave, 44100, subtype="FLOAT")
        item = publish(output / f"{arm}.wav", wave, playback_gain=0.5)
        item["seconds"] = time.monotonic() - start
        result["arms"][arm] = item
        waves.append(wave * 0.5)
        if arm == "video":
            subprocess.run(
                [
                    "ffmpeg",
                    "-v",
                    "error",
                    "-nostdin",
                    "-n",
                    "-i",
                    str(silent),
                    "-i",
                    str(output / "video.wav"),
                    "-map",
                    "0:v:0",
                    "-map",
                    "1:a:0",
                    "-c:v",
                    "copy",
                    "-c:a",
                    "aac",
                    str(output / "water-generated-full.mp4"),
                ],
                check=True,
            )
        write_json(output / "result.json", result)
        print(arm, item, flush=True)
    sf.write(
        output / "comparison.wav",
        np.concatenate([x for w in waves for x in (w, np.zeros(22050))]),
        44100,
        subtype="PCM_16",
    )
    result.update(
        status="complete",
        cuda_peak_gib=torch.cuda.max_memory_allocated() / 2**30,
        comparison_order=["text", "video", "static"],
        claim="Playable counterfactual, not quality/physical calibration or test generalization",
    )
    write_json(output / "result.json", result)


def assess(root, output):
    """Post-generation coarse event check; reference never enters render()."""
    import physical_sound_text_tags as tags

    result_path = output / "result.json"
    result = json.loads(result_path.read_text())
    assets = json.loads((root / "assets.json").read_text())
    if result["status"] != "complete":
        raise ValueError("Generation must finish first")
    source = root / "water" / assets["water"]["row"]["file_name"]
    if digest(source) != assets["water"]["source_sha256"]:
        raise ValueError("Changed evaluation source")
    reference = output / "reference-evaluation-only.wav"
    subprocess.run(
        [
            "ffmpeg",
            "-v",
            "error",
            "-nostdin",
            "-n",
            "-i",
            str(source),
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
            str(reference),
        ],
        check=True,
    )
    records = [
        (key, Path(item["path"]), item["sha256"])
        for key, item in result["arms"].items()
    ]
    records.append(("reference", reference, digest(reference)))
    manifest = {
        "status": "complete",
        "seconds": 8,
        "controls": [],
        "cases": [{"id": key, "diagnostic_id": "water-pour"} for key, _, _ in records],
        "rows": [
            {"case": i, "wav": str(path), "sha256": sha, "seed": 42}
            for i, (_, path, sha) in enumerate(records)
        ],
        "generator_result_sha256": digest(result_path),
        "reference_role": "opened TRAIN, post-generation diagnostic only",
    }
    manifest_path = output / "tag-inputs.json"
    write_json(manifest_path, manifest)
    for name, rms in (("raw", None), ("rms", 0.005)):
        tags.run(manifest_path, output / f"tags-{name}.json", [], ast_rms=rms)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("prepare", "render", "assess"))
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--water", type=Path)
    parser.add_argument("--source", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[2]
    for path in (args.root, args.output):
        if path is not None and path.resolve().is_relative_to(repository):
            raise ValueError("Assets and generated media must remain outside Git")
    if args.mode == "prepare":
        prepare(args.root, args.water)
    elif args.mode == "assess":
        assess(args.root, args.output)
    else:
        os.environ["HF_HUB_OFFLINE"] = "1"
        # Do not leave a terminal failure mislabeled as a live run.
        existed = args.output.exists()
        try:
            render(args.root, args.source, args.output)
        except Exception as exc:
            result_path = args.output / "result.json"
            if not existed and result_path.exists():
                result = json.loads(result_path.read_text())
                result.update(status="failed", error=f"{type(exc).__name__}: {exc}")
                write_json(result_path, result)
            raise


if __name__ == "__main__":
    main()
