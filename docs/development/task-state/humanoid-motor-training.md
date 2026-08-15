# Humanoid motor training rebuild — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE_R&D / TRAIN-4 / R137_FORMULATION_AUTHORIZED / REPORT_ONLY / NO_SOLVE_AUTHORITY` |
| Updated | 2026-08-15 |
| Task key | `humanoid-motor-training-rebuild` |
| Scope | Close `REQ-HUM-DATA-005/007` dynamic-reference feasibility before learned optimizer work |
| Definition of done | A hash-closed TRAIN-4 corpus/profile lineage passes the required optimizer-free gate and receives an evidence-backed decision |
| Authority | Working context only; Accepted SPEC/ADR, tracked profiles/manifests, exact gate artifacts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** Freeze R136; the selected successor is fixed-mode controller-reachable trajectory co-design, not changed contact semantics.
- **Why:** All `3200` equality systems pass rank/residual guards, yet only `782` are cone-feasible; every infeasible row has a negative friction margin and `1345` also have negative normal margin.
- **Next action:** Produce exactly one clean hash-closed R137 formulation report; it may inventory the symbolic graph but must perform zero reconstruction, controller derivation, numeric assembly or solve.
- **Current blocker:** The exact projected q/v plus frozen fixed-PD effort cannot satisfy the rigid unilateral friction cones broadly; `qdot=v`, release impulse and integrated feasibility remain unclaimed.
- **Do not retry:** Never restart R123/R127/R129/R130/R136, tune a frozen tolerance, substitute a witness, or start KTO/kinodynamics/PPO/PhysX.
- **Reconsider when:** Exact R137 `COMPLETE` may authorize only a separate report-only R138 implementation-conformance increment.

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
| R130–R137 decision | R130 is `INVALID`; R131–R135 close and conform the projected schedule/composition; sole R136 canonical `b522dc92062d3f760536669cc30a053f6c11d845dfac9e09f2865de597daf6d5` is valid but cone-infeasible at `2418/3200` rows | Fixed-mode co-design selected; exactly one report-only R137 formulation is authorized |
| Formal visual review | `PENDING` | No visual acceptance claim |

Current R133/R134/R135/R136 canonical SHA-256:
`f1fad2ca3c7abd49acaefd9fcd37873d02fb1d289a3081fd2039b0b1192fd3b6` /
`17d696166a05a24bd50a545ca31e4aabd15a72ed49444a09983c472ff6b13783` /
`21fd15693ad092158a9f1fd9d8d8f717cbea26eb38fdca6cd321b61ccc7b7525` /
`b522dc92062d3f760536669cc30a053f6c11d845dfac9e09f2865de597daf6d5`.
The [initial causal decision](../humanoid-train4-causal-research-2026-08-14.md)
and [bounded prototype decision](../humanoid-train4-v19-prototype-research-2026-08-14.md)
and [contact-boundary decision](../humanoid-train4-contact-boundary-research-2026-08-14.md)
and [support-authorization decision](../humanoid-train4-support-authorization-research-2026-08-14.md)
and [clip-global decision](../humanoid-train4-clip-global-research-2026-08-14.md)
and [coupled-solver decision](../humanoid-train4-coupled-trajectory-research-2026-08-14.md), the [native-dynamics decision](../humanoid-train4-native-dynamics-research-2026-08-14.md), [R123 redundant-contact research](../humanoid-train4-r123-redundant-contact-research-2026-08-15.md), [R127 consistency research](../humanoid-train4-r127-constraint-consistency-research-2026-08-15.md), [R130 projected-schedule research](../humanoid-train4-r130-projected-schedule-research-2026-08-15.md), [R136 cone-feasibility research](../humanoid-train4-r136-cone-feasibility-research-2026-08-15.md) and [R137 fixed-mode kinodynamic decision](../humanoid-train4-r137-fixed-mode-kinodynamic-research-2026-08-15.md)
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
  R77–R91 isolation, R93 offline PASS, R94–R97 native, R118 q/v conformance
  and R136 valid `2418/3200` pointwise cone infeasibility.
- **Decision:** V9 retains one complete-clip SQP over root XYZ and ten selected
  leg joints, adds stable rows only for the two foot boxes, and uses no
  root/joint post-projection. Every source contact point remains frozen;
  candidates require shared model/exact merit and explicit acceptance.
- **Rejected alternatives:** Point-entry semantics alone, blind post-scaling,
  dense category slack, mismatched row-L1/max merit, or looser safety limits.
- **Consequences:** NumPy/SciPy/OSQP are pinned private lab dependencies; final
  contact, collider, CoM, ROM and velocity facts are recomputed from emitted
  integer poses. The adapter has no runtime or corpus-admission authority.
- **Uncertainty:** Whether the fixed-mode exact-controller trajectory feasible
  set is nonempty under the unchanged rigid contact and descriptor limits.
- **Reconsider when:** R137 freezes the symbolic graph and a separate R138
  conformance proves its indexing/controller/event implementation; R136 cannot retry.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H22: exact offline reference is dynamically too demanding for fixed PD | R136 finds `2418/3200` cone-infeasible rows after every numeric guard passes | The fixed-mode integrated feasible set is still unknown | Supported for the frozen lineage; R137 formulation is the authorized next discriminator |
| H23: offline clearance misses PhysX impulse risk | R97 case `2` passes with impulse `4466405` | Only one contact case is tested | Freeze bounded support only |
| H24/H26: nonlocal coupling is hidden between motor samples | R122 reproduces the affine 240 Hz lift and local dynamics identities | Integrated dynamics remains untested | Keep blocked; invalid R123 cannot authorize R124 |
| H27: flat-foot point multipliers are structurally redundant | R123-RC1/R126 close gauge handling; R129 closes tangent projection | R127-RC1 proves a separate off-manifold q/v defect | Closed within exact claim |
| H28: affine q/v lift across contact exit causes actuator-invalid late projection | R132 lowers every changed-row correction; R133 passes the complete projected schedule with zero unsafe categories | The claim remains schedule-local and proves no integrated dynamics | Closed within the fixed-PD schedule claim |
| H29: the projected schedule is pointwise dynamically feasible | R122/R126 conform the kernel/solver; R133–R135 close the inputs and composition | R136 rejects it: only `782/3200` rows are cone-feasible | Rejected for the frozen lineage; no retry |

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

1. Freeze R123/R127/R129/R130/R136 and all superseded reports; never retry them.
2. Preserve the hash-closed R136 report/cache as research evidence only; it has no warm-start, candidate or corpus authority.
3. Build one report-only R137 fixed-mode controller-reachable kinodynamic formulation; perform zero real numeric work and stop before R138 or any solve.

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

- **Workspace state:** R136 evidence is immutable valid `INFEASIBLE`; the R137 fixed-mode report-only decision is tracked and R130 remains immutable `INVALID`.
- **Checks:** All six R136 validations PASS (`369/369` lab, `56/56` motor, host); all `3200` numeric rows are valid and the canonical report hash independently reproduces.
- **Remaining risk:** The repair family, integrated/fresh PhysX, full-corpus exact-zero coverage and visual review remain open.
- **Execution authority:** R137 formulation-only. No reconstruction, controller derivation, numeric assembly, R138 conformance, solve, candidate, PhysX or training is authorized yet.
- **Promotion needed:** None for reset semantics: ADR-070 is retained. Any
  future attempt to admit indexed running-scene reset requires a superseding
  ADR and new evidence.
