# R8b body proportions and foot — task state

| Field | Value |
| --- | --- |
| Status | `BODY_V5_AXES_AND_PROXIES_IMPLEMENTED / DYNAMIC_POSTURE_OPEN` |
| Updated | 2026-09-05 |
| Scope | Improve actual human-like BodySchema, foot mechanics, mass/inertia and leaning; visualization alone is insufficient |
| Authority | Working context only; current SPEC/ADR and exact artifacts take precedence |

## Resume in 60 seconds

- Missing torso/head in the old origin-line plot caused the apparent
  leg/trunk disproportion. All 19 physical colliders are now drawn.
- Initial V4 measures 170 cm stature, 86.5 cm hip, 139.65 cm shoulder,
  40.8 cm thigh and 39.6 cm shank. Do not shorten legs to repair that picture.
- Opt-in physical BodySchema V5 is implemented under ADR-114. Existing
  environments/checkpoints still use V4; full goal remains active and no
  optimizer is running. Do not treat the V5 diagnostic as a learned fix.
- Independent all-1200-frame mass audit: total 75.337 kg, legs 38.923%,
  pelvis/torso/head 51.241%. Same-pose carrier relocation changes whole COM
  by at most 0.756 mm, not evidence against dynamic inertia/control effects.
- Final torso tilt 29.94 degrees versus pelvis 10.44 degrees. Walking reward
  sees pelvis orientation but not torso orientation; causal repair untested.
- V4 torso collider is 90.7 mm behind pelvis collider. V5 places pelvis
  collider Z at source COM -70.7 mm and torso collider local Z at rib-bounds
  midpoint +8.556 mm; the gap becomes 21.444 mm. Joint anchors/COM unchanged.
- V4 has reversed hip/shoulder/elbow flexion and hip-adduction directions.
  V5 reverses those eight axes only. Bilateral production native state import
  proves the corrected directions and preserved backwards knee flexion.
- Foot is still a rigid box. The successful gait shows toe-edge roll and a
  flat final stance; adding forefoot articulation is not yet implemented.
- Next: source-preserving principal-inertia candidate, then separately
  identified foot/torso-control improvements. Do not repeat the finished
  mass audit or axis-sign investigation. Old controller reference signs
  must be adapted before running V5 standing; old weights are incompatible.
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
4. [Reviewed mass/posture evidence and implemented V5 discriminator](../r8b-mass-and-posture-research-2026-09-05.md).
5. [ADR-114](../../architecture/adr/114-anatomical-axes-and-sagittal-body-proxies.md).

## Decision and remaining uncertainty

- Observation: same native frame with actual colliders has a complete trunk
  and head; the parent-origin diagram does not represent physical dimensions.
- Decision: correct shared diagnostic geometry and opt-in physical V5 axes /
  sagittal proxies; preserve V4 and the successful checkpoint as control.
  No new anatomical segment lengths or arbitrary mass redistribution.
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

- Localized Rust V5 body factory, diagnostic descriptor/example and native
  tests now implemented. No environment/default/runtime authority changes.
- Motor native library tests: 130 passed, including old profile regression,
  V5 axis directions and V2–V5 neutral stature/sole/non-overlap checks.
- External descriptor and comparison image are in
  `/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/body-v5-01`.
- Build with `NEXTENGINE_PHYSX_SDK_DIR=/home/kaifaty/.cache/nextengine/physx/sdk/f259d3da157cc6120b378b53ee14c10805be89698242b03d7417f699ca711c3b`.
  Global active SDK has a different build profile and is correctly rejected;
  do not change global active locator or weaken manifest validation.
- Initial previews and failed CLI artifact retained externally; use audit-04.
- Physical foot successor remains open, not completed by the rendering fix.
- Mass audit is independently `SUPPORTED_BOUNDED`, with no load-bearing
  arithmetic defect. Its entry-script hash omits helper hashes; independent
  recomputation closes this result only. Do not repeatedly rerun/re-review it.
- Remaining routed SPEC-14/27/28, root-document gaps and Rust skill reads
  are completed, including testing-and-quality and ADR-030. Relevant R8
  roadmap scope was read. Do not restart these full reads on each continuation.
