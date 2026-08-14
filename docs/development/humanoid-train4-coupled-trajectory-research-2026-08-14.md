# TRAIN-4 coupled complete-clip trajectory research

| Field | Value |
| --- | --- |
| Date | 2026-08-14 |
| Scope | Optimizer-free learned-policy lane; offline constraint-solver research for `REQ-HUM-DATA-005/007` |
| Status | `V8 CLEAN CMU05/CMU16 PASS / CMU139 GLOBALIZATION FAILURE / SECOND-ORDER RESTORATION RESEARCH` |
| Current implementation | `nextengine.dimensionless-contact-trajectory-qp.v8` |
| Claim ceiling | Research infrastructure only; no V19, TRAIN-4 Advance, visual gate or learned optimization |

## Frozen boundary

The cycle retained active tangential displacement `2000 µm/frame`, active
normal displacement `1000 µm/frame`, active normal residual `5000 µm`, the
collider floor `-2 µm`, joint-velocity reserve `2500` basis points and root
vertical velocity `200060 µm/s`. It added no grace window, settling interval,
velocity zeroing, contact-point deletion, ROM/impact relaxation or post-solve
safety projection. Fresh scenes remain acceptance authority; indexed partial
reset remains report-only. Learned optimizer steps and training runs remain
zero.

The accepted offline question was narrower: can one complete immutable clip
be changed by one bounded root/leg trajectory solve so contact pose, finite and
analytic contact velocity, every collider sample, soft ROM and root/joint
velocity all pass simultaneously?

## Research sequence

All generated artifacts remain under the external TRAIN-4 evaluation root.

| Evidence | Result | Disposition |
| --- | --- | --- |
| R58 per-frame coupled Gauss-Newton | Root/contact corrections spike and do not preserve the temporal envelopes. | Reject independent frame solves. |
| R59 global normal equations | Improves several residuals but stagnates before simultaneous feasibility. | Keep the clip-global variable domain; reject weighted least squares as the constraint mechanism. |
| R60 stronger weights plus hard root closure | The post root operation repairs its own bound and re-breaks contact. | Reject every sequential root/joint post-pass. |
| R61 contact/joint baseline | Contact passes (`4999 µm` residual, `574/430 µm` finite, `573/457 µm` analytic) and joint reserve is `2500 bp`, but collider minimum is `-30748 µm` and root vertical velocity is `478470 µm/s`. | Proves contact plus joint feasibility, not complete feasibility. |
| R62 simultaneous weighted inequalities | Contact, collider, root and joint facts still trade against each other after a hard projection. | Reject penalty-only active constraints. |
| R63 all-collider coupled solve | Stable but stalls at collider `-30100 µm` and root `469680 µm/s`. | The active set is correct; scaling/solver is not. |
| R64–R67 root envelope, active corridor and smaller line search | Feasible directions collapse to tiny steps; R67 accepts steps as small as `1/512` while useful metrics barely improve. | Reject line-search tuning; normalize variables and rows explicitly. |
| R68 dimensionless sparse QP | Reaches contact `4999/1999/999`, but quantization leaves collider `-57 µm`, root `200100 µm/s` and joint `2501 bp`. | Feasible mechanism confirmed; add only internal quantization reserve. |
| R69 reserved sparse QP from R61 | Complete `cmu05` passes: residual `4900`, finite tangent/normal `1978/980`, analytic `1968/996`, collider `+30`, joint `2500`, root `199800`; zero dropped points. | First simultaneous complete-clip proof. |
| R70 raw canonical clip, eight outer iterations | Contact passes and all velocity/ROM facts pass; thirteen collider samples remain, minimum `-146 µm`. | R61 is not a semantic input, but eight direct iterations are insufficient. |
| R71 exact R70 continuation | Two more relinearizations reach collider `+1 µm` and complete PASS while contact and velocity facts remain within bounds. | Freeze a direct-source V8 candidate with at most twelve outer iterations; require a single-invocation reproduction before evidence promotion. |
| R72 raw canonical clip, one invocation with at most twelve iterations | Stops on exact PASS at iteration ten: residual `4900`, finite tangent/normal `1981/980`, analytic `1968/996`, collider `+1`, joint `2500`, root `199740`; zero dropped points. An independent FK audit of the emitted integers gives residual `4901`, finite normal `981` and collider `0`, still PASS. | Confirms the direct-source iteration bound and exposes that final metrics must be recomputed from emitted quantized root/joint values. R72 ran from a dirty research worktree, so clean-commit all-clip evidence remains required. |
| R73 clean V8 all-three/all-17 invocation | `cmu05` passes at iteration ten and `cmu16` at iteration five. The stronger raw `cmu139` mask takes a `231006 µm` first root step, reaches collider `-34056 µm` and residual `31581 µm`, then OSQP reports primal infeasible. | Reject V8 as an all-clip solver. The clean run confirms two controls and localizes the blocker to `cmu139`; no fresh PhysX run is authorized. |
| R74 point-entry stencil on raw `cmu139` | The 16 point-level edge differences are real, but the second QP is still primal infeasible. | Keep the semantic observation; reject it as sufficient remediation. |
| R75 applied-step cap, original V8 stencil | All twelve QPs remain feasible. Final collider is `-5682 µm` and residual `5811 µm`; collider alternates between approximately `-7..-9 mm` and `-20..-27 mm`. | A bounded step removes artificial infeasibility, but blind acceptance creates an active-set oscillation. |
| R76 point-entry plus applied-step cap | All twelve QPs remain feasible, but final residual/tangent/normal are `6296/2594/1078 µm` and collider is `-2732 µm`; alternate iterations still regress collider to `-26..-32 mm`. | Reject the combined semantic hypothesis. The remaining defect is step globalization, not a missing scalar or point-edge switch. |
| R77 full group-elastic in-QP trust | The first trial improves exact normalized merit `18.0222 -> 14.09237`, but actual/predicted ratio `0.204` correctly rejects it and shrinks the radius. Duplicating all row sides then reaches the OSQP iteration limit. | Exact rejection behaves as intended; reject the dense/full elastic encoding. |
| R78 selective category-elastic trust | The first trial is accepted at ratio `0.274` and merit `12.8099`, but temporarily creates collider `-33386 µm`; the next QP reaches the iteration limit. | Global category slack columns destroy useful clip-local sparsity and are not a production encoding. |
| R79 selective per-row L1 elastic trust | The first direction worsens gate-aligned max merit `18.0222 -> 24.4408`; both model and exact audit reject it. | A row-average L1 objective is not interchangeable with the current max-based acceptance metric. Stop representation tuning until model and exact merit are identical. |
| R80 exact max/sum backtracking | Five accepted hard-QP steps reduce worst normalized violation `6.0602 -> 2.4908` and total violation `18.0222 -> 9.5412`, eliminating the R75/R76 alternation. Acceptance nevertheless contracts from factor `1` to `0.5`, `0.125` and the minimum `0.0625`; final collider/residual remain `-12456/17402 µm`. Rejected full steps at trials 3–5 reduce total violation to `9.5286`, `8.7025` and `8.9733` while temporarily increasing the worst component. | Partial discriminator, not a candidate. Monotone max-first backtracking prevents oscillation but stalls at contact curvature and discards useful non-dominated steps. Stop step-factor tuning; test an exact filter, then second-order correction/restoration if rejection persists. |
| R81 exact worst/total feasibility-filter observation | The first two full candidates improve both measures. The filter then accepts the R80-rejected full third step (`3.3688/12.7776 -> 3.7554/9.5286`) and another non-dominated step (`3.7554/9.5286 -> 5.9160/8.8038`). Thus it crosses the max-first plateau, but permits worst violation to return close to the immutable source ceiling `6.0602`. The run was stopped between QPs before a report/candidate was emitted. | Support the filter diagnosis; reject this two-violation filter as a solver identity. It is an explicitly non-promotable interactive observation, not evidence. Move to exact-row second-order restoration rather than spend twelve more `100000`-iteration QPs on the permissive filter. |

R69 report SHA-256 is
`4e840f9f9d91f4b13ffbda8f23ab33b2158f61ad87bcdd1e12c6932ae9606b56`;
candidate SHA-256 is
`81c659703304db6341125011901ae98daab67afcceb51008f75801d3400c0266`.
R70 report/candidate SHA-256 are
`047b09c418ee33273ebcba0776a16d861bb7bba42085d5496b406f1f5b4dec80` /
`c8abd88d17673de2bc44411e1fdf3af607415e3dd6421fd5af1a78bca5d283df`.
R71 report/candidate SHA-256 are
`8cde1b6c210efdb6f2f9b3205d426de25817b8016a6c4c5b77d63a0704486f73` /
`917fccc2f7737bc29144411e1fdf3af607415e3dd6421fd5af1a78bca5d283df`.
R72 report/candidate SHA-256 are
`04247d340c0e533facdf71acd7a3c026ccd8a8f139b57340c7210ff04b5e7b5d` /
`10acecd2e9adad6cd850d906e9807d93059a16b9f62266f65913ab20b05e884e`;
the research oracle SHA-256 is
`81f7265a7bc706dba3f31f302ede95191cd3da0c75fbac4bfb55c2a27afe2491`.

R73 manifest SHA-256 is
`d0b3897545af22bfefa68e69562eb27e5bfc182b240e09baa12325d0ab31d37c`.
R74 report/candidate SHA-256 are
`c8cfcb6d52f80ef7c68b34bf0ab602a6919ece6cb2bee1cc0e5137c6c8710b72` /
`fe8a61023bac2d35eadec1488cd92761055d0d66243a3c1dd94335a363afc840`.
R75 report/candidate SHA-256 are
`8e14416db70286873230e42c0419080e4799f66aafddf7b2bde81d2cddc82c86` /
`5fbab9156e76981c88c6dbd9ba3168d04fa0a37eff081910bff3b9588796b433`.
R76 report/candidate SHA-256 are
`6e5093796cf1d786285813fef92c21682cb129e3dd0cd5ec3bf046b13590d00c` /
`32d62f740151a462ce8faef379752b0029fe9268122904d782efed5fc81733d1`.
R77/R78/R79 report SHA-256 are respectively
`acedf2f3fe68d98fe89cb0fbd5e7a7c5ec7f4d07db7e1e9d28c0571d16e83fa`,
`ff844f676a52f01faf1ac38b200346db64c9328f2bca23d30deed1d02f69bd87`
and `53b2d262b8843eb8ecbec66c9f4d14386d57a9093e0e0d045a09cf084a7ab933`.
R80 report/candidate/research-adapter SHA-256 are
`23d73e47b70c236079ea1d52f67e81de7313433b795e06f69525e06c3e90a60e` /
`570ae002ade352d5cc6e5fcb99ca6db2ca48871bf73f180ecc5d187a3688677c` /
`157cc9fc9ee296fb07375916b2e5b7b3f6abc24664792f6a1c87930c2cffeb88`.
The run was deliberately interrupted while solving trial six after the
minimum factor had already been consumed; its report `FAIL` and
`solver_failure/interrupted` termination are retained rather than relabelled.
The non-promotable R81 adapter SHA-256 is
`34f8ffcadaea0e51d05a512fcd6741aaa8379310caccd0cf38273c7f054fb0ea`.
R81 has no report/candidate hash and cannot be cited as acceptance evidence.

## Primary-source research decision

The observed failure is a known globalization failure of sequential convex
optimization, not evidence that the unchanged physical problem is infeasible:

- [TrajOpt](https://escholarship.org/uc/item/6km506db) places a box trust
  region inside each convex subproblem, converts infeasible constraints to
  L1 penalties, and accepts a step from true/model improvement rather than
  applying every convex answer.
- [SCvx](https://arxiv.org/abs/1608.05133) names the two matching defects
  *artificial infeasibility* and *approximation error*. It uses penalized
  virtual controls plus an adaptive trust region and rejects/re-solves when
  actual reduction disagrees with the linear prediction.
- [CRISP](https://arxiv.org/abs/2502.01055) is the closest contact-planning
  analogue: an L-infinity trust region is part of the QP, constraint penalties
  are individual, the radius follows actual/predicted merit reduction, and a
  second-order correction is reserved for persistent Maratos-effect rejection.
- [Trust-region SQP-filter methods](https://doi.org/10.1137/S1052623499357258)
  provide the alternative when one arbitrary penalty sum is not defensible:
  objective quality and constraint violation are filtered separately, with a
  feasibility-restoration path.
- [Contact Trust Region](https://arxiv.org/abs/2505.02291) shows that a
  symmetric geometric trust region can be inconsistent with unilateral
  contact. That is a secondary escalation if a correctly globalized V8 still
  fails at heel/forefoot entry; R74 proves it is not the first fix.
- [ContactIPM](https://arxiv.org/abs/2608.11731) demonstrates elastic contact
  relaxation with termination gated by the unrelaxed physical residual. Its
  full contact-implicit MPCC formulation would change the frozen-mode scope
  and is therefore a fallback, not the current intervention.

The immediate conclusion is narrower than adopting any paper wholesale. V8
lacks a globalization contract: it solves a hard local feasibility QP and
blindly applies the whole answer. R75/R76 show that bounding the applied step
prevents false infeasibility but not oscillation. R77/R78 show that an exact
actual/predicted rejection rule detects bad steps. R79 shows that the QP and
the exact audit must use the same violation functional; category-max and
row-average penalties cannot be mixed.

R80 supplied the missing discriminator without closing the clip. Exact
max-first acceptance makes monotone progress, unlike rejected R64–R67 weighted
least-squares line search, but reaches the minimum factor with all dominant
contact categories still roughly balanced at `2.3..2.49` times tolerance.
More importantly, its rejected full steps reduce total violation by more than
the accepted small steps while temporarily moving the worst category. That is
the signature for which a filter is preferable to a single scalar or
lexicographic merit.

R81 confirms that a filter can retain a useful non-monotone step, but also
rejects the naive choice of worst and total violation as its two coordinates.
Both coordinates measure feasibility, so reducing the total can conceal a
large regression in the exact-zero bottleneck. This is an evidence-specific
observation, not a claim that the SQP-filter paper prescribes those measures.

The smallest next discriminator, R82, is the standard second-order
correction/restoration pattern without explicit Hessians: evaluate the exact
per-row nonlinear error at the rejected trial point, solve one bounded
minimum-correction QP against that error with the shared row identity, and
accept only the corrected emitted integer-FK state. A temporary restoration
slack remains iteration-local; the unchanged final audit still requires zero
violations. Do not tune another stencil, cap, factor or penalty scalar.

## V8 solver identity

V8 starts from the exact V18 complete source clip and freezes every
point-consistent mode inferred from that source before solving. Production
review found that R57 had already removed `11` initially inferred active
points from `cmu139`: frames `628..629` and `724..732`. The first pair overlaps
the selected `@626/@627/@630` discriminator region. R69 used the already
reduced R57 mask and could not expose this because `cmu05` has no such mask
difference. V8 therefore does not inherit the R57 mask and gives the solver no
point-deletion operation. This is a stricter all-clip test, not a changed
tolerance.

V8 has `13` variables per frame: root XYZ and five selected joints on each
leg. Every SQP outer iteration constructs one dimensionless sparse QP with
explicit rows for:

- active normal residual;
- shared-active finite X/Y/Z displacement;
- hybrid-stencil analytic X/Y/Z contact velocity;
- every collider sample against the common ground floor;
- root vertical velocity;
- selected-joint velocity and soft ROM.

The hybrid stencil is forward at contact entry, backward at exit and centered
otherwise, with entry precedence. There is no root closure or joint-velocity
projection after the QP. Internal margins are stricter targets, not changed
requirements: collider `+50 µm`, root velocity reserve `300 µm/s`, joint
velocity factor `999/1000`, normal residual `100 µm`, finite normal/tangent
`20/20 µm`, analytic normal/tangent `300/2000 µm/s`.

The private lab adapter is pinned to NumPy `2.5.2`, SciPy `1.18.0` and OSQP
`1.1.3`. These are replaceable preprocessing dependencies and have no runtime,
physics, corpus-admission or public-schema authority. `optimizer_steps=0`
continues to mean no learned-policy optimizer; the QP is reported separately
as a bounded trajectory constraint solve.

## Decision and remaining gate

The repository contains the V8 profile, complete-clip builder dispatch,
exact-slice path and focused tests. The production path recomputes final FK,
effectors, center of mass and collider facts from emitted integer root/joint
values, and rejects any V8 profile that changes a frozen contact, collider,
root or joint bound. Clean R73 proves that implementation is not yet an
all-clip solution. The next accepted evidence sequence is:

1. close `cmu139` with one shared, evidence-backed globalization identity and
   no per-clip tuning, point deletion or changed physical limit;
2. rerun that unchanged identity on all three complete clips, all 17 exact
   slices and overlap identity from one clean commit;
3. only then run fresh-scene all-17 PhysX acceptance, retaining partial reset
   as report-only.

Any complete-clip failure returns to bounded solver research. Full 27-clip
V19, visual/exhaustive admission, TRAIN-4 Advance, TRAIN-5 and training remain
forbidden until the ordered gates pass.
