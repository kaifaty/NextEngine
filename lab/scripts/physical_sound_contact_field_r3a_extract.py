#!/usr/bin/env python3
"""Extract only R3A fit/development rows from a pinned raw ZIP deflate member."""

from __future__ import annotations

import argparse
import zlib
from pathlib import Path
from typing import BinaryIO, Callable

import numpy as np

import physical_sound_contact_field_r3a_common as common

CHUNK_BYTES = 8 * 1024 * 1024
NPY_HEADER_BYTES = 128


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--compressed-entry", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def validate_npy_header(header: bytes) -> None:
    if (
        len(header) != NPY_HEADER_BYTES
        or header[:6] != b"\x93NUMPY"
        or header[6:8] != bytes([1, 0])
        or int.from_bytes(header[8:10], "little") != 118
    ):
        raise common.R3AError("REALIMPACT transfer NPY header changed")
    descriptor = header[10:].decode("ascii")
    if (
        "'descr': '<f4'" not in descriptor
        or "'fortran_order': False" not in descriptor
        or "'shape': (3000, 230215)" not in descriptor
        or not descriptor.endswith("\n")
    ):
        raise common.R3AError("REALIMPACT transfer NPY descriptor changed")


def extract_rows(
    handle: BinaryIO,
    row_indices: list[int],
    *,
    sample_count: int = common.SAMPLE_COUNT,
    header_validator: Callable[[bytes], None] = validate_npy_header,
) -> tuple[np.ndarray, int, int]:
    if row_indices != sorted(row_indices) or len(set(row_indices)) != len(row_indices):
        raise common.R3AError("selected audio rows must be unique and ordered")
    if not row_indices or row_indices[-1] >= common.ROW_COUNT:
        raise common.R3AError("selected audio row is outside the frozen array")
    row_bytes = sample_count * 4
    stop_raw = NPY_HEADER_BYTES + (row_indices[-1] + 1) * row_bytes
    rows = {row: bytearray() for row in row_indices}
    header = bytearray()
    decoder = zlib.decompressobj(wbits=-15)
    raw_offset = 0
    compressed_read = 0

    while raw_offset < stop_raw:
        block = handle.read(CHUNK_BYTES)
        if not block:
            raise common.R3AError(
                "compressed audio member ended before development row"
            )
        compressed_read += len(block)
        remaining = stop_raw - raw_offset
        try:
            decoded = decoder.decompress(block, max_length=remaining)
        except zlib.error as error:
            raise common.R3AError(f"decompress transfer member: {error}") from error
        if not decoded:
            continue
        chunk_start = raw_offset
        chunk_end = chunk_start + len(decoded)
        if chunk_start < NPY_HEADER_BYTES:
            start = chunk_start
            end = min(chunk_end, NPY_HEADER_BYTES)
            header.extend(decoded[start - chunk_start : end - chunk_start])
        for row in row_indices:
            start = NPY_HEADER_BYTES + row * row_bytes
            end = start + row_bytes
            overlap_start = max(chunk_start, start)
            overlap_end = min(chunk_end, end)
            if overlap_start < overlap_end:
                rows[row].extend(
                    decoded[overlap_start - chunk_start : overlap_end - chunk_start]
                )
        raw_offset = chunk_end

    header_validator(bytes(header))
    if raw_offset != stop_raw or any(
        len(rows[row]) != row_bytes for row in row_indices
    ):
        raise common.R3AError("selected transfer rows are incomplete")
    stacked = np.stack(
        [np.frombuffer(rows[row], dtype="<f4").copy() for row in row_indices]
    )
    if stacked.shape != (len(row_indices), sample_count) or not np.all(
        np.isfinite(stacked)
    ):
        raise common.R3AError("selected transfer rows have invalid samples")
    return stacked, compressed_read, raw_offset


def run(
    root: Path,
    manifest_argument: Path,
    compressed_argument: Path,
    output_argument: Path,
) -> Path:
    manifest_path = common.external_file(root, manifest_argument, "R3A manifest")
    manifest_bytes, manifest = common.load_json(manifest_path, "R3A manifest")
    common.validate_manifest(manifest)
    common.validate_implementation(manifest, Path(__file__).resolve().parent)
    compressed_path = common.external_file(
        root, compressed_argument, "REALIMPACT compressed audio member"
    )
    if compressed_path.stat().st_size != common.AUDIO_ENTRY["compressed_bytes"]:
        raise common.R3AError("compressed audio member size changed")
    compressed_sha256 = common.sha256_file(compressed_path)
    authorized = [
        contact
        for contact in manifest["contacts"]
        if contact["waveform_access"] == "authorized"
    ]
    sealed = [
        contact
        for contact in manifest["contacts"]
        if contact["waveform_access"] == "sealed"
    ]
    row_indices = [int(contact["row_index"]) for contact in authorized]
    if (
        len(authorized) != 4
        or len(sealed) != 1
        or row_indices[-1] >= sealed[0]["row_index"]
    ):
        raise common.R3AError("R3A seal boundary changed")

    output, staging = common.prepare_output(root, output_argument)
    try:
        with compressed_path.open("rb") as handle:
            waveforms, compressed_read, raw_read = extract_rows(handle, row_indices)
        waveform_path = staging / "contacts.npy"
        with waveform_path.open("wb") as handle:
            np.save(handle, waveforms, allow_pickle=False)
        metadata = {
            "schema": "nextengine.experimental-physical-sound-r3a-contacts.v1",
            "profile": common.PROFILE_ID,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "sample_rate_hz": common.SAMPLE_RATE_HZ,
            "sample_count": common.SAMPLE_COUNT,
            "contacts": authorized,
            "sealed_contact_commitment": {
                "impact_index": sealed[0]["impact_index"],
                "vertex_id": sealed[0]["vertex_id"],
                "position_metres": sealed[0]["position_metres"],
                "row_index": sealed[0]["row_index"],
                "waveform_access": "sealed_not_decompressed",
            },
        }
        metadata_bytes = common.canonical_json(metadata)
        (staging / "contacts-metadata.json").write_bytes(metadata_bytes)
        report = {
            "schema": common.EXTRACTION_REPORT_SCHEMA,
            "status": "Validated",
            "decision": "R3AAuthorizedContactsExtracted",
            "profile": common.PROFILE_ID,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "extraction_runner_sha256": common.sha256_file(Path(__file__).resolve()),
            "compressed_member_sha256": compressed_sha256,
            "compressed_member_bytes": compressed_path.stat().st_size,
            "compressed_file_bytes_read_during_extraction": compressed_read,
            "uncompressed_bytes_decoded": raw_read,
            "uncompressed_stop_before_sealed_row_bytes": NPY_HEADER_BYTES
            + sealed[0]["row_index"] * common.SAMPLE_COUNT * 4,
            "waveforms_sha256": common.sha256_file(waveform_path),
            "contacts_metadata_sha256": common.sha256_bytes(metadata_bytes),
            "authorized_row_indices": row_indices,
            "sealed_row_index": sealed[0]["row_index"],
            "sealed_waveform_samples_decoded": 0,
            "neural_training_authorized": False,
        }
        (staging / "report.json").write_bytes(common.canonical_json(report))
        common.publish_output(output, staging)
    except BaseException:
        for child in staging.iterdir():
            child.unlink()
        staging.rmdir()
        raise
    return output


def main() -> None:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    output = run(root, arguments.manifest, arguments.compressed_entry, arguments.output)
    print(f"R3A contact extraction: {output}")
    print(f"report sha256: {common.sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
