#!/usr/bin/env python3
"""Verify E0/T0/D0R seals and expose the one-shot official V37 D0 provider."""

from __future__ import annotations

from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, cast

import physical_sound_v37_c0_query_surface_structural_cost_v1 as c0
import physical_sound_v37_d0r_complete_entry_readiness_v1 as d0r
import physical_sound_v37_query_surface_contract_v1 as contract
import physical_sound_v37_t0_exact_truth_protocol_v1 as t0

D0R_PROFILE_PATH = (
    "lab/profiles/physical-sound-v37-d0r-complete-entry-readiness.v1.json"
)
D0R_PROFILE_SHA256 = "75baf8cc56a60a6d623e36e19ac97e873663eef5e48f272b70e048ecfae565f3"
D0R_OWNER_SHA256 = "ebbdb386b8fb6cf2ea222ffd0b091b60a075a54a99ad851065605ef84165c50c"
D0R_SEAL_PATH = "lab/profiles/physical-sound-v37-d0r-readiness-seal.v1.json"
D0R_SEAL_DOCUMENT_SHA256 = (
    "32ffa8b9c20f9fe66f827421f22cd3c4a0778865628827ef2cc84a21c6aa348e"
)
D0R_SEAL_PAYLOAD_SHA256 = (
    "a2c3c6ce797f5511c3b3e3062ae23534b3414de4f6f529ab0991c142869b18c5"
)
T0_PROFILE_PATH = "lab/profiles/physical-sound-v37-t0-exact-truth-protocol.v1.json"
T0_PROFILE_SHA256 = "8eeda45716f95ec0b74abcc2ce90c1896657eb460fd3fd7a28fd8cf53257e5b6"
T0_SEAL_PATH = "lab/profiles/physical-sound-v37-t0-official-input-seal.v1.json"
T0_SEAL_DOCUMENT_SHA256 = (
    "9a4ed1c9193e793288c8db93a8b8cf46280b75104821bc04a3b0cf0317696f52"
)
T0_SEAL_PAYLOAD_SHA256 = (
    "7bae65cc0290c3ec54eedef5d9ef2dd3d8e4ab1881945db71c8c80dcd1361573"
)
E0_SEAL_PAYLOAD_SHA256 = (
    "608f901938c4031e63353a9231e005bb1040ac21c50ed3c7746e01525e211d7f"
)
OFFICIAL_D0_NAMESPACE = "v37-qso-v0-fresh-development-v1-d0"


class OfficialProviderError(RuntimeError):
    """The V37 official D0 pre-access closure or provider is invalid."""


@dataclass(frozen=True, slots=True)
class VerifiedD0PreAccess:
    d0r_context: d0r.LoadedContext
    t0_context: t0.LoadedContext
    d0r_seal: dict[str, Any]
    t0_seal: dict[str, Any]
    receipt: dict[str, object]


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def bound_file(path_text: str, expected_sha256: str) -> bytes:
    path = repository_root() / path_text
    if not path.is_file() or path.is_symlink():
        raise OfficialProviderError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    if d0r.sha256_bytes(data) != expected_sha256:
        raise OfficialProviderError(f"bound file drift: {path_text}")
    return data


def verify_pre_access() -> VerifiedD0PreAccess:
    bound_file(D0R_PROFILE_PATH, D0R_PROFILE_SHA256)
    bound_file(T0_PROFILE_PATH, T0_PROFILE_SHA256)
    bound_file(D0R_SEAL_PATH, D0R_SEAL_DOCUMENT_SHA256)
    bound_file(T0_SEAL_PATH, T0_SEAL_DOCUMENT_SHA256)
    if d0r.owner_identity()["sha256"] != D0R_OWNER_SHA256:
        raise OfficialProviderError("D0R target-aware owner identity drift")
    d0r_context = d0r.load_context(repository_root() / D0R_PROFILE_PATH)
    d0r_seal = d0r.validate_checked_readiness_seal(repository_root() / D0R_SEAL_PATH)
    readiness = cast(dict[str, Any], d0r_seal["readiness_seal"])
    if (
        d0r_seal.get("seal_sha256") != D0R_SEAL_PAYLOAD_SHA256
        or readiness.get("owner_sha256") != D0R_OWNER_SHA256
        or readiness.get("profile_sha256") != D0R_PROFILE_SHA256
        or readiness.get("topology_sha256")
        != contract.expected_topology_sha256(contract.ProviderKind.OFFICIAL_D0)
        or readiness.get("forbidden_access_count") != 0
        or readiness.get("repeat_exact") is not True
    ):
        raise OfficialProviderError("D0R readiness authority drift")
    t0_context = t0.load_context(repository_root() / T0_PROFILE_PATH)
    expected_t0 = t0.composite_seal(
        t0_context,
        t0.coefficient_commitment(t0_context.profile),
        t0.metadata_commitments(t0_context.f0_profile),
        t0.artificial_conformance(t0_context.profile),
    )
    t0_seal = t0.validate_checked_official_input_seal(
        expected_t0, repository_root() / T0_SEAL_PATH
    )
    if (
        t0_seal.get("seal_sha256") != T0_SEAL_PAYLOAD_SHA256
        or cast(dict[str, Any], t0_seal["composite_seal"]).get(
            "prior_e0_execution_seal_sha256"
        )
        != E0_SEAL_PAYLOAD_SHA256
        or d0r_context.environment != t0_context.environment
    ):
        raise OfficialProviderError("T0/E0 authority or environment drift")
    receipt: dict[str, object] = {
        "development_target_rows": 0,
        "d0r_owner_sha256": D0R_OWNER_SHA256,
        "d0r_seal_document_sha256": D0R_SEAL_DOCUMENT_SHA256,
        "d0r_seal_payload_sha256": D0R_SEAL_PAYLOAD_SHA256,
        "fresh_v37_official_truth_values_evaluated": 0,
        "method_holdout_target_rows": 0,
        "network_requests": 0,
        "official_capabilities_issued": 0,
        "official_d0_target_rows": 0,
        "official_h0_target_rows": 0,
        "official_truth_rows_evaluated": 0,
        "protected_signal_values_decoded": 0,
        "real_signal_values_decoded": 0,
        "status": "VerifiedBeforeOfficialCapability",
        "t0_seal_document_sha256": T0_SEAL_DOCUMENT_SHA256,
        "t0_seal_payload_sha256": T0_SEAL_PAYLOAD_SHA256,
        "train_target_rows": 0,
    }
    return VerifiedD0PreAccess(d0r_context, t0_context, d0r_seal, t0_seal, receipt)


class OfficialD0Provider:
    """Materialize the fresh V37 train/development roles exactly once."""

    def __init__(self, verified: VerifiedD0PreAccess) -> None:
        self._verified = verified
        self._opened: set[contract.RoleKind] = set()
        self._built: dict[contract.RoleKind, c0.BuiltRole] = {}
        self._target_roots: dict[str, object] = {}
        self._truth_rows = 0

    @property
    def kind(self) -> contract.ProviderKind:
        return contract.ProviderKind.OFFICIAL_D0

    @property
    def namespace(self) -> str:
        return OFFICIAL_D0_NAMESPACE

    def materialize(
        self, role: contract.RoleKind, capability: contract.AccessCapabilityV1
    ) -> contract.SurfaceQueryBatchV1:
        if (
            capability.provider_kind is not self.kind
            or capability.namespace != self.namespace
            or capability.execution_seal_sha256 != D0R_SEAL_PAYLOAD_SHA256
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
        role_name = role.value.replace("-", "_")
        built = c0.build_role(self._verified.d0r_context.c0_context, role_name)
        metadata = t0.official_role_metadata(
            self._verified.t0_context.f0_profile, role_name
        )
        evaluated = t0.evaluate_targets(
            self._verified.t0_context.profile, built.batch, metadata
        )
        result = replace(built.batch, targets=evaluated.targets)
        self._built[role] = built
        self._opened.add(role)
        self._truth_rows += result.row_count
        self._target_roots[role.value] = {
            "component_roots": {
                name: t0.array_sha256(values)
                for name, values in sorted(evaluated.components.items())
            },
            "row_count": result.row_count,
            "structural_root_sha256": result.structural_root_sha256,
            "target_root_sha256": result.target_root_sha256,
        }
        return result

    def built(self, role: contract.RoleKind) -> c0.BuiltRole:
        try:
            return self._built[role]
        except KeyError as error:
            raise OfficialProviderError(
                "official role structure requested before materialization"
            ) from error

    def evidence(self) -> dict[str, object]:
        train_rows = (
            self._built[contract.RoleKind.TRAIN].batch.row_count
            if contract.RoleKind.TRAIN in self._built
            else 0
        )
        development_rows = (
            self._built[contract.RoleKind.DEVELOPMENT].batch.row_count
            if contract.RoleKind.DEVELOPMENT in self._built
            else 0
        )
        return {
            **self._verified.receipt,
            "development_target_rows": development_rows,
            "fresh_v37_official_truth_values_evaluated": self._truth_rows
            * contract.TARGET_AXIS_COUNT,
            "official_capabilities_issued": 1,
            "official_d0_target_rows": self._truth_rows,
            "official_truth_rows_evaluated": self._truth_rows,
            "role_roots": dict(sorted(self._target_roots.items())),
            "status": "OfficialD0RolesMaterialized"
            if self._truth_rows
            else "OfficialD0CapabilityIssued",
            "train_target_rows": train_rows,
        }


def prepare_official_d0() -> tuple[
    VerifiedD0PreAccess, contract.AccessCapabilityV1, OfficialD0Provider
]:
    verified = verify_pre_access()
    capability = contract.AccessCapabilityV1.official(
        contract.ProviderKind.OFFICIAL_D0,
        OFFICIAL_D0_NAMESPACE,
        D0R_SEAL_PAYLOAD_SHA256,
    )
    return verified, capability, OfficialD0Provider(verified)
