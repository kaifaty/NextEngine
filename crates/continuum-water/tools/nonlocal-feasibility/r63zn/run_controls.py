#!/usr/bin/env python3
"""External R63ZN control runner; never candidate or checker authority."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
from collections.abc import Sequence


CACHE_BYTES = 1_033_625
ARTIFACT_BYTES = 12_916
AUDIT_BYTES = 420
RECEIPT_BYTES = 1_368
CHECKER_AUDIT_BYTES = 676
WORK_FIELDS = 67
EXPECTED_CACHE = "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84"
EXPECTED_ARTIFACT = "ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87"
EXPECTED_AUDIT = "fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80"
ALLOCATION_STDERR = b"allocation_probe_calls=0 allocation_probe_bytes=0\n"


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha(path: pathlib.Path) -> str:
    return sha(path.read_bytes())


def route(path: pathlib.Path, expected_size: int, expected_magic: bytes) -> int:
    data = path.read_bytes()
    if len(data) != expected_size or data[:8] != expected_magic:
        raise RuntimeError(f"malformed output: {path.name}")
    return int.from_bytes(data[20:24], "big")


def invoke(arguments: Sequence[str], expected_exit: int,
           expected_stderr: bytes) -> None:
    completed = subprocess.run(arguments, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, check=False)
    if completed.returncode != expected_exit:
        raise RuntimeError(
            f"unexpected exit {completed.returncode}, expected {expected_exit}: "
            + " ".join(arguments)
        )
    if completed.stdout:
        raise RuntimeError(f"unexpected stdout: {' '.join(arguments)}")
    if completed.stderr != expected_stderr:
        raise RuntimeError(
            f"unexpected stderr {completed.stderr!r}: {' '.join(arguments)}"
        )


def mutate(source: bytes, offset: int, replacement: bytes | None = None) -> bytes:
    data = bytearray(source)
    if replacement is None:
        data[offset] ^= 0x01
    else:
        data[offset:offset + len(replacement)] = replacement
    return bytes(data)


class Runner:
    def __init__(self, args: argparse.Namespace) -> None:
        self.candidate = args.candidate.resolve()
        self.checker = args.checker.resolve()
        self.mutator = args.mutator.resolve()
        self.cache_path = args.cache.resolve()
        self.artifact_path = args.artifact.resolve()
        self.audit_path = args.audit.resolve()
        self.output = args.output.resolve()
        self.output.mkdir(parents=True, exist_ok=False)
        self.cases = self.output / "cases"
        self.inputs = self.output / "inputs"
        self.cases.mkdir()
        self.inputs.mkdir()
        self.cache = self.cache_path.read_bytes()
        self.artifact = self.artifact_path.read_bytes()
        self.audit = self.audit_path.read_bytes()
        if len(self.cache) != CACHE_BYTES or sha(self.cache) != EXPECTED_CACHE:
            raise RuntimeError("cache identity mismatch")
        if len(self.artifact) != ARTIFACT_BYTES \
                or sha(self.artifact) != EXPECTED_ARTIFACT:
            raise RuntimeError("artifact identity mismatch")
        if len(self.audit) != AUDIT_BYTES or sha(self.audit) != EXPECTED_AUDIT:
            raise RuntimeError("audit identity mismatch")
        self.records: list[dict[str, object]] = []

    def candidate_run(self, name: str, cache: pathlib.Path,
                      artifact: pathlib.Path, audit: pathlib.Path,
                      selector: str | None = None) -> pathlib.Path:
        receipt = self.cases / f"{name}.receipt"
        command = [str(self.candidate), str(cache), str(artifact), str(audit),
                   str(receipt)]
        if selector is not None:
            command.append(selector)
        invoke(command, 0, ALLOCATION_STDERR)
        return receipt

    def checker_run(self, name: str, cache: pathlib.Path,
                    artifact: pathlib.Path, audit: pathlib.Path,
                    receipt: pathlib.Path, expected_exit: int,
                    expected_route: int,
                    selector: str | None = None) -> pathlib.Path:
        checker_audit = self.cases / f"{name}.audit"
        command = [str(self.checker), str(cache), str(artifact), str(audit),
                   str(receipt), str(checker_audit)]
        if selector is not None:
            command.append(selector)
        invoke(command, expected_exit, ALLOCATION_STDERR)
        actual_route = route(checker_audit, CHECKER_AUDIT_BYTES, b"NER63ZQ1")
        if actual_route != expected_route:
            raise RuntimeError(
                f"{name}: checker route {actual_route}, expected {expected_route}"
            )
        return checker_audit

    def record(self, name: str, category: str, receipt: pathlib.Path,
               checker_audit: pathlib.Path, candidate_route: int,
               checker_route: int, selector: str | None = None,
               mutated_input: pathlib.Path | None = None,
               validate_receipt: bool = True) -> None:
        if validate_receipt:
            actual_candidate_route = route(receipt, RECEIPT_BYTES, b"NER63ZN1")
            if actual_candidate_route != candidate_route:
                raise RuntimeError(
                    f"{name}: candidate route {actual_candidate_route}, "
                    f"expected {candidate_route}"
                )
        record: dict[str, object] = {
            "name": name,
            "category": category,
            "candidate_route": candidate_route,
            "checker_route": checker_route,
            "receipt_sha256": file_sha(receipt),
            "checker_audit_sha256": file_sha(checker_audit),
        }
        if selector is not None:
            record["selector"] = selector
        if mutated_input is not None:
            record["mutated_input_sha256"] = file_sha(mutated_input)
            record["mutated_input_bytes"] = mutated_input.stat().st_size
        self.records.append(record)

    def accepted_case(self, name: str, category: str,
                      selector: str | None = None,
                      candidate_route: int = 7,
                      checker_route: int = 0) -> pathlib.Path:
        receipt = self.candidate_run(name, self.cache_path, self.artifact_path,
                                     self.audit_path, selector)
        checker_audit = self.checker_run(
            name, self.cache_path, self.artifact_path, self.audit_path,
            receipt, 0, checker_route, selector
        )
        self.record(name, category, receipt, checker_audit, candidate_route,
                    checker_route, selector)
        return receipt

    def input_case(self, name: str, category: str, kind: str, data: bytes,
                   candidate_route: int, checker_route: int) -> None:
        extension = {"cache": "cache", "artifact": "artifact",
                     "audit": "parent-audit"}[kind]
        changed = self.inputs / f"{name}.{extension}.bin"
        changed.write_bytes(data)
        cache = changed if kind == "cache" else self.cache_path
        artifact = changed if kind == "artifact" else self.artifact_path
        audit = changed if kind == "audit" else self.audit_path
        receipt = self.candidate_run(name, cache, artifact, audit)
        checker_audit = self.checker_run(
            name, cache, artifact, audit, receipt, 0, checker_route
        )
        self.record(name, category, receipt, checker_audit, candidate_route,
                    checker_route, mutated_input=changed)

    def missing_case(self, name: str, kind: str, candidate_route: int,
                     checker_route: int) -> None:
        missing = self.inputs / f"{name}.missing"
        cache = missing if kind == "cache" else self.cache_path
        artifact = missing if kind == "artifact" else self.artifact_path
        audit = missing if kind == "audit" else self.audit_path
        receipt = self.candidate_run(name, cache, artifact, audit)
        checker_audit = self.checker_run(
            name, cache, artifact, audit, receipt, 0, checker_route
        )
        self.record(name, "input-read", receipt, checker_audit,
                    candidate_route, checker_route)

    def receipt_case(self, name: str, command: Sequence[str],
                     expected_checker_route: int, category: str) -> None:
        baseline_name = f"{name}-source"
        baseline = self.candidate_run(
            baseline_name, self.cache_path, self.artifact_path, self.audit_path
        )
        mutant = self.cases / f"{name}.mutant.receipt"
        invoke([str(self.mutator), str(baseline), str(mutant), *command], 0, b"")
        checker_audit = self.checker_run(
            name, self.cache_path, self.artifact_path, self.audit_path,
            mutant, 1, expected_checker_route
        )
        malformed = command[0] == "malformed"
        candidate_route = 7 if malformed else route(
            mutant, RECEIPT_BYTES, b"NER63ZN1"
        )
        self.record(name, category, mutant, checker_audit, candidate_route,
                    expected_checker_route, validate_receipt=not malformed)

    def wrong_selector_case(self, name: str, candidate_selector: str,
                            checker_selector: str | None) -> None:
        receipt = self.candidate_run(
            name, self.cache_path, self.artifact_path, self.audit_path,
            candidate_selector
        )
        checker_audit = self.checker_run(
            name, self.cache_path, self.artifact_path, self.audit_path,
            receipt, 1, 11, checker_selector
        )
        self.record(name, "selector-binding", receipt, checker_audit,
                    route(receipt, RECEIPT_BYTES, b"NER63ZN1"), 11,
                    candidate_selector)

    def run(self) -> dict[str, object]:
        self.accepted_case("baseline", "baseline")
        for name, kind, data, candidate_route, checker_route in (
            ("cache-short", "cache", self.cache[:-1], 0, 1),
            ("cache-trailing", "cache", self.cache + b"\x00", 0, 1),
            ("cache-same-size", "cache", mutate(self.cache, 100), 3, 4),
            ("artifact-short", "artifact", self.artifact[:-1], 1, 2),
            ("artifact-trailing", "artifact", self.artifact + b"\x00", 1, 2),
            ("artifact-same-size", "artifact", mutate(self.artifact, 100), 1, 2),
            ("audit-short", "audit", self.audit[:-1], 2, 3),
            ("audit-trailing", "audit", self.audit + b"\x00", 2, 3),
            ("audit-same-size", "audit", mutate(self.audit, 100), 2, 3),
        ):
            self.input_case(name, "input-shape", kind, data,
                            candidate_route, checker_route)
        self.missing_case("cache-missing", "cache", 0, 1)
        self.missing_case("artifact-missing", "artifact", 1, 2)
        self.missing_case("audit-missing", "audit", 2, 3)

        cache_controls = (
            ("cache-malformed-header", mutate(self.cache, 0)),
            ("cache-selected-length", mutate(
                self.cache, 762, (101).to_bytes(8, "little"))),
            ("cache-invalid-boolean", mutate(self.cache, 5690, b"\x02")),
            ("cache-factor-component", mutate(self.cache, 520_624)),
            ("cache-permutation-component", mutate(self.cache, 603_864)),
            ("cache-inverse-scale", mutate(self.cache, 604_680)),
            ("cache-original-rhs", mutate(self.cache, 604_920)),
            ("cache-baseline0", mutate(self.cache, 770)),
        )
        for name, data in cache_controls:
            self.input_case(name, "cache-typed-field", "cache", data, 3, 4)

        role2 = 1_236 + 2 * 1_944
        artifact_controls = (
            ("artifact-route", mutate(self.artifact, 63)),
            ("artifact-role2-exact", mutate(self.artifact, role2, b"\x00")),
            ("artifact-role2-underflow", mutate(
                self.artifact, role2 + 1, b"\x00")),
            ("artifact-role2-role", mutate(self.artifact, role2 + 2, b"\x03")),
            ("artifact-role2-count", mutate(
                self.artifact, role2 + 304, (101).to_bytes(8, "big"))),
            ("artifact-role2-value", mutate(self.artifact, role2 + 312)),
            ("artifact-role2-value-root", mutate(
                self.artifact, role2 + 144 + 64)),
            ("artifact-terminal-result", mutate(self.artifact, 484)),
        )
        for name, data in artifact_controls:
            self.input_case(name, "parent-typed-field", "artifact", data, 1, 2)

        for name, selector, candidate_route, checker_route in (
            ("x0-mismatch", "r63zn-x0-mismatch-v1", 4, 5),
            ("prefix-underflow", "r63zn-prefix-underflow-v1", 5, 6),
            ("rho-nonpositive", "r63zn-rho-nonpositive-v1", 6, 7),
            ("small-solve", "r63zn-small-solve-v1", 7, 0),
            ("state-difference", "r63zn-state-difference-v1", 7, 0),
        ):
            self.accepted_case(name, "entrypoint-control", selector,
                               candidate_route, checker_route)

        for index in range(6):
            self.receipt_case(f"semantic-root-{index}", ["root", str(index)],
                              11, "resealed-semantic")
        for index in range(WORK_FIELDS):
            self.receipt_case(f"work-{index}", ["work", str(index)],
                              12, "resealed-work")
        for command in ("event-delete", "event-duplicate", "event-reorder"):
            self.receipt_case(command, [command], 13, "resealed-event")
        self.receipt_case("route-drift", ["route"], 11, "resealed-route")
        self.receipt_case("malformed-receipt", ["malformed"], 9,
                          "malformed-receipt")
        self.receipt_case("seal-drift", ["seal"], 14, "seal-drift")

        self.wrong_selector_case("selector-x0-as-underflow",
                                 "r63zn-x0-mismatch-v1",
                                 "r63zn-prefix-underflow-v1")
        self.wrong_selector_case("selector-underflow-as-rho",
                                 "r63zn-prefix-underflow-v1",
                                 "r63zn-rho-nonpositive-v1")
        self.wrong_selector_case("selector-rho-as-baseline",
                                 "r63zn-rho-nonpositive-v1", None)

        audit_hashes = [str(record["checker_audit_sha256"])
                        for record in self.records]
        if len(set(audit_hashes)) != len(audit_hashes):
            raise RuntimeError("checker audits are not unique")
        return {
            "schema": "nextengine.nonlocal.r63zn.controls.v1",
            "candidate_sha256": file_sha(self.candidate),
            "checker_sha256": file_sha(self.checker),
            "mutator_sha256": file_sha(self.mutator),
            "cache_sha256": EXPECTED_CACHE,
            "artifact_sha256": EXPECTED_ARTIFACT,
            "parent_audit_sha256": EXPECTED_AUDIT,
            "control_count": len(self.records),
            "unique_checker_audits": len(set(audit_hashes)),
            "controls": self.records,
        }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate", required=True, type=pathlib.Path)
    parser.add_argument("--checker", required=True, type=pathlib.Path)
    parser.add_argument("--mutator", required=True, type=pathlib.Path)
    parser.add_argument("--cache", required=True, type=pathlib.Path)
    parser.add_argument("--artifact", required=True, type=pathlib.Path)
    parser.add_argument("--audit", required=True, type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = Runner(args).run()
    serialized = json.dumps(report, indent=2, sort_keys=True) + "\n"
    report_path = args.output.resolve() / "report.json"
    report_path.write_text(serialized, encoding="utf-8")
    sys.stdout.write(json.dumps({
        "control_count": report["control_count"],
        "report_sha256": sha(serialized.encode("utf-8")),
        "unique_checker_audits": report["unique_checker_audits"],
    }, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
