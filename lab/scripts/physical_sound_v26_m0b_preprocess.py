"""M0b preprocessing delta: surface fields plus bounded padded alignment."""

from __future__ import annotations

from typing import Any

import numpy as np
import physical_sound_v25_m0a_common as base
import physical_sound_v25_m0a_train as inherited_train
import physical_sound_v26_m0b_alignment as alignment
import physical_sound_v26_m0b_common as contract
import physical_sound_v26_m0b_surface as surface


class Preprocessor(inherited_train.Preprocessor):
    def __init__(
        self,
        manifest: dict[str, Any],
        combined: dict[str, Any],
        teacher_evidence: dict[str, Any],
        profile: base.ExecutionProfile,
    ) -> None:
        super().__init__(manifest, combined, teacher_evidence, profile)
        self.surface_cache: dict[str, surface.SurfaceEvaluator] = {}
        self.surface_query_count = 0
        self.alignment_records: dict[str, dict[str, Any]] = {}
        self.alignment_conformance = alignment.conformance_report(
            manifest["implementation_root_sha256"]
        )

    def _surface(self, mesh: base.Mesh) -> surface.SurfaceEvaluator:
        if mesh.sha256 not in self.surface_cache:
            self.surface_cache[mesh.sha256] = surface.SurfaceEvaluator(mesh)
        return self.surface_cache[mesh.sha256]

    def _record_surface(
        self, mesh: base.Mesh, point: np.ndarray, sample: surface.SurfaceSample
    ) -> None:
        source_hash = base.sha256_bytes(
            bytes.fromhex(mesh.sha256) + point.astype("<f8", copy=False).tobytes()
        )
        value = np.asarray(
            [
                *sample.face_vertices,
                *sample.weights,
                *sample.projected_point_metres,
                sample.plane_residual_metres,
                sample.reconstruction_residual_metres,
                sample.surface_tolerance_metres,
            ],
            dtype=np.float64,
        )
        self._record(
            "surface-query-barycentric-v1", source_hash, value, "indices-weights-metres"
        )
        self.surface_query_count += 1

    def synthetic(self, row: dict[str, Any]) -> inherited_train.PreparedSynthetic:
        axes = row.get("axes", {})
        target = axes.get("teacher_target", {})
        geometry = axes.get("geometry", {})
        impact = axes.get("impact", {})
        material = axes.get("material", {}).get("value_id")
        support = axes.get("support", {}).get("value_id")
        if material not in base.MATERIAL_PHYSICS or support not in base.SUPPORT_IDS:
            raise base.M0Error("M0b teacher material or support is unknown")
        mesh_ref = geometry.get("feature_artifact")
        mesh = self._mesh(mesh_ref, self.t0_root, "M0b teacher mesh")
        modes = self._modes(target.get("modal_parameters"))
        gains = self._gains(
            target.get("contact_gain_field"), mesh, modes, "M0b teacher gain field"
        )
        point = np.asarray(impact.get("point_metres"), dtype=np.float64)
        fine_sample, interpolated_gains = self._surface(mesh).interpolate(
            point,
            gains.values,
            mesh_sha256=gains.mesh_sha256,
            channel_count=len(modes.frequencies_hz),
        )
        self._record_surface(mesh, point, fine_sample)
        object_features = base.full_object_features(mesh, material, support)
        contact_features = base.contact_descriptor(mesh, point)
        count = len(modes.frequencies_hz)
        frequency = np.empty(self.profile.mode_count, dtype=np.float32)
        decay = np.empty(self.profile.mode_count, dtype=np.float32)
        gain = np.zeros(self.profile.mode_count, dtype=np.float32)
        mask = np.zeros(self.profile.mode_count, dtype=np.float32)
        frequency[:count] = modes.frequencies_hz
        decay[:count] = modes.decay_per_second
        gain[:count] = interpolated_gains
        mask[:count] = 1.0
        if count < self.profile.mode_count:
            tail = np.geomspace(
                max(float(frequency[count - 1]) * 1.01, 20.0),
                17_500.0,
                self.profile.mode_count - count + 1,
            )[1:]
            frequency[count:] = tail
            decay[count:] = float(decay[count - 1])
        _, audio_data = self._data(row["audio"], self.t0_root, "M0b teacher audio")
        _, waveform = base.decode_float_wav(audio_data)
        object_id = row["object_group_id"].removeprefix("v24-t0-object-")
        report = self.object_reports.get(object_id)
        if not isinstance(report, dict) or report.get("material_id") != material:
            raise base.M0Error("M0b teacher object report binding changed")
        coarse_mesh_path = self.t0_root / f"objects/{object_id}/mesh-coarse.bin"
        coarse_gain_path = (
            self.t0_root / f"objects/{object_id}/contact-gain-field-coarse.bin"
        )
        coarse_mesh_data = base.external_file(
            coarse_mesh_path, "M0b coarse mesh"
        ).read_bytes()
        coarse_mesh = base.parse_mesh(coarse_mesh_data)
        coarse_gain_data = base.external_file(
            coarse_gain_path, "M0b coarse gain field"
        ).read_bytes()
        coarse_gains = base.parse_gain_field(coarse_gain_data, coarse_mesh, modes)
        coarse_sample, _ = self._surface(coarse_mesh).interpolate(
            point,
            coarse_gains.values,
            mesh_sha256=coarse_gains.mesh_sha256,
            channel_count=len(modes.frequencies_hz),
        )
        self._record_surface(coarse_mesh, point, coarse_sample)
        coarse_object = base.full_object_features(coarse_mesh, material, support)
        coarse_contact = base.contact_descriptor(coarse_mesh, point)
        self._record(
            "geometry-descriptor-v2", mesh.sha256, object_features, "fixed-mixed"
        )
        self._record(
            "contact-descriptor-v1",
            base.sha256_bytes(point.astype("<f8").tobytes()),
            contact_features,
            "normalized",
        )
        self._record(
            "teacher-waveform-f32-v1", row["audio"]["sha256"], waveform, "amplitude"
        )
        example = base.SyntheticExample(
            row["row_id"],
            row["split_role"],
            row["sample_role"],
            object_id,
            object_features,
            contact_features,
            frequency,
            decay,
            gain,
            mask,
        )
        return inherited_train.PreparedSynthetic(
            example,
            waveform.astype(np.float32),
            mesh,
            coarse_mesh,
            coarse_object,
            coarse_contact,
            point,
            material,
            support,
            report["family"],
        )

    def transfer(self, row: dict[str, Any]) -> base.RealTransferExample:
        axes = row.get("axes", {})
        if set(axes) != {"material", "geometry", "impact", "listener"}:
            raise base.M0Error("M0b real transfer axes changed or were fabricated")
        material = axes["material"].get("value_id")
        if material != "glass":
            raise base.M0Error("M0b X0 semantic material changed")
        mesh = self._mesh(
            axes["geometry"]["feature_artifact"], self.x0_root, "M0b X0 mesh"
        )
        point = np.asarray(axes["impact"].get("point_metres"), dtype=np.float64)
        sample = self._surface(mesh).locate(point)
        self._record_surface(mesh, point, sample)
        object_features = base.full_object_features(mesh, material, None)
        if not np.array_equal(object_features[29:35], np.zeros(6, dtype=np.float32)):
            raise base.M0Error("M0b fabricated X0 physical material values")
        contact_features = base.contact_descriptor(mesh, point)
        _, data = self._data(row["audio"], self.x0_root, "M0b X0 transfer")
        aligned = alignment.align_transfer(data, self.profile)
        samples = aligned.samples
        self._record(
            "realimpact-peak-anchor-padded-v1",
            row["audio"]["sha256"],
            samples,
            "amplitude",
        )
        self.alignment_records[row["row_id"]] = {
            "row_id": row["row_id"],
            **aligned.canonical_record(row["audio"]["sha256"]),
        }
        return base.RealTransferExample(
            row["row_id"],
            row["sample_role"],
            object_features,
            contact_features,
            samples,
        )

    def manifest_record(self, source_hashes: dict[str, str]) -> dict[str, Any]:
        record = super().manifest_record(source_hashes)
        record["schema"] = contract.PREPROCESS_SCHEMA
        record["surface_query_transform"] = "surface-query-barycentric-v1"
        record["surface_query_count_before_candidate_freeze"] = self.surface_query_count
        record["nearest_vertex_gain_lookup_used"] = False
        record["alignment_transform"] = "realimpact-peak-anchor-padded-v1"
        record["alignment_conformance"] = self.alignment_conformance
        record["alignment_record_count"] = len(self.alignment_records)
        record["alignment_records"] = [
            self.alignment_records[key] for key in sorted(self.alignment_records)
        ]
        record["unpadded_alignment_used"] = False
        return record
