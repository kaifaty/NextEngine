#!/usr/bin/env python3
"""Focused guards for the V15-S0b YCB capability/cost adapter."""

from __future__ import annotations

import copy
import io
import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_ycb_capability_v1 as ycb_v1  # noqa: E402


def xlsx_bytes(rows: list[tuple[str, str, str, str | None]]) -> bytes:
    strings: list[str] = []

    def shared(value: str) -> int:
        if value not in strings:
            strings.append(value)
        return strings.index(value)

    all_rows = [("ID", "OBJECT", "Primary Material", "Secondary Material"), *rows]
    row_xml = []
    for row_index, row in enumerate(all_rows, start=1):
        cells = []
        for column, value in zip("ABCD", row):
            if value is None:
                continue
            index = shared(value)
            cells.append(f'<c r="{column}{row_index}" t="s"><v>{index}</v></c>')
        row_xml.append(f'<row r="{row_index}">{"".join(cells)}</row>')
    namespace = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
    sheet = (
        f'<?xml version="1.0"?><worksheet xmlns="{namespace}"><sheetData>'
        + "".join(row_xml)
        + "</sheetData></worksheet>"
    ).encode()
    shared_xml = (
        f'<?xml version="1.0"?><sst xmlns="{namespace}">'
        + "".join(f"<si><t>{value}</t></si>" for value in strings)
        + "</sst>"
    ).encode()
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w") as archive:
        archive.writestr("xl/sharedStrings.xml", shared_xml)
        archive.writestr("xl/worksheets/sheet1.xml", sheet)
    return output.getvalue()


def workbook_rows() -> list[tuple[str, str, str, str | None]]:
    names = {index: f"Object {index}" for index in range(1, 76)}
    names.update(
        {
            7: "Tune fish can",
            23: "Wineglass",
            27: "Skillet",
            30: "Fork",
            36: "Wood block",
            39: "Keys",
            43: "Philips screwdriver",
            63: "Marbles",
            70: "Colored wood block",
            71: "Nine-hole peg test",
        }
    )
    materials = {index: "Other Plastic" for index in range(1, 76)}
    for index in (23, 63):
        materials[index] = "Glass"
    for index in (27, 30, 39, 43):
        materials[index] = "Steel"
    for index in (36, 70, 71):
        materials[index] = "Wood"
    rows = [
        (str(index), names[index], materials[index], None) for index in range(1, 76)
    ]
    rows.extend(
        [
            ("x_01", "Generic mug", "Ceramic", None),
            ("x_02", "Trapezoid mug", "Ceramic", None),
        ]
    )
    return rows


def model_index_html(*, contradiction: bool = False) -> bytes:
    rows = []
    for index in range(1, 78):
        names = [f"{index:03d}_object_{index}"]
        if index == 63:
            names = ["063-a_marbles", "063-b_marbles"]
        for name in names:
            processed = (
                "N/A<sup>3</sup>"
                if contradiction and index == 1
                else f'<a href="data/berkeley/{name}/{name}_berkeley_meshes.tgz">Processed</a>'
            )
            rows.append(
                "<tr class='object_row'>"
                f"<td class='name_cell'>{name}</td>"
                f"<td>{processed}</td><td></td><td></td>"
                f'<td><a href="data/google/{name}_google_16k.tgz">16k</a></td>'
                f'<td><a href="data/google/{name}_google_64k.tgz">64k</a></td>'
                f'<td><a href="data/google/{name}_google_512k.tgz">512k</a></td>'
                "</tr>"
            )
    return ("<html><table>" + "".join(rows) + "</table></html>").encode()


def osf_entry(
    entry_id: str,
    kind: str,
    path: str,
    *,
    size: int | None = None,
    parent_id: str | None = "root",
) -> dict[str, object]:
    return {
        "component_id": "bj5w8",
        "current_version": 1,
        "date_created": "2022-03-10T00:00:00.000000",
        "date_modified": "2022-03-10T00:00:00.000000",
        "hashes": {"md5": None, "sha256": None},
        "id": entry_id,
        "kind": kind,
        "materialized_path": path,
        "name": path.rstrip("/").split("/")[-1],
        "parent_id": parent_id,
        "size": size,
    }


def synthetic_snapshot() -> dict[str, object]:
    entries = []
    parents = [
        ("v27", "/Vertical_Pokes/Known_Objects/027_skillet_7/"),
        ("v30", "/Vertical_Pokes/Known_Objects/030_fork_7/"),
        ("v36", "/Vertical_Pokes/Known_Objects/036_wood_block_13/"),
        ("v43", "/Vertical_Pokes/Known_Objects/043_phillips_screwdriver_7/"),
        ("h23", "/Horizontal_Pokes/Glass/Wineglass/"),
    ]
    for parent_id, path in parents:
        entries.append(osf_entry(parent_id, "folder", path))
        entries.append(
            osf_entry(
                f"{parent_id}-clip",
                "file",
                f"{path}Clip_1.ogg",
                size=100 + len(entries),
                parent_id=parent_id,
            )
        )
    entries.sort(key=lambda item: (item["component_id"], item["materialized_path"], item["id"]))
    return {
        "access": {
            "archive_body_bytes": 0,
            "audio_body_bytes": 0,
            "force_body_bytes": 0,
            "mesh_body_bytes": 0,
            "metadata_response_bytes": 12_345,
            "network_requests": 42,
            "payload_members_opened": 0,
            "signal_values_decoded": 0,
            "video_body_bytes": 0,
        },
        "components": [
            {
                "date_modified": modified,
                "id": component_id,
                "title": title,
            }
            for component_id, (title, modified) in sorted(ycb_v1.COMPONENTS.items())
        ],
        "entries": entries,
        "project": {
            "date_modified": ycb_v1.PROJECT_MODIFIED,
            "id": "4tcp6",
            "title": "YCB-impact sounds dataset",
        },
        "schema": ycb_v1.SNAPSHOT_SCHEMA,
    }


def build_fixture(snapshot: dict[str, object] | None = None) -> dict[str, bytes]:
    return ycb_v1.build_documents(
        xlsx_bytes(workbook_rows()),
        model_index_html(),
        b"paper identity verified by caller",
        ycb_v1.canonical_json(snapshot or synthetic_snapshot()),
    )


def osf_node(node_id: str, title: str, modified: str) -> dict[str, object]:
    return {
        "data": {
            "attributes": {"date_modified": modified, "title": title},
            "id": node_id,
            "type": "nodes",
        }
    }


def osf_api_entry(
    entry_id: str,
    kind: str,
    path: str,
    *,
    child_url: str | None = None,
    size: int | None = None,
) -> dict[str, object]:
    relationships: dict[str, object] = {
        "parent_folder": {"data": {"id": "root", "type": "files"}}
    }
    if kind == "folder":
        relationships["files"] = {
            "links": {"related": {"href": child_url, "meta": {}}}
        }
    return {
        "attributes": {
            "current_version": 1,
            "date_created": "2022-03-10T00:00:00.000000",
            "date_modified": "2022-03-10T00:00:00.000000",
            "extra": {"hashes": {"md5": None, "sha256": None}},
            "kind": kind,
            "materialized_path": path,
            "name": path.rstrip("/").split("/")[-1],
            "size": size,
        },
        "id": entry_id,
        "relationships": relationships,
        "type": "files",
    }


class YcbCapabilityV1Tests(unittest.TestCase):
    def test_acquisition_is_repeat_exact_metadata_only(self) -> None:
        robot_root = (
            "https://api.osf.io/v2/nodes/bj5w8/files/osfstorage/?page%5Bsize%5D=100"
        )
        robot_child = (
            "https://api.osf.io/v2/nodes/bj5w8/files/osfstorage/folder/"
            "?page%5Bsize%5D=100"
        )
        manual_root = (
            "https://api.osf.io/v2/nodes/hjdby/files/osfstorage/?page%5Bsize%5D=100"
        )
        responses = {
            "https://api.osf.io/v2/nodes/4tcp6/": osf_node(
                "4tcp6", "YCB-impact sounds dataset", ycb_v1.PROJECT_MODIFIED
            ),
            "https://api.osf.io/v2/nodes/bj5w8/": osf_node(
                "bj5w8", *ycb_v1.COMPONENTS["bj5w8"]
            ),
            "https://api.osf.io/v2/nodes/hjdby/": osf_node(
                "hjdby", *ycb_v1.COMPONENTS["hjdby"]
            ),
            robot_root: {
                "data": [
                    osf_api_entry(
                        "folder",
                        "folder",
                        "/Vertical_Pokes/",
                        child_url=robot_child.replace("?page%5Bsize%5D=100", ""),
                    )
                ],
                "links": {"next": None},
            },
            robot_child: {
                "data": [
                    osf_api_entry(
                        "clip", "file", "/Vertical_Pokes/Clip 1.ogx", size=321
                    )
                ],
                "links": {"next": None},
            },
            manual_root: {"data": [], "links": {"next": None}},
        }

        def fetch(url: str) -> tuple[str, str, bytes]:
            return url, "application/vnd.api+json", ycb_v1.canonical_json(responses[url])

        first = ycb_v1.acquire_osf_snapshot(fetch)
        second = ycb_v1.acquire_osf_snapshot(fetch)
        self.assertEqual(first, second)
        self.assertEqual(first["access"]["network_requests"], 6)
        for key, value in ycb_v1.ZERO_BODY_ACCESS.items():
            self.assertEqual(first["access"][key], value)
        self.assertEqual(len(first["entries"]), 2)

    def test_repeat_build_is_exact_and_signal_access_stays_zero(self) -> None:
        first = build_fixture()
        second = build_fixture()
        self.assertEqual(first, second)
        report = json.loads(first["report.json"])
        inventory = json.loads(first["capability-inventory.json"])
        self.assertEqual(
            report["decision"], "S0B_YCB_CAPABILITY_PASS_S0C_ROLE_FREEZE_NEXT"
        )
        self.assertEqual(report["counts"]["training_usable_count"], 0)
        self.assertEqual(report["counts"]["evaluation_complete_count"], 0)
        self.assertEqual(
            report["inputs"]["osf_snapshot"]["sha256"],
            ycb_v1.sha256_bytes(ycb_v1.canonical_json(synthetic_snapshot())),
        )
        for key, value in report["access"].items():
            if key != "metadata_files_parsed":
                self.assertEqual(value, 0)
        self.assertTrue(all(not item["training_usable"] for item in inventory["objects"]))
        self.assertTrue(all(not item["evaluation_complete"] for item in inventory["objects"]))

    def test_vertical_horizontal_and_alias_bindings_remain_distinct(self) -> None:
        inventory = json.loads(build_fixture()["capability-inventory.json"])
        objects = {item["object_id"]: item for item in inventory["objects"]}
        self.assertEqual(
            objects["23"]["recording_parents"][0]["acquisition_mode"],
            "horizontal-object",
        )
        self.assertEqual(
            objects["27"]["recording_parents"][0]["acquisition_mode"],
            "vertical-known",
        )
        self.assertEqual(objects["43"]["recording_parents"][0]["parent_id"], "v43")
        self.assertNotEqual(
            objects["23"]["recording_parents"][0]["group_id"],
            objects["27"]["recording_parents"][0]["group_id"],
        )

    def test_numeric_only_folder_match_gets_no_object_credit(self) -> None:
        snapshot = synthetic_snapshot()
        for entry in snapshot["entries"]:
            if entry["materialized_path"].startswith(
                "/Vertical_Pokes/Known_Objects/030_fork_7/"
            ):
                entry["materialized_path"] = entry["materialized_path"].replace(
                    "030_fork_7", "030_wrong_object_7"
                )
                entry["name"] = entry["materialized_path"].rstrip("/").split("/")[-1]
        snapshot["entries"].sort(
            key=lambda item: (item["component_id"], item["materialized_path"], item["id"])
        )
        inventory = json.loads(build_fixture(snapshot)["capability-inventory.json"])
        fork = next(item for item in inventory["objects"] if item["object_id"] == "30")
        self.assertEqual(fork["recording_parents"], [])

    def test_variant_mesh_is_ambiguous_and_never_grants_geometry(self) -> None:
        inventory = json.loads(build_fixture()["capability-inventory.json"])
        marbles = next(item for item in inventory["objects"] if item["object_id"] == "63")
        self.assertEqual(marbles["geometry"]["state"], "variant_ambiguous")
        self.assertEqual(marbles["capability_axes"]["geometry"], "unknown")
        self.assertFalse(marbles["training_usable"])

    def test_forbidden_snapshot_access_and_duplicate_path_fail_closed(self) -> None:
        leaked = synthetic_snapshot()
        leaked["access"]["audio_body_bytes"] = 1
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "forbidden payload"):
            build_fixture(leaked)

        oversized = synthetic_snapshot()
        oversized["access"]["network_requests"] = ycb_v1.MAX_REQUESTS + 1
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "acquisition limits"):
            build_fixture(oversized)

        duplicated = synthetic_snapshot()
        clone = copy.deepcopy(duplicated["entries"][0])
        clone["id"] = "another-id"
        duplicated["entries"].append(clone)
        duplicated["entries"].sort(
            key=lambda item: (item["component_id"], item["materialized_path"], item["id"])
        )
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "duplicate OSF entry id/path"):
            build_fixture(duplicated)

    def test_noncanonical_and_duplicate_json_fail_closed(self) -> None:
        snapshot = synthetic_snapshot()
        noncanonical = json.dumps(snapshot, separators=(",", ":")).encode()
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "not canonical"):
            ycb_v1.build_documents(
                xlsx_bytes(workbook_rows()), model_index_html(), b"paper", noncanonical
            )
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "duplicate JSON key"):
            ycb_v1.parse_json_bytes(b'{"schema":"a","schema":"b"}', "fixture")

    def test_workbook_duplicate_and_model_contradiction_fail_closed(self) -> None:
        rows = workbook_rows()
        rows[-1] = ("1", "Duplicate", "Steel", None)
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "duplicate YCB workbook ID"):
            ycb_v1.parse_workbook(xlsx_bytes(rows))
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "absent route as distorted"):
            ycb_v1.parse_model_index(model_index_html(contradiction=True))

    def test_osf_host_relationship_and_path_escape_fail_closed(self) -> None:
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "escaped allowed host"):
            ycb_v1.validate_osf_url("https://example.invalid/v2/nodes/bj5w8/")
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "escaped component"):
            ycb_v1.validate_osf_url(
                "https://api.osf.io/v2/nodes/hjdby/files/osfstorage/", "bj5w8"
            )
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "traversal"):
            ycb_v1.normalized_materialized_path("/safe/../escape/", "fixture")

    def test_output_is_external_fresh_and_never_replaced(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = ycb_v1.publish_directory(root / "result", build_fixture())
            self.assertTrue((output / "report.json").is_file())
            with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "refusing to replace"):
                ycb_v1.publish_directory(output, build_fixture())
        with self.assertRaisesRegex(ycb_v1.YcbCapabilityError, "outside the repository"):
            ycb_v1.ensure_external_fresh_path(
                ycb_v1.repository_root() / "forbidden-ycb-output", "fixture"
            )

    def test_real_source_identities_match_frozen_protocol(self) -> None:
        root = Path(
            "/home/kaifaty/.codex/experiments/nextengine/physical-sound/"
            "physical-sound-v14-n1c-source-research.xzOKNX"
        )
        paper = Path(
            "/home/kaifaty/.codex/experiments/nextengine/physical-sound/"
            "physical-sound-v15-s0b-research.L75yqv/ycb-impact-paper.pdf"
        )
        if not root.is_dir() or not paper.is_file():
            self.skipTest("external frozen S0b research inputs are unavailable")
        self.assertEqual(
            ycb_v1.hash_regular_file(root / "ycb_audio.xlsx", "workbook"),
            ycb_v1.WORKBOOK_IDENTITY,
        )
        model_index = paper.parent / "ycb-model-index.html"
        self.assertEqual(
            ycb_v1.hash_regular_file(model_index, "model index"),
            ycb_v1.MODEL_INDEX_IDENTITY,
        )
        self.assertEqual(ycb_v1.hash_regular_file(paper, "paper"), ycb_v1.PAPER_IDENTITY)


if __name__ == "__main__":
    unittest.main()
