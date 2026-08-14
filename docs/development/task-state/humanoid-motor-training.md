# Humanoid motor training rebuild — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE_R&D / TRAIN-4 / CLIP_GLOBAL_PROTOTYPE` |
| Updated | 2026-08-14 |
| Task key | `humanoid-motor-training-rebuild` |
| Scope | Close `REQ-HUM-DATA-005/007` dynamic-reference feasibility before any optimizer work |
| Definition of done | A hash-closed TRAIN-4 corpus/profile lineage passes the required optimizer-free gate and receives an evidence-backed decision |
| Authority | Working context only; Accepted SPEC/ADR, tracked profiles/manifests, exact gate artifacts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R57 closes the projection domain: three selected
  clips are solved once, all `17/17` cases are exact slices and 30 overlap
  pairs have zero disagreement. V7 is rejected as a complete-clip solver.
- **Why:** R57 complete clips pass `0/3`; selected slices pass `16/17`.
  Contact-edge velocity semantics remove only the central-difference symptom.
  Cross-boundary swing smoothing and sequential collider/root closure leave a
  coupled contact/collider/velocity conflict.
- **Next action:** Prototype one deterministic coupled trajectory solve on
  complete `cmu05`, then apply it unchanged to `cmu16` and `cmu139`.
- **Current blocker:** R56 keeps contact and joint reserve valid but stagnates
  with `27` flight collider deficits up to `30063 µm` after 30 alternating
  iterations; another ordered post-pass is not justified.
- **Do not retry:** Do not start PPO, build another broad whole-corpus
  ankle/retarget identity, zero reference velocities, add grace/settling, or
  loosen safety limits. The causal matrix rejects these as fixes.
- **Reconsider when:** One clip-global trajectory passes exact slices,
  complete-clip invariants and the same fresh all-17 safety matrix.

All TRAIN-5 checkpoints remain rejected. No optimizer run, multi-seed run,
TRAIN-5 Advance or TRAIN-6 work is authorized. Formal visual review remains
pending. This file cannot change those facts by itself.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| R14 source audit, SHA-256 `ded76473905f7f26e4db0dfaa236508f649d46a692ff28938203f92dcf1c9b61` | `FAIL / 204 of 12518 cases` | TRAIN-4 stays open: `121` ROM, `67` impact and `40` overlapping joint-safety/velocity cases |
| R15 controller/reset matrix, SHA-256 `a49f84b98e8b642933492a8eadafbb8b0d915d09777e6f0ea272978231ec4a7a` | Baseline reproduced `12518/12518`; interventions did not close without churn | Reject target lead, `D/K*qdot` feed-forward and zero-velocity fixes |
| R16 root-link counterfactual, SHA-256 `9ae54fadeb63b286b3d46a8d529e4488dcfeba2d71307c8e6c2f6ec21fa6c97a` | Root-link semantics corrected; failures `204 -> 220` | Confirmed defect, insufficient cause |
| R17 contact-projected reset, SHA-256 `60ce03b6fbc2f213d82c5f95b0f4e831ca81dd50b52af1750890d44566cfc25c` | Failures `204 -> 194` with `152` recovered and `142` regressed | Supports contact coupling; approximation is not a fix |
| R18 bounded offline prototype, canonical identity `794e479b61c5b3041b75f86095647c1663f127763a8eddd9f3292cee0fa524ae` | `17/17` artifacts close active point pose/velocity invariants | Permits the PhysX discriminator only; does not prove collider clearance |
| R27 complete PhysX probe, SHA-256 `fabfef54d01ac421778fac05d9aa2a1bb062803c61f8e335ed191ed81e249e14` | `FAIL`: fresh `3/17`, partial `4/17`, maximum impulse delta `252409 µN·s` | Reject R18 expansion; retain fresh-scene authority and add collider closure |
| R34 V4 fresh discriminator, file SHA-256 `5a2d34a586a9619b9b69a1e590751e535717abe1bad8379c16c2417c366da626` | `PASS 4/4`; required-safety and control regressions are zero | Permits the ordered all-17 offline prototype only |
| R39 V5 all-17 fresh, file SHA-256 `70555aa69b736c825e9085fab726f6533e29184f67c44144cab77cbe8c286299` | `FAIL 1/17`: delayed unsupported landing causes `cmu16@249` hard impact | Authorize clearance only with active support |
| R45 V6 all-17 fresh, file SHA-256 `ba60b124b8b9e4c8df024edff92b949d9dd68f213d02c0e675277d285f09aa40` | `FAIL 1/17`: one-microradian edit selects `cmu16@415` hard ROM | Preserve sub-micrometre admissible source clearance |
| R47 V7 all-17 offline, canonical SHA-256 `1e56a2d3d14c8d3a8291639da49d4fda46263aa37c1d6682205343d4043202bb` | `PASS 17/17`; max joint reserve `2464/2500`, analytic normal `466 µm/frame`, collider `-2 µm` | Permits authoritative all-17 fresh probe |
| R49 V7 all-17 fresh, file SHA-256 `6b977a870c50b25545c2b73bc371575b38b40f6a914d1638ab19a3c7a56b5f0c` | `PASS 17/17`; required safety `0`; controls `0`; optimizer/training `0` | Permits only one clip-global prototype |
| R57 V7 clip-global, file SHA-256 `59fb91c9e19e22dde5caa017724a26e643cd889239a789333e3ad7b25b866271` | Domain `PASS`: one solve/clip, exact slices `17/17`, overlap disagreement `0`; solver `FAIL`: complete clips `0/3`, selected slices `16/17` | Retain clip-global path; replace the sequential solver before any fresh probe |
| Formal visual review | `PENDING` | No visual acceptance claim |

The [initial causal decision](../humanoid-train4-causal-research-2026-08-14.md)
and [bounded prototype decision](../humanoid-train4-v19-prototype-research-2026-08-14.md)
and [contact-boundary decision](../humanoid-train4-contact-boundary-research-2026-08-14.md)
and [support-authorization decision](../humanoid-train4-support-authorization-research-2026-08-14.md)
and [clip-global decision](../humanoid-train4-clip-global-research-2026-08-14.md)
carry detailed evidence. The hashes above identify their external reports.

## Decisions that still constrain the work

### D-001 — Keep TRAIN-4 optimizer-free

- **Observation:** Full-schedule R14 remains non-zero and every TRAIN-5 result
  descends from the invalidated upstream admission.
- **Evidence:** Exact R14 source audit and case-level R15-R17 reproduction.
- **Decision:** Keep TRAIN-5/6 inactive and reject optimizer-based remediation.
- **Rejected alternatives:** Resume rejected checkpoints, tune PPO/reward, or
  reinterpret improved failure counts as gate admission.
- **Consequences:** Only bounded optimizer-free reference/reset/contact work is
  allowed.
- **Uncertainty:** Whether the selected prototype is sufficient for full-corpus
  exact-zero closure remains unknown.
- **Reconsider when:** A new immutable lineage passes the full optimizer-free
  gate and its decision report explicitly advances TRAIN-4.

### D-002 — Target contact-consistent reference construction

- **Observation:** Declared contact can combine height and speed from different
  foot points, admits high support speeds, and does not ensure the final
  velocity field satisfies the same sticking constraint as the final pose.
- **Evidence:** Source audit, offline contact-trajectory correlation, failure
  clustering, and R17's large recovered/regressed response.
- **Decision:** Prototype explicit `Flight`, `HeelSticking`,
  `ForefootSticking` and `FlatSticking` modes; require one physical point to
  satisfy height and speed; solve `h(q)=0` and `J(q)v=0` together.
- **Rejected alternatives:** Another local ankle reserve/smoothing change,
  controller lead/feed-forward, zero velocities, or broad bundled remediation.
- **Consequences:** Recompute root and joint velocities after final pose/root
  correction and cross-check analytic point velocity against emitted finite
  differences.
- **Uncertainty:** Contact-consistent construction is strongly supported but
  not yet proven sufficient under PhysX.
- **Reconsider when:** A smaller intervention closes the same bounded cases
  without regressing matched controls and preserves the frozen constraints.

### D-003 — Discriminate reset semantics before full V19

- **Observation:** R27 finds eight cases above the frozen first-tick impulse
  delta and a fresh-pass/partial-hard-ROM outcome divergence at `cmu16@415`.
- **Evidence:** Complete R27 report from clean commit `3e7c54a`, exact report
  SHA-256 recorded above.
- **Decision:** Retain ADR-070 unchanged. Fresh-scene execution is acceptance
  authority; indexed running-scene reset is diagnostic-only and cannot feed an
  optimizer or admission result.
- **Rejected alternatives:** Treat buffer clearing as fresh reset, loosen the
  impulse tolerance, or add settling/grace.
- **Consequences:** Full-corpus evaluation must use fresh scenes. A future
  vector trainer must genuinely replace a slot scene before TRAIN-5.
- **Uncertainty:** The exact hidden solver/contact state responsible for each
  impulse delta is not required to reject the non-equivalent implementation.
- **Reconsider when:** A new implementation constructs a fresh scene per reset
  and independently proves its contract.

### D-004 — Close full collider geometry, not active points alone

- **Observation:** The R18 projector applies root correction, leaves joint
  correction at zero and validates only active heel/forefoot points.
- **Evidence:** Exact offline FK places the right swing-foot collider at
  `-14905`, `-11639` and `-3544 µm` for the three tick-1 `cmu139` failures;
  PhysX names that same body in every first violation.
- **Decision:** Add the existing corpus invariant `minimum collider height >=
  -2 µm` to a new bounded identity and solve conflicts with temporally coupled
  leg/root degrees of freedom.
- **Rejected alternatives:** Global root lift, impact-limit tuning or another
  root-only contact projection.
- **Consequences:** Recompute final kinematics and velocities after the solve;
  preserve hard ROM/reserve and active-point bounds exactly.
- **Uncertainty:** Whether the chosen bounded degrees of freedom avoid new
  transition/ROM regressions.
- **Reconsider when:** The four-case discriminator fails without an admissible
  correction or needs a materially different contact-mode model.

### D-005 — Close the post-smoothing contact boundary

- **Observation:** R35 fails five mixed-support windows at the last active
  frame, while a parameter-only sweep closes none of the six failures.
- **Evidence:** R35 plus the dated contact-boundary research report.
- **Decision:** V5 final contact reprojection, second collider floor and
  upward-only root-velocity closure with eight smoothing passes are retained.
- **Rejected alternatives:** Another scalar sweep or looser safety bounds.
- **Consequence:** The boundary is closed offline; later V7 refinements govern
  unsupported frames and cannot feed an optimizer.
- **Reconsider when:** A clip-global solve violates an active boundary.

### D-006 — Require one clip-global value per source frame

- **Observation:** Window-local candidate outputs disagree on overlapping
  source frames. R57 now proves exact slices and zero overlap disagreement, but
  all three complete V7 trajectories fail unchanged bounds.
- **Decision:** Retain R57's one-solve/exact-slice path and replace its solver;
  domain identity alone does not authorize a full V19 corpus identity.
- **Rejected alternatives:** Pick one overlay, normalize disagreement away or
  treat bounded reset evidence as full-corpus feasibility.
- **Reconsider when:** One deterministic clip solve passes exact overlap,
  selected-window and complete-clip checks together.

### D-007 — Authorize swing clearance only from active support

- **Observation:** V5's unsupported `5 mm` target delayed `cmu16@249` contact;
  V6's sub-micrometre correction changed `cmu16@415` by one microradian.
- **Decision:** Use `20 mm` clearance only with same-frame active support;
  unsupported frames target zero with a validated `1 µm` quantization deadband.
- **Consequence:** V7 preserves exact V1 bytes for both all-flight controls and
  R49 passes all 17 fresh scenes without changing any safety bound.
- **Reconsider when:** A clip-global trajectory provides a stronger physical
  landing model that passes the same exact evidence.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: support-authorized quantized clearance fixes the bounded matrix | R47 offline and R49 fresh both pass `17/17` | Bounded windows are not a clip-global trajectory | `CONFIRMED_BOUNDED`; do not extrapolate |
| H2: indexed partial reset differs materially from a fresh scene | R27 has eight impulse-bound failures and one outcome divergence | Most case outcomes still agree | `CONFIRMED`; reject partial as acceptance evidence |
| H3: window-local success extrapolates to one corpus trajectory | R49 passes the selected windows | Overlap disagreement and global `cmu16` failure directly contradict it | `REJECTED`; require clip-global construction after bounded V7 |
| H4: contact-edge stencil alone closes complete clips | It reduces analytic tangent to `1916` on `cmu05` | Normal, joint, root and `cmu139` failures remain | `REJECTED`; keep explicit hybrid semantics inside a coupled solve |
| H5: ordered masking/projection can close the coupled set | R56 makes contact pass with joint reserve `2500` | `27` collider deficits remain after 30 iterations | `REJECTED`; prototype a trajectory-level constraint solve |

## Required context

Read these sources in precedence order before acting:

1. [Agent routing](../../architecture/agent-routing.md), especially the
   deterministic humanoid training, motor and roadmap rows.
2. [SPEC-35](../../architecture/35-deterministic-humanoid-training-substrate.md),
   [SPEC-34](../../architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md),
   [ADR-069](../../architecture/adr/069-biomechanics-body-schema-v2-and-solver-projection.md)
   and [ADR-070](../../architecture/adr/070-biomechanics-reference-tracking-training-environment.md).
3. [Current roadmap](../../roadmap.md), whose current WIP and stage/blocker
   facts outrank the implementation plan summary.
4. [TRAIN-0..9 implementation plan](../../plans/2026-08-12-humanoid-motor-training-rebuild.md)
   for stage contracts and historical evidence.
5. [Current reference-training contract](../../../.agents/skills/nextengine-training-runner/references/current-contract.md)
   and [diagnostic playbook](../../../.agents/skills/nextengine-training-diagnostics/references/diagnostic-playbook.md).
6. The exact R14-R17 reports in the external training store and the
   [causal research report](../humanoid-train4-causal-research-2026-08-14.md).

Use `nextengine-training-runner` before preparing any run and
`nextengine-training-diagnostics` for causal analysis. Use
`nextengine-isaac-correspondence` when a hypothesis depends on Isaac/CPU mirror
semantics.

## Next action

1. Freeze V7/R49 and R57; neither is corpus admission.
2. Implement the smallest coupled complete-clip solver on `cmu05`.
3. Require unchanged contact, collider, ROM, joint and root bounds together.
4. Apply one identity to all three clips, exact slices and fresh all-17 safety.
5. Keep full V19, visual/exhaustive gates and optimization blocked until then.

## Do not retry

- Rejected TRAIN-5 checkpoints or any optimizer run — upstream TRAIN-4 remains
  failed; reconsider only after a new evidence-backed TRAIN-4 Advance.
- Target lead, `D/K*qdot` feed-forward or zero reference/reset velocities —
  case-level counterfactuals churn controls, regress categories or invalidate
  full-reference initialization.
- Broad ankle/root/retarget/contact changes — repeated bundled adjustments did
  not establish first cause; reconsider only after the bounded prototype.
- Grace windows, uncounted settling or looser impact/ROM/velocity limits — they
  weaken the frozen criterion rather than fix the reference/reset state.
- Treating reduced failure counts as success — the gate requires exactly zero
  required-safety events over complete phase coverage.

## Handoff

- **Workspace state:** R57 belongs to clean commit `cb3fcae`; generated reports
  stay under the external TRAIN-4 evaluation root.
- **Checks:** focused builder/contact tests `12/12`; R57 exact slice and overlap
  identity pass, complete-clip solver fails; optimizer and training remain zero.
- **Remaining risk:** coupled complete-clip closure, fresh slice safety,
  full-corpus exact-zero coverage and visual review remain open.
- **Promotion needed:** None for reset semantics: ADR-070 is retained. Any
  future attempt to admit indexed running-scene reset requires a superseding
  ADR and new evidence.
