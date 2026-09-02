from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v29_q0m_inventory_v1 as q0m  # noqa: E402


YCB_PARENT_IDS = (
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    13,
    19,
    20,
    21,
    24,
    25,
    27,
    29,
    30,
    31,
    32,
    33,
    36,
    37,
    38,
    41,
    42,
    43,
    48,
    51,
    52,
    58,
    59,
    60,
    61,
    68,
    70,
    71,
)
LEGACY_YCB_IDS = (
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    11,
    13,
    19,
    20,
    24,
    25,
    27,
    29,
    36,
    38,
    41,
    48,
    51,
    60,
    61,
    68,
    70,
    71,
)
LEGACY_OBJECTFOLDER_IDS = (6, 36, 80, 94, 97)


def canonical(value: object) -> bytes:
    return q0m.canonical_json(value)


def objectfolder_fixture() -> bytes:
    materials = (
        ["Steel"] * 17
        + ["Iron"] * 15
        + ["Ceramic"] * 19
        + ["Wood"] * 18
        + ["Plastic"] * 12
        + ["Glass"] * 11
        + ["Polycarbonate"] * 8
    )
    cells = "".join(
        f"<tr><td>{object_id}</td><td>Object_{object_id}</td><td>{material}</td></tr>"
        for object_id, material in enumerate(materials, 1)
    )
    return f"<html><table>{cells}</table></html>".encode()


def ycb_material(object_id: int | str) -> str:
    if object_id in {27, 30, 31, 32, 37, 38, 42, 43, 48}:
        return "Steel"
    if object_id in {2, 5, 7, 10}:
        return "Aluminium"
    if object_id in {36, 70, 71}:
        return "Wood"
    return "Ceramic"


def ycb_fixture(*, remove_second_project_steel: bool = False) -> dict[str, object]:
    objects = []
    for object_id in [str(value) for value in range(1, 76)] + ["x_01", "x_02"]:
        numeric = int(object_id) if object_id.isdigit() else object_id
        material = ycb_material(numeric)
        if remove_second_project_steel and material == "Steel":
            material = "Ceramic"
        has_parent = isinstance(numeric, int) and numeric in YCB_PARENT_IDS
        axes = {
            "canonical_excitation": "partial" if has_parent else "unknown",
            "contact_geometry_binding": "unknown",
            "geometry": "partial" if has_parent else "unknown",
            "geometry_scale": "unknown",
            "listener_condition": "partial" if has_parent else "unknown",
            "material_identity": "known",
            "object_identity": "known",
            "recorded_response": "partial" if has_parent else "unknown",
            "support_condition": "partial" if has_parent else "unknown",
        }
        missing = [axis for axis, state in axes.items() if state != "known"]
        parents = []
        if has_parent:
            parents.append(
                {
                    "acquisition_mode": "vertical-known",
                    "conditions": ["top-poke-speed-unknown"],
                    "cost": {
                        "declared_bytes": 1000,
                        "file_count": 1,
                        "unknown_size_file_count": 0,
                    },
                    "group_id": f"ycb-group-{numeric:03}",
                    "materialized_path": f"/Vertical/{numeric:03}/",
                    "parent_id": f"parent-{numeric:03}",
                }
            )
        objects.append(
            {
                "capability_axes": axes,
                "evaluation_complete": False,
                "exposure_state": "not_assessed_freshness",
                "geometry": {
                    "exact_payload_identity": False,
                    "metric_scale_known": False,
                    "route_count": 0,
                    "state": "route_absent",
                    "variants": [],
                },
                "material_family": "Metal" if material == "Steel" else None,
                "missing_evaluation_axes": missing,
                "missing_training_axes": missing,
                "object_id": object_id,
                "object_name": f"YCB object {object_id}",
                "primary_material": material,
                "recording_parents": parents,
                "secondary_material": None,
                "training_usable": False,
            }
        )
    return {
        "acquisition_facts": {"mode": "metadata-only"},
        "objects": objects,
        "project": {
            "component_revision": "2022-09-27T11:36:47.167853",
            "id": "ycb-impact-sounds",
            "publisher": "iri-csic-upc-ctu",
        },
        "schema": q0m.YCB_SCHEMA,
    }


def legacy_fixture() -> dict[str, object]:
    assignments: list[dict[str, str]] = []
    for object_id in LEGACY_OBJECTFOLDER_IDS:
        assignments.append(
            {
                "corpus_role": "target" if object_id in {6, 94} else "reject_parent",
                "partition": "calibration",
                "source_id": f"objectfolder-real-demo-{object_id}",
            }
        )
    for object_id in LEGACY_YCB_IDS:
        assignments.append(
            {
                "corpus_role": "reject_parent",
                "partition": "dev",
                "source_id": f"ycb-impact-vertical-object-{object_id:03}-fixture",
            }
        )
    for index in range(34):
        assignments.append(
            {
                "corpus_role": "target" if index == 0 else "reject_parent",
                "partition": ("dev", "calibration", "holdout", "shadow")[index % 4],
                "source_id": f"other-frozen-source-{index:02}",
            }
        )
    assignments.sort(key=lambda row: row["source_id"])
    return {
        "assignments": assignments,
        "corpus_id": "av-msf-ycb-heller-freesound-objectfolder-ycb-vertical-kronland-soundpacks-explicit-e3",
        "corpus_plan_report": {"path": "plan.json", "sha256": "a" * 64},
        "internet_source_manifest": {"path": "source.json", "sha256": "b" * 64},
        "revision": "v3-project-disjoint-split-frozen",
        "schema": q0m.LEGACY_SCHEMA,
        "target_material_label": "Glass",
    }


def build_fixture(*, remove_second_project_steel: bool = False) -> dict[str, bytes]:
    return q0m.build_documents(
        objectfolder_fixture(),
        canonical(ycb_fixture(remove_second_project_steel=remove_second_project_steel)),
        canonical(legacy_fixture()),
    )


class Q0MInventoryTests(unittest.TestCase):
    def test_repeat_exact_feasible_inventory_has_zero_signal_access(self) -> None:
        first = build_fixture()
        second = build_fixture()
        self.assertEqual(first, second)
        report = json.loads(first["report.json"])
        self.assertEqual(
            report["decision"], "Q0M_METAL_SOURCE_INVENTORY_FEASIBLE_Q1M_NEXT"
        )
        self.assertGreaterEqual(report["available_counts"]["exact_steel_candidates"], 16)
        self.assertGreaterEqual(report["available_counts"]["non_metal_candidates"], 35)
        self.assertTrue(
            all(value == 0 for key, value in report["access"].items() if key != "metadata_bytes_read")
        )

    def test_material_labels_are_preserved_and_not_pooled(self) -> None:
        inventory = json.loads(build_fixture()["metal-source-inventory.json"])
        relations = {
            group["primary_material"]: group["material_relation"]
            for group in inventory["groups"]
        }
        self.assertEqual(relations["Steel"], "exact_steel_candidate")
        self.assertEqual(relations["Iron"], "other_metal_candidate")
        self.assertEqual(relations["Aluminium"], "other_metal_candidate")
        self.assertEqual(relations["Ceramic"], "non_metal_candidate")

    def test_historical_metal_reject_is_excluded_without_relabelling(self) -> None:
        inventory = json.loads(build_fixture()["metal-source-inventory.json"])
        row = next(
            group
            for group in inventory["groups"]
            if group["publisher_id"] == "iri-csic-upc-ctu" and group["object_id"] == "27"
        )
        self.assertEqual(row["primary_material"], "Steel")
        self.assertEqual(row["material_relation"], "exact_steel_candidate")
        self.assertFalse(row["candidate_available_after_historical_exclusion"])
        self.assertEqual(row["historical_exclusion"]["historical_role"], "reject_parent")

    def test_one_project_exact_steel_concentration_is_insufficient(self) -> None:
        report = json.loads(
            build_fixture(remove_second_project_steel=True)["report.json"]
        )
        self.assertEqual(report["project_revision_counts"]["exact_steel_candidates"], 1)
        self.assertEqual(
            report["decision"],
            "Q0M_SOURCE_IDENTITY_INSUFFICIENT_SOURCE_GROWTH_REQUIRED",
        )

    def test_objectfolder_duplicate_and_unknown_material_reject(self) -> None:
        source = objectfolder_fixture()
        duplicate = source.replace(
            b"</table>", b"<tr><td>1</td><td>Again</td><td>Steel</td></tr></table>"
        )
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "duplicate ObjectFolder"):
            q0m.parse_objectfolder(duplicate)
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "unknown ObjectFolder material"):
            q0m.parse_objectfolder(source.replace(b"Ceramic", b"Titanium", 1))

    def test_ycb_project_role_and_aggregate_mutations_reject(self) -> None:
        source = ycb_fixture()
        changed = copy.deepcopy(source)
        changed["project"]["component_revision"] = "substituted"
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "project identity"):
            q0m.validate_ycb(canonical(changed))
        changed = copy.deepcopy(source)
        changed["objects"][1]["training_usable"] = True
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "cannot be role usable"):
            q0m.validate_ycb(canonical(changed))
        changed = copy.deepcopy(source)
        changed["objects"][1]["recording_parents"][0]["acquisition_mode"] = "horizontal-aggregate"
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "aggregate YCB parent"):
            q0m.validate_ycb(canonical(changed))

    def test_unknown_ycb_field_and_material_reject(self) -> None:
        source = ycb_fixture()
        source["objects"][0]["pcm_samples"] = [0.0]
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "unknown or missing fields"):
            q0m.validate_ycb(canonical(source))
        source = ycb_fixture()
        source["objects"][0]["primary_material"] = "Titanium"
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "unknown YCB"):
            q0m.validate_ycb(canonical(source))

    def test_duplicate_json_key_rejects(self) -> None:
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "duplicate JSON key"):
            q0m.parse_json(b'{"schema":"a","schema":"b"}', "fixture")

    def test_legacy_target_role_and_duplicate_mutations_reject(self) -> None:
        source = legacy_fixture()
        source["target_material_label"] = "Metal"
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "identity changed"):
            q0m.validate_legacy(canonical(source))
        source = legacy_fixture()
        source["assignments"][1]["source_id"] = source["assignments"][0]["source_id"]
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "duplicate legacy"):
            q0m.validate_legacy(canonical(source))

    def test_symlink_repository_output_and_replacement_guards(self) -> None:
        with tempfile.TemporaryDirectory(prefix="q0m-test-") as temporary:
            root = Path(temporary)
            source = root / "source.json"
            source.write_bytes(b"{}")
            link = root / "link.json"
            link.symlink_to(source)
            with self.assertRaisesRegex(q0m.Q0MInventoryError, "symlink"):
                q0m.read_input(link, "fixture", (2, q0m.sha256_bytes(b"{}")))
            output = root / "output"
            output.mkdir()
            with self.assertRaisesRegex(q0m.Q0MInventoryError, "refusing to replace"):
                q0m.publish(output, {"report.json": b"{}"})
        repository_output = q0m.repository_root() / "forbidden-q0m-output"
        with self.assertRaisesRegex(q0m.Q0MInventoryError, "outside the repository"):
            q0m.ensure_external_path(repository_output, "fixture", True)

    def test_input_identity_drift_rejects_before_parse(self) -> None:
        with tempfile.TemporaryDirectory(prefix="q0m-test-") as temporary:
            source = Path(temporary) / "source.json"
            source.write_bytes(b"{}")
            with self.assertRaisesRegex(q0m.Q0MInventoryError, "identity changed"):
                q0m.read_input(source, "fixture", (2, "0" * 64))


if __name__ == "__main__":
    unittest.main()
