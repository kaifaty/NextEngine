#!/usr/bin/env python3
"""Build the V41 C0 disclosed physical-sound ML corpus outside the repository."""

from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import math
import os
import shutil
import struct
import subprocess
import tempfile
import zipfile
from collections import Counter, defaultdict
from itertools import pairwise
from pathlib import Path
from typing import Any

import numpy as np
from scipy import signal

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-profile.v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-corpus.v1"
FEATURE_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-acoustic-target.v1"
PROJECTION_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-role-projection.v1"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-access-ledger.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-report.v1"
CARD_SCHEMA = "nextengine.experimental-physical-sound-v41-c0-corpus-card.v1"
DECISION = "C0_DISCLOSED_CORPUS_REPEATABLE_B0_V0_AUTHORIZED"

ROLES = ("generator_development", "generator_train", "validator_calibration")
AXES = (
    "absolute_force",
    "acoustic_pseudo_target",
    "geometry",
    "impact_position",
    "listener_position",
    "material_composition",
    "material_identity",
    "object_identity",
    "support_condition",
    "transfer_response",
    "waveform",
)
MAX_JSON_BYTES = 4 * 1024 * 1024
HASH_LENGTH = 64
ARCHIVE_OK = 0
ARCHIVE_EOF = 1

CORPUS_POLICY = {
    "channel_count": 1,
    "dc_reference_samples": 480,
    "normalization_peak": 0.95,
    "onset_energy_window_samples": 48,
    "output_container": "wav_pcm_s16le",
    "pretrigger_samples": 480,
    "resample_policy": "scipy_resample_poly_kaiser_beta_5_constant_pad",
    "sample_rate_hz": 48000,
    "segment_samples": 144000,
    "silence_peak_floor": 1e-07,
    "source_payload_policy": "hash_verified_external_research_only",
}

FEATURE_POLICY = {
    "band_count": 24,
    "band_max_hz": 24000.0,
    "band_min_hz": 60.0,
    "damping_floor_db": -30.0,
    "damping_max_seconds": 1.5,
    "maximum_modes": 12,
    "modal_frequency_max_hz": 18000.0,
    "modal_frequency_min_hz": 80.0,
    "modal_minimum_distance_hz": 40.0,
    "modal_prominence_db": 6.0,
    "spectrogram_hop_samples": 512,
    "spectrogram_window_samples": 2048,
    "transient_envelope_bins": 48,
    "welch_hop_samples": 2048,
    "welch_window_samples": 8192,
}

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "authorizes_only": ["v41_b0_baselines", "v41_v0_lab_validator"],
    "corpus_build_authority": True,
    "model_training_authority": False,
    "product_authority": False,
    "protected_access_authority": False,
    "public_contract": False,
    "runtime_consumer_allowed": False,
    "validator_release_authority": False,
}


class CorpusError(RuntimeError):
    """C0 cannot publish a trustworthy disclosed corpus."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--d1-roster", required=True, type=Path)
    parser.add_argument("--identified-manifest", required=True, type=Path)
    parser.add_argument("--source-manifest", required=True, type=Path)
    parser.add_argument("--identified-report", required=True, type=Path)
    parser.add_argument("--cache", required=True, type=Path)
    parser.add_argument("--realimpact-root", action="append", default=[], type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def hash_file(path: Path) -> tuple[int, str]:
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
            size += len(chunk)
    return size, digest.hexdigest()


def require_hash(value: Any, context: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != HASH_LENGTH
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise CorpusError(f"{context} must be a lowercase SHA-256")
    return value


def require_dict(value: Any, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise CorpusError(f"{context} must be an object")
    return value


def require_list(value: Any, context: str) -> list[Any]:
    if not isinstance(value, list):
        raise CorpusError(f"{context} must be an array")
    return value


def read_regular(path: Path, context: str, maximum: int | None = None) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise CorpusError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or (maximum is not None and size > maximum):
        raise CorpusError(f"{context} has an invalid byte count")
    return path.read_bytes()


def parse_json_bytes(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise CorpusError(f"{context} is not valid UTF-8 JSON") from error
    return require_dict(value, context)


def read_canonical_profile(path: Path) -> tuple[bytes, dict[str, Any]]:
    data = read_regular(path, "C0 profile", MAX_JSON_BYTES)
    value = parse_json_bytes(data, "C0 profile")
    if canonical_json(value) != data:
        raise CorpusError("C0 profile must use canonical JSON")
    return data, value


def check_binding(path: Path, binding: dict[str, Any], context: str) -> bytes:
    require_hash(binding.get("sha256"), f"{context} SHA-256")
    expected_bytes = binding.get("bytes")
    if type(expected_bytes) is not int or expected_bytes <= 0:
        raise CorpusError(f"{context} binding has an invalid byte count")
    data = read_regular(path, context)
    if len(data) != expected_bytes or sha256_bytes(data) != binding["sha256"]:
        raise CorpusError(f"{context} bytes or SHA-256 changed")
    return data


def validate_environment(profile: dict[str, Any]) -> dict[str, Any]:
    environment = require_dict(profile.get("environment"), "environment")
    if environment.get("numpy_version") != np.__version__:
        raise CorpusError("NumPy version drift")
    import scipy

    if environment.get("scipy_version") != scipy.__version__:
        raise CorpusError("SciPy version drift")
    for key in ("ffmpeg", "libarchive"):
        binding = require_dict(environment.get(key), f"environment.{key}")
        path = Path(binding.get("path", ""))
        size, digest = hash_file(path)
        if size != binding.get("bytes") or digest != binding.get("sha256"):
            raise CorpusError(f"{key} executable/library drift")
    return environment


def validate_dependencies(profile: dict[str, Any]) -> None:
    bindings = require_list(profile.get("dependency_bindings"), "dependencies")
    seen: set[str] = set()
    for index, raw in enumerate(bindings):
        binding = require_dict(raw, f"dependencies[{index}]")
        path_text = binding.get("path")
        if not isinstance(path_text, str) or not path_text:
            raise CorpusError("dependency path must be non-empty")
        if path_text in seen:
            raise CorpusError("duplicate dependency path")
        seen.add(path_text)
        check_binding(repository_root() / path_text, binding, f"dependency {path_text}")
    if [item["path"] for item in bindings] != sorted(seen):
        raise CorpusError("dependency bindings must be path-sorted")


def validate_profile(value: dict[str, Any]) -> dict[str, Any]:
    required = {
        "authority",
        "corpus_policy",
        "d1_binding",
        "dependency_bindings",
        "environment",
        "expected_counts",
        "family_mapping",
        "feature_policy",
        "input_bindings",
        "parent_aliases",
        "profile_id",
        "realimpact_inputs",
        "schema",
    }
    if set(value) != required:
        raise CorpusError("C0 profile fields changed")
    if value["schema"] != PROFILE_SCHEMA or not isinstance(value["profile_id"], str):
        raise CorpusError("unknown C0 profile identity")
    if value["authority"] != AUTHORITY:
        raise CorpusError("C0 authority changed")
    if value["corpus_policy"] != CORPUS_POLICY:
        raise CorpusError("C0 corpus policy changed")
    if value["feature_policy"] != FEATURE_POLICY:
        raise CorpusError("C0 feature policy changed")
    counts = require_dict(value["expected_counts"], "expected counts")
    for key in (
        "identified_recordings",
        "physical_parents",
        "realimpact_transfers",
        "total_records",
    ):
        if type(counts.get(key)) is not int or counts[key] < 0:
            raise CorpusError(f"invalid expected count: {key}")
    role_records = require_dict(counts.get("role_records"), "role record counts")
    if set(role_records) != set(ROLES) or any(
        type(role_records[role]) is not int or role_records[role] < 0 for role in ROLES
    ):
        raise CorpusError("invalid role record counts")
    mapping = require_list(value["family_mapping"], "family mapping")
    mapping_keys: list[tuple[str, str]] = []
    for item in mapping:
        row = require_dict(item, "family mapping row")
        if set(row) != {"family_id", "project_id", "publisher_id"}:
            raise CorpusError("family mapping row fields changed")
        mapping_keys.append((row["publisher_id"], row["project_id"]))
    if mapping_keys != sorted(set(mapping_keys)):
        raise CorpusError("family mapping keys must be sorted and unique")
    aliases = require_list(value["parent_aliases"], "parent aliases")
    for row in aliases:
        alias = require_dict(row, "parent alias")
        if set(alias) != {"family_id", "match", "physical_parent_id"}:
            raise CorpusError("parent alias fields changed")
        require_dict(alias["match"], "parent alias match")
    inputs = require_dict(value["input_bindings"], "input bindings")
    if set(inputs) != {"identified_manifest", "identified_report", "source_manifest"}:
        raise CorpusError("input bindings changed")
    for name, binding in inputs.items():
        binding = require_dict(binding, f"input binding {name}")
        require_hash(binding.get("sha256"), f"input binding {name}")
        if type(binding.get("bytes")) is not int or binding["bytes"] <= 0:
            raise CorpusError("input binding byte count changed")
    d1 = require_dict(value["d1_binding"], "D1 binding")
    require_hash(d1.get("sha256"), "D1 roster")
    require_hash(d1.get("roster_root_sha256"), "D1 roster root")
    if type(d1.get("bytes")) is not int or d1["bytes"] <= 0:
        raise CorpusError("D1 binding byte count changed")
    realimpact = require_list(value["realimpact_inputs"], "REALIMPACT inputs")
    names = []
    for raw in realimpact:
        item = require_dict(raw, "REALIMPACT input")
        if set(item) != {"manifest", "root_name"}:
            raise CorpusError("REALIMPACT input fields changed")
        names.append(item["root_name"])
        require_hash(item["manifest"].get("sha256"), "REALIMPACT manifest")
    if names != sorted(set(names)):
        raise CorpusError("REALIMPACT inputs must be root-name sorted and unique")
    validate_dependencies(value)
    validate_environment(value)
    return value


def load_bound_json(
    path: Path, binding: dict[str, Any], context: str
) -> dict[str, Any]:
    return parse_json_bytes(check_binding(path, binding, context), context)


def load_d1_roster(
    path: Path, profile: dict[str, Any]
) -> tuple[bytes, dict[str, dict[str, Any]], dict[str, Any]]:
    data = check_binding(path, profile["d1_binding"], "D1 roster")
    roster = parse_json_bytes(data, "D1 roster")
    if canonical_json(roster) != data:
        raise CorpusError("D1 roster must remain canonical")
    binding = profile["d1_binding"]
    if roster.get("roster_root_sha256") != binding["roster_root_sha256"]:
        raise CorpusError("D1 roster root changed")
    if roster.get("authority", {}).get("authorizes_only") != "v41_c0_disclosed_corpus":
        raise CorpusError("D1 does not authorize C0")
    rows = require_list(roster.get("families"), "D1 families")
    families: dict[str, dict[str, Any]] = {}
    for raw in rows:
        row = require_dict(raw, "D1 family")
        family_id = row.get("family_id")
        if not isinstance(family_id, str) or family_id in families:
            raise CorpusError("D1 family identities changed")
        if row.get("role") not in ROLES or row.get("protected") is not False:
            raise CorpusError("D1 family role or protection changed")
        families[family_id] = row
    mapped = {row["family_id"] for row in profile["family_mapping"]}
    if mapped != set(families):
        raise CorpusError("family mapping does not cover the exact D1 roster")
    return data, families, roster


def family_lookup(profile: dict[str, Any]) -> dict[tuple[str, str], str]:
    return {
        (row["publisher_id"], row["project_id"]): row["family_id"]
        for row in profile["family_mapping"]
    }


def match_parent_alias(
    profile: dict[str, Any], family_id: str, fields: dict[str, Any], default: str
) -> str:
    matches = []
    for alias in profile["parent_aliases"]:
        if alias["family_id"] != family_id:
            continue
        if all(fields.get(key) == value for key, value in alias["match"].items()):
            matches.append(alias["physical_parent_id"])
    if len(matches) > 1:
        raise CorpusError("one record matches multiple physical-parent aliases")
    return matches[0] if matches else default


def cache_object(cache: Path, digest: str) -> Path:
    require_hash(digest, "cache object")
    return cache / "objects" / digest[:2] / digest


class ArchiveReader:
    def __init__(self, library_path: Path) -> None:
        self.library = ctypes.CDLL(str(library_path))
        self.library.archive_read_new.restype = ctypes.c_void_p
        for function in (
            "archive_read_support_filter_all",
            "archive_read_support_format_all",
            "archive_read_data_skip",
            "archive_read_free",
        ):
            getattr(self.library, function).argtypes = [ctypes.c_void_p]
        self.library.archive_read_open_filename.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_size_t,
        ]
        self.library.archive_read_next_header.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_void_p),
        ]
        self.library.archive_entry_pathname.argtypes = [ctypes.c_void_p]
        self.library.archive_entry_pathname.restype = ctypes.c_char_p
        self.library.archive_read_data.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.c_size_t,
        ]
        self.library.archive_read_data.restype = ctypes.c_ssize_t
        self.library.archive_error_string.argtypes = [ctypes.c_void_p]
        self.library.archive_error_string.restype = ctypes.c_char_p

    def _error(self, archive: int, context: str) -> CorpusError:
        raw = self.library.archive_error_string(archive)
        detail = raw.decode(errors="replace") if raw else "unknown libarchive error"
        return CorpusError(f"{context}: {detail}")

    def extract(self, path: Path, requested: set[str]) -> dict[str, bytes]:
        if not requested:
            return {}
        if zipfile.is_zipfile(path):
            with zipfile.ZipFile(path) as archive:
                names = archive.namelist()
                if len(names) != len(set(names)):
                    raise CorpusError("archive contains duplicate member names")
                missing = requested - set(names)
                if missing:
                    raise CorpusError(f"ZIP members are absent: {sorted(missing)}")
                return {name: archive.read(name) for name in sorted(requested)}

        archive = self.library.archive_read_new()
        if not archive:
            raise CorpusError("libarchive allocation failed")
        found: dict[str, bytes] = {}
        seen: set[str] = set()
        try:
            if self.library.archive_read_support_filter_all(archive) != ARCHIVE_OK:
                raise self._error(archive, "libarchive filter setup failed")
            if self.library.archive_read_support_format_all(archive) != ARCHIVE_OK:
                raise self._error(archive, "libarchive format setup failed")
            if (
                self.library.archive_read_open_filename(
                    archive, os.fsencode(path), 64 * 1024
                )
                != ARCHIVE_OK
            ):
                raise self._error(archive, "libarchive open failed")
            entry = ctypes.c_void_p()
            while True:
                status = self.library.archive_read_next_header(
                    archive, ctypes.byref(entry)
                )
                if status == ARCHIVE_EOF:
                    break
                if status != ARCHIVE_OK:
                    raise self._error(archive, "libarchive header read failed")
                raw_name = self.library.archive_entry_pathname(entry)
                if not raw_name:
                    raise CorpusError("archive member has no path")
                name = raw_name.decode("utf-8", errors="strict")
                if name in seen:
                    raise CorpusError("archive contains duplicate member names")
                seen.add(name)
                if name not in requested:
                    if self.library.archive_read_data_skip(archive) != ARCHIVE_OK:
                        raise self._error(archive, "archive member skip failed")
                    continue
                chunks = []
                buffer = ctypes.create_string_buffer(64 * 1024)
                while True:
                    count = self.library.archive_read_data(archive, buffer, len(buffer))
                    if count == 0:
                        break
                    if count < 0:
                        raise self._error(archive, "archive member read failed")
                    chunks.append(buffer.raw[:count])
                found[name] = b"".join(chunks)
        finally:
            self.library.archive_read_free(archive)
        missing = requested - set(found)
        if missing:
            raise CorpusError(f"archive members are absent: {sorted(missing)}")
        return found


def archive_member_for(
    source: dict[str, Any], recording_id: str
) -> tuple[dict[str, Any], str]:
    profile = require_dict(source.get("adapter_profile"), "adapter profile")
    recordings = require_list(profile.get("recordings"), "adapter recordings")
    matches = [
        row
        for row in recordings
        if row.get("recording_id", row.get("repeat_id")) == recording_id
    ]
    if len(matches) != 1 or not isinstance(matches[0].get("archive_entry"), str):
        raise CorpusError("archive recording identity is ambiguous or absent")
    archives = [
        row for row in source.get("artifacts", []) if row.get("role") == "audio_archive"
    ]
    if len(archives) != 1:
        raise CorpusError("source must bind exactly one audio archive")
    return archives[0], matches[0]["archive_entry"]


class PayloadResolver:
    def __init__(
        self,
        cache: Path,
        sources: dict[str, dict[str, Any]],
        entries: list[dict[str, Any]],
        archive_reader: ArchiveReader,
    ) -> None:
        if cache.is_symlink() or not cache.is_dir():
            raise CorpusError("source cache must be a regular non-symlink directory")
        self.cache = cache
        self.sources = sources
        self.extracted: dict[tuple[str, str], bytes] = {}
        self.payload_bytes_read = 0
        self.archive_bytes_read = 0
        requests: dict[str, set[str]] = defaultdict(set)
        archive_bindings: dict[str, dict[str, Any]] = {}
        for entry in entries:
            digest = entry["audio_file_sha256"]
            if cache_object(cache, digest).is_file():
                continue
            source = sources.get(entry["source_id"])
            if source is None:
                raise CorpusError("identified entry references an unknown source")
            archive, member = archive_member_for(source, entry["recording_id"])
            archive_digest = archive.get("expected_sha256")
            require_hash(archive_digest, "source archive")
            requests[archive_digest].add(member)
            current = archive_bindings.setdefault(archive_digest, archive)
            if current != archive:
                raise CorpusError("one archive hash has inconsistent bindings")
        for digest in sorted(requests):
            archive = archive_bindings[digest]
            path = cache_object(cache, digest)
            size, actual = hash_file(path)
            if size != archive.get("expected_byte_count") or actual != archive.get(
                "expected_sha256"
            ):
                raise CorpusError("source archive bytes or SHA-256 changed")
            extracted = archive_reader.extract(path, requests[digest])
            self.archive_bytes_read += size
            for member, data in extracted.items():
                self.extracted[(digest, member)] = data

    def resolve(self, entry: dict[str, Any]) -> bytes:
        digest = entry["audio_file_sha256"]
        expected = entry["audio_file_bytes"]
        direct = cache_object(self.cache, digest)
        if direct.is_file():
            data = read_regular(direct, "source payload")
        else:
            source = self.sources[entry["source_id"]]
            archive, member = archive_member_for(source, entry["recording_id"])
            data = self.extracted[(archive["expected_sha256"], member)]
        if len(data) != expected or sha256_bytes(data) != digest:
            raise CorpusError("identified payload bytes or SHA-256 changed")
        self.payload_bytes_read += len(data)
        return data


def decode_audio(
    payload: bytes, expected_sample_rate: int, ffmpeg_path: Path
) -> np.ndarray:
    with tempfile.TemporaryDirectory(prefix="nextengine-v41-c0-decode-") as temp:
        source = Path(temp) / "hash-verified-source.audio"
        source.write_bytes(payload)
        command = [
            str(ffmpeg_path),
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-protocol_whitelist",
            "file,pipe",
            "-fflags",
            "+bitexact",
            "-i",
            str(source),
            "-map",
            "0:a:0",
            "-vn",
            "-threads",
            "1",
            "-ac",
            "1",
            "-ar",
            str(expected_sample_rate),
            "-c:a",
            "pcm_f64le",
            "-flags:a",
            "+bitexact",
            "-f",
            "f64le",
            "pipe:1",
        ]
        completed = subprocess.run(
            command,
            check=False,
            capture_output=True,
            timeout=60,
        )
    if completed.returncode != 0 or completed.stderr:
        detail = completed.stderr.decode(errors="replace")[:400]
        raise CorpusError(f"ffmpeg decode failed: {detail}")
    if not completed.stdout or len(completed.stdout) % 8:
        raise CorpusError("ffmpeg returned an invalid f64le stream")
    decoded = np.frombuffer(completed.stdout, dtype="<f8").copy()
    if not np.all(np.isfinite(decoded)):
        raise CorpusError("decoded source contains non-finite samples")
    return decoded


def resample_mono(samples: np.ndarray, source_rate: int) -> np.ndarray:
    target_rate = CORPUS_POLICY["sample_rate_hz"]
    if source_rate == target_rate:
        return samples.astype(np.float64, copy=True)
    divisor = math.gcd(source_rate, target_rate)
    return signal.resample_poly(
        samples,
        target_rate // divisor,
        source_rate // divisor,
        window=("kaiser", 5.0),
        padtype="constant",
    ).astype(np.float64, copy=False)


def canonical_segment(samples: np.ndarray) -> tuple[np.ndarray, dict[str, Any]]:
    if samples.ndim != 1 or len(samples) == 0 or not np.all(np.isfinite(samples)):
        raise CorpusError("source waveform is empty or invalid")
    reference_count = min(len(samples), CORPUS_POLICY["dc_reference_samples"])
    centered = samples - float(np.mean(samples[:reference_count]))
    derivative = np.diff(centered, prepend=centered[0])
    energy = signal.convolve(
        derivative * derivative,
        np.ones(CORPUS_POLICY["onset_energy_window_samples"], dtype=np.float64),
        mode="same",
        method="direct",
    )
    onset = int(np.argmax(energy))
    pretrigger = CORPUS_POLICY["pretrigger_samples"]
    length = CORPUS_POLICY["segment_samples"]
    source_start = onset - pretrigger
    source_stop = source_start + length
    destination_start = max(0, -source_start)
    destination_stop = length - max(0, source_stop - len(centered))
    source_start = max(0, source_start)
    source_stop = min(len(centered), source_stop)
    segment = np.zeros(length, dtype=np.float64)
    if destination_stop > destination_start:
        segment[destination_start:destination_stop] = centered[source_start:source_stop]
    dc_count = min(pretrigger, length)
    segment -= float(np.mean(segment[:dc_count]))
    raw_peak = float(np.max(np.abs(segment)))
    if raw_peak < CORPUS_POLICY["silence_peak_floor"]:
        raise CorpusError("source segment is silent")
    gain = CORPUS_POLICY["normalization_peak"] / raw_peak
    normalized = np.clip(segment * gain, -1.0, 1.0)
    quantized = np.rint(normalized * 32767.0).astype("<i2")
    pcm = quantized.astype(np.float64) / 32767.0
    metadata = {
        "normalization_gain": rounded(gain),
        "raw_peak": rounded(raw_peak),
        "source_onset_sample_48k": onset,
        "source_sample_count_48k": len(samples),
    }
    return pcm, metadata


def wav_pcm16_bytes(samples: np.ndarray) -> bytes:
    if samples.dtype != np.float64 or samples.ndim != 1:
        raise CorpusError("canonical WAV input must be mono float64")
    quantized = np.rint(np.clip(samples, -1.0, 1.0) * 32767.0).astype("<i2")
    payload = quantized.tobytes()
    rate = CORPUS_POLICY["sample_rate_hz"]
    header = b"".join(
        [
            b"RIFF",
            struct.pack("<I", 36 + len(payload)),
            b"WAVEfmt ",
            struct.pack("<IHHIIHH", 16, 1, 1, rate, rate * 2, 2, 16),
            b"data",
            struct.pack("<I", len(payload)),
        ]
    )
    return header + payload


def rounded(value: float, digits: int = 9) -> float:
    if not math.isfinite(value):
        raise CorpusError("derived target is non-finite")
    result = round(float(value), digits)
    return 0.0 if result == 0 else result


def crossing_decay(
    envelope: np.ndarray, hop_seconds: float
) -> tuple[float | None, float]:
    if len(envelope) < 4:
        return None, 0.0
    peak_index = int(np.argmax(envelope))
    peak = float(envelope[peak_index])
    if peak <= 0:
        return None, 0.0
    db = 20.0 * np.log10(np.maximum(envelope / peak, 1e-12))
    after = db[peak_index:]
    below_5 = np.flatnonzero(after <= -5.0)
    if len(below_5) == 0:
        return None, 0.0
    start = peak_index + int(below_5[0])
    below_25 = np.flatnonzero(db[start:] <= -25.0)
    if len(below_25) == 0:
        return None, 0.0
    stop = start + int(below_25[0])
    duration = (stop - start) * hop_seconds
    if duration <= 0:
        return None, 0.0
    return rounded(duration * 3.0), rounded(min(1.0, (stop - start) / 20.0))


def extract_acoustic_target(samples: np.ndarray) -> dict[str, Any]:
    rate = CORPUS_POLICY["sample_rate_hz"]
    onset = CORPUS_POLICY["pretrigger_samples"]
    analysis = samples[onset:]
    if len(analysis) < FEATURE_POLICY["welch_window_samples"]:
        raise CorpusError("canonical segment is too short for acoustic targets")

    welch_n = FEATURE_POLICY["welch_window_samples"]
    welch_overlap = welch_n - FEATURE_POLICY["welch_hop_samples"]
    frequencies, power = signal.welch(
        analysis,
        fs=rate,
        window="hann",
        nperseg=welch_n,
        noverlap=welch_overlap,
        detrend=False,
        scaling="spectrum",
    )
    power = np.maximum(power, 1e-24)
    total_power = float(np.sum(power))
    centroid = float(np.sum(frequencies * power) / total_power)
    bandwidth = float(
        math.sqrt(np.sum(((frequencies - centroid) ** 2) * power) / total_power)
    )
    cumulative = np.cumsum(power)
    rolloff_index = min(
        int(np.searchsorted(cumulative, 0.95 * cumulative[-1])), len(power) - 1
    )
    flatness = float(np.exp(np.mean(np.log(power))) / np.mean(power))

    band_edges = np.geomspace(
        FEATURE_POLICY["band_min_hz"],
        FEATURE_POLICY["band_max_hz"],
        FEATURE_POLICY["band_count"] + 1,
    )
    band_energy = []
    for low, high in pairwise(band_edges):
        mask = (frequencies >= low) & (frequencies < high)
        value = float(np.sum(power[mask])) if np.any(mask) else 1e-24
        band_energy.append(
            rounded(10.0 * math.log10(max(value / total_power, 1e-24)), 6)
        )

    stft_n = FEATURE_POLICY["spectrogram_window_samples"]
    stft_overlap = stft_n - FEATURE_POLICY["spectrogram_hop_samples"]
    stft_f, stft_t, stft = signal.stft(
        analysis,
        fs=rate,
        window="hann",
        nperseg=stft_n,
        noverlap=stft_overlap,
        boundary=None,
        padded=False,
    )
    magnitude = np.abs(stft)
    rms_envelope = np.sqrt(np.mean(magnitude * magnitude, axis=0))
    decay_t20, decay_confidence = crossing_decay(
        rms_envelope, FEATURE_POLICY["spectrogram_hop_samples"] / rate
    )

    log_power = 10.0 * np.log10(power)
    bin_hz = rate / welch_n
    valid = (frequencies >= FEATURE_POLICY["modal_frequency_min_hz"]) & (
        frequencies <= FEATURE_POLICY["modal_frequency_max_hz"]
    )
    valid_indices = np.flatnonzero(valid)
    peak_local, properties = signal.find_peaks(
        log_power[valid],
        prominence=FEATURE_POLICY["modal_prominence_db"],
        distance=max(1, round(FEATURE_POLICY["modal_minimum_distance_hz"] / bin_hz)),
    )
    candidates = []
    for local, prominence in zip(peak_local, properties["prominences"], strict=True):
        index = int(valid_indices[local])
        candidates.append((float(prominence), index))
    candidates.sort(key=lambda item: (-item[0], -log_power[item[1]], item[1]))
    candidates = candidates[: FEATURE_POLICY["maximum_modes"]]
    modes = []
    maximum_time = FEATURE_POLICY["damping_max_seconds"]
    for prominence, index in sorted(candidates, key=lambda item: item[1]):
        frequency = float(frequencies[index])
        stft_index = int(np.argmin(np.abs(stft_f - frequency)))
        track = magnitude[stft_index]
        start = int(np.argmax(track))
        peak_amplitude = float(track[start])
        permitted = np.flatnonzero(
            (stft_t >= stft_t[start]) & (stft_t <= stft_t[start] + maximum_time)
        )
        permitted = permitted[track[permitted] >= peak_amplitude * 10 ** (-30.0 / 20.0)]
        t60: float | None = None
        q_factor: float | None = None
        confidence = 0.0
        if len(permitted) >= 6:
            x = stft_t[permitted] - stft_t[permitted[0]]
            y = np.log(np.maximum(track[permitted], 1e-12))
            slope, intercept = np.polyfit(x, y, 1)
            predicted = slope * x + intercept
            variance = float(np.sum((y - np.mean(y)) ** 2))
            r2 = 1.0 - float(np.sum((y - predicted) ** 2)) / max(variance, 1e-12)
            if slope < -1e-6:
                tau = -1.0 / float(slope)
                t60 = rounded(math.log(1000.0) * tau)
                q_factor = rounded(math.pi * frequency * tau)
                confidence = rounded(max(0.0, min(1.0, r2)))
        modes.append(
            {
                "decay_confidence": confidence,
                "frequency_hz": rounded(frequency, 6),
                "prominence_db": rounded(prominence, 6),
                "q_factor": q_factor,
                "relative_level_db": rounded(
                    log_power[index] - float(np.max(log_power)), 6
                ),
                "t60_seconds": t60,
            }
        )

    absolute = np.abs(analysis)
    energy = analysis * analysis
    energy_sum = float(np.sum(energy))
    temporal_centroid = float(
        np.sum(np.arange(len(analysis)) * energy) / energy_sum / rate
    )
    peak_offset = int(np.argmax(absolute))
    rms = float(math.sqrt(np.mean(energy)))
    zero_crossings = float(
        np.mean(np.signbit(analysis[1:]) != np.signbit(analysis[:-1]))
    )
    envelope_edges = np.concatenate(
        (
            np.array([0], dtype=np.int64),
            np.rint(
                np.geomspace(
                    rate * 0.001,
                    len(analysis),
                    FEATURE_POLICY["transient_envelope_bins"],
                )
            ).astype(np.int64),
        )
    )
    if len(envelope_edges) != FEATURE_POLICY["transient_envelope_bins"] + 1:
        raise CorpusError("transient envelope edges collapsed")
    envelope = []
    for start, stop in pairwise(envelope_edges):
        stop = max(stop, start + 1)
        value = math.sqrt(float(np.mean(analysis[start:stop] ** 2)))
        envelope.append(rounded(20.0 * math.log10(max(value, 1e-12)), 6))

    uncertainty = {
        "acoustic_targets_are_waveform_derived": True,
        "absolute_gain_comparable_across_sources": False,
        "decay_confidence": decay_confidence,
        "median_mode_decay_confidence": rounded(
            float(np.median([mode["decay_confidence"] for mode in modes]))
            if modes
            else 0.0
        ),
        "mode_count": len(modes),
    }
    return {
        "modes": modes,
        "sample_count": len(samples),
        "sample_rate_hz": rate,
        "schema": FEATURE_SCHEMA,
        "spectral": {
            "band_edges_hz": [rounded(value, 6) for value in band_edges],
            "band_energy_db": band_energy,
            "bandwidth_hz": rounded(bandwidth, 6),
            "centroid_hz": rounded(centroid, 6),
            "flatness": rounded(flatness),
            "rolloff_95_hz": rounded(float(frequencies[rolloff_index]), 6),
        },
        "time": {
            "crest_factor": rounded(float(np.max(absolute)) / max(rms, 1e-12)),
            "decay_t20_extrapolated_t60_seconds": decay_t20,
            "peak_offset_ms": rounded(peak_offset * 1000.0 / rate, 6),
            "rms": rounded(rms),
            "temporal_centroid_seconds": rounded(temporal_centroid),
            "zero_crossing_rate": rounded(zero_crossings),
        },
        "transient_envelope_db": envelope,
        "uncertainty": uncertainty,
    }


def content_object_path(kind: str, digest: str, suffix: str) -> str:
    return f"objects/{kind}/{digest[:2]}/{digest}{suffix}"


def store_object(
    files: dict[str, bytes], kind: str, data: bytes, suffix: str
) -> dict[str, Any]:
    digest = sha256_bytes(data)
    path = content_object_path(kind, digest, suffix)
    prior = files.setdefault(path, data)
    if prior != data:
        raise CorpusError("content-addressed object collision")
    return {"bytes": len(data), "path": path, "sha256": digest}


def identified_axis_mask() -> dict[str, bool]:
    return {
        "absolute_force": False,
        "acoustic_pseudo_target": True,
        "geometry": False,
        "impact_position": False,
        "listener_position": False,
        "material_composition": False,
        "material_identity": True,
        "object_identity": True,
        "support_condition": False,
        "transfer_response": False,
        "waveform": True,
    }


def transfer_axis_mask() -> dict[str, bool]:
    return {
        "absolute_force": False,
        "acoustic_pseudo_target": True,
        "geometry": True,
        "impact_position": True,
        "listener_position": True,
        "material_composition": False,
        "material_identity": True,
        "object_identity": True,
        "support_condition": False,
        "transfer_response": True,
        "waveform": True,
    }


def validate_identified_inputs(
    manifest: dict[str, Any], source_manifest: dict[str, Any], report: dict[str, Any]
) -> tuple[list[dict[str, Any]], dict[str, dict[str, Any]]]:
    if (
        manifest.get("schema")
        != "nextengine.experimental-physical-sound-identified-corpus.manifest.v1"
    ):
        raise CorpusError("identified manifest schema changed")
    if (
        source_manifest.get("schema")
        != "nextengine.experimental-physical-sound-internet-sources.manifest.v1"
    ):
        raise CorpusError("source manifest schema changed")
    if (
        report.get("schema")
        != "nextengine.experimental-physical-sound-identified-corpus.report.v1"
    ):
        raise CorpusError("identified report schema changed")
    entries = require_list(report.get("entries"), "identified report entries")
    sources_raw = require_list(
        source_manifest.get("sources"), "source manifest sources"
    )
    sources: dict[str, dict[str, Any]] = {}
    for raw in sources_raw:
        source = require_dict(raw, "source manifest row")
        source_id = source.get("id")
        if not isinstance(source_id, str) or source_id in sources:
            raise CorpusError("source manifest IDs changed")
        sources[source_id] = source
    if len(entries) != report.get("recording_count"):
        raise CorpusError("identified report recording count changed")
    if len({entry.get("entry_id") for entry in entries}) != len(entries):
        raise CorpusError("identified entry IDs are not unique")
    for entry in entries:
        require_hash(entry.get("audio_file_sha256"), "identified audio payload")
        source = sources.get(entry.get("source_id"))
        if source is None:
            raise CorpusError("identified entry source is absent")
        for key in ("publisher_id", "project_id", "declared_revision"):
            if entry.get(key) != source.get(key):
                raise CorpusError(
                    "identified entry lineage disagrees with source manifest"
                )
    return entries, sources


def build_identified_record(
    entry: dict[str, Any],
    family_id: str,
    family: dict[str, Any],
    profile: dict[str, Any],
    payload: bytes,
    ffmpeg_path: Path,
    files: dict[str, bytes],
) -> dict[str, Any]:
    source_rate = entry.get("sample_rate_hz")
    if type(source_rate) is not int or source_rate <= 0:
        raise CorpusError("identified sample rate is invalid")
    decoded = decode_audio(payload, source_rate, ffmpeg_path)
    resampled = resample_mono(decoded, source_rate)
    canonical, normalization = canonical_segment(resampled)
    wav = store_object(files, "pcm", wav_pcm16_bytes(canonical), ".wav")
    target_value = extract_acoustic_target(canonical)
    target = store_object(files, "features", canonical_json(target_value), ".json")
    parent = match_parent_alias(
        profile,
        family_id,
        {"source_id": entry["source_id"]},
        entry["object_group_id"],
    )
    return {
        "acoustic_target": target,
        "axis_mask": identified_axis_mask(),
        "canonical_pcm": wav,
        "family_component_id": family["family_component_id"],
        "family_id": family_id,
        "material_label": entry["material_label"],
        "normalization": normalization,
        "object_id": entry["object_id"],
        "physical_parent_id": parent,
        "provenance": {
            "declared_revision": entry["declared_revision"],
            "license_expression": entry["license_expression"],
            "project_id": entry["project_id"],
            "publisher_id": entry["publisher_id"],
            "redistribution_policy": entry["redistribution_policy"],
            "source_payload_bytes": entry["audio_file_bytes"],
            "source_payload_sha256": entry["audio_file_sha256"],
        },
        "record_id": f"identified:{entry['entry_id']}",
        "recording_id": entry["recording_id"],
        "role": family["role"],
        "source_kind": "identified_recording",
        "source_sample_rate_hz": source_rate,
    }


def load_realimpact_entries(
    roots: list[Path], profile: dict[str, Any]
) -> list[tuple[Path, dict[str, Any], dict[str, Any]]]:
    expected = {row["root_name"]: row for row in profile["realimpact_inputs"]}
    supplied = {root.name: root for root in roots}
    if set(supplied) != set(expected) or len(supplied) != len(roots):
        raise CorpusError("REALIMPACT roots do not match the frozen input set")
    result = []
    for name in sorted(expected):
        root = supplied[name]
        if root.is_symlink() or not root.is_dir():
            raise CorpusError("REALIMPACT root must be a non-symlink directory")
        manifest = load_bound_json(
            root / "manifest.json",
            expected[name]["manifest"],
            f"REALIMPACT {name} manifest",
        )
        entries = require_list(manifest.get("entries"), "REALIMPACT entries")
        if len(entries) != 1:
            raise CorpusError("each frozen REALIMPACT input must contain one entry")
        result.append((root, manifest, require_dict(entries[0], "REALIMPACT entry")))
    return result


def build_transfer_record(
    root: Path,
    entry: dict[str, Any],
    family: dict[str, Any],
    profile: dict[str, Any],
    files: dict[str, bytes],
) -> tuple[dict[str, Any], int]:
    payload_binding = require_dict(
        entry.get("audio_payload"), "REALIMPACT audio payload"
    )
    payload_path = root / payload_binding.get("path", "")
    data = read_regular(payload_path, "REALIMPACT transfer payload")
    if sha256_bytes(data) != payload_binding.get("sha256"):
        raise CorpusError("REALIMPACT transfer payload hash changed")
    if len(data) != entry.get("sample_count") * 4:
        raise CorpusError("REALIMPACT transfer payload length changed")
    samples = np.frombuffer(data, dtype="<f4").astype(np.float64)
    if entry.get("sample_rate_hz") != CORPUS_POLICY["sample_rate_hz"]:
        samples = resample_mono(samples, entry["sample_rate_hz"])
    canonical, normalization = canonical_segment(samples)
    wav = store_object(files, "pcm", wav_pcm16_bytes(canonical), ".wav")
    target = store_object(
        files, "features", canonical_json(extract_acoustic_target(canonical)), ".json"
    )
    family_id = family["family_id"]
    parent = match_parent_alias(
        profile,
        family_id,
        {"object_id": entry["object_id"]},
        f"realimpact:{entry['object_family_id']}",
    )
    record = {
        "acoustic_target": target,
        "axis_mask": transfer_axis_mask(),
        "canonical_pcm": wav,
        "family_component_id": family["family_component_id"],
        "family_id": family_id,
        "material_label": entry["material_family"].title(),
        "normalization": normalization,
        "object_id": entry["object_id"],
        "observed_physics": {
            "geometry_revision": entry["geometry_revision"],
            "impact_position_id": entry["impact_position_id"],
            "impact_position_metres": entry["impact_position_metres"],
            "listener_condition_id": entry["listener_condition_id"],
            "listener_position_metres": entry["listener_position_metres"],
            "transfer_kind": entry["recording_kind"],
        },
        "physical_parent_id": parent,
        "provenance": {
            "declared_revision": entry["source_adapter"]["repository_revision"],
            "license_expression": "MIT",
            "project_id": "realimpact",
            "publisher_id": "samuel-clarke",
            "redistribution_policy": "external_research_only",
            "source_payload_bytes": len(data),
            "source_payload_sha256": sha256_bytes(data),
        },
        "record_id": f"transfer:{entry['id']}",
        "recording_id": str(entry["source_adapter"]["row_index"]),
        "role": family["role"],
        "source_kind": "force_deconvolved_transfer",
        "source_sample_rate_hz": entry["sample_rate_hz"],
    }
    return record, len(data)


def validate_role_isolation(
    records: list[dict[str, Any]], expected_counts: dict[str, Any]
) -> dict[str, Any]:
    parent_roles: dict[str, set[str]] = defaultdict(set)
    component_roles: dict[str, set[str]] = defaultdict(set)
    for record in records:
        if set(record["axis_mask"]) != set(AXES):
            raise CorpusError("axis mask is incomplete")
        parent_roles[record["physical_parent_id"]].add(record["role"])
        component_roles[record["family_component_id"]].add(record["role"])
    cross_parent = {key: value for key, value in parent_roles.items() if len(value) > 1}
    cross_component = {
        key: value for key, value in component_roles.items() if len(value) > 1
    }
    if cross_parent or cross_component:
        raise CorpusError("physical parent or alias component crosses C0 roles")
    role_counts = Counter(record["role"] for record in records)
    if dict(sorted(role_counts.items())) != expected_counts["role_records"]:
        raise CorpusError("C0 role record counts changed")
    if len(parent_roles) != expected_counts["physical_parents"]:
        raise CorpusError("C0 physical-parent count changed")
    return {
        "cross_role_component_count": 0,
        "cross_role_parent_count": 0,
        "physical_parent_count": len(parent_roles),
        "role_record_counts": dict(sorted(role_counts.items())),
    }


def build_outputs(
    profile_bytes: bytes,
    profile: dict[str, Any],
    d1_bytes: bytes,
    families: dict[str, dict[str, Any]],
    d1_roster: dict[str, Any],
    identified_manifest: dict[str, Any],
    source_manifest: dict[str, Any],
    identified_report: dict[str, Any],
    resolver: PayloadResolver,
    realimpact_entries: list[tuple[Path, dict[str, Any], dict[str, Any]]],
) -> dict[str, bytes]:
    files: dict[str, bytes] = {}
    entries, _ = validate_identified_inputs(
        identified_manifest, source_manifest, identified_report
    )
    mapping = family_lookup(profile)
    ffmpeg_path = Path(profile["environment"]["ffmpeg"]["path"])
    records = []
    decoded_samples = 0
    for entry in sorted(entries, key=lambda row: row["entry_id"]):
        key = (entry["publisher_id"], entry["project_id"])
        family_id = mapping.get(key)
        if family_id is None:
            raise CorpusError(f"no D1 family mapping for {key}")
        payload = resolver.resolve(entry)
        record = build_identified_record(
            entry, family_id, families[family_id], profile, payload, ffmpeg_path, files
        )
        decoded_samples += record["normalization"]["source_sample_count_48k"]
        records.append(record)

    transfer_family = families.get("samuel-clarke--realimpact")
    if transfer_family is None:
        if realimpact_entries:
            raise CorpusError("REALIMPACT input has no D1 family")
    else:
        for root, _manifest, entry in realimpact_entries:
            record, payload_bytes = build_transfer_record(
                root, entry, transfer_family, profile, files
            )
            resolver.payload_bytes_read += payload_bytes
            decoded_samples += record["normalization"]["source_sample_count_48k"]
            records.append(record)

    records.sort(key=lambda row: row["record_id"])
    if len({record["record_id"] for record in records}) != len(records):
        raise CorpusError("C0 record IDs are not unique")
    counts = profile["expected_counts"]
    identified_count = sum(
        row["source_kind"] == "identified_recording" for row in records
    )
    transfer_count = sum(
        row["source_kind"] == "force_deconvolved_transfer" for row in records
    )
    if (
        identified_count != counts["identified_recordings"]
        or transfer_count != counts["realimpact_transfers"]
        or len(records) != counts["total_records"]
    ):
        raise CorpusError("C0 record counts changed")
    isolation = validate_role_isolation(records, counts)

    projections = {}
    for role in ROLES:
        rows = [
            {
                "acoustic_target": record["acoustic_target"],
                "axis_mask": record["axis_mask"],
                "canonical_pcm": record["canonical_pcm"],
                "family_component_id": record["family_component_id"],
                "family_id": record["family_id"],
                "material_label": record["material_label"],
                "physical_parent_id": record["physical_parent_id"],
                "record_id": record["record_id"],
            }
            for record in records
            if record["role"] == role
        ]
        projection = {
            "record_count": len(rows),
            "role": role,
            "rows": rows,
            "rows_root_sha256": sha256_bytes(canonical_json(rows)),
            "schema": PROJECTION_SCHEMA,
        }
        data = canonical_json(projection)
        path = f"projections/{role}.json"
        files[path] = data
        projections[role] = {
            "bytes": len(data),
            "path": path,
            "rows_root_sha256": projection["rows_root_sha256"],
            "sha256": sha256_bytes(data),
        }

    manifest_core = {
        "authority": AUTHORITY,
        "corpus_policy": CORPUS_POLICY,
        "d1_roster_root_sha256": d1_roster["roster_root_sha256"],
        "d1_roster_sha256": sha256_bytes(d1_bytes),
        "feature_policy": FEATURE_POLICY,
        "profile_sha256": sha256_bytes(profile_bytes),
        "projections": projections,
        "records": records,
        "schema": MANIFEST_SCHEMA,
    }
    manifest_root = sha256_bytes(canonical_json(manifest_core))
    manifest = {**manifest_core, "manifest_root_sha256": manifest_root}
    manifest_bytes = canonical_json(manifest)
    files["manifest.json"] = manifest_bytes

    material_counts = Counter(record["material_label"] for record in records)
    axis_counts = {
        axis: sum(record["axis_mask"][axis] for record in records) for axis in AXES
    }
    card = {
        "axis_observed_record_counts": axis_counts,
        "claim": (
            "disclosed internet evidence for repeatable lab training/calibration only; "
            "waveform-derived acoustic values are pseudo-targets, not measured mechanics"
        ),
        "limitations": [
            "absolute amplitude is not comparable across source projects",
            "identified recordings do not expose force, geometry, contact, listener, or support",
            "REALIMPACT rows expose transfer geometry/contact/listener but not raw force profiles",
            "no record grants protected admission, product, cooker, demo, or runtime authority",
        ],
        "material_record_counts": dict(sorted(material_counts.items())),
        "physical_parent_count": isolation["physical_parent_count"],
        "record_count": len(records),
        "role_record_counts": isolation["role_record_counts"],
        "schema": CARD_SCHEMA,
    }
    card_bytes = canonical_json(card)
    files["corpus-card.json"] = card_bytes

    access = {
        "archive_payload_bytes_read": resolver.archive_bytes_read,
        "candidate_output_bytes_read": 0,
        "decoded_samples_48k": decoded_samples,
        "model_bytes_read": 0,
        "network_requests": 0,
        "protected_payload_bytes_read": 0,
        "protected_role_signal_read": 0,
        "source_payload_bytes_read": resolver.payload_bytes_read,
    }
    access_ledger = {
        "access": access,
        "authority": AUTHORITY,
        "d1_roster_sha256": sha256_bytes(d1_bytes),
        "profile_sha256": sha256_bytes(profile_bytes),
        "schema": ACCESS_SCHEMA,
    }
    access_bytes = canonical_json(access_ledger)
    files["access-ledger.json"] = access_bytes

    gates = {
        "all_payloads_hash_verified": True,
        "all_records_have_complete_axis_masks": all(
            set(record["axis_mask"]) == set(AXES) for record in records
        ),
        "canonical_pcm_is_mono_48k_s16le": True,
        "d1_roster_hash_bound": True,
        "expected_record_counts": len(records) == counts["total_records"],
        "generator_validator_components_disjoint": (
            isolation["cross_role_component_count"] == 0
        ),
        "physical_parents_role_disjoint": isolation["cross_role_parent_count"] == 0,
        "protected_and_model_access_zero": (
            access["protected_payload_bytes_read"] == 0
            and access["protected_role_signal_read"] == 0
            and access["model_bytes_read"] == 0
        ),
    }
    if not all(gates.values()):
        raise CorpusError("C0 conjunctive gate failed")
    report = {
        "access": access,
        "artifacts": {
            "access_ledger_sha256": sha256_bytes(access_bytes),
            "corpus_card_sha256": sha256_bytes(card_bytes),
            "manifest_root_sha256": manifest_root,
            "manifest_sha256": sha256_bytes(manifest_bytes),
            "profile_sha256": sha256_bytes(profile_bytes),
        },
        "authority": AUTHORITY,
        "decision": DECISION,
        "gates": gates,
        "result": {
            "content_object_count": sum(path.startswith("objects/") for path in files),
            "identified_recordings": identified_count,
            "physical_parents": isolation["physical_parent_count"],
            "realimpact_transfers": transfer_count,
            "role_records": isolation["role_record_counts"],
            "total_records": len(records),
        },
        "schema": REPORT_SCHEMA,
    }
    files["profile.json"] = profile_bytes
    files["report.json"] = canonical_json(report)
    return files


def has_symlink_component(path: Path) -> bool:
    absolute = Path(os.path.abspath(path))
    current = Path(absolute.anchor)
    for part in absolute.parts[1:]:
        current /= part
        if current.is_symlink():
            return True
    return False


def prepare_output(output: Path) -> Path:
    if has_symlink_component(output):
        raise CorpusError("output path cannot contain symlinks")
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise CorpusError("output must remain outside the repository")
    if resolved.exists():
        raise CorpusError(f"refusing to replace existing output: {resolved}")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    return resolved


def publish_directory(output: Path, files: dict[str, bytes]) -> Path:
    destination = prepare_output(output)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent)
    )
    try:
        for relative, data in sorted(files.items()):
            path = Path(relative)
            if path.is_absolute() or ".." in path.parts:
                raise CorpusError("output path escapes the corpus root")
            target = staging / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def run(
    profile_path: Path,
    d1_roster_path: Path,
    identified_manifest_path: Path,
    source_manifest_path: Path,
    identified_report_path: Path,
    cache: Path,
    realimpact_roots: list[Path],
    output: Path,
) -> Path:
    profile_bytes, raw_profile = read_canonical_profile(profile_path)
    profile = validate_profile(raw_profile)
    d1_bytes, families, d1_roster = load_d1_roster(d1_roster_path, profile)
    bindings = profile["input_bindings"]
    identified_manifest = load_bound_json(
        identified_manifest_path, bindings["identified_manifest"], "identified manifest"
    )
    source_manifest = load_bound_json(
        source_manifest_path, bindings["source_manifest"], "source manifest"
    )
    identified_report = load_bound_json(
        identified_report_path, bindings["identified_report"], "identified report"
    )
    entries, sources = validate_identified_inputs(
        identified_manifest, source_manifest, identified_report
    )
    archive_reader = ArchiveReader(Path(profile["environment"]["libarchive"]["path"]))
    resolver = PayloadResolver(cache, sources, entries, archive_reader)
    realimpact_entries = load_realimpact_entries(realimpact_roots, profile)
    files = build_outputs(
        profile_bytes,
        profile,
        d1_bytes,
        families,
        d1_roster,
        identified_manifest,
        source_manifest,
        identified_report,
        resolver,
        realimpact_entries,
    )
    return publish_directory(output, files)


def main() -> None:
    arguments = parse_arguments()
    destination = run(
        arguments.profile,
        arguments.d1_roster,
        arguments.identified_manifest,
        arguments.source_manifest,
        arguments.identified_report,
        arguments.cache,
        arguments.realimpact_root,
        arguments.output,
    )
    print(destination)


if __name__ == "__main__":
    main()
