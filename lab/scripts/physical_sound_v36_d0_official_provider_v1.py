#!/usr/bin/env python3
"""Verify the V36 E0 seal and expose the one-shot official D0 provider."""

from __future__ import annotations

import copy
import json
import math
from collections.abc import Callable
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, cast

import numpy as np
import physical_sound_v33_i0_mode_local_spectral_owner_v1 as mode_local
import physical_sound_v36_e0_full_surrogate_seal_v1 as e0
import physical_sound_v36_f0_fresh_role_science_freeze_v1 as f0
import physical_sound_v36_owner_contract_v1 as contract

SEAL_PATH = "lab/profiles/physical-sound-v36-e0-execution-seal.v1.json"
SEAL_DOCUMENT_SHA256 = (
    "823cb21ca86d882c12392da6723773797383d054db45520248d31fbae711047c"
)
SEAL_PAYLOAD_SHA256 = "b436869654d833f97774b30683bf92672c62529e92a8c5957ee600a91d3ee516"
E0_RESULT_PATH = (
    "docs/development/physical-sound-v36-e0-full-surrogate-seal-result-2026-09-03.md"
)
E0_RESULT_SHA256 = "aff868b1faad22618b432b3ed64eba9e18d4e1510405917a65740079de492b05"
OFFICIAL_D0_NAMESPACE = "v36-fresh-role-unchanged-science-v1-d0"
SEAL_SCHEMA = "nextengine.experimental-physical-sound-v36-execution-seal.v1"
SEAL_STATUS = "SealedBeforeOfficialAccess"

EXPECTED_ORACLE: dict[str, Any] = {
    "contact": {
        "bound": "0.22",
        "components": {
            "curvature_u": "phi_u_plus+phi_u_minus-2*p1_contact",
            "difference_u": "phi_u_plus-phi_u_minus",
            "difference_v": "phi_v_plus-phi_v_minus",
            "surface_a": "sin(pi*u)*cos(2*pi*v)",
            "surface_b": "cos(3*pi*u)*sin(pi*v)",
        },
        "expression": (
            "bound*tanh(0.20*p1_contact+0.15*difference_u-0.13*difference_v+"
            "0.17*surface_a+0.14*surface_b+0.14*geometry0*p1_contact+"
            "0.11*geometry1*(2*u-1)+0.09*geometry2*(2*v-1)+"
            "0.085*ordinal*support+0.065*log_frequency*curvature_u)"
        ),
    },
    "decay": {
        "bound": "0.20",
        "expression": (
            "bound*tanh(0.37*loss+0.32*log_frequency+0.15*support*ordinal+"
            "0.16*log_youngs*log_density+0.08*poisson*family_index_a)"
        ),
    },
    "global_gain": {
        "bound": "0.16",
        "expression": (
            "bound*tanh(0.29*ordinal*geometry0+0.29*geometry2+"
            "0.26*log_youngs*ordinal-0.14*log_density+"
            "0.20*family*geometry1+0.13*geometry0*geometry2)"
        ),
    },
    "transcendentals": "python-math-binary64",
}

PARTITION_CODES = {
    (
        "kirchhoff-love-simply-supported-v1",
        "simply-supported-all-edges",
    ): (-1.0, -1.0, (1.0, 0.0), (1.0, 0.0, 0.0)),
    (
        "euler-bernoulli-cantilever-v1",
        "cantilever-clamped-u0",
    ): (1.0, 0.0, (0.0, 1.0), (0.0, 1.0, 0.0)),
    (
        "euler-bernoulli-simply-supported-v1",
        "simply-supported-both-ends",
    ): (1.0, 1.0, (0.0, 1.0), (0.0, 0.0, 1.0)),
}


class OfficialProviderError(RuntimeError):
    """The checked-in seal or official D0 provider boundary is invalid."""


@dataclass(frozen=True, slots=True)
class VerifiedD0PreAccess:
    seal: contract.ExecutionSeal
    context: e0.OwnerContext
    truth_contract: bytes
    receipt: dict[str, object]


class _StructuralRoleBuilder:
    """Reuse the sealed E0 container builder without surrogate target values."""

    def __init__(self, context: e0.OwnerContext) -> None:
        self._context = context

    def _analytic_train_targets(self, features: dict[str, np.ndarray]) -> np.ndarray:
        row_count = features["decay"].shape[0]
        return np.zeros((row_count, contract.TARGET_AXIS_COUNT), dtype=np.float64)

    def build_role(self, role: contract.RoleKind) -> contract.RoleBatch:
        build = cast(
            Callable[[object, contract.RoleKind], contract.RoleBatch],
            e0.SurrogateProvider._build_role,
        )
        return build(self, role)


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _bound_file(path_text: str, expected_sha256: str) -> bytes:
    path = repository_root() / path_text
    if not path.is_file() or path.is_symlink():
        raise OfficialProviderError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    if e0.sha256_bytes(data) != expected_sha256:
        raise OfficialProviderError(f"bound file drift: {path_text}")
    return data


def _string(value: object, label: str) -> str:
    if not isinstance(value, str):
        raise OfficialProviderError(f"{label} must be a string")
    return value


def _integer(value: object, label: str) -> int:
    if type(value) is not int:
        raise OfficialProviderError(f"{label} must be an integer")
    return value


def _boolean(value: object, label: str) -> bool:
    if type(value) is not bool:
        raise OfficialProviderError(f"{label} must be a boolean")
    return value


def parse_execution_seal(
    data: bytes, expected_document_sha256: str = SEAL_DOCUMENT_SHA256
) -> contract.ExecutionSeal:
    if e0.sha256_bytes(data) != expected_document_sha256:
        raise OfficialProviderError("execution seal document identity drift")
    document = e0.load_json_bytes(data, "execution seal")
    if data != e0.canonical_json(document):
        raise OfficialProviderError("execution seal document is noncanonical")
    expected_keys = {
        "claim",
        "execution_seal",
        "rehearsal_tree_sha256",
        "schema",
        "seal_sha256",
        "status",
    }
    if set(document) != expected_keys:
        raise OfficialProviderError("execution seal document closure drift")
    claimed_payload = _string(document["seal_sha256"], "seal_sha256")
    unsigned = copy.deepcopy(document)
    del unsigned["seal_sha256"]
    if (
        claimed_payload != SEAL_PAYLOAD_SHA256
        or e0.sha256_bytes(e0.canonical_json(unsigned)) != claimed_payload
    ):
        raise OfficialProviderError("execution seal payload identity drift")
    if (
        document["claim"] != e0.CLAIM
        or document["schema"] != SEAL_SCHEMA
        or document["status"] != SEAL_STATUS
    ):
        raise OfficialProviderError("execution seal authority drift")
    raw = document["execution_seal"]
    if not isinstance(raw, dict):
        raise OfficialProviderError("execution_seal must be an object")
    expected_seal_keys = {
        "contract_schema",
        "d0_rehearsal_root_sha256",
        "d0_topology_sha256",
        "environment_sha256",
        "forbidden_access_count",
        "h0_rehearsal_root_sha256",
        "h0_topology_sha256",
        "owner_sha256",
        "profile_sha256",
        "rehearsal_run_count",
        "repeat_exact",
    }
    if set(raw) != expected_seal_keys:
        raise OfficialProviderError("execution seal field closure drift")
    try:
        seal = contract.ExecutionSeal(
            contract_schema=_string(raw["contract_schema"], "contract_schema"),
            owner_sha256=_string(raw["owner_sha256"], "owner_sha256"),
            profile_sha256=_string(raw["profile_sha256"], "profile_sha256"),
            environment_sha256=_string(raw["environment_sha256"], "environment_sha256"),
            d0_rehearsal_root_sha256=_string(
                raw["d0_rehearsal_root_sha256"], "d0_rehearsal_root_sha256"
            ),
            h0_rehearsal_root_sha256=_string(
                raw["h0_rehearsal_root_sha256"], "h0_rehearsal_root_sha256"
            ),
            d0_topology_sha256=_string(raw["d0_topology_sha256"], "d0_topology_sha256"),
            h0_topology_sha256=_string(raw["h0_topology_sha256"], "h0_topology_sha256"),
            rehearsal_run_count=_integer(
                raw["rehearsal_run_count"], "rehearsal_run_count"
            ),
            repeat_exact=_boolean(raw["repeat_exact"], "repeat_exact"),
            forbidden_access_count=_integer(
                raw["forbidden_access_count"], "forbidden_access_count"
            ),
        )
    except contract.ContractError as error:
        raise OfficialProviderError(str(error)) from error
    if not isinstance(document["rehearsal_tree_sha256"], str):
        raise OfficialProviderError("rehearsal tree identity must be a string")
    return seal


def load_truth_contract() -> bytes:
    profile, _ = f0.load_profile(repository_root() / f0.PROFILE_PATH)
    v35_profile = f0.load_dependency_json(profile, "v35_f0_profile")
    _, effective = f0.build_experiments(profile, v35_profile)
    oracle = effective.get("oracle")
    if oracle != EXPECTED_ORACLE:
        raise OfficialProviderError("fresh V36 truth contract drift")
    result: bytes = e0.canonical_json(oracle)
    return result


def verify_execution_seal() -> VerifiedD0PreAccess:
    seal_data = _bound_file(SEAL_PATH, SEAL_DOCUMENT_SHA256)
    seal = parse_execution_seal(seal_data)
    _bound_file(E0_RESULT_PATH, E0_RESULT_SHA256)
    owner = e0.owner_identity()
    if owner["sha256"] != seal.owner_sha256:
        raise OfficialProviderError("sealed numeric owner drift")
    context = e0.load_context(repository_root() / e0.PROFILE_PATH)
    if e0.sha256_bytes(context.profile_data) != seal.profile_sha256:
        raise OfficialProviderError("sealed numeric profile drift")
    mode_local.configure_torch()
    environment = e0.environment_record(context.profile)
    environment_sha256 = e0.sha256_bytes(e0.canonical_json(environment))
    if environment_sha256 != seal.environment_sha256:
        raise OfficialProviderError("sealed execution environment drift")
    truth_contract = load_truth_contract()
    receipt: dict[str, object] = {
        "contract_schema": seal.contract_schema,
        "environment_sha256": environment_sha256,
        "fresh_v36_target_rows": 0,
        "official_capabilities_issued": 0,
        "official_d0_target_rows": 0,
        "owner_sha256": seal.owner_sha256,
        "profile_sha256": seal.profile_sha256,
        "seal_document_sha256": SEAL_DOCUMENT_SHA256,
        "seal_payload_sha256": SEAL_PAYLOAD_SHA256,
        "status": "VerifiedBeforeOfficialCapability",
        "truth_contract_sha256": e0.sha256_bytes(truth_contract),
    }
    return VerifiedD0PreAccess(seal, context, truth_contract, receipt)


def _partition(
    value: str,
) -> tuple[float, float, tuple[float, float], tuple[float, float, float], int]:
    try:
        parsed = json.loads(value)
    except json.JSONDecodeError as error:
        raise OfficialProviderError("local partition identity is invalid") from error
    if (
        not isinstance(parsed, list)
        or len(parsed) != 3
        or not isinstance(parsed[0], str)
        or not isinstance(parsed[1], str)
        or type(parsed[2]) is not int
    ):
        raise OfficialProviderError("local partition identity is invalid")
    codes = PARTITION_CODES.get((parsed[0], parsed[1]))
    if codes is None:
        raise OfficialProviderError("local partition is outside the frozen families")
    family, support, family_features, support_features = codes
    return family, support, family_features, support_features, parsed[2]


def official_targets(
    batch: contract.RoleBatch, truth_contract: bytes
) -> contract.FloatMatrix:
    truth = e0.load_json_bytes(truth_contract, "fresh V36 truth contract")
    if truth != EXPECTED_ORACLE:
        raise OfficialProviderError("fresh V36 truth contract drift")
    targets = np.empty((batch.row_count, contract.TARGET_AXIS_COUNT), dtype=np.float64)
    decay_features = batch.features.decay
    gain_features = batch.features.global_gain
    contact_features = batch.features.contact
    local_keys = batch.local_keys
    for index in range(batch.row_count):
        local = local_keys[index]
        decay_row = decay_features[index]
        gain_row = gain_features[index]
        contact_row = contact_features[index]
        family, support, family_features, support_features, ordinal_index = _partition(
            batch.local_partition_ids[index]
        )
        ordinal = 2.0 * float(ordinal_index) / 9.0 - 1.0
        expected_row_suffix = f":mode-{ordinal_index:02d}"
        if (
            not batch.row_ids[index].endswith(expected_row_suffix)
            or ordinal_index < 0
            or ordinal_index >= 10
            or decay_row[0] != ordinal
            or decay_row[1] != local[9]
            or tuple(decay_row[4:8]) != tuple(local[0:4])
            or tuple(gain_row[8:10]) != family_features
            or tuple(gain_row[10:13]) != tuple(local[4:7])
            or tuple(contact_row[6:9]) != support_features
            or contact_row[9] != local[7]
            or contact_row[10] != local[8]
            or contact_row[11] != local[10]
            or tuple(contact_row[32:35]) != tuple(local[4:7])
        ):
            raise OfficialProviderError("role feature/truth coordinate drift")

        log_youngs, log_density, poisson, loss = map(float, local[0:4])
        geometry0, geometry1, geometry2 = map(float, local[4:7])
        u = (float(local[7]) + 1.0) * 0.5
        v = (float(local[8]) + 1.0) * 0.5
        log_frequency = float(local[9])
        p1_contact = float(local[10])
        phi_u_minus, phi_u_plus, phi_v_minus, phi_v_plus = map(float, local[11:15])
        difference_u = phi_u_plus - phi_u_minus
        difference_v = phi_v_plus - phi_v_minus
        curvature_u = phi_u_plus + phi_u_minus - 2.0 * p1_contact
        surface_a = math.sin(math.pi * u) * math.cos(2.0 * math.pi * v)
        surface_b = math.cos(3.0 * math.pi * u) * math.sin(math.pi * v)
        family_index_a = float(decay_row[2])

        targets[index, 0] = 0.20 * math.tanh(
            0.37 * loss
            + 0.32 * log_frequency
            + 0.15 * support * ordinal
            + 0.16 * log_youngs * log_density
            + 0.08 * poisson * family_index_a
        )
        targets[index, 1] = 0.16 * math.tanh(
            0.29 * ordinal * geometry0
            + 0.29 * geometry2
            + 0.26 * log_youngs * ordinal
            - 0.14 * log_density
            + 0.20 * family * geometry1
            + 0.13 * geometry0 * geometry2
        )
        targets[index, 2] = 0.22 * math.tanh(
            0.20 * p1_contact
            + 0.15 * difference_u
            - 0.13 * difference_v
            + 0.17 * surface_a
            + 0.14 * surface_b
            + 0.14 * geometry0 * p1_contact
            + 0.11 * geometry1 * (2.0 * u - 1.0)
            + 0.09 * geometry2 * (2.0 * v - 1.0)
            + 0.085 * ordinal * support
            + 0.065 * log_frequency * curvature_u
        )
    bounds: np.ndarray = np.asarray([0.20, 0.16, 0.22], dtype=np.float64)
    if np.any(~np.isfinite(targets)) or np.any(np.abs(targets) > bounds[None, :]):
        raise OfficialProviderError("fresh V36 target contract failed")
    return e0.immutable_matrix(targets)


class OfficialD0Provider:
    """Materialize only the fresh train/development transaction under one seal."""

    def __init__(self, verified: VerifiedD0PreAccess) -> None:
        self._verified = verified
        self._builder = _StructuralRoleBuilder(verified.context)
        self._opened: set[contract.RoleKind] = set()

    @property
    def kind(self) -> contract.ProviderKind:
        return contract.ProviderKind.OFFICIAL_D0

    @property
    def namespace(self) -> str:
        return OFFICIAL_D0_NAMESPACE

    def materialize(
        self, role: contract.RoleKind, capability: contract.AccessCapability
    ) -> contract.RoleBatch:
        if (
            capability.provider_kind is not self.kind
            or capability.namespace != self.namespace
            or capability.execution_seal != self._verified.seal
        ):
            raise contract.ContractError("official D0 capability identity mismatch")
        if role not in capability.allowed_roles or role in self._opened:
            raise contract.ContractError(
                "official D0 role is forbidden or already open"
            )
        if (
            role is contract.RoleKind.DEVELOPMENT
            and contract.RoleKind.TRAIN not in self._opened
        ):
            raise contract.ContractError("official D0 development opened before train")
        structural = self._builder.build_role(role)
        result = replace(
            structural,
            targets=official_targets(structural, self._verified.truth_contract),
        )
        self._opened.add(role)
        return result


def prepare_official_d0() -> tuple[
    VerifiedD0PreAccess, contract.AccessCapability, OfficialD0Provider
]:
    verified = verify_execution_seal()
    capability = contract.AccessCapability.official(
        contract.ProviderKind.OFFICIAL_D0,
        OFFICIAL_D0_NAMESPACE,
        verified.seal,
    )
    receipt = {**verified.receipt, "official_capabilities_issued": 1}
    prepared = replace(verified, receipt=receipt)
    return prepared, capability, OfficialD0Provider(prepared)
