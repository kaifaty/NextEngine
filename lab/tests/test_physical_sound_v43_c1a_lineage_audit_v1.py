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
SCRIPT = SCRIPTS / "physical_sound_v43_c1a_lineage_audit_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v43-c1a-lineage-audit.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v43_c1a_lineage_audit_v1 as c1a  # noqa: E402


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


def record(
    record_id: str,
    role: str,
    family: str,
    object_id: str,
    parent: str,
    material: str,
) -> dict[str, object]:
    return {
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
    }


def append_parent_records(
    rows: list[dict[str, object]],
    role: str,
    family: str,
    parent_counts: list[int],
    material: str = "Metal",
) -> None:
    for parent_index, count in enumerate(parent_counts):
        object_id = f"{family}-object-{parent_index}"
        parent = f"{family}-parent-{parent_index}"
        for recording_index in range(count):
            rows.append(
                record(
                    f"{family}-{parent_index}-{recording_index}",
                    role,
                    family,
                    object_id,
                    parent,
                    material,
                )
            )


def build_records() -> list[dict[str, object]]:
    train: list[dict[str, object]] = []
    development: list[dict[str, object]] = []
    validator: list[dict[str, object]] = []

    for object_id, material in (("6", "Glass"), ("80", "Wood")):
        for recording_index in range(3):
            train.append(
                record(
                    f"of-{object_id}-{recording_index}",
                    "generator_train",
                    c1a.OF_FAMILY,
                    object_id,
                    f"of-parent-{object_id}",
                    material,
                )
            )
    append_parent_records(
        train,
        "generator_train",
        c1a.OF_FAMILY,
        [3, 3, 3],
        "Glass",
    )
    append_parent_records(
        train,
        "generator_train",
        "train-filler",
        [2] * 8 + [1] * 13,
    )

    for object_id, material in (("6", "Glass"), ("80", "Ceramic")):
        for recording_index in range(2):
            validator.append(
                record(
                    f"av-{object_id}-{recording_index}",
                    "validator_calibration",
                    c1a.AV_FAMILY,
                    object_id,
                    f"av-parent-{object_id}",
                    material,
                )
            )
    append_parent_records(
        validator,
        "validator_calibration",
        c1a.AV_FAMILY,
        [2] * 8,
        "Ceramic",
    )
    append_parent_records(
        validator,
        "validator_calibration",
        "validator-filler",
        [5],
        "Glass",
    )
    append_parent_records(
        development,
        "generator_development",
        "development-filler",
        [3] * 10 + [2] * 20,
        "Wood",
    )

    assert len(train) == 44
    assert len(development) == 70
    assert len(validator) == 25
    assert len({row["physical_parent_id"] for row in train}) == 26
    assert len({row["physical_parent_id"] for row in development}) == 30
    assert len({row["physical_parent_id"] for row in validator}) == 11
    return [*train, *development, *validator]


def projection(role: str, records: list[dict[str, object]]) -> bytes:
    rows = [{"record_id": row["record_id"]} for row in records if row["role"] == role]
    return c1a.canonical_json(
        {
            "role": role,
            "rows": rows,
            "schema": c1a.PROJECTION_SCHEMA,
        }
    )


def evidence_files(marker: str | None = None) -> dict[str, bytes]:
    marker = marker or (
        "We evaluate our method on ObjectFolder Real [12] and RealImpact [2], "
        "two real-world multisensory datasets that include impact sound recordings."
    )
    av_page = b"".join(
        [
            b'<button class="object-card" data-name="Object 6" ',
            b'data-original-material="Glass" data-demo-path="data/demo/6"></button>',
            b'<button class="object-card" data-name="Object 80" ',
            b'data-original-material="Ceramic" data-demo-path="data/demo/80"></button>',
        ]
    )
    cell = '<td style="text-align: center">{}</td>'
    of_page = "".join(
        [
            cell.format("6"),
            cell.format("Blue_Bowl"),
            cell.format("Glass"),
            cell.format("80"),
            cell.format("Spoon_Holder"),
            cell.format("Wood"),
        ]
    ).encode()
    return {
        "av-msf-object-6.png": b"png-object-6",
        "av-msf-object-80.png": b"png-object-80",
        "av-msf-page.html": av_page,
        "av-msf-paper-v2.pdf": b"pdf-evidence",
        "av-msf-paper-v2.txt": marker.encode(),
        "objectfolder-object-6-clip-0.mp4": b"mp4-object-6",
        "objectfolder-object-80-clip-0.mp4": b"mp4-object-80",
        "objectfolder-real-download.html": of_page,
    }


def write_profile(path: Path, profile: dict[str, object]) -> None:
    path.write_bytes(c1a.canonical_json(profile))


def update_binding(
    profile: dict[str, object], group: str, name: str, data: bytes
) -> None:
    bindings = profile[group]
    assert isinstance(bindings, dict)
    bindings[name] = bound(data)


def build_fixture(root: Path) -> dict[str, Path]:
    corpus = root / "corpus"
    evidence = root / "evidence"
    corpus.mkdir()
    evidence.mkdir()
    records = build_records()

    corpus_profile = c1a.canonical_json(
        {"authority": {"protected_access_authority": False}}
    )
    corpus_files = {
        "profile": corpus_profile,
        **{
            role: projection(role, records)
            for role in (
                "generator_development",
                "generator_train",
                "validator_calibration",
            )
        },
    }
    manifest = c1a.canonical_json(
        {
            "manifest_root_sha256": "1" * 64,
            "profile_sha256": sha256(corpus_profile),
            "records": records,
            "schema": c1a.MANIFEST_SCHEMA,
        }
    )
    (corpus / "profile.json").write_bytes(corpus_profile)
    (corpus / "manifest.json").write_bytes(manifest)
    projection_root = corpus / "projections"
    projection_root.mkdir()
    for role in (
        "generator_development",
        "generator_train",
        "validator_calibration",
    ):
        (projection_root / f"{role}.json").write_bytes(corpus_files[role])

    sources = evidence_files()
    for name, data in sources.items():
        (evidence / name).write_bytes(data)

    profile = copy.deepcopy(json.loads(PROFILE.read_text()))
    profile["input_bindings"] = {
        "manifest": bound(manifest),
        "profile": bound(corpus_profile),
        **{role: bound(corpus_files[role]) for role in c1a.PROJECTION_FILES},
    }
    profile["evidence_bindings"] = {name: bound(data) for name, data in sources.items()}
    profile_path = root / "profile.json"
    write_profile(profile_path, profile)
    return {
        "corpus": corpus,
        "evidence": evidence,
        "profile": profile_path,
    }


class LineageAuditTests(unittest.TestCase):
    def test_tracked_profile_is_canonical_and_bound(self) -> None:
        data, profile = c1a.read_canonical_profile(PROFILE)
        self.assertEqual(data, c1a.canonical_json(profile))
        c1a.validate_profile(profile)

    def test_full_run_repeats_and_reports_fail_closed_repair(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v43-c1a-repeat-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            c1a.run(paths["profile"], paths["corpus"], paths["evidence"], root / "a")
            c1a.run(paths["profile"], paths["corpus"], paths["evidence"], root / "b")
            self.assertEqual(artifact_tree(root / "a"), artifact_tree(root / "b"))
            report = json.loads((root / "a" / "report.json").read_text())
            self.assertEqual(report["decision"], c1a.DECISION)
            self.assertEqual(report["result"]["alias_findings"], 2)
            self.assertEqual(report["result"]["corrected_physical_parents"], 65)
            self.assertEqual(
                report["result"]["corrected_role_records"], c1a.CORRECTED_ROLES
            )
            self.assertEqual(
                report["result"]["validator_calibration_projects_after_repair"],
                1,
            )
            self.assertTrue(all(report["gates"].values()))
            access = json.loads((root / "a" / "access-ledger.json").read_text())
            for counter in (
                "acoustic_feature_objects_read",
                "audio_decoded_samples",
                "candidate_model_bytes_read",
                "network_requests",
                "protected_payload_bytes_read",
                "validator_candidate_outputs_read",
            ):
                self.assertEqual(access["access"][counter], 0)

    def test_bound_evidence_mutation_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v43-c1a-evidence-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            evidence = paths["evidence"] / "av-msf-object-6.png"
            evidence.write_bytes(evidence.read_bytes() + b"mutation")
            output = root / "output"
            with self.assertRaisesRegex(c1a.LineageAuditError, "binding changed"):
                c1a.run(paths["profile"], paths["corpus"], paths["evidence"], output)
            self.assertFalse(output.exists())

    def test_semantic_marker_and_material_mutations_publish_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v43-c1a-semantic-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            profile = json.loads(paths["profile"].read_text())
            paper = paths["evidence"] / "av-msf-paper-v2.txt"
            changed_paper = b"The required dataset declaration is absent."
            paper.write_bytes(changed_paper)
            update_binding(
                profile, "evidence_bindings", "av-msf-paper-v2.txt", changed_paper
            )
            write_profile(paths["profile"], profile)
            output = root / "marker-output"
            with self.assertRaisesRegex(c1a.LineageAuditError, "dataset marker"):
                c1a.run(paths["profile"], paths["corpus"], paths["evidence"], output)
            self.assertFalse(output.exists())

        with tempfile.TemporaryDirectory(prefix="nextengine-v43-c1a-material-") as temp:
            root = Path(temp)
            paths = build_fixture(root)
            profile = json.loads(paths["profile"].read_text())
            manifest_path = paths["corpus"] / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            target = next(
                row
                for row in manifest["records"]
                if row["family_id"] == c1a.OF_FAMILY and row["object_id"] == "80"
            )
            target["material_label"] = "Ceramic"
            manifest_bytes = c1a.canonical_json(manifest)
            manifest_path.write_bytes(manifest_bytes)
            update_binding(profile, "input_bindings", "manifest", manifest_bytes)
            write_profile(paths["profile"], profile)
            output = root / "material-output"
            with self.assertRaisesRegex(c1a.LineageAuditError, "material changed"):
                c1a.run(paths["profile"], paths["corpus"], paths["evidence"], output)
            self.assertFalse(output.exists())

    def test_projection_mismatch_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v43-c1a-projection-"
        ) as temp:
            root = Path(temp)
            paths = build_fixture(root)
            profile = json.loads(paths["profile"].read_text())
            projection_path = paths["corpus"] / "projections" / "generator_train.json"
            projection = json.loads(projection_path.read_text())
            projection["rows"][0]["record_id"] = "absent-record"
            projection_bytes = c1a.canonical_json(projection)
            projection_path.write_bytes(projection_bytes)
            update_binding(
                profile, "input_bindings", "generator_train", projection_bytes
            )
            write_profile(paths["profile"], profile)
            output = root / "output"
            with self.assertRaisesRegex(c1a.LineageAuditError, "projection/manifest"):
                c1a.run(paths["profile"], paths["corpus"], paths["evidence"], output)
            self.assertFalse(output.exists())

    def test_atomic_publication_and_path_guards(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v43-c1a-atomic-") as temp:
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
                c1a.publish_directory(output, {"a.json": b"{}\n", "b.json": b"{}\n"})
            self.assertFalse(output.exists())
            self.assertEqual(list(root.iterdir()), [])

            occupied = root / "occupied"
            occupied.mkdir()
            with self.assertRaisesRegex(c1a.LineageAuditError, "refusing to replace"):
                c1a.prepare_output(occupied)

            linked = root / "linked"
            linked.symlink_to(root / "actual", target_is_directory=True)
            with self.assertRaisesRegex(c1a.LineageAuditError, "symlink"):
                c1a.prepare_output(linked / "result")

        with self.assertRaisesRegex(c1a.LineageAuditError, "outside"):
            c1a.prepare_output(ROOT / "target" / "forbidden-c1a-output")

    def test_owner_has_no_network_decoder_feature_or_model_dependency(self) -> None:
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
            "objects/features",
            "objects/pcm",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
