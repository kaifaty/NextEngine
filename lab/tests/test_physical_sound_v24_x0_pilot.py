#!/usr/bin/env python3
"""Contract-only whole-entry tests for the V24 X0 disclosed real pilot."""

from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v24_x0_pilot as pilot


def write_json(path: Path, value: object) -> dict[str, str]:
    data = pilot.canonical_json(value)
    path.write_bytes(data)
    return {"path": str(path), "sha256": pilot.sha256_bytes(data)}


def write_blob(path: Path, data: bytes) -> dict[str, str]:
    path.write_bytes(data)
    return {"path": str(path), "sha256": pilot.sha256_bytes(data)}


def fixture_inputs(root: Path) -> tuple[dict[str, object], dict[str, Path]]:
    selected = pilot.profile("contract-fixture-v1")
    files: dict[str, Path] = {}
    preflight = {
        "status": "frozen_zero_audio_preflight",
        "profile": "realimpact-blue-bowl-canonical-contact-r3a-v1",
        "canonical_listener": {
            "azimuth_degrees": 0,
            "claim": "exact_published_condition_only_not_arbitrary_radiation",
            "distance_offset_millimetres": 0,
            "microphone_id": 7,
        },
    }
    report = {
        "status": "Validated",
        "audio_payload_bytes_accessed": 0,
        "authorized_audio_row_count": 4,
        "sealed_audio_row_count": 1,
    }
    contacts = {
        "sample_rate_hz": 48_000,
        "sample_count": selected.transfer_shape[1],
        "contacts": [
            {
                "impact_index": impact,
                "position_metres": list(point),
                "role": "representation_development" if role == "query" else "fit",
                "row_index": row,
                "vertex_id": vertex,
                "waveform_access": "authorized",
            }
            for impact, row, role, vertex, point in selected.contacts
        ],
        "sealed_contact_commitment": {
            "impact_index": selected.sealed[0],
            "position_metres": list(selected.sealed[3]),
            "row_index": selected.sealed[1],
            "vertex_id": selected.sealed[2],
            "waveform_access": "sealed_not_decompressed",
        },
    }
    extraction = {
        "status": "Validated",
        "sealed_row_index": selected.sealed[1],
        "sealed_waveform_samples_decoded": 0,
    }
    audio = {}
    entries = []
    recordings = []
    for index, recording in enumerate(("000", "020", "039")):
        data = bytes([index + 1]) * (31 + index)
        path = root / f"recording-{recording}.mp4"
        reference = write_blob(path, data)
        audio[recording] = path
        recordings.append(
            {
                "recording_id": recording,
                "byte_count": len(data),
                **reference,
            }
        )
        entries.append(
            {
                "source_id": "objectfolder-real-demo-6",
                "recording_id": recording,
                "material_label": "Glass",
                "object_id": "6",
                "audio_file_bytes": len(data),
                "audio_file_sha256": reference["sha256"],
            }
        )
    identified = {"status": "Validated", "entries": entries}
    sources = {
        "status": "Validated",
        "sources": [
            {
                "id": "objectfolder-real-demo-6",
                "adapter_evidence": {"material_label": "Glass"},
            }
        ],
    }
    predecessor_values = {
        "preflight_manifest": preflight,
        "preflight_report": report,
        "contacts_metadata": contacts,
        "extraction_report": extraction,
        "objectfolder_identified_report": identified,
        "objectfolder_source_report": sources,
    }
    artifacts: dict[str, object] = {}
    for name, value in predecessor_values.items():
        path = root / f"{name}.json"
        files[name] = path
        artifacts[name] = write_json(path, value)

    array_path = root / "contacts.npy"
    array = np.arange(32, dtype="<f4").reshape(4, 8) + 1.0
    with array_path.open("wb") as stream:
        np.save(stream, array, allow_pickle=False)
    files["contacts_array"] = array_path
    artifacts["contacts_array"] = {
        "path": str(array_path),
        "sha256": pilot.sha256_bytes(array_path.read_bytes()),
    }
    mesh = "\n".join(
        (
            "v 0 0 0",
            "v 1 0 0",
            "v 0 1 0",
            "v 0 0 1",
            "v 1 1 1",
            "f 1 2 3",
            "f 1 2 4",
            "f 2 3 5",
        )
    ).encode()
    mesh_path = root / "fixture.obj"
    mesh_ref = write_blob(mesh_path, mesh)
    files["mesh_source"] = mesh_path
    artifacts["mesh_source"] = {"mode": "local_fixture", **mesh_ref}
    artifacts["objectfolder_audio"] = recordings
    manifest = {
        "schema": pilot.MANIFEST_SCHEMA,
        "profile": "contract-fixture-v1",
        "protocol_sha256": pilot.PROTOCOL_SHA256,
        "artifacts": artifacts,
        "data_policy": {
            "external_output_only": True,
            "large_transfer_refetch_allowed": False,
            "model_training_authorized": False,
            "sealed_contact_access_allowed": False,
            "runtime_authorized": False,
        },
    }
    return manifest, files


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class PhysicalSoundV24X0PilotTests(unittest.TestCase):
    def test_full_cli_twice_is_exact_and_keeps_real_claims_narrow(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v24-x0-test-") as temporary:
            root = Path(temporary)
            manifest, _ = fixture_inputs(root)
            manifest_path = root / "manifest.json"
            manifest_path.write_bytes(pilot.canonical_json(manifest))
            command = [
                sys.executable,
                str(SCRIPTS / "physical_sound_v24_x0_pilot.py"),
                "--manifest",
                str(manifest_path),
                "--output",
            ]
            with mock.patch.object(pilot, "fetch_zip_entry", side_effect=AssertionError("network")):
                first_result = subprocess.run(
                    [*command, str(root / "run-a")], check=False, capture_output=True, text=True
                )
                second_result = subprocess.run(
                    [*command, str(root / "run-b")], check=False, capture_output=True, text=True
                )
            self.assertEqual(first_result.returncode, 0, first_result.stderr)
            self.assertEqual(second_result.returncode, 0, second_result.stderr)
            first = root / "run-a"
            second = root / "run-b"
            self.assertEqual(all_files(first), all_files(second))
            self.assertEqual(len(all_files(first)), 8)

            report = json.loads((first / "report.json").read_bytes())
            self.assertEqual(report["status"], "Validated")
            self.assertEqual(report["transfer_row_count"], 4)
            self.assertEqual(report["identified_recording_count"], 3)
            self.assertEqual(report["sealed_waveform_samples_decoded"], 0)
            self.assertFalse(report["model_training_authorized"])
            self.assertFalse(report["real_material_admission_authorized"])
            self.assertFalse(report["runtime_authorized"])

            mesh = (first / "mesh.bin").read_bytes()
            self.assertEqual(mesh[:8], b"NEMESH01")
            self.assertEqual(tuple(np.frombuffer(mesh[8:20], dtype="<u4")), (1, 5, 3))
            lane = json.loads((first / "x0-lane-records.json").read_bytes())
            self.assertEqual(len(lane["rows"]), 7)
            for row in lane["rows"]:
                self.assertEqual(row["split_role"], "development")
                self.assertNotIn("teacher_target", row["axes"])
                self.assertNotIn("support", row["axes"])
                self.assertNotIn("excitation", row["axes"])
                if row["evidence_lane"] == "exact_real_transfer":
                    self.assertEqual(set(row["axes"]), {"material", "geometry", "impact", "listener"})
                    self.assertNotIn("outward_normal", row["axes"]["impact"])
                else:
                    self.assertEqual(set(row["axes"]), {"material"})
            serialized = json.dumps(lane)
            self.assertNotIn("2407", serialized)
            self.assertNotIn("31104", serialized)

    def test_every_input_hash_and_output_occupancy_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v24-x0-hash-") as temporary:
            root = Path(temporary)
            manifest, _ = fixture_inputs(root)
            references = [*pilot.PREDECESSOR_HASHES, "mesh_source"]
            for index, name in enumerate(references):
                candidate = copy.deepcopy(manifest)
                candidate["artifacts"][name]["sha256"] = "0" * 64
                path = root / f"manifest-corrupt-{index}.json"
                path.write_bytes(pilot.canonical_json(candidate))
                with self.assertRaisesRegex(ValueError, "hash mismatch"):
                    pilot.run(path, root / f"output-corrupt-{index}")
            for index in range(3):
                candidate = copy.deepcopy(manifest)
                candidate["artifacts"]["objectfolder_audio"][index]["sha256"] = "0" * 64
                path = root / f"manifest-audio-{index}.json"
                path.write_bytes(pilot.canonical_json(candidate))
                with self.assertRaises(ValueError):
                    pilot.run(path, root / f"output-audio-{index}")
            occupied = root / "occupied"
            occupied.mkdir()
            manifest_path = root / "manifest-occupied.json"
            manifest_path.write_bytes(pilot.canonical_json(manifest))
            with self.assertRaisesRegex(ValueError, "new external path"):
                pilot.run(manifest_path, occupied)

    def test_dtype_shape_finiteness_contact_listener_material_and_seal_reject(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v24-x0-mutations-") as temporary:
            root = Path(temporary)
            base, files = fixture_inputs(root)

            for label, array in (
                ("dtype", np.ones((4, 8), dtype="<f8")),
                ("shape", np.ones((4, 7), dtype="<f4")),
                ("finite", np.full((4, 8), np.nan, dtype="<f4")),
            ):
                path = root / f"contacts-{label}.npy"
                with path.open("wb") as stream:
                    np.save(stream, array, allow_pickle=False)
                candidate = copy.deepcopy(base)
                candidate["artifacts"]["contacts_array"] = {
                    "path": str(path),
                    "sha256": pilot.sha256_bytes(path.read_bytes()),
                }
                manifest_path = root / f"manifest-{label}.json"
                manifest_path.write_bytes(pilot.canonical_json(candidate))
                with self.assertRaisesRegex(ValueError, "dtype, shape or finiteness"):
                    pilot.run(manifest_path, root / f"output-{label}")

            mutations = (
                ("contact-position", "contacts_metadata", lambda value: value["contacts"][0].update(position_metres=[9.0, 0.0, 0.0])),
                ("contact-vertex", "contacts_metadata", lambda value: value["contacts"][0].update(vertex_id=4)),
                ("listener", "preflight_manifest", lambda value: value["canonical_listener"].update(microphone_id=6)),
                ("material", "objectfolder_source_report", lambda value: value["sources"][0]["adapter_evidence"].update(material_label="Metal")),
                ("seal", "extraction_report", lambda value: value.update(sealed_waveform_samples_decoded=1)),
            )
            for label, artifact, mutate in mutations:
                candidate = copy.deepcopy(base)
                value = json.loads(files[artifact].read_bytes())
                mutate(value)
                changed = root / f"{artifact}-{label}.json"
                candidate["artifacts"][artifact] = write_json(changed, value)
                manifest_path = root / f"manifest-{label}.json"
                manifest_path.write_bytes(pilot.canonical_json(candidate))
                with self.assertRaises(ValueError):
                    pilot.run(manifest_path, root / f"output-{label}")

    def test_lane_semantics_missing_axes_and_atomic_failure_reject(self) -> None:
        transfer = {
            "row_id": "a",
            "evidence_lane": "exact_real_transfer",
            "audio_semantics": "force_deconvolved_transfer_response",
            "split_role": "development",
            "axes": {name: {} for name in ("material", "geometry", "impact", "listener")},
        }
        recording = {
            "row_id": "b",
            "evidence_lane": "identified_real_recording",
            "audio_semantics": "recorded_impact_waveform",
            "split_role": "development",
            "axes": {"material": {}},
        }
        rows = [copy.deepcopy(transfer) for _ in range(4)] + [copy.deepcopy(recording) for _ in range(3)]
        for index, row in enumerate(rows):
            row["row_id"] = f"row-{index}"
        pilot.validate_lane_rows(rows)
        wrong = copy.deepcopy(rows)
        wrong[0]["audio_semantics"] = "recorded_impact_waveform"
        with self.assertRaisesRegex(ValueError, "semantics"):
            pilot.validate_lane_rows(wrong)
        fabricated = copy.deepcopy(rows)
        fabricated[0]["axes"]["support"] = {}
        with self.assertRaisesRegex(ValueError, "fabricated"):
            pilot.validate_lane_rows(fabricated)
        missing = copy.deepcopy(rows)
        del missing[0]["axes"]["listener"]
        with self.assertRaisesRegex(ValueError, "fabricated"):
            pilot.validate_lane_rows(missing)

        with tempfile.TemporaryDirectory(prefix="nextengine-v24-x0-atomic-") as temporary:
            root = Path(temporary)
            manifest, _ = fixture_inputs(root)
            manifest_path = root / "manifest.json"
            manifest_path.write_bytes(pilot.canonical_json(manifest))
            output = root / "never-published"
            with mock.patch.object(pilot, "build_into", side_effect=ValueError("forced")):
                with self.assertRaisesRegex(ValueError, "forced"):
                    pilot.run(manifest_path, output)
            self.assertFalse(output.exists())
            self.assertFalse(any(path.name.startswith(".nextengine-v24-x0-") for path in root.iterdir()))


if __name__ == "__main__":
    unittest.main()
