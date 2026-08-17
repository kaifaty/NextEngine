#!/usr/bin/env python3
"""Run the pinned external Fun-CosyVoice3-0.5B-2512 CUDA demo."""

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

SOURCE_REVISION = "074ca6dc9e80a2f424f1f74b48bdd7d3fea531cc"
MATCHA_REVISION = "dd9105b34bf2be2230f4aa1e4769fb586a3c824e"
MODEL_REVISION = "29e01c4e8d000f4bcd70751be16fa94bf3d85a18"
DEFAULT_TEXT = "Привет! Сервер синтеза речи работает локально."
COSYVOICE3_PREFIX = "You are a helpful assistant.<|endofprompt|>"
REFERENCE_RELATIVE_PATH = Path("source/asset/zero_shot_prompt.wav")
REFERENCE_SIZE = 334_138
REFERENCE_SHA256 = "c7b31d6dbe7cc6a716dded00550db5b50940bf209e424e4ad207b12e657c8ff6"
REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
RESULT_PREFIX = "NEXTENGINE_COSYVOICE_RESULT="
SPEAKER_ID = "nextengine-official-demo"


@dataclass(frozen=True)
class Artifact:
    size: int
    sha256: str


MODEL_ARTIFACTS = {
    Path("CosyVoice-BlankEN/config.json"): Artifact(
        659, "168aa1bd401abc3bc262ba15ba4e499627a8b4e006e9d050b47c22de20660185"
    ),
    Path("CosyVoice-BlankEN/generation_config.json"): Artifact(
        242, "e558847a8b4402616f1273797b015104dc266fe4b520056fca88823ba8f8ebe6"
    ),
    Path("CosyVoice-BlankEN/merges.txt"): Artifact(
        1_402_109,
        "ac8ff86a72bee70828fbc1119bc4398c6f3a9a6e490d7b0dbe917be025478bd0",
    ),
    Path("CosyVoice-BlankEN/model.safetensors"): Artifact(
        988_097_824,
        "130282af0dfa9fe5840737cc49a0d339d06075f83c5a315c3372c9a0740d0b96",
    ),
    Path("CosyVoice-BlankEN/tokenizer_config.json"): Artifact(
        1_287, "482bd979881423375ca5414e4e0d94cd7c5349dbb17fffd46b4d36d71e62a1bc"
    ),
    Path("CosyVoice-BlankEN/vocab.json"): Artifact(
        2_776_833,
        "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910",
    ),
    Path("README.md"): Artifact(
        11_982, "d4ec0fe1342b60a424ce1cd8971c670d26be38068a96148bcb6a1b8a334c3088"
    ),
    Path("campplus.onnx"): Artifact(
        28_303_423,
        "a6ac6a63997761ae2997373e2ee1c47040854b4b759ea41ec48e4e42df0f4d73",
    ),
    Path("config.json"): Artifact(
        2, "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
    ),
    Path("configuration.json"): Artifact(
        47, "c502b6328c67638b401df8dd05de89e9e8d1cff9cd0ada10dfbdbe13556c20de"
    ),
    Path("cosyvoice3.yaml"): Artifact(
        6_934, "f5a6b2c6f05139d0f18861a1fe506f751e787026b77c05f7e8fef9f8a4405965"
    ),
    Path("flow.pt"): Artifact(
        1_329_116_148,
        "a6fab32a7825e5b0bc855ddd948f8db9370b0a786fbc249caa4595e95b608e4b",
    ),
    Path("hift.pt"): Artifact(
        83_202_622,
        "b279d7641eb97ae55b3b540cfba4f953c26492a2df758328a89a4d007ab87a65",
    ),
    Path("llm.pt"): Artifact(
        2_024_669_519,
        "69f43bd545131c30e98947fb360ea8b4dc9916d8e83dded7757c7ea4f5a24970",
    ),
    Path("speech_tokenizer_v3.onnx"): Artifact(
        969_451_503,
        "23236a74175dbdda47afc66dbadd5bcb41303c467a57c261cb8539ad9db9208d",
    ),
}


class DemoError(RuntimeError):
    """A stable, user-actionable demo setup error."""


def _default_root() -> Path:
    return Path(
        os.environ.get(
            "NEXTENGINE_COSYVOICE3_ROOT",
            Path.home()
            / ".local"
            / "share"
            / "nextengine"
            / "fun-cosyvoice3-0.5b-2512",
        )
    )


def _add_install_options(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--install-root", type=Path, default=_default_root())
    parser.add_argument("--cuda-device", type=int, default=0)
    parser.add_argument(
        "--skip-model-hash-check",
        action="store_true",
        help="Skip the 5.1 GiB content hash pass; sizes and pinned source still validate.",
    )


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    effective_argv = list(sys.argv[1:] if argv is None else argv)
    if effective_argv and effective_argv[0] == "_worker":
        return _parse_worker_args(effective_argv[1:])
    parser = argparse.ArgumentParser(
        description="Run Fun-CosyVoice3 through a pinned external CUDA environment."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    doctor = subparsers.add_parser("doctor", help="Validate the external installation.")
    _add_install_options(doctor)

    generate = subparsers.add_parser("generate", help="Generate one external WAV.")
    _add_install_options(generate)
    generate.add_argument("--text", default=DEFAULT_TEXT)
    generate.add_argument("--output", type=Path)
    generate.add_argument("--seed", type=int, default=1234)
    generate.add_argument("--stream", action="store_true")
    generate.add_argument("--fp32", action="store_true")
    generate.add_argument("--force", action="store_true")

    benchmark = subparsers.add_parser(
        "benchmark",
        help="Run one prewarm and repeated measured requests in one model session.",
    )
    _add_install_options(benchmark)
    benchmark.add_argument("--text", default=DEFAULT_TEXT)
    benchmark.add_argument("--seed", type=int, default=1234)
    benchmark.add_argument("--iterations", type=int, default=3)
    benchmark.add_argument("--stream", action="store_true")
    benchmark.add_argument("--fp32", action="store_true")
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
    worker.add_argument("--seed", type=int, default=1234)
    worker.add_argument("--iterations", type=int, default=3)
    worker.add_argument("--stream", action="store_true")
    worker.add_argument("--fp32", action="store_true")
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
    matcha = source / "third_party" / "Matcha-TTS"
    model = root / "model"
    python = root / "venv" / "bin" / "python"
    reference = root / REFERENCE_RELATIVE_PATH

    for label, path in (
        ("CosyVoice source", source),
        ("Matcha-TTS source", matcha),
        ("model directory", model),
        ("Python runtime", python),
        ("reference WAV", reference),
    ):
        if not path.exists():
            raise DemoError(f"missing {label}: {path}")
    if not python.is_file() or not os.access(python, os.X_OK):
        raise DemoError(f"Python runtime is not executable: {python}")

    source_revision = _git_revision(source)
    if source_revision != SOURCE_REVISION:
        raise DemoError(
            f"CosyVoice revision mismatch: expected {SOURCE_REVISION}, got {source_revision}"
        )
    matcha_revision = _git_revision(matcha)
    if matcha_revision != MATCHA_REVISION:
        raise DemoError(
            f"Matcha-TTS revision mismatch: expected {MATCHA_REVISION}, got {matcha_revision}"
        )

    checked_hashes: dict[str, str] = {}
    for relative, artifact in MODEL_ARTIFACTS.items():
        path = model / relative
        if not path.is_file():
            raise DemoError(f"missing model artifact: {path}")
        actual_size = path.stat().st_size
        if actual_size != artifact.size:
            raise DemoError(
                f"model size mismatch for {relative}: expected {artifact.size}, got {actual_size}"
            )
        if verify_model_hash:
            actual_hash = _sha256(path)
            if actual_hash != artifact.sha256:
                raise DemoError(
                    f"model SHA-256 mismatch for {relative}: "
                    f"expected {artifact.sha256}, got {actual_hash}"
                )
            checked_hashes[str(relative)] = actual_hash

    if reference.stat().st_size != REFERENCE_SIZE:
        raise DemoError(
            f"reference size mismatch: expected {REFERENCE_SIZE}, got {reference.stat().st_size}"
        )
    reference_hash = _sha256(reference)
    if reference_hash != REFERENCE_SHA256:
        raise DemoError(
            f"reference SHA-256 mismatch: expected {REFERENCE_SHA256}, got {reference_hash}"
        )

    return {
        "status": "PASS",
        "install_root": str(root),
        "source_revision": source_revision,
        "matcha_revision": matcha_revision,
        "model_revision": MODEL_REVISION,
        "model_bytes": sum(item.size for item in MODEL_ARTIFACTS.values()),
        "model_hashes_checked": verify_model_hash,
        "checked_hashes": checked_hashes,
        "python": str(python),
        "reference": str(reference),
        "reference_sha256": reference_hash,
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
    site_packages = root / "venv" / "lib" / "python3.10" / "site-packages"
    if not site_packages.is_dir():
        raise DemoError(f"missing Python 3.10 site-packages: {site_packages}")
    python_paths = [root / "source", root / "source" / "third_party" / "Matcha-TTS"]
    existing_python_path = environment.get("PYTHONPATH")
    if existing_python_path:
        python_paths.append(Path(existing_python_path))
    environment["PYTHONPATH"] = os.pathsep.join(str(path) for path in python_paths)

    library_paths = [site_packages / "torch" / "lib"]
    library_paths.extend(sorted((site_packages / "nvidia").glob("*/lib")))
    existing_library_path = environment.get("LD_LIBRARY_PATH")
    library_path_text = [str(path) for path in library_paths if path.is_dir()]
    if existing_library_path:
        library_path_text.append(existing_library_path)
    environment["LD_LIBRARY_PATH"] = os.pathsep.join(library_path_text)
    environment["CUDA_VISIBLE_DEVICES"] = str(cuda_device)
    environment["HF_HOME"] = str(root / "hf-cache")
    environment["MODELSCOPE_CACHE"] = str(root / "modelscope-cache")
    environment["MPLCONFIGDIR"] = str(root / "matplotlib-cache")
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
    start = time.perf_counter()
    process = subprocess.Popen(
        command,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    peak_gpu_memory = _sample_gpu_memory(cuda_device)
    try:
        while process.poll() is None:
            sample = _sample_gpu_memory(cuda_device)
            if sample is not None:
                peak_gpu_memory = max(peak_gpu_memory or 0, sample)
            time.sleep(0.1)
    except KeyboardInterrupt:
        process.terminate()
        process.wait()
        raise
    stdout, _ = process.communicate()
    return (
        process.returncode,
        time.perf_counter() - start,
        peak_gpu_memory,
        stdout,
    )


def _parse_worker_result(output: str) -> dict[str, Any]:
    for line in reversed(output.splitlines()):
        if line.startswith(RESULT_PREFIX):
            return json.loads(line.removeprefix(RESULT_PREFIX))
    raise DemoError("external runtime did not emit a structured result")


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
        command.extend(["--text", args.text, "--seed", str(args.seed)])
        if args.stream:
            command.append("--stream")
        if args.fp32:
            command.append("--fp32")
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
        raise DemoError(
            f"external CosyVoice runtime failed with exit code {code}:\n{output}"
        )
    result = _parse_worker_result(output)
    result["process_wall_seconds"] = process_wall
    result["peak_total_gpu_memory_mib"] = peak_gpu
    return result, output


def doctor(args: argparse.Namespace) -> int:
    installation = validate_installation(
        args.install_root, verify_model_hash=not args.skip_model_hash_check
    )
    root = Path(installation["install_root"])
    result, _ = _execute_worker(root, "doctor", args)
    print(json.dumps({**installation, "runtime": result}, indent=2, sort_keys=True))
    return 0


def generate(args: argparse.Namespace) -> int:
    if not args.text.strip():
        raise DemoError("--text cannot be empty")
    if args.seed < 0:
        raise DemoError("--seed cannot be negative")
    installation = validate_installation(
        args.install_root, verify_model_hash=not args.skip_model_hash_check
    )
    root = Path(installation["install_root"])
    default_name = (
        "fun-cosyvoice3-russian-stream.wav"
        if args.stream
        else "fun-cosyvoice3-russian-demo.wav"
    )
    output = _external_path(
        args.output if args.output is not None else root / "outputs" / default_name,
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
    if not args.text.strip():
        raise DemoError("--text cannot be empty")
    if args.seed < 0:
        raise DemoError("--seed cannot be negative")
    if args.iterations <= 0:
        raise DemoError("--iterations must be positive")
    installation = validate_installation(
        args.install_root, verify_model_hash=not args.skip_model_hash_check
    )
    root = Path(installation["install_root"])
    suffix = "stream" if args.stream else "offline"
    default_output = (
        root
        / "outputs"
        / f"benchmark-{suffix}-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
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


def _seed_runtime(seed: int, random: Any, numpy: Any, torch: Any) -> None:
    random.seed(seed)
    numpy.random.seed(seed)
    torch.manual_seed(seed)
    torch.cuda.manual_seed_all(seed)


def _prefixed_text(text: str) -> str:
    return text if "<|endofprompt|>" in text else COSYVOICE3_PREFIX + text


def _worker_imports() -> dict[str, Any]:
    import importlib.metadata
    import random

    import numpy
    import onnxruntime
    import torch
    import torchaudio
    from cosyvoice.cli.cosyvoice import AutoModel

    return {
        "metadata": importlib.metadata,
        "random": random,
        "numpy": numpy,
        "onnxruntime": onnxruntime,
        "torch": torch,
        "torchaudio": torchaudio,
        "AutoModel": AutoModel,
    }


def _worker_doctor(root: Path, modules: dict[str, Any]) -> dict[str, Any]:
    torch = modules["torch"]
    onnxruntime = modules["onnxruntime"]
    metadata = modules["metadata"]
    if not torch.cuda.is_available():
        raise DemoError("PyTorch CUDA is unavailable in the external runtime")
    session = onnxruntime.InferenceSession(
        str(root / "model" / "speech_tokenizer_v3.onnx"),
        providers=["CUDAExecutionProvider"],
    )
    if session.get_providers()[0] != "CUDAExecutionProvider":
        raise DemoError(
            f"speech tokenizer did not select CUDA: {session.get_providers()}"
        )
    return {
        "status": "PASS",
        "python": sys.version.split()[0],
        "torch": modules["torch"].__version__,
        "torchaudio": modules["torchaudio"].__version__,
        "torch_cuda": modules["torch"].version.cuda,
        "onnxruntime": onnxruntime.__version__,
        "onnxruntime_providers": onnxruntime.get_available_providers(),
        "speech_tokenizer_providers": session.get_providers(),
        "transformers": metadata.version("transformers"),
        "gpu": torch.cuda.get_device_name(0),
    }


def _worker_load_model(
    root: Path, fp16: bool, modules: dict[str, Any]
) -> tuple[Any, float]:
    torch = modules["torch"]
    auto_model = modules["AutoModel"]
    load_start = time.perf_counter()
    model = auto_model(model_dir=str(root / "model"), fp16=fp16)
    torch.cuda.synchronize()
    load_seconds = time.perf_counter() - load_start
    return model, load_seconds


def _worker_prepare_speaker(
    root: Path, model: Any, modules: dict[str, Any]
) -> float:
    torch = modules["torch"]
    start = time.perf_counter()
    model.add_zero_shot_spk(
        COSYVOICE3_PREFIX,
        str(root / REFERENCE_RELATIVE_PATH),
        SPEAKER_ID,
    )
    torch.cuda.synchronize()
    return time.perf_counter() - start


def _worker_synthesize(
    *,
    root: Path,
    model: Any,
    modules: dict[str, Any],
    text: str,
    seed: int,
    stream: bool,
    output: Path,
) -> dict[str, Any]:
    torch = modules["torch"]
    torchaudio = modules["torchaudio"]
    _seed_runtime(seed, modules["random"], modules["numpy"], torch)
    torch.cuda.synchronize()
    started = time.perf_counter()
    first_chunk_seconds: float | None = None
    chunks = []
    chunk_audio_seconds = []
    token_hop_len_before = getattr(model.model, "token_hop_len", None)
    for item in model.inference_cross_lingual(
        _prefixed_text(text),
        str(root / REFERENCE_RELATIVE_PATH),
        zero_shot_spk_id=SPEAKER_ID,
        stream=stream,
        text_frontend=False,
    ):
        if first_chunk_seconds is None:
            first_chunk_seconds = time.perf_counter() - started
        chunks.append(item["tts_speech"])
        chunk_audio_seconds.append(item["tts_speech"].shape[1] / model.sample_rate)
    torch.cuda.synchronize()
    wall_seconds = time.perf_counter() - started
    if not chunks:
        raise DemoError("CosyVoice produced no audio chunks")
    audio = torch.cat(chunks, dim=1).detach().cpu().contiguous()
    audio_seconds = audio.shape[1] / model.sample_rate
    pcm_sha256 = hashlib.sha256(audio.numpy().tobytes()).hexdigest()
    output.parent.mkdir(parents=True, exist_ok=True)
    torchaudio.save(
        str(output),
        audio,
        model.sample_rate,
        encoding="PCM_F",
        bits_per_sample=32,
    )
    return {
        "output": str(output),
        "bytes": output.stat().st_size,
        "sha256": _sha256(output),
        "pcm_sha256": pcm_sha256,
        "sample_rate_hz": model.sample_rate,
        "channels": audio.shape[0],
        "frames": audio.shape[1],
        "audio_seconds": audio_seconds,
        "wall_seconds": wall_seconds,
        "rtf": wall_seconds / audio_seconds,
        "stream": stream,
        "chunk_count": len(chunks),
        "chunk_audio_seconds": chunk_audio_seconds,
        "first_chunk_seconds": first_chunk_seconds if stream else None,
        "token_hop_len_before": token_hop_len_before,
        "token_hop_len_after": getattr(model.model, "token_hop_len", None),
    }


def _worker_generate(args: argparse.Namespace, modules: dict[str, Any]) -> dict[str, Any]:
    if args.output is None:
        raise DemoError("worker generate requires --output")
    root = args.install_root.resolve()
    torch = modules["torch"]
    torch.cuda.reset_peak_memory_stats()
    model, load_seconds = _worker_load_model(root, not args.fp32, modules)
    speaker_seconds = _worker_prepare_speaker(root, model, modules)
    request = _worker_synthesize(
        root=root,
        model=model,
        modules=modules,
        text=args.text,
        seed=args.seed,
        stream=args.stream,
        output=args.output,
    )
    return {
        "status": "PASS",
        "operation": "generate",
        "precision": "fp32" if args.fp32 else "fp16-autocast",
        "model_load_seconds": load_seconds,
        "speaker_prepare_seconds": speaker_seconds,
        "torch_peak_allocated_mib": torch.cuda.max_memory_allocated() / 1024**2,
        "torch_peak_reserved_mib": torch.cuda.max_memory_reserved() / 1024**2,
        "request": request,
    }


def _worker_benchmark(args: argparse.Namespace, modules: dict[str, Any]) -> dict[str, Any]:
    if args.output_dir is None:
        raise DemoError("worker benchmark requires --output-dir")
    root = args.install_root.resolve()
    output_dir = args.output_dir.resolve()
    torch = modules["torch"]
    torch.cuda.reset_peak_memory_stats()
    model, load_seconds = _worker_load_model(root, not args.fp32, modules)
    speaker_seconds = _worker_prepare_speaker(root, model, modules)
    prewarm = _worker_synthesize(
        root=root,
        model=model,
        modules=modules,
        text=args.text,
        seed=args.seed,
        stream=args.stream,
        output=output_dir / "prewarm.wav",
    )
    warm = []
    for index in range(1, args.iterations + 1):
        warm.append(
            _worker_synthesize(
                root=root,
                model=model,
                modules=modules,
                text=args.text,
                seed=args.seed,
                stream=args.stream,
                output=output_dir / f"warm_{index}.wav",
            )
        )
    wall_values = [item["wall_seconds"] for item in warm]
    rtf_values = [item["rtf"] for item in warm]
    audio_values = [item["audio_seconds"] for item in warm]
    first_chunk_values = [
        item["first_chunk_seconds"]
        for item in warm
        if item["first_chunk_seconds"] is not None
    ]
    return {
        "status": "PASS",
        "operation": "benchmark",
        "output_dir": str(output_dir),
        "precision": "fp32" if args.fp32 else "fp16-autocast",
        "stream": args.stream,
        "seed": args.seed,
        "text": args.text,
        "model_load_seconds": load_seconds,
        "speaker_prepare_seconds": speaker_seconds,
        "prewarm": prewarm,
        "warm": warm,
        "warm_iterations": len(warm),
        "warm_wall_seconds_mean": statistics.mean(wall_values),
        "warm_wall_seconds_min": min(wall_values),
        "warm_wall_seconds_max": max(wall_values),
        "warm_audio_seconds_mean": statistics.mean(audio_values),
        "warm_rtf_mean": statistics.mean(rtf_values),
        "warm_x_realtime_mean": statistics.mean(1 / value for value in rtf_values),
        "warm_first_chunk_seconds_mean": (
            statistics.mean(first_chunk_values) if first_chunk_values else None
        ),
        "fixed_seed_outputs_byte_identical": len(
            {item["sha256"] for item in warm}
        )
        == 1,
        "fixed_seed_pcm_identical": len({item["pcm_sha256"] for item in warm})
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
        result = _worker_doctor(root, modules)
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
