# Humanoid motor training rebuild — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE_R&D / TRAIN-4 / R114_COMPLETE / R115_KTO_EXECUTION_NEXT` |
| Updated | 2026-08-15 |
| Task key | `humanoid-motor-training-rebuild` |
| Scope | Close `REQ-HUM-DATA-005/007` dynamic-reference feasibility before learned optimizer work |
| Definition of done | A hash-closed TRAIN-4 corpus/profile lineage passes the required optimizer-free gate and receives an evidence-backed decision |
| Authority | Working context only; Accepted SPEC/ADR, tracked profiles/manifests, exact gate artifacts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** Clean R114 revision 2 freezes one bounded KTO execution
  over exact R113 identity without running a solver or scene.
- **Why:** All `69687` q/v/a variables, the continuous-V9 yaw branch, exact
  quantization, resource bounds and fail-closed gates now close.
- **Next action:** Hash-bind, validate and execute one R115 in-memory KTO solve;
  persist only its external transient warm-start cache if exact PASS occurs.
- **Current blocker:** KTO exact emitted feasibility with nonzero V7 progress
  has not been executed; fixed-PD dynamic feasibility remains downstream.
- **Do not retry:** Do not start PPO, substitute another scalar/vector/root/
  pose boundary, zero velocities, add grace/settling, tune controller, or
  loosen safety limits. The causal matrix rejects these as fixes.
- **Reconsider when:** R115 reports exact emitted PASS/FAIL under the R114
  budget, without a candidate artifact, scene, all-17 or learned optimization.

All TRAIN-5 checkpoints remain rejected. No learned optimizer run, multi-seed run,
TRAIN-5 Advance or TRAIN-6 work is authorized. Formal visual review remains
pending. This file cannot change those facts by itself.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| R14 source audit, SHA-256 `ded76473905f7f26e4db0dfaa236508f649d46a692ff28938203f92dcf1c9b61` | `FAIL / 204 of 12518 cases` | TRAIN-4 stays open: `121` ROM, `67` impact and `40` overlapping joint-safety/velocity cases |
| R27 complete PhysX probe, SHA-256 `fabfef54d01ac421778fac05d9aa2a1bb062803c61f8e335ed191ed81e249e14` | `FAIL`: fresh `3/17`, partial `4/17`, maximum impulse delta `252409 µN·s` | Reject R18 expansion; retain fresh-scene authority and add collider closure |
| R47 V7 all-17 offline, canonical SHA-256 `1e56a2d3d14c8d3a8291639da49d4fda46263aa37c1d6682205343d4043202bb` | `PASS 17/17`; max joint reserve `2464/2500`, analytic normal `466 µm/frame`, collider `-2 µm` | Permits authoritative all-17 fresh probe |
| R49 V7 all-17 fresh, file SHA-256 `6b977a870c50b25545c2b73bc371575b38b40f6a914d1638ab19a3c7a56b5f0c` | `PASS 17/17`; required safety `0`; controls `0`; optimizer/training `0` | Permits only one clip-global prototype |
| R57 V7 clip-global, file SHA-256 `59fb91c9e19e22dde5caa017724a26e643cd889239a789333e3ad7b25b866271` | Domain `PASS`: one solve/clip, exact slices `17/17`, overlap disagreement `0`; solver `FAIL`: complete clips `0/3`, selected slices `16/17` | Retain clip-global path; replace the sequential solver before any fresh probe |
| R69 coupled feasibility, report SHA-256 `4e840f9f9d91f4b13ffbda8f23ab33b2158f61ad87bcdd1e12c6932ae9606b56` | Complete `cmu05` passes contact, collider, ROM and root/joint velocity simultaneously | Accept the dimensionless sparse-QP mechanism; remove the R61 intermediate input |
| R73 clean V8 all-three, manifest SHA-256 `d0b3897545af22bfefa68e69562eb27e5bfc182b240e09baa12325d0ab31d37c` | `cmu05`/`cmu16` PASS; `cmu139` second QP primal infeasible after collider `-34056 µm` | Reject V8 as all-clip solver; keep fresh PhysX blocked |
| R75/R76 bounded-step counterfactuals | Twelve feasible QPs, but collider/contact alternate; best final collider `-2732 µm`, residual `6296 µm` | Trust removes artificial infeasibility; blind acceptance remains invalid |
| R111–R114 model/KTO lineage | R113 `PASS`, canonical `3ac92ae2ca508234a52d77f0414ad5557f1164028e51a3938cc045ac4c5147cf`; R114 revision 2 `COMPLETE`, canonical `7a735320509a303f9feacba79087f2042d451d526b57585d4fe4041603b46ae4`, 5/5 validations; solver-free R115 shape `69687×131180`, `426210` nonzeros | Permit one bounded R115 KTO execution only; no scene |
| Formal visual review | `PENDING` | No visual acceptance claim |

R109/R110-v2/R111/R112/R113/R114-v2 canonical SHA-256: `2867aecd144d7996d3bf5bd0b6498dc1a5d480f7b8060106a97c5c6fc07da784` / `83408b97b6ba13dc801b4d9b4f68e55146f9f39b09a451c238b24cc4c8c7d88d` / `eafc8fc7f5bc64706b53c313cff143e0c0f8bd7684371e714d94bdef3e86f058` / `dfb3bd892b04023054ce947127743e7ada78b40000a93e22887101c544c493f2` / `3ac92ae2ca508234a52d77f0414ad5557f1164028e51a3938cc045ac4c5147cf` / `7a735320509a303f9feacba79087f2042d451d526b57585d4fe4041603b46ae4`.
The [initial causal decision](../humanoid-train4-causal-research-2026-08-14.md)
and [bounded prototype decision](../humanoid-train4-v19-prototype-research-2026-08-14.md)
and [contact-boundary decision](../humanoid-train4-contact-boundary-research-2026-08-14.md)
and [support-authorization decision](../humanoid-train4-support-authorization-research-2026-08-14.md)
and [clip-global decision](../humanoid-train4-clip-global-research-2026-08-14.md)
and [coupled-solver decision](../humanoid-train4-coupled-trajectory-research-2026-08-14.md) plus the [native-dynamics decision](../humanoid-train4-native-dynamics-research-2026-08-14.md)
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

### D-008 — Globalize the dimensionless coupled constraint solve

- **Observation:** Sequential post-passes repeatedly repair one bound while
  breaking another; raw-variable least squares and line search are badly
  scaled and stagnate.
- **Evidence:** R69/R72 simultaneous `cmu05` PASS, R73 clean two-clip PASS,
  R77–R91 isolation, R93 offline PASS and R94–R97 native counterfactuals.
- **Decision:** V9 retains one complete-clip SQP over root XYZ and ten selected
  leg joints, adds stable rows only for the two foot boxes, and uses no
  root/joint post-projection. Every source contact point remains frozen;
  candidates require shared model/exact merit and explicit acceptance.
- **Rejected alternatives:** Point-entry semantics alone, blind post-scaling,
  dense category slack, mismatched row-L1/max merit, or looser safety limits.
- **Consequences:** NumPy/SciPy/OSQP are pinned private lab dependencies; final
  contact, collider, CoM, ROM and velocity facts are recomputed from emitted
  integer poses. The adapter has no runtime or corpus-admission authority.
- **Uncertainty:** Whether quantization-aware KTO can make nonzero tracking
  progress while the emitted complete clip remains exact-zero feasible.
- **Reconsider when:** The single R115 execution reports its exact emitted
  outcome and proves whether nonzero V7 progress is simultaneously feasible.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H22: V9 is kinematically safe but dynamically too demanding for fixed PD | R108 requires fixed-PD effort/point forces | R114-v2 closes KTO formulation, but dynamics remain untested | Run R115; formulate ID only after exact PASS |
| H23: offline clearance misses PhysX impulse risk | R97 case `2` passes with impulse `4466405` | Only one contact case is tested | Freeze bounded support only |
| H24/H26: nonlocal coupling is hidden between motor samples | R114-v2 lifts `69687` q/v/a; solver-free sparse assembly closes | 240 Hz effort state is still untested | Execute only the bounded R115 KTO stage |
| H25: reset mismatch causes R94 | Earlier reset concerns | R94 state is exact within quantization | Falsified; do not retry |

## Required context

Read these sources in precedence order before acting:

1. [Agent routing](../../architecture/agent-routing.md), especially the
   deterministic humanoid training, motor and roadmap rows.
2. [SPEC-35](../../architecture/35-deterministic-humanoid-training-substrate.md),
   [SPEC-34](../../architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md),
   [ADR-070](../../architecture/adr/070-biomechanics-reference-tracking-training-environment.md)
   and [ADR-071](../../architecture/adr/071-canonical-physics-material-lineage.md).
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

1. Freeze clean R73–R114 and all superseded semantic reports.
2. Retain R108's four stages; do not collapse or skip their gates.
3. Hash-bind, validate and execute exactly one clean bounded R115 KTO solve.
4. Run no ID/kinodynamic solve, candidate artifact, PhysX scene or all-17.

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

- **Workspace state:** Clean R114-v2 exists; R115 tooling is implemented and
  tracked research records R73–R114. Reports remain external/hash-bound.
- **Checks:** Full lab `235/235`; clean R114-v2 `COMPLETE`; every scene, solve,
  candidate artifact, PhysX, learned-optimizer and training count is zero.
- **Remaining risk:** KTO/dynamic feasibility,
  full-corpus exact-zero coverage and visual review remain open.
- **Promotion needed:** None for reset semantics: ADR-070 is retained. Any
  future attempt to admit indexed running-scene reset requires a superseding
  ADR and new evidence.
