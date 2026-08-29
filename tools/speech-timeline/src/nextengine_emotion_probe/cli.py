"""Command line entry point retained for the emotion probe."""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path
from typing import Sequence

from .probe import (
    DEFAULT_MODEL_ID,
    DEFAULT_MODEL_REVISION,
    EmotionProbe,
    ProbeError,
)
from .recording import record_microphone


def default_cache_dir() -> Path:
    return Path.home() / ".cache" / "nextengine" / "emotion2vec-plus-base" / "models"


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(prog="next-emotion-probe")
    root.add_argument("--model-id", default=DEFAULT_MODEL_ID)
    root.add_argument("--model-revision", default=DEFAULT_MODEL_REVISION)
    root.add_argument("--cache-dir", type=Path, default=default_cache_dir())
    root.add_argument("--device", choices=["auto", "cpu", "cuda"], default="auto")
    commands = root.add_subparsers(dest="command", required=True)

    commands.add_parser("download", help="download and initialize the model")

    analyze = commands.add_parser("analyze", help="analyze an existing audio file")
    analyze.add_argument("audio", type=Path)

    record = commands.add_parser("record", help="record the microphone and analyze it")
    record.add_argument("--seconds", type=float, default=5.0)
    record.add_argument("--target", help="optional PipeWire source node serial or name")
    record.add_argument("--keep-audio", type=Path)
    return root


def _print_json(payload: dict[str, object], stream: object = sys.stdout) -> None:
    print(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True), file=stream)


def run(arguments: argparse.Namespace) -> int:
    probe = EmotionProbe(
        arguments.model_id,
        arguments.model_revision,
        arguments.cache_dir,
        arguments.device,
    )
    if arguments.command == "download":
        probe.load()
        _print_json(
            {
                "schema_version": 1,
                "status": "ready",
                "model": {
                    "id": probe.model_id,
                    "revision": probe.model_revision,
                    "hub": "huggingface",
                    "device": probe.device,
                },
                "cache_dir": str(probe.cache_dir),
            }
        )
        return 0
    if arguments.command == "analyze":
        _print_json(probe.analyze(arguments.audio))
        return 0
    if arguments.command == "record":
        if arguments.keep_audio:
            audio_path = arguments.keep_audio.expanduser().resolve()
            record_microphone(audio_path, arguments.seconds, arguments.target)
            _print_json(probe.analyze(audio_path))
            return 0
        with tempfile.TemporaryDirectory(prefix="nextengine-emotion-probe-") as temp_dir:
            audio_path = Path(temp_dir) / "microphone.wav"
            record_microphone(audio_path, arguments.seconds, arguments.target)
            _print_json(probe.analyze(audio_path))
            return 0
    raise ProbeError(f"unsupported command: {arguments.command}")


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parser().parse_args(argv)
    try:
        return run(arguments)
    except ProbeError as error:
        _print_json(
            {
                "schema_version": 1,
                "status": "error",
                "error": str(error),
            },
            stream=sys.stderr,
        )
        return 2
