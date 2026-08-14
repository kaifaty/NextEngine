# TRAIN-4 coupled complete-clip trajectory research

| Field | Value |
| --- | --- |
| Date | 2026-08-14 |
| Scope | Optimizer-free learned-policy lane; offline constraint-solver research for `REQ-HUM-DATA-005/007` |
| Status | `CMU05 SINGLE-INVOCATION PASS / V8 STRONGER MASK IMPLEMENTED / CLEAN ALL-CLIP EVIDENCE PENDING` |
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

The repository now contains the V8 profile, complete-clip builder dispatch,
exact-slice path and focused tests. The production path recomputes final FK,
effectors, center of mass and collider facts from emitted integer root/joint
values, and rejects any V8 profile that changes a frozen contact, collider,
root or joint bound. This implementation and the dirty-worktree R72 proof do
not themselves pass TRAIN-4. The next accepted evidence sequence is:

1. from one clean commit, reproduce direct-source `cmu05` in one invocation;
2. apply the identical V8 profile to `cmu16` and `cmu139` with no per-clip
   tuning;
3. require all three complete clips, all 17 exact slices and overlap identity
   to pass offline;
4. only then run fresh-scene all-17 PhysX acceptance, retaining partial reset
   as report-only.

Any complete-clip failure returns to bounded solver research. Full 27-clip
V19, visual/exhaustive admission, TRAIN-4 Advance, TRAIN-5 and training remain
forbidden until the ordered gates pass.
