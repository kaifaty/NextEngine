from __future__ import annotations

from dataclasses import dataclass, field
from typing import Mapping, Sequence


REQUIRED_SAFETY_REASONS = (
    "hard_rom",
    "joint_safety",
    "joint_velocity",
    "effort_envelope",
    "hard_impact",
    "self_collision",
    "forbidden_contact",
    "fall",
    "world_bounds",
    "non_finite",
)
CONTACT_SAFETY_REASONS = (
    "hard_impact",
    "self_collision",
    "forbidden_contact",
)


@dataclass(frozen=True)
class DynamicAuditCase:
    ordinal: int
    clip_index: int
    clip_id: str
    split: str
    start_frame: int
    terminal_frame: int
    repeat_index: int

    @property
    def horizon_motor_ticks(self) -> int:
        return self.terminal_frame - self.start_frame


def build_dynamic_audit_cases(
    *,
    clips: Sequence[tuple[str, str, int]],
    horizon_motor_ticks: int,
    repeats: int,
) -> tuple[DynamicAuditCase, ...]:
    if (
        not clips
        or horizon_motor_ticks <= 0
        or isinstance(repeats, bool)
        or not isinstance(repeats, int)
        or repeats <= 0
    ):
        raise ValueError("invalid dynamic audit case bounds")
    clip_ids = [clip_id for clip_id, _, _ in clips]
    if (
        len(clip_ids) != len(set(clip_ids))
        or clip_ids != sorted(clip_ids, key=lambda value: value.encode("utf-8"))
        or any(
            not clip_id
            or not split
            or isinstance(frame_count, bool)
            or not isinstance(frame_count, int)
            or frame_count <= horizon_motor_ticks
            for clip_id, split, frame_count in clips
        )
    ):
        raise ValueError("invalid dynamic audit clip inventory")
    cases: list[DynamicAuditCase] = []
    for clip_index, (clip_id, split, frame_count) in enumerate(clips):
        for start_frame in range(frame_count - horizon_motor_ticks):
            for repeat_index in range(repeats):
                cases.append(
                    DynamicAuditCase(
                        ordinal=len(cases),
                        clip_index=clip_index,
                        clip_id=clip_id,
                        split=split,
                        start_frame=start_frame,
                        terminal_frame=start_frame + horizon_motor_ticks,
                        repeat_index=repeat_index,
                    )
                )
    return tuple(cases)


def validate_remediation_authorization(
    report: Mapping[str, object],
    *,
    admission_gate_report_sha256: str,
    reference_tracker_profile_sha256: str,
    corpus_manifest_sha256: str,
    body_schema_hash: str,
    compiled_descriptor_hash: str,
    usd_sha256: str,
    source_reference_tracker_profile_sha256: str | None = None,
    source_corpus_manifest_sha256: str | None = None,
) -> None:
    identities = report.get("identities")
    supersedes = report.get("supersedes")
    requirement = report.get("requirement_disposition")
    authorization = report.get("training_authorization")
    if not all(
        isinstance(value, Mapping)
        for value in (identities, supersedes, requirement, authorization)
    ):
        raise ValueError("TRAIN-4 remediation authorization is incomplete")
    authorized_tracker = (
        source_reference_tracker_profile_sha256
        or reference_tracker_profile_sha256
    )
    authorized_manifest = source_corpus_manifest_sha256 or corpus_manifest_sha256
    allowed_scope = authorization.get("allowed_scope")
    forbidden = authorization.get("forbidden")
    if (
        report.get("gate_id") != "TRAIN-4"
        or report.get("decision") != "RemediateDataOnly"
        or report.get("stage_status") != "Reopened"
        or report.get("claim") != "DataDynamicReferenceFeasibilityDiagnosticOnly"
        or report.get("exact_checks") != "Fail"
        or report.get("learned_policy_quality_claim") is not False
        or report.get("gate_optimizer_steps") != 0
        or report.get("gate_training_runs") != 0
        or supersedes.get("sha256") != admission_gate_report_sha256
        or supersedes.get("previous_decision") != "Advance"
        or identities.get("reference_tracker_profile_sha256")
        != authorized_tracker
        or identities.get("motion_corpus_manifest_sha256")
        != authorized_manifest
        or identities.get("body_schema_hash") != body_schema_hash
        or identities.get("compiled_descriptor_hash")
        != compiled_descriptor_hash
        or identities.get("usd_sha256") != usd_sha256
        or requirement.get("requirement_id") != "REQ-HUM-DATA-007"
        or requirement.get("status") != "Fail"
        or requirement.get("report_only_metrics_cannot_waive") is not True
        or authorization.get("authorized") is not False
        or not isinstance(allowed_scope, list)
        or "optimizer-free corpus diagnosis" not in allowed_scope
        or not isinstance(forbidden, list)
        or "optimizer execution" not in forbidden
    ):
        raise ValueError("TRAIN-4 remediation authorization closure mismatch")


@dataclass
class _CaseState:
    case: DynamicAuditCase
    action_channel_count: int
    contact_channel_count: int
    observed_motor_ticks: int = 0
    terminal: bool = False
    terminal_success: bool = False
    terminal_failure: bool = False
    terminal_truncated: bool = False
    safety_reasons: tuple[str, ...] = ()
    first_tracking_lost_tick: int | None = None
    maximum_root_position_error_micrometres: int = 0
    minimum_root_orientation_absolute_dot_q1_30: int | None = None
    maximum_joint_tracking_error_microradians: list[int] = field(init=False)
    contact_true_positive: list[int] = field(init=False)
    contact_false_positive: list[int] = field(init=False)
    contact_false_negative: list[int] = field(init=False)
    contact_true_negative: list[int] = field(init=False)
    first_violation: dict[str, object] | None = None

    def __post_init__(self) -> None:
        self.maximum_joint_tracking_error_microradians = (
            [0] * self.action_channel_count
        )
        self.contact_true_positive = [0] * self.contact_channel_count
        self.contact_false_positive = [0] * self.contact_channel_count
        self.contact_false_negative = [0] * self.contact_channel_count
        self.contact_true_negative = [0] * self.contact_channel_count


class DynamicFeasibilityAccumulator:
    def __init__(
        self,
        *,
        cases: Sequence[DynamicAuditCase],
        action_channel_ids: Sequence[str],
        contact_channel_ids: Sequence[str],
        contact_pair_ids: Sequence[str],
        reset_safety_window_motor_ticks: int,
    ) -> None:
        if (
            not cases
            or not action_channel_ids
            or len(action_channel_ids) != len(set(action_channel_ids))
            or not contact_channel_ids
            or len(contact_channel_ids) != len(set(contact_channel_ids))
            or not contact_pair_ids
            or len(contact_pair_ids) != len(set(contact_pair_ids))
            or reset_safety_window_motor_ticks <= 0
            or tuple(case.ordinal for case in cases) != tuple(range(len(cases)))
            or len({case.clip_id for case in cases}) == 0
        ):
            raise ValueError("invalid dynamic feasibility accumulator configuration")
        horizon = cases[0].horizon_motor_ticks
        if horizon <= 0 or any(case.horizon_motor_ticks != horizon for case in cases):
            raise ValueError("dynamic audit cases do not share one positive horizon")
        self.cases = tuple(cases)
        self.action_channel_ids = tuple(action_channel_ids)
        self.contact_channel_ids = tuple(contact_channel_ids)
        self.contact_pair_ids = tuple(contact_pair_ids)
        self.reset_safety_window_motor_ticks = reset_safety_window_motor_ticks
        self._states = [
            _CaseState(
                case=case,
                action_channel_count=len(self.action_channel_ids),
                contact_channel_count=len(self.contact_channel_ids),
            )
            for case in self.cases
        ]

    @property
    def completed_case_count(self) -> int:
        return sum(state.terminal for state in self._states)

    @property
    def case_count(self) -> int:
        return len(self.cases)

    def record_step(
        self,
        *,
        assignment_ordinal: int,
        clip_index: int,
        start_frame: int,
        reference_frame: int,
        elapsed_motor_ticks: int,
        done: bool,
        success: bool,
        failure: bool,
        truncated: bool,
        safety: Mapping[str, bool],
        tracking_lost: bool,
        root_position_error_micrometres: int,
        root_orientation_absolute_dot_q1_30: int,
        observed_joint_position_microradians: Sequence[int],
        reference_joint_target_microradians: Sequence[int],
        hard_rom_excess_microradians: Sequence[int],
        velocity_excess_microradians_per_second: Sequence[int],
        effort_envelope_violation: Sequence[bool],
        observed_contacts: Sequence[int],
        reference_contacts: Sequence[int],
        contact_pair_masks: Mapping[str, Sequence[bool]],
        contact_pair_maximum_impulses_micronewton_seconds: Sequence[int],
        forbidden_contact_mask: int,
    ) -> bool:
        if assignment_ordinal < 0:
            raise ValueError("diagnostic assignment ordinal is negative")
        if assignment_ordinal >= len(self._states):
            return False
        state = self._states[assignment_ordinal]
        case = state.case
        if state.terminal:
            raise ValueError("dynamic audit case emitted a step after termination")
        if (
            clip_index != case.clip_index
            or start_frame != case.start_frame
            or elapsed_motor_ticks != state.observed_motor_ticks + 1
            or not 1 <= elapsed_motor_ticks <= case.horizon_motor_ticks
            or reference_frame != case.start_frame + elapsed_motor_ticks
            or set(safety) != set(REQUIRED_SAFETY_REASONS)
            or set(contact_pair_masks) != set(CONTACT_SAFETY_REASONS)
            or (not done and (success or failure or truncated))
            or root_position_error_micrometres < 0
            or not 0 <= root_orientation_absolute_dot_q1_30 <= 1 << 30
        ):
            raise ValueError("dynamic audit episode identity or cadence mismatch")
        self._validate_widths(
            observed_joint_position_microradians=observed_joint_position_microradians,
            reference_joint_target_microradians=reference_joint_target_microradians,
            hard_rom_excess_microradians=hard_rom_excess_microradians,
            velocity_excess_microradians_per_second=(
                velocity_excess_microradians_per_second
            ),
            effort_envelope_violation=effort_envelope_violation,
            observed_contacts=observed_contacts,
            reference_contacts=reference_contacts,
            contact_pair_masks=contact_pair_masks,
            contact_pair_maximum_impulses_micronewton_seconds=(
                contact_pair_maximum_impulses_micronewton_seconds
            ),
        )
        reasons = tuple(reason for reason in REQUIRED_SAFETY_REASONS if safety[reason])
        if (reasons and not done) or (
            elapsed_motor_ticks == case.horizon_motor_ticks and not done
        ):
            raise ValueError("dynamic audit terminal contract was not honored")

        state.observed_motor_ticks = elapsed_motor_ticks
        if tracking_lost and state.first_tracking_lost_tick is None:
            state.first_tracking_lost_tick = elapsed_motor_ticks
        state.maximum_root_position_error_micrometres = max(
            state.maximum_root_position_error_micrometres,
            root_position_error_micrometres,
        )
        if (
            state.minimum_root_orientation_absolute_dot_q1_30 is None
            or root_orientation_absolute_dot_q1_30
            < state.minimum_root_orientation_absolute_dot_q1_30
        ):
            state.minimum_root_orientation_absolute_dot_q1_30 = (
                root_orientation_absolute_dot_q1_30
            )
        for channel, (observed, reference) in enumerate(
            zip(
                observed_joint_position_microradians,
                reference_joint_target_microradians,
                strict=True,
            )
        ):
            state.maximum_joint_tracking_error_microradians[channel] = max(
                state.maximum_joint_tracking_error_microradians[channel],
                abs(observed - reference),
            )
        for channel, (observed, reference) in enumerate(
            zip(observed_contacts, reference_contacts, strict=True)
        ):
            if observed not in (0, 1) or reference not in (0, 1):
                raise ValueError("dynamic audit contact sample is not binary")
            if observed and reference:
                state.contact_true_positive[channel] += 1
            elif observed:
                state.contact_false_positive[channel] += 1
            elif reference:
                state.contact_false_negative[channel] += 1
            else:
                state.contact_true_negative[channel] += 1

        if done:
            if reasons:
                if success or not failure or truncated:
                    raise ValueError("safety terminal has an invalid outcome")
            elif (
                not success
                or failure
                or truncated
                or elapsed_motor_ticks != case.horizon_motor_ticks
            ):
                raise ValueError("safe dynamic audit case did not complete its horizon")
            state.terminal = True
            state.terminal_success = success
            state.terminal_failure = failure
            state.terminal_truncated = truncated
            state.safety_reasons = reasons
            if reasons:
                state.first_violation = self._violation_record(
                    state=state,
                    reference_frame=reference_frame,
                    elapsed_motor_ticks=elapsed_motor_ticks,
                    reasons=reasons,
                    observed_joint_position_microradians=(
                        observed_joint_position_microradians
                    ),
                    reference_joint_target_microradians=(
                        reference_joint_target_microradians
                    ),
                    hard_rom_excess_microradians=hard_rom_excess_microradians,
                    velocity_excess_microradians_per_second=(
                        velocity_excess_microradians_per_second
                    ),
                    effort_envelope_violation=effort_envelope_violation,
                    contact_pair_masks=contact_pair_masks,
                    contact_pair_maximum_impulses_micronewton_seconds=(
                        contact_pair_maximum_impulses_micronewton_seconds
                    ),
                    root_position_error_micrometres=root_position_error_micrometres,
                    root_orientation_absolute_dot_q1_30=(
                        root_orientation_absolute_dot_q1_30
                    ),
                    forbidden_contact_mask=forbidden_contact_mask,
                )
        return True

    def report_sections(self) -> dict[str, object]:
        phase_results = [self._phase_result(state) for state in self._states]
        clip_ids = tuple(
            dict.fromkeys(case.clip_id for case in self.cases)
        )
        clip_summaries = {
            clip_id: self._summary(
                [state for state in self._states if state.case.clip_id == clip_id]
            )
            for clip_id in clip_ids
        }
        overall = self._summary(self._states)
        missing = [state.case.ordinal for state in self._states if not state.terminal]
        overall["missing_case_ordinals"] = missing
        overall["coverage_complete"] = not missing
        overall["status"] = (
            "PASS"
            if not missing and overall["required_safety_failed_case_count"] == 0
            else "FAIL"
        )
        return {
            "overall": overall,
            "clip_summaries": clip_summaries,
            "phase_results": phase_results,
        }

    def _validate_widths(self, **values: object) -> None:
        action_names = {
            "observed_joint_position_microradians",
            "reference_joint_target_microradians",
            "hard_rom_excess_microradians",
            "velocity_excess_microradians_per_second",
            "effort_envelope_violation",
        }
        contact_names = {"observed_contacts", "reference_contacts"}
        pair_names = {
            "contact_pair_maximum_impulses_micronewton_seconds"
        }
        for name, value in values.items():
            if name == "contact_pair_masks":
                if any(
                    len(mask) != len(self.contact_pair_ids)
                    for mask in value.values()  # type: ignore[union-attr]
                ):
                    raise ValueError("dynamic audit contact-pair width mismatch")
                continue
            expected = (
                len(self.action_channel_ids)
                if name in action_names
                else len(self.contact_channel_ids)
                if name in contact_names
                else len(self.contact_pair_ids)
                if name in pair_names
                else -1
            )
            if len(value) != expected:  # type: ignore[arg-type]
                raise ValueError(f"dynamic audit {name} width mismatch")

    def _violation_record(
        self,
        *,
        state: _CaseState,
        reference_frame: int,
        elapsed_motor_ticks: int,
        reasons: tuple[str, ...],
        observed_joint_position_microradians: Sequence[int],
        reference_joint_target_microradians: Sequence[int],
        hard_rom_excess_microradians: Sequence[int],
        velocity_excess_microradians_per_second: Sequence[int],
        effort_envelope_violation: Sequence[bool],
        contact_pair_masks: Mapping[str, Sequence[bool]],
        contact_pair_maximum_impulses_micronewton_seconds: Sequence[int],
        root_position_error_micrometres: int,
        root_orientation_absolute_dot_q1_30: int,
        forbidden_contact_mask: int,
    ) -> dict[str, object]:
        joint_channels = []
        for channel, channel_id in enumerate(self.action_channel_ids):
            categories = []
            if hard_rom_excess_microradians[channel] > 10:
                categories.append("hard_rom")
            if velocity_excess_microradians_per_second[channel] > 0:
                categories.append("joint_velocity")
            if effort_envelope_violation[channel]:
                categories.append("effort_envelope")
            if categories:
                observed = observed_joint_position_microradians[channel]
                reference = reference_joint_target_microradians[channel]
                joint_channels.append(
                    {
                        "action_channel": channel,
                        "channel_id": channel_id,
                        "categories": categories,
                        "hard_rom_excess_microradians": (
                            hard_rom_excess_microradians[channel]
                        ),
                        "velocity_excess_microradians_per_second": (
                            velocity_excess_microradians_per_second[channel]
                        ),
                        "observed_position_microradians": observed,
                        "reference_target_microradians": reference,
                        "absolute_tracking_error_microradians": abs(
                            observed - reference
                        ),
                    }
                )
        contact_pairs = []
        for pair_index, pair_id in enumerate(self.contact_pair_ids):
            categories = [
                reason
                for reason in CONTACT_SAFETY_REASONS
                if contact_pair_masks[reason][pair_index]
            ]
            if categories:
                contact_pairs.append(
                    {
                        "pair_index": pair_index,
                        "pair_id": pair_id,
                        "categories": categories,
                        "episode_maximum_impulse_micronewton_seconds": (
                            contact_pair_maximum_impulses_micronewton_seconds[
                                pair_index
                            ]
                        ),
                    }
                )
        return {
            "clip_id": state.case.clip_id,
            "start_frame": state.case.start_frame,
            "repeat_index": state.case.repeat_index,
            "terminal_motor_tick": elapsed_motor_ticks,
            "reference_frame": reference_frame,
            "reasons": list(reasons),
            "joint_channels": joint_channels,
            "contact_pairs": contact_pairs,
            "root_position_error_micrometres": root_position_error_micrometres,
            "root_orientation_absolute_dot_q1_30": (
                root_orientation_absolute_dot_q1_30
            ),
            "forbidden_contact_mask": forbidden_contact_mask,
        }

    def _phase_result(self, state: _CaseState) -> dict[str, object]:
        case = state.case
        return {
            "case_ordinal": case.ordinal,
            "clip_id": case.clip_id,
            "split": case.split,
            "start_frame": case.start_frame,
            "terminal_frame": case.terminal_frame,
            "repeat_index": case.repeat_index,
            "status": (
                "NOT_RUN"
                if not state.terminal
                else "FAIL"
                if state.safety_reasons
                else "PASS"
            ),
            "observed_motor_ticks": state.observed_motor_ticks,
            "reference_complete": state.terminal_success,
            "terminal_failure": state.terminal_failure,
            "terminal_truncated": state.terminal_truncated,
            "tracking_lost_report_only": state.first_tracking_lost_tick is not None,
            "first_tracking_lost_motor_tick": state.first_tracking_lost_tick,
            "maximum_root_position_error_micrometres": (
                state.maximum_root_position_error_micrometres
            ),
            "minimum_root_orientation_absolute_dot_q1_30": (
                state.minimum_root_orientation_absolute_dot_q1_30
            ),
            "maximum_joint_tracking_error_microradians_by_action_channel": (
                state.maximum_joint_tracking_error_microradians
            ),
            "contact_confusion_by_contact_channel": {
                "true_positive": state.contact_true_positive,
                "false_positive": state.contact_false_positive,
                "false_negative": state.contact_false_negative,
                "true_negative": state.contact_true_negative,
            },
            "first_required_safety_violation": state.first_violation,
        }

    def _summary(self, states: Sequence[_CaseState]) -> dict[str, object]:
        terminal_states = [state for state in states if state.terminal]
        safety_counts = {
            reason: sum(reason in state.safety_reasons for state in terminal_states)
            for reason in REQUIRED_SAFETY_REASONS
        }
        contact_counts = {
            key: [0] * len(self.contact_channel_ids)
            for key in (
                "true_positive",
                "false_positive",
                "false_negative",
                "true_negative",
            )
        }
        maximum_joint_error = [0] * len(self.action_channel_ids)
        maximum_root_error = 0
        minimum_orientation: int | None = None
        for state in states:
            maximum_root_error = max(
                maximum_root_error,
                state.maximum_root_position_error_micrometres,
            )
            if state.minimum_root_orientation_absolute_dot_q1_30 is not None:
                minimum_orientation = (
                    state.minimum_root_orientation_absolute_dot_q1_30
                    if minimum_orientation is None
                    else min(
                        minimum_orientation,
                        state.minimum_root_orientation_absolute_dot_q1_30,
                    )
                )
            for channel in range(len(self.action_channel_ids)):
                maximum_joint_error[channel] = max(
                    maximum_joint_error[channel],
                    state.maximum_joint_tracking_error_microradians[channel],
                )
            for key, source in (
                ("true_positive", state.contact_true_positive),
                ("false_positive", state.contact_false_positive),
                ("false_negative", state.contact_false_negative),
                ("true_negative", state.contact_true_negative),
            ):
                for channel, value in enumerate(source):
                    contact_counts[key][channel] += value
        true_positive = sum(contact_counts["true_positive"])
        false_positive = sum(contact_counts["false_positive"])
        false_negative = sum(contact_counts["false_negative"])
        first_examples = {}
        for reason in REQUIRED_SAFETY_REASONS:
            state = next(
                (state for state in terminal_states if reason in state.safety_reasons),
                None,
            )
            if state is not None:
                first_examples[reason] = state.first_violation
        joint_channel_counts = {
            reason: {channel_id: 0 for channel_id in self.action_channel_ids}
            for reason in ("hard_rom", "joint_velocity", "effort_envelope")
        }
        contact_pair_counts = {
            reason: {pair_id: 0 for pair_id in self.contact_pair_ids}
            for reason in CONTACT_SAFETY_REASONS
        }
        for state in terminal_states:
            violation = state.first_violation
            if violation is None:
                continue
            for item in violation["joint_channels"]:
                channel_id = item["channel_id"]
                for reason in item["categories"]:
                    joint_channel_counts[reason][channel_id] += 1
            for item in violation["contact_pairs"]:
                pair_id = item["pair_id"]
                for reason in item["categories"]:
                    contact_pair_counts[reason][pair_id] += 1
        terminal_tick_counts = {
            reason: {
                str(tick): sum(
                    reason in state.safety_reasons
                    and state.observed_motor_ticks == tick
                    for state in terminal_states
                )
                for tick in sorted(
                    {
                        state.observed_motor_ticks
                        for state in terminal_states
                        if reason in state.safety_reasons
                    }
                )
            }
            for reason in REQUIRED_SAFETY_REASONS
        }
        completed_case_count = len(terminal_states)
        required_safety_failed_case_count = sum(
            bool(state.safety_reasons) for state in terminal_states
        )
        return {
            "case_count": len(states),
            "completed_case_count": completed_case_count,
            "coverage_complete": completed_case_count == len(states),
            "status": (
                "PASS"
                if completed_case_count == len(states)
                and required_safety_failed_case_count == 0
                else "FAIL"
            ),
            "reference_complete_case_count": sum(
                state.terminal_success for state in terminal_states
            ),
            "tracking_lost_report_only_case_count": sum(
                state.first_tracking_lost_tick is not None for state in terminal_states
            ),
            "required_safety_failed_case_count": required_safety_failed_case_count,
            "reset_window_safety_failed_case_count": sum(
                bool(state.safety_reasons)
                and state.observed_motor_ticks
                <= self.reset_safety_window_motor_ticks
                for state in terminal_states
            ),
            "required_safety_event_counts": safety_counts,
            "required_safety_terminal_tick_counts": terminal_tick_counts,
            "first_required_safety_violation_by_reason": first_examples,
            "joint_violation_counts_by_reason_and_action_channel": {
                reason: {
                    channel_id: count
                    for channel_id, count in counts.items()
                    if count > 0
                }
                for reason, counts in joint_channel_counts.items()
            },
            "contact_violation_counts_by_reason_and_pair_id": {
                reason: {
                    pair_id: count
                    for pair_id, count in counts.items()
                    if count > 0
                }
                for reason, counts in contact_pair_counts.items()
            },
            "maximum_root_position_error_micrometres": maximum_root_error,
            "minimum_root_orientation_absolute_dot_q1_30": minimum_orientation,
            "maximum_joint_tracking_error_microradians_by_action_channel": (
                maximum_joint_error
            ),
            "contact_confusion_by_contact_channel": contact_counts,
            "contact_micro_precision": _ratio(
                true_positive, true_positive + false_positive
            ),
            "contact_micro_recall": _ratio(
                true_positive, true_positive + false_negative
            ),
        }


def _ratio(numerator: int, denominator: int) -> float | None:
    return None if denominator == 0 else numerator / denominator
