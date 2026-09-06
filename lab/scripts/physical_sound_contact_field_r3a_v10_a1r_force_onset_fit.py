#!/usr/bin/env python3
"""Fit the frozen V9 representation to ObjectFolder object 51 using force onset."""

from __future__ import annotations

import argparse
import json
import math
import shutil
import tarfile
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_contact_field_r3a_v10_a1r_force_source_inventory as source
import physical_sound_contact_field_r3a_v10_beer_glass_real_fit as representation

endpoint = representation.endpoint
FitError = representation.FitError

STUDY_ID = "physical-sound-contact-field-r3a-v10-a1r-force-onset-fit"
REVISION = "object51-force-onset-v9-representation-fit-v1"
MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v10-a1r-force-fit.manifest.v1"
)
MODEL_SCHEMA = "nextengine.experimental-physical-sound-r3a-v10-a1r-force-fit.model.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v10-a1r-force-fit.report.v1"

INVENTORY_MANIFEST_SHA256 = (
    "3041c19d07f78fc9bdfd092ced72e07b5e795b235a286a23ede68108556b6ed5"
)
INVENTORY_IMPLEMENTATION_SHA256 = (
    "f048bb9f9e17351822336f7d4d78b7a293e2e6e5154bb13c987bf64863d6a38d"
)
REPRESENTATION_IMPLEMENTATION_SHA256 = (
    "e792bee5d84e82452274e33cc1be2a3cf6999ec3fe191e35d01a91b5b4e38ad6"
)
ENDPOINT_IMPLEMENTATION_SHA256 = (
    "f8b7b2ea09a76ac4f724222c0fa3cb1237cfb4b0ff9a373d631c2dc151658df0"
)
PROTOCOL_SHA256 = (
    "0281a76ce6f18ab9033cabc4097f8d83fd579441bfe1e381f8307ca010a505dc"
)
PROTOCOL_PATH = (
    "docs/development/"
    "physical-sound-r3a-v10-a1r-object51-force-onset-fit-protocol-2026-08-31.md"
)

OBJECT_ID = "51"
FIT_CONTACT_IDS = (27, 15, 4, 3)
DEVELOPMENT_CONTACT_IDS = (9,)
SEALED_CONTACT_IDS = (18,)

SAMPLE_RATE_HZ = representation.SAMPLE_RATE_HZ
SOURCE_SAMPLES = representation.SOURCE_SAMPLES
ANALYSIS_SAMPLES = representation.ANALYSIS_SAMPLES
BASELINE_SAMPLES = representation.BASELINE_SAMPLES
ALIGNMENT_SAMPLE = representation.ALIGNMENT_SAMPLE
EXPECTED_CHANNEL_SAMPLES = len(FIT_CONTACT_IDS) * SOURCE_SAMPLES
EXPECTED_TOTAL_DECODED_SAMPLES = 2 * EXPECTED_CHANNEL_SAMPLES
EXPECTED_RETAINED_SAMPLES = len(FIT_CONTACT_IDS) * ANALYSIS_SAMPLES


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def validate_implementation(root: Path) -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    files = {
        "fit": Path(__file__).resolve(),
        "inventory": directory
        / "physical_sound_contact_field_r3a_v10_a1r_force_source_inventory.py",
        "representation": directory
        / "physical_sound_contact_field_r3a_v10_beer_glass_real_fit.py",
        "endpoints": directory / "physical_sound_contact_field_r3a_v4_common.py",
        "protocol": root / PROTOCOL_PATH,
    }
    observed = {
        name: representation.sha256_file(path) for name, path in files.items()
    }
    expected = {
        "inventory": INVENTORY_IMPLEMENTATION_SHA256,
        "representation": REPRESENTATION_IMPLEMENTATION_SHA256,
        "endpoints": ENDPOINT_IMPLEMENTATION_SHA256,
        "protocol": PROTOCOL_SHA256,
    }
    for name, value in expected.items():
        if observed[name] != value:
            raise FitError(f"A1R frozen {name} hash changed")
    frozen = {
        "mode_count": 64,
        "minimum_mode_separation_hz": 8.0,
        "noise_band_count": 96,
        "noise_loop_samples": 16_384,
        "noise_seed": 20_260_831,
        "residual_frame_samples": 2_048,
        "residual_frame_hop": 128,
        "residual_dct_coefficients": 8,
        "transient_samples": 2_048,
        "expected_contact_record_bytes": 5_904,
    }
    actual = {
        "mode_count": representation.MODE_COUNT,
        "minimum_mode_separation_hz": representation.MINIMUM_MODE_SEPARATION_HZ,
        "noise_band_count": representation.NOISE_BAND_COUNT,
        "noise_loop_samples": representation.NOISE_LOOP_SAMPLES,
        "noise_seed": representation.NOISE_SEED,
        "residual_frame_samples": representation.RESIDUAL_FRAME_SAMPLES,
        "residual_frame_hop": representation.RESIDUAL_FRAME_HOP,
        "residual_dct_coefficients": representation.RESIDUAL_DCT_COEFFICIENTS,
        "transient_samples": representation.TRANSIENT_SAMPLES,
        "expected_contact_record_bytes": representation.EXPECTED_CONTACT_RECORD_BYTES,
    }
    if actual != frozen:
        raise FitError("A1R inherited representation constants changed")
    return observed


def validate_inventory_manifest(root: Path, path: Path) -> dict[str, Any]:
    resolved = representation.require_external_file(root, path, "A1R inventory manifest")
    payload = resolved.read_bytes()
    if representation.sha256_bytes(payload) != INVENTORY_MANIFEST_SHA256:
        raise FitError("A1R inventory manifest hash changed")
    try:
        value = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise FitError("cannot parse A1R inventory manifest") from error
    if representation.canonical_json(value) != payload:
        raise FitError("A1R inventory manifest is not canonical")
    if (
        value.get("schema") != source.MANIFEST_SCHEMA
        or value.get("status") != "FrozenZeroDecodeForceSourceAndRoles"
        or value.get("next_authorized_role") != "fit"
        or value.get("implementation_sha256") != INVENTORY_IMPLEMENTATION_SHA256
    ):
        raise FitError("A1R inventory authority changed")
    if value.get("role_selection") != {
        "basis": "first_six_complete_raw_contacts_in_archive_order_before_pcm_decode",
        "benchmark_split_role": "absent",
        "fit": list(FIT_CONTACT_IDS),
        "development": list(DEVELOPMENT_CONTACT_IDS),
        "sealed": list(SEALED_CONTACT_IDS),
    }:
        raise FitError("A1R inventory roles changed")
    counters = (
        "microphone_sample_values_decoded",
        "force_sample_values_decoded",
        "fit_microphone_sample_values_decoded",
        "fit_force_sample_values_decoded",
        "development_microphone_sample_values_decoded",
        "development_force_sample_values_decoded",
        "sealed_microphone_sample_values_decoded",
        "sealed_force_sample_values_decoded",
    )
    if any(value.get(name) != 0 for name in counters):
        raise FitError("A1R inventory already decoded PCM samples")
    observed_roles = [
        (int(item["contact_id"]), item["role"], int(item["archive_order"]))
        for item in value.get("contacts", [])
    ]
    expected_roles = [
        (int(item["contact_id"]), item["role"], int(item["archive_order"]))
        for item in source.CONTACTS
    ]
    if observed_roles != expected_roles:
        raise FitError("A1R inventory contact order changed")
    for item in value["contacts"]:
        contact_id = int(item["contact_id"])
        observed = {
            member["path"]: (member["bytes"], member["sha256"])
            for member in item["raw_members"]
        }
        expected = {
            name: commitment
            for name, commitment in source.EXPECTED_RAW_COMMITMENTS.items()
            if name.startswith(f"{OBJECT_ID}/audio/{contact_id}/")
        }
        if observed != expected:
            raise FitError(f"A1R raw commitment changed for contact {contact_id}")
    return value


def validate_sources(
    root: Path,
    prefix: Path,
    page: Path,
    audio: Path,
    contacts: Path,
    points: Path,
    split: Path,
    scale: Path,
) -> Path:
    try:
        raw = source.require_exact_file(
            root,
            prefix,
            "A1R raw prefix",
            source.RAW_PREFIX_BYTES,
            source.RAW_PREFIX_SHA256,
        )
        official = source.require_exact_file(
            root,
            page,
            "A1R official page",
            source.OFFICIAL_PAGE_BYTES,
            source.OFFICIAL_PAGE_SHA256,
        )
        source.validate_official_page(official)
        arguments = {
            "audio": audio,
            "contacts": contacts,
            "point_cloud": points,
            "split": split,
            "scale": scale,
        }
        for name, argument in arguments.items():
            descriptor = source.compact.SOURCES[name]
            source.require_exact_file(
                root,
                argument,
                f"A1R compact {name}",
                descriptor["bytes"],
                descriptor["sha256"],
            )
    except source.InventoryError as error:
        raise FitError(str(error)) from error
    return raw


def fit_commitments(manifest: dict[str, Any]) -> dict[str, tuple[int, str]]:
    result: dict[str, tuple[int, str]] = {}
    for item in manifest["contacts"]:
        if item["role"] != "fit":
            continue
        contact_id = int(item["contact_id"])
        for member in item["raw_members"]:
            if member["path"].endswith(("/mic.wav", "/Force.wav")):
                result[member["path"]] = (member["bytes"], member["sha256"])
        expected = {
            f"{OBJECT_ID}/audio/{contact_id}/mic.wav",
            f"{OBJECT_ID}/audio/{contact_id}/Force.wav",
        }
        if not expected.issubset(result):
            raise FitError(f"A1R fit pair commitment missing for {contact_id}")
    if len(result) != 2 * len(FIT_CONTACT_IDS):
        raise FitError("A1R fit pair commitment count changed")
    return result


def extract_fit_pairs(
    prefix: Path, commitments: dict[str, tuple[int, str]]
) -> dict[int, dict[str, bytes]]:
    selected = set(commitments)
    found: dict[int, dict[str, bytes]] = {}
    try:
        with prefix.open("rb") as raw, tarfile.open(fileobj=raw, mode="r|gz") as archive:
            for member in archive:
                if member.name not in selected:
                    continue
                if not member.isfile():
                    raise FitError(f"A1R fit member is not a file: {member.name}")
                stream = archive.extractfile(member)
                if stream is None:
                    raise FitError(f"cannot read A1R fit member: {member.name}")
                payload = stream.read()
                expected_bytes, expected_sha256 = commitments[member.name]
                if (
                    len(payload) != expected_bytes
                    or representation.sha256_bytes(payload) != expected_sha256
                ):
                    raise FitError(f"A1R fit member changed: {member.name}")
                try:
                    source.parse_pcm_header(payload[:64], len(payload))
                except source.InventoryError as error:
                    raise FitError(str(error)) from error
                parts = member.name.split("/")
                contact_id = int(parts[2])
                channel = "force" if parts[3] == "Force.wav" else "microphone"
                if channel in found.setdefault(contact_id, {}):
                    raise FitError(f"duplicate A1R {channel}: {contact_id}")
                found[contact_id][channel] = payload
                if sum(len(pair) for pair in found.values()) == len(selected):
                    break
    except (tarfile.TarError, EOFError, OSError) as error:
        raise FitError("cannot extract A1R fit pairs") from error
    if tuple(found) != FIT_CONTACT_IDS or any(set(pair) != {"microphone", "force"} for pair in found.values()):
        raise FitError("A1R fit pair set or archive order changed")
    return found


def decode_pcm16(payload: bytes) -> np.ndarray:
    try:
        source.parse_pcm_header(payload[:64], len(payload))
    except source.InventoryError as error:
        raise FitError(str(error)) from error
    values = np.frombuffer(payload, dtype="<i2", offset=44)
    if values.shape != (SOURCE_SAMPLES,):
        raise FitError("A1R PCM sample count changed")
    return values.astype(np.float64) / 32768.0


def force_onset_sample(value: np.ndarray) -> tuple[int, float, float, float, float]:
    if value.shape != (SOURCE_SAMPLES,) or not np.isfinite(value).all():
        raise FitError("A1R force signal shape or finiteness changed")
    absolute = np.abs(value)
    peak = float(np.max(absolute))
    baseline = absolute[:BASELINE_SAMPLES]
    median = float(np.median(baseline))
    mad = float(np.median(np.abs(baseline - median)))
    peak_threshold = 0.05 * peak
    noise_threshold = median + 10.0 * mad
    threshold = max(peak_threshold, noise_threshold)
    if not math.isfinite(threshold) or peak <= 0.0 or threshold <= 0.0:
        raise FitError("A1R force onset threshold is invalid")
    indices = np.flatnonzero(absolute >= threshold)
    if indices.size == 0:
        raise FitError("A1R force onset threshold was never reached")
    return int(indices[0]), threshold, peak_threshold, noise_threshold, peak


def preprocess_pair(
    microphone: np.ndarray, force: np.ndarray
) -> tuple[np.ndarray, np.ndarray, dict[str, Any]]:
    if microphone.shape != (SOURCE_SAMPLES,) or not np.isfinite(microphone).all():
        raise FitError("A1R microphone shape or finiteness changed")
    microphone_median = float(np.median(microphone[:BASELINE_SAMPLES]))
    force_median = float(np.median(force[:BASELINE_SAMPLES]))
    centered_microphone = microphone - microphone_median
    centered_force = force - force_median
    onset, threshold, peak_threshold, noise_threshold, peak = force_onset_sample(
        centered_force
    )
    shift = ALIGNMENT_SAMPLE - onset
    aligned_microphone = representation.shift_and_crop(centered_microphone, shift)
    aligned_force = representation.shift_and_crop(centered_force, shift)
    return aligned_microphone, aligned_force, {
        "force_onset_sample": onset,
        "alignment_shift_samples": shift,
        "microphone_baseline_median": microphone_median,
        "force_baseline_median": force_median,
        "force_absolute_peak": peak,
        "force_onset_threshold": threshold,
        "force_peak_threshold": peak_threshold,
        "force_noise_threshold": noise_threshold,
        "aligned_force_sha256": representation.array_sha256(aligned_force),
    }


def load_fit_targets(
    pairs: dict[int, dict[str, bytes]],
) -> tuple[np.ndarray, list[dict[str, Any]], float]:
    rows = []
    descriptors = []
    for contact_id in FIT_CONTACT_IDS:
        microphone = decode_pcm16(pairs[contact_id]["microphone"])
        force = decode_pcm16(pairs[contact_id]["force"])
        aligned, _, descriptor = preprocess_pair(microphone, force)
        descriptor["contact_id"] = contact_id
        rows.append(aligned)
        descriptors.append(descriptor)
    values = np.stack(rows)
    peak = float(np.max(np.abs(values)))
    if not math.isfinite(peak) or peak <= 0.0:
        raise FitError("A1R fit-only microphone peak is invalid")
    scale = 0.92 / peak
    return np.ascontiguousarray(values * scale), descriptors, scale


def estimate_poles(targets: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    if targets.shape != (len(FIT_CONTACT_IDS), ANALYSIS_SAMPLES):
        raise FitError("A1R pole estimator input shape changed")
    window = np.hanning(ANALYSIS_SAMPLES)
    spectra = np.abs(np.fft.rfft(targets * window[None, :], axis=1))
    summary = np.sqrt(np.mean(np.square(spectra), axis=0))
    frequencies = np.fft.rfftfreq(ANALYSIS_SAMPLES, 1.0 / SAMPLE_RATE_HZ)
    eligible = (frequencies >= endpoint.EVALUATION_MIN_HZ) & (
        frequencies <= endpoint.EVALUATION_MAX_HZ
    )
    separation_bins = max(
        1,
        math.ceil(
            representation.MINIMUM_MODE_SEPARATION_HZ
            / (frequencies[1] - frequencies[0])
        ),
    )
    peaks, _ = representation.signal.find_peaks(summary, distance=separation_bins)
    peaks = peaks[eligible[peaks]]
    strength_order = peaks[np.argsort(-summary[peaks], kind="stable")]
    if strength_order.size < representation.MODE_COUNT:
        raise FitError("A1R Glass object exposes too few separated modal peaks")
    selected = strength_order[: representation.MODE_COUNT]

    tail = targets[:, ALIGNMENT_SAMPLE:]
    _, times, stft = representation.signal.stft(
        tail,
        fs=SAMPLE_RATE_HZ,
        window="hann",
        nperseg=endpoint.MODAL_STFT_SAMPLES,
        noverlap=endpoint.MODAL_STFT_SAMPLES - endpoint.MODAL_STFT_HOP,
        boundary=None,
        padded=False,
        axis=-1,
    )
    stft_frequencies = np.fft.rfftfreq(
        endpoint.MODAL_STFT_SAMPLES, 1.0 / SAMPLE_RATE_HZ
    )
    magnitude = np.sqrt(np.mean(np.square(np.abs(stft)), axis=0))
    rows = []
    for rank, peak_index in enumerate(selected):
        frequency = float(frequencies[peak_index])
        bin_index = int(np.argmin(np.abs(stft_frequencies - frequency)))
        envelope = magnitude[bin_index]
        maximum = max(float(np.max(envelope)), 1.0e-30)
        start = int(np.argmax(envelope))
        usable = np.arange(envelope.size) >= start
        usable &= times <= min(float(times[-1]), 2.5)
        usable &= envelope >= maximum * 1.0e-3
        if np.count_nonzero(usable) >= 6:
            slope = np.polyfit(
                times[usable], np.log(np.maximum(envelope[usable], 1.0e-30)), 1
            )[0]
            damping = float(np.clip(-slope, 0.25, 200.0))
        else:
            damping = 20.0
        rows.append((frequency, damping, float(rank)))
    poles = np.asarray(rows, dtype=np.float64)
    order = np.argsort(poles[:, 0], kind="stable")
    poles = np.ascontiguousarray(poles[order])
    if not np.isfinite(poles).all():
        raise FitError("A1R pole estimator produced non-finite values")
    return poles, summary[selected]


def build_manifest(
    environment: dict[str, Any], implementation: dict[str, str]
) -> dict[str, Any]:
    manifest = representation.build_manifest(environment, implementation)
    manifest.update(
        {
            "schema": MANIFEST_SCHEMA,
            "status": "FrozenA1RForceOnsetFitOnly",
            "study_id": STUDY_ID,
            "revision": REVISION,
            "inventory_manifest_sha256": INVENTORY_MANIFEST_SHA256,
            "object_id": OBJECT_ID,
            "fit_contacts": list(FIT_CONTACT_IDS),
            "protected_roles": {
                "development": list(DEVELOPMENT_CONTACT_IDS),
                "sealed": list(SEALED_CONTACT_IDS),
            },
            "source_files": {
                "raw_prefix": {
                    "url": source.RAW_ARCHIVE_URL,
                    "range": source.RAW_PREFIX_RANGE,
                    "bytes": source.RAW_PREFIX_BYTES,
                    "sha256": source.RAW_PREFIX_SHA256,
                },
                "official_page": {
                    "url": source.OFFICIAL_PAGE_URL,
                    "bytes": source.OFFICIAL_PAGE_BYTES,
                    "sha256": source.OFFICIAL_PAGE_SHA256,
                },
                "compact": source.compact.SOURCES,
            },
            "preprocessing": {
                "sample_rate_hz": SAMPLE_RATE_HZ,
                "source_samples": SOURCE_SAMPLES,
                "analysis_samples": ANALYSIS_SAMPLES,
                "baseline_samples": BASELINE_SAMPLES,
                "alignment_sample": ALIGNMENT_SAMPLE,
                "centering": "independent_microphone_and_force_first_12000_sample_median",
                "onset": "force_first_abs_gte_max_0.05_peak_and_baseline_abs_median_plus_10_mad",
                "shift": "microphone_and_force_together_zero_pad_without_wrap",
                "normalization": "one_fit_only_microphone_scale_to_global_peak_0.92",
                "resampling": "forbidden",
                "force_role": "synchronization_only_not_target_or_modal_input",
            },
            "microphone_sample_values_decoded": 0,
            "force_sample_values_decoded": 0,
            "fit_microphone_sample_values_decoded": 0,
            "fit_force_sample_values_decoded": 0,
            "development_microphone_sample_values_decoded": 0,
            "development_force_sample_values_decoded": 0,
            "sealed_microphone_sample_values_decoded": 0,
            "sealed_force_sample_values_decoded": 0,
            "sealed_authorized": False,
        }
    )
    return manifest


def run(
    root: Path,
    inventory_path: Path,
    prefix_path: Path,
    page_path: Path,
    audio_path: Path,
    contacts_path: Path,
    points_path: Path,
    split_path: Path,
    scale_path: Path,
    output_path: Path,
) -> Path:
    implementation = validate_implementation(root)
    inventory_manifest = validate_inventory_manifest(root, inventory_path)
    prefix = validate_sources(
        root,
        prefix_path,
        page_path,
        audio_path,
        contacts_path,
        points_path,
        split_path,
        scale_path,
    )
    environment = representation.environment_identity()
    manifest = build_manifest(environment, implementation)
    commitments = fit_commitments(inventory_manifest)
    pairs = extract_fit_pairs(prefix, commitments)
    targets, preprocessing, fit_scale = load_fit_targets(pairs)
    if targets.size != EXPECTED_RETAINED_SAMPLES:
        raise FitError("A1R retained microphone sample accounting changed")

    raw_poles, strengths = estimate_poles(targets)
    poles = raw_poles.copy()
    poles[:, :2] = poles[:, :2].astype("<f4").astype(np.float64)
    gains = representation.fit_modal_gains(targets, poles)
    centers = representation.band_centers_hz()
    carriers = representation.generate_carriers(centers)
    if not np.array_equal(carriers, representation.generate_carriers(centers)):
        raise FitError("A1R deterministic carrier regeneration changed")
    calibration = representation.carrier_calibration(carriers, centers)

    output, staging = representation.prepare_output(root, output_path)
    try:
        poles_payload = np.ascontiguousarray(poles[:, :2], dtype="<f4")
        centers_payload = np.ascontiguousarray(centers, dtype="<f4")
        poles_file = staging / "shared-poles.npy"
        centers_file = staging / "shared-band-centers.npy"
        np.save(poles_file, poles_payload, allow_pickle=False)
        np.save(centers_file, centers_payload, allow_pickle=False)
        shared_bytes = poles_payload.nbytes + centers_payload.nbytes

        contacts = []
        candidate_metrics = []
        modal_metrics = []
        stationary_metrics = []
        all_fit_contacts_pass = True
        maximum_record_bytes = 0
        for row, contact_id in enumerate(FIT_CONTACT_IDS):
            encoded = representation.encode_contact(
                targets[row], poles, gains[row], centers, carriers, calibration
            )
            repeated = representation.encode_contact(
                targets[row], poles, gains[row], centers, carriers, calibration
            )
            candidate, ablations, record, descriptor = encoded
            if (
                record != repeated[2]
                or not np.array_equal(candidate, repeated[0])
                or not np.array_equal(ablations, repeated[1])
                or descriptor != repeated[3]
            ):
                raise FitError(f"A1R contact {contact_id} reconstruction is not exact")
            (staging / f"contact-{contact_id}.bin").write_bytes(record)
            metrics = endpoint.metrics(targets[row], candidate)
            modal_only = endpoint.metrics(targets[row], ablations[0])
            stationary_only = endpoint.metrics(targets[row], ablations[1])
            within = {
                name: metrics[name] <= endpoint.ABSOLUTE_THRESHOLDS[name]
                for name in endpoint.PRIMARY_ENDPOINTS
            }
            finite = (
                np.isfinite(candidate).all()
                and all(math.isfinite(value) for value in metrics.values())
                and all(math.isfinite(value) for value in modal_only.values())
                and all(math.isfinite(value) for value in stationary_only.values())
            )
            contact_pass = (
                bool(finite)
                and all(within.values())
                and descriptor["residual_temporal_variation"]
                and descriptor["encoded_bytes"]
                <= representation.MAXIMUM_CONTACT_BYTES
            )
            all_fit_contacts_pass &= contact_pass
            maximum_record_bytes = max(maximum_record_bytes, len(record))
            candidate_metrics.append(metrics)
            modal_metrics.append(modal_only)
            stationary_metrics.append(stationary_only)
            inventory_contact = next(
                item
                for item in inventory_manifest["contacts"]
                if int(item["contact_id"]) == contact_id
            )
            contacts.append(
                {
                    "contact_id": contact_id,
                    "coordinate_m": inventory_contact["coordinate"]["coordinate_m"],
                    "microphone_sha256": commitments[
                        f"{OBJECT_ID}/audio/{contact_id}/mic.wav"
                    ][1],
                    "force_sha256": commitments[
                        f"{OBJECT_ID}/audio/{contact_id}/Force.wav"
                    ][1],
                    "preprocessing": preprocessing[row],
                    "metrics": metrics,
                    "within_absolute_threshold": within,
                    "finite": bool(finite),
                    "fit_gate_passed": bool(contact_pass),
                    "ablations": {
                        "modal_only": modal_only,
                        "stationary_first_temporal_coefficient": stationary_only,
                    },
                    **descriptor,
                }
            )

        microphone_samples = len(pairs) * SOURCE_SAMPLES
        force_samples = len(pairs) * SOURCE_SAMPLES
        protected_counters = {
            "development_microphone_sample_values_decoded": 0,
            "development_force_sample_values_decoded": 0,
            "sealed_microphone_sample_values_decoded": 0,
            "sealed_force_sample_values_decoded": 0,
            "method_holdout_waveform_sample_values_decoded": 0,
            "admission_shadow_waveform_sample_values_decoded": 0,
        }
        hard_gates = {
            "decoded_microphone_sample_count": microphone_samples
            == EXPECTED_CHANNEL_SAMPLES,
            "decoded_force_sample_count": force_samples == EXPECTED_CHANNEL_SAMPLES,
            "decoded_total_sample_count": microphone_samples + force_samples
            == EXPECTED_TOTAL_DECODED_SAMPLES,
            "retained_fit_sample_count": targets.size == EXPECTED_RETAINED_SAMPLES,
            "all_force_onsets_present": len(preprocessing) == len(FIT_CONTACT_IDS),
            "protected_decode_zero": all(
                value == 0 for value in protected_counters.values()
            ),
            "mode_count": poles.shape == (representation.MODE_COUNT, 3),
            "modes_inside_evaluation_band": bool(
                np.all(poles[:, 0] >= endpoint.EVALUATION_MIN_HZ)
                and np.all(poles[:, 0] <= endpoint.EVALUATION_MAX_HZ)
            ),
            "band_count": centers.shape == (representation.NOISE_BAND_COUNT,),
            "bands_inside_evaluation_band": bool(
                np.all(centers >= endpoint.EVALUATION_MIN_HZ)
                and np.all(centers <= endpoint.EVALUATION_MAX_HZ)
            ),
            "shared_decoder_budget": shared_bytes
            <= representation.MAXIMUM_SHARED_BYTES,
            "contact_record_budget": maximum_record_bytes
            <= representation.MAXIMUM_CONTACT_BYTES,
            "exact_contact_record_bytes": maximum_record_bytes
            == representation.EXPECTED_CONTACT_RECORD_BYTES,
            "all_fit_contacts_finite": all(item["finite"] for item in contacts),
            "all_fit_contacts_temporally_varying": all(
                item["residual_temporal_variation"] for item in contacts
            ),
            "carrier_regeneration_exact": True,
        }
        passed = all_fit_contacts_pass and all(hard_gates.values())
        decision = (
            "READY_FOR_A1R_DEVELOPMENT"
            if passed
            else "REJECT_A1R_V9_REAL_REPRESENTATION"
        )

        manifest.update(
            {
                "microphone_sample_values_decoded": microphone_samples,
                "force_sample_values_decoded": force_samples,
                "fit_microphone_sample_values_decoded": microphone_samples,
                "fit_force_sample_values_decoded": force_samples,
                "waveform_sample_values_decoded": microphone_samples,
                "fit_waveform_sample_values_decoded": microphone_samples,
                **protected_counters,
            }
        )
        manifest_bytes = representation.canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        model = {
            "schema": MODEL_SCHEMA,
            "status": "Frozen",
            "decision": decision,
            "manifest_sha256": representation.sha256_bytes(manifest_bytes),
            "fit_microphone_scale": fit_scale,
            "preprocessing": preprocessing,
            "poles_path": poles_file.name,
            "poles_sha256": representation.sha256_file(poles_file),
            "pole_selection_strength_sha256": representation.array_sha256(strengths),
            "band_centers_path": centers_file.name,
            "band_centers_sha256": representation.sha256_file(centers_file),
            "carrier_bank_sha256": representation.array_sha256(carriers),
            "carrier_calibration_sha256": representation.array_sha256(calibration),
            "shared_decoder_bytes": shared_bytes,
            "contact_records": {
                str(item["contact_id"]): item["record_sha256"] for item in contacts
            },
            "maximum_contact_record_bytes": maximum_record_bytes,
            **protected_counters,
        }
        model_bytes = representation.canonical_json(model)
        (staging / "model.json").write_bytes(model_bytes)
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": decision,
            "manifest_sha256": representation.sha256_bytes(manifest_bytes),
            "model_sha256": representation.sha256_bytes(model_bytes),
            "all_fit_contacts_passed": bool(all_fit_contacts_pass),
            "hard_gates": hard_gates,
            "fit_contacts": contacts,
            "metric_summary": representation.metric_summary(candidate_metrics),
            "ablation_summary": {
                "modal_only": representation.metric_summary(modal_metrics),
                "stationary_first_temporal_coefficient": representation.metric_summary(
                    stationary_metrics
                ),
            },
            "microphone_sample_values_decoded": microphone_samples,
            "force_sample_values_decoded": force_samples,
            "fit_analysis_sample_values_used": targets.size,
            **protected_counters,
            "real_quality_credit": bool(passed),
            "development_authorized": bool(passed),
            "sealed_authorized": False,
            "project_disjoint_validation_credit": False,
            "r3b_authorized": False,
            "runtime_or_public_contract_changed": False,
            "authored_clip_fallback_required": True,
        }
        (staging / "report.json").write_bytes(
            representation.canonical_json(report)
        )
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inventory-manifest", required=True, type=Path)
    parser.add_argument("--archive-prefix", required=True, type=Path)
    parser.add_argument("--official-page", required=True, type=Path)
    parser.add_argument("--audio", required=True, type=Path)
    parser.add_argument("--contacts", required=True, type=Path)
    parser.add_argument("--point-cloud", required=True, type=Path)
    parser.add_argument("--split", required=True, type=Path)
    parser.add_argument("--scale", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    output = run(
        repository_root(),
        arguments.inventory_manifest,
        arguments.archive_prefix,
        arguments.official_page,
        arguments.audio,
        arguments.contacts,
        arguments.point_cloud,
        arguments.split,
        arguments.scale,
        arguments.output,
    )
    report = json.loads((output / "report.json").read_bytes())
    print(f"R3A V10 A1R force-onset fit: {output}")
    print(f"decision: {report['decision']}")
    print(f"manifest sha256: {representation.sha256_file(output / 'manifest.json')}")
    print(f"model sha256: {representation.sha256_file(output / 'model.json')}")
    print(f"report sha256: {representation.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
