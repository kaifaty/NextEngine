#!/usr/bin/env python3
"""Focused guards for Physical Sound V15-S0a revision-aware exposure."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_revision_identity_v1 as revision_v1  # noqa: E402
import physical_sound_source_inventory_v1 as inventory_v1  # noqa: E402


REALIMPACT_IDS = (
    3,
    4,
    6,
    9,
    10,
    17,
    19,
    22,
    23,
    27,
    28,
    31,
    32,
    33,
    34,
    35,
    36,
    37,
    43,
    48,
    49,
    50,
    51,
    59,
    60,
    61,
    62,
    63,
    64,
    65,
    67,
    68,
    73,
    74,
    75,
    78,
    79,
    80,
    81,
    83,
    86,
    89,
    90,
    91,
    92,
    93,
    94,
    96,
    97,
    100,
)


def write(path: Path, data: bytes) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return path


def write_json(path: Path, value: object) -> Path:
    return write(path, revision_v1.canonical_json(value))


def historical_rows() -> dict[int, tuple[str, str]]:
    materials = ("Glass", "Wood", "Steel", "Iron")
    return {
        object_id: (f"Object_{object_id}", materials[(object_id - 1) % 4])
        for object_id in range(1, 101)
    }


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


def realimpact_names() -> dict[int, str]:
    result = {object_id: f"{object_id}_Object_{object_id}" for object_id in REALIMPACT_IDS}
    result[92] = "92_MetalSpatula"
    result[93] = "93_GreenGoblet"
    return result


def inventory_row(
    source_id: str,
    publisher_object_id: str,
    publisher_name: str,
    publisher_material: str | None,
    material_family: str | None,
) -> dict[str, object]:
    source_tier = (
        "T3_dense_real_listener"
        if source_id == revision_v1.REALIMPACT_SOURCE
        else "T2_sparse_real_contact"
    )
    return {
        "alias_status": "fixture",
        "candidate_state": (
            "source_ood"
            if source_id == revision_v1.REALIMPACT_SOURCE
            else "metadata_candidate"
        ),
        "exposure": {
            "queried_object_parent_sha256s": [],
            "roles": [],
            "state": "unexposed",
        },
        "inventory_object_id": inventory_v1.inventory_identity(
            source_id, publisher_object_id
        ),
        "material_family": material_family,
        "member_root_sha256": None,
        "publisher_material": publisher_material,
        "publisher_name": publisher_name,
        "publisher_object_id": publisher_object_id,
        "reason_codes": ["fixture"],
        "source_id": source_id,
        "source_tier": source_tier,
        "structural_presence": (
            {"archive_identity_available": True}
            if source_id == revision_v1.REALIMPACT_SOURCE
            else {"remote_archive_identity_available": True}
        ),
    }


def source_inventory(
    rows: dict[int, tuple[str, str]],
    names: dict[int, str],
) -> dict[str, object]:
    objects = []
    for object_id, (name, material) in sorted(rows.items()):
        objects.append(
            inventory_row(
                revision_v1.CURRENT_SOURCE,
                str(object_id),
                name,
                material,
                inventory_v1.MATERIAL_MAP.get(material),
            )
        )
    for object_id, name in sorted(names.items()):
        objects.append(
            inventory_row(
                revision_v1.REALIMPACT_SOURCE,
                name,
                name,
                None,
                None,
            )
        )
    objects.sort(key=lambda item: item["inventory_object_id"])
    return {
        "access": {},
        "alias_edges": [],
        "archives": [],
        "authority": {"public_contract": False, "runtime_consumer_allowed": False},
        "discrepancies": [],
        "input_identities": {},
        "objects": objects,
        "publisher_claims": [],
        "schema": inventory_v1.INVENTORY_SCHEMA,
        "sources": [],
    }


def exclusions(
    direct_ids: tuple[int, ...] = (), path_ids: tuple[int, ...] = ()
) -> dict[str, object]:
    return {
        "direct_exposures": [
            {
                "artifact_id": f"direct-{object_id}",
                "json_pointer": "/object_id",
                "object_id": str(object_id),
                "relative_path": f"prior/direct-{object_id}.json",
            }
            for object_id in direct_ids
        ],
        "path_token_exposures": [
            {
                "artifact_id": f"token-{object_id}",
                "json_pointer": "/path",
                "object_id": str(object_id),
                "relative_path": f"prior/token-{object_id}.json",
                "token": f"audio/{object_id}/",
            }
            for object_id in path_ids
        ],
        "revision": "fixture-exclusions-v0",
        "schema": "nextengine.experimental-physical-sound-object-exclusions.v0",
    }


PAPER_TEXT = """R EAL I MPACT
We introduce a dataset of 150,000 real
object impact sounds. Along with these sounds, metadata follows.
We purchase 50 objects from the ObjectFolder
dataset [19], which is comprised of household objects.
Each object in R EAL I MPACT has a high-
resolution 3D mesh model generated from a scan of the real
object. We select objects which are rigid and consist of a single ho-
mogeneous material belonging to one of the following cat-
egories.
""".encode()


def synthetic_inputs(
    root: Path,
    *,
    current_overrides: dict[int, tuple[str, str]] | None = None,
    direct_ids: tuple[int, ...] = (),
    path_ids: tuple[int, ...] = (),
) -> revision_v1.Inputs:
    historical = historical_rows()
    current = dict(historical)
    if current_overrides:
        current.update(current_overrides)
    names = realimpact_names()
    input_root = root / "inputs"
    return revision_v1.Inputs(
        source_inventory=write_json(
            input_root / "inventory.json", source_inventory(current, names)
        ),
        historical_exclusions=write_json(
            input_root / "exclusions.json", exclusions(direct_ids, path_ids)
        ),
        historical_table=write(
            input_root / "historical.md", markdown_table(historical)
        ),
        realimpact_names=write(
            input_root / "object_names.txt",
            ("\n".join(names.values()) + "\n").encode(),
        ),
        realimpact_paper_text=write(input_root / "paper.txt", PAPER_TEXT),
    )


def load(documents: dict[str, bytes], name: str) -> dict[str, object]:
    return json.loads(documents[name])


def group_with_member(
    identity: dict[str, object], source_id: str, publisher_object_id: str
) -> dict[str, object]:
    return next(
        group
        for group in identity["groups"]
        if any(
            member["source_id"] == source_id
            and member["publisher_object_id"] == publisher_object_id
            for member in group["members"]
        )
    )


def census_group(census: dict[str, object], physical_group_id: str) -> dict[str, object]:
    return next(
        group
        for group in census["groups"]
        if group["physical_group_id"] == physical_group_id
    )


class RevisionIdentityV1Tests(unittest.TestCase):
    def test_repeat_build_is_exact_and_all_signal_counters_are_zero(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(root)
            store = root / "store"
            store.mkdir()
            first = revision_v1.build_documents(
                inputs, store, expected_identities=None
            )
            second = revision_v1.build_documents(
                inputs, store, expected_identities=None
            )
        self.assertEqual(first, second)
        report = load(first, "report.json")
        identity = load(first, "identity-map.json")
        self.assertEqual(
            report["decision"],
            "S0A_IDENTITY_EXPOSURE_PASS_METAL_SCOPE_REMAINS_POTENTIAL",
        )
        self.assertEqual(report["counts"]["physical_group_count"], 100)
        self.assertEqual(len(identity["groups"]), 100)
        for key in (
            "force_sample_values_decoded",
            "network_archive_body_bytes",
            "network_requests",
            "npy_headers_parsed",
            "pcm_sample_values_decoded",
            "protected_signal_values_decoded",
            "source_payload_bytes_read",
            "source_payload_members_extracted",
            "wav_headers_parsed",
        ):
            self.assertEqual(report["access"][key], 0)

    def test_revision_drift_stays_distinct_and_realimpact_joins_historical(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(
                root, current_overrides={92: ("Current_Different_92", "Glass")}
            )
            store = root / "store"
            store.mkdir()
            documents = revision_v1.build_documents(
                inputs, store, expected_identities=None
            )
        identity = load(documents, "identity-map.json")
        historical = group_with_member(
            identity, revision_v1.HISTORICAL_SOURCE, "92"
        )
        current = group_with_member(identity, revision_v1.CURRENT_SOURCE, "92")
        realimpact = group_with_member(
            identity, revision_v1.REALIMPACT_SOURCE, "92_MetalSpatula"
        )
        self.assertNotEqual(
            historical["physical_group_id"], current["physical_group_id"]
        )
        self.assertEqual(
            historical["physical_group_id"], realimpact["physical_group_id"]
        )
        self.assertEqual(len(identity["discrepancies"]), 1)

    def test_exact_realimpact_name_exposes_only_historical_group(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(
                root, current_overrides={93: ("Current_Vase_93", "Glass")}
            )
            store = root / "store"
            write_json(store / "opened.json", {"dataset_object_id": "93_GreenGoblet"})
            documents = revision_v1.build_documents(
                inputs, store, expected_identities=None
            )
        identity = load(documents, "identity-map.json")
        census = load(documents, "exposure-census.json")
        historical = group_with_member(
            identity, revision_v1.REALIMPACT_SOURCE, "93_GreenGoblet"
        )
        current = group_with_member(identity, revision_v1.CURRENT_SOURCE, "93")
        historical_exposure = census_group(
            census, historical["physical_group_id"]
        )
        current_exposure = census_group(census, current["physical_group_id"])
        self.assertEqual(historical_exposure["exposure_state"], "exposed")
        self.assertEqual(
            historical_exposure["exposure_evidence"]["realimpact_name_record_count"],
            1,
        )
        self.assertEqual(current_exposure["exposure_state"], "unexposed")

    def test_source_listing_prefix_is_excluded_from_exposure(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(root)
            store = root / "store"
            write_json(
                store / "physical-sound-v14-n1b-fixture" / "inventory.json",
                {"publisher_object_id": "93_GreenGoblet"},
            )
            documents = revision_v1.build_documents(
                inputs, store, expected_identities=None
            )
        identity = load(documents, "identity-map.json")
        census = load(documents, "exposure-census.json")
        group = group_with_member(
            identity, revision_v1.REALIMPACT_SOURCE, "93_GreenGoblet"
        )
        exposure = census_group(census, group["physical_group_id"])
        self.assertEqual(exposure["exposure_state"], "unexposed")
        self.assertEqual(census["access"]["json_files_parsed"], 0)

    def test_direct_and_path_tokens_expose_both_sides_of_numeric_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(
                root,
                current_overrides={92: ("Current_Different_92", "Glass")},
                direct_ids=(92,),
                path_ids=(92,),
            )
            store = root / "store"
            store.mkdir()
            documents = revision_v1.build_documents(
                inputs, store, expected_identities=None
            )
        identity = load(documents, "identity-map.json")
        census = load(documents, "exposure-census.json")
        groups = [
            group
            for group in identity["groups"]
            if group["numeric_object_ids"] == ["92"]
        ]
        self.assertEqual(len(groups), 2)
        for group in groups:
            exposure = census_group(census, group["physical_group_id"])
            self.assertEqual(exposure["exposure_state"], "exposed")
            self.assertEqual(
                exposure["exposure_evidence"]["direct_numeric_record_count"], 1
            )
            self.assertEqual(
                exposure["exposure_evidence"]["path_token_record_count"], 1
            )

    def test_schema_claim_roster_and_duplicate_json_fail_closed(self) -> None:
        cases = []

        def unknown_row(inputs: revision_v1.Inputs, _store: Path) -> None:
            value = json.loads(inputs.source_inventory.read_bytes())
            value["objects"][0]["unknown"] = True
            inputs.source_inventory.write_bytes(revision_v1.canonical_json(value))

        def paper_claim(inputs: revision_v1.Inputs, _store: Path) -> None:
            inputs.realimpact_paper_text.write_bytes(b"unrelated paper")

        def roster_mismatch(inputs: revision_v1.Inputs, _store: Path) -> None:
            names = inputs.realimpact_names.read_text().splitlines()
            names[0] = "3_DifferentPublisherObject"
            inputs.realimpact_names.write_text("\n".join(names) + "\n")

        def duplicate_json(_inputs: revision_v1.Inputs, store: Path) -> None:
            write(store / "bad.json", b'{"a":1,"a":2}\n')

        cases.extend(
            [
                ("unknown row", unknown_row, "keys changed"),
                ("paper claim", paper_claim, "paper origin/material claim changed"),
                ("roster mismatch", roster_mismatch, "rosters disagree"),
                ("duplicate JSON", duplicate_json, "duplicate JSON key"),
            ]
        )
        for label, mutation, message in cases:
            with self.subTest(label=label), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                inputs = synthetic_inputs(root)
                store = root / "store"
                store.mkdir()
                mutation(inputs, store)
                with self.assertRaisesRegex(revision_v1.RevisionIdentityError, message):
                    revision_v1.build_documents(
                        inputs, store, expected_identities=None
                    )

    def test_symlink_real_hash_and_output_guards_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(root)
            store = root / "store"
            store.mkdir()
            outside = write(root / "outside.json", b"{}")
            (store / "link.json").symlink_to(outside)
            with self.assertRaisesRegex(
                revision_v1.RevisionIdentityError, "contains symlink"
            ):
                revision_v1.build_documents(inputs, store, expected_identities=None)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(root)
            store = root / "store"
            write(store / "oversized.json", b"{}\n")
            with mock.patch.object(revision_v1, "MAX_JSON_BYTES", 2):
                with self.assertRaisesRegex(
                    revision_v1.RevisionIdentityError, "exceeds limit"
                ):
                    revision_v1.build_documents(
                        inputs, store, expected_identities=None
                    )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(root)
            store = root / "store"
            store.mkdir()
            with self.assertRaisesRegex(
                revision_v1.RevisionIdentityError, "identity changed"
            ):
                revision_v1.build_documents(inputs, store)
        with self.assertRaisesRegex(
            revision_v1.RevisionIdentityError, "outside the repository"
        ):
            revision_v1.prepare_output(
                revision_v1.repository_root() / "forbidden-s0a-output"
            )

    def test_publish_requires_fresh_external_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = synthetic_inputs(root)
            store = root / "store"
            store.mkdir()
            documents = revision_v1.build_documents(
                inputs, store, expected_identities=None
            )
            output = revision_v1.publish(root / "output", documents)
            self.assertTrue((output / "report.json").is_file())
            with self.assertRaisesRegex(
                revision_v1.RevisionIdentityError, "refusing to replace"
            ):
                revision_v1.publish(output, documents)


if __name__ == "__main__":
    unittest.main()
