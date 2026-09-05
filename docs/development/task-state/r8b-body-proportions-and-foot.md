# R8b body proportions and foot — task state

| Field | Value |
| --- | --- |
| Status | `BODY_V6_FULL_INERTIA_IMPLEMENTED / UPRIGHT_BALANCE_AND_FEET_OPEN` |
| Updated | 2026-09-05 |
| Scope | Improve actual human-like BodySchema, foot mechanics, mass/inertia and leaning; visualization alone is insufficient |
| Authority | Working context only; current SPEC/ADR and exact artifacts take precedence |

## Resume in 60 seconds

- Missing torso/head in the old origin-line plot caused the apparent
  leg/trunk disproportion. All 19 physical colliders are now drawn.
- Initial V4 measures 170 cm stature, 86.5 cm hip, 139.65 cm shoulder,
  40.8 cm thigh and 39.6 cm shank. Do not shorten legs to repair that picture.
- Opt-in physical BodySchema V5/V6 is implemented under ADR-114/115. Existing
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
- V6 now carries full principal moments / frames for all six non-diagonal
  tensors. Integer and compiled-float reconstruction error <= 1 micro kg m²
  per component; old masses/COM/geometry and V5 descriptor bytes unchanged.
- Native 30-second procedural standing completes on V5 and V6. Final torso
  tilt is 7.365° / 8.078° respectively: inertia repair does NOT fix upright
  posture. The unchanged reference drives only knees/ankles (unchanged axes),
  so it is usable for this neutral control; old learned weights remain
  incompatible. No new training environment is selected.
- Next: upright reference/balance and foot mechanics, then a separately
  identified learning environment. Do not repeat the finished mass audit,
  axis-sign investigation or diagonalization to tune the leaning symptom.
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
6. [V6 implementation and standing discriminator](../r8b-principal-inertia-and-standing-2026-09-05.md)
   and [ADR-115](../../architecture/adr/115-full-principal-inertia-body-successor.md).

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
- Historical V4/V5 solver tensor loss is large in some rotation directions
  (foot ~24%, forearm-hand ~42% relative quadratic-form error). It is not a
  measured gait error. Full projection is implemented in V6, preserving
  source masses/COM. Its first compile exposed V3 physics descriptor's
  erroneous V1 exact-norm check for principal frames. ADR-115 isolates a
  bounded mass-frame validator; initial/shape poses keep their old rules.
  Native creation/motion and compiled payload reconstruction pass. No direct
  SDK mass-property getter was added; the existing setCMassLocalPose /
  setMassSpaceInertiaTensor path was inspected in pinned 5.9.0 source.
- Source toes have a non-realizable diagonal inertia; do not copy it into a
  new dynamic MTP link. Current merged foot tensors pass that elementary
  check. Source 2023 correction/proxy derivation remains to be examined.
- Success oracle for physical successor: loaded flat support, controlled
  heel rise, release/re-contact and bilateral symmetry, original safety;
  later learned quality needs a new compatible generation.

## Handoff

- Rust V6 body and mass-frame descriptor repair, inspection/standing examples
  implemented. No environment/default/runtime authority changes.
- Motor native library tests: 132 passed, including old profile regression,
  V5/V6 axis directions, V2–V6 neutral geometry and full tensor reconstruction.
- `play`, `persistence-replay`, `content-package`, format and links pass.
  Full host-check is still running (tool session `71889`, own xtask PID
  `2571258`, verification child `2587405` at last check). Workspace clippy
  passed; do not call the full check passed until its actual completion.
  Another host-check in `/home/kaifaty/Documents/NextEngine` is unrelated;
  do not stop or restart it. Resume the existing own handle, not a new check.
- External descriptor and comparison image are in
  `/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/body-v5-01`.
- V6 descriptor and both 30-second standing outputs are in sibling
  `body-v6-01`; exact hashes are in the V6 report. Probe records link poses
  each tick, not joint targets; extend telemetry when diagnosing the controller.
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
