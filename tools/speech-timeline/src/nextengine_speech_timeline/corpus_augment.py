"""Deterministic degradation generation for the reliability corpus (R2 step).

Implements the §9 increment of ``DEV-SPEECH-RELIABILITY-001``.  For every
admitted clean clip of a hash-closed prepared index the generator emits one
unchanged control reference plus at most ``max_variants_per_clip`` derived
conditions drawn from the recipe's manifest-fixed augmentation grid:

- ``attenuation`` — level attenuation (never labeled whisper);
- ``musan-noise`` — additive MUSAN noise at declared SNR values;
- ``rirs-room-response`` — convolution with a pinned RIRS_NOISES response;
- ``noise-plus-room-response`` — room response first, then additive noise;
- ``microphone-band-eq`` — brickwall band coloration between cut points;
- ``bounded-codec-damage`` — peak scaling with hard clipping plus seeded
  20 ms frame zero-loss.

Pinned numeric rules (documented, deterministic):

- All math runs in float64 over samples scaled to ``[-1, 1)``; every output
  sample converts back with ``round`` and clamps to int16 (profile
  ``pcm16-round-clamp-v0``).  Sources are already 16 kHz mono, so no
  resampler participates.
- RIR convolution uses unit-energy responses (``h /= ||h||_2``), full FFT
  convolution trimmed causally to the input length.
- Additive noise targets ``signal_rms / 10^(snr_db / 20)`` measured over the
  whole clip; the segment offset comes from the condition subseed, and short
  assets wrap deterministically.
- Band EQ is a real-FFT brickwall between the cut points.
- Frame loss zeroes 20 ms frames where a PCG64 draw seeded by the subseed
  falls below ``frame_loss_permyriad / 10000``.

Split discipline (§7): derivatives inherit the parent split verbatim, and
noise/RIR pools are drawn only from asset rows whose split equals the
parent's, so cross-partition leakage through shared audio is impossible by
construction.  Derivation rows copy the parent's ``normalized_reference``
verbatim — augmentation MUST NOT modify the transcript — and carry
``source_clip_id`` plus both audio hashes so every replay pairs back to its
clean control unambiguously.

Every derived file is re-read through the standard library before hashing,
and the augmented index publishes atomically with private permissions
outside this repository.
"""

from __future__ import annotations

from dataclasses import dataclass, field
import hashlib
import json
import math
from pathlib import Path
import wave

import numpy as np

from .profile import REPOSITORY_ROOT
from .reliability import (
    MAX_AUGMENTATION_ASSET_SAMPLES,
    _atomic_write,
    _canonical_json,
    _external_directory,
    _sha256_file,
    load_dataset_recipe,
    verify_indexed_audio,
)


AUGMENTED_INDEX_KIND = "nextengine.speech-reliability.augmented-index"
DERIVATION_KIND = "speech-reliability.derivation-record-v0"
PCM16_PROFILE = "pcm16-round-clamp-v0"

SUPPORTED_CONDITION_KINDS = (
    "attenuation",
    "musan-noise",
    "rirs-room-response",
    "noise-plus-room-response",
    "microphone-band-eq",
    "bounded-codec-damage",
)
CONDITION_PARAM_KEYS: dict[str, set[str]] = {
    "attenuation": {"gain_db"},
    "musan-noise": {"snr_db"},
    "rirs-room-response": {"wet_mix_permyriad"},
    "noise-plus-room-response": {"snr_db", "wet_mix_permyriad"},
    "microphone-band-eq": {"low_cut_hz", "high_cut_hz"},
    "bounded-codec-damage": {"clip_peak_dbfs", "frame_loss_permyriad"},
}

MAX_DERIVED_FILES = 200_000
MAX_RIR_SAMPLES = 64_000  # 4 s cap on impulse responses
FRAME_SAMPLES = 320  # 20 ms frames for loss simulation
SAMPLE_RATE_HZ = 16_000
RMS_SILENCE_FLOOR = 1e-4


class ReliabilityAugmentError(RuntimeError):
    """A stable fail-closed degradation-generation error."""


class _SilentSource(Exception):
    """Internal marker: a condition had no usable acoustic evidence."""


@dataclass
class _SkipLog:
    silent_source: list[str] = field(default_factory=list)
    bad_output: list[str] = field(default_factory=list)
    counts: dict[str, int] = field(default_factory=dict)

    def add(self, reason: str, label: str) -> None:
        bucket = getattr(self, reason)
        if len(bucket) < 10:
            bucket.append(label)
        self.counts[reason] = self.counts.get(reason, 0) + 1


def _read_all_rows(path: Path) -> tuple[list[dict[str, object]], str]:
    """Load every prepared row (speech plus noise/RIR assets) for pooling."""

    resolved = path.expanduser().resolve()
    try:
        raw = resolved.read_bytes()
    except OSError as error:
        raise ReliabilityAugmentError(f"cannot read prepared index: {error}") from error
    if not 0 < len(raw) <= 512 * 1024 * 1024:
        raise ReliabilityAugmentError("prepared index size is out of bounds")
    rows: list[dict[str, object]] = []
    for line_number, line in enumerate(raw.decode("utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise ReliabilityAugmentError(
                f"prepared index line {line_number} is not valid JSON: {error}"
            ) from error
        if not isinstance(value, dict) or value.get("entry_kind") not in {
            "speech",
            "noise",
            "rir",
        }:
            raise ReliabilityAugmentError(
                f"prepared index line {line_number} has an unsupported entry_kind"
            )
        required = {
            # The prepare-time contract pins 16 kHz mono pcm_s16le; the
            # prepared index itself carries only these fields.
            "schema_version",
            "source_id",
            "split",
            "relative_audio_path",
            "audio_sha256",
            "samples",
        }
        if value["entry_kind"] == "speech":
            required |= {"clip_id", "normalized_reference", "normalizer_profile"}
        else:
            required |= {"asset_id", "partition_group_hash"}
        missing = required - set(value)
        if missing or not isinstance(value.get("samples"), int):
            raise ReliabilityAugmentError(
                f"prepared index line {line_number} misses fields: "
                f"{sorted(missing) if missing else 'samples'}"
            )
        rows.append(value)
    if not rows:
        raise ReliabilityAugmentError("prepared index contains no rows")
    return rows, f"sha256:{hashlib.sha256(raw).hexdigest()}"


def _subseed_hex(*parts: str) -> str:
    return hashlib.sha256("\0".join(parts).encode("utf-8")).hexdigest()


def _bucket(subseed_hex: str, salt: str, modulo: int) -> int:
    digest = hashlib.sha256(f"{subseed_hex}\0{salt}".encode("utf-8")).hexdigest()
    return int(digest[:16], 16) % max(1, modulo)


def _read_wav(path: Path) -> np.ndarray:
    try:
        with wave.open(str(path), "rb") as handle:
            if (
                handle.getframerate(),
                handle.getnchannels(),
                handle.getsampwidth(),
            ) != (SAMPLE_RATE_HZ, 1, 2):
                raise ReliabilityAugmentError(
                    f"wav must be 16 kHz mono pcm_s16le: {path}"
                )
            raw = handle.readframes(handle.getnframes())
    except (OSError, EOFError, wave.Error) as error:
        raise ReliabilityAugmentError(f"cannot read wav {path}: {error}") from error
    # Internal DSP domain is [-1, 1); writing scales back by 32767.
    return np.frombuffer(raw, dtype="<i2").astype(np.float64) / 32_768.0


def _write_and_hash_wav(path: Path, samples_f: np.ndarray) -> tuple[str, int]:
    ints = np.clip(np.round(samples_f * 32767.0), -32768, 32767).astype("<i2")
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(SAMPLE_RATE_HZ)
        handle.writeframes(ints.tobytes())
    return f"sha256:{_sha256_file(path)}", int(ints.size)


def _rms(x: np.ndarray) -> float:
    if x.size == 0:
        return 0.0
    return float(math.sqrt(float(np.mean(x * x))))


def _fft_convolve_causal(x: np.ndarray, h: np.ndarray) -> np.ndarray:
    size = 1 << int(math.ceil(math.log2(x.size + h.size - 1)))
    spectrum = np.fft.rfft(x, size) * np.fft.rfft(h, size)
    full = np.fft.irfft(spectrum, size)[: x.size]
    if not bool(np.all(np.isfinite(full))):
        raise ReliabilityAugmentError("convolution produced non-finite samples")
    return full


def _segment_noise(
    noise: np.ndarray, length: int, subseed_hex: str
) -> np.ndarray:
    """Deterministically cut (or wrap-extend) one noise segment."""

    if noise.size == 0:
        raise ReliabilityAugmentError("noise asset decoded to zero samples")
    if noise.size >= length:
        offset = _bucket(subseed_hex, "noise-offset", noise.size - length + 1)
        return noise[offset : offset + length]
    repeats = int(math.ceil(length / noise.size))
    tiled = np.tile(noise, repeats)[:length]
    shift = _bucket(subseed_hex, "noise-wrap-shift", noise.size)
    return np.roll(tiled, -shift)


def _load_rir_unit_energy(store_root: Path, row: dict[str, object]) -> tuple[np.ndarray, str]:
    relative, audio_hash, _samples = verify_indexed_audio(
        store_root,
        row,
        str(row["source_id"]),
        str(row["asset_id"]),
        maximum_samples=MAX_AUGMENTATION_ASSET_SAMPLES,
    )
    rir = _read_wav(store_root / relative)[:MAX_RIR_SAMPLES]
    energy = float(math.sqrt(float(np.sum(rir * rir))))
    if energy < RMS_SILENCE_FLOOR:
        raise ReliabilityAugmentError("selected rir has near-zero energy")
    return rir / energy, audio_hash


def _pick_grid(parameters: dict[str, object], key: str, subseed_hex: str):
    values = parameters[key]
    if not isinstance(values, list) or not values:
        raise ReliabilityAugmentError(
            f"augmentation grid for {key} must be a non-empty list"
        )
    return values[_bucket(subseed_hex, f"param:{key}", len(values))]


def _condition_parameters(
    kind: str, declared: dict[str, object], subseed_hex: str
) -> dict[str, object]:
    expected = CONDITION_PARAM_KEYS[kind]
    if not isinstance(declared, dict) or set(declared) != expected:
        raise ReliabilityAugmentError(
            f"condition {kind} parameters must contain exactly "
            f"{sorted(expected)} as grid lists"
        )
    return {key: _pick_grid(declared, key, subseed_hex) for key in sorted(expected)}


class _Pools:
    """Split-matched noise/RIR asset rows from the prepared index."""

    def __init__(self, rows: list[dict[str, object]]) -> None:
        self._by_key: dict[tuple[str, str], list[dict[str, object]]] = {}
        for row in rows:
            key = (str(row["entry_kind"]), str(row["split"]))
            self._by_key.setdefault(key, []).append(row)

    def pick(self, kind: str, split: str, subseed_hex: str) -> dict[str, object]:
        pool = self._by_key.get((kind, split))
        if not pool:
            raise ReliabilityAugmentError(
                f"{kind} pool is empty for split {split!r}"
            )
        return pool[_bucket(subseed_hex, f"pool:{kind}", len(pool))]


def _derive_condition(
    kind: str,
    clean_f: np.ndarray,
    params: dict[str, object],
    *,
    split: str,
    pools: _Pools,
    store_root: Path,
    subseed: str,
) -> tuple[np.ndarray, dict[str, object], list[str]]:
    """Compute one derived condition; returns samples, aux identity, order."""

    aux: dict[str, object] = {}
    order: list[str] = []

    def noise_segment() -> np.ndarray:
        row = pools.pick("noise", split, subseed)
        relative, audio_hash, _ = verify_indexed_audio(
            store_root,
            row,
            str(row["source_id"]),
            str(row["asset_id"]),
            maximum_samples=MAX_AUGMENTATION_ASSET_SAMPLES,
        )
        aux["noise_asset_id"] = row["asset_id"]
        aux["noise_sha256"] = audio_hash
        segment = _segment_noise(_read_wav(store_root / relative), clean_f.size, subseed)
        if _rms(segment) < RMS_SILENCE_FLOOR:
            raise _SilentSource()
        return segment

    def convolve_with_rir(current: np.ndarray) -> np.ndarray:
        row = pools.pick("rir", split, subseed)
        rir, audio_hash = _load_rir_unit_energy(store_root, row)
        aux["rir_asset_id"] = row["asset_id"]
        aux["rir_sha256"] = audio_hash
        order.append("rir")
        return _fft_convolve_causal(current, rir)

    if kind == "attenuation":
        gain_db = float(params["gain_db"])  # type: ignore[arg-type]
        order.append("gain")
        y = clean_f * (10.0 ** (gain_db / 20.0))
        return y, aux, order

    if kind == "musan-noise":
        snr_db = float(params["snr_db"])  # type: ignore[arg-type]
        order.append("noise")
        noise = noise_segment()
        signal_rms = _rms(clean_f)
        if signal_rms < RMS_SILENCE_FLOOR:
            raise _SilentSource()
        noise_level = _rms(noise)
        target_noise_rms = signal_rms / (10.0 ** (snr_db / 20.0))
        return clean_f + noise * (target_noise_rms / noise_level), aux, order

    if kind == "rirs-room-response":
        wet_permyriad = float(params["wet_mix_permyriad"])  # type: ignore[arg-type]
        wet = wet_permyriad / 10_000.0
        reverbed = convolve_with_rir(clean_f)
        order.append("wet-mix")
        return wet * reverbed + (1.0 - wet) * clean_f, aux, order

    if kind == "noise-plus-room-response":
        snr_db = float(params["snr_db"])  # type: ignore[arg-type]
        wet_permyriad = float(params["wet_mix_permyriad"])  # type: ignore[arg-type]
        wet = wet_permyriad / 10_000.0
        reverbed = convolve_with_rir(clean_f)
        mixed = wet * reverbed + (1.0 - wet) * clean_f
        order.append("wet-mix")
        signal_rms = _rms(mixed)
        if signal_rms < RMS_SILENCE_FLOOR:
            raise _SilentSource()
        noise_row = pools.pick("noise", split, subseed)
        relative, audio_hash, _ = verify_indexed_audio(
            store_root,
            noise_row,
            str(noise_row["source_id"]),
            str(noise_row["asset_id"]),
            maximum_samples=MAX_AUGMENTATION_ASSET_SAMPLES,
        )
        aux["noise_asset_id"] = noise_row["asset_id"]
        aux["noise_sha256"] = audio_hash
        noise = _segment_noise(
            _read_wav(store_root / relative), clean_f.size, subseed
        )
        noise_rms = _rms(noise)
        if noise_rms < RMS_SILENCE_FLOOR:
            raise _SilentSource()
        order.append("noise")
        return mixed + noise * (
            (signal_rms / (10.0 ** (snr_db / 20.0))) / noise_rms
        ), aux, order

    if kind == "microphone-band-eq":
        low_hz = float(params["low_cut_hz"])  # type: ignore[arg-type]
        high_hz = float(params["high_cut_hz"])  # type: ignore[arg-type]
        size = clean_f.size
        freqs = np.fft.rfftfreq(size, d=1.0 / SAMPLE_RATE_HZ)
        mask = ((freqs >= low_hz) & (freqs <= high_hz)).astype(np.float64)
        order.append("eq")
        shaped = np.fft.irfft(np.fft.rfft(clean_f) * mask, size)
        if not bool(np.all(np.isfinite(shaped))):
            raise ReliabilityAugmentError("band EQ produced non-finite samples")
        return shaped, aux, order

    if kind == "bounded-codec-damage":
        peak_dbfs = float(params["clip_peak_dbfs"])  # type: ignore[arg-type]
        loss_permyriad = float(params["frame_loss_permyriad"])  # type: ignore[arg-type]
        current_peak = float(np.max(np.abs(clean_f))) if clean_f.size else 0.0
        order.append("peak-scale")
        if current_peak > RMS_SILENCE_FLOOR:
            scaled = clean_f * (
                (10.0 ** (peak_dbfs / 20.0)) / current_peak
            )
        else:
            raise _SilentSource()
        clipped = np.clip(scaled, -1.0, 1.0)
        order.append("frame-loss")
        loss_probability = loss_permyriad / 10_000.0
        rng = np.random.Generator(np.random.PCG64(int(subseed[:16], 16)))
        frame_count = int(math.ceil(clipped.size / FRAME_SAMPLES))
        lost = rng.random(frame_count) < loss_probability
        mask = np.repeat(lost, FRAME_SAMPLES)[: clipped.size]
        return clipped * (~mask).astype(np.float64), aux, order

    raise ReliabilityAugmentError(f"unsupported degradation kind: {kind}")


def augment_corpus(
    store: Path,
    *,
    manifest_path: Path,
    prepared_index_path: Path,
    out_index: Path,
    splits: tuple[str, ...] | None = None,
    limit: int | None = None,
    offset: int = 0,
) -> dict[str, object]:
    """Generate control references plus bounded derived conditions."""

    out_destination = out_index.expanduser().resolve()
    if out_destination == REPOSITORY_ROOT or out_destination.is_relative_to(
        REPOSITORY_ROOT
    ):
        raise ReliabilityAugmentError(
            "augmented index must stay outside the repository"
        )
    store_root = _external_directory(store, "corpus store")
    recipe = load_dataset_recipe(manifest_path)
    profile = recipe.augmentation_profile
    conditions_raw = profile["conditions"]
    assert isinstance(conditions_raw, list)
    for item in conditions_raw:
        kind_value = item.get("kind") if isinstance(item, dict) else None
        if kind_value not in SUPPORTED_CONDITION_KINDS:
            raise ReliabilityAugmentError(
                f"unsupported degradation kind in recipe: {kind_value!r}"
            )
    max_variants = int(profile["max_variants_per_clip"])  # type: ignore[arg-type]
    seed = str(profile["seed"])

    rows, _prepared_hash_unused = _read_all_rows(prepared_index_path)
    speech_rows = [row for row in rows if row["entry_kind"] == "speech"]
    pools = _Pools([row for row in rows if row["entry_kind"] != "speech"])
    selected = [
        row
        for row in speech_rows
        if splits is None or row["split"] in splits
    ]
    if offset < 0:
        raise ReliabilityAugmentError("offset must be non-negative")
    selected = selected[offset:]
    if limit is not None:
        if limit <= 0:
            raise ReliabilityAugmentError("limit must be positive")
        selected = selected[:limit]

    out_rows: list[dict[str, object]] = []
    skips = _SkipLog()
    kind_counts: dict[str, int] = {}
    split_counts: dict[str, int] = {}
    emitted_files = 0

    for row in selected:
        clip_id = str(row["clip_id"])
        split = str(row["split"])
        try:
            relative, audio_hash, samples = verify_indexed_audio(
                store_root, row, str(row["source_id"]), clip_id
            )
        except Exception as error:  # noqa: BLE001 - surfaced as typed failure
            raise ReliabilityAugmentError(
                f"parent clip failed verification: {clip_id}: {error}"
            ) from error
        clean_f = _read_wav(store_root / relative)

        control_subseed = _subseed_hex(seed, clip_id, "control")
        out_rows.append(
            {
                "kind": DERIVATION_KIND,
                "schema_version": 0,
                "entry_kind": "speech",
                "derivation_id": f"{clip_id}-a0",
                "clip_id": f"{clip_id}-a0",
                "source_id": str(row["source_id"]),
                "source_clip_id": clip_id,
                "source_audio_sha256": audio_hash,
                "condition_kind": "control",
                "parameters": {},
                "transform_order": ["control"],
                "seed_material": control_subseed,
                "split": split,
                "relative_audio_path": relative,
                "audio_sha256": audio_hash,
                "samples": samples,
                "duration_ms": round(samples * 1_000 / SAMPLE_RATE_HZ),
                "normalized_reference": str(row["normalized_reference"]),
                "normalizer_profile": str(row["normalizer_profile"]),
            }
        )
        emitted_files += 1

        for variant_index, condition in enumerate(conditions_raw[:max_variants], start=1):
            kind = str(condition["kind"])
            subseed = _subseed_hex(seed, clip_id, kind)
            params = _condition_parameters(kind, dict(condition["parameters"]), subseed)  # type: ignore[arg-type]
            derivation_id = f"{clip_id}-a{variant_index}"
            if emitted_files >= MAX_DERIVED_FILES:
                raise ReliabilityAugmentError(
                    f"derived-file cap exceeded: {MAX_DERIVED_FILES}"
                )
            destination = store_root / "audio" / "derived" / f"{derivation_id}.wav"
            try:
                derived, aux, order = _derive_condition(
                    kind,
                    clean_f,
                    params,
                    split=split,
                    pools=pools,
                    store_root=store_root,
                    subseed=subseed,
                )
            except _SilentSource:
                skips.add("silent_source", f"{clip_id}:{kind}")
                continue
            audio_sha256, out_samples = _write_and_hash_wav(destination, derived)
            if out_samples != clean_f.size:
                destination.unlink(missing_ok=True)
                skips.add("bad_output", f"{clip_id}:{kind}:length")
                continue
            emitted_files += 1
            out_rows.append(
                {
                    "kind": DERIVATION_KIND,
                    "schema_version": 0,
                    "entry_kind": "speech",
                    "derivation_id": derivation_id,
                    "clip_id": derivation_id,
                    "source_id": str(row["source_id"]),
                    "source_clip_id": clip_id,
                    "source_audio_sha256": audio_hash,
                    "condition_kind": kind,
                    "parameters": params,
                    "transform_order": order,
                    "seed_material": subseed,
                    "split": split,
                    "relative_audio_path": destination.relative_to(
                        store_root
                    ).as_posix(),
                    "audio_sha256": audio_sha256,
                    "samples": out_samples,
                    "duration_ms": round(out_samples * 1_000 / SAMPLE_RATE_HZ),
                    "normalized_reference": str(row["normalized_reference"]),
                    "normalizer_profile": str(row["normalizer_profile"]),
                    **aux,
                }
            )
            kind_counts[f"{split}:{kind}"] = kind_counts.get(f"{split}:{kind}", 0) + 1
            split_counts[split] = split_counts.get(split, 0) + 1

    if not out_rows:
        raise ReliabilityAugmentError(
            "no derivations were generated; refusing to publish an empty index"
        )
    out_destination.parent.mkdir(parents=True, exist_ok=True)
    encoded = b"".join(_canonical_json(row) + b"\n" for row in sorted(
        out_rows, key=lambda item: str(item["derivation_id"])
    ))
    _atomic_write(out_destination, encoded)
    digest = f"sha256:{hashlib.sha256(encoded).hexdigest()}"
    return {
        "schema_version": 0,
        "kind": AUGMENTED_INDEX_KIND,
        "status": "complete",
        "augmentation_profile_hash": recipe.manifest_hash,
        "augmentation_seed": seed,
        "pcm16_profile": PCM16_PROFILE,
        "prepared_index_sha256": _prepared_hash_unused,
        "clips_selected": len(selected),
        "selection_offset": offset,
        "rows_written": len(out_rows),
        "derived_files": emitted_files,
        "counts_by_split_kind": dict(sorted(kind_counts.items())),
        "derived_by_split": dict(sorted(split_counts.items())),
        "skips": {
            "silent_source": skips.silent_source,
            "bad_output": skips.bad_output,
            "counts": dict(sorted(skips.counts.items())),
        },
        "out_index_sha256": digest,
    }
