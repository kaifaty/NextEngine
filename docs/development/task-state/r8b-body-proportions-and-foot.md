# R8b body proportions and foot — task state

| Field | Value |
| --- | --- |
| Status | `MASS_AUDITED / HUMAN_BODY_SUCCESSOR_OPEN` |
| Updated | 2026-09-05 |
| Scope | Improve actual human-like BodySchema, foot mechanics, mass/inertia and leaning; visualization alone is insufficient |
| Authority | Working context only; current SPEC/ADR and exact artifacts take precedence |

## Resume in 60 seconds

- Missing torso/head in the old origin-line plot caused the apparent
  leg/trunk disproportion. All 19 physical colliders are now drawn.
- Initial V4 measures 170 cm stature, 86.5 cm hip, 139.65 cm shoulder,
  40.8 cm thigh and 39.6 cm shank. Do not shorten legs to repair that picture.
- Current physical body, environment, checkpoint and safety remain unchanged.
  Full user goal remains active; no new optimizer is running.
- Independent all-1200-frame mass audit: total 75.337 kg, legs 38.923%,
  pelvis/torso/head 51.241%. Same-pose carrier relocation changes whole COM
  by at most 0.756 mm, not evidence against dynamic inertia/control effects.
- Final torso tilt 29.94 degrees versus pelvis 10.44 degrees. Walking reward
  sees pelvis orientation but not torso orientation; causal repair untested.
- User's neutral side view exposes sagittal geometry: torso collider centre
  is 90.7 mm behind pelvis collider centre; this exists before learning.
  Original source lumbar offset matches, so do not call it a compiler sign
  bug. Proxy geometry/all-zero posture still need validation and improvement.
- Foot is still a rigid box. The successful gait shows toe-edge roll and a
  flat final stance; adding forefoot articulation is not yet implemented.
- Next: source-preserving principal-inertia candidate and neutral sagittal
  geometry check, then separately identified foot/torso-control improvements.
  All runtime changes need new identities and native checks before training.

## Required context

1. [Routing](../../architecture/agent-routing.md), current body/motor rows if
   changing physics; read their full SPEC/ADR set before that next change.
2. [SPEC-35](../../architecture/35-deterministic-humanoid-training-substrate.md),
   [ADR-069](../../architecture/adr/069-biomechanics-body-schema-v2-and-solver-projection.md),
   [ADR-102](../../architecture/adr/102-biomechanics-neutral-self-clearance-successor.md),
   [ADR-106](../../architecture/adr/106-walking-reference-and-leg-clearance-audit.md).
3. [Current review/evidence](../r8b-body-proportions-and-foot-review-2026-09-05.md)
   and [preserved walking result](../r8b-known-candidate-reuse-2026-09-05.md).
4. [Reviewed mass/posture evidence and new neutral-offset discriminator](../r8b-mass-and-posture-research-2026-09-05.md).

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
- Exploratory current solver tensor loss is large in some rotation directions
  (foot ~24%, forearm-hand ~42% relative quadratic-form error). It is not a
  measured gait error. Check full principal-axis projection without changing
  source masses/COM before arbitrary mass redistribution. Old schemas remain
  immutable; full projection needs Q30 norm/error bounds and native readback.
- Source toes have a non-realizable diagonal inertia; do not copy it into a
  new dynamic MTP link. Current merged foot tensors pass that elementary
  check. Source 2023 correction/proxy derivation remains to be examined.
- Success oracle for physical successor: loaded flat support, controlled
  heel rise, release/re-contact and bilateral symmetry, original safety;
  later learned quality needs a new compatible generation.

## Handoff

- Localized Python diagnostic change only, no new runtime authority.
- Focused tests and visual verification are recorded in the linked report.
- Initial previews and failed CLI artifact retained externally; use audit-04.
- Physical foot successor remains open, not completed by the rendering fix.
- Mass audit is independently `SUPPORTED_BOUNDED`, with no load-bearing
  arithmetic defect. Its entry-script hash omits helper hashes; independent
  recomputation closes this result only. Do not repeatedly rerun/re-review it.
- Complete remaining routed full reads before physical/public semantic edits:
  SPEC-14/27/28 and relevant current roadmap stage; Rust skill reads were
  truncated and must be completed when Rust implementation begins. Roots
  SPEC-00/01, SPEC-26/35 and ADR-013/027/058/059/062–071/100–102/106 have
  been read in full in this investigation. README/glossary/route small gaps
  from truncated combined outputs still need direct completion, not a reset
  of the already reviewed experiment or all source history.
