from __future__ import annotations

import html
from typing import Any

import numpy as np

from next_lab.motion_math import target_forward_kinematics
from next_lab.motion_retarget import CONTACT_IDS, RetargetedClip


def render_clip_preview(clip: RetargetedClip, descriptor: dict[str, Any]) -> str:
    width, height = 1600, 520
    frame_indices = _review_frames(len(clip.source_frames))
    panels: list[tuple[np.ndarray, np.ndarray, int]] = []
    for frame_index in frame_indices:
        target_positions, _ = target_forward_kinematics(
            descriptor,
            clip.root_position_um[frame_index].astype(np.float64) / 1_000_000.0,
            clip.root_quaternion_q1_30[frame_index].astype(np.float64) / float(1 << 30),
            clip.joint_position_urad[frame_index].astype(np.float64) / 1_000_000.0,
        )
        source_positions = clip.source_overlay_position_um[frame_index].astype(np.float64) / 1_000_000.0
        panels.append((target_positions, source_positions, frame_index))

    elements = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<rect width="100%" height="100%" fill="#0b1020"/>',
        '<style>text{font-family:ui-monospace,monospace;fill:#e5e7eb}.title{font-size:18px;font-weight:700}.small{font-size:11px}.tiny{font-size:9px}</style>',
        f'<text x="20" y="28" class="title">{html.escape(clip.clip_id)} · source/BodySchema overlay</text>',
        f'<text x="20" y="48" class="small">{html.escape(clip.skill)} · {html.escape(clip.split)}/{html.escape(clip.partition)} · cyan=CMU source · magenta=retarget · green=declared contact</text>',
    ]
    panel_width = 380
    for panel_index, (target, source, frame_index) in enumerate(panels):
        x0 = 20 + panel_index * 395
        y0 = 70
        projected = np.vstack((_project(target), _project(source)))
        minimum = np.min(projected, axis=0) - np.asarray((0.12, 0.08))
        maximum = np.max(projected, axis=0) + np.asarray((0.12, 0.08))
        scale = min((panel_width - 24) / (maximum[0] - minimum[0]), 390 / (maximum[1] - minimum[1]))

        def screen(points: np.ndarray) -> np.ndarray:
            values = _project(points)
            return np.column_stack(
                (
                    x0 + 12 + (values[:, 0] - minimum[0]) * scale,
                    y0 + 410 - (values[:, 1] - minimum[1]) * scale,
                )
            )

        target_screen = screen(target)
        source_screen = screen(source)
        ground_y = y0 + 410 - (0.0 - minimum[1]) * scale
        elements.extend(
            (
                f'<rect x="{x0}" y="{y0}" width="{panel_width}" height="420" rx="8" fill="#111827" stroke="#334155"/>',
                f'<line x1="{x0 + 6}" y1="{ground_y:.2f}" x2="{x0 + panel_width - 6}" y2="{ground_y:.2f}" stroke="#64748b" stroke-dasharray="5 4"/>',
                f'<text x="{x0 + 12}" y="{y0 + 20}" class="small">source frame {int(clip.source_frames[frame_index])} · correction {int(clip.ground_correction_um[frame_index])} um</text>',
            )
        )
        _append_target(elements, descriptor, target_screen)
        _append_source(elements, source_screen)
        active = [CONTACT_IDS[index].removeprefix("contact.") for index, value in enumerate(clip.contacts[frame_index]) if value]
        elements.append(
            f'<text x="{x0 + 12}" y="{y0 + 405}" class="tiny">contacts: {html.escape(", ".join(active) if active else "none")}</text>'
        )
    elements.append("</svg>")
    return "\n".join(elements) + "\n"


def _review_frames(frame_count: int) -> tuple[int, int, int, int]:
    return 0, (frame_count - 1) // 3, 2 * (frame_count - 1) // 3, frame_count - 1


def _project(points: np.ndarray) -> np.ndarray:
    return np.column_stack((points[:, 0] + 0.42 * points[:, 2], points[:, 1] + 0.16 * points[:, 2]))


def _append_target(elements: list[str], descriptor: dict[str, Any], points: np.ndarray) -> None:
    for body in descriptor["bodies"]:
        slot = int(body["body_slot"])
        parent = body["parent_body_slot"]
        if parent is not None:
            elements.append(
                f'<line x1="{points[int(parent), 0]:.2f}" y1="{points[int(parent), 1]:.2f}" x2="{points[slot, 0]:.2f}" y2="{points[slot, 1]:.2f}" stroke="#f472b6" stroke-width="3" opacity="0.85"/>'
            )
        elements.append(
            f'<circle cx="{points[slot, 0]:.2f}" cy="{points[slot, 1]:.2f}" r="3" fill="#f472b6"/>'
        )


def _append_source(elements: list[str], points: np.ndarray) -> None:
    index = {
        name: ordinal
        for ordinal, name in enumerate(
            (
                "lfemur",
                "ltibia",
                "lfoot",
                "rfemur",
                "rtibia",
                "rfoot",
                "lowerback",
                "thorax",
                "lhumerus",
                "lradius",
                "rhumerus",
                "rradius",
                "head",
            )
        )
    }
    edges = (
        ("lfemur", "ltibia"),
        ("ltibia", "lfoot"),
        ("rfemur", "rtibia"),
        ("rtibia", "rfoot"),
        ("lowerback", "thorax"),
        ("thorax", "head"),
        ("thorax", "lhumerus"),
        ("lhumerus", "lradius"),
        ("thorax", "rhumerus"),
        ("rhumerus", "rradius"),
    )
    for parent, child in edges:
        left, right = points[index[parent]], points[index[child]]
        elements.append(
            f'<line x1="{left[0]:.2f}" y1="{left[1]:.2f}" x2="{right[0]:.2f}" y2="{right[1]:.2f}" stroke="#22d3ee" stroke-width="2" opacity="0.75"/>'
        )
    for point in points:
        elements.append(f'<circle cx="{point[0]:.2f}" cy="{point[1]:.2f}" r="2" fill="#22d3ee"/>')
