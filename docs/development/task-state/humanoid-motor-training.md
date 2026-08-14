# Humanoid motor training rebuild — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE_R&D / TRAIN-4 / V19_PROTOTYPE_REJECTED` |
| Updated | 2026-08-14 |
| Task key | `humanoid-motor-training-rebuild` |
| Scope | Close `REQ-HUM-DATA-005/007` dynamic-reference feasibility before any optimizer work |
| Definition of done | A hash-closed TRAIN-4 corpus/profile lineage passes the required optimizer-free gate and receives an evidence-backed decision |
| Authority | Working context only; Accepted SPEC/ADR, tracked profiles/manifests, exact gate artifacts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R27 rejects the first bounded V19 projector. Build
  one collider-aware, temporally coupled root/leg-chain revision before any
  full corpus identity.
- **Why:** Complete R27 leaves `3/17` fresh-scene and `4/17` partial-reset
  failures, including passing-control regressions. All fresh failures are
  adjacent `cmu139` starts whose right swing-foot collider begins
  `3.544..14.905 mm` below ground. Partial reset also diverges in first-tick
  impulses and one case outcome.
- **Next action:** Freeze a new prototype identity that preserves the existing
  active-point bounds, requires every collider `>= -2 µm`, and solves stance
  contact plus swing clearance through bounded joint/root trajectory changes.
  Discriminate on `cmu139@626/627/630` and `cmu16@415`, then rerun all 17 only
  if controls do not regress.
- **Current blocker:** The root-only projector violates collider
  non-penetration; the indexed running-scene reset is proven non-equivalent and
  cannot serve as acceptance evidence under ADR-070.
- **Do not retry:** Do not start PPO, build another broad whole-corpus
  ankle/retarget identity, zero reference velocities, add grace/settling, or
  loosen safety limits. The causal matrix rejects these as fixes.
- **Reconsider when:** The collider-aware fresh-scene prototype has no
  passing-control regression and strictly decreases every targeted failure
  class without a new category.

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
| Formal visual review | `PENDING` | No visual acceptance claim |

The [initial causal decision](../humanoid-train4-causal-research-2026-08-14.md)
and [bounded prototype decision](../humanoid-train4-v19-prototype-research-2026-08-14.md)
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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: active-point projection misses swing/full-collider penetration | Exact FK and all three fresh failures identify the same penetrated right foot | Other selected clips can pass despite later reference penetration | Require all-collider clearance in the four-case discriminator |
| H2: indexed partial reset differs materially from a fresh scene | R27 has eight impulse-bound failures and one outcome divergence | Most case outcomes still agree | `CONFIRMED`; reject partial as acceptance evidence |
| H3: a coupled leg/root solve can retain stance while clearing swing geometry | Existing stance-chain solver substrate and localized collision mechanism | R18 used root only; no collider-aware revision exists | Bounded temporally coupled prototype, then fresh PhysX |

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

1. Create a new immutable bounded profile; do not modify or relabel R18.
2. Retain `2 mm/frame` tangential, `1 mm/frame` normal and `5 mm` active-point
   residual bounds; add all-collider minimum `-2 µm`.
3. Solve active stance plus inactive swing clearance with temporally coupled
   root/leg-chain corrections, preserving descriptor ROM/reserve.
4. Recompute final root/joint velocities and keep root-link semantics; forbid
   zero velocities, grace, settling and looser safety bounds.
5. Run fresh-scene `cmu139@626/627/630` plus `cmu16@415`; only then rerun all
   17. Partial reset remains labeled report-only.
6. Permit a full V19 build only with no passing-control regression, strict
   decrease in each targeted failure class and no new required-safety category.

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

- **Workspace state:** R27 evidence belongs to clean commit `3e7c54a`; generated
  reports stay under the external TRAIN-4 evaluation root.
- **Checks:** Focused contact/PhysX helper tests passed; R27 completed `17`
  fresh workers plus one partial worker and returned the expected bounded
  non-acceptance exit.
- **Remaining risk:** Collider-aware coupled correction is not implemented;
  exact-zero full coverage and visual review remain open.
- **Promotion needed:** None for reset semantics: ADR-070 is retained. Any
  future attempt to admit indexed running-scene reset requires a superseding
  ADR and new evidence.
