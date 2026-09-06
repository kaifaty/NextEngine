"""Frozen identity and manifest boundary for the V28 M0c resource successor."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import physical_sound_v25_m0a_common as inherited

PROTOCOL_PATH = (
    "docs/development/physical-sound-v28-r0-m0c-prefix-resource-protocol-2026-09-02.md"
)
PROTOCOL_SHA256 = "28cab4283e9ea0178b59100960245d27e04c5e749842dd87fa011ec788ca9000"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v28-m0c.manifest.v1"
MODEL_ID = inherited.MODEL_ID
EXPERIMENT_ID = "m0c-prefix-bounded-neural-student-v1"

INHERITED_IMPLEMENTATION_HASHES = {
    "physical_sound_v25_m0a_common.py": "592b38137eb088e4ffc8087c3c68c0c19b0a8ceed8d049e7e9b2f6a002573e89",
    "physical_sound_v25_m0a_evaluate.py": "e5c48a74f5df89d60433d2ef1605bce648250836809e04721a2aa1fefe3e552a",
    "physical_sound_v25_m0a_model.py": "6075b4121c5eb45aca2934308fb4e73c078c467669e458c0d36b65b066f4009b",
    "physical_sound_v25_m0a_train.py": "86ada58126ad901021cdf297cef2fc46059dc33763e960dbdc21ebf3972630b5",
    "physical_sound_v26_m0b_alignment.py": "1452848dc0ba8c678df0041b36cde4ad0ef9daad4a4431ee758a0d15dc8c8314",
    "physical_sound_v26_m0b_common.py": "5811796e75c23bde709027af77be97278c6773c2d0a71ecc13d2a2cbb7bd9624",
    "physical_sound_v26_m0b_preprocess.py": "12bf572706b504b5e048a4ea9f85ef58c1a336c49f1baa802cd663fa53bb5ced",
    "physical_sound_v26_m0b_surface.py": "c12799014ac8d7f735668bf465103dce7f90ba9cf4592eb27e5545bdd37b1705",
    "physical_sound_v26_m0b_train.py": "0c4492b7dace6d9dafe7729b7a1975c40d9b6c9cb2a2d6d28ff23f2fc037d40c",
}
OWN_IMPLEMENTATION_FILES = (
    "physical_sound_v28_m0c_common.py",
    "physical_sound_v28_m0c_model.py",
    "physical_sound_v28_m0c_resource.py",
    "physical_sound_v28_m0c_train.py",
)

M0Error = inherited.M0Error
MAX_JSON_BYTES = inherited.MAX_JSON_BYTES
MAX_ARTIFACT_BYTES = inherited.MAX_ARTIFACT_BYTES


def implementation_hashes() -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    observed_inherited = {
        name: inherited.sha256_file(directory / name)
        for name in sorted(INHERITED_IMPLEMENTATION_HASHES)
    }
    if observed_inherited != INHERITED_IMPLEMENTATION_HASHES:
        raise M0Error("M0c inherited M0a/M0b implementation hashes changed")
    own = {
        name: inherited.sha256_file(directory / name)
        for name in sorted(OWN_IMPLEMENTATION_FILES)
    }
    return dict(sorted({**observed_inherited, **own}.items()))


def implementation_root_sha256() -> str:
    return inherited.sha256_bytes(inherited.canonical_json(implementation_hashes()))


def _validate_protocol_file() -> None:
    path = inherited.repository_root() / PROTOCOL_PATH
    if inherited.sha256_file(path) != PROTOCOL_SHA256:
        raise M0Error("M0c R0 protocol file hash changed")


def load_manifest(path: Path) -> tuple[dict[str, Any], bytes]:
    _validate_protocol_file()
    path = inherited.external_file(path, "M0c manifest", 128 * 1024)
    data = path.read_bytes()
    manifest = inherited.parse_json_bytes(data, "M0c manifest", 128 * 1024)
    expected = {
        "schema",
        "profile",
        "protocol_sha256",
        "implementation_root_sha256",
        "inherited_implementation_hashes",
        "artifacts",
        "t0_root",
        "x0_root",
        "mlflow_root",
        "data_policy",
    }
    if set(manifest) != expected or manifest.get("schema") != MANIFEST_SCHEMA:
        raise M0Error("M0c manifest fields or schema changed")
    inherited.execution_profile(manifest.get("profile"))
    if manifest.get("protocol_sha256") != PROTOCOL_SHA256:
        raise M0Error("M0c protocol hash changed")
    if (
        manifest.get("inherited_implementation_hashes")
        != INHERITED_IMPLEMENTATION_HASHES
    ):
        raise M0Error("M0c inherited implementation manifest changed")
    if manifest.get("implementation_root_sha256") != implementation_root_sha256():
        raise M0Error("M0c implementation root changed")
    if manifest.get("data_policy") != {
        "external_output_only": True,
        "cpu_only": True,
        "network_allowed": False,
        "admission_shadow_access_allowed": False,
        "runtime_authorized": False,
        "mlflow_remote_allowed": False,
    }:
        raise M0Error("M0c data policy changed")
    artifacts = manifest.get("artifacts")
    expected_artifacts = {
        "combined_manifest",
        "teacher_evidence",
        "x0_lineage",
        "ffmpeg",
        "ffprobe",
    }
    if not isinstance(artifacts, dict) or set(artifacts) != expected_artifacts:
        raise M0Error("M0c artifact set changed")
    inherited.external_directory(Path(manifest["t0_root"]), "M0c T0 root")
    inherited.external_directory(Path(manifest["x0_root"]), "M0c X0 root")
    mlflow_root = Path(manifest["mlflow_root"])
    inherited.external_directory(
        mlflow_root, "M0c MLflow root", must_exist=mlflow_root.exists()
    )
    for name, maximum in (
        ("combined_manifest", MAX_JSON_BYTES),
        ("teacher_evidence", MAX_JSON_BYTES),
        ("x0_lineage", MAX_JSON_BYTES),
        ("ffmpeg", MAX_ARTIFACT_BYTES),
        ("ffprobe", MAX_ARTIFACT_BYTES),
    ):
        inherited.validate_ref(artifacts[name], f"M0c {name}", maximum)
    if manifest["profile"] == "official-v1":
        fixed = {
            "combined_manifest": inherited.OFFICIAL_COMBINED_SHA256,
            "teacher_evidence": inherited.OFFICIAL_T0_EVIDENCE_SHA256,
            "x0_lineage": inherited.OFFICIAL_X0_LINEAGE_SHA256,
        }
        for name, expected_hash in fixed.items():
            if artifacts[name]["sha256"] != expected_hash:
                raise M0Error(f"official M0c {name} hash changed")
    return manifest, data


def load_combined(manifest: dict[str, Any]) -> tuple[dict[str, Any], bytes]:
    return inherited.load_combined(manifest)


def validate_combined_lineage_bindings(
    combined: dict[str, Any], manifest: dict[str, Any]
) -> None:
    inherited.validate_combined_lineage_bindings(combined, manifest)
