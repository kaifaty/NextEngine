#!/usr/bin/env python3
"""External R63ZM mutation/reseal harness; never part of checker work."""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import sys


ARTIFACT_BYTES = 12916
AUDIT_BYTES = 420
CHECKER_WORK_FIELDS = 26
ROWS = 102
ROLES = 6
EVENTS = 9
PRODUCT_RECORD_BYTES = 1944
PRODUCT_BASE = 1236

SOURCE_ROOT = 28
ROUTE = 60
SELECTED_ROOT = 64
BUNDLE_ROOT = 96
PRODUCT_SET_ROOT = 128
EVENT_COUNT = 160
EVENT_ROOTS = 164
TRACE_ROOT = 452
RESULT_ROOT = 484
READ_WORK = 516
ADMISSION_EXACT = 724
ADMISSION_WORK = 732
SEAL_WORK = 876
PAYLOAD_ROOTS = 1004
PRODUCT_COUNT = 1228

VECTOR_DOMAIN = b"nextengine.nonlocal.r63zm.binary128-vector.v1"
PRODUCT_DOMAIN = b"nextengine.nonlocal.r63zm.product-binary.v1"
PRODUCT_SET_DOMAIN = b"nextengine.nonlocal.r63zm.product-set-binary.v1"
BUNDLE_DOMAIN = b"nextengine.nonlocal.r63zm.selected-bundle-binary.v1"
READ_WORK_DOMAIN = b"nextengine.nonlocal.r63zm.read-work-binary.v2"
ADMISSION_WORK_DOMAIN = b"nextengine.nonlocal.r63zm.admission-work-binary.v2"
SEAL_WORK_DOMAIN = b"nextengine.nonlocal.r63zm.seal-work-binary.v2"
EVENT_DOMAIN = b"nextengine.nonlocal.r63zm.event-binary.v2"
TRACE_DOMAIN = b"nextengine.nonlocal.r63zm.trace-binary.v2"
BODY_DOMAIN = b"nextengine.nonlocal.r63zm.artifact-body-binary.v1"
RESULT_DOMAIN = b"nextengine.nonlocal.r63zm.result-binary.v2"


def u64(value: int) -> bytes:
    return value.to_bytes(8, "big")


def tlv(tag: int, value: bytes) -> bytes:
    return bytes((tag,)) + u64(len(value)) + value


def sha(material: bytes) -> bytes:
    return hashlib.sha256(material).digest()


def read_u32(data: bytearray, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "big")


def read_u64(data: bytearray, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 8], "big")


def write_u32(data: bytearray, offset: int, value: int) -> None:
    data[offset : offset + 4] = value.to_bytes(4, "big")


def write_u64(data: bytearray, offset: int, value: int) -> None:
    data[offset : offset + 8] = value.to_bytes(8, "big")


def work_fields(data: bytearray, offset: int, count: int) -> tuple[int, ...]:
    return tuple(read_u64(data, offset + 8 * index) for index in range(count))


def work_root(domain: bytes, fields: tuple[int, ...]) -> bytes:
    return sha(tlv(1, domain) + tlv(2, b"".join(u64(value) for value in fields)))


def vector_root(payload: bytes, count: int) -> bytes:
    assert len(payload) == count * 16
    return sha(tlv(1, VECTOR_DOMAIN) + tlv(2, u64(count)) + tlv(3, payload))


def product_offset(role: int) -> int:
    return PRODUCT_BASE + role * PRODUCT_RECORD_BYTES


def payload_root(data: bytearray, index: int) -> bytes:
    offset = PAYLOAD_ROOTS + 32 * index
    return bytes(data[offset : offset + 32])


def product_root_from_fields(data: bytearray, role: int) -> bytes:
    base = product_offset(role)
    exact = data[base]
    no_underflow = data[base + 1]
    stored_role = data[base + 2]
    work = bytes(data[base + 8 : base + 8 + 17 * 8])
    roots = [
        bytes(data[base + 144 + 32 * index : base + 176 + 32 * index])
        for index in range(4)
    ]
    material = (
        tlv(1, PRODUCT_DOMAIN)
        + tlv(2, bytes((stored_role,)))
        + tlv(3, bytes((exact,)))
        + tlv(4, bytes((no_underflow,)))
        + tlv(5, u64(ROWS))
        + tlv(6, u64(315))
        + tlv(7, payload_root(data, role + 1))
        + tlv(8, work)
    )
    for index, root in enumerate(roots):
        material += tlv(0x10 + index, root)
    assert len(material) == 466
    return sha(material)


def product_set_root(data: bytearray) -> bytes:
    material = tlv(1, PRODUCT_SET_DOMAIN) + tlv(2, u64(ROLES))
    for role in range(ROLES):
        base = product_offset(role)
        material += tlv(0x10 + 2 * role, bytes((role,)))
        material += tlv(0x11 + 2 * role, bytes(data[base + 272 : base + 304]))
    assert len(material) == 379
    return sha(material)


def bundle_root(data: bytearray) -> bytes:
    material = (
        tlv(1, BUNDLE_DOMAIN)
        + tlv(2, bytes(data[SOURCE_ROOT : SOURCE_ROOT + 32]))
        + tlv(3, bytes(data[SELECTED_ROOT : SELECTED_ROOT + 32]))
    )
    for index in range(7):
        material += tlv(0x10 + index, payload_root(data, index))
    assert len(material) == 429
    return sha(material)


def event_root(kind: int, ordinal: int, child: bytes, auxiliary: bytes) -> bytes:
    material = (
        tlv(1, EVENT_DOMAIN)
        + tlv(2, bytes((kind,)))
        + tlv(3, u64(ordinal))
        + tlv(4, child)
        + tlv(5, auxiliary)
    )
    assert len(material) == 159
    return sha(material)


def trace_root(data: bytearray) -> bytes:
    route = read_u32(data, ROUTE)
    event_count = read_u32(data, EVENT_COUNT)
    material = tlv(1, TRACE_DOMAIN) + tlv(2, bytes((route & 0xFF,)))
    material += tlv(3, u64(event_count))
    for index in range(event_count):
        offset = EVENT_ROOTS + 32 * index
        material += tlv(0x10 + index, bytes(data[offset : offset + 32]))
    return sha(material)


def result_root(data: bytearray) -> bytes:
    route = read_u32(data, ROUTE)
    source_bytes = read_u64(data, 20)
    body = bytearray(data)
    body[RESULT_ROOT : RESULT_ROOT + 32] = bytes(32)
    body_root = sha(tlv(1, BODY_DOMAIN) + tlv(2, bytes(body)))
    material = (
        tlv(1, RESULT_DOMAIN)
        + tlv(2, u64(3))
        + tlv(3, bytes((route & 0xFF,)))
        + tlv(4, u64(source_bytes))
        + tlv(5, body_root)
        + tlv(6, u64(ARTIFACT_BYTES))
    )
    assert len(material) == 153
    return sha(material)


def rebuild_products_and_downstream(data: bytearray) -> None:
    data[BUNDLE_ROOT : BUNDLE_ROOT + 32] = bundle_root(data)
    for role in range(ROLES):
        base = product_offset(role)
        values = bytes(data[base + 312 : base + 312 + ROWS * 16])
        data[base + 144 + 2 * 32 : base + 144 + 3 * 32] = vector_root(values, ROWS)
        data[base + 272 : base + 304] = product_root_from_fields(data, role)
    rebuild_from_product_roots(data)


def rebuild_from_product_roots(data: bytearray) -> None:
    data[PRODUCT_SET_ROOT : PRODUCT_SET_ROOT + 32] = product_set_root(data)
    read_root = work_root(READ_WORK_DOMAIN, work_fields(data, READ_WORK, 26))
    admission_root = work_root(
        ADMISSION_WORK_DOMAIN, work_fields(data, ADMISSION_WORK, 18)
    )
    data[EVENT_ROOTS : EVENT_ROOTS + 32] = event_root(
        0, 0, bytes(data[SOURCE_ROOT : SOURCE_ROOT + 32]), read_root
    )
    data[EVENT_ROOTS + 32 : EVENT_ROOTS + 64] = event_root(
        1, 1, bytes(data[SELECTED_ROOT : SELECTED_ROOT + 32]), admission_root
    )
    data[EVENT_ROOTS + 64 : EVENT_ROOTS + 96] = event_root(
        2,
        2,
        bytes(data[BUNDLE_ROOT : BUNDLE_ROOT + 32]),
        bytes(data[SELECTED_ROOT : SELECTED_ROOT + 32]),
    )
    for role in range(ROLES):
        base = product_offset(role)
        event = event_root(
            3,
            3 + role,
            bytes(data[base + 272 : base + 304]),
            payload_root(data, role + 1),
        )
        offset = EVENT_ROOTS + 32 * (3 + role)
        data[offset : offset + 32] = event
    rebuild_from_events(data)


def rebuild_from_events(data: bytearray) -> None:
    data[TRACE_ROOT : TRACE_ROOT + 32] = trace_root(data)
    data[RESULT_ROOT : RESULT_ROOT + 32] = result_root(data)


def rebuild_result(data: bytearray) -> None:
    data[RESULT_ROOT : RESULT_ROOT + 32] = result_root(data)


def flip(data: bytearray, offset: int) -> None:
    data[offset] ^= 0x01


def run_producer(
    producer: pathlib.Path,
    cache: pathlib.Path,
    artifact: pathlib.Path,
) -> dict:
    completed = subprocess.run(
        [str(producer), str(cache), str(artifact)],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if completed.stdout:
        raise RuntimeError("producer emitted post-seal stdout")
    data = artifact.read_bytes()
    if len(data) != ARTIFACT_BYTES or data[:8] != b"NER63ZM1":
        raise RuntimeError("producer emitted malformed fixed artifact")
    if int.from_bytes(data[8:12], "big") != 3:
        raise RuntimeError("unexpected producer artifact version")
    route = int.from_bytes(data[ROUTE : ROUTE + 4], "big")
    expected_exit = 0 if route == 0 else 1
    if completed.returncode != expected_exit:
        raise RuntimeError("producer route/exit mismatch")
    return {
        "producer_route": route,
        "producer_exit": completed.returncode,
        "producer_first_failure": read_u64(data, ADMISSION_WORK + 8),
        "producer_allocation": completed.stderr.strip(),
        "producer_artifact_sha256": hashlib.sha256(data).hexdigest(),
    }


def run_checker(
    checker: pathlib.Path,
    cache: pathlib.Path,
    artifact: pathlib.Path,
    audit: pathlib.Path,
) -> dict:
    completed = subprocess.run(
        [str(checker), str(cache), str(artifact), str(audit)],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if completed.stdout:
        raise RuntimeError("checker emitted post-seal stdout")
    data = audit.read_bytes()
    if len(data) != AUDIT_BYTES or data[:8] != b"NER63QC1":
        raise RuntimeError("malformed checker audit")
    if int.from_bytes(data[8:12], "big") != 3:
        raise RuntimeError("unexpected checker audit version")
    if int.from_bytes(data[12:20], "big") != len(data):
        raise RuntimeError("checker audit size mismatch")
    receipt = {
        "route": int.from_bytes(data[20:24], "big"),
        "semantic": data[24] == 1,
        "work": data[25] == 1,
        "seal": data[26] == 1,
        "components": int.from_bytes(data[28:36], "big"),
        "checker_work": [
            int.from_bytes(data[36 + 8 * index : 44 + 8 * index], "big")
            for index in range(CHECKER_WORK_FIELDS)
        ],
        "cache_bytes": int.from_bytes(data[244:252], "big"),
        "cache_root": data[252:284].hex(),
        "artifact_bytes": int.from_bytes(data[284:292], "big"),
        "artifact_root": data[292:324].hex(),
        "result_root": data[324:356].hex(),
        "reconstructed_result_root": data[356:388].hex(),
        "checker_root": data[388:420].hex(),
        "audit_sha256": hashlib.sha256(data).hexdigest(),
        "exit": completed.returncode,
        "allocation": completed.stderr.strip(),
    }
    return receipt


def main() -> int:
    if len(sys.argv) != 6:
        return 2
    producer = pathlib.Path(sys.argv[1]).resolve()
    checker = pathlib.Path(sys.argv[2]).resolve()
    cache = pathlib.Path(sys.argv[3]).resolve()
    baseline_path = pathlib.Path(sys.argv[4]).resolve()
    output_dir = pathlib.Path(sys.argv[5]).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    baseline = bytearray(baseline_path.read_bytes())
    if len(baseline) != ARTIFACT_BYTES:
        raise RuntimeError("unexpected baseline artifact size")
    independently_resealed = bytearray(baseline)
    rebuild_products_and_downstream(independently_resealed)
    if independently_resealed != baseline:
        raise RuntimeError("independent Python baseline reseal mismatch")

    cases: list[tuple[str, int, bytearray]] = []

    def add(name: str, route: int, mutate) -> None:
        data = bytearray(baseline)
        mutate(data)
        cases.append((name, route, data))

    add("stale_version", 4, lambda data: write_u32(data, 8, 4))
    add("nonzero_padding", 4, lambda data: flip(data, ADMISSION_EXACT + 1))
    add("event_count", 4, lambda data: write_u32(data, EVENT_COUNT, 8))
    add("product_count", 4, lambda data: write_u32(data, PRODUCT_COUNT, 5))
    add("product_value_count", 4,
        lambda data: write_u64(data, product_offset(0) + 304, 101))

    def unknown_route(data: bytearray) -> None:
        write_u32(data, ROUTE, 99)
        rebuild_from_events(data)

    add("unknown_route_resealed", 8, unknown_route)

    def component_resealed(data: bytearray) -> None:
        flip(data, product_offset(0) + 312)
        rebuild_products_and_downstream(data)

    add("component_resealed", 5, component_resealed)

    def exact_flag_resealed(data: bytearray) -> None:
        data[product_offset(1)] = 0
        rebuild_products_and_downstream(data)

    add("product_semantic_resealed", 5, exact_flag_resealed)

    def product_work_resealed(data: bytearray) -> None:
        offset = product_offset(2) + 8 + 6 * 8
        write_u64(data, offset, read_u64(data, offset) + 1)
        rebuild_products_and_downstream(data)

    add("product_work_resealed", 6, product_work_resealed)

    def read_work_resealed(data: bytearray) -> None:
        write_u64(data, READ_WORK, read_u64(data, READ_WORK) + 1)
        rebuild_from_product_roots(data)

    add("read_work_resealed", 6, read_work_resealed)

    def admission_work_resealed(data: bytearray) -> None:
        write_u64(data, ADMISSION_WORK,
                  read_u64(data, ADMISSION_WORK) + 1)
        rebuild_from_product_roots(data)

    add("admission_work_resealed", 6, admission_work_resealed)

    def seal_work_resealed(data: bytearray) -> None:
        write_u64(data, SEAL_WORK + 12 * 8,
                  read_u64(data, SEAL_WORK + 12 * 8) + 1)
        rebuild_result(data)

    add("seal_work_resealed", 6, seal_work_resealed)

    def source_resealed(data: bytearray) -> None:
        flip(data, SOURCE_ROOT)
        rebuild_products_and_downstream(data)

    add("source_root_resealed", 7, source_resealed)

    def selected_resealed(data: bytearray) -> None:
        flip(data, SELECTED_ROOT)
        rebuild_products_and_downstream(data)

    add("selected_root_resealed", 7, selected_resealed)

    def payload_resealed(data: bytearray) -> None:
        flip(data, PAYLOAD_ROOTS + 32)
        rebuild_products_and_downstream(data)

    add("payload_root_resealed", 7, payload_resealed)

    def product_root_resealed(data: bytearray) -> None:
        flip(data, product_offset(3) + 272)
        rebuild_from_product_roots(data)

    add("product_root_resealed", 7, product_root_resealed)

    def product_set_resealed(data: bytearray) -> None:
        flip(data, PRODUCT_SET_ROOT)
        rebuild_from_events(data)

    add("product_set_root_resealed", 7, product_set_resealed)

    def output_vector_root_resealed(data: bytearray) -> None:
        flip(data, product_offset(4) + 144 + 2 * 32)
        rebuild_from_product_roots(data)

    add("output_vector_root_resealed", 7, output_vector_root_resealed)

    def deleted_event_resealed(data: bytearray) -> None:
        data[EVENT_ROOTS + 4 * 32 : EVENT_ROOTS + 5 * 32] = bytes(32)
        rebuild_from_events(data)

    add("deleted_event_resealed", 7, deleted_event_resealed)

    def duplicate_event_resealed(data: bytearray) -> None:
        data[EVENT_ROOTS + 5 * 32 : EVENT_ROOTS + 6 * 32] = data[
            EVENT_ROOTS + 4 * 32 : EVENT_ROOTS + 5 * 32
        ]
        rebuild_from_events(data)

    add("duplicate_event_resealed", 7, duplicate_event_resealed)

    def reordered_events_resealed(data: bytearray) -> None:
        left = bytes(data[EVENT_ROOTS + 3 * 32 : EVENT_ROOTS + 4 * 32])
        right = bytes(data[EVENT_ROOTS + 4 * 32 : EVENT_ROOTS + 5 * 32])
        data[EVENT_ROOTS + 3 * 32 : EVENT_ROOTS + 4 * 32] = right
        data[EVENT_ROOTS + 4 * 32 : EVENT_ROOTS + 5 * 32] = left
        rebuild_from_events(data)

    add("reordered_events_resealed", 7, reordered_events_resealed)

    def trace_resealed(data: bytearray) -> None:
        flip(data, TRACE_ROOT)
        rebuild_result(data)

    add("trace_root_resealed", 7, trace_resealed)
    add("result_root_drift", 7, lambda data: flip(data, RESULT_ROOT))

    receipts = []
    baseline_producer = run_producer(
        producer, cache, output_dir / "baseline.producer.bin"
    )
    if baseline_producer["producer_allocation"] != (
        "allocation_probe_calls=0 allocation_probe_bytes=0"
    ):
        raise RuntimeError("baseline producer allocation probe mismatch")
    if (output_dir / "baseline.producer.bin").read_bytes() != baseline:
        raise RuntimeError("real producer baseline differs from supplied artifact")
    baseline_receipt = run_checker(
        checker, cache, baseline_path, output_dir / "baseline.audit"
    )
    if baseline_receipt["route"] != 0 or baseline_receipt["exit"] != 0:
        raise RuntimeError("baseline checker did not accept")
    receipts.append({"name": "baseline", "expected": 0,
                     **baseline_producer, **baseline_receipt})
    for name, expected, data in cases:
        path = output_dir / f"{name}.bin"
        path.write_bytes(data)
        receipt = run_checker(checker, cache, path, output_dir / f"{name}.audit")
        receipts.append({"name": name, "expected": expected, **receipt})
        if receipt["route"] != expected or receipt["exit"] != 1:
            raise RuntimeError(
                f"{name}: expected route {expected}, got {receipt['route']}"
            )
        if receipt["allocation"] != "allocation_probe_calls=0 allocation_probe_bytes=0":
            raise RuntimeError(f"{name}: allocation probe mismatch")

    artifact_size_cases = (
        ("truncated_header", baseline[:7]),
        ("truncated_selected", baseline[:80]),
        ("truncated", baseline[:-1]),
        ("trailing", baseline + b"\x00"),
        ("oversized", baseline + b"\x00\x00"),
        ("oversized_alt", baseline + b"\x00\x01"),
    )
    for name, data in artifact_size_cases:
        path = output_dir / f"{name}.bin"
        path.write_bytes(data)
        receipt = run_checker(checker, cache, path, output_dir / f"{name}.audit")
        receipts.append({"name": name, "expected": 4, **receipt})
        if receipt["route"] != 4 or receipt["exit"] != 1:
            raise RuntimeError(f"{name}: malformed route missing")

    cache_baseline = bytearray(cache.read_bytes())
    cache_cases: list[tuple[str, int, bytearray]] = []

    def add_cache(name: str, route: int, offset: int, replacement: bytes | None = None) -> None:
        data = bytearray(cache_baseline)
        if replacement is None:
            flip(data, offset)
        else:
            data[offset : offset + len(replacement)] = replacement
        cache_cases.append((name, route, data))

    add_cache("cache_stale_version", 2, 8, (2).to_bytes(4, "little"))
    add_cache("cache_dimension", 2, 90, (103).to_bytes(8, "little"))
    add_cache("cache_sigma", 2, 520528)
    add_cache("cache_inverse", 2, 604680)
    add_cache("cache_projected_scale", 2, 608192)
    add_cache("cache_tangent_component", 2, 6448)
    for name, offset in (
        ("cache_projected_component", 606560),
        ("cache_original_component", 604920),
        ("cache_baseline0_component", 770),
        ("cache_baseline1_component", 2410),
        ("cache_baseline2_component", 4050),
        ("cache_common2_component", 1031019),
    ):
        add_cache(name, 2, offset)
    add_cache("cache_selected_length", 2, 606552, (101).to_bytes(8, "little"))
    add_cache("cache_invalid_early_certificate_bool", 2, 5690, b"\x02")
    add_cache("cache_invalid_profile_bool", 2, 775040, b"\x02")
    add_cache("cache_invalid_late_certificate_bool", 2, 1032659, b"\x02")
    add_cache("cache_invalid_skipped_string_length", 2, 25,
              (1033626).to_bytes(8, "little"))
    add_cache("cache_invalid_early_collection_count", 2, 5682,
              (65).to_bytes(8, "little"))
    add_cache("cache_invalid_profile_vector_length", 2, 775091,
              ((1 << 64) - 1).to_bytes(8, "little"))
    add_cache("cache_invalid_late_collection_count", 2, 1032651,
              (65).to_bytes(8, "little"))
    cache_cases.append(("cache_truncated_header", 1, cache_baseline[:7]))
    cache_cases.append(("cache_truncated_selected", 1, cache_baseline[:606600]))
    for name, expected, data in cache_cases:
        path = output_dir / f"{name}.bin"
        path.write_bytes(data)
        produced = output_dir / f"{name}.producer.bin"
        producer_receipt = run_producer(producer, path, produced)
        if producer_receipt["producer_route"] != expected:
            raise RuntimeError(
                f"{name}: producer expected route {expected}, "
                f"got {producer_receipt['producer_route']}"
            )
        if name.startswith("cache_invalid_") \
                and producer_receipt["producer_first_failure"] != 1:
            raise RuntimeError(
                f"{name}: parser failure was not first-specific"
            )
        receipt = run_checker(
            checker, path, produced, output_dir / f"{name}.audit"
        )
        receipts.append({"name": name, "expected": expected,
                         **producer_receipt, **receipt})
        if receipt["route"] != expected or receipt["exit"] != 0:
            raise RuntimeError(
                f"{name}: expected route {expected}, got {receipt['route']}"
            )
        if producer_receipt["producer_allocation"] != (
            "allocation_probe_calls=0 allocation_probe_bytes=0"
        ):
            raise RuntimeError(f"{name}: producer allocation probe mismatch")

    for name, data in (
        ("cache_truncated", cache_baseline[:-1]),
        ("cache_trailing", cache_baseline + b"\x00"),
        ("cache_oversized", cache_baseline + b"\x00\x00"),
        ("cache_oversized_alt", cache_baseline + b"\x00\x01"),
    ):
        path = output_dir / f"{name}.bin"
        path.write_bytes(data)
        produced = output_dir / f"{name}.producer.bin"
        producer_receipt = run_producer(producer, path, produced)
        receipt = run_checker(
            checker, path, produced, output_dir / f"{name}.audit"
        )
        receipts.append({"name": name, "expected": 1,
                         **producer_receipt, **receipt})
        if receipt["route"] != 1 or receipt["exit"] != 0:
            raise RuntimeError(f"{name}: malformed route missing")

    missing = output_dir / "cache_missing.bin"
    if missing.exists():
        raise RuntimeError("cache_missing control path unexpectedly exists")
    missing_artifact = output_dir / "cache_missing.producer.bin"
    producer_receipt = run_producer(producer, missing, missing_artifact)
    receipt = run_checker(
        checker, missing, missing_artifact, output_dir / "cache_missing.audit"
    )
    receipts.append({"name": "cache_missing", "expected": 1,
                     **producer_receipt, **receipt})
    if producer_receipt["producer_route"] != 1 \
            or receipt["route"] != 1 or receipt["exit"] != 0:
        raise RuntimeError("cache_missing: verified read rejection missing")

    audit_roots = [entry["audit_sha256"] for entry in receipts]
    if len(audit_roots) != len(set(audit_roots)):
        raise RuntimeError("checker audit is not uniquely input-bound")

    summary = {
        "schema": "r63zm-artifact-controls-v3",
        "status": "PASS",
        "controls": len(receipts) - 1,
        "baseline": receipts[0]["checker_root"],
        "routes": {entry["name"]: entry["route"] for entry in receipts[1:]},
        "receipts": receipts,
    }
    print(json.dumps(summary, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
