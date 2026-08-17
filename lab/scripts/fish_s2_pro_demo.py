#!/usr/bin/env python3
"""Run the pinned external Fish S2 Pro/s2.cpp local demo."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import struct
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

S2_CODE_REVISION = "2c33261938da1a41d713768b1b391b4d368d7d2c"
GGML_CODE_REVISION = "57ea0bc119d722d74594196cc5b494a34dd87be4"
S2_GGUF_REVISION = "a7320690b5585b03b20ed6484b55926f3015f48d"
TOKENIZER_SHA256 = "f24e08099d45a8adf3f52f5f0b03276e433bb9d689bb15fcbcc48ce58744588b"
REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


@dataclass(frozen=True)
class ModelProfile:
    filename: str
    size: int
    sha256: str
    default_codec: str


MODEL_PROFILES = {
    "q4": ModelProfile(
        filename="s2-pro-q4_k_m.gguf",
        size=3_566_165_088,
        sha256="83963e1b7cec980b41eb2163d617e2b6241bfd1564dd880e5b43fc4834807bd9",
        default_codec="gpu",
    ),
    "q5": ModelProfile(
        filename="s2-pro-q5_k_m.gguf",
        size=4_031_183_968,
        sha256="e445b0c8f32ed0ff584b906098f0fe53a67c0691249bfcccde569544f7d72cb9",
        default_codec="gpu",
    ),
    "q6": ModelProfile(
        filename="s2-pro-q6_k.gguf",
        size=4_525_266_528,
        sha256="84ac904172a2cadb84e8f7f14ea3f1acef0584987635e85f7207fd254eafa235",
        default_codec="gpu",
    ),
    "q8": ModelProfile(
        filename="s2-pro-q8_0.gguf",
        size=5_630_037_088,
        sha256="e2043182234786e7b975547d3bbcb23ff02e4ff684b82f7fa851287e4cb4f267",
        default_codec="cpu",
    ),
}


class DemoError(RuntimeError):
    """A stable, user-actionable demo setup error."""


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run the external Fish S2 Pro GGUF demo without engine integration."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    doctor = subparsers.add_parser("doctor", help="Validate the local installation.")
    _add_install_options(doctor)

    generate = subparsers.add_parser("generate", help="Generate one external WAV.")
    _add_install_options(generate)
    generate.add_argument(
        "--text",
        default="Привет! Это демонстрация Fish S2 Pro на локальном компьютере.",
    )
    generate.add_argument("--output", type=Path)
    generate.add_argument("--force", action="store_true")
    generate.add_argument("--max-tokens", type=int, default=384)
    generate.add_argument("--threads", type=int, default=0)
    generate.add_argument("--stream-file", action="store_true")

    server = subparsers.add_parser("server", help="Start the local HTTP server.")
    _add_install_options(server)
    server.add_argument("--host", default="127.0.0.1")
    server.add_argument("--port", type=int, default=3030)
    server.add_argument(
        "--allow-non-loopback",
        action="store_true",
        help="Explicitly permit binding beyond localhost.",
    )
    return parser.parse_args(argv)


def _add_install_options(parser: argparse.ArgumentParser) -> None:
    default_root = Path(
        os.environ.get(
            "NEXTENGINE_FISH_S2_ROOT",
            Path.home() / ".local" / "share" / "nextengine" / "fish-s2-pro",
        )
    )
    parser.add_argument("--install-root", type=Path, default=default_root)
    parser.add_argument("--quality", choices=sorted(MODEL_PROFILES), default="q6")
    parser.add_argument("--cuda-device", type=int, default=0)
    parser.add_argument("--gpu-layers", type=int, default=-1)
    parser.add_argument(
        "--codec",
        choices=("profile", "auto", "cpu", "gpu"),
        default="profile",
        help=(
            "profile uses GPU for Q4/Q5/Q6 and CPU for Q8 on the tested "
            "10 GiB RTX 3080."
        ),
    )
    parser.add_argument(
        "--skip-model-hash-check",
        action="store_true",
        help="Skip the multi-GB model SHA-256 pass; size and pinned source are still checked.",
    )


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(8 * 1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _git_revision(source: Path) -> str:
    try:
        return subprocess.run(
            ["git", "-C", str(source), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise DemoError(f"cannot read s2.cpp revision at {source}: {error}") from error


def validate_installation(
    install_root: Path, quality: str, *, verify_model_hash: bool
) -> dict[str, object]:
    root = install_root.expanduser().resolve()
    profile = MODEL_PROFILES[quality]
    source = root / "s2.cpp"
    binary = source / "build" / "s2"
    tokenizer = source / "tokenizer.json"
    model = root / "models" / profile.filename

    for label, path in (
        ("s2.cpp source", source),
        ("tokenizer", tokenizer),
        ("model", model),
    ):
        if not path.exists():
            raise DemoError(f"missing {label}: {path}")
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise DemoError(f"missing executable CUDA build: {binary}")

    code_revision = _git_revision(source)
    if code_revision != S2_CODE_REVISION:
        raise DemoError(
            f"s2.cpp revision mismatch: expected {S2_CODE_REVISION}, got {code_revision}"
        )
    ggml_revision = _git_revision(source / "ggml")
    if ggml_revision != GGML_CODE_REVISION:
        raise DemoError(
            f"ggml revision mismatch: expected {GGML_CODE_REVISION}, got {ggml_revision}"
        )
    if tokenizer.stat().st_size != 12_217_872:
        raise DemoError(f"tokenizer size mismatch: {tokenizer}")
    tokenizer_sha256 = _sha256(tokenizer)
    if tokenizer_sha256 != TOKENIZER_SHA256:
        raise DemoError(
            f"tokenizer SHA-256 mismatch: expected {TOKENIZER_SHA256}, got {tokenizer_sha256}"
        )
    if model.stat().st_size != profile.size:
        raise DemoError(
            f"model size mismatch: expected {profile.size}, got {model.stat().st_size}"
        )

    model_sha256: str | None = None
    if verify_model_hash:
        model_sha256 = _sha256(model)
        if model_sha256 != profile.sha256:
            raise DemoError(
                f"model SHA-256 mismatch: expected {profile.sha256}, got {model_sha256}"
            )
    return {
        "status": "PASS",
        "install_root": str(root),
        "quality": quality,
        "s2_code_revision": code_revision,
        "ggml_code_revision": ggml_revision,
        "gguf_revision": S2_GGUF_REVISION,
        "binary": str(binary),
        "tokenizer": str(tokenizer),
        "tokenizer_sha256": tokenizer_sha256,
        "model": str(model),
        "model_size": profile.size,
        "model_sha256": model_sha256,
        "model_sha256_expected": profile.sha256,
    }


def codec_arguments(codec: str, profile: ModelProfile) -> list[str]:
    selected = profile.default_codec if codec == "profile" else codec
    if selected == "auto":
        return ["--codec-auto"]
    if selected == "cpu":
        return ["--codec-cpu"]
    if selected == "gpu":
        return ["--codec-follow-backend"]
    raise DemoError(f"unsupported codec route: {selected}")


def _output_path(install_root: Path, quality: str, candidate: Path | None) -> Path:
    output = (
        candidate
        if candidate is not None
        else install_root / "outputs" / f"fish-s2-pro-{quality}-demo.wav"
    )
    output = output.expanduser().resolve()
    if output == REPOSITORY_ROOT or REPOSITORY_ROOT in output.parents:
        raise DemoError(f"generated WAV must stay outside the repository: {output}")
    return output


def _sample_gpu_memory(device: int) -> int | None:
    if device < 0:
        return None
    try:
        output = subprocess.run(
            [
                "nvidia-smi",
                f"--id={device}",
                "--query-gpu=memory.used",
                "--format=csv,noheader,nounits",
            ],
            check=True,
            capture_output=True,
            text=True,
            timeout=2,
        ).stdout.strip()
        return int(output.splitlines()[0])
    except (
        OSError,
        ValueError,
        IndexError,
        subprocess.CalledProcessError,
        subprocess.TimeoutExpired,
    ):
        return None


def _run_monitored(command: list[str], device: int) -> tuple[int, float, int | None]:
    start = time.perf_counter()
    process = subprocess.Popen(command)
    peak_gpu_memory = _sample_gpu_memory(device)
    try:
        while process.poll() is None:
            sample = _sample_gpu_memory(device)
            if sample is not None:
                peak_gpu_memory = max(peak_gpu_memory or 0, sample)
            time.sleep(0.2)
    except KeyboardInterrupt:
        process.terminate()
        process.wait()
        raise
    return process.wait(), time.perf_counter() - start, peak_gpu_memory


def _inspect_wav(path: Path) -> dict[str, object]:
    format_fields: tuple[int, int, int, int, int, int] | None = None
    data_size: int | None = None
    with path.open("rb") as wav_file:
        header = wav_file.read(12)
        if len(header) != 12 or header[:4] != b"RIFF" or header[8:] != b"WAVE":
            raise DemoError(f"not a RIFF/WAVE file: {path}")
        while chunk_header := wav_file.read(8):
            if len(chunk_header) != 8:
                raise DemoError(f"truncated WAV chunk header: {path}")
            chunk_id, chunk_size = struct.unpack("<4sI", chunk_header)
            if chunk_id == b"fmt ":
                chunk = wav_file.read(chunk_size)
                if len(chunk) != chunk_size or chunk_size < 16:
                    raise DemoError(f"invalid WAV fmt chunk: {path}")
                format_fields = struct.unpack("<HHIIHH", chunk[:16])
            elif chunk_id == b"data":
                data_size = chunk_size
                wav_file.seek(chunk_size, os.SEEK_CUR)
            else:
                wav_file.seek(chunk_size, os.SEEK_CUR)
            if chunk_size % 2:
                wav_file.seek(1, os.SEEK_CUR)
    if format_fields is None or data_size is None:
        raise DemoError(f"WAV is missing fmt or data chunk: {path}")
    format_tag, channels, sample_rate, byte_rate, block_align, bits_per_sample = (
        format_fields
    )
    if format_tag not in (1, 3):
        raise DemoError(f"unsupported WAV format tag {format_tag}: {path}")
    if sample_rate <= 0 or byte_rate <= 0 or block_align <= 0:
        raise DemoError(f"invalid WAV rate/alignment fields: {path}")
    frames = data_size // block_align
    sample_width = bits_per_sample // 8
    duration = frames / sample_rate if sample_rate else 0.0
    return {
        "format": "pcm" if format_tag == 1 else "ieee-float",
        "format_tag": format_tag,
        "channels": channels,
        "sample_rate_hz": sample_rate,
        "sample_width_bytes": sample_width,
        "frames": frames,
        "duration_seconds": duration,
        "sha256": _sha256(path),
        "bytes": path.stat().st_size,
    }


def generate(args: argparse.Namespace) -> int:
    if args.max_tokens <= 0:
        raise DemoError("--max-tokens must be positive")
    if args.threads < 0:
        raise DemoError("--threads cannot be negative")
    installation = validate_installation(
        args.install_root,
        args.quality,
        verify_model_hash=not args.skip_model_hash_check,
    )
    root = Path(installation["install_root"])
    output = _output_path(root, args.quality, args.output)
    if output.exists() and not args.force:
        raise DemoError(
            f"output already exists; pass --force or choose another path: {output}"
        )
    output.parent.mkdir(parents=True, exist_ok=True)
    staging_handle, staging_name = tempfile.mkstemp(
        prefix=f".{output.stem}-", suffix=".wav", dir=output.parent
    )
    os.close(staging_handle)
    staging = Path(staging_name)
    try:
        profile = MODEL_PROFILES[args.quality]
        command = [
            str(installation["binary"]),
            "--model",
            str(installation["model"]),
            "--tokenizer",
            str(installation["tokenizer"]),
            "--cuda",
            str(args.cuda_device),
            "--gpu-layers",
            str(args.gpu_layers),
            "--max-tokens",
            str(args.max_tokens),
            "--text",
            args.text,
            "--output",
            str(staging),
            *codec_arguments(args.codec, profile),
        ]
        if args.threads:
            command.extend(("--threads", str(args.threads)))
        if args.stream_file:
            command.append("--stream-file")
        return_code, elapsed, peak_gpu_memory = _run_monitored(
            command, args.cuda_device
        )
        if return_code != 0:
            raise DemoError(f"s2 synthesis failed with exit status {return_code}")
        wav = _inspect_wav(staging)
        os.replace(staging, output)
        summary = {
            "status": "PASS",
            "quality": args.quality,
            "codec": profile.default_codec if args.codec == "profile" else args.codec,
            "cold_elapsed_seconds": elapsed,
            "cold_realtime_factor": elapsed / float(wav["duration_seconds"]),
            "peak_total_gpu_memory_mib": peak_gpu_memory,
            "output": str(output),
            "wav": wav,
        }
        print(json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True))
        return 0
    finally:
        staging.unlink(missing_ok=True)


def server(args: argparse.Namespace) -> int:
    loopback_hosts = {"127.0.0.1", "localhost", "::1"}
    if args.host not in loopback_hosts and not args.allow_non_loopback:
        raise DemoError(
            "non-loopback HTTP binding requires the explicit --allow-non-loopback flag"
        )
    if not 1 <= args.port <= 65_535:
        raise DemoError("--port must be in the range 1..65535")
    installation = validate_installation(
        args.install_root,
        args.quality,
        verify_model_hash=not args.skip_model_hash_check,
    )
    profile = MODEL_PROFILES[args.quality]
    command = [
        str(installation["binary"]),
        "--model",
        str(installation["model"]),
        "--tokenizer",
        str(installation["tokenizer"]),
        "--cuda",
        str(args.cuda_device),
        "--gpu-layers",
        str(args.gpu_layers),
        *codec_arguments(args.codec, profile),
        "--server",
        "--host",
        args.host,
        "--port",
        str(args.port),
    ]
    process = subprocess.Popen(command)
    try:
        return process.wait()
    except KeyboardInterrupt:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        return 130


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        if args.command == "doctor":
            result = validate_installation(
                args.install_root,
                args.quality,
                verify_model_hash=not args.skip_model_hash_check,
            )
            print(json.dumps(result, indent=2, sort_keys=True))
            return 0
        if args.command == "generate":
            return generate(args)
        if args.command == "server":
            return server(args)
        raise DemoError(f"unsupported command: {args.command}")
    except DemoError as error:
        print(f"fish-s2-pro-demo: {error}", file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        print("fish-s2-pro-demo: interrupted", file=sys.stderr)
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
