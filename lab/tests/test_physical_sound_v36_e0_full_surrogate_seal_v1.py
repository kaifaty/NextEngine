from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
sys.path.insert(0, str(SCRIPTS))

import numpy as np
import physical_sound_v36_e0_full_surrogate_seal_v1 as e0
import physical_sound_v36_owner_contract_v1 as contract


class NullProvider:
    @property
    def kind(self) -> contract.ProviderKind:
        return contract.ProviderKind.SURROGATE_H0

    @property
    def namespace(self) -> str:
        return "discarded-e0-null-h0"

    def materialize(
        self, role: contract.RoleKind, capability: contract.AccessCapability
    ) -> contract.RoleBatch:
        raise AssertionError((role, capability))


def fake_rehearsal_record(owner_sha256: str) -> dict[str, object]:
    return {
        "d0": {
            "artifact_root_sha256": "a" * 64,
            "topology_sha256": contract.expected_topology_sha256(
                contract.PipelineKind.D0
            ),
        },
        "environment_sha256": "b" * 64,
        "h0": {
            "artifact_root_sha256": "c" * 64,
            "topology_sha256": contract.expected_topology_sha256(
                contract.PipelineKind.H0
            ),
        },
        "official_access": e0.zero_official_access(),
        "owner": {"sha256": owner_sha256},
        "profile_sha256": e0.PROFILE_SHA256,
    }


def write_fake_rehearsal(path: Path, owner_sha256: str) -> None:
    path.mkdir()
    (path / "rehearsal.json").write_bytes(
        e0.canonical_json(fake_rehearsal_record(owner_sha256))
    )
    (path / "report.json").write_bytes(e0.canonical_json({"decision": "Pass"}))
    (path / "terminal.json").write_bytes(e0.canonical_json({"terminal": "Pass"}))


def complete_d0_trace() -> contract.ExecutionTrace:
    stages = (
        *contract.stages_for_pipeline(contract.PipelineKind.D0),
        contract.LifecycleStage.TERMINAL_PUBLICATION,
    )
    return contract.ExecutionTrace(
        pipeline=contract.PipelineKind.D0,
        provider_kind=contract.ProviderKind.SURROGATE_D0,
        events=tuple(
            contract.TraceEvent(
                ordinal=index,
                stage=stage,
                callable_id=contract.STAGE_CALLABLES[stage],
            )
            for index, stage in enumerate(stages)
        ),
        terminal=contract.TerminalDecision.PASS,
        access=contract.AccessLedger(
            provider_calls=2,
            train_target_rows=1,
            development_target_rows=1,
        ),
    )


class FullSurrogateSealTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.context = e0.load_context(Path(e0.PROFILE_PATH))

    def test_context_removes_fresh_oracle_and_preserves_science_shape(self) -> None:
        self.assertNotIn("oracle", self.context.effective)
        self.assertEqual(self.context.effective["training"]["steps"], 1200)
        method = self.context.effective["method_overlay"]
        self.assertEqual(method["candidate"]["model"]["parameter_count"], 1859)
        self.assertEqual(len(method["controls"]), 9)
        self.assertEqual(len(method["ablations"]), 3)

    def test_role_enum_maps_only_holdout_to_profile_spelling(self) -> None:
        self.assertEqual(e0.profile_role_name(contract.RoleKind.TRAIN), "train")
        self.assertEqual(
            e0.profile_role_name(contract.RoleKind.DEVELOPMENT), "development"
        )
        self.assertEqual(
            e0.profile_role_name(contract.RoleKind.METHOD_HOLDOUT),
            "method_holdout",
        )

    def test_provider_identity_changes_only_claim_freeze_and_access_provenance(
        self,
    ) -> None:
        self.assertEqual(
            e0.claim_for_provider(contract.ProviderKind.SURROGATE_D0), e0.CLAIM
        )
        self.assertEqual(
            e0.claim_for_provider(contract.ProviderKind.OFFICIAL_D0),
            e0.OFFICIAL_D0_CLAIM,
        )
        seal = contract.ExecutionSeal(
            contract_schema=contract.CONTRACT_SCHEMA,
            owner_sha256="a" * 64,
            profile_sha256="b" * 64,
            environment_sha256="c" * 64,
            d0_rehearsal_root_sha256="d" * 64,
            h0_rehearsal_root_sha256="e" * 64,
            d0_topology_sha256=contract.expected_topology_sha256(
                contract.PipelineKind.D0
            ),
            h0_topology_sha256=contract.expected_topology_sha256(
                contract.PipelineKind.H0
            ),
            rehearsal_run_count=2,
            repeat_exact=True,
            forbidden_access_count=0,
        )
        d0_capability = contract.AccessCapability.official(
            contract.ProviderKind.OFFICIAL_D0, "v36-test-d0", seal
        )
        weights = b"candidate"
        freeze = json.loads(
            e0.candidate_freeze_document(self.context, d0_capability, weights)
        )
        self.assertEqual(freeze["provider_kind"], "official-d0")
        self.assertEqual(freeze["status"], "OfficialD0CandidateFrozenAfterPass")
        self.assertEqual(freeze["execution_seal"], e0.execution_seal_record(seal))

        bundle = e0.CandidateBundle(weights, b"raw", b"v34", b"no-geometry", b"")
        bundle = e0.CandidateBundle(
            bundle.candidate_weights,
            bundle.raw_mlp_weights,
            bundle.v34_shaped_weights,
            bundle.without_geometry_weights,
            e0.candidate_freeze_document(self.context, d0_capability, weights),
        )
        h0_capability = contract.AccessCapability.official(
            contract.ProviderKind.OFFICIAL_H0, "v36-test-h0", seal
        )
        self.assertEqual(
            e0.validate_candidate_bundle(self.context, h0_capability, bundle)["status"],
            "OfficialD0CandidateFrozenAfterPass",
        )
        trace = contract.ExecutionTrace(
            pipeline=contract.PipelineKind.D0,
            provider_kind=contract.ProviderKind.OFFICIAL_D0,
            events=(),
            terminal=contract.TerminalDecision.PASS,
            access=contract.AccessLedger(
                provider_calls=2,
                train_target_rows=6480,
                development_target_rows=4320,
                official_d0_target_rows=10800,
            ),
        )
        access = e0.official_access_record(trace)
        self.assertEqual(access["official_capabilities_issued"], 1)
        self.assertEqual(access["official_d0_target_rows"], 10800)
        self.assertEqual(access["fresh_v36_truth_values_evaluated"], 32400)

    def test_immutable_matrix_is_float64_c_contiguous_and_read_only(self) -> None:
        matrix = e0.immutable_matrix(np.asarray([[1.0, 2.0]], dtype=np.float32))
        self.assertEqual(matrix.dtype, np.dtype(np.float64))
        self.assertTrue(matrix.flags.c_contiguous)
        self.assertFalse(matrix.flags.writeable)

    def test_h0_without_candidate_rejects_before_any_role_or_output(self) -> None:
        provider = NullProvider()
        capability = contract.AccessCapability.surrogate(
            provider.kind, provider.namespace
        )
        with tempfile.TemporaryDirectory(prefix="nextengine-v36-e0-pre-") as temporary:
            output = Path(temporary) / "output"
            result = e0.execute_owner(
                self.context, capability, provider, output, candidate_bundle=None
            )
            self.assertIs(
                result.trace.terminal, contract.TerminalDecision.CONTRACT_REJECT
            )
            self.assertEqual(result.trace.access.target_rows_accessed, 0)
            self.assertFalse(output.exists())

    def test_seal_builder_accepts_only_two_byte_identical_rehearsals(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v36-e0-seal-") as temporary:
            root = Path(temporary)
            run_a = root / "a"
            run_b = root / "b"
            write_fake_rehearsal(run_a, "d" * 64)
            write_fake_rehearsal(run_b, "d" * 64)
            seal = e0.build_execution_seal(run_a, run_b)
            execution = seal["execution_seal"]
            self.assertIsInstance(execution, dict)
            self.assertEqual(execution["rehearsal_run_count"], 2)
            self.assertIs(execution["repeat_exact"], True)
            self.assertEqual(execution["forbidden_access_count"], 0)

            report = json.loads((run_b / "report.json").read_text())
            report["mutation"] = True
            (run_b / "report.json").write_bytes(e0.canonical_json(report))
            with self.assertRaisesRegex(e0.E0RehearsalError, "not byte-identical"):
                e0.build_execution_seal(run_a, run_b)

    def test_seal_builder_rejects_nested_symlink(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v36-e0-link-") as temporary:
            run = Path(temporary) / "run"
            write_fake_rehearsal(run, "d" * 64)
            (run / "alias.json").symlink_to(run / "report.json")
            with self.assertRaisesRegex(e0.E0RehearsalError, "non-symlinked"):
                e0.rehearsal_tree(run)

    def test_terminal_matrix_creates_container_and_all_scientific_paths(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-e0-matrix-"
        ) as temporary:
            matrix_root = Path(temporary) / "matrix"
            receipts = e0.terminal_matrix(
                complete_d0_trace(), contract.PipelineKind.D0, matrix_root
            )
            self.assertEqual(
                set(receipts),
                {decision.value for decision in e0.SCIENTIFIC_TERMINALS},
            )
            for decision in e0.SCIENTIFIC_TERMINALS:
                self.assertTrue(
                    (matrix_root / decision.value / "terminal.json").is_file()
                )

    def test_bound_dependency_drift_fails(self) -> None:
        declaration = self.context.profile["parent"]["x0_result"]
        with self.assertRaisesRegex(e0.E0RehearsalError, "bound file drift"):
            e0.bound_file(declaration["path"], "0" * 64)


if __name__ == "__main__":
    unittest.main()
