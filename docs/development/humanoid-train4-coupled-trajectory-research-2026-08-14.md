# TRAIN-4 coupled complete-clip trajectory research

| Field | Value |
| --- | --- |
| Date | 2026-08-14 |
| Scope | Optimizer-free learned-policy lane; offline constraint-solver research for `REQ-HUM-DATA-005/007` |
| Status | `V8 CLEAN CMU05/CMU16 PASS / CMU139 GLOBALIZATION FAILURE / MODEL-VALID TRUST RE-SOLVE` |
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
| R82 exact-row hard second-order correction | The first two capped primary steps reproduce the useful R80 path (`6.0602/18.0222 -> 4.7570/15.2353 -> 3.3690/12.7778`). The third full step lowers total violation to `9.5300` but raises the worst component to `3.7556`, so max-first acceptance correctly routes it to correction. The bounded correction with the original `58488`-row Jacobian and nonlinear trial-state bounds is `primal infeasible` after `16925` OSQP iterations; no corrected exact state is evaluated. | Reject hard all-row SOC as sufficient, not the second-order diagnosis. The report remains `FAIL`, the pending trial/candidate is non-admissible, and the last accepted state remains the second primary point. Next isolate artificial infeasibility with iteration-only normalized nonlinear restoration slack; final exact-zero audit remains unchanged. |
| R83 minimax nonlinear phase-I restoration | Relaxing only nonlinear rows makes the correction model feasible. Four bounded bisections bracket minimum normalized slack between `0.235364` infeasible and `0.470727` solved; linear/trust rows remain hard. The selected model balances every nonlinear group near `0.4707` and improves exact contact violations, but its `50 mm` root-saturated correction drives exact collider from baseline violation `3.1438` to `8.8592` (`-44298 µm`). Exact max-first audit rejects it and restores the baseline. | Reject the old-Jacobian minimax restoration identity. Its model predicts collider violation `0.4707` while integer FK measures `8.8592`, directly localizing stale linearization over the correction radius. Recompute the correction Jacobian at the nonlinear trial point before changing slack, trust or requirements. |
| R84 hard trial-Jacobian restoration | The correction uses the Jacobian already evaluated at the quantized rejected trial, with identical `252303` base nonzeros, nonlinear row bounds and component trust. It remains `primal infeasible` after `66000` iterations with primal residual `0.06245`; no corrected exact state is evaluated. | Trial-point relinearization alone cannot remove local incompatibility. R83 and R84 isolate two independent requirements: trial geometry for exact fidelity and iteration-only phase-I slack for model feasibility. Test their direct composition before considering another mechanism. |
| R85 trial-Jacobian minimax phase-I | The direct R83/R84 composition makes all tested slack levels feasible; the selected `0.235364` solve reaches `solved inaccurate` at `100000` iterations and balances trial-model nonlinear rows near `0.2354`. Exact contact violations improve, and total violation drops `12.7778 -> 11.9054`, but collider violation rises `3.1438 -> 6.6700` (`-33352 µm`). Exact max-first rejects the correction and restores the baseline. | Reject this local composition as an acceptance mechanism. Trial relinearization improves R83 collider error `8.8592 -> 6.6700` but still underpredicts it by a large margin over a `1.7887` normalized correction. Stop local solver-knob composition and map exact/model error versus frame and step scale before choosing a new geometry or trust model. |
| R86 directional model-fidelity audit | The hash-bound R85 direction reproduces the reference exact violations with zero normalized delta. At scales `0.125` and `0.25`, model/exact worst merits are `3.3245/3.2958` and `2.8832/3.1418`; both improve the accepted `3.3690` baseline, with actual/predicted ratios `1.042` and `0.695`. At `0.5`, the ratio becomes `-0.390`, exact merit regresses to `4.4442` and maximum collider prediction error grows to `13030 µm`. The dominant error is `collider.right-foot`, especially source frames `1383..1392`, worst at `1388`. | Accept only a measured local model-valid interval, not any audited scale as a candidate. A quarter direction uses at most `8944 µm` root component / `9555 µm` root norm / `24992 µrad` joint component, while the half direction is outside reliable geometry. R87 must re-solve the trial-point phase-I inside `10000 µm` root-component and `25000 µrad` joint-component trust bounds; it must not scale the stored R85 direction. |

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
R82 report/candidate/research-adapter SHA-256 are
`c6f8db607228b081594bc8cc43b87a05477ecfb2290c9dd5f95b543fa666a44e` /
`f0240e47757f1bac3394045f6287bd15d111d19e376fe7ab6651a9ec2e5cb4ea` /
`a13860b1be3a2f802bc80f0eba54fd4bd750e291b8627315b933018097f993f2`.
The candidate captures the rejected pending trial after correction solver
failure and is explicitly non-admissible; the report's
`final_accepted_exact_violation` identifies the retained baseline.
R83 report/candidate/research-adapter SHA-256 are
`7d8da4fe0a17510a65aa818fa551132b0ce881b6d6f69e7bfac0fcc1b0285c0b` /
`8f40a26c7a896700cc1047b3dbe44e2b5cd3bdb82f1ff18d0efa22e9a4642460` /
`339bf7cd7085caabbc80089e5a2d895dc84a0528f1adf26ef3c16780f6e09604`.
The R83 candidate is the restored accepted baseline, not the rejected
correction, and remains non-admissible because the unchanged gate is `FAIL`.
R84 report/candidate/research-adapter SHA-256 are
`513b52aa1ff500060962fe68dd910494a43b9e369bf52105077b1a369a144485` /
`f0240e47757f1bac3394045f6287bd15d111d19e376fe7ab6651a9ec2e5cb4ea` /
`1e66abebc8a01c890c93ae969af6771aa3a4403f2ab5188709f222da18340f06`.
The R84 candidate again captures the pending rejected primary trial after
correction solver failure and is explicitly non-admissible.
R85 report/candidate/research-adapter SHA-256 are
`228117a508ff22c9f3a5927862af8b79a40635446c3b02a305651231aeb69adf` /
`8f40a26c7a896700cc1047b3dbe44e2b5cd3bdb82f1ff18d0efa22e9a4642460` /
`bc23a1535deb1e5235ef312e5903f6d1b490d97fba85bd1a6a34a30f0a5732f1`.
The R85 candidate is the restored accepted baseline and remains
non-admissible because the unchanged gate is `FAIL`.
R86 report/direction/research-adapter SHA-256 are
`18e395edb33ed80a6f8946988e377f401943bdad4523813f648d42429eb15036` /
`23cdcfe5ec5ccc9d080cfdf893b1563976b4516a94c256fe86f013f535f3c515` /
`92676ca3cbb3d3351224d8922a207b537388db1a86623b0eeaa0fdbf74570f2f`.
R86 emits no candidate and has `candidate_status=NOT_EVALUATED`; every scale
is a research measurement rather than admissible TRAIN-4 evidence.

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

R82 tested the hard form of the standard second-order correction without an
explicit Hessian. Shared row identity stayed stable, and exact max-first
acceptance routed the expected third trial into correction, but the old
Jacobian cannot satisfy the full trial-state row set inside the retained
correction trust. This is direct evidence of artificial infeasibility inside the restoration step,
not evidence that the unchanged physical clip is infeasible.

R83 then tested lexicographic phase-I restoration. Sparse bound relaxation
avoids the dense R77/R78 encodings and proves a normalized nonlinear slack can
remove artificial infeasibility while linear and trust rows stay hard. It also
provides a stronger discriminator: the selected old-Jacobian model limits the
collider violation to `0.4707`, but exact integer FK measures `8.8592`. The
correction therefore leaves the validity radius of that Jacobian even though
the convex subproblem itself is solved accurately.

R84 reused the already computed Jacobian at the rejected nonlinear trial point
for one hard bounded correction. This removes the stale-geometry variable but
does not remove artificial infeasibility: the correction fails before exact
audit. Together R83/R84 now form a controlled two-factor result rather than an
invitation to tune another scalar.

R85 directly composed the R84 trial-point Jacobian and unchanged R83 phase-I.
It halves the selected model slack and reduces the exact collider regression,
but the corrected FK still disagrees qualitatively with a tightly balanced
linear model. This is the predeclared stop condition for local SOC/restoration
composition, not a reason to add a fifth scalar or relax the exact gate.

R86 completed that predeclared discriminator. The exact/model relationship is
reliable through a quarter of the stored direction and reverses before half:
the worst-merit actual/predicted ratio changes from `0.695` at `0.25` to
`-0.390` at `0.5`. Collider prediction error grows nonlinearly at the right
foot around source frame `1388`, from `2485 µm` to `13030 µm` over the
same interval. This supports an adaptive local trust contract and rejects both
the original `50 mm / 100 mrad` correction and blind post-scaling.

R87 is therefore a fresh trial-Jacobian minimax phase-I solve with root
component trust `10000 µm` and joint component trust `25000 µrad`. Those
bounds cover the largest exact-improving audited quarter direction but exclude
the first sign-reversing half direction. R87 must recompute the QP inside those
bounds, retain hard linear rows and unchanged exact max-first audit, and report
the actual/model ratio. Scaling or admitting the stored R85 direction is not
allowed. An exact-improving R87 result only justifies adaptive
relinearization; it does not pass `cmu139` or advance TRAIN-4.

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
