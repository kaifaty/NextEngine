#!/usr/bin/env python3
"""Run the pinned external VoxCPM2 CUDA evaluation."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

SOURCE_REVISION = "19b6bf7590025418821a86dcb817504e0ad7e5df"
MODEL_REVISION = "bffb3df5a29440629464e5e839f4d214c8714c3d"
DEFAULT_TEXT = "Привет! Сервер синтеза речи работает локально."
DEFAULT_CFG = 2.0
DEFAULT_STEPS = 10
DEFAULT_SEED = 1234
REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
RESULT_PREFIX = "NEXTENGINE_VOXCPM2_RESULT="

VOICE_CONTROLS = {
    "adult-male": (
        "A mature adult man with a low, calm, natural voice and clear articulation"
    ),
    "adult-female": (
        "A mature adult woman with a warm, calm, natural voice and clear articulation"
    ),
    "none": "",
}


@dataclass(frozen=True)
class Artifact:
    size: int
    sha256: str


MODEL_ARTIFACTS = {
    Path(".gitattributes"): Artifact(
        1_519, "11ad7efa24975ee4b0c3c3a38ed18737f0658a5f75a0a96787b576a78a023361"
    ),
    Path("README.md"): Artifact(
        7_776, "7384fad93ce2d98f47d5c3170597f3b31d414c12c92e7fdf3121fa90f19fe29d"
    ),
    Path("audiovae.pth"): Artifact(
        376_951_122,
        "94b5d51e107e0507d4acc976cfdadb64edd6fd06d1f751dadbf2fd1594274bf1",
    ),
    Path("config.json"): Artifact(
        4_336, "405f0dcd92f7feba6011ed4eac5c8d4f74cba9712f07fd5cfa3063bbdd95402c"
    ),
    Path("model.safetensors"): Artifact(
        4_580_080_592,
        "f7f964cfa9da23653baec6e6f7750719977ad944ed9f95fe52fe3a620506891d",
    ),
    Path("special_tokens_map.json"): Artifact(
        1_632, "068594063e37662c02b21acf42ebb334ef6a74fb810e68a2368f88f08351de76"
    ),
    Path("tokenization_voxcpm2.py"): Artifact(
        2_895, "84489ea32b6ee0cae22ed5480cacb6df85c46624c3119be9a2021c3649a12729"
    ),
    Path("tokenizer.json"): Artifact(
        3_676_772,
        "f8984687e4a92a3503d521396d454b7d68e9fdaab2a0288eb3536c7c1aa4bc20",
    ),
    Path("tokenizer_config.json"): Artifact(
        5_059, "e78a3ebb48a0b9437efd1823b6b726c823da89e49dd8bcc90c02419d9baa772b"
    ),
}


class DemoError(RuntimeError):
    """A stable, user-actionable demo setup error."""


def _default_root() -> Path:
    return Path(
        os.environ.get(
            "NEXTENGINE_VOXCPM2_ROOT",
            Path.home() / ".local" / "share" / "nextengine" / "voxcpm2",
        )
    )


def _add_install_options(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--install-root", type=Path, default=_default_root())
    parser.add_argument("--cuda-device", type=int, default=0)
    parser.add_argument(
        "--skip-model-hash-check",
        action="store_true",
        help="Skip the 4.96 GB content hash pass; sizes and source still validate.",
    )


def _add_synthesis_options(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--text", default=DEFAULT_TEXT)
    parser.add_argument(
        "--voice",
        choices=tuple(VOICE_CONTROLS),
        default="adult-male",
    )
    parser.add_argument(
        "--control",
        help="Override the selected Voice Design description.",
    )
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument("--cfg", type=float, default=DEFAULT_CFG)
    parser.add_argument("--steps", type=int, default=DEFAULT_STEPS)
    parser.add_argument("--stream", action="store_true")
    parser.add_argument("--no-optimize", action="store_true")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    effective_argv = list(sys.argv[1:] if argv is None else argv)
    if effective_argv and effective_argv[0] == "_worker":
        return _parse_worker_args(effective_argv[1:])
    parser = argparse.ArgumentParser(
        description="Run VoxCPM2 through a pinned external CUDA environment."
    )
    commands = parser.add_subparsers(dest="command", required=True)

    doctor = commands.add_parser("doctor", help="Validate the external installation.")
    _add_install_options(doctor)

    generate = commands.add_parser("generate", help="Generate one external WAV.")
    _add_install_options(generate)
    _add_synthesis_options(generate)
    generate.add_argument("--output", type=Path)
    generate.add_argument("--force", action="store_true")

    benchmark = commands.add_parser(
        "benchmark",
        help="Run one prewarm and repeated requests in one model session.",
    )
    _add_install_options(benchmark)
    _add_synthesis_options(benchmark)
    benchmark.add_argument("--iterations", type=int, default=3)
    benchmark.add_argument("--output-dir", type=Path)
    return parser.parse_args(effective_argv)


def _parse_worker_args(argv: list[str]) -> argparse.Namespace:
    worker = argparse.ArgumentParser(add_help=False)
    worker.set_defaults(command="_worker")
    worker.add_argument(
        "--operation", choices=("doctor", "generate", "benchmark"), required=True
    )
    worker.add_argument("--install-root", type=Path, required=True)
    worker.add_argument("--text", default=DEFAULT_TEXT)
    worker.add_argument("--control", default=VOICE_CONTROLS["adult-male"])
    worker.add_argument("--seed", type=int, default=DEFAULT_SEED)
    worker.add_argument("--cfg", type=float, default=DEFAULT_CFG)
    worker.add_argument("--steps", type=int, default=DEFAULT_STEPS)
    worker.add_argument("--stream", action="store_true")
    worker.add_argument("--no-optimize", action="store_true")
    worker.add_argument("--iterations", type=int, default=3)
    worker.add_argument("--output", type=Path)
    worker.add_argument("--output-dir", type=Path)
    return worker.parse_args(argv)


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(8 * 1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _git_revision(path: Path) -> str:
    try:
        return subprocess.run(
            ["git", "-C", str(path), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise DemoError(f"cannot read Git revision at {path}: {error}") from error


def validate_installation(
    install_root: Path, *, verify_model_hash: bool
) -> dict[str, Any]:
    root = install_root.expanduser().resolve()
    if root == REPOSITORY_ROOT or REPOSITORY_ROOT in root.parents:
        raise DemoError(f"installation must stay outside the repository: {root}")
    source = root / "source"
    model = root / "model"
    python = root / "venv" / "bin" / "python"
    for label, path in (
        ("VoxCPM source", source),
        ("model directory", model),
        ("Python runtime", python),
    ):
        if not path.exists():
            raise DemoError(f"missing {label}: {path}")
    if not python.is_file() or not os.access(python, os.X_OK):
        raise DemoError(f"Python runtime is not executable: {python}")

    source_revision = _git_revision(source)
    if source_revision != SOURCE_REVISION:
        raise DemoError(
            f"VoxCPM revision mismatch: expected {SOURCE_REVISION}, got {source_revision}"
        )

    checked_hashes: dict[str, str] = {}
    for relative, artifact in MODEL_ARTIFACTS.items():
        path = model / relative
        if not path.is_file():
            raise DemoError(f"missing model artifact: {path}")
        actual_size = path.stat().st_size
        if actual_size != artifact.size:
            raise DemoError(
                f"model size mismatch for {relative}: "
                f"expected {artifact.size}, got {actual_size}"
            )
        if verify_model_hash:
            actual_hash = _sha256(path)
            if actual_hash != artifact.sha256:
                raise DemoError(
                    f"model SHA-256 mismatch for {relative}: "
                    f"expected {artifact.sha256}, got {actual_hash}"
                )
            checked_hashes[str(relative)] = actual_hash

    return {
        "status": "PASS",
        "install_root": str(root),
        "source_revision": source_revision,
        "model_revision": MODEL_REVISION,
        "model_bytes": sum(item.size for item in MODEL_ARTIFACTS.values()),
        "model_hashes_checked": verify_model_hash,
        "checked_hashes": checked_hashes,
        "python": str(python),
    }


def _external_path(candidate: Path, label: str) -> Path:
    path = candidate.expanduser().resolve()
    if path == REPOSITORY_ROOT or REPOSITORY_ROOT in path.parents:
        raise DemoError(f"{label} must stay outside the repository: {path}")
    return path


def _runtime_environment(root: Path, cuda_device: int) -> dict[str, str]:
    if cuda_device < 0:
        raise DemoError("--cuda-device cannot be negative")
    environment = os.environ.copy()
    site_packages = root / "venv" / "lib" / "python3.11" / "site-packages"
    if not site_packages.is_dir():
        raise DemoError(f"missing Python 3.11 site-packages: {site_packages}")
    environment["PYTHONPATH"] = str(root / "source" / "src")
    library_paths = [site_packages / "torch" / "lib"]
    library_paths.extend(sorted((site_packages / "nvidia").glob("*/lib")))
    environment["LD_LIBRARY_PATH"] = os.pathsep.join(
        str(path) for path in library_paths if path.is_dir()
    )
    environment["CUDA_VISIBLE_DEVICES"] = str(cuda_device)
    environment["HF_HOME"] = str(root / "hf-cache")
    environment["MODELSCOPE_CACHE"] = str(root / "modelscope-cache")
    environment["MPLCONFIGDIR"] = str(root / "matplotlib-cache")
    environment["TORCHINDUCTOR_CACHE_DIR"] = str(root / "torch-cache")
    environment["HF_HUB_OFFLINE"] = "1"
    environment["TRANSFORMERS_OFFLINE"] = "1"
    environment["TOKENIZERS_PARALLELISM"] = "false"
    environment["PYTHONUNBUFFERED"] = "1"
    return environment


def _sample_gpu_memory(device: int) -> int | None:
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
    command: list[str], environment: dict[str, str], cuda_device: int
) -> tuple[int, float, int | None, str]:
    started = time.perf_counter()
    process = subprocess.Popen(
        command,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    peak_gpu = _sample_gpu_memory(cuda_device)
    try:
        while process.poll() is None:
            sample = _sample_gpu_memory(cuda_device)
            if sample is not None:
                peak_gpu = max(peak_gpu or 0, sample)
            time.sleep(0.1)
    except KeyboardInterrupt:
        process.terminate()
        process.wait()
        raise
    output, _ = process.communicate()
    return process.returncode, time.perf_counter() - started, peak_gpu, output


def _parse_worker_result(output: str) -> dict[str, Any]:
    for line in reversed(output.splitlines()):
        if line.startswith(RESULT_PREFIX):
            return json.loads(line.removeprefix(RESULT_PREFIX))
    raise DemoError("external runtime did not emit a structured result")


def _control_for(args: argparse.Namespace) -> str:
    raw = args.control if args.control is not None else VOICE_CONTROLS[args.voice]
    return raw.replace("(", "").replace(")", "").strip()


def _validate_synthesis_args(args: argparse.Namespace) -> None:
    if not args.text.strip():
        raise DemoError("--text cannot be empty")
    if args.seed < 0:
        raise DemoError("--seed cannot be negative")
    if not 0.1 <= args.cfg <= 10.0:
        raise DemoError("--cfg must be between 0.1 and 10.0")
    if not 1 <= args.steps <= 100:
        raise DemoError("--steps must be between 1 and 100")


def _worker_command(root: Path, operation: str, args: argparse.Namespace) -> list[str]:
    command = [
        str(root / "venv" / "bin" / "python"),
        str(Path(__file__).resolve()),
        "_worker",
        "--operation",
        operation,
        "--install-root",
        str(root),
    ]
    if operation != "doctor":
        command.extend(
            [
                "--text",
                args.text,
                "--control",
                _control_for(args),
                "--seed",
                str(args.seed),
                "--cfg",
                str(args.cfg),
                "--steps",
                str(args.steps),
            ]
        )
        if args.stream:
            command.append("--stream")
        if args.no_optimize:
            command.append("--no-optimize")
    if operation == "generate":
        command.extend(["--output", str(args.output)])
    elif operation == "benchmark":
        command.extend(
            [
                "--iterations",
                str(args.iterations),
                "--output-dir",
                str(args.output_dir),
            ]
        )
    return command


def _execute_worker(
    root: Path, operation: str, args: argparse.Namespace
) -> tuple[dict[str, Any], str]:
    environment = _runtime_environment(root, args.cuda_device)
    code, process_wall, peak_gpu, output = _run_monitored(
        _worker_command(root, operation, args), environment, args.cuda_device
    )
    if code != 0:
        raise DemoError(f"external VoxCPM2 runtime failed with exit code {code}:\n{output}")
    result = _parse_worker_result(output)
    result["process_wall_seconds"] = process_wall
    result["peak_total_gpu_memory_mib"] = peak_gpu
    return result, output


def doctor(args: argparse.Namespace) -> int:
    installation = validate_installation(
        args.install_root, verify_model_hash=not args.skip_model_hash_check
    )
    result, _ = _execute_worker(Path(installation["install_root"]), "doctor", args)
    print(json.dumps({**installation, "runtime": result}, indent=2, sort_keys=True))
    return 0


def generate(args: argparse.Namespace) -> int:
    _validate_synthesis_args(args)
    installation = validate_installation(
        args.install_root, verify_model_hash=not args.skip_model_hash_check
    )
    root = Path(installation["install_root"])
    mode = "stream" if args.stream else "offline"
    default_output = root / "outputs" / f"voxcpm2-russian-{args.voice}-{mode}.wav"
    output = _external_path(
        args.output if args.output is not None else default_output,
        "generated WAV",
    )
    if output.exists() and not args.force:
        raise DemoError(f"output exists; pass --force to replace it: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    args.output = output
    result, runtime_log = _execute_worker(root, "generate", args)
    if not output.is_file():
        raise DemoError(f"runtime reported success without output WAV: {output}")
    log_path = output.with_suffix(".log")
    log_path.write_text(runtime_log, encoding="utf-8")
    print(
        json.dumps(
            {**installation, **result, "runtime_log": str(log_path)},
            indent=2,
            sort_keys=True,
        )
    )
    return 0


def benchmark(args: argparse.Namespace) -> int:
    _validate_synthesis_args(args)
    if args.iterations <= 0:
        raise DemoError("--iterations must be positive")
    installation = validate_installation(
        args.install_root, verify_model_hash=not args.skip_model_hash_check
    )
    root = Path(installation["install_root"])
    mode = "stream" if args.stream else "offline"
    optimize = "eager" if args.no_optimize else "compiled"
    default_output = (
        root
        / "outputs"
        / (
            f"benchmark-{mode}-{optimize}-steps{args.steps}-"
            f"{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
        )
    )
    output_dir = _external_path(
        args.output_dir if args.output_dir is not None else default_output,
        "benchmark output directory",
    )
    if output_dir.exists():
        raise DemoError(f"benchmark output directory already exists: {output_dir}")
    output_dir.mkdir(parents=True)
    args.output_dir = output_dir
    result, runtime_log = _execute_worker(root, "benchmark", args)
    result = {**installation, **result}
    log_path = output_dir / "runtime.log"
    summary_path = output_dir / "summary.json"
    log_path.write_text(runtime_log, encoding="utf-8")
    result["runtime_log"] = str(log_path)
    summary_path.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


def _seed_runtime(seed: int, modules: dict[str, Any]) -> None:
    modules["random"].seed(seed)
    modules["numpy"].random.seed(seed)
    torch = modules["torch"]
    torch.manual_seed(seed)
    torch.cuda.manual_seed_all(seed)


def _designed_text(control: str, text: str) -> str:
    clean_control = control.replace("(", "").replace(")", "").strip()
    return f"({clean_control}){text}" if clean_control else text


def _worker_imports() -> dict[str, Any]:
    import importlib.metadata
    import random

    import numpy
    import soundfile
    import torch
    import torchaudio
    import transformers
    from voxcpm import VoxCPM

    return {
        "metadata": importlib.metadata,
        "random": random,
        "numpy": numpy,
        "soundfile": soundfile,
        "torch": torch,
        "torchaudio": torchaudio,
        "transformers": transformers,
        "VoxCPM": VoxCPM,
    }


def _worker_doctor(modules: dict[str, Any]) -> dict[str, Any]:
    torch = modules["torch"]
    if not torch.cuda.is_available():
        raise DemoError("PyTorch CUDA is unavailable in the external runtime")
    if not torch.cuda.is_bf16_supported():
        raise DemoError("GPU does not report BF16 support required by this model")
    return {
        "status": "PASS",
        "python": sys.version.split()[0],
        "voxcpm": modules["metadata"].version("voxcpm"),
        "torch": torch.__version__,
        "torchaudio": modules["torchaudio"].__version__,
        "transformers": modules["transformers"].__version__,
        "torch_cuda": torch.version.cuda,
        "gpu": torch.cuda.get_device_name(0),
        "compute_capability": list(torch.cuda.get_device_capability(0)),
        "bf16_supported": torch.cuda.is_bf16_supported(),
    }


def _worker_load_model(
    root: Path, optimize: bool, modules: dict[str, Any]
) -> tuple[Any, float]:
    torch = modules["torch"]
    _seed_runtime(DEFAULT_SEED, modules)
    torch.cuda.reset_peak_memory_stats()
    started = time.perf_counter()
    model = modules["VoxCPM"].from_pretrained(
        str(root / "model"),
        load_denoiser=False,
        optimize=optimize,
        device="cuda",
    )
    torch.cuda.synchronize()
    return model, time.perf_counter() - started


def _worker_synthesize(
    *,
    model: Any,
    modules: dict[str, Any],
    text: str,
    control: str,
    seed: int,
    cfg: float,
    steps: int,
    stream: bool,
    output: Path,
) -> dict[str, Any]:
    numpy = modules["numpy"]
    torch = modules["torch"]
    soundfile = modules["soundfile"]
    _seed_runtime(seed, modules)
    torch.cuda.synchronize()
    started = time.perf_counter()
    first_chunk_seconds: float | None = None
    chunks = []
    chunk_audio_seconds = []
    kwargs = {
        "text": _designed_text(control, text),
        "cfg_value": cfg,
        "inference_timesteps": steps,
        "normalize": False,
        "denoise": False,
        "retry_badcase": False,
    }
    if stream:
        for item in model.generate_streaming(**kwargs):
            if first_chunk_seconds is None:
                first_chunk_seconds = time.perf_counter() - started
            chunk = numpy.asarray(item, dtype=numpy.float32).reshape(-1)
            chunks.append(chunk)
            chunk_audio_seconds.append(chunk.size / model.tts_model.sample_rate)
    else:
        item = model.generate(**kwargs)
        chunks.append(numpy.asarray(item, dtype=numpy.float32).reshape(-1))
        chunk_audio_seconds.append(chunks[0].size / model.tts_model.sample_rate)
    torch.cuda.synchronize()
    wall_seconds = time.perf_counter() - started
    if not chunks:
        raise DemoError("VoxCPM2 produced no audio chunks")
    audio = numpy.ascontiguousarray(numpy.concatenate(chunks), dtype=numpy.float32)
    sample_rate = int(model.tts_model.sample_rate)
    audio_seconds = audio.size / sample_rate
    pcm_sha256 = hashlib.sha256(audio.tobytes()).hexdigest()
    output.parent.mkdir(parents=True, exist_ok=True)
    soundfile.write(output, audio, sample_rate, subtype="FLOAT")
    return {
        "output": str(output),
        "bytes": output.stat().st_size,
        "sha256": _sha256(output),
        "pcm_sha256": pcm_sha256,
        "sample_rate_hz": sample_rate,
        "channels": 1,
        "frames": audio.size,
        "audio_seconds": audio_seconds,
        "wall_seconds": wall_seconds,
        "rtf": wall_seconds / audio_seconds,
        "stream": stream,
        "chunk_count": len(chunks),
        "chunk_audio_seconds": chunk_audio_seconds,
        "first_chunk_seconds": first_chunk_seconds if stream else None,
    }


def _request_metadata(args: argparse.Namespace) -> dict[str, Any]:
    return {
        "text": args.text,
        "control": args.control,
        "seed": args.seed,
        "cfg": args.cfg,
        "inference_timesteps": args.steps,
        "normalize": False,
        "denoise": False,
        "retry_badcase": False,
        "stream": args.stream,
    }


def _worker_generate(args: argparse.Namespace, modules: dict[str, Any]) -> dict[str, Any]:
    if args.output is None:
        raise DemoError("worker generate requires --output")
    root = args.install_root.resolve()
    torch = modules["torch"]
    model, load_seconds = _worker_load_model(root, not args.no_optimize, modules)
    request = _worker_synthesize(
        model=model,
        modules=modules,
        text=args.text,
        control=args.control,
        seed=args.seed,
        cfg=args.cfg,
        steps=args.steps,
        stream=args.stream,
        output=args.output,
    )
    return {
        "status": "PASS",
        "operation": "generate",
        "optimized": not args.no_optimize,
        "precision": "bfloat16-lm/float32-audiovae",
        "model_load_seconds": load_seconds,
        "request_config": _request_metadata(args),
        "request": request,
        "torch_peak_allocated_mib": torch.cuda.max_memory_allocated() / 1024**2,
        "torch_peak_reserved_mib": torch.cuda.max_memory_reserved() / 1024**2,
    }


def _worker_benchmark(args: argparse.Namespace, modules: dict[str, Any]) -> dict[str, Any]:
    if args.output_dir is None:
        raise DemoError("worker benchmark requires --output-dir")
    root = args.install_root.resolve()
    output_dir = args.output_dir.resolve()
    torch = modules["torch"]
    model, load_seconds = _worker_load_model(root, not args.no_optimize, modules)
    prewarm = _worker_synthesize(
        model=model,
        modules=modules,
        text=args.text,
        control=args.control,
        seed=args.seed,
        cfg=args.cfg,
        steps=args.steps,
        stream=args.stream,
        output=output_dir / "prewarm.wav",
    )
    warm = []
    for index in range(1, args.iterations + 1):
        warm.append(
            _worker_synthesize(
                model=model,
                modules=modules,
                text=args.text,
                control=args.control,
                seed=args.seed,
                cfg=args.cfg,
                steps=args.steps,
                stream=args.stream,
                output=output_dir / f"warm_{index}.wav",
            )
        )
    walls = [item["wall_seconds"] for item in warm]
    rtfs = [item["rtf"] for item in warm]
    audios = [item["audio_seconds"] for item in warm]
    first_chunks = [
        item["first_chunk_seconds"]
        for item in warm
        if item["first_chunk_seconds"] is not None
    ]
    return {
        "status": "PASS",
        "operation": "benchmark",
        "output_dir": str(output_dir),
        "optimized": not args.no_optimize,
        "precision": "bfloat16-lm/float32-audiovae",
        "model_load_seconds": load_seconds,
        "request_config": _request_metadata(args),
        "prewarm": prewarm,
        "warm": warm,
        "warm_iterations": len(warm),
        "warm_wall_seconds_mean": statistics.mean(walls),
        "warm_wall_seconds_min": min(walls),
        "warm_wall_seconds_max": max(walls),
        "warm_audio_seconds_mean": statistics.mean(audios),
        "warm_rtf_mean": statistics.mean(rtfs),
        "warm_x_realtime_mean": statistics.mean(1 / value for value in rtfs),
        "warm_first_chunk_seconds_mean": (
            statistics.mean(first_chunks) if first_chunks else None
        ),
        "fixed_seed_outputs_byte_identical": len(
            {item["sha256"] for item in warm}
        )
        == 1,
        "fixed_seed_pcm_identical": len(
            {item["pcm_sha256"] for item in warm}
        )
        == 1,
        "torch_peak_allocated_mib": torch.cuda.max_memory_allocated() / 1024**2,
        "torch_peak_reserved_mib": torch.cuda.max_memory_reserved() / 1024**2,
    }


def worker(args: argparse.Namespace) -> int:
    root = args.install_root.expanduser().resolve()
    import_started = time.perf_counter()
    modules = _worker_imports()
    import_seconds = time.perf_counter() - import_started
    if args.operation == "doctor":
        result = _worker_doctor(modules)
    elif args.operation == "generate":
        result = _worker_generate(args, modules)
    else:
        result = _worker_benchmark(args, modules)
    result["process_import_seconds"] = import_seconds
    print(RESULT_PREFIX + json.dumps(result, sort_keys=True))
    return 0


def main(argv: list[str] | None = None) -> int:
    try:
        args = parse_args(argv)
        if args.command == "_worker":
            return worker(args)
        if args.command == "doctor":
            return doctor(args)
        if args.command == "generate":
            return generate(args)
        return benchmark(args)
    except DemoError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
