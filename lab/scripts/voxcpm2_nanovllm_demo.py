#!/usr/bin/env python3
"""Run a pinned Nano-vLLM-VoxCPM2 CUDA evaluation outside the repository."""

from __future__ import annotations

import argparse
import asyncio
import hashlib
import json
import os
import statistics
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


NANO_SOURCE_REVISION = "0ef61b0ba634dbf2fad9e916bc4fb696a3c0f51f"
NANO_VERSION = "2.0.3"
TORCH_VERSION = "2.10.0+cu130"
FLASH_ATTN_VERSION = "2.8.3.post1"
TRANSFORMERS_VERSION = "5.15.0"
FLASH_ATTN_EXTENSION_SHA256 = (
    "b85683e47a0583b48f294633bfbcc17ab28289cd8cdb8279ec70ab68296f9ff1"
)
MODEL_REVISION = "bffb3df5a29440629464e5e839f4d214c8714c3d"
DEFAULT_TEXT = (
    "Привет! Сервер синтеза речи работает локально."
)
DEFAULT_CFG = 2.0
DEFAULT_STEPS = 10
DEFAULT_SEED = 1234
DEFAULT_TEMPERATURE = 1.0
DEFAULT_MAX_GENERATE_LENGTH = 2000
DEFAULT_GPU_MEMORY_UTILIZATION = 0.90
DEFAULT_MAX_MODEL_LEN = 4096
DEFAULT_MAX_BATCHED_TOKENS = 4096
DEFAULT_MAX_SEQS = 1
REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
RESULT_PREFIX = "NEXTENGINE_VOXCPM2_NANOVLLM_RESULT="

MODEL_FILES = {
    Path("config.json"): 4_336,
    Path("model.safetensors"): 4_580_080_592,
    Path("audiovae.pth"): 376_951_122,
    Path("tokenizer.json"): 3_676_772,
}

VOICE_CONTROLS = {
    "adult-male": (
        "A mature adult man with a low, calm, natural voice and clear articulation"
    ),
    "adult-female": (
        "A mature adult woman with a warm, calm, natural voice and clear articulation"
    ),
    "none": "",
}


class DemoError(RuntimeError):
    """A stable, user-actionable demo setup error."""


def _default_root() -> Path:
    return Path(
        os.environ.get(
            "NEXTENGINE_VOXCPM2_NANOVLLM_ROOT",
            Path.home() / ".local" / "share" / "nextengine" / "voxcpm2-nanovllm",
        )
    )


def _default_model_root() -> Path:
    return Path(
        os.environ.get(
            "NEXTENGINE_VOXCPM2_MODEL_ROOT",
            Path.home() / ".local" / "share" / "nextengine" / "voxcpm2" / "model",
        )
    )


def _default_extra_site() -> Path:
    return Path(
        os.environ.get(
            "NEXTENGINE_VOXCPM2_NANOVLLM_EXTRA_SITE",
            _default_root() / "runtime-site",
        )
    )


def _add_install_options(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--install-root", type=Path, default=_default_root())
    parser.add_argument("--model-root", type=Path, default=_default_model_root())
    parser.add_argument(
        "--extra-site",
        type=Path,
        default=_default_extra_site(),
        help="Optional site-packages directory, used for a separately built FlashAttention.",
    )
    parser.add_argument("--cuda-device", type=int, default=0)


def _add_runtime_options(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--text", default=DEFAULT_TEXT)
    parser.add_argument("--voice", choices=tuple(VOICE_CONTROLS), default="adult-male")
    parser.add_argument("--control", help="Override the selected Voice Design description.")
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument("--cfg", type=float, default=DEFAULT_CFG)
    parser.add_argument("--steps", type=int, default=DEFAULT_STEPS)
    parser.add_argument("--temperature", type=float, default=DEFAULT_TEMPERATURE)
    parser.add_argument(
        "--max-generate-length", type=int, default=DEFAULT_MAX_GENERATE_LENGTH
    )
    parser.add_argument(
        "--gpu-memory-utilization",
        type=float,
        default=DEFAULT_GPU_MEMORY_UTILIZATION,
    )
    parser.add_argument("--max-model-len", type=int, default=DEFAULT_MAX_MODEL_LEN)
    parser.add_argument(
        "--max-num-batched-tokens", type=int, default=DEFAULT_MAX_BATCHED_TOKENS
    )
    parser.add_argument("--max-num-seqs", type=int, default=DEFAULT_MAX_SEQS)
    parser.add_argument("--enforce-eager", action="store_true")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    effective_argv = list(sys.argv[1:] if argv is None else argv)
    if effective_argv and effective_argv[0] == "_worker":
        return _parse_worker_args(effective_argv[1:])

    parser = argparse.ArgumentParser(
        description="Run VoxCPM2 through a pinned Nano-vLLM CUDA environment."
    )
    commands = parser.add_subparsers(dest="command", required=True)

    doctor = commands.add_parser("doctor", help="Validate the external runtime.")
    _add_install_options(doctor)

    generate = commands.add_parser("generate", help="Generate one external WAV.")
    _add_install_options(generate)
    _add_runtime_options(generate)
    generate.add_argument("--output", type=Path)
    generate.add_argument("--force", action="store_true")

    benchmark = commands.add_parser(
        "benchmark", help="Run one prewarm and repeated same-session requests."
    )
    _add_install_options(benchmark)
    _add_runtime_options(benchmark)
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
    worker.add_argument("--model-root", type=Path, required=True)
    worker.add_argument("--text", default=DEFAULT_TEXT)
    worker.add_argument("--control", default=VOICE_CONTROLS["adult-male"])
    worker.add_argument("--seed", type=int, default=DEFAULT_SEED)
    worker.add_argument("--cfg", type=float, default=DEFAULT_CFG)
    worker.add_argument("--steps", type=int, default=DEFAULT_STEPS)
    worker.add_argument("--temperature", type=float, default=DEFAULT_TEMPERATURE)
    worker.add_argument(
        "--max-generate-length", type=int, default=DEFAULT_MAX_GENERATE_LENGTH
    )
    worker.add_argument(
        "--gpu-memory-utilization",
        type=float,
        default=DEFAULT_GPU_MEMORY_UTILIZATION,
    )
    worker.add_argument("--max-model-len", type=int, default=DEFAULT_MAX_MODEL_LEN)
    worker.add_argument(
        "--max-num-batched-tokens", type=int, default=DEFAULT_MAX_BATCHED_TOKENS
    )
    worker.add_argument("--max-num-seqs", type=int, default=DEFAULT_MAX_SEQS)
    worker.add_argument("--enforce-eager", action="store_true")
    worker.add_argument("--iterations", type=int, default=3)
    worker.add_argument("--output", type=Path)
    worker.add_argument("--output-dir", type=Path)
    return worker.parse_args(argv)


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


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(8 * 1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _external_path(candidate: Path, label: str) -> Path:
    path = candidate.expanduser().resolve()
    if path == REPOSITORY_ROOT or REPOSITORY_ROOT in path.parents:
        raise DemoError(f"{label} must stay outside the repository: {path}")
    return path


def validate_installation(
    install_root: Path, model_root: Path, extra_site: Path | None
) -> dict[str, Any]:
    root = _external_path(install_root, "Nano-vLLM installation")
    model = _external_path(model_root, "VoxCPM2 model")
    source = root / "source"
    python = root / "venv" / "bin" / "python"
    for label, path in (
        ("Nano-vLLM source", source),
        ("VoxCPM2 model", model),
        ("Python runtime", python),
    ):
        if not path.exists():
            raise DemoError(f"missing {label}: {path}")
    if not python.is_file() or not os.access(python, os.X_OK):
        raise DemoError(f"Python runtime is not executable: {python}")

    revision = _git_revision(source)
    if revision != NANO_SOURCE_REVISION:
        raise DemoError(
            f"Nano-vLLM revision mismatch: expected {NANO_SOURCE_REVISION}, got {revision}"
        )
    for relative, expected_size in MODEL_FILES.items():
        path = model / relative
        if not path.is_file():
            raise DemoError(f"missing model artifact: {path}")
        if path.stat().st_size != expected_size:
            raise DemoError(
                f"model size mismatch for {relative}: expected {expected_size}, "
                f"got {path.stat().st_size}"
            )

    resolved_extra = None
    if extra_site is not None:
        resolved_extra = _external_path(extra_site, "extra site-packages")
        if not resolved_extra.is_dir():
            raise DemoError(f"missing extra site-packages directory: {resolved_extra}")

    return {
        "status": "PASS",
        "install_root": str(root),
        "source_revision": revision,
        "expected_nano_version": NANO_VERSION,
        "model_root": str(model),
        "model_revision": MODEL_REVISION,
        "model_files_checked": len(MODEL_FILES),
        "extra_site": str(resolved_extra) if resolved_extra else None,
        "python": str(python),
    }


def _runtime_environment(
    root: Path, cuda_device: int, extra_site: Path | None
) -> dict[str, str]:
    if cuda_device < 0:
        raise DemoError("--cuda-device cannot be negative")
    site = root / "venv" / "lib" / "python3.11" / "site-packages"
    if not site.is_dir():
        raise DemoError(f"missing Python 3.11 site-packages: {site}")

    environment = os.environ.copy()
    python_paths = [root / "source"]
    if extra_site is not None:
        python_paths.insert(0, extra_site)
    environment["PYTHONPATH"] = os.pathsep.join(str(path) for path in python_paths)

    library_paths = [site / "torch" / "lib"]
    library_paths.extend(sorted((site / "nvidia").glob("*/lib")))
    environment["LD_LIBRARY_PATH"] = os.pathsep.join(
        str(path) for path in library_paths if path.is_dir()
    )
    environment["CUDA_VISIBLE_DEVICES"] = str(cuda_device)
    environment["HF_HOME"] = str(root / "hf-cache")
    environment["TRITON_CACHE_DIR"] = str(root / "triton-cache")
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
    raise DemoError("external Nano-vLLM runtime did not emit a structured result")


def _control_for(args: argparse.Namespace) -> str:
    raw = args.control if args.control is not None else VOICE_CONTROLS[args.voice]
    return raw.replace("(", "").replace(")", "").strip()


def _designed_text(control: str, text: str) -> str:
    clean_control = control.replace("(", "").replace(")", "").strip()
    return f"({clean_control}){text}" if clean_control else text


def _validate_runtime_args(args: argparse.Namespace) -> None:
    if not args.text.strip():
        raise DemoError("--text cannot be empty")
    if args.seed < 0:
        raise DemoError("--seed cannot be negative")
    if not 0.1 <= args.cfg <= 10.0:
        raise DemoError("--cfg must be between 0.1 and 10.0")
    if not 1 <= args.steps <= 100:
        raise DemoError("--steps must be between 1 and 100")
    if not 0.0 < args.temperature <= 10.0:
        raise DemoError("--temperature must be between 0 and 10")
    if args.max_generate_length <= 0:
        raise DemoError("--max-generate-length must be positive")
    if not 0.1 <= args.gpu_memory_utilization <= 1.0:
        raise DemoError("--gpu-memory-utilization must be between 0.1 and 1.0")
    if args.max_model_len <= 0 or args.max_num_batched_tokens < args.max_model_len:
        raise DemoError("--max-num-batched-tokens must be at least --max-model-len")
    if args.max_num_seqs <= 0:
        raise DemoError("--max-num-seqs must be positive")


def _worker_command(
    root: Path, model: Path, operation: str, args: argparse.Namespace
) -> list[str]:
    command = [
        str(root / "venv" / "bin" / "python"),
        str(Path(__file__).resolve()),
        "_worker",
        "--operation",
        operation,
        "--install-root",
        str(root),
        "--model-root",
        str(model),
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
                "--temperature",
                str(args.temperature),
                "--max-generate-length",
                str(args.max_generate_length),
                "--gpu-memory-utilization",
                str(args.gpu_memory_utilization),
                "--max-model-len",
                str(args.max_model_len),
                "--max-num-batched-tokens",
                str(args.max_num_batched_tokens),
                "--max-num-seqs",
                str(args.max_num_seqs),
            ]
        )
        if args.enforce_eager:
            command.append("--enforce-eager")
    if operation == "generate":
        command.extend(["--output", str(args.output)])
    elif operation == "benchmark":
        command.extend(
            ["--iterations", str(args.iterations), "--output-dir", str(args.output_dir)]
        )
    return command


def _execute_worker(
    installation: dict[str, Any], operation: str, args: argparse.Namespace
) -> tuple[dict[str, Any], str]:
    root = Path(installation["install_root"])
    model = Path(installation["model_root"])
    extra = Path(installation["extra_site"]) if installation["extra_site"] else None
    environment = _runtime_environment(root, args.cuda_device, extra)
    code, process_wall, peak_gpu, output = _run_monitored(
        _worker_command(root, model, operation, args), environment, args.cuda_device
    )
    if code != 0:
        raise DemoError(
            f"external Nano-vLLM runtime failed with exit code {code}:\n{output}"
        )
    result = _parse_worker_result(output)
    result["process_wall_seconds"] = process_wall
    result["peak_total_gpu_memory_mib"] = peak_gpu
    return result, output


def doctor(args: argparse.Namespace) -> int:
    installation = validate_installation(args.install_root, args.model_root, args.extra_site)
    result, _ = _execute_worker(installation, "doctor", args)
    print(json.dumps({**installation, "runtime": result}, indent=2, sort_keys=True))
    return 0


def generate(args: argparse.Namespace) -> int:
    _validate_runtime_args(args)
    installation = validate_installation(args.install_root, args.model_root, args.extra_site)
    root = Path(installation["install_root"])
    default_output = root / "outputs" / f"voxcpm2-nanovllm-{args.voice}.wav"
    output = _external_path(
        args.output if args.output is not None else default_output, "generated WAV"
    )
    if output.exists() and not args.force:
        raise DemoError(f"output exists; pass --force to replace it: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    args.output = output
    result, runtime_log = _execute_worker(installation, "generate", args)
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
    _validate_runtime_args(args)
    if args.iterations <= 0:
        raise DemoError("--iterations must be positive")
    installation = validate_installation(args.install_root, args.model_root, args.extra_site)
    root = Path(installation["install_root"])
    mode = "eager" if args.enforce_eager else "cuda-graph"
    default_output = (
        root
        / "outputs"
        / f"benchmark-{mode}-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
    )
    output_dir = _external_path(
        args.output_dir if args.output_dir is not None else default_output,
        "benchmark output directory",
    )
    if output_dir.exists():
        raise DemoError(f"benchmark output directory already exists: {output_dir}")
    output_dir.mkdir(parents=True)
    args.output_dir = output_dir
    result, runtime_log = _execute_worker(installation, "benchmark", args)
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


def _worker_imports() -> dict[str, Any]:
    import importlib.metadata

    import flash_attn
    import flash_attn_2_cuda
    import numpy
    import soundfile
    import torch
    import transformers
    from nanovllm_voxcpm import VoxCPM

    return {
        "metadata": importlib.metadata,
        "flash_attn": flash_attn,
        "flash_attn_2_cuda": flash_attn_2_cuda,
        "numpy": numpy,
        "soundfile": soundfile,
        "torch": torch,
        "transformers": transformers,
        "VoxCPM": VoxCPM,
    }


def _worker_doctor(modules: dict[str, Any]) -> dict[str, Any]:
    torch = modules["torch"]
    if not torch.cuda.is_available():
        raise DemoError("PyTorch CUDA is unavailable in the Nano-vLLM runtime")
    if not torch.cuda.is_bf16_supported():
        raise DemoError("GPU does not report BF16 support required by VoxCPM2")
    versions = {
        "nano-vllm-voxcpm": modules["metadata"].version("nano-vllm-voxcpm"),
        "flash-attn": modules["metadata"].version("flash-attn"),
        "torch": torch.__version__,
        "transformers": modules["transformers"].__version__,
    }
    expected_versions = {
        "nano-vllm-voxcpm": NANO_VERSION,
        "flash-attn": FLASH_ATTN_VERSION,
        "torch": TORCH_VERSION,
        "transformers": TRANSFORMERS_VERSION,
    }
    if versions != expected_versions:
        raise DemoError(
            f"runtime version mismatch: expected {expected_versions}, got {versions}"
        )
    extension = Path(modules["flash_attn_2_cuda"].__file__).resolve()
    extension_sha256 = _sha256(extension)
    if extension_sha256 != FLASH_ATTN_EXTENSION_SHA256:
        raise DemoError(
            "FlashAttention extension mismatch: expected "
            f"{FLASH_ATTN_EXTENSION_SHA256}, got {extension_sha256}"
        )
    return {
        "status": "PASS",
        "python": sys.version.split()[0],
        "nano_vllm_voxcpm": versions["nano-vllm-voxcpm"],
        "flash_attn": versions["flash-attn"],
        "flash_attn_module": str(Path(modules["flash_attn"].__file__).resolve()),
        "flash_attn_extension": str(extension),
        "flash_attn_extension_sha256": extension_sha256,
        "torch": versions["torch"],
        "torch_cuda": torch.version.cuda,
        "transformers": versions["transformers"],
        "gpu": torch.cuda.get_device_name(0),
        "compute_capability": list(torch.cuda.get_device_capability(0)),
        "bf16_supported": torch.cuda.is_bf16_supported(),
    }


async def _load_server(
    args: argparse.Namespace, modules: dict[str, Any]
) -> tuple[Any, float, dict[str, Any]]:
    started = time.perf_counter()
    server = modules["VoxCPM"].from_pretrained(
        model=str(args.model_root.resolve()),
        inference_timesteps=args.steps,
        max_num_batched_tokens=args.max_num_batched_tokens,
        max_num_seqs=args.max_num_seqs,
        max_model_len=args.max_model_len,
        gpu_memory_utilization=args.gpu_memory_utilization,
        enforce_eager=args.enforce_eager,
        devices=[0],
    )
    await server.wait_for_ready()
    load_seconds = time.perf_counter() - started
    model_info = dict(await server.get_model_info())
    return server, load_seconds, model_info


async def _synthesize(
    *,
    server: Any,
    modules: dict[str, Any],
    args: argparse.Namespace,
    output: Path,
) -> dict[str, Any]:
    numpy = modules["numpy"]
    soundfile = modules["soundfile"]
    model_info = dict(await server.get_model_info())
    sample_rate = int(model_info["sample_rate"])
    chunks = []
    chunk_audio_seconds = []
    first_chunk_seconds = None
    started = time.perf_counter()
    async for item in server.generate(
        target_text=_designed_text(args.control, args.text),
        max_generate_length=args.max_generate_length,
        temperature=args.temperature,
        cfg_value=args.cfg,
        seed=args.seed,
    ):
        if first_chunk_seconds is None:
            first_chunk_seconds = time.perf_counter() - started
        chunk = numpy.asarray(item, dtype=numpy.float32).reshape(-1)
        chunks.append(chunk)
        chunk_audio_seconds.append(chunk.size / sample_rate)
    wall_seconds = time.perf_counter() - started
    if not chunks:
        raise DemoError("Nano-vLLM produced no audio chunks")
    audio = numpy.ascontiguousarray(numpy.concatenate(chunks), dtype=numpy.float32)
    audio_seconds = audio.size / sample_rate
    output.parent.mkdir(parents=True, exist_ok=True)
    soundfile.write(output, audio, sample_rate, subtype="FLOAT")
    return {
        "output": str(output),
        "bytes": output.stat().st_size,
        "sha256": _sha256(output),
        "pcm_sha256": hashlib.sha256(audio.tobytes()).hexdigest(),
        "sample_rate_hz": sample_rate,
        "channels": 1,
        "frames": audio.size,
        "audio_seconds": audio_seconds,
        "wall_seconds": wall_seconds,
        "rtf": wall_seconds / audio_seconds,
        "rtf_after_first_chunk": (
            (wall_seconds - first_chunk_seconds) / audio_seconds
            if first_chunk_seconds is not None
            else None
        ),
        "chunk_count": len(chunks),
        "chunk_audio_seconds": chunk_audio_seconds,
        "first_chunk_seconds": first_chunk_seconds,
    }


def _request_metadata(args: argparse.Namespace) -> dict[str, Any]:
    return {
        "text": args.text,
        "control": args.control,
        "seed": args.seed,
        "cfg": args.cfg,
        "inference_timesteps": args.steps,
        "temperature": args.temperature,
        "max_generate_length": args.max_generate_length,
        "gpu_memory_utilization": args.gpu_memory_utilization,
        "max_model_len": args.max_model_len,
        "max_num_batched_tokens": args.max_num_batched_tokens,
        "max_num_seqs": args.max_num_seqs,
        "enforce_eager": args.enforce_eager,
    }


async def _worker_generate_async(
    args: argparse.Namespace, modules: dict[str, Any]
) -> dict[str, Any]:
    if args.output is None:
        raise DemoError("worker generate requires --output")
    server, load_seconds, model_info = await _load_server(args, modules)
    try:
        request = await _synthesize(
            server=server, modules=modules, args=args, output=args.output
        )
    finally:
        await server.stop()
    return {
        "status": "PASS",
        "operation": "generate",
        "model_load_seconds": load_seconds,
        "model_info": model_info,
        "request_config": _request_metadata(args),
        "request": request,
    }


async def _worker_benchmark_async(
    args: argparse.Namespace, modules: dict[str, Any]
) -> dict[str, Any]:
    if args.output_dir is None:
        raise DemoError("worker benchmark requires --output-dir")
    output_dir = args.output_dir.resolve()
    server, load_seconds, model_info = await _load_server(args, modules)
    try:
        prewarm = await _synthesize(
            server=server,
            modules=modules,
            args=args,
            output=output_dir / "prewarm.wav",
        )
        warm = []
        for index in range(1, args.iterations + 1):
            warm.append(
                await _synthesize(
                    server=server,
                    modules=modules,
                    args=args,
                    output=output_dir / f"warm_{index}.wav",
                )
            )
    finally:
        await server.stop()

    walls = [item["wall_seconds"] for item in warm]
    rtfs = [item["rtf"] for item in warm]
    rtfs_after_first = [item["rtf_after_first_chunk"] for item in warm]
    audios = [item["audio_seconds"] for item in warm]
    first_chunks = [item["first_chunk_seconds"] for item in warm]
    return {
        "status": "PASS",
        "operation": "benchmark",
        "output_dir": str(output_dir),
        "model_load_seconds": load_seconds,
        "model_info": model_info,
        "request_config": _request_metadata(args),
        "prewarm": prewarm,
        "warm": warm,
        "warm_iterations": len(warm),
        "warm_wall_seconds_mean": statistics.mean(walls),
        "warm_wall_seconds_min": min(walls),
        "warm_wall_seconds_max": max(walls),
        "warm_audio_seconds_mean": statistics.mean(audios),
        "warm_rtf_mean": statistics.mean(rtfs),
        "warm_rtf_after_first_chunk_mean": statistics.mean(rtfs_after_first),
        "warm_x_realtime_mean": statistics.mean(1 / value for value in rtfs),
        "warm_first_chunk_seconds_mean": statistics.mean(first_chunks),
        "fixed_seed_pcm_identical": len({item["pcm_sha256"] for item in warm}) == 1,
    }


def worker(args: argparse.Namespace) -> int:
    import_started = time.perf_counter()
    modules = _worker_imports()
    import_seconds = time.perf_counter() - import_started
    if args.operation == "doctor":
        result = _worker_doctor(modules)
    elif args.operation == "generate":
        result = asyncio.run(_worker_generate_async(args, modules))
    else:
        result = asyncio.run(_worker_benchmark_async(args, modules))
    result["process_import_seconds"] = import_seconds
    print(RESULT_PREFIX + json.dumps(result, sort_keys=True))
    return 0


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        if args.command == "doctor":
            return doctor(args)
        if args.command == "generate":
            return generate(args)
        if args.command == "benchmark":
            return benchmark(args)
        return worker(args)
    except DemoError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
