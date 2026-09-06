from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v43_d2_c0r_identity_repair_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v43-d2-c0r-identity-repair.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v43_d2_c0r_identity_repair_v1 as d2  # noqa: E402


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def bound(data: bytes) -> dict[str, object]:
    return {"bytes": len(data), "sha256": sha256(data)}


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def write(root: Path, relative: str, data: bytes) -> None:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def content_binding(root: Path, kind: str, record_id: str) -> dict[str, object]:
    suffix = ".wav" if kind == "pcm" else ".json"
    data = f"{kind}:{record_id}".encode()
    digest = sha256(data)
    relative = f"objects/{kind}/{digest[:2]}/{digest}{suffix}"
    write(root, relative, data)
    return {**bound(data), "path": relative}


def make_record(
    root: Path,
    record_id: str,
    role: str,
    family: str,
    object_id: str,
    parent: str,
    material: str,
    source_kind: str = "identified_recording",
) -> dict[str, object]:
    return {
        "acoustic_target": content_binding(root, "features", record_id),
        "axis_mask": {"material_identity": True, "waveform": True},
        "canonical_pcm": content_binding(root, "pcm", record_id),
        "family_component_id": f"family:{family}",
        "family_id": family,
        "material_label": material,
        "object_id": object_id,
        "physical_parent_id": parent,
        "provenance": {
            "project_id": f"project:{family}",
            "publisher_id": f"publisher:{family}",
        },
        "record_id": record_id,
        "role": role,
        "source_kind": source_kind,
    }


def append_parent_records(
    root: Path,
    records: list[dict[str, object]],
    role: str,
    family: str,
    counts: list[int],
    material: str,
) -> None:
    for parent_index, count in enumerate(counts):
        parent = f"{family}-parent-{parent_index}"
        object_id = f"{family}-object-{parent_index}"
        for recording_index in range(count):
            record_id = f"{family}-{parent_index}-{recording_index}"
            records.append(
                make_record(
                    root,
                    record_id,
                    role,
                    family,
                    object_id,
                    parent,
                    material,
                )
            )


def build_records(root: Path) -> list[dict[str, object]]:
    train: list[dict[str, object]] = []
    development: list[dict[str, object]] = []
    validator: list[dict[str, object]] = []
    bowl_parent = "realimpact-6-bowl--objectfolder-real-object-6"

    for index in range(4):
        train.append(
            make_record(
                root,
                f"ri-6-{index}",
                d2.TRAIN_ROLE,
                d2.RI_FAMILY,
                "6",
                bowl_parent,
                "Glass",
                "force_deconvolved_transfer",
            )
        )
    for object_id, parent, material in (
        ("6", bowl_parent, "Glass"),
        ("80", "of-parent-80", "Wood"),
    ):
        for index in range(3):
            train.append(
                make_record(
                    root,
                    f"of-{object_id}-{index}",
                    d2.TRAIN_ROLE,
                    d2.OF_FAMILY,
                    object_id,
                    parent,
                    material,
                )
            )
    append_parent_records(root, train, d2.TRAIN_ROLE, d2.OF_FAMILY, [3, 3, 3], "Glass")
    append_parent_records(
        root, train, d2.TRAIN_ROLE, "train-a", [2] * 4 + [1] * 17, "Metal"
    )

    for object_id, material in (("6", "Glass"), ("80", "Ceramic")):
        for index in range(2):
            validator.append(
                make_record(
                    root,
                    f"av-{object_id}-{index}",
                    d2.VALIDATOR_ROLE,
                    d2.AV_FAMILY,
                    object_id,
                    f"av-parent-{object_id}",
                    material,
                )
            )
    append_parent_records(
        root,
        validator,
        d2.VALIDATOR_ROLE,
        d2.AV_FAMILY,
        [2] * 8,
        "Ceramic",
    )
    append_parent_records(
        root,
        validator,
        d2.VALIDATOR_ROLE,
        "validator-a",
        [5],
        "Glass",
    )
    append_parent_records(
        root,
        development,
        d2.DEVELOPMENT_ROLE,
        "development-a",
        [3] * 10 + [2] * 20,
        "Wood",
    )

    assert len(train) == 44
    assert len(development) == 70
    assert len(validator) == 25
    assert len({row["physical_parent_id"] for row in train}) == 26
    assert len({row["physical_parent_id"] for row in development}) == 30
    assert len({row["physical_parent_id"] for row in validator}) == 11
    return sorted([*train, *development, *validator], key=lambda row: row["record_id"])


def projection(role: str, records: list[dict[str, object]]) -> bytes:
    rows = [{"record_id": row["record_id"]} for row in records if row["role"] == role]
    return d2.canonical_json(
        {
            "record_count": len(rows),
            "role": role,
            "rows": rows,
            "rows_root_sha256": sha256(d2.canonical_json(rows)),
            "schema": d2.C0_PROJECTION_SCHEMA,
        }
    )


def d1_roster() -> bytes:
    assignments = {
        d2.RI_FAMILY: d2.TRAIN_ROLE,
        d2.OF_FAMILY: d2.TRAIN_ROLE,
        d2.AV_FAMILY: d2.VALIDATOR_ROLE,
        "train-a": d2.TRAIN_ROLE,
        "train-b": d2.TRAIN_ROLE,
        "train-c": d2.TRAIN_ROLE,
        "development-a": d2.DEVELOPMENT_ROLE,
        "development-b": d2.DEVELOPMENT_ROLE,
        "validator-a": d2.VALIDATOR_ROLE,
    }
    families = [
        {
            "family_commitment_sha256": sha256(f"commit:{family}".encode()),
            "family_component_id": f"family:{family}",
            "family_id": family,
            "role": role,
        }
        for family, role in sorted(assignments.items())
    ]
    return d2.canonical_json(
        {
            "families": families,
            "roster_root_sha256": "2" * 64,
            "schema": d2.D1_SCHEMA,
        }
    )


def c1a_files() -> dict[str, bytes]:
    aliases = [
        {
            "canonical_parent_id": "realimpact-6-bowl--objectfolder-real-object-6",
            "current_parent_ids": [
                "realimpact-6-bowl--objectfolder-real-object-6",
                "av-parent-6",
            ],
        },
        {
            "canonical_parent_id": d2.MATERIAL_QUARANTINE_PARENT,
            "current_parent_ids": ["of-parent-80", "av-parent-80"],
        },
    ]
    findings = [
        {
            "canonical_parent_id": alias["canonical_parent_id"],
            "object_id": object_id,
        }
        for alias, object_id in zip(aliases, ("6", "80"), strict=True)
    ]
    values = {
        "access_ledger": {"fixture": "access"},
        "alias_findings": {
            "cross_role_finding_count": 2,
            "findings": findings,
            "schema": d2.C1A_FINDINGS_SCHEMA,
        },
        "evidence_inventory": {"fixture": "evidence"},
        "profile": {"fixture": "profile"},
        "repair_plan": {
            "alias_overrides": aliases,
            "family_reassignment": {
                "family_id": d2.AV_FAMILY,
                "from_role": d2.VALIDATOR_ROLE,
                "record_count": 20,
                "to_role": d2.TRAIN_ROLE,
            },
            "material_quarantine_parent_ids": [d2.MATERIAL_QUARANTINE_PARENT],
            "next_stage": "v43_d2_c0r_identity_repair",
            "schema": d2.C1A_PLAN_SCHEMA,
        },
        "report": {
            "decision": "C0_IDENTITY_REPAIR_REQUIRED",
            "gates": {"fixture": True},
            "schema": d2.C1A_REPORT_SCHEMA,
        },
    }
    return {name: d2.canonical_json(value) for name, value in values.items()}


def update_binding(
    profile: dict[str, object], group: str, name: str, data: bytes
) -> None:
    inputs = profile["input_bindings"]
    assert isinstance(inputs, dict)
    bindings = inputs[group]
    assert isinstance(bindings, dict)
    bindings[name] = bound(data)


def build_fixture(root: Path) -> dict[str, Path]:
    c0 = root / "c0"
    c1a = root / "c1a"
    c0.mkdir()
    c1a.mkdir()
    records = build_records(c0)

    source_profile = d2.canonical_json(
        {"authority": {"protected_access_authority": False}}
    )
    c0_values: dict[str, bytes] = {
        "access_ledger": d2.canonical_json({"fixture": "access"}),
        "corpus_card": d2.canonical_json({"fixture": "card"}),
        "profile": source_profile,
        "report": d2.canonical_json(
            {
                "decision": "C0_DISCLOSED_CORPUS_REPEATABLE_B0_V0_AUTHORIZED",
                "gates": {"fixture": True},
            }
        ),
    }
    for role in d2.ROLES:
        c0_values[role] = projection(role, records)
    manifest_core = {
        "corpus_policy": {"sample_rate_hz": 48000},
        "feature_policy": {"representation": "fixture"},
        "manifest_root_sha256": "3" * 64,
        "profile_sha256": sha256(source_profile),
        "records": records,
        "schema": d2.C0_MANIFEST_SCHEMA,
    }
    c0_values["manifest"] = d2.canonical_json(manifest_core)
    for name, relative in d2.C0_FILES.items():
        write(c0, relative, c0_values[name])

    c1a_values = c1a_files()
    for name, relative in d2.C1A_FILES.items():
        write(c1a, relative, c1a_values[name])

    roster_bytes = d1_roster()
    roster = root / "d1.json"
    roster.write_bytes(roster_bytes)

    profile = copy.deepcopy(json.loads(PROFILE.read_text()))
    profile["input_bindings"]["c0"] = {
        name: bound(data) for name, data in c0_values.items()
    }
    profile["input_bindings"]["c1a"] = {
        name: bound(data) for name, data in c1a_values.items()
    }
    profile["input_bindings"]["d1_roster"] = {
        **bound(roster_bytes),
        "roster_root_sha256": "2" * 64,
    }
    profile_path = root / "profile.json"
    profile_path.write_bytes(d2.canonical_json(profile))
    return {"c0": c0, "c1a": c1a, "d1": roster, "profile": profile_path}


def run_fixture(paths: dict[str, Path], output: Path) -> Path:
    return d2.run(paths["profile"], paths["d1"], paths["c0"], paths["c1a"], output)


class IdentityRepairTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_bound(self) -> None:
        data, profile = d2.read_canonical_profile(PROFILE)
        self.assertEqual(data, d2.canonical_json(profile))
        d2.validate_profile(profile)

    def test_full_run_repeats_and_publishes_corrected_corpus(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v43-d2-repeat-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            run_fixture(paths, root / "a")
            run_fixture(paths, root / "b")
            first = artifact_tree(root / "a")
            self.assertEqual(first, artifact_tree(root / "b"))
            self.assertEqual(len(first), 288)
            report = json.loads(first["report.json"])
            self.assertEqual(report["decision"], d2.DECISION)
            self.assertEqual(report["result"]["physical_parents"], 65)
            self.assertEqual(report["result"]["role_records"], d2.CORRECTED_ROLES)
            self.assertEqual(report["result"]["role_parents"], d2.CORRECTED_PARENTS)
            self.assertEqual(report["result"]["material_quarantine_records"], 5)
            self.assertEqual(report["result"]["validator_projects"], 1)
            self.assertTrue(all(report["gates"].values()))
            for counter in (
                "acoustic_feature_objects_decoded",
                "audio_decoded_samples",
                "candidate_model_bytes_read",
                "network_requests",
                "protected_payload_bytes_read",
                "validator_candidate_outputs_read",
            ):
                self.assertEqual(report["access"][counter], 0)

            manifest = json.loads(first["manifest.json"])
            av = [
                row for row in manifest["records"] if row["family_id"] == d2.AV_FAMILY
            ]
            self.assertEqual(len(av), 20)
            self.assertEqual({row["role"] for row in av}, {d2.TRAIN_ROLE})
            object_80 = [
                row
                for row in manifest["records"]
                if row["physical_parent_id"] == d2.MATERIAL_QUARANTINE_PARENT
            ]
            self.assertEqual(len(object_80), 5)
            self.assertEqual({row["material_label"] for row in object_80}, {"Unknown"})
            self.assertTrue(
                all(row["axis_mask"]["material_identity"] is False for row in object_80)
            )
            object_6 = [row for row in manifest["records"] if row["object_id"] == "6"]
            self.assertEqual(
                {row["physical_parent_id"] for row in object_6},
                {"realimpact-6-bowl--objectfolder-real-object-6"},
            )
            source_objects = {
                path.relative_to(paths["c0"]).as_posix(): path.read_bytes()
                for path in (paths["c0"] / "objects").rglob("*")
                if path.is_file()
            }
            self.assertTrue(
                all(first[name] == data for name, data in source_objects.items())
            )

    def test_content_object_mutation_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v43-d2-object-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            content = next((paths["c0"] / "objects").rglob("*.wav"))
            content.write_bytes(content.read_bytes() + b"mutation")
            output = root / "output"
            with self.assertRaisesRegex(d2.IdentityRepairError, "binding changed"):
                run_fixture(paths, output)
            self.assertFalse(output.exists())

    def test_c1a_plan_mutation_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v43-d2-c1a-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            profile = json.loads(paths["profile"].read_text())
            plan_path = paths["c1a"] / "repair-plan.json"
            plan = json.loads(plan_path.read_text())
            plan["family_reassignment"]["to_role"] = d2.VALIDATOR_ROLE
            plan_bytes = d2.canonical_json(plan)
            plan_path.write_bytes(plan_bytes)
            update_binding(profile, "c1a", "repair_plan", plan_bytes)
            paths["profile"].write_bytes(d2.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(d2.IdentityRepairError, "reassignment"):
                run_fixture(paths, output)
            self.assertFalse(output.exists())

    def test_projection_mismatch_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v43-d2-projection-"
        ) as temp:
            root = Path(temp)
            paths = build_fixture(root)
            profile = json.loads(paths["profile"].read_text())
            path = paths["c0"] / "projections" / "generator_train.json"
            projection_value = json.loads(path.read_text())
            projection_value["rows"][0]["record_id"] = "missing-record"
            projection_value["rows_root_sha256"] = sha256(
                d2.canonical_json(projection_value["rows"])
            )
            data = d2.canonical_json(projection_value)
            path.write_bytes(data)
            update_binding(profile, "c0", "generator_train", data)
            paths["profile"].write_bytes(d2.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(d2.IdentityRepairError, "projection/manifest"):
                run_fixture(paths, output)
            self.assertFalse(output.exists())

    def test_atomic_publication_and_path_guards(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v43-d2-atomic-") as temp:
            root = Path(temp)
            output = root / "result"
            writes = 0
            original = Path.write_bytes

            def fail_second_write(path: Path, data: bytes) -> int:
                nonlocal writes
                writes += 1
                if writes == 2:
                    raise OSError("injected publication failure")
                return original(path, data)

            with (
                mock.patch.object(Path, "write_bytes", fail_second_write),
                self.assertRaisesRegex(OSError, "injected"),
            ):
                d2.publish_directory(
                    output, {"a.json": b"{}\n", "nested/b.json": b"{}\n"}
                )
            self.assertFalse(output.exists())
            self.assertEqual(list(root.iterdir()), [])

            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(d2.IdentityRepairError, "refusing to replace"):
                d2.prepare_output(occupied)

            linked = root / "linked"
            linked.symlink_to(root / "actual", target_is_directory=True)
            with self.assertRaisesRegex(d2.IdentityRepairError, "symlink"):
                d2.prepare_output(linked / "result")

        with self.assertRaisesRegex(d2.IdentityRepairError, "outside"):
            d2.prepare_output(ROOT / "target" / "forbidden-d2-output")

    def test_owner_has_no_network_decoder_or_model_dependency(self) -> None:
        source = SCRIPT.read_text()
        for forbidden in (
            "import librosa",
            "import requests",
            "import scipy",
            "import socket",
            "import soundfile",
            "import subprocess",
            "import torch",
            "import torchaudio",
            "import urllib",
            "import wave",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
