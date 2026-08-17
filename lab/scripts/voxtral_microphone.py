#!/usr/bin/env python3
"""Minimal live-microphone frontend for Voxtral Realtime via transcribe.cpp.

The tool captures 16 kHz mono S16_LE PCM from ALSA ``arecord`` and feeds it
directly into the transcribe.cpp streaming API. Audio is kept in memory and is
not written to the repository or a temporary file.
"""

from __future__ import annotations

import argparse
import array
import importlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from types import ModuleType
from typing import BinaryIO, Iterator, Sequence


SAMPLE_RATE = 16_000
SAMPLE_WIDTH_BYTES = 2
VALID_DELAYS_MS = tuple(range(80, 1_201, 80)) + (2_400,)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Stream the Linux microphone into Voxtral Realtime.",
    )
    parser.add_argument("model", nargs="?", help="path to a Voxtral Realtime GGUF")
    parser.add_argument(
        "--transcribe-root",
        type=Path,
        default=None,
        help="transcribe.cpp checkout (or set TRANSCRIBE_CPP_ROOT)",
    )
    parser.add_argument(
        "--library",
        type=Path,
        default=None,
        help="shared libtranscribe path (or set TRANSCRIBE_LIBRARY)",
    )
    parser.add_argument("--backend", default="cuda", choices=("auto", "cpu", "cuda", "vulkan"))
    parser.add_argument("--device", default="default", help="ALSA capture device (default: default)")
    parser.add_argument("--chunk-ms", type=int, default=250, help="microphone feed size (default: 250)")
    parser.add_argument(
        "--delay-ms",
        type=int,
        default=480,
        choices=VALID_DELAYS_MS,
        help="model transcription delay (default: 480)",
    )
    parser.add_argument("--language", default=None, help="optional language hint, e.g. ru")
    parser.add_argument(
        "--duration",
        type=float,
        default=None,
        help="stop after this many seconds; otherwise use Ctrl-C",
    )
    parser.add_argument(
        "--list-inputs",
        action="store_true",
        help="list ALSA capture devices and exit; model is not required",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="load model and validate streaming support without opening the microphone",
    )
    return parser


def transcribe_root(args: argparse.Namespace) -> Path:
    value = args.transcribe_root or os.environ.get("TRANSCRIBE_CPP_ROOT")
    if not value:
        raise SystemExit("set --transcribe-root or TRANSCRIBE_CPP_ROOT")
    root = Path(value).expanduser().resolve()
    binding = root / "bindings" / "python" / "src" / "transcribe_cpp"
    if not binding.is_dir():
        raise SystemExit(f"transcribe.cpp Python binding not found: {binding}")
    return root


def find_library(root: Path, explicit: Path | None) -> Path:
    configured = explicit or os.environ.get("TRANSCRIBE_LIBRARY")
    if configured:
        library = Path(configured).expanduser().resolve()
        if not library.is_file():
            raise SystemExit(f"libtranscribe not found: {library}")
        return library

    candidates = (
        root / "build" / "src" / "libtranscribe.so",
        root / "build-shared" / "src" / "libtranscribe.so",
        root / "build" / "src" / "libtranscribe.dylib",
        root / "build-shared" / "src" / "libtranscribe.dylib",
        root / "build" / "bin" / "transcribe.dll",
        root / "build-shared" / "bin" / "transcribe.dll",
    )
    for candidate in candidates:
        if candidate.is_file():
            return candidate.resolve()
    raise SystemExit(
        "shared libtranscribe was not found; build with "
        "-DTRANSCRIBE_BUILD_SHARED=ON or pass --library"
    )


def load_transcribe(root: Path, library: Path) -> ModuleType:
    os.environ["TRANSCRIBE_LIBRARY"] = os.fspath(library)
    binding_src = root / "bindings" / "python" / "src"
    sys.path.insert(0, os.fspath(binding_src))
    try:
        return importlib.import_module("transcribe_cpp")
    except Exception as error:
        raise SystemExit(f"failed to load transcribe.cpp Python binding: {error}") from error


def arecord_command(device: str) -> list[str]:
    executable = shutil.which("arecord")
    if executable is None:
        raise SystemExit("arecord is required (install alsa-utils)")
    return [
        executable,
        "--quiet",
        "--device",
        device,
        "--format",
        "S16_LE",
        "--rate",
        str(SAMPLE_RATE),
        "--channels",
        "1",
        "--file-type",
        "raw",
    ]


def pcm16le_to_float32(data: bytes) -> array.array:
    if len(data) % SAMPLE_WIDTH_BYTES:
        raise ValueError("PCM byte count must be aligned to signed 16-bit samples")
    pcm16 = array.array("h")
    pcm16.frombytes(data)
    if sys.byteorder == "big":
        pcm16.byteswap()
    return array.array("f", (sample / 32768.0 for sample in pcm16))


def pcm_chunks(
    source: BinaryIO,
    chunk_samples: int,
    max_samples: int | None,
) -> Iterator[array.array]:
    pending = bytearray()
    yielded = 0
    chunk_bytes = chunk_samples * SAMPLE_WIDTH_BYTES
    while max_samples is None or yielded < max_samples:
        data = source.read(chunk_bytes - len(pending))
        if not data:
            if pending:
                usable = len(pending) - len(pending) % SAMPLE_WIDTH_BYTES
                if usable:
                    remaining = None if max_samples is None else max_samples - yielded
                    raw = bytes(pending[:usable])
                    pcm = pcm16le_to_float32(raw)
                    yield pcm if remaining is None else pcm[:remaining]
            return
        pending.extend(data)
        if len(pending) < chunk_bytes:
            continue
        raw = bytes(pending[:chunk_bytes])
        del pending[:chunk_bytes]
        pcm = pcm16le_to_float32(raw)
        if max_samples is not None:
            pcm = pcm[: max_samples - yielded]
        yielded += len(pcm)
        if pcm:
            yield pcm


def stop_capture(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def render(text: object, *, interactive: bool, previous: str) -> str:
    committed = str(getattr(text, "committed"))
    tentative = str(getattr(text, "tentative"))
    display = committed + tentative
    if display == previous:
        return previous
    if interactive:
        print(f"\r\x1b[K{committed}\x1b[2m{tentative}\x1b[0m", end="", flush=True)
    else:
        print(display, flush=True)
    return display


def list_inputs() -> int:
    executable = shutil.which("arecord")
    if executable is None:
        raise SystemExit("arecord is required (install alsa-utils)")
    return subprocess.run([executable, "-L"], check=False).returncode


def validate_args(parser: argparse.ArgumentParser, args: argparse.Namespace) -> None:
    if args.list_inputs:
        return
    if not args.model:
        parser.error("model is required unless --list-inputs is used")
    if args.chunk_ms <= 0:
        parser.error("--chunk-ms must be positive")
    if args.duration is not None and args.duration <= 0:
        parser.error("--duration must be positive")
    if not Path(args.model).expanduser().is_file():
        parser.error(f"model not found: {args.model}")


def run(args: argparse.Namespace) -> int:
    root = transcribe_root(args)
    library = find_library(root, args.library)
    transcribe_cpp = load_transcribe(root, library)
    model_path = Path(args.model).expanduser().resolve()

    print(f"loading {model_path.name} with backend={args.backend} ...", flush=True)
    with transcribe_cpp.Model(model_path, backend=args.backend) as model:
        if not model.capabilities.supports_streaming:
            raise SystemExit(f"{model.arch}/{model.variant} does not support streaming")
        print(f"ready: {model.arch}/{model.variant} on {model.backend}")
        if args.check:
            return 0

        command = arecord_command(args.device)
        chunk_samples = max(1, SAMPLE_RATE * args.chunk_ms // 1_000)
        max_samples = None if args.duration is None else int(SAMPLE_RATE * args.duration)
        delay_tokens = args.delay_ms // 80
        family = transcribe_cpp.VoxtralRealtimeStreamOptions(
            num_delay_tokens=delay_tokens,
        )
        print(
            f"microphone={args.device}, chunk={args.chunk_ms} ms, "
            f"delay={args.delay_ms} ms; speak now (Ctrl-C to stop)"
        )

        capture = subprocess.Popen(command, stdout=subprocess.PIPE)
        if capture.stdout is None:
            stop_capture(capture)
            raise SystemExit("arecord did not provide a PCM stream")

        interactive = sys.stdout.isatty()
        previous = ""
        captured = 0
        started = time.monotonic()
        try:
            with model.session() as session:
                with session.stream(language=args.language, family=family) as stream:
                    try:
                        for chunk in pcm_chunks(capture.stdout, chunk_samples, max_samples):
                            captured += len(chunk)
                            update = stream.feed(chunk)
                            if update.committed_changed or update.tentative_changed:
                                previous = render(
                                    stream.text(), interactive=interactive, previous=previous
                                )
                    except KeyboardInterrupt:
                        pass
                    finally:
                        stop_capture(capture)
                    if captured:
                        stream.finalize()
                        final = stream.text()
                        render(final, interactive=interactive, previous=previous)
                        if interactive:
                            print()
                        print(f"\nfinal:\n{final.committed.strip()}")
        finally:
            stop_capture(capture)

        audio_seconds = captured / SAMPLE_RATE
        wall_seconds = time.monotonic() - started
        print(f"audio={audio_seconds:.2f}s wall={wall_seconds:.2f}s")
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    validate_args(parser, args)
    if args.list_inputs:
        return list_inputs()
    return run(args)


if __name__ == "__main__":
    raise SystemExit(main())
