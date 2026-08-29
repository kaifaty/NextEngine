#!/usr/bin/env python3
"""Deterministic adversarial controls for the frozen R63ZP revision-6 package."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


CANDIDATE_BYTES = 6176
CHECKER_BYTES = 1176
CANDIDATE_WORK_FIELDS = 64
CHECKER_WORK_FIELDS = 71
EVENT_SLOTS = 8

C_ROUTE = 16
C_FLAGS = 20
C_PARENTS = 24
C_ROLES = 120
C_ROOTS = 184
C_SCALARS = 280
C_WORK = 376
C_WORK_ROOT = 888
C_EVENT_COUNT = 920
C_EVENTS = 928
C_TRACE = 1184
C_P0 = 1216
C_Q0 = 2848
C_BOUNDS = 4480
C_BODY = 6112
C_RESULT = 6144

K_ROUTE = 16
K_FLAGS = 20
K_PARENTS = 24
K_CANDIDATE_ROOTS = 120
K_WORK = 248
K_WORK_ROOT = 816
K_EVENT_COUNT = 848
K_EVENTS = 856
K_TRACE = 1112
K_RESULT = 1144

KW_HEADER_PREDICATES = 9
KW_DYADIC_DECODES = 39
KW_EXACT_MULTIPLIES = 40
KW_POSTSEAL_X1_DECODES = 50

VECTOR_DOMAIN = b"nextengine.nonlocal.r63zp.quad-vector.v1"
CANDIDATE_WORK_DOMAIN = b"nextengine.nonlocal.r63zp.candidate-work.v1"
BODY_DOMAIN = b"nextengine.nonlocal.r63zp.product-body.v1"
CANDIDATE_EVENT_DOMAIN = b"nextengine.nonlocal.r63zp.candidate-event.v1"
CANDIDATE_TRACE_DOMAIN = b"nextengine.nonlocal.r63zp.candidate-trace.v1"
CANDIDATE_RESULT_DOMAIN = b"nextengine.nonlocal.r63zp.candidate-result.v1"
CHECKER_WORK_DOMAIN = b"nextengine.nonlocal.r63zp.checker-work.v1"
CHECKER_RESULT_DOMAIN = b"nextengine.nonlocal.r63zp.checker-result.v1"

ALLOCATION_STDERR = b"allocation_probe_calls=0 allocation_probe_bytes=0\n"
EXACT_PRODUCT_ROOT = bytes.fromhex(
    "271facfdbbb97c777d1a661cf73f5a1eaa25853d42f9e520cab9c785af272f2f"
)


def u32(value: int) -> bytes:
    return value.to_bytes(4, "big")


def u64(value: int) -> bytes:
    return value.to_bytes(8, "big")


def take_u32(data: bytes | bytearray, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "big")


def take_u64(data: bytes | bytearray, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 8], "big")


def field(tag: int, payload: bytes | bytearray) -> bytes:
    return bytes((tag,)) + u64(len(payload)) + bytes(payload)


def digest(*parts: bytes) -> bytes:
    sha = hashlib.sha256()
    for part in parts:
        sha.update(part)
    return sha.digest()


def vector_root(values: bytes | bytearray) -> bytes:
    return digest(field(1, VECTOR_DOMAIN), field(3, u64(102)), field(8, values))


def candidate_work_root(data: bytes | bytearray) -> bytes:
    route = take_u32(data, C_ROUTE)
    work = data[C_WORK : C_WORK + CANDIDATE_WORK_FIELDS * 8]
    return digest(
        field(1, CANDIDATE_WORK_DOMAIN), field(2, u32(route)), field(6, work)
    )


def candidate_body_root(data: bytes | bytearray) -> bytes:
    parts = [
        field(1, BODY_DOMAIN),
        field(7, data[C_PARENTS : C_PARENTS + 96]),
        field(7, data[C_ROLES : C_ROLES + 64]),
        field(7, data[C_ROOTS : C_ROOTS + 96]),
    ]
    parts.extend(field(4, data[C_SCALARS + 16 * i : C_SCALARS + 16 * (i + 1)])
                 for i in range(5))
    parts.extend(
        [
            field(3, u64(102)),
            field(2, u32((take_u32(data, C_FLAGS) >> 8) & 0xFF)),
            field(6, data[C_WORK : C_WORK + 42 * 8]),
            field(7, data[C_EVENTS : C_EVENTS + 4 * 32]),
            field(8, data[C_P0 : C_P0 + 102 * 16]),
            field(8, data[C_Q0 : C_Q0 + 102 * 16]),
            field(8, data[C_BOUNDS : C_BOUNDS + 102 * 16]),
        ]
    )
    return digest(*parts)


def candidate_event_root(data: bytes | bytearray, ordinal: int) -> bytes:
    route = take_u32(data, C_ROUTE) if ordinal >= 7 else 7
    parts = [field(1, CANDIDATE_EVENT_DOMAIN), field(2, u32(ordinal)),
             field(2, u32(route))]
    if ordinal == 1:
        parts.extend(
            [
                field(2, u32((take_u32(data, C_FLAGS) >> 8) & 0xFF)),
                field(7, data[C_PARENTS : C_PARENTS + 96]),
            ]
        )
    elif ordinal == 2:
        parts.extend(
            [
                field(5, data[C_ROLES : C_ROLES + 32]),
                # Event 2's derived-x0 root equals the admitted role-2 input root.
                field(5, data[C_ROLES : C_ROLES + 32]),
                field(5, data[C_ROLES : C_ROLES + 32]),
            ]
        )
    elif ordinal == 3:
        parts.append(field(7, data[C_ROLES : C_ROLES + 64]))
    elif ordinal == 4:
        parts.extend(
            [
                field(5, data[C_ROOTS : C_ROOTS + 32]),
                field(4, data[C_SCALARS : C_SCALARS + 16]),
                field(4, data[C_SCALARS + 16 : C_SCALARS + 32]),
            ]
        )
    elif ordinal == 5:
        parts.append(field(5, data[C_BODY : C_BODY + 32]))
    elif ordinal == 6:
        parts.extend(
            [
                field(5, data[C_ROOTS + 32 : C_ROOTS + 64]),
                field(5, data[C_ROOTS + 64 : C_ROOTS + 96]),
                field(4, data[C_SCALARS + 32 : C_SCALARS + 48]),
                field(4, data[C_SCALARS + 48 : C_SCALARS + 64]),
                field(4, data[C_SCALARS + 64 : C_SCALARS + 80]),
            ]
        )
    elif ordinal == 8:
        parts.extend(
            [
                field(5, data[C_BODY : C_BODY + 32]),
                field(5, data[C_WORK_ROOT : C_WORK_ROOT + 32]),
            ]
        )
    else:
        raise ValueError("event 7 is deliberately preserved: x1 root is not duplicated")
    return digest(*parts)


def candidate_trace_root(data: bytes | bytearray) -> bytes:
    count = take_u64(data, C_EVENT_COUNT)
    events = data[C_EVENTS : C_EVENTS + EVENT_SLOTS * 32]
    return digest(
        field(1, CANDIDATE_TRACE_DOMAIN), field(3, u64(count)), field(7, events)
    )


def candidate_result_root(data: bytes | bytearray) -> bytes:
    route = take_u32(data, C_ROUTE)
    flags = take_u32(data, C_FLAGS)
    count = take_u64(data, C_EVENT_COUNT)
    parts = [
        field(1, CANDIDATE_RESULT_DOMAIN),
        field(2, u32(route)),
        field(2, u32(flags)),
        field(7, data[C_PARENTS : C_PARENTS + 96]),
        field(7, data[C_ROLES : C_ROLES + 64]),
        field(7, data[C_ROOTS : C_ROOTS + 96]),
    ]
    parts.extend(field(4, data[C_SCALARS + 16 * i : C_SCALARS + 16 * (i + 1)])
                 for i in range(5))
    parts.extend(
        [
            field(3, u64(102)),
            field(5, data[C_WORK_ROOT : C_WORK_ROOT + 32]),
            field(3, u64(count)),
            field(5, data[C_TRACE : C_TRACE + 32]),
            field(5, data[C_BODY : C_BODY + 32]),
            field(3, u64(102)),
        ]
    )
    return digest(*parts)


def rebuild_candidate(data: bytearray) -> None:
    data[C_WORK_ROOT : C_WORK_ROOT + 32] = candidate_work_root(data)
    for ordinal in range(1, 5):
        start = C_EVENTS + (ordinal - 1) * 32
        data[start : start + 32] = candidate_event_root(data, ordinal)
    data[C_BODY : C_BODY + 32] = candidate_body_root(data)
    for ordinal in range(5, 7):
        start = C_EVENTS + (ordinal - 1) * 32
        data[start : start + 32] = candidate_event_root(data, ordinal)
    data[C_EVENTS + 7 * 32 : C_EVENTS + 8 * 32] = candidate_event_root(data, 8)
    rebuild_candidate_terminal(data)


def rebuild_candidate_terminal(data: bytearray) -> None:
    data[C_TRACE : C_TRACE + 32] = candidate_trace_root(data)
    data[C_RESULT : C_RESULT + 32] = candidate_result_root(data)


def checker_work_root(data: bytes | bytearray) -> bytes:
    route = take_u32(data, K_ROUTE)
    work = data[K_WORK : K_WORK + CHECKER_WORK_FIELDS * 8]
    return digest(
        field(1, CHECKER_WORK_DOMAIN), field(2, u32(route)), field(6, work)
    )


def checker_result_root(data: bytes | bytearray) -> bytes:
    route = take_u32(data, K_ROUTE)
    flags = take_u32(data, K_FLAGS)
    count = take_u64(data, K_EVENT_COUNT)
    return digest(
        field(1, CHECKER_RESULT_DOMAIN),
        field(2, u32(route)),
        field(2, u32(flags)),
        field(7, data[K_PARENTS : K_PARENTS + 96]),
        field(7, data[K_CANDIDATE_ROOTS : K_CANDIDATE_ROOTS + 128]),
        field(5, data[K_WORK_ROOT : K_WORK_ROOT + 32]),
        field(3, u64(count)),
        field(5, data[K_TRACE : K_TRACE + 32]),
    )


def rebuild_checker_work(data: bytearray) -> None:
    data[K_WORK_ROOT : K_WORK_ROOT + 32] = checker_work_root(data)
    data[K_RESULT : K_RESULT + 32] = checker_result_root(data)


def sha256(data: bytes | bytearray) -> str:
    return hashlib.sha256(data).hexdigest()


def execute(arguments: list[str], expected: set[int]) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(arguments, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            check=False)
    if result.returncode not in expected:
        raise RuntimeError(
            f"unexpected exit {result.returncode}: {' '.join(arguments)}\n"
            f"stdout={result.stdout!r}\nstderr={result.stderr!r}"
        )
    if result.stderr != ALLOCATION_STDERR:
        raise RuntimeError(f"allocation probe mismatch: {result.stderr!r}")
    if result.stdout:
        raise RuntimeError(f"unexpected stdout: {result.stdout!r}")
    return result


def write(path: Path, data: bytes | bytearray) -> None:
    path.write_bytes(data)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--producer", required=True)
    parser.add_argument("--checker", required=True)
    parser.add_argument("--cache", required=True)
    parser.add_argument("--parent", required=True)
    parser.add_argument("--parent-audit", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    cache = Path(args.cache)
    parent = Path(args.parent)
    parent_audit = Path(args.parent_audit)
    controls: list[dict[str, object]] = []
    audit_hashes: set[str] = set()

    def record(name: str, kind: str, route: int, artifact: bytes) -> None:
        identity = sha256(artifact)
        if identity in audit_hashes:
            raise RuntimeError(f"duplicate audit identity for {name}: {identity}")
        audit_hashes.add(identity)
        controls.append({"name": name, "kind": kind, "route": route,
                         "audit_sha256": identity})

    baseline_candidates: list[bytes] = []
    baseline_audits: list[bytes] = []
    for repeat in range(2):
        candidate_path = output / f"baseline-{repeat}.candidate"
        audit_path = output / f"baseline-{repeat}.audit"
        execute([args.producer, str(cache), str(parent), str(parent_audit),
                 str(candidate_path)], {0})
        execute([args.checker, str(cache), str(parent), str(parent_audit),
                 str(candidate_path), str(audit_path)], {0})
        candidate = candidate_path.read_bytes()
        audit = audit_path.read_bytes()
        if len(candidate) != CANDIDATE_BYTES or take_u32(candidate, C_ROUTE) != 7:
            raise RuntimeError("baseline candidate mismatch")
        if len(audit) != CHECKER_BYTES or take_u32(audit, K_ROUTE) != 7:
            raise RuntimeError("baseline audit mismatch")
        if audit[K_CANDIDATE_ROOTS + 96 : K_CANDIDATE_ROOTS + 128] \
                != EXACT_PRODUCT_ROOT:
            raise RuntimeError("exact product root mismatch")
        baseline_candidates.append(candidate)
        baseline_audits.append(audit)
    if baseline_candidates[0] != baseline_candidates[1] \
            or baseline_audits[0] != baseline_audits[1]:
        raise RuntimeError("baseline repeats are not byte-identical")
    baseline = baseline_candidates[0]
    baseline_audit = baseline_audits[0]

    exact = subprocess.run([args.checker, "--exact-controls"],
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                           check=False)
    if exact.returncode != 0 or exact.stderr != ALLOCATION_STDERR:
        raise RuntimeError(
            f"exact controls failed: rc={exact.returncode} "
            f"stdout={exact.stdout!r} stderr={exact.stderr!r}"
        )
    exact_report = json.loads(exact.stdout)
    for key in ("zero_valid", "negative_valid", "nonfinite_rejected",
                "alignment_valid", "capacity_rejected"):
        if exact_report.get(key) is not True:
            raise RuntimeError(f"exact control {key} did not pass")
    for key in ("dyadic_decodes", "exact_multiplies", "exact_additions",
                "alignment_shifts", "alignment_bits",
                "capacity_predicates"):
        if not isinstance(exact_report.get(key), int) or exact_report[key] <= 0:
            raise RuntimeError(f"exact control counter {key} is not positive")
    if exact_report.get("package_allocations") != 0:
        raise RuntimeError("exact controls allocated package memory")

    selector_routes = {1: 2, 2: 3, 3: 4, 4: 4, 5: 5, 6: 6}
    selector_events = {1: 2, 2: 5, 3: 6, 4: 6, 5: 6, 6: 8}
    for selector, expected_route in selector_routes.items():
        name = f"selector-{selector}-route-{expected_route}"
        candidate_path = output / f"{name}.candidate"
        audit_path = output / f"{name}.audit"
        execute([args.producer, str(cache), str(parent), str(parent_audit),
                 str(candidate_path), str(selector)], {1})
        execute([args.checker, str(cache), str(parent), str(parent_audit),
                 str(candidate_path), str(audit_path)], {0})
        candidate = candidate_path.read_bytes()
        audit = audit_path.read_bytes()
        if take_u32(candidate, C_ROUTE) != expected_route:
            raise RuntimeError(f"{name}: candidate route mismatch")
        if take_u64(candidate, C_EVENT_COUNT) != selector_events[selector]:
            raise RuntimeError(f"{name}: candidate event count mismatch")
        if take_u32(audit, K_ROUTE) != expected_route:
            raise RuntimeError(f"{name}: checker route mismatch")
        dyadic = take_u64(audit, K_WORK + KW_DYADIC_DECODES * 8)
        future = take_u64(audit, K_WORK + KW_POSTSEAL_X1_DECODES * 8)
        if selector == 1 and (dyadic != 0 or future != 0):
            raise RuntimeError("selector 1 crossed product/oracle boundary")
        if 2 <= selector <= 5 and (dyadic == 0 or future != 0):
            raise RuntimeError(f"selector {selector} causal work mismatch")
        if selector == 6 and (dyadic == 0 or future != 102):
            raise RuntimeError("selector 6 did not reach only the late x1 stage")
        record(name, "named-route", expected_route, audit)

    def check_candidate(name: str, candidate: bytes | bytearray,
                        expected_route: int) -> bytes:
        candidate_path = output / f"{name}.candidate"
        audit_path = output / f"{name}.audit"
        write(candidate_path, candidate)
        execute([args.checker, str(cache), str(parent), str(parent_audit),
                 str(candidate_path), str(audit_path)], {1})
        audit = audit_path.read_bytes()
        route = take_u32(audit, K_ROUTE)
        if route != expected_route:
            raise RuntimeError(f"{name}: route {route}, expected {expected_route}")
        record(name, "candidate", route, audit)
        return audit

    header_mutations = {
        "magic": (0, 0x01),
        "version": (11, 0x01),
        "size": (15, 0x01),
        "unknown-route": (19, 0x08),
        "reserved-flags": (20, 0x80),
        "unknown-selector": (22, 0x07),
        "work-count": (375, 0x01),
    }
    early_audits: list[bytes] = []
    for name, (offset, mask) in header_mutations.items():
        mutated = bytearray(baseline)
        mutated[offset] ^= mask
        early_audits.append(check_candidate(f"header-{name}", mutated, 8))

    for name, candidate in (
        ("candidate-truncated", baseline[:-1]),
        ("candidate-trailing", baseline + b"\0"),
    ):
        early_audits.append(check_candidate(name, candidate, 8))

    baseline_work = [take_u64(baseline, C_WORK + i * 8)
                     for i in range(CANDIDATE_WORK_FIELDS)]
    for index in range(CANDIDATE_WORK_FIELDS):
        mutated = bytearray(baseline)
        mutated[C_WORK + index * 8 : C_WORK + (index + 1) * 8] = u64(
            baseline_work[index] + 1
        )
        rebuild_candidate(mutated)
        check_candidate(f"candidate-work-{index:02d}", mutated, 10)

    semantic_offsets = {
        "parent-root": C_PARENTS,
        "role-root": C_ROLES,
        "p0-root": C_ROOTS,
        "q0-root": C_ROOTS + 32,
        "bound-root": C_ROOTS + 64,
        "rho-primary": C_SCALARS,
        "rho-bound": C_SCALARS + 16,
        "denominator-primary": C_SCALARS + 32,
        "denominator-bound": C_SCALARS + 48,
        "alpha": C_SCALARS + 64,
        "p0-component": C_P0,
        "q0-component": C_Q0,
        "bound-component": C_BOUNDS,
    }
    for name, offset in semantic_offsets.items():
        mutated = bytearray(baseline)
        mutated[offset] ^= 0x01
        if name == "p0-component":
            mutated[C_ROOTS : C_ROOTS + 32] = vector_root(
                mutated[C_P0 : C_P0 + 102 * 16]
            )
        elif name == "q0-component":
            mutated[C_ROOTS + 32 : C_ROOTS + 64] = vector_root(
                mutated[C_Q0 : C_Q0 + 102 * 16]
            )
        elif name == "bound-component":
            mutated[C_ROOTS + 64 : C_ROOTS + 96] = vector_root(
                mutated[C_BOUNDS : C_BOUNDS + 102 * 16]
            )
        rebuild_candidate(mutated)
        check_candidate(f"semantic-{name}", mutated, 9)

    mutated_flags = bytearray(baseline)
    mutated_flags[C_FLAGS + 3] ^= 0x20
    rebuild_candidate(mutated_flags)
    check_candidate("semantic-flags", mutated_flags, 9)

    for index in range(EVENT_SLOTS):
        mutated = bytearray(baseline)
        mutated[C_EVENTS + index * 32] ^= 0x01
        if index < 4:
            mutated[C_BODY : C_BODY + 32] = candidate_body_root(mutated)
            mutated[C_EVENTS + 4 * 32 : C_EVENTS + 5 * 32] = \
                candidate_event_root(mutated, 5)
            mutated[C_EVENTS + 7 * 32 : C_EVENTS + 8 * 32] = \
                candidate_event_root(mutated, 8)
        rebuild_candidate_terminal(mutated)
        check_candidate(f"event-{index + 1}", mutated, 11)

    event2 = bytearray(baseline)
    event2[C_EVENTS + 32] ^= 0x02
    event2[C_BODY : C_BODY + 32] = candidate_body_root(event2)
    event2[C_EVENTS + 4 * 32 : C_EVENTS + 5 * 32] = \
        candidate_event_root(event2, 5)
    event2[C_EVENTS + 7 * 32 : C_EVENTS + 8 * 32] = \
        candidate_event_root(event2, 8)
    rebuild_candidate_terminal(event2)
    event2_audit = check_candidate("causal-event-2-downstream-resealed",
                                   event2, 11)

    body = bytearray(baseline)
    body[C_BODY] ^= 0x02
    body[C_EVENTS + 4 * 32 : C_EVENTS + 5 * 32] = \
        candidate_event_root(body, 5)
    body[C_EVENTS + 7 * 32 : C_EVENTS + 8 * 32] = \
        candidate_event_root(body, 8)
    rebuild_candidate_terminal(body)
    body_audit = check_candidate("causal-body-event-5-downstream-resealed",
                                 body, 11)

    event6 = bytearray(baseline)
    event6[C_EVENTS + 5 * 32] ^= 0x02
    rebuild_candidate_terminal(event6)
    event6_audit = check_candidate("causal-event-6-downstream-resealed",
                                   event6, 11)

    for name, audit in (("event-2", event2_audit), ("body", body_audit)):
        if take_u64(audit, K_WORK + KW_DYADIC_DECODES * 8) != 0 \
                or take_u64(audit, K_WORK + KW_POSTSEAL_X1_DECODES * 8) != 0:
            raise RuntimeError(f"causal {name} crossed oracle/x1 boundary")
    if take_u64(event6_audit, K_WORK + KW_DYADIC_DECODES * 8) == 0 \
            or take_u64(event6_audit,
                        K_WORK + KW_POSTSEAL_X1_DECODES * 8) != 0:
        raise RuntimeError("causal event-6 did not stop between oracle and x1")

    for name, offset in (
        ("work-root", C_WORK_ROOT),
        ("body-root", C_BODY),
        ("trace-root", C_TRACE),
        ("result-root", C_RESULT),
    ):
        mutated = bytearray(baseline)
        mutated[offset] ^= 0x01
        check_candidate(f"seal-{name}", mutated, 12)

    # Early paths must report actually executed work, not the success ledger.
    baseline_checker_work = [take_u64(baseline_audit, K_WORK + i * 8)
                             for i in range(CHECKER_WORK_FIELDS)]
    for index, audit in enumerate(early_audits):
        early_work = [take_u64(audit, K_WORK + i * 8)
                      for i in range(CHECKER_WORK_FIELDS)]
        if early_work == baseline_checker_work:
            raise RuntimeError(f"early audit {index} copied baseline work")
        if any(early_work[i] != 0 for i in range(15, 58)):
            raise RuntimeError(f"early audit {index} executed semantic replay")
    if take_u64(early_audits[0], K_WORK + KW_HEADER_PREDICATES * 8) != 9:
        raise RuntimeError("first header mismatch did not execute exact fixed predicates")
    if take_u64(early_audits[-2], K_WORK + KW_HEADER_PREDICATES * 8) != 0:
        raise RuntimeError("truncated candidate claimed header predicates")

    # Input identity/length controls exercise independently verified route 1.
    input_cases: list[tuple[str, Path, bytes]] = []
    for label, path in (("cache", cache), ("parent", parent),
                        ("parent-audit", parent_audit)):
        source = path.read_bytes()
        same = bytearray(source)
        same[len(same) // 2] ^= 0x01
        input_cases.extend(
            [
                (f"{label}-same-size", path, bytes(same)),
                (f"{label}-truncated", path, source[:-1]),
                (f"{label}-trailing", path, source + b"\0"),
            ]
        )
    for name, original_path, payload in input_cases:
        changed = output / f"{name}.input"
        write(changed, payload)
        selected = {
            cache: cache,
            parent: parent,
            parent_audit: parent_audit,
        }
        selected[original_path] = changed
        candidate_path = output / f"{name}.candidate"
        audit_path = output / f"{name}.audit"
        execute([args.producer, str(selected[cache]), str(selected[parent]),
                 str(selected[parent_audit]), str(candidate_path)], {1})
        candidate = candidate_path.read_bytes()
        if len(candidate) != CANDIDATE_BYTES or take_u32(candidate, C_ROUTE) != 1:
            raise RuntimeError(f"{name}: producer did not seal route 1")
        execute([args.checker, str(selected[cache]), str(selected[parent]),
                 str(selected[parent_audit]), str(candidate_path), str(audit_path)],
                {0})
        audit = audit_path.read_bytes()
        route = take_u32(audit, K_ROUTE)
        if route != 1:
            raise RuntimeError(f"{name}: checker route {route}, expected 1")
        if take_u64(audit, K_WORK + KW_DYADIC_DECODES * 8) != 0 \
                or take_u64(audit, K_WORK + KW_POSTSEAL_X1_DECODES * 8) != 0:
            raise RuntimeError(f"{name}: route 1 crossed causal boundary")
        record(name, "input", route, audit)

    # A checker audit is not authoritative merely because it is self-sealed.
    # Mutate every checker-work field, reseal the two dependent roots, and
    # require the external control oracle to detect the deviation from replay.
    for index in range(CHECKER_WORK_FIELDS):
        mutated = bytearray(baseline_audit)
        start = K_WORK + index * 8
        mutated[start : start + 8] = u64(baseline_checker_work[index] + 1)
        rebuild_checker_work(mutated)
        if mutated[K_WORK_ROOT : K_WORK_ROOT + 32] != checker_work_root(mutated):
            raise RuntimeError("checker work reseal failed")
        if mutated[K_RESULT : K_RESULT + 32] != checker_result_root(mutated):
            raise RuntimeError("checker result reseal failed")
        observed = [take_u64(mutated, K_WORK + i * 8)
                    for i in range(CHECKER_WORK_FIELDS)]
        differences = [i for i, pair in enumerate(zip(observed,
                       baseline_checker_work)) if pair[0] != pair[1]]
        if differences != [index]:
            raise RuntimeError(f"checker work control {index} not isolated")
        path = output / f"checker-work-{index:02d}.audit"
        write(path, mutated)
        record(f"checker-work-{index:02d}", "checker-audit", 10, mutated)

    report = {
        "schema": "nextengine.nonlocal.r63zp.controls.v1",
        "candidate_revision": 6,
        "candidate_sha256": sha256(baseline),
        "checker_audit_sha256": sha256(baseline_audit),
        "exact_product_root": EXACT_PRODUCT_ROOT.hex(),
        "controls_passed": len(controls) + 1,
        "unique_negative_audits": len(audit_hashes),
        "exact_controls": exact_report,
        "controls": controls,
    }
    report_path = output / "report.json"
    report_path.write_text(json.dumps(report, sort_keys=True, separators=(",", ":"))
                           + "\n", encoding="utf-8")
    print(json.dumps({key: report[key] for key in (
        "schema", "candidate_revision", "candidate_sha256",
        "checker_audit_sha256", "exact_product_root", "controls_passed",
        "unique_negative_audits")}, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
