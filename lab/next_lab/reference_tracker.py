from __future__ import annotations

import hashlib
import json
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable, Mapping, Sequence

import numpy as np
from numpy.typing import NDArray


PROFILE_ID = "nextengine.motor.env.humanoid-reference-tracker.v1"
PROFILE_SHA256 = "4a898ccf67051b34b6266ec5293f74758103e7db260ded393e76161f72de527d"
SOFT_ROM_COST_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-soft-rom-cost.v1"
)
SOFT_ROM_COST_PROFILE_SHA256 = (
    "c482e68f05ad574b74ba037412a5d8b1d378966ac788de308b457885f7b0c35b"
)
PREDICTIVE_ROM_COST_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-predictive-rom-cost.v1"
)
PREDICTIVE_ROM_COST_PROFILE_SHA256 = (
    "2640aa58886b00c901240f9f2b8912cfcff74e5a8490e846ad69b35b4fedc3b5"
)
SAFETY_RESERVE_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-safety-reserve.v2"
)
SAFETY_RESERVE_PROFILE_SHA256 = (
    "c16662efd96977fd037d2db3cb3a88fa8ee3f8441eef5848ee2fe0f62068287c"
)
DYNAMIC_RESERVE_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-dynamic-reserve.v3"
)
DYNAMIC_RESERVE_PROFILE_SHA256 = (
    "b2840e93d858047a6267db1b6c01479c78e59b8866520e979db9c50082edf11c"
)
PHYSICS_VELOCITY_GUARD_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-physics-velocity-guard.v4"
)
PHYSICS_VELOCITY_GUARD_PROFILE_SHA256 = (
    "7061e43bc59097312c10e90ea566485116bca4b5ec40ab1e93e22919b2160b5d"
)
CONTACT_IMPACT_MARGIN_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-contact-impact-margin.v5"
)
CONTACT_IMPACT_MARGIN_PROFILE_SHA256 = (
    "6a8b7c5871c200377cec4895ebefe370861f83c20a060e77ea9055f88e82ca06"
)
TEMPORAL_CONTACT_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-temporal-contact.v6"
)
TEMPORAL_CONTACT_PROFILE_SHA256 = (
    "d7131e909586a140dc541d3c8580744dede485b1e9693fa48153d8ab30c16281"
)
STANCE_CHAIN_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-stance-chain.v7"
)
STANCE_CHAIN_PROFILE_SHA256 = (
    "3d429ccfc36831b1262ea30ae19e8a89b2efcdb04be2e9e6026f8518d59aa84d"
)
SWING_CLEARANCE_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-swing-clearance.v8"
)
SWING_CLEARANCE_PROFILE_SHA256 = (
    "ace9e32ec0777043b4f963353c2889b639cde03531a64ec3d4b10d3eeab69ae6"
)
CONTACT_SEATED_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-contact-seated.v9"
)
CONTACT_SEATED_PROFILE_SHA256 = (
    "4dd3a5afa237d595c904c85d79f733ee6ee6a441c74f8f6be364a85aaf7300c0"
)
SOLE_NORMAL_PROFILE_ID = (
    "nextengine.motor.env.humanoid-reference-tracker-sole-normal-closure.v10"
)
SOLE_NORMAL_PROFILE_SHA256 = (
    "25e81e1edeea13ea8c608be85eb4f06600378f8db6e3580860eae9cddbdd802a"
)
PROFILE_IDS_BY_SHA256 = {
    PROFILE_SHA256: PROFILE_ID,
    SOFT_ROM_COST_PROFILE_SHA256: SOFT_ROM_COST_PROFILE_ID,
    PREDICTIVE_ROM_COST_PROFILE_SHA256: PREDICTIVE_ROM_COST_PROFILE_ID,
    SAFETY_RESERVE_PROFILE_SHA256: SAFETY_RESERVE_PROFILE_ID,
    DYNAMIC_RESERVE_PROFILE_SHA256: DYNAMIC_RESERVE_PROFILE_ID,
    PHYSICS_VELOCITY_GUARD_PROFILE_SHA256: PHYSICS_VELOCITY_GUARD_PROFILE_ID,
    CONTACT_IMPACT_MARGIN_PROFILE_SHA256: CONTACT_IMPACT_MARGIN_PROFILE_ID,
    TEMPORAL_CONTACT_PROFILE_SHA256: TEMPORAL_CONTACT_PROFILE_ID,
    STANCE_CHAIN_PROFILE_SHA256: STANCE_CHAIN_PROFILE_ID,
    SWING_CLEARANCE_PROFILE_SHA256: SWING_CLEARANCE_PROFILE_ID,
    CONTACT_SEATED_PROFILE_SHA256: CONTACT_SEATED_PROFILE_ID,
    SOLE_NORMAL_PROFILE_SHA256: SOLE_NORMAL_PROFILE_ID,
}
SAFETY_CONTACT_PROFILE_SHA256 = "ad20d7a4abd5cc8b59069ecdb59161499ce7754953cbff2477f2850395adb42c"
SAFETY_CONTACT_PROFILE_V2_SHA256 = (
    "ba9d368e075f389a4dbff4a0ed9299b737edf4907be10ae6cf3aeb60b348729f"
)
ISAAC_VELOCITY_GUARD_PROFILE_SHA256 = (
    "a5c8448a71f0adc2c821b8d833332e0ecd1e003110ce47d969f56cac13881676"
)
CORPUS_MANIFEST_ID = "nextengine.private-motion-corpus-manifest.v1"
OBSERVATION_CHANNELS = 435
ACTION_CHANNELS = 23
REFERENCE_OFFSETS = (0, 4, 8, 16)
RESET_MODES = ("exact_reference", "small_perturbation", "neutral_entry", "near_failure")
SPLITS = ("train", "validation", "heldout")
NEUTRAL_ENTRY_SKILLS = frozenset(("neutral_idle", "start_left_lead", "start_right_lead"))
REQUIRED_ARRAYS: Mapping[str, tuple[tuple[int | None, ...], np.dtype[Any]]] = {
    "root_position_um": ((None, 3), np.dtype(np.int64)),
    "root_quaternion_q1_30": ((None, 4), np.dtype(np.int64)),
    "root_linear_velocity_um_s": ((None, 3), np.dtype(np.int64)),
    "root_yaw_velocity_urad_s": ((None,), np.dtype(np.int64)),
    "joint_position_urad": ((None, ACTION_CHANNELS), np.dtype(np.int64)),
    "joint_velocity_urad_s": ((None, ACTION_CHANNELS), np.dtype(np.int64)),
    "center_of_mass_um": ((None, 3), np.dtype(np.int64)),
    "effector_position_um": ((None, 6, 3), np.dtype(np.int64)),
    "contacts": ((None, 7), np.dtype(np.uint8)),
    "phase_u16": ((None,), np.dtype(np.uint16)),
}


class ReferenceTrackerError(ValueError):
    pass


def _sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def _canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"
    ).encode("utf-8")


def _canonical_manifest_hash(manifest: Mapping[str, Any]) -> str:
    value = dict(manifest)
    value.pop("manifest_sha256", None)
    return _sha256(_canonical_json(value))


def _require_hex_hash(value: Any, label: str) -> str:
    if not isinstance(value, str) or len(value) != 64:
        raise ReferenceTrackerError(f"{label} must be a lowercase SHA-256 hex string")
    try:
        decoded = bytes.fromhex(value)
    except ValueError as error:
        raise ReferenceTrackerError(f"{label} must be a lowercase SHA-256 hex string") from error
    if len(decoded) != 32 or value != value.lower():
        raise ReferenceTrackerError(f"{label} must be a lowercase SHA-256 hex string")
    return value


@dataclass(frozen=True)
class ReferenceTrackerProfile:
    document: Mapping[str, Any]
    document_sha256: str

    @classmethod
    def load(cls, path: Path) -> ReferenceTrackerProfile:
        payload = path.read_bytes()
        sha256 = _sha256(payload)
        expected_profile_id = PROFILE_IDS_BY_SHA256.get(sha256)
        if expected_profile_id is None:
            raise ReferenceTrackerError(
                "reference tracker profile hash mismatch: "
                f"expected one of {sorted(PROFILE_IDS_BY_SHA256)}, got {sha256}"
            )
        document = json.loads(payload)
        if "base_profile_sha256" in document:
            document = cls._materialize_variant(
                document,
                path=path,
                expected_profile_id=expected_profile_id,
            )
        cls._validate(document, expected_profile_id=expected_profile_id)
        return cls(document=document, document_sha256=sha256)

    @staticmethod
    def _materialize_variant(
        overlay: Mapping[str, Any], *, path: Path, expected_profile_id: str
    ) -> Mapping[str, Any]:
        variant_kind = overlay.get("variant", {}).get("kind")
        if variant_kind in {
            "replace-input-lineage.v1",
            "replace-input-lineage-and-append-reward-component.v1",
        }:
            if (
                overlay.get("schema_version") != 1
                or overlay.get("profile_id") != expected_profile_id
                or expected_profile_id
                not in {
                    SAFETY_RESERVE_PROFILE_ID,
                    DYNAMIC_RESERVE_PROFILE_ID,
                    PHYSICS_VELOCITY_GUARD_PROFILE_ID,
                    CONTACT_IMPACT_MARGIN_PROFILE_ID,
                    TEMPORAL_CONTACT_PROFILE_ID,
                    STANCE_CHAIN_PROFILE_ID,
                    SWING_CLEARANCE_PROFILE_ID,
                    CONTACT_SEATED_PROFILE_ID,
                    SOLE_NORMAL_PROFILE_ID,
                }
                or overlay.get("status") != "Frozen"
                or overlay.get("base_profile_id") != PROFILE_ID
                or overlay.get("base_profile_sha256") != PROFILE_SHA256
            ):
                raise ReferenceTrackerError("invalid input-lineage tracker variant")
            base_path = path.with_name("humanoid-reference-tracker.v1.json")
            base_payload = base_path.read_bytes()
            if _sha256(base_payload) != PROFILE_SHA256:
                raise ReferenceTrackerError("reference tracker variant base hash mismatch")
            document = json.loads(base_payload)
            document["profile_id"] = expected_profile_id
            for field in ("corpus", "training_authorization", "termination"):
                replacement = overlay["variant"].get(field)
                if not isinstance(replacement, dict):
                    raise ReferenceTrackerError(
                        f"input-lineage tracker variant has no {field}"
                    )
                document[field] = dict(replacement)
            if variant_kind == "replace-input-lineage-and-append-reward-component.v1":
                if expected_profile_id != CONTACT_IMPACT_MARGIN_PROFILE_ID:
                    raise ReferenceTrackerError("invalid impact-margin tracker variant")
                component = overlay["variant"].get("component")
                if not isinstance(component, dict):
                    raise ReferenceTrackerError(
                        "impact-margin tracker variant has no component"
                    )
                components = list(document["reward"]["components"])
                if components[-1]["id"] != "reward.terminal-failure":
                    raise ReferenceTrackerError("reference tracker terminal reward moved")
                components.insert(-1, dict(component))
                document["reward"]["components"] = components
            return document
        if (
            overlay.get("schema_version") != 1
            or overlay.get("profile_id") != expected_profile_id
            or overlay.get("status") != "Frozen"
            or overlay.get("base_profile_id") != PROFILE_ID
            or overlay.get("base_profile_sha256") != PROFILE_SHA256
            or overlay.get("variant", {}).get("kind")
            != "append-reward-component-before-terminal.v1"
        ):
            raise ReferenceTrackerError("invalid reference tracker profile variant")
        component = overlay["variant"].get("component")
        if not isinstance(component, dict):
            raise ReferenceTrackerError("reference tracker variant has no component")
        base_path = path.with_name("humanoid-reference-tracker.v1.json")
        base_payload = base_path.read_bytes()
        if _sha256(base_payload) != PROFILE_SHA256:
            raise ReferenceTrackerError("reference tracker variant base hash mismatch")
        document = json.loads(base_payload)
        document["profile_id"] = expected_profile_id
        components = list(document["reward"]["components"])
        if components[-1]["id"] != "reward.terminal-failure":
            raise ReferenceTrackerError("reference tracker terminal reward moved")
        components.insert(-1, dict(component))
        document["reward"]["components"] = components
        return document

    @staticmethod
    def _validate(
        document: Mapping[str, Any], *, expected_profile_id: str = PROFILE_ID
    ) -> None:
        if (
            document.get("schema_version") != 1
            or document.get("profile_id") != expected_profile_id
            or document.get("status") != "Frozen"
        ):
            raise ReferenceTrackerError("unsupported reference tracker profile")
        body = document["body_schema"]
        corpus = document["corpus"]
        authorization = document["training_authorization"]
        for label, value in (
            ("body schema", body["hash"]),
            ("compiled descriptor", body["compiled_descriptor_hash"]),
            ("descriptor file", body["descriptor_file_sha256"]),
            ("corpus profile", corpus["profile_sha256"]),
            ("corpus manifest", corpus["manifest_sha256"]),
            ("gate report", authorization["gate_report_sha256"]),
            (
                "safety contact profile",
                document["termination"]["safety_contact_profile_sha256"],
            ),
        ):
            _require_hex_hash(value, label)
        termination = document["termination"]
        expected_safety_contact_profile = (
            SAFETY_CONTACT_PROFILE_V2_SHA256
            if expected_profile_id
            in {
                SAFETY_RESERVE_PROFILE_ID,
                DYNAMIC_RESERVE_PROFILE_ID,
                PHYSICS_VELOCITY_GUARD_PROFILE_ID,
                CONTACT_IMPACT_MARGIN_PROFILE_ID,
                TEMPORAL_CONTACT_PROFILE_ID,
                STANCE_CHAIN_PROFILE_ID,
                SWING_CLEARANCE_PROFILE_ID,
                CONTACT_SEATED_PROFILE_ID,
                SOLE_NORMAL_PROFILE_ID,
            }
            else SAFETY_CONTACT_PROFILE_SHA256
        )
        if (
            document["termination"]["safety_contact_profile_sha256"]
            != expected_safety_contact_profile
        ):
            raise ReferenceTrackerError("safety contact profile mismatch")
        if expected_profile_id in {
            PHYSICS_VELOCITY_GUARD_PROFILE_ID,
            CONTACT_IMPACT_MARGIN_PROFILE_ID,
            TEMPORAL_CONTACT_PROFILE_ID,
            STANCE_CHAIN_PROFILE_ID,
            SWING_CLEARANCE_PROFILE_ID,
            CONTACT_SEATED_PROFILE_ID,
            SOLE_NORMAL_PROFILE_ID,
        }:
            if (
                _require_hex_hash(
                    termination.get("isaac_velocity_guard_profile_sha256"),
                    "Isaac velocity guard profile",
                )
                != ISAAC_VELOCITY_GUARD_PROFILE_SHA256
                or termination.get("isaac_physics_velocity_limit_basis_points")
                != 9_000
            ):
                raise ReferenceTrackerError("Isaac velocity guard profile mismatch")
        elif (
            "isaac_velocity_guard_profile_sha256" in termination
            or "isaac_physics_velocity_limit_basis_points" in termination
        ):
            raise ReferenceTrackerError("unexpected Isaac velocity guard profile")
        schedule = document["schedule"]
        if (
            schedule["physics_hz"] != 240
            or schedule["motor_hz"] != 60
            or schedule["reference_hz"] != 60
            or tuple(schedule["horizon_offsets_motor_ticks"]) != REFERENCE_OFFSETS
            or schedule["cursor_step_frames"] != 1
        ):
            raise ReferenceTrackerError("unsupported reference schedule")
        reset = document["reset"]
        weights = reset["mode_weights_basis_points"]
        if (
            tuple(weights) != RESET_MODES
            or sum(int(weights[mode]) for mode in RESET_MODES) != 10_000
            or weights["near_failure"] != 0
            or reset["near_failure_enabled"] is not False
            or frozenset(reset["neutral_entry_skills"]) != NEUTRAL_ENTRY_SKILLS
        ):
            raise ReferenceTrackerError("unsupported reset profile")
        observation = document["observation"]
        joint_order = observation["joint_state_order"]
        effector_order = observation["reference_effector_order"]
        contact_order = observation["contact_order"]
        dynamic_count = sum(int(value) for value in observation["dynamic_channels"].values())
        reference_count = sum(
            int(value) for value in observation["reference_channels_per_sample"].values()
        )
        computed_count = (
            dynamic_count
            + int(observation["phase_channels"])
            + int(observation["reference_sample_count"]) * reference_count
        )
        if (
            dynamic_count != 86
            or reference_count != 87
            or computed_count != OBSERVATION_CHANNELS
            or observation["actor_channel_count"] != OBSERVATION_CHANNELS
            or observation["critic_channel_count"] != OBSERVATION_CHANNELS
            or observation["critic_privileged_channels"]
            or len(joint_order) != ACTION_CHANNELS
            or len(set(joint_order)) != ACTION_CHANNELS
            or len(effector_order) != 6
            or len(set(effector_order)) != 6
            or len(contact_order) != 7
            or len(set(contact_order)) != 7
        ):
            raise ReferenceTrackerError("reference observation layout mismatch")
        action = document["action"]
        actuator_order = action["ordered_actuator_ids"]
        action_to_dof = action["reference_joint_dof_ordinal_by_action_channel"]
        if (
            action["channel_count"] != ACTION_CHANNELS
            or action["minimum_raw"] != -(1 << 30)
            or action["maximum_raw"] != 1 << 30
            or len(actuator_order) != ACTION_CHANNELS
            or len(set(actuator_order)) != ACTION_CHANNELS
            or sorted(action_to_dof) != list(range(ACTION_CHANNELS))
            or any(
                actuator_id.removeprefix("actuator.")
                != joint_order[dof].removeprefix("joint.")
                for actuator_id, dof in zip(actuator_order, action_to_dof, strict=True)
            )
        ):
            raise ReferenceTrackerError("reference action layout mismatch")
        component_ids = tuple(
            component["id"] for component in document["reward"]["components"]
        )
        expected_component_ids = (
            "reward.reference-root-orientation",
            "reward.reference-root-height",
            "reward.reference-root-linear-velocity",
            "reward.reference-root-angular-velocity",
            "reward.reference-joint-pose",
            "reward.reference-joint-velocity",
            "reward.reference-center-of-mass",
            "reward.reference-effectors",
            "reward.reference-contacts",
            "reward.contacting-sole-slip-cost",
            "reward.normalized-applied-effort-cost",
            "reward.applied-target-rate-cost",
            *(
                ("reward.soft-rom-excursion-cost",)
                if expected_profile_id == SOFT_ROM_COST_PROFILE_ID
                else ()
            ),
            *(
                ("reward.predictive-rom-excursion-cost",)
                if expected_profile_id == PREDICTIVE_ROM_COST_PROFILE_ID
                else ()
            ),
            *(
                ("reward.contact-impact-margin-cost",)
                if expected_profile_id == CONTACT_IMPACT_MARGIN_PROFILE_ID
                else ()
            ),
            "reward.terminal-failure",
        )
        if component_ids != expected_component_ids:
            raise ReferenceTrackerError("reference reward component closure mismatch")
        if expected_profile_id == CONTACT_IMPACT_MARGIN_PROFILE_ID:
            component = next(
                item
                for item in document["reward"]["components"]
                if item["id"] == "reward.contact-impact-margin-cost"
            )
            if component != {
                "id": "reward.contact-impact-margin-cost",
                "coefficient_q16": -65_536,
                "warning_basis_points": 9_500,
                "calibration_evidence_sha256": (
                    "46203c49aaa073300d6bbf8fd23034382cc3e22288697db55d4333633e2cbacc"
                ),
                "aggregation": (
                    "maximum over contact pairs of the clamped linear excursion "
                    "from 9500 basis points of that pair's immutable hard-impact "
                    "limit to unit cost at the exact hard limit; zero below the "
                    "warning boundary"
                ),
            }:
                raise ReferenceTrackerError("impact-margin reward component mismatch")
        failure_reasons = document["termination"]["failure_reasons"]
        if not failure_reasons or len(set(failure_reasons)) != len(failure_reasons):
            raise ReferenceTrackerError("reference termination closure mismatch")
        remediation_only = expected_profile_id in {
            TEMPORAL_CONTACT_PROFILE_ID,
            STANCE_CHAIN_PROFILE_ID,
            SWING_CLEARANCE_PROFILE_ID,
            CONTACT_SEATED_PROFILE_ID,
            SOLE_NORMAL_PROFILE_ID,
        }
        if remediation_only:
            source_lineage = authorization.get("source_lineage")
            if (
                not isinstance(source_lineage, Mapping)
                or set(source_lineage)
                != {
                    "reference_tracker_profile_sha256",
                    "motion_corpus_profile_sha256",
                    "motion_corpus_manifest_sha256",
                }
                or any(
                    _require_hex_hash(value, f"remediation {field}") != value
                    for field, value in source_lineage.items()
                )
            ):
                raise ReferenceTrackerError(
                    "reference remediation source lineage mismatch"
                )
        if (
            corpus["eligible_partition"] != "locomotion"
            or authorization["gate_id"] != "TRAIN-4"
            or authorization["decision"]
            != ("RemediateDataOnly" if remediation_only else "Advance")
        ):
            raise ReferenceTrackerError("reference input authorization mismatch")

    def subprofile_hash(self, name: str) -> str:
        try:
            value = self.document[name]
        except KeyError as error:
            raise ReferenceTrackerError(f"unknown reference subprofile: {name}") from error
        return _sha256(_canonical_json(value))

    def contract_projection(self) -> dict[str, Any]:
        document = self.document
        termination = document["termination"]
        return {
            "schema_version": 1,
            "profile_id": document["profile_id"],
            "body_schema_hash": document["body_schema"]["hash"],
            "compiled_descriptor_hash": document["body_schema"]["compiled_descriptor_hash"],
            "corpus_profile_hash": document["corpus"]["profile_sha256"],
            "corpus_manifest_hash": document["corpus"]["manifest_sha256"],
            "profile_document_sha256": self.document_sha256,
            "observation_layout_hash": self.subprofile_hash("observation"),
            "action_layout_hash": self.subprofile_hash("action"),
            "reset_profile_hash": self.subprofile_hash("reset"),
            "reward_profile_hash": self.subprofile_hash("reward"),
            "termination_profile_hash": self.subprofile_hash("termination"),
            "isaac_velocity_guard_profile_sha256": termination.get(
                "isaac_velocity_guard_profile_sha256"
            ),
            "isaac_physics_velocity_limit_basis_points": termination.get(
                "isaac_physics_velocity_limit_basis_points", 10_000
            ),
            "rng_derivation_profile_hash": self.subprofile_hash("rng"),
            "eligible_partition_id": document["corpus"]["eligible_partition"],
            "clip_selection_stream_id": "randomization.reference-clip",
            "phase_selection_stream_id": "randomization.reference-phase",
            "reset_mode_stream_id": "randomization.reset-mode",
            "physics_hz": document["schedule"]["physics_hz"],
            "motor_hz": document["schedule"]["motor_hz"],
            "reference_hz": document["schedule"]["reference_hz"],
            "horizon_offsets_motor_ticks": list(REFERENCE_OFFSETS),
            "reset_mode_weights_basis_points": [
                document["reset"]["mode_weights_basis_points"][mode] for mode in RESET_MODES
            ],
            "actor_channel_count": OBSERVATION_CHANNELS,
            "critic_channel_count": OBSERVATION_CHANNELS,
            "action_channel_count": ACTION_CHANNELS,
            "success_reason_id": document["termination"]["success_reason"],
            "failure_reason_ids": sorted(document["termination"]["failure_reasons"]),
        }


@dataclass(frozen=True)
class DescriptorLimits:
    soft_rom_spans_urad: NDArray[np.int64]
    soft_minimum_urad: NDArray[np.int64]
    soft_maximum_urad: NDArray[np.int64]
    hard_minimum_urad: NDArray[np.int64]
    hard_maximum_urad: NDArray[np.int64]
    maximum_velocity_urad_s: NDArray[np.int64]
    maximum_effort_unm: NDArray[np.int64]
    maximum_target_delta_urad: NDArray[np.int64]
    document_sha256: str

    @classmethod
    def load(cls, profile: ReferenceTrackerProfile, path: Path) -> DescriptorLimits:
        payload = path.read_bytes()
        document_sha256 = _sha256(payload)
        body = profile.document["body_schema"]
        if document_sha256 != body["descriptor_file_sha256"]:
            raise ReferenceTrackerError("compiled descriptor file hash mismatch")
        document = json.loads(payload)
        if (
            document["body_schema_hash"] != body["hash"]
            or document["compiled_descriptor_hash"] != body["compiled_descriptor_hash"]
            or document["action_width"] != ACTION_CHANNELS
        ):
            raise ReferenceTrackerError("compiled descriptor identity mismatch")
        joints = sorted(document["joints"], key=lambda item: int(item["dof_ordinal"]))
        actuators = sorted(document["actuators"], key=lambda item: int(item["dof_ordinal"]))
        if [int(item["dof_ordinal"]) for item in joints] != list(range(ACTION_CHANNELS)):
            raise ReferenceTrackerError("compiled joint ordinals are not closed")
        if [int(item["dof_ordinal"]) for item in actuators] != list(range(ACTION_CHANNELS)):
            raise ReferenceTrackerError("compiled actuator ordinals are not closed")
        if [item["joint_id"] for item in joints] != profile.document["observation"][
            "joint_state_order"
        ]:
            raise ReferenceTrackerError("compiled joint order does not match the profile")
        actuator_by_id = {item["actuator_id"]: item for item in document["actuators"]}
        action = profile.document["action"]
        if set(actuator_by_id) != set(action["ordered_actuator_ids"]) or [
            int(actuator_by_id[actuator_id]["dof_ordinal"])
            for actuator_id in action["ordered_actuator_ids"]
        ] != action["reference_joint_dof_ordinal_by_action_channel"]:
            raise ReferenceTrackerError("compiled actuator order does not match the profile")
        soft_spans = np.asarray(
            [int(joint["soft_limit_microradians"][1]) - int(joint["soft_limit_microradians"][0]) for joint in joints],
            dtype=np.int64,
        )
        soft_minimum = np.asarray(
            [int(joint["soft_limit_microradians"][0]) for joint in joints],
            dtype=np.int64,
        )
        soft_maximum = np.asarray(
            [int(joint["soft_limit_microradians"][1]) for joint in joints],
            dtype=np.int64,
        )
        hard_minimum = np.asarray(
            [int(joint["hard_limit_microradians"][0]) for joint in joints],
            dtype=np.int64,
        )
        hard_maximum = np.asarray(
            [int(joint["hard_limit_microradians"][1]) for joint in joints],
            dtype=np.int64,
        )
        velocities = np.asarray(
            [int(joint["maximum_velocity_microradians_per_second"]) for joint in joints],
            dtype=np.int64,
        )
        maximum_effort = np.asarray(
            [max(abs(int(actuator["effort_micronewton_metres"][0])), abs(int(actuator["effort_micronewton_metres"][1]))) for actuator in actuators],
            dtype=np.int64,
        )
        maximum_delta = np.asarray(
            [max(abs(int(actuator["target_delta_microradians_per_motor_tick"][0])), abs(int(actuator["target_delta_microradians_per_motor_tick"][1]))) for actuator in actuators],
            dtype=np.int64,
        )
        if np.any(soft_spans <= 0) or np.any(velocities <= 0) or np.any(maximum_effort <= 0) or np.any(maximum_delta <= 0):
            raise ReferenceTrackerError("compiled descriptor contains an invalid normalization bound")
        return cls(
            soft_spans,
            soft_minimum,
            soft_maximum,
            hard_minimum,
            hard_maximum,
            velocities,
            maximum_effort,
            maximum_delta,
            document_sha256,
        )


@dataclass(frozen=True)
class ReferenceClip:
    clip_id: str
    skill: str
    split: str
    split_group_id: str
    arrays: Mapping[str, NDArray[Any]]
    metadata: Mapping[str, Any]

    @property
    def frame_count(self) -> int:
        return int(self.arrays["phase_u16"].shape[0])


@dataclass(frozen=True)
class EpisodeSelection:
    split: str
    episode_ordinal: int
    vector_slot: int
    reset_mode: str
    clip_id: str
    phase_frame: int
    clip_seed_sha256: str
    phase_seed_sha256: str
    reset_seed_sha256: str


class ReferenceCorpus:
    def __init__(
        self,
        profile: ReferenceTrackerProfile,
        root: Path,
        gate_report_path: Path,
    ) -> None:
        self.profile = profile
        self.root = root.resolve()
        gate_payload = gate_report_path.read_bytes()
        expected_gate = profile.document["training_authorization"]["gate_report_sha256"]
        if _sha256(gate_payload) != expected_gate:
            raise ReferenceTrackerError("TRAIN-4 gate report hash mismatch")
        self.gate_report = json.loads(gate_payload)
        self._validate_gate()
        manifest_path = self.root / "corpus-manifest.json"
        manifest_payload = manifest_path.read_bytes()
        self.manifest_file_sha256 = _sha256(manifest_payload)
        self.manifest = json.loads(manifest_payload)
        self._validate_manifest()
        self._entries = {
            entry["clip_id"]: entry
            for entry in self.manifest["clips"]
            if entry["partition"] == profile.document["corpus"]["eligible_partition"]
        }
        self._cache: dict[str, ReferenceClip] = {}

    def _validate_gate(self) -> None:
        expected = self.profile.document
        identities = self.gate_report.get("identities", {})
        authorization = self.gate_report.get("training_authorization", {})
        expected_authorization = expected["training_authorization"]
        optimizer_free_preacceptance = (
            expected["profile_id"]
            in {
                SAFETY_RESERVE_PROFILE_ID,
                DYNAMIC_RESERVE_PROFILE_ID,
                PHYSICS_VELOCITY_GUARD_PROFILE_ID,
                CONTACT_IMPACT_MARGIN_PROFILE_ID,
            }
            and authorization.get("authorized") is False
            and "optimizer execution" in authorization.get("forbidden", ())
            and "optimizer-free" in expected["training_authorization"]["scope"]
        )
        source_lineage = expected_authorization.get("source_lineage", {})
        remediation_only = (
            expected["profile_id"]
            in {
                TEMPORAL_CONTACT_PROFILE_ID,
                STANCE_CHAIN_PROFILE_ID,
                SWING_CLEARANCE_PROFILE_ID,
                CONTACT_SEATED_PROFILE_ID,
                SOLE_NORMAL_PROFILE_ID,
            }
            and expected_authorization["decision"] == "RemediateDataOnly"
            and self.gate_report.get("decision") == "RemediateDataOnly"
            and self.gate_report.get("stage_status") == "Reopened"
            and authorization.get("authorized") is False
            and "optimizer execution" in authorization.get("forbidden", ())
            and "optimizer-free corpus diagnosis"
            in authorization.get("allowed_scope", ())
            and identities.get("reference_tracker_profile_sha256")
            == source_lineage.get("reference_tracker_profile_sha256")
            and identities.get("motion_corpus_profile_sha256")
            == source_lineage.get("motion_corpus_profile_sha256")
            and identities.get("motion_corpus_manifest_sha256")
            == source_lineage.get("motion_corpus_manifest_sha256")
        )
        if (
            self.gate_report.get("gate_id") != "TRAIN-4"
            or not (
                remediation_only
                or (
                    self.gate_report.get("decision") == "Advance"
                    and (
                        authorization.get("authorized")
                        or optimizer_free_preacceptance
                    )
                )
            )
            or identities.get("body_schema_hash") != expected["body_schema"]["hash"]
            or identities.get("compiled_descriptor_hash")
            != expected["body_schema"]["compiled_descriptor_hash"]
            or (
                not remediation_only
                and identities.get("motion_corpus_profile_sha256")
                != expected["corpus"]["profile_sha256"]
            )
            or (
                not remediation_only
                and identities.get("corpus_manifest_sha256")
                != expected["corpus"]["manifest_sha256"]
            )
        ):
            raise ReferenceTrackerError("TRAIN-4 gate does not authorize this tracker input")

    def _validate_manifest(self) -> None:
        expected = self.profile.document
        manifest = self.manifest
        if (
            manifest.get("schema_version") != 1
            or manifest.get("manifest_id") != CORPUS_MANIFEST_ID
            or manifest.get("status") != "VALIDATED"
            or _canonical_manifest_hash(manifest) != manifest.get("manifest_sha256")
            or manifest.get("manifest_sha256") != expected["corpus"]["manifest_sha256"]
            or manifest["profile"]["id"] != expected["corpus"]["profile_id"]
            or manifest["profile"]["sha256"] != expected["corpus"]["profile_sha256"]
            or manifest["target"]["body_schema_hash"] != expected["body_schema"]["hash"]
            or manifest["target"]["compiled_descriptor_hash"]
            != expected["body_schema"]["compiled_descriptor_hash"]
            or manifest["target"]["descriptor_file_sha256"]
            != expected["body_schema"]["descriptor_file_sha256"]
        ):
            raise ReferenceTrackerError("motion corpus manifest closure mismatch")
        clip_ids = [entry["clip_id"] for entry in manifest["clips"]]
        if len(clip_ids) != len(set(clip_ids)):
            raise ReferenceTrackerError("motion corpus contains duplicate clip IDs")
        if any(entry["validation"]["status"] != "PASS" for entry in manifest["clips"]):
            raise ReferenceTrackerError("motion corpus contains a failed clip")

    @property
    def input_provenance_root(self) -> str:
        preimage = bytearray(b"nextengine.reference-tracker-input-provenance.v1\0")
        for value in (
            self.profile.document_sha256,
            self.profile.document["corpus"]["manifest_sha256"],
            self.profile.document["training_authorization"]["gate_report_sha256"],
            self.profile.document["body_schema"]["descriptor_file_sha256"],
        ):
            preimage.extend(bytes.fromhex(value))
        return _sha256(bytes(preimage))

    def clip_ids(self, split: str, skills: Iterable[str] | None = None) -> tuple[str, ...]:
        if split not in SPLITS:
            raise ReferenceTrackerError(f"unsupported corpus split: {split}")
        skill_set = None if skills is None else frozenset(skills)
        values = [
            clip_id
            for clip_id, entry in self._entries.items()
            if entry["split"] == split and (skill_set is None or entry["skill"] in skill_set)
        ]
        return tuple(sorted(values, key=lambda value: value.encode("utf-8")))

    def load_clip(self, clip_id: str) -> ReferenceClip:
        if clip_id in self._cache:
            return self._cache[clip_id]
        try:
            entry = self._entries[clip_id]
        except KeyError as error:
            raise ReferenceTrackerError(f"clip is not admitted for reference tracking: {clip_id}") from error
        relative_path = Path(entry["artifact"]["relative_path"])
        if relative_path.is_absolute() or ".." in relative_path.parts:
            raise ReferenceTrackerError("motion corpus artifact path escapes the store")
        path = (self.root / relative_path).resolve()
        try:
            path.relative_to(self.root)
        except ValueError as error:
            raise ReferenceTrackerError("motion corpus artifact path escapes the store") from error
        payload = path.read_bytes()
        if _sha256(payload) != entry["artifact"]["sha256"] or len(payload) != entry["artifact"]["bytes"]:
            raise ReferenceTrackerError(f"motion corpus artifact hash mismatch: {clip_id}")
        arrays: dict[str, NDArray[Any]] = {}
        with np.load(path, allow_pickle=False) as archive:
            if "metadata_json_utf8" not in archive.files:
                raise ReferenceTrackerError(f"motion corpus metadata missing: {clip_id}")
            metadata = json.loads(archive["metadata_json_utf8"].tobytes())
            frame_count = int(entry["reference_frame_count"])
            for name, (shape, dtype) in REQUIRED_ARRAYS.items():
                if name not in archive.files:
                    raise ReferenceTrackerError(f"motion corpus array missing: {clip_id}/{name}")
                value = np.array(archive[name], copy=True)
                expected_shape = tuple(frame_count if dimension is None else dimension for dimension in shape)
                if value.shape != expected_shape or value.dtype != dtype:
                    raise ReferenceTrackerError(f"motion corpus array shape/type mismatch: {clip_id}/{name}")
                value.flags.writeable = False
                arrays[name] = value
        expected = self.profile.document
        if (
            metadata["clip_id"] != clip_id
            or metadata["skill"] != entry["skill"]
            or metadata["partition"] != "locomotion"
            or metadata["split"] != entry["split"]
            or metadata["split_group_id"] != entry["split_group_id"]
            or metadata["rate_hz"] != 60
            or metadata["profile_sha256"] != expected["corpus"]["profile_sha256"]
            or metadata["target_descriptor_sha256"]
            != expected["body_schema"]["descriptor_file_sha256"]
            or metadata["joint_ids"] != expected["observation"]["joint_state_order"]
            or metadata["effector_ids"]
            != expected["observation"]["reference_effector_order"]
            or metadata["contact_ids"] != expected["observation"]["contact_order"]
        ):
            raise ReferenceTrackerError(f"motion corpus clip metadata mismatch: {clip_id}")
        clip = ReferenceClip(
            clip_id=clip_id,
            skill=entry["skill"],
            split=entry["split"],
            split_group_id=entry["split_group_id"],
            arrays=arrays,
            metadata=metadata,
        )
        self._cache[clip_id] = clip
        return clip

    def validate_all_artifacts(self) -> int:
        for clip_id in sorted(self._entries, key=lambda value: value.encode("utf-8")):
            self.load_clip(clip_id)
        return len(self._entries)

    def select_episode(
        self,
        *,
        run_root: bytes,
        split: str,
        episode_ordinal: int,
        vector_slot: int,
        skills: Iterable[str] | None = None,
    ) -> EpisodeSelection:
        if len(run_root) != 32 or episode_ordinal < 0 or vector_slot < 0:
            raise ReferenceTrackerError("invalid episode identity")
        reset_seed = derive_named_seed(
            run_root, split, episode_ordinal, vector_slot, "randomization.reset-mode"
        )
        reset_bucket = int.from_bytes(reset_seed[:8], "little") % 10_000
        weights = self.profile.document["reset"]["mode_weights_basis_points"]
        cumulative = 0
        reset_mode = ""
        for mode in RESET_MODES:
            cumulative += int(weights[mode])
            if reset_bucket < cumulative:
                reset_mode = mode
                break
        if not reset_mode or reset_mode == "near_failure":
            raise ReferenceTrackerError("disabled reset mode was selected")
        allowed_skills = None if skills is None else frozenset(skills)
        if reset_mode == "neutral_entry":
            allowed_skills = (
                NEUTRAL_ENTRY_SKILLS
                if allowed_skills is None
                else allowed_skills.intersection(NEUTRAL_ENTRY_SKILLS)
            )
        clip_ids = self.clip_ids(split, allowed_skills)
        if not clip_ids:
            raise ReferenceTrackerError("episode selection has no eligible reference clips")
        clip_seed = derive_named_seed(
            run_root, split, episode_ordinal, vector_slot, "randomization.reference-clip"
        )
        clip_id = clip_ids[int.from_bytes(clip_seed[:8], "little") % len(clip_ids)]
        clip = self.load_clip(clip_id)
        phase_seed = derive_named_seed(
            run_root, split, episode_ordinal, vector_slot, "randomization.reference-phase"
        )
        phase_frame = int.from_bytes(phase_seed[:8], "little") % clip.frame_count
        return EpisodeSelection(
            split=split,
            episode_ordinal=episode_ordinal,
            vector_slot=vector_slot,
            reset_mode=reset_mode,
            clip_id=clip_id,
            phase_frame=phase_frame,
            clip_seed_sha256=clip_seed.hex(),
            phase_seed_sha256=phase_seed.hex(),
            reset_seed_sha256=reset_seed.hex(),
        )


def derive_named_seed(
    run_root: bytes,
    split: str,
    episode_ordinal: int,
    vector_slot: int,
    purpose_id: str,
) -> bytes:
    if len(run_root) != 32 or split not in SPLITS or episode_ordinal < 0 or vector_slot < 0:
        raise ReferenceTrackerError("invalid seed derivation input")
    preimage = bytearray(b"nextengine.reference-tracker-rng.v1\0")
    preimage.extend(run_root)
    for text in (split, purpose_id):
        encoded = text.encode("utf-8")
        preimage.extend(len(encoded).to_bytes(4, "little"))
        preimage.extend(encoded)
    preimage.extend(episode_ordinal.to_bytes(8, "little"))
    preimage.extend(vector_slot.to_bytes(4, "little"))
    return hashlib.sha256(preimage).digest()


@dataclass(frozen=True)
class TrackingState:
    root_position_um: NDArray[np.int64]
    root_quaternion_q1_30: NDArray[np.int64]
    root_linear_velocity_um_s: NDArray[np.int64]
    root_angular_velocity_urad_s: NDArray[np.int64]
    joint_position_urad: NDArray[np.int64]
    joint_velocity_urad_s: NDArray[np.int64]
    previous_applied_target_urad: NDArray[np.int64]
    center_of_mass_um: NDArray[np.int64]
    effector_position_um: NDArray[np.int64]
    contact_flags: NDArray[np.uint8]
    sole_planar_velocity_um_s: NDArray[np.int64]
    applied_effort_unm: NDArray[np.int64]
    applied_target_urad: NDArray[np.int64]
    maximum_contact_impulse_basis_points: int = 0


def reference_state(clip: ReferenceClip, frame: int) -> TrackingState:
    if frame < 0 or frame >= clip.frame_count:
        raise ReferenceTrackerError("reference frame is out of range")
    joint = np.asarray(clip.arrays["joint_position_urad"][frame], dtype=np.int64)
    return TrackingState(
        root_position_um=np.asarray(clip.arrays["root_position_um"][frame], dtype=np.int64),
        root_quaternion_q1_30=np.asarray(
            clip.arrays["root_quaternion_q1_30"][frame], dtype=np.int64
        ),
        root_linear_velocity_um_s=np.asarray(
            clip.arrays["root_linear_velocity_um_s"][frame], dtype=np.int64
        ),
        root_angular_velocity_urad_s=np.asarray(
            [0, clip.arrays["root_yaw_velocity_urad_s"][frame], 0], dtype=np.int64
        ),
        joint_position_urad=joint,
        joint_velocity_urad_s=np.asarray(
            clip.arrays["joint_velocity_urad_s"][frame], dtype=np.int64
        ),
        previous_applied_target_urad=joint.copy(),
        center_of_mass_um=np.asarray(clip.arrays["center_of_mass_um"][frame], dtype=np.int64),
        effector_position_um=np.asarray(
            clip.arrays["effector_position_um"][frame], dtype=np.int64
        ),
        contact_flags=np.asarray(clip.arrays["contacts"][frame], dtype=np.uint8),
        sole_planar_velocity_um_s=np.zeros((2, 2), dtype=np.int64),
        applied_effort_unm=np.zeros(ACTION_CHANNELS, dtype=np.int64),
        applied_target_urad=joint.copy(),
    )


def build_observation(
    state: TrackingState,
    clip: ReferenceClip,
    phase_frame: int,
) -> NDArray[np.int64]:
    _validate_state(state)
    if phase_frame < 0 or phase_frame >= clip.frame_count:
        raise ReferenceTrackerError("reference phase is out of range")
    root_quaternion = _normalized_quaternion(state.root_quaternion_q1_30)
    values: list[NDArray[np.int64]] = [
        _quaternion_q1_30(root_quaternion),
        _rotate_inverse(state.root_linear_velocity_um_s, root_quaternion),
        _rotate_inverse(state.root_angular_velocity_urad_s, root_quaternion),
        state.joint_position_urad,
        state.joint_velocity_urad_s,
        state.previous_applied_target_urad,
        state.contact_flags.astype(np.int64),
        np.asarray([clip.arrays["phase_u16"][phase_frame]], dtype=np.int64),
    ]
    for offset in REFERENCE_OFFSETS:
        frame = min(phase_frame + offset, clip.frame_count - 1)
        reference_quaternion = _normalized_quaternion(clip.arrays["root_quaternion_q1_30"][frame])
        relative_rotation = _quaternion_multiply(
            _quaternion_conjugate(root_quaternion), reference_quaternion
        )
        values.extend(
            (
                _rotate_inverse(
                    clip.arrays["root_position_um"][frame] - state.root_position_um,
                    root_quaternion,
                ),
                _quaternion_q1_30(relative_rotation),
                _rotate_inverse(clip.arrays["root_linear_velocity_um_s"][frame], root_quaternion),
                _rotate_inverse(
                    np.asarray(
                        [0, clip.arrays["root_yaw_velocity_urad_s"][frame], 0],
                        dtype=np.int64,
                    ),
                    root_quaternion,
                ),
                _rotate_inverse(
                    clip.arrays["center_of_mass_um"][frame] - state.root_position_um,
                    root_quaternion,
                ),
                np.asarray(clip.arrays["joint_position_urad"][frame], dtype=np.int64),
                np.asarray(clip.arrays["joint_velocity_urad_s"][frame], dtype=np.int64),
                np.concatenate(
                    [
                        _rotate_inverse(effector - state.root_position_um, root_quaternion)
                        for effector in clip.arrays["effector_position_um"][frame]
                    ]
                ),
                np.asarray(clip.arrays["contacts"][frame], dtype=np.int64),
            )
        )
    observation = np.concatenate([np.ravel(value) for value in values]).astype(np.int64, copy=False)
    if observation.shape != (OBSERVATION_CHANNELS,):
        raise ReferenceTrackerError(
            f"reference observation width mismatch: expected {OBSERVATION_CHANNELS}, got {observation.size}"
        )
    return observation


def _validate_state(state: TrackingState) -> None:
    shapes = (
        (state.root_position_um, (3,)),
        (state.root_quaternion_q1_30, (4,)),
        (state.root_linear_velocity_um_s, (3,)),
        (state.root_angular_velocity_urad_s, (3,)),
        (state.joint_position_urad, (ACTION_CHANNELS,)),
        (state.joint_velocity_urad_s, (ACTION_CHANNELS,)),
        (state.previous_applied_target_urad, (ACTION_CHANNELS,)),
        (state.center_of_mass_um, (3,)),
        (state.effector_position_um, (6, 3)),
        (state.contact_flags, (7,)),
        (state.sole_planar_velocity_um_s, (2, 2)),
        (state.applied_effort_unm, (ACTION_CHANNELS,)),
        (state.applied_target_urad, (ACTION_CHANNELS,)),
    )
    if any(value.shape != shape for value, shape in shapes):
        raise ReferenceTrackerError("tracking state shape mismatch")
    if (
        not isinstance(state.maximum_contact_impulse_basis_points, int)
        or state.maximum_contact_impulse_basis_points < 0
    ):
        raise ReferenceTrackerError("invalid contact impact margin state")


def _normalized_quaternion(value: Sequence[int] | NDArray[Any]) -> NDArray[np.float64]:
    quaternion = np.asarray(value, dtype=np.float64) / float(1 << 30)
    norm = float(np.linalg.norm(quaternion))
    if not np.isfinite(norm) or norm <= 1.0e-12:
        raise ReferenceTrackerError("invalid root quaternion")
    quaternion = quaternion / norm
    if quaternion[3] < 0 or (
        quaternion[3] == 0 and next((item for item in quaternion[:3] if item != 0), 1.0) < 0
    ):
        quaternion = -quaternion
    return quaternion


def _quaternion_q1_30(quaternion: NDArray[np.float64]) -> NDArray[np.int64]:
    value = np.rint(_normalized_quaternion(np.rint(quaternion * float(1 << 30))) * float(1 << 30))
    return value.astype(np.int64)


def _quaternion_conjugate(quaternion: NDArray[np.float64]) -> NDArray[np.float64]:
    return np.asarray([-quaternion[0], -quaternion[1], -quaternion[2], quaternion[3]])


def _quaternion_multiply(
    left: NDArray[np.float64], right: NDArray[np.float64]
) -> NDArray[np.float64]:
    lx, ly, lz, lw = left
    rx, ry, rz, rw = right
    value = np.asarray(
        [
            lw * rx + lx * rw + ly * rz - lz * ry,
            lw * ry - lx * rz + ly * rw + lz * rx,
            lw * rz + lx * ry - ly * rx + lz * rw,
            lw * rw - lx * rx - ly * ry - lz * rz,
        ],
        dtype=np.float64,
    )
    return _normalized_quaternion(np.rint(value * float(1 << 30)))


def _rotate_inverse(
    vector: Sequence[int] | NDArray[Any], quaternion: NDArray[np.float64]
) -> NDArray[np.int64]:
    x, y, z, w = quaternion
    matrix = np.asarray(
        [
            [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
            [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
            [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
        ]
    )
    return np.rint(matrix.T @ np.asarray(vector, dtype=np.float64)).astype(np.int64)


@dataclass(frozen=True)
class RewardResult:
    component_values_q16: tuple[tuple[str, int], ...]
    total_q16: int


def compute_reward(
    profile: ReferenceTrackerProfile,
    limits: DescriptorLimits,
    state: TrackingState,
    clip: ReferenceClip,
    phase_frame: int,
    *,
    terminal_failure: bool,
) -> RewardResult:
    _validate_state(state)
    reference = reference_state(clip, phase_frame)
    reward = profile.document["reward"]
    components = reward["components"]
    by_id = {component["id"]: component for component in components}
    orientation_dot = abs(
        float(
            np.dot(
                _normalized_quaternion(state.root_quaternion_q1_30),
                _normalized_quaternion(reference.root_quaternion_q1_30),
            )
        )
    )
    values: dict[str, float] = {
        "reward.reference-root-orientation": min(1.0, orientation_dot) ** 2,
        "reward.reference-root-height": _linear_similarity(
            abs(int(state.root_position_um[1]) - int(reference.root_position_um[1])),
            by_id["reward.reference-root-height"]["normalization_micrometres"],
        ),
        "reward.reference-root-linear-velocity": _linear_similarity(
            _vector_norm(state.root_linear_velocity_um_s - reference.root_linear_velocity_um_s),
            by_id["reward.reference-root-linear-velocity"][
                "normalization_micrometres_per_second"
            ],
        ),
        "reward.reference-root-angular-velocity": _linear_similarity(
            _vector_norm(
                state.root_angular_velocity_urad_s - reference.root_angular_velocity_urad_s
            ),
            by_id["reward.reference-root-angular-velocity"][
                "normalization_microradians_per_second"
            ],
        ),
        "reward.reference-joint-pose": 1.0
        - min(
            1.0,
            float(
                np.mean(
                    np.abs(state.joint_position_urad - reference.joint_position_urad)
                    / limits.soft_rom_spans_urad
                )
            ),
        ),
        "reward.reference-joint-velocity": 1.0
        - min(
            1.0,
            float(
                np.mean(
                    np.abs(state.joint_velocity_urad_s - reference.joint_velocity_urad_s)
                    / limits.maximum_velocity_urad_s
                )
            ),
        ),
        "reward.reference-center-of-mass": _linear_similarity(
            _vector_norm(state.center_of_mass_um - reference.center_of_mass_um),
            by_id["reward.reference-center-of-mass"]["normalization_micrometres"],
        ),
        "reward.reference-effectors": _linear_similarity(
            float(
                np.mean(
                    np.linalg.norm(
                        (state.effector_position_um - reference.effector_position_um).astype(
                            np.float64
                        ),
                        axis=1,
                    )
                )
            ),
            by_id["reward.reference-effectors"]["normalization_micrometres"],
        ),
        "reward.reference-contacts": float(
            np.count_nonzero(state.contact_flags == reference.contact_flags)
        )
        / 7.0,
        "reward.contacting-sole-slip-cost": _sole_slip_cost(
            state,
            by_id["reward.contacting-sole-slip-cost"][
                "normalization_micrometres_per_second"
            ],
        ),
        "reward.normalized-applied-effort-cost": min(
            1.0,
            float(np.mean(np.abs(state.applied_effort_unm) / limits.maximum_effort_unm)),
        ),
        "reward.applied-target-rate-cost": min(
            1.0,
            float(
                np.mean(
                    np.abs(state.applied_target_urad - state.previous_applied_target_urad)
                    / limits.maximum_target_delta_urad
                )
            ),
        ),
        "reward.soft-rom-excursion-cost": _soft_rom_excursion_cost(state, limits),
        "reward.predictive-rom-excursion-cost": _predictive_rom_excursion_cost(
            state, limits
        ),
        "reward.contact-impact-margin-cost": _contact_impact_margin_cost(
            state.maximum_contact_impulse_basis_points, 9_500
        ),
        "reward.terminal-failure": 1.0 if terminal_failure else 0.0,
    }
    ordered: list[tuple[str, int]] = []
    total = 0
    for component in components:
        component_id = component["id"]
        value_q16 = int(round(min(1.0, max(0.0, values[component_id])) * 65_536.0))
        ordered.append((component_id, value_q16))
        total += int(round(int(component["coefficient_q16"]) * value_q16 / 65_536.0))
    return RewardResult(tuple(ordered), total)


def _vector_norm(value: NDArray[Any]) -> float:
    return float(np.linalg.norm(np.asarray(value, dtype=np.float64)))


def _linear_similarity(error: float, normalization: int | float) -> float:
    return 1.0 - min(1.0, max(0.0, error / float(normalization)))


def _contact_impact_margin_cost(
    maximum_impulse_basis_points: int, warning_basis_points: int
) -> float:
    if not 0 <= warning_basis_points < 10_000:
        raise ReferenceTrackerError("invalid contact impact warning boundary")
    return min(
        1.0,
        max(
            0.0,
            (maximum_impulse_basis_points - warning_basis_points)
            / float(10_000 - warning_basis_points),
        ),
    )


def _sole_slip_cost(state: TrackingState, normalization: int | float) -> float:
    active = np.flatnonzero(state.contact_flags[:2] != 0)
    if active.size == 0:
        return 0.0
    speed = np.linalg.norm(state.sole_planar_velocity_um_s[active].astype(np.float64), axis=1)
    return min(1.0, float(np.mean(speed)) / float(normalization))


def _soft_rom_excursion_cost(
    state: TrackingState, limits: DescriptorLimits
) -> float:
    position = state.joint_position_urad.astype(np.float64)
    lower_span = (limits.soft_minimum_urad - limits.hard_minimum_urad).astype(
        np.float64
    )
    upper_span = (limits.hard_maximum_urad - limits.soft_maximum_urad).astype(
        np.float64
    )
    lower = np.divide(
        limits.soft_minimum_urad - position,
        lower_span,
        out=np.zeros_like(position),
        where=lower_span > 0,
    )
    upper = np.divide(
        position - limits.soft_maximum_urad,
        upper_span,
        out=np.zeros_like(position),
        where=upper_span > 0,
    )
    return float(np.clip(np.maximum(lower, upper), 0.0, 1.0).max())


def _predictive_rom_excursion_cost(
    state: TrackingState, limits: DescriptorLimits
) -> float:
    projected = np.rint(
        state.joint_position_urad.astype(np.float64)
        + state.joint_velocity_urad_s.astype(np.float64) / 60.0
    ).astype(np.int64)
    projected_state = TrackingState(
        **{**state.__dict__, "joint_position_urad": projected}
    )
    return _soft_rom_excursion_cost(projected_state, limits)


def _reward_distribution(
    profile: ReferenceTrackerProfile,
    samples: Sequence[RewardResult],
) -> dict[str, Any]:
    components = profile.document["reward"]["components"]
    if not samples:
        raise ReferenceTrackerError("reward distribution has no samples")
    by_id = {
        component["id"]: {
            "value_q16": [],
            "weighted_contribution_q16": [],
        }
        for component in components
    }
    totals: list[int] = []
    for sample in samples:
        values = dict(sample.component_values_q16)
        if tuple(values) != tuple(component["id"] for component in components):
            raise ReferenceTrackerError("reward component order changed during audit")
        for component in components:
            component_id = component["id"]
            value_q16 = values[component_id]
            contribution_q16 = int(
                round(int(component["coefficient_q16"]) * value_q16 / 65_536.0)
            )
            by_id[component_id]["value_q16"].append(value_q16)
            by_id[component_id]["weighted_contribution_q16"].append(contribution_q16)
        totals.append(sample.total_q16)

    def summarize(values: Sequence[int]) -> dict[str, int]:
        ordered = sorted(values)
        final = len(ordered) - 1
        return {
            "minimum": ordered[0],
            "p05": ordered[(final * 5) // 100],
            "median": ordered[final // 2],
            "p95": ordered[(final * 95) // 100],
            "maximum": ordered[-1],
            "distinct_value_count": len(set(ordered)),
        }

    return {
        "sample_count": len(samples),
        "total_q16": summarize(totals),
        "components": {
            component_id: {
                "value_q16": summarize(values["value_q16"]),
                "weighted_contribution_q16": summarize(
                    values["weighted_contribution_q16"]
                ),
            }
            for component_id, values in by_id.items()
        },
    }


def audit_reference_inputs(
    *,
    profile_path: Path,
    descriptor_path: Path,
    corpus_root: Path,
    gate_report_path: Path,
    output_store: Path,
) -> tuple[dict[str, Any], Path]:
    profile = ReferenceTrackerProfile.load(profile_path)
    tool_sha256 = _sha256(Path(__file__).read_bytes())
    limits = DescriptorLimits.load(profile, descriptor_path)
    corpus = ReferenceCorpus(profile, corpus_root, gate_report_path)
    validated_artifact_count = corpus.validate_all_artifacts()
    split_counts = {split: len(corpus.clip_ids(split)) for split in SPLITS}
    if any(count == 0 for count in split_counts.values()):
        raise ReferenceTrackerError("reference corpus split is empty")
    observation_checks = 0
    reward_checks = 0
    phase_offset_checks = 0
    cost_probe_checks = 0
    terminal_penalty_checks = 0
    natural_reward_samples: list[RewardResult] = []
    directed_reward_samples: list[RewardResult] = []
    phase_offset_reduced_total = False
    reward_document = profile.document["reward"]
    coefficients = {
        component["id"]: int(component["coefficient_q16"])
        for component in reward_document["components"]
    }
    cost_component_ids = (
        "reward.contacting-sole-slip-cost",
        "reward.normalized-applied-effort-cost",
        "reward.applied-target-rate-cost",
        *(
            ("reward.soft-rom-excursion-cost",)
            if "reward.soft-rom-excursion-cost" in coefficients
            else ()
        ),
        *(
            ("reward.predictive-rom-excursion-cost",)
            if "reward.predictive-rom-excursion-cost" in coefficients
            else ()
        ),
        *(
            ("reward.contact-impact-margin-cost",)
            if "reward.contact-impact-margin-cost" in coefficients
            else ()
        ),
    )
    if any(coefficients[component_id] >= 0 for component_id in cost_component_ids) or (
        coefficients["reward.terminal-failure"] >= 0
    ):
        raise ReferenceTrackerError("reward cost or terminal penalty has the wrong sign")
    positive_weight_q16 = sum(value for value in coefficients.values() if value > 0)
    maximum_positive_share_basis_points = max(
        value for value in coefficients.values() if value > 0
    ) * 10_000 // positive_weight_q16
    if maximum_positive_share_basis_points >= 5_000:
        raise ReferenceTrackerError("one non-terminal reward component dominates positive scale")
    for split in SPLITS:
        for clip_id in corpus.clip_ids(split):
            clip = corpus.load_clip(clip_id)
            for frame in sorted({0, clip.frame_count // 2, clip.frame_count - 1}):
                state = reference_state(clip, frame)
                observation = build_observation(state, clip, frame)
                if observation.shape != (OBSERVATION_CHANNELS,) or not np.all(
                    np.isfinite(observation)
                ):
                    raise ReferenceTrackerError("reference observation sanity failed")
                observation_checks += 1
                reward = compute_reward(
                    profile,
                    limits,
                    state,
                    clip,
                    frame,
                    terminal_failure=False,
                )
                reward_values = dict(reward.component_values_q16)
                expected_perfect_total = 458_752 + sum(
                    int(
                        round(
                            coefficients[component_id]
                            * reward_values[component_id]
                            / 65_536.0
                        )
                    )
                    for component_id in (
                        "reward.soft-rom-excursion-cost",
                        "reward.predictive-rom-excursion-cost",
                    )
                    if component_id in coefficients
                )
                if reward.total_q16 != expected_perfect_total:
                    raise ReferenceTrackerError("perfect-reference reward sanity failed")
                reward_checks += 1
                natural_reward_samples.append(reward)

                offset_frame = min(frame + 8, clip.frame_count - 1)
                if offset_frame == frame:
                    offset_frame = max(0, frame - 8)
                if offset_frame == frame:
                    raise ReferenceTrackerError("reference clip is too short for phase audit")
                phase_offset = compute_reward(
                    profile,
                    limits,
                    reference_state(clip, offset_frame),
                    clip,
                    frame,
                    terminal_failure=False,
                )
                if phase_offset.total_q16 > reward.total_q16:
                    raise ReferenceTrackerError("phase-offset reward exceeds perfect reward")
                phase_offset_reduced_total |= phase_offset.total_q16 < reward.total_q16
                phase_offset_checks += 1
                natural_reward_samples.append(phase_offset)

                contacts = state.contact_flags.copy()
                contacts[:2] = 1
                sole_velocity = np.zeros((2, 2), dtype=np.int64)
                sole_velocity[:, 0] = int(
                    next(
                        component["normalization_micrometres_per_second"]
                        for component in reward_document["components"]
                        if component["id"] == "reward.contacting-sole-slip-cost"
                    )
                )
                joint_position = state.joint_position_urad.copy()
                joint_velocity = state.joint_velocity_urad_s.copy()
                if (
                    "reward.soft-rom-excursion-cost" in coefficients
                    or "reward.predictive-rom-excursion-cost" in coefficients
                ):
                    upper_candidates = np.flatnonzero(
                        limits.hard_maximum_urad > limits.soft_maximum_urad
                    )
                    lower_candidates = np.flatnonzero(
                        limits.soft_minimum_urad > limits.hard_minimum_urad
                    )
                    if upper_candidates.size:
                        probe_dof = int(upper_candidates[0])
                        joint_position[probe_dof] = limits.hard_maximum_urad[
                            probe_dof
                        ]
                        joint_velocity[probe_dof] = 0
                    elif lower_candidates.size:
                        probe_dof = int(lower_candidates[0])
                        joint_position[probe_dof] = limits.hard_minimum_urad[
                            probe_dof
                        ]
                        joint_velocity[probe_dof] = 0
                    else:
                        raise ReferenceTrackerError(
                            "soft-ROM cost has no descriptor warning interval"
                        )
                cost_state = TrackingState(
                    **{
                        **state.__dict__,
                        "joint_position_urad": joint_position,
                        "joint_velocity_urad_s": joint_velocity,
                        "contact_flags": contacts,
                        "sole_planar_velocity_um_s": sole_velocity,
                        "applied_effort_unm": limits.maximum_effort_unm.copy(),
                        "applied_target_urad": state.previous_applied_target_urad
                        + limits.maximum_target_delta_urad,
                        "maximum_contact_impulse_basis_points": 10_000,
                    }
                )
                cost_probe = compute_reward(
                    profile,
                    limits,
                    cost_state,
                    clip,
                    frame,
                    terminal_failure=False,
                )
                cost_values = dict(cost_probe.component_values_q16)
                if any(cost_values[component_id] != 65_536 for component_id in cost_component_ids):
                    raise ReferenceTrackerError("directed reward cost probe did not reach unit scale")
                if cost_probe.total_q16 >= reward.total_q16:
                    raise ReferenceTrackerError("reward cost probe has the wrong total sign")
                cost_probe_checks += 1
                directed_reward_samples.append(cost_probe)

                terminal_probe = compute_reward(
                    profile,
                    limits,
                    state,
                    clip,
                    frame,
                    terminal_failure=True,
                )
                terminal_values = dict(terminal_probe.component_values_q16)
                if (
                    terminal_values["reward.terminal-failure"] != 65_536
                    or terminal_probe.total_q16
                    != reward.total_q16 + coefficients["reward.terminal-failure"]
                ):
                    raise ReferenceTrackerError("terminal reward penalty scale is incorrect")
                terminal_penalty_checks += 1
                directed_reward_samples.append(terminal_probe)
    if not phase_offset_reduced_total:
        raise ReferenceTrackerError("phase-offset reward is constant across the corpus audit")
    natural_distribution = _reward_distribution(profile, natural_reward_samples)
    directed_distribution = _reward_distribution(profile, directed_reward_samples)
    tracking_component_ids = tuple(coefficients)[:9]
    constant_tracking_components = [
        component_id
        for component_id in tracking_component_ids
        if natural_distribution["components"][component_id]["value_q16"][
            "distinct_value_count"
        ]
        == 1
    ]
    if constant_tracking_components:
        raise ReferenceTrackerError(
            "constant tracking reward components: " + ", ".join(constant_tracking_components)
        )
    selections: list[dict[str, Any]] = []
    run_root = hashlib.sha256(b"nextengine.train-5.reference-input-audit.v1").digest()
    for split in SPLITS:
        for episode_ordinal in range(8):
            selection = corpus.select_episode(
                run_root=run_root,
                split=split,
                episode_ordinal=episode_ordinal,
                vector_slot=episode_ordinal % 2,
            )
            repeated = corpus.select_episode(
                run_root=run_root,
                split=split,
                episode_ordinal=episode_ordinal,
                vector_slot=episode_ordinal % 2,
            )
            if selection != repeated or selection.clip_id not in corpus.clip_ids(split):
                raise ReferenceTrackerError("reference episode selection is not deterministic")
            selections.append(
                {
                    "split": selection.split,
                    "episode_ordinal": selection.episode_ordinal,
                    "vector_slot": selection.vector_slot,
                    "reset_mode": selection.reset_mode,
                    "clip_id": selection.clip_id,
                    "phase_frame": selection.phase_frame,
                    "clip_seed_sha256": selection.clip_seed_sha256,
                    "phase_seed_sha256": selection.phase_seed_sha256,
                    "reset_seed_sha256": selection.reset_seed_sha256,
                }
            )
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": "MOTOR-REFERENCE-ENV-P1-INPUT",
        "status": "PASS",
        "claim": "InputClosureAndPrePhysicsSanityOnly",
        "profile_sha256": profile.document_sha256,
        "tool_sha256": tool_sha256,
        "corpus_manifest_sha256": profile.document["corpus"]["manifest_sha256"],
        "corpus_manifest_file_sha256": corpus.manifest_file_sha256,
        "gate_report_sha256": profile.document["training_authorization"][
            "gate_report_sha256"
        ],
        "descriptor_file_sha256": limits.document_sha256,
        "input_provenance_root": corpus.input_provenance_root,
        "contract_projection": profile.contract_projection(),
        "split_clip_counts": split_counts,
        "validated_artifact_count": validated_artifact_count,
        "observation_fixture_count": observation_checks,
        "perfect_reward_fixture_count": reward_checks,
        "phase_offset_reward_fixture_count": phase_offset_checks,
        "cost_probe_fixture_count": cost_probe_checks,
        "terminal_penalty_fixture_count": terminal_penalty_checks,
        "reward_scale_assessment": {
            "status": "PASS",
            "arithmetic": reward_document["arithmetic"],
            "maximum_positive_component_share_basis_points": maximum_positive_share_basis_points,
            "dominance_rejection_threshold_basis_points": 5_000,
            "natural_perfect_and_phase_offset": natural_distribution,
            "directed_cost_and_terminal": directed_distribution,
            "constant_tracking_components": constant_tracking_components,
        },
        "deterministic_selection_fixtures": selections,
        "physics_episodes": 0,
        "optimizer_steps": 0,
        "learned_policy_claim": False,
    }
    payload = _canonical_json(report)
    destination = (
        output_store.resolve()
        / "evaluations"
        / "TRAIN-5"
        / f"reference-input-audit-{profile.document_sha256[:16]}-{tool_sha256[:16]}.json"
    )
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists() and destination.read_bytes() != payload:
        raise ReferenceTrackerError(f"refusing to overwrite a different audit: {destination}")
    if not destination.exists():
        destination.write_bytes(payload)
    return report, destination


def run_physx_baseline(
    *,
    runner: Path,
    profile_path: Path,
    corpus_root: Path,
    gate_report_path: Path,
    output_store: Path,
    split: str,
    clip_id: str,
    start_frame: int,
    baseline: str,
) -> tuple[dict[str, Any], Path]:
    if baseline not in ("zero_residual", "random_residual"):
        raise ReferenceTrackerError(f"unsupported reference baseline: {baseline}")
    profile = ReferenceTrackerProfile.load(profile_path)
    corpus = ReferenceCorpus(profile, corpus_root, gate_report_path)
    if clip_id not in corpus.clip_ids(split):
        raise ReferenceTrackerError("baseline clip does not belong to the declared split")
    clip = corpus.load_clip(clip_id)
    if start_frame < 0 or start_frame >= clip.frame_count - 1:
        raise ReferenceTrackerError("baseline start frame is out of range")
    runner_payload = runner.read_bytes()
    runner_sha256 = _sha256(runner_payload)
    adapter_tool_sha256 = _sha256(Path(__file__).read_bytes())
    document = {
        "schema_version": 1,
        "schema_id": "nextengine.motor.reference-baseline-input.v1",
        "profile_id": profile.document["profile_id"],
        "profile_sha256": profile.document_sha256,
        "corpus_manifest_sha256": profile.document["corpus"]["manifest_sha256"],
        "input_provenance_root": corpus.input_provenance_root,
        "baseline": baseline,
        "clip_id": clip.clip_id,
        "skill": clip.skill,
        "split": split,
        "start_frame": start_frame,
        "root_position_um": clip.arrays["root_position_um"].tolist(),
        "root_quaternion_q1_30": clip.arrays["root_quaternion_q1_30"].tolist(),
        "root_linear_velocity_um_s": clip.arrays["root_linear_velocity_um_s"].tolist(),
        "root_yaw_velocity_urad_s": clip.arrays["root_yaw_velocity_urad_s"].tolist(),
        "joint_position_urad": clip.arrays["joint_position_urad"].tolist(),
        "joint_velocity_urad_s": clip.arrays["joint_velocity_urad_s"].tolist(),
    }
    input_payload = _canonical_json(document)
    result = subprocess.run(
        [str(runner.resolve())],
        input=input_payload,
        capture_output=True,
        timeout=300,
        check=False,
    )
    if result.returncode != 0:
        diagnostic = result.stderr.decode("utf-8", errors="replace").strip()
        raise ReferenceTrackerError(f"canonical PhysX baseline failed: {diagnostic}")
    report = json.loads(result.stdout)
    if (
        report.get("check") != "MOTOR-REFERENCE-ENV-P1-PHYSX-BASELINE"
        or report.get("execution_status") != "PASS"
        or report.get("claim") != "CanonicalPhysXBaselineOnly"
        or report.get("profile_document_sha256") != profile.document_sha256
        or report.get("input_provenance_root") != corpus.input_provenance_root
        or report.get("baseline") != baseline
        or report.get("clip_id") != clip_id
        or report.get("split") != split
    ):
        raise ReferenceTrackerError("canonical PhysX baseline output closure mismatch")
    report["input_sha256"] = _sha256(input_payload)
    report["runner_sha256"] = runner_sha256
    report["adapter_tool_sha256"] = adapter_tool_sha256
    report["reference_artifact_sha256"] = corpus._entries[clip_id]["artifact"]["sha256"]
    output_payload = _canonical_json(report)
    destination = (
        output_store.resolve()
        / "evaluations"
        / "TRAIN-5"
        / (
            f"physx-baseline-{clip_id}-{start_frame}-{baseline}-"
            f"{runner_sha256[:16]}-{adapter_tool_sha256[:16]}.json"
        )
    )
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists() and destination.read_bytes() != output_payload:
        raise ReferenceTrackerError(f"refusing to overwrite a different baseline: {destination}")
    if not destination.exists():
        destination.write_bytes(output_payload)
    return report, destination
