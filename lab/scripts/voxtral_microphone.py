#!/usr/bin/env python3
"""Minimal live-microphone frontend for Voxtral Realtime via transcribe.cpp.

The tool captures 16 kHz mono S16_LE PCM from ALSA ``arecord`` or an exact
PipeWire source through ``pw-record`` and feeds it directly into the
transcribe.cpp streaming API. Audio is kept in memory and is not written to
the repository or a temporary file.
"""

from __future__ import annotations

import argparse
import array
import importlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
from types import ModuleType
from typing import BinaryIO, Iterator, Sequence


SAMPLE_RATE = 16_000
SAMPLE_WIDTH_BYTES = 2
VALID_DELAYS_MS = tuple(range(80, 1_201, 80)) + (2_400,)
DEFAULT_SILENCE_THRESHOLD_DBFS = -65.0


class SingleUseAction(argparse.Action):
    """Reject repeated value options instead of silently accepting the last."""

    def __call__(
        self,
        parser: argparse.ArgumentParser,
        namespace: argparse.Namespace,
        values: object,
        option_string: str | None = None,
    ) -> None:
        marker = f"_{self.dest}_was_set"
        if getattr(namespace, marker, False):
            parser.error(f"{option_string or self.dest} was specified more than once")
        setattr(namespace, marker, True)
        setattr(namespace, self.dest, values)


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
    parser.add_argument(
        "--device",
        action=SingleUseAction,
        default="default",
        help="ALSA device or pw:<exact PipeWire node name> (default: default)",
    )
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
        help="list ALSA and PipeWire capture sources; model is not required",
    )
    parser.add_argument(
        "--probe-microphone",
        action="store_true",
        help="measure the selected input and exit; model is not required",
    )
    parser.add_argument(
        "--probe-seconds",
        type=int,
        default=2,
        help="microphone preflight duration (default: 2)",
    )
    parser.add_argument(
        "--silence-threshold-dbfs",
        type=float,
        default=DEFAULT_SILENCE_THRESHOLD_DBFS,
        help="fail when preflight peak is below this level (default: -65)",
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


def capture_command(device: str, *, sample_count: int | None = None) -> list[str]:
    if device.startswith("pw:"):
        target = device.removeprefix("pw:")
        if not target:
            raise SystemExit("PipeWire device must be pw:<node name or serial>")
        executable = shutil.which("pw-record")
        if executable is None:
            raise SystemExit("pw-record is required for pw: devices")
        command = [
            executable,
            "--target",
            target,
            "--rate",
            str(SAMPLE_RATE),
            "--channels",
            "1",
            "--format",
            "s16",
            "--raw",
        ]
        if sample_count is not None:
            command.extend(("--sample-count", str(sample_count)))
        command.append("-")
        return command

    executable = shutil.which("arecord")
    if executable is None:
        raise SystemExit("arecord is required (install alsa-utils)")
    command = [
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
    if sample_count is not None:
        command.extend(("--samples", str(sample_count)))
    return command


def pcm16le_to_float32(data: bytes) -> array.array:
    if len(data) % SAMPLE_WIDTH_BYTES:
        raise ValueError("PCM byte count must be aligned to signed 16-bit samples")
    pcm16 = array.array("h")
    pcm16.frombytes(data)
    if sys.byteorder == "big":
        pcm16.byteswap()
    return array.array("f", (sample / 32768.0 for sample in pcm16))


def signal_levels_dbfs(data: bytes) -> tuple[float, float]:
    pcm = pcm16le_to_float32(data)
    if not pcm:
        return -math.inf, -math.inf
    peak = max(abs(sample) for sample in pcm)
    mean_square = sum(sample * sample for sample in pcm) / len(pcm)
    rms = math.sqrt(mean_square)
    rms_dbfs = -math.inf if rms == 0.0 else 20.0 * math.log10(rms)
    peak_dbfs = -math.inf if peak == 0.0 else 20.0 * math.log10(peak)
    return rms_dbfs, peak_dbfs


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


def alsa_hardware_inputs(output: str) -> list[tuple[str, str]]:
    inputs = []
    pattern = re.compile(
        r"^card\s+\d+:\s+(\S+)\s+\[([^]]+)],\s+device\s+(\d+):\s+([^[]+)",
        re.MULTILINE,
    )
    for card, card_description, device, device_description in pattern.findall(output):
        name = f"plughw:CARD={card},DEV={device}"
        description = f"{card_description.strip()} / {device_description.strip()}"
        inputs.append((name, description))
    return inputs


def pipewire_inputs() -> list[tuple[str, str, str | None]]:
    executable = shutil.which("pw-dump")
    if executable is None:
        return []
    result = subprocess.run(
        [executable], check=False, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
    )
    if result.returncode != 0:
        return []
    try:
        objects = json.loads(result.stdout)
    except json.JSONDecodeError:
        return []
    devices = {
        item.get("id"): item
        for item in objects
        if item.get("info", {}).get("props", {}).get("media.class") == "Audio/Device"
    }
    inputs = []
    for item in objects:
        props = item.get("info", {}).get("props", {})
        if props.get("media.class") != "Audio/Source":
            continue
        name = props.get("node.name")
        if not name:
            continue
        warning = None
        device = devices.get(props.get("device.id"))
        if device is not None:
            device_info = device.get("info", {})
            device_props = device_info.get("props", {})
            if device_props.get("device.api") == "bluez5":
                current_profiles = device_info.get("params", {}).get("Profile", [])
                current_profile_has_input = any(
                    isinstance(profile_class, list)
                    and bool(profile_class)
                    and profile_class[0] == "Audio/Source"
                    for profile in current_profiles
                    for profile_class in profile.get("classes", [])
                )
                if not current_profile_has_input:
                    warning = "Bluetooth capture profile is inactive"
            input_routes = [
                route
                for route in device_info.get("params", {}).get("EnumRoute", [])
                if route.get("direction") == "Input"
            ]
            if input_routes and all(route.get("available") == "no" for route in input_routes):
                warning = "no connected physical input port"
        inputs.append((f"pw:{name}", props.get("node.description", name), warning))
    return sorted(set(inputs))


def list_inputs() -> int:
    executable = shutil.which("arecord")
    if executable is None:
        raise SystemExit("arecord is required (install alsa-utils)")
    result = subprocess.run(
        [executable, "-l"], check=False, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
    )
    print("ALSA hardware capture devices:")
    alsa_inputs = alsa_hardware_inputs(result.stdout)
    if alsa_inputs:
        for name, description in alsa_inputs:
            print(f"  {name}\n    {description}")
    else:
        print("  none")

    print("\nPipeWire capture sources:")
    pw_inputs = pipewire_inputs()
    if pw_inputs:
        for name, description, warning in pw_inputs:
            print(f"  {name}\n    {description}")
            if warning:
                print(f"    warning: {warning}")
    else:
        print("  none")
    print(
        "\nThe aliases 'default' and 'pipewire' depend on the desktop default "
        "and may select silence. Prefer an exact entry above."
    )
    return result.returncode


def probe_microphone(device: str, *, seconds: int, threshold_dbfs: float) -> None:
    sample_count = SAMPLE_RATE * seconds
    command = capture_command(device, sample_count=sample_count)
    print(f"microphone preflight: device={device}; speak for {seconds}s ...", flush=True)
    try:
        result = subprocess.run(
            command,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=seconds + 5,
        )
    except subprocess.TimeoutExpired as error:
        raise SystemExit(f"microphone {device!r} did not finish its probe") from error
    expected_bytes = sample_count * SAMPLE_WIDTH_BYTES
    if len(result.stdout) < expected_bytes // 2:
        error_text = result.stderr.decode("utf-8", "replace").strip()
        detail = f": {error_text}" if error_text else ""
        if result.returncode != 0:
            raise SystemExit(f"cannot capture from microphone {device!r}{detail}")
        raise SystemExit(
            f"microphone {device!r} returned only {len(result.stdout)} of "
            f"{expected_bytes} expected PCM bytes"
        )
    rms_dbfs, peak_dbfs = signal_levels_dbfs(result.stdout)
    print(f"microphone level: rms={rms_dbfs:.1f} dBFS peak={peak_dbfs:.1f} dBFS")
    if peak_dbfs < threshold_dbfs:
        raise SystemExit(
            f"no usable signal from microphone {device!r}: peak {peak_dbfs:.1f} dBFS "
            f"is below {threshold_dbfs:.1f} dBFS. Run --list-inputs and select an "
            "exact plughw: or pw: source."
        )
    if rms_dbfs > -10.0 and peak_dbfs > -0.5:
        print(
            "warning: input is continuously near clipping; reduce capture gain or "
            "check that a microphone is physically connected",
            file=sys.stderr,
            flush=True,
        )


def validate_args(parser: argparse.ArgumentParser, args: argparse.Namespace) -> None:
    if args.probe_seconds <= 0:
        parser.error("--probe-seconds must be positive")
    if not math.isfinite(args.silence_threshold_dbfs) or not (
        -120.0 <= args.silence_threshold_dbfs <= 0.0
    ):
        parser.error("--silence-threshold-dbfs must be finite and between -120 and 0")
    if args.list_inputs or args.probe_microphone:
        return
    if not args.model:
        parser.error("model is required unless --list-inputs or --probe-microphone is used")
    if args.chunk_ms <= 0:
        parser.error("--chunk-ms must be positive")
    if args.duration is not None and args.duration <= 0:
        parser.error("--duration must be positive")
    if not Path(args.model).expanduser().is_file():
        parser.error(f"model not found: {args.model}")


def run(args: argparse.Namespace) -> int:
    if not args.check:
        probe_microphone(
            args.device,
            seconds=args.probe_seconds,
            threshold_dbfs=args.silence_threshold_dbfs,
        )
    root = transcribe_root(args)
    library = find_library(root, args.library)
    transcribe_cpp = load_transcribe(root, library)
    model_path = Path(args.model).expanduser().resolve()

    print(f"loading {model_path.name} with backend={args.backend} ...", flush=True)
    with transcribe_cpp.Model(model_path, backend=args.backend) as model:
        if not model.capabilities.supports_streaming:
            raise SystemExit(f"{model.arch}/{model.variant} does not support streaming")
        print(f"ready: {model.arch}/{model.variant} on {model.backend}", flush=True)
        if args.check:
            return 0

        command = capture_command(args.device)
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

        capture = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
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
                        if not final.committed.strip():
                            print(
                                "warning: microphone had a usable signal, but the model "
                                "returned no speech; check gain, distance, and language",
                                file=sys.stderr,
                            )
        finally:
            stop_capture(capture)

        if captured == 0:
            error_text = ""
            if capture.stderr is not None:
                error_text = capture.stderr.read().decode("utf-8", "replace").strip()
            detail = f": {error_text}" if error_text else ""
            raise SystemExit(f"microphone stopped before returning audio{detail}")

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
    if args.probe_microphone:
        probe_microphone(
            args.device,
            seconds=args.probe_seconds,
            threshold_dbfs=args.silence_threshold_dbfs,
        )
        return 0
    return run(args)


if __name__ == "__main__":
    raise SystemExit(main())
