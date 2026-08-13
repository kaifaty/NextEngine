from __future__ import annotations

import unittest

from next_lab.reference_dynamic_feasibility import (
    REQUIRED_SAFETY_REASONS,
    DynamicFeasibilityAccumulator,
    build_dynamic_audit_cases,
    validate_remediation_authorization,
)


def _safety(**active: bool) -> dict[str, bool]:
    return {reason: active.get(reason, False) for reason in REQUIRED_SAFETY_REASONS}


class DynamicFeasibilityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.cases = build_dynamic_audit_cases(
            clips=(("clip-a", "train", 4),),
            horizon_motor_ticks=2,
            repeats=1,
        )
        self.accumulator = DynamicFeasibilityAccumulator(
            cases=self.cases,
            action_channel_ids=("joint.left", "joint.right"),
            contact_channel_ids=("contact.left", "contact.right"),
            contact_pair_ids=("ground:left", "ground:right", "left:right"),
            reset_safety_window_motor_ticks=1,
        )

    def _record(
        self,
        *,
        assignment_ordinal: int,
        tick: int,
        done: bool,
        safety: dict[str, bool] | None = None,
        tracking_lost: bool = False,
        hard_rom_excess: tuple[int, int] = (0, 0),
        velocity_excess: tuple[int, int] = (0, 0),
        effort: tuple[bool, bool] = (False, False),
        pair_masks: dict[str, tuple[bool, bool, bool]] | None = None,
    ) -> bool:
        case = self.cases[assignment_ordinal]
        safety = _safety() if safety is None else safety
        pair_masks = pair_masks or {
            "hard_impact": (False, False, False),
            "self_collision": (False, False, False),
            "forbidden_contact": (False, False, False),
        }
        failed = any(safety.values())
        return self.accumulator.record_step(
            assignment_ordinal=assignment_ordinal,
            clip_index=case.clip_index,
            start_frame=case.start_frame,
            reference_frame=case.start_frame + tick,
            elapsed_motor_ticks=tick,
            done=done,
            success=done and not failed,
            failure=failed,
            truncated=False,
            safety=safety,
            tracking_lost=tracking_lost,
            root_position_error_micrometres=100 * tick,
            root_orientation_absolute_dot_q1_30=(1 << 30) - tick,
            observed_joint_position_microradians=(10 * tick, -20 * tick),
            reference_joint_target_microradians=(0, 0),
            hard_rom_excess_microradians=hard_rom_excess,
            velocity_excess_microradians_per_second=velocity_excess,
            effort_envelope_violation=effort,
            observed_contacts=(1, 0),
            reference_contacts=(1, tick % 2),
            contact_pair_masks=pair_masks,
            contact_pair_maximum_impulses_micronewton_seconds=(
                6_100_000,
                0,
                0,
            ),
            forbidden_contact_mask=0,
        )

    def test_case_inventory_is_canonical_and_exhaustive(self) -> None:
        cases = build_dynamic_audit_cases(
            clips=(("a", "train", 4), ("b", "heldout", 5)),
            horizon_motor_ticks=2,
            repeats=2,
        )
        self.assertEqual(len(cases), 10)
        self.assertEqual(
            (cases[0].clip_id, cases[0].start_frame, cases[0].repeat_index),
            ("a", 0, 0),
        )
        self.assertEqual(
            (cases[-1].clip_id, cases[-1].start_frame, cases[-1].repeat_index),
            ("b", 2, 1),
        )
        with self.assertRaisesRegex(ValueError, "clip inventory"):
            build_dynamic_audit_cases(
                clips=(("b", "train", 4), ("a", "train", 4)),
                horizon_motor_ticks=2,
                repeats=1,
            )

    def test_remediation_authorization_is_hash_closed_and_optimizer_free(self) -> None:
        report = {
            "gate_id": "TRAIN-4",
            "decision": "RemediateDataOnly",
            "stage_status": "Reopened",
            "claim": "DataDynamicReferenceFeasibilityDiagnosticOnly",
            "exact_checks": "Fail",
            "learned_policy_quality_claim": False,
            "gate_optimizer_steps": 0,
            "gate_training_runs": 0,
            "supersedes": {"sha256": "a" * 64, "previous_decision": "Advance"},
            "identities": {
                "reference_tracker_profile_sha256": "b" * 64,
                "motion_corpus_manifest_sha256": "c" * 64,
                "body_schema_hash": "d" * 64,
                "compiled_descriptor_hash": "e" * 64,
                "usd_sha256": "f" * 64,
            },
            "requirement_disposition": {
                "requirement_id": "REQ-HUM-DATA-007",
                "status": "Fail",
                "report_only_metrics_cannot_waive": True,
            },
            "training_authorization": {
                "authorized": False,
                "allowed_scope": ["optimizer-free corpus diagnosis"],
                "forbidden": ["optimizer execution"],
            },
        }
        arguments = {
            "admission_gate_report_sha256": "a" * 64,
            "reference_tracker_profile_sha256": "b" * 64,
            "corpus_manifest_sha256": "c" * 64,
            "body_schema_hash": "d" * 64,
            "compiled_descriptor_hash": "e" * 64,
            "usd_sha256": "f" * 64,
        }
        validate_remediation_authorization(report, **arguments)
        validate_remediation_authorization(
            report,
            **{
                **arguments,
                "reference_tracker_profile_sha256": "1" * 64,
                "corpus_manifest_sha256": "2" * 64,
                "source_reference_tracker_profile_sha256": "b" * 64,
                "source_corpus_manifest_sha256": "c" * 64,
            },
        )
        report["training_authorization"]["authorized"] = True
        with self.assertRaisesRegex(ValueError, "closure mismatch"):
            validate_remediation_authorization(report, **arguments)

    def test_safe_horizon_passes_while_tracking_loss_remains_report_only(self) -> None:
        self._record(assignment_ordinal=0, tick=1, done=False, tracking_lost=True)
        self._record(assignment_ordinal=0, tick=2, done=True, tracking_lost=True)
        self._record(assignment_ordinal=1, tick=1, done=False)
        self._record(assignment_ordinal=1, tick=2, done=True)
        sections = self.accumulator.report_sections()
        overall = sections["overall"]
        self.assertEqual(overall["status"], "PASS")
        self.assertEqual(overall["completed_case_count"], 2)
        self.assertEqual(overall["required_safety_failed_case_count"], 0)
        phase = sections["phase_results"][0]
        self.assertEqual(phase["status"], "PASS")
        self.assertEqual(phase["first_tracking_lost_motor_tick"], 1)
        self.assertEqual(
            phase["contact_confusion_by_contact_channel"]["true_positive"],
            [2, 0],
        )
        self.assertEqual(
            phase["contact_confusion_by_contact_channel"]["false_negative"],
            [0, 1],
        )

    def test_safety_failure_is_localized_to_phase_tick_joint_and_pair(self) -> None:
        safety = _safety(
            hard_rom=True,
            joint_safety=True,
            joint_velocity=True,
            effort_envelope=True,
            hard_impact=True,
        )
        pair_masks = {
            "hard_impact": (True, False, False),
            "self_collision": (False, False, False),
            "forbidden_contact": (False, False, False),
        }
        self._record(
            assignment_ordinal=0,
            tick=1,
            done=True,
            safety=safety,
            hard_rom_excess=(11, 0),
            velocity_excess=(0, 5),
            effort=(False, True),
            pair_masks=pair_masks,
        )
        sections = self.accumulator.report_sections()
        violation = sections["phase_results"][0][
            "first_required_safety_violation"
        ]
        self.assertEqual(violation["clip_id"], "clip-a")
        self.assertEqual(violation["start_frame"], 0)
        self.assertEqual(violation["terminal_motor_tick"], 1)
        self.assertEqual(
            [item["channel_id"] for item in violation["joint_channels"]],
            ["joint.left", "joint.right"],
        )
        self.assertEqual(
            violation["contact_pairs"][0]["pair_id"], "ground:left"
        )
        overall = sections["overall"]
        self.assertEqual(overall["status"], "FAIL")
        self.assertEqual(overall["reset_window_safety_failed_case_count"], 1)
        self.assertEqual(overall["required_safety_event_counts"]["hard_impact"], 1)
        self.assertEqual(
            overall["joint_violation_counts_by_reason_and_action_channel"][
                "hard_rom"
            ],
            {"joint.left": 1},
        )
        self.assertEqual(
            overall["contact_violation_counts_by_reason_and_pair_id"][
                "hard_impact"
            ],
            {"ground:left": 1},
        )

    def test_missing_tick_and_unexpected_cycle_are_fail_closed(self) -> None:
        with self.assertRaisesRegex(ValueError, "identity or cadence"):
            self._record(assignment_ordinal=0, tick=2, done=True)
        self.assertFalse(
            self.accumulator.record_step(
                assignment_ordinal=len(self.cases),
                clip_index=0,
                start_frame=0,
                reference_frame=1,
                elapsed_motor_ticks=1,
                done=False,
                success=False,
                failure=False,
                truncated=False,
                safety=_safety(),
                tracking_lost=False,
                root_position_error_micrometres=0,
                root_orientation_absolute_dot_q1_30=1 << 30,
                observed_joint_position_microradians=(0, 0),
                reference_joint_target_microradians=(0, 0),
                hard_rom_excess_microradians=(0, 0),
                velocity_excess_microradians_per_second=(0, 0),
                effort_envelope_violation=(False, False),
                observed_contacts=(0, 0),
                reference_contacts=(0, 0),
                contact_pair_masks={
                    "hard_impact": (False, False, False),
                    "self_collision": (False, False, False),
                    "forbidden_contact": (False, False, False),
                },
                contact_pair_maximum_impulses_micronewton_seconds=(0, 0, 0),
                forbidden_contact_mask=0,
            )
        )


if __name__ == "__main__":
    unittest.main()
