from __future__ import annotations

import argparse
import html
import json
from pathlib import Path
from typing import Any, Callable

from next_lab.motor_mirror import validate_biomechanics_descriptor
from next_lab.safety_contact_mirror import validate_safety_contact_golden


def render_native_standing_review(
    descriptor: dict[str, Any], review: dict[str, Any]
) -> str:
    validate_biomechanics_descriptor(descriptor)
    _validate_native_review(descriptor, review)
    width, height = 1800, 790
    samples = review["samples"]
    bodies = descriptor["bodies"]
    all_positions = [
        tuple(value / 1_000_000.0 for value in link["position_micrometres"])
        for sample in samples
        for link in sample["links"]
    ]
    bounds = {
        "front": _bounds(all_positions, 0, 1),
        "side": _bounds(all_positions, 2, 1),
    }
    elements = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<rect width="100%" height="100%" fill="#0b1020"/>',
        '<style>text{font-family:ui-monospace,monospace;fill:#e5e7eb}.title{font-size:20px;font-weight:700}.small{font-size:11px}.panel{font-size:16px;font-weight:700}.metric{font-size:10px}</style>',
        '<text x="24" y="32" class="title">TRAIN-3 native PhysX procedural standing · actual committed snapshots</text>',
        f'<text x="24" y="52" class="small">BodySchema {html.escape(review["body_schema_hash"])} · safety/contact {html.escape(review["safety_contact_profile_sha256"])}</text>',
    ]
    colors = ("#a78bfa", "#22d3ee", "#22c55e", "#f59e0b")
    for index, sample in enumerate(samples):
        x0 = 20 + index * 445
        elements.extend(
            (
                f'<rect x="{x0}" y="70" width="420" height="680" rx="10" fill="#111827" stroke="#334155"/>',
                f'<text x="{x0 + 16}" y="98" class="panel">t={sample["simulated_seconds"]}s · tick {sample["motor_tick"]}</text>',
            )
        )
        positions = [
            tuple(value / 1_000_000.0 for value in link["position_micrometres"])
            for link in sample["links"]
        ]
        for row, (name, horizontal) in enumerate((("front X/Y", 0), ("side Z/Y", 2))):
            panel_top = 116 + row * 285
            elements.append(
                f'<text x="{x0 + 16}" y="{panel_top + 17}" class="small">{name}</text>'
            )
            project = _projector(
                x0 + 12,
                panel_top + 23,
                396,
                250,
                bounds["front" if horizontal == 0 else "side"],
            )
            ground_y = project((0.0, 0.0))[1]
            elements.append(
                f'<line x1="{x0 + 12}" y1="{ground_y:.2f}" x2="{x0 + 408}" y2="{ground_y:.2f}" stroke="#64748b" stroke-dasharray="5 4"/>'
            )
            _append_actual_skeleton(
                elements,
                bodies,
                positions,
                horizontal,
                project,
                colors[index],
            )
        root = sample["links"][0]
        root_position = root["position_micrometres"]
        hard_violations, minimum_margin, maximum_velocity_ratio = _joint_metrics(
            descriptor, sample
        )
        nonsole = sum(
            contact["primary_role"] != 8 for contact in sample["classified_contacts"]
        )
        support = sum(
            contact["class"] == 1 for contact in sample["classified_contacts"]
        )
        disposition = "Running"
        if sample["terminal"] is not None:
            disposition = f'{sample["terminal"]["reason_id"]} / Truncated'
        lines = (
            f'root XYZ = {root_position[0] / 1e6:+.3f}, {root_position[1] / 1e6:.3f}, {root_position[2] / 1e6:+.3f} m',
            f'hard-ROM violations = {hard_violations} · min margin = {minimum_margin / 1e6:.3f} rad',
            f'max velocity / bound = {maximum_velocity_ratio:.3f}',
            f'sole samples = {support} · non-sole classified = {nonsole}',
            f'episode = {disposition}',
        )
        for line_index, line in enumerate(lines):
            elements.append(
                f'<text x="{x0 + 16}" y="{707 + line_index * 12}" class="metric">{html.escape(line)}</text>'
            )
    elements.extend(
        (
            '<text x="24" y="778" class="small">Fixed scale across time · body positions come from native PhysX snapshots · ground contacts at review samples are sole-only.</text>',
            "</svg>",
        )
    )
    return "\n".join(elements) + "\n"


def render_contact_semantics_review(
    descriptor: dict[str, Any], golden: dict[str, Any]
) -> str:
    validate_biomechanics_descriptor(descriptor)
    validate_safety_contact_golden(golden, descriptor)
    width, height = 1800, 680
    elements = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<rect width="100%" height="100%" fill="#0b1020"/>',
        '<style>text{font-family:ui-monospace,monospace;fill:#e5e7eb}.title{font-size:20px;font-weight:700}.small{font-size:11px}.row{font-size:14px;font-weight:700}.cell{font-size:10px}</style>',
        '<text x="24" y="32" class="title">TRAIN-3 contact and terminal semantics · CPU golden / independent mirror</text>',
        f'<text x="24" y="52" class="small">profile {html.escape(golden["safety_contact_profile_sha256"])} · red terminal failures cannot be waived by reward</text>',
    ]
    class_colors = {
        1: ("#22d3ee", "SoleSupport"),
        2: ("#f59e0b", "TransientAllowed"),
        3: ("#ef4444", "Forbidden"),
        4: ("#3b82f6", "BraceSupport"),
        5: ("#a78bfa", "GetUpSupport"),
        6: ("#dc2626", "SelfCollision"),
    }
    for row, scenario in enumerate(golden["contact_terminal_scenarios"]):
        y0 = 76 + row * 79
        elements.append(
            f'<rect x="20" y="{y0}" width="1760" height="67" rx="8" fill="#111827" stroke="#334155"/>'
        )
        elements.append(
            f'<text x="36" y="{y0 + 24}" class="row">{html.escape(scenario["name"])}</text>'
        )
        profile_name = {1: "locomotion", 2: "brace/fall", 3: "get-up"}[
            scenario["skill_profile"]
        ]
        elements.append(
            f'<text x="36" y="{y0 + 45}" class="small">profile={profile_name}</text>'
        )
        cell_x = 410
        final_decision: dict[str, Any] | None = None
        for tick in scenario["ticks"]:
            final_decision = tick["expected_decision"]
            for substep, frame in enumerate(tick["expected_frames"]):
                contact_class = frame["contacts"][0]["class"] if frame["contacts"] else 0
                color, label = class_colors.get(contact_class, ("#475569", "NoContact"))
                elements.append(
                    f'<rect x="{cell_x}" y="{y0 + 13}" width="118" height="40" rx="5" fill="{color}" opacity="0.9"/>'
                )
                elements.append(
                    f'<text x="{cell_x + 6}" y="{y0 + 30}" class="cell">tick {tick["motor_tick"]}.{substep + 1}</text>'
                )
                elements.append(
                    f'<text x="{cell_x + 6}" y="{y0 + 45}" class="cell">{label}</text>'
                )
                cell_x += 124
        assert final_decision is not None
        reason = final_decision["reason_id"] or "Running"
        disposition = {0: "Running", 1: "Terminated", 2: "Truncated"}[
            final_decision["disposition"]
        ]
        decision_color = "#22c55e" if disposition == "Running" else "#f59e0b" if disposition == "Truncated" else "#ef4444"
        elements.append(
            f'<rect x="1500" y="{y0 + 13}" width="260" height="40" rx="5" fill="{decision_color}" opacity="0.9"/>'
        )
        elements.append(
            f'<text x="1510" y="{y0 + 30}" class="cell">{disposition}</text>'
        )
        elements.append(
            f'<text x="1510" y="{y0 + 45}" class="cell">{html.escape(reason)}</text>'
        )
    elements.extend(
        (
            '<text x="24" y="660" class="small">The fifth consecutive low-impulse hand sample becomes forbidden; get-up knee support remains allowed; hard impact and self-collision terminate immediately.</text>',
            "</svg>",
        )
    )
    return "\n".join(elements) + "\n"


def _validate_native_review(
    descriptor: dict[str, Any], review: dict[str, Any]
) -> None:
    if review.get("schema_version") != 1 or review.get("status") != "PASS":
        raise ValueError("native safety review did not pass")
    if review.get("body_schema_hash") != descriptor.get("body_schema_hash"):
        raise ValueError("native review BodySchema mismatch")
    if review.get("compiled_descriptor_hash") != descriptor.get(
        "compiled_descriptor_hash"
    ):
        raise ValueError("native review descriptor mismatch")
    samples = review.get("samples")
    if not isinstance(samples, list) or [sample.get("motor_tick") for sample in samples] != [
        0,
        600,
        1200,
        1800,
    ]:
        raise ValueError("native review sample closure mismatch")
    if samples[-1]["terminal"]["reason_id"] != "terminal.timeout":
        raise ValueError("native review did not reach timeout")
    for sample in samples:
        if len(sample["links"]) != 24 or len(sample["joints"]) not in {0, 23}:
            raise ValueError("native review topology mismatch")
        if any(contact["primary_role"] != 8 for contact in sample["classified_contacts"]):
            raise ValueError("native standing contains non-sole contact")


def _bounds(
    positions: list[tuple[float, float, float]], horizontal: int, vertical: int
) -> tuple[float, float, float, float]:
    minimum_h = min(point[horizontal] for point in positions) - 0.16
    maximum_h = max(point[horizontal] for point in positions) + 0.16
    minimum_v = min(0.0, min(point[vertical] for point in positions) - 0.08)
    maximum_v = max(point[vertical] for point in positions) + 0.08
    return minimum_h, maximum_h, minimum_v, maximum_v


def _projector(
    x0: float,
    y0: float,
    width: float,
    height: float,
    bounds: tuple[float, float, float, float],
) -> Callable[[tuple[float, float]], tuple[float, float]]:
    minimum_h, maximum_h, minimum_v, maximum_v = bounds
    scale = min(width / (maximum_h - minimum_h), height / (maximum_v - minimum_v))

    def project(point: tuple[float, float]) -> tuple[float, float]:
        return (
            x0 + (point[0] - minimum_h) * scale,
            y0 + height - (point[1] - minimum_v) * scale,
        )

    return project


def _append_actual_skeleton(
    elements: list[str],
    bodies: list[dict[str, Any]],
    positions: list[tuple[float, float, float]],
    horizontal: int,
    project: Callable[[tuple[float, float]], tuple[float, float]],
    color: str,
) -> None:
    for slot, body in enumerate(bodies):
        parent = body["parent_body_slot"]
        point = positions[slot]
        x2, y2 = project((point[horizontal], point[1]))
        if parent is not None:
            parent_point = positions[parent]
            x1, y1 = project((parent_point[horizontal], parent_point[1]))
            elements.append(
                f'<line x1="{x1:.2f}" y1="{y1:.2f}" x2="{x2:.2f}" y2="{y2:.2f}" stroke="{color}" stroke-width="3"/>'
            )
        radius = 3 if body["non_colliding_carrier"] else 5
        fill = "#94a3b8" if body["non_colliding_carrier"] else color
        elements.append(
            f'<circle cx="{x2:.2f}" cy="{y2:.2f}" r="{radius}" fill="{fill}"/>'
        )


def _joint_metrics(
    descriptor: dict[str, Any], sample: dict[str, Any]
) -> tuple[int, int, float]:
    joint_by_ordinal = {
        int(joint["dof_ordinal"]): joint for joint in descriptor["joints"]
    }
    margins: list[int] = []
    ratios: list[float] = []
    violations = 0
    for state in sample["joints"]:
        joint = joint_by_ordinal[int(state["ordinal"])]
        position = int(state["position_microradians"])
        hard = joint["hard_limit_microradians"]
        violations += int(not hard[0] <= position <= hard[1])
        margins.append(min(position - hard[0], hard[1] - position))
        ratios.append(
            abs(int(state["velocity_microradians_per_second"]))
            / joint["maximum_velocity_microradians_per_second"]
        )
    return (
        violations,
        min(margins) if margins else 0,
        max(ratios) if ratios else 0.0,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("descriptor", type=Path)
    parser.add_argument("native_review", type=Path)
    parser.add_argument("safety_golden", type=Path)
    parser.add_argument("standing_output", type=Path)
    parser.add_argument("contacts_output", type=Path)
    arguments = parser.parse_args()
    descriptor = json.loads(arguments.descriptor.read_text(encoding="utf-8"))
    native_review = json.loads(arguments.native_review.read_text(encoding="utf-8"))
    safety_golden = json.loads(arguments.safety_golden.read_text(encoding="utf-8"))
    arguments.standing_output.write_text(
        render_native_standing_review(descriptor, native_review), encoding="utf-8"
    )
    arguments.contacts_output.write_text(
        render_contact_semantics_review(descriptor, safety_golden), encoding="utf-8"
    )


if __name__ == "__main__":
    main()
