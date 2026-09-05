"""Text-only TangoFlux baseline on the existing physical-sound pilot cases.

Powered by Stability AI. TangoFlux / Hung et al. is non-commercial research
only; see https://huggingface.co/declare-lab/TangoFlux. No engine admission.
Executes only the inspected, hash-pinned upstream model implementation.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import shutil
import time
from pathlib import Path

import numpy as np
import physical_sound_text_pilot as pilot
from scipy.io import wavfile
from scipy.signal import resample_poly

MODEL = "declare-lab/TangoFlux"
REVISION = "367005e963cb3a9fb2e03a46104d7de23e34ceea"
CODE_HASH = "209cfe8de77e39e935668b4e13ddb226ea2842b01d56d59898f970067de3481d"
RATE = 44100


def load_cases(path: Path | None):
    if path is None:
        return [{"id": key, "prompt": prompt} for key, prompt in pilot.CASES]
    if not 0 < path.stat().st_size <= 65536:
        raise ValueError("bounded prompt file required")
    rows = json.loads(path.read_text())
    if not isinstance(rows, list) or not 1 <= len(rows) <= 32:
        raise ValueError("one to 32 prompt cases required")
    seen = {"empty-prompt"}
    for row in rows:
        if not isinstance(row, dict) or set(row) - {"id", "prompt", "diagnostic_id"}:
            raise ValueError("unknown prompt fields")
        key, prompt = row.get("id"), row.get("prompt")
        if (
            not isinstance(key, str)
            or not re.fullmatch(r"[a-z][a-z0-9_-]{0,63}", key)
            or key in seen
        ):
            raise ValueError("unique safe case identifiers required")
        if (
            not isinstance(prompt, str)
            or not prompt.strip()
            or len(prompt) > 512
            or not prompt.isprintable()
        ):
            raise ValueError("nonempty bounded printable prompt required")
        if "diagnostic_id" in row:
            from physical_sound_text_tags import EXPECTED

            if (
                not isinstance(row["diagnostic_id"], str)
                or row["diagnostic_id"] not in EXPECTED
            ):
                raise ValueError("unknown diagnostic category")
        seen.add(key)
    return rows


def verify_weight_keys(missing: list[str], unexpected: list[str]):
    # The released safetensors omit only the tied T5 embedding alias.
    if missing != ["text_encoder.encoder.embed_tokens.weight"] or unexpected:
        raise ValueError(f"incomplete model weights: {missing}, {unexpected}")


def validate_adapter_weights(weights, expected):
    import torch

    if not expected or set(weights) != set(expected):
        raise ValueError("adapter key coverage mismatch")
    for name, value in weights.items():
        if (
            "lora_" not in name
            or value.shape != expected[name].shape
            or value.dtype != torch.float32
            or not torch.isfinite(value).all()
        ):
            raise ValueError(f"invalid adapter tensor: {name}")


def load_adapter(model, checkpoint: Path) -> dict:
    """Reload this experiment's adapter without accepting arbitrary base weights."""
    from peft import LoraConfig
    from safetensors.torch import load_file

    checkpoint = checkpoint.resolve()
    if not 0 < checkpoint.stat().st_size <= 16 * 1024**2:
        raise ValueError("adapter outside bounded size")
    report = json.loads((checkpoint.parent / "result.json").read_text())
    if report["status"] != "complete" or (report["model"], report["revision"]) != (
        MODEL,
        REVISION,
    ):
        raise ValueError("incomplete or incompatible adapter run")
    if any(
        report["adapter"][key] != value
        for key, value in (
            ("rank", 8),
            ("alpha", 8),
            ("target_modules", ["to_q", "to_v"]),
        )
    ):
        raise ValueError("unsupported adapter configuration")
    digest = hashlib.sha256(checkpoint.read_bytes()).hexdigest()
    matches = [
        row for row in report["checkpoints"] if row.get("adapter_sha256") == digest
    ]
    if len(matches) != 1:
        raise ValueError("checkpoint absent or ambiguous in training report")
    model.requires_grad_(False)
    model.transformer.add_adapter(
        LoraConfig(r=8, lora_alpha=8, target_modules=["to_q", "to_v"])
    )
    expected = {name: p for name, p in model.named_parameters() if p.requires_grad}
    weights = load_file(checkpoint)
    validate_adapter_weights(weights, expected)
    model.load_state_dict(weights, strict=False)
    model.requires_grad_(False)
    return {
        "path": str(checkpoint),
        "sha256": digest,
        "training_steps": matches[0]["step"],
    }


def load_models(output: Path):
    import torch
    from diffusers import AutoencoderOobleck
    from huggingface_hub import snapshot_download
    from safetensors.torch import load_file

    source = Path(snapshot_download(MODEL, revision=REVISION, local_files_only=True))
    code = source / "model.py"
    if hashlib.sha256(code.read_bytes()).hexdigest() != CODE_HASH:
        raise ValueError("unreviewed upstream model code")
    spec = importlib.util.spec_from_file_location("tangoflux_reviewed_model", code)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    # Reuse the cached T5 scaffold/tokenizer, avoiding a second full T5 download.
    # Every actual T5 weight is then replaced from TangoFlux, with exact key
    # coverage and tied-embedding identity checked below. No AudioLDM network
    # weights survive as an unreported part of this generator.
    scaffold = output / "t5-scaffold"
    scaffold.mkdir()
    base = Path(
        snapshot_download(pilot.MODEL, revision=pilot.REVISION, local_files_only=True)
    )
    for directory, names in (
        ("text_encoder_2", ("config.json", "model.safetensors")),
        (
            "tokenizer_2",
            (
                "tokenizer.json",
                "tokenizer_config.json",
                "special_tokens_map.json",
                "spiece.model",
            ),
        ),
    ):
        for name in names:
            (scaffold / name).symlink_to(base / directory / name)
    config = json.loads((source / "config.json").read_text())
    config["text_encoder_name"] = str(scaffold)
    model = module.TangoFlux(config)
    weights = load_file(source / "tangoflux.safetensors")
    keys = model.load_state_dict(weights, strict=False)
    verify_weight_keys(keys.missing_keys, keys.unexpected_keys)
    if (
        model.text_encoder.shared.weight.data_ptr()
        != model.text_encoder.encoder.embed_tokens.weight.data_ptr()
    ):
        raise ValueError("T5 embedding alias is not tied")
    for name, value in model.state_dict().items():
        expected = (
            weights["text_encoder.shared.weight"]
            if name in keys.missing_keys
            else weights[name]
        )
        if not torch.equal(value, expected):
            raise ValueError(f"loaded weight mismatch: {name}")
    del weights
    vae = AutoencoderOobleck()
    vae.load_state_dict(load_file(source / "vae.safetensors"), strict=True)
    for name in ("LICENSE.md", "STABILITY_AI_COMMUNITY_LICENSE.md"):
        shutil.copyfile(source / name, output / name)
    return model.eval(), vae.eval()


def publish(output: Path, name: str, wave: np.ndarray) -> tuple[dict, np.ndarray]:
    if (
        wave.ndim != 2
        or wave.shape[0] != 2
        or not wave.size
        or not np.isfinite(wave).all()
    ):
        raise ValueError("expected finite stereo waveform")
    peak = float(np.abs(wave).max())
    rms = float(np.sqrt(np.mean(wave.astype(np.float64) ** 2)))
    if rms < 1e-7:
        raise ValueError("silent generation")
    gain = min(1.0, 0.98 / max(peak, 1e-8))
    stereo = wave * gain
    native_path = output / f"{name}-stereo.wav"
    wavfile.write(native_path, RATE, np.round(stereo.T * 32767).astype(np.int16))
    mono = resample_poly(stereo.mean(axis=0), 160, 441).astype(np.float32)
    record = pilot.write_audio(output / f"{name}.wav", mono)
    return {
        **record,
        "native_wav": str(native_path),
        "native_sha256": hashlib.sha256(native_path.read_bytes()).hexdigest(),
        "native_peak": peak,
        "native_rms": rms,
        "native_pcm_gain": gain,
    }, mono * record["pcm_gain"]


def run(
    output: Path,
    steps: int,
    seeds: tuple[int, ...],
    seconds: float,
    adapter: Path | None = None,
    prompts: Path | None = None,
    diagnostics: bool = False,
):
    import torch

    output = output.resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("artifacts must remain outside the repository")
    if not 1 <= seconds <= 10 or not 1 <= steps <= 100:
        raise ValueError("bounded duration/steps required")
    if (
        not 1 <= len(seeds) <= 8
        or len(set(seeds)) != len(seeds)
        or min(seeds) < 0
        or max(seeds) >= 2**32
    ):
        raise ValueError("unique nonnegative seeds required")
    cases = load_cases(prompts)
    output.mkdir(parents=True, exist_ok=False)
    torch.set_num_threads(4)
    torch.manual_seed(0)
    started = time.monotonic()
    shutil.copyfile(__file__, output / "executed-script.py")
    report = {
        "model": MODEL,
        "revision": REVISION,
        "upstream_code_sha256": CODE_HASH,
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "attribution": "Powered by Stability AI; TangoFlux, Hung et al.",
        "license": "non-commercial research only; no engine redistribution",
        "status": "running",
        "reference_audio_input": False,
        "local_training_steps": 0,
        "steps": steps,
        "seconds": seconds,
        "seeds": seeds,
        "guidance_scale": 4.5,
        "precision": "float32",
        "negative_prompt": None,
        "cases": cases,
        "prompt_file_sha256": hashlib.sha256(prompts.read_bytes()).hexdigest()
        if prompts
        else None,
        "rows": [],
        "controls": [],
    }
    pilot.save_report(output / "result.json", report)
    try:
        model, vae = load_models(output)
        if adapter is not None:
            report["adapter"] = load_adapter(model, adapter)
            report["local_training_steps"] = report["adapter"]["training_steps"]
        device = "cuda" if torch.cuda.is_available() else "cpu"
        model.to(device)
        vae.to(device)
        report["device"] = torch.cuda.get_device_name() if device == "cuda" else device
        report["load_seconds"] = time.monotonic() - started
        preview = []
        for seed in seeds:
            for index, case in enumerate(
                (*cases, {"id": "empty-prompt", "prompt": ""})
            ):
                key, prompt = case["id"], case["prompt"]
                tick = time.monotonic()
                torch.manual_seed(seed)
                np.random.seed(seed)
                with torch.inference_mode():
                    latents = model.inference_flow(
                        prompt,
                        duration=seconds,
                        num_inference_steps=steps,
                        guidance_scale=4.5,
                        disable_progress=True,
                    )
                    # Full upstream 30-second latent horizon and full decode;
                    # CPU offload frees denoiser memory for the stereo decoder.
                    model.to("cpu")
                    if device == "cuda":
                        torch.cuda.empty_cache()
                    wave = (
                        vae.decode(latents.transpose(2, 1))
                        .sample[0, :, : int(seconds * RATE)]
                        .cpu()
                        .numpy()
                    )
                    model.to(device)
                record, mono = publish(output, f"{key}-seed{seed}", wave)
                record.update(
                    {"seed": seed, "inference_seconds": time.monotonic() - tick}
                )
                if index < len(cases):
                    report["rows"].append({"case": index, **record})
                    if seed == seeds[0]:
                        preview.extend(
                            (mono, np.zeros(pilot.RATE // 2, dtype=np.float32))
                        )
                        pilot.write_audio(
                            output / "preview.wav", np.concatenate(preview)
                        )
                else:
                    report["controls"].append({"id": key, **record})
                pilot.save_report(output / "result.json", report)
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
        report.update(
            {
                "status": "complete",
                "elapsed_seconds": time.monotonic() - started,
                "preview": str(output / "preview.wav"),
            }
        )
    except BaseException as error:
        report.update(
            {
                "status": "interrupted"
                if isinstance(error, KeyboardInterrupt)
                else "failed",
                "error": f"{type(error).__name__}: {error}",
            }
        )
        pilot.save_report(output / "result.json", report)
        raise
    pilot.save_report(output / "result.json", report)
    if diagnostics:
        import physical_sound_text_tags as tags

        del model, vae
        if torch.cuda.is_available():
            torch.cuda.empty_cache()
        tags.run(output / "result.json", output / "ast-clap.json", [], with_clap=True)
    print(
        json.dumps(
            {key: report[key] for key in ("status", "elapsed_seconds", "preview")}
        ),
        flush=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--steps", type=int, default=50)
    parser.add_argument("--seeds", type=int, nargs="+", default=[42, 123])
    parser.add_argument("--seconds", type=float, default=5.0)
    parser.add_argument(
        "--prompts",
        type=Path,
        help="JSON array of id/prompt cases; optional diagnostic_id",
    )
    parser.add_argument("--diagnostics", action="store_true")
    parser.add_argument(
        "--adapter",
        type=Path,
        help="External adapter checkpoint from the bounded training experiment",
    )
    args = parser.parse_args()
    run(
        args.output,
        args.steps,
        tuple(args.seeds),
        args.seconds,
        args.adapter,
        args.prompts,
        args.diagnostics,
    )
