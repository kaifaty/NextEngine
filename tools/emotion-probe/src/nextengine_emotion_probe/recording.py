from __future__ import annotations

import shutil
import subprocess
import wave
from pathlib import Path

from .probe import ProbeError

SAMPLE_RATE_HZ = 16_000
MAX_RECORD_SECONDS = 30.0


def validate_recorded_wav(output_path: Path, expected_frames: int) -> None:
    try:
        with wave.open(str(output_path), "rb") as wav:
            sample_rate = wav.getframerate()
            channels = wav.getnchannels()
            sample_width = wav.getsampwidth()
            frames = wav.getnframes()
    except (FileNotFoundError, EOFError, wave.Error) as error:
        raise ProbeError(f"pw-record did not produce a readable WAV: {error}") from error

    expected = (SAMPLE_RATE_HZ, 1, 2, expected_frames)
    actual = (sample_rate, channels, sample_width, frames)
    if actual != expected:
        raise ProbeError(
            "pw-record produced unexpected WAV metadata "
            f"(rate, channels, width, frames): expected {expected}, got {actual}"
        )


def recording_command(
    output_path: Path,
    seconds: float,
    target: str | None = None,
) -> list[str]:
    if not 0.5 <= seconds <= MAX_RECORD_SECONDS:
        raise ProbeError(
            f"recording duration must be between 0.5 and {MAX_RECORD_SECONDS:g} seconds"
        )
    sample_count = round(seconds * SAMPLE_RATE_HZ)
    command = [
        "pw-record",
        "--rate",
        str(SAMPLE_RATE_HZ),
        "--channels",
        "1",
        "--channel-map",
        "MONO",
        "--format",
        "s16",
        "--container",
        "wav",
        "--sample-count",
        str(sample_count),
    ]
    if target:
        command.extend(["--target", target])
    command.append(str(output_path))
    return command


def record_microphone(
    output_path: Path,
    seconds: float,
    target: str | None = None,
) -> None:
    if shutil.which("pw-record") is None:
        raise ProbeError("pw-record is not installed or not available on PATH")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    command = recording_command(output_path, seconds, target)
    try:
        result = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
            timeout=seconds + 10,
        )
    except subprocess.TimeoutExpired as error:
        raise ProbeError("microphone recording timed out") from error
    expected_frames = round(seconds * SAMPLE_RATE_HZ)
    try:
        validate_recorded_wav(output_path, expected_frames)
    except ProbeError as validation_error:
        diagnostic = result.stderr.strip() or result.stdout.strip() or "unknown error"
        raise ProbeError(
            f"pw-record failed with exit {result.returncode}: {diagnostic}; "
            f"{validation_error}"
        ) from validation_error
