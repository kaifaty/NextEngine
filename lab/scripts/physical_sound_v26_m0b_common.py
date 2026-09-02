"""Frozen identity and manifest boundary for the V26 M0b preprocessing repair."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import physical_sound_v25_m0a_common as inherited

PROTOCOL_PATH = (
    "docs/development/"
    "physical-sound-v26-p0a-barycentric-and-padded-alignment-protocol-2026-09-02.md"
)
PROTOCOL_SHA256 = "54522c26eb3db62ad316ea6001f9380651f760329f0db8556b441ad286b649e8"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v26-m0b.manifest.v1"
PREPROCESS_SCHEMA = "nextengine.experimental-physical-sound-v26-m0b.preprocess.v1"
SURFACE_REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-v26-m0b.surface-query.v1"
)
MODEL_ID = inherited.MODEL_ID
EXPERIMENT_ID = "m0b-barycentric-padded-preprocessing-repair-v1"

INHERITED_IMPLEMENTATION_HASHES = {
    "physical_sound_v25_m0a_common.py": "592b38137eb088e4ffc8087c3c68c0c19b0a8ceed8d049e7e9b2f6a002573e89",
    "physical_sound_v25_m0a_evaluate.py": "e5c48a74f5df89d60433d2ef1605bce648250836809e04721a2aa1fefe3e552a",
    "physical_sound_v25_m0a_model.py": "6075b4121c5eb45aca2934308fb4e73c078c467669e458c0d36b65b066f4009b",
    "physical_sound_v25_m0a_train.py": "86ada58126ad901021cdf297cef2fc46059dc33763e960dbdc21ebf3972630b5",
}
OWN_IMPLEMENTATION_FILES = (
    "physical_sound_v26_m0b_common.py",
    "physical_sound_v26_m0b_surface.py",
    "physical_sound_v26_m0b_alignment.py",
    "physical_sound_v26_m0b_preprocess.py",
    "physical_sound_v26_m0b_train.py",
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
        raise M0Error("M0b inherited M0a implementation hashes changed")
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
        raise M0Error("M0b P0a protocol file hash changed")


def load_manifest(path: Path) -> tuple[dict[str, Any], bytes]:
    _validate_protocol_file()
    path = inherited.external_file(path, "M0b manifest", 128 * 1024)
    data = path.read_bytes()
    manifest = inherited.parse_json_bytes(data, "M0b manifest", 128 * 1024)
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
        raise M0Error("M0b manifest fields or schema changed")
    inherited.execution_profile(manifest.get("profile"))
    if manifest.get("protocol_sha256") != PROTOCOL_SHA256:
        raise M0Error("M0b protocol hash changed")
    if (
        manifest.get("inherited_implementation_hashes")
        != INHERITED_IMPLEMENTATION_HASHES
    ):
        raise M0Error("M0b inherited implementation manifest changed")
    if manifest.get("implementation_root_sha256") != implementation_root_sha256():
        raise M0Error("M0b implementation root changed")
    if manifest.get("data_policy") != {
        "external_output_only": True,
        "cpu_only": True,
        "network_allowed": False,
        "admission_shadow_access_allowed": False,
        "runtime_authorized": False,
        "mlflow_remote_allowed": False,
    }:
        raise M0Error("M0b data policy changed")
    artifacts = manifest.get("artifacts")
    expected_artifacts = {
        "combined_manifest",
        "teacher_evidence",
        "x0_lineage",
        "ffmpeg",
        "ffprobe",
    }
    if not isinstance(artifacts, dict) or set(artifacts) != expected_artifacts:
        raise M0Error("M0b artifact set changed")
    inherited.external_directory(Path(manifest["t0_root"]), "M0b T0 root")
    inherited.external_directory(Path(manifest["x0_root"]), "M0b X0 root")
    mlflow_root = Path(manifest["mlflow_root"])
    inherited.external_directory(
        mlflow_root, "M0b MLflow root", must_exist=mlflow_root.exists()
    )
    for name, maximum in (
        ("combined_manifest", MAX_JSON_BYTES),
        ("teacher_evidence", MAX_JSON_BYTES),
        ("x0_lineage", MAX_JSON_BYTES),
        ("ffmpeg", MAX_ARTIFACT_BYTES),
        ("ffprobe", MAX_ARTIFACT_BYTES),
    ):
        inherited.validate_ref(artifacts[name], f"M0b {name}", maximum)
    if manifest["profile"] == "official-v1":
        fixed = {
            "combined_manifest": inherited.OFFICIAL_COMBINED_SHA256,
            "teacher_evidence": inherited.OFFICIAL_T0_EVIDENCE_SHA256,
            "x0_lineage": inherited.OFFICIAL_X0_LINEAGE_SHA256,
        }
        for name, expected_hash in fixed.items():
            if artifacts[name]["sha256"] != expected_hash:
                raise M0Error(f"official M0b {name} hash changed")
    return manifest, data


def load_combined(manifest: dict[str, Any]) -> tuple[dict[str, Any], bytes]:
    return inherited.load_combined(manifest)


def validate_combined_lineage_bindings(
    combined: dict[str, Any], manifest: dict[str, Any]
) -> None:
    inherited.validate_combined_lineage_bindings(combined, manifest)
