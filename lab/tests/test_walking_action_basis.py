from __future__ import annotations

import unittest
import json
from pathlib import Path
import subprocess
import tempfile
from unittest.mock import patch

import numpy as np

from next_lab.walking_action_basis import (
    ACTION_BASIS_MOTOR_STEPS,
    ACTION_WIDTH,
    Q1_30_ONE,
    action_tape_sha256,
    evaluate_action_basis,
    mirror_action_tape,
    walking_action_basis_tape,
    validate_action_basis_tape,
)
from lab.scripts.isaac_walking_action_basis_audit import supervise_audit


class WalkingActionBasisTests(unittest.TestCase):
    def test_supervisor_never_accepts_zero_exit_without_passed_gates(self) -> None:
        cases = (
            ({"status": "passed", "gates": {"complete": True}}, 0, 0),
            ({"status": "failed", "gates": {"complete": False}}, 0, 4),
            ({"status": "passed", "gates": {"complete": False}}, 0, 4),
            ({"status": "passed", "gates": {"complete": 1}}, 0, 4),
            ({"status": "passed", "gates": {}}, 0, 4),
            ([], 0, 4),
            (None, 0, 4),
            ({"status": "passed", "gates": {"complete": True}}, -11, 1),
        )
        for report, worker_exit, expected in cases:
            with self.subTest(report=report, worker_exit=worker_exit), tempfile.TemporaryDirectory() as root:
                output = Path(root) / "audit.json"

                def worker(command, check):
                    if report is not None:
                        output.write_text(json.dumps(report), encoding="utf-8")
                    return subprocess.CompletedProcess(command, worker_exit)

                with patch("lab.scripts.isaac_walking_action_basis_audit.subprocess.run", side_effect=worker):
                    self.assertEqual(supervise_audit(["worker"], output), expected)

    def test_supervisor_rejects_stale_report_without_starting_worker(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            output = Path(root) / "audit.json"
            output.write_text('{"status": "passed", "gates": {"complete": true}}')
            with patch("lab.scripts.isaac_walking_action_basis_audit.subprocess.run") as worker:
                with self.assertRaisesRegex(ValueError, "fresh"):
                    supervise_audit(["worker"], output)
                worker.assert_not_called()

    def test_tape_is_bounded_mirrored_and_float32_reversible(self) -> None:
        tape = walking_action_basis_tape()
        self.assertEqual(tape.shape, (3, ACTION_BASIS_MOTOR_STEPS, ACTION_WIDTH))
        self.assertEqual(tape.dtype, np.int64)
        self.assertTrue(np.all(np.abs(tape) <= Q1_30_ONE))
        np.testing.assert_array_equal(tape[0], np.zeros_like(tape[0]))
        np.testing.assert_array_equal(tape[2], mirror_action_tape(tape[1]))
        round_trip = np.rint(
            tape.astype(np.float32).astype(np.float64) / Q1_30_ONE * Q1_30_ONE
        ).astype(np.int64)
        np.testing.assert_array_equal(tape, round_trip)
        self.assertEqual(action_tape_sha256(tape),
                         "bdcaf944de58ade7472d166581618e5227ffb9121b715ac47f559ec9b2bec097")
        validate_action_basis_tape(tape)
        tape[1, 70, 0] += 1
        with self.assertRaisesRegex(ValueError, "frozen"):
            validate_action_basis_tape(tape)

    def test_gate_requires_bilateral_support_and_forward_progress(self) -> None:
        contacts = np.ones((3, ACTION_BASIS_MOTOR_STEPS, 2), dtype=np.bool_)
        contacts[1, 70:90] = [False, True]
        contacts[2, 70:90] = [True, False]
        positions = np.zeros((3, ACTION_BASIS_MOTOR_STEPS, 3), dtype=np.float64)
        positions[1:, -1, 2] = 0.02
        done = np.zeros((3, ACTION_BASIS_MOTOR_STEPS), dtype=np.bool_)
        report = evaluate_action_basis(
            contact_occupancy=contacts,
            root_position_m=positions,
            done=done,
        )
        self.assertEqual(report["status"], "passed")
        contacts[2] = True
        report = evaluate_action_basis(
            contact_occupancy=contacts,
            root_position_m=positions,
            done=done,
        )
        self.assertEqual(report["status"], "failed")
        self.assertFalse(report["gates"]["right_single_support"])

    def test_terminal_prefix_cannot_pass_or_include_reset_state(self) -> None:
        contacts = np.ones((3, 96, 2), dtype=np.bool_)
        contacts[1, 60:] = [False, True]
        contacts[2, 60:] = [True, False]
        positions = np.zeros((3, 96, 3))
        positions[1:, -1, 2] = 0.1
        done = np.zeros((3, 96), dtype=np.bool_)
        done[2, -1] = True
        report = evaluate_action_basis(contact_occupancy=contacts,
                                       root_position_m=positions, done=done)
        self.assertEqual(report["status"], "failed")
        self.assertFalse(report["gates"]["complete_horizon"])
        self.assertFalse(report["gates"]["no_early_terminal"])
        self.assertEqual(report["cases"]["right-swing"]["first_done_tick"], 96)
        self.assertIsNone(report["cases"]["left-swing"]["first_done_tick"])


if __name__ == "__main__":
    unittest.main()
