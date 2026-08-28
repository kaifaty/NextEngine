#!/usr/bin/env python3
"""Run the separately frozen exact-coordinate-weld REALIMPACT preflight V2."""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import shutil
import sys
from pathlib import Path
from typing import Any

import numpy as np


MANIFEST_SHA256 = "85ca065be72c7e282355fa15351d139f1d4a8a9cec80c73e1c097f08b1d1f46f"
PREREGISTRATION_MANIFEST_SHA256 = (
    "5be5f195ddc5b124e7220959efc9bb69c0bf469f495b2906e7da9f7515aae576"
)
PREREGISTRATION_REPORT_SHA256 = (
    "c2f51cffef3cac1e7b62de5b187fbd8b16e8fa89f0e8c9d755c4752405dd01ef"
)
V1_REPORT_SHA256 = "2fb9fd0f515146dd305901e65c88a9131af495d55c4dbf4c822f5e5b59cae25d"
V1_SCRIPT_SHA256 = "8c4b36d9d5f75116b9de51738c094dbe631b4935a5f90ba459f1464dccf912c0"
REPORT_SCHEMA = (
    "nextengine.experimental-realimpact-geometry-spatial-transfer-preflight.report.v2"
)
PROTOCOL_REVISION = "exact-coordinate-weld-before-frozen-preflight-v2"


def load_v1_module() -> Any:
    path = Path(__file__).with_name("physical_sound_realimpact_geometry_preflight.py")
    payload = path.read_bytes()
    actual = __import__("hashlib").sha256(payload).hexdigest()
    if actual != V1_SCRIPT_SHA256:
        raise RuntimeError(
            f"V1 geometry implementation changed: expected {V1_SCRIPT_SHA256}, got {actual}"
        )
    spec = importlib.util.spec_from_file_location("nextengine_geometry_preflight_v1", path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load V1 geometry implementation")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


V1 = load_v1_module()


def validate_manifest(manifest: dict[str, Any]) -> None:
    single = manifest.get("single_change", {})
    downstream = manifest.get("unchanged_downstream", {})
    opening = manifest.get("opening_protocol", {})
    if (
        manifest.get("schema")
        != "nextengine.experimental-realimpact-geometry-preflight.manifest.v2"
        or manifest.get("study_id") != V1.STUDY_ID
        or manifest.get("protocol_revision") != PROTOCOL_REVISION
        or manifest.get("phase") != "geometry-only-preflight"
        or single.get("id") != "exact-f64-coordinate-weld-v1"
        or single.get("vertex_order")
        != "lexicographic ascending x then y then z, matching numpy.unique(axis=0)"
        or downstream.get("spectral_face_target") != V1.SPECTRAL_FACE_TARGET
        or downstream.get("bem_face_target") != V1.BEM_FACE_TARGET
        or downstream.get("mode_count") != V1.NONCONSTANT_MODE_COUNT
        or downstream.get("maximum_relative_eigen_residual")
        != V1.MAX_RELATIVE_EIGEN_RESIDUAL
        or not opening.get("geometry_entries_only")
        or opening.get("reserved_audio_payload_bytes_allowed") != 0
        or opening.get("pitcher_audio_authorized_on_v2_support")
        or not opening.get("next_manifest_required_before_pitcher_audio")
    ):
        raise V1.PreflightError("geometry V2 manifest contract changed")
    forbidden = single.get("forbidden")
    if forbidden != [
        "positional tolerance",
        "hole filling",
        "mesh repair",
        "normal or winding repair",
        "face deletion",
        "shell thickness inference",
        "audio access",
    ]:
        raise V1.PreflightError("geometry V2 forbidden-operation boundary changed")


def exact_coordinate_weld(
    points: np.ndarray, faces: np.ndarray
) -> tuple[np.ndarray, np.ndarray, dict[str, Any]]:
    if points.dtype != np.float64 or not points.flags.c_contiguous:
        points = np.ascontiguousarray(points, dtype=np.float64)
    keys: dict[bytes, int] = {}
    representatives: list[tuple[float, float, float, bytes]] = []
    source_keys: list[bytes] = []
    for point in points:
        key = np.asarray(point, dtype="<f8").tobytes()
        source_keys.append(key)
        if key not in keys:
            keys[key] = -1
            representatives.append((float(point[0]), float(point[1]), float(point[2]), key))
    representatives.sort(key=lambda item: (item[0], item[1], item[2], item[3]))
    unique = np.empty((len(representatives), 3), dtype=np.float64)
    for index, (x, y, z, key) in enumerate(representatives):
        keys[key] = index
        unique[index] = (x, y, z)
    inverse = np.fromiter((keys[key] for key in source_keys), dtype=np.int64, count=len(points))
    welded_faces = np.ascontiguousarray(inverse[faces], dtype=np.int32)
    if any(len({int(value) for value in face}) != 3 for face in welded_faces):
        raise V1.PreflightError("exact-coordinate weld created a degenerate face")
    canonical = np.sort(welded_faces.astype(np.int64), axis=1)
    if len(np.unique(canonical, axis=0)) != len(welded_faces):
        raise V1.PreflightError("exact-coordinate weld created a duplicate face")
    if len(np.unique(welded_faces)) != len(unique):
        raise V1.PreflightError("exact-coordinate weld left an unused vertex")
    return unique, welded_faces, {
        "input_vertex_count": len(points),
        "exact_unique_vertex_count": len(unique),
        "collapsed_duplicate_vertex_count": len(points) - len(unique),
        "face_count": len(faces),
        "degenerate_face_count": 0,
        "duplicate_face_count": 0,
        "coordinate_identity": "exact parsed IEEE-754 binary64 bytes",
        "positional_tolerance": None,
    }


def transfer_modes_via_frozen_replay(
    spectral_points: np.ndarray,
    spectral_faces: np.ndarray,
    spectral_mass: np.ndarray,
    spectral_modes: np.ndarray,
    bem_points: np.ndarray,
    bem_faces: np.ndarray,
    collapses: np.ndarray,
) -> tuple[np.ndarray, np.ndarray]:
    # fast-simplification 0.1.13's replay ABI requires float32 points even
    # though simplify returns float64 points. This conversion changes no
    # frozen geometry or field rule; replay is used only to recover mapping.
    replay_points, replay_faces, mapping = V1.fast_simplification.replay_simplification(
        np.ascontiguousarray(spectral_points, dtype=np.float32),
        spectral_faces,
        np.ascontiguousarray(collapses, dtype=np.int32),
    )
    replay_points = np.ascontiguousarray(replay_points, dtype=np.float32)
    replay_faces = np.ascontiguousarray(replay_faces, dtype=np.int32)
    mapping = np.ascontiguousarray(mapping, dtype=np.int64)
    if replay_points.shape != bem_points.shape:
        raise V1.PreflightError("collapse replay BEM point count changed")
    if not np.array_equal(replay_faces, bem_faces):
        raise V1.PreflightError("collapse replay does not reproduce BEM faces")
    if len(mapping) != len(spectral_points) or np.min(mapping) < 0 or np.max(mapping) >= len(bem_points):
        raise V1.PreflightError("collapse replay mapping is invalid")
    _, _, bem_mass = V1.cotangent_system(bem_points, bem_faces)
    denominator = np.bincount(mapping, weights=spectral_mass, minlength=len(bem_points))
    if np.any(denominator <= 0.0):
        raise V1.PreflightError("field replay leaves an unmapped BEM vertex")
    transferred = np.empty((len(bem_points), spectral_modes.shape[1]), dtype=np.float64)
    for index in range(spectral_modes.shape[1]):
        numerator = np.bincount(
            mapping,
            weights=spectral_mass * spectral_modes[:, index],
            minlength=len(bem_points),
        )
        mode = numerator / denominator
        norm = float(np.sqrt(np.dot(bem_mass * mode, mode)))
        if not np.isfinite(norm) or norm <= 0.0:
            raise V1.PreflightError("transferred BEM mode mass norm is invalid")
        mode /= norm
        pivot = int(np.argmax(np.abs(mode)))
        if mode[pivot] < 0.0:
            mode *= -1.0
        transferred[:, index] = mode
    return transferred, mapping


def process_object(
    preregistration: dict[str, Any], source: dict[str, Any], object_id: str
) -> tuple[dict[str, Any], bytes]:
    frozen = V1.manifest_object(preregistration, object_id)
    profile = V1.source_profile(source, object_id)
    entry = next(
        item
        for item in profile["metadata_entries"]
        if item.get("name") == frozen.get("mesh_entry_name")
    )
    raw, fetch = V1.fetch_deflated_entry(
        str(frozen["archive_url"]), entry, str(frozen["mesh_raw_sha256"])
    )
    points, faces = V1.parse_obj(raw)
    welded_points, welded_faces, weld = exact_coordinate_weld(points, faces)
    welded_topology = V1.topology(welded_points, welded_faces)
    if not welded_topology.passed:
        raise V1.PreflightError("exact-coordinate welded original topology gate failed")
    spectral_points, spectral_faces, spectral_collapses = V1.simplify_mesh(
        welded_points, welded_faces, V1.SPECTRAL_FACE_TARGET
    )
    spectral_topology = V1.topology(spectral_points, spectral_faces)
    if not spectral_topology.passed:
        raise V1.PreflightError("exact-coordinate welded spectral topology gate failed")
    bem_points, bem_faces, bem_collapses = V1.simplify_mesh(
        spectral_points, spectral_faces, V1.BEM_FACE_TARGET
    )
    bem_topology = V1.topology(bem_points, bem_faces)
    if not bem_topology.passed:
        raise V1.PreflightError("exact-coordinate welded BEM topology gate failed")
    stiffness, mass_matrix, spectral_mass = V1.cotangent_system(
        spectral_points, spectral_faces
    )
    eigenvalues, modes, residuals = V1.canonical_eigenmodes(
        stiffness, mass_matrix, spectral_mass
    )
    transferred, mapping = transfer_modes_via_frozen_replay(
        spectral_points,
        spectral_faces,
        spectral_mass,
        modes,
        bem_points,
        bem_faces,
        bem_collapses,
    )
    arrays = [
        ("welded_points", welded_points),
        ("welded_faces", welded_faces),
        ("spectral_points", spectral_points),
        ("spectral_faces", spectral_faces),
        ("spectral_collapses", spectral_collapses),
        ("bem_points", bem_points),
        ("bem_faces", bem_faces),
        ("bem_collapses", bem_collapses),
        ("spectral_to_bem_mapping", mapping),
        ("eigenvalues", eigenvalues),
        ("eigenmodes", modes),
        ("relative_eigen_residuals", residuals),
        ("bem_modes", transferred),
    ]
    block = V1.encode_geometry_block(arrays)
    return {
        "object_id": object_id,
        "role": frozen["role"],
        "decision": "ExactCoordinateWeldGeometryPreflightSupported",
        "audio_entry_name_bound_but_not_read": frozen["audio_entry_name"],
        "fetch": fetch,
        "weld": weld,
        "welded_original": welded_topology.as_json(),
        "spectral": spectral_topology.as_json(),
        "bem": bem_topology.as_json(),
        "maximum_relative_eigen_residual": float(np.max(residuals)),
        "eigenvalue_minimum_per_square_metre": float(np.min(eigenvalues)),
        "eigenvalue_maximum_per_square_metre": float(np.max(eigenvalues)),
        "arrays": {name: V1.array_summary(value) for name, value in arrays},
        "block": {
            "path": f"{object_id}-geometry-block-v2.bin",
            "bytes": len(block),
            "sha256": V1.sha256_bytes(block),
        },
    }, block


def publish(
    output: Path,
    geometry_manifest: bytes,
    preregistration_manifest: bytes,
    preregistration_report: bytes,
    v1_report: bytes,
    report: bytes,
    blocks: dict[str, bytes],
) -> None:
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise V1.PreflightError(f"output must be absent or empty: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.parent / f".{output.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    try:
        (staging / "geometry-v2-manifest.json").write_bytes(geometry_manifest)
        (staging / "preregistration-manifest.json").write_bytes(
            preregistration_manifest
        )
        (staging / "preregistration-report.json").write_bytes(preregistration_report)
        (staging / "geometry-v1-report.json").write_bytes(v1_report)
        for name, payload in blocks.items():
            (staging / name).write_bytes(payload)
        (staging / "report.json").write_bytes(report)
        if output.exists():
            output.rmdir()
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--geometry-manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    arguments = parser.parse_args()
    geometry_path = arguments.geometry_manifest.resolve()
    geometry_bytes, geometry = V1.read_json(
        geometry_path, MANIFEST_SHA256, "geometry V2 manifest"
    )
    validate_manifest(geometry)
    references = {item["id"]: item for item in geometry["prerequisites"]}

    def read_reference(identifier: str, expected_hash: str) -> tuple[bytes, dict[str, Any]]:
        reference = references.get(identifier)
        if reference is None or reference.get("sha256") != expected_hash:
            raise V1.PreflightError(f"geometry V2 prerequisite changed: {identifier}")
        path = (geometry_path.parent / reference["path"]).resolve()
        return V1.read_json(path, expected_hash, identifier)

    preregistration_bytes, preregistration = read_reference(
        "real-spatial-transfer-preregistration-manifest",
        PREREGISTRATION_MANIFEST_SHA256,
    )
    preregistration_report_bytes, preregistration_report = read_reference(
        "real-spatial-transfer-preregistration-report", PREREGISTRATION_REPORT_SHA256
    )
    v1_report_bytes, v1_report = read_reference(
        "geometry-preflight-v1-rejection", V1_REPORT_SHA256
    )
    V1.validate_inputs(preregistration, preregistration_report)
    if (
        v1_report.get("decision") != "GeometryOnlyPreflightRejected"
        or v1_report.get("reserved_audio_payload_bytes_read") != 0
    ):
        raise V1.PreflightError("geometry V1 rejection does not preserve sealed audio")
    source_path, source = V1.resolve_source_manifest(geometry_path.parent / "preregistration-manifest.json", preregistration)
    results: list[dict[str, Any]] = []
    blocks: dict[str, bytes] = {}
    for object_id in V1.EXPECTED_OBJECTS:
        result, block = process_object(preregistration, source, object_id)
        results.append(result)
        blocks[f"{object_id}-geometry-block-v2.bin"] = block
    supported = all(
        result["decision"] == "ExactCoordinateWeldGeometryPreflightSupported"
        for result in results
    )
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": (
            "ExactCoordinateWeldGeometryPreflightSupported"
            if supported
            else "ExactCoordinateWeldGeometryPreflightRejected"
        ),
        "claim": "GEOMETRY_ONLY_EXACT_COORDINATE_WELD_PREFLIGHT / RESERVED_AUDIO_PAYLOAD_BYTES_READ_ZERO / NO_REAL_SPATIAL_TRANSFER_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_CREDIT",
        "study_id": V1.STUDY_ID,
        "protocol_revision": PROTOCOL_REVISION,
        "manifest_sha256": MANIFEST_SHA256,
        "preregistration_manifest_sha256": PREREGISTRATION_MANIFEST_SHA256,
        "preregistration_report_sha256": PREREGISTRATION_REPORT_SHA256,
        "v1_rejection_report_sha256": V1_REPORT_SHA256,
        "source_manifest": {"path": str(source_path), "sha256": V1.SOURCE_MANIFEST_SHA256},
        "implementation": {
            "script_sha256": V1.sha256_bytes(Path(__file__).read_bytes()),
            "shared_v1_script_sha256": V1_SCRIPT_SHA256,
            "python": sys.version.split()[0],
            "numpy": np.__version__,
            "scipy": V1.scipy.__version__,
            "fast_simplification": V1.fast_simplification.__version__,
            "fast_simplification_revision": V1.SIMPLIFIER_REVISION,
        },
        "single_change": geometry["single_change"],
        "network_requests": len(V1.EXPECTED_OBJECTS),
        "network_request_scope": "two transformed.obj raw-deflate entry ranges only",
        "mesh_compressed_bytes_read": sum(
            result["fetch"]["compressed_bytes_read"] for result in results
        ),
        "reserved_audio_payload_bytes_read": 0,
        "objects": results,
        "gate": {
            "every_exact_welded_original_topology_passed": all(
                result["welded_original"]["passed"] for result in results
            ),
            "every_spectral_topology_passed": all(
                result["spectral"]["passed"] for result in results
            ),
            "every_bem_topology_passed": all(
                result["bem"]["passed"] for result in results
            ),
            "every_object_has_64_modes": all(
                result["arrays"]["eigenvalues"]["shape"]
                == [V1.NONCONSTANT_MODE_COUNT]
                for result in results
            ),
            "every_relative_eigen_residual_passed": all(
                result["maximum_relative_eigen_residual"]
                <= V1.MAX_RELATIVE_EIGEN_RESIDUAL
                for result in results
            ),
            "reserved_audio_payload_bytes_zero": True,
        },
        "allowed_claims": geometry["allowed_claims"],
        "prohibited_claims": geometry["prohibited_claims"],
        "next_action": "freeze these exact byte-identical geometry blocks into a separate Pitcher calibration manifest before any audio access",
    }
    report_bytes = V1.pretty_json(report)
    publish(
        arguments.output.resolve(),
        geometry_bytes,
        preregistration_bytes,
        preregistration_report_bytes,
        v1_report_bytes,
        report_bytes,
        blocks,
    )
    print(f"REALIMPACT exact-weld geometry preflight: {arguments.output.resolve()}")
    print(f"decision: {report['decision']}")
    print("reserved audio payload bytes read: 0")
    print(f"report sha256: {V1.sha256_bytes(report_bytes)}")
    for result in results:
        print(
            f"{result['object_id']}: vertices {result['weld']['input_vertex_count']} -> "
            f"{result['weld']['exact_unique_vertex_count']}, "
            f"max residual {result['maximum_relative_eigen_residual']:.3e}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
