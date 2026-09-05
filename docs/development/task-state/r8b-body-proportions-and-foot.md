# R8b body proportions and foot — task state

| Field | Value |
| --- | --- |
| Status | `VISUALIZATION_FIXED / FOOT_SUCCESSOR_OPEN` |
| Updated | 2026-09-05 |
| Scope | Inspect apparent leg/trunk disproportion; improve diagnostic geometry and assess foot successor |
| Authority | Working context only; current SPEC/ADR and exact artifacts take precedence |

## Resume in 60 seconds

- Missing torso/head in the old origin-line plot caused the apparent
  leg/trunk disproportion. All 19 physical colliders are now drawn.
- Initial V4 measures 170 cm stature, 86.5 cm hip, 139.65 cm shoulder,
  40.8 cm thigh and 39.6 cm shank. Do not shorten legs to repair that picture.
- Current physical body, environment, checkpoint and safety remain unchanged.
- Foot is still a rigid box. The successful gait shows toe-edge roll and a
  flat final stance; adding forefoot articulation is not yet implemented.
- Next: bounded separately identified foot experiment, with passive/driven
  choice and physically justified parameters before any new training.

## Required context

1. [Routing](../../architecture/agent-routing.md), current body/motor rows if
   changing physics; read their full SPEC/ADR set before that next change.
2. [SPEC-35](../../architecture/35-deterministic-humanoid-training-substrate.md),
   [ADR-069](../../architecture/adr/069-biomechanics-body-schema-v2-and-solver-projection.md),
   [ADR-102](../../architecture/adr/102-biomechanics-neutral-self-clearance-successor.md),
   [ADR-106](../../architecture/adr/106-walking-reference-and-leg-clearance-audit.md).
3. [Current review/evidence](../r8b-body-proportions-and-foot-review-2026-09-05.md)
   and [preserved walking result](../r8b-known-candidate-reuse-2026-09-05.md).

## Decision and remaining uncertainty

- Observation: same native frame with actual colliders has a complete trunk
  and head; the parent-origin diagram does not represent physical dimensions.
- Decision: correct shared diagnostic geometry, preserve V4 and the known
  successful checkpoint as a control. No new anatomical proportions invented.
- Rejected: shortening legs from the stick picture; compulsory flat feet in
  every gait phase; changing body bytes under the old identity.
- Remaining: benefit of forefoot articulation versus contact-shape/parameter
  improvements. The current report cannot distinguish these physical options.
- Reconsider proportions only on landmark/collider/source measurements, not
  a learned pose or a skeleton missing geometry.
- Success oracle for physical successor: loaded flat support, controlled
  heel rise, release/re-contact and bilateral symmetry, original safety;
  later learned quality needs a new compatible generation.

## Handoff

- Localized Python diagnostic change only, no new runtime authority.
- Focused tests and visual verification are recorded in the linked report.
- Initial previews and failed CLI artifact retained externally; use audit-04.
- Physical foot successor remains open, not completed by the rendering fix.
