#!/usr/bin/env python3
"""Run the pinned external Fish S2 Pro/audio.cpp CUDA demo."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import statistics
import struct
import subprocess
import sys
import tempfile
import time
from datetime import UTC, datetime
from pathlib import Path

AUDIO_CPP_CODE_REVISION = "980bd4164b9de744b618a6b0d5e6e515de94999a"
AUDIO_CPP_GGUF_REVISION = "c3857f1ec35cfea8993924e7c2a6f682b5dc060b"
MODEL_SIZE = 6_317_911_232
MODEL_SHA256 = "4ffc169447b7a26df8bf49e8637adb4000bfa763a22c018b6c03968564259d0b"
MODEL_RELATIVE_PATH = Path("models/Fish-Audio-S2-Pro-GGUF/fish-audio-s2-pro-q8_0.gguf")
REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_TEXT = "Привет! Сервер синтеза речи работает локально."
METRIC_PATTERN = re.compile(
    r"^metrics(?:\[([^]]+)])?\.(wall_ms|audio_duration_ms|rtf|x_realtime|"
    r"sample_rate|channels)=(.+)$"
)


class DemoError(RuntimeError):
    """A stable, user-actionable demo setup error."""


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run Fish S2 Pro through pinned audio.cpp outside the engine."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    doctor = subparsers.add_parser("doctor", help="Validate the local installation.")
    _add_install_options(doctor)

    generate = subparsers.add_parser("generate", help="Generate one external WAV.")
    _add_install_options(generate)
    generate.add_argument("--text", default=DEFAULT_TEXT)
    generate.add_argument("--output", type=Path)
    generate.add_argument("--force", action="store_true")
    generate.add_argument("--max-tokens", type=int, default=384)
    generate.add_argument("--seed", type=int, default=1234)

    benchmark = subparsers.add_parser(
        "benchmark", help="Measure repeated requests in one warm model session."
    )
    _add_install_options(benchmark)
    benchmark.add_argument("--text", default=DEFAULT_TEXT)
    benchmark.add_argument("--max-tokens", type=int, default=384)
    benchmark.add_argument("--seed", type=int, default=1234)
    benchmark.add_argument("--iterations", type=int, default=3)
    benchmark.add_argument("--output-dir", type=Path)
    return parser.parse_args(argv)


def _add_install_options(parser: argparse.ArgumentParser) -> None:
    default_root = Path(
        os.environ.get(
            "NEXTENGINE_FISH_AUDIO_CPP_ROOT",
            Path.home() / ".local" / "share" / "nextengine" / "audio-cpp-fish-s2-pro",
        )
    )
    parser.add_argument("--install-root", type=Path, default=default_root)
    parser.add_argument("--cuda-device", type=int, default=0)
    parser.add_argument(
        "--threads",
        type=int,
        default=4,
        help="CPU helper threads; 4 was fastest in the bounded RTX 3080 probe.",
    )
    parser.add_argument(
        "--skip-model-hash-check",
        action="store_true",
        help="Skip the multi-GB SHA-256 pass; size and pinned source are still checked.",
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
        raise DemoError(
            f"cannot read audio.cpp revision at {source}: {error}"
        ) from error


def _read_cmake_cache(cache: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in cache.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith(("#", "//")) or "=" not in line:
            continue
        typed_key, value = line.split("=", 1)
        key = typed_key.split(":", 1)[0]
        values[key] = value
    return values


def validate_installation(
    install_root: Path, *, verify_model_hash: bool
) -> dict[str, object]:
    root = install_root.expanduser().resolve()
    source = root / "source"
    build = source / "build" / "linux-cuda-release"
    binary = build / "bin" / "audiocpp_cli"
    cache = build / "CMakeCache.txt"
    model = root / MODEL_RELATIVE_PATH

    for label, path in (
        ("audio.cpp source", source),
        ("CMake cache", cache),
        ("model", model),
    ):
        if not path.exists():
            raise DemoError(f"missing {label}: {path}")
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise DemoError(f"missing executable CUDA build: {binary}")

    code_revision = _git_revision(source)
    if code_revision != AUDIO_CPP_CODE_REVISION:
        raise DemoError(
            "audio.cpp revision mismatch: "
            f"expected {AUDIO_CPP_CODE_REVISION}, got {code_revision}"
        )
    expected_cache = {
        "AUDIOCPP_DEPLOYMENT_BUILD": "ON",
        "AUDIOCPP_MODEL_SET": "custom",
        "AUDIOCPP_MODELS": "fish_audio",
        "CMAKE_CUDA_ARCHITECTURES": "86",
        "ENGINE_ENABLE_CUDA": "ON",
        "ENGINE_ENABLE_CUDA_GRAPHS": "ON",
    }
    cache_values = _read_cmake_cache(cache)
    for key, expected in expected_cache.items():
        actual = cache_values.get(key)
        if actual != expected:
            raise DemoError(
                f"CMake cache mismatch for {key}: expected {expected}, got {actual}"
            )
    if model.stat().st_size != MODEL_SIZE:
        raise DemoError(
            f"model size mismatch: expected {MODEL_SIZE}, got {model.stat().st_size}"
        )

    model_sha256: str | None = None
    if verify_model_hash:
        model_sha256 = _sha256(model)
        if model_sha256 != MODEL_SHA256:
            raise DemoError(
                f"model SHA-256 mismatch: expected {MODEL_SHA256}, got {model_sha256}"
            )
    return {
        "status": "PASS",
        "install_root": str(root),
        "audio_cpp_revision": code_revision,
        "gguf_revision": AUDIO_CPP_GGUF_REVISION,
        "binary": str(binary),
        "model": str(model),
        "model_size": MODEL_SIZE,
        "model_sha256": model_sha256,
        "model_sha256_expected": MODEL_SHA256,
        "cuda_architecture": 86,
        "cuda_graphs": True,
    }


def _external_path(path: Path, label: str) -> Path:
    resolved = path.expanduser().resolve()
    if resolved == REPOSITORY_ROOT or REPOSITORY_ROOT in resolved.parents:
        raise DemoError(f"{label} must stay outside the repository: {resolved}")
    return resolved


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


def _run_monitored(
    command: list[str], device: int
) -> tuple[int, float, int | None, str]:
    start = time.perf_counter()
    process = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    peak_gpu_memory = _sample_gpu_memory(device)
    try:
        while process.poll() is None:
            sample = _sample_gpu_memory(device)
            if sample is not None:
                peak_gpu_memory = max(peak_gpu_memory or 0, sample)
            time.sleep(0.1)
    except KeyboardInterrupt:
        process.terminate()
        process.wait()
        raise
    stdout, _ = process.communicate()
    return process.returncode, time.perf_counter() - start, peak_gpu_memory, stdout


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
    return {
        "format": "pcm" if format_tag == 1 else "ieee-float",
        "channels": channels,
        "sample_rate_hz": sample_rate,
        "sample_width_bytes": bits_per_sample // 8,
        "frames": frames,
        "duration_seconds": frames / sample_rate,
        "sha256": _sha256(path),
        "bytes": path.stat().st_size,
    }


def _parse_metrics(output: str) -> dict[str, dict[str, float]]:
    metrics: dict[str, dict[str, float]] = {}
    for line in output.splitlines():
        match = METRIC_PATTERN.fullmatch(line.strip())
        if match is None:
            continue
        request_id, name, value = match.groups()
        request_id = request_id or "request"
        metrics.setdefault(request_id, {})[name] = float(value)
    return metrics


def _base_command(
    installation: dict[str, object], args: argparse.Namespace
) -> list[str]:
    if args.cuda_device < 0:
        raise DemoError("--cuda-device cannot be negative")
    if args.threads <= 0:
        raise DemoError("--threads must be positive")
    return [
        str(installation["binary"]),
        "--task",
        "tts",
        "--family",
        "fish_audio",
        "--model",
        str(installation["model"]),
        "--backend",
        "cuda",
        "--device",
        str(args.cuda_device),
        "--threads",
        str(args.threads),
        "--session-option",
        "fish_audio.weight_type=native",
        "--session-option",
        "fish_audio.codec_weight_type=native",
        "--session-option",
        "fish_audio.mem_saver=false",
        "--metrics",
    ]


def generate(args: argparse.Namespace) -> int:
    if args.max_tokens <= 0:
        raise DemoError("--max-tokens must be positive")
    installation = validate_installation(
        args.install_root, verify_model_hash=not args.skip_model_hash_check
    )
    root = Path(installation["install_root"])
    output = _external_path(
        args.output
        if args.output is not None
        else root / "outputs" / "fish-s2-pro-audio-cpp-q8-demo.wav",
        "generated WAV",
    )
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
        command = [
            *_base_command(installation, args),
            "--text",
            args.text,
            "--max-tokens",
            str(args.max_tokens),
            "--seed",
            str(args.seed),
            "--out",
            str(staging),
        ]
        return_code, elapsed, peak_gpu_memory, stdout = _run_monitored(
            command, args.cuda_device
        )
        if return_code != 0:
            raise DemoError(
                f"audio.cpp synthesis failed with exit status {return_code}:\n{stdout}"
            )
        metrics = _parse_metrics(stdout).get("request")
        if metrics is None:
            raise DemoError(f"audio.cpp did not emit request metrics:\n{stdout}")
        wav = _inspect_wav(staging)
        os.replace(staging, output)
        summary = {
            "status": "PASS",
            "engine": "audio.cpp",
            "profile": "q8_0",
            "process_elapsed_seconds": elapsed,
            "request_metrics": metrics,
            "peak_total_gpu_memory_mib": peak_gpu_memory,
            "output": str(output),
            "wav": wav,
        }
        print(json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True))
        return 0
    finally:
        staging.unlink(missing_ok=True)


def benchmark(args: argparse.Namespace) -> int:
    if args.max_tokens <= 0:
        raise DemoError("--max-tokens must be positive")
    if args.iterations <= 0:
        raise DemoError("--iterations must be positive")
    installation = validate_installation(
        args.install_root, verify_model_hash=not args.skip_model_hash_check
    )
    root = Path(installation["install_root"])
    timestamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    output_dir = _external_path(
        args.output_dir
        if args.output_dir is not None
        else root / "outputs" / f"audio-cpp-q8-benchmark-{timestamp}",
        "benchmark output directory",
    )
    if output_dir.exists():
        raise DemoError(f"benchmark output directory already exists: {output_dir}")
    output_dir.mkdir(parents=True)
    requests = [
        {
            "id": "prewarm",
            "text": args.text,
            "max_tokens": args.max_tokens,
            "seed": args.seed,
        },
        *[
            {
                "id": f"warm_{index}",
                "text": args.text,
                "max_tokens": args.max_tokens,
                "seed": args.seed,
            }
            for index in range(1, args.iterations + 1)
        ],
    ]
    request_path = output_dir / "requests.json"
    request_path.write_text(
        json.dumps({"requests": requests}, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    command = [
        *_base_command(installation, args),
        "--request-sequence",
        str(request_path),
        "--out-dir",
        str(output_dir),
    ]
    return_code, elapsed, peak_gpu_memory, stdout = _run_monitored(
        command, args.cuda_device
    )
    (output_dir / "audiocpp_cli.log").write_text(stdout, encoding="utf-8")
    if return_code != 0:
        raise DemoError(
            f"audio.cpp benchmark failed with exit status {return_code}:\n{stdout}"
        )
    metrics = _parse_metrics(stdout)
    if "prewarm" not in metrics:
        raise DemoError(f"audio.cpp did not emit prewarm metrics:\n{stdout}")
    warm = []
    for index in range(1, args.iterations + 1):
        request_id = f"warm_{index}"
        if request_id not in metrics:
            raise DemoError(f"audio.cpp did not emit metrics for {request_id}")
        warm.append(metrics[request_id])
    for path in sorted(output_dir.glob("*.wav")):
        _inspect_wav(path)
    wall_ms = [item["wall_ms"] for item in warm]
    rtf = [item["rtf"] for item in warm]
    x_realtime = [item["x_realtime"] for item in warm]
    summary = {
        "status": "PASS",
        "engine": "audio.cpp",
        "profile": "q8_0",
        "prewarm": metrics["prewarm"],
        "warm_iterations": args.iterations,
        "warm_wall_ms_mean": statistics.fmean(wall_ms),
        "warm_wall_ms_min": min(wall_ms),
        "warm_wall_ms_max": max(wall_ms),
        "warm_rtf_mean": statistics.fmean(rtf),
        "warm_x_realtime_mean": statistics.fmean(x_realtime),
        "process_elapsed_seconds": elapsed,
        "peak_total_gpu_memory_mib": peak_gpu_memory,
        "output_dir": str(output_dir),
    }
    (output_dir / "summary.json").write_text(
        json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        if args.command == "doctor":
            result = validate_installation(
                args.install_root,
                verify_model_hash=not args.skip_model_hash_check,
            )
            print(json.dumps(result, indent=2, sort_keys=True))
            return 0
        if args.command == "generate":
            return generate(args)
        if args.command == "benchmark":
            return benchmark(args)
        raise DemoError(f"unsupported command: {args.command}")
    except DemoError as error:
        print(f"fish-s2-pro-audio-cpp-demo: {error}", file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        print("fish-s2-pro-audio-cpp-demo: interrupted", file=sys.stderr)
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
