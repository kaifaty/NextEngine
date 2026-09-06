#!/usr/bin/env python3
"""Focused guards for the M1c historical census and shortlist."""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_historical_census_v0 as census_v0  # noqa: E402


def universe_html() -> bytes:
    rows = []
    for object_id in range(1, 101):
        material = "Glass" if object_id in {6, 59, 92} else "Plastic"
        rows.append(
            f"<tr><td>{object_id}</td><td>Object_{object_id}</td>"
            f"<td>{material}</td></tr>"
        )
    return ("<html><table>" + "".join(rows) + "</table></html>").encode()


def make_store(root: Path, value: object | None = None) -> Path:
    store = root / "store"
    store.mkdir()
    if value is None:
        value = {
            "schema": "nextengine.synthetic-history.v0",
            "study_id": "history",
            "object_id": "6",
            "path": "audio/6/0.wav",
        }
    (store / "history.json").write_bytes(census_v0.canonical_json(value))
    return store


def file_map(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class HistoricalCensusV0Tests(unittest.TestCase):
    def test_universe_requires_exact_identity_and_ids_1_to_100(self) -> None:
        data = universe_html()
        universe = census_v0.universe_from_html(
            data, expected_bytes=len(data), expected_sha256=census_v0.sha256_bytes(data)
        )
        self.assertEqual(len(universe["objects"]), 100)
        self.assertEqual(universe["objects"][91]["object_id"], "92")
        with self.assertRaisesRegex(census_v0.HistoricalCensusError, "identity changed"):
            census_v0.universe_from_html(
                data, expected_bytes=len(data), expected_sha256="0" * 64
            )

    def test_direct_identity_and_path_tokens_are_found(self) -> None:
        value = {
            "object": {"id": 59},
            "rows": [{"dataset_object_id": "92"}],
            "object_ids": [93],
            "path": "prefix/82/audio/0/mic.wav",
        }
        direct = census_v0.direct_object_pointers(value)
        tokens = census_v0.path_token_hits(value)
        self.assertEqual(set(direct), {"59", "92", "93"})
        self.assertEqual({item["object_id"] for item in tokens}, {"82"})

    def test_repeat_build_is_exact_and_referenced_glass_is_excluded(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            store = make_store(root)
            html = universe_html()
            html_path = root / "universe.html"
            html_path.write_bytes(html)
            first = census_v0.execute(
                store,
                html_path,
                root / "result-a",
                expected_universe_bytes=len(html),
                expected_universe_sha256=census_v0.sha256_bytes(html),
            )
            second = census_v0.execute(
                store,
                html_path,
                root / "result-b",
                expected_universe_bytes=len(html),
                expected_universe_sha256=census_v0.sha256_bytes(html),
            )
            first_files = file_map(first)
            second_files = file_map(second)
            shortlist = json.loads(first_files["shortlist.json"])
            report = json.loads(first_files["report.json"])
        self.assertEqual(first_files, second_files)
        self.assertEqual(
            [item["object_id"] for item in shortlist["fresh_candidates"]],
            ["59", "92"],
        )
        self.assertEqual(
            report["decision"], "M1C_HISTORICAL_CENSUS_FRESH_SHORTLIST_PASS"
        )
        self.assertEqual(report["census_access"]["signal_values_decoded"], 0)
        self.assertEqual(report["ledger_build_access"]["source_bytes_read"], 0)

    def test_generated_results_do_not_perturb_census(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            store = make_store(root)
            first = census_v0.scan_store(store)
            generated = store / "physical-sound-v13-m1c.generated"
            generated.mkdir()
            (generated / "report.json").write_bytes(census_v0.canonical_json({"x": 1}))
            second = census_v0.scan_store(store)
        self.assertEqual(first["included"], second["included"])
        self.assertEqual(first["excluded"], second["excluded"])

    def test_duplicate_json_and_symlink_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            store = make_store(root)
            (store / "history.json").write_bytes(b'{"object_id": 6, "object_id": 7}\n')
            with self.assertRaisesRegex(census_v0.HistoricalCensusError, "duplicate"):
                census_v0.scan_store(store)

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            store = make_store(root)
            os.symlink(store / "history.json", store / "linked.json")
            with self.assertRaisesRegex(census_v0.HistoricalCensusError, "symlink"):
                census_v0.scan_store(store)

    def test_output_inside_repository_is_rejected(self) -> None:
        output = census_v0.repository_root() / "forbidden-m1c-output"
        with self.assertRaisesRegex(census_v0.HistoricalCensusError, "outside"):
            census_v0.prepare_output(output)


if __name__ == "__main__":
    unittest.main()
