#!/usr/bin/env python3
"""Shared frozen boundary for the R3A V4 development representation frontier."""

from __future__ import annotations

import hashlib
import io
import json
import math
import os
import platform
import shutil
import sys
import wave
from collections.abc import Iterable
from pathlib import Path
from typing import Any

import numpy as np
import scipy
from scipy import signal

STUDY_ID = "physical-sound-contact-field-r3a-v4-dev"
REVISION = "shared-modal-multiresidual-frontier-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v4-dev.manifest.v1"
PREFLIGHT_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v4-dev-preflight.report.v1"
)
FIT_REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v4-dev-fit.report.v1"
MODEL_SCHEMA = "nextengine.experimental-physical-sound-r3a-v4-dev-model.v1"
EVALUATION_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v4-dev-evaluation.report.v1"
)

IMPLEMENTATION_FILES = {
    "common": "physical_sound_contact_field_r3a_v4_common.py",
    "preflight": "physical_sound_contact_field_r3a_v4_preflight.py",
    "fit": "physical_sound_contact_field_r3a_v4_fit.py",
    "evaluate": "physical_sound_contact_field_r3a_v4_evaluate.py",
}

SAMPLE_RATE_HZ = 48_000
ANALYSIS_SAMPLES = 3 * SAMPLE_RATE_HZ
PEAK_ALIGNMENT_SAMPLE = 512
FIT_CONTACTS_PER_OBJECT = 3
DEVELOPMENT_CONTACT_INDEX = 3
SEALED_CONTACT_INDEX = 4

EVALUATION_MIN_HZ = 120.0
EVALUATION_MAX_HZ = 18_000.0
SPECTRAL_FLOOR_DB = -100.0
PRIMARY_ENDPOINTS = (
    "absolute_rms_level_error_db",
    "gain_matched_multiresolution_log_spectrum_rmse_db",
    "normalized_envelope_rmse",
    "modal_frequency_median_error_cents",
    "decay_t60_relative_error",
)
ABSOLUTE_THRESHOLDS = {
    "absolute_rms_level_error_db": 0.5,
    "gain_matched_multiresolution_log_spectrum_rmse_db": 4.0,
    "normalized_envelope_rmse": 0.20,
    "modal_frequency_median_error_cents": 100.0,
    "decay_t60_relative_error": 0.35,
}

# These three points are intentionally broad and nested. They are frozen before
# any V4 development waveform is decoded by the evaluator.
CAPACITIES = (
    {
        "id": "compact",
        "mode_count": 64,
        "long_residual_rank": 24,
        "spectral_residual_bins": 16,
        "transient_residual_rank": 12,
    },
    {
        "id": "balanced",
        "mode_count": 192,
        "long_residual_rank": 48,
        "spectral_residual_bins": 32,
        "transient_residual_rank": 18,
    },
    {
        "id": "extended",
        "mode_count": 384,
        "long_residual_rank": 64,
        "spectral_residual_bins": 64,
        "transient_residual_rank": 24,
    },
)

MODAL_MINIMUM_SEPARATION_HZ = 6.0
MODAL_DAMPING_MINIMUM_PER_SECOND = 0.25
MODAL_DAMPING_MAXIMUM_PER_SECOND = 200.0
MODAL_STFT_SAMPLES = 8_192
MODAL_STFT_HOP = 1_024
MODAL_DAMPING_FLOOR_RATIO = 1.0e-3
MODAL_DAMPING_MAXIMUM_SECONDS = 2.5

LONG_FRAME_SAMPLES = 4_096
LONG_FRAME_HOP = 2_048
TRANSIENT_FRAME_SAMPLES = 512
TRANSIENT_FRAME_HOP = 256
TRANSIENT_SAMPLES = 16_384
RANDOMIZED_SVD_OVERSAMPLE = 16
RANDOMIZED_SVD_POWER_ITERATIONS = 2
RANDOMIZED_SVD_SEED = 20_260_830

MAXIMUM_SHARED_DECODER_BYTES = 4 * 1024 * 1024
MAXIMUM_CONTACT_RECORD_BYTES = 64 * 1024
MINIMUM_BEATEN_ENDPOINTS = 4
MAXIMUM_NORMALIZED_ERROR_RATIO = 0.8

REQUIRED_ENVIRONMENT = {
    "python": "3.11.15",
    "numpy": "1.26.4",
    "scipy": "1.11.4",
    "openblas_num_threads": "1",
    "omp_num_threads": "1",
    "mkl_num_threads": "1",
}

PARENT_OBJECTS = (
    {
        "id": "blue_bowl",
        "dataset_object_id": "6_Bowl",
        "profile": "realimpact-blue-bowl-canonical-contact-r3a-v1",
        "manifest_sha256": (
            "6ad5b42c96dcbc5ef901e66598eacb9e140630ebf8030864e12845863f16847b"
        ),
        "sample_count": 230_215,
        "report_sha256": (
            "4d364a28abd6c1fb8c35b8e8e4c7548c20f27c2a0ee636511ba8d3df74e6b827"
        ),
        "metadata_sha256": (
            "11204934e4928bb75a159410732a4d4d6926ad55f4378fae6bb9100da66483bd"
        ),
        "contacts_sha256": (
            "1971f01a202cbcbe36d7ac7f16c92b6d559f06a432eb948ae782c1ae43a11ba3"
        ),
        "development_lineage": "r3a_v1_rejected_representation",
    },
    {
        "id": "large_swan",
        "dataset_object_id": "79_LargeSwanCeramic",
        "profile": "realimpact-large-swan-canonical-contact-r3a-v2-dac-oracle",
        "manifest_sha256": (
            "6821c49b428e37abf0903f89fc6a2bdddbdb3ad7b11dc82463c9122abaff0a4c"
        ),
        "sample_count": 209_213,
        "report_sha256": (
            "79dc70ba358b3085184f3ebe30f057d95bb961e0ff6691841a787944fca17f1d"
        ),
        "metadata_sha256": (
            "90ab7064f038af9a23bfec74a9efc94a8d60bd60aa3962d067ed0099314b63e2"
        ),
        "contacts_sha256": (
            "f0ce1461642f9cf20a6cc89d58479f4162cd8665881e605c109583fde9aecb8d"
        ),
        "development_lineage": "r3a_v2_inconclusive_resampling_control",
    },
    {
        "id": "plastic_bin",
        "dataset_object_id": "73_PlasticBin",
        "profile": "realimpact-plastic-bin-canonical-contact-r3a-v3-ndac-oracle",
        "manifest_sha256": (
            "e556ddbc350ae4c6a6fb90935c43c00c34bceb891f3d595adc6d331fddb3801f"
        ),
        "sample_count": 209_057,
        "report_sha256": (
            "b259994c3bd77cd524a0308e2304ab41c8835bd9567a5a41b8242e60e9bcd3fb"
        ),
        "metadata_sha256": (
            "42937a3a8ff78a1caa1ef2f87138b24aa0b12b2d947d21a5d5c3335436eda799"
        ),
        "contacts_sha256": (
            "ed1520041006db7a9059cd637adaa37ec3e1c1d6c2905b4a8c9ab6e8d88d168b"
        ),
        "development_lineage": "r3a_v3a_opened_invalid_codec_infrastructure",
    },
    {
        "id": "purple_scoop",
        "dataset_object_id": "23_PurpleScoop",
        "profile": "realimpact-purple-scoop-canonical-contact-r3a-v3b-ndac-oracle",
        "manifest_sha256": (
            "57dcc3b3f56e22582e7e73161812204d1f3d1b03bfbc38611cfa4711525d8959"
        ),
        "sample_count": 230_980,
        "report_sha256": (
            "7f2d7de4fe8a667de6ae7f11a5a5a342357f35443030dfa0017f053b56907164"
        ),
        "metadata_sha256": (
            "579441d80f7441d95f691f2dcfd63026b952e43a94907563d5b7de071ef14dc1"
        ),
        "contacts_sha256": (
            "2dcab65966fc7a09f9cab188eb644af34014e9ed25a2a7a42ddcb1a9fe060b3c"
        ),
        "development_lineage": "r3a_v3b_rejected_native_ndac",
    },
)


class V4Error(RuntimeError):
    """The frozen V4 development boundary was violated."""


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def json_ready(value: Any) -> Any:
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, dict):
        return {key: json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def load_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    payload = path.read_bytes()
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise V4Error(f"cannot parse {label}") from error
    if not isinstance(value, dict) or canonical_json(value) != payload:
        raise V4Error(f"{label} is not canonical JSON")
    return payload, value


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def require_external_directory(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_dir():
        raise V4Error(f"{label} must be an external directory")
    return resolved


def require_external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise V4Error(f"{label} must be an external file")
    return resolved


def prepare_output(root: Path, output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(root):
        raise V4Error("V4 output must stay outside the repository")
    if resolved.exists():
        raise V4Error("V4 output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def publish_output(output: Path, staging: Path) -> None:
    staging.rename(output)


def implementation_hashes(directory: Path) -> dict[str, str]:
    result = {}
    for key, filename in IMPLEMENTATION_FILES.items():
        path = directory / filename
        if not path.is_file():
            raise V4Error(f"missing V4 implementation file: {filename}")
        result[key] = sha256_file(path)
    return result


def validate_implementation(manifest: dict[str, Any], directory: Path) -> None:
    if manifest.get("implementation_sha256") != implementation_hashes(directory):
        raise V4Error("V4 implementation changed after preflight")


def validate_thread_environment() -> dict[str, str]:
    observed = {
        "openblas_num_threads": os.environ.get("OPENBLAS_NUM_THREADS", ""),
        "omp_num_threads": os.environ.get("OMP_NUM_THREADS", ""),
        "mkl_num_threads": os.environ.get("MKL_NUM_THREADS", ""),
    }
    expected = {
        key: REQUIRED_ENVIRONMENT[key]
        for key in ("openblas_num_threads", "omp_num_threads", "mkl_num_threads")
    }
    if observed != expected:
        raise V4Error(f"linear-algebra thread environment is not frozen: {observed}")
    return observed


def environment_identity() -> dict[str, Any]:
    threads = validate_thread_environment()
    observed = {
        "python": platform.python_version(),
        "python_implementation": platform.python_implementation(),
        "numpy": np.__version__,
        "scipy": scipy.__version__,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "byteorder": sys.byteorder,
        **threads,
    }
    for key in ("python", "numpy", "scipy"):
        if observed[key] != REQUIRED_ENVIRONMENT[key]:
            raise V4Error(f"frozen {key} version changed: {observed[key]}")
    if observed["byteorder"] != "little":
        raise V4Error("V4 requires a little-endian host")
    return observed


def validate_parent_directory(
    root: Path, directory: Path, expected: dict[str, Any]
) -> dict[str, Any]:
    resolved = require_external_directory(root, directory, expected["id"])
    report_path = resolved / "report.json"
    metadata_path = resolved / "contacts-metadata.json"
    contacts_path = resolved / "contacts.npy"
    for path in (report_path, metadata_path, contacts_path):
        if not path.is_file():
            raise V4Error(f"{expected['id']} is missing {path.name}")
    report_bytes, report = load_json(report_path, f"{expected['id']} report")
    metadata_bytes, metadata = load_json(metadata_path, f"{expected['id']} metadata")
    hashes = {
        "report": sha256_bytes(report_bytes),
        "metadata": sha256_bytes(metadata_bytes),
        "contacts": sha256_file(contacts_path),
    }
    required_hashes = {
        "report": expected["report_sha256"],
        "metadata": expected["metadata_sha256"],
        "contacts": expected["contacts_sha256"],
    }
    if hashes != required_hashes:
        raise V4Error(f"{expected['id']} exact lineage changed")
    if (
        metadata.get("profile") != expected["profile"]
        or metadata.get("manifest_sha256") != expected["manifest_sha256"]
        or metadata.get("sample_rate_hz") != SAMPLE_RATE_HZ
        or metadata.get("sample_count") != expected["sample_count"]
        or len(metadata.get("contacts", [])) != 4
        or [item.get("role") for item in metadata["contacts"]]
        != ["fit", "fit", "fit", "representation_development"]
        or [item.get("row_index") for item in metadata["contacts"]]
        != [7, 607, 1207, 1807]
        or metadata.get("sealed_contact_commitment", {}).get("row_index") != 2407
        or metadata["sealed_contact_commitment"].get("waveform_access")
        != "sealed_not_decompressed"
        or report.get("sealed_waveform_samples_decoded") != 0
    ):
        raise V4Error(f"{expected['id']} role or seal changed")
    with contacts_path.open("rb") as handle:
        version = np.lib.format.read_magic(handle)
        if version == (1, 0):
            shape, fortran, dtype = np.lib.format.read_array_header_1_0(handle)
        elif version == (2, 0):
            shape, fortran, dtype = np.lib.format.read_array_header_2_0(handle)
        else:
            raise V4Error(f"{expected['id']} unsupported NPY version: {version}")
    if shape != (4, expected["sample_count"]) or fortran or dtype != np.dtype("<f4"):
        raise V4Error(f"{expected['id']} NPY header changed")
    return {
        **{key: value for key, value in expected.items()},
        "directory": str(resolved),
        "report_sha256": hashes["report"],
        "metadata_sha256": hashes["metadata"],
        "contacts_sha256": hashes["contacts"],
        "roles": ["fit", "fit", "fit", "representation_development"],
        "sealed_row": 2407,
        "sealed_waveform_samples_decoded": 0,
    }


def validate_manifest(manifest: dict[str, Any]) -> None:
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("study_id") != STUDY_ID
        or manifest.get("revision") != REVISION
        or manifest.get("sample_rate_hz") != SAMPLE_RATE_HZ
        or manifest.get("analysis_samples") != ANALYSIS_SAMPLES
        or manifest.get("capacities") != list(CAPACITIES)
        or manifest.get("absolute_thresholds") != ABSOLUTE_THRESHOLDS
        or manifest.get("primary_endpoints") != list(PRIMARY_ENDPOINTS)
        or manifest.get("development_waveforms_open_before_v4") is not True
        or manifest.get("new_object_or_sealed_waveform_access_authorized") is not False
    ):
        raise V4Error("V4 manifest changed")
    objects = manifest.get("objects")
    if not isinstance(objects, list) or [item.get("id") for item in objects] != [
        item["id"] for item in PARENT_OBJECTS
    ]:
        raise V4Error("V4 object roster changed")


def align_peak(value: np.ndarray) -> np.ndarray:
    if value.ndim != 1 or value.size < ANALYSIS_SAMPLES:
        raise V4Error("contact waveform is too short")
    if not np.all(np.isfinite(value)):
        raise V4Error("contact waveform contains non-finite samples")
    peak = int(np.argmax(np.abs(value)))
    shift = PEAK_ALIGNMENT_SAMPLE - peak
    aligned = np.zeros(value.size, dtype=np.float64)
    if shift >= 0:
        aligned[shift:] = value[: value.size - shift]
    else:
        aligned[:shift] = value[-shift:]
    return aligned[:ANALYSIS_SAMPLES]


def normalize_fit_group(values: np.ndarray) -> tuple[np.ndarray, float]:
    if values.ndim != 2 or values.shape[0] < FIT_CONTACTS_PER_OBJECT:
        raise V4Error("fit group shape is invalid")
    aligned = np.stack([align_peak(row) for row in values])
    fit_peak = float(np.max(np.abs(aligned[:FIT_CONTACTS_PER_OBJECT])))
    if not math.isfinite(fit_peak) or fit_peak <= 0.0:
        raise V4Error("fit-only peak is invalid")
    scale = 0.92 / fit_peak
    return np.ascontiguousarray(aligned * scale, dtype=np.float64), scale


def estimate_object_poles(fit_values: np.ndarray, maximum_modes: int) -> np.ndarray:
    if fit_values.shape != (FIT_CONTACTS_PER_OBJECT, ANALYSIS_SAMPLES):
        raise V4Error("pole estimator requires exactly three fit contacts")
    tail = fit_values[:, PEAK_ALIGNMENT_SAMPLE:]
    window = np.hanning(tail.shape[1])
    spectra = np.abs(np.fft.rfft(tail * window[None, :], axis=1))
    summary = np.sqrt(np.mean(np.square(spectra), axis=0))
    frequencies = np.fft.rfftfreq(tail.shape[1], 1.0 / SAMPLE_RATE_HZ)
    eligible = (frequencies >= EVALUATION_MIN_HZ) & (frequencies <= EVALUATION_MAX_HZ)
    separation_bins = max(
        1,
        math.ceil(MODAL_MINIMUM_SEPARATION_HZ / (frequencies[1] - frequencies[0])),
    )
    peaks, _ = signal.find_peaks(summary, distance=separation_bins)
    peaks = peaks[eligible[peaks]]
    order = peaks[np.argsort(-summary[peaks], kind="stable")]
    if order.size < maximum_modes:
        raise V4Error(
            f"object exposes only {order.size} separated peaks for {maximum_modes} modes"
        )
    selected = order[:maximum_modes]

    _, times, stft = signal.stft(
        tail,
        fs=SAMPLE_RATE_HZ,
        window="hann",
        nperseg=MODAL_STFT_SAMPLES,
        noverlap=MODAL_STFT_SAMPLES - MODAL_STFT_HOP,
        boundary=None,
        padded=False,
        axis=-1,
    )
    stft_frequencies = np.fft.rfftfreq(MODAL_STFT_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    magnitude = np.sqrt(np.mean(np.square(np.abs(stft)), axis=0))
    poles = []
    for selection_rank, peak in enumerate(selected):
        frequency = float(frequencies[peak])
        bin_index = int(np.argmin(np.abs(stft_frequencies - frequency)))
        envelope = magnitude[bin_index]
        maximum = max(float(np.max(envelope)), 1.0e-30)
        start = int(np.argmax(envelope))
        usable = np.arange(envelope.size) >= start
        usable &= times <= min(float(times[-1]), MODAL_DAMPING_MAXIMUM_SECONDS)
        usable &= envelope >= maximum * MODAL_DAMPING_FLOOR_RATIO
        if np.count_nonzero(usable) >= 6:
            slope = np.polyfit(
                times[usable], np.log(np.maximum(envelope[usable], 1.0e-30)), 1
            )[0]
            damping = float(
                np.clip(
                    -slope,
                    MODAL_DAMPING_MINIMUM_PER_SECOND,
                    MODAL_DAMPING_MAXIMUM_PER_SECOND,
                )
            )
        else:
            damping = 20.0
        poles.append((frequency, damping, float(selection_rank)))
    result = np.asarray(poles, dtype=np.float64)
    if not np.all(np.isfinite(result)):
        raise V4Error("pole estimator produced non-finite values")
    return result


def selected_poles(poles: np.ndarray, count: int) -> np.ndarray:
    selected = poles[poles[:, 2] < count]
    order = np.argsort(selected[:, 0], kind="stable")
    return np.ascontiguousarray(selected[order], dtype=np.float64)


def modal_gains(value: np.ndarray, poles: np.ndarray) -> np.ndarray:
    residual = np.asarray(value, dtype=np.float64).copy()
    gains = np.zeros((poles.shape[0], 2), dtype=np.float64)
    time = np.arange(ANALYSIS_SAMPLES - PEAK_ALIGNMENT_SAMPLE, dtype=np.float64)
    time /= SAMPLE_RATE_HZ
    rank_order = np.argsort(poles[:, 2], kind="stable")
    for index in rank_order:
        frequency, damping, _ = poles[index]
        envelope = np.exp(-damping * time)
        phase = 2.0 * math.pi * frequency * time
        cosine = envelope * np.cos(phase)
        sine = envelope * np.sin(phase)
        view = residual[PEAK_ALIGNMENT_SAMPLE:]
        gram = np.asarray(
            [
                [np.dot(cosine, cosine), np.dot(cosine, sine)],
                [np.dot(cosine, sine), np.dot(sine, sine)],
            ],
            dtype=np.float64,
        )
        rhs = np.asarray([np.dot(cosine, view), np.dot(sine, view)])
        gain = np.linalg.solve(gram + np.eye(2) * 1.0e-12, rhs)
        gains[index] = gain
        view -= gain[0] * cosine + gain[1] * sine
    return gains


def synthesize_modal(poles: np.ndarray, gains: np.ndarray) -> np.ndarray:
    if poles.shape != (gains.shape[0], 3) or gains.shape[1] != 2:
        raise V4Error("modal record shape is invalid")
    result = np.zeros(ANALYSIS_SAMPLES, dtype=np.float64)
    time = np.arange(ANALYSIS_SAMPLES - PEAK_ALIGNMENT_SAMPLE, dtype=np.float64)
    time /= SAMPLE_RATE_HZ
    for (frequency, damping, _), (cos_gain, sin_gain) in zip(poles, gains, strict=True):
        envelope = np.exp(-damping * time)
        phase = 2.0 * math.pi * frequency * time
        result[PEAK_ALIGNMENT_SAMPLE:] += envelope * (
            cos_gain * np.cos(phase) + sin_gain * np.sin(phase)
        )
    return result


def analysis_frames(
    value: np.ndarray, frame_samples: int, hop_samples: int
) -> np.ndarray:
    if value.ndim != 1 or frame_samples <= hop_samples or frame_samples % 2:
        raise V4Error("frame analysis arguments are invalid")
    pad = frame_samples // 2
    padded = np.pad(np.asarray(value, dtype=np.float64), (pad, pad))
    remainder = (padded.size - frame_samples) % hop_samples
    if remainder:
        padded = np.pad(padded, (0, hop_samples - remainder))
    starts = range(0, padded.size - frame_samples + 1, hop_samples)
    window = np.sqrt(np.hanning(frame_samples))
    return np.stack(
        [padded[start : start + frame_samples] * window for start in starts]
    )


def synthesis_frames(
    frames: np.ndarray, sample_count: int, frame_samples: int, hop_samples: int
) -> np.ndarray:
    if frames.ndim != 2 or frames.shape[1] != frame_samples:
        raise V4Error("frame synthesis shape is invalid")
    pad = frame_samples // 2
    total = (frames.shape[0] - 1) * hop_samples + frame_samples
    result = np.zeros(total, dtype=np.float64)
    weight = np.zeros(total, dtype=np.float64)
    window = np.sqrt(np.hanning(frame_samples))
    for index, frame in enumerate(frames):
        start = index * hop_samples
        result[start : start + frame_samples] += frame * window
        weight[start : start + frame_samples] += np.square(window)
    usable = weight > 1.0e-15
    result[usable] /= weight[usable]
    return np.ascontiguousarray(result[pad : pad + sample_count], dtype=np.float64)


def randomized_basis(frames: np.ndarray, rank: int, seed: int) -> np.ndarray:
    if frames.ndim != 2 or rank <= 0 or rank > min(frames.shape):
        raise V4Error("residual-basis rank is invalid")
    generator = np.random.Generator(np.random.PCG64(seed))
    width = min(rank + RANDOMIZED_SVD_OVERSAMPLE, min(frames.shape))
    omega = generator.standard_normal((frames.shape[1], width))
    projected = frames @ omega
    for _ in range(RANDOMIZED_SVD_POWER_ITERATIONS):
        projected = frames @ (frames.T @ projected)
    q, _ = np.linalg.qr(projected, mode="reduced")
    compressed = q.T @ frames
    _, _, right = np.linalg.svd(compressed, full_matrices=False)
    basis = np.ascontiguousarray(right[:rank], dtype=np.float64)
    for row in basis:
        pivot = int(np.argmax(np.abs(row)))
        if row[pivot] < 0.0:
            row *= -1.0
    if not np.all(np.isfinite(basis)):
        raise V4Error("residual basis contains non-finite values")
    return basis


def normalized_training_frames(
    residuals: Iterable[np.ndarray], frame_samples: int, hop_samples: int
) -> np.ndarray:
    rows = []
    for residual in residuals:
        scale = max(rms(residual), 1.0e-15)
        rows.append(analysis_frames(residual / scale, frame_samples, hop_samples))
    return np.ascontiguousarray(np.concatenate(rows, axis=0), dtype=np.float64)


def project_basis(
    value: np.ndarray, basis: np.ndarray, frame_samples: int, hop_samples: int
) -> tuple[np.ndarray, np.ndarray]:
    frames = analysis_frames(value, frame_samples, hop_samples)
    coefficients = frames @ basis.T
    reconstructed = synthesis_frames(
        coefficients @ basis, value.size, frame_samples, hop_samples
    )
    return reconstructed, np.ascontiguousarray(coefficients, dtype=np.float64)


def select_spectral_bins(residuals: Iterable[np.ndarray], count: int) -> np.ndarray:
    energy = None
    for residual in residuals:
        scale = max(rms(residual), 1.0e-15)
        frames = analysis_frames(residual / scale, LONG_FRAME_SAMPLES, LONG_FRAME_HOP)
        current = np.sum(np.square(np.abs(np.fft.rfft(frames, axis=1))), axis=0)
        energy = current if energy is None else energy + current
    if energy is None:
        raise V4Error("spectral-bin learner received no fit residuals")
    frequencies = np.fft.rfftfreq(LONG_FRAME_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    eligible = np.flatnonzero(
        (frequencies >= EVALUATION_MIN_HZ) & (frequencies <= EVALUATION_MAX_HZ)
    )
    if count <= 0 or count > eligible.size:
        raise V4Error("spectral-bin count is invalid")
    ranked = eligible[np.argsort(-energy[eligible], kind="stable")[:count]]
    return np.sort(ranked).astype(np.int64)


def project_spectral_bins(
    value: np.ndarray, selected_bins: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    frames = analysis_frames(value, LONG_FRAME_SAMPLES, LONG_FRAME_HOP)
    spectrum = np.fft.rfft(frames, axis=1)
    coefficients = spectrum[:, selected_bins]
    selected = np.zeros_like(spectrum)
    selected[:, selected_bins] = coefficients
    reconstructed_frames = np.fft.irfft(selected, n=LONG_FRAME_SAMPLES, axis=1)
    reconstructed = synthesis_frames(
        reconstructed_frames,
        value.size,
        LONG_FRAME_SAMPLES,
        LONG_FRAME_HOP,
    )
    return reconstructed, np.ascontiguousarray(coefficients, dtype=np.complex128)


def cook_contact(
    value: np.ndarray,
    poles: np.ndarray,
    long_basis: np.ndarray,
    spectral_bins: np.ndarray,
    transient_basis: np.ndarray,
) -> tuple[np.ndarray, dict[str, Any]]:
    gains = modal_gains(value, poles)
    modal = synthesize_modal(poles, gains)
    residual = value - modal
    long_value, long_coefficients = project_basis(
        residual, long_basis, LONG_FRAME_SAMPLES, LONG_FRAME_HOP
    )
    remaining = residual - long_value
    spectral_value, spectral_coefficients = project_spectral_bins(
        remaining, spectral_bins
    )
    remaining -= spectral_value
    transient_value, transient_coefficients = project_basis(
        remaining[:TRANSIENT_SAMPLES],
        transient_basis,
        TRANSIENT_FRAME_SAMPLES,
        TRANSIENT_FRAME_HOP,
    )
    candidate = modal + long_value + spectral_value
    candidate[:TRANSIENT_SAMPLES] += transient_value
    denominator = max(float(np.dot(candidate, candidate)), 1.0e-30)
    output_gain = float(np.dot(value, candidate) / denominator)
    candidate *= output_gain
    encoded_float_count = (
        poles.shape[0] * 2
        + long_coefficients.size
        + spectral_coefficients.size * 2
        + transient_coefficients.size
        + 1
    )
    record_bytes = 4 * encoded_float_count
    return np.ascontiguousarray(candidate, dtype=np.float64), {
        "modal_gains": gains,
        "long_coefficients": long_coefficients,
        "spectral_coefficients": spectral_coefficients,
        "transient_coefficients": transient_coefficients,
        "output_gain": output_gain,
        "encoded_float32_count": encoded_float_count,
        "encoded_bytes": record_bytes,
    }


def rms(value: np.ndarray) -> float:
    return float(np.sqrt(np.mean(np.square(value))))


def level_error(target: np.ndarray, candidate: np.ndarray) -> float:
    return abs(
        20.0 * math.log10(max(rms(candidate), 1.0e-15) / max(rms(target), 1.0e-15))
    )


def log_spectrum_error(target: np.ndarray, candidate: np.ndarray) -> float:
    errors = []
    for length in (2_048, 8_192, 32_768):
        hop = length // 2
        window = np.hanning(length)
        target_frames = np.stack(
            [
                target[start : start + length] * window
                for start in range(0, target.size - length + 1, hop)
            ]
        )
        candidate_frames = np.stack(
            [
                candidate[start : start + length] * window
                for start in range(0, candidate.size - length + 1, hop)
            ]
        )
        frequencies = np.fft.rfftfreq(length, 1.0 / SAMPLE_RATE_HZ)
        selected = (frequencies >= EVALUATION_MIN_HZ) & (
            frequencies <= EVALUATION_MAX_HZ
        )
        target_magnitude = np.abs(np.fft.rfft(target_frames, axis=1))[:, selected]
        candidate_magnitude = np.abs(np.fft.rfft(candidate_frames, axis=1))[:, selected]
        floor = max(
            float(np.max(target_magnitude)) * 10.0 ** (SPECTRAL_FLOOR_DB / 20.0),
            1.0e-15,
        )
        target_db = 20.0 * np.log10(np.maximum(target_magnitude, floor))
        candidate_db = 20.0 * np.log10(np.maximum(candidate_magnitude, floor))
        difference = candidate_db - target_db
        difference -= np.mean(difference)
        errors.append(float(np.sqrt(np.mean(np.square(difference)))))
    return float(np.mean(errors))


def smoothed_envelope(value: np.ndarray) -> np.ndarray:
    envelope = np.abs(signal.hilbert(value))
    width = SAMPLE_RATE_HZ // 200
    return np.convolve(envelope, np.ones(width) / width, mode="same")


def envelope_error(target: np.ndarray, candidate: np.ndarray) -> float:
    target_envelope = smoothed_envelope(target)
    candidate_envelope = smoothed_envelope(candidate)
    scale = max(float(np.max(target_envelope)), 1.0e-15)
    return float(
        np.sqrt(np.mean(np.square((candidate_envelope - target_envelope) / scale)))
    )


def dominant_frequencies(value: np.ndarray, count: int = 12) -> np.ndarray:
    magnitude = np.abs(np.fft.rfft(value * np.hanning(value.size)))
    frequencies = np.fft.rfftfreq(value.size, 1.0 / SAMPLE_RATE_HZ)
    eligible = (frequencies >= EVALUATION_MIN_HZ) & (frequencies <= EVALUATION_MAX_HZ)
    peaks, _ = signal.find_peaks(
        magnitude, distance=max(1, value.size // SAMPLE_RATE_HZ * 8)
    )
    peaks = peaks[eligible[peaks]]
    if peaks.size < count:
        peaks = np.flatnonzero(eligible)
    selected = peaks[np.argsort(-magnitude[peaks], kind="stable")[:count]]
    return np.sort(frequencies[selected])


def modal_frequency_error(target: np.ndarray, candidate: np.ndarray) -> float:
    target_frequencies = dominant_frequencies(target)
    candidate_frequencies = dominant_frequencies(candidate)
    errors = []
    remaining = list(candidate_frequencies)
    for frequency in target_frequencies:
        index = int(np.argmin(np.abs(np.asarray(remaining) - frequency)))
        matched = remaining.pop(index)
        errors.append(abs(1_200.0 * math.log2(matched / frequency)))
    return float(np.median(errors))


def decay_t60(value: np.ndarray) -> float:
    energy = np.square(value[PEAK_ALIGNMENT_SAMPLE:])
    schroeder = np.cumsum(energy[::-1])[::-1]
    if schroeder[0] <= 0.0:
        return 0.0
    db = 10.0 * np.log10(np.maximum(schroeder / schroeder[0], 1.0e-12))
    times = np.arange(db.size) / SAMPLE_RATE_HZ
    usable = (db <= -5.0) & (db >= -35.0)
    if np.count_nonzero(usable) < 16:
        return float(value.size / SAMPLE_RATE_HZ)
    slope = np.polyfit(times[usable], db[usable], 1)[0]
    return (
        float(-60.0 / slope) if slope < -1.0e-9 else float(value.size / SAMPLE_RATE_HZ)
    )


def metrics(target: np.ndarray, candidate: np.ndarray) -> dict[str, float]:
    target_t60 = decay_t60(target)
    candidate_t60 = decay_t60(candidate)
    return {
        "absolute_rms_level_error_db": level_error(target, candidate),
        "gain_matched_multiresolution_log_spectrum_rmse_db": log_spectrum_error(
            target, candidate
        ),
        "normalized_envelope_rmse": envelope_error(target, candidate),
        "modal_frequency_median_error_cents": modal_frequency_error(target, candidate),
        "decay_t60_relative_error": abs(candidate_t60 - target_t60)
        / max(target_t60, 1.0e-9),
    }


def candidate_gate(
    candidate: dict[str, float], baseline: dict[str, float]
) -> dict[str, Any]:
    beaten = {
        endpoint: candidate[endpoint] < baseline[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    }
    absolute = {
        endpoint: candidate[endpoint] <= ABSOLUTE_THRESHOLDS[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    }
    nonregression = {
        endpoint: beaten[endpoint]
        or candidate[endpoint] <= 0.25 * ABSOLUTE_THRESHOLDS[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    }
    candidate_sum = sum(
        candidate[endpoint] / ABSOLUTE_THRESHOLDS[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    )
    baseline_sum = sum(
        baseline[endpoint] / ABSOLUTE_THRESHOLDS[endpoint]
        for endpoint in PRIMARY_ENDPOINTS
    )
    passed = (
        sum(beaten.values()) >= MINIMUM_BEATEN_ENDPOINTS
        and all(absolute.values())
        and all(nonregression.values())
        and candidate_sum < MAXIMUM_NORMALIZED_ERROR_RATIO * baseline_sum
    )
    return {
        "passed": passed,
        "strictly_beats_nearest_fit_by_endpoint": beaten,
        "strictly_better_endpoint_count": sum(beaten.values()),
        "within_absolute_threshold_on_every_endpoint": absolute,
        "nonregressing_preserved_endpoint": nonregression,
        "normalized_candidate_error_sum": candidate_sum,
        "normalized_baseline_error_sum": baseline_sum,
        "required_normalized_improvement_ratio": MAXIMUM_NORMALIZED_ERROR_RATIO,
    }


def identity_gate(value: dict[str, float]) -> dict[str, Any]:
    exact = {endpoint: value[endpoint] == 0.0 for endpoint in PRIMARY_ENDPOINTS}
    return {"passed": all(exact.values()), "exact_zero": exact}


def pcm16_wav(value: np.ndarray) -> bytes:
    peak = max(float(np.max(np.abs(value))), 1.0e-15)
    samples = np.round(np.clip(value / peak * 0.95, -1.0, 1.0) * 32_767.0).astype("<i2")
    output = io.BytesIO()
    with wave.open(output, "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(SAMPLE_RATE_HZ)
        handle.writeframes(samples.tobytes())
    return output.getvalue()


def save_npy(path: Path, value: np.ndarray) -> dict[str, Any]:
    with path.open("wb") as handle:
        np.save(handle, np.ascontiguousarray(value), allow_pickle=False)
    return {
        "path": path.name,
        "shape": list(value.shape),
        "dtype": str(value.dtype),
        "bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def load_bound_npy(directory: Path, descriptor: dict[str, Any]) -> np.ndarray:
    path = directory / descriptor["path"]
    if (
        not path.is_file()
        or path.stat().st_size != descriptor["bytes"]
        or sha256_file(path) != descriptor["sha256"]
    ):
        raise V4Error(f"model artifact changed: {descriptor['path']}")
    with path.open("rb") as handle:
        value = np.load(handle, allow_pickle=False)
    if (
        list(value.shape) != descriptor["shape"]
        or str(value.dtype) != descriptor["dtype"]
    ):
        raise V4Error(f"model artifact shape changed: {descriptor['path']}")
    return np.ascontiguousarray(value, dtype=np.float64)
