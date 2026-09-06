#!/usr/bin/env python3
"""Focused guards for the Physical Sound V14-N1b source inventory."""

from __future__ import annotations

import io
import json
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_dataset_contract_v1 as contract_v1  # noqa: E402
import physical_sound_source_inventory_v1 as inventory_v1  # noqa: E402


def write(path: Path, data: bytes) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return path


def write_json(path: Path, value: object) -> Path:
    return write(path, inventory_v1.canonical_json(value))


def target_rows() -> dict[int, tuple[str, str]]:
    materials = ("Glass", "Wood", "Steel", "Iron")
    return {
        object_id: (f"Object_{object_id}", materials[(object_id - 1) % len(materials)])
        for object_id in range(1, 101)
    }


def html_table(rows: dict[int, tuple[str, str]]) -> bytes:
    body = "".join(
        f"<tr><td>{object_id}</td><td>{name}</td><td>{material}</td></tr>"
        for object_id, (name, material) in sorted(rows.items())
    )
    return f"<html><table><tbody>{body}</tbody></table></html>".encode()


def markdown_table(rows: dict[int, tuple[str, str]]) -> bytes:
    lines = []
    for start in range(1, 101, 3):
        cells = []
        for object_id in range(start, min(start + 3, 101)):
            name, material = rows[object_id]
            cells.extend((str(object_id), name, material))
        while len(cells) < 9:
            cells.extend(("", "", ""))
        lines.append("| " + " | ".join(cells) + " |")
    return ("\n".join(lines) + "\n").encode()


def tar_bytes(kind: str, object_ids: range = range(1, 101)) -> bytes:
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:gz") as archive:
        for object_id in object_ids:
            if kind == "audio":
                name = f"audio/{object_id}/0.wav"
            elif kind == "contacts":
                name = f"contacts/{object_id}/0.npy"
            else:
                name = f"global_gt_points/{object_id}.npy"
            payload = b"signal bytes must remain opaque"
            member = tarfile.TarInfo(name)
            member.size = len(payload)
            archive.addfile(member, io.BytesIO(payload))
    return output.getvalue()


def synthetic_inputs(root: Path) -> inventory_v1.Inputs:
    rows = target_rows()
    descriptor = {
        "contract_schema": contract_v1.CONTRACT_SCHEMA,
        "policy": {
            "minimum_per_material": {
                material: contract_v1.MINIMUM_PER_MATERIAL
                for material in inventory_v1.MATERIALS
            }
        },
        "public_contract": False,
        "runtime_consumer_allowed": False,
        "schema": contract_v1.DESCRIPTOR_SCHEMA,
    }
    ledger = contract_v1.synthetic_ledger()
    objectfolder2_lines = []
    materials = ("Glass", "Wood", "Steel", "Iron", "Ceramic")
    for object_id in range(1, 1001):
        material = materials[(object_id - 1) % len(materials)]
        objectfolder2_lines.append(
            f"{object_id},Synthetic_{object_id},1.0,{material},https://example.invalid/{object_id}"
        )
    realimpact_names = [
        f"{object_id}_{rows[object_id][0]}" for object_id in range(1, 51)
    ]
    urls = inventory_v1.expected_urls(realimpact_names)
    snapshot = {
        "acquired_at_utc": "2026-09-01T00:00:00Z",
        "entries": [
            {
                "accept_ranges": "bytes",
                "content_length": 1000 + index,
                "error": None,
                "etag": f'"fixture-{index}"',
                "final_url": url,
                "last_modified": "Mon, 01 Sep 2026 00:00:00 GMT",
                "requested_url": url,
                "status": 200,
            }
            for index, url in enumerate(urls)
        ],
        "schema": inventory_v1.HTTP_SCHEMA,
    }
    split = {
        "test": [],
        "train": [[str(object_id), "0"] for object_id in range(1, 101)],
        "val": [],
    }
    scale = {str(object_id): 1.0 for object_id in range(1, 101)}
    return inventory_v1.Inputs(
        descriptor=write_json(root / "descriptor.json", descriptor),
        ledger=write_json(root / "ledger.json", ledger),
        objectfolder_current=write(root / "current.html", html_table(rows)),
        objectfolder_historical=write(root / "historical.md", markdown_table(rows)),
        objectfolder2_csv=write(
            root / "objects.csv", ("\n".join(objectfolder2_lines) + "\n").encode()
        ),
        objectfolder2_readme=write(
            root / "objectfolder2-readme.md",
            b"ObjectFolder has 1,000 objects and a specified force profile.\n",
        ),
        realimpact_names=write(
            root / "object_names.txt", ("\n".join(realimpact_names) + "\n").encode()
        ),
        realimpact_readme=write(
            root / "realimpact-readme.md", b"The raw dataset is not yet released.\n"
        ),
        realimpact_download=write(
            root / "download.sh",
            b"wget http://downloads.cs.stanford.edu/viscam/RealImpact/$line.zip\n",
        ),
        audio=write(root / "audio.tar.gz", tar_bytes("audio")),
        contacts=write(root / "contacts.tar.gz", tar_bytes("contacts")),
        geometry=write(root / "geometry.tar.gz", tar_bytes("geometry")),
        split=write_json(root / "split.json", split),
        scale=write_json(root / "scale.json", scale),
        http_snapshot=write_json(root / "http-snapshot.json", snapshot),
    )


def load_document(documents: dict[str, bytes], name: str) -> dict[str, object]:
    return json.loads(documents[name])


class SourceInventoryV1Tests(unittest.TestCase):
    def test_repeat_build_is_exact_and_signal_counters_stay_zero(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            inputs = synthetic_inputs(Path(temporary))
            first = inventory_v1.build_documents(inputs, expected_identities=None)
            second = inventory_v1.build_documents(inputs, expected_identities=None)
        self.assertEqual(first, second)
        report = load_document(first, "report.json")
        inventory = load_document(first, "inventory.json")
        self.assertEqual(report["decision"], "N1B_METADATA_INVENTORY_PASS_N1C_MEMBER_PREFLIGHT_NEXT")
        self.assertEqual(report["counts"]["object_row_count"], 1150)
        self.assertEqual(report["counts"]["archive_count"], 70)
        self.assertEqual(report["counts"]["compact_split_object_count"], 100)
        self.assertEqual(len(inventory["objects"]), 1150)
        for key in (
            "force_sample_values_decoded",
            "network_archive_body_bytes",
            "network_requests",
            "npy_headers_parsed",
            "pcm_sample_values_decoded",
            "protected_signal_values_decoded",
            "source_payload_members_extracted",
            "wav_headers_parsed",
        ):
            self.assertEqual(report["access"][key], 0)
        self.assertGreater(report["access"]["tar_member_headers_seen"], 0)

    def test_revision_drift_is_not_merged_or_counted_as_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(root)
            rows = target_rows()
            rows[92] = ("Historical_Metal_Object", "Steel")
            inputs.objectfolder_historical.write_bytes(markdown_table(rows))
            documents = inventory_v1.build_documents(inputs, expected_identities=None)
        inventory = load_document(documents, "inventory.json")
        row = next(
            item
            for item in inventory["objects"]
            if item["source_id"] == "objectfolder_real_current"
            and item["publisher_object_id"] == "92"
        )
        self.assertEqual(row["alias_status"], "ambiguous_revision_identity")
        self.assertEqual(row["candidate_state"], "source_ood")
        self.assertIn("ambiguous_revision_identity", row["reason_codes"])
        self.assertFalse(
            any(
                edge["left"] == row["inventory_object_id"]
                or edge["right"] == row["inventory_object_id"]
                for edge in inventory["alias_edges"]
            )
        )

    def test_numeric_realimpact_id_without_exact_name_does_not_alias(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            inputs = synthetic_inputs(Path(temporary))
            names = inputs.realimpact_names.read_text().splitlines()
            names[0] = "1_DifferentObject"
            inputs.realimpact_names.write_text("\n".join(names) + "\n")
            snapshot = json.loads(inputs.http_snapshot.read_bytes())
            old_url = next(
                entry["requested_url"]
                for entry in snapshot["entries"]
                if entry["requested_url"].endswith("/1_Object_1.zip")
            )
            new_url = old_url.replace("1_Object_1.zip", "1_DifferentObject.zip")
            entry = next(item for item in snapshot["entries"] if item["requested_url"] == old_url)
            entry["requested_url"] = new_url
            entry["final_url"] = new_url
            snapshot["entries"].sort(key=lambda item: item["requested_url"])
            inputs.http_snapshot.write_bytes(inventory_v1.canonical_json(snapshot))
            documents = inventory_v1.build_documents(inputs, expected_identities=None)
        inventory = load_document(documents, "inventory.json")
        row = next(
            item
            for item in inventory["objects"]
            if item["source_id"] == "realimpact_fca2"
            and item["publisher_object_id"] == "1_DifferentObject"
        )
        self.assertEqual(row["alias_status"], "ambiguous_revision_identity")
        self.assertEqual(row["candidate_state"], "source_ood")
        self.assertEqual(row["exposure"]["queried_object_parent_sha256s"], [])

    def test_unknown_http_url_redirect_and_archive_member_fail_closed(self) -> None:
        cases: list[tuple[str, object, str]] = []

        def unknown_url(inputs: inventory_v1.Inputs) -> None:
            snapshot = json.loads(inputs.http_snapshot.read_bytes())
            snapshot["entries"][0]["requested_url"] = "https://example.invalid/archive"
            inputs.http_snapshot.write_bytes(inventory_v1.canonical_json(snapshot))

        def redirect_escape(inputs: inventory_v1.Inputs) -> None:
            snapshot = json.loads(inputs.http_snapshot.read_bytes())
            snapshot["entries"][0]["final_url"] = "https://example.invalid/archive"
            inputs.http_snapshot.write_bytes(inventory_v1.canonical_json(snapshot))

        def unknown_member(inputs: inventory_v1.Inputs) -> None:
            output = io.BytesIO()
            with tarfile.open(fileobj=output, mode="w:gz") as archive:
                member = tarfile.TarInfo("audio/1/metadata.yaml")
                member.size = 0
                archive.addfile(member, io.BytesIO())
            inputs.audio.write_bytes(output.getvalue())

        cases.extend(
            [
                ("unknown URL", unknown_url, "URL set changed or duplicated"),
                ("redirect escape", redirect_escape, "redirect escaped"),
                ("unknown member", unknown_member, "unexpected audio member"),
            ]
        )
        for label, mutation, message in cases:
            with self.subTest(label=label), tempfile.TemporaryDirectory() as temporary:
                inputs = synthetic_inputs(Path(temporary))
                mutation(inputs)
                with self.assertRaisesRegex(inventory_v1.SourceInventoryError, message):
                    inventory_v1.build_documents(inputs, expected_identities=None)

    def test_duplicate_json_and_real_identity_mismatch_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            inputs = synthetic_inputs(Path(temporary))
            inputs.descriptor.write_bytes(b'{"schema":"a","schema":"b"}\n')
            with self.assertRaisesRegex(inventory_v1.SourceInventoryError, "duplicate JSON key"):
                inventory_v1.build_documents(inputs, expected_identities=None)
        with tempfile.TemporaryDirectory() as temporary:
            inputs = synthetic_inputs(Path(temporary))
            with self.assertRaisesRegex(inventory_v1.SourceInventoryError, "identity changed"):
                inventory_v1.build_documents(inputs)

    def test_output_must_be_fresh_external_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(root / "inputs")
            documents = inventory_v1.build_documents(inputs, expected_identities=None)
            output = inventory_v1.publish(root / "output", documents)
            self.assertTrue((output / "report.json").is_file())
            with self.assertRaisesRegex(inventory_v1.SourceInventoryError, "refusing to replace"):
                inventory_v1.publish(output, documents)
        with self.assertRaisesRegex(inventory_v1.SourceInventoryError, "outside the repository"):
            inventory_v1.prepare_output(
                inventory_v1.repository_root() / "forbidden-n1b-output"
            )


if __name__ == "__main__":
    unittest.main()
