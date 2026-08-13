# Humanoid motor training rebuild — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE_R&D / TRAIN-4 / V19_PROTOTYPE_READY` |
| Updated | 2026-08-14 |
| Task key | `humanoid-motor-training-rebuild` |
| Scope | Close `REQ-HUM-DATA-005/007` dynamic-reference feasibility before any optimizer work |
| Definition of done | A hash-closed TRAIN-4 corpus/profile lineage passes the required optimizer-free gate and receives an evidence-backed decision |
| Authority | Working context only; Accepted SPEC/ADR, tracked profiles/manifests, exact gate artifacts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** The bounded causal cycle is complete. Build a small
  contact-consistent V19 reference/reset prototype before another full corpus
  identity.
- **Why:** R14 still fails `204/12518` cases. Controller lead, velocity
  feed-forward and zero-velocity variants churn or regress passing controls;
  correcting root-link/CoM velocity semantics alone worsens `204 -> 220`.
  Pointwise contact-mode and velocity-field inconsistency is strongly
  supported. Fresh-scene versus indexed partial reset remains open.
- **Next action:** Prototype explicit foot contact modes and solve final
  sticking-point pose/velocity consistency on three selected clips plus matched
  passing controls, paired under fresh-scene and indexed partial reset.
- **Current blocker:** Exact-zero required safety still fails, and ADR-070's
  fresh-scene reset contract conflicts with the current running-scene indexed
  reset implementation.
- **Do not retry:** Do not start PPO, build another broad whole-corpus
  ankle/retarget identity, zero reference velocities, add grace/settling, or
  loosen safety limits. The causal matrix rejects these as fixes.
- **Reconsider when:** The bounded prototype has no passing-control regression,
  strictly decreases every targeted failure class without a new category, and
  explicitly resolves fresh-scene versus partial-reset behavior.

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
| Formal visual review | `PENDING` | No visual acceptance claim |

The [detailed causal decision](../humanoid-train4-causal-research-2026-08-14.md)
is committed with the coherent roadmap and plan update. The hashes above
identify its external evidence.

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

- **Observation:** ADR-070 specifies a fresh scene; the vector environment
  performs indexed writes into a running scene and clears buffers.
- **Evidence:** Accepted-contract and implementation audit; no paired
  fresh-scene result exists yet.
- **Decision:** Run the same selected starts under fresh-scene and partial
  reset before a full V19 rebuild.
- **Rejected alternatives:** Assume buffer clearing proves equivalence, or
  change ADR-070 based only on general PhysX contact-cache behavior.
- **Consequences:** Divergent case outcomes or first-tick impulses stop the
  prototype and require the architecture workflow.
- **Uncertainty:** The practical effect of persistent contact state is open.
- **Reconsider when:** The paired experiment establishes equivalence or a
  concrete divergence with reproducible evidence.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: final contact mode, pose and velocity field are inconsistent | Mixed-point classifier, high declared support speeds, coupled ankle/sole failures and R17 sensitivity | No admissible contact-consistent PhysX run yet | Bounded pointwise contact prototype with matched passing controls |
| H2: indexed partial reset differs materially from a fresh scene | ADR/implementation conflict; PhysX keeps contact manifolds and caches | No paired project-specific result | Fresh-scene versus indexed-partial reset on identical selected starts |
| H3: root-link/CoM mismatch contributes but is not primary | Direct API mismatch and counterfactual sensitivity | Correct writer alone worsens total failures | Keep correct root-link semantics inside the contact-consistent prototype |

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

1. Use `cmu05-walk-validation`, `cmu16-walk-nominal-b` and
   `cmu139-walk-heldout` plus start-phase-matched passing controls.
2. Freeze prototype tolerances before PhysX results: `2 mm/frame` tangential,
   `1 mm/frame` normal and `5 mm` maximum normal residual at 60 Hz.
3. Require pointwise contact mode, final `h(q)=0`/`J(q)v=0` consistency and
   root-link velocity semantics; forbid zero velocities, grace, settling and
   looser safety bounds.
4. Pair selected starts under fresh-scene and indexed partial reset.
5. Permit a full V19 build only with no passing-control regression, strict
   decrease in each targeted failure class, no new required-safety category
   and an explicit ADR-070 disposition.

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

- **Workspace state:** The parallel roadmap worker's causal decision, plans and
  roadmap are committed at `b967c05`. This task-state is a follow-on resume
  surface; re-read those authorities before acting.
- **Checks:** This snapshot records documentation/evidence only; executable
  training checks are `NOT_RUN(NoExecutableChange)`.
- **Remaining risk:** The contact-consistent prototype and paired reset
  discriminator have not run; future evidence can make this snapshot stale
  until its next material-transition update.
- **Promotion needed:** Resolve any reset-semantic change through ADR/SPEC and
  routing updates; update roadmap facts only through the normal roadmap
  workflow.
