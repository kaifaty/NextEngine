"""Import acquired public sources into speech-reliability source indexes.

This operator tooling implements the acquisition step of
``DEV-SPEECH-RELIABILITY-001`` R0/R1: it takes externally downloaded raw
material (FLEURS ru_ru float32 WAV archives, MUSAN noise, RIRS_NOISES),
converts or references it into the strict store contract used by
``reliability-corpus prepare`` — 16 kHz mono pcm_s16le WAV, pinned SHA-256,
bounded sample counts — and publishes JSONL source indexes next to the data
in one explicit external store.

Fail-closed rules:

- The store must be an existing directory outside this repository.
- Every emitted row re-verifies the produced/referenced WAV through the
  standard-library reader before hashing; anything unreadable, misrated,
  multi-channel or over the sample cap is counted and skipped with a typed
  reason, never silently included.
- Conversion uses system ffmpeg only when a file does not already match the
  contract; conforming files are referenced in place so raw provenance stays
  untouched.
- Indexes are written atomically with private permissions; no dataset bytes
  ever enter the repository.
"""

from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
import hashlib
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import tempfile
import wave

from .profile import REPOSITORY_ROOT
from .reliability import (
    MAX_AUDIO_SAMPLES,
    MAX_AUGMENTATION_ASSET_SAMPLES,
    _atomic_write,
    _canonical_json,
    _external_directory,
    _sha256_file,
)


IMPORT_ID_RE = re.compile(r"^[a-z0-9][a-z0-9._-]{0,127}$")
RAW_WAV_NAME_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*\.wav$")
CONTRACT_SAMPLE_RATE_HZ = 16_000
MAX_INDEX_ROWS = 100_000
SKIP_SAMPLE_LIMIT = 10


class ReliabilityImportError(RuntimeError):
    """A stable fail-closed source-import error."""


@dataclass
class _SkipLog:
    too_long: list[str] = field(default_factory=list)
    invalid_wav: list[str] = field(default_factory=list)
    bad_name: list[str] = field(default_factory=list)
    missing_file: list[str] = field(default_factory=list)

    def add(self, reason: str, label: str) -> None:
        bucket = getattr(self, reason)
        if len(bucket) < SKIP_SAMPLE_LIMIT:
            bucket.append(label)
        setattr(self, reason, bucket)

    def payload(self) -> dict[str, object]:
        return {
            "too_long": self.too_long,
            "invalid_wav": self.invalid_wav,
            "bad_name": self.bad_name,
            "missing_file": self.missing_file,
        }


@dataclass(frozen=True)
class _WavProbe:
    samples: int
    rate: int
    channels: int
    width: int


def _probe_wav(path: Path) -> _WavProbe:
    try:
        with wave.open(str(path), "rb") as handle:
            probe = _WavProbe(
                samples=handle.getnframes(),
                rate=handle.getframerate(),
                channels=handle.getnchannels(),
                width=handle.getsampwidth(),
            )
            if probe.samples <= 0:
                raise ReliabilityImportError(f"empty wav: {path}")
            return probe
    except (OSError, EOFError, wave.Error) as error:
        raise ReliabilityImportError(f"cannot read wav {path}: {error}") from error


def _require_ffmpeg() -> str:
    binary = shutil.which("ffmpeg")
    if binary is None:
        raise ReliabilityImportError(
            "ffmpeg is required to convert non-conforming sources"
        )
    return binary


def convert_to_contract(source: Path, destination: Path) -> None:
    """Convert one external WAV into 16 kHz mono pcm_s16le via ffmpeg."""

    binary = _require_ffmpeg()
    destination.parent.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        [
            binary,
            "-y",
            "-loglevel",
            "error",
            "-i",
            str(source),
            "-vn",
            "-ac",
            "1",
            "-ar",
            str(CONTRACT_SAMPLE_RATE_HZ),
            "-c:a",
            "pcm_s16le",
            str(destination),
        ],
        capture_output=True,
        timeout=300,
    )
    if result.returncode != 0 or not destination.is_file():
        raise ReliabilityImportError(
            f"ffmpeg conversion failed for {source}: "
            f"{result.stderr.decode('utf-8', 'replace')[:256]}"
        )


def _conforms(probe: _WavProbe, maximum_samples: int) -> bool:
    return (
        probe.rate == CONTRACT_SAMPLE_RATE_HZ
        and probe.channels == 1
        and probe.width == 2
        and probe.samples <= maximum_samples
    )


def _validate_raw_name(name: str, label: str) -> str:
    if not RAW_WAV_NAME_RE.fullmatch(name):
        raise ReliabilityImportError(f"{label} has unsupported file name: {name!r}")
    return name


def _stable_id(text: str, label: str) -> str:
    lowered = text.lower()
    if not IMPORT_ID_RE.fullmatch(lowered):
        raise ReliabilityImportError(f"{label} produces invalid stable id: {text!r}")
    return lowered


def _resolve_store_relative(store_root: Path, relative: str, label: str) -> str:
    candidate = (store_root / relative).resolve()
    if not candidate.is_relative_to(store_root):
        raise ReliabilityImportError(f"{label} escapes the store: {relative}")
    return relative


def _write_index(path: Path, rows: list[dict[str, object]]) -> str:
    resolved = path.expanduser().resolve()
    if resolved == REPOSITORY_ROOT or resolved.is_relative_to(REPOSITORY_ROOT):
        raise ReliabilityImportError("source index must stay outside the repository")
    encoded = b"".join(_canonical_json(row) + b"\n" for row in sorted(
        rows, key=lambda item: str(item.get("clip_id") or item.get("asset_id"))
    ))
    _atomic_write(resolved, encoded)
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def _convert_worker(jobs: list[tuple[Path, Path]], workers: int) -> None:
    if not jobs:
        return
    with ThreadPoolExecutor(max_workers=max(1, workers)) as pool:
        failures = list(pool.map(_convert_one, jobs))
    failure = next((item for item in failures if item is not None), None)
    if failure is not None:
        raise failure


def _convert_one(job: tuple[Path, Path]) -> Exception | None:
    source, destination = job
    try:
        convert_to_contract(source, destination)
    except Exception as error:  # noqa: BLE001 - collected and re-raised once
        return error if isinstance(error, ReliabilityImportError) else (
            ReliabilityImportError(f"conversion failed for {source}: {error}")
        )
    return None


def import_fleurs_ru(
    store: Path,
    *,
    fleurs_root: Path,
    out_index: Path,
    splits: tuple[str, ...] = ("train", "dev", "test"),
    max_samples: int = MAX_AUDIO_SAMPLES,
    workers: int = 8,
) -> dict[str, object]:
    """Build the FLEURS ru_ru speech source index inside ``store``."""

    store_root = _external_directory(store, "corpus store")
    root = fleurs_root.expanduser().resolve()
    if not root.is_dir():
        raise ReliabilityImportError(f"FLEURS root does not exist: {root}")
    rows: list[dict[str, object]] = []
    skips = _SkipLog()
    conversions: list[tuple[Path, Path]] = []
    plan: list[tuple[str, str, str]] = []
    for split in splits:
        tsv_path = root / f"{split}.tsv"
        if not tsv_path.is_file():
            raise ReliabilityImportError(f"missing FLEURS transcript file: {tsv_path}")
        audio_dir = root / split
        if not audio_dir.is_dir():
            raise ReliabilityImportError(f"missing FLEURS audio directory: {audio_dir}")
        seen_ids: set[str] = set()
        for line_number, line in enumerate(
            tsv_path.read_text(encoding="utf-8").splitlines(), start=1
        ):
            if not line.strip():
                continue
            columns = line.split("\t")
            if len(columns) < 4:
                skips.add("bad_name", f"{split}.tsv:{line_number}:columns")
                continue
            filename, transcript = columns[1], columns[2]
            label = f"fleurs/{split}:{filename}"
            try:
                _validate_raw_name(filename, label)
                clip_id = _stable_id(f"fleurs-{split}-{Path(filename).stem}", label)
            except ReliabilityImportError:
                skips.add("bad_name", label)
                continue
            if clip_id in seen_ids:
                raise ReliabilityImportError(f"duplicate clip id: {clip_id}")
            seen_ids.add(clip_id)
            source_path = audio_dir / filename
            if not source_path.is_file():
                skips.add("missing_file", label)
                continue
            destination = store_root / "audio" / "fleurs" / split / filename
            plan.append((label, transcript.strip(), clip_id))
            conversions.append((source_path, destination))
    _convert_worker(conversions, workers)
    for (label, transcript, clip_id), (_, destination) in zip(plan, conversions):
        relative = _resolve_store_relative(
            store_root,
            destination.relative_to(store_root).as_posix(),
            label,
        )
        try:
            probe = _probe_wav(destination)
        except ReliabilityImportError:
            skips.add("invalid_wav", label)
            continue
        if probe.samples > max_samples:
            skips.add("too_long", label)
            continue
        if not transcript:
            skips.add("bad_name", f"{label}:empty-transcript")
            continue
        rows.append(
            {
                "schema_version": 0,
                "clip_id": clip_id,
                "speaker_id": "fleurs-unlabeled-speakers",
                "relative_audio_path": relative,
                "audio_sha256": f"sha256:{_sha256_file(destination)}",
                "samples": probe.samples,
                "sample_rate_hz": 16_000,
                "channels": 1,
                "encoding": "pcm_s16le_wav",
                "transcript": transcript[:4_096],
            }
        )
    if len(rows) > MAX_INDEX_ROWS:
        raise ReliabilityImportError("FLEURS index exceeds the bounded row count")
    destination = out_index.expanduser().resolve()
    destination.parent.mkdir(parents=True, exist_ok=True)
    digest = _write_index(destination, rows)
    return {
        "schema_version": 0,
        "kind": "nextengine.speech-reliability.import-report",
        "source_id": "fleurs-ru",
        "role": "speech",
        "rows_written": len(rows),
        "converted_files": len(conversions),
        "skips": skips.payload(),
        "out_index_sha256": digest,
    }


def import_musan_noise(
    store: Path,
    *,
    noise_root: Path,
    out_index: Path,
    max_samples: int = MAX_AUGMENTATION_ASSET_SAMPLES,
    workers: int = 8,
) -> dict[str, object]:
    """Build the MUSAN noise asset index, converting only what differs."""

    store_root = _external_directory(store, "corpus store")
    root = noise_root.expanduser().resolve()
    if not root.is_dir():
        raise ReliabilityImportError(f"MUSAN noise root does not exist: {root}")
    if not root.is_relative_to(store_root):
        raise ReliabilityImportError(
            "the raw MUSAN tree must live under the store root so conforming "
            "files can be referenced in place"
        )
    rows: list[dict[str, object]] = []
    skips = _SkipLog()
    conversions: list[tuple[Path, Path]] = []
    plan: list[tuple[str, str, str, Path]] = []
    used_names: set[str] = set()
    for source_path in sorted(root.rglob("*.wav")):
        group = _stable_id(
            source_path.parent.relative_to(root).as_posix().replace("/", "_"),
            str(source_path),
        )
        flat_stem = _stable_id(
            source_path.relative_to(root).as_posix().replace("/", "_")[:-4],
            str(source_path),
        )
        label = f"musan-noise/{flat_stem}"
        if flat_stem in used_names:
            raise ReliabilityImportError(f"duplicate flattened asset id: {flat_stem}")
        used_names.add(flat_stem)
        try:
            probe = _probe_wav(source_path)
        except ReliabilityImportError:
            skips.add("invalid_wav", label)
            continue
        if probe.samples > max_samples:
            skips.add("too_long", label)
            continue
        if _conforms(probe, max_samples):
            final_path = source_path
        else:
            final_path = store_root / "audio" / "musan-noise" / f"{flat_stem}.wav"
            conversions.append((source_path, final_path))
        plan.append((label, group, flat_stem, final_path))
    _convert_worker(conversions, workers)
    for label, group, flat_stem, final_path in plan:
        relative = _resolve_store_relative(
            store_root,
            final_path.resolve().relative_to(store_root).as_posix(),
            label,
        )
        try:
            probe = _probe_wav(final_path)
        except ReliabilityImportError:
            skips.add("invalid_wav", label)
            continue
        if probe.samples > max_samples:
            skips.add("too_long", label)
            continue
        rows.append(
            {
                "schema_version": 0,
                "asset_id": f"musan-noise-{flat_stem}",
                "partition_group_id": group,
                "relative_audio_path": relative,
                "audio_sha256": f"sha256:{_sha256_file(final_path)}",
                "samples": probe.samples,
                "sample_rate_hz": 16_000,
                "channels": 1,
                "encoding": "pcm_s16le_wav",
            }
        )
    if len(rows) > MAX_INDEX_ROWS:
        raise ReliabilityImportError("MUSAN index exceeds the bounded row count")
    destination = out_index.expanduser().resolve()
    destination.parent.mkdir(parents=True, exist_ok=True)
    digest = _write_index(destination, rows)
    return {
        "schema_version": 0,
        "kind": "nextengine.speech-reliability.import-report",
        "source_id": "musan-openslr-17-noise",
        "role": "noise",
        "rows_written": len(rows),
        "converted_files": len(conversions),
        "skips": skips.payload(),
        "out_index_sha256": digest,
    }


def import_rirs(
    store: Path,
    *,
    rirs_root: Path,
    out_index: Path,
    subsets: tuple[str, ...] = ("simulated_rirs", "real_rirs_isotropic_noises"),
    max_samples: int = MAX_AUGMENTATION_ASSET_SAMPLES,
) -> dict[str, object]:
    """Reference already-contract RIRS_NOISES assets by their room groups."""

    store_root = _external_directory(store, "corpus store")
    root = rirs_root.expanduser().resolve()
    if not root.is_dir():
        raise ReliabilityImportError(f"RIRS root does not exist: {root}")
    if not root.is_relative_to(store_root):
        raise ReliabilityImportError(
            "the raw RIRS tree must live under the store root so assets can "
            "be referenced in place"
        )
    rows: list[dict[str, object]] = []
    skips = _SkipLog()
    for subset in subsets:
        subset_dir = root / subset
        if not subset_dir.is_dir():
            raise ReliabilityImportError(f"missing RIRS subset: {subset_dir}")
        for source_path in sorted(subset_dir.rglob("*.wav")):
            name = _validate_raw_name(source_path.name, f"rirs/{subset}")
            flat_stem = _stable_id(f"{subset}-{Path(name).stem}", name)
            label = f"rirs/{flat_stem}"
            try:
                probe = _probe_wav(source_path)
            except ReliabilityImportError:
                skips.add("invalid_wav", label)
                continue
            if probe.samples > max_samples:
                skips.add("too_long", label)
                continue
            if not _conforms(probe, max_samples):
                # RIRS ships 16 kHz mono pcm_s16le; anything else is upstream
                # drift and fails closed instead of being converted silently.
                skips.add("invalid_wav", f"{label}:contract")
                continue
            relative = _resolve_store_relative(
                store_root,
                source_path.resolve().relative_to(store_root).as_posix(),
                label,
            )
            room_group = Path(name).stem.split("-")[0]
            rows.append(
                {
                    "schema_version": 0,
                    "asset_id": flat_stem,
                    "partition_group_id": _stable_id(room_group, label),
                    "relative_audio_path": relative,
                    "audio_sha256": f"sha256:{_sha256_file(source_path)}",
                    "samples": probe.samples,
                    "sample_rate_hz": 16_000,
                    "channels": 1,
                    "encoding": "pcm_s16le_wav",
                }
            )
    if len(rows) > MAX_INDEX_ROWS:
        raise ReliabilityImportError("RIRS index exceeds the bounded row count")
    destination = out_index.expanduser().resolve()
    destination.parent.mkdir(parents=True, exist_ok=True)
    digest = _write_index(destination, rows)
    return {
        "schema_version": 0,
        "kind": "nextengine.speech-reliability.import-report",
        "source_id": "rirs-noises-openslr-28",
        "role": "rir",
        "rows_written": len(rows),
        "converted_files": 0,
        "skips": skips.payload(),
        "out_index_sha256": digest,
    }
